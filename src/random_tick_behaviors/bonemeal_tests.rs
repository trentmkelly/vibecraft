//! Tests for the `BonemealableBlock` ports behind `BoneMealItem.growCrop`.

use super::test_world::{block, p, TestWorld};
use super::tests::{assert_same_state, replay, seed_with};
use super::*;
use crate::random_source::LegacyRandom;

fn apply(world: &mut TestWorld, pos: BlockPos, seed: i64) -> (BonemealOutcome, LegacyRandom) {
    let mut random = LegacyRandom::new(seed);
    let outcome = bonemeal_block(world, pos, &mut random);
    (outcome, random)
}

const APPLIED: BonemealOutcome = BonemealOutcome::Applied { grew: true };

/// `Mth.nextInt(random, 2, 5)`.
fn age_roll(random: &mut LegacyRandom) -> i32 {
    2 + random.next_i32_bound(4)
}

#[test]
fn wheat_gains_two_to_five_stages_and_caps_at_maturity() {
    for seed in [1, 2, 3, 4, 5] {
        let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:wheat"));
        let (outcome, random) = apply(&mut world, p(0, 64, 0), seed);
        let expected_roll = age_roll(&mut LegacyRandom::new(seed));
        assert_eq!(outcome, APPLIED);
        assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), expected_roll);
        assert_eq!(world.writes[0].2, UPDATE_CLIENTS);
        assert_same_state(random, replay(seed, |r| _ = age_roll(r)));
    }
    let mut near =
        TestWorld::default().with(p(0, 64, 0), with_int(&block("minecraft:wheat"), "age", 6));
    apply(&mut near, p(0, 64, 0), 1);
    assert_eq!(int_property(&near.at(p(0, 64, 0)), "age"), 7);

    let mut mature =
        TestWorld::default().with(p(0, 64, 0), with_int(&block("minecraft:wheat"), "age", 7));
    let (outcome, random) = apply(&mut mature, p(0, 64, 0), 1);
    assert_eq!(outcome, BonemealOutcome::NotApplicable);
    assert_same_state(random, LegacyRandom::new(1));
}

#[test]
fn beetroot_divides_the_roll_by_three_and_torchflower_adds_one() {
    for seed in [1, 2, 3, 4, 5, 6, 7, 8] {
        let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:beetroots"));
        apply(&mut world, p(0, 64, 0), seed);
        let expected = age_roll(&mut LegacyRandom::new(seed)) / 3;
        assert_eq!(int_property(&world.at(p(0, 64, 0)), "age"), expected);
    }
    let mut flower = TestWorld::default().with(
        p(0, 64, 0),
        with_int(&block("minecraft:torchflower_crop"), "age", 1),
    );
    let (_, random) = apply(&mut flower, p(0, 64, 0), 9);
    assert_eq!(flower.at(p(0, 64, 0)).registry_id, "minecraft:torchflower");
    assert_same_state(random, LegacyRandom::new(9));
}

#[test]
fn stem_bonemeal_grows_and_a_mature_stem_immediately_random_ticks() {
    let mut young = TestWorld::default().with(p(0, 64, 0), block("minecraft:pumpkin_stem"));
    apply(&mut young, p(0, 64, 0), 4);
    assert_eq!(
        int_property(&young.at(p(0, 64, 0)), "age"),
        age_roll(&mut LegacyRandom::new(4))
    );

    // A stem one stage short always reaches 7 and then runs `randomTick`,
    // which consumes the growth roll (bound 26 without farmland: speed 1).
    let mut near = TestWorld::default().with(
        p(0, 64, 0),
        with_int(&block("minecraft:melon_stem"), "age", 5),
    );
    let seed = seed_with(|r| {
        age_roll(r);
        r.next_i32_bound(26) != 0
    });
    let (outcome, random) = apply(&mut near, p(0, 64, 0), seed);
    assert_eq!(outcome, APPLIED);
    assert_eq!(int_property(&near.at(p(0, 64, 0)), "age"), 7);
    assert_same_state(
        random,
        replay(seed, |r| {
            age_roll(r);
            r.next_i32_bound(26);
        }),
    );

    let mut done = TestWorld::default().with(
        p(0, 64, 0),
        with_int(&block("minecraft:melon_stem"), "age", 7),
    );
    assert_eq!(
        apply(&mut done, p(0, 64, 0), 4).0,
        BonemealOutcome::NotApplicable
    );
}

#[test]
fn cocoa_and_sweet_berries_advance_one_stage() {
    let mut cocoa = TestWorld::default().with(p(0, 64, 0), block("minecraft:cocoa"));
    assert_eq!(apply(&mut cocoa, p(0, 64, 0), 1).0, APPLIED);
    assert_eq!(int_property(&cocoa.at(p(0, 64, 0)), "age"), 1);
    let mut ripe =
        TestWorld::default().with(p(0, 64, 0), with_int(&block("minecraft:cocoa"), "age", 2));
    assert_eq!(
        apply(&mut ripe, p(0, 64, 0), 1).0,
        BonemealOutcome::NotApplicable
    );

    let mut berries = TestWorld::default().with(p(0, 64, 0), block("minecraft:sweet_berry_bush"));
    assert_eq!(apply(&mut berries, p(0, 64, 0), 1).0, APPLIED);
    assert_eq!(int_property(&berries.at(p(0, 64, 0)), "age"), 1);
    // Needs air above.
    let mut covered = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:sweet_berry_bush"))
        .with(p(0, 65, 0), block("minecraft:stone"));
    assert_eq!(
        apply(&mut covered, p(0, 64, 0), 1).0,
        BonemealOutcome::NotApplicable
    );
}

