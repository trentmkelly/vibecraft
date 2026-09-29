//! Tests for the crop, stem, farmland and growth `randomTick` ports.
//!
//! Randomness is checked against a replay of the Java draw sequence: a seed is
//! searched for whose replayed draws select the branch under test, and the RNG
//! state after the tick must equal the replay's, which pins the exact number
//! and order of `RandomSource` calls.

use super::test_world::{block, p, TestWorld};
use super::*;
use crate::random_source::LegacyRandom;

/// The first seed whose fresh generator satisfies `check`.
pub(super) fn seed_with(check: impl Fn(&mut LegacyRandom) -> bool) -> i64 {
    (0..100_000)
        .find(|seed| check(&mut LegacyRandom::new(*seed)))
        .expect("no seed selects the branch")
}

/// A generator advanced exactly as the Java method would have advanced it.
pub(super) fn replay(seed: i64, draws: impl FnOnce(&mut LegacyRandom)) -> LegacyRandom {
    let mut random = LegacyRandom::new(seed);
    draws(&mut random);
    random
}

/// Asserts two generators are in the same state (same next draws).
pub(super) fn assert_same_state(mut actual: LegacyRandom, mut expected: LegacyRandom) {
    assert_eq!(
        actual.next_i64(),
        expected.next_i64(),
        "RNG draw count differs"
    );
}

fn moist_farmland(moisture: i32) -> BlockStateModel {
    with_int(&block("minecraft:farmland"), "moisture", moisture)
}

/// A crop at `(0, 64, 0)` with moisture-`moisture` farmland underneath and
/// around it (growth speed 10 at 7, 4 at 0).
fn crop_field(crop: BlockStateModel, moisture: i32) -> TestWorld {
    TestWorld::default()
        .fill(p(-1, 63, -1), p(1, 63, 1), &moist_farmland(moisture))
        .with(p(0, 64, 0), crop)
}

fn tick(world: &mut TestWorld, pos: BlockPos, seed: i64) -> LegacyRandom {
    let mut random = LegacyRandom::new(seed);
    let state = world.at(pos);
    random_tick(world, &state, pos, &mut random);
    random
}

#[test]
fn growth_speed_counts_farmland_moisture_and_neighbouring_crops() {
    let wheat = block("minecraft:wheat");
    let wet = crop_field(wheat.clone(), 7);
    assert_eq!(
        crops::growth_speed(&wet, "minecraft:wheat", p(0, 64, 0)),
        10.0
    );
    let dry = crop_field(wheat.clone(), 0);
    assert_eq!(
        crops::growth_speed(&dry, "minecraft:wheat", p(0, 64, 0)),
        4.0
    );
    let bare = TestWorld::default().with(p(0, 64, 0), wheat.clone());
    assert_eq!(
        crops::growth_speed(&bare, "minecraft:wheat", p(0, 64, 0)),
        1.0
    );

    // Crops on opposite axes (west + north) halve the speed; a diagonal
    // neighbour alone halves it once, not twice.
    let crowded = crop_field(wheat.clone(), 7)
        .with(p(-1, 64, 0), wheat.clone())
        .with(p(0, 64, -1), wheat.clone());
    assert_eq!(
        crops::growth_speed(&crowded, "minecraft:wheat", p(0, 64, 0)),
        5.0
    );
    let diagonal = crop_field(wheat.clone(), 7).with(p(1, 64, 1), wheat.clone());
    assert_eq!(
        crops::growth_speed(&diagonal, "minecraft:wheat", p(0, 64, 0)),
        5.0
    );
    // A different crop does not count as the same block type.
    let other = crop_field(wheat, 7).with(p(1, 64, 0), block("minecraft:carrots"));
    assert_eq!(
        crops::growth_speed(&other, "minecraft:wheat", p(0, 64, 0)),
        10.0
    );
}

#[test]
fn crop_grows_one_stage_when_the_roll_hits_and_uses_a_single_draw() {
    // growthSpeed 10 -> nextInt((int)(25 / 10) + 1) = nextInt(3).
    let hit = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = crop_field(block("minecraft:wheat"), 7);
    let random = tick(&mut world, p(0, 64, 0), hit);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);
    assert_eq!(
        world.writes,
        vec![(p(0, 64, 0), "minecraft:wheat[age=1]".into(), UPDATE_CLIENTS)]
    );
    assert_same_state(random, replay(hit, |r| _ = r.next_i32_bound(3)));

    let miss = seed_with(|r| r.next_i32_bound(3) != 0);
    let mut world = crop_field(block("minecraft:wheat"), 7);
    let random = tick(&mut world, p(0, 64, 0), miss);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 0);
    assert!(world.writes.is_empty());
    assert_same_state(random, replay(miss, |r| _ = r.next_i32_bound(3)));
}

