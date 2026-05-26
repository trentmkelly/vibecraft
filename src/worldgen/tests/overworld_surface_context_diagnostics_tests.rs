use super::*;

struct NormalOverworldNoiseConfig {
    biome_source_model: BiomeSourceModel,
    noise_settings: &'static super::super::NoiseGeneratorSettings,
}

struct SurfaceWaterDiagnosticContext {
    noise_router: super::super::NoiseRouter,
    climate_sampler: super::super::ClimateSampler,
    biome_zoom_seed: i64,
    surface_noise: Option<super::super::NormalNoiseSnapshot>,
    base_rng: crate::random_source::PositionalRandomFactory,
}

struct SurfaceWaterMismatch<'a> {
    pos: ChunkPos,
    local_x: usize,
    local_z: usize,
    world_y: i32,
    y_min: i32,
    y_max: i32,
    y_column: &'a [serde_json::Value],
    expected: &'a str,
    actual: &'a str,
    base: &'a LevelChunk,
    ore: &'a LevelChunk,
    seed: i64,
    config: &'a NormalOverworldNoiseConfig,
    diagnostics: &'a SurfaceWaterDiagnosticContext,
}

#[derive(Default)]
struct SurfaceWaterCounts {
    pair_counts: BTreeMap<(String, String), usize>,
    printed: usize,
}

#[derive(Default)]
struct HeightDeltaTotals {
    world_surface: usize,
    ocean_floor: usize,
    motion_blocking: usize,
    columns: usize,
}

#[derive(Default)]
struct HeightDeltaChunk {
    world_surface: usize,
    ocean_floor: usize,
    motion_blocking: usize,
    examples: Vec<String>,
}

struct TreeBlockDeltaColumnInput<'a> {
    pos: ChunkPos,
    local_x: usize,
    local_z: usize,
    surface_chunk: &'a LevelChunk,
    lightweight: &'a super::super::LightweightTreeContextChunk,
    config: &'a NormalOverworldNoiseConfig,
}

#[test]
#[ignore = "diagnostic for surface/water column mismatches against the vanilla fixture"]
fn normal_overworld_surface_water_column_diagnostic() {
    let fixture = load_vanilla_block_fixture();
    let seed = fixture_seed(&fixture);
    let fixture_chunks = fixture_chunks(&fixture);
    let config = normal_overworld_noise_config();
    let diagnostics = SurfaceWaterDiagnosticContext::new(seed, config.noise_settings);
    let mut counts = SurfaceWaterCounts::default();

    for fixture_chunk in fixture_chunks {
        diagnose_surface_water_fixture_chunk(
            fixture_chunk,
            seed,
            &config,
            &diagnostics,
            &mut counts,
        );
    }

    eprintln!("[worldgen-surface-water-summary] top expected->actual mismatches in y=48..80");
    let mut sorted_pairs = counts.pair_counts.into_iter().collect::<Vec<_>>();
    sorted_pairs.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((expected, actual), count) in sorted_pairs.into_iter().take(20) {
        eprintln!(
            "[worldgen-surface-water-pair] count={count} expected={expected} actual={actual}"
        );
    }
}

#[test]
#[ignore = "diagnostic for lightweight tree context heightmap drift after surface building"]
fn normal_overworld_tree_context_height_delta_diagnostic() {
    let fixture = load_vanilla_block_fixture();
    let seed = fixture_seed(&fixture);
    let fixture_chunks = fixture_chunks(&fixture);
    let config = normal_overworld_noise_config();
    let router = normal_overworld_noise_router(config.noise_settings);
    let mut totals = HeightDeltaTotals::default();

    for fixture_chunk in fixture_chunks {
        diagnose_tree_height_delta_chunk(fixture_chunk, seed, &config, router, &mut totals);
    }

    eprintln!(
        "[tree-context-height-delta-summary] chunks={} columns={} world_surface={} ocean_floor={} motion_blocking={}",
        fixture_chunks.len(),
        totals.columns,
        totals.world_surface,
        totals.ocean_floor,
        totals.motion_blocking
    );
}

