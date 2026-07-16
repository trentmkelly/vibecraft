//! Java-parity `ProfileResolver` and its two bounded, access-expiring caches.

#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

use crate::user_name_to_id_resolver::{is_valid_player_name, UserNameToIdResolver};

const CACHE_EXPIRATION: Duration = Duration::from_secs(10 * 60);
const CACHE_MAXIMUM_SIZE: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedGameProfile {
    pub id: String,
    pub name: String,
}

pub trait MinecraftSessionService {
    fn fetch_profile(&self, id: &str, require_secure: bool) -> Option<ResolvedGameProfile>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameOrId {
    Name(String),
    Id(String),
}

pub trait ProfileResolver {
    fn fetch_by_name(&mut self, name: &str) -> Option<ResolvedGameProfile>;
    fn fetch_by_id(&mut self, id: &str) -> Option<ResolvedGameProfile>;

    fn fetch_by_name_or_id(&mut self, name_or_id: NameOrId) -> Option<ResolvedGameProfile> {
        match name_or_id {
            NameOrId::Name(name) => self.fetch_by_name(&name),
            NameOrId::Id(id) => self.fetch_by_id(&id),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    last_access: Instant,
    sequence: u64,
}

#[derive(Debug, Clone)]
struct AccessCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    sequence: u64,
}

impl<K, V> Default for AccessCache<K, V> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            sequence: 0,
        }
    }
}

impl<K, V> AccessCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn get(&mut self, key: &K, now: Instant) -> Option<V> {
        let entry = self.entries.get_mut(key)?;
        if now.duration_since(entry.last_access) >= CACHE_EXPIRATION {
            self.entries.remove(key);
            return None;
        }
        self.sequence = self.sequence.wrapping_add(1);
        entry.last_access = now;
        entry.sequence = self.sequence;
        Some(entry.value.clone())
    }

    fn insert(&mut self, key: K, value: V, now: Instant) {
        self.sequence = self.sequence.wrapping_add(1);
        self.entries.insert(
            key,
            CacheEntry {
                value,
                last_access: now,
                sequence: self.sequence,
            },
        );
        if self.entries.len() > CACHE_MAXIMUM_SIZE {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.sequence)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&oldest);
            }
        }
    }
}

pub struct CachedProfileResolver<S, U> {
    session_service: S,
    name_to_id_cache: U,
    profile_cache_by_name: AccessCache<String, Option<ResolvedGameProfile>>,
    profile_cache_by_id: AccessCache<String, Option<ResolvedGameProfile>>,
}

impl<S, U> CachedProfileResolver<S, U>
where
    S: MinecraftSessionService,
    U: UserNameToIdResolver,
{
    pub fn new(session_service: S, name_to_id_cache: U) -> Self {
        Self {
            session_service,
            name_to_id_cache,
            profile_cache_by_name: AccessCache::default(),
            profile_cache_by_id: AccessCache::default(),
        }
    }

    fn fetch_by_id_at(&mut self, id: &str, now: Instant) -> Option<ResolvedGameProfile> {
        let key = id.to_string();
        if let Some(profile) = self.profile_cache_by_id.get(&key, now) {
            return profile;
        }
        let profile = self.session_service.fetch_profile(id, true);
        self.profile_cache_by_id.insert(key, profile.clone(), now);
        profile
    }

    fn fetch_by_name_at(&mut self, name: &str, now: Instant) -> Option<ResolvedGameProfile> {
        let key = name.to_string();
        if let Some(profile) = self.profile_cache_by_name.get(&key, now) {
            return profile;
        }
        let profile = self
            .name_to_id_cache
            .get_by_name(name)
            .and_then(|name_and_id| self.fetch_by_id_at(&name_and_id.uuid, now));
        self.profile_cache_by_name.insert(key, profile.clone(), now);
        profile
    }
}

