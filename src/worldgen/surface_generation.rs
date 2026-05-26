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
        SurfaceBuildInput {
            rule,
            biome_source_model,
            noise_router,
            settings,
            seed,
        },
        timings,
    );
}

#[derive(Clone, Copy)]
struct SurfaceBuildInput<'a> {
    rule: &'a DynSurfaceRule,
    biome_source_model: &'a BiomeSourceModel,
    noise_router: NoiseRouter,
    settings: &'a NoiseGeneratorSettings,
    seed: i64,
}

#[derive(Clone, Copy)]
struct SurfacePreliminaryLevels {
    north_west: i32,
    north_east: i32,
    south_west: i32,
    south_east: i32,
}

#[derive(Default)]
struct SurfaceDebugCounters {
    noise_sample_us: u128,
    rng_us: u128,
    height_read_us: u128,
    biome_us: u128,
    block_read_us: u128,
    ceiling_scan_us: u128,
    rule_us: u128,
    write_us: u128,
    flush_us: u128,
}

struct SurfaceBuildLoop<'a, 'b> {
    chunk: &'a crate::storage::chunk::LevelChunk,
    section_blocks: &'a mut GeneratedSectionBlocks,
    input: SurfaceBuildInput<'b>,
    algorithm: RandomAlgorithm,
    default_block: &'static str,
    surface_noise: Option<NormalNoiseSnapshot>,
    surface_secondary_noise: Option<NormalNoiseSnapshot>,
    chunk_min_x: i32,
    chunk_min_z: i32,
    prelim: SurfacePreliminaryLevels,
    base_rng: PositionalRandomFactory,
    climate_sampler: ClimateSampler,
    biome_zoom_seed: i64,
    surface_context: SurfaceRulesContext,
    chunk_noise_biomes: ChunkNoiseBiomeCache,
    surface_biome_cache: HashMap<(i32, i32, i32), (&'static str, f32)>,
    surface_noise_biome_cache: HashMap<(i32, i32, i32), &'static str>,
    default_block_id: u16,
    air_id: u16,
    water_id: u16,
    lava_id: u16,
    debug_enabled: bool,
    debug: SurfaceDebugCounters,
    timings: &'a mut LiveTerrainTimings,
}

struct SurfaceColumnState {
    block_x: i32,
    block_z: i32,
    local_x: usize,
    local_z: usize,
    surface_depth: i32,
    surface_secondary: f64,
    min_surface_level: i32,
    stone_depth_above: i32,
    water_height: i32,
    next_ceiling_stone_y: i32,
    end_y: i32,
    start_height: i32,
}

struct SurfaceBiomeResolver<'a, 'b> {
    input: SurfaceBuildInput<'b>,
    biome_zoom_seed: i64,
    climate_sampler: &'a ClimateSampler,
    chunk_noise_biomes: &'a ChunkNoiseBiomeCache,
    surface_biome_cache: &'a mut HashMap<(i32, i32, i32), (&'static str, f32)>,
    surface_noise_biome_cache: &'a mut HashMap<(i32, i32, i32), &'static str>,
    debug_enabled: bool,
    debug: &'a mut SurfaceDebugCounters,
}

fn build_surface_for_chunk_timed_with_sections(
    chunk: &mut crate::storage::chunk::LevelChunk,
    predecoded_section_blocks: Option<&mut GeneratedSectionBlocks>,
    input: SurfaceBuildInput<'_>,
    timings: &mut LiveTerrainTimings,
) {
    let total_started = Instant::now();
    let mut owned_section_blocks;
    let section_blocks: &mut GeneratedSectionBlocks =
        if let Some(section_blocks) = predecoded_section_blocks {
            section_blocks
        } else {
            owned_section_blocks = generated_section_blocks_for_surface(chunk);
            &mut owned_section_blocks
        };
    let started = Instant::now();
    let (surface_context, mut debug) = {
        let mut surface_loop = SurfaceBuildLoop::new(chunk, section_blocks, input, timings);
        surface_loop.run_columns();
        surface_loop.finish()
    };
    let flush_started = std::env::var_os("RUSTCRAFT_WORLDGEN_SURFACE_DEBUG")
        .is_some()
        .then(Instant::now);
    add_client_heightmaps_from_generated_sections(chunk, section_blocks);
    flush_generated_section_blocks(chunk, section_blocks);
    debug.flush_us = flush_started
        .map(|started| started.elapsed().as_micros())
        .unwrap_or(0);
    timings.surface_column_loop_ms = started.elapsed().as_millis();
    timings.surface_total_ms = total_started.elapsed().as_millis();
    log_surface_rule_debug(timings, &surface_context, &debug);
}