#[test]
#[ignore = "diagnostic for lightweight tree context synthetic block drift"]
fn normal_overworld_tree_context_block_delta_diagnostic() {
    let fixture = load_vanilla_block_fixture();
    let seed = fixture_seed(&fixture);
    let fixture_chunks = fixture_chunks(&fixture);
    let config = normal_overworld_noise_config();
    let router = normal_overworld_noise_router(config.noise_settings);
    let mut total_counts = BTreeMap::<(String, String), usize>::new();

    for fixture_chunk in fixture_chunks {
        diagnose_tree_block_delta_chunk(fixture_chunk, seed, &config, router, &mut total_counts);
    }

    eprintln!("[tree-context-block-delta-summary] top live->synthetic mismatches");
    let mut sorted = total_counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((live, synthetic), count) in sorted.into_iter().take(20) {
        eprintln!(
            "[tree-context-block-delta-pair] count={count} live={live} synthetic={synthetic}"
        );
    }
}

impl SurfaceWaterDiagnosticContext {
    fn new(seed: i64, settings: &super::super::NoiseGeneratorSettings) -> Self {
        let noise_router = normal_overworld_noise_router(settings);
        let climate_sampler =
            super::super::ClimateSampler::from_noise_router(&noise_router, seed, *settings);
        let surface_noise =
            super::super::random_state_normal_noise_snapshot(seed, *settings, "minecraft:surface");
        let algorithm = random_algorithm(settings);
        let base_rng = crate::random_source::random_state_seed_factories(seed, algorithm).base;

        Self {
            noise_router,
            climate_sampler,
            biome_zoom_seed: super::super::biome_manager_obfuscate_seed(seed),
            surface_noise,
            base_rng,
        }
    }
}

fn load_vanilla_block_fixture() -> serde_json::Value {
    let fixture_json = include_str!(
        "../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    );
    serde_json::from_str(fixture_json).expect("vanilla fixture should parse")
}

fn fixture_seed(fixture: &serde_json::Value) -> i64 {
    fixture
        .get("seed")
        .and_then(serde_json::Value::as_str)
        .expect("vanilla fixture should include a seed")
        .parse::<i64>()
        .expect("vanilla fixture seed should parse")
}

fn fixture_chunks(fixture: &serde_json::Value) -> &[serde_json::Value] {
    fixture
        .get("chunks")
        .and_then(serde_json::Value::as_array)
        .expect("fixture should include chunks")
}

fn normal_overworld_noise_config() -> NormalOverworldNoiseConfig {
    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };

    NormalOverworldNoiseConfig {
        biome_source_model: biome_source_model.clone(),
        noise_settings,
    }
}

fn normal_overworld_noise_router(
    settings: &super::super::NoiseGeneratorSettings,
) -> super::super::NoiseRouter {
    super::super::builtin_noise_router(super::super::noise_router_id_for_settings(*settings))
        .expect("normal overworld should have a built-in noise router")
        .router
}

fn random_algorithm(
    settings: &super::super::NoiseGeneratorSettings,
) -> crate::random_source::RandomAlgorithm {
    if settings.legacy_random_source {
        crate::random_source::RandomAlgorithm::Legacy
    } else {
        crate::random_source::RandomAlgorithm::Xoroshiro
    }
}

fn fixture_block_type(block: &serde_json::Value) -> &str {
    block
        .as_str()
        .and_then(|raw| {
            raw.split_once('[')
                .map_or(Some(raw), |(name, _)| Some(name))
        })
        .expect("fixture block should be a string")
}

fn is_surface_water_material(block: &str) -> bool {
    matches!(
        block,
        "minecraft:water"
            | "minecraft:sand"
            | "minecraft:dirt"
            | "minecraft:grass_block"
            | "minecraft:stone"
            | "minecraft:deepslate"
            | "minecraft:gravel"
            | "minecraft:air"
    )
}

fn fixture_chunk_pos(fixture_chunk: &serde_json::Value) -> ChunkPos {
    ChunkPos {
        x: fixture_chunk
            .get("chunkX")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkX") as i32,
        z: fixture_chunk
            .get("chunkZ")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include chunkZ") as i32,
    }
}

