use super::*;

#[test]
fn basalt_taper_uses_sample_distance_before_vertical_search() {
    let origin = BlockPos { x: 0, y: 64, z: 0 };
    for surface_y in [54, 64, 74] {
        let blocks = basalt_column_blocks_from_surface(
            BlockPos {
                x: 2,
                y: surface_y,
                z: 0,
            },
            origin,
            4,
            3,
            &[true; 8],
            &[false; 8],
        );
        // Java uses stepLimit=2, so blocksY=4-2/2=3; the loop is inclusive.
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].pos.y, surface_y);
        assert_eq!(blocks[3].pos.y, surface_y + 3);
    }
}

#[test]
fn basalt_square_corners_are_visited_and_tapered() {
    let blocks = basalt_column_blocks_from_surface(
        BlockPos { x: 3, y: 64, z: 3 },
        BlockPos { x: 0, y: 64, z: 0 },
        4,
        3,
        &[true; 8],
        &[false; 8],
    );
    // betweenClosed includes this corner; stepLimit=6 leaves two writes.
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1].pos, BlockPos { x: 3, y: 65, z: 3 });
}

#[test]
fn basalt_existing_column_consumes_height_without_replacement() {
    let blocks = basalt_column_blocks_from_surface(
        BlockPos { x: 0, y: 64, z: 0 },
        BlockPos { x: 0, y: 64, z: 0 },
        4,
        1,
        &[true, false, true, false, true],
        &[false, true, false, false, false],
    );
    // Existing basalt advances the cursor; the later solid obstruction stops it.
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1].pos.y, 66);
}

#[test]
fn basalt_air_and_block_identity_gates_match_java() {
    for air in ["minecraft:air", "minecraft:cave_air", "minecraft:void_air"] {
        assert!(basalt_columns_can_place_at(
            air,
            "minecraft:netherrack",
            32,
            32
        ));
        assert!(!basalt_columns_can_place_at("minecraft:air", air, 32, 32));
    }
    assert!(basalt_columns_is_air_or_lava_ocean(
        "minecraft:lava[level=5]",
        32,
        32
    ));
    assert!(!basalt_columns_is_air_or_lava_ocean(
        "minecraft:lava[level=0]",
        33,
        32
    ));
    for forbidden in [
        "minecraft:lava[level=5]",
        "minecraft:nether_brick_stairs[facing=north,half=bottom,shape=straight,waterlogged=false]",
        "minecraft:chest[facing=north,type=single,waterlogged=false]",
    ] {
        assert!(!basalt_columns_can_place_at(
            "minecraft:cave_air",
            forbidden,
            32,
            32
        ));
    }
}

#[test]
fn basalt_find_surface_obeys_search_limit_and_min_y_boundary() {
    let states = [
        (65, "minecraft:air"),
        (64, "minecraft:air"),
        (63, "minecraft:netherrack"),
        (62, "minecraft:bedrock"),
        (61, "minecraft:air"),
        (60, "minecraft:netherrack"),
    ];
    assert_eq!(
        basalt_columns_find_surface_y(65, 60, 1, &states, 32),
        None,
        "one iteration only checks the unsupported starting air at y=65"
    );
    assert_eq!(
        basalt_columns_find_surface_y(65, 60, 2, &states, 32),
        Some(64),
        "the second and final allowed iteration finds supported air at y=64"
    );
    assert_eq!(
        basalt_columns_find_surface_y(62, 60, 8, &states, 32),
        None,
        "Java excludes y=minY+1 and lower"
    );
    assert_eq!(
        basalt_columns_find_surface_y(64, 60, 0, &states, 32),
        None,
        "a zero limit must not inspect even a valid starting surface"
    );
}

#[test]
fn basalt_find_surface_uses_lava_level_and_requires_complete_samples() {
    let lava_surface = [
        (34, "minecraft:lava[level=0]"),
        (33, "minecraft:netherrack"),
        (32, "minecraft:lava[level=0]"),
        (31, "minecraft:netherrack"),
    ];
    assert_eq!(
        basalt_columns_find_surface_y(34, 30, 2, &lava_surface, 32),
        None,
        "above-sea lava cannot be the column surface"
    );
    assert_eq!(
        basalt_columns_find_surface_y(32, 30, 1, &lava_surface, 32),
        Some(32),
        "lava at the inclusive sea level can be replaced"
    );
    assert_eq!(
        basalt_columns_find_surface_y(34, 30, 4, &[(34, "minecraft:air")], 32),
        None,
        "an incomplete world snapshot cannot prove support below"
    );
}

#[test]
fn basalt_find_air_honors_inclusive_ceiling_forbidden_stop_and_limit() {
    let states = [
        (70, "minecraft:netherrack"),
        (71, "minecraft:bedrock"),
        (72, "minecraft:air"),
    ];
    assert_eq!(basalt_columns_find_air_y(70, 72, 3, &states), None);
    assert_eq!(
        basalt_columns_find_air_y(
            70,
            72,
            1,
            &[(70, "minecraft:netherrack"), (71, "minecraft:air")]
        ),
        None,
        "one iteration cannot reach the air at y=71"
    );
    assert_eq!(
        basalt_columns_find_air_y(
            70,
            72,
            2,
            &[(70, "minecraft:netherrack"), (71, "minecraft:air")]
        ),
        Some(71)
    );
    assert_eq!(
        basalt_columns_find_air_y(72, 72, 1, &[(72, "minecraft:void_air")]),
        Some(72),
        "maxY is included and Java accepts every air state"
    );
    assert_eq!(
        basalt_columns_find_air_y(72, 71, 1, &[(72, "minecraft:air")]),
        None,
        "positions above maxY are not examined"
    );
    assert_eq!(
        basalt_columns_find_air_y(70, 72, 0, &[(70, "minecraft:air")]),
        None,
        "a zero limit must not inspect the start position"
    );
}

#[test]
fn basalt_find_air_stops_on_forbidden_block_before_later_air() {
    let states = [
        (10, "minecraft:netherrack"),
        (11, "minecraft:nether_brick_fence"),
        (12, "minecraft:cave_air"),
    ];
    assert_eq!(basalt_columns_find_air_y(10, 12, 3, &states), None);
    assert_eq!(
        basalt_columns_find_air_y(12, 12, 1, &states),
        Some(12),
        "air at the initial cursor is found before any lower blocks are considered"
    );
}
