//! Live game-rule packet handling (`MinecraftServer.onGameRuleChanged`,
//! `handleSetGameRule`, `REQUEST_GAMERULE_VALUES`, login flags).

use super::super::*;
use crate::game_rules::{GameRuleValue, LiveGameRules};
use crate::network::play::CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID;
use crate::network::status::game_rule_live;
use crate::player_access::OpEntry;

fn shared_rules() -> SharedGameRules {
    LiveGameRules::shared(crate::game_rules::GameRules::new(false))
}

fn player_access(level: Option<u8>, profile: &NameAndId) -> Arc<Mutex<PlayerAccess>> {
    let mut access = PlayerAccess::default();
    if let Some(level) = level {
        access.op(OpEntry {
            user: profile.clone(),
            level,
            bypasses_player_limit: false,
        });
    }
    Arc::new(Mutex::new(access))
}

fn packet_ids(output: &[u8]) -> Vec<i32> {
    let mut ids = Vec::new();
    let mut input = Cursor::new(output);
    while input.position() < output.len() as u64 {
        let len = read_var_i32(&mut input).unwrap() as usize;
        let mut frame = vec![0; len];
        input.read_exact(&mut frame).unwrap();
        ids.push(read_var_i32(&mut Cursor::new(frame)).unwrap());
    }
    ids
}

fn set_game_rule_packet(entries: &[(&str, &str)]) -> Cursor<Vec<u8>> {
    let mut bytes = Vec::new();
    write_var_i32(&mut bytes, entries.len() as i32).unwrap();
    for (key, value) in entries {
        crate::network::codec::write_identifier(&mut bytes, &Identifier::parse(key).unwrap())
            .unwrap();
        crate::network::codec::write_string(&mut bytes, value, 32767).unwrap();
    }
    Cursor::new(bytes)
}

struct Fixture {
    profile: NameAndId,
    properties: ServerProperties,
    access: Arc<Mutex<PlayerAccess>>,
}

impl Fixture {
    fn new(op_level: Option<u8>) -> Self {
        let profile = NameAndId::create_offline("Steve");
        let access = player_access(op_level, &profile);
        Self {
            profile,
            properties: ServerProperties::load_or_default(std::path::Path::new("definitely-missing-test-server.properties")).unwrap(),
            access,
        }
    }

    fn player(&self) -> GameRuleSessionPlayer<'_> {
        GameRuleSessionPlayer {
            profile: &self.profile,
            properties: &self.properties,
            player_access: &self.access,
        }
    }
}

#[test]
fn join_flags_follow_reduced_debug_immediate_respawn_and_limited_crafting() {
    let rules = shared_rules();
    assert_eq!(
        join_game_rule_flags(&rules),
        game_rule_live::JoinGameRuleFlags {
            reduced_debug_info: false,
            show_death_screen: true,
            do_limited_crafting: false,
        }
    );
    {
        let mut live = lock_status_mutex(&rules);
        live.set("reduced_debug_info", "true", false).unwrap();
        live.set("immediate_respawn", "true", false).unwrap();
        live.set("limited_crafting", "true", false).unwrap();
    }
    let flags = join_game_rule_flags(&rules);
    assert!(flags.reduced_debug_info && !flags.show_death_screen && flags.do_limited_crafting);
}

#[test]
fn changes_after_join_are_replayed_as_java_on_game_rule_changed() {
    let rules = shared_rules();
    let clock = Arc::new(Mutex::new(ServerClockManager::default()));
    let fixture = Fixture::new(None);
    let mut sync = GameRuleSessionSync::joined(&rules);
    {
        let mut live = lock_status_mutex(&rules);
        live.set("reduced_debug_info", "true", false).unwrap();
        live.set("limited_crafting", "true", false).unwrap();
        live.set("immediate_respawn", "false", false).unwrap();
        live.set("advance_time", "false", false).unwrap();
        live.set("pvp", "false", false).unwrap();
    }
    let mut out = Vec::new();
    game_rule_live::replay_game_rule_changes(
        &mut out,
        CompressionState::disabled(),
        &mut sync,
        &rules,
        &clock,
        &fixture.player(),
    )
    .unwrap();
    assert_eq!(
        packet_ids(&out),
        vec![
            CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
            CLIENTBOUND_GAME_EVENT_PACKET_ID,
            CLIENTBOUND_GAME_EVENT_PACKET_ID,
            CLIENTBOUND_SET_TIME_PACKET_ID,
        ]
    );

    // Already-seen changes are not replayed.
    let mut again = Vec::new();
    game_rule_live::replay_game_rule_changes(
        &mut again,
        CompressionState::disabled(),
        &mut sync,
        &rules,
        &clock,
        &fixture.player(),
    )
    .unwrap();
    assert!(again.is_empty());
}

