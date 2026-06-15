use super::*;

#[test]
fn world_carver_types_match_vanilla_bootstrap_entries() {
    assert_eq!(
        WORLD_CARVER_TYPES
            .iter()
            .map(|carver| carver.id())
            .collect::<Vec<_>>(),
        vec![
            "minecraft:cave",
            "minecraft:nether_cave",
            "minecraft:canyon"
        ]
    );
    assert_eq!(
        super::super::world_carver_type("cave"),
        Some(WorldCarverType::Cave)
    );
    assert_eq!(
        super::super::world_carver_type("minecraft:nether_cave"),
        Some(WorldCarverType::NetherCave)
    );
}

#[test]
fn configured_carver_ids_match_vanilla_bootstrap_entries() {
    assert_eq!(
        CONFIGURED_CARVERS
            .iter()
            .map(|carver| carver.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:cave",
            "minecraft:cave_extra_underground",
            "minecraft:canyon",
            "minecraft:nether_cave",
        ]
    );
}

#[test]
fn configured_cave_carver_matches_vanilla_bootstrap_entry() {
    let cave = super::super::configured_carver("cave").unwrap();
    assert_eq!(cave.carver_type, WorldCarverType::Cave);
    assert_eq!(cave.probability, 0.15);
    assert!(super::super::carver_is_start_chunk(cave, 0.15));
    assert!(!super::super::carver_is_start_chunk(cave, 0.150_001));
    assert_eq!(
        cave.y,
        HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(180),
        }
    );
    assert_eq!(cave.y_scale, FloatProvider::Uniform { min: 0.1, max: 0.9 });
    assert_eq!(cave.lava_level, VerticalAnchor::AboveBottom(8));
    assert_eq!(cave.debug.barrier_state, "minecraft:crimson_button");
    assert_eq!(
        cave.replaceable_tag,
        "#minecraft:overworld_carver_replaceables"
    );
}

#[test]
fn configured_extra_cave_and_canyon_match_vanilla_bootstrap_entries() {
    let extra = super::super::configured_carver("cave_extra_underground").unwrap();
    assert_eq!(extra.probability, 0.07);
    assert_eq!(extra.y.max, VerticalAnchor::Absolute(47));
    assert_eq!(extra.debug.barrier_state, "minecraft:oak_button");

    let canyon = super::super::configured_carver("canyon").unwrap();
    assert_eq!(canyon.carver_type, WorldCarverType::Canyon);
    assert_eq!(canyon.probability, 0.01);
    assert_eq!(canyon.y_scale, FloatProvider::Constant(3.0));
    assert_eq!(canyon.debug.barrier_state, "minecraft:warped_button");
    assert!(matches!(
        canyon.shape,
        CarverShape::Canyon {
            vertical_rotation: FloatProvider::Uniform {
                min: -0.125,
                max: 0.125
            },
            ..
        }
    ));
}

#[test]
fn configured_nether_cave_matches_vanilla_bootstrap_entry() {
    let nether = super::super::configured_carver("nether_cave").unwrap();
    assert_eq!(nether.carver_type, WorldCarverType::NetherCave);
    assert_eq!(nether.probability, 0.2);
    assert_eq!(nether.y.min, VerticalAnchor::Absolute(0));
    assert_eq!(nether.y.max, VerticalAnchor::BelowTop(1));
    assert_eq!(
        super::super::carver_cave_bound(WorldCarverType::NetherCave),
        10
    );
    assert_eq!(
        super::super::carver_tunnel_y_scale(WorldCarverType::NetherCave),
        5.0
    );
    assert_eq!(super::super::nether_carver_thickness(0.5, 0.25), 2.5);
    assert_eq!(
        nether.replaceable_tag,
        "#minecraft:nether_carver_replaceables"
    );
    assert!(matches!(
        nether.shape,
        CarverShape::Cave {
            floor_level: FloatProvider::Constant(-0.7),
            ..
        }
    ));
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn configured_carver_json_codec_matches_vanilla_registry_files() {
    let dir = super::vanilla_data_path(&["data", "minecraft", "worldgen", "configured_carver"]);
    for id in [
        "minecraft:cave",
        "minecraft:cave_extra_underground",
        "minecraft:canyon",
        "minecraft:nether_cave",
    ] {
        let path = dir.join(format!("{}.json", id.strip_prefix("minecraft:").unwrap()));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"));
        let json: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("failed to parse {path:?}: {err}"));
        let parsed = super::super::parse_configured_carver_from_json(id, &json)
            .unwrap_or_else(|err| panic!("failed to decode {path:?}: {err}"));
        assert_eq!(
            parsed,
            *super::super::configured_carver(id).unwrap(),
            "{id} JSON codec output must match the builtin configured carver"
        );
    }
}

