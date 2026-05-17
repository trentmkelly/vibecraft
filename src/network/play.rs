#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Write};

use crate::network::codec::{write_bitset, write_collection, write_identifier, write_string, Uuid};
use crate::network::dispatch::{DecodedPacket, DispatchOutcome, PacketDirection, ProtocolState};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;
use crate::storage::chunk::{ChunkSection, LevelChunk, PalettedContainer};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

pub const SERVERBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 69;
pub const CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 141;

pub const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub const SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
pub const SERVERBOUND_CLIENT_COMMAND_PACKET_ID: i32 = 12;
pub const SERVERBOUND_CLIENT_TICK_END_PACKET_ID: i32 = 13;
pub const SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;
pub const SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
pub const SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;
pub const SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;
pub const SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;
pub const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
pub const SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
pub const SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;

pub const CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
pub const CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
pub const CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
pub const CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID: i32 = 10;
pub const CLIENTBOUND_ADD_ENTITY_PACKET_ID: i32 = 1;
pub const CLIENTBOUND_ANIMATE_PACKET_ID: i32 = 2;
pub const CLIENTBOUND_AWARD_STATS_PACKET_ID: i32 = 3;
pub const CLIENTBOUND_BOSS_EVENT_PACKET_ID: i32 = 9;
pub const CLIENTBOUND_CLEAR_TITLES_PACKET_ID: i32 = 14;
pub const CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID: i32 = 15;
pub const CLIENTBOUND_COMMANDS_PACKET_ID: i32 = 16;
pub const CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 17;
pub const CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID: i32 = 18;
pub const CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID: i32 = 19;
pub const CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID: i32 = 20;
pub const CLIENTBOUND_DEBUG_BLOCK_VALUE_PACKET_ID: i32 = 26;
pub const CLIENTBOUND_DEBUG_CHUNK_VALUE_PACKET_ID: i32 = 27;
pub const CLIENTBOUND_DEBUG_ENTITY_VALUE_PACKET_ID: i32 = 28;
pub const CLIENTBOUND_DEBUG_EVENT_PACKET_ID: i32 = 29;
pub const CLIENTBOUND_DEBUG_SAMPLE_PACKET_ID: i32 = 30;
pub const CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID: i32 = 39;
pub const CLIENTBOUND_GAME_EVENT_PACKET_ID: i32 = 38;
pub const CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID: i32 = 43;
pub const CLIENTBOUND_KEEP_ALIVE_PACKET_ID: i32 = 44;
pub const CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID: i32 = 47;
pub const CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID: i32 = 51;
pub const CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID: i32 = 52;
pub const CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID: i32 = 53;
pub const CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID: i32 = 54;
pub const CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID: i32 = 56;
pub const CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID: i32 = 68;
pub const CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID: i32 = 74;
pub const CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID: i32 = 75;
pub const CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID: i32 = 76;
pub const CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID: i32 = 77;
pub const CLIENTBOUND_RESET_SCORE_PACKET_ID: i32 = 79;
pub const CLIENTBOUND_PLAYER_POSITION_PACKET_ID: i32 = 72;
pub const CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID: i32 = 64;
pub const CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID: i32 = 70;
pub const CLIENTBOUND_RESPAWN_PACKET_ID: i32 = 82;
pub const CLIENTBOUND_ROTATE_HEAD_PACKET_ID: i32 = 83;
pub const CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID: i32 = 87;
pub const CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID: i32 = 88;
pub const CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID: i32 = 89;
pub const CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID: i32 = 90;
pub const CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID: i32 = 91;
pub const CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID: i32 = 92;
pub const CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID: i32 = 94;
pub const CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID: i32 = 95;
pub const CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID: i32 = 96;
pub const CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID: i32 = 97;
pub const CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID: i32 = 98;
pub const CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID: i32 = 99;
pub const CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID: i32 = 100;
pub const CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID: i32 = 101;
pub const CLIENTBOUND_SET_EQUIPMENT_PACKET_ID: i32 = 102;
pub const CLIENTBOUND_SET_EXPERIENCE_PACKET_ID: i32 = 103;
pub const CLIENTBOUND_SET_HEALTH_PACKET_ID: i32 = 104;
pub const CLIENTBOUND_SET_HELD_SLOT_PACKET_ID: i32 = 105;
pub const CLIENTBOUND_SET_OBJECTIVE_PACKET_ID: i32 = 106;
pub const CLIENTBOUND_SET_PASSENGERS_PACKET_ID: i32 = 107;
pub const CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID: i32 = 109;
pub const CLIENTBOUND_SET_SCORE_PACKET_ID: i32 = 110;
pub const CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID: i32 = 112;
pub const CLIENTBOUND_SET_TIME_PACKET_ID: i32 = 113;
pub const CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID: i32 = 114;
pub const CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID: i32 = 115;
pub const CLIENTBOUND_SOUND_ENTITY_PACKET_ID: i32 = 116;
pub const CLIENTBOUND_SOUND_PACKET_ID: i32 = 117;
pub const CLIENTBOUND_START_CONFIGURATION_PACKET_ID: i32 = 118;
pub const CLIENTBOUND_DISCONNECT_PACKET_ID: i32 = 32;
pub const CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID: i32 = 125;
pub const CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID: i32 = 130;
pub const CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID: i32 = 131;
pub const CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID: i32 = 132;
pub const CLIENTBOUND_UPDATE_RECIPES_PACKET_ID: i32 = 133;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayProtocolRegistry {
    serverbound: Vec<&'static str>,
    clientbound: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonPlayerSpawnInfo {
    pub dimension_type: Identifier,
    pub dimension: Identifier,
    pub seed: i64,
    pub game_mode: GameMode,
    pub previous_game_mode: Option<GameMode>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death_location: Option<(Identifier, [i32; 3])>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLoginPacket {
    pub player_id: i32,
    pub hardcore: bool,
    pub levels: Vec<Identifier>,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub do_limited_crafting: bool,
    pub spawn_info: CommonPlayerSpawnInfo,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundMovePlayerPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub has_position: bool,
    pub has_rotation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptTeleportationPacket {
    pub teleport_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSetCarriedItemPacket {
    pub slot: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetHeldSlotPacket {
    pub slot: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundChunkBatchReceivedPacket {
    pub desired_chunks_per_tick: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundChunkBatchFinishedPacket {
    pub batch_size: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundAddEntityPacket {
    pub id: i32,
    pub uuid: Uuid,
    pub entity_type: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub x_rot: u8,
    pub y_rot: u8,
    pub y_head_rot: u8,
    pub data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRemoveEntitiesPacket {
    pub entity_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEntityDataPacket {
    pub id: i32,
    pub packed_items: Vec<EntityDataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDataValue {
    pub index: u8,
    pub serializer_id: i32,
    pub encoded_payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetEntityMotionPacket {
    pub id: i32,
    pub movement: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTeleportEntityPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: u32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundMoveEntityPacket {
    pub id: i32,
    pub delta: [i16; 3],
    pub y_rot: u8,
    pub x_rot: u8,
    pub on_ground: bool,
    pub has_position: bool,
    pub has_rotation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRotateHeadPacket {
    pub id: i32,
    pub y_head_rot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetPassengersPacket {
    pub vehicle: i32,
    pub passengers: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetEntityLinkPacket {
    pub source_id: i32,
    pub dest_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetEquipmentPacket {
    pub entity: i32,
    pub slots: Vec<EquipmentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentEntry {
    pub slot: EquipmentSlotKind,
    pub item_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotKind {
    MainHand = 0,
    OffHand = 1,
    Feet = 2,
    Legs = 3,
    Chest = 4,
    Head = 5,
    Body = 6,
    Saddle = 7,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundUpdateAttributesPacket {
    pub entity_id: i32,
    pub attributes: Vec<AttributeSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSnapshot {
    pub attribute_id: i32,
    pub base: f64,
    pub modifiers: Vec<AttributeModifierSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeModifierSnapshot {
    pub id: Uuid,
    pub amount: f64,
    pub operation: AttributeModifierOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeModifierOperation {
    AddValue = 0,
    AddMultipliedBase = 1,
    AddMultipliedTotal = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundUpdateMobEffectPacket {
    pub entity_id: i32,
    pub effect_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: MobEffectFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobEffectFlags(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundAnimatePacket {
    pub id: i32,
    pub action: EntityAnimation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnimation {
    SwingMainHand = 0,
    WakeUp = 2,
    SwingOffHand = 3,
    CriticalHit = 4,
    MagicCriticalHit = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySpawnBundle {
    pub spawn: ClientboundAddEntityPacket,
    pub metadata: Option<ClientboundSetEntityDataPacket>,
    pub velocity: Option<ClientboundSetEntityMotionPacket>,
    pub equipment: Option<ClientboundSetEquipmentPacket>,
    pub attributes: Option<ClientboundUpdateAttributesPacket>,
    pub effects: Vec<ClientboundUpdateMobEffectPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundContainerPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slots: Vec<Option<i32>>,
    pub carried_item: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRecipePacket {
    pub recipes: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAdvancementsPacket {
    pub reset: bool,
    pub added: Vec<Identifier>,
    pub removed: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAwardStatsPacket {
    pub stats: Vec<(Identifier, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundGameRuleValuesPacket {
    pub values: BTreeMap<Identifier, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundScoreboardPacket {
    pub objective: String,
    pub owner: Option<String>,
    pub score: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundBossEventPacket {
    pub event_id: Uuid,
    pub operation: BossEventOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossEventOperation {
    Add,
    Remove,
    UpdateProgress,
    UpdateName,
    UpdateStyle,
    UpdateProperties,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTitlePacket {
    pub kind: TitlePacketKind,
    pub text: Option<String>,
    pub fade_in: Option<i32>,
    pub stay: Option<i32>,
    pub fade_out: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlePacketKind {
    Title,
    Subtitle,
    ActionBar,
    Times,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSoundPacket {
    pub sound_id: i32,
    pub source_id: i32,
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
    pub entity_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundParticlePacket {
    pub particle_id: i32,
    pub long_distance: bool,
    pub always_show: bool,
    pub position: Vec3,
    pub offset: Vec3,
    pub max_speed: f32,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundMapItemDataPacket {
    pub map_id: i32,
    pub scale: u8,
    pub locked: bool,
    pub decorations: usize,
    pub color_patch: Option<MapPatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPatch {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundWorldBorderPacket {
    pub kind: WorldBorderPacketKind,
    pub center: Option<(f64, f64)>,
    pub old_size: Option<f64>,
    pub new_size: Option<f64>,
    pub lerp_time_ms: Option<i64>,
    pub warning_blocks: Option<i32>,
    pub warning_time: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldBorderPacketKind {
    Initialize,
    SetCenter,
    LerpSize,
    SetSize,
    SetWarningDelay,
    SetWarningDistance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCommandsPacket {
    pub root_index: i32,
    pub node_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCommandSuggestionsPacket {
    pub transaction_id: i32,
    pub start: i32,
    pub length: i32,
    pub matches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugPacket {
    pub kind: DebugPacketKind,
    pub payload_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugPacketKind {
    BlockValue,
    ChunkValue,
    EntityValue,
    Event,
    Sample,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLevelChunkWithLightPacket {
    pub pos: ChunkPos,
    pub chunk_data: Option<ClientboundLevelChunkPacketData>,
    pub light_data: Option<ClientboundLightUpdatePacketData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacket {
    pub pos: ChunkPos,
    pub light_data: ClientboundLightUpdatePacketData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLightUpdatePacketData {
    pub sky_y_mask: Vec<u64>,
    pub block_y_mask: Vec<u64>,
    pub empty_sky_y_mask: Vec<u64>,
    pub empty_block_y_mask: Vec<u64>,
    pub sky_updates: Vec<Vec<i8>>,
    pub block_updates: Vec<Vec<i8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundLevelChunkPacketData {
    pub heightmaps: BTreeMap<String, Vec<i64>>,
    pub buffer: Vec<u8>,
    pub block_entity_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkChunkSection {
    pub non_empty_block_count: i16,
    pub block_states: NetworkPalettedContainer,
    pub biomes: NetworkPalettedContainer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPalettedContainer {
    pub bits_per_entry: u8,
    pub palette_ids: Vec<i32>,
    pub data: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundRespawnPacket {
    pub spawn_info: CommonPlayerSpawnInfo,
    pub data_to_keep: RespawnDataToKeep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnDataToKeep {
    bits: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerCombatKillPacket {
    pub player_id: i32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespawnReason {
    Death,
    WonGameReturnToOverworld,
    DimensionChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespawnRequest {
    pub reason: RespawnReason,
    pub keep_all_player_data: bool,
    pub missing_respawn_block: bool,
    pub hardcore: bool,
    pub active_effect_count: usize,
    pub respawn_anchor_depleted: bool,
    pub spawn_info: CommonPlayerSpawnInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDifficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub may_fly: bool,
    pub instabuild: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinGameSettings {
    pub login: ClientboundLoginPacket,
    pub difficulty: GameDifficulty,
    pub difficulty_locked: bool,
    pub abilities: PlayerAbilities,
    pub permission_level: u8,
    pub initial_recipes: bool,
    pub initial_recipe_book: bool,
    pub scoreboard: bool,
    pub server_status: bool,
    pub player_info_existing_count: usize,
    pub active_effect_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayInstruction {
    Login(ClientboundLoginPacket),
    ChangeDifficulty {
        difficulty: GameDifficulty,
        locked: bool,
    },
    PlayerAbilities(PlayerAbilities),
    SetHeldSlot(ClientboundSetHeldSlotPacket),
    UpdateRecipes,
    UpdatePermissionLevel(u8),
    SendInitialRecipeBook,
    UpdateScoreboard,
    TeleportToSpawn {
        teleport_id: i32,
    },
    ServerStatus,
    PlayerInfoUpdate {
        existing_players: usize,
    },
    BroadcastSelfPlayerInfo,
    SendLevelInfo,
    AddPlayerToLevel,
    BossEventsOnConnect,
    ActiveEffects {
        count: usize,
    },
    InitInventoryMenu,
    ChunkBatchStart,
    LevelChunkWithLight(ClientboundLevelChunkWithLightPacket),
    ChunkBatchFinished(ClientboundChunkBatchFinishedPacket),
    ForgetLevelChunk {
        pos: ChunkPos,
    },
    AddEntity(ClientboundAddEntityPacket),
    SetEntityData(ClientboundSetEntityDataPacket),
    SetEntityMotion(ClientboundSetEntityMotionPacket),
    SetEquipment(ClientboundSetEquipmentPacket),
    UpdateAttributes(ClientboundUpdateAttributesPacket),
    UpdateMobEffect(ClientboundUpdateMobEffectPacket),
    RemoveEntities(ClientboundRemoveEntitiesPacket),
    MoveEntity(ClientboundMoveEntityPacket),
    TeleportEntity(ClientboundTeleportEntityPacket),
    SetPassengers(ClientboundSetPassengersPacket),
    SetEntityLink(ClientboundSetEntityLinkPacket),
    RotateHead(ClientboundRotateHeadPacket),
    Animate(ClientboundAnimatePacket),
    Container(ClientboundContainerPacket),
    Recipes(ClientboundRecipePacket),
    Advancements(ClientboundAdvancementsPacket),
    AwardStats(ClientboundAwardStatsPacket),
    GameRuleValues(ClientboundGameRuleValuesPacket),
    Scoreboard(ClientboundScoreboardPacket),
    BossEvent(ClientboundBossEventPacket),
    Title(ClientboundTitlePacket),
    Sound(ClientboundSoundPacket),
    Particle(ClientboundParticlePacket),
    MapItemData(ClientboundMapItemDataPacket),
    WorldBorder(ClientboundWorldBorderPacket),
    Commands(ClientboundCommandsPacket),
    CommandSuggestions(ClientboundCommandSuggestionsPacket),
    Debug(ClientboundDebugPacket),
    CombatKill(ClientboundPlayerCombatKillPacket),
    NoRespawnBlockAvailable,
    Respawn(ClientboundRespawnPacket),
    SetDefaultSpawnPosition,
    SetExperience,
    SetHealth,
    SetGameModeSpectator,
    DisableSpectatorsGenerateChunks,
    RespawnAnchorDepleteSound,
    PlayerPosition {
        teleport_id: i32,
    },
    StartConfiguration,
    Disconnect(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Joining,
    WaitingForPlayerLoaded,
    Playing,
    Reconfiguring,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySession {
    pub state: PlayState,
    pub entity_id: i32,
    pub selected_slot: i16,
    pub pending_teleports: BTreeSet<i32>,
    pub last_move: Option<ServerboundMovePlayerPacket>,
    pub loaded: bool,
    pub disconnect_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChunkSender {
    pending_chunks: BTreeSet<ChunkPos>,
    memory_connection: bool,
    desired_chunks_per_tick: f32,
    batch_quota: f32,
    unacknowledged_batches: i32,
    max_unacknowledged_batches: i32,
}

impl Default for PlayProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayProtocolRegistry {
    pub fn new() -> Self {
        Self {
            serverbound: SERVERBOUND_PLAY_PACKET_NAMES.to_vec(),
            clientbound: CLIENTBOUND_PLAY_PACKET_NAMES.to_vec(),
        }
    }

    pub fn serverbound(&self) -> &[&'static str] {
        &self.serverbound
    }

    pub fn clientbound(&self) -> &[&'static str] {
        &self.clientbound
    }

    pub fn serverbound_name(&self, packet_id: i32) -> Option<&'static str> {
        self.serverbound.get(packet_id as usize).copied()
    }

    pub fn clientbound_name(&self, packet_id: i32) -> Option<&'static str> {
        self.clientbound.get(packet_id as usize).copied()
    }

    pub fn is_serverbound_play_packet(&self, packet_id: i32) -> bool {
        packet_id >= 0 && (packet_id as usize) < self.serverbound.len()
    }

    pub fn is_clientbound_play_packet(&self, packet_id: i32) -> bool {
        packet_id >= 0 && (packet_id as usize) < self.clientbound.len()
    }
}

impl Default for CommonPlayerSpawnInfo {
    fn default() -> Self {
        Self {
            dimension_type: Identifier::parse("minecraft:overworld").unwrap(),
            dimension: Identifier::parse("minecraft:overworld").unwrap(),
            seed: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
            portal_cooldown: 0,
            sea_level: 63,
        }
    }
}

impl PlaySession {
    pub fn new(entity_id: i32, selected_slot: i16) -> Self {
        Self {
            state: PlayState::Joining,
            entity_id,
            selected_slot,
            pending_teleports: BTreeSet::new(),
            last_move: None,
            loaded: false,
            disconnect_reason: None,
        }
    }

    pub fn join_sequence(&mut self, login: ClientboundLoginPacket) -> Vec<PlayInstruction> {
        self.state = PlayState::WaitingForPlayerLoaded;
        vec![
            PlayInstruction::Login(login),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket {
                slot: self.selected_slot as i32,
            }),
            PlayInstruction::PlayerPosition { teleport_id: 0 },
        ]
    }

    pub fn vanilla_join_sequence(&mut self, settings: JoinGameSettings) -> Vec<PlayInstruction> {
        self.state = PlayState::WaitingForPlayerLoaded;
        let mut instructions = vec![
            PlayInstruction::Login(settings.login),
            PlayInstruction::ChangeDifficulty {
                difficulty: settings.difficulty,
                locked: settings.difficulty_locked,
            },
            PlayInstruction::PlayerAbilities(settings.abilities),
            PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket {
                slot: self.selected_slot as i32,
            }),
        ];
        if settings.initial_recipes {
            instructions.push(PlayInstruction::UpdateRecipes);
        }
        instructions.push(PlayInstruction::UpdatePermissionLevel(
            settings.permission_level,
        ));
        if settings.initial_recipe_book {
            instructions.push(PlayInstruction::SendInitialRecipeBook);
        }
        if settings.scoreboard {
            instructions.push(PlayInstruction::UpdateScoreboard);
        }
        instructions.push(PlayInstruction::TeleportToSpawn { teleport_id: 0 });
        if settings.server_status {
            instructions.push(PlayInstruction::ServerStatus);
        }
        instructions.push(PlayInstruction::PlayerInfoUpdate {
            existing_players: settings.player_info_existing_count,
        });
        instructions.push(PlayInstruction::BroadcastSelfPlayerInfo);
        instructions.push(PlayInstruction::SendLevelInfo);
        instructions.push(PlayInstruction::AddPlayerToLevel);
        instructions.push(PlayInstruction::BossEventsOnConnect);
        if settings.active_effect_count > 0 {
            instructions.push(PlayInstruction::ActiveEffects {
                count: settings.active_effect_count,
            });
        }
        instructions.push(PlayInstruction::InitInventoryMenu);
        instructions
    }

    pub fn handle_decoded(&mut self, packet: DecodedPacket) -> DispatchOutcome {
        if packet.state != ProtocolState::Play || packet.direction != PacketDirection::Serverbound {
            return DispatchOutcome::Disconnect(format!(
                "unexpected {:?} {:?} packet {} during play",
                packet.state, packet.direction, packet.id
            ));
        }

        match packet.id {
            SERVERBOUND_PLAYER_LOADED_PACKET_ID => {
                self.loaded = true;
                self.state = PlayState::Playing;
                DispatchOutcome::Handled
            }
            SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundAcceptTeleportationPacket::read(&mut input) {
                    Ok(ack) => {
                        self.pending_teleports.remove(&ack.teleport_id);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad teleport ack: {err}")),
                }
            }
            SERVERBOUND_CLIENT_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match read_var_i32(&mut input) {
                    Ok(0..=2) => DispatchOutcome::Handled,
                    Ok(action) => DispatchOutcome::Disconnect(format!(
                        "invalid client command action {action}"
                    )),
                    Err(err) => DispatchOutcome::Disconnect(format!("bad client command: {err}")),
                }
            }
            SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChunkBatchReceivedPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad chunk batch received: {err}"))
                    }
                }
            }
            SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::Pos)
            }
            SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::PosRot)
            }
            SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::Rot)
            }
            SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => {
                self.handle_move_payload(packet.payload, MoveShape::StatusOnly)
            }
            SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCarriedItemPacket::read(&mut input) {
                    Ok(held) if (0..=8).contains(&held.slot) => {
                        self.selected_slot = held.slot;
                        DispatchOutcome::Handled
                    }
                    Ok(held) => DispatchOutcome::Disconnect(format!(
                        "invalid carried item slot {}",
                        held.slot
                    )),
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad carried item packet: {err}"))
                    }
                }
            }
            _ => {
                if PlayProtocolRegistry::new().is_serverbound_play_packet(packet.id) {
                    DispatchOutcome::Handled
                } else {
                    DispatchOutcome::Disconnect(format!("unknown play packet id {}", packet.id))
                }
            }
        }
    }

    pub fn request_reconfiguration(&mut self) -> PlayInstruction {
        self.state = PlayState::Reconfiguring;
        PlayInstruction::StartConfiguration
    }

    pub fn disconnect(&mut self, reason: impl Into<String>) -> PlayInstruction {
        let reason = reason.into();
        self.state = PlayState::Disconnected;
        self.disconnect_reason = Some(reason.clone());
        PlayInstruction::Disconnect(reason)
    }

    pub fn death_screen(&self, message: impl Into<String>) -> PlayInstruction {
        PlayInstruction::CombatKill(ClientboundPlayerCombatKillPacket {
            player_id: self.entity_id,
            message: message.into(),
        })
    }

    pub fn respawn_flow(&mut self, request: RespawnRequest) -> Vec<PlayInstruction> {
        let mut instructions = Vec::new();
        if request.missing_respawn_block {
            instructions.push(PlayInstruction::NoRespawnBlockAvailable);
        }
        instructions.push(PlayInstruction::Respawn(ClientboundRespawnPacket {
            spawn_info: request.spawn_info,
            data_to_keep: if request.keep_all_player_data {
                RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS
            } else {
                RespawnDataToKeep::NONE
            },
        }));
        instructions.push(PlayInstruction::TeleportToSpawn { teleport_id: 0 });
        instructions.push(PlayInstruction::SetDefaultSpawnPosition);
        instructions.push(PlayInstruction::ChangeDifficulty {
            difficulty: GameDifficulty::Normal,
            locked: false,
        });
        instructions.push(PlayInstruction::SetExperience);
        if request.active_effect_count > 0 {
            instructions.push(PlayInstruction::ActiveEffects {
                count: request.active_effect_count,
            });
        }
        instructions.push(PlayInstruction::SendLevelInfo);
        instructions.push(PlayInstruction::UpdatePermissionLevel(0));
        instructions.push(PlayInstruction::AddPlayerToLevel);
        instructions.push(PlayInstruction::InitInventoryMenu);
        instructions.push(PlayInstruction::SetHealth);
        if matches!(request.reason, RespawnReason::Death) && request.hardcore {
            instructions.push(PlayInstruction::SetGameModeSpectator);
            instructions.push(PlayInstruction::DisableSpectatorsGenerateChunks);
        }
        if request.respawn_anchor_depleted {
            instructions.push(PlayInstruction::RespawnAnchorDepleteSound);
        }
        instructions
    }

    fn handle_move_payload(&mut self, payload: Vec<u8>, shape: MoveShape) -> DispatchOutcome {
        let mut input = &payload[..];
        match ServerboundMovePlayerPacket::read_shape(&mut input, shape) {
            Ok(packet) => {
                self.last_move = Some(packet);
                DispatchOutcome::Handled
            }
            Err(err) => DispatchOutcome::Disconnect(format!("bad movement packet: {err}")),
        }
    }
}

impl PlayerChunkSender {
    pub const MIN_CHUNKS_PER_TICK: f32 = 0.01;
    pub const MAX_CHUNKS_PER_TICK: f32 = 64.0;
    pub const START_CHUNKS_PER_TICK: f32 = 9.0;
    pub const MAX_UNACKNOWLEDGED_BATCHES_AFTER_ACK: i32 = 10;

    pub fn new(memory_connection: bool) -> Self {
        Self {
            pending_chunks: BTreeSet::new(),
            memory_connection,
            desired_chunks_per_tick: Self::START_CHUNKS_PER_TICK,
            batch_quota: 0.0,
            unacknowledged_batches: 0,
            max_unacknowledged_batches: 1,
        }
    }

    pub fn mark_chunk_pending_to_send(&mut self, pos: ChunkPos) {
        self.pending_chunks.insert(pos);
    }

    pub fn drop_chunk(&mut self, pos: ChunkPos, player_alive: bool) -> Option<PlayInstruction> {
        if self.pending_chunks.remove(&pos) || !player_alive {
            None
        } else {
            Some(PlayInstruction::ForgetLevelChunk { pos })
        }
    }

    pub fn send_next_chunks(&mut self, player_pos: ChunkPos) -> Vec<PlayInstruction> {
        if self.unacknowledged_batches >= self.max_unacknowledged_batches {
            return Vec::new();
        }

        let max_batch_size = self.desired_chunks_per_tick.max(1.0);
        self.batch_quota = (self.batch_quota + self.desired_chunks_per_tick).min(max_batch_size);
        if self.batch_quota < 1.0 || self.pending_chunks.is_empty() {
            return Vec::new();
        }

        let chunks_to_send = self.collect_chunks_to_send(player_pos);
        if chunks_to_send.is_empty() {
            return Vec::new();
        }

        self.unacknowledged_batches += 1;
        self.batch_quota -= chunks_to_send.len() as f32;

        let mut instructions = Vec::with_capacity(chunks_to_send.len() + 2);
        instructions.push(PlayInstruction::ChunkBatchStart);
        instructions.extend(chunks_to_send.iter().copied().map(|pos| {
            PlayInstruction::LevelChunkWithLight(ClientboundLevelChunkWithLightPacket {
                pos,
                chunk_data: None,
                light_data: None,
            })
        }));
        instructions.push(PlayInstruction::ChunkBatchFinished(
            ClientboundChunkBatchFinishedPacket {
                batch_size: chunks_to_send.len() as i32,
            },
        ));
        instructions
    }

    pub fn on_chunk_batch_received_by_client(&mut self, desired_chunks_per_tick: f32) {
        self.unacknowledged_batches -= 1;
        self.desired_chunks_per_tick = if desired_chunks_per_tick.is_nan() {
            Self::MIN_CHUNKS_PER_TICK
        } else {
            desired_chunks_per_tick.clamp(Self::MIN_CHUNKS_PER_TICK, Self::MAX_CHUNKS_PER_TICK)
        };
        if self.unacknowledged_batches == 0 {
            self.batch_quota = 1.0;
        }
        self.max_unacknowledged_batches = Self::MAX_UNACKNOWLEDGED_BATCHES_AFTER_ACK;
    }

    pub fn is_pending(&self, pos: ChunkPos) -> bool {
        self.pending_chunks.contains(&pos)
    }

    pub fn desired_chunks_per_tick(&self) -> f32 {
        self.desired_chunks_per_tick
    }

    pub fn unacknowledged_batches(&self) -> i32 {
        self.unacknowledged_batches
    }

    fn collect_chunks_to_send(&mut self, player_pos: ChunkPos) -> Vec<ChunkPos> {
        let max_batch_size = self.batch_quota.floor() as usize;
        let mut chunks: Vec<_> = self.pending_chunks.iter().copied().collect();
        chunks.sort_by_key(|pos| (chunk_distance_squared(player_pos, *pos), *pos));
        if !self.memory_connection && chunks.len() > max_batch_size {
            chunks.truncate(max_batch_size);
        }

        for chunk in &chunks {
            self.pending_chunks.remove(chunk);
        }
        chunks
    }
}

fn chunk_distance_squared(from: ChunkPos, to: ChunkPos) -> i32 {
    let dx = from.x - to.x;
    let dz = from.z - to.z;
    dx * dx + dz * dz
}

impl ClientboundLevelChunkWithLightPacket {
    pub fn from_chunk(chunk: &LevelChunk, light_data: ClientboundLightUpdatePacketData) -> Self {
        Self {
            pos: chunk.pos,
            chunk_data: Some(ClientboundLevelChunkPacketData::from_chunk(chunk)),
            light_data: Some(light_data),
        }
    }
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

impl ClientboundAddEntityPacket {
    pub fn new(
        id: i32,
        uuid: Uuid,
        entity_type: i32,
        position: Vec3,
        movement: Vec3,
        rotation: (f32, f32),
        y_head_rot: f32,
        data: i32,
    ) -> Self {
        Self {
            id,
            uuid,
            entity_type,
            position,
            movement,
            x_rot: pack_degrees(rotation.0),
            y_rot: pack_degrees(rotation.1),
            y_head_rot: pack_degrees(y_head_rot),
            data,
        }
    }
}

impl ClientboundMoveEntityPacket {
    pub fn pos(id: i32, delta: [i16; 3], on_ground: bool) -> Self {
        Self {
            id,
            delta,
            y_rot: 0,
            x_rot: 0,
            on_ground,
            has_position: true,
            has_rotation: false,
        }
    }

    pub fn pos_rot(id: i32, delta: [i16; 3], y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self {
            id,
            delta,
            y_rot: pack_degrees(y_rot),
            x_rot: pack_degrees(x_rot),
            on_ground,
            has_position: true,
            has_rotation: true,
        }
    }

    pub fn rot(id: i32, y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self {
            id,
            delta: [0, 0, 0],
            y_rot: pack_degrees(y_rot),
            x_rot: pack_degrees(x_rot),
            on_ground,
            has_position: false,
            has_rotation: true,
        }
    }
}

impl ClientboundSetEntityMotionPacket {
    pub fn new(id: i32, movement: Vec3) -> Self {
        Self {
            id,
            movement: clamp_velocity(movement),
        }
    }
}

impl ClientboundRotateHeadPacket {
    pub fn new(id: i32, y_head_rot: f32) -> Self {
        Self {
            id,
            y_head_rot: pack_degrees(y_head_rot),
        }
    }
}

impl ClientboundSetEntityLinkPacket {
    pub fn new(source_id: i32, dest_id: Option<i32>) -> Self {
        Self {
            source_id,
            dest_id: dest_id.unwrap_or(0),
        }
    }
}

impl ClientboundSetEquipmentPacket {
    pub fn encoded_slot_bytes(&self) -> Vec<u8> {
        self.slots
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let slot = entry.slot as u8;
                if index + 1 == self.slots.len() {
                    slot
                } else {
                    slot | 0x80
                }
            })
            .collect()
    }
}

impl MobEffectFlags {
    pub const AMBIENT: Self = Self(1);
    pub const VISIBLE: Self = Self(2);
    pub const SHOW_ICON: Self = Self(4);
    pub const BLEND: Self = Self(8);

    pub fn from_parts(ambient: bool, visible: bool, show_icon: bool, blend: bool) -> Self {
        Self(
            (if ambient { Self::AMBIENT.0 } else { 0 })
                | (if visible { Self::VISIBLE.0 } else { 0 })
                | (if show_icon { Self::SHOW_ICON.0 } else { 0 })
                | (if blend { Self::BLEND.0 } else { 0 }),
        )
    }
}

impl EntitySpawnBundle {
    pub fn instructions(self) -> Vec<PlayInstruction> {
        let mut instructions = vec![PlayInstruction::AddEntity(self.spawn)];
        if let Some(metadata) = self.metadata {
            instructions.push(PlayInstruction::SetEntityData(metadata));
        }
        if let Some(velocity) = self.velocity {
            instructions.push(PlayInstruction::SetEntityMotion(velocity));
        }
        if let Some(equipment) = self.equipment {
            instructions.push(PlayInstruction::SetEquipment(equipment));
        }
        if let Some(attributes) = self.attributes {
            instructions.push(PlayInstruction::UpdateAttributes(attributes));
        }
        instructions.extend(
            self.effects
                .into_iter()
                .map(PlayInstruction::UpdateMobEffect),
        );
        instructions
    }
}

fn pack_degrees(degrees: f32) -> u8 {
    ((degrees * 256.0 / 360.0).floor() as i32 & 255) as u8
}

fn clamp_velocity(movement: Vec3) -> Vec3 {
    Vec3 {
        x: movement.x.clamp(-3.9, 3.9),
        y: movement.y.clamp(-3.9, 3.9),
        z: movement.z.clamp(-3.9, 3.9),
    }
}

impl ClientboundLevelChunkPacketData {
    pub const MAX_BUFFER_SIZE: usize = 2_097_152;

    pub fn from_chunk(chunk: &LevelChunk) -> Self {
        let mut buffer = Vec::new();
        for section in &chunk.sections {
            NetworkChunkSection::from_storage_section(section)
                .write(&mut buffer)
                .expect("writing chunk section to vec");
        }
        assert!(
            buffer.len() <= Self::MAX_BUFFER_SIZE,
            "chunk packet buffer exceeds vanilla two-megabyte guard"
        );

        Self {
            heightmaps: chunk
                .heightmaps
                .iter()
                .filter_map(|(name, tag)| match tag {
                    Tag::LongArray(values) => Some((name.clone(), values.clone())),
                    _ => None,
                })
                .collect(),
            buffer,
            block_entity_count: chunk.block_entities.len(),
        }
    }
}

impl ClientboundLightUpdatePacketData {
    pub const DATA_LAYER_SIZE: usize = 2048;

    pub fn from_chunk_sections(sections: &[ChunkSection]) -> Self {
        let mut data = Self {
            sky_y_mask: Vec::new(),
            block_y_mask: Vec::new(),
            empty_sky_y_mask: Vec::new(),
            empty_block_y_mask: Vec::new(),
            sky_updates: Vec::new(),
            block_updates: Vec::new(),
        };

        for (section_index, section) in sections.iter().enumerate() {
            data.add_layer(section_index, section.sky_light.as_deref(), true);
            data.add_layer(section_index, section.block_light.as_deref(), false);
        }

        data
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bitset(writer, &self.sky_y_mask)?;
        write_bitset(writer, &self.block_y_mask)?;
        write_bitset(writer, &self.empty_sky_y_mask)?;
        write_bitset(writer, &self.empty_block_y_mask)?;
        write_collection(writer, &self.sky_updates, write_data_layer)?;
        write_collection(writer, &self.block_updates, write_data_layer)
    }

    fn add_layer(&mut self, section_index: usize, layer: Option<&[i8]>, sky: bool) {
        let Some(layer) = layer else {
            return;
        };
        assert_eq!(
            layer.len(),
            Self::DATA_LAYER_SIZE,
            "light update data layers are always 2048 bytes"
        );
        let empty = layer.iter().all(|byte| *byte == 0);
        let mask = if sky {
            if empty {
                &mut self.empty_sky_y_mask
            } else {
                self.sky_updates.push(layer.to_vec());
                &mut self.sky_y_mask
            }
        } else if empty {
            &mut self.empty_block_y_mask
        } else {
            self.block_updates.push(layer.to_vec());
            &mut self.block_y_mask
        };
        set_bit(mask, section_index);
    }
}

impl NetworkChunkSection {
    pub fn from_storage_section(section: &ChunkSection) -> Self {
        Self {
            non_empty_block_count: 0,
            block_states: NetworkPalettedContainer::from_storage_container(&section.block_states),
            biomes: NetworkPalettedContainer::from_storage_container(&section.biomes),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.non_empty_block_count.to_be_bytes())?;
        self.block_states.write(writer)?;
        self.biomes.write(writer)
    }
}

impl NetworkPalettedContainer {
    pub fn single(global_id: i32) -> Self {
        Self {
            bits_per_entry: 0,
            palette_ids: vec![global_id],
            data: Vec::new(),
        }
    }

    pub fn from_storage_container(tag: &Tag) -> Self {
        let Ok(container) = PalettedContainer::from_nbt(tag, 0) else {
            return Self::single(0);
        };
        let palette_ids = container
            .palette
            .iter()
            .map(storage_palette_entry_network_id)
            .collect::<Vec<_>>();
        Self {
            bits_per_entry: if palette_ids.len() <= 1 { 0 } else { 4 },
            palette_ids: if palette_ids.is_empty() {
                vec![0]
            } else {
                palette_ids
            },
            data: container.data.unwrap_or_default(),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.bits_per_entry])?;
        if self.bits_per_entry == 0 {
            write_var_i32(writer, self.palette_ids.first().copied().unwrap_or(0))?;
        } else {
            write_var_i32(writer, self.palette_ids.len() as i32)?;
            for id in &self.palette_ids {
                write_var_i32(writer, *id)?;
            }
        }
        write_var_i32(writer, self.data.len() as i32)?;
        for word in &self.data {
            writer.write_all(&word.to_be_bytes())?;
        }
        Ok(())
    }
}

fn set_bit(mask: &mut Vec<u64>, index: usize) {
    let word = index / 64;
    if mask.len() <= word {
        mask.resize(word + 1, 0);
    }
    mask[word] |= 1_u64 << (index % 64);
}

fn write_data_layer<W: Write>(writer: &mut W, layer: &Vec<i8>) -> io::Result<()> {
    if layer.len() != ClientboundLightUpdatePacketData::DATA_LAYER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "light update layer must be 2048 bytes",
        ));
    }
    let bytes = layer.iter().map(|byte| *byte as u8).collect::<Vec<_>>();
    writer.write_all(&bytes)
}

fn storage_palette_entry_network_id(tag: &Tag) -> i32 {
    match tag {
        Tag::Int(id) => *id,
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(name, value)| {
                (name == "id" || name == "network_id")
                    .then_some(value)
                    .and_then(|value| match value {
                        Tag::Int(id) => Some(*id),
                        _ => None,
                    })
            })
            .unwrap_or(0),
        _ => 0,
    }
}

impl ServerboundAcceptTeleportationPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            teleport_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.teleport_id)
    }
}

impl ServerboundChunkBatchReceivedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            desired_chunks_per_tick: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f32(writer, self.desired_chunks_per_tick)
    }
}

impl ClientboundChunkBatchFinishedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            batch_size: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.batch_size)
    }
}

impl RespawnDataToKeep {
    pub const NONE: Self = Self { bits: 0 };
    pub const KEEP_ATTRIBUTE_MODIFIERS: Self = Self { bits: 1 };
    pub const KEEP_ENTITY_DATA: Self = Self { bits: 2 };
    pub const KEEP_ALL_DATA: Self = Self { bits: 3 };

    pub fn should_keep(self, mask: Self) -> bool {
        self.bits & mask.bits != 0
    }

    pub fn bits(self) -> u8 {
        self.bits
    }
}

impl ServerboundSetCarriedItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 2];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            slot: i16::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.slot.to_be_bytes())
    }
}

impl ClientboundSetHeldSlotPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)
    }
}

impl ClientboundGameRuleValuesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.values.len() as i32)?;
        for (key, value) in &self.values {
            write_identifier(writer, key)?;
            write_string(writer, value, 32767)?;
        }
        Ok(())
    }
}

impl ServerboundMovePlayerPacket {
    fn read_shape<R: Read>(reader: &mut R, shape: MoveShape) -> io::Result<Self> {
        let mut packet = Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: false,
            horizontal_collision: false,
            has_position: shape.has_position(),
            has_rotation: shape.has_rotation(),
        };
        if shape.has_position() {
            packet.x = read_f64(reader)?;
            packet.y = read_f64(reader)?;
            packet.z = read_f64(reader)?;
        }
        if shape.has_rotation() {
            packet.y_rot = read_f32(reader)?;
            packet.x_rot = read_f32(reader)?;
        }
        let flags = read_u8(reader)?;
        packet.on_ground = flags & 1 != 0;
        packet.horizontal_collision = flags & 2 != 0;
        Ok(packet)
    }

    fn write_shape<W: Write>(&self, writer: &mut W, shape: MoveShape) -> io::Result<()> {
        if shape.has_position() {
            writer.write_all(&self.x.to_be_bytes())?;
            writer.write_all(&self.y.to_be_bytes())?;
            writer.write_all(&self.z.to_be_bytes())?;
        }
        if shape.has_rotation() {
            writer.write_all(&self.y_rot.to_be_bytes())?;
            writer.write_all(&self.x_rot.to_be_bytes())?;
        }
        writer.write_all(&[pack_move_flags(self.on_ground, self.horizontal_collision)])
    }

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::PosRot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveShape {
    Pos,
    PosRot,
    Rot,
    StatusOnly,
}

impl MoveShape {
    fn has_position(self) -> bool {
        matches!(self, Self::Pos | Self::PosRot)
    }

    fn has_rotation(self) -> bool {
        matches!(self, Self::Rot | Self::PosRot)
    }
}

fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn write_f32<W: Write>(writer: &mut W, value: f32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

fn pack_move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
    (if on_ground { 1 } else { 0 }) | (if horizontal_collision { 2 } else { 0 })
}

static SERVERBOUND_PLAY_PACKET_NAMES: [&str; SERVERBOUND_PLAY_PACKET_COUNT_26_1_2] = [
    "accept_teleportation",
    "attack",
    "block_entity_tag_query",
    "bundle_item_selected",
    "change_difficulty",
    "change_game_mode",
    "chat_ack",
    "chat_command",
    "chat_command_signed",
    "chat",
    "chat_session_update",
    "chunk_batch_received",
    "client_command",
    "client_tick_end",
    "client_information",
    "command_suggestion",
    "configuration_acknowledged",
    "container_button_click",
    "container_click",
    "container_close",
    "container_slot_state_changed",
    "cookie_response",
    "custom_payload",
    "debug_subscription_request",
    "edit_book",
    "entity_tag_query",
    "interact",
    "jigsaw_generate",
    "keep_alive",
    "lock_difficulty",
    "move_player_pos",
    "move_player_pos_rot",
    "move_player_rot",
    "move_player_status_only",
    "move_vehicle",
    "paddle_boat",
    "pick_item_from_block",
    "pick_item_from_entity",
    "ping_request",
    "place_recipe",
    "player_abilities",
    "player_action",
    "player_command",
    "player_input",
    "player_loaded",
    "pong",
    "recipe_book_change_settings",
    "recipe_book_seen_recipe",
    "rename_item",
    "resource_pack",
    "seen_advancements",
    "select_trade",
    "set_beacon",
    "set_carried_item",
    "set_command_block",
    "set_command_minecart",
    "set_creative_mode_slot",
    "set_game_rule",
    "set_jigsaw_block",
    "set_structure_block",
    "set_test_block",
    "sign_update",
    "spectate_entity",
    "swing",
    "teleport_to_entity",
    "test_instance_block_action",
    "use_item_on",
    "use_item",
    "custom_click_action",
];

static CLIENTBOUND_PLAY_PACKET_NAMES: [&str; CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2] = [
    "bundle",
    "add_entity",
    "animate",
    "award_stats",
    "block_changed_ack",
    "block_destruction",
    "block_entity_data",
    "block_event",
    "block_update",
    "boss_event",
    "change_difficulty",
    "chunk_batch_finished",
    "chunk_batch_start",
    "chunks_biomes",
    "clear_titles",
    "command_suggestions",
    "commands",
    "container_close",
    "container_set_content",
    "container_set_data",
    "container_set_slot",
    "cookie_request",
    "cooldown",
    "custom_chat_completions",
    "custom_payload",
    "damage_event",
    "debug_block_value",
    "debug_chunk_value",
    "debug_entity_value",
    "debug_event",
    "debug_sample",
    "delete_chat",
    "disconnect",
    "disguised_chat",
    "entity_event",
    "entity_position_sync",
    "explode",
    "forget_level_chunk",
    "game_event",
    "game_rule_values",
    "game_test_highlight_pos",
    "mount_screen_open",
    "hurt_animation",
    "initialize_border",
    "keep_alive",
    "level_chunk_with_light",
    "level_event",
    "level_particles",
    "light_update",
    "login",
    "low_disk_space_warning",
    "map_item_data",
    "merchant_offers",
    "move_entity_pos",
    "move_entity_pos_rot",
    "move_minecart_along_track",
    "move_entity_rot",
    "move_vehicle",
    "open_book",
    "open_screen",
    "open_sign_editor",
    "ping",
    "pong_response",
    "place_ghost_recipe",
    "player_abilities",
    "player_chat",
    "player_combat_end",
    "player_combat_enter",
    "player_combat_kill",
    "player_info_remove",
    "player_info_update",
    "player_look_at",
    "player_position",
    "player_rotation",
    "recipe_book_add",
    "recipe_book_remove",
    "recipe_book_settings",
    "remove_entities",
    "remove_mob_effect",
    "reset_score",
    "resource_pack_pop",
    "resource_pack_push",
    "respawn",
    "rotate_head",
    "section_blocks_update",
    "select_advancements_tab",
    "server_data",
    "set_action_bar_text",
    "set_border_center",
    "set_border_lerp_size",
    "set_border_size",
    "set_border_warning_delay",
    "set_border_warning_distance",
    "set_camera",
    "set_chunk_cache_center",
    "set_chunk_cache_radius",
    "set_cursor_item",
    "set_default_spawn_position",
    "set_display_objective",
    "set_entity_data",
    "set_entity_link",
    "set_entity_motion",
    "set_equipment",
    "set_experience",
    "set_health",
    "set_held_slot",
    "set_objective",
    "set_passengers",
    "set_player_inventory",
    "set_player_team",
    "set_score",
    "set_simulation_distance",
    "set_subtitle_text",
    "set_time",
    "set_title_text",
    "set_titles_animation",
    "sound_entity",
    "sound",
    "start_configuration",
    "stop_sound",
    "store_cookie",
    "system_chat",
    "tab_list",
    "tag_query",
    "take_item_entity",
    "teleport_entity",
    "test_instance_block_status",
    "ticking_state",
    "ticking_step",
    "transfer",
    "update_advancements",
    "update_attributes",
    "update_mob_effect",
    "update_recipes",
    "update_tags",
    "projectile_power",
    "custom_report_details",
    "server_links",
    "waypoint",
    "clear_dialog",
    "show_dialog",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::codec::{cursor, read_identifier, read_string};

    fn decoded(id: i32, payload: Vec<u8>) -> DecodedPacket {
        DecodedPacket {
            state: ProtocolState::Play,
            direction: PacketDirection::Serverbound,
            id,
            payload,
        }
    }

    #[test]
    fn play_packet_registry_matches_game_protocol_order_and_counts() {
        let registry = PlayProtocolRegistry::new();
        assert_eq!(
            registry.serverbound().len(),
            SERVERBOUND_PLAY_PACKET_COUNT_26_1_2
        );
        assert_eq!(
            registry.clientbound().len(),
            CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2
        );
        assert_eq!(
            registry.serverbound_name(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID),
            Some("accept_teleportation")
        );
        assert_eq!(
            registry.serverbound_name(SERVERBOUND_PLAYER_LOADED_PACKET_ID),
            Some("player_loaded")
        );
        assert_eq!(
            registry.serverbound_name(SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID),
            Some("chunk_batch_received")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID),
            Some("chunk_batch_finished")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID),
            Some("chunk_batch_start")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID),
            Some("game_rule_values")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_ADD_ENTITY_PACKET_ID),
            Some("add_entity")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID),
            Some("remove_entities")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID),
            Some("set_entity_data")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID),
            Some("teleport_entity")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID),
            Some("update_mob_effect")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID),
            Some("container_set_content")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID),
            Some("recipe_book_add")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID),
            Some("update_advancements")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID),
            Some("set_objective")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_BOSS_EVENT_PACKET_ID),
            Some("boss_event")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_SOUND_PACKET_ID),
            Some("sound")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID),
            Some("level_particles")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_COMMANDS_PACKET_ID),
            Some("commands")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_DEBUG_SAMPLE_PACKET_ID),
            Some("debug_sample")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_LOGIN_PACKET_ID),
            Some("login")
        );
        assert_eq!(
            registry.clientbound_name(CLIENTBOUND_START_CONFIGURATION_PACKET_ID),
            Some("start_configuration")
        );
        assert_eq!(registry.serverbound().last(), Some(&"custom_click_action"));
        assert_eq!(registry.clientbound().last(), Some(&"show_dialog"));
    }

    #[test]
    fn broad_play_packet_families_are_represented_as_distinct_instructions() {
        let instructions = vec![
            PlayInstruction::Container(ClientboundContainerPacket {
                container_id: 1,
                state_id: 2,
                slots: vec![Some(5), None],
                carried_item: None,
            }),
            PlayInstruction::Recipes(ClientboundRecipePacket {
                recipes: vec![Identifier::parse("minecraft:stone").unwrap()],
            }),
            PlayInstruction::Advancements(ClientboundAdvancementsPacket {
                reset: true,
                added: vec![Identifier::parse("minecraft:story/root").unwrap()],
                removed: Vec::new(),
            }),
            PlayInstruction::AwardStats(ClientboundAwardStatsPacket {
                stats: vec![(Identifier::parse("minecraft:jump").unwrap(), 3)],
            }),
            PlayInstruction::GameRuleValues(ClientboundGameRuleValuesPacket {
                values: BTreeMap::from([(
                    Identifier::parse("minecraft:keep_inventory").unwrap(),
                    "true".to_string(),
                )]),
            }),
            PlayInstruction::Scoreboard(ClientboundScoreboardPacket {
                objective: "sidebar".to_string(),
                owner: Some("Steve".to_string()),
                score: Some(10),
            }),
            PlayInstruction::BossEvent(ClientboundBossEventPacket {
                event_id: Uuid([2; 16]),
                operation: BossEventOperation::UpdateProgress,
            }),
            PlayInstruction::Title(ClientboundTitlePacket {
                kind: TitlePacketKind::Times,
                text: None,
                fade_in: Some(10),
                stay: Some(70),
                fade_out: Some(20),
            }),
            PlayInstruction::Sound(ClientboundSoundPacket {
                sound_id: 1,
                source_id: 2,
                position: Vec3::ZERO,
                volume: 1.0,
                pitch: 1.0,
                seed: 99,
                entity_id: None,
            }),
            PlayInstruction::Particle(ClientboundParticlePacket {
                particle_id: 1,
                long_distance: false,
                always_show: true,
                position: Vec3::ZERO,
                offset: Vec3::ZERO,
                max_speed: 0.0,
                count: 1,
            }),
            PlayInstruction::MapItemData(ClientboundMapItemDataPacket {
                map_id: 1,
                scale: 2,
                locked: false,
                decorations: 1,
                color_patch: Some(MapPatch {
                    width: 1,
                    height: 1,
                    start_x: 0,
                    start_y: 0,
                }),
            }),
            PlayInstruction::WorldBorder(ClientboundWorldBorderPacket {
                kind: WorldBorderPacketKind::Initialize,
                center: Some((0.0, 0.0)),
                old_size: Some(6.0e7),
                new_size: Some(6.0e7),
                lerp_time_ms: Some(0),
                warning_blocks: Some(5),
                warning_time: Some(15),
            }),
            PlayInstruction::Commands(ClientboundCommandsPacket {
                root_index: 0,
                node_count: 1,
            }),
            PlayInstruction::CommandSuggestions(ClientboundCommandSuggestionsPacket {
                transaction_id: 4,
                start: 0,
                length: 2,
                matches: vec!["help".to_string()],
            }),
            PlayInstruction::Debug(ClientboundDebugPacket {
                kind: DebugPacketKind::Sample,
                payload_size: 8,
            }),
        ];

        assert_eq!(instructions.len(), 15);
        assert!(matches!(instructions[0], PlayInstruction::Container(_)));
        assert!(matches!(
            instructions[4],
            PlayInstruction::GameRuleValues(_)
        ));
        assert!(matches!(instructions[5], PlayInstruction::Scoreboard(_)));
        assert!(matches!(instructions[11], PlayInstruction::WorldBorder(_)));
        assert!(matches!(instructions[14], PlayInstruction::Debug(_)));
    }

    #[test]
    fn game_rule_values_packet_writes_registry_key_string_map() {
        let packet = ClientboundGameRuleValuesPacket {
            values: BTreeMap::from([
                (
                    Identifier::parse("minecraft:keep_inventory").unwrap(),
                    "true".to_string(),
                ),
                (
                    Identifier::parse("minecraft:random_tick_speed").unwrap(),
                    "3".to_string(),
                ),
            ]),
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        let mut input = cursor(bytes);

        assert_eq!(read_var_i32(&mut input).unwrap(), 2);
        assert_eq!(
            read_identifier(&mut input).unwrap(),
            Identifier::parse("minecraft:keep_inventory").unwrap()
        );
        assert_eq!(read_string(&mut input, 32767).unwrap(), "true");
        assert_eq!(
            read_identifier(&mut input).unwrap(),
            Identifier::parse("minecraft:random_tick_speed").unwrap()
        );
        assert_eq!(read_string(&mut input, 32767).unwrap(), "3");
    }

    #[test]
    fn entity_spawn_bundle_preserves_vanilla_spawn_then_state_update_order() {
        let spawn = ClientboundAddEntityPacket::new(
            7,
            Uuid([1; 16]),
            42,
            Vec3 {
                x: 1.0,
                y: 65.0,
                z: -2.0,
            },
            Vec3 {
                x: 4.5,
                y: -4.5,
                z: 0.25,
            },
            (45.0, 90.0),
            180.0,
            3,
        );
        assert_eq!(spawn.x_rot, 32);
        assert_eq!(spawn.y_rot, 64);
        assert_eq!(spawn.y_head_rot, 128);

        let velocity = ClientboundSetEntityMotionPacket::new(7, spawn.movement);
        assert_eq!(
            velocity.movement,
            Vec3 {
                x: 3.9,
                y: -3.9,
                z: 0.25,
            }
        );

        let instructions = EntitySpawnBundle {
            spawn: spawn.clone(),
            metadata: Some(ClientboundSetEntityDataPacket {
                id: 7,
                packed_items: vec![EntityDataValue {
                    index: 0,
                    serializer_id: 0,
                    encoded_payload: vec![0x20],
                }],
            }),
            velocity: Some(velocity),
            equipment: Some(ClientboundSetEquipmentPacket {
                entity: 7,
                slots: vec![
                    EquipmentEntry {
                        slot: EquipmentSlotKind::MainHand,
                        item_id: Some(1),
                    },
                    EquipmentEntry {
                        slot: EquipmentSlotKind::Head,
                        item_id: Some(2),
                    },
                ],
            }),
            attributes: Some(ClientboundUpdateAttributesPacket {
                entity_id: 7,
                attributes: vec![AttributeSnapshot {
                    attribute_id: 0,
                    base: 20.0,
                    modifiers: Vec::new(),
                }],
            }),
            effects: vec![ClientboundUpdateMobEffectPacket {
                entity_id: 7,
                effect_id: 1,
                amplifier: 0,
                duration_ticks: 200,
                flags: MobEffectFlags::from_parts(false, true, true, true),
            }],
        }
        .instructions();

        assert!(matches!(instructions[0], PlayInstruction::AddEntity(_)));
        assert!(matches!(instructions[1], PlayInstruction::SetEntityData(_)));
        assert!(matches!(
            instructions[2],
            PlayInstruction::SetEntityMotion(_)
        ));
        assert!(matches!(instructions[3], PlayInstruction::SetEquipment(_)));
        assert!(matches!(
            instructions[4],
            PlayInstruction::UpdateAttributes(_)
        ));
        assert!(matches!(
            instructions[5],
            PlayInstruction::UpdateMobEffect(_)
        ));
        let PlayInstruction::SetEquipment(equipment) = &instructions[3] else {
            panic!("expected equipment packet");
        };
        assert_eq!(equipment.encoded_slot_bytes(), vec![0x80, 5]);
        let PlayInstruction::UpdateMobEffect(effect) = instructions[5] else {
            panic!("expected effect packet");
        };
        assert_eq!(effect.flags, MobEffectFlags(14));
    }

    #[test]
    fn entity_movement_mount_link_and_animation_packets_capture_vanilla_shapes() {
        assert_eq!(
            ClientboundMoveEntityPacket::pos(7, [1, -2, 3], true),
            ClientboundMoveEntityPacket {
                id: 7,
                delta: [1, -2, 3],
                y_rot: 0,
                x_rot: 0,
                on_ground: true,
                has_position: true,
                has_rotation: false,
            }
        );
        assert_eq!(
            ClientboundMoveEntityPacket::pos_rot(7, [1, 2, 3], 90.0, 45.0, false).y_rot,
            64
        );
        assert_eq!(
            ClientboundMoveEntityPacket::rot(7, 180.0, 45.0, true).x_rot,
            32
        );
        assert_eq!(ClientboundRotateHeadPacket::new(7, 180.0).y_head_rot, 128);
        assert_eq!(
            ClientboundSetEntityLinkPacket::new(7, None),
            ClientboundSetEntityLinkPacket {
                source_id: 7,
                dest_id: 0,
            }
        );
        assert_eq!(
            ClientboundSetPassengersPacket {
                vehicle: 7,
                passengers: vec![8, 9],
            }
            .passengers,
            vec![8, 9]
        );
        assert_eq!(
            ClientboundAnimatePacket {
                id: 7,
                action: EntityAnimation::SwingOffHand,
            }
            .action as i32,
            3
        );
        assert_eq!(
            ClientboundRemoveEntitiesPacket {
                entity_ids: vec![7, 8]
            }
            .entity_ids,
            vec![7, 8]
        );
    }

    #[test]
    fn chunk_sender_starts_batches_sends_nearest_chunks_and_waits_for_first_ack() {
        let mut sender = PlayerChunkSender::new(false);
        for pos in [
            ChunkPos { x: 8, z: 0 },
            ChunkPos { x: 1, z: 0 },
            ChunkPos { x: -2, z: 0 },
            ChunkPos { x: 3, z: 4 },
            ChunkPos { x: 0, z: 2 },
            ChunkPos { x: 4, z: 4 },
            ChunkPos { x: -3, z: 3 },
            ChunkPos { x: 0, z: -1 },
            ChunkPos { x: 2, z: 2 },
            ChunkPos { x: 9, z: 9 },
        ] {
            sender.mark_chunk_pending_to_send(pos);
        }

        let batch = sender.send_next_chunks(ChunkPos { x: 0, z: 0 });
        assert_eq!(sender.unacknowledged_batches(), 1);
        assert_eq!(batch.first(), Some(&PlayInstruction::ChunkBatchStart));
        assert_eq!(
            batch.last(),
            Some(&PlayInstruction::ChunkBatchFinished(
                ClientboundChunkBatchFinishedPacket { batch_size: 9 }
            ))
        );
        let sent: Vec<_> = batch
            .iter()
            .filter_map(|instruction| match instruction {
                PlayInstruction::LevelChunkWithLight(packet) => Some(packet.pos),
                _ => None,
            })
            .collect();
        assert_eq!(
            sent,
            vec![
                ChunkPos { x: 0, z: -1 },
                ChunkPos { x: 1, z: 0 },
                ChunkPos { x: -2, z: 0 },
                ChunkPos { x: 0, z: 2 },
                ChunkPos { x: 2, z: 2 },
                ChunkPos { x: -3, z: 3 },
                ChunkPos { x: 3, z: 4 },
                ChunkPos { x: 4, z: 4 },
                ChunkPos { x: 8, z: 0 },
            ]
        );
        assert!(sender.is_pending(ChunkPos { x: 9, z: 9 }));
        assert!(sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());
    }

    #[test]
    fn chunk_sender_applies_client_feedback_clamp_and_allows_more_unacked_batches() {
        let mut sender = PlayerChunkSender::new(false);
        sender.mark_chunk_pending_to_send(ChunkPos { x: 0, z: 0 });
        assert!(!sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());

        sender.on_chunk_batch_received_by_client(f32::NAN);
        assert_eq!(
            sender.desired_chunks_per_tick(),
            PlayerChunkSender::MIN_CHUNKS_PER_TICK
        );
        sender.mark_chunk_pending_to_send(ChunkPos { x: 1, z: 0 });
        assert!(!sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());

        sender.on_chunk_batch_received_by_client(128.0);
        assert_eq!(
            sender.desired_chunks_per_tick(),
            PlayerChunkSender::MAX_CHUNKS_PER_TICK
        );

        let mut sender = PlayerChunkSender::new(false);
        sender.mark_chunk_pending_to_send(ChunkPos { x: 0, z: 0 });
        assert!(!sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());
        sender.on_chunk_batch_received_by_client(1.0);
        for x in 0..10 {
            sender.mark_chunk_pending_to_send(ChunkPos { x, z: 1 });
            assert!(!sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());
        }
        assert_eq!(sender.unacknowledged_batches(), 10);
        sender.mark_chunk_pending_to_send(ChunkPos { x: 10, z: 1 });
        assert!(sender.send_next_chunks(ChunkPos { x: 0, z: 0 }).is_empty());
    }

    #[test]
    fn chunk_batch_received_packet_uses_big_endian_float_payload() {
        let packet = ServerboundChunkBatchReceivedPacket {
            desired_chunks_per_tick: 12.5,
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(bytes, 12.5_f32.to_be_bytes());
        assert_eq!(
            ServerboundChunkBatchReceivedPacket::read(&mut cursor(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn light_update_data_uses_vanilla_masks_and_2048_byte_layers() {
        let sections = vec![
            ChunkSection {
                y: 0,
                block_states: PalettedContainer::single(Tag::Int(0), 4096).to_nbt(),
                biomes: PalettedContainer::single(Tag::Int(0), 64).to_nbt(),
                block_light: Some(vec![0; 2048]),
                sky_light: Some(vec![-1; 2048]),
            },
            ChunkSection {
                y: 1,
                block_states: PalettedContainer::single(Tag::Int(0), 4096).to_nbt(),
                biomes: PalettedContainer::single(Tag::Int(0), 64).to_nbt(),
                block_light: Some(vec![1; 2048]),
                sky_light: None,
            },
        ];

        let data = ClientboundLightUpdatePacketData::from_chunk_sections(&sections);
        assert_eq!(data.sky_y_mask, vec![1]);
        assert_eq!(data.empty_block_y_mask, vec![1]);
        assert_eq!(data.block_y_mask, vec![2]);
        assert_eq!(data.sky_updates.len(), 1);
        assert_eq!(data.block_updates.len(), 1);

        let mut payload = Vec::new();
        data.write(&mut payload).unwrap();
        assert!(!payload.is_empty());
    }

    #[test]
    fn chunk_section_serialization_matches_vanilla_section_field_order() {
        let section = NetworkChunkSection {
            non_empty_block_count: 2,
            block_states: NetworkPalettedContainer::single(5),
            biomes: NetworkPalettedContainer::single(7),
        };
        let mut bytes = Vec::new();
        section.write(&mut bytes).unwrap();

        assert_eq!(&bytes[0..2], &2_i16.to_be_bytes());
        assert_eq!(bytes[2], 0);
        assert_eq!(bytes[3], 5);
        assert_eq!(bytes[4], 0);
        assert_eq!(bytes[5], 0);
        assert_eq!(bytes[6], 7);
        assert_eq!(bytes[7], 0);
    }

    #[test]
    fn level_chunk_with_light_packet_carries_chunk_buffer_then_light_payload_data() {
        let mut heightmaps = BTreeMap::new();
        heightmaps.insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1, 2, 3]));
        let chunk = LevelChunk {
            pos: ChunkPos { x: 4, z: -2 },
            status: "minecraft:full".to_string(),
            inhabited_time: 0,
            sections: vec![ChunkSection {
                y: 0,
                block_states: PalettedContainer::single(Tag::Int(5), 4096).to_nbt(),
                biomes: PalettedContainer::single(Tag::Int(7), 64).to_nbt(),
                block_light: Some(vec![0; 2048]),
                sky_light: Some(vec![-1; 2048]),
            }],
            heightmaps,
            block_entities: vec![Tag::Compound(Vec::new())],
            entities: Vec::new(),
            structures: Tag::Compound(Vec::new()),
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            post_processing: Vec::new(),
        };
        let light_data = ClientboundLightUpdatePacketData::from_chunk_sections(&chunk.sections);
        let packet = ClientboundLevelChunkWithLightPacket::from_chunk(&chunk, light_data.clone());

        assert_eq!(packet.pos, chunk.pos);
        let chunk_data = packet.chunk_data.as_ref().unwrap();
        assert_eq!(chunk_data.heightmaps["WORLD_SURFACE"], vec![1, 2, 3]);
        assert_eq!(chunk_data.block_entity_count, 1);
        assert_eq!(chunk_data.buffer, vec![0, 0, 0, 5, 0, 0, 7, 0]);
        assert_eq!(packet.light_data, Some(light_data));
    }

    #[test]
    fn join_sequence_enters_play_with_login_held_slot_and_position_packets() {
        let mut session = PlaySession::new(42, 3);
        let login = ClientboundLoginPacket {
            player_id: 42,
            hardcore: false,
            levels: vec![Identifier::parse("minecraft:overworld").unwrap()],
            max_players: 20,
            chunk_radius: 10,
            simulation_distance: 10,
            reduced_debug_info: false,
            show_death_screen: true,
            do_limited_crafting: false,
            spawn_info: CommonPlayerSpawnInfo::default(),
            enforces_secure_chat: false,
        };

        let instructions = session.join_sequence(login.clone());
        assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
        assert_eq!(
            instructions,
            vec![
                PlayInstruction::Login(login),
                PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
                PlayInstruction::PlayerPosition { teleport_id: 0 }
            ]
        );
    }

    #[test]
    fn vanilla_join_sequence_matches_player_list_packet_and_side_effect_order() {
        let mut session = PlaySession::new(42, 3);
        let login = ClientboundLoginPacket {
            player_id: 42,
            hardcore: true,
            levels: vec![
                Identifier::parse("minecraft:overworld").unwrap(),
                Identifier::parse("minecraft:the_nether").unwrap(),
                Identifier::parse("minecraft:the_end").unwrap(),
            ],
            max_players: 20,
            chunk_radius: 10,
            simulation_distance: 10,
            reduced_debug_info: false,
            show_death_screen: true,
            do_limited_crafting: false,
            spawn_info: CommonPlayerSpawnInfo::default(),
            enforces_secure_chat: true,
        };
        let abilities = PlayerAbilities {
            invulnerable: false,
            flying: false,
            may_fly: false,
            instabuild: false,
            flying_speed: 0.05,
            walking_speed: 0.1,
        };

        let instructions = session.vanilla_join_sequence(JoinGameSettings {
            login: login.clone(),
            difficulty: GameDifficulty::Hard,
            difficulty_locked: true,
            abilities,
            permission_level: 2,
            initial_recipes: true,
            initial_recipe_book: true,
            scoreboard: true,
            server_status: true,
            player_info_existing_count: 2,
            active_effect_count: 1,
        });

        assert_eq!(session.state, PlayState::WaitingForPlayerLoaded);
        assert_eq!(
            instructions,
            vec![
                PlayInstruction::Login(login),
                PlayInstruction::ChangeDifficulty {
                    difficulty: GameDifficulty::Hard,
                    locked: true,
                },
                PlayInstruction::PlayerAbilities(abilities),
                PlayInstruction::SetHeldSlot(ClientboundSetHeldSlotPacket { slot: 3 }),
                PlayInstruction::UpdateRecipes,
                PlayInstruction::UpdatePermissionLevel(2),
                PlayInstruction::SendInitialRecipeBook,
                PlayInstruction::UpdateScoreboard,
                PlayInstruction::TeleportToSpawn { teleport_id: 0 },
                PlayInstruction::ServerStatus,
                PlayInstruction::PlayerInfoUpdate {
                    existing_players: 2,
                },
                PlayInstruction::BroadcastSelfPlayerInfo,
                PlayInstruction::SendLevelInfo,
                PlayInstruction::AddPlayerToLevel,
                PlayInstruction::BossEventsOnConnect,
                PlayInstruction::ActiveEffects { count: 1 },
                PlayInstruction::InitInventoryMenu,
            ]
        );
    }

    #[test]
    fn death_and_respawn_flow_match_player_list_respawn_packet_order() {
        let mut session = PlaySession::new(99, 0);
        assert_eq!(
            session.death_screen("{\"translate\":\"death.attack.generic\"}"),
            PlayInstruction::CombatKill(ClientboundPlayerCombatKillPacket {
                player_id: 99,
                message: "{\"translate\":\"death.attack.generic\"}".to_string(),
            })
        );

        let flow = session.respawn_flow(RespawnRequest {
            reason: RespawnReason::Death,
            keep_all_player_data: false,
            missing_respawn_block: true,
            hardcore: true,
            active_effect_count: 2,
            respawn_anchor_depleted: true,
            spawn_info: CommonPlayerSpawnInfo::default(),
        });

        assert_eq!(
            flow,
            vec![
                PlayInstruction::NoRespawnBlockAvailable,
                PlayInstruction::Respawn(ClientboundRespawnPacket {
                    spawn_info: CommonPlayerSpawnInfo::default(),
                    data_to_keep: RespawnDataToKeep::NONE,
                }),
                PlayInstruction::TeleportToSpawn { teleport_id: 0 },
                PlayInstruction::SetDefaultSpawnPosition,
                PlayInstruction::ChangeDifficulty {
                    difficulty: GameDifficulty::Normal,
                    locked: false,
                },
                PlayInstruction::SetExperience,
                PlayInstruction::ActiveEffects { count: 2 },
                PlayInstruction::SendLevelInfo,
                PlayInstruction::UpdatePermissionLevel(0),
                PlayInstruction::AddPlayerToLevel,
                PlayInstruction::InitInventoryMenu,
                PlayInstruction::SetHealth,
                PlayInstruction::SetGameModeSpectator,
                PlayInstruction::DisableSpectatorsGenerateChunks,
                PlayInstruction::RespawnAnchorDepleteSound,
            ]
        );
    }

    #[test]
    fn dimension_return_respawn_keeps_attribute_modifiers_like_vanilla_keep_all_path() {
        let mut session = PlaySession::new(99, 0);
        let flow = session.respawn_flow(RespawnRequest {
            reason: RespawnReason::WonGameReturnToOverworld,
            keep_all_player_data: true,
            missing_respawn_block: false,
            hardcore: false,
            active_effect_count: 0,
            respawn_anchor_depleted: false,
            spawn_info: CommonPlayerSpawnInfo {
                dimension: Identifier::parse("minecraft:overworld").unwrap(),
                previous_game_mode: Some(GameMode::Survival),
                ..CommonPlayerSpawnInfo::default()
            },
        });

        let PlayInstruction::Respawn(packet) = &flow[0] else {
            panic!("respawn packet should be first");
        };
        assert_eq!(packet.data_to_keep.bits(), 1);
        assert!(packet
            .data_to_keep
            .should_keep(RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS));
        assert!(!packet
            .data_to_keep
            .should_keep(RespawnDataToKeep::KEEP_ENTITY_DATA));
        assert!(!flow.contains(&PlayInstruction::SetGameModeSpectator));
    }

    #[test]
    fn player_loaded_packet_moves_session_to_playing() {
        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, Vec::new())),
            DispatchOutcome::Handled
        );
        assert_eq!(session.state, PlayState::Playing);
        assert!(session.loaded);
    }

    #[test]
    fn movement_packets_decode_flags_position_and_rotation_by_shape() {
        let movement = ServerboundMovePlayerPacket {
            x: 1.25,
            y: 65.0,
            z: -2.5,
            y_rot: 90.0,
            x_rot: 30.0,
            on_ground: true,
            horizontal_collision: true,
            has_position: true,
            has_rotation: true,
        };
        let mut payload = Vec::new();
        movement.write_pos_rot(&mut payload).unwrap();

        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, payload)),
            DispatchOutcome::Handled
        );
        let decoded = session.last_move.unwrap();
        assert_eq!(decoded.x, 1.25);
        assert_eq!(decoded.z, -2.5);
        assert_eq!(decoded.y_rot, 90.0);
        assert!(decoded.on_ground);
        assert!(decoded.horizontal_collision);
        assert!(decoded.has_position);
        assert!(decoded.has_rotation);
    }

    #[test]
    fn teleport_ack_and_held_slot_follow_play_state_validation() {
        let mut session = PlaySession::new(1, 0);
        session.pending_teleports.insert(7);

        let mut ack = Vec::new();
        ServerboundAcceptTeleportationPacket { teleport_id: 7 }
            .write(&mut ack)
            .unwrap();
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, ack)),
            DispatchOutcome::Handled
        );
        assert!(session.pending_teleports.is_empty());

        let mut held = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 8 }
            .write(&mut held)
            .unwrap();
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, held)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.selected_slot, 8);

        let mut invalid = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 9 }
            .write(&mut invalid)
            .unwrap();
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, invalid)),
            DispatchOutcome::Disconnect(_)
        ));
    }

    #[test]
    fn play_session_rejects_wrong_state_or_unknown_packets_and_can_reconfigure() {
        let mut session = PlaySession::new(1, 0);
        let wrong_state = DecodedPacket {
            state: ProtocolState::Configuration,
            direction: PacketDirection::Serverbound,
            id: SERVERBOUND_PLAYER_LOADED_PACKET_ID,
            payload: Vec::new(),
        };
        assert!(matches!(
            session.handle_decoded(wrong_state),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(999, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert_eq!(
            session.request_reconfiguration(),
            PlayInstruction::StartConfiguration
        );
        assert_eq!(session.state, PlayState::Reconfiguring);
    }

    #[test]
    fn small_play_packets_round_trip_vanilla_codecs() {
        let mut bytes = Vec::new();
        ClientboundSetHeldSlotPacket { slot: 4 }
            .write(&mut bytes)
            .unwrap();
        assert_eq!(
            ClientboundSetHeldSlotPacket::read(&mut cursor(bytes)).unwrap(),
            ClientboundSetHeldSlotPacket { slot: 4 }
        );

        let mut carried = Vec::new();
        ServerboundSetCarriedItemPacket { slot: 5 }
            .write(&mut carried)
            .unwrap();
        assert_eq!(
            ServerboundSetCarriedItemPacket::read(&mut cursor(carried)).unwrap(),
            ServerboundSetCarriedItemPacket { slot: 5 }
        );
    }
}
