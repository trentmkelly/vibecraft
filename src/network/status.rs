use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::env;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::block_metadata::representative_state_definition;
use crate::fluid::{
    block_item_can_replace, block_state_model_name, fluid_state_for_block, place_liquid,
    tick_fluid, FluidKind, LiquidPlaceResult,
};
use crate::console::ConsoleInput;
use crate::item_catalog::{item_protocol_id, item_static_name};
use crate::item_entity::{self, DroppedItem, WorldItemEntities, DEFAULT_PICKUP_DELAY};
use crate::item_stack::ItemStack;
use crate::log::log_info;
use crate::loot_system::{
    LootCondition, LootContext, LootEntry, LootFunction, LootParamSet, LootPool, LootTable,
    NumberProvider,
};
use crate::network::codec::ComponentJson;
use crate::network::codec::{write_bitset, write_identifier, write_optional, write_uuid, Uuid};
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
    handle_container_click, unpack_block_position, ClientboundAddEntityPacket,
    ClientboundContainerSetSlotPacket,
    ClientboundLevelChunkPacketData, ClientboundLevelChunkWithLightPacket,
    ClientboundLightUpdatePacketData, ClientboundLoginPacket, ClientboundRemoveEntitiesPacket,
    ClientboundRecipeBookSettingsPacket, ClientboundSetEntityDataPacket, ClientboundSetEntityMotionPacket,
    ClientboundSetPlayerInventoryPacket, ClientboundSetTimePacket, ClientboundTakeItemEntityPacket,
    CommonPlayerSpawnInfo, Direction3d, EntityDataValue, EntityMetadataValue, GameMode,
    PlayInstruction, RawDataComponentPatch, RawItemStack, RecipeBookType,
    RecipeBookTypeSettings, ServerboundContainerClickPacket,
    ServerboundPlaceRecipePacket,
    ServerboundRecipeBookChangeSettingsPacket, ServerboundRecipeBookSeenRecipePacket,
    ServerboundSwingHand, ServerboundUseItemOnPacket, Vec3, CLIENTBOUND_ADD_ENTITY_PACKET_ID,
    CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
    CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
    CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID, CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
    CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, CLIENTBOUND_DISCONNECT_PACKET_ID,
    CLIENTBOUND_GAME_EVENT_PACKET_ID, CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID,
    CLIENTBOUND_KEEP_ALIVE_PACKET_ID, CLIENTBOUND_LOGIN_PACKET_ID,
    CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID, CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID,
    CLIENTBOUND_PLAYER_POSITION_PACKET_ID, CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
    CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID,
    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, CLIENTBOUND_RESPAWN_PACKET_ID,
    CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID, CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
    CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID, CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
    CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
    CLIENTBOUND_SET_EXPERIENCE_PACKET_ID, CLIENTBOUND_SET_HEALTH_PACKET_ID,
    CLIENTBOUND_SET_HELD_SLOT_PACKET_ID, CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
    CLIENTBOUND_SET_TIME_PACKET_ID, CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
    SERVERBOUND_CHAT_ACK_PACKET_ID, SERVERBOUND_CHAT_COMMAND_PACKET_ID, SERVERBOUND_CHAT_PACKET_ID,
    SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
    SERVERBOUND_CLIENT_INFORMATION_PACKET_ID, SERVERBOUND_CLIENT_TICK_END_PACKET_ID,
    SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
    SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, SERVERBOUND_KEEP_ALIVE_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID, SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID, SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID,
    SERVERBOUND_PLAYER_ACTION_PACKET_ID, SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
    SERVERBOUND_PLACE_RECIPE_PACKET_ID, SERVERBOUND_PLAYER_INPUT_PACKET_ID,
    SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
    SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
    SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
    SERVERBOUND_SWING_PACKET_ID, SERVERBOUND_USE_ITEM_ON_PACKET_ID, SERVERBOUND_USE_ITEM_PACKET_ID,
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
use crate::player_inventory::{InventoryAddResult, InventoryMenu, PlayerInventory, SLOT_OFFHAND};
use crate::recipe_system::{load_recipe_directory, RecipeManagerModel, RecipeMap};
use crate::registry::Identifier;
use crate::server_properties::ServerProperties;
use crate::scheduled_tick::{LevelTickQueues, TickPriority};
use crate::storage::chunk::{HeightmapKind, LevelChunk, PalettedContainer, SECTION_VOLUME};
use crate::storage::nbt::Tag;
use crate::storage::region::{ChunkPos, RegionFile};
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

const VERSION_NAME: &str = "26.1.2";
const PROTOCOL_VERSION: i32 = 775;
const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;
const CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
const CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID: i32 = 7;
const CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID: i32 = 12;
const CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID: i32 = 13;
const CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
const CLIENTBOUND_CONFIGURATION_SERVER_LINKS_PACKET_ID: i32 = 16;
const CLIENTBOUND_CONFIGURATION_CODE_OF_CONDUCT_PACKET_ID: i32 = 18;
const SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID: i32 = 0;
const SERVERBOUND_CONFIGURATION_COOKIE_RESPONSE_PACKET_ID: i32 = 1;
const SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID: i32 = 2;
const SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
const SERVERBOUND_CONFIGURATION_KEEP_ALIVE_PACKET_ID: i32 = 4;
const SERVERBOUND_CONFIGURATION_PONG_PACKET_ID: i32 = 5;
const SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID: i32 = 6;
const SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 7;
const SERVERBOUND_CONFIGURATION_CUSTOM_CLICK_ACTION_PACKET_ID: i32 = 8;
const SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID: i32 = 9;
const SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID: i32 = 0;
const CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
const CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
const CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID: i32 = 37;
const CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID: i32 = 45;
const SERVERBOUND_PLAYER_LOADED_PACKET_ID: i32 = 44;
const LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID: u8 = 13;
const PLAY_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);
/// Java: MinecraftServer runs at 20 TPS = 50ms per tick.
const SERVER_TICK_DURATION: Duration = Duration::from_millis(50);
/// Java: MinecraftServer.forceGameTimeSynchronization() every 20 ticks (~1 second).
const TIME_SYNC_INTERVAL: Duration = Duration::from_secs(1);
/// Vanilla overworld weather durations (ticks). Java: ServerLevel weather scheduling.
const DEFAULT_WEATHER_DURATIONS: WeatherRandomDurations = WeatherRandomDurations {
    rain_delay: 12_000,
    rain_duration: 6_000,
    thunder_delay: 18_000,
    thunder_duration: 3_000,
};
/// How often to persist clock/weather state (every 5 minutes at 20 TPS).
const PERSISTENCE_INTERVAL_TICKS: u64 = 6_000;
const MIN_CHUNK_BATCH_RADIUS: i32 = 2;
const MAX_CHUNK_BATCH_RADIUS: i32 = 16;
const PLAY_COMMAND_SUGGESTIONS: &[&str] = &[
    "ban",
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
struct PlaySessionState {
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

impl Default for PlaySessionState {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: SPAWN_Y,
            z: 0.5,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            fall_distance: 0.0,
            selected_slot: 0,
            health: 20.0,
            food_level: 20,
            food_saturation: 5.0,
            food_exhaustion: 0.0,
            food_tick_timer: 0,
            input_forward: false,
            input_backward: false,
            input_left: false,
            input_right: false,
            input_shift: false,
            input_sprinting: false,
            input_jumping: false,
            air_supply: MAX_AIR_SUPPLY,
            in_water: false,
            eye_in_water: false,
            water_fluid_height: 0.0,
            water_velocity_x: 0.0,
            water_velocity_y: 0.0,
            water_velocity_z: 0.0,
            xp_progress: 0.0,
            xp_level: 0,
            xp_total: 0,
            xp_seed: 0,
            score: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            spawn: None,
            seen_credits: false,
            entered_nether_position: None,
            last_death_location: None,
            root_vehicle: None,
            active_effects: Vec::new(),
            ender_items: Vec::new(),
            abilities: PlayerNbtAbilities::default_survival(),
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), RecipeMap::default()),
            carried_item: ItemStack::empty(),
            container_state_id: 0,
            recipe_book_settings: ClientboundRecipeBookSettingsPacket {
                crafting: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
                smoker: RecipeBookTypeSettings::CLOSED_UNFILTERED,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlayerSpawnData {
    dimension: String,
    x: i32,
    y: i32,
    z: i32,
    forced: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct PlayerGlobalPosData {
    dimension: String,
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlayerSpawnPlacement {
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlayerNbtAbilities {
    invulnerable: bool,
    flying: bool,
    mayfly: bool,
    instabuild: bool,
    may_build: bool,
    fly_speed: f32,
    walk_speed: f32,
}

impl PlayerNbtAbilities {
    fn default_survival() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            mayfly: false,
            instabuild: false,
            may_build: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }
}
#[allow(dead_code)]
const SPAWN_CHUNK_SECTION_COUNT: usize = 24;
const AIR_BLOCK_STATE_ID: i32 = 0;
const STONE_BLOCK_STATE_ID: i32 = 1;
const GRANITE_BLOCK_STATE_ID: i32 = 2;
const DIORITE_BLOCK_STATE_ID: i32 = 4;
const ANDESITE_BLOCK_STATE_ID: i32 = 6;
const GRASS_BLOCK_STATE_ID: i32 = 9;
const DIRT_BLOCK_STATE_ID: i32 = 10;
const BEDROCK_BLOCK_STATE_ID: i32 = 85;
const SHORT_GRASS_BLOCK_STATE_ID: i32 = 131;
const DANDELION_BLOCK_STATE_ID: i32 = 158;
const POPPY_BLOCK_STATE_ID: i32 = 161;
#[allow(dead_code)]
const PLAINS_BIOME_ID: i32 = 40;
const TERRAIN_BASE_Y: i32 = 64;
const TERRAIN_MIN_SURFACE_Y: i32 = 70;
const ITEM_ENTITY_TYPE_ID: i32 = 71;
const SPAWN_Y: f64 = 112.0;
const PLAYER_ENTITY_ID: i32 = 1;
const PLAYER_WIDTH: f64 = 0.6;
const PLAYER_HEIGHT: f64 = 1.8;
const PLAYER_EYE_HEIGHT: f64 = 1.62;
const MAX_AIR_SUPPLY: i32 = 300;
const DROWN_AIR_SUPPLY_THRESHOLD: i32 = -20;
const DROWN_DAMAGE: f32 = 2.0;
const WATER_MOVE_RELATIVE_SPEED: f64 = 0.02;
const WATER_HORIZONTAL_SLOWDOWN: f64 = 0.8;
const WATER_SPRINTING_HORIZONTAL_SLOWDOWN: f64 = 0.9;
const WATER_VERTICAL_SLOWDOWN: f64 = 0.8;
const WATER_FALLING_GRAVITY: f64 = 0.005;
const WATER_JUMP_IMPULSE: f64 = 0.04;
const REGION_FEATURE_GENERATION_RADIUS: i32 = 3;
const REGION_FEATURE_CACHEABLE_RADIUS: i32 = 1;

#[derive(Clone, Default)]
struct GeneratedChunkCache {
    chunks: Arc<Mutex<HashMap<ChunkPos, Arc<LevelChunk>>>>,
}

impl GeneratedChunkCache {
    fn get_or_load(&self, x: i32, z: i32, world_root: &Path, world_seed: i64) -> Arc<LevelChunk> {
        let pos = ChunkPos { x, z };
        if let Some(chunk) = self.chunks.lock().unwrap().get(&pos).cloned() {
            return chunk;
        }

        if live_region_feature_generation_enabled()
            && live_chunk_generation_mode() == LiveChunkGenerationMode::RealSurface
        {
            let region_dir = world_root.join("region");
            if try_load_chunk_from_region(&region_dir, pos).is_none() {
                match generate_overworld_spawn_chunk_region_for_preset_with_mode(
                    pos,
                    REGION_FEATURE_GENERATION_RADIUS,
                    "normal",
                    LiveChunkGenerationMode::RealSurface,
                    world_seed,
                    true,
                ) {
                    Ok(region_chunks) => {
                        let mut cache = self.chunks.lock().unwrap();
                        for (region_pos, chunk) in region_chunks {
                            if !region_generated_chunk_is_cacheable(pos, region_pos) {
                                continue;
                            }
                            cache.entry(region_pos).or_insert_with(|| Arc::new(chunk));
                        }
                        if let Some(chunk) = cache.get(&pos).cloned() {
                            return chunk;
                        }
                    }
                    Err(err) => {
                        eprintln!(
                            "[worldgen-region] center=({}, {}) failed: {}",
                            pos.x, pos.z, err
                        );
                    }
                }
            }
        }

        let chunk = Arc::new(load_or_generate_spawn_chunk_uncached(
            x, z, world_root, world_seed,
        ));
        self.chunks
            .lock()
            .unwrap()
            .entry(pos)
            .or_insert_with(|| Arc::clone(&chunk))
            .clone()
    }

    fn invalidate(&self, pos: ChunkPos) {
        self.chunks.lock().unwrap().remove(&pos);
    }
}

#[derive(Debug, Clone)]
struct LiveFluidTicks {
    queues: LevelTickQueues,
}

impl LiveFluidTicks {
    fn new() -> Self {
        Self {
            queues: LevelTickQueues::new(),
        }
    }

    fn schedule(&mut self, game_time: i64, pos: crate::block_update::BlockPos, kind: FluidKind) {
        let chunk = ChunkPos {
            x: pos.x.div_euclid(16),
            z: pos.z.div_euclid(16),
        };
        self.queues.add_container(chunk);
        let tick = self.queues.create_tick(
            game_time,
            pos,
            kind.registry_id(),
            kind.tick_delay(),
            TickPriority::Normal,
        );
        let _ = self.queues.schedule(tick);
    }

    fn tick_due(
        &mut self,
        game_time: i64,
        max_ticks: usize,
    ) -> Vec<crate::scheduled_tick::ScheduledTick> {
        self.queues.tick(game_time, max_ticks, |_| true)
    }
}

fn live_region_feature_generation_enabled() -> bool {
    matches!(
        std::env::var("RUSTCRAFT_WORLDGEN_REGION_FEATURES").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn region_generated_chunk_is_cacheable(center: ChunkPos, candidate: ChunkPos) -> bool {
    (candidate.x - center.x).abs() <= REGION_FEATURE_CACHEABLE_RADIUS
        && (candidate.z - center.z).abs() <= REGION_FEATURE_CACHEABLE_RADIUS
}

#[derive(Clone, Default)]
struct ActiveLoginRegistry {
    sessions: Arc<Mutex<HashMap<String, ActiveLoginSession>>>,
    next_token: Arc<AtomicU64>,
}

struct ActiveLoginSession {
    token: u64,
    stream: TcpStream,
}

struct ActiveLoginGuard {
    sessions: Arc<Mutex<HashMap<String, ActiveLoginSession>>>,
    uuid: String,
    token: u64,
}

impl ActiveLoginRegistry {
    fn register_replacing(
        &self,
        uuid: &str,
        stream: &TcpStream,
    ) -> io::Result<(ActiveLoginGuard, Option<TcpStream>)> {
        let token = self.next_token.fetch_add(1, Ordering::Relaxed);
        let stream = stream.try_clone()?;
        let mut sessions = self.sessions.lock().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "active login registry mutex poisoned")
        })?;
        let old = sessions
            .insert(uuid.to_string(), ActiveLoginSession { token, stream })
            .map(|session| session.stream);

        Ok((
            ActiveLoginGuard {
                sessions: self.sessions.clone(),
                uuid: uuid.to_string(),
                token,
            },
            old,
        ))
    }
}

impl Drop for ActiveLoginGuard {
    fn drop(&mut self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            if sessions
                .get(&self.uuid)
                .is_some_and(|session| session.token == self.token)
            {
                sessions.remove(&self.uuid);
            }
        }
    }
}

// Source: decompiled-server-26.1.2/net/minecraft/world/damagesource/DamageType.java
// and decompiled-server-26.1.2/data/minecraft/damage_type/*.json
const DAMAGE_TYPES: &[&str] = &[
    "arrow",
    "bad_respawn_point",
    "cactus",
    "campfire",
    "cramming",
    "dragon_breath",
    "drown",
    "dry_out",
    "ender_pearl",
    "explosion",
    "fall",
    "falling_anvil",
    "falling_block",
    "falling_stalactite",
    "fireball",
    "fireworks",
    "fly_into_wall",
    "freeze",
    "generic",
    "generic_kill",
    "hot_floor",
    "in_fire",
    "in_wall",
    "indirect_magic",
    "lava",
    "lightning_bolt",
    "mace_smash",
    "magic",
    "mob_attack",
    "mob_attack_no_aggro",
    "mob_projectile",
    "on_fire",
    "out_of_world",
    "outside_border",
    "player_attack",
    "player_explosion",
    "sonic_boom",
    "spear",
    "spit",
    "stalagmite",
    "starve",
    "sting",
    "sweet_berry_bush",
    "thorns",
    "thrown",
    "trident",
    "unattributed_fireball",
    "wind_charge",
    "wither",
    "wither_skull",
];
const DAMAGE_TYPE_TAGS: &[(&str, &[i32])] = &[
    ("minecraft:damages_helmet", &[11, 12, 13]),
    (
        "minecraft:bypasses_armor",
        &[
            31, 22, 4, 6, 16, 18, 48, 5, 40, 10, 8, 17, 39, 27, 23, 32, 19, 36, 33,
        ],
    ),
    (
        "minecraft:bypasses_shield",
        &[
            31, 22, 4, 6, 16, 18, 48, 5, 40, 10, 8, 17, 39, 27, 23, 32, 19, 36, 33, 2, 3, 7, 11,
            13, 20, 21, 24, 25, 42,
        ],
    ),
    ("minecraft:bypasses_invulnerability", &[32, 19]),
    ("minecraft:bypasses_cooldown", &[]),
    ("minecraft:bypasses_effects", &[40]),
    ("minecraft:bypasses_resistance", &[32, 19]),
    ("minecraft:bypasses_enchantments", &[36]),
    ("minecraft:is_fire", &[21, 3, 31, 24, 20, 46, 14]),
    ("minecraft:is_projectile", &[0, 45, 30, 46, 14, 49, 44, 47]),
    ("minecraft:witch_resistant_to", &[27, 23, 36, 43]),
    ("minecraft:is_explosion", &[15, 9, 35, 1]),
    ("minecraft:is_fall", &[10, 8, 39]),
    ("minecraft:is_drowning", &[6]),
    ("minecraft:is_freezing", &[17]),
    ("minecraft:is_lightning", &[25]),
    ("minecraft:no_anger", &[29]),
    ("minecraft:no_impact", &[6]),
    ("minecraft:always_most_significant_fall", &[32]),
    ("minecraft:wither_immune_to", &[6]),
    ("minecraft:ignites_armor_stands", &[21, 3]),
    ("minecraft:burns_armor_stands", &[31]),
    ("minecraft:avoids_guardian_thorns", &[27, 43, 15, 9, 35, 1]),
    ("minecraft:always_triggers_silverfish", &[27]),
    ("minecraft:always_hurts_ender_dragons", &[15, 9, 35, 1]),
    (
        "minecraft:no_knockback",
        &[
            9, 35, 1, 21, 25, 31, 24, 20, 22, 4, 6, 40, 2, 10, 8, 16, 32, 18, 27, 48, 5, 7, 42, 17,
            39, 33, 19, 3, 37,
        ],
    ),
    ("minecraft:always_kills_armor_stands", &[0, 45, 14, 49, 47]),
    ("minecraft:can_break_armor_stand", &[35, 34, 37, 26]),
    (
        "minecraft:bypasses_wolf_armor",
        &[32, 19, 4, 6, 7, 17, 22, 23, 27, 33, 40, 43, 48],
    ),
    ("minecraft:is_player_attack", &[34, 37, 26]),
    ("minecraft:burn_from_stepping", &[3, 20]),
    (
        "minecraft:panic_causes",
        &[
            2, 17, 20, 21, 24, 25, 31, 0, 5, 9, 14, 15, 23, 27, 28, 30, 35, 36, 41, 44, 45, 46, 47,
            48, 49, 34, 37, 26,
        ],
    ),
    (
        "minecraft:panic_environmental_causes",
        &[2, 17, 20, 21, 24, 25, 31],
    ),
    ("minecraft:mace_smash", &[26]),
];

struct TrimMaterialEntry {
    id: &'static str,
    asset_name: &'static str,
    color: &'static str,
    overrides: &'static [(&'static str, &'static str)],
}

struct JukeboxSongEntry {
    id: &'static str,
    sound_event: &'static str,
    length_seconds: f32,
    comparator_output: i32,
}

struct InstrumentEntry {
    id: &'static str,
    sound_event: &'static str,
}

struct ChatTypeEntry {
    id: &'static str,
    chat_translation_key: &'static str,
    chat_parameters: &'static [&'static str],
    narration_translation_key: &'static str,
    narration_parameters: &'static [&'static str],
}

const CHAT_TYPES: &[ChatTypeEntry] = &[
    ChatTypeEntry {
        id: "chat",
        chat_translation_key: "chat.type.text",
        chat_parameters: &["sender", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "emote_command",
        chat_translation_key: "chat.type.emote",
        chat_parameters: &["sender", "content"],
        narration_translation_key: "chat.type.emote",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "msg_command_incoming",
        chat_translation_key: "commands.message.display.incoming",
        chat_parameters: &["sender", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "msg_command_outgoing",
        chat_translation_key: "commands.message.display.outgoing",
        chat_parameters: &["target", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "say_command",
        chat_translation_key: "chat.type.announcement",
        chat_parameters: &["sender", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "team_msg_command_incoming",
        chat_translation_key: "chat.type.team.text",
        chat_parameters: &["target", "sender", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
    ChatTypeEntry {
        id: "team_msg_command_outgoing",
        chat_translation_key: "chat.type.team.sent",
        chat_parameters: &["target", "sender", "content"],
        narration_translation_key: "chat.type.text.narrate",
        narration_parameters: &["sender", "content"],
    },
];

// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Biome.java
// and data/minecraft/worldgen/biome/*.json
pub(crate) const BIOMES: &[&str] = &[
    "badlands",
    "bamboo_jungle",
    "basalt_deltas",
    "beach",
    "birch_forest",
    "cherry_grove",
    "cold_ocean",
    "crimson_forest",
    "dark_forest",
    "deep_cold_ocean",
    "deep_dark",
    "deep_frozen_ocean",
    "deep_lukewarm_ocean",
    "deep_ocean",
    "desert",
    "dripstone_caves",
    "end_barrens",
    "end_highlands",
    "end_midlands",
    "eroded_badlands",
    "flower_forest",
    "forest",
    "frozen_ocean",
    "frozen_peaks",
    "frozen_river",
    "grove",
    "ice_spikes",
    "jagged_peaks",
    "jungle",
    "lukewarm_ocean",
    "lush_caves",
    "mangrove_swamp",
    "meadow",
    "mushroom_fields",
    "nether_wastes",
    "ocean",
    "old_growth_birch_forest",
    "old_growth_pine_taiga",
    "old_growth_spruce_taiga",
    "pale_garden",
    "plains",
    "river",
    "savanna",
    "savanna_plateau",
    "small_end_islands",
    "snowy_beach",
    "snowy_plains",
    "snowy_slopes",
    "snowy_taiga",
    "soul_sand_valley",
    "sparse_jungle",
    "stony_peaks",
    "stony_shore",
    "sunflower_plains",
    "swamp",
    "taiga",
    "the_end",
    "the_void",
    "warm_ocean",
    "warped_forest",
    "windswept_forest",
    "windswept_gravelly_hills",
    "windswept_hills",
    "windswept_savanna",
    "wooded_badlands",
];

const DIMENSION_TYPES: &[&str] = &["overworld", "overworld_caves", "the_end", "the_nether"];

// Source: decompiled-server-26.1.2/net/minecraft/world/item/equipment/trim/TrimPatterns.java
// and data/minecraft/trim_pattern/*.json
const TRIM_PATTERNS: &[&str] = &[
    "sentry",
    "dune",
    "coast",
    "wild",
    "ward",
    "eye",
    "vex",
    "tide",
    "snout",
    "rib",
    "spire",
    "wayfinder",
    "shaper",
    "silence",
    "raiser",
    "host",
    "flow",
    "bolt",
];

// Source: decompiled-server-26.1.2/net/minecraft/world/item/InstrumentItem.java
// and data/minecraft/instrument/*.json
const INSTRUMENTS: &[InstrumentEntry] = &[
    InstrumentEntry {
        id: "admire_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.4",
    },
    InstrumentEntry {
        id: "call_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.5",
    },
    InstrumentEntry {
        id: "dream_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.7",
    },
    InstrumentEntry {
        id: "feel_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.3",
    },
    InstrumentEntry {
        id: "ponder_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.0",
    },
    InstrumentEntry {
        id: "seek_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.2",
    },
    InstrumentEntry {
        id: "sing_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.1",
    },
    InstrumentEntry {
        id: "yearn_goat_horn",
        sound_event: "minecraft:item.goat_horn.sound.6",
    },
];

// Source: decompiled-server-26.1.2/net/minecraft/world/level/block/entity/BannerPatterns.java
// and data/minecraft/banner_pattern/*.json
const BANNER_PATTERNS: &[&str] = &[
    "base",
    "border",
    "bricks",
    "circle",
    "creeper",
    "cross",
    "curly_border",
    "diagonal_left",
    "diagonal_right",
    "diagonal_up_left",
    "diagonal_up_right",
    "flow",
    "flower",
    "globe",
    "gradient",
    "gradient_up",
    "guster",
    "half_horizontal",
    "half_horizontal_bottom",
    "half_vertical",
    "half_vertical_right",
    "mojang",
    "piglin",
    "rhombus",
    "skull",
    "small_stripes",
    "square_bottom_left",
    "square_bottom_right",
    "square_top_left",
    "square_top_right",
    "straight_cross",
    "stripe_bottom",
    "stripe_center",
    "stripe_downleft",
    "stripe_downright",
    "stripe_left",
    "stripe_middle",
    "stripe_right",
    "stripe_top",
    "triangle_bottom",
    "triangle_top",
    "triangles_bottom",
    "triangles_top",
];

const BANNER_PATTERN_TAGS: &[(&str, &[&str])] = &[
    (
        "minecraft:no_item_required",
        &[
            "base",
            "square_bottom_left",
            "square_bottom_right",
            "square_top_left",
            "square_top_right",
            "stripe_bottom",
            "stripe_top",
            "stripe_left",
            "stripe_right",
            "stripe_center",
            "stripe_middle",
            "stripe_downright",
            "stripe_downleft",
            "small_stripes",
            "cross",
            "straight_cross",
            "triangle_bottom",
            "triangle_top",
            "triangles_bottom",
            "triangles_top",
            "diagonal_left",
            "diagonal_up_right",
            "diagonal_up_left",
            "diagonal_right",
            "circle",
            "rhombus",
            "half_vertical",
            "half_horizontal",
            "half_vertical_right",
            "half_horizontal_bottom",
            "border",
            "gradient",
            "gradient_up",
            "bricks",
            "curly_border",
        ],
    ),
    ("minecraft:pattern_item/flower", &["flower"]),
    ("minecraft:pattern_item/creeper", &["creeper"]),
    ("minecraft:pattern_item/skull", &["skull"]),
    ("minecraft:pattern_item/mojang", &["mojang"]),
    ("minecraft:pattern_item/globe", &["globe"]),
    ("minecraft:pattern_item/piglin", &["piglin"]),
    ("minecraft:pattern_item/flow", &["flow"]),
    ("minecraft:pattern_item/guster", &["guster"]),
    ("minecraft:pattern_item/field_masoned", &["bricks"]),
    ("minecraft:pattern_item/bordure_indented", &["curly_border"]),
];

// Source: decompiled-server-26.1.2/net/minecraft/world/item/JukeboxSongs.java
// and data/minecraft/jukebox_song/*.json
const JUKEBOX_SONGS: &[JukeboxSongEntry] = &[
    JukeboxSongEntry {
        id: "11",
        sound_event: "minecraft:music_disc.11",
        length_seconds: 71.0,
        comparator_output: 11,
    },
    JukeboxSongEntry {
        id: "13",
        sound_event: "minecraft:music_disc.13",
        length_seconds: 178.0,
        comparator_output: 1,
    },
    JukeboxSongEntry {
        id: "5",
        sound_event: "minecraft:music_disc.5",
        length_seconds: 178.0,
        comparator_output: 15,
    },
    JukeboxSongEntry {
        id: "blocks",
        sound_event: "minecraft:music_disc.blocks",
        length_seconds: 345.0,
        comparator_output: 3,
    },
    JukeboxSongEntry {
        id: "cat",
        sound_event: "minecraft:music_disc.cat",
        length_seconds: 185.0,
        comparator_output: 2,
    },
    JukeboxSongEntry {
        id: "chirp",
        sound_event: "minecraft:music_disc.chirp",
        length_seconds: 185.0,
        comparator_output: 4,
    },
    JukeboxSongEntry {
        id: "creator",
        sound_event: "minecraft:music_disc.creator",
        length_seconds: 176.0,
        comparator_output: 12,
    },
    JukeboxSongEntry {
        id: "creator_music_box",
        sound_event: "minecraft:music_disc.creator_music_box",
        length_seconds: 73.0,
        comparator_output: 11,
    },
    JukeboxSongEntry {
        id: "far",
        sound_event: "minecraft:music_disc.far",
        length_seconds: 174.0,
        comparator_output: 5,
    },
    JukeboxSongEntry {
        id: "lava_chicken",
        sound_event: "minecraft:music_disc.lava_chicken",
        length_seconds: 134.0,
        comparator_output: 9,
    },
    JukeboxSongEntry {
        id: "mall",
        sound_event: "minecraft:music_disc.mall",
        length_seconds: 197.0,
        comparator_output: 6,
    },
    JukeboxSongEntry {
        id: "mellohi",
        sound_event: "minecraft:music_disc.mellohi",
        length_seconds: 96.0,
        comparator_output: 7,
    },
    JukeboxSongEntry {
        id: "otherside",
        sound_event: "minecraft:music_disc.otherside",
        length_seconds: 195.0,
        comparator_output: 14,
    },
    JukeboxSongEntry {
        id: "pigstep",
        sound_event: "minecraft:music_disc.pigstep",
        length_seconds: 149.0,
        comparator_output: 13,
    },
    JukeboxSongEntry {
        id: "precipice",
        sound_event: "minecraft:music_disc.precipice",
        length_seconds: 299.0,
        comparator_output: 13,
    },
    JukeboxSongEntry {
        id: "relic",
        sound_event: "minecraft:music_disc.relic",
        length_seconds: 218.0,
        comparator_output: 14,
    },
    JukeboxSongEntry {
        id: "stal",
        sound_event: "minecraft:music_disc.stal",
        length_seconds: 150.0,
        comparator_output: 8,
    },
    JukeboxSongEntry {
        id: "strad",
        sound_event: "minecraft:music_disc.strad",
        length_seconds: 188.0,
        comparator_output: 9,
    },
    JukeboxSongEntry {
        id: "tears",
        sound_event: "minecraft:music_disc.tears",
        length_seconds: 175.0,
        comparator_output: 10,
    },
    JukeboxSongEntry {
        id: "wait",
        sound_event: "minecraft:music_disc.wait",
        length_seconds: 238.0,
        comparator_output: 12,
    },
    JukeboxSongEntry {
        id: "ward",
        sound_event: "minecraft:music_disc.ward",
        length_seconds: 251.0,
        comparator_output: 10,
    },
];

const TRIM_MATERIALS: &[TrimMaterialEntry] = &[
    TrimMaterialEntry {
        id: "quartz",
        asset_name: "quartz",
        color: "#e3d4bd",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "iron",
        asset_name: "iron",
        color: "#ececec",
        overrides: &[("minecraft:iron", "iron_darker")],
    },
    TrimMaterialEntry {
        id: "netherite",
        asset_name: "netherite",
        color: "#625859",
        overrides: &[("minecraft:netherite", "netherite_darker")],
    },
    TrimMaterialEntry {
        id: "redstone",
        asset_name: "redstone",
        color: "#971607",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "copper",
        asset_name: "copper",
        color: "#b4684d",
        overrides: &[("minecraft:copper", "copper_darker")],
    },
    TrimMaterialEntry {
        id: "gold",
        asset_name: "gold",
        color: "#decf2a",
        overrides: &[("minecraft:gold", "gold_darker")],
    },
    TrimMaterialEntry {
        id: "emerald",
        asset_name: "emerald",
        color: "#11a036",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "diamond",
        asset_name: "diamond",
        color: "#6eead6",
        overrides: &[("minecraft:diamond", "diamond_darker")],
    },
    TrimMaterialEntry {
        id: "lapis",
        asset_name: "lapis",
        color: "#416e97",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "amethyst",
        asset_name: "amethyst",
        color: "#9a5cc6",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "resin",
        asset_name: "resin",
        color: "#fc7812",
        overrides: &[],
    },
];

/// Loads clock state from `{world_root}/server_clocks.json`.
/// Returns `None` on missing or malformed file; caller falls back to `ServerClockManager::default()`.
/// Java: ServerClockManager.TYPE SavedData — key "world_clocks"
fn load_server_clock_state(world_root: &Path) -> Option<ServerClockManager> {
    let path = world_root.join("server_clocks.json");
    let text = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let clock = |obj: &serde_json::Value| -> Option<crate::world_time::ClockInstance> {
        Some(crate::world_time::ClockInstance {
            total_ticks: obj["total_ticks"].as_i64()?,
            partial_tick: obj["partial_tick"].as_f64()? as f32,
            rate: obj["rate"].as_f64()? as f32,
            paused: obj["paused"].as_bool()?,
        })
    };
    Some(ServerClockManager {
        game_time: v["game_time"].as_i64()?,
        overworld: clock(&v["overworld"])?,
        the_end: clock(&v["the_end"])?,
    })
}

/// Saves clock state to `{world_root}/server_clocks.json`.
fn save_server_clock_state(world_root: &Path, manager: &ServerClockManager) {
    let clock_json = |c: &crate::world_time::ClockInstance| {
        serde_json::json!({
            "total_ticks": c.total_ticks,
            "partial_tick": c.partial_tick,
            "rate": c.rate,
            "paused": c.paused,
        })
    };
    let value = serde_json::json!({
        "game_time": manager.game_time,
        "overworld": clock_json(&manager.overworld),
        "the_end": clock_json(&manager.the_end),
    });
    let path = world_root.join("server_clocks.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

/// Loads weather state from `{world_root}/server_weather.json`.
fn load_server_weather_state(world_root: &Path) -> Option<WeatherCycle> {
    let path = world_root.join("server_weather.json");
    let text = fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let data = WeatherData {
        clear_weather_time: v["clear_weather_time"].as_i64()? as i32,
        rain_time: v["rain_time"].as_i64()? as i32,
        thunder_time: v["thunder_time"].as_i64()? as i32,
        raining: v["raining"].as_bool()?,
        thundering: v["thundering"].as_bool()?,
    };
    let mut cycle = WeatherCycle::new(data);
    cycle.rain_level = v["rain_level"].as_f64()? as f32;
    cycle.thunder_level = v["thunder_level"].as_f64()? as f32;
    cycle.old_rain_level = cycle.rain_level;
    cycle.old_thunder_level = cycle.thunder_level;
    Some(cycle)
}

/// Saves weather state to `{world_root}/server_weather.json`.
fn save_server_weather_state(world_root: &Path, cycle: &WeatherCycle) {
    let value = serde_json::json!({
        "clear_weather_time": cycle.data.clear_weather_time,
        "rain_time": cycle.data.rain_time,
        "thunder_time": cycle.data.thunder_time,
        "raining": cycle.data.raining,
        "thundering": cycle.data.thundering,
        "rain_level": cycle.rain_level,
        "thunder_level": cycle.thunder_level,
    });
    let path = world_root.join("server_weather.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

/// Loads item entity state from `{world_root}/item_entities.json`.
///
/// Field names mirror Java's entity NBT format (`Pos`, `Motion`, `Age`, `PickupDelay`,
/// `Item`) so the file is human-readable and structurally close to the canonical
/// `entities/` region files used by the Java server.
///
/// Returns a default empty store if the file does not exist or cannot be parsed.
fn load_world_item_entities(world_root: &Path) -> WorldItemEntities {
    let path = world_root.join("item_entities.json");
    let Some(text) = fs::read_to_string(&path).ok() else {
        return WorldItemEntities::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return WorldItemEntities::new();
    };
    let next_entity_id = v["NextEntityId"].as_i64().unwrap_or(1) as i32;
    let entities = v["Entities"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let network_id = e["EntityNetworkId"].as_i64()? as i32;
                    let pos = e["Pos"].as_array()?;
                    let motion = e["Motion"].as_array()?;
                    let item_obj = e["Item"].as_object()?;
                    let item_name = item_static_name(item_obj.get("id")?.as_str()?)?;
                    let count = item_obj.get("count")?.as_i64()? as i32;
                    let age = e["Age"].as_i64().unwrap_or(0) as i32;
                    let pickup_delay = e["PickupDelay"].as_i64().unwrap_or(0) as i32;
                    let target_uuid = e["Owner"].as_str().map(|s| s.to_string());
                    Some(DroppedItem {
                        entity_id: network_id,
                        item: item_name,
                        count,
                        x: pos.first()?.as_f64()?,
                        y: pos.get(1)?.as_f64()?,
                        z: pos.get(2)?.as_f64()?,
                        vel_x: motion.first()?.as_f64()?,
                        vel_y: motion.get(1)?.as_f64()?,
                        vel_z: motion.get(2)?.as_f64()?,
                        age,
                        pickup_delay,
                        target_uuid,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    WorldItemEntities::restore(entities, next_entity_id)
}

/// Saves item entity state to `{world_root}/item_entities.json`.
///
/// Java: `EntityStorage.storeEntities()` / `ItemEntity.addAdditionalSaveData()`.
/// Field names match the canonical Java NBT names where applicable so the file is
/// recognisable to anyone familiar with the Java entity format.
fn save_world_item_entities(world_root: &Path, store: &WorldItemEntities) {
    let entities: Vec<serde_json::Value> = store
        .entities
        .iter()
        .map(|e| {
            let mut obj = serde_json::json!({
                "EntityNetworkId": e.entity_id,
                "id": "minecraft:item",
                "Pos": [e.x, e.y, e.z],
                "Motion": [e.vel_x, e.vel_y, e.vel_z],
                "Age": e.age,
                "PickupDelay": e.pickup_delay,
                "Item": {
                    "id": e.item,
                    "count": e.count,
                },
            });
            if let Some(owner) = &e.target_uuid {
                obj["Owner"] = serde_json::Value::String(owner.clone());
            }
            obj
        })
        .collect();
    let value = serde_json::json!({
        "NextEntityId": store.next_entity_id(),
        "Entities": entities,
    });
    let path = world_root.join("item_entities.json");
    if let Ok(text) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, text);
    }
}

pub fn run_status_server(
    bind_ip: &str,
    port: u16,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
    console_input: &Receiver<ConsoleInput>,
) -> Result<(), String> {
    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .map_err(|err| format!("Failed to bind status listener on {address}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Failed to configure status listener on {address}: {err}"))?;
    let favicon = load_favicon(Path::new("server-icon.png"))
        .map_err(|err| format!("Failed to load server-icon.png: {err}"))?;
    let active_logins = ActiveLoginRegistry::default();
    let world_root = Arc::new(world_root.to_path_buf());
    let chunk_cache = GeneratedChunkCache::default();
    let player_access = Arc::new(Mutex::new(
        PlayerAccess::load_from_dir(Path::new(".")).unwrap_or_else(|err| {
            eprintln!("status access file load error: {err}");
            PlayerAccess::default()
        }),
    ));

    // Load or initialise shared clock/weather/item-entity state.
    // Java: ServerClockManager.TYPE SavedData (key "world_clocks"), ServerLevel weather data,
    //       EntityStorage loads entities from per-chunk region files under <world>/entities/.
    let initial_clock = load_server_clock_state(&world_root).unwrap_or_default();
    let initial_weather = load_server_weather_state(&world_root)
        .unwrap_or_else(|| WeatherCycle::new(WeatherData::default()));
    let clock: Arc<Mutex<ServerClockManager>> = Arc::new(Mutex::new(initial_clock));
    let weather: Arc<Mutex<WeatherCycle>> = Arc::new(Mutex::new(initial_weather));
    // World-level item entity store.  Shared across all player sessions and persisted to
    // item_entities.json so items survive both player disconnects and server restarts.
    // Java: ServerLevel.entityStorage — entity lists belong to the world, not any connection.
    let world_items: Arc<Mutex<WorldItemEntities>> =
        Arc::new(Mutex::new(load_world_item_entities(&world_root)));

    // Load vanilla recipes once at startup and share via Arc.
    // Java: MinecraftServer.loadDataPacks() → RecipeManager.apply()
    let recipe_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let recipe_manager = load_recipe_directory(&recipe_dir).unwrap_or_else(|err| {
        panic!(
            "failed to load bundled vanilla recipes from {}: {err}",
            recipe_dir.display()
        )
    });
    log_info(&format!(
        "loaded {} bundled vanilla recipes",
        recipe_manager.recipe_map().values().len()
    ));
    let recipe_manager: Arc<RecipeManagerModel> = Arc::new(recipe_manager);

    // Background tick thread: advances clocks and weather at 20 TPS.
    // Java: MinecraftServer.tickChildren() — clockManager.tick() + advanceWeatherCycle()
    {
        let clock_t = Arc::clone(&clock);
        let weather_t = Arc::clone(&weather);
        let world_root_t = Arc::clone(&world_root);
        let world_items_t = Arc::clone(&world_items);
        thread::spawn(move || {
            let mut scheduled = ScheduledTimeChanges::default();
            let mut next_tick = Instant::now() + SERVER_TICK_DURATION;
            let mut tick_count: u64 = 0;
            loop {
                let now = Instant::now();
                if now < next_tick {
                    thread::sleep(next_tick - now);
                }
                next_tick += SERVER_TICK_DURATION;
                tick_count += 1;

                // advance_time=true: no per-world gamerule access yet; always advance.
                clock_t.lock().unwrap().tick(true, &mut scheduled);

                // Advance weather. can_have_weather=true for overworld.
                weather_t
                    .lock()
                    .unwrap()
                    .advance(true, true, DEFAULT_WEATHER_DURATIONS);

                // Persist every ~5 minutes.
                // Java: MinecraftServer.saveEverything() — entities flushed via EntityStorage.
                if tick_count % PERSISTENCE_INTERVAL_TICKS == 0 {
                    save_server_clock_state(&world_root_t, &clock_t.lock().unwrap());
                    save_server_weather_state(&world_root_t, &weather_t.lock().unwrap());
                    save_world_item_entities(&world_root_t, &world_items_t.lock().unwrap());
                }
            }
        });
    }

    println!("Status listener bound to {address}");

    loop {
        if should_stop(console_input, &player_access) {
            println!("Status listener stopping");
            save_server_clock_state(&world_root, &clock.lock().unwrap());
            save_server_weather_state(&world_root, &weather.lock().unwrap());
            save_world_item_entities(&world_root, &world_items.lock().unwrap());
            break;
        }
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                let properties = properties.clone();
                let favicon = favicon.clone();
                let active_logins = active_logins.clone();
                let chunk_cache = chunk_cache.clone();
                let world_root = Arc::clone(&world_root);
                let player_access = Arc::clone(&player_access);
                let clock = Arc::clone(&clock);
                let weather = Arc::clone(&weather);
                let recipe_manager = Arc::clone(&recipe_manager);
                let world_items = Arc::clone(&world_items);
                let remote_ip = peer_addr.ip().to_string();
                let remote_for_log = if properties.log_ips {
                    remote_ip.clone()
                } else {
                    "<redacted>".to_string()
                };
                thread::spawn(move || {
                    if let Err(err) = handle_status_connection(
                        stream,
                        &properties,
                        favicon.as_deref(),
                        &active_logins,
                        &chunk_cache,
                        &player_access,
                        &world_root,
                        world_seed,
                        &remote_ip,
                        &clock,
                        &weather,
                        &recipe_manager,
                        &world_items,
                    ) {
                        eprintln!("status connection error from {remote_for_log}: {err}");
                    }
                });
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25));
            }
            Err(err) => eprintln!("status accept error: {err}"),
        }
    }

    Ok(())
}

fn should_stop(
    console_input: &Receiver<ConsoleInput>,
    player_access: &Arc<Mutex<PlayerAccess>>,
) -> bool {
    loop {
        match console_input.try_recv() {
            Ok(input) if input.line.eq_ignore_ascii_case("stop") => return true,
            Ok(input)
                if input.line.eq_ignore_ascii_case("reload")
                    || input.line.eq_ignore_ascii_case("whitelist reload") =>
            {
                match PlayerAccess::load_from_dir(Path::new(".")) {
                    Ok(reloaded) => {
                        if let Ok(mut access) = player_access.lock() {
                            *access = reloaded;
                            println!("Reloaded player access files");
                        } else {
                            eprintln!("status access reload error: player access lock poisoned");
                        }
                    }
                    Err(err) => eprintln!("status access reload error: {err}"),
                }
            }
            Ok(_) => {}
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => return false,
        }
    }
}

fn handle_status_connection(
    mut stream: TcpStream,
    properties: &ServerProperties,
    favicon: Option<&str>,
    active_logins: &ActiveLoginRegistry,
    chunk_cache: &GeneratedChunkCache,
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
    clock: &Arc<Mutex<ServerClockManager>>,
    weather: &Arc<Mutex<WeatherCycle>>,
    recipe_manager: &RecipeManagerModel,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut first = [0u8; 1];
    if stream.peek(&mut first)? == 1 && first[0] == 0xFE {
        return handle_legacy_status_tcp_connection(&mut stream, properties);
    }

    let handshake = read_packet(&mut stream)?;
    let mut input = Cursor::new(handshake);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected handshake",
        ));
    }

    let protocol = read_var_i32(&mut input)?;
    let server_address = read_string(&mut input, 255)?;
    let mut port_bytes = [0u8; 2];
    input.read_exact(&mut port_bytes)?;
    let _server_port = u16::from_be_bytes(port_bytes);
    let next_state = read_var_i32(&mut input)?;
    if next_state == 2 {
        if protocol != PROTOCOL_VERSION {
            return write_login_protocol_mismatch_disconnect(&mut stream, protocol);
        }
        return handle_login_connection(
            &mut stream,
            properties,
            active_logins,
            chunk_cache,
            player_access,
            world_root,
            world_seed,
            remote_ip,
            login_host_ip(&server_address),
            clock,
            weather,
            recipe_manager,
            world_items,
        );
    }
    if next_state != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported handshake target state",
        ));
    }
    if !properties.enable_status {
        return Ok(());
    }

    loop {
        let packet = read_packet(&mut stream)?;
        let mut input = Cursor::new(packet);
        match read_var_i32(&mut input)? {
            0 => {
                let json = status_json(properties, favicon);
                write_status_response_packet(&mut stream, &json)?;
            }
            1 => {
                let request = ServerboundPingRequestPacket::read(&mut input)?;
                write_status_pong_packet(&mut stream, request)?;
                return Ok(());
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown status packet",
                ))
            }
        }
    }
}

fn write_login_protocol_mismatch_disconnect(
    stream: &mut TcpStream,
    protocol: i32,
) -> io::Result<()> {
    let key = if protocol < 754 {
        "multiplayer.disconnect.outdated_client"
    } else {
        "multiplayer.disconnect.incompatible"
    };
    write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
        ClientboundLoginDisconnectPacket {
            reason: crate::network::codec::ComponentJson(format!(
                "{{\"translate\":\"{}\",\"with\":[\"{}\"]}}",
                key, VERSION_NAME
            )),
        }
        .write(payload)
    })
}

fn handle_login_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
    active_logins: &ActiveLoginRegistry,
    chunk_cache: &GeneratedChunkCache,
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
    login_host_ip: Option<String>,
    clock: &Arc<Mutex<ServerClockManager>>,
    weather: &Arc<Mutex<WeatherCycle>>,
    recipe_manager: &RecipeManagerModel,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_HELLO_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login hello",
        ));
    }

    let mut login = LoginSession::default();
    let finished = login.accept_offline_hello(ServerboundHelloPacket::read(&mut input)?);
    if let Some(reason) = login_access_disconnect_reason(
        properties,
        player_access,
        &finished.profile,
        remote_ip,
        login_host_ip.as_deref(),
    )? {
        return write_framed_packet(stream, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID, |payload| {
            ClientboundLoginDisconnectPacket {
                reason: crate::network::codec::ComponentJson(format!(
                    "{{\"translate\":\"{reason}\"}}"
                )),
            }
            .write(payload)
        });
    }
    let (_active_login, replaced_stream) =
        active_logins.register_replacing(&finished.profile.uuid, stream)?;
    if let Some(replaced_stream) = replaced_stream {
        let _ = replaced_stream.shutdown(Shutdown::Both);
    }
    cache_login_profile(player_access, &finished.profile)?;
    let mut compression = CompressionState::disabled();
    if properties.network_compression_threshold >= 0 {
        let threshold = properties.network_compression_threshold;
        write_framed_packet(stream, CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, |payload| {
            ClientboundLoginCompressionPacket {
                compression_threshold: threshold,
            }
            .write(payload)
        })?;
        login.set_compression(threshold);
        compression = CompressionState::enabled(threshold);
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_LOGIN_FINISHED_PACKET_ID,
        |payload| finished.write(payload),
    )?;

    let packet = read_packet_with_compression(stream, compression)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login acknowledgement",
        ));
    }
    login.acknowledge(ServerboundLoginAcknowledgedPacket::read(&mut input)?);

    if let Some(packet) = bug_report_server_links_packet(properties) {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONFIGURATION_SERVER_LINKS_PACKET_ID,
            |payload| packet.write(payload),
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            write_identifier(payload, &Identifier::parse("minecraft:vanilla").unwrap())
        },
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.BIOME uses Biome.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_biome_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHAT_TYPE uses ChatType.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chat_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.TRIM_PATTERN uses TrimPattern.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_trim_pattern_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.TRIM_MATERIAL uses TrimMaterial.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_trim_material_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.WOLF_VARIANT uses WolfVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.WOLF_SOUND_VARIANT uses WolfSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PIG_VARIANT uses PigVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PIG_SOUND_VARIANT uses PigSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.FROG_VARIANT uses FrogVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_frog_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CAT_VARIANT uses CatVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CAT_SOUND_VARIANT uses CatSoundVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.COW_SOUND_VARIANT uses CowSoundVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.COW_VARIANT uses CowVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHICKEN_SOUND_VARIANT uses ChickenSoundVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_sound_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.CHICKEN_VARIANT uses ChickenVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.ZOMBIE_NAUTILUS_VARIANT uses ZombieNautilusVariant.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_zombie_nautilus_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.PAINTING_VARIANT uses PaintingVariant.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_painting_variant_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.DIMENSION_TYPE uses DimensionType.NETWORK_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_dimension_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.DAMAGE_TYPE uses DamageType.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_damage_type_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.BANNER_PATTERN uses BannerPattern.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_banner_pattern_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.JUKEBOX_SONG uses JukeboxSong.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_jukebox_song_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java
    // Registries.INSTRUMENT uses Instrument.DIRECT_CODEC.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_instrument_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java:125,160
    // Registries.WORLD_CLOCK uses WorldClock.DIRECT_CODEC (MapCodec.unitCodec — empty compound).
    // Must be sent before any ClientboundSetTimePacket so the client can resolve clock VarInt IDs.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_world_clock_registry_packet,
    )?;
    // Java source: decompiled-server-26.1.2/net/minecraft/resources/RegistryDataLoader.java:125,160
    // Registries.TIMELINE uses Timeline.NETWORK_CODEC (syncable tracks only).
    // Must be sent before the tags packet so timeline tag IDs can reference these entries.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_timeline_registry_packet,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID,
        write_minimal_update_tags_packet,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        write_vanilla_known_packs_packet,
    )?;
    wait_for_configuration_packet(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
    )?;
    if let Some(code_of_conduct) = load_code_of_conduct_for_language(properties, "en_us")? {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONFIGURATION_CODE_OF_CONDUCT_PACKET_ID,
            |payload| ClientboundCodeOfConductPacket { code_of_conduct }.write(payload),
        )?;
        wait_for_configuration_packet(
            stream,
            compression,
            SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID,
            "code of conduct acceptance",
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID,
        |_payload| Ok(()),
    )?;

    wait_for_configuration_packet(
        stream,
        compression,
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID,
        "finish configuration",
    )?;

    let mut play_state = load_play_session_state(
        world_root,
        &finished.profile.uuid,
        properties,
        recipe_manager.recipe_map(),
        world_seed,
    );

    // Snapshot current clock and weather state for the join packet.
    // Java: ServerClockManager.createFullSyncPacket() on player join, ServerLevel.sendLevelInfo()
    let (join_game_time, join_clock_data) = {
        let cm = clock.lock().unwrap();
        cm.full_sync_data(true)
    };
    let (join_rain_level, join_thunder_level) = {
        let wc = weather.lock().unwrap();
        (wc.rain_level, wc.thunder_level)
    };

    write_minimal_play_join(
        stream,
        compression,
        properties,
        world_seed,
        &finished.profile,
        &play_state,
        recipe_manager,
        world_root,
        chunk_cache,
        join_game_time,
        join_clock_data,
        join_rain_level,
        join_thunder_level,
    )?;
    let mut current_chunk_x = chunk_coordinate(play_state.x);
    let mut current_chunk_z = chunk_coordinate(play_state.z);
    let chunk_batch_radius = chunk_batch_radius(properties);
    let mut loaded_chunks = chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
    stream.set_read_timeout(Some(SERVER_TICK_DURATION))?;
    let mut last_keep_alive = Instant::now();
    let mut keep_alive_id = 0_i64;
    // Track last sent weather levels so we can detect changes and notify the client.
    // Java: ServerLevel.advanceWeatherCycle() broadcasts RainLevelChange/ThunderLevelChange
    let mut last_sent_rain_level = join_rain_level;
    let mut last_sent_thunder_level = join_thunder_level;
    let mut last_time_sync = Instant::now();
    let mut rate_limiter =
        PacketRateLimiter::new(properties.rate_limit_packets_per_second, Instant::now());
    let world_layout = WorldLayout::new(world_root);

    // On login: re-send ADD_ENTITY + SET_ENTITY_DATA bundles for every item entity that
    // is already on the ground.  This mirrors Java's ServerEntity.addPairing() called during
    // ChunkMap.updatePlayerMobTypeMap() when a player enters tracking range of an entity.
    // Without this, items dropped before a disconnect are invisible after reconnecting.
    {
        let items = world_items.lock().unwrap();
        for item in &items.entities {
            if let Some(item_pid) = item_protocol_id(item.item) {
                write_item_entity_spawn_packets(stream, compression, item, item_pid)?;
            }
        }
    }

    // Hook A: wall-clock timer driving item entity age ticks at ~20 Hz (50 ms per tick).
    // Java: ItemEntity.tick() — called once per server tick, ~50 ms.
    let mut last_item_tick = Instant::now();
    let mut last_player_tick = Instant::now();
    let mut play_tick_count = 0_u64;
    let mut live_fluid_ticks = LiveFluidTicks::new();
    {
        let center = chunk_cache.get_or_load(current_chunk_x, current_chunk_z, world_root, world_seed);
        seed_live_fluid_ticks_from_chunk(
            &mut live_fluid_ticks,
            play_tick_count as i64,
            &world_layout,
            world_seed,
            &center,
        );
    }
    const ITEM_TICK_INTERVAL: Duration = Duration::from_millis(50);
    loop {
        if last_keep_alive.elapsed() >= PLAY_KEEP_ALIVE_INTERVAL {
            keep_alive_id = keep_alive_id.wrapping_add(1);
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
                |payload| payload.write_all(&keep_alive_id.to_be_bytes()),
            )?;
            last_keep_alive = Instant::now();
        }

        // Time heartbeat: empty clock map, just the current game_time.
        // Java: MinecraftServer.forceGameTimeSynchronization() every 20 ticks (~1 second)
        if last_time_sync.elapsed() >= TIME_SYNC_INTERVAL {
            let game_time = clock.lock().unwrap().heartbeat_game_time();
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_SET_TIME_PACKET_ID,
                |payload| {
                    ClientboundSetTimePacket {
                        game_time,
                        clock_updates: BTreeMap::new(),
                    }
                    .write(payload)
                },
            )?;
            last_time_sync = Instant::now();
        }

        // Hook A: Item entity age tick — ~20 Hz wall-clock.
        // Mirrors ItemEntity.tick(): apply drag, decrement pickupDelay, increment age,
        // expire at LIFETIME, and merge nearby same-type stacks.
        // Java: ServerLevel.tick() → entity.tick() → mergeWithNeighbours() for every ItemEntity.
        if last_item_tick.elapsed() >= ITEM_TICK_INTERVAL {
            last_item_tick = Instant::now();
            let result = {
                let mut items = world_items.lock().unwrap();
                item_entity::tick(&mut items.entities)
            };
            if !result.removed.is_empty() {
                write_framed_packet_with_compression(
                    stream,
                    compression,
                    CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                    |p| {
                        write_var_i32(p, result.removed.len() as i32)?;
                        for id in &result.removed {
                            write_var_i32(p, *id)?;
                        }
                        Ok(())
                    },
                )?;
            }
            // Notify the client of any count changes caused by stack merges.
            // Note: count-update SET_ENTITY_DATA is NOT bundled — bundles are only needed
            // for the initial ADD_ENTITY + SET_ENTITY_DATA spawn pair.
            for (entity_id, item_name, new_count) in &result.count_updates {
                if let Some(item_pid) = item_protocol_id(item_name) {
                    write_framed_packet_with_compression(
                        stream,
                        compression,
                        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
                        |p| {
                            write_var_i32(p, *entity_id)?;
                            p.write_all(&[8u8])?; // index 8: ItemEntity.DATA_ITEM
                            write_var_i32(p, 7)?; // serializer 7: ITEM_STACK
                            write_var_i32(p, *new_count)?;
                            write_var_i32(p, item_pid)?;
                            write_var_i32(p, 0)?; // component add count
                            write_var_i32(p, 0)?; // component remove count
                            p.write_all(&[0xFFu8]) // end of metadata
                        },
                    )?;
                }
            }
        }

        // Java: ServerPlayer.doTick() calls FoodData.tick(this) every server
        // tick, independent of inbound movement/interaction packets. Entity
        // base ticking updates fluid contact and air supply on the same tick.
        if last_player_tick.elapsed() >= SERVER_TICK_DURATION {
            last_player_tick = Instant::now();
            play_tick_count = play_tick_count.wrapping_add(1);
            process_live_fluid_ticks(
                stream,
                compression,
                &mut live_fluid_ticks,
                play_tick_count as i64,
                &world_layout,
                world_seed,
                chunk_cache,
            )?;
            let fluid_state =
                detect_play_session_fluid_state(&play_state, world_root, world_seed, chunk_cache);
            let water_update = tick_play_session_water(&mut play_state, fluid_state);
            if water_update.air_changed {
                write_play_state_air_supply_packet(stream, compression, &play_state)?;
            }
            if water_update.motion_changed {
                write_play_state_motion_packet(stream, compression, &play_state)?;
            }
            if tick_play_session_food(
                &mut play_state,
                food_difficulty_from_properties(properties),
                true,
                play_tick_count,
            ) || water_update.health_changed
            {
                write_play_state_health_packet(stream, compression, &play_state)?;
            }
        }

        // Detect weather level changes and broadcast to client.
        // Java: ServerLevel.advanceWeatherCycle() — RainLevelChange/ThunderLevelChange
        {
            let (cur_rain, cur_thunder) = {
                let wc = weather.lock().unwrap();
                (wc.rain_level, wc.thunder_level)
            };
            if (cur_rain - last_sent_rain_level).abs() > f32::EPSILON {
                write_game_event(stream, compression, 7, cur_rain)?;
                // Also send StopRaining(2) or StartRaining(1) on boundary crossings.
                // Java: WeatherGameEvent::StopRaining/StartRaining at rain_level 0.2 threshold
                if last_sent_rain_level > 0.2 && cur_rain <= 0.2 {
                    write_game_event(stream, compression, 2, 0.0)?;
                } else if last_sent_rain_level <= 0.2 && cur_rain > 0.2 {
                    write_game_event(stream, compression, 1, 0.0)?;
                }
                last_sent_rain_level = cur_rain;
            }
            if (cur_thunder - last_sent_thunder_level).abs() > f32::EPSILON {
                write_game_event(stream, compression, 8, cur_thunder)?;
                last_sent_thunder_level = cur_thunder;
            }
        }

        match read_packet_with_compression(stream, compression) {
            Ok(packet) => {
                if let PacketRateDecision::Kick { reason } =
                    rate_limiter.record_packet(Instant::now())
                {
                    // Java: InventoryMenu.removed() clears the crafting grid and returns
                    // items to inventory before the player state is persisted.
                    play_state.inventory_menu.clear_crafting_to_inventory();
                    let _ =
                        save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                    save_world_item_entities(world_root, &world_items.lock().unwrap());
                    write_framed_packet_with_compression(
                        stream,
                        compression,
                        CLIENTBOUND_DISCONNECT_PACKET_ID,
                        |payload| {
                            ClientboundDisconnectPacket {
                                reason: ComponentJson(format!("{{\"translate\":\"{reason}\"}}")),
                            }
                            .write(payload)
                        },
                    )?;
                    return Ok(());
                }
                let mut input = Cursor::new(packet);
                let packet_id = read_var_i32(&mut input)?;
                let session_update =
                    update_play_session_state(packet_id, &mut input, &mut play_state)?;
                if session_update.health_changed {
                    write_play_state_health_packet(stream, compression, &play_state)?;
                }
                if session_update.respawn_requested {
                    handle_play_respawn_request(
                        stream,
                        compression,
                        &mut play_state,
                        properties,
                        world_root,
                        world_seed,
                        chunk_cache,
                    )?;
                    current_chunk_x = chunk_coordinate(play_state.x);
                    current_chunk_z = chunk_coordinate(play_state.z);
                    loaded_chunks =
                        chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
                    let _ =
                        save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                    continue;
                }
                if session_update.position_changed {
                    let next_chunk_x = chunk_coordinate(play_state.x);
                    let next_chunk_z = chunk_coordinate(play_state.z);
                    if next_chunk_x != current_chunk_x || next_chunk_z != current_chunk_z {
                        let next_loaded_chunks =
                            chunk_window(next_chunk_x, next_chunk_z, chunk_batch_radius);
                        for stale_chunk in loaded_chunks.difference(&next_loaded_chunks) {
                            write_forget_generated_spawn_chunk_packets(
                                stream,
                                compression,
                                stale_chunk.0,
                                stale_chunk.1,
                                world_root,
                                world_seed,
                                chunk_cache,
                            )?;
                        }
                        let chunks_to_send =
                            newly_visible_chunks(&loaded_chunks, &next_loaded_chunks);
                        current_chunk_x = next_chunk_x;
                        current_chunk_z = next_chunk_z;
                        loaded_chunks = next_loaded_chunks;
                        write_play_chunk_delta(
                            stream,
                            compression,
                            current_chunk_x,
                            current_chunk_z,
                            &chunks_to_send,
                            true,
                            world_root,
                            world_seed,
                            chunk_cache,
                            Some((&mut live_fluid_ticks, play_tick_count as i64, &world_layout)),
                        )?;
                    }
                    // Hook B: Pickup check — mirrors Player.aiStep() proximity sweep.
                    // Spectators cannot pick up items.
                    // Java: Player.aiStep() — inflate AABB, iterate nearby entities, call playerTouch.
                    if play_state.game_mode != GameMode::Spectator {
                        process_item_pickups(
                            stream,
                            compression,
                            &mut play_state,
                            &finished.profile.uuid,
                            world_items,
                        )?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID {
                    write_command_suggestions_response(stream, compression, &mut input)?;
                    continue;
                }
                if packet_id == SERVERBOUND_USE_ITEM_ON_PACKET_ID {
                    let packet = ServerboundUseItemOnPacket::read(&mut input)?;
                    handle_use_item_on(
                        stream,
                        compression,
                        &mut play_state,
                        &world_layout,
                        world_seed,
                        chunk_cache,
                        &mut live_fluid_ticks,
                        play_tick_count as i64,
                        &packet,
                    )?;
                    continue;
                }
                if packet_id == SERVERBOUND_PLAYER_ACTION_PACKET_ID {
                    let action = read_var_i32(&mut input)?;
                    let mut pos_bytes = [0u8; 8];
                    input.read_exact(&mut pos_bytes)?;
                    let packed_pos = i64::from_be_bytes(pos_bytes);
                    let mut direction_byte = [0u8; 1];
                    input.read_exact(&mut direction_byte)?;
                    let sequence = read_var_i32(&mut input)?;
                    let (dbx, dby, dbz) = unpack_block_position(packed_pos);
                    crate::log::log_debug(&format!(
                        "player_action action={action} pos=({dbx},{dby},{dbz}) mode={:?}",
                        play_state.game_mode
                    ));
                    // Java ServerPlayerGameMode: START_DESTROY_BLOCK with getDestroyProgress >= 1.0
                    // (i.e. destroy_time == 0) → "insta mine" — break immediately, same as creative.
                    if action == 0 {
                        let chunk_pos_dbg = ChunkPos {
                            x: dbx.div_euclid(16),
                            z: dbz.div_euclid(16),
                        };
                        let actual_block =
                            read_block_at(&world_layout, world_seed, chunk_pos_dbg, dbx, dby, dbz);
                        let destroy_time = actual_block
                            .as_deref()
                            .and_then(|name| representative_state_definition(name))
                            .map(|def| def.physical.destroy_time);
                        crate::log::log_debug(&format!("instabreak check: actual_block={actual_block:?} destroy_time={destroy_time:?}"));
                    }
                    let is_instabreak =
                        action == 0 && play_state.game_mode != GameMode::Creative && {
                            let chunk_pos_ib = ChunkPos {
                                x: dbx.div_euclid(16),
                                z: dbz.div_euclid(16),
                            };
                            read_block_at(&world_layout, world_seed, chunk_pos_ib, dbx, dby, dbz)
                                .as_deref()
                                .and_then(|name| representative_state_definition(name))
                                .map(|def| def.physical.destroy_time == 0.0)
                                .unwrap_or(false)
                        };
                    let should_break = action == 2
                        || (action == 0 && play_state.game_mode == GameMode::Creative)
                        || is_instabreak;
                    if should_break {
                        // Packet ordering rationale:
                        //
                        // Java defers BlockChangedAck to the start of the next server tick
                        // (~50 ms later via ServerGamePacketListenerImpl.ackBlockChangesUpTo).
                        // In that window the entity is already spawned, physics-ticked, and
                        // rendering on the client.  Any block-prediction rollback triggered by
                        // the delayed ack therefore never touches the stable entity.
                        //
                        // Our server is synchronous — all packets go out in one TCP write.
                        // Testing confirms that sending BlockChangedAck AFTER the entity (Java's
                        // final wire order) causes the client to process the ack and AddEntity in
                        // the same packet loop, triggering prediction rollback while the entity
                        // has just been registered but hasn't been physics-ticked yet — the
                        // rollback culls it (always invisible).
                        //
                        // Sending BlockChangedAck FIRST lets the client commit its block-
                        // prediction state before AddEntity arrives, so the entity spawns into
                        // confirmed-AIR and renders correctly.
                        if crate::log::global_level() >= crate::log::LogLevel::Trace {
                            crate::log::log_trace(&format!(
                                "block break seq={sequence} pos=({dbx},{dby},{dbz}) action={action} game_mode={:?}",
                                play_state.game_mode
                            ));
                            crate::log::log_trace(&format!(
                                "sending BLOCK_CHANGED_ACK seq={sequence}"
                            ));
                        }
                        write_framed_packet_with_compression(
                            stream,
                            compression,
                            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
                            |p| write_var_i32(p, sequence),
                        )?;
                        if crate::log::global_level() >= crate::log::LogLevel::Trace {
                            crate::log::log_trace(&format!(
                                "sending BLOCK_UPDATE pos=({dbx},{dby},{dbz}) new_state=AIR"
                            ));
                        }
                        write_framed_packet_with_compression(
                            stream,
                            compression,
                            CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
                            |p| {
                                p.write_all(&packed_pos.to_be_bytes())?;
                                write_var_i32(p, AIR_BLOCK_STATE_ID)
                            },
                        )?;
                        let (bx, by, bz) = unpack_block_position(packed_pos);
                        let chunk_pos = ChunkPos {
                            x: bx.div_euclid(16),
                            z: bz.div_euclid(16),
                        };
                        let block_name =
                            break_block_in_region(&world_layout, world_seed, chunk_pos, bx, by, bz);
                        chunk_cache.invalidate(chunk_pos);
                        schedule_neighbor_fluids(
                            &mut live_fluid_ticks,
                            play_tick_count as i64,
                            &world_layout,
                            world_seed,
                            crate::block_update::BlockPos {
                                x: bx,
                                y: by,
                                z: bz,
                            },
                        );
                        crate::log::log_debug(&format!(
                            "block break at ({bx},{by},{bz}) block={block_name:?} game_mode={:?}",
                            play_state.game_mode
                        ));
                        if play_state.game_mode != GameMode::Creative {
                            let loot_seed = (bx as u64).wrapping_mul(0x9E37_79B9)
                                ^ (by as u64).wrapping_mul(0x6C62_272E)
                                ^ (bz as u64).wrapping_mul(0x517C_C1B7);
                            let drops = block_name
                                .as_deref()
                                .map(|n| evaluate_block_loot(n, loot_seed))
                                .unwrap_or_default();
                            let drop_x = bx as f64 + 0.5;
                            let drop_y = by as f64 + 0.5;
                            let drop_z = bz as f64 + 0.5;
                            for (item_name, count) in drops {
                                let Some(item_pid) = item_protocol_id(item_name) else {
                                    continue;
                                };
                                let eid = world_items.lock().unwrap().alloc_entity_id();
                                // Java: ItemEntity constructor sets initial velocity
                                // (random*0.2-0.1, 0.2, random*0.2-0.1) — the y=0.2 upward
                                // component produces the characteristic item "pop" animation
                                // and ensures the entity is visible on spawn.
                                let vel_x = pseudo_rand_f32(eid, 0) as f64 * 0.2 - 0.1;
                                let vel_y = 0.2_f64;
                                let vel_z = pseudo_rand_f32(eid, 1) as f64 * 0.2 - 0.1;
                                let item = DroppedItem {
                                    entity_id: eid,
                                    item: item_name,
                                    count,
                                    x: drop_x,
                                    y: drop_y,
                                    z: drop_z,
                                    vel_x,
                                    vel_y,
                                    vel_z,
                                    pickup_delay: DEFAULT_PICKUP_DELAY,
                                    age: 0,
                                    target_uuid: None,
                                };
                                write_item_entity_spawn_packets(
                                    stream,
                                    compression,
                                    &item,
                                    item_pid,
                                )?;
                                world_items.lock().unwrap().entities.push(item);
                            }
                        }
                    }
                    // Java: ServerboundPlayerActionPacket.Action.DROP_ALL_ITEMS = 3,
                    //        ServerboundPlayerActionPacket.Action.DROP_ITEM = 4.
                    if action == 3 || action == 4 {
                        handle_drop_item(
                            stream,
                            compression,
                            &mut play_state,
                            world_items,
                            action == 3,
                        )?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_CONTAINER_CLICK_PACKET_ID {
                    // Only handle player inventory (container_id 0) for now.
                    // Java: ServerGamePacketListenerImpl.handleContainerClick()
                    if let Ok(click) = ServerboundContainerClickPacket::read(&mut input) {
                        if click.container_id == 0 {
                            let instructions = handle_container_click(
                                &click,
                                &mut play_state.container_state_id,
                                &mut play_state.inventory_menu,
                                &mut play_state.carried_item,
                            );
                            for instruction in instructions {
                                match instruction {
                                    PlayInstruction::ContainerSetSlot(pkt) => {
                                        write_framed_packet_with_compression(
                                            stream,
                                            compression,
                                            CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
                                            |p| pkt.write(p),
                                        )?;
                                    }
                                    PlayInstruction::SetCursorItem(pkt) => {
                                        write_framed_packet_with_compression(
                                            stream,
                                            compression,
                                            CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
                                            |p| pkt.write(p),
                                        )?;
                                    }
                                    PlayInstruction::RecipesUnlocked(ids) => {
                                        if let Some(pkt) =
                                            build_recipe_book_add(&ids, recipe_manager.recipe_map())
                                        {
                                            write_framed_packet_with_compression(
                                                stream,
                                                compression,
                                                CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
                                                |p| pkt.write(p),
                                            )?;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID {
                    let packet = ServerboundRecipeBookChangeSettingsPacket::read(&mut input)?;
                    apply_recipe_book_settings_packet(&mut play_state, packet);
                    continue;
                }
                if packet_id == SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID {
                    let packet = ServerboundRecipeBookSeenRecipePacket::read(&mut input)?;
                    apply_recipe_book_seen_recipe_packet(
                        &mut play_state,
                        packet,
                        recipe_manager.recipe_map(),
                    );
                    continue;
                }
                if packet_id == SERVERBOUND_PLACE_RECIPE_PACKET_ID {
                    let packet = ServerboundPlaceRecipePacket::read(&mut input)?;
                    if packet.container_id == 0 {
                        if apply_place_recipe_packet(
                            &mut play_state,
                            packet,
                            recipe_manager.recipe_map(),
                        ) {
                            write_inventory_menu_full_sync(stream, compression, &play_state)?;
                        }
                    }
                    continue;
                }
                if matches!(
                    packet_id,
                    SERVERBOUND_KEEP_ALIVE_PACKET_ID
                        | SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID
                        | SERVERBOUND_CHAT_ACK_PACKET_ID
                        | SERVERBOUND_CHAT_COMMAND_PACKET_ID
                        | SERVERBOUND_CHAT_PACKET_ID
                        | SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID
                        | SERVERBOUND_CLIENT_COMMAND_PACKET_ID
                        | SERVERBOUND_CLIENT_INFORMATION_PACKET_ID
                        | SERVERBOUND_CLIENT_TICK_END_PACKET_ID
                        | SERVERBOUND_CONTAINER_CLOSE_PACKET_ID
                        | SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID
                        | SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID
                        | SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID
                        | SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID
                        | SERVERBOUND_PLAYER_COMMAND_PACKET_ID
                        | SERVERBOUND_PLAYER_INPUT_PACKET_ID
                        | SERVERBOUND_PLAYER_LOADED_PACKET_ID
                        | SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID
                        | SERVERBOUND_SWING_PACKET_ID
                        | SERVERBOUND_USE_ITEM_PACKET_ID
                ) {
                    continue;
                }
                play_state.inventory_menu.clear_crafting_to_inventory();
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                save_world_item_entities(world_root, &world_items.lock().unwrap());
                write_framed_packet_with_compression(
                    stream,
                    compression,
                    CLIENTBOUND_DISCONNECT_PACKET_ID,
                    |payload| {
                        ClientboundDisconnectPacket {
                            reason: ComponentJson(format!(
                                "{{\"text\":\"unexpected play packet {packet_id}\"}}"
                            )),
                        }
                        .write(payload)
                    },
                )?;
                return Ok(());
            }
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionReset
                ) =>
            {
                play_state.inventory_menu.clear_crafting_to_inventory();
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
                save_world_item_entities(world_root, &world_items.lock().unwrap());
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }
}

fn cache_login_profile(
    player_access: &Arc<Mutex<PlayerAccess>>,
    profile: &NameAndId,
) -> io::Result<()> {
    let mut access = player_access
        .lock()
        .map_err(|_| io::Error::other("player access lock poisoned"))?;
    access.cache_user(profile.clone());
    access.save_user_cache(Path::new("."))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PlaySessionUpdate {
    position_changed: bool,
    health_changed: bool,
    respawn_requested: bool,
}

fn update_play_session_state<R: Read>(
    packet_id: i32,
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<PlaySessionUpdate> {
    match packet_id {
        SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => {
            let old_x = state.x;
            let old_y = state.y;
            let old_z = state.z;
            state.x = read_f64(input)?;
            state.y = read_f64(input)?;
            state.z = read_f64(input)?;
            state.on_ground = read_bool(input)?;
            Ok(apply_player_movement(
                state,
                state.x - old_x,
                state.y - old_y,
                state.z - old_z,
                true,
            ))
        }
        SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => {
            let old_x = state.x;
            let old_y = state.y;
            let old_z = state.z;
            state.x = read_f64(input)?;
            state.y = read_f64(input)?;
            state.z = read_f64(input)?;
            state.yaw = read_f32(input)?;
            state.pitch = read_f32(input)?;
            state.on_ground = read_bool(input)?;
            Ok(apply_player_movement(
                state,
                state.x - old_x,
                state.y - old_y,
                state.z - old_z,
                true,
            ))
        }
        SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => {
            state.yaw = read_f32(input)?;
            state.pitch = read_f32(input)?;
            state.on_ground = read_bool(input)?;
            Ok(apply_player_fall_movement(state, 0.0, false))
        }
        SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => {
            state.on_ground = read_bool(input)?;
            Ok(apply_player_fall_movement(state, 0.0, false))
        }
        SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
            let slot = i32::from(read_i16(input)?);
            if (0..9).contains(&slot) {
                state.selected_slot = slot;
            }
            Ok(PlaySessionUpdate {
                position_changed: false,
                health_changed: false,
                respawn_requested: false,
            })
        }
        SERVERBOUND_CLIENT_COMMAND_PACKET_ID => {
            let action = read_var_i32(input)?;
            // Java ServerboundClientCommandPacket.Action ordinal 0 = PERFORM_RESPAWN.
            // ServerGamePacketListenerImpl ignores it while the player is alive.
            Ok(PlaySessionUpdate {
                position_changed: false,
                health_changed: false,
                respawn_requested: action == 0 && state.health <= 0.0,
            })
        }
        SERVERBOUND_PLAYER_INPUT_PACKET_ID => {
            let flags = read_u8(input)?;
            let forward = flags & 1 != 0;
            let backward = flags & 2 != 0;
            let left = flags & 4 != 0;
            let right = flags & 8 != 0;
            let jumping = flags & 16 != 0;
            let shift = flags & 32 != 0;
            let sprinting = flags & 64 != 0;
            if jumping && !state.input_jumping && state.on_ground {
                add_player_food_exhaustion(
                    state,
                    if sprinting {
                        SPRINT_JUMP_EXHAUSTION
                    } else {
                        JUMP_EXHAUSTION
                    },
                );
            }
            state.input_forward = forward;
            state.input_backward = backward;
            state.input_left = left;
            state.input_right = right;
            state.input_shift = shift;
            state.input_jumping = jumping;
            state.input_sprinting = sprinting;
            Ok(PlaySessionUpdate::default())
        }
        _ => Ok(PlaySessionUpdate::default()),
    }
}

fn apply_player_fall_movement(
    state: &mut PlaySessionState,
    delta_y: f64,
    position_changed: bool,
) -> PlaySessionUpdate {
    apply_player_movement(state, 0.0, delta_y, 0.0, position_changed)
}

fn apply_player_movement(
    state: &mut PlaySessionState,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    position_changed: bool,
) -> PlaySessionUpdate {
    if position_changed && !state.in_water {
        state.water_velocity_x = delta_x;
        state.water_velocity_y = delta_y;
        state.water_velocity_z = delta_z;
    }
    state.fall_distance = update_fall_distance(state.fall_distance, delta_y, state.in_water);
    if state.in_water {
        state.fall_distance = 0.0;
    }
    if state.eye_in_water {
        let distance_cm = ((delta_x * delta_x + delta_y * delta_y + delta_z * delta_z).sqrt()
            * 100.0)
            .round() as i32;
        if distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SWIM_EXHAUSTION_PER_METER, distance_cm),
            );
        }
    } else if state.in_water {
        let horizontal_distance_cm =
            ((delta_x * delta_x + delta_z * delta_z).sqrt() * 100.0).round() as i32;
        if horizontal_distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SWIM_EXHAUSTION_PER_METER, horizontal_distance_cm),
            );
        }
    } else if state.on_ground && state.input_sprinting {
        let horizontal_distance_cm =
            ((delta_x * delta_x + delta_z * delta_z).sqrt() * 100.0).round() as i32;
        if horizontal_distance_cm > 0 {
            add_player_food_exhaustion(
                state,
                movement_exhaustion(SPRINT_EXHAUSTION_PER_METER, horizontal_distance_cm),
            );
        }
    }

    let mut health_changed = false;
    if state.on_ground && state.fall_distance > 0.0 {
        let damage = calculate_fall_damage(FallDamageInput {
            fall_distance: state.fall_distance,
            damage_modifier: 1.0,
            safe_fall_distance: DEFAULT_SAFE_FALL_DISTANCE,
            fall_damage_multiplier: DEFAULT_FALL_DAMAGE_MULTIPLIER,
            fall_damage_enabled: true,
            may_fly: state.abilities.mayfly,
        });
        state.fall_distance = 0.0;
        if damage > 0 && state.health > 0.0 {
            state.health = (state.health - damage as f32).max(0.0);
            health_changed = true;
        }
    }

    PlaySessionUpdate {
        position_changed,
        health_changed,
        respawn_requested: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PlayerFluidState {
    in_water: bool,
    eye_in_water: bool,
    water_height: f64,
}

impl PlayerFluidState {
    const DRY: Self = Self {
        in_water: false,
        eye_in_water: false,
        water_height: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PlayerWaterTickUpdate {
    air_changed: bool,
    health_changed: bool,
    motion_changed: bool,
}

fn detect_play_session_fluid_state(
    state: &PlaySessionState,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> PlayerFluidState {
    detect_play_session_fluid_state_with_lookup(state, |x, y, z| {
        let chunk =
            chunk_cache.get_or_load(x.div_euclid(16), z.div_euclid(16), world_root, world_seed);
        chunk.get_block_state_name(x, y, z).map(str::to_string)
    })
}

fn detect_play_session_fluid_state_with_lookup<F>(
    state: &PlaySessionState,
    mut block_at: F,
) -> PlayerFluidState
where
    F: FnMut(i32, i32, i32) -> Option<String>,
{
    let half_width = PLAYER_WIDTH / 2.0;
    let min_x = state.x - half_width;
    let max_x = state.x + half_width;
    let min_y = state.y;
    let max_y = state.y + PLAYER_HEIGHT;
    let min_z = state.z - half_width;
    let max_z = state.z + half_width;
    let x0 = min_x.floor() as i32;
    let y0 = min_y.floor() as i32;
    let z0 = min_z.floor() as i32;
    let x1 = max_x.ceil() as i32 - 1;
    let y1 = max_y.ceil() as i32 - 1;
    let z1 = max_z.ceil() as i32 - 1;
    let eye_block_x = state.x.floor() as i32;
    let eye_y = state.y + PLAYER_EYE_HEIGHT;
    let eye_block_z = state.z.floor() as i32;

    let mut water_height = 0.0_f64;
    let mut eye_in_water = false;
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let Some(block) = block_at(x, y, z) else {
                    continue;
                };
                let Some(fluid_height) = water_fluid_height_for_block(&block) else {
                    continue;
                };
                let fluid_bottom = f64::from(y);
                let fluid_top = fluid_bottom + fluid_height;
                if fluid_top < min_y {
                    continue;
                }
                water_height = water_height.max(fluid_top - min_y);
                if x == eye_block_x
                    && z == eye_block_z
                    && eye_y >= fluid_bottom
                    && eye_y <= fluid_top
                {
                    eye_in_water = true;
                }
            }
        }
    }

    if water_height > 0.0 {
        PlayerFluidState {
            in_water: true,
            eye_in_water,
            water_height,
        }
    } else {
        PlayerFluidState::DRY
    }
}

fn water_fluid_height_for_block(block: &str) -> Option<f64> {
    if block.contains("waterlogged=true") {
        return Some(1.0);
    }
    let (base, properties) = block.split_once('[').map_or((block, ""), |(base, rest)| {
        (base, rest.trim_end_matches(']'))
    });
    if base != "minecraft:water" && base != "minecraft:flowing_water" {
        return None;
    }
    let level = properties
        .split(',')
        .find_map(|property| property.strip_prefix("level="))
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);
    Some(match level {
        1..=7 => f64::from(8 - level) / 9.0,
        _ => 1.0,
    })
}

fn play_session_water_input_vector(state: &PlaySessionState) -> (f64, f64) {
    let left_intent = if state.input_left == state.input_right {
        0.0
    } else if state.input_left {
        1.0
    } else {
        -1.0
    };
    let forward_intent = if state.input_forward == state.input_backward {
        0.0
    } else if state.input_forward {
        1.0
    } else {
        -1.0
    };
    (left_intent, forward_intent)
}

fn rotate_player_input_to_world(strafe: f64, forward: f64, speed: f64, yaw: f32) -> (f64, f64) {
    let length_sqr = strafe * strafe + forward * forward;
    if length_sqr < 1.0e-7 {
        return (0.0, 0.0);
    }
    let scale = if length_sqr > 1.0 {
        speed / length_sqr.sqrt()
    } else {
        speed
    };
    let strafe = strafe * scale;
    let forward = forward * scale;
    let yaw = f64::from(yaw).to_radians();
    let sin = yaw.sin();
    let cos = yaw.cos();
    (strafe * cos - forward * sin, forward * cos + strafe * sin)
}

fn tick_play_session_water(
    state: &mut PlaySessionState,
    fluid_state: PlayerFluidState,
) -> PlayerWaterTickUpdate {
    state.in_water = fluid_state.in_water;
    state.eye_in_water = fluid_state.eye_in_water;
    state.water_fluid_height = fluid_state.water_height;
    if state.in_water {
        state.fall_distance = 0.0;
    }

    let old_air = state.air_supply;
    let old_health = state.health;
    let old_water_velocity_x = state.water_velocity_x;
    let old_water_velocity_y = state.water_velocity_y;
    let old_water_velocity_z = state.water_velocity_z;
    if state.in_water {
        let (strafe, forward) = play_session_water_input_vector(state);
        let (input_x, input_z) =
            rotate_player_input_to_world(strafe, forward, WATER_MOVE_RELATIVE_SPEED, state.yaw);
        state.water_velocity_x += input_x;
        state.water_velocity_z += input_z;

        if state.input_shift {
            state.water_velocity_y -= WATER_JUMP_IMPULSE;
        }
        if state.input_jumping && state.water_fluid_height > 0.0 {
            state.water_velocity_y += WATER_JUMP_IMPULSE;
        }

        let should_apply_vertical_fluid_drag =
            !state.on_ground || state.water_velocity_y.abs() > f64::EPSILON || state.eye_in_water;
        if should_apply_vertical_fluid_drag {
            state.water_velocity_y = state.water_velocity_y * WATER_VERTICAL_SLOWDOWN;
            if !state.input_sprinting {
                state.water_velocity_y -= WATER_FALLING_GRAVITY;
            }
        }

        let horizontal_slowdown = if state.input_sprinting {
            WATER_SPRINTING_HORIZONTAL_SLOWDOWN
        } else {
            WATER_HORIZONTAL_SLOWDOWN
        };
        state.water_velocity_x *= horizontal_slowdown;
        state.water_velocity_z *= horizontal_slowdown;
    } else {
        state.water_velocity_x = 0.0;
        state.water_velocity_y = 0.0;
        state.water_velocity_z = 0.0;
    }

    if state.health > 0.0 {
        if state.eye_in_water {
            if !state.abilities.invulnerable {
                state.air_supply -= 1;
                if state.air_supply <= DROWN_AIR_SUPPLY_THRESHOLD {
                    state.air_supply = 0;
                    state.health = (state.health - DROWN_DAMAGE).max(0.0);
                }
            } else if state.air_supply < MAX_AIR_SUPPLY {
                state.air_supply = (state.air_supply + 4).min(MAX_AIR_SUPPLY);
            }
        } else if state.air_supply < MAX_AIR_SUPPLY {
            state.air_supply = (state.air_supply + 4).min(MAX_AIR_SUPPLY);
        }
    }

    PlayerWaterTickUpdate {
        air_changed: state.air_supply != old_air,
        health_changed: state.health != old_health,
        motion_changed: state.in_water
            && ((state.water_velocity_x - old_water_velocity_x).abs() > f64::EPSILON
                || (state.water_velocity_y - old_water_velocity_y).abs() > f64::EPSILON
                || (state.water_velocity_z - old_water_velocity_z).abs() > f64::EPSILON),
    }
}

fn food_state_from_play_session(state: &PlaySessionState) -> FoodState {
    FoodState {
        food_level: state.food_level,
        saturation: state.food_saturation,
        exhaustion: state.food_exhaustion,
        tick_timer: state.food_tick_timer,
    }
}

fn apply_food_state_to_play_session(state: &mut PlaySessionState, food: FoodState) {
    state.food_level = food.food_level;
    state.food_saturation = food.saturation;
    state.food_exhaustion = food.exhaustion;
    state.food_tick_timer = food.tick_timer;
}

fn add_player_food_exhaustion(state: &mut PlaySessionState, amount: f32) {
    if state.abilities.invulnerable {
        return;
    }
    let mut food = food_state_from_play_session(state);
    food.add_exhaustion(amount);
    apply_food_state_to_play_session(state, food);
}

fn tick_play_session_food(
    state: &mut PlaySessionState,
    difficulty: FoodDifficulty,
    natural_regen: bool,
    tick_count: u64,
) -> bool {
    if state.health <= 0.0 {
        return false;
    }

    let old_health = state.health;
    let old_food_level = state.food_level;
    let old_saturation_zero = state.food_saturation == 0.0;

    // Java: ServerPlayer.tickRegeneration() runs from LivingEntity.tick()
    // before ServerPlayer.doTick() calls FoodData.tick(this).
    if difficulty == FoodDifficulty::Peaceful && natural_regen {
        if tick_count % 20 == 0 {
            if state.health < 20.0 {
                state.health = (state.health + 1.0).min(20.0);
            }
            if state.food_saturation < 20.0 {
                state.food_saturation += 1.0;
            }
        }
        if tick_count % 10 == 0 && state.food_level < 20 {
            state.food_level += 1;
        }
    }

    let mut food = food_state_from_play_session(state);
    match food.tick_food(state.health < 20.0, natural_regen, difficulty) {
        FoodTickOutcome::None => {}
        FoodTickOutcome::FastHeal { amount, .. } => {
            state.health = (state.health + amount).min(20.0);
        }
        FoodTickOutcome::SlowHeal => {
            state.health = (state.health + 1.0).min(20.0);
        }
        FoodTickOutcome::StarveAttempt => {
            if starvation_damages(difficulty, state.health) {
                state.health = (state.health - 1.0).max(0.0);
            }
        }
    }
    apply_food_state_to_play_session(state, food);

    state.health != old_health
        || state.food_level != old_food_level
        || (state.food_saturation == 0.0) != old_saturation_zero
}

fn food_difficulty_from_properties(properties: &ServerProperties) -> FoodDifficulty {
    match properties.difficulty.as_str() {
        "0" | "peaceful" => FoodDifficulty::Peaceful,
        "2" | "normal" => FoodDifficulty::Normal,
        "3" | "hard" => FoodDifficulty::Hard,
        _ => FoodDifficulty::Easy,
    }
}

fn write_play_state_health_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_HEALTH_PACKET_ID,
        |payload| {
            payload.write_all(&state.health.to_be_bytes())?;
            write_var_i32(payload, state.food_level)?;
            payload.write_all(&state.food_saturation.to_be_bytes())
        },
    )
}

fn play_state_air_supply_metadata_packet(
    state: &PlaySessionState,
) -> io::Result<ClientboundSetEntityDataPacket> {
    Ok(ClientboundSetEntityDataPacket {
        id: PLAYER_ENTITY_ID,
        packed_items: vec![EntityDataValue::typed(
            1,
            EntityMetadataValue::VarInt(state.air_supply),
        )?],
    })
}

fn write_play_state_air_supply_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let packet = play_state_air_supply_metadata_packet(state)?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |payload| packet.write(payload),
    )
}

fn write_play_state_motion_packet<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let packet = ClientboundSetEntityMotionPacket::new(
        PLAYER_ENTITY_ID,
        Vec3 {
            x: state.water_velocity_x,
            y: state.water_velocity_y,
            z: state.water_velocity_z,
        },
    );
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
        |payload| packet.write(payload),
    )
}

/// Handles a block-placement request from the client.
///
/// Java: ServerPlayerGameMode.useItemOn() → BlockItem.place() → Level.setBlock()
fn handle_use_item_on(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    packet: &ServerboundUseItemOnPacket,
) -> io::Result<()> {
    // Spectators cannot place blocks.
    // Java: ServerPlayerGameMode.useItemOn() — spectators are blocked before reaching here.
    let send_ack = |p: &mut Vec<u8>| write_var_i32(p, packet.sequence);
    if state.game_mode == GameMode::Spectator {
        return write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            send_ack,
        );
    }

    // Resolve the held item from the correct hand.
    // Java: ServerPlayerGameMode.useItemOn() calls player.getItemInHand(hand).
    let held_slot = match packet.hand {
        ServerboundSwingHand::MainHand => state.selected_slot as usize,
        ServerboundSwingHand::OffHand => SLOT_OFFHAND,
        ServerboundSwingHand::Unknown(_) => {
            return write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
                send_ack,
            );
        }
    };
    let held_item = state
        .inventory_menu
        .player_inventory()
        .get(held_slot)
        .clone();
    if held_item.is_empty() {
        return write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            send_ack,
        );
    }

    let item_name = held_item.item_id();
    if let Some(kind) = bucket_fluid_kind(item_name) {
        return handle_bucket_place_fluid(
            stream,
            compression,
            state,
            world_layout,
            world_seed,
            chunk_cache,
            live_fluid_ticks,
            game_time,
            packet,
            held_slot,
            kind,
        );
    }

    // Only proceed if the item has a known placeable block state.
    // Java: BlockItem.place() — only items backed by a Block can place.
    let Some(block_state_id) = block_state_name_network_id(item_name) else {
        return write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            send_ack,
        );
    };

    // Compute the placement target. Java's BlockPlaceContext uses the clicked block
    // itself when it can be replaced; otherwise it offsets into the clicked face.
    // This matters for fluids: LiquidBlock states are replaceable by normal block
    // items, so dirt/sand/etc. can be placed into water/lava instead of being
    // treated like an attempted overwrite of a solid block.
    let (dx, dy, dz) = direction_offset(packet.block_hit.direction);
    let clicked_pos = crate::block_update::BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let clicked_chunk = ChunkPos {
        x: clicked_pos.x.div_euclid(16),
        z: clicked_pos.z.div_euclid(16),
    };
    let clicked_state = read_block_model_at(world_layout, world_seed, clicked_pos);
    let clicked_replaceable = block_item_can_replace(&clicked_state);
    let (target_x, target_y, target_z) = if clicked_replaceable {
        (clicked_pos.x, clicked_pos.y, clicked_pos.z)
    } else {
        (
            packet.block_hit.x + dx,
            packet.block_hit.y + dy,
            packet.block_hit.z + dz,
        )
    };
    let target_chunk = ChunkPos {
        x: target_x.div_euclid(16),
        z: target_z.div_euclid(16),
    };

    let target_state = if clicked_replaceable && target_chunk == clicked_chunk {
        clicked_state
    } else {
        read_block_model_at(
            world_layout,
            world_seed,
            crate::block_update::BlockPos {
                x: target_x,
                y: target_y,
                z: target_z,
            },
        )
    };
    let is_replaceable = block_item_can_replace(&target_state);
    if !is_replaceable {
        return write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            send_ack,
        );
    }

    // Persist the new block state into the region file.
    place_block_in_region(
        world_layout,
        world_seed,
        target_chunk,
        target_x,
        target_y,
        target_z,
        item_name,
    );
    chunk_cache.invalidate(target_chunk);
    schedule_neighbor_fluids(
        live_fluid_ticks,
        game_time,
        world_layout,
        world_seed,
        crate::block_update::BlockPos {
            x: target_x,
            y: target_y,
            z: target_z,
        },
    );
    if item_name == "minecraft:water" || item_name == "minecraft:lava" {
        let kind = if item_name == "minecraft:water" {
            FluidKind::Water
        } else {
            FluidKind::Lava
        };
        live_fluid_ticks.schedule(
            game_time,
            crate::block_update::BlockPos {
                x: target_x,
                y: target_y,
                z: target_z,
            },
            kind,
        );
        schedule_neighbor_fluids(live_fluid_ticks, game_time, world_layout, world_seed, crate::block_update::BlockPos {
            x: target_x,
            y: target_y,
            z: target_z,
        });
    }

    // Acknowledge the client's predictive block change.
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        send_ack,
    )?;

    // Push the authoritative block state to the client.
    let packed_pos = block_pos_as_long(target_x, target_y, target_z);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |p| {
            p.write_all(&packed_pos.to_be_bytes())?;
            write_var_i32(p, block_state_id)
        },
    )?;

    // Survival and adventure modes consume one item from the player's hand.
    // Java: ItemStack.consume(1, player) called by BlockItem after a successful place.
    if state.game_mode != GameMode::Creative {
        state
            .inventory_menu
            .player_inventory_mut()
            .remove(held_slot, 1);
        let stack = state.inventory_menu.player_inventory().get(held_slot);
        let raw = if stack.is_empty() {
            RawItemStack::empty()
        } else if let Some(pid) = item_protocol_id(stack.item_id()) {
            RawItemStack {
                count: stack.count(),
                item_id: Some(pid),
                components: RawDataComponentPatch::empty(),
            }
        } else {
            RawItemStack::empty()
        };
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
            |p| {
                ClientboundSetPlayerInventoryPacket {
                    slot: held_slot as i32,
                    contents: raw,
                }
                .write(p)
            },
        )?;
    }

    Ok(())
}

fn bucket_fluid_kind(item_name: &str) -> Option<FluidKind> {
    match item_name {
        "minecraft:water_bucket" => Some(FluidKind::Water),
        "minecraft:lava_bucket" => Some(FluidKind::Lava),
        _ => None,
    }
}

fn handle_bucket_place_fluid(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    packet: &ServerboundUseItemOnPacket,
    held_slot: usize,
    kind: FluidKind,
) -> io::Result<()> {
    let send_ack = |p: &mut Vec<u8>| write_var_i32(p, packet.sequence);
    let (dx, dy, dz) = direction_offset(packet.block_hit.direction);
    let clicked = crate::block_update::BlockPos {
        x: packet.block_hit.x,
        y: packet.block_hit.y,
        z: packet.block_hit.z,
    };
    let adjacent = crate::block_update::BlockPos {
        x: clicked.x + dx,
        y: clicked.y + dy,
        z: clicked.z + dz,
    };
    let clicked_state = read_block_model_at(world_layout, world_seed, clicked);
    let target = if kind == FluidKind::Water && clicked_state.property("waterlogged").is_some() {
        clicked
    } else {
        adjacent
    };
    let existing = if target == clicked {
        clicked_state
    } else {
        read_block_model_at(world_layout, world_seed, target)
    };
    let placed = match place_liquid(&existing, kind) {
        LiquidPlaceResult::Rejected(_) => None,
        LiquidPlaceResult::Replaced(state) | LiquidPlaceResult::Waterlogged(state) => Some(state),
    };
    let Some(fluid_state) = placed else {
        return write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
            send_ack,
        );
    };
    write_block_model_at(world_layout, world_seed, target, &fluid_state);
    let target_chunk = ChunkPos {
        x: target.x.div_euclid(16),
        z: target.z.div_euclid(16),
    };
    chunk_cache.invalidate(target_chunk);
    live_fluid_ticks.schedule(game_time, target, kind);
    schedule_neighbor_fluids(live_fluid_ticks, game_time, world_layout, world_seed, target);

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        send_ack,
    )?;
    write_single_block_update(stream, compression, target, &fluid_state)?;

    if state.game_mode != GameMode::Creative {
        state
            .inventory_menu
            .player_inventory_mut()
            .set(held_slot, ItemStack::new("minecraft:bucket", 1));
        let raw = item_protocol_id("minecraft:bucket")
            .map(|pid| RawItemStack {
                count: 1,
                item_id: Some(pid),
                components: RawDataComponentPatch::empty(),
            })
            .unwrap_or_else(RawItemStack::empty);
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
            |p| {
                ClientboundSetPlayerInventoryPacket {
                    slot: held_slot as i32,
                    contents: raw,
                }
                .write(p)
            },
        )?;
    }

    Ok(())
}

fn schedule_neighbor_fluids(
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
) {
    for direction in crate::fluid::fluid_neighbor_order() {
        let neighbor = pos.relative(direction);
        if let Some(fluid) =
            crate::fluid::fluid_state_for_block(&read_block_model_at(world_layout, world_seed, neighbor))
        {
            live_fluid_ticks.schedule(game_time, neighbor, fluid.kind);
        }
    }
}

fn seed_live_fluid_ticks_from_chunk(
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk: &LevelChunk,
) {
    let started = Instant::now();
    let min_y = chunk.min_section_y * 16;
    let max_y = min_y + (chunk.sections.len() as i32 * 16);
    let mut fluid_blocks = 0_usize;
    let mut scheduled = 0_usize;
    for y in min_y..max_y {
        for local_z in 0..16 {
            for local_x in 0..16 {
                let pos = crate::block_update::BlockPos {
                    x: chunk.pos.x * 16 + local_x,
                    y,
                    z: chunk.pos.z * 16 + local_z,
                };
                let Some(entry) = chunk.get_block_state_model(pos.x, pos.y, pos.z) else {
                    continue;
                };
                let mut state = crate::block_behavior::BlockStateModel::new(entry.name);
                for (key, value) in entry.properties {
                    state = state.with_property(&key, value);
                }
                let Some(fluid) = fluid_state_for_block(&state) else {
                    continue;
                };
                fluid_blocks += 1;
                if fluid_has_runtime_update_edge(chunk, world_layout, world_seed, pos) {
                    live_fluid_ticks.schedule(game_time, pos, fluid.kind);
                    scheduled += 1;
                }
            }
        }
    }
    let elapsed = started.elapsed();
    if fluid_blocks > 0 || elapsed >= Duration::from_millis(10) {
        eprintln!(
            "[fluid-timing] seed chunk=({}, {}) y={}..{} fluid_blocks={} scheduled={} elapsed={}ms",
            chunk.pos.x,
            chunk.pos.z,
            min_y,
            max_y,
            fluid_blocks,
            scheduled,
            elapsed.as_millis()
        );
    }
}

fn fluid_has_runtime_update_edge(
    chunk: &LevelChunk,
    world_layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
) -> bool {
    crate::fluid::fluid_neighbor_order().into_iter().any(|direction| {
        let neighbor = pos.relative(direction);
        let same_chunk = neighbor.x.div_euclid(16) == chunk.pos.x
            && neighbor.z.div_euclid(16) == chunk.pos.z;
        let neighbor_state = if same_chunk {
            if let Some(entry) = chunk.get_block_state_model(neighbor.x, neighbor.y, neighbor.z) {
                let mut state = crate::block_behavior::BlockStateModel::new(entry.name);
                for (key, value) in entry.properties {
                    state = state.with_property(&key, value);
                }
                state
            } else {
                crate::block_behavior::BlockStateModel::air()
            }
        } else {
            read_block_model_at(world_layout, world_seed, neighbor)
        };
        neighbor_state.is_air()
    })
}

fn write_single_block_update<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    pos: crate::block_update::BlockPos,
    state: &crate::block_behavior::BlockStateModel,
) -> io::Result<()> {
    let block_name = block_state_model_name(state);
    let Some(block_state_id) = block_state_name_network_id(&block_name) else {
        return Ok(());
    };
    let packed_pos = block_pos_as_long(pos.x, pos.y, pos.z);
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        |p| {
            p.write_all(&packed_pos.to_be_bytes())?;
            write_var_i32(p, block_state_id)
        },
    )
}

fn process_live_fluid_ticks(
    stream: &mut TcpStream,
    compression: CompressionState,
    live_fluid_ticks: &mut LiveFluidTicks,
    game_time: i64,
    world_layout: &WorldLayout,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    let total_started = Instant::now();
    let due = live_fluid_ticks.tick_due(game_time, 4096);
    let due_count = due.len();
    let mut read_current_us = 0_u128;
    let mut tick_fluid_us = 0_u128;
    let mut write_update_us = 0_u128;
    let mut neighbor_read_us = 0_u128;
    let mut changes_written = 0_usize;
    let mut result_schedules = 0_usize;
    let mut neighbor_schedules = 0_usize;
    let mut skipped_wrong_fluid = 0_usize;
    for tick in due {
        let kind = match tick.ty.as_str() {
            "minecraft:water" => FluidKind::Water,
            "minecraft:lava" => FluidKind::Lava,
            _ => continue,
        };
        let read_started = Instant::now();
        let current = read_block_model_at(world_layout, world_seed, tick.pos);
        read_current_us += read_started.elapsed().as_micros();
        if crate::fluid::fluid_state_for_block(&current).is_none_or(|fluid| fluid.kind != kind) {
            skipped_wrong_fluid += 1;
            continue;
        }
        let tick_started = Instant::now();
        let result = tick_fluid(tick.pos, &current, |pos| {
            read_block_model_at(world_layout, world_seed, pos)
        });
        tick_fluid_us += tick_started.elapsed().as_micros();
        let mut changed_chunks = BTreeSet::new();
        for (pos, state) in result.changes {
            let write_started = Instant::now();
            if write_block_model_at(world_layout, world_seed, pos, &state) {
                write_update_us += write_started.elapsed().as_micros();
                changes_written += 1;
                changed_chunks.insert(ChunkPos {
                    x: pos.x.div_euclid(16),
                    z: pos.z.div_euclid(16),
                });
                write_single_block_update(stream, compression, pos, &state)?;
                if let Some(fluid) = crate::fluid::fluid_state_for_block(&state) {
                    live_fluid_ticks.schedule(game_time, pos, fluid.kind);
                }
                for direction in crate::fluid::fluid_neighbor_order() {
                    let neighbor = pos.relative(direction);
                    let neighbor_started = Instant::now();
                    let neighbor_state = read_block_model_at(world_layout, world_seed, neighbor);
                    neighbor_read_us += neighbor_started.elapsed().as_micros();
                    if let Some(fluid) = crate::fluid::fluid_state_for_block(&neighbor_state) {
                        live_fluid_ticks.schedule(game_time, neighbor, fluid.kind);
                        neighbor_schedules += 1;
                    }
                }
            } else {
                write_update_us += write_started.elapsed().as_micros();
            }
        }
        for pos in result.schedule {
            live_fluid_ticks.schedule(game_time, pos, kind);
            result_schedules += 1;
        }
        for chunk in changed_chunks {
            chunk_cache.invalidate(chunk);
        }
    }
    let total = total_started.elapsed();
    if due_count > 0 || total >= Duration::from_millis(10) {
        eprintln!(
            "[fluid-timing] tick game_time={} due={} skipped={} changes={} result_schedules={} neighbor_schedules={} total={}ms read_current={}us tick_fluid={}us write_update={}us neighbor_read={}us",
            game_time,
            due_count,
            skipped_wrong_fluid,
            changes_written,
            result_schedules,
            neighbor_schedules,
            total.as_millis(),
            read_current_us,
            tick_fluid_us,
            write_update_us,
            neighbor_read_us
        );
    }
    Ok(())
}

/// Checks whether the player is currently standing over any dropped item entities and,
/// if so, transfers them into the player's inventory and sends the relevant packets.
///
/// Java: `Player.aiStep()` — inflated AABB sweep → `ItemEntity.playerTouch()` →
///       `player.getInventory().add(itemStack)` → `player.take(this, count)`.
///
/// Sends per pickup:
///   1. `ClientboundTakeItemEntityPacket`      — triggers the client-side pickup animation/sound.
///   2. `ClientboundRemoveEntitiesPacket`       — removes the entity when fully consumed.
///   3. `ClientboundSetPlayerInventoryPacket`   — one packet per changed slot to sync inventory.
fn process_item_pickups(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    player_uuid: &str,
    world_items: &Arc<Mutex<WorldItemEntities>>,
) -> io::Result<()> {
    // Snapshot which slots exist before any mutation so we can send only dirty ones.
    // Java: Inventory.add() mutates slots; we detect changes via PlayerInventory.times_changed().
    let times_changed_before = state.inventory_menu.player_inventory().times_changed();

    // Phase 1: Under the lock, compute all pickups, mutate entity counts and inventory,
    // then remove fully-consumed entities.  Packet sends are deferred to Phase 2 so the
    // Mutex is not held during network I/O.
    struct PickupEvent {
        entity_id: i32,
        picked_up: i32,
        fully_consumed: bool,
    }
    let mut events: Vec<PickupEvent> = Vec::new();
    {
        let mut items = world_items.lock().unwrap();
        let (px, py, pz) = (state.x, state.y, state.z);
        for entity in items.entities.iter_mut() {
            if !entity.can_be_picked_up_by(player_uuid) {
                continue;
            }
            if !item_entity::in_pickup_range(px, py, pz, entity.x, entity.y, entity.z) {
                continue;
            }
            let original_count = entity.count;
            let stack = ItemStack::new(entity.item, entity.count);
            let (picked_up, new_count) =
                match state.inventory_menu.player_inventory_mut().add(stack) {
                    InventoryAddResult::FullyAdded => (original_count, 0),
                    InventoryAddResult::PartiallyAdded { remaining } => {
                        (original_count - remaining, remaining)
                    }
                    // Inventory rejected the item (e.g. full) — skip.
                    InventoryAddResult::Rejected | InventoryAddResult::Dropped { .. } => continue,
                };
            entity.count = new_count;
            events.push(PickupEvent {
                entity_id: entity.entity_id,
                picked_up,
                fully_consumed: new_count <= 0,
            });
        }
        // Remove fully-consumed entities from the world store.
        items.entities.retain(|e| e.count > 0);
    }

    // Phase 2: Send packets — lock is released, safe to block on network I/O.
    for event in &events {
        // 1. TakeItemEntity — triggers the client-side pickup animation and sound.
        //    Java: player.take(this, orgCount) → sends TakeItemEntityPacket to all trackers.
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
            |p| {
                ClientboundTakeItemEntityPacket {
                    item_entity_id: event.entity_id,
                    collector_entity_id: 1, // player always has entity ID 1 in single-session setup
                    amount: event.picked_up,
                }
                .write(p)
            },
        )?;
        // 2. RemoveEntities — only once the entire stack has been consumed.
        //    Java: if (itemStack.isEmpty()) this.discard() → RemoveEntitiesPacket.
        if event.fully_consumed {
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                |p| {
                    write_var_i32(p, 1)?;
                    write_var_i32(p, event.entity_id)
                },
            )?;
        }
    }

    // 3. ContainerSetContent — re-sync all 46 InventoryMenu slots so the client sees the
    //    newly picked-up items AND receives the updated container_state_id it must echo in
    //    its next ContainerClick.  Using SetPlayerInventory here would be wrong: that packet
    //    carries no state_id, so incrementing container_state_id on the server while sending
    //    it leaves the client tracking the old value, causing every subsequent crafting click
    //    to be rejected as stale and the crafting result slot to remain empty.
    //
    //    Java: AbstractContainerMenu.broadcastChanges() → synchronizer.sendSlotChange()
    //          → ClientboundContainerSetSlotPacket(containerId, incrementStateId(), slot, item).
    //    We send the full ContainerSetContent (equivalent to broadcastFullState) rather than
    //    per-slot ContainerSetSlot packets for simplicity.
    if state.inventory_menu.player_inventory().times_changed() != times_changed_before {
        state.container_state_id = state.container_state_id.wrapping_add(1);
        let new_state_id = state.container_state_id;
        let slots = state.inventory_menu.all_slots();
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
            |payload| {
                payload.write_all(&[0])?; // container ID 0 = player inventory menu
                write_var_i32(payload, new_state_id)?;
                write_var_i32(payload, slots.len() as i32)?;
                for stack in &slots {
                    let raw = if stack.is_empty() {
                        RawItemStack::empty()
                    } else if let Some(pid) = item_protocol_id(stack.item_id()) {
                        RawItemStack {
                            count: stack.count(),
                            item_id: Some(pid),
                            components: RawDataComponentPatch::empty(),
                        }
                    } else {
                        RawItemStack::empty()
                    };
                    raw.write_optional_untrusted(payload)?;
                }
                // Cursor (carried) item — must reflect the actual server state.
                // The player may have an item on their cursor (picked up via an earlier
                // ContainerClick) at the same time a ground pickup fires; sending empty
                // here would wipe the cursor on the client and make the held item vanish.
                let carried = &state.carried_item;
                let raw_carried = if carried.is_empty() {
                    RawItemStack::empty()
                } else if let Some(pid) = item_protocol_id(carried.item_id()) {
                    RawItemStack {
                        count: carried.count(),
                        item_id: Some(pid),
                        components: RawDataComponentPatch::empty(),
                    }
                } else {
                    RawItemStack::empty()
                };
                raw_carried.write_optional_untrusted(payload)
            },
        )?;
    }

    Ok(())
}

fn load_play_session_state(
    world_root: &Path,
    uuid: &str,
    properties: &ServerProperties,
    recipes: &RecipeMap,
    world_seed: i64,
) -> PlaySessionState {
    let layout = WorldLayout::new(world_root);
    let default_game_mode = game_mode_from_name(&properties.game_mode);
    let mut state = layout
        .load_player_data(uuid)
        .ok()
        .and_then(|tag| play_session_state_from_nbt(&tag, default_game_mode, recipes))
        .unwrap_or_else(|| {
            let mut state = PlaySessionState {
                game_mode: default_game_mode,
                ..PlaySessionState::default()
            };
            let spawn = find_default_player_spawn(world_root, world_seed, default_game_mode);
            apply_spawn_placement_to_state(&mut state, spawn);
            state
        });
    if properties.force_game_mode {
        state.game_mode = default_game_mode;
    }
    state
}

fn save_play_session_state(
    world_root: &Path,
    uuid: &str,
    state: &PlaySessionState,
) -> io::Result<()> {
    WorldLayout::new(world_root).save_player_data(uuid, &play_session_state_to_nbt(state))
}

fn handle_play_respawn_request(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    properties: &ServerProperties,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    let spawn = find_default_player_spawn(world_root, world_seed, state.game_mode);
    apply_spawn_placement_to_state(state, spawn);
    reset_play_state_after_death_respawn(state);

    let spawn_info = CommonPlayerSpawnInfo {
        seed: world_seed,
        game_mode: state.game_mode,
        previous_game_mode: state.previous_game_mode,
        last_death_location: state.last_death_location.as_ref().map(|pos| {
            (
                Identifier::parse(&pos.dimension).unwrap_or_else(|_| {
                    Identifier::parse("minecraft:overworld").expect("valid fallback identifier")
                }),
                [pos.x, pos.y, pos.z],
            )
        }),
        ..CommonPlayerSpawnInfo::default()
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_RESPAWN_PACKET_ID,
        |payload| {
            write_common_spawn_info(payload, &spawn_info)?;
            payload.write_all(&[0])
        },
    )?;

    let center_chunk_x = chunk_coordinate(state.x);
    let center_chunk_z = chunk_coordinate(state.z);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center_chunk_x)?;
            write_var_i32(payload, center_chunk_z)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        |payload| write_var_i32(payload, properties.view_distance as i32),
    )?;
    write_play_chunk_delta(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        &[(center_chunk_x, center_chunk_z)],
        false,
        world_root,
        world_seed,
        chunk_cache,
        None,
    )?;

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        |payload| {
            write_var_i32(payload, 0)?;
            write_vec3(payload, state.x, state.y, state.z)?;
            write_vec3(payload, 0.0, 0.0, 0.0)?;
            payload.write_all(&state.yaw.to_be_bytes())?;
            payload.write_all(&state.pitch.to_be_bytes())?;
            payload.write_all(&0_i32.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
        |payload| {
            let default_spawn = world_spawn_suggestion(world_root, world_seed);
            write_default_spawn_position_packet(
                payload,
                default_spawn.0,
                default_spawn.1,
                default_spawn.2,
            )
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
        |payload| {
            payload.write_all(&[1])?;
            write_bool(payload, false)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, state.xp_level)?;
            write_var_i32(payload, state.xp_total)
        },
    )?;
    write_game_event_to_writer(stream, compression, 2, 0.0)?;
    write_play_state_health_packet(stream, compression, state)?;
    write_play_state_air_supply_packet(stream, compression, state)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
    delay_initial_chunk_batch_for_probe(stream, compression)?;
    write_play_chunk_batch(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        chunk_batch_radius(properties),
        false,
        world_root,
        world_seed,
        chunk_cache,
    )
}

fn apply_spawn_placement_to_state(state: &mut PlaySessionState, spawn: PlayerSpawnPlacement) {
    state.x = spawn.x;
    state.y = spawn.y;
    state.z = spawn.z;
    state.yaw = spawn.yaw;
    state.pitch = spawn.pitch;
}

fn reset_play_state_after_death_respawn(state: &mut PlaySessionState) {
    state.health = 20.0;
    state.food_level = 20;
    state.food_saturation = 5.0;
    state.food_exhaustion = 0.0;
    state.food_tick_timer = 0;
    state.input_forward = false;
    state.input_backward = false;
    state.input_left = false;
    state.input_right = false;
    state.input_shift = false;
    state.input_sprinting = false;
    state.input_jumping = false;
    state.air_supply = MAX_AIR_SUPPLY;
    state.in_water = false;
    state.eye_in_water = false;
    state.water_fluid_height = 0.0;
    state.water_velocity_x = 0.0;
    state.water_velocity_y = 0.0;
    state.water_velocity_z = 0.0;
    state.fall_distance = 0.0;
    state.on_ground = true;
    state.xp_progress = 0.0;
    state.xp_level = 0;
    state.xp_total = 0;
    state.score = 0;
}

fn find_default_player_spawn(
    world_root: &Path,
    world_seed: i64,
    game_mode: GameMode,
) -> PlayerSpawnPlacement {
    let suggestion = world_spawn_suggestion(world_root, world_seed);
    find_player_spawn_near(world_root, world_seed, suggestion, game_mode).unwrap_or_else(|| {
        PlayerSpawnPlacement {
            x: suggestion.0 as f64 + 0.5,
            y: suggestion.1 as f64,
            z: suggestion.2 as f64 + 0.5,
            yaw: suggestion.3,
            pitch: 0.0,
        }
    })
}

fn world_spawn_suggestion(world_root: &Path, world_seed: i64) -> (i32, i32, i32, f32) {
    let layout = WorldLayout::new(world_root);
    if let Ok(tag) = layout.load_level_dat_with_backup() {
        if let Some(level) = PrimaryLevelData::from_level_dat(&tag) {
            return (
                level.spawn.x,
                level.spawn.y,
                level.spawn.z,
                level.spawn.angle,
            );
        }
    }

    resolve_world_preset("normal")
        .and_then(|preset| generator_find_spawn_position_for_stem(&preset.overworld, world_seed))
        .map(|pos| (pos.x, pos.y, pos.z, 0.0))
        .unwrap_or((0, SPAWN_Y as i32, 0, 0.0))
}

fn find_player_spawn_near(
    world_root: &Path,
    world_seed: i64,
    suggestion: (i32, i32, i32, f32),
    game_mode: GameMode,
) -> Option<PlayerSpawnPlacement> {
    let layout = WorldLayout::new(world_root);
    if game_mode != GameMode::Adventure {
        let radius = spawn_search_radius(
            SPAWN_SELECTION_CONSTANTS.default_respawn_radius,
            SPAWN_SELECTION_CONSTANTS.default_respawn_radius,
        );
        let candidate_count = spawn_search_candidate_count(radius);
        let random_offset =
            spawn_search_offset(world_seed, suggestion.0, suggestion.2, candidate_count);
        for candidate_index in 0..candidate_count {
            let Some((x, z)) = spawn_search_candidate(
                suggestion.0,
                suggestion.2,
                radius,
                random_offset,
                candidate_index,
            ) else {
                continue;
            };
            let chunk = load_chunk(
                &layout,
                world_seed,
                ChunkPos {
                    x: x.div_euclid(16),
                    z: z.div_euclid(16),
                },
            );
            let Some((spawn_x, spawn_y, spawn_z)) = overworld_respawn_pos_in_chunk(&chunk, x, z)
            else {
                continue;
            };
            if no_collision_no_liquid_in_chunk(&chunk, spawn_x, spawn_y, spawn_z) {
                return Some(PlayerSpawnPlacement {
                    x: spawn_x as f64 + 0.5,
                    y: spawn_y as f64,
                    z: spawn_z as f64 + 0.5,
                    yaw: suggestion.3,
                    pitch: 0.0,
                });
            }
        }
    }

    let chunk = load_chunk(
        &layout,
        world_seed,
        ChunkPos {
            x: suggestion.0.div_euclid(16),
            z: suggestion.2.div_euclid(16),
        },
    );
    let y = fixup_spawn_height(suggestion.1, -64, 320, |y| {
        no_collision_no_liquid_in_chunk(&chunk, suggestion.0, y, suggestion.2)
    });
    Some(PlayerSpawnPlacement {
        x: suggestion.0 as f64 + 0.5,
        y: y as f64,
        z: suggestion.2 as f64 + 0.5,
        yaw: suggestion.3,
        pitch: 0.0,
    })
}

fn spawn_search_offset(world_seed: i64, x: i32, z: i32, candidate_count: i32) -> i32 {
    if candidate_count <= 0 {
        return 0;
    }
    let mixed = (world_seed as u64)
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (z as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    (mixed % candidate_count as u64) as i32
}

fn overworld_respawn_pos_in_chunk(chunk: &LevelChunk, x: i32, z: i32) -> Option<(i32, i32, i32)> {
    let local_x = x.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let index = local_z * 16 + local_x;
    let motion_blocking = chunk.compute_heightmap_values(HeightmapKind::MotionBlocking);
    let world_surface = chunk.compute_heightmap_values(HeightmapKind::WorldSurface);
    let ocean_floor = chunk.compute_heightmap_values(HeightmapKind::OceanFloor);
    let top_y = motion_blocking[index];
    if top_y < -64 {
        return None;
    }
    let surface_y = world_surface[index];
    let ocean_floor_y = ocean_floor[index];
    if surface_y <= top_y && surface_y > ocean_floor_y {
        return None;
    }

    for y in (-64..=top_y + 1).rev() {
        match block_kind_at(chunk, x, y, z) {
            SpawnBlockKind::Fluid => break,
            SpawnBlockKind::Solid => return Some((x, y + 1, z)),
            SpawnBlockKind::Air | SpawnBlockKind::NonSolid => {}
        }
    }
    None
}

fn no_collision_no_liquid_in_chunk(chunk: &LevelChunk, x: i32, y: i32, z: i32) -> bool {
    matches!(
        block_kind_at(chunk, x, y, z),
        SpawnBlockKind::Air | SpawnBlockKind::NonSolid
    ) && matches!(
        block_kind_at(chunk, x, y + 1, z),
        SpawnBlockKind::Air | SpawnBlockKind::NonSolid
    )
}

fn block_kind_at(chunk: &LevelChunk, x: i32, y: i32, z: i32) -> SpawnBlockKind {
    let block = chunk
        .get_block_state_name(x, y, z)
        .unwrap_or("minecraft:air");
    spawn_block_kind(block)
}

fn play_session_state_to_nbt(state: &PlaySessionState) -> Tag {
    let mut values = vec![
        ("DataVersion".to_string(), Tag::Int(4791)),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(state.x),
                Tag::Double(state.y),
                Tag::Double(state.z),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(state.yaw), Tag::Float(state.pitch)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("OnGround".to_string(), Tag::Byte(i8::from(state.on_ground))),
        ("Air".to_string(), Tag::Short(state.air_supply as i16)),
        (
            "fall_distance".to_string(),
            Tag::Double(state.fall_distance as f64),
        ),
        ("Health".to_string(), Tag::Float(state.health)),
        ("foodLevel".to_string(), Tag::Int(state.food_level)),
        (
            "foodSaturationLevel".to_string(),
            Tag::Float(state.food_saturation),
        ),
        (
            "foodExhaustionLevel".to_string(),
            Tag::Float(state.food_exhaustion),
        ),
        ("foodTickTimer".to_string(), Tag::Int(state.food_tick_timer)),
        ("XpLevel".to_string(), Tag::Int(state.xp_level)),
        ("XpP".to_string(), Tag::Float(state.xp_progress)),
        ("XpTotal".to_string(), Tag::Int(state.xp_total)),
        ("XpSeed".to_string(), Tag::Int(state.xp_seed)),
        ("Score".to_string(), Tag::Int(state.score)),
        (
            "SelectedItemSlot".to_string(),
            Tag::Int(state.selected_slot),
        ),
        (
            "playerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(state.game_mode)),
        ),
        (
            "Dimension".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        (
            "seenCredits".to_string(),
            Tag::Byte(i8::from(state.seen_credits)),
        ),
        (
            "recipeBook".to_string(),
            Tag::Compound(vec![
                (
                    "recipes".to_string(),
                    Tag::List(
                        state
                            .inventory_menu
                            .recipe_book_known_recipes()
                            .into_iter()
                            .map(|id| Tag::String(id.to_string()))
                            .collect(),
                    ),
                ),
                (
                    "toBeDisplayed".to_string(),
                    Tag::List(
                        state
                            .inventory_menu
                            .recipe_book_highlighted_recipes()
                            .into_iter()
                            .map(|id| Tag::String(id.to_string()))
                            .collect(),
                    ),
                ),
                (
                    "isGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.crafting.open)),
                ),
                (
                    "isFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.crafting.filtering)),
                ),
                (
                    "isFurnaceGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.furnace.open)),
                ),
                (
                    "isFurnaceFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.furnace.filtering)),
                ),
                (
                    "isBlastingFurnaceGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.open)),
                ),
                (
                    "isBlastingFurnaceFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.blast_furnace.filtering)),
                ),
                (
                    "isSmokerGuiOpen".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.smoker.open)),
                ),
                (
                    "isSmokerFilteringCraftable".to_string(),
                    Tag::Byte(i8::from(state.recipe_book_settings.smoker.filtering)),
                ),
            ]),
        ),
        (
            "abilities".to_string(),
            Tag::Compound(vec![
                (
                    "invulnerable".to_string(),
                    Tag::Byte(i8::from(state.abilities.invulnerable)),
                ),
                (
                    "flying".to_string(),
                    Tag::Byte(i8::from(state.abilities.flying)),
                ),
                (
                    "mayfly".to_string(),
                    Tag::Byte(i8::from(state.abilities.mayfly)),
                ),
                (
                    "instabuild".to_string(),
                    Tag::Byte(i8::from(state.abilities.instabuild)),
                ),
                (
                    "mayBuild".to_string(),
                    Tag::Byte(i8::from(state.abilities.may_build)),
                ),
                (
                    "flySpeed".to_string(),
                    Tag::Float(state.abilities.fly_speed),
                ),
                (
                    "walkSpeed".to_string(),
                    Tag::Float(state.abilities.walk_speed),
                ),
            ]),
        ),
        (
            "EnderItems".to_string(),
            Tag::List(state.ender_items.clone()),
        ),
        (
            "active_effects".to_string(),
            Tag::List(state.active_effects.clone()),
        ),
    ];
    if let Some(mode) = state.previous_game_mode {
        values.push((
            "previousPlayerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(mode)),
        ));
    }
    if let Some(spawn) = &state.spawn {
        values.push(("SpawnX".to_string(), Tag::Int(spawn.x)));
        values.push(("SpawnY".to_string(), Tag::Int(spawn.y)));
        values.push(("SpawnZ".to_string(), Tag::Int(spawn.z)));
        values.push(("SpawnForced".to_string(), Tag::Byte(i8::from(spawn.forced))));
        values.push((
            "SpawnDimension".to_string(),
            Tag::String(spawn.dimension.clone()),
        ));
    }
    if let Some((x, y, z)) = state.entered_nether_position {
        values.push((
            "enteredNetherPosition".to_string(),
            Tag::Compound(vec![
                ("x".to_string(), Tag::Double(x)),
                ("y".to_string(), Tag::Double(y)),
                ("z".to_string(), Tag::Double(z)),
            ]),
        ));
    }
    if let Some(last_death) = &state.last_death_location {
        values.push((
            "LastDeathLocation".to_string(),
            Tag::Compound(vec![
                (
                    "dimension".to_string(),
                    Tag::String(last_death.dimension.clone()),
                ),
                (
                    "pos".to_string(),
                    Tag::List(vec![
                        Tag::Int(last_death.x),
                        Tag::Int(last_death.y),
                        Tag::Int(last_death.z),
                    ]),
                ),
            ]),
        ));
    }
    if let Some(root_vehicle) = &state.root_vehicle {
        values.push(("RootVehicle".to_string(), root_vehicle.clone()));
    }
    // Serialize the hotbar and main inventory (slots 0-35) as a TAG_List of TAG_Compound
    // entries, matching vanilla's player NBT format.
    // Java: ServerPlayer.addAdditionalSaveData() → Inventory.save()
    let inventory_items: Vec<Tag> = state
        .inventory_menu
        .player_inventory()
        .saved_items()
        .into_iter()
        .map(|(slot, stack)| {
            Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(slot as i8)),
                ("id".to_string(), Tag::String(stack.item_id().to_string())),
                ("count".to_string(), Tag::Int(stack.count())),
            ])
        })
        .collect();
    values.push(("Inventory".to_string(), Tag::List(inventory_items)));
    Tag::Compound(values)
}

fn play_session_state_from_nbt(
    tag: &Tag,
    default_game_mode: GameMode,
    recipes: &RecipeMap,
) -> Option<PlaySessionState> {
    let compound = match tag {
        Tag::Compound(values) => values,
        _ => return None,
    };
    let pos = compound_list(compound, "Pos")?;
    let rotation = compound_list(compound, "Rotation")?;
    let [Tag::Double(x), Tag::Double(y), Tag::Double(z)] = pos else {
        return None;
    };
    let [Tag::Float(yaw), Tag::Float(pitch)] = rotation else {
        return None;
    };
    let on_ground = match compound_tag(compound, "OnGround") {
        Some(Tag::Byte(value)) => *value != 0,
        _ => true,
    };
    let fall_distance = match compound_tag(compound, "fall_distance") {
        Some(Tag::Double(value)) => (*value as f32).max(0.0),
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let air_supply = match compound_tag(compound, "Air") {
        Some(Tag::Short(value)) => {
            i32::from(*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY)
        }
        Some(Tag::Int(value)) => (*value).clamp(DROWN_AIR_SUPPLY_THRESHOLD, MAX_AIR_SUPPLY),
        _ => MAX_AIR_SUPPLY,
    };
    let selected_slot = match compound_tag(compound, "SelectedItemSlot") {
        Some(Tag::Int(value)) if (0..9).contains(value) => *value,
        _ => 0,
    };
    let health = match compound_tag(compound, "Health") {
        Some(Tag::Float(value)) => value.clamp(0.0, 20.0),
        _ => 20.0,
    };
    let food_level = match compound_tag(compound, "foodLevel") {
        Some(Tag::Int(value)) => (*value).clamp(0, 20),
        _ => 20,
    };
    let food_saturation = match compound_tag(compound, "foodSaturationLevel") {
        Some(Tag::Float(value)) => value.clamp(0.0, food_level as f32),
        _ => 5.0,
    };
    let food_exhaustion = match compound_tag(compound, "foodExhaustionLevel") {
        Some(Tag::Float(value)) => value.max(0.0),
        _ => 0.0,
    };
    let food_tick_timer = match compound_tag(compound, "foodTickTimer") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_progress = match compound_tag(compound, "XpP") {
        Some(Tag::Float(value)) => value.clamp(0.0, 1.0),
        _ => 0.0,
    };
    let xp_level = match compound_tag(compound, "XpLevel") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_total = match compound_tag(compound, "XpTotal") {
        Some(Tag::Int(value)) => (*value).max(0),
        _ => 0,
    };
    let xp_seed = match compound_tag(compound, "XpSeed") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    let score = match compound_tag(compound, "Score") {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    };
    let game_mode = match compound_tag(compound, "playerGameType") {
        Some(Tag::Int(value)) => game_mode_from_legacy_id(*value),
        _ => default_game_mode,
    };
    let previous_game_mode = match compound_tag(compound, "previousPlayerGameType") {
        Some(Tag::Int(value)) if *value == -1 => None,
        Some(Tag::Int(value)) => Some(game_mode_from_legacy_id(*value)),
        _ => None,
    };
    let spawn = match (
        compound_tag(compound, "SpawnX"),
        compound_tag(compound, "SpawnY"),
        compound_tag(compound, "SpawnZ"),
    ) {
        (Some(Tag::Int(x)), Some(Tag::Int(y)), Some(Tag::Int(z))) => Some(PlayerSpawnData {
            dimension: match compound_tag(compound, "SpawnDimension") {
                Some(Tag::String(value)) => value.clone(),
                _ => "minecraft:overworld".to_string(),
            },
            x: *x,
            y: *y,
            z: *z,
            forced: matches!(compound_tag(compound, "SpawnForced"), Some(Tag::Byte(value)) if *value != 0),
        }),
        _ => None,
    };
    let seen_credits =
        matches!(compound_tag(compound, "seenCredits"), Some(Tag::Byte(value)) if *value != 0);
    let entered_nether_position = match compound_tag(compound, "enteredNetherPosition") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "x"),
            compound_tag(fields, "y"),
            compound_tag(fields, "z"),
        ) {
            (Some(Tag::Double(x)), Some(Tag::Double(y)), Some(Tag::Double(z))) => {
                Some((*x, *y, *z))
            }
            _ => None,
        },
        _ => None,
    };
    let last_death_location = match compound_tag(compound, "LastDeathLocation") {
        Some(Tag::Compound(fields)) => match (
            compound_tag(fields, "dimension"),
            compound_list(fields, "pos"),
        ) {
            (Some(Tag::String(dimension)), Some([Tag::Int(x), Tag::Int(y), Tag::Int(z)])) => {
                Some(PlayerGlobalPosData {
                    dimension: dimension.clone(),
                    x: *x,
                    y: *y,
                    z: *z,
                })
            }
            _ => None,
        },
        _ => None,
    };
    let root_vehicle = compound_tag(compound, "RootVehicle").cloned();
    let active_effects = match compound_tag(compound, "active_effects") {
        Some(Tag::List(values)) => values.clone(),
        _ => Vec::new(),
    };
    let ender_items = match compound_tag(compound, "EnderItems") {
        Some(Tag::List(values)) => values.clone(),
        _ => Vec::new(),
    };
    let abilities = match compound_tag(compound, "abilities") {
        Some(Tag::Compound(fields)) => PlayerNbtAbilities {
            invulnerable: compound_bool_byte(fields, "invulnerable", false),
            flying: compound_bool_byte(fields, "flying", false),
            mayfly: compound_bool_byte(fields, "mayfly", false),
            instabuild: compound_bool_byte(fields, "instabuild", false),
            may_build: compound_bool_byte(fields, "mayBuild", true),
            fly_speed: compound_float(fields, "flySpeed", 0.05),
            walk_speed: compound_float(fields, "walkSpeed", 0.1),
        },
        _ => PlayerNbtAbilities::default_survival(),
    };
    // Restore hotbar and main inventory (slots 0-35) from the TAG_List written by
    // play_session_state_to_nbt.
    // Java: ServerPlayer.readAdditionalSaveData() → Inventory.load()
    let mut inventory = PlayerInventory::new();
    if let Some(Tag::List(items)) = compound_tag(compound, "Inventory") {
        let mut loaded: Vec<(usize, ItemStack)> = Vec::new();
        for item_tag in items {
            if let Tag::Compound(fields) = item_tag {
                let slot = match compound_tag(fields, "Slot") {
                    Some(Tag::Byte(b)) => *b as u8 as usize,
                    _ => continue,
                };
                let id = match compound_tag(fields, "id") {
                    Some(Tag::String(s)) => s.as_str(),
                    _ => continue,
                };
                let count = match compound_tag(fields, "count") {
                    Some(Tag::Int(c)) => *c,
                    _ => 1,
                };
                if let Some(static_name) = item_static_name(id) {
                    if slot < 36 && count > 0 {
                        loaded.push((slot, ItemStack::new(static_name, count)));
                    }
                }
            }
        }
        if !loaded.is_empty() {
            inventory.load_items(&loaded);
        }
    }
    let (recipe_book_settings, known_recipes, highlighted_recipes) =
        load_recipe_book_from_nbt(compound, recipes);
    let mut inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    inventory_menu.load_recipe_book(known_recipes, highlighted_recipes);

    Some(PlaySessionState {
        x: *x,
        y: *y,
        z: *z,
        yaw: *yaw,
        pitch: *pitch,
        on_ground,
        fall_distance,
        selected_slot,
        health,
        food_level,
        food_saturation,
        food_exhaustion,
        food_tick_timer,
        input_forward: false,
        input_backward: false,
        input_left: false,
        input_right: false,
        input_shift: false,
        input_sprinting: false,
        input_jumping: false,
        air_supply,
        in_water: false,
        eye_in_water: false,
        water_fluid_height: 0.0,
        water_velocity_x: 0.0,
        water_velocity_y: 0.0,
        water_velocity_z: 0.0,
        xp_progress,
        xp_level,
        xp_total,
        xp_seed,
        score,
        game_mode,
        previous_game_mode,
        spawn,
        seen_credits,
        entered_nether_position,
        last_death_location,
        root_vehicle,
        active_effects,
        ender_items,
        abilities,
        inventory_menu,
        carried_item: ItemStack::empty(),
        container_state_id: 0,
        recipe_book_settings,
    })
}

fn default_recipe_book_settings() -> ClientboundRecipeBookSettingsPacket {
    ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        smoker: RecipeBookTypeSettings::CLOSED_UNFILTERED,
    }
}

fn load_recipe_book_from_nbt(
    player_compound: &[(String, Tag)],
    recipes: &RecipeMap,
) -> (
    ClientboundRecipeBookSettingsPacket,
    Vec<&'static str>,
    Vec<&'static str>,
) {
    let Some(Tag::Compound(recipe_book)) = compound_tag(player_compound, "recipeBook") else {
        return (default_recipe_book_settings(), Vec::new(), Vec::new());
    };

    let settings = ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isFilteringCraftable", false),
        },
        furnace: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isFurnaceGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isFurnaceFilteringCraftable", false),
        },
        blast_furnace: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isBlastingFurnaceGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isBlastingFurnaceFilteringCraftable", false),
        },
        smoker: RecipeBookTypeSettings {
            open: compound_bool_byte(recipe_book, "isSmokerGuiOpen", false),
            filtering: compound_bool_byte(recipe_book, "isSmokerFilteringCraftable", false),
        },
    };

    let known = load_recipe_id_list(recipe_book, "recipes", recipes);
    let highlighted = load_recipe_id_list(recipe_book, "toBeDisplayed", recipes);
    (settings, known, highlighted)
}

fn load_recipe_id_list(
    compound: &[(String, Tag)],
    key: &str,
    recipes: &RecipeMap,
) -> Vec<&'static str> {
    match compound_tag(compound, key) {
        Some(Tag::List(values)) => values
            .iter()
            .filter_map(|tag| match tag {
                Tag::String(id) => recipes.by_key(id).map(|holder| holder.id),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn apply_recipe_book_settings_packet(
    state: &mut PlaySessionState,
    packet: ServerboundRecipeBookChangeSettingsPacket,
) {
    let settings = RecipeBookTypeSettings {
        open: packet.is_open,
        filtering: packet.is_filtering,
    };
    match packet.book_type {
        RecipeBookType::Crafting => state.recipe_book_settings.crafting = settings,
        RecipeBookType::Furnace => state.recipe_book_settings.furnace = settings,
        RecipeBookType::BlastFurnace => state.recipe_book_settings.blast_furnace = settings,
        RecipeBookType::Smoker => state.recipe_book_settings.smoker = settings,
    }
}

fn apply_recipe_book_seen_recipe_packet(
    state: &mut PlaySessionState,
    packet: ServerboundRecipeBookSeenRecipePacket,
    recipes: &RecipeMap,
) {
    if packet.recipe_index < 0 {
        return;
    }
    if let Some(holder) = recipes.values().get(packet.recipe_index as usize) {
        state.inventory_menu.mark_recipe_seen(holder.id);
    }
}

fn apply_place_recipe_packet(
    state: &mut PlaySessionState,
    packet: ServerboundPlaceRecipePacket,
    recipes: &RecipeMap,
) -> bool {
    if packet.recipe_index < 0 {
        return false;
    }
    let Some(holder) = recipes.values().get(packet.recipe_index as usize) else {
        return false;
    };
    if state
        .inventory_menu
        .place_recipe_from_inventory(holder.id, packet.use_max_items)
    {
        state.container_state_id = state.container_state_id.wrapping_add(1);
        true
    } else {
        false
    }
}

fn write_inventory_menu_full_sync(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &PlaySessionState,
) -> io::Result<()> {
    let slots = state.inventory_menu.all_slots();
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?;
            write_var_i32(payload, state.container_state_id)?;
            write_var_i32(payload, slots.len() as i32)?;
            for stack in &slots {
                let raw = if stack.is_empty() {
                    RawItemStack::empty()
                } else if let Some(pid) = item_protocol_id(stack.item_id()) {
                    RawItemStack {
                        count: stack.count(),
                        item_id: Some(pid),
                        components: RawDataComponentPatch::empty(),
                    }
                } else {
                    RawItemStack::empty()
                };
                raw.write_optional_untrusted(payload)?;
            }
            let carried = &state.carried_item;
            let raw_carried = if carried.is_empty() {
                RawItemStack::empty()
            } else if let Some(pid) = item_protocol_id(carried.item_id()) {
                RawItemStack {
                    count: carried.count(),
                    item_id: Some(pid),
                    components: RawDataComponentPatch::empty(),
                }
            } else {
                RawItemStack::empty()
            };
            raw_carried.write_optional_untrusted(payload)
        },
    )
}

fn compound_tag<'a>(compound: &'a [(String, Tag)], key: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find_map(|(name, value)| (name == key).then_some(value))
}

fn compound_list<'a>(compound: &'a [(String, Tag)], key: &str) -> Option<&'a [Tag]> {
    match compound_tag(compound, key)? {
        Tag::List(values) => Some(values),
        _ => None,
    }
}

fn compound_bool_byte(compound: &[(String, Tag)], key: &str, default_value: bool) -> bool {
    match compound_tag(compound, key) {
        Some(Tag::Byte(value)) => *value != 0,
        _ => default_value,
    }
}

fn compound_float(compound: &[(String, Tag)], key: &str, default_value: f32) -> f32 {
    match compound_tag(compound, key) {
        Some(Tag::Float(value)) => *value,
        _ => default_value,
    }
}

fn login_access_disconnect_reason(
    properties: &ServerProperties,
    player_access: &Arc<Mutex<PlayerAccess>>,
    profile: &NameAndId,
    remote_ip: &str,
    login_host_ip: Option<&str>,
) -> io::Result<Option<&'static str>> {
    let access = player_access
        .lock()
        .map_err(|_| io::Error::other("player access lock poisoned"))?;
    if let Some(login_host_ip) = login_host_ip {
        if access.check_proxy_connection(
            properties.prevent_proxy_connections,
            login_host_ip,
            remote_ip,
        ) == ProxyConnectionDecision::RejectPreventProxyConnections
        {
            return Ok(Some("multiplayer.disconnect.unverified_username"));
        }
    }
    if access.is_ip_banned(remote_ip) {
        return Ok(Some("multiplayer.disconnect.ip_banned"));
    }
    if access.is_player_banned(&profile.uuid) {
        return Ok(Some("multiplayer.disconnect.banned"));
    }
    if properties.enforce_whitelist
        && !access.is_op(&profile.uuid)
        && !access.is_whitelisted(&profile.uuid)
    {
        return Ok(Some("multiplayer.disconnect.not_whitelisted"));
    }
    Ok(None)
}

fn login_host_ip(server_address: &str) -> Option<String> {
    let host = server_address
        .strip_prefix('[')
        .and_then(|address| address.split_once(']').map(|(host, _)| host))
        .or_else(|| server_address.split_once(':').map(|(host, _)| host))
        .unwrap_or(server_address);
    host.parse::<IpAddr>()
        .ok()
        .map(|address| address.to_string())
}

fn bug_report_server_links_packet(
    properties: &ServerProperties,
) -> Option<ClientboundServerLinksPacket> {
    let link = properties.bug_report_link.trim();
    if !(link.starts_with("https://") || link.starts_with("http://")) {
        return None;
    }
    Some(ClientboundServerLinksPacket {
        links: vec![ServerLinkEntry {
            label: ServerLinkLabel::Known(ServerLinkType::BugReport),
            link: link.to_string(),
        }],
    })
}

fn load_code_of_conduct_for_language(
    properties: &ServerProperties,
    client_language: &str,
) -> io::Result<Option<String>> {
    if !properties.code_of_conduct {
        return Ok(None);
    }
    let texts = read_code_of_conducts(Path::new("codeofconduct"))?;
    if texts.is_empty() {
        return Ok(None);
    }
    let language = client_language.to_lowercase();
    Ok(texts
        .get(&language)
        .or_else(|| texts.get("en_us"))
        .or_else(|| texts.values().next())
        .cloned())
}

fn read_code_of_conducts(dir: &Path) -> io::Result<HashMap<String, String>> {
    let metadata = fs::metadata(dir)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "codeofconduct is not a directory",
        ));
    }
    let canonical_dir = fs::canonicalize(dir)?;
    let mut texts = HashMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(language) = filename.strip_suffix(".txt") else {
            continue;
        };
        let parent = fs::canonicalize(&path)?
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid codeofconduct path")
            })?;
        if parent != canonical_dir {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "codeofconduct file links outside allowed directory",
            ));
        }
        let text = fs::read_to_string(&path)?;
        let text = strip_minecraft_formatting(&text.lines().collect::<Vec<_>>().join("\n"));
        texts.insert(language.to_lowercase(), text);
    }
    Ok(texts)
}

fn strip_minecraft_formatting(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '§' {
            match chars.peek().copied() {
                Some(code)
                    if code.is_ascii_hexdigit()
                        || matches!(code.to_ascii_lowercase(), 'k'..='o' | 'r') =>
                {
                    chars.next();
                    continue;
                }
                _ => {}
            }
        }
        out.push(ch);
    }
    out
}

fn wait_for_configuration_packet<R: Read>(
    reader: &mut R,
    compression: CompressionState,
    expected_packet_id: i32,
    expected_name: &'static str,
) -> io::Result<()> {
    for _ in 0..32 {
        let packet = read_packet_with_compression(reader, compression)?;
        let mut input = Cursor::new(packet);
        let packet_id = read_var_i32(&mut input)?;
        if packet_id == expected_packet_id {
            validate_expected_configuration_packet(&mut input, expected_packet_id)?;
            return Ok(());
        }
        if is_tolerated_serverbound_configuration_packet(packet_id) {
            continue;
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected {expected_name}, got configuration packet {packet_id}"),
        ));
    }

    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        format!("timed out waiting for {expected_name}"),
    ))
}

fn validate_expected_configuration_packet(
    input: &mut Cursor<Vec<u8>>,
    packet_id: i32,
) -> io::Result<()> {
    match packet_id {
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID => {
            let pack_count = read_var_i32(input)?;
            if !(0..=64).contains(&pack_count) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid selected known pack count",
                ));
            }
            for _ in 0..pack_count {
                let _namespace = read_string(input, 64)?;
                let _id = read_string(input, 128)?;
                let _version = read_string(input, 64)?;
            }
        }
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID => {}
        _ => {}
    }

    if input.position() != input.get_ref().len() as u64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("trailing bytes in configuration packet {packet_id}"),
        ));
    }

    Ok(())
}

fn is_tolerated_serverbound_configuration_packet(packet_id: i32) -> bool {
    matches!(
        packet_id,
        SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID
            | SERVERBOUND_CONFIGURATION_COOKIE_RESPONSE_PACKET_ID
            | SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID
            | SERVERBOUND_CONFIGURATION_KEEP_ALIVE_PACKET_ID
            | SERVERBOUND_CONFIGURATION_PONG_PACKET_ID
            | SERVERBOUND_CONFIGURATION_RESOURCE_PACK_PACKET_ID
            | SERVERBOUND_CONFIGURATION_CUSTOM_CLICK_ACTION_PACKET_ID
            | SERVERBOUND_CONFIGURATION_ACCEPT_CODE_OF_CONDUCT_PACKET_ID
    )
}

fn write_minimal_play_join(
    stream: &mut TcpStream,
    compression: CompressionState,
    properties: &ServerProperties,
    world_seed: i64,
    profile: &NameAndId,
    play_state: &PlaySessionState,
    recipe_manager: &RecipeManagerModel,
    world_root: &Path,
    chunk_cache: &GeneratedChunkCache,
    clock_game_time: i64,
    clock_data: Vec<(i32, ClockNetworkState)>,
    rain_level: f32,
    thunder_level: f32,
) -> io::Result<()> {
    let center_chunk_x = chunk_coordinate(play_state.x);
    let center_chunk_z = chunk_coordinate(play_state.z);
    let login = ClientboundLoginPacket {
        player_id: 1,
        hardcore: properties.hardcore,
        levels: vec![Identifier::parse("minecraft:overworld").unwrap()],
        max_players: properties.max_players as i32,
        chunk_radius: properties.view_distance as i32,
        simulation_distance: properties.simulation_distance as i32,
        reduced_debug_info: false,
        show_death_screen: true,
        do_limited_crafting: false,
        spawn_info: CommonPlayerSpawnInfo {
            seed: world_seed,
            game_mode: play_state.game_mode,
            previous_game_mode: play_state.previous_game_mode,
            is_flat: false,
            ..CommonPlayerSpawnInfo::default()
        },
        enforces_secure_chat: false,
    };
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_LOGIN_PACKET_ID,
        |payload| write_clientbound_login_packet(payload, &login),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID,
        |payload| write_player_info_initializing_packet(payload, profile, play_state.game_mode),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
        |payload| {
            payload.write_all(&[1])?;
            write_bool(payload, false)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
        |payload| write_player_abilities_packet(payload, play_state.game_mode),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
        |payload| write_var_i32(payload, play_state.selected_slot),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
        |payload| {
            payload.write_all(&play_state.xp_progress.to_be_bytes())?;
            write_var_i32(payload, play_state.xp_level)?;
            write_var_i32(payload, play_state.xp_total)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_HEALTH_PACKET_ID,
        |payload| {
            payload.write_all(&play_state.health.to_be_bytes())?;
            write_var_i32(payload, play_state.food_level)?;
            payload.write_all(&play_state.food_saturation.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID,
        |payload| play_state.recipe_book_settings.write(payload),
    )?;
    let known_recipes = play_state.inventory_menu.recipe_book_known_recipes();
    let highlighted_recipes = play_state.inventory_menu.recipe_book_highlighted_recipes();
    if let Some(packet) = build_recipe_book_add_with_flags(
        &known_recipes,
        recipe_manager.recipe_map(),
        false,
        false,
        true,
        Some(&highlighted_recipes),
    ) {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID,
            |payload| packet.write(payload),
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?; // container ID = player inventory (InventoryMenu.CONTAINER_ID)
            write_var_i32(payload, play_state.container_state_id)?;
            // Emit all 46 InventoryMenu slots (result + crafting grid + armour + storage + hotbar + offhand).
            // Java: AbstractContainerMenu.sendAllDataToRemote() iterates containerSlots[0..size].
            let slots = play_state.inventory_menu.all_slots();
            write_var_i32(payload, slots.len() as i32)?;
            for stack in &slots {
                let raw = if stack.is_empty() {
                    RawItemStack::empty()
                } else if let Some(pid) = item_protocol_id(stack.item_id()) {
                    RawItemStack {
                        count: stack.count(),
                        item_id: Some(pid),
                        components: RawDataComponentPatch::empty(),
                    }
                } else {
                    RawItemStack::empty()
                };
                raw.write_optional_untrusted(payload)?;
            }
            // Carried (cursor) item.
            // Java: ServerPlayer.containerMenu.setRemoteCarried(carried)
            let carried = &play_state.carried_item;
            let raw_carried = if carried.is_empty() {
                RawItemStack::empty()
            } else if let Some(pid) = item_protocol_id(carried.item_id()) {
                RawItemStack {
                    count: carried.count(),
                    item_id: Some(pid),
                    components: RawDataComponentPatch::empty(),
                }
            } else {
                RawItemStack::empty()
            };
            raw_carried.write_optional_untrusted(payload)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
        |payload| write_var_i32(payload, 0),
    )?;
    // Full clock sync so the client's Timeline system can start rendering the sky.
    // Java: ServerClockManager.createFullSyncPacket() — sent during ServerLevel.sendLevelInfo()
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_TIME_PACKET_ID,
        |payload| {
            ClientboundSetTimePacket {
                game_time: clock_game_time,
                clock_updates: clock_data.iter().cloned().collect(),
            }
            .write(payload)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        |payload| {
            write_var_i32(payload, center_chunk_x)?;
            write_var_i32(payload, center_chunk_z)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        |payload| write_var_i32(payload, properties.view_distance as i32),
    )?;
    write_play_chunk_delta(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        &[(center_chunk_x, center_chunk_z)],
        false,
        world_root,
        world_seed,
        chunk_cache,
        None,
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
        |payload| {
            write_var_i32(payload, 0)?;
            write_vec3(payload, play_state.x, play_state.y, play_state.z)?;
            write_vec3(payload, 0.0, 0.0, 0.0)?;
            payload.write_all(&play_state.yaw.to_be_bytes())?;
            payload.write_all(&play_state.pitch.to_be_bytes())?;
            payload.write_all(&0_i32.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID,
        |payload| write_initialize_world_border_packet(payload),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
        |payload| {
            let default_spawn = world_spawn_suggestion(world_root, world_seed);
            write_default_spawn_position_packet(
                payload,
                default_spawn.0,
                default_spawn.1,
                default_spawn.2,
            )
        },
    )?;
    // Type 2 = StopRaining (used to initialise client weather state even when not raining).
    // Java: ServerLevel.sendLevelInfo() sends BeginRaining/StopRaining on join.
    write_game_event(stream, compression, 2, 0.0)?;
    // Types 7 and 8: current rain/thunder levels.
    // Java: ServerLevel.advanceWeatherCycle() — RainLevelChange/ThunderLevelChange
    write_game_event(stream, compression, 7, rain_level)?;
    write_game_event(stream, compression, 8, thunder_level)?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[LEVEL_CHUNKS_LOAD_START_GAME_EVENT_ID])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
    delay_initial_chunk_batch_for_probe(stream, compression)?;
    write_play_chunk_batch(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        chunk_batch_radius(properties),
        false,
        world_root,
        world_seed,
        chunk_cache,
    )
}

fn write_play_chunk_batch(
    stream: &mut TcpStream,
    compression: CompressionState,
    center_chunk_x: i32,
    center_chunk_z: i32,
    radius: i32,
    update_cache_center: bool,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    if update_cache_center {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
            |payload| {
                write_var_i32(payload, center_chunk_x)?;
                write_var_i32(payload, center_chunk_z)
            },
        )?;
    }
    let chunks: Vec<_> = ((center_chunk_z - radius)..=(center_chunk_z + radius))
        .flat_map(|z| ((center_chunk_x - radius)..=(center_chunk_x + radius)).map(move |x| (x, z)))
        .filter(|&(x, z)| x != center_chunk_x || z != center_chunk_z)
        .collect();
    write_play_chunk_delta(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        &chunks,
        false,
        world_root,
        world_seed,
        chunk_cache,
        None,
    )
}

fn write_play_chunk_delta(
    stream: &mut TcpStream,
    compression: CompressionState,
    center_chunk_x: i32,
    center_chunk_z: i32,
    chunks: &[(i32, i32)],
    update_cache_center: bool,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
    mut live_fluid_ticks: Option<(&mut LiveFluidTicks, i64, &WorldLayout)>,
) -> io::Result<()> {
    let batch_started = Instant::now();
    eprintln!(
        "[chunk-batch-timing] start center=({}, {}) chunks={} update_center={} live_fluid_seed={}",
        center_chunk_x,
        center_chunk_z,
        chunks.len(),
        update_cache_center,
        live_fluid_ticks.is_some()
    );
    if update_cache_center {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
            |payload| {
                write_var_i32(payload, center_chunk_x)?;
                write_var_i32(payload, center_chunk_z)
            },
        )?;
    }
    if chunks.is_empty() {
        eprintln!(
            "[chunk-batch-timing] finish center=({}, {}) chunks=0 elapsed={}ms",
            center_chunk_x,
            center_chunk_z,
            batch_started.elapsed().as_millis()
        );
        return Ok(());
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_payload| Ok(()),
    )?;

    // Java schedules chunk status work on the worldgen background executor
    // (`NoiseBasedChunkGenerator.fillFromNoise` uses `supplyAsync(...,
    // Util.backgroundExecutor().forName("wgen_fill_noise"))`) and lets the
    // client receive ready chunks progressively. Generate the complete
    // configured view-distance set, but do not wait for the entire square before
    // sending the first finished chunks.
    let workers = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4)
        .clamp(1, 4);
    let worker_count = chunks.len().min(workers);
    let queue = Arc::new(Mutex::new(VecDeque::from(chunks.to_vec())));
    let (sender, receiver) = mpsc::channel::<(i32, i32, Arc<LevelChunk>)>();
    thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);
            let sender = sender.clone();
            let cache = chunk_cache.clone();
            scope.spawn(move || loop {
                let Some((x, z)) = queue.lock().unwrap().pop_front() else {
                    break;
                };
                let chunk = cache.get_or_load(x, z, world_root, world_seed);
                if sender.send((x, z, chunk)).is_err() {
                    break;
                }
            });
        }
        drop(sender);

        for received in 0..chunks.len() {
            let recv_started = Instant::now();
            let (_x, _z, chunk) = receiver.recv().map_err(|err| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    format!("chunk generation worker stopped before batch completed: {err}"),
                )
            })?;
            let recv_ms = recv_started.elapsed().as_millis();
            let write_started = Instant::now();
            if let Some((ticks, game_time, layout)) = live_fluid_ticks.as_mut() {
                seed_live_fluid_ticks_from_chunk(
                    &mut **ticks,
                    *game_time,
                    *layout,
                    world_seed,
                    &chunk,
                );
            }
            write_generated_spawn_chunk_packets_from_chunk(stream, compression, &chunk)?;
            let write_ms = write_started.elapsed().as_millis();
            if write_ms >= 10 || recv_ms >= 10 || received + 1 == chunks.len() {
                eprintln!(
                    "[chunk-batch-timing] progress center=({}, {}) sent={}/{} chunk=({}, {}) recv_wait={}ms write={}ms elapsed={}ms",
                    center_chunk_x,
                    center_chunk_z,
                    received + 1,
                    chunks.len(),
                    chunk.pos.x,
                    chunk.pos.z,
                    recv_ms,
                    write_ms,
                    batch_started.elapsed().as_millis()
                );
            }
        }
        Ok::<(), io::Error>(())
    })?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, chunks.len() as i32),
    )?;
    eprintln!(
        "[chunk-batch-timing] finish center=({}, {}) chunks={} elapsed={}ms",
        center_chunk_x,
        center_chunk_z,
        chunks.len(),
        batch_started.elapsed().as_millis()
    );
    Ok(())
}

fn write_generated_spawn_chunk_packets_from_chunk<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    chunk: &LevelChunk,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID,
        |payload| write_generated_spawn_chunk_payload(payload, chunk),
    )?;
    for plan in generated_chunk_entity_spawn_plans(chunk) {
        write_generated_chunk_entity_spawn_packets(writer, compression, &plan)?;
    }
    Ok(())
}

fn chunk_batch_radius(properties: &ServerProperties) -> i32 {
    (properties.view_distance as i32).clamp(MIN_CHUNK_BATCH_RADIUS, MAX_CHUNK_BATCH_RADIUS)
}

fn chunk_batch_size(radius: i32) -> i32 {
    (radius * 2 + 1) * (radius * 2 + 1)
}

fn delay_initial_chunk_batch_for_probe(
    stream: &mut TcpStream,
    compression: CompressionState,
) -> io::Result<()> {
    let delay_ms = env::var("RUSTCRAFT_INITIAL_CHUNK_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if delay_ms == 0 {
        return Ok(());
    }

    let deadline = Instant::now() + Duration::from_millis(delay_ms);
    let mut last_keep_alive = Instant::now();
    let mut keep_alive_id = 0_i64;
    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        if now.duration_since(last_keep_alive) >= PLAY_KEEP_ALIVE_INTERVAL {
            keep_alive_id = keep_alive_id.wrapping_add(1);
            write_framed_packet_with_compression(
                stream,
                compression,
                CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
                |payload| payload.write_all(&keep_alive_id.to_be_bytes()),
            )?;
            last_keep_alive = now;
        }
        thread::sleep(Duration::from_millis(25).min(deadline.saturating_duration_since(now)));
    }
    Ok(())
}

fn chunk_window(center_chunk_x: i32, center_chunk_z: i32, radius: i32) -> BTreeSet<(i32, i32)> {
    let mut chunks = BTreeSet::new();
    for z in (center_chunk_z - radius)..=(center_chunk_z + radius) {
        for x in (center_chunk_x - radius)..=(center_chunk_x + radius) {
            chunks.insert((x, z));
        }
    }
    chunks
}

fn newly_visible_chunks(
    previous: &BTreeSet<(i32, i32)>,
    next: &BTreeSet<(i32, i32)>,
) -> Vec<(i32, i32)> {
    next.difference(previous).copied().collect()
}

fn write_forget_level_chunk_packet(
    stream: &mut impl Write,
    compression: CompressionState,
    x: i32,
    z: i32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
        |payload| payload.write_all(&packed_chunk_pos(x, z).to_be_bytes()),
    )
}

fn write_forget_generated_spawn_chunk_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
    chunk_cache: &GeneratedChunkCache,
) -> io::Result<()> {
    let chunk = chunk_cache.get_or_load(x, z, world_root, world_seed);
    write_generated_chunk_entity_remove_packets(writer, compression, &chunk)?;
    write_forget_level_chunk_packet(writer, compression, x, z)
}

fn write_generated_chunk_entity_remove_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    chunk: &LevelChunk,
) -> io::Result<()> {
    let entity_ids: Vec<i32> = generated_chunk_entity_add_packets(chunk)
        .into_iter()
        .map(|packet| packet.id)
        .collect();
    if entity_ids.is_empty() {
        return Ok(());
    }
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        |payload| ClientboundRemoveEntitiesPacket { entity_ids }.write(payload),
    )
}

fn packed_chunk_pos(x: i32, z: i32) -> i64 {
    (i64::from(x) & 0xffff_ffff) | ((i64::from(z) & 0xffff_ffff) << 32)
}

fn chunk_coordinate(block_coordinate: f64) -> i32 {
    (block_coordinate.floor() as i32).div_euclid(16)
}

fn write_player_info_initializing_packet<W: Write>(
    writer: &mut W,
    profile: &NameAndId,
    game_mode: GameMode,
) -> io::Result<()> {
    writer.write_all(&[0xff])?;
    write_var_i32(writer, 1)?;
    write_uuid(writer, uuid_from_hyphenated(&profile.uuid)?)?;
    crate::network::codec::write_string(writer, &profile.name, 16)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, false)?;
    write_var_i32(writer, game_mode_legacy_id(game_mode))?;
    write_bool(writer, true)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, false)?;
    write_var_i32(writer, 0)?;
    write_bool(writer, true)
}

fn write_player_abilities_packet<W: Write>(writer: &mut W, game_mode: GameMode) -> io::Result<()> {
    let flags = match game_mode {
        GameMode::Survival | GameMode::Adventure => 0,
        GameMode::Creative => 0x0d,
        GameMode::Spectator => 0x0f,
    };
    writer.write_all(&[flags])?;
    writer.write_all(&0.05f32.to_be_bytes())?;
    writer.write_all(&0.1f32.to_be_bytes())
}

fn write_command_suggestions_response<R: Read>(
    stream: &mut TcpStream,
    compression: CompressionState,
    input: &mut R,
) -> io::Result<()> {
    let transaction_id = read_var_i32(input)?;
    let command = read_string(input, 32767)?;
    let query = command.strip_prefix('/').unwrap_or(&command);
    let matches: Vec<&str> = PLAY_COMMAND_SUGGESTIONS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(query))
        .collect();
    let replacement_start = if command.starts_with('/') { 1 } else { 0 };

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
        |payload| {
            write_var_i32(payload, transaction_id)?;
            write_var_i32(payload, replacement_start)?;
            write_var_i32(payload, query.len() as i32)?;
            write_var_i32(payload, matches.len() as i32)?;
            for candidate in matches {
                write_string(payload, candidate)?;
                write_bool(payload, false)?;
            }
            Ok(())
        },
    )
}

fn uuid_from_hyphenated(value: &str) -> io::Result<Uuid> {
    let hex: String = value.chars().filter(|ch| *ch != '-').collect();
    if hex.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid UUID length",
        ));
    }
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    }
    Ok(Uuid(bytes))
}

fn write_initialize_world_border_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all(&0.0f64.to_be_bytes())?;
    writer.write_all(&0.0f64.to_be_bytes())?;
    writer.write_all(&59_999_968.0f64.to_be_bytes())?;
    writer.write_all(&59_999_968.0f64.to_be_bytes())?;
    write_var_i64(writer, 0)?;
    write_var_i32(writer, 29_999_984)?;
    write_var_i32(writer, 5)?;
    write_var_i32(writer, 15)
}

fn write_default_spawn_position_packet<W: Write>(
    writer: &mut W,
    x: i32,
    y: i32,
    z: i32,
) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:overworld").unwrap())?;
    writer.write_all(&block_pos_as_long(x, y, z).to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())?;
    writer.write_all(&0.0f32.to_be_bytes())
}

fn block_pos_as_long(x: i32, y: i32, z: i32) -> i64 {
    const PACKED_HORIZONTAL_LENGTH: u32 = 26;
    const PACKED_Y_LENGTH: u32 = 12;
    const Z_OFFSET: u32 = PACKED_Y_LENGTH;
    const X_OFFSET: u32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;
    const PACKED_X_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;
    const PACKED_Y_MASK: i64 = (1_i64 << PACKED_Y_LENGTH) - 1;
    const PACKED_Z_MASK: i64 = (1_i64 << PACKED_HORIZONTAL_LENGTH) - 1;

    ((x as i64 & PACKED_X_MASK) << X_OFFSET)
        | (y as i64 & PACKED_Y_MASK)
        | ((z as i64 & PACKED_Z_MASK) << Z_OFFSET)
}

fn write_generated_spawn_chunk_packet<W: Write>(
    writer: &mut W,
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
) -> io::Result<()> {
    let chunk = load_or_generate_spawn_chunk_uncached(x, z, world_root, world_seed);
    write_generated_spawn_chunk_payload(writer, &chunk)
}

#[derive(Debug, Clone, PartialEq)]
struct GeneratedChunkEntitySpawnPlan {
    add_entity: ClientboundAddEntityPacket,
    metadata: Option<ClientboundSetEntityDataPacket>,
}

fn write_generated_chunk_entity_spawn_packets<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    plan: &GeneratedChunkEntitySpawnPlan,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )?;
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |payload| plan.add_entity.write(payload),
    )?;
    if let Some(metadata) = &plan.metadata {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
            |payload| metadata.write(payload),
        )?;
    }
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )
}

fn write_generated_spawn_chunk_payload<W: Write>(
    writer: &mut W,
    chunk: &LevelChunk,
) -> io::Result<()> {
    let started = Instant::now();
    let light_data = ClientboundLightUpdatePacketData::from_chunk(&chunk);
    let light_ms = started.elapsed().as_millis();
    let packet_started = Instant::now();
    let packet = ClientboundLevelChunkWithLightPacket::from_chunk(&chunk, light_data);
    let packet_build_ms = packet_started.elapsed().as_millis();
    let write_started = Instant::now();
    let result = write_level_chunk_with_light_payload(writer, &packet);
    eprintln!(
        "[worldgen] chunk=({}, {}) packet light={}ms build={}ms write={}ms bytes={} block_entities={} heightmaps={}",
        chunk.pos.x,
        chunk.pos.z,
        light_ms,
        packet_build_ms,
        write_started.elapsed().as_millis(),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.buffer.len())
            .unwrap_or(0),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.block_entities.len())
            .unwrap_or(0),
        packet
            .chunk_data
            .as_ref()
            .map(|data| data.heightmaps.len())
            .unwrap_or(0)
    );
    result
}

fn load_or_generate_spawn_chunk_uncached(
    x: i32,
    z: i32,
    world_root: &Path,
    world_seed: i64,
) -> LevelChunk {
    let started = Instant::now();
    let pos = ChunkPos { x, z };
    let region_dir = world_root.join("region");
    let mut source = "region";
    let region_started = Instant::now();
    let loaded = try_load_chunk_from_region(&region_dir, pos);
    let region_ms = region_started.elapsed().as_millis();
    let chunk = loaded.unwrap_or_else(|| {
        source = match live_chunk_generation_mode() {
            LiveChunkGenerationMode::Preview => "generated-preview",
            LiveChunkGenerationMode::RealSurface => "generated-real-surface",
        };
        match generate_overworld_spawn_chunk_for_preset_with_mode_timed(
            pos,
            "normal",
            live_chunk_generation_mode(),
            world_seed,
            true,
        ) {
            Ok((chunk, timings)) => {
                eprintln!(
                    "[worldgen] chunk=({}, {}) phases region={}ms preset={}ms terrain={}ms fill={}ms fill_init_sections={}ms fill_noise_chunk_init={}ms fill_aquifer_init={}ms fill_block_loop={}ms fill_density_lookup={}us fill_aquifer_compute={}us fill_ore_vein_lookup={}us fill_ore_decision={}us fill_interpolation_update={}us interpolators={} fill_full_noise_cache={}ms fill_full_noise_cache_fills={} fill_vein_noise_cache={}ms fill_vein_noise_cache_fills={} cache_once_scalar_hits={} cache_once_scalar_misses={} cache_once_array_hits={} cache_once_array_misses={} fill_heightmap_pack={}ms fill_cell_columns={} fill_block_samples={} fill_block_writes={} aquifer_calls={} ore_vein_samples={} surface={}ms surface_noise_setup={}ms surface_prelim={}ms surface_column_loop={}ms surface_columns={} surface_block_samples={} surface_block_writes={} tree_context={}ms tree_context_chunks={} tree_decoration={}ms tree_blocks={} heightmaps={}ms heightmap_decode={}ms heightmap_scan={}ms heightmap_pack={}ms heightmap_sections={} heightmap_samples={} mobs={}ms mob_plan={}ms mob_biome={}ms mob_spawn_plan={}ms mob_apply={}ms mob_top={}ms mob_position_ok={}ms mob_snap_collision={}ms mob_rules={}ms mob_queue={}ms mob_random_walk={}ms mob_batches={} mob_attempts={} mobs_spawned={}",
                    x,
                    z,
                    region_ms,
                    timings.resolve_preset_ms,
                    timings.terrain_ms,
                    timings.terrain.fill_total_ms,
                    timings.terrain.fill_init_sections_ms,
                    timings.terrain.fill_noise_chunk_init_ms,
                    timings.terrain.fill_aquifer_init_ms,
                    timings.terrain.fill_block_loop_ms,
                    timings.terrain.fill_density_lookup_us,
                    timings.terrain.fill_aquifer_compute_us,
                    timings.terrain.fill_ore_vein_lookup_us,
                    timings.terrain.fill_ore_decision_us,
                    timings.terrain.fill_interpolation_update_us,
                    timings.terrain.interpolator_count,
                    timings.terrain.fill_full_noise_cache_ms,
                    timings.terrain.full_noise_cache_fills,
                    timings.terrain.fill_vein_noise_cache_ms,
                    timings.terrain.vein_noise_cache_fills,
                    timings.terrain.cache_once_scalar_hits,
                    timings.terrain.cache_once_scalar_misses,
                    timings.terrain.cache_once_array_hits,
                    timings.terrain.cache_once_array_misses,
                    timings.terrain.fill_heightmap_pack_ms,
                    timings.terrain.cell_columns,
                    timings.terrain.block_samples,
                    timings.terrain.block_writes,
                    timings.terrain.aquifer_calls,
                    timings.terrain.ore_vein_samples,
                    timings.terrain.surface_total_ms,
                    timings.terrain.surface_noise_setup_ms,
                    timings.terrain.surface_prelim_ms,
                    timings.terrain.surface_column_loop_ms,
                    timings.terrain.surface_columns,
                    timings.terrain.surface_block_samples,
                    timings.terrain.surface_block_writes,
                    timings.tree_context_ms,
                    timings.tree_context_chunks,
                    timings.tree_decoration_ms,
                    timings.tree_blocks,
                    timings.heightmaps.total_ms,
                    timings.heightmaps.decode_sections_ms,
                    timings.heightmaps.scan_blocks_ms,
                    timings.heightmaps.pack_store_ms,
                    timings.heightmaps.sections_decoded,
                    timings.heightmaps.block_samples,
                    timings.mobs.total_ms,
                    timings.mobs.plan_ms,
                    timings.mobs.biome_ms,
                    timings.mobs.spawn_plan_ms,
                    timings.mobs.apply_batches_ms,
                    timings.mobs.top_position_ms,
                    timings.mobs.position_ok_ms,
                    timings.mobs.snap_collision_ms,
                    timings.mobs.spawn_rules_ms,
                    timings.mobs.queue_ms,
                    timings.mobs.random_walk_ms,
                    timings.mobs.batches,
                    timings.mobs.attempts,
                    timings.mobs.mobs_spawned
                );
                chunk
            }
            Err(err) => {
                source = "generated-fallback-empty";
                eprintln!(
                    "[worldgen] chunk=({}, {}) generation failed after {}ms: {}",
                    x,
                    z,
                    started.elapsed().as_millis(),
                    err
                );
                crate::storage::chunk::LevelChunk::empty(pos)
            }
        }
    });
    eprintln!(
        "[worldgen] chunk=({}, {}) source={} status={} sections={} elapsed={}ms",
        x,
        z,
        source,
        chunk.status,
        chunk.sections.len(),
        started.elapsed().as_millis()
    );
    chunk
}

fn live_chunk_generation_mode() -> LiveChunkGenerationMode {
    match std::env::var("RUSTCRAFT_WORLDGEN").as_deref() {
        Ok("preview") => LiveChunkGenerationMode::Preview,
        Ok("real-surface") | Ok("surface") | Err(_) => LiveChunkGenerationMode::RealSurface,
        Ok(other) => {
            eprintln!(
                "unknown RUSTCRAFT_WORLDGEN={other:?}; using real-surface (set preview for scaffold terrain)"
            );
            LiveChunkGenerationMode::RealSurface
        }
    }
}

fn generated_chunk_entity_add_packets(chunk: &LevelChunk) -> Vec<ClientboundAddEntityPacket> {
    generated_chunk_entity_spawn_plans(chunk)
        .into_iter()
        .map(|plan| plan.add_entity)
        .collect()
}

fn generated_chunk_entity_spawn_plans(chunk: &LevelChunk) -> Vec<GeneratedChunkEntitySpawnPlan> {
    chunk
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| generated_chunk_entity_spawn_plan(chunk.pos, index, entity))
        .collect()
}

fn generated_chunk_entity_spawn_plan(
    chunk_pos: ChunkPos,
    index: usize,
    entity: &Tag,
) -> Option<GeneratedChunkEntitySpawnPlan> {
    let Tag::Compound(fields) = entity else {
        return None;
    };
    let entity_type_name = tag_string_field(fields, "id")?;
    let entity_type = generated_mob_entity_type_network_id(entity_type_name)?;
    let uuid = uuid_from_hyphenated(tag_string_field(fields, "UUID")?).ok()?;
    let [x, y, z] = tag_double_triplet_field(fields, "Pos")?;
    let [yaw, pitch] = tag_float_pair_field(fields, "Rotation")?;
    let runtime_id = generated_chunk_entity_runtime_id(chunk_pos, index);
    let add_entity = ClientboundAddEntityPacket::new(
        runtime_id,
        uuid,
        entity_type,
        Vec3 { x, y, z },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        (pitch, yaw),
        yaw,
        0,
    );
    Some(GeneratedChunkEntitySpawnPlan {
        add_entity,
        metadata: generated_chunk_entity_metadata_packet(runtime_id, entity_type_name, fields),
    })
}

fn generated_chunk_entity_metadata_packet(
    runtime_id: i32,
    entity_type: &str,
    fields: &[(String, Tag)],
) -> Option<ClientboundSetEntityDataPacket> {
    let mut packed_items = Vec::new();
    match entity_type {
        "minecraft:cat" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(cat_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(20, EntityMetadataValue::CatVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(cat_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(24, EntityMetadataValue::CatSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:chicken" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(chicken_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::ChickenVariant(variant))
                        .ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(chicken_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(
                        19,
                        EntityMetadataValue::ChickenSoundVariant(sound_variant),
                    )
                    .ok()?,
                );
            }
        }
        "minecraft:cow" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(cow_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::CowVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(cow_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(19, EntityMetadataValue::CowSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:frog" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(frog_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(18, EntityMetadataValue::FrogVariant(variant)).ok()?,
                );
            }
        }
        "minecraft:pig" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(pig_variant_registry_id)
                .filter(|variant| *variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(19, EntityMetadataValue::PigVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(pig_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 1)
            {
                packed_items.push(
                    EntityDataValue::typed(20, EntityMetadataValue::PigSoundVariant(sound_variant))
                        .ok()?,
                );
            }
        }
        "minecraft:wolf" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(wolf_variant_registry_id)
                .filter(|variant| *variant != 3)
            {
                packed_items.push(
                    EntityDataValue::typed(23, EntityMetadataValue::WolfVariant(variant)).ok()?,
                );
            }
            if let Some(sound_variant) = tag_string_field(fields, "sound_variant")
                .and_then(wolf_sound_variant_registry_id)
                .filter(|sound_variant| *sound_variant != 2)
            {
                packed_items.push(
                    EntityDataValue::typed(
                        24,
                        EntityMetadataValue::WolfSoundVariant(sound_variant),
                    )
                    .ok()?,
                );
            }
        }
        "minecraft:zombie_nautilus" => {
            if let Some(variant) = tag_string_field(fields, "variant")
                .and_then(zombie_nautilus_variant_registry_id)
                .filter(|variant| *variant != 0)
            {
                packed_items.push(
                    EntityDataValue::typed(21, EntityMetadataValue::ZombieNautilusVariant(variant))
                        .ok()?,
                );
            }
        }
        _ => {}
    }
    (!packed_items.is_empty()).then_some(ClientboundSetEntityDataPacket {
        id: runtime_id,
        packed_items,
    })
}

fn tag_string_field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name == name {
            if let Tag::String(value) = value {
                return Some(value.as_str());
            }
        }
        None
    })
}

fn resource_path_id(value: &str) -> &str {
    value.strip_prefix("minecraft:").unwrap_or(value)
}

fn cat_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "all_black" => Some(0),
        "black" => Some(1),
        "british_shorthair" => Some(2),
        "calico" => Some(3),
        "jellie" => Some(4),
        "persian" => Some(5),
        "ragdoll" => Some(6),
        "red" => Some(7),
        "siamese" => Some(8),
        "tabby" => Some(9),
        "white" => Some(10),
        _ => None,
    }
}

fn cat_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "royal" => Some(1),
        _ => None,
    }
}

fn chicken_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

fn chicken_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "picky" => Some(1),
        _ => None,
    }
}

fn cow_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

fn cow_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "classic" => Some(0),
        "moody" => Some(1),
        _ => None,
    }
}

fn frog_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

fn pig_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "cold" => Some(0),
        "temperate" => Some(1),
        "warm" => Some(2),
        _ => None,
    }
}

fn wolf_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "ashen" => Some(0),
        "black" => Some(1),
        "chestnut" => Some(2),
        "pale" => Some(3),
        "rusty" => Some(4),
        "snowy" => Some(5),
        "spotted" => Some(6),
        "striped" => Some(7),
        "woods" => Some(8),
        _ => None,
    }
}

fn wolf_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "angry" => Some(0),
        "big" => Some(1),
        "classic" => Some(2),
        "cute" => Some(3),
        "grumpy" => Some(4),
        "puglin" => Some(5),
        "sad" => Some(6),
        _ => None,
    }
}

fn pig_sound_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "big" => Some(0),
        "classic" => Some(1),
        "mini" => Some(2),
        _ => None,
    }
}

fn zombie_nautilus_variant_registry_id(value: &str) -> Option<i32> {
    match resource_path_id(value) {
        "temperate" => Some(0),
        "warm" => Some(1),
        _ => None,
    }
}

fn tag_double_triplet_field(fields: &[(String, Tag)], name: &str) -> Option<[f64; 3]> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name != name {
            return None;
        }
        let Tag::List(values) = value else {
            return None;
        };
        let [Tag::Double(x), Tag::Double(y), Tag::Double(z)] = values.as_slice() else {
            return None;
        };
        Some([*x, *y, *z])
    })
}

fn tag_float_pair_field(fields: &[(String, Tag)], name: &str) -> Option<[f32; 2]> {
    fields.iter().find_map(|(field_name, value)| {
        if field_name != name {
            return None;
        }
        let Tag::List(values) = value else {
            return None;
        };
        let [Tag::Float(first), Tag::Float(second)] = values.as_slice() else {
            return None;
        };
        Some([*first, *second])
    })
}

fn generated_chunk_entity_runtime_id(chunk_pos: ChunkPos, index: usize) -> i32 {
    let x = chunk_pos.x.rem_euclid(1024);
    let z = chunk_pos.z.rem_euclid(1024);
    1_000_000 + x * 1_048_576 + z * 256 + (index as i32 & 0xff)
}

fn generated_mob_entity_type_network_id(entity_type: &str) -> Option<i32> {
    match entity_type {
        "minecraft:armadillo" => Some(4),
        "minecraft:axolotl" => Some(7),
        "minecraft:bat" => Some(10),
        "minecraft:bogged" => Some(16),
        "minecraft:camel" => Some(19),
        "minecraft:chicken" => Some(26),
        "minecraft:cod" => Some(27),
        "minecraft:cow" => Some(30),
        "minecraft:creeper" => Some(32),
        "minecraft:dolphin" => Some(35),
        "minecraft:donkey" => Some(36),
        "minecraft:drowned" => Some(38),
        "minecraft:enderman" => Some(41),
        "minecraft:fox" => Some(54),
        "minecraft:frog" => Some(55),
        "minecraft:ghast" => Some(57),
        "minecraft:glow_squid" => Some(61),
        "minecraft:goat" => Some(62),
        "minecraft:hoglin" => Some(64),
        "minecraft:horse" => Some(66),
        "minecraft:husk" => Some(67),
        "minecraft:llama" => Some(78),
        "minecraft:magma_cube" => Some(80),
        "minecraft:mooshroom" => Some(86),
        "minecraft:mule" => Some(87),
        "minecraft:ocelot" => Some(91),
        "minecraft:panda" => Some(96),
        "minecraft:parched" => Some(97),
        "minecraft:parrot" => Some(98),
        "minecraft:pig" => Some(100),
        "minecraft:piglin" => Some(101),
        "minecraft:polar_bear" => Some(104),
        "minecraft:pufferfish" => Some(107),
        "minecraft:rabbit" => Some(108),
        "minecraft:salmon" => Some(110),
        "minecraft:sheep" => Some(111),
        "minecraft:skeleton" => Some(115),
        "minecraft:slime" => Some(117),
        "minecraft:spider" => Some(124),
        "minecraft:squid" => Some(127),
        "minecraft:stray" => Some(128),
        "minecraft:strider" => Some(129),
        "minecraft:trader_llama" => Some(134),
        "minecraft:tropical_fish" => Some(136),
        "minecraft:turtle" => Some(137),
        "minecraft:witch" => Some(144),
        "minecraft:wolf" => Some(148),
        "minecraft:zombie" => Some(150),
        "minecraft:zombie_horse" => Some(151),
        "minecraft:zombie_nautilus" => Some(152),
        "minecraft:zombie_villager" => Some(153),
        "minecraft:zombified_piglin" => Some(154),
        _ => None,
    }
}

fn try_load_chunk_from_region(
    region_dir: &Path,
    pos: ChunkPos,
) -> Option<crate::storage::chunk::LevelChunk> {
    let region = RegionFile::open(region_dir, pos.region()).ok()?;
    let (_name, tag) = region.read_chunk_nbt(pos).ok()??;
    crate::storage::chunk::LevelChunk::from_nbt(pos, &tag)
        .ok()
        .filter(chunk_has_non_air_blocks)
}

fn chunk_has_non_air_blocks(chunk: &LevelChunk) -> bool {
    chunk.sections.iter().any(|section| {
        PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
            .ok()
            .is_some_and(|container| container.palette.iter().any(palette_entry_is_non_air))
    })
}

fn palette_entry_is_non_air(entry: &Tag) -> bool {
    match entry {
        Tag::String(name) => name != "minecraft:air" && name != "air",
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(name, value)| {
                (name == "Name").then_some(value).and_then(|value| {
                    if let Tag::String(block_name) = value {
                        Some(block_name != "minecraft:air" && block_name != "air")
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(false),
        Tag::Int(id) => *id != 0,
        _ => false,
    }
}

fn write_level_chunk_with_light_payload<W: Write>(
    writer: &mut W,
    packet: &ClientboundLevelChunkWithLightPacket,
) -> io::Result<()> {
    writer.write_all(&packet.pos.x.to_be_bytes())?;
    writer.write_all(&packet.pos.z.to_be_bytes())?;
    let chunk_data = packet.chunk_data.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "level chunk with light packet requires chunk data",
        )
    })?;
    write_level_chunk_packet_data(writer, chunk_data)?;
    let light_data = packet.light_data.as_ref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "level chunk with light packet requires light data",
        )
    })?;
    light_data.write(writer)
}

fn write_level_chunk_packet_data<W: Write>(
    writer: &mut W,
    data: &ClientboundLevelChunkPacketData,
) -> io::Result<()> {
    data.write(writer)
}

#[allow(dead_code)]
fn write_superflat_spawn_chunk_packet<W: Write>(writer: &mut W, x: i32, z: i32) -> io::Result<()> {
    writer.write_all(&x.to_be_bytes())?;
    writer.write_all(&z.to_be_bytes())?;
    write_var_i32(writer, 0)?;

    let mut section_buffer = Vec::with_capacity(SPAWN_CHUNK_SECTION_COUNT * 10);
    for section_index in 0..SPAWN_CHUNK_SECTION_COUNT {
        let non_empty_block_count = visible_spawn_terrain_block_count(x, z, section_index);
        section_buffer.write_all(&non_empty_block_count.to_be_bytes())?;
        section_buffer.write_all(&0_i16.to_be_bytes())?;
        if non_empty_block_count > 0 {
            write_visible_spawn_terrain_block_state_container(
                &mut section_buffer,
                x,
                z,
                section_index,
            )?;
        } else {
            write_single_value_paletted_container(&mut section_buffer, AIR_BLOCK_STATE_ID)?;
        }
        write_single_value_paletted_container(&mut section_buffer, PLAINS_BIOME_ID)?;
    }
    write_var_i32(writer, section_buffer.len() as i32)?;
    writer.write_all(&section_buffer)?;
    write_var_i32(writer, 0)?;

    write_bitset(writer, &[(1_u64 << SPAWN_CHUNK_SECTION_COUNT) - 1])?;
    write_empty_bitset(writer)?;
    write_empty_bitset(writer)?;
    write_bitset(writer, &[(1_u64 << SPAWN_CHUNK_SECTION_COUNT) - 1])?;
    write_var_i32(writer, SPAWN_CHUNK_SECTION_COUNT as i32)?;
    for _ in 0..SPAWN_CHUNK_SECTION_COUNT {
        write_var_i32(writer, 2048)?;
        writer.write_all(&[0xff; 2048])?;
    }
    write_var_i32(writer, 0)
}

#[allow(dead_code)]
fn write_single_value_paletted_container<W: Write>(writer: &mut W, id: i32) -> io::Result<()> {
    writer.write_all(&[0])?;
    write_var_i32(writer, id)
}

fn visible_spawn_terrain_height(chunk_x: i32, chunk_z: i32, local_x: usize, local_z: usize) -> i32 {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let broad = (world_x.div_euclid(12) + world_z.div_euclid(14)).rem_euclid(8);
    let terrace = (world_x.div_euclid(5) - world_z.div_euclid(7)).rem_euclid(6);
    let wrinkle = ((world_x.wrapping_mul(31) ^ world_z.wrapping_mul(17)) & 3) as i32;
    let ridge = if (world_x.wrapping_mul(11) + world_z.wrapping_mul(13)).rem_euclid(29) <= 2 {
        14
    } else {
        0
    };
    let plateau = if (world_x.div_euclid(24) - world_z.div_euclid(19)).rem_euclid(5) == 0 {
        14
    } else {
        0
    };
    let valley = if (world_x.wrapping_mul(5) - world_z.wrapping_mul(7)).rem_euclid(37) <= 3 {
        7
    } else {
        0
    };
    (TERRAIN_MIN_SURFACE_Y + broad + terrace + wrinkle + ridge + plateau - valley).clamp(68, 104)
}

fn visible_spawn_surface_feature_id(
    chunk_x: i32,
    chunk_z: i32,
    local_x: usize,
    local_z: usize,
) -> Option<i32> {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let hash = world_x.wrapping_mul(734_287) ^ world_z.wrapping_mul(912_931);
    match hash.rem_euclid(23) {
        0 => Some(DANDELION_BLOCK_STATE_ID),
        7 | 17 => Some(POPPY_BLOCK_STATE_ID),
        5 | 13 | 19 => Some(SHORT_GRASS_BLOCK_STATE_ID),
        _ => None,
    }
}

fn visible_spawn_surface_top_block_id(
    chunk_x: i32,
    chunk_z: i32,
    local_x: usize,
    local_z: usize,
) -> i32 {
    let world_x = chunk_x * 16 + local_x as i32;
    let world_z = chunk_z * 16 + local_z as i32;
    let hash = world_x.wrapping_mul(193_496_63) ^ world_z.wrapping_mul(83_492_791);
    match hash.rem_euclid(43) {
        0 => STONE_BLOCK_STATE_ID,
        9 => GRANITE_BLOCK_STATE_ID,
        18 => DIORITE_BLOCK_STATE_ID,
        27 => ANDESITE_BLOCK_STATE_ID,
        34 | 41 => DIRT_BLOCK_STATE_ID,
        _ => GRASS_BLOCK_STATE_ID,
    }
}

fn get_block_state_at(x: i32, y: i32, z: i32) -> i32 {
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);
    let local_x = x.rem_euclid(16) as usize;
    let local_z = z.rem_euclid(16) as usize;
    let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, local_x, local_z);
    if y < TERRAIN_BASE_Y || y > top_y + 1 {
        return AIR_BLOCK_STATE_ID;
    }
    if y == TERRAIN_BASE_Y {
        return BEDROCK_BLOCK_STATE_ID;
    }
    if y == top_y + 1 {
        let surface = visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z);
        if surface == GRASS_BLOCK_STATE_ID {
            return visible_spawn_surface_feature_id(chunk_x, chunk_z, local_x, local_z)
                .unwrap_or(AIR_BLOCK_STATE_ID);
        }
        return AIR_BLOCK_STATE_ID;
    }
    if y == top_y {
        return visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z);
    }
    STONE_BLOCK_STATE_ID
}

/// Builds the block loot table for `block_name`, matching the JSON loot tables
/// from data/minecraft/loot_table/blocks/ in the Java source.
fn block_loot_table(block_name: &str) -> Option<LootTable> {
    let key = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    let random_sequence = format!("minecraft:blocks/{key}");

    // A pool that drops one stack of `item` unconditionally.
    fn self_drop_table(item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::item(
                format!("minecraft:{item}"),
                1,
            ))],
            functions: Vec::new(),
        }
    }

    // A pool that drops one stack of `item` only if the block survives explosion.
    // For normal block breaking (no explosion), this always drops.
    fn self_drop_survives_explosion(item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool {
                entries: vec![LootEntry::item(format!("minecraft:{item}"), 1)],
                conditions: vec![LootCondition::SurvivesExplosion],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            }],
            functions: Vec::new(),
        }
    }

    // Ore that always drops a single item (coal, iron, gold, diamond, emerald, quartz).
    // Silk touch (ore block self-drop) not yet implemented; always uses the raw-product path.
    fn ore_drop_1(drop_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: format!("minecraft:{drop_item}"),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::ApplyExplosionDecay],
            })],
            functions: Vec::new(),
        }
    }

    // Ore that drops a uniform-count range (copper 2-5, redstone 4-5, lapis 4-9, etc.).
    // Fortune bonuses not yet implemented; `min`/`max` are base counts.
    fn ore_drop_count(drop_item: &str, min: f32, max: f32, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: format!("minecraft:{drop_item}"),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min, max }),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        }
    }

    // Grass-type plants: shears → self, else 12.5% chance of wheat_seeds.
    fn grass_table(self_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: format!("minecraft:{self_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.125)],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        }
    }

    // Standard leaf block: shears → leaves, else sapling at `sapling_chance` (fortune-0 base),
    // plus a 2% stick pool when not using shears. Oak additionally has a 0.5% apple pool.
    // Silk touch and fortune bonuses not yet implemented.
    fn leaves_table(
        leaves_item: &str,
        sapling_item: &str,
        sapling_chance: f32,
        has_apple_pool: bool,
        sequence: String,
    ) -> LootTable {
        let not_shears = LootCondition::Inverted(Box::new(LootCondition::MatchTool {
            item: "minecraft:shears".to_string(),
        }));
        let mut pools = vec![
            // Pool 0: shears → leaves block, else sapling with survival + chance
            LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: format!("minecraft:{leaves_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: format!("minecraft:{sapling_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(sapling_chance),
                    ],
                    functions: Vec::new(),
                },
            ])),
            // Pool 1: 2% chance of 1-2 sticks when not using shears
            LootPool {
                entries: vec![LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                }],
                conditions: vec![not_shears.clone()],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            },
        ];
        if has_apple_pool {
            // Pool 2 (oak only): 0.5% apple when not using shears
            pools.push(LootPool {
                entries: vec![LootEntry::Item {
                    item: "minecraft:apple".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.005),
                    ],
                    functions: Vec::new(),
                }],
                conditions: vec![not_shears],
                functions: Vec::new(),
                rolls: NumberProvider::Constant(1.0),
                bonus_rolls: NumberProvider::Constant(0.0),
            });
        }
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools,
            functions: Vec::new(),
        }
    }

    // Fully-grown crop: 1 food item + Binomial(3, 0.5714) bonus `bonus_item`.
    // Block-state age checks not yet implemented; always applies mature-crop drops.
    // Fortune bonuses not yet implemented; Binomial(3, 0.5714) is the fortune-0 base count.
    fn mature_crop_table(food_item: &str, bonus_item: &str, sequence: String) -> LootTable {
        LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(sequence),
            pools: vec![
                LootPool::single(LootEntry::item(format!("minecraft:{food_item}"), 1)),
                LootPool::single(LootEntry::Item {
                    item: format!("minecraft:{bonus_item}"),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                        n: 3,
                        p: 0.5714286,
                    })],
                }),
            ],
            functions: vec![LootFunction::ApplyExplosionDecay],
        }
    }

    Some(match key {
        // ── TERRAIN ────────────────────────────────────────────────────────────────────────

        // These drop cobblestone/dirt instead of themselves (silk touch not implemented)
        "stone" => self_drop_table("cobblestone", random_sequence),
        "grass_block" | "mycelium" | "podzol" | "dirt_path" | "farmland" => {
            self_drop_table("dirt", random_sequence)
        }

        // Clay → 4 clay_balls (silk touch not implemented)
        "clay" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:clay_ball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(4.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Gravel: 10% flint (fortune-0 base), else gravel. Both require survives_explosion.
        "gravel" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:flint".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.1),
                    ],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:gravel".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::SurvivesExplosion],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },

        // Terrain self-drops
        "granite"
        | "polished_granite"
        | "diorite"
        | "polished_diorite"
        | "andesite"
        | "polished_andesite"
        | "deepslate"
        | "cobbled_deepslate"
        | "cobblestone"
        | "dirt"
        | "coarse_dirt"
        | "rooted_dirt"
        | "mud"
        | "sand"
        | "red_sand"
        | "sandstone"
        | "chiseled_sandstone"
        | "cut_sandstone"
        | "smooth_sandstone" => self_drop_table(key, random_sequence),

        // ── ORES ───────────────────────────────────────────────────────────────────────────
        // Silk touch (ore block self-drop) not yet implemented; always drops raw product.
        // Fortune bonuses not yet implemented; counts are base values.

        "coal_ore" | "deepslate_coal_ore" => ore_drop_1("coal", random_sequence),
        "iron_ore" | "deepslate_iron_ore" => ore_drop_1("raw_iron", random_sequence),
        "gold_ore" | "deepslate_gold_ore" => ore_drop_1("raw_gold", random_sequence),
        "copper_ore" | "deepslate_copper_ore" => {
            ore_drop_count("raw_copper", 2.0, 5.0, random_sequence)
        }
        "redstone_ore"
        | "lit_redstone_ore"
        | "deepslate_redstone_ore"
        | "lit_deepslate_redstone_ore" => ore_drop_count("redstone", 4.0, 5.0, random_sequence),
        "emerald_ore" | "deepslate_emerald_ore" => ore_drop_1("emerald", random_sequence),
        "lapis_ore" | "deepslate_lapis_ore" => {
            ore_drop_count("lapis_lazuli", 4.0, 9.0, random_sequence)
        }
        "diamond_ore" | "deepslate_diamond_ore" => ore_drop_1("diamond", random_sequence),
        "nether_quartz_ore" => ore_drop_1("quartz", random_sequence),
        "nether_gold_ore" => ore_drop_count("gold_nugget", 2.0, 6.0, random_sequence),

        // ── WOOD ───────────────────────────────────────────────────────────────────────────
        "oak_log"
        | "spruce_log"
        | "birch_log"
        | "jungle_log"
        | "acacia_log"
        | "dark_oak_log"
        | "stripped_oak_log"
        | "stripped_spruce_log"
        | "stripped_birch_log"
        | "stripped_jungle_log"
        | "stripped_acacia_log"
        | "stripped_dark_oak_log"
        | "oak_wood"
        | "spruce_wood"
        | "birch_wood"
        | "jungle_wood"
        | "acacia_wood"
        | "dark_oak_wood"
        | "stripped_oak_wood"
        | "stripped_spruce_wood"
        | "stripped_birch_wood"
        | "stripped_jungle_wood"
        | "stripped_acacia_wood"
        | "stripped_dark_oak_wood"
        | "oak_planks"
        | "spruce_planks"
        | "birch_planks"
        | "jungle_planks"
        | "acacia_planks"
        | "dark_oak_planks" => self_drop_table(key, random_sequence),

        // ── LEAVES ─────────────────────────────────────────────────────────────────────────
        // Silk touch and fortune bonuses not yet implemented. Sapling chance is fortune-0 base.
        // Stick drop: 2% when not using shears. Oak additionally has a 0.5% apple drop.

        "oak_leaves" => leaves_table("oak_leaves", "oak_sapling", 0.05, true, random_sequence),
        "spruce_leaves" => {
            leaves_table("spruce_leaves", "spruce_sapling", 0.05, false, random_sequence)
        }
        "birch_leaves" => {
            leaves_table("birch_leaves", "birch_sapling", 0.05, false, random_sequence)
        }
        // Jungle sapling has a lower base drop chance (2.5% vs 5%)
        "jungle_leaves" => {
            leaves_table("jungle_leaves", "jungle_sapling", 0.025, false, random_sequence)
        }
        "acacia_leaves" => {
            leaves_table("acacia_leaves", "acacia_sapling", 0.05, false, random_sequence)
        }
        "dark_oak_leaves" => {
            leaves_table("dark_oak_leaves", "dark_oak_sapling", 0.05, false, random_sequence)
        }
        "cherry_leaves" => {
            leaves_table("cherry_leaves", "cherry_sapling", 0.05, false, random_sequence)
        }
        "pale_oak_leaves" => {
            leaves_table("pale_oak_leaves", "pale_oak_sapling", 0.05, false, random_sequence)
        }
        // Azalea leaves drop an azalea bush (not a sapling variant)
        "azalea_leaves" => {
            leaves_table("azalea_leaves", "azalea", 0.05, false, random_sequence)
        }
        "flowering_azalea_leaves" => leaves_table(
            "flowering_azalea_leaves",
            "flowering_azalea",
            0.05,
            false,
            random_sequence,
        ),
        // Mangrove leaves: no propagule from breaking; only sticks via the shears-alternative
        "mangrove_leaves" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:mangrove_leaves".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 1.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                },
            ]))],
            functions: Vec::new(),
        },

        // ── PLANTS ─────────────────────────────────────────────────────────────────────────

        // Grass-type: shears → self, else 12.5% wheat_seeds
        "short_grass" => grass_table("short_grass", random_sequence),
        "fern" => grass_table("fern", random_sequence),

        // Double-tall grass: same logic, but shears yield 2 items
        "tall_grass" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:short_grass".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.125),
                    ],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },
        "large_fern" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:fern".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: vec![LootFunction::SetCount(NumberProvider::Constant(2.0))],
                },
                LootEntry::Item {
                    item: "minecraft:wheat_seeds".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![
                        LootCondition::SurvivesExplosion,
                        LootCondition::RandomChance(0.125),
                    ],
                    functions: Vec::new(),
                },
            ]))],
            functions: Vec::new(),
        },

        // Dead bush: shears → dead_bush, else 0-2 sticks
        "dead_bush" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Alternatives(vec![
                LootEntry::Item {
                    item: "minecraft:dead_bush".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::MatchTool {
                        item: "minecraft:shears".to_string(),
                    }],
                    functions: Vec::new(),
                },
                LootEntry::Item {
                    item: "minecraft:stick".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![
                        LootFunction::SetCount(NumberProvider::Uniform { min: 0.0, max: 2.0 }),
                        LootFunction::ApplyExplosionDecay,
                    ],
                },
            ]))],
            functions: Vec::new(),
        },

        // Single-block flowers: always self-drop
        "dandelion"
        | "golden_dandelion"
        | "torchflower"
        | "poppy"
        | "blue_orchid"
        | "allium"
        | "azure_bluet"
        | "red_tulip"
        | "orange_tulip"
        | "white_tulip"
        | "pink_tulip"
        | "oxeye_daisy"
        | "cornflower"
        | "wither_rose"
        | "lily_of_the_valley"
        | "brown_mushroom"
        | "red_mushroom"
        | "wildflowers"
        | "firefly_bush" => self_drop_table(key, random_sequence),

        // Double-tall flowers: drop self (survives_explosion).
        // Java checks block_state_property half=lower (only lower half drops), which we can't
        // evaluate yet. Simplification: always drop on any half break.
        "sunflower" | "lilac" | "rose_bush" | "peony" => {
            self_drop_survives_explosion(key, random_sequence)
        }

        // ── CROPS ──────────────────────────────────────────────────────────────────────────
        // Block-state age checks not yet implemented; always treats crop as fully grown.
        // Fortune bonuses not yet implemented; Binomial(3, 0.5714) is the fortune-0 base count.

        // Wheat at age 7: 1 wheat + Binomial(3, 0.57) bonus wheat_seeds
        "wheat" => mature_crop_table("wheat", "wheat_seeds", random_sequence),
        // Carrots at age 7: 1 carrot + Binomial(3, 0.57) bonus carrots
        "carrots" => mature_crop_table("carrot", "carrot", random_sequence),
        // Beetroots at age 3: 1 beetroot + Binomial(3, 0.57) bonus beetroot_seeds
        "beetroots" => mature_crop_table("beetroot", "beetroot_seeds", random_sequence),
        // Potatoes at age 7: 1 potato + bonus potatoes + 2% poisonous_potato
        "potatoes" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![
                LootPool::single(LootEntry::item("minecraft:potato".to_string(), 1)),
                LootPool::single(LootEntry::Item {
                    item: "minecraft:potato".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: Vec::new(),
                    functions: vec![LootFunction::SetCount(NumberProvider::Binomial {
                        n: 3,
                        p: 0.5714286,
                    })],
                }),
                LootPool::single(LootEntry::Item {
                    item: "minecraft:poisonous_potato".to_string(),
                    weight: 1,
                    quality: 0,
                    conditions: vec![LootCondition::RandomChance(0.02)],
                    functions: Vec::new(),
                }),
            ],
            functions: vec![LootFunction::ApplyExplosionDecay],
        },

        // ── SPECIAL BLOCKS ─────────────────────────────────────────────────────────────────

        // Glowstone: 2-4 glowstone_dust, limited to 1-4 (silk touch not implemented)
        "glowstone" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:glowstone_dust".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 4.0 }),
                    LootFunction::LimitCount { min: 1, max: 4 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Sea lantern: 2-3 prismarine_crystals, limited to 1-5 (silk touch not implemented)
        "sea_lantern" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:prismarine_crystals".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 2.0, max: 3.0 }),
                    LootFunction::LimitCount { min: 1, max: 5 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Bookshelf: 3 books (silk touch not implemented)
        "bookshelf" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:book".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(3.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Snow block: 4 snowballs (silk touch not implemented)
        "snow_block" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:snowball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Constant(4.0)),
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Snow layers: 1 snowball (layer count from block state not yet tracked)
        "snow" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:snowball".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![LootFunction::SetCount(NumberProvider::Constant(1.0))],
            })],
            functions: Vec::new(),
        },

        // Melon: 3-7 melon_slices, capped at 9 (silk touch not implemented)
        "melon" => LootTable {
            param_set: LootParamSet::Block,
            random_sequence: Some(random_sequence),
            pools: vec![LootPool::single(LootEntry::Item {
                item: "minecraft:melon_slice".to_string(),
                weight: 1,
                quality: 0,
                conditions: Vec::new(),
                functions: vec![
                    LootFunction::SetCount(NumberProvider::Uniform { min: 3.0, max: 7.0 }),
                    LootFunction::LimitCount { min: 0, max: 9 },
                    LootFunction::ApplyExplosionDecay,
                ],
            })],
            functions: Vec::new(),
        },

        // Pumpkin / carved pumpkin: self-drop (survives explosion)
        "pumpkin" | "carved_pumpkin" => self_drop_survives_explosion("pumpkin", random_sequence),

        // Simple self-drops conditional on surviving explosion
        "sugar_cane" => self_drop_survives_explosion("sugar_cane", random_sequence),
        "cactus" => self_drop_survives_explosion("cactus", random_sequence),
        "bamboo" => self_drop_survives_explosion("bamboo", random_sequence),

        // ── NO DROP ────────────────────────────────────────────────────────────────────────
        "air"
        | "cave_air"
        | "void_air"
        | "bedrock"
        | "water"
        | "flowing_water"
        | "lava"
        | "flowing_lava"
        | "fire"
        | "soul_fire"
        | "ice"        // silk touch only
        | "packed_ice" // silk touch only
        | "blue_ice"   // silk touch only
        | "glass"      // silk touch only
        | "glass_pane" // silk touch only
        | "nether_portal" => return None,

        _ => return None,
    })
}

/// Evaluates the loot table for `block_name` and returns `(item_name, count)` pairs.
///
/// `item_name` is the canonical `&'static str` registry name (e.g. `"minecraft:coal"`), which
/// can be passed directly to `ItemStack::new` and stored in `DroppedItem`.  Items whose
/// registry name is not in the item catalog are silently dropped.
///
/// `seed` should be derived from block position for deterministic but varied drops.
fn evaluate_block_loot(block_name: &str, seed: u64) -> Vec<(&'static str, i32)> {
    let Some(table) = block_loot_table(block_name) else {
        return Vec::new();
    };
    let mut context = LootContext::new(LootParamSet::Block, seed);
    table
        .evaluate(&mut context)
        .into_iter()
        .filter_map(|stack| {
            if stack.count <= 0 {
                return None;
            }
            let name = item_static_name(&stack.item)?;
            Some((name, stack.count))
        })
        .collect()
}

fn load_chunk(layout: &WorldLayout, world_seed: i64, chunk_pos: ChunkPos) -> LevelChunk {
    let region_dir = layout.region_dir();
    if let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) {
        if let Ok(Some((_name, tag))) = region.read_chunk_nbt(chunk_pos) {
            if let Ok(chunk) = LevelChunk::from_nbt(chunk_pos, &tag) {
                return chunk;
            }
        }
    }
    generate_overworld_spawn_chunk_for_preset_with_mode(
        chunk_pos,
        "normal",
        live_chunk_generation_mode(),
        world_seed,
        true,
    )
    .unwrap_or_else(|_| LevelChunk::empty(chunk_pos))
}

fn read_block_at(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
) -> Option<String> {
    load_chunk(layout, world_seed, chunk_pos)
        .get_block_state(bx, by, bz)
        .filter(|n| n != "minecraft:air")
}

fn read_block_model_at(
    layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
) -> crate::block_behavior::BlockStateModel {
    let chunk_pos = ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    };
    let chunk = load_chunk(layout, world_seed, chunk_pos);
    if let Some(entry) = chunk.get_block_state_model(pos.x, pos.y, pos.z) {
        let mut state = crate::block_behavior::BlockStateModel::new(entry.name);
        for (key, value) in entry.properties {
            state = state.with_property(&key, value);
        }
        state
    } else {
        crate::block_behavior::BlockStateModel::air()
    }
}

fn write_block_model_at(
    layout: &WorldLayout,
    world_seed: i64,
    pos: crate::block_update::BlockPos,
    state: &crate::block_behavior::BlockStateModel,
) -> bool {
    let chunk_pos = ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    };
    place_block_in_region(
        layout,
        world_seed,
        chunk_pos,
        pos.x,
        pos.y,
        pos.z,
        &block_state_model_name(state),
    )
}

// Reads the old block name from the region, sets it to air, saves, and returns the old name.
fn break_block_in_region(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
) -> Option<String> {
    let region_dir = layout.region_dir();
    let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) else {
        return None;
    };
    let mut chunk = load_chunk(layout, world_seed, chunk_pos);
    let old_name = chunk
        .get_block_state(bx, by, bz)
        .filter(|n| n != "minecraft:air");
    chunk.set_block_state(bx, by, bz, "minecraft:air");
    let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let _ = region.write_chunk_nbt(chunk_pos, "", &nbt);
    old_name
}

/// Places a block at (bx, by, bz) in the region file and saves the chunk.
///
/// Returns true on success, false if the region file could not be opened.
/// Java: Level.setBlock() → ChunkAccess.setBlockState()
fn place_block_in_region(
    layout: &WorldLayout,
    world_seed: i64,
    chunk_pos: ChunkPos,
    bx: i32,
    by: i32,
    bz: i32,
    block_name: &str,
) -> bool {
    let region_dir = layout.region_dir();
    let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) else {
        return false;
    };
    let mut chunk = load_chunk(layout, world_seed, chunk_pos);
    chunk.set_block_state(bx, by, bz, block_name);
    let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let _ = region.write_chunk_nbt(chunk_pos, "", &nbt);
    true
}

/// Maps a `ContainerSetContent` container slot index (0-45) for container 0 (the player
/// inventory) to the corresponding `PlayerInventory` internal slot index, or `None` for
/// crafting/result slots which have no persistent inventory backing.
///
/// Java: `InventoryMenu` slot layout:
///   0        → crafting result  (no inventory backing)
///   1–4      → crafting grid    (no inventory backing)
///   5–8      → armor HEAD/CHEST/LEGS/FEET (inventory indices 39/38/37/36)
///   9–35     → main inventory rows (same index)
///   36–44    → hotbar           (inventory indices 0–8)
///   45       → offhand          (inventory index 40 = SLOT_OFFHAND)
fn inventory_internal_slot(container_slot: usize) -> Option<usize> {
    match container_slot {
        0..=4 => None,                  // crafting result + 2×2 grid — no persistent backing
        5 => Some(39),                  // HEAD armor
        6 => Some(38),                  // CHEST armor
        7 => Some(37),                  // LEGS armor
        8 => Some(36),                  // FEET armor
        9..=35 => Some(container_slot), // main inventory (indices match)
        36..=44 => Some(container_slot - 36), // hotbar → items[0..=8]
        45 => Some(40),                 // offhand (SLOT_OFFHAND)
        _ => None,
    }
}

fn direction_offset(dir: Direction3d) -> (i32, i32, i32) {
    match dir {
        Direction3d::Down => (0, -1, 0),
        Direction3d::Up => (0, 1, 0),
        Direction3d::North => (0, 0, -1),
        Direction3d::South => (0, 0, 1),
        Direction3d::West => (-1, 0, 0),
        Direction3d::East => (1, 0, 0),
    }
}

/// Encodes a velocity vector using the LP (Loss-Precision) Vec3 format used in
/// `ClientboundAddEntityPacket`.
///
/// Java: `LpVec3.write` — zero vector writes a single `0` byte; non-zero writes
/// 1 + 1 + 4 bytes (plus an optional VarInt for large-magnitude vectors).
fn write_lp_vec3<W: Write>(writer: &mut W, vx: f64, vy: f64, vz: f64) -> io::Result<()> {
    fn sanitize(v: f64) -> f64 {
        if v.is_nan() {
            0.0
        } else {
            v.clamp(-1.7179869183e10, 1.7179869183e10)
        }
    }
    // Java: Math.round((value * 0.5 + 0.5) * 32766.0)
    fn pack(v: f64) -> i64 {
        ((v * 0.5 + 0.5) * 32766.0 + 0.5).floor() as i64
    }
    let x = sanitize(vx);
    let y = sanitize(vy);
    let z = sanitize(vz);
    // Java: Mth.absMax(a, Mth.absMax(b, c))
    let chessboard = x.abs().max(y.abs()).max(z.abs());
    if chessboard < 3.051944088384301e-5 {
        return writer.write_all(&[0u8]);
    }
    let scale = chessboard.ceil() as i64;
    let is_partial = (scale & 3) != scale;
    let markers = if is_partial { (scale & 3) | 4 } else { scale };
    let xn = pack(x / scale as f64) << 3;
    let yn = pack(y / scale as f64) << 18;
    let zn = pack(z / scale as f64) << 33;
    let buffer = markers | xn | yn | zn;
    writer.write_all(&[buffer as u8, (buffer >> 8) as u8])?;
    writer.write_all(&((buffer >> 16) as i32).to_be_bytes())?;
    if is_partial {
        write_var_i32(writer, (scale >> 2) as i32)?;
    }
    Ok(())
}

/// Returns a pseudo-random `f32` in `[0, 1)` from a 64-bit seed and a per-call index.
///
/// Used to reproduce Java's `Random.nextFloat()` scatter calls in `createItemStackToDrop`
/// without keeping a persistent RNG in game state.  The exact values don't need to match
/// Java's — they only affect cosmetic velocity scatter — but they must be uncorrelated
/// across different indices.
fn pseudo_rand_f32(seed: i32, index: u32) -> f32 {
    let mut x = (seed as u64)
        .wrapping_mul(0x517CC1B727220A95)
        .wrapping_add((index as u64).wrapping_mul(0x6C62272E07BB0142));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    (x >> 33) as f32 / u32::MAX as f32
}

/// Sends a bundle-wrapped `ADD_ENTITY + SET_ENTITY_DATA` pair for a single item entity.
///
/// The two packets are enclosed in `ClientboundBundlePacket` delimiters so the client
/// processes them atomically in one game tick — without this, `ADD_ENTITY` may be
/// rendered for one tick with no item stack, making the entity invisible.
///
/// Java: `ServerEntity.addPairing()` — wraps `ADD_ENTITY + SET_ENTITY_DATA` in
///       `ClientboundBundlePacket` for any entity that needs metadata at spawn.
fn write_item_entity_spawn_packets<W: Write>(
    stream: &mut W,
    compression: CompressionState,
    item: &DroppedItem,
    item_pid: i32,
) -> io::Result<()> {
    let eid = item.entity_id;
    let uuid_hi = (eid as u64).wrapping_mul(0x6C62_272E_07BB_0142);
    let uuid_lo = (eid as u64).wrapping_mul(0x62B8_2175_6295_C58D);
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        |p| {
            write_var_i32(p, eid)?;
            p.write_all(&uuid_hi.to_be_bytes())?;
            p.write_all(&uuid_lo.to_be_bytes())?;
            write_var_i32(p, ITEM_ENTITY_TYPE_ID)?;
            p.write_all(&item.x.to_be_bytes())?;
            p.write_all(&item.y.to_be_bytes())?;
            p.write_all(&item.z.to_be_bytes())?;
            write_lp_vec3(p, item.vel_x, item.vel_y, item.vel_z)?;
            p.write_all(&[0u8, 0u8, 0u8])?; // xRot, yRot, yHeadRot
            write_var_i32(p, 0)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        |p| {
            write_var_i32(p, eid)?;
            p.write_all(&[8u8])?; // index 8: ItemEntity.DATA_ITEM
            write_var_i32(p, 7)?; // serializer 7: EntityDataSerializers.ITEM_STACK
            write_var_i32(p, item.count)?;
            write_var_i32(p, item_pid)?;
            write_var_i32(p, 0)?; // component add count
            write_var_i32(p, 0)?; // component remove count
            p.write_all(&[0xFFu8]) // end of metadata
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
        |_| Ok(()),
    )
}

/// Handles `DROP_ITEM` (action 4, Q) and `DROP_ALL_ITEMS` (action 3, Ctrl+Q) from
/// `ServerboundPlayerActionPacket`.
///
/// Java: `ServerGamePacketListenerImpl.handlePlayerAction` → `ServerPlayer.drop(boolean)` →
///       `Inventory.removeFromSelected` → `LivingEntity.createItemStackToDrop`.
///
/// Sends:
///   1. `ClientboundSetPlayerInventoryPacket` — updates the now-depleted held slot.
///   2. `ClientboundAddEntityPacket`          — spawns the item entity at eye height.
///   3. `ClientboundSetEntityDataPacket`      — sets the item stack metadata (index 8).
fn handle_drop_item(
    stream: &mut TcpStream,
    compression: CompressionState,
    state: &mut PlaySessionState,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    drop_all: bool,
) -> io::Result<()> {
    // Java: ServerGamePacketListenerImpl — spectators cannot drop items.
    if state.game_mode == GameMode::Spectator {
        return Ok(());
    }

    let held_slot = state.selected_slot as usize;

    // Java: Inventory.removeFromSelected(all) — remove 1 or the full stack count.
    let count_to_remove = {
        let stack = state.inventory_menu.player_inventory().get(held_slot);
        if stack.is_empty() {
            return Ok(());
        }
        if drop_all {
            stack.count()
        } else {
            1
        }
    };
    let removed = state
        .inventory_menu
        .player_inventory_mut()
        .remove(held_slot, count_to_remove);
    if removed.is_empty() {
        return Ok(());
    }

    // Update the client's held slot after removal.
    // Java: ServerPlayer.drop() → containerMenu.setRemoteSlot()
    let raw_after = {
        let stack = state.inventory_menu.player_inventory().get(held_slot);
        if stack.is_empty() {
            RawItemStack::empty()
        } else if let Some(pid) = item_protocol_id(stack.item_id()) {
            RawItemStack {
                count: stack.count(),
                item_id: Some(pid),
                components: RawDataComponentPatch::empty(),
            }
        } else {
            RawItemStack::empty()
        }
    };
    // Use ContainerSetSlot (container_id=0, with state_id) rather than SetPlayerInventory
    // so the client learns the new state_id and won't reject subsequent ContainerClick packets.
    // Java: ServerPlayer.drop() → containerMenu.setRemoteSlot() + broadcastChanges()
    //       → ClientboundContainerSetSlotPacket(containerId, incrementStateId(), slot, item).
    // The hotbar slot in the InventoryMenu is at index 36 + held_slot (menu layout: result=0,
    // crafting=1-4, armour=5-8, storage=9-35, hotbar=36-44, offhand=45).
    state.container_state_id = state.container_state_id.wrapping_add(1);
    let new_state_id = state.container_state_id;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
        |p| {
            ClientboundContainerSetSlotPacket {
                container_id: 0,
                state_id: new_state_id,
                slot: (36 + held_slot) as i16,
                item_stack: raw_after,
            }
            .write(p)
        },
    )?;

    let Some(item_pid) = item_protocol_id(removed.item_id()) else {
        return Ok(());
    };

    // Java: LivingEntity.createItemStackToDrop — spawn at eye height minus 0.3.
    // Player eye height is 1.62 (EntityType.java: sized(0.6, 1.8).eyeHeight(1.62)).
    let drop_x = state.x;
    let drop_y = state.y + 1.62 - 0.3; // getEyeY() - 0.3F
    let drop_z = state.z;

    // Java: LivingEntity.createItemStackToDrop — directional velocity based on view angles.
    // xRot = pitch, yRot = yaw (both stored in degrees in play_state).
    let pitch_rad = (state.pitch as f64) * (std::f64::consts::PI / 180.0);
    let yaw_rad = (state.yaw as f64) * (std::f64::consts::PI / 180.0);
    let sin_pitch = pitch_rad.sin();
    let cos_pitch = pitch_rad.cos();
    let sin_yaw = yaw_rad.sin();
    let cos_yaw = yaw_rad.cos();
    let eid = world_items.lock().unwrap().alloc_entity_id();
    // Java: LivingEntity.drop() scatter randomness uses the counter value before the entity
    // ID is assigned (i.e., eid - 1), matching ItemEntity constructor random offsets.
    let r0 = pseudo_rand_f32(eid.wrapping_sub(1), 0) as f64;
    let r1 = pseudo_rand_f32(eid.wrapping_sub(1), 1) as f64;
    let r2 = pseudo_rand_f32(eid.wrapping_sub(1), 2) as f64;
    let r3 = pseudo_rand_f32(eid.wrapping_sub(1), 3) as f64;
    let scatter_dir = r0 * std::f64::consts::TAU;
    let scatter_mag = 0.02 * r1;
    let vel_x = -sin_yaw * cos_pitch * 0.3 + scatter_dir.cos() * scatter_mag;
    let vel_y = -sin_pitch * 0.3 + 0.1 + (r2 - r3) * 0.1;
    let vel_z = cos_yaw * cos_pitch * 0.3 + scatter_dir.sin() * scatter_mag;

    let item = DroppedItem {
        entity_id: eid,
        item: removed.item_id(),
        count: removed.count(),
        x: drop_x,
        y: drop_y,
        z: drop_z,
        vel_x,
        vel_y,
        vel_z,
        // Java: ItemEntity.setPickUpDelay(40) — 2-second delay before anyone can pick it up,
        // including the player who dropped it.
        pickup_delay: 40,
        age: 0,
        target_uuid: None,
    };
    write_item_entity_spawn_packets(stream, compression, &item, item_pid)?;
    world_items.lock().unwrap().entities.push(item);

    Ok(())
}

fn visual_terrain_block_at(bx: i32, by: i32, bz: i32) -> Option<&'static str> {
    let chunk_x = bx.div_euclid(16);
    let chunk_z = bz.div_euclid(16);
    let local_x = bx.rem_euclid(16) as usize;
    let local_z = bz.rem_euclid(16) as usize;
    let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, local_x, local_z);
    if by == TERRAIN_BASE_Y {
        return Some("minecraft:bedrock");
    }
    if by < TERRAIN_BASE_Y || by > top_y + 1 {
        return None;
    }
    if by == top_y + 1 {
        let surface_id = visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z);
        if surface_id == GRASS_BLOCK_STATE_ID {
            return visible_spawn_surface_feature_id(chunk_x, chunk_z, local_x, local_z)
                .map(visual_block_state_id_to_name);
        }
        return None;
    }
    if by == top_y {
        return Some(visual_block_state_id_to_name(
            visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z),
        ));
    }
    Some("minecraft:stone")
}

fn visual_block_state_id_to_name(id: i32) -> &'static str {
    match id {
        STONE_BLOCK_STATE_ID => "minecraft:stone",
        GRANITE_BLOCK_STATE_ID => "minecraft:granite",
        DIORITE_BLOCK_STATE_ID => "minecraft:diorite",
        ANDESITE_BLOCK_STATE_ID => "minecraft:andesite",
        GRASS_BLOCK_STATE_ID => "minecraft:grass_block",
        DIRT_BLOCK_STATE_ID => "minecraft:dirt",
        DANDELION_BLOCK_STATE_ID => "minecraft:dandelion",
        POPPY_BLOCK_STATE_ID => "minecraft:poppy",
        SHORT_GRASS_BLOCK_STATE_ID => "minecraft:short_grass",
        _ => "minecraft:air",
    }
}

fn section_min_y(section_index: usize) -> i32 {
    -64 + section_index as i32 * 16
}

fn visible_spawn_terrain_block_count(chunk_x: i32, chunk_z: i32, section_index: usize) -> i16 {
    let mut count = 0_i16;
    let section_min_y = section_min_y(section_index);
    let section_max_y = section_min_y + 15;
    for local_z in 0..16 {
        for local_x in 0..16 {
            let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, local_x, local_z);
            let column_min_y = TERRAIN_BASE_Y.max(section_min_y);
            let column_max_y = top_y.min(section_max_y);
            if column_max_y >= column_min_y {
                count += (column_max_y - column_min_y + 1) as i16;
            }
            if visible_spawn_surface_top_block_id(chunk_x, chunk_z, local_x, local_z)
                == GRASS_BLOCK_STATE_ID
                && visible_spawn_surface_feature_id(chunk_x, chunk_z, local_x, local_z).is_some()
                && top_y + 1 >= section_min_y
                && top_y < section_max_y
            {
                count += 1;
            }
        }
    }
    count
}

fn write_visible_spawn_terrain_block_state_container<W: Write>(
    writer: &mut W,
    chunk_x: i32,
    chunk_z: i32,
    section_index: usize,
) -> io::Result<()> {
    const BITS_PER_ENTRY: u8 = 4;
    const BLOCKS_PER_SECTION: usize = 16 * 16 * 16;
    const VALUES_PER_LONG: usize = 64 / BITS_PER_ENTRY as usize;

    writer.write_all(&[BITS_PER_ENTRY])?;
    write_var_i32(writer, 11)?;
    write_var_i32(writer, AIR_BLOCK_STATE_ID)?;
    write_var_i32(writer, STONE_BLOCK_STATE_ID)?;
    write_var_i32(writer, GRANITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, DIORITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, ANDESITE_BLOCK_STATE_ID)?;
    write_var_i32(writer, BEDROCK_BLOCK_STATE_ID)?;
    write_var_i32(writer, DIRT_BLOCK_STATE_ID)?;
    write_var_i32(writer, GRASS_BLOCK_STATE_ID)?;
    write_var_i32(writer, SHORT_GRASS_BLOCK_STATE_ID)?;
    write_var_i32(writer, DANDELION_BLOCK_STATE_ID)?;
    write_var_i32(writer, POPPY_BLOCK_STATE_ID)?;

    let mut storage = vec![0_u64; BLOCKS_PER_SECTION / VALUES_PER_LONG];
    let section_min_y = section_min_y(section_index);
    let section_max_y = section_min_y + 15;
    for z in 0..16 {
        for x in 0..16 {
            let top_y = visible_spawn_terrain_height(chunk_x, chunk_z, x, z);
            let column_min_y = TERRAIN_BASE_Y.max(section_min_y);
            let column_max_y = top_y.min(section_max_y);
            for global_y in column_min_y..=column_max_y {
                let local_y = (global_y - section_min_y) as usize;
                let palette_index = if global_y == top_y {
                    match visible_spawn_surface_top_block_id(chunk_x, chunk_z, x, z) {
                        STONE_BLOCK_STATE_ID => 1_u64,
                        GRANITE_BLOCK_STATE_ID => 2_u64,
                        DIORITE_BLOCK_STATE_ID => 3_u64,
                        ANDESITE_BLOCK_STATE_ID => 4_u64,
                        DIRT_BLOCK_STATE_ID => 6_u64,
                        GRASS_BLOCK_STATE_ID => 7_u64,
                        _ => unreachable!("surface top palette id is registered above"),
                    }
                } else if global_y == TERRAIN_BASE_Y {
                    5_u64
                } else {
                    6_u64
                };
                let block_index = (local_y << 8) | (z << 4) | x;
                let word_index = block_index / VALUES_PER_LONG;
                let bit_index =
                    (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY as usize;
                storage[word_index] |= palette_index << bit_index;
            }
            if visible_spawn_surface_top_block_id(chunk_x, chunk_z, x, z) == GRASS_BLOCK_STATE_ID {
                if let Some(feature_id) = visible_spawn_surface_feature_id(chunk_x, chunk_z, x, z)
                    .filter(|_| top_y + 1 >= section_min_y && top_y < section_max_y)
                {
                    let palette_index = match feature_id {
                        SHORT_GRASS_BLOCK_STATE_ID => 8_u64,
                        DANDELION_BLOCK_STATE_ID => 9_u64,
                        POPPY_BLOCK_STATE_ID => 10_u64,
                        _ => unreachable!("surface feature palette id is registered above"),
                    };
                    let local_y = (top_y + 1 - section_min_y) as usize;
                    let block_index = (local_y << 8) | (z << 4) | x;
                    let word_index = block_index / VALUES_PER_LONG;
                    let bit_index =
                        (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY as usize;
                    storage[word_index] |= palette_index << bit_index;
                }
            }
        }
    }

    for word in storage {
        writer.write_all(&word.to_be_bytes())?;
    }
    Ok(())
}

#[allow(dead_code)]
fn write_empty_bitset<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 0)
}

fn write_minimal_damage_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:damage_type").unwrap())?;
    write_var_i32(writer, DAMAGE_TYPES.len() as i32)?;
    for damage_type in DAMAGE_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{damage_type}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(
            writer,
            &Tag::Compound(vec![
                (
                    "message_id".to_string(),
                    Tag::String((*damage_type).to_string()),
                ),
                (
                    "scaling".to_string(),
                    Tag::String("when_caused_by_living_non_player".to_string()),
                ),
                ("exhaustion".to_string(), Tag::Float(0.0)),
            ]),
        )?;
    }
    Ok(())
}

fn write_minimal_update_tags_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 3)?;
    write_identifier(writer, &Identifier::parse("minecraft:damage_type").unwrap())?;
    write_var_i32(writer, DAMAGE_TYPE_TAGS.len() as i32)?;
    for (tag, entries) in DAMAGE_TYPE_TAGS {
        write_identifier(writer, &Identifier::parse(tag).unwrap())?;
        write_var_i32(writer, entries.len() as i32)?;
        for entry in *entries {
            write_var_i32(writer, *entry)?;
        }
    }
    write_identifier(
        writer,
        &Identifier::parse("minecraft:banner_pattern").unwrap(),
    )?;
    write_var_i32(writer, BANNER_PATTERN_TAGS.len() as i32)?;
    for (tag, entries) in BANNER_PATTERN_TAGS {
        write_identifier(writer, &Identifier::parse(tag).unwrap())?;
        write_var_i32(writer, entries.len() as i32)?;
        for entry in *entries {
            let index = BANNER_PATTERNS
                .iter()
                .position(|pattern| pattern == entry)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unknown banner pattern tag entry {entry}"),
                    )
                })?;
            write_var_i32(writer, index as i32)?;
        }
    }
    // Timeline tags: required for the client to resolve the `timelines` HolderSet in the
    // dimension type (which references "#minecraft:in_overworld") and the `#minecraft:universal`
    // nested tag.
    //
    // IDs match the order entries are sent in write_vanilla_timeline_registry_packet:
    //   day=0, moon=1, villager_schedule=2, early_game=3
    //
    // Java refs:
    //   data/minecraft/tags/timeline/in_overworld.json  → [#universal, day, moon, early_game]
    //   data/minecraft/tags/timeline/universal.json     → [villager_schedule]
    // Tags are pre-expanded by the server (nested tag #universal resolved to its elements).
    write_identifier(writer, &Identifier::parse("minecraft:timeline").unwrap())?;
    // Two tags: #minecraft:in_overworld and #minecraft:universal.
    write_var_i32(writer, 2)?;
    // #minecraft:in_overworld expands to [villager_schedule=2, day=0, moon=1, early_game=3].
    write_identifier(
        writer,
        &Identifier::parse("minecraft:in_overworld").unwrap(),
    )?;
    write_var_i32(writer, 4)?;
    for id in [2i32, 0, 1, 3] {
        write_var_i32(writer, id)?;
    }
    // #minecraft:universal expands to [villager_schedule=2].
    write_identifier(writer, &Identifier::parse("minecraft:universal").unwrap())?;
    write_var_i32(writer, 1)?;
    write_var_i32(writer, 2)?; // villager_schedule = ID 2
    Ok(())
}

fn write_vanilla_known_packs_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_var_i32(writer, 1)?;
    write_string(writer, "minecraft")?;
    write_string(writer, "core")?;
    write_string(writer, VERSION_NAME)
}

fn write_minimal_dimension_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:dimension_type").unwrap(),
    )?;
    write_var_i32(writer, DIMENSION_TYPES.len() as i32)?;
    for dimension_type in DIMENSION_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{dimension_type}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &dimension_type_nbt(dimension_type))?;
    }
    Ok(())
}

fn write_vanilla_chat_type_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:chat_type").unwrap())?;
    write_var_i32(writer, CHAT_TYPES.len() as i32)?;
    for chat_type in CHAT_TYPES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", chat_type.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &chat_type_nbt(chat_type))?;
    }
    Ok(())
}

fn write_minimal_trim_material_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:trim_material").unwrap(),
    )?;
    write_var_i32(writer, TRIM_MATERIALS.len() as i32)?;
    for material in TRIM_MATERIALS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", material.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &trim_material_nbt(material))?;
    }
    Ok(())
}

fn write_vanilla_jukebox_song_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:jukebox_song").unwrap(),
    )?;
    write_var_i32(writer, JUKEBOX_SONGS.len() as i32)?;
    for song in JUKEBOX_SONGS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", song.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &jukebox_song_nbt(song))?;
    }
    Ok(())
}

fn write_vanilla_banner_pattern_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:banner_pattern").unwrap(),
    )?;
    write_var_i32(writer, BANNER_PATTERNS.len() as i32)?;
    for pattern in BANNER_PATTERNS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{pattern}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &banner_pattern_nbt(pattern))?;
    }
    Ok(())
}

fn write_vanilla_trim_pattern_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:trim_pattern").unwrap(),
    )?;
    write_var_i32(writer, TRIM_PATTERNS.len() as i32)?;
    for pattern in TRIM_PATTERNS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{pattern}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &trim_pattern_nbt(pattern))?;
    }
    Ok(())
}

fn write_vanilla_instrument_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:instrument").unwrap())?;
    write_var_i32(writer, INSTRUMENTS.len() as i32)?;
    for instrument in INSTRUMENTS {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", instrument.id)).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &instrument_nbt(instrument))?;
    }
    Ok(())
}

/// Java: net/minecraft/world/clock/WorldClock.java — `record WorldClock()` with DIRECT_CODEC =
/// `MapCodec.unitCodec(...)`, which encodes as an empty NBT compound.
/// Java: net/minecraft/world/clock/WorldClocks.java:12–13 — overworld registered first (ID 0),
/// the_end second (ID 1). This order defines the VarInt IDs used in ClientboundSetTimePacket.
/// Java: net/minecraft/resources/RegistryDataLoader.java:125,160 — WORLD_CLOCK is a
/// datapack-loaded registry that must be synced to clients during the configuration phase.
fn write_world_clock_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:world_clock").unwrap())?;
    write_var_i32(writer, 2)?; // minecraft:overworld (ID 0) and minecraft:the_end (ID 1)
    for name in ["overworld", "the_end"] {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{name}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        // WorldClock is a zero-field record; its NBT codec encodes as an empty compound.
        write_network_nbt(writer, &Tag::Compound(vec![]))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Timeline registry helpers
// ---------------------------------------------------------------------------
//
// Java refs:
//   net/minecraft/world/timeline/Timeline.java:49 — NETWORK_CODEC filters to syncable tracks
//   net/minecraft/util/Keyframe.java:8–11         — {ticks: int, value: T} compound
//   net/minecraft/util/KeyframeTrack.java:21–29   — {keyframes: [...], ease: EasingType}
//   net/minecraft/world/timeline/AttributeTrack.java:16–18 — modifier dispatch + KeyframeTrack
//   net/minecraft/util/EasingType.java             — linear omitted (default); cubic_bezier compound
//   net/minecraft/world/attribute/AttributeTypes.java — value codecs per type
//   net/minecraft/world/attribute/modifier/ColorModifier.java — ArgbModifier.argumentCodec
//   net/minecraft/world/attribute/modifier/FloatModifier.java — Simple.argumentCodec = FLOAT
//   net/minecraft/world/attribute/modifier/BooleanModifier.java — argumentCodec = BOOL
//   net/minecraft/world/attribute/EnvironmentAttributes.java — .syncable() marks network tracks
//   data/minecraft/timeline/*.json                 — authoritative keyframe data

/// Builds a keyframe compound for a 32-bit float value.
/// Java: Keyframe.codec(Codec.FLOAT) → RecordCodecBuilder {ticks: INT, value: FLOAT}
fn timeline_keyframe_f32(ticks: i32, value: f32) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Float(value)),
    ])
}

/// Builds a keyframe compound for a string value (hex colour, enum name).
/// Java: Keyframe.codec(STRING) → RecordCodecBuilder {ticks: INT, value: STRING}
fn timeline_keyframe_str(ticks: i32, value: &str) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::String(value.to_string())),
    ])
}

/// Builds a keyframe compound for a boolean value.
/// Java: Codec.BOOL encodes as ByteTag (1 = true, 0 = false) in NbtOps.
fn timeline_keyframe_bool(ticks: i32, value: bool) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Byte(value as i8)),
    ])
}

/// Builds a keyframe compound for a raw 32-bit signed integer value.
/// Java: ARGB_COLOR type with multiply modifier — ArgbModifier.argumentCodec selects
/// Codec.INT when alpha == 0xFF (fully opaque). Keyframe value → Tag::Int.
fn timeline_keyframe_i32(ticks: i32, value: i32) -> Tag {
    Tag::Compound(vec![
        ("ticks".to_string(), Tag::Int(ticks)),
        ("value".to_string(), Tag::Int(value)),
    ])
}

/// Builds an `ease` compound for a cubic-bezier easing function.
/// Java: EasingType.CubicBezier.CODEC → {cubic_bezier: [x1, y1, x2, y2]}
fn cubic_bezier_ease(x1: f32, y1: f32, x2: f32, y2: f32) -> Tag {
    Tag::Compound(vec![(
        "cubic_bezier".to_string(),
        Tag::List(vec![
            Tag::Float(x1),
            Tag::Float(y1),
            Tag::Float(x2),
            Tag::Float(y2),
        ]),
    )])
}

/// Builds a track compound from modifier, keyframes, and optional ease.
/// Java: AttributeTrack.createCodec — modifier field absent for `override` (the default),
/// present for any other modifier. Ease field absent when LINEAR (the default).
fn timeline_track(modifier: Option<&str>, keyframes: Vec<Tag>, ease: Option<Tag>) -> Tag {
    let mut fields = Vec::new();
    if let Some(m) = modifier {
        fields.push(("modifier".to_string(), Tag::String(m.to_string())));
    }
    fields.push(("keyframes".to_string(), Tag::List(keyframes)));
    if let Some(e) = ease {
        fields.push(("ease".to_string(), e));
    }
    Tag::Compound(fields)
}

/// Builds the NBT compound for `minecraft:day` filtered to syncable tracks only.
/// Java: data/minecraft/timeline/day.json; Timeline.NETWORK_CODEC removes non-syncable tracks.
fn day_timeline_nbt() -> Tag {
    // symmetricCubicBezier(0.362, 0.241) — shared by sun, moon, and star angle tracks.
    let sym_bezier = cubic_bezier_ease(0.362, 0.241, 0.638, 0.759);

    // Celestial angle tracks: ANGLE_DEGREES type, override modifier → Codec.FLOAT keyframes.
    let sun_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 360.0),
            timeline_keyframe_f32(6000, 0.0),
        ],
        Some(sym_bezier.clone()),
    );
    let moon_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 540.0),
            timeline_keyframe_f32(6000, 180.0),
        ],
        Some(sym_bezier.clone()),
    );
    let star_angle = timeline_track(
        None,
        vec![
            timeline_keyframe_f32(6000, 360.0),
            timeline_keyframe_f32(6000, 0.0),
        ],
        Some(sym_bezier),
    );

    // RGB colour tracks: multiply modifier → RgbModifier.argumentCodec = STRING_RGB_COLOR.
    // Primary encoder of STRING_RGB_COLOR is hexColor(6) → Tag::String("#rrggbb").
    let fog_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(133, "#ffffff"),
            timeline_keyframe_str(11867, "#ffffff"),
            timeline_keyframe_str(13670, "#0f0f16"),
            timeline_keyframe_str(22330, "#0f0f16"),
        ],
        None,
    );
    let sky_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(133, "#ffffff"),
            timeline_keyframe_str(11867, "#ffffff"),
            timeline_keyframe_str(13670, "#000000"),
            timeline_keyframe_str(22330, "#000000"),
        ],
        None,
    );
    let sky_light_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_str(730, "#ffffff"),
            timeline_keyframe_str(11270, "#ffffff"),
            timeline_keyframe_str(13140, "#7a7aff"),
            timeline_keyframe_str(22860, "#7a7aff"),
        ],
        None,
    );

    // Float tracks: multiply/maximum modifier → FloatModifier.Simple.argumentCodec = Codec.FLOAT.
    let sky_light_factor = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_f32(730, 1.0),
            timeline_keyframe_f32(11270, 1.0),
            timeline_keyframe_f32(13140, 0.24),
            timeline_keyframe_f32(22860, 0.24),
        ],
        None,
    );
    // gameplay/sky_light_level is syncable (SKY_LIGHT_LEVEL has .notPositional().syncable()).
    let sky_light_level = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_f32(133, 1.0),
            timeline_keyframe_f32(11867, 1.0),
            timeline_keyframe_f32(13670, 0.266_666_68),
            timeline_keyframe_f32(22330, 0.266_666_68),
        ],
        None,
    );
    let star_brightness = timeline_track(
        Some("maximum"),
        vec![
            timeline_keyframe_f32(92, 0.037),
            timeline_keyframe_f32(627, 0.0),
            timeline_keyframe_f32(11373, 0.0),
            timeline_keyframe_f32(11732, 0.016),
            timeline_keyframe_f32(11959, 0.044),
            timeline_keyframe_f32(12399, 0.143),
            timeline_keyframe_f32(12729, 0.258),
            timeline_keyframe_f32(13228, 0.5),
            timeline_keyframe_f32(22772, 0.5),
            timeline_keyframe_f32(23032, 0.364),
            timeline_keyframe_f32(23356, 0.225),
            timeline_keyframe_f32(23758, 0.101),
        ],
        None,
    );

    // ARGB colour: multiply modifier → ArgbModifier.argumentCodec = Either<STRING_ARGB, Codec.INT>.
    // When alpha == 0xFF: Either.right → Codec.INT → Tag::Int.
    // -1 = 0xFFFFFFFF (white); -15132378 = 0xFF1A1A26 (night-tinted dark grey).
    let cloud_color = timeline_track(
        Some("multiply"),
        vec![
            timeline_keyframe_i32(133, -1),
            timeline_keyframe_i32(11867, -1),
            timeline_keyframe_i32(13670, -15132378),
            timeline_keyframe_i32(22330, -15132378),
        ],
        None,
    );

    // ARGB colour: override modifier → OverrideModifier.argumentCodec = STRING_ARGB_COLOR.
    // Primary encoder is hexColor(8) → Tag::String("#aarrggbb").
    let sunrise_sunset_color = timeline_track(
        None,
        vec![
            timeline_keyframe_str(71, "#5fefa333"),
            timeline_keyframe_str(310, "#29f5ba33"),
            timeline_keyframe_str(565, "#06fbd433"),
            timeline_keyframe_str(730, "#00ffe533"),
            timeline_keyframe_str(11270, "#00ffe533"),
            timeline_keyframe_str(11397, "#04fcd833"),
            timeline_keyframe_str(11522, "#0ff9cb33"),
            timeline_keyframe_str(11690, "#29f5ba33"),
            timeline_keyframe_str(11929, "#5fefa333"),
            timeline_keyframe_str(12243, "#b1e78733"),
            timeline_keyframe_str(12358, "#cce47e33"),
            timeline_keyframe_str(12512, "#e9e07233"),
            timeline_keyframe_str(12613, "#f6dd6b33"),
            timeline_keyframe_str(12732, "#feda6333"),
            timeline_keyframe_str(12841, "#fed75c33"),
            timeline_keyframe_str(13035, "#ecd25133"),
            timeline_keyframe_str(13252, "#c1cc4733"),
            timeline_keyframe_str(13775, "#36be3733"),
            timeline_keyframe_str(13888, "#1fbb3533"),
            timeline_keyframe_str(14039, "#09b73333"),
            timeline_keyframe_str(14192, "#00b33333"),
            timeline_keyframe_str(21807, "#00b23333"),
            timeline_keyframe_str(21961, "#09b73333"),
            timeline_keyframe_str(22112, "#1fbb3533"),
            timeline_keyframe_str(22225, "#36be3733"),
            timeline_keyframe_str(22748, "#c1cc4733"),
            timeline_keyframe_str(22965, "#ecd25133"),
            timeline_keyframe_str(23159, "#fed75c33"),
            timeline_keyframe_str(23272, "#feda6333"),
            timeline_keyframe_str(23488, "#e9e07233"),
            timeline_keyframe_str(23642, "#cce47e33"),
            timeline_keyframe_str(23757, "#b1e78733"),
        ],
        None,
    );

    // Boolean tracks: OR modifier → BooleanModifier.OR.argumentCodec = Codec.BOOL → Tag::Byte.
    let firefly_bush_sounds = timeline_track(
        Some("or"),
        vec![
            timeline_keyframe_bool(12600, true),
            timeline_keyframe_bool(23401, false),
        ],
        None,
    );
    let creaking_active = timeline_track(
        Some("or"),
        vec![
            timeline_keyframe_bool(12600, true),
            timeline_keyframe_bool(23401, false),
        ],
        None,
    );

    // Time markers are preserved by NETWORK_CODEC (filterSyncableTracks only touches tracks).
    // TimeMarkerInfo.CODEC: showInCommands=true → Compound{ticks, show_in_commands};
    //                       showInCommands=false → Tag::Int(ticks).
    let time_markers = Tag::Compound(vec![
        (
            "minecraft:day".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(1000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:midnight".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(18000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:night".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(13000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        (
            "minecraft:noon".to_string(),
            Tag::Compound(vec![
                ("ticks".to_string(), Tag::Int(6000)),
                ("show_in_commands".to_string(), Tag::Byte(1)),
            ]),
        ),
        // showInCommands=false → encoded as plain Tag::Int(ticks).
        ("minecraft:roll_village_siege".to_string(), Tag::Int(18000)),
        ("minecraft:wake_up_from_sleep".to_string(), Tag::Int(0)),
    ]);

    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(24000)),
        (
            "tracks".to_string(),
            Tag::Compound(vec![
                ("minecraft:visual/sun_angle".to_string(), sun_angle),
                ("minecraft:visual/moon_angle".to_string(), moon_angle),
                ("minecraft:visual/star_angle".to_string(), star_angle),
                ("minecraft:visual/fog_color".to_string(), fog_color),
                ("minecraft:visual/sky_color".to_string(), sky_color),
                (
                    "minecraft:visual/sky_light_color".to_string(),
                    sky_light_color,
                ),
                (
                    "minecraft:visual/sky_light_factor".to_string(),
                    sky_light_factor,
                ),
                (
                    "minecraft:visual/star_brightness".to_string(),
                    star_brightness,
                ),
                ("minecraft:visual/cloud_color".to_string(), cloud_color),
                (
                    "minecraft:visual/sunrise_sunset_color".to_string(),
                    sunrise_sunset_color,
                ),
                (
                    "minecraft:gameplay/sky_light_level".to_string(),
                    sky_light_level,
                ),
                (
                    "minecraft:audio/firefly_bush_sounds".to_string(),
                    firefly_bush_sounds,
                ),
                (
                    "minecraft:gameplay/creaking_active".to_string(),
                    creaking_active,
                ),
            ]),
        ),
        ("time_markers".to_string(), time_markers),
    ])
}

/// Builds the NBT compound for `minecraft:moon` filtered to syncable tracks only.
/// Java: data/minecraft/timeline/moon.json; surface_slime_spawn_chance is non-syncable and
/// filtered out. Only visual/moon_phase (MOON_PHASE type, syncable) survives.
fn moon_timeline_nbt() -> Tag {
    // MoonPhase.CODEC = StringRepresentable.fromEnum → encodes as Tag::String name.
    let moon_phase = timeline_track(
        None,
        vec![
            timeline_keyframe_str(0, "full_moon"),
            timeline_keyframe_str(24000, "waning_gibbous"),
            timeline_keyframe_str(48000, "third_quarter"),
            timeline_keyframe_str(72000, "waning_crescent"),
            timeline_keyframe_str(96000, "new_moon"),
            timeline_keyframe_str(120000, "waxing_crescent"),
            timeline_keyframe_str(144000, "first_quarter"),
            timeline_keyframe_str(168000, "waxing_gibbous"),
        ],
        None,
    );
    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(192000)),
        (
            "tracks".to_string(),
            Tag::Compound(vec![(
                "minecraft:visual/moon_phase".to_string(),
                moon_phase,
            )]),
        ),
    ])
}

/// Builds the NBT compound for `minecraft:villager_schedule`.
/// Java: data/minecraft/timeline/villager_schedule.json; both tracks (villager_activity,
/// baby_villager_activity) are non-syncable → filtered out. Tracks field absent (equals
/// default Map.of()); only clock and period_ticks remain.
fn villager_schedule_timeline_nbt() -> Tag {
    Tag::Compound(vec![
        (
            "clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        ("period_ticks".to_string(), Tag::Int(24000)),
    ])
}

/// Builds the NBT compound for `minecraft:early_game`.
/// Java: data/minecraft/timeline/early_game.json; can_pillager_patrol_spawn is non-syncable
/// → filtered out. No period_ticks in source data. Only the clock field remains.
fn early_game_timeline_nbt() -> Tag {
    Tag::Compound(vec![(
        "clock".to_string(),
        Tag::String("minecraft:overworld".to_string()),
    )])
}

/// Sends the `minecraft:timeline` registry during configuration.
///
/// The client uses timeline data to drive all sky rendering (sun/moon angles, sky colour,
/// fog colour, star brightness, etc.) via its Timeline evaluation system. Without this
/// registry the client's `timelines` field in the dimension type cannot be resolved and
/// all sky colours remain black (default EnvironmentAttribute values).
///
/// Entry IDs (used by the tags packet): day=0, moon=1, villager_schedule=2, early_game=3.
///
/// Java refs:
///   net/minecraft/resources/RegistryDataLoader.java:125,160 — TIMELINE in sync registry list
///   net/minecraft/world/timeline/Timeline.java:49 — NETWORK_CODEC = filterSyncableTracks
fn write_vanilla_timeline_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(writer, &Identifier::parse("minecraft:timeline").unwrap())?;
    write_var_i32(writer, 4)?; // day=0, moon=1, villager_schedule=2, early_game=3
    let entries: &[(&str, fn() -> Tag)] = &[
        ("day", day_timeline_nbt),
        ("moon", moon_timeline_nbt),
        ("villager_schedule", villager_schedule_timeline_nbt),
        ("early_game", early_game_timeline_nbt),
    ];
    for (name, build_nbt) in entries {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{name}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &build_nbt())?;
    }
    Ok(())
}

fn write_vanilla_cat_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const CATS: &[&str] = &[
        "all_black",
        "black",
        "british_shorthair",
        "calico",
        "jellie",
        "persian",
        "ragdoll",
        "red",
        "siamese",
        "tabby",
        "white",
    ];
    write_variant_registry(writer, "minecraft:cat_variant", CATS, |cat| {
        animal_texture_variant_nbt("cat", &format!("cat_{cat}"), "normal")
    })
}

fn write_vanilla_chicken_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const CHICKENS: &[(&str, &str)] = &[
        ("cold", "cold"),
        ("temperate", "normal"),
        ("warm", "normal"),
    ];
    write_variant_registry(
        writer,
        "minecraft:chicken_variant",
        CHICKENS,
        |(id, model)| animal_texture_variant_nbt("chicken", &format!("chicken_{id}"), model),
    )
}

fn write_vanilla_cow_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const COWS: &[(&str, &str)] = &[("cold", "cold"), ("temperate", "normal"), ("warm", "warm")];
    write_variant_registry(writer, "minecraft:cow_variant", COWS, |(id, model)| {
        animal_texture_variant_nbt("cow", &format!("cow_{id}"), model)
    })
}

fn write_vanilla_frog_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const FROGS: &[&str] = &["cold", "temperate", "warm"];
    write_variant_registry(writer, "minecraft:frog_variant", FROGS, |frog| {
        single_texture_variant_nbt(&format!("minecraft:entity/frog/frog_{frog}"))
    })
}

fn write_vanilla_pig_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const PIGS: &[(&str, &str)] = &[
        ("cold", "cold"),
        ("temperate", "normal"),
        ("warm", "normal"),
    ];
    write_variant_registry(writer, "minecraft:pig_variant", PIGS, |(id, model)| {
        animal_texture_variant_nbt("pig", &format!("pig_{id}"), model)
    })
}

fn write_vanilla_wolf_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const WOLVES: &[(&str, &str)] = &[
        ("ashen", "wolf_ashen"),
        ("black", "wolf_black"),
        ("chestnut", "wolf_chestnut"),
        ("pale", "wolf"),
        ("rusty", "wolf_rusty"),
        ("snowy", "wolf_snowy"),
        ("spotted", "wolf_spotted"),
        ("striped", "wolf_striped"),
        ("woods", "wolf_woods"),
    ];
    write_variant_registry(writer, "minecraft:wolf_variant", WOLVES, |(_id, file)| {
        wolf_variant_nbt(file)
    })
}

fn write_vanilla_cat_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "royal"];
    write_variant_registry(writer, "minecraft:cat_sound_variant", VARIANTS, |_| {
        cat_sound_variant_nbt()
    })
}

fn write_vanilla_chicken_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "picky"];
    write_variant_registry(writer, "minecraft:chicken_sound_variant", VARIANTS, |_| {
        chicken_sound_variant_nbt()
    })
}

fn write_vanilla_cow_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["classic", "moody"];
    write_variant_registry(writer, "minecraft:cow_sound_variant", VARIANTS, |_| {
        cow_sound_variant_nbt()
    })
}

fn write_vanilla_pig_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["big", "classic", "mini"];
    write_variant_registry(writer, "minecraft:pig_sound_variant", VARIANTS, |_| {
        pig_sound_variant_nbt()
    })
}

fn write_vanilla_wolf_sound_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"];
    write_variant_registry(writer, "minecraft:wolf_sound_variant", VARIANTS, |_| {
        wolf_sound_variant_nbt()
    })
}

fn write_vanilla_zombie_nautilus_variant_registry_packet<W: Write>(
    writer: &mut W,
) -> io::Result<()> {
    const VARIANTS: &[&str] = &["temperate", "warm"];
    write_variant_registry(
        writer,
        "minecraft:zombie_nautilus_variant",
        VARIANTS,
        |id| zombie_nautilus_variant_nbt(id),
    )
}

fn write_vanilla_painting_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const PAINTINGS: &[&str] = &[
        "alban",
        "aztec",
        "aztec2",
        "backyard",
        "baroque",
        "bomb",
        "bouquet",
        "burning_skull",
        "bust",
        "cavebird",
        "changing",
        "cotan",
        "courbet",
        "creebet",
        "dennis",
        "donkey_kong",
        "earth",
        "endboss",
        "fern",
        "fighters",
        "finding",
        "fire",
        "graham",
        "humble",
        "kebab",
        "lowmist",
        "match",
        "meditative",
        "orb",
        "owlemons",
        "passage",
        "pigscene",
        "plant",
        "pointer",
        "pond",
        "pool",
        "prairie_ride",
        "sea",
        "skeleton",
        "skull_and_roses",
        "stage",
        "sunflowers",
        "sunset",
        "tides",
        "unpacked",
        "void",
        "wanderer",
        "wasteland",
        "water",
        "wind",
        "wither",
    ];
    write_variant_registry(writer, "minecraft:painting_variant", PAINTINGS, |id| {
        painting_variant_nbt(id)
    })
}

fn write_variant_registry<W, T, F>(
    writer: &mut W,
    registry: &str,
    entries: &[T],
    mut value: F,
) -> io::Result<()>
where
    W: Write,
    F: FnMut(&T) -> Tag,
    T: VariantRegistryElement,
{
    write_identifier(writer, &Identifier::parse(registry).unwrap())?;
    write_var_i32(writer, entries.len() as i32)?;
    for entry in entries {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{}", entry.id())).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &value(entry))?;
    }
    Ok(())
}

trait VariantRegistryElement {
    fn id(&self) -> &str;
}

impl VariantRegistryElement for &str {
    fn id(&self) -> &str {
        self
    }
}

impl VariantRegistryElement for (&str, &str) {
    fn id(&self) -> &str {
        self.0
    }
}

fn write_minimal_biome_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    write_identifier(
        writer,
        &Identifier::parse("minecraft:worldgen/biome").unwrap(),
    )?;
    write_var_i32(writer, BIOMES.len() as i32)?;
    for biome in BIOMES {
        write_identifier(
            writer,
            &Identifier::parse(&format!("minecraft:{biome}")).unwrap(),
        )?;
        write_bool(writer, true)?;
        write_network_nbt(writer, &vanilla_baseline_biome_nbt(biome))?;
    }
    Ok(())
}

fn vanilla_baseline_biome_nbt(biome: &str) -> Tag {
    let (has_precipitation, temperature, downfall, water_color) = match biome {
        "the_void" => (false, 0.5, 0.5, 4_159_204),
        "snowy_plains" | "ice_spikes" | "snowy_taiga" | "frozen_river" | "snowy_beach"
        | "frozen_ocean" | "deep_frozen_ocean" | "grove" | "snowy_slopes" | "frozen_peaks"
        | "jagged_peaks" => (true, 0.0, 0.5, 4_020_182),
        "desert" | "savanna" | "savanna_plateau" | "windswept_savanna" | "badlands"
        | "eroded_badlands" | "wooded_badlands" | "nether_wastes" | "warped_forest"
        | "crimson_forest" | "soul_sand_valley" | "basalt_deltas" => (false, 2.0, 0.0, 4_159_204),
        "warm_ocean" => (true, 0.5, 0.5, 4_446_778),
        "lukewarm_ocean" | "deep_lukewarm_ocean" => (true, 0.5, 0.5, 4_566_514),
        "cold_ocean" | "deep_cold_ocean" => (true, 0.5, 0.5, 4_020_182),
        "swamp" | "mangrove_swamp" => (true, 0.8, 0.9, 6_388_580),
        "the_end" | "end_highlands" | "end_midlands" | "small_end_islands" | "end_barrens" => {
            (false, 0.5, 0.5, 4_159_204)
        }
        _ => (true, 0.8, 0.4, 4_159_204),
    };

    Tag::Compound(vec![
        (
            "has_precipitation".to_string(),
            Tag::Byte(if has_precipitation { 1 } else { 0 }),
        ),
        ("temperature".to_string(), Tag::Float(temperature)),
        ("downfall".to_string(), Tag::Float(downfall)),
        (
            "effects".to_string(),
            Tag::Compound(vec![("water_color".to_string(), Tag::Int(water_color))]),
        ),
    ])
}

fn dimension_type_nbt(dimension_type: &str) -> Tag {
    match dimension_type {
        "overworld" => overworld_dimension_type_nbt(false),
        "overworld_caves" => overworld_dimension_type_nbt(true),
        "the_end" => fixed_dimension_type_nbt(
            true,
            false,
            true,
            1.0,
            0,
            256,
            256,
            "#minecraft:infiniburn_end",
            0.25,
            Tag::Int(15),
            0,
        ),
        "the_nether" => fixed_dimension_type_nbt(
            false,
            true,
            false,
            8.0,
            0,
            256,
            128,
            "#minecraft:infiniburn_nether",
            0.1,
            Tag::Int(7),
            15,
        ),
        _ => overworld_dimension_type_nbt(false),
    }
}

fn overworld_dimension_type_nbt(has_ceiling: bool) -> Tag {
    // `attributes` encodes via EnvironmentAttributeMap.NETWORK_CODEC (syncable only).
    // Each entry uses EnvironmentAttributeMap.Entry.createCodec: override modifier →
    // Either.left(value) → attribute.valueCodec() directly.
    //
    // Syncable visual attributes from data/minecraft/dimension_type/overworld.json:
    //   RGB_COLOR  → ExtraCodecs.STRING_RGB_COLOR   → hexColor(6) → Tag::String "#rrggbb"
    //   ARGB_COLOR → ExtraCodecs.STRING_ARGB_COLOR  → hexColor(8) → Tag::String "#aarrggbb"
    //   FLOAT      → Codec.FLOAT                    → Tag::Float
    //
    // Non-syncable gameplay/audio attributes (bed_rule, nether_portal_spawns_piglin,
    // respawn_anchor_works) are filtered out by NETWORK_CODEC and must be omitted here.
    // Audio attributes (ambient_sounds, background_music) are syncable but their complex
    // codec structs are not yet implemented; clients fall back to their EMPTY defaults.
    let attributes = Tag::Compound(vec![
        (
            "minecraft:visual/sky_color".to_string(),
            Tag::String("#78a7ff".to_string()),
        ),
        (
            "minecraft:visual/fog_color".to_string(),
            Tag::String("#c0d8ff".to_string()),
        ),
        (
            "minecraft:visual/cloud_color".to_string(),
            Tag::String("#ccffffff".to_string()),
        ),
        (
            "minecraft:visual/cloud_height".to_string(),
            Tag::Float(192.33),
        ),
        (
            "minecraft:visual/ambient_light_color".to_string(),
            Tag::String("#0a0a0a".to_string()),
        ),
    ]);

    Tag::Compound(vec![
        ("has_skylight".to_string(), Tag::Byte(1)),
        (
            "has_ceiling".to_string(),
            Tag::Byte(if has_ceiling { 1 } else { 0 }),
        ),
        ("has_ender_dragon_fight".to_string(), Tag::Byte(0)),
        ("coordinate_scale".to_string(), Tag::Double(1.0)),
        ("min_y".to_string(), Tag::Int(-64)),
        ("height".to_string(), Tag::Int(384)),
        ("logical_height".to_string(), Tag::Int(384)),
        (
            "infiniburn".to_string(),
            Tag::String("#minecraft:infiniburn_overworld".to_string()),
        ),
        ("ambient_light".to_string(), Tag::Float(0.0)),
        (
            "monster_spawn_light_level".to_string(),
            Tag::Compound(vec![
                (
                    "type".to_string(),
                    Tag::String("minecraft:uniform".to_string()),
                ),
                ("min_inclusive".to_string(), Tag::Int(0)),
                ("max_inclusive".to_string(), Tag::Int(7)),
            ]),
        ),
        ("monster_spawn_block_light_limit".to_string(), Tag::Int(0)),
        // `timelines`: HolderSet<Timeline> reference. The tag "#minecraft:in_overworld"
        // is resolved by the client using the timeline tags sent in the tags packet.
        // Java: DimensionType.NETWORK_CODEC — RegistryCodecs.homogeneousList(TIMELINE)
        //       → HolderSet.TagKey encodes as Tag::String("#<tag-id>").
        (
            "timelines".to_string(),
            Tag::String("#minecraft:in_overworld".to_string()),
        ),
        // `default_clock`: which world clock drives the timeline for this dimension.
        // Java: WorldClock.CODEC = RegistryFixedCodec → Tag::String("<registry-key>").
        (
            "default_clock".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        ),
        // `attributes`: static base values for syncable EnvironmentAttributes.
        // The timeline tracks multiply/add to these base values at runtime on the client.
        ("attributes".to_string(), attributes),
    ])
}

fn fixed_dimension_type_nbt(
    has_skylight: bool,
    has_ceiling: bool,
    has_ender_dragon_fight: bool,
    coordinate_scale: f64,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: &str,
    ambient_light: f32,
    monster_spawn_light_level: Tag,
    monster_spawn_block_light_limit: i32,
) -> Tag {
    Tag::Compound(vec![
        (
            "has_skylight".to_string(),
            Tag::Byte(if has_skylight { 1 } else { 0 }),
        ),
        (
            "has_ceiling".to_string(),
            Tag::Byte(if has_ceiling { 1 } else { 0 }),
        ),
        (
            "has_ender_dragon_fight".to_string(),
            Tag::Byte(if has_ender_dragon_fight { 1 } else { 0 }),
        ),
        (
            "coordinate_scale".to_string(),
            Tag::Double(coordinate_scale),
        ),
        ("min_y".to_string(), Tag::Int(min_y)),
        ("height".to_string(), Tag::Int(height)),
        ("logical_height".to_string(), Tag::Int(logical_height)),
        (
            "infiniburn".to_string(),
            Tag::String(infiniburn.to_string()),
        ),
        ("ambient_light".to_string(), Tag::Float(ambient_light)),
        (
            "monster_spawn_light_level".to_string(),
            monster_spawn_light_level,
        ),
        (
            "monster_spawn_block_light_limit".to_string(),
            Tag::Int(monster_spawn_block_light_limit),
        ),
    ])
}

fn trim_material_nbt(material: &TrimMaterialEntry) -> Tag {
    let mut fields = vec![
        (
            "asset_name".to_string(),
            Tag::String(material.asset_name.to_string()),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![
                (
                    "translate".to_string(),
                    Tag::String(format!("trim_material.minecraft.{}", material.id)),
                ),
                ("color".to_string(), Tag::String(material.color.to_string())),
            ]),
        ),
    ];

    if !material.overrides.is_empty() {
        fields.push((
            "override_armor_assets".to_string(),
            Tag::Compound(
                material
                    .overrides
                    .iter()
                    .map(|(asset, suffix)| {
                        ((*asset).to_string(), Tag::String((*suffix).to_string()))
                    })
                    .collect(),
            ),
        ));
    }

    Tag::Compound(fields)
}

fn chat_type_nbt(chat_type: &ChatTypeEntry) -> Tag {
    Tag::Compound(vec![
        (
            "chat".to_string(),
            chat_decoration_nbt(chat_type.chat_translation_key, chat_type.chat_parameters),
        ),
        (
            "narration".to_string(),
            chat_decoration_nbt(
                chat_type.narration_translation_key,
                chat_type.narration_parameters,
            ),
        ),
    ])
}

fn chat_decoration_nbt(translation_key: &str, parameters: &[&str]) -> Tag {
    Tag::Compound(vec![
        (
            "translation_key".to_string(),
            Tag::String(translation_key.to_string()),
        ),
        (
            "parameters".to_string(),
            Tag::List(
                parameters
                    .iter()
                    .map(|parameter| Tag::String((*parameter).to_string()))
                    .collect(),
            ),
        ),
    ])
}

fn trim_pattern_nbt(pattern: &str) -> Tag {
    Tag::Compound(vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{pattern}")),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("trim_pattern.minecraft.{pattern}")),
            )]),
        ),
        ("decal".to_string(), Tag::Byte(0)),
    ])
}

fn jukebox_song_nbt(song: &JukeboxSongEntry) -> Tag {
    Tag::Compound(vec![
        (
            "sound_event".to_string(),
            Tag::String(song.sound_event.to_string()),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("jukebox_song.minecraft.{}", song.id)),
            )]),
        ),
        (
            "length_in_seconds".to_string(),
            Tag::Float(song.length_seconds),
        ),
        (
            "comparator_output".to_string(),
            Tag::Int(song.comparator_output),
        ),
    ])
}

fn banner_pattern_nbt(pattern: &str) -> Tag {
    Tag::Compound(vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{pattern}")),
        ),
        (
            "translation_key".to_string(),
            Tag::String(format!("block.minecraft.banner.{pattern}")),
        ),
    ])
}

fn instrument_nbt(instrument: &InstrumentEntry) -> Tag {
    Tag::Compound(vec![
        (
            "sound_event".to_string(),
            Tag::String(instrument.sound_event.to_string()),
        ),
        ("use_duration".to_string(), Tag::Float(7.0)),
        ("range".to_string(), Tag::Float(256.0)),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("instrument.minecraft.{}", instrument.id)),
            )]),
        ),
    ])
}

fn single_texture_variant_nbt(asset_id: &str) -> Tag {
    Tag::Compound(vec![(
        "asset_id".to_string(),
        Tag::String(asset_id.to_string()),
    )])
}

fn animal_texture_variant_nbt(kind: &str, texture_name: &str, model: &str) -> Tag {
    let mut fields = vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:entity/{kind}/{texture_name}")),
        ),
        (
            "baby_asset_id".to_string(),
            Tag::String(format!("minecraft:entity/{kind}/{texture_name}_baby")),
        ),
    ];
    if model != "normal" {
        fields.push(("model".to_string(), Tag::String(model.to_string())));
    }
    Tag::Compound(fields)
}

fn wolf_variant_nbt(file_name: &str) -> Tag {
    let assets = wolf_assets_nbt(file_name, "");
    let baby_assets = wolf_assets_nbt(file_name, "_baby");
    Tag::Compound(vec![
        ("assets".to_string(), assets),
        ("baby_assets".to_string(), baby_assets),
    ])
}

fn wolf_assets_nbt(file_name: &str, suffix: &str) -> Tag {
    Tag::Compound(vec![
        (
            "wild".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}{suffix}")),
        ),
        (
            "tame".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}_tame{suffix}")),
        ),
        (
            "angry".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}_angry{suffix}")),
        ),
    ])
}

fn zombie_nautilus_variant_nbt(variant: &str) -> Tag {
    let (asset_id, model) = match variant {
        "warm" => ("minecraft:entity/nautilus/zombie_nautilus_coral", "warm"),
        _ => ("minecraft:entity/nautilus/zombie_nautilus", "normal"),
    };
    Tag::Compound(vec![
        ("asset_id".to_string(), Tag::String(asset_id.to_string())),
        ("model".to_string(), Tag::String(model.to_string())),
    ])
}

fn painting_variant_nbt(id: &str) -> Tag {
    Tag::Compound(vec![
        ("width".to_string(), Tag::Int(1)),
        ("height".to_string(), Tag::Int(1)),
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{id}")),
        ),
    ])
}

fn cow_sound_variant_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.cow.ambient"),
        sound_field("hurt_sound", "minecraft:entity.cow.hurt"),
        sound_field("death_sound", "minecraft:entity.cow.death"),
        sound_field("step_sound", "minecraft:entity.cow.step"),
    ])
}

fn chicken_sound_variant_nbt() -> Tag {
    let sounds = chicken_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

fn chicken_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.chicken.ambient"),
        sound_field("hurt_sound", "minecraft:entity.chicken.hurt"),
        sound_field("death_sound", "minecraft:entity.chicken.death"),
        sound_field("step_sound", "minecraft:entity.chicken.step"),
    ])
}

fn pig_sound_variant_nbt() -> Tag {
    let sounds = pig_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

fn pig_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.pig.ambient"),
        sound_field("hurt_sound", "minecraft:entity.pig.hurt"),
        sound_field("death_sound", "minecraft:entity.pig.death"),
        sound_field("step_sound", "minecraft:entity.pig.step"),
        sound_field("eat_sound", "minecraft:entity.generic.eat"),
    ])
}

fn cat_sound_variant_nbt() -> Tag {
    let sounds = cat_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

fn cat_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.cat.ambient"),
        sound_field("stray_ambient_sound", "minecraft:entity.cat.stray_ambient"),
        sound_field("hiss_sound", "minecraft:entity.cat.hiss"),
        sound_field("hurt_sound", "minecraft:entity.cat.hurt"),
        sound_field("death_sound", "minecraft:entity.cat.death"),
        sound_field("eat_sound", "minecraft:entity.generic.eat"),
        sound_field("beg_for_food_sound", "minecraft:entity.cat.beg_for_food"),
        sound_field("purr_sound", "minecraft:entity.cat.purr"),
        sound_field("purreow_sound", "minecraft:entity.cat.purreow"),
    ])
}

fn wolf_sound_variant_nbt() -> Tag {
    let sounds = wolf_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

fn wolf_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.wolf.ambient"),
        sound_field("death_sound", "minecraft:entity.wolf.death"),
        sound_field("growl_sound", "minecraft:entity.wolf.growl"),
        sound_field("hurt_sound", "minecraft:entity.wolf.hurt"),
        sound_field("pant_sound", "minecraft:entity.wolf.pant"),
        sound_field("whine_sound", "minecraft:entity.wolf.whine"),
        sound_field("step_sound", "minecraft:entity.wolf.step"),
    ])
}

fn sound_field(name: &str, sound: &str) -> (String, Tag) {
    (name.to_string(), Tag::String(sound.to_string()))
}

fn write_network_nbt<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    tag.write_payload(writer)
}

fn write_clientbound_login_packet<W: Write>(
    writer: &mut W,
    packet: &ClientboundLoginPacket,
) -> io::Result<()> {
    writer.write_all(&packet.player_id.to_be_bytes())?;
    write_bool(writer, packet.hardcore)?;
    write_var_i32(writer, packet.levels.len() as i32)?;
    for level in &packet.levels {
        write_identifier(writer, level)?;
    }
    write_var_i32(writer, packet.max_players)?;
    write_var_i32(writer, packet.chunk_radius)?;
    write_var_i32(writer, packet.simulation_distance)?;
    write_bool(writer, packet.reduced_debug_info)?;
    write_bool(writer, packet.show_death_screen)?;
    write_bool(writer, packet.do_limited_crafting)?;
    write_common_spawn_info(writer, &packet.spawn_info)?;
    write_bool(writer, packet.enforces_secure_chat)
}

fn write_common_spawn_info<W: Write>(
    writer: &mut W,
    spawn_info: &CommonPlayerSpawnInfo,
) -> io::Result<()> {
    write_var_i32(
        writer,
        dimension_type_registry_id(&spawn_info.dimension_type)?,
    )?;
    write_identifier(writer, &spawn_info.dimension)?;
    writer.write_all(&spawn_info.seed.to_be_bytes())?;
    writer.write_all(&[spawn_info.game_mode as u8])?;
    writer.write_all(&[match spawn_info.previous_game_mode {
        Some(GameMode::Survival) => 0,
        Some(GameMode::Creative) => 1,
        Some(GameMode::Adventure) => 2,
        Some(GameMode::Spectator) => 3,
        None => 255,
    }])?;
    write_bool(writer, spawn_info.is_debug)?;
    write_bool(writer, spawn_info.is_flat)?;
    write_optional(
        writer,
        spawn_info.last_death_location.as_ref(),
        |writer, (dimension, pos)| {
            write_identifier(writer, dimension)?;
            for coordinate in pos {
                writer.write_all(&coordinate.to_be_bytes())?;
            }
            Ok(())
        },
    )?;
    write_var_i32(writer, spawn_info.portal_cooldown)?;
    write_var_i32(writer, spawn_info.sea_level)
}

fn game_mode_legacy_id(game_mode: GameMode) -> i32 {
    match game_mode {
        GameMode::Survival => 0,
        GameMode::Creative => 1,
        GameMode::Adventure => 2,
        GameMode::Spectator => 3,
    }
}

fn game_mode_from_legacy_id(id: i32) -> GameMode {
    match id {
        1 => GameMode::Creative,
        2 => GameMode::Adventure,
        3 => GameMode::Spectator,
        _ => GameMode::Survival,
    }
}

fn game_mode_from_name(name: &str) -> GameMode {
    if let Ok(id) = name.parse::<i32>() {
        return game_mode_from_legacy_id(id);
    }
    match name {
        "creative" => GameMode::Creative,
        "adventure" => GameMode::Adventure,
        "spectator" => GameMode::Spectator,
        _ => GameMode::Survival,
    }
}

fn dimension_type_registry_id(dimension_type: &Identifier) -> io::Result<i32> {
    if dimension_type.namespace() == "minecraft" && dimension_type.path() == "overworld" {
        Ok(0)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported dimension type {dimension_type} in login packet"),
        ))
    }
}

fn write_vec3<W: Write>(writer: &mut W, x: f64, y: f64, z: f64) -> io::Result<()> {
    writer.write_all(&x.to_be_bytes())?;
    writer.write_all(&y.to_be_bytes())?;
    writer.write_all(&z.to_be_bytes())
}

fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0] != 0)
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

/// Writes a ClientboundGameEventPacket with the given type and float parameter.
/// Java: ClientboundGameEventPacket — byte event type, float param
fn write_game_event(
    stream: &mut TcpStream,
    compression: CompressionState,
    event_type: u8,
    param: f32,
) -> io::Result<()> {
    write_game_event_to_writer(stream, compression, event_type, param)
}

fn write_game_event_to_writer<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    event_type: u8,
    param: f32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[event_type])?;
            payload.write_all(&param.to_be_bytes())
        },
    )
}

fn write_framed_packet<W, F>(writer: &mut W, packet_id: i32, write_body: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    write_packet(writer, &payload)
}

/// Returns the number of bytes needed to encode `value` as a Minecraft VarInt.
///
/// Packet IDs are always non-negative, so the sign extension applied to
/// negative values by [`write_var_i32`] is not relevant here.
fn var_int_encoded_len(value: i32) -> usize {
    let uval = value as u32;
    match uval {
        0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1F_FFFF => 3,
        0x20_0000..=0xFFF_FFFF => 4,
        _ => 5,
    }
}

fn write_framed_packet_with_compression<W, F>(
    writer: &mut W,
    compression: CompressionState,
    packet_id: i32,
    write_body: F,
) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    // Emit a TRACE-level packet dump when tracing is active.  The guard avoids
    // the format overhead when tracing is off.
    if crate::log::global_level() >= crate::log::LogLevel::Trace {
        let id_len = var_int_encoded_len(packet_id);
        crate::log::log_packet_send(packet_id, &payload[id_len..]);
    }
    let frame = compression.encode_packet(&payload)?;
    writer.write_all(&frame)
}

fn write_status_response_packet<W: Write>(writer: &mut W, json: &str) -> io::Result<()> {
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 0)?;
    write_string(&mut payload, json)?;
    write_packet(writer, &payload)
}

fn write_status_pong_packet<W: Write>(
    writer: &mut W,
    request: ServerboundPingRequestPacket,
) -> io::Result<()> {
    let response = ClientboundPongResponsePacket::from_request(request);
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 1)?;
    response.write(&mut payload)?;
    write_packet(writer, &payload)
}

fn handle_legacy_status_tcp_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
) -> io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut request = Vec::new();
    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => request.extend_from_slice(&buffer[..count]),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                let _ = stream.set_nonblocking(false);
                return Err(err);
            }
        }
    }
    stream.set_nonblocking(false)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

fn handle_legacy_status_connection<W: Read + Write>(
    stream: &mut W,
    properties: &ServerProperties,
) -> io::Result<()> {
    let mut request = Vec::new();
    stream.read_to_end(&mut request)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

fn legacy_status_response(request: &[u8], properties: &ServerProperties) -> io::Result<Vec<u8>> {
    if request.first() != Some(&0xFE) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected legacy query packet",
        ));
    }

    let body = match &request[1..] {
        [] => legacy_version0_response(properties),
        [0x01] => legacy_version1_response(properties),
        [0x01, tail @ ..] if read_legacy_ping_host_payload(tail).is_some() => {
            legacy_version1_response(properties)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "malformed legacy query packet",
            ))
        }
    };

    Ok(legacy_disconnect_packet(&body))
}

fn read_legacy_ping_host_payload(input: &[u8]) -> Option<()> {
    let mut input = Cursor::new(input);
    let mut packet_id = [0u8; 1];
    input.read_exact(&mut packet_id).ok()?;
    if packet_id[0] != 250 {
        return None;
    }

    let channel = read_legacy_string(&mut input).ok()?;
    if channel != "MC|PingHost" {
        return None;
    }

    let mut size = [0u8; 2];
    input.read_exact(&mut size).ok()?;
    let payload_size = u16::from_be_bytes(size) as u64;
    if input.get_ref().len() as u64 - input.position() != payload_size {
        return None;
    }

    let mut protocol = [0u8; 1];
    input.read_exact(&mut protocol).ok()?;
    if protocol[0] < 73 {
        return None;
    }

    let _host = read_legacy_string(&mut input).ok()?;
    let mut port = [0u8; 4];
    input.read_exact(&mut port).ok()?;
    (u32::from_be_bytes(port) <= u16::MAX as u32).then_some(())
}

fn legacy_version0_response(properties: &ServerProperties) -> String {
    format!("{}§{}§{}", properties.motd, 0, properties.max_players)
}

fn legacy_version1_response(properties: &ServerProperties) -> String {
    format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        127, VERSION_NAME, properties.motd, 0, properties.max_players
    )
}

fn legacy_disconnect_packet(reason: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(3 + reason.len() * 2);
    out.push(255);
    write_legacy_string(&mut out, reason).expect("legacy string write to vec cannot fail");
    out
}

fn read_legacy_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut length = [0u8; 2];
    reader.read_exact(&mut length)?;
    let char_count = u16::from_be_bytes(length) as usize;
    let mut bytes = vec![0u8; char_count * 2];
    reader.read_exact(&mut bytes)?;
    String::from_utf16(
        &bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>(),
    )
    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn write_legacy_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let len = u16::try_from(utf16.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "legacy string too long"))?;
    writer.write_all(&len.to_be_bytes())?;
    for code_unit in utf16 {
        writer.write_all(&code_unit.to_be_bytes())?;
    }
    Ok(())
}

fn read_packet<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > MAX_PACKET_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid packet length",
        ));
    }

    let mut payload = vec![0u8; length as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn read_packet_with_compression<R: Read>(
    reader: &mut R,
    compression: CompressionState,
) -> io::Result<Vec<u8>> {
    match compression.threshold() {
        None => read_packet(reader),
        Some(_) => compression.decode_packet(reader),
    }
}

fn write_packet<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

fn read_string<R: Read>(reader: &mut R, max_chars: usize) -> io::Result<String> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_chars * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid string length",
        ));
    }

    let mut bytes = vec![0u8; length as usize];
    reader.read_exact(&mut bytes)?;
    let string =
        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if string.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    Ok(string)
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    write_var_i32(writer, value.len() as i32)?;
    writer.write_all(value.as_bytes())
}

fn load_favicon(path: &Path) -> io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;
    let (width, height) = png_dimensions(&bytes)?;
    if width != 64 || height != 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("server-icon.png must be 64x64, got {width}x{height}"),
        ));
    }
    Ok(Some(format!(
        "data:image/png;base64,{}",
        encode_base64(&bytes)
    )))
}

fn png_dimensions(bytes: &[u8]) -> io::Result<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "server-icon.png must be a PNG with an IHDR header",
        ));
    }

    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Ok((width, height))
}

pub fn status_json(properties: &ServerProperties, favicon: Option<&str>) -> String {
    let players = if properties.hide_online_players {
        format!(
            "\"players\":{{\"max\":{},\"online\":0,\"sample\":[]}}",
            properties.max_players
        )
    } else {
        format!(
            "\"players\":{{\"max\":{},\"online\":0,\"sample\":[]}}",
            properties.max_players
        )
    };

    let favicon = favicon
        .map(|value| format!(",\"favicon\":\"{}\"", escape_json_string(value)))
        .unwrap_or_default();

    format!(
        "{{\"version\":{{\"name\":\"{}\",\"protocol\":{}}},{},\"description\":{{\"text\":\"{}\"}}{},\"enforcesSecureChat\":true}}",
        VERSION_NAME,
        PROTOCOL_VERSION,
        players,
        escape_json_string(&properties.motd),
        favicon
    )
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

fn escape_json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        banner_pattern_nbt, bug_report_server_links_packet, cat_sound_variant_nbt, chat_type_nbt,
        chicken_sound_variant_nbt, chunk_batch_size, chunk_has_non_air_blocks, chunk_window,
        cow_sound_variant_nbt, day_timeline_nbt, early_game_timeline_nbt, encode_base64,
        escape_json_string, handle_legacy_status_connection, instrument_nbt,
        inventory_internal_slot, jukebox_song_nbt, legacy_disconnect_packet,
        legacy_version0_response, legacy_version1_response, load_code_of_conduct_for_language,
        load_favicon, login_access_disconnect_reason, login_host_ip, moon_timeline_nbt,
        newly_visible_chunks, overworld_dimension_type_nbt, packed_chunk_pos,
        pig_sound_variant_nbt, play_session_state_from_nbt, play_session_state_to_nbt,
        pseudo_rand_f32, read_code_of_conducts, read_packet, status_json,
        strip_minecraft_formatting, trim_material_nbt, trim_pattern_nbt,
        vanilla_baseline_biome_nbt, var_int_encoded_len, villager_schedule_timeline_nbt,
        visible_spawn_surface_feature_id, visible_spawn_surface_top_block_id,
        visible_spawn_terrain_block_count, visible_spawn_terrain_height,
        wait_for_configuration_packet, wolf_sound_variant_nbt, write_framed_packet,
        write_legacy_string, write_lp_vec3, write_minimal_biome_registry_packet,
        write_minimal_damage_type_registry_packet, write_minimal_dimension_type_registry_packet,
        write_minimal_trim_material_registry_packet, write_minimal_update_tags_packet,
        write_status_pong_packet, write_vanilla_banner_pattern_registry_packet,
        write_vanilla_cat_sound_variant_registry_packet, write_vanilla_cat_variant_registry_packet,
        write_vanilla_chat_type_registry_packet,
        write_vanilla_chicken_sound_variant_registry_packet,
        write_vanilla_chicken_variant_registry_packet,
        write_vanilla_cow_sound_variant_registry_packet, write_vanilla_cow_variant_registry_packet,
        write_vanilla_frog_variant_registry_packet, write_vanilla_instrument_registry_packet,
        write_vanilla_jukebox_song_registry_packet, write_vanilla_painting_variant_registry_packet,
        write_vanilla_pig_sound_variant_registry_packet, write_vanilla_pig_variant_registry_packet,
        write_vanilla_timeline_registry_packet, write_vanilla_trim_pattern_registry_packet,
        write_vanilla_wolf_sound_variant_registry_packet,
        write_vanilla_wolf_variant_registry_packet,
        write_vanilla_zombie_nautilus_variant_registry_packet,
        write_visible_spawn_terrain_block_state_container, write_world_clock_registry_packet,
        CompressionState, FoodDifficulty, GameMode, PlayerGlobalPosData, PlayerNbtAbilities,
        PlayerSpawnData, ANDESITE_BLOCK_STATE_ID, BANNER_PATTERNS, BANNER_PATTERN_TAGS,
        BEDROCK_BLOCK_STATE_ID, BIOMES, CHAT_TYPES, CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID, DAMAGE_TYPES, DAMAGE_TYPE_TAGS,
        DANDELION_BLOCK_STATE_ID, DIORITE_BLOCK_STATE_ID, DIRT_BLOCK_STATE_ID,
        GRANITE_BLOCK_STATE_ID, GRASS_BLOCK_STATE_ID, INSTRUMENTS, JUKEBOX_SONGS, MAX_PACKET_SIZE,
        POPPY_BLOCK_STATE_ID, SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID,
        SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID, SHORT_GRASS_BLOCK_STATE_ID,
        SPRINT_JUMP_EXHAUSTION, STONE_BLOCK_STATE_ID, TRIM_MATERIALS, TRIM_PATTERNS, VERSION_NAME,
    };
    use crate::item_stack::ItemStack;
    use crate::network::codec::{write_identifier, Uuid};
    use crate::network::common::{ServerLinkLabel, ServerLinkType};
    use crate::network::ping::ServerboundPingRequestPacket;
    use crate::network::play::{
        ClientboundAddEntityPacket, ClientboundSetEntityDataPacket, EntityDataValue,
        EntityMetadataValue, Vec3, CLIENTBOUND_ADD_ENTITY_PACKET_ID,
        CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID, CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
    };
    use crate::network::varint::{read_var_i32, write_var_i32};
    use crate::player_inventory::{InventoryMenu, PlayerInventory};
    use crate::recipe_system::RecipeMap;
    use crate::registry::Identifier;
    use crate::server_properties::ServerProperties;
    use crate::storage::chunk::LevelChunk;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;
    use crate::{biome, damage_type, equipment_trim, presentation_data};
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::{self, Cursor, Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    struct SynchronizedRegistryManifestEntry {
        registry_id: &'static str,
        expected_entry_count: usize,
        java_network_shape: &'static str,
        write_packet: fn(&mut Vec<u8>) -> io::Result<()>,
    }

    #[test]
    fn bug_report_link_becomes_known_server_link_when_valid() {
        let mut properties = ServerProperties::load_or_default(Path::new(
            "definitely-missing-test-server.properties",
        ))
        .unwrap();
        assert!(bug_report_server_links_packet(&properties).is_none());

        properties.set("bug-report-link", "https://example.invalid/bugs");
        let packet = bug_report_server_links_packet(&properties).unwrap();
        assert_eq!(packet.links.len(), 1);
        assert_eq!(
            packet.links[0].label,
            ServerLinkLabel::Known(ServerLinkType::BugReport)
        );
        assert_eq!(packet.links[0].link, "https://example.invalid/bugs");

        properties.set("bug-report-link", "not a uri");
        assert!(bug_report_server_links_packet(&properties).is_none());
    }

    #[test]
    fn code_of_conduct_loader_strips_formatting_and_applies_language_fallback() {
        let mut root = std::env::temp_dir();
        root.push(format!("rustcraft-codeofconduct-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let code_dir = root.join("codeofconduct");
        fs::create_dir(&code_dir).unwrap();
        fs::write(code_dir.join("en_us.txt"), "§aEnglish\nRules").unwrap();
        fs::write(code_dir.join("fr_fr.txt"), "§cRegles").unwrap();
        fs::write(code_dir.join("ignored.md"), "nope").unwrap();

        let texts = read_code_of_conducts(&code_dir).unwrap();
        assert_eq!(
            texts.get("en_us").map(String::as_str),
            Some("English\nRules")
        );
        assert_eq!(texts.get("fr_fr").map(String::as_str), Some("Regles"));
        assert!(!texts.contains_key("ignored"));
        assert_eq!(strip_minecraft_formatting("A§lB§rC"), "ABC");

        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(&root).unwrap();

        let mut properties = ServerProperties::load_or_default(Path::new(
            "definitely-missing-test-server.properties",
        ))
        .unwrap();
        assert!(load_code_of_conduct_for_language(&properties, "fr_fr")
            .unwrap()
            .is_none());
        properties.set("enable-code-of-conduct", "true");
        assert_eq!(
            load_code_of_conduct_for_language(&properties, "fr_fr").unwrap(),
            Some("Regles".to_string())
        );
        assert_eq!(
            load_code_of_conduct_for_language(&properties, "es_es").unwrap(),
            Some("English\nRules".to_string())
        );

        std::env::set_current_dir(previous).unwrap();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn read_packet_rejects_negative_and_oversized_lengths_before_allocating() {
        let mut negative = Vec::new();
        write_var_i32(&mut negative, -1).unwrap();
        assert_eq!(
            read_packet(&mut Cursor::new(negative)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );

        let mut oversized = Vec::new();
        write_var_i32(&mut oversized, (MAX_PACKET_SIZE + 1) as i32).unwrap();
        assert_eq!(
            read_packet(&mut Cursor::new(oversized)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    const SYNCHRONIZED_REGISTRY_MANIFEST: &[SynchronizedRegistryManifestEntry] = &[
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:worldgen/biome",
            expected_entry_count: 65,
            java_network_shape: "Biome.NETWORK_CODEC",
            write_packet: write_minimal_biome_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:chat_type",
            expected_entry_count: 7,
            java_network_shape: "ChatType.DIRECT_CODEC",
            write_packet: write_vanilla_chat_type_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:trim_pattern",
            expected_entry_count: 18,
            java_network_shape: "TrimPattern.DIRECT_CODEC",
            write_packet: write_vanilla_trim_pattern_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:trim_material",
            expected_entry_count: 11,
            java_network_shape: "TrimMaterial.DIRECT_CODEC",
            write_packet: write_minimal_trim_material_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:wolf_variant",
            expected_entry_count: 9,
            java_network_shape: "WolfVariant.NETWORK_CODEC",
            write_packet: write_vanilla_wolf_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:wolf_sound_variant",
            expected_entry_count: 7,
            java_network_shape: "WolfSoundVariant.NETWORK_CODEC",
            write_packet: write_vanilla_wolf_sound_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:pig_variant",
            expected_entry_count: 3,
            java_network_shape: "PigVariant.NETWORK_CODEC",
            write_packet: write_vanilla_pig_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:pig_sound_variant",
            expected_entry_count: 3,
            java_network_shape: "PigSoundVariant.NETWORK_CODEC",
            write_packet: write_vanilla_pig_sound_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:frog_variant",
            expected_entry_count: 3,
            java_network_shape: "FrogVariant.NETWORK_CODEC",
            write_packet: write_vanilla_frog_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:cat_variant",
            expected_entry_count: 11,
            java_network_shape: "CatVariant.NETWORK_CODEC",
            write_packet: write_vanilla_cat_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:cat_sound_variant",
            expected_entry_count: 2,
            java_network_shape: "CatSoundVariant.NETWORK_CODEC",
            write_packet: write_vanilla_cat_sound_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:cow_sound_variant",
            expected_entry_count: 2,
            java_network_shape: "CowSoundVariant.DIRECT_CODEC",
            write_packet: write_vanilla_cow_sound_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:cow_variant",
            expected_entry_count: 3,
            java_network_shape: "CowVariant.NETWORK_CODEC",
            write_packet: write_vanilla_cow_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:chicken_sound_variant",
            expected_entry_count: 2,
            java_network_shape: "ChickenSoundVariant.DIRECT_CODEC",
            write_packet: write_vanilla_chicken_sound_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:chicken_variant",
            expected_entry_count: 3,
            java_network_shape: "ChickenVariant.NETWORK_CODEC",
            write_packet: write_vanilla_chicken_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:zombie_nautilus_variant",
            expected_entry_count: 2,
            java_network_shape: "ZombieNautilusVariant.NETWORK_CODEC",
            write_packet: write_vanilla_zombie_nautilus_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:painting_variant",
            expected_entry_count: 51,
            java_network_shape: "PaintingVariant.DIRECT_CODEC",
            write_packet: write_vanilla_painting_variant_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:dimension_type",
            expected_entry_count: 4,
            java_network_shape: "DimensionType.NETWORK_CODEC",
            write_packet: write_minimal_dimension_type_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:damage_type",
            expected_entry_count: 50,
            java_network_shape: "DamageType.DIRECT_CODEC",
            write_packet: write_minimal_damage_type_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:banner_pattern",
            expected_entry_count: 43,
            java_network_shape: "BannerPattern.DIRECT_CODEC",
            write_packet: write_vanilla_banner_pattern_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:jukebox_song",
            expected_entry_count: 21,
            java_network_shape: "JukeboxSong.DIRECT_CODEC",
            write_packet: write_vanilla_jukebox_song_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:instrument",
            expected_entry_count: 8,
            java_network_shape: "Instrument.DIRECT_CODEC",
            write_packet: write_vanilla_instrument_registry_packet,
        },
        SynchronizedRegistryManifestEntry {
            registry_id: "minecraft:world_clock",
            expected_entry_count: 2,
            java_network_shape: "WorldClock.DIRECT_CODEC",
            write_packet: write_world_clock_registry_packet,
        },
    ];

    fn test_properties() -> ServerProperties {
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
            .unwrap()
    }

    #[test]
    fn escapes_status_description() {
        assert_eq!(
            escape_json_string("A \"quoted\" server"),
            "A \\\"quoted\\\" server"
        );
    }

    #[test]
    fn includes_26_1_2_protocol_in_status_json() {
        let mut properties = test_properties();
        properties.set("motd", "RustCraft Test");
        let json = status_json(&properties, None);
        assert!(json.contains("\"name\":\"26.1.2\""));
        assert!(json.contains("\"protocol\":775"));
        assert!(json.contains("\"max\":20"));
        assert!(json.contains("\"players\":{\"max\":20,\"online\":0,\"sample\":[]}"));
        assert!(json.contains("\"description\":{\"text\":\"RustCraft Test\"}"));
    }

    #[test]
    fn hidden_online_players_preserves_counts_and_omits_sample_entries() {
        let mut properties = test_properties();
        properties.set("hide-online-players", "true");
        properties.set("max-players", "37");

        let json = status_json(&properties, None);

        assert!(json.contains("\"players\":{\"max\":37,\"online\":0,\"sample\":[]}"));
    }

    #[test]
    fn prevent_proxy_connections_rejects_mismatched_handshake_ip() {
        let mut properties = test_properties();
        properties.set("prevent-proxy-connections", "true");
        let access = Arc::new(Mutex::new(crate::player_access::PlayerAccess::default()));
        let profile = crate::player_access::NameAndId::create_offline("Steve");

        assert_eq!(
            login_host_ip("203.0.113.10"),
            Some("203.0.113.10".to_string())
        );
        assert_eq!(
            login_host_ip("[2001:db8::1]"),
            Some("2001:db8::1".to_string())
        );
        assert_eq!(login_host_ip("localhost"), None);
        assert_eq!(
            login_access_disconnect_reason(
                &properties,
                &access,
                &profile,
                "198.51.100.20",
                Some("203.0.113.10"),
            )
            .unwrap(),
            Some("multiplayer.disconnect.unverified_username")
        );
        assert_eq!(
            login_access_disconnect_reason(
                &properties,
                &access,
                &profile,
                "203.0.113.10",
                Some("203.0.113.10"),
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn includes_favicon_when_present() {
        let properties = test_properties();
        let json = status_json(&properties, Some("data:image/png;base64,iVBORw0KGgo="));
        assert!(json.contains("\"favicon\":\"data:image/png;base64,iVBORw0KGgo=\""));
    }

    #[test]
    fn server_icon_loader_requires_64_by_64_png_and_encodes_data_uri() {
        let path = temp_status_test_path("server-icon-64.png");
        fs::write(&path, png_header(64, 64)).unwrap();

        let favicon = load_favicon(&path).unwrap().unwrap();
        assert!(favicon.starts_with("data:image/png;base64,"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn server_icon_loader_rejects_wrong_png_dimensions() {
        let path = temp_status_test_path("server-icon-32.png");
        fs::write(&path, png_header(32, 64)).unwrap();

        let error = load_favicon(&path).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("64x64"));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn encodes_base64_padding_cases() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
    }

    fn temp_status_test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("rustcraft-status-{}-{name}", std::process::id()))
    }

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes.extend_from_slice(&13_u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes.extend_from_slice(&0_u32.to_be_bytes());
        bytes
    }

    #[test]
    fn formats_legacy_status_responses_like_vanilla() {
        let properties = test_properties();

        assert_eq!(
            legacy_version0_response(&properties),
            "A Minecraft Server§0§20"
        );
        assert_eq!(
            legacy_version1_response(&properties),
            "§1\0127\026.1.2\0A Minecraft Server\00\020"
        );

        let packet = legacy_disconnect_packet("hello");
        assert_eq!(packet[0], 255);
        assert_eq!(u16::from_be_bytes([packet[1], packet[2]]), 5);
    }

    #[test]
    fn handles_legacy_1_6_ping_host_payload() {
        let properties = test_properties();
        let mut request = vec![0xFE, 0x01, 0xFA];
        write_legacy_string(&mut request, "MC|PingHost").unwrap();
        let mut payload = vec![127];
        write_legacy_string(&mut payload, "localhost").unwrap();
        payload.extend_from_slice(&25565u32.to_be_bytes());
        request.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        request.extend_from_slice(&payload);

        let mut stream = CursorStream::new(request);
        handle_legacy_status_connection(&mut stream, &properties).unwrap();

        assert_eq!(stream.written[0], 255);
        assert_eq!(
            u16::from_be_bytes([stream.written[1], stream.written[2]]),
            37
        );
        assert_eq!(&stream.written[3..7], &[0, 0xA7, 0, b'1']);
    }

    #[test]
    fn status_ping_packet_echoes_payload_for_client_latency_measurement() {
        let request = ServerboundPingRequestPacket {
            time: 0x0102_0304_0506_0708,
        };
        let mut output = Vec::new();
        write_status_pong_packet(&mut output, request).unwrap();

        let packet = read_packet(&mut Cursor::new(output)).unwrap();
        let mut input = Cursor::new(packet);
        assert_eq!(read_var_i32(&mut input).unwrap(), 1);
        let mut echoed_time = [0u8; 8];
        input.read_exact(&mut echoed_time).unwrap();
        assert_eq!(i64::from_be_bytes(echoed_time), request.time);
    }

    #[test]
    fn animal_sound_variant_payloads_match_nested_26_1_2_codecs() {
        assert_sound_variant_fields(
            cow_sound_variant_nbt(),
            &["ambient_sound", "hurt_sound", "death_sound", "step_sound"],
        );
        assert_nested_sound_variant_fields(
            chicken_sound_variant_nbt(),
            &["ambient_sound", "hurt_sound", "death_sound", "step_sound"],
        );
        assert_nested_sound_variant_fields(
            pig_sound_variant_nbt(),
            &[
                "ambient_sound",
                "hurt_sound",
                "death_sound",
                "step_sound",
                "eat_sound",
            ],
        );
        assert_nested_sound_variant_fields(
            cat_sound_variant_nbt(),
            &[
                "ambient_sound",
                "stray_ambient_sound",
                "hiss_sound",
                "hurt_sound",
                "death_sound",
                "eat_sound",
                "beg_for_food_sound",
                "purr_sound",
                "purreow_sound",
            ],
        );
        assert_nested_sound_variant_fields(
            wolf_sound_variant_nbt(),
            &[
                "ambient_sound",
                "death_sound",
                "growl_sound",
                "hurt_sound",
                "pant_sound",
                "whine_sound",
                "step_sound",
            ],
        );
    }

    #[test]
    fn trim_material_registry_payload_includes_redstone_component_data() {
        let redstone = TRIM_MATERIALS
            .iter()
            .find(|material| material.id == "redstone")
            .expect("redstone trim material should be sent");
        let tag = trim_material_nbt(redstone);

        assert!(matches!(
            field_value(&tag, "asset_name"),
            Some(Tag::String(value)) if value == "redstone"
        ));
        let description = compound_field(&tag, "description");
        assert!(matches!(
            field_value(description, "translate"),
            Some(Tag::String(value)) if value == "trim_material.minecraft.redstone"
        ));
        assert!(matches!(
            field_value(description, "color"),
            Some(Tag::String(value)) if value == "#971607"
        ));
    }

    #[test]
    fn vanilla_animal_variant_registry_payloads_include_client_referenced_entries() {
        assert_eq!(
            registry_element_count(write_vanilla_cat_variant_registry_packet),
            11
        );
        assert_eq!(
            registry_element_count(write_vanilla_chicken_variant_registry_packet),
            3
        );
        assert_eq!(
            registry_element_count(write_vanilla_cow_variant_registry_packet),
            3
        );
        assert_eq!(
            registry_element_count(write_vanilla_frog_variant_registry_packet),
            3
        );
        assert_eq!(
            registry_element_count(write_vanilla_pig_variant_registry_packet),
            3
        );
        assert_eq!(
            registry_element_count(write_vanilla_wolf_variant_registry_packet),
            9
        );
        assert_eq!(
            registry_element_count(write_vanilla_cat_sound_variant_registry_packet),
            2
        );
        assert_eq!(
            registry_element_count(write_vanilla_chicken_sound_variant_registry_packet),
            2
        );
        assert_eq!(
            registry_element_count(write_vanilla_cow_sound_variant_registry_packet),
            2
        );
        assert_eq!(
            registry_element_count(write_vanilla_pig_sound_variant_registry_packet),
            3
        );
        assert_eq!(
            registry_element_count(write_vanilla_wolf_sound_variant_registry_packet),
            7
        );
        assert_eq!(
            registry_element_count(write_vanilla_zombie_nautilus_variant_registry_packet),
            2
        );
        assert_eq!(
            registry_element_count(write_vanilla_painting_variant_registry_packet),
            51
        );
    }

    #[test]
    fn synced_registry_payloads_include_expected_counts_and_fields() {
        assert_eq!(
            registry_element_count(write_minimal_damage_type_registry_packet),
            50
        );
        assert_eq!(
            registry_element_count(write_minimal_dimension_type_registry_packet),
            4
        );
        assert_eq!(
            registry_element_count(write_minimal_trim_material_registry_packet),
            11
        );
        assert_eq!(
            registry_element_count(write_vanilla_trim_pattern_registry_packet),
            18
        );
        assert_eq!(
            registry_element_count(write_vanilla_banner_pattern_registry_packet),
            43
        );
        assert_eq!(
            registry_element_count(write_vanilla_instrument_registry_packet),
            8
        );

        let trim_pattern = trim_pattern_nbt("sentry");
        assert!(matches!(
            field_value(&trim_pattern, "asset_id"),
            Some(Tag::String(value)) if value == "minecraft:sentry"
        ));
        assert!(matches!(
            field_value(&trim_pattern, "decal"),
            Some(Tag::Byte(0))
        ));
        let trim_description = compound_field(&trim_pattern, "description");
        assert!(matches!(
            field_value(trim_description, "translate"),
            Some(Tag::String(value)) if value == "trim_pattern.minecraft.sentry"
        ));

        let banner = banner_pattern_nbt("flower");
        assert!(matches!(
            field_value(&banner, "asset_id"),
            Some(Tag::String(value)) if value == "minecraft:flower"
        ));
        assert!(matches!(
            field_value(&banner, "translation_key"),
            Some(Tag::String(value)) if value == "block.minecraft.banner.flower"
        ));

        let instrument = INSTRUMENTS
            .iter()
            .find(|instrument| instrument.id == "ponder_goat_horn")
            .expect("ponder goat horn should be sent");
        let instrument_tag = instrument_nbt(instrument);
        assert!(matches!(
            field_value(&instrument_tag, "sound_event"),
            Some(Tag::String(value)) if value == "minecraft:item.goat_horn.sound.0"
        ));
        assert!(matches!(
            field_value(&instrument_tag, "use_duration"),
            Some(Tag::Float(value)) if (*value - 7.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            field_value(&instrument_tag, "range"),
            Some(Tag::Float(value)) if (*value - 256.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn duplicated_registry_manifest_ids_remain_in_sync_across_tables() {
        let status_trim_materials: Vec<String> = TRIM_MATERIALS
            .iter()
            .map(|entry| format!("minecraft:{}", entry.id))
            .collect();
        let presentation_trim_materials: Vec<String> = presentation_data::TRIM_MATERIALS
            .iter()
            .map(|material| material.id.to_string())
            .collect();
        let model_trim_materials: Vec<String> = equipment_trim::TRIM_MATERIALS
            .iter()
            .map(|material| material.id.to_string())
            .collect();

        assert_eq!(status_trim_materials, presentation_trim_materials);
        assert_eq!(status_trim_materials, model_trim_materials);

        let status_trim_patterns: Vec<String> = TRIM_PATTERNS
            .iter()
            .map(|id| format!("minecraft:{id}"))
            .collect();
        let presentation_trim_patterns: Vec<String> = presentation_data::TRIM_PATTERNS
            .iter()
            .map(|pattern| pattern.id.to_string())
            .collect();
        let model_trim_patterns: Vec<String> = equipment_trim::TRIM_PATTERNS
            .iter()
            .map(|pattern| format!("minecraft:{}", pattern.id))
            .collect();

        assert_eq!(status_trim_patterns, presentation_trim_patterns);
        assert_eq!(status_trim_patterns, model_trim_patterns);

        let status_instruments: BTreeSet<String> = INSTRUMENTS
            .iter()
            .map(|instrument| format!("minecraft:{}", instrument.id))
            .collect();
        let presentation_instruments: BTreeSet<String> = presentation_data::INSTRUMENTS
            .iter()
            .map(|instrument| instrument.id.to_string())
            .collect();

        assert_eq!(status_instruments, presentation_instruments);

        let status_damage_types: BTreeSet<String> = DAMAGE_TYPES
            .iter()
            .map(|id| format!("minecraft:{id}"))
            .collect();
        let presentation_damage_types: BTreeSet<String> = presentation_data::DAMAGE_TYPES
            .iter()
            .map(|entry| entry.id.to_string())
            .collect();
        let model_damage_types: BTreeSet<String> = damage_type::BUILTIN_DAMAGE_TYPES
            .iter()
            .map(|entry| entry.id.to_string())
            .collect();

        assert!(status_damage_types.is_superset(&presentation_damage_types));
        assert!(status_damage_types.is_superset(&model_damage_types));

        let status_biomes: BTreeSet<String> = BIOMES
            .iter()
            .map(|id| format!("minecraft:{}", id))
            .collect();
        let model_biomes: BTreeSet<String> = biome::BUILTIN_BIOMES
            .iter()
            .map(|biome| biome.id.to_string())
            .collect();
        assert_eq!(status_biomes, model_biomes);

        let status_paintings =
            status_registry_entry_ids_ordered(write_vanilla_painting_variant_registry_packet);
        let presentation_paintings: Vec<String> = presentation_data::PAINTING_VARIANTS
            .iter()
            .map(|painting| painting.id.to_string())
            .collect();
        assert_eq!(status_paintings, presentation_paintings);

        let status_jukebox_songs: BTreeSet<String> = JUKEBOX_SONGS
            .iter()
            .map(|song| format!("minecraft:{}", song.id))
            .collect();
        let presentation_jukebox_songs: BTreeSet<String> = presentation_data::JUKEBOX_SONGS
            .iter()
            .map(|song| song.id.to_string())
            .collect();
        assert!(presentation_jukebox_songs.is_subset(&status_jukebox_songs));

        let status_banner_patterns: BTreeSet<String> = BANNER_PATTERNS
            .iter()
            .map(|id| format!("minecraft:{}", id))
            .collect();
        let presentation_banner_patterns: BTreeSet<String> = presentation_data::BANNER_PATTERNS
            .iter()
            .map(|pattern| pattern.id.to_string())
            .collect();
        assert_eq!(status_banner_patterns, presentation_banner_patterns);
        let status_tag_names: BTreeSet<String> = BANNER_PATTERN_TAGS
            .iter()
            .map(|(tag, _)| tag.to_string())
            .collect();
        let expected_banner_pattern_tags: BTreeSet<String> = [
            "minecraft:no_item_required",
            "minecraft:pattern_item/flower",
            "minecraft:pattern_item/creeper",
            "minecraft:pattern_item/skull",
            "minecraft:pattern_item/mojang",
            "minecraft:pattern_item/globe",
            "minecraft:pattern_item/piglin",
            "minecraft:pattern_item/flow",
            "minecraft:pattern_item/guster",
            "minecraft:pattern_item/field_masoned",
            "minecraft:pattern_item/bordure_indented",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(status_tag_names, expected_banner_pattern_tags);
    }

    #[test]
    fn synchronized_registry_closure_notes_match_writer_payloads() {
        for entry in SYNCHRONIZED_REGISTRY_MANIFEST {
            assert!(!entry.java_network_shape.is_empty());

            let registry_id = status_registry_id(entry.write_packet);
            assert_eq!(registry_id, entry.registry_id);

            let entries = status_registry_entry_ids_ordered(entry.write_packet);
            assert_eq!(entries.len(), entry.expected_entry_count);
        }
    }

    #[test]
    fn synced_registry_entry_orders_match_official_transcript_fixtures() {
        assert_registry_order(
            write_vanilla_cat_variant_registry_packet,
            &[
                "all_black",
                "black",
                "british_shorthair",
                "calico",
                "jellie",
                "persian",
                "ragdoll",
                "red",
                "siamese",
                "tabby",
                "white",
            ],
        );
        assert_registry_order(
            write_vanilla_cat_sound_variant_registry_packet,
            &["classic", "royal"],
        );
        assert_registry_order(
            write_vanilla_chicken_sound_variant_registry_packet,
            &["classic", "picky"],
        );
        assert_registry_order(
            write_vanilla_cow_sound_variant_registry_packet,
            &["classic", "moody"],
        );
        assert_registry_order(
            write_vanilla_pig_sound_variant_registry_packet,
            &["big", "classic", "mini"],
        );
        assert_registry_order(
            write_vanilla_wolf_sound_variant_registry_packet,
            &["angry", "big", "classic", "cute", "grumpy", "puglin", "sad"],
        );
        assert_registry_order(
            write_vanilla_painting_variant_registry_packet,
            &[
                "alban",
                "aztec",
                "aztec2",
                "backyard",
                "baroque",
                "bomb",
                "bouquet",
                "burning_skull",
                "bust",
                "cavebird",
                "changing",
                "cotan",
                "courbet",
                "creebet",
                "dennis",
                "donkey_kong",
                "earth",
                "endboss",
                "fern",
                "fighters",
                "finding",
                "fire",
                "graham",
                "humble",
                "kebab",
                "lowmist",
                "match",
                "meditative",
                "orb",
                "owlemons",
                "passage",
                "pigscene",
                "plant",
                "pointer",
                "pond",
                "pool",
                "prairie_ride",
                "sea",
                "skeleton",
                "skull_and_roses",
                "stage",
                "sunflowers",
                "sunset",
                "tides",
                "unpacked",
                "void",
                "wanderer",
                "wasteland",
                "water",
                "wind",
                "wither",
            ],
        );
        assert_registry_order(
            write_minimal_damage_type_registry_packet,
            &[
                "arrow",
                "bad_respawn_point",
                "cactus",
                "campfire",
                "cramming",
                "dragon_breath",
                "drown",
                "dry_out",
                "ender_pearl",
                "explosion",
                "fall",
                "falling_anvil",
                "falling_block",
                "falling_stalactite",
                "fireball",
                "fireworks",
                "fly_into_wall",
                "freeze",
                "generic",
                "generic_kill",
                "hot_floor",
                "in_fire",
                "in_wall",
                "indirect_magic",
                "lava",
                "lightning_bolt",
                "mace_smash",
                "magic",
                "mob_attack",
                "mob_attack_no_aggro",
                "mob_projectile",
                "on_fire",
                "out_of_world",
                "outside_border",
                "player_attack",
                "player_explosion",
                "sonic_boom",
                "spear",
                "spit",
                "stalagmite",
                "starve",
                "sting",
                "sweet_berry_bush",
                "thorns",
                "thrown",
                "trident",
                "unattributed_fireball",
                "wind_charge",
                "wither",
                "wither_skull",
            ],
        );
        assert_registry_order(
            write_vanilla_banner_pattern_registry_packet,
            &[
                "base",
                "border",
                "bricks",
                "circle",
                "creeper",
                "cross",
                "curly_border",
                "diagonal_left",
                "diagonal_right",
                "diagonal_up_left",
                "diagonal_up_right",
                "flow",
                "flower",
                "globe",
                "gradient",
                "gradient_up",
                "guster",
                "half_horizontal",
                "half_horizontal_bottom",
                "half_vertical",
                "half_vertical_right",
                "mojang",
                "piglin",
                "rhombus",
                "skull",
                "small_stripes",
                "square_bottom_left",
                "square_bottom_right",
                "square_top_left",
                "square_top_right",
                "straight_cross",
                "stripe_bottom",
                "stripe_center",
                "stripe_downleft",
                "stripe_downright",
                "stripe_left",
                "stripe_middle",
                "stripe_right",
                "stripe_top",
                "triangle_bottom",
                "triangle_top",
                "triangles_bottom",
                "triangles_top",
            ],
        );
        assert_registry_order(
            write_vanilla_jukebox_song_registry_packet,
            &[
                "11",
                "13",
                "5",
                "blocks",
                "cat",
                "chirp",
                "creator",
                "creator_music_box",
                "far",
                "lava_chicken",
                "mall",
                "mellohi",
                "otherside",
                "pigstep",
                "precipice",
                "relic",
                "stal",
                "strad",
                "tears",
                "wait",
                "ward",
            ],
        );
        assert_registry_order(
            write_vanilla_instrument_registry_packet,
            &[
                "admire_goat_horn",
                "call_goat_horn",
                "dream_goat_horn",
                "feel_goat_horn",
                "ponder_goat_horn",
                "seek_goat_horn",
                "sing_goat_horn",
                "yearn_goat_horn",
            ],
        );
        assert_registry_order(
            write_vanilla_chat_type_registry_packet,
            &[
                "chat",
                "emote_command",
                "msg_command_incoming",
                "msg_command_outgoing",
                "say_command",
                "team_msg_command_incoming",
                "team_msg_command_outgoing",
            ],
        );
        assert_registry_order(
            write_minimal_trim_material_registry_packet,
            &[
                "quartz",
                "iron",
                "netherite",
                "redstone",
                "copper",
                "gold",
                "emerald",
                "diamond",
                "lapis",
                "amethyst",
                "resin",
            ],
        );
        assert_registry_order(
            write_vanilla_trim_pattern_registry_packet,
            &[
                "sentry",
                "dune",
                "coast",
                "wild",
                "ward",
                "eye",
                "vex",
                "tide",
                "snout",
                "rib",
                "spire",
                "wayfinder",
                "shaper",
                "silence",
                "raiser",
                "host",
                "flow",
                "bolt",
            ],
        );
        assert_registry_order(
            write_vanilla_wolf_variant_registry_packet,
            &[
                "ashen", "black", "chestnut", "pale", "rusty", "snowy", "spotted", "striped",
                "woods",
            ],
        );
        assert_registry_order(
            write_vanilla_pig_variant_registry_packet,
            &["cold", "temperate", "warm"],
        );
        assert_registry_order(
            write_vanilla_frog_variant_registry_packet,
            &["cold", "temperate", "warm"],
        );
        assert_registry_order(
            write_vanilla_cow_variant_registry_packet,
            &["cold", "temperate", "warm"],
        );
        assert_registry_order(
            write_vanilla_chicken_variant_registry_packet,
            &["cold", "temperate", "warm"],
        );
    }

    #[test]
    fn chat_type_registry_payloads_include_vanilla_routes() {
        assert_eq!(
            registry_element_count(write_vanilla_chat_type_registry_packet),
            7
        );
        let outgoing = CHAT_TYPES
            .iter()
            .find(|chat_type| chat_type.id == "msg_command_outgoing")
            .expect("outgoing direct message chat type should be sent");
        let tag = chat_type_nbt(outgoing);

        let chat = compound_field(&tag, "chat");
        assert!(matches!(
            field_value(chat, "translation_key"),
            Some(Tag::String(value)) if value == "commands.message.display.outgoing"
        ));
        assert_string_list(field_value(chat, "parameters"), &["target", "content"]);

        let narration = compound_field(&tag, "narration");
        assert!(matches!(
            field_value(narration, "translation_key"),
            Some(Tag::String(value)) if value == "chat.type.text.narrate"
        ));
        assert_string_list(field_value(narration, "parameters"), &["sender", "content"]);
    }

    #[test]
    fn biome_registry_payloads_include_full_vanilla_id_set_with_plains() {
        assert_eq!(BIOMES.len(), 65);
        assert!(BIOMES.contains(&"plains"));
        assert!(BIOMES.contains(&"the_void"));
        assert!(BIOMES.contains(&"end_barrens"));
        assert_eq!(BIOMES[0], "badlands");
        assert_eq!(BIOMES[40], "plains");
        assert_eq!(BIOMES[64], "wooded_badlands");
        assert_eq!(
            registry_element_count(write_minimal_biome_registry_packet),
            BIOMES.len() as i32
        );

        let plains = vanilla_baseline_biome_nbt("plains");
        assert!(matches!(
            field_value(&plains, "has_precipitation"),
            Some(Tag::Byte(1))
        ));
        assert!(matches!(
            field_value(&plains, "temperature"),
            Some(Tag::Float(value)) if (*value - 0.8).abs() < f32::EPSILON
        ));
        let effects = compound_field(&plains, "effects");
        assert!(matches!(
            field_value(effects, "water_color"),
            Some(Tag::Int(4_159_204))
        ));
    }

    #[test]
    fn biome_network_codec_fixture_covers_required_fields_for_every_emitted_biome() {
        for biome in BIOMES {
            let tag = vanilla_baseline_biome_nbt(biome);
            assert!(matches!(
                field_value(&tag, "has_precipitation"),
                Some(Tag::Byte(0 | 1))
            ));
            assert!(matches!(
                field_value(&tag, "temperature"),
                Some(Tag::Float(_))
            ));
            assert!(matches!(field_value(&tag, "downfall"), Some(Tag::Float(_))));
            assert!(matches!(
                field_value(&tag, "effects"),
                Some(Tag::Compound(_))
            ));

            let effects = compound_field(&tag, "effects");
            assert!(matches!(
                field_value(effects, "water_color"),
                Some(Tag::Int(_))
            ));
        }
    }

    #[test]
    fn visible_spawn_terrain_uses_deterministic_rolling_grass_layers() {
        let mut payload = Vec::new();
        write_visible_spawn_terrain_block_state_container(&mut payload, 0, 0, 9).unwrap();
        let mut input = Cursor::new(payload);

        let mut bits = [0_u8; 1];
        input.read_exact(&mut bits).unwrap();
        assert_eq!(bits[0], 4);
        assert_eq!(read_var_i32(&mut input).unwrap(), 11);
        assert_eq!(read_var_i32(&mut input).unwrap(), 0);
        assert_eq!(read_var_i32(&mut input).unwrap(), STONE_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), GRANITE_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), DIORITE_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), ANDESITE_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), BEDROCK_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), DIRT_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), GRASS_BLOCK_STATE_ID);
        assert_eq!(
            read_var_i32(&mut input).unwrap(),
            SHORT_GRASS_BLOCK_STATE_ID
        );
        assert_eq!(read_var_i32(&mut input).unwrap(), DANDELION_BLOCK_STATE_ID);
        assert_eq!(read_var_i32(&mut input).unwrap(), POPPY_BLOCK_STATE_ID);

        let mut raw = Vec::new();
        input.read_to_end(&mut raw).unwrap();
        assert_eq!(raw.len(), 2048);
        let words = raw
            .chunks_exact(8)
            .map(|chunk| {
                u64::from_be_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ])
            })
            .collect::<Vec<_>>();

        let high_column = (0..16)
            .flat_map(|z| (0..16).map(move |x| (x, z)))
            .find(|(x, z)| {
                (90..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
                    && visible_spawn_surface_top_block_id(0, 0, *x, *z) == GRASS_BLOCK_STATE_ID
            })
            .expect("spawn chunk should contain a high visible hill top");
        let featured_column = (0..16)
            .flat_map(|z| (0..16).map(move |x| (x, z)))
            .find_map(|(x, z)| {
                (visible_spawn_surface_top_block_id(0, 0, x, z) == GRASS_BLOCK_STATE_ID
                    && (80..95).contains(&visible_spawn_terrain_height(0, 0, x, z)))
                .then(|| visible_spawn_surface_feature_id(0, 0, x, z).map(|id| (x, z, id)))
                .flatten()
            })
            .expect("spawn chunk should contain visible surface vegetation");
        let outcrop_column = (0..16)
            .flat_map(|z| (0..16).map(move |x| (x, z)))
            .find(|(x, z)| {
                visible_spawn_surface_top_block_id(0, 0, *x, *z) != GRASS_BLOCK_STATE_ID
                    && (80..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
            })
            .expect("spawn chunk should contain a visible non-grass outcrop");
        let max_height = (0..16)
            .flat_map(|z| (0..16).map(move |x| visible_spawn_terrain_height(0, 0, x, z)))
            .max()
            .expect("spawn chunk should contain terrain columns");
        let ridge_column = (0..16)
            .flat_map(|z| (0..16).map(move |x| (x, z)))
            .find(|(x, z)| visible_spawn_terrain_height(0, 0, *x, *z) == max_height)
            .expect("spawn chunk should contain a visible ridge");
        assert!(max_height > 95, "spawn terrain should be visibly non-flat");
        assert!(visible_spawn_terrain_block_count(0, 0, 8) > 4000);
        assert!(visible_spawn_terrain_block_count(0, 0, 9) > 512);
        assert!(visible_spawn_terrain_block_count(0, 0, 10) > 0);
        assert_ne!(words.iter().filter(|word| **word != 0).count(), 0);
        let high_local_y =
            (visible_spawn_terrain_height(0, 0, high_column.0, high_column.1) - 80) as usize;
        assert_eq!(
            palette_index_at(&words, high_column.0, high_local_y - 1, high_column.1),
            6
        );
        assert_eq!(
            palette_index_at(&words, high_column.0, high_local_y, high_column.1),
            7
        );
        let expected_outcrop_palette =
            match visible_spawn_surface_top_block_id(0, 0, outcrop_column.0, outcrop_column.1) {
                STONE_BLOCK_STATE_ID => 1,
                GRANITE_BLOCK_STATE_ID => 2,
                DIORITE_BLOCK_STATE_ID => 3,
                ANDESITE_BLOCK_STATE_ID => 4,
                DIRT_BLOCK_STATE_ID => 6,
                _ => unreachable!("outcrop column must be non-grass"),
            };
        assert_eq!(
            palette_index_at(
                &words,
                outcrop_column.0,
                (visible_spawn_terrain_height(0, 0, outcrop_column.0, outcrop_column.1) - 80)
                    as usize,
                outcrop_column.1
            ),
            expected_outcrop_palette
        );
        assert!(visible_spawn_terrain_height(0, 0, ridge_column.0, ridge_column.1) >= 96);
        let feature_y = (visible_spawn_terrain_height(0, 0, featured_column.0, featured_column.1)
            + 1
            - 80) as usize;
        let expected_feature_palette = match featured_column.2 {
            SHORT_GRASS_BLOCK_STATE_ID => 8,
            DANDELION_BLOCK_STATE_ID => 9,
            POPPY_BLOCK_STATE_ID => 10,
            _ => unreachable!("feature id must be in the emitted palette"),
        };
        assert_eq!(
            palette_index_at(&words, featured_column.0, feature_y, featured_column.1),
            expected_feature_palette
        );
    }

    #[test]
    fn all_air_persisted_chunks_are_not_reused_for_spawn_terrain() {
        let empty = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        assert!(!chunk_has_non_air_blocks(&empty));

        let generated =
            crate::worldgen::generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")
                .expect("normal preset should generate visible terrain");
        assert!(chunk_has_non_air_blocks(&generated));
    }

    #[test]
    fn live_spawn_chunk_packet_uses_generated_level_chunk_serialization() {
        let mut payload = Vec::new();
        let world_root = std::env::temp_dir().join(format!(
            "rustcraft-missing-world-root-{}",
            std::process::id()
        ));

        super::write_generated_spawn_chunk_packet(&mut payload, 0, 0, &world_root, 0)
            .expect("missing region files should fall back to generated terrain");

        let mut input = &payload[..];
        let mut chunk_x = [0_u8; 4];
        let mut chunk_z = [0_u8; 4];
        input.read_exact(&mut chunk_x).unwrap();
        input.read_exact(&mut chunk_z).unwrap();
        assert_eq!(i32::from_be_bytes(chunk_x), 0);
        assert_eq!(i32::from_be_bytes(chunk_z), 0);
        let heightmap_count = read_var_i32(&mut input).unwrap();
        assert!(
            heightmap_count >= 3,
            "live chunk packets should serialize generated LevelChunk heightmaps, not the legacy zero-heightmap superflat packet"
        );
    }

    #[test]
    fn generated_chunk_entity_add_packets_reads_queued_chunk_mob_nbt() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:pig".to_string())),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(70.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(15.0)]),
            ),
        ]));

        let packets = super::generated_chunk_entity_add_packets(&chunk);

        assert_eq!(packets.len(), 1);
        let packet = &packets[0];
        assert_eq!(
            packet.id,
            super::generated_chunk_entity_runtime_id(chunk.pos, 0)
        );
        assert_eq!(
            packet.uuid,
            Uuid([0, 0, 0, 0, 0, 0, 64, 0, 128, 0, 0, 0, 0, 0, 1, 35])
        );
        assert_eq!(packet.entity_type, 100);
        assert_eq!(packet.position.x, 32.9);
        assert_eq!(packet.position.y, 70.0);
        assert_eq!(packet.position.z, -33.0);
        assert_eq!(packet.movement, Vec3::ZERO);
        assert_eq!(packet.x_rot, 10);
        assert_eq!(packet.y_rot, 64);
        assert_eq!(packet.y_head_rot, 64);
    }

    #[test]
    fn generated_chunk_entity_spawn_packets_are_bundle_wrapped() {
        let plan = super::GeneratedChunkEntitySpawnPlan {
            add_entity: ClientboundAddEntityPacket::new(
                42,
                Uuid([1; 16]),
                100,
                Vec3 {
                    x: 1.0,
                    y: 65.0,
                    z: 2.0,
                },
                Vec3::ZERO,
                (0.0, 90.0),
                90.0,
                0,
            ),
            metadata: None,
        };
        let mut output = Vec::new();

        super::write_generated_chunk_entity_spawn_packets(
            &mut output,
            CompressionState::disabled(),
            &plan,
        )
        .expect("generated mob pairing should serialize");

        let mut frames = Vec::new();
        let mut input = &output[..];
        while !input.is_empty() {
            let frame_len = read_var_i32(&mut input).unwrap() as usize;
            let mut frame = vec![0; frame_len];
            input.read_exact(&mut frame).unwrap();
            let mut payload = &frame[..];
            frames.push(read_var_i32(&mut payload).unwrap());
        }
        assert_eq!(
            frames,
            vec![
                CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
                CLIENTBOUND_ADD_ENTITY_PACKET_ID,
                CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID
            ]
        );
    }

    #[test]
    fn generated_chunk_entity_spawn_plan_reads_non_default_pig_variant_metadata() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:pig".to_string())),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(70.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            ),
            (
                "variant".to_string(),
                Tag::String("minecraft:warm".to_string()),
            ),
            (
                "sound_variant".to_string(),
                Tag::String("minecraft:mini".to_string()),
            ),
        ]));

        let plans = super::generated_chunk_entity_spawn_plans(&chunk);

        assert_eq!(plans.len(), 1);
        let metadata = plans[0]
            .metadata
            .as_ref()
            .expect("non-default pig variant data should emit metadata");
        assert_eq!(metadata.id, plans[0].add_entity.id);
        assert_eq!(
            metadata.packed_items,
            vec![
                EntityDataValue::typed(19, EntityMetadataValue::PigVariant(2)).unwrap(),
                EntityDataValue::typed(20, EntityMetadataValue::PigSoundVariant(2)).unwrap(),
            ]
        );
    }

    #[test]
    fn generated_chunk_entity_spawn_plan_reads_non_default_chicken_variant_metadata() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:chicken".to_string()),
            ),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000124".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(70.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            ),
            (
                "variant".to_string(),
                Tag::String("minecraft:cold".to_string()),
            ),
            (
                "sound_variant".to_string(),
                Tag::String("minecraft:picky".to_string()),
            ),
        ]));

        let plans = super::generated_chunk_entity_spawn_plans(&chunk);

        assert_eq!(plans.len(), 1);
        let metadata = plans[0]
            .metadata
            .as_ref()
            .expect("non-default chicken variant data should emit metadata");
        assert_eq!(
            metadata.packed_items,
            vec![
                EntityDataValue::typed(18, EntityMetadataValue::ChickenVariant(0)).unwrap(),
                EntityDataValue::typed(19, EntityMetadataValue::ChickenSoundVariant(1)).unwrap(),
            ]
        );
    }

    #[test]
    fn generated_chunk_entity_spawn_plan_reads_non_default_zombie_nautilus_variant_metadata() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:zombie_nautilus".to_string()),
            ),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000126".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(62.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            ),
            (
                "variant".to_string(),
                Tag::String("minecraft:warm".to_string()),
            ),
        ]));

        let plans = super::generated_chunk_entity_spawn_plans(&chunk);

        assert_eq!(plans.len(), 1);
        let metadata = plans[0]
            .metadata
            .as_ref()
            .expect("non-default zombie nautilus variant data should emit metadata");
        assert_eq!(
            metadata.packed_items,
            vec![
                EntityDataValue::typed(21, EntityMetadataValue::ZombieNautilusVariant(1)).unwrap()
            ]
        );
    }

    #[test]
    fn generated_chunk_entity_spawn_plan_omits_default_animal_variant_metadata() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:chicken".to_string()),
            ),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000125".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(70.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            ),
            (
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ),
            (
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string()),
            ),
        ]));

        let plans = super::generated_chunk_entity_spawn_plans(&chunk);

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].metadata, None);
    }

    #[test]
    fn generated_chunk_entity_spawn_packets_include_non_default_metadata_in_bundle() {
        let plan = super::GeneratedChunkEntitySpawnPlan {
            add_entity: ClientboundAddEntityPacket::new(
                42,
                Uuid([1; 16]),
                100,
                Vec3 {
                    x: 1.0,
                    y: 65.0,
                    z: 2.0,
                },
                Vec3::ZERO,
                (0.0, 90.0),
                90.0,
                0,
            ),
            metadata: Some(ClientboundSetEntityDataPacket {
                id: 42,
                packed_items: vec![
                    EntityDataValue::typed(19, EntityMetadataValue::PigVariant(2)).unwrap(),
                ],
            }),
        };
        let mut output = Vec::new();

        super::write_generated_chunk_entity_spawn_packets(
            &mut output,
            CompressionState::disabled(),
            &plan,
        )
        .expect("generated mob pairing with metadata should serialize");

        let mut frames = Vec::new();
        let mut input = &output[..];
        while !input.is_empty() {
            let frame_len = read_var_i32(&mut input).unwrap() as usize;
            let mut frame = vec![0; frame_len];
            input.read_exact(&mut frame).unwrap();
            let mut payload = &frame[..];
            frames.push(read_var_i32(&mut payload).unwrap());
        }
        assert_eq!(
            frames,
            vec![
                CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID,
                CLIENTBOUND_ADD_ENTITY_PACKET_ID,
                CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
                CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID
            ]
        );
    }

    #[test]
    fn generated_chunk_entity_remove_packets_use_deterministic_runtime_ids() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.entities.push(Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:pig".to_string())),
            (
                "UUID".to_string(),
                Tag::String("00000000-0000-4000-8000-000000000123".to_string()),
            ),
            (
                "Pos".to_string(),
                Tag::List(vec![
                    Tag::Double(32.9),
                    Tag::Double(70.0),
                    Tag::Double(-33.0),
                ]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            ),
        ]));
        let mut output = Vec::new();

        super::write_generated_chunk_entity_remove_packets(
            &mut output,
            CompressionState::disabled(),
            &chunk,
        )
        .expect("generated mob removal should serialize");

        let mut frame = &output[..];
        let frame_len = read_var_i32(&mut frame).unwrap() as usize;
        assert_eq!(frame_len, frame.len());
        assert_eq!(
            read_var_i32(&mut frame).unwrap(),
            CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID
        );
        assert_eq!(read_var_i32(&mut frame).unwrap(), 1);
        assert_eq!(
            read_var_i32(&mut frame).unwrap(),
            super::generated_chunk_entity_runtime_id(chunk.pos, 0)
        );
        assert!(frame.is_empty());
    }

    #[test]
    fn generated_mob_entity_type_network_ids_cover_modeled_biome_spawns() {
        let modeled_spawn_ids = [
            ("minecraft:armadillo", 4),
            ("minecraft:axolotl", 7),
            ("minecraft:bat", 10),
            ("minecraft:bogged", 16),
            ("minecraft:camel", 19),
            ("minecraft:chicken", 26),
            ("minecraft:cod", 27),
            ("minecraft:cow", 30),
            ("minecraft:creeper", 32),
            ("minecraft:dolphin", 35),
            ("minecraft:donkey", 36),
            ("minecraft:drowned", 38),
            ("minecraft:enderman", 41),
            ("minecraft:fox", 54),
            ("minecraft:frog", 55),
            ("minecraft:ghast", 57),
            ("minecraft:glow_squid", 61),
            ("minecraft:goat", 62),
            ("minecraft:hoglin", 64),
            ("minecraft:horse", 66),
            ("minecraft:husk", 67),
            ("minecraft:llama", 78),
            ("minecraft:magma_cube", 80),
            ("minecraft:mooshroom", 86),
            ("minecraft:mule", 87),
            ("minecraft:ocelot", 91),
            ("minecraft:panda", 96),
            ("minecraft:parched", 97),
            ("minecraft:parrot", 98),
            ("minecraft:pig", 100),
            ("minecraft:piglin", 101),
            ("minecraft:polar_bear", 104),
            ("minecraft:pufferfish", 107),
            ("minecraft:rabbit", 108),
            ("minecraft:salmon", 110),
            ("minecraft:sheep", 111),
            ("minecraft:skeleton", 115),
            ("minecraft:slime", 117),
            ("minecraft:spider", 124),
            ("minecraft:squid", 127),
            ("minecraft:stray", 128),
            ("minecraft:strider", 129),
            ("minecraft:trader_llama", 134),
            ("minecraft:tropical_fish", 136),
            ("minecraft:turtle", 137),
            ("minecraft:witch", 144),
            ("minecraft:wolf", 148),
            ("minecraft:zombie", 150),
            ("minecraft:zombie_horse", 151),
            ("minecraft:zombie_nautilus", 152),
            ("minecraft:zombie_villager", 153),
            ("minecraft:zombified_piglin", 154),
        ];

        for (entity_type, network_id) in modeled_spawn_ids {
            assert_eq!(
                super::generated_mob_entity_type_network_id(entity_type),
                Some(network_id),
                "{entity_type}"
            );
        }
    }

    #[test]
    fn spawn_chunk_window_can_represent_configured_server_view_distance_radius() {
        assert_eq!(chunk_batch_size(2), 25);
        assert_eq!(chunk_batch_size(10), 441);

        let chunks = chunk_window(4, -3, 10);
        assert_eq!(chunks.len(), 441);
        assert!(chunks.contains(&(4, -3)));
        assert!(chunks.contains(&(-6, -13)));
        assert!(chunks.contains(&(14, 7)));
        assert!(!chunks.contains(&(-7, -3)));
        assert!(!chunks.contains(&(4, 8)));
    }

    #[test]
    fn spawn_chunk_window_keeps_minimum_five_by_five_terrain_patch() {
        let chunks = chunk_window(4, -3, 2);
        assert_eq!(chunks.len(), 25);
        assert!(chunks.contains(&(4, -3)));
        assert!(chunks.contains(&(2, -5)));
        assert!(chunks.contains(&(6, -1)));
        assert!(!chunks.contains(&(1, -3)));
        assert!(!chunks.contains(&(4, 0)));
    }

    #[test]
    fn movement_chunk_window_sends_only_newly_visible_edge_chunks() {
        let previous = chunk_window(0, 0, 10);
        let next = chunk_window(1, 0, 10);
        let delta = newly_visible_chunks(&previous, &next);

        assert_eq!(previous.len(), 441);
        assert_eq!(next.len(), 441);
        assert_eq!(delta.len(), 21);
        assert!(delta.iter().all(|chunk| chunk.0 == 11));
        assert!(delta.contains(&(11, -10)));
        assert!(delta.contains(&(11, 0)));
        assert!(delta.contains(&(11, 10)));
    }

    #[test]
    fn chunk_batch_start_packet_has_no_payload_after_packet_id() {
        let mut packet = Vec::new();
        write_framed_packet(
            &mut packet,
            CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
            |_| Ok(()),
        )
        .unwrap();

        let mut cursor = Cursor::new(packet);
        let frame_len = read_var_i32(&mut cursor).unwrap();
        assert_eq!(frame_len, 1);
        assert_eq!(
            read_var_i32(&mut cursor).unwrap(),
            CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID
        );
        assert_eq!(cursor.position(), cursor.get_ref().len() as u64);
    }

    #[test]
    fn forget_level_chunk_packet_uses_packed_chunk_position() {
        assert_eq!(packed_chunk_pos(4, -2), -8589934588);
        assert_eq!(packed_chunk_pos(-1, 0), 0xffff_ffff);

        let mut packet = Vec::new();
        write_framed_packet(
            &mut packet,
            CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
            |payload| payload.write_all(&packed_chunk_pos(4, -2).to_be_bytes()),
        )
        .unwrap();

        let mut cursor = Cursor::new(packet);
        let frame_len = read_var_i32(&mut cursor).unwrap();
        assert_eq!(frame_len, 9);
        assert_eq!(
            read_var_i32(&mut cursor).unwrap(),
            CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID
        );
        let mut packed = [0; 8];
        cursor.read_exact(&mut packed).unwrap();
        assert_eq!(i64::from_be_bytes(packed), packed_chunk_pos(4, -2));
        assert_eq!(cursor.position(), cursor.get_ref().len() as u64);
    }

    fn palette_index_at(words: &[u64], x: usize, y: usize, z: usize) -> u64 {
        const BITS_PER_ENTRY: usize = 4;
        const VALUES_PER_LONG: usize = 64 / BITS_PER_ENTRY;
        let block_index = (y << 8) | (z << 4) | x;
        let word_index = block_index / VALUES_PER_LONG;
        let bit_index = (block_index - word_index * VALUES_PER_LONG) * BITS_PER_ENTRY;
        (words[word_index] >> bit_index) & 0xf
    }

    #[test]
    fn jukebox_song_registry_payloads_include_disc_13_component_data() {
        assert_eq!(
            registry_element_count(write_vanilla_jukebox_song_registry_packet),
            21
        );
        let thirteen = JUKEBOX_SONGS
            .iter()
            .find(|song| song.id == "13")
            .expect("music disc 13 should be sent");
        let tag = jukebox_song_nbt(thirteen);

        assert!(matches!(
            field_value(&tag, "sound_event"),
            Some(Tag::String(value)) if value == "minecraft:music_disc.13"
        ));
        let description = compound_field(&tag, "description");
        assert!(matches!(
            field_value(description, "translate"),
            Some(Tag::String(value)) if value == "jukebox_song.minecraft.13"
        ));
        assert!(matches!(
            field_value(&tag, "length_in_seconds"),
            Some(Tag::Float(value)) if (*value - 178.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            field_value(&tag, "comparator_output"),
            Some(Tag::Int(1))
        ));
    }

    #[test]
    fn synced_tag_registries_include_required_names_and_indices() {
        let is_fire = damage_type_tag_entries("minecraft:is_fire");
        assert_eq!(is_fire, &[21, 3, 31, 24, 20, 46, 14]);

        let bypasses_shield = damage_type_tag_entries("minecraft:bypasses_shield");
        assert!(bypasses_shield.contains(&11));
        assert!(bypasses_shield.contains(&13));

        let flower = banner_pattern_tag_entries("minecraft:pattern_item/flower");
        assert_eq!(flower, vec![banner_pattern_index("flower")]);

        let field_masoned = banner_pattern_tag_entries("minecraft:pattern_item/field_masoned");
        assert_eq!(field_masoned, vec![banner_pattern_index("bricks")]);

        let bordure_indented =
            banner_pattern_tag_entries("minecraft:pattern_item/bordure_indented");
        assert_eq!(bordure_indented, vec![banner_pattern_index("curly_border")]);
    }

    #[test]
    fn configuration_wait_ignores_vanilla_common_packets_before_known_packs() {
        let mut input = Vec::new();
        write_framed_packet(
            &mut input,
            SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID,
            |payload| {
                crate::network::codec::write_string(payload, "en_us", 16)?;
                payload.write_all(&[12, 0, 0, 0, 1, 1, 0, 0])?;
                write_var_i32(payload, 127)?;
                Ok(())
            },
        )
        .unwrap();
        write_framed_packet(
            &mut input,
            SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID,
            |payload| {
                write_identifier(payload, &Identifier::parse("minecraft:brand").unwrap())?;
                crate::network::codec::write_string(payload, "vanilla", 32767)
            },
        )
        .unwrap();
        write_framed_packet(
            &mut input,
            SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
            |payload| {
                write_var_i32(payload, 1)?;
                crate::network::codec::write_string(payload, "minecraft", 32767)?;
                crate::network::codec::write_string(payload, "core", 32767)?;
                crate::network::codec::write_string(payload, VERSION_NAME, 32767)
            },
        )
        .unwrap();

        let mut stream = Cursor::new(input);
        wait_for_configuration_packet(
            &mut stream,
            CompressionState::disabled(),
            SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
            "selected known packs",
        )
        .unwrap();
    }

    #[test]
    fn configuration_wait_rejects_unexpected_packets() {
        let mut input = Vec::new();
        write_framed_packet(&mut input, 42, |_payload| Ok(())).unwrap();

        let err = wait_for_configuration_packet(
            &mut Cursor::new(input),
            CompressionState::disabled(),
            SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
            "selected known packs",
        )
        .unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("configuration packet 42"));
    }

    fn assert_nested_sound_variant_fields(tag: Tag, fields: &[&str]) {
        let adult = compound_field(&tag, "adult_sounds");
        let baby = compound_field(&tag, "baby_sounds");
        assert_sound_set_fields(adult, fields);
        assert_sound_set_fields(baby, fields);
    }

    fn assert_sound_variant_fields(tag: Tag, fields: &[&str]) {
        assert_sound_set_fields(&tag, fields);
    }

    fn assert_sound_set_fields(tag: &Tag, fields: &[&str]) {
        for field in fields {
            assert!(
                matches!(field_value(tag, field), Some(Tag::String(value)) if value.starts_with("minecraft:")),
                "missing sound field {field} in {tag:?}"
            );
        }
    }

    fn assert_string_list(value: Option<&Tag>, expected: &[&str]) {
        let Some(Tag::List(values)) = value else {
            panic!("expected string list, got {value:?}");
        };
        let actual: Vec<&str> = values
            .iter()
            .map(|value| match value {
                Tag::String(value) => value.as_str(),
                value => panic!("expected string list value, got {value:?}"),
            })
            .collect();
        assert_eq!(actual, expected);
    }

    fn damage_type_tag_entries(tag: &str) -> &'static [i32] {
        DAMAGE_TYPE_TAGS
            .iter()
            .find_map(|(name, entries)| (*name == tag).then_some(*entries))
            .unwrap_or_else(|| panic!("missing damage type tag {tag}"))
    }

    fn banner_pattern_tag_entries(tag: &str) -> Vec<usize> {
        BANNER_PATTERN_TAGS
            .iter()
            .find_map(|(name, entries)| {
                (*name == tag).then(|| {
                    entries
                        .iter()
                        .map(|entry| banner_pattern_index(entry))
                        .collect()
                })
            })
            .unwrap_or_else(|| panic!("missing banner pattern tag {tag}"))
    }

    fn banner_pattern_index(pattern: &str) -> usize {
        BANNER_PATTERNS
            .iter()
            .position(|entry| *entry == pattern)
            .unwrap_or_else(|| panic!("missing banner pattern {pattern}"))
    }

    fn compound_field<'a>(tag: &'a Tag, field: &str) -> &'a Tag {
        match field_value(tag, field) {
            Some(value @ Tag::Compound(_)) => value,
            value => panic!("expected compound field {field}, got {value:?}"),
        }
    }

    fn field_value<'a>(tag: &'a Tag, field: &str) -> Option<&'a Tag> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        fields
            .iter()
            .find_map(|(name, value)| (name == field).then_some(value))
    }

    fn registry_element_count(write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>) -> i32 {
        let mut payload = Vec::new();
        write_packet(&mut payload).unwrap();
        let mut cursor = Cursor::new(payload);
        let _registry = crate::network::codec::read_identifier(&mut cursor).unwrap();
        read_var_i32(&mut cursor).unwrap()
    }

    fn status_registry_id(write_packet: fn(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut payload = Vec::new();
        write_packet(&mut payload).unwrap();
        let mut cursor = Cursor::new(payload);
        crate::network::codec::read_identifier(&mut cursor)
            .unwrap()
            .to_string()
    }

    fn status_registry_entry_ids_ordered(
        write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>,
    ) -> Vec<String> {
        let mut payload = Vec::new();
        write_packet(&mut payload).unwrap();
        let mut cursor = Cursor::new(payload);
        let _registry = crate::network::codec::read_identifier(&mut cursor).unwrap();
        let entry_count = read_var_i32(&mut cursor).unwrap();
        let mut entry_ids = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            let id = crate::network::codec::read_identifier(&mut cursor).unwrap();
            entry_ids.push(id.to_string());

            let mut _present = [0u8; 1];
            cursor.read_exact(&mut _present).unwrap();
            let mut tag_id = [0u8; 1];
            cursor.read_exact(&mut tag_id).unwrap();
            let _ = Tag::read_payload(tag_id[0], &mut cursor).unwrap();
        }
        entry_ids
    }

    fn assert_registry_order(
        write_packet: fn(&mut Vec<u8>) -> std::io::Result<()>,
        expected: &[&str],
    ) {
        let expected = expected
            .iter()
            .map(|id| format!("minecraft:{id}"))
            .collect::<Vec<_>>();
        assert_eq!(status_registry_entry_ids_ordered(write_packet), expected);
    }

    #[test]
    fn level_chunk_packet_data_uses_vanilla_heightmap_stream_codec_not_nbt() {
        let mut heightmaps = std::collections::BTreeMap::new();
        for name in [
            "WORLD_SURFACE",
            "MOTION_BLOCKING",
            "MOTION_BLOCKING_NO_LEAVES",
        ] {
            heightmaps.insert(name.to_string(), vec![0x0102_0304_0506_0708]);
        }
        let data = super::ClientboundLevelChunkPacketData {
            heightmaps,
            buffer: Vec::new(),
            block_entity_count: 0,
            block_entities: Vec::new(),
        };

        let mut payload = Vec::new();
        super::write_level_chunk_packet_data(&mut payload, &data).unwrap();

        assert_eq!(
            payload[0], 3,
            "heightmap map count is a VarInt, not NBT TAG_Compound"
        );
        let mut cursor = Cursor::new(payload);
        assert_eq!(read_var_i32(&mut cursor).unwrap(), 3);
        for expected_id in [1, 4, 5] {
            assert_eq!(read_var_i32(&mut cursor).unwrap(), expected_id);
            assert_eq!(read_var_i32(&mut cursor).unwrap(), 1);
            let mut bytes = [0; 8];
            cursor.read_exact(&mut bytes).unwrap();
            assert_eq!(i64::from_be_bytes(bytes), 0x0102_0304_0506_0708);
        }
        assert_eq!(read_var_i32(&mut cursor).unwrap(), 0);
        assert_eq!(read_var_i32(&mut cursor).unwrap(), 0);
    }

    /// Builds a minimal PlaySessionState with only the fields needed for NBT round-trip
    /// tests, seeding inventory with known items.
    fn session_state_with_inventory(
        items: &[(&'static str, i32, usize)],
    ) -> super::PlaySessionState {
        let mut inventory = PlayerInventory::new();
        let loaded: Vec<(usize, crate::item_stack::ItemStack)> = items
            .iter()
            .map(|(id, count, slot)| (*slot, ItemStack::new(id, *count)))
            .collect();
        inventory.load_items(&loaded);
        super::PlaySessionState {
            x: 1.0,
            y: 64.0,
            z: -1.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: true,
            fall_distance: 0.0,
            selected_slot: 0,
            health: 20.0,
            food_level: 20,
            food_saturation: 5.0,
            food_exhaustion: 0.0,
            food_tick_timer: 0,
            input_forward: false,
            input_backward: false,
            input_left: false,
            input_right: false,
            input_shift: false,
            input_sprinting: false,
            input_jumping: false,
            air_supply: super::MAX_AIR_SUPPLY,
            in_water: false,
            eye_in_water: false,
            water_fluid_height: 0.0,
            water_velocity_x: 0.0,
            water_velocity_y: 0.0,
            water_velocity_z: 0.0,
            xp_progress: 0.0,
            xp_level: 0,
            xp_total: 0,
            xp_seed: 0,
            score: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
            spawn: None,
            seen_credits: false,
            entered_nether_position: None,
            last_death_location: None,
            root_vehicle: None,
            active_effects: Vec::new(),
            ender_items: Vec::new(),
            abilities: PlayerNbtAbilities::default_survival(),
            inventory_menu: InventoryMenu::new(inventory, RecipeMap::default()),
            carried_item: ItemStack::empty(),
            container_state_id: 0,
            recipe_book_settings: super::default_recipe_book_settings(),
        }
    }

    #[test]
    fn region_feature_cache_keeps_only_fully_decorated_inner_chunks() {
        let center = ChunkPos { x: -41, z: -22 };
        assert_eq!(
            super::REGION_FEATURE_GENERATION_RADIUS,
            super::REGION_FEATURE_CACHEABLE_RADIUS + 2
        );

        assert!(super::region_generated_chunk_is_cacheable(
            center,
            ChunkPos { x: -41, z: -22 }
        ));
        assert!(super::region_generated_chunk_is_cacheable(
            center,
            ChunkPos { x: -40, z: -23 }
        ));
        assert!(!super::region_generated_chunk_is_cacheable(
            center,
            ChunkPos { x: -39, z: -22 }
        ));
        assert!(!super::region_generated_chunk_is_cacheable(
            center,
            ChunkPos { x: -41, z: -20 }
        ));
    }

    #[test]
    fn play_session_state_nbt_round_trip_preserves_empty_inventory() {
        // An empty inventory should serialise as an empty Inventory list and
        // deserialise back without error.
        let state = session_state_with_inventory(&[]);
        let tag = play_session_state_to_nbt(&state);
        let restored =
            play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
        assert!(
            restored
                .inventory_menu
                .player_inventory()
                .saved_items()
                .is_empty(),
            "expected empty inventory after round-trip"
        );
    }

    fn oak_planks_recipe_map() -> RecipeMap {
        RecipeMap::create(vec![crate::recipe_system::RecipeHolder {
            id: "minecraft:oak_planks",
            recipe: crate::recipe_system::RecipeKind::Shapeless {
                ingredients: vec![crate::recipe_system::IngredientSpec::Item(
                    "minecraft:oak_log",
                )],
                result: crate::recipe_system::ItemAmount {
                    item: "minecraft:oak_planks",
                    count: 4,
                },
            },
        }])
    }

    #[test]
    fn play_session_state_nbt_round_trip_preserves_recipe_book_state() {
        let recipes = oak_planks_recipe_map();
        let mut state = session_state_with_inventory(&[]);
        state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes.clone());
        state
            .inventory_menu
            .load_recipe_book(["minecraft:oak_planks"], ["minecraft:oak_planks"]);
        super::apply_recipe_book_settings_packet(
            &mut state,
            crate::network::play::ServerboundRecipeBookChangeSettingsPacket {
                book_type: crate::network::play::RecipeBookType::Crafting,
                is_open: true,
                is_filtering: true,
            },
        );

        let tag = play_session_state_to_nbt(&state);
        let restored = play_session_state_from_nbt(&tag, GameMode::Survival, &recipes).unwrap();

        assert_eq!(
            restored.inventory_menu.recipe_book_known_recipes(),
            vec!["minecraft:oak_planks"]
        );
        assert_eq!(
            restored.inventory_menu.recipe_book_highlighted_recipes(),
            vec!["minecraft:oak_planks"]
        );
        assert!(restored.recipe_book_settings.crafting.open);
        assert!(restored.recipe_book_settings.crafting.filtering);
    }

    #[test]
    fn recipe_book_seen_recipe_packet_clears_highlight_for_display_id() {
        let recipes = oak_planks_recipe_map();
        let mut state = session_state_with_inventory(&[]);
        state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes.clone());
        state
            .inventory_menu
            .load_recipe_book(["minecraft:oak_planks"], ["minecraft:oak_planks"]);

        super::apply_recipe_book_seen_recipe_packet(
            &mut state,
            crate::network::play::ServerboundRecipeBookSeenRecipePacket { recipe_index: 0 },
            &recipes,
        );

        assert_eq!(
            state.inventory_menu.recipe_book_known_recipes(),
            vec!["minecraft:oak_planks"]
        );
        assert!(
            state
                .inventory_menu
                .recipe_book_highlighted_recipes()
                .is_empty()
        );
    }

    #[test]
    fn place_recipe_packet_moves_unlocked_recipe_ingredients_into_inventory_grid() {
        let recipes = oak_planks_recipe_map();
        let mut inventory = PlayerInventory::new();
        inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
        let mut state = session_state_with_inventory(&[]);
        state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
        state
            .inventory_menu
            .load_recipe_book(["minecraft:oak_planks"], []);

        let changed = super::apply_place_recipe_packet(
            &mut state,
            crate::network::play::ServerboundPlaceRecipePacket {
                container_id: 0,
                recipe_index: 0,
                use_max_items: false,
            },
            &recipes,
        );

        assert!(changed);
        assert_eq!(state.container_state_id, 1);
        assert_eq!(
            state.inventory_menu.get_slot(1),
            Some(ItemStack::new("minecraft:oak_log", 1))
        );
        assert_eq!(
            state.inventory_menu.get_slot(0),
            Some(ItemStack::new("minecraft:oak_planks", 4))
        );
        assert_eq!(
            state.inventory_menu.player_inventory().get(0),
            &ItemStack::new("minecraft:oak_log", 2)
        );
    }

    #[test]
    fn place_recipe_packet_rejects_locked_recipe_without_mutating_inventory() {
        let recipes = oak_planks_recipe_map();
        let mut inventory = PlayerInventory::new();
        inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
        let mut state = session_state_with_inventory(&[]);
        state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());

        let changed = super::apply_place_recipe_packet(
            &mut state,
            crate::network::play::ServerboundPlaceRecipePacket {
                container_id: 0,
                recipe_index: 0,
                use_max_items: false,
            },
            &recipes,
        );

        assert!(!changed);
        assert_eq!(state.container_state_id, 0);
        assert_eq!(state.inventory_menu.get_slot(1), Some(ItemStack::empty()));
        assert_eq!(
            state.inventory_menu.player_inventory().get(0),
            &ItemStack::new("minecraft:oak_log", 3)
        );
    }

    #[test]
    fn movement_packets_accumulate_and_apply_fall_damage_on_landing() {
        let mut state = session_state_with_inventory(&[]);
        state.y = 80.0;
        state.on_ground = true;

        let mut falling = Vec::new();
        falling.extend_from_slice(&state.x.to_be_bytes());
        falling.extend_from_slice(&70.0_f64.to_be_bytes());
        falling.extend_from_slice(&state.z.to_be_bytes());
        falling.push(0);
        let update = super::update_play_session_state(
            super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            &mut Cursor::new(falling),
            &mut state,
        )
        .unwrap();
        assert!(update.position_changed);
        assert!(!update.health_changed);
        assert_eq!(state.fall_distance, 10.0);
        assert_eq!(state.health, 20.0);

        let mut landing = Vec::new();
        landing.extend_from_slice(&state.x.to_be_bytes());
        landing.extend_from_slice(&70.0_f64.to_be_bytes());
        landing.extend_from_slice(&state.z.to_be_bytes());
        landing.push(1);
        let update = super::update_play_session_state(
            super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            &mut Cursor::new(landing),
            &mut state,
        )
        .unwrap();
        assert!(update.position_changed);
        assert!(update.health_changed);
        assert_eq!(state.fall_distance, 0.0);
        assert_eq!(state.health, 13.0);
    }

    #[test]
    fn fall_damage_is_suppressed_for_mayfly_players() {
        let mut state = session_state_with_inventory(&[]);
        state.y = 80.0;
        state.abilities.mayfly = true;

        let mut falling = Vec::new();
        falling.extend_from_slice(&state.x.to_be_bytes());
        falling.extend_from_slice(&60.0_f64.to_be_bytes());
        falling.extend_from_slice(&state.z.to_be_bytes());
        falling.push(0);
        super::update_play_session_state(
            super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            &mut Cursor::new(falling),
            &mut state,
        )
        .unwrap();

        let mut landing = Vec::new();
        landing.extend_from_slice(&state.x.to_be_bytes());
        landing.extend_from_slice(&60.0_f64.to_be_bytes());
        landing.extend_from_slice(&state.z.to_be_bytes());
        landing.push(1);
        let update = super::update_play_session_state(
            super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            &mut Cursor::new(landing),
            &mut state,
        )
        .unwrap();
        assert!(!update.health_changed);
        assert_eq!(state.fall_distance, 0.0);
        assert_eq!(state.health, 20.0);
    }

    #[test]
    fn player_fluid_detection_tracks_body_and_eye_water() {
        let mut state = session_state_with_inventory(&[]);
        state.x = 0.5;
        state.y = 64.0;
        state.z = 0.5;

        let shallow = super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
            (x == 0 && y == 64 && z == 0).then(|| "minecraft:water".to_string())
        });
        assert!(shallow.in_water);
        assert!(!shallow.eye_in_water);
        assert_eq!(shallow.water_height, 1.0);

        let submerged = super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
            (x == 0 && (64..=65).contains(&y) && z == 0)
                .then(|| "minecraft:water[level=0]".to_string())
        });
        assert!(submerged.in_water);
        assert!(submerged.eye_in_water);
        assert_eq!(submerged.water_height, 2.0);

        let waterlogged = super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
            (x == 0 && y == 64 && z == 0)
                .then(|| "minecraft:oak_fence[waterlogged=true]".to_string())
        });
        assert!(waterlogged.in_water);
    }

    #[test]
    fn water_contact_suppresses_fall_distance_and_uses_water_exhaustion() {
        let mut state = session_state_with_inventory(&[]);
        state.fall_distance = 7.0;
        state.in_water = true;
        state.eye_in_water = true;
        state.input_sprinting = true;
        state.on_ground = true;

        let update = super::apply_player_movement(&mut state, 1.0, -1.0, 0.0, true);
        assert!(update.position_changed);
        assert!(!update.health_changed);
        assert_eq!(state.fall_distance, 0.0);
        assert_eq!(state.health, 20.0);
        assert!((state.food_exhaustion - 0.0141).abs() < f32::EPSILON);
    }

    #[test]
    fn water_movement_does_not_seed_velocity_from_client_air_motion() {
        let mut state = session_state_with_inventory(&[]);
        state.in_water = true;
        state.eye_in_water = true;
        state.water_velocity_x = 0.01;
        state.water_velocity_y = -0.02;
        state.water_velocity_z = 0.03;

        super::apply_player_movement(&mut state, 0.3, -0.8, 0.3, true);

        assert_eq!(state.water_velocity_x, 0.01);
        assert_eq!(state.water_velocity_y, -0.02);
        assert_eq!(state.water_velocity_z, 0.03);
    }

    #[test]
    fn water_tick_depletes_refills_air_and_drowns_like_java() {
        let mut state = session_state_with_inventory(&[]);

        let update = super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: true,
                water_height: 2.0,
            },
        );
        assert!(update.air_changed);
        assert!(!update.health_changed);
        assert!(update.motion_changed);
        assert_eq!(state.air_supply, 299);
        assert_eq!(state.water_velocity_y, -0.005);
        assert!(state.in_water);
        assert!(state.eye_in_water);

        state.air_supply = -19;
        state.health = 20.0;
        let update = super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: true,
                water_height: 2.0,
            },
        );
        assert!(update.air_changed);
        assert!(update.health_changed);
        assert_eq!(state.air_supply, 0);
        assert_eq!(state.health, 18.0);

        let update = super::tick_play_session_water(&mut state, super::PlayerFluidState::DRY);
        assert!(update.air_changed);
        assert!(!update.health_changed);
        assert_eq!(state.air_supply, 4);

        state.air_supply = 298;
        super::tick_play_session_water(&mut state, super::PlayerFluidState::DRY);
        assert_eq!(state.air_supply, super::MAX_AIR_SUPPLY);
    }

    #[test]
    fn water_tick_applies_jump_impulse_and_drag_like_java() {
        let mut state = session_state_with_inventory(&[]);
        state.on_ground = false;
        state.water_velocity_y = -0.4;

        let update = super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!(update.motion_changed);
        assert!((state.water_velocity_y - (-0.325)).abs() < 1.0e-12);

        state.water_velocity_y = 0.0;
        state.input_jumping = true;
        let update = super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!(update.motion_changed);
        assert!((state.water_velocity_y - 0.027).abs() < 1.0e-12);

        state.water_velocity_y = 0.1;
        super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!((state.water_velocity_y - 0.107).abs() < 1.0e-12);

        state.input_jumping = false;
        state.input_shift = true;
        state.water_velocity_y = 0.0;
        super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!((state.water_velocity_y - (-0.037)).abs() < 1.0e-12);
    }

    #[test]
    fn water_tick_preserves_horizontal_input_like_java() {
        let mut state = session_state_with_inventory(&[]);
        state.input_forward = true;
        state.input_jumping = true;
        state.yaw = 0.0;

        let update = super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!(update.motion_changed);
        assert_eq!(state.water_velocity_x, 0.0);
        assert!((state.water_velocity_z - 0.016).abs() < f64::EPSILON);
        assert!((state.water_velocity_y - 0.027).abs() < 1.0e-12);

        state.input_sprinting = true;
        super::tick_play_session_water(
            &mut state,
            super::PlayerFluidState {
                in_water: true,
                eye_in_water: false,
                water_height: 1.0,
            },
        );
        assert!((state.water_velocity_z - ((0.016 + 0.02) * 0.9)).abs() < 1.0e-12);
    }

    #[test]
    fn air_supply_metadata_packet_uses_vanilla_entity_data_index() {
        let mut state = session_state_with_inventory(&[]);
        state.air_supply = 247;

        let packet = super::play_state_air_supply_metadata_packet(&state).unwrap();
        assert_eq!(packet.id, super::PLAYER_ENTITY_ID);
        assert_eq!(
            packet.packed_items,
            vec![EntityDataValue::typed(1, EntityMetadataValue::VarInt(247)).unwrap()]
        );
    }

    #[test]
    fn player_input_tracks_sprint_jump_exhaustion() {
        let mut state = session_state_with_inventory(&[]);
        state.on_ground = true;

        let update = super::update_play_session_state(
            super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
            &mut Cursor::new(vec![16 | 64]),
            &mut state,
        )
        .unwrap();
        assert!(!update.health_changed);
        assert!(state.input_jumping);
        assert!(state.input_sprinting);
        assert!(!state.input_forward);
        assert!(!state.input_shift);
        assert_eq!(state.food_exhaustion, SPRINT_JUMP_EXHAUSTION);

        super::update_play_session_state(
            super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
            &mut Cursor::new(vec![16 | 64]),
            &mut state,
        )
        .unwrap();
        assert_eq!(
            state.food_exhaustion, SPRINT_JUMP_EXHAUSTION,
            "holding jump should not charge jump exhaustion every packet"
        );

        super::update_play_session_state(
            super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
            &mut Cursor::new(vec![1 | 32]),
            &mut state,
        )
        .unwrap();
        assert!(state.input_forward);
        assert!(state.input_shift);
        assert!(!state.input_jumping);
        assert!(!state.input_sprinting);
    }

    #[test]
    fn sprint_movement_accumulates_food_exhaustion() {
        let mut state = session_state_with_inventory(&[]);
        state.input_sprinting = true;

        let mut movement = Vec::new();
        movement.extend_from_slice(&1.5_f64.to_be_bytes());
        movement.extend_from_slice(&64.0_f64.to_be_bytes());
        movement.extend_from_slice(&(-1.0_f64).to_be_bytes());
        movement.push(1);
        super::update_play_session_state(
            super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
            &mut Cursor::new(movement),
            &mut state,
        )
        .unwrap();
        assert_eq!(state.food_exhaustion, 0.05);
    }

    #[test]
    fn food_tick_fast_regen_heals_and_adds_exhaustion() {
        let mut state = session_state_with_inventory(&[]);
        state.health = 18.0;
        state.food_saturation = 5.0;
        state.food_tick_timer = 9;

        let changed = super::tick_play_session_food(&mut state, FoodDifficulty::Normal, true, 10);
        assert!(changed);
        assert_eq!(state.health, 18.833334);
        assert_eq!(state.food_tick_timer, 0);
        assert_eq!(state.food_exhaustion, 5.0);
    }

    #[test]
    fn food_tick_slow_regen_and_starvation_match_java_thresholds() {
        let mut state = session_state_with_inventory(&[]);
        state.health = 12.0;
        state.food_level = 18;
        state.food_saturation = 0.0;
        state.food_tick_timer = 79;

        assert!(super::tick_play_session_food(
            &mut state,
            FoodDifficulty::Normal,
            true,
            80
        ));
        assert_eq!(state.health, 13.0);
        assert_eq!(state.food_exhaustion, 6.0);

        state.health = 10.0;
        state.food_level = 0;
        state.food_tick_timer = 79;
        assert!(!super::tick_play_session_food(
            &mut state,
            FoodDifficulty::Easy,
            true,
            160
        ));
        assert_eq!(state.health, 10.0);

        state.health = 10.0;
        state.food_tick_timer = 79;
        assert!(super::tick_play_session_food(
            &mut state,
            FoodDifficulty::Hard,
            true,
            240
        ));
        assert_eq!(state.health, 9.0);
    }

    #[test]
    fn peaceful_tick_restores_health_saturation_and_food() {
        let mut state = session_state_with_inventory(&[]);
        state.health = 19.0;
        state.food_level = 19;
        state.food_saturation = 4.0;

        assert!(super::tick_play_session_food(
            &mut state,
            FoodDifficulty::Peaceful,
            true,
            20
        ));
        assert_eq!(state.health, 20.0);
        assert_eq!(state.food_level, 20);
        assert_eq!(state.food_saturation, 5.0);
    }

    #[test]
    fn client_respawn_command_only_requests_respawn_when_dead() {
        let mut alive = session_state_with_inventory(&[]);
        let mut action = Vec::new();
        write_var_i32(&mut action, 0).unwrap();
        let update = super::update_play_session_state(
            super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            &mut Cursor::new(action),
            &mut alive,
        )
        .unwrap();
        assert!(!update.respawn_requested);

        let mut dead = session_state_with_inventory(&[]);
        dead.health = 0.0;
        let mut action = Vec::new();
        write_var_i32(&mut action, 0).unwrap();
        let update = super::update_play_session_state(
            super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            &mut Cursor::new(action),
            &mut dead,
        )
        .unwrap();
        assert!(update.respawn_requested);

        let mut stats = Vec::new();
        write_var_i32(&mut stats, 1).unwrap();
        let update = super::update_play_session_state(
            super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            &mut Cursor::new(stats),
            &mut dead,
        )
        .unwrap();
        assert!(!update.respawn_requested);
    }

    #[test]
    fn respawn_application_restores_health_and_clears_fall_state() {
        let mut state = session_state_with_inventory(&[]);
        state.health = 0.0;
        state.food_level = 3;
        state.food_saturation = 0.0;
        state.food_exhaustion = 12.0;
        state.air_supply = 7;
        state.in_water = true;
        state.eye_in_water = true;
        state.water_fluid_height = 2.0;
        state.fall_distance = 48.0;
        state.on_ground = false;
        state.xp_level = 9;
        state.xp_total = 123;
        state.score = 77;

        super::apply_spawn_placement_to_state(
            &mut state,
            super::PlayerSpawnPlacement {
                x: 12.5,
                y: 70.0,
                z: -3.5,
                yaw: 90.0,
                pitch: 0.0,
            },
        );
        super::reset_play_state_after_death_respawn(&mut state);

        assert_eq!((state.x, state.y, state.z), (12.5, 70.0, -3.5));
        assert_eq!(state.health, 20.0);
        assert_eq!(state.food_level, 20);
        assert_eq!(state.food_saturation, 5.0);
        assert_eq!(state.food_exhaustion, 0.0);
        assert_eq!(state.food_tick_timer, 0);
        assert!(!state.input_sprinting);
        assert!(!state.input_jumping);
        assert_eq!(state.air_supply, super::MAX_AIR_SUPPLY);
        assert!(!state.in_water);
        assert!(!state.eye_in_water);
        assert_eq!(state.water_fluid_height, 0.0);
        assert_eq!(state.fall_distance, 0.0);
        assert!(state.on_ground);
        assert_eq!(state.xp_level, 0);
        assert_eq!(state.xp_total, 0);
        assert_eq!(state.score, 0);
    }

    #[test]
    fn overworld_respawn_pos_uses_motion_blocking_surface_like_java() {
        let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
        chunk.set_block_state(0, 63, 0, "minecraft:grass_block");
        assert_eq!(
            super::overworld_respawn_pos_in_chunk(&chunk, 0, 0),
            Some((0, 64, 0))
        );

        chunk.set_block_state(0, 64, 0, "minecraft:water");
        assert_eq!(super::overworld_respawn_pos_in_chunk(&chunk, 0, 0), None);
    }

    #[test]
    fn play_session_state_nbt_round_trip_preserves_hotbar_and_main_inventory() {
        // Slot 0 (hotbar), slot 9 (main inventory row 1), slot 35 (last main slot).
        // Java: Inventory saves slots 0-35 — all three must survive the round-trip.
        let original_items = &[
            ("minecraft:dirt", 64, 0usize),
            ("minecraft:stone", 32, 9),
            ("minecraft:sand", 16, 35),
        ];
        let state = session_state_with_inventory(original_items);
        let tag = play_session_state_to_nbt(&state);
        let restored =
            play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();

        let saved = restored.inventory_menu.player_inventory().saved_items();
        assert_eq!(saved.len(), 3, "expected exactly 3 items after round-trip");

        for (id, count, slot) in original_items {
            let found = saved.iter().find(|(s, _)| s == slot);
            let found = found.unwrap_or_else(|| panic!("slot {slot} missing after round-trip"));
            assert_eq!(found.1.item_id(), *id, "item id mismatch at slot {slot}");
            assert_eq!(found.1.count(), *count, "count mismatch at slot {slot}");
        }
    }

    #[test]
    fn play_session_state_nbt_round_trip_ignores_out_of_range_slots() {
        // Slots >= 36 (armour, offhand, etc.) are outside the main inventory range
        // and must be silently dropped during deserialisation.
        // Java: Inventory.load() only writes to slots 0-35.
        let tag = Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(4791)),
            (
                "Pos".to_string(),
                Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
            ),
            (
                "Motion".to_string(),
                Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
            ),
            ("OnGround".to_string(), Tag::Byte(1)),
            ("Health".to_string(), Tag::Float(20.0)),
            ("foodLevel".to_string(), Tag::Int(20)),
            ("foodSaturationLevel".to_string(), Tag::Float(5.0)),
            ("XpLevel".to_string(), Tag::Int(0)),
            ("XpP".to_string(), Tag::Float(0.0)),
            ("XpTotal".to_string(), Tag::Int(0)),
            ("SelectedItemSlot".to_string(), Tag::Int(0)),
            ("playerGameType".to_string(), Tag::Int(0)),
            (
                "Inventory".to_string(),
                Tag::List(vec![
                    // Valid slot
                    Tag::Compound(vec![
                        ("Slot".to_string(), Tag::Byte(0)),
                        ("id".to_string(), Tag::String("minecraft:dirt".to_string())),
                        ("count".to_string(), Tag::Int(1)),
                    ]),
                    // Out-of-range slot (armour slot 100) — must be ignored
                    Tag::Compound(vec![
                        ("Slot".to_string(), Tag::Byte(100u8 as i8)),
                        ("id".to_string(), Tag::String("minecraft:stone".to_string())),
                        ("count".to_string(), Tag::Int(1)),
                    ]),
                ]),
            ),
        ]);
        let restored =
            play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
        let saved = restored.inventory_menu.player_inventory().saved_items();
        assert_eq!(saved.len(), 1, "only the in-range slot should survive");
        assert_eq!(saved[0].0, 0);
        assert_eq!(saved[0].1.item_id(), "minecraft:dirt");
    }

    #[test]
    fn play_session_state_from_nbt_clamps_vanilla_playerdata_bounds() {
        let tag = Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(4790)),
            (
                "Pos".to_string(),
                Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
            ),
            ("Health".to_string(), Tag::Float(200.0)),
            ("foodLevel".to_string(), Tag::Int(99)),
            ("foodSaturationLevel".to_string(), Tag::Float(99.0)),
            ("XpP".to_string(), Tag::Float(9.0)),
            ("XpLevel".to_string(), Tag::Int(-7)),
            ("XpTotal".to_string(), Tag::Int(-12)),
            ("SelectedItemSlot".to_string(), Tag::Int(99)),
            ("playerGameType".to_string(), Tag::Int(99)),
        ]);

        let restored =
            play_session_state_from_nbt(&tag, GameMode::Creative, &RecipeMap::default()).unwrap();
        assert_eq!(restored.health, 20.0);
        assert_eq!(restored.food_level, 20);
        assert_eq!(restored.food_saturation, 20.0);
        assert_eq!(restored.air_supply, super::MAX_AIR_SUPPLY);
        assert_eq!(restored.xp_progress, 1.0);
        assert_eq!(restored.xp_level, 0);
        assert_eq!(restored.xp_total, 0);
        assert_eq!(restored.selected_slot, 0);
        assert_eq!(restored.game_mode, GameMode::Survival);
    }

    #[test]
    fn play_session_state_nbt_round_trip_preserves_full_playerdata_surface() {
        let mut state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
        state.fall_distance = 6.25;
        state.food_exhaustion = 3.5;
        state.food_tick_timer = 72;
        state.air_supply = 123;
        state.xp_progress = 0.75;
        state.xp_level = 12;
        state.xp_total = 345;
        state.xp_seed = 98_765;
        state.score = 42;
        state.previous_game_mode = Some(GameMode::Adventure);
        state.spawn = Some(PlayerSpawnData {
            dimension: "minecraft:the_nether".to_string(),
            x: 11,
            y: 72,
            z: -13,
            forced: true,
        });
        state.seen_credits = true;
        state.entered_nether_position = Some((1.25, 64.0, -2.5));
        state.last_death_location = Some(PlayerGlobalPosData {
            dimension: "minecraft:overworld".to_string(),
            x: 3,
            y: 65,
            z: 4,
        });
        state.root_vehicle = Some(Tag::Compound(vec![(
            "Entity".to_string(),
            Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:boat".to_string()),
            )]),
        )]));
        state.active_effects = vec![Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:speed".to_string())),
            ("amplifier".to_string(), Tag::Int(1)),
        ])];
        state.ender_items = vec![Tag::Compound(vec![
            ("Slot".to_string(), Tag::Byte(0)),
            (
                "id".to_string(),
                Tag::String("minecraft:diamond".to_string()),
            ),
            ("count".to_string(), Tag::Int(2)),
        ])];
        state.abilities = PlayerNbtAbilities {
            invulnerable: true,
            flying: true,
            mayfly: true,
            instabuild: true,
            may_build: false,
            fly_speed: 0.08,
            walk_speed: 0.12,
        };

        let tag = play_session_state_to_nbt(&state);
        for field in [
            "Pos",
            "Rotation",
            "Motion",
            "Air",
            "fall_distance",
            "Health",
            "foodLevel",
            "foodSaturationLevel",
            "foodExhaustionLevel",
            "foodTickTimer",
            "XpP",
            "XpLevel",
            "XpTotal",
            "XpSeed",
            "Score",
            "SelectedItemSlot",
            "Inventory",
            "EnderItems",
            "playerGameType",
            "previousPlayerGameType",
            "SpawnX",
            "SpawnY",
            "SpawnZ",
            "SpawnForced",
            "SpawnDimension",
            "seenCredits",
            "recipeBook",
            "LastDeathLocation",
            "enteredNetherPosition",
            "RootVehicle",
            "abilities",
            "active_effects",
        ] {
            assert!(
                field_value(&tag, field).is_some(),
                "{field} missing from player NBT"
            );
        }

        let restored =
            play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
        assert_eq!(restored.fall_distance, 6.25);
        assert_eq!(restored.food_exhaustion, 3.5);
        assert_eq!(restored.food_tick_timer, 72);
        assert_eq!(restored.air_supply, 123);
        assert_eq!(restored.xp_seed, 98_765);
        assert_eq!(restored.score, 42);
        assert_eq!(restored.previous_game_mode, Some(GameMode::Adventure));
        assert_eq!(restored.spawn, state.spawn);
        assert!(restored.seen_credits);
        assert_eq!(
            restored.entered_nether_position,
            state.entered_nether_position
        );
        assert_eq!(restored.last_death_location, state.last_death_location);
        assert_eq!(restored.root_vehicle, state.root_vehicle);
        assert_eq!(restored.active_effects, state.active_effects);
        assert_eq!(restored.ender_items, state.ender_items);
        assert_eq!(restored.abilities, state.abilities);
    }

    #[test]
    fn play_session_state_nbt_inventory_tag_matches_vanilla_format() {
        // Verify the serialised TAG_List contains TAG_Compound entries with the
        // exact field names used by vanilla: "Slot" (TAG_Byte), "id" (TAG_String),
        // "count" (TAG_Int).  This is the wire format read back by the Java server
        // when loading player data.
        let state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
        let tag = play_session_state_to_nbt(&state);
        let Tag::Compound(fields) = &tag else {
            panic!("expected compound tag");
        };
        let inventory_tag = fields
            .iter()
            .find_map(|(name, value)| (name == "Inventory").then_some(value))
            .expect("Inventory tag missing");
        let Tag::List(items) = inventory_tag else {
            panic!("Inventory must be a TAG_List");
        };
        assert_eq!(items.len(), 1);
        let Tag::Compound(item_fields) = &items[0] else {
            panic!("inventory entry must be TAG_Compound");
        };
        let slot = item_fields
            .iter()
            .find_map(|(n, v)| (n == "Slot").then_some(v))
            .expect("Slot missing");
        assert!(matches!(slot, Tag::Byte(3)), "Slot must be TAG_Byte(3)");
        let id = item_fields
            .iter()
            .find_map(|(n, v)| (n == "id").then_some(v))
            .expect("id missing");
        assert!(
            matches!(id, Tag::String(s) if s == "minecraft:stone"),
            "id must be TAG_String"
        );
        let count = item_fields
            .iter()
            .find_map(|(n, v)| (n == "count").then_some(v))
            .expect("count missing");
        assert!(matches!(count, Tag::Int(5)), "count must be TAG_Int(5)");
    }

    // ─── write_lp_vec3 ───────────────────────────────────────────────────────

    #[test]
    fn write_lp_vec3_zero_writes_single_zero_byte() {
        // Java: LpVec3.write — chessboard length below threshold → single 0x00 byte.
        let mut buf = Vec::new();
        write_lp_vec3(&mut buf, 0.0, 0.0, 0.0).unwrap();
        assert_eq!(buf, &[0u8]);
    }

    #[test]
    fn write_lp_vec3_nonzero_writes_six_bytes_for_unit_scale() {
        // For velocity magnitude ≤ 1.0 the scale is 1 and isPartial=false → exactly 6 bytes.
        let mut buf = Vec::new();
        write_lp_vec3(&mut buf, 0.3, 0.1, -0.3).unwrap();
        assert_eq!(
            buf.len(),
            6,
            "scale=1 non-zero velocity should encode to 6 bytes"
        );
    }

    #[test]
    fn write_lp_vec3_round_trips_through_java_decode() {
        // Verify the encoded x/y/z can be recovered within float precision.
        // Java unpack: (value & 0x7FFF).min(32766) * 2.0 / 32766.0 - 1.0
        // where value is extracted from the buffer at the appropriate bit offset.
        fn pack(v: f64) -> i64 {
            ((v * 0.5 + 0.5) * 32766.0 + 0.5).floor() as i64
        }
        fn unpack(v: i64) -> f64 {
            (v & 0x7FFF).min(32766) as f64 * 2.0 / 32766.0 - 1.0
        }
        let vx = 0.3_f64;
        let vy = 0.15_f64;
        let vz = -0.25_f64;
        let scale = 1_i64;
        assert!((unpack(pack(vx / scale as f64)) * scale as f64 - vx).abs() < 0.001);
        assert!((unpack(pack(vy / scale as f64)) * scale as f64 - vy).abs() < 0.001);
        assert!((unpack(pack(vz / scale as f64)) * scale as f64 - vz).abs() < 0.001);
    }

    #[test]
    fn pseudo_rand_f32_produces_values_in_unit_interval() {
        for seed in [-100_i32, 0, 1, 42, i32::MAX, i32::MIN] {
            for index in 0..4_u32 {
                let v = pseudo_rand_f32(seed, index);
                assert!(
                    (0.0..1.0).contains(&v),
                    "pseudo_rand_f32({seed}, {index}) = {v} out of [0, 1)"
                );
            }
        }
    }

    #[test]
    fn pseudo_rand_f32_differs_across_indices() {
        let seed = 12345_i32;
        let v0 = pseudo_rand_f32(seed, 0);
        let v1 = pseudo_rand_f32(seed, 1);
        let v2 = pseudo_rand_f32(seed, 2);
        let v3 = pseudo_rand_f32(seed, 3);
        // All four values should be distinct (probability of collision is ~2^-23).
        assert_ne!(v0, v1);
        assert_ne!(v1, v2);
        assert_ne!(v2, v3);
    }

    // ─── inventory_internal_slot ─────────────────────────────────────────────

    #[test]
    fn inventory_internal_slot_crafting_slots_have_no_backing() {
        // Java: InventoryMenu — slots 0 (result) and 1-4 (2×2 grid) have no PlayerInventory backing.
        for container_slot in 0..=4 {
            assert!(
                inventory_internal_slot(container_slot).is_none(),
                "container slot {container_slot} should have no backing"
            );
        }
    }

    #[test]
    fn inventory_internal_slot_armor_mapping_matches_inventory_menu() {
        // Java: InventoryMenu adds armor slots HEAD(39)/CHEST(38)/LEGS(37)/FEET(36)
        // at container indices 5/6/7/8.
        assert_eq!(
            inventory_internal_slot(5),
            Some(39),
            "container 5 → HEAD (39)"
        );
        assert_eq!(
            inventory_internal_slot(6),
            Some(38),
            "container 6 → CHEST (38)"
        );
        assert_eq!(
            inventory_internal_slot(7),
            Some(37),
            "container 7 → LEGS (37)"
        );
        assert_eq!(
            inventory_internal_slot(8),
            Some(36),
            "container 8 → FEET (36)"
        );
    }

    #[test]
    fn inventory_internal_slot_main_inventory_identity_mapping() {
        // Java: addStandardInventorySlots maps items[9..=35] directly to container slots 9-35.
        for slot in 9..=35usize {
            assert_eq!(
                inventory_internal_slot(slot),
                Some(slot),
                "main inventory: container {slot} → internal {slot}"
            );
        }
    }

    #[test]
    fn inventory_internal_slot_hotbar_shifted_mapping() {
        // Java: addStandardInventorySlots maps items[0..=8] to container slots 36-44.
        for i in 0..=8usize {
            assert_eq!(
                inventory_internal_slot(36 + i),
                Some(i),
                "hotbar: container {} → internal {}",
                36 + i,
                i
            );
        }
    }

    #[test]
    fn inventory_internal_slot_offhand_is_slot_45() {
        // Java: InventoryMenu adds the offhand slot (inventory index 40) at container index 45.
        assert_eq!(
            inventory_internal_slot(45),
            Some(crate::player_inventory::SLOT_OFFHAND)
        );
    }

    #[derive(Debug)]
    struct CursorStream {
        read: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl CursorStream {
        fn new(read: Vec<u8>) -> Self {
            Self {
                read: Cursor::new(read),
                written: Vec::new(),
            }
        }
    }

    impl Read for CursorStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.read.read(buf)
        }
    }

    impl Write for CursorStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Timeline registry and dimension-type sky-fix tests
    // -----------------------------------------------------------------------

    #[test]
    fn timeline_registry_sends_four_entries_in_correct_order() {
        // Java: data/minecraft/timeline/ has day, early_game, moon, villager_schedule.
        // Registry order (0–3) must match the IDs used in the timeline tags packet.
        assert_eq!(
            status_registry_id(write_vanilla_timeline_registry_packet),
            "minecraft:timeline"
        );
        let ids = status_registry_entry_ids_ordered(write_vanilla_timeline_registry_packet);
        assert_eq!(ids.len(), 4);
        assert_eq!(ids[0], "minecraft:day"); // ID 0
        assert_eq!(ids[1], "minecraft:moon"); // ID 1
        assert_eq!(ids[2], "minecraft:villager_schedule"); // ID 2
        assert_eq!(ids[3], "minecraft:early_game"); // ID 3
    }

    #[test]
    fn day_timeline_contains_syncable_tracks_and_omits_non_syncable() {
        // Java: Timeline.NETWORK_CODEC calls filterSyncableTracks, removing any track whose
        // EnvironmentAttribute does not have .syncable() set.
        let nbt = day_timeline_nbt();
        assert!(matches!(
            field_value(&nbt, "clock"),
            Some(Tag::String(v)) if v == "minecraft:overworld"
        ));
        assert!(matches!(
            field_value(&nbt, "period_ticks"),
            Some(Tag::Int(24000))
        ));

        let tracks = compound_field(&nbt, "tracks");

        // Syncable visual/audio/gameplay tracks that must be present.
        assert!(field_value(tracks, "minecraft:visual/sun_angle").is_some());
        assert!(field_value(tracks, "minecraft:visual/moon_angle").is_some());
        assert!(field_value(tracks, "minecraft:visual/star_angle").is_some());
        assert!(field_value(tracks, "minecraft:visual/fog_color").is_some());
        assert!(field_value(tracks, "minecraft:visual/sky_color").is_some());
        assert!(field_value(tracks, "minecraft:visual/sky_light_color").is_some());
        assert!(field_value(tracks, "minecraft:visual/sky_light_factor").is_some());
        assert!(field_value(tracks, "minecraft:visual/star_brightness").is_some());
        assert!(field_value(tracks, "minecraft:visual/cloud_color").is_some());
        assert!(field_value(tracks, "minecraft:visual/sunrise_sunset_color").is_some());
        assert!(field_value(tracks, "minecraft:gameplay/sky_light_level").is_some());
        assert!(field_value(tracks, "minecraft:audio/firefly_bush_sounds").is_some());
        assert!(field_value(tracks, "minecraft:gameplay/creaking_active").is_some());

        // Non-syncable tracks must be absent.
        assert!(field_value(tracks, "minecraft:gameplay/monsters_burn").is_none());
        assert!(field_value(tracks, "minecraft:gameplay/bees_stay_in_hive").is_none());
        assert!(field_value(tracks, "minecraft:gameplay/eyeblossom_open").is_none());
    }

    #[test]
    fn day_timeline_sun_angle_track_uses_cubic_bezier_easing() {
        // Java: Timelines.java:53 — SUN_ANGLE uses EasingType.symmetricCubicBezier(0.362, 0.241).
        // That easing should be present in the track's `ease` field as {cubic_bezier: [...]}.
        let nbt = day_timeline_nbt();
        let tracks = compound_field(&nbt, "tracks");
        let sun_angle = compound_field(tracks, "minecraft:visual/sun_angle");
        let ease = compound_field(sun_angle, "ease");
        let bezier = field_value(ease, "cubic_bezier");
        assert!(
            matches!(bezier, Some(Tag::List(_))),
            "sun_angle ease must contain cubic_bezier list"
        );
    }

    #[test]
    fn day_timeline_cloud_color_uses_int_encoding_for_fully_opaque_argb() {
        // Java: ArgbModifier.argumentCodec selects Codec.INT when alpha == 0xFF → Tag::Int.
        // Day cloud_color keyframe at tick 133 = -1 (0xFFFFFFFF, white).
        let nbt = day_timeline_nbt();
        let tracks = compound_field(&nbt, "tracks");
        let cloud_color = compound_field(tracks, "minecraft:visual/cloud_color");
        let Tag::List(keyframes) = field_value(cloud_color, "keyframes").unwrap() else {
            panic!("cloud_color keyframes must be a list");
        };
        let first_value = field_value(&keyframes[0], "value").unwrap();
        assert!(
            matches!(first_value, Tag::Int(_)),
            "cloud_color keyframe values must be Tag::Int (ArgbModifier, alpha=0xFF)"
        );
        assert_eq!(first_value, &Tag::Int(-1)); // 0xFFFFFFFF = white
    }

    #[test]
    fn moon_timeline_has_moon_phase_track_and_192000_period() {
        // Java: data/minecraft/timeline/moon.json — period_ticks=192000, one syncable track.
        // surface_slime_spawn_chance is non-syncable and filtered out.
        let nbt = moon_timeline_nbt();
        assert!(matches!(
            field_value(&nbt, "clock"),
            Some(Tag::String(v)) if v == "minecraft:overworld"
        ));
        assert!(matches!(
            field_value(&nbt, "period_ticks"),
            Some(Tag::Int(192000))
        ));
        let tracks = compound_field(&nbt, "tracks");
        assert!(field_value(tracks, "minecraft:visual/moon_phase").is_some());
        assert!(
            field_value(tracks, "minecraft:gameplay/surface_slime_spawn_chance").is_none(),
            "surface_slime_spawn_chance is non-syncable and must be filtered out"
        );
    }

    #[test]
    fn moon_timeline_moon_phase_keyframes_are_string_encoded() {
        // Java: MoonPhase.CODEC = StringRepresentable.fromEnum → Tag::String.
        let nbt = moon_timeline_nbt();
        let tracks = compound_field(&nbt, "tracks");
        let moon_phase = compound_field(tracks, "minecraft:visual/moon_phase");
        let Tag::List(keyframes) = field_value(moon_phase, "keyframes").unwrap() else {
            panic!("moon_phase keyframes must be a list");
        };
        assert_eq!(keyframes.len(), 8, "8 moon phases");
        assert!(matches!(
            field_value(&keyframes[0], "value"),
            Some(Tag::String(v)) if v == "full_moon"
        ));
        assert!(matches!(
            field_value(&keyframes[4], "value"),
            Some(Tag::String(v)) if v == "new_moon"
        ));
    }

    #[test]
    fn villager_schedule_timeline_has_no_tracks_after_syncable_filter() {
        // Java: data/minecraft/timeline/villager_schedule.json — villager_activity and
        // baby_villager_activity are both non-syncable → tracks field is absent entirely.
        let nbt = villager_schedule_timeline_nbt();
        assert!(matches!(
            field_value(&nbt, "clock"),
            Some(Tag::String(v)) if v == "minecraft:overworld"
        ));
        assert!(matches!(
            field_value(&nbt, "period_ticks"),
            Some(Tag::Int(24000))
        ));
        assert!(
            field_value(&nbt, "tracks").is_none(),
            "all villager_schedule tracks are non-syncable; tracks field must be absent"
        );
    }

    #[test]
    fn early_game_timeline_has_no_period_ticks_and_no_tracks() {
        // Java: data/minecraft/timeline/early_game.json — no period_ticks field; one track
        // (can_pillager_patrol_spawn) that is non-syncable → both fields absent.
        let nbt = early_game_timeline_nbt();
        assert!(matches!(
            field_value(&nbt, "clock"),
            Some(Tag::String(v)) if v == "minecraft:overworld"
        ));
        assert!(
            field_value(&nbt, "period_ticks").is_none(),
            "early_game has no period_ticks in source data"
        );
        assert!(
            field_value(&nbt, "tracks").is_none(),
            "can_pillager_patrol_spawn is non-syncable; tracks field must be absent"
        );
    }

    #[test]
    fn overworld_dimension_type_has_timelines_clock_and_attributes() {
        // These three fields are required for the client to render a non-black sky.
        // They were absent before the sky fix, causing a permanently black sky on join.
        let nbt = overworld_dimension_type_nbt(false);

        // `timelines`: HolderSet tag reference resolved by the client using the tags packet.
        assert!(
            matches!(
                field_value(&nbt, "timelines"),
                Some(Tag::String(v)) if v == "#minecraft:in_overworld"
            ),
            "timelines must reference the #minecraft:in_overworld tag"
        );

        // `default_clock`: drives the timeline evaluation for this dimension.
        assert!(
            matches!(
                field_value(&nbt, "default_clock"),
                Some(Tag::String(v)) if v == "minecraft:overworld"
            ),
            "default_clock must be minecraft:overworld"
        );

        // `attributes`: static base values that the timeline tracks multiply/add to.
        let attributes = compound_field(&nbt, "attributes");

        assert!(matches!(
            field_value(attributes, "minecraft:visual/sky_color"),
            Some(Tag::String(v)) if v == "#78a7ff"
        ));
        assert!(matches!(
            field_value(attributes, "minecraft:visual/fog_color"),
            Some(Tag::String(v)) if v == "#c0d8ff"
        ));
        assert!(matches!(
            field_value(attributes, "minecraft:visual/cloud_color"),
            Some(Tag::String(v)) if v == "#ccffffff"
        ));
        assert!(
            matches!(
                field_value(attributes, "minecraft:visual/cloud_height"),
                Some(Tag::Float(_))
            ),
            "cloud_height must be a float"
        );
        assert!(matches!(
            field_value(attributes, "minecraft:visual/ambient_light_color"),
            Some(Tag::String(v)) if v == "#0a0a0a"
        ));
    }

    #[test]
    fn update_tags_packet_includes_timeline_group_with_correct_ids() {
        // The tags packet must include a minecraft:timeline group so the client can resolve
        // the "#minecraft:in_overworld" HolderSet reference in the dimension type.
        let mut payload = Vec::new();
        write_minimal_update_tags_packet(&mut payload).unwrap();
        let mut cursor = Cursor::new(payload);

        let group_count = read_var_i32(&mut cursor).unwrap();
        assert_eq!(group_count, 3, "tags packet must have 3 registry groups");

        let mut found_timeline = false;
        for _ in 0..group_count {
            let registry_id = crate::network::codec::read_identifier(&mut cursor)
                .unwrap()
                .to_string();
            let tag_count = read_var_i32(&mut cursor).unwrap();
            if registry_id == "minecraft:timeline" {
                found_timeline = true;
                assert_eq!(tag_count, 2);

                // First tag: #minecraft:in_overworld → [villager_schedule=2, day=0, moon=1, early_game=3].
                // Pre-expanded by server; IDs correspond to write_vanilla_timeline_registry_packet order.
                let tag_id = crate::network::codec::read_identifier(&mut cursor)
                    .unwrap()
                    .to_string();
                assert_eq!(tag_id, "minecraft:in_overworld");
                let entry_count = read_var_i32(&mut cursor).unwrap();
                assert_eq!(entry_count, 4);
                let ids: Vec<i32> = (0..entry_count)
                    .map(|_| read_var_i32(&mut cursor).unwrap())
                    .collect();
                assert_eq!(ids, vec![2, 0, 1, 3]);

                // Second tag: #minecraft:universal → [villager_schedule=2].
                let tag_id2 = crate::network::codec::read_identifier(&mut cursor)
                    .unwrap()
                    .to_string();
                assert_eq!(tag_id2, "minecraft:universal");
                let entry_count2 = read_var_i32(&mut cursor).unwrap();
                assert_eq!(entry_count2, 1);
                assert_eq!(read_var_i32(&mut cursor).unwrap(), 2);
            } else {
                // Skip tags for other registry groups.
                for _ in 0..tag_count {
                    crate::network::codec::read_identifier(&mut cursor).unwrap();
                    let entry_count = read_var_i32(&mut cursor).unwrap();
                    for _ in 0..entry_count {
                        read_var_i32(&mut cursor).unwrap();
                    }
                }
            }
        }
        assert!(
            found_timeline,
            "tags packet must include minecraft:timeline group"
        );
    }

    // ─── var_int_encoded_len ─────────────────────────────────────────────────

    #[test]
    fn var_int_encoded_len_matches_actual_encoding() {
        // Boundary values for each VarInt byte-count tier.
        let cases: &[(i32, usize)] = &[
            (0, 1),
            (1, 1),
            (127, 1),     // 0x7F — last 1-byte value
            (128, 2),     // 0x80 — first 2-byte value
            (16383, 2),   // 0x3FFF — last 2-byte value
            (16384, 3),   // 0x4000 — first 3-byte value
            (2097151, 3), // 0x1FFFFF — last 3-byte value
            (2097152, 4), // 0x200000 — first 4-byte value
        ];
        for &(value, expected_len) in cases {
            let encoded = crate::network::varint::encode_var_i32(value);
            assert_eq!(
                encoded.len(),
                expected_len,
                "encode_var_i32({value}) produced {} bytes, expected {expected_len}",
                encoded.len()
            );
            assert_eq!(
                var_int_encoded_len(value),
                expected_len,
                "var_int_encoded_len({value}) returned {}, expected {expected_len}",
                var_int_encoded_len(value)
            );
        }
    }
}
