// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use std::collections::BTreeMap;

use super::*;
use super::super::player_death::{hurt_player, hurt_player_by_mob};

fn survival_player() -> PlaySessionState {
    PlaySessionState::default()
}

fn equip(state: &mut PlaySessionState, menu_slot: usize, item: &'static str, enchant: Option<(&str, i32)>) {
    let mut stack = ItemStack::new(item, 1);
    if let Some((id, level)) = enchant {
        stack.set_component(ItemComponent::Enchantments(BTreeMap::from([(id.to_string(), level)])));
    }
    assert!(state.inventory_menu.set_slot(menu_slot, stack));
}

fn add_effect(state: &mut PlaySessionState, id: &str, amplifier: i8) {
    state.active_effects.push(Tag::Compound(vec![
        ("id".to_string(), Tag::String(id.to_string())),
        ("amplifier".to_string(), Tag::Byte(amplifier)),
        ("duration".to_string(), Tag::Int(600)),
    ]));
}

fn set_rules(state: &mut PlaySessionState, edit: impl FnOnce(&mut DamageRules)) {
    edit(&mut state.combat.hurt.rules);
}

#[test]
fn hurt_cooldown_only_lets_larger_hits_through_for_the_difference() {
    // LivingEntity.hurtServer: invulnerableTime > 10 -> only `damage - lastHurt` is dealt.
    let mut state = survival_player();
    assert!(hurt_player(&mut state, "minecraft:generic", 6.0));
    assert_eq!(state.health, 14.0);
    assert_eq!(state.combat.hurt.invulnerable_time, 20);
    assert!(!hurt_player(&mut state, "minecraft:generic", 4.0), "not above lastHurt");
    assert!(!hurt_player(&mut state, "minecraft:generic", 6.0), "equal is rejected");
    assert!(hurt_player(&mut state, "minecraft:generic", 8.0));
    assert_eq!(state.health, 12.0);
    assert_eq!(state.combat.hurt.last_hurt, 8.0);
    // After the cooldown falls to 10 a full hit lands again.
    for _ in 0..10 {
        tick_hurt_cooldown(&mut state);
    }
    assert_eq!(state.combat.hurt.invulnerable_time, 10);
    assert!(hurt_player(&mut state, "minecraft:generic", 3.0));
    assert_eq!(state.health, 9.0);
}

#[test]
fn creative_players_only_take_damage_that_bypasses_invulnerability() {
    let mut state = survival_player();
    state.abilities.invulnerable = true;
    assert!(!hurt_player(&mut state, "minecraft:generic", 5.0));
    assert!(!hurt_player(&mut state, "minecraft:lava", 4.0));
    assert_eq!(state.health, 20.0);
    assert!(hurt_player(&mut state, "minecraft:out_of_world", 4.0));
    assert_eq!(state.health, 16.0);
    assert!(hurt_player(&mut state, "minecraft:generic_kill", f32::MAX));
    assert_eq!(state.health, 0.0);
}

#[test]
fn dead_players_cannot_be_hurt_again() {
    let mut state = survival_player();
    state.health = 0.0;
    assert!(!hurt_player(&mut state, "minecraft:generic_kill", 1.0));
}

#[test]
fn damage_game_rules_gate_their_damage_tags() {
    for (rule, damage_type) in [
        ("drowning", "minecraft:drown"),
        ("fall", "minecraft:fall"),
        ("fire", "minecraft:in_fire"),
        ("fire", "minecraft:on_fire"),
        ("fire", "minecraft:lava"),
        ("freeze", "minecraft:freeze"),
    ] {
        let mut state = survival_player();
        set_rules(&mut state, |rules| match rule {
            "drowning" => rules.drowning_damage = false,
            "fall" => rules.fall_damage = false,
            "fire" => rules.fire_damage = false,
            _ => rules.freeze_damage = false,
        });
        assert!(!hurt_player(&mut state, damage_type, 2.0), "{damage_type}");
        assert_eq!(state.health, 20.0, "{damage_type}");
        // Unrelated damage still lands.
        assert!(hurt_player(&mut state, "minecraft:generic", 2.0));
    }
}

#[test]
fn damage_tags_come_from_the_loaded_tag_data() {
    let source = |id: &str| super::super::player_death::simple_damage_source(id);
    assert!(source_is(&source("minecraft:fall"), DamageTag::BypassesArmor));
    assert!(source_is(&source("minecraft:fall"), DamageTag::IsFall));
    assert!(source_is(&source("minecraft:starve"), DamageTag::BypassesEffects));
    assert!(source_is(&source("minecraft:out_of_world"), DamageTag::BypassesInvulnerability));
    assert!(source_is(&source("minecraft:lava"), DamageTag::IsFire));
    assert!(!source_is(&source("minecraft:cactus"), DamageTag::BypassesArmor));
    // bypasses_cooldown has no data file in vanilla, so it is an empty tag.
    assert!(!source_is(&source("minecraft:falling_anvil"), DamageTag::BypassesCooldown));
    assert!(source_is(&source("minecraft:falling_anvil"), DamageTag::DamagesHelmet));
}

