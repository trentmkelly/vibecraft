use super::*;

#[test]
    fn lightweight_tree_context_exposes_dry_surface_as_vegetation_support() {
        let mut heights = super::super::TreeDecorationHeights {
            ocean_floor: [64; 16 * 16],
            world_surface: [64; 16 * 16],
            motion_blocking: [64; 16 * 16],
            motion_blocking_no_leaves: [64; 16 * 16],
        };
        let wet_column_index = 16;
        heights.ocean_floor[wet_column_index] = 62;
        heights.world_surface[wet_column_index] = 64;
        heights.motion_blocking[wet_column_index] = 64;
        heights.motion_blocking_no_leaves[wet_column_index] = 62;

        let chunk = super::super::LightweightTreeContextChunk {
            terrain_heights: heights,
            min_y: -64,
            max_y: 320,
        };

        assert_eq!(
            chunk.synthetic_block_state(0, 63, 0),
            "minecraft:grass_block"
        );
        assert_eq!(chunk.synthetic_block_state(0, 62, 0), "minecraft:stone");
        assert_eq!(chunk.synthetic_block_state(0, 63, 1), "minecraft:water");
        assert_eq!(chunk.synthetic_block_state(0, 61, 1), "minecraft:stone");
    }

    #[test]
    #[ignore = "diagnostic parity stocktake; run explicitly while aligning worldgen stages"]
    fn normal_overworld_stage_parity_stocktake() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("vanilla fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut base_chunks = Vec::new();
        let mut carver_chunks = Vec::new();
        let mut structure_chunks = Vec::new();
        let mut ore_chunks = Vec::new();
        let mut full_chunks = Vec::new();
        for chunk in chunks {
            let pos = ChunkPos {
                x: chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut carver = base.clone();
            super::super::apply_configured_carvers_for_biome_source(
                &mut carver,
                biome_source_model,
                noise_settings,
                seed,
            );
            let mut structure = carver.clone();
            super::super::apply_mineshaft_underground_structures_to_chunk(&mut structure, seed);
            let mut ore = structure.clone();
            super::super::apply_underground_ore_decoration_to_chunk(
                &mut ore,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            let mut full = ore.clone();
            super::super::apply_initial_tree_decoration_to_chunk(
                &mut full,
                biome_source_model,
                noise_settings,
                seed,
                None,
                None,
                None,
            );
            base_chunks.push(base);
            carver_chunks.push(carver);
            structure_chunks.push(structure);
            ore_chunks.push(ore);
            full_chunks.push(full);
        }

        for (label, chunks) in [
            ("noise_surface", &base_chunks),
            ("carvers", &carver_chunks),
            ("structures", &structure_chunks),
            ("ores", &ore_chunks),
            ("trees", &full_chunks),
        ] {
            let block = crate::worldgen_comparison::vanilla_worldgen_block_array_parity_score(
                fixture_json,
                chunks,
            )
            .expect("block parity score should compute");
            let heightmap = crate::worldgen_comparison::vanilla_worldgen_heightmap_parity_score(
                fixture_json,
                chunks,
            )
            .expect("heightmap parity score should compute");
            let column = crate::worldgen_comparison::vanilla_worldgen_column_profile_parity_score(
                fixture_json,
                chunks,
            )
            .expect("column-profile parity score should compute");
            eprintln!(
                "[worldgen-stage-parity] stage={} block={:.6} ({}/{}) heightmap={:.6} ({}/{}) column={:.6} ({}/{})",
                label,
                block.score,
                block.matching_blocks,
                block.total_blocks,
                heightmap.score,
                heightmap.matching_columns,
                heightmap.total_columns,
                column.score,
                column.matching_columns,
                column.total_columns
            );
            for mismatch in block.mismatches.iter().take(8) {
                eprintln!(
                    "[worldgen-stage-parity-mismatch] stage={} count={} expected={} actual={}",
                    label, mismatch.count, mismatch.expected, mismatch.actual
                );
            }
        }
    }

    #[test]
    #[ignore = "diagnostic for source-chunk decoration biome sets and scheduled features"]
    fn normal_overworld_decoration_source_feature_schedule_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        let router =
            super::super::builtin_noise_router(super::super::noise_router_id_for_settings(**noise_settings))
                .expect("normal overworld should have a built-in noise router")
                .router;
        let climate_sampler =
            super::super::ClimateSampler::from_noise_router(&router, seed, **noise_settings);
        let global_biome_steps = super::super::possible_biome_feature_steps_for_source(biome_source_model);
        let features_per_step = super::super::build_features_per_step(&global_biome_steps, true)
            .expect("global features should sort");
        let min_section_y = noise_settings.noise.min_y.div_euclid(16);
        let sample_quart_y = ((noise_settings.sea_level + 1).clamp(
            noise_settings.noise.min_y,
            noise_settings.noise.min_y + noise_settings.noise.height - 1,
        )) >> 2;

        for fixture_chunk in chunks.iter().take(4) {
            let target_pos = ChunkPos {
                x: fixture_chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: fixture_chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            for source_z in target_pos.z - 1..=target_pos.z + 1 {
                for source_x in target_pos.x - 1..=target_pos.x + 1 {
                    let source_pos = ChunkPos {
                        x: source_x,
                        z: source_z,
                    };
                    let mut biomes = std::collections::BTreeSet::new();
                    for chunk_z in source_pos.z - 1..=source_pos.z + 1 {
                        for chunk_x in source_pos.x - 1..=source_pos.x + 1 {
                            let chunk_quart_x = chunk_x * 4;
                            let chunk_quart_z = chunk_z * 4;
                            for local_z in 0..4 {
                                for local_x in 0..4 {
                                    if let Some(biome) = super::super::get_biome(
                                        biome_source_model,
                                        chunk_quart_x + local_x,
                                        sample_quart_y,
                                        chunk_quart_z + local_z,
                                        &climate_sampler,
                                    ) {
                                        biomes.insert(biome);
                                    }
                                }
                            }
                        }
                    }
                    let possible_steps = super::super::possible_biome_feature_steps_for_decoration_region(
                        source_pos,
                        biome_source_model,
                        noise_settings,
                        &climate_sampler,
                    );
                    let plan = super::super::biome_decoration_feature_plan(
                        seed,
                        source_pos.x,
                        source_pos.z,
                        min_section_y,
                        &features_per_step,
                        &possible_steps,
                    );
                    let ore_features = plan
                        .feature_calls
                        .iter()
                        .filter(|call| {
                            call.step_index == GenerationDecorationStep::UndergroundOres as usize
                        })
                        .map(|call| call.feature)
                        .collect::<Vec<_>>();
                    let vegetal_features = plan
                        .feature_calls
                        .iter()
                        .filter(|call| {
                            call.step_index == GenerationDecorationStep::VegetalDecoration as usize
                        })
                        .map(|call| call.feature)
                        .collect::<Vec<_>>();
                    eprintln!(
                        "[decoration-source-schedule] target=({}, {}) source=({}, {}) biomes=[{}] ores=[{}] vegetal=[{}]",
                        target_pos.x,
                        target_pos.z,
                        source_pos.x,
                        source_pos.z,
                        biomes.into_iter().collect::<Vec<_>>().join(","),
                        ore_features.join(","),
                        vegetal_features.join(","),
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "diagnostic for Java-shaped tree feature region writes"]
    fn normal_overworld_region_tree_write_parity_stocktake() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("vanilla fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut region_chunks = BTreeMap::<ChunkPos, LevelChunk>::new();
        let mut source_positions = Vec::new();
        for chunk in chunks {
            let pos = ChunkPos {
                x: chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut chunk = base;
            super::super::apply_configured_carvers_for_biome_source(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
            );
            super::super::apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
            super::super::apply_underground_ore_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            region_chunks.insert(pos, chunk);
            source_positions.push(pos);
        }

        let mut total_tree_blocks = 0;
        for pos in source_positions {
            let result = super::super::apply_initial_tree_decoration_from_source_into_region(
                &mut region_chunks,
                pos,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            total_tree_blocks += result.placed_blocks;
        }

        let generated = region_chunks.values().cloned().collect::<Vec<_>>();
        let block = crate::worldgen_comparison::vanilla_worldgen_block_array_parity_score(
            fixture_json,
            &generated,
        )
        .expect("block parity score should compute");
        let heightmap = crate::worldgen_comparison::vanilla_worldgen_heightmap_parity_score(
            fixture_json,
            &generated,
        )
        .expect("heightmap parity score should compute");
        let column = crate::worldgen_comparison::vanilla_worldgen_column_profile_parity_score(
            fixture_json,
            &generated,
        )
        .expect("column-profile parity score should compute");
        eprintln!(
            "[worldgen-region-tree-parity] tree_blocks={} block={:.6} ({}/{}) heightmap={:.6} ({}/{}) column={:.6} ({}/{})",
            total_tree_blocks,
            block.score,
            block.matching_blocks,
            block.total_blocks,
            heightmap.score,
            heightmap.matching_columns,
            heightmap.total_columns,
            column.score,
            column.matching_columns,
            column.total_columns
        );
        for mismatch in block.mismatches.iter().take(8) {
            eprintln!(
                "[worldgen-region-tree-parity-mismatch] count={} expected={} actual={}",
                mismatch.count, mismatch.expected, mismatch.actual
            );
        }
    }

    #[test]
    #[ignore = "diagnostic for final tree-stage leaf/log placement drift"]
    fn normal_overworld_tree_mismatch_shape_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut generated_chunks = Vec::new();
        for chunk in chunks {
            let pos = ChunkPos {
                x: chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut chunk = base;
            super::super::apply_configured_carvers_for_biome_source(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
            );
            super::super::apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
            super::super::apply_underground_ore_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            super::super::apply_initial_tree_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
                None,
                None,
            );
            generated_chunks.push(chunk);
        }

        let tracked = [
            "minecraft:oak_leaves",
            "minecraft:birch_leaves",
            "minecraft:oak_log",
            "minecraft:birch_log",
            "minecraft:leaf_litter",
            "minecraft:air",
        ];
        let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
        let mut y_ranges: BTreeMap<(String, String), (i32, i32)> = BTreeMap::new();
        let mut examples: BTreeMap<(String, String), Vec<BlockPos>> = BTreeMap::new();

        for (chunk_index, fixture_chunk) in chunks.iter().enumerate() {
            let chunk_x = fixture_chunk
                .get("chunkX")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkX") as i32;
            let chunk_z = fixture_chunk
                .get("chunkZ")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkZ") as i32;
            let y_min = fixture_chunk
                .get("yMin")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include yMin") as i32;
            let blocks = fixture_chunk
                .get("blocks")
                .and_then(serde_json::Value::as_array)
                .expect("fixture chunk should include blocks");
            let generated = &generated_chunks[chunk_index];

            for (local_x, y_column) in blocks.iter().enumerate() {
                let y_column = y_column.as_array().expect("x column should be an array");
                for (y_offset, z_column) in y_column.iter().enumerate() {
                    let world_y = y_min + y_offset as i32;
                    let z_column = z_column.as_array().expect("z column should be an array");
                    for (local_z, expected) in z_column.iter().enumerate() {
                        let expected = expected
                            .as_str()
                            .expect("fixture block should be a string")
                            .split_once('[')
                            .map_or_else(|| expected.as_str().unwrap(), |(id, _)| id);
                        let world_x = chunk_x * 16 + local_x as i32;
                        let world_z = chunk_z * 16 + local_z as i32;
                        let actual = generated
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if expected == actual {
                            continue;
                        }
                        if !tracked.contains(&expected) && !tracked.contains(&actual.as_str()) {
                            continue;
                        }
                        let key = (expected.to_string(), actual);
                        *counts.entry(key.clone()).or_insert(0) += 1;
                        y_ranges
                            .entry(key.clone())
                            .and_modify(|range| {
                                range.0 = range.0.min(world_y);
                                range.1 = range.1.max(world_y);
                            })
                            .or_insert((world_y, world_y));
                        let examples = examples.entry(key).or_default();
                        if examples.len() < 8 {
                            examples.push(BlockPos {
                                x: world_x,
                                y: world_y,
                                z: world_z,
                            });
                        }
                    }
                }
            }
        }

        let mut sorted = counts.into_iter().collect::<Vec<_>>();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for ((expected, actual), count) in sorted.into_iter().take(20) {
            let (min_y, max_y) = y_ranges[&(expected.clone(), actual.clone())];
            let examples = examples[&(expected.clone(), actual.clone())]
                .iter()
                .map(|pos| format!("({}, {}, {})", pos.x, pos.y, pos.z))
                .collect::<Vec<_>>()
                .join(", ");
            eprintln!(
                "[tree-shape-diagnostic] count={} expected={} actual={} y={}..{} examples={}",
                count, expected, actual, min_y, max_y, examples
            );
        }
    }

    #[test]
    #[ignore = "diagnostic for tree trunks with missing or unsupported bottom logs"]
    fn normal_overworld_tree_root_support_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");
        let fixture_chunks = fixture
            .get("chunks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture should include chunks");

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };

        let mut generated_chunks = Vec::new();
        for fixture_chunk in fixture_chunks {
            let pos = ChunkPos {
                x: fixture_chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX") as i32,
                z: fixture_chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ") as i32,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut chunk = base;
            super::super::apply_configured_carvers_for_biome_source(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
            );
            super::super::apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
            super::super::apply_underground_ore_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
            );
            super::super::apply_initial_tree_decoration_to_chunk(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                None,
                None,
                None,
            );
            generated_chunks.push(chunk);
        }

        let mut unsupported_generated_logs = Vec::new();
        let mut missing_expected_base_logs = Vec::new();
        for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
            let chunk_x = fixture_chunk
                .get("chunkX")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkX") as i32;
            let chunk_z = fixture_chunk
                .get("chunkZ")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include chunkZ") as i32;
            let y_min = fixture_chunk
                .get("yMin")
                .and_then(serde_json::Value::as_i64)
                .expect("fixture chunk should include yMin") as i32;
            let blocks = fixture_chunk
                .get("blocks")
                .and_then(serde_json::Value::as_array)
                .expect("fixture chunk should include blocks");
            let generated = &generated_chunks[chunk_index];

            for (local_x, y_column) in blocks.iter().enumerate() {
                let y_column = y_column.as_array().expect("x column should be an array");
                for (y_offset, z_column) in y_column.iter().enumerate() {
                    let world_y = y_min + y_offset as i32;
                    let z_column = z_column.as_array().expect("z column should be an array");
                    for (local_z, expected) in z_column.iter().enumerate() {
                        let world_x = chunk_x * 16 + local_x as i32;
                        let world_z = chunk_z * 16 + local_z as i32;
                        let expected = expected
                            .as_str()
                            .expect("fixture block should be a string")
                            .split_once('[')
                            .map_or_else(|| expected.as_str().unwrap(), |(id, _)| id);
                        let actual = generated
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if super::super::block_matches_tag(&actual, "minecraft:logs") {
                            let below = generated
                                .get_block_state(world_x, world_y - 1, world_z)
                                .unwrap_or_else(|| "minecraft:air".to_string());
                            let vertical_below = generated
                                .get_block_state(world_x, world_y - 1, world_z)
                                .is_some_and(|below| {
                                    super::super::block_matches_tag(&below, "minecraft:logs")
                                });
                            if !vertical_below
                                && !super::super::block_blocks_motion(&below)
                                && unsupported_generated_logs.len() < 16
                            {
                                unsupported_generated_logs.push((world_x, world_y, world_z, below));
                            }
                        }
                        if super::super::block_matches_tag(expected, "minecraft:logs")
                            && actual == "minecraft:air"
                        {
                            let above = generated
                                .get_block_state(world_x, world_y + 1, world_z)
                                .unwrap_or_else(|| "minecraft:air".to_string());
                            if super::super::block_matches_tag(&above, "minecraft:logs")
                                && missing_expected_base_logs.len() < 16
                            {
                                missing_expected_base_logs.push((
                                    world_x,
                                    world_y,
                                    world_z,
                                    expected.to_string(),
                                    above,
                                ));
                            }
                        }
                    }
                }
            }
        }

        eprintln!(
            "[tree-root-support] unsupported_generated_logs={} examples={:?}",
            unsupported_generated_logs.len(),
            unsupported_generated_logs
        );
        eprintln!(
            "[tree-root-support] missing_expected_base_logs={} examples={:?}",
            missing_expected_base_logs.len(),
            missing_expected_base_logs
        );
    }

    #[test]
    #[ignore = "diagnostic for unsupported tree logs in region-generated chunks"]
    fn normal_overworld_region_tree_root_support_diagnostic() {
        let fixture_json =
            include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_json).expect("vanilla fixture should parse");
        let seed = fixture
            .get("seed")
            .and_then(serde_json::Value::as_str)
            .expect("vanilla fixture should include a seed")
            .parse::<i64>()
            .expect("vanilla fixture seed should parse");

        let center = ChunkPos {
            x: std::env::var("RUSTCRAFT_TREE_ROOT_DIAG_CENTER_X")
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(0),
            z: std::env::var("RUSTCRAFT_TREE_ROOT_DIAG_CENTER_Z")
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(0),
        };
        let radius = std::env::var("RUSTCRAFT_TREE_ROOT_DIAG_RADIUS")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(2);
        let chunks = super::super::generate_overworld_spawn_chunk_region_for_preset_with_mode(
            center,
            radius,
            "normal",
            super::super::LiveChunkGenerationMode::RealSurface,
            seed,
            false,
        )
        .expect("region generation should succeed");

        let mut unsupported_logs = Vec::new();
        let mut floating_trunks = Vec::new();
        for (chunk_pos, chunk) in &chunks {
            let min_y = chunk.min_section_y * 16;
            let max_y = min_y + chunk.sections.len() as i32 * 16;
            for local_z in 0..16 {
                for local_x in 0..16 {
                    let world_x = chunk_pos.x * 16 + local_x;
                    let world_z = chunk_pos.z * 16 + local_z;
                    for world_y in min_y..max_y {
                        let Some(state) = chunk.get_block_state(world_x, world_y, world_z) else {
                            continue;
                        };
                        if !super::super::block_matches_tag(&state, "minecraft:logs") {
                            continue;
                        }
                        let below = chunks
                            .get(&ChunkPos {
                                x: world_x.div_euclid(16),
                                z: world_z.div_euclid(16),
                            })
                            .and_then(|below_chunk| {
                                below_chunk.get_block_state(world_x, world_y - 1, world_z)
                            })
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        let below_is_log = super::super::block_matches_tag(&below, "minecraft:logs");
                        if !below_is_log && !super::super::block_blocks_motion(&below) {
                            if unsupported_logs.len() < 24 {
                                unsupported_logs.push((world_x, world_y, world_z, state, below));
                            }
                            let above = chunk
                                .get_block_state(world_x, world_y + 1, world_z)
                                .unwrap_or_else(|| "minecraft:air".to_string());
                            if super::super::block_matches_tag(&above, "minecraft:logs")
                                && floating_trunks.len() < 24
                            {
                                floating_trunks.push((world_x, world_y, world_z, above));
                            }
                        }
                    }
                }
            }
        }

        eprintln!(
            "[region-tree-root-support] center=({}, {}) radius={} chunks={} unsupported_logs={} examples={:?}",
            center.x,
            center.z,
            radius,
            chunks.len(),
            unsupported_logs.len(),
            unsupported_logs
        );
        eprintln!(
            "[region-tree-root-support] floating_trunks={} examples={:?}",
            floating_trunks.len(),
            floating_trunks
        );
    }

