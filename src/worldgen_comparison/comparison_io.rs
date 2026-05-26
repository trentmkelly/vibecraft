#![allow(dead_code)]

use super::*;

pub fn build_worldgen_chunk_comparisons(
    seeds: &[i64],
    chunks: &[ChunkCoord],
) -> Vec<WorldgenChunkComparison> {
    seeds
        .iter()
        .flat_map(|seed| {
            chunks.iter().map(move |chunk| {
                let sample = build_seed_parity_sample(*seed, chunk.x, chunk.z);
                WorldgenChunkComparison {
                    seed: *seed,
                    chunk: *chunk,
                    fingerprint: fingerprint_sample(&sample),
                }
            })
        })
        .collect()
}

pub fn build_worldgen_source_family_goldens(
    seed: i64,
    chunk: ChunkCoord,
) -> Vec<WorldgenSourceFamilyGolden> {
    let context = SourceFamilyGoldenContext::new(seed, chunk);
    vec![
        blending_source_family_golden(&context),
        block_predicate_source_family_golden(&context),
        carver_source_family_golden(&context),
        feature_source_family_golden(&context),
        flat_generator_source_family_golden(&context),
        height_provider_source_family_golden(&context),
        material_rule_source_family_golden(&context),
        placement_modifier_source_family_golden(&context),
        preset_source_family_golden(&context),
        structure_source_family_golden(&context),
        synth_noise_source_family_golden(&context),
    ]
}

struct SourceFamilyGoldenContext {
    seed: i64,
    chunk: ChunkCoord,
    sample: SeedParitySample,
    height_context: WorldGenerationHeightContext,
    block_context: BlockPredicateContext,
    placement_origin: BlockPos,
    material_context: SurfaceMaterialContext,
}

impl SourceFamilyGoldenContext {
    fn new(seed: i64, chunk: ChunkCoord) -> Self {
        let sample = build_seed_parity_sample(seed, chunk.x, chunk.z);
        let height_context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        let block_context = BlockPredicateContext {
            min_y: -64,
            height: 384,
            block: "minecraft:grass_block",
            fluid: "minecraft:empty",
            solid: true,
            replaceable: false,
            unobstructed: true,
        };
        let placement_origin = BlockPos {
            x: chunk.x * 16,
            y: 72,
            z: chunk.z * 16,
        };
        let material_context = SurfaceMaterialContext {
            seed,
            random_algorithm: crate::worldgen::RandomAlgorithm::Xoroshiro,
            x: chunk.x * 16,
            y: 64,
            z: chunk.z * 16,
            biome: "minecraft:plains",
            stone_depth_above: 0,
            stone_depth_below: 4,
            surface_depth: 3,
            preliminary_surface_y: 68,
            water_height: 63,
            temperature: 0.8,
            noise: 0.25,
            steep: false,
            hole: false,
        };
        Self {
            seed,
            chunk,
            sample,
            height_context,
            block_context,
            placement_origin,
            material_context,
        }
    }
}

fn blending_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "blending",
        context.seed,
        context.chunk,
        1,
        1,
        format!(
            "{:?}",
            blending_output_for_old_height(Some(72.0), Some(2.0))
        ),
    )
}

fn block_predicate_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "block_predicate",
        context.seed,
        context.chunk,
        BLOCK_PREDICATE_TYPES.len(),
        BLOCK_PREDICATE_TYPES.len(),
        format!(
            "{:?}:{:?}",
            BLOCK_PREDICATE_TYPES,
            block_predicate_test(BlockPredicate::Solid, context.block_context, 64)
        ),
    )
}

fn carver_source_family_golden(context: &SourceFamilyGoldenContext) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "carver",
        context.seed,
        context.chunk,
        CONFIGURED_CARVERS.len(),
        3,
        format!(
            "{:?}:{:?}",
            CONFIGURED_CARVERS,
            configured_carver("cave").is_some_and(|carver| carver_is_start_chunk(carver, 0.15))
        ),
    )
}

fn feature_source_family_golden(context: &SourceFamilyGoldenContext) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "feature",
        context.seed,
        context.chunk,
        CONFIGURED_FEATURES.len(),
        PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
        format!("{:?}", CONFIGURED_FEATURES),
    )
}

