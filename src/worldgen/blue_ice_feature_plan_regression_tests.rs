use super::*;

#[test]
fn blue_ice_start_checks_water_block_identity_and_sea_level() {
    assert!(blue_ice_can_start(
        62,
        63,
        "minecraft:water[level=3]",
        "minecraft:stone",
        &["minecraft:packed_ice"]
    ));
    assert!(blue_ice_can_start(
        62,
        63,
        "minecraft:stone",
        "minecraft:water[level=0]",
        &["minecraft:packed_ice"]
    ));
    assert!(!blue_ice_can_start(
        63,
        63,
        "minecraft:water[level=0]",
        "minecraft:stone",
        &["minecraft:packed_ice"]
    ));
    assert!(!blue_ice_can_start(
        62,
        63,
        "minecraft:water[level=0]",
        "minecraft:stone",
        &["minecraft:ice"]
    ));
}

#[test]
fn blue_ice_spread_accepts_all_air_states_and_water_levels() {
    for candidate in [
        "minecraft:air",
        "minecraft:cave_air",
        "minecraft:void_air",
        "minecraft:water[level=7]",
        "minecraft:packed_ice",
        "minecraft:ice",
    ] {
        assert!(blue_ice_spread_can_place(
            candidate,
            &["minecraft:blue_ice"]
        ));
        assert!(!blue_ice_spread_can_place(
            candidate,
            &["minecraft:packed_ice"]
        ));
    }
    assert!(!blue_ice_spread_can_place(
        "minecraft:stone",
        &["minecraft:blue_ice"]
    ));
    assert!(!blue_ice_spread_can_place(
        "minecraft:oak_slab[type=bottom,waterlogged=true]",
        &["minecraft:blue_ice"]
    ));
}

#[test]
fn blue_ice_candidate_planner_preserves_java_attempt_and_random_draw_order() {
    let origin = BlockPos {
        x: -31,
        y: 63,
        z: 47,
    };
    for algorithm in [RandomAlgorithm::Legacy, RandomAlgorithm::Xoroshiro] {
        let mut actual_random = RandomSourceKind::new(0x5eed, algorithm);
        let mut reference_random = actual_random;
        let mut expected = Vec::with_capacity(200);

        // BlueIceFeature.place draws Y first, then X and Z with the per-attempt
        // xzDiff bound. Legal y offsets are -5..=4, so each of its 200 attempts
        // consumes all six draws and yields one candidate.
        for _ in 0..200 {
            let y_offset = worldgen_random_next_i32_bound(&mut reference_random, 5)
                - worldgen_random_next_i32_bound(&mut reference_random, 6);
            let mut xz_diff = 3;
            if y_offset < 2 {
                xz_diff += y_offset / 2;
            }
            assert!((1..=3).contains(&xz_diff));
            let x = worldgen_random_next_i32_bound(&mut reference_random, xz_diff)
                - worldgen_random_next_i32_bound(&mut reference_random, xz_diff);
            let z = worldgen_random_next_i32_bound(&mut reference_random, xz_diff)
                - worldgen_random_next_i32_bound(&mut reference_random, xz_diff);
            expected.push(BlockPos {
                x: origin.x + x,
                y: origin.y + y_offset,
                z: origin.z + z,
            });
        }

        let actual = blue_ice_spread_candidates(origin, &mut actual_random);
        assert_eq!(actual, expected);
        assert_eq!(
            worldgen_random_next_i32_bound(&mut actual_random, 10_000),
            worldgen_random_next_i32_bound(&mut reference_random, 10_000),
            "planner must leave the random stream after exactly 200 Java attempts"
        );
    }
}