#[test]
fn dry_farmland_lowers_the_growth_chance() {
    // growthSpeed 4 -> nextInt((int)(25 / 4) + 1) = nextInt(7).
    let seed = seed_with(|r| r.next_i32_bound(7) == 0);
    let mut world = crop_field(block("minecraft:carrots"), 0);
    let random = tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);
    assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(7)));
}

#[test]
fn crop_needs_light_nine_and_is_not_rolled_at_max_age() {
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut dark = crop_field(block("minecraft:wheat"), 7);
    dark.light = 8;
    let random = tick(&mut dark, p(0, 64, 0), seed);
    assert!(dark.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));

    let mut mature = crop_field(with_int(&block("minecraft:wheat"), "age", 7), 7);
    let random = tick(&mut mature, p(0, 64, 0), seed);
    assert!(mature.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}

#[test]
fn beetroot_skips_two_thirds_of_ticks_before_the_crop_roll() {
    // BeetrootBlock.randomTick: `if (random.nextInt(3) != 0) super.randomTick`.
    let skipped = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = crop_field(block("minecraft:beetroots"), 7);
    let random = tick(&mut world, p(0, 64, 0), skipped);
    assert!(world.writes.is_empty());
    assert_same_state(random, replay(skipped, |r| _ = r.next_i32_bound(3)));

    let grows = seed_with(|r| r.next_i32_bound(3) != 0 && r.next_i32_bound(3) == 0);
    let mut world = crop_field(block("minecraft:beetroots"), 7);
    let random = tick(&mut world, p(0, 64, 0), grows);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);
    assert_same_state(
        random,
        replay(grows, |r| {
            r.next_i32_bound(3);
            r.next_i32_bound(3);
        }),
    );
}

#[test]
fn beetroot_matures_at_age_three() {
    let seed = seed_with(|r| r.next_i32_bound(3) != 0 && r.next_i32_bound(3) == 0);
    let full = with_int(&block("minecraft:beetroots"), "age", 3);
    let mut world = crop_field(full, 7);
    tick(&mut world, p(0, 64, 0), seed);
    assert!(world.writes.is_empty());
}

#[test]
fn torchflower_crop_turns_into_the_flower_on_its_last_stage() {
    let seed = seed_with(|r| r.next_i32_bound(3) != 0 && r.next_i32_bound(3) == 0);
    let young = with_int(&block("minecraft:torchflower_crop"), "age", 1);
    let mut world = crop_field(young, 7);
    tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(world.at(p(0, 64, 0)).registry_id, "minecraft:torchflower");

    let mut world = crop_field(block("minecraft:torchflower_crop"), 7);
    tick(&mut world, p(0, 64, 0), seed);
    let grown = world.at(p(0, 64, 0));
    assert_eq!(grown.registry_id, "minecraft:torchflower_crop");
    assert_eq!(int_property(&grown, "age"), 1);
}

#[test]
fn stem_grows_then_sets_fruit_and_attaches() {
    let stem = with_int(&block("minecraft:pumpkin_stem"), "age", 3);
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = crop_field(stem, 7);
    let random = tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 4);
    assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(3)));

    // Age 7: after the growth roll, `Plane.HORIZONTAL.getRandomDirection`
    // draws nextInt(4) over [north, east, south, west].
    let ripe = with_int(&block("minecraft:pumpkin_stem"), "age", 7);
    let seed = seed_with(|r| r.next_i32_bound(3) == 0 && r.next_i32_bound(4) == 1);
    let mut world = crop_field(ripe, 7).fill(p(-2, 63, -2), p(2, 63, 2), &block("minecraft:dirt"));
    for x in -1..=1 {
        for z in -1..=1 {
            world.put(p(x, 63, z), moist_farmland(7));
        }
    }
    let random = tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(world.at(p(1, 64, 0)).registry_id, "minecraft:pumpkin");
    let attached = world.at(p(0, 64, 0));
    assert_eq!(attached.registry_id, "minecraft:attached_pumpkin_stem");
    assert_eq!(attached.property("facing"), Some("east"));
    assert!(world
        .writes
        .iter()
        .all(|(_, _, flags)| *flags == UPDATE_ALL));
    assert_same_state(
        random,
        replay(seed, |r| {
            r.next_i32_bound(3);
            r.next_i32_bound(4);
        }),
    );
}

