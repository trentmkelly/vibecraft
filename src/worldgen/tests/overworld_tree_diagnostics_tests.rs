use super::*;

const VANILLA_BLOCK_ARRAY_FIXTURE_JSON: &str =
    include_str!("../../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
const TREE_SHAPE_TRACKED_BLOCKS: &[&str] = &[
    "minecraft:oak_leaves",
    "minecraft:birch_leaves",
    "minecraft:oak_log",
    "minecraft:birch_log",
    "minecraft:leaf_litter",
    "minecraft:air",
];

fn vanilla_fixture() -> serde_json::Value {
    serde_json::from_str(VANILLA_BLOCK_ARRAY_FIXTURE_JSON).expect("vanilla fixture should parse")
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
        .expect("vanilla fixture should include chunks")
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

fn fixture_chunk_y_min(fixture_chunk: &serde_json::Value) -> i32 {
    fixture_chunk
        .get("yMin")
        .and_then(serde_json::Value::as_i64)
        .expect("fixture chunk should include yMin") as i32
}

fn fixture_chunk_blocks(fixture_chunk: &serde_json::Value) -> &[serde_json::Value] {
    fixture_chunk
        .get("blocks")
        .and_then(serde_json::Value::as_array)
        .expect("fixture chunk should include blocks")
}

fn fixture_block_id(block: &serde_json::Value) -> &str {
    let block = block.as_str().expect("fixture block should be a string");
    block.split_once('[').map_or(block, |(id, _)| id)
}

fn normal_overworld_noise_generator(
    preset: &super::super::ResolvedWorldPreset,
) -> (
    &crate::biome::BiomeSourceModel,
    &super::super::NoiseGeneratorSettings,
) {
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = &preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    (biome_source_model, noise_settings)
}

fn normal_overworld_generator() -> (
    super::super::ResolvedWorldPreset,
    crate::biome::BiomeSourceModel,
    &'static super::super::NoiseGeneratorSettings,
) {
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
    (preset.clone(), biome_source_model.clone(), noise_settings)
}

fn generate_chunk_through_ores(
    pos: ChunkPos,
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> LevelChunk {
    let (base, _, _) = super::super::generate_real_surface_base_chunk(
        pos,
        biome_source_model,
        noise_settings,
        seed,
    )
    .expect("real-surface base generation should succeed");
    let mut chunk = base;
    super::super::apply_configured_carvers_for_biome_source(
        &mut chunk,
        biome_source_model,
        noise_settings,
        seed,
    );
    super::super::apply_mineshaft_underground_structures_to_chunk(&mut chunk, seed);
    super::super::apply_underground_ore_decoration_to_chunk(
        &mut chunk,
        biome_source_model,
        noise_settings,
        seed,
        None,
    );
    chunk
}

fn generate_chunk_through_trees(
    pos: ChunkPos,
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> LevelChunk {
    let mut chunk = generate_chunk_through_ores(pos, biome_source_model, noise_settings, seed);
    super::super::apply_initial_tree_decoration_to_chunk(
        &mut chunk,
        biome_source_model,
        noise_settings,
        seed,
        None,
        None,
        None,
    );
    chunk
}

fn generate_fixture_chunks_through_trees(
    fixture_chunks: &[serde_json::Value],
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> Vec<LevelChunk> {
    fixture_chunks
        .iter()
        .map(|chunk| {
            generate_chunk_through_trees(
                fixture_chunk_pos(chunk),
                biome_source_model,
                noise_settings,
                seed,
            )
        })
        .collect()
}

struct StageParityChunks {
    base: Vec<LevelChunk>,
    carver: Vec<LevelChunk>,
    structure: Vec<LevelChunk>,
    ore: Vec<LevelChunk>,
    tree: Vec<LevelChunk>,
}

fn generate_stage_parity_chunks(
    fixture_chunks: &[serde_json::Value],
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> StageParityChunks {
    let mut base_chunks = Vec::new();
    let mut carver_chunks = Vec::new();
    let mut structure_chunks = Vec::new();
    let mut ore_chunks = Vec::new();
    let mut tree_chunks = Vec::new();

    for fixture_chunk in fixture_chunks {
        let pos = fixture_chunk_pos(fixture_chunk);
        let (base, _, _) = super::super::generate_real_surface_base_chunk(
            pos,
            biome_source_model,
            noise_settings,
            seed,
        )
        .expect("real-surface base generation should succeed");
        let mut carver = base.clone();
        super::super::apply_configured_carvers_for_biome_source(
            &mut carver,
            biome_source_model,
            noise_settings,
            seed,
        );
        let mut structure = carver.clone();
        super::super::apply_mineshaft_underground_structures_to_chunk(&mut structure, seed);
        let mut ore = structure.clone();
        super::super::apply_underground_ore_decoration_to_chunk(
            &mut ore,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );
        let mut tree = ore.clone();
        super::super::apply_initial_tree_decoration_to_chunk(
            &mut tree,
            biome_source_model,
            noise_settings,
            seed,
            None,
            None,
            None,
        );
        base_chunks.push(base);
        carver_chunks.push(carver);
        structure_chunks.push(structure);
        ore_chunks.push(ore);
        tree_chunks.push(tree);
    }

    StageParityChunks {
        base: base_chunks,
        carver: carver_chunks,
        structure: structure_chunks,
        ore: ore_chunks,
        tree: tree_chunks,
    }
}

fn print_stage_parity(label: &str, chunks: &[LevelChunk]) {
    let block = crate::worldgen_comparison::vanilla_worldgen_block_array_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        chunks,
    )
    .expect("block parity score should compute");
    let heightmap = crate::worldgen_comparison::vanilla_worldgen_heightmap_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        chunks,
    )
    .expect("heightmap parity score should compute");
    let column = crate::worldgen_comparison::vanilla_worldgen_column_profile_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        chunks,
    )
    .expect("column-profile parity score should compute");
    eprintln!(
        "[worldgen-stage-parity] stage={} block={:.6} ({}/{}) heightmap={:.6} ({}/{}) column={:.6} ({}/{})",
        label,
        block.score,
        block.matching_blocks,
        block.total_blocks,
        heightmap.score,
        heightmap.matching_columns,
        heightmap.total_columns,
        column.score,
        column.matching_columns,
        column.total_columns
    );
    for mismatch in block.mismatches.iter().take(8) {
        eprintln!(
            "[worldgen-stage-parity-mismatch] stage={} count={} expected={} actual={}",
            label, mismatch.count, mismatch.expected, mismatch.actual
        );
    }
}

fn collect_decoration_region_biomes(
    source_pos: ChunkPos,
    sample_quart_y: i32,
    biome_source_model: &crate::biome::BiomeSourceModel,
    climate_sampler: &super::super::ClimateSampler,
) -> std::collections::BTreeSet<&'static str> {
    let mut biomes = std::collections::BTreeSet::new();
    for chunk_z in source_pos.z - 1..=source_pos.z + 1 {
        for chunk_x in source_pos.x - 1..=source_pos.x + 1 {
            let chunk_quart_x = chunk_x * 4;
            let chunk_quart_z = chunk_z * 4;
            for local_z in 0..4 {
                for local_x in 0..4 {
                    if let Some(biome) = super::super::get_biome(
                        biome_source_model,
                        chunk_quart_x + local_x,
                        sample_quart_y,
                        chunk_quart_z + local_z,
                        climate_sampler,
                    ) {
                        biomes.insert(biome);
                    }
                }
            }
        }
    }
    biomes
}

fn feature_names_for_step(
    plan: &super::super::BiomeDecorationFeaturePlan,
    step: GenerationDecorationStep,
) -> Vec<&'static str> {
    plan.feature_calls
        .iter()
        .filter(|call| call.step_index == step as usize)
        .map(|call| call.feature)
        .collect()
}

fn print_source_feature_schedule(
    target_pos: ChunkPos,
    source_pos: ChunkPos,
    setup: &DecorationScheduleSetup<'_>,
) {
    let biomes = collect_decoration_region_biomes(
        source_pos,
        setup.sample_quart_y,
        setup.biome_source_model,
        setup.climate_sampler,
    );
    let possible_steps = super::super::possible_biome_feature_steps_for_decoration_region(
        source_pos,
        setup.biome_source_model,
        setup.noise_settings,
        setup.climate_sampler,
    );
    let plan = super::super::biome_decoration_feature_plan(
        setup.seed,
        source_pos.x,
        source_pos.z,
        setup.min_section_y,
        setup.features_per_step,
        &possible_steps,
    );
    let ore_features = feature_names_for_step(&plan, GenerationDecorationStep::UndergroundOres);
    let vegetal_features =
        feature_names_for_step(&plan, GenerationDecorationStep::VegetalDecoration);
    eprintln!(
        "[decoration-source-schedule] target=({}, {}) source=({}, {}) biomes=[{}] ores=[{}] vegetal=[{}]",
        target_pos.x,
        target_pos.z,
        source_pos.x,
        source_pos.z,
        biomes.into_iter().collect::<Vec<_>>().join(","),
        ore_features.join(","),
        vegetal_features.join(","),
    );
}

struct DecorationScheduleSetup<'a> {
    biome_source_model: &'a crate::biome::BiomeSourceModel,
    noise_settings: &'a super::super::NoiseGeneratorSettings,
    climate_sampler: &'a super::super::ClimateSampler,
    features_per_step: &'a [super::super::StepFeatureDataModel],
    min_section_y: i32,
    sample_quart_y: i32,
    seed: i64,
}

