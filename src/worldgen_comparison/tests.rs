use super::*;

const SEEDS: &[i64] = &[0, 1, -1, 12_345, 8_675_309, i64::MIN + 1, i64::MAX];
const CHUNKS: &[ChunkCoord] = &[
    ChunkCoord { x: 0, z: 0 },
    ChunkCoord { x: 1, z: -1 },
    ChunkCoord { x: -12, z: 34 },
    ChunkCoord { x: 1875, z: -1875 },
    ChunkCoord {
        x: i32::MIN / 1024,
        z: i32::MAX / 1024,
    },
];

#[test]
fn worldgen_chunk_comparison_matrix_is_deterministic_across_many_seeds_and_coordinates() {
    let first = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);
    let second = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);

    assert_eq!(first, second);
    assert_eq!(first.len(), SEEDS.len() * CHUNKS.len());
    assert!(first.iter().all(|entry| entry.fingerprint != 0));
}

#[test]
fn worldgen_source_family_golden_matrix_is_deterministic() {
    let chunk = ChunkCoord { x: -12, z: 34 };
    let first = build_worldgen_source_family_goldens(8_675_309, chunk);
    let second = build_worldgen_source_family_goldens(8_675_309, chunk);

    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|entry| (
                entry.family,
                entry.registry_items,
                entry.codec_items,
                entry.fingerprint
            ))
            .collect::<Vec<_>>(),
        vec![
            ("blending", 1, 1, 0x73ac_67d6_03ec_92f7),
            ("block_predicate", 13, 13, 0xc3ee_462f_0289_7bb6),
            ("carver", 4, 3, 0x4250_358c_87d5_b357),
            ("feature", 221, 9, 0x4406_a49b_7ed4_41f7),
            ("flat_generator", 9, 9, 0xd90a_d773_babb_cb17),
            ("height_provider", 6, 6, 0x7703_bfeb_40d0_6186),
            ("material_rule", 4, 11, 0x7094_bb91_9777_7c3a),
            ("placement_modifier", 9, 9, 0xb7be_3703_8b30_0f4f),
            ("preset", 21, 3, 0xc6f1_8fbd_24d2_044a),
            ("structure", 77, 56, 0x4ce0_499c_e60f_f4bb),
            ("synth_noise", 63, 40, 0xcb94_13d2_e01a_94cc),
        ]
    );
}

#[test]
fn worldgen_chunk_comparison_fingerprints_change_with_seed_or_coordinate() {
    let baseline =
        build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
    let different_seed =
        build_worldgen_chunk_comparisons(&[54_321], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
    let different_chunk =
        build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 2, z: 3 }])[0].clone();

    assert_ne!(baseline.fingerprint, different_seed.fingerprint);
    assert_ne!(baseline.fingerprint, different_chunk.fingerprint);
}

#[test]
fn worldgen_comparison_diff_reports_matching_seed_chunk_fingerprint_changes() {
    let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
    let mut right = left.clone();
    right[0].fingerprint ^= 0xfeed;

    assert_eq!(
        diff_worldgen_comparisons(&left, &right),
        vec![WorldgenComparisonDiff {
            seed: 12_345,
            chunk: ChunkCoord { x: 0, z: 0 },
            left_fingerprint: left[0].fingerprint,
            right_fingerprint: right[0].fingerprint,
        }]
    );
}

#[test]
fn worldgen_comparison_diff_ignores_mismatched_rows_so_matrices_fail_closed_elsewhere() {
    let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
    let right = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 1, z: 0 }]);

    assert!(diff_worldgen_comparisons(&left, &right).is_empty());
}

