#![allow(dead_code)]

use crate::seed_validation::{build_seed_parity_sample, ChunkCoord, SeedParitySample};
use crate::storage::chunk::{
    ChunkSection, LevelChunk, PalettedContainer, BIOME_SECTION_VOLUME, SECTION_VOLUME,
};
use crate::storage::nbt::Tag;
use crate::worldgen::{
    blending_output_for_old_height, block_predicate_test, carver_is_start_chunk, configured_carver,
    density_function_type, height_provider_sample_with_rolls, normal_noise_value_factor,
    placement_modifier_positions, surface_condition_test, surface_rule_apply, BlockPos,
    BlockPredicate, BlockPredicateContext, CaveSurface, HeightProvider, PlacementModifier,
    SurfaceConditionSource, SurfaceMaterialContext, SurfaceRuleSource, VerticalAnchor,
    WorldGenerationHeightContext, BLOCK_PREDICATE_TYPES, BUILTIN_NOISE_GENERATOR_SETTINGS,
    BUILTIN_NOISE_ROUTERS, CONFIGURED_CARVERS, CONFIGURED_FEATURES, DENSITY_FUNCTION_TYPES,
    FLAT_GENERATOR_PRESETS, HEIGHT_PROVIDER_TYPES, NORMAL_NOISE_PARAMETERS,
    PLACED_FEATURE_BOOTSTRAP_SOURCES, STRUCTURE_FAMILIES, STRUCTURE_PIECE_TYPES,
    SURFACE_CONDITION_TYPES, SURFACE_RULE_TYPES, SYNTH_NOISE_SOURCES, WORLD_PRESETS,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkComparison {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenComparisonDiff {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub left_fingerprint: u64,
    pub right_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenSourceFamilyGolden {
    pub family: &'static str,
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub registry_items: usize,
    pub codec_items: usize,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkSignature {
    pub dimension: String,
    pub chunk: ChunkCoord,
    pub status: String,
    pub section_count: usize,
    pub non_empty_section_count: usize,
    pub heightmaps: Vec<WorldgenNamedArraySignature>,
    pub block_palette: Vec<String>,
    pub biome_palette: Vec<String>,
    pub sections: Vec<WorldgenSectionSignature>,
    pub structures: WorldgenStructureSignature,
    pub payload_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenSectionSignature {
    pub y: i8,
    pub block_palette: Vec<String>,
    pub block_data_entries: usize,
    pub block_data_fingerprint: u64,
    pub biome_palette: Vec<String>,
    pub biome_data_entries: usize,
    pub biome_data_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenNamedArraySignature {
    pub name: String,
    pub entries: usize,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenStructureSignature {
    pub start_keys: Vec<String>,
    pub reference_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkSignatureDiff {
    pub chunk: ChunkCoord,
    pub field: &'static str,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldgenChunkReportDiff {
    MissingExpected { chunk: ChunkCoord },
    MissingActual { chunk: ChunkCoord },
    Field(WorldgenChunkSignatureDiff),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaFixtureChunkSummary {
    pub dimension: String,
    pub chunk: ChunkCoord,
    pub status: String,
    pub section_count: usize,
    pub non_empty_section_count: usize,
    pub heightmaps: Vec<WorldgenNamedArraySignature>,
    pub block_palette: Vec<String>,
    pub biome_palette: Vec<String>,
    pub sections: Vec<WorldgenSectionSignature>,
    pub structures: WorldgenStructureSignature,
    pub payload_fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaFixtureReport {
    pub format: String,
    pub chunks: Vec<VanillaFixtureChunkSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VanillaBlockArrayParityScore {
    pub matching_blocks: usize,
    pub total_blocks: usize,
    pub score: f64,
    pub mismatches: Vec<VanillaBlockArrayMismatchCount>,
    pub samples: Vec<VanillaBlockArrayMismatchSample>,
    pub chunk_mismatches: Vec<VanillaChunkMismatchCount>,
    pub y_band_mismatches: Vec<VanillaYBandMismatchCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaBlockArrayMismatchCount {
    pub expected: String,
    pub actual: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaBlockArrayMismatchSample {
    pub chunk: ChunkCoord,
    pub local_x: usize,
    pub y: i32,
    pub local_z: usize,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaChunkMismatchCount {
    pub chunk: ChunkCoord,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaYBandMismatchCount {
    pub y_min: i32,
    pub y_max_exclusive: i32,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VanillaHeightmapParityScore {
    pub matching_columns: usize,
    pub total_columns: usize,
    pub score: f64,
    pub mismatches: Vec<VanillaHeightmapMismatchCount>,
    pub samples: Vec<VanillaHeightmapMismatchSample>,
    pub chunk_mismatches: Vec<VanillaChunkMismatchCount>,
    pub delta_mismatches: Vec<VanillaHeightmapDeltaMismatchCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaHeightmapMismatchCount {
    pub expected_height: i32,
    pub actual_height: i32,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaHeightmapMismatchSample {
    pub chunk: ChunkCoord,
    pub local_x: usize,
    pub local_z: usize,
    pub expected_height: i32,
    pub actual_height: i32,
    pub expected_top_block: String,
    pub actual_top_block: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaHeightmapDeltaMismatchCount {
    pub delta: i32,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VanillaBiomeGridParityScore {
    pub matching_biomes: usize,
    pub total_biomes: usize,
    pub score: f64,
    pub mismatches: Vec<VanillaBiomeGridMismatchCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaBiomeGridMismatchCount {
    pub expected: String,
    pub actual: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VanillaColumnProfileParityScore {
    pub matching_columns: usize,
    pub total_columns: usize,
    pub score: f64,
    pub first_diff_y_mismatches: Vec<VanillaColumnFirstDiffYMismatchCount>,
    pub prefix_len_mismatches: Vec<VanillaColumnPrefixLenMismatchCount>,
    pub surface_stack_mismatches: Vec<VanillaColumnSurfaceStackMismatchCount>,
    pub samples: Vec<VanillaColumnProfileMismatchSample>,
    pub chunk_mismatches: Vec<VanillaChunkMismatchCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaColumnFirstDiffYMismatchCount {
    pub y: i32,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaColumnPrefixLenMismatchCount {
    pub matching_prefix_blocks: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaColumnSurfaceStackMismatchCount {
    pub expected_stack: String,
    pub actual_stack: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaColumnProfileMismatchSample {
    pub chunk: ChunkCoord,
    pub local_x: usize,
    pub local_z: usize,
    pub first_diff_y: i32,
    pub matching_prefix_blocks: usize,
    pub expected_at_first_diff: String,
    pub actual_at_first_diff: String,
    pub expected_surface_height: i32,
    pub actual_surface_height: i32,
    pub expected_surface_stack: String,
    pub actual_surface_stack: String,
}

impl VanillaBlockArrayParityScore {
    pub fn from_counts(matching_blocks: usize, total_blocks: usize) -> Result<Self, String> {
        Self::from_counts_and_mismatches(
            matching_blocks,
            total_blocks,
            BTreeMap::new(),
            Vec::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        )
    }

    pub fn from_counts_and_mismatches(
        matching_blocks: usize,
        total_blocks: usize,
        mismatches: BTreeMap<(String, String), usize>,
        samples: Vec<VanillaBlockArrayMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
        y_band_mismatches: BTreeMap<i32, usize>,
    ) -> Result<Self, String> {
        if total_blocks == 0 {
            return Err("cannot score worldgen parity without block samples".to_string());
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected, actual), count)| VanillaBlockArrayMismatchCount {
                    expected,
                    actual,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected.cmp(&right.expected))
                .then_with(|| left.actual.cmp(&right.actual))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        let mut y_band_mismatches = y_band_mismatches
            .into_iter()
            .map(|(y_min, count)| VanillaYBandMismatchCount {
                y_min,
                y_max_exclusive: y_min + 16,
                count,
            })
            .collect::<Vec<_>>();
        y_band_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.y_min.cmp(&right.y_min))
        });
        Ok(Self {
            matching_blocks,
            total_blocks,
            score: matching_blocks as f64 / total_blocks as f64,
            mismatches,
            samples,
            chunk_mismatches,
            y_band_mismatches,
        })
    }
}

impl VanillaHeightmapParityScore {
    pub fn from_counts_and_mismatches(
        matching_columns: usize,
        total_columns: usize,
        mismatches: BTreeMap<(i32, i32), usize>,
        samples: Vec<VanillaHeightmapMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
        delta_mismatches: BTreeMap<i32, usize>,
    ) -> Result<Self, String> {
        if total_columns == 0 {
            return Err("cannot score worldgen heightmap parity without columns".to_string());
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected_height, actual_height), count)| VanillaHeightmapMismatchCount {
                    expected_height,
                    actual_height,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected_height.cmp(&right.expected_height))
                .then_with(|| left.actual_height.cmp(&right.actual_height))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        let mut delta_mismatches = delta_mismatches
            .into_iter()
            .map(|(delta, count)| VanillaHeightmapDeltaMismatchCount { delta, count })
            .collect::<Vec<_>>();
        delta_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.delta.cmp(&right.delta))
        });
        Ok(Self {
            matching_columns,
            total_columns,
            score: matching_columns as f64 / total_columns as f64,
            mismatches,
            samples,
            chunk_mismatches,
            delta_mismatches,
        })
    }
}

impl VanillaBiomeGridParityScore {
    pub fn from_counts_and_mismatches(
        matching_biomes: usize,
        total_biomes: usize,
        mismatches: BTreeMap<(String, String), usize>,
    ) -> Result<Self, String> {
        if total_biomes == 0 {
            return Err(
                "cannot score worldgen biome-grid parity without biome samples".to_string(),
            );
        }
        let mut mismatches = mismatches
            .into_iter()
            .map(
                |((expected, actual), count)| VanillaBiomeGridMismatchCount {
                    expected,
                    actual,
                    count,
                },
            )
            .collect::<Vec<_>>();
        mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected.cmp(&right.expected))
                .then_with(|| left.actual.cmp(&right.actual))
        });
        Ok(Self {
            matching_biomes,
            total_biomes,
            score: matching_biomes as f64 / total_biomes as f64,
            mismatches,
        })
    }
}

impl VanillaColumnProfileParityScore {
    pub fn from_counts_and_mismatches(
        matching_columns: usize,
        total_columns: usize,
        first_diff_y_mismatches: BTreeMap<i32, usize>,
        prefix_len_mismatches: BTreeMap<usize, usize>,
        surface_stack_mismatches: BTreeMap<(String, String), usize>,
        samples: Vec<VanillaColumnProfileMismatchSample>,
        chunk_mismatches: BTreeMap<(i32, i32), usize>,
    ) -> Result<Self, String> {
        if total_columns == 0 {
            return Err("cannot score worldgen column-profile parity without columns".to_string());
        }
        let mut first_diff_y_mismatches = first_diff_y_mismatches
            .into_iter()
            .map(|(y, count)| VanillaColumnFirstDiffYMismatchCount { y, count })
            .collect::<Vec<_>>();
        first_diff_y_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.y.cmp(&right.y))
        });
        let mut prefix_len_mismatches = prefix_len_mismatches
            .into_iter()
            .map(
                |(matching_prefix_blocks, count)| VanillaColumnPrefixLenMismatchCount {
                    matching_prefix_blocks,
                    count,
                },
            )
            .collect::<Vec<_>>();
        prefix_len_mismatches.sort_by(|left, right| {
            right.count.cmp(&left.count).then_with(|| {
                left.matching_prefix_blocks
                    .cmp(&right.matching_prefix_blocks)
            })
        });
        let mut surface_stack_mismatches = surface_stack_mismatches
            .into_iter()
            .map(
                |((expected_stack, actual_stack), count)| VanillaColumnSurfaceStackMismatchCount {
                    expected_stack,
                    actual_stack,
                    count,
                },
            )
            .collect::<Vec<_>>();
        surface_stack_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.expected_stack.cmp(&right.expected_stack))
                .then_with(|| left.actual_stack.cmp(&right.actual_stack))
        });
        let mut chunk_mismatches = chunk_mismatches
            .into_iter()
            .map(|((x, z), count)| VanillaChunkMismatchCount {
                chunk: ChunkCoord { x, z },
                count,
            })
            .collect::<Vec<_>>();
        chunk_mismatches.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.chunk.x.cmp(&right.chunk.x))
                .then_with(|| left.chunk.z.cmp(&right.chunk.z))
        });
        Ok(Self {
            matching_columns,
            total_columns,
            score: matching_columns as f64 / total_columns as f64,
            first_diff_y_mismatches,
            prefix_len_mismatches,
            surface_stack_mismatches,
            samples,
            chunk_mismatches,
        })
    }
}

pub fn vanilla_worldgen_block_array_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaBlockArrayParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_blocks = 0usize;
    let mut total_blocks = 0usize;
    let mut mismatches = BTreeMap::<(String, String), usize>::new();
    let mut samples = Vec::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut y_band_mismatches = BTreeMap::<i32, usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }

        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for (local_x, y_column) in blocks.iter().enumerate() {
            let y_column = y_column.as_array().ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array")
            })?;
            for (y_offset, z_column) in y_column.iter().enumerate() {
                let z_column = z_column.as_array().ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} is not an array"
                    )
                })?;
                if z_column.len() != 16 {
                    return Err(format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} has {} z entries, expected 16",
                        z_column.len()
                    ));
                }
                let world_y = y_min + y_offset as i32;
                for (local_z, expected) in z_column.iter().enumerate() {
                    let expected = expected
                        .as_str()
                        .ok_or_else(|| {
                            format!(
                                "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} y_offset={y_offset} z={local_z} is not a block string"
                            )
                        })
                        .map(vanilla_block_type)?;
                    let actual = actual
                        .get_block_state(
                            chunk_x * 16 + local_x as i32,
                            world_y,
                            chunk_z * 16 + local_z as i32,
                        )
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    if actual == expected {
                        matching_blocks += 1;
                    } else {
                        let chunk = ChunkCoord {
                            x: chunk_x,
                            z: chunk_z,
                        };
                        let expected_sample = expected.clone();
                        let actual_sample = actual.clone();
                        *mismatches.entry((expected, actual)).or_default() += 1;
                        *chunk_mismatches.entry((chunk.x, chunk.z)).or_default() += 1;
                        *y_band_mismatches
                            .entry(world_y.div_euclid(16) * 16)
                            .or_default() += 1;
                        if samples.len() < 128 {
                            samples.push(VanillaBlockArrayMismatchSample {
                                chunk,
                                local_x,
                                y: world_y,
                                local_z,
                                expected: expected_sample,
                                actual: actual_sample,
                            });
                        }
                    }
                    total_blocks += 1;
                }
            }
        }
    }

    VanillaBlockArrayParityScore::from_counts_and_mismatches(
        matching_blocks,
        total_blocks,
        mismatches,
        samples,
        chunk_mismatches,
        y_band_mismatches,
    )
}

