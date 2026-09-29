// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use std::io::Cursor;

use super::*;
use crate::game_rules::{GameRules, LiveGameRules};
use crate::network::play::ClientboundPlayerCombatKillPacket;

struct Fixture {
    rules: SharedGameRules,
    items: Arc<Mutex<WorldItemEntities>>,
    mobs: Arc<Mutex<LiveMobStore>>,
    bus: WorldPacketBus,
    profile: NameAndId,
}

impl Fixture {
    fn new() -> Self {
        Self {
            rules: LiveGameRules::shared(GameRules::new(false)),
            items: Arc::new(Mutex::new(WorldItemEntities::new())),
            mobs: Arc::new(Mutex::new(LiveMobStore::default())),
            bus: WorldPacketBus::default(),
            profile: NameAndId {
                uuid: "00000000-0000-0000-0000-000000000001".to_string(),
                name: "Steve".to_string(),
            },
        }
    }

    fn context(&self) -> PlayerLifecycleContext<'_> {
        PlayerLifecycleContext {
            game_rules: &self.rules,
            world_items: &self.items,
            world_mobs: &self.mobs,
            bus: &self.bus,
            profile: &self.profile,
        }
    }

    fn set_rule(&self, rule: &str, value: &str) {
        lock_status_mutex(&self.rules).set(rule, value, false).unwrap();
    }
}

/// Splits uncompressed frames into `(packet id, body)` pairs.
fn frames(bytes: &[u8]) -> Vec<(i32, Vec<u8>)> {
    let mut cursor = Cursor::new(bytes);
    let mut out = Vec::new();
    while (cursor.position() as usize) < bytes.len() {
        let length = read_var_i32(&mut cursor).unwrap() as usize;
        let start = cursor.position() as usize;
        let mut payload = &bytes[start..start + length];
        let id = read_var_i32(&mut payload).unwrap();
        out.push((id, payload.to_vec()));
        cursor.set_position((start + length) as u64);
    }
    out
}

fn dead_player() -> PlaySessionState {
    let mut state = PlaySessionState::default();
    state.x = 10.5;
    state.y = 70.0;
    state.z = -3.5;
    state.health = 0.0;
    state
}

fn kill_message(sent: &[u8]) -> String {
    let (id, body) = frames(sent).into_iter().next().expect("kill packet");
    assert_eq!(id, CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID);
    let packet = ClientboundPlayerCombatKillPacket::read(&mut body.as_slice()).unwrap();
    assert_eq!(packet.player_id, PLAYER_ENTITY_ID);
    packet.message
}

#[test]
fn fall_death_sends_combat_kill_with_java_fall_message() {
    let fixture = Fixture::new();
    let mut state = PlaySessionState::default();
    state.fall_distance = 30.0;
    hurt_player(&mut state, "minecraft:fall", 40.0);
    assert_eq!(state.health, 0.0);
    let mut sent = Vec::new();
    die(&mut sent, CompressionState::disabled(), &mut state, &fixture.context(), 1).unwrap();
    // CombatTracker.getDeathMessage: fall >5 blocks -> death.fell.accident.generic.
    assert!(kill_message(&sent).contains("death.fell.accident.generic"));
    assert!(state.combat.dead);
}

#[test]
fn death_broadcasts_the_message_as_system_chat_to_every_session() {
    let fixture = Fixture::new();
    let inbox = fixture.bus.subscribe(9);
    let mut state = PlaySessionState::default();
    hurt_player(&mut state, "minecraft:drown", 30.0);
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 1).unwrap();
    let mut wire = Vec::new();
    inbox.drain_into(&mut wire, CompressionState::disabled()).unwrap();
    let chat: Vec<_> = frames(&wire)
        .into_iter()
        .filter(|(id, _)| *id == CLIENTBOUND_SYSTEM_CHAT_PACKET_ID)
        .collect();
    assert_eq!(chat.len(), 1, "one death message broadcast");
    let body = &chat[0].1;
    let text = String::from_utf8_lossy(body);
    assert!(text.contains("death.attack.drown"), "{text}");
    assert!(text.contains("Steve"), "victim name is a component argument");
    assert_eq!(*body.last().unwrap(), 0, "overlay = false");
}

#[test]
fn show_death_messages_off_sends_empty_kill_message_and_no_broadcast() {
    let fixture = Fixture::new();
    fixture.set_rule("show_death_messages", "false");
    let inbox = fixture.bus.subscribe(9);
    let mut state = PlaySessionState::default();
    hurt_player(&mut state, "minecraft:starve", 20.0);
    let mut sent = Vec::new();
    die(&mut sent, CompressionState::disabled(), &mut state, &fixture.context(), 1).unwrap();
    assert!(!kill_message(&sent).contains("death."), "CommonComponents.EMPTY");
    let mut wire = Vec::new();
    inbox.drain_into(&mut wire, CompressionState::disabled()).unwrap();
    assert!(frames(&wire)
        .iter()
        .all(|(id, _)| *id != CLIENTBOUND_SYSTEM_CHAT_PACKET_ID));
}

