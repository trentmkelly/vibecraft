//! Tests for the block-growing `randomTick` ports: sugar cane, cactus,
//! bamboo, the growing-plant heads and vines.

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

fn with_leaves(state: &BlockStateModel, leaves: &str) -> BlockStateModel {
    state.clone().try_set_property("leaves", leaves)
}

#[test]
fn sugar_cane_ages_then_grows_upward_without_using_the_rng() {
    let cane = block("minecraft:sugar_cane");
    let mut world = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), with_int(&cane, "age", 4));
    let random = tick(&mut world, p(0, 64, 0), 5);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 5);
    assert_eq!(
        world.writes,
        vec![(
            p(0, 64, 0),
            "minecraft:sugar_cane[age=5]".into(),
            UPDATE_QUIET
        )]
    );
    assert_same_state(random, LegacyRandom::new(5));

    let mut ripe = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), with_int(&cane, "age", 15));
    tick(&mut ripe, p(0, 64, 0), 5);
    assert_eq!(ripe.at(p(0, 65, 0)).registry_id, "minecraft:sugar_cane");
    assert_eq!(int_property(&ripe.at(p(0, 64, 0)), "age"), 0);
    assert_eq!(
        ripe.writes[0],
        (
            p(0, 65, 0),
            "minecraft:sugar_cane[age=0]".into(),
            UPDATE_ALL
        )
    );
    assert_eq!(ripe.writes[1].2, UPDATE_QUIET);
}

#[test]
fn sugar_cane_stops_at_height_three_or_when_blocked() {
    let ripe = with_int(&block("minecraft:sugar_cane"), "age", 15);
    let mut tall = TestWorld::default()
        .with(p(0, 62, 0), ripe.clone())
        .with(p(0, 63, 0), ripe.clone())
        .with(p(0, 64, 0), ripe.clone());
    tick(&mut tall, p(0, 64, 0), 1);
    assert!(tall.writes.is_empty());

    let mut blocked = TestWorld::default()
        .with(p(0, 64, 0), ripe)
        .with(p(0, 65, 0), block("minecraft:stone"));
    tick(&mut blocked, p(0, 64, 0), 1);
    assert!(blocked.writes.is_empty());
}

#[test]
fn cactus_ages_and_grows_a_new_segment_at_age_fifteen() {
    let cactus = block("minecraft:cactus");
    let mut young = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), with_int(&cactus, "age", 3));
    tick(&mut young, p(0, 64, 0), 1);
    assert_eq!(int_property(&young.at(p(0, 64, 0)), "age"), 4);
    assert_eq!(young.writes[0].2, UPDATE_QUIET);

    let mut ripe = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), with_int(&cactus, "age", 15));
    tick(&mut ripe, p(0, 64, 0), 1);
    assert_eq!(ripe.at(p(0, 65, 0)).registry_id, "minecraft:cactus");
    assert_eq!(int_property(&ripe.at(p(0, 64, 0)), "age"), 0);
    assert_eq!(ripe.writes[0].2, UPDATE_ALL);

    // A full-height column with age 15 returns before any write.
    let full = with_int(&cactus, "age", 15);
    let mut column = TestWorld::default()
        .with(p(0, 62, 0), full.clone())
        .with(p(0, 63, 0), full.clone())
        .with(p(0, 64, 0), full);
    tick(&mut column, p(0, 64, 0), 1);
    assert!(column.writes.is_empty());
}