impl<S, U> ProfileResolver for CachedProfileResolver<S, U>
where
    S: MinecraftSessionService,
    U: UserNameToIdResolver,
{
    fn fetch_by_name(&mut self, name: &str) -> Option<ResolvedGameProfile> {
        if is_valid_player_name(name) {
            self.fetch_by_name_at(name, Instant::now())
        } else {
            None
        }
    }

    fn fetch_by_id(&mut self, id: &str) -> Option<ResolvedGameProfile> {
        self.fetch_by_id_at(id, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player_access::NameAndId;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/players/ProfileResolver.java");

    #[derive(Default)]
    struct Session {
        profiles: HashMap<String, ResolvedGameProfile>,
        lookups: RefCell<Vec<String>>,
    }

    impl MinecraftSessionService for Session {
        fn fetch_profile(&self, id: &str, require_secure: bool) -> Option<ResolvedGameProfile> {
            assert!(require_secure);
            self.lookups.borrow_mut().push(id.to_string());
            self.profiles.get(id).cloned()
        }
    }

    #[derive(Default)]
    struct NameCache {
        profiles: HashMap<String, NameAndId>,
        lookups: RefCell<Vec<String>>,
    }

    impl UserNameToIdResolver for NameCache {
        fn add(&mut self, profile: NameAndId) {
            self.profiles.insert(profile.name.to_lowercase(), profile);
        }

        fn get_by_name(&mut self, name: &str) -> Option<NameAndId> {
            self.lookups.borrow_mut().push(name.to_string());
            self.profiles.get(&name.to_lowercase()).cloned()
        }

        fn get_by_id(&mut self, id: &str) -> Option<NameAndId> {
            self.profiles.values().find(|profile| profile.uuid == id).cloned()
        }

        fn resolve_offline_users(&mut self, _value: bool) {}
        fn save(&self) {}
    }

    fn profile(id: &str, name: &str) -> ResolvedGameProfile {
        ResolvedGameProfile {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn profile_resolver_source_matches_java_cache_contract() {
        for fragment in [
            "Optional<GameProfile> fetchByName(String name)",
            "Optional<GameProfile> fetchById(UUID id)",
            "fetchByNameOrId",
            "expireAfterAccess(Duration.ofMinutes(10L))",
            "maximumSize(256L)",
            "sessionService.fetchProfile(profileId, true)",
            "StringUtil.isValidPlayerName(name)",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn cached_resolver_fetches_by_id_once_and_by_name_through_name_cache() {
        let id = "069a79f4-44e9-4726-a5be-fca90e38aaf5";
        let mut session = Session::default();
        session.profiles.insert(id.to_string(), profile(id, "Notch"));
        let mut names = NameCache::default();
        names.add(NameAndId {
            uuid: id.to_string(),
            name: "Notch".to_string(),
        });
        let mut resolver = CachedProfileResolver::new(session, names);

        assert_eq!(resolver.fetch_by_id(id), Some(profile(id, "Notch")));
        assert_eq!(resolver.fetch_by_id(id), Some(profile(id, "Notch")));
        assert_eq!(resolver.fetch_by_name("NOTCH"), Some(profile(id, "Notch")));
        assert_eq!(resolver.fetch_by_name("Notch"), Some(profile(id, "Notch")));
        assert_eq!(&*resolver.session_service.lookups.borrow(), &[id.to_string()]);
        assert_eq!(
            &*resolver.name_to_id_cache.lookups.borrow(),
            &["NOTCH", "Notch"]
        );
    }

    #[test]
    fn cached_resolver_rejects_invalid_names_and_caches_negative_profiles() {
        let mut resolver = CachedProfileResolver::new(Session::default(), NameCache::default());
        assert_eq!(resolver.fetch_by_name("bad\nname"), None);
        assert_eq!(resolver.fetch_by_name("Missing"), None);
        assert_eq!(resolver.fetch_by_name("Missing"), None);
        assert_eq!(&*resolver.name_to_id_cache.lookups.borrow(), &["Missing"]);
    }

    #[test]
    fn name_or_id_dispatch_matches_java_either_mapping() {
        let id = "00000000-0000-0000-0000-000000000001";
        let mut session = Session::default();
        session.profiles.insert(id.to_string(), profile(id, "Alex"));
        let mut names = NameCache::default();
        names.add(NameAndId {
            uuid: id.to_string(),
            name: "Alex".to_string(),
        });
        let mut resolver = CachedProfileResolver::new(session, names);
        assert_eq!(resolver.fetch_by_name_or_id(NameOrId::Name("Alex".to_string())), Some(profile(id, "Alex")));
        assert_eq!(resolver.fetch_by_name_or_id(NameOrId::Id(id.to_string())), Some(profile(id, "Alex")));
    }
}
