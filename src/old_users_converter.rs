//! Legacy account-file migration from `OldUsersConverter.java`.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};
use serde_json::{Map, Value};

use crate::command::PermissionLevel;
use crate::player_access::NameAndId;
use crate::stored_user_list::{
    IpBanList, IpBanListEntry, ServerOpList, ServerOpListEntry, UserBanList, UserBanListEntry,
    UserWhiteList, UserWhiteListEntry,
};

pub const OLD_IPBANLIST: &str = "banned-ips.txt";
pub const OLD_USERBANLIST: &str = "banned-players.txt";
pub const OLD_OPLIST: &str = "ops.txt";
pub const OLD_WHITELIST: &str = "white-list.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileLookupError {
    NotFound,
    Backend(String),
}

pub trait LegacyProfileResolver {
    fn uses_authentication(&self) -> bool;
    fn lookup_profiles(&mut self, names: &[String]) -> Vec<Result<NameAndId, ProfileLookupError>>;
    fn cached_profile(&mut self, name: &str) -> Option<NameAndId>;
    fn cache_profile(&mut self, profile: NameAndId);
}

pub fn read_old_list_format(contents: &str) -> (Vec<String>, HashMap<String, Vec<String>>) {
    let lines: Vec<String> = contents.lines().map(ToString::to_string).collect();
    let mut user_map = HashMap::new();
    for line in &lines {
        let trimmed = line.trim();
        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            let parts: Vec<String> = trimmed.split_terminator('|').map(ToString::to_string).collect();
            if let Some(first) = parts.first() {
                user_map.insert(first.to_lowercase(), parts);
            }
        }
    }
    (lines, user_map)
}

pub fn parse_date(value: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S %z")
        .ok()
        .map(|date| date.with_timezone(&Local))
}

fn profile_results<R: LegacyProfileResolver>(
    resolver: &mut R,
    names: impl IntoIterator<Item = String>,
) -> Vec<Result<NameAndId, ProfileLookupError>> {
    let names: Vec<String> = names
        .into_iter()
        .filter(|name| !name.is_empty())
        .collect();
    if resolver.uses_authentication() {
        resolver.lookup_profiles(&names)
    } else {
        names
            .into_iter()
            .map(|name| Ok(NameAndId::create_offline(&name)))
            .collect()
    }
}

pub fn convert_user_banlist<R: LegacyProfileResolver>(root: &Path, resolver: &mut R) -> bool {
    let old_path = root.join(OLD_USERBANLIST);
    if !old_path.is_file() {
        return true;
    }
    let new_path = root.join("banned-players.json");
    let mut bans = UserBanList::new(&new_path, ());
    let _ = bans.load();
    let Ok(contents) = fs::read_to_string(&old_path) else {
        return false;
    };
    let (_, users) = read_old_list_format(&contents);
    let names: Vec<String> = users.keys().cloned().collect();
    for result in profile_results(resolver, names) {
        let profile = match result {
            Ok(profile) => profile,
            Err(ProfileLookupError::NotFound) => continue,
            Err(ProfileLookupError::Backend(_)) => return false,
        };
        resolver.cache_profile(profile.clone());
        let Some(parts) = users.get(&profile.name.to_lowercase()) else {
            return false;
        };
        let entry = UserBanListEntry::with_details(
            Some(profile),
            parts.get(1).and_then(|value| parse_date(value)),
            parts.get(2).cloned(),
            parts.get(3).and_then(|value| parse_date(value)),
            parts.get(4).cloned(),
        );
        if bans.add(entry).is_err() {
            return false;
        }
    }
    if bans.save().is_err() {
        return false;
    }
    rename_old_file(&old_path)
}

pub fn convert_ip_banlist<R: LegacyProfileResolver>(root: &Path, _resolver: &mut R) -> bool {
    let old_path = root.join(OLD_IPBANLIST);
    if !old_path.is_file() {
        return true;
    }
    let Ok(contents) = fs::read_to_string(&old_path) else {
        return false;
    };
    let (_, users) = read_old_list_format(&contents);
    let new_path = root.join("banned-ips.json");
    let mut bans = IpBanList::new(&new_path, ());
    let _ = bans.load();
    for (ip, parts) in users {
        if bans
            .add(IpBanListEntry::with_details(
                Some(ip),
                parts.get(1).and_then(|value| parse_date(value)),
                parts.get(2).cloned(),
                parts.get(3).and_then(|value| parse_date(value)),
                parts.get(4).cloned(),
            ))
            .is_err()
        {
            return false;
        }
    }
    if bans.save().is_err() {
        return false;
    }
    rename_old_file(&old_path)
}

pub fn convert_ops_list<R: LegacyProfileResolver>(root: &Path, resolver: &mut R, permissions: PermissionLevel) -> bool {
    let old_path = root.join(OLD_OPLIST);
    if !old_path.is_file() {
        return true;
    }
    let mut list = ServerOpList::new(root.join("ops.json"), ());
    let _ = list.load();
    convert_name_list(root, OLD_OPLIST, resolver, |profile| {
        list.add(ServerOpListEntry::new(profile, permissions, false)).is_ok()
    }) && list.save().is_ok() && rename_old_file(&old_path)
}

