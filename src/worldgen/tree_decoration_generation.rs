use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TreeDecorationResult {
    pub(super) placed_blocks: usize,
    pub(super) context_build_ms: u128,
    pub(super) context_chunks: usize,
}

pub(super) type DecorationBiomeSteps = Vec<&'static [&'static [&'static str]]>;
pub(super) type DecorationBiomeStepsByChunk = HashMap<ChunkPos, DecorationBiomeSteps>;

struct ChunkTreeDecorationBlocksInput<'a> {
    chunk: &'a mut LevelChunk,
    biome_source_model: &'a BiomeSourceModel,
    settings: &'a NoiseGeneratorSettings,
    seed: i64,
    climate_sampler: &'a ClimateSampler,
    global_features_per_step: Option<&'a [StepFeatureDataModel]>,
    source_region_biome_steps: &'a DecorationBiomeStepsByChunk,
    context_cache: &'a TreeDecorationContextCache,
    target_terrain_heights: &'a TreeDecorationHeights,
    source_radius: i32,
    diagnostics: &'a mut TreeDecorationDiagnostics,
}

struct ChunkDecorationBounds {
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
}

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

    let mut placed = 0;
    let mut target_terrain_heights = None;
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let Some(surface_rule) = load_surface_rule(settings.id) else {
        return TreeDecorationResult::default();
    };
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
            &surface_rule,
        )
    });
    diagnostics.context_chunks = context_cache.context_chunks;
    diagnostics.context_chunk_build_ms = context_cache.context_chunk_build_ms;
    diagnostics.context_heightmap_ms = context_cache.context_heightmap_ms;

    let terrain_heights = &*target_terrain_heights
        .get_or_insert_with(|| tree_decoration_terrain_heights(chunk, settings));
    placed +=
        apply_initial_simple_vegetation_decoration_to_chunk(SimpleVegetationDecorationInput {
            chunk,
            biome_source_model,
            settings,
            seed,
            decoration_region_biome_steps: &target_region_biome_steps,
            source_region_biome_steps,
            target_terrain_heights: terrain_heights,
            context_cache: &context_cache,
            phase: SimpleVegetationPhase::BeforeTrees,
        });
    placed += apply_tree_decoration_blocks_to_chunk(ChunkTreeDecorationBlocksInput {
        chunk,
        biome_source_model,
        settings,
        seed,
        climate_sampler: &climate_sampler,
        global_features_per_step: global_features_per_step.as_deref(),
        source_region_biome_steps,
        context_cache: &context_cache,
        target_terrain_heights: terrain_heights,
        source_radius,
        diagnostics: &mut diagnostics,
    });
    print_tree_decoration_debug_report(&diagnostics);

    let terrain_heights = &*target_terrain_heights
        .get_or_insert_with(|| tree_decoration_terrain_heights(chunk, settings));
    placed +=
        apply_initial_simple_vegetation_decoration_to_chunk(SimpleVegetationDecorationInput {
            chunk,
            biome_source_model,
            settings,
            seed,
            decoration_region_biome_steps: &target_region_biome_steps,
            source_region_biome_steps,
            target_terrain_heights: terrain_heights,
            context_cache: &context_cache,
            phase: SimpleVegetationPhase::AfterTrees,
        });
    TreeDecorationResult {
        placed_blocks: placed,
        context_build_ms: diagnostics.context_chunk_build_ms,
        context_chunks: diagnostics.context_chunks,
    }
}

fn apply_tree_decoration_blocks_to_chunk(mut input: ChunkTreeDecorationBlocksInput<'_>) -> usize {
    let target_pos = input.chunk.pos;
    let bounds = ChunkDecorationBounds {
        min_x: target_pos.x * 16,
        max_x: target_pos.x * 16 + 15,
        min_z: target_pos.z * 16,
        max_z: target_pos.z * 16 + 15,
    };
    let generated_chunks = lightweight_context_refs(input.context_cache);
    let mut region_overlay = TreeBlockOverlay::default();
    let mut placed = 0;

    for source_z in target_pos.z - input.source_radius..=target_pos.z + input.source_radius {
        for source_x in target_pos.x - input.source_radius..=target_pos.x + input.source_radius {
            let source_pos = ChunkPos {
                x: source_x,
                z: source_z,
            };
            placed += apply_tree_decoration_source_to_chunk(
                &mut region_overlay,
                &generated_chunks,
                &bounds,
                &mut input,
                source_pos,
            );
        }
    }
    placed
}

