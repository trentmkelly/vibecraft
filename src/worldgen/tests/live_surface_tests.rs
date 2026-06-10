use super::*;

#[test]
fn chunk_biome_storage_is_depth_aware_at_every_quart_position() {
    // Regression: the chunk-biome populator used to bypass depth on the
    // overworld via a column-only shortcut (`overworld_column_biomes`
    // computed with `depth = 0.0`). The surface generator's biome
    // resolver, in contrast, always sampled the depth-aware climate when
    // the fiddled-distance lookup landed on a neighbour-chunk quart —
    // which is what happens for the 1-block-wide strip on every chunk
    // edge. The result was a terracotta outline around every chunk in the
    // badlands biome and a similar sand outline in the beach biome.
    //
    // This test exercises the full chunk pipeline and asserts that the
    // stored biome at every quart matches what `get_biome` (the
    // depth-aware reference) reports for the same `(quart_x, quart_y,
    // quart_z)`. If they agree, the cache-hit and cache-miss paths of the
    // surface biome resolver always agree and the chunk-border outline
    // cannot reappear.
    let pos = ChunkPos { x: 4, z: 4 };
    let seed = 0;
    let chunk = super::super::generate_overworld_chunk_for_preset_with_mode(
        pos,
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        seed,
    )
    .expect("real-surface chunk should generate for biome storage check");

    let settings = *super::super::builtin_noise_generator_settings("overworld").unwrap();
    let router =
        super::super::builtin_noise_router(super::super::noise_router_id_for_settings(settings))
            .expect("normal overworld must have a router")
            .router;
    let climate_sampler = super::super::ClimateSampler::from_noise_router(&router, seed, settings);
    let biome_source = super::super::BiomeSourceModel::MultiNoisePreset {
        preset: "minecraft:overworld",
    };

    let stored = super::super::ChunkNoiseBiomeCache::from_chunk(&chunk);
    let min_section_y = chunk.min_section_y;
    let max_section_y = chunk
        .sections
        .iter()
        .map(|section| i32::from(section.y))
        .max()
        .unwrap_or(min_section_y);

    let mut mismatches: Vec<(i32, i32, i32, &'static str, &'static str)> = Vec::new();
    for section_y in min_section_y..=max_section_y {
        for local_y in 0..4_i32 {
            let quart_y = section_y * 4 + local_y;
            for local_z in 0..4_i32 {
                let quart_z = pos.z * 4 + local_z;
                for local_x in 0..4_i32 {
                    let quart_x = pos.x * 4 + local_x;
                    let Some(stored_biome) = stored.get(quart_x, quart_y, quart_z) else {
                        continue;
                    };
                    let reference = super::super::get_biome(
                        &biome_source,
                        quart_x,
                        quart_y,
                        quart_z,
                        &climate_sampler,
                    )
                    .unwrap_or("minecraft:plains");
                    if stored_biome != reference {
                        mismatches.push((quart_x, quart_y, quart_z, stored_biome, reference));
                    }
                }
            }
        }
    }

    if !mismatches.is_empty() {
        for (qx, qy, qz, stored_biome, reference) in mismatches.iter().take(10) {
            eprintln!(
                "[chunk-biome-mismatch] quart=({qx}, {qy}, {qz}) stored={stored_biome} \
                 reference={reference}"
            );
        }
        panic!(
            "{} chunk-biome quart(s) disagreed with the depth-aware reference \
             — this regresses the badlands/beach chunk-outline fix",
            mismatches.len()
        );
    }
}