fn decoration_schedule_setup<'a>(
    biome_source_model: &'a crate::biome::BiomeSourceModel,
    noise_settings: &'a super::super::NoiseGeneratorSettings,
    seed: i64,
) -> (
    super::super::ClimateSampler,
    Vec<super::super::StepFeatureDataModel>,
    i32,
    i32,
) {
    let router = super::super::builtin_noise_router(super::super::noise_router_id_for_settings(
        *noise_settings,
    ))
    .expect("normal overworld should have a built-in noise router")
    .router;
    let climate_sampler =
        super::super::ClimateSampler::from_noise_router(&router, seed, *noise_settings);
    let global_biome_steps =
        super::super::possible_biome_feature_steps_for_source(biome_source_model);
    let features_per_step = super::super::build_features_per_step(&global_biome_steps, true)
        .expect("global features should sort");
    let min_section_y = noise_settings.noise.min_y.div_euclid(16);
    let sample_quart_y = ((noise_settings.sea_level + 1).clamp(
        noise_settings.noise.min_y,
        noise_settings.noise.min_y + noise_settings.noise.height - 1,
    )) >> 2;
    (
        climate_sampler,
        features_per_step,
        min_section_y,
        sample_quart_y,
    )
}

fn generate_region_chunks_through_ores(
    fixture_chunks: &[serde_json::Value],
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> (BTreeMap<ChunkPos, LevelChunk>, Vec<ChunkPos>) {
    let mut region_chunks = BTreeMap::<ChunkPos, LevelChunk>::new();
    let mut source_positions = Vec::new();
    for fixture_chunk in fixture_chunks {
        let pos = fixture_chunk_pos(fixture_chunk);
        let chunk = generate_chunk_through_ores(pos, biome_source_model, noise_settings, seed);
        region_chunks.insert(pos, chunk);
        source_positions.push(pos);
    }
    (region_chunks, source_positions)
}

fn apply_trees_to_region_sources(
    region_chunks: &mut BTreeMap<ChunkPos, LevelChunk>,
    source_positions: &[ChunkPos],
    biome_source_model: &crate::biome::BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> usize {
    let mut total_tree_blocks = 0;
    for &pos in source_positions {
        let result = super::super::apply_initial_tree_decoration_from_source_into_region(
            region_chunks,
            pos,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );
        total_tree_blocks += result.placed_blocks;
    }
    total_tree_blocks
}

fn print_region_tree_parity(total_tree_blocks: usize, generated: &[LevelChunk]) {
    let block = crate::worldgen_comparison::vanilla_worldgen_block_array_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        generated,
    )
    .expect("block parity score should compute");
    let heightmap = crate::worldgen_comparison::vanilla_worldgen_heightmap_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        generated,
    )
    .expect("heightmap parity score should compute");
    let column = crate::worldgen_comparison::vanilla_worldgen_column_profile_parity_score(
        VANILLA_BLOCK_ARRAY_FIXTURE_JSON,
        generated,
    )
    .expect("column-profile parity score should compute");
    eprintln!(
        "[worldgen-region-tree-parity] tree_blocks={} block={:.6} ({}/{}) heightmap={:.6} ({}/{}) column={:.6} ({}/{})",
        total_tree_blocks,
        block.score,
        block.matching_blocks,
        block.total_blocks,
        heightmap.score,
        heightmap.matching_columns,
        heightmap.total_columns,
        column.score,
        column.matching_columns,
        column.total_columns
    );
    for mismatch in block.mismatches.iter().take(8) {
        eprintln!(
            "[worldgen-region-tree-parity-mismatch] count={} expected={} actual={}",
            mismatch.count, mismatch.expected, mismatch.actual
        );
    }
}

