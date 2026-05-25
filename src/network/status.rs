use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::block_metadata::representative_state_definition;
use crate::command::{
    debug_biome_at_command_source, execute_builtin_command, LevelBasedPermissionSet,
    PermissionLevel, ServerCommandState,
};
use crate::console::ConsoleInput;
use crate::fluid::{
    block_item_can_replace, block_state_model_name, place_liquid,
    tick_fluid, FluidKind, LiquidPlaceResult,
};
use crate::inventory::same_item_same_components;
use crate::item_catalog::{item_protocol_id, item_static_name, item_static_name_from_protocol_id};
use crate::item_entity::{self, DroppedItem, WorldItemEntities, DEFAULT_PICKUP_DELAY};
use crate::item_properties::ItemComponent;
use crate::item_stack::ItemStack;
use crate::log::log_info;
use crate::loot_system::{
    LootCondition, LootContext, LootEntry, LootFunction, LootParamSet, LootPool, LootTable,
    NumberProvider,
};
use crate::network::codec::ComponentJson;
use crate::network::codec::{write_bitset, write_identifier, write_uuid, Uuid};
use crate::network::common::{
    ClientboundDisconnectPacket, ClientboundServerLinksPacket, ServerLinkEntry, ServerLinkLabel,
    ServerLinkType,
};
use crate::network::compression::CompressionState;
use crate::network::configuration::ClientboundCodeOfConductPacket;
use crate::network::login::{
    ClientboundLoginCompressionPacket, ClientboundLoginDisconnectPacket, LoginSession,
    ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
    CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID,
    CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID,
};
use crate::network::ping::{ClientboundPongResponsePacket, ServerboundPingRequestPacket};
use crate::network::play::{
    block_state_name_network_id, build_recipe_book_add, build_recipe_book_add_with_flags,
    handle_container_click, raw_item_stack_from_item_stack, unpack_block_position,
    ClientboundAddEntityPacket, ClientboundContainerSetSlotPacket, ClientboundLevelChunkPacketData,
    ClientboundLevelChunkWithLightPacket, ClientboundLightUpdatePacketData, ClientboundLoginPacket,
    ClientboundCommandsPacket, ClientboundRecipeBookSettingsPacket,
    ClientboundRemoveEntitiesPacket, ClientboundRespawnPacket, ClientboundSetEntityDataPacket,
    ClientboundSetEntityMotionPacket, ClientboundSetHeldSlotPacket,
    ClientboundSetPlayerInventoryPacket, ClientboundSetTimePacket, ClientboundSystemChatPacket,
    ClientboundTakeItemEntityPacket, CommandNodeEntryData, CommandNodeStubData,
    CommonPlayerSpawnInfo, Direction3d, EntityDataValue, EntityMetadataValue, GameMode,
    PlayInstruction, PlayerChunkSender, RawDataComponentPatch, RawItemStack, ReadyChunkBatch,
    RecipeBookType, RecipeBookTypeSettings, RespawnDataToKeep,
    ServerboundChatCommandPacket, ServerboundChatCommandSignedPacket, ServerboundChatPacket,
    ServerboundChunkBatchReceivedPacket, ServerboundCommandSuggestionPacket,
    ServerboundContainerClickPacket,
    ServerboundContainerClosePacket, ServerboundEditBookPacket, ServerboundPickItemFromBlockPacket,
    ServerboundPickItemFromEntityPacket, ServerboundPlaceRecipePacket,
    ServerboundPlayerAbilitiesPacket, ServerboundRecipeBookChangeSettingsPacket,
    ServerboundRecipeBookSeenRecipePacket, ServerboundSetCreativeModeSlotPacket,
    ServerboundSwingHand, ServerboundUseItemOnPacket, Vec3, CLIENTBOUND_ADD_ENTITY_PACKET_ID,
    CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
    CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
    CLIENTBOUND_COMMANDS_PACKET_ID, CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
    CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
    CLIENTBOUND_DISCONNECT_PACKET_ID, CLIENTBOUND_GAME_EVENT_PACKET_ID,
    CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID, CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
    CLIENTBOUND_LOGIN_PACKET_ID,
    CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID, CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID,
    CLIENTBOUND_PLAYER_POSITION_PACKET_ID, CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
    CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID, CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
    CLIENTBOUND_RESPAWN_PACKET_ID, CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
    CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
    CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID, CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
    CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
    CLIENTBOUND_SET_HEALTH_PACKET_ID, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
    CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID, CLIENTBOUND_SET_TIME_PACKET_ID,
    CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
    SERVERBOUND_CHAT_ACK_PACKET_ID, SERVERBOUND_CHAT_COMMAND_PACKET_ID,
    SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, SERVERBOUND_CHAT_PACKET_ID,
    SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
    SERVERBOUND_CLIENT_INFORMATION_PACKET_ID, SERVERBOUND_CLIENT_TICK_END_PACKET_ID,
    SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
    SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, SERVERBOUND_EDIT_BOOK_PACKET_ID,
    SERVERBOUND_KEEP_ALIVE_PACKET_ID, SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID, SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID,
    SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID, SERVERBOUND_PLACE_RECIPE_PACKET_ID,
    SERVERBOUND_PLAYER_ABILITIES_PACKET_ID, SERVERBOUND_PLAYER_ACTION_PACKET_ID,
    SERVERBOUND_PLAYER_COMMAND_PACKET_ID, SERVERBOUND_PLAYER_INPUT_PACKET_ID,
    SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
    SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID, SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
    SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, SERVERBOUND_SWING_PACKET_ID,
    SERVERBOUND_USE_ITEM_ON_PACKET_ID, SERVERBOUND_USE_ITEM_PACKET_ID,
};
use crate::network::rate_limit::{PacketRateDecision, PacketRateLimiter};
use crate::network::varint::{read_var_i32, write_var_i32, write_var_i64};
use crate::player_access::{NameAndId, PlayerAccess, ProxyConnectionDecision};
use crate::player_entity::{
    calculate_fall_damage, movement_exhaustion, starvation_damages, update_fall_distance,
    Difficulty as FoodDifficulty, FallDamageInput, FoodState, FoodTickOutcome,
    DEFAULT_FALL_DAMAGE_MULTIPLIER, DEFAULT_SAFE_FALL_DISTANCE, JUMP_EXHAUSTION,
    SPRINT_EXHAUSTION_PER_METER, SPRINT_JUMP_EXHAUSTION, SWIM_EXHAUSTION_PER_METER,
};
use crate::player_inventory::{
    InventoryAddResult, InventoryMenu, PlayerInventory, HOTBAR_SIZE, INVENTORY_SIZE, SLOT_OFFHAND,
};
use crate::recipe_system::{load_recipe_directory, RecipeManagerModel, RecipeMap};
use crate::registry::Identifier;
use crate::scheduled_tick::{LevelTickQueues, TickPriority};
use crate::server_properties::ServerProperties;
use crate::storage::chunk::{HeightmapKind, LevelChunk, PalettedContainer, SECTION_VOLUME};
use crate::storage::nbt::Tag;
use crate::storage::region::{ChunkPos, RegionCompression, RegionFile};
use crate::storage::world::{PrimaryLevelData, WorldLayout};
use crate::weather::{WeatherCycle, WeatherData, WeatherRandomDurations};
use crate::world_time::{ClockNetworkState, ScheduledTimeChanges, ServerClockManager};
use crate::worldgen::{
    fixup_spawn_height, generate_overworld_spawn_chunk_for_preset_with_mode,
    generate_overworld_spawn_chunk_for_preset_with_mode_timed,
    generate_overworld_spawn_chunk_region_for_preset_with_mode,
    generator_find_spawn_position_for_stem, resolve_world_preset, spawn_block_kind,
    spawn_search_candidate, spawn_search_candidate_count, spawn_search_radius,
    LiveChunkGenerationMode, SpawnBlockKind, SPAWN_SELECTION_CONSTANTS,
};

