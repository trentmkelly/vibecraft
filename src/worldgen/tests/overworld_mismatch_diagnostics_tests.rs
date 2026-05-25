use super::*;

#[test]
    #[ignore = "diagnostic for tree placement feature seed/index drift"]
    fn normal_overworld_tree_feature_seed_candidate_diagnostic() {
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

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        let router_id = super::super::noise_router_id_for_settings(**noise_settings);
        let noise_router = super::super::builtin_noise_router(router_id)
            .map(|entry| entry.router)
            .expect("normal overworld should have a built-in noise router");
        let climate_sampler =
            super::super::ClimateSampler::from_noise_router(&noise_router, seed, **noise_settings);
        let global_biome_steps = super::super::possible_biome_feature_steps_for_source(biome_source_model);
        let global_features_per_step =
            super::super::build_features_per_step(&global_biome_steps, true).unwrap();
        let region_biome_steps = super::super::possible_biome_feature_steps_for_decoration_region(
            ChunkPos { x: -1, z: -1 },
            biome_source_model,
            noise_settings,
            &climate_sampler,
        );
        let plan = super::super::biome_decoration_feature_plan(
            seed,
            -1,
            -1,
            noise_settings.noise.min_y.div_euclid(16),
            &global_features_per_step,
            &region_biome_steps,
        );
        let tree_call = plan
            .feature_calls
            .iter()
            .find(|call| call.feature == "minecraft:trees_birch_and_oak_leaf_litter")
            .expect("forest fixture chunk should schedule birch/oak leaf-litter trees");
        eprintln!(
            "[tree-seed-diagnostic] planned feature={} step={} global_index={} seed={} decoration_seed={}",
            tree_call.feature,
            tree_call.step_index,
            tree_call.global_feature_index,
            tree_call.seed,
            plan.decoration_seed,
        );

        fn locals_for_seed(seed: i64) -> Vec<(usize, usize)> {
            let mut random = crate::random_source::RandomSourceKind::new(
                seed,
                crate::random_source::RandomAlgorithm::Xoroshiro,
            );
            let count = super::super::live_tree_count(
                super::super::NoisePreviewTreeCountKind::CountExtra {
                    count: 10,
                    inverse_chance_weight: 10,
                    extra: 1,
                },
                &mut random,
            );
            (0..count)
                .map(|_| {
                    (
                        super::super::feature_random_next_i32_bound(&mut random, 16) as usize,
                        super::super::feature_random_next_i32_bound(&mut random, 16) as usize,
                    )
                })
                .collect()
        }

        eprintln!(
            "[tree-seed-diagnostic] planned_locals={:?}",
            locals_for_seed(tree_call.seed)
        );
        for candidate_index in 0..128_i32 {
            let candidate_seed = crate::random_source::feature_seed(
                plan.decoration_seed,
                candidate_index,
                GenerationDecorationStep::VegetalDecoration as i32,
            );
            let locals = locals_for_seed(candidate_seed);
            if locals.contains(&(12, 4)) || locals.contains(&(15, 10)) {
                eprintln!(
                    "[tree-seed-diagnostic] candidate_index={} seed={} locals={:?}",
                    candidate_index, candidate_seed, locals
                );
            }
        }
        let vegetation_features = global_features_per_step
            .get(GenerationDecorationStep::VegetalDecoration as usize)
            .expect("vegetation step should exist");
        for (index, feature) in vegetation_features.features.iter().enumerate() {
            if feature.feature.contains("tree") || feature.feature.contains("birch") {
                eprintln!(
                    "[tree-seed-diagnostic] vegetation_index={} feature={}",
                    index, feature.feature
                );
            }
        }
    }

    #[test]
    #[ignore = "diagnostic parity buckets for base-stone and cave material mismatches"]
    fn normal_overworld_base_stone_mismatch_diagnostic() {
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

        let mut stage_chunks: Vec<(&'static str, Vec<LevelChunk>)> = vec![
            ("noise_surface", Vec::new()),
            ("carvers", Vec::new()),
            ("structures", Vec::new()),
            ("ores", Vec::new()),
            ("trees", Vec::new()),
        ];

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
            let mut carver = base.clone();
            super::super::apply_configured_carvers_for_biome_source(
                &mut carver,
                biome_source_model,
                noise_settings,
                seed,
            );
            let mut ore = carver.clone();
            super::super::apply_mineshaft_underground_structures_to_chunk(&mut ore, seed);
            let structure = ore.clone();
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

            stage_chunks[0].1.push(base);
            stage_chunks[1].1.push(carver);
            stage_chunks[2].1.push(structure);
            stage_chunks[3].1.push(ore);
            stage_chunks[4].1.push(full);
        }

        let tracked_pairs = [
            ("minecraft:tuff", "minecraft:deepslate"),
            ("minecraft:deepslate", "minecraft:tuff"),
            ("minecraft:granite", "minecraft:stone"),
            ("minecraft:stone", "minecraft:granite"),
            ("minecraft:cave_air", "minecraft:deepslate"),
            ("minecraft:deepslate", "minecraft:cave_air"),
            ("minecraft:cave_air", "minecraft:stone"),
            ("minecraft:stone", "minecraft:cave_air"),
        ];

        for (stage_name, chunks) in &stage_chunks {
            let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
            let mut y_ranges: BTreeMap<(String, String), (i32, i32)> = BTreeMap::new();
            let mut examples: BTreeMap<(String, String), Vec<BlockPos>> = BTreeMap::new();

            for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
                let chunk_x = fixture_chunk
                    .get("chunkX")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkX")
                    as i32;
                let chunk_z = fixture_chunk
                    .get("chunkZ")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include chunkZ")
                    as i32;
                let y_min = fixture_chunk
                    .get("yMin")
                    .and_then(serde_json::Value::as_i64)
                    .expect("fixture chunk should include yMin") as i32;
                let blocks = fixture_chunk
                    .get("blocks")
                    .and_then(serde_json::Value::as_array)
                    .expect("fixture chunk should include blocks");
                let generated = &chunks[chunk_index];

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
                            if !tracked_pairs.contains(&(expected, actual.as_str())) {
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
                            if examples.len() < 5 {
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

            eprintln!("[base-stone-diagnostic] stage={stage_name}");
            for &(expected, actual) in &tracked_pairs {
                let key = (expected.to_string(), actual.to_string());
                let count = counts.get(&key).copied().unwrap_or(0);
                if count == 0 {
                    continue;
                }
                let (min_y, max_y) = y_ranges[&key];
                let examples = examples
                    .get(&key)
                    .map(|positions| {
                        positions
                            .iter()
                            .map(|pos| format!("({}, {}, {})", pos.x, pos.y, pos.z))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                eprintln!(
                    "[base-stone-diagnostic] stage={} expected={} actual={} count={} y={}..{} examples={}",
                    stage_name, expected, actual, count, min_y, max_y, examples
                );
            }
        }
    }

    #[test]
    #[ignore = "diagnostic for cave-air mismatches while aligning carver/aquifer parity"]
    fn normal_overworld_cave_air_mismatch_diagnostic() {
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

        let mut printed = 0_usize;
        for fixture_chunk in chunks {
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
            let pos = ChunkPos {
                x: chunk_x,
                z: chunk_z,
            };
            let (base, _, _) = super::super::generate_real_surface_base_chunk(
                pos,
                biome_source_model,
                noise_settings,
                seed,
            )
            .expect("real-surface base generation should succeed");
            let mut carver = base.clone();
            let carved = super::super::apply_configured_carvers_for_biome_source(
                &mut carver,
                biome_source_model,
                noise_settings,
                seed,
            );

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
                            .map_or_else(
                                || expected.as_str().unwrap().to_string(),
                                |(id, _)| id.to_string(),
                            );
                        if expected != "minecraft:cave_air" {
                            continue;
                        }
                        let world_x = chunk_x * 16 + local_x as i32;
                        let world_z = chunk_z * 16 + local_z as i32;
                        let actual = carver
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        if matches!(
                            actual.as_str(),
                            "minecraft:cave_air"
                                | "minecraft:air"
                                | "minecraft:water"
                                | "minecraft:lava"
                        ) {
                            continue;
                        }
                        let before = base
                            .get_block_state(world_x, world_y, world_z)
                            .unwrap_or_else(|| "minecraft:air".to_string());
                        let mask_index = super::super::carver_mask_index(
                            world_x,
                            world_y,
                            world_z,
                            noise_settings.noise.min_y,
                        )
                        .expect("fixture y should be in mask range");
                        let mask_set = carver.carving_mask.as_ref().is_some_and(|words| {
                            let word = mask_index / 64;
                            let bit = mask_index % 64;
                            words
                                .get(word)
                                .is_some_and(|value| ((*value as u64) & (1_u64 << bit)) != 0)
                        });
                        let detail = ore_vein_material_detail_at(
                            pos,
                            world_x,
                            world_y,
                            world_z,
                            noise_settings,
                            seed,
                        );
                        eprintln!(
                            "[cave-air-diagnostic] chunk=({chunk_x},{chunk_z}) local=({local_x},{world_y},{local_z}) world=({world_x},{world_y},{world_z}) before={before} after_carvers={actual} mask_set={mask_set} carved_blocks={carved} density={:.6} direct={:.6} sloped={:.6} entrances={:.6} underground={:.6} caves={:.6} post={:.6} noodle={:.6}",
                            detail.density,
                            detail.direct_density,
                            detail.sloped_cheese,
                            detail.entrances,
                            detail.underground,
                            detail.caves,
                            detail.post_process,
                            detail.noodle
                        );
                        printed += 1;
                        if printed >= 32 {
                            return;
                        }
                    }
                }
            }
        }
        assert!(printed > 0, "diagnostic should find cave-air mismatches");
    }

    #[test]
    #[ignore = "diagnostic for base material-rule drift against the vanilla fixture"]
    fn normal_overworld_ore_vein_material_mismatch_diagnostic() {
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
            base_chunks.push(base);
        }

        let block = crate::worldgen_comparison::vanilla_worldgen_block_array_parity_score(
            fixture_json,
            &base_chunks,
        )
        .expect("block parity score should compute");
        eprintln!(
            "[ore-vein-material-diagnostic] base_block_score={:.6} ({}/{})",
            block.score, block.matching_blocks, block.total_blocks
        );

        let interesting = [
            "minecraft:tuff",
            "minecraft:deepslate",
            "minecraft:granite",
            "minecraft:diorite",
            "minecraft:andesite",
            "minecraft:gravel",
            "minecraft:stone",
        ];
        let mut printed = 0;
        for sample in &block.samples {
            if !interesting.contains(&sample.expected.as_str())
                && !interesting.contains(&sample.actual.as_str())
            {
                continue;
            }
            let world_x = sample.chunk.x * 16 + sample.local_x as i32;
            let world_z = sample.chunk.z * 16 + sample.local_z as i32;
            let detail = ore_vein_material_detail_at(
                ChunkPos {
                    x: sample.chunk.x,
                    z: sample.chunk.z,
                },
                world_x,
                sample.y,
                world_z,
                noise_settings,
                seed,
            );
            eprintln!(
                "[ore-vein-material-sample] chunk=({}, {}) local=({}, {}, {}) world=({}, {}, {}) expected={} actual={} density={:.6} toggle={:.6} ridged={:.6} gap={:.6} decision={:?}",
                sample.chunk.x,
                sample.chunk.z,
                sample.local_x,
                sample.y,
                sample.local_z,
                world_x,
                sample.y,
                world_z,
                sample.expected,
                sample.actual,
                detail.density,
                detail.vein_toggle,
                detail.vein_ridged,
                detail.vein_gap,
                detail.decision
            );
            printed += 1;
            if printed >= 24 {
                break;
            }
        }
    }

    #[test]
    #[ignore = "diagnostic for one remaining andesite/diorite ore placement drift"]
    fn normal_overworld_andesite_diorite_write_trace_diagnostic() {
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

        let preset = super::super::resolve_world_preset("normal").expect("normal preset should resolve");
        let super::super::ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } = &preset.overworld.generator
        else {
            panic!("normal overworld should use a noise generator");
        };
        let target_pos = ChunkPos { x: -1, z: -1 };
        let watched = BlockPos {
            x: -4,
            y: 27,
            z: -14,
        };

        let (base, _, _) = super::super::generate_real_surface_base_chunk(
            target_pos,
            biome_source_model,
            noise_settings,
            seed,
        )
        .expect("real-surface base generation should succeed");
        let mut chunk = base.clone();
        super::super::apply_configured_carvers_for_biome_source(
            &mut chunk,
            biome_source_model,
            noise_settings,
            seed,
        );
        super::super::apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);

        let router =
            super::super::builtin_noise_router(super::super::noise_router_id_for_settings(**noise_settings))
                .map(|entry| entry.router)
                .expect("normal overworld should have a router");
        let climate_sampler =
            super::super::ClimateSampler::from_noise_router(&router, seed, **noise_settings);
        let source_steps_cache = super::super::source_decoration_biome_steps_cache(
            target_pos,
            1,
            biome_source_model,
            noise_settings,
            &climate_sampler,
        );
        let global_biome_steps = super::super::possible_biome_feature_steps_for_source(biome_source_model);
        let features_per_step = super::super::build_features_per_step(&global_biome_steps, true)
            .expect("global features should sort");
        let context_chunks = super::super::build_underground_ore_decoration_context_chunks(
            target_pos,
            noise_settings,
            seed,
        );
        let mut block_cache =
            super::super::OreBlockCache::from_chunk_with_read_context(&chunk, &context_chunks);

        eprintln!(
            "[andesite-diorite-trace] watched=({}, {}, {}) initial={:?}",
            watched.x,
            watched.y,
            watched.z,
            block_cache.block_state_name(watched.x, watched.y, watched.z)
        );

        for source_z in target_pos.z - 1..=target_pos.z + 1 {
            for source_x in target_pos.x - 1..=target_pos.x + 1 {
                let source_pos = ChunkPos {
                    x: source_x,
                    z: source_z,
                };
                let possible_steps =
                    source_steps_cache
                        .get(&source_pos)
                        .cloned()
                        .unwrap_or_else(|| {
                            super::super::possible_biome_feature_steps_for_decoration_region(
                                source_pos,
                                biome_source_model,
                                noise_settings,
                                &climate_sampler,
                            )
                        });
                if possible_steps.is_empty() {
                    continue;
                }
                let skip_biome_filter = super::super::biome_steps_share_decoration_step_features(
                    &possible_steps,
                    super::super::GenerationDecorationStep::UndergroundOres,
                );
                let plan = super::super::biome_decoration_feature_plan(
                    seed,
                    source_pos.x,
                    source_pos.z,
                    noise_settings.noise.min_y.div_euclid(16),
                    &features_per_step,
                    &possible_steps,
                );
                for call in plan.feature_calls.iter().filter(|call| {
                    call.step_index == super::super::GenerationDecorationStep::UndergroundOres as usize
                        && matches!(
                            call.feature,
                            "minecraft:ore_granite_lower"
                                | "minecraft:ore_diorite_lower"
                                | "minecraft:ore_andesite_lower"
                                | "minecraft:ore_granite_upper"
                                | "minecraft:ore_diorite_upper"
                                | "minecraft:ore_andesite_upper"
                        )
                }) {
                    let Some(feature) = super::super::placed_ore_feature(call.feature) else {
                        continue;
                    };
                    let Some(config) =
                        super::super::configured_ore_configuration(feature.configured_feature)
                    else {
                        continue;
                    };
                    let mut random = super::super::RandomSourceKind::new(
                        call.seed,
                        crate::random_source::RandomAlgorithm::Xoroshiro,
                    );
                    let origin = BlockPos {
                        x: source_pos.x * 16,
                        y: noise_settings.noise.min_y,
                        z: source_pos.z * 16,
                    };
                    let count = match feature.placement.first().copied() {
                        Some(super::super::PlacementModifier::Count { count }) => count.max(0),
                        Some(super::super::PlacementModifier::RarityFilter { chance }) => {
                            if chance > 0
                                && super::super::feature_random_next_f32(&mut random) < 1.0 / chance as f32
                            {
                                1
                            } else {
                                0
                            }
                        }
                        _ => 0,
                    };
                    for origin_index in 0..count {
                        let origin_x =
                            origin.x + super::super::feature_random_next_i32_bound(&mut random, 16);
                        let origin_z =
                            origin.z + super::super::feature_random_next_i32_bound(&mut random, 16);
                        let origin_y = super::super::height_provider_sample_with_random(
                            match feature.placement.get(2).copied() {
                                Some(super::super::PlacementModifier::HeightRange { height }) => height,
                                _ => continue,
                            },
                            super::super::WorldGenerationHeightContext {
                                min_y: noise_settings.noise.min_y,
                                height: noise_settings.noise.height,
                            },
                            &mut random,
                        );
                        let origin = BlockPos {
                            x: origin_x,
                            y: origin_y,
                            z: origin_z,
                        };
                        if !skip_biome_filter
                            && !super::super::biome_allows_feature_at(
                                biome_source_model,
                                noise_settings,
                                &climate_sampler,
                                origin,
                                call.feature,
                            )
                        {
                            continue;
                        }
                        let direction = super::super::feature_random_next_f32(&mut random);
                        let spread_xy = config.size as f32 / 8.0;
                        let max_radius =
                            ((config.size as f32 / 16.0 * 2.0 + 1.0) / 2.0).ceil() as i32;
                        let spread_ceil = spread_xy.ceil() as i32;
                        let x_start = origin.x - spread_ceil - max_radius;
                        let y_start = origin.y - 2 - max_radius;
                        let z_start = origin.z - spread_ceil - max_radius;
                        let size_xz = 2 * (spread_ceil + max_radius);
                        let size_y = 2 * (2 + max_radius);
                        let y_rolls = [(
                            super::super::feature_random_next_i32_bound(&mut random, 3),
                            super::super::feature_random_next_i32_bound(&mut random, 3),
                        )];
                        if !super::super::ore_origin_overlaps_ocean_floor_wg(
                            &block_cache,
                            x_start,
                            y_start,
                            z_start,
                            size_xz,
                        ) {
                            continue;
                        }
                        let radius_rolls = (0..config.size.max(0))
                            .map(|_| super::super::feature_random_next_f64(&mut random))
                            .collect::<Vec<_>>();
                        let spheres = super::super::ore_vein_spheres(
                            origin,
                            config.size,
                            direction,
                            &y_rolls,
                            &radius_rolls,
                        );
                        let candidates = super::super::ore_vein_position_candidates(
                            &spheres,
                            x_start,
                            y_start,
                            z_start,
                            size_xz,
                            size_y,
                            noise_settings.noise.min_y
                                ..noise_settings.noise.min_y + noise_settings.noise.height,
                        );
                        for pos in candidates {
                            let Some(current) = block_cache.block_state_name(pos.x, pos.y, pos.z)
                            else {
                                continue;
                            };
                            let new_block = config.target_states.iter().find_map(|target| {
                                if !super::super::rule_test_matches(target.target, current) {
                                    return None;
                                }
                                let skip_air_check = match config.discard_chance_on_air_exposure {
                                    chance if chance <= 0.0 => true,
                                    chance if chance >= 1.0 => false,
                                    chance => super::super::feature_random_next_f32(&mut random) >= chance,
                                };
                                (skip_air_check
                                    || !super::super::is_adjacent_to_ore_air(&block_cache, pos))
                                .then_some(target.state)
                            });
                            if pos == watched {
                                eprintln!(
                                    "[andesite-diorite-trace] source=({}, {}) feature={} origin_index={} origin=({}, {}, {}) current={} new={:?}",
                                    source_pos.x,
                                    source_pos.z,
                                    call.feature,
                                    origin_index,
                                    origin.x,
                                    origin.y,
                                    origin.z,
                                    current,
                                    new_block
                                );
                            }
                            if pos.x.div_euclid(16) == target_pos.x
                                && pos.z.div_euclid(16) == target_pos.z
                            {
                                if let Some(new_block) = new_block {
                                    block_cache.set_block_state(pos.x, pos.y, pos.z, new_block);
                                }
                            }
                        }
                    }
                }
            }
        }

        eprintln!(
            "[andesite-diorite-trace] final={:?}",
            block_cache.block_state_name(watched.x, watched.y, watched.z)
        );
    }

    #[derive(Debug)]
    struct OreVeinMaterialDetail {
        density: f64,
        direct_density: f64,
        sloped_cheese: f64,
        entrances: f64,
        underground: f64,
        caves: f64,
        post_process: f64,
        noodle: f64,
        vein_toggle: f64,
        vein_ridged: f64,
        vein_gap: f64,
        decision: Option<&'static str>,
    }

    fn ore_vein_material_detail_at(
        chunk_pos: ChunkPos,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        settings: &super::super::NoiseGeneratorSettings,
        seed: i64,
    ) -> OreVeinMaterialDetail {
        let router_id = super::super::noise_router_id_for_settings(*settings);
        let noise_router = super::super::builtin_noise_router(router_id)
            .map(|entry| entry.router)
            .unwrap_or(super::super::NONE_NOISE_ROUTER);
        let chunk_min_x = chunk_pos.x * 16;
        let chunk_min_z = chunk_pos.z * 16;
        let mut noise_chunk =
            super::super::NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
        let cell_width = settings.noise.cell_width();
        let cell_height = settings.noise.cell_height();
        let cell_x_index = (world_x - chunk_min_x).div_euclid(cell_width);
        let cell_z_index = (world_z - chunk_min_z).div_euclid(cell_width);
        let cell_noise_y = world_y.div_euclid(cell_height);
        let cell_y_index = cell_noise_y - settings.noise.min_y.div_euclid(cell_height);

        noise_chunk.advance_cell_x(cell_x_index);
        noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
        noise_chunk.update_for_y(
            world_y,
            f64::from(world_y.rem_euclid(cell_height)) / f64::from(cell_height),
        );
        noise_chunk.update_for_x(
            world_x,
            f64::from((world_x - chunk_min_x).rem_euclid(cell_width)) / f64::from(cell_width),
        );
        noise_chunk.update_for_z(
            world_z,
            f64::from((world_z - chunk_min_z).rem_euclid(cell_width)) / f64::from(cell_width),
        );

        let vein_toggle = noise_chunk.cached_vein_toggle(world_x, world_y, world_z);
        let vein_ridged = noise_chunk.vein_ridged_at(world_x, world_y, world_z);
        let vein_gap = noise_chunk.vein_gap_at(world_x, world_y, world_z);
        let algorithm = if settings.legacy_random_source {
            crate::random_source::RandomAlgorithm::Legacy
        } else {
            crate::random_source::RandomAlgorithm::Xoroshiro
        };
        let ore_factory = crate::random_source::random_state_seed_factories(seed, algorithm).ore;
        OreVeinMaterialDetail {
            density: noise_chunk.interpolated_density(world_x, world_y, world_z),
            direct_density: noise_chunk.full_noise_uncached_at(world_x, world_y, world_z),
            sloped_cheese: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_SLOPED_CHEESE_REFERENCE_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            entrances: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_CAVES_ENTRANCES_REFERENCE_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            underground: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_UNDERGROUND_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            caves: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_CAVES_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            post_process: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_FINAL_POST_PROCESS_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            noodle: super::super::eval_density_fn_with_interp(
                super::super::OVERWORLD_CAVES_NOODLE_REFERENCE_DENSITY,
                &noise_chunk,
                world_x,
                world_y,
                world_z,
            ),
            vein_toggle,
            vein_ridged,
            vein_gap,
            decision: super::super::ore_vein_decision_after_toggle(
                ore_factory,
                BlockPos {
                    x: world_x,
                    y: world_y,
                    z: world_z,
                },
                vein_toggle,
                || vein_ridged,
                || vein_gap,
                false,
            ),
        }
    }
