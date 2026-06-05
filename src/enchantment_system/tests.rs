use super::*;

#[test]
fn enchantment_registry_and_effect_component_surface_match_decompiled_keys() {
    assert_eq!(ENCHANTMENTS.len(), 43);
    assert_eq!(ENCHANTMENTS[0].id, "minecraft:protection");
    assert_eq!(ENCHANTMENTS.last().unwrap().id, "minecraft:vanishing_curse");
    assert_eq!(ENCHANTMENT_EFFECT_COMPONENTS.len(), 31);
    assert!(ENCHANTMENT_EFFECT_COMPONENTS.contains(&EnchantmentEffectHook::DamageProtection));
    assert!(ENCHANTMENT_EFFECT_COMPONENTS.contains(&EnchantmentEffectHook::RepairWithXp));
    assert_eq!(PROVIDER_TYPES.len(), 3);
    assert_eq!(VANILLA_PROVIDERS.len(), 7);
}

#[test]
fn costs_slots_weights_and_compatibility_follow_enchantment_definition_contracts() {
    let protection = enchantment("protection").unwrap();
    assert_eq!(protection.weight, 10);
    assert_eq!(protection.max_level, 4);
    assert_eq!(protection.min_cost.calculate(3), 23);
    assert_eq!(protection.max_cost.calculate(3), 34);

    let fire = enchantment("fire_protection").unwrap();
    let blast = enchantment("blast_protection").unwrap();
    assert!(!are_compatible(fire, blast));
    assert!(are_compatible(fire, enchantment("thorns").unwrap()));
    assert!(!are_compatible(
        enchantment("sharpness").unwrap(),
        enchantment("smite").unwrap()
    ));
    assert!(!are_compatible(
        enchantment("silk_touch").unwrap(),
        enchantment("fortune").unwrap()
    ));
}

#[test]
fn representative_damage_mining_movement_loot_and_post_attack_hooks_are_modeled() {
    assert_eq!(damage_bonus("minecraft:sharpness", 5, None), 3.0);
    assert_eq!(damage_bonus("minecraft:smite", 2, Some("undead")), 5.0);
    assert_eq!(damage_bonus("minecraft:impaling", 3, Some("aquatic")), 7.5);
    assert_eq!(
        protection_bonus("minecraft:feather_falling", 4, "is_fall"),
        12.0
    );
    assert_eq!(
        protection_bonus("minecraft:projectile_protection", 4, "is_projectile"),
        8.0
    );

    assert!(enchantment("efficiency")
        .unwrap()
        .hooks
        .contains(&EnchantmentEffectHook::Attributes));
    assert!(enchantment("fortune")
        .unwrap()
        .hooks
        .contains(&EnchantmentEffectHook::BlockExperience));
    assert!(enchantment("depth_strider")
        .unwrap()
        .hooks
        .contains(&EnchantmentEffectHook::LocationChanged));
    assert!(enchantment("thorns")
        .unwrap()
        .hooks
        .contains(&EnchantmentEffectHook::PostAttack));
    assert!(enchantment("unbreaking")
        .unwrap()
        .hooks
        .contains(&EnchantmentEffectHook::ItemDamage));
}

#[test]
fn protection_damage_reduction_caps_at_80_percent() {
    // Prot I = 4% reduction
    let dmg = protection_damage_reduction(1, 10.0);
    assert!((dmg - 9.6).abs() < 0.01);

    // Prot IV full armor (max per piece is 20, total realistic max 80%)
    let dmg = protection_damage_reduction(20, 10.0);
    assert!((dmg - 2.0).abs() < 0.01);

    // Over cap (100 total) clamped to 80%
    let dmg = protection_damage_reduction(25, 10.0);
    assert!((dmg - 2.0).abs() < 0.01);
}