pub fn convert_white_list<R: LegacyProfileResolver>(root: &Path, resolver: &mut R) -> bool {
    let old_path = root.join(OLD_WHITELIST);
    if !old_path.is_file() {
        return true;
    }
    let mut list = UserWhiteList::new(root.join("whitelist.json"), ());
    let _ = list.load();
    convert_name_list(root, OLD_WHITELIST, resolver, |profile| {
        list.add(UserWhiteListEntry::new(profile)).is_ok()
    }) && list.save().is_ok() && rename_old_file(&old_path)
}

fn convert_name_list<R, F>(root: &Path, old_name: &str, resolver: &mut R, mut add: F) -> bool
where
    R: LegacyProfileResolver,
    F: FnMut(NameAndId) -> bool,
{
    let old_path = root.join(old_name);
    if !old_path.is_file() {
        return true;
    }
    let Ok(contents) = fs::read_to_string(&old_path) else {
        return false;
    };
    let (lines, _) = read_old_list_format(&contents);
    for result in profile_results(resolver, lines) {
        match result {
            Ok(profile) => {
                resolver.cache_profile(profile.clone());
                if !add(profile) {
                    return false;
                }
            }
            Err(ProfileLookupError::NotFound) => {}
            Err(ProfileLookupError::Backend(_)) => return false,
        }
    }
    true
}

pub fn convert_mob_owner_if_necessary<R: LegacyProfileResolver>(
    resolver: &mut R,
    owner: &str,
    singleplayer: bool,
) -> Option<String> {
    if !owner.is_empty() && owner.encode_utf16().count() <= 16 {
        if let Some(profile) = resolver.cached_profile(owner) {
            return Some(profile.uuid);
        }
        if !singleplayer && resolver.uses_authentication() {
            if let Some(Ok(profile)) = resolver.lookup_profiles(&[owner.to_string()]).into_iter().next() {
                resolver.cache_profile(profile.clone());
                return Some(profile.uuid);
            }
        } else {
            return Some(NameAndId::create_offline(owner).uuid);
        }
        None
    } else {
        let value = Value::Object(Map::from_iter([
            ("uuid".to_string(), Value::String(owner.to_string())),
            ("name".to_string(), Value::String(String::new())),
        ]));
        NameAndId::from_json(&value).map(|profile| profile.uuid)
    }
}

pub fn convert_players<R: LegacyProfileResolver>(root: &Path, resolver: &mut R) -> bool {
    let old_dir = root.join("players");
    if !old_dir.is_dir() {
        return true;
    }
    let Ok(entries) = fs::read_dir(&old_dir) else {
        return false;
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name.to_lowercase().ends_with(".dat") && file_name.len() > 4 {
            names.push(file_name[..file_name.len() - 4].to_string());
        }
    }
    let results = profile_results(resolver, names.clone());
    for (name_result, name) in results.into_iter().zip(names) {
        let source = old_dir.join(format!("{name}.dat"));
        match name_result {
            Ok(profile) => {
                resolver.cache_profile(profile.clone());
                if move_player_file(&source, &root.join("playerdata"), &format!("{}.dat", profile.uuid)).is_err() {
                    return false;
                }
            }
            Err(ProfileLookupError::NotFound) => {
                if move_player_file(&source, &root.join("unknownplayers"), &format!("{name}.dat")).is_err() {
                    return false;
                }
            }
            Err(ProfileLookupError::Backend(_)) => return false,
        }
    }
    true
}

fn move_player_file(source: &Path, directory: &Path, file_name: &str) -> std::io::Result<()> {
    ensure_directory_exists(directory)?;
    fs::rename(source, directory.join(file_name))
}

fn ensure_directory_exists(directory: &Path) -> std::io::Result<()> {
    if directory.exists() {
        if directory.is_dir() {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "path is not a directory"))
        }
    } else {
        fs::create_dir_all(directory)
    }
}

pub fn are_old_userlists_removed(root: &Path) -> bool {
    [OLD_USERBANLIST, OLD_IPBANLIST, OLD_OPLIST, OLD_WHITELIST]
        .iter()
        .all(|name| !root.join(name).is_file())
}