#[test]
fn cactus_flowers_at_age_eight_with_the_height_dependent_chance() {
    let cactus = with_int(&block("minecraft:cactus"), "age", 8);
    let hit = seed_with(|r| r.next_f64() <= 0.1);
    let mut world = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), cactus.clone());
    let random = tick(&mut world, p(0, 64, 0), hit);
    assert_eq!(world.at(p(0, 65, 0)).registry_id, "minecraft:cactus_flower");
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), 9);
    assert_same_state(random, replay(hit, |r| _ = r.next_f64()));

    // A miss still ages the cactus; height >= 3 uses 0.25 instead of 0.1.
    let between = seed_with(|r| {
        let roll = r.next_f64();
        roll > 0.1 && roll <= 0.25
    });
    let mut short = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:sand"))
        .with(p(0, 64, 0), cactus.clone());
    tick(&mut short, p(0, 64, 0), between);
    assert!(short.at(p(0, 65, 0)).is_air());
    let mut tall = TestWorld::default()
        .with(p(0, 61, 0), block("minecraft:sand"))
        .with(p(0, 62, 0), block("minecraft:cactus"))
        .with(p(0, 63, 0), block("minecraft:cactus"))
        .with(p(0, 64, 0), cactus);
    tick(&mut tall, p(0, 64, 0), between);
    assert_eq!(tall.at(p(0, 65, 0)).registry_id, "minecraft:cactus_flower");
}

#[test]
fn bamboo_grows_a_small_leafed_segment_on_a_one_in_three_roll() {
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:dirt"))
        .with(p(0, 64, 0), block("minecraft:bamboo"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    let grown = world.at(p(0, 65, 0));
    assert_eq!(grown.registry_id, "minecraft:bamboo");
    assert_eq!(grown.property("leaves"), Some("small"));
    assert_eq!(int_property(&grown, "age"), 0);
    assert_eq!(int_property(&grown, "stage"), 0);
    assert_eq!(world.writes[0].2, UPDATE_ALL);
    assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(3)));

    let miss = seed_with(|r| r.next_i32_bound(3) != 0);
    let mut idle = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:dirt"))
        .with(p(0, 64, 0), block("minecraft:bamboo"));
    tick(&mut idle, p(0, 64, 0), miss);
    assert!(idle.writes.is_empty());
}

#[test]
fn bamboo_promotes_leaves_to_large_and_shrinks_the_ones_below() {
    let bamboo = block("minecraft:bamboo");
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:dirt"))
        .with(p(0, 64, 0), bamboo.clone())
        .with(p(0, 65, 0), with_leaves(&bamboo, "small"))
        .with(p(0, 66, 0), with_leaves(&bamboo, "small"));
    tick(&mut world, p(0, 66, 0), seed);
    let top = world.at(p(0, 67, 0));
    assert_eq!(top.property("leaves"), Some("large"));
    assert_eq!(int_property(&top, "age"), 1);
    assert_eq!(world.at(p(0, 65, 0)).property("leaves"), Some("small"));
    assert_eq!(world.at(p(0, 64, 0)).property("leaves"), Some("none"));
}

#[test]
fn bamboo_rolls_the_stage_float_only_from_height_eleven() {
    let bamboo = block("minecraft:bamboo");
    let seed = seed_with(|r| r.next_i32_bound(3) == 0 && r.next_f32() < 0.25);
    let mut world = TestWorld::default().with(p(0, 63, 0), block("minecraft:dirt"));
    for y in 64..=74 {
        world.put(p(0, y, 0), bamboo.clone());
    }
    let random = tick(&mut world, p(0, 74, 0), seed);
    assert_eq!(int_property(&world.at(p(0, 75, 0)), "stage"), 1);
    assert_same_state(
        random,
        replay(seed, |r| {
            r.next_i32_bound(3);
            r.next_f32();
        }),
    );

    // Height 15 always ends the stalk (stage 1) without needing the float
    // to succeed, but the float is still drawn (height >= 11).
    let mut tall = TestWorld::default().with(p(0, 63, 0), block("minecraft:dirt"));
    for y in 64..=78 {
        tall.put(p(0, y, 0), bamboo.clone());
    }
    let miss = seed_with(|r| r.next_i32_bound(3) == 0 && r.next_f32() >= 0.25);
    tick(&mut tall, p(0, 78, 0), miss);
    assert_eq!(int_property(&tall.at(p(0, 79, 0)), "stage"), 1);
}

