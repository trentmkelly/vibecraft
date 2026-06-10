use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationDecorationStep {
    RawGeneration,
    Lakes,
    LocalModifications,
    UndergroundStructures,
    SurfaceStructures,
    Strongholds,
    UndergroundOres,
    UndergroundDecoration,
    FluidSprings,
    VegetalDecoration,
    TopLayerModification,
}

impl GenerationDecorationStep {
    pub const VALUES: [GenerationDecorationStep; 11] = [
        GenerationDecorationStep::RawGeneration,
        GenerationDecorationStep::Lakes,
        GenerationDecorationStep::LocalModifications,
        GenerationDecorationStep::UndergroundStructures,
        GenerationDecorationStep::SurfaceStructures,
        GenerationDecorationStep::Strongholds,
        GenerationDecorationStep::UndergroundOres,
        GenerationDecorationStep::UndergroundDecoration,
        GenerationDecorationStep::FluidSprings,
        GenerationDecorationStep::VegetalDecoration,
        GenerationDecorationStep::TopLayerModification,
    ];

    pub fn serialized_name(self) -> &'static str {
        match self {
            GenerationDecorationStep::RawGeneration => "raw_generation",
            GenerationDecorationStep::Lakes => "lakes",
            GenerationDecorationStep::LocalModifications => "local_modifications",
            GenerationDecorationStep::UndergroundStructures => "underground_structures",
            GenerationDecorationStep::SurfaceStructures => "surface_structures",
            GenerationDecorationStep::Strongholds => "strongholds",
            GenerationDecorationStep::UndergroundOres => "underground_ores",
            GenerationDecorationStep::UndergroundDecoration => "underground_decoration",
            GenerationDecorationStep::FluidSprings => "fluid_springs",
            GenerationDecorationStep::VegetalDecoration => "vegetal_decoration",
            GenerationDecorationStep::TopLayerModification => "top_layer_modification",
        }
    }
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
    MatchingBlocksAt {
        offset_y: i32,
        blocks: &'static [&'static str],
    },
    MatchingBlockTag {
        tag: &'static str,
    },
    MatchingFluids {
        fluids: &'static [&'static str],
    },
    MatchingFluidsAt {
        offset_y: i32,
        fluids: &'static [&'static str],
    },
    Solid,
    SolidAt {
        offset_y: i32,
    },
    Replaceable,
    ReplaceableAt {
        offset_y: i32,
    },
    WouldSurvive {
        offset_y: i32,
        state: &'static str,
        survives: bool,
    },
    HasSturdyFace {
        offset_y: i32,
        direction: &'static str,
        sturdy: bool,
    },
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
pub enum IntProviderModel {
    Constant(i32),
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    CountProvider {
        provider: IntProviderModel,
        sampled_count: i32,
    },
    NoiseBasedCount {
        noise_to_count_ratio: i32,
        noise_factor: f64,
        noise_offset: f64,
        sampled_noise: f64,
    },
    NoiseThresholdCount {
        noise_level: f64,
        below_noise: i32,
        above_noise: i32,
        sampled_noise: f64,
    },
    CountOnEveryLayer {
        positions: &'static [BlockPos],
    },
    EnvironmentScan {
        direction_y: i32,
        target_condition: BlockPredicate,
        allowed_search_condition: BlockPredicate,
        max_steps: i32,
        states: &'static [BlockPredicateContext],
    },
    InSquare,
    Heightmap {
        heightmap: HeightmapKind,
    },
    HeightRange {
        height: HeightProvider,
    },
    RandomOffset {
        xz_spread: i32,
        y_spread: i32,
    },
    Fixed {
        positions: &'static [BlockPos],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacedOreFeatureModel {
    pub configured_feature: &'static str,
    pub placement: Vec<PlacementModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacedDiskFeatureModel {
    pub configured_feature: &'static str,
    pub placement: Vec<PlacementModifier>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSorterData {
    pub feature_index: usize,
    pub step: usize,
    pub feature: &'static str,
}

impl PartialOrd for FeatureSorterData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeatureSorterData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.step
            .cmp(&other.step)
            .then_with(|| self.feature_index.cmp(&other.feature_index))
            .then_with(|| self.feature.cmp(other.feature))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSorterSourceModel {
    pub id: &'static str,
    pub feature_steps: &'static [&'static [&'static str]],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepFeatureDataModel {
    pub features: Vec<FeatureSorterData>,
}

impl StepFeatureDataModel {
    pub fn feature_names(&self) -> Vec<&'static str> {
        self.features
            .iter()
            .map(|feature| feature.feature)
            .collect()
    }

    pub fn index_mapping(&self, feature: &str) -> Option<usize> {
        self.features
            .iter()
            .position(|candidate| candidate.feature == feature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeDecorationFeatureCall {
    pub step_index: usize,
    pub global_feature_index: usize,
    pub feature: &'static str,
    pub seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeDecorationStructureCall {
    pub step_index: usize,
    pub step_structure_index: usize,
    pub structure: &'static str,
    pub seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiomeDecorationFeaturePlan {
    pub origin: BlockPos,
    pub decoration_seed: i64,
    pub feature_calls: Vec<BiomeDecorationFeatureCall>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnOriginalMobsPlan {
    pub center: ChunkPos,
    pub biome_sample_pos: BlockPos,
    pub decoration_seed: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationMobSpawnBatchPlan {
    pub category: &'static str,
    pub entity_type: &'static str,
    pub count: i32,
    pub start_x: i32,
    pub start_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationMobSpawnAttemptPlan {
    pub mob_index: i32,
    pub attempt_index: i32,
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationMobSpawnPositionPlan {
    pub entity_type: &'static str,
    pub heightmap: HeightmapKind,
    pub placement_type: &'static str,
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkGenerationMobEntitySnapPlan {
    pub entity_type: &'static str,
    pub width: f32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkGenerationMobCollisionPlan {
    pub entity_type: &'static str,
    pub width: f32,
    pub height: f32,
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGenerationMobSpawnPlan {
    pub chunk: ChunkPos,
    pub batches: Vec<ChunkGenerationMobSpawnBatchPlan>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenSettingsModel {
    pub seed: i64,
    pub generate_structures: bool,
    pub generate_bonus_chest: bool,
    pub dimensions: ParsedWorldDimensions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorldDimensions {
    pub stems: Vec<(String, ParsedLevelStem)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorldPreset {
    pub dimensions: ParsedWorldDimensions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLevelStem {
    pub dimension_type: String,
    pub generator: ParsedChunkGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedChunkGenerator {
    Noise {
        biome_source: ParsedBiomeSource,
        settings: String,
    },
    Flat {
        settings: ParsedFlatGeneratorSettings,
    },
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedBiomeSource {
    MultiNoisePreset { preset: String },
    TheEnd,
    Fixed { biome: String },
    Checkerboard { biomes: Vec<String>, scale: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFlatGeneratorSettings {
    pub biome: String,
    pub structure_overrides: Vec<String>,
    pub add_lakes: bool,
    pub decoration: bool,
    pub layers: Vec<(i32, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedWorldgenPresetRegistry {
    pub world_presets: BTreeMap<String, ParsedWorldPreset>,
    pub flat_level_generator_presets: BTreeMap<String, ParsedFlatGeneratorSettings>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedFeatureInvocation {
    pub feature: &'static str,
    pub source: ConfiguredFeatureSource,
    pub pos: BlockPos,
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