fn fixture_x_columns(fixture_chunk: &serde_json::Value) -> &[serde_json::Value] {
    fixture_chunk
        .get("blocks")
        .and_then(serde_json::Value::as_array)
        .expect("fixture chunk should include blocks")
}

fn fixture_y_min(fixture_chunk: &serde_json::Value) -> i32 {
    fixture_chunk
        .get("yMin")
        .and_then(serde_json::Value::as_i64)
        .expect("fixture chunk should include yMin") as i32
}

fn fixture_y_max(fixture_chunk: &serde_json::Value, y_min: i32) -> i32 {
    y_min
        + fixture_x_columns(fixture_chunk)
            .first()
            .and_then(serde_json::Value::as_array)
            .expect("fixture chunk should include x/y block arrays")
            .len() as i32
}

fn generate_surface_and_ore_chunks(
    pos: ChunkPos,
    seed: i64,
    config: &NormalOverworldNoiseConfig,
) -> (LevelChunk, LevelChunk) {
    let (base, _, _) = super::super::generate_real_surface_base_chunk(
        pos,
        &config.biome_source_model,
        config.noise_settings,
        seed,
    )
    .expect("real-surface base generation should succeed");
    let mut carver = base.clone();
    super::super::apply_configured_carvers_for_biome_source(
        &mut carver,
        &config.biome_source_model,
        config.noise_settings,
        seed,
    );
    let mut ore = carver.clone();
    super::super::apply_mineshaft_underground_structures_to_chunk(&mut ore, seed);
    super::super::apply_underground_ore_decoration_to_chunk(
        &mut ore,
        &config.biome_source_model,
        config.noise_settings,
        seed,
        None,
    );
    (base, ore)
}

fn diagnose_surface_water_fixture_chunk(
    fixture_chunk: &serde_json::Value,
    seed: i64,
    config: &NormalOverworldNoiseConfig,
    diagnostics: &SurfaceWaterDiagnosticContext,
    counts: &mut SurfaceWaterCounts,
) {
    let pos = fixture_chunk_pos(fixture_chunk);
    let y_min = fixture_y_min(fixture_chunk);
    let y_max = fixture_y_max(fixture_chunk, y_min);
    let x_columns = fixture_x_columns(fixture_chunk);
    let (base, ore) = generate_surface_and_ore_chunks(pos, seed, config);

    for (local_x, y_column_value) in x_columns.iter().enumerate() {
        let y_column = y_column_value
            .as_array()
            .expect("fixture x column should include y array");
        diagnose_surface_water_y_column(
            SurfaceWaterColumnInput {
                pos,
                local_x,
                y_min,
                y_max,
                y_column,
                base: &base,
                ore: &ore,
                seed,
                config,
                diagnostics,
            },
            counts,
        );
    }
}

struct SurfaceWaterColumnInput<'a> {
    pos: ChunkPos,
    local_x: usize,
    y_min: i32,
    y_max: i32,
    y_column: &'a [serde_json::Value],
    base: &'a LevelChunk,
    ore: &'a LevelChunk,
    seed: i64,
    config: &'a NormalOverworldNoiseConfig,
    diagnostics: &'a SurfaceWaterDiagnosticContext,
}

