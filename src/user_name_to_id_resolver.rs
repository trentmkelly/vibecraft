//! Java-parity user-name/profile cache.
//!
//! This module ports `UserNameToIdResolver` and
//! `CachedUserNameToIdResolver`. The Java implementation maintains indexes by
//! lower-cased name and UUID, refreshes missing names through the profile
//! repository, and persists at most the 1,000 most-recently-used profiles.

#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chrono::{DateTime, Datelike, Local, Months, TimeZone};
use serde_json::{Map, Value};

use crate::player_access::NameAndId;

const MRU_LIMIT: usize = 1_000;
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// Rust equivalent of Java's `UserNameToIdResolver` interface.
pub trait UserNameToIdResolver {
    fn add(&mut self, profile: NameAndId);
    fn get_by_name(&mut self, name: &str) -> Option<NameAndId>;
    fn get_by_id(&mut self, id: &str) -> Option<NameAndId>;
    fn resolve_offline_users(&mut self, value: bool);
    fn save(&self);
}

/// The single operation used from Mojang's `GameProfileRepository`.
pub trait GameProfileRepository {
    fn find_profile_by_name(&self, name: &str) -> Option<NameAndId>;
}

#[derive(Debug, Clone)]
struct GameProfileInfo {
    profile: NameAndId,
    expiration_date: DateTime<Local>,
    last_access: u64,
}

/// Persistent, Java-compatible implementation of `UserNameToIdResolver`.
pub struct CachedUserNameToIdResolver<R> {
    resolve_offline_users: bool,
    profiles_by_name: HashMap<String, Rc<RefCell<GameProfileInfo>>>,
    profiles_by_uuid: HashMap<String, Rc<RefCell<GameProfileInfo>>>,
    profile_repository: R,
    file: PathBuf,
    operation_count: u64,
}

impl<R: GameProfileRepository> CachedUserNameToIdResolver<R> {
    pub fn new(profile_repository: R, file: impl Into<PathBuf>) -> Self {
        let mut resolver = Self {
            resolve_offline_users: true,
            profiles_by_name: HashMap::new(),
            profiles_by_uuid: HashMap::new(),
            profile_repository,
            file: file.into(),
            operation_count: 0,
        };

        // Java loads newest-first JSON and reverses it before assigning access
        // counters, preserving the file's MRU ordering after reconstruction.
        let mut loaded = load_profiles(&resolver.file);
        loaded.reverse();
        for profile in loaded {
            resolver.safe_add(profile);
        }
        resolver
    }

    fn next_operation(&mut self) -> u64 {
        self.operation_count = self.operation_count.wrapping_add(1);
        self.operation_count
    }

    fn safe_add(&mut self, mut info: GameProfileInfo) {
        info.last_access = self.next_operation();
        let lower_name = info.profile.name.to_lowercase();
        let uuid = info.profile.uuid.clone();
        let info = Rc::new(RefCell::new(info));

        // Java performs two independent ConcurrentHashMap.put calls. Sharing
        // the entry preserves volatile last-access updates across both indexes,
        // including Java's stale-key behavior when a name or UUID is replaced.
        self.profiles_by_name.insert(lower_name, Rc::clone(&info));
        self.profiles_by_uuid.insert(uuid, info);
    }

    fn add_at(&mut self, profile: NameAndId, now: DateTime<Local>) -> NameAndId {
        // Calendar.add(MONTH, 1), including end-of-month clamping.
        let expiration_date = now
            .checked_add_months(Months::new(1))
            .unwrap_or_else(|| end_of_next_month(now));
        let result = profile.clone();
        self.safe_add(GameProfileInfo {
            profile,
            expiration_date,
            last_access: 0,
        });
        self.save();
        result
    }

