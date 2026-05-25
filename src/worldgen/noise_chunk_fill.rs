use super::*;

// ── fill_from_noise_chunk ────────────────────────────────────────────────────

/// Fill a chunk's blocks using cell-based trilinear interpolation of the noise
/// density functions.
///
/// Mirrors Java `NoiseBasedChunkGenerator.doFill`.
///
/// When `settings.aquifers_enabled`, block placement uses `NoiseBasedAquifer`
/// (Voronoi cell + barrier noise) for underground fluid pockets.  Otherwise
/// a simplified global-fluid rule is applied (sea-level water, bedrock lava).
///
/// The returned chunk has status `"minecraft:noise"` and contains
/// `WORLD_SURFACE_WG` and `OCEAN_FLOOR_WG` heightmaps.
pub fn fill_from_noise_chunk(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> LevelChunk {
    fill_from_noise_chunk_timed(pos, settings, seed, noise_router).0
}

pub fn fill_from_noise_chunk_timed(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> (LevelChunk, LiveTerrainTimings) {
    // Activate the thread-local noise-snapshot cache so that Perlin noise tables
    // are initialised once per noise key per chunk, not once per cell-corner
    // evaluation — this makes chunk generation ~1000× faster.
    with_noise_snapshot_cache(|| {
        fill_from_noise_chunk_inner_timed(pos, settings, seed, noise_router)
    })
}

fn fill_from_noise_chunk_inner(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> LevelChunk {
    fill_from_noise_chunk_inner_timed(pos, settings, seed, noise_router).0
}

pub(super) fn fill_from_noise_chunk_inner_timed(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> (LevelChunk, LiveTerrainTimings) {
    let (mut chunk, timings, section_blocks, _) =
        fill_from_noise_chunk_inner_sections_timed(pos, settings, seed, noise_router);
    flush_generated_section_blocks(&mut chunk, &section_blocks);
    (chunk, timings)
}

pub(super) fn fill_from_noise_chunk_inner_sections_timed(
    pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) -> (
    LevelChunk,
    LiveTerrainTimings,
    GeneratedSectionBlocks,
    LiveNoiseGenerationContext,
) {
    let total_started = Instant::now();
    let mut timings = LiveTerrainTimings::default();
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:noise".to_string();

    let min_y = settings.noise.min_y;
    let height = settings.noise.height;
    let section_count = (height + 15) / 16;
    let min_section = min_y.div_euclid(16);
    chunk.min_section_y = min_section;

    let started = Instant::now();
    let mut section_blocks = GeneratedSectionBlocks::new(min_section, section_count);
    // Initialise all chunk sections to air.
    chunk.sections = (0..section_count)
        .map(|i| {
            let section_y = min_section + i;
            let biome = Tag::String("minecraft:plains".to_string());
            ChunkSection {
                y: section_y as i8,
                block_states: PalettedContainer::single(
                    block_state_tag_fast("minecraft:air"),
                    SECTION_VOLUME,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(biome, BIOME_SECTION_VOLUME).to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1i8; 2048]),
            }
        })
        .collect();
    timings.fill_init_sections_ms = started.elapsed().as_millis();

    let chunk_min_x = pos.x * 16;
    let chunk_min_z = pos.z * 16;

    let cell_width = settings.noise.cell_width();
    let cell_height = settings.noise.cell_height();
    let cell_count_xz = 16 / cell_width;
    let cell_count_y = height / cell_height;
    let cell_noise_min_y = min_y.div_euclid(cell_height);

    let started = Instant::now();
    let mut noise_chunk = NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
    timings.fill_noise_chunk_init_ms = started.elapsed().as_millis();
    timings.interpolator_count = noise_chunk.interpolators.len();

    // Create the noise-based aquifer when enabled, or use None for the simple path.
    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let started = Instant::now();
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
    timings.fill_aquifer_init_ms = started.elapsed().as_millis();
    let material_rules = NoiseMaterialRuleList::new(settings, factories.ore);

    // Heightmap accumulators (Y+1 of the highest non-air block).
    let mut ocean_floor = [min_y; 256];
    let mut world_surface = [min_y; 256];
    let detailed_timing = std::env::var_os("RUSTCRAFT_WORLDGEN_DETAILED_TIMING").is_some();

    let started = Instant::now();
    for cell_x_index in 0..cell_count_xz {
        // Fill the next-X slice for this column and set cell_start_block_x.
        noise_chunk.advance_cell_x(cell_x_index);

        for cell_z_index in 0..cell_count_xz {
            timings.cell_columns += 1;
            // Iterate Y from top of the world downward (matches Java doFill).
            for cell_y_index in (0..cell_count_y).rev() {
                noise_chunk.select_cell_yz(cell_y_index, cell_z_index);

                for y_in_cell in (0..cell_height).rev() {
                    let pos_y = (cell_noise_min_y + cell_y_index) * cell_height + y_in_cell;
                    let section_index = (pos_y.div_euclid(16) - min_section) as usize;
                    let local_y = (pos_y & 15) as usize;
                    let factor_y = y_in_cell as f64 / cell_height as f64;
                    if detailed_timing {
                        let interpolation_started = Instant::now();
                        noise_chunk.update_for_y(pos_y, factor_y);
                        timings.fill_interpolation_update_us +=
                            interpolation_started.elapsed().as_micros();
                    } else {
                        noise_chunk.update_for_y(pos_y, factor_y);
                    }

                    for x_in_cell in 0..cell_width {
                        let pos_x = chunk_min_x + cell_x_index * cell_width + x_in_cell;
                        let local_x = (pos_x & 15) as usize;
                        let factor_x = x_in_cell as f64 / cell_width as f64;
                        if detailed_timing {
                            let interpolation_started = Instant::now();
                            noise_chunk.update_for_x(pos_x, factor_x);
                            timings.fill_interpolation_update_us +=
                                interpolation_started.elapsed().as_micros();
                        } else {
                            noise_chunk.update_for_x(pos_x, factor_x);
                        }

                        for z_in_cell in 0..cell_width {
                            let pos_z = chunk_min_z + cell_z_index * cell_width + z_in_cell;
                            let local_z = (pos_z & 15) as usize;
                            let factor_z = z_in_cell as f64 / cell_width as f64;
                            if detailed_timing {
                                let interpolation_started = Instant::now();
                                noise_chunk.update_for_z(pos_z, factor_z);
                                timings.fill_interpolation_update_us +=
                                    interpolation_started.elapsed().as_micros();
                            } else {
                                noise_chunk.update_for_z(pos_z, factor_z);
                            }

                            timings.block_samples += 1;
                            let density = if detailed_timing {
                                let density_started = Instant::now();
                                let density = noise_chunk.interpolated_density(pos_x, pos_y, pos_z);
                                timings.fill_density_lookup_us +=
                                    density_started.elapsed().as_micros();
                                density
                            } else {
                                noise_chunk.interpolated_density(pos_x, pos_y, pos_z)
                            };

                            let block = material_rules.calculate(MaterialRuleCalculationInput {
                                aquifer: aquifer.as_mut(),
                                noise_chunk: &noise_chunk,
                                settings,
                                pos: BlockPos {
                                    x: pos_x,
                                    y: pos_y,
                                    z: pos_z,
                                },
                                density,
                                timings: &mut timings,
                                detailed_timing,
                            });

                            if block != "minecraft:air" {
                                let block_id = section_blocks.id_for(block);
                                let block_index = local_y * 256 + local_z * 16 + local_x;
                                let section = &mut section_blocks.sections[section_index];
                                let old_id = section.ids[block_index];
                                if old_id == 0 {
                                    section.non_air_blocks += 1;
                                }
                                section.ids[block_index] = block_id;
                                timings.block_writes += 1;
                                let idx = local_z * 16 + local_x;
                                if pos_y + 1 > world_surface[idx] {
                                    world_surface[idx] = pos_y + 1;
                                }
                                // Ocean floor tracks solid (non-fluid) blocks only.
                                if block != "minecraft:water"
                                    && block != "minecraft:lava"
                                    && pos_y + 1 > ocean_floor[idx]
                                {
                                    ocean_floor[idx] = pos_y + 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Swap so that the filled next-slice becomes the current slice for the
        // next iteration.
        noise_chunk.swap_slices();
    }
    timings.fill_block_loop_ms = started.elapsed().as_millis();
    let fill_stats = noise_chunk.take_fill_stats();
    timings.fill_full_noise_cache_ms = fill_stats.full_noise_cache_ms;
    timings.fill_full_noise_cache_us = fill_stats.full_noise_cache_us;
    timings.full_noise_cache_fills = fill_stats.full_noise_cache_fills;
    timings.fill_vein_noise_cache_ms = fill_stats.vein_noise_cache_ms;
    timings.fill_vein_noise_cache_us = fill_stats.vein_noise_cache_us;
    timings.vein_noise_cache_fills = fill_stats.vein_noise_cache_fills;
    timings.cache_once_scalar_hits = fill_stats.cache_once_scalar_hits;
    timings.cache_once_scalar_misses = fill_stats.cache_once_scalar_misses;
    timings.cache_once_array_hits = fill_stats.cache_once_array_hits;
    timings.cache_once_array_misses = fill_stats.cache_once_array_misses;

    let started = Instant::now();
    chunk.heightmaps = BTreeMap::from([
        (
            HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(world_surface)),
        ),
        (
            HeightmapKind::OceanFloorWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(ocean_floor)),
        ),
    ]);
    timings.fill_heightmap_pack_ms = started.elapsed().as_millis();
    timings.fill_total_ms = total_started.elapsed().as_millis();

    (
        chunk,
        timings,
        section_blocks,
        LiveNoiseGenerationContext {
            noise_chunk,
            aquifer,
        },
    )
}

pub(super) fn flush_generated_section_blocks(chunk: &mut LevelChunk, section_blocks: &GeneratedSectionBlocks) {
    let containers = section_blocks.to_paletted_containers();
    for (section, blocks) in chunk.sections.iter_mut().zip(containers.iter()) {
        section.block_states = blocks.to_nbt();
    }
}