#[derive(Default)]
struct TreeShapeMismatchStats {
    counts: BTreeMap<(String, String), usize>,
    y_ranges: BTreeMap<(String, String), (i32, i32)>,
    examples: BTreeMap<(String, String), Vec<BlockPos>>,
}

fn record_tree_shape_mismatch(
    stats: &mut TreeShapeMismatchStats,
    expected: &str,
    actual: String,
    pos: BlockPos,
) {
    if expected == actual {
        return;
    }
    if !TREE_SHAPE_TRACKED_BLOCKS.contains(&expected)
        && !TREE_SHAPE_TRACKED_BLOCKS.contains(&actual.as_str())
    {
        return;
    }

    let key = (expected.to_string(), actual);
    *stats.counts.entry(key.clone()).or_insert(0) += 1;
    stats
        .y_ranges
        .entry(key.clone())
        .and_modify(|range| {
            range.0 = range.0.min(pos.y);
            range.1 = range.1.max(pos.y);
        })
        .or_insert((pos.y, pos.y));
    let examples = stats.examples.entry(key).or_default();
    if examples.len() < 8 {
        examples.push(pos);
    }
}

fn scan_tree_shape_mismatches(
    fixture_chunks: &[serde_json::Value],
    generated_chunks: &[LevelChunk],
) -> TreeShapeMismatchStats {
    let mut stats = TreeShapeMismatchStats::default();
    for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
        let chunk_pos = fixture_chunk_pos(fixture_chunk);
        let y_min = fixture_chunk_y_min(fixture_chunk);
        let blocks = fixture_chunk_blocks(fixture_chunk);
        let generated = &generated_chunks[chunk_index];

        for (local_x, y_column) in blocks.iter().enumerate() {
            let y_column = y_column.as_array().expect("x column should be an array");
            for (y_offset, z_column) in y_column.iter().enumerate() {
                let world_y = y_min + y_offset as i32;
                let z_column = z_column.as_array().expect("z column should be an array");
                for (local_z, expected) in z_column.iter().enumerate() {
                    let pos = BlockPos {
                        x: chunk_pos.x * 16 + local_x as i32,
                        y: world_y,
                        z: chunk_pos.z * 16 + local_z as i32,
                    };
                    let actual = generated
                        .get_block_state(pos.x, pos.y, pos.z)
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    record_tree_shape_mismatch(&mut stats, fixture_block_id(expected), actual, pos);
                }
            }
        }
    }
    stats
}