pub(super) const VERSION_NAME: &str = "26.1.2";
pub(super) const PROTOCOL_VERSION: i32 = 775;
pub(super) const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;
pub(super) const CLIENTBOUND_CONFIGURATION_DISCONNECT_PACKET_ID: i32 = 2;
pub(super) const CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
pub(super) const CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID: i32 = 7;
pub(super) const CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID: i32 = 12;
pub(super) const CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID: i32 = 13;
pub(super) const CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
pub(super) const CLIENTBOUND_CONFIGURATION_SERVER_LINKS_PACKET_ID: i32 = 16;
pub(super) const CLIENTBOUND_CONFIGURATION_CODE_OF_CONDUCT_PACKET_ID: i32 = 18;
pub(super) const SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID: i32 = 0;
pub(super) const SERVERBOUND_CONFIGURATION_COOKIE_RESPONSE_PACKET_ID: i32 = 1;
pub(super) const SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID: i32 = 2;
pub(super) const SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
pub(super) const SERVERBOUND_CONFIGURATION_KEEP_ALIVE_PACKET_ID: i32 = 4;
pub(super) const SERVERBOUND_CONFIGURATION_PONG_PACKET_ID: i32 = 5;
pub(super) const SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID: i32 = 6;
pub(super) const SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 7;
pub(super) const SERVERBOUND_CONFIGURATION_CUSTOM_CLICK_ACTION_PACKET_ID: i32 = 8;
pub(super) const SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID: i32 = 9;
pub(super) const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
pub(super) const CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
pub(super) const CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
pub(super) const CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID: i32 = 37;
pub(super) const CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID: i32 = 45;
pub(super) const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
pub(super) const LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID: u8 = 13;
pub(super) const PLAY_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);
/// Java: MinecraftServer runs at 20 TPS = 50ms per tick.
pub(super) const SERVER_TICK_DURATION: Duration = Duration::from_millis(50);
/// Java: MinecraftServer.forceGameTimeSynchronization() every 20 ticks (~1 second).
pub(super) const TIME_SYNC_INTERVAL: Duration = Duration::from_secs(1);
/// Vanilla overworld weather durations (ticks). Java: ServerLevel weather scheduling.
pub(super) const DEFAULT_WEATHER_DURATIONS: WeatherRandomDurations = WeatherRandomDurations {
    rain_delay: 12_000,
    rain_duration: 6_000,
    thunder_delay: 18_000,
    thunder_duration: 3_000,
};
/// How often to persist clock/weather state (every 5 minutes at 20 TPS).
pub(super) const PERSISTENCE_INTERVAL_TICKS: u64 = 6_000;
pub(super) const MIN_CHUNK_BATCH_RADIUS: i32 = 2;
pub(super) const MAX_CHUNK_BATCH_RADIUS: i32 = 16;
pub(super) const PLAY_COMMAND_SUGGESTIONS: &[&str] = &[
    "ban",
    "biome",
    "deop",
    "gamemode",
    "give",
    "help",
    "kick",
    "list",
    "me",
    "op",
    "pardon",
    "say",
    "tell",
    "tp",
    "whitelist",
];

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlaySessionState {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    fall_distance: f32,
    selected_slot: i32,
    health: f32,
    food_level: i32,
    food_saturation: f32,
    food_exhaustion: f32,
    food_tick_timer: i32,
    input_forward: bool,
    input_backward: bool,
    input_left: bool,
    input_right: bool,
    input_shift: bool,
    input_sprinting: bool,
    input_jumping: bool,
    air_supply: i32,
    in_water: bool,
    eye_in_water: bool,
    water_fluid_height: f64,
    water_velocity_x: f64,
    water_velocity_y: f64,
    water_velocity_z: f64,
    xp_progress: f32,
    xp_level: i32,
    xp_total: i32,
    xp_seed: i32,
    score: i32,
    game_mode: GameMode,
    previous_game_mode: Option<GameMode>,
    spawn: Option<PlayerSpawnData>,
    seen_credits: bool,
    entered_nether_position: Option<(f64, f64, f64)>,
    last_death_location: Option<PlayerGlobalPosData>,
    root_vehicle: Option<Tag>,
    active_effects: Vec<Tag>,
    ender_items: Vec<Tag>,
    abilities: PlayerNbtAbilities,
    /// Player inventory + 2×2 crafting grid. The state ID (incremented on each accepted
    /// container click or broadcast) is tracked separately in `container_state_id`.
    inventory_menu: InventoryMenu,
    /// Item currently held on the cursor (not in any slot).
    carried_item: ItemStack,
    /// Monotonically-increasing state ID matching `AbstractContainerMenu.stateId` in Java.
    /// Sent in every `ContainerSetSlot` and `ContainerSetContent` packet; validated by the
    /// server when a `ServerboundContainerClickPacket` arrives.
    container_state_id: i32,
    recipe_book_settings: ClientboundRecipeBookSettingsPacket,
}

mod chunk_0;
pub use chunk_0::*;

mod chunk_0_2;
pub use chunk_0_2::*;

mod chunk_a;
pub use chunk_a::*;

mod chunk_b;
pub use chunk_b::*;

mod chunk_b_2;
pub use chunk_b_2::*;

mod player_book_packets;
mod player_creative_packets;

mod chunk_c;
pub use chunk_c::*;

mod play_session_world_packets;
pub use play_session_world_packets::*;

mod chunk_d;
pub use chunk_d::*;

mod chunk_d_2;
pub use chunk_d_2::*;

mod chunk_e;
pub use chunk_e::*;

mod chunk_e_2;
pub use chunk_e_2::*;

#[cfg(test)]
mod tests;