#[test]
fn kelp_grows_exactly_one_block_into_water() {
    let mut world = TestWorld::default()
        .with(p(0, 64, 0), with_int(&block("minecraft:kelp"), "age", 3))
        .with(p(0, 65, 0), block("minecraft:water"))
        .with(p(0, 66, 0), block("minecraft:water"));
    let (outcome, random) = apply(&mut world, p(0, 64, 0), 1);
    assert_eq!(outcome, APPLIED);
    assert_eq!(int_property(&world.at(p(0, 65, 0)), "age"), 4);
    assert_eq!(world.at(p(0, 66, 0)).registry_id, "minecraft:water");
    assert_eq!(world.writes[0].2, UPDATE_ALL);
    assert_same_state(random, LegacyRandom::new(1));

    let mut blocked = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:kelp"))
        .with(p(0, 65, 0), block("minecraft:stone"));
    assert_eq!(
        apply(&mut blocked, p(0, 64, 0), 1).0,
        BonemealOutcome::NotApplicable
    );
}

/// The number of blocks `NetherVines.getBlocksToGrowWhenBonemealed` yields.
fn nether_vine_count(random: &mut LegacyRandom) -> i32 {
    let mut probability = 1.0_f64;
    let mut count = 0;
    while random.next_f64() < probability {
        probability *= 0.826;
        count += 1;
    }
    count
}

#[test]
fn nether_vines_grow_a_geometric_number_of_blocks() {
    for seed in [1, 2, 3, 4, 5, 6] {
        let mut world = TestWorld::default().with(
            p(0, 64, 0),
            with_int(&block("minecraft:weeping_vines"), "age", 2),
        );
        let (outcome, random) = apply(&mut world, p(0, 64, 0), seed);
        assert_eq!(outcome, APPLIED);
        let count = nether_vine_count(&mut LegacyRandom::new(seed));
        for i in 1..=count {
            let grown = world.at(p(0, 64 - i, 0));
            assert_eq!(grown.registry_id, "minecraft:weeping_vines");
            assert_eq!(int_property(&grown, "age"), 2 + i);
        }
        assert!(world.at(p(0, 64 - count - 1, 0)).is_air());
        assert_same_state(random, replay(seed, |r| _ = nether_vine_count(r)));
    }
}

#[test]
fn nether_vines_stop_at_an_obstruction() {
    let seed = seed_with(|r| nether_vine_count(r) >= 3);
    let mut world = TestWorld::default()
        .with(p(0, 64, 0), block("minecraft:twisting_vines"))
        .with(p(0, 66, 0), block("minecraft:stone"));
    apply(&mut world, p(0, 64, 0), seed);
    assert_eq!(
        world.at(p(0, 65, 0)).registry_id,
        "minecraft:twisting_vines"
    );
    assert_eq!(world.at(p(0, 66, 0)).registry_id, "minecraft:stone");
}

#[test]
fn cave_vines_bonemeal_adds_berries_only_once() {
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:cave_vines"));
    assert_eq!(apply(&mut world, p(0, 64, 0), 1).0, APPLIED);
    assert_eq!(world.at(p(0, 64, 0)).property("berries"), Some("true"));
    assert_eq!(world.writes[0].2, UPDATE_CLIENTS);
    assert_eq!(
        apply(&mut world, p(0, 64, 0), 1).0,
        BonemealOutcome::NotApplicable
    );
}

#[test]
fn bamboo_sapling_and_stalks_grow_upward() {
    let mut sapling = TestWorld::default().with(p(0, 64, 0), block("minecraft:bamboo_sapling"));
    assert_eq!(apply(&mut sapling, p(0, 64, 0), 1).0, APPLIED);
    assert_eq!(sapling.at(p(0, 65, 0)).property("leaves"), Some("small"));

    for seed in [1, 2, 3, 4] {
        let mut world = TestWorld::default()
            .with(p(0, 63, 0), block("minecraft:dirt"))
            .with(p(0, 64, 0), block("minecraft:bamboo"));
        let (outcome, random) = apply(&mut world, p(0, 64, 0), seed);
        assert_eq!(outcome, APPLIED);
        let new_bamboo = 1 + LegacyRandom::new(seed).next_i32_bound(2);
        for y in 65..65 + new_bamboo {
            assert_eq!(world.at(p(0, y, 0)).registry_id, "minecraft:bamboo");
        }
        assert!(world.at(p(0, 65 + new_bamboo, 0)).is_air());
        assert_same_state(random, replay(seed, |r| _ = r.next_i32_bound(2)));
    }
}

#[test]
fn sapling_bonemeal_succeeds_on_a_forty_five_percent_float() {
    let success = seed_with(|r| r.next_f32() < 0.45);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:oak_sapling"));
    let (outcome, random) = apply(&mut world, p(0, 64, 0), success);
    assert_eq!(outcome, APPLIED);
    assert_eq!(int_property(&world.at(p(0, 64, 0)), "stage"), 1);
    assert_same_state(random, replay(success, |r| _ = r.next_f32()));

    // A failed roll still consumes the bone meal but changes nothing.
    let failure = seed_with(|r| r.next_f32() >= 0.45);
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:oak_sapling"));
    let (outcome, _) = apply(&mut world, p(0, 64, 0), failure);
    assert_eq!(outcome, BonemealOutcome::Applied { grew: false });
    assert!(world.writes.is_empty());
}

#[test]
fn other_blocks_are_not_bonemealable_here() {
    let mut world = TestWorld::default().with(p(0, 64, 0), block("minecraft:stone"));
    let (outcome, random) = apply(&mut world, p(0, 64, 0), 1);
    assert_eq!(outcome, BonemealOutcome::NotApplicable);
    assert_same_state(random, LegacyRandom::new(1));
}