#[test]
fn sharpness_smite_bane_and_knockback_bonuses_match_vanilla_formulas() {
    // Sharpness I: 0.5*1 + 0.5 = 1.0
    assert!((sharpness_bonus(1) - 1.0).abs() < 0.001);
    // Sharpness V: 0.5*5 + 0.5 = 3.0
    assert!((sharpness_bonus(5) - 3.0).abs() < 0.001);
    assert_eq!(sharpness_bonus(0), 0.0);

    // Smite V on undead: 5 * 2.5 = 12.5
    assert!((smite_bonus(5, true) - 12.5).abs() < 0.001);
    // Smite V on non-undead: 0
    assert_eq!(smite_bonus(5, false), 0.0);

    // Bane IV on arthropod: 4 * 2.5 = 10.0
    assert!((bane_of_arthropods_bonus(4, true) - 10.0).abs() < 0.001);

    // Knockback II: 2 * 3 = 6 blocks
    assert!((knockback_bonus_blocks(2) - 6.0).abs() < 0.001);
    assert!((punch_knockback_bonus_blocks(2) - 6.0).abs() < 0.001);
}

#[test]
fn efficiency_speed_bonus_matches_vanilla_formula() {
    // Efficiency I: 1*1+1 = 2.0 added speed
    assert!((efficiency_speed_bonus(1) - 2.0).abs() < 0.001);
    // Efficiency V: 5*5+1 = 26.0
    assert!((efficiency_speed_bonus(5) - 26.0).abs() < 0.001);
    assert_eq!(efficiency_speed_bonus(0), 0.0);
}

#[test]
fn sweeping_edge_ratio_matches_vanilla_formula() {
    // Level 1: 1/(1+1) = 0.5
    assert!((sweeping_edge_ratio(1) - 0.5).abs() < 0.001);
    // Level 3: 3/(3+1) = 0.75
    assert!((sweeping_edge_ratio(3) - 0.75).abs() < 0.001);
}

#[test]
fn feather_falling_reduces_fall_damage_per_level() {
    // Level 4: 12*4=48% reduction
    let dmg = feather_falling_damage_reduction(4, 10.0);
    assert!((dmg - 5.2).abs() < 0.01);
    // Level 1: 12% reduction
    let dmg = feather_falling_damage_reduction(1, 10.0);
    assert!((dmg - 8.8).abs() < 0.01);
}

#[test]
fn mending_power_impaling_depth_strider_follow_vanilla() {
    assert_eq!(mending_repair_from_xp(5), 10);
    assert!((power_arrow_bonus(1, 6.0) - 6.0).abs() < 0.001); // 6 * (0.5+0.5) = 6
    assert!((impaling_bonus(3, true) - 7.5).abs() < 0.001); // 3*2.5 = 7.5
    assert_eq!(impaling_bonus(3, false), 0.0);
    assert!((depth_strider_speed_factor(3) - 1.0).abs() < 0.001);
    assert!((depth_strider_speed_factor(1) - 0.333).abs() < 0.01);
    assert!(aqua_affinity_removes_underwater_penalty(1));
    assert!(!aqua_affinity_removes_underwater_penalty(0));
    assert_eq!(respiration_bonus_ticks(3), 900);
}

#[test]
fn thorns_and_fortune_and_infinity_match_vanilla_behavior() {
    assert!((thorns_activation_chance(1) - 0.15).abs() < 0.001);
    assert!((thorns_activation_chance(4) - 0.60).abs() < 0.001);
    assert_eq!(fortune_extra_drops(3, 0.99), 3); // 0.99 * 4 = 3
    assert_eq!(fortune_extra_drops(3, 0.0), 0);
    assert!(infinity_prevents_consumption(true, true));
    assert!(!infinity_prevents_consumption(false, true));
    assert!(!infinity_prevents_consumption(true, false));
    assert!(curse_of_vanishing_destroys_on_death(true));
    assert!(!curse_of_vanishing_destroys_on_death(false));
}

