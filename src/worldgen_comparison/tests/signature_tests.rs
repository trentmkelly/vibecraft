use super::super::*;

#[test]
fn chunk_signature_normalizes_generated_chunk_shape_for_vanilla_fixture_diffs() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let signature = build_chunk_signature(&chunk);

    assert_eq!(signature.chunk, ChunkCoord { x: 0, z: 0 });
    assert_eq!(signature.status, "minecraft:full");
    assert_eq!(signature.section_count, 1);
    assert_eq!(signature.non_empty_section_count, 1);
    assert!(signature.heightmaps.iter().any(|heightmap| {
        (heightmap.name == "WORLD_SURFACE_WG" || heightmap.name == "WORLD_SURFACE")
            && heightmap.entries > 0
    }));
    assert!(signature.heightmaps.iter().any(|heightmap| {
        (heightmap.name == "OCEAN_FLOOR_WG" || heightmap.name == "OCEAN_FLOOR")
            && heightmap.entries > 0
    }));
    assert!(signature
        .block_palette
        .contains(&"minecraft:grass_block".to_string()));
    assert_eq!(
        signature.biome_palette,
        vec!["minecraft:plains".to_string()]
    );
    assert_ne!(signature.payload_fingerprint, 0);
}

#[test]
fn chunk_signature_captures_structure_keys_and_section_data_fingerprints() {
    let mut chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 2, z: -3 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    chunk.structures = Tag::Compound(vec![
        (
            "starts".to_string(),
            Tag::Compound(vec![(
                "minecraft:village".to_string(),
                Tag::Compound(vec![]),
            )]),
        ),
        (
            "References".to_string(),
            Tag::Compound(vec![(
                "minecraft:mineshaft".to_string(),
                Tag::LongArray(vec![1, 2, 3]),
            )]),
        ),
    ]);
    let first = build_chunk_signature(&chunk);
    let second = build_chunk_signature(&chunk);

    assert_eq!(first, second);
    assert_eq!(
        first.structures.start_keys,
        vec!["minecraft:village".to_string()]
    );
    assert_eq!(
        first.structures.reference_keys,
        vec!["minecraft:mineshaft".to_string()]
    );
    assert!(first
        .sections
        .iter()
        .all(|section| section.block_data_fingerprint != 0));
}

#[test]
fn chunk_signature_diff_reports_field_level_drift() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let left = build_chunk_signature(&chunk);
    let mut right = left.clone();
    right.status = "minecraft:noise".to_string();
    right.block_palette.push("minecraft:water".to_string());
    right.payload_fingerprint ^= 0xfeed;

    let diffs = diff_chunk_signatures(&left, &right);
    assert_eq!(
        diffs.iter().map(|diff| diff.field).collect::<Vec<_>>(),
        vec!["status", "block_palette", "payload_fingerprint"]
    );
    assert!(diffs
        .iter()
        .all(|diff| diff.chunk == ChunkCoord { x: 0, z: 0 }));
    assert!(diffs[0].left.contains("minecraft:full"));
    assert!(diffs[0].right.contains("minecraft:noise"));
}

#[test]
fn chunk_signature_report_diff_fails_closed_on_missing_extra_and_field_drift() {
    let origin = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate origin chunk");
    let expected_extra = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 1, z: 0 },
        "flat",
    )
    .expect("flat preset should generate expected extra chunk");
    let actual_extra = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 2, z: 0 },
        "flat",
    )
    .expect("flat preset should generate actual extra chunk");

    let expected_origin = build_chunk_signature(&origin);
    let mut actual_origin = expected_origin.clone();
    actual_origin.status = "minecraft:noise".to_string();

    let diffs = diff_chunk_signature_reports(
        &[expected_origin, build_chunk_signature(&expected_extra)],
        &[actual_origin, build_chunk_signature(&actual_extra)],
    );

    assert!(diffs.contains(&WorldgenChunkReportDiff::Field(
        WorldgenChunkSignatureDiff {
            chunk: ChunkCoord { x: 0, z: 0 },
            field: "status",
            left: "\"minecraft:full\"".to_string(),
            right: "\"minecraft:noise\"".to_string(),
        }
    )));
    assert!(diffs.contains(&WorldgenChunkReportDiff::MissingActual {
        chunk: ChunkCoord { x: 1, z: 0 },
    }));
    assert!(diffs.contains(&WorldgenChunkReportDiff::MissingExpected {
        chunk: ChunkCoord { x: 2, z: 0 },
    }));
}