    fn get_by_name_at(&mut self, name: &str, now: DateTime<Local>) -> Option<NameAndId> {
        let user_name = name.to_lowercase();
        let expired = self
            .profiles_by_name
            .get(&user_name)
            .is_some_and(|info| now >= info.borrow().expiration_date);
        if expired {
            if let Some(info) = self.profiles_by_name.remove(&user_name) {
                self.profiles_by_uuid.remove(&info.borrow().profile.uuid);
            }
        }

        if let Some(info) = self.profiles_by_name.get(&user_name).cloned() {
            let operation = self.next_operation();
            let mut info = info.borrow_mut();
            info.last_access = operation;
            return Some(info.profile.clone());
        }

        let resolved = if valid_player_name(&user_name) {
            self.profile_repository.find_profile_by_name(&user_name)
        } else {
            None
        };
        let profile = resolved.or_else(|| {
            self.resolve_offline_users
                .then(|| NameAndId::create_offline(&user_name))
        });
        if let Some(profile) = profile {
            return Some(self.add_at(profile, now));
        }

        if expired {
            self.save();
        }
        None
    }

    fn mru_profiles(&self) -> Vec<Rc<RefCell<GameProfileInfo>>> {
        let mut profiles: Vec<_> = self.profiles_by_uuid.values().cloned().collect();
        profiles.sort_unstable_by_key(|profile| {
            std::cmp::Reverse(profile.borrow().last_access)
        });
        profiles.truncate(MRU_LIMIT);
        profiles
    }
}

impl<R: GameProfileRepository> UserNameToIdResolver for CachedUserNameToIdResolver<R> {
    fn add(&mut self, profile: NameAndId) {
        self.add_at(profile, Local::now());
    }

    fn get_by_name(&mut self, name: &str) -> Option<NameAndId> {
        self.get_by_name_at(name, Local::now())
    }

    fn get_by_id(&mut self, id: &str) -> Option<NameAndId> {
        let info = self.profiles_by_uuid.get(id)?.clone();
        let operation = self.next_operation();
        let mut info = info.borrow_mut();
        info.last_access = operation;
        Some(info.profile.clone())
    }

    fn resolve_offline_users(&mut self, value: bool) {
        self.resolve_offline_users = value;
    }

    fn save(&self) {
        let entries: Vec<_> = self
            .mru_profiles()
            .into_iter()
            .map(|profile| profile_json(&profile.borrow()))
            .collect();
        if let Ok(json) = serde_json::to_string(&entries) {
            // Java deliberately ignores IO failures from this best-effort cache.
            let _ignored = fs::write(&self.file, json);
        }
    }
}

fn valid_player_name(name: &str) -> bool {
    // Java checks UTF-16 length and rejects control/non-ASCII characters only.
    name.encode_utf16().count() <= 16
        && name
            .chars()
            .all(|character| character as u32 > 32 && (character as u32) < 127)
}

fn end_of_next_month(now: DateTime<Local>) -> DateTime<Local> {
    let (year, month) = if now.month() == 12 {
        (now.year() + 1, 1)
    } else {
        (now.year(), now.month() + 1)
    };
    let first_after = if month == 12 {
        Local.with_ymd_and_hms(year + 1, 1, 1, now.hour(), now.minute(), now.second())
    } else {
        Local.with_ymd_and_hms(year, month + 1, 1, now.hour(), now.minute(), now.second())
    };
    first_after
        .single()
        .unwrap_or(now)
        .checked_sub_signed(chrono::Duration::days(1))
        .unwrap_or(now)
}

use chrono::Timelike;

fn profile_json(info: &GameProfileInfo) -> Value {
    let mut object = Map::new();
    object.insert("uuid".to_string(), Value::String(info.profile.uuid.clone()));
    object.insert("name".to_string(), Value::String(info.profile.name.clone()));
    object.insert(
        "expiresOn".to_string(),
        Value::String(info.expiration_date.format(DATE_FORMAT).to_string()),
    );
    Value::Object(object)
}

