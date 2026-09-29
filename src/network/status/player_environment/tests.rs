// `PlaySessionState` has ~60 fields; tests set only the few they care about.
#![allow(clippy::field_reassign_with_default)]

use std::collections::BTreeMap;

use super::*;

/// Player standing on the block at y = 63, feet in the cell y = 64.
fn standing_player() -> PlaySessionState {
    let mut state = PlaySessionState::default();
    state.x = 0.5;
    state.y = 64.0;
    state.z = 0.5;
    state.on_ground = true;
    state
}

/// Runs one environment tick over a world given as `(x, y, z) -> state`, air elsewhere.
fn tick_in(state: &mut PlaySessionState, world: &[((i32, i32, i32), &str)]) -> bool {
    let map: BTreeMap<_, _> = world.iter().map(|(pos, name)| (*pos, name.to_string())).collect();
    tick_player_environment(state, -64, &mut |x, y, z| map.get(&(x, y, z)).cloned())
}

fn floor_block() -> ((i32, i32, i32), &'static str) {
    ((0, 63, 0), "minecraft:stone")
}

fn add_effect(state: &mut PlaySessionState, id: &str) {
    state.active_effects.push(Tag::Compound(vec![
        ("id".to_string(), Tag::String(id.to_string())),
        ("amplifier".to_string(), Tag::Byte(0)),
    ]));
}

fn cooled_down(state: &mut PlaySessionState) {
    state.combat.hurt.invulnerable_time = 0;
}

#[test]
fn falling_below_the_world_hurts_four_and_bypasses_creative_invulnerability() {
    let mut state = standing_player();
    state.y = -129.0;
    assert!(tick_in(&mut state, &[]));
    assert_eq!(state.health, 16.0);
    // y == minY - 64 is still inside the world (strict less-than).
    let mut edge = standing_player();
    edge.y = -128.0;
    assert!(!tick_in(&mut edge, &[]));
    let mut creative = standing_player();
    creative.y = -200.0;
    creative.abilities.invulnerable = true;
    tick_in(&mut creative, &[]);
    assert_eq!(creative.health, 16.0);
}

#[test]
fn void_damage_repeats_once_the_hurt_cooldown_allows_it() {
    let mut state = standing_player();
    state.y = -129.0;
    tick_in(&mut state, &[]);
    tick_in(&mut state, &[]);
    assert_eq!(state.health, 16.0, "equal damage inside the cooldown is rejected");
    state.combat.hurt.invulnerable_time = 10;
    tick_in(&mut state, &[]);
    assert_eq!(state.health, 12.0);
}

#[test]
fn standing_in_lava_burns_for_fifteen_seconds_and_hurts_four() {
    let mut state = standing_player();
    let world = [floor_block(), ((0, 64, 0), "minecraft:lava[level=0]")];
    assert!(tick_in(&mut state, &world));
    assert_eq!(state.health, 16.0);
    assert_eq!(state.combat.environment.remaining_fire_ticks, 300);
    // In lava the periodic on_fire damage is skipped and fall distance halves.
    state.fall_distance = 8.0;
    cooled_down(&mut state);
    tick_in(&mut state, &world);
    assert_eq!(state.fall_distance, 4.0);
    assert_eq!(state.health, 12.0, "only lavaHurt this tick");
}

#[test]
fn fire_resistance_makes_lava_harmless_but_the_player_still_ignites() {
    let mut state = standing_player();
    add_effect(&mut state, "minecraft:fire_resistance");
    tick_in(&mut state, &[floor_block(), ((0, 64, 0), "minecraft:lava[level=0]")]);
    assert_eq!(state.health, 20.0);
}

