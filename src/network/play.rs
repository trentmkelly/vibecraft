#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Write};

use crate::network::codec::{
    read_identifier, read_string, read_uuid, write_bitset, write_collection, write_enum_index,
    write_identifier, write_string, write_uuid, Uuid,
};
use crate::network::common::ServerboundResourcePackPacket;
use crate::network::dispatch::{DecodedPacket, DispatchOutcome, PacketDirection, ProtocolState};
use crate::network::varint::{read_var_i32, read_var_i64, write_var_i32, write_var_i64};
use crate::registry::Identifier;
use crate::storage::chunk::{ChunkSection, LevelChunk, PalettedContainer};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

pub const SERVERBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 69;
pub const CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2: usize = 141;

pub const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub const SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID: i32 = 4;
pub const SERVERBOUND_CHAT_ACK_PACKET_ID: i32 = 6;
pub const SERVERBOUND_CHAT_COMMAND_PACKET_ID: i32 = 7;
pub const SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID: i32 = 8;
pub const SERVERBOUND_CHAT_PACKET_ID: i32 = 9;
pub const SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID: i32 = 10;
pub const SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID: i32 = 11;
pub const SERVERBOUND_CLIENT_COMMAND_PACKET_ID: i32 = 12;
pub const SERVERBOUND_CLIENT_TICK_END_PACKET_ID: i32 = 13;
pub const SERVERBOUND_CLIENT_INFORMATION_PACKET_ID: i32 = 14;
pub const SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID: i32 = 15;
pub const SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID: i32 = 16;
pub const SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID: i32 = 17;
pub const SERVERBOUND_CONTAINER_CLICK_PACKET_ID: i32 = 18;
pub const SERVERBOUND_CONTAINER_CLOSE_PACKET_ID: i32 = 19;
pub const SERVERBOUND_EDIT_BOOK_PACKET_ID: i32 = 24;
pub const SERVERBOUND_INTERACT_PACKET_ID: i32 = 26;
pub const SERVERBOUND_JIGSAW_GENERATE_PACKET_ID: i32 = 27;
pub const SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID: i32 = 29;
pub const SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID: i32 = 30;
pub const SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID: i32 = 31;
pub const SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID: i32 = 32;
pub const SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID: i32 = 33;
pub const SERVERBOUND_MOVE_VEHICLE_PACKET_ID: i32 = 34;
pub const SERVERBOUND_KEEP_ALIVE_PACKET_ID: i32 = 28;
pub const SERVERBOUND_PLAYER_ACTION_PACKET_ID: i32 = 41;
pub const SERVERBOUND_PLAYER_COMMAND_PACKET_ID: i32 = 42;
pub const SERVERBOUND_PADDLE_BOAT_PACKET_ID: i32 = 35;
pub const SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID: i32 = 36;
pub const SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID: i32 = 37;
pub const SERVERBOUND_PLAYER_INPUT_PACKET_ID: i32 = 43;
pub const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub const SERVERBOUND_PONG_PACKET_ID: i32 = 45;
pub const SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID: i32 = 46;
pub const SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID: i32 = 47;
pub const SERVERBOUND_RENAME_ITEM_PACKET_ID: i32 = 48;
pub const SERVERBOUND_RESOURCE_PACK_PACKET_ID: i32 = 49;
pub const SERVERBOUND_SELECT_TRADE_PACKET_ID: i32 = 51;
pub const SERVERBOUND_SET_BEACON_PACKET_ID: i32 = 52;
pub const SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID: i32 = 53;
pub const SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID: i32 = 54;
pub const SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID: i32 = 55;
pub const SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID: i32 = 56;
pub const SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID: i32 = 59;
pub const SERVERBOUND_SIGN_UPDATE_PACKET_ID: i32 = 61;
pub const SERVERBOUND_SWING_PACKET_ID: i32 = 63;
pub const SERVERBOUND_USE_ITEM_ON_PACKET_ID: i32 = 66;
pub const SERVERBOUND_USE_ITEM_PACKET_ID: i32 = 67;

