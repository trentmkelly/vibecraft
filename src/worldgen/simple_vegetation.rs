use super::*;

#[derive(Debug, Clone)]
pub(super) struct PlacedSimpleVegetationFeature {
    pub(super) configured_feature: &'static str,
    pub(super) placement: Vec<PlacementModifier>,
}

pub(super) fn placed_simple_vegetation_feature(id: &str) -> Option<PlacedSimpleVegetationFeature> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let air_filter = PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::MatchingBlockTag {
            tag: "minecraft:air",
        },
    };
    let leaf_litter_filter = PlacementModifier::BlockPredicateFilter {
        predicate: BlockPredicate::AllOf {
            predicates: &[
                BlockPredicate::MatchingBlockTag {
                    tag: "minecraft:air",
                },
                BlockPredicate::MatchingBlocksAt {
                    offset_y: -1,
                    blocks: &["minecraft:grass_block"],
                },
            ],
        },
    };
    let in_square = PlacementModifier::InSquare;
    let biome = PlacementModifier::BiomeFilter;
    let heightmap = PlacementModifier::Heightmap {
        heightmap: HeightmapKind::MotionBlocking,
    };
    let world_surface_wg = PlacementModifier::Heightmap {
        heightmap: HeightmapKind::WorldSurfaceWg,
    };
    let random_offset = |xz_spread, y_spread| PlacementModifier::RandomOffset {
        xz_spread,
        y_spread,
    };
    let count = |count| PlacementModifier::Count { count };
    let rarity = |chance| PlacementModifier::RarityFilter { chance };
    let noise_threshold =
        |noise_level, below_noise, above_noise| PlacementModifier::NoiseThresholdCount {
            noise_level,
            below_noise,
            above_noise,
            sampled_noise: 0.0,
        };
    let feature = match name {
        "patch_grass_plain" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                noise_threshold(-0.8, 5, 10),
                in_square,
                world_surface_wg,
                biome,
                count(32),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_grass_meadow" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                noise_threshold(-0.8, 5, 10),
                in_square,
                world_surface_wg,
                biome,
                count(16),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_grass_forest" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                count(2),
                in_square,
                world_surface_wg,
                biome,
                count(32),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_leaf_litter" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:leaf_litter",
            placement: vec![
                count(2),
                in_square,
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::WorldSurface,
                },
                biome,
                count(32),
                random_offset(7, 3),
                leaf_litter_filter,
            ],
        },
        "patch_grass_badlands" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                in_square,
                world_surface_wg,
                biome,
                count(32),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_grass_savanna" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                count(20),
                in_square,
                world_surface_wg,
                biome,
                count(32),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_grass_normal" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:grass",
            placement: vec![
                count(5),
                in_square,
                world_surface_wg,
                biome,
                count(32),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_tall_grass_2" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:tall_grass",
            placement: vec![
                noise_threshold(-0.8, 0, 7),
                rarity(32),
                in_square,
                heightmap,
                biome,
                count(96),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_tall_grass" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:tall_grass",
            placement: vec![
                rarity(5),
                in_square,
                heightmap,
                biome,
                count(96),
                random_offset(7, 3),
                air_filter,
            ],
        },
        "patch_bush" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:bush",
            placement: vec![
                rarity(4),
                in_square,
                heightmap,
                biome,
                count(24),
                random_offset(5, 3),
                air_filter,
            ],
        },
        "flower_plains" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:flower_plain",
            placement: vec![
                noise_threshold(-0.8, 15, 4),
                rarity(32),
                in_square,
                heightmap,
                biome,
                count(64),
                random_offset(6, 2),
                air_filter,
            ],
        },
        "flower_default" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:flower_default",
            placement: vec![rarity(32), in_square, heightmap, biome],
        },
        "forest_flowers" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:forest_flowers",
            placement: vec![
                rarity(7),
                in_square,
                heightmap,
                PlacementModifier::CountProvider {
                    provider: IntProviderModel::Uniform {
                        min_inclusive: -3,
                        max_inclusive: 1,
                    },
                    sampled_count: 0,
                },
                biome,
            ],
        },
        "patch_sunflower" => PlacedSimpleVegetationFeature {
            configured_feature: "minecraft:sunflower",
            placement: vec![
                rarity(3),
                in_square,
                heightmap,
                biome,
                count(96),
                random_offset(7, 3),
                air_filter,
            ],
        },
        _ => return None,
    };
    Some(feature)
}

