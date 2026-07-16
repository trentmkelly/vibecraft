//! Persistent stored-user lists and IP-ban entries.
//!
//! This is the Rust counterpart of `StoredUserEntry`, `StoredUserList`,
//! `BanListEntry`, `IpBanListEntry`, and `IpBanList` from the Java server.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde_json::{Map, Value};

use crate::command::PermissionLevel;
use crate::player_access::NameAndId;

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";
const EXPIRES_NEVER: &str = "forever";

/// The common Java `StoredUserEntry<K>` contract.
pub trait StoredUserEntry<K>: Clone + PartialEq {
    fn user(&self) -> Option<&K>;
    fn has_expired(&self, now: DateTime<Local>) -> bool;
    fn serialize(&self) -> Value;
}

/// Java's map-backed stored list, with explicit entry construction on load.
#[derive(Debug, Clone)]
pub struct StoredUserList<K, V> {
    file: PathBuf,
    map: HashMap<String, V>,
    _key: std::marker::PhantomData<K>,
}

impl<K, V> StoredUserList<K, V>
where
    K: Clone + Eq + Hash + ToString,
    V: StoredUserEntry<K>,
{
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            map: HashMap::new(),
            _key: std::marker::PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn file(&self) -> &Path {
        &self.file
    }

    pub fn add(&mut self, entry: V) -> std::io::Result<bool> {
        let Some(user) = entry.user() else {
            return Ok(false);
        };
        let key = user.to_string();
        if self.map.get(&key) == Some(&entry) {
            return Ok(false);
        }
        self.map.insert(key, entry);
        self.save()?;
        Ok(true)
    }

    pub fn get(&mut self, user: &K, now: DateTime<Local>) -> Option<&V> {
        self.remove_expired(now);
        self.map.get(&user.to_string())
    }

    pub fn remove(&mut self, user: &K) -> std::io::Result<bool> {
        if self.map.remove(&user.to_string()).is_none() {
            return Ok(false);
        }
        self.save()?;
        Ok(true)
    }

    #[allow(dead_code)]
    pub fn remove_entry(&mut self, entry: &V) -> std::io::Result<bool> {
        let Some(user) = entry.user() else {
            return Ok(false);
        };
        self.remove(user)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) -> std::io::Result<()> {
        self.map.clear();
        self.save()
    }

    #[allow(dead_code)]
    pub fn get_user_list(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[allow(dead_code)]
    pub fn contains(&self, user: &K) -> bool {
        self.map.contains_key(&user.to_string())
    }

    pub fn entries(&self) -> impl Iterator<Item = &V> {
        self.map.values()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let values: Vec<Value> = self.map.values().map(StoredUserEntry::serialize).collect();
        let parent = self.file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        fs::write(&self.file, serde_json::to_string_pretty(&values)?)
    }

    pub fn load<F>(&mut self, create_entry: F) -> std::io::Result<()>
    where
        F: Fn(&Map<String, Value>) -> V,
    {
        let contents = match fs::read_to_string(&self.file) {
            Ok(contents) => contents,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        let parsed: Value = serde_json::from_str(&contents).map_err(invalid_data)?;
        let Value::Array(entries) = parsed else {
            return Err(invalid_data("stored user list root must be an array"));
        };
        self.map.clear();
        for value in entries {
            let Value::Object(object) = value else {
                return Err(invalid_data("stored user list entry must be an object"));
            };
            let entry = create_entry(&object);
            if let Some(user) = entry.user() {
                self.map.insert(user.to_string(), entry);
            }
        }
        Ok(())
    }

    fn remove_expired(&mut self, now: DateTime<Local>) {
        self.map.retain(|_, entry| !entry.has_expired(now));
    }
}

fn invalid_data(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(ErrorKind::InvalidData, error.to_string())
}

/// Common fields and behavior from Java `BanListEntry<T>`.
#[derive(Debug, Clone)]
pub struct BanListEntry<T> {
    pub user: Option<T>,
    pub created: DateTime<Local>,
    pub source: String,
    pub expires: Option<DateTime<Local>>,
    pub reason: Option<String>,
}

impl<T: PartialEq> PartialEq for BanListEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.user == other.user
            && self.source == other.source
            && self.expires == other.expires
            && self.reason == other.reason
    }
}

impl<T: Eq> Eq for BanListEntry<T> {}