#[test]
fn set_game_rule_packet_requires_gamemaster_and_announces_to_ops() {
    let rules = shared_rules();
    let clock = Arc::new(Mutex::new(ServerClockManager::default()));

    let plain = Fixture::new(None);
    let mut packet = set_game_rule_packet(&[("minecraft:keep_inventory", "true")]);
    game_rule_live::handle_set_game_rule_packet(&mut packet, &rules, &plain.player()).unwrap();
    assert!(!lock_status_mutex(&rules).bool("keep_inventory"), "non-op is rejected");

    let admin = Fixture::new(Some(2));
    let mut sync = GameRuleSessionSync::joined(&rules);
    let mut plain_sync = GameRuleSessionSync::joined(&rules);
    let mut packet = set_game_rule_packet(&[
        ("minecraft:keep_inventory", "true"),
        ("minecraft:random_tick_speed", "-4"),
        ("minecraft:unknown_rule", "1"),
        ("other:keep_inventory", "false"),
    ]);
    game_rule_live::handle_set_game_rule_packet(&mut packet, &rules, &admin.player()).unwrap();
    assert!(lock_status_mutex(&rules).bool("keep_inventory"));
    assert_eq!(
        lock_status_mutex(&rules).get("random_tick_speed"),
        Some(GameRuleValue::Int(3)),
        "an unparsable value is ignored"
    );

    // The op session receives the commands.gamerule.set message; a non-op session does not.
    let mut op_out = Vec::new();
    game_rule_live::replay_game_rule_changes(
        &mut op_out,
        CompressionState::disabled(),
        &mut sync,
        &rules,
        &clock,
        &admin.player(),
    )
    .unwrap();
    assert_eq!(packet_ids(&op_out), vec![CLIENTBOUND_SYSTEM_CHAT_PACKET_ID]);

    let mut plain_out = Vec::new();
    game_rule_live::replay_game_rule_changes(
        &mut plain_out,
        CompressionState::disabled(),
        &mut plain_sync,
        &rules,
        &clock,
        &plain.player(),
    )
    .unwrap();
    assert!(plain_out.is_empty(), "non-ops get no announcement and no client-visible rule");
}

#[test]
fn request_gamerule_values_lists_enabled_rules_for_gamemasters_only() {
    let rules = shared_rules();
    let mut denied = Vec::new();
    game_rule_live::handle_request_gamerule_values(
        &mut denied,
        CompressionState::disabled(),
        &rules,
        &Fixture::new(None).player(),
    )
    .unwrap();
    assert!(denied.is_empty());

    let mut out = Vec::new();
    game_rule_live::handle_request_gamerule_values(
        &mut out,
        CompressionState::disabled(),
        &rules,
        &Fixture::new(Some(2)).player(),
    )
    .unwrap();
    assert_eq!(packet_ids(&out), vec![CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID]);
}

#[test]
fn client_command_request_gamerule_values_is_routed_before_state_updates() {
    let rules = shared_rules();
    let fixture = Fixture::new(Some(4));
    let mut out = Vec::new();
    let mut request = Cursor::new(vec![game_rule_live::CLIENT_COMMAND_REQUEST_GAMERULE_VALUES as u8]);
    assert!(game_rule_live::try_handle_game_rule_packet(
        &mut out,
        CompressionState::disabled(),
        SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
        &mut request,
        &rules,
        &fixture.player(),
    )
    .unwrap());
    assert_eq!(packet_ids(&out), vec![CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID]);

    let mut respawn = Cursor::new(vec![0u8]);
    assert!(!game_rule_live::try_handle_game_rule_packet(
        &mut Vec::new(),
        CompressionState::disabled(),
        SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
        &mut respawn,
        &rules,
        &fixture.player(),
    )
    .unwrap());
}

#[test]
fn gamerule_command_state_round_trips_through_the_live_store() {
    let rules = shared_rules();
    let mut state = ServerCommandState::default();
    game_rule_live::seed_command_game_rules(&mut state, &rules);
    let result = crate::command::execute_builtin_command(
        &mut state,
        crate::command::LevelBasedPermissionSet::GAMEMASTER,
        "gamerule keep_inventory true",
    )
    .unwrap();
    assert_eq!(state.feedback_args, vec!["keep_inventory", "true"]);
    assert_eq!(result.feedback_key, "commands.gamerule.set");
    game_rule_live::apply_command_game_rule_changes(&state, &rules);
    assert!(lock_status_mutex(&rules).bool("keep_inventory"));
    assert_eq!(lock_status_mutex(&rules).changes_since(0).len(), 1);

    let mut query = ServerCommandState::default();
    game_rule_live::seed_command_game_rules(&mut query, &rules);
    crate::command::execute_builtin_command(
        &mut query,
        crate::command::LevelBasedPermissionSet::GAMEMASTER,
        "gamerule keep_inventory",
    )
    .unwrap();
    assert_eq!(query.feedback_args, vec!["keep_inventory", "true"]);
}
