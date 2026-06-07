#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::block_update::{BlockPos, Direction};
use crate::chunk_manager::ChunkManager;
use crate::command_execution::{CommandSourceStackModel, Vec3};
use crate::map_state::DyeColor;
use crate::recipe_system::FuelValues;
use crate::redstone::{comparator_output, ComparatorMode, MAX_SIGNAL};
use crate::spawning::{spawner_tick_plan, SpawnerConfig, SpawnerTickPlan};
use crate::special_block::{
    command_block_tick, CommandBlockMode, CommandBlockState, SpecialBlockAction,
};
use crate::storage::datafix::require_current_world_data_version;
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;
use crate::vibration::{
    calibrated_sculk_sensor_receive, redstone_strength_for_distance, tick_vibration,
    vibration_frequency, SculkSensorAction, VibrationData, VibrationInfo, VibrationTickAction,
    NO_VIBRATION_FREQUENCY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockEntityTypeId {
    Furnace,
    Chest,
    TrappedChest,
    EnderChest,
    Jukebox,
    Dispenser,
    Dropper,
    Sign,
    HangingSign,
    MobSpawner,
    CreakingHeart,
    Piston,
    BrewingStand,
    EnchantingTable,
    EndPortal,
    Beacon,
    Skull,
    DaylightDetector,
    Hopper,
    Comparator,
    Banner,
    StructureBlock,
    EndGateway,
    CommandBlock,
    ShulkerBox,
    Bed,
    Conduit,
    Barrel,
    Smoker,
    BlastFurnace,
    Lectern,
    Bell,
    Jigsaw,
    Campfire,
    Beehive,
    SculkSensor,
    CalibratedSculkSensor,
    SculkCatalyst,
    SculkShrieker,
    ChiseledBookshelf,
    Shelf,
    BrushableBlock,
    DecoratedPot,
    Crafter,
    TrialSpawner,
    Vault,
    TestBlock,
    TestInstanceBlock,
    CopperGolemStatue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityTickKind {
    None,
    Server,
    Client,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityTypeInfo {
    pub id: BlockEntityTypeId,
    pub key: &'static str,
    pub valid_blocks: &'static [&'static str],
    pub tick_kind: BlockEntityTickKind,
    pub op_only_custom_data: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntity {
    pub ty: BlockEntityTypeId,
    pub pos: BlockPos,
    pub block_state: String,
    pub custom_data: BTreeMap<String, Tag>,
    pub components: BTreeMap<String, Tag>,
    pub has_level: bool,
    pub removed: bool,
    pub changed: bool,
    pub tick_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBlockEntityDataPacket {
    pub pos: BlockPos,
    pub ty: BlockEntityTypeId,
    pub tag: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntityChangedEffect {
    pub chunk_pos: ChunkPos,
    pub update_output_signal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TickingBlockEntity {
    pub entity: BlockEntity,
    pub client_side: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityMenuOpen {
    pub container_id: i32,
    pub menu_type: &'static str,
    pub initial_slots: Vec<Option<PotItemStack>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntityMenuClose {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntityDestructionContext {
    pub correct_tool: bool,
    pub silk_touch: bool,
    pub explosion_survives: bool,
    pub do_tile_drops: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityDestructionDrops {
    pub block_item: Option<String>,
    pub stored_items: Vec<PotItemStack>,
}

impl BlockEntityMenuOpen {
    pub fn close(self) -> BlockEntityMenuClose {
        BlockEntityMenuClose {
            container_id: self.container_id,
        }
    }
}

pub fn open_ender_chest_menu(
    container_id: i32,
    ender_chest_slots: Vec<Option<PotItemStack>>,
) -> BlockEntityMenuOpen {
    BlockEntityMenuOpen {
        container_id,
        menu_type: "generic_9x3",
        initial_slots: ender_chest_slots,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlockEntityState {
    pub mode: TestBlockMode,
    pub message: String,
    pub powered: bool,
    pub triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestInstanceStatus {
    Cleared,
    Running,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceBlockEntityData {
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: String,
    pub ignore_entities: bool,
    pub status: TestInstanceStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceErrorMarker {
    pub pos: BlockPos,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestInstanceBlockEntityState {
    pub data: TestInstanceBlockEntityData,
    pub errors: Vec<TestInstanceErrorMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BedBlockEntity {
    pub color: DyeColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndPortalBlockEntity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheEndGatewayBlockEntity {
    pub age: i64,
    pub teleport_cooldown: i32,
    pub exit_portal: Option<BlockPos>,
    pub exact_teleport: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJointType {
    Rollable,
    Aligned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawGenerationPlan {
    pub pool: String,
    pub target: String,
    pub levels: i32,
    pub keep_jigsaws: bool,
    pub start_pos: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawBlockEntity {
    pub name: String,
    pub target: String,
    pub pool: String,
    pub joint: JigsawJointType,
    pub final_state: String,
    pub placement_priority: i32,
    pub selection_priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureBlockMode {
    Save,
    Load,
    Corner,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureRenderMode {
    None,
    Box,
    BoxAndInvisibleBlocks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructureRenderableBox {
    pub min: BlockPos,
    pub max: BlockPos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureBlockEntity {
    pub structure_name: Option<String>,
    pub author: String,
    pub metadata: String,
    pub structure_pos: BlockPos,
    pub structure_size: (i32, i32, i32),
    pub mirror: StructureMirror,
    pub rotation: StructureRotation,
    pub mode: StructureBlockMode,
    pub ignore_entities: bool,
    pub strict: bool,
    pub powered: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparatorBlockEntity {
    pub mode: ComparatorMode,
    pub output_signal: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaylightDetectorBlockEntity {
    pub inverted: bool,
    pub power: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockEntity {
    pub command: String,
    pub success_count: i32,
    pub custom_name: Option<String>,
    pub track_output: bool,
    pub last_output: Option<String>,
    pub update_last_execution: bool,
    pub last_execution: i64,
    pub powered: bool,
    pub automatic: bool,
    pub condition_met: bool,
    pub mode: CommandBlockMode,
    pub conditional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockExecutionContext {
    pub pos: BlockPos,
    pub level: String,
    pub game_time: i64,
    pub command_blocks_enabled: bool,
    pub has_permission: bool,
    pub previous_success: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockExecution {
    pub command: String,
    pub source: CommandSourceStackModel,
    pub success_count: i32,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockUpdate {
    pub command: String,
    pub mode: CommandBlockMode,
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockChainEntry {
    pub pos: BlockPos,
    pub facing: Direction,
    pub block: CommandBlockEntity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockChainStep {
    pub pos: BlockPos,
    pub command: String,
    pub success_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JukeboxSongEvent {
    Started,
    Stopped,
    ItemChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JukeboxBlockEntity {
    pub item: Option<PotItemStack>,
    pub is_playing: bool,
    pub ticks_since_song_started: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantingTableBlockEntity {
    pub custom_name: Option<String>,
    pub time: i32,
    pub flip: f32,
    pub o_flip: f32,
    pub flip_t: f32,
    pub flip_a: f32,
    pub open: f32,
    pub o_open: f32,
    pub rot: f32,
    pub o_rot: f32,
    pub t_rot: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfBlockEntity {
    pub items: Vec<Option<PotItemStack>>,
    pub align_items_to_bottom: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconBeamBlock {
    Transparent,
    Blocking,
    Bedrock,
    TintedGlass(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconBeamSection {
    pub color: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconEffectApplication {
    pub effect: String,
    pub duration_ticks: i32,
    pub amplifier: i32,
    pub range: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconBlockEntity {
    pub levels: i32,
    pub primary_power: Option<String>,
    pub secondary_power: Option<String>,
    pub custom_name: Option<String>,
    pub lock_key: Option<String>,
    pub payment_item: Option<PotItemStack>,
    pub beam_sections: Vec<BeaconBeamSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LecternBlockEntity {
    pub book: Option<PotItemStack>,
    pub page: i32,
    pub page_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangingSignAttachment {
    Wall,
    Ceiling,
    CeilingMiddle,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignLine {
    pub raw: String,
    pub filtered: String,
    pub click_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignText {
    pub lines: [SignLine; 4],
    pub color: DyeColor,
    pub has_glowing_text: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignBlockEntityModel {
    pub front_text: SignText,
    pub back_text: SignText,
    pub is_waxed: bool,
    pub player_who_may_edit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HangingSignBlockEntityModel {
    pub sign: SignBlockEntityModel,
    pub attachment: HangingSignAttachment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewingRecipe {
    pub source_item: &'static str,
    pub source_potion: &'static str,
    pub ingredient: &'static str,
    pub result_item: &'static str,
    pub result_potion: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrewingStandTickResult {
    Idle,
    FuelLoaded,
    Started,
    Brewing,
    Brewed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewingStandBlockEntity {
    pub items: Vec<Option<PotItemStack>>,
    pub brew_time: i32,
    pub fuel: i32,
    pub ingredient: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrafterRecipe {
    pub pattern: [Option<&'static str>; 9],
    pub result: PotItemStack,
    pub remaining_items: Vec<PotItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrafterPulseResult {
    NotTriggered,
    NoRecipe,
    Crafted {
        result: PotItemStack,
        remaining_items: Vec<PotItemStack>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrafterBlockEntity {
    pub items: Vec<Option<PotItemStack>>,
    pub disabled_slots: [bool; 9],
    pub triggered: bool,
    pub crafting_ticks_remaining: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnerCustomSpawnRules {
    pub block_light_limit: (i32, i32),
    pub sky_light_limit: (i32, i32),
    pub requires_no_sky_access: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnDataModel {
    pub entity: Tag,
    pub custom_spawn_rules: Option<SpawnerCustomSpawnRules>,
    pub equipment: Option<Tag>,
    pub weight: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnerTickResult {
    Idle,
    CountDown,
    Delay,
    MobCapReached,
    SpawnRulesFailed,
    Spawned { entity_id: String, count: i32 },
    TrySpawn { entity_id: String, attempts: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnerSpawnContext {
    pub player_in_range: bool,
    pub spawner_blocks_work: bool,
    pub nearby_entities: i32,
    pub block_light: i32,
    pub sky_light: i32,
    pub no_sky_access: bool,
    pub collision_free: bool,
    pub spawn_rules_ok: bool,
    pub obstruction_free: bool,
    pub delay_roll: i32,
    pub potential_roll: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnerBlockEntity {
    pub spawn_delay: i32,
    pub min_spawn_delay: i32,
    pub max_spawn_delay: i32,
    pub spawn_count: i32,
    pub max_nearby_entities: i32,
    pub required_player_range: i32,
    pub spawn_range: i32,
    pub spawn_potentials: Vec<SpawnDataModel>,
    pub next_spawn_data: Option<SpawnDataModel>,
    pub spin: f64,
    pub old_spin: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrialSpawnerStateModel {
    Inactive,
    WaitingForPlayers,
    Active,
    WaitingForRewardEjection,
    EjectingReward,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerConfigModel {
    pub spawn_range: i32,
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub total_mobs_added_per_player: f32,
    pub simultaneous_mobs_added_per_player: f32,
    pub ticks_between_spawn: i32,
    pub spawn_potentials: Vec<SpawnDataModel>,
    pub loot_tables_to_eject: Vec<String>,
    pub items_to_drop_when_ominous: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerFullConfigModel {
    pub normal_config: TrialSpawnerConfigModel,
    pub ominous_config: TrialSpawnerConfigModel,
    pub target_cooldown_length: i32,
    pub required_player_range: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrialSpawnerBlockEntity {
    pub state: TrialSpawnerStateModel,
    pub is_ominous: bool,
    pub config: TrialSpawnerFullConfigModel,
    pub detected_players: Vec<String>,
    pub current_mobs: Vec<String>,
    pub cooldown_ends_at: i64,
    pub next_mob_spawns_at: i64,
    pub total_mobs_spawned: i32,
    pub next_spawn_data: Option<SpawnDataModel>,
    pub ejecting_loot_table: Option<String>,
    pub spin: f64,
    pub old_spin: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultStateModel {
    Inactive,
    Active,
    Unlocking,
    Ejecting,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VaultConfigModel {
    pub loot_table: String,
    pub activation_range: f64,
    pub deactivation_range: f64,
    pub key_item: PotItemStack,
    pub override_loot_table_to_display: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VaultBlockEntity {
    pub state: VaultStateModel,
    pub is_ominous: bool,
    pub config: VaultConfigModel,
    pub rewarded_players: BTreeSet<String>,
    pub connected_players: BTreeSet<String>,
    pub display_item: Option<PotItemStack>,
    pub items_to_eject: Vec<PotItemStack>,
    pub total_ejections_needed: i32,
    pub state_updating_resumes_at: i64,
    pub last_insert_fail_timestamp: i64,
    pub connected_particles_range: f64,
    pub current_spin: f32,
    pub previous_spin: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultTickResult {
    Waiting,
    StateChanged(VaultStateModel),
    EjectedItem(PotItemStack),
    EjectionFinished,
    DisplayItemCycled(Option<PotItemStack>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultInsertResult {
    IgnoredInactive,
    WrongKey { expected: String },
    AlreadyRewarded,
    Unlocking { items_to_eject: usize },
    EmptyReward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrialSpawnerTickResult {
    StateChanged(TrialSpawnerStateModel),
    Waiting,
    DetectedPlayers(usize),
    SpawnMob {
        entity_id: String,
    },
    ReadyForRewards,
    EjectedReward {
        loot_table: String,
        remaining_players: usize,
    },
    CooldownFinished,
    BecameOminous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialSpawnerTickContext {
    pub game_time: i64,
    pub can_spawn_in_level: bool,
    pub detected_player_count: usize,
    pub current_mobs_alive: usize,
    pub spawn_success: bool,
    pub apply_ominous: bool,
    pub roll: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnaceBlockEntityKind {
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnaceCookingRecipe {
    pub recipe_id: String,
    pub recipe_type: String,
    pub input_item: String,
    pub result: PotItemStack,
    pub cooking_time: i32,
    pub experience_millis: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnaceTickResult {
    Idle,
    LitChanged { lit: bool },
    Cooking,
    Burned { output_count: i32 },
    Cooling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractFurnaceBlockEntity {
    pub kind: FurnaceBlockEntityKind,
    pub items: [Option<PotItemStack>; 3],
    pub lit_time_remaining: i32,
    pub lit_total_time: i32,
    pub cooking_time_spent: i32,
    pub cooking_total_time: i32,
    pub recipes_used: BTreeMap<String, (i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerBlockEntityKind {
    Chest,
    TrappedChest,
    Barrel,
    ShulkerBox,
    Dispenser,
    Dropper,
    Hopper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShulkerBoxAnimationStatus {
    Closed,
    Opening,
    Opened,
    Closing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerActivation {
    None,
    Dispense { slot: usize },
    Drop { slot: usize },
    Push { from_slot: usize },
    Pull { to_slot: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChestLidController {
    should_be_open: bool,
    openness: f32,
    previous_openness: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerBlockEntityModel {
    pub kind: ContainerBlockEntityKind,
    pub items: Vec<Option<PotItemStack>>,
    pub custom_name: Option<String>,
    pub lock_key: Option<String>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
    pub viewer_count: i32,
    pub chest_lid: ChestLidController,
    pub lid_progress: f32,
    pub shulker_status: ShulkerBoxAnimationStatus,
    pub shulker_color: Option<DyeColor>,
    pub transfer_cooldown: i32,
    pub facing: Direction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerPatternLayer {
    pub pattern: String,
    pub color: DyeColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerBlockEntity {
    pub base_color: DyeColor,
    pub patterns: Vec<BannerPatternLayer>,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PotDecorations {
    pub back: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub front: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotItemStack {
    pub item_id: String,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoratedPotWobbleStyle {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecoratedPotBlockEntity {
    pub decorations: PotDecorations,
    pub item: Option<PotItemStack>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
    pub wobble_started_at_tick: i64,
    pub last_wobble_style: Option<DecoratedPotWobbleStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedPotDrops {
    pub decoration_items: Vec<String>,
    pub stored_item: Option<PotItemStack>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperWeatherState {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperGolemStatuePose {
    Standing,
    Sitting,
    Running,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperGolemStatueBlockEntity {
    pub weather_state: CopperWeatherState,
    pub waxed: bool,
    pub pose: CopperGolemStatuePose,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkullBlockEntity {
    pub profile: Option<Tag>,
    pub note_block_sound: Option<String>,
    pub custom_name: Option<String>,
    pub animation_tick_count: i32,
    pub is_animating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellBlockEvent {
    pub event_id: i32,
    pub event_param: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConduitBlockEntity {
    pub tick_count: i32,
    pub active_rotation: i32,
    pub is_active: bool,
    pub is_hunting: bool,
    pub effect_blocks: Vec<BlockPos>,
    pub destroy_target: Option<String>,
    pub next_ambient_sound_activation: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConduitTarget {
    pub id: String,
    pub pos: BlockPos,
    pub alive: bool,
    pub enemy: bool,
    pub in_water_or_rain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConduitEffectApplication {
    pub range: i32,
    pub duration_ticks: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampfireBlockEntity {
    pub items: Vec<Option<PotItemStack>>,
    pub cooking_progress: [i32; 4],
    pub cooking_time: [i32; 4],
    pub signal_fire: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CampfireTickResult {
    NoChange,
    Changed,
    Cooked { slot: usize, item: PotItemStack },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkSensorPhase {
    Listening,
    Ticking,
    VibrationDone,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SculkSensorBlockEntity {
    pub vibration_data: VibrationData,
    pub last_vibration_frequency: u8,
    pub phase: SculkSensorPhase,
    pub listener_radius: i32,
    pub power: u8,
    pub active_ticks: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibratedSculkSensorBlockEntity {
    pub sensor: SculkSensorBlockEntity,
    pub back_signal: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SculkChargeCursor {
    pub pos: BlockPos,
    pub charge: i32,
    pub decay_delay: i32,
    pub update_delay: i32,
    pub facings: Vec<Direction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SculkCatalystBlockEntity {
    pub cursors: Vec<SculkChargeCursor>,
    pub pulse_ticks: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeehiveOccupant {
    pub entity_type: String,
    pub entity_data: Tag,
    pub ticks_in_hive: i32,
    pub min_ticks_in_hive: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeeReleaseStatus {
    HoneyDelivered,
    BeeReleased,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeeReleaseEvent {
    pub entity_type: String,
    pub status: BeeReleaseStatus,
    pub honey_level: i32,
    pub stay_out_of_hive_ticks: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeehiveBlockEntity {
    pub occupants: Vec<BeehiveOccupant>,
    pub saved_flower_pos: Option<BlockPos>,
    pub honey_level: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreakingHeartStateModel {
    Uprooted,
    Dormant,
    Awake,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreakingHeartAction {
    None,
    StateChanged(CreakingHeartStateModel),
    SpawnProtector {
        attempts: i32,
        range_xz: i32,
        range_y: i32,
    },
    RemoveProtector,
    HurtPulse {
        total_ticks: i32,
        particle_ticks: i32,
        resin_clumps: i32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreakingHeartBlockEntity {
    pub creaking_uuid: Option<String>,
    pub ticks_existed: i64,
    pub ticker: i32,
    pub emitter_ticks: i32,
    pub output_signal: i32,
    pub state: CreakingHeartStateModel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CreakingHeartTickContext {
    pub has_required_logs: bool,
    pub creaking_active: bool,
    pub spawning_monsters: bool,
    pub player_nearby: bool,
    pub protector_resolved: bool,
    pub protector_distance: Option<f64>,
    pub protector_persistent: bool,
    pub player_stuck_in_protector: bool,
    pub next_ticker_offset: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkCatalystEventResult {
    Ignored,
    Bloom { pulse_ticks: i32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SculkShriekerBlockEntity {
    pub warning_level: i32,
    pub vibration_data: VibrationData,
    pub shrieking_ticks: i32,
    pub can_summon: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SculkShriekResult {
    Ignored,
    Shriek {
        warning_level: i32,
    },
    ReplySound {
        warning_level: i32,
        darkness_radius: i32,
    },
    SummonWarden {
        warning_level: i32,
        attempts: i32,
        range_xz: i32,
        range_y: i32,
        darkness_radius: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SculkSensorTickResult {
    None,
    Particle { travel_time_in_ticks: i32 },
    Activate { frequency: u8, redstone: u8 },
    Cooldown,
    Deactivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellTickEffects {
    pub play_resonate_sound: bool,
    pub glowing_raiders: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BellBlockEntity {
    pub last_ring_timestamp: u64,
    pub ticks: i32,
    pub shaking: bool,
    pub click_direction: Option<Direction>,
    pub heard_bell_entities: usize,
    pub nearby_raiders_within_hear_radius: usize,
    pub nearby_raiders_within_highlight_radius: usize,
    pub resonating: bool,
    pub resonation_ticks: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushResult {
    CoolingDown,
    InProgress { dusted: i32 },
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushableBlockEntity {
    pub brush_count: i32,
    pub brush_count_resets_at_tick: u64,
    pub cooldown_ends_at_tick: u64,
    pub item: Option<PotItemStack>,
    pub hit_direction: Option<Direction>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockEntityError {
    UnknownType(String),
    MissingId,
    InvalidBlockState {
        ty: BlockEntityTypeId,
        block_state: String,
    },
    UnsupportedDataVersion(String),
}

mod test_blocks;

mod structures;

mod functional_blocks;
#[cfg(test)]
use functional_blocks::*;

mod beacon_signs_brewing;

mod spawners;

mod vault_banner_furnace;

mod container_decorative;

mod sculk_conduit_campfire;

mod beehive_creaking_bell;
pub use beehive_creaking_bell::*;

mod block_entity_types_data;
pub use block_entity_types_data::*;

mod dispatcher;
pub use dispatcher::*;

#[cfg(test)]
mod tests;
