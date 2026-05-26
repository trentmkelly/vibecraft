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

    let dimensions = NoiseFillDimensions::from_chunk(pos, settings);

    let started = Instant::now();
    let mut section_blocks = initialize_noise_sections(&mut chunk, &dimensions);
    timings.fill_init_sections_ms = started.elapsed().as_millis();

    let started = Instant::now();
    let mut noise_chunk = NoiseChunk::new(
        dimensions.chunk_min_x,
        dimensions.chunk_min_z,
        *settings,
        seed,
        noise_router,
    );
    timings.fill_noise_chunk_init_ms = started.elapsed().as_millis();
    timings.interpolator_count = noise_chunk.interpolators.len();

    let algorithm = if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    };
    let factories = crate::random_source::random_state_seed_factories(seed, algorithm);
    let started = Instant::now();
    let mut aquifer = create_noise_aquifer(
        &mut noise_chunk,
        &dimensions,
        settings,
        seed,
        noise_router,
        factories.aquifer,
    );
    timings.fill_aquifer_init_ms = started.elapsed().as_millis();
    let mut heightmaps = NoiseFillHeightmaps::new(dimensions.min_y);

    fill_noise_chunk_blocks(
        NoiseFillBlockContext {
            dimensions: &dimensions,
            settings,
            material_rules: &NoiseMaterialRuleList::new(settings, factories.ore),
            detailed_timing: std::env::var_os("RUSTCRAFT_WORLDGEN_DETAILED_TIMING").is_some(),
        },
        &mut noise_chunk,
        &mut aquifer,
        &mut section_blocks,
        &mut timings,
        &mut heightmaps,
    );

    let started = Instant::now();
    chunk.heightmaps = heightmaps.into_nbt();
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

#[derive(Clone, Copy)]
struct NoiseFillDimensions {
    min_y: i32,
    height: i32,
    section_count: i32,
    min_section: i32,
    chunk_min_x: i32,
    chunk_min_z: i32,
    cell_width: i32,
    cell_height: i32,
    cell_count_xz: i32,
    cell_count_y: i32,
    cell_noise_min_y: i32,
}

impl NoiseFillDimensions {
    fn from_chunk(pos: ChunkPos, settings: &NoiseGeneratorSettings) -> Self {
        let min_y = settings.noise.min_y;
        let height = settings.noise.height;
        let cell_width = settings.noise.cell_width();
        let cell_height = settings.noise.cell_height();
        Self {
            min_y,
            height,
            section_count: (height + 15) / 16,
            min_section: min_y.div_euclid(16),
            chunk_min_x: pos.x * 16,
            chunk_min_z: pos.z * 16,
            cell_width,
            cell_height,
            cell_count_xz: 16 / cell_width,
            cell_count_y: height / cell_height,
            cell_noise_min_y: min_y.div_euclid(cell_height),
        }
    }
}

fn initialize_noise_sections(
    chunk: &mut LevelChunk,
    dimensions: &NoiseFillDimensions,
) -> GeneratedSectionBlocks {
    chunk.min_section_y = dimensions.min_section;
    chunk.sections = (0..dimensions.section_count)
        .map(|i| {
            let section_y = dimensions.min_section + i;
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
    GeneratedSectionBlocks::new(dimensions.min_section, dimensions.section_count)
}

fn create_noise_aquifer(
    noise_chunk: &mut NoiseChunk,
    dimensions: &NoiseFillDimensions,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
    pos_factory: crate::random_source::PositionalRandomFactory,
) -> Option<NoiseBasedAquifer> {
    settings.aquifers_enabled.then(|| {
        NoiseBasedAquifer::new(
            noise_chunk,
            dimensions.chunk_min_x,
            dimensions.chunk_min_x + 15,
            dimensions.chunk_min_z,
            dimensions.chunk_min_z + 15,
            dimensions.min_y,
            dimensions.height,
            seed,
            *settings,
            noise_router,
            pos_factory,
        )
    })
}

struct NoiseFillHeightmaps {
    ocean_floor: [i32; 256],
    world_surface: [i32; 256],
}

impl NoiseFillHeightmaps {
    fn new(min_y: i32) -> Self {
        Self {
            ocean_floor: [min_y; 256],
            world_surface: [min_y; 256],
        }
    }

    fn record_non_air_block(&mut self, local_x: usize, local_z: usize, pos_y: i32, block: &str) {
        let index = local_z * 16 + local_x;
        let height = pos_y + 1;
        if height > self.world_surface[index] {
            self.world_surface[index] = height;
        }
        if block != "minecraft:water"
            && block != "minecraft:lava"
            && height > self.ocean_floor[index]
        {
            self.ocean_floor[index] = height;
        }
    }

    fn into_nbt(self) -> BTreeMap<String, Tag> {
        BTreeMap::from([
            (
                HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
                Tag::LongArray(pack_heightmap(self.world_surface)),
            ),
            (
                HeightmapKind::OceanFloorWg.storage_name().to_string(),
                Tag::LongArray(pack_heightmap(self.ocean_floor)),
            ),
        ])
    }
}

struct NoiseFillBlockContext<'a> {
    dimensions: &'a NoiseFillDimensions,
    settings: &'a NoiseGeneratorSettings,
    material_rules: &'a NoiseMaterialRuleList,
    detailed_timing: bool,
}

#[derive(Clone, Copy)]
struct NoiseCellIndices {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Clone, Copy)]
struct NoiseSampleY {
    pos_y: i32,
    section_index: usize,
    local_y: usize,
    factor_y: f64,
}