#[test]
fn chunk_signature_report_diff_keys_chunks_by_dimension_and_coordinate() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let overworld = build_chunk_signature(&chunk);
    let mut nether = overworld.clone();
    nether.dimension = "the_nether".to_string();

    assert_eq!(
        diff_chunk_signature_reports(&[overworld.clone(), nether], &[overworld]),
        vec![WorldgenChunkReportDiff::MissingActual {
            chunk: ChunkCoord { x: 0, z: 0 },
        }]
    );
}

#[test]
fn rustcraft_worldgen_report_uses_gate_compatible_chunk_shape() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");

    let report = build_rustcraft_worldgen_report([("overworld", &chunk)]);
    assert_eq!(
        report.get("format").and_then(Value::as_str),
        Some("rustcraft-worldgen-signatures-v1")
    );
    let chunks = report
        .get("chunks")
        .and_then(Value::as_array)
        .expect("report should include chunks");
    assert_eq!(chunks.len(), 1);
    let summary = &chunks[0];
    assert_eq!(
        summary.get("dimension").and_then(Value::as_str),
        Some("overworld")
    );
    assert_eq!(summary.get("chunkX").and_then(Value::as_i64), Some(0));
    assert_eq!(summary.get("chunkZ").and_then(Value::as_i64), Some(0));
    assert_eq!(
        summary.get("status").and_then(Value::as_str),
        Some("minecraft:full")
    );
    assert!(summary.get("sectionCount").and_then(Value::as_u64).unwrap() > 0);
    assert!(summary
        .get("heightmaps")
        .and_then(Value::as_object)
        .is_some());
    assert!(summary
        .get("structures")
        .and_then(Value::as_object)
        .is_some());
    assert!(summary
        .get("blockPalette")
        .and_then(Value::as_array)
        .is_some());
    assert!(summary
        .get("biomePalette")
        .and_then(Value::as_array)
        .is_some());
    assert!(summary.get("payloadSha256").unwrap().is_null());
    assert!(
        summary
            .get("payloadFingerprint")
            .and_then(Value::as_u64)
            .unwrap()
            > 0
    );
    assert!(summary
        .get("sections")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .all(|section| section
            .get("blockStatesData")
            .and_then(Value::as_object)
            .is_some()));
}

#[test]
fn rustcraft_worldgen_report_round_trips_to_signatures_for_regression_diffs() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let expected = build_chunk_signature(&chunk);
    let report = build_rustcraft_worldgen_report([("overworld", &chunk)]);
    let raw = serde_json::to_string(&report).unwrap();

    let parsed = parse_rustcraft_worldgen_report(&raw).unwrap();

    assert_eq!(parsed, vec![expected]);

    let mut altered = report.clone();
    altered["chunks"][0]["status"] = Value::String("minecraft:noise".to_string());
    let altered_signatures =
        parse_rustcraft_worldgen_report(&serde_json::to_string(&altered).unwrap()).unwrap();
    assert_eq!(
        diff_chunk_signature_reports(&parsed, &altered_signatures),
        vec![WorldgenChunkReportDiff::Field(WorldgenChunkSignatureDiff {
            chunk: ChunkCoord { x: 0, z: 0 },
            field: "status",
            left: "\"minecraft:full\"".to_string(),
            right: "\"minecraft:noise\"".to_string(),
        })]
    );
}

