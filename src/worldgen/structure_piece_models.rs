use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuriedTreasurePieceModel {
    pub bounding_box: StructureBoundingBoxModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuriedTreasurePlacementModel {
    pub chest_pos: BlockPos,
    pub bounding_box: StructureBoundingBoxModel,
    pub side_fill: [(&'static str, BlockPos); 6],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScatteredFeaturePieceModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutPieceModel {
    pub scattered: ScatteredFeaturePieceModel,
    pub spawned_witch: bool,
    pub spawned_cat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutSaveTagModel {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
    pub witch: bool,
    pub cat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwampHutEntitySpawnModel {
    pub entity: &'static str,
    pub pos: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwampHutPostProcessModel {
    pub piece: SwampHutPieceModel,
    pub blocks: Vec<StructurePiecePlacementBlock>,
    pub fill_columns: Vec<BlockPos>,
    pub entity_spawns: Vec<SwampHutEntitySpawnModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesertPyramidPieceModel {
    pub scattered: ScatteredFeaturePieceModel,
    pub has_placed_chest: [bool; 4],
    pub potential_suspicious_sand_world_positions: Vec<BlockPos>,
    pub random_collapsed_roof_pos: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesertPyramidSaveTagModel {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
    pub has_placed_chest: [bool; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesertPyramidArchaeologyPlacement {
    pub pos: BlockPos,
    pub state: &'static str,
    pub loot_table: Option<&'static str>,
    pub loot_seed: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JungleTemplePieceModel {
    pub scattered: ScatteredFeaturePieceModel,
    pub placed_main_chest: bool,
    pub placed_hidden_chest: bool,
    pub placed_trap1: bool,
    pub placed_trap2: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JungleTempleSaveTagModel {
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub height_position: i32,
    pub placed_main_chest: bool,
    pub placed_hidden_chest: bool,
    pub placed_trap1: bool,
    pub placed_trap2: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JungleTempleContainerPlacement {
    pub kind: &'static str,
    pub pos: BlockPos,
    pub facing: Option<HorizontalDirection>,
    pub loot_table: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JungleTemplePostProcessModel {
    pub piece: JungleTemplePieceModel,
    pub containers: Vec<JungleTempleContainerPlacement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IglooTemplateKind {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IglooPieceModel {
    pub template: IglooTemplateKind,
    pub template_name: &'static str,
    pub template_position: BlockPos,
    pub rotation: StructureRotation,
    pub pivot: BlockPos,
    pub offset: BlockPos,
    pub depth: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IglooPostProcessModel {
    pub piece: IglooPieceModel,
    pub entrance_pos: BlockPos,
    pub adjusted_template_position: BlockPos,
    pub trapdoor_pos: Option<BlockPos>,
    pub should_cover_trapdoor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFossilPieceModel {
    pub template_index: usize,
    pub template_name: &'static str,
    pub template_position: BlockPos,
    pub rotation: StructureRotation,
    pub processor: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFossilGenerationPointModel {
    pub position: BlockPos,
    pub piece: NetherFossilPieceModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriedGhastPlacementModel {
    pub pos: BlockPos,
    pub rotation: StructureRotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuinedPortalVerticalPlacement {
    OnLandSurface,
    PartlyBuried,
    OnOceanFloor,
    InMountain,
    Underground,
    InNether,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuinedPortalSetupModel {
    pub placement: RuinedPortalVerticalPlacement,
    pub air_pocket_probability: f32,
    pub mossiness: f32,
    pub overgrown: bool,
    pub vines: bool,
    pub can_be_cold: bool,
    pub replace_with_blackstone: bool,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuinedPortalPropertiesModel {
    pub cold: bool,
    pub mossiness: f32,
    pub air_pocket: bool,
    pub overgrown: bool,
    pub vines: bool,
    pub replace_with_blackstone: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuinedPortalMirrorModel {
    None,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuinedPortalPieceModel {
    pub template_name: &'static str,
    pub template_position: BlockPos,
    pub vertical_placement: RuinedPortalVerticalPlacement,
    pub properties: RuinedPortalPropertiesModel,
    pub rotation: StructureRotation,
    pub mirror: RuinedPortalMirrorModel,
    pub pivot: BlockPos,
    pub ignore_processor: &'static str,
    pub lava_replacement: &'static str,
    pub include_blackstone_replace_processor: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipwreckPieceModel {
    pub template_name: &'static str,
    pub template_position: BlockPos,
    pub rotation: StructureRotation,
    pub is_beached: bool,
    pub height_adjusted: bool,
    pub pivot: BlockPos,
    pub processor: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShipwreckSaveTagModel {
    pub is_beached: bool,
    pub rotation: StructureRotation,
    pub height_adjusted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OceanRuinBiomeType {
    Warm,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OceanRuinStructureConfigModel {
    pub biome_type: OceanRuinBiomeType,
    pub large_probability: f32,
    pub cluster_probability: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OceanRuinPieceModel {
    pub template_name: &'static str,
    pub template_position: BlockPos,
    pub rotation: StructureRotation,
    pub integrity: f32,
    pub biome_type: OceanRuinBiomeType,
    pub is_large: bool,
    pub block_rot_processor_integrity: f32,
    pub suspicious_block: &'static str,
    pub suspicious_loot_table: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OceanRuinSaveTagModel {
    pub rotation: StructureRotation,
    pub integrity: f32,
    pub biome_type: OceanRuinBiomeType,
    pub is_large: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OceanRuinMarkerActionModel {
    pub marker_id: &'static str,
    pub pos: BlockPos,
    pub placed_block: &'static str,
    pub loot_table: Option<&'static str>,
    pub spawned_entity: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineshaftTypeModel {
    Normal,
    Mesa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MineshaftMaterialModel {
    pub serialized_name: &'static str,
    pub wood_state: &'static str,
    pub planks_state: &'static str,
    pub fence_state: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineshaftPieceKindModel {
    Corridor,
    Crossing,
    Stairs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MineshaftRoomModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub mineshaft_type: MineshaftTypeModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MineshaftCorridorModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub mineshaft_type: MineshaftTypeModel,
    pub has_rails: bool,
    pub spider_corridor: bool,
    pub has_placed_spider: bool,
    pub num_sections: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MineshaftCorridorSaveTagModel {
    pub has_rails: bool,
    pub spider_corridor: bool,
    pub has_placed_spider: bool,
    pub num_sections: i32,
    pub mineshaft_type_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MineshaftGeneratedPieceModel {
    Room {
        bounding_box: StructureBoundingBoxModel,
        mineshaft_type: MineshaftTypeModel,
        child_entrance_boxes: Vec<StructureBoundingBoxModel>,
        gen_depth: i32,
    },
    Corridor {
        model: MineshaftCorridorModel,
        gen_depth: i32,
    },
    Crossing {
        bounding_box: StructureBoundingBoxModel,
        direction: HorizontalDirection,
        mineshaft_type: MineshaftTypeModel,
        is_two_floored: bool,
        gen_depth: i32,
    },
    Stairs {
        bounding_box: StructureBoundingBoxModel,
        direction: HorizontalDirection,
        mineshaft_type: MineshaftTypeModel,
        gen_depth: i32,
    },
}

impl MineshaftGeneratedPieceModel {
    pub const fn bounding_box(&self) -> StructureBoundingBoxModel {
        match self {
            Self::Room { bounding_box, .. }
            | Self::Crossing { bounding_box, .. }
            | Self::Stairs { bounding_box, .. } => *bounding_box,
            Self::Corridor { model, .. } => model.bounding_box,
        }
    }

    pub const fn as_structure_piece(&self) -> StructurePieceModel {
        StructurePieceModel {
            bounding_box: self.bounding_box(),
        }
    }

    pub const fn gen_depth(&self) -> i32 {
        match self {
            Self::Room { gen_depth, .. }
            | Self::Corridor { gen_depth, .. }
            | Self::Crossing { gen_depth, .. }
            | Self::Stairs { gen_depth, .. } => *gen_depth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongholdPieceKindModel {
    Straight,
    PrisonHall,
    LeftTurn,
    RightTurn,
    RoomCrossing,
    StraightStairsDown,
    StairsDown,
    FiveCrossing,
    ChestCorridor,
    Library,
    PortalRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongholdSmallDoorTypeModel {
    Opening,
    WoodDoor,
    Grates,
    IronDoor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdPieceWeightModel {
    pub kind: StrongholdPieceKindModel,
    pub weight: i32,
    pub max_place_count: i32,
    pub place_count: i32,
    pub min_depth: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdStartPieceModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub entry_door: StrongholdSmallDoorTypeModel,
    pub is_source: bool,
    pub previous_piece: Option<StrongholdPieceKindModel>,
    pub portal_room_piece: Option<StrongholdPortalRoomModel>,
    pub pending_children: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdPortalRoomModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub gen_depth: i32,
    pub has_placed_spawner: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdPortalRoomSaveTagModel {
    pub has_placed_spawner: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherFortressPieceKindModel {
    BridgeStraight,
    BridgeCrossing,
    RoomCrossing,
    StairsRoom,
    MonsterThrone,
    CastleEntrance,
    CastleSmallCorridor,
    CastleSmallCorridorCrossing,
    CastleSmallCorridorRightTurn,
    CastleSmallCorridorLeftTurn,
    CastleCorridorStairs,
    CastleCorridorTBalcony,
    CastleStalkRoom,
    BridgeEndFiller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherFortressPiecePoolModel {
    Bridge,
    Castle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetherFortressChildDirectionModel {
    Forward,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFortressPieceWeightModel {
    pub kind: NetherFortressPieceKindModel,
    pub weight: i32,
    pub max_place_count: i32,
    pub place_count: i32,
    pub allow_in_row: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFortressStartPieceModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub previous_piece: Option<NetherFortressPieceKindModel>,
    pub bridge_piece_count: usize,
    pub castle_piece_count: usize,
    pub pending_children: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFortressPieceSelectionModel {
    pub selected_kind: NetherFortressPieceKindModel,
    pub fallback_to_end_filler: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFortressChildAnchorModel {
    pub foot: BlockPos,
    pub direction: HorizontalDirection,
    pub next_depth: i32,
    pub is_castle: bool,
    pub within_start_range: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OceanMonumentDirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OceanMonumentGenerationPointModel {
    pub biome_check_center: BlockPos,
    pub biome_check_radius: i32,
    pub heightmap: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OceanMonumentBuildingModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub source_room_index: i32,
    pub core_room_index: i32,
    pub top_connect_index: i32,
    pub left_wing_connect_index: i32,
    pub right_wing_connect_index: i32,
    pub child_piece_offset: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OceanMonumentRoomDefinitionModel {
    pub index: i32,
    pub claimed: bool,
    pub is_source: bool,
    pub is_special: bool,
    pub opening_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCityTemplatePieceModel {
    pub template_name: &'static str,
    pub template_id: &'static str,
    pub position: BlockPos,
    pub rotation: StructureRotation,
    pub overwrite: bool,
    pub processor: &'static str,
    pub gen_depth: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCityBridgeCandidateModel {
    pub rotation: StructureRotation,
    pub offset: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndCityMarkerActionModel {
    pub marker_id: &'static str,
    pub target_pos: BlockPos,
    pub loot_table: Option<&'static str>,
    pub spawned_entity: Option<&'static str>,
    pub item_frame_facing: Option<HorizontalDirection>,
    pub item: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WoodlandMansionMirrorModel {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WoodlandMansionTemplatePieceModel {
    pub template_name: &'static str,
    pub template_id: &'static str,
    pub position: BlockPos,
    pub rotation: StructureRotation,
    pub mirror: WoodlandMansionMirrorModel,
    pub processor: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WoodlandMansionPlacementDataModel {
    pub position: BlockPos,
    pub rotation: StructureRotation,
    pub wall_type: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WoodlandMansionMarkerActionModel {
    pub marker_id: &'static str,
    pub target_pos: BlockPos,
    pub loot_table: Option<&'static str>,
    pub chest_facing: Option<HorizontalDirection>,
    pub spawned_entity: Option<&'static str>,
    pub spawn_count: i32,
    pub clears_marker_block: bool,
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
pub struct JigsawStartPoolModel {
    pub structure_family: &'static str,
    pub source_file: &'static str,
    pub pool: &'static str,
}