#[test]
fn fire_aspect_frost_walker_soul_speed_swift_sneak_match_vanilla() {
    // FireAspect II = 8 seconds on fire
    assert_eq!(fire_aspect_seconds_on_fire(2), 8);
    assert_eq!(fire_aspect_seconds_on_fire(1), 4);
    assert_eq!(fire_aspect_seconds_on_fire(0), 0);
    assert_eq!(flame_seconds_on_fire(1), 5);
    assert_eq!(flame_seconds_on_fire(0), 0);
    // FrostWalker II: radius 4 blocks
    assert_eq!(frost_walker_radius(2), 4);
    assert_eq!(frost_walker_radius(1), 3);
    // SoulSpeed III: 0.09 attribute bonus
    assert!((soul_speed_attribute_bonus(3) - 0.09).abs() < 0.001);
    // SwiftSneak III: 0.45 speed modifier
    assert!((swift_sneak_speed_modifier(3) - 0.45).abs() < 0.001);
}

#[test]
fn channeling_riptide_multishot_quickcharge_binding_breach_wind_burst_match_vanilla() {
    // Channeling only fires in thunderstorm with open sky
    assert!(channeling_can_strike(1, true, true));
    assert!(!channeling_can_strike(1, false, true));
    assert!(!channeling_can_strike(1, true, false));
    assert!(!channeling_can_strike(0, true, true));
    // Riptide I: power = 0.9
    assert!((riptide_thrust_power(1) - 0.9).abs() < 0.001);
    // Riptide III: power = 1.5
    assert!((riptide_thrust_power(3) - 1.5).abs() < 0.001);
    // MultiShot always gives 2 extra (3 total)
    assert_eq!(multishot_extra_projectiles(), 2);
    // Piercing III can pass through four entities including the first target.
    assert_eq!(piercing_entity_limit(3), 4);
    assert_eq!(piercing_entity_limit(0), 1);
    // QuickCharge III reduces 15 ticks
    assert_eq!(quick_charge_use_ticks_reduction(3), 15);
    // BindingCurse: survival cannot remove, creative can
    assert!(!binding_curse_can_remove(true, false));
    assert!(binding_curse_can_remove(true, true));
    assert!(binding_curse_can_remove(false, false));
    // Breach IV: 60% armor reduction
    assert!((breach_armor_reduction_fraction(4) - 0.6).abs() < 0.001);
    // WindBurst III: 3.0 blocks knockback
    assert!((wind_burst_knockback_blocks(3) - 3.0).abs() < 0.001);
}

#[test]
fn loyalty_enables_return_at_any_level() {
    assert!(loyalty_enables_return(1));
    assert!(loyalty_enables_return(3));
    assert!(!loyalty_enables_return(0));
}

#[test]
fn providers_cover_spawn_raid_and_loot_selection_paths() {
    assert_eq!(
        VANILLA_PROVIDERS[0].provider_type,
        ProviderType::ByCostWithDifficulty
    );
    assert_eq!(VANILLA_PROVIDERS[0].min_cost, 5);
    assert_eq!(VANILLA_PROVIDERS[0].max_cost, 17);
    assert_eq!(VANILLA_PROVIDERS[1].enchantment, Some("minecraft:piercing"));
    assert_eq!(
        VANILLA_PROVIDERS[6].enchantment,
        Some("minecraft:silk_touch")
    );
}