fn diagnose_surface_water_y_column(
    input: SurfaceWaterColumnInput<'_>,
    counts: &mut SurfaceWaterCounts,
) {
    for (y_offset, z_column_value) in input.y_column.iter().enumerate() {
        let world_y = input.y_min + y_offset as i32;
        if !(48..=80).contains(&world_y) {
            continue;
        }
        let z_column = z_column_value
            .as_array()
            .expect("fixture y column should include z array");
        for (local_z, expected_value) in z_column.iter().enumerate() {
            let expected = fixture_block_type(expected_value);
            let world_x = input.pos.x * 16 + input.local_x as i32;
            let world_z = input.pos.z * 16 + local_z as i32;
            let actual = input
                .ore
                .get_block_state(world_x, world_y, world_z)
                .unwrap_or_else(|| "minecraft:air".to_string());
            if surface_water_mismatch_should_be_skipped(expected, actual.as_str()) {
                continue;
            }
            *counts
                .pair_counts
                .entry((expected.to_string(), actual.clone()))
                .or_default() += 1;
            if counts.printed >= 16 {
                continue;
            }
            report_surface_water_mismatch(&SurfaceWaterMismatch {
                pos: input.pos,
                local_x: input.local_x,
                local_z,
                world_y,
                y_min: input.y_min,
                y_max: input.y_max,
                y_column: input.y_column,
                expected,
                actual: actual.as_str(),
                base: input.base,
                ore: input.ore,
                seed: input.seed,
                config: input.config,
                diagnostics: input.diagnostics,
            });
            counts.printed += 1;
        }
    }
}

fn surface_water_mismatch_should_be_skipped(expected: &str, actual: &str) -> bool {
    expected == actual
        || (!is_surface_water_material(expected) && !is_surface_water_material(actual))
}

fn report_surface_water_mismatch(sample: &SurfaceWaterMismatch<'_>) {
    let world_x = sample.pos.x * 16 + sample.local_x as i32;
    let world_z = sample.pos.z * 16 + sample.local_z as i32;
    let surface_context = surface_water_rule_context(sample, world_x, world_z);
    let density_context = density_and_aquifer_at(
        sample.pos,
        world_x,
        sample.world_y,
        world_z,
        sample.seed,
        sample.config.noise_settings,
        sample.diagnostics.noise_router,
    );
    let actual_top =
        top_non_air_from_chunk(sample.ore, world_x, world_z, sample.y_min, sample.y_max);
    let expected_top = top_non_air_from_fixture(sample.y_column, sample.y_min, sample.local_z);

    print_surface_water_sample(
        sample,
        world_x,
        world_z,
        surface_context,
        density_context,
        expected_top,
        actual_top,
    );
    eprintln!(
        "[worldgen-surface-water-column] expected {}",
        column_stack_from_fixture(
            sample.y_column,
            sample.y_min,
            sample.local_z,
            sample.world_y - 6,
            sample.world_y + 8
        )
    );
    eprintln!(
        "[worldgen-surface-water-column] actual   {}",
        column_stack_from_chunk(
            sample.ore,
            world_x,
            world_z,
            sample.world_y - 6,
            sample.world_y + 8
        )
    );
}

fn surface_water_rule_context(
    sample: &SurfaceWaterMismatch<'_>,
    world_x: i32,
    world_z: i32,
) -> (i32, &'static str, i32, &'static str, i32, f64, i32) {
    let start_height =
        super::super::read_world_surface_wg(sample.base, sample.local_x, sample.local_z) + 1;
    let biome_y = if sample.config.noise_settings.legacy_random_source {
        0
    } else {
        start_height
    };
    let biome = surface_water_biome(sample, world_x, biome_y, world_z);
    let rule_biome_y = if sample.config.noise_settings.legacy_random_source {
        0
    } else {
        sample.world_y
    };
    let rule_biome = surface_water_biome(sample, world_x, rule_biome_y, world_z);
    let surface_noise_value = surface_water_noise_value(sample, world_x, world_z);
    let surface_depth = surface_water_depth(sample, world_x, world_z, surface_noise_value);
    (
        start_height,
        biome,
        biome_y,
        rule_biome,
        rule_biome_y,
        surface_noise_value,
        surface_depth,
    )
}

fn surface_water_biome(
    sample: &SurfaceWaterMismatch<'_>,
    world_x: i32,
    biome_y: i32,
    world_z: i32,
) -> &'static str {
    super::super::biome_manager_get_biome(
        &sample.config.biome_source_model,
        sample.diagnostics.biome_zoom_seed,
        world_x,
        biome_y,
        world_z,
        &sample.diagnostics.climate_sampler,
    )
    .unwrap_or("minecraft:plains")
}