pub fn vanilla_worldgen_heightmap_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaHeightmapParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_columns = 0usize;
    let mut total_columns = 0usize;
    let mut mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut samples = Vec::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut delta_mismatches = BTreeMap::<i32, usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let y_max_exclusive = json_i32_field(expected_chunk, "yMaxExclusive")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for local_x in 0..16 {
            let y_column = blocks
                .get(local_x)
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array"
                    )
                })?;
            for local_z in 0..16 {
                let expected_height =
                    expected_world_surface_height(y_column, y_min, local_z, chunk_x, chunk_z)?;
                let actual_height = actual_world_surface_height(
                    actual,
                    chunk_x,
                    chunk_z,
                    local_x as i32,
                    local_z,
                    y_min,
                    y_max_exclusive,
                );
                if actual_height == expected_height {
                    matching_columns += 1;
                } else {
                    *mismatches
                        .entry((expected_height, actual_height))
                        .or_default() += 1;
                    let chunk = ChunkCoord {
                        x: chunk_x,
                        z: chunk_z,
                    };
                    *chunk_mismatches.entry((chunk.x, chunk.z)).or_default() += 1;
                    *delta_mismatches
                        .entry(actual_height - expected_height)
                        .or_default() += 1;
                    if samples.len() < 128 {
                        let expected_top_block = block_at_world_surface_height(
                            y_column,
                            y_min,
                            local_z,
                            expected_height,
                            chunk_x,
                            chunk_z,
                        )?;
                        let actual_top_block = actual_block_at_world_surface_height(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            actual_height,
                        );
                        samples.push(VanillaHeightmapMismatchSample {
                            chunk,
                            local_x,
                            local_z,
                            expected_height,
                            actual_height,
                            expected_top_block,
                            actual_top_block,
                        });
                    }
                }
                total_columns += 1;
            }
        }
    }

    VanillaHeightmapParityScore::from_counts_and_mismatches(
        matching_columns,
        total_columns,
        mismatches,
        samples,
        chunk_mismatches,
        delta_mismatches,
    )
}