#[test]
fn real_surface_generation_mode_uses_noise_and_surface_pipeline() {
    let chunk = super::super::generate_overworld_chunk_for_preset_with_mode(
        ChunkPos { x: 0, z: 0 },
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        0,
    )
    .expect("real-surface mode should generate overworld surface terrain");

    assert_eq!(chunk.status, "minecraft:surface");
    assert_eq!(chunk.min_section_y, -4);
    assert_eq!(chunk.sections.len(), 24);
    assert_eq!(chunk.sections[0].y, -4);
    assert_eq!(chunk.sections.last().unwrap().y, 19);
    assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
    assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));
    assert!(chunk.heightmaps.contains_key("WORLD_SURFACE"));
    assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING"));
    assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING_NO_LEAVES"));
    let packet_data = crate::network::play::ClientboundLevelChunkPacketData::from_chunk(&chunk);
    assert!(packet_data.heightmaps.contains_key("WORLD_SURFACE"));
    assert!(packet_data.heightmaps.contains_key("MOTION_BLOCKING"));
    assert!(packet_data
        .heightmaps
        .contains_key("MOTION_BLOCKING_NO_LEAVES"));
    assert!(
        (-64..320).any(|y| {
            chunk
                .get_block_state(0, y, 0)
                .is_some_and(|name| name != "minecraft:air")
        }),
        "real-surface chunk should contain non-air blocks in the origin column"
    );
}

#[test]
fn real_surface_generation_places_initial_tree_decoration() {
    let tree_logs = [
        "minecraft:oak_log",
        "minecraft:birch_log",
        "minecraft:spruce_log",
        "minecraft:acacia_log",
        "minecraft:jungle_log",
    ];
    let tree_leaves = [
        "minecraft:oak_leaves",
        "minecraft:birch_leaves",
        "minecraft:spruce_leaves",
        "minecraft:acacia_leaves",
        "minecraft:jungle_leaves",
    ];

    let mut found_logs = 0;
    let mut found_leaves = 0;
    'chunks: for chunk_z in -4..=4 {
        for chunk_x in -4..=4 {
            let pos = ChunkPos {
                x: chunk_x,
                z: chunk_z,
            };
            let chunk = super::super::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                super::super::LiveChunkGenerationMode::RealSurface,
                0,
            )
            .expect("real-surface chunk generation should succeed while scanning for trees");

            let chunk_min_x = pos.x * 16;
            let chunk_min_z = pos.z * 16;
            for local_z in 0..16 {
                for local_x in 0..16 {
                    for y in -64..320 {
                        let Some(state) =
                            chunk.get_block_state(chunk_min_x + local_x, y, chunk_min_z + local_z)
                        else {
                            continue;
                        };
                        if tree_logs.contains(&state.as_str()) {
                            found_logs += 1;
                        } else if tree_leaves.contains(&state.as_str()) {
                            found_leaves += 1;
                        }
                        if found_logs > 0 && found_leaves > 0 {
                            break 'chunks;
                        }
                    }
                }
            }
        }
    }

    assert!(
            found_logs > 0 && found_leaves > 0,
            "real-surface generation should place visible tree logs and leaves near spawn; found_logs={found_logs}, found_leaves={found_leaves}"
        );
}

#[test]
fn live_blob_tree_foliage_consumes_java_corner_rolls() {
    let seed = 12345;
    let origin = BlockPos { x: 8, y: 64, z: 8 };
    let trunk = TrunkPlacerModel {
        base_height: 4,
        height_rand_a: 2,
        height_rand_b: 0,
        kind: TrunkPlacerKind::Straight,
    };
    let mut random = crate::random_source::RandomSourceKind::new(
        seed,
        crate::random_source::RandomAlgorithm::Xoroshiro,
    );
    let mut expected_random = crate::random_source::RandomSourceKind::new(
        seed,
        crate::random_source::RandomAlgorithm::Xoroshiro,
    );
    for _ in 0..16 {
        super::super::random_next_i32_bound(&mut expected_random, 2);
    }

    let plan = super::super::live_straight_blob_tree_placement_plan(
        super::super::LiveStraightBlobTreeInput {
            origin,
            trunk,
            clipped_tree_height: super::super::trunk_placer_height(trunk, 1, 0),
            foliage: FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Blob { height: 3 },
            },
            trunk_state: "minecraft:oak_log",
            foliage_state: "minecraft:oak_leaves",
            below_trunk_state: "minecraft:dirt",
            rand_a: 1,
            rand_b: 0,
        },
        &mut random,
    )
    .expect("live straight blob tree should plan");

    assert_eq!(
            super::super::random_next_i32_bound(&mut random, 10_000),
            super::super::random_next_i32_bound(&mut expected_random, 10_000),
            "BlobFoliagePlacer checks four signed corners across four rows and consumes one random roll for each corner"
        );
    assert!(!plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: origin.x - 1,
                    y: origin.y + 5,
                    z: origin.z - 1,
                }
    }));

    let top_leaf_count = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Leaves && block.pos.y == origin.y + 5)
        .count();
    assert_eq!(
        top_leaf_count, 5,
        "BlobFoliagePlacer top row still consumes corner rolls, but y=0 skips every corner"
    );
}

