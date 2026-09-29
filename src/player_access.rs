use std::fs;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::block_update::BlockPos;
use chrono::{DateTime, Local};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameAndId {
    pub uuid: String,
    pub name: String,
}

impl fmt::Display for NameAndId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&normalize_uuid(&self.uuid).unwrap_or_else(|| self.uuid.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanEntry<T> {
    pub user: T,
    pub created: String,
    pub source: String,
    pub expires: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpEntry {
    pub user: NameAndId,
    pub level: u8,
    pub bypasses_player_limit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyConnectionDecision {
    Allow,
    RejectPreventProxyConnections,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnProtection {
    pub radius: u32,
    pub spawn_dimension: String,
    pub spawn_pos: BlockPos,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpLogPolicy {
    Include,
    Redact,
}

#[cfg(test)]
impl IpLogPolicy {
    pub fn format_remote(self, ip: &str) -> String {
        match self {
            Self::Include => ip.to_string(),
            Self::Redact => "IP hidden".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerAccess {
    banned_players: Vec<BanEntry<NameAndId>>,
    banned_ips: Vec<BanEntry<String>>,
    whitelist: Vec<NameAndId>,
    ops: Vec<OpEntry>,
    user_cache: Vec<NameAndId>,
    /// Java `MinecraftServer.usingWhitelist` (the `white-list` property, toggled live by
    /// `/whitelist on|off`).
    using_whitelist: bool,
    /// Directory the JSON lists were loaded from and are written back to (Java's server
    /// root, where `StoredUserList.save` writes). `None` keeps the lists memory-only.
    storage_dir: Option<PathBuf>,
    #[cfg(test)]
    profile_cache: ProfileCache,
}

impl PlayerAccess {
    pub fn load_from_dir(dir: &Path) -> std::io::Result<Self> {
        let mut access = Self {
            banned_players: load_name_ban_entries(&dir.join("banned-players.json"))?,
            banned_ips: load_ip_ban_entries(&dir.join("banned-ips.json"))?,
            whitelist: load_name_and_id_entries(&dir.join("whitelist.json"))?,
            ops: load_op_entries(&dir.join("ops.json"))?,
            storage_dir: Some(dir.to_path_buf()),
            ..Self::default()
        };
        let cached = load_user_cache_entries(&dir.join("usercache.json"), SystemTime::now())?;
        for user in cached {
            access.cache_user(user);
        }
        Ok(access)
    }

    /// Java `UserBanList.add` (`StoredUserList.add`): stores `entry`, replacing any entry for
    /// the same user, and saves `banned-players.json`. Returns false (and writes nothing) when
    /// an equal entry (same user, source, expiry and reason) is already present.
    pub fn ban_player(&mut self, entry: BanEntry<NameAndId>) -> bool {
        let previous = self
            .banned_players
            .iter()
            .position(|existing| existing.user.uuid == entry.user.uuid);
        if previous.is_some_and(|index| same_ban(&self.banned_players[index], &entry)) {
            return false;
        }
        self.banned_players
            .retain(|existing| existing.user.uuid != entry.user.uuid);
        self.banned_players.push(entry);
        self.save_list("banned-players.json", self.banned_players_json());
        true
    }

    /// Java `IpBanList.add`; see [`Self::ban_player`].
    pub fn ban_ip(&mut self, entry: BanEntry<String>) -> bool {
        let previous = self
            .banned_ips
            .iter()
            .position(|existing| existing.user == entry.user);
        if previous.is_some_and(|index| same_ban(&self.banned_ips[index], &entry)) {
            return false;
        }
        self.banned_ips.retain(|existing| existing.user != entry.user);
        self.banned_ips.push(entry);
        self.save_list("banned-ips.json", self.banned_ips_json());
        true
    }

    /// Java `UserBanList.remove` (`/pardon`): true when an entry was removed.
    pub fn pardon_player(&mut self, uuid: &str) -> bool {
        let old_len = self.banned_players.len();
        self.banned_players.retain(|entry| entry.user.uuid != uuid);
        let removed = self.banned_players.len() != old_len;
        if removed {
            self.save_list("banned-players.json", self.banned_players_json());
        }
        removed
    }

    /// Java `IpBanList.remove` (`/pardon-ip`): true when an entry was removed.
    pub fn pardon_ip(&mut self, ip: &str) -> bool {
        let old_len = self.banned_ips.len();
        self.banned_ips.retain(|entry| entry.user != ip);
        let removed = self.banned_ips.len() != old_len;
        if removed {
            self.save_list("banned-ips.json", self.banned_ips_json());
        }
        removed
    }

    /// Java `UserWhiteList.add`: true when the user was newly listed.
    pub fn whitelist(&mut self, user: NameAndId) -> bool {
        if self.is_whitelisted(&user.uuid) {
            return false;
        }
        self.whitelist.push(user);
        self.save_list("whitelist.json", self.whitelist_json());
        true
    }

    /// Java `UserWhiteList.remove`: true when the user was listed.
    pub fn unwhitelist(&mut self, uuid: &str) -> bool {
        let old_len = self.whitelist.len();
        self.whitelist.retain(|entry| entry.uuid != uuid);
        let removed = self.whitelist.len() != old_len;
        if removed {
            self.save_list("whitelist.json", self.whitelist_json());
        }
        removed
    }

    /// Java `ServerOpList.add`: stores `entry` (replacing any entry for the same user) and
    /// saves `ops.json`. Returns false when an identical entry already exists.
    pub fn op(&mut self, entry: OpEntry) -> bool {
        if self.ops.contains(&entry) {
            return false;
        }
        self.ops
            .retain(|existing| existing.user.uuid != entry.user.uuid);
        self.ops.push(entry);
        self.save_list("ops.json", self.ops_json());
        true
    }

    /// Java `ServerOpList.remove` (`PlayerList.deop`): true when the user was an operator.
    pub fn deop(&mut self, uuid: &str) -> bool {
        let old_len = self.ops.len();
        self.ops.retain(|entry| entry.user.uuid != uuid);
        let removed = self.ops.len() != old_len;
        if removed {
            self.save_list("ops.json", self.ops_json());
        }
        removed
    }

    /// Java `DedicatedPlayerList.reloadWhiteList` (`UserWhiteList.load`): re-reads
    /// `whitelist.json` from the storage directory. A memory-only list is left untouched.
    pub fn reload_whitelist(&mut self) -> std::io::Result<()> {
        if let Some(dir) = &self.storage_dir {
            self.whitelist = load_name_and_id_entries(&dir.join("whitelist.json"))?;
        }
        Ok(())
    }

    /// Re-reads every list file from the storage directory (operator `reload`): the ban, op
    /// and whitelist lists follow the files while `usercache.json` and the live `white-list`
    /// flag are kept.
    pub fn reload_lists(&mut self) -> std::io::Result<()> {
        let Some(dir) = self.storage_dir.clone() else { return Ok(()) };
        self.banned_players = load_name_ban_entries(&dir.join("banned-players.json"))?;
        self.banned_ips = load_ip_ban_entries(&dir.join("banned-ips.json"))?;
        self.whitelist = load_name_and_id_entries(&dir.join("whitelist.json"))?;
        self.ops = load_op_entries(&dir.join("ops.json"))?;
        Ok(())
    }

    /// Java `MinecraftServer.isUsingWhitelist`.
    pub fn using_whitelist(&self) -> bool {
        self.using_whitelist
    }

    /// Java `MinecraftServer.setUsingWhitelist`.
    pub fn set_using_whitelist(&mut self, using: bool) {
        self.using_whitelist = using;
    }

    /// Writes one list file; failures are logged like Java's `StoredUserList.save` callers
    /// (`LOGGER.warn("Failed to save ...")`) and never abort the command.
    fn save_list(&self, file: &str, contents: String) {
        let Some(dir) = &self.storage_dir else { return };
        if let Err(err) = fs::create_dir_all(dir).and_then(|()| fs::write(dir.join(file), contents))
        {
            eprintln!("Failed to save {file}: {err}");
        }
    }

    /// The unexpired user bans (Java `UserBanList.getEntries` after `removeExpired`).
    pub fn banned_players(&self) -> Vec<&BanEntry<NameAndId>> {
        let now = Local::now();
        self.banned_players
            .iter()
            .filter(|entry| !ban_has_expired(entry, now))
            .collect()
    }

    /// The unexpired IP bans (Java `IpBanList.getEntries` after `removeExpired`).
    pub fn banned_ips(&self) -> Vec<&BanEntry<String>> {
        let now = Local::now();
        self.banned_ips
            .iter()
            .filter(|entry| !ban_has_expired(entry, now))
            .collect()
    }

    /// The whitelisted users (Java `UserWhiteList.getEntries`).
    pub fn whitelisted(&self) -> &[NameAndId] {
        &self.whitelist
    }

    /// The operator entries (Java `ServerOpList.getEntries`).
    pub fn operators(&self) -> &[OpEntry] {
        &self.ops
    }

    /// Java `ServerOpList.canBypassPlayerLimit`.
    pub fn can_bypass_player_limit(&self, uuid: &str) -> bool {
        self.ops
            .iter()
            .find(|entry| entry.user.uuid == uuid)
            .is_some_and(|entry| entry.bypasses_player_limit)
    }

    /// The cached profiles (Java `GameProfileCache` entries), newest last.
    pub fn cached_users(&self) -> &[NameAndId] {
        &self.user_cache
    }

    pub fn cache_user(&mut self, user: NameAndId) {
        self.user_cache
            .retain(|existing| existing.uuid != user.uuid);
        #[cfg(test)]
        self.profile_cache.insert(user.clone(), SystemTime::now());
        self.user_cache.push(user);
    }

    #[cfg(test)]
    pub fn lookup_cached_profile(&mut self, name: &str, now: SystemTime) -> Option<NameAndId> {
        self.profile_cache.lookup(name, now)
    }

    /// The live (unexpired) ban of `uuid`, Java `UserBanList.get`.
    pub fn player_ban(&self, uuid: &str) -> Option<&BanEntry<NameAndId>> {
        let now = Local::now();
        self.banned_players
            .iter()
            .find(|entry| entry.user.id() == uuid && !ban_has_expired(entry, now))
    }

    /// The live (unexpired) ban of `ip`, Java `IpBanList.get`.
    pub fn ip_ban(&self, ip: &str) -> Option<&BanEntry<String>> {
        let now = Local::now();
        self.banned_ips
            .iter()
            .find(|entry| entry.user == ip && !ban_has_expired(entry, now))
    }

    #[cfg(test)]
    pub fn is_player_banned(&self, uuid: &str) -> bool {
        self.player_ban(uuid).is_some()
    }

    #[cfg(test)]
    pub fn is_ip_banned(&self, ip: &str) -> bool {
        self.ip_ban(ip).is_some()
    }

    pub fn is_whitelisted(&self, uuid: &str) -> bool {
        self.whitelist.iter().any(|entry| entry.uuid == uuid)
    }

    pub fn op_level(&self, uuid: &str) -> Option<u8> {
        self.ops
            .iter()
            .find(|entry| entry.user.uuid == uuid)
            .map(|entry| entry.level)
    }

    pub fn has_ops(&self) -> bool {
        !self.ops.is_empty()
    }

    pub fn is_op(&self, uuid: &str) -> bool {
        self.op_level(uuid).is_some()
    }

    /// 1:1 with Java `DedicatedServer.isUnderSpawnProtection` (gated by
    /// `ServerLevel.mayInteract`): true when the block at `pos` is within the
    /// `spawn-protection` radius of the world spawn and the player is a non-op on
    /// a server that has operators. Wired into the live block-break handler.
    pub fn is_under_spawn_protection(
        &self,
        protection: &SpawnProtection,
        dimension: &str,
        pos: BlockPos,
        player_uuid: &str,
    ) -> bool {
        if dimension != protection.spawn_dimension
            || !self.has_ops()
            || self.is_op(player_uuid)
            || protection.radius == 0
        {
            return false;
        }

        let xd = pos.x.abs_diff(protection.spawn_pos.x);
        let zd = pos.z.abs_diff(protection.spawn_pos.z);
        xd.max(zd) <= protection.radius
    }

    pub fn check_proxy_connection(
        &self,
        prevent_proxy_connections: bool,
        resolved_login_host_ip: &str,
        remote_socket_ip: &str,
    ) -> ProxyConnectionDecision {
        if prevent_proxy_connections && resolved_login_host_ip != remote_socket_ip {
            ProxyConnectionDecision::RejectPreventProxyConnections
        } else {
            ProxyConnectionDecision::Allow
        }
    }

    #[cfg(test)]
    pub fn ip_log_policy(&self, log_ips: bool) -> IpLogPolicy {
        if log_ips {
            IpLogPolicy::Include
        } else {
            IpLogPolicy::Redact
        }
    }

    #[cfg(test)]
    pub fn save_all(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("banned-players.json"), self.banned_players_json())?;
        fs::write(dir.join("banned-ips.json"), self.banned_ips_json())?;
        fs::write(dir.join("whitelist.json"), self.whitelist_json())?;
        fs::write(dir.join("ops.json"), self.ops_json())?;
        self.save_user_cache(dir)?;
        Ok(())
    }

    pub fn save_user_cache(&self, dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("usercache.json"), self.user_cache_json())?;
        Ok(())
    }

    fn banned_players_json(&self) -> String {
        gson_pretty(
            self.banned_players
                .iter()
                .map(|entry| {
                    let mut fields = name_and_id_fields(&entry.user);
                    append_ban_fields(entry, &mut fields);
                    fields
                })
                .collect(),
        )
    }

    fn banned_ips_json(&self) -> String {
        gson_pretty(
            self.banned_ips
                .iter()
                .map(|entry| {
                    let mut fields = vec![("ip", Value::String(entry.user.clone()))];
                    append_ban_fields(entry, &mut fields);
                    fields
                })
                .collect(),
        )
    }

    fn whitelist_json(&self) -> String {
        gson_pretty(self.whitelist.iter().map(name_and_id_fields).collect())
    }

    /// Java `ServerOpListEntry.serialize`.
    fn ops_json(&self) -> String {
        gson_pretty(
            self.ops
                .iter()
                .map(|entry| {
                    let mut fields = name_and_id_fields(&entry.user);
                    fields.push(("level", Value::from(entry.level)));
                    fields.push(("bypassesPlayerLimit", Value::Bool(entry.bypasses_player_limit)));
                    fields
                })
                .collect(),
        )
    }

    fn user_cache_json(&self) -> String {
        let expires_on = user_cache_expires_on(SystemTime::now());
        json_array(
            self.user_cache
                .iter()
                .map(|user| {
                    let Value::Object(mut object) = user.to_json_value() else {
                        return "{}".to_string();
                    };
                    object.insert("expiresOn".to_string(), Value::String(expires_on.clone()));
                    serde_json::to_string(&Value::Object(object))
                        .unwrap_or_else(|_| "{}".to_string())
                })
                .collect(),
        )
    }
}

impl NameAndId {
    /// Returns the UUID string represented by Java's `NameAndId.id()` record
    /// accessor. UUIDs are stored as strings in this Rust port because the
    /// surrounding player-list and persistence APIs use textual identifiers.
    pub fn id(&self) -> &str {
        &self.uuid
    }

    /// Reads the legacy player-list representation used by Java
    /// `NameAndId.fromJson`: `{ "uuid": "...", "name": "..." }`.
    pub fn from_json(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let uuid = normalize_uuid(object.get("uuid")?.as_str()?)?;
        let name = object.get("name")?.as_str()?.to_string();
        Some(Self { uuid, name })
    }

    /// Appends this record to the legacy JSON shape used by ban, op, and
    /// whitelist files. Existing fields are intentionally overwritten, just as
    /// Gson's `JsonObject.addProperty` does.
    pub fn append_to(&self, output: &mut Map<String, Value>) {
        output.insert(
            "uuid".to_string(),
            Value::String(normalize_uuid(&self.uuid).unwrap_or_else(|| self.uuid.clone())),
        );
        output.insert("name".to_string(), Value::String(self.name.clone()));
    }

    pub fn to_json_value(&self) -> Value {
        let mut output = Map::new();
        self.append_to(&mut output);
        Value::Object(output)
    }

    /// Encodes the `NameAndId.CODEC` record shape: `{ "id": "...", "name":
    /// "..." }`. Unlike the legacy JSON helper, this returns an error for a
    /// malformed UUID because Java's `UUIDUtil.STRING_CODEC` does.
    #[allow(dead_code)]
    pub fn to_codec_value(&self) -> Result<Value, String> {
        let uuid = normalize_uuid(&self.uuid)
            .ok_or_else(|| format!("Invalid UUID {}", self.uuid))?;
        let mut output = Map::new();
        output.insert("id".to_string(), Value::String(uuid));
        output.insert("name".to_string(), Value::String(self.name.clone()));
        Ok(Value::Object(output))
    }

    #[allow(dead_code)]
    pub fn from_codec_value(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "NameAndId must be a JSON object".to_string())?;
        let uuid = object
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "NameAndId.id must be a string".to_string())?;
        let uuid = normalize_uuid(uuid).ok_or_else(|| format!("Invalid UUID {uuid}"))?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| "NameAndId.name must be a string".to_string())?;
        Ok(Self {
            uuid,
            name: name.to_string(),
        })
    }

    pub fn create_offline(name: &str) -> Self {
        Self {
            uuid: offline_player_uuid(name),
            name: name.to_string(),
        }
    }
}

fn normalize_uuid(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() != 36 || ![8, 13, 18, 23].iter().all(|&index| bytes[index] == b'-') {
        return None;
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| !matches!(index, 8 | 13 | 18 | 23) && !byte.is_ascii_hexdigit())
    {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

fn offline_player_uuid(name: &str) -> String {
    let digest = md5(format!("OfflinePlayer:{name}").as_bytes());
    let mut bytes = digest;
    bytes[6] = (bytes[6] & 0x0F) | 0x30;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
    format_uuid(bytes)
}

fn format_uuid(bytes: [u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn md5(input: &[u8]) -> [u8; 16] {
    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    for chunk in message.chunks_exact(64) {
        let mut m = [0u32; 16];
        for (i, word) in m.iter_mut().enumerate() {
            let start = i * 4;
            *word = u32::from_le_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | ((!b) & d), i)
            } else if i < 32 {
                ((d & b) | ((!d) & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | (!d)), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&a0.to_le_bytes());
    out[4..8].copy_from_slice(&b0.to_le_bytes());
    out[8..12].copy_from_slice(&c0.to_le_bytes());
    out[12..16].copy_from_slice(&d0.to_le_bytes());
    out
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct ProfileCache {
    entries: Vec<ProfileCacheEntry>,
    ttl: Duration,
}

#[cfg(test)]
impl Default for ProfileCache {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            ttl: Duration::from_secs(60 * 60 * 24 * 30),
        }
    }
}

#[cfg(test)]
impl ProfileCache {
    pub fn insert(&mut self, profile: NameAndId, now: SystemTime) {
        self.entries
            .retain(|entry| !entry.profile.name.eq_ignore_ascii_case(&profile.name));
        self.entries.push(ProfileCacheEntry {
            profile,
            cached_at: now,
        });
    }

    pub fn lookup(&mut self, name: &str, now: SystemTime) -> Option<NameAndId> {
        let ttl = self.ttl;
        self.entries.retain(|entry| {
            now.duration_since(entry.cached_at)
                .map(|age| age <= ttl)
                .unwrap_or(true)
        });
        self.entries
            .iter()
            .find(|entry| entry.profile.name.eq_ignore_ascii_case(name))
            .map(|entry| entry.profile.clone())
    }
}

#[cfg(test)]
#[derive(Debug, Clone)]
struct ProfileCacheEntry {
    profile: NameAndId,
    cached_at: SystemTime,
}

fn user_cache_expires_on(now: SystemTime) -> String {
    let expires = now + Duration::from_secs(60 * 60 * 24 * 30);
    let seconds = expires
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} +0000")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

fn parse_user_cache_expires_on(value: &str) -> Option<SystemTime> {
    let (date, rest) = value.split_once(' ')?;
    let (time, offset) = rest.split_once(' ')?;
    if offset != "+0000" {
        return None;
    }
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;
    if date_parts.next().is_some() {
        return None;
    }
    let mut time_parts = time.split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: i64 = time_parts.next()?.parse().ok()?;
    if time_parts.next().is_some()
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let seconds = days
        .checked_mul(86_400)?
        .checked_add(hour * 3_600 + minute * 60 + second)?;
    (seconds >= 0).then(|| UNIX_EPOCH + Duration::from_secs(seconds as u64))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 }.div_euclid(400);
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2).div_euclid(5) + day - 1;
    let doe = yoe * 365 + yoe.div_euclid(4) - yoe.div_euclid(100) + doy;
    era * 146_097 + doe - 719_468
}

fn json_array(entries: Vec<String>) -> String {
    if entries.is_empty() {
        "[]\n".to_string()
    } else {
        format!("[\n  {}\n]\n", entries.join(",\n  "))
    }
}

/// Java `BanListEntry.DATE_FORMAT` (`yyyy-MM-dd HH:mm:ss Z`).
pub const BAN_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// `BanListEntry.DATE_FORMAT.format(new Date())`: the current time for a fresh ban's `created`.
pub fn ban_timestamp_now() -> String {
    Local::now().format(BAN_DATE_FORMAT).to_string()
}

/// `BanListEntry.equals`: user, source, expiry and reason (not the creation time).
fn same_ban<T: PartialEq>(left: &BanEntry<T>, right: &BanEntry<T>) -> bool {
    left.user == right.user
        && left.source == right.source
        && left.expires == right.expires
        && left.reason == right.reason
}

/// `BanListEntry.hasExpired`: an unparsable expiry reads as "never" like Java's
/// `ParseException` fallback (`expires = null`).
fn ban_has_expired<T>(entry: &BanEntry<T>, now: DateTime<Local>) -> bool {
    entry
        .expires
        .as_deref()
        .and_then(|text| DateTime::parse_from_str(text, BAN_DATE_FORMAT).ok())
        .is_some_and(|expires| expires < now)
}

/// One serialized list entry: JSON members in Java's insertion order (`JsonObject` keeps it).
type JsonFields = Vec<(&'static str, Value)>;

/// `NameAndId.appendTo`: `uuid` then `name`.
fn name_and_id_fields(user: &NameAndId) -> JsonFields {
    let mut object = Map::new();
    user.append_to(&mut object);
    vec![
        ("uuid", object.remove("uuid").unwrap_or(Value::Null)),
        ("name", Value::String(user.name.clone())),
    ]
}

/// `BanListEntry.serialize`: `reason` is omitted when null (Gson drops `JsonNull` members).
fn append_ban_fields<T>(entry: &BanEntry<T>, fields: &mut JsonFields) {
    fields.push(("created", Value::String(entry.created.clone())));
    fields.push(("source", Value::String(entry.source.clone())));
    fields.push((
        "expires",
        Value::String(entry.expires.clone().unwrap_or_else(|| "forever".to_string())),
    ));
    if let Some(reason) = &entry.reason {
        fields.push(("reason", Value::String(reason.clone())));
    }
}

/// Java `StoredUserList.GSON` output (`setPrettyPrinting`, HTML escaping on): a JSON array of
/// objects with two-space indentation and `< > & = '` written as `\u00XX` escapes.
fn gson_pretty(entries: Vec<JsonFields>) -> String {
    if entries.is_empty() {
        return "[]".to_string();
    }
    let objects: Vec<String> = entries
        .iter()
        .map(|fields| {
            let members: Vec<String> = fields
                .iter()
                .map(|(key, value)| {
                    let value = serde_json::to_string(value).unwrap_or_else(|_| "null".into());
                    format!("    \"{key}\": {}", gson_html_escape(&value))
                })
                .collect();
            format!("  {{\n{}\n  }}", members.join(",\n"))
        })
        .collect();
    format!("[\n{}\n]", objects.join(",\n"))
}

/// Gson's default HTML-safe escaping; those characters only occur inside string values here.
fn gson_html_escape(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('=', "\\u003d")
        .replace('\'', "\\u0027")
}

fn load_name_and_id_entries(path: &Path) -> std::io::Result<Vec<NameAndId>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| serde_json::from_str::<Value>(&object).ok())
        .filter_map(|object| NameAndId::from_json(&object))
        .collect())
}

fn load_user_cache_entries(path: &Path, now: SystemTime) -> std::io::Result<Vec<NameAndId>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| {
            let expires_on = json_string_field(&object, "expiresOn")?;
            let expires_at = parse_user_cache_expires_on(&expires_on)?;
            if expires_at <= now {
                return None;
            }
            Some(NameAndId {
                uuid: json_string_field(&object, "uuid")?,
                name: json_string_field(&object, "name")?,
            })
        })
        .collect())
}