fn flat_generator_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "flat_generator",
        context.seed,
        context.chunk,
        FLAT_GENERATOR_PRESETS.len(),
        FLAT_GENERATOR_PRESETS.len(),
        format!("{:?}", FLAT_GENERATOR_PRESETS),
    )
}

fn height_provider_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "height_provider",
        context.seed,
        context.chunk,
        HEIGHT_PROVIDER_TYPES.len(),
        HEIGHT_PROVIDER_TYPES.len(),
        format!(
            "{:?}:{:?}",
            HEIGHT_PROVIDER_TYPES,
            height_provider_sample_with_rolls(
                HeightProvider::Trapezoid {
                    min_inclusive: VerticalAnchor::Absolute(40),
                    max_inclusive: VerticalAnchor::Absolute(80),
                    plateau: 8,
                },
                context.height_context,
                4,
                9,
                0,
            )
        ),
    )
}

fn material_rule_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "material_rule",
        context.seed,
        context.chunk,
        SURFACE_RULE_TYPES.len(),
        SURFACE_CONDITION_TYPES.len(),
        format!(
            "{:?}:{:?}",
            surface_condition_test(
                &SurfaceConditionSource::StoneDepth {
                    offset: 1,
                    add_surface_depth: true,
                    secondary_depth_range: 0,
                    surface: CaveSurface::Floor,
                },
                &context.material_context,
                &context.height_context,
            ),
            surface_rule_apply(
                &SurfaceRuleSource::Block("minecraft:grass_block"),
                &context.material_context,
                &context.height_context,
            )
        ),
    )
}

fn placement_modifier_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "placement_modifier",
        context.seed,
        context.chunk,
        PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
        9,
        format!(
            "{:?}",
            (
                placement_modifier_positions(
                    PlacementModifier::Count { count: 2 },
                    context.placement_origin,
                    2,
                    0,
                    0,
                ),
                placement_modifier_positions(
                    PlacementModifier::InSquare,
                    context.placement_origin,
                    3,
                    5,
                    0,
                ),
                placement_modifier_positions(
                    PlacementModifier::RandomOffset {
                        xz_spread: 2,
                        y_spread: 1,
                    },
                    context.placement_origin,
                    3,
                    5,
                    7,
                ),
            )
        ),
    )
}

fn preset_source_family_golden(context: &SourceFamilyGoldenContext) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "preset",
        context.seed,
        context.chunk,
        WORLD_PRESETS.len() + BUILTIN_NOISE_GENERATOR_SETTINGS.len() + BUILTIN_NOISE_ROUTERS.len(),
        3,
        format!(
            "{:?}:{:?}:{:?}",
            WORLD_PRESETS, BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS
        ),
    )
}

fn structure_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "structure",
        context.seed,
        context.chunk,
        STRUCTURE_FAMILIES.len() + STRUCTURE_PIECE_TYPES.len(),
        STRUCTURE_PIECE_TYPES.len(),
        format!(
            "{:?}:{:?}",
            STRUCTURE_FAMILIES, context.sample.structure_chunks
        ),
    )
}

fn synth_noise_source_family_golden(
    context: &SourceFamilyGoldenContext,
) -> WorldgenSourceFamilyGolden {
    source_family_golden(
        "synth_noise",
        context.seed,
        context.chunk,
        NORMAL_NOISE_PARAMETERS.len(),
        SYNTH_NOISE_SOURCES.len() + DENSITY_FUNCTION_TYPES.len(),
        format!(
            "{:?}:{:?}:{:?}",
            SYNTH_NOISE_SOURCES,
            normal_noise_value_factor(NORMAL_NOISE_PARAMETERS[11]),
            density_function_type("old_blended_noise")
        ),
    )
}

fn source_family_golden(
    family: &'static str,
    seed: i64,
    chunk: ChunkCoord,
    registry_items: usize,
    codec_items: usize,
    payload: String,
) -> WorldgenSourceFamilyGolden {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_bytes(&mut hash, family.as_bytes());
    mix_i64(&mut hash, seed);
    mix_i32(&mut hash, chunk.x);
    mix_i32(&mut hash, chunk.z);
    mix_bytes(&mut hash, payload.as_bytes());
    WorldgenSourceFamilyGolden {
        family,
        seed,
        chunk,
        registry_items,
        codec_items,
        fingerprint: hash,
    }
}