#[test]
fn full_diamond_armor_reduces_mob_attacks_like_combat_rules() {
    let mut state = survival_player();
    for (slot, item) in [
        (5, "minecraft:diamond_helmet"),
        (6, "minecraft:diamond_chestplate"),
        (7, "minecraft:diamond_leggings"),
        (8, "minecraft:diamond_boots"),
    ] {
        equip(&mut state, slot, item, None);
    }
    assert_eq!(armor_totals(&state), (20.0, 8.0));
    // toughness 4 -> realArmor = clamp(20 - 10 / 4, 4, 20) = 17.5 -> 10 * (1 - 0.7).
    assert!(hurt_player_by_mob(&mut state, 7, "minecraft:zombie", [0.0, 64.0, 2.0], 10.0));
    assert!((state.health - 17.0).abs() < 1.0e-5, "{}", state.health);
    // Armor wears by max(1, damage / 4) = 2 on every piece.
    let helmet = state.inventory_menu.get_slot(5).unwrap();
    assert_eq!(helmet.damage_value(), 2);
}

#[test]
fn armor_is_bypassed_by_fall_but_feather_falling_still_applies() {
    let mut state = survival_player();
    equip(&mut state, 8, "minecraft:iron_boots", Some(("minecraft:feather_falling", 4)));
    assert!(hurt_player(&mut state, "minecraft:fall", 10.0));
    // 12 protection points -> 10 * (1 - 12 / 25) = 5.2; iron boots are not worn down.
    assert!((state.health - 14.8).abs() < 1.0e-5, "{}", state.health);
    assert_eq!(state.inventory_menu.get_slot(8).unwrap().damage_value(), 0);
}

#[test]
fn fire_protection_only_counts_against_fire_damage() {
    let mut fire = survival_player();
    equip(&mut fire, 5, "minecraft:golden_helmet", Some(("minecraft:fire_protection", 2)));
    hurt_player(&mut fire, "minecraft:in_fire", 10.0);
    // in_fire does not bypass armor: golden helmet (2 armor): realArmor = clamp(2 - 10 / 2, 0.4, 20)
    // -> 10 * 0.984 = 9.84, then 4 protection points -> * (1 - 4 / 25).
    assert!((fire.health - (20.0 - 9.84 * 0.84)).abs() < 1.0e-4, "{}", fire.health);
    let mut other = survival_player();
    equip(&mut other, 5, "minecraft:golden_helmet", Some(("minecraft:fire_protection", 2)));
    hurt_player(&mut other, "minecraft:generic", 10.0);
    // generic bypasses armor and the enchantment does not match it.
    assert_eq!(other.health, 10.0);
}

#[test]
fn resistance_scales_damage_unless_the_source_bypasses_it() {
    let mut state = survival_player();
    add_effect(&mut state, "minecraft:resistance", 1);
    // amplifier 1 -> (25 - 10) / 25 = 0.6.
    hurt_player(&mut state, "minecraft:cactus", 10.0);
    assert!((state.health - 14.0).abs() < 1.0e-5, "{}", state.health);
    // out_of_world bypasses resistance.
    state.combat.hurt.invulnerable_time = 0;
    hurt_player(&mut state, "minecraft:out_of_world", 4.0);
    assert!((state.health - 10.0).abs() < 1.0e-5, "{}", state.health);
}

#[test]
fn fire_resistance_blocks_fire_damage_entirely() {
    let mut state = survival_player();
    add_effect(&mut state, "minecraft:fire_resistance", 0);
    assert!(!hurt_player(&mut state, "minecraft:lava", 4.0));
    assert!(!hurt_player(&mut state, "minecraft:on_fire", 1.0));
    assert_eq!(state.health, 20.0);
    assert!(hurt_player(&mut state, "minecraft:cactus", 1.0));
}

#[test]
fn absorption_soaks_damage_before_health() {
    let mut state = survival_player();
    state.combat.hurt.absorption = 4.0;
    hurt_player(&mut state, "minecraft:generic", 3.0);
    assert_eq!((state.health, state.combat.hurt.absorption), (20.0, 1.0));
    state.combat.hurt.invulnerable_time = 0;
    hurt_player(&mut state, "minecraft:generic", 3.0);
    assert_eq!((state.health, state.combat.hurt.absorption), (18.0, 0.0));
}

