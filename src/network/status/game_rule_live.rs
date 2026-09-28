//! Live game-rule networking: `MinecraftServer.onGameRuleChanged` fan-out, the serverbound
//! `set_game_rule` handler, the `REQUEST_GAMERULE_VALUES` client command, and the
//! join-time flags derived from the rules.
//!
//! Java broadcasts to every player from the server thread; VibeCraft sessions each replay the
//! shared [`LiveGameRules`] change log on their own tick (see `game_rules/live.rs`).

use super::*;
use crate::command::{LevelBasedPermissionSet, Permission, PermissionLevel};
use crate::game_rules::{
    deserialize_game_rule_value, game_rule_definition, GameRuleChange, GameRuleValue, GameRules,
    LiveGameRules, SharedGameRules,
};
use crate::log::log_warn;
use crate::network::play::{ClientboundGameRuleValuesPacket, CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID};

/// The single live player's network entity id, as sent in `ClientboundLoginPacket.player_id`.
const SESSION_PLAYER_ENTITY_ID: i32 = 1;
/// `ClientboundGameEventPacket.IMMEDIATE_RESPAWN`
const GAME_EVENT_IMMEDIATE_RESPAWN: u8 = 11;
/// `ClientboundGameEventPacket.LIMITED_CRAFTING`
const GAME_EVENT_LIMITED_CRAFTING: u8 = 12;
/// `ClientboundEntityEventPacket` ids sent for `GameRules.REDUCED_DEBUG_INFO` (true / false).
const ENTITY_EVENT_REDUCED_DEBUG_INFO_ON: i8 = 22;
const ENTITY_EVENT_REDUCED_DEBUG_INFO_OFF: i8 = 23;
/// `ServerboundClientCommandPacket.Action.REQUEST_GAMERULE_VALUES` ordinal.
pub(super) const CLIENT_COMMAND_REQUEST_GAMERULE_VALUES: i32 = 2;

/// Whether the world enables `minecart_improvements` (`GameRules(FeatureFlagSet, ...)` only
/// registers `max_minecart_speed` then). Read from `level.dat`'s `enabled_features`.
fn world_enables_minecart_improvements(layout: &WorldLayout) -> bool {
    layout
        .load_level_dat_with_backup()
        .ok()
        .and_then(|tag| PrimaryLevelData::from_level_dat(&tag))
        .is_some_and(|level| {
            level
                .data_configuration
                .enabled_features
                .contains(crate::registry::feature_flags::MINECART_IMPROVEMENTS)
        })
}

/// Loads the world's `game_rules` saved data (`MinecraftServer` constructor:
/// `new GameRules(enabledFeatures, savedDataStorage.computeIfAbsent(GameRuleMap.TYPE))`) and
/// applies the legacy `announce-player-achievements` override (`DedicatedServer.initServer`).
pub(super) fn load_live_game_rules(
    world_root: &Path,
    properties: &ServerProperties,
) -> SharedGameRules {
    let minecart_improvements =
        world_enables_minecart_improvements(&WorldLayout::new(world_root));
    let mut rules = GameRules::load_from_world(world_root, minecart_improvements)
        .unwrap_or_else(|err| {
            log_warn(&format!("Failed to load game rules, using defaults: {err}"));
            GameRules::new(minecart_improvements)
        });
    if let Err(err) = properties.migrate_legacy_announce_player_achievements(&mut rules) {
        log_warn(&format!("Failed to apply announce-player-achievements: {err:?}"));
    }
    let shared = LiveGameRules::shared(rules);
    // The override above is a `GameRules.set`, so the store starts dirty when it applied.
    if properties.announce_player_achievements.is_some() {
        save_live_game_rules(world_root, &shared);
    }
    shared
}

/// Writes `data/minecraft/game_rules.dat` when the rules changed since the last save.
pub(super) fn save_live_game_rules(world_root: &Path, rules: &SharedGameRules) {
    let mut rules = lock_status_mutex(rules);
    if rules.take_dirty() || !world_root.join("data/minecraft/game_rules.dat").is_file() {
        if let Err(err) = rules.rules().save_to_world(world_root) {
            log_warn(&format!("Failed to save game rules: {err}"));
        }
    }
}

/// Per-connection cursor into the shared game-rule change log.
#[derive(Debug, Clone, Copy)]
pub(super) struct GameRuleSessionSync {
    last_seq: u64,
}

impl GameRuleSessionSync {
    /// A session only sees changes made after it joined (its login packet already carries
    /// the then-current values).
    pub(super) fn joined(rules: &SharedGameRules) -> Self {
        Self {
            last_seq: lock_status_mutex(rules).current_seq(),
        }
    }
}