#[test]
fn mob_kill_message_names_the_attacker() {
    let fixture = Fixture::new();
    let mut state = PlaySessionState::default();
    hurt_player_by_mob(&mut state, 77, "minecraft:zombie", [0.0; 3], 25.0);
    let mut sent = Vec::new();
    die(&mut sent, CompressionState::disabled(), &mut state, &fixture.context(), 1).unwrap();
    let message = kill_message(&sent);
    assert!(message.contains("death.attack.mob"), "{message}");
    assert!(message.contains("entity.minecraft.zombie"), "{message}");
}

#[test]
fn survival_death_drops_whole_inventory_as_item_entities() {
    let fixture = Fixture::new();
    let inbox = fixture.bus.subscribe(9);
    let mut state = dead_player();
    let inventory = state.inventory_menu.player_inventory_mut();
    inventory.set(0, ItemStack::new("minecraft:cobblestone", 32));
    inventory.set(20, ItemStack::new("minecraft:stick", 5));
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();

    for slot in 0..state.inventory_menu.player_inventory().container_size() {
        assert!(state.inventory_menu.player_inventory().get(slot).is_empty());
    }
    let items = lock_status_mutex(&fixture.items);
    let mut dropped: Vec<(&str, i32)> = items.entities.iter().map(|e| (e.item, e.count)).collect();
    dropped.sort_unstable();
    assert_eq!(dropped, vec![("minecraft:cobblestone", 32), ("minecraft:stick", 5)]);
    for item in &items.entities {
        // createItemStackToDrop: eyeY - 0.3, 40 tick pickup delay, upward 0.2 push.
        assert!((item.y - (70.0 + 1.62 - 0.3)).abs() < 1e-9);
        assert_eq!(item.pickup_delay, 40);
        assert_eq!(item.vel_y, 0.2);
        assert!(item.vel_x.hypot(item.vel_z) <= 0.5);
    }
    let mut wire = Vec::new();
    inbox.drain_into(&mut wire, CompressionState::disabled()).unwrap();
    let adds = frames(&wire)
        .into_iter()
        .filter(|(id, _)| *id == CLIENTBOUND_ADD_ENTITY_PACKET_ID)
        .count();
    assert_eq!(adds, 2, "every dropped stack is announced to all sessions");
}

#[test]
fn keep_inventory_keeps_items_and_drops_no_experience() {
    let fixture = Fixture::new();
    fixture.set_rule("keep_inventory", "true");
    let mut state = dead_player();
    state.xp_level = 10;
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:cobblestone", 32));
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();
    assert_eq!(state.inventory_menu.player_inventory().get(0).count(), 32);
    let items = lock_status_mutex(&fixture.items);
    assert!(items.entities.is_empty());
    assert!(items.xp_orbs.is_empty());
}

#[test]
fn spectator_death_drops_nothing() {
    let fixture = Fixture::new();
    let mut state = dead_player();
    state.game_mode = GameMode::Spectator;
    state.xp_level = 10;
    state
        .inventory_menu
        .player_inventory_mut()
        .set(0, ItemStack::new("minecraft:cobblestone", 32));
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();
    assert_eq!(state.inventory_menu.player_inventory().get(0).count(), 32);
    let items = lock_status_mutex(&fixture.items);
    assert!(items.entities.is_empty() && items.xp_orbs.is_empty());
}

#[test]
fn curse_of_vanishing_items_are_destroyed_not_dropped() {
    let fixture = Fixture::new();
    let mut state = dead_player();
    let mut cursed = ItemStack::new("minecraft:diamond_sword", 1);
    cursed.set_component(ItemComponent::Enchantments(
        [(VANISHING_CURSE.to_string(), 1)].into_iter().collect(),
    ));
    let inventory = state.inventory_menu.player_inventory_mut();
    inventory.set(0, cursed);
    inventory.set(1, ItemStack::new("minecraft:apple", 3));
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();
    let items = lock_status_mutex(&fixture.items);
    assert_eq!(items.entities.len(), 1);
    assert_eq!(items.entities[0].item, "minecraft:apple");
}

#[test]
fn death_awards_seven_xp_per_level_capped_at_one_hundred() {
    assert_eq!(death_experience_reward(10, false, GameMode::Survival), 70);
    assert_eq!(death_experience_reward(30, false, GameMode::Survival), 100);
    assert_eq!(death_experience_reward(0, false, GameMode::Survival), 0);
    assert_eq!(death_experience_reward(10, true, GameMode::Survival), 0);
    assert_eq!(death_experience_reward(10, false, GameMode::Spectator), 0);
}