#[test]
fn vanilla_worldgen_block_array_parity_score_counts_matching_block_types() {
    let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(0, -64, 0, "minecraft:stone");
    chunk.set_block_state(1, -64, 0, "minecraft:dirt");
    let mut blocks = vec![vec![vec!["minecraft:air"; 16]; 1]; 16];
    blocks[0][0][0] = "minecraft:stone";
    blocks[1][0][0] = "minecraft:grass_block[snowy=false]";
    let fixture = json!({
        "format": "rustcraft-vanilla-worldgen-block-array-target-v1",
        "seed": "0",
        "chunks": [{
            "dimension": "overworld",
            "chunkX": 0,
            "chunkZ": 0,
            "yMin": -64,
            "yMaxExclusive": -63,
            "blocks": blocks
        }]
    })
    .to_string();

    let score = vanilla_worldgen_block_array_parity_score(&fixture, &[chunk]).unwrap();

    assert_eq!(score.matching_blocks, 255);
    assert_eq!(score.total_blocks, 256);
    assert_eq!(score.score, 255.0 / 256.0);
    assert_eq!(
        score.mismatches,
        vec![VanillaBlockArrayMismatchCount {
            expected: "minecraft:grass_block".to_string(),
            actual: "minecraft:dirt".to_string(),
            count: 1,
        },]
    );
    assert_eq!(
        score.samples,
        vec![VanillaBlockArrayMismatchSample {
            chunk: ChunkCoord { x: 0, z: 0 },
            local_x: 1,
            y: -64,
            local_z: 0,
            expected: "minecraft:grass_block".to_string(),
            actual: "minecraft:dirt".to_string(),
        },]
    );
    assert_eq!(
        score.chunk_mismatches,
        vec![VanillaChunkMismatchCount {
            chunk: ChunkCoord { x: 0, z: 0 },
            count: 1,
        }]
    );
    assert_eq!(
        score.y_band_mismatches,
        vec![VanillaYBandMismatchCount {
            y_min: -64,
            y_max_exclusive: -48,
            count: 1,
        }]
    );
}

#[test]
fn vanilla_worldgen_heightmap_parity_score_counts_matching_surface_columns() {
    let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(0, -64, 0, "minecraft:stone");
    chunk.set_block_state(1, -64, 0, "minecraft:dirt");
    let mut blocks = vec![vec![vec!["minecraft:air"; 16]; 1]; 16];
    blocks[0][0][0] = "minecraft:stone";
    blocks[1][0][0] = "minecraft:grass_block[snowy=false]";
    blocks[2][0][0] = "minecraft:stone";
    let fixture = json!({
        "format": "rustcraft-vanilla-worldgen-block-array-target-v1",
        "seed": "0",
        "chunks": [{
            "dimension": "overworld",
            "chunkX": 0,
            "chunkZ": 0,
            "yMin": -64,
            "yMaxExclusive": -63,
            "blocks": blocks
        }]
    })
    .to_string();

    let score = vanilla_worldgen_heightmap_parity_score(&fixture, &[chunk]).unwrap();

    assert_eq!(score.matching_columns, 253);
    assert_eq!(score.total_columns, 256);
    assert_eq!(score.score, 253.0 / 256.0);
    assert_eq!(
        score.mismatches,
        vec![VanillaHeightmapMismatchCount {
            expected_height: -63,
            actual_height: -64,
            count: 3,
        }]
    );
    assert_eq!(
        score.samples,
        vec![
            VanillaHeightmapMismatchSample {
                chunk: ChunkCoord { x: 0, z: 0 },
                local_x: 0,
                local_z: 0,
                expected_height: -63,
                actual_height: -64,
                expected_top_block: "minecraft:stone".to_string(),
                actual_top_block: "minecraft:air".to_string(),
            },
            VanillaHeightmapMismatchSample {
                chunk: ChunkCoord { x: 0, z: 0 },
                local_x: 1,
                local_z: 0,
                expected_height: -63,
                actual_height: -64,
                expected_top_block: "minecraft:grass_block".to_string(),
                actual_top_block: "minecraft:air".to_string(),
            },
            VanillaHeightmapMismatchSample {
                chunk: ChunkCoord { x: 0, z: 0 },
                local_x: 2,
                local_z: 0,
                expected_height: -63,
                actual_height: -64,
                expected_top_block: "minecraft:stone".to_string(),
                actual_top_block: "minecraft:air".to_string(),
            },
        ]
    );
    assert_eq!(
        score.delta_mismatches,
        vec![VanillaHeightmapDeltaMismatchCount {
            delta: -1,
            count: 3,
        }]
    );
}

