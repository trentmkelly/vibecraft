//! Live wiring of the operator / whitelist / ban lists.
//!
//! Java keeps these lists on the single `PlayerList` (`ServerOpList`, `UserWhiteList`,
//! `UserBanList`, `IpBanList`), and the commands mutate them directly:
//! `OpCommand`/`DeOpCommands` -> `PlayerList.op/deop`, `BanPlayerCommands`/`BanIpCommands`/
//! `PardonCommand`/`PardonIpCommand` -> the ban lists, `WhitelistCommand` -> the whitelist plus
//! `MinecraftServer.setUsingWhitelist` / `kickUnlistedPlayers`. VibeCraft's command engine works
//! on a per-command [`ServerCommandState`] snapshot, so every runner (in-game chat commands,
//! console, RCON) follows the same three steps:
//!
//! 1. [`seed_access_state`] copies the shared [`PlayerAccess`] into the command state,
//! 2. the command runs against the state,
//! 3. [`apply_access_changes`] diffs the state against the seed, writes the differences back to
//!    the shared [`PlayerAccess`] (which persists `ops.json`, `whitelist.json`,
//!    `banned-players.json` and `banned-ips.json` on every change like
//!    `StoredUserList.save`) and performs the live side effects: `whitelist reload`
//!    (`DedicatedPlayerList.reloadWhiteList`) and `MinecraftServer.kickUnlistedPlayers`.
//!
//! Login enforcement ([`can_player_login`]) is `PlayerList.canPlayerLogin`, and the permission
//! level notification of `PlayerList.sendPlayerPermissionLevel` is
//! [`write_permission_level_update`].

use super::*;
use crate::chat_component::{Component, ComponentArgument};
use crate::command::PlayerIpAddress;
use crate::player_access::{BanEntry, OpEntry};

use super::game_rule_live::SESSION_PLAYER_ENTITY_ID;
use super::player_messaging_live::{disconnect_payload, disconnect_reason_json};

/// `PlayerList.sendPlayerPermissionLevel`: entity event `24 + permission level`.
const PERMISSION_LEVEL_EVENT_BASE: i8 = 24;
/// `multiplayer.disconnect.not_whitelisted`.
const NOT_WHITELISTED_KEY: &str = "multiplayer.disconnect.not_whitelisted";
/// `multiplayer.disconnect.server_full`.
const SERVER_FULL_KEY: &str = "multiplayer.disconnect.server_full";

/// Loads the server-root lists (`DedicatedPlayerList` constructor) and seeds the live
/// `white-list` flag (`DedicatedServer.isUsingWhitelist`) from `server.properties`.
pub(super) fn load_live_player_access(properties: &ServerProperties) -> PlayerAccess {
    let mut access = PlayerAccess::load_from_dir(Path::new(".")).unwrap_or_else(|err| {
        eprintln!("status access file load error: {err}");
        PlayerAccess::default()
    });
    access.set_using_whitelist(properties.white_list);
    access
}

/// A view of the connected sessions, obtainable from the registry (console/RCON) or from one
/// session's guard (in-game commands); both share the same session map and packet bus.
#[derive(Clone, Copy)]
pub(super) struct LiveSessions<'a> {
    sessions: &'a Mutex<HashMap<String, ActiveLoginSession>>,
    world_bus: &'a WorldPacketBus,
}

impl ActiveLoginRegistry {
    pub(super) fn live_sessions(&self) -> LiveSessions<'_> {
        LiveSessions { sessions: &self.sessions, world_bus: &self.world_bus }
    }
}

impl ActiveLoginGuard {
    pub(super) fn live_sessions(&self) -> LiveSessions<'_> {
        LiveSessions { sessions: &self.sessions, world_bus: &self.world_bus }
    }
}

