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
    let mut chunk = match &preset.overworld.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => materialize_flat_chunk(pos, settings),
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => match mode {
            LiveChunkGenerationMode::Preview => {
                materialize_noise_preview_chunk(pos, biome_source_model, noise_settings)
            }
            LiveChunkGenerationMode::RealSurface => {
                let base_started = Instant::now();
                match generate_real_surface_base_chunk(
                    pos,
                    biome_source_model,
                    noise_settings,
                    seed,
                ) {
                    Some((mut chunk, terrain_timings, mut noise_context)) => {
                        timings.base_generation_ms = base_started.elapsed().as_millis();
                        timings.terrain = terrain_timings;
                        timings.region_biome_steps_ms = 0;

                        let phase_started = Instant::now();
                        timings.carver_blocks =
                            apply_configured_carvers_for_biome_source_with_noise_context(
                                &mut chunk,
                                biome_source_model,
                                noise_settings,
                                seed,
                                &mut noise_context,
                            );
                        timings.carvers_ms = phase_started.elapsed().as_millis();

                        let phase_started = Instant::now();
                        timings.underground_structure_blocks =
                            apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
                        timings.underground_structures_ms = phase_started.elapsed().as_millis();

                        let source_radius = tree_decoration_source_radius();
                        let decoration_context_router =
                            builtin_noise_router(noise_router_id_for_settings(**noise_settings))
                                .map(|entry| entry.router)
                                .unwrap_or(NONE_NOISE_ROUTER);
                        let surface_rule = load_surface_rule(noise_settings.id)
                            .ok_or_else(|| format!("missing surface rule {}", noise_settings.id))?;
                        let decoration_context_cache = build_tree_decoration_context_cache(
                            chunk.pos,
                            source_radius,
                            biome_source_model,
                            noise_settings,
                            seed,
                            decoration_context_router,
                            &surface_rule,
                        );
                        let source_decoration_steps = source_decoration_biome_steps_cache(
                            chunk.pos,
                            source_radius,
                            biome_source_model,
                            noise_settings,
                            &ClimateSampler::from_noise_router(
                                &decoration_context_router,
                                seed,
                                **noise_settings,
                            ),
                        );

                        let phase_started = Instant::now();
                        timings.ore_blocks = apply_underground_ore_decoration_to_chunk_with_context(
                            &mut chunk,
                            biome_source_model,
                            noise_settings,
                            seed,
                            None,
                            Some(&decoration_context_cache.cached_region_chunks),
                            Some(&source_decoration_steps),
                        );
                        timings.ore_decoration_ms = phase_started.elapsed().as_millis();

                        let phase_started = Instant::now();
                        let tree_result = apply_initial_tree_decoration_to_chunk(
                            &mut chunk,
                            biome_source_model,
                            noise_settings,
                            seed,
                            None,
                            Some(decoration_context_cache),
                            Some(&source_decoration_steps),
                        );
                        timings.tree_blocks = tree_result.placed_blocks;
                        timings.tree_context_ms = tree_result.context_build_ms;
                        timings.tree_context_chunks = tree_result.context_chunks;
                        timings.tree_decoration_ms = phase_started.elapsed().as_millis();
                        chunk
                    }
                    None => {
                        let router_id = noise_router_id_for_settings(**noise_settings);
                        let noise_router = builtin_noise_router(router_id)
                            .map(|e| e.router)
                            .unwrap_or(NONE_NOISE_ROUTER);
                        let (mut chunk, terrain_timings) =
                            fill_from_noise_chunk_timed(pos, noise_settings, seed, noise_router);
                        timings.terrain = terrain_timings;
                        chunk.status = "minecraft:surface".to_string();
                        chunk
                    }
                }
            }
        },
        ResolvedChunkGenerator::Debug { .. } => {
            return Err("Debug overworld chunk generation is not implemented".to_string())
        }
    };
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
        return Ok(chunks);
    }

    let preset = resolve_world_preset(preset_id)?;
    let ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
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
        return Ok(chunks);
    };

    let mut chunks = BTreeMap::new();
    let mut source_positions = Vec::new();
    for z in center.z - radius..=center.z + radius {
        for x in center.x - radius..=center.x + radius {
            let pos = ChunkPos { x, z };
            let Some((mut chunk, _, mut noise_context)) =
                generate_real_surface_base_chunk(pos, biome_source_model, noise_settings, seed)
            else {
                let router_id = noise_router_id_for_settings(**noise_settings);
                let noise_router = builtin_noise_router(router_id)
                    .map(|e| e.router)
                    .unwrap_or(NONE_NOISE_ROUTER);
                let (mut chunk, _) =
                    fill_from_noise_chunk_timed(pos, noise_settings, seed, noise_router);
                chunk.status = "minecraft:surface".to_string();
                chunks.insert(pos, chunk);
                source_positions.push(pos);
                continue;
            };
            apply_configured_carvers_for_biome_source_with_noise_context(
                &mut chunk,
                biome_source_model,
                noise_settings,
                seed,
                &mut noise_context,
            );
            apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
            chunks.insert(pos, chunk);
            source_positions.push(pos);
        }
    }

    let feature_source_radius = radius.saturating_sub(1);
    let feature_source_positions = source_positions
        .iter()
        .copied()
        .filter(|pos| {
            (pos.x - center.x).abs() <= feature_source_radius
                && (pos.z - center.z).abs() <= feature_source_radius
        })
        .collect::<Vec<_>>();

    for pos in feature_source_positions.iter().copied() {
        apply_underground_ore_decoration_from_source_into_region(
            &mut chunks,
            pos,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );
    }

    for pos in feature_source_positions.iter().copied() {
        apply_initial_simple_vegetation_decoration_from_source_into_region(
            &mut chunks,
            pos,
            biome_source_model,
            noise_settings,
            seed,
            SimpleVegetationPhase::BeforeTrees,
        );
    }

    for pos in feature_source_positions.iter().copied() {
        apply_initial_tree_decoration_from_source_into_region(
            &mut chunks,
            pos,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );
    }

    for pos in feature_source_positions {
        apply_initial_simple_vegetation_decoration_from_source_into_region(
            &mut chunks,
            pos,
            biome_source_model,
            noise_settings,
            seed,
            SimpleVegetationPhase::AfterTrees,
        );
    }

    for chunk in chunks.values_mut() {
        add_client_heightmaps_from_blocks(chunk);
        apply_spawn_original_mobs_to_generated_chunk(
            chunk,
            &preset.overworld,
            seed,
            spawn_mobs_game_rule,
        );
        chunk.status = "minecraft:spawn".to_string();
    }

    Ok(chunks)
}
