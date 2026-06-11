use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedBlockState {
    pub state: &'static str,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBasedBlockStateProviderRule {
    pub if_true: BlockPredicate,
    pub then: Box<BlockStateProviderModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockStateProviderModel {
    Simple(&'static str),
    Weighted(Vec<WeightedBlockState>),
    RotatedBlock(&'static str),
    RandomizedInt {
        source: Box<BlockStateProviderModel>,
        property: &'static str,
        min_inclusive: i32,
        max_inclusive: i32,
    },
    RuleBased {
        fallback: Option<Box<BlockStateProviderModel>>,
        rules: Vec<RuleBasedBlockStateProviderRule>,
    },
    Noise {
        states: Vec<&'static str>,
    },
    NoiseThreshold {
        threshold: f32,
        high_chance: f32,
        default_state: &'static str,
        low_states: Vec<&'static str>,
        high_states: Vec<&'static str>,
    },
    DualNoise {
        variety_min: i32,
        variety_max: i32,
        states: Vec<&'static str>,
    },
}

#[derive(Debug, Clone, PartialEq)]
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
    BlockTag(&'static str),
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
pub struct DesertWellSuspiciousSandPlacement {
    pub pos: BlockPos,
    pub state: &'static str,
    pub loot_table: &'static str,
    pub loot_seed: i64,
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

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPileConfigurationModel {
    pub state_provider: BlockStateProviderModel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskConfigurationModel {
    pub state_provider: BlockStateProviderModel,
    pub target: BlockPredicate,
    pub radius: IntProviderModel,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DripstoneClusterSampledConfig {
    pub floor_to_ceiling_search_range: i32,
    pub height: i32,
    pub x_radius: i32,
    pub z_radius: i32,
    pub max_stalagmite_stalactite_height_diff: i32,
    pub height_deviation: i32,
    pub dripstone_block_layer_thickness: i32,
    pub density: f32,
    pub wetness: f32,
    pub chance_of_dripstone_column_at_max_distance_from_center: f32,
    pub max_distance_from_edge_affecting_chance_of_dripstone_column: i32,
    pub max_distance_from_center_affecting_height_bias: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DripstoneClusterColumnInput {
    pub dx: i32,
    pub dz: i32,
    pub ceiling_y: Option<i32>,
    pub floor_y: Option<i32>,
    pub floor_pool_supported: bool,
    pub ceiling_is_lava: bool,
    pub floor_is_lava: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DripstoneClusterColumnRolls {
    pub water_roll: f32,
    pub stalactite_roll: f64,
    pub stalactite_density_roll: f32,
    pub stalactite_biased_height: f32,
    pub stalagmite_roll: f64,
    pub stalagmite_density_roll: f32,
    pub stalagmite_biased_height: f32,
    pub stalagmite_height_diff_roll: i32,
    pub overlap_split_roll: i32,
    pub merge_tips_roll: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointedDripstoneDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointedDripstoneThickness {
    Base,
    Middle,
    Frustum,
    Tip,
    TipMerge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointedDripstoneBlockModel {
    pub pos: BlockPos,
    pub direction: PointedDripstoneDirection,
    pub thickness: PointedDripstoneThickness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DripstoneClusterColumnPlan {
    pub water_pos: Option<BlockPos>,
    pub ceiling_dripstone_blocks: Vec<BlockPos>,
    pub floor_dripstone_blocks: Vec<BlockPos>,
    pub stalactite: Vec<PointedDripstoneBlockModel>,
    pub stalagmite: Vec<PointedDripstoneBlockModel>,
    pub merge_tips: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointedDripstoneConfigurationModel {
    pub chance_of_taller_dripstone: f32,
    pub chance_of_directional_spread: f32,
    pub chance_of_spread_radius2: f32,
    pub chance_of_spread_radius3: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointedDripstoneSpreadRoll {
    pub direction: HorizontalDirection,
    pub direction_roll: f32,
    pub radius2_roll: f32,
    pub radius2_direction: HorizontalDirection,
    pub radius3_roll: f32,
    pub radius3_direction: HorizontalDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointedDripstoneFeaturePlan {
    pub tip_direction: Option<PointedDripstoneDirection>,
    pub dripstone_blocks: Vec<BlockPos>,
    pub pointed_blocks: Vec<PointedDripstoneBlockModel>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LargeDripstoneSampledConfig {
    pub floor_to_ceiling_search_range: i32,
    pub column_radius_min: i32,
    pub column_radius_max: i32,
    pub height_scale: f64,
    pub max_column_radius_to_cave_height_ratio: f32,
    pub stalactite_bluntness: f64,
    pub stalagmite_bluntness: f64,
    pub wind_speed: f64,
    pub wind_direction_radians: f64,
    pub min_radius_for_wind: i32,
    pub min_bluntness_for_wind: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LargeDripstoneModel {
    pub root: BlockPos,
    pub pointing_up: bool,
    pub radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LargeDripstoneBlockModel {
    pub pos: BlockPos,
    pub pointing_up: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargeDripstonePlacementPlan {
    pub stalactite: LargeDripstoneModel,
    pub stalagmite: LargeDripstoneModel,
    pub wind_enabled: bool,
    pub stalactite_blocks: Vec<LargeDripstoneBlockModel>,
    pub stalagmite_blocks: Vec<LargeDripstoneBlockModel>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RootSystemConfigurationModel {
    pub tree_feature: &'static str,
    pub required_vertical_space_for_tree: i32,
    pub root_radius: i32,
    pub root_replaceable: &'static str,
    pub root_state_provider: BlockStateProviderModel,
    pub root_placement_attempts: i32,
    pub root_column_max_height: i32,
    pub hanging_root_radius: i32,
    pub hanging_roots_vertical_span: i32,
    pub hanging_root_state_provider: BlockStateProviderModel,
    pub hanging_root_placement_attempts: i32,
    pub allowed_vertical_water_for_tree: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSystemTreeCandidateModel {
    pub pos: BlockPos,
    pub allowed_tree_position: bool,
    pub vertical_space_states: Vec<&'static str>,
    pub below_state: &'static str,
    pub tree_feature_places: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RootSystemOffsetRoll {
    pub positive_x: i32,
    pub negative_x: i32,
    pub positive_y: i32,
    pub negative_y: i32,
    pub positive_z: i32,
    pub negative_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSystemPlacementKind {
    RootedDirt,
    HangingRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootSystemPlacementBlock {
    pub pos: BlockPos,
    pub state: &'static str,
    pub kind: RootSystemPlacementKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSystemPlacementPlan {
    pub tree_origin: Option<BlockPos>,
    pub blocks: Vec<RootSystemPlacementBlock>,
    pub attempted_roots: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeConfigurationModel {
    pub trunk_provider: BlockStateProviderModel,
    pub foliage_provider: BlockStateProviderModel,
    pub dirt_provider: BlockStateProviderModel,
    pub trunk_placer: TrunkPlacerModel,
    pub foliage_placer: FoliagePlacerModel,
    pub minimum_size: FeatureSizeModel,
    pub root_placer: Option<RootPlacerModel>,
    pub decorators: Vec<TreeDecoratorModel>,
    pub ignore_vines: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeLeafDistanceUpdate {
    pub pos: BlockPos,
    pub distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeFoliageAttachmentModel {
    pub pos: BlockPos,
    pub radius_offset: i32,
    pub double_trunk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrunkPlacementPlan {
    pub blocks: Vec<TreePlacementBlock>,
    pub attachments: Vec<TreeFoliageAttachmentModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpwardsBranchingBranchModel {
    pub trunk_y_offset: i32,
    pub direction: HorizontalDirection,
    pub branch_pos: i32,
    pub branch_steps: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MegaJungleBranchModel {
    pub branch_height: i32,
    pub angle_radians: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CherryBranchModel {
    pub start_offset_from_origin: i32,
    pub direction: HorizontalDirection,
    pub horizontal_length: i32,
    pub end_offset_from_origin: i32,
    pub middle_continues_upwards: bool,
    pub grow_vertically: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FancyTrunkClusterRollModel {
    pub shape_float: f32,
    pub angle_float: f32,
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
    LeaveVine {
        probability: f32,
    },
    PaleMoss {
        leaves_probability: f32,
        trunk_probability: f32,
        ground_probability: f32,
    },
    CreakingHeart {
        probability: f32,
    },
    Cocoa {
        probability: f32,
    },
    Beehive {
        probability: f32,
    },
    AlterGround,
    AttachedToLeaves {
        probability: f32,
    },
    PlaceOnGround,
    AttachedToLogs {
        probability: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeDecoratorPlacement {
    pub pos: BlockPos,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeehiveDecoratorPlacement {
    pub pos: BlockPos,
    pub state: &'static str,
    pub bee_ticks_in_hive: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceOnGroundAttemptContext {
    pub pos: BlockPos,
    pub above_is_air_or_vine: bool,
    pub pos_is_solid_render: bool,
    pub motion_blocking_no_leaves_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlterGroundScanContext {
    pub pos: BlockPos,
    pub provider_state: Option<&'static str>,
    pub is_air: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaleMossAttachmentContext {
    pub pos: BlockPos,
    pub down_air: bool,
    pub below_air: Vec<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrunkVineLogContext {
    pub pos: BlockPos,
    pub west_air: bool,
    pub east_air: bool,
    pub north_air: bool,
    pub south_air: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CocoaLogContext {
    pub pos: BlockPos,
    pub north_air: bool,
    pub east_air: bool,
    pub south_air: bool,
    pub west_air: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveVineLeafContext {
    pub pos: BlockPos,
    pub west_air: bool,
    pub east_air: bool,
    pub north_air: bool,
    pub south_air: bool,
    pub west_below_air: [bool; 4],
    pub east_below_air: [bool; 4],
    pub north_below_air: [bool; 4],
    pub south_below_air: [bool; 4],
}
