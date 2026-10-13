// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use std::io::Cursor;

use super::*;
use crate::xp_orb_entity::XpOrbRandom;

fn profile() -> NameAndId {
    NameAndId {
        uuid: "00000000-0000-0000-0000-000000000001".to_string(),
        name: "Steve".to_string(),
    }
}

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

fn store_with_orb(x: f64, y: f64, z: f64, value: i32) -> Arc<Mutex<WorldItemEntities>> {
    let mut store = WorldItemEntities::new();
    let id = store.alloc_entity_id();
    let mut orb = XpOrbEntity::spawn(
        id,
        (x, y, z),
        (0.0, 0.0, 0.0),
        value,
        &mut XpOrbRandom::new(1),
    );
    orb.vel_x = 0.0;
    orb.vel_y = 0.0;
    orb.vel_z = 0.0;
    store.xp_orbs.push(orb);
    Arc::new(Mutex::new(store))
}

fn player_at(x: f64, y: f64, z: f64) -> PlaySessionState {
    let mut state = PlaySessionState::default();
    state.x = x;
    state.y = y;
    state.z = z;
    state
}

#[test]
fn spawn_packets_are_a_bundle_of_add_entity_and_value_metadata() {
    let orb = XpOrbEntity::spawn(
        5,
        (1.0, 2.0, 3.0),
        (0.0, 0.0, 0.0),
        37,
        &mut XpOrbRandom::new(1),
    );
    let mut wire = Vec::new();
    write_xp_orb_spawn_packets(&mut wire, CompressionState::disabled(), &orb).unwrap();
    let sent = frames(&wire);
    let ids: Vec<i32> = sent.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids,
        vec![
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
            CLIENTBOUND_ADD_ENTITY_PACKET_ID,
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID
        ]
    );
    let mut add = sent[1].1.as_slice();
    assert_eq!(read_var_i32(&mut add).unwrap(), 5);
    add = &add[16..]; // UUID
    assert_eq!(read_var_i32(&mut add).unwrap(), XP_ORB_ENTITY_TYPE_ID);
    // SET_ENTITY_DATA: id, index 8, INT serializer, value, terminator.
    let mut data = sent[2].1.as_slice();
    assert_eq!(read_var_i32(&mut data).unwrap(), 5);
    assert_eq!(data[0], 8);
    data = &data[1..];
    assert_eq!(read_var_i32(&mut data).unwrap(), 1);
    assert_eq!(read_var_i32(&mut data).unwrap(), 37);
    assert_eq!(data, [0xFF]);
}

#[test]
fn orb_entity_type_matches_registry_order() {
    // EntityType registration order: experience_orb is 49, item is 71.
    assert_eq!(XP_ORB_ENTITY_TYPE_ID, 49);
    assert_eq!(ITEM_ENTITY_TYPE_ID, 71);
}