pub fn vanilla_worldgen_biome_grid_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaBiomeGridParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_biomes = 0usize;
    let mut total_biomes = 0usize;
    let mut mismatches = BTreeMap::<(String, String), usize>::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let quart_y_min = json_i32_field(expected_chunk, "quartYMin")?;
        let biomes = expected_chunk
            .get("biomes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing biomes")
            })?;
        if biomes.len() != 4 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} quart-x columns, expected 4",
                biomes.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for (quart_x, y_column) in biomes.iter().enumerate() {
            let y_column = y_column.as_array().ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} is not an array"
                )
            })?;
            for (quart_y_offset, z_column) in y_column.iter().enumerate() {
                let z_column = z_column.as_array().ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} is not an array"
                    )
                })?;
                if z_column.len() != 4 {
                    return Err(format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} has {} quart-z entries, expected 4",
                        z_column.len()
                    ));
                }
                let quart_y = quart_y_min + quart_y_offset as i32;
                for (quart_z, expected) in z_column.iter().enumerate() {
                    let expected = expected
                        .as_str()
                        .ok_or_else(|| {
                            format!(
                                "vanilla fixture chunk ({chunk_x},{chunk_z}) quart_x={quart_x} quart_y_offset={quart_y_offset} quart_z={quart_z} is not a biome string"
                            )
                        })?
                        .to_string();
                    let actual = actual_chunk_biome(actual, quart_x, quart_y, quart_z)
                        .unwrap_or_else(|| "<missing>".to_string());
                    if actual == expected {
                        matching_biomes += 1;
                    } else {
                        *mismatches.entry((expected, actual)).or_default() += 1;
                    }
                    total_biomes += 1;
                }
            }
        }
    }

    VanillaBiomeGridParityScore::from_counts_and_mismatches(
        matching_biomes,
        total_biomes,
        mismatches,
    )
}