#[derive(Clone, Copy)]
struct NoiseSampleX {
    pos_x: i32,
    pos_y: i32,
    cell_z_index: i32,
    section_index: usize,
    local_x: usize,
    local_y: usize,
}

fn fill_noise_chunk_blocks(
    context: NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    aquifer: &mut Option<NoiseBasedAquifer>,
    section_blocks: &mut GeneratedSectionBlocks,
    timings: &mut LiveTerrainTimings,
    heightmaps: &mut NoiseFillHeightmaps,
) {
    let started = Instant::now();
    for cell_x_index in 0..context.dimensions.cell_count_xz {
        noise_chunk.advance_cell_x(cell_x_index);
        fill_noise_chunk_x_slice(
            &context,
            noise_chunk,
            aquifer,
            section_blocks,
            timings,
            heightmaps,
            cell_x_index,
        );
        noise_chunk.swap_slices();
    }
    timings.fill_block_loop_ms = started.elapsed().as_millis();
    apply_noise_fill_stats(noise_chunk, timings);
}

fn fill_noise_chunk_x_slice(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    aquifer: &mut Option<NoiseBasedAquifer>,
    section_blocks: &mut GeneratedSectionBlocks,
    timings: &mut LiveTerrainTimings,
    heightmaps: &mut NoiseFillHeightmaps,
    cell_x_index: i32,
) {
    for cell_z_index in 0..context.dimensions.cell_count_xz {
        timings.cell_columns += 1;
        for cell_y_index in (0..context.dimensions.cell_count_y).rev() {
            fill_noise_cell_y(
                context,
                noise_chunk,
                aquifer,
                section_blocks,
                timings,
                heightmaps,
                NoiseCellIndices {
                    x: cell_x_index,
                    y: cell_y_index,
                    z: cell_z_index,
                },
            );
        }
    }
}

fn fill_noise_cell_y(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    aquifer: &mut Option<NoiseBasedAquifer>,
    section_blocks: &mut GeneratedSectionBlocks,
    timings: &mut LiveTerrainTimings,
    heightmaps: &mut NoiseFillHeightmaps,
    cell: NoiseCellIndices,
) {
    noise_chunk.select_cell_yz(cell.y, cell.z);

    for y_in_cell in (0..context.dimensions.cell_height).rev() {
        let sample_y = sample_y_position(context.dimensions, cell.y, y_in_cell);
        update_noise_y(context, noise_chunk, timings, sample_y);

        for x_in_cell in 0..context.dimensions.cell_width {
            let sample_x = sample_x_position(context.dimensions, cell, sample_y, x_in_cell);
            update_noise_x(context, noise_chunk, timings, sample_x, x_in_cell);
            fill_noise_z_row(
                context,
                noise_chunk,
                aquifer,
                section_blocks,
                timings,
                heightmaps,
                sample_x,
            );
        }
    }
}

fn sample_y_position(
    dimensions: &NoiseFillDimensions,
    cell_y_index: i32,
    y_in_cell: i32,
) -> NoiseSampleY {
    let pos_y = (dimensions.cell_noise_min_y + cell_y_index) * dimensions.cell_height + y_in_cell;
    NoiseSampleY {
        pos_y,
        section_index: (pos_y.div_euclid(16) - dimensions.min_section) as usize,
        local_y: (pos_y & 15) as usize,
        factor_y: y_in_cell as f64 / dimensions.cell_height as f64,
    }
}

fn sample_x_position(
    dimensions: &NoiseFillDimensions,
    cell: NoiseCellIndices,
    sample_y: NoiseSampleY,
    x_in_cell: i32,
) -> NoiseSampleX {
    let pos_x = dimensions.chunk_min_x + cell.x * dimensions.cell_width + x_in_cell;
    NoiseSampleX {
        pos_x,
        pos_y: sample_y.pos_y,
        cell_z_index: cell.z,
        section_index: sample_y.section_index,
        local_x: (pos_x & 15) as usize,
        local_y: sample_y.local_y,
    }
}

fn update_noise_y(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    timings: &mut LiveTerrainTimings,
    sample_y: NoiseSampleY,
) {
    if context.detailed_timing {
        let interpolation_started = Instant::now();
        noise_chunk.update_for_y(sample_y.pos_y, sample_y.factor_y);
        timings.fill_interpolation_update_us += interpolation_started.elapsed().as_micros();
    } else {
        noise_chunk.update_for_y(sample_y.pos_y, sample_y.factor_y);
    }
}

