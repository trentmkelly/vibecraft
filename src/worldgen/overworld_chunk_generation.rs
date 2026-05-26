use super::*;

pub fn generate_overworld_chunk_for_preset(
    pos: ChunkPos,
    preset_id: &str,
) -> Result<LevelChunk, String> {
    generate_overworld_chunk_for_preset_with_mode(
        pos,
        preset_id,
        LiveChunkGenerationMode::Preview,
        0,
    )
}

pub fn generate_overworld_chunk_for_preset_with_mode(
    pos: ChunkPos,
    preset_id: &str,
    mode: LiveChunkGenerationMode,
    seed: i64,
) -> Result<LevelChunk, String> {
    let preset = resolve_world_preset(preset_id)?;
    generate_chunk_for_stem_with_mode(pos, &preset.overworld, mode, seed)
}

pub fn generate_overworld_spawn_chunk_for_preset_with_mode(
    pos: ChunkPos,
    preset_id: &str,
    mode: LiveChunkGenerationMode,
    seed: i64,
    spawn_mobs_game_rule: bool,
) -> Result<LevelChunk, String> {
    let preset = resolve_world_preset(preset_id)?;
    let mut chunk = generate_chunk_for_stem_with_mode(pos, &preset.overworld, mode, seed)?;
    apply_spawn_original_mobs_to_generated_chunk(
        &mut chunk,
        &preset.overworld,
        seed,
        spawn_mobs_game_rule,
    );
    chunk.status = "minecraft:spawn".to_string();
    Ok(chunk)
}

pub fn generate_overworld_spawn_chunk_for_preset_with_mode_timed(
    pos: ChunkPos,
    preset_id: &str,
    mode: LiveChunkGenerationMode,
    seed: i64,
    spawn_mobs_game_rule: bool,
) -> Result<(LevelChunk, LiveChunkGenerationTimings), String> {
    let mut timings = LiveChunkGenerationTimings::default();

    let started = Instant::now();
    let preset = resolve_world_preset(preset_id)?;
    timings.resolve_preset_ms = started.elapsed().as_millis();

    let started = Instant::now();
    let mut chunk = timed_overworld_chunk_for_generator(
        pos,
        &preset.overworld.generator,
        mode,
        seed,
        &mut timings,
    )?;
    timings.terrain_ms = started.elapsed().as_millis();

    let started = Instant::now();
    timings.heightmaps = add_client_heightmaps_from_blocks_timed(&mut chunk);
    timings.heightmaps.total_ms = started.elapsed().as_millis();

    let started = Instant::now();
    timings.mobs = apply_spawn_original_mobs_to_generated_chunk_timed(
        &mut chunk,
        &preset.overworld,
        seed,
        spawn_mobs_game_rule,
    );
    timings.mobs.total_ms = started.elapsed().as_millis();

    chunk.status = "minecraft:spawn".to_string();
    Ok((chunk, timings))
}

fn timed_overworld_chunk_for_generator(
    pos: ChunkPos,
    generator: &ResolvedChunkGenerator,
    mode: LiveChunkGenerationMode,
    seed: i64,
    timings: &mut LiveChunkGenerationTimings,
) -> Result<LevelChunk, String> {
    match generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(materialize_flat_chunk(pos, settings)),
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => match mode {
            LiveChunkGenerationMode::Preview => Ok(materialize_noise_preview_chunk(
                pos,
                biome_source_model,
                noise_settings,
            )),
            LiveChunkGenerationMode::RealSurface => {
                generate_timed_real_surface_chunk(TimedRealSurfaceChunkInput {
                    pos,
                    biome_source_model,
                    noise_settings,
                    seed,
                    timings,
                })
            }
        },
        ResolvedChunkGenerator::Debug { .. } => {
            Err("Debug overworld chunk generation is not implemented".to_string())
        }
    }
}

struct TimedRealSurfaceChunkInput<'a> {
    pos: ChunkPos,
    biome_source_model: &'a BiomeSourceModel,
    noise_settings: &'a NoiseGeneratorSettings,
    seed: i64,
    timings: &'a mut LiveChunkGenerationTimings,
}

fn generate_timed_real_surface_chunk(
    input: TimedRealSurfaceChunkInput<'_>,
) -> Result<LevelChunk, String> {
    let base_started = Instant::now();
    let Some((mut chunk, terrain_timings, mut noise_context)) = generate_real_surface_base_chunk(
        input.pos,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
    ) else {
        return Ok(timed_fallback_noise_chunk(input));
    };
    input.timings.base_generation_ms = base_started.elapsed().as_millis();
    input.timings.terrain = terrain_timings;
    input.timings.region_biome_steps_ms = 0;

    apply_timed_real_surface_base_stages(
        &mut chunk,
        &mut noise_context,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
        input.timings,
    );
    let decoration_context = timed_decoration_context(&chunk, &input)?;
    apply_timed_real_surface_decoration_stages(&mut chunk, decoration_context, input);
    Ok(chunk)
}