pub fn vanilla_worldgen_column_profile_parity_score(
    vanilla_fixture_json: &str,
    actual_chunks: &[LevelChunk],
) -> Result<VanillaColumnProfileParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let format = fixture
        .get("format")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing format".to_string())?;
    if format != "rustcraft-vanilla-worldgen-block-array-target-v1" {
        return Err(format!(
            "unsupported vanilla worldgen block-array fixture format: {format}"
        ));
    }

    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;
    let mut matching_columns = 0usize;
    let mut total_columns = 0usize;
    let mut first_diff_y_mismatches = BTreeMap::<i32, usize>::new();
    let mut prefix_len_mismatches = BTreeMap::<usize, usize>::new();
    let mut surface_stack_mismatches = BTreeMap::<(String, String), usize>::new();
    let mut chunk_mismatches = BTreeMap::<(i32, i32), usize>::new();
    let mut samples = Vec::new();

    for expected_chunk in chunks {
        let dimension = expected_chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let chunk_x = json_i32_field(expected_chunk, "chunkX")?;
        let chunk_z = json_i32_field(expected_chunk, "chunkZ")?;
        let y_min = json_i32_field(expected_chunk, "yMin")?;
        let y_max_exclusive = json_i32_field(expected_chunk, "yMaxExclusive")?;
        let blocks = expected_chunk
            .get("blocks")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                format!("vanilla fixture chunk ({chunk_x},{chunk_z}) is missing blocks")
            })?;
        if blocks.len() != 16 {
            return Err(format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) has {} x columns, expected 16",
                blocks.len()
            ));
        }

        let actual = actual_chunks
            .iter()
            .find(|chunk| chunk.pos.x == chunk_x && chunk.pos.z == chunk_z)
            .ok_or_else(|| format!("missing actual generated chunk ({chunk_x},{chunk_z})"))?;

        for local_x in 0..16 {
            let y_column = blocks
                .get(local_x)
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    format!(
                        "vanilla fixture chunk ({chunk_x},{chunk_z}) x={local_x} is not an array"
                    )
                })?;
            for local_z in 0..16 {
                let mut first_diff = None;
                for world_y in y_min..y_max_exclusive {
                    let expected = expected_block_at_world_y(
                        y_column, y_min, local_z, world_y, chunk_x, chunk_z,
                    )?;
                    let actual_block = actual
                        .get_block_state(
                            chunk_x * 16 + local_x as i32,
                            world_y,
                            chunk_z * 16 + local_z as i32,
                        )
                        .unwrap_or_else(|| "minecraft:air".to_string());
                    if expected != actual_block {
                        first_diff = Some((world_y, expected, actual_block));
                        break;
                    }
                }

                match first_diff {
                    None => matching_columns += 1,
                    Some((first_diff_y, expected_at_first_diff, actual_at_first_diff)) => {
                        let matching_prefix_blocks = (first_diff_y - y_min) as usize;
                        let expected_surface_height = expected_world_surface_height(
                            y_column, y_min, local_z, chunk_x, chunk_z,
                        )?;
                        let actual_surface_height = actual_world_surface_height(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            y_min,
                            y_max_exclusive,
                        );
                        let expected_surface_stack = expected_surface_stack_signature(
                            y_column,
                            y_min,
                            local_z,
                            expected_surface_height,
                            chunk_x,
                            chunk_z,
                        )?;
                        let actual_surface_stack = actual_surface_stack_signature(
                            actual,
                            chunk_x,
                            chunk_z,
                            local_x as i32,
                            local_z,
                            actual_surface_height,
                        );
                        *first_diff_y_mismatches.entry(first_diff_y).or_default() += 1;
                        *prefix_len_mismatches
                            .entry(matching_prefix_blocks)
                            .or_default() += 1;
                        *surface_stack_mismatches
                            .entry((expected_surface_stack.clone(), actual_surface_stack.clone()))
                            .or_default() += 1;
                        *chunk_mismatches.entry((chunk_x, chunk_z)).or_default() += 1;
                        if samples.len() < 128 {
                            samples.push(VanillaColumnProfileMismatchSample {
                                chunk: ChunkCoord {
                                    x: chunk_x,
                                    z: chunk_z,
                                },
                                local_x,
                                local_z,
                                first_diff_y,
                                matching_prefix_blocks,
                                expected_at_first_diff,
                                actual_at_first_diff,
                                expected_surface_height,
                                actual_surface_height,
                                expected_surface_stack,
                                actual_surface_stack,
                            });
                        }
                    }
                }
                total_columns += 1;
            }
        }
    }

    VanillaColumnProfileParityScore::from_counts_and_mismatches(
        matching_columns,
        total_columns,
        first_diff_y_mismatches,
        prefix_len_mismatches,
        surface_stack_mismatches,
        samples,
        chunk_mismatches,
    )
}