pub fn diff_worldgen_comparisons(
    left: &[WorldgenChunkComparison],
    right: &[WorldgenChunkComparison],
) -> Vec<WorldgenComparisonDiff> {
    left.iter()
        .zip(right)
        .filter_map(|(left, right)| {
            if left.seed == right.seed
                && left.chunk == right.chunk
                && left.fingerprint != right.fingerprint
            {
                Some(WorldgenComparisonDiff {
                    seed: left.seed,
                    chunk: left.chunk,
                    left_fingerprint: left.fingerprint,
                    right_fingerprint: right.fingerprint,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn build_chunk_signature(chunk: &LevelChunk) -> WorldgenChunkSignature {
    let sections = chunk
        .sections
        .iter()
        .map(section_signature)
        .collect::<Vec<_>>();
    let mut block_palette = sections
        .iter()
        .flat_map(|section| section.block_palette.clone())
        .collect::<Vec<_>>();
    block_palette.sort();
    block_palette.dedup();
    let mut biome_palette = sections
        .iter()
        .flat_map(|section| section.biome_palette.clone())
        .collect::<Vec<_>>();
    biome_palette.sort();
    biome_palette.dedup();
    let mut heightmaps = chunk
        .heightmaps
        .iter()
        .map(|(name, tag)| named_array_signature(name, tag))
        .collect::<Vec<_>>();
    heightmaps.sort_by(|left, right| left.name.cmp(&right.name));

    WorldgenChunkSignature {
        dimension: "overworld".to_string(),
        chunk: ChunkCoord {
            x: chunk.pos.x,
            z: chunk.pos.z,
        },
        status: chunk.status.clone(),
        section_count: chunk.sections.len(),
        non_empty_section_count: sections
            .iter()
            .filter(|section| !section.block_palette.is_empty())
            .count(),
        heightmaps,
        block_palette,
        biome_palette,
        structures: structure_signature(&chunk.structures),
        payload_fingerprint: tag_fingerprint(
            &chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        sections,
    }
}

pub fn diff_chunk_signatures(
    left: &WorldgenChunkSignature,
    right: &WorldgenChunkSignature,
) -> Vec<WorldgenChunkSignatureDiff> {
    let chunk = left.chunk;
    let mut diffs = Vec::new();
    push_diff(&mut diffs, chunk, "chunk", &left.chunk, &right.chunk);
    push_diff(
        &mut diffs,
        chunk,
        "dimension",
        &left.dimension,
        &right.dimension,
    );
    push_diff(&mut diffs, chunk, "status", &left.status, &right.status);
    push_diff(
        &mut diffs,
        chunk,
        "section_count",
        &left.section_count,
        &right.section_count,
    );
    push_diff(
        &mut diffs,
        chunk,
        "non_empty_section_count",
        &left.non_empty_section_count,
        &right.non_empty_section_count,
    );
    push_diff(
        &mut diffs,
        chunk,
        "heightmaps",
        &left.heightmaps,
        &right.heightmaps,
    );
    push_diff(
        &mut diffs,
        chunk,
        "block_palette",
        &left.block_palette,
        &right.block_palette,
    );
    push_diff(
        &mut diffs,
        chunk,
        "biome_palette",
        &left.biome_palette,
        &right.biome_palette,
    );
    push_diff(
        &mut diffs,
        chunk,
        "sections",
        &left.sections,
        &right.sections,
    );
    push_diff(
        &mut diffs,
        chunk,
        "structures",
        &left.structures,
        &right.structures,
    );
    push_diff(
        &mut diffs,
        chunk,
        "payload_fingerprint",
        &left.payload_fingerprint,
        &right.payload_fingerprint,
    );
    diffs
}

pub fn diff_chunk_signature_reports(
    expected: &[WorldgenChunkSignature],
    actual: &[WorldgenChunkSignature],
) -> Vec<WorldgenChunkReportDiff> {
    let expected_by_chunk = signature_map(expected);
    let actual_by_chunk = signature_map(actual);
    let mut diffs = Vec::new();

    for (chunk_key, expected_signature) in &expected_by_chunk {
        let chunk = expected_signature.chunk;
        let Some(actual_signature) = actual_by_chunk.get(chunk_key) else {
            diffs.push(WorldgenChunkReportDiff::MissingActual { chunk });
            continue;
        };
        diffs.extend(
            diff_chunk_signatures(expected_signature, actual_signature)
                .into_iter()
                .map(WorldgenChunkReportDiff::Field),
        );
    }

    for (chunk_key, actual_signature) in &actual_by_chunk {
        if !expected_by_chunk.contains_key(chunk_key) {
            diffs.push(WorldgenChunkReportDiff::MissingExpected {
                chunk: actual_signature.chunk,
            });
        }
    }

    diffs
}

pub fn diff_rustcraft_worldgen_report_files(
    expected_path: impl AsRef<Path>,
    actual_path: impl AsRef<Path>,
) -> Result<Vec<WorldgenChunkReportDiff>, String> {
    let expected = load_rustcraft_worldgen_report(expected_path)?;
    let actual = load_rustcraft_worldgen_report(actual_path)?;
    Ok(diff_chunk_signature_reports(&expected, &actual))
}

pub fn signature_from_vanilla_fixture_summary(
    summary: VanillaFixtureChunkSummary,
) -> WorldgenChunkSignature {
    WorldgenChunkSignature {
        dimension: summary.dimension,
        chunk: summary.chunk,
        status: summary.status,
        section_count: summary.section_count,
        non_empty_section_count: summary.non_empty_section_count,
        heightmaps: sorted_named_arrays(summary.heightmaps),
        block_palette: sorted_unique(summary.block_palette),
        biome_palette: sorted_unique(summary.biome_palette),
        sections: sorted_sections(summary.sections),
        structures: WorldgenStructureSignature {
            start_keys: sorted_unique(summary.structures.start_keys),
            reference_keys: sorted_unique(summary.structures.reference_keys),
        },
        payload_fingerprint: summary.payload_fingerprint,
    }
}

pub fn load_vanilla_fixture_report(path: impl AsRef<Path>) -> Result<VanillaFixtureReport, String> {
    parse_vanilla_fixture_report(&fs::read_to_string(path.as_ref()).map_err(|err| {
        format!(
            "failed to read vanilla fixture report {}: {err}",
            path.as_ref().display()
        )
    })?)
}

pub fn parse_vanilla_fixture_report(raw: &str) -> Result<VanillaFixtureReport, String> {
    let root: Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid fixture report JSON: {err}"))?;
    let format = string_field_value(&root, "format")
        .unwrap_or("unknown")
        .to_string();
    let mut chunks = Vec::new();
    for result in array_field(&root, "results")? {
        let fixture_dimensions = fixture_dimensions_by_chunk(result)?;
        for artifact in array_field(result, "artifacts")? {
            for chunk in array_field(artifact, "requestedChunks")? {
                chunks.push(parse_fixture_chunk(chunk, &fixture_dimensions)?);
            }
        }
    }
    Ok(VanillaFixtureReport { format, chunks })
}

pub fn load_rustcraft_worldgen_report(
    path: impl AsRef<Path>,
) -> Result<Vec<WorldgenChunkSignature>, String> {
    parse_rustcraft_worldgen_report(&fs::read_to_string(path.as_ref()).map_err(|err| {
        format!(
            "failed to read RustCraft worldgen report {}: {err}",
            path.as_ref().display()
        )
    })?)
}

pub fn parse_rustcraft_worldgen_report(raw: &str) -> Result<Vec<WorldgenChunkSignature>, String> {
    let root: Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid RustCraft report JSON: {err}"))?;
    let format = string_field_value(&root, "format").unwrap_or("unknown");
    if format != "rustcraft-worldgen-signatures-v1" {
        return Err(format!(
            "unsupported RustCraft worldgen report format {format:?}"
        ));
    }
    array_field(&root, "chunks")?
        .iter()
        .map(parse_rustcraft_chunk_signature)
        .collect()
}

pub fn fixture_report_signatures(report: &VanillaFixtureReport) -> Vec<WorldgenChunkSignature> {
    report
        .chunks
        .iter()
        .cloned()
        .map(signature_from_vanilla_fixture_summary)
        .collect()
}

pub fn build_rustcraft_worldgen_report<'a>(
    chunks: impl IntoIterator<Item = (&'a str, &'a LevelChunk)>,
) -> Value {
    let chunks = chunks
        .into_iter()
        .map(|(dimension, chunk)| {
            rustcraft_chunk_signature_json(dimension, &build_chunk_signature(chunk))
        })
        .collect::<Vec<_>>();
    json!({
        "format": "rustcraft-worldgen-signatures-v1",
        "chunks": chunks,
    })
}

pub fn rustcraft_chunk_signature_json(
    dimension: &str,
    signature: &WorldgenChunkSignature,
) -> Value {
    json!({
        "dimension": dimension,
        "chunkX": signature.chunk.x,
        "chunkZ": signature.chunk.z,
        "status": signature.status,
        "sectionCount": signature.section_count,
        "nonEmptySectionCount": signature.non_empty_section_count,
        "heightmaps": signature.heightmaps.iter().map(|heightmap| {
            (
                heightmap.name.clone(),
                json!({
                    "type": "long_array",
                    "entries": heightmap.entries,
                    "fingerprint": heightmap.fingerprint,
                }),
            )
        }).collect::<serde_json::Map<_, _>>(),
        "structures": {
            "startKeys": signature.structures.start_keys,
            "referenceKeys": signature.structures.reference_keys,
        },
        "blockPalette": signature.block_palette,
        "biomePalette": signature.biome_palette,
        "payloadSha256": null,
        "payloadFingerprint": signature.payload_fingerprint,
        "sections": signature.sections.iter().map(|section| json!({
            "y": section.y,
            "blockPalette": section.block_palette,
            "blockStatesData": {
                "entries": section.block_data_entries,
                "sha256": null,
                "fingerprint": section.block_data_fingerprint,
            },
            "biomePalette": section.biome_palette,
            "biomeData": {
                "entries": section.biome_data_entries,
                "sha256": null,
                "fingerprint": section.biome_data_fingerprint,
            },
        })).collect::<Vec<_>>(),
    })
}

fn parse_fixture_chunk(
    value: &Value,
    fixture_dimensions: &BTreeMap<(i32, i32), String>,
) -> Result<VanillaFixtureChunkSummary, String> {
    let chunk = ChunkCoord {
        x: i32_field(value, "chunkX")?,
        z: i32_field(value, "chunkZ")?,
    };
    let dimension = fixture_dimensions
        .get(&(chunk.x, chunk.z))
        .cloned()
        .unwrap_or_else(|| "overworld".to_string());
    Ok(VanillaFixtureChunkSummary {
        dimension,
        chunk,
        status: string_field_value(value, "status")
            .ok_or_else(|| "fixture chunk missing status".to_string())?
            .to_string(),
        section_count: usize_field(value, "sectionCount")?,
        non_empty_section_count: usize_field(value, "nonEmptySectionCount")?,
        heightmaps: parse_heightmaps(
            value
                .get("heightmaps")
                .ok_or_else(|| "fixture chunk missing heightmaps".to_string())?,
        )?,
        block_palette: string_array_field(value, "blockPalette")?,
        biome_palette: string_array_field(value, "biomePalette")?,
        sections: parse_sections(array_field(value, "sections")?)?,
        structures: parse_structures(
            value
                .get("structures")
                .ok_or_else(|| "fixture chunk missing structures".to_string())?,
        )?,
        payload_fingerprint: hex_fingerprint_field(value, "payloadSha256")?,
    })
}

fn parse_rustcraft_chunk_signature(value: &Value) -> Result<WorldgenChunkSignature, String> {
    Ok(WorldgenChunkSignature {
        dimension: required_non_empty_string(value, "dimension", "RustCraft chunk")?.to_string(),
        chunk: ChunkCoord {
            x: i32_field(value, "chunkX")?,
            z: i32_field(value, "chunkZ")?,
        },
        status: string_field_value(value, "status")
            .ok_or_else(|| "RustCraft chunk missing status".to_string())?
            .to_string(),
        section_count: usize_field(value, "sectionCount")?,
        non_empty_section_count: usize_field(value, "nonEmptySectionCount")?,
        heightmaps: parse_heightmaps(
            value
                .get("heightmaps")
                .ok_or_else(|| "RustCraft chunk missing heightmaps".to_string())?,
        )?,
        block_palette: sorted_unique(string_array_field(value, "blockPalette")?),
        biome_palette: sorted_unique(string_array_field(value, "biomePalette")?),
        sections: sorted_sections(parse_sections(array_field(value, "sections")?)?),
        structures: {
            let structures = parse_structures(
                value
                    .get("structures")
                    .ok_or_else(|| "RustCraft chunk missing structures".to_string())?,
            )?;
            WorldgenStructureSignature {
                start_keys: sorted_unique(structures.start_keys),
                reference_keys: sorted_unique(structures.reference_keys),
            }
        },
        payload_fingerprint: usize_field(value, "payloadFingerprint")? as u64,
    })
}

fn parse_heightmaps(value: &Value) -> Result<Vec<WorldgenNamedArraySignature>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "heightmaps must be an object".to_string())?;
    let mut entries = object
        .iter()
        .map(|(name, value)| {
            Ok(WorldgenNamedArraySignature {
                name: name.clone(),
                entries: usize_field(value, "entries")?,
                fingerprint: value
                    .get("fingerprint")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

fn parse_sections(values: &[Value]) -> Result<Vec<WorldgenSectionSignature>, String> {
    values
        .iter()
        .map(|value| {
            Ok(WorldgenSectionSignature {
                y: i8_field(value, "y")?,
                block_palette: string_array_field(value, "blockPalette")?,
                block_data_entries: usize_field(
                    value
                        .get("blockStatesData")
                        .ok_or_else(|| "section missing blockStatesData".to_string())?,
                    "entries",
                )?,
                block_data_fingerprint: optional_report_fingerprint_field(
                    value
                        .get("blockStatesData")
                        .ok_or_else(|| "section missing blockStatesData".to_string())?,
                    "sha256",
                )?,
                biome_palette: string_array_field(value, "biomePalette")?,
                biome_data_entries: usize_field(
                    value
                        .get("biomeData")
                        .ok_or_else(|| "section missing biomeData".to_string())?,
                    "entries",
                )?,
                biome_data_fingerprint: optional_report_fingerprint_field(
                    value
                        .get("biomeData")
                        .ok_or_else(|| "section missing biomeData".to_string())?,
                    "sha256",
                )?,
            })
        })
        .collect()
}

fn parse_structures(value: &Value) -> Result<WorldgenStructureSignature, String> {
    Ok(WorldgenStructureSignature {
        start_keys: string_array_field(value, "startKeys")?,
        reference_keys: string_array_field(value, "referenceKeys")?,
    })
}

fn fixture_dimensions_by_chunk(result: &Value) -> Result<BTreeMap<(i32, i32), String>, String> {
    let mut dimensions = BTreeMap::new();
    let Some(fixture) = result.get("fixture") else {
        return Ok(dimensions);
    };
    for chunk in array_field(fixture, "chunks")? {
        dimensions.insert(
            (i32_field(chunk, "x")?, i32_field(chunk, "z")?),
            string_field_value(chunk, "dimension")
                .unwrap_or("overworld")
                .to_string(),
        );
    }
    Ok(dimensions)
}

fn array_field<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    Ok(value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]))
}

fn string_array_field(value: &Value, field: &str) -> Result<Vec<String>, String> {
    array_field(value, field)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| format!("{field} entries must be strings"))
        })
        .collect()
}

fn string_field_value<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn required_non_empty_string<'a>(
    value: &'a Value,
    field: &str,
    context: &str,
) -> Result<&'a str, String> {
    let raw =
        string_field_value(value, field).ok_or_else(|| format!("{context} missing {field}"))?;
    if raw.is_empty() {
        return Err(format!("{context} has empty {field}"));
    }
    Ok(raw)
}

fn i32_field(value: &Value, field: &str) -> Result<i32, String> {
    let raw = value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing integer field {field}"))?;
    i32::try_from(raw).map_err(|_| format!("{field} is out of i32 range"))
}

fn i8_field(value: &Value, field: &str) -> Result<i8, String> {
    let raw = value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing integer field {field}"))?;
    i8::try_from(raw).map_err(|_| format!("{field} is out of i8 range"))
}

fn usize_field(value: &Value, field: &str) -> Result<usize, String> {
    let raw = value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing unsigned field {field}"))?;
    usize::try_from(raw).map_err(|_| format!("{field} is out of usize range"))
}

fn hex_fingerprint_field(value: &Value, field: &str) -> Result<u64, String> {
    optional_hex_fingerprint_field(value, field).and_then(|fingerprint| {
        if fingerprint == 0 {
            Err(format!("{field} must contain a non-null SHA-256 string"))
        } else {
            Ok(fingerprint)
        }
    })
}

fn optional_hex_fingerprint_field(value: &Value, field: &str) -> Result<u64, String> {
    match value.get(field) {
        Some(Value::String(hex)) => Ok(fingerprint_hex(hex)),
        Some(Value::Null) | None => Ok(0),
        _ => Err(format!("{field} must be a SHA-256 string or null")),
    }
}

fn optional_report_fingerprint_field(value: &Value, hash_field: &str) -> Result<u64, String> {
    let hashed = optional_hex_fingerprint_field(value, hash_field)?;
    if hashed != 0 {
        return Ok(hashed);
    }
    Ok(value
        .get("fingerprint")
        .and_then(Value::as_u64)
        .unwrap_or(0))
}

fn fingerprint_hex(hex: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_bytes(&mut hash, hex.as_bytes());
    hash
}

fn sorted_named_arrays(
    mut values: Vec<WorldgenNamedArraySignature>,
) -> Vec<WorldgenNamedArraySignature> {
    values.sort_by(|left, right| left.name.cmp(&right.name));
    values
}

fn sorted_sections(mut values: Vec<WorldgenSectionSignature>) -> Vec<WorldgenSectionSignature> {
    for section in &mut values {
        section.block_palette = sorted_unique(std::mem::take(&mut section.block_palette));
        section.biome_palette = sorted_unique(std::mem::take(&mut section.biome_palette));
    }
    values.sort_by_key(|section| section.y);
    values
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn signature_map(
    signatures: &[WorldgenChunkSignature],
) -> BTreeMap<(&str, i32, i32), &WorldgenChunkSignature> {
    signatures
        .iter()
        .map(|signature| {
            (
                (
                    signature.dimension.as_str(),
                    signature.chunk.x,
                    signature.chunk.z,
                ),
                signature,
            )
        })
        .collect()
}

fn push_diff<T: std::fmt::Debug + PartialEq>(
    diffs: &mut Vec<WorldgenChunkSignatureDiff>,
    chunk: ChunkCoord,
    field: &'static str,
    left: &T,
    right: &T,
) {
    if left != right {
        diffs.push(WorldgenChunkSignatureDiff {
            chunk,
            field,
            left: format!("{left:?}"),
            right: format!("{right:?}"),
        });
    }
}

fn section_signature(section: &ChunkSection) -> WorldgenSectionSignature {
    let block_palette = palette_names(&section.block_states, true);
    let biome_palette = palette_names(&section.biomes, false);
    let block_data = container_data(&section.block_states);
    let biome_data = container_data(&section.biomes);
    WorldgenSectionSignature {
        y: section.y,
        block_palette,
        block_data_entries: block_data.len(),
        block_data_fingerprint: fingerprint_i64s(block_data),
        biome_palette,
        biome_data_entries: biome_data.len(),
        biome_data_fingerprint: fingerprint_i64s(biome_data),
    }
}

fn named_array_signature(name: &str, tag: &Tag) -> WorldgenNamedArraySignature {
    let values = match tag {
        Tag::LongArray(values) => values.as_slice(),
        _ => &[],
    };
    WorldgenNamedArraySignature {
        name: name.to_string(),
        entries: values.len(),
        fingerprint: fingerprint_i64s(values),
    }
}

fn structure_signature(tag: &Tag) -> WorldgenStructureSignature {
    let starts = compound_field(tag, "starts")
        .map(compound_keys)
        .unwrap_or_default();
    let references = compound_field(tag, "References")
        .or_else(|| compound_field(tag, "references"))
        .map(compound_keys)
        .unwrap_or_default();
    WorldgenStructureSignature {
        start_keys: starts,
        reference_keys: references,
    }
}

fn palette_names(container: &Tag, block_states: bool) -> Vec<String> {
    let mut names = compound_field(container, "palette")
        .and_then(|tag| match tag {
            Tag::List(values) => Some(values.as_slice()),
            _ => None,
        })
        .unwrap_or(&[])
        .iter()
        .filter_map(|tag| {
            if block_states {
                string_field(tag, "Name")
            } else if let Tag::String(value) = tag {
                Some(value.as_str())
            } else {
                None
            }
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn container_data(container: &Tag) -> &[i64] {
    match compound_field(container, "data") {
        Some(Tag::LongArray(values)) => values.as_slice(),
        _ => &[],
    }
}

fn compound_field<'a>(tag: &'a Tag, name: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .find(|(field_name, _)| field_name == name)
            .map(|(_, value)| value),
        _ => None,
    }
}

fn string_field<'a>(tag: &'a Tag, name: &str) -> Option<&'a str> {
    match compound_field(tag, name) {
        Some(Tag::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn compound_keys(tag: &Tag) -> Vec<String> {
    let mut keys = match tag {
        Tag::Compound(fields) => fields.iter().map(|(name, _)| name.clone()).collect(),
        _ => Vec::new(),
    };
    keys.sort();
    keys
}

fn tag_fingerprint(tag: &Tag) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_tag(&mut hash, tag);
    hash
}

fn fingerprint_i64s(values: &[i64]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for value in values {
        mix_i64(&mut hash, *value);
    }
    hash
}

fn mix_tag(hash: &mut u64, tag: &Tag) {
    match tag {
        Tag::End => mix_bytes(hash, &[0]),
        Tag::Byte(value) => mix_bytes(hash, &[*value as u8]),
        Tag::Short(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Int(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Long(value) => mix_i64(hash, *value),
        Tag::Float(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::Double(value) => mix_bytes(hash, &value.to_le_bytes()),
        Tag::ByteArray(values) => {
            for value in values {
                mix_bytes(hash, &[*value as u8]);
            }
        }
        Tag::String(value) => mix_bytes(hash, value.as_bytes()),
        Tag::List(values) => {
            for value in values {
                mix_tag(hash, value);
            }
        }
        Tag::Compound(fields) => {
            let sorted = fields
                .iter()
                .map(|(name, tag)| (name.as_str(), tag))
                .collect::<BTreeMap<_, _>>();
            for (name, tag) in sorted {
                mix_bytes(hash, name.as_bytes());
                mix_tag(hash, tag);
            }
        }
        Tag::IntArray(values) => {
            for value in values {
                mix_i32(hash, *value);
            }
        }
        Tag::LongArray(values) => {
            for value in values {
                mix_i64(hash, *value);
            }
        }
    }
}

fn fingerprint_sample(sample: &SeedParitySample) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_i64(&mut hash, sample.seed);
    mix_i32(&mut hash, sample.chunk.x);
    mix_i32(&mut hash, sample.chunk.z);
    mix_i32(&mut hash, sample.biome_sample_block.0);
    mix_i32(&mut hash, sample.biome_sample_block.1);
    mix_i32(&mut hash, sample.biome_sample_block.2);
    mix_i32(&mut hash, sample.spawn_search_candidate.0);
    mix_i32(&mut hash, sample.spawn_search_candidate.1);
    mix_i64(&mut hash, sample.decoration_seed);
    mix_i64(&mut hash, sample.feature_seed);
    mix_i64(&mut hash, sample.slime_chunk_seed);
    mix_i64(&mut hash, sample.loot_sequence_seed.lo);
    mix_i64(&mut hash, sample.loot_sequence_seed.hi);
    for (id, chunk) in &sample.structure_chunks {
        mix_bytes(&mut hash, id.as_bytes());
        mix_i32(&mut hash, chunk.x);
        mix_i32(&mut hash, chunk.z);
    }
    hash
}

fn mix_i32(hash: &mut u64, value: i32) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_i64(hash: &mut u64, value: i64) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