#[test]
fn burning_deals_one_damage_every_twenty_ticks_and_counts_down() {
    let mut state = standing_player();
    state.combat.environment.remaining_fire_ticks = 41;
    let world = [floor_block()];
    tick_in(&mut state, &world); // 41 -> 40, no damage at 41
    assert_eq!((state.health, state.combat.environment.remaining_fire_ticks), (20.0, 40));
    tick_in(&mut state, &world); // 40 % 20 == 0 -> damage
    assert_eq!((state.health, state.combat.environment.remaining_fire_ticks), (19.0, 39));
    cooled_down(&mut state);
    state.combat.environment.remaining_fire_ticks = 20;
    tick_in(&mut state, &world);
    assert_eq!(state.health, 18.0);
    assert_eq!(state.combat.environment.remaining_fire_ticks, 19);
}

#[test]
fn fire_extinguished_by_water_and_burning_ignores_the_fire_damage_rule() {
    let mut state = standing_player();
    state.combat.environment.remaining_fire_ticks = 100;
    tick_in(&mut state, &[floor_block(), ((0, 64, 0), "minecraft:water[level=0]")]);
    assert!(state.combat.environment.remaining_fire_ticks <= 0);

    let mut safe = standing_player();
    safe.combat.hurt.rules.fire_damage = false;
    safe.combat.environment.remaining_fire_ticks = 40;
    tick_in(&mut safe, &[floor_block()]);
    assert_eq!(safe.health, 20.0);
}

#[test]
fn player_grace_period_after_fire_is_twenty_negative_ticks() {
    let mut state = standing_player();
    tick_in(&mut state, &[floor_block()]);
    assert_eq!(state.combat.environment.remaining_fire_ticks, -20);
}

#[test]
fn fire_blocks_ignite_for_eight_seconds_and_deal_fire_damage() {
    let mut fire = standing_player();
    tick_in(&mut fire, &[floor_block(), ((0, 64, 0), "minecraft:fire")]);
    assert_eq!(fire.health, 19.0);
    assert!(fire.combat.environment.remaining_fire_ticks >= 160 - 1);
    let mut soul = standing_player();
    tick_in(&mut soul, &[floor_block(), ((0, 64, 0), "minecraft:soul_fire")]);
    assert_eq!(soul.health, 18.0);
}

#[test]
fn burning_time_shrinks_with_fire_protection() {
    let mut state = standing_player();
    let mut boots = ItemStack::new("minecraft:iron_boots", 1);
    boots.set_component(ItemComponent::Enchantments(BTreeMap::from([(
        "minecraft:fire_protection".to_string(),
        4,
    )])));
    state.inventory_menu.set_slot(8, boots);
    // 1 - 0.15 * 4 = 0.4 -> ceil(160 * 0.4) = 64.
    ignite_for_ticks(&mut state, FIRE_IGNITE_TICKS);
    assert_eq!(state.combat.environment.remaining_fire_ticks, 64);
}

#[test]
fn head_inside_a_solid_block_suffocates() {
    let mut state = standing_player();
    let head = ((0, 65, 0), "minecraft:stone");
    assert!(tick_in(&mut state, &[floor_block(), head]));
    assert_eq!(state.health, 19.0);
    let mut spectator = standing_player();
    spectator.game_mode = GameMode::Spectator;
    tick_in(&mut spectator, &[floor_block(), head]);
    assert_eq!(spectator.health, 20.0);
    // Glass does not suffocate.
    let mut glass = standing_player();
    tick_in(&mut glass, &[floor_block(), ((0, 65, 0), "minecraft:glass")]);
    assert_eq!(glass.health, 20.0);
}

#[test]
fn cactus_and_lit_campfires_hurt_on_contact() {
    let mut cactus = standing_player();
    tick_in(&mut cactus, &[floor_block(), ((0, 64, 0), "minecraft:cactus[age=0]")]);
    assert_eq!(cactus.health, 19.0);
    let mut lit = standing_player();
    tick_in(&mut lit, &[floor_block(), ((0, 64, 0), "minecraft:campfire[lit=true]")]);
    assert_eq!(lit.health, 19.0);
    let mut soul = standing_player();
    tick_in(&mut soul, &[floor_block(), ((0, 64, 0), "minecraft:soul_campfire[lit=true]")]);
    assert_eq!(soul.health, 18.0);
    let mut unlit = standing_player();
    tick_in(&mut unlit, &[floor_block(), ((0, 64, 0), "minecraft:campfire[lit=false]")]);
    assert_eq!(unlit.health, 20.0);
}