pub const CLIENTBOUND_LOGIN_PACKET_ID: i32 = 49;
pub const CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
pub const CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
pub const CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID: i32 = 10;
pub const CLIENTBOUND_ADD_ENTITY_PACKET_ID: i32 = 1;
pub const CLIENTBOUND_ANIMATE_PACKET_ID: i32 = 2;
pub const CLIENTBOUND_AWARD_STATS_PACKET_ID: i32 = 3;
pub const CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID: i32 = 4;
pub const CLIENTBOUND_BLOCK_UPDATE_PACKET_ID: i32 = 8;
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
pub const CLIENTBOUND_PING_PACKET_ID: i32 = 61;
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
pub const CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID: i32 = 35;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
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
pub struct ServerboundChangeDifficultyPacket {
    pub difficulty: GameDifficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundChatAckPacket {
    pub offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageSignature(pub [u8; MessageSignature::BYTES]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSeenMessagesUpdate {
    pub offset: i32,
    pub acknowledged: Vec<u8>,
    pub checksum: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentSignature {
    pub name: String,
    pub signature: MessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatPacket {
    pub message: String,
    pub timestamp_epoch_millis: i64,
    pub salt: i64,
    pub signature: Option<MessageSignature>,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandPacket {
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatCommandSignedPacket {
    pub command: String,
    pub timestamp_epoch_millis: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature>,
    pub last_seen_messages: LastSeenMessagesUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundChatSessionUpdatePacket {
    pub session_id: Uuid,
    pub expires_at_epoch_millis: i64,
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundClientTickEndPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundLockDifficultyPacket {
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPaddleBoatPacket {
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerInputPacket {
    pub input: ServerboundPlayerInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerLoadedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPlayerCommandAction {
    StopSleeping,
    StartSprinting,
    StopSprinting,
    StartRidingJump,
    StopRidingJump,
    OpenInventory,
    StartFallFlying,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerCommandPacket {
    pub entity_id: i32,
    pub action: ServerboundPlayerCommandAction,
    pub data: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction3d {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPlayerAction {
    StartDestroyBlock,
    AbortDestroyBlock,
    StopDestroyBlock,
    DropAllItems,
    DropItem,
    ReleaseUseItem,
    SwapItemWithOffhand,
    Stab,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPlayerActionPacket {
    pub action: ServerboundPlayerAction,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction3d,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundUseItemPacket {
    pub hand: ServerboundSwingHand,
    pub sequence: i32,
    pub y_rot: f32,
    pub x_rot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockHitResultPacketData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction3d,
    pub click_x: f32,
    pub click_y: f32,
    pub click_z: f32,
    pub inside: bool,
    pub world_border_hit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundUseItemOnPacket {
    pub hand: ServerboundSwingHand,
    pub block_hit: BlockHitResultPacketData,
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPongPacket {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundJigsawGeneratePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub levels: i32,
    pub keep_jigsaws: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSignUpdatePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub is_front_text: bool,
    pub lines: [String; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSetBeaconPacket {
    pub primary_effect_id: Option<i32>,
    pub secondary_effect_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCommandBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub command: String,
    pub mode: CommandBlockMode,
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCommandMinecartPacket {
    pub entity_id: i32,
    pub command: String,
    pub track_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureBlockUpdateType {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ServerboundSetStructureBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub update_type: StructureBlockUpdateType,
    pub mode: StructureBlockMode,
    pub name: String,
    pub offset: [i8; 3],
    pub size: [u8; 3],
    pub mirror: StructureMirror,
    pub rotation: StructureRotation,
    pub data: String,
    pub integrity: f32,
    pub seed: i64,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSelectTradePacket {
    pub item: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundRenameItemPacket {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundContainerClosePacket {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundContainerButtonClickPacket {
    pub container_id: i32,
    pub button_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDataComponentPatch {
    pub added: Vec<(i32, Vec<u8>)>,
    pub removed: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawItemStack {
    pub count: i32,
    pub item_id: Option<i32>,
    pub components: RawDataComponentPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedPatchMap {
    pub added_component_hashes: Vec<(i32, i32)>,
    pub removed_components: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedStack {
    pub item_id: Option<i32>,
    pub count: i32,
    pub components: HashedPatchMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundContainerClickPacket {
    pub container_id: i32,
    pub state_id: i32,
    pub slot_num: i16,
    pub button_num: i8,
    pub container_input: ContainerInput,
    pub changed_slots: BTreeMap<i32, HashedStack>,
    pub carried_item: HashedStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSetCreativeModeSlotPacket {
    pub slot_num: i16,
    pub item_stack: RawItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCommandSuggestionPacket {
    pub id: i32,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundEditBookPacket {
    pub slot: i32,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerboundInteractPacket {
    pub entity_id: i32,
    pub hand: ServerboundInteractionHand,
    pub location: Vec3,
    pub using_secondary_action: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundInteractionHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromBlockPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundPickItemFromEntityPacket {
    pub entity_id: i32,
    pub include_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundRecipeBookChangeSettingsPacket {
    pub book_type: RecipeBookType,
    pub is_open: bool,
    pub is_filtering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundRecipeBookSeenRecipePacket {
    pub recipe_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundConfigurationAcknowledgedPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundClientCommandAction {
    PerformRespawn,
    RequestStats,
    RequestGameruleValues,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundClientCommandPacket {
    pub action: ServerboundClientCommandAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundSwingHand {
    MainHand,
    OffHand,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundSwingPacket {
    pub hand: ServerboundSwingHand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundChangeDifficultyPacket {
    pub difficulty: GameDifficulty,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheCenterPacket {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetChunkCacheRadiusPacket {
    pub radius: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionData {
    pub dimension: Identifier,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionPacket {
    pub respawn_data: ClientboundSetDefaultSpawnPositionData,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetExperiencePacket {
    pub experience_progress: f32,
    pub experience_level: i32,
    pub total_experience: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetHealthPacket {
    pub health: f32,
    pub food: i32,
    pub saturation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetHeldSlotPacket {
    pub slot: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockDestructionPacket {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub progress: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockEventPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub action: u8,
    pub param: u8,
    pub block_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundBlockUpdatePacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_state_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundLevelEventPacket {
    pub event_type: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub data: i32,
    pub global_event: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPlayerInfoRemovePacket {
    pub profile_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundContainerClosePacket {
    pub container_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundContainerSetDataPacket {
    pub container_id: i32,
    pub id: i16,
    pub value: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundMountScreenOpenPacket {
    pub container_id: i32,
    pub inventory_columns: i32,
    pub entity_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCooldownPacket {
    pub cooldown_group: Identifier,
    pub duration: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerAbilitiesPacket {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub instant_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClockNetworkState {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetTimePacket {
    pub game_time: i64,
    pub clock_updates: BTreeMap<Identifier, ClockNetworkState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientboundGameEventType {
    NoRespawnBlockAvailable,
    StartRaining,
    StopRaining,
    ChangeGameMode,
    WinGame,
    DemoEvent,
    PlayArrowHitSound,
    RainLevelChange,
    ThunderLevelChange,
    PufferFishSting,
    GuardianElderEffect,
    ImmediateRespawn,
    LimitedCrafting,
    LevelChunksLoadStart,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundGameEventPacket {
    pub event: ClientboundGameEventType,
    pub param: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetSimulationDistancePacket {
    pub simulation_distance: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTickingStatePacket {
    pub tick_rate: f32,
    pub is_frozen: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundTickingStepPacket {
    pub tick_steps: i32,
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
pub struct ClientboundMoveVehiclePacket {
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundEntityPositionSyncPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerPositionPacket {
    pub id: i32,
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: u32,
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
pub struct ClientboundEntityEventPacket {
    pub entity_id: i32,
    pub event_id: i8,
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
pub struct ClientboundRemoveMobEffectPacket {
    pub entity_id: i32,
    pub effect_id: i32,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundInitializeBorderPacket {
    pub new_center_x: f64,
    pub new_center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
    pub new_absolute_max_size: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderCenterPacket {
    pub new_center_x: f64,
    pub new_center_z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderLerpSizePacket {
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundSetBorderSizePacket {
    pub size: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDelayPacket {
    pub warning_delay: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetBorderWarningDistancePacket {
    pub warning_blocks: i32,
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
pub struct ClientboundRecipeBookRemovePacket {
    pub recipe_display_ids: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeBookTypeSettings {
    pub open: bool,
    pub filtering: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    pub crafting: RecipeBookTypeSettings,
    pub furnace: RecipeBookTypeSettings,
    pub blast_furnace: RecipeBookTypeSettings,
    pub smoker: RecipeBookTypeSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAdvancementsPacket {
    pub reset: bool,
    pub added: Vec<Identifier>,
    pub removed: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundAwardStatsPacket {
    pub stats: Vec<AwardedStat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwardedStat {
    pub stat_type_id: i32,
    pub stat_value_id: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientboundPlayerLookAtPacket {
    pub from_anchor: EntityAnchor,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub target_entity: Option<(i32, EntityAnchor)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetTitleTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetSubtitleTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSetActionBarTextPacket {
    pub text: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundSystemChatPacket {
    pub content: Tag,
    pub overlay: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTabListPacket {
    pub header: Tag,
    pub footer: Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet = 0,
    Eyes = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResetScorePacket {
    pub owner: String,
    pub objective_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSetDisplayObjectivePacket {
    pub slot: i32,
    pub objective_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket {
    pub id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDebugSamplePacket {
    pub sample: Vec<i64>,
    pub sample_type: RemoteDebugSampleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteDebugSampleType {
    TickTime = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundStartConfigurationPacket;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundClearTitlesPacket {
    pub reset_times: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundSetTitlesAnimationPacket {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStopSoundPacket {
    pub source: Option<SoundSource>,
    pub name: Option<Identifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master = 0,
    Music = 1,
    Records = 2,
    Weather = 3,
    Blocks = 4,
    Hostile = 5,
    Neutral = 6,
    Players = 7,
    Ambient = 8,
    Voice = 9,
    Ui = 10,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundCommandSuggestionsPacket {
    pub transaction_id: i32,
    pub start: i32,
    pub length: i32,
    pub suggestions: Vec<CommandSuggestionEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSuggestionEntry {
    pub text: String,
    pub tooltip: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDeleteChatPacket {
    pub message_signature: PackedMessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedMessageSignature {
    CacheId(i32),
    Full(MessageSignature),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlockUpdate {
    pub packed_pos: u16,
    pub block_state_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSectionBlocksUpdatePacket {
    pub section_pos: SectionPos,
    pub updates: Vec<SectionBlockUpdate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundBlockEntityDataPacket {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block_entity_type_id: i32,
    pub tag: Tag,
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
    pub fluid_count: i16,
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

impl GameDifficulty {
    fn from_wire_index(index: i32) -> io::Result<Self> {
        match index {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid difficulty index",
            )),
        }
    }

    fn to_wire_index(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
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
    pub last_vehicle_move: Option<ServerboundMoveVehiclePacket>,
    pub last_chat_ack: Option<ServerboundChatAckPacket>,
    pub last_chat: Option<ServerboundChatPacket>,
    pub last_chat_command: Option<ServerboundChatCommandPacket>,
    pub last_signed_chat_command: Option<ServerboundChatCommandSignedPacket>,
    pub last_chat_session_update: Option<ServerboundChatSessionUpdatePacket>,
    pub last_player_command: Option<ServerboundPlayerCommandPacket>,
    pub last_player_action: Option<ServerboundPlayerActionPacket>,
    pub last_use_item: Option<ServerboundUseItemPacket>,
    pub last_use_item_on: Option<ServerboundUseItemOnPacket>,
    pub last_pong: Option<ServerboundPongPacket>,
    pub last_jigsaw_generate: Option<ServerboundJigsawGeneratePacket>,
    pub last_sign_update: Option<ServerboundSignUpdatePacket>,
    pub last_set_beacon: Option<ServerboundSetBeaconPacket>,
    pub last_set_command_block: Option<ServerboundSetCommandBlockPacket>,
    pub last_set_command_minecart: Option<ServerboundSetCommandMinecartPacket>,
    pub last_set_structure_block: Option<ServerboundSetStructureBlockPacket>,
    pub last_select_trade: Option<ServerboundSelectTradePacket>,
    pub last_rename_item: Option<ServerboundRenameItemPacket>,
    pub last_command_suggestion: Option<ServerboundCommandSuggestionPacket>,
    pub last_edit_book: Option<ServerboundEditBookPacket>,
    pub last_interact: Option<ServerboundInteractPacket>,
    pub last_resource_pack_response: Option<ServerboundResourcePackPacket>,
    pub last_container_close: Option<ServerboundContainerClosePacket>,
    pub last_container_button_click: Option<ServerboundContainerButtonClickPacket>,
    pub last_container_click: Option<ServerboundContainerClickPacket>,
    pub last_set_creative_mode_slot: Option<ServerboundSetCreativeModeSlotPacket>,
    pub last_pick_item_from_block: Option<ServerboundPickItemFromBlockPacket>,
    pub last_pick_item_from_entity: Option<ServerboundPickItemFromEntityPacket>,
    pub last_recipe_book_change_settings: Option<ServerboundRecipeBookChangeSettingsPacket>,
    pub last_recipe_book_seen_recipe: Option<ServerboundRecipeBookSeenRecipePacket>,
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
            last_vehicle_move: None,
            last_chat_ack: None,
            last_chat: None,
            last_chat_command: None,
            last_signed_chat_command: None,
            last_chat_session_update: None,
            last_player_command: None,
            last_player_action: None,
            last_use_item: None,
            last_use_item_on: None,
            last_pong: None,
            last_jigsaw_generate: None,
            last_sign_update: None,
            last_set_beacon: None,
            last_set_command_block: None,
            last_set_command_minecart: None,
            last_set_structure_block: None,
            last_select_trade: None,
            last_rename_item: None,
            last_command_suggestion: None,
            last_edit_book: None,
            last_interact: None,
            last_resource_pack_response: None,
            last_container_close: None,
            last_container_button_click: None,
            last_container_click: None,
            last_set_creative_mode_slot: None,
            last_pick_item_from_block: None,
            last_pick_item_from_entity: None,
            last_recipe_book_change_settings: None,
            last_recipe_book_seen_recipe: None,
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

        if matches!(self.state, PlayState::WaitingForPlayerLoaded)
            && is_command_like_play_packet(packet.id)
        {
            return DispatchOutcome::Disconnect(format!(
                "command packet {} before player_loaded",
                packet.id
            ));
        }

        match packet.id {
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
            SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChangeDifficultyPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad change difficulty packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CHAT_ACK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatAckPacket::read(&mut input) {
                    Ok(ack) => {
                        self.last_chat_ack = Some(ack);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad chat ack packet: {err}")),
                }
            }
            SERVERBOUND_CHAT_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatCommandPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_chat_command = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad chat command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatCommandSignedPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_signed_chat_command = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad signed chat command packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CHAT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatPacket::read(&mut input) {
                    Ok(chat) => {
                        self.last_chat = Some(chat);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad chat packet: {err}")),
                }
            }
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundChatSessionUpdatePacket::read(&mut input) {
                    Ok(update) => {
                        self.last_chat_session_update = Some(update);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad chat session update packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CLIENT_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundClientCommandPacket::read(&mut input) {
                    Ok(cmd) => {
                        if matches!(cmd.action, ServerboundClientCommandAction::Unknown(_)) {
                            DispatchOutcome::Disconnect("unknown client command action".to_string())
                        } else {
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad client command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CLIENT_TICK_END_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundClientTickEndPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad client tick end packet: {err}"))
                    }
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
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundCommandSuggestionPacket::read(&mut input) {
                    Ok(suggestion) => {
                        self.last_command_suggestion = Some(suggestion);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad command suggestion packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundConfigurationAcknowledgedPacket::read(&mut input) {
                    Ok(_) => {
                        self.state = PlayState::Reconfiguring;
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad configuration acknowledged packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerButtonClickPacket::read(&mut input) {
                    Ok(click) => {
                        self.last_container_button_click = Some(click);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad container button click packet: {err}"
                    )),
                }
            }
            SERVERBOUND_CONTAINER_CLICK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerClickPacket::read(&mut input) {
                    Ok(click) => {
                        self.last_container_click = Some(click);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad container click packet: {err}"))
                    }
                }
            }
            SERVERBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundContainerClosePacket::read(&mut input) {
                    Ok(close) => {
                        self.last_container_close = Some(close);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad container close packet: {err}"))
                    }
                }
            }
            SERVERBOUND_EDIT_BOOK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundEditBookPacket::read(&mut input) {
                    Ok(book) => {
                        self.last_edit_book = Some(book);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad edit book packet: {err}")),
                }
            }
            SERVERBOUND_INTERACT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundInteractPacket::read(&mut input) {
                    Ok(interact) => {
                        self.last_interact = Some(interact);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad interact packet: {err}")),
                }
            }
            SERVERBOUND_JIGSAW_GENERATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundJigsawGeneratePacket::read(&mut input) {
                    Ok(jigsaw) => {
                        self.last_jigsaw_generate = Some(jigsaw);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad jigsaw generate packet: {err}"))
                    }
                }
            }
            SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundLockDifficultyPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad lock difficulty packet: {err}"))
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
            SERVERBOUND_MOVE_VEHICLE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundMoveVehiclePacket::read(&mut input) {
                    Ok(packet) => {
                        self.last_vehicle_move = Some(packet);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad vehicle movement packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PADDLE_BOAT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPaddleBoatPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad paddle boat packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPickItemFromBlockPacket::read(&mut input) {
                    Ok(pick) => {
                        self.last_pick_item_from_block = Some(pick);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad pick item from block packet: {err}"
                    )),
                }
            }
            SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPickItemFromEntityPacket::read(&mut input) {
                    Ok(pick) => {
                        self.last_pick_item_from_entity = Some(pick);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad pick item from entity packet: {err}"
                    )),
                }
            }
            SERVERBOUND_PLAYER_INPUT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerInputPacket::read(&mut input) {
                    Ok(_) => DispatchOutcome::Handled,
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player input packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_LOADED_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerLoadedPacket::read(&mut input) {
                    Ok(_) => {
                        self.loaded = true;
                        self.state = PlayState::Playing;
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player loaded packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_COMMAND_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerCommandPacket::read(&mut input) {
                    Ok(command) => {
                        if matches!(command.action, ServerboundPlayerCommandAction::Unknown(_)) {
                            DispatchOutcome::Disconnect("unknown player command action".to_string())
                        } else {
                            self.last_player_command = Some(command);
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player command packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PLAYER_ACTION_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPlayerActionPacket::read(&mut input) {
                    Ok(action) => {
                        if matches!(action.action, ServerboundPlayerAction::Unknown(_)) {
                            DispatchOutcome::Disconnect(
                                "unknown player action packet action".to_string(),
                            )
                        } else {
                            self.last_player_action = Some(action);
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad player action packet: {err}"))
                    }
                }
            }
            SERVERBOUND_PONG_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundPongPacket::read(&mut input) {
                    Ok(pong) => {
                        self.last_pong = Some(pong);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad pong packet: {err}")),
                }
            }
            SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRecipeBookChangeSettingsPacket::read(&mut input) {
                    Ok(settings) => {
                        self.last_recipe_book_change_settings = Some(settings);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad recipe book change settings packet: {err}"
                    )),
                }
            }
            SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRecipeBookSeenRecipePacket::read(&mut input) {
                    Ok(recipe) => {
                        self.last_recipe_book_seen_recipe = Some(recipe);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad recipe book seen recipe packet: {err}"
                    )),
                }
            }
            SERVERBOUND_RENAME_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundRenameItemPacket::read(&mut input) {
                    Ok(rename_item) => {
                        self.last_rename_item = Some(rename_item);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad rename item packet: {err}"))
                    }
                }
            }
            SERVERBOUND_RESOURCE_PACK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundResourcePackPacket::read(&mut input) {
                    Ok(response) => {
                        self.last_resource_pack_response = Some(response);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad resource pack packet: {err}"))
                    }
                }
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
            SERVERBOUND_SET_BEACON_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetBeaconPacket::read(&mut input) {
                    Ok(beacon) => {
                        self.last_set_beacon = Some(beacon);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad set beacon packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCommandBlockPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_set_command_block = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad set command block packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCommandMinecartPacket::read(&mut input) {
                    Ok(command) => {
                        self.last_set_command_minecart = Some(command);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set command minecart packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetCreativeModeSlotPacket::read(&mut input) {
                    Ok(slot) => {
                        self.last_set_creative_mode_slot = Some(slot);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set creative mode slot packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSetStructureBlockPacket::read(&mut input) {
                    Ok(structure) => {
                        self.last_set_structure_block = Some(structure);
                        DispatchOutcome::Handled
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!(
                        "bad set structure block packet: {err}"
                    )),
                }
            }
            SERVERBOUND_SELECT_TRADE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSelectTradePacket::read(&mut input) {
                    Ok(select_trade) => {
                        self.last_select_trade = Some(select_trade);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad select trade packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SIGN_UPDATE_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSignUpdatePacket::read(&mut input) {
                    Ok(sign_update) => {
                        self.last_sign_update = Some(sign_update);
                        DispatchOutcome::Handled
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad sign update packet: {err}"))
                    }
                }
            }
            SERVERBOUND_SWING_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundSwingPacket::read(&mut input) {
                    Ok(swing) => {
                        if matches!(swing.hand, ServerboundSwingHand::Unknown(_)) {
                            DispatchOutcome::Disconnect("unknown swing hand".to_string())
                        } else {
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad swing packet: {err}")),
                }
            }
            SERVERBOUND_USE_ITEM_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundUseItemPacket::read(&mut input) {
                    Ok(use_item) => {
                        if matches!(use_item.hand, ServerboundSwingHand::Unknown(_)) {
                            DispatchOutcome::Disconnect(
                                "unknown use item interaction hand".to_string(),
                            )
                        } else {
                            self.last_use_item = Some(use_item);
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => DispatchOutcome::Disconnect(format!("bad use item packet: {err}")),
                }
            }
            SERVERBOUND_USE_ITEM_ON_PACKET_ID => {
                let mut input = &packet.payload[..];
                match ServerboundUseItemOnPacket::read(&mut input) {
                    Ok(use_item_on) => {
                        if matches!(use_item_on.hand, ServerboundSwingHand::Unknown(_)) {
                            DispatchOutcome::Disconnect(
                                "unknown use item on interaction hand".to_string(),
                            )
                        } else {
                            self.last_use_item_on = Some(use_item_on);
                            DispatchOutcome::Handled
                        }
                    }
                    Err(err) => {
                        DispatchOutcome::Disconnect(format!("bad use item on packet: {err}"))
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

fn is_command_like_play_packet(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_CHAT_COMMAND_PACKET_ID
            | SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID
            | SERVERBOUND_CHAT_PACKET_ID
            | SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID
    )
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

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    Ok(read_u8(reader)? != 0)
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn read_nullable_signature<R: Read>(reader: &mut R) -> io::Result<Option<MessageSignature>> {
    if read_bool(reader)? {
        Ok(Some(MessageSignature::read(reader)?))
    } else {
        Ok(None)
    }
}

fn write_nullable_signature<W: Write>(
    writer: &mut W,
    signature: Option<&MessageSignature>,
) -> io::Result<()> {
    match signature {
        Some(signature) => {
            write_bool(writer, true)?;
            signature.write(writer)
        }
        None => write_bool(writer, false),
    }
}

const BLOCK_POS_PACKED_HORIZONTAL_LENGTH: i64 = 26;
const BLOCK_POS_PACKED_Y_LENGTH: i64 = 12;
const BLOCK_POS_PACKED_X_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_POS_PACKED_Y_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_Y_LENGTH) - 1;
const BLOCK_POS_PACKED_Z_MASK: i64 = (1_i64 << BLOCK_POS_PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_POS_X_OFFSET: i64 = BLOCK_POS_PACKED_Y_LENGTH + BLOCK_POS_PACKED_HORIZONTAL_LENGTH;
const BLOCK_POS_Z_OFFSET: i64 = BLOCK_POS_PACKED_Y_LENGTH;

pub fn pack_block_position(x: i32, y: i32, z: i32) -> i64 {
    ((x as i64 & BLOCK_POS_PACKED_X_MASK) << BLOCK_POS_X_OFFSET)
        | ((z as i64 & BLOCK_POS_PACKED_Z_MASK) << BLOCK_POS_Z_OFFSET)
        | (y as i64 & BLOCK_POS_PACKED_Y_MASK)
}

pub fn unpack_block_position(packed: i64) -> (i32, i32, i32) {
    let x = (packed << (64 - (BLOCK_POS_X_OFFSET + BLOCK_POS_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_POS_PACKED_HORIZONTAL_LENGTH)) as i32;
    let y = (packed << (64 - BLOCK_POS_PACKED_Y_LENGTH) >> (64 - BLOCK_POS_PACKED_Y_LENGTH)) as i32;
    let z = (packed << (64 - (BLOCK_POS_Z_OFFSET + BLOCK_POS_PACKED_HORIZONTAL_LENGTH))
        >> (64 - BLOCK_POS_PACKED_HORIZONTAL_LENGTH)) as i32;
    (x, y, z)
}

fn read_block_position<R: Read>(reader: &mut R) -> io::Result<(i32, i32, i32)> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(unpack_block_position(i64::from_be_bytes(bytes)))
}

fn write_block_position<W: Write>(writer: &mut W, x: i32, y: i32, z: i32) -> io::Result<()> {
    writer.write_all(&pack_block_position(x, y, z).to_be_bytes())
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

impl SectionPos {
    pub fn packed_long(self) -> i64 {
        let x = (self.x as i64) & 0x3f_ffff;
        let y = (self.y as i64) & 0x0f_ffff;
        let z = (self.z as i64) & 0x3f_ffff;
        (x << 42) | y | (z << 20)
    }
}

impl ClientboundSectionBlocksUpdatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, self.section_pos.packed_long())?;
        write_var_i32(writer, self.updates.len() as i32)?;
        for update in &self.updates {
            if update.packed_pos > 0x0fff {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "section-relative block position exceeds 12 bits",
                ));
            }
            let packed_change =
                (i64::from(update.block_state_id) << 12) | i64::from(update.packed_pos);
            write_var_i64(writer, packed_change)?;
        }
        Ok(())
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

impl ClientboundRemoveEntitiesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_ids.len() as i32)?;
        for id in &self.entity_ids {
            write_var_i32(writer, *id)?;
        }
        Ok(())
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

    fn write_delta<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i16(writer, self.delta[0])?;
        write_i16(writer, self.delta[1])?;
        write_i16(writer, self.delta[2])
    }

    pub fn write_pos<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        self.write_delta(writer)?;
        write_bool(writer, self.on_ground)
    }

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        self.write_delta(writer)?;
        writer.write_all(&[self.y_rot, self.x_rot])?;
        write_bool(writer, self.on_ground)
    }

    pub fn write_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        writer.write_all(&[self.y_rot, self.x_rot])?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundMoveVehiclePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_vec3(writer, self.position)?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)
    }
}

fn write_position_move_rotation<W: Write>(
    writer: &mut W,
    position: Vec3,
    movement: Vec3,
    y_rot: f32,
    x_rot: f32,
) -> io::Result<()> {
    write_vec3(writer, position)?;
    write_vec3(writer, movement)?;
    write_f32(writer, y_rot)?;
    write_f32(writer, x_rot)
}

impl ClientboundTeleportEntityPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_i32(writer, self.relative_flags as i32)?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundEntityPositionSyncPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundPlayerPositionPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_i32(writer, self.relative_flags as i32)
    }
}

impl ClientboundSetEntityMotionPacket {
    pub fn new(id: i32, movement: Vec3) -> Self {
        Self {
            id,
            movement: clamp_velocity(movement),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_lp_vec3(writer, self.movement)
    }
}

impl ClientboundRotateHeadPacket {
    pub fn new(id: i32, y_head_rot: f32) -> Self {
        Self {
            id,
            y_head_rot: pack_degrees(y_head_rot),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_i8(writer, self.y_head_rot as i8)
    }
}

impl ClientboundSetPassengersPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.vehicle)?;
        write_var_i32(writer, self.passengers.len() as i32)?;
        for passenger in &self.passengers {
            write_var_i32(writer, *passenger)?;
        }
        Ok(())
    }
}

impl ClientboundEntityEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.entity_id)?;
        write_i8(writer, self.event_id)
    }
}

impl ClientboundAnimatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        writer.write_all(&[self.action as u8])
    }
}

impl ClientboundInitializeBorderPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.new_center_x)?;
        write_f64(writer, self.new_center_z)?;
        write_f64(writer, self.old_size)?;
        write_f64(writer, self.new_size)?;
        write_var_i64(writer, self.lerp_time)?;
        write_var_i32(writer, self.new_absolute_max_size)?;
        write_var_i32(writer, self.warning_blocks)?;
        write_var_i32(writer, self.warning_time)
    }
}

impl ClientboundSetBorderCenterPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.new_center_x)?;
        write_f64(writer, self.new_center_z)
    }
}

impl ClientboundSetBorderLerpSizePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.old_size)?;
        write_f64(writer, self.new_size)?;
        write_var_i64(writer, self.lerp_time)
    }
}

impl ClientboundSetBorderSizePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.size)
    }
}

impl ClientboundSetBorderWarningDelayPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.warning_delay)
    }
}

impl ClientboundSetBorderWarningDistancePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.warning_blocks)
    }
}

impl ClientboundClearTitlesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.reset_times)
    }
}

impl ClientboundSetTitlesAnimationPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.fade_in)?;
        write_i32(writer, self.stay)?;
        write_i32(writer, self.fade_out)
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

impl ClientboundRecipeBookRemovePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(
            writer,
            &self.recipe_display_ids,
            |writer, recipe_display_id| write_var_i32(writer, *recipe_display_id),
        )
    }
}

impl RecipeBookTypeSettings {
    pub const CLOSED_UNFILTERED: Self = Self {
        open: false,
        filtering: false,
    };

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.open)?;
        write_bool(writer, self.filtering)
    }
}

impl ClientboundRecipeBookSettingsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.crafting.write(writer)?;
        self.furnace.write(writer)?;
        self.blast_furnace.write(writer)?;
        self.smoker.write(writer)
    }
}

impl ClientboundCommandSuggestionsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.transaction_id)?;
        write_var_i32(writer, self.start)?;
        write_var_i32(writer, self.length)?;
        write_var_i32(writer, self.suggestions.len() as i32)?;
        for suggestion in &self.suggestions {
            write_string(writer, &suggestion.text, 32767)?;
            match &suggestion.tooltip {
                Some(tooltip) => {
                    write_bool(writer, true)?;
                    write_network_tag(writer, tooltip)?;
                }
                None => write_bool(writer, false)?,
            }
        }
        Ok(())
    }
}

impl ClientboundDebugSamplePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.sample.len() as i32)?;
        for value in &self.sample {
            write_i64(writer, *value)?;
        }
        write_enum_index(
            writer,
            self.sample_type as usize,
            RemoteDebugSampleType::COUNT,
        )
    }
}

impl RemoteDebugSampleType {
    const COUNT: usize = 1;
}

impl ClientboundStartConfigurationPacket {
    pub fn write<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ClientboundStopSoundPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let flags =
            (if self.source.is_some() { 1 } else { 0 }) | (if self.name.is_some() { 2 } else { 0 });
        writer.write_all(&[flags])?;
        if let Some(source) = self.source {
            write_enum_index(writer, source as usize, SoundSource::COUNT)?;
        }
        if let Some(name) = &self.name {
            write_identifier(writer, name)?;
        }
        Ok(())
    }
}

impl SoundSource {
    const COUNT: usize = 11;
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

impl ClientboundRemoveMobEffectPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.effect_id)
    }
}

impl ClientboundUpdateMobEffectPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.effect_id)?;
        write_var_i32(writer, self.amplifier)?;
        write_var_i32(writer, self.duration_ticks)?;
        writer.write_all(&[self.flags.0])
    }
}

impl ClientboundPlayerLookAtPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_enum_index(writer, self.from_anchor as usize, EntityAnchor::COUNT)?;
        write_f64(writer, self.x)?;
        write_f64(writer, self.y)?;
        write_f64(writer, self.z)?;
        match self.target_entity {
            Some((entity_id, to_anchor)) => {
                write_bool(writer, true)?;
                write_var_i32(writer, entity_id)?;
                write_enum_index(writer, to_anchor as usize, EntityAnchor::COUNT)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ClientboundSetTitleTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSetSubtitleTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSetActionBarTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSystemChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.content)?;
        write_bool(writer, self.overlay)
    }
}

impl ClientboundTabListPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.header)?;
        write_network_tag(writer, &self.footer)
    }
}

impl EntityAnchor {
    const COUNT: usize = 2;
}

impl ClientboundResetScorePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.owner, 32767)?;
        match &self.objective_name {
            Some(objective_name) => {
                write_bool(writer, true)?;
                write_string(writer, objective_name, 32767)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ClientboundSetDisplayObjectivePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)?;
        write_string(writer, &self.objective_name, 32767)
    }
}

impl ClientboundResourcePackPopPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.id {
            Some(id) => {
                write_bool(writer, true)?;
                write_uuid(writer, id)
            }
            None => write_bool(writer, false),
        }
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
            non_empty_block_count: section_non_empty_block_count(&section.block_states),
            fluid_count: 0,
            block_states: NetworkPalettedContainer::from_storage_container(
                &section.block_states,
                PaletteKind::BlockState,
            ),
            biomes: NetworkPalettedContainer::from_storage_container(
                &section.biomes,
                PaletteKind::Biome,
            ),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.non_empty_block_count.to_be_bytes())?;
        writer.write_all(&self.fluid_count.to_be_bytes())?;
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

    fn from_storage_container(tag: &Tag, kind: PaletteKind) -> Self {
        let Ok(container) = PalettedContainer::from_nbt(tag, 0) else {
            return Self::single(0);
        };
        let palette_ids = container
            .palette
            .iter()
            .map(|entry| storage_palette_entry_network_id(entry, kind))
            .collect::<Vec<_>>();
        let data = container.data.unwrap_or_default();
        Self {
            bits_per_entry: if data.is_empty() {
                0
            } else {
                packed_storage_bits_per_entry(container.palette.len()) as u8
            },
            palette_ids: if palette_ids.is_empty() {
                vec![0]
            } else {
                palette_ids
            },
            data,
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
        for word in &self.data {
            writer.write_all(&word.to_be_bytes())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteKind {
    BlockState,
    Biome,
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
    write_var_i32(writer, layer.len() as i32)?;
    let bytes = layer.iter().map(|byte| *byte as u8).collect::<Vec<_>>();
    writer.write_all(&bytes)
}

fn storage_palette_entry_network_id(tag: &Tag, kind: PaletteKind) -> i32 {
    match tag {
        Tag::Int(id) => *id,
        Tag::String(name) => match kind {
            PaletteKind::BlockState => block_state_name_network_id(name).unwrap_or(0),
            PaletteKind::Biome => biome_name_network_id(name).unwrap_or(0),
        },
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
            .or_else(|| {
                fields.iter().find_map(|(name, value)| {
                    (name == "Name")
                        .then_some(value)
                        .and_then(|value| match value {
                            Tag::String(name) => match kind {
                                PaletteKind::BlockState => block_state_name_network_id(name),
                                PaletteKind::Biome => biome_name_network_id(name),
                            },
                            _ => None,
                        })
                })
            })
            .unwrap_or(0),
        _ => 0,
    }
}

fn section_non_empty_block_count(tag: &Tag) -> i16 {
    let Ok(container) = PalettedContainer::from_nbt(tag, 4096) else {
        return 0;
    };
    let non_air = container
        .palette
        .iter()
        .map(|entry| !storage_palette_entry_is_air(entry))
        .collect::<Vec<_>>();
    if non_air.is_empty() {
        return 0;
    }
    let Some(data) = &container.data else {
        return if non_air.first().copied().unwrap_or(false) {
            4096
        } else {
            0
        };
    };

    let bits_per_entry = packed_storage_bits_per_entry(container.palette.len());
    let values_per_long = 64 / bits_per_entry;
    let mut count = 0_i16;
    for index in 0..container.expected_entries {
        let word_index = index / values_per_long;
        let Some(word) = data.get(word_index) else {
            break;
        };
        let bit_index = (index - word_index * values_per_long) * bits_per_entry;
        let palette_index = ((*word as u64) >> bit_index) & ((1_u64 << bits_per_entry) - 1);
        if non_air
            .get(palette_index as usize)
            .copied()
            .unwrap_or(false)
        {
            count += 1;
        }
    }
    count
}

fn packed_storage_bits_per_entry(palette_len: usize) -> usize {
    let palette_len = palette_len.max(1) as u64;
    let needed = 64 - palette_len.saturating_sub(1).leading_zeros() as usize;
    needed.max(4)
}

fn storage_palette_entry_is_air(tag: &Tag) -> bool {
    match tag {
        Tag::Int(id) => *id == 0,
        Tag::Compound(fields) => fields.iter().any(|(name, value)| {
            (name == "Name" || name == "id")
                && matches!(value, Tag::String(block_name) if block_name == "minecraft:air")
        }),
        _ => true,
    }
}

fn block_state_name_network_id(name: &str) -> Option<i32> {
    Some(match name {
        "minecraft:air" => 0,
        "minecraft:stone" => 1,
        "minecraft:granite" => 2,
        "minecraft:diorite" => 4,
        "minecraft:andesite" => 6,
        "minecraft:grass_block" => 9,
        "minecraft:dirt" => 10,
        "minecraft:sand" => 118,
        "minecraft:sandstone" => 578,
        "minecraft:water" => 86,
        "minecraft:oak_log" => 137,
        "minecraft:oak_leaves" => 279,
        "minecraft:bedrock" => 85,
        "minecraft:deepslate" => 27924,
        "minecraft:short_grass" => 2248,
        "minecraft:dandelion" => 2321,
        "minecraft:poppy" => 2324,
        "minecraft:birch_log" => 143,
        "minecraft:birch_leaves" => 335,
        "minecraft:sunflower" => 12916,
        _ => return None,
    })
}

fn biome_name_network_id(name: &str) -> Option<i32> {
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    crate::network::status::BIOMES
        .iter()
        .position(|biome| *biome == key)
        .map(|index| index as i32)
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

impl ServerboundChangeDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            difficulty: GameDifficulty::from_wire_index(read_var_i32(reader)?)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.difficulty.to_wire_index())
    }
}

impl ServerboundChatAckPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            offset: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.offset)
    }
}

impl MessageSignature {
    pub const BYTES: usize = 256;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; Self::BYTES];
        reader.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl PackedMessageSignature {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::CacheId(id) => write_var_i32(writer, id + 1),
            Self::Full(signature) => {
                write_var_i32(writer, 0)?;
                signature.write(writer)
            }
        }
    }
}

impl ClientboundDeleteChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.message_signature.write(writer)
    }
}

impl LastSeenMessagesUpdate {
    pub const ACKNOWLEDGED_BITS: usize = 20;
    pub const ACKNOWLEDGED_BYTES: usize = 3;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let offset = read_var_i32(reader)?;
        let mut acknowledged = vec![0; Self::ACKNOWLEDGED_BYTES];
        reader.read_exact(&mut acknowledged)?;
        Ok(Self {
            offset,
            acknowledged,
            checksum: read_u8(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.acknowledged.len() != Self::ACKNOWLEDGED_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "last-seen acknowledged bitset must be 3 bytes",
            ));
        }
        if self.acknowledged[2] & !0x0f != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "last-seen acknowledged bitset exceeds 20 bits",
            ));
        }
        write_var_i32(writer, self.offset)?;
        writer.write_all(&self.acknowledged)?;
        writer.write_all(&[self.checksum])
    }
}

impl ArgumentSignature {
    pub const MAX_ARGUMENT_NAME_CHARS: usize = 16;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, Self::MAX_ARGUMENT_NAME_CHARS)?,
            signature: MessageSignature::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, Self::MAX_ARGUMENT_NAME_CHARS)?;
        self.signature.write(writer)
    }
}

impl ServerboundChatPacket {
    pub const MAX_MESSAGE_CHARS: usize = 256;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            message: read_string(reader, Self::MAX_MESSAGE_CHARS)?,
            timestamp_epoch_millis: read_i64(reader)?,
            salt: read_i64(reader)?,
            signature: read_nullable_signature(reader)?,
            last_seen_messages: LastSeenMessagesUpdate::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.message, Self::MAX_MESSAGE_CHARS)?;
        write_i64(writer, self.timestamp_epoch_millis)?;
        write_i64(writer, self.salt)?;
        write_nullable_signature(writer, self.signature.as_ref())?;
        self.last_seen_messages.write(writer)
    }
}

impl ServerboundChatCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            command: read_string(reader, 32767)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.command, 32767)
    }
}

impl ServerboundChatCommandSignedPacket {
    pub const MAX_ARGUMENT_SIGNATURES: usize = 8;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            command: read_string(reader, 32767)?,
            timestamp_epoch_millis: read_i64(reader)?,
            salt: read_i64(reader)?,
            argument_signatures: read_limited_collection(
                reader,
                Self::MAX_ARGUMENT_SIGNATURES,
                ArgumentSignature::read,
            )?,
            last_seen_messages: LastSeenMessagesUpdate::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.argument_signatures.len() > Self::MAX_ARGUMENT_SIGNATURES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many argument signatures",
            ));
        }
        write_string(writer, &self.command, 32767)?;
        write_i64(writer, self.timestamp_epoch_millis)?;
        write_i64(writer, self.salt)?;
        write_collection(writer, &self.argument_signatures, |writer, entry| {
            entry.write(writer)
        })?;
        self.last_seen_messages.write(writer)
    }
}

impl ServerboundChatSessionUpdatePacket {
    pub const MAX_PUBLIC_KEY_BYTES: usize = 512;
    pub const MAX_SIGNATURE_BYTES: usize = 4096;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            session_id: read_uuid(reader)?,
            expires_at_epoch_millis: read_i64(reader)?,
            public_key: read_length_prefixed_bytes(reader, Self::MAX_PUBLIC_KEY_BYTES)?,
            key_signature: read_length_prefixed_bytes(reader, Self::MAX_SIGNATURE_BYTES)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, self.session_id)?;
        write_i64(writer, self.expires_at_epoch_millis)?;
        write_length_prefixed_bytes(writer, &self.public_key, Self::MAX_PUBLIC_KEY_BYTES)?;
        write_length_prefixed_bytes(writer, &self.key_signature, Self::MAX_SIGNATURE_BYTES)
    }
}

impl ServerboundClientCommandAction {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::PerformRespawn,
            1 => Self::RequestStats,
            2 => Self::RequestGameruleValues,
            _ => Self::Unknown(id as u8),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::PerformRespawn => 0,
            Self::RequestStats => 1,
            Self::RequestGameruleValues => 2,
            Self::Unknown(value) => i32::from(value),
        }
    }
}

impl ServerboundClientCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            action: ServerboundClientCommandAction::from_id(read_var_i32(reader)?),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.action.to_id())
    }
}

impl ServerboundClientTickEndPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundLockDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            locked: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.locked)
    }
}

impl ServerboundPaddleBoatPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            left: read_bool(reader)?,
            right: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.left)?;
        write_bool(writer, self.right)
    }
}

impl ServerboundPlayerInput {
    fn from_flags(flags: u8) -> Self {
        Self {
            forward: flags & 1 != 0,
            backward: flags & 2 != 0,
            left: flags & 4 != 0,
            right: flags & 8 != 0,
            jump: flags & 16 != 0,
            shift: flags & 32 != 0,
            sprint: flags & 64 != 0,
        }
    }

    fn to_flags(self) -> u8 {
        (if self.forward { 1 } else { 0 })
            | (if self.backward { 2 } else { 0 })
            | (if self.left { 4 } else { 0 })
            | (if self.right { 8 } else { 0 })
            | (if self.jump { 16 } else { 0 })
            | (if self.shift { 32 } else { 0 })
            | (if self.sprint { 64 } else { 0 })
    }
}

impl ServerboundPlayerInputPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let flags = read_u8(reader)?;
        Ok(Self {
            input: ServerboundPlayerInput::from_flags(flags),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.input.to_flags()])
    }
}

impl ServerboundPlayerLoadedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundPlayerCommandAction {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::StopSleeping,
            1 => Self::StartSprinting,
            2 => Self::StopSprinting,
            3 => Self::StartRidingJump,
            4 => Self::StopRidingJump,
            5 => Self::OpenInventory,
            6 => Self::StartFallFlying,
            _ => Self::Unknown(id),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::StopSleeping => 0,
            Self::StartSprinting => 1,
            Self::StopSprinting => 2,
            Self::StartRidingJump => 3,
            Self::StopRidingJump => 4,
            Self::OpenInventory => 5,
            Self::StartFallFlying => 6,
            Self::Unknown(value) => value,
        }
    }
}

impl ServerboundPlayerCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            action: ServerboundPlayerCommandAction::from_id(read_var_i32(reader)?),
            data: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.action.to_id())?;
        write_var_i32(writer, self.data)
    }
}

impl Direction3d {
    fn from_id(id: u8) -> Self {
        match id % 6 {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            _ => Self::East,
        }
    }

    fn to_id(self) -> u8 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    fn from_enum_id(id: i32) -> Self {
        match id.rem_euclid(6) {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            _ => Self::East,
        }
    }

    fn to_enum_id(self) -> i32 {
        i32::from(self.to_id())
    }
}

impl ServerboundPlayerAction {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::StartDestroyBlock,
            1 => Self::AbortDestroyBlock,
            2 => Self::StopDestroyBlock,
            3 => Self::DropAllItems,
            4 => Self::DropItem,
            5 => Self::ReleaseUseItem,
            6 => Self::SwapItemWithOffhand,
            7 => Self::Stab,
            _ => Self::Unknown(id),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::StartDestroyBlock => 0,
            Self::AbortDestroyBlock => 1,
            Self::StopDestroyBlock => 2,
            Self::DropAllItems => 3,
            Self::DropItem => 4,
            Self::ReleaseUseItem => 5,
            Self::SwapItemWithOffhand => 6,
            Self::Stab => 7,
            Self::Unknown(value) => value,
        }
    }
}

impl ServerboundPlayerActionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let action = ServerboundPlayerAction::from_id(read_var_i32(reader)?);
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            action,
            x,
            y,
            z,
            direction: Direction3d::from_id(read_u8(reader)?),
            sequence: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.action.to_id())?;
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.direction.to_id()])?;
        write_var_i32(writer, self.sequence)
    }
}

impl ServerboundSwingHand {
    fn from_id(id: i32) -> Self {
        match id {
            0 => Self::MainHand,
            1 => Self::OffHand,
            _ => Self::Unknown(id as u8),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
            Self::Unknown(value) => i32::from(value),
        }
    }
}

impl ServerboundSwingPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())
    }
}