impl LiveSessions<'_> {
    /// `ServerPlayer.getIpAddress` of every in-play player.
    fn addresses(&self) -> Vec<PlayerIpAddress> {
        lock_status_mutex(self.sessions)
            .iter()
            .filter(|(_, session)| session.in_play)
            .filter_map(|(uuid, session)| {
                let ip = session.stream.peer_addr().ok()?.ip().to_string();
                let player = NameAndId { uuid: uuid.clone(), name: session.name.clone() };
                Some(PlayerIpAddress { player, ip })
            })
            .collect()
    }

    /// The in-play players (`PlayerList.getPlayers`).
    pub(super) fn in_play_profiles(&self) -> Vec<NameAndId> {
        lock_status_mutex(self.sessions)
            .iter()
            .filter(|(_, session)| session.in_play)
            .map(|(uuid, session)| NameAndId { uuid: uuid.clone(), name: session.name.clone() })
            .collect()
    }

    /// `ServerGamePacketListenerImpl.disconnect(reason)` for the in-play player `uuid`.
    fn disconnect_uuid(&self, uuid: &str, reason_json: String) {
        let token = lock_status_mutex(self.sessions)
            .get(uuid)
            .filter(|session| session.in_play)
            .map(|session| session.token);
        let Some(token) = token else { return };
        match disconnect_payload(reason_json) {
            Ok(payload) => {
                self.world_bus.disconnect(token, &payload);
            }
            Err(err) => eprintln!("failed to encode disconnect packet: {err}"),
        }
    }

    /// Disconnects the in-play player named `name` (case-insensitive,
    /// `PlayerList.getPlayerByName`); true when a player was found.
    fn disconnect_named(&self, name: &str, reason_json: String) -> bool {
        let uuid = lock_status_mutex(self.sessions)
            .iter()
            .find(|(_, session)| session.in_play && session.name.eq_ignore_ascii_case(name))
            .map(|(uuid, _)| uuid.clone());
        match uuid {
            Some(uuid) => {
                self.disconnect_uuid(&uuid, reason_json);
                true
            }
            None => false,
        }
    }

    /// `MinecraftServer.kickUnlistedPlayers`: with `enforce-whitelist` and the whitelist on,
    /// every online player missing from `whitelist.json` (operators are not exempt) is
    /// disconnected with `multiplayer.disconnect.not_whitelisted`.
    fn kick_unlisted_players(&self, access: &PlayerAccess, properties: &ServerProperties) {
        if !(properties.enforce_whitelist && access.using_whitelist()) {
            return;
        }
        for player in self.in_play_profiles() {
            if !access.is_whitelisted(&player.uuid) {
                self.disconnect_uuid(&player.uuid, translatable_json(NOT_WHITELISTED_KEY));
            }
        }
    }
}

fn translatable_json(key: &str) -> String {
    Component::translatable(key, Vec::new()).to_json()
}

/// What the shared lists held when the command state was seeded.
pub(super) struct AccessSnapshot {
    banned_players: Vec<String>,
    banned_ips: Vec<String>,
    operators: Vec<String>,
    whitelisted: Vec<String>,
    using_whitelist: bool,
}

/// Copies the shared lists into `state` (see the module docs) and returns the seed.
pub(super) fn seed_access_state(
    state: &mut ServerCommandState,
    player_access: &Arc<Mutex<PlayerAccess>>,
    sessions: LiveSessions<'_>,
) -> AccessSnapshot {
    let access = lock_status_mutex(player_access);
    state.banned_players = access.banned_players().into_iter().cloned().collect();
    state.banned_ips = access.banned_ips().into_iter().cloned().collect();
    state.operator_players = access.operators().iter().map(|entry| entry.user.clone()).collect();
    state.whitelisted_players = access.whitelisted().to_vec();
    state.whitelist_enabled = access.using_whitelist();
    state.known_profiles = access.cached_users().to_vec();
    state.online_player_addresses = sessions.addresses();
    AccessSnapshot {
        banned_players: state.banned_players.iter().map(|entry| entry.user.uuid.clone()).collect(),
        banned_ips: state.banned_ips.iter().map(|entry| entry.user.clone()).collect(),
        operators: state.operator_players.iter().map(|user| user.uuid.clone()).collect(),
        whitelisted: state.whitelisted_players.iter().map(|user| user.uuid.clone()).collect(),
        using_whitelist: state.whitelist_enabled,
    }
}

/// Inputs shared by [`apply_access_changes`] callers.
pub(super) struct AccessContext<'a> {
    pub access: &'a Arc<Mutex<PlayerAccess>>,
    pub properties: &'a ServerProperties,
    pub sessions: LiveSessions<'a>,
    /// `server.properties`, rewritten when `/whitelist on|off` flips `white-list`
    /// (`DedicatedServer.setUsingWhitelist` -> `settings.update`). `None` keeps it in memory.
    pub properties_file: Option<&'a Path>,
}