fn surface_water_noise_value(sample: &SurfaceWaterMismatch<'_>, world_x: i32, world_z: i32) -> f64 {
    sample
        .diagnostics
        .surface_noise
        .as_ref()
        .map(|snap| {
            super::super::normal_noise_sample(snap, f64::from(world_x), 0.0, f64::from(world_z))
        })
        .unwrap_or(0.0)
}

fn surface_water_depth(
    sample: &SurfaceWaterMismatch<'_>,
    world_x: i32,
    world_z: i32,
    surface_noise_value: f64,
) -> i32 {
    let mut at_rng = sample.diagnostics.base_rng.at(world_x, 0, world_z);
    let jitter = super::super::random_next_f64(&mut at_rng) * 0.25;
    (surface_noise_value * 2.75 + 3.0 + jitter) as i32
}

fn print_surface_water_sample(
    sample: &SurfaceWaterMismatch<'_>,
    world_x: i32,
    world_z: i32,
    surface_context: (i32, &'static str, i32, &'static str, i32, f64, i32),
    density_context: (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        Option<&'static str>,
    ),
    expected_top: i32,
    actual_top: i32,
) {
    let (
        start_height,
        biome,
        biome_y,
        rule_biome,
        rule_biome_y,
        surface_noise_value,
        surface_depth,
    ) = surface_context;
    let (
        density,
        direct_density,
        direct_base3d,
        direct_sloped,
        direct_depth,
        direct_continents,
        direct_erosion,
        direct_ridges,
        direct_prelim,
        aquifer_state,
    ) = density_context;
    let chunk_x = sample.pos.x;
    let chunk_z = sample.pos.z;
    let local_x = sample.local_x;
    let local_z = sample.local_z;
    let world_y = sample.world_y;
    let expected = sample.expected;
    let actual = sample.actual;

    eprintln!(
        "[worldgen-surface-water-sample] chunk=({chunk_x},{chunk_z}) local=({local_x},{local_z}) world=({world_x},{world_z}) y={world_y} expected={expected} actual={actual} expected_top={expected_top} actual_top={actual_top} base_world_surface_wg={start_height} surface_biome={biome}@{biome_y} rule_biome={rule_biome}@{rule_biome_y} surface_noise={surface_noise_value:.6} surface_depth={surface_depth} density={density:.6} direct_density={direct_density:.6} base3d={direct_base3d:.6} sloped={direct_sloped:.6} depth={direct_depth:.6} continents={direct_continents:.6} erosion={direct_erosion:.6} ridges={direct_ridges:.6} prelim={direct_prelim:.6} aquifer={aquifer_state:?}"
    );
}

fn top_non_air_from_chunk(
    chunk: &LevelChunk,
    world_x: i32,
    world_z: i32,
    y_min: i32,
    y_max: i32,
) -> i32 {
    (y_min..y_max)
        .rev()
        .find(|y| {
            chunk
                .get_block_state(world_x, *y, world_z)
                .is_some_and(|block| block != "minecraft:air")
        })
        .map_or(y_min, |y| y + 1)
}

fn top_non_air_from_fixture(y_column: &[serde_json::Value], y_min: i32, local_z: usize) -> i32 {
    (0..y_column.len())
        .rev()
        .find(|offset| {
            y_column[*offset]
                .as_array()
                .and_then(|z_column| z_column.get(local_z))
                .map(fixture_block_type)
                .is_some_and(|block| block != "minecraft:air")
        })
        .map_or(y_min, |offset| y_min + offset as i32 + 1)
}