pub(super) fn configured_simple_vegetation_block(
    id: &str,
    random: &mut RandomSourceKind,
    pos: BlockPos,
) -> Option<SimpleBlockConfigurationModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    let mut random_roll = || feature_random_next_i32_bound(random, i32::MAX);
    let to_place = match name {
        "grass" => BlockStateProviderModel::Simple("minecraft:short_grass"),
        "tall_grass" => BlockStateProviderModel::Simple("minecraft:tall_grass"),
        "bush" => BlockStateProviderModel::Simple("minecraft:bush"),
        "sunflower" => BlockStateProviderModel::Simple("minecraft:sunflower"),
        "leaf_litter" => {
            let mut entries = Vec::with_capacity(12);
            for amount in 1..=3 {
                for facing in ["north", "east", "south", "west"] {
                    let state = match (amount, facing) {
                        (1, "north") => "minecraft:leaf_litter[amount=1,facing=north]",
                        (1, "east") => "minecraft:leaf_litter[amount=1,facing=east]",
                        (1, "south") => "minecraft:leaf_litter[amount=1,facing=south]",
                        (1, "west") => "minecraft:leaf_litter[amount=1,facing=west]",
                        (2, "north") => "minecraft:leaf_litter[amount=2,facing=north]",
                        (2, "east") => "minecraft:leaf_litter[amount=2,facing=east]",
                        (2, "south") => "minecraft:leaf_litter[amount=2,facing=south]",
                        (2, "west") => "minecraft:leaf_litter[amount=2,facing=west]",
                        (3, "north") => "minecraft:leaf_litter[amount=3,facing=north]",
                        (3, "east") => "minecraft:leaf_litter[amount=3,facing=east]",
                        (3, "south") => "minecraft:leaf_litter[amount=3,facing=south]",
                        (3, "west") => "minecraft:leaf_litter[amount=3,facing=west]",
                        _ => unreachable!(),
                    };
                    entries.push(WeightedBlockState { state, weight: 1 });
                }
            }
            BlockStateProviderModel::Weighted(entries)
        }
        "flower_default" => BlockStateProviderModel::Weighted(vec![
            WeightedBlockState {
                state: "minecraft:poppy",
                weight: 2,
            },
            WeightedBlockState {
                state: "minecraft:dandelion",
                weight: 1,
            },
        ]),
        "flower_plain" => {
            let provider = BlockStateProviderModel::NoiseThreshold {
                threshold: -0.8,
                high_chance: 0.33333334,
                default_state: "minecraft:dandelion",
                low_states: vec![
                    "minecraft:orange_tulip",
                    "minecraft:red_tulip",
                    "minecraft:pink_tulip",
                    "minecraft:white_tulip",
                ],
                high_states: vec![
                    "minecraft:poppy",
                    "minecraft:azure_bluet",
                    "minecraft:oxeye_daisy",
                    "minecraft:cornflower",
                ],
            };
            let noise_value = vegetation_flower_noise(pos.x, pos.z, 2345, 0.005);
            let state = block_state_provider_sample_with_noise_value(
                &provider,
                random_roll(),
                noise_value,
            )?;
            BlockStateProviderModel::Simple(state)
        }
        "forest_flowers" => {
            let states = [
                "minecraft:lilac",
                "minecraft:rose_bush",
                "minecraft:peony",
                "minecraft:lily_of_the_valley",
            ];
            BlockStateProviderModel::Simple(
                states[feature_random_next_i32_bound(random, states.len() as i32) as usize],
            )
        }
        _ => return None,
    };
    Some(SimpleBlockConfigurationModel {
        to_place,
        schedule_tick: false,
    })
}