impl ServerboundUseItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?),
            sequence: read_var_i32(reader)?,
            y_rot: read_f32(reader)?,
            x_rot: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())?;
        write_var_i32(writer, self.sequence)?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)
    }
}

impl BlockHitResultPacketData {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            direction: Direction3d::from_enum_id(read_var_i32(reader)?),
            click_x: read_f32(reader)?,
            click_y: read_f32(reader)?,
            click_z: read_f32(reader)?,
            inside: read_bool(reader)?,
            world_border_hit: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.direction.to_enum_id())?;
        write_f32(writer, self.click_x)?;
        write_f32(writer, self.click_y)?;
        write_f32(writer, self.click_z)?;
        write_bool(writer, self.inside)?;
        write_bool(writer, self.world_border_hit)
    }
}

impl ServerboundUseItemOnPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?),
            block_hit: BlockHitResultPacketData::read(reader)?,
            sequence: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())?;
        self.block_hit.write(writer)?;
        write_var_i32(writer, self.sequence)
    }
}

impl ServerboundPongPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            id: i32::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ServerboundConfigurationAcknowledgedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundJigsawGeneratePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            levels: read_var_i32(reader)?,
            keep_jigsaws: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.levels)?;
        write_bool(writer, self.keep_jigsaws)
    }
}

impl ServerboundSignUpdatePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let is_front_text = read_bool(reader)?;
        Ok(Self {
            x,
            y,
            z,
            is_front_text,
            lines: [
                read_string(reader, 384)?,
                read_string(reader, 384)?,
                read_string(reader, 384)?,
                read_string(reader, 384)?,
            ],
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_bool(writer, self.is_front_text)?;
        for line in &self.lines {
            write_string(writer, line, 384)?;
        }
        Ok(())
    }
}

impl ServerboundSetBeaconPacket {
    fn read_optional_mob_effect<R: Read>(reader: &mut R) -> io::Result<Option<i32>> {
        if read_bool(reader)? {
            Ok(Some(read_var_i32(reader)?))
        } else {
            Ok(None)
        }
    }

    fn write_optional_mob_effect<W: Write>(
        writer: &mut W,
        effect_id: Option<i32>,
    ) -> io::Result<()> {
        write_bool(writer, effect_id.is_some())?;
        if let Some(effect_id) = effect_id {
            write_var_i32(writer, effect_id)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            primary_effect_id: Self::read_optional_mob_effect(reader)?,
            secondary_effect_id: Self::read_optional_mob_effect(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Self::write_optional_mob_effect(writer, self.primary_effect_id)?;
        Self::write_optional_mob_effect(writer, self.secondary_effect_id)
    }
}

impl CommandBlockMode {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Sequence),
            1 => Ok(Self::Auto),
            2 => Ok(Self::Redstone),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid command block mode {id}"),
            )),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Sequence => 0,
            Self::Auto => 1,
            Self::Redstone => 2,
        }
    }
}

impl ServerboundSetCommandBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let command = read_string(reader, 32767)?;
        let mode = CommandBlockMode::from_id(read_var_i32(reader)?)?;
        let flags = read_u8(reader)?;
        Ok(Self {
            x,
            y,
            z,
            command,
            mode,
            track_output: flags & 1 != 0,
            conditional: flags & 2 != 0,
            automatic: flags & 4 != 0,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_string(writer, &self.command, 32767)?;
        write_var_i32(writer, self.mode.to_id())?;
        let flags = (if self.track_output { 1 } else { 0 })
            | (if self.conditional { 2 } else { 0 })
            | (if self.automatic { 4 } else { 0 });
        writer.write_all(&[flags])
    }
}

impl ServerboundSetCommandMinecartPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            command: read_string(reader, 32767)?,
            track_output: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_string(writer, &self.command, 32767)?;
        write_bool(writer, self.track_output)
    }
}

impl StructureBlockUpdateType {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::UpdateData),
            1 => Ok(Self::SaveArea),
            2 => Ok(Self::LoadArea),
            3 => Ok(Self::ScanArea),
            _ => Err(invalid_data("invalid structure block update type")),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::UpdateData => 0,
            Self::SaveArea => 1,
            Self::LoadArea => 2,
            Self::ScanArea => 3,
        }
    }
}

impl StructureBlockMode {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Save),
            1 => Ok(Self::Load),
            2 => Ok(Self::Corner),
            3 => Ok(Self::Data),
            _ => Err(invalid_data("invalid structure block mode")),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Save => 0,
            Self::Load => 1,
            Self::Corner => 2,
            Self::Data => 3,
        }
    }
}

impl StructureMirror {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::LeftRight),
            2 => Ok(Self::FrontBack),
            _ => Err(invalid_data("invalid structure mirror")),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::LeftRight => 1,
            Self::FrontBack => 2,
        }
    }
}

