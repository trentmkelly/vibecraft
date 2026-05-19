#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::block_update::{BlockPos, Direction};
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerBlockEntityModel {
    pub kind: ContainerBlockEntityKind,
    pub items: Vec<Option<PotItemStack>>,
    pub custom_name: Option<String>,
    pub lock_key: Option<String>,
    pub loot_table: Option<String>,
    pub loot_table_seed: i64,
    pub viewer_count: i32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl TestBlockMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "log" => Some(Self::Log),
            "fail" => Some(Self::Fail),
            "accept" => Some(Self::Accept),
            _ => None,
        }
    }
}

impl Default for TestBlockEntityState {
    fn default() -> Self {
        Self {
            mode: TestBlockMode::Fail,
            message: String::new(),
            powered: false,
            triggered: false,
        }
    }
}

impl TestBlockEntityState {
    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "mode".to_string(),
                Tag::String(self.mode.as_str().to_string()),
            ),
            ("message".to_string(), Tag::String(self.message.clone())),
            ("powered".to_string(), Tag::Byte(i8::from(self.powered))),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        Self {
            mode: entries
                .and_then(|entries| get_string(entries, "mode"))
                .and_then(TestBlockMode::from_str)
                .unwrap_or(TestBlockMode::Fail),
            message: entries
                .and_then(|entries| get_string(entries, "message"))
                .unwrap_or("")
                .to_string(),
            powered: entries
                .and_then(|entries| get_byte(entries, "powered"))
                .unwrap_or(0)
                != 0,
            triggered: false,
        }
    }

    pub fn reset(&mut self) {
        self.triggered = false;
        if self.mode == TestBlockMode::Start {
            self.powered = false;
        }
    }

    pub fn trigger(&mut self) {
        if self.mode == TestBlockMode::Start {
            self.powered = true;
        } else {
            self.triggered = true;
        }
    }
}

impl TestInstanceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Running => "running",
            Self::Finished => "finished",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "cleared" => Some(Self::Cleared),
            "running" => Some(Self::Running),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

impl Default for TestInstanceBlockEntityData {
    fn default() -> Self {
        Self {
            test: None,
            size: (0, 0, 0),
            rotation: "none".to_string(),
            ignore_entities: false,
            status: TestInstanceStatus::Cleared,
            error_message: None,
        }
    }
}

impl TestInstanceBlockEntityData {
    pub fn with_status(&self, status: TestInstanceStatus) -> Self {
        Self {
            status,
            error_message: None,
            ..self.clone()
        }
    }

    pub fn with_error(&self, error: impl Into<String>) -> Self {
        Self {
            status: TestInstanceStatus::Finished,
            error_message: Some(error.into()),
            ..self.clone()
        }
    }

    fn to_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(test) = &self.test {
            fields.push(("test".to_string(), Tag::String(test.clone())));
        }
        fields.push((
            "size".to_string(),
            Tag::List(vec![
                Tag::Int(self.size.0),
                Tag::Int(self.size.1),
                Tag::Int(self.size.2),
            ]),
        ));
        fields.push(("rotation".to_string(), Tag::String(self.rotation.clone())));
        fields.push((
            "ignore_entities".to_string(),
            Tag::Byte(i8::from(self.ignore_entities)),
        ));
        fields.push((
            "status".to_string(),
            Tag::String(self.status.as_str().to_string()),
        ));
        if let Some(error_message) = &self.error_message {
            fields.push((
                "error_message".to_string(),
                Tag::String(error_message.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    fn from_tag(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let size = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "size"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) if values.len() == 3 => Some((
                    tag_int_or_zero(&values[0]),
                    tag_int_or_zero(&values[1]),
                    tag_int_or_zero(&values[2]),
                )),
                _ => None,
            })
            .unwrap_or((0, 0, 0));
        Self {
            test: entries
                .and_then(|entries| get_string(entries, "test"))
                .map(ToString::to_string),
            size,
            rotation: entries
                .and_then(|entries| get_string(entries, "rotation"))
                .unwrap_or("none")
                .to_string(),
            ignore_entities: entries
                .and_then(|entries| get_byte(entries, "ignore_entities"))
                .unwrap_or(0)
                != 0,
            status: entries
                .and_then(|entries| get_string(entries, "status"))
                .and_then(TestInstanceStatus::from_str)
                .unwrap_or(TestInstanceStatus::Cleared),
            error_message: entries
                .and_then(|entries| get_string(entries, "error_message"))
                .map(ToString::to_string),
        }
    }
}

impl Default for TestInstanceBlockEntityState {
    fn default() -> Self {
        Self {
            data: TestInstanceBlockEntityData::default(),
            errors: Vec::new(),
        }
    }
}

impl TestInstanceBlockEntityState {
    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![("data".to_string(), self.data.to_tag())];
        if !self.errors.is_empty() {
            fields.push((
                "errors".to_string(),
                Tag::List(
                    self.errors
                        .iter()
                        .map(TestInstanceErrorMarker::to_tag)
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let data = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "data"))
            .map(|(_, tag)| TestInstanceBlockEntityData::from_tag(tag))
            .unwrap_or_default();
        let errors = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "errors"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(TestInstanceErrorMarker::from_tag)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self { data, errors }
    }

    pub fn set_running(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Running);
    }

    pub fn set_success(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Finished);
    }

    pub fn set_error_message(&mut self, message: impl Into<String>) {
        self.data = self.data.with_error(message);
    }

    pub fn mark_error(&mut self, pos: BlockPos, text: impl Into<String>) {
        self.errors.push(TestInstanceErrorMarker {
            pos,
            text: text.into(),
        });
    }

    pub fn clear_error_markers(&mut self) {
        self.errors.clear();
    }
}

impl TestInstanceErrorMarker {
    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "pos".to_string(),
                Tag::List(vec![
                    Tag::Int(self.pos.x),
                    Tag::Int(self.pos.y),
                    Tag::Int(self.pos.z),
                ]),
            ),
            ("text".to_string(), Tag::String(self.text.clone())),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let pos =
            entries
                .iter()
                .find(|(name, _)| name == "pos")
                .and_then(|(_, tag)| match tag {
                    Tag::List(values) if values.len() == 3 => Some(BlockPos {
                        x: tag_int_or_zero(&values[0]),
                        y: tag_int_or_zero(&values[1]),
                        z: tag_int_or_zero(&values[2]),
                    }),
                    _ => None,
                })?;
        let text = get_string(entries, "text")?.to_string();
        Some(Self { pos, text })
    }
}

impl BedBlockEntity {
    pub fn from_block_state(block_state: &str) -> Option<Self> {
        let color = match block_state.strip_prefix("minecraft:")? {
            "white_bed" => DyeColor::White,
            "orange_bed" => DyeColor::Orange,
            "magenta_bed" => DyeColor::Magenta,
            "light_blue_bed" => DyeColor::LightBlue,
            "yellow_bed" => DyeColor::Yellow,
            "lime_bed" => DyeColor::Lime,
            "pink_bed" => DyeColor::Pink,
            "gray_bed" => DyeColor::Gray,
            "light_gray_bed" => DyeColor::LightGray,
            "cyan_bed" => DyeColor::Cyan,
            "purple_bed" => DyeColor::Purple,
            "blue_bed" => DyeColor::Blue,
            "brown_bed" => DyeColor::Brown,
            "green_bed" => DyeColor::Green,
            "red_bed" => DyeColor::Red,
            "black_bed" => DyeColor::Black,
            _ => return None,
        };
        Some(Self { color })
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }
}

impl EndPortalBlockEntity {
    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }
}

impl TheEndGatewayBlockEntity {
    pub const SPAWN_TIME: i64 = 200;
    pub const COOLDOWN_TIME: i32 = 40;
    pub const ATTENTION_INTERVAL: i64 = 2400;
    pub const EVENT_COOLDOWN: i32 = 1;
    pub const GATEWAY_HEIGHT_ABOVE_SURFACE: i32 = 10;

    pub fn new() -> Self {
        Self {
            age: 0,
            teleport_cooldown: 0,
            exit_portal: None,
            exact_teleport: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![("Age".to_string(), Tag::Long(self.age))];
        if let Some(exit_portal) = self.exit_portal {
            fields.push(("exit_portal".to_string(), block_pos_to_tag(exit_portal)));
        }
        if self.exact_teleport {
            fields.push(("ExactTeleport".to_string(), Tag::Byte(1)));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        Self {
            age: entries
                .iter()
                .find(|(name, _)| name == "Age")
                .map(|(_, tag)| tag_long_or_zero(tag))
                .unwrap_or(0),
            teleport_cooldown: 0,
            exit_portal: entries
                .iter()
                .find(|(name, _)| name == "exit_portal")
                .and_then(|(_, tag)| block_pos_from_tag(tag)),
            exact_teleport: get_bool(entries, "ExactTeleport").unwrap_or(false),
        }
    }

    pub fn beam_animation_tick(&mut self) {
        self.age += 1;
        if self.is_cooling_down() {
            self.teleport_cooldown -= 1;
        }
    }

    pub fn portal_tick(&mut self) -> bool {
        let was_spawning = self.is_spawning();
        let was_cooling_down = self.is_cooling_down();
        self.age += 1;
        if was_cooling_down {
            self.teleport_cooldown -= 1;
        } else if self.age % Self::ATTENTION_INTERVAL == 0 {
            self.trigger_cooldown();
        }
        was_spawning != self.is_spawning() || was_cooling_down != self.is_cooling_down()
    }

    pub fn is_spawning(&self) -> bool {
        self.age < Self::SPAWN_TIME
    }

    pub fn is_cooling_down(&self) -> bool {
        self.teleport_cooldown > 0
    }

    pub fn spawn_percent(&self, partial_tick: f32) -> f32 {
        ((self.age as f32 + partial_tick) / Self::SPAWN_TIME as f32).clamp(0.0, 1.0)
    }

    pub fn cooldown_percent(&self, partial_tick: f32) -> f32 {
        1.0 - ((self.teleport_cooldown as f32 - partial_tick) / Self::COOLDOWN_TIME as f32)
            .clamp(0.0, 1.0)
    }

    pub fn trigger_cooldown(&mut self) {
        self.teleport_cooldown = Self::COOLDOWN_TIME;
    }

    pub fn trigger_event(&mut self, event: i32) -> bool {
        if event == Self::EVENT_COOLDOWN {
            self.trigger_cooldown();
            true
        } else {
            false
        }
    }

    pub fn set_exit_position(&mut self, exit_portal: BlockPos, exact_teleport: bool) {
        self.exit_portal = Some(exit_portal);
        self.exact_teleport = exact_teleport;
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl JigsawJointType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "rollable" => Some(Self::Rollable),
            "aligned" => Some(Self::Aligned),
            _ => None,
        }
    }
}

impl JigsawBlockEntity {
    pub const EMPTY_ID: &'static str = "minecraft:empty";
    pub const DEFAULT_FINAL_STATE: &'static str = "minecraft:air";

    pub fn new() -> Self {
        Self {
            name: Self::EMPTY_ID.to_string(),
            target: Self::EMPTY_ID.to_string(),
            pool: Self::EMPTY_ID.to_string(),
            joint: JigsawJointType::Rollable,
            final_state: Self::DEFAULT_FINAL_STATE.to_string(),
            placement_priority: 0,
            selection_priority: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("name".to_string(), Tag::String(self.name.clone())),
            ("target".to_string(), Tag::String(self.target.clone())),
            ("pool".to_string(), Tag::String(self.pool.clone())),
            (
                "final_state".to_string(),
                Tag::String(self.final_state.clone()),
            ),
            (
                "joint".to_string(),
                Tag::String(self.joint.as_str().to_string()),
            ),
            (
                "placement_priority".to_string(),
                Tag::Int(self.placement_priority),
            ),
            (
                "selection_priority".to_string(),
                Tag::Int(self.selection_priority),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        Self {
            name: get_string(entries, "name")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            target: get_string(entries, "target")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            pool: get_string(entries, "pool")
                .unwrap_or(Self::EMPTY_ID)
                .to_string(),
            final_state: get_string(entries, "final_state")
                .unwrap_or(Self::DEFAULT_FINAL_STATE)
                .to_string(),
            joint: get_string(entries, "joint")
                .and_then(JigsawJointType::from_str)
                .unwrap_or(JigsawJointType::Rollable),
            placement_priority: get_int(entries, "placement_priority").unwrap_or(0),
            selection_priority: get_int(entries, "selection_priority").unwrap_or(0),
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn generation_plan(
        &self,
        block_pos: BlockPos,
        orientation_front: Direction,
        levels: i32,
        keep_jigsaws: bool,
    ) -> JigsawGenerationPlan {
        JigsawGenerationPlan {
            pool: self.pool.clone(),
            target: self.target.clone(),
            levels,
            keep_jigsaws,
            start_pos: block_pos.relative(orientation_front),
        }
    }
}

impl StructureBlockMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Save => "SAVE",
            Self::Load => "LOAD",
            Self::Corner => "CORNER",
            Self::Data => "DATA",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "SAVE" | "save" => Some(Self::Save),
            "LOAD" | "load" => Some(Self::Load),
            "CORNER" | "corner" => Some(Self::Corner),
            "DATA" | "data" => Some(Self::Data),
            _ => None,
        }
    }
}

impl StructureMirror {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::LeftRight => "LEFT_RIGHT",
            Self::FrontBack => "FRONT_BACK",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" | "none" => Some(Self::None),
            "LEFT_RIGHT" | "left_right" => Some(Self::LeftRight),
            "FRONT_BACK" | "front_back" => Some(Self::FrontBack),
            _ => None,
        }
    }
}

impl StructureRotation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Clockwise90 => "CLOCKWISE_90",
            Self::Clockwise180 => "CLOCKWISE_180",
            Self::Counterclockwise90 => "COUNTERCLOCKWISE_90",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NONE" | "none" => Some(Self::None),
            "CLOCKWISE_90" | "clockwise_90" => Some(Self::Clockwise90),
            "CLOCKWISE_180" | "clockwise_180" => Some(Self::Clockwise180),
            "COUNTERCLOCKWISE_90" | "counterclockwise_90" => Some(Self::Counterclockwise90),
            _ => None,
        }
    }
}

impl StructureBlockEntity {
    pub const MAX_OFFSET_PER_AXIS: i32 = 48;
    pub const MAX_SIZE_PER_AXIS: i32 = 48;

    pub fn new(mode: StructureBlockMode) -> Self {
        Self {
            structure_name: None,
            author: String::new(),
            metadata: String::new(),
            structure_pos: BlockPos { x: 0, y: 1, z: 0 },
            structure_size: (0, 0, 0),
            mirror: StructureMirror::None,
            rotation: StructureRotation::None,
            mode,
            ignore_entities: true,
            strict: false,
            powered: false,
            show_air: false,
            show_bounding_box: true,
            integrity: 1.0,
            seed: 0,
        }
    }

    pub fn has_structure_name(&self) -> bool {
        self.structure_name.is_some()
    }

    pub fn structure_name(&self) -> &str {
        self.structure_name.as_deref().unwrap_or("")
    }

    pub fn set_structure_name(&mut self, structure_name: Option<&str>) {
        self.structure_name = structure_name
            .filter(|name| !name.is_empty())
            .map(ToString::to_string);
    }

    pub fn set_structure_pos(&mut self, pos: BlockPos) {
        self.structure_pos = Self::clamp_structure_pos(pos);
    }

    pub fn set_structure_size(&mut self, size: (i32, i32, i32)) {
        self.structure_size = Self::clamp_structure_size(size);
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "name".to_string(),
                Tag::String(self.structure_name().to_string()),
            ),
            ("author".to_string(), Tag::String(self.author.clone())),
            ("metadata".to_string(), Tag::String(self.metadata.clone())),
            ("posX".to_string(), Tag::Int(self.structure_pos.x)),
            ("posY".to_string(), Tag::Int(self.structure_pos.y)),
            ("posZ".to_string(), Tag::Int(self.structure_pos.z)),
            ("sizeX".to_string(), Tag::Int(self.structure_size.0)),
            ("sizeY".to_string(), Tag::Int(self.structure_size.1)),
            ("sizeZ".to_string(), Tag::Int(self.structure_size.2)),
            (
                "rotation".to_string(),
                Tag::String(self.rotation.as_str().to_string()),
            ),
            (
                "mirror".to_string(),
                Tag::String(self.mirror.as_str().to_string()),
            ),
            (
                "mode".to_string(),
                Tag::String(self.mode.as_str().to_string()),
            ),
            (
                "ignoreEntities".to_string(),
                Tag::Byte(i8::from(self.ignore_entities)),
            ),
            ("strict".to_string(), Tag::Byte(i8::from(self.strict))),
            ("powered".to_string(), Tag::Byte(i8::from(self.powered))),
            ("showair".to_string(), Tag::Byte(i8::from(self.show_air))),
            (
                "showboundingbox".to_string(),
                Tag::Byte(i8::from(self.show_bounding_box)),
            ),
            ("integrity".to_string(), Tag::Float(self.integrity)),
            ("seed".to_string(), Tag::Long(self.seed)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(StructureBlockMode::Data);
        };
        let mut entity = Self::new(
            get_string(entries, "mode")
                .and_then(StructureBlockMode::from_str)
                .unwrap_or(StructureBlockMode::Data),
        );
        entity.set_structure_name(get_string(entries, "name"));
        entity.author = get_string(entries, "author").unwrap_or("").to_string();
        entity.metadata = get_string(entries, "metadata").unwrap_or("").to_string();
        entity.structure_pos = Self::clamp_structure_pos(BlockPos {
            x: get_int(entries, "posX").unwrap_or(0),
            y: get_int(entries, "posY").unwrap_or(1),
            z: get_int(entries, "posZ").unwrap_or(0),
        });
        entity.structure_size = Self::clamp_structure_size((
            get_int(entries, "sizeX").unwrap_or(0),
            get_int(entries, "sizeY").unwrap_or(0),
            get_int(entries, "sizeZ").unwrap_or(0),
        ));
        entity.rotation = get_string(entries, "rotation")
            .and_then(StructureRotation::from_str)
            .unwrap_or(StructureRotation::None);
        entity.mirror = get_string(entries, "mirror")
            .and_then(StructureMirror::from_str)
            .unwrap_or(StructureMirror::None);
        entity.ignore_entities = get_bool(entries, "ignoreEntities").unwrap_or(true);
        entity.strict = get_bool(entries, "strict").unwrap_or(false);
        entity.powered = get_bool(entries, "powered").unwrap_or(false);
        entity.show_air = get_bool(entries, "showair").unwrap_or(false);
        entity.show_bounding_box = get_bool(entries, "showboundingbox").unwrap_or(true);
        entity.integrity = get_float(entries, "integrity").unwrap_or(1.0);
        entity.seed = get_long(entries, "seed").unwrap_or(0);
        entity
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn render_mode(&self) -> StructureRenderMode {
        if !matches!(
            self.mode,
            StructureBlockMode::Save | StructureBlockMode::Load
        ) {
            StructureRenderMode::None
        } else if self.mode == StructureBlockMode::Save && self.show_air {
            StructureRenderMode::BoxAndInvisibleBlocks
        } else if self.mode != StructureBlockMode::Save && !self.show_bounding_box {
            StructureRenderMode::None
        } else {
            StructureRenderMode::Box
        }
    }

    pub fn renderable_box(&self) -> StructureRenderableBox {
        let x_origin = self.structure_pos.x;
        let z_origin = self.structure_pos.z;
        let y0 = self.structure_pos.y;
        let y1 = y0 + self.structure_size.1;
        let (x_diff, z_diff) = match self.mirror {
            StructureMirror::LeftRight => (self.structure_size.0, -self.structure_size.2),
            StructureMirror::FrontBack => (-self.structure_size.0, self.structure_size.2),
            StructureMirror::None => (self.structure_size.0, self.structure_size.2),
        };
        let (x0, z0, x1, z1) = match self.rotation {
            StructureRotation::Clockwise90 => {
                let x0 = if z_diff < 0 { x_origin } else { x_origin + 1 };
                let z0 = if x_diff < 0 { z_origin + 1 } else { z_origin };
                (x0, z0, x0 - z_diff, z0 + x_diff)
            }
            StructureRotation::Clockwise180 => {
                let x0 = if x_diff < 0 { x_origin } else { x_origin + 1 };
                let z0 = if z_diff < 0 { z_origin } else { z_origin + 1 };
                (x0, z0, x0 - x_diff, z0 - z_diff)
            }
            StructureRotation::Counterclockwise90 => {
                let x0 = if z_diff < 0 { x_origin + 1 } else { x_origin };
                let z0 = if x_diff < 0 { z_origin } else { z_origin + 1 };
                (x0, z0, x0 + z_diff, z0 - x_diff)
            }
            StructureRotation::None => {
                let x0 = if x_diff < 0 { x_origin + 1 } else { x_origin };
                let z0 = if z_diff < 0 { z_origin + 1 } else { z_origin };
                (x0, z0, x0 + x_diff, z0 + z_diff)
            }
        };
        StructureRenderableBox {
            min: BlockPos {
                x: x0.min(x1),
                y: y0.min(y1),
                z: z0.min(z1),
            },
            max: BlockPos {
                x: x0.max(x1),
                y: y0.max(y1),
                z: z0.max(z1),
            },
        }
    }

    fn clamp_structure_pos(pos: BlockPos) -> BlockPos {
        BlockPos {
            x: pos
                .x
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
            y: pos
                .y
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
            z: pos
                .z
                .clamp(-Self::MAX_OFFSET_PER_AXIS, Self::MAX_OFFSET_PER_AXIS),
        }
    }

    fn clamp_structure_size(size: (i32, i32, i32)) -> (i32, i32, i32) {
        (
            size.0.clamp(0, Self::MAX_SIZE_PER_AXIS),
            size.1.clamp(0, Self::MAX_SIZE_PER_AXIS),
            size.2.clamp(0, Self::MAX_SIZE_PER_AXIS),
        )
    }
}

impl ComparatorBlockEntity {
    pub fn new(mode: ComparatorMode) -> Self {
        Self {
            mode,
            output_signal: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![(
            "OutputSignal".to_string(),
            Tag::Int(self.output_signal),
        )])
    }

    pub fn load_additional(mode: ComparatorMode, tag: &Tag) -> Self {
        let output_signal = compound_entries(tag)
            .and_then(|entries| get_int(entries, "OutputSignal"))
            .unwrap_or(0);
        Self {
            mode,
            output_signal,
        }
    }

    pub fn calculate_output(&self, rear_input: u8, side_input: u8) -> u8 {
        comparator_output(self.mode, rear_input, side_input)
    }

    pub fn update_output(&mut self, rear_input: u8, side_input: u8) -> bool {
        let next = self.calculate_output(rear_input, side_input);
        let next = i32::from(next);
        let changed = self.output_signal != next;
        self.output_signal = next;
        changed
    }
}

impl DaylightDetectorBlockEntity {
    pub const TICK_INTERVAL: u64 = 20;

    pub fn new(inverted: bool) -> Self {
        Self { inverted, power: 0 }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(Vec::new())
    }

    pub fn calculate_power(
        inverted: bool,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> u8 {
        let mut target = effective_sky_brightness;
        if inverted {
            target = i32::from(MAX_SIGNAL) - target;
        } else if target > 0 {
            let mut sun_angle = sun_angle_degrees.to_radians();
            let offset = if sun_angle < std::f32::consts::PI {
                0.0
            } else {
                std::f32::consts::TAU
            };
            sun_angle += (offset - sun_angle) * 0.2;
            target = ((target as f32) * sun_angle.cos()).round() as i32;
        }

        target.clamp(0, i32::from(MAX_SIGNAL)) as u8
    }

    pub fn update_signal(&mut self, effective_sky_brightness: i32, sun_angle_degrees: f32) -> bool {
        let next =
            Self::calculate_power(self.inverted, effective_sky_brightness, sun_angle_degrees);
        let changed = self.power != next;
        self.power = next;
        changed
    }

    pub fn tick(
        &mut self,
        game_time: u64,
        effective_sky_brightness: i32,
        sun_angle_degrees: f32,
    ) -> bool {
        if game_time % Self::TICK_INTERVAL != 0 {
            return false;
        }
        self.update_signal(effective_sky_brightness, sun_angle_degrees)
    }
}

impl CommandBlockEntity {
    pub const NO_LAST_EXECUTION: i64 = -1;
    pub const PERMISSION_LEVEL: &'static str = "gamemaster";
    pub const SEARGE_COMMAND: &'static str = "Searge";
    pub const SEARGE_OUTPUT: &'static str = "#itzlipofutzli";

    pub fn new(mode: CommandBlockMode, conditional: bool) -> Self {
        Self {
            command: String::new(),
            success_count: 0,
            custom_name: None,
            track_output: true,
            last_output: None,
            update_last_execution: true,
            last_execution: Self::NO_LAST_EXECUTION,
            powered: false,
            automatic: false,
            condition_met: false,
            mode,
            conditional,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Command".to_string(), Tag::String(self.command.clone())),
            ("SuccessCount".to_string(), Tag::Int(self.success_count)),
            (
                "TrackOutput".to_string(),
                Tag::Byte(self.track_output as i8),
            ),
            (
                "UpdateLastExecution".to_string(),
                Tag::Byte(self.update_last_execution as i8),
            ),
            ("powered".to_string(), Tag::Byte(self.powered as i8)),
            (
                "conditionMet".to_string(),
                Tag::Byte(self.condition_met as i8),
            ),
            ("auto".to_string(), Tag::Byte(self.automatic as i8)),
        ];

        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if self.track_output {
            if let Some(last_output) = &self.last_output {
                entries.push(("LastOutput".to_string(), Tag::String(last_output.clone())));
            }
        }
        if self.update_last_execution && self.last_execution != Self::NO_LAST_EXECUTION {
            entries.push(("LastExecution".to_string(), Tag::Long(self.last_execution)));
        }

        Tag::Compound(entries)
    }

    pub fn load_additional(mode: CommandBlockMode, conditional: bool, tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(mode, conditional);
        };
        let track_output = get_bool(entries, "TrackOutput").unwrap_or(true);
        let update_last_execution = get_bool(entries, "UpdateLastExecution").unwrap_or(true);
        Self {
            command: get_string(entries, "Command").unwrap_or("").to_string(),
            success_count: get_int(entries, "SuccessCount").unwrap_or(0),
            custom_name: get_string(entries, "CustomName").map(ToString::to_string),
            track_output,
            last_output: if track_output {
                get_string(entries, "LastOutput").map(ToString::to_string)
            } else {
                None
            },
            update_last_execution,
            last_execution: if update_last_execution {
                get_long(entries, "LastExecution").unwrap_or(Self::NO_LAST_EXECUTION)
            } else {
                Self::NO_LAST_EXECUTION
            },
            powered: get_bool(entries, "powered").unwrap_or(false),
            automatic: get_bool(entries, "auto").unwrap_or(false),
            condition_met: get_bool(entries, "conditionMet").unwrap_or(false),
            mode,
            conditional,
        }
    }

    pub fn set_command(&mut self, command: impl Into<String>) {
        self.command = command.into();
        self.success_count = 0;
    }

    pub fn set_automatic(&mut self, automatic: bool, has_level: bool) -> bool {
        let previous = self.automatic;
        self.automatic = automatic;
        !previous
            && automatic
            && !self.powered
            && has_level
            && self.mode != CommandBlockMode::Sequence
    }

    pub fn mark_condition_met(&mut self, previous_command_success: bool) -> bool {
        self.condition_met = !self.conditional || previous_command_success;
        self.condition_met
    }

    pub fn can_use(&self, player_can_use_gamemaster_blocks: bool) -> bool {
        player_can_use_gamemaster_blocks
    }

    pub fn open_editor_packet(
        &self,
        pos: BlockPos,
        player_can_use_gamemaster_blocks: bool,
    ) -> Option<ClientboundBlockEntityDataPacket> {
        self.can_use(player_can_use_gamemaster_blocks)
            .then(|| ClientboundBlockEntityDataPacket {
                pos,
                ty: BlockEntityTypeId::CommandBlock,
                tag: self.save_additional(),
            })
    }

    pub fn apply_client_update(
        &mut self,
        update: CommandBlockUpdate,
        player_can_use_gamemaster_blocks: bool,
        has_level: bool,
    ) -> bool {
        if !self.can_use(player_can_use_gamemaster_blocks) {
            return false;
        }

        self.mode = update.mode;
        self.conditional = update.conditional;
        self.track_output = update.track_output;
        if !self.track_output {
            self.last_output = None;
        }
        self.set_automatic(update.automatic, has_level);
        self.set_command(update.command);
        true
    }

    pub fn command_source_stack(
        pos: BlockPos,
        level: impl Into<String>,
    ) -> CommandSourceStackModel {
        CommandSourceStackModel::new("CommandBlockEntity", level, 2).with_position(Vec3 {
            x: f64::from(pos.x) + 0.5,
            y: f64::from(pos.y) + 0.5,
            z: f64::from(pos.z) + 0.5,
        })
    }

    pub fn execution_action(
        &self,
        has_permission: bool,
        previous_success: bool,
    ) -> SpecialBlockAction {
        command_block_tick(
            &CommandBlockState {
                command: self.command.clone(),
                mode: self.mode.clone(),
                powered: self.powered,
                previously_powered: false,
                conditional: self.conditional,
                previous_success,
            },
            has_permission,
        )
    }

    pub fn perform_command(
        &mut self,
        game_time: i64,
        command_blocks_enabled: bool,
        has_permission: bool,
        previous_success: bool,
    ) -> bool {
        if self.update_last_execution && self.last_execution == game_time {
            return false;
        }
        if self.command.eq_ignore_ascii_case(Self::SEARGE_COMMAND) {
            self.last_output = Some(Self::SEARGE_OUTPUT.to_string());
            self.success_count = 1;
            return true;
        }

        self.success_count = 0;
        let executed = match self
            .execution_action(has_permission && command_blocks_enabled, previous_success)
        {
            SpecialBlockAction::ExecuteCommand { success_count } => {
                self.success_count = success_count;
                true
            }
            SpecialBlockAction::Noop => false,
            _ => false,
        };

        if executed {
            if self.update_last_execution {
                self.last_execution = game_time;
            } else {
                self.last_execution = Self::NO_LAST_EXECUTION;
            }
        }
        executed
    }

    pub fn execute_from_context(
        &mut self,
        context: CommandBlockExecutionContext,
        output: Option<String>,
    ) -> Option<CommandBlockExecution> {
        let source = Self::command_source_stack(context.pos, context.level);
        if !self.perform_command(
            context.game_time,
            context.command_blocks_enabled,
            context.has_permission,
            context.previous_success,
        ) {
            return None;
        }

        if self.track_output {
            if let Some(output) = output.clone() {
                self.last_output = Some(output);
            }
        } else {
            self.last_output = None;
        }

        let output = output.or_else(|| self.last_output.clone());
        Some(CommandBlockExecution {
            command: self.command.clone(),
            source,
            success_count: self.success_count,
            output,
        })
    }
}

pub fn execute_command_block_chain(
    entries: &mut [CommandBlockChainEntry],
    start_pos: BlockPos,
    context: CommandBlockExecutionContext,
) -> Vec<CommandBlockChainStep> {
    let mut steps = Vec::new();
    let mut current_pos = start_pos;
    let mut previous_success = context.previous_success;

    for _ in 0..entries.len() {
        let Some(index) = entries.iter().position(|entry| entry.pos == current_pos) else {
            break;
        };
        let next_pos = entries[index].pos.relative(entries[index].facing);
        let entry = &mut entries[index];
        let mut step_context = context.clone();
        step_context.pos = entry.pos;
        step_context.previous_success = previous_success;

        let success_count = if entry
            .block
            .execute_from_context(step_context, None)
            .is_some()
        {
            entry.block.success_count
        } else {
            0
        };
        previous_success = success_count > 0;
        steps.push(CommandBlockChainStep {
            pos: entry.pos,
            command: entry.block.command.clone(),
            success_count,
        });

        current_pos = next_pos;
        if !entries
            .iter()
            .any(|entry| entry.pos == current_pos && entry.block.mode == CommandBlockMode::Sequence)
        {
            break;
        }
    }

    steps
}

impl JukeboxBlockEntity {
    pub const RECORD_ITEM_TAG_ID: &'static str = "RecordItem";
    pub const TICKS_SINCE_SONG_STARTED_TAG_ID: &'static str = "ticks_since_song_started";
    pub const STOP_LEVEL_EVENT: i32 = 1011;

    pub fn new() -> Self {
        Self {
            item: None,
            is_playing: false,
            ticks_since_song_started: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(item) = self.item.as_ref().filter(|item| !item.is_empty()) {
            entries.push((Self::RECORD_ITEM_TAG_ID.to_string(), item.to_tag()));
        }
        if self.song_item().is_some() {
            entries.push((
                Self::TICKS_SINCE_SONG_STARTED_TAG_ID.to_string(),
                Tag::Long(self.ticks_since_song_started),
            ));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let item = entries
            .iter()
            .find(|(key, _)| key == Self::RECORD_ITEM_TAG_ID)
            .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        let mut jukebox = Self {
            item,
            is_playing: false,
            ticks_since_song_started: 0,
        };
        if jukebox.song_item().is_some() {
            jukebox.ticks_since_song_started =
                get_long(entries, Self::TICKS_SINCE_SONG_STARTED_TAG_ID).unwrap_or(0);
        }
        jukebox
    }

    pub fn set_the_item(&mut self, item: Option<PotItemStack>) -> JukeboxSongEvent {
        self.item = item.filter(|item| !item.is_empty());
        self.ticks_since_song_started = 0;
        if self.song_item().is_some() {
            self.is_playing = true;
            JukeboxSongEvent::Started
        } else {
            self.is_playing = false;
            JukeboxSongEvent::Stopped
        }
    }

    pub fn set_song_item_without_playing(&mut self, item: PotItemStack) -> JukeboxSongEvent {
        self.item = (!item.is_empty()).then_some(item);
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        JukeboxSongEvent::ItemChanged
    }

    pub fn remove_the_item(&mut self) -> Option<PotItemStack> {
        self.is_playing = false;
        self.ticks_since_song_started = 0;
        self.item.take()
    }

    pub fn pop_out_the_item(&mut self) -> Option<PotItemStack> {
        self.remove_the_item()
    }

    pub fn tick(&mut self) -> bool {
        if self.is_playing {
            self.ticks_since_song_started += 1;
            true
        } else {
            false
        }
    }

    pub fn redstone_signal(&self) -> u8 {
        if self.is_playing {
            MAX_SIGNAL
        } else {
            0
        }
    }

    pub fn comparator_output(&self) -> u8 {
        self.song_item()
            .map(jukebox_song_comparator_output)
            .unwrap_or(0)
    }

    pub fn can_place_item(&self, item: &PotItemStack) -> bool {
        self.item.is_none() && jukebox_song_item_id(&item.item_id).is_some()
    }

    pub fn can_take_item(&self, destination_has_empty_slot: bool) -> bool {
        destination_has_empty_slot
    }

    fn song_item(&self) -> Option<&str> {
        self.item
            .as_ref()
            .and_then(|item| jukebox_song_item_id(&item.item_id))
    }
}

fn jukebox_song_item_id(item_id: &str) -> Option<&str> {
    item_id.strip_prefix("minecraft:music_disc_")
}

fn jukebox_song_comparator_output(song_id: &str) -> u8 {
    match song_id {
        "13" => 1,
        "cat" => 2,
        "blocks" => 3,
        "chirp" => 4,
        "far" => 5,
        "mall" => 6,
        "mellohi" => 7,
        "stal" => 8,
        "strad" | "lava_chicken" => 9,
        "ward" | "tears" => 10,
        "11" | "creator_music_box" => 11,
        "wait" | "creator" => 12,
        "pigstep" | "precipice" => 13,
        "otherside" | "relic" => 14,
        "5" => 15,
        _ => 0,
    }
}

impl EnchantingTableBlockEntity {
    pub const DEFAULT_NAME: &'static str = "container.enchant";

    pub fn new() -> Self {
        Self {
            custom_name: None,
            time: 0,
            flip: 0.0,
            o_flip: 0.0,
            flip_t: 0.0,
            flip_a: 0.0,
            open: 0.0,
            o_open: 0.0,
            rot: 0.0,
            o_rot: 0.0,
            t_rot: 0.0,
        }
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "enchantment",
            initial_slots: vec![None, None],
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut table = Self::new();
        if let Some(entries) = compound_entries(tag) {
            table.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        }
        table
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn bookshelf_offsets() -> Vec<BlockPos> {
        let mut offsets = Vec::new();
        for x in -2i32..=2 {
            for y in 0i32..=1 {
                for z in -2i32..=2 {
                    if x.abs() == 2 || z.abs() == 2 {
                        offsets.push(BlockPos { x, y, z });
                    }
                }
            }
        }
        offsets
    }

    pub fn count_valid_bookshelves(
        is_power_provider: impl Fn(BlockPos) -> bool,
        is_power_transmitter: impl Fn(BlockPos) -> bool,
    ) -> usize {
        Self::bookshelf_offsets()
            .into_iter()
            .filter(|offset| {
                is_power_provider(*offset)
                    && is_power_transmitter(BlockPos {
                        x: offset.x / 2,
                        y: offset.y,
                        z: offset.z / 2,
                    })
            })
            .take(15)
            .count()
    }

    pub fn book_animation_tick(
        &mut self,
        player_offset_xz: Option<(f64, f64)>,
        next_flip_delta: Option<f32>,
    ) {
        self.o_open = self.open;
        self.o_rot = self.rot;
        if let Some((xd, zd)) = player_offset_xz {
            self.t_rot = (zd.atan2(xd)) as f32;
            self.open += 0.1;
            if self.open < 0.5 {
                if let Some(delta) = next_flip_delta {
                    let old = self.flip_t;
                    if delta != 0.0 {
                        self.flip_t += delta;
                    } else {
                        self.flip_t += 1.0;
                    }
                    if self.flip_t == old {
                        self.flip_t += 1.0;
                    }
                }
            }
        } else {
            self.t_rot += 0.02;
            self.open -= 0.1;
        }

        self.rot = wrap_radians(self.rot);
        self.t_rot = wrap_radians(self.t_rot);
        let rot_dir = wrap_radians(self.t_rot - self.rot);
        self.rot += rot_dir * 0.4;
        self.open = self.open.clamp(0.0, 1.0);
        self.time += 1;
        self.o_flip = self.flip;
        let diff = ((self.flip_t - self.flip) * 0.4).clamp(-0.2, 0.2);
        self.flip_a += (diff - self.flip_a) * 0.9;
        self.flip += self.flip_a;
    }
}

fn wrap_radians(mut value: f32) -> f32 {
    while value >= std::f32::consts::PI {
        value -= std::f32::consts::TAU;
    }
    while value < -std::f32::consts::PI {
        value += std::f32::consts::TAU;
    }
    value
}

impl ShelfBlockEntity {
    pub const MAX_ITEMS: usize = 3;
    pub const ALIGN_ITEMS_TO_BOTTOM_TAG: &'static str = "align_items_to_bottom";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::MAX_ITEMS],
            align_items_to_bottom: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![(
            "Items".to_string(),
            Tag::List(
                self.items
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, item)| {
                        let item = item.as_ref().filter(|item| !item.is_empty())?;
                        let mut tag = match item.to_tag() {
                            Tag::Compound(entries) => entries,
                            _ => return None,
                        };
                        tag.push(("Slot".to_string(), Tag::Byte(slot as i8)));
                        Some(Tag::Compound(tag))
                    })
                    .collect(),
            ),
        )];
        entries.push((
            Self::ALIGN_ITEMS_TO_BOTTOM_TAG.to_string(),
            Tag::Byte(self.align_items_to_bottom as i8),
        ));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut shelf = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return shelf;
        };
        shelf.align_items_to_bottom =
            get_bool(entries, Self::ALIGN_ITEMS_TO_BOTTOM_TAG).unwrap_or(false);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(key, _)| key == "Items")
            .map(|(_, tag)| tag)
        {
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) = get_byte(item_entries, "Slot") else {
                    continue;
                };
                if let Some(slot) = usize::try_from(slot)
                    .ok()
                    .filter(|slot| *slot < Self::MAX_ITEMS)
                {
                    shelf.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
        shelf
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn get_item(&self, slot: usize) -> Option<&PotItemStack> {
        self.items.get(slot).and_then(Option::as_ref)
    }

    pub fn set_item_no_update(&mut self, slot: usize, item: Option<PotItemStack>) -> bool {
        let Some(target) = self.items.get_mut(slot) else {
            return false;
        };
        *target = item.filter(|item| !item.is_empty());
        true
    }

    pub fn remove_item_no_update(&mut self, slot: usize) -> Option<PotItemStack> {
        self.items.get_mut(slot).and_then(Option::take)
    }

    pub fn swap_item_no_update(
        &mut self,
        slot: usize,
        held_item_stack: Option<PotItemStack>,
    ) -> Option<PotItemStack> {
        let retrieved = self.remove_item_no_update(slot);
        self.set_item_no_update(slot, held_item_stack);
        retrieved
    }

    pub fn filled_slot_count(&self) -> usize {
        self.items.iter().filter(|item| item.is_some()).count()
    }

    pub fn comparator_output(&self) -> u8 {
        self.filled_slot_count() as u8
    }
}

impl BeaconBeamSection {
    pub fn new(color: i32) -> Self {
        Self { color, height: 1 }
    }

    pub fn increase_height(&mut self) {
        self.height += 1;
    }
}

impl BeaconBlockEntity {
    pub const MAX_LEVELS: i32 = 4;
    pub const BLOCKS_CHECK_PER_TICK: i32 = 10;
    pub const DEFAULT_NAME: &'static str = "container.beacon";

    pub fn new() -> Self {
        Self {
            levels: 0,
            primary_power: None,
            secondary_power: None,
            custom_name: None,
            lock_key: None,
            payment_item: None,
            beam_sections: Vec::new(),
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(primary) = &self.primary_power {
            entries.push(("primary_effect".to_string(), Tag::String(primary.clone())));
        }
        if let Some(secondary) = &self.secondary_power {
            entries.push((
                "secondary_effect".to_string(),
                Tag::String(secondary.clone()),
            ));
        }
        entries.push(("Levels".to_string(), Tag::Int(self.levels)));
        if let Some(custom_name) = &self.custom_name {
            entries.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        if let Some(lock_key) = &self.lock_key {
            entries.push(("Lock".to_string(), Tag::String(lock_key.clone())));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut beacon = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return beacon;
        };
        beacon.primary_power = get_string(entries, "primary_effect").and_then(filter_beacon_effect);
        beacon.secondary_power =
            get_string(entries, "secondary_effect").and_then(filter_beacon_effect);
        beacon.levels = get_int(entries, "Levels")
            .unwrap_or(0)
            .clamp(0, Self::MAX_LEVELS);
        beacon.custom_name = get_string(entries, "CustomName").map(ToString::to_string);
        beacon.lock_key = get_string(entries, "Lock").map(ToString::to_string);
        beacon
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn display_name(&self) -> &str {
        self.custom_name.as_deref().unwrap_or(Self::DEFAULT_NAME)
    }

    pub fn set_primary_power(&mut self, effect: Option<&str>) {
        self.primary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn set_secondary_power(&mut self, effect: Option<&str>) {
        self.secondary_power = effect.and_then(filter_beacon_effect);
    }

    pub fn can_pay_with(item: &PotItemStack) -> bool {
        matches!(
            item.item_id.as_str(),
            "minecraft:netherite_ingot"
                | "minecraft:emerald"
                | "minecraft:diamond"
                | "minecraft:gold_ingot"
                | "minecraft:iron_ingot"
                | "minecraft:amethyst_shard"
        )
    }

    pub fn set_payment_item(&mut self, item: Option<PotItemStack>) -> bool {
        if item.as_ref().is_some_and(|item| !Self::can_pay_with(item)) {
            return false;
        }
        self.payment_item = item.filter(|item| !item.is_empty());
        true
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "beacon",
            initial_slots: vec![self.payment_item.clone()],
        }
    }

    pub fn comparator_output(&self) -> u8 {
        self.levels.clamp(0, Self::MAX_LEVELS) as u8
    }

    pub fn update_base(
        beacon_pos: BlockPos,
        min_y: i32,
        is_base_block: impl Fn(BlockPos) -> bool,
    ) -> i32 {
        let mut levels = 0;
        for step in 1..=Self::MAX_LEVELS {
            let y = beacon_pos.y - step;
            if y < min_y {
                break;
            }
            let mut ok = true;
            'layer: for x in beacon_pos.x - step..=beacon_pos.x + step {
                for z in beacon_pos.z - step..=beacon_pos.z + step {
                    if !is_base_block(BlockPos { x, y, z }) {
                        ok = false;
                        break 'layer;
                    }
                }
            }
            if !ok {
                break;
            }
            levels = step;
        }
        levels
    }

    pub fn scan_beam(blocks: impl IntoIterator<Item = BeaconBeamBlock>) -> Vec<BeaconBeamSection> {
        let mut sections: Vec<BeaconBeamSection> = Vec::new();
        let mut last: Option<BeaconBeamSection> = None;
        for block in blocks {
            match block {
                BeaconBeamBlock::TintedGlass(color) => {
                    if sections.len() <= 1 {
                        let section = BeaconBeamSection::new(color);
                        sections.push(section.clone());
                        last = Some(section);
                    } else if let Some(current) = last.as_mut() {
                        if current.color == color {
                            current.increase_height();
                            if let Some(stored) = sections.last_mut() {
                                stored.increase_height();
                            }
                        } else {
                            let averaged = average_argb(current.color, color);
                            let section = BeaconBeamSection::new(averaged);
                            sections.push(section.clone());
                            last = Some(section);
                        }
                    }
                }
                BeaconBeamBlock::Transparent | BeaconBeamBlock::Bedrock => {
                    if let Some(stored) = sections.last_mut() {
                        stored.increase_height();
                    } else {
                        let mut section = BeaconBeamSection::new(0xFFFF_FFFFu32 as i32);
                        section.increase_height();
                        sections.push(section.clone());
                        last = Some(section);
                    }
                }
                BeaconBeamBlock::Blocking => return Vec::new(),
            }
        }
        sections
    }

    pub fn effect_applications(&self) -> Vec<BeaconEffectApplication> {
        if self.levels <= 0 {
            return Vec::new();
        }
        let Some(primary) = self.primary_power.as_ref() else {
            return Vec::new();
        };
        let range = self.levels * 10 + 10;
        let duration_ticks = (9 + self.levels * 2) * 20;
        let mut out = vec![BeaconEffectApplication {
            effect: primary.clone(),
            duration_ticks,
            amplifier: if self.levels >= 4 && self.secondary_power.as_ref() == Some(primary) {
                1
            } else {
                0
            },
            range,
        }];
        if self.levels >= 4 {
            if let Some(secondary) = self.secondary_power.as_ref() {
                if secondary != primary {
                    out.push(BeaconEffectApplication {
                        effect: secondary.clone(),
                        duration_ticks,
                        amplifier: 0,
                        range,
                    });
                }
            }
        }
        out
    }
}

fn filter_beacon_effect(effect: &str) -> Option<String> {
    matches!(
        effect,
        "minecraft:speed"
            | "minecraft:haste"
            | "minecraft:resistance"
            | "minecraft:jump_boost"
            | "minecraft:strength"
            | "minecraft:regeneration"
    )
    .then(|| effect.to_string())
}

fn average_argb(left: i32, right: i32) -> i32 {
    let left = left as u32;
    let right = right as u32;
    let avg = |shift| (((left >> shift) & 0xFFu32) + ((right >> shift) & 0xFFu32)) / 2u32;
    ((avg(24) << 24) | (avg(16) << 16) | (avg(8) << 8) | avg(0)) as i32
}

impl LecternBlockEntity {
    pub const DATA_PAGE: i32 = 0;
    pub const SLOT_BOOK: usize = 0;
    pub const NUM_SLOTS: usize = 1;
    pub const PAGE_CHANGE_IMPULSE_TICKS: i32 = 2;
    pub const DISPLAY_NAME: &'static str = "container.lectern";

    pub fn new() -> Self {
        Self {
            book: None,
            page: 0,
            page_count: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = Vec::new();
        if let Some(book) = self.book.as_ref().filter(|book| !book.is_empty()) {
            entries.push(("Book".to_string(), book.to_tag()));
            entries.push(("Page".to_string(), Tag::Int(self.page)));
        }
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag, page_count: i32) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let book = entries
            .iter()
            .find(|(key, _)| key == "Book")
            .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        let mut lectern = Self {
            book,
            page: 0,
            page_count: page_count.max(0),
        };
        lectern.page = lectern.clamp_page(get_int(entries, "Page").unwrap_or(0));
        lectern
    }

    pub fn has_book(&self) -> bool {
        self.book.as_ref().is_some_and(|book| {
            !book.is_empty()
                && matches!(
                    book.item_id.as_str(),
                    "minecraft:written_book" | "minecraft:writable_book"
                )
        })
    }

    pub fn set_book(&mut self, book: Option<PotItemStack>, page_count: i32) {
        self.book = book.filter(|book| !book.is_empty());
        self.page = 0;
        self.page_count = if self.has_book() {
            page_count.max(0)
        } else {
            0
        };
    }

    pub fn clear_content(&mut self) {
        self.book = None;
        self.page = 0;
        self.page_count = 0;
    }

    pub fn set_page(&mut self, page: i32) -> bool {
        let new_page = self.clamp_page(page);
        let changed = self.page != new_page;
        self.page = new_page;
        changed
    }

    pub fn remove_book_no_update(&mut self) -> Option<PotItemStack> {
        let book = self.book.take();
        self.page = 0;
        self.page_count = 0;
        book
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "lectern",
            initial_slots: vec![self.book.clone()],
        }
    }

    pub fn get_redstone_signal(&self) -> u8 {
        if !self.has_book() {
            return 0;
        }
        let progress = if self.page_count > 1 {
            self.page as f32 / (self.page_count as f32 - 1.0)
        } else {
            1.0
        };
        (progress * 14.0).floor() as u8 + 1
    }

    fn clamp_page(&self, page: i32) -> i32 {
        if self.page_count <= 0 {
            0
        } else {
            page.clamp(0, self.page_count - 1)
        }
    }
}

impl HangingSignAttachment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wall => "wall",
            Self::Ceiling => "ceiling",
            Self::CeilingMiddle => "ceiling_middle",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "wall" => Some(Self::Wall),
            "ceiling" => Some(Self::Ceiling),
            "ceiling_middle" => Some(Self::CeilingMiddle),
            _ => None,
        }
    }
}

impl Default for SignLine {
    fn default() -> Self {
        Self {
            raw: String::new(),
            filtered: String::new(),
            click_command: None,
        }
    }
}

impl SignLine {
    pub fn new(raw: impl Into<String>, filtered: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            filtered: filtered.into(),
            click_command: None,
        }
    }

    pub fn with_click_command(mut self, command: impl Into<String>) -> Self {
        self.click_command = Some(command.into());
        self
    }

    pub fn visible_text(&self, should_filter: bool) -> &str {
        if should_filter {
            &self.filtered
        } else {
            &self.raw
        }
    }
}

impl Default for SignText {
    fn default() -> Self {
        Self {
            lines: std::array::from_fn(|_| SignLine::default()),
            color: DyeColor::Black,
            has_glowing_text: false,
        }
    }
}

impl SignText {
    pub const LINES: usize = 4;

    pub fn set_message(
        &mut self,
        index: usize,
        raw: impl Into<String>,
        filtered: impl Into<String>,
    ) -> bool {
        if index >= Self::LINES {
            return false;
        }
        self.lines[index] = SignLine::new(raw, filtered);
        true
    }

    pub fn has_message(&self, should_filter: bool) -> bool {
        self.lines
            .iter()
            .any(|line| !line.visible_text(should_filter).is_empty())
    }

    pub fn has_any_click_commands(&self, should_filter: bool) -> bool {
        self.lines.iter().any(|line| {
            !line.visible_text(should_filter).is_empty() && line.click_command.is_some()
        })
    }

    pub fn to_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "messages".to_string(),
                Tag::List(self.lines.iter().map(sign_line_to_tag).collect()),
            ),
            (
                "color".to_string(),
                Tag::String(self.color.vanilla_name().to_string()),
            ),
            (
                "has_glowing_text".to_string(),
                Tag::Byte(self.has_glowing_text as i8),
            ),
        ];
        if self.lines.iter().any(|line| line.filtered != line.raw) {
            fields.push((
                "filtered_messages".to_string(),
                Tag::List(
                    self.lines
                        .iter()
                        .map(|line| Tag::String(line.filtered.clone()))
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn from_tag(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        let mut text = Self::default();
        if let Some(Tag::List(messages)) = entries
            .iter()
            .find(|(name, _)| name == "messages")
            .map(|(_, tag)| tag)
        {
            for (index, message) in messages.iter().take(Self::LINES).enumerate() {
                text.lines[index] = sign_line_from_tag(message);
            }
        }
        if let Some(Tag::List(filtered_messages)) = entries
            .iter()
            .find(|(name, _)| name == "filtered_messages")
            .map(|(_, tag)| tag)
        {
            for (index, message) in filtered_messages.iter().take(Self::LINES).enumerate() {
                if let Tag::String(filtered) = message {
                    text.lines[index].filtered = filtered.clone();
                }
            }
        } else {
            for line in &mut text.lines {
                line.filtered = line.raw.clone();
            }
        }
        text.color = get_string(entries, "color")
            .and_then(DyeColor::from_vanilla_name)
            .unwrap_or(DyeColor::Black);
        text.has_glowing_text = get_bool(entries, "has_glowing_text").unwrap_or(false);
        text
    }
}

impl Default for SignBlockEntityModel {
    fn default() -> Self {
        Self {
            front_text: SignText::default(),
            back_text: SignText::default(),
            is_waxed: false,
            player_who_may_edit: None,
        }
    }
}

impl SignBlockEntityModel {
    pub const MAX_TEXT_LINE_WIDTH: i32 = 90;
    pub const TEXT_LINE_HEIGHT: i32 = 10;

    pub fn text(&self, front_text: bool) -> &SignText {
        if front_text {
            &self.front_text
        } else {
            &self.back_text
        }
    }

    pub fn text_mut(&mut self, front_text: bool) -> &mut SignText {
        if front_text {
            &mut self.front_text
        } else {
            &mut self.back_text
        }
    }

    pub fn set_allowed_player_editor(&mut self, player_uuid: Option<String>) {
        self.player_who_may_edit = player_uuid;
    }

    pub fn player_is_too_far_away_to_edit(&self, player_uuid: &str, distance: f64) -> bool {
        self.player_who_may_edit.as_deref() != Some(player_uuid) || distance > 4.0
    }

    pub fn tick_editing_player(&mut self, player_uuid: &str, distance: f64) -> bool {
        if self.player_is_too_far_away_to_edit(player_uuid, distance) {
            self.player_who_may_edit = None;
            true
        } else {
            false
        }
    }

    pub fn update_sign_text(
        &mut self,
        player_uuid: &str,
        front_text: bool,
        lines: [SignLine; 4],
        player_filters_text: bool,
    ) -> bool {
        if self.is_waxed || self.player_who_may_edit.as_deref() != Some(player_uuid) {
            return false;
        }
        let text = self.text_mut(front_text);
        if player_filters_text {
            for (slot, line) in lines.into_iter().enumerate() {
                text.lines[slot].raw = line.filtered.clone();
                text.lines[slot].filtered = line.filtered;
                text.lines[slot].click_command = line.click_command;
            }
        } else {
            text.lines = lines;
        }
        self.player_who_may_edit = None;
        true
    }

    pub fn set_waxed(&mut self, is_waxed: bool) -> bool {
        if self.is_waxed == is_waxed {
            false
        } else {
            self.is_waxed = is_waxed;
            true
        }
    }

    pub fn can_execute_click_commands(&self, front_text: bool, should_filter: bool) -> bool {
        self.is_waxed && self.text(front_text).has_any_click_commands(should_filter)
    }

    pub fn executable_click_commands(&self, front_text: bool, should_filter: bool) -> Vec<String> {
        if !self.is_waxed {
            return Vec::new();
        }
        self.text(front_text)
            .lines
            .iter()
            .filter(|line| !line.visible_text(should_filter).is_empty())
            .filter_map(|line| line.click_command.clone())
            .collect()
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("front_text".to_string(), self.front_text.to_tag()),
            ("back_text".to_string(), self.back_text.to_tag()),
            ("is_waxed".to_string(), Tag::Byte(self.is_waxed as i8)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        Self {
            front_text: entries
                .iter()
                .find(|(name, _)| name == "front_text")
                .map(|(_, tag)| SignText::from_tag(tag))
                .unwrap_or_default(),
            back_text: entries
                .iter()
                .find(|(name, _)| name == "back_text")
                .map(|(_, tag)| SignText::from_tag(tag))
                .unwrap_or_default(),
            is_waxed: get_bool(entries, "is_waxed").unwrap_or(false),
            player_who_may_edit: None,
        }
    }
}

impl HangingSignBlockEntityModel {
    pub const MAX_TEXT_LINE_WIDTH: i32 = 60;
    pub const TEXT_LINE_HEIGHT: i32 = 9;

    pub fn new(attachment: HangingSignAttachment) -> Self {
        Self {
            sign: SignBlockEntityModel::default(),
            attachment,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = match self.sign.save_additional() {
            Tag::Compound(entries) => entries,
            _ => Vec::new(),
        };
        entries.push((
            "attachment".to_string(),
            Tag::String(self.attachment.as_str().to_string()),
        ));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let sign = SignBlockEntityModel::load_additional(tag);
        let attachment = compound_entries(tag)
            .and_then(|entries| get_string(entries, "attachment"))
            .and_then(HangingSignAttachment::from_str)
            .unwrap_or(HangingSignAttachment::Ceiling);
        Self { sign, attachment }
    }
}

impl BrewingRecipe {
    pub const fn new(
        source_item: &'static str,
        source_potion: &'static str,
        ingredient: &'static str,
        result_item: &'static str,
        result_potion: &'static str,
    ) -> Self {
        Self {
            source_item,
            source_potion,
            ingredient,
            result_item,
            result_potion,
        }
    }

    fn applies_to(&self, stack: &PotItemStack, ingredient: &PotItemStack) -> bool {
        let (item, potion) = brewing_stack_parts(&stack.item_id);
        item == self.source_item
            && potion == Some(self.source_potion)
            && ingredient.item_id == self.ingredient
    }

    fn result_stack(&self, count: i32) -> PotItemStack {
        PotItemStack {
            item_id: brewing_stack_id(self.result_item, self.result_potion),
            count,
        }
    }
}

impl BrewingStandBlockEntity {
    pub const CONTAINER_SIZE: usize = 5;
    pub const INGREDIENT_SLOT: usize = 3;
    pub const FUEL_SLOT: usize = 4;
    pub const FUEL_USES: i32 = 20;
    pub const BREW_TIME: i32 = 400;
    pub const DATA_BREW_TIME: i32 = 0;
    pub const DATA_FUEL_USES: i32 = 1;
    pub const DISPLAY_NAME: &'static str = "container.brewing";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::CONTAINER_SIZE],
            brew_time: 0,
            fuel: 0,
            ingredient: None,
        }
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= Self::CONTAINER_SIZE {
            return false;
        }
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn potion_bits(&self) -> [bool; 3] {
        [
            self.items[0].is_some(),
            self.items[1].is_some(),
            self.items[2].is_some(),
        ]
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: "brewing_stand",
            initial_slots: self.items.clone(),
        }
    }

    pub fn is_brewable(&self, recipes: &[BrewingRecipe]) -> bool {
        let Some(ingredient) = self.items[Self::INGREDIENT_SLOT].as_ref() else {
            return false;
        };
        recipes.iter().any(|recipe| {
            self.items[..3]
                .iter()
                .flatten()
                .any(|stack| recipe.applies_to(stack, ingredient))
        })
    }

    pub fn server_tick(&mut self, recipes: &[BrewingRecipe]) -> BrewingStandTickResult {
        if self.fuel <= 0
            && self.items[Self::FUEL_SLOT]
                .as_ref()
                .is_some_and(is_brewing_fuel)
        {
            self.fuel = Self::FUEL_USES;
            shrink_stack(&mut self.items[Self::FUEL_SLOT], 1);
            return BrewingStandTickResult::FuelLoaded;
        }

        let brewable = self.is_brewable(recipes);
        let ingredient_id = self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .map(|stack| stack.item_id.clone());
        if self.brew_time > 0 {
            self.brew_time -= 1;
            if self.brew_time == 0 && brewable {
                self.do_brew(recipes);
                return BrewingStandTickResult::Brewed;
            }
            if !brewable || ingredient_id != self.ingredient {
                self.brew_time = 0;
                return BrewingStandTickResult::Cancelled;
            }
            return BrewingStandTickResult::Brewing;
        }

        if brewable && self.fuel > 0 {
            self.fuel -= 1;
            self.brew_time = Self::BREW_TIME;
            self.ingredient = ingredient_id;
            return BrewingStandTickResult::Started;
        }

        BrewingStandTickResult::Idle
    }

    fn do_brew(&mut self, recipes: &[BrewingRecipe]) {
        let Some(ingredient) = self.items[Self::INGREDIENT_SLOT].as_ref().cloned() else {
            return;
        };
        for slot in 0..3 {
            let Some(stack) = self.items[slot].as_ref() else {
                continue;
            };
            if let Some(recipe) = recipes
                .iter()
                .find(|recipe| recipe.applies_to(stack, &ingredient))
            {
                self.items[slot] = Some(recipe.result_stack(stack.count));
            }
        }
        shrink_stack(&mut self.items[Self::INGREDIENT_SLOT], 1);
        self.ingredient = self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .map(|stack| stack.item_id.clone());
    }

    pub fn can_place_item(
        &self,
        slot: usize,
        stack: &PotItemStack,
        recipes: &[BrewingRecipe],
    ) -> bool {
        match slot {
            Self::INGREDIENT_SLOT => recipes
                .iter()
                .any(|recipe| recipe.ingredient == stack.item_id),
            Self::FUEL_SLOT => is_brewing_fuel(stack),
            0..=2 => {
                is_brewing_container(&stack.item_id)
                    && self.items.get(slot).is_some_and(Option::is_none)
            }
            _ => false,
        }
    }

    pub fn slots_for_face(direction: Direction) -> &'static [usize] {
        match direction {
            Direction::Up => &[Self::INGREDIENT_SLOT],
            Direction::Down => &[0, 1, 2, Self::INGREDIENT_SLOT],
            _ => &[0, 1, 2, Self::FUEL_SLOT],
        }
    }

    pub fn can_take_item_through_face(
        slot: usize,
        stack: &PotItemStack,
        _direction: Direction,
    ) -> bool {
        slot != Self::INGREDIENT_SLOT || stack.item_id == "minecraft:glass_bottle"
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(&self.items)
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("BrewTime".to_string(), Tag::Short(self.brew_time as i16)),
            ("Items".to_string(), container_items_tag(&self.items)),
            ("Fuel".to_string(), Tag::Byte(self.fuel as i8)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut stand = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return stand;
        };
        load_container_items(entries, &mut stand.items);
        stand.brew_time = get_short(entries, "BrewTime").unwrap_or(0) as i32;
        stand.fuel = get_byte(entries, "Fuel").unwrap_or(0) as i32;
        if stand.brew_time > 0 {
            stand.ingredient = stand.items[Self::INGREDIENT_SLOT]
                .as_ref()
                .map(|stack| stack.item_id.clone());
        }
        stand
    }
}

impl CrafterRecipe {
    pub fn matches(&self, crafter: &CrafterBlockEntity) -> bool {
        self.pattern.iter().enumerate().all(|(slot, expected)| {
            if crafter.disabled_slots[slot] {
                expected.is_none()
            } else {
                match (expected, crafter.items[slot].as_ref()) {
                    (Some(expected), Some(stack)) => stack.item_id == *expected && stack.count > 0,
                    (None, None) => true,
                    _ => false,
                }
            }
        })
    }
}

impl CrafterBlockEntity {
    pub const CONTAINER_WIDTH: usize = 3;
    pub const CONTAINER_HEIGHT: usize = 3;
    pub const CONTAINER_SIZE: usize = 9;
    pub const DATA_TRIGGERED: usize = 9;
    pub const NUM_DATA: usize = 10;
    pub const SLOT_DISABLED: i32 = 1;
    pub const SLOT_ENABLED: i32 = 0;
    pub const MAX_STACK_SIZE: i32 = 64;
    pub const MAX_CRAFTING_TICKS: i32 = 6;
    pub const CRAFTING_TICK_DELAY: i32 = 4;
    pub const DISPLAY_NAME: &'static str = "container.crafter";

    pub fn new() -> Self {
        Self {
            items: vec![None; Self::CONTAINER_SIZE],
            disabled_slots: [false; Self::CONTAINER_SIZE],
            triggered: false,
            crafting_ticks_remaining: 0,
        }
    }

    pub fn set_slot_state(&mut self, slot: usize, enabled: bool) -> bool {
        if !self.slot_can_be_disabled(slot) {
            return false;
        }
        self.disabled_slots[slot] = !enabled;
        true
    }

    pub fn is_slot_disabled(&self, slot: usize) -> bool {
        self.disabled_slots.get(slot).copied().unwrap_or(false)
    }

    pub fn set_triggered(&mut self, triggered: bool) {
        self.triggered = triggered;
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= Self::CONTAINER_SIZE {
            return false;
        }
        if self.is_slot_disabled(slot) {
            self.set_slot_state(slot, true);
        }
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn can_place_item(&self, slot: usize, stack: &PotItemStack) -> bool {
        if slot >= Self::CONTAINER_SIZE || self.is_slot_disabled(slot) || stack.is_empty() {
            return false;
        }
        let Some(slot_stack) = self.items[slot].as_ref() else {
            return true;
        };
        if slot_stack.count >= Self::MAX_STACK_SIZE {
            return false;
        }
        !self.smaller_stack_exists(slot_stack.count, slot_stack, slot)
    }

    fn smaller_stack_exists(
        &self,
        base_size: i32,
        base_item: &PotItemStack,
        base_slot: usize,
    ) -> bool {
        for slot in (base_slot + 1)..Self::CONTAINER_SIZE {
            if self.is_slot_disabled(slot) {
                continue;
            }
            match self.items[slot].as_ref() {
                None => return true,
                Some(stack) if stack.count < base_size && stack.item_id == base_item.item_id => {
                    return true
                }
                _ => {}
            }
        }
        false
    }

    fn slot_can_be_disabled(&self, slot: usize) -> bool {
        slot < Self::CONTAINER_SIZE && self.items[slot].is_none()
    }

    pub fn redstone_signal(&self) -> u8 {
        self.items
            .iter()
            .zip(self.disabled_slots)
            .filter(|(stack, disabled)| stack.is_some() || *disabled)
            .count() as u8
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        let mut initial_slots = self.items.clone();
        initial_slots.push(None);
        BlockEntityMenuOpen {
            container_id,
            menu_type: "crafter_3x3",
            initial_slots,
        }
    }

    pub fn server_tick(&mut self) -> bool {
        let next = self.crafting_ticks_remaining - 1;
        if next >= 0 {
            self.crafting_ticks_remaining = next;
            next == 0
        } else {
            false
        }
    }

    pub fn pulse_craft(&mut self, recipes: &[CrafterRecipe]) -> CrafterPulseResult {
        if !self.triggered {
            return CrafterPulseResult::NotTriggered;
        }
        let Some(recipe) = recipes.iter().find(|recipe| recipe.matches(self)) else {
            return CrafterPulseResult::NoRecipe;
        };
        self.crafting_ticks_remaining = Self::MAX_CRAFTING_TICKS;
        for stack in &mut self.items {
            if stack.is_some() {
                shrink_stack(stack, 1);
            }
        }
        CrafterPulseResult::Crafted {
            result: recipe.result.clone(),
            remaining_items: recipe.remaining_items.clone(),
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "crafting_ticks_remaining".to_string(),
                Tag::Int(self.crafting_ticks_remaining),
            ),
            ("Items".to_string(), container_items_tag(&self.items)),
            (
                "disabled_slots".to_string(),
                Tag::IntArray(
                    self.disabled_slots
                        .iter()
                        .enumerate()
                        .filter_map(|(slot, disabled)| disabled.then_some(slot as i32))
                        .collect(),
                ),
            ),
            ("triggered".to_string(), Tag::Int(self.triggered as i32)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut crafter = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return crafter;
        };
        load_container_items(entries, &mut crafter.items);
        crafter.crafting_ticks_remaining =
            get_int(entries, "crafting_ticks_remaining").unwrap_or(0);
        if let Some(Tag::IntArray(disabled_slots)) = entries
            .iter()
            .find(|(name, _)| name == "disabled_slots")
            .map(|(_, tag)| tag)
        {
            for slot in disabled_slots {
                if (0..Self::CONTAINER_SIZE as i32).contains(slot) {
                    crafter.disabled_slots[*slot as usize] = true;
                }
            }
        }
        crafter.triggered = get_int(entries, "triggered").unwrap_or(0) != 0;
        crafter
    }
}

impl Default for SpawnerCustomSpawnRules {
    fn default() -> Self {
        Self {
            block_light_limit: (0, 15),
            sky_light_limit: (0, 15),
            requires_no_sky_access: false,
        }
    }
}

impl SpawnerCustomSpawnRules {
    pub fn is_valid_position(&self, block_light: i32, sky_light: i32, no_sky_access: bool) -> bool {
        (self.block_light_limit.0..=self.block_light_limit.1).contains(&block_light)
            && (self.sky_light_limit.0..=self.sky_light_limit.1).contains(&sky_light)
            && (!self.requires_no_sky_access || no_sky_access)
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "block_light_limit".to_string(),
                Tag::IntArray(vec![self.block_light_limit.0, self.block_light_limit.1]),
            ),
            (
                "sky_light_limit".to_string(),
                Tag::IntArray(vec![self.sky_light_limit.0, self.sky_light_limit.1]),
            ),
            (
                "requires_no_sky_access".to_string(),
                Tag::Byte(i8::from(self.requires_no_sky_access)),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            block_light_limit: int_range_field(entries, "block_light_limit").unwrap_or((0, 15)),
            sky_light_limit: int_range_field(entries, "sky_light_limit").unwrap_or((0, 15)),
            requires_no_sky_access: get_byte(entries, "requires_no_sky_access").unwrap_or(0) != 0,
        })
    }
}

impl Default for SpawnDataModel {
    fn default() -> Self {
        Self::new("minecraft:pig")
    }
}

impl SpawnDataModel {
    pub fn new(entity_id: impl Into<String>) -> Self {
        Self {
            entity: Tag::Compound(vec![("id".to_string(), Tag::String(entity_id.into()))]),
            custom_spawn_rules: None,
            equipment: None,
            weight: 1,
        }
    }

    pub fn entity_id(&self) -> Option<&str> {
        compound_entries(&self.entity).and_then(|entries| get_string(entries, "id"))
    }

    fn to_tag(&self) -> Tag {
        let mut entries = vec![("entity".to_string(), self.entity.clone())];
        if let Some(rules) = &self.custom_spawn_rules {
            entries.push(("custom_spawn_rules".to_string(), rules.to_tag()));
        }
        if let Some(equipment) = &self.equipment {
            entries.push(("equipment".to_string(), equipment.clone()));
        }
        if self.weight != 1 {
            entries.push(("weight".to_string(), Tag::Int(self.weight)));
        }
        Tag::Compound(entries)
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let entity = entries
            .iter()
            .find(|(name, _)| name == "entity")
            .map(|(_, tag)| tag.clone())
            .unwrap_or_else(|| {
                Tag::Compound(vec![(
                    "id".to_string(),
                    Tag::String("minecraft:pig".to_string()),
                )])
            });
        let custom_spawn_rules = entries
            .iter()
            .find(|(name, _)| name == "custom_spawn_rules")
            .and_then(|(_, tag)| SpawnerCustomSpawnRules::from_tag(tag));
        let equipment = entries
            .iter()
            .find(|(name, _)| name == "equipment")
            .map(|(_, tag)| tag.clone());
        let weight = get_int(entries, "weight").unwrap_or(1).max(1);
        Some(Self {
            entity,
            custom_spawn_rules,
            equipment,
            weight,
        })
    }
}

impl Default for SpawnerBlockEntity {
    fn default() -> Self {
        let config = SpawnerConfig::default();
        Self {
            spawn_delay: config.spawn_delay,
            min_spawn_delay: config.min_spawn_delay,
            max_spawn_delay: config.max_spawn_delay,
            spawn_count: config.spawn_count,
            max_nearby_entities: config.max_nearby_entities,
            required_player_range: config.required_player_range,
            spawn_range: config.spawn_range,
            spawn_potentials: vec![SpawnDataModel::default()],
            next_spawn_data: None,
            spin: 0.0,
            old_spin: 0.0,
        }
    }
}

impl SpawnerBlockEntity {
    pub const EVENT_SPAWN: i32 = 1;

    pub fn config(&self) -> SpawnerConfig {
        SpawnerConfig {
            spawn_delay: self.spawn_delay,
            min_spawn_delay: self.min_spawn_delay,
            max_spawn_delay: self.max_spawn_delay,
            spawn_count: self.spawn_count,
            max_nearby_entities: self.max_nearby_entities,
            required_player_range: self.required_player_range,
            spawn_range: self.spawn_range,
        }
    }

    pub fn get_or_create_next_spawn_data(&mut self, random_roll: usize) -> &SpawnDataModel {
        if self.next_spawn_data.is_none() {
            let selected = weighted_spawn_data(&self.spawn_potentials, random_roll)
                .cloned()
                .unwrap_or_default();
            self.next_spawn_data = Some(selected);
        }
        self.next_spawn_data.as_ref().unwrap()
    }

    pub fn set_entity_id(&mut self, entity_id: impl Into<String>) {
        let entity_id = entity_id.into();
        let data = self
            .next_spawn_data
            .get_or_insert_with(SpawnDataModel::default);
        data.entity = Tag::Compound(vec![("id".to_string(), Tag::String(entity_id))]);
    }

    pub fn delay(&mut self, random_roll: i32) {
        self.spawn_delay = if self.max_spawn_delay <= self.min_spawn_delay {
            self.min_spawn_delay
        } else {
            self.min_spawn_delay
                + random_roll.rem_euclid(self.max_spawn_delay - self.min_spawn_delay)
        };
    }

    pub fn server_tick(
        &mut self,
        player_in_range: bool,
        spawner_blocks_work: bool,
        nearby_entities: i32,
        random_roll: i32,
    ) -> SpawnerTickResult {
        match spawner_tick_plan(
            self.config(),
            player_in_range,
            spawner_blocks_work,
            nearby_entities,
        ) {
            SpawnerTickPlan::Idle => SpawnerTickResult::Idle,
            SpawnerTickPlan::CountDown { next_delay } => {
                self.spawn_delay = next_delay;
                SpawnerTickResult::CountDown
            }
            SpawnerTickPlan::Delay { .. } => {
                self.delay(random_roll);
                SpawnerTickResult::Delay
            }
            SpawnerTickPlan::TrySpawn { attempts } => {
                let entity_id = self
                    .get_or_create_next_spawn_data(random_roll as usize)
                    .entity_id()
                    .unwrap_or("minecraft:pig")
                    .to_string();
                SpawnerTickResult::TrySpawn {
                    entity_id,
                    attempts,
                }
            }
        }
    }

    pub fn server_tick_with_context(&mut self, context: SpawnerSpawnContext) -> SpawnerTickResult {
        match self.server_tick(
            context.player_in_range,
            context.spawner_blocks_work,
            context.nearby_entities,
            context.delay_roll,
        ) {
            SpawnerTickResult::TrySpawn {
                entity_id,
                attempts,
            } => {
                if context.nearby_entities >= self.max_nearby_entities {
                    self.finish_spawn_cycle(context.delay_roll, context.potential_roll);
                    return SpawnerTickResult::MobCapReached;
                }
                let rules = self
                    .next_spawn_data
                    .as_ref()
                    .and_then(|data| data.custom_spawn_rules.as_ref());
                let custom_rules_ok = rules
                    .map(|rules| {
                        rules.is_valid_position(
                            context.block_light,
                            context.sky_light,
                            context.no_sky_access,
                        )
                    })
                    .unwrap_or(true);
                if !custom_rules_ok
                    || !context.collision_free
                    || !context.spawn_rules_ok
                    || !context.obstruction_free
                {
                    return SpawnerTickResult::SpawnRulesFailed;
                }
                self.finish_spawn_cycle(context.delay_roll, context.potential_roll);
                SpawnerTickResult::Spawned {
                    entity_id,
                    count: attempts,
                }
            }
            other => other,
        }
    }

    pub fn finish_spawn_cycle(&mut self, random_roll: i32, potential_roll: usize) {
        self.delay(random_roll);
        if let Some(data) = weighted_spawn_data(&self.spawn_potentials, potential_roll).cloned() {
            self.next_spawn_data = Some(data);
        }
    }

    pub fn client_tick(&mut self, player_in_range: bool) {
        if !player_in_range {
            self.old_spin = self.spin;
            return;
        }
        if self.spawn_delay > 0 {
            self.spawn_delay -= 1;
        }
        self.old_spin = self.spin;
        self.spin = (self.spin + 1000.0 / (self.spawn_delay as f64 + 200.0)) % 360.0;
    }

    pub fn on_event_triggered(&mut self, client_side: bool, event_id: i32) -> bool {
        if event_id != Self::EVENT_SPAWN {
            return false;
        }
        if client_side {
            self.spawn_delay = self.min_spawn_delay;
        }
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = vec![
            ("Delay".to_string(), Tag::Short(self.spawn_delay as i16)),
            (
                "MinSpawnDelay".to_string(),
                Tag::Short(self.min_spawn_delay as i16),
            ),
            (
                "MaxSpawnDelay".to_string(),
                Tag::Short(self.max_spawn_delay as i16),
            ),
            (
                "SpawnCount".to_string(),
                Tag::Short(self.spawn_count as i16),
            ),
            (
                "MaxNearbyEntities".to_string(),
                Tag::Short(self.max_nearby_entities as i16),
            ),
            (
                "RequiredPlayerRange".to_string(),
                Tag::Short(self.required_player_range as i16),
            ),
            (
                "SpawnRange".to_string(),
                Tag::Short(self.spawn_range as i16),
            ),
        ];
        if let Some(data) = &self.next_spawn_data {
            entries.push(("SpawnData".to_string(), data.to_tag()));
        }
        entries.push((
            "SpawnPotentials".to_string(),
            Tag::List(
                self.spawn_potentials
                    .iter()
                    .map(SpawnDataModel::to_tag)
                    .collect(),
            ),
        ));
        Tag::Compound(entries)
    }

    pub fn update_tag(&self) -> Tag {
        let Tag::Compound(mut entries) = self.save_additional() else {
            return Tag::Compound(Vec::new());
        };
        entries.retain(|(name, _)| name != "SpawnPotentials");
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut spawner = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return spawner;
        };
        spawner.spawn_delay = get_short(entries, "Delay").unwrap_or(20);
        spawner.min_spawn_delay = get_short(entries, "MinSpawnDelay").unwrap_or(200);
        spawner.max_spawn_delay = get_short(entries, "MaxSpawnDelay").unwrap_or(800);
        spawner.spawn_count = get_short(entries, "SpawnCount").unwrap_or(4);
        spawner.max_nearby_entities = get_short(entries, "MaxNearbyEntities").unwrap_or(6);
        spawner.required_player_range = get_short(entries, "RequiredPlayerRange").unwrap_or(16);
        spawner.spawn_range = get_short(entries, "SpawnRange").unwrap_or(4);
        spawner.next_spawn_data = entries
            .iter()
            .find(|(name, _)| name == "SpawnData")
            .and_then(|(_, tag)| SpawnDataModel::from_tag(tag));
        if let Some(Tag::List(potentials)) = entries
            .iter()
            .find(|(name, _)| name == "SpawnPotentials")
            .map(|(_, tag)| tag)
        {
            spawner.spawn_potentials = potentials
                .iter()
                .filter_map(SpawnDataModel::from_tag)
                .collect();
        }
        if spawner.spawn_potentials.is_empty() {
            spawner.spawn_potentials = vec![spawner.next_spawn_data.clone().unwrap_or_default()];
        }
        spawner
    }
}

impl TrialSpawnerStateModel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::WaitingForPlayers => "waiting_for_players",
            Self::Active => "active",
            Self::WaitingForRewardEjection => "waiting_for_reward_ejection",
            Self::EjectingReward => "ejecting_reward",
            Self::Cooldown => "cooldown",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "inactive" => Some(Self::Inactive),
            "waiting_for_players" => Some(Self::WaitingForPlayers),
            "active" => Some(Self::Active),
            "waiting_for_reward_ejection" => Some(Self::WaitingForRewardEjection),
            "ejecting_reward" => Some(Self::EjectingReward),
            "cooldown" => Some(Self::Cooldown),
            _ => None,
        }
    }

    pub fn light_level(self) -> i32 {
        match self {
            Self::Inactive | Self::Cooldown => 0,
            Self::WaitingForPlayers => 4,
            Self::Active | Self::WaitingForRewardEjection | Self::EjectingReward => 8,
        }
    }
}

impl Default for TrialSpawnerConfigModel {
    fn default() -> Self {
        Self {
            spawn_range: 4,
            total_mobs: 6.0,
            simultaneous_mobs: 2.0,
            total_mobs_added_per_player: 2.0,
            simultaneous_mobs_added_per_player: 1.0,
            ticks_between_spawn: 40,
            spawn_potentials: Vec::new(),
            loot_tables_to_eject: vec![
                "minecraft:spawners/trial_chamber/consumables".to_string(),
                "minecraft:spawners/trial_chamber/key".to_string(),
            ],
            items_to_drop_when_ominous:
                "minecraft:spawners/trial_chamber/items_to_drop_when_ominous".to_string(),
        }
    }
}

impl TrialSpawnerConfigModel {
    pub fn target_total_mobs(&self, additional_players: usize) -> i32 {
        (self.total_mobs + self.total_mobs_added_per_player * additional_players as f32).floor()
            as i32
    }

    pub fn target_simultaneous_mobs(&self, additional_players: usize) -> i32 {
        (self.simultaneous_mobs
            + self.simultaneous_mobs_added_per_player * additional_players as f32)
            .floor() as i32
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("spawn_range".to_string(), Tag::Int(self.spawn_range)),
            ("total_mobs".to_string(), Tag::Float(self.total_mobs)),
            (
                "simultaneous_mobs".to_string(),
                Tag::Float(self.simultaneous_mobs),
            ),
            (
                "total_mobs_added_per_player".to_string(),
                Tag::Float(self.total_mobs_added_per_player),
            ),
            (
                "simultaneous_mobs_added_per_player".to_string(),
                Tag::Float(self.simultaneous_mobs_added_per_player),
            ),
            (
                "ticks_between_spawn".to_string(),
                Tag::Int(self.ticks_between_spawn),
            ),
            (
                "spawn_potentials".to_string(),
                Tag::List(
                    self.spawn_potentials
                        .iter()
                        .map(SpawnDataModel::to_tag)
                        .collect(),
                ),
            ),
            (
                "loot_tables_to_eject".to_string(),
                Tag::List(
                    self.loot_tables_to_eject
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "items_to_drop_when_ominous".to_string(),
                Tag::String(self.items_to_drop_when_ominous.clone()),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        config.spawn_range = get_int(entries, "spawn_range").unwrap_or(config.spawn_range);
        config.total_mobs = get_float(entries, "total_mobs").unwrap_or(config.total_mobs);
        config.simultaneous_mobs =
            get_float(entries, "simultaneous_mobs").unwrap_or(config.simultaneous_mobs);
        config.total_mobs_added_per_player = get_float(entries, "total_mobs_added_per_player")
            .unwrap_or(config.total_mobs_added_per_player);
        config.simultaneous_mobs_added_per_player =
            get_float(entries, "simultaneous_mobs_added_per_player")
                .unwrap_or(config.simultaneous_mobs_added_per_player);
        config.ticks_between_spawn =
            get_int(entries, "ticks_between_spawn").unwrap_or(config.ticks_between_spawn);
        if let Some(Tag::List(values)) = entries
            .iter()
            .find(|(name, _)| name == "spawn_potentials")
            .map(|(_, tag)| tag)
        {
            config.spawn_potentials = values.iter().filter_map(SpawnDataModel::from_tag).collect();
        }
        if let Some(values) = string_list_field(entries, "loot_tables_to_eject") {
            config.loot_tables_to_eject = values;
        }
        config.items_to_drop_when_ominous = get_string(entries, "items_to_drop_when_ominous")
            .unwrap_or(&config.items_to_drop_when_ominous)
            .to_string();
        config
    }
}

impl Default for TrialSpawnerFullConfigModel {
    fn default() -> Self {
        Self {
            normal_config: TrialSpawnerConfigModel::default(),
            ominous_config: TrialSpawnerConfigModel::default(),
            target_cooldown_length: 36_000,
            required_player_range: 14,
        }
    }
}

impl TrialSpawnerFullConfigModel {
    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("normal_config".to_string(), self.normal_config.to_tag()),
            ("ominous_config".to_string(), self.ominous_config.to_tag()),
            (
                "target_cooldown_length".to_string(),
                Tag::Int(self.target_cooldown_length),
            ),
            (
                "required_player_range".to_string(),
                Tag::Int(self.required_player_range),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "normal_config") {
            config.normal_config = TrialSpawnerConfigModel::from_tag(tag);
        }
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "ominous_config") {
            config.ominous_config = TrialSpawnerConfigModel::from_tag(tag);
        }
        config.target_cooldown_length =
            get_int(entries, "target_cooldown_length").unwrap_or(config.target_cooldown_length);
        config.required_player_range =
            get_int(entries, "required_player_range").unwrap_or(config.required_player_range);
        config
    }
}

impl Default for TrialSpawnerBlockEntity {
    fn default() -> Self {
        Self {
            state: TrialSpawnerStateModel::Inactive,
            is_ominous: false,
            config: TrialSpawnerFullConfigModel::default(),
            detected_players: Vec::new(),
            current_mobs: Vec::new(),
            cooldown_ends_at: 0,
            next_mob_spawns_at: 0,
            total_mobs_spawned: 0,
            next_spawn_data: None,
            ejecting_loot_table: None,
            spin: 0.0,
            old_spin: 0.0,
        }
    }
}

impl TrialSpawnerBlockEntity {
    pub const DETECT_PLAYER_SPAWN_BUFFER: i64 = 40;
    pub const TIME_BETWEEN_REWARD_EJECTIONS: i64 = 30;
    pub const TICKS_BETWEEN_OMINOUS_ITEM_SPAWNERS: i64 = 160;

    pub fn active_config(&self) -> &TrialSpawnerConfigModel {
        if self.is_ominous {
            &self.config.ominous_config
        } else {
            &self.config.normal_config
        }
    }

    pub fn apply_ominous(&mut self, game_time: i64) {
        self.is_ominous = true;
        self.current_mobs.clear();
        self.total_mobs_spawned = 0;
        self.next_spawn_data = None;
        self.next_mob_spawns_at =
            game_time + i64::from(self.config.ominous_config.ticks_between_spawn);
        self.cooldown_ends_at = game_time + Self::TICKS_BETWEEN_OMINOUS_ITEM_SPAWNERS;
    }

    pub fn override_entity_to_spawn(&mut self, entity_id: impl Into<String>) {
        let data = SpawnDataModel::new(entity_id);
        self.config.normal_config.spawn_potentials = vec![data.clone()];
        self.config.ominous_config.spawn_potentials = vec![data];
        self.next_spawn_data = None;
        self.state = TrialSpawnerStateModel::Inactive;
        self.detected_players.clear();
        self.current_mobs.clear();
        self.total_mobs_spawned = 0;
    }

    pub fn tick_server(&mut self, context: TrialSpawnerTickContext) -> TrialSpawnerTickResult {
        self.current_mobs
            .truncate(context.current_mobs_alive.min(self.current_mobs.len()));
        if context.apply_ominous && !self.is_ominous {
            self.apply_ominous(context.game_time);
            return TrialSpawnerTickResult::BecameOminous;
        }

        match self.state {
            TrialSpawnerStateModel::Inactive => {
                self.state = TrialSpawnerStateModel::WaitingForPlayers;
                TrialSpawnerTickResult::StateChanged(self.state)
            }
            TrialSpawnerStateModel::WaitingForPlayers => {
                if !context.can_spawn_in_level || self.active_config().spawn_potentials.is_empty() {
                    return TrialSpawnerTickResult::Waiting;
                }
                self.detect_players(context.detected_player_count, context.game_time);
                if self.detected_players.is_empty() {
                    TrialSpawnerTickResult::Waiting
                } else {
                    self.state = TrialSpawnerStateModel::Active;
                    TrialSpawnerTickResult::DetectedPlayers(self.detected_players.len())
                }
            }
            TrialSpawnerStateModel::Active => {
                if !context.can_spawn_in_level {
                    self.state = TrialSpawnerStateModel::WaitingForPlayers;
                    return TrialSpawnerTickResult::StateChanged(self.state);
                }
                self.detect_players(context.detected_player_count, context.game_time);
                let additional_players = self.detected_players.len().saturating_sub(1);
                let target_total = self.active_config().target_total_mobs(additional_players);
                if self.total_mobs_spawned >= target_total {
                    if self.current_mobs.is_empty() {
                        self.cooldown_ends_at =
                            context.game_time + i64::from(self.config.target_cooldown_length);
                        self.total_mobs_spawned = 0;
                        self.next_mob_spawns_at = 0;
                        self.state = TrialSpawnerStateModel::WaitingForRewardEjection;
                        return TrialSpawnerTickResult::ReadyForRewards;
                    }
                    return TrialSpawnerTickResult::Waiting;
                }
                let simultaneous = self
                    .active_config()
                    .target_simultaneous_mobs(additional_players);
                if context.game_time >= self.next_mob_spawns_at
                    && (self.current_mobs.len() as i32) < simultaneous
                    && context.spawn_success
                {
                    let spawn_data = self.select_next_spawn_data(context.roll);
                    let entity_id = spawn_data
                        .entity_id()
                        .unwrap_or("minecraft:pig")
                        .to_string();
                    self.current_mobs
                        .push(format!("mob-{}", self.total_mobs_spawned + 1));
                    self.total_mobs_spawned += 1;
                    self.next_mob_spawns_at =
                        context.game_time + i64::from(self.active_config().ticks_between_spawn);
                    self.next_spawn_data =
                        weighted_spawn_data(&self.active_config().spawn_potentials, context.roll)
                            .cloned();
                    TrialSpawnerTickResult::SpawnMob { entity_id }
                } else {
                    TrialSpawnerTickResult::Waiting
                }
            }
            TrialSpawnerStateModel::WaitingForRewardEjection => {
                let cooldown_started_at =
                    self.cooldown_ends_at - i64::from(self.config.target_cooldown_length);
                if context.game_time >= cooldown_started_at + Self::DETECT_PLAYER_SPAWN_BUFFER {
                    self.state = TrialSpawnerStateModel::EjectingReward;
                    TrialSpawnerTickResult::StateChanged(self.state)
                } else {
                    TrialSpawnerTickResult::Waiting
                }
            }
            TrialSpawnerStateModel::EjectingReward => {
                let cooldown_started_at =
                    self.cooldown_ends_at - i64::from(self.config.target_cooldown_length);
                if (context.game_time - cooldown_started_at) % Self::TIME_BETWEEN_REWARD_EJECTIONS
                    != 0
                {
                    return TrialSpawnerTickResult::Waiting;
                }
                if self.detected_players.is_empty() {
                    self.ejecting_loot_table = None;
                    self.state = TrialSpawnerStateModel::Cooldown;
                    return TrialSpawnerTickResult::StateChanged(self.state);
                }
                let loot_table = self
                    .ejecting_loot_table
                    .clone()
                    .or_else(|| {
                        self.active_config()
                            .loot_tables_to_eject
                            .get(context.roll)
                            .cloned()
                    })
                    .unwrap_or_else(|| "minecraft:empty".to_string());
                self.ejecting_loot_table = Some(loot_table.clone());
                self.detected_players.remove(0);
                TrialSpawnerTickResult::EjectedReward {
                    loot_table,
                    remaining_players: self.detected_players.len(),
                }
            }
            TrialSpawnerStateModel::Cooldown => {
                self.detect_players(context.detected_player_count, context.game_time);
                if !self.detected_players.is_empty() {
                    self.total_mobs_spawned = 0;
                    self.next_mob_spawns_at = 0;
                    self.state = TrialSpawnerStateModel::Active;
                    TrialSpawnerTickResult::StateChanged(self.state)
                } else if context.game_time >= self.cooldown_ends_at {
                    self.is_ominous = false;
                    self.current_mobs.clear();
                    self.next_spawn_data = None;
                    self.ejecting_loot_table = None;
                    self.state = TrialSpawnerStateModel::WaitingForPlayers;
                    TrialSpawnerTickResult::CooldownFinished
                } else {
                    TrialSpawnerTickResult::Waiting
                }
            }
        }
    }

    fn detect_players(&mut self, count: usize, game_time: i64) {
        let previous_count = self.detected_players.len();
        for index in self.detected_players.len()..count {
            self.detected_players.push(format!("player-{index}"));
        }
        if self.detected_players.len() > previous_count {
            self.next_mob_spawns_at = self
                .next_mob_spawns_at
                .max(game_time + Self::DETECT_PLAYER_SPAWN_BUFFER);
        }
    }

    fn select_next_spawn_data(&mut self, roll: usize) -> SpawnDataModel {
        if let Some(data) = &self.next_spawn_data {
            return data.clone();
        }
        let data = weighted_spawn_data(&self.active_config().spawn_potentials, roll)
            .cloned()
            .unwrap_or_default();
        self.next_spawn_data = Some(data.clone());
        data
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![
            (
                "state".to_string(),
                Tag::String(self.state.as_str().to_string()),
            ),
            (
                "is_ominous".to_string(),
                Tag::Byte(i8::from(self.is_ominous)),
            ),
            ("config".to_string(), self.config.to_tag()),
            (
                "registered_players".to_string(),
                Tag::List(
                    self.detected_players
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "current_mobs".to_string(),
                Tag::List(
                    self.current_mobs
                        .iter()
                        .map(|value| Tag::String(value.clone()))
                        .collect(),
                ),
            ),
            (
                "cooldown_ends_at".to_string(),
                Tag::Long(self.cooldown_ends_at),
            ),
            (
                "next_mob_spawns_at".to_string(),
                Tag::Long(self.next_mob_spawns_at),
            ),
            (
                "total_mobs_spawned".to_string(),
                Tag::Int(self.total_mobs_spawned),
            ),
        ];
        if let Some(data) = &self.next_spawn_data {
            fields.push(("spawn_data".to_string(), data.to_tag()));
        }
        if let Some(loot_table) = &self.ejecting_loot_table {
            fields.push((
                "ejecting_loot_table".to_string(),
                Tag::String(loot_table.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn update_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if self.state == TrialSpawnerStateModel::Active {
            fields.push((
                "next_mob_spawns_at".to_string(),
                Tag::Long(self.next_mob_spawns_at),
            ));
        }
        if let Some(data) = &self.next_spawn_data {
            fields.push(("spawn_data".to_string(), data.to_tag()));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut spawner = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return spawner;
        };
        spawner.state = get_string(entries, "state")
            .and_then(TrialSpawnerStateModel::from_str)
            .unwrap_or(TrialSpawnerStateModel::Inactive);
        spawner.is_ominous = get_byte(entries, "is_ominous").unwrap_or(0) != 0;
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "config") {
            spawner.config = TrialSpawnerFullConfigModel::from_tag(tag);
        }
        spawner.detected_players =
            string_list_field(entries, "registered_players").unwrap_or_default();
        spawner.current_mobs = string_list_field(entries, "current_mobs").unwrap_or_default();
        spawner.cooldown_ends_at = get_long(entries, "cooldown_ends_at").unwrap_or(0);
        spawner.next_mob_spawns_at = get_long(entries, "next_mob_spawns_at").unwrap_or(0);
        spawner.total_mobs_spawned = get_int(entries, "total_mobs_spawned").unwrap_or(0);
        spawner.next_spawn_data = entries
            .iter()
            .find(|(name, _)| name == "spawn_data")
            .and_then(|(_, tag)| SpawnDataModel::from_tag(tag));
        spawner.ejecting_loot_table =
            get_string(entries, "ejecting_loot_table").map(ToString::to_string);
        spawner
    }
}

impl VaultStateModel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Unlocking => "unlocking",
            Self::Ejecting => "ejecting",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "inactive" => Some(Self::Inactive),
            "active" => Some(Self::Active),
            "unlocking" => Some(Self::Unlocking),
            "ejecting" => Some(Self::Ejecting),
            _ => None,
        }
    }

    pub fn light_level(self) -> i32 {
        match self {
            Self::Inactive => 6,
            Self::Active | Self::Unlocking | Self::Ejecting => 12,
        }
    }
}

impl Default for VaultConfigModel {
    fn default() -> Self {
        Self {
            loot_table: "minecraft:chests/trial_chambers/reward".to_string(),
            activation_range: 4.0,
            deactivation_range: 4.5,
            key_item: PotItemStack {
                item_id: "minecraft:trial_key".to_string(),
                count: 1,
            },
            override_loot_table_to_display: None,
        }
    }
}

impl VaultConfigModel {
    fn to_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "loot_table".to_string(),
                Tag::String(self.loot_table.clone()),
            ),
            (
                "activation_range".to_string(),
                Tag::Double(self.activation_range),
            ),
            (
                "deactivation_range".to_string(),
                Tag::Double(self.deactivation_range),
            ),
            ("key_item".to_string(), pot_item_to_tag(&self.key_item, 0)),
        ];
        if let Some(table) = &self.override_loot_table_to_display {
            fields.push((
                "override_loot_table_to_display".to_string(),
                Tag::String(table.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    fn from_tag(tag: &Tag) -> Self {
        let mut config = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return config;
        };
        config.loot_table = get_string(entries, "loot_table")
            .unwrap_or(&config.loot_table)
            .to_string();
        config.activation_range =
            get_double(entries, "activation_range").unwrap_or(config.activation_range);
        config.deactivation_range =
            get_double(entries, "deactivation_range").unwrap_or(config.deactivation_range);
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "key_item") {
            config.key_item = pot_item_from_tag(tag).unwrap_or(config.key_item);
        }
        config.override_loot_table_to_display =
            get_string(entries, "override_loot_table_to_display").map(ToString::to_string);
        config
    }
}

impl Default for VaultBlockEntity {
    fn default() -> Self {
        Self {
            state: VaultStateModel::Inactive,
            is_ominous: false,
            config: VaultConfigModel::default(),
            rewarded_players: BTreeSet::new(),
            connected_players: BTreeSet::new(),
            display_item: None,
            items_to_eject: Vec::new(),
            total_ejections_needed: 0,
            state_updating_resumes_at: 0,
            last_insert_fail_timestamp: 0,
            connected_particles_range: VaultConfigModel::default().deactivation_range,
            current_spin: 0.0,
            previous_spin: 0.0,
        }
    }
}

impl VaultBlockEntity {
    pub const UNLOCKING_DELAY_TICKS: i64 = 14;
    pub const STATE_UPDATE_RATE_TICKS: i64 = 20;
    pub const INSERT_FAIL_SOUND_BUFFER_TICKS: i64 = 15;
    pub const MAX_REWARDED_PLAYERS: usize = 128;

    pub fn tick_client(&mut self) {
        self.previous_spin = self.current_spin;
        self.current_spin = (self.current_spin + 10.0).rem_euclid(360.0);
    }

    pub fn tick_server(
        &mut self,
        game_time: i64,
        detected_players: &[String],
        display_roll: Option<PotItemStack>,
    ) -> VaultTickResult {
        if game_time % 20 == 0 && self.state == VaultStateModel::Active {
            self.display_item = display_roll;
            return VaultTickResult::DisplayItemCycled(self.display_item.clone());
        }
        if game_time < self.state_updating_resumes_at {
            return VaultTickResult::Waiting;
        }
        match self.state {
            VaultStateModel::Inactive => {
                self.update_connected_players(detected_players, self.config.activation_range);
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                if self.connected_players.is_empty() {
                    VaultTickResult::Waiting
                } else {
                    self.state = VaultStateModel::Active;
                    VaultTickResult::StateChanged(self.state)
                }
            }
            VaultStateModel::Active => {
                self.update_connected_players(detected_players, self.config.deactivation_range);
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                if self.connected_players.is_empty() {
                    self.state = VaultStateModel::Inactive;
                    self.display_item = None;
                    VaultTickResult::StateChanged(self.state)
                } else {
                    VaultTickResult::Waiting
                }
            }
            VaultStateModel::Unlocking => {
                self.state = VaultStateModel::Ejecting;
                self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                VaultTickResult::StateChanged(self.state)
            }
            VaultStateModel::Ejecting => {
                if let Some(item) = self.items_to_eject.pop() {
                    self.display_item = self.items_to_eject.last().cloned();
                    self.state_updating_resumes_at = game_time + Self::STATE_UPDATE_RATE_TICKS;
                    VaultTickResult::EjectedItem(item)
                } else {
                    self.total_ejections_needed = 0;
                    self.update_connected_players(detected_players, self.config.deactivation_range);
                    self.state = if self.connected_players.is_empty() {
                        VaultStateModel::Inactive
                    } else {
                        VaultStateModel::Active
                    };
                    VaultTickResult::EjectionFinished
                }
            }
        }
    }

    pub fn try_insert_key(
        &mut self,
        player: impl Into<String>,
        inserted: &PotItemStack,
        rewards: Vec<PotItemStack>,
        game_time: i64,
    ) -> VaultInsertResult {
        if self.state == VaultStateModel::Inactive {
            return VaultInsertResult::IgnoredInactive;
        }
        if inserted.item_id != self.config.key_item.item_id
            || inserted.count < self.config.key_item.count
        {
            if game_time >= self.last_insert_fail_timestamp + Self::INSERT_FAIL_SOUND_BUFFER_TICKS {
                self.last_insert_fail_timestamp = game_time;
            }
            return VaultInsertResult::WrongKey {
                expected: self.config.key_item.item_id.clone(),
            };
        }
        let player = player.into();
        if self.rewarded_players.contains(&player) {
            if game_time >= self.last_insert_fail_timestamp + Self::INSERT_FAIL_SOUND_BUFFER_TICKS {
                self.last_insert_fail_timestamp = game_time;
            }
            return VaultInsertResult::AlreadyRewarded;
        }
        if rewards.is_empty() {
            return VaultInsertResult::EmptyReward;
        }
        self.items_to_eject = rewards;
        self.total_ejections_needed = self.items_to_eject.len() as i32;
        self.display_item = self.items_to_eject.last().cloned();
        self.state = VaultStateModel::Unlocking;
        self.state_updating_resumes_at = game_time + Self::UNLOCKING_DELAY_TICKS;
        self.add_rewarded_player(player);
        VaultInsertResult::Unlocking {
            items_to_eject: self.items_to_eject.len(),
        }
    }

    fn add_rewarded_player(&mut self, player: String) {
        self.rewarded_players.insert(player);
        while self.rewarded_players.len() > Self::MAX_REWARDED_PLAYERS {
            if let Some(first) = self.rewarded_players.iter().next().cloned() {
                self.rewarded_players.remove(&first);
            }
        }
    }

    fn update_connected_players(&mut self, detected_players: &[String], range: f64) {
        self.connected_players = detected_players
            .iter()
            .filter(|player| !self.rewarded_players.contains(*player))
            .cloned()
            .collect();
        self.connected_particles_range = range;
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "state".to_string(),
                Tag::String(self.state.as_str().to_string()),
            ),
            (
                "is_ominous".to_string(),
                Tag::Byte(i8::from(self.is_ominous)),
            ),
            ("config".to_string(), self.config.to_tag()),
            ("shared_data".to_string(), self.shared_data_tag()),
            ("server_data".to_string(), self.server_data_tag()),
        ])
    }

    pub fn get_update_tag(&self) -> Tag {
        Tag::Compound(vec![("shared_data".to_string(), self.shared_data_tag())])
    }

    fn shared_data_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "connected_players".to_string(),
                string_list_tag(self.connected_players.iter().cloned()),
            ),
            (
                "connected_particles_range".to_string(),
                Tag::Double(self.connected_particles_range),
            ),
        ];
        if let Some(item) = &self.display_item {
            fields.push(("display_item".to_string(), pot_item_to_tag(item, 0)));
        }
        Tag::Compound(fields)
    }

    fn server_data_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "rewarded_players".to_string(),
                string_list_tag(self.rewarded_players.iter().cloned()),
            ),
            (
                "state_updating_resumes_at".to_string(),
                Tag::Long(self.state_updating_resumes_at),
            ),
            (
                "items_to_eject".to_string(),
                Tag::List(
                    self.items_to_eject
                        .iter()
                        .enumerate()
                        .map(|(slot, item)| pot_item_to_tag(item, slot as i8))
                        .collect(),
                ),
            ),
            (
                "total_ejections_needed".to_string(),
                Tag::Int(self.total_ejections_needed),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut vault = Self::default();
        let Some(entries) = compound_entries(tag) else {
            return vault;
        };
        vault.state = get_string(entries, "state")
            .and_then(VaultStateModel::from_str)
            .unwrap_or(VaultStateModel::Inactive);
        vault.is_ominous = get_byte(entries, "is_ominous").unwrap_or(0) != 0;
        if let Some((_, tag)) = entries.iter().find(|(name, _)| name == "config") {
            vault.config = VaultConfigModel::from_tag(tag);
        }
        if let Some(shared) = entries
            .iter()
            .find(|(name, _)| name == "shared_data")
            .and_then(|(_, tag)| compound_entries(tag))
        {
            vault.connected_players = string_list_field(shared, "connected_players")
                .unwrap_or_default()
                .into_iter()
                .collect();
            vault.connected_particles_range = get_double(shared, "connected_particles_range")
                .unwrap_or(vault.config.deactivation_range);
            vault.display_item = shared
                .iter()
                .find(|(name, _)| name == "display_item")
                .and_then(|(_, tag)| pot_item_from_tag(tag));
        }
        if let Some(server) = entries
            .iter()
            .find(|(name, _)| name == "server_data")
            .and_then(|(_, tag)| compound_entries(tag))
        {
            vault.rewarded_players = string_list_field(server, "rewarded_players")
                .unwrap_or_default()
                .into_iter()
                .collect();
            vault.state_updating_resumes_at =
                get_long(server, "state_updating_resumes_at").unwrap_or(0);
            vault.total_ejections_needed = get_int(server, "total_ejections_needed").unwrap_or(0);
            if let Some(Tag::List(items)) = server
                .iter()
                .find(|(name, _)| name == "items_to_eject")
                .map(|(_, tag)| tag)
            {
                vault.items_to_eject = items.iter().filter_map(pot_item_from_tag).collect();
            }
        }
        vault
    }
}

impl DyeColor {
    fn vanilla_name(self) -> &'static str {
        match self {
            DyeColor::White => "white",
            DyeColor::Orange => "orange",
            DyeColor::Magenta => "magenta",
            DyeColor::LightBlue => "light_blue",
            DyeColor::Yellow => "yellow",
            DyeColor::Lime => "lime",
            DyeColor::Pink => "pink",
            DyeColor::Gray => "gray",
            DyeColor::LightGray => "light_gray",
            DyeColor::Cyan => "cyan",
            DyeColor::Purple => "purple",
            DyeColor::Blue => "blue",
            DyeColor::Brown => "brown",
            DyeColor::Green => "green",
            DyeColor::Red => "red",
            DyeColor::Black => "black",
        }
    }

    fn from_vanilla_name(value: &str) -> Option<Self> {
        match value {
            "white" => Some(DyeColor::White),
            "orange" => Some(DyeColor::Orange),
            "magenta" => Some(DyeColor::Magenta),
            "light_blue" => Some(DyeColor::LightBlue),
            "yellow" => Some(DyeColor::Yellow),
            "lime" => Some(DyeColor::Lime),
            "pink" => Some(DyeColor::Pink),
            "gray" => Some(DyeColor::Gray),
            "light_gray" => Some(DyeColor::LightGray),
            "cyan" => Some(DyeColor::Cyan),
            "purple" => Some(DyeColor::Purple),
            "blue" => Some(DyeColor::Blue),
            "brown" => Some(DyeColor::Brown),
            "green" => Some(DyeColor::Green),
            "red" => Some(DyeColor::Red),
            "black" => Some(DyeColor::Black),
            _ => None,
        }
    }
}

impl BannerPatternLayer {
    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("pattern".to_string(), Tag::String(self.pattern.clone())),
            (
                "color".to_string(),
                Tag::String(self.color.vanilla_name().to_string()),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            pattern: get_string(entries, "pattern")?.to_string(),
            color: DyeColor::from_vanilla_name(get_string(entries, "color")?)?,
        })
    }
}

impl BannerBlockEntity {
    pub const MAX_PATTERNS: usize = 6;

    pub fn from_block_state(block_state: &str) -> Option<Self> {
        let name = block_state.strip_prefix("minecraft:")?;
        let color_name = name
            .strip_suffix("_wall_banner")
            .or_else(|| name.strip_suffix("_banner"))?;
        Some(Self {
            base_color: DyeColor::from_vanilla_name(color_name)?,
            patterns: Vec::new(),
            custom_name: None,
        })
    }

    pub fn add_pattern(&mut self, pattern: impl Into<String>, color: DyeColor) -> bool {
        if self.patterns.len() >= Self::MAX_PATTERNS {
            return false;
        }
        self.patterns.push(BannerPatternLayer {
            pattern: pattern.into(),
            color,
        });
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.patterns.is_empty() {
            fields.push((
                "patterns".to_string(),
                Tag::List(
                    self.patterns
                        .iter()
                        .map(BannerPatternLayer::to_tag)
                        .collect(),
                ),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(block_state: &str, tag: &Tag) -> Option<Self> {
        let mut banner = Self::from_block_state(block_state)?;
        let entries = compound_entries(tag);
        banner.custom_name = entries
            .and_then(|entries| get_string(entries, "CustomName"))
            .map(ToString::to_string);
        banner.patterns = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "patterns"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(BannerPatternLayer::from_tag)
                        .take(Self::MAX_PATTERNS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Some(banner)
    }
}

impl Default for PotDecorations {
    fn default() -> Self {
        Self {
            back: None,
            left: None,
            right: None,
            front: None,
        }
    }
}

impl PotDecorations {
    const BRICK: &'static str = "minecraft:brick";

    pub fn new(
        back: Option<String>,
        left: Option<String>,
        right: Option<String>,
        front: Option<String>,
    ) -> Self {
        Self {
            back: Self::normalize_side(back),
            left: Self::normalize_side(left),
            right: Self::normalize_side(right),
            front: Self::normalize_side(front),
        }
    }

    fn normalize_side(side: Option<String>) -> Option<String> {
        side.filter(|item| item != Self::BRICK)
    }

    pub fn ordered(&self) -> Vec<String> {
        [&self.back, &self.left, &self.right, &self.front]
            .into_iter()
            .map(|side| side.clone().unwrap_or_else(|| Self::BRICK.to_string()))
            .collect()
    }

    fn is_empty(&self) -> bool {
        self.back.is_none() && self.left.is_none() && self.right.is_none() && self.front.is_none()
    }

    fn to_tag(&self) -> Tag {
        Tag::List(self.ordered().into_iter().map(Tag::String).collect())
    }

    fn from_tag(tag: &Tag) -> Self {
        let Tag::List(values) = tag else {
            return Self::default();
        };
        let item = |index: usize| -> Option<String> {
            values.get(index).and_then(|tag| match tag {
                Tag::String(item) if item != Self::BRICK => Some(item.clone()),
                _ => None,
            })
        };
        Self::new(item(0), item(1), item(2), item(3))
    }
}

impl PotItemStack {
    fn is_empty(&self) -> bool {
        self.item_id == "minecraft:air" || self.count <= 0
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("id".to_string(), Tag::String(self.item_id.clone())),
            ("count".to_string(), Tag::Int(self.count)),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let stack = Self {
            item_id: get_string(entries, "id")?.to_string(),
            count: get_int(entries, "count").unwrap_or(1),
        };
        (!stack.is_empty()).then_some(stack)
    }
}

impl FurnaceBlockEntityKind {
    pub fn menu_type(self) -> &'static str {
        match self {
            Self::Furnace => "furnace",
            Self::BlastFurnace => "blast_furnace",
            Self::Smoker => "smoker",
        }
    }

    pub fn recipe_type(self) -> &'static str {
        match self {
            Self::Furnace => "smelting",
            Self::BlastFurnace => "blasting",
            Self::Smoker => "smoking",
        }
    }

    pub fn default_cooking_time(self) -> i32 {
        match self {
            Self::Furnace => 200,
            Self::BlastFurnace | Self::Smoker => 100,
        }
    }

    fn burn_duration(self, fuel_values: &FuelValues, fuel: Option<&PotItemStack>) -> i32 {
        let Some(fuel) = fuel else {
            return 0;
        };
        let burn = fuel_values.burn_duration(Some(fuel.item_id.as_str()));
        match self {
            Self::Furnace => burn,
            Self::BlastFurnace | Self::Smoker => burn / 2,
        }
    }
}

impl FurnaceCookingRecipe {
    pub fn new(
        recipe_id: &str,
        recipe_type: &str,
        input_item: &str,
        result_item: &str,
        cooking_time: i32,
        experience_millis: i32,
    ) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_type: recipe_type.to_string(),
            input_item: input_item.to_string(),
            result: PotItemStack {
                item_id: result_item.to_string(),
                count: 1,
            },
            cooking_time,
            experience_millis,
        }
    }
}

impl AbstractFurnaceBlockEntity {
    pub const INGREDIENT_SLOT: usize = 0;
    pub const FUEL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const SLOT_COUNT: usize = 3;
    pub const MAX_STACK_SIZE: i32 = 64;

    pub fn furnace() -> Self {
        Self::new(FurnaceBlockEntityKind::Furnace)
    }

    pub fn blast_furnace() -> Self {
        Self::new(FurnaceBlockEntityKind::BlastFurnace)
    }

    pub fn smoker() -> Self {
        Self::new(FurnaceBlockEntityKind::Smoker)
    }

    pub fn new(kind: FurnaceBlockEntityKind) -> Self {
        Self {
            kind,
            items: [None, None, None],
            lit_time_remaining: 0,
            lit_total_time: 0,
            cooking_time_spent: 0,
            cooking_total_time: kind.default_cooking_time(),
            recipes_used: BTreeMap::new(),
        }
    }

    pub fn set_item(
        &mut self,
        slot: usize,
        stack: Option<PotItemStack>,
        recipe: Option<&FurnaceCookingRecipe>,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        let same_input = slot == Self::INGREDIENT_SLOT && self.items[slot] == stack;
        self.items[slot] = stack.filter(|item| !item.is_empty());
        if slot == Self::INGREDIENT_SLOT && !same_input {
            self.cooking_total_time = recipe
                .filter(|recipe| self.recipe_matches(recipe))
                .map(|recipe| recipe.cooking_time)
                .unwrap_or_else(|| self.kind.default_cooking_time());
            self.cooking_time_spent = 0;
        }
        true
    }

    pub fn server_tick(
        &mut self,
        fuel_values: &FuelValues,
        recipe: Option<&FurnaceCookingRecipe>,
    ) -> FurnaceTickResult {
        let was_lit = self.is_lit();
        if self.lit_time_remaining > 0 {
            self.lit_time_remaining -= 1;
        }
        let is_lit_after_decrement = self.is_lit();
        let has_ingredient = self.items[Self::INGREDIENT_SLOT].is_some();
        let has_fuel = self.items[Self::FUEL_SLOT].is_some();

        if is_lit_after_decrement || has_fuel && has_ingredient {
            if let Some(recipe) = recipe.filter(|recipe| self.recipe_matches(recipe)) {
                self.cooking_total_time = recipe.cooking_time;
                if self.can_burn(recipe) {
                    if !self.is_lit() {
                        let new_lit_time = self
                            .kind
                            .burn_duration(fuel_values, self.items[Self::FUEL_SLOT].as_ref());
                        self.lit_time_remaining = new_lit_time;
                        self.lit_total_time = new_lit_time;
                        if new_lit_time > 0 {
                            self.consume_fuel();
                        }
                    }

                    if self.is_lit() {
                        self.cooking_time_spent += 1;
                        if self.cooking_time_spent == self.cooking_total_time {
                            self.cooking_time_spent = 0;
                            self.burn(recipe);
                            self.record_recipe(recipe);
                            return FurnaceTickResult::Burned {
                                output_count: self.items[Self::RESULT_SLOT]
                                    .as_ref()
                                    .map(|stack| stack.count)
                                    .unwrap_or(0),
                            };
                        }
                        return if was_lit != self.is_lit() {
                            FurnaceTickResult::LitChanged { lit: self.is_lit() }
                        } else {
                            FurnaceTickResult::Cooking
                        };
                    }
                }
                self.cooking_time_spent = 0;
            } else if has_ingredient {
                self.cooking_time_spent = 0;
            }
        } else if self.cooking_time_spent > 0 {
            self.cooking_time_spent =
                (self.cooking_time_spent - 2).clamp(0, self.cooking_total_time);
            return FurnaceTickResult::Cooling;
        }

        if was_lit != self.is_lit() {
            FurnaceTickResult::LitChanged { lit: self.is_lit() }
        } else {
            FurnaceTickResult::Idle
        }
    }

    pub fn is_lit(&self) -> bool {
        self.lit_time_remaining > 0
    }

    pub fn can_place_item(
        &self,
        slot: usize,
        stack: &PotItemStack,
        fuel_values: &FuelValues,
    ) -> bool {
        match slot {
            Self::RESULT_SLOT => false,
            Self::FUEL_SLOT => {
                fuel_values.is_fuel(stack.item_id.as_str())
                    || stack.item_id == "minecraft:bucket"
                        && self.items[Self::FUEL_SLOT]
                            .as_ref()
                            .is_none_or(|fuel| fuel.item_id != "minecraft:bucket")
            }
            _ => slot < Self::SLOT_COUNT,
        }
    }

    pub fn can_take_item_through_face(
        &self,
        slot: usize,
        item_id: &str,
        direction: Direction,
    ) -> bool {
        direction != Direction::Down
            || slot != Self::FUEL_SLOT
            || item_id == "minecraft:water_bucket"
            || item_id == "minecraft:bucket"
    }

    pub fn max_stack_size(&self, slot: usize, item: &PotItemStack) -> i32 {
        if slot == Self::FUEL_SLOT && item.item_id == "minecraft:bucket" {
            1
        } else {
            Self::MAX_STACK_SIZE
        }
    }

    pub fn get_slots_for_face(direction: Direction) -> &'static [usize] {
        match direction {
            Direction::Down => &[Self::RESULT_SLOT, Self::FUEL_SLOT],
            Direction::Up => &[Self::INGREDIENT_SLOT],
            _ => &[Self::FUEL_SLOT],
        }
    }

    pub fn comparator_output(&self) -> u8 {
        let non_empty = self.items.iter().filter(|stack| stack.is_some()).count();
        if non_empty == 0 {
            return 0;
        }
        let fullness: f32 = self
            .items
            .iter()
            .filter_map(|stack| stack.as_ref())
            .map(|stack| (stack.count.max(0) as f32 / Self::MAX_STACK_SIZE as f32).min(1.0))
            .sum::<f32>()
            / Self::SLOT_COUNT as f32;
        (1 + (fullness * 14.0).floor() as u8).min(MAX_SIGNAL)
    }

    pub fn open_menu(&self, container_id: i32) -> BlockEntityMenuOpen {
        BlockEntityMenuOpen {
            container_id,
            menu_type: self.kind.menu_type(),
            initial_slots: self.items.to_vec(),
        }
    }

    pub fn xp_to_award_and_clear(&mut self, fraction_roll: f32) -> i32 {
        let total = self
            .recipes_used
            .iter()
            .map(|(_, (times_used, experience_millis))| {
                if *times_used <= 0 || *experience_millis <= 0 {
                    return 0;
                }
                let total_millis = *times_used * *experience_millis;
                let whole = total_millis / 1000;
                let fraction = (total_millis % 1000) as f32 / 1000.0;
                if fraction != 0.0 && fraction_roll < fraction {
                    whole + 1
                } else {
                    whole
                }
            })
            .sum();
        self.recipes_used.clear();
        total
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "cooking_time_spent".to_string(),
                Tag::Short(self.cooking_time_spent as i16),
            ),
            (
                "cooking_total_time".to_string(),
                Tag::Short(self.cooking_total_time as i16),
            ),
            (
                "lit_time_remaining".to_string(),
                Tag::Short(self.lit_time_remaining as i16),
            ),
            (
                "lit_total_time".to_string(),
                Tag::Short(self.lit_total_time as i16),
            ),
            ("Items".to_string(), self.items_tag()),
            ("RecipesUsed".to_string(), self.recipes_used_tag()),
        ])
    }

    pub fn load_additional(kind: FurnaceBlockEntityKind, tag: &Tag) -> Self {
        let mut furnace = Self::new(kind);
        let Some(entries) = compound_entries(tag) else {
            return furnace;
        };
        furnace.cooking_time_spent = get_short(entries, "cooking_time_spent").unwrap_or(0).max(0);
        furnace.cooking_total_time = get_short(entries, "cooking_total_time")
            .unwrap_or(kind.default_cooking_time())
            .max(0);
        furnace.lit_time_remaining = get_short(entries, "lit_time_remaining").unwrap_or(0).max(0);
        furnace.lit_total_time = get_short(entries, "lit_total_time").unwrap_or(0).max(0);
        if let Some(Tag::List(items)) = entries
            .iter()
            .find(|(name, _)| name == "Items")
            .map(|(_, tag)| tag)
        {
            for item in items {
                if let Some(item_entries) = compound_entries(item) {
                    let slot = get_byte(item_entries, "Slot").unwrap_or(-1);
                    if (0..Self::SLOT_COUNT as i8).contains(&slot) {
                        furnace.items[slot as usize] = PotItemStack::from_tag(item);
                    }
                }
            }
        }
        if let Some(Tag::Compound(recipes)) = entries
            .iter()
            .find(|(name, _)| name == "RecipesUsed")
            .map(|(_, tag)| tag)
        {
            for (recipe_id, tag) in recipes {
                if let Tag::Compound(values) = tag {
                    let count = get_int(values, "count").unwrap_or(0).max(0);
                    let experience = get_int(values, "experience_millis").unwrap_or(0).max(0);
                    if count > 0 {
                        furnace
                            .recipes_used
                            .insert(recipe_id.clone(), (count, experience));
                    }
                }
            }
        }
        furnace
    }

    fn recipe_matches(&self, recipe: &FurnaceCookingRecipe) -> bool {
        recipe.recipe_type == self.kind.recipe_type()
            && self.items[Self::INGREDIENT_SLOT]
                .as_ref()
                .is_some_and(|input| input.item_id == recipe.input_item && input.count > 0)
    }

    fn can_burn(&self, recipe: &FurnaceCookingRecipe) -> bool {
        match &self.items[Self::RESULT_SLOT] {
            None => true,
            Some(result) if result.item_id == recipe.result.item_id => {
                result.count + recipe.result.count <= Self::MAX_STACK_SIZE
            }
            Some(_) => false,
        }
    }

    fn burn(&mut self, recipe: &FurnaceCookingRecipe) {
        match &mut self.items[Self::RESULT_SLOT] {
            Some(result) => result.count += recipe.result.count,
            slot @ None => *slot = Some(recipe.result.clone()),
        }
        if let Some(input) = &mut self.items[Self::INGREDIENT_SLOT] {
            input.count -= 1;
            if input.count <= 0 {
                self.items[Self::INGREDIENT_SLOT] = None;
            }
        }
        if self.items[Self::INGREDIENT_SLOT]
            .as_ref()
            .is_some_and(|input| input.item_id == "minecraft:wet_sponge")
            && self.items[Self::FUEL_SLOT]
                .as_ref()
                .is_some_and(|fuel| fuel.item_id == "minecraft:bucket")
        {
            self.items[Self::FUEL_SLOT] = Some(PotItemStack {
                item_id: "minecraft:water_bucket".to_string(),
                count: 1,
            });
        }
    }

    fn consume_fuel(&mut self) {
        if let Some(fuel) = &mut self.items[Self::FUEL_SLOT] {
            fuel.count -= 1;
            if fuel.count <= 0 {
                self.items[Self::FUEL_SLOT] = if fuel.item_id == "minecraft:lava_bucket" {
                    Some(PotItemStack {
                        item_id: "minecraft:bucket".to_string(),
                        count: 1,
                    })
                } else {
                    None
                };
            }
        }
    }

    fn record_recipe(&mut self, recipe: &FurnaceCookingRecipe) {
        let entry = self
            .recipes_used
            .entry(recipe.recipe_id.clone())
            .or_insert((0, recipe.experience_millis));
        entry.0 += 1;
        entry.1 = recipe.experience_millis;
    }

    fn items_tag(&self) -> Tag {
        Tag::List(
            self.items
                .iter()
                .enumerate()
                .filter_map(|(slot, item)| {
                    let mut tag = item.as_ref()?.to_tag();
                    if let Tag::Compound(entries) = &mut tag {
                        entries.insert(0, ("Slot".to_string(), Tag::Byte(slot as i8)));
                    }
                    Some(tag)
                })
                .collect(),
        )
    }

    fn recipes_used_tag(&self) -> Tag {
        Tag::Compound(
            self.recipes_used
                .iter()
                .map(|(recipe_id, (count, experience_millis))| {
                    (
                        recipe_id.clone(),
                        Tag::Compound(vec![
                            ("count".to_string(), Tag::Int(*count)),
                            (
                                "experience_millis".to_string(),
                                Tag::Int(*experience_millis),
                            ),
                        ]),
                    )
                })
                .collect(),
        )
    }
}

impl ContainerBlockEntityKind {
    pub fn size(self) -> usize {
        match self {
            Self::Dispenser | Self::Dropper => 9,
            Self::Hopper => 5,
            Self::Chest | Self::TrappedChest | Self::Barrel | Self::ShulkerBox => 27,
        }
    }

    pub fn menu_type(self) -> &'static str {
        match self {
            Self::Chest | Self::TrappedChest | Self::Barrel => "generic_9x3",
            Self::ShulkerBox => "shulker_box",
            Self::Dispenser | Self::Dropper => "generic_3x3",
            Self::Hopper => "hopper",
        }
    }

    pub fn default_name(self) -> &'static str {
        match self {
            Self::Chest | Self::TrappedChest => "container.chest",
            Self::Barrel => "container.barrel",
            Self::ShulkerBox => "container.shulkerBox",
            Self::Dispenser => "container.dispenser",
            Self::Dropper => "container.dropper",
            Self::Hopper => "container.hopper",
        }
    }
}

impl ContainerBlockEntityModel {
    pub const CHEST_LID_STEP: f32 = 0.1;
    pub const SHULKER_OPENING_TICK_LENGTH: i32 = 10;
    pub const SHULKER_MAX_LID_HEIGHT: f32 = 0.5;
    pub const SHULKER_MAX_LID_ROTATION: f32 = 270.0;
    pub const HOPPER_MOVE_ITEM_SPEED: i32 = 8;
    pub const HOPPER_NO_COOLDOWN: i32 = -1;
    pub const MAX_STACK_SIZE: i32 = 64;

    pub fn new(kind: ContainerBlockEntityKind) -> Self {
        Self {
            kind,
            items: vec![None; kind.size()],
            custom_name: None,
            lock_key: None,
            loot_table: None,
            loot_table_seed: 0,
            viewer_count: 0,
            lid_progress: 0.0,
            shulker_status: ShulkerBoxAnimationStatus::Closed,
            shulker_color: None,
            transfer_cooldown: if kind == ContainerBlockEntityKind::Hopper {
                Self::HOPPER_NO_COOLDOWN
            } else {
                0
            },
            facing: Direction::Down,
        }
    }

    pub fn can_open(&self, player_lock_key: Option<&str>, spectator: bool) -> bool {
        if self.loot_table.is_some() && spectator {
            return false;
        }
        self.lock_key
            .as_deref()
            .is_none_or(|lock| player_lock_key == Some(lock))
    }

    pub fn create_menu(
        &mut self,
        player_lock_key: Option<&str>,
        spectator: bool,
    ) -> Option<&'static str> {
        if !self.can_open(player_lock_key, spectator) {
            return None;
        }
        self.unpack_loot_table();
        Some(self.kind.menu_type())
    }

    pub fn open_menu(
        &mut self,
        container_id: i32,
        player_lock_key: Option<&str>,
        spectator: bool,
    ) -> Option<BlockEntityMenuOpen> {
        let menu_type = self.create_menu(player_lock_key, spectator)?;
        self.start_open();
        Some(BlockEntityMenuOpen {
            container_id,
            menu_type,
            initial_slots: self.items.clone(),
        })
    }

    pub fn close_menu(&mut self, menu: BlockEntityMenuOpen) -> BlockEntityMenuClose {
        self.stop_open();
        menu.close()
    }

    pub fn unpack_loot_table(&mut self) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            true
        } else {
            false
        }
    }

    pub fn set_item(&mut self, slot: usize, stack: Option<PotItemStack>) -> bool {
        if slot >= self.items.len() {
            return false;
        }
        self.unpack_loot_table();
        self.items[slot] = stack.filter(|stack| !stack.is_empty());
        true
    }

    pub fn start_open(&mut self) {
        self.viewer_count = (self.viewer_count + 1).max(0);
        if self.kind == ContainerBlockEntityKind::ShulkerBox && self.viewer_count == 1 {
            self.shulker_status = ShulkerBoxAnimationStatus::Opening;
        }
    }

    pub fn stop_open(&mut self) {
        self.viewer_count -= 1;
        if self.kind == ContainerBlockEntityKind::ShulkerBox && self.viewer_count <= 0 {
            self.shulker_status = ShulkerBoxAnimationStatus::Closing;
        }
        if self.kind != ContainerBlockEntityKind::ShulkerBox {
            self.viewer_count = self.viewer_count.max(0);
        }
    }

    pub fn tick_lid(&mut self) {
        match self.kind {
            ContainerBlockEntityKind::Chest | ContainerBlockEntityKind::TrappedChest => {
                if self.viewer_count > 0 {
                    self.lid_progress = (self.lid_progress + Self::CHEST_LID_STEP).min(1.0);
                } else {
                    self.lid_progress = (self.lid_progress - Self::CHEST_LID_STEP).max(0.0);
                }
            }
            ContainerBlockEntityKind::ShulkerBox => self.tick_shulker_animation(),
            _ => {}
        }
    }

    pub fn trapped_chest_signal(&self) -> u8 {
        if self.kind == ContainerBlockEntityKind::TrappedChest {
            self.viewer_count.clamp(0, i32::from(MAX_SIGNAL)) as u8
        } else {
            0
        }
    }

    pub fn barrel_is_open(&self) -> bool {
        self.kind == ContainerBlockEntityKind::Barrel && self.viewer_count > 0
    }

    pub fn merged_chest_access_size(&self, neighbour_is_same_chest_type: bool) -> usize {
        if matches!(
            self.kind,
            ContainerBlockEntityKind::Chest | ContainerBlockEntityKind::TrappedChest
        ) && neighbour_is_same_chest_type
        {
            self.kind.size() * 2
        } else {
            self.kind.size()
        }
    }

    pub fn shulker_is_closed(&self) -> bool {
        self.kind == ContainerBlockEntityKind::ShulkerBox
            && self.shulker_status == ShulkerBoxAnimationStatus::Closed
    }

    pub fn can_place_through_face(
        &self,
        _slot: usize,
        item_id: &str,
        _direction: Direction,
    ) -> bool {
        self.kind != ContainerBlockEntityKind::ShulkerBox || !item_id.ends_with("shulker_box")
    }

    pub fn random_non_empty_slot(&self, random_rolls: &[usize]) -> Option<usize> {
        let mut replace_slot = None;
        let mut replace_odds = 1;
        let mut roll_index = 0;
        for (slot, stack) in self.items.iter().enumerate() {
            if stack.is_some() {
                let roll = random_rolls.get(roll_index).copied().unwrap_or(0) % replace_odds;
                roll_index += 1;
                if roll == 0 {
                    replace_slot = Some(slot);
                }
                replace_odds += 1;
            }
        }
        replace_slot
    }

    pub fn activate_once(&self, random_rolls: &[usize]) -> ContainerActivation {
        let Some(slot) = self.random_non_empty_slot(random_rolls) else {
            return ContainerActivation::None;
        };
        match self.kind {
            ContainerBlockEntityKind::Dispenser => ContainerActivation::Dispense { slot },
            ContainerBlockEntityKind::Dropper => ContainerActivation::Drop { slot },
            _ => ContainerActivation::None,
        }
    }

    pub fn hopper_tick(
        &mut self,
        enabled: bool,
        attached_has_space: bool,
        source_has_item: bool,
    ) -> ContainerActivation {
        if self.kind != ContainerBlockEntityKind::Hopper {
            return ContainerActivation::None;
        }
        self.transfer_cooldown -= 1;
        if self.transfer_cooldown > 0 || !enabled {
            return ContainerActivation::None;
        }
        self.transfer_cooldown = 0;

        if attached_has_space {
            if let Some(slot) = self.items.iter().position(Option::is_some) {
                self.transfer_cooldown = Self::HOPPER_MOVE_ITEM_SPEED;
                return ContainerActivation::Push { from_slot: slot };
            }
        }
        if source_has_item && self.items.iter().any(Option::is_none) {
            let slot = self.items.iter().position(Option::is_none).unwrap_or(0);
            self.transfer_cooldown = Self::HOPPER_MOVE_ITEM_SPEED;
            return ContainerActivation::Pull { to_slot: slot };
        }
        ContainerActivation::None
    }

    pub fn hopper_slots_for_face(&self, _direction: Direction) -> Vec<usize> {
        if self.kind == ContainerBlockEntityKind::Hopper {
            (0..self.items.len()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn hopper_can_place_item(
        &self,
        slot: usize,
        item: &PotItemStack,
        direction: Direction,
    ) -> bool {
        self.hopper_slots_for_face(direction).contains(&slot) && !item.is_empty()
    }

    pub fn hopper_can_take_item(&self, slot: usize, direction: Direction) -> bool {
        self.hopper_slots_for_face(direction).contains(&slot) && self.items[slot].is_some()
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(&self.items)
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(name.clone())));
        }
        if let Some(lock) = &self.lock_key {
            fields.push(("lock".to_string(), Tag::String(lock.clone())));
        }
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
        } else {
            fields.push(("Items".to_string(), container_items_tag(&self.items)));
        }
        if self.kind == ContainerBlockEntityKind::Hopper {
            fields.push((
                "TransferCooldown".to_string(),
                Tag::Int(self.transfer_cooldown),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(kind: ContainerBlockEntityKind, tag: &Tag) -> Self {
        let mut container = Self::new(kind);
        let Some(entries) = compound_entries(tag) else {
            return container;
        };
        container.custom_name = get_string(entries, "CustomName").map(str::to_string);
        container.lock_key = get_string(entries, "lock").map(str::to_string);
        container.loot_table = get_string(entries, "LootTable").map(str::to_string);
        container.loot_table_seed = get_long(entries, "LootTableSeed").unwrap_or(0);
        if container.loot_table.is_none() {
            load_container_items(entries, &mut container.items);
        }
        if kind == ContainerBlockEntityKind::Hopper {
            container.transfer_cooldown =
                get_int(entries, "TransferCooldown").unwrap_or(Self::HOPPER_NO_COOLDOWN);
        }
        container
    }

    fn tick_shulker_animation(&mut self) {
        match self.shulker_status {
            ShulkerBoxAnimationStatus::Closed => self.lid_progress = 0.0,
            ShulkerBoxAnimationStatus::Opening => {
                self.lid_progress += 0.1;
                if self.lid_progress >= 1.0 {
                    self.lid_progress = 1.0;
                    self.shulker_status = ShulkerBoxAnimationStatus::Opened;
                }
            }
            ShulkerBoxAnimationStatus::Opened => self.lid_progress = 1.0,
            ShulkerBoxAnimationStatus::Closing => {
                self.lid_progress -= 0.1;
                if self.lid_progress <= 0.0 {
                    self.lid_progress = 0.0;
                    self.shulker_status = ShulkerBoxAnimationStatus::Closed;
                }
            }
        }
    }
}

impl DecoratedPotWobbleStyle {
    pub fn id(self) -> i32 {
        match self {
            Self::Positive => 0,
            Self::Negative => 1,
        }
    }

    pub fn duration(self) -> i32 {
        match self {
            Self::Positive => 7,
            Self::Negative => 10,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Positive),
            1 => Some(Self::Negative),
            _ => None,
        }
    }
}

impl Default for DecoratedPotBlockEntity {
    fn default() -> Self {
        Self {
            decorations: PotDecorations::default(),
            item: None,
            loot_table: None,
            loot_table_seed: 0,
            wobble_started_at_tick: 0,
            last_wobble_style: None,
        }
    }
}

impl DecoratedPotBlockEntity {
    pub const EVENT_POT_WOBBLES: i32 = 1;

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if !self.decorations.is_empty() {
            fields.push(("sherds".to_string(), self.decorations.to_tag()));
        }
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            if !item.is_empty() {
                fields.push(("item".to_string(), item.to_tag()));
            }
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::default();
        };
        let decorations = entries
            .iter()
            .find(|(name, _)| name == "sherds")
            .map(|(_, tag)| PotDecorations::from_tag(tag))
            .unwrap_or_default();
        let loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        let loot_table_seed = entries
            .iter()
            .find_map(|(name, tag)| match tag {
                Tag::Long(seed) if name == "LootTableSeed" => Some(*seed),
                _ => None,
            })
            .unwrap_or(0);
        let item = if loot_table.is_some() {
            None
        } else {
            entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag))
        };
        Self {
            decorations,
            item,
            loot_table,
            loot_table_seed,
            wobble_started_at_tick: 0,
            last_wobble_style: None,
        }
    }

    pub fn trigger_event(&mut self, event: i32, data: i32, game_time: i64) -> bool {
        let Some(style) = DecoratedPotWobbleStyle::from_id(data) else {
            return false;
        };
        if event != Self::EVENT_POT_WOBBLES {
            return false;
        }
        self.wobble_started_at_tick = game_time;
        self.last_wobble_style = Some(style);
        true
    }

    pub fn destruction_drops(&self) -> DecoratedPotDrops {
        DecoratedPotDrops {
            decoration_items: self.decorations.ordered(),
            stored_item: self.item.clone().filter(|item| !item.is_empty()),
        }
    }

    pub fn comparator_output(&self) -> u8 {
        inventory_comparator_output(std::slice::from_ref(&self.item))
    }
}

impl CopperWeatherState {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }
}

impl CopperGolemStatuePose {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::Sitting => "sitting",
            Self::Running => "running",
            Self::Star => "star",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Standing => Self::Sitting,
            Self::Sitting => Self::Running,
            Self::Running => Self::Star,
            Self::Star => Self::Standing,
        }
    }

    pub fn comparator_output(self) -> u8 {
        match self {
            Self::Standing => 1,
            Self::Sitting => 2,
            Self::Running => 3,
            Self::Star => 4,
        }
    }
}

impl CopperGolemStatueBlockEntity {
    pub fn from_block_state(block_state: &str, pose: CopperGolemStatuePose) -> Option<Self> {
        let id = block_state.strip_prefix("minecraft:")?;
        let (waxed, id) = id
            .strip_prefix("waxed_")
            .map(|id| (true, id))
            .unwrap_or((false, id));
        let weather_state = match id {
            "copper_golem_statue" => CopperWeatherState::Unaffected,
            "exposed_copper_golem_statue" => CopperWeatherState::Exposed,
            "weathered_copper_golem_statue" => CopperWeatherState::Weathered,
            "oxidized_copper_golem_statue" => CopperWeatherState::Oxidized,
            _ => return None,
        };
        Some(Self {
            weather_state,
            waxed,
            pose,
            custom_name: None,
        })
    }

    pub fn update_pose(&mut self) {
        self.pose = self.pose.next();
    }

    pub fn comparator_output(&self) -> u8 {
        self.pose.comparator_output()
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(custom_name) = &self.custom_name {
            fields.push(("CustomName".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn clone_item_components(&self) -> Tag {
        let mut fields = vec![(
            "minecraft:block_state".to_string(),
            Tag::Compound(vec![(
                "copper_golem_pose".to_string(),
                Tag::String(self.pose.serialized_name().to_string()),
            )]),
        )];
        if let Some(custom_name) = &self.custom_name {
            fields.push((
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            ));
        }
        Tag::Compound(fields)
    }
}

impl SkullBlockEntity {
    pub fn new() -> Self {
        Self {
            profile: None,
            note_block_sound: None,
            custom_name: None,
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(profile) = &self.profile {
            fields.push(("profile".to_string(), profile.clone()));
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            fields.push((
                "note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("custom_name".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Tag::Compound(entries) = tag else {
            return Self::new();
        };
        Self {
            profile: entries
                .iter()
                .find(|(name, _)| name == "profile")
                .map(|(_, tag)| tag.clone()),
            note_block_sound: get_string(entries, "note_block_sound").map(ToString::to_string),
            custom_name: get_string(entries, "custom_name").map(ToString::to_string),
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn apply_implicit_components(&mut self, components: &BTreeMap<String, Tag>) {
        self.profile = components.get("minecraft:profile").cloned();
        self.note_block_sound = components
            .get("minecraft:note_block_sound")
            .and_then(|tag| match tag {
                Tag::String(id) => Some(id.clone()),
                _ => None,
            });
        self.custom_name = components
            .get("minecraft:custom_name")
            .and_then(|tag| match tag {
                Tag::String(name) => Some(name.clone()),
                _ => None,
            });
    }

    pub fn collect_implicit_components(&self) -> BTreeMap<String, Tag> {
        let mut components = BTreeMap::new();
        if let Some(profile) = &self.profile {
            components.insert("minecraft:profile".to_string(), profile.clone());
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            components.insert(
                "minecraft:note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            );
        }
        if let Some(custom_name) = &self.custom_name {
            components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            );
        }
        components
    }

    pub fn remove_components_from_tag(tag: &mut Tag) {
        if let Tag::Compound(entries) = tag {
            entries.retain(|(name, _)| {
                name != "profile" && name != "note_block_sound" && name != "custom_name"
            });
        }
    }

    pub fn animation_tick(&mut self, powered: bool) {
        if powered {
            self.is_animating = true;
            self.animation_tick_count += 1;
        } else {
            self.is_animating = false;
        }
    }

    pub fn animation(&self, partial_tick: f32) -> f32 {
        if self.is_animating {
            self.animation_tick_count as f32 + partial_tick
        } else {
            self.animation_tick_count as f32
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl ConduitBlockEntity {
    pub const BLOCK_REFRESH_RATE: i64 = 40;
    pub const EFFECT_DURATION_TICKS: i32 = 260;
    pub const MIN_ACTIVE_SIZE: usize = 16;
    pub const MIN_KILL_SIZE: usize = 42;
    pub const KILL_RANGE: f64 = 8.0;
    pub const ROTATION_SPEED: f32 = -0.0375;

    pub fn new() -> Self {
        Self {
            tick_count: 0,
            active_rotation: 0,
            is_active: false,
            is_hunting: false,
            effect_blocks: Vec::new(),
            destroy_target: None,
            next_ambient_sound_activation: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(target) = &self.destroy_target {
            fields.push(("Target".to_string(), Tag::String(target.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut conduit = Self::new();
        if let Some(entries) = compound_entries(tag) {
            conduit.destroy_target = get_string(entries, "Target").map(ToString::to_string);
        }
        conduit
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn active_rotation(&self, partial_tick: f32) -> f32 {
        (self.active_rotation as f32 + partial_tick) * Self::ROTATION_SPEED
    }

    pub fn effect_range(effect_block_count: usize) -> i32 {
        (effect_block_count / 7) as i32 * 16
    }

    pub fn apply_effects(&self) -> Option<ConduitEffectApplication> {
        self.is_active.then(|| ConduitEffectApplication {
            range: Self::effect_range(self.effect_blocks.len()),
            duration_ticks: Self::EFFECT_DURATION_TICKS,
        })
    }

    pub fn server_tick(
        &mut self,
        game_time: i64,
        pos: BlockPos,
        is_water_at: impl Fn(BlockPos) -> bool,
        block_at: impl Fn(BlockPos) -> &'static str,
        targets: &[ConduitTarget],
    ) -> Option<String> {
        self.tick_count += 1;
        if game_time % Self::BLOCK_REFRESH_RATE != 0 {
            if self.is_active {
                self.active_rotation += 1;
            }
            return None;
        }

        self.is_active = self.update_shape(pos, is_water_at, block_at);
        self.is_hunting = self.effect_blocks.len() >= Self::MIN_KILL_SIZE;
        if !self.is_active {
            self.destroy_target = None;
            return None;
        }
        let target_changed = self.update_destroy_target(pos, targets);
        if self.is_active {
            self.active_rotation += 1;
        }
        if self.destroy_target.is_some() && self.is_hunting && target_changed {
            self.destroy_target.clone()
        } else {
            None
        }
    }

    pub fn update_shape(
        &mut self,
        pos: BlockPos,
        is_water_at: impl Fn(BlockPos) -> bool,
        block_at: impl Fn(BlockPos) -> &'static str,
    ) -> bool {
        self.effect_blocks.clear();
        for ox in -1..=1 {
            for oy in -1..=1 {
                for oz in -1..=1 {
                    if !is_water_at(offset_pos(pos, ox, oy, oz)) {
                        return false;
                    }
                }
            }
        }

        for ox in -2_i32..=2 {
            for oy in -2_i32..=2 {
                for oz in -2_i32..=2 {
                    let ax = ox.abs();
                    let ay = oy.abs();
                    let az = oz.abs();
                    let frame_position = (ax > 1 || ay > 1 || az > 1)
                        && ((ox == 0 && (ay == 2 || az == 2))
                            || (oy == 0 && (ax == 2 || az == 2))
                            || (oz == 0 && (ax == 2 || ay == 2)));
                    if frame_position {
                        let test_pos = offset_pos(pos, ox, oy, oz);
                        if Self::is_valid_frame_block(block_at(test_pos)) {
                            self.effect_blocks.push(test_pos);
                        }
                    }
                }
            }
        }
        self.effect_blocks.len() >= Self::MIN_ACTIVE_SIZE
    }

    pub fn update_destroy_target(&mut self, pos: BlockPos, targets: &[ConduitTarget]) -> bool {
        if !self.is_hunting {
            let changed = self.destroy_target.is_some();
            self.destroy_target = None;
            return changed;
        }
        if let Some(current) = self.destroy_target.as_ref() {
            if targets.iter().any(|target| {
                target.id == *current
                    && target.alive
                    && target.enemy
                    && target.in_water_or_rain
                    && closer_than(pos, target.pos, Self::KILL_RANGE)
            }) {
                return false;
            }
        }
        let next = targets
            .iter()
            .find(|target| {
                target.alive
                    && target.enemy
                    && target.in_water_or_rain
                    && closer_than(pos, target.pos, Self::KILL_RANGE)
            })
            .map(|target| target.id.clone());
        let changed = self.destroy_target != next;
        self.destroy_target = next;
        changed
    }

    pub fn is_valid_frame_block(block: &str) -> bool {
        matches!(
            block,
            "minecraft:prismarine"
                | "minecraft:prismarine_bricks"
                | "minecraft:sea_lantern"
                | "minecraft:dark_prismarine"
        )
    }
}

impl CampfireBlockEntity {
    pub const NUM_SLOTS: usize = 4;
    pub const DEFAULT_COOKING_TIME: i32 = 600;
    pub const BURN_COOL_SPEED: i32 = 2;

    pub fn new(signal_fire: bool) -> Self {
        Self {
            items: vec![None; Self::NUM_SLOTS],
            cooking_progress: [0; Self::NUM_SLOTS],
            cooking_time: [0; Self::NUM_SLOTS],
            signal_fire,
        }
    }

    pub fn place_food(&mut self, item: PotItemStack, cooking_time: Option<i32>) -> bool {
        if item.is_empty() {
            return false;
        }
        let Some(slot) = self.items.iter().position(Option::is_none) else {
            return false;
        };
        self.items[slot] = Some(PotItemStack {
            item_id: item.item_id,
            count: 1,
        });
        self.cooking_progress[slot] = 0;
        self.cooking_time[slot] = cooking_time.unwrap_or(Self::DEFAULT_COOKING_TIME).max(1);
        true
    }

    pub fn cook_tick(
        &mut self,
        lit: bool,
        recipe_result: impl Fn(&PotItemStack) -> PotItemStack,
    ) -> Vec<CampfireTickResult> {
        if !lit {
            return self.cooldown_tick();
        }

        let mut results = Vec::new();
        for slot in 0..Self::NUM_SLOTS {
            if let Some(item) = self.items[slot].as_ref() {
                self.cooking_progress[slot] += 1;
                if self.cooking_progress[slot] >= self.cooking_time[slot] {
                    let cooked = recipe_result(item);
                    self.items[slot] = None;
                    self.cooking_progress[slot] = 0;
                    self.cooking_time[slot] = 0;
                    results.push(CampfireTickResult::Cooked { slot, item: cooked });
                } else {
                    results.push(CampfireTickResult::Changed);
                }
            }
        }
        if results.is_empty() {
            results.push(CampfireTickResult::NoChange);
        }
        results
    }

    pub fn cooldown_tick(&mut self) -> Vec<CampfireTickResult> {
        let mut changed = false;
        for slot in 0..Self::NUM_SLOTS {
            if self.cooking_progress[slot] > 0 {
                changed = true;
                self.cooking_progress[slot] = (self.cooking_progress[slot] - Self::BURN_COOL_SPEED)
                    .clamp(0, self.cooking_time[slot]);
            }
        }
        vec![if changed {
            CampfireTickResult::Changed
        } else {
            CampfireTickResult::NoChange
        }]
    }

    pub fn clear_content(&mut self) {
        self.items.fill(None);
    }

    pub fn get_update_tag(&self) -> Tag {
        Tag::Compound(vec![("Items".to_string(), self.items_tag())])
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("Items".to_string(), self.items_tag()),
            (
                "CookingTimes".to_string(),
                Tag::IntArray(self.cooking_progress.to_vec()),
            ),
            (
                "CookingTotalTimes".to_string(),
                Tag::IntArray(self.cooking_time.to_vec()),
            ),
            (
                "SignalFire".to_string(),
                Tag::Byte(i8::from(self.signal_fire)),
            ),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(false);
        };
        let mut campfire = Self::new(get_bool(entries, "SignalFire").unwrap_or(false));
        if let Some(Tag::List(items)) = entries
            .iter()
            .find_map(|(name, tag)| (name == "Items").then_some(tag))
        {
            for item_tag in items {
                let Some(item_entries) = compound_entries(item_tag) else {
                    continue;
                };
                let Some(slot) =
                    get_byte(item_entries, "Slot").and_then(|slot| usize::try_from(slot).ok())
                else {
                    continue;
                };
                if slot < Self::NUM_SLOTS {
                    campfire.items[slot] = PotItemStack::from_tag(item_tag);
                }
            }
        }
        if let Some(values) = get_int_array(entries, "CookingTimes") {
            for (slot, value) in values.iter().copied().take(Self::NUM_SLOTS).enumerate() {
                campfire.cooking_progress[slot] = value;
            }
        }
        if let Some(values) = get_int_array(entries, "CookingTotalTimes") {
            for (slot, value) in values.iter().copied().take(Self::NUM_SLOTS).enumerate() {
                campfire.cooking_time[slot] = value;
            }
        }
        campfire
    }

    fn items_tag(&self) -> Tag {
        Tag::List(
            self.items
                .iter()
                .enumerate()
                .filter_map(|(slot, item)| {
                    let mut tag = item.as_ref()?.to_tag();
                    if let Tag::Compound(entries) = &mut tag {
                        entries.insert(0, ("Slot".to_string(), Tag::Byte(slot as i8)));
                    }
                    Some(tag)
                })
                .collect(),
        )
    }
}

impl SculkSensorPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listening => "listening",
            Self::Ticking => "ticking",
            Self::VibrationDone => "vibration_done",
            Self::Cooldown => "cooldown",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "listening" => Some(Self::Listening),
            "ticking" => Some(Self::Ticking),
            "vibration_done" => Some(Self::VibrationDone),
            "cooldown" => Some(Self::Cooldown),
            _ => None,
        }
    }
}

impl SculkSensorBlockEntity {
    pub const DEFAULT_LAST_VIBRATION_FREQUENCY: u8 = 0;
    pub const LISTENER_RADIUS: i32 = 8;
    pub const ACTIVE_TICKS: i32 = 30;
    pub const COOLDOWN_TICKS: i32 = 10;

    pub fn new() -> Self {
        Self {
            vibration_data: VibrationData::new(),
            last_vibration_frequency: Self::DEFAULT_LAST_VIBRATION_FREQUENCY,
            phase: SculkSensorPhase::Listening,
            listener_radius: Self::LISTENER_RADIUS,
            power: 0,
            active_ticks: 0,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "last_vibration_frequency".to_string(),
                Tag::Int(i32::from(self.last_vibration_frequency)),
            ),
            ("listener".to_string(), self.listener_tag()),
            (
                "phase".to_string(),
                Tag::String(self.phase.as_str().to_string()),
            ),
            ("power".to_string(), Tag::Byte(self.power as i8)),
            ("active_ticks".to_string(), Tag::Int(self.active_ticks)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let mut sensor = Self::new();
        sensor.last_vibration_frequency = get_int(entries, "last_vibration_frequency")
            .unwrap_or(0)
            .clamp(0, 15) as u8;
        sensor.phase = get_string(entries, "phase")
            .and_then(SculkSensorPhase::from_str)
            .unwrap_or(SculkSensorPhase::Listening);
        sensor.power = get_byte(entries, "power").unwrap_or(0).clamp(0, 15) as u8;
        sensor.active_ticks = get_int(entries, "active_ticks").unwrap_or(0).max(0);
        if let Some(listener_tag) = entries.iter().find(|(name, _)| name == "listener") {
            sensor.vibration_data = vibration_data_from_tag(&listener_tag.1);
        }
        sensor
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn can_receive_vibration(&self, event_id: &str, sensor_can_activate: bool) -> bool {
        sensor_can_activate
            && self.phase == SculkSensorPhase::Listening
            && vibration_frequency(event_id) != NO_VIBRATION_FREQUENCY
    }

    pub fn queue_vibration(&mut self, vibration: VibrationInfo, game_time: i64) -> bool {
        if !self.can_receive_vibration(vibration.event.id, true) {
            return false;
        }
        self.vibration_data
            .selector
            .add_candidate(vibration, game_time);
        true
    }

    pub fn receive_vibration(
        &mut self,
        event_id: &str,
        distance: f32,
    ) -> Option<SculkSensorTickResult> {
        if !self.can_receive_vibration(event_id, true) {
            return None;
        }
        let frequency = vibration_frequency(event_id);
        let redstone = redstone_strength_for_distance(distance, self.listener_radius);
        self.last_vibration_frequency = frequency;
        self.power = redstone;
        self.phase = SculkSensorPhase::VibrationDone;
        self.active_ticks = Self::ACTIVE_TICKS;
        Some(SculkSensorTickResult::Activate {
            frequency,
            redstone,
        })
    }

    pub fn tick(&mut self, game_time: i64) -> SculkSensorTickResult {
        match self.phase {
            SculkSensorPhase::Listening | SculkSensorPhase::Ticking => {
                match tick_vibration(&mut self.vibration_data, game_time) {
                    VibrationTickAction::Selected {
                        travel_time_in_ticks,
                        ..
                    } => {
                        self.phase = SculkSensorPhase::Ticking;
                        SculkSensorTickResult::Particle {
                            travel_time_in_ticks,
                        }
                    }
                    VibrationTickAction::ReloadParticle {
                        travel_time_in_ticks,
                    } => SculkSensorTickResult::Particle {
                        travel_time_in_ticks,
                    },
                    VibrationTickAction::Received {
                        event_id,
                        frequency,
                    } => {
                        let distance = 0.0;
                        let redstone =
                            redstone_strength_for_distance(distance, self.listener_radius);
                        self.last_vibration_frequency = frequency;
                        self.power = redstone;
                        self.phase = SculkSensorPhase::VibrationDone;
                        self.active_ticks = Self::ACTIVE_TICKS;
                        let _ = event_id;
                        SculkSensorTickResult::Activate {
                            frequency,
                            redstone,
                        }
                    }
                    VibrationTickAction::None => SculkSensorTickResult::None,
                }
            }
            SculkSensorPhase::VibrationDone => {
                self.active_ticks = self.active_ticks.saturating_sub(1);
                if self.active_ticks == 0 {
                    self.phase = SculkSensorPhase::Cooldown;
                    self.active_ticks = Self::COOLDOWN_TICKS;
                    SculkSensorTickResult::Cooldown
                } else {
                    SculkSensorTickResult::None
                }
            }
            SculkSensorPhase::Cooldown => {
                self.active_ticks = self.active_ticks.saturating_sub(1);
                if self.active_ticks == 0 {
                    self.phase = SculkSensorPhase::Listening;
                    self.power = 0;
                    SculkSensorTickResult::Deactivate
                } else {
                    SculkSensorTickResult::None
                }
            }
        }
    }

    fn listener_tag(&self) -> Tag {
        let mut fields = vec![
            (
                "travel_time_in_ticks".to_string(),
                Tag::Int(self.vibration_data.travel_time_in_ticks),
            ),
            (
                "reload_vibration_particle".to_string(),
                Tag::Byte(i8::from(self.vibration_data.reload_vibration_particle)),
            ),
        ];
        if let Some(vibration) = &self.vibration_data.current_vibration {
            fields.push((
                "event".to_string(),
                Tag::String(vibration.event.id.to_string()),
            ));
            fields.push(("distance".to_string(), Tag::Float(vibration.distance)));
        }
        Tag::Compound(fields)
    }
}

impl CalibratedSculkSensorBlockEntity {
    pub const LISTENER_RADIUS: i32 = 16;

    pub fn new(back_signal: u8) -> Self {
        let mut sensor = SculkSensorBlockEntity::new();
        sensor.listener_radius = Self::LISTENER_RADIUS;
        Self {
            sensor,
            back_signal: back_signal.min(15),
        }
    }

    pub fn set_back_signal(&mut self, back_signal: u8) {
        self.back_signal = back_signal.min(15);
    }

    pub fn can_receive_vibration(&self, event_id: &str, sensor_can_activate: bool) -> bool {
        if !self
            .sensor
            .can_receive_vibration(event_id, sensor_can_activate)
        {
            return false;
        }
        let frequency = vibration_frequency(event_id);
        self.back_signal == 0 || self.back_signal == frequency
    }

    pub fn receive_vibration(
        &mut self,
        event_id: &str,
        distance: f32,
    ) -> Option<SculkSensorTickResult> {
        match calibrated_sculk_sensor_receive(
            self.back_signal,
            event_id,
            distance,
            Self::LISTENER_RADIUS,
        ) {
            SculkSensorAction::Activate {
                frequency,
                redstone,
            } if self.sensor.can_receive_vibration(event_id, true) => {
                self.sensor.last_vibration_frequency = frequency;
                self.sensor.power = redstone;
                self.sensor.phase = SculkSensorPhase::VibrationDone;
                self.sensor.active_ticks = SculkSensorBlockEntity::ACTIVE_TICKS;
                Some(SculkSensorTickResult::Activate {
                    frequency,
                    redstone,
                })
            }
            _ => None,
        }
    }

    pub fn tick(&mut self, game_time: i64) -> SculkSensorTickResult {
        self.sensor.tick(game_time)
    }

    pub fn save_additional(&self) -> Tag {
        let mut entries = match self.sensor.save_additional() {
            Tag::Compound(entries) => entries,
            _ => Vec::new(),
        };
        entries.push(("back_signal".to_string(), Tag::Byte(self.back_signal as i8)));
        Tag::Compound(entries)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let sensor = SculkSensorBlockEntity::load_additional(tag);
        let back_signal = compound_entries(tag)
            .and_then(|entries| get_byte(entries, "back_signal"))
            .unwrap_or(0)
            .clamp(0, 15) as u8;
        Self {
            sensor: SculkSensorBlockEntity {
                listener_radius: Self::LISTENER_RADIUS,
                ..sensor
            },
            back_signal,
        }
    }
}

impl SculkChargeCursor {
    pub const MAX_CHARGE: i32 = 1000;

    pub fn new(pos: BlockPos, charge: i32) -> Self {
        Self {
            pos,
            charge: charge.clamp(0, Self::MAX_CHARGE),
            decay_delay: 1,
            update_delay: 0,
            facings: Vec::new(),
        }
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("pos".to_string(), block_pos_to_tag(self.pos)),
            ("charge".to_string(), Tag::Int(self.charge)),
            ("decay_delay".to_string(), Tag::Int(self.decay_delay)),
            ("update_delay".to_string(), Tag::Int(self.update_delay)),
            (
                "facings".to_string(),
                Tag::List(
                    self.facings
                        .iter()
                        .map(|direction| Tag::String(direction_name(*direction).to_string()))
                        .collect(),
                ),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            pos: entries
                .iter()
                .find(|(name, _)| name == "pos")
                .and_then(|(_, tag)| block_pos_from_tag(tag))?,
            charge: get_int(entries, "charge")
                .unwrap_or(0)
                .clamp(0, Self::MAX_CHARGE),
            decay_delay: get_int(entries, "decay_delay").unwrap_or(1).clamp(0, 1),
            update_delay: get_int(entries, "update_delay").unwrap_or(0).max(0),
            facings: entries
                .iter()
                .find(|(name, _)| name == "facings")
                .and_then(|(_, tag)| match tag {
                    Tag::List(values) => Some(
                        values
                            .iter()
                            .filter_map(|tag| match tag {
                                Tag::String(name) => direction_from_name(name),
                                _ => None,
                            })
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default(),
        })
    }
}

impl SculkCatalystBlockEntity {
    pub const LISTENER_RADIUS: i32 = 8;
    pub const PULSE_TICKS: i32 = 8;
    pub const MAX_CURSORS: usize = 32;
    pub const MAX_CHARGE: i32 = 1000;
    pub const MAX_CURSOR_DISTANCE: i32 = 1024;

    pub fn new() -> Self {
        Self {
            cursors: Vec::new(),
            pulse_ticks: 0,
        }
    }

    pub fn add_cursors(&mut self, start_pos: BlockPos, mut charge: i32) {
        while charge > 0 && self.cursors.len() < Self::MAX_CURSORS {
            let current_charge = charge.min(Self::MAX_CHARGE);
            self.cursors
                .push(SculkChargeCursor::new(start_pos, current_charge));
            charge -= current_charge;
        }
    }

    pub fn handle_entity_die(
        &mut self,
        source_pos: BlockPos,
        experience_reward: i32,
        should_drop_experience: bool,
        experience_already_consumed: bool,
    ) -> SculkCatalystEventResult {
        if experience_already_consumed {
            return SculkCatalystEventResult::Ignored;
        }
        if should_drop_experience && experience_reward > 0 {
            self.add_cursors(offset_pos(source_pos, 0, 1, 0), experience_reward);
        }
        self.pulse_ticks = Self::PULSE_TICKS;
        SculkCatalystEventResult::Bloom {
            pulse_ticks: Self::PULSE_TICKS,
        }
    }

    pub fn tick(&mut self, origin: BlockPos) {
        self.pulse_ticks = self.pulse_ticks.saturating_sub(1);
        self.cursors.retain_mut(|cursor| {
            if chessboard_distance(cursor.pos, origin) > Self::MAX_CURSOR_DISTANCE {
                return false;
            }
            if cursor.update_delay > 0 {
                cursor.update_delay -= 1;
                return true;
            }
            if cursor.decay_delay > 0 {
                cursor.decay_delay -= 1;
            } else {
                cursor.charge = (cursor.charge - 1).max(0);
                cursor.decay_delay = 1;
            }
            cursor.charge > 0
        });
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "cursors".to_string(),
                Tag::List(self.cursors.iter().map(SculkChargeCursor::to_tag).collect()),
            ),
            ("pulse_ticks".to_string(), Tag::Int(self.pulse_ticks)),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let cursors = entries
            .iter()
            .find(|(name, _)| name == "cursors")
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(SculkChargeCursor::from_tag)
                        .take(Self::MAX_CURSORS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self {
            cursors,
            pulse_ticks: get_int(entries, "pulse_ticks").unwrap_or(0).max(0),
        }
    }
}

impl BeehiveOccupant {
    pub const DEFAULT_ENTITY_TYPE: &'static str = "minecraft:bee";

    pub fn bee(ticks_in_hive: i32, has_nectar: bool) -> Self {
        let mut entity_fields = Vec::new();
        if has_nectar {
            entity_fields.push(("HasNectar".to_string(), Tag::Byte(1)));
        }
        Self {
            entity_type: Self::DEFAULT_ENTITY_TYPE.to_string(),
            entity_data: Tag::Compound(entity_fields),
            ticks_in_hive: ticks_in_hive.max(0),
            min_ticks_in_hive: if has_nectar {
                BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTAR
            } else {
                BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS
            },
        }
    }

    pub fn has_nectar(&self) -> bool {
        compound_entries(&self.entity_data)
            .and_then(|entries| get_bool(entries, "HasNectar"))
            .unwrap_or(false)
    }

    fn tick_ready(&mut self) -> bool {
        let was_ready = self.ticks_in_hive > self.min_ticks_in_hive;
        self.ticks_in_hive += 1;
        was_ready
    }

    fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "entity_type".to_string(),
                Tag::String(self.entity_type.clone()),
            ),
            ("entity_data".to_string(), self.entity_data.clone()),
            ("ticks_in_hive".to_string(), Tag::Int(self.ticks_in_hive)),
            (
                "min_ticks_in_hive".to_string(),
                Tag::Int(self.min_ticks_in_hive),
            ),
        ])
    }

    fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        Some(Self {
            entity_type: get_string(entries, "entity_type")
                .unwrap_or(Self::DEFAULT_ENTITY_TYPE)
                .to_string(),
            entity_data: entries
                .iter()
                .find(|(name, _)| name == "entity_data")
                .map(|(_, tag)| tag.clone())
                .unwrap_or_else(|| Tag::Compound(Vec::new())),
            ticks_in_hive: get_int(entries, "ticks_in_hive").unwrap_or(0).max(0),
            min_ticks_in_hive: get_int(entries, "min_ticks_in_hive")
                .unwrap_or(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS)
                .max(0),
        })
    }
}

impl BeehiveBlockEntity {
    pub const MAX_OCCUPANTS: usize = 3;
    pub const MIN_TICKS_BEFORE_REENTERING_HIVE: i32 = 400;
    pub const MIN_OCCUPATION_TICKS_NECTAR: i32 = 2400;
    pub const MIN_OCCUPATION_TICKS_NECTARLESS: i32 = 600;
    pub const MAX_HONEY_LEVEL: i32 = 5;
    pub const PLAYER_ANGER_RADIUS_SQUARED: f64 = 16.0;
    pub const WORK_SOUND_CHANCE: f64 = 0.005;

    pub fn new() -> Self {
        Self {
            occupants: Vec::new(),
            saved_flower_pos: None,
            honey_level: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.occupants.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.occupants.len() == Self::MAX_OCCUPANTS
    }

    pub fn occupant_count(&self) -> usize {
        self.occupants.len()
    }

    pub fn add_occupant(
        &mut self,
        occupant: BeehiveOccupant,
        saved_flower_pos: Option<BlockPos>,
    ) -> bool {
        if self.is_full() {
            return false;
        }
        if self.saved_flower_pos.is_none() {
            self.saved_flower_pos = saved_flower_pos;
        }
        self.occupants.push(occupant);
        true
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![
            (
                "bees".to_string(),
                Tag::List(self.occupants.iter().map(BeehiveOccupant::to_tag).collect()),
            ),
            ("honey_level".to_string(), Tag::Int(self.honey_level)),
        ];
        if let Some(pos) = self.saved_flower_pos {
            fields.push(("flower_pos".to_string(), block_pos_to_tag(pos)));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new();
        };
        let occupants = entries
            .iter()
            .find(|(name, _)| name == "bees")
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(BeehiveOccupant::from_tag)
                        .take(Self::MAX_OCCUPANTS)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self {
            occupants,
            saved_flower_pos: entries
                .iter()
                .find(|(name, _)| name == "flower_pos")
                .and_then(|(_, tag)| block_pos_from_tag(tag)),
            honey_level: get_int(entries, "honey_level")
                .unwrap_or(0)
                .clamp(0, Self::MAX_HONEY_LEVEL),
        }
    }

    pub fn tick(
        &mut self,
        bees_stay_in_hive: bool,
        front_blocked: bool,
        honey_bonus_roll: bool,
    ) -> Vec<BeeReleaseEvent> {
        let mut released = Vec::new();
        let mut index = 0;
        while index < self.occupants.len() {
            if self.occupants[index].tick_ready() {
                let status = if self.occupants[index].has_nectar() {
                    BeeReleaseStatus::HoneyDelivered
                } else {
                    BeeReleaseStatus::BeeReleased
                };
                if let Some(event) = self.release_at(
                    index,
                    status,
                    bees_stay_in_hive,
                    front_blocked,
                    honey_bonus_roll,
                ) {
                    released.push(event);
                    continue;
                }
            }
            index += 1;
        }
        released
    }

    pub fn empty_all_living_from_hive(
        &mut self,
        status: BeeReleaseStatus,
        is_sedated: bool,
    ) -> Vec<BeeReleaseEvent> {
        let mut released = Vec::new();
        while !self.occupants.is_empty() {
            if let Some(mut event) = self.release_at(0, status, false, false, false) {
                event.stay_out_of_hive_ticks = if is_sedated {
                    Self::MIN_TICKS_BEFORE_REENTERING_HIVE
                } else {
                    0
                };
                released.push(event);
            } else {
                break;
            }
        }
        released
    }

    pub fn on_fire_nearby(&mut self) -> Vec<BeeReleaseEvent> {
        self.empty_all_living_from_hive(BeeReleaseStatus::Emergency, false)
    }

    fn release_at(
        &mut self,
        index: usize,
        status: BeeReleaseStatus,
        bees_stay_in_hive: bool,
        front_blocked: bool,
        honey_bonus_roll: bool,
    ) -> Option<BeeReleaseEvent> {
        if status != BeeReleaseStatus::Emergency && (bees_stay_in_hive || front_blocked) {
            return None;
        }
        let occupant = self.occupants.remove(index);
        if status == BeeReleaseStatus::HoneyDelivered && self.honey_level < Self::MAX_HONEY_LEVEL {
            let level_increase = if honey_bonus_roll { 2 } else { 1 };
            self.honey_level = (self.honey_level + level_increase).min(Self::MAX_HONEY_LEVEL);
        }
        Some(BeeReleaseEvent {
            entity_type: occupant.entity_type,
            status,
            honey_level: self.honey_level,
            stay_out_of_hive_ticks: 0,
        })
    }
}

impl CreakingHeartBlockEntity {
    pub const PLAYER_DETECTION_RANGE: i32 = 32;
    pub const CREAKING_ROAMING_RADIUS: i32 = 32;
    pub const DISTANCE_CREAKING_TOO_FAR: f64 = 34.0;
    pub const SPAWN_RANGE_XZ: i32 = 16;
    pub const SPAWN_RANGE_Y: i32 = 8;
    pub const ATTEMPTS_PER_SPAWN: i32 = 5;
    pub const UPDATE_TICKS: i32 = 20;
    pub const UPDATE_TICKS_VARIANCE: i32 = 5;
    pub const HURT_CALL_TOTAL_TICKS: i32 = 100;
    pub const NUMBER_OF_HURT_CALLS: i32 = 10;
    pub const HURT_CALL_INTERVAL: i32 = 10;
    pub const HURT_CALL_PARTICLE_TICKS: i32 = 50;
    pub const MAX_RESIN_DEPTH: i32 = 2;
    pub const MAX_RESIN_COUNT: i32 = 64;
    pub const TICKS_GRACE_PERIOD: i64 = 30;

    pub fn new() -> Self {
        Self {
            creaking_uuid: None,
            ticks_existed: 0,
            ticker: 0,
            emitter_ticks: 0,
            output_signal: 0,
            state: CreakingHeartStateModel::Uprooted,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(uuid) = &self.creaking_uuid {
            fields.push(("creaking".to_string(), Tag::String(uuid.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut heart = Self::new();
        if let Some(entries) = compound_entries(tag) {
            if let Some(uuid) = get_string(entries, "creaking") {
                heart.set_creaking_uuid(uuid.to_string());
            }
        }
        heart
    }

    pub fn set_creaking_uuid(&mut self, uuid: String) {
        self.creaking_uuid = Some(uuid);
        self.ticks_existed = 0;
    }

    pub fn clear_creaking(&mut self) {
        self.creaking_uuid = None;
    }

    pub fn server_tick(
        &mut self,
        has_required_logs: bool,
        creaking_active: bool,
        spawning_monsters: bool,
        player_nearby: bool,
        protector_resolved: bool,
        protector_distance: Option<f64>,
        protector_persistent: bool,
        player_stuck_in_protector: bool,
        next_ticker_offset: i32,
    ) -> Vec<CreakingHeartAction> {
        self.ticks_existed += 1;
        let mut actions = Vec::new();
        let computed_signal = self.compute_analog_output_signal(protector_distance);
        if self.output_signal != computed_signal {
            self.output_signal = computed_signal;
        }

        if self.emitter_ticks > 0 {
            self.emitter_ticks -= 1;
        }

        self.ticker -= 1;
        if self.ticker >= 0 {
            return actions;
        }
        self.ticker = Self::UPDATE_TICKS
            + next_ticker_offset.clamp(0, Self::UPDATE_TICKS_VARIANCE.saturating_sub(1));

        let updated_state = self.updated_state(has_required_logs, creaking_active);
        if updated_state != self.state {
            self.state = updated_state;
            actions.push(CreakingHeartAction::StateChanged(updated_state));
            if updated_state == CreakingHeartStateModel::Uprooted {
                return actions;
            }
        }

        if self.creaking_uuid.is_none() {
            if self.state == CreakingHeartStateModel::Awake && spawning_monsters && player_nearby {
                actions.push(CreakingHeartAction::SpawnProtector {
                    attempts: Self::ATTEMPTS_PER_SPAWN,
                    range_xz: Self::SPAWN_RANGE_XZ,
                    range_y: Self::SPAWN_RANGE_Y,
                });
            }
        } else if protector_resolved {
            let too_far = protector_distance
                .map(|distance| distance > Self::DISTANCE_CREAKING_TOO_FAR)
                .unwrap_or(false);
            if (!creaking_active && !protector_persistent) || too_far || player_stuck_in_protector {
                self.clear_creaking();
                actions.push(CreakingHeartAction::RemoveProtector);
            }
        } else if self.ticks_existed >= Self::TICKS_GRACE_PERIOD {
            self.clear_creaking();
            actions.push(CreakingHeartAction::RemoveProtector);
        }
        actions
    }

    pub fn on_protector_spawned(&mut self, uuid: String) {
        self.creaking_uuid = Some(uuid);
    }

    pub fn creaking_hurt(&mut self, state_awake: bool, resin_clumps: i32) -> CreakingHeartAction {
        if self.creaking_uuid.is_none() || self.emitter_ticks > 0 {
            return CreakingHeartAction::None;
        }
        self.emitter_ticks = Self::HURT_CALL_TOTAL_TICKS;
        CreakingHeartAction::HurtPulse {
            total_ticks: Self::HURT_CALL_TOTAL_TICKS,
            particle_ticks: Self::HURT_CALL_PARTICLE_TICKS,
            resin_clumps: if state_awake {
                resin_clumps.clamp(2, 3)
            } else {
                0
            },
        }
    }

    pub fn remove_protector(&mut self) -> CreakingHeartAction {
        if self.creaking_uuid.take().is_some() {
            CreakingHeartAction::RemoveProtector
        } else {
            CreakingHeartAction::None
        }
    }

    pub fn compute_analog_output_signal(&self, protector_distance: Option<f64>) -> i32 {
        if self.creaking_uuid.is_none() {
            return 0;
        }
        let Some(distance) = protector_distance else {
            return 0;
        };
        let scaled_distance = distance.clamp(0.0, f64::from(Self::CREAKING_ROAMING_RADIUS))
            / f64::from(Self::CREAKING_ROAMING_RADIUS);
        15 - (scaled_distance * 15.0).floor() as i32
    }

    fn updated_state(
        &self,
        has_required_logs: bool,
        creaking_active: bool,
    ) -> CreakingHeartStateModel {
        if !has_required_logs && self.creaking_uuid.is_none() {
            CreakingHeartStateModel::Uprooted
        } else if creaking_active {
            CreakingHeartStateModel::Awake
        } else {
            CreakingHeartStateModel::Dormant
        }
    }
}

impl SculkShriekerBlockEntity {
    pub const LISTENER_RADIUS: i32 = 8;
    pub const WARNING_SOUND_RADIUS: i32 = 10;
    pub const WARDEN_SPAWN_ATTEMPTS: i32 = 20;
    pub const WARDEN_SPAWN_RANGE_XZ: i32 = 5;
    pub const WARDEN_SPAWN_RANGE_Y: i32 = 6;
    pub const DARKNESS_RADIUS: i32 = 40;
    pub const SHRIEKING_TICKS: i32 = 90;
    pub const WARDEN_SUMMON_WARNING_LEVEL: i32 = 4;

    pub fn new(can_summon: bool) -> Self {
        Self {
            warning_level: 0,
            vibration_data: VibrationData::new(),
            shrieking_ticks: 0,
            can_summon,
        }
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            ("warning_level".to_string(), Tag::Int(self.warning_level)),
            ("listener".to_string(), self.listener_tag()),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Some(entries) = compound_entries(tag) else {
            return Self::new(false);
        };
        Self {
            warning_level: get_int(entries, "warning_level").unwrap_or(0).clamp(0, 4),
            vibration_data: entries
                .iter()
                .find(|(name, _)| name == "listener")
                .map(|(_, tag)| vibration_data_from_tag(tag))
                .unwrap_or_default(),
            shrieking_ticks: 0,
            can_summon: false,
        }
    }

    pub fn can_receive_vibration(
        &self,
        shrieking_block_state: bool,
        has_player_source: bool,
    ) -> bool {
        !shrieking_block_state && has_player_source
    }

    pub fn try_shriek(
        &mut self,
        has_player: bool,
        can_respond: bool,
        tracker_warning_level: Option<i32>,
        warden_spawn_available: bool,
    ) -> SculkShriekResult {
        if !has_player || self.shrieking_ticks > 0 {
            return SculkShriekResult::Ignored;
        }

        self.warning_level = 0;
        if can_respond && tracker_warning_level.is_none() {
            return SculkShriekResult::Ignored;
        }
        if let Some(warning_level) = tracker_warning_level {
            self.warning_level = warning_level.clamp(0, Self::WARDEN_SUMMON_WARNING_LEVEL);
        }
        self.start_shrieking();
        self.try_respond(can_respond, warden_spawn_available)
    }

    pub fn try_respond(
        &self,
        can_respond: bool,
        warden_spawn_available: bool,
    ) -> SculkShriekResult {
        if !can_respond || !self.can_summon || self.warning_level <= 0 {
            return SculkShriekResult::Shriek {
                warning_level: self.warning_level,
            };
        }
        if self.warning_level >= Self::WARDEN_SUMMON_WARNING_LEVEL && warden_spawn_available {
            SculkShriekResult::SummonWarden {
                warning_level: self.warning_level,
                attempts: Self::WARDEN_SPAWN_ATTEMPTS,
                range_xz: Self::WARDEN_SPAWN_RANGE_XZ,
                range_y: Self::WARDEN_SPAWN_RANGE_Y,
                darkness_radius: Self::DARKNESS_RADIUS,
            }
        } else {
            SculkShriekResult::ReplySound {
                warning_level: self.warning_level,
                darkness_radius: Self::DARKNESS_RADIUS,
            }
        }
    }

    pub fn tick(&mut self) -> SculkShriekResult {
        if self.shrieking_ticks > 0 {
            self.shrieking_ticks -= 1;
        }
        SculkShriekResult::Ignored
    }

    fn start_shrieking(&mut self) {
        self.shrieking_ticks = Self::SHRIEKING_TICKS;
    }

    fn listener_tag(&self) -> Tag {
        let mut fields = vec![(
            "travel_time_in_ticks".to_string(),
            Tag::Int(self.vibration_data.travel_time_in_ticks),
        )];
        if let Some(vibration) = &self.vibration_data.current_vibration {
            fields.push((
                "event".to_string(),
                Tag::String(vibration.event.id.to_string()),
            ));
            fields.push(("distance".to_string(), Tag::Float(vibration.distance)));
        }
        Tag::Compound(fields)
    }
}

impl BellBlockEntity {
    pub const EVENT_RING: i32 = 1;
    pub const DURATION: i32 = 50;
    pub const GLOW_DURATION: i32 = 60;
    pub const MIN_TICKS_BETWEEN_SEARCHES: u64 = 60;
    pub const MAX_RESONATION_TICKS: i32 = 40;
    pub const TICKS_BEFORE_RESONATION: i32 = 5;
    pub const SEARCH_RADIUS: f64 = 48.0;
    pub const HEAR_BELL_RADIUS: f64 = 32.0;
    pub const HIGHLIGHT_RAIDERS_RADIUS: f64 = 48.0;

    pub fn new() -> Self {
        Self {
            last_ring_timestamp: 0,
            ticks: 0,
            shaking: false,
            click_direction: None,
            heard_bell_entities: 0,
            nearby_raiders_within_hear_radius: 0,
            nearby_raiders_within_highlight_radius: 0,
            resonating: false,
            resonation_ticks: 0,
        }
    }

    pub fn direction_3d_data_value(direction: Direction) -> i32 {
        match direction {
            Direction::Down => 0,
            Direction::Up => 1,
            Direction::North => 2,
            Direction::South => 3,
            Direction::West => 4,
            Direction::East => 5,
        }
    }

    pub fn direction_from_3d_data_value(value: i32) -> Direction {
        match value {
            0 => Direction::Down,
            1 => Direction::Up,
            2 => Direction::North,
            3 => Direction::South,
            4 => Direction::West,
            5 => Direction::East,
            _ => Direction::Down,
        }
    }

    pub fn on_hit(&mut self, click_direction: Direction) -> BellBlockEvent {
        self.click_direction = Some(click_direction);
        if self.shaking {
            self.ticks = 0;
        } else {
            self.shaking = true;
        }
        BellBlockEvent {
            event_id: Self::EVENT_RING,
            event_param: Self::direction_3d_data_value(click_direction),
        }
    }

    pub fn trigger_event(
        &mut self,
        event_id: i32,
        event_param: i32,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) -> bool {
        if event_id != Self::EVENT_RING {
            return false;
        }
        self.update_entities(
            game_time,
            nearby_living_within_hear_radius,
            nearby_raiders_within_hear_radius,
            nearby_raiders_within_highlight_radius,
        );
        self.resonation_ticks = 0;
        self.click_direction = Some(Self::direction_from_3d_data_value(event_param));
        self.ticks = 0;
        self.shaking = true;
        true
    }

    pub fn update_entities(
        &mut self,
        game_time: u64,
        nearby_living_within_hear_radius: usize,
        nearby_raiders_within_hear_radius: usize,
        nearby_raiders_within_highlight_radius: usize,
    ) {
        if game_time > self.last_ring_timestamp + Self::MIN_TICKS_BETWEEN_SEARCHES
            || self.last_ring_timestamp == 0
        {
            self.last_ring_timestamp = game_time;
            self.heard_bell_entities = nearby_living_within_hear_radius;
            self.nearby_raiders_within_hear_radius = nearby_raiders_within_hear_radius;
            self.nearby_raiders_within_highlight_radius = nearby_raiders_within_highlight_radius;
        }
    }

    pub fn tick(&mut self) -> BellTickEffects {
        let mut effects = BellTickEffects {
            play_resonate_sound: false,
            glowing_raiders: 0,
        };

        if self.shaking {
            self.ticks += 1;
        }

        if self.ticks >= Self::DURATION {
            self.shaking = false;
            self.ticks = 0;
        }

        if self.ticks >= Self::TICKS_BEFORE_RESONATION
            && self.resonation_ticks == 0
            && self.nearby_raiders_within_hear_radius > 0
        {
            self.resonating = true;
            effects.play_resonate_sound = true;
        }

        if self.resonating {
            if self.resonation_ticks < Self::MAX_RESONATION_TICKS {
                self.resonation_ticks += 1;
            } else {
                effects.glowing_raiders = self.nearby_raiders_within_highlight_radius;
                self.resonating = false;
            }
        }

        effects
    }

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![])
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}

impl BrushableBlockEntity {
    pub const BRUSH_COOLDOWN_TICKS: u64 = 10;
    pub const BRUSH_RESET_TICKS: u64 = 40;
    pub const REQUIRED_BRUSHES_TO_BREAK: i32 = 10;
    pub const RETRACTION_SPEED: i32 = 2;
    pub const RETRACTION_TICKS: u64 = 4;

    pub fn new() -> Self {
        Self {
            brush_count: 0,
            brush_count_resets_at_tick: 0,
            cooldown_ends_at_tick: 0,
            item: None,
            hit_direction: None,
            loot_table: None,
            loot_table_seed: 0,
        }
    }

    pub fn set_loot_table(&mut self, loot_table: impl Into<String>, seed: i64) {
        self.loot_table = Some(loot_table.into());
        self.loot_table_seed = seed;
        self.item = None;
    }

    pub fn unpack_loot_table(&mut self, generated_item: Option<PotItemStack>) -> bool {
        if self.loot_table.take().is_some() {
            self.loot_table_seed = 0;
            self.item = generated_item;
            true
        } else {
            false
        }
    }

    pub fn brush(
        &mut self,
        game_time: u64,
        direction: Direction,
        generated_loot_item: Option<PotItemStack>,
    ) -> BrushResult {
        if self.hit_direction.is_none() {
            self.hit_direction = Some(direction);
        }
        self.brush_count_resets_at_tick = game_time + Self::BRUSH_RESET_TICKS;
        if game_time < self.cooldown_ends_at_tick {
            return BrushResult::CoolingDown;
        }

        self.cooldown_ends_at_tick = game_time + Self::BRUSH_COOLDOWN_TICKS;
        self.unpack_loot_table(generated_loot_item);
        self.brush_count += 1;
        if self.brush_count >= Self::REQUIRED_BRUSHES_TO_BREAK {
            self.brush_count = Self::REQUIRED_BRUSHES_TO_BREAK;
            return BrushResult::Completed;
        }

        BrushResult::InProgress {
            dusted: self.completion_state(),
        }
    }

    pub fn check_reset(&mut self, game_time: u64) -> Option<i32> {
        if self.brush_count != 0 && game_time >= self.brush_count_resets_at_tick {
            let previous = self.completion_state();
            self.brush_count = (self.brush_count - Self::RETRACTION_SPEED).max(0);
            let current = self.completion_state();
            if self.brush_count == 0 {
                self.hit_direction = None;
                self.brush_count_resets_at_tick = 0;
                self.cooldown_ends_at_tick = 0;
            } else {
                self.brush_count_resets_at_tick = game_time + Self::RETRACTION_TICKS;
            }
            return (previous != current).then_some(current);
        }

        None
    }

    pub fn completion_state(&self) -> i32 {
        if self.brush_count == 0 {
            0
        } else if self.brush_count < 3 {
            1
        } else if self.brush_count < 6 {
            2
        } else {
            3
        }
    }

    pub fn drop_content(&mut self) -> Option<(PotItemStack, Direction)> {
        let item = self.item.take()?;
        Some((item, self.hit_direction.unwrap_or(Direction::Up)))
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(loot_table) = &self.loot_table {
            fields.push(("LootTable".to_string(), Tag::String(loot_table.clone())));
            if self.loot_table_seed != 0 {
                fields.push(("LootTableSeed".to_string(), Tag::Long(self.loot_table_seed)));
            }
        } else if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let mut brushable = Self::new();
        let Some(entries) = compound_entries(tag) else {
            return brushable;
        };
        brushable.loot_table = get_string(entries, "LootTable").map(ToString::to_string);
        brushable.loot_table_seed = entries
            .iter()
            .find(|(name, _)| name == "LootTableSeed")
            .map(|(_, tag)| tag_long_or_zero(tag))
            .unwrap_or(0);
        if brushable.loot_table.is_none() {
            brushable.item = entries
                .iter()
                .find(|(name, _)| name == "item")
                .and_then(|(_, tag)| PotItemStack::from_tag(tag));
        }
        brushable.hit_direction =
            get_string(entries, "hit_direction").and_then(direction_from_name);
        brushable
    }

    pub fn get_update_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(direction) = self.hit_direction {
            fields.push((
                "hit_direction".to_string(),
                Tag::String(direction_name(direction).to_string()),
            ));
        }
        if let Some(item) = &self.item {
            fields.push(("item".to_string(), item.to_tag()));
        }
        Tag::Compound(fields)
    }
}

// Source: decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BlockEntityType.java
pub const BLOCK_ENTITY_TYPES: &[BlockEntityTypeInfo] = &[
    info(
        BlockEntityTypeId::Furnace,
        "furnace",
        &["minecraft:furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Chest,
        "chest",
        &[
            "minecraft:chest",
            "minecraft:copper_chest",
            "minecraft:exposed_copper_chest",
            "minecraft:weathered_copper_chest",
            "minecraft:oxidized_copper_chest",
            "minecraft:waxed_copper_chest",
            "minecraft:waxed_exposed_copper_chest",
            "minecraft:waxed_weathered_copper_chest",
            "minecraft:waxed_oxidized_copper_chest",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TrappedChest,
        "trapped_chest",
        &["minecraft:trapped_chest"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EnderChest,
        "ender_chest",
        &["minecraft:ender_chest"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Jukebox,
        "jukebox",
        &["minecraft:jukebox"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Dispenser,
        "dispenser",
        &["minecraft:dispenser"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Dropper,
        "dropper",
        &["minecraft:dropper"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Sign,
        "sign",
        &[
            "minecraft:oak_sign",
            "minecraft:spruce_sign",
            "minecraft:birch_sign",
            "minecraft:acacia_sign",
            "minecraft:cherry_sign",
            "minecraft:jungle_sign",
            "minecraft:dark_oak_sign",
            "minecraft:pale_oak_sign",
            "minecraft:mangrove_sign",
            "minecraft:crimson_sign",
            "minecraft:warped_sign",
            "minecraft:bamboo_sign",
            "minecraft:oak_wall_sign",
            "minecraft:spruce_wall_sign",
            "minecraft:birch_wall_sign",
            "minecraft:acacia_wall_sign",
            "minecraft:cherry_wall_sign",
            "minecraft:jungle_wall_sign",
            "minecraft:dark_oak_wall_sign",
            "minecraft:pale_oak_wall_sign",
            "minecraft:mangrove_wall_sign",
            "minecraft:crimson_wall_sign",
            "minecraft:warped_wall_sign",
            "minecraft:bamboo_wall_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::HangingSign,
        "hanging_sign",
        &[
            "minecraft:oak_hanging_sign",
            "minecraft:spruce_hanging_sign",
            "minecraft:birch_hanging_sign",
            "minecraft:acacia_hanging_sign",
            "minecraft:cherry_hanging_sign",
            "minecraft:jungle_hanging_sign",
            "minecraft:dark_oak_hanging_sign",
            "minecraft:pale_oak_hanging_sign",
            "minecraft:crimson_hanging_sign",
            "minecraft:warped_hanging_sign",
            "minecraft:mangrove_hanging_sign",
            "minecraft:bamboo_hanging_sign",
            "minecraft:oak_wall_hanging_sign",
            "minecraft:spruce_wall_hanging_sign",
            "minecraft:birch_wall_hanging_sign",
            "minecraft:acacia_wall_hanging_sign",
            "minecraft:cherry_wall_hanging_sign",
            "minecraft:jungle_wall_hanging_sign",
            "minecraft:dark_oak_wall_hanging_sign",
            "minecraft:pale_oak_wall_hanging_sign",
            "minecraft:crimson_wall_hanging_sign",
            "minecraft:warped_wall_hanging_sign",
            "minecraft:mangrove_wall_hanging_sign",
            "minecraft:bamboo_wall_hanging_sign",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::MobSpawner,
        "mob_spawner",
        &["minecraft:spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::CreakingHeart,
        "creaking_heart",
        &["minecraft:creaking_heart"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Piston,
        "piston",
        &["minecraft:moving_piston"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BrewingStand,
        "brewing_stand",
        &["minecraft:brewing_stand"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::EnchantingTable,
        "enchanting_table",
        &["minecraft:enchanting_table"],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::EndPortal,
        "end_portal",
        &["minecraft:end_portal"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Beacon,
        "beacon",
        &["minecraft:beacon"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Skull,
        "skull",
        &[
            "minecraft:skeleton_wall_skull",
            "minecraft:creeper_head",
            "minecraft:creeper_wall_head",
            "minecraft:dragon_wall_head",
            "minecraft:skeleton_skull",
            "minecraft:player_head",
            "minecraft:player_wall_head",
            "minecraft:dragon_head",
            "minecraft:zombie_head",
            "minecraft:zombie_wall_head",
            "minecraft:wither_skeleton_skull",
            "minecraft:wither_skeleton_wall_skull",
            "minecraft:piglin_head",
            "minecraft:piglin_wall_head",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::DaylightDetector,
        "daylight_detector",
        &["minecraft:daylight_detector"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Hopper,
        "hopper",
        &["minecraft:hopper"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Comparator,
        "comparator",
        &["minecraft:comparator"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Banner,
        "banner",
        &[
            "minecraft:white_banner",
            "minecraft:orange_banner",
            "minecraft:magenta_banner",
            "minecraft:light_blue_banner",
            "minecraft:yellow_banner",
            "minecraft:lime_banner",
            "minecraft:pink_banner",
            "minecraft:gray_banner",
            "minecraft:light_gray_banner",
            "minecraft:cyan_banner",
            "minecraft:purple_banner",
            "minecraft:blue_banner",
            "minecraft:brown_banner",
            "minecraft:green_banner",
            "minecraft:red_banner",
            "minecraft:black_banner",
            "minecraft:white_wall_banner",
            "minecraft:orange_wall_banner",
            "minecraft:magenta_wall_banner",
            "minecraft:light_blue_wall_banner",
            "minecraft:yellow_wall_banner",
            "minecraft:lime_wall_banner",
            "minecraft:pink_wall_banner",
            "minecraft:gray_wall_banner",
            "minecraft:light_gray_wall_banner",
            "minecraft:cyan_wall_banner",
            "minecraft:purple_wall_banner",
            "minecraft:blue_wall_banner",
            "minecraft:brown_wall_banner",
            "minecraft:green_wall_banner",
            "minecraft:red_wall_banner",
            "minecraft:black_wall_banner",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::StructureBlock,
        "structure_block",
        &["minecraft:structure_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::EndGateway,
        "end_gateway",
        &["minecraft:end_gateway"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CommandBlock,
        "command_block",
        &[
            "minecraft:command_block",
            "minecraft:chain_command_block",
            "minecraft:repeating_command_block",
        ],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::ShulkerBox,
        "shulker_box",
        &[
            "minecraft:shulker_box",
            "minecraft:black_shulker_box",
            "minecraft:blue_shulker_box",
            "minecraft:brown_shulker_box",
            "minecraft:cyan_shulker_box",
            "minecraft:gray_shulker_box",
            "minecraft:green_shulker_box",
            "minecraft:light_blue_shulker_box",
            "minecraft:light_gray_shulker_box",
            "minecraft:lime_shulker_box",
            "minecraft:magenta_shulker_box",
            "minecraft:orange_shulker_box",
            "minecraft:pink_shulker_box",
            "minecraft:purple_shulker_box",
            "minecraft:red_shulker_box",
            "minecraft:white_shulker_box",
            "minecraft:yellow_shulker_box",
        ],
        BlockEntityTickKind::Client,
        false,
    ),
    info(
        BlockEntityTypeId::Bed,
        "bed",
        &[
            "minecraft:black_bed",
            "minecraft:blue_bed",
            "minecraft:brown_bed",
            "minecraft:cyan_bed",
            "minecraft:gray_bed",
            "minecraft:green_bed",
            "minecraft:light_blue_bed",
            "minecraft:light_gray_bed",
            "minecraft:lime_bed",
            "minecraft:magenta_bed",
            "minecraft:orange_bed",
            "minecraft:pink_bed",
            "minecraft:purple_bed",
            "minecraft:red_bed",
            "minecraft:white_bed",
            "minecraft:yellow_bed",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Conduit,
        "conduit",
        &["minecraft:conduit"],
        BlockEntityTickKind::Both,
        false,
    ),
    info(
        BlockEntityTypeId::Barrel,
        "barrel",
        &["minecraft:barrel"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Smoker,
        "smoker",
        &["minecraft:smoker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::BlastFurnace,
        "blast_furnace",
        &["minecraft:blast_furnace"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Lectern,
        "lectern",
        &["minecraft:lectern"],
        BlockEntityTickKind::None,
        true,
    ),
    info(
        BlockEntityTypeId::Bell,
        "bell",
        &["minecraft:bell"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Jigsaw,
        "jigsaw",
        &["minecraft:jigsaw"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Campfire,
        "campfire",
        &["minecraft:campfire", "minecraft:soul_campfire"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::Beehive,
        "beehive",
        &["minecraft:bee_nest", "minecraft:beehive"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkSensor,
        "sculk_sensor",
        &["minecraft:sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::CalibratedSculkSensor,
        "calibrated_sculk_sensor",
        &["minecraft:calibrated_sculk_sensor"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkCatalyst,
        "sculk_catalyst",
        &["minecraft:sculk_catalyst"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::SculkShrieker,
        "sculk_shrieker",
        &["minecraft:sculk_shrieker"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::ChiseledBookshelf,
        "chiseled_bookshelf",
        &["minecraft:chiseled_bookshelf"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Shelf,
        "shelf",
        &[
            "minecraft:acacia_shelf",
            "minecraft:bamboo_shelf",
            "minecraft:birch_shelf",
            "minecraft:cherry_shelf",
            "minecraft:crimson_shelf",
            "minecraft:dark_oak_shelf",
            "minecraft:jungle_shelf",
            "minecraft:mangrove_shelf",
            "minecraft:oak_shelf",
            "minecraft:pale_oak_shelf",
            "minecraft:spruce_shelf",
            "minecraft:warped_shelf",
        ],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::BrushableBlock,
        "brushable_block",
        &["minecraft:suspicious_sand", "minecraft:suspicious_gravel"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::DecoratedPot,
        "decorated_pot",
        &["minecraft:decorated_pot"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::Crafter,
        "crafter",
        &["minecraft:crafter"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TrialSpawner,
        "trial_spawner",
        &["minecraft:trial_spawner"],
        BlockEntityTickKind::Server,
        true,
    ),
    info(
        BlockEntityTypeId::Vault,
        "vault",
        &["minecraft:vault"],
        BlockEntityTickKind::Server,
        false,
    ),
    info(
        BlockEntityTypeId::TestBlock,
        "test_block",
        &["minecraft:test_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::TestInstanceBlock,
        "test_instance_block",
        &["minecraft:test_instance_block"],
        BlockEntityTickKind::None,
        false,
    ),
    info(
        BlockEntityTypeId::CopperGolemStatue,
        "copper_golem_statue",
        &[
            "minecraft:copper_golem_statue",
            "minecraft:exposed_copper_golem_statue",
            "minecraft:weathered_copper_golem_statue",
            "minecraft:oxidized_copper_golem_statue",
            "minecraft:waxed_copper_golem_statue",
            "minecraft:waxed_exposed_copper_golem_statue",
            "minecraft:waxed_weathered_copper_golem_statue",
            "minecraft:waxed_oxidized_copper_golem_statue",
        ],
        BlockEntityTickKind::Server,
        false,
    ),
];

const fn info(
    id: BlockEntityTypeId,
    key: &'static str,
    valid_blocks: &'static [&'static str],
    tick_kind: BlockEntityTickKind,
    op_only_custom_data: bool,
) -> BlockEntityTypeInfo {
    BlockEntityTypeInfo {
        id,
        key,
        valid_blocks,
        tick_kind,
        op_only_custom_data,
    }
}

pub fn type_info(ty: BlockEntityTypeId) -> &'static BlockEntityTypeInfo {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.id == ty)
        .expect("block entity type table covers every id")
}

pub fn type_by_key(key: &str) -> Option<BlockEntityTypeId> {
    BLOCK_ENTITY_TYPES
        .iter()
        .find(|info| info.key == key || format!("minecraft:{}", info.key) == key)
        .map(|info| info.id)
}

pub fn is_valid_block_state(ty: BlockEntityTypeId, block_state: &str) -> bool {
    type_info(ty).valid_blocks.contains(&block_state)
}

pub fn only_op_can_set_nbt(ty: BlockEntityTypeId) -> bool {
    type_info(ty).op_only_custom_data
}

pub fn has_block_entity_for_block(registry_id: &str) -> bool {
    BLOCK_ENTITY_TYPES
        .iter()
        .any(|entry| entry.valid_blocks.contains(&registry_id))
}

impl BlockEntity {
    pub fn new(
        ty: BlockEntityTypeId,
        pos: BlockPos,
        block_state: &str,
    ) -> Result<Self, BlockEntityError> {
        if !is_valid_block_state(ty, block_state) {
            return Err(BlockEntityError::InvalidBlockState {
                ty,
                block_state: block_state.to_string(),
            });
        }

        Ok(Self {
            ty,
            pos,
            block_state: block_state.to_string(),
            custom_data: BTreeMap::new(),
            components: BTreeMap::new(),
            has_level: false,
            removed: false,
            changed: false,
            tick_count: 0,
        })
    }

    pub fn set_level(&mut self) {
        self.has_level = true;
    }

    pub fn set_removed(&mut self) {
        self.removed = true;
    }

    pub fn clear_removed(&mut self) {
        self.removed = false;
    }

    pub fn set_changed(&mut self) {
        if self.has_level {
            self.changed = true;
        }
    }

    pub fn save_custom_only(&self) -> Tag {
        compound_from_map(&self.custom_data)
    }

    pub fn save_without_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn save_with_id(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        compound_from_map(&values)
    }

    pub fn save_with_full_metadata(&self) -> Tag {
        let mut values = self.custom_data.clone();
        values.insert(
            "id".to_string(),
            Tag::String(type_info(self.ty).key.to_string()),
        );
        values.insert("x".to_string(), Tag::Int(self.pos.x));
        values.insert("y".to_string(), Tag::Int(self.pos.y));
        values.insert("z".to_string(), Tag::Int(self.pos.z));
        values.insert(
            "components".to_string(),
            compound_from_map(&self.components),
        );
        compound_from_map(&values)
    }

    pub fn data_get_block_nbt(&self) -> Tag {
        self.save_with_full_metadata()
    }

    pub fn destruction_drops(
        &self,
        context: BlockEntityDestructionContext,
    ) -> BlockEntityDestructionDrops {
        if !context.do_tile_drops || !context.explosion_survives {
            return BlockEntityDestructionDrops {
                block_item: None,
                stored_items: Vec::new(),
            };
        }

        let block_item = (context.correct_tool || context.silk_touch)
            .then(|| block_item_from_state(&self.block_state).to_string());
        let stored_items = stored_item_drops_from_tag(&self.save_without_metadata());
        BlockEntityDestructionDrops {
            block_item,
            stored_items,
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        match self.ty {
            BlockEntityTypeId::Chest
            | BlockEntityTypeId::TrappedChest
            | BlockEntityTypeId::Barrel
            | BlockEntityTypeId::Hopper
            | BlockEntityTypeId::Dispenser
            | BlockEntityTypeId::Dropper => Tag::Compound(Vec::new()),
            _ => self.save_without_metadata(),
        }
    }

    pub fn get_update_packet(&self) -> ClientboundBlockEntityDataPacket {
        ClientboundBlockEntityDataPacket {
            pos: self.pos,
            ty: self.ty,
            tag: self.get_update_tag(),
        }
    }

    pub fn handle_update_tag(&mut self, tag: &Tag) {
        let Some(entries) = compound_entries(tag) else {
            return;
        };

        self.custom_data.clear();
        self.components.clear();
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    self.components = map_from_compound(value);
                }
                _ => {
                    self.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    pub fn tick(&mut self, client_side: bool) -> bool {
        let tick_kind = type_info(self.ty).tick_kind;
        let should_tick = matches!(
            (tick_kind, client_side),
            (BlockEntityTickKind::Both, _)
                | (BlockEntityTickKind::Server, false)
                | (BlockEntityTickKind::Client, true)
        );

        if should_tick && !self.removed && self.has_level {
            self.tick_count += 1;
            true
        } else {
            false
        }
    }
}

impl TickingBlockEntity {
    pub fn new(entity: BlockEntity, client_side: bool) -> Self {
        Self {
            entity,
            client_side,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.entity.tick(self.client_side)
    }

    pub fn is_removed(&self) -> bool {
        self.entity.removed
    }

    pub fn pos(&self) -> BlockPos {
        self.entity.pos
    }

    pub fn type_key(&self) -> &'static str {
        type_info(self.entity.ty).key
    }
}

pub fn load_static(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
) -> Result<BlockEntity, BlockEntityError> {
    let values = compound_entries(tag);
    let id = values
        .and_then(|entries| get_string(entries, "id"))
        .ok_or(BlockEntityError::MissingId)?;
    let ty = type_by_key(id).ok_or_else(|| BlockEntityError::UnknownType(id.to_string()))?;
    let mut entity = BlockEntity::new(ty, pos, block_state)?;

    if let Some(entries) = values {
        for (key, value) in entries {
            match key.as_str() {
                "id" | "x" | "y" | "z" => {}
                "components" => {
                    entity.components = map_from_compound(value);
                }
                _ => {
                    entity.custom_data.insert(key.clone(), value.clone());
                }
            }
        }
    }

    Ok(entity)
}

pub fn load_static_with_data_version(
    pos: BlockPos,
    block_state: &str,
    tag: &Tag,
    data_version: i32,
) -> Result<BlockEntity, BlockEntityError> {
    require_current_world_data_version(data_version)
        .map_err(BlockEntityError::UnsupportedDataVersion)?;
    load_static(pos, block_state, tag)
}

pub fn corrected_pos_from_chunk(base_chunk_x: i32, base_chunk_z: i32, tag: &Tag) -> BlockPos {
    let entries = compound_entries(tag);
    let x = entries
        .and_then(|entries| get_int(entries, "x"))
        .unwrap_or(0);
    let y = entries
        .and_then(|entries| get_int(entries, "y"))
        .unwrap_or(0);
    let z = entries
        .and_then(|entries| get_int(entries, "z"))
        .unwrap_or(0);
    let section_x = x.div_euclid(16);
    let section_z = z.div_euclid(16);

    if section_x == base_chunk_x && section_z == base_chunk_z {
        BlockPos { x, y, z }
    } else {
        BlockPos {
            x: base_chunk_x * 16 + x.rem_euclid(16),
            y,
            z: base_chunk_z * 16 + z.rem_euclid(16),
        }
    }
}

pub fn block_entity_packet_from_chunk(
    entity: &BlockEntity,
    chunk_min_y: i32,
) -> (u8, i16, BlockEntityTypeId, Tag) {
    let packed_xz = ((entity.pos.x & 15) << 4) | (entity.pos.z & 15);
    let section_y = (entity.pos.y - chunk_min_y) as i16;
    (
        packed_xz as u8,
        section_y,
        entity.ty,
        entity.get_update_tag(),
    )
}

fn compound_from_map(values: &BTreeMap<String, Tag>) -> Tag {
    Tag::Compound(
        values
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    )
}

fn map_from_compound(tag: &Tag) -> BTreeMap<String, Tag> {
    compound_entries(tag)
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

fn inventory_comparator_output(items: &[Option<PotItemStack>]) -> u8 {
    let non_empty = items.iter().filter(|stack| stack.is_some()).count();
    if non_empty == 0 {
        return 0;
    }
    let fullness = items
        .iter()
        .filter_map(|stack| stack.as_ref())
        .map(|stack| (stack.count.max(0) as f32 / 64.0).min(1.0))
        .sum::<f32>()
        / items.len().max(1) as f32;
    (1 + (fullness * 14.0).floor() as u8).min(MAX_SIGNAL)
}

fn block_item_from_state(block_state: &str) -> &str {
    block_state.split('[').next().unwrap_or(block_state)
}

fn stored_item_drops_from_tag(tag: &Tag) -> Vec<PotItemStack> {
    let Some(entries) = compound_entries(tag) else {
        return Vec::new();
    };
    let mut drops = Vec::new();

    if let Some(Tag::List(items)) = entries
        .iter()
        .find(|(key, _)| key == "Items")
        .map(|(_, tag)| tag)
    {
        drops.extend(items.iter().filter_map(PotItemStack::from_tag));
    }

    for key in ["item", "Book", "RecordItem"] {
        if let Some(item) = entries
            .iter()
            .find(|(name, _)| name == key)
            .and_then(|(_, tag)| PotItemStack::from_tag(tag))
        {
            drops.push(item);
        }
    }
    drops
}

fn container_items_tag(items: &[Option<PotItemStack>]) -> Tag {
    Tag::List(
        items
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| {
                let mut tag = item.as_ref()?.to_tag();
                if let Tag::Compound(entries) = &mut tag {
                    entries.insert(0, ("Slot".to_string(), Tag::Byte(slot as i8)));
                }
                Some(tag)
            })
            .collect(),
    )
}

fn load_container_items(entries: &[(String, Tag)], items: &mut [Option<PotItemStack>]) {
    if let Some(Tag::List(saved_items)) = entries
        .iter()
        .find(|(name, _)| name == "Items")
        .map(|(_, tag)| tag)
    {
        for item in saved_items {
            if let Some(item_entries) = compound_entries(item) {
                let slot = get_byte(item_entries, "Slot").unwrap_or(-1);
                if (0..items.len() as i8).contains(&slot) {
                    items[slot as usize] = PotItemStack::from_tag(item);
                }
            }
        }
    }
}

fn shrink_stack(stack: &mut Option<PotItemStack>, amount: i32) {
    if let Some(item) = stack {
        item.count -= amount;
        if item.count <= 0 {
            *stack = None;
        }
    }
}

fn is_brewing_fuel(stack: &PotItemStack) -> bool {
    stack.item_id == "minecraft:blaze_powder" && stack.count > 0
}

fn is_brewing_container(item_id: &str) -> bool {
    let (item, potion) = brewing_stack_parts(item_id);
    matches!(
        item,
        "minecraft:potion" | "minecraft:splash_potion" | "minecraft:lingering_potion"
    ) && potion.is_some()
        || item == "minecraft:glass_bottle"
}

fn brewing_stack_id(item: &str, potion: &str) -> String {
    format!("{item}#{potion}")
}

fn brewing_stack_parts(item_id: &str) -> (&str, Option<&str>) {
    item_id
        .split_once('#')
        .map_or((item_id, None), |(item, potion)| (item, Some(potion)))
}

fn sign_line_to_tag(line: &SignLine) -> Tag {
    if let Some(command) = &line.click_command {
        Tag::Compound(vec![
            ("text".to_string(), Tag::String(line.raw.clone())),
            ("run_command".to_string(), Tag::String(command.clone())),
        ])
    } else {
        Tag::String(line.raw.clone())
    }
}

fn sign_line_from_tag(tag: &Tag) -> SignLine {
    match tag {
        Tag::String(raw) => SignLine::new(raw.clone(), raw.clone()),
        Tag::Compound(entries) => {
            let raw = get_string(entries, "text").unwrap_or("").to_string();
            let mut line = SignLine::new(raw.clone(), raw);
            line.click_command = get_string(entries, "run_command").map(str::to_string);
            line
        }
        _ => SignLine::default(),
    }
}

fn compound_entries(tag: &Tag) -> Option<&Vec<(String, Tag)>> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn get_string<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::String(value) if name == key => Some(value.as_str()),
        _ => None,
    })
}

fn get_int(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Int(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_short(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Short(value) if name == key => Some(i32::from(*value)),
        _ => None,
    })
}

fn get_long(entries: &[(String, Tag)], key: &str) -> Option<i64> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Long(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_int_array<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a [i32]> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::IntArray(value) if name == key => Some(value.as_slice()),
        _ => None,
    })
}

fn string_list_field(entries: &[(String, Tag)], key: &str) -> Option<Vec<String>> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::List(values) if name == key => Some(
            values
                .iter()
                .filter_map(|tag| match tag {
                    Tag::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    })
}

fn string_list_tag(values: impl IntoIterator<Item = String>) -> Tag {
    Tag::List(values.into_iter().map(Tag::String).collect())
}

fn int_range_field(entries: &[(String, Tag)], key: &str) -> Option<(i32, i32)> {
    let values = get_int_array(entries, key)?;
    (values.len() == 2).then_some((values[0], values[1]))
}

fn get_float(entries: &[(String, Tag)], key: &str) -> Option<f32> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Float(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_double(entries: &[(String, Tag)], key: &str) -> Option<f64> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Double(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_byte(entries: &[(String, Tag)], key: &str) -> Option<i8> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value),
        _ => None,
    })
}

fn get_bool(entries: &[(String, Tag)], key: &str) -> Option<bool> {
    entries.iter().find_map(|(name, value)| match value {
        Tag::Byte(value) if name == key => Some(*value != 0),
        _ => None,
    })
}

fn block_pos_to_tag(pos: BlockPos) -> Tag {
    Tag::List(vec![Tag::Int(pos.x), Tag::Int(pos.y), Tag::Int(pos.z)])
}

fn offset_pos(pos: BlockPos, x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos {
        x: pos.x + x,
        y: pos.y + y,
        z: pos.z + z,
    }
}

fn closer_than(left: BlockPos, right: BlockPos, range: f64) -> bool {
    let dx = f64::from(left.x - right.x);
    let dy = f64::from(left.y - right.y);
    let dz = f64::from(left.z - right.z);
    dx * dx + dy * dy + dz * dz < range * range
}

fn chessboard_distance(left: BlockPos, right: BlockPos) -> i32 {
    (left.x - right.x)
        .abs()
        .max((left.y - right.y).abs())
        .max((left.z - right.z).abs())
}

fn block_pos_from_tag(tag: &Tag) -> Option<BlockPos> {
    match tag {
        Tag::List(values) if values.len() == 3 => Some(BlockPos {
            x: tag_int_or_zero(&values[0]),
            y: tag_int_or_zero(&values[1]),
            z: tag_int_or_zero(&values[2]),
        }),
        _ => None,
    }
}

fn vibration_data_from_tag(tag: &Tag) -> VibrationData {
    let Some(entries) = compound_entries(tag) else {
        return VibrationData::new();
    };
    let mut data = VibrationData::new();
    data.travel_time_in_ticks = get_int(entries, "travel_time_in_ticks").unwrap_or(0).max(0);
    data.reload_vibration_particle =
        get_bool(entries, "reload_vibration_particle").unwrap_or(false);
    if let Some(event) = get_string(entries, "event").and_then(crate::game_event::game_event_by_id)
    {
        data.current_vibration = Some(VibrationInfo {
            event,
            distance: get_float(entries, "distance").unwrap_or(0.0),
            pos: crate::entity_physics::Vec3::ZERO,
            source_entity: None,
            projectile_owner: None,
        });
    }
    data
}

fn tag_int_or_zero(tag: &Tag) -> i32 {
    match tag {
        Tag::Byte(value) => *value as i32,
        Tag::Short(value) => *value as i32,
        Tag::Int(value) => *value,
        Tag::Long(value) => *value as i32,
        _ => 0,
    }
}

fn weighted_spawn_data(values: &[SpawnDataModel], roll: usize) -> Option<&SpawnDataModel> {
    if values.is_empty() {
        return None;
    }
    let total_weight: i32 = values.iter().map(|value| value.weight.max(1)).sum();
    let mut remaining = (roll as i32).rem_euclid(total_weight.max(1));
    for value in values {
        remaining -= value.weight.max(1);
        if remaining < 0 {
            return Some(value);
        }
    }
    values.last()
}

fn pot_item_to_tag(item: &PotItemStack, slot: i8) -> Tag {
    let mut tag = match item.to_tag() {
        Tag::Compound(entries) => entries,
        _ => Vec::new(),
    };
    tag.push(("Slot".to_string(), Tag::Byte(slot)));
    Tag::Compound(tag)
}

fn pot_item_from_tag(tag: &Tag) -> Option<PotItemStack> {
    PotItemStack::from_tag(tag)
}

fn tag_long_or_zero(tag: &Tag) -> i64 {
    match tag {
        Tag::Byte(value) => *value as i64,
        Tag::Short(value) => *value as i64,
        Tag::Int(value) => *value as i64,
        Tag::Long(value) => *value,
        _ => 0,
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

fn direction_from_name(value: &str) -> Option<Direction> {
    match value {
        "down" => Some(Direction::Down),
        "up" => Some(Direction::Up),
        "north" => Some(Direction::North),
        "south" => Some(Direction::South),
        "west" => Some(Direction::West),
        "east" => Some(Direction::East),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> BlockPos {
        BlockPos {
            x: 18,
            y: 64,
            z: 35,
        }
    }

    fn stack(item_id: &str, count: i32) -> PotItemStack {
        PotItemStack {
            item_id: item_id.to_string(),
            count,
        }
    }

    fn assert_menu(
        menu: &BlockEntityMenuOpen,
        container_id: i32,
        menu_type: &'static str,
        initial_slots: &[Option<PotItemStack>],
    ) {
        assert_eq!(menu.container_id, container_id);
        assert_eq!(menu.menu_type, menu_type);
        assert_eq!(menu.initial_slots, initial_slots);
    }

    #[test]
    fn block_entity_registry_matches_26_1_2_type_surface() {
        assert_eq!(BLOCK_ENTITY_TYPES.len(), 49);
        assert_eq!(type_info(BlockEntityTypeId::Furnace).key, "furnace");
        assert_eq!(
            type_info(BlockEntityTypeId::CopperGolemStatue).key,
            "copper_golem_statue"
        );
        assert_eq!(
            type_info(BlockEntityTypeId::CopperGolemStatue).valid_blocks,
            &[
                "minecraft:copper_golem_statue",
                "minecraft:exposed_copper_golem_statue",
                "minecraft:weathered_copper_golem_statue",
                "minecraft:oxidized_copper_golem_statue",
                "minecraft:waxed_copper_golem_statue",
                "minecraft:waxed_exposed_copper_golem_statue",
                "minecraft:waxed_weathered_copper_golem_statue",
                "minecraft:waxed_oxidized_copper_golem_statue",
            ]
        );
        assert!(is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:oak_wall_sign"
        ));
        assert!(!is_valid_block_state(
            BlockEntityTypeId::Sign,
            "minecraft:stone"
        ));
    }

    #[test]
    fn op_only_custom_data_matches_vanilla_guarded_types() {
        assert!(only_op_can_set_nbt(BlockEntityTypeId::CommandBlock));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Lectern));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::Sign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::HangingSign));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::MobSpawner));
        assert!(only_op_can_set_nbt(BlockEntityTypeId::TrialSpawner));
        assert!(!only_op_can_set_nbt(BlockEntityTypeId::Vault));
    }

    #[test]
    fn validates_block_state_on_creation_and_load() {
        assert!(BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").is_ok());
        assert_eq!(
            BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:furnace"),
            Err(BlockEntityError::InvalidBlockState {
                ty: BlockEntityTypeId::Chest,
                block_state: "minecraft:furnace".to_string()
            })
        );
    }

    #[test]
    fn detects_block_entity_support_for_block_states() {
        assert!(has_block_entity_for_block("minecraft:chest"));
        assert!(has_block_entity_for_block("minecraft:oak_sign"));
        assert!(has_block_entity_for_block("minecraft:oak_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:lectern"));
        assert!(has_block_entity_for_block("minecraft:command_block"));
        assert!(has_block_entity_for_block(
            "minecraft:wither_skeleton_skull"
        ));
        assert!(has_block_entity_for_block("minecraft:red_banner"));
        assert!(has_block_entity_for_block("minecraft:conduit"));
        assert!(has_block_entity_for_block("minecraft:bell"));
        assert!(has_block_entity_for_block("minecraft:crimson_hanging_sign"));
        assert!(has_block_entity_for_block("minecraft:brown_banner"));
        assert!(has_block_entity_for_block("minecraft:waxed_copper_chest"));
        assert!(has_block_entity_for_block("minecraft:dark_oak_wall_sign"));
        assert!(has_block_entity_for_block("minecraft:spawner"));
        assert!(has_block_entity_for_block("minecraft:vault"));
        assert!(has_block_entity_for_block("minecraft:trial_spawner"));
        assert!(has_block_entity_for_block(
            "minecraft:calibrated_sculk_sensor"
        ));
        assert!(has_block_entity_for_block("minecraft:chiseled_bookshelf"));
        assert!(has_block_entity_for_block("minecraft:suspicious_sand"));
        assert!(has_block_entity_for_block("minecraft:green_bed"));
        assert!(has_block_entity_for_block("minecraft:black_shulker_box"));
        assert!(has_block_entity_for_block("minecraft:crimson_shelf"));
        assert!(has_block_entity_for_block(
            "minecraft:waxed_oxidized_copper_golem_statue"
        ));
        assert!(has_block_entity_for_block(
            "minecraft:warped_wall_hanging_sign"
        ));
        assert!(has_block_entity_for_block("minecraft:campfire"));
        assert!(!has_block_entity_for_block("minecraft:candle"));
        assert!(!has_block_entity_for_block("minecraft:cauldron"));
        assert!(!has_block_entity_for_block("minecraft:stone"));
        assert!(!has_block_entity_for_block("minecraft:dirt"));
    }

    #[test]
    fn bed_block_entity_is_color_only_placeholder() {
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:white_bed"),
            Some(BedBlockEntity {
                color: DyeColor::White
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:light_blue_bed"),
            Some(BedBlockEntity {
                color: DyeColor::LightBlue
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state("minecraft:black_bed"),
            Some(BedBlockEntity {
                color: DyeColor::Black
            })
        );
        assert_eq!(BedBlockEntity::from_block_state("minecraft:stone"), None);

        let bed = BlockEntity::new(BlockEntityTypeId::Bed, pos(), "minecraft:red_bed").unwrap();
        assert_eq!(bed.ty, BlockEntityTypeId::Bed);
        assert_eq!(
            BedBlockEntity::from_block_state(&bed.block_state),
            Some(BedBlockEntity {
                color: DyeColor::Red
            })
        );
        assert_eq!(
            BedBlockEntity::from_block_state(&bed.block_state)
                .unwrap()
                .save_additional(),
            Tag::Compound(Vec::new())
        );
        assert_eq!(
            bed.save_with_full_metadata(),
            Tag::Compound(vec![
                ("components".to_string(), Tag::Compound(Vec::new())),
                ("id".to_string(), Tag::String("bed".to_string())),
                ("x".to_string(), Tag::Int(pos().x)),
                ("y".to_string(), Tag::Int(pos().y)),
                ("z".to_string(), Tag::Int(pos().z)),
            ])
        );
    }

    #[test]
    fn end_portal_block_entity_is_zero_data_portal_placeholder() {
        let portal =
            BlockEntity::new(BlockEntityTypeId::EndPortal, pos(), "minecraft:end_portal").unwrap();
        assert_eq!(portal.ty, BlockEntityTypeId::EndPortal);
        assert_eq!(type_info(BlockEntityTypeId::EndPortal).key, "end_portal");
        assert_eq!(
            type_info(BlockEntityTypeId::EndPortal).valid_blocks,
            &["minecraft:end_portal"]
        );
        assert_eq!(
            EndPortalBlockEntity.save_additional(),
            Tag::Compound(Vec::new())
        );
        assert_eq!(
            portal.save_with_full_metadata(),
            Tag::Compound(vec![
                ("components".to_string(), Tag::Compound(Vec::new())),
                ("id".to_string(), Tag::String("end_portal".to_string())),
                ("x".to_string(), Tag::Int(pos().x)),
                ("y".to_string(), Tag::Int(pos().y)),
                ("z".to_string(), Tag::Int(pos().z)),
            ])
        );
    }

    #[test]
    fn end_gateway_block_entity_saves_ticks_cooldown_and_exit_like_java() {
        let mut gateway = TheEndGatewayBlockEntity::new();
        assert!(gateway.is_spawning());
        assert!(!gateway.is_cooling_down());
        assert_eq!(gateway.spawn_percent(0.0), 0.0);
        assert_eq!(TheEndGatewayBlockEntity::SPAWN_TIME, 200);
        assert_eq!(TheEndGatewayBlockEntity::COOLDOWN_TIME, 40);
        assert_eq!(TheEndGatewayBlockEntity::ATTENTION_INTERVAL, 2400);
        assert_eq!(TheEndGatewayBlockEntity::GATEWAY_HEIGHT_ABOVE_SURFACE, 10);

        gateway.age = 199;
        assert_eq!(gateway.spawn_percent(0.5), 0.9975);
        assert!(gateway.portal_tick());
        assert_eq!(gateway.age, 200);
        assert!(!gateway.is_spawning());

        gateway.set_exit_position(
            BlockPos {
                x: 12,
                y: 80,
                z: -7,
            },
            true,
        );
        let saved = gateway.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("Age".to_string(), Tag::Long(200)),
                (
                    "exit_portal".to_string(),
                    Tag::List(vec![Tag::Int(12), Tag::Int(80), Tag::Int(-7)])
                ),
                ("ExactTeleport".to_string(), Tag::Byte(1)),
            ])
        );
        assert_eq!(TheEndGatewayBlockEntity::load_additional(&saved), gateway);
        assert_eq!(gateway.get_update_tag(), saved);

        gateway.trigger_cooldown();
        assert!(gateway.is_cooling_down());
        assert_eq!(gateway.teleport_cooldown, 40);
        assert_eq!(gateway.cooldown_percent(0.0), 0.0);
        gateway.beam_animation_tick();
        assert_eq!(gateway.age, 201);
        assert_eq!(gateway.teleport_cooldown, 39);
        assert!((gateway.cooldown_percent(0.0) - 0.025).abs() < f32::EPSILON * 4.0);
        assert!(gateway.trigger_event(TheEndGatewayBlockEntity::EVENT_COOLDOWN));
        assert_eq!(gateway.teleport_cooldown, 40);
        assert!(!gateway.trigger_event(99));

        let mut attention = TheEndGatewayBlockEntity {
            age: 2399,
            ..TheEndGatewayBlockEntity::new()
        };
        assert!(attention.portal_tick());
        assert_eq!(attention.age, 2400);
        assert_eq!(attention.teleport_cooldown, 40);
    }

    #[test]
    fn jigsaw_block_entity_saves_priorities_joint_and_generation_plan_like_java() {
        let mut jigsaw = JigsawBlockEntity::new();
        assert_eq!(jigsaw.name, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.target, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.pool, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(jigsaw.final_state, JigsawBlockEntity::DEFAULT_FINAL_STATE);
        assert_eq!(jigsaw.joint, JigsawJointType::Rollable);

        jigsaw.name = "minecraft:house/start".to_string();
        jigsaw.target = "minecraft:house/door".to_string();
        jigsaw.pool = "minecraft:village/plains/houses".to_string();
        jigsaw.final_state = "minecraft:oak_planks".to_string();
        jigsaw.joint = JigsawJointType::Aligned;
        jigsaw.placement_priority = 7;
        jigsaw.selection_priority = -3;

        let saved = jigsaw.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "name".to_string(),
                    Tag::String("minecraft:house/start".to_string())
                ),
                (
                    "target".to_string(),
                    Tag::String("minecraft:house/door".to_string())
                ),
                (
                    "pool".to_string(),
                    Tag::String("minecraft:village/plains/houses".to_string())
                ),
                (
                    "final_state".to_string(),
                    Tag::String("minecraft:oak_planks".to_string())
                ),
                ("joint".to_string(), Tag::String("aligned".to_string())),
                ("placement_priority".to_string(), Tag::Int(7)),
                ("selection_priority".to_string(), Tag::Int(-3)),
            ])
        );
        assert_eq!(JigsawBlockEntity::load_additional(&saved), jigsaw);
        assert_eq!(jigsaw.get_update_tag(), saved);

        let plan =
            jigsaw.generation_plan(BlockPos { x: 4, y: 70, z: 8 }, Direction::North, 5, true);
        assert_eq!(
            plan,
            JigsawGenerationPlan {
                pool: "minecraft:village/plains/houses".to_string(),
                target: "minecraft:house/door".to_string(),
                levels: 5,
                keep_jigsaws: true,
                start_pos: BlockPos { x: 4, y: 70, z: 7 },
            }
        );

        let defaults = JigsawBlockEntity::load_additional(&Tag::Compound(vec![(
            "joint".to_string(),
            Tag::String("unknown".to_string()),
        )]));
        assert_eq!(defaults.name, JigsawBlockEntity::EMPTY_ID);
        assert_eq!(defaults.joint, JigsawJointType::Rollable);
        assert_eq!(defaults.placement_priority, 0);
        assert_eq!(defaults.selection_priority, 0);
    }

    #[test]
    fn comparator_block_entity_persists_output_and_uses_compare_subtract_logic() {
        let mut compare = ComparatorBlockEntity::new(ComparatorMode::Compare);
        assert_eq!(
            compare.save_additional(),
            Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(0))])
        );
        assert!(compare.update_output(12, 7));
        assert_eq!(compare.output_signal, 12);
        assert!(!compare.update_output(12, 7));
        assert_eq!(compare.calculate_output(3, 10), 0);

        let saved = compare.save_additional();
        assert_eq!(
            ComparatorBlockEntity::load_additional(ComparatorMode::Compare, &saved),
            compare
        );
        assert_eq!(
            ComparatorBlockEntity::load_additional(
                ComparatorMode::Subtract,
                &Tag::Compound(vec![("OutputSignal".to_string(), Tag::Int(99))]),
            )
            .output_signal,
            99
        );

        let mut subtract = ComparatorBlockEntity::new(ComparatorMode::Subtract);
        assert!(subtract.update_output(12, 7));
        assert_eq!(subtract.output_signal, 5);
        assert!(subtract.update_output(3, 10));
        assert_eq!(subtract.output_signal, 0);
    }

    #[test]
    fn daylight_detector_updates_power_with_vanilla_solar_math_and_tick_cadence() {
        let mut normal = DaylightDetectorBlockEntity::new(false);
        assert_eq!(normal.save_additional(), Tag::Compound(Vec::new()));
        assert!(!normal.tick(19, 15, 0.0));
        assert_eq!(normal.power, 0);
        assert!(normal.tick(20, 15, 0.0));
        assert_eq!(normal.power, 15);
        assert!(normal.update_signal(15, 180.0));
        assert_eq!(normal.power, 0);
        assert!(!normal.update_signal(-4, 0.0));
        assert_eq!(normal.power, 0);

        assert_eq!(
            DaylightDetectorBlockEntity::calculate_power(false, 10, 90.0),
            3
        );
        assert_eq!(
            DaylightDetectorBlockEntity::calculate_power(false, 99, 0.0),
            15
        );

        let mut inverted = DaylightDetectorBlockEntity::new(true);
        assert!(inverted.update_signal(4, 90.0));
        assert_eq!(inverted.power, 11);
        assert!(inverted.update_signal(99, 0.0));
        assert_eq!(inverted.power, 0);
    }

    #[test]
    fn block_entity_comparator_outputs_cover_boundary_states() {
        assert_eq!(inventory_comparator_output(&[]), 0);
        assert_eq!(inventory_comparator_output(&[None]), 0);
        assert_eq!(
            inventory_comparator_output(&[Some(stack("minecraft:stone", 1))]),
            1
        );
        assert_eq!(
            inventory_comparator_output(&[Some(stack("minecraft:stone", 64))]),
            MAX_SIGNAL
        );

        let mut furnace = AbstractFurnaceBlockEntity::new(FurnaceBlockEntityKind::Furnace);
        assert_eq!(furnace.comparator_output(), 0);
        furnace.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(stack("minecraft:iron_ore", 1)),
            None,
        );
        assert_eq!(furnace.comparator_output(), 1);
        for slot in 0..AbstractFurnaceBlockEntity::SLOT_COUNT {
            furnace.set_item(slot, Some(stack("minecraft:stone", 64)), None);
        }
        assert_eq!(furnace.comparator_output(), MAX_SIGNAL);

        let mut brewing = BrewingStandBlockEntity::new();
        assert_eq!(brewing.comparator_output(), 0);
        brewing.set_item(0, Some(stack("minecraft:potion", 1)));
        assert_eq!(brewing.comparator_output(), 1);
        for slot in 0..BrewingStandBlockEntity::CONTAINER_SIZE {
            brewing.set_item(slot, Some(stack("minecraft:potion", 64)));
        }
        assert_eq!(brewing.comparator_output(), MAX_SIGNAL);

        for kind in [
            ContainerBlockEntityKind::Chest,
            ContainerBlockEntityKind::TrappedChest,
            ContainerBlockEntityKind::Barrel,
            ContainerBlockEntityKind::ShulkerBox,
            ContainerBlockEntityKind::Dispenser,
            ContainerBlockEntityKind::Dropper,
            ContainerBlockEntityKind::Hopper,
        ] {
            let mut container = ContainerBlockEntityModel::new(kind);
            assert_eq!(container.comparator_output(), 0, "{kind:?} empty");
            container.set_item(0, Some(stack("minecraft:stone", 1)));
            assert!(container.comparator_output() > 0, "{kind:?} partial");
            for slot in 0..container.items.len() {
                container.set_item(slot, Some(stack("minecraft:stone", 64)));
            }
            assert_eq!(container.comparator_output(), MAX_SIGNAL, "{kind:?} full");
        }

        let mut trapped = ContainerBlockEntityModel::new(ContainerBlockEntityKind::TrappedChest);
        trapped.viewer_count = 0;
        assert_eq!(trapped.trapped_chest_signal(), 0);
        trapped.viewer_count = i32::from(MAX_SIGNAL);
        assert_eq!(trapped.trapped_chest_signal(), MAX_SIGNAL);
        trapped.viewer_count = i32::from(MAX_SIGNAL) + 1;
        assert_eq!(trapped.trapped_chest_signal(), MAX_SIGNAL);

        let mut jukebox = JukeboxBlockEntity::new();
        assert_eq!(jukebox.comparator_output(), 0);
        assert_eq!(jukebox.redstone_signal(), 0);
        jukebox.set_the_item(Some(stack("minecraft:music_disc_13", 1)));
        assert_eq!(jukebox.comparator_output(), 1);
        assert_eq!(jukebox.redstone_signal(), MAX_SIGNAL);
        jukebox.set_the_item(Some(stack("minecraft:music_disc_5", 1)));
        assert_eq!(jukebox.comparator_output(), MAX_SIGNAL);

        let mut shelf = ShelfBlockEntity::new();
        assert_eq!(shelf.comparator_output(), 0);
        shelf.set_item_no_update(0, Some(stack("minecraft:book", 1)));
        assert_eq!(shelf.comparator_output(), 1);
        for slot in 0..ShelfBlockEntity::MAX_ITEMS {
            shelf.set_item_no_update(slot, Some(stack("minecraft:book", 1)));
        }
        assert_eq!(shelf.comparator_output(), 3);

        let mut beacon = BeaconBlockEntity::new();
        beacon.levels = -1;
        assert_eq!(beacon.comparator_output(), 0);
        beacon.levels = BeaconBlockEntity::MAX_LEVELS;
        assert_eq!(beacon.comparator_output(), 4);
        beacon.levels = 99;
        assert_eq!(beacon.comparator_output(), 4);

        let mut lectern = LecternBlockEntity::new();
        assert_eq!(lectern.get_redstone_signal(), 0);
        lectern.set_book(Some(stack("minecraft:written_book", 1)), 4);
        assert_eq!(lectern.get_redstone_signal(), 1);
        lectern.set_page(3);
        assert_eq!(lectern.get_redstone_signal(), MAX_SIGNAL);

        let mut crafter = CrafterBlockEntity::new();
        assert_eq!(crafter.redstone_signal(), 0);
        for slot in 0..CrafterBlockEntity::CONTAINER_SIZE {
            crafter.set_slot_state(slot, false);
        }
        assert_eq!(crafter.redstone_signal(), 9);

        let mut pot = DecoratedPotBlockEntity::default();
        assert_eq!(pot.comparator_output(), 0);
        pot.item = Some(stack("minecraft:diamond", 1));
        assert_eq!(pot.comparator_output(), 1);
        pot.item = Some(stack("minecraft:diamond", 64));
        assert_eq!(pot.comparator_output(), MAX_SIGNAL);

        let mut statue = CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:copper_golem_statue",
            CopperGolemStatuePose::Standing,
        )
        .unwrap();
        assert_eq!(statue.comparator_output(), 1);
        statue.update_pose();
        assert_eq!(statue.comparator_output(), 2);
        statue.update_pose();
        assert_eq!(statue.comparator_output(), 3);
        statue.update_pose();
        assert_eq!(statue.comparator_output(), 4);

        let mut heart = CreakingHeartBlockEntity::new();
        assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 0);
        heart.set_creaking_uuid("protector".to_string());
        assert_eq!(heart.compute_analog_output_signal(None), 0);
        assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 15);
        assert_eq!(
            heart.compute_analog_output_signal(Some(f64::from(
                CreakingHeartBlockEntity::CREAKING_ROAMING_RADIUS
            ))),
            0
        );
    }

    #[test]
    fn gui_block_entities_open_with_menu_id_initial_slots_and_close_state() {
        let mut furnace = AbstractFurnaceBlockEntity::furnace();
        furnace.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(stack("minecraft:iron_ore", 3)),
            None,
        );
        assert_menu(
            &furnace.open_menu(1),
            1,
            "furnace",
            &[Some(stack("minecraft:iron_ore", 3)), None, None],
        );

        let mut blast = AbstractFurnaceBlockEntity::blast_furnace();
        blast.set_item(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            Some(stack("minecraft:coal", 2)),
            None,
        );
        assert_menu(
            &blast.open_menu(2),
            2,
            "blast_furnace",
            &[None, Some(stack("minecraft:coal", 2)), None],
        );
        assert_menu(
            &AbstractFurnaceBlockEntity::smoker().open_menu(3),
            3,
            "smoker",
            &[None, None, None],
        );

        let ender_slots = vec![Some(stack("minecraft:ender_pearl", 16)); 27];
        let ender_menu = open_ender_chest_menu(4, ender_slots.clone());
        assert_menu(&ender_menu, 4, "generic_9x3", &ender_slots);
        assert_eq!(ender_menu.close(), BlockEntityMenuClose { container_id: 4 });

        for (index, (kind, menu_type, slot_count)) in [
            (ContainerBlockEntityKind::Chest, "generic_9x3", 27),
            (ContainerBlockEntityKind::TrappedChest, "generic_9x3", 27),
            (ContainerBlockEntityKind::Barrel, "generic_9x3", 27),
            (ContainerBlockEntityKind::ShulkerBox, "shulker_box", 27),
            (ContainerBlockEntityKind::Dispenser, "generic_3x3", 9),
            (ContainerBlockEntityKind::Dropper, "generic_3x3", 9),
            (ContainerBlockEntityKind::Hopper, "hopper", 5),
        ]
        .into_iter()
        .enumerate()
        {
            let mut container = ContainerBlockEntityModel::new(kind);
            container.lock_key = Some("key".to_string());
            assert_eq!(container.open_menu(20 + index as i32, None, false), None);
            container.set_item(0, Some(stack("minecraft:apple", 5)));

            let menu = container
                .open_menu(20 + index as i32, Some("key"), false)
                .unwrap();
            let mut expected = vec![None; slot_count];
            expected[0] = Some(stack("minecraft:apple", 5));
            assert_menu(&menu, 20 + index as i32, menu_type, &expected);
            assert_eq!(container.viewer_count, 1);
            let close = container.close_menu(menu);
            assert_eq!(
                close,
                BlockEntityMenuClose {
                    container_id: 20 + index as i32
                }
            );
            assert_eq!(container.viewer_count, 0);
        }

        let enchantment = EnchantingTableBlockEntity::new();
        assert_menu(&enchantment.open_menu(40), 40, "enchantment", &[None, None]);

        let mut brewing = BrewingStandBlockEntity::new();
        brewing.set_item(0, Some(stack("minecraft:potion", 1)));
        assert_menu(
            &brewing.open_menu(41),
            41,
            "brewing_stand",
            &[Some(stack("minecraft:potion", 1)), None, None, None, None],
        );

        let mut beacon = BeaconBlockEntity::new();
        assert!(beacon.set_payment_item(Some(stack("minecraft:emerald", 1))));
        assert_menu(
            &beacon.open_menu(42),
            42,
            "beacon",
            &[Some(stack("minecraft:emerald", 1))],
        );

        let mut lectern = LecternBlockEntity::new();
        lectern.set_book(Some(stack("minecraft:written_book", 1)), 3);
        assert_menu(
            &lectern.open_menu(43),
            43,
            "lectern",
            &[Some(stack("minecraft:written_book", 1))],
        );

        let mut crafter = CrafterBlockEntity::new();
        crafter.set_item(8, Some(stack("minecraft:redstone", 4)));
        let mut crafter_slots = vec![None; CrafterBlockEntity::CONTAINER_SIZE + 1];
        crafter_slots[8] = Some(stack("minecraft:redstone", 4));
        assert_menu(&crafter.open_menu(44), 44, "crafter_3x3", &crafter_slots);
    }

    #[test]
    fn command_block_entity_persists_base_fields_and_models_execution_gate() {
        let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, true);
        command.set_command("say hello");
        command.custom_name = Some("\"Runner\"".to_string());
        command.track_output = true;
        command.last_output = Some("\"previous\"".to_string());
        command.last_execution = 41;
        command.powered = true;
        command.automatic = true;
        command.mark_condition_met(true);

        let saved = command.save_additional();
        assert_eq!(
            CommandBlockEntity::load_additional(CommandBlockMode::Redstone, true, &saved),
            command
        );
        assert_eq!(
            command.execution_action(true, true),
            SpecialBlockAction::ExecuteCommand { success_count: 1 }
        );
        assert_eq!(command.perform_command(42, true, true, true), true);
        assert_eq!(command.success_count, 1);
        assert_eq!(command.last_execution, 42);
        assert_eq!(command.perform_command(42, true, true, true), false);

        let mut blocked = CommandBlockEntity::new(CommandBlockMode::Auto, false);
        blocked.set_command("say no");
        assert_eq!(
            blocked.execution_action(false, true),
            SpecialBlockAction::ExecuteCommand { success_count: 0 }
        );
        assert!(blocked.set_automatic(true, true));
        assert!(!blocked.set_automatic(true, true));
        assert!(blocked.can_use(true));
        assert!(!blocked.can_use(false));

        let mut searge = CommandBlockEntity::new(CommandBlockMode::Auto, false);
        searge.set_command("Searge");
        assert!(searge.perform_command(9, false, false, false));
        assert_eq!(searge.success_count, 1);
        assert_eq!(
            searge.last_output.as_deref(),
            Some(CommandBlockEntity::SEARGE_OUTPUT)
        );

        let loaded_without_tracking = CommandBlockEntity::load_additional(
            CommandBlockMode::Auto,
            false,
            &Tag::Compound(vec![
                ("TrackOutput".to_string(), Tag::Byte(0)),
                (
                    "LastOutput".to_string(),
                    Tag::String("\"ignored\"".to_string()),
                ),
                ("UpdateLastExecution".to_string(), Tag::Byte(0)),
                ("LastExecution".to_string(), Tag::Long(99)),
            ]),
        );
        assert_eq!(loaded_without_tracking.last_output, None);
        assert_eq!(
            loaded_without_tracking.last_execution,
            CommandBlockEntity::NO_LAST_EXECUTION
        );
    }

    #[test]
    fn command_block_execution_uses_block_source_and_captures_output() {
        let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
        command.set_command("say hello");
        command.powered = true;

        let execution = command
            .execute_from_context(
                CommandBlockExecutionContext {
                    pos: BlockPos { x: 4, y: 64, z: -2 },
                    level: "minecraft:overworld".to_string(),
                    game_time: 100,
                    command_blocks_enabled: true,
                    has_permission: true,
                    previous_success: true,
                },
                Some("{\"text\":\"hello\"}".to_string()),
            )
            .expect("powered redstone command block should execute on the leading edge");

        assert_eq!(execution.command, "say hello");
        assert_eq!(execution.success_count, 1);
        assert_eq!(execution.output.as_deref(), Some("{\"text\":\"hello\"}"));
        assert_eq!(command.last_output.as_deref(), Some("{\"text\":\"hello\"}"));
        assert_eq!(command.last_execution, 100);
        assert_eq!(execution.source.source, "CommandBlockEntity");
        assert_eq!(execution.source.level, "minecraft:overworld");
        assert_eq!(execution.source.permission_level, 2);
        assert_eq!(
            execution.source.position,
            Vec3 {
                x: 4.5,
                y: 64.5,
                z: -1.5,
            }
        );

        assert!(
            command
                .execute_from_context(
                    CommandBlockExecutionContext {
                        pos: BlockPos { x: 4, y: 64, z: -2 },
                        level: "minecraft:overworld".to_string(),
                        game_time: 100,
                        command_blocks_enabled: true,
                        has_permission: true,
                        previous_success: true,
                    },
                    Some("{\"text\":\"again\"}".to_string()),
                )
                .is_none(),
            "command block should not run twice in the same game tick"
        );

        let mut denied = CommandBlockEntity::new(CommandBlockMode::Auto, false);
        denied.set_command("say denied");
        let denied_execution = denied
            .execute_from_context(
                CommandBlockExecutionContext {
                    pos: BlockPos { x: 0, y: 70, z: 0 },
                    level: "minecraft:overworld".to_string(),
                    game_time: 101,
                    command_blocks_enabled: true,
                    has_permission: false,
                    previous_success: true,
                },
                Some("{\"text\":\"denied\"}".to_string()),
            )
            .expect("permission-denied command block records a zero-success execution");
        assert_eq!(denied_execution.success_count, 0);
        assert_eq!(denied.success_count, 0);
        assert_eq!(denied.last_output.as_deref(), Some("{\"text\":\"denied\"}"));
    }

    #[test]
    fn command_block_editor_packet_and_client_update_require_permission() {
        let mut command = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
        command.set_command("say old");
        command.last_output = Some("{\"text\":\"old\"}".to_string());

        assert_eq!(command.open_editor_packet(pos(), false), None);
        let packet = command
            .open_editor_packet(pos(), true)
            .expect("operators can open command block editor data");
        assert_eq!(packet.pos, pos());
        assert_eq!(packet.ty, BlockEntityTypeId::CommandBlock);
        let packet_entries = compound_entries(&packet.tag).unwrap();
        assert_eq!(
            get_string(packet_entries, "Command"),
            Some("say old"),
            "editor packet carries the current command string"
        );
        assert_eq!(
            get_string(packet_entries, "LastOutput"),
            Some("{\"text\":\"old\"}")
        );

        assert!(!command.apply_client_update(
            CommandBlockUpdate {
                command: "say denied".to_string(),
                mode: CommandBlockMode::Auto,
                track_output: false,
                conditional: true,
                automatic: true,
            },
            false,
            true,
        ));
        assert_eq!(command.command, "say old");

        assert!(command.apply_client_update(
            CommandBlockUpdate {
                command: "say new".to_string(),
                mode: CommandBlockMode::Auto,
                track_output: false,
                conditional: true,
                automatic: true,
            },
            true,
            true,
        ));
        assert_eq!(command.command, "say new");
        assert_eq!(command.mode, CommandBlockMode::Auto);
        assert!(command.conditional);
        assert!(command.automatic);
        assert!(!command.track_output);
        assert_eq!(command.last_output, None);
    }

    #[test]
    fn command_block_chain_executes_facing_order_and_respects_conditional_flag() {
        let mut root = CommandBlockEntity::new(CommandBlockMode::Redstone, false);
        root.set_command("say root");
        root.powered = true;

        let mut chain_one = CommandBlockEntity::new(CommandBlockMode::Sequence, false);
        chain_one.set_command("say first");
        chain_one.powered = true;

        let mut chain_two = CommandBlockEntity::new(CommandBlockMode::Sequence, true);
        chain_two.set_command("say second");
        chain_two.powered = true;

        let mut entries = vec![
            CommandBlockChainEntry {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                facing: Direction::East,
                block: root,
            },
            CommandBlockChainEntry {
                pos: BlockPos { x: 1, y: 64, z: 0 },
                facing: Direction::East,
                block: chain_one,
            },
            CommandBlockChainEntry {
                pos: BlockPos { x: 2, y: 64, z: 0 },
                facing: Direction::East,
                block: chain_two,
            },
        ];

        let steps = execute_command_block_chain(
            &mut entries,
            BlockPos { x: 0, y: 64, z: 0 },
            CommandBlockExecutionContext {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                level: "minecraft:overworld".to_string(),
                game_time: 200,
                command_blocks_enabled: true,
                has_permission: true,
                previous_success: true,
            },
        );
        assert_eq!(
            steps,
            vec![
                CommandBlockChainStep {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    command: "say root".to_string(),
                    success_count: 1,
                },
                CommandBlockChainStep {
                    pos: BlockPos { x: 1, y: 64, z: 0 },
                    command: "say first".to_string(),
                    success_count: 1,
                },
                CommandBlockChainStep {
                    pos: BlockPos { x: 2, y: 64, z: 0 },
                    command: "say second".to_string(),
                    success_count: 1,
                },
            ]
        );

        entries[1].block.command.clear();
        entries[0].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
        entries[1].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
        entries[2].block.last_execution = CommandBlockEntity::NO_LAST_EXECUTION;
        let conditional_steps = execute_command_block_chain(
            &mut entries,
            BlockPos { x: 0, y: 64, z: 0 },
            CommandBlockExecutionContext {
                pos: BlockPos { x: 0, y: 64, z: 0 },
                level: "minecraft:overworld".to_string(),
                game_time: 201,
                command_blocks_enabled: true,
                has_permission: true,
                previous_success: true,
            },
        );
        assert_eq!(
            conditional_steps
                .iter()
                .map(|step| step.success_count)
                .collect::<Vec<_>>(),
            vec![1, 0, 0]
        );
    }

    #[test]
    fn jukebox_block_entity_tracks_disc_playback_ticks_and_outputs() {
        let mut jukebox = JukeboxBlockEntity::new();
        let disc = PotItemStack {
            item_id: "minecraft:music_disc_13".to_string(),
            count: 1,
        };
        assert!(jukebox.can_place_item(&disc));
        assert_eq!(
            jukebox.set_the_item(Some(disc.clone())),
            JukeboxSongEvent::Started
        );
        assert!(jukebox.is_playing);
        assert_eq!(jukebox.redstone_signal(), 15);
        assert_eq!(jukebox.comparator_output(), 1);
        assert!(jukebox.tick());
        assert_eq!(jukebox.ticks_since_song_started, 1);

        let saved = jukebox.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("RecordItem".to_string(), disc.to_tag()),
                ("ticks_since_song_started".to_string(), Tag::Long(1)),
            ])
        );
        let loaded = JukeboxBlockEntity::load_additional(&saved);
        assert_eq!(loaded.item, Some(disc.clone()));
        assert_eq!(loaded.ticks_since_song_started, 1);
        assert!(!loaded.is_playing);
        assert_eq!(loaded.comparator_output(), 1);

        assert!(!jukebox.can_place_item(&PotItemStack {
            item_id: "minecraft:music_disc_5".to_string(),
            count: 1,
        }));
        assert_eq!(jukebox.pop_out_the_item(), Some(disc));
        assert!(!jukebox.is_playing);
        assert_eq!(jukebox.redstone_signal(), 0);
        assert_eq!(jukebox.comparator_output(), 0);
        assert_eq!(jukebox.save_additional(), Tag::Compound(Vec::new()));

        let mut inert = JukeboxBlockEntity::new();
        assert_eq!(
            inert.set_the_item(Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 1,
            })),
            JukeboxSongEvent::Stopped
        );
        assert!(!inert.tick());
        assert_eq!(inert.comparator_output(), 0);
        assert!(!inert.can_place_item(&PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 1,
        }));
        assert!(inert.can_take_item(true));
        assert!(!inert.can_take_item(false));

        let mut without_playing = JukeboxBlockEntity::new();
        assert_eq!(
            without_playing.set_song_item_without_playing(PotItemStack {
                item_id: "minecraft:music_disc_5".to_string(),
                count: 1,
            }),
            JukeboxSongEvent::ItemChanged
        );
        assert!(!without_playing.is_playing);
        assert_eq!(without_playing.redstone_signal(), 0);
        assert_eq!(without_playing.comparator_output(), 15);
    }

    #[test]
    fn enchanting_table_saves_name_scans_bookshelves_and_animates_book_like_java() {
        let mut table = EnchantingTableBlockEntity::new();
        assert_eq!(
            table.display_name(),
            EnchantingTableBlockEntity::DEFAULT_NAME
        );
        table.custom_name = Some("\"Arcana\"".to_string());
        let saved = table.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![(
                "CustomName".to_string(),
                Tag::String("\"Arcana\"".to_string())
            )])
        );
        assert_eq!(
            EnchantingTableBlockEntity::load_additional(&saved).custom_name,
            Some("\"Arcana\"".to_string())
        );
        assert_eq!(table.get_update_tag(), saved);

        let offsets = EnchantingTableBlockEntity::bookshelf_offsets();
        assert_eq!(offsets.len(), 32);
        assert!(offsets.contains(&BlockPos { x: -2, y: 0, z: 0 }));
        assert!(offsets.contains(&BlockPos { x: 2, y: 1, z: 2 }));
        let power = |pos: BlockPos| pos.x == 2 || pos.z == -2;
        let transmit = |_pos: BlockPos| true;
        assert_eq!(
            EnchantingTableBlockEntity::count_valid_bookshelves(power, transmit),
            15
        );

        table.book_animation_tick(Some((1.0, 0.0)), Some(2.0));
        assert_eq!(table.time, 1);
        assert_eq!(table.o_open, 0.0);
        assert_eq!(table.open, 0.1);
        assert_eq!(table.t_rot, 0.0);
        assert!(table.flip > 0.0);
        let previous_flip = table.flip;
        table.book_animation_tick(None, None);
        assert_eq!(table.time, 2);
        assert_eq!(table.o_open, 0.1);
        assert_eq!(table.open, 0.0);
        assert!(table.t_rot > 0.0);
        assert!(table.flip >= previous_flip);

        table.rot = std::f32::consts::TAU;
        table.t_rot = -std::f32::consts::TAU;
        table.book_animation_tick(None, None);
        assert!(table.rot < std::f32::consts::PI);
        assert!(table.t_rot > -std::f32::consts::PI);
    }

    #[test]
    fn shelf_block_entity_saves_three_items_align_flag_and_swaps_slots() {
        let mut shelf = ShelfBlockEntity::new();
        assert_eq!(shelf.items.len(), ShelfBlockEntity::MAX_ITEMS);
        assert_eq!(shelf.comparator_output(), 0);
        assert!(shelf.set_item_no_update(
            0,
            Some(PotItemStack {
                item_id: "minecraft:book".to_string(),
                count: 1,
            })
        ));
        assert!(shelf.set_item_no_update(
            2,
            Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 3,
            })
        ));
        assert!(!shelf.set_item_no_update(
            3,
            Some(PotItemStack {
                item_id: "minecraft:apple".to_string(),
                count: 1,
            })
        ));
        shelf.align_items_to_bottom = true;
        assert_eq!(shelf.filled_slot_count(), 2);
        assert_eq!(shelf.comparator_output(), 2);

        let saved = shelf.save_additional();
        assert_eq!(ShelfBlockEntity::load_additional(&saved), shelf);
        assert_eq!(shelf.get_update_tag(), saved);
        assert_eq!(
            shelf.swap_item_no_update(
                0,
                Some(PotItemStack {
                    item_id: "minecraft:stick".to_string(),
                    count: 4,
                })
            ),
            Some(PotItemStack {
                item_id: "minecraft:book".to_string(),
                count: 1,
            })
        );
        assert_eq!(
            shelf.get_item(0),
            Some(&PotItemStack {
                item_id: "minecraft:stick".to_string(),
                count: 4,
            })
        );
        assert_eq!(
            shelf.remove_item_no_update(2),
            Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 3,
            })
        );
        assert_eq!(shelf.comparator_output(), 1);

        let out_of_range = Tag::Compound(vec![(
            "Items".to_string(),
            Tag::List(vec![Tag::Compound(vec![
                ("id".to_string(), Tag::String("minecraft:apple".to_string())),
                ("count".to_string(), Tag::Int(1)),
                ("Slot".to_string(), Tag::Byte(9)),
            ])]),
        )]);
        assert_eq!(
            ShelfBlockEntity::load_additional(&out_of_range),
            ShelfBlockEntity::new()
        );
    }

    #[test]
    fn beacon_tier_effect_payment_and_beam_state_match_vanilla_rules() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let full_four_tier = |block: BlockPos| {
            let dy = pos.y - block.y;
            (1..=4).contains(&dy) && block.x.abs() <= dy && block.z.abs() <= dy
        };
        assert_eq!(BeaconBlockEntity::update_base(pos, 0, full_four_tier), 4);
        let broken_second_tier = |block: BlockPos| {
            let dy = pos.y - block.y;
            (1..=4).contains(&dy)
                && block.x.abs() <= dy
                && block.z.abs() <= dy
                && !(dy == 2 && block.x == 2 && block.z == 0)
        };
        assert_eq!(
            BeaconBlockEntity::update_base(pos, 0, broken_second_tier),
            1
        );
        assert_eq!(
            BeaconBlockEntity::update_base(BlockPos { x: 0, y: 1, z: 0 }, 1, |_| true),
            0
        );

        let mut beacon = BeaconBlockEntity::new();
        beacon.levels = 4;
        beacon.set_primary_power(Some("minecraft:speed"));
        beacon.set_secondary_power(Some("minecraft:speed"));
        assert_eq!(beacon.comparator_output(), 4);
        assert_eq!(
            beacon.effect_applications(),
            vec![BeaconEffectApplication {
                effect: "minecraft:speed".to_string(),
                duration_ticks: 340,
                amplifier: 1,
                range: 50,
            }]
        );
        beacon.set_secondary_power(Some("minecraft:regeneration"));
        assert_eq!(
            beacon.effect_applications(),
            vec![
                BeaconEffectApplication {
                    effect: "minecraft:speed".to_string(),
                    duration_ticks: 340,
                    amplifier: 0,
                    range: 50,
                },
                BeaconEffectApplication {
                    effect: "minecraft:regeneration".to_string(),
                    duration_ticks: 340,
                    amplifier: 0,
                    range: 50,
                },
            ]
        );

        beacon.set_primary_power(Some("minecraft:night_vision"));
        assert_eq!(beacon.primary_power, None);
        assert!(BeaconBlockEntity::can_pay_with(&PotItemStack {
            item_id: "minecraft:amethyst_shard".to_string(),
            count: 1,
        }));
        assert!(!beacon.set_payment_item(Some(PotItemStack {
            item_id: "minecraft:apple".to_string(),
            count: 1,
        })));
        assert!(beacon.set_payment_item(Some(PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 1,
        })));

        beacon.primary_power = Some("minecraft:haste".to_string());
        beacon.secondary_power = Some("minecraft:regeneration".to_string());
        beacon.custom_name = Some("\"Beacon\"".to_string());
        beacon.lock_key = Some("secret".to_string());
        let saved = beacon.save_additional();
        let loaded = BeaconBlockEntity::load_additional(&saved);
        assert_eq!(loaded.levels, 4);
        assert_eq!(loaded.primary_power.as_deref(), Some("minecraft:haste"));
        assert_eq!(
            loaded.secondary_power.as_deref(),
            Some("minecraft:regeneration")
        );
        assert_eq!(loaded.display_name(), "\"Beacon\"");
        assert_eq!(loaded.lock_key.as_deref(), Some("secret"));
        assert_eq!(loaded.get_update_tag(), loaded.save_additional());

        let sections = BeaconBlockEntity::scan_beam([
            BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
            BeaconBeamBlock::TintedGlass(0xFFFF_0000u32 as i32),
            BeaconBeamBlock::TintedGlass(0xFF00_00FFu32 as i32),
            BeaconBeamBlock::Transparent,
        ]);
        assert_eq!(
            sections,
            vec![
                BeaconBeamSection {
                    color: 0xFFFF_0000u32 as i32,
                    height: 1,
                },
                BeaconBeamSection {
                    color: 0xFFFF_0000u32 as i32,
                    height: 1,
                },
                BeaconBeamSection {
                    color: 0xFF7F_007Fu32 as i32,
                    height: 2,
                },
            ]
        );
        assert_eq!(
            BeaconBlockEntity::scan_beam([BeaconBeamBlock::Blocking]),
            Vec::<BeaconBeamSection>::new()
        );
    }

    #[test]
    fn lectern_block_entity_tracks_book_pages_and_comparator_signal() {
        let mut lectern = LecternBlockEntity::new();
        assert_eq!(lectern.save_additional(), Tag::Compound(Vec::new()));
        assert!(!lectern.has_book());
        assert_eq!(lectern.get_redstone_signal(), 0);

        let book = PotItemStack {
            item_id: "minecraft:written_book".to_string(),
            count: 1,
        };
        lectern.set_book(Some(book.clone()), 5);
        assert!(lectern.has_book());
        assert_eq!(lectern.page, 0);
        assert_eq!(lectern.page_count, 5);
        assert_eq!(lectern.get_redstone_signal(), 1);
        assert!(lectern.set_page(2));
        assert_eq!(lectern.get_redstone_signal(), 8);
        assert!(lectern.set_page(99));
        assert_eq!(lectern.page, 4);
        assert_eq!(lectern.get_redstone_signal(), 15);
        assert!(!lectern.set_page(4));

        let saved = lectern.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("Book".to_string(), book.to_tag()),
                ("Page".to_string(), Tag::Int(4)),
            ])
        );
        assert_eq!(LecternBlockEntity::load_additional(&saved, 5), lectern);
        assert_eq!(
            LecternBlockEntity::load_additional(
                &Tag::Compound(vec![
                    ("Book".to_string(), book.to_tag()),
                    ("Page".to_string(), Tag::Int(-3)),
                ]),
                5,
            )
            .page,
            0
        );

        assert_eq!(lectern.remove_book_no_update(), Some(book));
        assert!(!lectern.has_book());
        assert_eq!(lectern.page, 0);
        assert_eq!(lectern.page_count, 0);
        assert_eq!(lectern.get_redstone_signal(), 0);

        let mut single_page = LecternBlockEntity::new();
        single_page.set_book(
            Some(PotItemStack {
                item_id: "minecraft:writable_book".to_string(),
                count: 1,
            }),
            1,
        );
        assert_eq!(single_page.get_redstone_signal(), 15);
        single_page.clear_content();
        assert_eq!(single_page.save_additional(), Tag::Compound(Vec::new()));
    }

    #[test]
    fn sign_block_entities_track_front_back_text_filtering_wax_and_hanging_shape() {
        let mut sign = SignBlockEntityModel::default();
        assert_eq!(SignBlockEntityModel::MAX_TEXT_LINE_WIDTH, 90);
        assert_eq!(SignBlockEntityModel::TEXT_LINE_HEIGHT, 10);
        assert_eq!(sign.front_text.color, DyeColor::Black);
        assert!(!sign.front_text.has_message(false));
        assert!(!sign.update_sign_text(
            "player-a",
            true,
            std::array::from_fn(|_| SignLine::default()),
            false,
        ));

        sign.set_allowed_player_editor(Some("player-a".to_string()));
        assert!(!sign.player_is_too_far_away_to_edit("player-a", 4.0));
        let front_lines = [
            SignLine::new("raw one", "filtered one").with_click_command("/say front"),
            SignLine::new("raw two", "filtered two"),
            SignLine::new("", ""),
            SignLine::new("raw four", "filtered four"),
        ];
        assert!(sign.update_sign_text("player-a", true, front_lines, false));
        assert!(sign.player_who_may_edit.is_none());
        sign.front_text.color = DyeColor::Blue;
        sign.front_text.has_glowing_text = true;
        assert_eq!(sign.front_text.lines[0].visible_text(false), "raw one");
        assert_eq!(sign.front_text.lines[0].visible_text(true), "filtered one");
        assert!(!sign.can_execute_click_commands(true, false));
        assert!(sign.set_waxed(true));
        assert!(sign.can_execute_click_commands(true, false));
        assert_eq!(
            sign.executable_click_commands(true, false),
            vec!["/say front".to_string()]
        );

        sign.set_allowed_player_editor(Some("player-b".to_string()));
        let filtered_back_lines = [
            SignLine::new("unsafe", "safe"),
            SignLine::new("raw", "clean"),
            SignLine::new("", ""),
            SignLine::new("last", "filtered last"),
        ];
        assert!(!sign.update_sign_text("player-b", false, filtered_back_lines.clone(), true));
        assert_eq!(sign.back_text.lines[0].raw, "");
        assert!(sign.set_waxed(false));
        assert!(sign.update_sign_text("player-b", false, filtered_back_lines, true));
        assert_eq!(sign.back_text.lines[0].raw, "safe");
        assert_eq!(sign.back_text.lines[0].filtered, "safe");

        sign.set_allowed_player_editor(Some("player-c".to_string()));
        assert!(sign.tick_editing_player("player-c", 4.01));
        assert!(sign.player_who_may_edit.is_none());

        let saved = sign.save_additional();
        let loaded = SignBlockEntityModel::load_additional(&saved);
        assert_eq!(loaded.front_text.color, DyeColor::Blue);
        assert!(loaded.front_text.has_glowing_text);
        assert_eq!(loaded.front_text.lines[0].raw, "raw one");
        assert_eq!(loaded.front_text.lines[0].filtered, "filtered one");
        assert_eq!(
            loaded.front_text.lines[0].click_command.as_deref(),
            Some("/say front")
        );
        assert_eq!(loaded.back_text.lines[3].raw, "filtered last");
        assert!(!loaded.is_waxed);

        let mut hanging = HangingSignBlockEntityModel::new(HangingSignAttachment::CeilingMiddle);
        hanging.sign = loaded;
        assert_eq!(HangingSignBlockEntityModel::MAX_TEXT_LINE_WIDTH, 60);
        assert_eq!(HangingSignBlockEntityModel::TEXT_LINE_HEIGHT, 9);
        let loaded_hanging =
            HangingSignBlockEntityModel::load_additional(&hanging.save_additional());
        assert_eq!(
            loaded_hanging.attachment,
            HangingSignAttachment::CeilingMiddle
        );
        assert_eq!(loaded_hanging.sign.front_text.lines[0].raw, "raw one");
    }

    #[test]
    fn brewing_stand_ticks_fuel_recipes_sided_slots_and_save_load_like_java() {
        let recipes = [
            BrewingRecipe::new(
                "minecraft:potion",
                "water",
                "minecraft:nether_wart",
                "minecraft:potion",
                "awkward",
            ),
            BrewingRecipe::new(
                "minecraft:potion",
                "awkward",
                "minecraft:blaze_powder",
                "minecraft:potion",
                "strength",
            ),
            BrewingRecipe::new(
                "minecraft:potion",
                "awkward",
                "minecraft:gunpowder",
                "minecraft:splash_potion",
                "awkward",
            ),
        ];
        let mut stand = BrewingStandBlockEntity::new();
        assert_eq!(stand.items.len(), BrewingStandBlockEntity::CONTAINER_SIZE);
        assert_eq!(stand.potion_bits(), [false, false, false]);
        assert_eq!(
            BrewingStandBlockEntity::slots_for_face(Direction::Up),
            &[BrewingStandBlockEntity::INGREDIENT_SLOT]
        );
        assert_eq!(
            BrewingStandBlockEntity::slots_for_face(Direction::Down),
            &[0, 1, 2, BrewingStandBlockEntity::INGREDIENT_SLOT]
        );
        assert_eq!(
            BrewingStandBlockEntity::slots_for_face(Direction::North),
            &[0, 1, 2, BrewingStandBlockEntity::FUEL_SLOT]
        );

        let water = PotItemStack {
            item_id: brewing_stack_id("minecraft:potion", "water"),
            count: 1,
        };
        let nether_wart = PotItemStack {
            item_id: "minecraft:nether_wart".to_string(),
            count: 1,
        };
        let blaze_powder = PotItemStack {
            item_id: "minecraft:blaze_powder".to_string(),
            count: 2,
        };
        assert!(stand.can_place_item(0, &water, &recipes));
        assert!(stand.can_place_item(
            BrewingStandBlockEntity::INGREDIENT_SLOT,
            &nether_wart,
            &recipes
        ));
        assert!(stand.can_place_item(BrewingStandBlockEntity::FUEL_SLOT, &blaze_powder, &recipes));
        assert!(!BrewingStandBlockEntity::can_take_item_through_face(
            BrewingStandBlockEntity::INGREDIENT_SLOT,
            &nether_wart,
            Direction::Down
        ));
        assert!(BrewingStandBlockEntity::can_take_item_through_face(
            BrewingStandBlockEntity::INGREDIENT_SLOT,
            &PotItemStack {
                item_id: "minecraft:glass_bottle".to_string(),
                count: 1,
            },
            Direction::Down
        ));

        stand.set_item(0, Some(water.clone()));
        stand.set_item(1, Some(water));
        stand.set_item(BrewingStandBlockEntity::INGREDIENT_SLOT, Some(nether_wart));
        stand.set_item(BrewingStandBlockEntity::FUEL_SLOT, Some(blaze_powder));
        assert_eq!(stand.potion_bits(), [true, true, false]);
        assert_eq!(
            stand.server_tick(&recipes),
            BrewingStandTickResult::FuelLoaded
        );
        assert_eq!(stand.fuel, BrewingStandBlockEntity::FUEL_USES);
        assert_eq!(
            stand.items[BrewingStandBlockEntity::FUEL_SLOT]
                .as_ref()
                .map(|stack| stack.count),
            Some(1)
        );
        assert_eq!(stand.server_tick(&recipes), BrewingStandTickResult::Started);
        assert_eq!(stand.fuel, BrewingStandBlockEntity::FUEL_USES - 1);
        assert_eq!(stand.brew_time, BrewingStandBlockEntity::BREW_TIME);
        assert_eq!(stand.ingredient.as_deref(), Some("minecraft:nether_wart"));
        for _ in 1..BrewingStandBlockEntity::BREW_TIME {
            assert_eq!(stand.server_tick(&recipes), BrewingStandTickResult::Brewing);
        }
        assert_eq!(stand.server_tick(&recipes), BrewingStandTickResult::Brewed);
        assert_eq!(stand.items[BrewingStandBlockEntity::INGREDIENT_SLOT], None);
        assert_eq!(
            stand.items[0].as_ref().map(|stack| stack.item_id.as_str()),
            Some("minecraft:potion#awkward")
        );
        assert_eq!(
            stand.items[1].as_ref().map(|stack| stack.item_id.as_str()),
            Some("minecraft:potion#awkward")
        );

        let saved = stand.save_additional();
        assert_eq!(BrewingStandBlockEntity::load_additional(&saved), stand);

        let mut cancelled = BrewingStandBlockEntity::new();
        cancelled.set_item(
            0,
            Some(PotItemStack {
                item_id: brewing_stack_id("minecraft:potion", "awkward"),
                count: 1,
            }),
        );
        cancelled.set_item(
            BrewingStandBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:blaze_powder".to_string(),
                count: 1,
            }),
        );
        cancelled.fuel = 1;
        assert_eq!(
            cancelled.server_tick(&recipes),
            BrewingStandTickResult::Started
        );
        cancelled.set_item(
            BrewingStandBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:gunpowder".to_string(),
                count: 1,
            }),
        );
        assert_eq!(
            cancelled.server_tick(&recipes),
            BrewingStandTickResult::Cancelled
        );
        assert_eq!(cancelled.brew_time, 0);
    }

    #[test]
    fn crafter_block_entity_tracks_disabled_slots_triggered_pulse_and_output_like_java() {
        let mut crafter = CrafterBlockEntity::new();
        assert_eq!(CrafterBlockEntity::CONTAINER_WIDTH, 3);
        assert_eq!(CrafterBlockEntity::CONTAINER_HEIGHT, 3);
        assert_eq!(CrafterBlockEntity::NUM_DATA, 10);
        assert_eq!(crafter.redstone_signal(), 0);
        assert!(crafter.set_slot_state(8, false));
        assert!(crafter.is_slot_disabled(8));
        assert_eq!(crafter.redstone_signal(), 1);
        assert!(crafter.set_item(
            8,
            Some(PotItemStack {
                item_id: "minecraft:stone".to_string(),
                count: 1,
            }),
        ));
        assert!(!crafter.is_slot_disabled(8));
        assert!(crafter.set_item(8, None));
        assert!(crafter.set_item(
            0,
            Some(PotItemStack {
                item_id: "minecraft:oak_planks".to_string(),
                count: 2,
            }),
        ));
        assert!(crafter.set_item(
            1,
            Some(PotItemStack {
                item_id: "minecraft:oak_planks".to_string(),
                count: 1,
            }),
        ));
        assert!(!crafter.can_place_item(
            0,
            &PotItemStack {
                item_id: "minecraft:oak_planks".to_string(),
                count: 1,
            },
        ));
        assert!(crafter.can_place_item(
            2,
            &PotItemStack {
                item_id: "minecraft:oak_planks".to_string(),
                count: 1,
            },
        ));
        assert_eq!(crafter.redstone_signal(), 2);

        let recipe = CrafterRecipe {
            pattern: [
                Some("minecraft:oak_planks"),
                Some("minecraft:oak_planks"),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            result: PotItemStack {
                item_id: "minecraft:stick".to_string(),
                count: 4,
            },
            remaining_items: vec![PotItemStack {
                item_id: "minecraft:bowl".to_string(),
                count: 1,
            }],
        };
        assert_eq!(
            crafter.pulse_craft(std::slice::from_ref(&recipe)),
            CrafterPulseResult::NotTriggered
        );
        crafter.set_triggered(true);
        assert_eq!(
            crafter.pulse_craft(std::slice::from_ref(&recipe)),
            CrafterPulseResult::Crafted {
                result: PotItemStack {
                    item_id: "minecraft:stick".to_string(),
                    count: 4,
                },
                remaining_items: vec![PotItemStack {
                    item_id: "minecraft:bowl".to_string(),
                    count: 1,
                }],
            }
        );
        assert_eq!(
            crafter.crafting_ticks_remaining,
            CrafterBlockEntity::MAX_CRAFTING_TICKS
        );
        assert_eq!(
            crafter.items[0],
            Some(PotItemStack {
                item_id: "minecraft:oak_planks".to_string(),
                count: 1,
            })
        );
        assert_eq!(crafter.items[1], None);
        for _ in 1..CrafterBlockEntity::MAX_CRAFTING_TICKS {
            assert!(!crafter.server_tick());
        }
        assert!(crafter.server_tick());
        assert_eq!(crafter.crafting_ticks_remaining, 0);

        let saved = crafter.save_additional();
        let loaded = CrafterBlockEntity::load_additional(&saved);
        assert_eq!(loaded, crafter);

        let mut no_recipe = CrafterBlockEntity::new();
        no_recipe.set_triggered(true);
        assert_eq!(
            no_recipe.pulse_craft(&[recipe]),
            CrafterPulseResult::NoRecipe
        );
    }

    #[test]
    fn spawner_block_entity_tracks_spawn_data_rules_delay_and_nbt_like_java() {
        let mut spawner = SpawnerBlockEntity::default();
        assert_eq!(spawner.spawn_delay, 20);
        assert_eq!(spawner.min_spawn_delay, 200);
        assert_eq!(spawner.max_spawn_delay, 800);
        assert_eq!(spawner.spawn_count, 4);
        assert_eq!(spawner.max_nearby_entities, 6);
        assert_eq!(spawner.required_player_range, 16);
        assert_eq!(spawner.spawn_range, 4);
        assert_eq!(
            spawner.server_tick(false, true, 0, 0),
            SpawnerTickResult::Idle
        );

        spawner.set_entity_id("minecraft:zombie");
        assert_eq!(
            spawner
                .next_spawn_data
                .as_ref()
                .and_then(SpawnDataModel::entity_id),
            Some("minecraft:zombie")
        );

        spawner.spawn_potentials = vec![
            SpawnDataModel {
                weight: 1,
                ..SpawnDataModel::new("minecraft:zombie")
            },
            SpawnDataModel {
                weight: 3,
                custom_spawn_rules: Some(SpawnerCustomSpawnRules {
                    block_light_limit: (0, 7),
                    sky_light_limit: (0, 15),
                    requires_no_sky_access: true,
                }),
                equipment: Some(Tag::Compound(vec![(
                    "mainhand".to_string(),
                    Tag::String("minecraft:iron_sword".to_string()),
                )])),
                ..SpawnDataModel::new("minecraft:skeleton")
            },
        ];
        spawner.spawn_delay = -1;
        assert_eq!(
            spawner.server_tick(true, true, 0, 12),
            SpawnerTickResult::Delay
        );
        assert_eq!(spawner.spawn_delay, 212);

        spawner.spawn_delay = 2;
        assert_eq!(
            spawner.server_tick(true, true, 0, 0),
            SpawnerTickResult::CountDown
        );
        assert_eq!(spawner.spawn_delay, 1);

        spawner.spawn_delay = 0;
        spawner.next_spawn_data = Some(spawner.spawn_potentials[1].clone());
        assert_eq!(
            spawner.server_tick_with_context(SpawnerSpawnContext {
                player_in_range: true,
                spawner_blocks_work: true,
                nearby_entities: 0,
                block_light: 8,
                sky_light: 0,
                no_sky_access: true,
                collision_free: true,
                spawn_rules_ok: true,
                obstruction_free: true,
                delay_roll: 5,
                potential_roll: 0,
            }),
            SpawnerTickResult::SpawnRulesFailed
        );
        assert_eq!(spawner.spawn_delay, 0);

        assert_eq!(
            spawner.server_tick_with_context(SpawnerSpawnContext {
                player_in_range: true,
                spawner_blocks_work: true,
                nearby_entities: spawner.max_nearby_entities,
                block_light: 0,
                sky_light: 0,
                no_sky_access: true,
                collision_free: true,
                spawn_rules_ok: true,
                obstruction_free: true,
                delay_roll: 7,
                potential_roll: 0,
            }),
            SpawnerTickResult::Delay
        );
        assert_eq!(spawner.spawn_delay, 207);

        spawner.spawn_delay = 0;
        spawner.next_spawn_data = Some(spawner.spawn_potentials[1].clone());
        assert_eq!(
            spawner.server_tick_with_context(SpawnerSpawnContext {
                player_in_range: true,
                spawner_blocks_work: true,
                nearby_entities: 0,
                block_light: 7,
                sky_light: 0,
                no_sky_access: true,
                collision_free: true,
                spawn_rules_ok: true,
                obstruction_free: true,
                delay_roll: 11,
                potential_roll: 1,
            }),
            SpawnerTickResult::Spawned {
                entity_id: "minecraft:skeleton".to_string(),
                count: 4,
            }
        );
        assert_eq!(spawner.spawn_delay, 211);
        assert_eq!(
            spawner
                .next_spawn_data
                .as_ref()
                .and_then(SpawnDataModel::entity_id),
            Some("minecraft:skeleton")
        );

        let saved = spawner.save_additional();
        let update_tag = spawner.update_tag();
        assert!(compound_entries(&saved)
            .unwrap()
            .iter()
            .any(|(name, _)| name == "SpawnPotentials"));
        assert!(!compound_entries(&update_tag)
            .unwrap()
            .iter()
            .any(|(name, _)| name == "SpawnPotentials"));
        assert_eq!(SpawnerBlockEntity::load_additional(&saved), spawner);

        assert!(spawner.on_event_triggered(true, SpawnerBlockEntity::EVENT_SPAWN));
        assert_eq!(spawner.spawn_delay, spawner.min_spawn_delay);
        assert!(!spawner.on_event_triggered(true, 99));
    }

    #[test]
    fn trial_spawner_state_machine_configs_rewards_and_nbt_like_java() {
        let mut spawner = TrialSpawnerBlockEntity::default();
        spawner.config.normal_config.spawn_potentials =
            vec![SpawnDataModel::new("minecraft:zombie")];
        spawner.config.ominous_config.spawn_potentials =
            vec![SpawnDataModel::new("minecraft:breeze")];
        spawner.config.normal_config.total_mobs = 2.0;
        spawner.config.normal_config.simultaneous_mobs = 1.0;
        spawner.config.normal_config.ticks_between_spawn = 5;
        spawner.config.target_cooldown_length = 100;

        assert_eq!(spawner.state, TrialSpawnerStateModel::Inactive);
        assert_eq!(spawner.config.required_player_range, 14);
        assert_eq!(TrialSpawnerStateModel::Active.light_level(), 8);
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 0,
                can_spawn_in_level: true,
                detected_player_count: 0,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::WaitingForPlayers)
        );

        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 1,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::DetectedPlayers(2)
        );
        assert_eq!(spawner.state, TrialSpawnerStateModel::Active);
        assert_eq!(spawner.next_mob_spawns_at, 41);
        assert_eq!(spawner.active_config().target_total_mobs(1), 4);
        assert_eq!(spawner.active_config().target_simultaneous_mobs(1), 2);

        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 41,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: true,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::SpawnMob {
                entity_id: "minecraft:zombie".to_string(),
            }
        );
        assert_eq!(spawner.total_mobs_spawned, 1);
        assert_eq!(spawner.current_mobs.len(), 1);
        assert_eq!(spawner.next_mob_spawns_at, 46);

        spawner.total_mobs_spawned = spawner.active_config().target_total_mobs(1);
        spawner.current_mobs.clear();
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 47,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::ReadyForRewards
        );
        assert_eq!(
            spawner.state,
            TrialSpawnerStateModel::WaitingForRewardEjection
        );
        assert_eq!(spawner.cooldown_ends_at, 147);

        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 87,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::EjectingReward)
        );
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 107,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 1,
            }),
            TrialSpawnerTickResult::EjectedReward {
                loot_table: "minecraft:spawners/trial_chamber/key".to_string(),
                remaining_players: 1,
            }
        );
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 137,
                can_spawn_in_level: true,
                detected_player_count: 2,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::EjectedReward {
                loot_table: "minecraft:spawners/trial_chamber/key".to_string(),
                remaining_players: 0,
            }
        );
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 167,
                can_spawn_in_level: true,
                detected_player_count: 0,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::StateChanged(TrialSpawnerStateModel::Cooldown)
        );
        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 180,
                can_spawn_in_level: true,
                detected_player_count: 0,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: false,
                roll: 0,
            }),
            TrialSpawnerTickResult::CooldownFinished
        );
        assert_eq!(spawner.state, TrialSpawnerStateModel::WaitingForPlayers);

        assert_eq!(
            spawner.tick_server(TrialSpawnerTickContext {
                game_time: 200,
                can_spawn_in_level: true,
                detected_player_count: 1,
                current_mobs_alive: 0,
                spawn_success: false,
                apply_ominous: true,
                roll: 0,
            }),
            TrialSpawnerTickResult::BecameOminous
        );
        assert!(spawner.is_ominous);
        assert_eq!(spawner.next_mob_spawns_at, 240);
        assert_eq!(spawner.cooldown_ends_at, 360);

        spawner.override_entity_to_spawn("minecraft:husk");
        assert_eq!(spawner.state, TrialSpawnerStateModel::Inactive);
        assert_eq!(
            spawner.config.normal_config.spawn_potentials[0].entity_id(),
            Some("minecraft:husk")
        );
        assert_eq!(
            spawner.config.ominous_config.spawn_potentials[0].entity_id(),
            Some("minecraft:husk")
        );

        spawner.state = TrialSpawnerStateModel::Active;
        spawner.next_mob_spawns_at = 500;
        spawner.next_spawn_data = Some(SpawnDataModel::new("minecraft:husk"));
        let update_tag = spawner.update_tag();
        assert!(compound_entries(&update_tag)
            .unwrap()
            .iter()
            .any(|(name, _)| name == "next_mob_spawns_at"));
        assert!(compound_entries(&update_tag)
            .unwrap()
            .iter()
            .any(|(name, _)| name == "spawn_data"));

        let saved = spawner.save_additional();
        assert_eq!(TrialSpawnerBlockEntity::load_additional(&saved), spawner);
    }

    #[test]
    fn vault_block_entity_tracks_key_unlock_ejection_shared_update_and_nbt_like_java() {
        let mut vault = VaultBlockEntity::default();
        assert_eq!(vault.state, VaultStateModel::Inactive);
        assert_eq!(vault.state.light_level(), 6);
        assert_eq!(vault.config.activation_range, 4.0);
        assert_eq!(vault.config.deactivation_range, 4.5);
        assert_eq!(vault.config.key_item.item_id, "minecraft:trial_key");

        assert_eq!(
            vault.try_insert_key(
                "player-a",
                &PotItemStack {
                    item_id: "minecraft:trial_key".to_string(),
                    count: 1,
                },
                vec![PotItemStack {
                    item_id: "minecraft:diamond".to_string(),
                    count: 1,
                }],
                0,
            ),
            VaultInsertResult::IgnoredInactive
        );

        assert_eq!(
            vault.tick_server(20, &["player-a".to_string()], None),
            VaultTickResult::StateChanged(VaultStateModel::Active)
        );
        assert_eq!(
            vault.connected_particles_range,
            vault.config.activation_range
        );
        assert!(vault.connected_players.contains("player-a"));

        assert_eq!(
            vault.try_insert_key(
                "player-a",
                &PotItemStack {
                    item_id: "minecraft:stick".to_string(),
                    count: 1,
                },
                vec![PotItemStack {
                    item_id: "minecraft:diamond".to_string(),
                    count: 1,
                }],
                21,
            ),
            VaultInsertResult::WrongKey {
                expected: "minecraft:trial_key".to_string(),
            }
        );
        assert_eq!(vault.last_insert_fail_timestamp, 21);

        assert_eq!(
            vault.try_insert_key(
                "player-a",
                &PotItemStack {
                    item_id: "minecraft:trial_key".to_string(),
                    count: 1,
                },
                vec![
                    PotItemStack {
                        item_id: "minecraft:emerald".to_string(),
                        count: 2,
                    },
                    PotItemStack {
                        item_id: "minecraft:diamond".to_string(),
                        count: 1,
                    },
                ],
                22,
            ),
            VaultInsertResult::Unlocking { items_to_eject: 2 }
        );
        assert_eq!(vault.state, VaultStateModel::Unlocking);
        assert_eq!(vault.state_updating_resumes_at, 36);
        assert_eq!(
            vault.display_item,
            Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 1,
            })
        );
        assert!(vault.rewarded_players.contains("player-a"));

        assert_eq!(
            vault.try_insert_key(
                "player-a",
                &PotItemStack {
                    item_id: "minecraft:trial_key".to_string(),
                    count: 1,
                },
                vec![PotItemStack {
                    item_id: "minecraft:gold_ingot".to_string(),
                    count: 1,
                }],
                37,
            ),
            VaultInsertResult::AlreadyRewarded
        );

        assert_eq!(
            vault.tick_server(35, &["player-a".to_string()], None),
            VaultTickResult::Waiting
        );
        assert_eq!(
            vault.tick_server(36, &["player-a".to_string()], None),
            VaultTickResult::StateChanged(VaultStateModel::Ejecting)
        );
        assert_eq!(
            vault.tick_server(56, &["player-a".to_string()], None),
            VaultTickResult::EjectedItem(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 1,
            })
        );
        assert_eq!(
            vault.display_item,
            Some(PotItemStack {
                item_id: "minecraft:emerald".to_string(),
                count: 2,
            })
        );
        assert_eq!(
            vault.tick_server(76, &["player-a".to_string()], None),
            VaultTickResult::EjectedItem(PotItemStack {
                item_id: "minecraft:emerald".to_string(),
                count: 2,
            })
        );
        assert_eq!(
            vault.tick_server(96, &["player-a".to_string()], None),
            VaultTickResult::EjectionFinished
        );
        assert_eq!(vault.state, VaultStateModel::Inactive);

        for index in 0..130 {
            vault.add_rewarded_player(format!("player-{index:03}"));
        }
        assert_eq!(
            vault.rewarded_players.len(),
            VaultBlockEntity::MAX_REWARDED_PLAYERS
        );
        assert!(!vault.rewarded_players.contains("player-000"));
        assert!(vault.rewarded_players.contains("player-129"));

        vault.state = VaultStateModel::Active;
        assert_eq!(
            vault.tick_server(
                120,
                &["player-new".to_string()],
                Some(PotItemStack {
                    item_id: "minecraft:apple".to_string(),
                    count: 1,
                }),
            ),
            VaultTickResult::DisplayItemCycled(Some(PotItemStack {
                item_id: "minecraft:apple".to_string(),
                count: 1,
            }))
        );
        vault.tick_client();
        assert_eq!(vault.previous_spin, 0.0);
        assert_eq!(vault.current_spin, 10.0);

        let update_tag = vault.get_update_tag();
        let update_entries = compound_entries(&update_tag).unwrap();
        assert!(update_entries.iter().any(|(name, _)| name == "shared_data"));
        assert!(!update_entries
            .iter()
            .any(|(name, _)| name == "server_data" || name == "config"));

        let saved = vault.save_additional();
        let loaded = VaultBlockEntity::load_additional(&saved);
        assert_eq!(loaded.state, vault.state);
        assert_eq!(loaded.is_ominous, vault.is_ominous);
        assert_eq!(loaded.config, vault.config);
        assert_eq!(loaded.rewarded_players, vault.rewarded_players);
        assert_eq!(loaded.connected_players, vault.connected_players);
        assert_eq!(loaded.display_item, vault.display_item);
        assert_eq!(loaded.items_to_eject, vault.items_to_eject);
        assert_eq!(loaded.total_ejections_needed, vault.total_ejections_needed);
        assert_eq!(
            loaded.state_updating_resumes_at,
            vault.state_updating_resumes_at
        );
        assert_eq!(
            loaded.connected_particles_range,
            vault.connected_particles_range
        );
        assert_eq!(loaded.last_insert_fail_timestamp, 0);
        assert_eq!(loaded.current_spin, 0.0);
        assert_eq!(loaded.previous_spin, 0.0);
    }

    #[test]
    fn furnace_family_ticks_fuel_recipes_xp_sided_slots_and_speed_like_java() {
        let fuels = FuelValues::vanilla();
        let smelting = FurnaceCookingRecipe::new(
            "minecraft:iron_ingot_from_smelting_raw_iron",
            "smelting",
            "minecraft:raw_iron",
            "minecraft:iron_ingot",
            200,
            700,
        );
        let blasting = FurnaceCookingRecipe::new(
            "minecraft:iron_ingot_from_blasting_raw_iron",
            "blasting",
            "minecraft:raw_iron",
            "minecraft:iron_ingot",
            100,
            700,
        );
        let smoking = FurnaceCookingRecipe::new(
            "minecraft:cooked_beef_from_smoking",
            "smoking",
            "minecraft:beef",
            "minecraft:cooked_beef",
            100,
            350,
        );

        let mut furnace = AbstractFurnaceBlockEntity::furnace();
        assert_eq!(furnace.kind.recipe_type(), "smelting");
        assert_eq!(furnace.kind.default_cooking_time(), 200);
        furnace.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:raw_iron".to_string(),
                count: 1,
            }),
            Some(&smelting),
        );
        furnace.set_item(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:coal".to_string(),
                count: 1,
            }),
            Some(&smelting),
        );
        assert_eq!(
            furnace.server_tick(&fuels, Some(&smelting)),
            FurnaceTickResult::LitChanged { lit: true }
        );
        assert_eq!(furnace.lit_time_remaining, 1600);
        assert_eq!(furnace.lit_total_time, 1600);
        assert_eq!(furnace.cooking_time_spent, 1);
        assert!(furnace.items[AbstractFurnaceBlockEntity::FUEL_SLOT].is_none());

        for _ in 1..199 {
            furnace.server_tick(&fuels, Some(&smelting));
        }
        assert_eq!(
            furnace.server_tick(&fuels, Some(&smelting)),
            FurnaceTickResult::Burned { output_count: 1 }
        );
        assert!(furnace.items[AbstractFurnaceBlockEntity::INGREDIENT_SLOT].is_none());
        assert_eq!(
            furnace.items[AbstractFurnaceBlockEntity::RESULT_SLOT],
            Some(PotItemStack {
                item_id: "minecraft:iron_ingot".to_string(),
                count: 1,
            })
        );
        assert_eq!(
            furnace
                .recipes_used
                .get("minecraft:iron_ingot_from_smelting_raw_iron"),
            Some(&(1, 700))
        );
        assert_eq!(furnace.xp_to_award_and_clear(0.0), 1);
        assert!(furnace.recipes_used.is_empty());

        let saved = furnace.save_additional();
        let loaded =
            AbstractFurnaceBlockEntity::load_additional(FurnaceBlockEntityKind::Furnace, &saved);
        assert_eq!(loaded, furnace);

        let mut invalid_fuel = AbstractFurnaceBlockEntity::furnace();
        invalid_fuel.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:raw_iron".to_string(),
                count: 1,
            }),
            Some(&smelting),
        );
        invalid_fuel.set_item(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:stone".to_string(),
                count: 1,
            }),
            Some(&smelting),
        );
        assert_eq!(
            invalid_fuel.server_tick(&fuels, Some(&smelting)),
            FurnaceTickResult::Idle
        );
        assert_eq!(invalid_fuel.cooking_time_spent, 0);

        let mut blast = AbstractFurnaceBlockEntity::blast_furnace();
        assert_eq!(blast.kind.recipe_type(), "blasting");
        assert_eq!(blast.kind.default_cooking_time(), 100);
        blast.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:raw_iron".to_string(),
                count: 1,
            }),
            Some(&blasting),
        );
        blast.set_item(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:coal".to_string(),
                count: 1,
            }),
            Some(&blasting),
        );
        assert_eq!(
            blast.server_tick(&fuels, Some(&blasting)),
            FurnaceTickResult::LitChanged { lit: true }
        );
        assert_eq!(blast.lit_total_time, 800);
        for _ in 1..99 {
            blast.server_tick(&fuels, Some(&blasting));
        }
        assert_eq!(
            blast.server_tick(&fuels, Some(&blasting)),
            FurnaceTickResult::Burned { output_count: 1 }
        );

        let mut smoker = AbstractFurnaceBlockEntity::smoker();
        smoker.set_item(
            AbstractFurnaceBlockEntity::INGREDIENT_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:beef".to_string(),
                count: 1,
            }),
            Some(&smoking),
        );
        smoker.set_item(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            Some(PotItemStack {
                item_id: "minecraft:coal".to_string(),
                count: 1,
            }),
            Some(&smoking),
        );
        assert_eq!(
            smoker.server_tick(&fuels, Some(&smoking)),
            FurnaceTickResult::LitChanged { lit: true }
        );
        assert_eq!(smoker.kind.recipe_type(), "smoking");
        assert_eq!(smoker.lit_total_time, 800);

        assert_eq!(
            AbstractFurnaceBlockEntity::get_slots_for_face(Direction::Up),
            &[AbstractFurnaceBlockEntity::INGREDIENT_SLOT]
        );
        assert_eq!(
            AbstractFurnaceBlockEntity::get_slots_for_face(Direction::Down),
            &[
                AbstractFurnaceBlockEntity::RESULT_SLOT,
                AbstractFurnaceBlockEntity::FUEL_SLOT,
            ]
        );
        assert!(smoker.can_take_item_through_face(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            "minecraft:bucket",
            Direction::Down
        ));
        assert!(!smoker.can_take_item_through_face(
            AbstractFurnaceBlockEntity::FUEL_SLOT,
            "minecraft:coal",
            Direction::Down
        ));
        assert!(!smoker.can_place_item(
            AbstractFurnaceBlockEntity::RESULT_SLOT,
            &PotItemStack {
                item_id: "minecraft:iron_ingot".to_string(),
                count: 1,
            },
            &fuels,
        ));
        assert_eq!(
            smoker.max_stack_size(
                AbstractFurnaceBlockEntity::FUEL_SLOT,
                &PotItemStack {
                    item_id: "minecraft:bucket".to_string(),
                    count: 16,
                },
            ),
            1
        );
        assert!(smoker.comparator_output() > 0);
    }

    #[test]
    fn container_block_entities_track_loot_openers_lids_redstone_and_hopper_like_java() {
        let mut chest = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Chest);
        chest.custom_name = Some("Supply Cache".to_string());
        chest.lock_key = Some("brass_key".to_string());
        chest.loot_table = Some("minecraft:chests/simple_dungeon".to_string());
        chest.loot_table_seed = 42;

        assert_eq!(chest.kind.size(), 27);
        assert_eq!(chest.kind.menu_type(), "generic_9x3");
        assert_eq!(chest.kind.default_name(), "container.chest");
        assert!(!chest.can_open(None, false));
        assert_eq!(chest.create_menu(Some("brass_key"), true), None);
        assert_eq!(
            chest.create_menu(Some("brass_key"), false),
            Some("generic_9x3")
        );
        assert!(chest.loot_table.is_none());
        assert_eq!(chest.loot_table_seed, 0);

        assert!(chest.set_item(
            0,
            Some(PotItemStack {
                item_id: "minecraft:apple".to_string(),
                count: 32,
            }),
        ));
        assert!(chest.comparator_output() > 0);
        assert_eq!(chest.merged_chest_access_size(false), 27);
        assert_eq!(chest.merged_chest_access_size(true), 54);

        chest.start_open();
        for _ in 0..10 {
            chest.tick_lid();
        }
        assert!((chest.lid_progress - 1.0).abs() < 0.001);
        chest.stop_open();
        for _ in 0..10 {
            chest.tick_lid();
        }
        assert!((chest.lid_progress - 0.0).abs() < 0.001);

        let loaded_chest = ContainerBlockEntityModel::load_additional(
            ContainerBlockEntityKind::Chest,
            &chest.save_additional(),
        );
        assert_eq!(loaded_chest.custom_name.as_deref(), Some("Supply Cache"));
        assert_eq!(loaded_chest.lock_key.as_deref(), Some("brass_key"));
        assert_eq!(
            loaded_chest.items[0],
            Some(PotItemStack {
                item_id: "minecraft:apple".to_string(),
                count: 32,
            })
        );

        let mut trapped = ContainerBlockEntityModel::new(ContainerBlockEntityKind::TrappedChest);
        for _ in 0..20 {
            trapped.start_open();
        }
        assert_eq!(trapped.trapped_chest_signal(), 15);
        for _ in 0..6 {
            trapped.stop_open();
        }
        assert_eq!(trapped.trapped_chest_signal(), 14);

        let mut barrel = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Barrel);
        barrel.start_open();
        barrel.tick_lid();
        assert!(barrel.barrel_is_open());
        assert_eq!(barrel.lid_progress, 0.0);
        barrel.stop_open();
        assert!(!barrel.barrel_is_open());

        let mut shulker = ContainerBlockEntityModel::new(ContainerBlockEntityKind::ShulkerBox);
        shulker.shulker_color = Some(DyeColor::Purple);
        assert_eq!(shulker.kind.size(), 27);
        assert!(shulker.shulker_is_closed());
        shulker.start_open();
        assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opening);
        for _ in 0..ContainerBlockEntityModel::SHULKER_OPENING_TICK_LENGTH {
            shulker.tick_lid();
        }
        assert_eq!(shulker.shulker_status, ShulkerBoxAnimationStatus::Opened);
        assert!(!shulker.can_place_through_face(0, "minecraft:white_shulker_box", Direction::Up));
        assert!(shulker.can_place_through_face(0, "minecraft:diamond", Direction::Up));
        shulker.stop_open();
        for _ in 0..ContainerBlockEntityModel::SHULKER_OPENING_TICK_LENGTH {
            shulker.tick_lid();
        }
        assert!(shulker.shulker_is_closed());

        let mut dispenser = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Dispenser);
        assert_eq!(dispenser.kind.size(), 9);
        dispenser.set_item(
            1,
            Some(PotItemStack {
                item_id: "minecraft:arrow".to_string(),
                count: 1,
            }),
        );
        dispenser.set_item(
            5,
            Some(PotItemStack {
                item_id: "minecraft:egg".to_string(),
                count: 1,
            }),
        );
        assert_eq!(
            dispenser.activate_once(&[0, 1]),
            ContainerActivation::Dispense { slot: 1 }
        );

        let mut dropper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Dropper);
        dropper.set_item(
            2,
            Some(PotItemStack {
                item_id: "minecraft:cobblestone".to_string(),
                count: 1,
            }),
        );
        assert_eq!(
            dropper.activate_once(&[0]),
            ContainerActivation::Drop { slot: 2 }
        );

        let mut hopper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Hopper);
        assert_eq!(hopper.kind.size(), 5);
        assert_eq!(
            hopper.transfer_cooldown,
            ContainerBlockEntityModel::HOPPER_NO_COOLDOWN
        );
        hopper.set_item(
            0,
            Some(PotItemStack {
                item_id: "minecraft:iron_ingot".to_string(),
                count: 1,
            }),
        );
        assert_eq!(
            hopper.hopper_slots_for_face(Direction::Down),
            vec![0, 1, 2, 3, 4]
        );
        assert!(hopper.hopper_can_place_item(
            1,
            &PotItemStack {
                item_id: "minecraft:gold_ingot".to_string(),
                count: 1,
            },
            Direction::Up,
        ));
        assert!(hopper.hopper_can_take_item(0, Direction::Down));
        assert_eq!(
            hopper.hopper_tick(true, true, true),
            ContainerActivation::Push { from_slot: 0 }
        );
        assert_eq!(
            hopper.transfer_cooldown,
            ContainerBlockEntityModel::HOPPER_MOVE_ITEM_SPEED
        );
        assert_eq!(
            hopper.hopper_tick(true, true, true),
            ContainerActivation::None
        );

        let mut pulling_hopper = ContainerBlockEntityModel::new(ContainerBlockEntityKind::Hopper);
        pulling_hopper.transfer_cooldown = 0;
        assert_eq!(
            pulling_hopper.hopper_tick(true, false, true),
            ContainerActivation::Pull { to_slot: 0 }
        );
        let loaded_hopper = ContainerBlockEntityModel::load_additional(
            ContainerBlockEntityKind::Hopper,
            &pulling_hopper.save_additional(),
        );
        assert_eq!(
            loaded_hopper.transfer_cooldown,
            ContainerBlockEntityModel::HOPPER_MOVE_ITEM_SPEED
        );
    }

    #[test]
    fn structure_block_entity_round_trips_bounds_and_render_box() {
        let mut structure = StructureBlockEntity::new(StructureBlockMode::Save);
        assert!(!structure.has_structure_name());
        assert_eq!(structure.structure_name(), "");
        structure.set_structure_name(Some("minecraft:village/plains/houses/plains_small_house_1"));
        structure.author = "Builder".to_string();
        structure.metadata = "data".to_string();
        structure.set_structure_pos(BlockPos {
            x: 99,
            y: -99,
            z: 7,
        });
        structure.set_structure_size((50, -2, 12));
        structure.mirror = StructureMirror::LeftRight;
        structure.rotation = StructureRotation::Clockwise90;
        structure.ignore_entities = false;
        structure.strict = true;
        structure.powered = true;
        structure.show_air = true;
        structure.show_bounding_box = false;
        structure.integrity = 0.65;
        structure.seed = 12345;

        assert_eq!(
            structure.structure_pos,
            BlockPos {
                x: 48,
                y: -48,
                z: 7
            }
        );
        assert_eq!(structure.structure_size, (48, 0, 12));
        assert_eq!(
            structure.render_mode(),
            StructureRenderMode::BoxAndInvisibleBlocks
        );
        assert_eq!(
            structure.renderable_box(),
            StructureRenderableBox {
                min: BlockPos {
                    x: 48,
                    y: -48,
                    z: 7
                },
                max: BlockPos {
                    x: 60,
                    y: -48,
                    z: 55
                },
            }
        );

        let saved = structure.save_additional();
        assert_eq!(StructureBlockEntity::load_additional(&saved), structure);
        assert_eq!(structure.get_update_tag(), saved);

        let loaded = StructureBlockEntity::load_additional(&Tag::Compound(vec![
            ("name".to_string(), Tag::String(String::new())),
            ("posX".to_string(), Tag::Int(-99)),
            ("posY".to_string(), Tag::Int(2)),
            ("posZ".to_string(), Tag::Int(99)),
            ("sizeX".to_string(), Tag::Int(-1)),
            ("sizeY".to_string(), Tag::Int(64)),
            ("sizeZ".to_string(), Tag::Int(9)),
            (
                "rotation".to_string(),
                Tag::String("CLOCKWISE_180".to_string()),
            ),
            ("mirror".to_string(), Tag::String("FRONT_BACK".to_string())),
            ("mode".to_string(), Tag::String("LOAD".to_string())),
            ("showboundingbox".to_string(), Tag::Byte(0)),
        ]));
        assert_eq!(loaded.structure_name, None);
        assert_eq!(
            loaded.structure_pos,
            BlockPos {
                x: -48,
                y: 2,
                z: 48
            }
        );
        assert_eq!(loaded.structure_size, (0, 48, 9));
        assert_eq!(loaded.rotation, StructureRotation::Clockwise180);
        assert_eq!(loaded.mirror, StructureMirror::FrontBack);
        assert_eq!(loaded.mode, StructureBlockMode::Load);
        assert_eq!(loaded.render_mode(), StructureRenderMode::None);
        assert!(loaded.ignore_entities);
        assert_eq!(loaded.integrity, 1.0);
    }

    #[test]
    fn banner_block_entity_tracks_color_patterns_and_update_tag_shape() {
        let mut banner =
            BannerBlockEntity::from_block_state("minecraft:light_blue_wall_banner").unwrap();
        assert_eq!(banner.base_color, DyeColor::LightBlue);
        assert!(banner.add_pattern("minecraft:stripe_bottom", DyeColor::Red));
        assert!(banner.add_pattern("minecraft:flower", DyeColor::Yellow));
        assert!(banner.add_pattern("minecraft:creeper", DyeColor::Green));
        assert!(banner.add_pattern("minecraft:skull", DyeColor::Black));
        assert!(banner.add_pattern("minecraft:mojang", DyeColor::Purple));
        assert!(banner.add_pattern("minecraft:globe", DyeColor::White));
        assert!(!banner.add_pattern("minecraft:extra", DyeColor::Orange));
        banner.custom_name = Some("{\"text\":\"Marker\"}".to_string());

        let saved = banner.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "patterns".to_string(),
                    Tag::List(vec![
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:stripe_bottom".to_string())
                            ),
                            ("color".to_string(), Tag::String("red".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:flower".to_string())
                            ),
                            ("color".to_string(), Tag::String("yellow".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:creeper".to_string())
                            ),
                            ("color".to_string(), Tag::String("green".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:skull".to_string())
                            ),
                            ("color".to_string(), Tag::String("black".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:mojang".to_string())
                            ),
                            ("color".to_string(), Tag::String("purple".to_string())),
                        ]),
                        Tag::Compound(vec![
                            (
                                "pattern".to_string(),
                                Tag::String("minecraft:globe".to_string())
                            ),
                            ("color".to_string(), Tag::String("white".to_string())),
                        ]),
                    ])
                ),
                (
                    "CustomName".to_string(),
                    Tag::String("{\"text\":\"Marker\"}".to_string())
                ),
            ])
        );

        let loaded =
            BannerBlockEntity::load_additional("minecraft:light_blue_wall_banner", &saved).unwrap();
        assert_eq!(loaded, banner);
        assert_eq!(BannerBlockEntity::from_block_state("minecraft:stone"), None);

        let mut entity = BlockEntity::new(
            BlockEntityTypeId::Banner,
            pos(),
            "minecraft:light_blue_banner",
        )
        .unwrap();
        entity.custom_data.insert(
            "patterns".to_string(),
            match saved {
                Tag::Compound(fields) => fields
                    .into_iter()
                    .find(|(key, _)| key == "patterns")
                    .map(|(_, value)| value)
                    .unwrap(),
                _ => unreachable!(),
            },
        );
        assert!(
            matches!(entity.get_update_tag(), Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "patterns") && fields.iter().all(|(key, _)| key != "id"))
        );
    }

    #[test]
    fn banner_pattern_layers_preserve_order_and_enforce_six_layer_cap() {
        let mut banner = BannerBlockEntity::from_block_state("minecraft:red_banner").unwrap();
        let layers = [
            ("minecraft:stripe_bottom", DyeColor::White),
            ("minecraft:stripe_top", DyeColor::Black),
            ("minecraft:stripe_left", DyeColor::Blue),
            ("minecraft:stripe_right", DyeColor::Yellow),
            ("minecraft:diagonal_left", DyeColor::Green),
            ("minecraft:diagonal_right", DyeColor::Purple),
        ];
        for (pattern, color) in layers {
            assert!(banner.add_pattern(pattern, color));
        }
        assert!(!banner.add_pattern("minecraft:globe", DyeColor::Cyan));
        banner.custom_name = Some("{\"text\":\"Six Layers\"}".to_string());

        let saved = banner.save_additional();
        let Tag::Compound(fields) = &saved else {
            panic!("banner save_additional should produce a compound");
        };
        assert_eq!(fields[0].0, "patterns");
        assert_eq!(fields[1].0, "CustomName");

        let Tag::List(saved_layers) = &fields[0].1 else {
            panic!("patterns should be a list");
        };
        assert_eq!(saved_layers.len(), BannerBlockEntity::MAX_PATTERNS);
        for (idx, (expected_pattern, expected_color)) in layers.iter().enumerate() {
            assert_eq!(
                saved_layers[idx],
                Tag::Compound(vec![
                    (
                        "pattern".to_string(),
                        Tag::String((*expected_pattern).to_string())
                    ),
                    (
                        "color".to_string(),
                        Tag::String(expected_color.vanilla_name().to_string())
                    ),
                ])
            );
        }

        let loaded = BannerBlockEntity::load_additional("minecraft:red_wall_banner", &saved)
            .expect("saved red banner should load");
        assert_eq!(loaded.base_color, DyeColor::Red);
        assert_eq!(loaded.patterns, banner.patterns);
        assert_eq!(loaded.custom_name, banner.custom_name);

        let overlong = Tag::Compound(vec![(
            "patterns".to_string(),
            Tag::List(
                (0..8)
                    .map(|idx| {
                        BannerPatternLayer {
                            pattern: format!("minecraft:test_{idx}"),
                            color: DyeColor::White,
                        }
                        .to_tag()
                    })
                    .collect(),
            ),
        )]);
        let truncated =
            BannerBlockEntity::load_additional("minecraft:white_banner", &overlong).unwrap();
        assert_eq!(truncated.patterns.len(), BannerBlockEntity::MAX_PATTERNS);
        assert_eq!(truncated.patterns[5].pattern, "minecraft:test_5");
    }

    #[test]
    fn decorated_pot_saves_sherds_item_loot_and_wobble_like_java() {
        let mut pot = DecoratedPotBlockEntity {
            decorations: PotDecorations::new(
                Some("minecraft:angler_pottery_sherd".to_string()),
                None,
                Some("minecraft:arms_up_pottery_sherd".to_string()),
                Some("minecraft:brick".to_string()),
            ),
            item: Some(PotItemStack {
                item_id: "minecraft:diamond".to_string(),
                count: 2,
            }),
            ..DecoratedPotBlockEntity::default()
        };

        assert_eq!(
            pot.decorations.ordered(),
            vec![
                "minecraft:angler_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
                "minecraft:arms_up_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
            ]
        );
        let saved = pot.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                (
                    "sherds".to_string(),
                    Tag::List(vec![
                        Tag::String("minecraft:angler_pottery_sherd".to_string()),
                        Tag::String("minecraft:brick".to_string()),
                        Tag::String("minecraft:arms_up_pottery_sherd".to_string()),
                        Tag::String("minecraft:brick".to_string()),
                    ]),
                ),
                (
                    "item".to_string(),
                    Tag::Compound(vec![
                        (
                            "id".to_string(),
                            Tag::String("minecraft:diamond".to_string())
                        ),
                        ("count".to_string(), Tag::Int(2)),
                    ]),
                ),
            ])
        );
        assert_eq!(DecoratedPotBlockEntity::load_additional(&saved), pot);

        pot.loot_table = Some("minecraft:chests/trial_chambers/reward".to_string());
        pot.loot_table_seed = 123;
        let loot_saved = pot.save_additional();
        assert!(
            matches!(&loot_saved, Tag::Compound(fields) if fields.iter().any(|(key, _)| key == "LootTable") && fields.iter().all(|(key, _)| key != "item"))
        );
        let loaded_loot = DecoratedPotBlockEntity::load_additional(&loot_saved);
        assert_eq!(loaded_loot.loot_table, pot.loot_table);
        assert_eq!(loaded_loot.loot_table_seed, 123);
        assert_eq!(loaded_loot.item, None);

        assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);
        assert_eq!(DecoratedPotWobbleStyle::Negative.duration(), 10);
        assert!(pot.trigger_event(
            DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
            DecoratedPotWobbleStyle::Negative.id(),
            42,
        ));
        assert_eq!(pot.wobble_started_at_tick, 42);
        assert_eq!(
            pot.last_wobble_style,
            Some(DecoratedPotWobbleStyle::Negative)
        );
        assert!(!pot.trigger_event(99, DecoratedPotWobbleStyle::Positive.id(), 43));
        assert!(!pot.trigger_event(DecoratedPotBlockEntity::EVENT_POT_WOBBLES, 99, 43));
    }

    #[test]
    fn decorated_pot_wobble_and_destruction_drops_preserve_sherds_and_item() {
        let mut pot = DecoratedPotBlockEntity {
            decorations: PotDecorations::new(
                Some("minecraft:arms_up_pottery_sherd".to_string()),
                Some("minecraft:blade_pottery_sherd".to_string()),
                None,
                Some("minecraft:brewer_pottery_sherd".to_string()),
            ),
            ..DecoratedPotBlockEntity::default()
        };

        pot.item = Some(PotItemStack {
            item_id: "minecraft:emerald".to_string(),
            count: 3,
        });
        assert!(pot.trigger_event(
            DecoratedPotBlockEntity::EVENT_POT_WOBBLES,
            DecoratedPotWobbleStyle::Positive.id(),
            2400,
        ));
        assert_eq!(pot.wobble_started_at_tick, 2400);
        assert_eq!(
            pot.last_wobble_style,
            Some(DecoratedPotWobbleStyle::Positive)
        );
        assert_eq!(DecoratedPotWobbleStyle::Positive.duration(), 7);

        let drops = pot.destruction_drops();
        assert_eq!(
            drops.decoration_items,
            vec![
                "minecraft:arms_up_pottery_sherd".to_string(),
                "minecraft:blade_pottery_sherd".to_string(),
                "minecraft:brick".to_string(),
                "minecraft:brewer_pottery_sherd".to_string(),
            ]
        );
        assert_eq!(
            drops.stored_item,
            Some(PotItemStack {
                item_id: "minecraft:emerald".to_string(),
                count: 3,
            })
        );

        pot.item = Some(PotItemStack {
            item_id: "minecraft:air".to_string(),
            count: 0,
        });
        assert_eq!(pot.destruction_drops().stored_item, None);
    }

    #[test]
    fn brushable_block_entity_brushes_resets_loot_and_update_tag_like_java() {
        assert_eq!(BrushableBlockEntity::BRUSH_COOLDOWN_TICKS, 10);
        assert_eq!(BrushableBlockEntity::BRUSH_RESET_TICKS, 40);
        assert_eq!(BrushableBlockEntity::REQUIRED_BRUSHES_TO_BREAK, 10);

        let mut brushable = BrushableBlockEntity::new();
        brushable.set_loot_table("minecraft:archaeology/desert_pyramid", 99);
        assert_eq!(
            brushable.save_additional(),
            Tag::Compound(vec![
                (
                    "LootTable".to_string(),
                    Tag::String("minecraft:archaeology/desert_pyramid".to_string())
                ),
                ("LootTableSeed".to_string(), Tag::Long(99)),
            ])
        );

        let generated_item = PotItemStack {
            item_id: "minecraft:diamond".to_string(),
            count: 1,
        };
        assert_eq!(
            brushable.brush(100, Direction::North, Some(generated_item.clone())),
            BrushResult::InProgress { dusted: 1 }
        );
        assert_eq!(brushable.hit_direction, Some(Direction::North));
        assert_eq!(brushable.brush_count, 1);
        assert_eq!(brushable.brush_count_resets_at_tick, 140);
        assert_eq!(brushable.cooldown_ends_at_tick, 110);
        assert_eq!(brushable.item, Some(generated_item.clone()));
        assert_eq!(brushable.loot_table, None);
        assert_eq!(
            brushable.get_update_tag(),
            Tag::Compound(vec![
                (
                    "hit_direction".to_string(),
                    Tag::String("north".to_string())
                ),
                ("item".to_string(), generated_item.to_tag()),
            ])
        );
        assert_eq!(
            brushable.brush(105, Direction::South, None),
            BrushResult::CoolingDown
        );
        assert_eq!(brushable.hit_direction, Some(Direction::North));

        assert_eq!(
            brushable.brush(110, Direction::South, None),
            BrushResult::InProgress { dusted: 1 }
        );
        assert_eq!(
            brushable.brush(120, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(130, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(140, Direction::South, None),
            BrushResult::InProgress { dusted: 2 }
        );
        assert_eq!(
            brushable.brush(150, Direction::South, None),
            BrushResult::InProgress { dusted: 3 }
        );
        assert_eq!(brushable.brush_count, 6);

        assert_eq!(brushable.check_reset(189), None);
        assert_eq!(brushable.check_reset(190), Some(2));
        assert_eq!(brushable.brush_count, 4);
        assert_eq!(brushable.brush_count_resets_at_tick, 194);
        assert_eq!(brushable.check_reset(194), Some(1));
        assert_eq!(brushable.brush_count, 2);
        assert_eq!(brushable.check_reset(198), Some(0));
        assert_eq!(brushable.brush_count, 0);
        assert_eq!(brushable.hit_direction, None);
        assert_eq!(brushable.cooldown_ends_at_tick, 0);

        brushable.hit_direction = Some(Direction::East);
        brushable.item = Some(generated_item.clone());
        let saved_item = brushable.save_additional();
        assert_eq!(
            BrushableBlockEntity::load_additional(&saved_item),
            brushable
        );
        assert_eq!(
            brushable.drop_content(),
            Some((generated_item, Direction::East))
        );
        assert_eq!(brushable.item, None);

        let mut completing = BrushableBlockEntity::new();
        for step in 0..9 {
            assert!(matches!(
                completing.brush(step * 10, Direction::Up, None),
                BrushResult::InProgress { .. }
            ));
        }
        assert_eq!(
            completing.brush(90, Direction::Up, None),
            BrushResult::Completed
        );
        assert_eq!(completing.brush_count, 10);
    }

    #[test]
    fn copper_golem_statue_tracks_weather_pose_comparator_and_clone_components() {
        let mut statue = CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:waxed_weathered_copper_golem_statue",
            CopperGolemStatuePose::Standing,
        )
        .unwrap();
        assert_eq!(statue.weather_state, CopperWeatherState::Weathered);
        assert_eq!(statue.weather_state.serialized_name(), "weathered");
        assert!(statue.waxed);
        assert_eq!(statue.comparator_output(), 1);

        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Sitting);
        assert_eq!(statue.comparator_output(), 2);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Running);
        assert_eq!(statue.comparator_output(), 3);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Star);
        assert_eq!(statue.comparator_output(), 4);
        statue.update_pose();
        assert_eq!(statue.pose, CopperGolemStatuePose::Standing);

        statue.custom_name = Some("{\"text\":\"Copper Buddy\"}".to_string());
        assert_eq!(
            statue.save_additional(),
            Tag::Compound(vec![(
                "CustomName".to_string(),
                Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
            )])
        );
        assert_eq!(
            statue.clone_item_components(),
            Tag::Compound(vec![
                (
                    "minecraft:block_state".to_string(),
                    Tag::Compound(vec![(
                        "copper_golem_pose".to_string(),
                        Tag::String("standing".to_string())
                    )])
                ),
                (
                    "minecraft:custom_name".to_string(),
                    Tag::String("{\"text\":\"Copper Buddy\"}".to_string())
                ),
            ])
        );

        let oxidized = CopperGolemStatueBlockEntity::from_block_state(
            "minecraft:oxidized_copper_golem_statue",
            CopperGolemStatuePose::Star,
        )
        .unwrap();
        assert_eq!(oxidized.weather_state, CopperWeatherState::Oxidized);
        assert!(!oxidized.waxed);
        assert_eq!(oxidized.comparator_output(), 4);
        assert_eq!(
            CopperGolemStatueBlockEntity::from_block_state(
                "minecraft:copper_block",
                CopperGolemStatuePose::Standing,
            ),
            None
        );
    }

    #[test]
    fn skull_block_entity_saves_profile_components_and_animation_like_java() {
        let profile = Tag::Compound(vec![
            ("name".to_string(), Tag::String("Steve".to_string())),
            (
                "id".to_string(),
                Tag::String("8667ba71-b85a-4004-af54-457a9734eed7".to_string()),
            ),
        ]);
        let mut skull = SkullBlockEntity::new();
        skull.profile = Some(profile.clone());
        skull.note_block_sound = Some("minecraft:block.note_block.basedrum".to_string());
        skull.custom_name = Some("{\"text\":\"Head\"}".to_string());

        let saved = skull.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![
                ("profile".to_string(), profile.clone()),
                (
                    "note_block_sound".to_string(),
                    Tag::String("minecraft:block.note_block.basedrum".to_string())
                ),
                (
                    "custom_name".to_string(),
                    Tag::String("{\"text\":\"Head\"}".to_string())
                ),
            ])
        );
        assert_eq!(SkullBlockEntity::load_additional(&saved), skull);
        assert_eq!(skull.get_update_tag(), saved);

        skull.animation_tick(true);
        skull.animation_tick(true);
        assert!(skull.is_animating);
        assert_eq!(skull.animation_tick_count, 2);
        assert_eq!(skull.animation(0.5), 2.5);
        skull.animation_tick(false);
        assert!(!skull.is_animating);
        assert_eq!(skull.animation(0.5), 2.0);

        let mut tag_with_components = saved.clone();
        SkullBlockEntity::remove_components_from_tag(&mut tag_with_components);
        assert_eq!(tag_with_components, Tag::Compound(vec![]));

        let mut from_components = SkullBlockEntity::new();
        from_components.apply_implicit_components(&BTreeMap::from([
            ("minecraft:profile".to_string(), profile.clone()),
            (
                "minecraft:note_block_sound".to_string(),
                Tag::String("minecraft:block.note_block.harp".to_string()),
            ),
            (
                "minecraft:custom_name".to_string(),
                Tag::String("{\"text\":\"Component Head\"}".to_string()),
            ),
        ]));
        assert_eq!(from_components.profile, Some(profile.clone()));
        assert_eq!(
            from_components.note_block_sound,
            Some("minecraft:block.note_block.harp".to_string())
        );
        assert_eq!(
            from_components.custom_name,
            Some("{\"text\":\"Component Head\"}".to_string())
        );
        assert_eq!(
            from_components.collect_implicit_components(),
            BTreeMap::from([
                ("minecraft:profile".to_string(), profile),
                (
                    "minecraft:note_block_sound".to_string(),
                    Tag::String("minecraft:block.note_block.harp".to_string())
                ),
                (
                    "minecraft:custom_name".to_string(),
                    Tag::String("{\"text\":\"Component Head\"}".to_string())
                ),
            ])
        );
    }

    #[test]
    fn conduit_block_entity_scans_frame_applies_effects_and_tracks_target() {
        assert_eq!(ConduitBlockEntity::BLOCK_REFRESH_RATE, 40);
        assert_eq!(ConduitBlockEntity::MIN_ACTIVE_SIZE, 16);
        assert_eq!(ConduitBlockEntity::MIN_KILL_SIZE, 42);
        assert_eq!(ConduitBlockEntity::EFFECT_DURATION_TICKS, 260);
        assert_eq!(ConduitBlockEntity::KILL_RANGE, 8.0);
        assert!(ConduitBlockEntity::is_valid_frame_block(
            "minecraft:sea_lantern"
        ));
        assert!(!ConduitBlockEntity::is_valid_frame_block("minecraft:stone"));

        let origin = BlockPos {
            x: 10,
            y: 64,
            z: 10,
        };
        let mut conduit = ConduitBlockEntity::new();
        assert!(!conduit.update_shape(origin, |_| false, |_| "minecraft:sea_lantern"));
        assert!(conduit.effect_blocks.is_empty());

        assert!(conduit.update_shape(origin, |_| true, |_| "minecraft:prismarine"));
        assert_eq!(conduit.effect_blocks.len(), 42);
        assert_eq!(
            ConduitBlockEntity::effect_range(conduit.effect_blocks.len()),
            96
        );
        conduit.is_active = true;
        assert_eq!(
            conduit.apply_effects(),
            Some(ConduitEffectApplication {
                range: 96,
                duration_ticks: 260,
            })
        );

        let active_frame: Vec<BlockPos> = conduit.effect_blocks.iter().copied().take(16).collect();
        let active_frame_lookup = |pos: BlockPos| {
            if active_frame.contains(&pos) {
                "minecraft:dark_prismarine"
            } else {
                "minecraft:air"
            }
        };
        let mut minimum = ConduitBlockEntity::new();
        assert!(minimum.update_shape(origin, |_| true, active_frame_lookup));
        assert_eq!(minimum.effect_blocks.len(), 16);
        assert_eq!(
            ConduitBlockEntity::effect_range(minimum.effect_blocks.len()),
            32
        );
        minimum.is_hunting = minimum.effect_blocks.len() >= ConduitBlockEntity::MIN_KILL_SIZE;
        assert!(!minimum.update_destroy_target(
            origin,
            &[ConduitTarget {
                id: "guardian".to_string(),
                pos: BlockPos {
                    x: 12,
                    y: 64,
                    z: 10
                },
                alive: true,
                enemy: true,
                in_water_or_rain: true,
            }]
        ));
        assert_eq!(minimum.destroy_target, None);

        let targets = vec![
            ConduitTarget {
                id: "outside".to_string(),
                pos: BlockPos {
                    x: 18,
                    y: 64,
                    z: 10,
                },
                alive: true,
                enemy: true,
                in_water_or_rain: true,
            },
            ConduitTarget {
                id: "guardian".to_string(),
                pos: BlockPos {
                    x: 17,
                    y: 64,
                    z: 10,
                },
                alive: true,
                enemy: true,
                in_water_or_rain: true,
            },
        ];
        let mut hunting = ConduitBlockEntity::new();
        let attacked =
            hunting.server_tick(40, origin, |_| true, |_| "minecraft:sea_lantern", &targets);
        assert_eq!(attacked.as_deref(), Some("guardian"));
        assert!(hunting.is_active);
        assert!(hunting.is_hunting);
        assert_eq!(hunting.destroy_target.as_deref(), Some("guardian"));
        assert_eq!(hunting.active_rotation(0.0), -0.0375);

        assert_eq!(
            hunting.server_tick(80, origin, |_| true, |_| "minecraft:sea_lantern", &targets),
            None
        );
        let dead_target = [ConduitTarget {
            id: "guardian".to_string(),
            pos: BlockPos {
                x: 17,
                y: 64,
                z: 10,
            },
            alive: false,
            enemy: true,
            in_water_or_rain: true,
        }];
        assert_eq!(
            hunting.server_tick(
                120,
                origin,
                |_| true,
                |_| "minecraft:sea_lantern",
                &dead_target
            ),
            None
        );
        assert_eq!(hunting.destroy_target, None);

        hunting.destroy_target = Some("guardian".to_string());
        let saved = hunting.save_additional();
        assert_eq!(
            saved,
            Tag::Compound(vec![(
                "Target".to_string(),
                Tag::String("guardian".to_string())
            )])
        );
        assert_eq!(
            ConduitBlockEntity::load_additional(&saved).destroy_target,
            Some("guardian".to_string())
        );
        assert_eq!(hunting.get_update_tag(), saved);
    }

    #[test]
    fn campfire_block_entity_cooks_cools_saves_and_updates_items_like_java() {
        assert_eq!(CampfireBlockEntity::NUM_SLOTS, 4);
        assert_eq!(CampfireBlockEntity::DEFAULT_COOKING_TIME, 600);
        assert_eq!(CampfireBlockEntity::BURN_COOL_SPEED, 2);

        let mut campfire = CampfireBlockEntity::new(true);
        assert!(campfire.place_food(
            PotItemStack {
                item_id: "minecraft:cod".to_string(),
                count: 3,
            },
            Some(3),
        ));
        assert_eq!(
            campfire.items[0],
            Some(PotItemStack {
                item_id: "minecraft:cod".to_string(),
                count: 1,
            })
        );
        assert_eq!(campfire.cooking_time[0], 3);
        assert!((1..CampfireBlockEntity::NUM_SLOTS).all(|slot| campfire.items[slot].is_none()));

        assert_eq!(
            campfire.cook_tick(true, |item| PotItemStack {
                item_id: format!(
                    "minecraft:cooked_{}",
                    item.item_id.trim_start_matches("minecraft:")
                ),
                count: 1,
            }),
            vec![CampfireTickResult::Changed]
        );
        assert_eq!(campfire.cooking_progress[0], 1);
        assert_eq!(campfire.cooldown_tick(), vec![CampfireTickResult::Changed]);
        assert_eq!(campfire.cooking_progress[0], 0);

        campfire.cooking_progress[0] = 2;
        assert_eq!(
            campfire.cook_tick(true, |item| PotItemStack {
                item_id: format!(
                    "minecraft:cooked_{}",
                    item.item_id.trim_start_matches("minecraft:")
                ),
                count: 1,
            }),
            vec![CampfireTickResult::Cooked {
                slot: 0,
                item: PotItemStack {
                    item_id: "minecraft:cooked_cod".to_string(),
                    count: 1,
                },
            }]
        );
        assert_eq!(campfire.items[0], None);
        assert_eq!(campfire.cooking_progress[0], 0);
        assert_eq!(campfire.cooking_time[0], 0);

        assert!(campfire.place_food(
            PotItemStack {
                item_id: "minecraft:salmon".to_string(),
                count: 1,
            },
            None,
        ));
        campfire.cooking_progress[0] = 5;
        assert!(campfire.place_food(
            PotItemStack {
                item_id: "minecraft:beef".to_string(),
                count: 1,
            },
            Some(10),
        ));
        assert_eq!(campfire.cooking_time[0], 600);
        assert_eq!(campfire.cooking_time[1], 10);

        let saved = campfire.save_additional();
        assert_eq!(CampfireBlockEntity::load_additional(&saved), campfire);
        assert_eq!(
            campfire.get_update_tag(),
            Tag::Compound(vec![(
                "Items".to_string(),
                Tag::List(vec![
                    Tag::Compound(vec![
                        ("Slot".to_string(), Tag::Byte(0)),
                        (
                            "id".to_string(),
                            Tag::String("minecraft:salmon".to_string())
                        ),
                        ("count".to_string(), Tag::Int(1)),
                    ]),
                    Tag::Compound(vec![
                        ("Slot".to_string(), Tag::Byte(1)),
                        ("id".to_string(), Tag::String("minecraft:beef".to_string())),
                        ("count".to_string(), Tag::Int(1)),
                    ]),
                ])
            )])
        );

        campfire.clear_content();
        assert!(campfire.items.iter().all(Option::is_none));
        assert_eq!(campfire.cooldown_tick(), vec![CampfireTickResult::Changed]);
        assert_eq!(campfire.cooking_progress[0], 3);
    }

    #[test]
    fn sculk_sensor_block_entity_tracks_vibration_phase_frequency_and_power() {
        assert_eq!(SculkSensorBlockEntity::LISTENER_RADIUS, 8);
        assert_eq!(SculkSensorBlockEntity::DEFAULT_LAST_VIBRATION_FREQUENCY, 0);

        let mut sensor = SculkSensorBlockEntity::new();
        assert!(sensor.can_receive_vibration("minecraft:step", true));
        assert!(!sensor.can_receive_vibration("minecraft:unknown", true));
        assert!(!sensor.can_receive_vibration("minecraft:step", false));

        assert_eq!(
            sensor.receive_vibration("minecraft:block_place", 3.2),
            Some(SculkSensorTickResult::Activate {
                frequency: 13,
                redstone: 9,
            })
        );
        assert_eq!(sensor.last_vibration_frequency, 13);
        assert_eq!(sensor.power, 9);
        assert_eq!(sensor.phase, SculkSensorPhase::VibrationDone);
        assert!(!sensor.can_receive_vibration("minecraft:step", true));
        for _ in 0..SculkSensorBlockEntity::ACTIVE_TICKS - 1 {
            assert_eq!(sensor.tick(0), SculkSensorTickResult::None);
        }
        assert_eq!(sensor.tick(0), SculkSensorTickResult::Cooldown);
        assert_eq!(sensor.phase, SculkSensorPhase::Cooldown);
        for _ in 0..SculkSensorBlockEntity::COOLDOWN_TICKS - 1 {
            assert_eq!(sensor.tick(0), SculkSensorTickResult::None);
        }
        assert_eq!(sensor.tick(0), SculkSensorTickResult::Deactivate);
        assert_eq!(sensor.phase, SculkSensorPhase::Listening);
        assert_eq!(sensor.power, 0);

        let mut delayed = SculkSensorBlockEntity::new();
        let event = crate::game_event::game_event_by_id("minecraft:entity_damage").unwrap();
        assert!(delayed.queue_vibration(
            VibrationInfo {
                event,
                distance: 2.9,
                pos: crate::entity_physics::Vec3::ZERO,
                source_entity: Some("zombie".to_string()),
                projectile_owner: None,
            },
            5,
        ));
        assert_eq!(delayed.tick(5), SculkSensorTickResult::None);
        assert_eq!(
            delayed.tick(6),
            SculkSensorTickResult::Particle {
                travel_time_in_ticks: 2
            }
        );
        assert_eq!(delayed.phase, SculkSensorPhase::Ticking);
        assert_eq!(delayed.tick(7), SculkSensorTickResult::None);
        assert_eq!(
            delayed.tick(8),
            SculkSensorTickResult::Activate {
                frequency: 7,
                redstone: 15,
            }
        );
        assert_eq!(delayed.last_vibration_frequency, 7);

        let saved = delayed.save_additional();
        let loaded = SculkSensorBlockEntity::load_additional(&saved);
        assert_eq!(
            loaded.last_vibration_frequency,
            delayed.last_vibration_frequency
        );
        assert_eq!(loaded.phase, delayed.phase);
        assert_eq!(loaded.power, delayed.power);
        assert_eq!(loaded.active_ticks, delayed.active_ticks);
        assert_eq!(delayed.get_update_tag(), saved);
    }

    #[test]
    fn calibrated_sculk_sensor_filters_vibrations_by_back_signal() {
        assert_eq!(CalibratedSculkSensorBlockEntity::LISTENER_RADIUS, 16);

        let mut sensor = CalibratedSculkSensorBlockEntity::new(13);
        assert!(sensor.can_receive_vibration("minecraft:block_place", true));
        assert!(!sensor.can_receive_vibration("minecraft:explode", true));
        assert_eq!(sensor.receive_vibration("minecraft:explode", 4.0), None);
        assert_eq!(
            sensor.receive_vibration("minecraft:block_place", 4.0),
            Some(SculkSensorTickResult::Activate {
                frequency: 13,
                redstone: 12,
            })
        );
        assert_eq!(sensor.sensor.listener_radius, 16);
        assert_eq!(sensor.sensor.last_vibration_frequency, 13);
        assert_eq!(sensor.sensor.power, 12);

        let saved = sensor.save_additional();
        let loaded = CalibratedSculkSensorBlockEntity::load_additional(&saved);
        assert_eq!(loaded.back_signal, 13);
        assert_eq!(loaded.sensor.listener_radius, 16);
        assert_eq!(loaded.sensor.last_vibration_frequency, 13);
        assert_eq!(loaded.sensor.power, 12);

        let mut unfiltered = CalibratedSculkSensorBlockEntity::new(0);
        assert_eq!(
            unfiltered.receive_vibration("minecraft:explode", 4.0),
            Some(SculkSensorTickResult::Activate {
                frequency: 15,
                redstone: 12,
            })
        );
        unfiltered.set_back_signal(99);
        assert_eq!(unfiltered.back_signal, 15);
    }

    #[test]
    fn sculk_catalyst_block_entity_queues_charge_and_pulses_on_mob_death() {
        assert_eq!(SculkCatalystBlockEntity::LISTENER_RADIUS, 8);
        assert_eq!(SculkCatalystBlockEntity::PULSE_TICKS, 8);
        assert_eq!(SculkCatalystBlockEntity::MAX_CURSORS, 32);
        assert_eq!(SculkCatalystBlockEntity::MAX_CHARGE, 1000);

        let mut catalyst = SculkCatalystBlockEntity::new();
        assert_eq!(
            catalyst.handle_entity_die(BlockPos { x: 3, y: 64, z: -2 }, 2300, true, false,),
            SculkCatalystEventResult::Bloom { pulse_ticks: 8 }
        );
        assert_eq!(catalyst.pulse_ticks, 8);
        assert_eq!(
            catalyst.cursors,
            vec![
                SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 1000),
                SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 1000),
                SculkChargeCursor::new(BlockPos { x: 3, y: 65, z: -2 }, 300),
            ]
        );

        let saved = catalyst.save_additional();
        assert_eq!(SculkCatalystBlockEntity::load_additional(&saved), catalyst);
        catalyst.tick(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(catalyst.pulse_ticks, 7);
        assert_eq!(catalyst.cursors[0].decay_delay, 0);
        catalyst.tick(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(catalyst.cursors[0].charge, 999);
        assert_eq!(catalyst.cursors[0].decay_delay, 1);

        let mut ignored = SculkCatalystBlockEntity::new();
        assert_eq!(
            ignored.handle_entity_die(BlockPos { x: 0, y: 0, z: 0 }, 5, true, true),
            SculkCatalystEventResult::Ignored
        );
        assert!(ignored.cursors.is_empty());
        assert_eq!(ignored.pulse_ticks, 0);

        ignored.add_cursors(BlockPos { x: 0, y: 0, z: 0 }, 40_000);
        assert_eq!(ignored.cursors.len(), 32);
        assert!(ignored.cursors.iter().all(|cursor| cursor.charge == 1000));
        ignored.cursors[0].pos = BlockPos {
            x: 2000,
            y: 0,
            z: 0,
        };
        ignored.tick(BlockPos { x: 0, y: 0, z: 0 });
        assert_eq!(ignored.cursors.len(), 31);
    }

    #[test]
    fn beehive_block_entity_persists_occupants_releases_and_increments_honey_like_java() {
        assert_eq!(BeehiveBlockEntity::MAX_OCCUPANTS, 3);
        assert_eq!(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTAR, 2400);
        assert_eq!(BeehiveBlockEntity::MIN_OCCUPATION_TICKS_NECTARLESS, 600);
        assert_eq!(BeehiveBlockEntity::MIN_TICKS_BEFORE_REENTERING_HIVE, 400);
        assert_eq!(BeehiveBlockEntity::MAX_HONEY_LEVEL, 5);
        assert_eq!(BeehiveBlockEntity::WORK_SOUND_CHANCE, 0.005);

        let mut hive = BeehiveBlockEntity::new();
        assert!(hive.is_empty());
        assert!(hive.add_occupant(
            BeehiveOccupant::bee(600, false),
            Some(BlockPos { x: 2, y: 70, z: -3 })
        ));
        assert!(hive.add_occupant(BeehiveOccupant::bee(2400, true), None));
        assert!(hive.add_occupant(BeehiveOccupant::bee(2401, true), None));
        assert!(!hive.add_occupant(BeehiveOccupant::bee(0, false), None));
        assert!(hive.is_full());
        assert_eq!(hive.occupant_count(), 3);
        assert_eq!(hive.saved_flower_pos, Some(BlockPos { x: 2, y: 70, z: -3 }));

        let saved = hive.save_additional();
        let loaded = BeehiveBlockEntity::load_additional(&saved);
        assert_eq!(loaded, hive);

        let blocked = hive.tick(true, false, false);
        assert!(blocked.is_empty());
        assert_eq!(hive.occupant_count(), 3);
        assert_eq!(hive.occupants[0].ticks_in_hive, 601);
        assert_eq!(hive.occupants[1].ticks_in_hive, 2401);
        assert_eq!(hive.occupants[2].ticks_in_hive, 2402);

        let released = hive.tick(false, true, false);
        assert!(released.is_empty());
        assert_eq!(hive.occupant_count(), 3);

        let released = hive.tick(false, false, false);
        assert_eq!(
            released,
            vec![
                BeeReleaseEvent {
                    entity_type: "minecraft:bee".to_string(),
                    status: BeeReleaseStatus::BeeReleased,
                    honey_level: 0,
                    stay_out_of_hive_ticks: 0,
                },
                BeeReleaseEvent {
                    entity_type: "minecraft:bee".to_string(),
                    status: BeeReleaseStatus::HoneyDelivered,
                    honey_level: 1,
                    stay_out_of_hive_ticks: 0,
                },
                BeeReleaseEvent {
                    entity_type: "minecraft:bee".to_string(),
                    status: BeeReleaseStatus::HoneyDelivered,
                    honey_level: 2,
                    stay_out_of_hive_ticks: 0,
                },
            ]
        );
        assert!(hive.is_empty());

        hive.honey_level = 4;
        assert!(hive.add_occupant(BeehiveOccupant::bee(2401, true), None));
        let released = hive.tick(false, false, true);
        assert_eq!(released[0].honey_level, 5);
        assert_eq!(hive.honey_level, 5);

        assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
        assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
        let emergency = hive.on_fire_nearby();
        assert_eq!(emergency.len(), 2);
        assert!(emergency
            .iter()
            .all(|event| event.status == BeeReleaseStatus::Emergency));
        assert!(hive.is_empty());

        assert!(hive.add_occupant(BeehiveOccupant::bee(0, false), None));
        let sedated = hive.empty_all_living_from_hive(BeeReleaseStatus::Emergency, true);
        assert_eq!(sedated[0].stay_out_of_hive_ticks, 400);
    }

    #[test]
    fn creaking_heart_block_entity_tracks_state_protector_and_output_like_java() {
        assert_eq!(CreakingHeartBlockEntity::PLAYER_DETECTION_RANGE, 32);
        assert_eq!(CreakingHeartBlockEntity::CREAKING_ROAMING_RADIUS, 32);
        assert_eq!(CreakingHeartBlockEntity::DISTANCE_CREAKING_TOO_FAR, 34.0);
        assert_eq!(CreakingHeartBlockEntity::SPAWN_RANGE_XZ, 16);
        assert_eq!(CreakingHeartBlockEntity::SPAWN_RANGE_Y, 8);
        assert_eq!(CreakingHeartBlockEntity::ATTEMPTS_PER_SPAWN, 5);
        assert_eq!(CreakingHeartBlockEntity::UPDATE_TICKS, 20);
        assert_eq!(CreakingHeartBlockEntity::UPDATE_TICKS_VARIANCE, 5);
        assert_eq!(CreakingHeartBlockEntity::HURT_CALL_TOTAL_TICKS, 100);
        assert_eq!(CreakingHeartBlockEntity::HURT_CALL_INTERVAL, 10);
        assert_eq!(CreakingHeartBlockEntity::HURT_CALL_PARTICLE_TICKS, 50);
        assert_eq!(CreakingHeartBlockEntity::MAX_RESIN_DEPTH, 2);
        assert_eq!(CreakingHeartBlockEntity::MAX_RESIN_COUNT, 64);
        assert_eq!(CreakingHeartBlockEntity::TICKS_GRACE_PERIOD, 30);

        let mut heart = CreakingHeartBlockEntity::new();
        heart.ticker = -1;
        let actions = heart.server_tick(true, true, true, true, false, None, false, false, 4);
        assert_eq!(
            actions,
            vec![
                CreakingHeartAction::StateChanged(CreakingHeartStateModel::Awake),
                CreakingHeartAction::SpawnProtector {
                    attempts: 5,
                    range_xz: 16,
                    range_y: 8,
                },
            ]
        );
        assert_eq!(heart.ticker, 24);
        assert_eq!(heart.state, CreakingHeartStateModel::Awake);

        heart.on_protector_spawned("00000000-0000-0000-0000-000000000001".to_string());
        assert_eq!(heart.compute_analog_output_signal(Some(0.0)), 15);
        assert_eq!(heart.compute_analog_output_signal(Some(16.0)), 8);
        assert_eq!(heart.compute_analog_output_signal(Some(32.0)), 0);
        assert_eq!(heart.compute_analog_output_signal(Some(64.0)), 0);

        let saved = heart.save_additional();
        assert_eq!(
            CreakingHeartBlockEntity::load_additional(&saved).creaking_uuid,
            heart.creaking_uuid
        );

        let hurt = heart.creaking_hurt(true, 3);
        assert_eq!(
            hurt,
            CreakingHeartAction::HurtPulse {
                total_ticks: 100,
                particle_ticks: 50,
                resin_clumps: 3,
            }
        );
        assert_eq!(heart.emitter_ticks, 100);
        assert_eq!(heart.creaking_hurt(true, 2), CreakingHeartAction::None);
        heart.server_tick(true, true, true, true, true, Some(4.0), false, false, 0);
        assert_eq!(heart.emitter_ticks, 99);

        heart.ticker = -1;
        let actions = heart.server_tick(true, false, true, true, true, Some(35.0), false, false, 0);
        assert!(actions.contains(&CreakingHeartAction::StateChanged(
            CreakingHeartStateModel::Dormant
        )));
        assert!(actions.contains(&CreakingHeartAction::RemoveProtector));
        assert!(heart.creaking_uuid.is_none());

        heart.state = CreakingHeartStateModel::Dormant;
        heart.ticker = -1;
        let actions = heart.server_tick(false, true, true, true, false, None, false, false, 0);
        assert_eq!(
            actions,
            vec![CreakingHeartAction::StateChanged(
                CreakingHeartStateModel::Uprooted
            )]
        );

        let mut unresolved = CreakingHeartBlockEntity::load_additional(&Tag::Compound(vec![(
            "creaking".to_string(),
            Tag::String("00000000-0000-0000-0000-000000000002".to_string()),
        )]));
        unresolved.ticks_existed = 29;
        unresolved.ticker = -1;
        assert!(unresolved
            .server_tick(true, true, true, true, false, None, false, false, 0)
            .contains(&CreakingHeartAction::RemoveProtector));
        assert!(unresolved.creaking_uuid.is_none());
    }

    #[test]
    fn sculk_shrieker_block_entity_tracks_warning_shriek_and_warden_response() {
        assert_eq!(SculkShriekerBlockEntity::LISTENER_RADIUS, 8);
        assert_eq!(SculkShriekerBlockEntity::WARNING_SOUND_RADIUS, 10);
        assert_eq!(SculkShriekerBlockEntity::SHRIEKING_TICKS, 90);
        assert_eq!(SculkShriekerBlockEntity::DARKNESS_RADIUS, 40);
        assert_eq!(SculkShriekerBlockEntity::WARDEN_SUMMON_WARNING_LEVEL, 4);
        assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_ATTEMPTS, 20);
        assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_RANGE_XZ, 5);
        assert_eq!(SculkShriekerBlockEntity::WARDEN_SPAWN_RANGE_Y, 6);

        let mut shrieker = SculkShriekerBlockEntity::new(true);
        assert!(shrieker.can_receive_vibration(false, true));
        assert!(!shrieker.can_receive_vibration(false, false));
        assert!(!shrieker.can_receive_vibration(true, true));
        assert_eq!(
            shrieker.try_shriek(false, true, Some(1), false),
            SculkShriekResult::Ignored
        );
        assert_eq!(
            shrieker.try_shriek(true, true, None, false),
            SculkShriekResult::Ignored
        );

        assert_eq!(
            shrieker.try_shriek(true, true, Some(3), false),
            SculkShriekResult::ReplySound {
                warning_level: 3,
                darkness_radius: 40,
            }
        );
        assert_eq!(shrieker.warning_level, 3);
        assert_eq!(shrieker.shrieking_ticks, 90);
        assert_eq!(
            shrieker.try_shriek(true, true, Some(4), true),
            SculkShriekResult::Ignored
        );
        assert_eq!(shrieker.tick(), SculkShriekResult::Ignored);
        assert_eq!(shrieker.shrieking_ticks, 89);

        shrieker.shrieking_ticks = 0;
        assert_eq!(
            shrieker.try_shriek(true, true, Some(4), true),
            SculkShriekResult::SummonWarden {
                warning_level: 4,
                attempts: 20,
                range_xz: 5,
                range_y: 6,
                darkness_radius: 40,
            }
        );

        let saved = shrieker.save_additional();
        let loaded = SculkShriekerBlockEntity::load_additional(&saved);
        assert_eq!(loaded.warning_level, 4);
        assert_eq!(loaded.shrieking_ticks, 0);
        assert!(!loaded.can_summon);

        let mut disabled = SculkShriekerBlockEntity::new(false);
        assert_eq!(
            disabled.try_shriek(true, false, None, false),
            SculkShriekResult::Shriek { warning_level: 0 }
        );
        assert_eq!(disabled.shrieking_ticks, 90);
    }

    #[test]
    fn bell_block_entity_tracks_ring_resonation_and_raider_glow_like_java() {
        assert_eq!(BellBlockEntity::DURATION, 50);
        assert_eq!(BellBlockEntity::GLOW_DURATION, 60);
        assert_eq!(BellBlockEntity::MIN_TICKS_BETWEEN_SEARCHES, 60);
        assert_eq!(BellBlockEntity::MAX_RESONATION_TICKS, 40);
        assert_eq!(BellBlockEntity::TICKS_BEFORE_RESONATION, 5);
        assert_eq!(BellBlockEntity::SEARCH_RADIUS, 48.0);
        assert_eq!(BellBlockEntity::HEAR_BELL_RADIUS, 32.0);
        assert_eq!(BellBlockEntity::HIGHLIGHT_RAIDERS_RADIUS, 48.0);

        let mut bell = BellBlockEntity::new();
        let block_event = bell.on_hit(Direction::North);
        assert_eq!(
            block_event,
            BellBlockEvent {
                event_id: BellBlockEntity::EVENT_RING,
                event_param: 2,
            }
        );
        assert!(bell.shaking);
        assert_eq!(bell.click_direction, Some(Direction::North));

        bell.ticks = 12;
        assert_eq!(bell.on_hit(Direction::East).event_param, 5);
        assert_eq!(bell.ticks, 0);
        assert_eq!(bell.click_direction, Some(Direction::East));

        assert!(bell.trigger_event(1, 3, 100, 4, 1, 2));
        assert_eq!(bell.click_direction, Some(Direction::South));
        assert_eq!(bell.last_ring_timestamp, 100);
        assert_eq!(bell.heard_bell_entities, 4);
        assert_eq!(bell.nearby_raiders_within_hear_radius, 1);
        assert_eq!(bell.nearby_raiders_within_highlight_radius, 2);
        assert_eq!(bell.ticks, 0);
        assert!(bell.shaking);
        assert!(!bell.trigger_event(99, 0, 100, 0, 0, 0));

        for _ in 0..4 {
            assert_eq!(
                bell.tick(),
                BellTickEffects {
                    play_resonate_sound: false,
                    glowing_raiders: 0,
                }
            );
        }
        assert_eq!(bell.ticks, 4);
        assert_eq!(
            bell.tick(),
            BellTickEffects {
                play_resonate_sound: true,
                glowing_raiders: 0,
            }
        );
        assert!(bell.resonating);
        assert_eq!(bell.resonation_ticks, 1);

        for _ in 0..39 {
            let effects = bell.tick();
            assert!(!effects.play_resonate_sound);
            assert_eq!(effects.glowing_raiders, 0);
        }
        assert_eq!(bell.resonation_ticks, 40);
        assert_eq!(
            bell.tick(),
            BellTickEffects {
                play_resonate_sound: false,
                glowing_raiders: 2,
            }
        );
        assert!(!bell.resonating);

        while bell.shaking {
            bell.tick();
        }
        assert_eq!(bell.ticks, 0);
        assert_eq!(bell.save_additional(), Tag::Compound(vec![]));
        assert_eq!(bell.get_update_tag(), Tag::Compound(vec![]));

        let mut cached = bell.clone();
        cached.update_entities(120, 7, 3, 5);
        assert_eq!(cached.heard_bell_entities, 4);
        cached.update_entities(161, 7, 3, 5);
        assert_eq!(cached.heard_bell_entities, 7);
        assert_eq!(cached.nearby_raiders_within_highlight_radius, 5);
    }

    #[test]
    fn save_modes_match_metadata_and_custom_data_boundaries() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        entity
            .custom_data
            .insert("front_text".to_string(), Tag::String("hello".to_string()));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Name\"".to_string()),
        );

        assert_eq!(
            entity.save_custom_only(),
            Tag::Compound(vec![(
                "front_text".to_string(),
                Tag::String("hello".to_string())
            )])
        );
        assert!(
            matches!(entity.save_without_metadata(), Tag::Compound(values) if values.iter().any(|(k, _)| k == "components") && values.iter().all(|(k, _)| k != "id"))
        );
        assert!(
            matches!(entity.save_with_id(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "id" && *v == Tag::String("sign".to_string())) && values.iter().all(|(k, _)| k != "x"))
        );
        assert!(
            matches!(entity.save_with_full_metadata(), Tag::Compound(values) if values.iter().any(|(k, v)| k == "x" && *v == Tag::Int(18)))
        );
    }

    #[test]
    fn load_static_reads_id_components_and_custom_payload() {
        let tag = Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:campfire".to_string()),
            ),
            ("x".to_string(), Tag::Int(18)),
            ("y".to_string(), Tag::Int(64)),
            ("z".to_string(), Tag::Int(35)),
            ("CookingTimes".to_string(), Tag::List(vec![Tag::Int(10)])),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:lore".to_string(),
                    Tag::String("[]".to_string()),
                )]),
            ),
        ]);
        let entity = load_static(pos(), "minecraft:campfire", &tag).unwrap();
        assert_eq!(entity.ty, BlockEntityTypeId::Campfire);
        assert!(entity.custom_data.contains_key("CookingTimes"));
        assert!(entity.components.contains_key("minecraft:lore"));
    }

    #[test]
    fn load_static_with_data_version_refuses_unsafe_migrations() {
        let tag = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:campfire".to_string()),
        )]);

        let entity = load_static_with_data_version(
            pos(),
            "minecraft:campfire",
            &tag,
            crate::storage::datafix::TARGET_DATA_VERSION,
        )
        .unwrap();
        assert_eq!(entity.ty, BlockEntityTypeId::Campfire);

        assert!(matches!(
            load_static_with_data_version(
                pos(),
                "minecraft:campfire",
                &tag,
                crate::storage::datafix::TARGET_DATA_VERSION - 1,
            ),
            Err(BlockEntityError::UnsupportedDataVersion(message))
                if message.contains("unsafe migrations")
        ));
    }

    #[test]
    fn save_load_round_trip_preserves_generic_fields_for_every_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity
                .custom_data
                .insert("CustomInt".to_string(), Tag::Int(42));
            entity.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String("\"Round Trip\"".to_string()),
            );

            let tag = entity.save_with_full_metadata();
            let loaded = load_static(pos(), block_state, &tag).unwrap();

            assert_eq!(loaded.ty, entity.ty, "type failed for {}", info.key);
            assert_eq!(loaded.pos, entity.pos, "position failed for {}", info.key);
            assert_eq!(
                loaded.block_state, entity.block_state,
                "block state failed for {}",
                info.key
            );
            assert_eq!(
                loaded.custom_data, entity.custom_data,
                "custom data failed for {}",
                info.key
            );
            assert_eq!(
                loaded.components, entity.components,
                "components failed for {}",
                info.key
            );
            assert!(
                !loaded.has_level,
                "level attachment leaked for {}",
                info.key
            );
            assert!(!loaded.removed, "removed flag leaked for {}", info.key);
            assert!(!loaded.changed, "changed flag leaked for {}", info.key);
            assert_eq!(loaded.tick_count, 0, "tick count leaked for {}", info.key);
        }
    }

    #[test]
    fn placement_then_chunk_unload_reload_preserves_block_entity_nbt_for_every_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let placed_pos = BlockPos {
                x: 18,
                y: 73,
                z: -29,
            };
            let mut placed = BlockEntity::new(info.id, placed_pos, block_state).unwrap();
            placed.set_level();
            placed.set_changed();
            placed.tick_count = 99;
            placed
                .custom_data
                .insert("ChunkUnloadProbe".to_string(), Tag::Long(123_456));
            placed.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String(format!("\"{}\"", info.key)),
            );

            let saved_at_unload = placed.save_with_full_metadata();
            let reloaded = load_static(placed_pos, block_state, &saved_at_unload).unwrap();
            let saved_after_reload = reloaded.save_with_full_metadata();

            assert_eq!(
                saved_after_reload, saved_at_unload,
                "chunk unload/reload NBT identity failed for {}",
                info.key
            );
            assert!(
                !reloaded.has_level && !reloaded.changed && !reloaded.removed,
                "runtime placement flags leaked through chunk reload for {}",
                info.key
            );
            assert_eq!(
                reloaded.tick_count, 0,
                "scheduler tick state leaked through chunk reload for {}",
                info.key
            );
        }
    }

    #[test]
    fn data_get_block_exposes_full_nbt_for_every_block_entity_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let probe_pos = BlockPos {
                x: -12,
                y: 81,
                z: 44,
            };
            let mut entity = BlockEntity::new(info.id, probe_pos, block_state).unwrap();
            entity
                .custom_data
                .insert("DataProbe".to_string(), Tag::String(info.key.to_string()));
            entity.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String("\"Data Probe\"".to_string()),
            );

            let tag = entity.data_get_block_nbt();
            let entries = compound_entries(&tag).expect("/data get block result is compound");

            assert_eq!(
                get_string(entries, "id"),
                Some(info.key),
                "/data id failed for {}",
                info.key
            );
            assert_eq!(get_int(entries, "x"), Some(probe_pos.x));
            assert_eq!(get_int(entries, "y"), Some(probe_pos.y));
            assert_eq!(get_int(entries, "z"), Some(probe_pos.z));
            assert!(
                entries.iter().any(|(key, value)| key == "DataProbe"
                    && value == &Tag::String(info.key.to_string())),
                "/data custom field missing for {}",
                info.key
            );
            assert!(
                entries.iter().any(|(key, value)| {
                    key == "components"
                        && matches!(value, Tag::Compound(values) if values.iter().any(
                            |(component_key, component_value)| component_key == "minecraft:custom_name"
                                && component_value == &Tag::String("\"Data Probe\"".to_string())
                        ))
                }),
                "/data components missing for {}",
                info.key
            );

            let loaded = load_static(probe_pos, block_state, &tag).unwrap();
            assert_eq!(
                loaded.custom_data, entity.custom_data,
                "{} custom data",
                info.key
            );
            assert_eq!(
                loaded.components, entity.components,
                "{} components",
                info.key
            );
        }
    }

    #[test]
    fn destruction_drops_cover_tool_silk_explosion_gamerule_and_stored_items_for_every_type() {
        let correct_tool = BlockEntityDestructionContext {
            correct_tool: true,
            silk_touch: false,
            explosion_survives: true,
            do_tile_drops: true,
        };
        let silk_touch = BlockEntityDestructionContext {
            correct_tool: false,
            silk_touch: true,
            explosion_survives: true,
            do_tile_drops: true,
        };
        let wrong_tool = BlockEntityDestructionContext {
            correct_tool: false,
            silk_touch: false,
            explosion_survives: true,
            do_tile_drops: true,
        };
        let explosion_consumed = BlockEntityDestructionContext {
            correct_tool: true,
            silk_touch: false,
            explosion_survives: false,
            do_tile_drops: true,
        };
        let tile_drops_disabled = BlockEntityDestructionContext {
            correct_tool: true,
            silk_touch: false,
            explosion_survives: true,
            do_tile_drops: false,
        };

        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity.custom_data.insert(
                "Items".to_string(),
                Tag::List(vec![stack("minecraft:apple", 2).to_tag()]),
            );
            entity
                .custom_data
                .insert("item".to_string(), stack("minecraft:diamond", 1).to_tag());
            entity.custom_data.insert(
                "Book".to_string(),
                stack("minecraft:written_book", 1).to_tag(),
            );
            entity.custom_data.insert(
                "RecordItem".to_string(),
                stack("minecraft:music_disc_13", 1).to_tag(),
            );

            assert_eq!(
                entity.destruction_drops(correct_tool).block_item.as_deref(),
                Some(block_item_from_state(block_state)),
                "{} correct-tool block drop",
                info.key
            );
            assert_eq!(
                entity.destruction_drops(silk_touch).block_item.as_deref(),
                Some(block_item_from_state(block_state)),
                "{} silk-touch block drop",
                info.key
            );

            let wrong_tool_drops = entity.destruction_drops(wrong_tool);
            assert_eq!(
                wrong_tool_drops.block_item, None,
                "{} wrong-tool block drop",
                info.key
            );
            assert_eq!(
                wrong_tool_drops.stored_items,
                vec![
                    stack("minecraft:apple", 2),
                    stack("minecraft:diamond", 1),
                    stack("minecraft:written_book", 1),
                    stack("minecraft:music_disc_13", 1),
                ],
                "{} stored item drops",
                info.key
            );

            assert_eq!(
                entity.destruction_drops(explosion_consumed),
                BlockEntityDestructionDrops {
                    block_item: None,
                    stored_items: Vec::new(),
                },
                "{} explosion consumed drops",
                info.key
            );
            assert_eq!(
                entity.destruction_drops(tile_drops_disabled),
                BlockEntityDestructionDrops {
                    block_item: None,
                    stored_items: Vec::new(),
                },
                "{} doTileDrops=false drops",
                info.key
            );
        }
    }

    #[test]
    fn update_tag_subset_is_stable_for_every_type() {
        for info in BLOCK_ENTITY_TYPES {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has at least one valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity
                .custom_data
                .insert("CustomInt".to_string(), Tag::Int(42));
            entity.components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String("\"Update Tag\"".to_string()),
            );

            let tag = entity.get_update_tag();
            let entries = compound_entries(&tag).expect("update tag is compound");

            assert!(
                entries
                    .iter()
                    .all(|(key, _)| key != "id" && key != "x" && key != "y" && key != "z"),
                "metadata leaked into update tag for {}",
                info.key
            );

            match info.id {
                BlockEntityTypeId::Chest
                | BlockEntityTypeId::TrappedChest
                | BlockEntityTypeId::Barrel
                | BlockEntityTypeId::Hopper
                | BlockEntityTypeId::Dispenser
                | BlockEntityTypeId::Dropper => {
                    assert!(
                        entries.is_empty(),
                        "container inventory data leaked into update tag for {}",
                        info.key
                    );
                }
                _ => {
                    assert!(
                        entries
                            .iter()
                            .any(|(key, value)| key == "CustomInt" && *value == Tag::Int(42)),
                        "custom data missing from update tag for {}",
                        info.key
                    );
                    assert!(
                        entries.iter().any(|(key, _)| key == "components"),
                        "components missing from update tag for {}",
                        info.key
                    );
                }
            }
        }
    }

    #[test]
    fn wrong_chunk_positions_are_corrected_like_vanilla() {
        let tag = Tag::Compound(vec![
            ("x".to_string(), Tag::Int(34)),
            ("y".to_string(), Tag::Int(-20)),
            ("z".to_string(), Tag::Int(-17)),
        ]);
        assert_eq!(
            corrected_pos_from_chunk(0, 0, &tag),
            BlockPos {
                x: 2,
                y: -20,
                z: 15
            }
        );
    }

    #[test]
    fn ticking_requires_level_side_match_and_not_removed() {
        let mut furnace =
            BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
        assert!(!furnace.tick(false));
        furnace.set_level();
        assert!(furnace.tick(false));
        assert!(!furnace.tick(true));
        furnace.set_removed();
        assert!(!furnace.tick(false));

        let mut conduit =
            BlockEntity::new(BlockEntityTypeId::Conduit, pos(), "minecraft:conduit").unwrap();
        conduit.set_level();
        assert!(conduit.tick(false));
        assert!(conduit.tick(true));
    }

    #[test]
    fn tick_dispatch_advances_scheduler_state_for_every_tickable_block_entity_type() {
        let tickable: Vec<&BlockEntityTypeInfo> = BLOCK_ENTITY_TYPES
            .iter()
            .filter(|info| info.tick_kind != BlockEntityTickKind::None)
            .collect();
        assert_eq!(tickable.len(), 32);

        for info in tickable {
            let block_state = info
                .valid_blocks
                .first()
                .expect("every block entity type has a valid block");
            let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
            entity.set_level();

            let server_ticks = matches!(
                info.tick_kind,
                BlockEntityTickKind::Server | BlockEntityTickKind::Both
            );
            let client_ticks = matches!(
                info.tick_kind,
                BlockEntityTickKind::Client | BlockEntityTickKind::Both
            );

            assert_eq!(
                entity.tick(false),
                server_ticks,
                "{} server tick dispatch",
                info.key
            );
            assert_eq!(
                entity.tick_count,
                u64::from(server_ticks),
                "{} server tick count",
                info.key
            );
            assert_eq!(
                entity.tick(true),
                client_ticks,
                "{} client tick dispatch",
                info.key
            );
            assert_eq!(
                entity.tick_count,
                u64::from(server_ticks) + u64::from(client_ticks),
                "{} client tick count",
                info.key
            );

            entity.set_removed();
            assert!(
                !entity.tick(false) && !entity.tick(true),
                "{} removed entity ticked",
                info.key
            );
        }
    }

    #[test]
    fn ticking_block_entity_wrapper_exposes_scheduler_shape() {
        let mut furnace =
            BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
        furnace.set_level();
        let mut ticker = TickingBlockEntity::new(furnace, false);

        assert_eq!(ticker.pos(), pos());
        assert_eq!(ticker.type_key(), "furnace");
        assert!(!ticker.is_removed());
        assert!(ticker.tick());
        assert_eq!(ticker.entity.tick_count, 1);

        ticker.entity.set_removed();
        assert!(ticker.is_removed());
        assert!(!ticker.tick());
    }

    #[test]
    fn changed_flag_only_sets_when_attached_to_level() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Bell, pos(), "minecraft:bell").unwrap();
        entity.set_changed();
        assert!(!entity.changed);
        entity.set_level();
        entity.set_changed();
        assert!(entity.changed);
    }

    #[test]
    fn update_packets_use_position_type_and_update_tag() {
        let mut sign =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        sign.custom_data
            .insert("front_text".to_string(), Tag::String("hi".to_string()));
        let packet = sign.get_update_packet();
        assert_eq!(packet.pos, pos());
        assert_eq!(packet.ty, BlockEntityTypeId::Sign);
        assert!(
            matches!(packet.tag, Tag::Compound(values) if values.iter().any(|(k, _)| k == "front_text"))
        );

        let chest = BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").unwrap();
        assert_eq!(chest.get_update_tag(), Tag::Compound(Vec::new()));
    }

    #[test]
    fn handle_update_tag_applies_network_subset_without_metadata() {
        let mut entity =
            BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
        entity
            .custom_data
            .insert("old_text".to_string(), Tag::String("stale".to_string()));
        entity
            .components
            .insert("old_component".to_string(), Tag::Int(1));

        entity.handle_update_tag(&Tag::Compound(vec![
            ("x".to_string(), Tag::Int(999)),
            ("id".to_string(), Tag::String("minecraft:chest".to_string())),
            ("front_text".to_string(), Tag::String("hello".to_string())),
            (
                "components".to_string(),
                Tag::Compound(vec![(
                    "minecraft:custom_name".to_string(),
                    Tag::String("Sign".to_string()),
                )]),
            ),
        ]));

        assert_eq!(entity.ty, BlockEntityTypeId::Sign);
        assert_eq!(entity.pos, pos());
        assert!(!entity.custom_data.contains_key("old_text"));
        assert_eq!(
            entity.custom_data.get("front_text"),
            Some(&Tag::String("hello".to_string()))
        );
        assert!(!entity.components.contains_key("old_component"));
        assert_eq!(
            entity.components.get("minecraft:custom_name"),
            Some(&Tag::String("Sign".to_string()))
        );
    }

    #[test]
    fn test_block_entity_state_saves_loads_and_tracks_triggers_like_java() {
        let mut state = TestBlockEntityState {
            mode: TestBlockMode::Start,
            message: "begin".to_string(),
            powered: false,
            triggered: true,
        };
        assert_eq!(
            state.save_additional(),
            Tag::Compound(vec![
                ("mode".to_string(), Tag::String("start".to_string())),
                ("message".to_string(), Tag::String("begin".to_string())),
                ("powered".to_string(), Tag::Byte(0)),
            ])
        );

        state.trigger();
        assert!(state.powered);
        assert!(state.triggered);
        state.reset();
        assert!(!state.powered);
        assert!(!state.triggered);

        let loaded = TestBlockEntityState::load_additional(&Tag::Compound(vec![
            ("mode".to_string(), Tag::String("accept".to_string())),
            ("message".to_string(), Tag::String("done".to_string())),
            ("powered".to_string(), Tag::Byte(1)),
        ]));
        assert_eq!(loaded.mode, TestBlockMode::Accept);
        assert_eq!(loaded.message, "done");
        assert!(loaded.powered);
        assert!(!loaded.triggered);
        assert_eq!(
            TestBlockEntityState::load_additional(&Tag::Compound(Vec::new())).mode,
            TestBlockMode::Fail
        );
    }

    #[test]
    fn test_instance_block_entity_state_saves_loads_status_and_errors() {
        let mut state = TestInstanceBlockEntityState {
            data: TestInstanceBlockEntityData {
                test: Some("minecraft:always_pass".to_string()),
                size: (3, 4, 5),
                rotation: "clockwise_90".to_string(),
                ignore_entities: true,
                status: TestInstanceStatus::Cleared,
                error_message: None,
            },
            errors: Vec::new(),
        };
        state.set_running();
        state.mark_error(BlockPos { x: 1, y: 2, z: 3 }, "bad block");
        state.set_error_message("failed");

        let saved = state.save_additional();
        let loaded = TestInstanceBlockEntityState::load_additional(&saved);
        assert_eq!(loaded.data.test.as_deref(), Some("minecraft:always_pass"));
        assert_eq!(loaded.data.size, (3, 4, 5));
        assert_eq!(loaded.data.rotation, "clockwise_90");
        assert!(loaded.data.ignore_entities);
        assert_eq!(loaded.data.status, TestInstanceStatus::Finished);
        assert_eq!(loaded.data.error_message.as_deref(), Some("failed"));
        assert_eq!(
            loaded.errors,
            vec![TestInstanceErrorMarker {
                pos: BlockPos { x: 1, y: 2, z: 3 },
                text: "bad block".to_string(),
            }]
        );

        let mut success = loaded.clone();
        success.set_success();
        assert_eq!(success.data.status, TestInstanceStatus::Finished);
        assert_eq!(success.data.error_message, None);
        success.clear_error_markers();
        assert!(success.errors.is_empty());
    }

    #[test]
    fn chunk_packet_data_packs_local_xz_y_type_and_tag() {
        let entity = BlockEntity::new(BlockEntityTypeId::Vault, pos(), "minecraft:vault").unwrap();
        let (packed_xz, y, ty, tag) = block_entity_packet_from_chunk(&entity, -64);
        assert_eq!(packed_xz, 0x23);
        assert_eq!(y, 128);
        assert_eq!(ty, BlockEntityTypeId::Vault);
        assert_eq!(
            tag,
            Tag::Compound(vec![("components".to_string(), Tag::Compound(Vec::new()))])
        );
    }
}