impl<T> BanListEntry<T> {
    pub fn new(user: Option<T>, created: Option<DateTime<Local>>, source: Option<String>, expires: Option<DateTime<Local>>, reason: Option<String>) -> Self {
        Self {
            user,
            created: created.unwrap_or_else(Local::now),
            source: source.unwrap_or_else(|| "(Unknown)".to_string()),
            expires,
            reason,
        }
    }

    pub fn reason_message(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    fn serialize_common(&self, object: &mut Map<String, Value>) {
        object.insert("created".to_string(), Value::String(self.created.format(DATE_FORMAT).to_string()));
        object.insert("source".to_string(), Value::String(self.source.clone()));
        object.insert("expires".to_string(), Value::String(self.expires.as_ref().map_or_else(|| EXPIRES_NEVER.to_string(), |date| date.format(DATE_FORMAT).to_string())));
        object.insert("reason".to_string(), self.reason.clone().map_or(Value::Null, Value::String));
    }
}

impl<T> BanListEntry<T>
where
    T: Clone,
{
    pub fn from_json_common(object: &Map<String, Value>, user: Option<T>) -> Self {
        Self {
            user,
            created: object.get("created").and_then(parse_date).unwrap_or_else(Local::now),
            source: object.get("source").and_then(Value::as_str).unwrap_or("(Unknown)").to_string(),
            expires: object
                .get("expires")
                .and_then(Value::as_str)
                .and_then(|value| (value != EXPIRES_NEVER).then_some(value).and_then(parse_date_text)),
            reason: object.get("reason").and_then(|value| (!value.is_null()).then(|| value.as_str())).flatten().map(ToString::to_string),
        }
    }
}

/// Java `IpBanListEntry`, including its null-user load behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpBanListEntry {
    pub common: BanListEntry<String>,
}

impl IpBanListEntry {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            common: BanListEntry::new(Some(address.into()), None, None, None, None),
        }
    }

    pub fn with_details(address: Option<String>, created: Option<DateTime<Local>>, source: Option<String>, expires: Option<DateTime<Local>>, reason: Option<String>) -> Self {
        Self {
            common: BanListEntry::new(address, created, source, expires, reason),
        }
    }

    pub fn from_json(object: &Map<String, Value>) -> Self {
        let user = object.get("ip").and_then(Value::as_str).map(ToString::to_string);
        Self {
            common: BanListEntry::from_json_common(object, user),
        }
    }

    pub fn user(&self) -> Option<&String> {
        self.common.user.as_ref()
    }

    pub fn display_name(&self) -> String {
        self.user().map_or_else(|| "null".to_string(), ToString::to_string)
    }
}

impl StoredUserEntry<String> for IpBanListEntry {
    fn user(&self) -> Option<&String> {
        self.user()
    }

    fn has_expired(&self, now: DateTime<Local>) -> bool {
        self.common.expires.is_some_and(|expires| expires < now)
    }

    fn serialize(&self) -> Value {
        let Some(user) = self.user() else {
            return Value::Object(Map::new());
        };
        let mut object = Map::new();
        object.insert("ip".to_string(), Value::String(user.clone()));
        self.common.serialize_common(&mut object);
        Value::Object(object)
    }
}

/// Notification callbacks used by `IpBanList.add/remove/clear`.
pub trait IpBanNotifications {
    fn ip_banned(&mut self, ban: &IpBanListEntry);
    fn ip_unbanned(&mut self, ip: &str);
}

impl IpBanNotifications for () {
    fn ip_banned(&mut self, _ban: &IpBanListEntry) {}
    fn ip_unbanned(&mut self, _ip: &str) {}
}

/// Java `IpBanList` facade over `StoredUserList<String, IpBanListEntry>`.
#[derive(Debug, Clone)]
pub struct IpBanList<N = ()> {
    list: StoredUserList<String, IpBanListEntry>,
    notifications: N,
}