fn generated_section_blocks_for_surface(
    chunk: &crate::storage::chunk::LevelChunk,
) -> GeneratedSectionBlocks {
    chunk.sections.iter().fold(
        GeneratedSectionBlocks::new(chunk.min_section_y, chunk.sections.len() as i32),
        |mut generated, section| {
            copy_section_blocks_for_surface(&mut generated, section);
            generated
        },
    )
}

fn copy_section_blocks_for_surface(
    generated: &mut GeneratedSectionBlocks,
    section: &crate::storage::chunk::ChunkSection,
) {
    let Ok(container) = PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME) else {
        return;
    };
    let section_index = i32::from(section.y) - generated.min_section_y;
    if section_index < 0 || (section_index as usize) >= generated.sections.len() {
        return;
    }
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

fn surface_random_algorithm(settings: &NoiseGeneratorSettings) -> RandomAlgorithm {
    if settings.legacy_random_source {
        RandomAlgorithm::Legacy
    } else {
        RandomAlgorithm::Xoroshiro
    }
}

fn surface_preliminary_levels(
    input: SurfaceBuildInput<'_>,
    chunk_min_x: i32,
    chunk_min_z: i32,
) -> SurfacePreliminaryLevels {
    let prelim_fn = input.noise_router.preliminary_surface_level;
    let prelim_q = |bx: i32, bz: i32| {
        let qx = (bx >> 2) << 2; // QuartPos round-down
        let qz = (bz >> 2) << 2;
        prelim_fn
            .compute_with_noise(input.seed, *input.settings, qx, 0, qz)
            .floor() as i32
    };
    SurfacePreliminaryLevels {
        north_west: prelim_q(chunk_min_x, chunk_min_z),
        north_east: prelim_q(chunk_min_x + 16, chunk_min_z),
        south_west: prelim_q(chunk_min_x, chunk_min_z + 16),
        south_east: prelim_q(chunk_min_x + 16, chunk_min_z + 16),
    }
}