#[test]
fn rustcraft_worldgen_report_requires_explicit_chunk_dimension() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let mut report = build_rustcraft_worldgen_report([("overworld", &chunk)]);
    report["chunks"][0]
        .as_object_mut()
        .unwrap()
        .remove("dimension");

    assert_eq!(
        parse_rustcraft_worldgen_report(&serde_json::to_string(&report).unwrap()).unwrap_err(),
        "RustCraft chunk missing dimension"
    );

    report["chunks"][0]["dimension"] = Value::String(String::new());
    assert_eq!(
        parse_rustcraft_worldgen_report(&serde_json::to_string(&report).unwrap()).unwrap_err(),
        "RustCraft chunk has empty dimension"
    );
}

#[test]
fn rustcraft_worldgen_report_loads_from_generated_json_file() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let report = build_rustcraft_worldgen_report([("overworld", &chunk)]);
    let root = std::env::temp_dir().join(format!(
        "rustcraft-worldgen-report-load-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("worldgen_chunks.json");
    std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();

    let loaded = load_rustcraft_worldgen_report(&path).unwrap();

    assert_eq!(loaded, vec![build_chunk_signature(&chunk)]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn rustcraft_worldgen_report_file_diff_flags_snapshot_drift() {
    let chunk = crate::worldgen::generate_overworld_chunk_for_preset(
        crate::storage::region::ChunkPos { x: 0, z: 0 },
        "flat",
    )
    .expect("flat preset should generate a concrete chunk");
    let expected = build_rustcraft_worldgen_report([("overworld", &chunk)]);
    let mut actual = expected.clone();
    actual["chunks"][0]["status"] = Value::String("minecraft:noise".to_string());

    let root = std::env::temp_dir().join(format!(
        "rustcraft-worldgen-report-diff-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let expected_path = root.join("accepted.json");
    let actual_path = root.join("actual.json");
    std::fs::write(
        &expected_path,
        serde_json::to_string_pretty(&expected).unwrap(),
    )
    .unwrap();
    std::fs::write(&actual_path, serde_json::to_string_pretty(&actual).unwrap()).unwrap();

    assert_eq!(
        diff_rustcraft_worldgen_report_files(&expected_path, &actual_path).unwrap(),
        vec![WorldgenChunkReportDiff::Field(WorldgenChunkSignatureDiff {
            chunk: ChunkCoord { x: 0, z: 0 },
            field: "status",
            left: "\"minecraft:full\"".to_string(),
            right: "\"minecraft:noise\"".to_string(),
        })]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn vanilla_fixture_summary_normalizes_to_chunk_signature_for_rust_diffs() {
    let fixture = VanillaFixtureChunkSummary {
        dimension: "overworld".to_string(),
        chunk: ChunkCoord { x: 1, z: -2 },
        status: "minecraft:full".to_string(),
        section_count: 2,
        non_empty_section_count: 2,
        heightmaps: vec![
            WorldgenNamedArraySignature {
                name: "WORLD_SURFACE".to_string(),
                entries: 37,
                fingerprint: 20,
            },
            WorldgenNamedArraySignature {
                name: "MOTION_BLOCKING".to_string(),
                entries: 37,
                fingerprint: 10,
            },
        ],
        block_palette: vec![
            "minecraft:water".to_string(),
            "minecraft:stone".to_string(),
            "minecraft:stone".to_string(),
        ],
        biome_palette: vec![
            "minecraft:forest".to_string(),
            "minecraft:plains".to_string(),
        ],
        sections: vec![
            WorldgenSectionSignature {
                y: 1,
                block_palette: vec!["minecraft:water".to_string()],
                block_data_entries: 256,
                block_data_fingerprint: 99,
                biome_palette: vec!["minecraft:forest".to_string()],
                biome_data_entries: 64,
                biome_data_fingerprint: 77,
            },
            WorldgenSectionSignature {
                y: 0,
                block_palette: vec!["minecraft:stone".to_string(), "minecraft:air".to_string()],
                block_data_entries: 256,
                block_data_fingerprint: 55,
                biome_palette: vec!["minecraft:plains".to_string()],
                biome_data_entries: 64,
                biome_data_fingerprint: 33,
            },
        ],
        structures: WorldgenStructureSignature {
            start_keys: vec!["minecraft:village".to_string()],
            reference_keys: vec![
                "minecraft:mineshaft".to_string(),
                "minecraft:mineshaft".to_string(),
            ],
        },
        payload_fingerprint: 1234,
    };

    let signature = signature_from_vanilla_fixture_summary(fixture);

    assert_eq!(signature.chunk, ChunkCoord { x: 1, z: -2 });
    assert_eq!(
        signature
            .heightmaps
            .iter()
            .map(|heightmap| heightmap.name.as_str())
            .collect::<Vec<_>>(),
        vec!["MOTION_BLOCKING", "WORLD_SURFACE"]
    );
    assert_eq!(
        signature.block_palette,
        vec!["minecraft:stone".to_string(), "minecraft:water".to_string()]
    );
    assert_eq!(signature.sections[0].y, 0);
    assert_eq!(signature.sections[1].y, 1);
    assert_eq!(
        signature.structures.reference_keys,
        vec!["minecraft:mineshaft".to_string()]
    );
}

#[test]
fn parses_vanilla_fixture_report_json_into_signatures() {
    let report = parse_vanilla_fixture_report(
        r#"{
          "format": "rustcraft-vanilla-worldgen-fixtures-v1",
          "results": [{
            "fixture": {
              "chunks": [{ "x": 0, "z": -1, "dimension": "the_nether" }]
            },
            "artifacts": [{
              "requestedChunks": [{
                "chunkX": 0,
                "chunkZ": -1,
                "status": "minecraft:full",
                "sectionCount": 1,
                "nonEmptySectionCount": 1,
                "heightmaps": {
                  "WORLD_SURFACE": { "type": "long_array", "entries": 37 },
                  "MOTION_BLOCKING": { "type": "long_array", "entries": 37 }
                },
                "structures": {
                  "startKeys": [],
                  "referenceKeys": ["minecraft:mineshaft"]
                },
                "sections": [{
                  "y": 0,
                  "blockPalette": ["minecraft:stone", "minecraft:water"],
                  "blockStatesData": {
                    "entries": 256,
                    "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                  },
                  "biomePalette": ["minecraft:forest"],
                  "biomeData": {
                    "entries": 0,
                    "sha256": null
                  }
                }],
                "blockPalette": ["minecraft:water", "minecraft:stone"],
                "biomePalette": ["minecraft:forest"],
                "payloadSha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
              }]
            }]
          }]
        }"#,
    )
    .expect("fixture JSON should parse");

    assert_eq!(report.format, "rustcraft-vanilla-worldgen-fixtures-v1");
    assert_eq!(report.chunks.len(), 1);
    let signatures = fixture_report_signatures(&report);
    assert_eq!(signatures[0].dimension, "the_nether");
    assert_eq!(signatures[0].chunk, ChunkCoord { x: 0, z: -1 });
    assert_eq!(
        signatures[0].block_palette,
        vec!["minecraft:stone".to_string(), "minecraft:water".to_string()]
    );
    assert_eq!(
        signatures[0].structures.reference_keys,
        vec!["minecraft:mineshaft".to_string()]
    );
    assert_ne!(signatures[0].payload_fingerprint, 0);
}

#[test]
fn parses_real_tmp_oracle_fixture_report_when_available() {
    let path = std::path::Path::new("/tmp/rustcraft-vanilla-fixtures.json");
    if !path.exists() {
        return;
    }

    let report = load_vanilla_fixture_report(path).expect("real fixture report should parse");
    let signatures = fixture_report_signatures(&report);
    assert!(!signatures.is_empty());
    assert!(signatures
        .iter()
        .all(|signature| signature.status == "minecraft:full"));
    assert!(signatures.iter().any(|signature| signature
        .block_palette
        .contains(&"minecraft:stone".to_string())));
}