pub(super) fn vegetation_flower_noise(world_x: i32, world_z: i32, seed: i64, scale: f64) -> f64 {
    let x = f64::from(world_x) * scale;
    let z = f64::from(world_z) * scale;
    let seed_offset = (seed as f64).sin() * 31.415_926_535_897_93;
    ((x + seed_offset).sin() * 0.55 + (z - seed_offset).cos() * 0.45).clamp(-1.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SimpleVegetationPhase {
    BeforeTrees,
    AfterTrees,
}

pub(super) fn simple_vegetation_phase(feature: &str) -> SimpleVegetationPhase {
    match feature.strip_prefix("minecraft:").unwrap_or(feature) {
        // In forest-like vanilla biome JSON these flower patches are ordered
        // before the tree placed feature in the vegetal-decoration step, so
        // they must be visible to tree validation and tree decorators.
        "forest_flowers" | "flower_forest_flowers" | "wildflowers_birch_forest" => {
            SimpleVegetationPhase::BeforeTrees
        }
        _ => SimpleVegetationPhase::AfterTrees,
    }
}

pub(super) fn apply_initial_simple_vegetation_decoration_to_chunk(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: &[&'static [&'static [&'static str]]],
    source_region_biome_steps: &DecorationBiomeStepsByChunk,
    target_terrain_heights: &TreeDecorationHeights,
    context_cache: &TreeDecorationContextCache,
    phase: SimpleVegetationPhase,
) -> usize {
    let total_started = Instant::now();
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return 0;
    }
    if decoration_region_biome_steps.is_empty() {
        return 0;
    }

    let Some(router) =
        builtin_noise_router(noise_router_id_for_settings(*settings)).map(|entry| entry.router)
    else {
        return 0;
    };
    let climate_sampler = ClimateSampler::from_noise_router(&router, seed, *settings);
    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let feature_source_steps = if global_biome_steps.is_empty() {
        decoration_region_biome_steps
    } else {
        &global_biome_steps
    };
    let feature_sort_started = Instant::now();
    let features_per_step = match build_features_per_step(feature_source_steps, true) {
        Ok(features) => features,
        Err(_) => return 0,
    };
    let feature_sort_ms = feature_sort_started.elapsed().as_millis();

    let target_pos = chunk.pos;
    let mut placed = 0;
    let mut plan_ms = 0_u128;
    let mut placement_ms = 0_u128;
    let mut calls = 0_usize;
    for source_z in target_pos.z - 1..=target_pos.z + 1 {
        for source_x in target_pos.x - 1..=target_pos.x + 1 {
            let source_pos = ChunkPos {
                x: source_x,
                z: source_z,
            };
            let Some(source_terrain_heights) = (if source_pos == target_pos {
                Some(SourceTerrainHeights::Full(target_terrain_heights))
            } else {
                context_cache
                    .cached_region_chunks
                    .get(&source_pos)
                    .map(SourceTerrainHeights::Lazy)
            }) else {
                continue;
            };
            let possible_steps = source_region_biome_steps
                .get(&source_pos)
                .cloned()
                .unwrap_or_else(|| decoration_region_biome_steps.to_vec());
            if possible_steps.is_empty() {
                continue;
            }
            let plan_started = Instant::now();
            let plan = biome_decoration_feature_plan(
                seed,
                source_pos.x,
                source_pos.z,
                settings.noise.min_y.div_euclid(16),
                &features_per_step,
                &possible_steps,
            );
            plan_ms += plan_started.elapsed().as_millis();
            for call in plan.feature_calls.iter().filter(|call| {
                call.step_index == GenerationDecorationStep::VegetalDecoration as usize
                    && placed_simple_vegetation_feature(call.feature).is_some()
                    && simple_vegetation_phase(call.feature) == phase
            }) {
                let Some(feature) = placed_simple_vegetation_feature(call.feature) else {
                    continue;
                };
                calls += 1;
                let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
                let placement_started = Instant::now();
                placed += place_simple_vegetation_feature_positions_depth_first(
                    chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    &climate_sampler,
                    call.feature,
                    &feature,
                    &feature.placement,
                    BlockPos {
                        x: source_pos.x * 16,
                        y: settings.noise.min_y,
                        z: source_pos.z * 16,
                    },
                    source_terrain_heights,
                    &mut random,
                );
                placement_ms += placement_started.elapsed().as_millis();
            }
        }
    }
    if std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_DEBUG").is_some() {
        eprintln!(
            "[simple-vegetation-debug] total={}ms feature_sort={}ms plan={}ms placement={}ms calls={} placed={}",
            total_started.elapsed().as_millis(),
            feature_sort_ms,
            plan_ms,
            placement_ms,
            calls,
            placed
        );
    }
    placed
}

#[allow(clippy::too_many_arguments)]
pub(super) fn place_simple_vegetation_feature_positions_depth_first(
    target_chunk: &mut LevelChunk,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    placed_feature_id: &str,
    feature: &PlacedSimpleVegetationFeature,
    modifiers: &[PlacementModifier],
    position: BlockPos,
    source_terrain_heights: SourceTerrainHeights<'_>,
    random: &mut RandomSourceKind,
) -> usize {
    let Some((modifier, remaining_modifiers)) = modifiers.split_first() else {
        return place_configured_simple_vegetation_in_target_chunk(
            target_chunk,
            settings,
            feature.configured_feature,
            position,
            random,
        );
    };

    match *modifier {
        PlacementModifier::Count { count } => {
            let mut placed = 0;
            for _ in 0..count.max(0) {
                placed += place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                );
            }
            placed
        }
        PlacementModifier::CountProvider { provider, .. } => {
            let mut placed = 0;
            for _ in 0..sample_int_provider(provider, random).clamp(0, i32::MAX) {
                placed += place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                );
            }
            placed
        }
        PlacementModifier::NoiseThresholdCount {
            noise_level,
            below_noise,
            above_noise,
            ..
        } => {
            let noise = vegetation_flower_noise(
                position.x,
                position.z,
                seedless_noise_salt(placed_feature_id),
                0.005,
            );
            let count = if noise < noise_level {
                below_noise
            } else {
                above_noise
            };
            let mut placed = 0;
            for _ in 0..count.max(0) {
                placed += place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                );
            }
            placed
        }
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && feature_random_next_i32_bound(random, chance) == 0 {
                place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                )
            } else {
                0
            }
        }
        PlacementModifier::InSquare => place_simple_vegetation_feature_positions_depth_first(
            target_chunk,
            source_pos,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            feature,
            remaining_modifiers,
            BlockPos {
                x: position.x + feature_random_next_i32_bound(random, 16),
                y: position.y,
                z: position.z + feature_random_next_i32_bound(random, 16),
            },
            source_terrain_heights,
            random,
        ),
        PlacementModifier::Heightmap { heightmap } => {
            let y = simple_vegetation_source_height(
                source_pos,
                source_terrain_heights,
                heightmap,
                position.x,
                position.z,
                settings,
            );
            if y <= settings.noise.min_y {
                0
            } else {
                place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    BlockPos { y, ..position },
                    source_terrain_heights,
                    random,
                )
            }
        }
        PlacementModifier::RandomOffset {
            xz_spread,
            y_spread,
        } => place_simple_vegetation_feature_positions_depth_first(
            target_chunk,
            source_pos,
            biome_source_model,
            settings,
            climate_sampler,
            placed_feature_id,
            feature,
            remaining_modifiers,
            BlockPos {
                x: position.x + sample_triangle_int(random, xz_spread),
                y: position.y + sample_triangle_int(random, y_spread),
                z: position.z + sample_triangle_int(random, xz_spread),
            },
            source_terrain_heights,
            random,
        ),
        PlacementModifier::BlockPredicateFilter { predicate } => {
            if block_predicate_test_in_chunk(target_chunk, settings, predicate, position) {
                place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                )
            } else {
                0
            }
        }
        PlacementModifier::BiomeFilter => {
            if biome_allows_feature_at(
                biome_source_model,
                settings,
                climate_sampler,
                position,
                placed_feature_id,
            ) {
                place_simple_vegetation_feature_positions_depth_first(
                    target_chunk,
                    source_pos,
                    biome_source_model,
                    settings,
                    climate_sampler,
                    placed_feature_id,
                    feature,
                    remaining_modifiers,
                    position,
                    source_terrain_heights,
                    random,
                )
            } else {
                0
            }
        }
        _ => 0,
    }
}