#[test]
fn vanilla_worldgen_biome_grid_parity_score_counts_matching_quart_biomes() {
    let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    chunk.sections.push(ChunkSection {
        y: -4,
        block_states: PalettedContainer::single(
            Tag::Compound(vec![(
                "Name".to_string(),
                Tag::String("minecraft:air".to_string()),
            )]),
            SECTION_VOLUME,
        )
        .to_nbt(),
        biomes: PalettedContainer::single(
            Tag::String("minecraft:forest".to_string()),
            BIOME_SECTION_VOLUME,
        )
        .to_nbt(),
        block_light: None,
        sky_light: None,
    });
    let mut biomes = vec![vec![vec!["minecraft:forest"; 4]; 1]; 4];
    biomes[1][0][0] = "minecraft:river";
    let fixture = json!({
        "format": "rustcraft-vanilla-worldgen-block-array-target-v1",
        "seed": "0",
        "chunks": [{
            "dimension": "overworld",
            "chunkX": 0,
            "chunkZ": 0,
            "quartYMin": -16,
            "quartYMaxExclusive": -15,
            "biomes": biomes
        }]
    })
    .to_string();

    let score = vanilla_worldgen_biome_grid_parity_score(&fixture, &[chunk]).unwrap();

    assert_eq!(score.matching_biomes, 15);
    assert_eq!(score.total_biomes, 16);
    assert_eq!(score.score, 15.0 / 16.0);
    assert_eq!(
        score.mismatches,
        vec![VanillaBiomeGridMismatchCount {
            expected: "minecraft:river".to_string(),
            actual: "minecraft:forest".to_string(),
            count: 1,
        }]
    );
}

#[test]
fn vanilla_worldgen_column_profile_parity_score_counts_matching_full_columns() {
    let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    let mut block_states = PalettedContainer::single(
        Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String("minecraft:air".to_string()),
        )]),
        SECTION_VOLUME,
    );
    block_states.set_entry(
        0,
        Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String("minecraft:stone".to_string()),
        )]),
    );
    block_states.set_entry(
        256,
        Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String("minecraft:dirt".to_string()),
        )]),
    );
    block_states.set_entry(
        1,
        Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String("minecraft:stone".to_string()),
        )]),
    );
    chunk.sections.push(ChunkSection {
        y: -4,
        block_states: block_states.to_nbt(),
        biomes: PalettedContainer::single(
            Tag::String("minecraft:forest".to_string()),
            BIOME_SECTION_VOLUME,
        )
        .to_nbt(),
        block_light: None,
        sky_light: None,
    });
    let mut blocks = vec![vec![vec!["minecraft:air"; 16]; 2]; 16];
    blocks[0][0][0] = "minecraft:stone";
    blocks[0][1][0] = "minecraft:grass_block";
    blocks[1][0][0] = "minecraft:stone";
    let fixture = json!({
        "format": "rustcraft-vanilla-worldgen-block-array-target-v1",
        "seed": "0",
        "chunks": [{
            "dimension": "overworld",
            "chunkX": 0,
            "chunkZ": 0,
            "yMin": -64,
            "yMaxExclusive": -62,
            "blocks": blocks
        }]
    })
    .to_string();

    let score = vanilla_worldgen_column_profile_parity_score(&fixture, &[chunk]).unwrap();

    assert_eq!(score.matching_columns, 255);
    assert_eq!(score.total_columns, 256);
    assert_eq!(score.score, 255.0 / 256.0);
    assert_eq!(
        score.first_diff_y_mismatches,
        vec![VanillaColumnFirstDiffYMismatchCount { y: -63, count: 1 }]
    );
    assert_eq!(
        score.prefix_len_mismatches,
        vec![VanillaColumnPrefixLenMismatchCount {
            matching_prefix_blocks: 1,
            count: 1,
        }]
    );
    assert_eq!(score.surface_stack_mismatches.len(), 1);
    assert_eq!(score.samples[0].chunk, ChunkCoord { x: 0, z: 0 });
    assert_eq!(score.samples[0].local_x, 0);
    assert_eq!(score.samples[0].local_z, 0);
    assert_eq!(score.samples[0].first_diff_y, -63);
    assert_eq!(score.samples[0].matching_prefix_blocks, 1);
    assert_eq!(
        score.samples[0].expected_at_first_diff,
        "minecraft:grass_block"
    );
    assert_eq!(score.samples[0].actual_at_first_diff, "minecraft:dirt");
}