fn log_surface_rule_debug(
    timings: &LiveTerrainTimings,
    surface_context: &SurfaceRulesContext,
    debug: &SurfaceDebugCounters,
) {
    if let Some(profile) = &surface_context.profile {
        let profile = *profile.borrow();
        eprintln!(
            "[surface-rule-debug] total={}ms loop={}ms columns={} samples={} writes={} noise_sample={}us rng={}us height_reads={}us biome={}us block_reads={}us ceiling_scan={}us rule_apply={}us writes={}us flush={}us rule_visits={} sequence={} condition_rules={} block_rules={} bandlands={} condition_tests={} cache_hits={} computes={} compute={}us",
            timings.surface_total_ms,
            timings.surface_column_loop_ms,
            timings.surface_columns,
            timings.surface_block_samples,
            timings.surface_block_writes,
            debug.noise_sample_us,
            debug.rng_us,
            debug.height_read_us,
            debug.biome_us,
            debug.block_read_us,
            debug.ceiling_scan_us,
            debug.rule_us,
            debug.write_us,
            debug.flush_us,
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

impl<'a, 'b> SurfaceBuildLoop<'a, 'b> {
    fn new(
        chunk: &'a crate::storage::chunk::LevelChunk,
        section_blocks: &'a mut GeneratedSectionBlocks,
        input: SurfaceBuildInput<'b>,
        timings: &'a mut LiveTerrainTimings,
    ) -> Self {
        let algorithm = surface_random_algorithm(input.settings);
        let heights = WorldGenerationHeightContext {
            min_y: input.settings.noise.min_y,
            height: input.settings.noise.height,
        };
        let started = Instant::now();
        let surface_noise =
            random_state_normal_noise_snapshot(input.seed, *input.settings, "minecraft:surface");
        let surface_secondary_noise = random_state_normal_noise_snapshot(
            input.seed,
            *input.settings,
            "minecraft:surface_secondary",
        );
        timings.surface_noise_setup_ms = started.elapsed().as_millis();

        let chunk_min_x = chunk.pos.x * 16;
        let chunk_min_z = chunk.pos.z * 16;
        let started = Instant::now();
        let prelim = surface_preliminary_levels(input, chunk_min_x, chunk_min_z);
        timings.surface_prelim_ms = started.elapsed().as_millis();

        let default_block_id = section_blocks.id_for(input.settings.default_block);
        let air_id = section_blocks.id_for("minecraft:air");
        let water_id = section_blocks.id_for("minecraft:water");
        let lava_id = section_blocks.id_for("minecraft:lava");
        Self {
            chunk,
            section_blocks,
            input,
            algorithm,
            default_block: input.settings.default_block,
            surface_noise,
            surface_secondary_noise,
            chunk_min_x,
            chunk_min_z,
            prelim,
            base_rng: random_state_seed_factories(input.seed, algorithm).base,
            climate_sampler: ClimateSampler::from_noise_router(
                &input.noise_router,
                input.seed,
                *input.settings,
            ),
            biome_zoom_seed: biome_manager_obfuscate_seed(input.seed),
            surface_context: SurfaceRulesContext::new(input.seed, algorithm, heights),
            chunk_noise_biomes: ChunkNoiseBiomeCache::from_chunk(chunk),
            surface_biome_cache: HashMap::new(),
            surface_noise_biome_cache: HashMap::new(),
            default_block_id,
            air_id,
            water_id,
            lava_id,
            debug_enabled: std::env::var_os("RUSTCRAFT_WORLDGEN_SURFACE_DEBUG").is_some(),
            debug: SurfaceDebugCounters::default(),
            timings,
        }
    }

    fn run_columns(&mut self) {
        for local_z in 0..16_i32 {
            for local_x in 0..16_i32 {
                self.build_column(local_x, local_z);
            }
        }
    }

    fn finish(self) -> (SurfaceRulesContext, SurfaceDebugCounters) {
        (self.surface_context, self.debug)
    }

    fn build_column(&mut self, local_x: i32, local_z: i32) {
        self.timings.surface_columns += 1;
        let mut column = self.prepare_column(local_x, local_z);
        let steep = self.is_steep_column(column.local_x, column.local_z);
        self.surface_context.update_xz(
            column.block_x,
            column.block_z,
            column.surface_depth,
            column.surface_secondary,
            steep,
            column.surface_depth <= 0,
            column.min_surface_level,
        );
        for y in (column.end_y..=column.start_height).rev() {
            self.process_column_y(&mut column, y);
        }
    }

    fn prepare_column(&mut self, local_x: i32, local_z: i32) -> SurfaceColumnState {
        let block_x = self.chunk_min_x + local_x;
        let block_z = self.chunk_min_z + local_z;
        let local_x_usize = local_x as usize;
        let local_z_usize = local_z as usize;
        let surface_depth = self.surface_depth(block_x, block_z);
        let surface_secondary = self.surface_secondary(block_x, block_z);
        let min_surface_level = self.min_surface_level(local_x, local_z, surface_depth);
        SurfaceColumnState {
            block_x,
            block_z,
            local_x: local_x_usize,
            local_z: local_z_usize,
            surface_depth,
            surface_secondary,
            min_surface_level,
            stone_depth_above: 0,
            water_height: i32::MIN,
            next_ceiling_stone_y: i32::MAX,
            end_y: self.input.settings.noise.min_y,
            start_height: read_world_surface_wg(self.chunk, local_x_usize, local_z_usize) + 1,
        }
    }

    fn surface_depth(&mut self, block_x: i32, block_z: i32) -> i32 {
        let debug_started = self.debug_enabled.then(Instant::now);
        let surface_noise = self
            .surface_noise
            .as_ref()
            .map(|snap| normal_noise_sample(snap, f64::from(block_x), 0.0, f64::from(block_z)))
            .unwrap_or(0.0);
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.noise_sample_us += elapsed;
        });

        let debug_started = self.debug_enabled.then(Instant::now);
        let mut at_rng = self.base_rng.at(block_x, 0, block_z);
        let jitter = random_next_f64(&mut at_rng) * 0.25;
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.rng_us += elapsed;
        });
        (surface_noise * 2.75 + 3.0 + jitter) as i32
    }

    fn surface_secondary(&mut self, block_x: i32, block_z: i32) -> f64 {
        let debug_started = self.debug_enabled.then(Instant::now);
        let surface_secondary = self
            .surface_secondary_noise
            .as_ref()
            .map(|snap| normal_noise_sample(snap, f64::from(block_x), 0.0, f64::from(block_z)))
            .unwrap_or(0.0);
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.noise_sample_us += elapsed;
        });
        surface_secondary
    }

    fn is_steep_column(&mut self, local_x: usize, local_z: usize) -> bool {
        let debug_started = self.debug_enabled.then(Instant::now);
        let h_n = read_world_surface_wg(self.chunk, local_x, local_z.saturating_sub(1));
        let h_s = read_world_surface_wg(self.chunk, local_x, (local_z + 1).min(15));
        let h_w = read_world_surface_wg(self.chunk, local_x.saturating_sub(1), local_z);
        let h_e = read_world_surface_wg(self.chunk, (local_x + 1).min(15), local_z);
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.height_read_us += elapsed;
        });
        h_s >= h_n + 4 || h_w >= h_e + 4
    }

    fn min_surface_level(&self, local_x: i32, local_z: i32, surface_depth: i32) -> i32 {
        let tx = f64::from(local_x) / 16.0;
        let tz = f64::from(local_z) / 16.0;
        let prelim = lerp(
            tz,
            lerp(
                tx,
                f64::from(self.prelim.north_west),
                f64::from(self.prelim.north_east),
            ),
            lerp(
                tx,
                f64::from(self.prelim.south_west),
                f64::from(self.prelim.south_east),
            ),
        )
        .floor() as i32;
        prelim + surface_depth - 8
    }

    fn process_column_y(&mut self, column: &mut SurfaceColumnState, y: i32) {
        self.timings.surface_block_samples += 1;
        let block_id = self.block_id_with_debug(column.block_x, y, column.block_z);
        if block_id == self.air_id {
            column.stone_depth_above = 0;
            column.water_height = i32::MIN;
        } else if block_id == self.water_id || block_id == self.lava_id {
            if column.water_height == i32::MIN {
                column.water_height = y + 1;
            }
        } else {
            self.process_solid_block(column, y, block_id);
        }
    }

    fn block_id_with_debug(&mut self, block_x: i32, y: i32, block_z: i32) -> u16 {
        let debug_started = self.debug_enabled.then(Instant::now);
        let block_id = self.section_blocks.get_id(block_x, y, block_z);
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.block_read_us += elapsed;
        });
        block_id
    }

    fn process_solid_block(&mut self, column: &mut SurfaceColumnState, y: i32, block_id: u16) {
        self.refresh_next_ceiling_stone_y(column, y);
        column.stone_depth_above += 1;
        let stone_depth_below = y - column.next_ceiling_stone_y + 1;
        if block_id == self.default_block_id {
            self.apply_default_block_surface_rule(column, y, stone_depth_below);
        }
    }

    fn refresh_next_ceiling_stone_y(&mut self, column: &mut SurfaceColumnState, y: i32) {
        if column.next_ceiling_stone_y < y {
            return;
        }
        const WAY_BELOW_MIN_Y: i32 = i32::MIN / 2;
        column.next_ceiling_stone_y = WAY_BELOW_MIN_Y;
        let debug_started = self.debug_enabled.then(Instant::now);
        let mut la = y - 1;
        while la >= column.end_y - 1 {
            let la_block_id = self
                .section_blocks
                .get_id(column.block_x, la, column.block_z);
            if la_block_id == self.air_id
                || la_block_id == self.water_id
                || la_block_id == self.lava_id
            {
                column.next_ceiling_stone_y = la + 1;
                break;
            }
            la -= 1;
        }
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.ceiling_scan_us += elapsed;
        });
    }

    fn apply_default_block_surface_rule(
        &mut self,
        column: &mut SurfaceColumnState,
        y: i32,
        stone_depth_below: i32,
    ) {
        if y < column.min_surface_level && y >= 8 {
            return;
        }
        self.surface_context.update_y(
            column.stone_depth_above,
            stone_depth_below,
            column.water_height,
            y,
        );
        let band_fn = |wx: i32, by: i32, wz: i32| {
            get_clay_band(
                self.input.seed,
                self.algorithm,
                *self.input.settings,
                wx,
                by,
                wz,
            )
        };
        let mut resolver = SurfaceBiomeResolver {
            input: self.input,
            biome_zoom_seed: self.biome_zoom_seed,
            climate_sampler: &self.climate_sampler,
            chunk_noise_biomes: &self.chunk_noise_biomes,
            surface_biome_cache: &mut self.surface_biome_cache,
            surface_noise_biome_cache: &mut self.surface_noise_biome_cache,
            debug_enabled: self.debug_enabled,
            debug: &mut self.debug,
        };
        let mut biome_resolver =
            |block_x: i32, block_y: i32, block_z: i32| resolver.resolve(block_x, block_y, block_z);
        let debug_started = self.debug_enabled.then(Instant::now);
        let new_block = dyn_surface_rule_apply(
            self.input.rule,
            &mut self.surface_context,
            *self.input.settings,
            &band_fn,
            &mut biome_resolver,
        );
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.rule_us += elapsed;
        });
        if let Some(new_block) = new_block {
            self.write_surface_block(column, y, new_block);
        }
    }

    fn write_surface_block(&mut self, column: &SurfaceColumnState, y: i32, new_block: &str) {
        if new_block == self.default_block {
            return;
        }
        let debug_started = self.debug_enabled.then(Instant::now);
        self.section_blocks
            .set_name(column.block_x, y, column.block_z, new_block);
        self.add_debug_time(debug_started, |debug, elapsed| {
            debug.write_us += elapsed;
        });
        self.timings.surface_block_writes += 1;
    }

    fn add_debug_time(
        &mut self,
        started: Option<Instant>,
        add: impl FnOnce(&mut SurfaceDebugCounters, u128),
    ) {
        if let Some(started) = started {
            add(&mut self.debug, started.elapsed().as_micros());
        }
    }
}