fn load_name_ban_entries(path: &Path) -> std::io::Result<Vec<BanEntry<NameAndId>>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| {
            let user = NameAndId {
                uuid: json_string_field(&object, "uuid")?,
                name: json_string_field(&object, "name")?,
            };
            Some(BanEntry {
                user,
                created: json_string_field(&object, "created").unwrap_or_default(),
                source: json_string_field(&object, "source").unwrap_or_default(),
                expires: json_optional_date(&object),
                reason: json_string_field(&object, "reason"),
            })
        })
        .collect())
}

fn load_ip_ban_entries(path: &Path) -> std::io::Result<Vec<BanEntry<String>>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| {
            Some(BanEntry {
                user: json_string_field(&object, "ip")?,
                created: json_string_field(&object, "created").unwrap_or_default(),
                source: json_string_field(&object, "source").unwrap_or_default(),
                expires: json_optional_date(&object),
                reason: json_string_field(&object, "reason"),
            })
        })
        .collect())
}

fn load_op_entries(path: &Path) -> std::io::Result<Vec<OpEntry>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| {
            let user = NameAndId {
                uuid: json_string_field(&object, "uuid")?,
                name: json_string_field(&object, "name")?,
            };
            Some(OpEntry {
                user,
                level: json_u8_field(&object, "level").unwrap_or(0),
                bypasses_player_limit: json_bool_field(&object, "bypassesPlayerLimit")
                    .unwrap_or(false),
            })
        })
        .collect())
}