#[test]
fn normal_overworld_generation_keeps_vanilla_block_array_parity_above_threshold() {
    let score = vanilla_worldgen_block_array_parity_score_for_normal_overworld(include_str!(
        "../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    ))
    .expect("vanilla block-array fixture should score against generated Rust chunks");

    print_vanilla_worldgen_mismatch_counts(&score, 30);

    assert!(
        score.score >= 0.98,
        "worldgen block parity score {:.6} ({}/{}) is below required 0.98",
        score.score,
        score.matching_blocks,
        score.total_blocks
    );
}

#[test]
fn normal_overworld_generation_keeps_vanilla_heightmap_parity_above_threshold() {
    let score = vanilla_worldgen_heightmap_parity_score_for_normal_overworld(include_str!(
        "../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    ))
    .expect("vanilla block-array fixture should score heightmaps against generated Rust chunks");

    print_vanilla_worldgen_heightmap_mismatch_counts(&score, 30);

    assert!(
        score.score >= 0.98,
        "worldgen heightmap parity score {:.6} ({}/{}) is below required 0.98",
        score.score,
        score.matching_columns,
        score.total_columns
    );
}

#[test]
fn normal_overworld_generation_keeps_vanilla_biome_grid_parity_above_threshold() {
    let score = vanilla_worldgen_biome_grid_parity_score_for_normal_overworld(include_str!(
        "../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    ))
    .expect("vanilla block-array fixture should score biome grids against generated Rust chunks");

    print_vanilla_worldgen_biome_grid_mismatch_counts(&score, 30);

    assert!(
        score.score >= 0.98,
        "worldgen biome-grid parity score {:.6} ({}/{}) is below required 0.98",
        score.score,
        score.matching_biomes,
        score.total_biomes
    );
}

#[test]
fn normal_overworld_generation_keeps_vanilla_column_profile_parity_above_threshold() {
    let score = vanilla_worldgen_column_profile_parity_score_for_normal_overworld(include_str!(
        "../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    ))
    .expect(
        "vanilla block-array fixture should score column profiles against generated Rust chunks",
    );

    print_vanilla_worldgen_column_profile_mismatch_counts(&score, 30);

    assert!(
        score.score >= 0.98,
        "worldgen column-profile parity score {:.6} ({}/{}) is below required 0.98",
        score.score,
        score.matching_columns,
        score.total_columns
    );
}

#[test]
#[ignore = "diagnostic for investigating tree decoration density parity"]
fn normal_overworld_tree_density_diagnostic() {
    let diagnostic = vanilla_worldgen_tree_density_diagnostic_for_normal_overworld(include_str!(
        "../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
    ))
    .expect("vanilla block-array fixture should diagnose tree density");

    print_vanilla_worldgen_tree_density_diagnostic(&diagnostic);

    assert!(
        diagnostic.actual_tree_blocks > 0 || diagnostic.expected_tree_blocks > 0,
        "fixture should contain tree blocks in at least one side of the comparison"
    );
}

#[test]
fn normal_overworld_carver_preserves_known_vanilla_cave_opening() {
    let fixture =
        include_str!("../../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
    let fixture_json: Value =
        serde_json::from_str(fixture).expect("vanilla worldgen fixture must parse");
    let seed = fixture_json
        .get("seed")
        .and_then(Value::as_str)
        .expect("fixture must include seed")
        .parse::<i64>()
        .expect("fixture seed must be numeric");
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
        crate::storage::region::ChunkPos { x: -1, z: -1 },
        "normal",
        crate::worldgen::LiveChunkGenerationMode::RealSurface,
        seed,
    )
    .expect("fixture target chunk must generate");

    let cave_opening_samples = [
        BlockPos {
            x: -8,
            y: -28,
            z: -14,
        },
        BlockPos {
            x: -7,
            y: -28,
            z: -15,
        },
        BlockPos {
            x: -6,
            y: -27,
            z: -14,
        },
        BlockPos {
            x: -5,
            y: -26,
            z: -14,
        },
        BlockPos {
            x: -4,
            y: -25,
            z: -14,
        },
    ];

    for pos in cave_opening_samples {
        let expected = vanilla_fixture_block_type_at(&fixture_json, pos)
            .expect("sampled cave opening coordinate must exist in vanilla fixture");
        assert_eq!(
            expected, "minecraft:air",
            "fixture coordinate ({},{},{}) should be a vanilla cave opening",
            pos.x, pos.y, pos.z
        );
        let actual = chunk
            .get_block_state(pos.x, pos.y, pos.z)
            .unwrap_or_else(|| "minecraft:air".to_string());
        assert!(
            actual == expected || (expected == "minecraft:air" && actual == "minecraft:cave_air"),
            "Rust output at fixed-seed cave opening ({},{},{}) expected {expected}, got {actual}",
            pos.x,
            pos.y,
            pos.z
        );
    }
}

fn vanilla_fixture_block_type_at(fixture: &Value, pos: BlockPos) -> Result<String, String> {
    let chunk_x = pos.x.div_euclid(16);
    let chunk_z = pos.z.div_euclid(16);
    let local_x = pos.x.rem_euclid(16) as usize;
    let local_z = pos.z.rem_euclid(16) as usize;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen fixture is missing chunks".to_string())?;
    let chunk = chunks
        .iter()
        .find(|chunk| {
            json_i32_field(chunk, "chunkX") == Ok(chunk_x)
                && json_i32_field(chunk, "chunkZ") == Ok(chunk_z)
        })
        .ok_or_else(|| format!("vanilla fixture is missing chunk ({chunk_x},{chunk_z})"))?;
    let y_min = json_i32_field(chunk, "yMin")?;
    let y_offset = usize::try_from(pos.y - y_min)
        .map_err(|_| format!("y {} is below fixture yMin {y_min}", pos.y))?;
    chunk
        .get("blocks")
        .and_then(Value::as_array)
        .and_then(|blocks| blocks.get(local_x))
        .and_then(Value::as_array)
        .and_then(|y_column| y_column.get(y_offset))
        .and_then(Value::as_array)
        .and_then(|z_column| z_column.get(local_z))
        .and_then(Value::as_str)
        .map(vanilla_block_type)
        .ok_or_else(|| {
            format!(
                "vanilla fixture is missing block at ({},{},{})",
                pos.x, pos.y, pos.z
            )
        })
}

fn print_vanilla_worldgen_mismatch_counts(score: &VanillaBlockArrayParityScore, limit: usize) {
    println!(
        "worldgen block parity score {:.6} ({}/{})",
        score.score, score.matching_blocks, score.total_blocks
    );
    if score.mismatches.is_empty() {
        println!("worldgen block parity mismatches: none");
        return;
    }
    println!(
        "top {} worldgen block parity mismatches by expected -> actual block type:",
        limit.min(score.mismatches.len())
    );
    for mismatch in score.mismatches.iter().take(limit) {
        println!(
            "{:>8}  expected {:<36} actual {}",
            mismatch.count, mismatch.expected, mismatch.actual
        );
    }
    println!(
        "top {} worldgen block mismatch chunks:",
        limit.min(score.chunk_mismatches.len())
    );
    for mismatch in score.chunk_mismatches.iter().take(limit) {
        println!(
            "{:>8}  chunk ({:>4},{:>4})",
            mismatch.count, mismatch.chunk.x, mismatch.chunk.z
        );
    }
    println!(
        "top {} worldgen block mismatch y-bands:",
        limit.min(score.y_band_mismatches.len())
    );
    for mismatch in score.y_band_mismatches.iter().take(limit) {
        println!(
            "{:>8}  y [{:>4},{:>4})",
            mismatch.count, mismatch.y_min, mismatch.y_max_exclusive
        );
    }
    println!(
        "first {} worldgen block mismatch samples:",
        limit.min(score.samples.len())
    );
    for sample in score.samples.iter().take(limit) {
        println!(
            "  chunk ({:>4},{:>4}) local ({:>2},{:>4},{:>2}) world ({:>5},{:>4},{:>5}) expected {:<36} actual {}",
            sample.chunk.x,
            sample.chunk.z,
            sample.local_x,
            sample.y,
            sample.local_z,
            sample.chunk.x * 16 + sample.local_x as i32,
            sample.y,
            sample.chunk.z * 16 + sample.local_z as i32,
            sample.expected,
            sample.actual
        );
    }
}

fn print_vanilla_worldgen_tree_density_diagnostic(diagnostic: &VanillaTreeDensityDiagnostic) {
    println!(
        "worldgen tree density totals: expected_tree={} actual_tree={} delta={} expected_logs={} actual_logs={} log_delta={} expected_leaves={} actual_leaves={} leaf_delta={}",
        diagnostic.expected_tree_blocks,
        diagnostic.actual_tree_blocks,
        diagnostic.actual_tree_blocks as isize - diagnostic.expected_tree_blocks as isize,
        diagnostic.expected_logs,
        diagnostic.actual_logs,
        diagnostic.actual_logs as isize - diagnostic.expected_logs as isize,
        diagnostic.expected_leaves,
        diagnostic.actual_leaves,
        diagnostic.actual_leaves as isize - diagnostic.expected_leaves as isize,
    );
    println!(
        "worldgen tree density overlap: leaf_matches={} log_matches={} extra_actual_leaves={} missing_expected_leaves={} extra_actual_logs={} missing_expected_logs={} expected_log_columns={} actual_log_columns={} matching_log_columns={}",
        diagnostic.leaf_matches,
        diagnostic.log_matches,
        diagnostic.extra_actual_leaves,
        diagnostic.missing_expected_leaves,
        diagnostic.extra_actual_logs,
        diagnostic.missing_expected_logs,
        diagnostic.expected_log_columns,
        diagnostic.actual_log_columns,
        diagnostic.matching_log_columns,
    );
    println!("worldgen tree density by chunk:");
    for chunk in &diagnostic.chunks {
        println!(
            "  chunk ({:>4},{:>4}) expected_tree={:>5} actual_tree={:>5} delta={:>5} expected_logs={:>4} actual_logs={:>4} log_delta={:>4} expected_leaves={:>5} actual_leaves={:>5} leaf_delta={:>5} extra_leaves={:>5} missing_leaves={:>5} extra_logs={:>4} missing_logs={:>4} expected_log_columns={:>3} actual_log_columns={:>3} matching_log_columns={:>3}",
            chunk.chunk.x,
            chunk.chunk.z,
            chunk.expected_tree_blocks,
            chunk.actual_tree_blocks,
            chunk.actual_tree_blocks as isize - chunk.expected_tree_blocks as isize,
            chunk.expected_logs,
            chunk.actual_logs,
            chunk.actual_logs as isize - chunk.expected_logs as isize,
            chunk.expected_leaves,
            chunk.actual_leaves,
            chunk.actual_leaves as isize - chunk.expected_leaves as isize,
            chunk.extra_actual_leaves,
            chunk.missing_expected_leaves,
            chunk.extra_actual_logs,
            chunk.missing_expected_logs,
            chunk.expected_log_columns.len(),
            chunk.actual_log_columns.len(),
            chunk.matching_log_columns,
        );
        print_tree_log_column_samples("expected-only", &chunk.expected_only_log_columns, 6);
        print_tree_log_column_samples("actual-only", &chunk.actual_only_log_columns, 6);
    }
}

fn print_tree_log_column_samples(label: &str, columns: &[VanillaTreeLogColumn], limit: usize) {
    if columns.is_empty() {
        println!("    {label}: none");
        return;
    }
    let samples = columns
        .iter()
        .take(limit)
        .map(|column| {
            format!(
                "({},{} y={}..{} logs={} {})",
                column.world_x,
                column.world_z,
                column.min_y,
                column.max_y,
                column.logs,
                column.block
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "    {label} log columns (showing {}/{}): {}",
        limit.min(columns.len()),
        columns.len(),
        samples
    );
}

fn print_vanilla_worldgen_column_profile_mismatch_counts(
    score: &VanillaColumnProfileParityScore,
    limit: usize,
) {
    println!(
        "worldgen column-profile parity score {:.6} ({}/{})",
        score.score, score.matching_columns, score.total_columns
    );
    if score.first_diff_y_mismatches.is_empty() {
        println!("worldgen column-profile parity mismatches: none");
        return;
    }
    println!(
        "top {} worldgen column first-diff y values:",
        limit.min(score.first_diff_y_mismatches.len())
    );
    for mismatch in score.first_diff_y_mismatches.iter().take(limit) {
        println!("{:>8}  first diff y {:>4}", mismatch.count, mismatch.y);
    }
    println!(
        "top {} worldgen column matching-prefix lengths from yMin upward:",
        limit.min(score.prefix_len_mismatches.len())
    );
    for mismatch in score.prefix_len_mismatches.iter().take(limit) {
        println!(
            "{:>8}  prefix {:>4} blocks",
            mismatch.count, mismatch.matching_prefix_blocks
        );
    }
    println!(
        "top {} worldgen column mismatch chunks:",
        limit.min(score.chunk_mismatches.len())
    );
    for mismatch in score.chunk_mismatches.iter().take(limit) {
        println!(
            "{:>8}  chunk ({:>4},{:>4})",
            mismatch.count, mismatch.chunk.x, mismatch.chunk.z
        );
    }
    println!(
        "top {} worldgen column surface stack mismatches:",
        limit.min(score.surface_stack_mismatches.len())
    );
    for mismatch in score.surface_stack_mismatches.iter().take(limit) {
        println!(
            "{:>8}  expected [{}] actual [{}]",
            mismatch.count, mismatch.expected_stack, mismatch.actual_stack
        );
    }
    println!(
        "first {} worldgen column-profile mismatch samples:",
        limit.min(score.samples.len())
    );
    for sample in score.samples.iter().take(limit) {
        println!(
            "  chunk ({:>4},{:>4}) local ({:>2},{:>2}) world ({:>5},{:>5}) first_diff_y {:>4} prefix {:>4} expected {:<32} actual {:<32} expected_surface {:>4} actual_surface {:>4}",
            sample.chunk.x,
            sample.chunk.z,
            sample.local_x,
            sample.local_z,
            sample.chunk.x * 16 + sample.local_x as i32,
            sample.chunk.z * 16 + sample.local_z as i32,
            sample.first_diff_y,
            sample.matching_prefix_blocks,
            sample.expected_at_first_diff,
            sample.actual_at_first_diff,
            sample.expected_surface_height,
            sample.actual_surface_height
        );
    }
}

fn print_vanilla_worldgen_biome_grid_mismatch_counts(
    score: &VanillaBiomeGridParityScore,
    limit: usize,
) {
    println!(
        "worldgen biome-grid parity score {:.6} ({}/{})",
        score.score, score.matching_biomes, score.total_biomes
    );
    if score.mismatches.is_empty() {
        println!("worldgen biome-grid parity mismatches: none");
        return;
    }
    println!(
        "top {} worldgen biome-grid parity mismatches by expected -> actual biome:",
        limit.min(score.mismatches.len())
    );
    for mismatch in score.mismatches.iter().take(limit) {
        println!(
            "{:>8}  expected {:<36} actual {}",
            mismatch.count, mismatch.expected, mismatch.actual
        );
    }
}

fn print_vanilla_worldgen_heightmap_mismatch_counts(
    score: &VanillaHeightmapParityScore,
    limit: usize,
) {
    println!(
        "worldgen heightmap parity score {:.6} ({}/{})",
        score.score, score.matching_columns, score.total_columns
    );
    if score.mismatches.is_empty() {
        println!("worldgen heightmap parity mismatches: none");
        return;
    }
    println!(
        "top {} worldgen heightmap parity mismatches by expected -> actual height:",
        limit.min(score.mismatches.len())
    );
    for mismatch in score.mismatches.iter().take(limit) {
        println!(
            "{:>8}  expected {:>4}  actual {:>4}",
            mismatch.count, mismatch.expected_height, mismatch.actual_height
        );
    }
    println!(
        "top {} worldgen heightmap mismatch chunks:",
        limit.min(score.chunk_mismatches.len())
    );
    for mismatch in score.chunk_mismatches.iter().take(limit) {
        println!(
            "{:>8}  chunk ({:>4},{:>4})",
            mismatch.count, mismatch.chunk.x, mismatch.chunk.z
        );
    }
    println!(
        "top {} worldgen heightmap actual-minus-expected deltas:",
        limit.min(score.delta_mismatches.len())
    );
    for mismatch in score.delta_mismatches.iter().take(limit) {
        println!("{:>8}  delta {:+}", mismatch.count, mismatch.delta);
    }
    println!(
        "first {} worldgen heightmap mismatch samples:",
        limit.min(score.samples.len())
    );
    for sample in score.samples.iter().take(limit) {
        println!(
            "  chunk ({:>4},{:>4}) local ({:>2},{:>2}) world ({:>5},{:>5}) expected {:>4} {:<32} actual {:>4} {:<32} delta {:+}",
            sample.chunk.x,
            sample.chunk.z,
            sample.local_x,
            sample.local_z,
            sample.chunk.x * 16 + sample.local_x as i32,
            sample.chunk.z * 16 + sample.local_z as i32,
            sample.expected_height,
            sample.expected_top_block,
            sample.actual_height,
            sample.actual_top_block,
            sample.actual_height - sample.expected_height
        );
    }
}

mod signature_tests;