impl<N> IpBanList<N>
where
    N: IpBanNotifications,
{
    pub fn new(file: impl Into<PathBuf>, notifications: N) -> Self {
        Self {
            list: StoredUserList::new(file),
            notifications,
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        self.list.load(IpBanListEntry::from_json)
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.list.save()
    }

    pub fn is_banned(&mut self, ip: &str, now: DateTime<Local>) -> bool {
        self.list.get(&ip.to_string(), now).is_some()
    }

    pub fn is_banned_socket_address(&mut self, address: &str, now: DateTime<Local>) -> bool {
        self.is_banned(&ip_from_socket_address(address), now)
    }

    pub fn get(&mut self, ip: &str, now: DateTime<Local>) -> Option<&IpBanListEntry> {
        self.list.get(&ip.to_string(), now)
    }

    pub fn get_socket_address(&mut self, address: &str, now: DateTime<Local>) -> Option<&IpBanListEntry> {
        self.get(&ip_from_socket_address(address), now)
    }

    pub fn add(&mut self, entry: IpBanListEntry) -> std::io::Result<bool> {
        let changed = self.list.add(entry.clone())?;
        if changed && entry.user().is_some() {
            self.notifications.ip_banned(&entry);
        }
        Ok(changed)
    }

    pub fn remove(&mut self, ip: &str) -> std::io::Result<bool> {
        let changed = self.list.remove(&ip.to_string())?;
        if changed {
            self.notifications.ip_unbanned(ip);
        }
        Ok(changed)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) -> std::io::Result<()> {
        let users: Vec<String> = self.list.entries().filter_map(|entry| entry.user().cloned()).collect();
        for user in users {
            self.notifications.ip_unbanned(&user);
        }
        self.list.clear()
    }

    pub fn entries(&self) -> impl Iterator<Item = &IpBanListEntry> {
        self.list.entries()
    }

    pub fn notifications(&self) -> &N {
        &self.notifications
    }

}

/// Notification callbacks used by `UserBanList.add/remove/clear`.
pub trait PlayerBanNotifications {
    fn player_banned(&mut self, ban: &UserBanListEntry);
    fn player_unbanned(&mut self, player: &NameAndId);
}

impl PlayerBanNotifications for () {
    fn player_banned(&mut self, _ban: &UserBanListEntry) {}
    fn player_unbanned(&mut self, _player: &NameAndId) {}
}

/// Java `UserBanListEntry`, keyed by the UUID portion of `NameAndId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserBanListEntry {
    pub common: BanListEntry<NameAndId>,
}

impl UserBanListEntry {
    pub fn new(user: Option<NameAndId>) -> Self {
        Self {
            common: BanListEntry::new(user, None, None, None, None),
        }
    }

    pub fn with_details(user: Option<NameAndId>, created: Option<DateTime<Local>>, source: Option<String>, expires: Option<DateTime<Local>>, reason: Option<String>) -> Self {
        Self {
            common: BanListEntry::new(user, created, source, expires, reason),
        }
    }

    pub fn from_json(object: &Map<String, Value>) -> Self {
        let user = NameAndId::from_json(&Value::Object(object.clone()));
        Self {
            common: BanListEntry::from_json_common(object, user),
        }
    }

    pub fn user(&self) -> Option<&NameAndId> {
        self.common.user.as_ref()
    }

    pub fn display_name(&self) -> String {
        self.user().map_or_else(|| "commands.banlist.entry.unknown".to_string(), |user| user.name.clone())
    }
}

impl StoredUserEntry<NameAndId> for UserBanListEntry {
    fn user(&self) -> Option<&NameAndId> {
        self.user()
    }

    fn has_expired(&self, now: DateTime<Local>) -> bool {
        self.common.expires.is_some_and(|expires| expires < now)
    }

    fn serialize(&self) -> Value {
        let Some(user) = self.user() else {
            return Value::Object(Map::new());
        };
        let mut object = Map::new();
        user.append_to(&mut object);
        self.common.serialize_common(&mut object);
        Value::Object(object)
    }
}

/// Java `UserBanList` facade over the generic stored list.
#[derive(Debug, Clone)]
pub struct UserBanList<N = ()> {
    list: StoredUserList<NameAndId, UserBanListEntry>,
    notifications: N,
}