fn print_tree_shape_mismatches(stats: TreeShapeMismatchStats) {
    let mut sorted = stats.counts.into_iter().collect::<Vec<_>>();
    sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for ((expected, actual), count) in sorted.into_iter().take(20) {
        let range_key = (expected.clone(), actual.clone());
        let (min_y, max_y) = stats.y_ranges[&range_key];
        let examples = stats.examples[&range_key]
            .iter()
            .map(|pos| format!("({}, {}, {})", pos.x, pos.y, pos.z))
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!(
            "[tree-shape-diagnostic] count={} expected={} actual={} y={}..{} examples={}",
            count, expected, actual, min_y, max_y, examples
        );
    }
}

#[derive(Default)]
struct FixtureTreeRootSupportStats {
    unsupported_generated_logs: Vec<(i32, i32, i32, String)>,
    missing_expected_base_logs: Vec<(i32, i32, i32, String, String)>,
}

fn record_generated_log_support(
    stats: &mut FixtureTreeRootSupportStats,
    generated: &LevelChunk,
    pos: BlockPos,
    actual: &str,
) {
    if !super::super::block_matches_tag(actual, "minecraft:logs") {
        return;
    }
    let below = generated
        .get_block_state(pos.x, pos.y - 1, pos.z)
        .unwrap_or_else(|| "minecraft:air".to_string());
    let vertical_below = generated
        .get_block_state(pos.x, pos.y - 1, pos.z)
        .is_some_and(|below| super::super::block_matches_tag(&below, "minecraft:logs"));
    if !vertical_below
        && !super::super::block_blocks_motion(&below)
        && stats.unsupported_generated_logs.len() < 16
    {
        stats
            .unsupported_generated_logs
            .push((pos.x, pos.y, pos.z, below));
    }
}