#[test]
fn world_carver_can_reach_and_mask_indices_match_vanilla() {
    assert!(super::super::carver_can_reach(
        8.0, 8.0, 8.0, 8.0, 0, 10, 1.0
    ));
    assert!(super::super::carver_can_reach(
        8.0, 8.0, 30.0, 8.0, 0, 10, 4.0
    ));
    assert!(!super::super::carver_can_reach(
        8.0, 8.0, 80.0, 8.0, 9, 10, 1.0
    ));
    assert_eq!(
        super::super::carver_mask_index(17, -60, 31, -64),
        Some(1_265)
    );
    assert_eq!(
        super::super::carver_mask_position(1_265, 16, 16, -64),
        BlockPos {
            x: 17,
            y: -60,
            z: 31,
        }
    );
}

#[test]
fn carver_block_replacement_and_lava_level_match_vanilla() {
    let cave = super::super::configured_carver("cave").unwrap();
    let nether = super::super::configured_carver("nether_cave").unwrap();
    let height_context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };
    let nether_height_context = WorldGenerationHeightContext {
        min_y: 0,
        height: 128,
    };
    assert!(super::super::carver_can_replace_block(
        cave,
        "minecraft:stone"
    ));
    assert!(!super::super::carver_can_replace_block(
        cave,
        "minecraft:bedrock"
    ));
    assert_eq!(
        super::super::carver_effective_lava_y(cave, height_context),
        -56
    );
    assert_eq!(
        super::super::carver_effective_lava_y(nether, nether_height_context),
        31
    );
}

#[test]
fn overworld_carver_block_outcomes_match_vanilla() {
    let cave = super::super::configured_carver("cave").unwrap();
    let height_context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };

    assert_eq!(
        super::super::carver_carve_block(
            cave,
            height_context,
            super::super::CarverBlockInput {
                pos: BlockPos { x: 1, y: -57, z: 2 },
                block: "minecraft:stone",
                was_masked: false,
                aquifer_state: Some("minecraft:air"),
                should_schedule_fluid_update: false,
                debug_enabled: false,
            },
        )
        .unwrap()
        .state,
        "minecraft:lava"
    );
    assert_eq!(
        super::super::carver_carve_block(
            cave,
            height_context,
            super::super::CarverBlockInput {
                pos: BlockPos { x: 1, y: 60, z: 2 },
                block: "minecraft:stone",
                was_masked: false,
                aquifer_state: Some("minecraft:water"),
                should_schedule_fluid_update: true,
                debug_enabled: false,
            },
        )
        .unwrap(),
        super::super::CarverBlockOutcome {
            pos: BlockPos { x: 1, y: 60, z: 2 },
            state: "minecraft:water",
            mask_index: 31_777,
            mark_postprocessing: true,
        }
    );
    assert!(super::super::carver_carve_block(
        cave,
        height_context,
        super::super::CarverBlockInput {
            pos: BlockPos { x: 1, y: 60, z: 2 },
            block: "minecraft:bedrock",
            was_masked: false,
            aquifer_state: Some("minecraft:air"),
            should_schedule_fluid_update: false,
            debug_enabled: false,
        },
    )
    .is_none());
    assert_eq!(
        super::super::carver_carve_block(
            cave,
            height_context,
            super::super::CarverBlockInput {
                pos: BlockPos { x: 1, y: 60, z: 2 },
                block: "minecraft:bedrock",
                was_masked: true,
                aquifer_state: None,
                should_schedule_fluid_update: false,
                debug_enabled: true,
            },
        )
        .unwrap()
        .state,
        "minecraft:crimson_button"
    );
}