#[test]
fn live_blob_tree_uses_clipped_height_for_trunk_and_foliage_origin() {
    let origin = BlockPos { x: 8, y: 64, z: 8 };
    let trunk = TrunkPlacerModel {
        base_height: 4,
        height_rand_a: 2,
        height_rand_b: 0,
        kind: TrunkPlacerKind::Straight,
    };
    let clipped_tree_height = 3;
    let mut random = crate::random_source::RandomSourceKind::new(
        12345,
        crate::random_source::RandomAlgorithm::Xoroshiro,
    );

    let plan = super::super::live_straight_blob_tree_placement_plan(
        super::super::LiveStraightBlobTreeInput {
            origin,
            trunk,
            clipped_tree_height,
            foliage: FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Blob { height: 3 },
            },
            trunk_state: "minecraft:oak_log",
            foliage_state: "minecraft:oak_leaves",
            below_trunk_state: "minecraft:dirt",
            rand_a: 1,
            rand_b: 0,
        },
        &mut random,
    )
    .expect("live straight blob tree should plan");

    let trunk_positions = plan
        .blocks
        .iter()
        .filter(|block| block.kind == TreePlacementBlockKind::Log)
        .map(|block| block.pos)
        .collect::<Vec<_>>();
    assert_eq!(
        trunk_positions,
        vec![
            origin,
            BlockPos {
                x: origin.x,
                y: origin.y + 1,
                z: origin.z,
            },
            BlockPos {
                x: origin.x,
                y: origin.y + 2,
                z: origin.z,
            },
        ],
        "Java TreeFeature passes clippedTreeHeight into TrunkPlacer.placeTrunk"
    );
    assert!(
        plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos.y == origin.y + clipped_tree_height
        }),
        "Java TreeFeature passes clippedTreeHeight into FoliagePlacer.createFoliage"
    );
}

#[test]
fn live_tree_configs_keep_vanilla_decorator_sets() {
    assert_eq!(
        super::super::live_birch_tree_config().base_height,
        5,
        "Java birch/birch_bees_0002 trunk placer uses base_height=5, not oak's base_height=4"
    );
    assert!(matches!(
        super::super::live_oak_leaf_litter_tree_config().decorators,
        super::super::LiveTreeDecoratorSet::BeesAndLeafLitter {
            probability_per_tree: 2_000
        }
    ));
    assert!(matches!(
        super::super::live_birch_tree_config().decorators,
        super::super::LiveTreeDecoratorSet::Bees {
            probability_per_tree: 2_000
        }
    ));
    assert!(matches!(
        super::super::live_oak_bees_005_tree_config().decorators,
        super::super::LiveTreeDecoratorSet::Bees {
            probability_per_tree: 50_000
        }
    ));
}