/// Flags `PlayerList.placeNewPlayer` derives from the rules for `ClientboundLoginPacket`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct JoinGameRuleFlags {
    pub reduced_debug_info: bool,
    /// `!GameRules.IMMEDIATE_RESPAWN`
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
}

pub(super) fn join_game_rule_flags(rules: &SharedGameRules) -> JoinGameRuleFlags {
    let rules = lock_status_mutex(rules);
    JoinGameRuleFlags {
        reduced_debug_info: rules.bool("reduced_debug_info"),
        show_death_screen: !rules.bool("immediate_respawn"),
        do_limited_crafting: rules.bool("limited_crafting"),
    }
}

/// Builds a translatable component NBT (`Component.translatable(key, args...)` with string args).
pub(super) fn translatable_component_tag(key: &str, args: &[String]) -> Tag {
    let mut entries = vec![("translate".to_string(), Tag::String(key.to_string()))];
    if !args.is_empty() {
        entries.push((
            "with".to_string(),
            Tag::List(args.iter().map(|arg| Tag::String(arg.clone())).collect()),
        ));
    }
    Tag::Compound(entries)
}

/// Sends a translatable system chat message (`Player.sendSystemMessage`).
pub(super) fn write_system_chat_translatable<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    key: &str,
    args: &[String],
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SYSTEM_CHAT_PACKET_ID,
        |payload| {
            ClientboundSystemChatPacket {
                content: translatable_component_tag(key, args),
                overlay: false,
            }
            .write(payload)
        },
    )
}

/// Who the session belongs to, for permission-gated game-rule packets.
pub(super) struct GameRuleSessionPlayer<'a> {
    pub profile: &'a NameAndId,
    pub properties: &'a ServerProperties,
    pub player_access: &'a Arc<Mutex<PlayerAccess>>,
}

impl GameRuleSessionPlayer<'_> {
    fn permissions(&self) -> LevelBasedPermissionSet {
        super::play_session_world_packets::player_permission_set(
            self.profile,
            self.properties,
            self.player_access,
        )
    }

    /// `Permissions.COMMANDS_GAMEMASTER`
    fn is_gamemaster(&self) -> bool {
        self.permissions()
            .has_permission(Permission::CommandLevel(PermissionLevel::Gamemasters))
    }

    /// `PlayerList.isOp`
    fn is_op(&self) -> bool {
        lock_status_mutex(self.player_access).is_op(&self.profile.uuid)
    }
}

/// Replays every game-rule change this session has not yet seen, mirroring
/// `MinecraftServer.onGameRuleChanged` plus `broadcastGameRuleChangeToOperators`.
pub(super) fn replay_game_rule_changes<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    sync: &mut GameRuleSessionSync,
    rules: &SharedGameRules,
    clock: &Arc<Mutex<ServerClockManager>>,
    player: &GameRuleSessionPlayer<'_>,
) -> io::Result<()> {
    let changes = lock_status_mutex(rules).changes_since(sync.last_seq);
    for change in changes {
        sync.last_seq = change.seq;
        if change.applied {
            write_game_rule_changed(writer, compression, &change, clock)?;
        }
        if change.announce_to_operators && player.is_op() {
            write_system_chat_translatable(
                writer,
                compression,
                "commands.gamerule.set",
                &[change.rule.to_string(), change.value.sync_value()],
            )?;
        }
    }
    Ok(())
}

/// `MinecraftServer.onGameRuleChanged`'s per-rule packets for one player.
fn write_game_rule_changed<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    change: &GameRuleChange,
    clock: &Arc<Mutex<ServerClockManager>>,
) -> io::Result<()> {
    let enabled = matches!(change.value, GameRuleValue::Bool(true));
    match change.rule {
        "reduced_debug_info" => write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
            |payload| {
                ClientboundEntityEventPacket {
                    entity_id: SESSION_PLAYER_ENTITY_ID,
                    event_id: if enabled {
                        ENTITY_EVENT_REDUCED_DEBUG_INFO_ON
                    } else {
                        ENTITY_EVENT_REDUCED_DEBUG_INFO_OFF
                    },
                }
                .write(payload)
            },
        ),
        "limited_crafting" | "immediate_respawn" => {
            let event = if change.rule == "limited_crafting" {
                GAME_EVENT_LIMITED_CRAFTING
            } else {
                GAME_EVENT_IMMEDIATE_RESPAWN
            };
            write_game_event_to_writer(writer, compression, event, if enabled { 1.0 } else { 0.0 })
        }
        // ClientClockManager.createFullSyncPacket with the new rate policy.
        "advance_time" => {
            let (game_time, clocks) = lock_status_mutex(clock).full_sync_data(enabled);
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_SET_TIME_PACKET_ID,
                |payload| {
                    ClientboundSetTimePacket {
                        game_time,
                        clock_updates: clocks.into_iter().collect(),
                    }
                    .write(payload)
                },
            )
        }
        // TODO(gamerule-locator_bar): needs the live ServerWaypointManager
        // (updatePlayer / breakAllConnections); no live waypoint tracker exists yet.
        // TODO(gamerule-spawn_monsters): needs the live MinecraftServer.updateMobSpawningFlags
        // consumer; VibeCraft has no live natural-spawn loop yet.
        _ => Ok(()),
    }
}