fn block_predicate_context_for_chunk_pos(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
    position: BlockPos,
) -> BlockPredicateContext {
    let block = chunk
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    block_predicate_context_for_state(block, settings.noise.min_y, settings.noise.height)
}

pub(super) fn block_predicate_test_in_chunk(
    chunk: &LevelChunk,
    settings: &NoiseGeneratorSettings,
    predicate: BlockPredicate,
    position: BlockPos,
) -> bool {
    let context = block_predicate_context_for_chunk_pos(chunk, settings, position);
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&context.block),
        BlockPredicate::MatchingBlocksAt { offset_y, blocks } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context = block_predicate_context_for_chunk_pos(chunk, settings, offset_pos);
            blocks.contains(&offset_context.block)
        }
        BlockPredicate::MatchingBlockTag { tag } => block_matches_tag(context.block, tag),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&context.fluid),
        BlockPredicate::MatchingFluidsAt { offset_y, fluids } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context = block_predicate_context_for_chunk_pos(chunk, settings, offset_pos);
            fluids.contains(&offset_context.fluid)
        }
        BlockPredicate::Solid => context.solid,
        BlockPredicate::SolidAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_chunk_pos(chunk, settings, offset_pos).solid
        }
        BlockPredicate::Replaceable => context.replaceable,
        BlockPredicate::ReplaceableAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_chunk_pos(chunk, settings, offset_pos).replaceable
        }
        BlockPredicate::WouldSurvive {
            offset_y,
            state: _,
            survives,
        }
        | BlockPredicate::HasSturdyFace {
            offset_y,
            direction: _,
            sturdy: survives,
        } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y
                && y < settings.noise.min_y + settings.noise.height
                && survives
        }
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y && y < settings.noise.min_y + settings.noise.height
        }
        BlockPredicate::AnyOf { predicates } => predicates
            .iter()
            .any(|predicate| block_predicate_test_in_chunk(chunk, settings, *predicate, position)),
        BlockPredicate::AllOf { predicates } => predicates
            .iter()
            .all(|predicate| block_predicate_test_in_chunk(chunk, settings, *predicate, position)),
        BlockPredicate::Not { predicate } => {
            !block_predicate_test_in_chunk(chunk, settings, *predicate, position)
        }
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => context.unobstructed,
    }
}