fn timed_fallback_noise_chunk(input: TimedRealSurfaceChunkInput<'_>) -> LevelChunk {
    let router_id = noise_router_id_for_settings(*input.noise_settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let (mut chunk, terrain_timings) =
        fill_from_noise_chunk_timed(input.pos, input.noise_settings, input.seed, noise_router);
    input.timings.terrain = terrain_timings;
    chunk.status = "minecraft:surface".to_string();
    chunk
}

fn apply_timed_real_surface_base_stages(
    chunk: &mut LevelChunk,
    noise_context: &mut LiveNoiseGenerationContext,
    biome_source_model: &BiomeSourceModel,
    noise_settings: &NoiseGeneratorSettings,
    seed: i64,
    timings: &mut LiveChunkGenerationTimings,
) {
    let phase_started = Instant::now();
    timings.carver_blocks = apply_configured_carvers_for_biome_source_with_noise_context(
        chunk,
        biome_source_model,
        noise_settings,
        seed,
        noise_context,
    );
    timings.carvers_ms = phase_started.elapsed().as_millis();

    let phase_started = Instant::now();
    timings.underground_structure_blocks =
        apply_mineshaft_underground_structures_to_chunk(chunk, seed);
    timings.underground_structures_ms = phase_started.elapsed().as_millis();
}

struct TimedDecorationContext {
    context_cache: TreeDecorationContextCache,
    source_steps: DecorationBiomeStepsByChunk,
}

fn timed_decoration_context(
    chunk: &LevelChunk,
    input: &TimedRealSurfaceChunkInput<'_>,
) -> Result<TimedDecorationContext, String> {
    let source_radius = tree_decoration_source_radius();
    let router = builtin_noise_router(noise_router_id_for_settings(*input.noise_settings))
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let surface_rule = load_surface_rule(input.noise_settings.id)
        .ok_or_else(|| format!("missing surface rule {}", input.noise_settings.id))?;
    let context_cache = build_tree_decoration_context_cache(
        chunk.pos,
        source_radius,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
        router,
        &surface_rule,
    );
    let source_steps = source_decoration_biome_steps_cache(
        chunk.pos,
        source_radius,
        input.biome_source_model,
        input.noise_settings,
        &ClimateSampler::from_noise_router(&router, input.seed, *input.noise_settings),
    );
    Ok(TimedDecorationContext {
        context_cache,
        source_steps,
    })
}

fn apply_timed_real_surface_decoration_stages(
    chunk: &mut LevelChunk,
    decoration_context: TimedDecorationContext,
    input: TimedRealSurfaceChunkInput<'_>,
) {
    let phase_started = Instant::now();
    input.timings.ore_blocks = apply_underground_ore_decoration_to_chunk_with_context(
        chunk,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
        None,
        Some(&decoration_context.context_cache.cached_region_chunks),
        Some(&decoration_context.source_steps),
    );
    input.timings.ore_decoration_ms = phase_started.elapsed().as_millis();

    let phase_started = Instant::now();
    let tree_result = apply_initial_tree_decoration_to_chunk(
        chunk,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
        None,
        Some(decoration_context.context_cache),
        Some(&decoration_context.source_steps),
    );
    input.timings.tree_blocks = tree_result.placed_blocks;
    input.timings.tree_context_ms = tree_result.context_build_ms;
    input.timings.tree_context_chunks = tree_result.context_chunks;
    input.timings.tree_decoration_ms = phase_started.elapsed().as_millis();
}

pub fn generate_overworld_spawn_chunk_region_for_preset_with_mode(
    center: ChunkPos,
    radius: i32,
    preset_id: &str,
    mode: LiveChunkGenerationMode,
    seed: i64,
    spawn_mobs_game_rule: bool,
) -> Result<BTreeMap<ChunkPos, LevelChunk>, String> {
    let radius = radius.max(0);
    if mode != LiveChunkGenerationMode::RealSurface {
        return generate_fallback_spawn_chunk_region(
            center,
            radius,
            preset_id,
            mode,
            seed,
            spawn_mobs_game_rule,
        );
    }

    let preset = resolve_world_preset(preset_id)?;
    let ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        return generate_fallback_spawn_chunk_region(
            center,
            radius,
            preset_id,
            mode,
            seed,
            spawn_mobs_game_rule,
        );
    };

    generate_real_surface_spawn_chunk_region(RealSurfaceRegionInput {
        center,
        radius,
        stem: &preset.overworld,
        biome_source_model,
        noise_settings,
        seed,
        spawn_mobs_game_rule,
    })
}

fn generate_fallback_spawn_chunk_region(
    center: ChunkPos,
    radius: i32,
    preset_id: &str,
    mode: LiveChunkGenerationMode,
    seed: i64,
    spawn_mobs_game_rule: bool,
) -> Result<BTreeMap<ChunkPos, LevelChunk>, String> {
    let mut chunks = BTreeMap::new();
    for z in center.z - radius..=center.z + radius {
        for x in center.x - radius..=center.x + radius {
            let pos = ChunkPos { x, z };
            chunks.insert(
                pos,
                generate_overworld_spawn_chunk_for_preset_with_mode(
                    pos,
                    preset_id,
                    mode,
                    seed,
                    spawn_mobs_game_rule,
                )?,
            );
        }
    }
    Ok(chunks)
}