fn record_missing_expected_base_log(
    stats: &mut FixtureTreeRootSupportStats,
    generated: &LevelChunk,
    pos: BlockPos,
    expected: &str,
    actual: &str,
) {
    if !super::super::block_matches_tag(expected, "minecraft:logs") || actual != "minecraft:air" {
        return;
    }
    let above = generated
        .get_block_state(pos.x, pos.y + 1, pos.z)
        .unwrap_or_else(|| "minecraft:air".to_string());
    if super::super::block_matches_tag(&above, "minecraft:logs")
        && stats.missing_expected_base_logs.len() < 16
    {
        stats
            .missing_expected_base_logs
            .push((pos.x, pos.y, pos.z, expected.to_string(), above));
    }
}

fn scan_fixture_tree_root_support(
    fixture_chunks: &[serde_json::Value],
    generated_chunks: &[LevelChunk],
) -> FixtureTreeRootSupportStats {
    let mut stats = FixtureTreeRootSupportStats::default();
    for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
        let chunk_pos = fixture_chunk_pos(fixture_chunk);
        let y_min = fixture_chunk_y_min(fixture_chunk);
        let blocks = fixture_chunk_blocks(fixture_chunk);
        let generated = &generated_chunks[chunk_index];

        for (local_x, y_column) in blocks.iter().enumerate() {
            let y_column = y_column.as_array().expect("x column should be an array");
            for (y_offset, z_column) in y_column.iter().enumerate() {
                let world_y = y_min + y_offset as i32;
                let z_column = z_column.as_array().expect("z column should be an array");
                for (local_z, expected) in z_column.iter().enumerate() {
                    let pos = BlockPos {
                        x: chunk_pos.x * 16 + local_x as i32,
                        y: world_y,
                        z: chunk_pos.z * 16 + local_z as i32,
                    };
                    let expected = fixture_block_id(expected);
                    let actual = generated
                        .get_block_state(pos.x, pos.y, pos.z)
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    record_generated_log_support(&mut stats, generated, pos, &actual);
                    record_missing_expected_base_log(&mut stats, generated, pos, expected, &actual);
                }
            }
        }
    }
    stats
}

fn print_fixture_tree_root_support(stats: FixtureTreeRootSupportStats) {
    eprintln!(
        "[tree-root-support] unsupported_generated_logs={} examples={:?}",
        stats.unsupported_generated_logs.len(),
        stats.unsupported_generated_logs
    );
    eprintln!(
        "[tree-root-support] missing_expected_base_logs={} examples={:?}",
        stats.missing_expected_base_logs.len(),
        stats.missing_expected_base_logs
    );
}

fn tree_root_diagnostic_center() -> ChunkPos {
    ChunkPos {
        x: std::env::var("VIBECRAFT_TREE_ROOT_DIAG_CENTER_X")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0),
        z: std::env::var("VIBECRAFT_TREE_ROOT_DIAG_CENTER_Z")
            .ok()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0),
    }
}

fn tree_root_diagnostic_radius() -> i32 {
    std::env::var("VIBECRAFT_TREE_ROOT_DIAG_RADIUS")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(2)
}

#[derive(Default)]
struct RegionTreeRootSupportStats {
    unsupported_logs: Vec<(i32, i32, i32, String, String)>,
    floating_trunks: Vec<(i32, i32, i32, String)>,
}

fn block_below_region_pos(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> String {
    chunks
        .get(&ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        })
        .and_then(|below_chunk| below_chunk.get_block_state(world_x, world_y - 1, world_z))
        .unwrap_or_else(|| "minecraft:air".to_string())
}

