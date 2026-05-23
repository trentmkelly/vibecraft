use super::*;

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
pub struct ConcentricRingBiomeSearchResult {
    pub block_x: i32,
    pub block_z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureExclusionZoneModel {
    pub other_set: &'static str,
    pub chunk_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGeneratorStructureStateModel {
    pub level_seed: i64,
    pub concentric_rings_seed: i64,
    pub possible_structure_sets: Vec<StructureSetEntry>,
    pub(super) placements_for_structure: BTreeMap<&'static str, Vec<StructurePlacementKind>>,
    pub(super) ring_positions: BTreeMap<&'static str, Vec<ChunkPos>>,
    pub(super) has_generated_positions: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePieceNeighborState {
    pub direction: HorizontalDirection,
    pub chest: bool,
    pub solid_render: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePieceMirror {
    None,
    LeftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePieceRotation {
    None,
    Clockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructurePieceOrientationState {
    pub orientation: Option<HorizontalDirection>,
    pub mirror: StructurePieceMirror,
    pub rotation: StructurePieceRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawProjectionModel {
    TerrainMatching,
    Rigid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawDirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJointTypeModel {
    Rollable,
    Aligned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawConnectorModel {
    pub name: &'static str,
    pub target: &'static str,
    pub pool: &'static str,
    pub front: JigsawDirectionModel,
    pub top: JigsawDirectionModel,
    pub joint: JigsawJointTypeModel,
    pub placement_priority: i32,
    pub selection_priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawLocalConnectorModel {
    pub connector: JigsawConnectorModel,
    pub local_pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawPoolSizeModel {
    pub name: &'static str,
    pub fallback: Option<&'static str>,
    pub max_size: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawPoolAvailabilityWarning {
    EmptyOrNonExistentTarget,
    EmptyOrNonExistentFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawPoolAvailabilityDecisionModel {
    pub can_place_children: bool,
    pub warning: Option<JigsawPoolAvailabilityWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawChildFreeShapeScope {
    SourcePiece,
    Context,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawChildFreeShapeSelectionModel {
    pub scope: JigsawChildFreeShapeScope,
    pub initialized_source_shape: Option<StructureBoundingBoxModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawAcceptedChildSchedulingModel {
    pub child_depth: i32,
    pub queue_for_expansion: bool,
    pub placement_priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequencedPriorityQueueModel<T> {
    pub(super) queues_by_priority: BTreeMap<i32, VecDeque<T>>,
    pub(super) highest_priority: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiquidSettingsModel {
    IgnoreWaterlogging,
    ApplyWaterlogging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionPaddingModel {
    pub bottom: i32,
    pub top: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawMaxDistanceModel {
    pub horizontal: i32,
    pub vertical: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawJunctionModel {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
    pub delta_y: i32,
    pub dest_projection: JigsawProjectionModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawJunctionTagModel {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
    pub delta_y: i32,
    pub dest_proj: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawPoolAliasWeightedTarget {
    pub target: &'static str,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawPoolAliasWeightedGroup {
    pub bindings: Vec<JigsawPoolAliasBindingModel>,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JigsawPoolAliasBindingModel {
    Direct {
        alias: &'static str,
        target: &'static str,
    },
    Random {
        alias: &'static str,
        targets: Vec<JigsawPoolAliasWeightedTarget>,
    },
    RandomGroup {
        groups: Vec<JigsawPoolAliasWeightedGroup>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JigsawPoolAliasLookupModel {
    pub mappings: BTreeMap<&'static str, &'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawPoolElementTypeModel {
    Single,
    List,
    Feature,
    Empty,
    LegacySingle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawPoolElementModel {
    pub element_type: JigsawPoolElementTypeModel,
    pub projection: JigsawProjectionModel,
    pub location: Option<&'static str>,
    pub processors: &'static [&'static str],
    pub override_liquid_settings: Option<LiquidSettingsModel>,
    pub children: Vec<JigsawPoolElementModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawTemplatePoolElementEntry {
    pub element: JigsawPoolElementModel,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawTemplatePoolModel {
    pub fallback: &'static str,
    pub raw_templates: Vec<JigsawTemplatePoolElementEntry>,
    pub expanded_template_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplatePoolRegistry {
    pub pools: BTreeMap<String, ParsedJigsawTemplatePool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJigsawTemplatePool {
    pub fallback: String,
    pub elements: Vec<ParsedJigsawTemplatePoolEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJigsawTemplatePoolEntry {
    pub element: ParsedJigsawPoolElement,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJigsawPoolElement {
    pub element_type: String,
    pub projection: Option<String>,
    pub location: Option<String>,
    pub processors: Vec<String>,
    pub feature: Option<String>,
    pub children: Vec<ParsedJigsawPoolElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProcessorListRegistry {
    pub lists: BTreeMap<String, ParsedStructureProcessorList>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStructureProcessorList {
    pub processors: Vec<ParsedStructureProcessor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStructureProcessor {
    pub processor_type: String,
    pub rules: Vec<ParsedStructureProcessorRule>,
    pub delegate: Option<Box<ParsedStructureProcessor>>,
    pub limit: Option<i32>,
    pub integrity: Option<String>,
    pub rottable_blocks: Option<String>,
    pub cannot_replace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStructureProcessorRule {
    pub input_predicate_type: String,
    pub location_predicate_type: String,
    pub output_state_name: Option<String>,
    pub block_entity_modifier_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawCandidatePoolSource {
    Target,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawCandidateElementModel {
    pub source: JigsawCandidatePoolSource,
    pub raw_template_index: usize,
    pub element: JigsawPoolElementModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultFeatureJigsawModel {
    pub name: &'static str,
    pub final_state: &'static str,
    pub pool: &'static str,
    pub target: &'static str,
    pub joint: &'static str,
    pub orientation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolElementStructurePieceModel {
    pub element: JigsawPoolElementModel,
    pub position: (i32, i32, i32),
    pub ground_level_delta: i32,
    pub rotation: StructurePieceRotation,
    pub bounding_box: StructureBoundingBoxModel,
    pub liquid_settings: LiquidSettingsModel,
    pub junctions: Vec<JigsawJunctionModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolElementStructurePieceTagModel {
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,
    pub ground_level_delta: i32,
    pub rotation: StructurePieceRotation,
    pub junctions: Vec<JigsawJunctionTagModel>,
    pub liquid_settings: Option<LiquidSettingsModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJunctionYOffsetCase {
    BothRigid,
    SourceRigid,
    TargetRigid,
    BothTerrainMatching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawChildPlacementYModel {
    pub target_box_y: i32,
    pub target_ground_level_delta: i32,
    pub junction_y: i32,
    pub case: JigsawJunctionYOffsetCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawChildBoxPlacementModel {
    pub raw_target_box_pos: BlockPos,
    pub raw_target_bounding_box: StructureBoundingBoxModel,
    pub y_offset: i32,
    pub target_box_position: BlockPos,
    pub target_bounding_box: StructureBoundingBoxModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawStructureModel {
    pub start_pool: &'static str,
    pub start_jigsaw_name: Option<&'static str>,
    pub max_depth: i32,
    pub start_height: HeightProvider,
    pub use_expansion_hack: bool,
    pub project_start_to_heightmap: Option<&'static str>,
    pub max_distance_from_center: JigsawMaxDistanceModel,
    pub pool_aliases: Vec<JigsawPoolAliasBindingModel>,
    pub dimension_padding: DimensionPaddingModel,
    pub liquid_settings: LiquidSettingsModel,
    pub terrain_adjustment: TerrainAdjustmentModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawGenerationPointModel {
    pub start_pos: (i32, i32, i32),
    pub pool_alias_lookup: JigsawPoolAliasLookupModel,
    pub max_depth: i32,
    pub use_expansion_hack: bool,
    pub project_start_to_heightmap: Option<&'static str>,
    pub max_distance_from_center: JigsawMaxDistanceModel,
    pub dimension_padding: DimensionPaddingModel,
    pub liquid_settings: LiquidSettingsModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawExpansionBoundsModel {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x_exclusive: i32,
    pub max_y_exclusive: i32,
    pub max_z_exclusive: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawStartAnchorAdjustmentModel {
    pub local_anchor: BlockPos,
    pub adjusted_position: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureProcessorTypeModel {
    BlockIgnore,
    BlockRot,
    Gravity,
    JigsawReplacement,
    Rule,
    Nop,
    BlockAge,
    BlackstoneReplace,
    LavaSubmergedBlock,
    ProtectedBlocks,
    Capped,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructureProcessorModel {
    BlockIgnore {
        blocks: Vec<&'static str>,
    },
    BlockRot {
        rottable_blocks: Option<&'static str>,
        integrity: f32,
    },
    Gravity {
        heightmap: &'static str,
        offset: i32,
    },
    JigsawReplacement,
    Rule {
        rules: usize,
    },
    Nop,
    BlockAge {
        mossiness: f32,
    },
    BlackstoneReplace,
    LavaSubmergedBlock,
    ProtectedBlocks {
        cannot_replace: &'static str,
    },
    Capped {
        delegate: Box<StructureProcessorModel>,
        limit: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRuleTestTypeModel {
    AlwaysTrue,
    BlockMatch,
    BlockStateMatch,
    TagMatch,
    RandomBlockMatch,
    RandomBlockStateMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructurePosRuleTestTypeModel {
    AlwaysTrue,
    LinearPos,
    AxisAlignedLinearPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleBlockEntityModifierTypeModel {
    Clear,
    Passthrough,
    AppendStatic,
    AppendLoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateNbtValueModel {
    String(&'static str),
    Long(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemplateCompoundTagModel {
    pub values: BTreeMap<&'static str, TemplateNbtValueModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBlockEntityModifierModel {
    Clear,
    Passthrough,
    AppendStatic { data: TemplateCompoundTagModel },
    AppendLoot { loot_table: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePathFactoryModel {
    pub source_dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureTemplateFileKind {
    Nbt,
    Snbt,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StructureTemplateManagerModel {
    pub cache: BTreeMap<Identifier, Option<&'static str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSourceKindModel {
    Directory,
    ResourceManager,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSourceModel {
    pub kind: TemplateSourceKindModel,
    pub source_dir: Option<&'static str>,
    pub load_as_text: bool,
    pub available: BTreeMap<Identifier, &'static str>,
    pub fail_on_load: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateLoadAttemptModel {
    pub source_kind: TemplateSourceKindModel,
    pub id: Identifier,
    pub result: TemplateLoadAttemptResultModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateLoadAttemptResultModel {
    SourceUnavailable,
    Missing,
    ErrorSuppressed,
    Loaded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureStartModel {
    pub structure: Option<&'static str>,
    pub chunk_pos: ChunkPos,
    pub references: i32,
    pub pieces: Vec<StructurePieceModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureSpawnBoundingBoxTypeModel {
    Piece,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSpawnOverrideModel {
    pub category: &'static str,
    pub bounding_box: StructureSpawnBoundingBoxTypeModel,
    pub spawns: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureSpawnCandidateModel<'a> {
    pub structure: &'static str,
    pub start: &'a StructureStartModel,
    pub override_model: Option<StructureSpawnOverrideModel>,
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
pub struct StructureReferenceModel {
    pub structure: &'static str,
    pub source_chunk_key: i64,
}