pub fn vanilla_worldgen_block_array_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaBlockArrayParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let seed = fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut actual_chunks = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let dimension = chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let pos = crate::storage::region::ChunkPos {
            x: json_i32_field(chunk, "chunkX")?,
            z: json_i32_field(chunk, "chunkZ")?,
        };
        actual_chunks.push(
            crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                crate::worldgen::LiveChunkGenerationMode::RealSurface,
                seed,
            )?,
        );
    }

    vanilla_worldgen_block_array_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_heightmap_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaHeightmapParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let seed = fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut actual_chunks = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let dimension = chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let pos = crate::storage::region::ChunkPos {
            x: json_i32_field(chunk, "chunkX")?,
            z: json_i32_field(chunk, "chunkZ")?,
        };
        actual_chunks.push(
            crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                crate::worldgen::LiveChunkGenerationMode::RealSurface,
                seed,
            )?,
        );
    }

    vanilla_worldgen_heightmap_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_biome_grid_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaBiomeGridParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let seed = fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut actual_chunks = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let dimension = chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let pos = crate::storage::region::ChunkPos {
            x: json_i32_field(chunk, "chunkX")?,
            z: json_i32_field(chunk, "chunkZ")?,
        };
        actual_chunks.push(
            crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                crate::worldgen::LiveChunkGenerationMode::RealSurface,
                seed,
            )?,
        );
    }

    vanilla_worldgen_biome_grid_parity_score(vanilla_fixture_json, &actual_chunks)
}