impl SurfaceBiomeResolver<'_, '_> {
    fn resolve(&mut self, block_x: i32, block_y: i32, block_z: i32) -> (&'static str, f32) {
        let biome_y = if self.input.settings.legacy_random_source {
            0
        } else {
            block_y
        };
        let biome_key = (block_x, biome_y, block_z);
        if let Some(cached) = self.surface_biome_cache.get(&biome_key).copied() {
            return cached;
        }
        let debug_started = self.debug_enabled.then(Instant::now);
        let biome = biome_manager_get_biome_cached(
            self.input.biome_source_model,
            self.biome_zoom_seed,
            BlockPos {
                x: block_x,
                y: biome_y,
                z: block_z,
            },
            self.climate_sampler,
            Some(self.chunk_noise_biomes),
            self.surface_noise_biome_cache,
        )
        .unwrap_or("minecraft:plains");
        let temperature = surface_biome_temperature(biome);
        if let Some(started) = debug_started {
            self.debug.biome_us += started.elapsed().as_micros();
        }
        self.surface_biome_cache
            .insert(biome_key, (biome, temperature));
        (biome, temperature)
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
            SurfaceBuildInput {
                rule: surface_rule,
                biome_source_model,
                noise_router,
                settings,
                seed,
            },
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
    let overworld_column_biomes =
        overworld_column_biomes_from_climate(overworld_2d_climate.as_ref());
    let population = NoiseChunkBiomePopulation {
        biome_source_model,
        climate_sampler: &climate_sampler,
        overworld_2d_climate: overworld_2d_climate.as_ref(),
        overworld_column_biomes,
        chunk_quart_x,
        chunk_quart_z,
    };

    let fill_started = debug_enabled.then(Instant::now);
    let mut selections = 0_usize;
    for section in &mut chunk.sections {
        selections += populate_noise_chunk_biome_section(section, &population, &mut biome_tags);
    }
    log_biome_storage_debug(BiomeStorageDebugReport {
        enabled: debug_enabled,
        total_started,
        climate_ms,
        cache_ms,
        fill_started,
        sections: chunk.sections.len(),
        selections,
        tags: biome_tags.len(),
    });
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

struct NoiseChunkBiomePopulation<'a> {
    biome_source_model: &'a BiomeSourceModel,
    climate_sampler: &'a ClimateSampler,
    overworld_2d_climate: Option<&'a [OverworldBiome2dClimate; 16]>,
    overworld_column_biomes: Option<[&'static str; 16]>,
    chunk_quart_x: i32,
    chunk_quart_z: i32,
}

impl NoiseChunkBiomePopulation<'_> {
    fn biome_at(
        &self,
        local_x: usize,
        local_y: usize,
        local_z: usize,
        section_quart_y: i32,
    ) -> &'static str {
        if let Some(column_biomes) = &self.overworld_column_biomes {
            return column_biomes[local_z * 4 + local_x];
        }

        let quart_x = self.chunk_quart_x + local_x as i32;
        let quart_y = section_quart_y + local_y as i32;
        let quart_z = self.chunk_quart_z + local_z as i32;
        self.overworld_2d_climate
            .and_then(|cache| {
                let cached = cache[local_z * 4 + local_x];
                let block_y = quart_y * 4;
                let depth = cached.depth_offset + OVERWORLD_DEPTH_GRADIENT_DENSITY.compute(block_y);
                let climate = climate_target(
                    cached.temperature,
                    cached.humidity,
                    cached.continentalness,
                    cached.erosion,
                    depth as f32,
                    cached.weirdness,
                );
                select_biome_from_source(
                    self.biome_source_model,
                    quart_x,
                    quart_y,
                    quart_z,
                    climate,
                    0.0,
                )
            })
            .or_else(|| {
                get_biome(
                    self.biome_source_model,
                    quart_x,
                    quart_y,
                    quart_z,
                    self.climate_sampler,
                )
            })
            .unwrap_or("minecraft:plains")
    }
}

