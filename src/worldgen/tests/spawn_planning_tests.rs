use super::*;

fn initial_spawn_input(
    debug_only_half_world: bool,
    debug_world_recreate: bool,
    is_debug: bool,
    generator_spawn_height: i32,
) -> super::super::InitialSpawnPositionInput {
    super::super::InitialSpawnPositionInput {
        debug_only_half_world,
        debug_world_recreate,
        is_debug,
        spawn_chunk: crate::storage::region::ChunkPos { x: 7, z: -3 },
        generator_spawn_height,
        min_y: -64,
        world_surface_height_at_chunk_center: 70,
    }
}

#[test]
fn spawn_selection_constants_and_initial_positions_match_vanilla() {
    assert_eq!(SPAWN_SELECTION_CONSTANTS.initial_chunk_search_radius, 5);
    assert_eq!(SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius, 3);
    assert_eq!(
        SPAWN_SELECTION_CONSTANTS.spawn_search_absolute_max_attempts,
        1024
    );
    assert_eq!(SPAWN_SELECTION_CONSTANTS.large_search_coprime, 17);

    assert_eq!(
        super::super::initial_spawn_position(initial_spawn_input(true, true, false, 64)),
        super::super::InitialSpawnKind::DebugHalfWorld {
            x: 0,
            y: 64,
            z: -100
        }
    );
    assert_eq!(
        super::super::initial_spawn_position(initial_spawn_input(false, false, true, 64)),
        super::super::InitialSpawnKind::DebugWorld { x: 0, y: 80, z: 0 }
    );
    assert_eq!(
        super::super::initial_spawn_position(initial_spawn_input(false, false, false, 90)),
        super::super::InitialSpawnKind::Normal {
            x: 120,
            y: 90,
            z: -40
        }
    );
    assert_eq!(
        super::super::initial_spawn_position(initial_spawn_input(false, false, false, -80)),
        super::super::InitialSpawnKind::Normal {
            x: 120,
            y: 70,
            z: -40
        }
    );
}

#[test]
fn initial_spawn_chunk_spiral_matches_vanilla_search_order() {
    let offsets = super::super::initial_spawn_chunk_spiral_offsets();
    assert_eq!(offsets.len(), 121);
    assert_eq!(
        &offsets[..12],
        &[
            (0, 0),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (2, 0),
            (2, 1)
        ]
    );
    assert_eq!(offsets.last(), Some(&(5, -5)));
    assert!(offsets.contains(&(-5, -5)));
    assert!(offsets.contains(&(5, 5)));
}

#[test]
fn climate_spawn_position_uses_vanilla_two_pass_radial_search() {
    let settings = *super::super::builtin_noise_generator_settings("overworld").unwrap();
    let router = super::super::builtin_noise_router("overworld")
        .unwrap()
        .router;
    let spawn = super::super::climate_spawn_position(settings.spawn_target, router, settings, 0);
    assert_eq!(spawn.y, 0);
    assert_ne!(
        spawn,
        BlockPos { x: 0, y: 0, z: 0 },
        "overworld spawn target search should move away from origin when climate fitness improves"
    );
    assert!(
        spawn.x.abs() <= 2560 && spawn.z.abs() <= 2560,
        "two-pass radial search must stay inside the vanilla 2048+512 search envelope"
    );
    assert_eq!(
        super::super::climate_spawn_position(&[], router, settings, 0),
        BlockPos { x: 0, y: 0, z: 0 }
    );
}

#[test]
fn noise_generator_find_spawn_position_uses_generated_surface_column() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let spawn = super::super::generator_find_spawn_position_for_stem(&normal.overworld, 0)
        .expect("overworld spawn position should resolve");
    let chunk_pos = ChunkPos {
        x: spawn.x.div_euclid(16),
        z: spawn.z.div_euclid(16),
    };
    let chunk = super::super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
        .expect("spawn chunk should generate");
    assert_eq!(
        chunk
            .get_block_state(spawn.x, spawn.y, spawn.z)
            .as_deref()
            .map(super::super::spawn_block_kind),
        Some(SpawnBlockKind::Air),
        "spawn position must be in a non-colliding, non-liquid block"
    );
    assert!(
        matches!(
            chunk
                .get_block_state(spawn.x, spawn.y - 1, spawn.z)
                .as_deref()
                .map(super::super::spawn_block_kind),
            Some(SpawnBlockKind::Solid)
        ),
        "spawn position must stand on a solid generated block"
    );
}

#[test]
fn spawn_original_mobs_plan_matches_noise_generator_gate() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let center = ChunkPos { x: 2, z: -3 };

    let plan = super::super::spawn_original_mobs_plan_for_stem(1234, center, &normal.overworld)
        .expect("overworld noise generator should spawn original mobs");

    assert_eq!(plan.center, center);
    assert_eq!(
        plan.biome_sample_pos,
        BlockPos {
            x: 32,
            y: 320,
            z: -48,
        }
    );
    assert_eq!(
        plan.decoration_seed,
        crate::random_source::decoration_seed(
            1234,
            32,
            -48,
            crate::random_source::RandomAlgorithm::Legacy,
        )
    );
    assert_eq!(
        super::super::spawn_original_mobs_plan_for_stem(1234, center, &normal.end),
        None
    );

    let flat = super::super::resolve_world_preset("flat").unwrap();

    assert_eq!(
        super::super::spawn_original_mobs_plan_for_stem(1234, center, &flat.overworld),
        None
    );
}