fn read_optional(path: &Path) -> std::io::Result<String> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(raw),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err),
    }
}

fn json_optional_date(object: &str) -> Option<String> {
    json_string_field(object, "expires").filter(|value| value != "forever")
}

fn json_objects(raw: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut depth = 0_i32;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in raw.char_indices() {
        if in_string {
            escaped = ch == '\\' && !escaped;
            if ch == '"' && !escaped {
                in_string = false;
            } else if ch != '\\' {
                escaped = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(raw[start..=index].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    objects
}

fn json_string_field(object: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let after_key = object.split_once(&needle)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    parse_json_string(after_colon).map(|(value, _)| value)
}

fn json_u8_field(object: &str, field: &str) -> Option<u8> {
    let needle = format!("\"{field}\"");
    let after_key = object.split_once(&needle)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let digits: String = after_colon
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn json_bool_field(object: &str, field: &str) -> Option<bool> {
    let needle = format!("\"{field}\"");
    let after_key = object.split_once(&needle)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    if after_colon.starts_with("true") {
        Some(true)
    } else if after_colon.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn parse_json_string(input: &str) -> Option<(String, &str)> {
    let mut chars = input.char_indices();
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut out = String::new();
    let mut units: Vec<u16> = Vec::new();
    while let Some((index, ch)) = chars.next() {
        if ch == '"' {
            flush_utf16(&mut out, &mut units);
            return Some((out, &input[index + 1..]));
        }
        if ch != '\\' {
            flush_utf16(&mut out, &mut units);
            out.push(ch);
            continue;
        }
        let (_, escape) = chars.next()?;
        if escape == 'u' {
            // `\uXXXX` is a UTF-16 code unit; pairs are combined by `flush_utf16`.
            let hex: String = chars.by_ref().take(4).map(|(_, digit)| digit).collect();
            units.push(u16::from_str_radix(&hex, 16).ok()?);
            continue;
        }
        flush_utf16(&mut out, &mut units);
        out.push(match escape {
            'b' => '\u{0008}',
            'f' => '\u{000c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            other => other,
        });
    }
    None
}

/// Appends pending `\uXXXX` UTF-16 code units (lone surrogates become U+FFFD like Gson's
/// lenient decoding of malformed text).
fn flush_utf16(out: &mut String, units: &mut Vec<u16>) {
    out.extend(char::decode_utf16(units.drain(..)).map(|unit| unit.unwrap_or('\u{FFFD}')));
}

#[cfg(test)]
mod tests;