fn update_noise_x(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    timings: &mut LiveTerrainTimings,
    sample_x: NoiseSampleX,
    x_in_cell: i32,
) {
    let factor_x = x_in_cell as f64 / context.dimensions.cell_width as f64;
    if context.detailed_timing {
        let interpolation_started = Instant::now();
        noise_chunk.update_for_x(sample_x.pos_x, factor_x);
        timings.fill_interpolation_update_us += interpolation_started.elapsed().as_micros();
    } else {
        noise_chunk.update_for_x(sample_x.pos_x, factor_x);
    }
}

fn fill_noise_z_row(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    aquifer: &mut Option<NoiseBasedAquifer>,
    section_blocks: &mut GeneratedSectionBlocks,
    timings: &mut LiveTerrainTimings,
    heightmaps: &mut NoiseFillHeightmaps,
    sample_x: NoiseSampleX,
) {
    for z_in_cell in 0..context.dimensions.cell_width {
        let pos_z = context.dimensions.chunk_min_z
            + sample_x.cell_z_index * context.dimensions.cell_width
            + z_in_cell;
        let local_z = (pos_z & 15) as usize;
        update_noise_z(context, noise_chunk, timings, pos_z, z_in_cell);
        write_noise_block(
            context,
            NoiseBlockSample {
                pos: BlockPos {
                    x: sample_x.pos_x,
                    y: sample_x.pos_y,
                    z: pos_z,
                },
                section_index: sample_x.section_index,
                local_x: sample_x.local_x,
                local_y: sample_x.local_y,
                local_z,
            },
            noise_chunk,
            aquifer,
            section_blocks,
            timings,
            heightmaps,
        );
    }
}

fn update_noise_z(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &mut NoiseChunk,
    timings: &mut LiveTerrainTimings,
    pos_z: i32,
    z_in_cell: i32,
) {
    let factor_z = z_in_cell as f64 / context.dimensions.cell_width as f64;
    if context.detailed_timing {
        let interpolation_started = Instant::now();
        noise_chunk.update_for_z(pos_z, factor_z);
        timings.fill_interpolation_update_us += interpolation_started.elapsed().as_micros();
    } else {
        noise_chunk.update_for_z(pos_z, factor_z);
    }
}

struct NoiseBlockSample {
    pos: BlockPos,
    section_index: usize,
    local_x: usize,
    local_y: usize,
    local_z: usize,
}

fn write_noise_block(
    context: &NoiseFillBlockContext<'_>,
    sample: NoiseBlockSample,
    noise_chunk: &mut NoiseChunk,
    aquifer: &mut Option<NoiseBasedAquifer>,
    section_blocks: &mut GeneratedSectionBlocks,
    timings: &mut LiveTerrainTimings,
    heightmaps: &mut NoiseFillHeightmaps,
) {
    timings.block_samples += 1;
    let density = interpolated_density(context, noise_chunk, timings, sample.pos);
    let block = context
        .material_rules
        .calculate(MaterialRuleCalculationInput {
            aquifer: aquifer.as_mut(),
            noise_chunk,
            settings: context.settings,
            pos: sample.pos,
            density,
            timings,
            detailed_timing: context.detailed_timing,
        });

    if block == "minecraft:air" {
        return;
    }

    let block_id = section_blocks.id_for(block);
    let block_index = sample.local_y * 256 + sample.local_z * 16 + sample.local_x;
    let section = &mut section_blocks.sections[sample.section_index];
    let old_id = section.ids[block_index];
    if old_id == 0 {
        section.non_air_blocks += 1;
    }
    section.ids[block_index] = block_id;
    timings.block_writes += 1;
    heightmaps.record_non_air_block(sample.local_x, sample.local_z, sample.pos.y, block);
}

fn interpolated_density(
    context: &NoiseFillBlockContext<'_>,
    noise_chunk: &NoiseChunk,
    timings: &mut LiveTerrainTimings,
    pos: BlockPos,
) -> f64 {
    if context.detailed_timing {
        let density_started = Instant::now();
        let density = noise_chunk.interpolated_density(pos.x, pos.y, pos.z);
        timings.fill_density_lookup_us += density_started.elapsed().as_micros();
        density
    } else {
        noise_chunk.interpolated_density(pos.x, pos.y, pos.z)
    }
}

fn apply_noise_fill_stats(noise_chunk: &mut NoiseChunk, timings: &mut LiveTerrainTimings) {
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
}

pub(super) fn flush_generated_section_blocks(
    chunk: &mut LevelChunk,
    section_blocks: &GeneratedSectionBlocks,
) {
    let containers = section_blocks.to_paletted_containers();
    for (section, blocks) in chunk.sections.iter_mut().zip(containers.iter()) {
        section.block_states = blocks.to_nbt();
    }
}