impl StructureRotation {
    fn from_id(id: i32) -> io::Result<Self> {
        match id.rem_euclid(4) {
            0 => Ok(Self::None),
            1 => Ok(Self::Clockwise90),
            2 => Ok(Self::Clockwise180),
            _ => Ok(Self::Counterclockwise90),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }
}

impl ServerboundSetStructureBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let update_type = StructureBlockUpdateType::from_id(read_var_i32(reader)?)?;
        let mode = StructureBlockMode::from_id(read_var_i32(reader)?)?;
        let name = read_string(reader, 32767)?;
        let offset = [
            read_clamped_i8(reader, -48, 48)?,
            read_clamped_i8(reader, -48, 48)?,
            read_clamped_i8(reader, -48, 48)?,
        ];
        let size = [
            read_clamped_i8(reader, 0, 48)? as u8,
            read_clamped_i8(reader, 0, 48)? as u8,
            read_clamped_i8(reader, 0, 48)? as u8,
        ];
        let mirror = StructureMirror::from_id(read_var_i32(reader)?)?;
        let rotation = StructureRotation::from_id(read_var_i32(reader)?)?;
        let data = read_string(reader, 128)?;
        let integrity = read_f32(reader)?.clamp(0.0, 1.0);
        let seed = read_var_i64(reader)?;
        let flags = read_u8(reader)?;
        Ok(Self {
            x,
            y,
            z,
            update_type,
            mode,
            name,
            offset,
            size,
            mirror,
            rotation,
            data,
            integrity,
            seed,
            ignore_entities: flags & 1 != 0,
            strict: flags & 8 != 0,
            show_air: flags & 2 != 0,
            show_bounding_box: flags & 4 != 0,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.update_type.to_id())?;
        write_var_i32(writer, self.mode.to_id())?;
        write_string(writer, &self.name, 32767)?;
        for value in self.offset {
            writer.write_all(&[value as u8])?;
        }
        for value in self.size {
            writer.write_all(&[value])?;
        }
        write_var_i32(writer, self.mirror.to_id())?;
        write_var_i32(writer, self.rotation.to_id())?;
        write_string(writer, &self.data, 128)?;
        write_f32(writer, self.integrity)?;
        write_var_i64(writer, self.seed)?;
        let flags = (if self.ignore_entities { 1 } else { 0 })
            | (if self.show_air { 2 } else { 0 })
            | (if self.show_bounding_box { 4 } else { 0 })
            | (if self.strict { 8 } else { 0 });
        writer.write_all(&[flags])
    }
}

impl ServerboundSelectTradePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            item: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.item)
    }
}

impl ServerboundRenameItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, 32767)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 32767)
    }
}

impl ServerboundContainerClosePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            container_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)
    }
}

impl ServerboundContainerButtonClickPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            container_id: read_var_i32(reader)?,
            button_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.button_id)
    }
}

impl RawDataComponentPatch {
    pub fn empty() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
        }
    }

    pub fn read_delimited<R: Read>(reader: &mut R) -> io::Result<Self> {
        let added_count = read_limited_len(reader, 65536, "data component add count")?;
        let removed_count = read_limited_len(reader, 65536, "data component remove count")?;
        let mut added = Vec::with_capacity(added_count);
        for _ in 0..added_count {
            let component_type_id = read_var_i32(reader)?;
            let payload = read_length_prefixed_bytes(reader, i32::MAX as usize)?;
            added.push((component_type_id, payload));
        }
        let mut removed = Vec::with_capacity(removed_count);
        for _ in 0..removed_count {
            removed.push(read_var_i32(reader)?);
        }
        Ok(Self { added, removed })
    }

    pub fn write_delimited<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.added.len() as i32)?;
        write_var_i32(writer, self.removed.len() as i32)?;
        for (component_type_id, payload) in &self.added {
            write_var_i32(writer, *component_type_id)?;
            write_length_prefixed_bytes(writer, payload, i32::MAX as usize)?;
        }
        for component_type_id in &self.removed {
            write_var_i32(writer, *component_type_id)?;
        }
        Ok(())
    }
}

impl RawItemStack {
    pub fn empty() -> Self {
        Self {
            count: 0,
            item_id: None,
            components: RawDataComponentPatch::empty(),
        }
    }

    pub fn read_optional_untrusted<R: Read>(reader: &mut R) -> io::Result<Self> {
        let count = read_var_i32(reader)?;
        if count <= 0 {
            return Ok(Self::empty());
        }
        Ok(Self {
            count,
            item_id: Some(read_var_i32(reader)?),
            components: RawDataComponentPatch::read_delimited(reader)?,
        })
    }

    pub fn write_optional_untrusted<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.count <= 0 {
            return write_var_i32(writer, 0);
        }
        let item_id = self.item_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "non-empty item stack missing item id",
            )
        })?;
        write_var_i32(writer, self.count)?;
        write_var_i32(writer, item_id)?;
        self.components.write_delimited(writer)
    }
}

impl HashedPatchMap {
    pub const MAX_HASHED_COMPONENTS: usize = 256;

    pub fn empty() -> Self {
        Self {
            added_component_hashes: Vec::new(),
            removed_components: Vec::new(),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let added_len = read_limited_len(
            reader,
            Self::MAX_HASHED_COMPONENTS,
            "hashed patch add count",
        )?;
        let mut added_component_hashes = Vec::with_capacity(added_len);
        for _ in 0..added_len {
            added_component_hashes.push((read_var_i32(reader)?, read_i32(reader)?));
        }
        let removed_len = read_limited_len(
            reader,
            Self::MAX_HASHED_COMPONENTS,
            "hashed patch remove count",
        )?;
        let mut removed_components = Vec::with_capacity(removed_len);
        for _ in 0..removed_len {
            removed_components.push(read_var_i32(reader)?);
        }
        Ok(Self {
            added_component_hashes,
            removed_components,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.added_component_hashes.len() > Self::MAX_HASHED_COMPONENTS
            || self.removed_components.len() > Self::MAX_HASHED_COMPONENTS
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many hashed patch components",
            ));
        }
        write_var_i32(writer, self.added_component_hashes.len() as i32)?;
        for (component_type_id, hash) in &self.added_component_hashes {
            write_var_i32(writer, *component_type_id)?;
            write_i32(writer, *hash)?;
        }
        write_var_i32(writer, self.removed_components.len() as i32)?;
        for component_type_id in &self.removed_components {
            write_var_i32(writer, *component_type_id)?;
        }
        Ok(())
    }
}

impl HashedStack {
    pub fn empty() -> Self {
        Self {
            item_id: None,
            count: 0,
            components: HashedPatchMap::empty(),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        if !read_bool(reader)? {
            return Ok(Self::empty());
        }
        Ok(Self {
            item_id: Some(read_var_i32(reader)?),
            count: read_var_i32(reader)?,
            components: HashedPatchMap::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.item_id {
            Some(item_id) => {
                write_bool(writer, true)?;
                write_var_i32(writer, item_id)?;
                write_var_i32(writer, self.count)?;
                self.components.write(writer)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ContainerInput {
    fn from_wire_id(id: i32) -> Self {
        match id {
            1 => Self::QuickMove,
            2 => Self::Swap,
            3 => Self::Clone,
            4 => Self::Throw,
            5 => Self::QuickCraft,
            6 => Self::PickupAll,
            _ => Self::Pickup,
        }
    }

    fn to_wire_id(self) -> i32 {
        match self {
            Self::Pickup => 0,
            Self::QuickMove => 1,
            Self::Swap => 2,
            Self::Clone => 3,
            Self::Throw => 4,
            Self::QuickCraft => 5,
            Self::PickupAll => 6,
        }
    }
}

impl ServerboundContainerClickPacket {
    pub const MAX_CHANGED_SLOTS: usize = 128;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let container_id = read_var_i32(reader)?;
        let state_id = read_var_i32(reader)?;
        let slot_num = read_i16(reader)?;
        let button_num = read_i8(reader)?;
        let container_input = ContainerInput::from_wire_id(read_var_i32(reader)?);
        let changed_len = read_limited_len(reader, Self::MAX_CHANGED_SLOTS, "changed slot count")?;
        let mut changed_slots = BTreeMap::new();
        for _ in 0..changed_len {
            changed_slots.insert(read_i16(reader)? as i32, HashedStack::read(reader)?);
        }
        Ok(Self {
            container_id,
            state_id,
            slot_num,
            button_num,
            container_input,
            changed_slots,
            carried_item: HashedStack::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.changed_slots.len() > Self::MAX_CHANGED_SLOTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many changed slots",
            ));
        }
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.state_id)?;
        write_i16(writer, self.slot_num)?;
        write_i8(writer, self.button_num)?;
        write_var_i32(writer, self.container_input.to_wire_id())?;
        write_var_i32(writer, self.changed_slots.len() as i32)?;
        for (slot, stack) in &self.changed_slots {
            write_i16(writer, *slot as i16)?;
            stack.write(writer)?;
        }
        self.carried_item.write(writer)
    }
}

impl ServerboundSetCreativeModeSlotPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot_num: read_i16(reader)?,
            item_stack: RawItemStack::read_optional_untrusted(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i16(writer, self.slot_num)?;
        self.item_stack.write_optional_untrusted(writer)
    }
}

impl ServerboundCommandSuggestionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_var_i32(reader)?,
            command: read_string(reader, 32500)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_string(writer, &self.command, 32500)
    }
}

impl ServerboundEditBookPacket {
    pub const MAX_PAGES: usize = 100;
    pub const MAX_PAGE_CHARS: usize = 1024;
    pub const MAX_TITLE_CHARS: usize = 32;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let slot = read_var_i32(reader)?;
        let page_count = read_var_i32(reader)?;
        if page_count < 0 || page_count as usize > Self::MAX_PAGES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid edit book page count",
            ));
        }
        let mut pages = Vec::with_capacity(page_count as usize);
        for _ in 0..page_count {
            pages.push(read_string(reader, Self::MAX_PAGE_CHARS)?);
        }
        let title = if read_bool(reader)? {
            Some(read_string(reader, Self::MAX_TITLE_CHARS)?)
        } else {
            None
        };
        Ok(Self { slot, pages, title })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.pages.len() > Self::MAX_PAGES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many edit book pages",
            ));
        }
        write_var_i32(writer, self.slot)?;
        write_var_i32(writer, self.pages.len() as i32)?;
        for page in &self.pages {
            write_string(writer, page, Self::MAX_PAGE_CHARS)?;
        }
        write_bool(writer, self.title.is_some())?;
        if let Some(title) = &self.title {
            write_string(writer, title, Self::MAX_TITLE_CHARS)?;
        }
        Ok(())
    }
}

impl ServerboundInteractionHand {
    fn from_id(id: i32) -> Self {
        match id {
            1 => Self::OffHand,
            _ => Self::MainHand,
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
        }
    }
}

impl ServerboundInteractPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            hand: ServerboundInteractionHand::from_id(read_var_i32(reader)?),
            location: read_lp_vec3(reader)?,
            using_secondary_action: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.hand.to_id())?;
        write_lp_vec3(writer, self.location)?;
        write_bool(writer, self.using_secondary_action)
    }
}

fn read_lp_vec3<R: Read>(reader: &mut R) -> io::Result<Vec3> {
    let lowest = read_u8(reader)?;
    if lowest == 0 {
        return Ok(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
    }

    let middle = read_u8(reader)?;
    let mut highest_bytes = [0u8; 4];
    reader.read_exact(&mut highest_bytes)?;
    let highest = u32::from_be_bytes(highest_bytes) as u64;
    let buffer = (highest << 16) | ((middle as u64) << 8) | lowest as u64;
    let mut scale = (lowest & 3) as u64;
    if lowest & 4 == 4 {
        scale |= (read_var_i32(reader)? as u32 as u64) << 2;
    }
    let scale = scale as f64;

    Ok(Vec3 {
        x: unpack_lp_vec3_component(buffer >> 3) * scale,
        y: unpack_lp_vec3_component(buffer >> 18) * scale,
        z: unpack_lp_vec3_component(buffer >> 33) * scale,
    })
}

fn write_lp_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    const ABS_MAX_VALUE: f64 = 1.7179869183E10;
    const ABS_MIN_VALUE: f64 = 3.051944088384301E-5;

    let x = sanitize_lp_vec3_component(value.x, ABS_MAX_VALUE);
    let y = sanitize_lp_vec3_component(value.y, ABS_MAX_VALUE);
    let z = sanitize_lp_vec3_component(value.z, ABS_MAX_VALUE);
    let chessboard_length = x.abs().max(y.abs()).max(z.abs());
    if chessboard_length < ABS_MIN_VALUE {
        return writer.write_all(&[0]);
    }

    let scale = chessboard_length.ceil() as u64;
    let is_partial = (scale & 3) != scale;
    let markers = if is_partial { (scale & 3) | 4 } else { scale };
    let buffer = markers
        | (pack_lp_vec3_component(x / scale as f64) << 3)
        | (pack_lp_vec3_component(y / scale as f64) << 18)
        | (pack_lp_vec3_component(z / scale as f64) << 33);
    writer.write_all(&[(buffer & 0xff) as u8, ((buffer >> 8) & 0xff) as u8])?;
    writer.write_all(&((buffer >> 16) as u32).to_be_bytes())?;
    if is_partial {
        write_var_i32(writer, (scale >> 2) as i32)?;
    }
    Ok(())
}

fn sanitize_lp_vec3_component(value: f64, abs_max: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-abs_max, abs_max)
    }
}

fn pack_lp_vec3_component(value: f64) -> u64 {
    ((value * 0.5 + 0.5) * 32766.0).round() as u64
}

fn unpack_lp_vec3_component(value: u64) -> f64 {
    (value & 32767).min(32766) as f64 * 2.0 / 32766.0 - 1.0
}

impl ServerboundPickItemFromBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            include_data: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_bool(writer, self.include_data)
    }
}

impl ServerboundPickItemFromEntityPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            include_data: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_bool(writer, self.include_data)
    }
}

impl RecipeBookType {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Crafting),
            1 => Ok(Self::Furnace),
            2 => Ok(Self::BlastFurnace),
            3 => Ok(Self::Smoker),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid recipe book type {id}"),
            )),
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Crafting => 0,
            Self::Furnace => 1,
            Self::BlastFurnace => 2,
            Self::Smoker => 3,
        }
    }
}

impl ServerboundRecipeBookChangeSettingsPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            book_type: RecipeBookType::from_id(read_var_i32(reader)?)?,
            is_open: read_bool(reader)?,
            is_filtering: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.book_type.to_id())?;
        write_bool(writer, self.is_open)?;
        write_bool(writer, self.is_filtering)
    }
}

impl ServerboundRecipeBookSeenRecipePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            recipe_index: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.recipe_index)
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

impl ClientboundChangeDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            difficulty: GameDifficulty::from_wire_index(read_var_i32(reader)?)?,
            locked: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.difficulty.to_wire_index())?;
        write_bool(writer, self.locked)
    }
}

impl ClientboundSetChunkCacheCenterPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            x: read_var_i32(reader)?,
            z: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.x)?;
        write_var_i32(writer, self.z)
    }
}

impl ClientboundSetChunkCacheRadiusPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            radius: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.radius)
    }
}

impl ClientboundSetDefaultSpawnPositionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let dimension = read_identifier(reader)?;
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            respawn_data: ClientboundSetDefaultSpawnPositionData {
                dimension,
                x,
                y,
                z,
                yaw: read_f32(reader)?,
                pitch: read_f32(reader)?,
            },
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.respawn_data.dimension)?;
        write_block_position(
            writer,
            self.respawn_data.x,
            self.respawn_data.y,
            self.respawn_data.z,
        )?;
        writer.write_all(&self.respawn_data.yaw.to_be_bytes())?;
        writer.write_all(&self.respawn_data.pitch.to_be_bytes())
    }
}

impl ClientboundSetExperiencePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            experience_progress: read_f32(reader)?,
            experience_level: read_var_i32(reader)?,
            total_experience: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.experience_progress.to_be_bytes())?;
        write_var_i32(writer, self.experience_level)?;
        write_var_i32(writer, self.total_experience)
    }
}

impl ClientboundSetHealthPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            health: read_f32(reader)?,
            food: read_var_i32(reader)?,
            saturation: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.health.to_be_bytes())?;
        write_var_i32(writer, self.food)?;
        writer.write_all(&self.saturation.to_be_bytes())
    }
}

impl ClockNetworkState {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            total_ticks: read_var_i64(reader)?,
            partial_tick: read_f32(reader)?,
            rate: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i64(writer, self.total_ticks)?;
        writer.write_all(&self.partial_tick.to_be_bytes())?;
        writer.write_all(&self.rate.to_be_bytes())
    }
}

impl ClientboundSetTimePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let game_time = read_var_i64(reader)?;
        let clock_updates_len = read_var_i32(reader)?;
        if clock_updates_len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid clock update map length",
            ));
        }

        let mut clock_updates = BTreeMap::new();
        for _ in 0..clock_updates_len {
            let key = read_identifier(reader)?;
            let state = ClockNetworkState::read(reader)?;
            clock_updates.insert(key, state);
        }

        Ok(Self {
            game_time,
            clock_updates,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i64(writer, self.game_time)?;
        write_var_i32(writer, self.clock_updates.len() as i32)?;
        for (clock, state) in &self.clock_updates {
            write_identifier(writer, clock)?;
            state.write(writer)?;
        }
        Ok(())
    }
}

impl ClientboundGameEventType {
    fn from_id(id: u8) -> Self {
        match id {
            0 => Self::NoRespawnBlockAvailable,
            1 => Self::StartRaining,
            2 => Self::StopRaining,
            3 => Self::ChangeGameMode,
            4 => Self::WinGame,
            5 => Self::DemoEvent,
            6 => Self::PlayArrowHitSound,
            7 => Self::RainLevelChange,
            8 => Self::ThunderLevelChange,
            9 => Self::PufferFishSting,
            10 => Self::GuardianElderEffect,
            11 => Self::ImmediateRespawn,
            12 => Self::LimitedCrafting,
            13 => Self::LevelChunksLoadStart,
            _ => Self::Unknown(id),
        }
    }

    fn to_id(self) -> u8 {
        match self {
            Self::NoRespawnBlockAvailable => 0,
            Self::StartRaining => 1,
            Self::StopRaining => 2,
            Self::ChangeGameMode => 3,
            Self::WinGame => 4,
            Self::DemoEvent => 5,
            Self::PlayArrowHitSound => 6,
            Self::RainLevelChange => 7,
            Self::ThunderLevelChange => 8,
            Self::PufferFishSting => 9,
            Self::GuardianElderEffect => 10,
            Self::ImmediateRespawn => 11,
            Self::LimitedCrafting => 12,
            Self::LevelChunksLoadStart => 13,
            Self::Unknown(value) => value,
        }
    }
}

