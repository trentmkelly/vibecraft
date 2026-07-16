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
}