fn load_profiles(path: &Path) -> Vec<GameProfileInfo> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(error) if error.kind() == ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            crate::log::log_warn(&format!(
                "Failed to load profile cache {}: {error}",
                path.display()
            ));
            return Vec::new();
        }
    };
    let entries = match serde_json::from_str(&json) {
        Ok(Value::Array(entries)) => entries,
        Ok(_) => {
            crate::log::log_warn(&format!(
                "Failed to load profile cache {}: root is not a JSON array",
                path.display()
            ));
            return Vec::new();
        }
        Err(error) => {
            crate::log::log_warn(&format!(
                "Failed to load profile cache {}: {error}",
                path.display()
            ));
            return Vec::new();
        }
    };
    entries.into_iter().filter_map(read_profile).collect()
}

fn read_profile(value: Value) -> Option<GameProfileInfo> {
    let object = value.as_object()?;
    let uuid = object.get("uuid")?.as_str()?;
    let name = object.get("name")?.as_str()?;
    let expires_on = object.get("expiresOn")?.as_str()?;
    valid_uuid(uuid)?;
    let expiration_date = match DateTime::parse_from_str(expires_on, DATE_FORMAT) {
        Ok(expiration_date) => expiration_date.with_timezone(&Local),
        Err(error) => {
            crate::log::log_warn(&format!("Failed to parse date {expires_on}: {error}"));
            return None;
        }
    };
    Some(GameProfileInfo {
        profile: NameAndId {
            uuid: uuid.to_string(),
            name: name.to_string(),
        },
        expiration_date,
        last_access: 0,
    })
}

