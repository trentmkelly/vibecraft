//! Explosion-related parts of the player hurt pipeline.

use std::collections::BTreeMap;

use super::*;

fn survival_player() -> PlaySessionState {
    PlaySessionState::default()
}

fn equip(state: &mut PlaySessionState, menu_slot: usize, item: &'static str, level: i32) {
    let mut stack = ItemStack::new(item, 1);
    stack.set_component(ItemComponent::Enchantments(BTreeMap::from([(
        "minecraft:blast_protection".to_string(),
        level,
    )])));
    assert!(state.inventory_menu.set_slot(menu_slot, stack));
}

#[test]
fn blast_protection_grants_explosion_knockback_resistance_per_worn_piece() {
    // blast_protection.json: attributes add_value linear(0.15, 0.15 per level above 1).
    let mut state = survival_player();
    assert_eq!(explosion_knockback_resistance(&state), 0.0);
    equip(&mut state, 6, "minecraft:iron_chestplate", 2);
    assert!((explosion_knockback_resistance(&state) - 0.30).abs() < 1.0E-9);
    equip(&mut state, 7, "minecraft:iron_leggings", 4);
    assert!((explosion_knockback_resistance(&state) - 0.90).abs() < 1.0E-9);
    equip(&mut state, 8, "minecraft:iron_boots", 1);
    assert_eq!(explosion_knockback_resistance(&state), 1.0, "clamped to the attribute range");
}

#[test]
fn other_protection_enchantments_do_not_resist_explosion_knockback() {
    let mut state = survival_player();
    let mut helmet = ItemStack::new("minecraft:iron_helmet", 1);
    helmet.set_component(ItemComponent::Enchantments(BTreeMap::from([(
        "minecraft:protection".to_string(),
        4,
    )])));
    assert!(state.inventory_menu.set_slot(5, helmet));
    assert_eq!(explosion_knockback_resistance(&state), 0.0);
}

#[test]
fn explosion_damage_is_cut_by_blast_protection() {
    let source = crate::damage_type::explosion_source(
        Some(crate::damage_type::DamageEntityRef::non_living(500)),
        None,
    );
    let mut bare = survival_player();
    assert!(hurt_server(&mut bare, source, 10.0, HurtOrigin::default()));
    let mut protected = survival_player();
    equip(&mut protected, 6, "minecraft:leather_chestplate", 4);
    assert!(hurt_server(&mut protected, source, 10.0, HurtOrigin::default()));
    assert!(protected.health > bare.health, "{} vs {}", protected.health, bare.health);
}

#[test]
fn explosions_caused_by_a_player_need_pvp() {
    // ServerPlayer.hurtServer: `source.getEntity() instanceof Player` requires canHarmPlayer.
    let direct = Some(crate::damage_type::DamageEntityRef::non_living(500));
    let by_player = crate::damage_type::explosion_source(
        direct,
        Some(crate::damage_type::DamageEntityRef::player(2, false)),
    );
    let anonymous = crate::damage_type::explosion_source(direct, None);
    let mut state = survival_player();
    state.combat.hurt.rules.pvp = false;
    assert!(!hurt_server(&mut state, by_player, 6.0, HurtOrigin::default()));
    assert_eq!(state.health, 20.0);
    assert!(hurt_server(&mut state, anonymous, 6.0, HurtOrigin::default()), "no responsible player");
    state.combat.hurt.rules.pvp = true;
    state.combat.hurt.invulnerable_time = 0;
    assert!(hurt_server(&mut state, by_player, 20.0, HurtOrigin::default()));
}
