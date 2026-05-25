use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TreeDecorationResult {
    pub(super) placed_blocks: usize,
    pub(super) context_build_ms: u128,
    pub(super) context_chunks: usize,
}

pub(super) type DecorationBiomeSteps = Vec<&'static [&'static [&'static str]]>;
pub(super) type DecorationBiomeStepsByChunk = HashMap<ChunkPos, DecorationBiomeSteps>;

pub(super) fn source_decoration_biome_steps_cache(
    center_pos: ChunkPos,
    source_radius: i32,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
) -> DecorationBiomeStepsByChunk {
    let mut cache = DecorationBiomeStepsByChunk::new();
    let sample_quart_y = ((settings.sea_level + 1).clamp(
        settings.noise.min_y,
        settings.noise.min_y + settings.noise.height - 1,
    )) >> 2;
    with_noise_snapshot_cache(|| {
        let mut biome_cache: HashMap<(i32, i32, i32), Option<&'static str>> = HashMap::new();
        for source_z in center_pos.z - source_radius..=center_pos.z + source_radius {
            for source_x in center_pos.x - source_radius..=center_pos.x + source_radius {
                let source_pos = ChunkPos {
                    x: source_x,
                    z: source_z,
                };
                let mut steps = Vec::new();
                let mut seen = Vec::new();
                for chunk_z in source_pos.z - 1..=source_pos.z + 1 {
                    for chunk_x in source_pos.x - 1..=source_pos.x + 1 {
                        let chunk_quart_x = chunk_x * 4;
                        let chunk_quart_z = chunk_z * 4;
                        for local_z in 0..4 {
                            for local_x in 0..4 {
                                let quart_x = chunk_quart_x + local_x;
                                let quart_z = chunk_quart_z + local_z;
                                let biome = *biome_cache
                                    .entry((quart_x, sample_quart_y, quart_z))
                                    .or_insert_with(|| {
                                        get_biome(
                                            biome_source_model,
                                            quart_x,
                                            sample_quart_y,
                                            quart_z,
                                            climate_sampler,
                                        )
                                    });
                                let Some(biome) = biome else {
                                    continue;
                                };
                                if seen.contains(&biome) {
                                    continue;
                                }
                                if let Some(generation) = biome_generation_settings(biome) {
                                    seen.push(biome);
                                    steps.push(generation.feature_steps);
                                }
                            }
                        }
                    }
                }
                if steps.is_empty() {
                    steps = possible_biome_feature_steps_for_source(biome_source_model);
                }
                cache.insert(source_pos, steps);
            }
        }
    });
    cache
}

pub(super) fn apply_initial_tree_decoration_to_chunk(
    chunk: &mut LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: Option<&[&'static [&'static [&'static str]]]>,
    precomputed_context: Option<TreeDecorationContextCache>,
    precomputed_source_steps: Option<&DecorationBiomeStepsByChunk>,
) -> TreeDecorationResult {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return TreeDecorationResult::default();
    }

    let chunk_min_x = chunk.pos.x * 16;
    let chunk_min_z = chunk.pos.z * 16;
    let chunk_max_x = chunk_min_x + 15;
    let chunk_max_z = chunk_min_z + 15;
    let mut placed = 0;
    let mut target_terrain_heights = None;
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    if load_surface_rule(settings.id).is_none() {
        return TreeDecorationResult::default();
    }
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let global_features_per_step = if global_biome_steps.is_empty() {
        None
    } else {
        build_features_per_step(&global_biome_steps, true).ok()
    };
    let mut diagnostics = TreeDecorationDiagnostics::default();
    let source_radius = tree_decoration_source_radius();
    let owned_source_region_biome_steps;
    let source_region_biome_steps = if let Some(source_steps) = precomputed_source_steps {
        source_steps
    } else {
        owned_source_region_biome_steps = source_decoration_biome_steps_cache(
            chunk.pos,
            source_radius,
            biome_source_model,
            settings,
            &climate_sampler,
        );
        &owned_source_region_biome_steps
    };
    let target_region_biome_steps = decoration_region_biome_steps
        .map(|steps| steps.to_vec())
        .or_else(|| source_region_biome_steps.get(&chunk.pos).cloned())
        .unwrap_or_else(|| possible_biome_feature_steps_for_source(biome_source_model));
    let context_cache = precomputed_context.unwrap_or_else(|| {
        build_tree_decoration_context_cache(
            chunk.pos,
            source_radius,
            biome_source_model,
            settings,
            seed,
            noise_router,
            &load_surface_rule(settings.id).expect("surface rule was checked above"),
        )
    });
    diagnostics.context_chunks = context_cache.context_chunks;
    diagnostics.context_chunk_build_ms = context_cache.context_chunk_build_ms;
    diagnostics.context_heightmap_ms = context_cache.context_heightmap_ms;

    let target_pos = chunk.pos;
    let terrain_heights = &*target_terrain_heights
        .get_or_insert_with(|| tree_decoration_terrain_heights(chunk, settings));
    placed += apply_initial_simple_vegetation_decoration_to_chunk(
        chunk,
        biome_source_model,
        settings,
        seed,
        &target_region_biome_steps,
        source_region_biome_steps,
        terrain_heights,
        &context_cache,
        SimpleVegetationPhase::BeforeTrees,
    );
    let mut generated_chunks = HashMap::new();
    for (pos, cached) in &context_cache.cached_region_chunks {
        generated_chunks.insert(*pos, TreeContextChunkRef::Lightweight(cached));
    }
    let mut region_overlay = TreeBlockOverlay::default();

    for source_z in target_pos.z - source_radius..=target_pos.z + source_radius {
        for source_x in target_pos.x - source_radius..=target_pos.x + source_radius {
            let source_pos = ChunkPos {
                x: source_x,
                z: source_z,
            };
            let Some(source_terrain_heights) = (if source_pos == target_pos {
                Some(SourceTerrainHeights::Full(terrain_heights))
            } else {
                context_cache
                    .cached_region_chunks
                    .get(&source_pos)
                    .map(SourceTerrainHeights::Lazy)
            }) else {
                continue;
            };
            let Some(source_chunk) = (if source_pos == target_pos {
                Some(TreeContextChunkRef::Full(&*chunk))
            } else {
                context_cache
                    .cached_region_chunks
                    .get(&source_pos)
                    .map(TreeContextChunkRef::Lightweight)
            }) else {
                continue;
            };
            let source_min_x = source_pos.x * 16;
            let source_min_z = source_pos.z * 16;
            let block_context = TreeDecorationBlockContext {
                source_pos,
                source_chunk,
                target_pos,
                target_chunk: &*chunk,
                generated_chunks: &generated_chunks,
                region_overlay: Some(&region_overlay),
            };
            let source_region_biome_steps = source_region_biome_steps
                .get(&source_pos)
                .cloned()
                .unwrap_or_else(|| possible_biome_feature_steps_for_source(biome_source_model));
            let planned_blocks = live_tree_decoration_blocks(
                source_pos,
                seed,
                settings,
                biome_source_model,
                &climate_sampler,
                global_features_per_step.as_deref(),
                &source_region_biome_steps,
                &block_context,
                source_terrain_heights,
                &mut diagnostics,
            );
            for block in planned_blocks {
                diagnostics.output_blocks_seen += 1;
                let started = Instant::now();
                let world_x = source_min_x + block.pos.x;
                let world_z = source_min_z + block.pos.z;
                if world_x < chunk_min_x
                    || world_x > chunk_max_x
                    || world_z < chunk_min_z
                    || world_z > chunk_max_z
                {
                    diagnostics.source_write_filter_ms += started.elapsed().as_millis();
                    continue;
                }
                diagnostics.output_blocks_in_target += 1;

                let current = chunk
                    .get_block_state_name(world_x, block.pos.y, world_z)
                    .unwrap_or("minecraft:air");
                let can_replace = tree_placement_block_can_replace(block.kind, current);
                if !can_replace {
                    diagnostics.source_write_filter_ms += started.elapsed().as_millis();
                    continue;
                }
                chunk.set_block_state(world_x, block.pos.y, world_z, block.state);
                region_overlay.insert((world_x, block.pos.y, world_z), block.state);
                diagnostics.output_blocks_written += 1;
                diagnostics.source_write_filter_ms += started.elapsed().as_millis();
                placed += 1;
            }
        }
    }
    if std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_DEBUG").is_some() {
        eprintln!(
            "[tree-decoration-debug] context_chunks={} context_build={}ms context_heightmaps={}ms sources={} source_total={}us source_context_clone={}us source_biomes={}ms feature_sort={}ms source_plan={}ms source_context_map={}ms feature_calls={} tree_feature_calls={} attempts={} candidates={} candidate_biomes={}ms validation={}ms accepts={} rejects={} placement_plan={}ms plan_blocks={} filter={}ms filtered_blocks={} output_seen={} output_in_target={} output_written={} write_filter={}ms",
            diagnostics.context_chunks,
            diagnostics.context_chunk_build_ms,
            diagnostics.context_heightmap_ms,
            diagnostics.sources_evaluated,
            diagnostics.source_total_us,
            diagnostics.source_context_clone_us,
            diagnostics.source_biome_steps_ms,
            diagnostics.source_feature_sort_ms,
            diagnostics.source_plan_ms,
            diagnostics.source_context_map_ms,
            diagnostics.feature_calls_total,
            diagnostics.tree_feature_calls,
            diagnostics.tree_attempts,
            diagnostics.tree_candidates,
            diagnostics.candidate_biome_ms,
            diagnostics.validation_ms,
            diagnostics.validation_accepts,
            diagnostics.validation_rejects,
            diagnostics.placement_plan_ms,
            diagnostics.placement_plan_blocks,
            diagnostics.filter_ms,
            diagnostics.filtered_blocks,
            diagnostics.output_blocks_seen,
            diagnostics.output_blocks_in_target,
            diagnostics.output_blocks_written,
            diagnostics.source_write_filter_ms,
        );
    }

    let terrain_heights = &*target_terrain_heights
        .get_or_insert_with(|| tree_decoration_terrain_heights(chunk, settings));
    placed += apply_initial_simple_vegetation_decoration_to_chunk(
        chunk,
        biome_source_model,
        settings,
        seed,
        &target_region_biome_steps,
        source_region_biome_steps,
        terrain_heights,
        &context_cache,
        SimpleVegetationPhase::AfterTrees,
    );
    TreeDecorationResult {
        placed_blocks: placed,
        context_build_ms: diagnostics.context_chunk_build_ms,
        context_chunks: diagnostics.context_chunks,
    }
}

pub(super) fn apply_initial_tree_decoration_from_source_into_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    decoration_region_biome_steps: Option<&[&'static [&'static [&'static str]]]>,
) -> TreeDecorationResult {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return TreeDecorationResult::default();
    }
    let Some(source_chunk) = chunks.get(&source_pos) else {
        return TreeDecorationResult::default();
    };

    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let global_features_per_step = if global_biome_steps.is_empty() {
        None
    } else {
        build_features_per_step(&global_biome_steps, true).ok()
    };
    let region_biome_steps = decoration_region_biome_steps
        .map(|steps| steps.to_vec())
        .unwrap_or_else(|| {
            possible_biome_feature_steps_for_decoration_region(
                source_pos,
                biome_source_model,
                settings,
                &climate_sampler,
            )
        });
    let terrain_heights = tree_decoration_terrain_heights(source_chunk, settings);
    let mut generated_chunks = HashMap::new();
    for (pos, chunk) in chunks.iter() {
        generated_chunks.insert(*pos, TreeContextChunkRef::Full(chunk));
    }
    let mut region_overlay = TreeBlockOverlay::default();
    let block_context = TreeDecorationBlockContext {
        source_pos,
        source_chunk: TreeContextChunkRef::Full(source_chunk),
        target_pos: source_pos,
        target_chunk: source_chunk,
        generated_chunks: &generated_chunks,
        region_overlay: Some(&region_overlay),
    };
    let mut diagnostics = TreeDecorationDiagnostics::default();
    let planned_blocks = live_tree_decoration_blocks(
        source_pos,
        seed,
        settings,
        biome_source_model,
        &climate_sampler,
        global_features_per_step.as_deref(),
        &region_biome_steps,
        &block_context,
        SourceTerrainHeights::Full(&terrain_heights),
        &mut diagnostics,
    );
    let source_min_x = source_pos.x * 16;
    let source_min_z = source_pos.z * 16;
    let mut placed = 0;
    for block in planned_blocks {
        let world_x = source_min_x + block.pos.x;
        let world_z = source_min_z + block.pos.z;
        let target_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if (target_pos.x - source_pos.x).abs() > 1 || (target_pos.z - source_pos.z).abs() > 1 {
            continue;
        }
        let Some(target_chunk) = chunks.get_mut(&target_pos) else {
            continue;
        };
        let current = target_chunk
            .get_block_state_name(world_x, block.pos.y, world_z)
            .unwrap_or("minecraft:air");
        let can_replace = tree_placement_block_can_replace(block.kind, current);
        if can_replace {
            target_chunk.set_block_state(world_x, block.pos.y, world_z, block.state);
            region_overlay.insert((world_x, block.pos.y, world_z), block.state);
            placed += 1;
        }
    }

    TreeDecorationResult {
        placed_blocks: placed,
        context_build_ms: 0,
        context_chunks: chunks.len().saturating_sub(1),
    }
}

pub(super) fn apply_initial_simple_vegetation_decoration_from_source_into_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    phase: SimpleVegetationPhase,
) -> usize {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return 0;
    }
    let Some(source_chunk) = chunks.get(&source_pos) else {
        return 0;
    };
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let global_biome_steps = possible_biome_feature_steps_for_source(biome_source_model);
    let feature_source_steps = if global_biome_steps.is_empty() {
        possible_biome_feature_steps_for_decoration_region(
            source_pos,
            biome_source_model,
            settings,
            &climate_sampler,
        )
    } else {
        global_biome_steps
    };
    if feature_source_steps.is_empty() {
        return 0;
    }
    let features_per_step = match build_features_per_step(&feature_source_steps, true) {
        Ok(features) => features,
        Err(_) => return 0,
    };
    let possible_steps = possible_biome_feature_steps_for_decoration_region(
        source_pos,
        biome_source_model,
        settings,
        &climate_sampler,
    );
    if possible_steps.is_empty() {
        return 0;
    }
    let source_terrain_heights = tree_decoration_terrain_heights(source_chunk, settings);
    let plan = biome_decoration_feature_plan(
        seed,
        source_pos.x,
        source_pos.z,
        settings.noise.min_y.div_euclid(16),
        &features_per_step,
        &possible_steps,
    );

    let mut placed = 0;
    for call in plan.feature_calls.iter().filter(|call| {
        call.step_index == GenerationDecorationStep::VegetalDecoration as usize
            && placed_simple_vegetation_feature(call.feature).is_some()
            && simple_vegetation_phase(call.feature) == phase
    }) {
        let Some(feature) = placed_simple_vegetation_feature(call.feature) else {
            continue;
        };
        let mut random = RandomSourceKind::new(call.seed, RandomAlgorithm::Xoroshiro);
        placed += place_simple_vegetation_feature_positions_depth_first_in_region(
            chunks,
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
            SourceTerrainHeights::Full(&source_terrain_heights),
            &mut random,
        );
    }
    placed
}