fn record_region_log_support(
    stats: &mut RegionTreeRootSupportStats,
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
    chunk: &LevelChunk,
    pos: BlockPos,
    state: String,
) {
    let below = block_below_region_pos(chunks, pos.x, pos.y, pos.z);
    let below_is_log = super::super::block_matches_tag(&below, "minecraft:logs");
    if below_is_log || super::super::block_blocks_motion(&below) {
        return;
    }
    if stats.unsupported_logs.len() < 24 {
        stats
            .unsupported_logs
            .push((pos.x, pos.y, pos.z, state, below));
    }
    let above = chunk
        .get_block_state(pos.x, pos.y + 1, pos.z)
        .unwrap_or_else(|| "minecraft:air".to_string());
    if super::super::block_matches_tag(&above, "minecraft:logs") && stats.floating_trunks.len() < 24
    {
        stats.floating_trunks.push((pos.x, pos.y, pos.z, above));
    }
}

fn scan_region_tree_root_support(
    chunks: &BTreeMap<ChunkPos, LevelChunk>,
) -> RegionTreeRootSupportStats {
    let mut stats = RegionTreeRootSupportStats::default();
    for (chunk_pos, chunk) in chunks {
        let min_y = chunk.min_section_y * 16;
        let max_y = min_y + chunk.sections.len() as i32 * 16;
        for local_z in 0..16 {
            for local_x in 0..16 {
                let world_x = chunk_pos.x * 16 + local_x;
                let world_z = chunk_pos.z * 16 + local_z;
                for world_y in min_y..max_y {
                    let Some(state) = chunk.get_block_state(world_x, world_y, world_z) else {
                        continue;
                    };
                    if !super::super::block_matches_tag(&state, "minecraft:logs") {
                        continue;
                    }
                    record_region_log_support(
                        &mut stats,
                        chunks,
                        chunk,
                        BlockPos {
                            x: world_x,
                            y: world_y,
                            z: world_z,
                        },
                        state,
                    );
                }
            }
        }
    }
    stats
}

fn print_region_tree_root_support(
    center: ChunkPos,
    radius: i32,
    chunk_count: usize,
    stats: RegionTreeRootSupportStats,
) {
    eprintln!(
        "[region-tree-root-support] center=({}, {}) radius={} chunks={} unsupported_logs={} examples={:?}",
        center.x,
        center.z,
        radius,
        chunk_count,
        stats.unsupported_logs.len(),
        stats.unsupported_logs
    );
    eprintln!(
        "[region-tree-root-support] floating_trunks={} examples={:?}",
        stats.floating_trunks.len(),
        stats.floating_trunks
    );
}

#[test]
fn lightweight_tree_context_exposes_dry_surface_as_vegetation_support() {
    let mut heights = super::super::TreeDecorationHeights {
        ocean_floor: [64; 16 * 16],
        world_surface: [64; 16 * 16],
        motion_blocking: [64; 16 * 16],
        motion_blocking_no_leaves: [64; 16 * 16],
    };
    let wet_column_index = 16;
    heights.ocean_floor[wet_column_index] = 62;
    heights.world_surface[wet_column_index] = 64;
    heights.motion_blocking[wet_column_index] = 64;
    heights.motion_blocking_no_leaves[wet_column_index] = 62;

    let chunk = super::super::LightweightTreeContextChunk {
        terrain_heights: heights,
        min_y: -64,
        max_y: 320,
    };

    assert_eq!(
        chunk.synthetic_block_state(0, 63, 0),
        "minecraft:grass_block"
    );
    assert_eq!(chunk.synthetic_block_state(0, 62, 0), "minecraft:stone");
    assert_eq!(chunk.synthetic_block_state(0, 63, 1), "minecraft:water");
    assert_eq!(chunk.synthetic_block_state(0, 61, 1), "minecraft:stone");
}