#[test]
#[ignore = "wall-clock performance guard; run explicitly after worldgen optimization changes"]
fn real_surface_spawn_chunk_generation_stays_under_debug_budget() {
    let max_ms = std::env::var("RUSTCRAFT_WORLDGEN_CHUNK_MAX_MS")
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or(4);
    let seed = std::env::var("RUSTCRAFT_WORLDGEN_TEST_SEED")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let pos = ChunkPos { x: 0, z: 0 };

    let started = std::time::Instant::now();
    let (chunk, timings) = super::super::generate_overworld_spawn_chunk_for_preset_with_mode_timed(
        pos,
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        seed,
        true,
    )
    .expect("real-surface spawn chunk generation should succeed");
    let elapsed_ms = started.elapsed().as_millis();

    assert_eq!(chunk.status, "minecraft:spawn");
    assert!(
        (-64..320).any(|y| {
            chunk
                .get_block_state(0, y, 0)
                .is_some_and(|name| name != "minecraft:air")
        }),
        "generated chunk should contain non-air blocks in the origin column"
    );
    eprintln!(
            "[worldgen-perf-test] chunk=({}, {}) elapsed={}ms threshold={}ms target=4ms/chunk terrain={}ms base_generation={}ms region_biome_steps={}ms carvers={}ms/{}blocks underground_structures={}ms/{}blocks ore_decoration={}ms/{}blocks tree_context={}ms/{}chunks tree_decoration={}ms/{}blocks fill={}ms fill_init_sections={}ms fill_noise_chunk_init={}ms fill_aquifer_init={}ms fill_block_loop={}ms full_noise_cache={}us/{}fills biome_storage={}ms fill_density_lookup={}us fill_aquifer_compute={}us fill_ore_vein_lookup={}us fill_ore_decision={}us fill_interpolation_update={}us interpolators={} surface={}ms heightmaps={}ms mobs={}ms mob_plan={}ms mob_apply={}ms block_writes={} aquifer_calls={} ore_vein_samples={}",
            pos.x,
            pos.z,
            elapsed_ms,
            max_ms,
            timings.terrain_ms,
            timings.base_generation_ms,
            timings.region_biome_steps_ms,
            timings.carvers_ms,
            timings.carver_blocks,
            timings.underground_structures_ms,
            timings.underground_structure_blocks,
            timings.ore_decoration_ms,
            timings.ore_blocks,
            timings.tree_context_ms,
            timings.tree_context_chunks,
            timings.tree_decoration_ms,
            timings.tree_blocks,
            timings.terrain.fill_total_ms,
            timings.terrain.fill_init_sections_ms,
            timings.terrain.fill_noise_chunk_init_ms,
            timings.terrain.fill_aquifer_init_ms,
            timings.terrain.fill_block_loop_ms,
            timings.terrain.fill_full_noise_cache_us,
            timings.terrain.full_noise_cache_fills,
            timings.terrain.biome_storage_ms,
            timings.terrain.fill_density_lookup_us,
            timings.terrain.fill_aquifer_compute_us,
            timings.terrain.fill_ore_vein_lookup_us,
            timings.terrain.fill_ore_decision_us,
            timings.terrain.fill_interpolation_update_us,
            timings.terrain.interpolator_count,
            timings.terrain.surface_total_ms,
            timings.heightmaps.total_ms,
            timings.mobs.total_ms,
            timings.mobs.plan_ms,
            timings.mobs.apply_batches_ms,
            timings.terrain.block_writes,
            timings.terrain.aquifer_calls,
            timings.terrain.ore_vein_samples
        );
    assert!(
            elapsed_ms <= max_ms,
            "real-surface spawn chunk generation took {elapsed_ms}ms, above {max_ms}ms budget (~4ms/chunk Java target); timings={timings:?}"
        );
}