#[allow(clippy::too_many_arguments)]
fn place_simple_vegetation_feature_positions_depth_first_in_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
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
        return place_configured_simple_vegetation_in_region(
            chunks,
            source_pos,
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
                placed += place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
                placed += place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
                placed += place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
                place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
        PlacementModifier::InSquare => {
            place_simple_vegetation_feature_positions_depth_first_in_region(
                chunks,
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
            )
        }
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
                place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
        } => place_simple_vegetation_feature_positions_depth_first_in_region(
            chunks,
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
            if block_predicate_test_in_region(chunks, settings, predicate, position) {
                place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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
                place_simple_vegetation_feature_positions_depth_first_in_region(
                    chunks,
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

pub(super) fn place_configured_simple_vegetation_in_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    configured_feature: &'static str,
    position: BlockPos,
    random: &mut RandomSourceKind,
) -> usize {
    let target_pos = ChunkPos {
        x: position.x.div_euclid(16),
        z: position.z.div_euclid(16),
    };
    if (target_pos.x - source_pos.x).abs() > 1 || (target_pos.z - source_pos.z).abs() > 1 {
        return 0;
    }
    if !(settings.noise.min_y..settings.noise.min_y + settings.noise.height).contains(&position.y) {
        return 0;
    }
    let current = region_static_block_name(chunks, position).unwrap_or("minecraft:air");
    let below = region_static_block_name(
        chunks,
        BlockPos {
            y: position.y - 1,
            ..position
        },
    )
    .unwrap_or("minecraft:air");
    let above = region_static_block_name(
        chunks,
        BlockPos {
            y: position.y + 1,
            ..position
        },
    )
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

    let Some(chunk) = chunks.get_mut(&target_pos) else {
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

pub(super) fn tree_placement_block_can_replace(
    kind: TreePlacementBlockKind,
    current: &str,
) -> bool {
    match kind {
        TreePlacementBlockKind::DirtBelowTrunk => {
            !block_matches_tag(current, "minecraft:cannot_replace_below_tree_trunk")
        }
        TreePlacementBlockKind::Log | TreePlacementBlockKind::Leaves => {
            tree_valid_pos(current) || block_matches_tag(current, "minecraft:logs")
        }
        TreePlacementBlockKind::GroundCover => {
            matches!(
                current,
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            )
        }
    }
}

#[derive(Clone)]
pub(super) struct LightweightTreeContextChunk {
    pub(super) terrain_heights: TreeDecorationHeights,
    pub(super) min_y: i32,
    pub(super) max_y: i32,
}

impl LightweightTreeContextChunk {
    pub(super) fn synthetic_block_state(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> &'static str {
        if world_y < self.min_y || world_y >= self.max_y {
            return "minecraft:air";
        }
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_z * 16 + local_x;
        let ocean_floor = self.terrain_heights.ocean_floor[index];
        let world_surface = self.terrain_heights.world_surface[index];
        if world_y >= world_surface {
            "minecraft:air"
        } else if world_y >= ocean_floor {
            "minecraft:water"
        } else if world_y == ocean_floor - 1 && world_surface <= ocean_floor {
            "minecraft:grass_block"
        } else {
            "minecraft:stone"
        }
    }
}

pub(super) struct TreeDecorationContextCache {
    pub(super) cached_region_chunks: HashMap<ChunkPos, LightweightTreeContextChunk>,
    pub(super) context_chunk_build_ms: u128,
    pub(super) context_heightmap_ms: u128,
    pub(super) context_chunks: usize,
}

pub(super) fn build_tree_decoration_context_cache(
    chunk_pos: ChunkPos,
    source_radius: i32,
    _biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    _surface_rule: &DynSurfaceRule,
) -> TreeDecorationContextCache {
    let mut cached_region_chunks = HashMap::new();
    let radius = source_radius.max(0);
    let diameter = radius * 2 + 1;
    let mut context_positions =
        Vec::with_capacity((diameter * diameter).saturating_sub(1) as usize);
    for region_z in chunk_pos.z - radius..=chunk_pos.z + radius {
        for region_x in chunk_pos.x - radius..=chunk_pos.x + radius {
            let region_pos = ChunkPos {
                x: region_x,
                z: region_z,
            };
            if region_pos != chunk_pos {
                context_positions.push(region_pos);
            }
        }
    }
    let context_started = Instant::now();
    let region_chunks =
        build_lightweight_tree_context_chunks(&context_positions, settings, seed, noise_router);
    let context_chunk_build_ms = context_started.elapsed().as_millis();
    let heightmap_started = Instant::now();
    for (region_pos, region_chunk) in region_chunks {
        cached_region_chunks.insert(region_pos, region_chunk);
    }
    TreeDecorationContextCache {
        cached_region_chunks,
        context_chunk_build_ms,
        context_heightmap_ms: heightmap_started.elapsed().as_millis(),
        context_chunks: context_positions.len(),
    }
}

pub(super) fn build_lightweight_tree_context_chunks(
    context_positions: &[ChunkPos],
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> Vec<(ChunkPos, LightweightTreeContextChunk)> {
    let build_one = |region_pos: ChunkPos| {
        let terrain_heights =
            noise_tree_context_heights_inner(region_pos, settings, seed, noise_router);
        (
            region_pos,
            LightweightTreeContextChunk {
                terrain_heights,
                min_y: settings.noise.min_y,
                max_y: settings.noise.min_y + settings.noise.height,
            },
        )
    };

    if context_positions.len() <= 1 {
        return with_noise_snapshot_cache(|| {
            context_positions
                .iter()
                .copied()
                .map(build_one)
                .collect::<Vec<_>>()
        });
    }

    let worker_count = tree_context_worker_count(context_positions.len());
    if worker_count <= 1 {
        return with_noise_snapshot_cache(|| {
            context_positions
                .iter()
                .copied()
                .map(build_one)
                .collect::<Vec<_>>()
        });
    }

    let settings = *settings;
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);
        for worker_index in 0..worker_count {
            let positions = context_positions
                .iter()
                .copied()
                .skip(worker_index)
                .step_by(worker_count)
                .collect::<Vec<_>>();
            handles.push(scope.spawn(move || {
                with_noise_snapshot_cache(|| {
                    positions
                        .into_iter()
                        .map(|region_pos| {
                            let terrain_heights = noise_tree_context_heights_inner(
                                region_pos,
                                &settings,
                                seed,
                                noise_router,
                            );
                            (
                                region_pos,
                                LightweightTreeContextChunk {
                                    terrain_heights,
                                    min_y: settings.noise.min_y,
                                    max_y: settings.noise.min_y + settings.noise.height,
                                },
                            )
                        })
                        .collect::<Vec<_>>()
                })
            }));
        }

        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .expect("tree context worker should not panic")
                    .into_iter()
            })
            .collect::<Vec<_>>()
    })
}