#[test]
fn stem_does_not_fruit_onto_unsupported_or_occupied_ground() {
    let ripe = with_int(&block("minecraft:melon_stem"), "age", 7);
    let seed = seed_with(|r| r.next_i32_bound(3) == 0 && r.next_i32_bound(4) == 1);
    // East cell is air but its floor is stone (not a stem-fruit support).
    let mut world = crop_field(ripe, 7).with(p(1, 63, 0), block("minecraft:stone"));
    tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(world.at(p(0, 64, 0)).registry_id, "minecraft:melon_stem");
    assert!(world.writes.is_empty());
}

#[test]
fn nether_wart_ages_on_a_one_in_ten_roll_up_to_three() {
    let seed = seed_with(|r| r.next_i32_bound(10) == 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:nether_wart"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);
    assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(10)));

    let mut mature = TestWorld::default().with(
        p(0, 64, 0),
        with_int(&block("minecraft:nether_wart"), "age", 3),
    );
    let random = tick(&mut mature, p(0, 64, 0), seed);
    assert!(mature.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}

#[test]
fn sweet_berry_bush_short_circuits_before_reading_light() {
    let seed = seed_with(|r| r.next_i32_bound(5) == 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:sweet_berry_bush"));
    tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);

    let mut dark = TestWorld::default().with(p(0, 64, 0), block("minecraft:sweet_berry_bush"));
    dark.light = 8;
    tick(&mut dark, p(0, 64, 0), seed);
    assert!(dark.writes.is_empty());

    let miss = seed_with(|r| r.next_i32_bound(5) != 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:sweet_berry_bush"));
    tick(&mut world, p(0, 64, 0), miss);
    assert!(world.writes.is_empty());
}

#[test]
fn cocoa_pods_ripen_to_age_two() {
    let seed = seed_with(|r| r.next_i32_bound(5) == 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:cocoa"));
    tick(&mut world, p(0, 64, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 1);

    let mut ripe =
        TestWorld::default().with(p(0, 64, 0), with_int(&block("minecraft:cocoa"), "age", 2));
    tick(&mut ripe, p(0, 64, 0), seed);
    assert!(ripe.writes.is_empty());
}

#[test]
fn farmland_dries_out_then_reverts_to_dirt_without_water_or_crop() {
    let mut world = TestWorld::default().with(p(0, 64, 0), moist_farmland(3));
    tick(&mut world, p(0, 64, 0), 1);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "moisture"), 2);
    assert_eq!(world.writes[0].2, UPDATE_CLIENTS);

    let mut dry = TestWorld::default().with(p(0, 64, 0), moist_farmland(0));
    tick(&mut dry, p(0, 64, 0), 1);
    assert_eq!(dry.at(p(0, 64, 0)).registry_id, "minecraft:dirt");
    assert_eq!(dry.writes[0].2, UPDATE_ALL);

    // A crop on top maintains the farmland (`maintains_farmland`).
    let mut planted = TestWorld::default()
        .with(p(0, 64, 0), moist_farmland(0))
        .with(p(0, 65, 0), block("minecraft:wheat"));
    tick(&mut planted, p(0, 64, 0), 1);
    assert_eq!(planted.at(p(0, 64, 0)).registry_id, "minecraft:farmland");
    assert!(planted.writes.is_empty());
}

#[test]
fn farmland_soaks_from_water_within_four_blocks_or_rain() {
    let mut near_water = TestWorld::default()
        .with(p(0, 64, 0), moist_farmland(2))
        .with(p(4, 64, -4), block("minecraft:water"));
    tick(&mut near_water, p(0, 64, 0), 1);
    assert_eq!(int_property(&near_water.at(p(0, 64, 0)), "moisture"), 7);

    // The water box spans y..y+1 only, and only +-4 horizontally.
    let mut too_far = TestWorld::default()
        .with(p(0, 64, 0), moist_farmland(2))
        .with(p(5, 64, 0), block("minecraft:water"))
        .with(p(0, 63, 0), block("minecraft:water"));
    tick(&mut too_far, p(0, 64, 0), 1);
    assert_eq!(int_property(&too_far.at(p(0, 64, 0)), "moisture"), 1);

    let mut rained_on = TestWorld::default().with(p(0, 64, 0), moist_farmland(0));
    rained_on.raining = true;
    tick(&mut rained_on, p(0, 64, 0), 1);
    assert_eq!(int_property(&rained_on.at(p(0, 64, 0)), "moisture"), 7);

    let mut full = TestWorld::default()
        .with(p(0, 64, 0), moist_farmland(7))
        .with(p(1, 64, 0), block("minecraft:water"));
    tick(&mut full, p(0, 64, 0), 1);
    assert!(full.writes.is_empty());
}