/// `ServerGamePacketListenerImpl.handleSetGameRule`.
pub(super) fn handle_set_game_rule_packet(
    input: &mut Cursor<Vec<u8>>,
    rules: &SharedGameRules,
    player: &GameRuleSessionPlayer<'_>,
) -> io::Result<()> {
    let packet = ServerboundSetGameRulePacket::read(input)?;
    if !player.is_gamemaster() {
        log_warn(&format!(
            "Player {} tried to set game rule values without required permissions",
            player.profile.name
        ));
        return Ok(());
    }
    for entry in packet.entries {
        let definition = (entry.game_rule_key.namespace() == "minecraft")
            .then(|| game_rule_definition(entry.game_rule_key.path()))
            .flatten();
        let Some(definition) = definition else {
            log_warn(&format!(
                "Received request to set unknown game rule: {}",
                entry.game_rule_key
            ));
            continue;
        };
        // `GameRule.deserialize(...).result().ifPresent(...)`: an unparsable value is ignored.
        if let Ok(value) = deserialize_game_rule_value(&entry.value, definition) {
            lock_status_mutex(rules).set_from_client(definition, value);
        }
    }
    Ok(())
}

/// `ServerGamePacketListenerImpl.sendGameRuleValues` (client command `REQUEST_GAMERULE_VALUES`).
pub(super) fn handle_request_gamerule_values<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    rules: &SharedGameRules,
    player: &GameRuleSessionPlayer<'_>,
) -> io::Result<()> {
    if !player.is_gamemaster() {
        log_warn(&format!(
            "Player {} tried to request game rule values without required permissions",
            player.profile.name
        ));
        return Ok(());
    }
    let values = lock_status_mutex(rules)
        .rules()
        .available_rules()
        .filter_map(|(definition, value)| {
            Identifier::new("minecraft", definition.name)
                .ok()
                .map(|key| (key, value.sync_value()))
        })
        .collect();
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID,
        |payload| ClientboundGameRuleValuesPacket { values }.write(payload),
    )
}

/// Routes the game-rule related serverbound packets. Returns `true` when the packet was fully
/// handled here (`set_game_rule`, or `client_command` with `REQUEST_GAMERULE_VALUES`).
pub(super) fn try_handle_game_rule_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    packet_id: i32,
    input: &mut Cursor<Vec<u8>>,
    rules: &SharedGameRules,
    player: &GameRuleSessionPlayer<'_>,
) -> io::Result<bool> {
    if packet_id == SERVERBOUND_SET_GAME_RULE_PACKET_ID {
        handle_set_game_rule_packet(input, rules, player)?;
        return Ok(true);
    }
    if packet_id == SERVERBOUND_CLIENT_COMMAND_PACKET_ID {
        // Peek the action so the ordinary respawn/stats handling still sees the packet.
        let action = read_var_i32(&mut input.clone())?;
        if action == CLIENT_COMMAND_REQUEST_GAMERULE_VALUES {
            handle_request_gamerule_values(writer, compression, rules, player)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Seeds a command execution's [`ServerCommandState`] with the live rule values, so
/// `/gamerule` reads (`GameRules.get`) see the server-wide store.
pub(super) fn seed_command_game_rules(state: &mut ServerCommandState, rules: &SharedGameRules) {
    state.game_rules = lock_status_mutex(rules)
        .rules()
        .available_rules()
        .map(|(definition, value)| crate::command::GameRuleState {
            name: definition.name.to_string(),
            value,
        })
        .collect();
}

/// Applies the `GameRules.set` calls a command made back to the live store, which queues the
/// `onGameRuleChanged` fan-out for every session.
pub(super) fn apply_command_game_rule_changes(
    state: &ServerCommandState,
    rules: &SharedGameRules,
) {
    let mut rules = lock_status_mutex(rules);
    for sync in &state.game_rule_syncs {
        let name = sync.rule.strip_prefix("minecraft:").unwrap_or(&sync.rule);
        if let Err(err) = rules.set(name, &sync.value, false) {
            log_warn(&format!("Tried to set invalid game rule '{name}': {err:?}"));
        }
    }
}