#[test]
fn mob_damage_scales_with_difficulty() {
    let hurt = |difficulty: Difficulty| {
        let mut state = survival_player();
        set_rules(&mut state, |rules| rules.difficulty = difficulty);
        let accepted = hurt_player_by_mob(&mut state, 3, "minecraft:zombie", [0.0; 3], 4.0);
        (accepted, state.health)
    };
    assert_eq!(hurt(Difficulty::Peaceful), (false, 20.0));
    assert_eq!(hurt(Difficulty::Easy), (true, 17.0)); // min(4 / 2 + 1, 4) = 3
    assert_eq!(hurt(Difficulty::Normal), (true, 16.0));
    assert_eq!(hurt(Difficulty::Hard), (true, 14.0));
}

#[test]
fn damage_pushes_exhaustion_from_the_damage_type() {
    let mut state = survival_player();
    hurt_player(&mut state, "minecraft:cactus", 1.0);
    // DamageTypes.CACTUS exhaustion 0.1.
    assert!((state.food_exhaustion - 0.1).abs() < 1.0e-6, "{}", state.food_exhaustion);
}

fn frames(bytes: &[u8]) -> Vec<(i32, Vec<u8>)> {
    let mut cursor = std::io::Cursor::new(bytes);
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

#[test]
fn full_hits_send_a_damage_event_and_mob_hits_add_a_hurt_animation() {
    let mut state = survival_player();
    hurt_player(&mut state, "minecraft:cactus", 1.0);
    let mut sent = Vec::new();
    flush_hurt_packets(&mut sent, CompressionState::disabled(), &mut state).unwrap();
    let packets = frames(&sent);
    assert_eq!(packets.len(), 1, "cactus has no knockback so no hurt animation");
    assert_eq!(packets[0].0, CLIENTBOUND_DAMAGE_EVENT_PACKET_ID);
    let event = ClientboundDamageEventPacket::read(&mut packets[0].1.as_slice()).unwrap();
    assert_eq!(event.entity_id, PLAYER_ENTITY_ID);
    assert_eq!(
        Some(event.source_type_id as usize),
        registry_element_id("minecraft:damage_type", "minecraft:cactus")
    );
    assert_eq!((event.source_cause_id, event.source_direct_id), (-1, -1));

    state.combat.hurt.invulnerable_time = 0;
    state.yaw = 0.0;
    hurt_player_by_mob(&mut state, 42, "minecraft:zombie", [0.5, 64.0, 5.0], 2.0);
    let mut sent = Vec::new();
    flush_hurt_packets(&mut sent, CompressionState::disabled(), &mut state).unwrap();
    let packets = frames(&sent);
    assert_eq!(
        packets.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![CLIENTBOUND_DAMAGE_EVENT_PACKET_ID, CLIENTBOUND_HURT_ANIMATION_PACKET_ID]
    );
    let event = ClientboundDamageEventPacket::read(&mut packets[0].1.as_slice()).unwrap();
    assert_eq!((event.source_cause_id, event.source_direct_id), (42, 42));
    let animation = ClientboundHurtAnimationPacket::read(&mut packets[1].1.as_slice()).unwrap();
    // atan2(dz = 5 - 0 (player z default 0.5 -> 4.5), dx) points along +z: 90 degrees.
    assert_eq!(animation.id, PLAYER_ENTITY_ID);
    assert!((animation.yaw - 90.0).abs() < 1.0, "{}", animation.yaw);
}

#[test]
fn cooldown_rejected_and_partial_hits_send_no_damage_event() {
    let mut state = survival_player();
    hurt_player(&mut state, "minecraft:generic", 6.0);
    hurt_player(&mut state, "minecraft:generic", 8.0);
    let mut sent = Vec::new();
    flush_hurt_packets(&mut sent, CompressionState::disabled(), &mut state).unwrap();
    // tookFullDamage is false for the second hit: exactly one DamageEvent.
    assert_eq!(frames(&sent).len(), 1);
}

#[test]
fn hurt_cooldown_counts_down_once_per_tick_and_stops_at_zero() {
    let mut state = survival_player();
    state.combat.hurt.invulnerable_time = 1;
    tick_hurt_cooldown(&mut state);
    tick_hurt_cooldown(&mut state);
    assert_eq!(state.combat.hurt.invulnerable_time, 0);
}

#[test]
fn breaking_helmet_queues_the_equipment_break_event() {
    let mut state = survival_player();
    let mut helmet = ItemStack::new("minecraft:leather_helmet", 1);
    helmet.set_damage_value(helmet.max_damage() - 1);
    assert!(state.inventory_menu.set_slot(5, helmet));
    hurt_player_by_mob(&mut state, 1, "minecraft:zombie", [0.0; 3], 3.0);
    assert!(state.inventory_menu.get_slot(5).unwrap().is_empty());
    assert!(state.combat.hurt.pending_packets.contains(&PendingHurtPacket::EntityEvent(49)));
    assert!(state.combat.hurt.pending_packets.contains(&PendingHurtPacket::InventorySync));
}