#[test]
fn player_collects_orb_in_reach_and_gains_experience() {
    let world = store_with_orb(0.5, 64.0, 0.5, 7);
    let bus = WorldPacketBus::default();
    let inbox = bus.subscribe(3);
    let mut state = player_at(0.5, 64.0, 0.5);
    let mut wire = Vec::new();
    tick_xp_orbs(
        &mut wire,
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();

    // Level 0 needs 7 points: a 7-point orb is exactly one level, no leftover progress.
    assert_eq!((state.xp_level, state.xp_total), (1, 7));
    assert_eq!(
        state.score, 7,
        "Player.giveExperiencePoints -> increaseScore"
    );
    assert_eq!(state.combat.take_xp_delay, 2);
    assert!(lock_status_mutex(&world).xp_orbs.is_empty());

    let sent = frames(&wire);
    assert_eq!(sent[0].0, CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID);
    assert_eq!(sent[1].0, CLIENTBOUND_SET_EXPERIENCE_PACKET_ID);
    let mut broadcast = Vec::new();
    inbox
        .drain_into(&mut broadcast, CompressionState::disabled())
        .unwrap();
    assert_eq!(
        frames(&broadcast)[0].0,
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID
    );
}

#[test]
fn take_xp_delay_limits_pickups_to_one_orb_count_per_two_ticks() {
    let world = store_with_orb(0.0, 64.0, 0.0, 1);
    lock_status_mutex(&world).xp_orbs[0].orb.count = 3;
    let bus = WorldPacketBus::default();
    let mut state = player_at(0.0, 64.0, 0.0);
    let mut wire = Vec::new();
    for _ in 0..2 {
        tick_xp_orbs(
            &mut wire,
            CompressionState::disabled(),
            &mut state,
            &world,
            &bus,
            &profile(),
        )
        .unwrap();
    }
    // The second call happens with takeXpDelay still 2 (nothing decremented it).
    assert_eq!(state.xp_total, 1);
    assert_eq!(lock_status_mutex(&world).xp_orbs[0].orb.count, 2);
    state.combat.take_xp_delay = 0;
    tick_xp_orbs(
        &mut wire,
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(state.xp_total, 2);
    assert_eq!(lock_status_mutex(&world).xp_orbs[0].orb.count, 1);
}

#[test]
fn orbs_out_of_reach_or_for_dead_and_spectator_players_stay_put() {
    let bus = WorldPacketBus::default();
    let far = store_with_orb(50.0, 64.0, 0.0, 3);
    let mut state = player_at(0.0, 64.0, 0.0);
    tick_xp_orbs(
        &mut Vec::new(),
        CompressionState::disabled(),
        &mut state,
        &far,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(state.xp_total, 0);

    let near = store_with_orb(0.0, 64.0, 0.0, 3);
    let mut dead = player_at(0.0, 64.0, 0.0);
    dead.health = 0.0;
    tick_xp_orbs(
        &mut Vec::new(),
        CompressionState::disabled(),
        &mut dead,
        &near,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(dead.xp_total, 0);

    let mut spectator = player_at(0.0, 64.0, 0.0);
    spectator.game_mode = GameMode::Spectator;
    tick_xp_orbs(
        &mut Vec::new(),
        CompressionState::disabled(),
        &mut spectator,
        &near,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(spectator.xp_total, 0);
    assert_eq!(lock_status_mutex(&near).xp_orbs.len(), 1);
}

#[test]
fn shared_clock_steps_orbs_once_per_interval_regardless_of_session_count() {
    let world = store_with_orb(0.0, 64.0, 0.0, 1);
    let bus = WorldPacketBus::default();
    // Player beyond pickup reach but within follow range so motion is observable.
    let mut state = player_at(6.0, 64.0, 0.0);
    for _ in 0..3 {
        tick_xp_orbs(
            &mut Vec::new(),
            CompressionState::disabled(),
            &mut state,
            &world,
            &bus,
            &profile(),
        )
        .unwrap();
    }
    assert_eq!(
        lock_status_mutex(&world).xp_orbs[0].tick_count,
        1,
        "three back-to-back session ticks advance the shared orb once"
    );
}

#[test]
fn expired_orbs_are_announced_to_every_session() {
    let world = store_with_orb(50.0, 64.0, 0.0, 1);
    lock_status_mutex(&world).xp_orbs[0].orb.age = 5999;
    let bus = WorldPacketBus::default();
    let inbox = bus.subscribe(3);
    let mut state = player_at(0.0, 64.0, 0.0);
    tick_xp_orbs(
        &mut Vec::new(),
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();
    assert!(lock_status_mutex(&world).xp_orbs.is_empty());
    let mut wire = Vec::new();
    inbox
        .drain_into(&mut wire, CompressionState::disabled())
        .unwrap();
    assert_eq!(frames(&wire)[0].0, CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID);
}

/// Applied enchantments live on the item; stored book enchantments do not repair.
fn damaged_mending_stack(damage: u32) -> ItemStack {
    let mut stack = ItemStack::new("minecraft:diamond_sword", 1);
    stack.set_damage_value(damage);
    stack.set_component(crate::item_properties::ItemComponent::Enchantments(
        std::collections::BTreeMap::from([("minecraft:mending".to_owned(), 1)]),
    ));
    stack
}

#[test]
fn pickup_repairs_selected_equipment_and_syncs_components_before_awarding_remainder() {
    let world = store_with_orb(0.0, 64.0, 0.0, 5);
    let bus = WorldPacketBus::default();
    let mut state = player_at(0.0, 64.0, 0.0);
    let inventory = state.inventory_menu.player_inventory_mut();
    inventory.set_selected_slot(3).unwrap();
    inventory.set(3, damaged_mending_stack(5));
    // A damaged enchanted item in an unselected hotbar slot is not equipment.
    inventory.set(0, damaged_mending_stack(10));
    // Player.increaseScore uses Java int addition, including overflow.
    state.score = i32::MAX - 1;
    let mut wire = Vec::new();
    tick_xp_orbs(
        &mut wire,
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(
        state
            .inventory_menu
            .player_inventory()
            .get(3)
            .damage_value(),
        0
    );
    assert_eq!(
        state
            .inventory_menu
            .player_inventory()
            .get(0)
            .damage_value(),
        10
    );
    // Java truncates 5 * 5 / 10: two XP spent, three awarded.
    assert_eq!(state.xp_total, 3);
    assert_eq!(state.score, i32::MIN + 1);
    let sent = frames(&wire);
    assert_eq!(
        sent.iter().map(|frame| frame.0).collect::<Vec<_>>(),
        [
            CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
            CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        ]
    );
    // Compare the trusted wire form for every slot. This catches durability and
    // enchantment patches being discarded by the inventory synchronization path.
    let mut expected = vec![0];
    write_var_i32(&mut expected, state.container_state_id).unwrap();
    let slots = state.inventory_menu.all_slots();
    write_var_i32(&mut expected, slots.len() as i32).unwrap();
    for stack in &slots {
        raw_item_stack_from_item_stack(stack)
            .unwrap()
            .write_optional_trusted(&mut expected)
            .unwrap();
    }
    raw_item_stack_from_item_stack(&state.carried_item)
        .unwrap()
        .write_optional_trusted(&mut expected)
        .unwrap();
    assert_eq!(sent[1].1, expected);
}

#[test]
fn orb_spent_entirely_on_offhand_repair_still_consumes_pickup_without_xp_packet() {
    let world = store_with_orb(0.0, 64.0, 0.0, 3);
    lock_status_mutex(&world).xp_orbs[0].orb.count = 2;
    let bus = WorldPacketBus::default();
    let mut state = player_at(0.0, 64.0, 0.0);
    state
        .inventory_menu
        .player_inventory_mut()
        .set(40, damaged_mending_stack(9));
    let mut wire = Vec::new();
    tick_xp_orbs(
        &mut wire,
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();
    assert_eq!(
        state
            .inventory_menu
            .player_inventory()
            .get(40)
            .damage_value(),
        3
    );
    assert_eq!((state.xp_total, state.score), (0, 0));
    assert_eq!(state.combat.take_xp_delay, 2);
    assert_eq!(lock_status_mutex(&world).xp_orbs[0].orb.count, 1);
    assert_eq!(
        frames(&wire)
            .iter()
            .map(|frame| frame.0)
            .collect::<Vec<_>>(),
        [
            CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        ]
    );
    let before = state.inventory_menu.player_inventory().times_changed();
    wire.clear();
    tick_xp_orbs(
        &mut wire,
        CompressionState::disabled(),
        &mut state,
        &world,
        &bus,
        &profile(),
    )
    .unwrap();
    assert!(wire.is_empty());
    assert_eq!(
        state.inventory_menu.player_inventory().times_changed(),
        before
    );
}