fn rename_old_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    fs::rename(path, path.with_file_name(format!("{file_name}.converted"))).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/players/OldUsersConverter.java");

    #[derive(Default)]
    struct Resolver {
        authenticated: bool,
        profiles: HashMap<String, NameAndId>,
        failures: HashMap<String, ProfileLookupError>,
        cache: RefCell<HashMap<String, NameAndId>>,
    }

    impl LegacyProfileResolver for Resolver {
        fn uses_authentication(&self) -> bool {
            self.authenticated
        }

        fn lookup_profiles(&mut self, names: &[String]) -> Vec<Result<NameAndId, ProfileLookupError>> {
            names.iter().map(|name| self.profiles.get(&name.to_lowercase()).cloned().ok_or_else(|| self.failures.get(&name.to_lowercase()).cloned().unwrap_or(ProfileLookupError::NotFound))).collect()
        }

        fn cached_profile(&mut self, name: &str) -> Option<NameAndId> {
            self.cache.borrow().get(&name.to_lowercase()).cloned()
        }

        fn cache_profile(&mut self, profile: NameAndId) {
            self.cache.borrow_mut().insert(profile.name.to_lowercase(), profile);
        }
    }

    fn root(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("vibecraft-old-users-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("fixture root");
        path
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_matches_java_converter_surface() {
        for fragment in [
            "OLD_IPBANLIST = new File(\"banned-ips.txt\")",
            "readOldListFormat",
            "lookupPlayers",
            "convertUserBanlist",
            "convertIpBanlist",
            "convertOpsList",
            "convertWhiteList",
            "convertMobOwnerIfNecessary",
            "convertPlayers",
            "areOldUserlistsRemoved",
            "renameOldFile",
        ] {
            assert!(JAVA_SOURCE.contains(fragment), "missing Java source fragment: {fragment}");
        }
    }

    #[test]
    fn old_list_parser_matches_trim_comment_case_fold_and_duplicate_rules() {
        let (lines, map) = read_old_list_format("# comment\n Steve | date | reason \nsteve|new\n\nAlex");
        assert_eq!(lines.len(), 5);
        assert_eq!(map["steve"], vec!["steve", "new"]);
        assert_eq!(map["alex"], vec!["Alex"]);
    }

    #[test]
    fn offline_conversion_migrates_all_lists_and_renames_sources() {
        let path = root("lists");
        fs::write(path.join(OLD_IPBANLIST), "127.0.0.1|2026-07-16 00:00:00 +0000|Console|forever|bad").expect("ip fixture");
        fs::write(path.join(OLD_USERBANLIST), "Steve|2026-07-16 00:00:00 +0000|Console|forever|bad").expect("ban fixture");
        fs::write(path.join(OLD_OPLIST), "Steve\n").expect("op fixture");
        fs::write(path.join(OLD_WHITELIST), "Steve\n").expect("white fixture");
        let mut resolver = Resolver::default();
        assert!(convert_ip_banlist(&path, &mut resolver));
        assert!(convert_user_banlist(&path, &mut resolver));
        assert!(convert_ops_list(&path, &mut resolver, PermissionLevel::Owners));
        assert!(convert_white_list(&path, &mut resolver));
        assert!(path.join("banned-ips.txt.converted").is_file());
        assert!(path.join("banned-players.txt.converted").is_file());
        assert!(path.join("ops.txt.converted").is_file());
        assert!(path.join("white-list.txt.converted").is_file());
        assert!(are_old_userlists_removed(&path));
        let _ignored = fs::remove_dir_all(path);
    }

    #[test]
    fn mob_owner_and_player_file_conversion_match_authentication_branches() {
        let path = root("players");
        fs::create_dir_all(path.join("players")).expect("old player directory");
        fs::write(path.join("players/Steve.dat"), b"player").expect("player fixture");
        let mut resolver = Resolver::default();
        let offline = convert_mob_owner_if_necessary(&mut resolver, "Steve", true).expect("offline owner");
        assert_eq!(offline, NameAndId::create_offline("Steve").uuid);
        assert!(convert_players(&path, &mut resolver));
        assert!(path.join(format!("playerdata/{}.dat", offline)).is_file());
        assert_eq!(convert_mob_owner_if_necessary(&mut resolver, offline.as_str(), true), Some(offline));
        let _ignored = fs::remove_dir_all(path);
    }

    #[test]
    fn authentication_failures_abort_conversion_but_not_found_entries_are_skipped() {
        let path = root("lookup-failures");
        fs::write(path.join(OLD_USERBANLIST), "Missing\nBackend\n").expect("ban fixture");
        let mut resolver = Resolver {
            authenticated: true,
            failures: HashMap::from([
                ("missing".to_string(), ProfileLookupError::NotFound),
                ("backend".to_string(), ProfileLookupError::Backend("offline".to_string())),
            ]),
            ..Resolver::default()
        };
        assert!(!convert_user_banlist(&path, &mut resolver));
        assert!(path.join(OLD_USERBANLIST).is_file());
        assert!(!path.join("banned-players.txt.converted").exists());

        let _ignored = fs::remove_dir_all(path);
    }

    #[test]
    fn empty_old_lists_still_write_new_json_before_renaming() {
        let path = root("empty-lists");
        fs::write(path.join(OLD_USERBANLIST), "# only a comment\n\n").expect("ban fixture");
        fs::write(path.join(OLD_OPLIST), "\n").expect("op fixture");
        let mut resolver = Resolver::default();
        assert!(convert_user_banlist(&path, &mut resolver));
        assert!(convert_ops_list(&path, &mut resolver, PermissionLevel::Owners));
        assert_eq!(fs::read_to_string(path.join("banned-players.json")).expect("ban json"), "[]");
        assert_eq!(fs::read_to_string(path.join("ops.json")).expect("op json"), "[]");
        assert!(path.join("banned-players.txt.converted").is_file());
        assert!(path.join("ops.txt.converted").is_file());

        let _ignored = fs::remove_dir_all(path);
    }
}