#[test]
#[ignore = "diagnostic for Java-shaped region feature generation throughput"]
fn real_surface_region_spawn_chunk_generation_diagnostic() {
    let seed = std::env::var("RUSTCRAFT_WORLDGEN_TEST_SEED")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let radius = std::env::var("RUSTCRAFT_WORLDGEN_REGION_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1);
    let center = ChunkPos { x: 0, z: 0 };
    let started = std::time::Instant::now();
    let chunks = super::super::generate_overworld_spawn_chunk_region_for_preset_with_mode(
        center,
        radius,
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        seed,
        true,
    )
    .expect("region real-surface generation should succeed");
    let elapsed_ms = started.elapsed().as_millis();
    let chunk_count = chunks.len().max(1);
    let center_chunk = chunks
        .get(&center)
        .expect("region generation should include center chunk");
    assert_eq!(center_chunk.status, "minecraft:spawn");
    eprintln!(
            "[worldgen-region-perf-test] center=({}, {}) radius={} chunks={} elapsed={}ms avg_per_chunk={:.3}ms",
            center.x,
            center.z,
            radius,
            chunks.len(),
            elapsed_ms,
            elapsed_ms as f64 / chunk_count as f64
        );
}

#[test]
#[ignore = "diagnostic for live multiplayer terrain reports"]
fn live_seed_chunk_surface_summary() {
    let seed = std::env::var("RUSTCRAFT_WORLDGEN_TEST_SEED")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let center_x = std::env::var("RUSTCRAFT_WORLDGEN_TEST_CHUNK_X")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);
    let center_z = std::env::var("RUSTCRAFT_WORLDGEN_TEST_CHUNK_Z")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);
    let radius = std::env::var("RUSTCRAFT_WORLDGEN_TEST_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    for chunk_z in center_z - radius..=center_z + radius {
        for chunk_x in center_x - radius..=center_x + radius {
            let pos = ChunkPos {
                x: chunk_x,
                z: chunk_z,
            };
            let (chunk, _timings) =
                super::super::generate_overworld_spawn_chunk_for_preset_with_mode_timed(
                    pos,
                    "normal",
                    super::super::LiveChunkGenerationMode::RealSurface,
                    seed,
                    true,
                )
                .expect("live seed diagnostic chunk should generate");

            let mut min_top = i32::MAX;
            let mut max_top = i32::MIN;
            let mut top_blocks = BTreeMap::<String, usize>::new();
            for local_z in 0..16 {
                for local_x in 0..16 {
                    let world_x = chunk_x * 16 + local_x;
                    let world_z = chunk_z * 16 + local_z;
                    let mut top = -64;
                    let mut top_block = "minecraft:air".to_string();
                    for y in (-64..320).rev() {
                        let block = chunk
                            .get_block_state(world_x, y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if block != "minecraft:air" {
                            top = y;
                            top_block = block;
                            break;
                        }
                    }
                    min_top = min_top.min(top);
                    max_top = max_top.max(top);
                    *top_blocks.entry(top_block).or_insert(0) += 1;
                }
            }
            eprintln!(
                "[live-seed-surface-summary] seed={} chunk=({}, {}) top_y={}..{} top_blocks={:?}",
                seed, chunk_x, chunk_z, min_top, max_top, top_blocks
            );
        }
    }
}