fn tree_context_worker_count(context_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let default_workers = available.min(8).min(context_count).max(1);
    std::env::var("RUSTCRAFT_WORLDGEN_TREE_CONTEXT_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|requested| requested.clamp(1, context_count.max(1)))
        .unwrap_or(default_workers)
}

pub(super) fn tree_decoration_source_radius() -> i32 {
    std::env::var("RUSTCRAFT_WORLDGEN_TREE_SOURCE_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1)
        .clamp(0, 1)
}

pub(super) fn noise_tree_context_heights(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> TreeDecorationHeights {
    with_noise_snapshot_cache(|| {
        noise_tree_context_heights_inner(pos, settings, seed, noise_router)
    })
}

fn noise_tree_context_heights_inner(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> TreeDecorationHeights {
    let min_y = settings.noise.min_y;
    let height = settings.noise.height;
    let chunk_min_x = pos.x * 16;
    let chunk_min_z = pos.z * 16;
    let cell_width = settings.noise.cell_width();
    let cell_height = settings.noise.cell_height();
    let cell_count_xz = 16 / cell_width;
    let cell_noise_min_y = min_y.div_euclid(cell_height);

    let mut noise_chunk = NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let mut aquifer = settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            &mut noise_chunk,
            chunk_min_x,
            chunk_min_x + 15,
            chunk_min_z,
            chunk_min_z + 15,
            min_y,
            height,
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });
    let mut ocean_floor = [min_y; 16 * 16];
    let mut world_surface = [min_y; 16 * 16];
    let mut motion_blocking = [min_y; 16 * 16];
    let mut motion_blocking_no_leaves = [min_y; 16 * 16];
    let mut found_ocean_floor = [false; 16 * 16];
    let mut found_world_surface = [false; 16 * 16];
    let mut remaining_ocean_floor = 16 * 16;
    let mut remaining_world_surface = 16 * 16;
    let mut density_samples = 0_usize;
    let mut fluid_samples = 0_usize;
    let mut lowest_sampled_y = i32::MAX;
    let mut highest_sampled_y = i32::MIN;
    let mut scan_top_by_column = [min_y; 16 * 16];
    let mut max_scan_top_y = min_y;
    for local_z in 0..16 {
        for local_x in 0..16 {
            let world_x = chunk_min_x + local_x as i32;
            let world_z = chunk_min_z + local_z as i32;
            // Java already has neighbor chunks here. This lightweight fallback
            // only needs the worldgen heightmaps, so bound the scan using the
            // same preliminary surface signal Java feeds into aquifers/surface rules.
            let top_y = (noise_chunk.preliminary_surface_level(world_x, world_z) + 64)
                .clamp(min_y, min_y + height - 1);
            scan_top_by_column[local_z * 16 + local_x] = top_y;
            max_scan_top_y = max_scan_top_y.max(top_y);
        }
    }
    let max_scan_top_cell_y = (max_scan_top_y - min_y).div_euclid(cell_height);

    'cells: for cell_x_index in 0..cell_count_xz {
        noise_chunk.advance_cell_x(cell_x_index);
        for cell_z_index in 0..cell_count_xz {
            for cell_y_index in (0..=max_scan_top_cell_y).rev() {
                noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
                for y_in_cell in (0..cell_height).rev() {
                    let pos_y = (cell_noise_min_y + cell_y_index) * cell_height + y_in_cell;
                    let factor_y = y_in_cell as f64 / cell_height as f64;
                    noise_chunk.update_for_y(pos_y, factor_y);
                    for x_in_cell in 0..cell_width {
                        let pos_x = chunk_min_x + cell_x_index * cell_width + x_in_cell;
                        let local_x = (pos_x & 15) as usize;
                        let factor_x = x_in_cell as f64 / cell_width as f64;
                        noise_chunk.update_for_x(pos_x, factor_x);
                        for z_in_cell in 0..cell_width {
                            let pos_z = chunk_min_z + cell_z_index * cell_width + z_in_cell;
                            let local_z = (pos_z & 15) as usize;
                            let factor_z = z_in_cell as f64 / cell_width as f64;
                            noise_chunk.update_for_z(pos_z, factor_z);
                            let index = local_z * 16 + local_x;
                            if pos_y > scan_top_by_column[index] {
                                continue;
                            }
                            if found_ocean_floor[index] && found_world_surface[index] {
                                continue;
                            }
                            density_samples += 1;
                            lowest_sampled_y = lowest_sampled_y.min(pos_y);
                            highest_sampled_y = highest_sampled_y.max(pos_y);
                            let density = noise_chunk.interpolated_density(pos_x, pos_y, pos_z);
                            let block_kind = noise_context_heightmap_block_kind(
                                aquifer.as_mut(),
                                &noise_chunk,
                                settings,
                                pos_x,
                                pos_y,
                                pos_z,
                                density,
                            );
                            if block_kind == NoiseHeightmapBlockKind::Fluid {
                                fluid_samples += 1;
                            }
                            if block_kind == NoiseHeightmapBlockKind::Air {
                                continue;
                            }
                            if !found_world_surface[index] {
                                world_surface[index] = pos_y + 1;
                                motion_blocking[index] = pos_y + 1;
                                motion_blocking_no_leaves[index] = pos_y + 1;
                                found_world_surface[index] = true;
                                remaining_world_surface -= 1;
                            }
                            if block_kind == NoiseHeightmapBlockKind::Solid {
                                ocean_floor[index] = pos_y + 1;
                                found_ocean_floor[index] = true;
                                remaining_ocean_floor -= 1;
                            }
                            if remaining_world_surface == 0 && remaining_ocean_floor == 0 {
                                break 'cells;
                            }
                        }
                    }
                }
            }
        }
        noise_chunk.swap_slices();
    }

    if std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_HEIGHT_DEBUG").is_some() {
        eprintln!(
            "[tree-height-debug] chunk=({}, {}) density_samples={} fluid_samples={} y_range={}..{} remaining_world_surface={} remaining_ocean_floor={}",
            pos.x,
            pos.z,
            density_samples,
            fluid_samples,
            lowest_sampled_y,
            highest_sampled_y,
            remaining_world_surface,
            remaining_ocean_floor
        );
    }

    TreeDecorationHeights {
        ocean_floor,
        world_surface,
        motion_blocking,
        motion_blocking_no_leaves,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoiseHeightmapBlockKind {
    Air,
    Fluid,
    Solid,
}

fn noise_context_heightmap_block_kind(
    aquifer: Option<&mut NoiseBasedAquifer>,
    noise_chunk: &NoiseChunk,
    settings: &NoiseGeneratorSettings,
    x: i32,
    y: i32,
    z: i32,
    density: f64,
) -> NoiseHeightmapBlockKind {
    if density > 0.0 {
        return NoiseHeightmapBlockKind::Solid;
    }

    let substance = if let Some(aquifer) = aquifer {
        aquifer.compute_substance(noise_chunk, x, y, z, density)
    } else {
        Some(global_fluid_status(y, settings.sea_level, settings.default_fluid).at(y))
    };
    match substance {
        None => NoiseHeightmapBlockKind::Solid,
        Some("minecraft:water" | "minecraft:lava") => NoiseHeightmapBlockKind::Fluid,
        Some(_) => NoiseHeightmapBlockKind::Air,
    }
}