pub(super) fn region_static_block_name(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    position: BlockPos,
) -> Option<&'static str> {
    let pos = ChunkPos {
        x: position.x.div_euclid(16),
        z: position.z.div_euclid(16),
    };
    chunks
        .get(&pos)?
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
}

fn block_predicate_context_for_region_pos(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    settings: &NoiseGeneratorSettings,
    position: BlockPos,
) -> BlockPredicateContext {
    let block = region_static_block_name(chunks, position).unwrap_or("minecraft:air");
    block_predicate_context_for_state(block, settings.noise.min_y, settings.noise.height)
}

pub(super) fn block_predicate_test_in_region(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    settings: &NoiseGeneratorSettings,
    predicate: BlockPredicate,
    position: BlockPos,
) -> bool {
    let context = block_predicate_context_for_region_pos(chunks, settings, position);
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&context.block),
        BlockPredicate::MatchingBlocksAt { offset_y, blocks } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context =
                block_predicate_context_for_region_pos(chunks, settings, offset_pos);
            blocks.contains(&offset_context.block)
        }
        BlockPredicate::MatchingBlockTag { tag } => block_matches_tag(context.block, tag),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&context.fluid),
        BlockPredicate::MatchingFluidsAt { offset_y, fluids } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            let offset_context =
                block_predicate_context_for_region_pos(chunks, settings, offset_pos);
            fluids.contains(&offset_context.fluid)
        }
        BlockPredicate::Solid => context.solid,
        BlockPredicate::SolidAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_region_pos(chunks, settings, offset_pos).solid
        }
        BlockPredicate::Replaceable => context.replaceable,
        BlockPredicate::ReplaceableAt { offset_y } => {
            let offset_pos = BlockPos {
                y: position.y + offset_y,
                ..position
            };
            if offset_pos.y < settings.noise.min_y
                || offset_pos.y >= settings.noise.min_y + settings.noise.height
            {
                return false;
            }
            block_predicate_context_for_region_pos(chunks, settings, offset_pos).replaceable
        }
        BlockPredicate::WouldSurvive {
            offset_y,
            state: _,
            survives,
        }
        | BlockPredicate::HasSturdyFace {
            offset_y,
            direction: _,
            sturdy: survives,
        } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y
                && y < settings.noise.min_y + settings.noise.height
                && survives
        }
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = position.y + offset_y;
            y >= settings.noise.min_y && y < settings.noise.min_y + settings.noise.height
        }
        BlockPredicate::AnyOf { predicates } => predicates.iter().any(|predicate| {
            block_predicate_test_in_region(chunks, settings, *predicate, position)
        }),
        BlockPredicate::AllOf { predicates } => predicates.iter().all(|predicate| {
            block_predicate_test_in_region(chunks, settings, *predicate, position)
        }),
        BlockPredicate::Not { predicate } => {
            !block_predicate_test_in_region(chunks, settings, *predicate, position)
        }
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => context.unobstructed,
    }
}