#[test]
fn chunk_generation_mob_spawn_plan_matches_creature_selection_order() {
    let plains = super::super::biome_generation_settings("minecraft:plains").unwrap();
    let chunk = ChunkPos { x: 2, z: -3 };
    let mut random = crate::random_source::RandomSourceKind::new(
        4096,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    let plan = super::super::chunk_generation_mob_spawn_plan(chunk, plains, true, &mut random);

    assert_eq!(plan.chunk, chunk);
    assert_eq!(
        plan.batches,
        vec![super::super::ChunkGenerationMobSpawnBatchPlan {
            category: "creature",
            entity_type: "minecraft:pig",
            count: 4,
            start_x: 37,
            start_z: -37,
        }]
    );

    let mut random = crate::random_source::RandomSourceKind::new(
        4096,
        crate::random_source::RandomAlgorithm::Legacy,
    );
    assert_eq!(
        super::super::chunk_generation_mob_spawn_plan(chunk, plains, false, &mut random).batches,
        Vec::new()
    );

    let the_void = super::super::biome_generation_settings("minecraft:the_void").unwrap();
    assert_eq!(
        super::super::chunk_generation_mob_spawn_plan(chunk, the_void, true, &mut random).batches,
        Vec::new()
    );
}

#[test]
fn chunk_generation_mob_spawn_attempt_plan_matches_vanilla_offsets() {
    let chunk = ChunkPos { x: 2, z: -3 };
    let mut random = crate::random_source::RandomSourceKind::new(
        4096,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    assert!(random.next_f32() < 0.1);
    assert_eq!(super::super::random_next_i32_bound(&mut random, 46), 15);
    assert_eq!(super::super::random_next_i32_bound(&mut random, 1), 0);
    let batch = super::super::ChunkGenerationMobSpawnBatchPlan {
        category: "creature",
        entity_type: "minecraft:pig",
        count: 4,
        start_x: 32 + super::super::random_next_i32_bound(&mut random, 16),
        start_z: -48 + super::super::random_next_i32_bound(&mut random, 16),
    };

    let attempts = super::super::chunk_generation_mob_spawn_attempt_plan(chunk, batch, &mut random);

    assert_eq!(batch.start_x, 37);
    assert_eq!(batch.start_z, -37);
    assert_eq!(attempts.len(), 16);
    assert_eq!(
        &attempts[..8],
        &[
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 0,
                attempt_index: 0,
                x: 37,
                z: -37,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 0,
                attempt_index: 1,
                x: 38,
                z: -37,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 0,
                attempt_index: 2,
                x: 40,
                z: -36,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 0,
                attempt_index: 3,
                x: 39,
                z: -35,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 1,
                attempt_index: 0,
                x: 41,
                z: -35,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 1,
                attempt_index: 1,
                x: 38,
                z: -38,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 1,
                attempt_index: 2,
                x: 41,
                z: -40,
            },
            super::super::ChunkGenerationMobSpawnAttemptPlan {
                mob_index: 1,
                attempt_index: 3,
                x: 42,
                z: -40,
            },
        ]
    );
    assert!(attempts
        .iter()
        .all(|attempt| (32..48).contains(&attempt.x) && (-48..-32).contains(&attempt.z)));
}

#[test]
fn chunk_generation_mob_top_non_colliding_pos_uses_spawn_heightmap() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let chunk_pos = ChunkPos { x: 2, z: -3 };
    let chunk = super::super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
        .expect("surface chunk should generate");
    let x = 37;
    let z = -37;

    let plan = super::super::chunk_generation_mob_top_non_colliding_pos(
        &chunk,
        "minecraft:pig",
        x,
        z,
        false,
    );

    let local_x = x.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let height = chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves)
        [local_z * 16 + local_x];
    assert_eq!(plan.entity_type, "minecraft:pig");
    assert_eq!(plan.heightmap, HeightmapKind::MotionBlockingNoLeaves);
    assert_eq!(plan.placement_type, "on_ground");
    assert_eq!(plan.pos, BlockPos { x, y: height, z });
    assert!(
        chunk
            .get_block_state(x, plan.pos.y - 1, z)
            .as_deref()
            .is_some_and(|block| !super::super::is_surface_air(block)),
        "on-ground top position should stand above a non-air block"
    );
}