struct RealSurfaceRegionInput<'a> {
    center: ChunkPos,
    radius: i32,
    stem: &'a ResolvedLevelStem,
    biome_source_model: &'a BiomeSourceModel,
    noise_settings: &'a NoiseGeneratorSettings,
    seed: i64,
    spawn_mobs_game_rule: bool,
}

fn generate_real_surface_spawn_chunk_region(
    input: RealSurfaceRegionInput<'_>,
) -> Result<BTreeMap<ChunkPos, LevelChunk>, String> {
    let mut chunks = BTreeMap::new();
    let source_positions = populate_real_surface_region_base(&mut chunks, &input);
    let feature_source_positions = region_feature_source_positions(
        &source_positions,
        input.center,
        input.radius.saturating_sub(1),
    );

    apply_real_surface_region_features(&mut chunks, &feature_source_positions, &input);
    finalize_real_surface_spawn_region(&mut chunks, &input);
    Ok(chunks)
}

fn populate_real_surface_region_base(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    input: &RealSurfaceRegionInput<'_>,
) -> Vec<ChunkPos> {
    let mut source_positions = Vec::new();
    for z in input.center.z - input.radius..=input.center.z + input.radius {
        for x in input.center.x - input.radius..=input.center.x + input.radius {
            let pos = ChunkPos { x, z };
            let chunk = real_surface_region_base_chunk(pos, input);
            chunks.insert(pos, chunk);
            source_positions.push(pos);
        }
    }
    source_positions
}

fn real_surface_region_base_chunk(pos: ChunkPos, input: &RealSurfaceRegionInput<'_>) -> LevelChunk {
    let Some((mut chunk, _, mut noise_context)) = generate_real_surface_base_chunk(
        pos,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
    ) else {
        return fallback_real_surface_region_chunk(pos, input);
    };
    apply_configured_carvers_for_biome_source_with_noise_context(
        &mut chunk,
        input.biome_source_model,
        input.noise_settings,
        input.seed,
        &mut noise_context,
    );
    apply_mineshaft_underground_structures_to_chunk(&mut chunk, input.seed);
    chunk
}

fn fallback_real_surface_region_chunk(
    pos: ChunkPos,
    input: &RealSurfaceRegionInput<'_>,
) -> LevelChunk {
    let router_id = noise_router_id_for_settings(*input.noise_settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let (mut chunk, _) =
        fill_from_noise_chunk_timed(pos, input.noise_settings, input.seed, noise_router);
    chunk.status = "minecraft:surface".to_string();
    chunk
}

fn region_feature_source_positions(
    source_positions: &[ChunkPos],
    center: ChunkPos,
    feature_source_radius: i32,
) -> Vec<ChunkPos> {
    source_positions
        .iter()
        .copied()
        .filter(|pos| {
            (pos.x - center.x).abs() <= feature_source_radius
                && (pos.z - center.z).abs() <= feature_source_radius
        })
        .collect()
}

fn apply_real_surface_region_features(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    feature_source_positions: &[ChunkPos],
    input: &RealSurfaceRegionInput<'_>,
) {
    for pos in feature_source_positions.iter().copied() {
        apply_underground_ore_decoration_from_source_into_region(
            chunks,
            pos,
            input.biome_source_model,
            input.noise_settings,
            input.seed,
            None,
        );
    }
    for pos in feature_source_positions.iter().copied() {
        apply_initial_simple_vegetation_decoration_from_source_into_region(
            chunks,
            pos,
            input.biome_source_model,
            input.noise_settings,
            input.seed,
            SimpleVegetationPhase::BeforeTrees,
        );
    }
    for pos in feature_source_positions.iter().copied() {
        apply_initial_tree_decoration_from_source_into_region(
            chunks,
            pos,
            input.biome_source_model,
            input.noise_settings,
            input.seed,
            None,
        );
    }
    for pos in feature_source_positions.iter().copied() {
        apply_initial_simple_vegetation_decoration_from_source_into_region(
            chunks,
            pos,
            input.biome_source_model,
            input.noise_settings,
            input.seed,
            SimpleVegetationPhase::AfterTrees,
        );
    }
}

fn finalize_real_surface_spawn_region(
    chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    input: &RealSurfaceRegionInput<'_>,
) {
    for chunk in chunks.values_mut() {
        add_client_heightmaps_from_blocks(chunk);
        apply_spawn_original_mobs_to_generated_chunk(
            chunk,
            input.stem,
            input.seed,
            input.spawn_mobs_game_rule,
        );
        chunk.status = "minecraft:spawn".to_string();
    }
}
