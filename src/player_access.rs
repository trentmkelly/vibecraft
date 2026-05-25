use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameAndId {
    pub uuid: String,
    pub name: String,
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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[cfg(test)]
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
    #[cfg(test)]
    profile_cache: ProfileCache,
}

impl PlayerAccess {
    pub fn load_from_dir(dir: &Path) -> std::io::Result<Self> {
        let mut access = Self::default();
        access.banned_players = load_name_ban_entries(&dir.join("banned-players.json"))?;
        access.banned_ips = load_ip_ban_entries(&dir.join("banned-ips.json"))?;
        access.whitelist = load_name_and_id_entries(&dir.join("whitelist.json"))?;
        access.ops = load_op_entries(&dir.join("ops.json"))?;
        let cached = load_user_cache_entries(&dir.join("usercache.json"), SystemTime::now())?;
        for user in cached {
            access.cache_user(user);
        }
        Ok(access)
    }

    #[cfg(test)]
    pub fn ban_player(&mut self, entry: BanEntry<NameAndId>) {
        self.banned_players
            .retain(|existing| existing.user.uuid != entry.user.uuid);
        self.banned_players.push(entry);
    }

    #[cfg(test)]
    pub fn ban_ip(&mut self, entry: BanEntry<String>) {
        self.banned_ips
            .retain(|existing| existing.user != entry.user);
        self.banned_ips.push(entry);
    }

    #[cfg(test)]
    pub fn whitelist(&mut self, user: NameAndId) {
        self.whitelist.retain(|existing| existing.uuid != user.uuid);
        self.whitelist.push(user);
    }