#[test]
fn nether_carver_block_outcomes_match_vanilla() {
    let nether = super::super::configured_carver("nether_cave").unwrap();
    let nether_height_context = WorldGenerationHeightContext {
        min_y: 0,
        height: 128,
    };

    assert_eq!(
        super::super::carver_carve_block(
            nether,
            nether_height_context,
            super::super::CarverBlockInput {
                pos: BlockPos { x: 1, y: 31, z: 2 },
                block: "minecraft:netherrack",
                was_masked: false,
                aquifer_state: Some("minecraft:air"),
                should_schedule_fluid_update: true,
                debug_enabled: false,
            },
        )
        .unwrap(),
        super::super::CarverBlockOutcome {
            pos: BlockPos { x: 1, y: 31, z: 2 },
            state: "minecraft:lava",
            mask_index: 7_969,
            mark_postprocessing: false,
        }
    );
    assert_eq!(
        super::super::carver_carve_block(
            nether,
            nether_height_context,
            super::super::CarverBlockInput {
                pos: BlockPos { x: 1, y: 32, z: 2 },
                block: "minecraft:netherrack",
                was_masked: false,
                aquifer_state: Some("minecraft:water"),
                should_schedule_fluid_update: true,
                debug_enabled: false,
            },
        )
        .unwrap()
        .state,
        "minecraft:cave_air"
    );
}

#[test]
fn carver_ellipsoid_candidate_positions_match_vanilla() {
    let height_context = WorldGenerationHeightContext {
        min_y: -64,
        height: 384,
    };
    let ellipsoid =
        super::super::carver_ellipsoid_candidate_positions(super::super::CarverEllipsoidInput {
            chunk_min_x: 0,
            chunk_min_z: 0,
            height_context,
            upgrading: false,
            x: 8.0,
            y: 64.0,
            z: 8.0,
            horizontal_radius: 2.0,
            vertical_radius: 2.0,
            existing_mask_indices: &[],
            debug_enabled: false,
            skip_model: super::super::CarverSkipModel::Cave { floor_level: -0.7 },
        });
    assert!(ellipsoid.contains(&BlockPos { x: 8, y: 65, z: 8 }));
    assert!(!ellipsoid.contains(&BlockPos { x: 8, y: 62, z: 8 }));
    let masked_index = super::super::carver_mask_index(8, 65, 8, -64).unwrap();
    assert!(!super::super::carver_ellipsoid_candidate_positions(
        super::super::CarverEllipsoidInput {
            chunk_min_x: 0,
            chunk_min_z: 0,
            height_context,
            upgrading: false,
            x: 8.0,
            y: 64.0,
            z: 8.0,
            horizontal_radius: 2.0,
            vertical_radius: 2.0,
            existing_mask_indices: &[masked_index],
            debug_enabled: false,
            skip_model: super::super::CarverSkipModel::Cave { floor_level: -0.7 },
        },
    )
    .contains(&BlockPos { x: 8, y: 65, z: 8 }));
    assert!(super::super::carver_ellipsoid_candidate_positions(
        super::super::CarverEllipsoidInput {
            chunk_min_x: 0,
            chunk_min_z: 0,
            height_context,
            upgrading: false,
            x: 8.0,
            y: 64.0,
            z: 8.0,
            horizontal_radius: 2.0,
            vertical_radius: 2.0,
            existing_mask_indices: &[masked_index],
            debug_enabled: true,
            skip_model: super::super::CarverSkipModel::Cave { floor_level: -0.7 },
        },
    )
    .contains(&BlockPos { x: 8, y: 65, z: 8 }));
    assert!(super::super::carver_ellipsoid_candidate_positions(
        super::super::CarverEllipsoidInput {
            chunk_min_x: 0,
            chunk_min_z: 0,
            height_context,
            upgrading: false,
            x: 100.0,
            y: 64.0,
            z: 8.0,
            horizontal_radius: 2.0,
            vertical_radius: 2.0,
            existing_mask_indices: &[],
            debug_enabled: false,
            skip_model: super::super::CarverSkipModel::None,
        },
    )
    .is_empty());
}

#[test]
fn cave_carver_counts_and_thickness_match_vanilla_sampling() {
    assert_eq!(super::super::cave_carver_cave_count(15, 14, 7, 3), 3);
    assert_eq!(super::super::cave_carver_cave_count(15, 0, 9, 9), 0);
    let mut cave_random = super::super::LegacyRandom::new(12345);
    assert_eq!(
        super::super::sample_cave_carver_cave_count(
            super::super::WorldCarverType::Cave,
            &mut cave_random
        ),
        1
    );
    let mut nether_cave_random = super::super::LegacyRandom::new(8675309);
    assert_eq!(
        super::super::sample_cave_carver_cave_count(
            super::super::WorldCarverType::NetherCave,
            &mut nether_cave_random
        ),
        0
    );
    assert!((super::super::cave_carver_thickness(0.5, 0.25, 1, 1.0, 1.0) - 1.25).abs() < 0.0001);
    assert!((super::super::cave_carver_thickness(0.5, 0.25, 0, 0.5, 0.5) - 2.1875).abs() < 0.0001);
    assert_eq!(super::super::cave_room_radii(2.5, 0.5), (4.0, 2.0));
}