fn lightweight_context_refs(
    context_cache: &TreeDecorationContextCache,
) -> HashMap<ChunkPos, TreeContextChunkRef<'_>> {
    context_cache
        .cached_region_chunks
        .iter()
        .map(|(pos, cached)| (*pos, TreeContextChunkRef::Lightweight(cached)))
        .collect()
}

fn apply_tree_decoration_source_to_chunk(
    region_overlay: &mut TreeBlockOverlay,
    generated_chunks: &HashMap<ChunkPos, TreeContextChunkRef<'_>>,
    bounds: &ChunkDecorationBounds,
    input: &mut ChunkTreeDecorationBlocksInput<'_>,
    source_pos: ChunkPos,
) -> usize {
    let Some(source_terrain_heights) = tree_source_terrain_heights(
        input.chunk.pos,
        input.target_terrain_heights,
        input.context_cache,
        source_pos,
    ) else {
        return 0;
    };
    let Some(source_chunk) = tree_source_chunk_ref(
        input.chunk.pos,
        &*input.chunk,
        input.context_cache,
        source_pos,
    ) else {
        return 0;
    };
    let block_context = TreeDecorationBlockContext {
        source_pos,
        source_chunk,
        target_pos: input.chunk.pos,
        target_chunk: &*input.chunk,
        generated_chunks,
        region_overlay: Some(region_overlay),
    };
    let source_region_biome_steps = input
        .source_region_biome_steps
        .get(&source_pos)
        .cloned()
        .unwrap_or_else(|| possible_biome_feature_steps_for_source(input.biome_source_model));
    let planned_blocks = live_tree_decoration_blocks(
        source_pos,
        input.seed,
        input.settings,
        input.biome_source_model,
        input.climate_sampler,
        input.global_features_per_step,
        &source_region_biome_steps,
        &block_context,
        source_terrain_heights,
        input.diagnostics,
    );
    write_planned_tree_blocks_to_chunk(
        input.chunk,
        input.diagnostics,
        region_overlay,
        bounds,
        source_pos,
        planned_blocks,
    )
}

fn tree_source_terrain_heights<'a>(
    target_pos: ChunkPos,
    target_terrain_heights: &'a TreeDecorationHeights,
    context_cache: &'a TreeDecorationContextCache,
    source_pos: ChunkPos,
) -> Option<SourceTerrainHeights<'a>> {
    if source_pos == target_pos {
        Some(SourceTerrainHeights::Full(target_terrain_heights))
    } else {
        context_cache
            .cached_region_chunks
            .get(&source_pos)
            .map(SourceTerrainHeights::Lazy)
    }
}

fn tree_source_chunk_ref<'a>(
    target_pos: ChunkPos,
    target_chunk: &'a LevelChunk,
    context_cache: &'a TreeDecorationContextCache,
    source_pos: ChunkPos,
) -> Option<TreeContextChunkRef<'a>> {
    if source_pos == target_pos {
        Some(TreeContextChunkRef::Full(target_chunk))
    } else {
        context_cache
            .cached_region_chunks
            .get(&source_pos)
            .map(TreeContextChunkRef::Lightweight)
    }
}

fn write_planned_tree_blocks_to_chunk(
    chunk: &mut LevelChunk,
    diagnostics: &mut TreeDecorationDiagnostics,
    region_overlay: &mut TreeBlockOverlay,
    bounds: &ChunkDecorationBounds,
    source_pos: ChunkPos,
    planned_blocks: Vec<TreePlacementBlock>,
) -> usize {
    let mut placed = 0;
    let source_min_x = source_pos.x * 16;
    let source_min_z = source_pos.z * 16;
    for block in planned_blocks {
        diagnostics.output_blocks_seen += 1;
        let started = Instant::now();
        let world_x = source_min_x + block.pos.x;
        let world_z = source_min_z + block.pos.z;
        if !tree_block_is_inside_target(bounds, world_x, world_z) {
            diagnostics.source_write_filter_ms += started.elapsed().as_millis();
            continue;
        }
        diagnostics.output_blocks_in_target += 1;

        let current = chunk
            .get_block_state_name(world_x, block.pos.y, world_z)
            .unwrap_or("minecraft:air");
        if !tree_placement_block_can_replace(block.kind, current) {
            diagnostics.source_write_filter_ms += started.elapsed().as_millis();
            continue;
        }
        chunk.set_block_state(world_x, block.pos.y, world_z, block.state);
        region_overlay.insert((world_x, block.pos.y, world_z), block.state);
        diagnostics.output_blocks_written += 1;
        diagnostics.source_write_filter_ms += started.elapsed().as_millis();
        placed += 1;
    }
    placed
}

