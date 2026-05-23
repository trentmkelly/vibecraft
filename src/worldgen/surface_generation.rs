use super::*;

pub(super) fn read_world_surface_wg(
    chunk: &crate::storage::chunk::LevelChunk,
    local_x: usize,
    local_z: usize,
) -> i32 {
    let hm = match chunk.heightmaps.get("WORLD_SURFACE_WG") {
        Some(crate::storage::nbt::Tag::LongArray(longs)) => longs,
        _ => return 0,
    };
    const BITS: usize = 9;
    const MASK: u64 = (1u64 << BITS) - 1;
    let index = local_z * 16 + local_x;
    let bit_offset = index * BITS;
    let word_index = bit_offset / 64;
    let bit_index = bit_offset % 64;
    if word_index >= hm.len() {
        return 0;
    }
    let word = hm[word_index] as u64;
    let mut value = (word >> bit_index) & MASK;
    let spill = bit_index + BITS;
    if spill > 64 && word_index + 1 < hm.len() {
        let next = hm[word_index + 1] as u64;
        value |= (next << (64 - bit_index)) & MASK;
    }
    value as i32
}

/// Returns `true` when `block` is neither air nor a fluid.
///
/// Mirrors Java `SurfaceSystem.isStone`: `!state.isAir() && state.getFluidState().isEmpty()`.
pub(super) fn is_surface_stone(block: &str) -> bool {
    !is_surface_air(block) && !is_surface_fluid(block)
}