pub fn vanilla_worldgen_column_profile_parity_score_for_normal_overworld(
    vanilla_fixture_json: &str,
) -> Result<VanillaColumnProfileParityScore, String> {
    let fixture: Value = serde_json::from_str(vanilla_fixture_json)
        .map_err(|err| format!("failed to parse vanilla worldgen block-array fixture: {err}"))?;
    let seed = fixture
        .get("seed")
        .and_then(Value::as_str)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing seed".to_string())?
        .parse::<i64>()
        .map_err(|err| format!("vanilla worldgen block-array fixture has invalid seed: {err}"))?;
    let chunks = fixture
        .get("chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "vanilla worldgen block-array fixture is missing chunks".to_string())?;

    let mut actual_chunks = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let dimension = chunk
            .get("dimension")
            .and_then(Value::as_str)
            .unwrap_or("overworld");
        if dimension != "overworld" {
            return Err(format!(
                "unsupported vanilla worldgen block-array dimension: {dimension}"
            ));
        }
        let pos = crate::storage::region::ChunkPos {
            x: json_i32_field(chunk, "chunkX")?,
            z: json_i32_field(chunk, "chunkZ")?,
        };
        actual_chunks.push(
            crate::worldgen::generate_overworld_chunk_for_preset_with_mode(
                pos,
                "normal",
                crate::worldgen::LiveChunkGenerationMode::RealSurface,
                seed,
            )?,
        );
    }

    vanilla_worldgen_column_profile_parity_score(vanilla_fixture_json, &actual_chunks)
}

fn json_i32_field(value: &Value, field: &str) -> Result<i32, String> {
    let number = value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        format!("vanilla worldgen block-array fixture is missing integer {field}")
    })?;
    i32::try_from(number).map_err(|_| {
        format!("vanilla worldgen block-array fixture field {field} is out of i32 range")
    })
}

fn vanilla_block_type(block_state: &str) -> String {
    block_state
        .split_once('[')
        .map(|(name, _)| name)
        .unwrap_or(block_state)
        .to_string()
}

fn expected_world_surface_height(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<i32, String> {
    for (y_offset, z_column) in y_column.iter().enumerate().rev() {
        let z_column = z_column.as_array().ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
        let block = z_column
            .get(local_z)
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
                )
            })
            .map(vanilla_block_type)?;
        if block != "minecraft:air" {
            return Ok(y_min + y_offset as i32 + 1);
        }
    }
    Ok(y_min)
}

fn block_at_world_surface_height(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    height: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    if height <= y_min {
        return Ok("minecraft:air".to_string());
    }
    let y_offset = (height - 1 - y_min) as usize;
    let z_column = y_column
        .get(y_offset)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
    z_column
        .get(local_z)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
            )
        })
        .map(vanilla_block_type)
}