#[test]
fn bamboo_stops_at_sixteen_and_ended_stalks_do_not_tick() {
    let bamboo = block("minecraft:bamboo");
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = TestWorld::default().with(p(0, 63, 0), block("minecraft:dirt"));
    for y in 64..=79 {
        world.put(p(0, y, 0), bamboo.clone());
    }
    tick(&mut world, p(0, 79, 0), seed);
    assert!(world.writes.is_empty());

    let ended = with_int(&bamboo, "stage", 1);
    let mut ended_world = TestWorld::default()
        .with(p(0, 63, 0), block("minecraft:dirt"))
        .with(p(0, 64, 0), ended);
    let random = tick(&mut ended_world, p(0, 64, 0), seed);
    assert!(ended_world.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}

#[test]
fn bamboo_sapling_becomes_a_small_leafed_bamboo() {
    let seed = seed_with(|r| r.next_i32_bound(3) == 0);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:bamboo_sapling"));
    tick(&mut world, p(0, 64, 0), seed);
    let bamboo = world.at(p(0, 65, 0));
    assert_eq!(bamboo.registry_id, "minecraft:bamboo");
    assert_eq!(bamboo.property("leaves"), Some("small"));
}

#[test]
fn kelp_grows_into_water_on_its_fourteen_percent_roll() {
    let seed = seed_with(|r| r.next_f64() < 0.14);
    let mut world = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:kelp"))
        .with(p(0, 65, 0), block("minecraft:water"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    let grown = world.at(p(0, 65, 0));
    assert_eq!(grown.registry_id, "minecraft:kelp");
    assert_eq!(int_property(&grown, "age"), 1);
    assert_eq!(world.writes[0].2, UPDATE_ALL);
    assert_same_state(random, replay(seed, |r| _ = r.next_f64()));

    // Kelp only grows into water, but the roll is still drawn.
    let mut stone = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:kelp"))
        .with(p(0, 65, 0), block("minecraft:stone"));
    let random = tick(&mut stone, p(0, 64, 0), seed);
    assert!(stone.writes.is_empty());
    assert_same_state(random, replay(seed, |r| _ = r.next_f64()));

    // Age 25 heads are not rolled at all.
    let mut mature = TestWorld::default()
        .with(p(0, 64, 0), with_int(&block("minecraft:kelp"), "age", 25))
        .with(p(0, 65, 0), block("minecraft:water"));
    let random = tick(&mut mature, p(0, 64, 0), seed);
    assert!(mature.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}

#[test]
fn nether_vines_grow_in_their_direction_at_ten_percent() {
    let seed = seed_with(|r| r.next_f64() < 0.1);
    let mut twisting = TestWorld::default().with(p(0, 64, 0), block("minecraft:twisting_vines"));
    tick(&mut twisting, p(0, 64, 0), seed);
    assert_eq!(
        twisting.at(p(0, 65, 0)).registry_id,
        "minecraft:twisting_vines"
    );

    let mut weeping = TestWorld::default().with(p(0, 64, 0), block("minecraft:weeping_vines"));
    tick(&mut weeping, p(0, 64, 0), seed);
    assert_eq!(
        weeping.at(p(0, 63, 0)).registry_id,
        "minecraft:weeping_vines"
    );
    assert!(weeping.at(p(0, 65, 0)).is_air());

    let miss = seed_with(|r| r.next_f64() >= 0.1);
    let mut idle = TestWorld::default().with(p(0, 64, 0), block("minecraft:weeping_vines"));
    tick(&mut idle, p(0, 64, 0), miss);
    assert!(idle.writes.is_empty());
}

#[test]
fn cave_vines_roll_berries_after_the_growth_roll() {
    let seed = seed_with(|r| r.next_f64() < 0.1 && r.next_f32() < 0.11);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:cave_vines"));
    let random = tick(&mut world, p(0, 64, 0), seed);
    let grown = world.at(p(0, 63, 0));
    assert_eq!(grown.registry_id, "minecraft:cave_vines");
    assert_eq!(grown.property("berries"), Some("true"));
    assert_eq!(int_property(&grown, "age"), 1);
    assert_same_state(
        random,
        replay(seed, |r| {
            r.next_f64();
            r.next_f32();
        }),
    );

    let no_berries = seed_with(|r| r.next_f64() < 0.1 && r.next_f32() >= 0.11);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:cave_vines"));
    tick(&mut world, p(0, 64, 0), no_berries);
    assert_eq!(world.at(p(0, 63, 0)).property("berries"), Some("false"));
}

fn vine_on_north_wall() -> TestWorld {
    let vine = with_bool(&block("minecraft:vine"), "north", true);
    TestWorld::default()
        .with(p(0, 64, -1), block("minecraft:stone"))
        .with(p(1, 64, -1), block("minecraft:stone"))
        .with(p(0, 64, 0), vine)
}

/// A seed whose first draws are `nextInt(4) == 0` then the given direction
/// index from `Direction.VALUES` (down, up, north, south, west, east).
fn vine_seed(direction_index: i32) -> i64 {
    seed_with(|r| r.next_i32_bound(4) == 0 && r.next_i32_bound(6) == direction_index)
}

#[test]
fn vine_spreads_around_a_corner_along_the_wall() {
    // East (index 5) is unset; the counter-clockwise neighbour (north) has a
    // face and the cell north-east is stone, so the vine copies that face.
    let seed = vine_seed(5);
    let mut world = vine_on_north_wall();
    let random = tick(&mut world, p(0, 64, 0), seed);
    let spread = world.at(p(1, 64, 0));
    assert_eq!(spread.registry_id, "minecraft:vine");
    assert_eq!(spread.property("north"), Some("true"));
    assert_eq!(world.writes[0].2, UPDATE_CLIENTS);
    assert_same_state(
        random,
        replay(seed, |r| {
            r.next_i32_bound(4);
            r.next_i32_bound(6);
        }),
    );
}

#[test]
fn vine_attaches_a_new_face_onto_an_adjacent_solid_block() {
    // West (index 4) is unset and the west neighbour is solid.
    let seed = vine_seed(4);
    let mut world = vine_on_north_wall().with(p(-1, 64, 0), block("minecraft:stone"));
    tick(&mut world, p(0, 64, 0), seed);
    let vine = world.at(p(0, 64, 0));
    assert_eq!(vine.property("west"), Some("true"));
    assert_eq!(vine.property("north"), Some("true"));
}

#[test]
fn vine_grows_downward_copying_random_faces() {
    // Down (index 0): four `nextBoolean` calls in N, E, S, W order; the vine
    // only has the north face, so the first boolean decides the copy.
    let seed =
        seed_with(|r| r.next_i32_bound(4) == 0 && r.next_i32_bound(6) == 0 && r.next_bits(1) != 0);
    let mut world = vine_on_north_wall();
    let random = tick(&mut world, p(0, 64, 0), seed);
    let below = world.at(p(0, 63, 0));
    assert_eq!(below.registry_id, "minecraft:vine");
    assert_eq!(below.property("north"), Some("true"));
    assert_same_state(
        random,
        replay(seed, |r| {
            r.next_i32_bound(4);
            r.next_i32_bound(6);
            for _ in 0..4 {
                r.next_bits(1);
            }
        }),
    );

    // When the north boolean is false nothing is copied and nothing placed.
    let empty =
        seed_with(|r| r.next_i32_bound(4) == 0 && r.next_i32_bound(6) == 0 && r.next_bits(1) == 0);
    let mut world = vine_on_north_wall();
    tick(&mut world, p(0, 64, 0), empty);
    assert!(world.writes.is_empty());
}

#[test]
fn vine_caps_the_local_density_and_honours_the_spread_gamerule() {
    let seed = vine_seed(5);
    let mut crowded = vine_on_north_wall();
    for x in 1..=4 {
        crowded.put(p(x, 64, 1), block("minecraft:vine"));
    }
    tick(&mut crowded, p(0, 64, 0), seed);
    assert!(
        crowded.writes.is_empty(),
        "five vines within the box block spreading"
    );

    let mut disabled = vine_on_north_wall();
    disabled.spread_vines = false;
    let random = tick(&mut disabled, p(0, 64, 0), seed);
    assert!(disabled.writes.is_empty());
    assert_same_state(random, LegacyRandom::new(seed));
}