/// Writes the differences between the finished command `state` and its `seed` back to the
/// shared lists, then performs the whitelist reload / unlisted-player kicks the command asked
/// for.
pub(super) fn apply_access_changes(
    context: &AccessContext<'_>,
    seed: &AccessSnapshot,
    state: &ServerCommandState,
) {
    {
        let mut access = lock_status_mutex(context.access);
        apply_ban_changes(&mut access, seed, state);
        apply_operator_changes(&mut access, context.properties, seed, state);
        apply_whitelist_changes(&mut access, seed, state);
        if state.whitelist_enabled != seed.using_whitelist {
            access.set_using_whitelist(state.whitelist_enabled);
            persist_white_list_property(context.properties_file, state.whitelist_enabled);
        }
        if state.whitelist_reload_requests > 0 {
            if let Err(err) = access.reload_whitelist() {
                eprintln!("Failed to load white-list: {err}");
            }
        }
        if state.kick_unlisted_requests > 0 {
            context.sessions.kick_unlisted_players(&access, context.properties);
        }
    }
}

/// `UserBanList` / `IpBanList` `add` and `remove` for the entries the command added / removed.
fn apply_ban_changes(access: &mut PlayerAccess, seed: &AccessSnapshot, state: &ServerCommandState) {
    for entry in &state.banned_players {
        if !seed.banned_players.contains(&entry.user.uuid) {
            access.ban_player(entry.clone());
        }
    }
    for uuid in &seed.banned_players {
        if !state.banned_players.iter().any(|entry| &entry.user.uuid == uuid) {
            access.pardon_player(uuid);
        }
    }
    for entry in &state.banned_ips {
        if !seed.banned_ips.contains(&entry.user) {
            access.ban_ip(entry.clone());
        }
    }
    for ip in &seed.banned_ips {
        if !state.banned_ips.iter().any(|entry| &entry.user == ip) {
            access.pardon_ip(ip);
        }
    }
}

/// `PlayerList.op` (new entries take `op-permission-level` and keep an existing
/// `bypassesPlayerLimit`) and `PlayerList.deop`. The target sessions notice the permission
/// change through their per-tick permission poll (see [`write_permission_level_update`]).
fn apply_operator_changes(
    access: &mut PlayerAccess,
    properties: &ServerProperties,
    seed: &AccessSnapshot,
    state: &ServerCommandState,
) {
    for user in &state.operator_players {
        if !seed.operators.contains(&user.uuid) {
            let entry = OpEntry {
                user: user.clone(),
                level: properties.op_permission_level.min(4) as u8,
                bypasses_player_limit: access.can_bypass_player_limit(&user.uuid),
            };
            access.op(entry);
        }
    }
    for uuid in &seed.operators {
        if !state.operator_players.iter().any(|user| &user.uuid == uuid) {
            access.deop(uuid);
        }
    }
}

/// `UserWhiteList.add` / `remove` for the users the command added / removed.
fn apply_whitelist_changes(
    access: &mut PlayerAccess,
    seed: &AccessSnapshot,
    state: &ServerCommandState,
) {
    for user in &state.whitelisted_players {
        if !seed.whitelisted.contains(&user.uuid) {
            access.whitelist(user.clone());
        }
    }
    for uuid in &seed.whitelisted {
        if !state.whitelisted_players.iter().any(|user| &user.uuid == uuid) {
            access.unwhitelist(uuid);
        }
    }
}

/// `DedicatedServer.setUsingWhitelist`: stores `white-list` back into `server.properties`.
fn persist_white_list_property(path: Option<&Path>, enabled: bool) {
    let Some(path) = path else { return };
    let result = ServerProperties::load_or_default(path).and_then(|mut properties| {
        properties.set("white-list", enabled.to_string());
        properties.save(path)
    });
    if let Err(err) = result {
        eprintln!("Failed to update white-list in {}: {err}", path.display());
    }
}

/// Disconnects requested by a console/RCON command (`/kick`, the ban family): the in-game
/// runner delivers these through `CommandEffects`, which needs a player session.
pub(super) fn apply_console_disconnects(sessions: LiveSessions<'_>, state: &ServerCommandState) {
    for request in &state.disconnected_players {
        sessions.disconnect_named(&request.player.name, disconnect_reason_json(&request.reason));
    }
}