#[test]
fn can_enchant_matches_26_1_2_supported_items() {
    use super::can_enchant;
    // Sharpness (enchantable/sharp_weapon = swords + spears + axes) — NOT pickaxes/mace.
    assert!(can_enchant("minecraft:diamond_sword", "minecraft:sharpness"));
    assert!(can_enchant("minecraft:iron_axe", "minecraft:sharpness"));
    assert!(can_enchant("minecraft:netherite_spear", "minecraft:sharpness"));
    assert!(!can_enchant("minecraft:diamond_pickaxe", "minecraft:sharpness"));
    assert!(!can_enchant("minecraft:mace", "minecraft:sharpness"));

    // Knockback (enchantable/melee_weapon = swords + spears) — NOT axes.
    assert!(can_enchant("minecraft:diamond_sword", "minecraft:knockback"));
    assert!(!can_enchant("minecraft:diamond_axe", "minecraft:knockback"));

    // Smite (enchantable/weapon = sharp_weapon + mace) — includes mace + axes.
    assert!(can_enchant("minecraft:mace", "minecraft:smite"));
    assert!(can_enchant("minecraft:diamond_axe", "minecraft:smite"));

    // Fortune (enchantable/mining_loot = axes/pickaxes/shovels/hoes) — NOT shears.
    assert!(can_enchant("minecraft:diamond_pickaxe", "minecraft:fortune"));
    assert!(!can_enchant("minecraft:shears", "minecraft:fortune"));
    // Efficiency (enchantable/mining) DOES include shears.
    assert!(can_enchant("minecraft:shears", "minecraft:efficiency"));

    // Protection (enchantable/armor) — any armour piece, not weapons.
    assert!(can_enchant("minecraft:diamond_chestplate", "minecraft:protection"));
    assert!(can_enchant("minecraft:turtle_helmet", "minecraft:protection"));
    assert!(!can_enchant("minecraft:diamond_sword", "minecraft:protection"));

    // Unbreaking (enchantable/durability) — broad: tools, armour, elytra, etc.
    assert!(can_enchant("minecraft:elytra", "minecraft:unbreaking"));
    assert!(can_enchant("minecraft:fishing_rod", "minecraft:unbreaking"));

    // Unknown enchantment id -> false.
    assert!(!can_enchant("minecraft:diamond_sword", "minecraft:nonexistent"));
}

#[test]
fn is_primary_item_respects_primary_vs_supported() {
    use super::is_primary_item;
    // Sharpness: primary = melee_weapon (swords/spears), even though it is SUPPORTED on
    // axes (anvil) — the enchanting table only offers it on the primary items.
    assert!(is_primary_item("minecraft:diamond_sword", "minecraft:sharpness"));
    assert!(is_primary_item("minecraft:netherite_spear", "minecraft:sharpness"));
    assert!(!is_primary_item("minecraft:iron_axe", "minecraft:sharpness"));
    // Thorns: primary = chest_armor (not other armour slots).
    assert!(is_primary_item("minecraft:diamond_chestplate", "minecraft:thorns"));
    assert!(!is_primary_item("minecraft:diamond_helmet", "minecraft:thorns"));
    // Protection: no primary override -> primary == supported (any armour).
    assert!(is_primary_item("minecraft:diamond_helmet", "minecraft:protection"));
    assert!(!is_primary_item("minecraft:diamond_sword", "minecraft:protection"));
}

#[test]
fn get_enchantment_cost_matches_java_formula() {
    use super::get_enchantment_cost;
    use crate::random_source::LegacyRandom;

    // Zero enchantability -> no cost.
    let mut r = LegacyRandom::new(1);
    assert_eq!(get_enchantment_cost(&mut r, 0, 15, 0), 0);

    // Known property: with 15 bookcases the top slot is always 30 or 31
    // (selected = nextInt(8)+1 + 7 + nextInt(16) in [9,31]; slot 2 = max(selected, 30)),
    // and the first slot is always >= 1. Verify across many seeds.
    for seed in 0..200i64 {
        let mut rng = LegacyRandom::new(seed);
        let c0 = get_enchantment_cost(&mut rng, 0, 15, 15);
        let mut rng = LegacyRandom::new(seed);
        let c2 = get_enchantment_cost(&mut rng, 2, 15, 15);
        assert!(c0 >= 1, "slot0 >= 1");
        assert!((30..=31).contains(&c2), "slot2 in 30..=31, got {c2}");
    }

    // Bookcases are capped at 15 (no panic / same range for higher values).
    let mut rng = LegacyRandom::new(7);
    let c = get_enchantment_cost(&mut rng, 2, 100, 15);
    assert!((30..=31).contains(&c));
}