fn tree_block_is_inside_target(bounds: &ChunkDecorationBounds, world_x: i32, world_z: i32) -> bool {
    world_x >= bounds.min_x
        && world_x <= bounds.max_x
        && world_z >= bounds.min_z
        && world_z <= bounds.max_z
}

fn print_tree_decoration_debug_report(diagnostics: &TreeDecorationDiagnostics) {
    if std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_DEBUG").is_none() {
        return;
    }
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
            RegionSimpleVegetationPlacementInput {
                chunks,
                source_pos,
                biome_source_model,
                settings,
                climate_sampler: &climate_sampler,
                placed_feature_id: call.feature,
                feature: &feature,
                modifiers: &feature.placement,
                position: BlockPos {
                    x: source_pos.x * 16,
                    y: settings.noise.min_y,
                    z: source_pos.z * 16,
                },
                source_terrain_heights: SourceTerrainHeights::Full(&source_terrain_heights),
                random: &mut random,
            },
        );
    }
    placed
}

struct RegionSimpleVegetationPlacementInput<'a> {
    chunks: &'a mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    biome_source_model: &'a BiomeSourceModel,
    settings: &'a NoiseGeneratorSettings,
    climate_sampler: &'a ClimateSampler,
    placed_feature_id: &'a str,
    feature: &'a PlacedSimpleVegetationFeature,
    modifiers: &'a [PlacementModifier],
    position: BlockPos,
    source_terrain_heights: SourceTerrainHeights<'a>,
    random: &'a mut RandomSourceKind,
}

fn place_simple_vegetation_feature_positions_depth_first_in_region(
    input: RegionSimpleVegetationPlacementInput<'_>,
) -> usize {
    let mut walker = RegionSimpleVegetationPlacementWalker {
        chunks: input.chunks,
        source_pos: input.source_pos,
        biome_source_model: input.biome_source_model,
        settings: input.settings,
        climate_sampler: input.climate_sampler,
        placed_feature_id: input.placed_feature_id,
        feature: input.feature,
        source_terrain_heights: input.source_terrain_heights,
        random: input.random,
    };
    walker.place(input.modifiers, input.position)
}

struct RegionSimpleVegetationPlacementWalker<'a> {
    chunks: &'a mut BTreeMap<ChunkPos, LevelChunk>,
    source_pos: ChunkPos,
    biome_source_model: &'a BiomeSourceModel,
    settings: &'a NoiseGeneratorSettings,
    climate_sampler: &'a ClimateSampler,
    placed_feature_id: &'a str,
    feature: &'a PlacedSimpleVegetationFeature,
    source_terrain_heights: SourceTerrainHeights<'a>,
    random: &'a mut RandomSourceKind,
}