impl<N> UserBanList<N>
where
    N: PlayerBanNotifications,
{
    pub fn new(file: impl Into<PathBuf>, notifications: N) -> Self {
        Self {
            list: StoredUserList::new(file),
            notifications,
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        self.list.load(UserBanListEntry::from_json)
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.list.save()
    }

    pub fn is_banned(&mut self, user: &NameAndId, now: DateTime<Local>) -> bool {
        self.list.get(user, now).is_some()
    }

    pub fn get_user_list(&self) -> Vec<String> {
        self.list
            .entries()
            .filter_map(UserBanListEntry::user)
            .map(|user| user.name.clone())
            .collect()
    }

    pub fn add(&mut self, entry: UserBanListEntry) -> std::io::Result<bool> {
        let changed = self.list.add(entry.clone())?;
        if changed && entry.user().is_some() {
            self.notifications.player_banned(&entry);
        }
        Ok(changed)
    }

    pub fn remove(&mut self, user: &NameAndId) -> std::io::Result<bool> {
        let changed = self.list.remove(user)?;
        if changed {
            self.notifications.player_unbanned(user);
        }
        Ok(changed)
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        let users: Vec<NameAndId> = self.list.entries().filter_map(|entry| entry.user().cloned()).collect();
        for user in users {
            self.notifications.player_unbanned(&user);
        }
        self.list.clear()
    }

    pub fn entries(&self) -> impl Iterator<Item = &UserBanListEntry> {
        self.list.entries()
    }

    pub fn notifications(&self) -> &N {
        &self.notifications
    }
}

/// Notification callbacks used by `UserWhiteList.add/remove/clear`.
pub trait AllowlistNotifications {
    fn player_added(&mut self, player: &NameAndId);
    fn player_removed(&mut self, player: &NameAndId);
}

impl AllowlistNotifications for () {
    fn player_added(&mut self, _player: &NameAndId) {}
    fn player_removed(&mut self, _player: &NameAndId) {}
}

/// Java `UserWhiteListEntry`, a non-expiring stored `NameAndId` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWhiteListEntry {
    pub user: Option<NameAndId>,
}

impl UserWhiteListEntry {
    pub fn new(user: NameAndId) -> Self {
        Self { user: Some(user) }
    }

    pub fn from_json(object: &Map<String, Value>) -> Self {
        Self {
            user: NameAndId::from_json(&Value::Object(object.clone())),
        }
    }

    pub fn user(&self) -> Option<&NameAndId> {
        self.user.as_ref()
    }
}

impl StoredUserEntry<NameAndId> for UserWhiteListEntry {
    fn user(&self) -> Option<&NameAndId> {
        self.user()
    }

    fn has_expired(&self, _now: DateTime<Local>) -> bool {
        false
    }

    fn serialize(&self) -> Value {
        let Some(user) = self.user() else {
            return Value::Object(Map::new());
        };
        user.to_json_value()
    }
}

/// Java `UserWhiteList` facade over a UUID-keyed stored list.
#[derive(Debug, Clone)]
pub struct UserWhiteList<N = ()> {
    list: StoredUserList<NameAndId, UserWhiteListEntry>,
    notifications: N,
}

impl<N> UserWhiteList<N>
where
    N: AllowlistNotifications,
{
    pub fn new(file: impl Into<PathBuf>, notifications: N) -> Self {
        Self {
            list: StoredUserList::new(file),
            notifications,
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        self.list.load(UserWhiteListEntry::from_json)
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.list.save()
    }

    pub fn is_white_listed(&mut self, user: &NameAndId, now: DateTime<Local>) -> bool {
        self.list.get(user, now).is_some()
    }

    pub fn contains(&mut self, user: &NameAndId) -> bool {
        self.is_white_listed(user, Local::now())
    }

    pub fn add(&mut self, entry: UserWhiteListEntry) -> std::io::Result<bool> {
        let changed = self.list.add(entry.clone())?;
        if changed {
            if let Some(user) = entry.user() {
                self.notifications.player_added(user);
            }
        }
        Ok(changed)
    }

    pub fn remove(&mut self, user: &NameAndId) -> std::io::Result<bool> {
        let changed = self.list.remove(user)?;
        if changed {
            self.notifications.player_removed(user);
        }
        Ok(changed)
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        let users: Vec<NameAndId> = self.list.entries().filter_map(UserWhiteListEntry::user).cloned().collect();
        for user in users {
            self.notifications.player_removed(&user);
        }
        self.list.clear()
    }

    pub fn get_user_list(&self) -> Vec<String> {
        self.list
            .entries()
            .filter_map(UserWhiteListEntry::user)
            .map(|user| user.name.clone())
            .collect()
    }

    pub fn entries(&self) -> impl Iterator<Item = &UserWhiteListEntry> {
        self.list.entries()
    }

    pub fn notifications(&self) -> &N {
        &self.notifications
    }
}

/// Notification callbacks used by `ServerOpList.add/remove/clear`.
pub trait OperatorNotifications {
    fn player_oped(&mut self, operator: &ServerOpListEntry);
    fn player_deoped(&mut self, operator: &ServerOpListEntry);
}

impl OperatorNotifications for () {
    fn player_oped(&mut self, _operator: &ServerOpListEntry) {}
    fn player_deoped(&mut self, _operator: &ServerOpListEntry) {}
}

/// Java `ServerOpListEntry` with level-based permissions and player-limit bypass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOpListEntry {
    pub user: Option<NameAndId>,
    pub permissions: PermissionLevel,
    pub bypasses_player_limit: bool,
}

