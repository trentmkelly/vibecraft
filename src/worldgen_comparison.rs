#![allow(dead_code)]

use crate::seed_validation::{build_seed_parity_sample, ChunkCoord, SeedParitySample};
use crate::storage::chunk::{
    ChunkSection, LevelChunk, PalettedContainer, BIOME_SECTION_VOLUME,
};
#[cfg(test)]
use crate::storage::chunk::SECTION_VOLUME;
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
use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaTreeDensityDiagnostic {
    pub chunks: Vec<VanillaTreeDensityChunk>,
    pub expected_logs: usize,
    pub actual_logs: usize,
    pub expected_leaves: usize,
    pub actual_leaves: usize,
    pub expected_tree_blocks: usize,
    pub actual_tree_blocks: usize,
    pub leaf_matches: usize,
    pub log_matches: usize,
    pub extra_actual_leaves: usize,
    pub missing_expected_leaves: usize,
    pub extra_actual_logs: usize,
    pub missing_expected_logs: usize,
    pub expected_log_columns: usize,
    pub actual_log_columns: usize,
    pub matching_log_columns: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaTreeDensityChunk {
    pub chunk: ChunkCoord,
    pub expected_logs: usize,
    pub actual_logs: usize,
    pub expected_leaves: usize,
    pub actual_leaves: usize,
    pub expected_tree_blocks: usize,
    pub actual_tree_blocks: usize,
    pub leaf_matches: usize,
    pub log_matches: usize,
    pub extra_actual_leaves: usize,
    pub missing_expected_leaves: usize,
    pub extra_actual_logs: usize,
    pub missing_expected_logs: usize,
    pub expected_log_columns: Vec<VanillaTreeLogColumn>,
    pub actual_log_columns: Vec<VanillaTreeLogColumn>,
    pub matching_log_columns: usize,
    pub expected_only_log_columns: Vec<VanillaTreeLogColumn>,
    pub actual_only_log_columns: Vec<VanillaTreeLogColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaTreeLogColumn {
    pub world_x: i32,
    pub world_z: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub logs: usize,
    pub block: String,
}

mod parity_scoring;
#[cfg(test)]
pub use parity_scoring::*;

mod tree_diagnostic;
pub use tree_diagnostic::*;

mod comparison_io;
pub use comparison_io::*;

#[cfg(test)]
mod tests;
