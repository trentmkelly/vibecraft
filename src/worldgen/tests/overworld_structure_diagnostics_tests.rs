use super::*;

type StructureBox = super::super::StructureBoundingBoxModel;
type MineshaftPiece = super::super::MineshaftGeneratedPieceModel;

#[derive(Debug)]
struct ClosestMineshaftCandidate {
    distance: i32,
    source_x: i32,
    source_z: i32,
    piece_count: usize,
    piece_index: usize,
    bounding_box: StructureBox,
}

fn load_target_fixture() -> serde_json::Value {
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

fn normal_overworld_noise_models() -> (
    BiomeSourceModel,
    &'static super::super::NoiseGeneratorSettings,
) {
    let preset =
        super::super::resolve_world_preset("normal").expect("normal preset should resolve");
    let super::super::ResolvedChunkGenerator::Noise {
        biome_source_model,
        noise_settings,
        ..
    } = preset.overworld.generator
    else {
        panic!("normal overworld should use a noise generator");
    };
    (biome_source_model, noise_settings)
}

fn generated_chunks_for_fixture(
    fixture_chunks: &[serde_json::Value],
    biome_source_model: &BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    seed: i64,
) -> Vec<LevelChunk> {
    let mut generated_chunks = Vec::new();
    for fixture_chunk in fixture_chunks {
        let (base, _, _) = super::super::generate_real_surface_base_chunk(
            fixture_chunk_pos(fixture_chunk),
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
        super::super::apply_underground_ore_decoration_to_chunk(
            &mut chunk,
            biome_source_model,
            noise_settings,
            seed,
            None,
        );
        super::super::apply_initial_tree_decoration_to_chunk(
            &mut chunk,
            biome_source_model,
            noise_settings,
            seed,
            None,
            None,
            None,
        );
        generated_chunks.push(chunk);
    }
    generated_chunks
}

fn fixture_block_id(block: &serde_json::Value) -> &str {
    let block = block.as_str().expect("fixture block should be a string");
    block.split_once('[').map_or(block, |(id, _)| id)
}

fn missing_cave_air_positions(
    fixture_chunks: &[serde_json::Value],
    generated_chunks: &[LevelChunk],
) -> Vec<BlockPos> {
    let mut missing_cave_air = Vec::new();
    for (chunk_index, fixture_chunk) in fixture_chunks.iter().enumerate() {
        let chunk_pos = fixture_chunk_pos(fixture_chunk);
        let y_min = fixture_chunk
            .get("yMin")
            .and_then(serde_json::Value::as_i64)
            .expect("fixture chunk should include yMin") as i32;
        let blocks = fixture_chunk
            .get("blocks")
            .and_then(serde_json::Value::as_array)
            .expect("fixture chunk should include blocks");
        let generated = &generated_chunks[chunk_index];

        for (local_x, y_column) in blocks.iter().enumerate() {
            let y_column = y_column.as_array().expect("x column should be an array");
            for (y_offset, z_column) in y_column.iter().enumerate() {
                collect_missing_cave_air_in_column(
                    &mut missing_cave_air,
                    generated,
                    chunk_pos,
                    y_min + y_offset as i32,
                    local_x,
                    z_column,
                );
            }
        }
    }
    missing_cave_air
}

fn collect_missing_cave_air_in_column(
    missing_cave_air: &mut Vec<BlockPos>,
    generated: &LevelChunk,
    chunk_pos: ChunkPos,
    world_y: i32,
    local_x: usize,
    z_column: &serde_json::Value,
) {
    let z_column = z_column.as_array().expect("z column should be an array");
    for (local_z, expected) in z_column.iter().enumerate() {
        if fixture_block_id(expected) != "minecraft:cave_air" {
            continue;
        }
        let world_x = chunk_pos.x * 16 + local_x as i32;
        let world_z = chunk_pos.z * 16 + local_z as i32;
        let actual = generated
            .get_block_state(world_x, world_y, world_z)
            .unwrap_or_else(|| "minecraft:air".to_string());
        if actual == "minecraft:deepslate" || actual == "minecraft:stone" {
            missing_cave_air.push(BlockPos {
                x: world_x,
                y: world_y,
                z: world_z,
            });
        }
    }
}

fn bounding_box_for_positions(positions: &[BlockPos]) -> Option<StructureBox> {
    positions.iter().copied().fold(None, |acc, pos| {
        Some(acc.map_or_else(
            || StructureBox {
                min_x: pos.x,
                min_y: pos.y,
                min_z: pos.z,
                max_x: pos.x,
                max_y: pos.y,
                max_z: pos.z,
            },
            |box_: StructureBox| box_.encapsulate_pos(pos),
        ))
    })
}

fn log_missing_cave_air_summary(missing_cave_air: &[BlockPos], missing_box: Option<StructureBox>) {
    eprintln!(
        "[cave-air-structure] missing_cave_air={} missing_box={:?}",
        missing_cave_air.len(),
        missing_box
    );
}

fn axis_gap(a_min: i32, a_max: i32, b_min: i32, b_max: i32) -> i32 {
    if a_max < b_min {
        b_min - a_max
    } else if b_max < a_min {
        a_min - b_max
    } else {
        0
    }
}

fn bounding_box_distance(a: StructureBox, b: StructureBox) -> i32 {
    axis_gap(a.min_x, a.max_x, b.min_x, b.max_x)
        + axis_gap(a.min_y, a.max_y, b.min_y, b.max_y)
        + axis_gap(a.min_z, a.max_z, b.min_z, b.max_z)
}

fn closest_mineshaft_piece(
    pieces: &[MineshaftPiece],
    missing: StructureBox,
) -> Option<(i32, usize, StructureBox)> {
    pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            let bounding_box = piece.bounding_box();
            (
                bounding_box_distance(bounding_box, missing),
                index,
                bounding_box,
            )
        })
        .min_by_key(|(distance, _, _)| *distance)
}