fn overworld_column_biomes_from_climate(
    climate_cache: Option<&[OverworldBiome2dClimate; 16]>,
) -> Option<[&'static str; 16]> {
    climate_cache.map(|cache| {
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
    })
}

fn populate_noise_chunk_biome_section(
    section: &mut crate::storage::chunk::ChunkSection,
    population: &NoiseChunkBiomePopulation<'_>,
    biome_tags: &mut HashMap<&'static str, Tag>,
) -> usize {
    let section_quart_y = i32::from(section.y) * 4;
    let mut palette_names: Vec<&'static str> = Vec::with_capacity(2);
    let mut indices = vec![0_u64; BIOME_SECTION_VOLUME];

    for local_y in 0..4_usize {
        for local_z in 0..4_usize {
            for local_x in 0..4_usize {
                let biome = population.biome_at(local_x, local_y, local_z, section_quart_y);
                let index = local_y * 16 + local_z * 4 + local_x;
                indices[index] = biome_palette_index(&mut palette_names, biome) as u64;
            }
        }
    }

    let palette = biome_palette_tags(&palette_names, biome_tags);
    section.biomes = biome_section_palette_nbt(palette, &indices);
    BIOME_SECTION_VOLUME
}

fn biome_palette_index(palette_names: &mut Vec<&'static str>, biome: &'static str) -> usize {
    match palette_names
        .iter()
        .position(|candidate| *candidate == biome)
    {
        Some(index) => index,
        None => {
            palette_names.push(biome);
            palette_names.len() - 1
        }
    }
}