#[test]
#[ignore = "diagnostic parity stocktake; run explicitly while aligning worldgen stages"]
fn normal_overworld_stage_parity_stocktake() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let chunks = fixture_chunks(&fixture);
    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let (biome_source_model, noise_settings) = normal_overworld_noise_generator(&preset);
    let stage_chunks =
        generate_stage_parity_chunks(chunks, biome_source_model, noise_settings, seed);

    for (label, chunks) in [
        ("noise_surface", &stage_chunks.base),
        ("carvers", &stage_chunks.carver),
        ("structures", &stage_chunks.structure),
        ("ores", &stage_chunks.ore),
        ("trees", &stage_chunks.tree),
    ] {
        print_stage_parity(label, chunks);
    }
}

#[test]
#[ignore = "diagnostic for source-chunk decoration biome sets and scheduled features"]
fn normal_overworld_decoration_source_feature_schedule_diagnostic() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let chunks = fixture_chunks(&fixture);
    let (_preset, biome_source_model, noise_settings) = normal_overworld_generator();
    let (climate_sampler, features_per_step, min_section_y, sample_quart_y) =
        decoration_schedule_setup(&biome_source_model, noise_settings, seed);
    let setup = DecorationScheduleSetup {
        biome_source_model: &biome_source_model,
        noise_settings,
        climate_sampler: &climate_sampler,
        features_per_step: &features_per_step,
        min_section_y,
        sample_quart_y,
        seed,
    };

    for fixture_chunk in chunks.iter().take(4) {
        let target_pos = fixture_chunk_pos(fixture_chunk);
        for source_z in target_pos.z - 1..=target_pos.z + 1 {
            for source_x in target_pos.x - 1..=target_pos.x + 1 {
                print_source_feature_schedule(
                    target_pos,
                    ChunkPos {
                        x: source_x,
                        z: source_z,
                    },
                    &setup,
                );
            }
        }
    }
}

#[test]
#[ignore = "diagnostic for Java-shaped tree feature region writes"]
fn normal_overworld_region_tree_write_parity_stocktake() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let chunks = fixture_chunks(&fixture);
    let (_preset, biome_source_model, noise_settings) = normal_overworld_generator();
    let (mut region_chunks, source_positions) =
        generate_region_chunks_through_ores(chunks, &biome_source_model, noise_settings, seed);
    let total_tree_blocks = apply_trees_to_region_sources(
        &mut region_chunks,
        &source_positions,
        &biome_source_model,
        noise_settings,
        seed,
    );
    let generated = region_chunks.values().cloned().collect::<Vec<_>>();
    print_region_tree_parity(total_tree_blocks, &generated);
}

#[test]
#[ignore = "diagnostic for final tree-stage leaf/log placement drift"]
fn normal_overworld_tree_mismatch_shape_diagnostic() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let chunks = fixture_chunks(&fixture);
    let (_preset, biome_source_model, noise_settings) = normal_overworld_generator();
    let generated_chunks =
        generate_fixture_chunks_through_trees(chunks, &biome_source_model, noise_settings, seed);
    let stats = scan_tree_shape_mismatches(chunks, &generated_chunks);
    print_tree_shape_mismatches(stats);
}

#[test]
#[ignore = "diagnostic for tree trunks with missing or unsupported bottom logs"]
fn normal_overworld_tree_root_support_diagnostic() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let fixture_chunks = fixture_chunks(&fixture);
    let (_preset, biome_source_model, noise_settings) = normal_overworld_generator();
    let generated_chunks = generate_fixture_chunks_through_trees(
        fixture_chunks,
        &biome_source_model,
        noise_settings,
        seed,
    );
    let stats = scan_fixture_tree_root_support(fixture_chunks, &generated_chunks);
    print_fixture_tree_root_support(stats);
}

#[test]
#[ignore = "diagnostic for unsupported tree logs in region-generated chunks"]
fn normal_overworld_region_tree_root_support_diagnostic() {
    let fixture = vanilla_fixture();
    let seed = fixture_seed(&fixture);
    let center = tree_root_diagnostic_center();
    let radius = tree_root_diagnostic_radius();
    let chunks = super::super::generate_overworld_spawn_chunk_region_for_preset_with_mode(
        center,
        radius,
        "normal",
        super::super::LiveChunkGenerationMode::RealSurface,
        seed,
        false,
    )
    .expect("region generation should succeed");
    let stats = scan_region_tree_root_support(&chunks);
    print_region_tree_root_support(center, radius, chunks.len(), stats);
}