#[test]
#[ignore = "wall-clock performance guard; run explicitly after worldgen optimization changes"]
fn real_surface_spawn_area_generation_stays_under_debug_budget() {
    let default_chunk_ms = 4_u128;
    let chunk_count = 9_u128;
    let max_total_ms = std::env::var("RUSTCRAFT_WORLDGEN_SPAWN_AREA_MAX_MS")
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or(default_chunk_ms * chunk_count);
    let max_chunk_ms = std::env::var("RUSTCRAFT_WORLDGEN_CHUNK_MAX_MS")
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or(default_chunk_ms);
    let seed = std::env::var("RUSTCRAFT_WORLDGEN_TEST_SEED")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);

    let started = std::time::Instant::now();
    let mut slowest_chunk = (ChunkPos { x: 0, z: 0 }, 0_u128);
    let mut generated = 0_usize;
    for chunk_z in -1..=1 {
        for chunk_x in -1..=1 {
            let pos = ChunkPos {
                x: chunk_x,
                z: chunk_z,
            };
            let chunk_started = std::time::Instant::now();
            let (chunk, timings) =
                super::super::generate_overworld_spawn_chunk_for_preset_with_mode_timed(
                    pos,
                    "normal",
                    super::super::LiveChunkGenerationMode::RealSurface,
                    seed,
                    true,
                )
                .expect("real-surface spawn-area chunk generation should succeed");
            let chunk_elapsed_ms = chunk_started.elapsed().as_millis();
            generated += 1;
            if chunk_elapsed_ms > slowest_chunk.1 {
                slowest_chunk = (pos, chunk_elapsed_ms);
            }

            assert_eq!(chunk.status, "minecraft:spawn");
            let local_x = if chunk_x == 0 { 0 } else { 8 };
            let local_z = if chunk_z == 0 { 0 } else { 8 };
            assert!(
                    (-64..320).any(|y| {
                        chunk
                            .get_block_state(local_x, y, local_z)
                            .is_some_and(|name| name != "minecraft:air")
                    }),
                    "generated chunk ({chunk_x}, {chunk_z}) should contain non-air blocks in its sampled column"
                );
            assert!(
                    chunk_elapsed_ms <= max_chunk_ms,
                    "real-surface spawn-area chunk ({chunk_x}, {chunk_z}) took {chunk_elapsed_ms}ms, above {max_chunk_ms}ms budget; timings={timings:?}"
                );
        }
    }
    let elapsed_ms = started.elapsed().as_millis();

    eprintln!(
            "[worldgen-spawn-area-perf-test] chunks={} elapsed={}ms threshold={}ms target=4ms/chunk slowest_chunk=({}, {}) slowest_ms={}",
            generated,
            elapsed_ms,
            max_total_ms,
            slowest_chunk.0.x,
            slowest_chunk.0.z,
            slowest_chunk.1
        );
    assert!(
            elapsed_ms <= max_total_ms,
            "real-surface 3x3 spawn-area generation took {elapsed_ms}ms, above {max_total_ms}ms budget (~4ms/chunk Java target); slowest_chunk={slowest_chunk:?}"
        );
}