fn biome_palette_tags(
    palette_names: &[&'static str],
    biome_tags: &mut HashMap<&'static str, Tag>,
) -> Vec<Tag> {
    palette_names
        .iter()
        .map(|biome| {
            biome_tags
                .entry(*biome)
                .or_insert_with(|| Tag::String((*biome).to_string()))
                .clone()
        })
        .collect()
}

fn biome_section_palette_nbt(palette: Vec<Tag>, indices: &[u64]) -> Tag {
    if palette.len() == 1 {
        return PalettedContainer::single(palette[0].clone(), BIOME_SECTION_VOLUME).to_nbt();
    }
    PalettedContainer {
        data: Some(pack_palette_indices(
            indices,
            palette_bits_for_size(palette.len()),
        )),
        palette,
        expected_entries: BIOME_SECTION_VOLUME,
    }
    .to_nbt()
}

struct BiomeStorageDebugReport {
    enabled: bool,
    total_started: Option<Instant>,
    climate_ms: Option<u128>,
    cache_ms: Option<u128>,
    fill_started: Option<Instant>,
    sections: usize,
    selections: usize,
    tags: usize,
}

fn log_biome_storage_debug(report: BiomeStorageDebugReport) {
    if !report.enabled {
        return;
    }
    eprintln!(
        "[biome-storage-debug] total={}ms climate={}ms cache={}ms fill={}ms sections={} selections={} tags={}",
        report
            .total_started
            .map(|started| started.elapsed().as_millis())
            .unwrap_or(0),
        report.climate_ms.unwrap_or(0),
        report.cache_ms.unwrap_or(0),
        report
            .fill_started
            .map(|started| started.elapsed().as_millis())
            .unwrap_or(0),
        report.sections,
        report.selections,
        report.tags
    );
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