pub(super) fn seedless_noise_salt(id: &str) -> i64 {
    id.bytes().fold(0_i64, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(i64::from(byte))
    })
}

pub(super) fn simple_vegetation_source_height(
    source_pos: ChunkPos,
    source_terrain_heights: SourceTerrainHeights<'_>,
    heightmap: HeightmapKind,
    world_x: i32,
    world_z: i32,
    settings: &NoiseGeneratorSettings,
) -> i32 {
    source_terrain_heights.world_height(
        source_pos,
        heightmap,
        world_x,
        world_z,
        settings.noise.min_y,
    )
}

pub(super) fn place_configured_simple_vegetation_in_target_chunk(
    chunk: &mut LevelChunk,
    settings: &NoiseGeneratorSettings,
    configured_feature: &'static str,
    position: BlockPos,
    random: &mut RandomSourceKind,
) -> usize {
    if position.x.div_euclid(16) != chunk.pos.x || position.z.div_euclid(16) != chunk.pos.z {
        return 0;
    }
    if !(settings.noise.min_y..settings.noise.min_y + settings.noise.height).contains(&position.y) {
        return 0;
    }
    let current = chunk
        .get_block_state_name(position.x, position.y, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let below = chunk
        .get_block_state_name(position.x, position.y - 1, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let above = chunk
        .get_block_state_name(position.x, position.y + 1, position.z)
        .and_then(carver_static_block_name)
        .unwrap_or("minecraft:air");
    let Some(config) = configured_simple_vegetation_block(configured_feature, random, position)
    else {
        return 0;
    };
    let Some(plan) = simple_block_placement_plan(
        &config,
        SimpleBlockPlacementContext {
            origin_block: current,
            below_block: below,
            above_block: above,
        },
        random,
    ) else {
        return 0;
    };

    chunk.set_block_state(position.x, position.y, position.z, plan.state);
    let mut placed = 1;
    if let Some(upper_state) = plan.upper_state {
        if position.y + 1 < settings.noise.min_y + settings.noise.height {
            chunk.set_block_state(position.x, position.y + 1, position.z, upper_state);
            placed += 1;
        }
    }
    placed
}

pub(super) fn sample_triangle_int(random: &mut RandomSourceKind, range: i32) -> i32 {
    feature_random_next_i32_bound(random, range + 1)
        - feature_random_next_i32_bound(random, range + 1)
}