impl RegionSimpleVegetationPlacementWalker<'_> {
    fn place(&mut self, modifiers: &[PlacementModifier], position: BlockPos) -> usize {
        let Some((modifier, remaining_modifiers)) = modifiers.split_first() else {
            return place_configured_simple_vegetation_in_region(
                self.chunks,
                self.source_pos,
                self.settings,
                self.feature.configured_feature,
                position,
                self.random,
            );
        };

        match *modifier {
            PlacementModifier::Count { count } => {
                self.place_repeated(count.max(0), remaining_modifiers, position)
            }
            PlacementModifier::CountProvider { provider, .. } => {
                let count = sample_int_provider(provider, self.random).clamp(0, i32::MAX);
                self.place_repeated(count, remaining_modifiers, position)
            }
            PlacementModifier::NoiseThresholdCount {
                noise_level,
                below_noise,
                above_noise,
                ..
            } => self.place_noise_threshold_count(
                noise_level,
                below_noise,
                above_noise,
                remaining_modifiers,
                position,
            ),
            PlacementModifier::RarityFilter { chance } => {
                self.place_rarity_filter(chance, remaining_modifiers, position)
            }
            PlacementModifier::InSquare => self.place_in_square(remaining_modifiers, position),
            PlacementModifier::Heightmap { heightmap } => {
                self.place_heightmap(heightmap, remaining_modifiers, position)
            }
            PlacementModifier::RandomOffset {
                xz_spread,
                y_spread,
            } => self.place_random_offset(xz_spread, y_spread, remaining_modifiers, position),
            PlacementModifier::BlockPredicateFilter { predicate } => {
                self.place_block_predicate(predicate, remaining_modifiers, position)
            }
            PlacementModifier::BiomeFilter => {
                self.place_biome_filter(remaining_modifiers, position)
            }
            _ => 0,
        }
    }

    fn place_repeated(
        &mut self,
        count: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let mut placed = 0;
        for _ in 0..count {
            placed += self.place(remaining_modifiers, position);
        }
        placed
    }

    fn place_noise_threshold_count(
        &mut self,
        noise_level: f64,
        below_noise: i32,
        above_noise: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let noise = vegetation_flower_noise(
            position.x,
            position.z,
            seedless_noise_salt(self.placed_feature_id),
            0.005,
        );
        let count = if noise < noise_level {
            below_noise
        } else {
            above_noise
        };
        self.place_repeated(count.max(0), remaining_modifiers, position)
    }

    fn place_rarity_filter(
        &mut self,
        chance: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if chance > 0 && feature_random_next_i32_bound(self.random, chance) == 0 {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
    }

    fn place_in_square(
        &mut self,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let x_offset = feature_random_next_i32_bound(self.random, 16);
        let z_offset = feature_random_next_i32_bound(self.random, 16);
        self.place(
            remaining_modifiers,
            BlockPos {
                x: position.x + x_offset,
                y: position.y,
                z: position.z + z_offset,
            },
        )
    }

    fn place_heightmap(
        &mut self,
        heightmap: HeightmapKind,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let y = simple_vegetation_source_height(
            self.source_pos,
            self.source_terrain_heights,
            heightmap,
            position.x,
            position.z,
            self.settings,
        );
        if y <= self.settings.noise.min_y {
            0
        } else {
            self.place(remaining_modifiers, BlockPos { y, ..position })
        }
    }

    fn place_random_offset(
        &mut self,
        xz_spread: i32,
        y_spread: i32,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        let x_offset = sample_triangle_int(self.random, xz_spread);
        let y_offset = sample_triangle_int(self.random, y_spread);
        let z_offset = sample_triangle_int(self.random, xz_spread);
        self.place(
            remaining_modifiers,
            BlockPos {
                x: position.x + x_offset,
                y: position.y + y_offset,
                z: position.z + z_offset,
            },
        )
    }

    fn place_block_predicate(
        &mut self,
        predicate: BlockPredicate,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if block_predicate_test_in_region(self.chunks, self.settings, predicate, position) {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
    }

    fn place_biome_filter(
        &mut self,
        remaining_modifiers: &[PlacementModifier],
        position: BlockPos,
    ) -> usize {
        if biome_allows_feature_at(
            self.biome_source_model,
            self.settings,
            self.climate_sampler,
            position,
            self.placed_feature_id,
        ) {
            self.place(remaining_modifiers, position)
        } else {
            0
        }
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
            .flat_map(|handle| match handle.join() {
                Ok(chunks) => chunks.into_iter(),
                Err(payload) => std::panic::resume_unwind(payload),
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

#[derive(Debug, Clone, Copy)]
struct NoiseTreeContextGeometry {
    min_y: i32,
    height: i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
    cell_width: i32,
    cell_height: i32,
    cell_count_xz: i32,
    cell_noise_min_y: i32,
}

impl NoiseTreeContextGeometry {
    fn new(pos: ChunkPos, settings: &NoiseGeneratorSettings) -> Self {
        let min_y = settings.noise.min_y;
        let cell_height = settings.noise.cell_height();
        Self {
            min_y,
            height: settings.noise.height,
            chunk_min_x: pos.x * 16,
            chunk_min_z: pos.z * 16,
            cell_width: settings.noise.cell_width(),
            cell_height,
            cell_count_xz: 16 / settings.noise.cell_width(),
            cell_noise_min_y: min_y.div_euclid(cell_height),
        }
    }
}

struct NoiseTreeContextScanTops {
    by_column: [i32; 16 * 16],
    max_cell_y: i32,
}

struct NoiseTreeContextHeightmaps {
    ocean_floor: [i32; 16 * 16],
    world_surface: [i32; 16 * 16],
    motion_blocking: [i32; 16 * 16],
    motion_blocking_no_leaves: [i32; 16 * 16],
    found_ocean_floor: [bool; 16 * 16],
    found_world_surface: [bool; 16 * 16],
    remaining_ocean_floor: usize,
    remaining_world_surface: usize,
}

impl NoiseTreeContextHeightmaps {
    fn new(min_y: i32) -> Self {
        Self {
            ocean_floor: [min_y; 16 * 16],
            world_surface: [min_y; 16 * 16],
            motion_blocking: [min_y; 16 * 16],
            motion_blocking_no_leaves: [min_y; 16 * 16],
            found_ocean_floor: [false; 16 * 16],
            found_world_surface: [false; 16 * 16],
            remaining_ocean_floor: 16 * 16,
            remaining_world_surface: 16 * 16,
        }
    }

    fn is_complete_at(&self, index: usize) -> bool {
        self.found_ocean_floor[index] && self.found_world_surface[index]
    }

    fn record_block(&mut self, index: usize, y: i32, block_kind: NoiseHeightmapBlockKind) -> bool {
        if !self.found_world_surface[index] {
            self.world_surface[index] = y + 1;
            self.motion_blocking[index] = y + 1;
            self.motion_blocking_no_leaves[index] = y + 1;
            self.found_world_surface[index] = true;
            self.remaining_world_surface -= 1;
        }
        if block_kind == NoiseHeightmapBlockKind::Solid {
            self.ocean_floor[index] = y + 1;
            self.found_ocean_floor[index] = true;
            self.remaining_ocean_floor -= 1;
        }
        self.remaining_world_surface == 0 && self.remaining_ocean_floor == 0
    }

    fn into_tree_decoration_heights(self) -> TreeDecorationHeights {
        TreeDecorationHeights {
            ocean_floor: self.ocean_floor,
            world_surface: self.world_surface,
            motion_blocking: self.motion_blocking,
            motion_blocking_no_leaves: self.motion_blocking_no_leaves,
        }
    }
}

struct NoiseTreeContextHeightStats {
    density_samples: usize,
    fluid_samples: usize,
    lowest_sampled_y: i32,
    highest_sampled_y: i32,
}

impl Default for NoiseTreeContextHeightStats {
    fn default() -> Self {
        Self {
            density_samples: 0,
            fluid_samples: 0,
            lowest_sampled_y: i32::MAX,
            highest_sampled_y: i32::MIN,
        }
    }
}

fn noise_tree_context_heights_inner(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> TreeDecorationHeights {
    let geometry = NoiseTreeContextGeometry::new(pos, settings);
    let mut noise_chunk = NoiseChunk::new(
        geometry.chunk_min_x,
        geometry.chunk_min_z,
        *settings,
        seed,
        noise_router,
    );
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let mut aquifer = settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            &mut noise_chunk,
            NoiseBasedAquiferBounds {
                chunk_min_x: geometry.chunk_min_x,
                chunk_max_x: geometry.chunk_min_x + 15,
                chunk_min_z: geometry.chunk_min_z,
                chunk_max_z: geometry.chunk_min_z + 15,
                min_block_y: geometry.min_y,
                y_block_size: geometry.height,
            },
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });
    let scan_tops = noise_tree_context_scan_tops(&mut noise_chunk, geometry);
    let mut heightmaps = NoiseTreeContextHeightmaps::new(geometry.min_y);
    let mut stats = NoiseTreeContextHeightStats::default();
    scan_noise_tree_context_heightmaps(NoiseTreeContextHeightmapScan {
        geometry,
        settings,
        noise_chunk: &mut noise_chunk,
        aquifer: aquifer.as_mut(),
        scan_top_by_column: &scan_tops.by_column,
        max_scan_top_cell_y: scan_tops.max_cell_y,
        heightmaps: &mut heightmaps,
        stats: &mut stats,
    });
    print_noise_tree_context_height_debug(pos, &heightmaps, &stats);
    heightmaps.into_tree_decoration_heights()
}

fn noise_tree_context_scan_tops(
    noise_chunk: &mut NoiseChunk,
    geometry: NoiseTreeContextGeometry,
) -> NoiseTreeContextScanTops {
    let mut by_column = [geometry.min_y; 16 * 16];
    let mut max_scan_top_y = geometry.min_y;
    for local_z in 0..16 {
        for local_x in 0..16 {
            let world_x = geometry.chunk_min_x + local_x as i32;
            let world_z = geometry.chunk_min_z + local_z as i32;
            // Java already has neighbor chunks here. This lightweight fallback
            // only needs the worldgen heightmaps, so bound the scan using the
            // same preliminary surface signal Java feeds into aquifers/surface rules.
            let top_y = (noise_chunk.preliminary_surface_level(world_x, world_z) + 64)
                .clamp(geometry.min_y, geometry.min_y + geometry.height - 1);
            by_column[local_z * 16 + local_x] = top_y;
            max_scan_top_y = max_scan_top_y.max(top_y);
        }
    }
    NoiseTreeContextScanTops {
        by_column,
        max_cell_y: (max_scan_top_y - geometry.min_y).div_euclid(geometry.cell_height),
    }
}

struct NoiseTreeContextHeightmapScan<'a> {
    geometry: NoiseTreeContextGeometry,
    settings: &'a NoiseGeneratorSettings,
    noise_chunk: &'a mut NoiseChunk,
    aquifer: Option<&'a mut NoiseBasedAquifer>,
    scan_top_by_column: &'a [i32; 16 * 16],
    max_scan_top_cell_y: i32,
    heightmaps: &'a mut NoiseTreeContextHeightmaps,
    stats: &'a mut NoiseTreeContextHeightStats,
}

fn scan_noise_tree_context_heightmaps(mut input: NoiseTreeContextHeightmapScan<'_>) {
    let geometry = input.geometry;
    'cells: for cell_x_index in 0..geometry.cell_count_xz {
        input.noise_chunk.advance_cell_x(cell_x_index);
        for cell_z_index in 0..geometry.cell_count_xz {
            for cell_y_index in (0..=input.max_scan_top_cell_y).rev() {
                input.noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
                for y_in_cell in (0..geometry.cell_height).rev() {
                    let pos_y = (geometry.cell_noise_min_y + cell_y_index) * geometry.cell_height
                        + y_in_cell;
                    let factor_y = y_in_cell as f64 / geometry.cell_height as f64;
                    input.noise_chunk.update_for_y(pos_y, factor_y);
                    for x_in_cell in 0..geometry.cell_width {
                        let pos_x =
                            geometry.chunk_min_x + cell_x_index * geometry.cell_width + x_in_cell;
                        let local_x = (pos_x & 15) as usize;
                        let factor_x = x_in_cell as f64 / geometry.cell_width as f64;
                        input.noise_chunk.update_for_x(pos_x, factor_x);
                        for z_in_cell in 0..geometry.cell_width {
                            let pos_z = geometry.chunk_min_z
                                + cell_z_index * geometry.cell_width
                                + z_in_cell;
                            let local_z = (pos_z & 15) as usize;
                            let factor_z = z_in_cell as f64 / geometry.cell_width as f64;
                            input.noise_chunk.update_for_z(pos_z, factor_z);
                            let index = local_z * 16 + local_x;
                            if pos_y > input.scan_top_by_column[index] {
                                continue;
                            }
                            if input.heightmaps.is_complete_at(index) {
                                continue;
                            }
                            input.stats.density_samples += 1;
                            input.stats.lowest_sampled_y = input.stats.lowest_sampled_y.min(pos_y);
                            input.stats.highest_sampled_y =
                                input.stats.highest_sampled_y.max(pos_y);
                            let density =
                                input.noise_chunk.interpolated_density(pos_x, pos_y, pos_z);
                            let block_kind = noise_context_heightmap_block_kind(
                                input.aquifer.as_deref_mut(),
                                input.noise_chunk,
                                input.settings,
                                pos_x,
                                pos_y,
                                pos_z,
                                density,
                            );
                            if block_kind == NoiseHeightmapBlockKind::Fluid {
                                input.stats.fluid_samples += 1;
                            }
                            if block_kind == NoiseHeightmapBlockKind::Air {
                                continue;
                            }
                            if input.heightmaps.record_block(index, pos_y, block_kind) {
                                break 'cells;
                            }
                        }
                    }
                }
            }
        }
        input.noise_chunk.swap_slices();
    }
}

fn print_noise_tree_context_height_debug(
    pos: ChunkPos,
    heightmaps: &NoiseTreeContextHeightmaps,
    stats: &NoiseTreeContextHeightStats,
) {
    if std::env::var_os("RUSTCRAFT_WORLDGEN_TREE_HEIGHT_DEBUG").is_some() {
        eprintln!(
            "[tree-height-debug] chunk=({}, {}) density_samples={} fluid_samples={} y_range={}..{} remaining_world_surface={} remaining_ocean_floor={}",
            pos.x,
            pos.z,
            stats.density_samples,
            stats.fluid_samples,
            stats.lowest_sampled_y,
            stats.highest_sampled_y,
            heightmaps.remaining_world_surface,
            heightmaps.remaining_ocean_floor
        );
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