#[test]
#[ignore = "diagnostic; run explicitly when inspecting visible chunk-border terrain artifacts"]
fn real_surface_adjacent_chunk_border_continuity_diagnostic() {
    let seed = std::env::var("RUSTCRAFT_WORLDGEN_TEST_SEED")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let radius = std::env::var("RUSTCRAFT_WORLDGEN_BORDER_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1);
    let (chunks, total_generation_ms) = generate_border_diagnostic_chunks(seed, radius);
    let mut samples = collect_border_samples(&chunks, radius);

    samples.sort_by_key(|sample| {
        std::cmp::Reverse(
            (sample.height_a - sample.height_b)
                .abs()
                .max((sample.terrain_height_a - sample.terrain_height_b).abs()),
        )
    });
    print_border_diagnostic(seed, radius, chunks.len(), total_generation_ms, &samples);
}

#[derive(Debug)]
struct EdgeSample {
    axis: &'static str,
    chunk_a: ChunkPos,
    chunk_b: ChunkPos,
    world_x_a: i32,
    world_z_a: i32,
    world_x_b: i32,
    world_z_b: i32,
    height_a: i32,
    height_b: i32,
    terrain_height_a: i32,
    terrain_height_b: i32,
    top_a: String,
    top_b: String,
}

fn generate_border_diagnostic_chunks(
    seed: i64,
    radius: i32,
) -> (BTreeMap<(i32, i32), LevelChunk>, u128) {
    let mut chunks = BTreeMap::<(i32, i32), LevelChunk>::new();
    let mut total_generation_ms = 0_u128;
    for chunk_z in -radius..=radius {
        for chunk_x in -radius..=radius {
            let pos = ChunkPos {
                x: chunk_x,
                z: chunk_z,
            };
            let started = std::time::Instant::now();
            let chunk = super::super::generate_overworld_spawn_chunk_for_preset_with_mode(
                pos,
                "normal",
                super::super::LiveChunkGenerationMode::RealSurface,
                seed,
                true,
            )
            .expect("real-surface diagnostic chunk generation should succeed");
            total_generation_ms += started.elapsed().as_millis();
            chunks.insert((chunk_x, chunk_z), chunk);
        }
    }
    (chunks, total_generation_ms)
}

fn collect_border_samples(
    chunks: &BTreeMap<(i32, i32), LevelChunk>,
    radius: i32,
) -> Vec<EdgeSample> {
    let mut samples = Vec::<EdgeSample>::new();
    for chunk_z in -radius..=radius {
        for chunk_x in -radius..=radius {
            let Some(chunk) = chunks.get(&(chunk_x, chunk_z)) else {
                continue;
            };
            if let Some(east) = chunks.get(&(chunk_x + 1, chunk_z)) {
                push_east_border_samples(&mut samples, chunk_x, chunk_z, chunk, east);
            }
            if let Some(south) = chunks.get(&(chunk_x, chunk_z + 1)) {
                push_south_border_samples(&mut samples, chunk_x, chunk_z, chunk, south);
            }
        }
    }
    samples
}

fn push_east_border_samples(
    samples: &mut Vec<EdgeSample>,
    chunk_x: i32,
    chunk_z: i32,
    chunk: &LevelChunk,
    east: &LevelChunk,
) {
    for local_z in 0..16_i32 {
        let world_z = chunk_z * 16 + local_z;
        let world_x_a = chunk_x * 16 + 15;
        let world_x_b = (chunk_x + 1) * 16;
        let (height_a, terrain_height_a, top_a) = border_top_block(chunk, world_x_a, world_z);
        let (height_b, terrain_height_b, top_b) = border_top_block(east, world_x_b, world_z);
        samples.push(EdgeSample {
            axis: "x",
            chunk_a: ChunkPos {
                x: chunk_x,
                z: chunk_z,
            },
            chunk_b: ChunkPos {
                x: chunk_x + 1,
                z: chunk_z,
            },
            world_x_a,
            world_z_a: world_z,
            world_x_b,
            world_z_b: world_z,
            height_a,
            height_b,
            terrain_height_a,
            terrain_height_b,
            top_a,
            top_b,
        });
    }
}

fn push_south_border_samples(
    samples: &mut Vec<EdgeSample>,
    chunk_x: i32,
    chunk_z: i32,
    chunk: &LevelChunk,
    south: &LevelChunk,
) {
    for local_x in 0..16_i32 {
        let world_x = chunk_x * 16 + local_x;
        let world_z_a = chunk_z * 16 + 15;
        let world_z_b = (chunk_z + 1) * 16;
        let (height_a, terrain_height_a, top_a) = border_top_block(chunk, world_x, world_z_a);
        let (height_b, terrain_height_b, top_b) = border_top_block(south, world_x, world_z_b);
        samples.push(EdgeSample {
            axis: "z",
            chunk_a: ChunkPos {
                x: chunk_x,
                z: chunk_z,
            },
            chunk_b: ChunkPos {
                x: chunk_x,
                z: chunk_z + 1,
            },
            world_x_a: world_x,
            world_z_a,
            world_x_b: world_x,
            world_z_b,
            height_a,
            height_b,
            terrain_height_a,
            terrain_height_b,
            top_a,
            top_b,
        });
    }
}

fn border_top_block(chunk: &LevelChunk, world_x: i32, world_z: i32) -> (i32, i32, String) {
    let local_x = world_x.rem_euclid(16) as usize;
    let local_z = world_z.rem_euclid(16) as usize;
    let height = chunk
        .heightmap_value(HeightmapKind::WorldSurface, local_x, local_z)
        .or_else(|| chunk.heightmap_value(HeightmapKind::WorldSurfaceWg, local_x, local_z))
        .unwrap_or(0);
    let terrain_height = chunk
        .heightmap_value(HeightmapKind::MotionBlockingNoLeaves, local_x, local_z)
        .or_else(|| chunk.heightmap_value(HeightmapKind::OceanFloorWg, local_x, local_z))
        .unwrap_or(0);
    let top_y = height - 1;
    let top = chunk
        .get_block_state(world_x, top_y, world_z)
        .unwrap_or_else(|| "minecraft:air".to_string());
    (height, terrain_height, top)
}

fn print_border_diagnostic(
    seed: i64,
    radius: i32,
    chunk_count: usize,
    total_generation_ms: u128,
    samples: &[EdgeSample],
) {
    let stats = border_sample_stats(samples);
    eprintln!(
            "[worldgen-border-diagnostic] seed={} radius={} chunks={} generation_total={}ms samples={} max_height_delta={} avg_height_delta={:.3} high_delta_ge_8={} max_no_leaves_delta={} avg_no_leaves_delta={:.3} high_no_leaves_delta_ge_8={} top_block_mismatches={}",
            seed,
            radius,
            chunk_count,
            total_generation_ms,
            samples.len(),
            stats.max_delta,
            stats.average_delta,
            stats.high_delta_count,
            stats.max_terrain_delta,
            stats.average_terrain_delta,
            stats.high_terrain_delta_count,
            stats.material_mismatch_count
        );

    for sample in samples.iter().take(16) {
        print_border_sample(sample);
    }
}

struct BorderSampleStats {
    max_delta: i32,
    average_delta: f64,
    high_delta_count: usize,
    max_terrain_delta: i32,
    average_terrain_delta: f64,
    high_terrain_delta_count: usize,
    material_mismatch_count: usize,
}

fn border_sample_stats(samples: &[EdgeSample]) -> BorderSampleStats {
    let total_delta: i32 = samples
        .iter()
        .map(|sample| (sample.height_a - sample.height_b).abs())
        .sum();
    let total_terrain_delta: i32 = samples
        .iter()
        .map(|sample| (sample.terrain_height_a - sample.terrain_height_b).abs())
        .sum();
    BorderSampleStats {
        max_delta: samples
            .first()
            .map(|sample| (sample.height_a - sample.height_b).abs())
            .unwrap_or(0),
        average_delta: sample_average(total_delta, samples.len()),
        high_delta_count: samples
            .iter()
            .filter(|sample| (sample.height_a - sample.height_b).abs() >= 8)
            .count(),
        max_terrain_delta: samples
            .iter()
            .map(|sample| (sample.terrain_height_a - sample.terrain_height_b).abs())
            .max()
            .unwrap_or(0),
        average_terrain_delta: sample_average(total_terrain_delta, samples.len()),
        high_terrain_delta_count: samples
            .iter()
            .filter(|sample| (sample.terrain_height_a - sample.terrain_height_b).abs() >= 8)
            .count(),
        material_mismatch_count: samples
            .iter()
            .filter(|sample| sample.top_a != sample.top_b)
            .count(),
    }
}

fn sample_average(total: i32, sample_count: usize) -> f64 {
    if sample_count == 0 {
        0.0
    } else {
        f64::from(total) / sample_count as f64
    }
}

fn print_border_sample(sample: &EdgeSample) {
    eprintln!(
            "[worldgen-border-diagnostic-sample] axis={} chunks=({},{})->({},{}) a=({}, {}) h={} no_leaves_h={} top={} b=({}, {}) h={} no_leaves_h={} top={} delta={} no_leaves_delta={}",
            sample.axis,
            sample.chunk_a.x,
            sample.chunk_a.z,
            sample.chunk_b.x,
            sample.chunk_b.z,
            sample.world_x_a,
            sample.world_z_a,
            sample.height_a,
            sample.terrain_height_a,
            sample.top_a,
            sample.world_x_b,
            sample.world_z_b,
            sample.height_b,
            sample.terrain_height_b,
            sample.top_b,
            (sample.height_a - sample.height_b).abs(),
            (sample.terrain_height_a - sample.terrain_height_b).abs()
        );
}