impl ServerOpListEntry {
    pub fn new(user: NameAndId, permissions: PermissionLevel, bypasses_player_limit: bool) -> Self {
        Self {
            user: Some(user),
            permissions,
            bypasses_player_limit,
        }
    }

    pub fn from_json(object: &Map<String, Value>) -> Self {
        let level = object.get("level").and_then(Value::as_i64).map_or(PermissionLevel::All, |level| match level {
            ..=0 => PermissionLevel::All,
            1 => PermissionLevel::Moderators,
            2 => PermissionLevel::Gamemasters,
            3 => PermissionLevel::Admins,
            _ => PermissionLevel::Owners,
        });
        Self {
            user: NameAndId::from_json(&Value::Object(object.clone())),
            permissions: level,
            bypasses_player_limit: object.get("bypassesPlayerLimit").and_then(Value::as_bool).unwrap_or(false),
        }
    }

    pub fn user(&self) -> Option<&NameAndId> {
        self.user.as_ref()
    }

    pub fn get_bypasses_player_limit(&self) -> bool {
        self.bypasses_player_limit
    }
}

impl StoredUserEntry<NameAndId> for ServerOpListEntry {
    fn user(&self) -> Option<&NameAndId> {
        self.user()
    }

    fn has_expired(&self, _now: DateTime<Local>) -> bool {
        false
    }

    fn serialize(&self) -> Value {
        let Some(user) = self.user() else {
            return Value::Object(Map::new());
        };
        let mut object = Map::new();
        user.append_to(&mut object);
        object.insert("level".to_string(), Value::from(self.permissions as i64));
        object.insert("bypassesPlayerLimit".to_string(), Value::Bool(self.bypasses_player_limit));
        Value::Object(object)
    }
}

/// Java `ServerOpList` facade over a UUID-keyed stored list.
#[derive(Debug, Clone)]
pub struct ServerOpList<N = ()> {
    list: StoredUserList<NameAndId, ServerOpListEntry>,
    notifications: N,
}