impl ClientboundGameEventPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = [0u8; 1];
        reader.read_exact(&mut value)?;
        Ok(Self {
            event: ClientboundGameEventType::from_id(value[0]),
            param: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.event.to_id()])?;
        writer.write_all(&self.param.to_be_bytes())
    }
}

impl ClientboundSetSimulationDistancePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            simulation_distance: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.simulation_distance)
    }
}

impl ClientboundTickingStatePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            tick_rate: read_f32(reader)?,
            is_frozen: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.tick_rate.to_be_bytes())?;
        write_bool(writer, self.is_frozen)
    }
}

impl ClientboundTickingStepPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            tick_steps: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.tick_steps)
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

impl ClientboundBlockDestructionPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.progress])
    }
}

impl ClientboundBlockEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.action, self.param])?;
        write_var_i32(writer, self.block_id)
    }
}

impl ClientboundBlockUpdatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.block_state_id)
    }
}

impl ClientboundBlockEntityDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.block_entity_type_id)?;
        write_network_compound_tag(writer, &self.tag)
    }
}

impl ClientboundLevelEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.event_type)?;
        write_block_position(writer, self.x, self.y, self.z)?;
        write_i32(writer, self.data)?;
        write_bool(writer, self.global_event)
    }
}

impl ClientboundPlayerInfoRemovePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.profile_ids.len() as i32)?;
        for id in &self.profile_ids {
            write_uuid(writer, *id)?;
        }
        Ok(())
    }
}

impl ClientboundContainerClosePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)
    }
}

impl ClientboundContainerSetDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_i16(writer, self.id)?;
        write_i16(writer, self.value)
    }
}

impl ClientboundMountScreenOpenPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.inventory_columns)?;
        write_i32(writer, self.entity_id)
    }
}

impl ClientboundCooldownPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.cooldown_group)?;
        write_var_i32(writer, self.duration)
    }
}

impl ClientboundPlayerAbilitiesPacket {
    pub fn flags(&self) -> u8 {
        (if self.invulnerable { 1 } else { 0 })
            | (if self.flying { 2 } else { 0 })
            | (if self.can_fly { 4 } else { 0 })
            | (if self.instant_build { 8 } else { 0 })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.flags()])?;
        write_f32(writer, self.flying_speed)?;
        write_f32(writer, self.walking_speed)
    }
}

impl ClientboundAwardStatsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.stats.len() as i32)?;
        for stat in &self.stats {
            write_var_i32(writer, stat.stat_type_id)?;
            write_var_i32(writer, stat.stat_value_id)?;
            write_var_i32(writer, stat.value)?;
        }
        Ok(())
    }
}

impl ClientboundPingPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            id: i32::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
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

    pub fn write_pos<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::Pos)
    }

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::PosRot)
    }

    pub fn write_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::Rot)
    }

    pub fn write_status_only<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write_shape(writer, MoveShape::StatusOnly)
    }
}

impl ServerboundMoveVehiclePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            position: Vec3 {
                x: read_f64(reader)?,
                y: read_f64(reader)?,
                z: read_f64(reader)?,
            },
            y_rot: read_f32(reader)?,
            x_rot: read_f32(reader)?,
            on_ground: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.position.x.to_be_bytes())?;
        writer.write_all(&self.position.y.to_be_bytes())?;
        writer.write_all(&self.position.z.to_be_bytes())?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)?;
        write_bool(writer, self.on_ground)
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

fn read_i8<R: Read>(reader: &mut R) -> io::Result<i8> {
    Ok(read_u8(reader)? as i8)
}

fn write_i8<W: Write>(writer: &mut W, value: i8) -> io::Result<()> {
    writer.write_all(&[value as u8])
}

fn read_clamped_i8<R: Read>(reader: &mut R, min: i8, max: i8) -> io::Result<i8> {
    Ok((read_u8(reader)? as i8).clamp(min, max))
}

fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn expect_empty_payload<R: Read>(reader: &mut R) -> io::Result<()> {
    let mut byte = [0u8; 1];
    match reader.read(&mut byte)? {
        0 => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected empty payload",
        )),
    }
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn write_f32<W: Write>(writer: &mut W, value: f32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

fn write_i16<W: Write>(writer: &mut W, value: i16) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

fn write_i32<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

fn write_i64<W: Write>(writer: &mut W, value: i64) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn write_network_compound_tag<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    match tag {
        Tag::Compound(fields) if fields.is_empty() => writer.write_all(&[0]),
        Tag::Compound(_) => write_network_tag(writer, tag),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "network compound tag payload must be a compound",
        )),
    }
}

fn write_network_tag<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        tag.write_payload(writer)?;
    }
    Ok(())
}

fn read_length_prefixed_bytes<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "length-prefixed payload too large",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_length_prefixed_bytes<W: Write>(
    writer: &mut W,
    payload: &[u8],
    max_size: usize,
) -> io::Result<()> {
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "length-prefixed payload too large",
        ));
    }
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

fn read_limited_collection<R, T, F>(
    reader: &mut R,
    max_len: usize,
    mut read: F,
) -> io::Result<Vec<T>>
where
    R: Read,
    F: FnMut(&mut R) -> io::Result<T>,
{
    let len = read_var_i32(reader)?;
    if len < 0 || len as usize > max_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "collection length exceeds packet limit",
        ));
    }
    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        values.push(read(reader)?);
    }
    Ok(values)
}

fn read_limited_len<R: Read>(
    reader: &mut R,
    max_len: usize,
    description: &'static str,
) -> io::Result<usize> {
    let len = read_var_i32(reader)?;
    if len < 0 || len as usize > max_len {
        return Err(io::Error::new(io::ErrorKind::InvalidData, description));
    }
    Ok(len as usize)
}

fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

fn write_f64<W: Write>(writer: &mut W, value: f64) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}