fn scan_mineshaft_candidates(
    seed: i64,
    missing_box: Option<StructureBox>,
) -> (usize, usize, Vec<ClosestMineshaftCandidate>) {
    let mut candidate_count = 0usize;
    let mut near_missing_count = 0usize;
    let mut closest_candidates = Vec::new();
    for source_x in -64..=64 {
        for source_z in -64..=64 {
            let Some(intersects_missing) = log_mineshaft_candidate(
                seed,
                source_x,
                source_z,
                missing_box,
                &mut closest_candidates,
            ) else {
                continue;
            };
            candidate_count += 1;
            if intersects_missing {
                near_missing_count += 1;
            }
        }
    }
    closest_candidates.sort_by_key(|candidate| candidate.distance);
    (candidate_count, near_missing_count, closest_candidates)
}

fn mineshaft_candidate_exists(seed: i64, source_x: i32, source_z: i32) -> bool {
    super::super::structure_frequency_reducer_should_generate(
        super::super::FrequencyReductionMethod::LegacyType3,
        seed,
        0,
        source_x,
        source_z,
        0.004,
    )
    .expect("mineshaft frequency check should be valid")
}

fn log_mineshaft_candidate(
    seed: i64,
    source_x: i32,
    source_z: i32,
    missing_box: Option<StructureBox>,
    closest_candidates: &mut Vec<ClosestMineshaftCandidate>,
) -> Option<bool> {
    if !mineshaft_candidate_exists(seed, source_x, source_z) {
        return None;
    }

    let mut random = super::super::RandomSourceKind::Legacy(super::super::LegacyRandom::new(
        crate::random_source::large_feature_seed(seed, source_x, source_z),
    ));
    let first_roll = super::super::random_next_f64(&mut random);
    let room = super::super::mineshaft_room(
        ChunkPos {
            x: source_x,
            z: source_z,
        },
        super::super::MineshaftTypeModel::Normal,
        super::super::random_next_i32_bound(&mut random, 6),
        super::super::random_next_i32_bound(&mut random, 6),
        super::super::random_next_i32_bound(&mut random, 6),
    )
    .expect("room rolls should be valid");
    let intersects_missing =
        missing_box.is_some_and(|missing| room.bounding_box.inflated_by(96).intersects(missing));
    let pieces = mineshaft_pieces_for_source(seed, source_x, source_z, 63, -64);
    let intersecting_pieces = pieces
        .iter()
        .filter(|piece| missing_box.is_some_and(|missing| piece.bounding_box().intersects(missing)))
        .count();
    let closest_piece = missing_box.and_then(|missing| closest_mineshaft_piece(&pieces, missing));

    if let Some((distance, piece_index, bounding_box)) = closest_piece {
        closest_candidates.push(ClosestMineshaftCandidate {
            distance,
            source_x,
            source_z,
            piece_count: pieces.len(),
            piece_index,
            bounding_box,
        });
    }
    if intersects_missing || intersecting_pieces > 0 {
        eprintln!(
            "[cave-air-structure-candidate] chunk=({}, {}) first_roll={:.9} room={:?} generated_pieces={} intersecting_pieces={} closest_piece={:?}",
            source_x,
            source_z,
            first_roll,
            room.bounding_box,
            pieces.len(),
            intersecting_pieces,
            closest_piece
        );
    }
    Some(intersects_missing)
}

fn mineshaft_pieces_for_source(
    seed: i64,
    source_x: i32,
    source_z: i32,
    sea_level: i32,
    min_y: i32,
) -> Vec<MineshaftPiece> {
    super::super::mineshaft_generate_pieces_for_start(
        seed,
        ChunkPos {
            x: source_x,
            z: source_z,
        },
        super::super::MineshaftTypeModel::Normal,
        sea_level,
        min_y,
    )
}

fn log_closest_mineshaft_candidates(closest_candidates: Vec<ClosestMineshaftCandidate>) {
    for candidate in closest_candidates.into_iter().take(8) {
        eprintln!(
            "[cave-air-structure-closest] distance={} chunk=({}, {}) generated_pieces={} piece_index={} box={:?}",
            candidate.distance,
            candidate.source_x,
            candidate.source_z,
            candidate.piece_count,
            candidate.piece_index,
            candidate.bounding_box
        );
    }
}