#[test]
fn sweet_berry_bushes_hurt_only_when_grown_and_moving() {
    let bush = |age: &'static str| [floor_block(), ((0, 64, 0), age)];
    let mut still = standing_player();
    still.fall_distance = 6.0;
    tick_in(&mut still, &bush("minecraft:sweet_berry_bush[age=3]"));
    tick_in(&mut still, &bush("minecraft:sweet_berry_bush[age=3]"));
    assert_eq!(still.health, 20.0, "no movement, no damage");
    assert_eq!(still.fall_distance, 0.0, "makeStuckInBlock resets fall distance");

    let mut moving = standing_player();
    tick_in(&mut moving, &bush("minecraft:sweet_berry_bush[age=3]"));
    moving.x += 0.05;
    tick_in(&mut moving, &bush("minecraft:sweet_berry_bush[age=3]"));
    assert_eq!(moving.health, 19.0);

    let mut sapling = standing_player();
    tick_in(&mut sapling, &bush("minecraft:sweet_berry_bush[age=0]"));
    sapling.x += 0.05;
    tick_in(&mut sapling, &bush("minecraft:sweet_berry_bush[age=0]"));
    assert_eq!(sapling.health, 20.0);
}

#[test]
fn magma_blocks_burn_feet_unless_sneaking() {
    let magma = [((0, 63, 0), "minecraft:magma_block")];
    let mut state = standing_player();
    tick_in(&mut state, &magma);
    assert_eq!(state.health, 19.0);
    let mut sneaking = standing_player();
    sneaking.input_shift = true;
    tick_in(&mut sneaking, &magma);
    assert_eq!(sneaking.health, 20.0);
    let mut airborne = standing_player();
    airborne.on_ground = false;
    tick_in(&mut airborne, &magma);
    assert_eq!(airborne.health, 20.0);
}

#[test]
fn powder_snow_freezes_then_damages_every_forty_ticks() {
    let mut state = standing_player();
    let world = [floor_block(), ((0, 64, 0), "minecraft:powder_snow")];
    for tick in 1..=139 {
        state.combat.tick_count = tick;
        tick_in(&mut state, &world);
    }
    assert_eq!(state.combat.environment.ticks_frozen, 139);
    assert_eq!(state.health, 20.0);
    state.combat.tick_count = 140;
    tick_in(&mut state, &world);
    assert_eq!(state.combat.environment.ticks_frozen, 140);
    assert_eq!(state.health, 20.0, "140 % 40 != 0");
    state.combat.tick_count = 160;
    tick_in(&mut state, &world);
    assert_eq!(state.health, 19.0);
    // Leaving the snow thaws two ticks per tick.
    tick_in(&mut state, &[floor_block()]);
    assert_eq!(state.combat.environment.ticks_frozen, 138);
}

#[test]
fn leather_armor_and_spectators_do_not_freeze() {
    let mut state = standing_player();
    state.inventory_menu.set_slot(8, ItemStack::new("minecraft:leather_boots", 1));
    tick_in(&mut state, &[floor_block(), ((0, 64, 0), "minecraft:powder_snow")]);
    assert_eq!(state.combat.environment.ticks_frozen, 0);
}

#[test]
fn freeze_damage_rule_off_prevents_freeze_damage() {
    let mut state = standing_player();
    state.combat.hurt.rules.freeze_damage = false;
    state.combat.environment.ticks_frozen = 140;
    state.combat.tick_count = 40;
    tick_in(&mut state, &[floor_block(), ((0, 64, 0), "minecraft:powder_snow")]);
    assert_eq!(state.health, 20.0);
}