fn valid_uuid(value: &str) -> Option<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return None;
    }
    for (index, byte) in bytes.iter().enumerate() {
        let hyphen = matches!(index, 8 | 13 | 18 | 23);
        if (hyphen && *byte != b'-') || (!hyphen && !byte.is_ascii_hexdigit()) {
            return None;
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(vibecraft_has_decompiled_sources)]
    const INTERFACE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/players/UserNameToIdResolver.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const CACHE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/players/CachedUserNameToIdResolver.java");

    #[derive(Default)]
    struct Repository {
        profiles: HashMap<String, NameAndId>,
        lookups: RefCell<Vec<String>>,
    }

    impl GameProfileRepository for Repository {
        fn find_profile_by_name(&self, name: &str) -> Option<NameAndId> {
            self.lookups.borrow_mut().push(name.to_string());
            self.profiles.get(name).cloned()
        }
    }

    fn temp_file(test: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("vibecraft-{test}-{nonce}.json"))
    }

    fn profile(name: &str, uuid: &str) -> NameAndId {
        NameAndId {
            name: name.to_string(),
            uuid: uuid.to_string(),
        }
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn interface_and_cache_shapes_are_anchored_to_java() {
        for method in ["void add(", "get(String name)", "get(UUID id)", "resolveOfflineUsers(", "void save()"] {
            assert!(INTERFACE_JAVA.contains(method));
        }
        for behavior in [
            "GAMEPROFILES_MRU_LIMIT = 1000",
            "name().toLowerCase(Locale.ROOT)",
            "c.add(2, 1)",
            "new Date().getTime() >= profileInfo.expirationDate.getTime()",
            "NameAndId.createOffline(name)",
            ".sorted(Comparator.comparing(CachedUserNameToIdResolver.GameProfileInfo::lastAccess).reversed())",
        ] {
            assert!(CACHE_JAVA.contains(behavior), "missing Java behavior: {behavior}");
        }
    }

    #[test]
    fn name_lookup_is_case_insensitive_and_repository_results_are_cached() {
        let path = temp_file("lookup");
        let expected = profile("Notch", "069a79f4-44e9-4726-a5be-fca90e38aaf5");
        let mut repository = Repository::default();
        repository.profiles.insert("notch".to_string(), expected.clone());
        let mut cache = CachedUserNameToIdResolver::new(repository, &path);

        assert_eq!(cache.get_by_name("NoTcH"), Some(expected.clone()));
        assert_eq!(cache.get_by_name("NOTCH"), Some(expected.clone()));
        assert_eq!(cache.get_by_id(&expected.uuid), Some(expected));
        assert_eq!(&*cache.profile_repository.lookups.borrow(), &["notch"]);
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn unknown_and_invalid_names_follow_offline_resolution_switch() {
        let path = temp_file("offline");
        let mut cache = CachedUserNameToIdResolver::new(Repository::default(), &path);
        assert_eq!(cache.get_by_name("Steve"), Some(NameAndId::create_offline("steve")));

        let mut disabled = CachedUserNameToIdResolver::new(Repository::default(), &path);
        disabled.resolve_offline_users(false);
        assert_eq!(disabled.get_by_name("unknown"), None);
        assert_eq!(disabled.get_by_name("bad\nname"), None);
        assert_eq!(&*disabled.profile_repository.lookups.borrow(), &["unknown"]);
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn expired_names_are_evicted_but_uuid_lookup_matches_java_non_expiring_path() {
        let path = temp_file("expiry");
        let now = Local.with_ymd_and_hms(2026, 1, 31, 12, 0, 0).single().expect("valid test time");
        let expected = profile("Alex", "ec561538-f3fd-461d-aff5-086b22154bce");
        let mut cache = CachedUserNameToIdResolver::new(Repository::default(), &path);
        cache.add_at(expected.clone(), now);
        assert_eq!(cache.get_by_id(&expected.uuid), Some(expected.clone()));
        cache.resolve_offline_users(false);
        assert_eq!(cache.get_by_name_at("Alex", now + chrono::Duration::days(29)), None);
        assert_eq!(cache.get_by_id(&expected.uuid), None);
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn persistence_loads_valid_entries_and_preserves_mru_order() {
        let path = temp_file("persistence");
        let mut cache = CachedUserNameToIdResolver::new(Repository::default(), &path);
        let first = profile("First", "00000000-0000-0000-0000-000000000001");
        let second = profile("Second", "00000000-0000-0000-0000-000000000002");
        cache.add(first.clone());
        cache.add(second.clone());
        assert_eq!(cache.get_by_name("First"), Some(first.clone()));
        cache.save();

        let json: Value = serde_json::from_str(&fs::read_to_string(&path).expect("saved cache"))
            .expect("valid JSON");
        assert_eq!(json[0]["name"], "First");
        let mut loaded = CachedUserNameToIdResolver::new(Repository::default(), &path);
        assert_eq!(loaded.get_by_id(&first.uuid), Some(first));
        assert_eq!(loaded.get_by_id(&second.uuid), Some(second));
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn malformed_file_and_write_failures_are_ignored() {
        let path = temp_file("malformed");
        fs::write(&path, "not json").expect("write malformed fixture");
        let mut cache = CachedUserNameToIdResolver::new(Repository::default(), &path);
        cache.add(profile("Player", "00000000-0000-0000-0000-000000000003"));

        let directory = std::env::temp_dir();
        let mut unwritable = CachedUserNameToIdResolver::new(Repository::default(), directory);
        unwritable.add(profile("Player", "00000000-0000-0000-0000-000000000004"));
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn save_is_limited_to_one_thousand_most_recent_uuid_entries() {
        let path = temp_file("mru-limit");
        let mut cache = CachedUserNameToIdResolver::new(Repository::default(), &path);
        let now = Local
            .with_ymd_and_hms(2026, 2, 1, 0, 0, 0)
            .single()
            .expect("valid test time");
        for index in 0..=MRU_LIMIT {
            cache.safe_add(GameProfileInfo {
                profile: profile(
                    &format!("Player{index}"),
                    &format!("00000000-0000-0000-0000-{index:012x}"),
                ),
                expiration_date: now + chrono::Duration::days(30),
                last_access: 0,
            });
        }
        cache.save();

        let entries: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(&path).expect("saved cache"))
                .expect("valid cache JSON");
        assert_eq!(entries.len(), MRU_LIMIT);
        assert_eq!(entries[0]["name"], "Player1000");
        assert!(entries.iter().all(|entry| entry["name"] != "Player0"));
        let _ignored = fs::remove_file(path);
    }
}