pub(super) fn is_surface_air(block: &str) -> bool {
    matches!(
        block,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

pub(super) fn is_surface_fluid(block: &str) -> bool {
    matches!(block, "minecraft:water" | "minecraft:lava")
}

pub(super) fn surface_biome_temperature(biome: &str) -> f32 {
    match biome.strip_prefix("minecraft:").unwrap_or(biome) {
        "frozen_ocean" | "deep_frozen_ocean" | "frozen_river" | "snowy_plains" | "ice_spikes"
        | "snowy_beach" | "snowy_taiga" | "grove" | "snowy_slopes" | "frozen_peaks"
        | "jagged_peaks" => 0.0,
        "desert" | "badlands" | "eroded_badlands" | "wooded_badlands" | "savanna"
        | "savanna_plateau" | "windswept_savanna" => 2.0,
        _ => 0.8,
    }
}

/// Evaluate the clay band at (worldX, y, worldZ) for a given world seed.
///
/// Generates the clay band array from `"minecraft:clay_bands"` positional random
/// and applies the `minecraft:clay_bands_offset` noise offset — mirroring
/// `SurfaceSystem.generateBands` + `SurfaceSystem.getBand`.
pub(super) fn get_clay_band(
    seed: i64,
    algorithm: RandomAlgorithm,
    settings: NoiseGeneratorSettings,
    world_x: i32,
    y: i32,
    world_z: i32,
) -> &'static str {
    use crate::random_source::{random_state_named_factory, random_state_seed_factories};

    let base = random_state_seed_factories(seed, algorithm).base;
    // Java: generateBands(noiseRandom.fromHashOf(Identifier.withDefaultNamespace("clay_bands")))
    let factory = random_state_named_factory(base, "minecraft:clay_bands");
    let mut rng = factory.at(0, 0, 0);

    const LEN: usize = 192;
    // 0=TERRACOTTA, 1=ORANGE, 2=YELLOW, 3=BROWN, 4=RED, 5=WHITE, 6=LIGHT_GRAY
    let mut bands = [0u8; LEN];

    // Orange stripes (mirroring the first loop in generateBands)
    let mut i = 0usize;
    while i < LEN {
        i += (random_next_i32_bound(&mut rng, 5) + 1) as usize;
        if i < LEN {
            bands[i] = 1;
        }
    }

    // makeBands helper: YELLOW (baseWidth=1), BROWN (baseWidth=2), RED (baseWidth=1)
    for (base_width, kind) in [(1u32, 2u8), (2, 3), (1, 4)] {
        let count = random_next_i32_bound(&mut rng, 10) + 6;
        for _ in 0..count {
            let width = base_width as usize + random_next_i32_bound(&mut rng, 3) as usize;
            let start = random_next_i32_bound(&mut rng, LEN as i32) as usize;
            for p in 0..width {
                if start + p < LEN {
                    bands[start + p] = kind;
                }
            }
        }
    }

    // White + light-gray stripes
    // Java: nextIntBetweenInclusive(9, 15) = nextInt(7) + 9
    let white_count = random_next_i32_bound(&mut rng, 7) + 9;
    let mut w_start = 0usize;
    for _ in 0..white_count {
        // Java: nextInt(16) + 4
        w_start += (random_next_i32_bound(&mut rng, 16) + 4) as usize;
        if w_start >= LEN {
            break;
        }
        bands[w_start] = 5;
        // Java: nextBoolean() = nextInt(2) != 0
        if w_start > 0 && random_next_i32_bound(&mut rng, 2) != 0 {
            bands[w_start - 1] = 6;
        }
        if w_start + 1 < LEN && random_next_i32_bound(&mut rng, 2) != 0 {
            bands[w_start + 1] = 6;
        }
    }

    // getBand: apply clay_bands_offset noise
    let offset = random_state_normal_noise_snapshot(seed, settings, "minecraft:clay_bands_offset")
        .map(|snap| {
            (normal_noise_sample(&snap, world_x as f64, 0.0, world_z as f64) * 4.0).round() as i32
        })
        .unwrap_or(0);

    let idx = ((y + offset + LEN as i32).rem_euclid(LEN as i32)) as usize;
    match bands[idx] {
        0 => "minecraft:terracotta",
        1 => "minecraft:orange_terracotta",
        2 => "minecraft:yellow_terracotta",
        3 => "minecraft:brown_terracotta",
        4 => "minecraft:red_terracotta",
        5 => "minecraft:white_terracotta",
        6 => "minecraft:light_gray_terracotta",
        _ => "minecraft:terracotta",
    }
}

/// Apply surface rules to a noise-filled chunk.
///
/// Converts plain stone/water terrain into grass, dirt, sand, deepslate,
/// bedrock, etc. according to the surface rule tree — mirroring Java's
/// `SurfaceSystem.buildSurface`.
///
/// Must be called inside `with_noise_snapshot_cache`.
pub(super) fn build_surface_for_chunk(
    chunk: &mut crate::storage::chunk::LevelChunk,
    rule: &DynSurfaceRule,
    biome_source_model: &BiomeSourceModel,
    noise_router: NoiseRouter,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) {
    let mut timings = LiveTerrainTimings::default();
    build_surface_for_chunk_timed(
        chunk,
        rule,
        biome_source_model,
        noise_router,
        settings,
        seed,
        &mut timings,
    );
}

pub(super) fn build_surface_for_chunk_timed(
    chunk: &mut crate::storage::chunk::LevelChunk,
    rule: &DynSurfaceRule,
    biome_source_model: &BiomeSourceModel,
    noise_router: NoiseRouter,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    timings: &mut LiveTerrainTimings,
) {
    build_surface_for_chunk_timed_with_sections(
        chunk,
        None,
        rule,
        biome_source_model,
        noise_router,
        settings,
        seed,
        timings,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_surface_for_chunk_timed_with_sections(
    chunk: &mut crate::storage::chunk::LevelChunk,
    predecoded_section_blocks: Option<&mut GeneratedSectionBlocks>,
    rule: &DynSurfaceRule,
    biome_source_model: &BiomeSourceModel,
    noise_router: NoiseRouter,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    timings: &mut LiveTerrainTimings,
) {
    let total_started = Instant::now();
    let min_y = settings.noise.min_y;
    let heights = WorldGenerationHeightContext {
        min_y,
        height: settings.noise.height,
    };
    let algorithm = if settings.legacy_random_source {
        RandomAlgorithm::Legacy
    } else {
        RandomAlgorithm::Xoroshiro
    };
    let default_block = settings.default_block;

    // Java uses DimensionType.WAY_BELOW_MIN_Y = Integer.MIN_VALUE / 2 as the
    // "no ceiling stone found" sentinel.
    const WAY_BELOW_MIN_Y: i32 = i32::MIN / 2;

    let started = Instant::now();
    // Pre-sample surface/secondary noises (thread-local cache keeps this cheap).
    let surface_noise = random_state_normal_noise_snapshot(seed, *settings, "minecraft:surface");
    let surface_secondary_noise =
        random_state_normal_noise_snapshot(seed, *settings, "minecraft:surface_secondary");
    timings.surface_noise_setup_ms = started.elapsed().as_millis();

    let chunk_min_x = chunk.pos.x * 16;
    let chunk_min_z = chunk.pos.z * 16;
    let mut owned_section_blocks;
    let section_blocks: &mut GeneratedSectionBlocks = if let Some(section_blocks) =
        predecoded_section_blocks
    {
        section_blocks
    } else {
        owned_section_blocks = chunk.sections.iter().fold(
            GeneratedSectionBlocks::new(chunk.min_section_y, chunk.sections.len() as i32),
            |mut generated, section| {
                if let Ok(container) =
                    PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
                {
                    let section_index = i32::from(section.y) - generated.min_section_y;
                    if section_index >= 0 && (section_index as usize) < generated.sections.len() {
                        for index in 0..SECTION_VOLUME {
                            let Some(name) = container
                                .get_entry(index)
                                .and_then(block_name_from_tag_fast)
                            else {
                                continue;
                            };
                            if name == "minecraft:air" {
                                continue;
                            }
                            let id = generated.id_for(name);
                            let section_cache = &mut generated.sections[section_index as usize];
                            section_cache.ids[index] = id;
                            section_cache.non_air_blocks += 1;
                        }
                    }
                }
                generated
            },
        );
        &mut owned_section_blocks
    };

    let started = Instant::now();
    // Pre-compute the 4 preliminary-surface-level corner values used by all
    // columns in this chunk for the minSurfaceLevel bilinear interpolation.
    // Java: cornerCellX = blockX >> 4; surfaceCellToBlockCoord(cornerCellX) = cornerCellX << 4
    // ⟹ for any blockX in the chunk, cornerCellX * 16 == chunk_min_x.
    let prelim_fn = noise_router.preliminary_surface_level;
    let prelim_q = |bx: i32, bz: i32| {
        let qx = (bx >> 2) << 2; // QuartPos round-down
        let qz = (bz >> 2) << 2;
        prelim_fn
            .compute_with_noise(seed, *settings, qx, 0, qz)
            .floor() as i32
    };
    let prelim00 = prelim_q(chunk_min_x, chunk_min_z);
    let prelim10 = prelim_q(chunk_min_x + 16, chunk_min_z);
    let prelim01 = prelim_q(chunk_min_x, chunk_min_z + 16);
    let prelim11 = prelim_q(chunk_min_x + 16, chunk_min_z + 16);
    timings.surface_prelim_ms = started.elapsed().as_millis();

    let base_rng = random_state_seed_factories(seed, algorithm).base;
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let biome_zoom_seed = biome_manager_obfuscate_seed(seed);
    let mut surface_context = SurfaceRulesContext::new(seed, algorithm, heights);
    let chunk_noise_biomes = ChunkNoiseBiomeCache::from_chunk(chunk);
    let mut surface_biome_cache: HashMap<(i32, i32, i32), (&'static str, f32)> = HashMap::new();
    let mut surface_noise_biome_cache: HashMap<(i32, i32, i32), &'static str> = HashMap::new();
    let default_block_id = section_blocks.id_for(default_block);
    let air_id = section_blocks.id_for("minecraft:air");
    let water_id = section_blocks.id_for("minecraft:water");
    let lava_id = section_blocks.id_for("minecraft:lava");
    let surface_debug = std::env::var_os("RUSTCRAFT_WORLDGEN_SURFACE_DEBUG").is_some();
    let mut surface_noise_sample_us = 0_u128;
    let mut surface_rng_us = 0_u128;
    let mut surface_height_read_us = 0_u128;
    let mut surface_biome_us = 0_u128;
    let mut surface_block_read_us = 0_u128;
    let mut surface_ceiling_scan_us = 0_u128;
    let mut surface_rule_us = 0_u128;
    let mut surface_write_us = 0_u128;
    let started = Instant::now();
    for local_z in 0..16_i32 {
        for local_x in 0..16_i32 {
            timings.surface_columns += 1;
            let block_x = chunk_min_x + local_x;
            let block_z = chunk_min_z + local_z;
            let lx = local_x as usize;
            let lz = local_z as usize;

            // getSurfaceDepth: (int)(surfaceNoise * 2.75 + 3.0 + random * 0.25)
            let debug_started = surface_debug.then(Instant::now);
            let surface_noise_val = surface_noise
                .as_ref()
                .map(|snap| normal_noise_sample(snap, block_x as f64, 0.0, block_z as f64))
                .unwrap_or(0.0);
            if let Some(started) = debug_started {
                surface_noise_sample_us += started.elapsed().as_micros();
            }
            let surface_depth = {
                let debug_started = surface_debug.then(Instant::now);
                let mut at_rng = base_rng.at(block_x, 0, block_z);
                let jitter = random_next_f64(&mut at_rng) * 0.25;
                if let Some(started) = debug_started {
                    surface_rng_us += started.elapsed().as_micros();
                }
                (surface_noise_val * 2.75 + 3.0 + jitter) as i32
            };

            // getSurfaceSecondary: surfaceSecondaryNoise.getValue(blockX, 0, blockZ)
            let debug_started = surface_debug.then(Instant::now);
            let surface_secondary = surface_secondary_noise
                .as_ref()
                .map(|snap| normal_noise_sample(snap, block_x as f64, 0.0, block_z as f64))
                .unwrap_or(0.0);
            if let Some(started) = debug_started {
                surface_noise_sample_us += started.elapsed().as_micros();
            }

            // Steep: height-diff ≥ 4 between neighbouring columns.
            let debug_started = surface_debug.then(Instant::now);
            let h_n = read_world_surface_wg(chunk, lx, lz.saturating_sub(1));
            let h_s = read_world_surface_wg(chunk, lx, (lz + 1).min(15));
            let h_w = read_world_surface_wg(chunk, lx.saturating_sub(1), lz);
            let h_e = read_world_surface_wg(chunk, (lx + 1).min(15), lz);
            if let Some(started) = debug_started {
                surface_height_read_us += started.elapsed().as_micros();
            }
            let steep = h_s >= h_n + 4 || h_w >= h_e + 4;
            let hole = surface_depth <= 0;

            // minSurfaceLevel: bilinear interpolation of the 4 corner preliminary
            // levels + surfaceDepth - HOW_FAR_BELOW_PRELIMINARY_SURFACE_LEVEL_TO_BUILD_SURFACE
            let tx = local_x as f64 / 16.0;
            let tz = local_z as f64 / 16.0;
            let prelim = lerp(
                tz,
                lerp(tx, prelim00 as f64, prelim10 as f64),
                lerp(tx, prelim01 as f64, prelim11 as f64),
            )
            .floor() as i32;
            // Java: HOW_FAR_BELOW = 8
            let min_surface_level = prelim + surface_depth - 8;

            surface_context.update_xz(
                block_x,
                block_z,
                surface_depth,
                surface_secondary,
                steep,
                hole,
                min_surface_level,
            );

            let mut stone_depth_above: i32 = 0;
            let mut water_height: i32 = i32::MIN;
            let mut next_ceiling_stone_y: i32 = i32::MAX;
            let end_y = min_y;
            let start_height = read_world_surface_wg(chunk, lx, lz) + 1;

            for y in (end_y..=start_height).rev() {
                timings.surface_block_samples += 1;
                let debug_started = surface_debug.then(Instant::now);
                let block_id = section_blocks.get_id(block_x, y, block_z);
                if let Some(started) = debug_started {
                    surface_block_read_us += started.elapsed().as_micros();
                }

                if block_id == air_id {
                    stone_depth_above = 0;
                    water_height = i32::MIN;
                } else if block_id == water_id || block_id == lava_id {
                    if water_height == i32::MIN {
                        water_height = y + 1;
                    }
                } else {
                    // Solid block.
                    if next_ceiling_stone_y >= y {
                        next_ceiling_stone_y = WAY_BELOW_MIN_Y;
                        let mut la = y - 1;
                        let debug_started = surface_debug.then(Instant::now);
                        while la >= end_y - 1 {
                            let la_block_id = section_blocks.get_id(block_x, la, block_z);
                            if la_block_id == air_id
                                || la_block_id == water_id
                                || la_block_id == lava_id
                            {
                                next_ceiling_stone_y = la + 1;
                                break;
                            }
                            la -= 1;
                        }
                        if let Some(started) = debug_started {
                            surface_ceiling_scan_us += started.elapsed().as_micros();
                        }
                    }

                    stone_depth_above += 1;
                    let stone_depth_below = y - next_ceiling_stone_y + 1;

                    if block_id == default_block_id {
                        // Overworld surface rules are top-level:
                        // bedrock floor, above_preliminary_surface, then the
                        // low-Y deepslate gradient. Between the preliminary
                        // surface gate and y=8, Java can only fall through, so
                        // avoid resolving the expensive biome supplier there.
                        if y < min_surface_level && y >= 8 {
                            continue;
                        }
                        surface_context.update_y(
                            stone_depth_above,
                            stone_depth_below,
                            water_height,
                            y,
                        );
                        let band_fn = |wx: i32, by: i32, wz: i32| {
                            get_clay_band(seed, algorithm, *settings, wx, by, wz)
                        };
                        let mut biome_resolver = |block_x: i32, block_y: i32, block_z: i32| {
                            let biome_y = if settings.legacy_random_source {
                                0
                            } else {
                                block_y
                            };
                            let biome_key = (block_x, biome_y, block_z);
                            if let Some(cached) = surface_biome_cache.get(&biome_key).copied() {
                                return cached;
                            }
                            let debug_started = surface_debug.then(Instant::now);
                            let biome = biome_manager_get_biome_cached(
                                biome_source_model,
                                biome_zoom_seed,
                                block_x,
                                biome_y,
                                block_z,
                                &climate_sampler,
                                Some(&chunk_noise_biomes),
                                &mut surface_noise_biome_cache,
                            )
                            .unwrap_or("minecraft:plains");
                            let temperature = surface_biome_temperature(biome);
                            if let Some(started) = debug_started {
                                surface_biome_us += started.elapsed().as_micros();
                            }
                            surface_biome_cache.insert(biome_key, (biome, temperature));
                            (biome, temperature)
                        };
                        let debug_started = surface_debug.then(Instant::now);
                        if let Some(new_block) = dyn_surface_rule_apply(
                            rule,
                            &mut surface_context,
                            *settings,
                            &band_fn,
                            &mut biome_resolver,
                        ) {
                            if let Some(started) = debug_started {
                                surface_rule_us += started.elapsed().as_micros();
                            }
                            if new_block == default_block {
                                continue;
                            }
                            let debug_started = surface_debug.then(Instant::now);
                            section_blocks.set_name(block_x, y, block_z, &new_block);
                            if let Some(started) = debug_started {
                                surface_write_us += started.elapsed().as_micros();
                            }
                            timings.surface_block_writes += 1;
                        } else if let Some(started) = debug_started {
                            surface_rule_us += started.elapsed().as_micros();
                        }
                    }
                }
            }
        }
    }
    let flush_started = surface_debug.then(Instant::now);
    add_client_heightmaps_from_generated_sections(chunk, section_blocks);
    flush_generated_section_blocks(chunk, section_blocks);
    let surface_flush_us = flush_started
        .map(|started| started.elapsed().as_micros())
        .unwrap_or(0);
    timings.surface_column_loop_ms = started.elapsed().as_millis();
    timings.surface_total_ms = total_started.elapsed().as_millis();
    if let Some(profile) = &surface_context.profile {
        let profile = *profile.borrow();
        eprintln!(
            "[surface-rule-debug] total={}ms loop={}ms columns={} samples={} writes={} noise_sample={}us rng={}us height_reads={}us biome={}us block_reads={}us ceiling_scan={}us rule_apply={}us writes={}us flush={}us rule_visits={} sequence={} condition_rules={} block_rules={} bandlands={} condition_tests={} cache_hits={} computes={} compute={}us",
            timings.surface_total_ms,
            timings.surface_column_loop_ms,
            timings.surface_columns,
            timings.surface_block_samples,
            timings.surface_block_writes,
            surface_noise_sample_us,
            surface_rng_us,
            surface_height_read_us,
            surface_biome_us,
            surface_block_read_us,
            surface_ceiling_scan_us,
            surface_rule_us,
            surface_write_us,
            surface_flush_us,
            profile.rule_visits,
            profile.sequence_rule_visits,
            profile.condition_rule_visits,
            profile.block_rule_visits,
            profile.bandlands_rule_visits,
            profile.condition_tests,
            profile.condition_cache_hits,
            profile.condition_computes,
            profile.condition_compute_us,
        );
        let print_kind = |name: &str, kind: SurfaceConditionKindProfile| {
            if kind.tests != 0 || kind.computes != 0 {
                eprintln!(
                    "[surface-rule-debug] condition={} tests={} cache_hits={} computes={} compute={}us",
                    name, kind.tests, kind.cache_hits, kind.computes, kind.compute_us
                );
            }
        };
        print_kind("biome", profile.biome);
        print_kind("noise_threshold", profile.noise_threshold);
        print_kind("vertical_gradient", profile.vertical_gradient);
        print_kind("y_above", profile.y_above);
        print_kind("water", profile.water);
        print_kind("stone_depth", profile.stone_depth);
        print_kind("not", profile.not);
        print_kind("steep", profile.steep);
        print_kind("hole", profile.hole);
        print_kind(
            "above_preliminary_surface",
            profile.above_preliminary_surface,
        );
        print_kind("temperature", profile.temperature);
    }
}

/// Fill terrain from the noise density function AND apply surface rules.
///
/// Returns a chunk with status `"minecraft:surface"`, mirroring the Java
/// pipeline of `NoiseBasedChunkGenerator.doFill` followed by
/// `NoiseBasedChunkGenerator.buildSurface`.
///
/// Both phases share the same noise-snapshot cache for performance.
pub fn fill_noise_and_build_surface(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    surface_rule: &DynSurfaceRule,
) -> crate::storage::chunk::LevelChunk {
    fill_noise_and_build_surface_timed(
        pos,
        biome_source_model,
        settings,
        seed,
        noise_router,
        surface_rule,
    )
    .0
}

pub fn fill_noise_and_build_surface_timed(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    surface_rule: &DynSurfaceRule,
) -> (crate::storage::chunk::LevelChunk, LiveTerrainTimings) {
    let (chunk, timings, _) = fill_noise_and_build_surface_timed_with_context(
        pos,
        biome_source_model,
        settings,
        seed,
        noise_router,
        surface_rule,
    );
    (chunk, timings)
}

pub(super) fn fill_noise_and_build_surface_timed_with_context(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    surface_rule: &DynSurfaceRule,
) -> (
    crate::storage::chunk::LevelChunk,
    LiveTerrainTimings,
    LiveNoiseGenerationContext,
) {
    with_noise_snapshot_cache(|| {
        let (mut chunk, mut timings, mut section_blocks, noise_context) =
            fill_from_noise_chunk_inner_sections_timed(pos, settings, seed, noise_router);
        let started = Instant::now();
        populate_noise_chunk_biomes(&mut chunk, biome_source_model, settings, seed, noise_router);
        timings.biome_storage_ms = started.elapsed().as_millis();
        build_surface_for_chunk_timed_with_sections(
            &mut chunk,
            Some(&mut section_blocks),
            surface_rule,
            biome_source_model,
            noise_router,
            settings,
            seed,
            &mut timings,
        );
        chunk.status = "minecraft:surface".to_string();
        (chunk, timings, noise_context)
    })
}

pub(super) fn generate_real_surface_base_chunk(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    noise_settings: &NoiseGeneratorSettings,
    seed: i64,
) -> Option<(LevelChunk, LiveTerrainTimings, LiveNoiseGenerationContext)> {
    let router_id = noise_router_id_for_settings(*noise_settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|e| e.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let rule = load_surface_rule(noise_settings.id)?;
    Some(fill_noise_and_build_surface_timed_with_context(
        pos,
        biome_source_model,
        noise_settings,
        seed,
        noise_router,
        &rule,
    ))
}

pub(super) fn fill_noise_and_build_surface_without_biome_storage_timed(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    surface_rule: &DynSurfaceRule,
) -> (crate::storage::chunk::LevelChunk, LiveTerrainTimings) {
    with_noise_snapshot_cache(|| {
        let (mut chunk, mut timings) =
            fill_from_noise_chunk_inner_timed(pos, settings, seed, noise_router);
        timings.biome_storage_ms = 0;
        build_surface_for_chunk_timed(
            &mut chunk,
            surface_rule,
            biome_source_model,
            noise_router,
            settings,
            seed,
            &mut timings,
        );
        chunk.status = "minecraft:surface".to_string();
        (chunk, timings)
    })
}

pub(super) fn populate_noise_chunk_biomes(
    chunk: &mut crate::storage::chunk::LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) {
    let debug_enabled = std::env::var_os("RUSTCRAFT_WORLDGEN_BIOME_DEBUG").is_some();
    let total_started = debug_enabled.then(Instant::now);
    let climate_started = debug_enabled.then(Instant::now);
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let climate_ms = climate_started.map(|started| started.elapsed().as_millis());
    let chunk_quart_x = chunk.pos.x * 4;
    let chunk_quart_z = chunk.pos.z * 4;
    let cache_started = debug_enabled.then(Instant::now);
    let overworld_2d_climate = overworld_biome_2d_climate_cache(
        biome_source_model,
        settings,
        &climate_sampler,
        chunk_quart_x,
        chunk_quart_z,
    );
    let cache_ms = cache_started.map(|started| started.elapsed().as_millis());
    let mut biome_tags: HashMap<&'static str, Tag> = HashMap::new();
    let overworld_column_biomes = overworld_2d_climate.as_ref().map(|cache| {
        let mut biomes = ["minecraft:plains"; 16];
        for local_z in 0..4_usize {
            for local_x in 0..4_usize {
                let cached = cache[local_z * 4 + local_x];
                let climate = climate_target(
                    cached.temperature,
                    cached.humidity,
                    cached.continentalness,
                    cached.erosion,
                    0.0,
                    cached.weirdness,
                );
                biomes[local_z * 4 + local_x] =
                    select_climate_biome(overworld_biome_parameters(), climate)
                        .unwrap_or("minecraft:plains");
            }
        }
        biomes
    });

    let fill_started = debug_enabled.then(Instant::now);
    let mut selections = 0_usize;
    for section in &mut chunk.sections {
        let section_quart_y = i32::from(section.y) * 4;
        let mut palette_names: Vec<&'static str> = Vec::with_capacity(2);
        let mut indices = vec![0_u64; BIOME_SECTION_VOLUME];

        for local_y in 0..4_usize {
            for local_z in 0..4_usize {
                for local_x in 0..4_usize {
                    let quart_x = chunk_quart_x + local_x as i32;
                    let quart_y = section_quart_y + local_y as i32;
                    let quart_z = chunk_quart_z + local_z as i32;
                    let biome = if let Some(column_biomes) = &overworld_column_biomes {
                        column_biomes[local_z * 4 + local_x]
                    } else {
                        overworld_2d_climate
                            .as_ref()
                            .and_then(|cache| {
                                let cached = cache[local_z * 4 + local_x];
                                let block_y = quart_y * 4;
                                let depth = cached.depth_offset
                                    + OVERWORLD_DEPTH_GRADIENT_DENSITY.compute(block_y);
                                let climate = climate_target(
                                    cached.temperature,
                                    cached.humidity,
                                    cached.continentalness,
                                    cached.erosion,
                                    depth as f32,
                                    cached.weirdness,
                                );
                                select_biome_from_source(
                                    biome_source_model,
                                    quart_x,
                                    quart_y,
                                    quart_z,
                                    climate,
                                    0.0,
                                )
                            })
                            .or_else(|| {
                                get_biome(
                                    biome_source_model,
                                    quart_x,
                                    quart_y,
                                    quart_z,
                                    &climate_sampler,
                                )
                            })
                            .unwrap_or("minecraft:plains")
                    };
                    selections += 1;
                    let index = local_y * 16 + local_z * 4 + local_x;
                    let palette_index = match palette_names
                        .iter()
                        .position(|candidate| *candidate == biome)
                    {
                        Some(index) => index,
                        None => {
                            palette_names.push(biome);
                            palette_names.len() - 1
                        }
                    };
                    indices[index] = palette_index as u64;
                }
            }
        }

        let palette = palette_names
            .iter()
            .map(|biome| {
                biome_tags
                    .entry(*biome)
                    .or_insert_with(|| Tag::String((*biome).to_string()))
                    .clone()
            })
            .collect::<Vec<_>>();
        section.biomes = if palette.len() == 1 {
            PalettedContainer::single(palette[0].clone(), BIOME_SECTION_VOLUME).to_nbt()
        } else {
            PalettedContainer {
                data: Some(pack_palette_indices(
                    &indices,
                    palette_bits_for_size(palette.len()),
                )),
                palette,
                expected_entries: BIOME_SECTION_VOLUME,
            }
            .to_nbt()
        };
    }
    if debug_enabled {
        eprintln!(
            "[biome-storage-debug] total={}ms climate={}ms cache={}ms fill={}ms sections={} selections={} tags={}",
            total_started
                .map(|started| started.elapsed().as_millis())
                .unwrap_or(0),
            climate_ms.unwrap_or(0),
            cache_ms.unwrap_or(0),
            fill_started
                .map(|started| started.elapsed().as_millis())
                .unwrap_or(0),
            chunk.sections.len(),
            selections,
            biome_tags.len()
        );
    }
}

#[derive(Clone, Copy)]
struct OverworldBiome2dClimate {
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth_offset: f64,
    weirdness: f32,
}

fn overworld_biome_2d_climate_cache(
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    chunk_quart_x: i32,
    chunk_quart_z: i32,
) -> Option<[OverworldBiome2dClimate; 16]> {
    if !matches!(
        biome_source_model,
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld"
        }
    ) || !matches!(
        settings.id,
        "minecraft:overworld" | "minecraft:large_biomes" | "minecraft:amplified"
    ) {
        return None;
    }

    let mut cache = [OverworldBiome2dClimate {
        temperature: 0.0,
        humidity: 0.0,
        continentalness: 0.0,
        erosion: 0.0,
        depth_offset: 0.0,
        weirdness: 0.0,
    }; 16];
    let y0 = 0;
    let depth_gradient_at_y0 = OVERWORLD_DEPTH_GRADIENT_DENSITY.compute(y0);
    for local_z in 0..4_usize {
        for local_x in 0..4_usize {
            let quart_x = chunk_quart_x + local_x as i32;
            let quart_z = chunk_quart_z + local_z as i32;
            let block_x = quart_x * 4;
            let block_z = quart_z * 4;
            let eval = |function: DensityFunction| {
                function.compute_with_noise(
                    climate_sampler.seed,
                    climate_sampler.settings,
                    block_x,
                    y0,
                    block_z,
                )
            };
            cache[local_z * 4 + local_x] = OverworldBiome2dClimate {
                temperature: eval(climate_sampler.temperature) as f32,
                humidity: eval(climate_sampler.humidity) as f32,
                continentalness: eval(climate_sampler.continentalness) as f32,
                erosion: eval(climate_sampler.erosion) as f32,
                depth_offset: eval(climate_sampler.depth) - depth_gradient_at_y0,
                weirdness: eval(climate_sampler.weirdness) as f32,
            };
        }
    }
    Some(cache)
}