fn column_stack_from_fixture(
    y_column: &[serde_json::Value],
    y_min: i32,
    local_z: usize,
    y0: i32,
    y1: i32,
) -> String {
    (y0..=y1)
        .map(|y| {
            let block = y_column
                .get((y - y_min) as usize)
                .and_then(serde_json::Value::as_array)
                .and_then(|z_column| z_column.get(local_z))
                .map(fixture_block_type)
                .unwrap_or("minecraft:air");
            format!("{y}:{block}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn column_stack_from_chunk(
    chunk: &LevelChunk,
    world_x: i32,
    world_z: i32,
    y0: i32,
    y1: i32,
) -> String {
    (y0..=y1)
        .map(|y| {
            let block = chunk
                .get_block_state(world_x, y, world_z)
                .unwrap_or_else(|| "minecraft:air".to_string());
            format!("{y}:{block}")
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn density_and_aquifer_at(
    chunk_pos: ChunkPos,
    x: i32,
    y: i32,
    z: i32,
    seed: i64,
    settings: &super::super::NoiseGeneratorSettings,
    noise_router: super::super::NoiseRouter,
) -> (
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    Option<&'static str>,
) {
    let chunk_min_x = chunk_pos.x * 16;
    let chunk_min_z = chunk_pos.z * 16;
    let mut noise_chunk =
        super::super::NoiseChunk::new(chunk_min_x, chunk_min_z, *settings, seed, noise_router);
    let factories =
        crate::random_source::random_state_seed_factories(seed, random_algorithm(settings));
    let mut aquifer = settings.aquifers_enabled.then(|| {
        super::super::NoiseBasedAquifer::new(
            &mut noise_chunk,
            super::super::NoiseBasedAquiferBounds {
                chunk_min_x,
                chunk_max_x: chunk_min_x + 15,
                chunk_min_z,
                chunk_max_z: chunk_min_z + 15,
                min_block_y: settings.noise.min_y,
                y_block_size: settings.noise.height,
            },
            seed,
            *settings,
            noise_router,
            factories.aquifer,
        )
    });

    prepare_noise_chunk_for_block(
        &mut noise_chunk,
        settings,
        chunk_min_x,
        chunk_min_z,
        x,
        y,
        z,
    );
    let interpolated_density = noise_chunk.interpolated_density(x, y, z);
    let aquifer_state = aquifer
        .as_mut()
        .and_then(|aquifer| aquifer.compute_substance(&noise_chunk, x, y, z, interpolated_density));
    (
        interpolated_density,
        noise_router
            .final_density
            .compute_with_noise(seed, *settings, x, y, z),
        super::super::BASE_3D_NOISE_OVERWORLD_DENSITY.compute_with_noise(seed, *settings, x, y, z),
        super::super::OVERWORLD_SLOPED_CHEESE_DENSITY.compute_with_noise(seed, *settings, x, y, z),
        noise_router
            .depth
            .compute_with_noise(seed, *settings, x, y, z),
        noise_router
            .continents
            .compute_with_noise(seed, *settings, x, y, z),
        noise_router
            .erosion
            .compute_with_noise(seed, *settings, x, y, z),
        noise_router
            .ridges
            .compute_with_noise(seed, *settings, x, y, z),
        noise_router
            .preliminary_surface_level
            .compute_with_noise(seed, *settings, x, y, z),
        aquifer_state,
    )
}

fn prepare_noise_chunk_for_block(
    noise_chunk: &mut super::super::NoiseChunk,
    settings: &super::super::NoiseGeneratorSettings,
    chunk_min_x: i32,
    chunk_min_z: i32,
    x: i32,
    y: i32,
    z: i32,
) {
    let cell_width = settings.noise.cell_width();
    let cell_height = settings.noise.cell_height();
    let cell_x_index = (x - chunk_min_x).div_euclid(cell_width);
    let cell_z_index = (z - chunk_min_z).div_euclid(cell_width);
    let cell_y_index = y.div_euclid(cell_height) - noise_chunk.cell_noise_min_y;
    noise_chunk.advance_cell_x(cell_x_index);
    noise_chunk.select_cell_yz(cell_y_index, cell_z_index);
    noise_chunk.update_for_y(
        y,
        f64::from(y.rem_euclid(cell_height)) / f64::from(cell_height),
    );
    noise_chunk.update_for_x(
        x,
        f64::from((x - chunk_min_x).rem_euclid(cell_width)) / f64::from(cell_width),
    );
    noise_chunk.update_for_z(
        z,
        f64::from((z - chunk_min_z).rem_euclid(cell_width)) / f64::from(cell_width),
    );
}

fn diagnose_tree_height_delta_chunk(
    fixture_chunk: &serde_json::Value,
    seed: i64,
    config: &NormalOverworldNoiseConfig,
    router: super::super::NoiseRouter,
    totals: &mut HeightDeltaTotals,
) {
    let pos = fixture_chunk_pos(fixture_chunk);
    let (surface_chunk, _, _) = super::super::generate_real_surface_base_chunk(
        pos,
        &config.biome_source_model,
        config.noise_settings,
        seed,
    )
    .expect("real-surface base generation should succeed");
    let live_heights =
        super::super::tree_decoration_terrain_heights(&surface_chunk, config.noise_settings);
    let lightweight_heights =
        super::super::noise_tree_context_heights(pos, config.noise_settings, seed, router);
    let chunk_delta = compare_tree_height_contexts(pos, &live_heights, &lightweight_heights);

    totals.world_surface += chunk_delta.world_surface;
    totals.ocean_floor += chunk_delta.ocean_floor;
    totals.motion_blocking += chunk_delta.motion_blocking;
    totals.columns += 16 * 16;
    eprintln!(
        "[tree-context-height-delta] chunk=({},{}) world_surface={} ocean_floor={} motion_blocking={} examples={}",
        pos.x,
        pos.z,
        chunk_delta.world_surface,
        chunk_delta.ocean_floor,
        chunk_delta.motion_blocking,
        chunk_delta.examples.join(" | ")
    );
}

fn compare_tree_height_contexts(
    pos: ChunkPos,
    live_heights: &super::super::TreeDecorationHeights,
    lightweight_heights: &super::super::TreeDecorationHeights,
) -> HeightDeltaChunk {
    let mut chunk_delta = HeightDeltaChunk::default();
    for local_z in 0..16_usize {
        for local_x in 0..16_usize {
            let index = local_z * 16 + local_x;
            let live_world_surface = live_heights.world_surface[index];
            let lightweight_world_surface = lightweight_heights.world_surface[index];
            let live_ocean_floor = live_heights.ocean_floor[index];
            let lightweight_ocean_floor = lightweight_heights.ocean_floor[index];
            let live_motion_blocking = live_heights.motion_blocking[index];
            let lightweight_motion_blocking = lightweight_heights.motion_blocking[index];
            if live_world_surface == lightweight_world_surface
                && live_ocean_floor == lightweight_ocean_floor
                && live_motion_blocking == lightweight_motion_blocking
            {
                continue;
            }
            record_tree_height_delta(
                &mut chunk_delta,
                pos,
                local_x,
                local_z,
                (
                    live_world_surface,
                    lightweight_world_surface,
                    live_ocean_floor,
                    lightweight_ocean_floor,
                    live_motion_blocking,
                    lightweight_motion_blocking,
                ),
            );
        }
    }
    chunk_delta
}

fn record_tree_height_delta(
    chunk_delta: &mut HeightDeltaChunk,
    pos: ChunkPos,
    local_x: usize,
    local_z: usize,
    heights: (i32, i32, i32, i32, i32, i32),
) {
    let (
        live_world_surface,
        lightweight_world_surface,
        live_ocean_floor,
        lightweight_ocean_floor,
        live_motion_blocking,
        lightweight_motion_blocking,
    ) = heights;
    chunk_delta.world_surface += usize::from(live_world_surface != lightweight_world_surface);
    chunk_delta.ocean_floor += usize::from(live_ocean_floor != lightweight_ocean_floor);
    chunk_delta.motion_blocking += usize::from(live_motion_blocking != lightweight_motion_blocking);
    if chunk_delta.examples.len() < 8 {
        chunk_delta.examples.push(format!(
            "local=({local_x},{local_z}) world=({},{}) live_ws={} light_ws={} live_of={} light_of={} live_mb={} light_mb={}",
            pos.x * 16 + local_x as i32,
            pos.z * 16 + local_z as i32,
            live_world_surface,
            lightweight_world_surface,
            live_ocean_floor,
            lightweight_ocean_floor,
            live_motion_blocking,
            lightweight_motion_blocking
        ));
    }
}

fn diagnose_tree_block_delta_chunk(
    fixture_chunk: &serde_json::Value,
    seed: i64,
    config: &NormalOverworldNoiseConfig,
    router: super::super::NoiseRouter,
    total_counts: &mut BTreeMap<(String, String), usize>,
) {
    let pos = fixture_chunk_pos(fixture_chunk);
    let (surface_chunk, _, _) = super::super::generate_real_surface_base_chunk(
        pos,
        &config.biome_source_model,
        config.noise_settings,
        seed,
    )
    .expect("real-surface base generation should succeed");
    let lightweight = super::super::LightweightTreeContextChunk {
        terrain_heights: super::super::noise_tree_context_heights(
            pos,
            config.noise_settings,
            seed,
            router,
        ),
        min_y: config.noise_settings.noise.min_y,
        max_y: config.noise_settings.noise.min_y + config.noise_settings.noise.height,
    };
    let (chunk_counts, examples) =
        collect_tree_block_delta_pairs(pos, &surface_chunk, &lightweight, config, total_counts);

    let mut sorted = chunk_counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    let summary = sorted
        .into_iter()
        .take(8)
        .map(|((live, synthetic), count)| format!("{count}:{live}->{synthetic}"))
        .collect::<Vec<_>>()
        .join(", ");
    eprintln!(
        "[tree-context-block-delta] chunk=({},{}) top_pairs=[{}] examples={}",
        pos.x,
        pos.z,
        summary,
        examples.join(" | ")
    );
}

fn collect_tree_block_delta_pairs(
    pos: ChunkPos,
    surface_chunk: &LevelChunk,
    lightweight: &super::super::LightweightTreeContextChunk,
    config: &NormalOverworldNoiseConfig,
    total_counts: &mut BTreeMap<(String, String), usize>,
) -> (BTreeMap<(String, String), usize>, Vec<String>) {
    let mut chunk_counts = BTreeMap::<(String, String), usize>::new();
    let mut examples = Vec::new();
    for local_z in 0..16_usize {
        for local_x in 0..16_usize {
            collect_tree_block_delta_column(
                TreeBlockDeltaColumnInput {
                    pos,
                    local_x,
                    local_z,
                    surface_chunk,
                    lightweight,
                    config,
                },
                &mut chunk_counts,
                total_counts,
                &mut examples,
            );
        }
    }
    (chunk_counts, examples)
}

fn collect_tree_block_delta_column(
    input: TreeBlockDeltaColumnInput<'_>,
    chunk_counts: &mut BTreeMap<(String, String), usize>,
    total_counts: &mut BTreeMap<(String, String), usize>,
    examples: &mut Vec<String>,
) {
    let index = input.local_z * 16 + input.local_x;
    let ocean_floor = input.lightweight.terrain_heights.ocean_floor[index];
    let world_surface = input.lightweight.terrain_heights.world_surface[index];
    let min_y = (ocean_floor.min(world_surface) - 6).max(input.config.noise_settings.noise.min_y);
    let max_y = (world_surface + 8).min(
        input.config.noise_settings.noise.min_y + input.config.noise_settings.noise.height - 1,
    );
    let world_x = input.pos.x * 16 + input.local_x as i32;
    let world_z = input.pos.z * 16 + input.local_z as i32;

    for y in min_y..=max_y {
        let live = input
            .surface_chunk
            .get_block_state_name(world_x, y, world_z)
            .unwrap_or("minecraft:air");
        let synthetic = input.lightweight.synthetic_block_state(world_x, y, world_z);
        if live == synthetic {
            continue;
        }
        *chunk_counts
            .entry((live.to_string(), synthetic.to_string()))
            .or_default() += 1;
        *total_counts
            .entry((live.to_string(), synthetic.to_string()))
            .or_default() += 1;
        if examples.len() < 8 {
            examples.push(format!(
                "world=({world_x},{y},{world_z}) live={live} synthetic={synthetic} of={ocean_floor} ws={world_surface}"
            ));
        }
    }
}