#[test]
fn death_spawns_orbs_totalling_the_reward_and_announces_them() {
    let fixture = Fixture::new();
    let inbox = fixture.bus.subscribe(9);
    let mut state = dead_player();
    state.xp_level = 10;
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();
    let items = lock_status_mutex(&fixture.items);
    let total: i32 = items.xp_orbs.iter().map(|orb| orb.value() * orb.orb.count).sum();
    assert_eq!(total, 70);
    let mut wire = Vec::new();
    inbox.drain_into(&mut wire, CompressionState::disabled()).unwrap();
    let adds = frames(&wire)
        .into_iter()
        .filter(|(id, _)| *id == CLIENTBOUND_ADD_ENTITY_PACKET_ID)
        .count();
    assert_eq!(adds, items.xp_orbs.len());
}

#[test]
fn death_records_last_death_location() {
    let fixture = Fixture::new();
    let mut state = dead_player();
    die(&mut Vec::new(), CompressionState::disabled(), &mut state, &fixture.context(), 5).unwrap();
    assert_eq!(
        state.last_death_location,
        Some(PlayerGlobalPosData {
            dimension: "minecraft:overworld".to_string(),
            x: 10,
            y: 70,
            z: -4,
        })
    );
}

#[test]
fn lifecycle_tick_dies_exactly_once() {
    let fixture = Fixture::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let mut client = std::net::TcpStream::connect(addr).unwrap();
    let (mut server, _) = listener.accept().unwrap();
    let mut state = dead_player();
    tick_player_lifecycle(&mut server, CompressionState::disabled(), &mut state, &fixture.context())
        .unwrap();
    tick_player_lifecycle(&mut server, CompressionState::disabled(), &mut state, &fixture.context())
        .unwrap();
    drop(server);
    let mut wire = Vec::new();
    std::io::Read::read_to_end(&mut client, &mut wire).unwrap();
    let kills = frames(&wire)
        .into_iter()
        .filter(|(id, _)| *id == CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID)
        .count();
    assert_eq!(kills, 1);
    assert_eq!(state.combat.tick_count, 2);
}

#[test]
fn living_player_is_left_alone() {
    let fixture = Fixture::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let mut client = std::net::TcpStream::connect(addr).unwrap();
    let (mut server, _) = listener.accept().unwrap();
    let mut state = PlaySessionState::default();
    tick_player_lifecycle(&mut server, CompressionState::disabled(), &mut state, &fixture.context())
        .unwrap();
    drop(server);
    let mut wire = Vec::new();
    std::io::Read::read_to_end(&mut client, &mut wire).unwrap();
    assert!(wire.is_empty());
    assert!(!state.combat.dead);
}

#[test]
fn hurt_helpers_record_combat_entries_and_clamp_health() {
    let mut state = PlaySessionState::default();
    hurt_player(&mut state, "minecraft:drown", 2.0);
    assert_eq!(state.health, 18.0);
    hurt_player_by_mob(&mut state, 3, "minecraft:zombie", [0.0; 3], 100.0);
    assert_eq!(state.health, 0.0);
    assert_eq!(state.combat.tracker.entries.len(), 2);
    assert_eq!(state.combat.last_hurt_by_mob, Some((3, 0)));
    assert_eq!(state.combat.attacker_types.get(&3).map(String::as_str), Some("minecraft:zombie"));
}

#[test]
fn kill_command_kills_even_creative_players_with_the_generic_kill_message() {
    let fixture = Fixture::new();
    let mut state = PlaySessionState::default();
    state.game_mode = GameMode::Creative;
    state.abilities = PlayerNbtAbilities::for_game_mode(GameMode::Creative);
    kill_player(&mut state);
    assert_eq!(state.health, 0.0);
    let mut sent = Vec::new();
    die(&mut sent, CompressionState::disabled(), &mut state, &fixture.context(), 1).unwrap();
    assert!(kill_message(&sent).contains("death.attack.genericKill"));
}

/// `/kill` targeting the executing player zeroes its health and syncs it; a command
/// state that killed someone else leaves the player alone.
#[test]
fn apply_kill_command_only_kills_the_targeted_executing_player() {
    let fixture = Fixture::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let mut stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let mut command_state = crate::command::ServerCommandState::default();

    let mut state = PlaySessionState::default();
    let untouched = state.health;
    apply_kill_command(&mut stream, CompressionState::disabled(), &mut state, &fixture.profile, &command_state)
        .unwrap();
    assert_eq!(state.health, untouched, "no kill recorded, so nothing happens");

    command_state.killed_entities.push(crate::command::EntityRef {
        id: "Alex".to_string(),
        display_name: "Alex".to_string(),
    });
    apply_kill_command(&mut stream, CompressionState::disabled(), &mut state, &fixture.profile, &command_state)
        .unwrap();
    assert_eq!(state.health, untouched, "a different player was killed");

    command_state.killed_entities.push(crate::command::EntityRef {
        id: "Steve".to_string(),
        display_name: "Steve".to_string(),
    });
    apply_kill_command(&mut stream, CompressionState::disabled(), &mut state, &fixture.profile, &command_state)
        .unwrap();
    assert_eq!(state.health, 0.0);
}