    #[cfg(test)]
    pub fn op(&mut self, entry: OpEntry) {
        self.ops
            .retain(|existing| existing.user.uuid != entry.user.uuid);
        self.ops.push(entry);
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

    pub fn is_player_banned(&self, uuid: &str) -> bool {
        self.banned_players
            .iter()
            .any(|entry| entry.user.uuid == uuid)
    }

    pub fn is_ip_banned(&self, ip: &str) -> bool {
        self.banned_ips.iter().any(|entry| entry.user == ip)
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

    #[cfg(test)]
    pub fn has_ops(&self) -> bool {
        !self.ops.is_empty()
    }

    pub fn is_op(&self, uuid: &str) -> bool {
        self.op_level(uuid).is_some()
    }

    #[cfg(test)]
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

    #[cfg(test)]
    fn banned_players_json(&self) -> String {
        json_array(
            self.banned_players
                .iter()
                .map(|entry| {
                    format!(
                        "{{\"uuid\":\"{}\",\"name\":\"{}\",\"created\":\"{}\",\"source\":\"{}\",\"expires\":\"{}\",\"reason\":\"{}\"}}",
                        escape(&entry.user.uuid),
                        escape(&entry.user.name),
                        escape(&entry.created),
                        escape(&entry.source),
                        escape(entry.expires.as_deref().unwrap_or("forever")),
                        escape(entry.reason.as_deref().unwrap_or(""))
                    )
                })
                .collect(),
        )
    }

    #[cfg(test)]
    fn banned_ips_json(&self) -> String {
        json_array(
            self.banned_ips
                .iter()
                .map(|entry| {
                    format!(
                        "{{\"ip\":\"{}\",\"created\":\"{}\",\"source\":\"{}\",\"expires\":\"{}\",\"reason\":\"{}\"}}",
                        escape(&entry.user),
                        escape(&entry.created),
                        escape(&entry.source),
                        escape(entry.expires.as_deref().unwrap_or("forever")),
                        escape(entry.reason.as_deref().unwrap_or(""))
                    )
                })
                .collect(),
        )
    }

    #[cfg(test)]
    fn whitelist_json(&self) -> String {
        json_array(self.whitelist.iter().map(name_and_id_json).collect())
    }

    #[cfg(test)]
    fn ops_json(&self) -> String {
        json_array(
            self.ops
                .iter()
                .map(|entry| {
                    format!(
                        "{{\"uuid\":\"{}\",\"name\":\"{}\",\"level\":{},\"bypassesPlayerLimit\":{}}}",
                        escape(&entry.user.uuid),
                        escape(&entry.user.name),
                        entry.level,
                        entry.bypasses_player_limit
                    )
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
                    format!(
                        "{{\"uuid\":\"{}\",\"name\":\"{}\",\"expiresOn\":\"{}\"}}",
                        escape(&user.uuid),
                        escape(&user.name),
                        escape(&expires_on)
                    )
                })
                .collect(),
        )
    }
}

impl NameAndId {
    pub fn create_offline(name: &str) -> Self {
        Self {
            uuid: offline_player_uuid(name),
            name: name.to_string(),
        }
    }
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

#[cfg(test)]
fn name_and_id_json(user: &NameAndId) -> String {
    format!(
        "{{\"uuid\":\"{}\",\"name\":\"{}\"}}",
        escape(&user.uuid),
        escape(&user.name)
    )
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

fn escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            ch => vec![ch],
        })
        .collect()
}

fn load_name_and_id_entries(path: &Path) -> std::io::Result<Vec<NameAndId>> {
    let raw = read_optional(path)?;
    Ok(json_objects(&raw)
        .into_iter()
        .filter_map(|object| {
            Some(NameAndId {
                uuid: json_string_field(&object, "uuid")?,
                name: json_string_field(&object, "name")?,
            })
        })
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
                level: json_u8_field(&object, "level").unwrap_or(4),
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
    let mut escaped = false;
    for (index, ch) in chars {
        if escaped {
            out.push(match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some((out, &input[index + 1..]));
        } else {
            out.push(ch);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        BanEntry, BlockPos, IpLogPolicy, NameAndId, OpEntry, PlayerAccess, ProxyConnectionDecision,
        SpawnProtection,
    };
    use std::fs;

    fn player() -> NameAndId {
        NameAndId {
            uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            name: "Steve".to_string(),
        }
    }

    #[test]
    fn checks_bans_whitelist_and_op_level() {
        let mut access = PlayerAccess::default();
        let player = player();
        access.ban_player(BanEntry {
            user: player.clone(),
            created: "2026-05-16 00:00:00 +0000".to_string(),
            source: "Server".to_string(),
            expires: None,
            reason: Some("test".to_string()),
        });
        access.ban_ip(BanEntry {
            user: "127.0.0.1".to_string(),
            created: "2026-05-16 00:00:00 +0000".to_string(),
            source: "Server".to_string(),
            expires: None,
            reason: None,
        });
        access.whitelist(player.clone());
        access.op(OpEntry {
            user: player.clone(),
            level: 4,
            bypasses_player_limit: true,
        });

        assert!(access.is_player_banned(&player.uuid));
        assert!(access.is_ip_banned("127.0.0.1"));
        assert!(access.is_whitelisted(&player.uuid));
        assert_eq!(access.op_level(&player.uuid), Some(4));
    }

    #[test]
    fn writes_vanilla_access_control_files() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-access-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let mut access = PlayerAccess::default();
        let player = player();
        access.cache_user(player.clone());
        access.whitelist(player.clone());
        access.op(OpEntry {
            user: player,
            level: 3,
            bypasses_player_limit: false,
        });
        access.save_all(&dir).unwrap();

        assert!(dir.join("ops.json").is_file());
        assert!(dir.join("whitelist.json").is_file());
        assert!(dir.join("banned-players.json").is_file());
        assert!(dir.join("banned-ips.json").is_file());
        assert!(dir.join("usercache.json").is_file());
        assert!(fs::read_to_string(dir.join("ops.json"))
            .unwrap()
            .contains("\"level\":3"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_vanilla_access_control_files() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-access-load-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("whitelist.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Steve\"}]",
        )
        .unwrap();
        fs::write(
            dir.join("ops.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\",\"level\":3,\"bypassesPlayerLimit\":true}]",
        )
        .unwrap();
        fs::write(
            dir.join("banned-players.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"Griefer\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"test\"}]",
        )
        .unwrap();
        fs::write(
            dir.join("banned-ips.json"),
            "[{\"ip\":\"127.0.0.1\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"test\"}]",
        )
        .unwrap();
        fs::write(
            dir.join("usercache.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000004\",\"name\":\"Cached\",\"expiresOn\":\"2999-01-01 00:00:00 +0000\"}]",
        )
        .unwrap();

        let mut access = PlayerAccess::load_from_dir(&dir).unwrap();
        assert!(access.is_whitelisted("00000000-0000-0000-0000-000000000001"));
        assert_eq!(
            access.op_level("00000000-0000-0000-0000-000000000002"),
            Some(3)
        );
        assert!(access.is_player_banned("00000000-0000-0000-0000-000000000003"));
        assert!(access.is_ip_banned("127.0.0.1"));
        assert_eq!(
            access
                .lookup_cached_profile("cached", std::time::SystemTime::now())
                .unwrap()
                .name,
            "Cached"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reload_from_dir_reflects_hot_edited_operator_whitelist_and_ban_files() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-access-hot-edit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("whitelist.json"), "[]").unwrap();
        fs::write(
            dir.join("ops.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Steve\",\"level\":4,\"bypassesPlayerLimit\":false}]",
        )
        .unwrap();
        fs::write(dir.join("banned-players.json"), "[]").unwrap();
        fs::write(dir.join("banned-ips.json"), "[]").unwrap();
        fs::write(dir.join("usercache.json"), "[]").unwrap();

        let first = PlayerAccess::load_from_dir(&dir).unwrap();
        assert_eq!(
            first.op_level("00000000-0000-0000-0000-000000000001"),
            Some(4)
        );
        assert!(!first.is_whitelisted("00000000-0000-0000-0000-000000000002"));
        assert!(!first.is_player_banned("00000000-0000-0000-0000-000000000003"));

        fs::write(
            dir.join("ops.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\",\"level\":2,\"bypassesPlayerLimit\":true}]",
        )
        .unwrap();
        fs::write(
            dir.join("whitelist.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\"}]",
        )
        .unwrap();
        fs::write(
            dir.join("banned-players.json"),
            "[{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"Griefer\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"reload\"}]",
        )
        .unwrap();

        let reloaded = PlayerAccess::load_from_dir(&dir).unwrap();
        assert!(!reloaded.is_op("00000000-0000-0000-0000-000000000001"));
        assert_eq!(
            reloaded.op_level("00000000-0000-0000-0000-000000000002"),
            Some(2)
        );
        assert!(reloaded.is_whitelisted("00000000-0000-0000-0000-000000000002"));
        assert!(reloaded.is_player_banned("00000000-0000-0000-0000-000000000003"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn usercache_loader_ignores_malformed_missing_and_expired_entries() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-usercache-corrupt-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("usercache.json"),
            concat!(
                "[",
                "{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Valid\",\"expiresOn\":\"2999-01-01 00:00:00 +0000\"},",
                "{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Expired\",\"expiresOn\":\"2000-01-01 00:00:00 +0000\"},",
                "{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"MissingDate\"},",
                "{\"uuid\":\"00000000-0000-0000-0000-000000000004\",\"name\":\"MalformedDate\",\"expiresOn\":\"not a date\"}",
                "]"
            ),
        )
        .unwrap();

        let mut access = PlayerAccess::load_from_dir(&dir).unwrap();
        let now = std::time::SystemTime::now();
        assert_eq!(
            access.lookup_cached_profile("valid", now).unwrap().name,
            "Valid"
        );
        assert!(access.lookup_cached_profile("expired", now).is_none());
        assert!(access.lookup_cached_profile("missingdate", now).is_none());
        assert!(access.lookup_cached_profile("malformeddate", now).is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn profile_cache_expires_entries() {
        let mut access = PlayerAccess::default();
        let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
        let player = player();
        access.profile_cache.insert(player.clone(), now);

        assert_eq!(
            access.lookup_cached_profile("steve", now + std::time::Duration::from_secs(60)),
            Some(player)
        );
        assert_eq!(
            access.lookup_cached_profile(
                "steve",
                now + std::time::Duration::from_secs(60 * 60 * 24 * 31)
            ),
            None
        );
    }

    #[test]
    fn derives_offline_mode_uuid() {
        let player = NameAndId::create_offline("Steve");
        assert_eq!(player.name, "Steve");
        assert_eq!(player.uuid, "5627dd98-e6be-3c21-b8a8-e92344183641");
    }

    #[test]
    fn prevent_proxy_connections_rejects_host_socket_ip_mismatch() {
        let access = PlayerAccess::default();
        assert_eq!(
            access.check_proxy_connection(true, "203.0.113.10", "198.51.100.20"),
            ProxyConnectionDecision::RejectPreventProxyConnections
        );
        assert_eq!(
            access.check_proxy_connection(true, "203.0.113.10", "203.0.113.10"),
            ProxyConnectionDecision::Allow
        );
        assert_eq!(
            access.check_proxy_connection(false, "203.0.113.10", "198.51.100.20"),
            ProxyConnectionDecision::Allow
        );
    }

    #[test]
    fn ip_log_policy_redacts_remote_addresses_when_disabled() {
        let access = PlayerAccess::default();
        assert_eq!(access.ip_log_policy(true), IpLogPolicy::Include);
        assert_eq!(access.ip_log_policy(false), IpLogPolicy::Redact);
        assert_eq!(
            access.ip_log_policy(true).format_remote("203.0.113.10"),
            "203.0.113.10"
        );
        assert_eq!(
            access.ip_log_policy(false).format_remote("203.0.113.10"),
            "IP hidden"
        );
    }

    #[test]
    fn spawn_protection_matches_dedicated_server_rules() {
        let mut access = PlayerAccess::default();
        let player = player();
        let op = NameAndId {
            uuid: "00000000-0000-0000-0000-000000000002".to_string(),
            name: "Alex".to_string(),
        };
        let protection = SpawnProtection {
            radius: 16,
            spawn_dimension: "minecraft:overworld".to_string(),
            spawn_pos: BlockPos { x: 0, y: 64, z: 0 },
        };

        assert!(!access.is_under_spawn_protection(
            &protection,
            "minecraft:overworld",
            BlockPos { x: 0, y: 64, z: 0 },
            &player.uuid
        ));

        access.op(OpEntry {
            user: op.clone(),
            level: 4,
            bypasses_player_limit: true,
        });

        assert!(access.is_under_spawn_protection(
            &protection,
            "minecraft:overworld",
            BlockPos {
                x: 16,
                y: -64,
                z: 0
            },
            &player.uuid
        ));
        assert!(!access.is_under_spawn_protection(
            &protection,
            "minecraft:overworld",
            BlockPos { x: 17, y: 64, z: 0 },
            &player.uuid
        ));
        assert!(!access.is_under_spawn_protection(
            &protection,
            "minecraft:the_nether",
            BlockPos { x: 0, y: 64, z: 0 },
            &player.uuid
        ));
        assert!(!access.is_under_spawn_protection(
            &protection,
            "minecraft:overworld",
            BlockPos { x: 0, y: 64, z: 0 },
            &op.uuid
        ));
        assert!(!access.is_under_spawn_protection(
            &SpawnProtection {
                radius: 0,
                ..protection
            },
            "minecraft:overworld",
            BlockPos { x: 0, y: 64, z: 0 },
            &player.uuid
        ));
    }
}