fn land(state: &mut PlaySessionState, distance: f32, below: &str) -> f32 {
    state.combat.hurt.pending_landing = Some(distance);
    tick_in(state, &[((0, 63, 0), below)]);
    20.0 - state.health
}

#[test]
fn landing_damage_depends_on_the_block_fallen_on() {
    // floor((10 + 1e-6 - 3) * modifier).
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:stone"), 7.0);
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:hay_block"), 1.0);
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:honey_block"), 1.0);
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:slime_block"), 0.0);
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:powder_snow"), 0.0);
    // BedBlock halves the distance: (5 - 3) = 2.
    assert_eq!(land(&mut standing_player(), 10.0, "minecraft:red_bed[facing=north,occupied=false,part=foot]"), 2.0);
    // Dripstone tip: distance + 2.5 = 12.5 - 3 = 9.5, doubled and floored.
    assert_eq!(
        land(&mut standing_player(), 10.0, "minecraft:pointed_dripstone[thickness=tip,vertical_direction=up,waterlogged=false]"),
        19.0
    );
    // A sneaking player is not protected by slime in 26.1.2, it just takes nothing extra.
    let mut sneaking = standing_player();
    sneaking.input_shift = true;
    assert_eq!(land(&mut sneaking, 10.0, "minecraft:slime_block"), 0.0);
}

#[test]
fn safe_distances_mayfly_and_the_fall_rule_take_no_damage() {
    assert_eq!(land(&mut standing_player(), 3.0, "minecraft:stone"), 0.0);
    let mut flyer = standing_player();
    flyer.abilities.mayfly = true;
    assert_eq!(land(&mut flyer, 30.0, "minecraft:stone"), 0.0);
    let mut rule_off = standing_player();
    rule_off.combat.hurt.rules.fall_damage = false;
    assert_eq!(land(&mut rule_off, 30.0, "minecraft:stone"), 0.0);
}

#[test]
fn honey_landing_queues_the_slide_entity_event() {
    let mut state = standing_player();
    land(&mut state, 10.0, "minecraft:honey_block");
    assert!(state
        .combat
        .hurt
        .pending_packets
        .contains(&super::super::player_damage::PendingHurtPacket::EntityEvent(54)));
}

#[test]
fn movement_records_the_landing_instead_of_damaging_immediately() {
    let mut state = standing_player();
    state.fall_distance = 10.0;
    super::super::apply_player_movement(&mut state, 0.0, 0.0, 0.0, true);
    assert_eq!(state.fall_distance, 0.0);
    assert_eq!(state.combat.hurt.pending_landing, Some(10.0));
    assert_eq!(state.health, 20.0);
}

#[test]
fn cobwebs_reset_the_fall_distance() {
    let mut state = standing_player();
    state.fall_distance = 12.0;
    tick_in(&mut state, &[floor_block(), ((0, 64, 0), "minecraft:cobweb")]);
    assert_eq!(state.fall_distance, 0.0);
}

#[test]
fn air_supply_helpers_follow_effects_and_respiration() {
    let mut state = standing_player();
    state.air_supply = 100;
    assert_eq!(decrease_air_supply(&mut state), 99);
    assert!(!has_water_breathing(&state));
    assert!(effects_refill_air_supply(&state));
    add_effect(&mut state, "minecraft:water_breathing");
    assert!(has_water_breathing(&state));
    let mut helmet = ItemStack::new("minecraft:iron_helmet", 1);
    helmet.set_component(ItemComponent::Enchantments(BTreeMap::from([(
        "minecraft:respiration".to_string(),
        3,
    )])));
    let mut diver = standing_player();
    diver.air_supply = 100;
    diver.inventory_menu.set_slot(5, helmet);
    // bonus 3: a decrement is skipped with probability 3/4, so over many draws it averages 25%.
    let lost = (0..400).filter(|_| decrease_air_supply(&mut diver) < 100).count();
    assert!((60..=140).contains(&lost), "{lost}");
}