fn mineshaft_piece_kind(piece: &MineshaftPiece) -> &'static str {
    match piece {
        super::super::MineshaftGeneratedPieceModel::Room { .. } => "room",
        super::super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
        super::super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
        super::super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
    }
}

fn log_nearest_mineshaft_start(
    seed: i64,
    biome_source_model: &BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
) {
    let source_pos = ChunkPos { x: -1, z: 4 };
    let start_pos = super::super::mineshaft_start_pos(source_pos);
    let pieces = mineshaft_pieces_for_source(
        seed,
        source_pos.x,
        source_pos.z,
        noise_settings.sea_level,
        noise_settings.noise.min_y,
    );
    let room_box = pieces[0].bounding_box();
    let y_offset = room_box.min_y - 50;
    let stub_pos = BlockPos {
        x: start_pos.x,
        y: start_pos.y + y_offset,
        z: start_pos.z,
    };
    let biome = biome_at_stub(seed, biome_source_model, noise_settings, stub_pos);
    eprintln!(
        "[mineshaft-nearest] source=({}, {}) seed={} start={:?} y_offset={} stub={:?} biome={} pieces={}",
        source_pos.x,
        source_pos.z,
        seed,
        start_pos,
        y_offset,
        stub_pos,
        biome,
        pieces.len()
    );
    log_first_mineshaft_pieces(&pieces);
    log_mineshaft_pieces_near_missing(&pieces);
}

fn biome_at_stub(
    seed: i64,
    biome_source_model: &BiomeSourceModel,
    noise_settings: &super::super::NoiseGeneratorSettings,
    stub_pos: BlockPos,
) -> &'static str {
    let router = super::super::builtin_noise_router(super::super::noise_router_id_for_settings(
        *noise_settings,
    ))
    .expect("normal overworld should have a router")
    .router;
    let climate_sampler =
        super::super::ClimateSampler::from_noise_router(&router, seed, *noise_settings);
    super::super::get_biome(
        biome_source_model,
        stub_pos.x >> 2,
        stub_pos.y >> 2,
        stub_pos.z >> 2,
        &climate_sampler,
    )
    .unwrap_or("minecraft:unknown")
}

fn log_first_mineshaft_pieces(pieces: &[MineshaftPiece]) {
    for (index, piece) in pieces.iter().enumerate().take(40) {
        eprintln!(
            "[mineshaft-nearest-first] index={} kind={} depth={} box={:?}",
            index,
            mineshaft_piece_kind(piece),
            piece.gen_depth(),
            piece.bounding_box()
        );
    }
}

fn missing_cave_air_diagnostic_box() -> StructureBox {
    StructureBox {
        min_x: 0,
        min_y: -56,
        min_z: 0,
        max_x: 22,
        max_y: -45,
        max_z: 15,
    }
}

fn log_mineshaft_pieces_near_missing(pieces: &[MineshaftPiece]) {
    let missing = missing_cave_air_diagnostic_box();
    for (index, piece) in pieces.iter().enumerate() {
        let bounding_box = piece.bounding_box();
        let distance = bounding_box_distance(bounding_box, missing);
        if distance <= 24 {
            eprintln!(
                "[mineshaft-nearest-piece] index={} kind={} depth={} distance={} box={:?}",
                index,
                mineshaft_piece_kind(piece),
                piece.gen_depth(),
                distance,
                bounding_box
            );
        }
    }
}

#[test]
#[ignore = "diagnostic for missing cave-air that may come from underground structures"]
fn normal_overworld_cave_air_structure_candidate_diagnostic() {
    let fixture = load_target_fixture();
    let seed = fixture_seed(&fixture);
    let chunks = fixture_chunks(&fixture);
    let (biome_source_model, noise_settings) = normal_overworld_noise_models();
    let generated_chunks =
        generated_chunks_for_fixture(chunks, &biome_source_model, noise_settings, seed);

    let missing_cave_air = missing_cave_air_positions(chunks, &generated_chunks);
    let missing_box = bounding_box_for_positions(&missing_cave_air);
    log_missing_cave_air_summary(&missing_cave_air, missing_box);

    let (candidate_count, near_missing_count, closest_candidates) =
        scan_mineshaft_candidates(seed, missing_box);
    log_closest_mineshaft_candidates(closest_candidates);
    eprintln!(
        "[cave-air-structure] candidate_count={} near_missing_count={}",
        candidate_count, near_missing_count
    );
}

#[test]
#[ignore = "diagnostic for the mineshaft start nearest the vanilla cave-air mismatch"]
fn normal_overworld_mineshaft_nearest_start_diagnostic() {
    let fixture = load_target_fixture();
    let seed = fixture_seed(&fixture);
    let (biome_source_model, noise_settings) = normal_overworld_noise_models();
    log_nearest_mineshaft_start(seed, &biome_source_model, noise_settings);
}