#[test]
fn cave_tunnel_steps_match_vanilla_radius_and_reach_rules() {
    let tunnel_steps = super::super::cave_tunnel_steps(super::super::CaveTunnelInput {
        chunk_middle_x: 8.0,
        chunk_middle_z: 8.0,
        x: 8.0,
        y: 64.0,
        z: 8.0,
        thickness: 2.0,
        horizontal_rotation: 0.0,
        vertical_rotation: 0.0,
        distance: 4,
        y_scale: 1.0,
        horizontal_radius_multiplier: 1.0,
        vertical_radius_multiplier: 1.0,
        random_quarter_skip_rolls: &[1, 1, 1, 1],
        rotation_rolls: &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 4],
    });
    assert_eq!(tunnel_steps.len(), 4);
    assert!(tunnel_steps.iter().all(|step| step.carve && step.can_reach));
    assert!((tunnel_steps[0].x - 9.0).abs() < 0.0001);
    assert!((tunnel_steps[2].horizontal_radius - 3.5).abs() < 0.0001);
}

#[test]
fn cave_tunnel_split_branch_matches_vanilla_rolls() {
    let branch = super::super::cave_tunnel_split_branch(super::super::CaveTunnelSplitInput {
        x: 8.0,
        y: 64.0,
        z: 8.0,
        thickness: 2.0,
        horizontal_rotation: 0.0,
        vertical_rotation: 0.0,
        distance: 8,
        split_roll: 0,
        steep_roll: 1,
        left_thickness_roll: 0.25,
        right_thickness_roll: 0.75,
        rotation_rolls: &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 8],
    })
    .unwrap();
    assert_eq!(branch.split_step, 2);
    assert!((branch.x - 11.0).abs() < 0.0001);
    assert!((branch.left_thickness - 0.625).abs() < 0.0001);
    assert!((branch.right_thickness - 0.875).abs() < 0.0001);
    assert!((branch.left_horizontal_rotation + std::f32::consts::FRAC_PI_2).abs() < 0.0001);
    assert!((branch.right_horizontal_rotation - std::f32::consts::FRAC_PI_2).abs() < 0.0001);
    assert!(
        super::super::cave_tunnel_split_branch(super::super::CaveTunnelSplitInput {
            x: 8.0,
            y: 64.0,
            z: 8.0,
            thickness: 1.0,
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            distance: 8,
            split_roll: 0,
            steep_roll: 1,
            left_thickness_roll: 0.25,
            right_thickness_roll: 0.75,
            rotation_rolls: &[],
        },)
        .is_none()
    );
}

#[test]
fn canyon_tunnel_steps_and_width_factors_match_vanilla() {
    let canyon_steps = super::super::canyon_tunnel_steps(super::super::CanyonTunnelInput {
        chunk_middle_x: 8.0,
        chunk_middle_z: 8.0,
        x: 8.0,
        y: 64.0,
        z: 8.0,
        thickness: 2.0,
        horizontal_rotation: 0.0,
        vertical_rotation: 0.0,
        distance: 4,
        y_scale: 1.0,
        default_vertical_factor: 0.75,
        center_vertical_factor: 1.0,
        random_quarter_skip_rolls: &[1, 1, 1, 1],
        horizontal_radius_factor_rolls: &[0.5, 1.0, 1.5, 2.0],
        vertical_radius_rolls: &[1.0, 1.0, 1.0, 1.0],
        rotation_rolls: &[(0.5, 0.5, 0.0, 0.5, 0.5, 0.0); 4],
    });
    assert_eq!(canyon_steps.len(), 4);
    assert!((canyon_steps[0].horizontal_radius - 0.75).abs() < 0.0001);
    assert!((canyon_steps[2].horizontal_radius - 5.25).abs() < 0.0001);
    assert!((canyon_steps[2].vertical_radius - 6.125).abs() < 0.0001);
    assert_eq!(
        super::super::canyon_width_factors(4, 2, &[(0, 0.5), (1, 1.0), (0, 0.25), (1, 0.0)]),
        vec![1.5625, 1.5625, 1.1289063, 1.1289063]
    );
    assert!((super::super::canyon_vertical_radius(0.75, 1.0, 4.0, 8, 4, 1.0) - 7.0).abs() < 0.0001);
}