#[test]
fn chunk_generation_spawn_position_ok_matches_placement_type_primitives() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let chunk_pos = ChunkPos { x: 2, z: -3 };
    let chunk = super::super::generator_build_surface_for_stem(chunk_pos, &normal.overworld)
        .expect("surface chunk should generate");
    let pos = super::super::chunk_generation_mob_top_non_colliding_pos(
        &chunk,
        "minecraft:pig",
        37,
        -37,
        false,
    )
    .pos;

    assert!(super::super::chunk_generation_spawn_position_ok(
        &chunk,
        "minecraft:pig",
        pos
    ));
    assert!(!super::super::chunk_generation_spawn_position_ok(
        &chunk,
        "minecraft:pig",
        BlockPos {
            x: pos.x,
            y: pos.y - 1,
            z: pos.z,
        }
    ));
    assert_eq!(
        super::super::spawn_placement_type("minecraft:squid"),
        "in_water"
    );
    assert_eq!(
        super::super::spawn_placement_type("minecraft:strider"),
        "in_lava"
    );
    assert_eq!(
        super::super::spawn_placement_type("minecraft:fox"),
        "no_restrictions"
    );
    assert_eq!(
        super::super::spawn_placement_heightmap("minecraft:pig"),
        HeightmapKind::MotionBlockingNoLeaves
    );
}

#[test]
fn chunk_generation_mob_entity_snap_plan_clamps_width_and_rolls_yaw() {
    let chunk = ChunkPos { x: 2, z: -3 };
    let mut random = crate::random_source::RandomSourceKind::new(
        1,
        crate::random_source::RandomAlgorithm::Legacy,
    );

    let snap = super::super::chunk_generation_mob_entity_snap_plan(
        chunk,
        "minecraft:pig",
        BlockPos {
            x: 32,
            y: 70,
            z: -33,
        },
        &mut random,
    );

    assert_eq!(snap.entity_type, "minecraft:pig");
    assert_eq!(snap.width, 0.9);
    assert!((snap.x - 32.9).abs() < 0.000001);
    assert_eq!(snap.y, 70.0);
    assert_eq!(snap.z, -33.0);
    assert!((snap.yaw - 263.11615).abs() < 0.0001);
    assert_eq!(snap.pitch, 0.0);
}

#[test]
fn chunk_generation_mob_collision_plan_rejects_solid_blocks_inside_spawn_aabb() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let mut chunk =
        super::super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
            .expect("surface chunk should generate");
    let pos = super::super::chunk_generation_mob_top_non_colliding_pos(
        &chunk,
        "minecraft:pig",
        37,
        -37,
        false,
    )
    .pos;
    let snap = super::super::ChunkGenerationMobEntitySnapPlan {
        entity_type: "minecraft:pig",
        width: 0.9,
        x: 37.0,
        y: f64::from(pos.y),
        z: -37.0,
        yaw: 0.0,
        pitch: 0.0,
    };

    let collision = super::super::chunk_generation_mob_collision_plan(snap);
    assert_eq!(collision.entity_type, "minecraft:pig");
    assert_eq!(collision.width, 0.9);
    assert_eq!(collision.height, 0.9);
    assert!((collision.min_x - 36.55).abs() < 0.000001);
    assert_eq!(collision.min_y, f64::from(pos.y));
    assert!((collision.max_x - 37.45).abs() < 0.000001);
    assert!((collision.max_y - (f64::from(pos.y) + 0.9)).abs() < 0.000001);
    assert!(super::super::chunk_generation_mob_no_collision(
        &chunk, collision
    ));

    chunk.set_block_state(37, pos.y, -37, "minecraft:stone");

    assert!(!super::super::chunk_generation_mob_no_collision(
        &chunk, collision
    ));
}

#[test]
fn chunk_generation_mob_spawn_rules_cover_common_creature_predicates() {
    let normal = super::super::resolve_world_preset("normal").unwrap();
    let mut chunk =
        super::super::generator_build_surface_for_stem(ChunkPos { x: 2, z: -3 }, &normal.overworld)
            .expect("surface chunk should generate");
    let pos = super::super::chunk_generation_mob_top_non_colliding_pos(
        &chunk,
        "minecraft:pig",
        37,
        -37,
        false,
    )
    .pos;

    chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:grass_block");
    assert!(super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:pig",
        pos
    ));
    assert!(super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:rabbit",
        pos
    ));
    assert!(super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:goat",
        pos
    ));
    assert!(!super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:mooshroom",
        pos
    ));

    chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:mycelium");
    assert!(super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:mooshroom",
        pos
    ));
    assert!(!super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:pig",
        pos
    ));

    chunk.set_block_state(pos.x, pos.y - 1, pos.z, "minecraft:stone");
    assert!(super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:goat",
        pos
    ));
    assert!(!super::super::chunk_generation_mob_spawn_rules_ok(
        &chunk,
        "minecraft:rabbit",
        pos
    ));

    chunk.set_block_state(pos.x, pos.y + 1, pos.z, "minecraft:stone");
    assert!(!super::super::chunk_generation_is_bright_enough_to_spawn(
        &chunk, pos
    ));
}