fn expected_block_at_world_y(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    world_y: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    let y_offset = (world_y - y_min) as usize;
    let z_column = y_column
        .get(y_offset)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} is not an array"
            )
        })?;
    z_column
        .get(local_z)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "vanilla fixture chunk ({chunk_x},{chunk_z}) y_offset={y_offset} z={local_z} is not a block string"
            )
        })
        .map(vanilla_block_type)
}

fn expected_surface_stack_signature(
    y_column: &[Value],
    y_min: i32,
    local_z: usize,
    surface_height: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<String, String> {
    let mut entries = Vec::new();
    for relative_y in -4..=3 {
        let world_y = surface_height + relative_y;
        let block = if world_y < y_min || world_y >= y_min + y_column.len() as i32 {
            "minecraft:air".to_string()
        } else {
            expected_block_at_world_y(y_column, y_min, local_z, world_y, chunk_x, chunk_z)?
        };
        entries.push(format!("{relative_y:+}:{block}"));
    }
    Ok(entries.join("|"))
}

fn actual_surface_stack_signature(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    surface_height: i32,
) -> String {
    let world_x = chunk_x * 16 + local_x;
    let world_z = chunk_z * 16 + local_z as i32;
    (-4..=3)
        .map(|relative_y| {
            let world_y = surface_height + relative_y;
            let block = chunk
                .get_block_state(world_x, world_y, world_z)
                .unwrap_or_else(|| "minecraft:air".to_string());
            format!("{relative_y:+}:{block}")
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn actual_world_surface_height(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    y_min: i32,
    y_max_exclusive: i32,
) -> i32 {
    let world_x = chunk_x * 16 + local_x;
    let world_z = chunk_z * 16 + local_z as i32;
    for world_y in (y_min..y_max_exclusive).rev() {
        let block = chunk
            .get_block_state(world_x, world_y, world_z)
            .unwrap_or_else(|| "minecraft:air".to_string());
        if block != "minecraft:air" {
            return world_y + 1;
        }
    }
    y_min
}

fn actual_block_at_world_surface_height(
    chunk: &LevelChunk,
    chunk_x: i32,
    chunk_z: i32,
    local_x: i32,
    local_z: usize,
    height: i32,
) -> String {
    chunk
        .get_block_state(
            chunk_x * 16 + local_x,
            height - 1,
            chunk_z * 16 + local_z as i32,
        )
        .unwrap_or_else(|| "minecraft:air".to_string())
}

fn actual_chunk_biome(
    chunk: &LevelChunk,
    quart_x: usize,
    quart_y: i32,
    quart_z: usize,
) -> Option<String> {
    if quart_x >= 4 || quart_z >= 4 {
        return None;
    }
    let section_y = quart_y.div_euclid(4) as i8;
    let local_y = quart_y.rem_euclid(4) as usize;
    let index = local_y * 16 + quart_z * 4 + quart_x;
    let section = chunk
        .sections
        .iter()
        .find(|section| section.y == section_y)?;
    let container = PalettedContainer::from_nbt(&section.biomes, BIOME_SECTION_VOLUME).ok()?;
    match container.get_entry(index)? {
        Tag::String(name) => Some(name.clone()),
        _ => None,
    }
}

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
    let sample = build_seed_parity_sample(seed, chunk.x, chunk.z);
    let context = WorldGenerationHeightContext {
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

    vec![
        source_family_golden(
            "blending",
            seed,
            chunk,
            1,
            1,
            format!(
                "{:?}",
                blending_output_for_old_height(Some(72.0), Some(2.0))
            ),
        ),
        source_family_golden(
            "block_predicate",
            seed,
            chunk,
            BLOCK_PREDICATE_TYPES.len(),
            BLOCK_PREDICATE_TYPES.len(),
            format!(
                "{:?}:{:?}",
                BLOCK_PREDICATE_TYPES,
                block_predicate_test(BlockPredicate::Solid, block_context, 64)
            ),
        ),
        source_family_golden(
            "carver",
            seed,
            chunk,
            CONFIGURED_CARVERS.len(),
            3,
            format!(
                "{:?}:{:?}",
                CONFIGURED_CARVERS,
                carver_is_start_chunk(configured_carver("cave").unwrap(), 0.15)
            ),
        ),
        source_family_golden(
            "feature",
            seed,
            chunk,
            CONFIGURED_FEATURES.len(),
            PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
            format!("{:?}", CONFIGURED_FEATURES),
        ),
        source_family_golden(
            "flat_generator",
            seed,
            chunk,
            FLAT_GENERATOR_PRESETS.len(),
            FLAT_GENERATOR_PRESETS.len(),
            format!("{:?}", FLAT_GENERATOR_PRESETS),
        ),
        source_family_golden(
            "height_provider",
            seed,
            chunk,
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
                    context,
                    4,
                    9,
                    0,
                )
            ),
        ),
        source_family_golden(
            "material_rule",
            seed,
            chunk,
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
                    &material_context,
                    &context,
                ),
                surface_rule_apply(
                    &SurfaceRuleSource::Block("minecraft:grass_block"),
                    &material_context,
                    &context,
                )
            ),
        ),
        source_family_golden(
            "placement_modifier",
            seed,
            chunk,
            PLACED_FEATURE_BOOTSTRAP_SOURCES.len(),
            9,
            format!(
                "{:?}",
                (
                    placement_modifier_positions(
                        PlacementModifier::Count { count: 2 },
                        placement_origin,
                        2,
                        0,
                        0,
                    ),
                    placement_modifier_positions(
                        PlacementModifier::InSquare,
                        placement_origin,
                        3,
                        5,
                        0,
                    ),
                    placement_modifier_positions(
                        PlacementModifier::RandomOffset {
                            xz_spread: 2,
                            y_spread: 1,
                        },
                        placement_origin,
                        3,
                        5,
                        7,
                    ),
                )
            ),
        ),
        source_family_golden(
            "preset",
            seed,
            chunk,
            WORLD_PRESETS.len()
                + BUILTIN_NOISE_GENERATOR_SETTINGS.len()
                + BUILTIN_NOISE_ROUTERS.len(),
            3,
            format!(
                "{:?}:{:?}:{:?}",
                WORLD_PRESETS, BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS
            ),
        ),
        source_family_golden(
            "structure",
            seed,
            chunk,
            STRUCTURE_FAMILIES.len() + STRUCTURE_PIECE_TYPES.len(),
            STRUCTURE_PIECE_TYPES.len(),
            format!("{:?}:{:?}", STRUCTURE_FAMILIES, sample.structure_chunks),
        ),
        source_family_golden(
            "synth_noise",
            seed,
            chunk,
            NORMAL_NOISE_PARAMETERS.len(),
            SYNTH_NOISE_SOURCES.len() + DENSITY_FUNCTION_TYPES.len(),
            format!(
                "{:?}:{:?}:{:?}",
                SYNTH_NOISE_SOURCES,
                normal_noise_value_factor(NORMAL_NOISE_PARAMETERS[11]),
                density_function_type("old_blended_noise")
            ),
        ),
    ]
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

#[cfg(test)]
mod tests {
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

        assert_eq!(score.matching_blocks, 254);
        assert_eq!(score.total_blocks, 256);
        assert_eq!(score.score, 254.0 / 256.0);
        assert_eq!(
            score.mismatches,
            vec![
                VanillaBlockArrayMismatchCount {
                    expected: "minecraft:grass_block".to_string(),
                    actual: "minecraft:air".to_string(),
                    count: 1,
                },
                VanillaBlockArrayMismatchCount {
                    expected: "minecraft:stone".to_string(),
                    actual: "minecraft:air".to_string(),
                    count: 1,
                },
            ]
        );
        assert_eq!(
            score.samples,
            vec![
                VanillaBlockArrayMismatchSample {
                    chunk: ChunkCoord { x: 0, z: 0 },
                    local_x: 0,
                    y: -64,
                    local_z: 0,
                    expected: "minecraft:stone".to_string(),
                    actual: "minecraft:air".to_string(),
                },
                VanillaBlockArrayMismatchSample {
                    chunk: ChunkCoord { x: 0, z: 0 },
                    local_x: 1,
                    y: -64,
                    local_z: 0,
                    expected: "minecraft:grass_block".to_string(),
                    actual: "minecraft:air".to_string(),
                },
            ]
        );
        assert_eq!(
            score.chunk_mismatches,
            vec![VanillaChunkMismatchCount {
                chunk: ChunkCoord { x: 0, z: 0 },
                count: 2,
            }]
        );
        assert_eq!(
            score.y_band_mismatches,
            vec![VanillaYBandMismatchCount {
                y_min: -64,
                y_max_exclusive: -48,
                count: 2,
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
            "../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
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
            "../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
        ))
        .expect(
            "vanilla block-array fixture should score heightmaps against generated Rust chunks",
        );

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
            "../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
        ))
        .expect(
            "vanilla block-array fixture should score biome grids against generated Rust chunks",
        );

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
            "../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json"
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
    fn normal_overworld_carver_preserves_known_vanilla_cave_opening() {
        let fixture =
            include_str!("../harness/mineflayer/fixtures/vanilla_worldgen_block_array_target.json");
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
}
