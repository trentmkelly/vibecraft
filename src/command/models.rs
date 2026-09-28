use super::*;
use crate::random_sequences::{RandomSequence, RandomSequences};

pub(super) const VANILLA_TRIM_PATTERNS: &[&str] = &[
    "minecraft:sentry",
    "minecraft:dune",
    "minecraft:coast",
    "minecraft:wild",
    "minecraft:ward",
    "minecraft:eye",
    "minecraft:vex",
    "minecraft:tide",
    "minecraft:snout",
    "minecraft:rib",
    "minecraft:spire",
    "minecraft:wayfinder",
    "minecraft:shaper",
    "minecraft:silence",
    "minecraft:raiser",
    "minecraft:host",
    "minecraft:flow",
    "minecraft:bolt",
];
pub(super) const VANILLA_TRIM_MATERIALS: &[&str] = &[
    "minecraft:quartz",
    "minecraft:iron",
    "minecraft:netherite",
    "minecraft:redstone",
    "minecraft:copper",
    "minecraft:gold",
    "minecraft:emerald",
    "minecraft:diamond",
    "minecraft:lapis",
    "minecraft:amethyst",
    "minecraft:resin",
];
pub(super) const TRIMMABLE_ARMOR_ITEMS: &[&str] = &[
    "minecraft:leather_helmet",
    "minecraft:leather_chestplate",
    "minecraft:leather_leggings",
    "minecraft:leather_boots",
    "minecraft:chainmail_helmet",
    "minecraft:chainmail_chestplate",
    "minecraft:chainmail_leggings",
    "minecraft:chainmail_boots",
    "minecraft:iron_helmet",
    "minecraft:iron_chestplate",
    "minecraft:iron_leggings",
    "minecraft:iron_boots",
    "minecraft:golden_helmet",
    "minecraft:golden_chestplate",
    "minecraft:golden_leggings",
    "minecraft:golden_boots",
    "minecraft:diamond_helmet",
    "minecraft:diamond_chestplate",
    "minecraft:diamond_leggings",
    "minecraft:diamond_boots",
    "minecraft:netherite_helmet",
    "minecraft:netherite_chestplate",
    "minecraft:netherite_leggings",
    "minecraft:netherite_boots",
    "minecraft:turtle_helmet",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    All = 0,
    Moderators = 1,
    Gamemasters = 2,
    Admins = 3,
    Owners = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    CommandLevel(PermissionLevel),
    CommandsEntitySelectors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelBasedPermissionSet {
    pub(super) level: PermissionLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAvailability {
    Available,
    Hidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerCommandState {
    pub player_idle_timeout_minutes: u32,
    pub autosave_enabled: bool,
    pub save_all_requests: Vec<SaveAllRequest>,
    pub save_all_should_fail: bool,
    pub side_feedback: Vec<CommandResult>,
    pub published_server: Option<PublishRequest>,
    pub publish_should_fail: bool,
    pub next_available_publish_port: u16,
    pub random_sequences: RandomSequences,
    pub level_random: Option<RandomSequence>,
    pub random_seed_defaults: RandomSeedDefaults,
    pub random_broadcasts: Vec<RandomSample>,
    pub available_data_packs: Vec<String>,
    pub selected_data_packs: Vec<String>,
    pub disabled_data_packs: Vec<String>,
    pub feature_data_packs: Vec<String>,
    pub unavailable_feature_data_packs: Vec<String>,
    pub created_data_packs: Vec<CreatedDataPack>,
    pub reload_requests: Vec<ReloadRequest>,
    pub transfer_requests: Vec<TransferRequest>,
    pub chase_session: Option<ChaseSession>,
    pub chase_events: Vec<ChaseEvent>,
    pub perf_recording: bool,
    pub perf_reports: Vec<PerfReport>,
    pub perf_report_should_fail: bool,
    pub debug_profiler_running: bool,
    pub debug_profiler_results: Vec<DebugProfilerResult>,
    pub debug_trace_events: Vec<DebugTraceEvent>,
    pub config_players: Vec<NameAndId>,
    pub config_dialog_events: Vec<DebugConfigDialogEvent>,
    pub dialog_events: Vec<DialogCommandEvent>,
    pub active_effects: Vec<ActiveEffect>,
    pub mob_spawning_events: Vec<DebugMobSpawningEvent>,
    pub debug_path_events: Vec<DebugPathEvent>,
    pub unreachable_debug_paths: Vec<BlockPos>,
    pub incomplete_debug_paths: Vec<BlockPos>,
    pub jfr_recording: bool,
    pub jfr_recordings: Vec<String>,
    pub next_jfr_recording_path: String,
    pub known_recipes: Vec<String>,
    pub player_recipes: Vec<PlayerRecipeBook>,
    pub advancements: Vec<AdvancementDefinition>,
    pub player_advancements: Vec<PlayerAdvancementProgress>,
    pub advancement_flush_events: Vec<AdvancementFlushEvent>,
    pub entity_attributes: Vec<EntityAttributeState>,
    pub fetched_profiles: Vec<FetchProfileEvent>,
    pub avatar_profiles: Vec<AvatarProfile>,
    pub bossbars: Vec<CustomBossBar>,
    pub command_time_millis: u64,
    pub game_time_ticks: u64,
    pub world_clock_ticks: i64,
    pub world_clock_paused: bool,
    pub world_clock_rate: f32,
    pub world_preset: String,
    pub stopwatches: Vec<StopwatchState>,
    pub scheduled_functions: Vec<ScheduledFunction>,
    pub available_functions: Vec<CommandFunctionDefinition>,
    pub function_tags: Vec<CommandFunctionTag>,
    pub queued_functions: Vec<QueuedFunctionCall>,
    pub macro_functions: Vec<String>,
    pub macro_entity_nbt_sources: Vec<CommandEntityNbtSource>,
    pub macro_block_nbt_sources: Vec<CommandBlockNbtSource>,
    pub macro_storage_nbt_sources: Vec<CommandStorageNbtSource>,
    pub function_permission_level: PermissionLevel,
    pub command_source_player: Option<NameAndId>,
    pub command_source_entity: Option<EntityRef>,
    pub command_source_position: Vec3,
    pub command_source_yaw: f32,
    pub command_source_pitch: f32,
    pub command_source_dimension: String,
    pub execute_events: Vec<ExecuteCommandEvent>,
    pub debug_world: bool,
    pub blocks: Vec<BlockStateEntry>,
    pub biomes: Vec<BiomeEntry>,
    pub clone_events: Vec<CloneEvent>,
    pub fill_events: Vec<FillEvent>,
    pub fill_biome_events: Vec<FillBiomeEvent>,
    pub forced_chunks: Vec<ForcedChunk>,
    pub locatable_entries: Vec<CommandLocatableEntry>,
    pub locate_results: Vec<CommandLocateResult>,
    pub max_block_modifications: i32,
    pub online_players: Vec<NameAndId>,
    pub last_list_includes_uuids: bool,
    pub player_inventories: Vec<CommandPlayerInventory>,
    pub entity_item_slots: Vec<CommandEntityItemSlot>,
    pub block_item_slots: Vec<CommandBlockItemSlot>,
    pub item_modifier_events: Vec<CommandItemModifierEvent>,
    pub item_enchantments: Vec<CommandItemEnchantment>,
    pub command_loot_tables: Vec<CommandLootTable>,
    pub entity_loot_tables: Vec<CommandEntityLootTable>,
    pub loot_events: Vec<CommandLootEvent>,
    pub available_templates: Vec<String>,
    pub place_events: Vec<CommandPlaceEvent>,
    pub raids: Vec<CommandRaidState>,
    pub raid_events: Vec<CommandRaidEvent>,
    pub player_game_modes: Vec<PlayerGameMode>,
    pub player_experience: Vec<PlayerExperienceState>,
    pub default_game_mode: GameMode,
    pub force_game_mode: Option<GameMode>,
    pub difficulty: Difficulty,
    pub game_rules: Vec<GameRuleState>,
    pub game_rule_syncs: Vec<GameRuleSyncEvent>,
    /// Translation arguments of the last successful command's feedback message
    /// (`Component.translatable(key, args...)`), e.g. `[rule, value]` for `commands.gamerule.set`.
    pub feedback_args: Vec<String>,
    pub camera_targets: Vec<CameraTarget>,
    pub untrackable_entities: Vec<EntityRef>,
    pub max_players: u32,
    pub singleplayer_owner: Option<NameAndId>,
    pub disconnected_players: Vec<PlayerDisconnect>,
    pub online_player_addresses: Vec<PlayerIpAddress>,
    pub banned_players: Vec<BanEntry<NameAndId>>,
    pub banned_ips: Vec<BanEntry<String>>,
    pub operator_players: Vec<NameAndId>,
    pub killed_entities: Vec<EntityRef>,
    pub ban_player_feedback_events: Vec<BanPlayerFeedbackEvent>,
    pub ban_ip_feedback_events: Vec<BanIpFeedbackEvent>,
    pub teams: Vec<TeamState>,
    pub player_teams: Vec<TeamMembership>,
    pub scoreboard_objectives: Vec<ScoreboardObjective>,
    pub scoreboard_scores: Vec<ScoreboardScore>,
    pub scoreboard_display_slots: Vec<ScoreboardDisplaySlot>,
    pub chat_events: Vec<ChatCommandEvent>,
    pub title_events: Vec<TitleCommandEvent>,
    pub sound_events: Vec<SoundCommandEvent>,
    pub particle_events: Vec<ParticleCommandEvent>,
    pub warden_spawn_trackers: Vec<WardenSpawnTrackerState>,
    pub waypoints: Vec<WaypointState>,
    pub world_border: WorldBorder,
    pub setblock_events: Vec<SetBlockEvent>,
    pub server_pack_events: Vec<ServerPackCommandEvent>,
    pub summoned_entities: Vec<SummonedEntity>,
    pub armor_trim_spawns: Vec<ArmorTrimSpawn>,
    pub swing_events: Vec<SwingCommandEvent>,
    pub rotation_requests: Vec<RotationRequest>,
    pub return_events: Vec<ReturnCommandEvent>,
    pub ride_events: Vec<RideCommandEvent>,
    pub damage_events: Vec<DamageCommandEvent>,
    pub invulnerable_entities: Vec<EntityRef>,
    pub entity_mounts: Vec<EntityMount>,
    pub ride_mount_failures: Vec<EntityMount>,
    pub entity_positions: Vec<EntityPosition>,
    pub teleport_side_effects: Vec<TeleportSideEffect>,
    pub entity_states: Vec<EntityState>,
    pub entity_tags: Vec<EntityTags>,
    pub world_spawn: RespawnData,
    pub player_spawns: Vec<PlayerSpawn>,
    pub weather: WeatherState,
    pub whitelist_enabled: bool,
    pub whitelisted_players: Vec<NameAndId>,
    pub whitelist_reload_requests: u32,
    pub kick_unlisted_requests: u32,
    pub tick_rate: TickRateController,
    pub tick_feedback_events: Vec<TickCommandFeedbackEvent>,
    pub average_tick_time_nanos: u64,
    pub tick_time_samples_nanos: Vec<u64>,
    pub halt_requested: bool,
    pub world_seed: i64,
    pub version: VersionInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveAllRequest {
    pub flush: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    pub host: String,
    pub port: u16,
    pub targets: Vec<NameAndId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerIpAddress {
    pub player: NameAndId,
    pub ip: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChaseSession {
    Leading { bind_address: String, port: u16 },
    Following { host: String, port: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChaseEvent {
    LeadStarted { bind_address: String, port: u16 },
    FollowStarted { host: String, port: u16 },
    LeadStopped,
    FollowStopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPlayerInventory {
    pub player: NameAndId,
    pub items: Vec<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItemStack {
    pub item: String,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLootTable {
    pub id: String,
    pub drops: Vec<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntityLootTable {
    pub entity: EntityRef,
    pub table: String,
    pub drops: Vec<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandLootEvent {
    pub target: CommandLootTarget,
    pub source: CommandLootSource,
    pub drops: Vec<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandLootTarget {
    Give {
        players: Vec<NameAndId>,
    },
    Spawn {
        position: Vec3,
    },
    Insert {
        pos: BlockPos,
    },
    ReplaceEntity {
        entities: Vec<EntityRef>,
        slot: String,
        count: usize,
    },
    ReplaceBlock {
        pos: BlockPos,
        slot: String,
        count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandLootSource {
    LootTable {
        table: String,
    },
    Fish {
        table: String,
        pos: BlockPos,
        tool: Option<String>,
    },
    Kill {
        entity: EntityRef,
        table: String,
    },
    Mine {
        pos: BlockPos,
        block: String,
        tool: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandPlaceEvent {
    pub kind: PlaceKind,
    pub id: String,
    pub position: BlockPos,
    pub rotation: Option<String>,
    pub mirror: Option<String>,
    pub integrity: Option<f32>,
    pub seed: Option<i32>,
    pub strict: bool,
    pub target: Option<String>,
    pub max_depth: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceKind {
    Feature,
    Jigsaw,
    Structure,
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRaidState {
    pub center: BlockPos,
    pub omen_level: i32,
    pub groups_spawned: i32,
    pub raiders_alive: i32,
    pub health: i32,
    pub total_health: i32,
    pub stopped: bool,
    pub glowing: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandRaidEvent {
    Sound {
        sound: &'static str,
        source: SoundSource,
        position: Vec3,
        volume: f32,
        pitch: f32,
    },
    SpawnLeader {
        entity_type: &'static str,
        position: Vec3,
        patrol_leader: bool,
        head_item: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEntityItemSlot {
    pub entity: EntityRef,
    pub slot: String,
    pub item: Option<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandBlockItemSlot {
    pub pos: BlockPos,
    pub slot: String,
    pub item: Option<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItemModifierEvent {
    pub target: CommandItemTarget,
    pub modifier: String,
    pub input: Option<CommandItemStack>,
    pub output: Option<CommandItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandItemTarget {
    Entity { entity: EntityRef, slot: String },
    Block { pos: BlockPos, slot: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItemEnchantment {
    pub target: EntityRef,
    pub item: String,
    pub enchantment: String,
    pub level: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerExperienceState {
    pub player: NameAndId,
    pub level: i32,
    pub progress: f32,
    pub total: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchProfileEvent {
    pub query: FetchProfileQuery,
    pub profile: NameAndId,
    pub encoded_profile: String,
    pub encoded_head_component: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchProfileQuery {
    Name(String),
    Id(String),
    Entity(EntityRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarProfile {
    pub entity: EntityRef,
    pub profile: NameAndId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteCommandEvent {
    pub sources: Vec<ExecuteSourceSnapshot>,
    pub command: String,
    pub result: i32,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecuteSourceSnapshot {
    pub entity: Option<EntityRef>,
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dimension: String,
    pub anchor: EntityAnchor,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CommandSourceSnapshot {
    pub(super) entity: Option<EntityRef>,
    pub(super) player: Option<NameAndId>,
    pub(super) position: Vec3,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) dimension: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishRequest {
    pub port: u16,
    pub allow_commands: bool,
    pub gamemode: Option<GameMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

pub use crate::game_rules::GameRuleValue;

/// One live game-rule value in the command state; the canonical id/type/default/bounds table
/// is `crate::game_rules::vanilla_game_rules` (mirrors `GameRules.java`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleState {
    pub name: String,
    pub value: GameRuleValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuleSyncEvent {
    pub rule: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSeedDefaults {
    pub salt: i32,
    pub include_world_seed: bool,
    pub include_sequence_id: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSample {
    pub value: i32,
    pub min: i32,
    pub max: i32,
    pub sequence: Option<String>,
    pub announced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRef {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandEntityNbtSource {
    pub entity: EntityRef,
    pub nbt: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockNbtSource {
    pub pos: BlockPos,
    pub nbt: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandStorageNbtSource {
    pub id: String,
    pub nbt: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDisconnect {
    pub player: NameAndId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanPlayerFeedbackEvent {
    pub player: NameAndId,
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BanIpFeedbackEvent {
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadRequest {
    pub selected_packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedDataPack {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerfReport {
    pub ticks: u32,
    pub duration_nanos: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugProfilerResult {
    pub duration_nanos: u64,
    pub tick_duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugTraceEvent {
    pub function: String,
    pub output: String,
    pub command_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugConfigDialogEvent {
    pub target: String,
    pub dialog: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogCommandEvent {
    Show {
        targets: Vec<NameAndId>,
        dialog: DialogCommandDialog,
    },
    Clear {
        targets: Vec<NameAndId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogCommandDialog {
    Reference(String),
    Inline { title: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEffect {
    pub target: EntityRef,
    pub effect: String,
    pub duration_ticks: i32,
    pub amplifier: u8,
    pub show_particles: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugMobSpawningEvent {
    pub category: String,
    pub position: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugPathEvent {
    pub source: EntityRef,
    pub target: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRecipeBook {
    pub player: NameAndId,
    pub recipes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementDefinition {
    pub id: String,
    pub parent: Option<String>,
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAdvancementProgress {
    pub player: NameAndId,
    pub advancement: String,
    pub completed_criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementFlushEvent {
    pub player: NameAndId,
    pub hide_advancement_toasts: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityAttributeState {
    pub target: String,
    pub attribute: String,
    pub default_base: f64,
    pub base: f64,
    pub modifiers: Vec<AttributeModifierState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifierState {
    pub id: String,
    pub value: f64,
    pub operation: AttributeOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperation {
    Value,
    MultipliedBase,
    MultipliedTotal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomBossBar {
    pub id: String,
    pub name: String,
    pub color: BossBarCommandColor,
    pub overlay: BossBarCommandOverlay,
    pub value: i32,
    pub max: i32,
    pub visible: bool,
    pub players: Vec<NameAndId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarCommandColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarCommandOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerGameMode {
    pub player: NameAndId,
    pub gamemode: GameMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraTarget {
    pub player: NameAndId,
    pub target: Option<EntityRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopwatchState {
    pub id: String,
    pub creation_time_millis: u64,
    pub accumulated_elapsed_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledFunction {
    pub id: String,
    pub function: String,
    pub tag: bool,
    pub trigger_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFunctionDefinition {
    pub id: String,
    pub commands: Vec<String>,
    pub macro_parameters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantiatedCommandFunction {
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFunctionTag {
    pub id: String,
    pub functions: Vec<String>,
}

pub(super) const MAX_COMMAND_FUNCTION_LINE_LENGTH: usize = 2_000_000;
