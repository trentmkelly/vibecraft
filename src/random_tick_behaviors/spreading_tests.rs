//! Tests for grass/mycelium spread and decay, nylium and sapling ticks.

use super::test_world::{block, p, TestWorld};
use super::tests::{assert_same_state, replay, seed_with};
use super::*;
use crate::random_source::LegacyRandom;

fn tick(world: &mut TestWorld, pos: BlockPos, seed: i64) -> LegacyRandom {
    let mut random = LegacyRandom::new(seed);
    let state = world.at(pos);
    random_tick(world, &state, pos, &mut random);
    random
}

/// A seed whose first spread attempt targets `pos + (dx, dy, dz)`: the draws
/// are `nextInt(3) - 1`, `nextInt(5) - 3`, `nextInt(3) - 1`.
fn spread_seed(dx: i32, dy: i32, dz: i32) -> i64 {
    seed_with(|r| {
        r.next_i32_bound(3) - 1 == dx
            && r.next_i32_bound(5) - 3 == dy
            && r.next_i32_bound(3) - 1 == dz
    })
}

/// Consumes the twelve draws of the four spread attempts.
fn four_spread_attempts(random: &mut LegacyRandom) {
    for _ in 0..4 {
        random.next_i32_bound(3);
        random.next_i32_bound(5);
        random.next_i32_bound(3);
    }
}

fn snow_layers(layers: i32) -> BlockStateModel {
    with_int(&block("minecraft:snow"), "layers", layers)
}

#[test]
fn grass_spreads_onto_lit_dirt_with_a_free_top() {
    let seed = spread_seed(1, 0, 0);
    let mut world = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(1, 64, 0), block("minecraft:dirt"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    let spread = world.at(p(1, 64, 0));
    assert_eq!(spread.registry_id, "minecraft:grass_block");
    assert_eq!(spread.property("snowy"), Some("false"));
    assert_eq!(world.writes[0].2, UPDATE_ALL);
    assert_same_state(random, replay(seed, four_spread_attempts));
}

#[test]
fn spread_dirt_under_snow_becomes_snowy_grass() {
    let seed = spread_seed(1, 0, 0);
    let mut world = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:mycelium"))
        .with(p(1, 64, 0), block("minecraft:dirt"))
        .with(p(1, 65, 0), snow_layers(1));
    tick(&mut world, p(0, 64, 0), seed);
    let spread = world.at(p(1, 64, 0));
    assert_eq!(spread.registry_id, "minecraft:mycelium");
    assert_eq!(spread.property("snowy"), Some("true"));
}

#[test]
fn grass_does_not_spread_onto_covered_or_submerged_dirt() {
    let seed = spread_seed(1, 0, 0);
    let mut covered = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(1, 64, 0), block("minecraft:dirt"))
        .with(p(1, 65, 0), block("minecraft:stone"));
    tick(&mut covered, p(0, 64, 0), seed);
    assert!(covered.writes.is_empty());

    let mut thick_snow = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(1, 64, 0), block("minecraft:dirt"))
        .with(p(1, 65, 0), snow_layers(8));
    tick(&mut thick_snow, p(0, 64, 0), seed);
    assert!(thick_snow.writes.is_empty());

    let mut submerged = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(1, 64, 0), block("minecraft:dirt"))
        .with(p(1, 65, 0), block("minecraft:water"));
    tick(&mut submerged, p(0, 64, 0), seed);
    assert!(submerged.writes.is_empty());
}

#[test]
fn grass_needs_light_nine_above_to_spread_but_not_to_survive() {
    let seed = spread_seed(1, 0, 0);
    let mut dark = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(1, 64, 0), block("minecraft:dirt"));
    dark.light = 8;
    let random = tick(&mut dark, p(0, 64, 0), seed);
    assert!(dark.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}

#[test]
fn smothered_grass_and_mycelium_revert_to_dirt() {
    for id in ["minecraft:grass_block", "minecraft:mycelium"] {
        let mut world = TestWorld::default()
            .with(p(0, 64, 0), block(id))
            .with(p(0, 65, 0), block("minecraft:stone"));
        let random = tick(&mut world, p(0, 64, 0), 3);
        assert_eq!(world.at(p(0, 64, 0)).registry_id, "minecraft:dirt");
        assert_eq!(world.writes[0].2, UPDATE_ALL);
        assert_same_state(random, LegacyRandom::new(3));
    }
    // A single snow layer, or transparent plants, keep the block alive.
    let mut snowed = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(0, 65, 0), snow_layers(1));
    tick(&mut snowed, p(0, 64, 0), 3);
    assert_eq!(snowed.at(p(0, 64, 0)).registry_id, "minecraft:grass_block");
    // A full-strength water column above kills it.
    let mut drowned = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:grass_block"))
        .with(p(0, 65, 0), block("minecraft:water"));
    tick(&mut drowned, p(0, 64, 0), 3);
    assert_eq!(drowned.at(p(0, 64, 0)).registry_id, "minecraft:dirt");
}

#[test]
fn covered_nylium_becomes_netherrack() {
    for id in ["minecraft:crimson_nylium", "minecraft:warped_nylium"] {
        let mut covered = TestWorld::default()
            .with(p(0, 64, 0), block(id))
            .with(p(0, 65, 0), block("minecraft:stone"));
        tick(&mut covered, p(0, 64, 0), 1);
        assert_eq!(covered.at(p(0, 64, 0)).registry_id, "minecraft:netherrack");
        assert_eq!(covered.writes[0].2, UPDATE_ALL);

        let mut open = TestWorld::default().with(p(0, 64, 0), block(id));
        tick(&mut open, p(0, 64, 0), 1);
        assert!(open.writes.is_empty());
    }
}

#[test]
fn sapling_advances_from_stage_zero_to_one_on_a_one_in_seven_roll() {
    let seed = seed_with(|r| r.next_i32_bound(7) == 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:oak_sapling"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "stage"), 1);
    assert_eq!(world.writes[0].2, UPDATE_QUIET);
    assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(7)));

    // The light gate comes first, so a dark sapling consumes no RNG.
    let mut dark = TestWorld::default().with(p(0, 64, 0), block("minecraft:oak_sapling"));
    dark.light = 8;
    let random = tick(&mut dark, p(0, 64, 0), seed);
    assert!(dark.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));

    let miss = seed_with(|r| r.next_i32_bound(7) != 0);
    let mut idle = TestWorld::default().with(p(0, 64, 0), block("minecraft:oak_sapling"));
    tick(&mut idle, p(0, 64, 0), miss);
    assert!(idle.writes.is_empty());
}

#[test]
fn every_sapling_variant_is_dispatched() {
    let seed = seed_with(|r| r.next_i32_bound(7) == 0);
    for id in [
        "minecraft:spruce_sapling",
        "minecraft:birch_sapling",
        "minecraft:jungle_sapling",
        "minecraft:acacia_sapling",
        "minecraft:cherry_sapling",
        "minecraft:dark_oak_sapling",
        "minecraft:pale_oak_sapling",
    ] {
        let mut world = TestWorld::default().with(p(0, 64, 0), block(id));
        tick(&mut world, p(0, 64, 0), seed);
        assert_eq!(int_property(&world.at(p(0, 64, 0)), "stage"), 1, "{id}");
    }
}