/// `PlayerList.canPlayerLogin` (its proxy-independent part): the disconnect component's JSON,
/// or `None` when the login may proceed. Order: user ban, whitelist, IP ban, server full.
pub(super) fn can_player_login(
    access: &PlayerAccess,
    properties: &ServerProperties,
    profile: &NameAndId,
    remote_ip: &str,
    online_players: usize,
) -> Option<String> {
    if let Some(ban) = access.player_ban(&profile.uuid) {
        return Some(ban_component(
            "multiplayer.disconnect.banned.reason",
            "multiplayer.disconnect.banned.expiration",
            ban,
        ));
    }
    if !is_white_listed(access, &profile.uuid) {
        return Some(translatable_json(NOT_WHITELISTED_KEY));
    }
    if let Some(ban) = access.ip_ban(remote_ip) {
        return Some(ban_component(
            "multiplayer.disconnect.banned_ip.reason",
            "multiplayer.disconnect.banned_ip.expiration",
            ban,
        ));
    }
    let full = online_players >= properties.max_players as usize;
    (full && !access.can_bypass_player_limit(&profile.uuid))
        .then(|| translatable_json(SERVER_FULL_KEY))
}

/// `DedicatedPlayerList.isWhiteListed`: everyone passes while the whitelist is off, otherwise
/// operators and listed users do.
fn is_white_listed(access: &PlayerAccess, uuid: &str) -> bool {
    !access.using_whitelist() || access.is_op(uuid) || access.is_whitelisted(uuid)
}

/// The ban disconnect message: `<reason key>(BanListEntry.getReasonMessage)` followed by the
/// `<expiration key>(BAN_DATE_FORMAT)` sibling when the ban expires.
fn ban_component<T>(reason_key: &str, expiration_key: &str, ban: &BanEntry<T>) -> String {
    let reason = match &ban.reason {
        Some(reason) => Component::literal(reason.clone()),
        None => Component::translatable("multiplayer.disconnect.banned.reason.default", Vec::new()),
    };
    let mut message =
        Component::translatable(reason_key, vec![ComponentArgument::Component(Box::new(reason))]);
    if let Some(expires) = ban.expires.as_deref().and_then(format_ban_expiration) {
        message = message.append(Component::translatable(
            expiration_key,
            vec![ComponentArgument::String(expires)],
        ));
    }
    message.to_json()
}

/// `PlayerList.BAN_DATE_FORMAT` (`yyyy-MM-dd 'at' HH:mm:ss z`) of a stored expiry. Java's `z`
/// prints the JVM zone abbreviation; without a zone database, UTC prints as `UTC` and other
/// offsets as `GMT+hh:mm` (Java's own fallback for zones lacking an abbreviation).
// TODO(ban-expiry-zone-name): abbreviations such as `CEST` need a tz database.
fn format_ban_expiration(stored: &str) -> Option<String> {
    let expires = chrono::DateTime::parse_from_str(stored, crate::player_access::BAN_DATE_FORMAT)
        .ok()?
        .with_timezone(&chrono::Local);
    let offset_seconds = expires.offset().local_minus_utc();
    let zone = if offset_seconds == 0 {
        "UTC".to_string()
    } else {
        let sign = if offset_seconds < 0 { '-' } else { '+' };
        let minutes = offset_seconds.abs() / 60;
        format!("GMT{sign}{:02}:{:02}", minutes / 60, minutes % 60)
    };
    Some(format!("{} {zone}", expires.format("%Y-%m-%d at %H:%M:%S")))
}

/// `PlayerList.sendPlayerPermissionLevel`: the `ENTITY_EVENT` carrying the new permission level
/// (`24..=28`) followed by the resent command tree (`Commands.sendCommands`).
pub(super) fn write_permission_level_update(
    stream: &mut ClientStream,
    compression: CompressionState,
    sync: &mut CommandTreeSync,
) -> io::Result<()> {
    let event_id = PERMISSION_LEVEL_EVENT_BASE + sync.current_level().min(4) as i8;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
        |payload| {
            ClientboundEntityEventPacket { entity_id: SESSION_PLAYER_ENTITY_ID, event_id }
                .write(payload)
        },
    )?;
    write_join_commands_packet(stream, compression, sync)
}

#[cfg(test)]
mod tests;
