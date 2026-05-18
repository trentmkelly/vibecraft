#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::biome::{
    biome_source_from_stem_id, climate_target, select_biome_from_source, span, BiomeSourceModel,
    ClimateParameterPoint,
};
use crate::random_source::{large_feature_seed_with_salt, LegacyRandom};
use crate::storage::chunk::{
    BlockStateEntry, ChunkSection, HeightmapKind, LevelChunk, PalettedContainer,
    BIOME_SECTION_VOLUME, SECTION_VOLUME,
};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseSettings {
    pub min_y: i32,
    pub height: i32,
    pub size_horizontal: i32,
    pub size_vertical: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalNoiseParameters {
    pub id: &'static str,
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynthNoiseSource {
    pub id: &'static str,
    pub codec: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoiseGeneratorSettings {
    pub id: &'static str,
    pub noise: NoiseSettings,
    pub default_block: &'static str,
    pub default_fluid: &'static str,
    pub noise_router: NoiseRouterPreset,
    pub surface_rule: SurfaceRulePreset,
    pub spawn_target: &'static [ClimateParameterPoint],
    pub sea_level: i32,
    pub disable_mob_generation: bool,
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseRouter {
    pub barrier: DensityFunction,
    pub fluid_level_floodedness: DensityFunction,
    pub fluid_level_spread: DensityFunction,
    pub lava: DensityFunction,
    pub temperature: DensityFunction,
    pub vegetation: DensityFunction,
    pub continents: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub ridges: DensityFunction,
    pub preliminary_surface_level: DensityFunction,
    pub final_density: DensityFunction,
    pub vein_toggle: DensityFunction,
    pub vein_ridged: DensityFunction,
    pub vein_gap: DensityFunction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseRouterEntry {
    pub id: &'static str,
    pub router: NoiseRouter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceRulePresetData {
    pub id: &'static str,
    pub rule: SurfaceRuleKind,
    pub blocks: &'static [&'static str],
    pub conditions: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRuleKind {
    OverworldLike {
        preliminary_surface_check: bool,
        bedrock_roof: bool,
        bedrock_floor: bool,
        deepslate: bool,
    },
    Nether,
    State(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceMaterialContext {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub biome: &'static str,
    pub stone_depth_above: i32,
    pub stone_depth_below: i32,
    pub surface_depth: i32,
    pub preliminary_surface_y: i32,
    pub water_height: i32,
    pub temperature: f32,
    pub noise: f64,
    pub steep: bool,
    pub hole: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSurface {
    Floor,
    Ceiling,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceConditionSource {
    Biome(&'static [&'static str]),
    NoiseThreshold {
        min: f64,
        max: f64,
    },
    VerticalGradient {
        true_at_and_below: VerticalAnchor,
        false_at_and_above: VerticalAnchor,
    },
    YAbove {
        anchor: VerticalAnchor,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface: CaveSurface,
    },
    Not(&'static SurfaceConditionSource),
    Steep,
    Hole,
    AbovePreliminarySurface,
    Temperature,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SurfaceRuleSource {
    Bandlands,
    Block(&'static str),
    Sequence(&'static [SurfaceRuleSource]),
    Condition {
        condition: &'static SurfaceConditionSource,
        rule: &'static SurfaceRuleSource,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceRuleType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceConditionType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AquiferNoiseSettings {
    pub x_range: i32,
    pub y_range: i32,
    pub z_range: i32,
    pub x_separation: i32,
    pub y_separation: i32,
    pub z_separation: i32,
    pub x_spacing: i32,
    pub y_spacing: i32,
    pub z_spacing: i32,
    pub max_reasonable_distance_to_center: i32,
    pub sample_offset_x: i32,
    pub sample_offset_y: i32,
    pub sample_offset_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluidStatus {
    pub fluid_level: i32,
    pub fluid_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaveGenerationFamily {
    pub id: &'static str,
    pub noises: &'static [&'static str],
    pub output: CaveDensityOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveDensityOutput {
    CacheOnce,
    Clamp { min: i32, max: i32 },
    RangeChoice,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVeinifierConstants {
    pub veininess_threshold: f64,
    pub edge_roundoff_begin: i32,
    pub max_edge_roundoff: f64,
    pub vein_solidness: f64,
    pub min_richness: f64,
    pub max_richness: f64,
    pub max_richness_threshold: f64,
    pub chance_of_raw_ore_block: f64,
    pub skip_ore_if_gap_noise_is_below: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreVeinType {
    pub id: &'static str,
    pub ore: &'static str,
    pub raw_ore_block: &'static str,
    pub filler: &'static str,
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVeinDecisionInput {
    pub y: i32,
    pub vein_toggle: f64,
    pub vein_ridged: f64,
    pub vein_gap: f64,
    pub solidness_random: f64,
    pub richness_random: f64,
    pub raw_ore_random: f64,
    pub debug_ore_veins: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfiguredCarver {
    pub id: &'static str,
    pub carver_type: WorldCarverType,
    pub probability: f32,
    pub y: HeightRange,
    pub y_scale: FloatProvider,
    pub lava_level: VerticalAnchor,
    pub debug: CarverDebugSettings,
    pub replaceable_tag: &'static str,
    pub shape: CarverShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldCarverType {
    Cave,
    NetherCave,
    Canyon,
}

impl WorldCarverType {
    pub const fn id(self) -> &'static str {
        match self {
            WorldCarverType::Cave => "minecraft:cave",
            WorldCarverType::NetherCave => "minecraft:nether_cave",
            WorldCarverType::Canyon => "minecraft:canyon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CarverShape {
    Cave {
        horizontal_radius_multiplier: FloatProvider,
        vertical_radius_multiplier: FloatProvider,
        floor_level: FloatProvider,
    },
    Canyon {
        vertical_rotation: FloatProvider,
        shape: CanyonShapeConfiguration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanyonShapeConfiguration {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatProvider {
    Constant(f32),
    Uniform { min: f32, max: f32 },
    Trapezoid { min: f32, max: f32, plateau: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightRange {
    pub min: VerticalAnchor,
    pub max: VerticalAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationHeightContext {
    pub min_y: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightProviderType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedHeightProvider {
    pub weight: i32,
    pub provider: HeightProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightProvider {
    Constant {
        value: VerticalAnchor,
    },
    Uniform {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
    },
    BiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    VeryBiasedToBottom {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        inner: i32,
    },
    Trapezoid {
        min_inclusive: VerticalAnchor,
        max_inclusive: VerticalAnchor,
        plateau: i32,
    },
    WeightedList {
        distribution: &'static [WeightedHeightProvider],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPredicateType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPredicateContext {
    pub min_y: i32,
    pub height: i32,
    pub block: &'static str,
    pub fluid: &'static str,
    pub solid: bool,
    pub replaceable: bool,
    pub unobstructed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockPredicate {
    MatchingBlocks {
        blocks: &'static [&'static str],
    },
    MatchingFluids {
        fluids: &'static [&'static str],
    },
    Solid,
    Replaceable,
    InsideWorldBounds {
        offset_y: i32,
    },
    AnyOf {
        predicates: &'static [BlockPredicate],
    },
    AllOf {
        predicates: &'static [BlockPredicate],
    },
    Not {
        predicate: &'static BlockPredicate,
    },
    True,
    Unobstructed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementModifier {
    RarityFilter {
        chance: i32,
    },
    BiomeFilter,
    BlockPredicateFilter {
        predicate: BlockPredicate,
    },
    SurfaceWaterDepthFilter {
        max_water_depth: i32,
    },
    SurfaceRelativeThresholdFilter {
        heightmap: HeightmapKind,
        min_inclusive: i32,
        max_inclusive: i32,
    },
    Count {
        count: i32,
    },
    InSquare,
    Heightmap {
        heightmap: HeightmapKind,
    },
    RandomOffset {
        xz_spread: i32,
        y_spread: i32,
    },
    Fixed {
        positions: &'static [BlockPos],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementContextModel {
    pub min_y: i32,
    pub world_surface_height: i32,
    pub ocean_floor_height: i32,
    pub biome_allows_feature: bool,
    pub block_predicate: BlockPredicateContext,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeGenerationSettingsModel {
    pub biome: &'static str,
    pub carvers: &'static [&'static str],
    pub feature_steps: &'static [&'static [&'static str]],
    pub creature_spawn_probability: f32,
    pub spawn_costs: &'static [MobSpawnCostModel],
    pub spawners: &'static [MobSpawnerGroupModel],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobSpawnCostModel {
    pub entity_type: &'static str,
    pub energy_budget: f64,
    pub charge: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobSpawnerGroupModel {
    pub category: &'static str,
    pub entries: &'static [MobSpawnerDataModel],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobSpawnerDataModel {
    pub entity_type: &'static str,
    pub weight: i32,
    pub min_count: i32,
    pub max_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FeatureSorterData {
    pub feature_index: usize,
    pub step: usize,
    pub feature: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFeatureDataModel {
    pub features: Vec<&'static str>,
}

impl StepFeatureDataModel {
    pub fn index_mapping(&self, feature: &str) -> Option<usize> {
        self.features
            .iter()
            .position(|candidate| *candidate == feature)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatLayerInfo {
    pub height: i32,
    pub block: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatGeneratorPreset {
    pub id: &'static str,
    pub display: &'static str,
    pub biome: &'static str,
    pub structures: &'static [&'static str],
    pub add_lakes: bool,
    pub decoration: bool,
    pub layers: &'static [FlatLayerInfo],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatGeneratorSettingsModel {
    pub biome: &'static str,
    pub structure_overrides: Vec<&'static str>,
    pub add_lakes: bool,
    pub decoration: bool,
    pub layers: Vec<FlatLayerInfo>,
    pub expanded_layers: Vec<Option<&'static str>>,
    pub top_layer_modifications: Vec<(usize, &'static str)>,
    pub void_generation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatNoiseColumn {
    pub min_y: i32,
    pub states: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelStemPreset {
    pub dimension: &'static str,
    pub generator: &'static str,
    pub biome_source: &'static str,
    pub noise_settings: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkGeneratorKind {
    Noise,
    Flat,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedChunkGenerator {
    Noise {
        biome_source: &'static str,
        biome_source_model: BiomeSourceModel,
        noise_settings: &'static NoiseGeneratorSettings,
    },
    Flat {
        biome_source_model: BiomeSourceModel,
        settings: FlatGeneratorSettingsModel,
    },
    Debug {
        biome: &'static str,
        biome_source_model: BiomeSourceModel,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLevelStem {
    pub dimension: &'static str,
    pub generator: ResolvedChunkGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedWorldPreset {
    pub id: &'static str,
    pub overworld: ResolvedLevelStem,
    pub nether: ResolvedLevelStem,
    pub end: ResolvedLevelStem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldPresetEntry {
    pub id: &'static str,
    pub overworld: LevelStemPreset,
    pub nether: LevelStemPreset,
    pub end: LevelStemPreset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarverDebugSettings {
    pub enabled: bool,
    pub barrier_state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureType {
    pub id: &'static str,
    pub configuration: FeatureConfigurationKind,
    pub family: FeatureFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfiguredFeatureEntry {
    pub id: &'static str,
    pub source: ConfiguredFeatureSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfiguredFeatureSource {
    Aquatic,
    Cave,
    End,
    MiscOverworld,
    Nether,
    Ore,
    Pile,
    Tree,
    Vegetation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacedFeatureSourceEntry {
    pub source: PlacedFeatureSource,
    pub keys: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacedFeatureSource {
    Aquatic,
    Cave,
    End,
    MiscOverworld,
    Nether,
    Ore,
    Tree,
    Vegetation,
    Village,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldgenTypeRegistry {
    pub id: &'static str,
    pub entries: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureBehaviorModel {
    pub feature_type: &'static str,
    pub behavior: &'static str,
    pub success_condition: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedBlockState {
    pub state: &'static str,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStateProviderModel {
    Simple(&'static str),
    Weighted(Vec<WeightedBlockState>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleBlockConfigurationModel {
    pub to_place: BlockStateProviderModel,
    pub schedule_tick: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleBlockPlacementContext {
    pub origin_block: &'static str,
    pub below_block: &'static str,
    pub above_block: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleBlockPlacementPlan {
    pub state: &'static str,
    pub upper_state: Option<&'static str>,
    pub schedule_tick: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTestModel {
    AlwaysTrue,
    BlockMatch(&'static str),
    TagMatch(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetBlockStateModel {
    pub target: RuleTestModel,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OreConfigurationModel {
    pub target_states: Vec<TargetBlockStateModel>,
    pub size: i32,
    pub discard_chance_on_air_exposure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScatteredOreAttempt {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OreVeinSphere {
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub radius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrePlacementContext {
    pub pos: BlockPos,
    pub current_block: &'static str,
    pub adjacent_to_air: bool,
    pub air_check_roll: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrePlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AquaticPlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpringConfigurationModel {
    pub state: &'static str,
    pub requires_block_below: bool,
    pub rock_count: i32,
    pub hole_count: i32,
    pub valid_blocks: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpringPlacementContext {
    pub origin: BlockPos,
    pub above_block: &'static str,
    pub below_block: &'static str,
    pub current_block: &'static str,
    pub west_block: &'static str,
    pub east_block: &'static str,
    pub north_block: &'static str,
    pub south_block: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpringPlacementPlan {
    pub pos: BlockPos,
    pub state: &'static str,
    pub schedule_tick: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VegetationPatchConfigurationModel {
    pub replaceable: &'static [&'static str],
    pub ground_state: BlockStateProviderModel,
    pub vegetation_feature: &'static str,
    pub surface: CaveSurface,
    pub depth_min: i32,
    pub depth_max: i32,
    pub extra_bottom_block_chance: f32,
    pub vertical_range: i32,
    pub vegetation_chance: f32,
    pub xz_radius_min: i32,
    pub xz_radius_max: i32,
    pub extra_edge_column_chance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VegetationPatchGroundColumn {
    pub surface_pos: BlockPos,
    pub ground_start: BlockPos,
    pub depth: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VegetationPatchBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VegetationPatchPlan {
    pub ground: Vec<VegetationPatchBlock>,
    pub vegetation_origins: Vec<BlockPos>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LakeConfigurationModel {
    pub fluid: BlockStateProviderModel,
    pub barrier: BlockStateProviderModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LakeBoundaryBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state: &'static str,
    pub solid: bool,
    pub liquid: bool,
    pub cannot_replace: bool,
    pub should_freeze: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LakePlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub schedule_tick: bool,
    pub mark_above_for_post_processing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FossilFeatureConfigurationModel {
    pub fossil_structures: Vec<&'static str>,
    pub overlay_structures: Vec<&'static str>,
    pub fossil_processors: &'static str,
    pub overlay_processors: &'static str,
    pub max_empty_corners_allowed: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FossilPlacementPlan {
    pub fossil_structure: &'static str,
    pub overlay_structure: &'static str,
    pub rotation: StructureRotation,
    pub target_pos: BlockPos,
    pub fossil_processors: &'static str,
    pub overlay_processors: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeLayerSettingsModel {
    pub filling: f64,
    pub inner_layer: f64,
    pub middle_layer: f64,
    pub outer_layer: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeCrackSettingsModel {
    pub generate_crack_chance: f32,
    pub base_crack_size: f64,
    pub crack_point_offset: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeodeConfigurationModel {
    pub filling_provider: BlockStateProviderModel,
    pub inner_layer_provider: BlockStateProviderModel,
    pub alternate_inner_layer_provider: BlockStateProviderModel,
    pub middle_layer_provider: BlockStateProviderModel,
    pub outer_layer_provider: BlockStateProviderModel,
    pub inner_placements: &'static [&'static str],
    pub cannot_replace: &'static [&'static str],
    pub invalid_blocks: &'static [&'static str],
    pub layers: GeodeLayerSettingsModel,
    pub crack: GeodeCrackSettingsModel,
    pub use_potential_placements_chance: f32,
    pub use_alternate_layer0_chance: f32,
    pub placements_require_layer0_alternate: bool,
    pub outer_wall_distance_max: i32,
    pub invalid_blocks_threshold: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeodeDistributionPoint {
    pub pos: BlockPos,
    pub offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeLayerThresholds {
    pub inner_air: f64,
    pub innermost_block_layer: f64,
    pub inner_crust: f64,
    pub outer_crust: f64,
    pub crack_size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeodeLayer {
    CrackAir,
    Filling,
    Inner,
    AlternateInner,
    Middle,
    Outer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeodePlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub layer: GeodeLayer,
    pub potential_crystal_source: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IcebergShapeModel {
    pub snow_on_top: bool,
    pub shape_angle: f64,
    pub shape_ellipse_a: i32,
    pub shape_ellipse_c: i32,
    pub is_ellipse: bool,
    pub over_water_height: i32,
    pub under_water_height: i32,
    pub width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IcebergCutoutModel {
    pub local_origin: BlockPos,
    pub angle_is_shape_perpendicular: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcebergBlockAction {
    Keep,
    MainBlock,
    SnowBlock,
    Air,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedPlacedFeatureModel {
    pub feature: &'static str,
    pub chance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RandomFeatureConfigurationModel {
    pub features: Vec<WeightedPlacedFeatureModel>,
    pub default_feature: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRandomFeatureConfigurationModel {
    pub features: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomBooleanFeatureConfigurationModel {
    pub feature_true: &'static str,
    pub feature_false: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillLayerConfigurationModel {
    pub height: i32,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndIslandPlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaceSphereConfigurationModel {
    pub target_state: &'static str,
    pub replace_state: &'static str,
    pub radius_min: i32,
    pub radius_max: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasaltPillarBlockKind {
    Core,
    HangOff,
    Base,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasaltPillarPlacementBlock {
    pub pos: BlockPos,
    pub kind: BasaltPillarBlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnFeatureConfigurationModel {
    pub reach_min: i32,
    pub reach_max: i32,
    pub height_min: i32,
    pub height_max: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasaltColumnPlacementBlock {
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaFeatureConfigurationModel {
    pub contents: &'static str,
    pub rim: &'static str,
    pub size_min: i32,
    pub size_max: i32,
    pub rim_size_min: i32,
    pub rim_size_max: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaPlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetherForestVegetationConfigModel {
    pub state_provider: BlockStateProviderModel,
    pub spread_width: i32,
    pub spread_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VineColumnBlockKind {
    Plant,
    Head,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VineColumnBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub kind: VineColumnBlockKind,
    pub age: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndGatewayConfigurationModel {
    pub exit: Option<BlockPos>,
    pub exact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeaturePlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChorusPlantPlacementKind {
    Plant,
    Flower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChorusPlantPlacementBlock {
    pub pos: BlockPos,
    pub kind: ChorusPlantPlacementKind,
    pub age: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndPodiumBlockKind {
    Bedrock,
    EndStone,
    Air,
    EndPortal,
    WallTorch(HorizontalDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndPodiumPlacementBlock {
    pub pos: BlockPos,
    pub kind: EndPodiumBlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndSpikeModel {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: i32,
    pub height: i32,
    pub guarded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSpikeConfigurationModel {
    pub crystal_invulnerable: bool,
    pub spikes: Vec<EndSpikeModel>,
    pub crystal_beam_target: Option<BlockPos>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IronBarsConnectionDirection {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndSpikeBlockKind {
    Obsidian,
    Air,
    Bedrock,
    Fire,
    IronBars {
        north: bool,
        south: bool,
        west: bool,
        east: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndSpikePlacementBlock {
    pub pos: BlockPos,
    pub kind: EndSpikeBlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndCrystalPlacement {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub beam_target: Option<BlockPos>,
    pub invulnerable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HugeFungusConfigurationModel {
    pub valid_base_state: &'static str,
    pub stem_state: &'static str,
    pub hat_state: &'static str,
    pub decor_state: &'static str,
    pub planted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeFungusStemKind {
    Stem,
    CornerStem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugeFungusStemBlock {
    pub pos: BlockPos,
    pub kind: HugeFungusStemKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeFungusHatRole {
    Bottom,
    Inside,
    Corner,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugeFungusHatCell {
    pub pos: BlockPos,
    pub role: HugeFungusHatRole,
    pub radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugeFungusHatPlacement {
    None,
    Hat,
    Decor,
    HatWithWeepingVines,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDirection {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPileConfigurationModel {
    pub state_provider: BlockStateProviderModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskConfigurationModel {
    pub state_provider: BlockStateProviderModel,
    pub target: BlockPredicate,
    pub radius: i32,
    pub half_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskPlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub mark_above_for_post_processing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowAndFreezeColumn {
    pub x: i32,
    pub z: i32,
    pub motion_blocking_height: i32,
    pub should_freeze: bool,
    pub should_snow: bool,
    pub below_has_snowy_property: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnowAndFreezePlacement {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnderwaterMagmaConfigurationModel {
    pub floor_search_range: i32,
    pub placement_radius_around_floor: i32,
    pub placement_probability_per_valid_position: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnderwaterMagmaCandidate {
    pub pos: BlockPos,
    pub block: &'static str,
    pub below_visible_from_above: bool,
    pub horizontal_visible_from_outside: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSizeModel {
    TwoLayers {
        limit: i32,
        lower_size: i32,
        upper_size: i32,
        min_clipped_height: Option<i32>,
    },
    ThreeLayers {
        limit: i32,
        upper_limit: i32,
        lower_size: i32,
        middle_size: i32,
        upper_size: i32,
        min_clipped_height: Option<i32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrunkPlacerKind {
    Straight,
    Forking,
    Giant,
    MegaJungle,
    DarkOak,
    Fancy,
    Bending {
        min_height_for_leaves: i32,
        bend_length_min: i32,
        bend_length_max: i32,
    },
    UpwardsBranching {
        place_branch_per_log_probability: f32,
        extra_branch_steps_min: i32,
        extra_branch_length_min: i32,
    },
    Cherry {
        branch_count_min: i32,
        branch_count_max: i32,
        branch_horizontal_length_min: i32,
        branch_horizontal_length_max: i32,
        branch_start_offset_from_top_min: i32,
        branch_start_offset_from_top_max: i32,
        branch_end_offset_from_top_min: i32,
        branch_end_offset_from_top_max: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrunkPlacerModel {
    pub base_height: i32,
    pub height_rand_a: i32,
    pub height_rand_b: i32,
    pub kind: TrunkPlacerKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoliagePlacerKind {
    Blob {
        height: i32,
    },
    Spruce {
        height_min: i32,
        height_max: i32,
    },
    Pine {
        height_min: i32,
        height_max: i32,
    },
    Acacia,
    Bush {
        height: i32,
    },
    Fancy {
        height: i32,
    },
    Jungle {
        height: i32,
    },
    MegaPine {
        height_min: i32,
        height_max: i32,
    },
    DarkOak,
    RandomSpread {
        foliage_height_min: i32,
        foliage_height_max: i32,
        leaf_placement_attempts: i32,
    },
    Cherry {
        height: i32,
        wide_bottom_layer_hole_chance: f32,
        corner_hole_chance: f32,
        hanging_leaves_chance: f32,
        hanging_leaves_extension_chance: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FoliagePlacerModel {
    pub radius_min: i32,
    pub radius_max: i32,
    pub offset_min: i32,
    pub offset_max: i32,
    pub kind: FoliagePlacerKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MangroveRootPlacementModel {
    pub max_root_width: i32,
    pub max_root_length: i32,
    pub random_skew_chance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RootPlacerModel {
    pub above_root_placement_chance: Option<f32>,
    pub mangrove_root_placement: MangroveRootPlacementModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreePlacementBlockKind {
    DirtBelowTrunk,
    Log,
    Leaves,
    GroundCover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreePlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub kind: TreePlacementBlockKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreePlacementPlan {
    pub blocks: Vec<TreePlacementBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallenTreeConfigurationModel {
    pub trunk_provider: BlockStateProviderModel,
    pub min_log_length: i32,
    pub max_log_length: i32,
    pub stump_decorators: Vec<TreeDecoratorModel>,
    pub log_decorators: Vec<TreeDecoratorModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallenTreeBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub mark_above_for_post_processing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallenTreePlacementPlan {
    pub blocks: Vec<FallenTreeBlock>,
    pub stump_decorators: usize,
    pub log_decorators: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TreeDecoratorModel {
    TrunkVine,
    LeaveVine,
    PaleMoss,
    CreakingHeart,
    Cocoa { probability: f32 },
    Beehive { probability: f32 },
    AlterGround,
    AttachedToLeaves { probability: f32 },
    PlaceOnGround,
    AttachedToLogs { probability: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterRoomBounds {
    pub min_y: i32,
    pub max_y: i32,
    pub min_openings: i32,
    pub max_openings: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterRoomRadii {
    pub x_radius: i32,
    pub z_radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterRoomProbe {
    pub dx: i32,
    pub dy: i32,
    pub dz: i32,
    pub solid: bool,
    pub empty: bool,
    pub above_empty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterRoomShellBlock {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSetEntry {
    pub id: &'static str,
    pub structures: &'static [&'static str],
    pub placement: StructurePlacementKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePlacementKind {
    RandomSpread {
        spacing: i32,
        separation: i32,
        salt: i32,
        spread_type: RandomSpreadType,
    },
    ConcentricRings {
        distance: i32,
        spread: i32,
        count: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcentricRingPlacementCandidate {
    pub index: i32,
    pub circle: i32,
    pub chunk_pos: ChunkPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureExclusionZoneModel {
    pub other_set: &'static str,
    pub chunk_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureBoundingBoxModel {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePieceModel {
    pub bounding_box: StructureBoundingBoxModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePiecePlacementBlock {
    pub local_pos: BlockPos,
    pub world_pos: BlockPos,
    pub state: &'static str,
    pub edge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureStartModel {
    pub structure: Option<&'static str>,
    pub chunk_pos: ChunkPos,
    pub references: i32,
    pub pieces: Vec<StructurePieceModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureStartTagModel {
    pub id: &'static str,
    pub chunk_x: Option<i32>,
    pub chunk_z: Option<i32>,
    pub references: Option<i32>,
    pub children: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureAccessModel {
    pub starts: BTreeMap<&'static str, StructureStartModel>,
    pub references: BTreeMap<&'static str, Vec<i64>>,
    pub unsaved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainAdjustmentModel {
    None,
    Bury,
    BeardThin,
    BeardBox,
    Encapsulate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureCheckResultModel {
    StartPresent,
    StartNotPresent,
    ChunkLoadNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomSpreadType {
    Linear,
    Triangular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrequencyReductionMethod {
    Default,
    LegacyType1,
    LegacyType2,
    LegacyType3,
}

impl RandomSpreadType {
    pub fn evaluate(self, random: &mut LegacyRandom, limit: i32) -> i32 {
        match self {
            RandomSpreadType::Linear => random.next_i32_bound(limit),
            RandomSpreadType::Triangular => {
                (random.next_i32_bound(limit) + random.next_i32_bound(limit)) / 2
            }
        }
    }
}

impl FrequencyReductionMethod {
    pub fn id(self) -> &'static str {
        match self {
            FrequencyReductionMethod::Default => "default",
            FrequencyReductionMethod::LegacyType1 => "legacy_type_1",
            FrequencyReductionMethod::LegacyType2 => "legacy_type_2",
            FrequencyReductionMethod::LegacyType3 => "legacy_type_3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureFamilyEntry {
    pub family: StructureFamily,
    pub structures: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawPoolBootstrapSource {
    pub source_file: &'static str,
    pub registrations: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendingConstants {
    pub height_blending_range_cells: i32,
    pub height_blending_range_chunks: i32,
    pub density_blending_range_cells: i32,
    pub density_blending_range_chunks: i32,
    pub old_chunk_xz_radius: i32,
    pub cell_width: i32,
    pub cell_height: i32,
    pub cell_ratio: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendingDataPacked<'a> {
    pub min_section: i32,
    pub max_section: i32,
    pub heights: Option<&'a [f64]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendingOutput {
    pub alpha: f64,
    pub blending_offset: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpgradeDataModel {
    pub tag_indices: &'static str,
    pub tag_sides: &'static str,
    pub tag_neighbor_block_ticks: &'static str,
    pub tag_neighbor_fluid_ticks: &'static str,
    pub block_fixers: &'static [&'static str],
    pub chunky_fixers: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnSelectionConstants {
    pub initial_chunk_search_radius: i32,
    pub player_spawn_ticket_radius: i32,
    pub spawn_search_absolute_max_attempts: i32,
    pub default_respawn_radius: i32,
    pub small_search_coprime_threshold: i32,
    pub large_search_coprime: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnColumnHeights {
    pub top_y: i32,
    pub surface_y: i32,
    pub ocean_floor_y: i32,
    pub min_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnBlockKind {
    Solid,
    Air,
    Fluid,
    NonSolid,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitialSpawnKind {
    DebugHalfWorld { x: i32, y: i32, z: i32 },
    DebugWorld { x: i32, y: i32, z: i32 },
    Normal { x: i32, y: i32, z: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureFamily {
    Village,
    Stronghold,
    Mineshaft,
    OceanMonument,
    WoodlandMansion,
    Bastion,
    Fortress,
    AncientCity,
    TrialChambers,
    EndCity,
    RuinedPortal,
    Shipwreck,
    BuriedTreasure,
    Igloo,
    SwampHut,
    PillagerOutpost,
    TrailRuins,
    Fossil,
    DesertPyramid,
    JungleTemple,
    OceanRuins,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureConfigurationKind {
    None,
    Tree,
    FallenTree,
    BlockPile,
    Spring,
    ReplaceBlock,
    Fossil,
    HugeMushroom,
    Spike,
    BlockColumn,
    VegetationPatch,
    RootSystem,
    MultifaceGrowth,
    UnderwaterMagma,
    BlockState,
    BlockBlob,
    Disk,
    Lake,
    Ore,
    EndSpike,
    EndGateway,
    Probability,
    Count,
    SimpleBlock,
    HugeFungus,
    NetherForestVegetation,
    TwistingVines,
    Column,
    Delta,
    ReplaceSphere,
    Layer,
    RandomFeature,
    SimpleRandomFeature,
    RandomBooleanFeature,
    Geode,
    DripstoneCluster,
    LargeDripstone,
    PointedDripstone,
    SculkPatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureFamily {
    Selector,
    Tree,
    Vegetation,
    Cave,
    Ore,
    Lake,
    Disk,
    End,
    Nether,
    Aquatic,
    StructureLike,
    BlockPlacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseRouterPreset {
    Overworld { large_biomes: bool, amplified: bool },
    Nether,
    End,
    Caves,
    FloatingIslands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRulePreset {
    Overworld,
    Nether,
    End,
    OverworldLike {
        bedrock_roof: bool,
        bedrock_floor: bool,
        surface: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DensityFunction {
    Reference(&'static str),
    Constant(f64),
    YClampedGradient {
        from_y: i32,
        to_y: i32,
        from_value: f64,
        to_value: f64,
    },
    Clamp {
        input: &'static DensityFunction,
        min: f64,
        max: f64,
    },
    Mapped {
        kind: MappedDensityFunction,
        input: &'static DensityFunction,
    },
    Binary {
        kind: BinaryDensityFunction,
        argument1: &'static DensityFunction,
        argument2: &'static DensityFunction,
    },
    Marker {
        kind: DensityMarker,
        input: &'static DensityFunction,
    },
    Noise {
        noise: &'static str,
        xz_scale: f64,
        y_scale: f64,
    },
    ShiftedNoise {
        shift_x: &'static DensityFunction,
        shift_y: &'static DensityFunction,
        shift_z: &'static DensityFunction,
        xz_scale: f64,
        y_scale: f64,
        noise: &'static str,
    },
    BlendedNoise {
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale_multiplier: f64,
    },
    EndIslands {
        seed: i64,
    },
    WeirdScaledSampler {
        input: &'static DensityFunction,
        noise: &'static str,
        rarity_mapper: RarityValueMapper,
    },
    BlendAlpha,
    BlendOffset,
    BlendDensity {
        input: &'static DensityFunction,
    },
    Beardifier,
    Spline,
    FindTopSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedDensityFunction {
    Abs,
    Square,
    Cube,
    HalfNegative,
    QuarterNegative,
    Invert,
    Squeeze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryDensityFunction {
    Add,
    Mul,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityMarker {
    Interpolated,
    FlatCache,
    Cache2D,
    CacheOnce,
    CacheAllInCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RarityValueMapper {
    Type1,
    Type2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityFunctionType {
    pub id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DensityFunctionEntry {
    pub id: &'static str,
    pub function: DensityFunction,
}

pub const OVERWORLD_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 384, 1, 2);
pub const NETHER_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 1, 2);
pub const END_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 128, 2, 1);
pub const CAVES_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(-64, 192, 1, 2);
pub const FLOATING_ISLANDS_NOISE_SETTINGS: NoiseSettings = NoiseSettings::new(0, 256, 2, 1);

pub const NORMAL_NOISE_INPUT_FACTOR: f64 = 1.0181268882175227;
pub const NORMAL_NOISE_TARGET_DEVIATION: f64 = 1.0 / 3.0;

pub const SYNTH_NOISE_SOURCES: &[SynthNoiseSource] = &[
    SynthNoiseSource {
        id: "minecraft:normal_noise",
        codec: "NormalNoise.NoiseParameters",
    },
    SynthNoiseSource {
        id: "minecraft:perlin_noise",
        codec: "PerlinNoise",
    },
    SynthNoiseSource {
        id: "minecraft:perlin_simplex_noise",
        codec: "PerlinSimplexNoise",
    },
    SynthNoiseSource {
        id: "minecraft:simplex_noise",
        codec: "SimplexNoise",
    },
    SynthNoiseSource {
        id: "minecraft:improved_noise",
        codec: "ImprovedNoise",
    },
    SynthNoiseSource {
        id: "minecraft:blended_noise",
        codec: "DensityFunctions.BlendedNoise",
    },
];

pub const NORMAL_NOISE_PARAMETERS: &[NormalNoiseParameters] = &[
    NormalNoiseParameters {
        id: "minecraft:temperature",
        first_octave: -10,
        amplitudes: &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:vegetation",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:continentalness",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:erosion",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 0.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:temperature_large",
        first_octave: -12,
        amplitudes: &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:vegetation_large",
        first_octave: -10,
        amplitudes: &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:continentalness_large",
        first_octave: -11,
        amplitudes: &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:erosion_large",
        first_octave: -11,
        amplitudes: &[1.0, 1.0, 0.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:nether/temperature",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:nether/vegetation",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ridge",
        first_octave: -7,
        amplitudes: &[1.0, 2.0, 1.0, 0.0, 0.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:offset",
        first_octave: -3,
        amplitudes: &[1.0, 1.0, 1.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_barrier",
        first_octave: -3,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_fluid_level_floodedness",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_lava",
        first_octave: -1,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:aquifer_fluid_level_spread",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar",
        first_octave: -7,
        amplitudes: &[1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar_rareness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:pillar_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_elevation",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_modulator",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_2d_thickness",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_1",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_2",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_rarity",
        first_octave: -11,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_3d_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_roughness",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:spaghetti_roughness_modulator",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_entrance",
        first_octave: -7,
        amplitudes: &[0.4, 0.5, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_layer",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:cave_cheese",
        first_octave: -8,
        amplitudes: &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_veininess",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_vein_a",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_vein_b",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ore_gap",
        first_octave: -5,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_thickness",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_ridge_a",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:noodle_ridge_b",
        first_octave: -7,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:jagged",
        first_octave: -16,
        amplitudes: &[
            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ],
    },
    NormalNoiseParameters {
        id: "minecraft:surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:surface_secondary",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 0.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:clay_bands_offset",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_pillar",
        first_octave: -2,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_pillar_roof",
        first_octave: -8,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:badlands_surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_pillar",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_pillar_roof",
        first_octave: -3,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:iceberg_surface",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:surface_swamp",
        first_octave: -2,
        amplitudes: &[1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:calcite",
        first_octave: -9,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:gravel",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:powder_snow",
        first_octave: -6,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:packed_ice",
        first_octave: -7,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:ice",
        first_octave: -4,
        amplitudes: &[1.0, 1.0, 1.0, 1.0],
    },
    NormalNoiseParameters {
        id: "minecraft:soul_sand_layer",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:gravel_layer",
        first_octave: -8,
        amplitudes: &[1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:patch",
        first_octave: -5,
        amplitudes: &[1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334],
    },
    NormalNoiseParameters {
        id: "minecraft:netherrack",
        first_octave: -3,
        amplitudes: &[1.0, 0.0, 0.0, 0.35],
    },
    NormalNoiseParameters {
        id: "minecraft:nether_wart",
        first_octave: -3,
        amplitudes: &[1.0, 0.0, 0.0, 0.9],
    },
    NormalNoiseParameters {
        id: "minecraft:nether_state_selector",
        first_octave: -4,
        amplitudes: &[1.0],
    },
];

pub const HEIGHT_PROVIDER_TYPES: &[HeightProviderType] = &[
    HeightProviderType {
        id: "minecraft:constant",
    },
    HeightProviderType {
        id: "minecraft:uniform",
    },
    HeightProviderType {
        id: "minecraft:biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:very_biased_to_bottom",
    },
    HeightProviderType {
        id: "minecraft:trapezoid",
    },
    HeightProviderType {
        id: "minecraft:weighted_list",
    },
];

pub const BLOCK_PREDICATE_TYPES: &[BlockPredicateType] = &[
    BlockPredicateType {
        id: "minecraft:matching_blocks",
    },
    BlockPredicateType {
        id: "minecraft:matching_block_tag",
    },
    BlockPredicateType {
        id: "minecraft:matching_fluids",
    },
    BlockPredicateType {
        id: "minecraft:has_sturdy_face",
    },
    BlockPredicateType {
        id: "minecraft:solid",
    },
    BlockPredicateType {
        id: "minecraft:replaceable",
    },
    BlockPredicateType {
        id: "minecraft:would_survive",
    },
    BlockPredicateType {
        id: "minecraft:inside_world_bounds",
    },
    BlockPredicateType {
        id: "minecraft:any_of",
    },
    BlockPredicateType {
        id: "minecraft:all_of",
    },
    BlockPredicateType {
        id: "minecraft:not",
    },
    BlockPredicateType {
        id: "minecraft:true",
    },
    BlockPredicateType {
        id: "minecraft:unobstructed",
    },
];

pub const FLAT_DEFAULT_LAYERS: &[FlatLayerInfo] = &[
    FlatLayerInfo {
        height: 1,
        block: "minecraft:bedrock",
    },
    FlatLayerInfo {
        height: 2,
        block: "minecraft:dirt",
    },
    FlatLayerInfo {
        height: 1,
        block: "minecraft:grass_block",
    },
];

pub const FLAT_GENERATOR_PRESETS: &[FlatGeneratorPreset] = &[
    FlatGeneratorPreset {
        id: "minecraft:classic_flat",
        display: "minecraft:grass_block",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 2,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:tunnelers_dream",
        display: "minecraft:stone",
        biome: "minecraft:windswept_hills",
        structures: &["minecraft:mineshafts", "minecraft:strongholds"],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 230,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:water_world",
        display: "minecraft:water_bucket",
        biome: "minecraft:deep_ocean",
        structures: &[
            "minecraft:ocean_ruins",
            "minecraft:shipwrecks",
            "minecraft:ocean_monuments",
        ],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 64,
                block: "minecraft:deepslate",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 5,
                block: "minecraft:gravel",
            },
            FlatLayerInfo {
                height: 90,
                block: "minecraft:water",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:overworld",
        display: "minecraft:short_grass",
        biome: "minecraft:plains",
        structures: &[
            "minecraft:villages",
            "minecraft:mineshafts",
            "minecraft:pillager_outposts",
            "minecraft:ruined_portals",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: true,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:snowy_kingdom",
        display: "minecraft:snow",
        biome: "minecraft:snowy_plains",
        structures: &["minecraft:villages", "minecraft:igloos"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 59,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:snow",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:bottomless_pit",
        display: "minecraft:feather",
        biome: "minecraft:plains",
        structures: &["minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 2,
                block: "minecraft:cobblestone",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:dirt",
            },
            FlatLayerInfo {
                height: 1,
                block: "minecraft:grass_block",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:desert",
        display: "minecraft:sand",
        biome: "minecraft:desert",
        structures: &[
            "minecraft:villages",
            "minecraft:desert_pyramids",
            "minecraft:mineshafts",
            "minecraft:strongholds",
        ],
        add_lakes: true,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 52,
                block: "minecraft:sandstone",
            },
            FlatLayerInfo {
                height: 8,
                block: "minecraft:sand",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:redstone_ready",
        display: "minecraft:redstone",
        biome: "minecraft:desert",
        structures: &[],
        add_lakes: false,
        decoration: false,
        layers: &[
            FlatLayerInfo {
                height: 1,
                block: "minecraft:bedrock",
            },
            FlatLayerInfo {
                height: 3,
                block: "minecraft:stone",
            },
            FlatLayerInfo {
                height: 116,
                block: "minecraft:sandstone",
            },
        ],
    },
    FlatGeneratorPreset {
        id: "minecraft:the_void",
        display: "minecraft:barrier",
        biome: "minecraft:the_void",
        structures: &[],
        add_lakes: true,
        decoration: false,
        layers: &[FlatLayerInfo {
            height: 1,
            block: "minecraft:air",
        }],
    },
];

pub const VANILLA_DIMENSION_Y_SIZE: i32 = 384;
pub const FLAT_GENERATOR_MIN_Y: i32 = 0;
pub const FLAT_GENERATOR_GEN_DEPTH: i32 = 384;
pub const FLAT_GENERATOR_SEA_LEVEL: i32 = -63;

pub fn flat_generator_settings(
    preset: &FlatGeneratorPreset,
) -> Result<FlatGeneratorSettingsModel, String> {
    let expanded_raw = expand_flat_layers(preset.layers)?;
    let mut expanded_layers = expanded_raw.clone();
    let mut top_layer_modifications = Vec::new();
    let void_generation = expanded_layers
        .iter()
        .all(|state| state.map(|block| block == "minecraft:air").unwrap_or(true));

    for (y, state) in expanded_layers.iter_mut().enumerate() {
        let Some(block) = *state else {
            continue;
        };
        if !motion_blocking_block(block) {
            *state = None;
            top_layer_modifications.push((y, block));
        }
    }

    Ok(FlatGeneratorSettingsModel {
        biome: preset.biome,
        structure_overrides: preset.structures.to_vec(),
        add_lakes: preset.add_lakes,
        decoration: preset.decoration,
        layers: preset.layers.to_vec(),
        expanded_layers,
        top_layer_modifications,
        void_generation,
    })
}

pub fn default_flat_generator_settings() -> Result<FlatGeneratorSettingsModel, String> {
    let expanded_layers = expand_flat_layers(FLAT_DEFAULT_LAYERS)?;
    Ok(FlatGeneratorSettingsModel {
        biome: "minecraft:plains",
        structure_overrides: vec!["minecraft:strongholds", "minecraft:villages"],
        add_lakes: false,
        decoration: false,
        layers: FLAT_DEFAULT_LAYERS.to_vec(),
        expanded_layers,
        top_layer_modifications: Vec::new(),
        void_generation: false,
    })
}

pub fn expand_flat_layers(layers: &[FlatLayerInfo]) -> Result<Vec<Option<&'static str>>, String> {
    let total_height = layers.iter().try_fold(0_i32, |sum, layer| {
        if layer.height < 0 {
            Err(format!(
                "flat layer {} has negative height {}",
                layer.block, layer.height
            ))
        } else {
            Ok(sum + layer.height)
        }
    })?;
    if total_height > VANILLA_DIMENSION_Y_SIZE {
        return Err(format!(
            "Sum of layer heights is > {}",
            VANILLA_DIMENSION_Y_SIZE
        ));
    }

    let mut expanded = Vec::with_capacity(total_height as usize);
    for layer in layers {
        for _ in 0..layer.height {
            expanded.push(Some(layer.block));
        }
    }
    Ok(expanded)
}

pub fn flat_base_height(
    layers: &[Option<&'static str>],
    min_y: i32,
    height: i32,
    heightmap: HeightmapKind,
) -> i32 {
    let max_layer = layers.len().min(height.max(0) as usize);
    for layer_index in (0..max_layer).rev() {
        if layers[layer_index].is_some_and(|block| heightmap_opaque(heightmap, block)) {
            return min_y + layer_index as i32 + 1;
        }
    }
    min_y
}

pub fn flat_base_column(
    layers: &[Option<&'static str>],
    min_y: i32,
    height: i32,
) -> FlatNoiseColumn {
    let states = (0..height.max(0) as usize)
        .map(|index| {
            layers
                .get(index)
                .and_then(|state| *state)
                .unwrap_or("minecraft:air")
        })
        .collect();
    FlatNoiseColumn { min_y, states }
}

pub fn materialize_flat_chunk(pos: ChunkPos, settings: &FlatGeneratorSettingsModel) -> LevelChunk {
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:full".to_string();

    let layers = &settings.expanded_layers;
    let max_layer = layers
        .iter()
        .rposition(Option::is_some)
        .map(|index| index + 1)
        .unwrap_or(0);
    let section_count = max_layer.div_ceil(16);
    chunk.sections = (0..section_count)
        .map(|section_index| {
            let section_min_y = FLAT_GENERATOR_MIN_Y + section_index as i32 * 16;
            let block_states = flat_section_block_states(layers, section_index);
            ChunkSection {
                y: (section_min_y / 16) as i8,
                block_states: block_states.to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String(settings.biome.to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            }
        })
        .collect();

    let world_surface = flat_base_height(
        layers,
        FLAT_GENERATOR_MIN_Y,
        FLAT_GENERATOR_GEN_DEPTH,
        HeightmapKind::WorldSurfaceWg,
    );
    let ocean_floor = flat_base_height(
        layers,
        FLAT_GENERATOR_MIN_Y,
        FLAT_GENERATOR_GEN_DEPTH,
        HeightmapKind::OceanFloorWg,
    );
    chunk.heightmaps = BTreeMap::from([
        (
            HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap([world_surface; 16 * 16])),
        ),
        (
            HeightmapKind::OceanFloorWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap([ocean_floor; 16 * 16])),
        ),
    ]);
    chunk
}

pub fn build_features_per_step(
    feature_sources: &[&[&[&'static str]]],
    try_reducing_error: bool,
) -> Result<Vec<StepFeatureDataModel>, String> {
    use std::collections::{BTreeMap, BTreeSet};

    let mut feature_indices = BTreeMap::<&'static str, usize>::new();
    let mut next_feature_index = 0_usize;
    let mut edges = BTreeMap::<FeatureSorterData, BTreeSet<FeatureSorterData>>::new();
    let mut max_step = 0_usize;

    for features_for_step in feature_sources {
        max_step = max_step.max(features_for_step.len());
        let mut feature_list = Vec::new();
        for (step, features) in features_for_step.iter().enumerate() {
            for feature in *features {
                let feature_index = *feature_indices.entry(*feature).or_insert_with(|| {
                    let index = next_feature_index;
                    next_feature_index += 1;
                    index
                });
                feature_list.push(FeatureSorterData {
                    feature_index,
                    step,
                    feature,
                });
            }
        }

        for (index, feature) in feature_list.iter().copied().enumerate() {
            let data = edges.entry(feature).or_default();
            if let Some(next) = feature_list.get(index + 1) {
                data.insert(*next);
            }
        }
    }

    let mut discovered = BTreeSet::new();
    let mut currently_visiting = BTreeSet::new();
    let mut sorted_features = Vec::new();
    for feature in edges.keys().copied().collect::<Vec<_>>() {
        if !currently_visiting.is_empty() {
            return Err(
                "You somehow broke the universe; DFS bork (iteration finished with non-empty in-progress vertex set"
                    .to_string(),
            );
        }
        if !discovered.contains(&feature)
            && feature_sorter_dfs(
                feature,
                &edges,
                &mut discovered,
                &mut currently_visiting,
                &mut sorted_features,
            )
        {
            return if try_reducing_error {
                Err(format!(
                    "Feature order cycle found, involved sources: {}",
                    feature_sources.len()
                ))
            } else {
                Err("Feature order cycle found".to_string())
            };
        }
    }

    sorted_features.reverse();
    Ok((0..max_step)
        .map(|step| StepFeatureDataModel {
            features: sorted_features
                .iter()
                .filter(|feature| feature.step == step)
                .map(|feature| feature.feature)
                .collect(),
        })
        .collect())
}

fn feature_sorter_dfs(
    feature: FeatureSorterData,
    edges: &std::collections::BTreeMap<
        FeatureSorterData,
        std::collections::BTreeSet<FeatureSorterData>,
    >,
    discovered: &mut std::collections::BTreeSet<FeatureSorterData>,
    currently_visiting: &mut std::collections::BTreeSet<FeatureSorterData>,
    sorted_features: &mut Vec<FeatureSorterData>,
) -> bool {
    if discovered.contains(&feature) {
        return false;
    }
    if !currently_visiting.insert(feature) {
        return true;
    }
    for child in edges.get(&feature).into_iter().flatten().copied() {
        if feature_sorter_dfs(
            child,
            edges,
            discovered,
            currently_visiting,
            sorted_features,
        ) {
            return true;
        }
    }
    currently_visiting.remove(&feature);
    discovered.insert(feature);
    sorted_features.push(feature);
    false
}

pub fn materialize_noise_preview_chunk(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
) -> LevelChunk {
    let mut chunk = LevelChunk::empty(pos);
    chunk.status = "minecraft:full".to_string();

    let min_y = settings.noise.min_y;
    let max_y = settings.noise.min_y + settings.noise.height;
    let min_section = min_y.div_euclid(16);
    let section_count = (settings.noise.height + 15) / 16;
    let mut terrain_heights = [settings.sea_level + 1; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            let world_x = pos.x * 16 + x as i32;
            let world_z = pos.z * 16 + z as i32;
            terrain_heights[z * 16 + x] =
                noise_preview_terrain_height(world_x, world_z, settings).clamp(min_y + 1, max_y);
        }
    }
    let biome = noise_preview_biome(biome_source_model, pos);
    let mut overlay_blocks = noise_preview_tree_blocks(pos, settings, biome, &terrain_heights);
    overlay_blocks.extend(noise_preview_ground_cover_blocks(
        pos,
        settings,
        biome,
        &terrain_heights,
    ));

    chunk.sections = (0..section_count)
        .map(|section_offset| {
            let section_y = min_section + section_offset;
            ChunkSection {
                y: section_y as i8,
                block_states: noise_preview_section_block_states(
                    section_y,
                    min_y,
                    settings,
                    &terrain_heights,
                    &overlay_blocks,
                )
                .to_nbt(),
                biomes: PalettedContainer::single(
                    Tag::String(biome.to_string()),
                    BIOME_SECTION_VOLUME,
                )
                .to_nbt(),
                block_light: None,
                sky_light: Some(vec![-1; 2048]),
            }
        })
        .collect();

    let mut world_surface = terrain_heights.map(|height| height.max(settings.sea_level + 1));
    for block in &overlay_blocks {
        if (0..16).contains(&block.pos.x) && (0..16).contains(&block.pos.z) {
            let index = block.pos.z as usize * 16 + block.pos.x as usize;
            world_surface[index] = world_surface[index].max(block.pos.y + 1);
        }
    }
    chunk.heightmaps = BTreeMap::from([
        (
            HeightmapKind::WorldSurfaceWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(world_surface)),
        ),
        (
            HeightmapKind::OceanFloorWg.storage_name().to_string(),
            Tag::LongArray(pack_heightmap(terrain_heights)),
        ),
    ]);
    chunk
}

fn noise_preview_section_block_states(
    section_y: i32,
    min_y: i32,
    settings: &NoiseGeneratorSettings,
    surface_heights: &[i32; 16 * 16],
    tree_blocks: &[TreePlacementBlock],
) -> PalettedContainer {
    let mut palette: Vec<&'static str> = Vec::new();
    let mut indices = vec![0_u64; SECTION_VOLUME];
    for local_y in 0..16 {
        let world_y = section_y * 16 + local_y as i32;
        for z in 0..16 {
            for x in 0..16 {
                let surface_height = surface_heights[z * 16 + x];
                let block = tree_blocks
                    .iter()
                    .find(|block| {
                        block.pos.x == x as i32 && block.pos.y == world_y && block.pos.z == z as i32
                    })
                    .map(|block| block.state)
                    .unwrap_or_else(|| {
                        noise_preview_block_at(world_y, min_y, surface_height, settings.sea_level)
                    });
                let palette_index = match palette.iter().position(|entry| *entry == block) {
                    Some(index) => index as u64,
                    None => {
                        palette.push(block);
                        (palette.len() - 1) as u64
                    }
                };
                indices[(local_y << 8) | (z << 4) | x] = palette_index;
            }
        }
    }

    if palette.len() == 1 {
        return PalettedContainer::single(block_state_tag(palette[0]), SECTION_VOLUME);
    }

    PalettedContainer {
        palette: palette.into_iter().map(block_state_tag).collect(),
        data: Some(pack_palette_indices(
            &indices,
            bits_for_palette(indices.iter().copied().max().unwrap_or(0) + 1),
        )),
        expected_entries: SECTION_VOLUME,
    }
}

fn noise_preview_block_at(
    world_y: i32,
    min_y: i32,
    surface_height: i32,
    sea_level: i32,
) -> &'static str {
    if world_y <= min_y {
        "minecraft:bedrock"
    } else if world_y >= surface_height {
        if world_y <= sea_level {
            "minecraft:water"
        } else {
            "minecraft:air"
        }
    } else if world_y == surface_height - 1 {
        if surface_height <= sea_level + 1 {
            "minecraft:sand"
        } else {
            "minecraft:grass_block"
        }
    } else if world_y >= surface_height - 4 {
        if surface_height <= sea_level + 1 {
            "minecraft:sandstone"
        } else {
            "minecraft:dirt"
        }
    } else if world_y < min_y + 5 {
        "minecraft:deepslate"
    } else {
        "minecraft:stone"
    }
}

fn noise_preview_tree_blocks(
    chunk_pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    biome: &str,
    terrain_heights: &[i32; 16 * 16],
) -> Vec<TreePlacementBlock> {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return Vec::new();
    }

    let Some(generation) = biome_generation_settings(biome) else {
        return Vec::new();
    };
    if !biome_has_placed_feature(generation, "trees_plains")
        && !biome_has_placed_feature(generation, "trees_birch_and_oak_leaf_litter")
    {
        return Vec::new();
    }

    let seed = (chunk_pos.x as i64 * 341_873_128_712 + chunk_pos.z as i64 * 132_897_987_541) as u64;
    let mut blocks = Vec::new();
    for (index, (local_x, local_z)) in noise_preview_tree_origins(generation, seed)
        .into_iter()
        .enumerate()
    {
        let surface_height = terrain_heights[local_z * 16 + local_x];
        if surface_height <= settings.sea_level + 2 {
            continue;
        }
        let tree_seed = seed.rotate_left((index as u32 + 1) * 7);
        let forest_tree = biome_has_placed_feature(generation, "trees_birch_and_oak_leaf_litter")
            && (tree_seed & 1) == 0;
        let plan = simple_tree_placement_plan(
            BlockPos {
                x: local_x as i32,
                y: surface_height,
                z: local_z as i32,
            },
            TrunkPlacerModel {
                base_height: if forest_tree { 5 } else { 4 },
                height_rand_a: 2,
                height_rand_b: 1,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Blob { height: 3 },
            },
            if forest_tree {
                "minecraft:birch_log"
            } else {
                "minecraft:oak_log"
            },
            if forest_tree {
                "minecraft:birch_leaves"
            } else {
                "minecraft:oak_leaves"
            },
            "minecraft:dirt",
            (tree_seed & 0xffff) as i32,
            ((tree_seed >> 16) & 0xffff) as i32,
        )
        .expect("hard-coded preview tree configuration must validate");
        blocks.extend(plan.blocks);
    }
    blocks
        .into_iter()
        .filter(|block| {
            (0..16).contains(&block.pos.x)
                && (settings.noise.min_y..settings.noise.min_y + settings.noise.height)
                    .contains(&block.pos.y)
                && (0..16).contains(&block.pos.z)
        })
        .collect()
}

fn noise_preview_tree_origins(
    biome: &BiomeGenerationSettingsModel,
    seed: u64,
) -> Vec<(usize, usize)> {
    if biome_has_placed_feature(biome, "trees_birch_and_oak_leaf_litter") {
        let candidates = [(4, 4), (11, 5), (6, 12), (13, 13)];
        return candidates
            .into_iter()
            .enumerate()
            .filter_map(|(index, pos)| {
                if seed.rotate_left(5 + index as u32 * 9) % 3 != 1 {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect();
    }

    if biome_has_placed_feature(biome, "trees_plains") && seed.rotate_left(13) % 5 == 0 {
        vec![(8, 8)]
    } else {
        Vec::new()
    }
}

fn noise_preview_ground_cover_blocks(
    chunk_pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    biome: &str,
    terrain_heights: &[i32; 16 * 16],
) -> Vec<TreePlacementBlock> {
    if settings.id != "minecraft:overworld" && settings.id != "minecraft:large_biomes" {
        return Vec::new();
    }

    let Some(generation) = biome_generation_settings(biome) else {
        return Vec::new();
    };
    let grass = biome_has_placed_feature(generation, "patch_grass_plain")
        || biome_has_placed_feature(generation, "patch_grass_forest");
    let flowers = biome_has_placed_feature(generation, "flower_plains")
        || biome_has_placed_feature(generation, "flower_default")
        || biome_has_placed_feature(generation, "forest_flowers");
    let sunflowers = biome_has_placed_feature(generation, "patch_sunflower");
    if !grass && !flowers && !sunflowers {
        return Vec::new();
    }

    let seed = (chunk_pos.x as i64 * 341_873_128_712 + chunk_pos.z as i64 * 132_897_987_541) as u64;
    let mut blocks = Vec::new();
    for z in 0..16 {
        for x in 0..16 {
            let surface_height = terrain_heights[z * 16 + x];
            if surface_height <= settings.sea_level + 1 {
                continue;
            }
            let roll = noise_preview_cover_roll(seed, x as u64, z as u64);
            let state = if sunflowers && roll % 97 == 0 {
                Some("minecraft:sunflower")
            } else if flowers && roll % 23 == 0 {
                Some(if biome == "minecraft:forest" {
                    "minecraft:poppy"
                } else {
                    "minecraft:dandelion"
                })
            } else if grass && roll % 7 == 0 {
                Some("minecraft:short_grass")
            } else {
                None
            };
            if let Some(state) = state {
                blocks.push(TreePlacementBlock {
                    pos: BlockPos {
                        x: x as i32,
                        y: surface_height,
                        z: z as i32,
                    },
                    state,
                    kind: TreePlacementBlockKind::GroundCover,
                });
            }
        }
    }
    blocks
}

fn noise_preview_cover_roll(seed: u64, x: u64, z: u64) -> u64 {
    let mut value =
        seed ^ x.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ z.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn noise_preview_terrain_height(x: i32, z: i32, settings: &NoiseGeneratorSettings) -> i32 {
    let (scale, amplitude) = match settings.id {
        "minecraft:large_biomes" => (76.0, 30.0),
        "minecraft:amplified" => (38.0, 70.0),
        "minecraft:nether" => (30.0, 24.0),
        "minecraft:end" => (52.0, 42.0),
        _ => (44.0, 34.0),
    };
    let xf = x as f64 / scale;
    let zf = z as f64 / scale;
    let broad = (xf.sin() * 0.55 + zf.cos() * 0.45) * amplitude;
    let detail = ((xf * 2.7 + zf * 1.3).sin() * (zf * 2.1 - xf * 0.9).cos()) * amplitude * 0.28;
    let ridge = ((x as i64 * 341_873_128_712 + z as i64 * 132_897_987_541) as u64).rotate_left(17)
        as f64
        / u64::MAX as f64
        - 0.5;
    settings.sea_level + 8 + (broad + detail + ridge * 12.0).round() as i32
}

pub fn noise_preview_base_height(
    x: i32,
    z: i32,
    settings: &NoiseGeneratorSettings,
    heightmap: HeightmapKind,
) -> i32 {
    let terrain_height = noise_preview_terrain_height(x, z, settings).clamp(
        settings.noise.min_y + 1,
        settings.noise.min_y + settings.noise.height,
    );
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => {
            terrain_height.max(settings.sea_level + 1)
        }
        HeightmapKind::OceanFloorWg
        | HeightmapKind::OceanFloor
        | HeightmapKind::MotionBlocking
        | HeightmapKind::MotionBlockingNoLeaves => terrain_height,
    }
}

pub fn noise_preview_base_column(
    x: i32,
    z: i32,
    settings: &NoiseGeneratorSettings,
) -> FlatNoiseColumn {
    let min_y = settings.noise.min_y;
    let height = settings.noise.height;
    let terrain_height =
        noise_preview_terrain_height(x, z, settings).clamp(min_y + 1, min_y + height);
    let states = (0..height.max(0))
        .map(|offset| {
            noise_preview_block_at(min_y + offset, min_y, terrain_height, settings.sea_level)
        })
        .collect();
    FlatNoiseColumn { min_y, states }
}

fn noise_preview_biome(biome_source_model: &BiomeSourceModel, pos: ChunkPos) -> &'static str {
    let quart_x = pos.x * 4 + 2;
    let quart_z = pos.z * 4 + 2;
    select_biome_from_source(
        biome_source_model,
        quart_x,
        16,
        quart_z,
        climate_target(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        0.0,
    )
    .unwrap_or("minecraft:plains")
}

fn flat_section_block_states(
    layers: &[Option<&'static str>],
    section_index: usize,
) -> PalettedContainer {
    let section_start = section_index * 16;
    let mut palette: Vec<&'static str> = Vec::new();
    let mut indices = vec![0_u64; SECTION_VOLUME];

    for local_y in 0..16 {
        let block = layers
            .get(section_start + local_y)
            .and_then(|state| *state)
            .unwrap_or("minecraft:air");
        let palette_index = match palette.iter().position(|entry| *entry == block) {
            Some(index) => index as u64,
            None => {
                palette.push(block);
                (palette.len() - 1) as u64
            }
        };
        for z in 0..16 {
            for x in 0..16 {
                indices[(local_y << 8) | (z << 4) | x] = palette_index;
            }
        }
    }

    if palette.len() == 1 {
        return PalettedContainer::single(block_state_tag(palette[0]), SECTION_VOLUME);
    }

    PalettedContainer {
        palette: palette.into_iter().map(block_state_tag).collect(),
        data: Some(pack_palette_indices(
            &indices,
            bits_for_palette(indices.iter().copied().max().unwrap_or(0) + 1),
        )),
        expected_entries: SECTION_VOLUME,
    }
}

fn block_state_tag(block: &'static str) -> Tag {
    BlockStateEntry::new(block).to_nbt()
}

fn pack_palette_indices(indices: &[u64], bits_per_entry: usize) -> Vec<i64> {
    let values_per_long = 64 / bits_per_entry;
    let mut packed = vec![0_u64; indices.len().div_ceil(values_per_long)];
    for (index, value) in indices.iter().copied().enumerate() {
        let word_index = index / values_per_long;
        let bit_index = (index - word_index * values_per_long) * bits_per_entry;
        packed[word_index] |= value << bit_index;
    }
    packed.into_iter().map(|word| word as i64).collect()
}

fn pack_heightmap(values: [i32; 16 * 16]) -> Vec<i64> {
    const BITS_PER_ENTRY: usize = 9;
    let mut packed = vec![0_u64; (values.len() * BITS_PER_ENTRY).div_ceil(64)];
    for (index, value) in values.into_iter().enumerate() {
        let bit_offset = index * BITS_PER_ENTRY;
        let word_index = bit_offset / 64;
        let bit_index = bit_offset % 64;
        let value = value.max(0) as u64 & ((1 << BITS_PER_ENTRY) - 1);
        packed[word_index] |= value << bit_index;
        let spill = bit_index + BITS_PER_ENTRY;
        if spill > 64 {
            packed[word_index + 1] |= value >> (64 - bit_index);
        }
    }
    packed.into_iter().map(|word| word as i64).collect()
}

fn bits_for_palette(palette_len: u64) -> usize {
    let needed = 64 - (palette_len.saturating_sub(1)).leading_zeros() as usize;
    needed.max(4)
}

fn heightmap_opaque(heightmap: HeightmapKind, block: &'static str) -> bool {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => block != "minecraft:air",
        HeightmapKind::OceanFloor
        | HeightmapKind::OceanFloorWg
        | HeightmapKind::MotionBlocking
        | HeightmapKind::MotionBlockingNoLeaves => motion_blocking_block(block),
    }
}

fn motion_blocking_block(block: &'static str) -> bool {
    !matches!(
        block,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:snow"
    )
}

pub const NETHER_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_nether",
    generator: "minecraft:noise",
    biome_source: "minecraft:multi_noise/nether",
    noise_settings: Some("minecraft:nether"),
};

pub const END_LEVEL_STEM: LevelStemPreset = LevelStemPreset {
    dimension: "minecraft:the_end",
    generator: "minecraft:noise",
    biome_source: "minecraft:the_end",
    noise_settings: Some("minecraft:end"),
};

pub const WORLD_PRESETS: &[WorldPresetEntry] = &[
    WorldPresetEntry {
        id: "minecraft:normal",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:flat",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:flat",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:large_biomes",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:large_biomes"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:amplified",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:multi_noise/overworld",
            noise_settings: Some("minecraft:amplified"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:single_biome_surface",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:noise",
            biome_source: "minecraft:fixed/plains",
            noise_settings: Some("minecraft:overworld"),
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
    WorldPresetEntry {
        id: "minecraft:debug_all_block_states",
        overworld: LevelStemPreset {
            dimension: "minecraft:overworld",
            generator: "minecraft:debug",
            biome_source: "minecraft:plains",
            noise_settings: None,
        },
        nether: NETHER_LEVEL_STEM,
        end: END_LEVEL_STEM,
    },
];

pub const OVERWORLD_SPAWN_TARGET: &[ClimateParameterPoint] = &[
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(-1.0, -0.16),
        offset: 0,
    },
    ClimateParameterPoint {
        temperature: span(-1.0, 1.0),
        humidity: span(-1.0, 1.0),
        continentalness: span(-0.11, 1.0),
        erosion: span(-1.0, 1.0),
        depth: span(0.0, 0.0),
        weirdness: span(0.16, 1.0),
        offset: 0,
    },
];

pub const BUILTIN_NOISE_GENERATOR_SETTINGS: &[NoiseGeneratorSettings] = &[
    NoiseGeneratorSettings {
        id: "minecraft:overworld",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:large_biomes",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: true,
            amplified: false,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:amplified",
        noise: OVERWORLD_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Overworld {
            large_biomes: false,
            amplified: true,
        },
        surface_rule: SurfaceRulePreset::Overworld,
        spawn_target: OVERWORLD_SPAWN_TARGET,
        sea_level: 63,
        disable_mob_generation: false,
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
    },
    NoiseGeneratorSettings {
        id: "minecraft:nether",
        noise: NETHER_NOISE_SETTINGS,
        default_block: "minecraft:netherrack",
        default_fluid: "minecraft:lava",
        noise_router: NoiseRouterPreset::Nether,
        surface_rule: SurfaceRulePreset::Nether,
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:end",
        noise: END_NOISE_SETTINGS,
        default_block: "minecraft:end_stone",
        default_fluid: "minecraft:air",
        noise_router: NoiseRouterPreset::End,
        surface_rule: SurfaceRulePreset::End,
        spawn_target: &[],
        sea_level: 0,
        disable_mob_generation: true,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:caves",
        noise: CAVES_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::Caves,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: true,
            surface: true,
        },
        spawn_target: &[],
        sea_level: 32,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
    NoiseGeneratorSettings {
        id: "minecraft:floating_islands",
        noise: FLOATING_ISLANDS_NOISE_SETTINGS,
        default_block: "minecraft:stone",
        default_fluid: "minecraft:water",
        noise_router: NoiseRouterPreset::FloatingIslands,
        surface_rule: SurfaceRulePreset::OverworldLike {
            bedrock_roof: false,
            bedrock_floor: false,
            surface: false,
        },
        spawn_target: &[],
        sea_level: -64,
        disable_mob_generation: false,
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
    },
];

pub const ZERO_DENSITY: DensityFunction = DensityFunction::Constant(0.0);
pub const Y_DENSITY: DensityFunction = DensityFunction::YClampedGradient {
    from_y: -4064,
    to_y: 4062,
    from_value: -4064.0,
    to_value: 4062.0,
};
pub const SHIFT_X_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &SHIFT_X_CACHE_2D_DENSITY,
};
pub const SHIFT_X_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &SHIFT_A_DENSITY,
};
pub const SHIFT_A_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:shift",
    xz_scale: 0.25,
    y_scale: 0.0,
};
pub const SHIFT_Z_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::FlatCache,
    input: &SHIFT_Z_CACHE_2D_DENSITY,
};
pub const SHIFT_Z_CACHE_2D_DENSITY: DensityFunction = DensityFunction::Marker {
    kind: DensityMarker::Cache2D,
    input: &SHIFT_B_DENSITY,
};
pub const SHIFT_B_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:shift",
    xz_scale: 0.25,
    y_scale: 0.0,
};
pub const BASE_3D_NOISE_OVERWORLD_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.125,
    xz_factor: 80.0,
    y_factor: 160.0,
    smear_scale_multiplier: 8.0,
};
pub const BASE_3D_NOISE_NETHER_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.375,
    xz_factor: 80.0,
    y_factor: 60.0,
    smear_scale_multiplier: 8.0,
};
pub const BASE_3D_NOISE_END_DENSITY: DensityFunction = DensityFunction::BlendedNoise {
    xz_scale: 0.25,
    y_scale: 0.25,
    xz_factor: 80.0,
    y_factor: 160.0,
    smear_scale_multiplier: 4.0,
};

pub const DENSITY_FUNCTION_TYPES: &[DensityFunctionType] = &[
    DensityFunctionType { id: "blend_alpha" },
    DensityFunctionType { id: "blend_offset" },
    DensityFunctionType { id: "beardifier" },
    DensityFunctionType {
        id: "old_blended_noise",
    },
    DensityFunctionType { id: "interpolated" },
    DensityFunctionType { id: "flat_cache" },
    DensityFunctionType { id: "cache_2d" },
    DensityFunctionType { id: "cache_once" },
    DensityFunctionType {
        id: "cache_all_in_cell",
    },
    DensityFunctionType { id: "noise" },
    DensityFunctionType { id: "end_islands" },
    DensityFunctionType {
        id: "weird_scaled_sampler",
    },
    DensityFunctionType {
        id: "shifted_noise",
    },
    DensityFunctionType { id: "range_choice" },
    DensityFunctionType { id: "shift_a" },
    DensityFunctionType { id: "shift_b" },
    DensityFunctionType { id: "shift" },
    DensityFunctionType {
        id: "blend_density",
    },
    DensityFunctionType { id: "clamp" },
    DensityFunctionType { id: "abs" },
    DensityFunctionType { id: "square" },
    DensityFunctionType { id: "cube" },
    DensityFunctionType {
        id: "half_negative",
    },
    DensityFunctionType {
        id: "quarter_negative",
    },
    DensityFunctionType { id: "invert" },
    DensityFunctionType { id: "squeeze" },
    DensityFunctionType { id: "add" },
    DensityFunctionType { id: "mul" },
    DensityFunctionType { id: "min" },
    DensityFunctionType { id: "max" },
    DensityFunctionType { id: "spline" },
    DensityFunctionType { id: "constant" },
    DensityFunctionType {
        id: "y_clamped_gradient",
    },
    DensityFunctionType {
        id: "find_top_surface",
    },
];

pub const BUILTIN_DENSITY_FUNCTIONS: &[DensityFunctionEntry] = &[
    DensityFunctionEntry {
        id: "minecraft:zero",
        function: ZERO_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:y",
        function: Y_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:shift_x",
        function: SHIFT_X_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:shift_z",
        function: SHIFT_Z_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/base_3d_noise",
        function: BASE_3D_NOISE_OVERWORLD_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:nether/base_3d_noise",
        function: BASE_3D_NOISE_NETHER_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:end/base_3d_noise",
        function: BASE_3D_NOISE_END_DENSITY,
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/continents",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:continentalness",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/erosion",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:erosion",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/ridges",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:ridge",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/ridges_folded",
        function: DensityFunction::Mapped {
            kind: MappedDensityFunction::Abs,
            input: &RIDGE_FOLD_SOURCE_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/continents",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:continentalness_large",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld_large_biomes/erosion",
        function: DensityFunction::ShiftedNoise {
            shift_x: &SHIFT_X_DENSITY,
            shift_y: &ZERO_DENSITY,
            shift_z: &SHIFT_Z_DENSITY,
            xz_scale: 0.25,
            y_scale: 0.0,
            noise: "minecraft:erosion_large",
        },
    },
    DensityFunctionEntry {
        id: "minecraft:end/sloped_cheese",
        function: DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &END_ISLANDS_DENSITY,
            argument2: &BASE_3D_NOISE_END_DENSITY,
        },
    },
    DensityFunctionEntry {
        id: "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
        function: DensityFunction::Marker {
            kind: DensityMarker::CacheOnce,
            input: &SPAGHETTI_2D_THICKNESS_MODULATOR_DENSITY,
        },
    },
];

pub const RIDGE_FOLD_SOURCE_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &RIDGE_SCALE_DENSITY,
    argument2: &RIDGE_OFFSET_DENSITY,
};
pub const RIDGE_SCALE_DENSITY: DensityFunction = DensityFunction::Constant(-3.0);
pub const RIDGE_OFFSET_DENSITY: DensityFunction = DensityFunction::Constant(2.0);
pub const END_ISLANDS_DENSITY: DensityFunction = DensityFunction::EndIslands { seed: 0 };
pub const SPAGHETTI_2D_THICKNESS_MODULATOR_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Add,
    argument1: &SPAGHETTI_2D_THICKNESS_MID_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_SCALED_DENSITY,
};
pub const SPAGHETTI_2D_THICKNESS_MID_DENSITY: DensityFunction = DensityFunction::Constant(-0.95);
pub const SPAGHETTI_2D_THICKNESS_SCALED_DENSITY: DensityFunction = DensityFunction::Binary {
    kind: BinaryDensityFunction::Mul,
    argument1: &SPAGHETTI_2D_THICKNESS_FACTOR_DENSITY,
    argument2: &SPAGHETTI_2D_THICKNESS_NOISE_DENSITY,
};
pub const SPAGHETTI_2D_THICKNESS_FACTOR_DENSITY: DensityFunction = DensityFunction::Constant(-0.35);
pub const SPAGHETTI_2D_THICKNESS_NOISE_DENSITY: DensityFunction = DensityFunction::Noise {
    noise: "minecraft:spaghetti_2d_thickness",
    xz_scale: 2.0,
    y_scale: 1.0,
};
pub const TEST_NEGATIVE_DENSITY: DensityFunction = DensityFunction::Constant(-2.0);
pub const TEST_POSITIVE_DENSITY: DensityFunction = DensityFunction::Constant(3.0);

pub const OVERWORLD_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    barrier: DensityFunction::Noise {
        noise: "minecraft:aquifer_barrier",
        xz_scale: 1.0,
        y_scale: 0.5,
    },
    fluid_level_floodedness: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_floodedness",
        xz_scale: 1.0,
        y_scale: 0.67,
    },
    fluid_level_spread: DensityFunction::Noise {
        noise: "minecraft:aquifer_fluid_level_spread",
        xz_scale: 1.0,
        y_scale: 0.7142857142857143,
    },
    lava: DensityFunction::Noise {
        noise: "minecraft:aquifer_lava",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation",
    },
    continents: DensityFunction::Reference("minecraft:overworld/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld/depth"),
    ridges: DensityFunction::Reference("minecraft:overworld/ridges"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld/final_density"),
    vein_toggle: DensityFunction::Noise {
        noise: "minecraft:ore_veininess",
        xz_scale: 1.5,
        y_scale: 1.5,
    },
    vein_ridged: DensityFunction::Reference("minecraft:overworld/vein_ridged"),
    vein_gap: DensityFunction::Noise {
        noise: "minecraft:ore_gap",
        xz_scale: 1.0,
        y_scale: 1.0,
    },
};

pub const LARGE_BIOMES_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature_large",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &SHIFT_X_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &SHIFT_Z_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation_large",
    },
    continents: DensityFunction::Reference("minecraft:overworld_large_biomes/continents"),
    erosion: DensityFunction::Reference("minecraft:overworld_large_biomes/erosion"),
    depth: DensityFunction::Reference("minecraft:overworld_large_biomes/depth"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld_large_biomes/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld_large_biomes/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const AMPLIFIED_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    depth: DensityFunction::Reference("minecraft:overworld_amplified/depth"),
    preliminary_surface_level: DensityFunction::Reference(
        "minecraft:overworld_amplified/preliminary_surface_level",
    ),
    final_density: DensityFunction::Reference("minecraft:overworld_amplified/final_density"),
    ..OVERWORLD_NOISE_ROUTER
};

pub const NETHER_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    temperature: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:temperature_nether",
    },
    vegetation: DensityFunction::ShiftedNoise {
        shift_x: &ZERO_DENSITY,
        shift_y: &ZERO_DENSITY,
        shift_z: &ZERO_DENSITY,
        xz_scale: 0.25,
        y_scale: 0.0,
        noise: "minecraft:vegetation_nether",
    },
    final_density: DensityFunction::Reference("minecraft:nether/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};

pub const CAVES_NOISE_ROUTER: NoiseRouter =
    NoiseRouter::simple(DensityFunction::Reference("minecraft:caves/final_density"));
pub const FLOATING_ISLANDS_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(
    DensityFunction::Reference("minecraft:floating_islands/final_density"),
);
pub const END_NOISE_ROUTER: NoiseRouter = NoiseRouter {
    erosion: DensityFunction::Marker {
        kind: DensityMarker::Cache2D,
        input: &END_ISLANDS_DENSITY,
    },
    final_density: DensityFunction::Reference("minecraft:end/final_density"),
    ..NoiseRouter::simple(ZERO_DENSITY)
};
pub const NONE_NOISE_ROUTER: NoiseRouter = NoiseRouter::simple(ZERO_DENSITY);

pub const BUILTIN_NOISE_ROUTERS: &[NoiseRouterEntry] = &[
    NoiseRouterEntry {
        id: "minecraft:overworld",
        router: OVERWORLD_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:large_biomes",
        router: LARGE_BIOMES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:amplified",
        router: AMPLIFIED_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:nether",
        router: NETHER_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:end",
        router: END_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:caves",
        router: CAVES_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:floating_islands",
        router: FLOATING_ISLANDS_NOISE_ROUTER,
    },
    NoiseRouterEntry {
        id: "minecraft:none",
        router: NONE_NOISE_ROUTER,
    },
];

pub const SURFACE_RULE_TYPES: &[SurfaceRuleType] = &[
    SurfaceRuleType { id: "bandlands" },
    SurfaceRuleType { id: "block" },
    SurfaceRuleType { id: "sequence" },
    SurfaceRuleType { id: "condition" },
];

pub const SURFACE_CONDITION_TYPES: &[SurfaceConditionType] = &[
    SurfaceConditionType { id: "biome" },
    SurfaceConditionType {
        id: "noise_threshold",
    },
    SurfaceConditionType {
        id: "vertical_gradient",
    },
    SurfaceConditionType { id: "y_above" },
    SurfaceConditionType { id: "water" },
    SurfaceConditionType { id: "stone_depth" },
    SurfaceConditionType { id: "not" },
    SurfaceConditionType { id: "steep" },
    SurfaceConditionType { id: "hole" },
    SurfaceConditionType {
        id: "above_preliminary_surface",
    },
    SurfaceConditionType { id: "temperature" },
];

pub const OVERWORLD_SURFACE_BLOCKS: &[&str] = &[
    "minecraft:air",
    "minecraft:bedrock",
    "minecraft:white_terracotta",
    "minecraft:orange_terracotta",
    "minecraft:terracotta",
    "minecraft:red_sand",
    "minecraft:red_sandstone",
    "minecraft:stone",
    "minecraft:deepslate",
    "minecraft:dirt",
    "minecraft:podzol",
    "minecraft:coarse_dirt",
    "minecraft:mycelium",
    "minecraft:grass_block",
    "minecraft:calcite",
    "minecraft:gravel",
    "minecraft:sand",
    "minecraft:sandstone",
    "minecraft:packed_ice",
    "minecraft:snow_block",
    "minecraft:mud",
    "minecraft:powder_snow",
    "minecraft:ice",
    "minecraft:water",
];

pub const NETHER_SURFACE_BLOCKS: &[&str] = &[
    "minecraft:bedrock",
    "minecraft:gravel",
    "minecraft:lava",
    "minecraft:netherrack",
    "minecraft:soul_sand",
    "minecraft:soul_soil",
    "minecraft:basalt",
    "minecraft:blackstone",
    "minecraft:warped_wart_block",
    "minecraft:warped_nylium",
    "minecraft:nether_wart_block",
    "minecraft:crimson_nylium",
];

pub const OVERWORLD_SURFACE_CONDITIONS: &[&str] = &[
    "above_preliminary_surface",
    "bedrock_floor_vertical_gradient",
    "deepslate_vertical_gradient",
    "water",
    "stone_depth",
    "biome",
    "noise_threshold",
    "hole",
    "steep",
    "temperature",
];

pub const NETHER_SURFACE_CONDITIONS: &[&str] = &[
    "bedrock_floor_vertical_gradient",
    "bedrock_roof_vertical_gradient",
    "y_above",
    "hole",
    "noise_threshold",
    "biome",
    "stone_depth",
];

pub const BUILTIN_SURFACE_RULE_PRESETS: &[SurfaceRulePresetData] = &[
    SurfaceRulePresetData {
        id: "minecraft:overworld",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: true,
            bedrock_roof: false,
            bedrock_floor: true,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:caves",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: true,
            bedrock_floor: true,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:floating_islands",
        rule: SurfaceRuleKind::OverworldLike {
            preliminary_surface_check: false,
            bedrock_roof: false,
            bedrock_floor: false,
            deepslate: true,
        },
        blocks: OVERWORLD_SURFACE_BLOCKS,
        conditions: OVERWORLD_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:nether",
        rule: SurfaceRuleKind::Nether,
        blocks: NETHER_SURFACE_BLOCKS,
        conditions: NETHER_SURFACE_CONDITIONS,
    },
    SurfaceRulePresetData {
        id: "minecraft:end",
        rule: SurfaceRuleKind::State("minecraft:end_stone"),
        blocks: &["minecraft:end_stone"],
        conditions: &[],
    },
    SurfaceRulePresetData {
        id: "minecraft:air",
        rule: SurfaceRuleKind::State("minecraft:air"),
        blocks: &["minecraft:air"],
        conditions: &[],
    },
];

pub const AQUIFER_NOISE_SETTINGS: AquiferNoiseSettings = AquiferNoiseSettings {
    x_range: 10,
    y_range: 9,
    z_range: 10,
    x_separation: 6,
    y_separation: 3,
    z_separation: 6,
    x_spacing: 16,
    y_spacing: 12,
    z_spacing: 16,
    max_reasonable_distance_to_center: 11,
    sample_offset_x: -5,
    sample_offset_y: 1,
    sample_offset_z: -5,
};

pub const AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS: &[(i32, i32)] = &[
    (0, 0),
    (-2, -1),
    (-1, -1),
    (0, -1),
    (1, -1),
    (-3, 0),
    (-2, 0),
    (-1, 0),
    (1, 0),
    (-2, 1),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub const CAVE_GENERATION_FAMILIES: &[CaveGenerationFamily] = &[
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/spaghetti_roughness_function",
        noises: &[
            "minecraft:spaghetti_roughness",
            "minecraft:spaghetti_roughness_modulator",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/entrances",
        noises: &[
            "minecraft:spaghetti_3d_rarity",
            "minecraft:spaghetti_3d_thickness",
            "minecraft:spaghetti_3d_1",
            "minecraft:spaghetti_3d_2",
            "minecraft:cave_entrance",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/noodle",
        noises: &[
            "minecraft:noodle",
            "minecraft:noodle_thickness",
            "minecraft:noodle_ridge_a",
            "minecraft:noodle_ridge_b",
        ],
        output: CaveDensityOutput::RangeChoice,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/pillars",
        noises: &[
            "minecraft:pillar",
            "minecraft:pillar_rareness",
            "minecraft:pillar_thickness",
        ],
        output: CaveDensityOutput::CacheOnce,
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/spaghetti_2d",
        noises: &[
            "minecraft:spaghetti_2d_modulator",
            "minecraft:spaghetti_2d",
            "minecraft:spaghetti_2d_elevation",
            "minecraft:spaghetti_2d_thickness",
        ],
        output: CaveDensityOutput::Clamp { min: -1, max: 1 },
    },
    CaveGenerationFamily {
        id: "minecraft:overworld/caves/underground",
        noises: &["minecraft:cave_layer", "minecraft:cave_cheese"],
        output: CaveDensityOutput::Max,
    },
];

pub const ORE_VEINIFIER_CONSTANTS: OreVeinifierConstants = OreVeinifierConstants {
    veininess_threshold: 0.4,
    edge_roundoff_begin: 20,
    max_edge_roundoff: 0.2,
    vein_solidness: 0.7,
    min_richness: 0.1,
    max_richness: 0.3,
    max_richness_threshold: 0.6,
    chance_of_raw_ore_block: 0.02,
    skip_ore_if_gap_noise_is_below: -0.3,
};

pub const ORE_VEIN_TYPES: &[OreVeinType] = &[
    OreVeinType {
        id: "copper",
        ore: "minecraft:copper_ore",
        raw_ore_block: "minecraft:raw_copper_block",
        filler: "minecraft:granite",
        min_y: 0,
        max_y: 50,
    },
    OreVeinType {
        id: "iron",
        ore: "minecraft:deepslate_iron_ore",
        raw_ore_block: "minecraft:raw_iron_block",
        filler: "minecraft:tuff",
        min_y: -60,
        max_y: -8,
    },
];

pub const WORLD_CARVER_TYPES: &[WorldCarverType] = &[
    WorldCarverType::Cave,
    WorldCarverType::NetherCave,
    WorldCarverType::Canyon,
];

pub const CONFIGURED_CARVERS: &[ConfiguredCarver] = &[
    ConfiguredCarver {
        id: "minecraft:cave",
        carver_type: WorldCarverType::Cave,
        probability: 0.15,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(180),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:crimson_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:cave_extra_underground",
        carver_type: WorldCarverType::Cave,
        probability: 0.07,
        y: HeightRange {
            min: VerticalAnchor::AboveBottom(8),
            max: VerticalAnchor::Absolute(47),
        },
        y_scale: FloatProvider::Uniform { min: 0.1, max: 0.9 },
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:oak_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Uniform { min: 0.7, max: 1.4 },
            vertical_radius_multiplier: FloatProvider::Uniform { min: 0.8, max: 1.3 },
            floor_level: FloatProvider::Uniform {
                min: -1.0,
                max: -0.4,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:canyon",
        carver_type: WorldCarverType::Canyon,
        probability: 0.01,
        y: HeightRange {
            min: VerticalAnchor::Absolute(10),
            max: VerticalAnchor::Absolute(67),
        },
        y_scale: FloatProvider::Constant(3.0),
        lava_level: VerticalAnchor::AboveBottom(8),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:warped_button",
        },
        replaceable_tag: "#minecraft:overworld_carver_replaceables",
        shape: CarverShape::Canyon {
            vertical_rotation: FloatProvider::Uniform {
                min: -0.125,
                max: 0.125,
            },
            shape: CanyonShapeConfiguration {
                distance_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                thickness: FloatProvider::Trapezoid {
                    min: 0.0,
                    max: 6.0,
                    plateau: 2.0,
                },
                width_smoothness: 3,
                horizontal_radius_factor: FloatProvider::Uniform {
                    min: 0.75,
                    max: 1.0,
                },
                vertical_radius_default_factor: 1.0,
                vertical_radius_center_factor: 0.0,
            },
        },
    },
    ConfiguredCarver {
        id: "minecraft:nether_cave",
        carver_type: WorldCarverType::NetherCave,
        probability: 0.2,
        y: HeightRange {
            min: VerticalAnchor::Absolute(0),
            max: VerticalAnchor::BelowTop(1),
        },
        y_scale: FloatProvider::Constant(0.5),
        lava_level: VerticalAnchor::AboveBottom(10),
        debug: CarverDebugSettings {
            enabled: false,
            barrier_state: "minecraft:air",
        },
        replaceable_tag: "#minecraft:nether_carver_replaceables",
        shape: CarverShape::Cave {
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-0.7),
        },
    },
];

pub const FEATURE_TYPES: &[FeatureType] = &[
    feature_type(
        "minecraft:no_op",
        FeatureConfigurationKind::None,
        FeatureFamily::Selector,
    ),
    feature_type(
        "minecraft:tree",
        FeatureConfigurationKind::Tree,
        FeatureFamily::Tree,
    ),
    feature_type(
        "minecraft:fallen_tree",
        FeatureConfigurationKind::FallenTree,
        FeatureFamily::Tree,
    ),
    feature_type(
        "minecraft:block_pile",
        FeatureConfigurationKind::BlockPile,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:spring_feature",
        FeatureConfigurationKind::Spring,
        FeatureFamily::Cave,
    ),
    feature_type(
        "minecraft:chorus_plant",
        FeatureConfigurationKind::None,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:replace_single_block",
        FeatureConfigurationKind::ReplaceBlock,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:void_start_platform",
        FeatureConfigurationKind::None,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:desert_well",
        FeatureConfigurationKind::None,
        FeatureFamily::StructureLike,
    ),
    feature_type(
        "minecraft:fossil",
        FeatureConfigurationKind::Fossil,
        FeatureFamily::StructureLike,
    ),
    feature_type(
        "minecraft:huge_red_mushroom",
        FeatureConfigurationKind::HugeMushroom,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:huge_brown_mushroom",
        FeatureConfigurationKind::HugeMushroom,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:spike",
        FeatureConfigurationKind::Spike,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:glowstone_blob",
        FeatureConfigurationKind::None,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:freeze_top_layer",
        FeatureConfigurationKind::None,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:vines",
        FeatureConfigurationKind::None,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:block_column",
        FeatureConfigurationKind::BlockColumn,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:vegetation_patch",
        FeatureConfigurationKind::VegetationPatch,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:waterlogged_vegetation_patch",
        FeatureConfigurationKind::VegetationPatch,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:root_system",
        FeatureConfigurationKind::RootSystem,
        FeatureFamily::Tree,
    ),
    feature_type(
        "minecraft:multiface_growth",
        FeatureConfigurationKind::MultifaceGrowth,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:underwater_magma",
        FeatureConfigurationKind::UnderwaterMagma,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:monster_room",
        FeatureConfigurationKind::None,
        FeatureFamily::StructureLike,
    ),
    feature_type(
        "minecraft:blue_ice",
        FeatureConfigurationKind::None,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:iceberg",
        FeatureConfigurationKind::BlockState,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:block_blob",
        FeatureConfigurationKind::BlockBlob,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:disk",
        FeatureConfigurationKind::Disk,
        FeatureFamily::Disk,
    ),
    feature_type(
        "minecraft:lake",
        FeatureConfigurationKind::Lake,
        FeatureFamily::Lake,
    ),
    feature_type(
        "minecraft:ore",
        FeatureConfigurationKind::Ore,
        FeatureFamily::Ore,
    ),
    feature_type(
        "minecraft:end_platform",
        FeatureConfigurationKind::None,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:end_spike",
        FeatureConfigurationKind::EndSpike,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:end_island",
        FeatureConfigurationKind::None,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:end_gateway",
        FeatureConfigurationKind::EndGateway,
        FeatureFamily::End,
    ),
    feature_type(
        "minecraft:seagrass",
        FeatureConfigurationKind::Probability,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:kelp",
        FeatureConfigurationKind::None,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:coral_tree",
        FeatureConfigurationKind::None,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:coral_mushroom",
        FeatureConfigurationKind::None,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:coral_claw",
        FeatureConfigurationKind::None,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:sea_pickle",
        FeatureConfigurationKind::Count,
        FeatureFamily::Aquatic,
    ),
    feature_type(
        "minecraft:simple_block",
        FeatureConfigurationKind::SimpleBlock,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:bamboo",
        FeatureConfigurationKind::Probability,
        FeatureFamily::Vegetation,
    ),
    feature_type(
        "minecraft:huge_fungus",
        FeatureConfigurationKind::HugeFungus,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:nether_forest_vegetation",
        FeatureConfigurationKind::NetherForestVegetation,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:weeping_vines",
        FeatureConfigurationKind::None,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:twisting_vines",
        FeatureConfigurationKind::TwistingVines,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:basalt_columns",
        FeatureConfigurationKind::Column,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:delta_feature",
        FeatureConfigurationKind::Delta,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:netherrack_replace_blobs",
        FeatureConfigurationKind::ReplaceSphere,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:fill_layer",
        FeatureConfigurationKind::Layer,
        FeatureFamily::BlockPlacement,
    ),
    feature_type(
        "minecraft:bonus_chest",
        FeatureConfigurationKind::None,
        FeatureFamily::StructureLike,
    ),
    feature_type(
        "minecraft:basalt_pillar",
        FeatureConfigurationKind::None,
        FeatureFamily::Nether,
    ),
    feature_type(
        "minecraft:scattered_ore",
        FeatureConfigurationKind::Ore,
        FeatureFamily::Ore,
    ),
    feature_type(
        "minecraft:random_selector",
        FeatureConfigurationKind::RandomFeature,
        FeatureFamily::Selector,
    ),
    feature_type(
        "minecraft:simple_random_selector",
        FeatureConfigurationKind::SimpleRandomFeature,
        FeatureFamily::Selector,
    ),
    feature_type(
        "minecraft:random_boolean_selector",
        FeatureConfigurationKind::RandomBooleanFeature,
        FeatureFamily::Selector,
    ),
    feature_type(
        "minecraft:geode",
        FeatureConfigurationKind::Geode,
        FeatureFamily::Cave,
    ),
    feature_type(
        "minecraft:dripstone_cluster",
        FeatureConfigurationKind::DripstoneCluster,
        FeatureFamily::Cave,
    ),
    feature_type(
        "minecraft:large_dripstone",
        FeatureConfigurationKind::LargeDripstone,
        FeatureFamily::Cave,
    ),
    feature_type(
        "minecraft:pointed_dripstone",
        FeatureConfigurationKind::PointedDripstone,
        FeatureFamily::Cave,
    ),
    feature_type(
        "minecraft:sculk_patch",
        FeatureConfigurationKind::SculkPatch,
        FeatureFamily::Cave,
    ),
];

pub const CONFIGURED_FEATURES: &[ConfiguredFeatureEntry] = &[
    cf("minecraft:seagrass_short", ConfiguredFeatureSource::Aquatic),
    cf(
        "minecraft:seagrass_slightly_less_short",
        ConfiguredFeatureSource::Aquatic,
    ),
    cf("minecraft:seagrass_mid", ConfiguredFeatureSource::Aquatic),
    cf("minecraft:seagrass_tall", ConfiguredFeatureSource::Aquatic),
    cf("minecraft:sea_pickle", ConfiguredFeatureSource::Aquatic),
    cf("minecraft:kelp", ConfiguredFeatureSource::Aquatic),
    cf(
        "minecraft:warm_ocean_vegetation",
        ConfiguredFeatureSource::Aquatic,
    ),
    cf("minecraft:monster_room", ConfiguredFeatureSource::Cave),
    cf("minecraft:fossil_coal", ConfiguredFeatureSource::Cave),
    cf("minecraft:fossil_diamonds", ConfiguredFeatureSource::Cave),
    cf("minecraft:dripstone_cluster", ConfiguredFeatureSource::Cave),
    cf("minecraft:large_dripstone", ConfiguredFeatureSource::Cave),
    cf("minecraft:pointed_dripstone", ConfiguredFeatureSource::Cave),
    cf("minecraft:underwater_magma", ConfiguredFeatureSource::Cave),
    cf("minecraft:glow_lichen", ConfiguredFeatureSource::Cave),
    cf(
        "minecraft:rooted_azalea_tree",
        ConfiguredFeatureSource::Cave,
    ),
    cf("minecraft:cave_vine", ConfiguredFeatureSource::Cave),
    cf("minecraft:cave_vine_in_moss", ConfiguredFeatureSource::Cave),
    cf("minecraft:moss_vegetation", ConfiguredFeatureSource::Cave),
    cf("minecraft:moss_patch", ConfiguredFeatureSource::Cave),
    cf(
        "minecraft:moss_patch_bonemeal",
        ConfiguredFeatureSource::Cave,
    ),
    cf("minecraft:dripleaf", ConfiguredFeatureSource::Cave),
    cf(
        "minecraft:clay_with_dripleaves",
        ConfiguredFeatureSource::Cave,
    ),
    cf(
        "minecraft:clay_pool_with_dripleaves",
        ConfiguredFeatureSource::Cave,
    ),
    cf("minecraft:lush_caves_clay", ConfiguredFeatureSource::Cave),
    cf(
        "minecraft:moss_patch_ceiling",
        ConfiguredFeatureSource::Cave,
    ),
    cf("minecraft:spore_blossom", ConfiguredFeatureSource::Cave),
    cf("minecraft:amethyst_geode", ConfiguredFeatureSource::Cave),
    cf(
        "minecraft:sculk_patch_deep_dark",
        ConfiguredFeatureSource::Cave,
    ),
    cf(
        "minecraft:sculk_patch_ancient_city",
        ConfiguredFeatureSource::Cave,
    ),
    cf("minecraft:sculk_vein", ConfiguredFeatureSource::Cave),
    cf("minecraft:end_platform", ConfiguredFeatureSource::End),
    cf("minecraft:end_spike", ConfiguredFeatureSource::End),
    cf("minecraft:end_gateway_return", ConfiguredFeatureSource::End),
    cf(
        "minecraft:end_gateway_delayed",
        ConfiguredFeatureSource::End,
    ),
    cf("minecraft:chorus_plant", ConfiguredFeatureSource::End),
    cf("minecraft:end_island", ConfiguredFeatureSource::End),
    cf(
        "minecraft:ice_spike",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:ice_patch",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:forest_rock",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:iceberg_packed",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:iceberg_blue",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf("minecraft:blue_ice", ConfiguredFeatureSource::MiscOverworld),
    cf(
        "minecraft:lake_lava",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:disk_clay",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:disk_gravel",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:disk_sand",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:freeze_top_layer",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:disk_grass",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:bonus_chest",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:void_start_platform",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:desert_well",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:spring_lava_overworld",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:spring_lava_frozen",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf(
        "minecraft:spring_water",
        ConfiguredFeatureSource::MiscOverworld,
    ),
    cf("minecraft:delta", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:small_basalt_columns",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:large_basalt_columns",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:basalt_blobs", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:blackstone_blobs",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:glowstone_extra", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:crimson_forest_vegetation",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:crimson_forest_vegetation_bonemeal",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:warped_forest_vegetation",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:warped_forest_vegetation_bonemeal",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:nether_sprouts", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:nether_sprouts_bonemeal",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:twisting_vines", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:twisting_vines_bonemeal",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:weeping_vines", ConfiguredFeatureSource::Nether),
    cf("minecraft:crimson_roots", ConfiguredFeatureSource::Nether),
    cf("minecraft:basalt_pillar", ConfiguredFeatureSource::Nether),
    cf(
        "minecraft:spring_lava_nether",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:spring_nether_closed",
        ConfiguredFeatureSource::Nether,
    ),
    cf(
        "minecraft:spring_nether_open",
        ConfiguredFeatureSource::Nether,
    ),
    cf("minecraft:patch_fire", ConfiguredFeatureSource::Nether),
    cf("minecraft:patch_soul_fire", ConfiguredFeatureSource::Nether),
    cf("minecraft:ore_magma", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_soul_sand", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_nether_gold", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_quartz", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_gravel_nether", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_blackstone", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_dirt", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_gravel", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_granite", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_diorite", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_andesite", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_tuff", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_coal", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_coal_buried", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_iron", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_iron_small", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_gold", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_gold_buried", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_redstone", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_diamond_small", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_diamond_medium", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_diamond_large", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_diamond_buried", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_lapis", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_lapis_buried", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_infested", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_emerald", ConfiguredFeatureSource::Ore),
    cf(
        "minecraft:ore_ancient_debris_large",
        ConfiguredFeatureSource::Ore,
    ),
    cf(
        "minecraft:ore_ancient_debris_small",
        ConfiguredFeatureSource::Ore,
    ),
    cf("minecraft:ore_copper_small", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_copper_large", ConfiguredFeatureSource::Ore),
    cf("minecraft:ore_clay", ConfiguredFeatureSource::Ore),
    cf("minecraft:pile_hay", ConfiguredFeatureSource::Pile),
    cf("minecraft:pile_melon", ConfiguredFeatureSource::Pile),
    cf("minecraft:pile_snow", ConfiguredFeatureSource::Pile),
    cf("minecraft:pile_ice", ConfiguredFeatureSource::Pile),
    cf("minecraft:pile_pumpkin", ConfiguredFeatureSource::Pile),
    cf("minecraft:crimson_fungus", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:crimson_fungus_planted",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:warped_fungus", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:warped_fungus_planted",
        ConfiguredFeatureSource::Tree,
    ),
    cf(
        "minecraft:huge_brown_mushroom",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:huge_red_mushroom", ConfiguredFeatureSource::Tree),
    cf("minecraft:oak", ConfiguredFeatureSource::Tree),
    cf("minecraft:dark_oak", ConfiguredFeatureSource::Tree),
    cf("minecraft:pale_oak", ConfiguredFeatureSource::Tree),
    cf("minecraft:pale_oak_bonemeal", ConfiguredFeatureSource::Tree),
    cf("minecraft:pale_oak_creaking", ConfiguredFeatureSource::Tree),
    cf("minecraft:birch", ConfiguredFeatureSource::Tree),
    cf("minecraft:acacia", ConfiguredFeatureSource::Tree),
    cf("minecraft:spruce", ConfiguredFeatureSource::Tree),
    cf("minecraft:pine", ConfiguredFeatureSource::Tree),
    cf("minecraft:jungle_tree", ConfiguredFeatureSource::Tree),
    cf("minecraft:fancy_oak", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:jungle_tree_no_vine",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:mega_jungle_tree", ConfiguredFeatureSource::Tree),
    cf("minecraft:mega_spruce", ConfiguredFeatureSource::Tree),
    cf("minecraft:mega_pine", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:super_birch_bees_0002",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:super_birch_bees", ConfiguredFeatureSource::Tree),
    cf("minecraft:swamp_oak", ConfiguredFeatureSource::Tree),
    cf("minecraft:jungle_bush", ConfiguredFeatureSource::Tree),
    cf("minecraft:azalea_tree", ConfiguredFeatureSource::Tree),
    cf("minecraft:mangrove", ConfiguredFeatureSource::Tree),
    cf("minecraft:tall_mangrove", ConfiguredFeatureSource::Tree),
    cf("minecraft:cherry", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:oak_bees_0002_leaf_litter",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:oak_bees_002", ConfiguredFeatureSource::Tree),
    cf("minecraft:oak_bees_005", ConfiguredFeatureSource::Tree),
    cf("minecraft:birch_bees_0002", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:birch_bees_0002_leaf_litter",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:birch_bees_002", ConfiguredFeatureSource::Tree),
    cf("minecraft:birch_bees_005", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:fancy_oak_bees_0002_leaf_litter",
        ConfiguredFeatureSource::Tree,
    ),
    cf(
        "minecraft:fancy_oak_bees_002",
        ConfiguredFeatureSource::Tree,
    ),
    cf(
        "minecraft:fancy_oak_bees_005",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:fancy_oak_bees", ConfiguredFeatureSource::Tree),
    cf("minecraft:cherry_bees_005", ConfiguredFeatureSource::Tree),
    cf("minecraft:oak_leaf_litter", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:dark_oak_leaf_litter",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:birch_leaf_litter", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:fancy_oak_leaf_litter",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:fallen_oak_tree", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:fallen_jungle_tree",
        ConfiguredFeatureSource::Tree,
    ),
    cf(
        "minecraft:fallen_spruce_tree",
        ConfiguredFeatureSource::Tree,
    ),
    cf("minecraft:fallen_birch_tree", ConfiguredFeatureSource::Tree),
    cf(
        "minecraft:fallen_super_birch_tree",
        ConfiguredFeatureSource::Tree,
    ),
    cf(
        "minecraft:bamboo_no_podzol",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:bamboo_some_podzol",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:vines", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:brown_mushroom",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:red_mushroom",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:sunflower", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:pumpkin", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:berry_bush", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:taiga_grass", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:grass", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:grass_jungle",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:dead_bush", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:dry_grass", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:melon", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:waterlily", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:tall_grass", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:large_fern", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:bush", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:leaf_litter", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:firefly_bush",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:cactus", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:sugar_cane", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:flower_default",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_flower_forest",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_swamp",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_plain",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_meadow",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_cherry",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:flower_pale_garden",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:wildflower", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:forest_flowers",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:pale_forest_flower",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:dark_forest_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:pale_garden_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:pale_moss_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:pale_moss_patch",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:pale_moss_patch_bonemeal",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_flower_forest",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:meadow_trees",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:trees_taiga", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:trees_badlands",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:trees_grove", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:trees_savanna",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:trees_snowy", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:trees_birch", ConfiguredFeatureSource::Vegetation),
    cf("minecraft:birch_tall", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:trees_windswept_hills",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf("minecraft:trees_water", ConfiguredFeatureSource::Vegetation),
    cf(
        "minecraft:trees_birch_and_oak_leaf_litter",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_plains",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_sparse_jungle",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_old_growth_spruce_taiga",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_old_growth_pine_taiga",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:trees_jungle",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:bamboo_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:mushroom_island_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
    cf(
        "minecraft:mangrove_vegetation",
        ConfiguredFeatureSource::Vegetation,
    ),
];

pub const PLACED_FEATURE_BOOTSTRAP_SOURCES: &[PlacedFeatureSourceEntry] = &[
    placed_source(
        PlacedFeatureSource::Aquatic,
        &[
            "minecraft:seagrass_warm",
            "minecraft:seagrass_normal",
            "minecraft:seagrass_cold",
            "minecraft:seagrass_river",
            "minecraft:seagrass_swamp",
            "minecraft:seagrass_deep_warm",
            "minecraft:seagrass_deep",
            "minecraft:seagrass_deep_cold",
            "minecraft:sea_pickle",
            "minecraft:kelp_cold",
            "minecraft:kelp_warm",
            "minecraft:warm_ocean_vegetation",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Cave,
        &[
            "minecraft:monster_room",
            "minecraft:monster_room_deep",
            "minecraft:fossil_upper",
            "minecraft:fossil_lower",
            "minecraft:dripstone_cluster",
            "minecraft:large_dripstone",
            "minecraft:pointed_dripstone",
            "minecraft:underwater_magma",
            "minecraft:glow_lichen",
            "minecraft:rooted_azalea_tree",
            "minecraft:cave_vines",
            "minecraft:lush_caves_vegetation",
            "minecraft:lush_caves_clay",
            "minecraft:lush_caves_ceiling_vegetation",
            "minecraft:spore_blossom",
            "minecraft:classic_vines_cave_feature",
            "minecraft:amethyst_geode",
            "minecraft:sculk_patch_deep_dark",
            "minecraft:sculk_patch_ancient_city",
            "minecraft:sculk_vein",
        ],
    ),
    placed_source(
        PlacedFeatureSource::End,
        &[
            "minecraft:end_platform",
            "minecraft:end_spike",
            "minecraft:end_gateway_return",
            "minecraft:chorus_plant",
            "minecraft:end_island_decorated",
        ],
    ),
    placed_source(
        PlacedFeatureSource::MiscOverworld,
        &[
            "minecraft:ice_spike",
            "minecraft:ice_patch",
            "minecraft:forest_rock",
            "minecraft:iceberg_packed",
            "minecraft:iceberg_blue",
            "minecraft:blue_ice",
            "minecraft:lake_lava_underground",
            "minecraft:lake_lava_surface",
            "minecraft:disk_clay",
            "minecraft:disk_gravel",
            "minecraft:disk_sand",
            "minecraft:disk_grass",
            "minecraft:freeze_top_layer",
            "minecraft:void_start_platform",
            "minecraft:desert_well",
            "minecraft:spring_lava",
            "minecraft:spring_lava_frozen",
            "minecraft:spring_water",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Nether,
        &[
            "minecraft:delta",
            "minecraft:small_basalt_columns",
            "minecraft:large_basalt_columns",
            "minecraft:basalt_blobs",
            "minecraft:blackstone_blobs",
            "minecraft:glowstone_extra",
            "minecraft:glowstone",
            "minecraft:crimson_forest_vegetation",
            "minecraft:warped_forest_vegetation",
            "minecraft:nether_sprouts",
            "minecraft:twisting_vines",
            "minecraft:weeping_vines",
            "minecraft:patch_crimson_roots",
            "minecraft:basalt_pillar",
            "minecraft:spring_delta",
            "minecraft:spring_closed",
            "minecraft:spring_closed_double",
            "minecraft:spring_open",
            "minecraft:patch_soul_fire",
            "minecraft:patch_fire",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Ore,
        &[
            "minecraft:ore_magma",
            "minecraft:ore_soul_sand",
            "minecraft:ore_gold_deltas",
            "minecraft:ore_quartz_deltas",
            "minecraft:ore_gold_nether",
            "minecraft:ore_quartz_nether",
            "minecraft:ore_gravel_nether",
            "minecraft:ore_blackstone",
            "minecraft:ore_dirt",
            "minecraft:ore_gravel",
            "minecraft:ore_granite_upper",
            "minecraft:ore_granite_lower",
            "minecraft:ore_diorite_upper",
            "minecraft:ore_diorite_lower",
            "minecraft:ore_andesite_upper",
            "minecraft:ore_andesite_lower",
            "minecraft:ore_tuff",
            "minecraft:ore_coal_upper",
            "minecraft:ore_coal_lower",
            "minecraft:ore_iron_upper",
            "minecraft:ore_iron_middle",
            "minecraft:ore_iron_small",
            "minecraft:ore_gold_extra",
            "minecraft:ore_gold",
            "minecraft:ore_gold_lower",
            "minecraft:ore_redstone",
            "minecraft:ore_redstone_lower",
            "minecraft:ore_diamond",
            "minecraft:ore_diamond_medium",
            "minecraft:ore_diamond_large",
            "minecraft:ore_diamond_buried",
            "minecraft:ore_lapis",
            "minecraft:ore_lapis_buried",
            "minecraft:ore_infested",
            "minecraft:ore_emerald",
            "minecraft:ore_ancient_debris_large",
            "minecraft:ore_debris_small",
            "minecraft:ore_copper",
            "minecraft:ore_copper_large",
            "minecraft:ore_clay",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Tree,
        &[
            "minecraft:crimson_fungi",
            "minecraft:warped_fungi",
            "minecraft:oak_checked",
            "minecraft:dark_oak_checked",
            "minecraft:pale_oak_checked",
            "minecraft:pale_oak_creaking_checked",
            "minecraft:birch_checked",
            "minecraft:acacia_checked",
            "minecraft:spruce_checked",
            "minecraft:mangrove_checked",
            "minecraft:cherry_checked",
            "minecraft:pine_on_snow",
            "minecraft:spruce_on_snow",
            "minecraft:pine_checked",
            "minecraft:jungle_tree",
            "minecraft:fancy_oak_checked",
            "minecraft:mega_jungle_tree_checked",
            "minecraft:mega_spruce_checked",
            "minecraft:mega_pine_checked",
            "minecraft:tall_mangrove_checked",
            "minecraft:jungle_bush",
            "minecraft:super_birch_bees_0002",
            "minecraft:super_birch_bees",
            "minecraft:oak_bees_0002_leaf_litter",
            "minecraft:oak_bees_002",
            "minecraft:birch_bees_0002",
            "minecraft:birch_bees_0002_leaf_litter",
            "minecraft:birch_bees_002",
            "minecraft:fancy_oak_bees_0002_leaf_litter",
            "minecraft:fancy_oak_bees_002",
            "minecraft:fancy_oak_bees",
            "minecraft:cherry_bees_005",
            "minecraft:oak_leaf_litter",
            "minecraft:dark_oak_leaf_litter",
            "minecraft:birch_leaf_litter",
            "minecraft:fancy_oak_leaf_litter",
            "minecraft:fallen_oak_tree",
            "minecraft:fallen_birch_tree",
            "minecraft:fallen_super_birch_tree",
            "minecraft:fallen_spruce_tree",
            "minecraft:fallen_jungle_tree",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Vegetation,
        &[
            "minecraft:bamboo_light",
            "minecraft:bamboo",
            "minecraft:vines",
            "minecraft:patch_sunflower",
            "minecraft:patch_pumpkin",
            "minecraft:patch_grass_plain",
            "minecraft:patch_grass_meadow",
            "minecraft:patch_grass_forest",
            "minecraft:patch_grass_badlands",
            "minecraft:patch_grass_savanna",
            "minecraft:patch_grass_normal",
            "minecraft:patch_grass_taiga_2",
            "minecraft:patch_grass_taiga",
            "minecraft:patch_grass_jungle",
            "minecraft:grass_bonemeal",
            "minecraft:patch_dead_bush_2",
            "minecraft:patch_dead_bush",
            "minecraft:patch_dead_bush_badlands",
            "minecraft:patch_dry_grass_badlands",
            "minecraft:patch_dry_grass_desert",
            "minecraft:patch_melon",
            "minecraft:patch_melon_sparse",
            "minecraft:patch_berry_common",
            "minecraft:patch_berry_rare",
            "minecraft:patch_waterlily",
            "minecraft:patch_tall_grass_2",
            "minecraft:patch_tall_grass",
            "minecraft:patch_large_fern",
            "minecraft:patch_bush",
            "minecraft:patch_leaf_litter",
            "minecraft:patch_cactus_desert",
            "minecraft:patch_cactus_decorated",
            "minecraft:patch_sugar_cane_swamp",
            "minecraft:patch_sugar_cane_desert",
            "minecraft:patch_sugar_cane_badlands",
            "minecraft:patch_sugar_cane",
            "minecraft:patch_firefly_bush_swamp",
            "minecraft:patch_firefly_bush_near_water_swamp",
            "minecraft:patch_firefly_bush_near_water",
            "minecraft:brown_mushroom_nether",
            "minecraft:red_mushroom_nether",
            "minecraft:brown_mushroom_normal",
            "minecraft:red_mushroom_normal",
            "minecraft:brown_mushroom_taiga",
            "minecraft:red_mushroom_taiga",
            "minecraft:brown_mushroom_old_growth",
            "minecraft:red_mushroom_old_growth",
            "minecraft:brown_mushroom_swamp",
            "minecraft:red_mushroom_swamp",
            "minecraft:flower_warm",
            "minecraft:flower_default",
            "minecraft:flower_flower_forest",
            "minecraft:flower_swamp",
            "minecraft:flower_plains",
            "minecraft:flower_meadow",
            "minecraft:flower_cherry",
            "minecraft:flower_pale_garden",
            "minecraft:wildflowers_birch_forest",
            "minecraft:wildflowers_meadow",
            "minecraft:trees_plains",
            "minecraft:dark_forest_vegetation",
            "minecraft:pale_garden_vegetation",
            "minecraft:flower_forest_flowers",
            "minecraft:forest_flowers",
            "minecraft:pale_garden_flowers",
            "minecraft:pale_moss_patch",
            "minecraft:trees_flower_forest",
            "minecraft:trees_meadow",
            "minecraft:trees_cherry",
            "minecraft:trees_taiga",
            "minecraft:trees_grove",
            "minecraft:trees_badlands",
            "minecraft:trees_snowy",
            "minecraft:trees_swamp",
            "minecraft:trees_windswept_savanna",
            "minecraft:trees_savanna",
            "minecraft:birch_tall",
            "minecraft:trees_birch",
            "minecraft:trees_windswept_forest",
            "minecraft:trees_windswept_hills",
            "minecraft:trees_water",
            "minecraft:trees_birch_and_oak_leaf_litter",
            "minecraft:trees_sparse_jungle",
            "minecraft:trees_old_growth_spruce_taiga",
            "minecraft:trees_old_growth_pine_taiga",
            "minecraft:trees_jungle",
            "minecraft:bamboo_vegetation",
            "minecraft:mushroom_island_vegetation",
            "minecraft:trees_mangrove",
        ],
    ),
    placed_source(
        PlacedFeatureSource::Village,
        &[
            "minecraft:pile_hay",
            "minecraft:pile_melon",
            "minecraft:pile_snow",
            "minecraft:pile_ice",
            "minecraft:pile_pumpkin",
            "minecraft:oak",
            "minecraft:acacia",
            "minecraft:spruce",
            "minecraft:pine",
            "minecraft:patch_cactus",
            "minecraft:flower_plain",
            "minecraft:patch_taiga_grass",
            "minecraft:patch_berry_bush",
        ],
    ),
];

pub const PLAINS_FEATURE_STEPS: &[&[&str]] = &[
    &[],
    &[
        "minecraft:lake_lava_underground",
        "minecraft:lake_lava_surface",
    ],
    &["minecraft:amethyst_geode"],
    &["minecraft:monster_room", "minecraft:monster_room_deep"],
    &[],
    &[],
    &[
        "minecraft:ore_dirt",
        "minecraft:ore_gravel",
        "minecraft:ore_granite_upper",
        "minecraft:ore_granite_lower",
        "minecraft:ore_diorite_upper",
        "minecraft:ore_diorite_lower",
        "minecraft:ore_andesite_upper",
        "minecraft:ore_andesite_lower",
        "minecraft:ore_tuff",
        "minecraft:ore_coal_upper",
        "minecraft:ore_coal_lower",
        "minecraft:ore_iron_upper",
        "minecraft:ore_iron_middle",
        "minecraft:ore_iron_small",
        "minecraft:ore_gold",
        "minecraft:ore_gold_lower",
        "minecraft:ore_redstone",
        "minecraft:ore_redstone_lower",
        "minecraft:ore_diamond",
        "minecraft:ore_diamond_medium",
        "minecraft:ore_diamond_large",
        "minecraft:ore_diamond_buried",
        "minecraft:ore_lapis",
        "minecraft:ore_lapis_buried",
        "minecraft:ore_copper",
        "minecraft:underwater_magma",
        "minecraft:disk_sand",
        "minecraft:disk_clay",
        "minecraft:disk_gravel",
    ],
    &[],
    &["minecraft:spring_water", "minecraft:spring_lava"],
    &[
        "minecraft:glow_lichen",
        "minecraft:patch_tall_grass_2",
        "minecraft:patch_bush",
        "minecraft:trees_plains",
        "minecraft:flower_plains",
        "minecraft:patch_grass_plain",
        "minecraft:brown_mushroom_normal",
        "minecraft:red_mushroom_normal",
        "minecraft:patch_pumpkin",
        "minecraft:patch_sugar_cane",
        "minecraft:patch_firefly_bush_near_water",
    ],
    &["minecraft:freeze_top_layer"],
];

pub const SUNFLOWER_PLAINS_FEATURE_STEPS: &[&[&str]] = &[
    PLAINS_FEATURE_STEPS[0],
    PLAINS_FEATURE_STEPS[1],
    PLAINS_FEATURE_STEPS[2],
    PLAINS_FEATURE_STEPS[3],
    PLAINS_FEATURE_STEPS[4],
    PLAINS_FEATURE_STEPS[5],
    PLAINS_FEATURE_STEPS[6],
    PLAINS_FEATURE_STEPS[7],
    PLAINS_FEATURE_STEPS[8],
    &[
        "minecraft:glow_lichen",
        "minecraft:patch_tall_grass_2",
        "minecraft:patch_sunflower",
        "minecraft:trees_plains",
        "minecraft:flower_plains",
        "minecraft:patch_grass_plain",
        "minecraft:brown_mushroom_normal",
        "minecraft:red_mushroom_normal",
        "minecraft:patch_pumpkin",
        "minecraft:patch_sugar_cane",
        "minecraft:patch_firefly_bush_near_water",
    ],
    PLAINS_FEATURE_STEPS[10],
];

pub const FOREST_FEATURE_STEPS: &[&[&str]] = &[
    PLAINS_FEATURE_STEPS[0],
    PLAINS_FEATURE_STEPS[1],
    PLAINS_FEATURE_STEPS[2],
    PLAINS_FEATURE_STEPS[3],
    PLAINS_FEATURE_STEPS[4],
    PLAINS_FEATURE_STEPS[5],
    PLAINS_FEATURE_STEPS[6],
    PLAINS_FEATURE_STEPS[7],
    PLAINS_FEATURE_STEPS[8],
    &[
        "minecraft:glow_lichen",
        "minecraft:forest_flowers",
        "minecraft:trees_birch_and_oak_leaf_litter",
        "minecraft:patch_bush",
        "minecraft:flower_default",
        "minecraft:patch_grass_forest",
        "minecraft:brown_mushroom_normal",
        "minecraft:red_mushroom_normal",
        "minecraft:patch_pumpkin",
        "minecraft:patch_sugar_cane",
        "minecraft:patch_firefly_bush_near_water",
    ],
    PLAINS_FEATURE_STEPS[10],
];

pub const PLAINS_AMBIENT_SPAWNS: &[MobSpawnerDataModel] = &[MobSpawnerDataModel {
    entity_type: "minecraft:bat",
    weight: 10,
    min_count: 8,
    max_count: 8,
}];

pub const PLAINS_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:horse",
        weight: 5,
        min_count: 2,
        max_count: 6,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:donkey",
        weight: 1,
        min_count: 1,
        max_count: 3,
    },
];

pub const FOREST_CREATURE_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:sheep",
        weight: 12,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:pig",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:chicken",
        weight: 10,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:cow",
        weight: 8,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:wolf",
        weight: 5,
        min_count: 4,
        max_count: 4,
    },
];

pub const FOREST_MONSTER_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:spider",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:zombie",
        weight: 95,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:zombie_villager",
        weight: 5,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:skeleton",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:creeper",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:slime",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:enderman",
        weight: 10,
        min_count: 1,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:witch",
        weight: 5,
        min_count: 1,
        max_count: 1,
    },
];

pub const PLAINS_MONSTER_SPAWNS: &[MobSpawnerDataModel] = &[
    MobSpawnerDataModel {
        entity_type: "minecraft:spider",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:zombie",
        weight: 90,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:zombie_villager",
        weight: 5,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:zombie_horse",
        weight: 5,
        min_count: 1,
        max_count: 1,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:skeleton",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:creeper",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:slime",
        weight: 100,
        min_count: 4,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:enderman",
        weight: 10,
        min_count: 1,
        max_count: 4,
    },
    MobSpawnerDataModel {
        entity_type: "minecraft:witch",
        weight: 5,
        min_count: 1,
        max_count: 1,
    },
];

pub const PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS: &[MobSpawnerDataModel] =
    &[MobSpawnerDataModel {
        entity_type: "minecraft:glow_squid",
        weight: 10,
        min_count: 4,
        max_count: 6,
    }];

pub const PLAINS_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: PLAINS_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: PLAINS_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

pub const FOREST_SPAWNER_GROUPS: &[MobSpawnerGroupModel] = &[
    MobSpawnerGroupModel {
        category: "ambient",
        entries: PLAINS_AMBIENT_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "axolotls",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "creature",
        entries: FOREST_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "misc",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "monster",
        entries: FOREST_MONSTER_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "underground_water_creature",
        entries: PLAINS_UNDERGROUND_WATER_CREATURE_SPAWNS,
    },
    MobSpawnerGroupModel {
        category: "water_ambient",
        entries: &[],
    },
    MobSpawnerGroupModel {
        category: "water_creature",
        entries: &[],
    },
];

const OVERWORLD_COMMON_CARVERS: &[&str] = &[
    "minecraft:cave",
    "minecraft:cave_extra_underground",
    "minecraft:canyon",
];

pub const BUILTIN_BIOME_GENERATION_SETTINGS: &[BiomeGenerationSettingsModel] = &[
    BiomeGenerationSettingsModel {
        biome: "minecraft:plains",
        carvers: OVERWORLD_COMMON_CARVERS,
        feature_steps: PLAINS_FEATURE_STEPS,
        creature_spawn_probability: 0.1,
        spawn_costs: &[],
        spawners: PLAINS_SPAWNER_GROUPS,
    },
    BiomeGenerationSettingsModel {
        biome: "minecraft:sunflower_plains",
        carvers: OVERWORLD_COMMON_CARVERS,
        feature_steps: SUNFLOWER_PLAINS_FEATURE_STEPS,
        creature_spawn_probability: 0.1,
        spawn_costs: &[],
        spawners: PLAINS_SPAWNER_GROUPS,
    },
    BiomeGenerationSettingsModel {
        biome: "minecraft:forest",
        carvers: OVERWORLD_COMMON_CARVERS,
        feature_steps: FOREST_FEATURE_STEPS,
        creature_spawn_probability: 0.1,
        spawn_costs: &[],
        spawners: FOREST_SPAWNER_GROUPS,
    },
];

pub const WORLDGEN_TYPE_REGISTRIES: &[WorldgenTypeRegistry] = &[
    WorldgenTypeRegistry {
        id: "minecraft:height_provider_type",
        entries: &[
            "minecraft:constant",
            "minecraft:uniform",
            "minecraft:biased_to_bottom",
            "minecraft:very_biased_to_bottom",
            "minecraft:trapezoid",
            "minecraft:weighted_list",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:block_predicate_type",
        entries: &[
            "minecraft:matching_blocks",
            "minecraft:matching_block_tag",
            "minecraft:matching_fluids",
            "minecraft:has_sturdy_face",
            "minecraft:solid",
            "minecraft:replaceable",
            "minecraft:would_survive",
            "minecraft:inside_world_bounds",
            "minecraft:any_of",
            "minecraft:all_of",
            "minecraft:not",
            "minecraft:true",
            "minecraft:unobstructed",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:placement_modifier_type",
        entries: &[
            "minecraft:block_predicate_filter",
            "minecraft:rarity_filter",
            "minecraft:surface_water_depth_filter",
            "minecraft:biome",
            "minecraft:count",
            "minecraft:noise_based_count",
            "minecraft:noise_threshold_count",
            "minecraft:count_on_every_layer",
            "minecraft:environment_scan",
            "minecraft:heightmap",
            "minecraft:height_range",
            "minecraft:in_square",
            "minecraft:random_offset",
            "minecraft:fixed_placement",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:trunk_placer_type",
        entries: &[
            "minecraft:straight_trunk_placer",
            "minecraft:forking_trunk_placer",
            "minecraft:giant_trunk_placer",
            "minecraft:mega_jungle_trunk_placer",
            "minecraft:dark_oak_trunk_placer",
            "minecraft:fancy_trunk_placer",
            "minecraft:bending_trunk_placer",
            "minecraft:upwards_branching_trunk_placer",
            "minecraft:cherry_trunk_placer",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:foliage_placer_type",
        entries: &[
            "minecraft:blob_foliage_placer",
            "minecraft:spruce_foliage_placer",
            "minecraft:pine_foliage_placer",
            "minecraft:acacia_foliage_placer",
            "minecraft:bush_foliage_placer",
            "minecraft:fancy_foliage_placer",
            "minecraft:jungle_foliage_placer",
            "minecraft:mega_pine_foliage_placer",
            "minecraft:dark_oak_foliage_placer",
            "minecraft:random_spread_foliage_placer",
            "minecraft:cherry_foliage_placer",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:block_state_provider_type",
        entries: &[
            "minecraft:simple_state_provider",
            "minecraft:weighted_state_provider",
            "minecraft:noise_threshold_provider",
            "minecraft:noise_provider",
            "minecraft:dual_noise_provider",
            "minecraft:rotated_block_provider",
            "minecraft:randomized_int_state_provider",
            "minecraft:rule_based_state_provider",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:tree_decorator_type",
        entries: &[
            "minecraft:trunk_vine",
            "minecraft:leave_vine",
            "minecraft:pale_moss",
            "minecraft:creaking_heart",
            "minecraft:cocoa",
            "minecraft:beehive",
            "minecraft:alter_ground",
            "minecraft:attached_to_leaves",
            "minecraft:place_on_ground",
            "minecraft:attached_to_logs",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:feature_size_type",
        entries: &[
            "minecraft:two_layers_feature_size",
            "minecraft:three_layers_feature_size",
        ],
    },
    WorldgenTypeRegistry {
        id: "minecraft:root_placer_type",
        entries: &["minecraft:mangrove_root_placer"],
    },
];

pub const FEATURE_BEHAVIOR_MODELS: &[FeatureBehaviorModel] = &[
    FeatureBehaviorModel { feature_type: "minecraft:tree", behavior: "validates roots/trunk/foliage/decorators, computes bounding box, updates leaves", success_condition: "at least one trunk placer node and all bounding-box updates succeed" },
    FeatureBehaviorModel { feature_type: "minecraft:vegetation_patch", behavior: "samples xz radius, validates replaceable ground, places depth columns, optionally places vegetation", success_condition: "at least one ground block or vegetation feature is placed" },
    FeatureBehaviorModel { feature_type: "minecraft:spring_feature", behavior: "requires valid block above, optional valid block below, exact adjacent rock and hole counts", success_condition: "rockCount == config.rockCount && holeCount == config.holeCount" },
    FeatureBehaviorModel { feature_type: "minecraft:ore", behavior: "builds sinusoidal vein ellipsoids, culls enclosed spheres, tests target states and discard chance", success_condition: "one or more ore blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:scattered_ore", behavior: "uses ore target tests with scattered attempts", success_condition: "one or more ore blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:disk", behavior: "fills xz disk columns while source blocks and vertical bounds match", success_condition: "one or more disk blocks are placed" },
    FeatureBehaviorModel { feature_type: "minecraft:lake", behavior: "carves an ellipsoid cavity, validates solid/fluid boundary constraints, fills fluid and barrier blocks", success_condition: "cavity validation succeeds" },
    FeatureBehaviorModel { feature_type: "minecraft:geode", behavior: "samples layered ellipsoid thresholds for filling/inner/alternate/budding states and cracks", success_condition: "all sampled positions are evaluated and set through geode layers" },
    FeatureBehaviorModel { feature_type: "minecraft:fossil", behavior: "selects fossil templates, offsets by random rotation and integrity, places fossil and overlay processors", success_condition: "template placement succeeds" },
    FeatureBehaviorModel { feature_type: "minecraft:monster_room", behavior: "validates solid floor/ceiling, counts wall openings, builds cobble shell, chests, and spawner", success_condition: "1 <= openingCount <= 5" },
];

pub const MONSTER_ROOM_BOUNDS: MonsterRoomBounds = MonsterRoomBounds {
    min_y: -1,
    max_y: 4,
    min_openings: 1,
    max_openings: 5,
};

pub const STRUCTURE_TYPES: &[&str] = &[
    "minecraft:buried_treasure",
    "minecraft:desert_pyramid",
    "minecraft:end_city",
    "minecraft:fortress",
    "minecraft:igloo",
    "minecraft:jigsaw",
    "minecraft:jungle_temple",
    "minecraft:mineshaft",
    "minecraft:nether_fossil",
    "minecraft:ocean_monument",
    "minecraft:ocean_ruin",
    "minecraft:ruined_portal",
    "minecraft:shipwreck",
    "minecraft:stronghold",
    "minecraft:swamp_hut",
    "minecraft:woodland_mansion",
];

pub const BUILTIN_STRUCTURES: &[&str] = &[
    "minecraft:pillager_outpost",
    "minecraft:mineshaft",
    "minecraft:mineshaft_mesa",
    "minecraft:mansion",
    "minecraft:jungle_pyramid",
    "minecraft:desert_pyramid",
    "minecraft:igloo",
    "minecraft:shipwreck",
    "minecraft:shipwreck_beached",
    "minecraft:swamp_hut",
    "minecraft:stronghold",
    "minecraft:monument",
    "minecraft:ocean_ruin_cold",
    "minecraft:ocean_ruin_warm",
    "minecraft:fortress",
    "minecraft:nether_fossil",
    "minecraft:end_city",
    "minecraft:buried_treasure",
    "minecraft:bastion_remnant",
    "minecraft:village_plains",
    "minecraft:village_desert",
    "minecraft:village_savanna",
    "minecraft:village_snowy",
    "minecraft:village_taiga",
    "minecraft:ruined_portal",
    "minecraft:ruined_portal_desert",
    "minecraft:ruined_portal_jungle",
    "minecraft:ruined_portal_swamp",
    "minecraft:ruined_portal_mountain",
    "minecraft:ruined_portal_ocean",
    "minecraft:ruined_portal_nether",
    "minecraft:ancient_city",
    "minecraft:trail_ruins",
    "minecraft:trial_chambers",
];

pub const BUILTIN_STRUCTURE_SETS: &[StructureSetEntry] = &[
    structure_set(
        "minecraft:villages",
        &[
            "minecraft:village_plains",
            "minecraft:village_desert",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
        ],
        random_spread(34, 8, RandomSpreadType::Linear, 10387312),
    ),
    structure_set(
        "minecraft:desert_pyramids",
        &["minecraft:desert_pyramid"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357617),
    ),
    structure_set(
        "minecraft:igloos",
        &["minecraft:igloo"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357618),
    ),
    structure_set(
        "minecraft:jungle_temples",
        &["minecraft:jungle_pyramid"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357619),
    ),
    structure_set(
        "minecraft:swamp_huts",
        &["minecraft:swamp_hut"],
        random_spread(32, 8, RandomSpreadType::Linear, 14357620),
    ),
    structure_set(
        "minecraft:pillager_outposts",
        &["minecraft:pillager_outpost"],
        random_spread(32, 8, RandomSpreadType::Linear, 165745296),
    ),
    structure_set(
        "minecraft:ancient_cities",
        &["minecraft:ancient_city"],
        random_spread(24, 8, RandomSpreadType::Linear, 20083232),
    ),
    structure_set(
        "minecraft:ocean_monuments",
        &["minecraft:monument"],
        random_spread(32, 5, RandomSpreadType::Triangular, 10387313),
    ),
    structure_set(
        "minecraft:woodland_mansions",
        &["minecraft:mansion"],
        random_spread(80, 20, RandomSpreadType::Triangular, 10387319),
    ),
    structure_set(
        "minecraft:buried_treasures",
        &["minecraft:buried_treasure"],
        random_spread(1, 0, RandomSpreadType::Linear, 0),
    ),
    structure_set(
        "minecraft:mineshafts",
        &["minecraft:mineshaft", "minecraft:mineshaft_mesa"],
        random_spread(1, 0, RandomSpreadType::Linear, 0),
    ),
    structure_set(
        "minecraft:ruined_portals",
        &[
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_swamp",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_nether",
        ],
        random_spread(40, 15, RandomSpreadType::Linear, 34222645),
    ),
    structure_set(
        "minecraft:shipwrecks",
        &["minecraft:shipwreck", "minecraft:shipwreck_beached"],
        random_spread(24, 4, RandomSpreadType::Linear, 165745295),
    ),
    structure_set(
        "minecraft:ocean_ruins",
        &["minecraft:ocean_ruin_cold", "minecraft:ocean_ruin_warm"],
        random_spread(20, 8, RandomSpreadType::Linear, 14357621),
    ),
    structure_set(
        "minecraft:nether_complexes",
        &["minecraft:fortress", "minecraft:bastion_remnant"],
        random_spread(27, 4, RandomSpreadType::Linear, 30084232),
    ),
    structure_set(
        "minecraft:nether_fossils",
        &["minecraft:nether_fossil"],
        random_spread(2, 1, RandomSpreadType::Linear, 14357921),
    ),
    structure_set(
        "minecraft:end_cities",
        &["minecraft:end_city"],
        random_spread(20, 11, RandomSpreadType::Triangular, 10387313),
    ),
    structure_set(
        "minecraft:strongholds",
        &["minecraft:stronghold"],
        StructurePlacementKind::ConcentricRings {
            distance: 32,
            spread: 3,
            count: 128,
        },
    ),
    structure_set(
        "minecraft:trail_ruins",
        &["minecraft:trail_ruins"],
        random_spread(34, 8, RandomSpreadType::Linear, 83469867),
    ),
    structure_set(
        "minecraft:trial_chambers",
        &["minecraft:trial_chambers"],
        random_spread(34, 12, RandomSpreadType::Linear, 94251327),
    ),
];

pub const STRUCTURE_FAMILIES: &[StructureFamilyEntry] = &[
    structure_family(
        StructureFamily::Village,
        &[
            "minecraft:village_plains",
            "minecraft:village_desert",
            "minecraft:village_savanna",
            "minecraft:village_snowy",
            "minecraft:village_taiga",
        ],
    ),
    structure_family(StructureFamily::Stronghold, &["minecraft:stronghold"]),
    structure_family(
        StructureFamily::Mineshaft,
        &["minecraft:mineshaft", "minecraft:mineshaft_mesa"],
    ),
    structure_family(StructureFamily::OceanMonument, &["minecraft:monument"]),
    structure_family(StructureFamily::WoodlandMansion, &["minecraft:mansion"]),
    structure_family(StructureFamily::Bastion, &["minecraft:bastion_remnant"]),
    structure_family(StructureFamily::Fortress, &["minecraft:fortress"]),
    structure_family(StructureFamily::AncientCity, &["minecraft:ancient_city"]),
    structure_family(
        StructureFamily::TrialChambers,
        &["minecraft:trial_chambers"],
    ),
    structure_family(StructureFamily::EndCity, &["minecraft:end_city"]),
    structure_family(
        StructureFamily::RuinedPortal,
        &[
            "minecraft:ruined_portal",
            "minecraft:ruined_portal_desert",
            "minecraft:ruined_portal_jungle",
            "minecraft:ruined_portal_swamp",
            "minecraft:ruined_portal_mountain",
            "minecraft:ruined_portal_ocean",
            "minecraft:ruined_portal_nether",
        ],
    ),
    structure_family(
        StructureFamily::Shipwreck,
        &["minecraft:shipwreck", "minecraft:shipwreck_beached"],
    ),
    structure_family(
        StructureFamily::BuriedTreasure,
        &["minecraft:buried_treasure"],
    ),
    structure_family(StructureFamily::Igloo, &["minecraft:igloo"]),
    structure_family(StructureFamily::SwampHut, &["minecraft:swamp_hut"]),
    structure_family(
        StructureFamily::PillagerOutpost,
        &["minecraft:pillager_outpost"],
    ),
    structure_family(StructureFamily::TrailRuins, &["minecraft:trail_ruins"]),
    structure_family(StructureFamily::Fossil, &["minecraft:nether_fossil"]),
    structure_family(
        StructureFamily::DesertPyramid,
        &["minecraft:desert_pyramid"],
    ),
    structure_family(StructureFamily::JungleTemple, &["minecraft:jungle_pyramid"]),
    structure_family(
        StructureFamily::OceanRuins,
        &["minecraft:ocean_ruin_cold", "minecraft:ocean_ruin_warm"],
    ),
];

pub const STRUCTURE_POOL_ELEMENT_TYPES: &[&str] = &[
    "minecraft:single_pool_element",
    "minecraft:list_pool_element",
    "minecraft:feature_pool_element",
    "minecraft:empty_pool_element",
    "minecraft:legacy_single_pool_element",
];

pub const STRUCTURE_PROCESSOR_TYPES: &[&str] = &[
    "minecraft:block_ignore",
    "minecraft:block_rot",
    "minecraft:gravity",
    "minecraft:jigsaw_replacement",
    "minecraft:rule",
    "minecraft:nop",
    "minecraft:block_age",
    "minecraft:blackstone_replace",
    "minecraft:lava_submerged_block",
    "minecraft:protected_blocks",
    "minecraft:capped",
];

pub const STRUCTURE_RULE_TEST_TYPES: &[&str] = &[
    "minecraft:always_true",
    "minecraft:block_match",
    "minecraft:blockstate_match",
    "minecraft:tag_match",
    "minecraft:random_block_match",
    "minecraft:random_blockstate_match",
];

pub const STRUCTURE_POS_RULE_TEST_TYPES: &[&str] = &[
    "minecraft:always_true",
    "minecraft:linear_pos",
    "minecraft:axis_aligned_linear_pos",
];

pub const STRUCTURE_PIECE_TYPES: &[&str] = &[
    "mscorridor",
    "mscrossing",
    "msroom",
    "msstairs",
    "nebcr",
    "nebef",
    "nebs",
    "neccs",
    "nectb",
    "nece",
    "nescsc",
    "nesclt",
    "nesc",
    "nescrt",
    "necsr",
    "nemt",
    "nerc",
    "nesr",
    "nestart",
    "shcc",
    "shfc",
    "sh5c",
    "shlt",
    "shli",
    "shpr",
    "shph",
    "shrt",
    "shrc",
    "shsd",
    "shstart",
    "shs",
    "shssd",
    "tejp",
    "orp",
    "iglu",
    "rupo",
    "tesh",
    "tedp",
    "omb",
    "omcr",
    "omdxr",
    "omdxyr",
    "omdyr",
    "omdyzr",
    "omdzr",
    "omentry",
    "ompenthouse",
    "omsimple",
    "omsimplet",
    "omwr",
    "ecp",
    "wmp",
    "btp",
    "shipwreck",
    "nefos",
    "jigsaw",
];

pub const STRUCTURE_PROCESSOR_LISTS: &[&str] = &[
    "minecraft:empty",
    "minecraft:zombie_plains",
    "minecraft:zombie_savanna",
    "minecraft:zombie_snowy",
    "minecraft:zombie_taiga",
    "minecraft:zombie_desert",
    "minecraft:mossify_10_percent",
    "minecraft:mossify_20_percent",
    "minecraft:mossify_70_percent",
    "minecraft:street_plains",
    "minecraft:street_savanna",
    "minecraft:street_snowy_or_taiga",
    "minecraft:farm_plains",
    "minecraft:farm_savanna",
    "minecraft:farm_snowy",
    "minecraft:farm_taiga",
    "minecraft:farm_desert",
    "minecraft:outpost_rot",
    "minecraft:bottom_rampart",
    "minecraft:treasure_rooms",
    "minecraft:housing",
    "minecraft:side_wall_degradation",
    "minecraft:stable_degradation",
    "minecraft:bastion_generic_degradation",
    "minecraft:rampart_degradation",
    "minecraft:entrance_replacement",
    "minecraft:bridge",
    "minecraft:roof",
    "minecraft:high_wall",
    "minecraft:high_rampart",
    "minecraft:fossil_rot",
    "minecraft:fossil_coal",
    "minecraft:fossil_diamonds",
    "minecraft:ancient_city_start_degradation",
    "minecraft:ancient_city_generic_degradation",
    "minecraft:ancient_city_walls_degradation",
    "minecraft:trail_ruins_houses_archaeology",
    "minecraft:trail_ruins_roads_archaeology",
    "minecraft:trail_ruins_tower_top_archaeology",
    "minecraft:trial_chambers_copper_bulb_degradation",
];

pub const JIGSAW_POOL_BOOTSTRAP_SOURCES: &[JigsawPoolBootstrapSource] = &[
    JigsawPoolBootstrapSource {
        source_file: "AncientCityStructurePieces.java",
        registrations: 1,
    },
    JigsawPoolBootstrapSource {
        source_file: "AncientCityStructurePools.java",
        registrations: 6,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionBridgePools.java",
        registrations: 7,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionHoglinStablePools.java",
        registrations: 13,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionHousingUnitsPools.java",
        registrations: 15,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionPieces.java",
        registrations: 1,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionSharedPools.java",
        registrations: 4,
    },
    JigsawPoolBootstrapSource {
        source_file: "BastionTreasureRoomPools.java",
        registrations: 20,
    },
    JigsawPoolBootstrapSource {
        source_file: "DesertVillagePools.java",
        registrations: 12,
    },
    JigsawPoolBootstrapSource {
        source_file: "PillagerOutpostPools.java",
        registrations: 4,
    },
    JigsawPoolBootstrapSource {
        source_file: "PlainVillagePools.java",
        registrations: 17,
    },
    JigsawPoolBootstrapSource {
        source_file: "Pools.java",
        registrations: 2,
    },
    JigsawPoolBootstrapSource {
        source_file: "SavannaVillagePools.java",
        registrations: 12,
    },
    JigsawPoolBootstrapSource {
        source_file: "SnowyVillagePools.java",
        registrations: 11,
    },
    JigsawPoolBootstrapSource {
        source_file: "TaigaVillagePools.java",
        registrations: 10,
    },
    JigsawPoolBootstrapSource {
        source_file: "TrailRuinsStructurePools.java",
        registrations: 7,
    },
    JigsawPoolBootstrapSource {
        source_file: "TrialChambersStructurePools.java",
        registrations: 34,
    },
];

pub const BLENDING_CONSTANTS: BlendingConstants = BlendingConstants {
    height_blending_range_cells: 27,
    height_blending_range_chunks: 7,
    density_blending_range_cells: 2,
    density_blending_range_chunks: 2,
    old_chunk_xz_radius: 8,
    cell_width: 4,
    cell_height: 8,
    cell_ratio: 2,
};

pub const BLENDING_CELL_COLUMN_COUNT: usize = 16;
pub const BLENDING_NO_VALUE: f64 = f64::MAX;

pub const UPGRADE_DATA_MODEL: UpgradeDataModel = UpgradeDataModel {
    tag_indices: "Indices",
    tag_sides: "Sides",
    tag_neighbor_block_ticks: "neighbor_block_ticks",
    tag_neighbor_fluid_ticks: "neighbor_fluid_ticks",
    block_fixers: &["blacklist", "default", "chest", "leaves", "stem_block"],
    chunky_fixers: &["leaves"],
};

pub const SPAWN_SELECTION_CONSTANTS: SpawnSelectionConstants = SpawnSelectionConstants {
    initial_chunk_search_radius: 5,
    player_spawn_ticket_radius: 3,
    spawn_search_absolute_max_attempts: 1024,
    default_respawn_radius: 10,
    small_search_coprime_threshold: 16,
    large_search_coprime: 17,
};

const fn structure_family(
    family: StructureFamily,
    structures: &'static [&'static str],
) -> StructureFamilyEntry {
    StructureFamilyEntry { family, structures }
}

const fn structure_set(
    id: &'static str,
    structures: &'static [&'static str],
    placement: StructurePlacementKind,
) -> StructureSetEntry {
    StructureSetEntry {
        id,
        structures,
        placement,
    }
}

const fn random_spread(
    spacing: i32,
    separation: i32,
    spread_type: RandomSpreadType,
    salt: i32,
) -> StructurePlacementKind {
    StructurePlacementKind::RandomSpread {
        spacing,
        separation,
        salt,
        spread_type,
    }
}

pub fn validate_random_spread_placement(spacing: i32, separation: i32) -> Result<(), String> {
    if !(0..=4096).contains(&spacing) || !(0..=4096).contains(&separation) {
        return Err("Random spread spacing and separation must be in 0..=4096".to_string());
    }
    if spacing <= separation {
        return Err("Spacing has to be larger than separation".to_string());
    }
    Ok(())
}

pub fn random_spread_potential_structure_chunk(
    seed: i64,
    source_x: i32,
    source_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
    spread_type: RandomSpreadType,
) -> Result<ChunkPos, String> {
    validate_random_spread_placement(spacing, separation)?;
    let spaced_grid_x = source_x.div_euclid(spacing);
    let spaced_grid_z = source_z.div_euclid(spacing);
    let mut random = LegacyRandom::new(large_feature_seed_with_salt(
        seed,
        spaced_grid_x,
        spaced_grid_z,
        salt,
    ));
    let limit = spacing - separation;
    let spread_x = spread_type.evaluate(&mut random, limit);
    let spread_z = spread_type.evaluate(&mut random, limit);
    Ok(ChunkPos {
        x: spaced_grid_x * spacing + spread_x,
        z: spaced_grid_z * spacing + spread_z,
    })
}

pub fn random_spread_is_placement_chunk(
    seed: i64,
    source_x: i32,
    source_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
    spread_type: RandomSpreadType,
) -> Result<bool, String> {
    let chunk = random_spread_potential_structure_chunk(
        seed,
        source_x,
        source_z,
        spacing,
        separation,
        salt,
        spread_type,
    )?;
    Ok(chunk.x == source_x && chunk.z == source_z)
}

pub fn validate_structure_frequency(frequency: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&frequency) {
        return Err("Structure placement frequency must be in 0.0..=1.0".to_string());
    }
    Ok(())
}

pub fn structure_frequency_reducer_should_generate(
    method: FrequencyReductionMethod,
    seed: i64,
    salt: i32,
    source_x: i32,
    source_z: i32,
    frequency: f32,
) -> Result<bool, String> {
    validate_structure_frequency(frequency)?;
    if frequency >= 1.0 {
        return Ok(true);
    }
    if frequency <= 0.0 {
        return Ok(false);
    }
    Ok(match method {
        FrequencyReductionMethod::Default => {
            let mut random =
                LegacyRandom::new(large_feature_seed_with_salt(seed, salt, source_x, source_z));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType1 => {
            let cx = source_x >> 4;
            let cz = source_z >> 4;
            let mut random = LegacyRandom::new((cx ^ (cz << 4)) as i64 ^ seed);
            random.next_i32();
            random.next_i32_bound((1.0 / frequency) as i32) == 0
        }
        FrequencyReductionMethod::LegacyType2 => {
            let mut random = LegacyRandom::new(large_feature_seed_with_salt(
                seed, source_x, source_z, 10_387_320,
            ));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType3 => {
            let mut random = LegacyRandom::new(crate::random_source::large_feature_seed(
                seed, source_x, source_z,
            ));
            random.next_f64() < frequency as f64
        }
    })
}

pub fn validate_structure_locate_offset(offset: BlockPos) -> Result<(), String> {
    if (-16..=16).contains(&offset.x)
        && (-16..=16).contains(&offset.y)
        && (-16..=16).contains(&offset.z)
    {
        Ok(())
    } else {
        Err("Structure locate offset components must be in -16..=16".to_string())
    }
}

pub fn structure_locate_pos(
    chunk_pos: ChunkPos,
    locate_offset: BlockPos,
) -> Result<BlockPos, String> {
    validate_structure_locate_offset(locate_offset)?;
    Ok(BlockPos {
        x: chunk_pos.x * 16 + locate_offset.x,
        y: locate_offset.y,
        z: chunk_pos.z * 16 + locate_offset.z,
    })
}

pub fn validate_concentric_rings_placement(
    distance: i32,
    spread: i32,
    count: i32,
) -> Result<(), String> {
    if !(0..=1023).contains(&distance) || !(0..=1023).contains(&spread) {
        return Err("Concentric rings distance and spread must be in 0..=1023".to_string());
    }
    if !(1..=4095).contains(&count) {
        return Err("Concentric rings count must be in 1..=4095".to_string());
    }
    Ok(())
}

pub fn concentric_ring_initial_candidates(
    seed: i64,
    distance: i32,
    spread: i32,
    count: i32,
) -> Result<Vec<ConcentricRingPlacementCandidate>, String> {
    validate_concentric_rings_placement(distance, spread, count)?;
    let mut random = LegacyRandom::new(seed);
    let mut angle = random.next_f64() * std::f64::consts::PI * 2.0;
    let mut position_in_circle = 0;
    let mut circle = 0;
    let mut current_spread = spread;
    let mut candidates = Vec::with_capacity(count as usize);
    for index in 0..count {
        let dist = 4.0 * distance as f64
            + (distance * circle * 6) as f64
            + (random.next_f64() - 0.5) * (distance as f64 * 2.5);
        let initial_x = (angle.cos() * dist).round() as i32;
        let initial_z = (angle.sin() * dist).round() as i32;
        let _biome_search_generator = random.fork();
        candidates.push(ConcentricRingPlacementCandidate {
            index,
            circle,
            chunk_pos: ChunkPos {
                x: initial_x,
                z: initial_z,
            },
        });
        angle += (std::f64::consts::PI * 2.0) / current_spread as f64;
        position_in_circle += 1;
        if position_in_circle == current_spread {
            circle += 1;
            position_in_circle = 0;
            current_spread += 2 * current_spread / (circle + 1);
            current_spread = current_spread.min(count - index);
            angle += random.next_f64() * std::f64::consts::PI * 2.0;
        }
    }
    Ok(candidates)
}

pub fn concentric_rings_is_placement_chunk(
    ring_positions: &[ChunkPos],
    source_x: i32,
    source_z: i32,
) -> bool {
    ring_positions
        .iter()
        .any(|position| position.x == source_x && position.z == source_z)
}

pub fn validate_structure_exclusion_zone(
    zone: StructureExclusionZoneModel,
) -> Result<StructureExclusionZoneModel, String> {
    if (1..=16).contains(&zone.chunk_count) {
        Ok(zone)
    } else {
        Err("Structure exclusion zone chunk_count must be in 1..=16".to_string())
    }
}

pub fn structure_has_chunk_in_range(
    structure_chunks: &[ChunkPos],
    source_x: i32,
    source_z: i32,
    range: i32,
) -> bool {
    (source_x - range..=source_x + range).any(|test_x| {
        (source_z - range..=source_z + range).any(|test_z| {
            structure_chunks
                .iter()
                .any(|chunk| chunk.x == test_x && chunk.z == test_z)
        })
    })
}

pub fn structure_exclusion_zone_forbids(
    zone: StructureExclusionZoneModel,
    other_structure_chunks: &[ChunkPos],
    source_x: i32,
    source_z: i32,
) -> Result<bool, String> {
    validate_structure_exclusion_zone(zone)?;
    Ok(structure_has_chunk_in_range(
        other_structure_chunks,
        source_x,
        source_z,
        zone.chunk_count,
    ))
}

impl StructureBoundingBoxModel {
    pub fn union(self, other: StructureBoundingBoxModel) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            min_z: self.min_z.min(other.min_z),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
            max_z: self.max_z.max(other.max_z),
        }
    }

    pub fn intersects(self, other: StructureBoundingBoxModel) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }

    pub fn is_inside(self, pos: BlockPos) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.z >= self.min_z
            && pos.z <= self.max_z
            && pos.y >= self.min_y
            && pos.y <= self.max_y
    }

    pub fn center(self) -> BlockPos {
        BlockPos {
            x: self.min_x + (self.max_x - self.min_x + 1) / 2,
            y: self.min_y + (self.max_y - self.min_y + 1) / 2,
            z: self.min_z + (self.max_z - self.min_z + 1) / 2,
        }
    }

    pub fn inflated_by(self, amount: i32) -> StructureBoundingBoxModel {
        StructureBoundingBoxModel {
            min_x: self.min_x - amount,
            min_y: self.min_y - amount,
            min_z: self.min_z - amount,
            max_x: self.max_x + amount,
            max_y: self.max_y + amount,
            max_z: self.max_z + amount,
        }
    }
}

pub fn structure_make_bounding_box(
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
    width: i32,
    height: i32,
    depth: i32,
) -> StructureBoundingBoxModel {
    match direction {
        HorizontalDirection::North | HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x + width - 1,
            max_y: y + height - 1,
            max_z: z + depth - 1,
        },
        HorizontalDirection::West | HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x + depth - 1,
            max_y: y + height - 1,
            max_z: z + width - 1,
        },
    }
}

pub fn structure_orient_box(
    foot: BlockPos,
    offset: BlockPos,
    width: i32,
    height: i32,
    depth: i32,
    direction: HorizontalDirection,
) -> StructureBoundingBoxModel {
    match direction {
        HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: foot.x + offset.x,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.z,
            max_x: foot.x + width - 1 + offset.x,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + depth - 1 + offset.z,
        },
        HorizontalDirection::North => StructureBoundingBoxModel {
            min_x: foot.x + offset.x,
            min_y: foot.y + offset.y,
            min_z: foot.z - depth + 1 + offset.z,
            max_x: foot.x + width - 1 + offset.x,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + offset.z,
        },
        HorizontalDirection::West => StructureBoundingBoxModel {
            min_x: foot.x - depth + 1 + offset.z,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.x,
            max_x: foot.x + offset.z,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + width - 1 + offset.x,
        },
        HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: foot.x + offset.z,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.x,
            max_x: foot.x + depth - 1 + offset.z,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + width - 1 + offset.x,
        },
    }
}

pub fn structure_piece_world_pos(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    x: i32,
    y: i32,
    z: i32,
) -> BlockPos {
    let Some(orientation) = orientation else {
        return BlockPos { x, y, z };
    };
    let world_x = match orientation {
        HorizontalDirection::North | HorizontalDirection::South => bounding_box.min_x + x,
        HorizontalDirection::West => bounding_box.max_x - z,
        HorizontalDirection::East => bounding_box.min_x + z,
    };
    let world_z = match orientation {
        HorizontalDirection::North => bounding_box.max_z - z,
        HorizontalDirection::South => bounding_box.min_z + z,
        HorizontalDirection::West | HorizontalDirection::East => bounding_box.min_z + x,
    };
    BlockPos {
        x: world_x,
        y: y + bounding_box.min_y,
        z: world_z,
    }
}

pub fn structure_piece_is_close_to_chunk(
    piece: StructurePieceModel,
    chunk_pos: ChunkPos,
    distance: i32,
) -> bool {
    let chunk_min_x = chunk_pos.x * 16;
    let chunk_min_z = chunk_pos.z * 16;
    piece.bounding_box.intersects(StructureBoundingBoxModel {
        min_x: chunk_min_x - distance,
        min_y: i32::MIN,
        min_z: chunk_min_z - distance,
        max_x: chunk_min_x + 15 + distance,
        max_y: i32::MAX,
        max_z: chunk_min_z + 15 + distance,
    })
}

pub fn structure_piece_locator_position(piece: StructurePieceModel) -> BlockPos {
    piece.bounding_box.center()
}

pub fn structure_piece_place_block(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    local_pos: BlockPos,
    state: &'static str,
    edge: bool,
) -> Option<StructurePiecePlacementBlock> {
    let world_pos = structure_piece_world_pos(
        bounding_box,
        orientation,
        local_pos.x,
        local_pos.y,
        local_pos.z,
    );
    chunk_bb
        .is_inside(world_pos)
        .then_some(StructurePiecePlacementBlock {
            local_pos,
            world_pos,
            state,
            edge,
        })
}

pub fn structure_piece_generate_air_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
) -> Vec<StructurePiecePlacementBlock> {
    structure_piece_generate_box(
        bounding_box,
        orientation,
        chunk_bb,
        min,
        max,
        "minecraft:air",
        "minecraft:air",
        false,
        |_| false,
    )
}

pub fn structure_piece_generate_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
    edge_block: &'static str,
    fill_block: &'static str,
    skip_air: bool,
    mut is_existing_air: impl FnMut(BlockPos) -> bool,
) -> Vec<StructurePiecePlacementBlock> {
    let mut blocks = Vec::new();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                let local_pos = BlockPos { x, y, z };
                let world_pos = structure_piece_world_pos(bounding_box, orientation, x, y, z);
                if skip_air && is_existing_air(world_pos) {
                    continue;
                }
                let edge = y == min.y
                    || y == max.y
                    || x == min.x
                    || x == max.x
                    || z == min.z
                    || z == max.z;
                if let Some(block) = structure_piece_place_block(
                    bounding_box,
                    orientation,
                    chunk_bb,
                    local_pos,
                    if edge { edge_block } else { fill_block },
                    edge,
                ) {
                    blocks.push(block);
                }
            }
        }
    }
    blocks
}

pub fn structure_piece_generate_maybe_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    random_values: &[f32],
    probability: f32,
    min: BlockPos,
    max: BlockPos,
    edge_block: &'static str,
    fill_block: &'static str,
    skip_air: bool,
    has_to_be_inside: bool,
    mut is_existing_air: impl FnMut(BlockPos) -> bool,
    mut is_interior: impl FnMut(BlockPos) -> bool,
) -> Vec<StructurePiecePlacementBlock> {
    let mut blocks = Vec::new();
    let mut random_index = 0usize;
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                let random_value = random_values.get(random_index).copied().unwrap_or(1.0);
                random_index += 1;
                if random_value > probability {
                    continue;
                }
                let local_pos = BlockPos { x, y, z };
                let world_pos = structure_piece_world_pos(bounding_box, orientation, x, y, z);
                if skip_air && is_existing_air(world_pos) {
                    continue;
                }
                if has_to_be_inside && !is_interior(world_pos) {
                    continue;
                }
                let edge = y == min.y
                    || y == max.y
                    || x == min.x
                    || x == max.x
                    || z == min.z
                    || z == max.z;
                if let Some(block) = structure_piece_place_block(
                    bounding_box,
                    orientation,
                    chunk_bb,
                    local_pos,
                    if edge { edge_block } else { fill_block },
                    edge,
                ) {
                    blocks.push(block);
                }
            }
        }
    }
    blocks
}

pub fn structure_piece_maybe_generate_block(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    random_value: f32,
    probability: f32,
    local_pos: BlockPos,
    state: &'static str,
) -> Option<StructurePiecePlacementBlock> {
    (random_value < probability)
        .then(|| {
            structure_piece_place_block(bounding_box, orientation, chunk_bb, local_pos, state, true)
        })
        .flatten()
}

impl TerrainAdjustmentModel {
    pub fn id(self) -> &'static str {
        match self {
            TerrainAdjustmentModel::None => "none",
            TerrainAdjustmentModel::Bury => "bury",
            TerrainAdjustmentModel::BeardThin => "beard_thin",
            TerrainAdjustmentModel::BeardBox => "beard_box",
            TerrainAdjustmentModel::Encapsulate => "encapsulate",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "none" => Some(TerrainAdjustmentModel::None),
            "bury" => Some(TerrainAdjustmentModel::Bury),
            "beard_thin" => Some(TerrainAdjustmentModel::BeardThin),
            "beard_box" => Some(TerrainAdjustmentModel::BeardBox),
            "encapsulate" => Some(TerrainAdjustmentModel::Encapsulate),
            _ => None,
        }
    }

    pub fn jigsaw_edge_needed(self) -> i32 {
        match self {
            TerrainAdjustmentModel::None => 0,
            TerrainAdjustmentModel::Bury
            | TerrainAdjustmentModel::BeardThin
            | TerrainAdjustmentModel::BeardBox
            | TerrainAdjustmentModel::Encapsulate => 12,
        }
    }
}

pub fn structure_adjust_bounding_box(
    terrain_adjustment: TerrainAdjustmentModel,
    bounding_box: StructureBoundingBoxModel,
) -> StructureBoundingBoxModel {
    if terrain_adjustment == TerrainAdjustmentModel::None {
        bounding_box
    } else {
        bounding_box.inflated_by(12)
    }
}

pub fn jigsaw_max_distance_with_terrain_is_valid(
    horizontal_max_distance_from_center: i32,
    terrain_adjustment: TerrainAdjustmentModel,
) -> bool {
    horizontal_max_distance_from_center + terrain_adjustment.jigsaw_edge_needed() <= 128
}

impl StructureStartModel {
    pub fn invalid() -> Self {
        Self {
            structure: None,
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.pieces.is_empty()
    }

    pub fn max_references(&self) -> i32 {
        1
    }

    pub fn can_be_referenced(&self) -> bool {
        self.references < self.max_references()
    }

    pub fn add_reference(&mut self) {
        self.references += 1;
    }

    pub fn bounding_box(&self) -> Option<StructureBoundingBoxModel> {
        self.pieces
            .iter()
            .map(|piece| piece.bounding_box)
            .reduce(StructureBoundingBoxModel::union)
    }

    pub fn create_tag(&self, chunk_pos: ChunkPos) -> StructureStartTagModel {
        if self.is_valid() {
            StructureStartTagModel {
                id: self.structure.unwrap_or("minecraft:unknown"),
                chunk_x: Some(chunk_pos.x),
                chunk_z: Some(chunk_pos.z),
                references: Some(self.references),
                children: self.pieces.len(),
            }
        } else {
            StructureStartTagModel {
                id: "INVALID",
                chunk_x: None,
                chunk_z: None,
                references: None,
                children: 0,
            }
        }
    }
}

impl StructureAccessModel {
    pub fn get_start_for_structure(&self, structure: &'static str) -> Option<&StructureStartModel> {
        self.starts.get(structure)
    }

    pub fn set_start_for_structure(&mut self, structure: &'static str, start: StructureStartModel) {
        self.starts.insert(structure, start);
        self.unsaved = true;
    }

    pub fn set_all_starts(&mut self, starts: BTreeMap<&'static str, StructureStartModel>) {
        self.starts = starts;
        self.unsaved = true;
    }

    pub fn get_references_for_structure(&self, structure: &'static str) -> &[i64] {
        self.references
            .get(structure)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn add_reference_for_structure(&mut self, structure: &'static str, reference: i64) {
        let references = self.references.entry(structure).or_default();
        if !references.contains(&reference) {
            references.push(reference);
        }
        self.unsaved = true;
    }

    pub fn set_all_references(&mut self, references: BTreeMap<&'static str, Vec<i64>>) {
        self.references = references;
        self.unsaved = true;
    }

    pub fn has_any_structure_references(&self) -> bool {
        self.references
            .values()
            .any(|references| !references.is_empty())
    }
}

pub fn structure_start_reference_pos(first_piece: StructurePieceModel) -> BlockPos {
    let center = first_piece.bounding_box.center();
    BlockPos {
        x: center.x,
        y: first_piece.bounding_box.min_y,
        z: center.z,
    }
}

pub fn structure_pieces_intersecting_chunk(
    start: &StructureStartModel,
    chunk_bb: StructureBoundingBoxModel,
) -> Vec<StructurePieceModel> {
    start
        .pieces
        .iter()
        .copied()
        .filter(|piece| piece.bounding_box.intersects(chunk_bb))
        .collect()
}

pub fn structure_has_piece_at(pos: BlockPos, start: &StructureStartModel) -> bool {
    start
        .pieces
        .iter()
        .any(|piece| piece.bounding_box.is_inside(pos))
}

pub fn structure_start_contains_pos(pos: BlockPos, start: &StructureStartModel) -> bool {
    start
        .bounding_box()
        .is_some_and(|bounding_box| bounding_box.is_inside(pos))
}

pub fn first_structure_start_containing_pos(
    pos: BlockPos,
    starts: &[StructureStartModel],
) -> Option<StructureStartModel> {
    starts
        .iter()
        .find(|start| structure_start_contains_pos(pos, start))
        .cloned()
}

pub fn first_structure_start_with_piece_at(
    pos: BlockPos,
    starts: &[StructureStartModel],
) -> Option<StructureStartModel> {
    starts
        .iter()
        .find(|start| structure_has_piece_at(pos, start))
        .cloned()
}

pub fn structure_check_result_from_cached_references(
    references: Option<i32>,
    require_unreferenced: bool,
) -> StructureCheckResultModel {
    match references {
        None => StructureCheckResultModel::StartNotPresent,
        Some(reference_count) if require_unreferenced && reference_count != 0 => {
            StructureCheckResultModel::StartNotPresent
        }
        Some(_) => StructureCheckResultModel::StartPresent,
    }
}

pub fn structure_fast_check_allows_lookup(result: StructureCheckResultModel) -> bool {
    result != StructureCheckResultModel::StartNotPresent
}

pub fn structure_locate_can_return_fast(
    result: StructureCheckResultModel,
    create_reference: bool,
) -> bool {
    !create_reference && result == StructureCheckResultModel::StartPresent
}

pub fn structure_start_can_satisfy_lookup(
    start: &StructureStartModel,
    create_reference: bool,
) -> bool {
    start.is_valid() && (!create_reference || start.can_be_referenced())
}

pub fn structure_try_add_reference(start: &mut StructureStartModel) -> bool {
    if start.can_be_referenced() {
        start.add_reference();
        true
    } else {
        false
    }
}

pub fn structure_access_valid_starts_for_references(
    access_by_chunk: &BTreeMap<i64, StructureAccessModel>,
    structure: &'static str,
    references: &[i64],
) -> Vec<StructureStartModel> {
    references
        .iter()
        .filter_map(|reference| access_by_chunk.get(reference))
        .filter_map(|access| access.get_start_for_structure(structure))
        .filter(|start| start.is_valid())
        .cloned()
        .collect()
}

const fn feature_type(
    id: &'static str,
    configuration: FeatureConfigurationKind,
    family: FeatureFamily,
) -> FeatureType {
    FeatureType {
        id,
        configuration,
        family,
    }
}

const fn cf(id: &'static str, source: ConfiguredFeatureSource) -> ConfiguredFeatureEntry {
    ConfiguredFeatureEntry { id, source }
}

const fn placed_source(
    source: PlacedFeatureSource,
    keys: &'static [&'static str],
) -> PlacedFeatureSourceEntry {
    PlacedFeatureSourceEntry { source, keys }
}

impl NoiseSettings {
    pub const fn new(min_y: i32, height: i32, size_horizontal: i32, size_vertical: i32) -> Self {
        Self {
            min_y,
            height,
            size_horizontal,
            size_vertical,
        }
    }

    pub fn validate(self) -> Result<(), String> {
        if self.min_y + self.height > 2032 {
            return Err("min_y + height cannot be higher than: 2032".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height has to be a multiple of 16".to_string());
        }
        if self.min_y % 16 != 0 {
            return Err("min_y has to be a multiple of 16".to_string());
        }
        if !(1..=4).contains(&self.size_horizontal) {
            return Err("size_horizontal must be in 1..=4".to_string());
        }
        if !(1..=4).contains(&self.size_vertical) {
            return Err("size_vertical must be in 1..=4".to_string());
        }
        Ok(())
    }

    pub fn cell_height(self) -> i32 {
        self.size_vertical * 4
    }

    pub fn cell_width(self) -> i32 {
        self.size_horizontal * 4
    }

    pub fn clamp_to_height(self, min_y: i32, max_y: i32) -> Self {
        let new_min_y = self.min_y.max(min_y);
        let new_height = (self.min_y + self.height).min(max_y + 1) - new_min_y;
        Self::new(
            new_min_y,
            new_height,
            self.size_horizontal,
            self.size_vertical,
        )
    }
}

impl VerticalAnchor {
    pub fn resolve_y(self, context: WorldGenerationHeightContext) -> i32 {
        match self {
            VerticalAnchor::Absolute(y) => y,
            VerticalAnchor::AboveBottom(offset) => context.min_y + offset,
            VerticalAnchor::BelowTop(offset) => context.min_y + context.height - 1 - offset,
        }
    }
}

pub fn height_provider_type(id: &str) -> Option<&'static HeightProviderType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    HEIGHT_PROVIDER_TYPES.iter().find(|provider_type| {
        provider_type
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(provider_type.id)
            == name
    })
}

pub fn height_provider_sample_bounds(
    provider: HeightProvider,
    context: WorldGenerationHeightContext,
) -> (i32, i32) {
    match provider {
        HeightProvider::Constant { value } => {
            let y = value.resolve_y(context);
            (y, y)
        }
        HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        }
        | HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            ..
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                (min, min)
            } else {
                (min, max)
            }
        }
        HeightProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        }
        | HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if max - min - inner + 1 <= 0 {
                (min, min)
            } else {
                (min, max - 1)
            }
        }
        HeightProvider::WeightedList { distribution } => distribution
            .iter()
            .map(|entry| height_provider_sample_bounds(entry.provider, context))
            .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
            .unwrap_or((context.min_y, context.min_y)),
    }
}

pub fn height_provider_sample_with_rolls(
    provider: HeightProvider,
    context: WorldGenerationHeightContext,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> i32 {
    match provider {
        HeightProvider::Constant { value } => value.resolve_y(context),
        HeightProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                min
            } else {
                min + first_roll.rem_euclid(max - min + 1)
            }
        }
        HeightProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            let outer_bound = max - min - inner + 1;
            if outer_bound <= 0 {
                min
            } else {
                let limit = first_roll.rem_euclid(outer_bound);
                min + second_roll.rem_euclid(limit + inner)
            }
        }
        HeightProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
            inner,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if max - min - inner + 1 <= 0 {
                min
            } else {
                let upper = min + inner + first_roll.rem_euclid(max - (min + inner) + 1);
                let biased_upper = min + second_roll.rem_euclid(upper - min);
                min + third_roll.rem_euclid(biased_upper - min + inner)
            }
        }
        HeightProvider::Trapezoid {
            min_inclusive,
            max_inclusive,
            plateau,
        } => {
            let min = min_inclusive.resolve_y(context);
            let max = max_inclusive.resolve_y(context);
            if min > max {
                return min;
            }

            let range = max - min;
            if plateau >= range {
                return min + first_roll.rem_euclid(range + 1);
            }

            let plateau_start = (range - plateau) / 2;
            let plateau_end = range - plateau_start;
            min + first_roll.rem_euclid(plateau_end + 1) + second_roll.rem_euclid(plateau_start + 1)
        }
        HeightProvider::WeightedList { distribution } => {
            let positive_weight_total = distribution
                .iter()
                .map(|entry| entry.weight.max(0))
                .sum::<i32>();
            if positive_weight_total <= 0 {
                return context.min_y;
            }

            let mut choice = first_roll.rem_euclid(positive_weight_total);
            let selected = distribution
                .iter()
                .find(|entry| {
                    let weight = entry.weight.max(0);
                    if choice < weight {
                        true
                    } else {
                        choice -= weight;
                        false
                    }
                })
                .expect("positive total weight must select a provider");
            height_provider_sample_with_rolls(
                selected.provider,
                context,
                second_roll,
                third_roll,
                0,
            )
        }
    }
}

pub fn block_predicate_type(id: &str) -> Option<&'static BlockPredicateType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BLOCK_PREDICATE_TYPES.iter().find(|predicate_type| {
        predicate_type
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(predicate_type.id)
            == name
    })
}

pub fn block_predicate_test(
    predicate: BlockPredicate,
    context: BlockPredicateContext,
    origin_y: i32,
) -> bool {
    match predicate {
        BlockPredicate::MatchingBlocks { blocks } => blocks.contains(&context.block),
        BlockPredicate::MatchingFluids { fluids } => fluids.contains(&context.fluid),
        BlockPredicate::Solid => context.solid,
        BlockPredicate::Replaceable => context.replaceable,
        BlockPredicate::InsideWorldBounds { offset_y } => {
            let y = origin_y + offset_y;
            y >= context.min_y && y < context.min_y + context.height
        }
        BlockPredicate::AnyOf { predicates } => predicates
            .iter()
            .any(|predicate| block_predicate_test(*predicate, context, origin_y)),
        BlockPredicate::AllOf { predicates } => predicates
            .iter()
            .all(|predicate| block_predicate_test(*predicate, context, origin_y)),
        BlockPredicate::Not { predicate } => !block_predicate_test(*predicate, context, origin_y),
        BlockPredicate::True => true,
        BlockPredicate::Unobstructed => context.unobstructed,
    }
}

pub fn placement_modifier_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:placement_modifier_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:").unwrap_or(entry) == name)
}

pub fn placement_modifier_positions(
    modifier: PlacementModifier,
    origin: BlockPos,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> Vec<BlockPos> {
    match modifier {
        PlacementModifier::RarityFilter { chance } => {
            if chance > 0 && first_roll.rem_euclid(chance) == 0 {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::Count { count } => vec![origin; count.max(0) as usize],
        PlacementModifier::InSquare => vec![BlockPos {
            x: origin.x + first_roll.rem_euclid(16),
            y: origin.y,
            z: origin.z + second_roll.rem_euclid(16),
        }],
        PlacementModifier::RandomOffset {
            xz_spread,
            y_spread,
        } => {
            let xz_bound = xz_spread.abs() * 2 + 1;
            let y_bound = y_spread.abs() * 2 + 1;
            vec![BlockPos {
                x: origin.x + first_roll.rem_euclid(xz_bound) - xz_spread.abs(),
                y: origin.y + second_roll.rem_euclid(y_bound) - y_spread.abs(),
                z: origin.z + third_roll.rem_euclid(xz_bound) - xz_spread.abs(),
            }]
        }
        PlacementModifier::Fixed { positions } => {
            let chunk_x = origin.x.div_euclid(16);
            let chunk_z = origin.z.div_euclid(16);
            positions
                .iter()
                .copied()
                .filter(|pos| pos.x.div_euclid(16) == chunk_x && pos.z.div_euclid(16) == chunk_z)
                .collect()
        }
        PlacementModifier::BiomeFilter
        | PlacementModifier::BlockPredicateFilter { .. }
        | PlacementModifier::SurfaceWaterDepthFilter { .. }
        | PlacementModifier::SurfaceRelativeThresholdFilter { .. }
        | PlacementModifier::Heightmap { .. } => Vec::new(),
    }
}

pub fn placed_feature_positions(
    modifiers: &[PlacementModifier],
    origin: BlockPos,
    context: PlacementContextModel,
    rolls: &[(i32, i32, i32)],
) -> Vec<BlockPos> {
    modifiers
        .iter()
        .enumerate()
        .fold(vec![origin], |positions, (index, modifier)| {
            let (first_roll, second_roll, third_roll) =
                rolls.get(index).copied().unwrap_or((0, 0, 0));
            positions
                .into_iter()
                .flat_map(|position| {
                    placement_modifier_positions_with_context(
                        *modifier,
                        position,
                        context,
                        first_roll,
                        second_roll,
                        third_roll,
                    )
                })
                .collect()
        })
}

pub fn placement_modifier_positions_with_context(
    modifier: PlacementModifier,
    origin: BlockPos,
    context: PlacementContextModel,
    first_roll: i32,
    second_roll: i32,
    third_roll: i32,
) -> Vec<BlockPos> {
    match modifier {
        PlacementModifier::BiomeFilter => {
            if context.biome_allows_feature {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::BlockPredicateFilter { predicate } => {
            if block_predicate_test(predicate, context.block_predicate, origin.y) {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::SurfaceWaterDepthFilter { max_water_depth } => {
            if context.world_surface_height - context.ocean_floor_height <= max_water_depth {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::SurfaceRelativeThresholdFilter {
            heightmap,
            min_inclusive,
            max_inclusive,
        } => {
            let surface = placement_context_height(context, heightmap);
            let min_y = surface + min_inclusive;
            let max_y = surface + max_inclusive;
            if (min_y..=max_y).contains(&origin.y) {
                vec![origin]
            } else {
                Vec::new()
            }
        }
        PlacementModifier::Heightmap { heightmap } => {
            let height = placement_context_height(context, heightmap);
            if height > context.min_y {
                vec![BlockPos {
                    x: origin.x,
                    y: height,
                    z: origin.z,
                }]
            } else {
                Vec::new()
            }
        }
        _ => placement_modifier_positions(modifier, origin, first_roll, second_roll, third_roll),
    }
}

fn placement_context_height(context: PlacementContextModel, heightmap: HeightmapKind) -> i32 {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => context.world_surface_height,
        HeightmapKind::OceanFloor
        | HeightmapKind::OceanFloorWg
        | HeightmapKind::MotionBlocking
        | HeightmapKind::MotionBlockingNoLeaves => context.ocean_floor_height,
    }
}

pub fn flat_generator_preset(id: &str) -> Option<&'static FlatGeneratorPreset> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    FLAT_GENERATOR_PRESETS
        .iter()
        .find(|preset| preset.id.strip_prefix("minecraft:").unwrap_or(preset.id) == name)
}

pub fn flat_layers_total_height(layers: &[FlatLayerInfo]) -> i32 {
    layers.iter().map(|layer| layer.height).sum()
}

pub fn validate_flat_layers(layers: &[FlatLayerInfo]) -> Result<(), String> {
    let total_height = flat_layers_total_height(layers);
    if total_height > 384 {
        Err("Sum of layer heights is > 384".to_string())
    } else {
        Ok(())
    }
}

pub fn flat_block_at_y(layers: &[FlatLayerInfo], y: i32) -> Option<&'static str> {
    if y < 0 {
        return None;
    }
    let mut cursor = 0;
    for layer in layers {
        let next = cursor + layer.height;
        if y < next {
            return Some(layer.block);
        }
        cursor = next;
    }
    None
}

pub fn flat_layers_are_void(layers: &[FlatLayerInfo]) -> bool {
    layers.iter().all(|layer| layer.block == "minecraft:air")
}

pub fn world_preset(id: &str) -> Option<&'static WorldPresetEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLD_PRESETS
        .iter()
        .find(|preset| preset.id.strip_prefix("minecraft:").unwrap_or(preset.id) == name)
}

pub fn chunk_generator_kind(id: &str) -> Option<ChunkGeneratorKind> {
    match id.strip_prefix("minecraft:").unwrap_or(id) {
        "noise" => Some(ChunkGeneratorKind::Noise),
        "flat" => Some(ChunkGeneratorKind::Flat),
        "debug" => Some(ChunkGeneratorKind::Debug),
        _ => None,
    }
}

pub fn resolve_level_stem(stem: &LevelStemPreset) -> Result<ResolvedLevelStem, String> {
    let generator = match chunk_generator_kind(stem.generator)
        .ok_or_else(|| format!("Unknown chunk generator {}", stem.generator))?
    {
        ChunkGeneratorKind::Noise => {
            let settings_id = stem
                .noise_settings
                .ok_or_else(|| format!("Noise generator {} has no settings", stem.dimension))?;
            let settings = builtin_noise_generator_settings(settings_id)
                .ok_or_else(|| format!("Unknown noise settings {settings_id}"))?;
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Noise {
                biome_source: stem.biome_source,
                biome_source_model,
                noise_settings: settings,
            }
        }
        ChunkGeneratorKind::Flat => {
            if stem.noise_settings.is_some() {
                return Err(format!(
                    "Flat generator {} must not carry noise settings",
                    stem.dimension
                ));
            }
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Flat {
                biome_source_model,
                settings: default_flat_generator_settings()?,
            }
        }
        ChunkGeneratorKind::Debug => {
            if stem.noise_settings.is_some() {
                return Err(format!(
                    "Debug generator {} must not carry noise settings",
                    stem.dimension
                ));
            }
            let biome_source_model = biome_source_from_stem_id(stem.biome_source)
                .ok_or_else(|| format!("Unknown biome source {}", stem.biome_source))?;
            ResolvedChunkGenerator::Debug {
                biome: stem.biome_source,
                biome_source_model,
            }
        }
    };
    Ok(ResolvedLevelStem {
        dimension: stem.dimension,
        generator,
    })
}

pub fn resolve_world_preset(id: &str) -> Result<ResolvedWorldPreset, String> {
    let preset = world_preset(id).ok_or_else(|| format!("Unknown world preset {id}"))?;
    Ok(ResolvedWorldPreset {
        id: preset.id,
        overworld: resolve_level_stem(&preset.overworld)?,
        nether: resolve_level_stem(&preset.nether)?,
        end: resolve_level_stem(&preset.end)?,
    })
}

pub fn generate_chunk_for_stem(
    pos: ChunkPos,
    stem: &ResolvedLevelStem,
) -> Result<LevelChunk, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(materialize_flat_chunk(pos, settings)),
        ResolvedChunkGenerator::Noise {
            biome_source_model,
            noise_settings,
            ..
        } => Ok(materialize_noise_preview_chunk(
            pos,
            biome_source_model,
            noise_settings,
        )),
        ResolvedChunkGenerator::Debug { .. } => Err(format!(
            "Debug chunk generation for {} is not implemented",
            stem.dimension
        )),
    }
}

pub fn generator_base_height_for_stem(
    x: i32,
    z: i32,
    heightmap: HeightmapKind,
    stem: &ResolvedLevelStem,
) -> Result<i32, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(flat_base_height(
            &settings.expanded_layers,
            FLAT_GENERATOR_MIN_Y,
            FLAT_GENERATOR_GEN_DEPTH,
            heightmap,
        )),
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            Ok(noise_preview_base_height(x, z, noise_settings, heightmap))
        }
        ResolvedChunkGenerator::Debug { .. } => Err(format!(
            "Debug base-height query for {} is not implemented",
            stem.dimension
        )),
    }
}

pub fn generator_base_column_for_stem(
    x: i32,
    z: i32,
    stem: &ResolvedLevelStem,
) -> Result<FlatNoiseColumn, String> {
    match &stem.generator {
        ResolvedChunkGenerator::Flat { settings, .. } => Ok(flat_base_column(
            &settings.expanded_layers,
            FLAT_GENERATOR_MIN_Y,
            FLAT_GENERATOR_GEN_DEPTH,
        )),
        ResolvedChunkGenerator::Noise { noise_settings, .. } => {
            Ok(noise_preview_base_column(x, z, noise_settings))
        }
        ResolvedChunkGenerator::Debug { .. } => Err(format!(
            "Debug base-column query for {} is not implemented",
            stem.dimension
        )),
    }
}

pub fn generate_overworld_chunk_for_preset(
    pos: ChunkPos,
    preset_id: &str,
) -> Result<LevelChunk, String> {
    let preset = resolve_world_preset(preset_id)?;
    generate_chunk_for_stem(pos, &preset.overworld)
}

pub fn world_preset_from_overworld_generator(generator: &str) -> Option<&'static str> {
    match generator {
        "minecraft:flat" | "flat" => Some("minecraft:flat"),
        "minecraft:debug" | "debug" => Some("minecraft:debug_all_block_states"),
        "minecraft:noise" | "noise" => Some("minecraft:normal"),
        _ => None,
    }
}

pub fn world_preset_dimensions_in_order(preset: &WorldPresetEntry) -> [&'static str; 3] {
    [
        preset.overworld.dimension,
        preset.nether.dimension,
        preset.end.dimension,
    ]
}

pub fn validate_world_preset_dimensions(dimensions: &[&str]) -> Result<(), String> {
    if dimensions.contains(&"minecraft:overworld") {
        Ok(())
    } else {
        Err("Missing overworld dimension".to_string())
    }
}

pub fn builtin_noise_generator_settings(id: &str) -> Option<&'static NoiseGeneratorSettings> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_GENERATOR_SETTINGS.iter().find(|settings| {
        settings
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|settings_name| settings_name == name)
    })
}

impl DensityFunction {
    pub fn compute(self, block_y: i32) -> f64 {
        match self {
            DensityFunction::Reference(id) => builtin_density_function(id)
                .map(|entry| entry.function.compute(block_y))
                .unwrap_or(0.0),
            DensityFunction::Constant(value) => value,
            DensityFunction::YClampedGradient {
                from_y,
                to_y,
                from_value,
                to_value,
            } => {
                if block_y <= from_y {
                    from_value
                } else if block_y >= to_y {
                    to_value
                } else {
                    let progress = f64::from(block_y - from_y) / f64::from(to_y - from_y);
                    from_value + progress * (to_value - from_value)
                }
            }
            DensityFunction::Clamp { input, min, max } => input.compute(block_y).clamp(min, max),
            DensityFunction::Mapped { kind, input } => kind.transform(input.compute(block_y)),
            DensityFunction::Binary {
                kind,
                argument1,
                argument2,
            } => kind.apply(argument1.compute(block_y), argument2.compute(block_y)),
            DensityFunction::Marker { input, .. } | DensityFunction::BlendDensity { input } => {
                input.compute(block_y)
            }
            DensityFunction::BlendAlpha => 1.0,
            DensityFunction::BlendOffset => 0.0,
            DensityFunction::Noise { .. }
            | DensityFunction::ShiftedNoise { .. }
            | DensityFunction::BlendedNoise { .. }
            | DensityFunction::EndIslands { .. }
            | DensityFunction::WeirdScaledSampler { .. }
            | DensityFunction::Beardifier
            | DensityFunction::Spline
            | DensityFunction::FindTopSurface => 0.0,
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            DensityFunction::Reference(_) => "reference",
            DensityFunction::Constant(_) => "constant",
            DensityFunction::YClampedGradient { .. } => "y_clamped_gradient",
            DensityFunction::Clamp { .. } => "clamp",
            DensityFunction::Mapped { kind, .. } => kind.serialized_name(),
            DensityFunction::Binary { kind, .. } => kind.serialized_name(),
            DensityFunction::Marker { kind, .. } => kind.serialized_name(),
            DensityFunction::Noise { .. } => "noise",
            DensityFunction::ShiftedNoise { .. } => "shifted_noise",
            DensityFunction::BlendedNoise { .. } => "old_blended_noise",
            DensityFunction::EndIslands { .. } => "end_islands",
            DensityFunction::WeirdScaledSampler { .. } => "weird_scaled_sampler",
            DensityFunction::BlendAlpha => "blend_alpha",
            DensityFunction::BlendOffset => "blend_offset",
            DensityFunction::BlendDensity { .. } => "blend_density",
            DensityFunction::Beardifier => "beardifier",
            DensityFunction::Spline => "spline",
            DensityFunction::FindTopSurface => "find_top_surface",
        }
    }
}

impl NoiseRouter {
    pub const fn simple(final_density: DensityFunction) -> Self {
        Self {
            barrier: ZERO_DENSITY,
            fluid_level_floodedness: ZERO_DENSITY,
            fluid_level_spread: ZERO_DENSITY,
            lava: ZERO_DENSITY,
            temperature: ZERO_DENSITY,
            vegetation: ZERO_DENSITY,
            continents: ZERO_DENSITY,
            erosion: ZERO_DENSITY,
            depth: ZERO_DENSITY,
            ridges: ZERO_DENSITY,
            preliminary_surface_level: ZERO_DENSITY,
            final_density,
            vein_toggle: ZERO_DENSITY,
            vein_ridged: ZERO_DENSITY,
            vein_gap: ZERO_DENSITY,
        }
    }

    pub fn field_type_names(self) -> [&'static str; 15] {
        [
            self.barrier.type_name(),
            self.fluid_level_floodedness.type_name(),
            self.fluid_level_spread.type_name(),
            self.lava.type_name(),
            self.temperature.type_name(),
            self.vegetation.type_name(),
            self.continents.type_name(),
            self.erosion.type_name(),
            self.depth.type_name(),
            self.ridges.type_name(),
            self.preliminary_surface_level.type_name(),
            self.final_density.type_name(),
            self.vein_toggle.type_name(),
            self.vein_ridged.type_name(),
            self.vein_gap.type_name(),
        ]
    }
}

impl MappedDensityFunction {
    pub fn serialized_name(self) -> &'static str {
        match self {
            MappedDensityFunction::Abs => "abs",
            MappedDensityFunction::Square => "square",
            MappedDensityFunction::Cube => "cube",
            MappedDensityFunction::HalfNegative => "half_negative",
            MappedDensityFunction::QuarterNegative => "quarter_negative",
            MappedDensityFunction::Invert => "invert",
            MappedDensityFunction::Squeeze => "squeeze",
        }
    }

    pub fn transform(self, input: f64) -> f64 {
        match self {
            MappedDensityFunction::Abs => input.abs(),
            MappedDensityFunction::Square => input * input,
            MappedDensityFunction::Cube => input * input * input,
            MappedDensityFunction::HalfNegative => {
                if input > 0.0 {
                    input
                } else {
                    input * 0.5
                }
            }
            MappedDensityFunction::QuarterNegative => {
                if input > 0.0 {
                    input
                } else {
                    input * 0.25
                }
            }
            MappedDensityFunction::Invert => -input,
            MappedDensityFunction::Squeeze => {
                let clamped = input.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
        }
    }
}

impl BinaryDensityFunction {
    pub fn serialized_name(self) -> &'static str {
        match self {
            BinaryDensityFunction::Add => "add",
            BinaryDensityFunction::Mul => "mul",
            BinaryDensityFunction::Min => "min",
            BinaryDensityFunction::Max => "max",
        }
    }

    pub fn apply(self, first: f64, second: f64) -> f64 {
        match self {
            BinaryDensityFunction::Add => first + second,
            BinaryDensityFunction::Mul => first * second,
            BinaryDensityFunction::Min => first.min(second),
            BinaryDensityFunction::Max => first.max(second),
        }
    }
}

impl DensityMarker {
    pub fn serialized_name(self) -> &'static str {
        match self {
            DensityMarker::Interpolated => "interpolated",
            DensityMarker::FlatCache => "flat_cache",
            DensityMarker::Cache2D => "cache_2d",
            DensityMarker::CacheOnce => "cache_once",
            DensityMarker::CacheAllInCell => "cache_all_in_cell",
        }
    }
}

impl RarityValueMapper {
    pub fn serialized_name(self) -> &'static str {
        match self {
            RarityValueMapper::Type1 => "type_1",
            RarityValueMapper::Type2 => "type_2",
        }
    }
}

impl FluidStatus {
    pub fn at(self, block_y: i32) -> &'static str {
        if block_y < self.fluid_level {
            self.fluid_type
        } else {
            "minecraft:air"
        }
    }
}

pub fn disabled_aquifer_substance(
    density: f64,
    fluid: FluidStatus,
    block_y: i32,
) -> Option<&'static str> {
    if density > 0.0 {
        None
    } else {
        Some(fluid.at(block_y))
    }
}

pub fn aquifer_similarity(distance_sqr_1: i32, distance_sqr_2: i32) -> f64 {
    1.0 - f64::from((distance_sqr_2 - distance_sqr_1).abs()) / 25.0
}

pub fn builtin_normal_noise_parameters(id: &str) -> Option<&'static NormalNoiseParameters> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    NORMAL_NOISE_PARAMETERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn synth_noise_source(id: &str) -> Option<&'static SynthNoiseSource> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    SYNTH_NOISE_SOURCES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn normal_noise_expected_deviation(octave_span: i32) -> f64 {
    0.1 * (1.0 + 1.0 / (f64::from(octave_span) + 1.0))
}

pub fn normal_noise_value_factor(parameters: NormalNoiseParameters) -> f64 {
    let mut min_octave = i32::MAX;
    let mut max_octave = i32::MIN;
    for (index, amplitude) in parameters.amplitudes.iter().enumerate() {
        if *amplitude != 0.0 {
            min_octave = min_octave.min(index as i32);
            max_octave = max_octave.max(index as i32);
        }
    }
    NORMAL_NOISE_TARGET_DEVIATION / 2.0 / normal_noise_expected_deviation(max_octave - min_octave)
}

pub fn builtin_density_function(id: &str) -> Option<&'static DensityFunctionEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_DENSITY_FUNCTIONS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn density_function_type(id: &str) -> Option<&'static DensityFunctionType> {
    DENSITY_FUNCTION_TYPES.iter().find(|kind| kind.id == id)
}

pub fn builtin_noise_router(id: &str) -> Option<&'static NoiseRouterEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_ROUTERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn builtin_surface_rule_preset(id: &str) -> Option<&'static SurfaceRulePresetData> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_SURFACE_RULE_PRESETS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn surface_rule_type(id: &str) -> Option<&'static SurfaceRuleType> {
    SURFACE_RULE_TYPES.iter().find(|entry| entry.id == id)
}

pub fn surface_condition_type(id: &str) -> Option<&'static SurfaceConditionType> {
    SURFACE_CONDITION_TYPES.iter().find(|entry| entry.id == id)
}

pub fn surface_condition_test(
    condition: &SurfaceConditionSource,
    context: &SurfaceMaterialContext,
    height_context: &WorldGenerationHeightContext,
) -> bool {
    match condition {
        SurfaceConditionSource::Biome(targets) => targets.contains(&context.biome),
        SurfaceConditionSource::NoiseThreshold { min, max } => {
            context.noise >= *min && context.noise <= *max
        }
        SurfaceConditionSource::VerticalGradient {
            true_at_and_below,
            false_at_and_above,
        } => {
            let true_y = true_at_and_below.resolve_y(*height_context);
            let false_y = false_at_and_above.resolve_y(*height_context);
            context.y <= true_y || (context.y < false_y && context.noise < 0.0)
        }
        SurfaceConditionSource::YAbove {
            anchor,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let mut threshold = anchor.resolve_y(*height_context)
                + context.surface_depth * *surface_depth_multiplier;
            if *add_stone_depth {
                threshold += context.stone_depth_above;
            }
            context.y >= threshold
        }
        SurfaceConditionSource::Water {
            offset,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let mut threshold =
                context.water_height + *offset + context.surface_depth * *surface_depth_multiplier;
            if *add_stone_depth {
                threshold += context.stone_depth_above;
            }
            context.y <= threshold
        }
        SurfaceConditionSource::StoneDepth {
            offset,
            add_surface_depth,
            secondary_depth_range,
            surface,
        } => {
            let mut threshold = *offset;
            if *add_surface_depth {
                threshold += context.surface_depth;
            }
            let depth = match surface {
                CaveSurface::Floor => context.stone_depth_above,
                CaveSurface::Ceiling => context.stone_depth_below,
            };
            depth <= threshold + *secondary_depth_range
        }
        SurfaceConditionSource::Not(target) => {
            !surface_condition_test(target, context, height_context)
        }
        SurfaceConditionSource::Steep => context.steep,
        SurfaceConditionSource::Hole => context.hole,
        SurfaceConditionSource::AbovePreliminarySurface => {
            context.y >= context.preliminary_surface_y
        }
        SurfaceConditionSource::Temperature => context.temperature < 0.15,
    }
}

pub fn surface_rule_apply(
    rule: &SurfaceRuleSource,
    context: &SurfaceMaterialContext,
    height_context: &WorldGenerationHeightContext,
) -> Option<&'static str> {
    match rule {
        SurfaceRuleSource::Bandlands => Some(match (context.x + context.z).rem_euclid(5) {
            0 => "minecraft:white_terracotta",
            1 => "minecraft:orange_terracotta",
            2 => "minecraft:terracotta",
            3 => "minecraft:red_sand",
            _ => "minecraft:red_sandstone",
        }),
        SurfaceRuleSource::Block(block) => Some(block),
        SurfaceRuleSource::Sequence(rules) => rules
            .iter()
            .find_map(|rule| surface_rule_apply(rule, context, height_context)),
        SurfaceRuleSource::Condition { condition, rule } => {
            surface_condition_test(condition, context, height_context)
                .then(|| surface_rule_apply(rule, context, height_context))
                .flatten()
        }
    }
}

pub fn cave_generation_family(id: &str) -> Option<&'static CaveGenerationFamily> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CAVE_GENERATION_FAMILIES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn ore_vein_type_for_toggle(vein_toggle: f64) -> OreVeinType {
    if vein_toggle > 0.0 {
        ORE_VEIN_TYPES[0]
    } else {
        ORE_VEIN_TYPES[1]
    }
}

pub fn ore_vein_richness(veininess_ridged: f64) -> f64 {
    clamped_map(
        veininess_ridged,
        ORE_VEINIFIER_CONSTANTS.veininess_threshold,
        ORE_VEINIFIER_CONSTANTS.max_richness_threshold,
        ORE_VEINIFIER_CONSTANTS.min_richness,
        ORE_VEINIFIER_CONSTANTS.max_richness,
    )
}

pub fn ore_vein_decision(input: OreVeinDecisionInput) -> Option<&'static str> {
    let default_state = input.debug_ore_veins.then_some("minecraft:air");
    let vein_type = ore_vein_type_for_toggle(input.vein_toggle);
    let veininess_ridged = input.vein_toggle.abs();
    let distance_from_top = vein_type.max_y - input.y;
    let distance_from_bottom = input.y - vein_type.min_y;
    if distance_from_bottom < 0 || distance_from_top < 0 {
        return default_state;
    }

    let distance_from_edge = distance_from_top.min(distance_from_bottom);
    let edge_roundoff = clamped_map(
        f64::from(distance_from_edge),
        0.0,
        f64::from(ORE_VEINIFIER_CONSTANTS.edge_roundoff_begin),
        -ORE_VEINIFIER_CONSTANTS.max_edge_roundoff,
        0.0,
    );
    if veininess_ridged + edge_roundoff < ORE_VEINIFIER_CONSTANTS.veininess_threshold {
        return default_state;
    }
    if input.solidness_random > ORE_VEINIFIER_CONSTANTS.vein_solidness {
        return default_state;
    }
    if input.vein_ridged >= 0.0 {
        return default_state;
    }

    let richness = ore_vein_richness(veininess_ridged);
    if input.richness_random < richness
        && input.vein_gap > ORE_VEINIFIER_CONSTANTS.skip_ore_if_gap_noise_is_below
    {
        if input.raw_ore_random < ORE_VEINIFIER_CONSTANTS.chance_of_raw_ore_block {
            Some(vein_type.raw_ore_block)
        } else {
            Some(vein_type.ore)
        }
    } else if input.debug_ore_veins {
        Some("minecraft:oak_button")
    } else {
        Some(vein_type.filler)
    }
}

fn clamped_map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let clamped = value.clamp(from_min, from_max);
    let progress = (clamped - from_min) / (from_max - from_min);
    to_min + progress * (to_max - to_min)
}

pub fn configured_carver(id: &str) -> Option<&'static ConfiguredCarver> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CONFIGURED_CARVERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn world_carver_type(id: &str) -> Option<WorldCarverType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLD_CARVER_TYPES
        .iter()
        .copied()
        .find(|carver| carver.id().strip_prefix("minecraft:") == Some(name))
}

pub fn carver_is_start_chunk(carver: &ConfiguredCarver, random_next_float: f32) -> bool {
    random_next_float <= carver.probability
}

pub fn feature_type_by_id(id: &str) -> Option<&'static FeatureType> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    FEATURE_TYPES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn configured_feature(id: &str) -> Option<&'static ConfiguredFeatureEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CONFIGURED_FEATURES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn placed_feature_source(id: &str) -> Option<PlacedFeatureSource> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| {
            entry.keys.iter().any(|key| {
                key.strip_prefix("minecraft:")
                    .is_some_and(|entry_name| entry_name == name)
            })
        })
        .map(|entry| entry.source)
}

pub fn biome_generation_settings(id: &str) -> Option<&'static BiomeGenerationSettingsModel> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_BIOME_GENERATION_SETTINGS.iter().find(|entry| {
        entry
            .biome
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn biome_has_placed_feature(biome: &BiomeGenerationSettingsModel, feature: &str) -> bool {
    let name = feature.strip_prefix("minecraft:").unwrap_or(feature);
    biome.feature_steps.iter().any(|step| {
        step.iter().any(|entry| {
            entry
                .strip_prefix("minecraft:")
                .is_some_and(|entry_name| entry_name == name)
        })
    })
}

pub fn biome_spawns_for_category(
    biome: &BiomeGenerationSettingsModel,
    category: &str,
) -> &'static [MobSpawnerDataModel] {
    biome
        .spawners
        .iter()
        .find(|group| group.category == category)
        .map(|group| group.entries)
        .unwrap_or(&[])
}

pub fn block_state_provider_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:block_state_provider_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn block_state_provider_sample(
    provider: &BlockStateProviderModel,
    random_roll: i32,
) -> Option<&'static str> {
    match provider {
        BlockStateProviderModel::Simple(state) => Some(*state),
        BlockStateProviderModel::Weighted(entries) => {
            let total_weight = entries.iter().try_fold(0_i32, |total, entry| {
                (entry.weight > 0).then_some(total + entry.weight)
            })?;
            let mut roll = random_roll.rem_euclid(total_weight);
            entries.iter().find_map(|entry| {
                roll -= entry.weight;
                (roll < 0).then_some(entry.state)
            })
        }
    }
}

pub fn simple_block_placement_plan(
    config: &SimpleBlockConfigurationModel,
    context: SimpleBlockPlacementContext,
    random_roll: i32,
) -> Option<SimpleBlockPlacementPlan> {
    let state = block_state_provider_sample(&config.to_place, random_roll)?;
    if !simple_block_can_survive(state, context) {
        return None;
    }
    if simple_block_is_double_plant(state) {
        if context.above_block != "minecraft:air" {
            return None;
        }
        return Some(SimpleBlockPlacementPlan {
            state,
            upper_state: Some(state),
            schedule_tick: config.schedule_tick,
        });
    }
    Some(SimpleBlockPlacementPlan {
        state,
        upper_state: None,
        schedule_tick: config.schedule_tick,
    })
}

pub fn simple_block_can_survive(state: &str, context: SimpleBlockPlacementContext) -> bool {
    if context.origin_block != "minecraft:air" {
        return false;
    }
    if simple_block_is_plant(state) {
        return matches!(
            context.below_block,
            "minecraft:grass_block"
                | "minecraft:dirt"
                | "minecraft:coarse_dirt"
                | "minecraft:podzol"
                | "minecraft:farmland"
                | "minecraft:moss_block"
        );
    }
    true
}

fn simple_block_is_plant(state: &str) -> bool {
    matches!(
        state,
        "minecraft:short_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:tall_grass"
            | "minecraft:dandelion"
            | "minecraft:poppy"
            | "minecraft:azure_bluet"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:sunflower"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:lilac"
    )
}

fn simple_block_is_double_plant(state: &str) -> bool {
    matches!(
        state,
        "minecraft:sunflower"
            | "minecraft:rose_bush"
            | "minecraft:peony"
            | "minecraft:lilac"
            | "minecraft:tall_grass"
            | "minecraft:large_fern"
    )
}

pub fn replace_block_result(
    current_block: &'static str,
    targets: &[TargetBlockStateModel],
) -> Option<&'static str> {
    targets
        .iter()
        .find(|target| rule_test_matches(target.target, current_block))
        .map(|target| target.state)
}

pub fn rule_test_matches(test: RuleTestModel, block: &str) -> bool {
    match test {
        RuleTestModel::AlwaysTrue => true,
        RuleTestModel::BlockMatch(expected) => block == expected,
        RuleTestModel::TagMatch(blocks) => blocks.contains(&block),
    }
}

pub fn ore_should_skip_air_check(discard_chance_on_air_exposure: f32, random_roll: f32) -> bool {
    if discard_chance_on_air_exposure <= 0.0 {
        true
    } else if discard_chance_on_air_exposure >= 1.0 {
        false
    } else {
        random_roll >= discard_chance_on_air_exposure
    }
}

pub fn ore_can_place(
    current_block: &'static str,
    adjacent_to_air: bool,
    config: &OreConfigurationModel,
    target: TargetBlockStateModel,
    random_roll: f32,
) -> bool {
    rule_test_matches(target.target, current_block)
        && (ore_should_skip_air_check(config.discard_chance_on_air_exposure, random_roll)
            || !adjacent_to_air)
}

pub fn scattered_ore_offset(
    origin: BlockPos,
    try_index: i32,
    axis_rolls: [(f32, f32); 3],
) -> BlockPos {
    let max_distance = try_index.min(7);
    let axis =
        |(first, second): (f32, f32)| ((first - second) * max_distance as f32).round() as i32;
    BlockPos {
        x: origin.x + axis(axis_rolls[0]),
        y: origin.y + axis(axis_rolls[1]),
        z: origin.z + axis(axis_rolls[2]),
    }
}

pub fn scattered_ore_attempt(
    origin: BlockPos,
    try_index: i32,
    axis_rolls: [(f32, f32); 3],
    current_block: &'static str,
    adjacent_to_air: bool,
    config: &OreConfigurationModel,
    air_check_roll: f32,
) -> Option<ScatteredOreAttempt> {
    let pos = scattered_ore_offset(origin, try_index, axis_rolls);
    config.target_states.iter().copied().find_map(|target| {
        ore_can_place(
            current_block,
            adjacent_to_air,
            config,
            target,
            air_check_roll,
        )
        .then_some(ScatteredOreAttempt {
            pos,
            state: target.state,
        })
    })
}

pub fn ore_vein_spheres(
    origin: BlockPos,
    size: i32,
    direction_roll: f32,
    y_rolls: &[(i32, i32)],
    radius_rolls: &[f64],
) -> Vec<OreVeinSphere> {
    let direction = direction_roll * std::f32::consts::PI;
    let spread_xy = size as f32 / 8.0;
    let x0 = f64::from(origin.x) + f64::from(direction.sin() * spread_xy);
    let x1 = f64::from(origin.x) - f64::from(direction.sin() * spread_xy);
    let z0 = f64::from(origin.z) + f64::from(direction.cos() * spread_xy);
    let z1 = f64::from(origin.z) - f64::from(direction.cos() * spread_xy);
    let default_y = (0, 0);
    let (y_roll_0, y_roll_1) = y_rolls.first().copied().unwrap_or(default_y);
    let y0 = f64::from(origin.y + y_roll_0 - 2);
    let y1 = f64::from(origin.y + y_roll_1 - 2);

    let mut spheres = Vec::new();
    for i in 0..size.max(0) {
        let step = i as f64 / size as f64;
        let radius_roll = radius_rolls.get(i as usize).copied().unwrap_or(0.0);
        let center_x = x0 + (x1 - x0) * step;
        let center_y = y0 + (y1 - y0) * step;
        let center_z = z0 + (z1 - z0) * step;
        let size_scale = radius_roll * f64::from(size) / 16.0;
        let radius = (((std::f64::consts::PI * step).sin() + 1.0) * size_scale + 1.0) / 2.0;
        spheres.push(OreVeinSphere {
            center_x,
            center_y,
            center_z,
            radius,
        });
    }

    for i in 0..spheres.len().saturating_sub(1) {
        if spheres[i].radius <= 0.0 {
            continue;
        }
        for j in i + 1..spheres.len() {
            if spheres[j].radius <= 0.0 {
                continue;
            }
            let dx = spheres[i].center_x - spheres[j].center_x;
            let dy = spheres[i].center_y - spheres[j].center_y;
            let dz = spheres[i].center_z - spheres[j].center_z;
            let dr = spheres[i].radius - spheres[j].radius;
            if ore_vein_sphere_is_shadowed(dr, dx, dy, dz) {
                if dr > 0.0 {
                    spheres[j].radius = -1.0;
                } else {
                    spheres[i].radius = -1.0;
                }
            }
        }
    }

    spheres
        .into_iter()
        .filter(|sphere| sphere.radius >= 0.0)
        .collect()
}

pub fn ore_vein_position_candidates(
    spheres: &[OreVeinSphere],
    x_start: i32,
    y_start: i32,
    z_start: i32,
    size_xz: i32,
    size_y: i32,
    build_height: std::ops::Range<i32>,
) -> Vec<BlockPos> {
    let mut positions = Vec::new();
    for sphere in spheres {
        let x_min = (sphere.center_x - sphere.radius)
            .floor()
            .max(f64::from(x_start)) as i32;
        let y_min = (sphere.center_y - sphere.radius)
            .floor()
            .max(f64::from(y_start)) as i32;
        let z_min = (sphere.center_z - sphere.radius)
            .floor()
            .max(f64::from(z_start)) as i32;
        let x_max = (sphere.center_x + sphere.radius)
            .floor()
            .max(f64::from(x_min)) as i32;
        let y_max = (sphere.center_y + sphere.radius)
            .floor()
            .max(f64::from(y_min)) as i32;
        let z_max = (sphere.center_z + sphere.radius)
            .floor()
            .max(f64::from(z_min)) as i32;
        for x in x_min..=x_max {
            let xd = (f64::from(x) + 0.5 - sphere.center_x) / sphere.radius;
            if xd * xd >= 1.0 {
                continue;
            }
            for y in y_min..=y_max {
                let yd = (f64::from(y) + 0.5 - sphere.center_y) / sphere.radius;
                if xd * xd + yd * yd >= 1.0 {
                    continue;
                }
                for z in z_min..=z_max {
                    let zd = (f64::from(z) + 0.5 - sphere.center_z) / sphere.radius;
                    if xd * xd + yd * yd + zd * zd >= 1.0 || !build_height.contains(&y) {
                        continue;
                    }
                    let bitset_index =
                        x - x_start + (y - y_start) * size_xz + (z - z_start) * size_xz * size_y;
                    if bitset_index >= 0 {
                        let pos = BlockPos { x, y, z };
                        if !positions.contains(&pos) {
                            positions.push(pos);
                        }
                    }
                }
            }
        }
    }
    positions
}

pub fn ore_placement_plan(
    config: &OreConfigurationModel,
    candidates: &[OrePlacementContext],
) -> Vec<OrePlacementBlock> {
    candidates
        .iter()
        .filter_map(|candidate| {
            config.target_states.iter().copied().find_map(|target| {
                ore_can_place(
                    candidate.current_block,
                    candidate.adjacent_to_air,
                    config,
                    target,
                    candidate.air_check_roll,
                )
                .then_some(OrePlacementBlock {
                    pos: candidate.pos,
                    state: target.state,
                })
            })
        })
        .collect()
}

pub fn aquatic_feature_offset(
    origin: BlockPos,
    x_rolls: (i32, i32),
    z_rolls: (i32, i32),
) -> (i32, i32) {
    (
        origin.x + x_rolls.0 - x_rolls.1,
        origin.z + z_rolls.0 - z_rolls.1,
    )
}

pub fn seagrass_placement_plan(
    pos: BlockPos,
    current_block: &'static str,
    above_block: &'static str,
    can_survive: bool,
    tall_probability: f64,
    tall_roll: f64,
) -> Vec<AquaticPlacementBlock> {
    if current_block != "minecraft:water" || !can_survive {
        return Vec::new();
    }
    if tall_roll < tall_probability {
        if above_block == "minecraft:water" {
            vec![
                AquaticPlacementBlock {
                    pos,
                    state: "minecraft:tall_seagrass",
                },
                AquaticPlacementBlock {
                    pos: BlockPos {
                        x: pos.x,
                        y: pos.y + 1,
                        z: pos.z,
                    },
                    state: "minecraft:tall_seagrass[half=upper]",
                },
            ]
        } else {
            Vec::new()
        }
    } else {
        vec![AquaticPlacementBlock {
            pos,
            state: "minecraft:seagrass",
        }]
    }
}

pub fn sea_pickle_placement_plan(
    pos: BlockPos,
    current_block: &'static str,
    can_survive: bool,
    pickle_roll: i32,
) -> Option<AquaticPlacementBlock> {
    if current_block == "minecraft:water" && can_survive {
        Some(AquaticPlacementBlock {
            pos,
            state: match pickle_roll.rem_euclid(4) + 1 {
                1 => "minecraft:sea_pickle[pickles=1]",
                2 => "minecraft:sea_pickle[pickles=2]",
                3 => "minecraft:sea_pickle[pickles=3]",
                _ => "minecraft:sea_pickle[pickles=4]",
            },
        })
    } else {
        None
    }
}

pub fn kelp_placement_plan(
    origin: BlockPos,
    water_column: &[bool],
    survival_column: &[bool],
    height_roll: i32,
    age_rolls: &[i32],
    below_is_kelp: bool,
) -> Vec<AquaticPlacementBlock> {
    if !water_column.first().copied().unwrap_or(false) {
        return Vec::new();
    }
    let height = 1 + height_roll.rem_euclid(10);
    let mut blocks = Vec::new();
    for h in 0..=height {
        let current_water = water_column.get(h as usize).copied().unwrap_or(false);
        let above_water = water_column.get(h as usize + 1).copied().unwrap_or(false);
        let can_survive = survival_column.get(h as usize).copied().unwrap_or(false);
        let pos = BlockPos {
            x: origin.x,
            y: origin.y + h,
            z: origin.z,
        };
        if current_water && above_water && can_survive {
            if h == height {
                let age = 20 + age_rolls.first().copied().unwrap_or(0).rem_euclid(4);
                blocks.push(AquaticPlacementBlock {
                    pos,
                    state: kelp_state_for_age(age),
                });
            } else {
                blocks.push(AquaticPlacementBlock {
                    pos,
                    state: "minecraft:kelp_plant",
                });
            }
        } else if h > 0 {
            let below_index = h as usize - 1;
            if survival_column.get(below_index).copied().unwrap_or(false) && !below_is_kelp {
                let age = 20 + age_rolls.first().copied().unwrap_or(0).rem_euclid(4);
                blocks.pop();
                blocks.push(AquaticPlacementBlock {
                    pos: BlockPos {
                        x: origin.x,
                        y: origin.y + h - 1,
                        z: origin.z,
                    },
                    state: kelp_state_for_age(age),
                });
            }
            break;
        }
    }
    blocks
}

const fn kelp_state_for_age(age: i32) -> &'static str {
    match age {
        20 => "minecraft:kelp[age=20]",
        21 => "minecraft:kelp[age=21]",
        22 => "minecraft:kelp[age=22]",
        _ => "minecraft:kelp[age=23]",
    }
}

pub fn coral_block_can_place(current_block: &'static str, above_block: &'static str) -> bool {
    (current_block == "minecraft:water" || block_is_coral(current_block))
        && above_block == "minecraft:water"
}

pub fn coral_block_placement_plan(
    pos: BlockPos,
    current_block: &'static str,
    above_block: &'static str,
    coral_state: &'static str,
    coral_roll: f32,
    sea_pickle_roll: f32,
    pickle_count_roll: i32,
    wall_fan_rolls: &[(HorizontalDirection, f32, bool)],
) -> Vec<AquaticPlacementBlock> {
    if !coral_block_can_place(current_block, above_block) {
        return Vec::new();
    }
    let mut blocks = vec![AquaticPlacementBlock {
        pos,
        state: coral_state,
    }];
    if coral_roll < 0.25 {
        blocks.push(AquaticPlacementBlock {
            pos: BlockPos {
                x: pos.x,
                y: pos.y + 1,
                z: pos.z,
            },
            state: "minecraft:tube_coral",
        });
    } else if sea_pickle_roll < 0.05 {
        blocks.push(AquaticPlacementBlock {
            pos: BlockPos {
                x: pos.x,
                y: pos.y + 1,
                z: pos.z,
            },
            state: match pickle_count_roll.rem_euclid(4) + 1 {
                1 => "minecraft:sea_pickle[pickles=1]",
                2 => "minecraft:sea_pickle[pickles=2]",
                3 => "minecraft:sea_pickle[pickles=3]",
                _ => "minecraft:sea_pickle[pickles=4]",
            },
        });
    }
    for (direction, roll, side_is_water) in wall_fan_rolls {
        if *roll < 0.2 && *side_is_water {
            blocks.push(AquaticPlacementBlock {
                pos: offset_horizontal(pos, *direction, 1),
                state: coral_wall_fan_state(*direction),
            });
        }
    }
    blocks
}

pub fn coral_tree_positions(
    origin: BlockPos,
    trunk_height_roll: i32,
    branch_directions: &[HorizontalDirection],
    branch_height_rolls: &[i32],
    branch_step_rolls: &[f32],
) -> Vec<BlockPos> {
    let trunk_height = trunk_height_roll.rem_euclid(3) + 1;
    let mut positions = Vec::new();
    for y in 0..trunk_height {
        positions.push(BlockPos {
            x: origin.x,
            y: origin.y + y,
            z: origin.z,
        });
    }
    let trunk_top = BlockPos {
        x: origin.x,
        y: origin.y + trunk_height,
        z: origin.z,
    };
    for (branch_index, direction) in branch_directions.iter().take(4).enumerate() {
        let mut pos = offset_horizontal(trunk_top, *direction, 1);
        let branch_height = branch_height_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(5)
            + 2;
        let mut segment_length = 0;
        for j in 0..branch_height {
            positions.push(pos);
            segment_length += 1;
            pos.y += 1;
            let roll = branch_step_rolls
                .get(branch_index * 5 + j as usize)
                .copied()
                .unwrap_or(1.0);
            if j == 0 || (segment_length >= 2 && roll < 0.25) {
                pos = offset_horizontal(pos, *direction, 1);
                segment_length = 0;
            }
        }
    }
    positions
}

pub fn coral_mushroom_positions(
    origin: BlockPos,
    height_roll: i32,
    width_roll: i32,
    length_roll: i32,
    sink_roll: i32,
    skip_rolls: &[f32],
) -> Vec<BlockPos> {
    let height = height_roll.rem_euclid(3) + 3;
    let width = width_roll.rem_euclid(3) + 3;
    let length = length_roll.rem_euclid(3) + 3;
    let sink = sink_roll.rem_euclid(3) + 1;
    let mut positions = Vec::new();
    let mut roll_index = 0;
    for x in 0..=width {
        for y in 0..=height {
            for z in 0..=length {
                let not_x_edge_or_y_edge = (x != 0 && x != width) || (y != 0 && y != height);
                let not_z_edge_or_y_edge = (z != 0 && z != length) || (y != 0 && y != height);
                let not_x_edge_or_z_edge = (x != 0 && x != width) || (z != 0 && z != length);
                let on_shell =
                    x == 0 || x == width || y == 0 || y == height || z == 0 || z == length;
                if not_x_edge_or_y_edge && not_z_edge_or_y_edge && not_x_edge_or_z_edge && on_shell
                {
                    let roll = skip_rolls.get(roll_index).copied().unwrap_or(1.0);
                    roll_index += 1;
                    if roll >= 0.1 {
                        positions.push(BlockPos {
                            x: origin.x + x,
                            y: origin.y + y - sink,
                            z: origin.z + z,
                        });
                    }
                }
            }
        }
    }
    positions
}

pub fn coral_claw_positions(
    origin: BlockPos,
    claw_direction: HorizontalDirection,
    branch_directions: &[HorizontalDirection],
    sideway_rolls: &[i32],
    inway_rolls: &[i32],
    up_rolls: &[f32],
) -> Vec<BlockPos> {
    let mut positions = vec![origin];
    for (branch_index, direction) in branch_directions.iter().take(3).enumerate() {
        let sideway_length = sideway_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(2)
            + 1;
        let inway_length = inway_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(3)
            + if *direction == claw_direction { 2 } else { 3 };
        let mut pos = offset_horizontal(origin, *direction, 1);
        if *direction != claw_direction {
            pos.y += 1;
        }
        for _ in 0..sideway_length {
            positions.push(pos);
            pos = offset_horizontal(pos, *direction, 1);
        }
        pos = offset_horizontal(pos, direction.opposite(), 1);
        pos.y += 1;
        for i in 0..inway_length {
            pos = offset_horizontal(pos, claw_direction, 1);
            positions.push(pos);
            if up_rolls
                .get(branch_index * 5 + i as usize)
                .copied()
                .unwrap_or(1.0)
                < 0.25
            {
                pos.y += 1;
            }
        }
    }
    positions
}

pub fn vegetation_patch_radius(min_radius: i32, max_radius: i32, roll: i32) -> i32 {
    let span = (max_radius - min_radius + 1).max(1);
    min_radius + roll.rem_euclid(span) + 1
}

pub fn vegetation_patch_should_try_column(
    dx: i32,
    dz: i32,
    x_radius: i32,
    z_radius: i32,
    extra_edge_column_chance: f32,
    edge_roll: f32,
) -> bool {
    let is_x_edge = dx == -x_radius || dx == x_radius;
    let is_z_edge = dz == -z_radius || dz == z_radius;
    let is_corner = is_x_edge && is_z_edge;
    let is_edge_but_not_corner = (is_x_edge || is_z_edge) && !is_corner;
    !is_corner
        && (!is_edge_but_not_corner
            || (extra_edge_column_chance != 0.0 && edge_roll <= extra_edge_column_chance))
}

pub fn vegetation_patch_depth(
    min_depth: i32,
    max_depth: i32,
    depth_roll: i32,
    extra_bottom_block_chance: f32,
    extra_roll: f32,
) -> i32 {
    let span = (max_depth - min_depth + 1).max(1);
    min_depth
        + depth_roll.rem_euclid(span)
        + i32::from(extra_bottom_block_chance > 0.0 && extra_roll < extra_bottom_block_chance)
}

pub fn vegetation_patch_place_ground(
    config: &VegetationPatchConfigurationModel,
    start: BlockPos,
    existing_blocks: &[&'static str],
    depth: i32,
    random_roll: i32,
) -> Option<Vec<VegetationPatchBlock>> {
    let state = block_state_provider_sample(&config.ground_state, random_roll)?;
    let mut blocks = Vec::new();
    for i in 0..depth.max(0) {
        let existing = existing_blocks
            .get(i as usize)
            .copied()
            .unwrap_or("minecraft:air");
        if existing == state {
            continue;
        }
        if !config.replaceable.contains(&existing) {
            return (!blocks.is_empty()).then_some(blocks);
        }
        blocks.push(VegetationPatchBlock {
            pos: offset_vertical(start, config.surface, i),
            state,
        });
    }
    Some(blocks)
}

pub fn vegetation_patch_plan(
    config: &VegetationPatchConfigurationModel,
    columns: &[VegetationPatchGroundColumn],
    existing_blocks: &[&[&'static str]],
    vegetation_rolls: &[f32],
) -> VegetationPatchPlan {
    let mut ground = Vec::new();
    let mut vegetation_origins = Vec::new();
    for (index, column) in columns.iter().enumerate() {
        if let Some(mut column_blocks) = vegetation_patch_place_ground(
            config,
            column.ground_start,
            existing_blocks.get(index).copied().unwrap_or(&[]),
            column.depth,
            index as i32,
        ) {
            if !column_blocks.is_empty() {
                ground.append(&mut column_blocks);
                if config.vegetation_chance > 0.0
                    && vegetation_rolls.get(index).copied().unwrap_or(1.0)
                        < config.vegetation_chance
                {
                    vegetation_origins.push(offset_vertical(
                        column.surface_pos,
                        config.surface.opposite(),
                        1,
                    ));
                }
            }
        }
    }
    VegetationPatchPlan {
        ground,
        vegetation_origins,
    }
}

pub fn lake_grid_index(x: i32, y: i32, z: i32) -> usize {
    ((x * 16 + z) * 8 + y) as usize
}

pub fn lake_is_boundary(grid: &[bool], x: i32, y: i32, z: i32) -> bool {
    if *grid.get(lake_grid_index(x, y, z)).unwrap_or(&false) {
        return false;
    }
    (x < 15 && *grid.get(lake_grid_index(x + 1, y, z)).unwrap_or(&false))
        || (x > 0 && *grid.get(lake_grid_index(x - 1, y, z)).unwrap_or(&false))
        || (z < 15 && *grid.get(lake_grid_index(x, y, z + 1)).unwrap_or(&false))
        || (z > 0 && *grid.get(lake_grid_index(x, y, z - 1)).unwrap_or(&false))
        || (y < 7 && *grid.get(lake_grid_index(x, y + 1, z)).unwrap_or(&false))
        || (y > 0 && *grid.get(lake_grid_index(x, y - 1, z)).unwrap_or(&false))
}

pub fn lake_can_place(
    min_y: i32,
    origin_y: i32,
    grid: &[bool],
    boundary: &[LakeBoundaryBlock],
    fluid: &'static str,
) -> bool {
    if origin_y <= min_y + 4 {
        return false;
    }
    boundary.iter().all(|block| {
        !lake_is_boundary(grid, block.x, block.y, block.z)
            || if block.y >= 4 {
                !block.liquid
            } else {
                block.solid || block.state == fluid
            }
    })
}

pub fn lake_placement_plan(
    origin: BlockPos,
    config: &LakeConfigurationModel,
    grid: &[bool],
    boundary: &[LakeBoundaryBlock],
    barrier_rolls: &[i32],
    freeze_water: bool,
) -> Option<Vec<LakePlacementBlock>> {
    let fluid = block_state_provider_sample(&config.fluid, 0)?;
    let barrier = block_state_provider_sample(&config.barrier, 0)?;
    let mut blocks = Vec::new();
    for x in 0..16 {
        for z in 0..16 {
            for y in 0..8 {
                if *grid.get(lake_grid_index(x, y, z)).unwrap_or(&false) {
                    let place_air = y >= 4;
                    blocks.push(LakePlacementBlock {
                        pos: BlockPos {
                            x: origin.x + x,
                            y: origin.y + y,
                            z: origin.z + z,
                        },
                        state: if place_air {
                            "minecraft:cave_air"
                        } else {
                            fluid
                        },
                        schedule_tick: place_air,
                        mark_above_for_post_processing: place_air,
                    });
                }
            }
        }
    }
    if barrier != "minecraft:air" {
        for block in boundary {
            if lake_is_boundary(grid, block.x, block.y, block.z)
                && (block.y < 4
                    || barrier_rolls
                        .get(lake_grid_index(block.x, block.y, block.z))
                        .copied()
                        .unwrap_or(1)
                        .rem_euclid(2)
                        != 0)
                && block.solid
                && !block.cannot_replace
            {
                blocks.push(LakePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + block.x,
                        y: origin.y + block.y,
                        z: origin.z + block.z,
                    },
                    state: barrier,
                    schedule_tick: false,
                    mark_above_for_post_processing: true,
                });
            }
        }
    }
    if freeze_water && fluid == "minecraft:water" {
        for block in boundary
            .iter()
            .filter(|block| block.y == 4 && block.should_freeze)
        {
            blocks.push(LakePlacementBlock {
                pos: BlockPos {
                    x: origin.x + block.x,
                    y: origin.y + 4,
                    z: origin.z + block.z,
                },
                state: "minecraft:ice",
                schedule_tick: false,
                mark_above_for_post_processing: false,
            });
        }
    }
    Some(blocks)
}

pub fn validate_fossil_config(
    config: &FossilFeatureConfigurationModel,
) -> Result<(), &'static str> {
    if config.fossil_structures.is_empty() {
        Err("Fossil structure lists need at least one entry")
    } else if config.fossil_structures.len() != config.overlay_structures.len() {
        Err("Fossil structure lists must be equal lengths")
    } else if !(0..=7).contains(&config.max_empty_corners_allowed) {
        Err("max_empty_corners_allowed must be in 0..=7")
    } else {
        Ok(())
    }
}

pub fn fossil_rotation(rotation_roll: i32) -> StructureRotation {
    match rotation_roll.rem_euclid(4) {
        0 => StructureRotation::None,
        1 => StructureRotation::Clockwise90,
        2 => StructureRotation::Clockwise180,
        _ => StructureRotation::Counterclockwise90,
    }
}

pub fn fossil_target_y(lowest_surface_y: i32, min_y: i32, depth_roll: i32) -> i32 {
    (lowest_surface_y - 15 - depth_roll.rem_euclid(10)).max(min_y + 10)
}

pub fn fossil_low_corner(origin: BlockPos, rotated_size_x: i32, rotated_size_z: i32) -> BlockPos {
    BlockPos {
        x: origin.x - rotated_size_x / 2,
        y: origin.y,
        z: origin.z - rotated_size_z / 2,
    }
}

pub fn fossil_placement_plan(
    config: &FossilFeatureConfigurationModel,
    origin: BlockPos,
    rotated_size_x: i32,
    rotated_size_z: i32,
    lowest_surface_y: i32,
    min_y: i32,
    rotation_roll: i32,
    fossil_index_roll: i32,
    depth_roll: i32,
    empty_corners: i32,
) -> Option<FossilPlacementPlan> {
    validate_fossil_config(config).ok()?;
    if empty_corners > config.max_empty_corners_allowed {
        return None;
    }
    let index = fossil_index_roll.rem_euclid(config.fossil_structures.len() as i32) as usize;
    let low_corner = fossil_low_corner(origin, rotated_size_x, rotated_size_z);
    Some(FossilPlacementPlan {
        fossil_structure: config.fossil_structures[index],
        overlay_structure: config.overlay_structures[index],
        rotation: fossil_rotation(rotation_roll),
        target_pos: BlockPos {
            x: low_corner.x,
            y: fossil_target_y(lowest_surface_y, min_y, depth_roll),
            z: low_corner.z,
        },
        fossil_processors: config.fossil_processors,
        overlay_processors: config.overlay_processors,
    })
}

pub fn validate_geode_config(config: &GeodeConfigurationModel) -> Result<(), &'static str> {
    let layer_values = [
        config.layers.filling,
        config.layers.inner_layer,
        config.layers.middle_layer,
        config.layers.outer_layer,
    ];
    if layer_values
        .iter()
        .any(|layer| !(0.01..=50.0).contains(layer))
    {
        Err("geode layer values must be in 0.01..=50.0")
    } else if !(0.0..=1.0).contains(&config.crack.generate_crack_chance)
        || !(0.0..=1.0).contains(&config.use_potential_placements_chance)
        || !(0.0..=1.0).contains(&config.use_alternate_layer0_chance)
    {
        Err("geode chances must be in 0.0..=1.0")
    } else if !(0.0..=5.0).contains(&config.crack.base_crack_size) {
        Err("geode crack base size must be in 0.0..=5.0")
    } else if !(0..=10).contains(&config.crack.crack_point_offset) {
        Err("geode crack point offset must be in 0..=10")
    } else if !(1..=20).contains(&config.outer_wall_distance_max) {
        Err("geode outer wall distance max must be in 1..=20")
    } else if config.inner_placements.is_empty() {
        Err("geode inner placements need at least one state")
    } else {
        Ok(())
    }
}

pub fn geode_invalid_point_count(config: &GeodeConfigurationModel, sampled_states: &[&str]) -> i32 {
    sampled_states
        .iter()
        .filter(|state| **state == "minecraft:air" || config.invalid_blocks.contains(state))
        .count() as i32
}

pub fn geode_can_place(config: &GeodeConfigurationModel, sampled_states: &[&str]) -> bool {
    geode_invalid_point_count(config, sampled_states) <= config.invalid_blocks_threshold
}

pub fn geode_layer_thresholds(
    layers: GeodeLayerSettingsModel,
    crack: GeodeCrackSettingsModel,
    num_points: i32,
    outer_wall_distance_max: i32,
    crack_roll: f64,
) -> GeodeLayerThresholds {
    let crack_size_adjustment = f64::from(num_points) / f64::from(outer_wall_distance_max.max(1));
    GeodeLayerThresholds {
        inner_air: 1.0 / layers.filling.sqrt(),
        innermost_block_layer: 1.0 / (layers.inner_layer + crack_size_adjustment).sqrt(),
        inner_crust: 1.0 / (layers.middle_layer + crack_size_adjustment).sqrt(),
        outer_crust: 1.0 / (layers.outer_layer + crack_size_adjustment).sqrt(),
        crack_size: 1.0
            / (crack.base_crack_size
                + crack_roll / 2.0
                + if num_points > 3 {
                    crack_size_adjustment
                } else {
                    0.0
                })
            .sqrt(),
    }
}

pub fn geode_should_generate_crack(crack: GeodeCrackSettingsModel, roll: f32) -> bool {
    roll < crack.generate_crack_chance
}

pub fn geode_crack_points(origin: BlockPos, num_points: i32, offset_roll: i32) -> Vec<BlockPos> {
    let crack_offset = num_points * 2 + 1;
    let offsets = match offset_roll.rem_euclid(4) {
        0 => [
            (crack_offset, 7, 0),
            (crack_offset, 5, 0),
            (crack_offset, 1, 0),
        ],
        1 => [
            (0, 7, crack_offset),
            (0, 5, crack_offset),
            (0, 1, crack_offset),
        ],
        2 => [
            (crack_offset, 7, crack_offset),
            (crack_offset, 5, crack_offset),
            (crack_offset, 1, crack_offset),
        ],
        _ => [(0, 7, 0), (0, 5, 0), (0, 1, 0)],
    };
    offsets
        .into_iter()
        .map(|(x, y, z)| BlockPos {
            x: origin.x + x,
            y: origin.y + y,
            z: origin.z + z,
        })
        .collect()
}

fn geode_distance_sqr(a: BlockPos, b: BlockPos) -> f64 {
    let dx = f64::from(a.x - b.x);
    let dy = f64::from(a.y - b.y);
    let dz = f64::from(a.z - b.z);
    dx * dx + dy * dy + dz * dz
}

pub fn geode_shell_density(
    pos: BlockPos,
    points: &[GeodeDistributionPoint],
    noise_offset: f64,
) -> f64 {
    points
        .iter()
        .map(|point| 1.0 / (geode_distance_sqr(pos, point.pos) + f64::from(point.offset)).sqrt())
        .sum::<f64>()
        + noise_offset * points.len() as f64
}

pub fn geode_crack_density(
    pos: BlockPos,
    crack_points: &[BlockPos],
    crack_point_offset: i32,
    noise_offset: f64,
) -> f64 {
    crack_points
        .iter()
        .map(|point| 1.0 / (geode_distance_sqr(pos, *point) + f64::from(crack_point_offset)).sqrt())
        .sum::<f64>()
        + noise_offset * crack_points.len() as f64
}

pub fn geode_layer_for_density(
    shell_density: f64,
    crack_density: f64,
    thresholds: GeodeLayerThresholds,
    should_generate_crack: bool,
    alternate_inner_roll: f32,
    use_alternate_layer0_chance: f32,
) -> Option<GeodeLayer> {
    if shell_density < thresholds.outer_crust {
        None
    } else if should_generate_crack
        && crack_density >= thresholds.crack_size
        && shell_density < thresholds.inner_air
    {
        Some(GeodeLayer::CrackAir)
    } else if shell_density >= thresholds.inner_air {
        Some(GeodeLayer::Filling)
    } else if shell_density >= thresholds.innermost_block_layer {
        if alternate_inner_roll < use_alternate_layer0_chance {
            Some(GeodeLayer::AlternateInner)
        } else {
            Some(GeodeLayer::Inner)
        }
    } else if shell_density >= thresholds.inner_crust {
        Some(GeodeLayer::Middle)
    } else {
        Some(GeodeLayer::Outer)
    }
}

pub fn geode_placement_block(
    config: &GeodeConfigurationModel,
    pos: BlockPos,
    layer: GeodeLayer,
    provider_roll: i32,
    potential_roll: f32,
) -> Option<GeodePlacementBlock> {
    let state = match layer {
        GeodeLayer::CrackAir => "minecraft:air",
        GeodeLayer::Filling => {
            block_state_provider_sample(&config.filling_provider, provider_roll)?
        }
        GeodeLayer::Inner => {
            block_state_provider_sample(&config.inner_layer_provider, provider_roll)?
        }
        GeodeLayer::AlternateInner => {
            block_state_provider_sample(&config.alternate_inner_layer_provider, provider_roll)?
        }
        GeodeLayer::Middle => {
            block_state_provider_sample(&config.middle_layer_provider, provider_roll)?
        }
        GeodeLayer::Outer => {
            block_state_provider_sample(&config.outer_layer_provider, provider_roll)?
        }
    };
    let potential_crystal_source = matches!(layer, GeodeLayer::Inner | GeodeLayer::AlternateInner)
        && (!config.placements_require_layer0_alternate || layer == GeodeLayer::AlternateInner)
        && potential_roll < config.use_potential_placements_chance;
    Some(GeodePlacementBlock {
        pos,
        state,
        layer,
        potential_crystal_source,
    })
}

pub fn geode_inner_placement(config: &GeodeConfigurationModel, roll: i32) -> Option<&'static str> {
    if config.inner_placements.is_empty() {
        None
    } else {
        Some(
            config.inner_placements[roll.rem_euclid(config.inner_placements.len() as i32) as usize],
        )
    }
}

pub fn iceberg_shape_model(
    snow_roll: f64,
    angle_roll: f64,
    ellipse_a_roll: i32,
    ellipse_c_roll: i32,
    ellipse_roll: f64,
    height_roll: i32,
    tall_roll: f64,
    tall_extra_roll: i32,
    underwater_roll: i32,
    width_plus_roll: i32,
    width_minus_roll: i32,
) -> IcebergShapeModel {
    let is_ellipse = ellipse_roll > 0.7;
    let mut over_water_height = if is_ellipse {
        height_roll.rem_euclid(6) + 6
    } else {
        height_roll.rem_euclid(15) + 3
    };
    if !is_ellipse && tall_roll > 0.9 {
        over_water_height += tall_extra_roll.rem_euclid(19) + 7;
    }
    IcebergShapeModel {
        snow_on_top: snow_roll > 0.7,
        shape_angle: angle_roll * 2.0 * std::f64::consts::PI,
        shape_ellipse_a: 11 - ellipse_a_roll.rem_euclid(5),
        shape_ellipse_c: 3 + ellipse_c_roll.rem_euclid(3),
        is_ellipse,
        over_water_height,
        under_water_height: (over_water_height + underwater_roll.rem_euclid(11)).min(18),
        width: (over_water_height + width_plus_roll.rem_euclid(7) - width_minus_roll.rem_euclid(5))
            .min(11),
    }
}

pub fn iceberg_ellipse_c(y_off: i32, height: i32, shape_ellipse_c: i32) -> i32 {
    if y_off > 0 && height - y_off <= 3 {
        shape_ellipse_c - (4 - (height - y_off))
    } else {
        shape_ellipse_c
    }
}

pub fn iceberg_signed_distance_circle(
    xo: i32,
    zo: i32,
    origin: BlockPos,
    radius: i32,
    float_roll: f32,
) -> f64 {
    let off = 10.0 * f64::from(float_roll.clamp(0.2, 0.8)) / f64::from(radius.max(1));
    let dx = f64::from(xo - origin.x);
    let dz = f64::from(zo - origin.z);
    off + dx.powi(2) + dz.powi(2) - f64::from(radius).powi(2)
}

pub fn iceberg_signed_distance_ellipse(
    xo: i32,
    zo: i32,
    origin: BlockPos,
    a: i32,
    c: i32,
    angle: f64,
) -> f64 {
    let dx = f64::from(xo - origin.x);
    let dz = f64::from(zo - origin.z);
    ((dx * angle.cos() - dz * angle.sin()) / f64::from(a.max(1))).powi(2)
        + ((dx * angle.sin() + dz * angle.cos()) / f64::from(c.max(1))).powi(2)
        - 1.0
}

pub fn iceberg_height_radius_round(
    y_off: i32,
    height: i32,
    width: i32,
    float_roll: f32,
    tall_height_roll: i32,
    tall_y_roll: i32,
) -> i32 {
    let k = 3.5 - float_roll;
    let mut effective_y = y_off;
    let mut scale = (1.0 - (y_off as f32).powi(2) / (height as f32 * k)) * width as f32;
    if height > 15 + tall_height_roll.rem_euclid(5) {
        if y_off < 3 + tall_y_roll.rem_euclid(6) {
            effective_y = y_off / 2;
        }
        scale = (1.0 - effective_y as f32 / (height as f32 * k * 0.4)) * width as f32;
    }
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_height_radius_ellipse(y_off: i32, height: i32, width: i32) -> i32 {
    let scale = (1.0 - (y_off as f32).powi(2) / height as f32) * width as f32;
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_height_radius_steep(y_off: i32, height: i32, width: i32, float_roll: f32) -> i32 {
    let k = 1.0 + float_roll / 2.0;
    let scale = (1.0 - y_off as f32 / (height as f32 * k)) * width as f32;
    (scale / 2.0).ceil() as i32
}

pub fn iceberg_set_block_action(
    current_state: &str,
    h_diff: i32,
    height: i32,
    is_ellipse: bool,
    snow_on_top: bool,
    snow_height_roll: i32,
    ellipse_skip_roll: f64,
) -> IcebergBlockAction {
    if !matches!(
        current_state,
        "minecraft:air" | "minecraft:snow_block" | "minecraft:ice" | "minecraft:water"
    ) {
        return IcebergBlockAction::Keep;
    }
    let randomness = !is_ellipse || ellipse_skip_roll > 0.05;
    let divisor = if is_ellipse { 3 } else { 2 };
    let snow_limit =
        snow_height_roll.rem_euclid((height / divisor).max(1)) as f64 + f64::from(height) * 0.6;
    if snow_on_top
        && current_state != "minecraft:water"
        && f64::from(h_diff) <= snow_limit
        && randomness
    {
        IcebergBlockAction::SnowBlock
    } else {
        IcebergBlockAction::MainBlock
    }
}

pub fn iceberg_should_skip_surface_noise(
    signed_distance: f64,
    is_ellipse: bool,
    roll: f64,
) -> bool {
    let compare_val = if is_ellipse { -0.5 } else { -6.0 };
    signed_distance > compare_val && roll > 0.9
}

pub fn iceberg_carve_action(current_state: &str, under_water: bool) -> IcebergBlockAction {
    if matches!(
        current_state,
        "minecraft:packed_ice" | "minecraft:snow_block" | "minecraft:blue_ice"
    ) {
        if under_water {
            IcebergBlockAction::Water
        } else {
            IcebergBlockAction::Air
        }
    } else {
        IcebergBlockAction::Keep
    }
}

pub fn iceberg_smooth_action(
    current_state: &str,
    below_is_air: bool,
    horizontal_non_iceberg_neighbors: i32,
) -> IcebergBlockAction {
    if matches!(
        current_state,
        "minecraft:packed_ice" | "minecraft:snow_block" | "minecraft:blue_ice" | "minecraft:snow"
    ) && below_is_air
    {
        IcebergBlockAction::Air
    } else if matches!(
        current_state,
        "minecraft:packed_ice" | "minecraft:snow_block" | "minecraft:blue_ice"
    ) && horizontal_non_iceberg_neighbors >= 3
    {
        IcebergBlockAction::Air
    } else {
        IcebergBlockAction::Keep
    }
}

pub fn validate_weighted_placed_feature(
    feature: WeightedPlacedFeatureModel,
) -> Result<WeightedPlacedFeatureModel, &'static str> {
    if feature.feature.is_empty() {
        Err("weighted placed feature must reference a feature")
    } else if !(0.0..=1.0).contains(&feature.chance) {
        Err("weighted placed feature chance must be in 0.0..=1.0")
    } else {
        Ok(feature)
    }
}

pub fn random_selector_feature<'a>(
    config: &'a RandomFeatureConfigurationModel,
    chance_rolls: &[f32],
) -> Option<&'a str> {
    for (index, feature) in config.features.iter().enumerate() {
        validate_weighted_placed_feature(*feature).ok()?;
        let roll = chance_rolls.get(index).copied().unwrap_or(1.0);
        if roll < feature.chance {
            return Some(feature.feature);
        }
    }
    Some(config.default_feature)
}

pub fn simple_random_selector_feature<'a>(
    config: &'a SimpleRandomFeatureConfigurationModel,
    index_roll: i32,
) -> Option<&'a str> {
    if config.features.is_empty() {
        None
    } else {
        Some(config.features[index_roll.rem_euclid(config.features.len() as i32) as usize])
    }
}

pub fn random_boolean_selector_feature(
    config: RandomBooleanFeatureConfigurationModel,
    roll: bool,
) -> &'static str {
    if roll {
        config.feature_true
    } else {
        config.feature_false
    }
}

pub fn validate_fill_layer_config(
    config: FillLayerConfigurationModel,
    dimension_y_size: i32,
) -> Result<FillLayerConfigurationModel, &'static str> {
    if !(0..=dimension_y_size).contains(&config.height) {
        Err("fill layer height must be in 0..=dimension_y_size")
    } else if config.state.is_empty() {
        Err("fill layer state must not be empty")
    } else {
        Ok(config)
    }
}

pub fn fill_layer_placement_plan(
    origin: BlockPos,
    min_y: i32,
    config: FillLayerConfigurationModel,
    air_columns: &[bool],
) -> Vec<BlockPos> {
    let mut placements = Vec::new();
    let y = min_y + config.height;
    for dx in 0..16 {
        for dz in 0..16 {
            let index = (dx * 16 + dz) as usize;
            if air_columns.get(index).copied().unwrap_or(false) {
                placements.push(BlockPos {
                    x: origin.x + dx,
                    y,
                    z: origin.z + dz,
                });
            }
        }
    }
    placements
}

pub fn end_island_layer_radius(size: f32) -> i32 {
    size.ceil() as i32
}

pub fn end_island_next_size(size: f32, shrink_roll: i32) -> f32 {
    size - (shrink_roll.rem_euclid(2) as f32 + 0.5)
}

pub fn end_island_placement_plan(
    origin: BlockPos,
    size_roll: i32,
    shrink_rolls: &[i32],
) -> Vec<EndIslandPlacementBlock> {
    let mut blocks = Vec::new();
    let mut size = size_roll.rem_euclid(3) as f32 + 4.0;
    let mut y_offset = 0;
    let mut shrink_index = 0;
    while size > 0.5 {
        let min = (-size).floor() as i32;
        let max = end_island_layer_radius(size);
        for x in min..=max {
            for z in min..=max {
                if (x * x + z * z) as f32 <= (size + 1.0) * (size + 1.0) {
                    blocks.push(EndIslandPlacementBlock {
                        pos: BlockPos {
                            x: origin.x + x,
                            y: origin.y + y_offset,
                            z: origin.z + z,
                        },
                        state: "minecraft:end_stone",
                    });
                }
            }
        }
        let shrink_roll = shrink_rolls.get(shrink_index).copied().unwrap_or(0);
        shrink_index += 1;
        size = end_island_next_size(size, shrink_roll);
        y_offset -= 1;
    }
    blocks
}

pub fn validate_replace_sphere_config(
    config: ReplaceSphereConfigurationModel,
) -> Result<ReplaceSphereConfigurationModel, &'static str> {
    if config.target_state.is_empty() || config.replace_state.is_empty() {
        Err("replace sphere target and replacement states must not be empty")
    } else if !(0..=12).contains(&config.radius_min)
        || !(0..=12).contains(&config.radius_max)
        || config.radius_min > config.radius_max
    {
        Err("replace sphere radius bounds must be ordered in 0..=12")
    } else {
        Ok(config)
    }
}

pub fn replace_sphere_radius(config: ReplaceSphereConfigurationModel, roll: i32) -> i32 {
    let span = (config.radius_max - config.radius_min + 1).max(1);
    config.radius_min + roll.rem_euclid(span)
}

pub fn replace_sphere_find_target(
    origin: BlockPos,
    min_y: i32,
    max_y: i32,
    column_states: &[&str],
    target_state: &str,
) -> Option<BlockPos> {
    let mut y = origin.y.clamp(min_y + 1, max_y);
    while y > min_y + 1 {
        let offset = origin.y.clamp(min_y + 1, max_y) - y;
        if column_states.get(offset as usize).copied() == Some(target_state) {
            return Some(BlockPos {
                x: origin.x,
                y,
                z: origin.z,
            });
        }
        y -= 1;
    }
    None
}

pub fn replace_sphere_positions(
    center: BlockPos,
    radius_x: i32,
    radius_y: i32,
    radius_z: i32,
) -> Vec<BlockPos> {
    let maximum_radius = radius_x.max(radius_y).max(radius_z);
    let mut positions = Vec::new();
    for dy in -radius_y..=radius_y {
        for dz in -radius_z..=radius_z {
            for dx in -radius_x..=radius_x {
                if dx.abs() + dy.abs() + dz.abs() <= maximum_radius {
                    positions.push(BlockPos {
                        x: center.x + dx,
                        y: center.y + dy,
                        z: center.z + dz,
                    });
                }
            }
        }
    }
    positions
}

pub fn basalt_pillar_can_start(origin_empty: bool, above_empty: bool) -> bool {
    origin_empty && !above_empty
}

pub fn basalt_pillar_hangoff_places(roll: i32) -> bool {
    roll.rem_euclid(10) != 0
}

pub fn basalt_pillar_base_places(dx: i32, dz: i32, roll: i32) -> bool {
    let probability = dx.abs() * dz.abs();
    roll.rem_euclid(10) < 10 - probability
}

pub fn basalt_pillar_placement_plan(
    origin: BlockPos,
    empty_down: &[bool],
    outside_build_height: &[bool],
    hangoff_rolls: &[(i32, i32, i32, i32)],
    base_rolls: &[i32],
    base_drop_empty_below: &[&[bool]],
    base_supported_below: &[bool],
) -> Vec<BasaltPillarPlacementBlock> {
    let mut blocks = Vec::new();
    let mut y_steps = 0usize;
    let mut north_active = true;
    let mut south_active = true;
    let mut west_active = true;
    let mut east_active = true;
    while empty_down.get(y_steps).copied().unwrap_or(false) {
        if outside_build_height.get(y_steps).copied().unwrap_or(false) {
            return blocks;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y - y_steps as i32,
            z: origin.z,
        };
        blocks.push(BasaltPillarPlacementBlock {
            pos,
            kind: BasaltPillarBlockKind::Core,
        });
        let rolls = hangoff_rolls.get(y_steps).copied().unwrap_or((0, 0, 0, 0));
        if north_active && basalt_pillar_hangoff_places(rolls.0) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::North, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            north_active = false;
        }
        if south_active && basalt_pillar_hangoff_places(rolls.1) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::South, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            south_active = false;
        }
        if west_active && basalt_pillar_hangoff_places(rolls.2) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::West, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            west_active = false;
        }
        if east_active && basalt_pillar_hangoff_places(rolls.3) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::East, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            east_active = false;
        }
        y_steps += 1;
    }

    if y_steps == 0 {
        return blocks;
    }
    let base_y = origin.y - y_steps as i32;
    let mut base_index = 0usize;
    for dx in -3..=3 {
        for dz in -3..=3 {
            let roll = base_rolls.get(base_index).copied().unwrap_or(10);
            let drop_empty = base_drop_empty_below
                .get(base_index)
                .copied()
                .unwrap_or(&[]);
            let supported = base_supported_below
                .get(base_index)
                .copied()
                .unwrap_or(false);
            base_index += 1;
            if !basalt_pillar_base_places(dx, dz, roll) {
                continue;
            }
            let drop = drop_empty
                .iter()
                .take(3)
                .take_while(|empty| **empty)
                .count() as i32;
            if supported {
                blocks.push(BasaltPillarPlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: base_y - drop,
                        z: origin.z + dz,
                    },
                    kind: BasaltPillarBlockKind::Base,
                });
            }
        }
    }
    blocks
}

pub fn basalt_columns_cannot_place_on(state: &str) -> bool {
    matches!(
        state,
        "minecraft:lava"
            | "minecraft:bedrock"
            | "minecraft:magma_block"
            | "minecraft:soul_sand"
            | "minecraft:nether_bricks"
            | "minecraft:nether_brick_fence"
            | "minecraft:nether_brick_stairs"
            | "minecraft:nether_wart"
            | "minecraft:chest"
            | "minecraft:spawner"
    )
}

pub fn basalt_columns_is_air_or_lava_ocean(state: &str, y: i32, lava_sea_level: i32) -> bool {
    state == "minecraft:air" || (state == "minecraft:lava" && y <= lava_sea_level)
}

pub fn basalt_columns_can_place_at(
    state: &str,
    below_state: &str,
    y: i32,
    lava_sea_level: i32,
) -> bool {
    basalt_columns_is_air_or_lava_ocean(state, y, lava_sea_level)
        && below_state != "minecraft:air"
        && !basalt_columns_cannot_place_on(below_state)
}

pub fn validate_column_feature_config(
    config: ColumnFeatureConfigurationModel,
) -> Result<ColumnFeatureConfigurationModel, &'static str> {
    if !(0..=3).contains(&config.reach_min)
        || !(0..=3).contains(&config.reach_max)
        || config.reach_min > config.reach_max
    {
        Err("column reach bounds must be ordered in 0..=3")
    } else if !(1..=10).contains(&config.height_min)
        || !(1..=10).contains(&config.height_max)
        || config.height_min > config.height_max
    {
        Err("column height bounds must be ordered in 1..=10")
    } else {
        Ok(config)
    }
}

pub fn basalt_columns_cluster_parameters(
    column_height: i32,
    clustered_roll: f32,
) -> (bool, i32, i32) {
    let clustered = clustered_roll < 0.9;
    let reach = column_height.min(if clustered { 5 } else { 8 });
    let count = if clustered { 50 } else { 15 };
    (clustered, reach, count)
}

pub fn basalt_column_blocks_from_surface(
    surface_pos: BlockPos,
    origin: BlockPos,
    column_height: i32,
    reach: i32,
    air_or_lava_ocean_above: &[bool],
    already_basalt_above: &[bool],
) -> Vec<BasaltColumnPlacementBlock> {
    let mut blocks = Vec::new();
    let step_limit = (surface_pos.x - origin.x).abs()
        + (surface_pos.y - origin.y).abs()
        + (surface_pos.z - origin.z).abs();
    if step_limit > reach {
        return blocks;
    }
    let mut blocks_y = column_height - step_limit / 2;
    let mut step = 0usize;
    while blocks_y >= 0 {
        if air_or_lava_ocean_above.get(step).copied().unwrap_or(false) {
            blocks.push(BasaltColumnPlacementBlock {
                pos: BlockPos {
                    x: surface_pos.x,
                    y: surface_pos.y + step as i32,
                    z: surface_pos.z,
                },
            });
        } else if !already_basalt_above.get(step).copied().unwrap_or(false) {
            break;
        }
        blocks_y -= 1;
        step += 1;
    }
    blocks
}

pub fn validate_delta_config(
    config: DeltaFeatureConfigurationModel,
) -> Result<DeltaFeatureConfigurationModel, &'static str> {
    let valid_size = (0..=16).contains(&config.size_min)
        && (0..=16).contains(&config.size_max)
        && config.size_min <= config.size_max;
    let valid_rim = (0..=16).contains(&config.rim_size_min)
        && (0..=16).contains(&config.rim_size_max)
        && config.rim_size_min <= config.rim_size_max;
    if config.contents.is_empty() || config.rim.is_empty() {
        Err("delta contents and rim states must not be empty")
    } else if !valid_size {
        Err("delta size bounds must be ordered in 0..=16")
    } else if !valid_rim {
        Err("delta rim size bounds must be ordered in 0..=16")
    } else {
        Ok(config)
    }
}

pub fn delta_cannot_replace(state: &str) -> bool {
    matches!(
        state,
        "minecraft:bedrock"
            | "minecraft:nether_bricks"
            | "minecraft:nether_brick_fence"
            | "minecraft:nether_brick_stairs"
            | "minecraft:nether_wart"
            | "minecraft:chest"
            | "minecraft:spawner"
    )
}

pub fn delta_is_clear(
    state: &str,
    contents: &str,
    up_air: bool,
    down_air: bool,
    north_air: bool,
    south_air: bool,
    west_air: bool,
    east_air: bool,
) -> bool {
    state != contents
        && !delta_cannot_replace(state)
        && !up_air
        && !down_air
        && !north_air
        && !south_air
        && !west_air
        && !east_air
}

pub fn delta_has_rim(spawn_roll: f64, rim_x: i32, rim_z: i32) -> bool {
    spawn_roll < 0.9 && rim_x != 0 && rim_z != 0
}

pub fn delta_candidate_offsets(radius_x: i32, radius_z: i32) -> Vec<(i32, i32)> {
    let radius_limit = radius_x.max(radius_z);
    let mut offsets = Vec::new();
    for dz in -radius_z..=radius_z {
        for dx in -radius_x..=radius_x {
            if dx.abs() + dz.abs() <= radius_limit {
                offsets.push((dx, dz));
            }
        }
    }
    offsets
}

pub fn glowstone_can_start(origin_empty: bool, above_state: &str) -> bool {
    origin_empty
        && matches!(
            above_state,
            "minecraft:netherrack" | "minecraft:basalt" | "minecraft:blackstone"
        )
}

pub fn glowstone_candidate_offset(
    x_roll_a: i32,
    x_roll_b: i32,
    y_roll: i32,
    z_roll_a: i32,
    z_roll_b: i32,
) -> BlockPos {
    BlockPos {
        x: x_roll_a.rem_euclid(8) - x_roll_b.rem_euclid(8),
        y: -y_roll.rem_euclid(12),
        z: z_roll_a.rem_euclid(8) - z_roll_b.rem_euclid(8),
    }
}

pub fn glowstone_can_grow(candidate_empty: bool, glowstone_neighbors: i32) -> bool {
    candidate_empty && glowstone_neighbors == 1
}

pub fn nether_forest_vegetation_can_start(
    below_state: &str,
    y: i32,
    min_y: i32,
    max_y: i32,
) -> bool {
    matches!(
        below_state,
        "minecraft:crimson_nylium" | "minecraft:warped_nylium"
    ) && y >= min_y + 1
        && y + 1 <= max_y
}

pub fn validate_nether_forest_vegetation_config(
    config: &NetherForestVegetationConfigModel,
) -> Result<(), &'static str> {
    if config.spread_width <= 0 || config.spread_height <= 0 {
        Err("nether forest vegetation spread values must be positive")
    } else {
        Ok(())
    }
}

pub fn nether_forest_vegetation_offset(
    spread_width: i32,
    spread_height: i32,
    x_a: i32,
    x_b: i32,
    y_a: i32,
    y_b: i32,
    z_a: i32,
    z_b: i32,
) -> BlockPos {
    BlockPos {
        x: x_a.rem_euclid(spread_width) - x_b.rem_euclid(spread_width),
        y: y_a.rem_euclid(spread_height) - y_b.rem_euclid(spread_height),
        z: z_a.rem_euclid(spread_width) - z_b.rem_euclid(spread_width),
    }
}

pub fn nether_forest_vegetation_attempts(spread_width: i32) -> i32 {
    spread_width * spread_width
}

pub fn twisting_vines_valid_ground(state: &str) -> bool {
    matches!(
        state,
        "minecraft:netherrack" | "minecraft:warped_nylium" | "minecraft:warped_wart_block"
    )
}

pub fn weeping_vines_valid_ceiling(state: &str) -> bool {
    matches!(
        state,
        "minecraft:netherrack" | "minecraft:nether_wart_block"
    )
}

pub fn vine_height(base_roll: i32, max_height: i32, double_roll: i32, single_roll: i32) -> i32 {
    let mut height = 1 + base_roll.rem_euclid(max_height.max(1));
    if double_roll.rem_euclid(6) == 0 {
        height *= 2;
    }
    if single_roll.rem_euclid(5) == 0 {
        height = 1;
    }
    height
}

pub fn vine_age(min_age: i32, max_age: i32, roll: i32) -> i32 {
    let span = (max_age - min_age + 1).max(1);
    min_age + roll.rem_euclid(span)
}

pub fn twisting_vines_column(
    origin: BlockPos,
    total_height: i32,
    empty_up: &[bool],
    blocked_above: &[bool],
    age_roll: i32,
) -> Vec<VineColumnBlock> {
    let mut blocks = Vec::new();
    for height in 1..=total_height {
        let step = (height - 1) as usize;
        if !empty_up.get(step).copied().unwrap_or(false) {
            continue;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y + step as i32,
            z: origin.z,
        };
        if height == total_height || blocked_above.get(step).copied().unwrap_or(false) {
            blocks.push(VineColumnBlock {
                pos,
                state: "minecraft:twisting_vines",
                kind: VineColumnBlockKind::Head,
                age: Some(vine_age(17, 25, age_roll)),
            });
            break;
        }
        blocks.push(VineColumnBlock {
            pos,
            state: "minecraft:twisting_vines_plant",
            kind: VineColumnBlockKind::Plant,
            age: None,
        });
    }
    blocks
}

pub fn weeping_vines_column(
    origin: BlockPos,
    total_height: i32,
    empty_down: &[bool],
    blocked_below: &[bool],
    age_roll: i32,
) -> Vec<VineColumnBlock> {
    let mut blocks = Vec::new();
    for height in 0..=total_height {
        let step = height as usize;
        if !empty_down.get(step).copied().unwrap_or(false) {
            continue;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y - height,
            z: origin.z,
        };
        if height == total_height || blocked_below.get(step).copied().unwrap_or(false) {
            blocks.push(VineColumnBlock {
                pos,
                state: "minecraft:weeping_vines",
                kind: VineColumnBlockKind::Head,
                age: Some(vine_age(17, 25, age_roll)),
            });
            break;
        }
        blocks.push(VineColumnBlock {
            pos,
            state: "minecraft:weeping_vines_plant",
            kind: VineColumnBlockKind::Plant,
            age: None,
        });
    }
    blocks
}

pub fn weeping_vines_wart_can_grow(
    candidate_empty: bool,
    wart_or_netherrack_neighbors: i32,
) -> bool {
    candidate_empty && wart_or_netherrack_neighbors == 1
}

pub fn end_platform_blocks(origin: BlockPos) -> Vec<FeaturePlacementBlock> {
    let mut blocks = Vec::new();
    for dz in -2..=2 {
        for dx in -2..=2 {
            for dy in -1..3 {
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    state: if dy == -1 {
                        "minecraft:obsidian"
                    } else {
                        "minecraft:air"
                    },
                });
            }
        }
    }
    blocks
}

pub fn void_start_platform_origin(feature_origin_y: i32) -> BlockPos {
    BlockPos {
        x: 8,
        y: feature_origin_y + 3,
        z: 8,
    }
}

pub fn checkerboard_distance(xa: i32, za: i32, xb: i32, zb: i32) -> i32 {
    (xa - xb).abs().max((za - zb).abs())
}

pub fn chunk_pos_containing_block(x: i32, z: i32) -> ChunkPos {
    ChunkPos {
        x: x.div_euclid(16),
        z: z.div_euclid(16),
    }
}

pub fn void_start_platform_applies_to_chunk(chunk_pos: ChunkPos) -> bool {
    let platform_chunk = chunk_pos_containing_block(8, 8);
    checkerboard_distance(chunk_pos.x, chunk_pos.z, platform_chunk.x, platform_chunk.z) <= 1
}

pub fn void_start_platform_blocks(
    chunk_pos: ChunkPos,
    feature_origin_y: i32,
) -> Vec<FeaturePlacementBlock> {
    if !void_start_platform_applies_to_chunk(chunk_pos) {
        return Vec::new();
    }
    let origin = void_start_platform_origin(feature_origin_y);
    let mut blocks = Vec::new();
    for z in (chunk_pos.z * 16)..=(chunk_pos.z * 16 + 15) {
        for x in (chunk_pos.x * 16)..=(chunk_pos.x * 16 + 15) {
            if checkerboard_distance(origin.x, origin.z, x, z) <= 16 {
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos { x, y: origin.y, z },
                    state: if x == origin.x && z == origin.z {
                        "minecraft:cobblestone"
                    } else {
                        "minecraft:stone"
                    },
                });
            }
        }
    }
    blocks
}

pub fn end_gateway_known_exit(exit: BlockPos, exact: bool) -> EndGatewayConfigurationModel {
    EndGatewayConfigurationModel {
        exit: Some(exit),
        exact,
    }
}

pub fn end_gateway_delayed_exit_search() -> EndGatewayConfigurationModel {
    EndGatewayConfigurationModel {
        exit: None,
        exact: false,
    }
}

pub fn end_gateway_blocks(origin: BlockPos) -> Vec<FeaturePlacementBlock> {
    let mut blocks = Vec::new();
    for dy in -2i32..=2 {
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let same_x = dx == 0;
                let same_y = dy == 0;
                let same_z = dz == 0;
                let end = dy.abs() == 2;
                let state = if same_x && same_y && same_z {
                    "minecraft:end_gateway"
                } else if same_y {
                    "minecraft:air"
                } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                    "minecraft:bedrock"
                } else {
                    "minecraft:air"
                };
                blocks.push(FeaturePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    state,
                });
            }
        }
    }
    blocks
}

pub fn supports_chorus_plant(state: &str) -> bool {
    matches!(state, "minecraft:end_stone")
}

pub fn chorus_plant_can_start(origin_empty: bool, below_state: &str) -> bool {
    origin_empty && supports_chorus_plant(below_state)
}

pub fn chorus_all_horizontal_neighbors_empty(neighbors: [bool; 4], ignored: Option<usize>) -> bool {
    neighbors
        .into_iter()
        .enumerate()
        .all(|(index, empty)| ignored == Some(index) || empty)
}

pub fn chorus_branch_target_within_spread(
    target: BlockPos,
    start: BlockPos,
    max_horizontal_spread: i32,
) -> bool {
    (target.x - start.x).abs() < max_horizontal_spread
        && (target.z - start.z).abs() < max_horizontal_spread
}

pub fn chorus_trunk_height(depth: i32, height_roll: i32) -> i32 {
    let mut height = height_roll.rem_euclid(4) + 1;
    if depth == 0 {
        height += 1;
    }
    height
}

pub fn chorus_stem_attempts(depth: i32, stem_roll: i32) -> i32 {
    let mut stems = stem_roll.rem_euclid(4);
    if depth == 0 {
        stems += 1;
    }
    stems
}

pub fn chorus_trunk_and_terminal_flower(
    current: BlockPos,
    depth: i32,
    height_roll: i32,
    placed_stem: bool,
) -> Vec<ChorusPlantPlacementBlock> {
    let height = chorus_trunk_height(depth, height_roll);
    let mut blocks = Vec::new();
    blocks.push(ChorusPlantPlacementBlock {
        pos: current,
        kind: ChorusPlantPlacementKind::Plant,
        age: None,
    });
    for i in 0..height {
        blocks.push(ChorusPlantPlacementBlock {
            pos: BlockPos {
                x: current.x,
                y: current.y + i + 1,
                z: current.z,
            },
            kind: ChorusPlantPlacementKind::Plant,
            age: None,
        });
    }
    if !placed_stem {
        blocks.push(ChorusPlantPlacementBlock {
            pos: BlockPos {
                x: current.x,
                y: current.y + height,
                z: current.z,
            },
            kind: ChorusPlantPlacementKind::Flower,
            age: Some(5),
        });
    }
    blocks
}

pub fn end_podium_location(offset: BlockPos) -> BlockPos {
    offset
}

pub fn end_podium_inside_rim(pos: BlockPos, origin: BlockPos) -> bool {
    let dx = pos.x - origin.x;
    let dy = pos.y - origin.y;
    let dz = pos.z - origin.z;
    ((dx * dx + dy * dy + dz * dz) as f64) < 2.5_f64.powi(2)
}

pub fn end_podium_inside_body(pos: BlockPos, origin: BlockPos) -> bool {
    let dx = pos.x - origin.x;
    let dy = pos.y - origin.y;
    let dz = pos.z - origin.z;
    ((dx * dx + dy * dy + dz * dz) as f64) < 3.5_f64.powi(2)
}

pub fn end_podium_blocks(origin: BlockPos, active: bool) -> Vec<EndPodiumPlacementBlock> {
    let mut blocks = Vec::new();
    for y in (origin.y - 1)..=(origin.y + 32) {
        for z in (origin.z - 4)..=(origin.z + 4) {
            for x in (origin.x - 4)..=(origin.x + 4) {
                let pos = BlockPos { x, y, z };
                let inside_rim = end_podium_inside_rim(pos, origin);
                if !inside_rim && !end_podium_inside_body(pos, origin) {
                    continue;
                }
                let kind = if y < origin.y {
                    if inside_rim {
                        EndPodiumBlockKind::Bedrock
                    } else {
                        EndPodiumBlockKind::EndStone
                    }
                } else if y > origin.y {
                    EndPodiumBlockKind::Air
                } else if !inside_rim {
                    EndPodiumBlockKind::Bedrock
                } else if active {
                    EndPodiumBlockKind::EndPortal
                } else {
                    EndPodiumBlockKind::Air
                };
                blocks.push(EndPodiumPlacementBlock { pos, kind });
            }
        }
    }
    for y in 0..4 {
        blocks.push(EndPodiumPlacementBlock {
            pos: BlockPos {
                x: origin.x,
                y: origin.y + y,
                z: origin.z,
            },
            kind: EndPodiumBlockKind::Bedrock,
        });
    }
    let torch_y = origin.y + 2;
    for direction in [
        HorizontalDirection::North,
        HorizontalDirection::South,
        HorizontalDirection::West,
        HorizontalDirection::East,
    ] {
        let pos = offset_horizontal(
            BlockPos {
                x: origin.x,
                y: torch_y,
                z: origin.z,
            },
            direction,
            1,
        );
        blocks.push(EndPodiumPlacementBlock {
            pos,
            kind: EndPodiumBlockKind::WallTorch(direction),
        });
    }
    blocks
}

pub fn end_spike_from_size(index: i32, size: i32) -> EndSpikeModel {
    let angle = 2.0 * (-std::f64::consts::PI + (std::f64::consts::PI / 10.0) * index as f64);
    EndSpikeModel {
        center_x: (42.0 * angle.cos()).floor() as i32,
        center_z: (42.0 * angle.sin()).floor() as i32,
        radius: 2 + size / 3,
        height: 76 + size * 3,
        guarded: size == 1 || size == 2,
    }
}

pub fn end_spike_is_center_within_chunk(spike: EndSpikeModel, chunk_origin: BlockPos) -> bool {
    chunk_origin.x.div_euclid(16) == spike.center_x.div_euclid(16)
        && chunk_origin.z.div_euclid(16) == spike.center_z.div_euclid(16)
}

pub fn end_spike_top_bounding_box(
    spike: EndSpikeModel,
    min_y: i32,
    max_y: i32,
) -> (BlockPos, BlockPos) {
    (
        BlockPos {
            x: spike.center_x - spike.radius,
            y: min_y,
            z: spike.center_z - spike.radius,
        },
        BlockPos {
            x: spike.center_x + spike.radius,
            y: max_y,
            z: spike.center_z + spike.radius,
        },
    )
}

pub fn end_spike_cylinder_and_air_blocks(
    spike: EndSpikeModel,
    min_y: i32,
) -> Vec<EndSpikePlacementBlock> {
    let mut blocks = Vec::new();
    for y in min_y..=(spike.height + 10) {
        for z in (spike.center_z - spike.radius)..=(spike.center_z + spike.radius) {
            for x in (spike.center_x - spike.radius)..=(spike.center_x + spike.radius) {
                let dx = x - spike.center_x;
                let dz = z - spike.center_z;
                if dx * dx + dz * dz <= spike.radius * spike.radius + 1 && y < spike.height {
                    blocks.push(EndSpikePlacementBlock {
                        pos: BlockPos { x, y, z },
                        kind: EndSpikeBlockKind::Obsidian,
                    });
                } else if y > 65 {
                    blocks.push(EndSpikePlacementBlock {
                        pos: BlockPos { x, y, z },
                        kind: EndSpikeBlockKind::Air,
                    });
                }
            }
        }
    }
    blocks
}

pub fn end_spike_guard_cage_blocks(spike: EndSpikeModel) -> Vec<EndSpikePlacementBlock> {
    if !spike.guarded {
        return Vec::new();
    }
    let mut blocks = Vec::new();
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            for dy in 0i32..=3 {
                let x_side = dx.abs() == 2;
                let z_side = dz.abs() == 2;
                let top = dy == 3;
                if !x_side && !z_side && !top {
                    continue;
                }
                let x_edge = dx == -2 || dx == 2 || top;
                let z_edge = dz == -2 || dz == 2 || top;
                blocks.push(EndSpikePlacementBlock {
                    pos: BlockPos {
                        x: spike.center_x + dx,
                        y: spike.height + dy,
                        z: spike.center_z + dz,
                    },
                    kind: EndSpikeBlockKind::IronBars {
                        north: x_edge && dz != -2,
                        south: x_edge && dz != 2,
                        west: z_edge && dx != -2,
                        east: z_edge && dx != 2,
                    },
                });
            }
        }
    }
    blocks
}

pub fn end_crystal_for_spike(
    spike: EndSpikeModel,
    config: &EndSpikeConfigurationModel,
    yaw_roll: f32,
) -> EndCrystalPlacement {
    EndCrystalPlacement {
        x: spike.center_x as f64 + 0.5,
        y: spike.height as f64 + 1.0,
        z: spike.center_z as f64 + 0.5,
        yaw: yaw_roll * 360.0,
        beam_target: config.crystal_beam_target,
        invulnerable: config.crystal_invulnerable,
    }
}

pub fn end_spike_crystal_support_blocks(spike: EndSpikeModel) -> Vec<EndSpikePlacementBlock> {
    let crystal_block_pos = BlockPos {
        x: spike.center_x,
        y: spike.height + 1,
        z: spike.center_z,
    };
    vec![
        EndSpikePlacementBlock {
            pos: BlockPos {
                x: crystal_block_pos.x,
                y: crystal_block_pos.y - 1,
                z: crystal_block_pos.z,
            },
            kind: EndSpikeBlockKind::Bedrock,
        },
        EndSpikePlacementBlock {
            pos: crystal_block_pos,
            kind: EndSpikeBlockKind::Fire,
        },
    ]
}

pub fn huge_fungus_can_start(config: &HugeFungusConfigurationModel, below_state: &str) -> bool {
    below_state == config.valid_base_state
}

pub fn huge_fungus_total_height(height_roll: i32, double_roll: i32) -> i32 {
    let mut total_height = 4 + height_roll.rem_euclid(10);
    if double_roll.rem_euclid(12) == 0 {
        total_height *= 2;
    }
    total_height
}

pub fn huge_fungus_fits_height(
    origin_y: i32,
    total_height: i32,
    gen_depth: i32,
    planted: bool,
) -> bool {
    planted || origin_y + total_height + 1 < gen_depth
}

pub fn huge_fungus_is_huge(planted: bool, roll: f32) -> bool {
    !planted && roll < 0.06
}

pub fn huge_fungus_stem_blocks(
    origin: BlockPos,
    total_height: i32,
    is_huge: bool,
    corner_rolls: &[f32],
) -> Vec<HugeFungusStemBlock> {
    let stem_radius: i32 = if is_huge { 1 } else { 0 };
    let mut blocks = Vec::new();
    let mut corner_index = 0usize;
    for dx in -stem_radius..=stem_radius {
        for dz in -stem_radius..=stem_radius {
            let corner = is_huge && dx.abs() == stem_radius && dz.abs() == stem_radius;
            let place_corner =
                !corner || corner_rolls.get(corner_index).copied().unwrap_or(1.0) < 0.1;
            if corner {
                corner_index += 1;
            }
            if !place_corner {
                continue;
            }
            for dy in 0..total_height {
                blocks.push(HugeFungusStemBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    kind: if corner {
                        HugeFungusStemKind::CornerStem
                    } else {
                        HugeFungusStemKind::Stem
                    },
                });
            }
        }
    }
    blocks
}

pub fn huge_fungus_hat_height(total_height: i32, roll: i32) -> i32 {
    (roll.rem_euclid(1 + total_height / 3) + 5).min(total_height)
}

pub fn huge_fungus_hat_radius(
    dy: i32,
    total_height: i32,
    hat_height: i32,
    is_huge: bool,
    top_roll: i32,
) -> i32 {
    let hat_start_y = total_height - hat_height;
    let mut radius = if dy < total_height - top_roll.rem_euclid(3) {
        2
    } else {
        1
    };
    if hat_height > 8 && dy < hat_start_y + 4 {
        radius = 3;
    }
    if is_huge {
        radius += 1;
    }
    radius
}

pub fn huge_fungus_hat_cells(
    origin: BlockPos,
    total_height: i32,
    hat_height_roll: i32,
    top_rolls: &[i32],
    is_huge: bool,
) -> Vec<HugeFungusHatCell> {
    let hat_height = huge_fungus_hat_height(total_height, hat_height_roll);
    let hat_start_y = total_height - hat_height;
    let mut cells = Vec::new();
    for dy in hat_start_y..=total_height {
        let radius = huge_fungus_hat_radius(
            dy,
            total_height,
            hat_height,
            is_huge,
            top_rolls
                .get((dy - hat_start_y) as usize)
                .copied()
                .unwrap_or(0),
        );
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let edge_x = dx == -radius || dx == radius;
                let edge_z = dz == -radius || dz == radius;
                let inside = !edge_x && !edge_z && dy != total_height;
                let corner = edge_x && edge_z;
                let bottom = dy < hat_start_y + 3;
                let role = if bottom {
                    HugeFungusHatRole::Bottom
                } else if inside {
                    HugeFungusHatRole::Inside
                } else if corner {
                    HugeFungusHatRole::Corner
                } else {
                    HugeFungusHatRole::Edge
                };
                cells.push(HugeFungusHatCell {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + dy,
                        z: origin.z + dz,
                    },
                    role,
                    radius,
                });
            }
        }
    }
    cells
}

pub fn huge_fungus_hat_drop_outcome(
    below_same_hat: bool,
    hat_roll: f32,
    vine_roll: i32,
    place_vines: bool,
) -> HugeFungusHatPlacement {
    if below_same_hat {
        HugeFungusHatPlacement::Hat
    } else if hat_roll < 0.15 {
        if place_vines && vine_roll.rem_euclid(11) == 0 {
            HugeFungusHatPlacement::HatWithWeepingVines
        } else {
            HugeFungusHatPlacement::Hat
        }
    } else {
        HugeFungusHatPlacement::None
    }
}

pub fn huge_fungus_hat_block_outcome(
    decor_roll: f32,
    hat_roll: f32,
    vine_roll: f32,
    decor_probability: f32,
    hat_probability: f32,
    vines_probability: f32,
) -> HugeFungusHatPlacement {
    if decor_roll < decor_probability {
        HugeFungusHatPlacement::Decor
    } else if hat_roll < hat_probability {
        if vine_roll < vines_probability {
            HugeFungusHatPlacement::HatWithWeepingVines
        } else {
            HugeFungusHatPlacement::Hat
        }
    } else {
        HugeFungusHatPlacement::None
    }
}

pub fn huge_fungus_hat_probabilities(
    role: HugeFungusHatRole,
    place_vines: bool,
) -> Option<(f32, f32, f32)> {
    match role {
        HugeFungusHatRole::Bottom => None,
        HugeFungusHatRole::Inside => Some((0.1, 0.2, if place_vines { 0.1 } else { 0.0 })),
        HugeFungusHatRole::Corner => Some((0.01, 0.7, if place_vines { 0.083 } else { 0.0 })),
        HugeFungusHatRole::Edge => Some((0.0005, 0.98, if place_vines { 0.07 } else { 0.0 })),
    }
}

pub fn huge_fungus_weeping_vine_height(base_roll: i32, double_roll: i32) -> i32 {
    let mut height = 1 + base_roll.rem_euclid(5);
    if double_roll.rem_euclid(7) == 0 {
        height *= 2;
    }
    height
}

fn block_is_coral(block: &str) -> bool {
    block.contains("_coral")
}

fn offset_horizontal(pos: BlockPos, direction: HorizontalDirection, distance: i32) -> BlockPos {
    match direction {
        HorizontalDirection::North => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z - distance,
        },
        HorizontalDirection::South => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z + distance,
        },
        HorizontalDirection::West => BlockPos {
            x: pos.x - distance,
            y: pos.y,
            z: pos.z,
        },
        HorizontalDirection::East => BlockPos {
            x: pos.x + distance,
            y: pos.y,
            z: pos.z,
        },
    }
}

impl HorizontalDirection {
    const fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

impl CaveSurface {
    const fn y_step(self) -> i32 {
        match self {
            Self::Floor => -1,
            Self::Ceiling => 1,
        }
    }

    const fn opposite(self) -> Self {
        match self {
            Self::Floor => Self::Ceiling,
            Self::Ceiling => Self::Floor,
        }
    }
}

fn offset_vertical(pos: BlockPos, surface: CaveSurface, distance: i32) -> BlockPos {
    BlockPos {
        x: pos.x,
        y: pos.y + surface.y_step() * distance,
        z: pos.z,
    }
}

const fn coral_wall_fan_state(direction: HorizontalDirection) -> &'static str {
    match direction {
        HorizontalDirection::North => "minecraft:tube_coral_wall_fan[facing=north]",
        HorizontalDirection::South => "minecraft:tube_coral_wall_fan[facing=south]",
        HorizontalDirection::West => "minecraft:tube_coral_wall_fan[facing=west]",
        HorizontalDirection::East => "minecraft:tube_coral_wall_fan[facing=east]",
    }
}

pub fn block_pile_placement_candidates(
    origin: BlockPos,
    min_y: i32,
    x_radius_roll: i32,
    z_radius_roll: i32,
    shape_rolls: &[(f32, f32, f32)],
) -> Vec<BlockPos> {
    if origin.y < min_y + 5 {
        return Vec::new();
    }
    let x_radius = 2 + x_radius_roll.rem_euclid(2);
    let z_radius = 2 + z_radius_roll.rem_euclid(2);
    let mut positions = Vec::new();
    let mut roll_index = 0;
    for y_offset in 0..=1 {
        for z in origin.z - z_radius..=origin.z + z_radius {
            for x in origin.x - x_radius..=origin.x + x_radius {
                let (first, second, sparse) = shape_rolls
                    .get(roll_index)
                    .copied()
                    .unwrap_or((0.0, 0.0, 1.0));
                roll_index += 1;
                let dx = origin.x - x;
                let dz = origin.z - z;
                let in_blob = (dx * dx + dz * dz) as f32 <= first * 10.0 - second * 6.0;
                if in_blob || sparse < 0.031 {
                    positions.push(BlockPos {
                        x,
                        y: origin.y + y_offset,
                        z,
                    });
                }
            }
        }
    }
    positions
}

pub fn block_pile_try_place(
    config: &BlockPileConfigurationModel,
    candidate_empty: bool,
    below_block: &'static str,
    below_sturdy: bool,
    dirt_path_random: bool,
    provider_roll: i32,
) -> Option<&'static str> {
    if !candidate_empty {
        return None;
    }
    let may_place = if below_block == "minecraft:dirt_path" {
        dirt_path_random
    } else {
        below_sturdy
    };
    if may_place {
        block_state_provider_sample(&config.state_provider, provider_roll)
    } else {
        None
    }
}

pub fn disk_placement_plan(
    origin: BlockPos,
    config: &DiskConfigurationModel,
    column_contexts: &[(BlockPos, BlockPredicateContext)],
    provider_rolls: &[i32],
) -> Vec<DiskPlacementBlock> {
    let radius = config.radius.clamp(0, 8);
    let half_height = config.half_height.clamp(0, 4);
    let top = origin.y + half_height;
    let bottom_exclusive = origin.y - half_height - 1;
    let mut placed = Vec::new();
    let mut provider_index = 0;
    for z in origin.z - radius..=origin.z + radius {
        for x in origin.x - radius..=origin.x + radius {
            let dx = x - origin.x;
            let dz = z - origin.z;
            if dx * dx + dz * dz > radius * radius {
                continue;
            }
            let mut placed_above = false;
            for y in (bottom_exclusive + 1..=top).rev() {
                let pos = BlockPos { x, y, z };
                let Some((_, context)) = column_contexts
                    .iter()
                    .find(|(context_pos, _)| *context_pos == pos)
                else {
                    placed_above = false;
                    continue;
                };
                if block_predicate_test(config.target, *context, y) {
                    let provider_roll = provider_rolls.get(provider_index).copied().unwrap_or(0);
                    provider_index += 1;
                    if let Some(state) =
                        block_state_provider_sample(&config.state_provider, provider_roll)
                    {
                        placed.push(DiskPlacementBlock {
                            pos,
                            state,
                            mark_above_for_post_processing: !placed_above,
                        });
                        placed_above = true;
                    }
                } else {
                    placed_above = false;
                }
            }
        }
    }
    placed
}

pub fn snow_and_freeze_placement_plan(
    origin: BlockPos,
    columns: &[SnowAndFreezeColumn],
) -> Vec<SnowAndFreezePlacement> {
    let mut placements = Vec::new();
    for dx in 0..16 {
        for dz in 0..16 {
            let x = origin.x + dx;
            let z = origin.z + dz;
            let Some(column) = columns.iter().find(|column| column.x == x && column.z == z) else {
                continue;
            };
            let top = BlockPos {
                x,
                y: column.motion_blocking_height,
                z,
            };
            let below = BlockPos {
                x,
                y: column.motion_blocking_height - 1,
                z,
            };
            if column.should_freeze {
                placements.push(SnowAndFreezePlacement {
                    pos: below,
                    state: "minecraft:ice",
                });
            }
            if column.should_snow {
                placements.push(SnowAndFreezePlacement {
                    pos: top,
                    state: "minecraft:snow",
                });
                if column.below_has_snowy_property {
                    placements.push(SnowAndFreezePlacement {
                        pos: below,
                        state: "minecraft:snowy=true",
                    });
                }
            }
        }
    }
    placements
}

pub fn underwater_magma_placement_plan(
    origin: BlockPos,
    floor_y: Option<i32>,
    config: UnderwaterMagmaConfigurationModel,
    candidates: &[UnderwaterMagmaCandidate],
    probability_rolls: &[f32],
) -> Vec<BlockPos> {
    let Some(floor_y) = floor_y else {
        return Vec::new();
    };
    let radius = config.placement_radius_around_floor.clamp(0, 64);
    let probability = config
        .placement_probability_per_valid_position
        .clamp(0.0, 1.0);
    let floor_pos = BlockPos {
        x: origin.x,
        y: floor_y,
        z: origin.z,
    };
    let mut placements = Vec::new();
    let mut roll_index = 0;
    for y in floor_pos.y - radius..=floor_pos.y + radius {
        for z in floor_pos.z - radius..=floor_pos.z + radius {
            for x in floor_pos.x - radius..=floor_pos.x + radius {
                let roll = probability_rolls.get(roll_index).copied().unwrap_or(0.0);
                roll_index += 1;
                if roll >= probability {
                    continue;
                }
                let pos = BlockPos { x, y, z };
                if candidates
                    .iter()
                    .find(|candidate| candidate.pos == pos)
                    .is_some_and(underwater_magma_is_valid_placement)
                {
                    placements.push(pos);
                }
            }
        }
    }
    placements
}

pub fn underwater_magma_is_valid_placement(candidate: &UnderwaterMagmaCandidate) -> bool {
    !matches!(candidate.block, "minecraft:water" | "minecraft:air")
        && !candidate.below_visible_from_above
        && !candidate.horizontal_visible_from_outside
}

pub fn feature_size_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:feature_size_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn trunk_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:trunk_placer_type", id)
}

pub fn foliage_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:foliage_placer_type", id)
}

pub fn root_placer_type(id: &str) -> Option<&'static str> {
    worldgen_type_registry_entry("minecraft:root_placer_type", id)
}

fn worldgen_type_registry_entry(registry_id: &str, id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == registry_id)?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn validate_trunk_placer(placer: TrunkPlacerModel) -> Result<TrunkPlacerModel, String> {
    if !(0..=32).contains(&placer.base_height)
        || !(0..=24).contains(&placer.height_rand_a)
        || !(0..=24).contains(&placer.height_rand_b)
    {
        return Err("trunk placer base fields are outside vanilla codec ranges".to_string());
    }

    let valid = match placer.kind {
        TrunkPlacerKind::Straight
        | TrunkPlacerKind::Forking
        | TrunkPlacerKind::Giant
        | TrunkPlacerKind::MegaJungle
        | TrunkPlacerKind::DarkOak
        | TrunkPlacerKind::Fancy => true,
        TrunkPlacerKind::Bending {
            min_height_for_leaves,
            bend_length_min,
            bend_length_max,
        } => {
            min_height_for_leaves > 0
                && (1..=64).contains(&bend_length_min)
                && (1..=64).contains(&bend_length_max)
                && bend_length_min <= bend_length_max
        }
        TrunkPlacerKind::UpwardsBranching {
            place_branch_per_log_probability,
            extra_branch_steps_min,
            extra_branch_length_min,
        } => {
            (0.0..=1.0).contains(&place_branch_per_log_probability)
                && extra_branch_steps_min > 0
                && extra_branch_length_min >= 0
        }
        TrunkPlacerKind::Cherry {
            branch_count_min,
            branch_count_max,
            branch_horizontal_length_min,
            branch_horizontal_length_max,
            branch_start_offset_from_top_min,
            branch_start_offset_from_top_max,
            branch_end_offset_from_top_min,
            branch_end_offset_from_top_max,
        } => {
            (1..=3).contains(&branch_count_min)
                && (1..=3).contains(&branch_count_max)
                && branch_count_min <= branch_count_max
                && (2..=16).contains(&branch_horizontal_length_min)
                && (2..=16).contains(&branch_horizontal_length_max)
                && branch_horizontal_length_min <= branch_horizontal_length_max
                && (-16..=0).contains(&branch_start_offset_from_top_min)
                && (-16..=0).contains(&branch_start_offset_from_top_max)
                && branch_start_offset_from_top_min <= branch_start_offset_from_top_max
                && branch_start_offset_from_top_max - branch_start_offset_from_top_min >= 1
                && (-16..=16).contains(&branch_end_offset_from_top_min)
                && (-16..=16).contains(&branch_end_offset_from_top_max)
                && branch_end_offset_from_top_min <= branch_end_offset_from_top_max
        }
    };
    if valid {
        Ok(placer)
    } else {
        Err("trunk placer variant fields are outside vanilla codec ranges".to_string())
    }
}

pub fn trunk_placer_height(placer: TrunkPlacerModel, rand_a: i32, rand_b: i32) -> i32 {
    placer.base_height
        + rand_a.rem_euclid(placer.height_rand_a + 1)
        + rand_b.rem_euclid(placer.height_rand_b + 1)
}

pub fn tree_valid_pos(state: &str) -> bool {
    matches!(
        state,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:vine"
            | "minecraft:water"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:snow"
            | "minecraft:dandelion"
            | "minecraft:poppy"
            | "minecraft:blue_orchid"
            | "minecraft:allium"
            | "minecraft:azure_bluet"
            | "minecraft:orange_tulip"
            | "minecraft:pink_tulip"
            | "minecraft:red_tulip"
            | "minecraft:white_tulip"
            | "minecraft:oxeye_daisy"
            | "minecraft:cornflower"
            | "minecraft:lily_of_the_valley"
            | "minecraft:wither_rose"
    ) || state.ends_with("_leaves")
        || state.ends_with("_sapling")
        || state.ends_with("_flower")
}

pub fn tree_max_free_height(
    tree_height: i32,
    min_size: FeatureSizeModel,
    rows: &[&[&str]],
    ignore_vines: bool,
) -> i32 {
    for y in 0..=tree_height + 1 {
        let radius = feature_size_at_height(min_size, tree_height, y);
        let radius_width = radius * 2 + 1;
        let expected_width = (radius_width * radius_width).max(0) as usize;
        let row = rows.get(y as usize).copied().unwrap_or(&[]);
        if row.len() < expected_width
            || row.iter().take(expected_width).any(|state| {
                !tree_valid_pos(state) || (!ignore_vines && *state == "minecraft:vine")
            })
        {
            return y - 2;
        }
    }
    tree_height
}

pub fn tree_can_place(
    origin: BlockPos,
    trunk_origin: BlockPos,
    tree_height: i32,
    min_size: FeatureSizeModel,
    min_clipped_height: Option<i32>,
    build_min_y: i32,
    build_max_y: i32,
    rows: &[&[&str]],
    ignore_vines: bool,
) -> bool {
    let min_y = origin.y.min(trunk_origin.y);
    let max_y = origin.y.max(trunk_origin.y) + tree_height + 1;
    if min_y < build_min_y + 1 || max_y > build_max_y + 1 {
        return false;
    }
    let clipped_tree_height = tree_max_free_height(tree_height, min_size, rows, ignore_vines);
    clipped_tree_height >= tree_height
        || min_clipped_height.is_some_and(|min| clipped_tree_height >= min)
}

pub fn validate_foliage_placer(placer: FoliagePlacerModel) -> Result<FoliagePlacerModel, String> {
    if !(0..=16).contains(&placer.radius_min)
        || !(0..=16).contains(&placer.radius_max)
        || placer.radius_min > placer.radius_max
        || !(0..=16).contains(&placer.offset_min)
        || !(0..=16).contains(&placer.offset_max)
        || placer.offset_min > placer.offset_max
    {
        return Err("foliage placer base providers are outside vanilla codec ranges".to_string());
    }

    let valid = match placer.kind {
        FoliagePlacerKind::Blob { height }
        | FoliagePlacerKind::Bush { height }
        | FoliagePlacerKind::Fancy { height }
        | FoliagePlacerKind::Jungle { height }
        | FoliagePlacerKind::Cherry { height, .. } => (0..=16).contains(&height),
        FoliagePlacerKind::Spruce {
            height_min,
            height_max,
        }
        | FoliagePlacerKind::Pine {
            height_min,
            height_max,
        }
        | FoliagePlacerKind::MegaPine {
            height_min,
            height_max,
        } => {
            (0..=16).contains(&height_min)
                && (0..=16).contains(&height_max)
                && height_min <= height_max
        }
        FoliagePlacerKind::Acacia | FoliagePlacerKind::DarkOak => true,
        FoliagePlacerKind::RandomSpread {
            foliage_height_min,
            foliage_height_max,
            leaf_placement_attempts,
        } => {
            (1..=512).contains(&foliage_height_min)
                && (1..=512).contains(&foliage_height_max)
                && foliage_height_min <= foliage_height_max
                && (0..=256).contains(&leaf_placement_attempts)
        }
    };

    let chance_valid = match placer.kind {
        FoliagePlacerKind::Cherry {
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
            ..
        } => [
            wide_bottom_layer_hole_chance,
            corner_hole_chance,
            hanging_leaves_chance,
            hanging_leaves_extension_chance,
        ]
        .into_iter()
        .all(|chance| (0.0..=1.0).contains(&chance)),
        _ => true,
    };

    if valid && chance_valid {
        Ok(placer)
    } else {
        Err("foliage placer variant fields are outside vanilla codec ranges".to_string())
    }
}

pub fn validate_root_placer(placer: RootPlacerModel) -> Result<RootPlacerModel, String> {
    if placer
        .above_root_placement_chance
        .is_some_and(|chance| !(0.0..=1.0).contains(&chance))
        || !(1..=12).contains(&placer.mangrove_root_placement.max_root_width)
        || !(1..=64).contains(&placer.mangrove_root_placement.max_root_length)
        || !(0.0..=1.0).contains(&placer.mangrove_root_placement.random_skew_chance)
    {
        Err("root placer fields are outside vanilla codec ranges".to_string())
    } else {
        Ok(placer)
    }
}

pub fn simple_tree_placement_plan(
    origin: BlockPos,
    trunk: TrunkPlacerModel,
    foliage: FoliagePlacerModel,
    trunk_state: &'static str,
    foliage_state: &'static str,
    below_trunk_state: &'static str,
    rand_a: i32,
    rand_b: i32,
) -> Result<TreePlacementPlan, String> {
    validate_trunk_placer(trunk)?;
    validate_foliage_placer(foliage)?;
    if trunk.kind != TrunkPlacerKind::Straight {
        return Err(
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string(),
        );
    }

    let (foliage_height, bush_shape) = match foliage.kind {
        FoliagePlacerKind::Blob { height } => (height, false),
        FoliagePlacerKind::Bush { height } => (height, true),
        _ => {
            return Err(
                "only blob and bush foliage placement is modeled by simple_tree_placement_plan"
                    .to_string(),
            )
        }
    };

    let tree_height = trunk_placer_height(trunk, rand_a, rand_b);
    let leaf_radius = foliage.radius_min;
    let mut blocks = Vec::new();
    push_tree_block(
        &mut blocks,
        TreePlacementBlock {
            pos: BlockPos {
                x: origin.x,
                y: origin.y - 1,
                z: origin.z,
            },
            state: below_trunk_state,
            kind: TreePlacementBlockKind::DirtBelowTrunk,
        },
    );
    for y in 0..tree_height {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x,
                    y: origin.y + y,
                    z: origin.z,
                },
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }

    let foliage_origin = BlockPos {
        x: origin.x,
        y: origin.y + tree_height,
        z: origin.z,
    };
    for yo in (0..=foliage_height).rev() {
        let current_radius = if bush_shape {
            leaf_radius - 1 - yo
        } else {
            (leaf_radius - 1 - yo / 2).max(0)
        };
        if current_radius < 0 {
            continue;
        }
        place_simple_leaves_row(
            &mut blocks,
            foliage_origin,
            current_radius,
            -yo,
            foliage_state,
        );
    }

    Ok(TreePlacementPlan { blocks })
}

pub fn fallen_tree_log_length(min_length: i32, max_length: i32, sample_roll: i32) -> i32 {
    let span = (max_length - min_length + 1).max(1);
    min_length + sample_roll.rem_euclid(span) - 2
}

pub fn fallen_tree_start_pos(
    origin: BlockPos,
    direction: HorizontalDirection,
    distance_roll: i32,
    ground_probe: &[bool],
) -> Option<BlockPos> {
    let mut pos = offset_horizontal(origin, direction, 2 + distance_roll.rem_euclid(2));
    pos.y += 1;
    for can_place in ground_probe.iter().copied().take(6) {
        if can_place {
            return Some(pos);
        }
        pos.y -= 1;
    }
    None
}

pub fn fallen_tree_can_place_log(
    valid_tree_positions: &[bool],
    over_solid_ground: &[bool],
) -> bool {
    let mut ground_gap = 0;
    for (index, valid) in valid_tree_positions.iter().copied().enumerate() {
        if !valid {
            return false;
        }
        if !over_solid_ground.get(index).copied().unwrap_or(false) {
            ground_gap += 1;
            if ground_gap > 2 {
                return false;
            }
        } else {
            ground_gap = 0;
        }
    }
    true
}

pub fn fallen_tree_placement_plan(
    origin: BlockPos,
    config: &FallenTreeConfigurationModel,
    direction: HorizontalDirection,
    log_length_roll: i32,
    distance_roll: i32,
    ground_probe: &[bool],
    valid_tree_positions: &[bool],
    over_solid_ground: &[bool],
) -> Option<FallenTreePlacementPlan> {
    let trunk_state = block_state_provider_sample(&config.trunk_provider, 0)?;
    let log_length = fallen_tree_log_length(
        config.min_log_length,
        config.max_log_length,
        log_length_roll,
    );
    let start = fallen_tree_start_pos(origin, direction, distance_roll, ground_probe)?;
    let valid_len = log_length.max(0) as usize;
    if valid_tree_positions.len() < valid_len || over_solid_ground.len() < valid_len {
        return None;
    }
    if !fallen_tree_can_place_log(
        &valid_tree_positions[..valid_len],
        &over_solid_ground[..valid_len],
    ) {
        return None;
    }
    let mut blocks = vec![FallenTreeBlock {
        pos: origin,
        state: trunk_state,
        mark_above_for_post_processing: true,
    }];
    for i in 0..log_length.max(0) {
        blocks.push(FallenTreeBlock {
            pos: offset_horizontal(start, direction, i),
            state: rotated_log_state(trunk_state, direction),
            mark_above_for_post_processing: true,
        });
    }
    Some(FallenTreePlacementPlan {
        blocks,
        stump_decorators: config.stump_decorators.len(),
        log_decorators: config.log_decorators.len(),
    })
}

fn rotated_log_state(state: &'static str, direction: HorizontalDirection) -> &'static str {
    match (state, direction) {
        ("minecraft:oak_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:oak_log[axis=x]"
        }
        ("minecraft:oak_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:oak_log[axis=z]"
        }
        ("minecraft:birch_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:birch_log[axis=x]"
        }
        ("minecraft:birch_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:birch_log[axis=z]"
        }
        ("minecraft:spruce_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:spruce_log[axis=x]"
        }
        ("minecraft:spruce_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:spruce_log[axis=z]"
        }
        ("minecraft:jungle_log", HorizontalDirection::East | HorizontalDirection::West) => {
            "minecraft:jungle_log[axis=x]"
        }
        ("minecraft:jungle_log", HorizontalDirection::North | HorizontalDirection::South) => {
            "minecraft:jungle_log[axis=z]"
        }
        _ => state,
    }
}

fn place_simple_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

fn push_tree_block(blocks: &mut Vec<TreePlacementBlock>, block: TreePlacementBlock) {
    if !blocks.iter().any(|existing| existing.pos == block.pos) {
        blocks.push(block);
    }
}

pub fn validate_feature_size(size: FeatureSizeModel) -> Result<FeatureSizeModel, String> {
    let min_clipped_height = match size {
        FeatureSizeModel::TwoLayers {
            min_clipped_height, ..
        }
        | FeatureSizeModel::ThreeLayers {
            min_clipped_height, ..
        } => min_clipped_height,
    };
    if min_clipped_height.is_some_and(|height| !(0..=80).contains(&height)) {
        return Err("min_clipped_height must be in 0..=80".to_string());
    }
    let valid = match size {
        FeatureSizeModel::TwoLayers {
            limit,
            lower_size,
            upper_size,
            ..
        } => {
            (0..=81).contains(&limit)
                && (0..=16).contains(&lower_size)
                && (0..=16).contains(&upper_size)
        }
        FeatureSizeModel::ThreeLayers {
            limit,
            upper_limit,
            lower_size,
            middle_size,
            upper_size,
            ..
        } => {
            (0..=80).contains(&limit)
                && (0..=80).contains(&upper_limit)
                && (0..=16).contains(&lower_size)
                && (0..=16).contains(&middle_size)
                && (0..=16).contains(&upper_size)
        }
    };
    if valid {
        Ok(size)
    } else {
        Err("feature size fields are outside vanilla codec ranges".to_string())
    }
}

pub fn feature_size_at_height(size: FeatureSizeModel, tree_height: i32, yo: i32) -> i32 {
    match size {
        FeatureSizeModel::TwoLayers {
            limit,
            lower_size,
            upper_size,
            ..
        } => {
            if yo < limit {
                lower_size
            } else {
                upper_size
            }
        }
        FeatureSizeModel::ThreeLayers {
            limit,
            upper_limit,
            lower_size,
            middle_size,
            upper_size,
            ..
        } => {
            if yo < limit {
                lower_size
            } else if yo >= tree_height - upper_limit {
                upper_size
            } else {
                middle_size
            }
        }
    }
}

pub fn tree_decorator_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:tree_decorator_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn validate_tree_decorator(
    decorator: TreeDecoratorModel,
) -> Result<TreeDecoratorModel, String> {
    let probability = match decorator {
        TreeDecoratorModel::Cocoa { probability }
        | TreeDecoratorModel::Beehive { probability }
        | TreeDecoratorModel::AttachedToLeaves { probability }
        | TreeDecoratorModel::AttachedToLogs { probability } => Some(probability),
        TreeDecoratorModel::TrunkVine
        | TreeDecoratorModel::LeaveVine
        | TreeDecoratorModel::PaleMoss
        | TreeDecoratorModel::CreakingHeart
        | TreeDecoratorModel::AlterGround
        | TreeDecoratorModel::PlaceOnGround => None,
    };
    if probability.is_some_and(|probability| !(0.0..=1.0).contains(&probability)) {
        Err("tree decorator probability must be in 0.0..=1.0".to_string())
    } else {
        Ok(decorator)
    }
}

pub fn tree_decorator_should_place(probability: f32, random_next_float: f32) -> bool {
    random_next_float < probability
}

pub fn spring_feature_can_place(
    valid_above: bool,
    requires_block_below: bool,
    valid_below: bool,
    current_is_air_or_valid: bool,
    adjacent_rock_count: i32,
    adjacent_hole_count: i32,
    required_rock_count: i32,
    required_hole_count: i32,
) -> bool {
    valid_above
        && (!requires_block_below || valid_below)
        && current_is_air_or_valid
        && adjacent_rock_count == required_rock_count
        && adjacent_hole_count == required_hole_count
}

pub fn spring_placement_plan(
    config: &SpringConfigurationModel,
    context: SpringPlacementContext,
) -> Option<SpringPlacementPlan> {
    let valid_above = config.valid_blocks.contains(&context.above_block);
    let valid_below = config.valid_blocks.contains(&context.below_block);
    let current_is_air_or_valid = context.current_block == "minecraft:air"
        || config.valid_blocks.contains(&context.current_block);
    let adjacent_blocks = [
        context.west_block,
        context.east_block,
        context.north_block,
        context.south_block,
        context.below_block,
    ];
    let adjacent_rock_count = adjacent_blocks
        .iter()
        .filter(|block| config.valid_blocks.contains(block))
        .count() as i32;
    let adjacent_hole_count = adjacent_blocks
        .iter()
        .filter(|block| **block == "minecraft:air")
        .count() as i32;
    spring_feature_can_place(
        valid_above,
        config.requires_block_below,
        valid_below,
        current_is_air_or_valid,
        adjacent_rock_count,
        adjacent_hole_count,
        config.rock_count,
        config.hole_count,
    )
    .then_some(SpringPlacementPlan {
        pos: context.origin,
        state: config.state,
        schedule_tick: true,
    })
}

pub fn monster_room_opening_count_is_valid(openings: i32) -> bool {
    openings >= MONSTER_ROOM_BOUNDS.min_openings && openings <= MONSTER_ROOM_BOUNDS.max_openings
}

pub fn monster_room_radii(x_roll: i32, z_roll: i32) -> MonsterRoomRadii {
    MonsterRoomRadii {
        x_radius: x_roll.rem_euclid(2) + 2,
        z_radius: z_roll.rem_euclid(2) + 2,
    }
}

pub fn monster_room_bounds_for_radius(radius: i32) -> (i32, i32) {
    (-radius - 1, radius + 1)
}

pub fn monster_room_opening_count(
    radii: MonsterRoomRadii,
    probes: &[MonsterRoomProbe],
) -> Option<i32> {
    let (min_x, max_x) = monster_room_bounds_for_radius(radii.x_radius);
    let (min_z, max_z) = monster_room_bounds_for_radius(radii.z_radius);
    let mut openings = 0;
    for probe in probes {
        if probe.dy == MONSTER_ROOM_BOUNDS.min_y && !probe.solid {
            return None;
        }
        if probe.dy == MONSTER_ROOM_BOUNDS.max_y && !probe.solid {
            return None;
        }
        if (probe.dx == min_x || probe.dx == max_x || probe.dz == min_z || probe.dz == max_z)
            && probe.dy == 0
            && probe.empty
            && probe.above_empty
        {
            openings += 1;
        }
    }
    Some(openings)
}

pub fn monster_room_can_place(radii: MonsterRoomRadii, probes: &[MonsterRoomProbe]) -> bool {
    monster_room_opening_count(radii, probes).is_some_and(monster_room_opening_count_is_valid)
}

pub fn monster_room_shell_state(
    dx: i32,
    dy: i32,
    dz: i32,
    radii: MonsterRoomRadii,
    y: i32,
    below_solid: bool,
    current_solid: bool,
    current_is_chest: bool,
    mossy_roll: i32,
) -> Option<&'static str> {
    let (min_x, max_x) = monster_room_bounds_for_radius(radii.x_radius);
    let (min_z, max_z) = monster_room_bounds_for_radius(radii.z_radius);
    let boundary = dx == min_x
        || dy == MONSTER_ROOM_BOUNDS.min_y
        || dz == min_z
        || dx == max_x
        || dy == MONSTER_ROOM_BOUNDS.max_y
        || dz == max_z;
    if boundary {
        if y >= 0 && !below_solid {
            Some("minecraft:cave_air")
        } else if current_solid && !current_is_chest {
            if dy == MONSTER_ROOM_BOUNDS.min_y && mossy_roll.rem_euclid(4) != 0 {
                Some("minecraft:mossy_cobblestone")
            } else {
                Some("minecraft:cobblestone")
            }
        } else {
            None
        }
    } else if !current_is_chest {
        Some("minecraft:cave_air")
    } else {
        None
    }
}

pub fn monster_room_chest_can_place(empty: bool, horizontal_solid_neighbors: i32) -> bool {
    empty && horizontal_solid_neighbors == 1
}

pub fn monster_room_spawner_mob(mob_roll: i32) -> &'static str {
    match mob_roll.rem_euclid(4) {
        0 => "minecraft:skeleton",
        1 | 2 => "minecraft:zombie",
        _ => "minecraft:spider",
    }
}

pub fn ore_vein_sphere_is_shadowed(radius_delta: f64, dx: f64, dy: f64, dz: f64) -> bool {
    radius_delta * radius_delta > dx * dx + dy * dy + dz * dz
}

pub fn blending_height_to_offset(height: f64) -> f64 {
    let target_y = height + 0.5;
    let target_y_mod = target_y.rem_euclid(8.0);
    (32.0 * (target_y - 128.0) - 3.0 * (target_y - 120.0) * target_y_mod
        + 3.0 * target_y_mod * target_y_mod)
        / (128.0 * (32.0 - 3.0 * target_y_mod))
}

pub fn blending_smooth_alpha(distance: f64, range_cells: i32) -> f64 {
    let alpha = (distance / f64::from(range_cells + 1)).clamp(0.0, 1.0);
    3.0 * alpha * alpha - 2.0 * alpha * alpha * alpha
}

pub fn validate_blending_data_packed(data: BlendingDataPacked<'_>) -> Result<(), String> {
    match data.heights {
        Some(heights) if heights.len() != BLENDING_CELL_COLUMN_COUNT => Err(format!(
            "heights has to be of length {BLENDING_CELL_COLUMN_COUNT}"
        )),
        _ => Ok(()),
    }
}

pub fn blending_output_for_old_height(
    height: Option<f64>,
    distance_cells: Option<f64>,
) -> BlendingOutput {
    match (height, distance_cells) {
        (Some(height), Some(distance)) => BlendingOutput {
            alpha: blending_smooth_alpha(distance, BLENDING_CONSTANTS.height_blending_range_cells),
            blending_offset: blending_height_to_offset(height),
        },
        (Some(height), None) => BlendingOutput {
            alpha: 0.0,
            blending_offset: blending_height_to_offset(height),
        },
        (None, _) => BlendingOutput {
            alpha: 1.0,
            blending_offset: 0.0,
        },
    }
}

pub fn initial_spawn_position(
    debug_only_half_world: bool,
    debug_world_recreate: bool,
    is_debug: bool,
    spawn_chunk_x: i32,
    spawn_chunk_z: i32,
    generator_spawn_height: i32,
    min_y: i32,
    world_surface_height_at_chunk_center: i32,
) -> InitialSpawnKind {
    if debug_only_half_world && debug_world_recreate {
        InitialSpawnKind::DebugHalfWorld {
            x: 0,
            y: 64,
            z: -100,
        }
    } else if is_debug {
        InitialSpawnKind::DebugWorld { x: 0, y: 80, z: 0 }
    } else {
        let y = if generator_spawn_height < min_y {
            world_surface_height_at_chunk_center
        } else {
            generator_spawn_height
        };
        InitialSpawnKind::Normal {
            x: spawn_chunk_x * 16 + 8,
            y,
            z: spawn_chunk_z * 16 + 8,
        }
    }
}

pub fn initial_spawn_chunk_spiral_offsets() -> Vec<(i32, i32)> {
    let radius = SPAWN_SELECTION_CONSTANTS.initial_chunk_search_radius;
    let mut x_offset = 0;
    let mut z_offset = 0;
    let mut dx = 0;
    let mut dz = -1;
    let mut offsets = Vec::with_capacity(((radius * 2 + 1) * (radius * 2 + 1)) as usize);

    for _ in 0..(radius * 2 + 1).pow(2) {
        if x_offset >= -radius && x_offset <= radius && z_offset >= -radius && z_offset <= radius {
            offsets.push((x_offset, z_offset));
        }

        if x_offset == z_offset
            || (x_offset < 0 && x_offset == -z_offset)
            || (x_offset > 0 && x_offset == 1 - z_offset)
        {
            let old_dx = dx;
            dx = -dz;
            dz = old_dx;
        }

        x_offset += dx;
        z_offset += dz;
    }

    offsets
}

pub fn spawn_search_candidate_count(radius: i32) -> i32 {
    let side = i64::from(radius.max(0)) * 2 + 1;
    i64::from(SPAWN_SELECTION_CONSTANTS.spawn_search_absolute_max_attempts).min(side * side) as i32
}

pub fn spawn_search_coprime(candidate_count: i32) -> i32 {
    if candidate_count <= SPAWN_SELECTION_CONSTANTS.small_search_coprime_threshold {
        candidate_count - 1
    } else {
        SPAWN_SELECTION_CONSTANTS.large_search_coprime
    }
}

pub fn spawn_search_radius(respawn_radius_rule: i32, distance_to_border: i32) -> i32 {
    let mut radius = respawn_radius_rule.max(0);
    if distance_to_border < radius {
        radius = distance_to_border;
    }
    if distance_to_border <= 1 {
        radius = 1;
    }
    radius
}

pub fn spawn_search_candidate(
    spawn_x: i32,
    spawn_z: i32,
    radius: i32,
    random_offset: i32,
    candidate_index: i32,
) -> Option<(i32, i32)> {
    let candidate_count = spawn_search_candidate_count(radius);
    if candidate_index >= candidate_count {
        return None;
    }
    let side = radius.max(0) * 2 + 1;
    let value = (random_offset.rem_euclid(candidate_count)
        + spawn_search_coprime(candidate_count) * candidate_index)
        .rem_euclid(candidate_count);
    let delta_x = value % side;
    let delta_z = value / side;
    Some((spawn_x + delta_x - radius, spawn_z + delta_z - radius))
}

pub fn overworld_respawn_y(
    heights: SpawnColumnHeights,
    cave_world: bool,
    blocks_from_top_plus_one_down: &[SpawnBlockKind],
) -> Option<i32> {
    let top_y = heights.top_y;
    if top_y < heights.min_y {
        return None;
    }
    if heights.surface_y <= top_y && heights.surface_y > heights.ocean_floor_y {
        return None;
    }
    let mut y = top_y + 1;
    for block in blocks_from_top_plus_one_down {
        if y < heights.min_y {
            break;
        }
        if *block == SpawnBlockKind::Fluid {
            break;
        }
        if *block == SpawnBlockKind::Solid {
            return Some(y + 1);
        }
        y -= 1;
    }
    if cave_world && blocks_from_top_plus_one_down.is_empty() {
        None
    } else {
        None
    }
}

pub fn fixup_spawn_height(
    spawn_y: i32,
    min_y: i32,
    max_y: i32,
    no_collision_no_liquid: impl Fn(i32) -> bool,
) -> i32 {
    let mut y = spawn_y;
    while !no_collision_no_liquid(y) && y < max_y {
        y += 1;
    }
    y -= 1;
    while no_collision_no_liquid(y) && y > min_y {
        y -= 1;
    }
    y + 1
}

pub fn carver_can_reach(
    chunk_mid_x: f64,
    chunk_mid_z: f64,
    x: f64,
    z: f64,
    current_step: i32,
    total_steps: i32,
    thickness: f32,
) -> bool {
    let xd = x - chunk_mid_x;
    let zd = z - chunk_mid_z;
    let remaining = f64::from(total_steps - current_step);
    let rr = f64::from(thickness + 2.0 + 16.0);
    xd * xd + zd * zd - remaining * remaining <= rr * rr
}

#[cfg(test)]
mod tests {
    use super::{
        builtin_density_function, builtin_noise_generator_settings, builtin_noise_router,
        density_function_type, AquiferNoiseSettings, BinaryDensityFunction,
        BiomeGenerationSettingsModel, BlendingDataPacked, BlendingOutput, BlockPos, BlockPredicate,
        BlockPredicateContext, BlockStateProviderModel, CarverShape, CaveDensityOutput,
        CaveSurface, ConfiguredFeatureSource, DensityFunction, DensityMarker,
        FeatureConfigurationKind, FeatureFamily, FeatureSizeModel, FlatLayerInfo, FloatProvider,
        FluidStatus, FoliagePlacerKind, FoliagePlacerModel, HeightProvider, HeightRange,
        MangroveRootPlacementModel, MappedDensityFunction, MobSpawnerDataModel, NoiseRouterPreset,
        NoiseSettings, OreVeinDecisionInput, OreVeinifierConstants, PlacedFeatureSource,
        PlacementContextModel, PlacementModifier, RandomSpreadType, RootPlacerModel,
        SpawnBlockKind, SpawnColumnHeights, StructureFamily, StructurePlacementKind,
        SurfaceConditionSource, SurfaceMaterialContext, SurfaceRuleKind, SurfaceRulePreset,
        SurfaceRuleSource, TreeDecoratorModel, TreePlacementBlockKind, TrunkPlacerKind,
        TrunkPlacerModel, VerticalAnchor, WeightedBlockState, WeightedHeightProvider,
        WorldCarverType, WorldGenerationHeightContext, AQUIFER_NOISE_SETTINGS,
        AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS, BLENDING_CELL_COLUMN_COUNT, BLENDING_CONSTANTS,
        BLENDING_NO_VALUE, BLOCK_PREDICATE_TYPES, BUILTIN_DENSITY_FUNCTIONS,
        BUILTIN_NOISE_GENERATOR_SETTINGS, BUILTIN_NOISE_ROUTERS, BUILTIN_STRUCTURES,
        BUILTIN_STRUCTURE_SETS, BUILTIN_SURFACE_RULE_PRESETS, CAVES_NOISE_SETTINGS,
        CAVE_GENERATION_FAMILIES, CONFIGURED_CARVERS, CONFIGURED_FEATURES, DENSITY_FUNCTION_TYPES,
        END_NOISE_SETTINGS, FEATURE_BEHAVIOR_MODELS, FEATURE_TYPES, FLAT_DEFAULT_LAYERS,
        FLAT_GENERATOR_PRESETS, FLOATING_ISLANDS_NOISE_SETTINGS, HEIGHT_PROVIDER_TYPES,
        JIGSAW_POOL_BOOTSTRAP_SOURCES, MONSTER_ROOM_BOUNDS, NETHER_NOISE_SETTINGS,
        NORMAL_NOISE_INPUT_FACTOR, NORMAL_NOISE_PARAMETERS, NORMAL_NOISE_TARGET_DEVIATION,
        ORE_VEINIFIER_CONSTANTS, ORE_VEIN_TYPES, OVERWORLD_NOISE_SETTINGS, OVERWORLD_SPAWN_TARGET,
        PLACED_FEATURE_BOOTSTRAP_SOURCES, SPAWN_SELECTION_CONSTANTS, STRUCTURE_FAMILIES,
        STRUCTURE_PIECE_TYPES, STRUCTURE_POOL_ELEMENT_TYPES, STRUCTURE_POS_RULE_TEST_TYPES,
        STRUCTURE_PROCESSOR_LISTS, STRUCTURE_PROCESSOR_TYPES, STRUCTURE_RULE_TEST_TYPES,
        STRUCTURE_TYPES, SURFACE_CONDITION_TYPES, SURFACE_RULE_TYPES, SYNTH_NOISE_SOURCES,
        TEST_NEGATIVE_DENSITY, TEST_POSITIVE_DENSITY, UPGRADE_DATA_MODEL, WORLDGEN_TYPE_REGISTRIES,
        WORLD_CARVER_TYPES, WORLD_PRESETS, Y_DENSITY,
    };
    use crate::biome::{quantize_coord, BiomeSourceModel};
    use crate::storage::chunk::HeightmapKind;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;
    use std::collections::BTreeMap;

    #[test]
    fn noise_settings_presets_match_26_1_2_constants() {
        assert_eq!(OVERWORLD_NOISE_SETTINGS, NoiseSettings::new(-64, 384, 1, 2));
        assert_eq!(NETHER_NOISE_SETTINGS, NoiseSettings::new(0, 128, 1, 2));
        assert_eq!(END_NOISE_SETTINGS, NoiseSettings::new(0, 128, 2, 1));
        assert_eq!(CAVES_NOISE_SETTINGS, NoiseSettings::new(-64, 192, 1, 2));
        assert_eq!(
            FLOATING_ISLANDS_NOISE_SETTINGS,
            NoiseSettings::new(0, 256, 2, 1)
        );
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_width(), 4);
        assert_eq!(OVERWORLD_NOISE_SETTINGS.cell_height(), 8);
    }

    #[test]
    fn noise_settings_validation_and_clamp_follow_vanilla_rules() {
        assert!(NoiseSettings::new(-64, 384, 1, 2).validate().is_ok());
        assert_eq!(
            NoiseSettings::new(-63, 384, 1, 2).validate().unwrap_err(),
            "min_y has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(-64, 383, 1, 2).validate().unwrap_err(),
            "height has to be a multiple of 16"
        );
        assert_eq!(
            NoiseSettings::new(0, 2033, 1, 2).validate().unwrap_err(),
            "min_y + height cannot be higher than: 2032"
        );
        assert_eq!(
            NoiseSettings::new(-64, 384, 1, 2).clamp_to_height(0, 255),
            NoiseSettings::new(0, 256, 1, 2)
        );
    }

    #[test]
    fn synth_noise_sources_and_parameters_match_vanilla_bootstrap() {
        assert_eq!(
            SYNTH_NOISE_SOURCES
                .iter()
                .map(|source| source.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:normal_noise",
                "minecraft:perlin_noise",
                "minecraft:perlin_simplex_noise",
                "minecraft:simplex_noise",
                "minecraft:improved_noise",
                "minecraft:blended_noise",
            ]
        );
        assert_eq!(NORMAL_NOISE_PARAMETERS.len(), 62);
        assert_eq!(NORMAL_NOISE_INPUT_FACTOR, 1.0181268882175227);
        assert_eq!(NORMAL_NOISE_TARGET_DEVIATION, 1.0 / 3.0);

        let ids = NORMAL_NOISE_PARAMETERS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        assert_eq!(
            &ids[..12],
            &[
                "minecraft:temperature",
                "minecraft:vegetation",
                "minecraft:continentalness",
                "minecraft:erosion",
                "minecraft:temperature_large",
                "minecraft:vegetation_large",
                "minecraft:continentalness_large",
                "minecraft:erosion_large",
                "minecraft:nether/temperature",
                "minecraft:nether/vegetation",
                "minecraft:ridge",
                "minecraft:offset",
            ]
        );
        assert_eq!(
            &ids[ids.len() - 6..],
            &[
                "minecraft:soul_sand_layer",
                "minecraft:gravel_layer",
                "minecraft:patch",
                "minecraft:netherrack",
                "minecraft:nether_wart",
                "minecraft:nether_state_selector",
            ]
        );

        let temperature = super::builtin_normal_noise_parameters("temperature").unwrap();
        assert_eq!(temperature.first_octave, -10);
        assert_eq!(temperature.amplitudes, &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            super::builtin_normal_noise_parameters("minecraft:cave_cheese")
                .unwrap()
                .amplitudes,
            &[0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0]
        );
        assert_eq!(
            super::builtin_normal_noise_parameters("minecraft:jagged")
                .unwrap()
                .amplitudes
                .len(),
            16
        );
        assert!((super::normal_noise_expected_deviation(2) - 0.13333333333333333).abs() < 1e-12);
        assert!(
            (super::normal_noise_value_factor(
                *super::builtin_normal_noise_parameters("offset").unwrap()
            ) - 1.25)
                .abs()
                < 1e-12
        );
        assert!(super::synth_noise_source("blended_noise").is_some());
        assert!(super::synth_noise_source("value_noise").is_none());
    }

    #[test]
    fn height_provider_types_match_vanilla_registry_order() {
        assert_eq!(
            HEIGHT_PROVIDER_TYPES
                .iter()
                .map(|provider_type| provider_type.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:constant",
                "minecraft:uniform",
                "minecraft:biased_to_bottom",
                "minecraft:very_biased_to_bottom",
                "minecraft:trapezoid",
                "minecraft:weighted_list",
            ]
        );
        assert!(super::height_provider_type("constant").is_some());
        assert!(super::height_provider_type("minecraft:weighted_list").is_some());
        assert!(super::height_provider_type("clamped").is_none());
    }

    #[test]
    fn height_provider_sampling_envelopes_follow_vanilla_edge_cases() {
        let context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        assert_eq!(VerticalAnchor::Absolute(12).resolve_y(context), 12);
        assert_eq!(VerticalAnchor::AboveBottom(8).resolve_y(context), -56);
        assert_eq!(VerticalAnchor::BelowTop(1).resolve_y(context), 318);

        let constant = HeightProvider::Constant {
            value: VerticalAnchor::AboveBottom(8),
        };
        assert_eq!(
            super::height_provider_sample_bounds(constant, context),
            (-56, -56)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(constant, context, 99, 0, 0),
            -56
        );

        let uniform = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::Absolute(10),
            max_inclusive: VerticalAnchor::Absolute(14),
        };
        assert_eq!(
            super::height_provider_sample_bounds(uniform, context),
            (10, 14)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(uniform, context, 7, 0, 0),
            12
        );

        let empty_uniform = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::Absolute(20),
            max_inclusive: VerticalAnchor::Absolute(10),
        };
        assert_eq!(
            super::height_provider_sample_with_rolls(empty_uniform, context, 0, 0, 0),
            20
        );

        let biased = HeightProvider::BiasedToBottom {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            inner: 2,
        };
        assert_eq!(
            super::height_provider_sample_bounds(biased, context),
            (0, 9)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(biased, context, 4, 6, 0),
            0
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(biased, context, 8, 9, 0),
            9
        );

        let very_biased = HeightProvider::VeryBiasedToBottom {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            inner: 2,
        };
        assert_eq!(
            super::height_provider_sample_bounds(very_biased, context),
            (0, 9)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(very_biased, context, 8, 9, 9),
            9
        );

        let trapezoid = HeightProvider::Trapezoid {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            plateau: 2,
        };
        assert_eq!(
            super::height_provider_sample_bounds(trapezoid, context),
            (0, 10)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(trapezoid, context, 6, 4, 0),
            10
        );

        static DISTRIBUTION: &[WeightedHeightProvider] = &[
            WeightedHeightProvider {
                weight: 2,
                provider: HeightProvider::Constant {
                    value: VerticalAnchor::Absolute(4),
                },
            },
            WeightedHeightProvider {
                weight: 3,
                provider: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::Absolute(20),
                    max_inclusive: VerticalAnchor::Absolute(22),
                },
            },
        ];
        let weighted = HeightProvider::WeightedList {
            distribution: DISTRIBUTION,
        };
        assert_eq!(
            super::height_provider_sample_bounds(weighted, context),
            (4, 22)
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(weighted, context, 1, 99, 0),
            4
        );
        assert_eq!(
            super::height_provider_sample_with_rolls(weighted, context, 4, 5, 0),
            22
        );
    }

    #[test]
    fn block_predicate_types_match_vanilla_registry_order() {
        assert_eq!(
            BLOCK_PREDICATE_TYPES
                .iter()
                .map(|predicate_type| predicate_type.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:matching_blocks",
                "minecraft:matching_block_tag",
                "minecraft:matching_fluids",
                "minecraft:has_sturdy_face",
                "minecraft:solid",
                "minecraft:replaceable",
                "minecraft:would_survive",
                "minecraft:inside_world_bounds",
                "minecraft:any_of",
                "minecraft:all_of",
                "minecraft:not",
                "minecraft:true",
                "minecraft:unobstructed",
            ]
        );
        assert!(super::block_predicate_type("matching_blocks").is_some());
        assert!(super::block_predicate_type("minecraft:not").is_some());
        assert!(super::block_predicate_type("height_range").is_none());
    }

    #[test]
    fn block_predicate_core_evaluators_follow_vanilla_boolean_rules() {
        let grass = BlockPredicateContext {
            min_y: -64,
            height: 384,
            block: "minecraft:grass_block",
            fluid: "minecraft:empty",
            solid: true,
            replaceable: false,
            unobstructed: true,
        };
        assert!(super::block_predicate_test(
            BlockPredicate::True,
            grass,
            320
        ));
        assert!(super::block_predicate_test(
            BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt", "minecraft:grass_block"],
            },
            grass,
            64
        ));
        assert!(!super::block_predicate_test(
            BlockPredicate::MatchingFluids {
                fluids: &["minecraft:water"],
            },
            grass,
            64
        ));
        assert!(super::block_predicate_test(
            BlockPredicate::Solid,
            grass,
            64
        ));
        assert!(!super::block_predicate_test(
            BlockPredicate::Replaceable,
            grass,
            64
        ));
        assert!(super::block_predicate_test(
            BlockPredicate::InsideWorldBounds { offset_y: -1 },
            grass,
            320
        ));
        assert!(!super::block_predicate_test(
            BlockPredicate::InsideWorldBounds { offset_y: 0 },
            grass,
            320
        ));

        static SOLID: BlockPredicate = BlockPredicate::Solid;
        static REPLACEABLE: BlockPredicate = BlockPredicate::Replaceable;
        static UNOBSTRUCTED: BlockPredicate = BlockPredicate::Unobstructed;
        static ALL_PREDICATES: &[BlockPredicate] = &[SOLID, UNOBSTRUCTED];
        static ANY_PREDICATES: &[BlockPredicate] = &[REPLACEABLE, UNOBSTRUCTED];
        assert!(super::block_predicate_test(
            BlockPredicate::AllOf {
                predicates: ALL_PREDICATES,
            },
            grass,
            64
        ));
        assert!(super::block_predicate_test(
            BlockPredicate::AnyOf {
                predicates: ANY_PREDICATES,
            },
            grass,
            64
        ));
        assert!(super::block_predicate_test(
            BlockPredicate::Not {
                predicate: &REPLACEABLE,
            },
            grass,
            64
        ));
    }

    #[test]
    fn placement_modifier_registry_and_core_positions_follow_vanilla_rules() {
        assert!(super::placement_modifier_type("rarity_filter").is_some());
        assert!(super::placement_modifier_type("minecraft:fixed_placement").is_some());
        assert!(super::placement_modifier_type("matching_blocks").is_none());

        let origin = BlockPos {
            x: 32,
            y: 70,
            z: -16,
        };
        assert_eq!(
            super::placement_modifier_positions(
                PlacementModifier::RarityFilter { chance: 4 },
                origin,
                8,
                0,
                0
            ),
            vec![origin]
        );
        assert!(super::placement_modifier_positions(
            PlacementModifier::RarityFilter { chance: 4 },
            origin,
            9,
            0,
            0
        )
        .is_empty());
        assert_eq!(
            super::placement_modifier_positions(
                PlacementModifier::Count { count: 3 },
                origin,
                0,
                0,
                0
            ),
            vec![origin, origin, origin]
        );
        assert_eq!(
            super::placement_modifier_positions(PlacementModifier::InSquare, origin, 19, 31, 0),
            vec![BlockPos {
                x: 35,
                y: 70,
                z: -1,
            }]
        );
        assert_eq!(
            super::placement_modifier_positions(
                PlacementModifier::RandomOffset {
                    xz_spread: 4,
                    y_spread: 2,
                },
                origin,
                8,
                4,
                0
            ),
            vec![BlockPos {
                x: 36,
                y: 72,
                z: -20,
            }]
        );

        static FIXED_POSITIONS: &[BlockPos] = &[
            BlockPos {
                x: 34,
                y: 70,
                z: -8,
            },
            BlockPos {
                x: 48,
                y: 70,
                z: -8,
            },
            BlockPos {
                x: 35,
                y: 71,
                z: -1,
            },
        ];
        assert_eq!(
            super::placement_modifier_positions(
                PlacementModifier::Fixed {
                    positions: FIXED_POSITIONS,
                },
                origin,
                0,
                0,
                0
            ),
            vec![
                BlockPos {
                    x: 34,
                    y: 70,
                    z: -8
                },
                BlockPos {
                    x: 35,
                    y: 71,
                    z: -1
                },
            ]
        );

        let placement_context = PlacementContextModel {
            min_y: -64,
            world_surface_height: 81,
            ocean_floor_height: 63,
            biome_allows_feature: true,
            block_predicate: BlockPredicateContext {
                min_y: -64,
                height: 384,
                block: "minecraft:grass_block",
                fluid: "minecraft:empty",
                solid: true,
                replaceable: false,
                unobstructed: true,
            },
        };
        assert_eq!(
            super::placement_modifier_positions_with_context(
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::WorldSurface,
                },
                origin,
                placement_context,
                0,
                0,
                0,
            ),
            vec![BlockPos {
                x: 32,
                y: 81,
                z: -16,
            }]
        );
        assert_eq!(
            super::placement_modifier_positions_with_context(
                PlacementModifier::SurfaceRelativeThresholdFilter {
                    heightmap: HeightmapKind::WorldSurface,
                    min_inclusive: -16,
                    max_inclusive: 0,
                },
                BlockPos {
                    x: 32,
                    y: 70,
                    z: -16,
                },
                placement_context,
                0,
                0,
                0,
            ),
            vec![origin]
        );
        assert!(super::placement_modifier_positions_with_context(
            PlacementModifier::SurfaceWaterDepthFilter { max_water_depth: 8 },
            origin,
            placement_context,
            0,
            0,
            0,
        )
        .is_empty());
        assert_eq!(
            super::placement_modifier_positions_with_context(
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::Solid,
                },
                origin,
                placement_context,
                0,
                0,
                0,
            ),
            vec![origin]
        );
        assert_eq!(
            super::placed_feature_positions(
                &[
                    PlacementModifier::BiomeFilter,
                    PlacementModifier::Count { count: 2 },
                    PlacementModifier::InSquare,
                    PlacementModifier::Heightmap {
                        heightmap: HeightmapKind::WorldSurface,
                    },
                ],
                BlockPos {
                    x: 32,
                    y: 0,
                    z: -16
                },
                placement_context,
                &[(0, 0, 0), (0, 0, 0), (3, 4, 0), (0, 0, 0)],
            ),
            vec![
                BlockPos {
                    x: 35,
                    y: 81,
                    z: -12,
                },
                BlockPos {
                    x: 35,
                    y: 81,
                    z: -12,
                },
            ]
        );
        assert!(super::placed_feature_positions(
            &[PlacementModifier::BiomeFilter],
            origin,
            PlacementContextModel {
                biome_allows_feature: false,
                ..placement_context
            },
            &[(0, 0, 0)],
        )
        .is_empty());
    }

    #[test]
    fn flat_generator_defaults_and_presets_match_vanilla_bootstrap() {
        assert_eq!(
            FLAT_DEFAULT_LAYERS,
            &[
                FlatLayerInfo {
                    height: 1,
                    block: "minecraft:bedrock"
                },
                FlatLayerInfo {
                    height: 2,
                    block: "minecraft:dirt"
                },
                FlatLayerInfo {
                    height: 1,
                    block: "minecraft:grass_block"
                },
            ]
        );
        assert_eq!(super::flat_layers_total_height(FLAT_DEFAULT_LAYERS), 4);
        assert_eq!(
            super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 0),
            Some("minecraft:bedrock")
        );
        assert_eq!(
            super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 2),
            Some("minecraft:dirt")
        );
        assert_eq!(
            super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 3),
            Some("minecraft:grass_block")
        );
        assert_eq!(super::flat_block_at_y(FLAT_DEFAULT_LAYERS, 4), None);

        assert_eq!(
            FLAT_GENERATOR_PRESETS
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:classic_flat",
                "minecraft:tunnelers_dream",
                "minecraft:water_world",
                "minecraft:overworld",
                "minecraft:snowy_kingdom",
                "minecraft:bottomless_pit",
                "minecraft:desert",
                "minecraft:redstone_ready",
                "minecraft:the_void",
            ]
        );
        let overworld = super::flat_generator_preset("overworld").unwrap();
        assert_eq!(overworld.display, "minecraft:short_grass");
        assert_eq!(overworld.biome, "minecraft:plains");
        assert!(overworld.add_lakes);
        assert!(overworld.decoration);
        assert_eq!(super::flat_layers_total_height(overworld.layers), 64);
        assert_eq!(
            super::flat_block_at_y(overworld.layers, 0),
            Some("minecraft:bedrock")
        );
        assert_eq!(
            super::flat_block_at_y(overworld.layers, 63),
            Some("minecraft:grass_block")
        );
        assert!(overworld
            .structures
            .contains(&"minecraft:pillager_outposts"));

        let water = super::flat_generator_preset("minecraft:water_world").unwrap();
        assert_eq!(water.layers[0].height, 1);
        assert_eq!(water.layers[0].block, "minecraft:bedrock");
        assert_eq!(super::flat_layers_total_height(water.layers), 170);
        assert_eq!(
            super::flat_block_at_y(water.layers, 169),
            Some("minecraft:water")
        );

        let void = super::flat_generator_preset("the_void").unwrap();
        assert!(super::flat_layers_are_void(void.layers));
        assert_eq!(void.biome, "minecraft:the_void");
        assert_eq!(
            super::flat_block_at_y(void.layers, 0),
            Some("minecraft:air")
        );

        assert!(super::validate_flat_layers(FLAT_DEFAULT_LAYERS).is_ok());
        assert_eq!(
            super::validate_flat_layers(&[FlatLayerInfo {
                height: 385,
                block: "minecraft:stone",
            }]),
            Err("Sum of layer heights is > 384".to_string())
        );
    }

    #[test]
    fn world_preset_sources_match_vanilla_bootstrap_dimensions() {
        assert_eq!(
            WORLD_PRESETS
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:normal",
                "minecraft:flat",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:single_biome_surface",
                "minecraft:debug_all_block_states",
            ]
        );

        let normal = super::world_preset("normal").unwrap();
        assert_eq!(
            super::world_preset_dimensions_in_order(normal),
            [
                "minecraft:overworld",
                "minecraft:the_nether",
                "minecraft:the_end"
            ]
        );
        assert_eq!(normal.overworld.generator, "minecraft:noise");
        assert_eq!(normal.overworld.noise_settings, Some("minecraft:overworld"));
        assert_eq!(normal.nether.noise_settings, Some("minecraft:nether"));
        assert_eq!(normal.end.biome_source, "minecraft:the_end");

        let flat = super::world_preset("minecraft:flat").unwrap();
        assert_eq!(flat.overworld.generator, "minecraft:flat");
        assert_eq!(flat.overworld.biome_source, "minecraft:plains");
        assert_eq!(flat.overworld.noise_settings, None);

        let amplified = super::world_preset("amplified").unwrap();
        assert_eq!(
            amplified.overworld.noise_settings,
            Some("minecraft:amplified")
        );
        let single = super::world_preset("single_biome_surface").unwrap();
        assert_eq!(single.overworld.biome_source, "minecraft:fixed/plains");
        let debug = super::world_preset("debug_all_block_states").unwrap();
        assert_eq!(debug.overworld.generator, "minecraft:debug");

        assert_eq!(
            super::world_preset_from_overworld_generator("flat"),
            Some("minecraft:flat")
        );
        assert_eq!(
            super::world_preset_from_overworld_generator("minecraft:debug"),
            Some("minecraft:debug_all_block_states")
        );
        assert_eq!(
            super::world_preset_from_overworld_generator("minecraft:noise"),
            Some("minecraft:normal")
        );
        assert_eq!(super::world_preset_from_overworld_generator("custom"), None);

        assert!(super::validate_world_preset_dimensions(&[
            "minecraft:the_nether",
            "minecraft:overworld",
        ])
        .is_ok());
        assert_eq!(
            super::validate_world_preset_dimensions(&["minecraft:the_nether"]),
            Err("Missing overworld dimension".to_string())
        );
    }

    #[test]
    fn world_preset_resolver_dispatches_vanilla_generator_codecs() {
        assert_eq!(
            super::chunk_generator_kind("noise"),
            Some(super::ChunkGeneratorKind::Noise)
        );
        assert_eq!(
            super::chunk_generator_kind("minecraft:flat"),
            Some(super::ChunkGeneratorKind::Flat)
        );
        assert_eq!(
            super::chunk_generator_kind("minecraft:debug"),
            Some(super::ChunkGeneratorKind::Debug)
        );
        assert_eq!(super::chunk_generator_kind("minecraft:custom"), None);

        let normal = super::resolve_world_preset("normal").unwrap();
        assert_eq!(normal.id, "minecraft:normal");
        assert_eq!(normal.overworld.dimension, "minecraft:overworld");
        match normal.overworld.generator {
            super::ResolvedChunkGenerator::Noise {
                biome_source,
                biome_source_model,
                noise_settings,
            } => {
                assert_eq!(biome_source, "minecraft:multi_noise/overworld");
                assert_eq!(
                    biome_source_model,
                    BiomeSourceModel::MultiNoisePreset {
                        preset: "minecraft:overworld"
                    }
                );
                assert_eq!(noise_settings.id, "minecraft:overworld");
            }
            _ => panic!("normal overworld should resolve to noise"),
        }
        match normal.nether.generator {
            super::ResolvedChunkGenerator::Noise {
                biome_source,
                biome_source_model,
                noise_settings,
            } => {
                assert_eq!(biome_source, "minecraft:multi_noise/nether");
                assert_eq!(
                    biome_source_model,
                    BiomeSourceModel::MultiNoisePreset {
                        preset: "minecraft:nether"
                    }
                );
                assert_eq!(noise_settings.id, "minecraft:nether");
            }
            _ => panic!("normal nether should resolve to noise"),
        }
        match normal.end.generator {
            super::ResolvedChunkGenerator::Noise {
                biome_source,
                biome_source_model,
                noise_settings,
            } => {
                assert_eq!(biome_source, "minecraft:the_end");
                assert_eq!(biome_source_model, BiomeSourceModel::TheEnd);
                assert_eq!(noise_settings.id, "minecraft:end");
            }
            _ => panic!("normal end should resolve to noise"),
        }

        let large = super::resolve_world_preset("large_biomes").unwrap();
        match large.overworld.generator {
            super::ResolvedChunkGenerator::Noise { noise_settings, .. } => {
                assert_eq!(noise_settings.id, "minecraft:large_biomes");
            }
            _ => panic!("large biomes overworld should resolve to noise"),
        }

        let amplified = super::resolve_world_preset("amplified").unwrap();
        match amplified.overworld.generator {
            super::ResolvedChunkGenerator::Noise { noise_settings, .. } => {
                assert_eq!(noise_settings.id, "minecraft:amplified");
            }
            _ => panic!("amplified overworld should resolve to noise"),
        }

        let single = super::resolve_world_preset("single_biome_surface").unwrap();
        match single.overworld.generator {
            super::ResolvedChunkGenerator::Noise {
                biome_source,
                biome_source_model,
                noise_settings,
            } => {
                assert_eq!(biome_source, "minecraft:fixed/plains");
                assert_eq!(
                    biome_source_model,
                    BiomeSourceModel::Fixed {
                        biome: "minecraft:plains"
                    }
                );
                assert_eq!(noise_settings.id, "minecraft:overworld");
            }
            _ => panic!("single biome overworld should resolve to noise"),
        }

        let flat = super::resolve_world_preset("flat").unwrap();
        match flat.overworld.generator {
            super::ResolvedChunkGenerator::Flat {
                biome_source_model,
                settings,
            } => {
                assert_eq!(
                    biome_source_model,
                    BiomeSourceModel::Fixed {
                        biome: "minecraft:plains"
                    }
                );
                assert_eq!(settings.biome, "minecraft:plains");
                assert_eq!(
                    settings.expanded_layers,
                    vec![
                        Some("minecraft:bedrock"),
                        Some("minecraft:dirt"),
                        Some("minecraft:dirt"),
                        Some("minecraft:grass_block")
                    ]
                );
            }
            _ => panic!("flat overworld should resolve to flat"),
        }

        let debug = super::resolve_world_preset("debug_all_block_states").unwrap();
        match debug.overworld.generator {
            super::ResolvedChunkGenerator::Debug {
                biome,
                biome_source_model,
            } => {
                assert_eq!(biome, "minecraft:plains");
                assert_eq!(
                    biome_source_model,
                    BiomeSourceModel::Fixed {
                        biome: "minecraft:plains"
                    }
                );
            }
            _ => panic!("debug overworld should resolve to debug"),
        }
    }

    #[test]
    fn world_preset_resolver_rejects_invalid_generator_wiring() {
        assert_eq!(
            super::resolve_world_preset("missing").unwrap_err(),
            "Unknown world preset missing".to_string()
        );

        assert_eq!(
            super::resolve_level_stem(&super::LevelStemPreset {
                dimension: "minecraft:overworld",
                generator: "minecraft:custom",
                biome_source: "minecraft:plains",
                noise_settings: None,
            })
            .unwrap_err(),
            "Unknown chunk generator minecraft:custom".to_string()
        );

        assert_eq!(
            super::resolve_level_stem(&super::LevelStemPreset {
                dimension: "minecraft:overworld",
                generator: "minecraft:noise",
                biome_source: "minecraft:plains",
                noise_settings: None,
            })
            .unwrap_err(),
            "Noise generator minecraft:overworld has no settings".to_string()
        );

        assert_eq!(
            super::resolve_level_stem(&super::LevelStemPreset {
                dimension: "minecraft:overworld",
                generator: "minecraft:flat",
                biome_source: "minecraft:plains",
                noise_settings: Some("minecraft:overworld"),
            })
            .unwrap_err(),
            "Flat generator minecraft:overworld must not carry noise settings".to_string()
        );
    }

    #[test]
    fn resolved_flat_generator_materializes_overworld_chunks() {
        let chunk = super::generate_overworld_chunk_for_preset(ChunkPos { x: -3, z: 5 }, "flat")
            .expect("flat preset should generate a concrete chunk");
        assert_eq!(chunk.pos, ChunkPos { x: -3, z: 5 });
        assert_eq!(chunk.status, "minecraft:full");
        assert_eq!(chunk.sections.len(), 1);
        assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));
    }

    #[test]
    fn resolved_noise_generator_materializes_preview_terrain_chunks() {
        let chunk = super::generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")
            .expect("normal preset should generate preview terrain");
        assert_eq!(chunk.status, "minecraft:full");
        assert_eq!(chunk.sections.len(), 24);
        assert_eq!(chunk.sections[0].y, -4);
        assert_eq!(chunk.sections.last().unwrap().y, 19);
        assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));

        let overworld_settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let low = super::noise_preview_terrain_height(0, 0, overworld_settings);
        let nearby = super::noise_preview_terrain_height(15, 15, overworld_settings);
        let far = super::noise_preview_terrain_height(96, -48, overworld_settings);
        assert_ne!(low, far);
        assert!((low - nearby).abs() < 40);

        let Tag::Compound(section) = &chunk.sections[8].block_states else {
            panic!("block states should be stored as a compound");
        };
        let Some((_, Tag::List(palette))) = section.iter().find(|(name, _)| name == "palette")
        else {
            panic!("block states should include a palette");
        };
        assert!(palette.contains(&super::block_state_tag("minecraft:grass_block")));
        assert!(palette.contains(&super::block_state_tag("minecraft:stone")));
    }

    #[test]
    fn noise_preview_trees_follow_biome_generation_settings() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let terrain_heights = [settings.sea_level + 8; 16 * 16];
        let plains = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:plains",
            &terrain_heights,
        );
        let forest = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:forest",
            &terrain_heights,
        );
        let unknown = super::noise_preview_tree_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:badlands",
            &terrain_heights,
        );

        assert!(plains
            .iter()
            .any(|block| block.state == "minecraft:oak_log"));
        assert!(forest.len() > plains.len());
        assert!(forest
            .iter()
            .any(|block| block.state == "minecraft:birch_log"));
        assert!(unknown.is_empty());
    }

    #[test]
    fn forest_preview_chunk_uses_forest_decoration_palette() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let chunk = super::materialize_noise_preview_chunk(
            ChunkPos { x: 0, z: 0 },
            &BiomeSourceModel::Fixed {
                biome: "minecraft:forest",
            },
            settings,
        );

        assert!(chunk.sections.iter().any(|section| {
            let Tag::Compound(block_states) = &section.block_states else {
                return false;
            };
            let Some((_, Tag::List(palette))) =
                block_states.iter().find(|(name, _)| name == "palette")
            else {
                return false;
            };
            palette.contains(&super::block_state_tag("minecraft:birch_log"))
                || palette.contains(&super::block_state_tag("minecraft:birch_leaves"))
        }));
    }

    #[test]
    fn noise_preview_ground_cover_follows_biome_features() {
        let settings = super::builtin_noise_generator_settings("overworld").unwrap();
        let terrain_heights = [settings.sea_level + 8; 16 * 16];
        let plains = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:plains",
            &terrain_heights,
        );
        let sunflower = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:sunflower_plains",
            &terrain_heights,
        );
        let unknown = super::noise_preview_ground_cover_blocks(
            ChunkPos { x: 0, z: 0 },
            settings,
            "minecraft:badlands",
            &terrain_heights,
        );

        assert!(plains
            .iter()
            .any(|block| matches!(block.state, "minecraft:short_grass" | "minecraft:dandelion")));
        assert!(sunflower
            .iter()
            .any(|block| block.state == "minecraft:sunflower"));
        assert!(sunflower.len() >= plains.len());
        assert!(unknown.is_empty());
    }

    #[test]
    fn resolved_generators_answer_base_height_and_column_queries() {
        let flat = super::resolve_world_preset("flat").unwrap();
        assert_eq!(
            super::generator_base_height_for_stem(
                0,
                0,
                HeightmapKind::WorldSurfaceWg,
                &flat.overworld
            )
            .unwrap(),
            4
        );
        let flat_column = super::generator_base_column_for_stem(0, 0, &flat.overworld).unwrap();
        assert_eq!(flat_column.min_y, super::FLAT_GENERATOR_MIN_Y);
        assert_eq!(flat_column.states[0], "minecraft:bedrock");
        assert_eq!(flat_column.states[3], "minecraft:grass_block");

        let normal = super::resolve_world_preset("normal").unwrap();
        let world_surface = super::generator_base_height_for_stem(
            96,
            -48,
            HeightmapKind::WorldSurfaceWg,
            &normal.overworld,
        )
        .unwrap();
        let ocean_floor = super::generator_base_height_for_stem(
            96,
            -48,
            HeightmapKind::OceanFloorWg,
            &normal.overworld,
        )
        .unwrap();
        assert!(world_surface >= ocean_floor);

        let column = super::generator_base_column_for_stem(96, -48, &normal.overworld).unwrap();
        assert_eq!(column.min_y, super::OVERWORLD_NOISE_SETTINGS.min_y);
        assert_eq!(
            column.states.len(),
            super::OVERWORLD_NOISE_SETTINGS.height as usize
        );
        assert_eq!(column.states[0], "minecraft:bedrock");
        assert!(column.states.contains(&"minecraft:stone"));
        assert!(column.states.contains(&"minecraft:air"));
    }

    #[test]
    fn unresolved_debug_generation_fails_closed() {
        assert_eq!(
            super::generate_overworld_chunk_for_preset(
                ChunkPos { x: 0, z: 0 },
                "debug_all_block_states"
            )
            .unwrap_err(),
            "Debug chunk generation for minecraft:overworld is not implemented".to_string()
        );
    }

    #[test]
    fn noise_generator_settings_bootstrap_matches_vanilla_order_and_flags() {
        assert_eq!(
            BUILTIN_NOISE_GENERATOR_SETTINGS
                .iter()
                .map(|settings| settings.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
            ]
        );

        let overworld = builtin_noise_generator_settings("overworld").unwrap();
        assert_eq!(overworld.default_block, "minecraft:stone");
        assert_eq!(overworld.default_fluid, "minecraft:water");
        assert_eq!(
            overworld.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: false
            }
        );
        assert_eq!(overworld.surface_rule, SurfaceRulePreset::Overworld);
        assert_eq!(overworld.sea_level, 63);
        assert!(overworld.aquifers_enabled);
        assert!(overworld.ore_veins_enabled);
        assert!(!overworld.legacy_random_source);

        let large = builtin_noise_generator_settings("large_biomes").unwrap();
        assert_eq!(
            large.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: true,
                amplified: false
            }
        );
        let amplified = builtin_noise_generator_settings("amplified").unwrap();
        assert_eq!(
            amplified.noise_router,
            NoiseRouterPreset::Overworld {
                large_biomes: false,
                amplified: true
            }
        );

        let nether = builtin_noise_generator_settings("nether").unwrap();
        assert_eq!(nether.default_block, "minecraft:netherrack");
        assert_eq!(nether.default_fluid, "minecraft:lava");
        assert_eq!(nether.sea_level, 32);
        assert!(nether.legacy_random_source);

        let end = builtin_noise_generator_settings("end").unwrap();
        assert_eq!(end.default_block, "minecraft:end_stone");
        assert_eq!(end.default_fluid, "minecraft:air");
        assert!(end.disable_mob_generation);
        assert_eq!(end.sea_level, 0);
    }

    #[test]
    fn overworld_spawn_target_matches_overworld_biome_builder() {
        assert_eq!(OVERWORLD_SPAWN_TARGET.len(), 2);
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.min,
            quantize_coord(-0.11)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].continentalness.max,
            quantize_coord(1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.min,
            quantize_coord(-1.0)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[0].weirdness.max,
            quantize_coord(-0.16)
        );
        assert_eq!(
            OVERWORLD_SPAWN_TARGET[1].weirdness.min,
            quantize_coord(0.16)
        );
        assert_eq!(OVERWORLD_SPAWN_TARGET[1].weirdness.max, quantize_coord(1.0));
    }

    #[test]
    fn density_function_type_registry_matches_densityfunctions_bootstrap_order() {
        assert_eq!(DENSITY_FUNCTION_TYPES.len(), 34);
        assert_eq!(
            DENSITY_FUNCTION_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec![
                "blend_alpha",
                "blend_offset",
                "beardifier",
                "old_blended_noise",
                "interpolated",
                "flat_cache",
                "cache_2d",
                "cache_once",
                "cache_all_in_cell",
                "noise",
                "end_islands",
                "weird_scaled_sampler",
                "shifted_noise",
                "range_choice",
                "shift_a",
                "shift_b",
                "shift",
                "blend_density",
                "clamp",
                "abs",
                "square",
                "cube",
                "half_negative",
                "quarter_negative",
                "invert",
                "squeeze",
                "add",
                "mul",
                "min",
                "max",
                "spline",
                "constant",
                "y_clamped_gradient",
                "find_top_surface",
            ]
        );
        assert!(density_function_type("shifted_noise").is_some());
        assert!(density_function_type("missing").is_none());
    }

    #[test]
    fn density_function_core_evaluators_follow_vanilla_transform_rules() {
        assert_eq!(Y_DENSITY.compute(-5000), -4064.0);
        assert_eq!(Y_DENSITY.compute(5000), 4062.0);
        assert_eq!(Y_DENSITY.compute(0), 0.0);

        assert_eq!(MappedDensityFunction::Abs.transform(-2.0), 2.0);
        assert_eq!(MappedDensityFunction::Square.transform(-2.0), 4.0);
        assert_eq!(MappedDensityFunction::Cube.transform(-2.0), -8.0);
        assert_eq!(MappedDensityFunction::HalfNegative.transform(-2.0), -1.0);
        assert_eq!(MappedDensityFunction::QuarterNegative.transform(-2.0), -0.5);
        assert_eq!(MappedDensityFunction::Invert.transform(2.0), -2.0);
        assert!((MappedDensityFunction::Squeeze.transform(1.0) - 0.4583333333333333).abs() < 1e-12);

        let add = DensityFunction::Binary {
            kind: BinaryDensityFunction::Add,
            argument1: &TEST_NEGATIVE_DENSITY,
            argument2: &TEST_POSITIVE_DENSITY,
        };
        assert_eq!(add.compute(0), 1.0);
        assert_eq!(BinaryDensityFunction::Mul.apply(-2.0, 3.0), -6.0);
        assert_eq!(BinaryDensityFunction::Min.apply(-2.0, 3.0), -2.0);
        assert_eq!(BinaryDensityFunction::Max.apply(-2.0, 3.0), 3.0);
    }

    #[test]
    fn noise_router_density_function_bootstrap_keys_match_vanilla_prefix() {
        assert_eq!(
            BUILTIN_DENSITY_FUNCTIONS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:zero",
                "minecraft:y",
                "minecraft:shift_x",
                "minecraft:shift_z",
                "minecraft:overworld/base_3d_noise",
                "minecraft:nether/base_3d_noise",
                "minecraft:end/base_3d_noise",
                "minecraft:overworld/continents",
                "minecraft:overworld/erosion",
                "minecraft:overworld/ridges",
                "minecraft:overworld/ridges_folded",
                "minecraft:overworld_large_biomes/continents",
                "minecraft:overworld_large_biomes/erosion",
                "minecraft:end/sloped_cheese",
                "minecraft:overworld/caves/spaghetti_2d_thickness_modulator",
            ]
        );
        assert_eq!(
            builtin_density_function("overworld/base_3d_noise")
                .unwrap()
                .function
                .type_name(),
            "old_blended_noise"
        );
        assert_eq!(
            builtin_density_function("shift_x")
                .unwrap()
                .function
                .type_name(),
            DensityMarker::FlatCache.serialized_name()
        );
        assert_eq!(
            builtin_density_function("overworld/caves/spaghetti_2d_thickness_modulator")
                .unwrap()
                .function
                .type_name(),
            "cache_once"
        );
    }

    #[test]
    fn noise_router_record_shape_and_presets_match_noise_router_data() {
        assert_eq!(
            BUILTIN_NOISE_ROUTERS
                .iter()
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:large_biomes",
                "minecraft:amplified",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:caves",
                "minecraft:floating_islands",
                "minecraft:none",
            ]
        );

        let overworld = builtin_noise_router("overworld").unwrap().router;
        assert_eq!(
            overworld.field_type_names(),
            [
                "noise",
                "noise",
                "noise",
                "noise",
                "shifted_noise",
                "shifted_noise",
                "reference",
                "reference",
                "reference",
                "reference",
                "reference",
                "reference",
                "noise",
                "reference",
                "noise",
            ]
        );
        assert_eq!(
            overworld.barrier,
            DensityFunction::Noise {
                noise: "minecraft:aquifer_barrier",
                xz_scale: 1.0,
                y_scale: 0.5,
            }
        );
        assert_eq!(
            overworld.final_density,
            DensityFunction::Reference("minecraft:overworld/final_density")
        );

        let large = builtin_noise_router("large_biomes").unwrap().router;
        assert_eq!(
            large.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::SHIFT_X_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::SHIFT_Z_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature_large",
            }
        );
        assert_eq!(
            large.continents,
            DensityFunction::Reference("minecraft:overworld_large_biomes/continents")
        );

        let nether = builtin_noise_router("nether").unwrap().router;
        assert_eq!(
            nether.temperature,
            DensityFunction::ShiftedNoise {
                shift_x: &super::ZERO_DENSITY,
                shift_y: &super::ZERO_DENSITY,
                shift_z: &super::ZERO_DENSITY,
                xz_scale: 0.25,
                y_scale: 0.0,
                noise: "minecraft:temperature_nether",
            }
        );
        assert_eq!(
            builtin_noise_router("end")
                .unwrap()
                .router
                .erosion
                .type_name(),
            "cache_2d"
        );
        assert_eq!(
            builtin_noise_router("none").unwrap().router.final_density,
            DensityFunction::Constant(0.0)
        );
    }

    #[test]
    fn surface_rule_codecs_and_presets_match_surface_rule_data() {
        assert_eq!(
            SURFACE_RULE_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec!["bandlands", "block", "sequence", "condition"]
        );
        assert_eq!(
            SURFACE_CONDITION_TYPES
                .iter()
                .map(|kind| kind.id)
                .collect::<Vec<_>>(),
            vec![
                "biome",
                "noise_threshold",
                "vertical_gradient",
                "y_above",
                "water",
                "stone_depth",
                "not",
                "steep",
                "hole",
                "above_preliminary_surface",
                "temperature",
            ]
        );

        assert_eq!(
            BUILTIN_SURFACE_RULE_PRESETS
                .iter()
                .map(|preset| preset.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld",
                "minecraft:caves",
                "minecraft:floating_islands",
                "minecraft:nether",
                "minecraft:end",
                "minecraft:air",
            ]
        );

        let overworld = super::builtin_surface_rule_preset("overworld").unwrap();
        assert_eq!(
            overworld.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: true,
                bedrock_roof: false,
                bedrock_floor: true,
                deepslate: true,
            }
        );
        assert!(overworld.blocks.contains(&"minecraft:grass_block"));
        assert!(overworld.blocks.contains(&"minecraft:deepslate"));
        assert!(overworld.blocks.contains(&"minecraft:powder_snow"));
        assert!(overworld.conditions.contains(&"above_preliminary_surface"));
        assert!(overworld.conditions.contains(&"temperature"));

        let caves = super::builtin_surface_rule_preset("caves").unwrap();
        assert_eq!(
            caves.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: false,
                bedrock_roof: true,
                bedrock_floor: true,
                deepslate: true,
            }
        );
        let floating = super::builtin_surface_rule_preset("floating_islands").unwrap();
        assert_eq!(
            floating.rule,
            SurfaceRuleKind::OverworldLike {
                preliminary_surface_check: false,
                bedrock_roof: false,
                bedrock_floor: false,
                deepslate: true,
            }
        );

        let nether = super::builtin_surface_rule_preset("nether").unwrap();
        assert_eq!(nether.rule, SurfaceRuleKind::Nether);
        assert!(nether.blocks.contains(&"minecraft:netherrack"));
        assert!(nether.blocks.contains(&"minecraft:warped_nylium"));
        assert!(nether.blocks.contains(&"minecraft:crimson_nylium"));
        assert!(nether
            .conditions
            .contains(&"bedrock_roof_vertical_gradient"));
        assert!(nether.conditions.contains(&"noise_threshold"));

        assert_eq!(
            super::builtin_surface_rule_preset("end").unwrap().rule,
            SurfaceRuleKind::State("minecraft:end_stone")
        );
        assert_eq!(
            super::builtin_surface_rule_preset("air").unwrap().rule,
            SurfaceRuleKind::State("minecraft:air")
        );
    }

    #[test]
    fn material_rule_sources_evaluate_surface_conditions_in_vanilla_order() {
        assert!(super::surface_rule_type("block").is_some());
        assert!(super::surface_rule_type("sequence").is_some());
        assert!(super::surface_rule_type("condition").is_some());
        assert!(super::surface_condition_type("biome").is_some());
        assert!(super::surface_condition_type("noise_threshold").is_some());
        assert!(super::surface_condition_type("stone_depth").is_some());

        static PLAINS_OR_FOREST: SurfaceConditionSource =
            SurfaceConditionSource::Biome(&["minecraft:plains", "minecraft:forest"]);
        static DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::NoiseThreshold {
            min: -0.25,
            max: 0.25,
        };
        static NOT_DRY_NOISE: SurfaceConditionSource = SurfaceConditionSource::Not(&DRY_NOISE);
        static FLOOR: SurfaceConditionSource = SurfaceConditionSource::StoneDepth {
            offset: 0,
            add_surface_depth: true,
            secondary_depth_range: 0,
            surface: CaveSurface::Floor,
        };
        static GRASS: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:grass_block");
        static DIRT: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:dirt");
        static STONE: SurfaceRuleSource = SurfaceRuleSource::Block("minecraft:stone");
        static PLAINS_GRASS: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &PLAINS_OR_FOREST,
            rule: &GRASS,
        };
        static DRY_STONE: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &NOT_DRY_NOISE,
            rule: &STONE,
        };
        static FLOOR_DIRT: SurfaceRuleSource = SurfaceRuleSource::Condition {
            condition: &FLOOR,
            rule: &DIRT,
        };
        static RULES: &[SurfaceRuleSource] = &[PLAINS_GRASS, DRY_STONE, FLOOR_DIRT];
        static SEQUENCE: SurfaceRuleSource = SurfaceRuleSource::Sequence(RULES);

        let heights = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        let plains_surface = SurfaceMaterialContext {
            x: 12,
            y: 64,
            z: -4,
            biome: "minecraft:plains",
            stone_depth_above: 0,
            stone_depth_below: 3,
            surface_depth: 3,
            preliminary_surface_y: 62,
            water_height: 63,
            temperature: 0.8,
            noise: 0.0,
            steep: false,
            hole: false,
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &plains_surface, &heights),
            Some("minecraft:grass_block")
        );

        let noisy_desert = SurfaceMaterialContext {
            biome: "minecraft:desert",
            noise: 0.6,
            ..plains_surface
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &noisy_desert, &heights),
            Some("minecraft:stone")
        );

        let quiet_desert_floor = SurfaceMaterialContext {
            biome: "minecraft:desert",
            noise: 0.0,
            stone_depth_above: 2,
            ..plains_surface
        };
        assert_eq!(
            super::surface_rule_apply(&SEQUENCE, &quiet_desert_floor, &heights),
            Some("minecraft:dirt")
        );

        assert!(super::surface_condition_test(
            &SurfaceConditionSource::YAbove {
                anchor: VerticalAnchor::Absolute(59),
                surface_depth_multiplier: 1,
                add_stone_depth: true,
            },
            &quiet_desert_floor,
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::Water {
                offset: 2,
                surface_depth_multiplier: 0,
                add_stone_depth: false,
            },
            &quiet_desert_floor,
            &heights
        ));
        assert!(super::surface_condition_test(
            &SurfaceConditionSource::VerticalGradient {
                true_at_and_below: VerticalAnchor::Absolute(60),
                false_at_and_above: VerticalAnchor::Absolute(70),
            },
            &SurfaceMaterialContext {
                y: 65,
                noise: -0.5,
                ..quiet_desert_floor
            },
            &heights
        ));
        assert_eq!(
            super::surface_rule_apply(&SurfaceRuleSource::Bandlands, &quiet_desert_floor, &heights),
            Some("minecraft:red_sand")
        );
    }

    #[test]
    fn aquifer_constants_and_disabled_behavior_match_decompiled_rules() {
        assert_eq!(
            AQUIFER_NOISE_SETTINGS,
            AquiferNoiseSettings {
                x_range: 10,
                y_range: 9,
                z_range: 10,
                x_separation: 6,
                y_separation: 3,
                z_separation: 6,
                x_spacing: 16,
                y_spacing: 12,
                z_spacing: 16,
                max_reasonable_distance_to_center: 11,
                sample_offset_x: -5,
                sample_offset_y: 1,
                sample_offset_z: -5,
            }
        );
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS.len(), 13);
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[0], (0, 0));
        assert_eq!(AQUIFER_SURFACE_SAMPLING_OFFSETS_IN_CHUNKS[5], (-3, 0));

        let water = FluidStatus {
            fluid_level: 63,
            fluid_type: "minecraft:water",
        };
        assert_eq!(water.at(62), "minecraft:water");
        assert_eq!(water.at(63), "minecraft:air");
        assert_eq!(super::disabled_aquifer_substance(0.1, water, 62), None);
        assert_eq!(
            super::disabled_aquifer_substance(-0.1, water, 62),
            Some("minecraft:water")
        );
        assert_eq!(super::aquifer_similarity(100, 144), -0.76);
    }

    #[test]
    fn cave_generation_families_cover_noise_router_data_cave_builders() {
        assert_eq!(
            CAVE_GENERATION_FAMILIES
                .iter()
                .map(|family| family.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:overworld/caves/spaghetti_roughness_function",
                "minecraft:overworld/caves/entrances",
                "minecraft:overworld/caves/noodle",
                "minecraft:overworld/caves/pillars",
                "minecraft:overworld/caves/spaghetti_2d",
                "minecraft:overworld/caves/underground",
            ]
        );
        let entrances = super::cave_generation_family("overworld/caves/entrances").unwrap();
        assert!(entrances.noises.contains(&"minecraft:spaghetti_3d_1"));
        assert!(entrances.noises.contains(&"minecraft:cave_entrance"));
        assert_eq!(entrances.output, CaveDensityOutput::CacheOnce);

        let noodle = super::cave_generation_family("overworld/caves/noodle").unwrap();
        assert!(noodle.noises.contains(&"minecraft:noodle_ridge_a"));
        assert_eq!(noodle.output, CaveDensityOutput::RangeChoice);

        let spaghetti_2d = super::cave_generation_family("overworld/caves/spaghetti_2d").unwrap();
        assert!(spaghetti_2d
            .noises
            .contains(&"minecraft:spaghetti_2d_elevation"));
        assert_eq!(
            spaghetti_2d.output,
            CaveDensityOutput::Clamp { min: -1, max: 1 }
        );
    }

    #[test]
    fn ore_veinifier_constants_and_vein_types_match_decompiled_values() {
        assert_eq!(
            ORE_VEINIFIER_CONSTANTS,
            OreVeinifierConstants {
                veininess_threshold: 0.4,
                edge_roundoff_begin: 20,
                max_edge_roundoff: 0.2,
                vein_solidness: 0.7,
                min_richness: 0.1,
                max_richness: 0.3,
                max_richness_threshold: 0.6,
                chance_of_raw_ore_block: 0.02,
                skip_ore_if_gap_noise_is_below: -0.3,
            }
        );
        assert_eq!(ORE_VEIN_TYPES.len(), 2);
        assert_eq!(ORE_VEIN_TYPES[0].id, "copper");
        assert_eq!(ORE_VEIN_TYPES[0].ore, "minecraft:copper_ore");
        assert_eq!(
            ORE_VEIN_TYPES[0].raw_ore_block,
            "minecraft:raw_copper_block"
        );
        assert_eq!(ORE_VEIN_TYPES[0].filler, "minecraft:granite");
        assert_eq!((ORE_VEIN_TYPES[0].min_y, ORE_VEIN_TYPES[0].max_y), (0, 50));
        assert_eq!(ORE_VEIN_TYPES[1].id, "iron");
        assert_eq!(ORE_VEIN_TYPES[1].ore, "minecraft:deepslate_iron_ore");
        assert_eq!(ORE_VEIN_TYPES[1].raw_ore_block, "minecraft:raw_iron_block");
        assert_eq!(ORE_VEIN_TYPES[1].filler, "minecraft:tuff");
        assert_eq!(
            (ORE_VEIN_TYPES[1].min_y, ORE_VEIN_TYPES[1].max_y),
            (-60, -8)
        );
    }

    #[test]
    fn ore_veinifier_decision_matches_vanilla_branching() {
        let base = OreVeinDecisionInput {
            y: 25,
            vein_toggle: 0.61,
            vein_ridged: -0.1,
            vein_gap: 0.0,
            solidness_random: 0.5,
            richness_random: 0.2,
            raw_ore_random: 0.5,
            debug_ore_veins: false,
        };
        assert_eq!(super::ore_vein_richness(0.4), 0.1);
        assert_eq!(super::ore_vein_richness(0.6), 0.3);
        assert_eq!(super::ore_vein_decision(base), Some("minecraft:copper_ore"));
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                raw_ore_random: 0.01,
                ..base
            }),
            Some("minecraft:raw_copper_block")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                y: -30,
                vein_toggle: -0.61,
                raw_ore_random: 0.5,
                ..base
            }),
            Some("minecraft:deepslate_iron_ore")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                vein_gap: -0.31,
                ..base
            }),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                solidness_random: 0.71,
                ..base
            }),
            None
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                y: 100,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:air")
        );
        assert_eq!(
            super::ore_vein_decision(OreVeinDecisionInput {
                richness_random: 0.99,
                debug_ore_veins: true,
                ..base
            }),
            Some("minecraft:oak_button")
        );
    }

    #[test]
    fn configured_carvers_match_vanilla_bootstrap_entries() {
        assert_eq!(
            WORLD_CARVER_TYPES
                .iter()
                .map(|carver| carver.id())
                .collect::<Vec<_>>(),
            vec![
                "minecraft:cave",
                "minecraft:nether_cave",
                "minecraft:canyon"
            ]
        );
        assert_eq!(
            super::world_carver_type("cave"),
            Some(WorldCarverType::Cave)
        );
        assert_eq!(
            super::world_carver_type("minecraft:nether_cave"),
            Some(WorldCarverType::NetherCave)
        );

        assert_eq!(
            CONFIGURED_CARVERS
                .iter()
                .map(|carver| carver.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:cave",
                "minecraft:cave_extra_underground",
                "minecraft:canyon",
                "minecraft:nether_cave",
            ]
        );

        let cave = super::configured_carver("cave").unwrap();
        assert_eq!(cave.carver_type, WorldCarverType::Cave);
        assert_eq!(cave.probability, 0.15);
        assert!(super::carver_is_start_chunk(cave, 0.15));
        assert!(!super::carver_is_start_chunk(cave, 0.150_001));
        assert_eq!(
            cave.y,
            HeightRange {
                min: VerticalAnchor::AboveBottom(8),
                max: VerticalAnchor::Absolute(180),
            }
        );
        assert_eq!(cave.y_scale, FloatProvider::Uniform { min: 0.1, max: 0.9 });
        assert_eq!(cave.lava_level, VerticalAnchor::AboveBottom(8));
        assert_eq!(cave.debug.barrier_state, "minecraft:crimson_button");
        assert_eq!(
            cave.replaceable_tag,
            "#minecraft:overworld_carver_replaceables"
        );

        let extra = super::configured_carver("cave_extra_underground").unwrap();
        assert_eq!(extra.probability, 0.07);
        assert_eq!(extra.y.max, VerticalAnchor::Absolute(47));
        assert_eq!(extra.debug.barrier_state, "minecraft:oak_button");

        let canyon = super::configured_carver("canyon").unwrap();
        assert_eq!(canyon.carver_type, WorldCarverType::Canyon);
        assert_eq!(canyon.probability, 0.01);
        assert_eq!(canyon.y_scale, FloatProvider::Constant(3.0));
        assert_eq!(canyon.debug.barrier_state, "minecraft:warped_button");
        assert!(matches!(
            canyon.shape,
            CarverShape::Canyon {
                vertical_rotation: FloatProvider::Uniform {
                    min: -0.125,
                    max: 0.125
                },
                ..
            }
        ));

        let nether = super::configured_carver("nether_cave").unwrap();
        assert_eq!(nether.carver_type, WorldCarverType::NetherCave);
        assert_eq!(nether.probability, 0.2);
        assert_eq!(nether.y.min, VerticalAnchor::Absolute(0));
        assert_eq!(nether.y.max, VerticalAnchor::BelowTop(1));
        assert_eq!(
            nether.replaceable_tag,
            "#minecraft:nether_carver_replaceables"
        );
        assert!(matches!(
            nether.shape,
            CarverShape::Cave {
                floor_level: FloatProvider::Constant(-0.7),
                ..
            }
        ));
    }

    #[test]
    fn world_carver_can_reach_matches_vanilla_distance_gate() {
        assert!(super::carver_can_reach(8.0, 8.0, 8.0, 8.0, 0, 10, 1.0));
        assert!(super::carver_can_reach(8.0, 8.0, 30.0, 8.0, 0, 10, 4.0));
        assert!(!super::carver_can_reach(8.0, 8.0, 80.0, 8.0, 9, 10, 1.0));
    }

    #[test]
    fn feature_type_registry_matches_vanilla_feature_order() {
        assert_eq!(FEATURE_TYPES.len(), 60);
        assert_eq!(
            FEATURE_TYPES
                .iter()
                .map(|feature| feature.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:no_op",
                "minecraft:tree",
                "minecraft:fallen_tree",
                "minecraft:block_pile",
                "minecraft:spring_feature",
                "minecraft:chorus_plant",
                "minecraft:replace_single_block",
                "minecraft:void_start_platform",
                "minecraft:desert_well",
                "minecraft:fossil",
                "minecraft:huge_red_mushroom",
                "minecraft:huge_brown_mushroom",
                "minecraft:spike",
                "minecraft:glowstone_blob",
                "minecraft:freeze_top_layer",
                "minecraft:vines",
                "minecraft:block_column",
                "minecraft:vegetation_patch",
                "minecraft:waterlogged_vegetation_patch",
                "minecraft:root_system",
                "minecraft:multiface_growth",
                "minecraft:underwater_magma",
                "minecraft:monster_room",
                "minecraft:blue_ice",
                "minecraft:iceberg",
                "minecraft:block_blob",
                "minecraft:disk",
                "minecraft:lake",
                "minecraft:ore",
                "minecraft:end_platform",
                "minecraft:end_spike",
                "minecraft:end_island",
                "minecraft:end_gateway",
                "minecraft:seagrass",
                "minecraft:kelp",
                "minecraft:coral_tree",
                "minecraft:coral_mushroom",
                "minecraft:coral_claw",
                "minecraft:sea_pickle",
                "minecraft:simple_block",
                "minecraft:bamboo",
                "minecraft:huge_fungus",
                "minecraft:nether_forest_vegetation",
                "minecraft:weeping_vines",
                "minecraft:twisting_vines",
                "minecraft:basalt_columns",
                "minecraft:delta_feature",
                "minecraft:netherrack_replace_blobs",
                "minecraft:fill_layer",
                "minecraft:bonus_chest",
                "minecraft:basalt_pillar",
                "minecraft:scattered_ore",
                "minecraft:random_selector",
                "minecraft:simple_random_selector",
                "minecraft:random_boolean_selector",
                "minecraft:geode",
                "minecraft:dripstone_cluster",
                "minecraft:large_dripstone",
                "minecraft:pointed_dripstone",
                "minecraft:sculk_patch",
            ]
        );

        let tree = super::feature_type_by_id("tree").unwrap();
        assert_eq!(tree.configuration, FeatureConfigurationKind::Tree);
        assert_eq!(tree.family, FeatureFamily::Tree);

        let ore = super::feature_type_by_id("minecraft:ore").unwrap();
        assert_eq!(ore.configuration, FeatureConfigurationKind::Ore);
        assert_eq!(ore.family, FeatureFamily::Ore);

        let random_selector = super::feature_type_by_id("random_selector").unwrap();
        assert_eq!(
            random_selector.configuration,
            FeatureConfigurationKind::RandomFeature
        );
        assert_eq!(random_selector.family, FeatureFamily::Selector);

        let sculk_patch = super::feature_type_by_id("sculk_patch").unwrap();
        assert_eq!(
            sculk_patch.configuration,
            FeatureConfigurationKind::SculkPatch
        );
        assert_eq!(sculk_patch.family, FeatureFamily::Cave);
    }

    #[test]
    fn configured_feature_bootstrap_keys_match_vanilla_sources() {
        assert_eq!(CONFIGURED_FEATURES.len(), 221);

        let source_counts = [
            (ConfiguredFeatureSource::Aquatic, 7),
            (ConfiguredFeatureSource::Cave, 24),
            (ConfiguredFeatureSource::End, 6),
            (ConfiguredFeatureSource::MiscOverworld, 18),
            (ConfiguredFeatureSource::Nether, 22),
            (ConfiguredFeatureSource::Ore, 32),
            (ConfiguredFeatureSource::Pile, 5),
            (ConfiguredFeatureSource::Tree, 50),
            (ConfiguredFeatureSource::Vegetation, 57),
        ];
        for (source, expected_count) in source_counts {
            assert_eq!(
                CONFIGURED_FEATURES
                    .iter()
                    .filter(|feature| feature.source == source)
                    .count(),
                expected_count
            );
        }

        assert_eq!(
            CONFIGURED_FEATURES
                .iter()
                .take(7)
                .map(|feature| feature.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:seagrass_short",
                "minecraft:seagrass_slightly_less_short",
                "minecraft:seagrass_mid",
                "minecraft:seagrass_tall",
                "minecraft:sea_pickle",
                "minecraft:kelp",
                "minecraft:warm_ocean_vegetation",
            ]
        );
        assert_eq!(
            CONFIGURED_FEATURES.last().map(|feature| feature.id),
            Some("minecraft:mangrove_vegetation")
        );

        assert_eq!(
            super::configured_feature("ore_diamond_buried").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Ore)
        );
        assert_eq!(
            super::configured_feature("minecraft:pale_oak_creaking").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Tree)
        );
        assert_eq!(
            super::configured_feature("sculk_patch_ancient_city").map(|feature| feature.source),
            Some(ConfiguredFeatureSource::Cave)
        );
    }

    #[test]
    fn placed_feature_bootstrap_keys_match_vanilla_sources() {
        assert_eq!(PLACED_FEATURE_BOOTSTRAP_SOURCES.len(), 9);
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .iter()
                .map(|entry| (entry.source, entry.keys.len()))
                .collect::<Vec<_>>(),
            vec![
                (PlacedFeatureSource::Aquatic, 12),
                (PlacedFeatureSource::Cave, 20),
                (PlacedFeatureSource::End, 5),
                (PlacedFeatureSource::MiscOverworld, 18),
                (PlacedFeatureSource::Nether, 20),
                (PlacedFeatureSource::Ore, 40),
                (PlacedFeatureSource::Tree, 41),
                (PlacedFeatureSource::Vegetation, 89),
                (PlacedFeatureSource::Village, 13),
            ]
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .iter()
                .map(|entry| entry.keys.len())
                .sum::<usize>(),
            258
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES[0].keys.first().copied(),
            Some("minecraft:seagrass_warm")
        );
        assert_eq!(
            PLACED_FEATURE_BOOTSTRAP_SOURCES
                .last()
                .and_then(|entry| entry.keys.last())
                .copied(),
            Some("minecraft:patch_berry_bush")
        );
        assert_eq!(
            super::placed_feature_source("ore_diamond"),
            Some(PlacedFeatureSource::Ore)
        );
        assert_eq!(
            super::placed_feature_source("minecraft:pale_oak_creaking_checked"),
            Some(PlacedFeatureSource::Tree)
        );
        assert_eq!(
            super::placed_feature_source("trees_mangrove"),
            Some(PlacedFeatureSource::Vegetation)
        );
    }

    #[test]
    fn biome_generation_settings_plains_matches_registry_payload() {
        let plains = super::biome_generation_settings("plains").unwrap();
        assert_eq!(
            *plains,
            BiomeGenerationSettingsModel {
                biome: "minecraft:plains",
                carvers: &[
                    "minecraft:cave",
                    "minecraft:cave_extra_underground",
                    "minecraft:canyon",
                ],
                feature_steps: super::PLAINS_FEATURE_STEPS,
                creature_spawn_probability: 0.1,
                spawn_costs: &[],
                spawners: super::PLAINS_SPAWNER_GROUPS,
            }
        );
        assert_eq!(plains.feature_steps.len(), 11);
        assert_eq!(plains.feature_steps[1].len(), 2);
        assert_eq!(plains.feature_steps[6].len(), 29);
        assert_eq!(
            plains.feature_steps[9],
            &[
                "minecraft:glow_lichen",
                "minecraft:patch_tall_grass_2",
                "minecraft:patch_bush",
                "minecraft:trees_plains",
                "minecraft:flower_plains",
                "minecraft:patch_grass_plain",
                "minecraft:brown_mushroom_normal",
                "minecraft:red_mushroom_normal",
                "minecraft:patch_pumpkin",
                "minecraft:patch_sugar_cane",
                "minecraft:patch_firefly_bush_near_water",
            ]
        );
        assert!(super::biome_has_placed_feature(plains, "trees_plains"));
        assert!(super::biome_has_placed_feature(
            plains,
            "minecraft:ore_diamond_buried"
        ));
        assert!(!super::biome_has_placed_feature(plains, "trees_jungle"));
        assert_eq!(
            super::biome_spawns_for_category(plains, "creature"),
            &[
                MobSpawnerDataModel {
                    entity_type: "minecraft:sheep",
                    weight: 12,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:pig",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:chicken",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:cow",
                    weight: 8,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:horse",
                    weight: 5,
                    min_count: 2,
                    max_count: 6,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:donkey",
                    weight: 1,
                    min_count: 1,
                    max_count: 3,
                },
            ]
        );
        assert_eq!(
            super::biome_spawns_for_category(plains, "underground_water_creature"),
            &[MobSpawnerDataModel {
                entity_type: "minecraft:glow_squid",
                weight: 10,
                min_count: 4,
                max_count: 6,
            }]
        );
        assert!(super::biome_spawns_for_category(plains, "water_creature").is_empty());
        assert!(super::biome_generation_settings("minecraft:badlands").is_none());
    }

    #[test]
    fn biome_generation_settings_cover_neighboring_overworld_payloads() {
        let sunflower = super::biome_generation_settings("sunflower_plains").unwrap();
        assert_eq!(sunflower.biome, "minecraft:sunflower_plains");
        assert_eq!(sunflower.carvers, super::OVERWORLD_COMMON_CARVERS);
        assert!(super::biome_has_placed_feature(
            sunflower,
            "minecraft:patch_sunflower"
        ));
        assert!(super::biome_has_placed_feature(
            sunflower,
            "minecraft:trees_plains"
        ));
        assert!(!super::biome_has_placed_feature(
            sunflower,
            "minecraft:trees_birch_and_oak_leaf_litter"
        ));
        assert_eq!(
            super::biome_spawns_for_category(sunflower, "creature"),
            super::PLAINS_CREATURE_SPAWNS
        );

        let forest = super::biome_generation_settings("minecraft:forest").unwrap();
        assert_eq!(forest.biome, "minecraft:forest");
        assert_eq!(forest.feature_steps.len(), 11);
        assert!(super::biome_has_placed_feature(
            forest,
            "minecraft:forest_flowers"
        ));
        assert!(super::biome_has_placed_feature(
            forest,
            "minecraft:trees_birch_and_oak_leaf_litter"
        ));
        assert!(!super::biome_has_placed_feature(
            forest,
            "minecraft:flower_plains"
        ));
        assert_eq!(
            super::biome_spawns_for_category(forest, "creature"),
            &[
                MobSpawnerDataModel {
                    entity_type: "minecraft:sheep",
                    weight: 12,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:pig",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:chicken",
                    weight: 10,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:cow",
                    weight: 8,
                    min_count: 4,
                    max_count: 4,
                },
                MobSpawnerDataModel {
                    entity_type: "minecraft:wolf",
                    weight: 5,
                    min_count: 4,
                    max_count: 4,
                },
            ]
        );
        assert_eq!(
            super::biome_spawns_for_category(forest, "monster")[1],
            MobSpawnerDataModel {
                entity_type: "minecraft:zombie",
                weight: 95,
                min_count: 4,
                max_count: 4,
            }
        );
    }

    #[test]
    fn feature_sorter_builds_step_order_and_index_mapping_like_vanilla() {
        let plains = super::biome_generation_settings("plains").unwrap();
        let forest = super::biome_generation_settings("forest").unwrap();
        let sorted =
            super::build_features_per_step(&[plains.feature_steps, forest.feature_steps], true)
                .unwrap();

        assert_eq!(sorted.len(), 11);
        assert!(sorted[0].features.is_empty());
        assert_eq!(
            sorted[1].features,
            vec![
                "minecraft:lake_lava_underground",
                "minecraft:lake_lava_surface"
            ]
        );
        assert!(sorted[6].features.contains(&"minecraft:ore_diamond_buried"));
        assert!(sorted[9].features.contains(&"minecraft:trees_plains"));
        assert!(sorted[9]
            .features
            .contains(&"minecraft:trees_birch_and_oak_leaf_litter"));
        assert_eq!(
            sorted[9].index_mapping("minecraft:trees_plains"),
            Some(
                sorted[9]
                    .features
                    .iter()
                    .position(|feature| *feature == "minecraft:trees_plains")
                    .unwrap()
            )
        );
        assert_eq!(sorted[9].index_mapping("minecraft:missing"), None);
    }

    #[test]
    fn feature_sorter_reports_order_cycles() {
        static SOURCE_A: &[&[&str]] = &[&["minecraft:a", "minecraft:b"]];
        static SOURCE_B: &[&[&str]] = &[&["minecraft:b", "minecraft:a"]];

        assert_eq!(
            super::build_features_per_step(&[SOURCE_A, SOURCE_B], false).unwrap_err(),
            "Feature order cycle found".to_string()
        );
        assert_eq!(
            super::build_features_per_step(&[SOURCE_A, SOURCE_B], true).unwrap_err(),
            "Feature order cycle found, involved sources: 2".to_string()
        );
    }

    #[test]
    fn feature_placement_support_registries_match_vanilla_type_bootstraps() {
        assert_eq!(
            WORLDGEN_TYPE_REGISTRIES
                .iter()
                .map(|registry| (registry.id, registry.entries.len()))
                .collect::<Vec<_>>(),
            vec![
                ("minecraft:height_provider_type", 6),
                ("minecraft:block_predicate_type", 13),
                ("minecraft:placement_modifier_type", 14),
                ("minecraft:trunk_placer_type", 9),
                ("minecraft:foliage_placer_type", 11),
                ("minecraft:block_state_provider_type", 8),
                ("minecraft:tree_decorator_type", 10),
                ("minecraft:feature_size_type", 2),
                ("minecraft:root_placer_type", 1),
            ]
        );
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:placement_modifier_type")
            .unwrap()
            .entries
            .contains(&"minecraft:environment_scan"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:tree_decorator_type")
            .unwrap()
            .entries
            .contains(&"minecraft:creaking_heart"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:trunk_placer_type")
            .unwrap()
            .entries
            .contains(&"minecraft:upwards_branching_trunk_placer"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:block_state_provider_type")
            .unwrap()
            .entries
            .contains(&"minecraft:rule_based_state_provider"));
        assert_eq!(
            super::block_state_provider_type("simple_state_provider"),
            Some("minecraft:simple_state_provider")
        );
        assert_eq!(
            super::block_state_provider_type("minecraft:weighted_state_provider"),
            Some("minecraft:weighted_state_provider")
        );
        assert_eq!(super::block_state_provider_type("missing"), None);
        assert_eq!(
            super::block_state_provider_sample(
                &BlockStateProviderModel::Simple("minecraft:oak_log"),
                99
            ),
            Some("minecraft:oak_log")
        );
        let weighted = BlockStateProviderModel::Weighted(vec![
            WeightedBlockState {
                state: "minecraft:stone",
                weight: 2,
            },
            WeightedBlockState {
                state: "minecraft:andesite",
                weight: 1,
            },
        ]);
        assert_eq!(
            super::block_state_provider_sample(&weighted, 0),
            Some("minecraft:stone")
        );
        assert_eq!(
            super::block_state_provider_sample(&weighted, 2),
            Some("minecraft:andesite")
        );
        assert_eq!(
            super::block_state_provider_sample(
                &BlockStateProviderModel::Weighted(vec![WeightedBlockState {
                    state: "minecraft:air",
                    weight: 0,
                }]),
                0,
            ),
            None
        );
        let flower_provider = BlockStateProviderModel::Simple("minecraft:dandelion");
        let simple_config = super::SimpleBlockConfigurationModel {
            to_place: flower_provider,
            schedule_tick: false,
        };
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                0,
            ),
            Some(super::SimpleBlockPlacementPlan {
                state: "minecraft:dandelion",
                upper_state: None,
                schedule_tick: false,
            })
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:stone",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                0,
            ),
            None
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &simple_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:stone",
                    above_block: "minecraft:air",
                },
                0,
            ),
            None
        );

        let sunflower_provider = BlockStateProviderModel::Simple("minecraft:sunflower");
        let sunflower_config = super::SimpleBlockConfigurationModel {
            to_place: sunflower_provider,
            schedule_tick: true,
        };
        assert_eq!(
            super::simple_block_placement_plan(
                &sunflower_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                0,
            ),
            Some(super::SimpleBlockPlacementPlan {
                state: "minecraft:sunflower",
                upper_state: Some("minecraft:sunflower"),
                schedule_tick: true,
            })
        );
        assert_eq!(
            super::simple_block_placement_plan(
                &sunflower_config,
                super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:oak_leaves",
                },
                0,
            ),
            None
        );

        let replace_targets = [
            super::TargetBlockStateModel {
                target: super::RuleTestModel::BlockMatch("minecraft:stone"),
                state: "minecraft:granite",
            },
            super::TargetBlockStateModel {
                target: super::RuleTestModel::TagMatch(&[
                    "minecraft:dirt",
                    "minecraft:grass_block",
                ]),
                state: "minecraft:coarse_dirt",
            },
            super::TargetBlockStateModel {
                target: super::RuleTestModel::AlwaysTrue,
                state: "minecraft:air",
            },
        ];
        assert_eq!(
            super::replace_block_result("minecraft:stone", &replace_targets),
            Some("minecraft:granite")
        );
        assert_eq!(
            super::replace_block_result("minecraft:grass_block", &replace_targets),
            Some("minecraft:coarse_dirt")
        );
        assert_eq!(
            super::replace_block_result("minecraft:deepslate", &replace_targets),
            Some("minecraft:air")
        );
        assert_eq!(super::replace_block_result("minecraft:stone", &[]), None);

        let ore_config = super::OreConfigurationModel {
            target_states: vec![super::TargetBlockStateModel {
                target: super::RuleTestModel::BlockMatch("minecraft:stone"),
                state: "minecraft:iron_ore",
            }],
            size: 9,
            discard_chance_on_air_exposure: 0.5,
        };
        let ore_target = ore_config.target_states[0];
        assert!(super::ore_should_skip_air_check(0.0, 0.0));
        assert!(!super::ore_should_skip_air_check(1.0, 1.0));
        assert!(!super::ore_should_skip_air_check(0.5, 0.49));
        assert!(super::ore_should_skip_air_check(0.5, 0.5));
        assert!(super::ore_can_place(
            "minecraft:stone",
            false,
            &ore_config,
            ore_target,
            0.0,
        ));
        assert!(!super::ore_can_place(
            "minecraft:dirt",
            false,
            &ore_config,
            ore_target,
            1.0,
        ));
        assert!(!super::ore_can_place(
            "minecraft:stone",
            true,
            &ore_config,
            ore_target,
            0.0,
        ));
        assert_eq!(
            super::scattered_ore_offset(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                9,
                [(1.0, 0.0), (0.0, 1.0), (0.75, 0.25)],
            ),
            BlockPos {
                x: 17,
                y: 13,
                z: 34,
            }
        );
        assert_eq!(
            super::scattered_ore_attempt(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                2,
                [(1.0, 0.0), (0.0, 1.0), (0.5, 0.5)],
                "minecraft:stone",
                false,
                &ore_config,
                0.0,
            ),
            Some(super::ScatteredOreAttempt {
                pos: BlockPos {
                    x: 12,
                    y: 18,
                    z: 30,
                },
                state: "minecraft:iron_ore",
            })
        );
        let ore_spheres =
            super::ore_vein_spheres(BlockPos { x: 8, y: 32, z: 8 }, 8, 0.0, &[(2, 2)], &[1.0; 8]);
        assert!(!ore_spheres.is_empty());
        assert!(ore_spheres
            .iter()
            .any(|sphere| (sphere.center_z - 8.5).abs() < f64::EPSILON));
        let ore_candidates =
            super::ore_vein_position_candidates(&ore_spheres, 6, 28, 6, 6, 6, 0..384);
        assert!(!ore_candidates.is_empty());
        let sampled_candidate = ore_candidates[0];
        assert_eq!(
            ore_candidates
                .iter()
                .filter(|pos| **pos == sampled_candidate)
                .count(),
            1
        );
        let ore_plan = super::ore_placement_plan(
            &ore_config,
            &[
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 32, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: false,
                    air_check_roll: 0.0,
                },
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 33, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: true,
                    air_check_roll: 0.0,
                },
                super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 34, z: 8 },
                    current_block: "minecraft:dirt",
                    adjacent_to_air: false,
                    air_check_roll: 1.0,
                },
            ],
        );
        assert_eq!(
            ore_plan,
            vec![super::OrePlacementBlock {
                pos: BlockPos { x: 8, y: 32, z: 8 },
                state: "minecraft:iron_ore",
            }]
        );
        assert_eq!(
            super::aquatic_feature_offset(
                BlockPos {
                    x: 20,
                    y: 60,
                    z: 30
                },
                (7, 3),
                (1, 6)
            ),
            (24, 25)
        );
        assert_eq!(
            super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:water",
                true,
                0.7,
                0.1,
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 62, z: 1 },
                    state: "minecraft:tall_seagrass",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 63, z: 1 },
                    state: "minecraft:tall_seagrass[half=upper]",
                },
            ]
        );
        assert_eq!(
            super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:air",
                true,
                0.7,
                0.1,
            ),
            Vec::new()
        );
        assert_eq!(
            super::sea_pickle_placement_plan(
                BlockPos { x: 2, y: 61, z: 2 },
                "minecraft:water",
                true,
                2,
            ),
            Some(super::AquaticPlacementBlock {
                pos: BlockPos { x: 2, y: 61, z: 2 },
                state: "minecraft:sea_pickle[pickles=3]",
            })
        );
        assert_eq!(
            super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, true, true],
                &[true, true, true],
                1,
                &[2],
                false,
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 50, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 51, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 52, z: 3 },
                    state: "minecraft:kelp[age=22]",
                },
            ]
        );
        assert_eq!(
            super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, false],
                &[true, true],
                5,
                &[0],
                false,
            ),
            vec![super::AquaticPlacementBlock {
                pos: BlockPos { x: 3, y: 50, z: 3 },
                state: "minecraft:kelp[age=20]",
            }]
        );
        assert_eq!(
            super::coral_block_placement_plan(
                BlockPos { x: 4, y: 55, z: 4 },
                "minecraft:water",
                "minecraft:water",
                "minecraft:brain_coral_block",
                0.9,
                0.01,
                1,
                &[(super::HorizontalDirection::East, 0.1, true)],
            ),
            vec![
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 55, z: 4 },
                    state: "minecraft:brain_coral_block",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 56, z: 4 },
                    state: "minecraft:sea_pickle[pickles=2]",
                },
                super::AquaticPlacementBlock {
                    pos: BlockPos { x: 5, y: 55, z: 4 },
                    state: "minecraft:tube_coral_wall_fan[facing=east]",
                },
            ]
        );
        assert!(super::coral_block_placement_plan(
            BlockPos { x: 4, y: 55, z: 4 },
            "minecraft:stone",
            "minecraft:water",
            "minecraft:brain_coral_block",
            0.0,
            0.0,
            0,
            &[],
        )
        .is_empty());
        let coral_tree = super::coral_tree_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            1,
            &[
                super::HorizontalDirection::North,
                super::HorizontalDirection::East,
            ],
            &[0, 1],
            &[1.0; 10],
        );
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 62, z: -1 }));
        assert!(coral_tree.contains(&BlockPos { x: 1, y: 62, z: 0 }));
        let coral_mushroom = super::coral_mushroom_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            0,
            0,
            0,
            0,
            &[1.0; 128],
        );
        assert!(coral_mushroom.contains(&BlockPos { x: 1, y: 59, z: 1 }));
        assert!(!coral_mushroom.contains(&BlockPos { x: 0, y: 59, z: 0 }));
        let coral_claw = super::coral_claw_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            super::HorizontalDirection::North,
            &[
                super::HorizontalDirection::North,
                super::HorizontalDirection::East,
            ],
            &[0, 0],
            &[0, 0],
            &[1.0; 10],
        );
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: -1 }));
        assert!(coral_claw.contains(&BlockPos { x: 1, y: 61, z: 0 }));
        let vegetation_config = super::VegetationPatchConfigurationModel {
            replaceable: &["minecraft:dirt", "minecraft:grass_block"],
            ground_state: BlockStateProviderModel::Simple("minecraft:moss_block"),
            vegetation_feature: "minecraft:patch_grass",
            surface: CaveSurface::Floor,
            depth_min: 1,
            depth_max: 2,
            extra_bottom_block_chance: 0.5,
            vertical_range: 5,
            vegetation_chance: 0.75,
            xz_radius_min: 1,
            xz_radius_max: 2,
            extra_edge_column_chance: 0.25,
        };
        assert_eq!(super::vegetation_patch_radius(1, 2, 1), 3);
        assert!(!super::vegetation_patch_should_try_column(
            3, 3, 3, 3, 1.0, 0.0
        ));
        assert!(!super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.5
        ));
        assert!(super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.25
        ));
        assert_eq!(super::vegetation_patch_depth(1, 2, 0, 0.5, 0.25), 2);
        assert_eq!(
            super::vegetation_patch_place_ground(
                &vegetation_config,
                BlockPos { x: 5, y: 63, z: 5 },
                &["minecraft:dirt", "minecraft:stone"],
                3,
                0,
            ),
            Some(vec![super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }])
        );
        let vegetation_plan = super::vegetation_patch_plan(
            &vegetation_config,
            &[super::VegetationPatchGroundColumn {
                surface_pos: BlockPos { x: 5, y: 64, z: 5 },
                ground_start: BlockPos { x: 5, y: 63, z: 5 },
                depth: 1,
            }],
            &[&["minecraft:dirt"]],
            &[0.25],
        );
        assert_eq!(
            vegetation_plan.ground,
            vec![super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }]
        );
        assert_eq!(
            vegetation_plan.vegetation_origins,
            vec![BlockPos { x: 5, y: 65, z: 5 }]
        );
        let lake_config = super::LakeConfigurationModel {
            fluid: BlockStateProviderModel::Simple("minecraft:water"),
            barrier: BlockStateProviderModel::Simple("minecraft:stone"),
        };
        let mut lake_grid = vec![false; 2048];
        lake_grid[super::lake_grid_index(8, 3, 8)] = true;
        assert!(super::lake_is_boundary(&lake_grid, 8, 4, 8));
        assert_eq!(
            super::lake_grid_index(8, 3, 8),
            ((8 * 16 + 8) * 8 + 3) as usize
        );
        let lake_boundary = [
            super::LakeBoundaryBlock {
                x: 8,
                y: 4,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: true,
            },
            super::LakeBoundaryBlock {
                x: 8,
                y: 2,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: false,
            },
        ];
        assert!(super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &lake_boundary,
            "minecraft:water"
        ));
        let invalid_lake_boundary = [super::LakeBoundaryBlock {
            x: 8,
            y: 4,
            z: 8,
            state: "minecraft:water",
            solid: false,
            liquid: true,
            cannot_replace: false,
            should_freeze: false,
        }];
        assert!(!super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &invalid_lake_boundary,
            "minecraft:water"
        ));
        let lake_plan = super::lake_placement_plan(
            BlockPos { x: 0, y: 60, z: 0 },
            &lake_config,
            &lake_grid,
            &lake_boundary,
            &[1; 2048],
            true,
        )
        .unwrap();
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 63, z: 8 },
            state: "minecraft:water",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:stone",
            schedule_tick: false,
            mark_above_for_post_processing: true,
        }));
        assert!(lake_plan.contains(&super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:ice",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        let fossil_config = super::FossilFeatureConfigurationModel {
            fossil_structures: vec!["minecraft:fossil/spine_1", "minecraft:fossil/skull_1"],
            overlay_structures: vec![
                "minecraft:fossil/spine_1_coal",
                "minecraft:fossil/skull_1_coal",
            ],
            fossil_processors: "minecraft:fossil_rot",
            overlay_processors: "minecraft:fossil_coal",
            max_empty_corners_allowed: 4,
        };
        assert_eq!(super::validate_fossil_config(&fossil_config), Ok(()));
        assert_eq!(
            super::validate_fossil_config(&super::FossilFeatureConfigurationModel {
                fossil_structures: Vec::new(),
                overlay_structures: Vec::new(),
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
                max_empty_corners_allowed: 4,
            }),
            Err("Fossil structure lists need at least one entry")
        );
        assert_eq!(
            super::fossil_rotation(3),
            super::StructureRotation::Counterclockwise90
        );
        assert_eq!(super::fossil_target_y(50, -64, 9), 26);
        assert_eq!(
            super::fossil_low_corner(
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8
            ),
            BlockPos {
                x: 94,
                y: 40,
                z: 196
            }
        );
        assert_eq!(
            super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                4,
            ),
            Some(super::FossilPlacementPlan {
                fossil_structure: "minecraft:fossil/skull_1",
                overlay_structure: "minecraft:fossil/skull_1_coal",
                rotation: super::StructureRotation::Clockwise90,
                target_pos: BlockPos {
                    x: 94,
                    y: 35,
                    z: 196
                },
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
            })
        );
        assert_eq!(
            super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                5,
            ),
            None
        );
        let geode_config = super::GeodeConfigurationModel {
            filling_provider: BlockStateProviderModel::Simple("minecraft:air"),
            inner_layer_provider: BlockStateProviderModel::Simple("minecraft:amethyst_block"),
            alternate_inner_layer_provider: BlockStateProviderModel::Simple(
                "minecraft:budding_amethyst",
            ),
            middle_layer_provider: BlockStateProviderModel::Simple("minecraft:calcite"),
            outer_layer_provider: BlockStateProviderModel::Simple("minecraft:smooth_basalt"),
            inner_placements: &[
                "minecraft:small_amethyst_bud",
                "minecraft:medium_amethyst_bud",
                "minecraft:large_amethyst_bud",
                "minecraft:amethyst_cluster",
            ],
            cannot_replace: &["minecraft:bedrock"],
            invalid_blocks: &["minecraft:water", "minecraft:lava"],
            layers: super::GeodeLayerSettingsModel {
                filling: 1.7,
                inner_layer: 2.2,
                middle_layer: 3.2,
                outer_layer: 4.2,
            },
            crack: super::GeodeCrackSettingsModel {
                generate_crack_chance: 0.95,
                base_crack_size: 2.0,
                crack_point_offset: 2,
            },
            use_potential_placements_chance: 0.35,
            use_alternate_layer0_chance: 0.083,
            placements_require_layer0_alternate: true,
            outer_wall_distance_max: 6,
            invalid_blocks_threshold: 1,
        };
        assert_eq!(super::validate_geode_config(&geode_config), Ok(()));
        assert!(super::geode_can_place(
            &geode_config,
            &["minecraft:stone", "minecraft:water"]
        ));
        assert!(!super::geode_can_place(
            &geode_config,
            &["minecraft:air", "minecraft:water"]
        ));
        let geode_thresholds = super::geode_layer_thresholds(
            geode_config.layers,
            geode_config.crack,
            4,
            geode_config.outer_wall_distance_max,
            0.0,
        );
        assert!(geode_thresholds.inner_air > geode_thresholds.innermost_block_layer);
        assert!(geode_thresholds.innermost_block_layer > geode_thresholds.inner_crust);
        assert!(geode_thresholds.inner_crust > geode_thresholds.outer_crust);
        assert!(super::geode_should_generate_crack(geode_config.crack, 0.94));
        let crack_points = super::geode_crack_points(BlockPos { x: 0, y: 0, z: 0 }, 4, 2);
        assert_eq!(crack_points[0], BlockPos { x: 9, y: 7, z: 9 });
        let distribution_points = [super::GeodeDistributionPoint {
            pos: BlockPos { x: 0, y: 0, z: 0 },
            offset: 1,
        }];
        assert!(
            super::geode_shell_density(BlockPos { x: 0, y: 0, z: 0 }, &distribution_points, 0.0)
                > super::geode_shell_density(
                    BlockPos { x: 16, y: 0, z: 0 },
                    &distribution_points,
                    0.0
                )
        );
        assert_eq!(
            super::geode_layer_for_density(
                geode_thresholds.inner_air + 0.1,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::Filling)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.innermost_block_layer + geode_thresholds.inner_air) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.0,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::AlternateInner)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::Outer)
        );
        assert_eq!(
            super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                geode_thresholds.crack_size,
                geode_thresholds,
                true,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::GeodeLayer::CrackAir)
        );
        assert_eq!(
            super::geode_placement_block(
                &geode_config,
                BlockPos { x: 1, y: 2, z: 3 },
                super::GeodeLayer::AlternateInner,
                0,
                0.0,
            ),
            Some(super::GeodePlacementBlock {
                pos: BlockPos { x: 1, y: 2, z: 3 },
                state: "minecraft:budding_amethyst",
                layer: super::GeodeLayer::AlternateInner,
                potential_crystal_source: true,
            })
        );
        assert_eq!(
            super::geode_inner_placement(&geode_config, 3),
            Some("minecraft:amethyst_cluster")
        );
        let iceberg_shape = super::iceberg_shape_model(0.8, 0.25, 0, 2, 0.8, 5, 0.0, 0, 10, 6, 0);
        assert_eq!(iceberg_shape.shape_ellipse_a, 11);
        assert_eq!(iceberg_shape.shape_ellipse_c, 5);
        assert!(iceberg_shape.is_ellipse);
        assert_eq!(iceberg_shape.over_water_height, 11);
        assert_eq!(iceberg_shape.under_water_height, 18);
        assert_eq!(iceberg_shape.width, 11);
        assert_eq!(super::iceberg_ellipse_c(9, 11, 5), 3);
        assert!(
            super::iceberg_signed_distance_circle(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 5, 0.5,)
                < 0.0
        );
        assert!(
            super::iceberg_signed_distance_ellipse(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 11, 5, 0.0,)
                < 0.0
        );
        assert_eq!(super::iceberg_height_radius_ellipse(0, 11, 11), 6);
        assert_eq!(super::iceberg_height_radius_steep(1, 11, 11, 0.0), 5);
        assert!(super::iceberg_height_radius_round(0, 11, 11, 0.5, 0, 0) > 0);
        assert_eq!(
            super::iceberg_set_block_action("minecraft:air", 1, 11, true, true, 0, 0.1,),
            super::IcebergBlockAction::SnowBlock
        );
        assert_eq!(
            super::iceberg_set_block_action("minecraft:stone", 1, 11, true, true, 0, 0.1,),
            super::IcebergBlockAction::Keep
        );
        assert!(super::iceberg_should_skip_surface_noise(-0.25, true, 0.95));
        assert_eq!(
            super::iceberg_carve_action("minecraft:packed_ice", true),
            super::IcebergBlockAction::Water
        );
        assert_eq!(
            super::iceberg_carve_action("minecraft:blue_ice", false),
            super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::iceberg_smooth_action("minecraft:packed_ice", false, 3),
            super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::iceberg_smooth_action("minecraft:snow", true, 0),
            super::IcebergBlockAction::Air
        );
        let random_feature = super::RandomFeatureConfigurationModel {
            features: vec![
                super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_tulip",
                    chance: 0.2,
                },
                super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_grass",
                    chance: 0.5,
                },
            ],
            default_feature: "minecraft:flower_default",
        };
        assert_eq!(
            super::validate_weighted_placed_feature(random_feature.features[0]),
            Ok(random_feature.features[0])
        );
        assert_eq!(
            super::validate_weighted_placed_feature(super::WeightedPlacedFeatureModel {
                feature: "minecraft:bad",
                chance: 1.1,
            }),
            Err("weighted placed feature chance must be in 0.0..=1.0")
        );
        assert_eq!(
            super::random_selector_feature(&random_feature, &[0.3, 0.25]),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::random_selector_feature(&random_feature, &[0.3, 0.6]),
            Some("minecraft:flower_default")
        );
        let simple_random_feature = super::SimpleRandomFeatureConfigurationModel {
            features: vec![
                "minecraft:flower_plain",
                "minecraft:patch_grass",
                "minecraft:patch_sunflower",
            ],
        };
        assert_eq!(
            super::simple_random_selector_feature(&simple_random_feature, 4),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::simple_random_selector_feature(
                &super::SimpleRandomFeatureConfigurationModel { features: vec![] },
                0,
            ),
            None
        );
        let random_boolean_feature = super::RandomBooleanFeatureConfigurationModel {
            feature_true: "minecraft:flower_cherry",
            feature_false: "minecraft:patch_grass",
        };
        assert_eq!(
            super::random_boolean_selector_feature(random_boolean_feature, true),
            "minecraft:flower_cherry"
        );
        assert_eq!(
            super::random_boolean_selector_feature(random_boolean_feature, false),
            "minecraft:patch_grass"
        );
        let fill_layer_config = super::FillLayerConfigurationModel {
            height: 32,
            state: "minecraft:lava",
        };
        assert_eq!(
            super::validate_fill_layer_config(fill_layer_config, 384),
            Ok(fill_layer_config)
        );
        assert_eq!(
            super::validate_fill_layer_config(
                super::FillLayerConfigurationModel {
                    height: 385,
                    state: "minecraft:lava",
                },
                384,
            ),
            Err("fill layer height must be in 0..=dimension_y_size")
        );
        let mut fill_air = vec![false; 256];
        fill_air[0] = true;
        fill_air[17] = true;
        assert_eq!(
            super::fill_layer_placement_plan(
                BlockPos { x: 16, y: 0, z: 32 },
                -64,
                fill_layer_config,
                &fill_air,
            ),
            vec![
                BlockPos {
                    x: 16,
                    y: -32,
                    z: 32
                },
                BlockPos {
                    x: 17,
                    y: -32,
                    z: 33
                },
            ]
        );
        assert_eq!(super::end_island_layer_radius(4.0), 4);
        assert_eq!(super::end_island_next_size(4.0, 1), 2.5);
        let end_island =
            super::end_island_placement_plan(BlockPos { x: 0, y: 80, z: 0 }, 0, &[1, 1, 1]);
        assert!(end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 80, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 79, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(!end_island.contains(&super::EndIslandPlacementBlock {
            pos: BlockPos { x: 5, y: 80, z: 5 },
            state: "minecraft:end_stone",
        }));
        let replace_sphere = super::ReplaceSphereConfigurationModel {
            target_state: "minecraft:netherrack",
            replace_state: "minecraft:basalt",
            radius_min: 3,
            radius_max: 7,
        };
        assert_eq!(
            super::validate_replace_sphere_config(replace_sphere),
            Ok(replace_sphere)
        );
        assert_eq!(super::replace_sphere_radius(replace_sphere, 5), 3);
        assert_eq!(
            super::validate_replace_sphere_config(super::ReplaceSphereConfigurationModel {
                target_state: "minecraft:netherrack",
                replace_state: "minecraft:basalt",
                radius_min: 8,
                radius_max: 7,
            }),
            Err("replace sphere radius bounds must be ordered in 0..=12")
        );
        assert_eq!(
            super::replace_sphere_find_target(
                BlockPos { x: 4, y: 70, z: 8 },
                -64,
                320,
                &["minecraft:air", "minecraft:netherrack"],
                "minecraft:netherrack",
            ),
            Some(BlockPos { x: 4, y: 69, z: 8 })
        );
        let sphere_positions =
            super::replace_sphere_positions(BlockPos { x: 0, y: 0, z: 0 }, 1, 2, 3);
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: 0, z: 0 }));
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: -2, z: 0 }));
        assert!(!sphere_positions.contains(&BlockPos { x: 1, y: 2, z: 3 }));
        assert!(super::basalt_pillar_can_start(true, false));
        assert!(!super::basalt_pillar_can_start(true, true));
        assert!(super::basalt_pillar_hangoff_places(9));
        assert!(!super::basalt_pillar_hangoff_places(10));
        assert!(super::basalt_pillar_base_places(1, 2, 7));
        assert!(!super::basalt_pillar_base_places(3, 3, 1));
        let base_drop = vec![&[][..]; 49];
        let base_supported = vec![true; 49];
        let basalt_pillar = super::basalt_pillar_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &[true, true, false],
            &[false, false],
            &[(1, 1, 10, 1), (1, 10, 1, 1)],
            &[0; 49],
            &base_drop,
            &base_supported,
        );
        assert!(basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            kind: super::BasaltPillarBlockKind::Core,
        }));
        assert!(basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: -1 },
            kind: super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(!basalt_pillar.contains(&super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: -1, y: 64, z: 0 },
            kind: super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(basalt_pillar.iter().any(|block| {
            block.kind == super::BasaltPillarBlockKind::Base && block.pos.y == 62
        }));
        assert!(super::basalt_columns_cannot_place_on(
            "minecraft:magma_block"
        ));
        assert!(!super::basalt_columns_cannot_place_on(
            "minecraft:netherrack"
        ));
        assert!(super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            31,
            32,
        ));
        assert!(!super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            33,
            32,
        ));
        assert!(super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:netherrack",
            64,
            32,
        ));
        assert!(!super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:magma_block",
            64,
            32,
        ));
        let column_config = super::ColumnFeatureConfigurationModel {
            reach_min: 1,
            reach_max: 3,
            height_min: 5,
            height_max: 10,
        };
        assert_eq!(
            super::validate_column_feature_config(column_config),
            Ok(column_config)
        );
        assert_eq!(
            super::validate_column_feature_config(super::ColumnFeatureConfigurationModel {
                reach_min: 4,
                reach_max: 3,
                height_min: 5,
                height_max: 10,
            }),
            Err("column reach bounds must be ordered in 0..=3")
        );
        assert_eq!(
            super::basalt_columns_cluster_parameters(7, 0.5),
            (true, 5, 50)
        );
        assert_eq!(
            super::basalt_columns_cluster_parameters(7, 0.95),
            (false, 7, 15)
        );
        let column_blocks = super::basalt_column_blocks_from_surface(
            BlockPos { x: 1, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            4,
            3,
            &[true, true, false, true],
            &[false, true, false, false],
        );
        assert!(column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
        }));
        assert!(column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 65, z: 0 },
        }));
        assert!(!column_blocks.contains(&super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 67, z: 0 },
        }));
        let delta_config = super::DeltaFeatureConfigurationModel {
            contents: "minecraft:lava",
            rim: "minecraft:magma_block",
            size_min: 3,
            size_max: 7,
            rim_size_min: 0,
            rim_size_max: 2,
        };
        assert_eq!(super::validate_delta_config(delta_config), Ok(delta_config));
        assert_eq!(
            super::validate_delta_config(super::DeltaFeatureConfigurationModel {
                contents: "minecraft:lava",
                rim: "minecraft:magma_block",
                size_min: 17,
                size_max: 17,
                rim_size_min: 0,
                rim_size_max: 2,
            }),
            Err("delta size bounds must be ordered in 0..=16")
        );
        assert!(super::delta_cannot_replace("minecraft:bedrock"));
        assert!(!super::delta_cannot_replace("minecraft:netherrack"));
        assert!(super::delta_is_clear(
            "minecraft:netherrack",
            "minecraft:lava",
            false,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(!super::delta_is_clear(
            "minecraft:netherrack",
            "minecraft:lava",
            false,
            false,
            true,
            false,
            false,
            false,
        ));
        assert!(super::delta_has_rim(0.5, 1, 2));
        assert!(!super::delta_has_rim(0.95, 1, 2));
        let delta_offsets = super::delta_candidate_offsets(2, 1);
        assert!(delta_offsets.contains(&(0, 0)));
        assert!(delta_offsets.contains(&(2, 0)));
        assert!(!delta_offsets.contains(&(2, 1)));
        assert!(super::glowstone_can_start(true, "minecraft:netherrack"));
        assert!(!super::glowstone_can_start(true, "minecraft:air"));
        assert_eq!(
            super::glowstone_candidate_offset(7, 1, 11, 2, 6),
            BlockPos {
                x: 6,
                y: -11,
                z: -4
            }
        );
        assert!(super::glowstone_can_grow(true, 1));
        assert!(!super::glowstone_can_grow(true, 2));
        let nether_vegetation = super::NetherForestVegetationConfigModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:crimson_roots"),
            spread_width: 8,
            spread_height: 4,
        };
        assert_eq!(
            super::validate_nether_forest_vegetation_config(&nether_vegetation),
            Ok(())
        );
        assert!(super::nether_forest_vegetation_can_start(
            "minecraft:crimson_nylium",
            64,
            -64,
            320,
        ));
        assert!(!super::nether_forest_vegetation_can_start(
            "minecraft:netherrack",
            64,
            -64,
            320,
        ));
        assert_eq!(super::nether_forest_vegetation_attempts(8), 64);
        assert_eq!(
            super::nether_forest_vegetation_offset(8, 4, 7, 1, 3, 1, 2, 6),
            BlockPos { x: 6, y: 2, z: -4 }
        );
        assert!(super::twisting_vines_valid_ground(
            "minecraft:warped_nylium"
        ));
        assert!(super::weeping_vines_valid_ceiling(
            "minecraft:nether_wart_block"
        ));
        assert_eq!(super::vine_height(2, 8, 6, 1), 6);
        assert_eq!(super::vine_height(2, 8, 1, 5), 1);
        assert_eq!(super::vine_age(17, 25, 9), 17);
        let twisting_column = super::twisting_vines_column(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            &[true, true, true],
            &[false, true, false],
            1,
        );
        assert_eq!(
            twisting_column,
            vec![
                super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    state: "minecraft:twisting_vines_plant",
                    kind: super::VineColumnBlockKind::Plant,
                    age: None,
                },
                super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 65, z: 0 },
                    state: "minecraft:twisting_vines",
                    kind: super::VineColumnBlockKind::Head,
                    age: Some(18),
                },
            ]
        );
        let weeping_column = super::weeping_vines_column(
            BlockPos { x: 0, y: 70, z: 0 },
            2,
            &[true, true, true],
            &[false, false, true],
            2,
        );
        assert_eq!(
            weeping_column.last(),
            Some(&super::VineColumnBlock {
                pos: BlockPos { x: 0, y: 68, z: 0 },
                state: "minecraft:weeping_vines",
                kind: super::VineColumnBlockKind::Head,
                age: Some(19),
            })
        );
        assert!(super::weeping_vines_wart_can_grow(true, 1));
        assert!(!super::weeping_vines_wart_can_grow(true, 2));
        let end_platform = super::end_platform_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(end_platform.len(), 100);
        assert_eq!(
            end_platform
                .iter()
                .filter(|block| block.state == "minecraft:obsidian")
                .count(),
            25
        );
        assert!(end_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos {
                x: -2,
                y: 63,
                z: -2
            },
            state: "minecraft:obsidian",
        }));
        assert!(end_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 2, y: 66, z: 2 },
            state: "minecraft:air",
        }));
        assert_eq!(
            super::void_start_platform_origin(64),
            BlockPos { x: 8, y: 67, z: 8 }
        );
        assert!(super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 1,
            z: 1
        }));
        assert!(!super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 2,
            z: 0
        }));
        let void_platform = super::void_start_platform_blocks(ChunkPos { x: 0, z: 0 }, 64);
        assert!(void_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 8, y: 67, z: 8 },
            state: "minecraft:cobblestone",
        }));
        assert!(void_platform.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 67, z: 0 },
            state: "minecraft:stone",
        }));
        assert_eq!(
            super::void_start_platform_blocks(ChunkPos { x: 2, z: 0 }, 64),
            Vec::new()
        );
        assert_eq!(
            super::end_gateway_known_exit(BlockPos { x: 1, y: 2, z: 3 }, true),
            super::EndGatewayConfigurationModel {
                exit: Some(BlockPos { x: 1, y: 2, z: 3 }),
                exact: true,
            }
        );
        assert_eq!(
            super::end_gateway_delayed_exit_search(),
            super::EndGatewayConfigurationModel {
                exit: None,
                exact: false,
            }
        );
        let gateway = super::end_gateway_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(gateway.len(), 45);
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            state: "minecraft:end_gateway",
        }));
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            state: "minecraft:bedrock",
        }));
        assert!(gateway.contains(&super::FeaturePlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
            state: "minecraft:air",
        }));
        assert_eq!(
            gateway
                .iter()
                .filter(|block| block.state == "minecraft:bedrock")
                .count(),
            12
        );
        assert!(super::chorus_plant_can_start(true, "minecraft:end_stone"));
        assert!(!super::chorus_plant_can_start(false, "minecraft:end_stone"));
        assert!(!super::chorus_plant_can_start(true, "minecraft:stone"));
        assert!(super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            Some(1),
        ));
        assert!(!super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            None,
        ));
        assert!(super::chorus_branch_target_within_spread(
            BlockPos { x: 7, y: 68, z: -7 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert!(!super::chorus_branch_target_within_spread(
            BlockPos { x: 8, y: 68, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert_eq!(super::chorus_trunk_height(0, 0), 2);
        assert_eq!(super::chorus_trunk_height(1, 3), 4);
        assert_eq!(super::chorus_stem_attempts(0, 0), 1);
        assert_eq!(super::chorus_stem_attempts(1, 3), 3);
        let chorus_trunk =
            super::chorus_trunk_and_terminal_flower(BlockPos { x: 0, y: 64, z: 0 }, 0, 0, false);
        assert_eq!(
            chorus_trunk.last(),
            Some(&super::ChorusPlantPlacementBlock {
                pos: BlockPos { x: 0, y: 66, z: 0 },
                kind: super::ChorusPlantPlacementKind::Flower,
                age: Some(5),
            })
        );
        assert_eq!(
            chorus_trunk
                .iter()
                .filter(|block| block.kind == super::ChorusPlantPlacementKind::Plant)
                .count(),
            3
        );
        assert_eq!(
            super::end_podium_location(BlockPos { x: 1, y: 2, z: 3 }),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert!(super::end_podium_inside_rim(
            BlockPos { x: 2, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(!super::end_podium_inside_rim(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(super::end_podium_inside_body(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        let inactive_podium = super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, false);
        let active_podium = super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, true);
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 63, z: 0 },
            kind: super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 63, z: 0 },
            kind: super::EndPodiumBlockKind::EndStone,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 64, z: 0 },
            kind: super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: -1 },
            kind: super::EndPodiumBlockKind::WallTorch(super::HorizontalDirection::North),
        }));
        assert!(active_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::EndPodiumBlockKind::EndPortal,
        }));
        assert!(inactive_podium.contains(&super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::EndPodiumBlockKind::Air,
        }));
        assert_eq!(
            active_podium
                .iter()
                .filter(|block| block.kind
                    == super::EndPodiumBlockKind::WallTorch(super::HorizontalDirection::East))
                .count(),
            1
        );
        let spike = super::end_spike_from_size(0, 2);
        assert_eq!(
            spike,
            super::EndSpikeModel {
                center_x: 42,
                center_z: 0,
                radius: 2,
                height: 82,
                guarded: true,
            }
        );
        assert!(super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 32, y: 0, z: 0 },
        ));
        assert!(!super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 16, y: 0, z: 0 },
        ));
        assert_eq!(
            super::end_spike_top_bounding_box(spike, -64, 320),
            (
                BlockPos {
                    x: 40,
                    y: -64,
                    z: -2
                },
                BlockPos {
                    x: 44,
                    y: 320,
                    z: 2
                },
            )
        );
        let spike_blocks = super::end_spike_cylinder_and_air_blocks(spike, 64);
        assert!(spike_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 64, z: 0 },
            kind: super::EndSpikeBlockKind::Obsidian,
        }));
        assert!(spike_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos {
                x: 40,
                y: 82,
                z: -2
            },
            kind: super::EndSpikeBlockKind::Air,
        }));
        let cage_blocks = super::end_spike_guard_cage_blocks(spike);
        assert!(cage_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 40, y: 82, z: 0 },
            kind: super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: false,
                east: false,
            },
        }));
        assert!(cage_blocks.contains(&super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 85, z: 0 },
            kind: super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: true,
                east: true,
            },
        }));
        assert_eq!(
            super::end_spike_guard_cage_blocks(super::EndSpikeModel {
                guarded: false,
                ..spike
            }),
            Vec::new()
        );
        let spike_config = super::EndSpikeConfigurationModel {
            crystal_invulnerable: true,
            spikes: vec![spike],
            crystal_beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
        };
        assert_eq!(
            super::end_crystal_for_spike(spike, &spike_config, 0.25),
            super::EndCrystalPlacement {
                x: 42.5,
                y: 83.0,
                z: 0.5,
                yaw: 90.0,
                beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
                invulnerable: true,
            }
        );
        assert_eq!(
            super::end_spike_crystal_support_blocks(spike),
            vec![
                super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 82, z: 0 },
                    kind: super::EndSpikeBlockKind::Bedrock,
                },
                super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 83, z: 0 },
                    kind: super::EndSpikeBlockKind::Fire,
                },
            ]
        );
        let huge_fungus_config = super::HugeFungusConfigurationModel {
            valid_base_state: "minecraft:crimson_nylium",
            stem_state: "minecraft:crimson_stem",
            hat_state: "minecraft:nether_wart_block",
            decor_state: "minecraft:shroomlight",
            planted: false,
        };
        assert!(super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:crimson_nylium",
        ));
        assert!(!super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:warped_nylium",
        ));
        assert_eq!(super::huge_fungus_total_height(0, 1), 4);
        assert_eq!(super::huge_fungus_total_height(9, 0), 26);
        assert!(super::huge_fungus_fits_height(64, 10, 80, false));
        assert!(!super::huge_fungus_fits_height(64, 15, 80, false));
        assert!(super::huge_fungus_fits_height(64, 15, 80, true));
        assert!(super::huge_fungus_is_huge(false, 0.05));
        assert!(!super::huge_fungus_is_huge(true, 0.05));
        let stem_blocks = super::huge_fungus_stem_blocks(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            true,
            &[0.0, 1.0, 1.0, 1.0],
        );
        assert!(stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos {
                x: -1,
                y: 64,
                z: -1
            },
            kind: super::HugeFungusStemKind::CornerStem,
        }));
        assert!(!stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos { x: -1, y: 64, z: 1 },
            kind: super::HugeFungusStemKind::CornerStem,
        }));
        assert!(stem_blocks.contains(&super::HugeFungusStemBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            kind: super::HugeFungusStemKind::Stem,
        }));
        assert_eq!(super::huge_fungus_hat_height(12, 0), 5);
        assert_eq!(super::huge_fungus_hat_radius(7, 12, 5, false, 0), 2);
        assert_eq!(super::huge_fungus_hat_radius(10, 12, 5, false, 2), 1);
        assert_eq!(super::huge_fungus_hat_radius(4, 12, 9, true, 0), 4);
        let hat_cells =
            super::huge_fungus_hat_cells(BlockPos { x: 0, y: 64, z: 0 }, 8, 0, &[0; 9], false);
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Bottom));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Inside));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::HugeFungusHatRole::Corner));
        assert_eq!(
            super::huge_fungus_hat_drop_outcome(false, 0.1, 0, true),
            super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::huge_fungus_hat_drop_outcome(false, 0.2, 0, true),
            super::HugeFungusHatPlacement::None
        );
        assert_eq!(
            super::huge_fungus_hat_block_outcome(0.05, 0.0, 0.0, 0.1, 0.2, 0.1),
            super::HugeFungusHatPlacement::Decor
        );
        assert_eq!(
            super::huge_fungus_hat_block_outcome(0.2, 0.1, 0.05, 0.1, 0.2, 0.1),
            super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::huge_fungus_hat_probabilities(super::HugeFungusHatRole::Edge, true),
            Some((0.0005, 0.98, 0.07))
        );
        assert_eq!(super::huge_fungus_weeping_vine_height(4, 7), 10);

        let pile_config = super::BlockPileConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:hay_block"),
        };
        let all_shape_rolls = vec![(1.0, 0.0, 1.0); 7 * 7 * 2];
        let pile_positions = super::block_pile_placement_candidates(
            BlockPos { x: 0, y: 64, z: 0 },
            -64,
            1,
            1,
            &all_shape_rolls,
        );
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 64, z: 0 }));
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 65, z: 0 }));
        assert!(pile_positions.len() > 20);
        assert!(super::block_pile_placement_candidates(
            BlockPos { x: 0, y: -60, z: 0 },
            -64,
            0,
            0,
            &all_shape_rolls,
        )
        .is_empty());
        assert_eq!(
            super::block_pile_try_place(
                &pile_config,
                true,
                "minecraft:grass_block",
                true,
                false,
                0,
            ),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", true, false, 0,),
            None
        );
        assert_eq!(
            super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", false, true, 0,),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::block_pile_try_place(
                &pile_config,
                false,
                "minecraft:grass_block",
                true,
                true,
                0,
            ),
            None
        );

        let disk_config = super::DiskConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:clay"),
            target: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt"],
            },
            radius: 1,
            half_height: 1,
        };
        let mut disk_contexts = Vec::new();
        for y in 63..=65 {
            disk_contexts.push((
                BlockPos { x: 0, y, z: 0 },
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
            ));
        }
        let disk = super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk.len(), 3);
        assert_eq!(disk[0].pos, BlockPos { x: 0, y: 65, z: 0 });
        assert!(disk[0].mark_above_for_post_processing);
        assert!(!disk[1].mark_above_for_post_processing);
        assert!(disk.iter().all(|block| block.state == "minecraft:clay"));

        disk_contexts[1].1.block = "minecraft:stone";
        let disk_with_gap = super::disk_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &disk_config,
            &disk_contexts,
            &[],
        );
        assert_eq!(disk_with_gap.len(), 2);
        assert!(disk_with_gap[0].mark_above_for_post_processing);
        assert!(disk_with_gap[1].mark_above_for_post_processing);

        let snow_plan = super::snow_and_freeze_placement_plan(
            BlockPos {
                x: 32,
                y: 0,
                z: -16,
            },
            &[
                super::SnowAndFreezeColumn {
                    x: 32,
                    z: -16,
                    motion_blocking_height: 70,
                    should_freeze: true,
                    should_snow: true,
                    below_has_snowy_property: true,
                },
                super::SnowAndFreezeColumn {
                    x: 33,
                    z: -16,
                    motion_blocking_height: 65,
                    should_freeze: false,
                    should_snow: true,
                    below_has_snowy_property: false,
                },
            ],
        );
        assert_eq!(
            snow_plan,
            vec![
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:ice",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 70,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 32,
                        y: 69,
                        z: -16
                    },
                    state: "minecraft:snowy=true",
                },
                super::SnowAndFreezePlacement {
                    pos: BlockPos {
                        x: 33,
                        y: 65,
                        z: -16
                    },
                    state: "minecraft:snow",
                },
            ]
        );

        let magma_config = super::UnderwaterMagmaConfigurationModel {
            floor_search_range: 12,
            placement_radius_around_floor: 1,
            placement_probability_per_valid_position: 0.5,
        };
        let valid_magma = super::UnderwaterMagmaCandidate {
            pos: BlockPos { x: 0, y: 62, z: 0 },
            block: "minecraft:stone",
            below_visible_from_above: false,
            horizontal_visible_from_outside: false,
        };
        assert!(super::underwater_magma_is_valid_placement(&valid_magma));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                block: "minecraft:water",
                ..valid_magma
            }
        ));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                below_visible_from_above: true,
                ..valid_magma
            }
        ));
        assert!(!super::underwater_magma_is_valid_placement(
            &super::UnderwaterMagmaCandidate {
                horizontal_visible_from_outside: true,
                ..valid_magma
            }
        ));
        assert_eq!(
            super::underwater_magma_placement_plan(
                BlockPos { x: 0, y: 70, z: 0 },
                Some(62),
                magma_config,
                &[valid_magma],
                &[0.0; 27],
            ),
            vec![BlockPos { x: 0, y: 62, z: 0 }]
        );
        assert!(super::underwater_magma_placement_plan(
            BlockPos { x: 0, y: 70, z: 0 },
            None,
            magma_config,
            &[valid_magma],
            &[0.0; 27],
        )
        .is_empty());
        assert_eq!(
            super::feature_size_type("two_layers_feature_size"),
            Some("minecraft:two_layers_feature_size")
        );
        assert_eq!(
            super::feature_size_type("minecraft:three_layers_feature_size"),
            Some("minecraft:three_layers_feature_size")
        );
        assert_eq!(super::feature_size_type("missing"), None);

        let two = FeatureSizeModel::TwoLayers {
            limit: 2,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: None,
        };
        assert_eq!(super::validate_feature_size(two), Ok(two));
        assert_eq!(super::feature_size_at_height(two, 7, 1), 0);
        assert_eq!(super::feature_size_at_height(two, 7, 2), 1);

        let three = FeatureSizeModel::ThreeLayers {
            limit: 1,
            upper_limit: 2,
            lower_size: 0,
            middle_size: 1,
            upper_size: 2,
            min_clipped_height: Some(80),
        };
        assert_eq!(super::validate_feature_size(three), Ok(three));
        assert_eq!(super::feature_size_at_height(three, 8, 0), 0);
        assert_eq!(super::feature_size_at_height(three, 8, 5), 1);
        assert_eq!(super::feature_size_at_height(three, 8, 6), 2);
        assert_eq!(
            super::validate_feature_size(FeatureSizeModel::TwoLayers {
                limit: 82,
                lower_size: 0,
                upper_size: 1,
                min_clipped_height: None,
            })
            .unwrap_err(),
            "feature size fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_feature_size(FeatureSizeModel::ThreeLayers {
                limit: 1,
                upper_limit: 1,
                lower_size: 0,
                middle_size: 1,
                upper_size: 2,
                min_clipped_height: Some(81),
            })
            .unwrap_err(),
            "min_clipped_height must be in 0..=80".to_string()
        );
        assert_eq!(
            super::tree_decorator_type("trunk_vine"),
            Some("minecraft:trunk_vine")
        );
        assert_eq!(
            super::tree_decorator_type("minecraft:attached_to_logs"),
            Some("minecraft:attached_to_logs")
        );
        assert_eq!(super::tree_decorator_type("missing"), None);
        assert_eq!(
            super::trunk_placer_type("straight_trunk_placer"),
            Some("minecraft:straight_trunk_placer")
        );
        assert_eq!(
            super::foliage_placer_type("minecraft:cherry_foliage_placer"),
            Some("minecraft:cherry_foliage_placer")
        );
        assert_eq!(
            super::root_placer_type("mangrove_root_placer"),
            Some("minecraft:mangrove_root_placer")
        );
        let straight_trunk = TrunkPlacerModel {
            base_height: 5,
            height_rand_a: 2,
            height_rand_b: 1,
            kind: TrunkPlacerKind::Straight,
        };
        assert_eq!(
            super::validate_trunk_placer(straight_trunk),
            Ok(straight_trunk)
        );
        assert_eq!(super::trunk_placer_height(straight_trunk, 1, 1), 7);
        assert!(super::tree_valid_pos("minecraft:air"));
        assert!(super::tree_valid_pos("minecraft:oak_leaves"));
        assert!(super::tree_valid_pos("minecraft:dandelion"));
        assert!(!super::tree_valid_pos("minecraft:stone"));
        let min_size = FeatureSizeModel::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: Some(3),
        };
        let free_row = ["minecraft:air"; 9];
        let vine_row = ["minecraft:vine"; 9];
        let stone_row = ["minecraft:stone"; 9];
        assert_eq!(
            super::tree_max_free_height(
                5,
                min_size,
                &[&free_row, &free_row, &free_row, &stone_row],
                true,
            ),
            1
        );
        assert_eq!(
            super::tree_max_free_height(5, min_size, &[&vine_row], false),
            -2
        );
        assert!(super::tree_can_place(
            BlockPos { x: 0, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row, &free_row, &free_row, &free_row, &free_row, &free_row, &free_row,],
            true,
        ));
        assert!(!super::tree_can_place(
            BlockPos { x: 0, y: -64, z: 0 },
            BlockPos { x: 0, y: -64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row],
            true,
        ));
        assert_eq!(
            super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 33,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            })
            .unwrap_err(),
            "trunk placer base fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 5,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Cherry {
                    branch_count_min: 1,
                    branch_count_max: 3,
                    branch_horizontal_length_min: 2,
                    branch_horizontal_length_max: 16,
                    branch_start_offset_from_top_min: -1,
                    branch_start_offset_from_top_max: -1,
                    branch_end_offset_from_top_min: -16,
                    branch_end_offset_from_top_max: 16,
                },
            })
            .unwrap_err(),
            "trunk placer variant fields are outside vanilla codec ranges".to_string()
        );
        let blob_foliage = FoliagePlacerModel {
            radius_min: 1,
            radius_max: 2,
            offset_min: 0,
            offset_max: 1,
            kind: FoliagePlacerKind::Blob { height: 3 },
        };
        assert_eq!(
            super::validate_foliage_placer(blob_foliage),
            Ok(blob_foliage)
        );
        assert_eq!(
            super::validate_foliage_placer(FoliagePlacerModel {
                radius_min: 0,
                radius_max: 16,
                offset_min: 0,
                offset_max: 16,
                kind: FoliagePlacerKind::RandomSpread {
                    foliage_height_min: 0,
                    foliage_height_max: 1,
                    leaf_placement_attempts: 1,
                },
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        let mangrove_root = RootPlacerModel {
            above_root_placement_chance: Some(0.5),
            mangrove_root_placement: MangroveRootPlacementModel {
                max_root_width: 8,
                max_root_length: 15,
                random_skew_chance: 0.2,
            },
        };
        assert_eq!(
            super::validate_root_placer(mangrove_root),
            Ok(mangrove_root)
        );
        assert_eq!(
            super::validate_root_placer(RootPlacerModel {
                above_root_placement_chance: Some(1.25),
                mangrove_root_placement: MangroveRootPlacementModel {
                    max_root_width: 8,
                    max_root_length: 15,
                    random_skew_chance: 0.2,
                },
            })
            .unwrap_err(),
            "root placer fields are outside vanilla codec ranges".to_string()
        );
        let tree_plan = super::simple_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            straight_trunk,
            blob_foliage,
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            1,
            1,
        )
        .unwrap();
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos == BlockPos { x: 8, y: 63, z: 8 }
                && block.state == "minecraft:dirt"
        }));
        assert_eq!(
            tree_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Log)
                .count(),
            7
        );
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 8, y: 71, z: 8 }
                && block.state == "minecraft:oak_leaves"
        }));
        assert_eq!(
            super::simple_tree_placement_plan(
                BlockPos { x: 8, y: 64, z: 8 },
                TrunkPlacerModel {
                    base_height: 5,
                    height_rand_a: 0,
                    height_rand_b: 0,
                    kind: TrunkPlacerKind::Forking,
                },
                blob_foliage,
                "minecraft:oak_log",
                "minecraft:oak_leaves",
                "minecraft:dirt",
                0,
                0,
            )
            .unwrap_err(),
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string()
        );
        let fallen_config = super::FallenTreeConfigurationModel {
            trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
            min_log_length: 4,
            max_log_length: 7,
            stump_decorators: vec![TreeDecoratorModel::TrunkVine],
            log_decorators: vec![TreeDecoratorModel::AttachedToLogs { probability: 0.1 }],
        };
        assert_eq!(super::fallen_tree_log_length(4, 7, 0), 2);
        assert_eq!(
            super::fallen_tree_start_pos(
                BlockPos { x: 0, y: 64, z: 0 },
                super::HorizontalDirection::East,
                1,
                &[false, true],
            ),
            Some(BlockPos { x: 3, y: 64, z: 0 })
        );
        assert!(super::fallen_tree_can_place_log(
            &[true, true, true, true],
            &[true, false, false, true],
        ));
        assert!(!super::fallen_tree_can_place_log(
            &[true, true, true],
            &[false, false, false],
        ));
        let fallen_plan = super::fallen_tree_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &fallen_config,
            super::HorizontalDirection::East,
            1,
            0,
            &[true],
            &[true, true, true],
            &[true, true, true],
        )
        .unwrap();
        assert_eq!(fallen_plan.stump_decorators, 1);
        assert_eq!(fallen_plan.log_decorators, 1);
        assert_eq!(
            fallen_plan.blocks[0],
            super::FallenTreeBlock {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                state: "minecraft:oak_log",
                mark_above_for_post_processing: true,
            }
        );
        assert!(fallen_plan.blocks.contains(&super::FallenTreeBlock {
            pos: BlockPos { x: 2, y: 65, z: 0 },
            state: "minecraft:oak_log[axis=x]",
            mark_above_for_post_processing: true,
        }));
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::Cocoa { probability: 0.25 }),
            Ok(TreeDecoratorModel::Cocoa { probability: 0.25 })
        );
        assert_eq!(
            super::validate_tree_decorator(TreeDecoratorModel::Beehive { probability: 1.5 })
                .unwrap_err(),
            "tree decorator probability must be in 0.0..=1.0".to_string()
        );
        assert!(super::tree_decorator_should_place(0.25, 0.249));
        assert!(!super::tree_decorator_should_place(0.25, 0.25));
    }

    #[test]
    fn feature_placement_behavior_models_cover_required_families() {
        assert_eq!(
            FEATURE_BEHAVIOR_MODELS
                .iter()
                .map(|model| model.feature_type)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:tree",
                "minecraft:vegetation_patch",
                "minecraft:spring_feature",
                "minecraft:ore",
                "minecraft:scattered_ore",
                "minecraft:disk",
                "minecraft:lake",
                "minecraft:geode",
                "minecraft:fossil",
                "minecraft:monster_room",
            ]
        );

        assert!(super::spring_feature_can_place(
            true, true, true, true, 4, 1, 4, 1
        ));
        assert!(!super::spring_feature_can_place(
            true, true, false, true, 4, 1, 4, 1
        ));
        assert!(!super::spring_feature_can_place(
            true, false, false, true, 3, 1, 4, 1
        ));
        let spring_config = super::SpringConfigurationModel {
            state: "minecraft:water",
            requires_block_below: true,
            rock_count: 4,
            hole_count: 1,
            valid_blocks: &["minecraft:stone", "minecraft:dirt"],
        };
        assert_eq!(
            super::spring_placement_plan(
                &spring_config,
                super::SpringPlacementContext {
                    origin: BlockPos { x: 4, y: 32, z: 4 },
                    above_block: "minecraft:stone",
                    below_block: "minecraft:stone",
                    current_block: "minecraft:air",
                    west_block: "minecraft:stone",
                    east_block: "minecraft:stone",
                    north_block: "minecraft:air",
                    south_block: "minecraft:dirt",
                },
            ),
            Some(super::SpringPlacementPlan {
                pos: BlockPos { x: 4, y: 32, z: 4 },
                state: "minecraft:water",
                schedule_tick: true,
            })
        );
        assert_eq!(
            super::spring_placement_plan(
                &spring_config,
                super::SpringPlacementContext {
                    origin: BlockPos { x: 4, y: 32, z: 4 },
                    above_block: "minecraft:stone",
                    below_block: "minecraft:air",
                    current_block: "minecraft:air",
                    west_block: "minecraft:stone",
                    east_block: "minecraft:stone",
                    north_block: "minecraft:air",
                    south_block: "minecraft:dirt",
                },
            ),
            None
        );
        assert_eq!(MONSTER_ROOM_BOUNDS.min_y, -1);
        assert_eq!(MONSTER_ROOM_BOUNDS.max_y, 4);
        assert!(!super::monster_room_opening_count_is_valid(0));
        assert!(super::monster_room_opening_count_is_valid(1));
        assert!(super::monster_room_opening_count_is_valid(5));
        assert!(!super::monster_room_opening_count_is_valid(6));
        let room_radii = super::monster_room_radii(0, 1);
        assert_eq!(
            room_radii,
            super::MonsterRoomRadii {
                x_radius: 2,
                z_radius: 3,
            }
        );
        assert_eq!(
            super::monster_room_bounds_for_radius(room_radii.x_radius),
            (-3, 3)
        );
        let probes = [
            super::MonsterRoomProbe {
                dx: 0,
                dy: -1,
                dz: 0,
                solid: true,
                empty: false,
                above_empty: false,
            },
            super::MonsterRoomProbe {
                dx: 0,
                dy: 4,
                dz: 0,
                solid: true,
                empty: false,
                above_empty: false,
            },
            super::MonsterRoomProbe {
                dx: -3,
                dy: 0,
                dz: 0,
                solid: false,
                empty: true,
                above_empty: true,
            },
        ];
        assert_eq!(
            super::monster_room_opening_count(room_radii, &probes),
            Some(1)
        );
        assert!(super::monster_room_can_place(room_radii, &probes));
        let invalid_floor = [super::MonsterRoomProbe {
            dx: 0,
            dy: -1,
            dz: 0,
            solid: false,
            empty: true,
            above_empty: true,
        }];
        assert_eq!(
            super::monster_room_opening_count(room_radii, &invalid_floor),
            None
        );
        assert_eq!(
            super::monster_room_shell_state(-3, -1, 0, room_radii, 31, true, true, false, 1),
            Some("minecraft:mossy_cobblestone")
        );
        assert_eq!(
            super::monster_room_shell_state(-3, 0, 0, room_radii, 32, false, true, false, 0),
            Some("minecraft:cave_air")
        );
        assert_eq!(
            super::monster_room_shell_state(0, 0, 0, room_radii, 32, true, true, false, 0),
            Some("minecraft:cave_air")
        );
        assert!(super::monster_room_chest_can_place(true, 1));
        assert!(!super::monster_room_chest_can_place(true, 2));
        assert_eq!(super::monster_room_spawner_mob(0), "minecraft:skeleton");
        assert_eq!(super::monster_room_spawner_mob(1), "minecraft:zombie");
        assert_eq!(super::monster_room_spawner_mob(2), "minecraft:zombie");
        assert_eq!(super::monster_room_spawner_mob(3), "minecraft:spider");
        assert!(super::ore_vein_sphere_is_shadowed(3.0, 1.0, 1.0, 1.0));
        assert!(!super::ore_vein_sphere_is_shadowed(1.0, 2.0, 0.0, 0.0));
    }

    #[test]
    fn structure_registries_and_sets_match_vanilla_bootstrap() {
        assert_eq!(STRUCTURE_TYPES.len(), 16);
        assert_eq!(BUILTIN_STRUCTURES.len(), 34);
        assert_eq!(BUILTIN_STRUCTURE_SETS.len(), 20);
        assert_eq!(
            BUILTIN_STRUCTURE_SETS
                .iter()
                .map(|set| set.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:villages",
                "minecraft:desert_pyramids",
                "minecraft:igloos",
                "minecraft:jungle_temples",
                "minecraft:swamp_huts",
                "minecraft:pillager_outposts",
                "minecraft:ancient_cities",
                "minecraft:ocean_monuments",
                "minecraft:woodland_mansions",
                "minecraft:buried_treasures",
                "minecraft:mineshafts",
                "minecraft:ruined_portals",
                "minecraft:shipwrecks",
                "minecraft:ocean_ruins",
                "minecraft:nether_complexes",
                "minecraft:nether_fossils",
                "minecraft:end_cities",
                "minecraft:strongholds",
                "minecraft:trail_ruins",
                "minecraft:trial_chambers",
            ]
        );

        let villages = &BUILTIN_STRUCTURE_SETS[0];
        assert_eq!(villages.structures.len(), 5);
        assert_eq!(
            villages.placement,
            StructurePlacementKind::RandomSpread {
                spacing: 34,
                separation: 8,
                salt: 10387312,
                spread_type: RandomSpreadType::Linear,
            }
        );

        let strongholds = BUILTIN_STRUCTURE_SETS
            .iter()
            .find(|set| set.id == "minecraft:strongholds")
            .unwrap();
        assert_eq!(
            strongholds.placement,
            StructurePlacementKind::ConcentricRings {
                distance: 32,
                spread: 3,
                count: 128,
            }
        );

        let mansions = BUILTIN_STRUCTURE_SETS
            .iter()
            .find(|set| set.id == "minecraft:woodland_mansions")
            .unwrap();
        assert_eq!(
            mansions.placement,
            StructurePlacementKind::RandomSpread {
                spacing: 80,
                separation: 20,
                salt: 10387319,
                spread_type: RandomSpreadType::Triangular,
            }
        );
    }

    #[test]
    fn random_spread_structure_placement_uses_vanilla_grid_and_salt_math() {
        assert_eq!(
            super::validate_random_spread_placement(32, 32).unwrap_err(),
            "Spacing has to be larger than separation".to_string()
        );
        assert_eq!(
            super::validate_random_spread_placement(4097, 0).unwrap_err(),
            "Random spread spacing and separation must be in 0..=4096".to_string()
        );

        let village = super::random_spread_potential_structure_chunk(
            12345,
            0,
            0,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap();
        assert_eq!(village, ChunkPos { x: 21, z: 5 });
        assert!(!super::random_spread_is_placement_chunk(
            12345,
            0,
            0,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap());
        assert!(super::random_spread_is_placement_chunk(
            12345,
            village.x,
            village.z,
            34,
            8,
            10387312,
            RandomSpreadType::Linear,
        )
        .unwrap());

        assert_eq!(
            super::random_spread_potential_structure_chunk(
                12345,
                -1,
                -1,
                80,
                20,
                10387319,
                RandomSpreadType::Triangular,
            )
            .unwrap(),
            ChunkPos { x: -37, z: -54 }
        );
    }

    #[test]
    fn structure_frequency_reducers_match_vanilla_methods() {
        assert_eq!(super::FrequencyReductionMethod::Default.id(), "default");
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType1.id(),
            "legacy_type_1"
        );
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType2.id(),
            "legacy_type_2"
        );
        assert_eq!(
            super::FrequencyReductionMethod::LegacyType3.id(),
            "legacy_type_3"
        );
        assert_eq!(
            super::validate_structure_frequency(1.25).unwrap_err(),
            "Structure placement frequency must be in 0.0..=1.0".to_string()
        );

        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            1.0,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            0.0,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::Default,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType1,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType2,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
        assert!(!super::structure_frequency_reducer_should_generate(
            super::FrequencyReductionMethod::LegacyType3,
            12345,
            10387312,
            21,
            5,
            0.5,
        )
        .unwrap());
    }

    #[test]
    fn structure_locate_pos_uses_chunk_min_block_and_validated_offset() {
        assert_eq!(
            super::structure_locate_pos(ChunkPos { x: 21, z: -5 }, BlockPos { x: 8, y: 0, z: 8 })
                .unwrap(),
            BlockPos {
                x: 344,
                y: 0,
                z: -72
            }
        );
        assert_eq!(
            super::structure_locate_pos(
                ChunkPos { x: -2, z: 3 },
                BlockPos {
                    x: -16,
                    y: 16,
                    z: 16
                }
            )
            .unwrap(),
            BlockPos {
                x: -48,
                y: 16,
                z: 64
            }
        );
        assert_eq!(
            super::structure_locate_pos(ChunkPos { x: 0, z: 0 }, BlockPos { x: 17, y: 0, z: 0 })
                .unwrap_err(),
            "Structure locate offset components must be in -16..=16".to_string()
        );
    }

    #[test]
    fn concentric_rings_initial_candidates_follow_vanilla_ring_progression() {
        assert_eq!(
            super::validate_concentric_rings_placement(1024, 3, 128).unwrap_err(),
            "Concentric rings distance and spread must be in 0..=1023".to_string()
        );
        assert_eq!(
            super::validate_concentric_rings_placement(32, 3, 0).unwrap_err(),
            "Concentric rings count must be in 1..=4095".to_string()
        );

        let candidates = super::concentric_ring_initial_candidates(12345, 32, 3, 8).unwrap();
        assert_eq!(candidates.len(), 8);
        assert_eq!(
            candidates[0],
            super::ConcentricRingPlacementCandidate {
                index: 0,
                circle: 0,
                chunk_pos: ChunkPos { x: -105, z: 124 },
            }
        );
        assert_eq!(
            candidates[2],
            super::ConcentricRingPlacementCandidate {
                index: 2,
                circle: 0,
                chunk_pos: ChunkPos { x: 114, z: 21 },
            }
        );
        assert_eq!(candidates[3].circle, 1);
        let ring_positions = candidates
            .iter()
            .map(|candidate| candidate.chunk_pos)
            .collect::<Vec<_>>();
        assert!(super::concentric_rings_is_placement_chunk(
            &ring_positions,
            -105,
            124
        ));
        assert!(!super::concentric_rings_is_placement_chunk(
            &ring_positions,
            0,
            0
        ));
    }

    #[test]
    fn structure_exclusion_zone_checks_square_chunk_range() {
        let zone = super::StructureExclusionZoneModel {
            other_set: "minecraft:villages",
            chunk_count: 3,
        };
        assert_eq!(super::validate_structure_exclusion_zone(zone), Ok(zone));
        assert_eq!(
            super::validate_structure_exclusion_zone(super::StructureExclusionZoneModel {
                other_set: "minecraft:villages",
                chunk_count: 17,
            })
            .unwrap_err(),
            "Structure exclusion zone chunk_count must be in 1..=16".to_string()
        );

        let other_chunks = [
            ChunkPos { x: 10, z: -4 },
            ChunkPos { x: -12, z: 8 },
            ChunkPos { x: 40, z: 40 },
        ];
        assert!(super::structure_has_chunk_in_range(&other_chunks, 7, -1, 3));
        assert!(!super::structure_has_chunk_in_range(
            &other_chunks,
            6,
            -1,
            3
        ));
        assert!(super::structure_exclusion_zone_forbids(zone, &other_chunks, 7, -1).unwrap());
        assert!(!super::structure_exclusion_zone_forbids(zone, &other_chunks, 0, 0).unwrap());
    }

    #[test]
    fn structure_start_validity_references_tags_and_piece_queries_match_vanilla() {
        let invalid = super::StructureStartModel::invalid();
        assert!(!invalid.is_valid());
        assert_eq!(
            invalid.create_tag(ChunkPos { x: 4, z: -7 }),
            super::StructureStartTagModel {
                id: "INVALID",
                chunk_x: None,
                chunk_z: None,
                references: None,
                children: 0,
            }
        );

        let first_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 20,
                min_z: -16,
                max_x: 47,
                max_y: 35,
                max_z: -1,
            },
        };
        let second_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 48,
                min_y: 18,
                min_z: -8,
                max_x: 63,
                max_y: 30,
                max_z: 7,
            },
        };
        let mut start = super::StructureStartModel {
            structure: Some("minecraft:village_plains"),
            chunk_pos: ChunkPos { x: 2, z: -1 },
            references: 0,
            pieces: vec![first_piece, second_piece],
        };

        assert!(start.is_valid());
        assert!(start.can_be_referenced());
        start.add_reference();
        assert_eq!(start.references, 1);
        assert!(!start.can_be_referenced());
        assert_eq!(
            start.bounding_box(),
            Some(super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 18,
                min_z: -16,
                max_x: 63,
                max_y: 35,
                max_z: 7,
            })
        );
        assert_eq!(
            start.create_tag(ChunkPos { x: 2, z: -1 }),
            super::StructureStartTagModel {
                id: "minecraft:village_plains",
                chunk_x: Some(2),
                chunk_z: Some(-1),
                references: Some(1),
                children: 2,
            }
        );
        assert_eq!(
            super::structure_start_reference_pos(first_piece),
            BlockPos {
                x: 40,
                y: 20,
                z: -8,
            }
        );
        assert_eq!(
            super::structure_pieces_intersecting_chunk(
                &start,
                super::StructureBoundingBoxModel {
                    min_x: 48,
                    min_y: -64,
                    min_z: -16,
                    max_x: 63,
                    max_y: 320,
                    max_z: -1,
                }
            ),
            vec![second_piece]
        );
    }

    #[test]
    fn structure_manager_position_queries_use_union_and_piece_boxes() {
        let first_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 10,
                min_z: 0,
                max_x: 4,
                max_y: 20,
                max_z: 4,
            },
        };
        let second_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 10,
                min_z: 10,
                max_x: 14,
                max_y: 20,
                max_z: 14,
            },
        };
        let start = super::StructureStartModel {
            structure: Some("minecraft:stronghold"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![first_piece, second_piece],
        };
        let inside_first_edge = BlockPos { x: 4, y: 20, z: 4 };
        let inside_union_gap = BlockPos { x: 7, y: 15, z: 7 };
        let outside = BlockPos {
            x: 15,
            y: 15,
            z: 15,
        };

        assert!(first_piece.bounding_box.is_inside(inside_first_edge));
        assert!(super::structure_start_contains_pos(
            inside_first_edge,
            &start
        ));
        assert!(super::structure_has_piece_at(inside_first_edge, &start));

        assert!(super::structure_start_contains_pos(
            inside_union_gap,
            &start
        ));
        assert!(!super::structure_has_piece_at(inside_union_gap, &start));
        assert!(!super::structure_start_contains_pos(outside, &start));
        assert!(!super::structure_has_piece_at(outside, &start));

        let invalid = super::StructureStartModel::invalid();
        assert_eq!(
            super::first_structure_start_containing_pos(
                inside_union_gap,
                &[invalid.clone(), start.clone()]
            ),
            Some(start.clone())
        );
        assert_eq!(
            super::first_structure_start_with_piece_at(inside_union_gap, &[start.clone()]),
            None
        );
        assert_eq!(
            super::first_structure_start_with_piece_at(
                inside_first_edge,
                &[invalid, start.clone()]
            ),
            Some(start)
        );
    }

    #[test]
    fn structure_piece_bounding_box_and_world_coordinates_match_vanilla_orientation() {
        assert_eq!(
            super::structure_make_bounding_box(
                10,
                20,
                30,
                super::HorizontalDirection::South,
                3,
                4,
                5
            ),
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 12,
                max_y: 23,
                max_z: 34,
            }
        );
        assert_eq!(
            super::structure_make_bounding_box(
                10,
                20,
                30,
                super::HorizontalDirection::East,
                3,
                4,
                5
            ),
            super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 14,
                max_y: 23,
                max_z: 32,
            }
        );

        let foot = BlockPos {
            x: 100,
            y: 40,
            z: 200,
        };
        let offset = BlockPos { x: 2, y: 3, z: 4 };
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::North),
            super::StructureBoundingBoxModel {
                min_x: 102,
                min_y: 43,
                min_z: 198,
                max_x: 106,
                max_y: 48,
                max_z: 204,
            }
        );
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::West),
            super::StructureBoundingBoxModel {
                min_x: 98,
                min_y: 43,
                min_z: 202,
                max_x: 104,
                max_y: 48,
                max_z: 206,
            }
        );
        assert_eq!(
            super::structure_orient_box(foot, offset, 5, 6, 7, super::HorizontalDirection::East),
            super::StructureBoundingBoxModel {
                min_x: 104,
                min_y: 43,
                min_z: 202,
                max_x: 110,
                max_y: 48,
                max_z: 206,
            }
        );

        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 50,
            min_y: 60,
            min_z: 70,
            max_x: 59,
            max_y: 69,
            max_z: 79,
        };
        assert_eq!(
            super::structure_piece_world_pos(bounding_box, None, 1, 2, 3),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::North),
                1,
                2,
                3
            ),
            BlockPos {
                x: 51,
                y: 62,
                z: 76,
            }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::West),
                1,
                2,
                3
            ),
            BlockPos {
                x: 56,
                y: 62,
                z: 71,
            }
        );
        assert_eq!(
            super::structure_piece_world_pos(
                bounding_box,
                Some(super::HorizontalDirection::East),
                1,
                2,
                3
            ),
            BlockPos {
                x: 53,
                y: 62,
                z: 71,
            }
        );
    }

    #[test]
    fn structure_piece_chunk_proximity_and_locator_position_match_vanilla() {
        let even_sized_piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 20,
                min_z: -16,
                max_x: 47,
                max_y: 35,
                max_z: -1,
            },
        };
        assert_eq!(
            super::structure_piece_locator_position(even_sized_piece),
            BlockPos {
                x: 40,
                y: 28,
                z: -8,
            }
        );

        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 20,
                min_y: -10,
                min_z: 20,
                max_x: 25,
                max_y: 120,
                max_z: 25,
            },
        };
        assert!(super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 1, z: 1 },
            0
        ));
        assert!(!super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            0
        ));
        assert!(super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            7
        ));
    }

    #[test]
    fn structure_piece_box_generation_uses_vanilla_loop_edges_and_chunk_clipping() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let chunk_bb = super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: 20,
            min_z: 30,
            max_x: 12,
            max_y: 22,
            max_z: 32,
        };
        let blocks = super::structure_piece_generate_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 2, y: 2, z: 2 },
            "minecraft:cobblestone",
            "minecraft:mossy_cobblestone",
            false,
            |_| false,
        );

        assert_eq!(blocks.len(), 18);
        assert_eq!(
            blocks.first().copied(),
            Some(super::StructurePiecePlacementBlock {
                local_pos: BlockPos { x: 1, y: 0, z: 0 },
                world_pos: BlockPos {
                    x: 11,
                    y: 20,
                    z: 30,
                },
                state: "minecraft:cobblestone",
                edge: true,
            })
        );
        assert!(blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 1, y: 1, z: 1 }
                && block.world_pos
                    == BlockPos {
                        x: 11,
                        y: 21,
                        z: 31,
                    }
                && block.state == "minecraft:mossy_cobblestone"
                && !block.edge
        }));
        assert!(blocks
            .iter()
            .all(|block| chunk_bb.is_inside(block.world_pos)));
    }

    #[test]
    fn structure_piece_air_box_and_skip_air_match_vanilla_generation_rules() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 10,
            max_y: 74,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let air_blocks = super::structure_piece_generate_air_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
        );
        assert_eq!(air_blocks.len(), 8);
        assert!(air_blocks
            .iter()
            .all(|block| block.state == "minecraft:air" && block.edge));

        let skipped_world_pos = BlockPos { x: 1, y: 66, z: 3 };
        let blocks = super::structure_piece_generate_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            |world_pos| world_pos == skipped_world_pos,
        );
        assert_eq!(blocks.len(), 7);
        assert!(!blocks
            .iter()
            .any(|block| block.world_pos == skipped_world_pos));
    }

    #[test]
    fn structure_piece_maybe_box_and_single_block_use_vanilla_probability_edges() {
        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 10,
            max_y: 20,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let blocks = super::structure_piece_generate_maybe_box(
            bounding_box,
            Some(super::HorizontalDirection::South),
            chunk_bb,
            &[0.75, 0.76, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60],
            0.75,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 1, y: 1, z: 1 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            true,
            |world_pos| world_pos == BlockPos { x: 0, y: 10, z: 0 },
            |world_pos| world_pos.y >= 11,
        );

        assert_eq!(blocks.len(), 4);
        assert!(blocks
            .iter()
            .all(|block| block.state == "minecraft:stone_bricks" && block.edge));
        assert!(!blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 0, y: 0, z: 0 }
                || block.local_pos == BlockPos { x: 0, y: 0, z: 1 }
                || block.local_pos == BlockPos { x: 1, y: 0, z: 0 }
        }));

        assert_eq!(
            super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::HorizontalDirection::South),
                chunk_bb,
                0.49,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            )
            .map(|block| block.world_pos),
            Some(BlockPos { x: 2, y: 13, z: 4 })
        );
        assert_eq!(
            super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::HorizontalDirection::South),
                chunk_bb,
                0.5,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            ),
            None
        );
    }

    #[test]
    fn structure_check_presence_and_lookup_branches_match_vanilla() {
        assert_eq!(
            super::structure_check_result_from_cached_references(None, false),
            super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(0), true),
            super::StructureCheckResultModel::StartPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(1), true),
            super::StructureCheckResultModel::StartNotPresent
        );
        assert_eq!(
            super::structure_check_result_from_cached_references(Some(2), false),
            super::StructureCheckResultModel::StartPresent
        );

        assert!(!super::structure_fast_check_allows_lookup(
            super::StructureCheckResultModel::StartNotPresent
        ));
        assert!(super::structure_fast_check_allows_lookup(
            super::StructureCheckResultModel::ChunkLoadNeeded
        ));
        assert!(super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::StartPresent,
            false
        ));
        assert!(!super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::StartPresent,
            true
        ));
        assert!(!super::structure_locate_can_return_fast(
            super::StructureCheckResultModel::ChunkLoadNeeded,
            false
        ));

        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 20,
                min_z: 0,
                max_x: 15,
                max_y: 30,
                max_z: 15,
            },
        };
        let mut start = super::StructureStartModel {
            structure: Some("minecraft:desert_pyramid"),
            chunk_pos: ChunkPos { x: 0, z: 0 },
            references: 0,
            pieces: vec![piece],
        };
        assert!(super::structure_start_can_satisfy_lookup(&start, false));
        assert!(super::structure_start_can_satisfy_lookup(&start, true));
        assert!(super::structure_try_add_reference(&mut start));
        assert_eq!(start.references, 1);
        assert!(super::structure_start_can_satisfy_lookup(&start, false));
        assert!(!super::structure_start_can_satisfy_lookup(&start, true));
        assert!(!super::structure_try_add_reference(&mut start));

        let invalid = super::StructureStartModel::invalid();
        assert!(!super::structure_start_can_satisfy_lookup(&invalid, false));
    }

    #[test]
    fn terrain_adjustment_ids_and_bounding_boxes_match_vanilla() {
        let ids = [
            (super::TerrainAdjustmentModel::None, "none"),
            (super::TerrainAdjustmentModel::Bury, "bury"),
            (super::TerrainAdjustmentModel::BeardThin, "beard_thin"),
            (super::TerrainAdjustmentModel::BeardBox, "beard_box"),
            (super::TerrainAdjustmentModel::Encapsulate, "encapsulate"),
        ];
        for (adjustment, id) in ids {
            assert_eq!(adjustment.id(), id);
            assert_eq!(super::TerrainAdjustmentModel::from_id(id), Some(adjustment));
        }
        assert_eq!(super::TerrainAdjustmentModel::from_id("beard"), None);

        let bounding_box = super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 40,
            max_y: 50,
            max_z: 60,
        };
        assert_eq!(
            super::structure_adjust_bounding_box(super::TerrainAdjustmentModel::None, bounding_box),
            bounding_box
        );
        assert_eq!(
            super::structure_adjust_bounding_box(super::TerrainAdjustmentModel::Bury, bounding_box),
            super::StructureBoundingBoxModel {
                min_x: -2,
                min_y: 8,
                min_z: 18,
                max_x: 52,
                max_y: 62,
                max_z: 72,
            }
        );

        assert!(super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::TerrainAdjustmentModel::None
        ));
        assert!(!super::jigsaw_max_distance_with_terrain_is_valid(
            128,
            super::TerrainAdjustmentModel::Bury
        ));
        assert!(super::jigsaw_max_distance_with_terrain_is_valid(
            116,
            super::TerrainAdjustmentModel::Encapsulate
        ));
        assert!(!super::jigsaw_max_distance_with_terrain_is_valid(
            117,
            super::TerrainAdjustmentModel::BeardBox
        ));
    }

    #[test]
    fn structure_access_stores_starts_and_reference_sets_like_chunks() {
        let piece = super::StructurePieceModel {
            bounding_box: super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 64,
                min_z: 48,
                max_x: 47,
                max_y: 80,
                max_z: 63,
            },
        };
        let start = super::StructureStartModel {
            structure: Some("minecraft:shipwreck"),
            chunk_pos: ChunkPos { x: 2, z: 3 },
            references: 0,
            pieces: vec![piece],
        };
        let invalid = super::StructureStartModel::invalid();
        let mut access = super::StructureAccessModel::default();

        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[]
        );
        assert!(!access.has_any_structure_references());
        assert!(!access.unsaved);

        access.set_start_for_structure("minecraft:shipwreck", start.clone());
        access.set_start_for_structure("minecraft:mineshaft", invalid.clone());
        assert_eq!(
            access.get_start_for_structure("minecraft:shipwreck"),
            Some(&start)
        );
        assert!(access.unsaved);

        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        access.add_reference_for_structure("minecraft:shipwreck", 0x0000_0002_0000_0003);
        assert_eq!(
            access.get_references_for_structure("minecraft:shipwreck"),
            &[0x0000_0002_0000_0003]
        );
        assert!(access.has_any_structure_references());

        let mut replacement_starts = BTreeMap::new();
        replacement_starts.insert("minecraft:mineshaft", invalid.clone());
        access.set_all_starts(replacement_starts);
        assert!(access
            .get_start_for_structure("minecraft:shipwreck")
            .is_none());
        assert_eq!(
            access.get_start_for_structure("minecraft:mineshaft"),
            Some(&invalid)
        );

        let mut replacement_references = BTreeMap::new();
        replacement_references.insert("minecraft:mineshaft", vec![7]);
        access.set_all_references(replacement_references);
        assert_eq!(
            access.get_references_for_structure("minecraft:mineshaft"),
            &[7]
        );
    }

    #[test]
    fn structure_access_resolves_only_valid_referenced_starts() {
        let valid_start = super::StructureStartModel {
            structure: Some("minecraft:buried_treasure"),
            chunk_pos: ChunkPos { x: 1, z: 1 },
            references: 0,
            pieces: vec![super::StructurePieceModel {
                bounding_box: super::StructureBoundingBoxModel {
                    min_x: 16,
                    min_y: 45,
                    min_z: 16,
                    max_x: 31,
                    max_y: 55,
                    max_z: 31,
                },
            }],
        };
        let mut valid_access = super::StructureAccessModel::default();
        valid_access.set_start_for_structure("minecraft:buried_treasure", valid_start.clone());

        let mut invalid_access = super::StructureAccessModel::default();
        invalid_access.set_start_for_structure(
            "minecraft:buried_treasure",
            super::StructureStartModel::invalid(),
        );

        let mut chunks = BTreeMap::new();
        chunks.insert(11, valid_access);
        chunks.insert(12, invalid_access);
        let starts = super::structure_access_valid_starts_for_references(
            &chunks,
            "minecraft:buried_treasure",
            &[11, 12, 13],
        );
        assert_eq!(starts, vec![valid_start]);
    }

    #[test]
    fn structure_family_coverage_matches_builtin_structure_keys() {
        assert_eq!(STRUCTURE_FAMILIES.len(), 21);
        assert_eq!(
            STRUCTURE_FAMILIES
                .iter()
                .map(|entry| entry.structures.len())
                .sum::<usize>(),
            BUILTIN_STRUCTURES.len()
        );

        let villages = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::Village)
            .unwrap();
        assert_eq!(villages.structures.len(), 5);

        let ruined_portals = STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::RuinedPortal)
            .unwrap();
        assert_eq!(ruined_portals.structures.len(), 7);
        assert!(ruined_portals
            .structures
            .contains(&"minecraft:ruined_portal_nether"));

        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::OceanRuins)
            .unwrap()
            .structures
            .contains(&"minecraft:ocean_ruin_warm"));
        assert!(STRUCTURE_FAMILIES
            .iter()
            .find(|entry| entry.family == StructureFamily::TrialChambers)
            .unwrap()
            .structures
            .contains(&"minecraft:trial_chambers"));
    }

    #[test]
    fn jigsaw_and_processor_registries_match_vanilla_bootstrap_surface() {
        assert_eq!(STRUCTURE_POOL_ELEMENT_TYPES.len(), 5);
        assert_eq!(STRUCTURE_PROCESSOR_TYPES.len(), 11);
        assert_eq!(STRUCTURE_RULE_TEST_TYPES.len(), 6);
        assert_eq!(STRUCTURE_POS_RULE_TEST_TYPES.len(), 3);
        assert_eq!(STRUCTURE_PIECE_TYPES.len(), 56);
        assert_eq!(STRUCTURE_PROCESSOR_LISTS.len(), 40);
        assert_eq!(
            STRUCTURE_PROCESSOR_LISTS.first().copied(),
            Some("minecraft:empty")
        );
        assert_eq!(
            STRUCTURE_PROCESSOR_LISTS.last().copied(),
            Some("minecraft:trial_chambers_copper_bulb_degradation")
        );
        assert!(STRUCTURE_PROCESSOR_TYPES.contains(&"minecraft:jigsaw_replacement"));
        assert!(STRUCTURE_POOL_ELEMENT_TYPES.contains(&"minecraft:legacy_single_pool_element"));
        assert_eq!(STRUCTURE_PIECE_TYPES.first().copied(), Some("mscorridor"));
        assert_eq!(STRUCTURE_PIECE_TYPES.last().copied(), Some("jigsaw"));
        assert!(STRUCTURE_PIECE_TYPES.contains(&"shpr"));
        assert!(STRUCTURE_PIECE_TYPES.contains(&"shipwreck"));

        assert_eq!(JIGSAW_POOL_BOOTSTRAP_SOURCES.len(), 17);
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .map(|source| source.registrations)
                .sum::<usize>(),
            176
        );
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .find(|source| source.source_file == "TrialChambersStructurePools.java")
                .map(|source| source.registrations),
            Some(34)
        );
        assert_eq!(
            JIGSAW_POOL_BOOTSTRAP_SOURCES
                .iter()
                .find(|source| source.source_file == "PlainVillagePools.java")
                .map(|source| source.registrations),
            Some(17)
        );
    }

    #[test]
    fn terrain_blending_and_upgrade_data_match_vanilla_constants() {
        assert_eq!(BLENDING_CONSTANTS.height_blending_range_cells, 27);
        assert_eq!(BLENDING_CONSTANTS.height_blending_range_chunks, 7);
        assert_eq!(BLENDING_CONSTANTS.density_blending_range_cells, 2);
        assert_eq!(BLENDING_CONSTANTS.density_blending_range_chunks, 2);
        assert_eq!(BLENDING_CONSTANTS.old_chunk_xz_radius, 8);
        assert_eq!(BLENDING_CONSTANTS.cell_width, 4);
        assert_eq!(BLENDING_CONSTANTS.cell_height, 8);
        assert_eq!(BLENDING_CONSTANTS.cell_ratio, 2);
        assert_eq!(BLENDING_CELL_COLUMN_COUNT, 16);
        assert_eq!(BLENDING_NO_VALUE, f64::MAX);

        assert_eq!(super::blending_smooth_alpha(0.0, 27), 0.0);
        assert_eq!(super::blending_smooth_alpha(28.0, 27), 1.0);
        assert!((super::blending_smooth_alpha(14.0, 27) - 0.5).abs() < f64::EPSILON);
        assert!((super::blending_height_to_offset(127.5)).abs() < f64::EPSILON);
        assert!(super::blending_height_to_offset(63.5) < 0.0);
        assert!(super::blending_height_to_offset(191.5) > 0.0);
        assert_eq!(
            super::blending_output_for_old_height(None, None),
            BlendingOutput {
                alpha: 1.0,
                blending_offset: 0.0
            }
        );
        assert_eq!(
            super::blending_output_for_old_height(Some(127.5), None),
            BlendingOutput {
                alpha: 0.0,
                blending_offset: 0.0
            }
        );
        let blended = super::blending_output_for_old_height(Some(63.5), Some(14.0));
        assert!((blended.alpha - 0.5).abs() < f64::EPSILON);
        assert!(blended.blending_offset < 0.0);
        assert_eq!(
            super::validate_blending_data_packed(BlendingDataPacked {
                min_section: -4,
                max_section: 20,
                heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT])
            }),
            Ok(())
        );
        assert_eq!(
            super::validate_blending_data_packed(BlendingDataPacked {
                min_section: -4,
                max_section: 20,
                heights: Some(&[0.0; BLENDING_CELL_COLUMN_COUNT - 1])
            }),
            Err("heights has to be of length 16".to_string())
        );

        assert_eq!(UPGRADE_DATA_MODEL.tag_indices, "Indices");
        assert_eq!(UPGRADE_DATA_MODEL.tag_sides, "Sides");
        assert_eq!(
            UPGRADE_DATA_MODEL.tag_neighbor_block_ticks,
            "neighbor_block_ticks"
        );
        assert_eq!(
            UPGRADE_DATA_MODEL.tag_neighbor_fluid_ticks,
            "neighbor_fluid_ticks"
        );
        assert_eq!(
            UPGRADE_DATA_MODEL.block_fixers,
            &["blacklist", "default", "chest", "leaves", "stem_block"]
        );
        assert_eq!(UPGRADE_DATA_MODEL.chunky_fixers, &["leaves"]);
    }

    #[test]
    fn spawn_selection_constants_and_initial_positions_match_vanilla() {
        assert_eq!(SPAWN_SELECTION_CONSTANTS.initial_chunk_search_radius, 5);
        assert_eq!(SPAWN_SELECTION_CONSTANTS.player_spawn_ticket_radius, 3);
        assert_eq!(
            SPAWN_SELECTION_CONSTANTS.spawn_search_absolute_max_attempts,
            1024
        );
        assert_eq!(SPAWN_SELECTION_CONSTANTS.large_search_coprime, 17);

        assert_eq!(
            super::initial_spawn_position(true, true, false, 7, -3, 64, -64, 70),
            super::InitialSpawnKind::DebugHalfWorld {
                x: 0,
                y: 64,
                z: -100
            }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, true, 7, -3, 64, -64, 70),
            super::InitialSpawnKind::DebugWorld { x: 0, y: 80, z: 0 }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, false, 7, -3, 90, -64, 70),
            super::InitialSpawnKind::Normal {
                x: 120,
                y: 90,
                z: -40
            }
        );
        assert_eq!(
            super::initial_spawn_position(false, false, false, 7, -3, -80, -64, 70),
            super::InitialSpawnKind::Normal {
                x: 120,
                y: 70,
                z: -40
            }
        );
    }

    #[test]
    fn initial_spawn_chunk_spiral_matches_vanilla_search_order() {
        let offsets = super::initial_spawn_chunk_spiral_offsets();
        assert_eq!(offsets.len(), 121);
        assert_eq!(
            &offsets[..12],
            &[
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (2, 0),
                (2, 1)
            ]
        );
        assert_eq!(offsets.last(), Some(&(5, -5)));
        assert!(offsets.contains(&(-5, -5)));
        assert!(offsets.contains(&(5, 5)));
    }

    #[test]
    fn player_spawn_search_candidate_math_matches_vanilla() {
        assert_eq!(super::spawn_search_candidate_count(0), 1);
        assert_eq!(super::spawn_search_candidate_count(1), 9);
        assert_eq!(super::spawn_search_candidate_count(16), 1024);
        assert_eq!(super::spawn_search_coprime(9), 8);
        assert_eq!(super::spawn_search_coprime(17), 17);

        assert_eq!(super::spawn_search_radius(10, 20), 10);
        assert_eq!(super::spawn_search_radius(10, 4), 4);
        assert_eq!(super::spawn_search_radius(10, 1), 1);
        assert_eq!(super::spawn_search_radius(-5, 20), 0);

        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 0, 0),
            Some((99, 199))
        );
        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 0, 1),
            Some((101, 201))
        );
        assert_eq!(
            super::spawn_search_candidate(100, 200, 1, 8, 0),
            Some((101, 201))
        );
        assert_eq!(super::spawn_search_candidate(100, 200, 1, 0, 9), None);
    }

    #[test]
    fn overworld_respawn_candidate_rules_match_vanilla() {
        let normal_column = SpawnColumnHeights {
            top_y: 64,
            surface_y: 66,
            ocean_floor_y: 63,
            min_y: -64,
        };
        assert_eq!(
            super::overworld_respawn_y(
                normal_column,
                false,
                &[
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Solid
                ]
            ),
            Some(64)
        );

        assert_eq!(
            super::overworld_respawn_y(
                SpawnColumnHeights {
                    top_y: -80,
                    ..normal_column
                },
                false,
                &[SpawnBlockKind::Solid]
            ),
            None
        );
        assert_eq!(
            super::overworld_respawn_y(
                SpawnColumnHeights {
                    surface_y: 64,
                    ocean_floor_y: 62,
                    ..normal_column
                },
                false,
                &[SpawnBlockKind::Solid]
            ),
            None
        );
        assert_eq!(
            super::overworld_respawn_y(
                normal_column,
                false,
                &[
                    SpawnBlockKind::Air,
                    SpawnBlockKind::Fluid,
                    SpawnBlockKind::Solid
                ]
            ),
            None
        );
    }

    #[test]
    fn spawn_height_fixup_walks_like_vanilla() {
        let blocked_until_70 = |y| y >= 70;
        assert_eq!(
            super::fixup_spawn_height(64, -64, 320, blocked_until_70),
            70
        );

        let air_above_ground = |y| y >= 65;
        assert_eq!(
            super::fixup_spawn_height(80, -64, 320, air_above_ground),
            65
        );
    }

    #[test]
    fn flat_generator_expands_layers_bottom_up_like_vanilla() {
        let settings = super::default_flat_generator_settings().unwrap();
        assert_eq!(settings.biome, "minecraft:plains");
        assert_eq!(
            settings.structure_overrides,
            vec!["minecraft:strongholds", "minecraft:villages"]
        );
        assert_eq!(
            settings.expanded_layers,
            vec![
                Some("minecraft:bedrock"),
                Some("minecraft:dirt"),
                Some("minecraft:dirt"),
                Some("minecraft:grass_block")
            ]
        );
        assert!(!settings.void_generation);
        assert_eq!(
            super::flat_base_height(
                &settings.expanded_layers,
                super::FLAT_GENERATOR_MIN_Y,
                super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            4
        );
        assert_eq!(
            super::flat_base_column(&settings.expanded_layers, 0, 6).states,
            vec![
                "minecraft:bedrock",
                "minecraft:dirt",
                "minecraft:dirt",
                "minecraft:grass_block",
                "minecraft:air",
                "minecraft:air"
            ]
        );
    }

    #[test]
    fn flat_generator_handles_void_and_non_motion_blocking_layers() {
        let void_settings = super::flat_generator_settings(
            super::flat_generator_preset("minecraft:the_void").unwrap(),
        )
        .unwrap();
        assert!(void_settings.void_generation);
        assert_eq!(void_settings.expanded_layers, vec![None]);
        assert_eq!(
            void_settings.top_layer_modifications,
            vec![(0, "minecraft:air")]
        );

        let snowy = super::flat_generator_settings(
            super::flat_generator_preset("minecraft:snowy_kingdom").unwrap(),
        )
        .unwrap();
        assert_eq!(snowy.expanded_layers[0], Some("minecraft:bedrock"));
        assert_eq!(snowy.expanded_layers[63], Some("minecraft:grass_block"));
        assert_eq!(snowy.expanded_layers[64], None);
        assert_eq!(snowy.top_layer_modifications, vec![(64, "minecraft:snow")]);
        assert_eq!(
            super::flat_base_height(
                &snowy.expanded_layers,
                super::FLAT_GENERATOR_MIN_Y,
                super::FLAT_GENERATOR_GEN_DEPTH,
                HeightmapKind::MotionBlocking
            ),
            64
        );
    }

    #[test]
    fn flat_generator_materializes_chunk_sections_and_heightmaps() {
        let settings = super::default_flat_generator_settings().unwrap();
        let chunk = super::materialize_flat_chunk(ChunkPos { x: 2, z: -1 }, &settings);
        assert_eq!(chunk.status, "minecraft:full");
        assert_eq!(chunk.sections.len(), 1);
        assert_eq!(chunk.sections[0].y, 0);
        assert!(chunk.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(chunk.heightmaps.contains_key("OCEAN_FLOOR_WG"));

        let Tag::Compound(section) = &chunk.sections[0].block_states else {
            panic!("block states should be stored as a compound");
        };
        let Some((_, Tag::List(palette))) = section.iter().find(|(name, _)| name == "palette")
        else {
            panic!("block states should include a palette");
        };
        assert_eq!(palette.len(), 4);
        assert!(matches!(
            &chunk.sections[0].biomes,
            Tag::Compound(fields)
                if matches!(
                    fields.iter().find(|(name, _)| name == "palette"),
                    Some((_, Tag::List(values))) if values == &vec![Tag::String("minecraft:plains".to_string())]
                )
        ));
    }
}