impl<N> ServerOpList<N>
where
    N: OperatorNotifications,
{
    pub fn new(file: impl Into<PathBuf>, notifications: N) -> Self {
        Self {
            list: StoredUserList::new(file),
            notifications,
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        self.list.load(ServerOpListEntry::from_json)
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.list.save()
    }

    pub fn get_user_list(&self) -> Vec<String> {
        self.list
            .entries()
            .filter_map(ServerOpListEntry::user)
            .map(|user| user.name.clone())
            .collect()
    }

    pub fn add(&mut self, entry: ServerOpListEntry) -> std::io::Result<bool> {
        let changed = self.list.add(entry.clone())?;
        if changed && entry.user().is_some() {
            self.notifications.player_oped(&entry);
        }
        Ok(changed)
    }

    pub fn remove(&mut self, user: &NameAndId, now: DateTime<Local>) -> std::io::Result<bool> {
        let entry = self.list.get(user, now).cloned();
        let changed = self.list.remove(user)?;
        if changed {
            if let Some(entry) = entry {
                self.notifications.player_deoped(&entry);
            }
        }
        Ok(changed)
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        let entries: Vec<ServerOpListEntry> = self.list.entries().filter(|entry| entry.user().is_some()).cloned().collect();
        for entry in entries {
            self.notifications.player_deoped(&entry);
        }
        self.list.clear()
    }

    pub fn can_bypass_player_limit(&mut self, user: &NameAndId, now: DateTime<Local>) -> bool {
        self.list.get(user, now).is_some_and(ServerOpListEntry::get_bypasses_player_limit)
    }

    pub fn contains(&mut self, user: &NameAndId) -> bool {
        self.list.get(user, Local::now()).is_some()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ServerOpListEntry> {
        self.list.entries()
    }

    pub fn notifications(&self) -> &N {
        &self.notifications
    }
}

fn parse_date(value: &Value) -> Option<DateTime<Local>> {
    parse_date_text(value.as_str()?)
}

fn parse_date_text(value: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_str(value, DATE_FORMAT)
        .ok()
        .map(|date| date.with_timezone(&Local))
}

fn ip_from_socket_address(address: &str) -> String {
    let mut ip = address.to_string();
    if let Some(index) = ip.find('/') {
        ip = ip[index + 1..].to_string();
    }
    if let Some(index) = ip.find(':') {
        ip.truncate(index);
    }
    ip
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[cfg(vibecraft_has_decompiled_sources)]
    const STORED_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/StoredUserEntry.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const STORED_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/StoredUserList.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const BAN_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/BanListEntry.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const IP_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/IpBanList.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const IP_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/IpBanListEntry.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const USER_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/UserBanList.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const USER_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/UserBanListEntry.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const WHITE_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/UserWhiteList.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const WHITE_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/UserWhiteListEntry.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const OP_LIST_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/ServerOpList.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const OP_ENTRY_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/players/ServerOpListEntry.java");

    #[derive(Default, Debug)]
    struct Notifications {
        banned: RefCell<Vec<String>>,
        unbanned: RefCell<Vec<String>>,
    }

    impl IpBanNotifications for Notifications {
        fn ip_banned(&mut self, ban: &IpBanListEntry) {
            self.banned.borrow_mut().push(ban.display_name());
        }

        fn ip_unbanned(&mut self, ip: &str) {
            self.unbanned.borrow_mut().push(ip.to_string());
        }
    }

    #[derive(Default, Debug)]
    struct PlayerNotifications {
        banned: RefCell<Vec<String>>,
        unbanned: RefCell<Vec<String>>,
    }

    impl PlayerBanNotifications for PlayerNotifications {
        fn player_banned(&mut self, ban: &UserBanListEntry) {
            self.banned.borrow_mut().push(ban.display_name());
        }

        fn player_unbanned(&mut self, player: &NameAndId) {
            self.unbanned.borrow_mut().push(player.name.clone());
        }
    }

    impl AllowlistNotifications for PlayerNotifications {
        fn player_added(&mut self, player: &NameAndId) {
            self.banned.borrow_mut().push(player.name.clone());
        }

        fn player_removed(&mut self, player: &NameAndId) {
            self.unbanned.borrow_mut().push(player.name.clone());
        }
    }

    impl OperatorNotifications for PlayerNotifications {
        fn player_oped(&mut self, operator: &ServerOpListEntry) {
            self.banned.borrow_mut().push(operator.user().map_or_else(|| "null".to_string(), |user| user.name.clone()));
        }

        fn player_deoped(&mut self, operator: &ServerOpListEntry) {
            self.unbanned.borrow_mut().push(operator.user().map_or_else(|| "null".to_string(), |user| user.name.clone()));
        }
    }

    fn fixture_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("vibecraft-{name}-{}.json", std::process::id()))
    }

    fn now() -> DateTime<Local> {
        Local::now()
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn source_shapes_match_all_five_java_classes() {
        for (source, fragments) in [
            (STORED_ENTRY_JAVA, &["private final @Nullable T user", "boolean hasExpired()", "protected abstract void serialize"][..]),
            (STORED_LIST_JAVA, &["Map<String, V> map", "public boolean add", "removeExpired", "GSON.toJson"][..]),
            (BAN_ENTRY_JAVA, &["EXPIRES_NEVER = \"forever\"", "(Unknown)", "getReasonMessage", "hasExpired"][..]),
            (IP_LIST_JAVA, &["extends StoredUserList<String, IpBanListEntry>", "getIpFromAddress", "notificationService.ipBanned", "notificationService.ipUnbanned"][..]),
            (IP_ENTRY_JAVA, &["extends BanListEntry<String>", "object.has(\"ip\")", "Component.literal(String.valueOf(this.getUser()))"][..]),
            (USER_LIST_JAVA, &["extends StoredUserList<NameAndId, UserBanListEntry>", "getUserList", "notificationService.playerBanned", "notificationService.playerUnbanned"][..]),
            (USER_ENTRY_JAVA, &["extends BanListEntry<NameAndId>", "NameAndId.fromJson(object)", "commands.banlist.entry.unknown"][..]),
            (WHITE_LIST_JAVA, &["extends StoredUserList<NameAndId, UserWhiteListEntry>", "isWhiteListed", "notificationService.playerAddedToAllowlist", "notificationService.playerRemovedFromAllowlist"][..]),
            (WHITE_ENTRY_JAVA, &["extends StoredUserEntry<NameAndId>", "NameAndId.fromJson(object)", "this.getUser().appendTo(object)"][..]),
            (OP_LIST_JAVA, &["extends StoredUserList<NameAndId, ServerOpListEntry>", "canBypassPlayerLimit", "notificationService.playerOped", "notificationService.playerDeoped"][..]),
            (OP_ENTRY_JAVA, &["LevelBasedPermissionSet permissions", "PermissionLevel.byId", "bypassesPlayerLimit", "getBypassesPlayerLimit"][..]),
        ] {
            for fragment in fragments {
                assert!(source.contains(fragment), "missing Java source fragment: {fragment}");
            }
        }
    }

    #[test]
    fn ip_list_matches_java_add_replace_expiry_notifications_and_socket_parsing() {
        let path = fixture_path("ip-list");
        let _ignored = fs::remove_file(&path);
        let notifications = Notifications::default();
        let mut list = IpBanList::new(&path, notifications);
        let current = now();
        let entry = IpBanListEntry::with_details(Some("127.0.0.1".to_string()), Some(current), Some("Console".to_string()), Some(current + chrono::Duration::hours(1)), Some("test".to_string()));
        assert!(list.add(entry.clone()).expect("add IP ban"));
        assert!(!list.add(entry).expect("duplicate add"));
        assert!(list.is_banned_socket_address("/127.0.0.1:25565", current));
        assert!(list.get_socket_address("/127.0.0.1:25565", current).is_some());
        assert_eq!(list.notifications().banned.borrow().as_slice(), ["127.0.0.1"]);
        assert!(list.remove("127.0.0.1").expect("remove IP ban"));
        assert_eq!(list.notifications().unbanned.borrow().as_slice(), ["127.0.0.1"]);

        let expired = IpBanListEntry::with_details(Some("192.0.2.1".to_string()), Some(current), None, Some(current - chrono::Duration::seconds(1)), None);
        assert!(list.add(expired).expect("add expired ban"));
        assert!(!list.is_banned("192.0.2.1", current));
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn ip_entry_round_trips_vanilla_json_and_null_users_are_not_inserted() {
        let path = fixture_path("ip-load");
        let _ignored = fs::remove_file(&path);
        fs::write(&path, r#"[{"ip":"203.0.113.5","created":"2026-07-16 12:00:00 +0000","source":"Server","expires":"forever","reason":null},{"created":"2026-07-16 12:00:00 +0000"}]"#).expect("write fixture");
        let mut list = IpBanList::new(&path, Notifications::default());
        list.load().expect("load IP bans");
        assert_eq!(list.entries().count(), 1);
        let saved: Value = serde_json::from_str(&fs::read_to_string(&path).expect("saved JSON")).expect("valid saved JSON");
        assert_eq!(saved[0]["ip"], "203.0.113.5");
        assert_eq!(saved[0]["expires"], EXPIRES_NEVER);
        assert_eq!(IpBanListEntry::from_json(saved[0].as_object().expect("object")).display_name(), "203.0.113.5");
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn ban_entry_defaults_and_equality_match_java() {
        let one = IpBanListEntry::new("198.51.100.1");
        let two = IpBanListEntry::with_details(Some("198.51.100.1".to_string()), Some(one.common.created + chrono::Duration::days(1)), Some("(Unknown)".to_string()), None, None);
        assert_eq!(one, two);
        assert_eq!(one.common.source, "(Unknown)");
        assert_eq!(one.common.reason_message(), None);
    }

    #[test]
    fn user_ban_list_keys_by_uuid_lists_names_and_notifies() {
        let path = fixture_path("user-list");
        let _ignored = fs::remove_file(&path);
        let user = NameAndId::create_offline("Steve");
        let replacement = UserBanListEntry::with_details(
            Some(user.clone()),
            Some(now()),
            Some("Console".to_string()),
            None,
            Some("test".to_string()),
        );
        let notifications = PlayerNotifications::default();
        let mut list = UserBanList::new(&path, notifications);
        assert!(list.add(replacement.clone()).expect("add user ban"));
        assert!(!list.add(replacement).expect("duplicate user ban"));
        assert!(list.is_banned(&user, now()));
        assert_eq!(list.get_user_list(), vec!["Steve".to_string()]);
        assert_eq!(list.notifications().banned.borrow().as_slice(), ["Steve"]);
        assert!(list.remove(&user).expect("remove user ban"));
        assert_eq!(list.notifications().unbanned.borrow().as_slice(), ["Steve"]);

        let unknown = UserBanListEntry::new(None);
        assert_eq!(unknown.display_name(), "commands.banlist.entry.unknown");
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn user_ban_entry_round_trips_name_and_id_json() {
        let path = fixture_path("user-load");
        let _ignored = fs::remove_file(&path);
        let user = NameAndId::create_offline("Alex");
        let entry = UserBanListEntry::with_details(Some(user.clone()), Some(now()), None, None, None);
        let value = entry.serialize();
        let object = value.as_object().expect("serialized user ban object");
        assert_eq!(UserBanListEntry::from_json(object).user(), Some(&user));
        assert_eq!(object["uuid"], user.uuid.as_str());
        assert_eq!(object["name"], "Alex");
    }

    #[test]
    fn user_whitelist_is_uuid_keyed_non_expiring_and_notifies() {
        let path = fixture_path("white-list");
        let _ignored = fs::remove_file(&path);
        let user = NameAndId::create_offline("Alex");
        let notifications = PlayerNotifications::default();
        let mut list = UserWhiteList::new(&path, notifications);
        assert!(list.add(UserWhiteListEntry::new(user.clone())).expect("add allowlist entry"));
        assert!(!list.add(UserWhiteListEntry::new(user.clone())).expect("duplicate allowlist entry"));
        assert!(list.is_white_listed(&user, now()));
        assert_eq!(list.get_user_list(), vec!["Alex".to_string()]);
        assert_eq!(list.notifications().banned.borrow().as_slice(), ["Alex"]);
        assert!(list.remove(&user).expect("remove allowlist entry"));
        assert_eq!(list.notifications().unbanned.borrow().as_slice(), ["Alex"]);

        let unknown = UserWhiteListEntry::from_json(&Map::new());
        assert!(unknown.user().is_none());
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn server_op_list_persists_permission_level_and_bypass_flag() {
        let path = fixture_path("op-list");
        let _ignored = fs::remove_file(&path);
        let user = NameAndId::create_offline("Operator");
        let notifications = PlayerNotifications::default();
        let mut list = ServerOpList::new(&path, notifications);
        let entry = ServerOpListEntry::new(user.clone(), PermissionLevel::Admins, true);
        assert!(list.add(entry.clone()).expect("add operator"));
        assert!(!list.add(entry).expect("duplicate operator"));
        assert!(list.can_bypass_player_limit(&user, now()));
        assert_eq!(list.get_user_list(), vec!["Operator".to_string()]);
        assert_eq!(list.notifications().banned.borrow().as_slice(), ["Operator"]);

        let saved: Value = serde_json::from_str(&fs::read_to_string(&path).expect("saved operators")).expect("valid operator JSON");
        assert_eq!(saved[0]["level"], 3);
        assert_eq!(saved[0]["bypassesPlayerLimit"], true);

        assert!(list.remove(&user, now()).expect("remove operator"));
        assert_eq!(list.notifications().unbanned.borrow().as_slice(), ["Operator"]);
        let _ignored = fs::remove_file(path);
    }

    #[test]
    fn server_op_entry_defaults_missing_level_to_all_and_clamps_ids() {
        let user = NameAndId::create_offline("Alex");
        let mut object = Map::new();
        user.append_to(&mut object);
        let default = ServerOpListEntry::from_json(&object);
        assert_eq!(default.permissions, PermissionLevel::All);
        object.insert("level".to_string(), Value::from(999));
        let clamped = ServerOpListEntry::from_json(&object);
        assert_eq!(clamped.permissions, PermissionLevel::Owners);
        assert!(!clamped.get_bypasses_player_limit());
    }
}