fn write_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    write_f64(writer, value.x)?;
    write_f64(writer, value.y)?;
    write_f64(writer, value.z)
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
                stats: vec![AwardedStat {
                    stat_type_id: 8,
                    stat_value_id: 23,
                    value: 3,
                }],
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
                suggestions: vec![CommandSuggestionEntry {
                    text: "help".to_string(),
                    tooltip: None,
                }],
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
        let mut move_pos = Vec::new();
        ClientboundMoveEntityPacket::pos(300, [1, -2, 3], true)
            .write_pos(&mut move_pos)
            .unwrap();
        assert_eq!(
            move_pos,
            vec![0xac, 0x02, 0x00, 0x01, 0xff, 0xfe, 0x00, 0x03, 0x01]
        );

        let mut move_pos_rot = Vec::new();
        ClientboundMoveEntityPacket::pos_rot(300, [1, 2, 3], 90.0, 45.0, false)
            .write_pos_rot(&mut move_pos_rot)
            .unwrap();
        assert_eq!(
            move_pos_rot,
            vec![0xac, 0x02, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x40, 0x20, 0x00]
        );

        let mut move_rot = Vec::new();
        ClientboundMoveEntityPacket::rot(300, 180.0, 45.0, true)
            .write_rot(&mut move_rot)
            .unwrap();
        assert_eq!(move_rot, vec![0xac, 0x02, 0x80, 0x20, 0x01]);
        assert_eq!(ClientboundRotateHeadPacket::new(7, 180.0).y_head_rot, 128);
        let motion = ClientboundSetEntityMotionPacket::new(
            7,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        let mut motion_payload = Vec::new();
        motion.write(&mut motion_payload).unwrap();
        assert_eq!(motion_payload, vec![7, 0]);
        let move_vehicle = ClientboundMoveVehiclePacket {
            position: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            y_rot: 90.0,
            x_rot: 45.0,
        };
        let mut move_vehicle_payload = Vec::new();
        move_vehicle.write(&mut move_vehicle_payload).unwrap();
        assert_eq!(&move_vehicle_payload[..8], &1.0_f64.to_be_bytes());
        assert_eq!(&move_vehicle_payload[8..16], &2.0_f64.to_be_bytes());
        assert_eq!(&move_vehicle_payload[16..24], &3.0_f64.to_be_bytes());
        assert_eq!(&move_vehicle_payload[24..28], &90.0_f32.to_be_bytes());
        assert_eq!(&move_vehicle_payload[28..32], &45.0_f32.to_be_bytes());
        let mut rotate_head_payload = Vec::new();
        ClientboundRotateHeadPacket::new(7, 180.0)
            .write(&mut rotate_head_payload)
            .unwrap();
        assert_eq!(rotate_head_payload, vec![7, 128]);
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
        let mut passengers_payload = Vec::new();
        ClientboundSetPassengersPacket {
            vehicle: 7,
            passengers: vec![8, 9],
        }
        .write(&mut passengers_payload)
        .unwrap();
        assert_eq!(passengers_payload, vec![7, 2, 8, 9]);
        assert_eq!(
            ClientboundAnimatePacket {
                id: 7,
                action: EntityAnimation::SwingOffHand,
            }
            .action as i32,
            3
        );
        let mut animate_payload = Vec::new();
        ClientboundAnimatePacket {
            id: 7,
            action: EntityAnimation::SwingOffHand,
        }
        .write(&mut animate_payload)
        .unwrap();
        assert_eq!(animate_payload, vec![7, 3]);
        let mut entity_event_payload = Vec::new();
        ClientboundEntityEventPacket {
            entity_id: 7,
            event_id: 3,
        }
        .write(&mut entity_event_payload)
        .unwrap();
        assert_eq!(entity_event_payload, vec![0, 0, 0, 7, 3]);
        assert_eq!(
            ClientboundRemoveEntitiesPacket {
                entity_ids: vec![7, 8]
            }
            .entity_ids,
            vec![7, 8]
        );
        let mut remove_payload = Vec::new();
        ClientboundRemoveEntitiesPacket {
            entity_ids: vec![7, 8],
        }
        .write(&mut remove_payload)
        .unwrap();
        assert_eq!(remove_payload, vec![2, 7, 8]);

        let border = ClientboundInitializeBorderPacket {
            new_center_x: 1.0,
            new_center_z: 2.0,
            old_size: 100.0,
            new_size: 200.0,
            lerp_time: 300,
            new_absolute_max_size: 400,
            warning_blocks: 5,
            warning_time: 6,
        };
        let mut border_payload = Vec::new();
        border.write(&mut border_payload).unwrap();
        assert_eq!(&border_payload[..8], &1.0_f64.to_be_bytes());
        assert_eq!(&border_payload[8..16], &2.0_f64.to_be_bytes());
        assert_eq!(&border_payload[16..24], &100.0_f64.to_be_bytes());
        assert_eq!(&border_payload[24..32], &200.0_f64.to_be_bytes());
        assert_eq!(&border_payload[32..], &[0xac, 0x02, 0x90, 0x03, 5, 6]);

        let mut border_center = Vec::new();
        ClientboundSetBorderCenterPacket {
            new_center_x: 1.0,
            new_center_z: 2.0,
        }
        .write(&mut border_center)
        .unwrap();
        assert_eq!(&border_center[..8], &1.0_f64.to_be_bytes());
        assert_eq!(&border_center[8..], &2.0_f64.to_be_bytes());

        let mut border_lerp = Vec::new();
        ClientboundSetBorderLerpSizePacket {
            old_size: 100.0,
            new_size: 200.0,
            lerp_time: 300,
        }
        .write(&mut border_lerp)
        .unwrap();
        assert_eq!(&border_lerp[..8], &100.0_f64.to_be_bytes());
        assert_eq!(&border_lerp[8..16], &200.0_f64.to_be_bytes());
        assert_eq!(&border_lerp[16..], &[0xac, 0x02]);

        let mut border_size = Vec::new();
        ClientboundSetBorderSizePacket { size: 200.0 }
            .write(&mut border_size)
            .unwrap();
        assert_eq!(border_size, 200.0_f64.to_be_bytes());

        let mut warning_delay = Vec::new();
        ClientboundSetBorderWarningDelayPacket { warning_delay: 6 }
            .write(&mut warning_delay)
            .unwrap();
        assert_eq!(warning_delay, vec![6]);

        let mut warning_distance = Vec::new();
        ClientboundSetBorderWarningDistancePacket { warning_blocks: 5 }
            .write(&mut warning_distance)
            .unwrap();
        assert_eq!(warning_distance, vec![5]);

        let mut clear_titles = Vec::new();
        ClientboundClearTitlesPacket { reset_times: true }
            .write(&mut clear_titles)
            .unwrap();
        assert_eq!(clear_titles, vec![1]);

        let mut title_times = Vec::new();
        ClientboundSetTitlesAnimationPacket {
            fade_in: 10,
            stay: 70,
            fade_out: 20,
        }
        .write(&mut title_times)
        .unwrap();
        assert_eq!(
            title_times,
            [
                10_i32.to_be_bytes(),
                70_i32.to_be_bytes(),
                20_i32.to_be_bytes()
            ]
            .concat()
        );

        let mut clientbound_close = Vec::new();
        ClientboundContainerClosePacket { container_id: 128 }
            .write(&mut clientbound_close)
            .unwrap();
        assert_eq!(clientbound_close, vec![0x80, 0x01]);

        let mut set_data = Vec::new();
        ClientboundContainerSetDataPacket {
            container_id: 2,
            id: -3,
            value: 400,
        }
        .write(&mut set_data)
        .unwrap();
        assert_eq!(set_data, vec![2, 0xff, 0xfd, 0x01, 0x90]);

        let mut mount_screen = Vec::new();
        ClientboundMountScreenOpenPacket {
            container_id: 2,
            inventory_columns: 5,
            entity_id: 300,
        }
        .write(&mut mount_screen)
        .unwrap();
        assert_eq!(
            mount_screen,
            [vec![2, 5], 300_i32.to_be_bytes().to_vec()].concat()
        );

        let mut cooldown = Vec::new();
        ClientboundCooldownPacket {
            cooldown_group: Identifier::parse("minecraft:ender_pearl").unwrap(),
            duration: 20,
        }
        .write(&mut cooldown)
        .unwrap();
        assert_eq!(
            cooldown,
            [vec![21], b"minecraft:ender_pearl".to_vec(), vec![20]].concat()
        );

        let abilities = ClientboundPlayerAbilitiesPacket {
            invulnerable: true,
            flying: false,
            can_fly: true,
            instant_build: true,
            flying_speed: 0.05,
            walking_speed: 0.1,
        };
        let mut abilities_payload = Vec::new();
        abilities.write(&mut abilities_payload).unwrap();
        assert_eq!(abilities_payload[0], 0b1101);
        assert_eq!(&abilities_payload[1..5], &0.05_f32.to_be_bytes());
        assert_eq!(&abilities_payload[5..9], &0.1_f32.to_be_bytes());

        let mut block_destruction = Vec::new();
        ClientboundBlockDestructionPacket {
            id: 99,
            x: -12,
            y: 64,
            z: 34,
            progress: 9,
        }
        .write(&mut block_destruction)
        .unwrap();
        assert_eq!(block_destruction[0], 99);
        assert_eq!(block_destruction.len(), 10);
        assert_eq!(*block_destruction.last().unwrap(), 9);

        let mut block_event = Vec::new();
        ClientboundBlockEventPacket {
            x: -12,
            y: 64,
            z: 34,
            action: 1,
            param: 2,
            block_id: 300,
        }
        .write(&mut block_event)
        .unwrap();
        assert_eq!(block_event.len(), 12);
        assert_eq!(&block_event[8..10], &[1, 2]);
        assert_eq!(&block_event[10..], &[0xac, 0x02]);

        let mut block_update = Vec::new();
        ClientboundBlockUpdatePacket {
            x: -12,
            y: 64,
            z: 34,
            block_state_id: 300,
        }
        .write(&mut block_update)
        .unwrap();
        assert_eq!(block_update.len(), 10);
        assert_eq!(&block_update[8..], &[0xac, 0x02]);

        let mut level_event = Vec::new();
        ClientboundLevelEventPacket {
            event_type: 2001,
            x: -12,
            y: 64,
            z: 34,
            data: 300,
            global_event: true,
        }
        .write(&mut level_event)
        .unwrap();
        assert_eq!(&level_event[..4], &2001_i32.to_be_bytes());
        assert_eq!(&level_event[12..16], &300_i32.to_be_bytes());
        assert_eq!(level_event[16], 1);

        let mut player_info_remove = Vec::new();
        ClientboundPlayerInfoRemovePacket {
            profile_ids: vec![Uuid([1; 16]), Uuid([2; 16])],
        }
        .write(&mut player_info_remove)
        .unwrap();
        assert_eq!(player_info_remove[0], 2);
        assert_eq!(&player_info_remove[1..17], &[1; 16]);
        assert_eq!(&player_info_remove[17..33], &[2; 16]);

        let mut recipe_remove = Vec::new();
        ClientboundRecipeBookRemovePacket {
            recipe_display_ids: vec![1, 128],
        }
        .write(&mut recipe_remove)
        .unwrap();
        assert_eq!(recipe_remove, vec![2, 1, 0x80, 0x01]);

        let mut recipe_settings = Vec::new();
        ClientboundRecipeBookSettingsPacket {
            crafting: RecipeBookTypeSettings {
                open: true,
                filtering: false,
            },
            furnace: RecipeBookTypeSettings {
                open: false,
                filtering: true,
            },
            blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
            smoker: RecipeBookTypeSettings {
                open: true,
                filtering: true,
            },
        }
        .write(&mut recipe_settings)
        .unwrap();
        assert_eq!(recipe_settings, vec![1, 0, 0, 1, 0, 0, 1, 1]);

        let mut command_suggestions = Vec::new();
        ClientboundCommandSuggestionsPacket {
            transaction_id: 4,
            start: 1,
            length: 2,
            suggestions: vec![
                CommandSuggestionEntry {
                    text: "help".to_string(),
                    tooltip: None,
                },
                CommandSuggestionEntry {
                    text: "hello".to_string(),
                    tooltip: Some(Tag::Compound(vec![(
                        "text".to_string(),
                        Tag::String("tooltip".to_string()),
                    )])),
                },
            ],
        }
        .write(&mut command_suggestions)
        .unwrap();
        assert_eq!(&command_suggestions[..6], &[4, 1, 2, 2, 4, b'h']);
        assert!(command_suggestions.ends_with(&[0]));
        assert!(command_suggestions
            .windows(4)
            .any(|window| window == [1, 10, 8, 0]));

        let mut debug_sample = Vec::new();
        ClientboundDebugSamplePacket {
            sample: vec![10, -20],
            sample_type: RemoteDebugSampleType::TickTime,
        }
        .write(&mut debug_sample)
        .unwrap();
        assert_eq!(debug_sample[0], 2);
        assert_eq!(&debug_sample[1..9], &10_i64.to_be_bytes());
        assert_eq!(&debug_sample[9..17], &(-20_i64).to_be_bytes());
        assert_eq!(debug_sample[17], 0);

        let mut start_configuration = Vec::new();
        ClientboundStartConfigurationPacket
            .write(&mut start_configuration)
            .unwrap();
        assert!(start_configuration.is_empty());

        let mut remove_effect = Vec::new();
        ClientboundRemoveMobEffectPacket {
            entity_id: 129,
            effect_id: 5,
        }
        .write(&mut remove_effect)
        .unwrap();
        assert_eq!(remove_effect, vec![0x81, 0x01, 5]);

        let mut update_effect = Vec::new();
        ClientboundUpdateMobEffectPacket {
            entity_id: 129,
            effect_id: 5,
            amplifier: 2,
            duration_ticks: 600,
            flags: MobEffectFlags::from_parts(true, false, true, true),
        }
        .write(&mut update_effect)
        .unwrap();
        assert_eq!(update_effect, vec![0x81, 0x01, 5, 2, 0xd8, 0x04, 0x0d]);

        let mut award_stats = Vec::new();
        ClientboundAwardStatsPacket {
            stats: vec![AwardedStat {
                stat_type_id: 8,
                stat_value_id: 23,
                value: 300,
            }],
        }
        .write(&mut award_stats)
        .unwrap();
        assert_eq!(award_stats, vec![1, 8, 23, 0xac, 0x02]);

        let mut section_blocks = Vec::new();
        ClientboundSectionBlocksUpdatePacket {
            section_pos: SectionPos { x: 1, y: -2, z: 3 },
            updates: vec![SectionBlockUpdate {
                packed_pos: 0x0abc,
                block_state_id: 118,
            }],
        }
        .write(&mut section_blocks)
        .unwrap();
        assert_eq!(
            &section_blocks[..8],
            &0x0000_0400_003f_fffe_i64.to_be_bytes()
        );
        assert_eq!(&section_blocks[8..], &[1, 0xbc, 0xd5, 0x1d]);

        let mut block_entity = Vec::new();
        ClientboundBlockEntityDataPacket {
            x: 1,
            y: 64,
            z: -2,
            block_entity_type_id: 1,
            tag: Tag::Compound(Vec::new()),
        }
        .write(&mut block_entity)
        .unwrap();
        assert_eq!(
            block_entity,
            [
                pack_block_position(1, 64, -2).to_be_bytes().to_vec(),
                vec![1, 0],
            ]
            .concat()
        );

        let mut named_like_network_nbt = Vec::new();
        ClientboundBlockEntityDataPacket {
            x: 0,
            y: 0,
            z: 0,
            block_entity_type_id: 1,
            tag: Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:chest".to_string()),
            )]),
        }
        .write(&mut named_like_network_nbt)
        .unwrap();
        assert_eq!(named_like_network_nbt[9], 10);
        assert_eq!(
            &named_like_network_nbt[10..13],
            &[8, 0, 2],
            "network NBT uses writeAnyTag and must not include a root name"
        );

        let title_tag = Tag::Compound(vec![("text".to_string(), Tag::String("Title".to_string()))]);
        let mut title = Vec::new();
        ClientboundSetTitleTextPacket {
            text: title_tag.clone(),
        }
        .write(&mut title)
        .unwrap();
        assert_eq!(&title[..4], &[10, 8, 0, 4]);

        let mut subtitle = Vec::new();
        ClientboundSetSubtitleTextPacket {
            text: title_tag.clone(),
        }
        .write(&mut subtitle)
        .unwrap();
        assert_eq!(subtitle, title);

        let mut action_bar = Vec::new();
        ClientboundSetActionBarTextPacket {
            text: title_tag.clone(),
        }
        .write(&mut action_bar)
        .unwrap();
        assert_eq!(action_bar, title);

        let mut system_chat = Vec::new();
        ClientboundSystemChatPacket {
            content: title_tag.clone(),
            overlay: true,
        }
        .write(&mut system_chat)
        .unwrap();
        assert!(system_chat.starts_with(&title));
        assert_eq!(*system_chat.last().unwrap(), 1);

        let footer_tag = Tag::Compound(vec![(
            "text".to_string(),
            Tag::String("Footer".to_string()),
        )]);
        let mut tab_list = Vec::new();
        ClientboundTabListPacket {
            header: title_tag,
            footer: footer_tag,
        }
        .write(&mut tab_list)
        .unwrap();
        assert!(tab_list.starts_with(&title));
        assert_eq!(tab_list.iter().filter(|byte| **byte == 10).count(), 2);

        let mut reset_score = Vec::new();
        ClientboundResetScorePacket {
            owner: "Alex".to_string(),
            objective_name: Some("kills".to_string()),
        }
        .write(&mut reset_score)
        .unwrap();
        assert_eq!(
            reset_score,
            [vec![4], b"Alex".to_vec(), vec![1, 5], b"kills".to_vec()].concat()
        );

        let mut display_objective = Vec::new();
        ClientboundSetDisplayObjectivePacket {
            slot: 1,
            objective_name: "sidebar".to_string(),
        }
        .write(&mut display_objective)
        .unwrap();
        assert_eq!(
            display_objective,
            [vec![1, 7], b"sidebar".to_vec()].concat()
        );

        let mut pack_pop = Vec::new();
        ClientboundResourcePackPopPacket {
            id: Some(Uuid([3; 16])),
        }
        .write(&mut pack_pop)
        .unwrap();
        assert_eq!(pack_pop[0], 1);
        assert_eq!(&pack_pop[1..], &[3; 16]);

        let teleport = ClientboundTeleportEntityPacket {
            id: 7,
            position: Vec3 {
                x: 1.25,
                y: 64.0,
                z: -2.5,
            },
            movement: Vec3 {
                x: 0.1,
                y: -0.2,
                z: 0.3,
            },
            y_rot: 90.0,
            x_rot: 30.0,
            relative_flags: 0b1_0010_0011,
            on_ground: true,
        };
        let mut teleport_payload = Vec::new();
        teleport.write(&mut teleport_payload).unwrap();
        assert_eq!(teleport_payload[0], 7);
        assert_eq!(&teleport_payload[1..9], &1.25_f64.to_be_bytes());
        assert_eq!(&teleport_payload[25..33], &0.1_f64.to_be_bytes());
        assert_eq!(&teleport_payload[49..53], &90.0_f32.to_be_bytes());
        assert_eq!(&teleport_payload[53..57], &30.0_f32.to_be_bytes());
        assert_eq!(&teleport_payload[57..61], &0b1_0010_0011_i32.to_be_bytes());
        assert_eq!(teleport_payload[61], 1);

        let mut position_sync = Vec::new();
        ClientboundEntityPositionSyncPacket {
            id: 8,
            position: teleport.position,
            movement: teleport.movement,
            y_rot: teleport.y_rot,
            x_rot: teleport.x_rot,
            on_ground: false,
        }
        .write(&mut position_sync)
        .unwrap();
        assert_eq!(position_sync[0], 8);
        assert_eq!(position_sync.len(), 58);
        assert_eq!(*position_sync.last().unwrap(), 0);

        let mut player_position = Vec::new();
        ClientboundPlayerPositionPacket {
            id: 9,
            position: teleport.position,
            movement: teleport.movement,
            y_rot: teleport.y_rot,
            x_rot: teleport.x_rot,
            relative_flags: 0b1_0010_0011,
        }
        .write(&mut player_position)
        .unwrap();
        assert_eq!(player_position[0], 9);
        assert_eq!(player_position.len(), 61);
        assert_eq!(&player_position[57..61], &0b1_0010_0011_i32.to_be_bytes());

        let mut look_at = Vec::new();
        ClientboundPlayerLookAtPacket {
            from_anchor: EntityAnchor::Eyes,
            x: 10.0,
            y: 64.5,
            z: -7.25,
            target_entity: Some((33, EntityAnchor::Feet)),
        }
        .write(&mut look_at)
        .unwrap();
        assert_eq!(look_at[0], 1);
        assert_eq!(&look_at[1..9], &10.0_f64.to_be_bytes());
        assert_eq!(&look_at[9..17], &64.5_f64.to_be_bytes());
        assert_eq!(&look_at[17..25], &(-7.25_f64).to_be_bytes());
        assert_eq!(&look_at[25..], &[1, 33, 0]);

        let mut stop_sound = Vec::new();
        ClientboundStopSoundPacket {
            source: Some(SoundSource::Blocks),
            name: Some(Identifier::parse("minecraft:block.note_block.harp").unwrap()),
        }
        .write(&mut stop_sound)
        .unwrap();
        assert_eq!(
            stop_sound,
            [vec![3, 4, 31], b"minecraft:block.note_block.harp".to_vec()].concat()
        );

        let mut cached_delete_chat = Vec::new();
        ClientboundDeleteChatPacket {
            message_signature: PackedMessageSignature::CacheId(7),
        }
        .write(&mut cached_delete_chat)
        .unwrap();
        assert_eq!(cached_delete_chat, vec![8]);

        let mut full_delete_chat = Vec::new();
        ClientboundDeleteChatPacket {
            message_signature: PackedMessageSignature::Full(MessageSignature([9; 256])),
        }
        .write(&mut full_delete_chat)
        .unwrap();
        assert_eq!(full_delete_chat[0], 0);
        assert_eq!(&full_delete_chat[1..], &[9; 256]);
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
        assert!(
            payload.windows(3).any(|bytes| bytes == [0x80, 0x10, 0xff]),
            "sky light data layers use ByteBufCodecs.byteArray(2048): VarInt length then bytes"
        );
        assert!(
            payload.windows(3).any(|bytes| bytes == [0x80, 0x10, 0x01]),
            "block light data layers use ByteBufCodecs.byteArray(2048): VarInt length then bytes"
        );
    }

    #[test]
    fn chunk_section_serialization_matches_vanilla_section_field_order() {
        let section = NetworkChunkSection {
            non_empty_block_count: 2,
            fluid_count: 0,
            block_states: NetworkPalettedContainer::single(5),
            biomes: NetworkPalettedContainer::single(7),
        };
        let mut bytes = Vec::new();
        section.write(&mut bytes).unwrap();

        assert_eq!(&bytes[0..2], &2_i16.to_be_bytes());
        assert_eq!(&bytes[2..4], &0_i16.to_be_bytes());
        assert_eq!(bytes[4], 0);
        assert_eq!(bytes[5], 5);
        assert_eq!(bytes[6], 0);
        assert_eq!(bytes[7], 7);
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn generated_terrain_block_state_names_use_current_protocol_state_ids() {
        assert_eq!(block_state_name_network_id("minecraft:water"), Some(86));
        assert_eq!(block_state_name_network_id("minecraft:sand"), Some(118));
        assert_eq!(
            block_state_name_network_id("minecraft:sandstone"),
            Some(578)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:short_grass"),
            Some(2248)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:dandelion"),
            Some(2321)
        );
        assert_eq!(block_state_name_network_id("minecraft:poppy"), Some(2324));
        assert_eq!(block_state_name_network_id("minecraft:oak_log"), Some(137));
        assert_eq!(
            block_state_name_network_id("minecraft:birch_log"),
            Some(143)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:oak_leaves"),
            Some(279)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:birch_leaves"),
            Some(335)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:sunflower"),
            Some(12916)
        );
        assert_eq!(
            block_state_name_network_id("minecraft:deepslate"),
            Some(27924)
        );
    }

    #[test]
    fn storage_palette_network_bits_match_packed_storage_width() {
        let palette = (0..17).map(Tag::Int).collect::<Vec<_>>();
        let container = PalettedContainer {
            palette,
            data: Some(vec![16]),
            expected_entries: 4096,
        };

        let network = NetworkPalettedContainer::from_storage_container(
            &container.to_nbt(),
            PaletteKind::BlockState,
        );

        assert_eq!(network.bits_per_entry, 5);
        assert_eq!(network.palette_ids.len(), 17);
        assert_eq!(network.data, vec![16]);
    }

    #[test]
    fn biome_palette_network_ids_follow_synchronized_biome_registry_order() {
        assert_eq!(biome_name_network_id("minecraft:plains"), Some(40));
        assert_eq!(biome_name_network_id("plains"), Some(40));
        assert_eq!(biome_name_network_id("minecraft:the_void"), Some(57));

        let network = NetworkPalettedContainer::from_storage_container(
            &PalettedContainer::single(Tag::String("minecraft:plains".to_string()), 64).to_nbt(),
            PaletteKind::Biome,
        );

        assert_eq!(network.bits_per_entry, 0);
        assert_eq!(network.palette_ids, vec![40]);
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
        assert_eq!(chunk_data.buffer, vec![0x10, 0, 0, 0, 0, 5, 0, 7]);
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
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, vec![0])),
            DispatchOutcome::Disconnect(_)
        ));
        assert_eq!(session.state, PlayState::Playing);
        assert!(session.loaded);
    }

    #[test]
    fn malformed_serverbound_scalar_packets_disconnect_session() {
        let mut session = PlaySession::new(1, 0);
        session.state = PlayState::Playing;

        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CLIENT_COMMAND_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CLIENT_COMMAND_PACKET_ID, vec![3])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CLIENT_TICK_END_PACKET_ID, vec![0])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_PADDLE_BOAT_PACKET_ID, vec![1])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_INPUT_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, vec![0])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_SWING_PACKET_ID, vec![3])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_ACK_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_PACKET_ID, vec![1])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, vec![1])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, Vec::new())),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_CONTAINER_CLICK_PACKET_ID, vec![1, 2])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                vec![0]
            )),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(
                SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID,
                vec![0; 24]
            )),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, vec![0; 16])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID, vec![1])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID, vec![0; 8])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_EDIT_BOOK_PACKET_ID, vec![0, 101])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(SERVERBOUND_INTERACT_PACKET_ID, vec![1, 0])),
            DispatchOutcome::Disconnect(_)
        ));
        assert!(matches!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID,
                vec![0; 8]
            )),
            DispatchOutcome::Disconnect(_)
        ));
    }

    #[test]
    fn command_like_packets_wait_for_player_loaded_boundary() {
        let mut session = PlaySession::new(1, 0);
        session.state = PlayState::WaitingForPlayerLoaded;

        for id in [
            SERVERBOUND_CHAT_COMMAND_PACKET_ID,
            SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
            SERVERBOUND_CHAT_PACKET_ID,
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
        ] {
            assert!(matches!(
                session.handle_decoded(decoded(id, Vec::new())),
                DispatchOutcome::Disconnect(reason)
                    if reason == format!("command packet {id} before player_loaded")
            ));
        }

        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, Vec::new())),
            DispatchOutcome::Handled
        );
        let mut command_suggestion = Vec::new();
        ServerboundCommandSuggestionPacket {
            id: 7,
            command: "/ti".to_string(),
        }
        .write(&mut command_suggestion)
        .unwrap();
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
                command_suggestion
            )),
            DispatchOutcome::Handled
        );
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
    fn move_player_packet_shapes_match_vanilla_field_layouts() {
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

        let mut pos = Vec::new();
        movement.write_pos(&mut pos).unwrap();
        assert_eq!(pos.len(), 25);
        let decoded_pos =
            ServerboundMovePlayerPacket::read_shape(&mut cursor(pos), MoveShape::Pos).unwrap();
        assert_eq!(decoded_pos.x, 1.25);
        assert_eq!(decoded_pos.y, 65.0);
        assert_eq!(decoded_pos.z, -2.5);
        assert_eq!(decoded_pos.y_rot, 0.0);
        assert!(decoded_pos.on_ground);
        assert!(decoded_pos.horizontal_collision);
        assert!(decoded_pos.has_position);
        assert!(!decoded_pos.has_rotation);

        let mut pos_rot = Vec::new();
        movement.write_pos_rot(&mut pos_rot).unwrap();
        assert_eq!(pos_rot.len(), 33);
        let decoded_pos_rot =
            ServerboundMovePlayerPacket::read_shape(&mut cursor(pos_rot), MoveShape::PosRot)
                .unwrap();
        assert_eq!(decoded_pos_rot.x, 1.25);
        assert_eq!(decoded_pos_rot.y_rot, 90.0);
        assert!(decoded_pos_rot.has_position);
        assert!(decoded_pos_rot.has_rotation);

        let mut rot = Vec::new();
        movement.write_rot(&mut rot).unwrap();
        assert_eq!(rot.len(), 9);
        let decoded_rot =
            ServerboundMovePlayerPacket::read_shape(&mut cursor(rot), MoveShape::Rot).unwrap();
        assert_eq!(decoded_rot.x, 0.0);
        assert_eq!(decoded_rot.y_rot, 90.0);
        assert!(decoded_rot.horizontal_collision);
        assert!(!decoded_rot.has_position);
        assert!(decoded_rot.has_rotation);

        let mut status_only = Vec::new();
        movement.write_status_only(&mut status_only).unwrap();
        assert_eq!(status_only, vec![3]);
        let decoded_status = ServerboundMovePlayerPacket::read_shape(
            &mut cursor(status_only),
            MoveShape::StatusOnly,
        )
        .unwrap();
        assert!(decoded_status.on_ground);
        assert!(decoded_status.horizontal_collision);
        assert!(!decoded_status.has_position);
        assert!(!decoded_status.has_rotation);
    }

    #[test]
    fn move_vehicle_packet_matches_vanilla_field_layout() {
        let vehicle = ServerboundMoveVehiclePacket {
            position: Vec3 {
                x: 1.25,
                y: 65.0,
                z: -2.5,
            },
            y_rot: 90.0,
            x_rot: 30.0,
            on_ground: true,
        };
        let mut payload = Vec::new();
        vehicle.write(&mut payload).unwrap();
        assert_eq!(payload.len(), 33);

        let decoded_vehicle =
            ServerboundMoveVehiclePacket::read(&mut cursor(payload.clone())).unwrap();
        assert_eq!(decoded_vehicle, vehicle);

        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_MOVE_VEHICLE_PACKET_ID, payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_vehicle_move, Some(vehicle));
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

        let mut server_command = Vec::new();
        ServerboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Easy,
        }
        .write(&mut server_command)
        .unwrap();
        assert_eq!(
            ServerboundChangeDifficultyPacket::read(&mut cursor(server_command)).unwrap(),
            ServerboundChangeDifficultyPacket {
                difficulty: GameDifficulty::Easy,
            }
        );

        let mut client_command = Vec::new();
        ServerboundClientCommandPacket {
            action: ServerboundClientCommandAction::RequestStats,
        }
        .write(&mut client_command)
        .unwrap();
        assert_eq!(client_command, vec![1]);
        let parsed_client_command =
            ServerboundClientCommandPacket::read(&mut cursor(client_command)).unwrap();
        assert!(matches!(
            parsed_client_command.action,
            ServerboundClientCommandAction::RequestStats
        ));
        let mut unknown_client_command = Vec::new();
        ServerboundClientCommandPacket {
            action: ServerboundClientCommandAction::Unknown(128),
        }
        .write(&mut unknown_client_command)
        .unwrap();
        assert_eq!(unknown_client_command, vec![0x80, 0x01]);
        let parsed_unknown_client_command =
            ServerboundClientCommandPacket::read(&mut cursor(unknown_client_command)).unwrap();
        assert!(matches!(
            parsed_unknown_client_command.action,
            ServerboundClientCommandAction::Unknown(128)
        ));

        let mut client_tick_end = Vec::new();
        ServerboundClientTickEndPacket
            .write(&mut client_tick_end)
            .unwrap();
        assert_eq!(
            ServerboundClientTickEndPacket::read(&mut cursor(client_tick_end)).unwrap(),
            ServerboundClientTickEndPacket
        );

        let mut lock_difficulty = Vec::new();
        ServerboundLockDifficultyPacket { locked: true }
            .write(&mut lock_difficulty)
            .unwrap();
        assert_eq!(
            ServerboundLockDifficultyPacket::read(&mut cursor(lock_difficulty)).unwrap(),
            ServerboundLockDifficultyPacket { locked: true }
        );

        let mut paddle_boat = Vec::new();
        ServerboundPaddleBoatPacket {
            left: true,
            right: false,
        }
        .write(&mut paddle_boat)
        .unwrap();
        assert_eq!(
            ServerboundPaddleBoatPacket::read(&mut cursor(paddle_boat)).unwrap(),
            ServerboundPaddleBoatPacket {
                left: true,
                right: false,
            }
        );

        let mut player_input = Vec::new();
        ServerboundPlayerInputPacket {
            input: ServerboundPlayerInput {
                forward: true,
                backward: false,
                left: true,
                right: false,
                jump: true,
                shift: false,
                sprint: true,
            },
        }
        .write(&mut player_input)
        .unwrap();
        assert_eq!(
            ServerboundPlayerInputPacket::read(&mut cursor(player_input)).unwrap(),
            ServerboundPlayerInputPacket {
                input: ServerboundPlayerInput {
                    forward: true,
                    backward: false,
                    left: true,
                    right: false,
                    jump: true,
                    shift: false,
                    sprint: true,
                },
            }
        );

        let mut player_command = Vec::new();
        ServerboundPlayerCommandPacket {
            entity_id: 37,
            action: ServerboundPlayerCommandAction::StartRidingJump,
            data: 128,
        }
        .write(&mut player_command)
        .unwrap();
        assert_eq!(player_command, vec![37, 3, 0x80, 0x01]);
        assert_eq!(
            ServerboundPlayerCommandPacket::read(&mut cursor(player_command.clone())).unwrap(),
            ServerboundPlayerCommandPacket {
                entity_id: 37,
                action: ServerboundPlayerCommandAction::StartRidingJump,
                data: 128,
            }
        );
        let mut session = PlaySession::new(1, 0);
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
                player_command
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_player_command,
            Some(ServerboundPlayerCommandPacket {
                entity_id: 37,
                action: ServerboundPlayerCommandAction::StartRidingJump,
                data: 128,
            })
        );

        let mut player_action = Vec::new();
        ServerboundPlayerActionPacket {
            action: ServerboundPlayerAction::StopDestroyBlock,
            x: -12,
            y: 64,
            z: 34,
            direction: Direction3d::West,
            sequence: 300,
        }
        .write(&mut player_action)
        .unwrap();
        assert_eq!(player_action[0], 2);
        assert_eq!(player_action[9], 4);
        assert_eq!(&player_action[10..], &[0xac, 0x02]);
        let parsed_action =
            ServerboundPlayerActionPacket::read(&mut cursor(player_action.clone())).unwrap();
        assert_eq!(
            parsed_action,
            ServerboundPlayerActionPacket {
                action: ServerboundPlayerAction::StopDestroyBlock,
                x: -12,
                y: 64,
                z: 34,
                direction: Direction3d::West,
                sequence: 300,
            }
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_PLAYER_ACTION_PACKET_ID, player_action)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_player_action, Some(parsed_action));

        let mut player_loaded = Vec::new();
        ServerboundPlayerLoadedPacket
            .write(&mut player_loaded)
            .unwrap();
        assert_eq!(
            ServerboundPlayerLoadedPacket::read(&mut cursor(player_loaded)).unwrap(),
            ServerboundPlayerLoadedPacket
        );

        let mut swing = Vec::new();
        ServerboundSwingPacket {
            hand: ServerboundSwingHand::OffHand,
        }
        .write(&mut swing)
        .unwrap();
        assert_eq!(swing, vec![1]);
        assert_eq!(
            ServerboundSwingPacket::read(&mut cursor(swing)).unwrap(),
            ServerboundSwingPacket {
                hand: ServerboundSwingHand::OffHand,
            }
        );
        let mut unknown_swing = Vec::new();
        ServerboundSwingPacket {
            hand: ServerboundSwingHand::Unknown(128),
        }
        .write(&mut unknown_swing)
        .unwrap();
        assert_eq!(unknown_swing, vec![0x80, 0x01]);
        assert_eq!(
            ServerboundSwingPacket::read(&mut cursor(unknown_swing)).unwrap(),
            ServerboundSwingPacket {
                hand: ServerboundSwingHand::Unknown(128),
            }
        );

        let use_item = ServerboundUseItemPacket {
            hand: ServerboundSwingHand::OffHand,
            sequence: 300,
            y_rot: 45.0,
            x_rot: -10.5,
        };
        let mut use_item_payload = Vec::new();
        use_item.write(&mut use_item_payload).unwrap();
        assert_eq!(
            use_item_payload,
            vec![1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00]
        );
        assert_eq!(
            ServerboundUseItemPacket::read(&mut cursor(use_item_payload.clone())).unwrap(),
            use_item
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_PACKET_ID, use_item_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_use_item, Some(use_item));

        let use_item_on = ServerboundUseItemOnPacket {
            hand: ServerboundSwingHand::MainHand,
            block_hit: BlockHitResultPacketData {
                x: -12,
                y: 64,
                z: 34,
                direction: Direction3d::Up,
                click_x: 0.25,
                click_y: 0.5,
                click_z: 0.75,
                inside: true,
                world_border_hit: false,
            },
            sequence: 301,
        };
        let mut use_item_on_payload = Vec::new();
        use_item_on.write(&mut use_item_on_payload).unwrap();
        assert_eq!(use_item_on_payload[0], 0);
        assert_eq!(use_item_on_payload[9], 1);
        assert_eq!(&use_item_on_payload[22..24], &[1, 0]);
        assert_eq!(&use_item_on_payload[24..], &[0xad, 0x02]);
        assert_eq!(
            ServerboundUseItemOnPacket::read(&mut cursor(use_item_on_payload.clone())).unwrap(),
            use_item_on
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                use_item_on_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_use_item_on, Some(use_item_on));

        let mut pong = Vec::new();
        ServerboundPongPacket { id: 0x01020304 }
            .write(&mut pong)
            .unwrap();
        assert_eq!(pong, vec![1, 2, 3, 4]);
        assert_eq!(
            ServerboundPongPacket::read(&mut cursor(pong.clone())).unwrap(),
            ServerboundPongPacket { id: 0x01020304 }
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_PONG_PACKET_ID, pong)),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_pong,
            Some(ServerboundPongPacket { id: 0x01020304 })
        );

        let mut ping = Vec::new();
        ClientboundPingPacket { id: -0x01020304 }
            .write(&mut ping)
            .unwrap();
        assert_eq!(ping, (-0x01020304_i32).to_be_bytes());
        assert_eq!(
            ClientboundPingPacket::read(&mut cursor(ping)).unwrap(),
            ClientboundPingPacket { id: -0x01020304 }
        );

        let mut configuration_ack = Vec::new();
        ServerboundConfigurationAcknowledgedPacket
            .write(&mut configuration_ack)
            .unwrap();
        assert!(configuration_ack.is_empty());
        assert_eq!(
            ServerboundConfigurationAcknowledgedPacket::read(&mut cursor(
                configuration_ack.clone()
            ))
            .unwrap(),
            ServerboundConfigurationAcknowledgedPacket
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
                configuration_ack
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.state, PlayState::Reconfiguring);

        let mut jigsaw_generate = Vec::new();
        let jigsaw_packet = ServerboundJigsawGeneratePacket {
            x: -12,
            y: 64,
            z: 34,
            levels: 7,
            keep_jigsaws: true,
        };
        jigsaw_packet.write(&mut jigsaw_generate).unwrap();
        assert_eq!(jigsaw_generate.len(), 10);
        assert_eq!(&jigsaw_generate[8..], &[7, 1]);
        assert_eq!(
            ServerboundJigsawGeneratePacket::read(&mut cursor(jigsaw_generate.clone())).unwrap(),
            jigsaw_packet
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_JIGSAW_GENERATE_PACKET_ID,
                jigsaw_generate
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_jigsaw_generate, Some(jigsaw_packet));

        let sign_update = ServerboundSignUpdatePacket {
            x: -12,
            y: 64,
            z: 34,
            is_front_text: false,
            lines: [
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
            ],
        };
        let mut sign_payload = Vec::new();
        sign_update.write(&mut sign_payload).unwrap();
        assert_eq!(sign_payload[8], 0);
        assert_eq!(
            ServerboundSignUpdatePacket::read(&mut cursor(sign_payload.clone())).unwrap(),
            sign_update
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_SIGN_UPDATE_PACKET_ID, sign_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_sign_update, Some(sign_update));

        let set_beacon = ServerboundSetBeaconPacket {
            primary_effect_id: Some(1),
            secondary_effect_id: Some(128),
        };
        let mut set_beacon_payload = Vec::new();
        set_beacon.write(&mut set_beacon_payload).unwrap();
        assert_eq!(set_beacon_payload, vec![1, 1, 1, 0x80, 0x01]);
        assert_eq!(
            ServerboundSetBeaconPacket::read(&mut cursor(set_beacon_payload.clone())).unwrap(),
            set_beacon
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_BEACON_PACKET_ID,
                set_beacon_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_beacon, Some(set_beacon));

        let mut empty_beacon_payload = Vec::new();
        ServerboundSetBeaconPacket {
            primary_effect_id: None,
            secondary_effect_id: None,
        }
        .write(&mut empty_beacon_payload)
        .unwrap();
        assert_eq!(empty_beacon_payload, vec![0, 0]);

        let mut select_trade_payload = Vec::new();
        let select_trade = ServerboundSelectTradePacket { item: 128 };
        select_trade.write(&mut select_trade_payload).unwrap();
        assert_eq!(select_trade_payload, vec![0x80, 0x01]);
        assert_eq!(
            ServerboundSelectTradePacket::read(&mut cursor(select_trade_payload.clone())).unwrap(),
            select_trade
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SELECT_TRADE_PACKET_ID,
                select_trade_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_select_trade, Some(select_trade));

        let rename_item = ServerboundRenameItemPacket {
            name: "Sharp Thing".to_string(),
        };
        let mut rename_payload = Vec::new();
        rename_item.write(&mut rename_payload).unwrap();
        assert_eq!(rename_payload[0], 11);
        assert_eq!(
            ServerboundRenameItemPacket::read(&mut cursor(rename_payload.clone())).unwrap(),
            rename_item
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_RENAME_ITEM_PACKET_ID, rename_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_rename_item, Some(rename_item));

        let container_close = ServerboundContainerClosePacket { container_id: 128 };
        let mut container_close_payload = Vec::new();
        container_close.write(&mut container_close_payload).unwrap();
        assert_eq!(container_close_payload, vec![0x80, 0x01]);
        assert_eq!(
            ServerboundContainerClosePacket::read(&mut cursor(container_close_payload.clone()))
                .unwrap(),
            container_close
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CONTAINER_CLOSE_PACKET_ID,
                container_close_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_container_close, Some(container_close));

        let container_button_click = ServerboundContainerButtonClickPacket {
            container_id: 128,
            button_id: 7,
        };
        let mut button_click_payload = Vec::new();
        container_button_click
            .write(&mut button_click_payload)
            .unwrap();
        assert_eq!(button_click_payload, vec![0x80, 0x01, 7]);
        assert_eq!(
            ServerboundContainerButtonClickPacket::read(&mut cursor(button_click_payload.clone()))
                .unwrap(),
            container_button_click
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
                button_click_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_container_button_click,
            Some(container_button_click)
        );

        let creative_slot = ServerboundSetCreativeModeSlotPacket {
            slot_num: -1,
            item_stack: RawItemStack {
                count: 3,
                item_id: Some(42),
                components: RawDataComponentPatch {
                    added: vec![(7, vec![0xaa, 0xbb])],
                    removed: vec![9],
                },
            },
        };
        let mut creative_slot_payload = Vec::new();
        creative_slot.write(&mut creative_slot_payload).unwrap();
        assert_eq!(
            creative_slot_payload,
            vec![0xff, 0xff, 3, 42, 1, 1, 7, 2, 0xaa, 0xbb, 9]
        );
        assert_eq!(
            ServerboundSetCreativeModeSlotPacket::read(&mut cursor(creative_slot_payload.clone()))
                .unwrap(),
            creative_slot
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                creative_slot_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_creative_mode_slot, Some(creative_slot));

        let mut changed_slots = BTreeMap::new();
        changed_slots.insert(
            5,
            HashedStack {
                item_id: Some(42),
                count: 3,
                components: HashedPatchMap {
                    added_component_hashes: vec![(7, 0x01020304)],
                    removed_components: vec![9],
                },
            },
        );
        let container_click = ServerboundContainerClickPacket {
            container_id: 1,
            state_id: 2,
            slot_num: -1,
            button_num: -2,
            container_input: ContainerInput::Throw,
            changed_slots,
            carried_item: HashedStack::empty(),
        };
        let mut container_click_payload = Vec::new();
        container_click.write(&mut container_click_payload).unwrap();
        assert_eq!(
            container_click_payload,
            vec![1, 2, 0xff, 0xff, 0xfe, 4, 1, 0, 5, 1, 42, 3, 1, 7, 1, 2, 3, 4, 1, 9, 0]
        );
        assert_eq!(
            ServerboundContainerClickPacket::read(&mut cursor(container_click_payload.clone()))
                .unwrap(),
            container_click
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                container_click_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_container_click, Some(container_click));
        let mut too_many_changed_slots = vec![1, 2, 0, 0, 0, 0];
        write_var_i32(&mut too_many_changed_slots, 129).unwrap();
        assert!(
            ServerboundContainerClickPacket::read(&mut cursor(too_many_changed_slots)).is_err()
        );
        assert!(ServerboundContainerClickPacket {
            container_id: 0,
            state_id: 0,
            slot_num: 0,
            button_num: 0,
            container_input: ContainerInput::Pickup,
            changed_slots: (0..129).map(|slot| (slot, HashedStack::empty())).collect(),
            carried_item: HashedStack::empty(),
        }
        .write(&mut Vec::new())
        .is_err());

        let edit_book = ServerboundEditBookPacket {
            slot: 1,
            pages: vec!["page one".to_string(), "page two".to_string()],
            title: Some("Title".to_string()),
        };
        let mut edit_book_payload = Vec::new();
        edit_book.write(&mut edit_book_payload).unwrap();
        assert_eq!(
            edit_book_payload,
            vec![
                1, 2, 8, b'p', b'a', b'g', b'e', b' ', b'o', b'n', b'e', 8, b'p', b'a', b'g', b'e',
                b' ', b't', b'w', b'o', 1, 5, b'T', b'i', b't', b'l', b'e'
            ]
        );
        assert_eq!(
            ServerboundEditBookPacket::read(&mut cursor(edit_book_payload.clone())).unwrap(),
            edit_book
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_EDIT_BOOK_PACKET_ID, edit_book_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_edit_book, Some(edit_book));
        assert!(ServerboundEditBookPacket {
            slot: 0,
            pages: vec!["x".to_string(); 101],
            title: None,
        }
        .write(&mut Vec::new())
        .is_err());

        let interact = ServerboundInteractPacket {
            entity_id: 128,
            hand: ServerboundInteractionHand::OffHand,
            location: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            using_secondary_action: true,
        };
        let mut interact_payload = Vec::new();
        interact.write(&mut interact_payload).unwrap();
        assert_eq!(interact_payload, vec![0x80, 0x01, 1, 0, 1]);
        assert_eq!(
            ServerboundInteractPacket::read(&mut cursor(interact_payload.clone())).unwrap(),
            interact
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_INTERACT_PACKET_ID, interact_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_interact, Some(interact));
        let invalid_hand_payload = vec![1, 7, 0, 0];
        assert_eq!(
            ServerboundInteractPacket::read(&mut cursor(invalid_hand_payload)).unwrap(),
            ServerboundInteractPacket {
                entity_id: 1,
                hand: ServerboundInteractionHand::MainHand,
                location: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                using_secondary_action: false,
            }
        );

        let chat_ack = ServerboundChatAckPacket { offset: 128 };
        let mut chat_ack_payload = Vec::new();
        chat_ack.write(&mut chat_ack_payload).unwrap();
        assert_eq!(chat_ack_payload, vec![0x80, 0x01]);
        assert_eq!(
            ServerboundChatAckPacket::read(&mut cursor(chat_ack_payload.clone())).unwrap(),
            chat_ack
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_ACK_PACKET_ID, chat_ack_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_chat_ack, Some(chat_ack));

        let last_seen = LastSeenMessagesUpdate {
            offset: 2,
            acknowledged: vec![0b1010_0001, 0, 0b0000_1000],
            checksum: 5,
        };
        let chat = ServerboundChatPacket {
            message: "hi".to_string(),
            timestamp_epoch_millis: 100,
            salt: -7,
            signature: Some(MessageSignature([7; MessageSignature::BYTES])),
            last_seen_messages: last_seen.clone(),
        };
        let mut chat_payload = Vec::new();
        chat.write(&mut chat_payload).unwrap();
        assert_eq!(&chat_payload[..2], &[2, b'h']);
        assert_eq!(chat_payload[2], b'i');
        assert_eq!(&chat_payload[3..11], &100_i64.to_be_bytes());
        assert_eq!(&chat_payload[11..19], &(-7_i64).to_be_bytes());
        assert_eq!(chat_payload[19], 1);
        assert_eq!(&chat_payload[276..], &[2, 0b1010_0001, 0, 0b0000_1000, 5]);
        assert_eq!(
            ServerboundChatPacket::read(&mut cursor(chat_payload.clone())).unwrap(),
            chat
        );
        assert_eq!(
            session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, chat_payload)),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_chat, Some(chat));

        let chat_command = ServerboundChatCommandPacket {
            command: "seed".to_string(),
        };
        let mut chat_command_payload = Vec::new();
        chat_command.write(&mut chat_command_payload).unwrap();
        assert_eq!(chat_command_payload, vec![4, b's', b'e', b'e', b'd']);
        assert_eq!(
            ServerboundChatCommandPacket::read(&mut cursor(chat_command_payload.clone())).unwrap(),
            chat_command
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CHAT_COMMAND_PACKET_ID,
                chat_command_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_chat_command, Some(chat_command));

        let signed_command = ServerboundChatCommandSignedPacket {
            command: "msg Notch hello".to_string(),
            timestamp_epoch_millis: 101,
            salt: 9,
            argument_signatures: vec![ArgumentSignature {
                name: "message".to_string(),
                signature: MessageSignature([8; MessageSignature::BYTES]),
            }],
            last_seen_messages: last_seen,
        };
        let mut signed_command_payload = Vec::new();
        signed_command.write(&mut signed_command_payload).unwrap();
        assert_eq!(signed_command_payload[0], 15);
        assert_eq!(&signed_command_payload[16..24], &101_i64.to_be_bytes());
        assert_eq!(&signed_command_payload[24..32], &9_i64.to_be_bytes());
        assert_eq!(signed_command_payload[32], 1);
        assert_eq!(
            &signed_command_payload[33..41],
            &[7, b'm', b'e', b's', b's', b'a', b'g', b'e']
        );
        assert_eq!(
            &signed_command_payload[signed_command_payload.len() - 5..],
            &[2, 0b1010_0001, 0, 0b0000_1000, 5]
        );
        assert_eq!(
            ServerboundChatCommandSignedPacket::read(&mut cursor(signed_command_payload.clone()))
                .unwrap(),
            signed_command
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
                signed_command_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_signed_chat_command, Some(signed_command));
        assert!(LastSeenMessagesUpdate {
            offset: 0,
            acknowledged: vec![0, 0, 0x10],
            checksum: 0,
        }
        .write(&mut Vec::new())
        .is_err());
        assert!(ServerboundChatCommandSignedPacket {
            command: String::new(),
            timestamp_epoch_millis: 0,
            salt: 0,
            argument_signatures: vec![
                ArgumentSignature {
                    name: String::new(),
                    signature: MessageSignature([0; MessageSignature::BYTES]),
                };
                9
            ],
            last_seen_messages: LastSeenMessagesUpdate {
                offset: 0,
                acknowledged: vec![0, 0, 0],
                checksum: 0,
            },
        }
        .write(&mut Vec::new())
        .is_err());

        let chat_session_update = ServerboundChatSessionUpdatePacket {
            session_id: Uuid([4; 16]),
            expires_at_epoch_millis: 1_234_567_890,
            public_key: vec![1, 2, 3],
            key_signature: vec![4, 5],
        };
        let mut chat_session_payload = Vec::new();
        chat_session_update
            .write(&mut chat_session_payload)
            .unwrap();
        let mut expected_chat_session = vec![4; 16];
        expected_chat_session.extend_from_slice(&1_234_567_890_i64.to_be_bytes());
        expected_chat_session.extend_from_slice(&[3, 1, 2, 3, 2, 4, 5]);
        assert_eq!(chat_session_payload, expected_chat_session);
        assert_eq!(
            ServerboundChatSessionUpdatePacket::read(&mut cursor(chat_session_payload.clone()))
                .unwrap(),
            chat_session_update
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID,
                chat_session_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_chat_session_update, Some(chat_session_update));
        assert!(ServerboundChatSessionUpdatePacket {
            session_id: Uuid([0; 16]),
            expires_at_epoch_millis: 0,
            public_key: vec![0; 513],
            key_signature: Vec::new(),
        }
        .write(&mut Vec::new())
        .is_err());

        let resource_pack_response = ServerboundResourcePackPacket {
            id: Uuid([9; 16]),
            action: crate::network::common::ResourcePackAction::Accepted,
        };
        let mut resource_pack_payload = Vec::new();
        resource_pack_response
            .write(&mut resource_pack_payload)
            .unwrap();
        let mut expected_resource_pack = vec![9; 16];
        expected_resource_pack.push(3);
        assert_eq!(resource_pack_payload, expected_resource_pack);
        assert_eq!(
            ServerboundResourcePackPacket::read(&mut cursor(resource_pack_payload.clone()))
                .unwrap(),
            resource_pack_response
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_RESOURCE_PACK_PACKET_ID,
                resource_pack_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_resource_pack_response,
            Some(resource_pack_response)
        );

        let command_block = ServerboundSetCommandBlockPacket {
            x: -12,
            y: 64,
            z: 34,
            command: "say hi".to_string(),
            mode: CommandBlockMode::Redstone,
            track_output: true,
            conditional: false,
            automatic: true,
        };
        let mut command_block_payload = Vec::new();
        command_block.write(&mut command_block_payload).unwrap();
        assert_eq!(command_block_payload.len(), 17);
        assert_eq!(
            &command_block_payload[8..],
            &[6, b's', b'a', b'y', b' ', b'h', b'i', 2, 5]
        );
        assert_eq!(
            ServerboundSetCommandBlockPacket::read(&mut cursor(command_block_payload.clone()))
                .unwrap(),
            command_block
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID,
                command_block_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_command_block, Some(command_block));
        assert!(ServerboundSetCommandBlockPacket::read(&mut cursor(vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0
        ]))
        .is_err());

        let structure_block = ServerboundSetStructureBlockPacket {
            x: -12,
            y: 64,
            z: 34,
            update_type: StructureBlockUpdateType::LoadArea,
            mode: StructureBlockMode::Load,
            name: "demo:house".to_string(),
            offset: [-2, 3, 4],
            size: [5, 6, 7],
            mirror: StructureMirror::FrontBack,
            rotation: StructureRotation::Counterclockwise90,
            data: "metadata".to_string(),
            integrity: 0.75,
            seed: 128,
            ignore_entities: true,
            strict: true,
            show_air: false,
            show_bounding_box: true,
        };
        let mut structure_block_payload = Vec::new();
        structure_block.write(&mut structure_block_payload).unwrap();
        assert_eq!(
            &structure_block_payload[8..],
            &[
                2, 1, 10, b'd', b'e', b'm', b'o', b':', b'h', b'o', b'u', b's', b'e', 0xfe, 3, 4,
                5, 6, 7, 2, 3, 8, b'm', b'e', b't', b'a', b'd', b'a', b't', b'a', 0x3f, 0x40, 0, 0,
                0x80, 0x01, 13
            ]
        );
        assert_eq!(
            ServerboundSetStructureBlockPacket::read(&mut cursor(structure_block_payload.clone()))
                .unwrap(),
            structure_block
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID,
                structure_block_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_structure_block, Some(structure_block));

        let clamped_structure = ServerboundSetStructureBlockPacket::read(&mut cursor(vec![
            0, 0, 0, 0, 0, 0, 0, 0, // BlockPos
            0, 0, 0, // update type, mode, empty name
            200, 60, 255, // offset clamps to -48, 48, -1
            255, 60, 10, // size clamps to 0, 48, 10
            0, 7, 0, // mirror, wrapped rotation, empty data
            0x3f, 0xc0, 0, 0, // integrity 1.5 clamps to 1.0
            0, 15,
        ]))
        .unwrap();
        assert_eq!(clamped_structure.offset, [-48, 48, -1]);
        assert_eq!(clamped_structure.size, [0, 48, 10]);
        assert_eq!(
            clamped_structure.rotation,
            StructureRotation::Counterclockwise90
        );
        assert_eq!(clamped_structure.integrity, 1.0);
        assert!(clamped_structure.ignore_entities);
        assert!(clamped_structure.strict);
        assert!(clamped_structure.show_air);
        assert!(clamped_structure.show_bounding_box);

        let command_minecart = ServerboundSetCommandMinecartPacket {
            entity_id: 128,
            command: "say hi".to_string(),
            track_output: true,
        };
        let mut command_minecart_payload = Vec::new();
        command_minecart
            .write(&mut command_minecart_payload)
            .unwrap();
        assert_eq!(
            command_minecart_payload,
            vec![0x80, 0x01, 6, b's', b'a', b'y', b' ', b'h', b'i', 1]
        );
        assert_eq!(
            ServerboundSetCommandMinecartPacket::read(&mut cursor(
                command_minecart_payload.clone()
            ))
            .unwrap(),
            command_minecart
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID,
                command_minecart_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_set_command_minecart, Some(command_minecart));

        let command_suggestion = ServerboundCommandSuggestionPacket {
            id: 128,
            command: "/time set day".to_string(),
        };
        let mut command_suggestion_payload = Vec::new();
        command_suggestion
            .write(&mut command_suggestion_payload)
            .unwrap();
        assert_eq!(
            command_suggestion_payload,
            vec![
                0x80, 0x01, 13, b'/', b't', b'i', b'm', b'e', b' ', b's', b'e', b't', b' ', b'd',
                b'a', b'y'
            ]
        );
        assert_eq!(
            ServerboundCommandSuggestionPacket::read(&mut cursor(
                command_suggestion_payload.clone()
            ))
            .unwrap(),
            command_suggestion
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
                command_suggestion_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_command_suggestion, Some(command_suggestion));

        let pick_item_from_block = ServerboundPickItemFromBlockPacket {
            x: -12,
            y: 64,
            z: 34,
            include_data: true,
        };
        let mut pick_block_payload = Vec::new();
        pick_item_from_block.write(&mut pick_block_payload).unwrap();
        assert_eq!(pick_block_payload.len(), 9);
        assert_eq!(pick_block_payload[8], 1);
        assert_eq!(
            ServerboundPickItemFromBlockPacket::read(&mut cursor(pick_block_payload.clone()))
                .unwrap(),
            pick_item_from_block
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID,
                pick_block_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_pick_item_from_block,
            Some(pick_item_from_block)
        );

        let pick_item_from_entity = ServerboundPickItemFromEntityPacket {
            entity_id: 128,
            include_data: false,
        };
        let mut pick_entity_payload = Vec::new();
        pick_item_from_entity
            .write(&mut pick_entity_payload)
            .unwrap();
        assert_eq!(pick_entity_payload, vec![0x80, 0x01, 0]);
        assert_eq!(
            ServerboundPickItemFromEntityPacket::read(&mut cursor(pick_entity_payload.clone()))
                .unwrap(),
            pick_item_from_entity
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID,
                pick_entity_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_pick_item_from_entity,
            Some(pick_item_from_entity)
        );

        let recipe_book_settings = ServerboundRecipeBookChangeSettingsPacket {
            book_type: RecipeBookType::BlastFurnace,
            is_open: true,
            is_filtering: false,
        };
        let mut recipe_book_settings_payload = Vec::new();
        recipe_book_settings
            .write(&mut recipe_book_settings_payload)
            .unwrap();
        assert_eq!(recipe_book_settings_payload, vec![2, 1, 0]);
        assert_eq!(
            ServerboundRecipeBookChangeSettingsPacket::read(&mut cursor(
                recipe_book_settings_payload.clone()
            ))
            .unwrap(),
            recipe_book_settings
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
                recipe_book_settings_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(
            session.last_recipe_book_change_settings,
            Some(recipe_book_settings)
        );
        assert!(
            ServerboundRecipeBookChangeSettingsPacket::read(&mut cursor(vec![4, 0, 0])).is_err()
        );

        let seen_recipe = ServerboundRecipeBookSeenRecipePacket { recipe_index: 128 };
        let mut seen_recipe_payload = Vec::new();
        seen_recipe.write(&mut seen_recipe_payload).unwrap();
        assert_eq!(seen_recipe_payload, vec![0x80, 0x01]);
        assert_eq!(
            ServerboundRecipeBookSeenRecipePacket::read(&mut cursor(seen_recipe_payload.clone()))
                .unwrap(),
            seen_recipe
        );
        assert_eq!(
            session.handle_decoded(decoded(
                SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
                seen_recipe_payload
            )),
            DispatchOutcome::Handled
        );
        assert_eq!(session.last_recipe_book_seen_recipe, Some(seen_recipe));

        let mut change_difficulty = Vec::new();
        ClientboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Hard,
            locked: true,
        }
        .write(&mut change_difficulty)
        .unwrap();
        assert_eq!(
            ClientboundChangeDifficultyPacket::read(&mut cursor(change_difficulty)).unwrap(),
            ClientboundChangeDifficultyPacket {
                difficulty: GameDifficulty::Hard,
                locked: true,
            }
        );

        let mut chunk_cache_center = Vec::new();
        ClientboundSetChunkCacheCenterPacket { x: 12, z: -34 }
            .write(&mut chunk_cache_center)
            .unwrap();
        assert_eq!(
            ClientboundSetChunkCacheCenterPacket::read(&mut cursor(chunk_cache_center)).unwrap(),
            ClientboundSetChunkCacheCenterPacket { x: 12, z: -34 }
        );

        let mut chunk_cache_radius = Vec::new();
        ClientboundSetChunkCacheRadiusPacket { radius: 5 }
            .write(&mut chunk_cache_radius)
            .unwrap();
        assert_eq!(
            ClientboundSetChunkCacheRadiusPacket::read(&mut cursor(chunk_cache_radius)).unwrap(),
            ClientboundSetChunkCacheRadiusPacket { radius: 5 }
        );

        let mut spawn_position = Vec::new();
        ClientboundSetDefaultSpawnPositionPacket {
            respawn_data: ClientboundSetDefaultSpawnPositionData {
                dimension: Identifier::parse("minecraft:the_end").unwrap(),
                x: 1,
                y: 2,
                z: 3,
                yaw: 45.0,
                pitch: -23.5,
            },
        }
        .write(&mut spawn_position)
        .unwrap();
        assert_eq!(
            ClientboundSetDefaultSpawnPositionPacket::read(&mut cursor(spawn_position)).unwrap(),
            ClientboundSetDefaultSpawnPositionPacket {
                respawn_data: ClientboundSetDefaultSpawnPositionData {
                    dimension: Identifier::parse("minecraft:the_end").unwrap(),
                    x: 1,
                    y: 2,
                    z: 3,
                    yaw: 45.0,
                    pitch: -23.5,
                },
            }
        );

        let mut experience = Vec::new();
        ClientboundSetExperiencePacket {
            experience_progress: 0.75,
            experience_level: 3,
            total_experience: 42,
        }
        .write(&mut experience)
        .unwrap();
        assert_eq!(
            ClientboundSetExperiencePacket::read(&mut cursor(experience)).unwrap(),
            ClientboundSetExperiencePacket {
                experience_progress: 0.75,
                experience_level: 3,
                total_experience: 42,
            }
        );

        let mut health = Vec::new();
        ClientboundSetHealthPacket {
            health: 14.5,
            food: 19,
            saturation: 2.3,
        }
        .write(&mut health)
        .unwrap();
        assert_eq!(
            ClientboundSetHealthPacket::read(&mut cursor(health)).unwrap(),
            ClientboundSetHealthPacket {
                health: 14.5,
                food: 19,
                saturation: 2.3,
            }
        );

        let mut set_time = Vec::new();
        let mut clock_updates = BTreeMap::new();
        clock_updates.insert(
            Identifier::parse("minecraft:overworld").unwrap(),
            ClockNetworkState {
                total_ticks: 12345,
                partial_tick: 0.25,
                rate: 1.0,
            },
        );
        ClientboundSetTimePacket {
            game_time: 900_000,
            clock_updates,
        }
        .write(&mut set_time)
        .unwrap();
        assert_eq!(
            ClientboundSetTimePacket::read(&mut cursor(set_time)).unwrap(),
            ClientboundSetTimePacket {
                game_time: 900_000,
                clock_updates: BTreeMap::from([(
                    Identifier::parse("minecraft:overworld").unwrap(),
                    ClockNetworkState {
                        total_ticks: 12345,
                        partial_tick: 0.25,
                        rate: 1.0,
                    },
                )]),
            }
        );

        let mut game_event = Vec::new();
        ClientboundGameEventPacket {
            event: ClientboundGameEventType::RainLevelChange,
            param: 0.5,
        }
        .write(&mut game_event)
        .unwrap();
        assert_eq!(
            ClientboundGameEventPacket::read(&mut cursor(game_event)).unwrap(),
            ClientboundGameEventPacket {
                event: ClientboundGameEventType::RainLevelChange,
                param: 0.5,
            }
        );

        let mut simulation = Vec::new();
        ClientboundSetSimulationDistancePacket {
            simulation_distance: 8,
        }
        .write(&mut simulation)
        .unwrap();
        assert_eq!(
            ClientboundSetSimulationDistancePacket::read(&mut cursor(simulation)).unwrap(),
            ClientboundSetSimulationDistancePacket {
                simulation_distance: 8
            }
        );

        let mut ticking_state = Vec::new();
        ClientboundTickingStatePacket {
            tick_rate: 0.5,
            is_frozen: true,
        }
        .write(&mut ticking_state)
        .unwrap();
        assert_eq!(
            ClientboundTickingStatePacket::read(&mut cursor(ticking_state)).unwrap(),
            ClientboundTickingStatePacket {
                tick_rate: 0.5,
                is_frozen: true,
            }
        );

        let mut ticking_step = Vec::new();
        ClientboundTickingStepPacket { tick_steps: 7 }
            .write(&mut ticking_step)
            .unwrap();
        assert_eq!(
            ClientboundTickingStepPacket::read(&mut cursor(ticking_step)).unwrap(),
            ClientboundTickingStepPacket { tick_steps: 7 }
        );
    }

    #[test]
    fn serverbound_scalar_packet_payload_validation_rejects_malformed_inputs() {
        assert!(ServerboundClientCommandPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
        assert!(
            ServerboundChunkBatchReceivedPacket::read(&mut cursor(vec![0x7f, 0x7f, 0x7f])).is_err()
        );
        assert!(ServerboundLockDifficultyPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
        assert!(ServerboundPaddleBoatPacket::read(&mut cursor(vec![1])).is_err());
        assert!(ServerboundPlayerInputPacket::read(&mut cursor(Vec::<u8>::new())).is_err());
        assert!(ServerboundClientTickEndPacket::read(&mut cursor(vec![1])).is_err());
        assert!(ServerboundPlayerLoadedPacket::read(&mut cursor(vec![2])).is_err());
        assert!(ServerboundChangeDifficultyPacket::read(&mut cursor(vec![0x10])).is_err());
    }
}
