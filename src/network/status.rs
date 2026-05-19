use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::console::ConsoleInput;
use crate::network::codec::ComponentJson;
use crate::network::codec::{write_bitset, write_identifier, write_optional, write_uuid, Uuid};
use crate::network::common::ClientboundDisconnectPacket;
use crate::network::compression::CompressionState;
use crate::network::login::{
    ClientboundLoginCompressionPacket, ClientboundLoginDisconnectPacket, LoginSession,
    ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
    CLIENTBOUND_LOGIN_COMPRESSION_PACKET_ID, CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID,
    CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID,
};
use crate::network::ping::{ClientboundPongResponsePacket, ServerboundPingRequestPacket};
use crate::network::play::{
    unpack_block_position, ClientboundLevelChunkPacketData, ClientboundLevelChunkWithLightPacket,
    ClientboundLightUpdatePacketData, ClientboundLoginPacket, CommonPlayerSpawnInfo, GameMode,
    CLIENTBOUND_ADD_ENTITY_PACKET_ID, CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
    CLIENTBOUND_BLOCK_UPDATE_PACKET_ID, CLIENTBOUND_CHANGE_DIFFICULTY_PACKET_ID,
    CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID, CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
    CLIENTBOUND_DISCONNECT_PACKET_ID, CLIENTBOUND_GAME_EVENT_PACKET_ID,
    CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID, CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
    CLIENTBOUND_LOGIN_PACKET_ID, CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID,
    CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID, CLIENTBOUND_PLAYER_POSITION_PACKET_ID,
    CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID, CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
    CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID, CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION_PACKET_ID,
    CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, CLIENTBOUND_SET_EXPERIENCE_PACKET_ID,
    CLIENTBOUND_SET_HEALTH_PACKET_ID, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
    CLIENTBOUND_SET_TIME_PACKET_ID, SERVERBOUND_CHAT_ACK_PACKET_ID,
    SERVERBOUND_CHAT_COMMAND_PACKET_ID, SERVERBOUND_CHAT_PACKET_ID,
    SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
    SERVERBOUND_CLIENT_INFORMATION_PACKET_ID, SERVERBOUND_CLIENT_TICK_END_PACKET_ID,
    SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID, SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
    SERVERBOUND_CONTAINER_CLOSE_PACKET_ID, SERVERBOUND_KEEP_ALIVE_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID, SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
    SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID, SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID,
    SERVERBOUND_PLAYER_ACTION_PACKET_ID, SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
    SERVERBOUND_PLAYER_INPUT_PACKET_ID, SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
    SERVERBOUND_SWING_PACKET_ID, SERVERBOUND_USE_ITEM_ON_PACKET_ID, SERVERBOUND_USE_ITEM_PACKET_ID,
};
use crate::network::varint::{read_var_i32, write_var_i32, write_var_i64};
use crate::player_access::{NameAndId, PlayerAccess};
use crate::registry::Identifier;
use crate::server_properties::ServerProperties;
use crate::storage::nbt::Tag;
use crate::storage::region::{ChunkPos, RegionFile};
use crate::storage::world::WorldLayout;
use crate::worldgen::generate_overworld_chunk_for_preset;

const VERSION_NAME: &str = "26.1.2";
const PROTOCOL_VERSION: i32 = 775;
const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;
const CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID: i32 = 3;
const CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID: i32 = 7;
const CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID: i32 = 12;
const CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID: i32 = 13;
const CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID: i32 = 14;
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
    selected_slot: i32,
    health: f32,
    food_level: i32,
    food_saturation: f32,
    xp_progress: f32,
    xp_level: i32,
    xp_total: i32,
    game_mode: GameMode,
    previous_game_mode: Option<GameMode>,
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
            selected_slot: 0,
            health: 20.0,
            food_level: 20,
            food_saturation: 5.0,
            xp_progress: 0.0,
            xp_level: 0,
            xp_total: 0,
            game_mode: GameMode::Survival,
            previous_game_mode: None,
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
    let player_access = Arc::new(Mutex::new(
        PlayerAccess::load_from_dir(Path::new(".")).unwrap_or_else(|err| {
            eprintln!("status access file load error: {err}");
            PlayerAccess::default()
        }),
    ));
    println!("Status listener bound to {address}");

    loop {
        if should_stop(console_input, &player_access) {
            println!("Status listener stopping");
            break;
        }
        match listener.accept() {
            Ok((stream, peer_addr)) => {
                let properties = properties.clone();
                let favicon = favicon.clone();
                let active_logins = active_logins.clone();
                let world_root = Arc::clone(&world_root);
                let player_access = Arc::clone(&player_access);
                let remote_ip = peer_addr.ip().to_string();
                thread::spawn(move || {
                    if let Err(err) = handle_status_connection(
                        stream,
                        &properties,
                        favicon.as_deref(),
                        &active_logins,
                        &player_access,
                        &world_root,
                        world_seed,
                        &remote_ip,
                    ) {
                        eprintln!("status connection error: {err}");
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
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
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
    let _server_address = read_string(&mut input, 255)?;
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
            player_access,
            world_root,
            world_seed,
            remote_ip,
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
    player_access: &Arc<Mutex<PlayerAccess>>,
    world_root: &Path,
    world_seed: i64,
    remote_ip: &str,
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
    if let Some(reason) =
        login_access_disconnect_reason(properties, player_access, &finished.profile, remote_ip)?
    {
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

    let mut play_state = load_play_session_state(world_root, &finished.profile.uuid, properties);
    write_minimal_play_join(
        stream,
        compression,
        properties,
        world_seed,
        &finished.profile,
        &play_state,
        world_root,
    )?;
    let mut current_chunk_x = chunk_coordinate(play_state.x);
    let mut current_chunk_z = chunk_coordinate(play_state.z);
    let chunk_batch_radius = chunk_batch_radius(properties);
    let mut loaded_chunks = chunk_window(current_chunk_x, current_chunk_z, chunk_batch_radius);
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    let mut last_keep_alive = Instant::now();
    let mut keep_alive_id = 0_i64;
    let mut entity_id_counter: i32 = 1; // player has entity ID 1; start here so first drop = 2
    let world_layout = WorldLayout::new(world_root);
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
        match read_packet_with_compression(stream, compression) {
            Ok(packet) => {
                let mut input = Cursor::new(packet);
                let packet_id = read_var_i32(&mut input)?;
                if update_play_session_state(packet_id, &mut input, &mut play_state)? {
                    let next_chunk_x = chunk_coordinate(play_state.x);
                    let next_chunk_z = chunk_coordinate(play_state.z);
                    if next_chunk_x != current_chunk_x || next_chunk_z != current_chunk_z {
                        let next_loaded_chunks =
                            chunk_window(next_chunk_x, next_chunk_z, chunk_batch_radius);
                        for stale_chunk in loaded_chunks.difference(&next_loaded_chunks) {
                            write_forget_level_chunk_packet(
                                stream,
                                compression,
                                stale_chunk.0,
                                stale_chunk.1,
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
                        )?;
                    }
                    continue;
                }
                if packet_id == SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID {
                    write_command_suggestions_response(stream, compression, &mut input)?;
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
                    let should_break =
                        action == 2 || (action == 0 && play_state.game_mode == GameMode::Creative);
                    if should_break {
                        write_framed_packet_with_compression(
                            stream,
                            compression,
                            CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
                            |p| write_var_i32(p, sequence),
                        )?;
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
                        let block_state = get_block_state_at(bx, by, bz);
                        save_broken_block_to_region(&world_layout, chunk_pos, bx, by, bz);
                        if play_state.game_mode != GameMode::Creative {
                            if let Some(item_id) = block_state_to_item_drop(block_state) {
                                entity_id_counter = entity_id_counter.wrapping_add(1);
                                let eid = entity_id_counter;
                                let drop_x = bx as f64 + 0.5;
                                let drop_y = by as f64 + 0.5;
                                let drop_z = bz as f64 + 0.5;
                                write_framed_packet_with_compression(
                                    stream,
                                    compression,
                                    CLIENTBOUND_ADD_ENTITY_PACKET_ID,
                                    |p| {
                                        write_var_i32(p, eid)?;
                                        let uuid_hi =
                                            (eid as u64).wrapping_mul(0x6C62_272E_07BB_0142);
                                        let uuid_lo =
                                            (eid as u64).wrapping_mul(0x62B8_2175_6295_C58D);
                                        p.write_all(&uuid_hi.to_be_bytes())?;
                                        p.write_all(&uuid_lo.to_be_bytes())?;
                                        write_var_i32(p, ITEM_ENTITY_TYPE_ID)?;
                                        p.write_all(&drop_x.to_be_bytes())?;
                                        p.write_all(&drop_y.to_be_bytes())?;
                                        p.write_all(&drop_z.to_be_bytes())?;
                                        p.write_all(&[0u8])?; // movement = Vec3.ZERO (LpVec3: 0x00)
                                        p.write_all(&[0u8, 0u8, 0u8])?; // xRot, yRot, yHeadRot
                                        write_var_i32(p, 0) // data
                                    },
                                )?;
                                write_framed_packet_with_compression(
                                    stream,
                                    compression,
                                    CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
                                    |p| {
                                        write_var_i32(p, eid)?;
                                        p.write_all(&[8u8])?; // metadata index 8 = item stack
                                        write_var_i32(p, 7)?; // serializer id: ItemStack
                                        write_var_i32(p, 1)?; // count = 1
                                        write_var_i32(p, item_id)?;
                                        write_var_i32(p, 0)?; // add_components = 0
                                        write_var_i32(p, 0)?; // remove_components = 0
                                        p.write_all(&[0xFFu8]) // end of metadata
                                    },
                                )?;
                            }
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
                        | SERVERBOUND_CONTAINER_CLICK_PACKET_ID
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
                        | SERVERBOUND_USE_ITEM_ON_PACKET_ID
                        | SERVERBOUND_USE_ITEM_PACKET_ID
                ) {
                    continue;
                }
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
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
                let _ = save_play_session_state(world_root, &finished.profile.uuid, &play_state);
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

fn update_play_session_state<R: Read>(
    packet_id: i32,
    input: &mut R,
    state: &mut PlaySessionState,
) -> io::Result<bool> {
    match packet_id {
        SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID => {
            state.x = read_f64(input)?;
            state.y = read_f64(input)?;
            state.z = read_f64(input)?;
            state.on_ground = read_bool(input)?;
            Ok(true)
        }
        SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID => {
            state.x = read_f64(input)?;
            state.y = read_f64(input)?;
            state.z = read_f64(input)?;
            state.yaw = read_f32(input)?;
            state.pitch = read_f32(input)?;
            state.on_ground = read_bool(input)?;
            Ok(true)
        }
        SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID => {
            state.yaw = read_f32(input)?;
            state.pitch = read_f32(input)?;
            state.on_ground = read_bool(input)?;
            Ok(true)
        }
        SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID => {
            state.on_ground = read_bool(input)?;
            Ok(true)
        }
        SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID => {
            let slot = i32::from(read_i16(input)?);
            if (0..9).contains(&slot) {
                state.selected_slot = slot;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn load_play_session_state(
    world_root: &Path,
    uuid: &str,
    properties: &ServerProperties,
) -> PlaySessionState {
    let layout = WorldLayout::new(world_root);
    let default_game_mode = game_mode_from_name(&properties.game_mode);
    let mut state = layout
        .load_player_data(uuid)
        .ok()
        .and_then(|tag| play_session_state_from_nbt(&tag, default_game_mode))
        .unwrap_or_else(|| PlaySessionState {
            game_mode: default_game_mode,
            ..PlaySessionState::default()
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
        ("Health".to_string(), Tag::Float(state.health)),
        ("foodLevel".to_string(), Tag::Int(state.food_level)),
        (
            "foodSaturationLevel".to_string(),
            Tag::Float(state.food_saturation),
        ),
        ("XpLevel".to_string(), Tag::Int(state.xp_level)),
        ("XpP".to_string(), Tag::Float(state.xp_progress)),
        ("XpTotal".to_string(), Tag::Int(state.xp_total)),
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
            "recipeBook".to_string(),
            Tag::Compound(vec![
                ("recipes".to_string(), Tag::List(vec![])),
                ("toBeDisplayed".to_string(), Tag::List(vec![])),
            ]),
        ),
    ];
    if let Some(mode) = state.previous_game_mode {
        values.push((
            "previousPlayerGameType".to_string(),
            Tag::Int(game_mode_legacy_id(mode)),
        ));
    }
    Tag::Compound(values)
}

fn play_session_state_from_nbt(tag: &Tag, default_game_mode: GameMode) -> Option<PlaySessionState> {
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
    let game_mode = match compound_tag(compound, "playerGameType") {
        Some(Tag::Int(value)) => game_mode_from_legacy_id(*value),
        _ => default_game_mode,
    };
    let previous_game_mode = match compound_tag(compound, "previousPlayerGameType") {
        Some(Tag::Int(value)) if *value == -1 => None,
        Some(Tag::Int(value)) => Some(game_mode_from_legacy_id(*value)),
        _ => None,
    };
    Some(PlaySessionState {
        x: *x,
        y: *y,
        z: *z,
        yaw: *yaw,
        pitch: *pitch,
        on_ground,
        selected_slot,
        health,
        food_level,
        food_saturation,
        xp_progress,
        xp_level,
        xp_total,
        game_mode,
        previous_game_mode,
    })
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

fn login_access_disconnect_reason(
    properties: &ServerProperties,
    player_access: &Arc<Mutex<PlayerAccess>>,
    profile: &NameAndId,
    remote_ip: &str,
) -> io::Result<Option<&'static str>> {
    let access = player_access
        .lock()
        .map_err(|_| io::Error::other("player access lock poisoned"))?;
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
    world_root: &Path,
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
        CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        |payload| {
            payload.write_all(&[0])?;
            write_var_i32(payload, 0)?;
            write_var_i32(payload, 46)?;
            for _ in 0..46 {
                write_var_i32(payload, 0)?;
            }
            write_var_i32(payload, 0)
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
        |payload| write_var_i32(payload, 0),
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_SET_TIME_PACKET_ID,
        |payload| {
            payload.write_all(&0_i64.to_be_bytes())?;
            write_var_i32(payload, 0)
        },
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
        |payload| write_default_spawn_position_packet(payload, 0, SPAWN_Y as i32, 0),
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
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[2])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[7])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[8])?;
            payload.write_all(&0.0f32.to_be_bytes())
        },
    )?;
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
        .collect();
    write_play_chunk_delta(
        stream,
        compression,
        center_chunk_x,
        center_chunk_z,
        &chunks,
        false,
        world_root,
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
    if chunks.is_empty() {
        return Ok(());
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_payload| Ok(()),
    )?;
    for &(x, z) in chunks {
        write_framed_packet_with_compression(
            stream,
            compression,
            CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID,
            |payload| write_generated_spawn_chunk_packet(payload, x, z, world_root),
        )?;
    }
    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, chunks.len() as i32),
    )
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
    stream: &mut TcpStream,
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
) -> io::Result<()> {
    let pos = ChunkPos { x, z };
    let region_dir = world_root.join("region");
    let chunk = try_load_chunk_from_region(&region_dir, pos)
        .unwrap_or_else(|| {
            generate_overworld_chunk_for_preset(pos, "normal")
                .unwrap_or_else(|_| crate::storage::chunk::LevelChunk::empty(pos))
        });
    let light_data = ClientboundLightUpdatePacketData::from_chunk_sections(&chunk.sections);
    let packet = ClientboundLevelChunkWithLightPacket::from_chunk(&chunk, light_data);
    write_level_chunk_with_light_payload(writer, &packet)
}

fn try_load_chunk_from_region(
    region_dir: &Path,
    pos: ChunkPos,
) -> Option<crate::storage::chunk::LevelChunk> {
    let region = RegionFile::open(region_dir, pos.region()).ok()?;
    let (_name, tag) = region.read_chunk_nbt(pos).ok()??;
    crate::storage::chunk::LevelChunk::from_nbt(pos, &tag).ok()
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
    write_level_chunk_heightmaps(writer, &data.heightmaps)?;
    write_var_i32(writer, data.buffer.len() as i32)?;
    writer.write_all(&data.buffer)?;
    write_var_i32(writer, data.block_entity_count as i32)
}

fn write_level_chunk_heightmaps<W: Write>(
    writer: &mut W,
    heightmaps: &std::collections::BTreeMap<String, Vec<i64>>,
) -> io::Result<()> {
    let entries = clientbound_heightmap_entries(heightmaps);
    write_var_i32(writer, entries.len() as i32)?;
    for (type_id, values) in entries {
        write_var_i32(writer, type_id)?;
        write_var_i32(writer, values.len() as i32)?;
        for value in values {
            writer.write_all(&value.to_be_bytes())?;
        }
    }
    Ok(())
}

fn clientbound_heightmap_entries(
    heightmaps: &std::collections::BTreeMap<String, Vec<i64>>,
) -> Vec<(i32, Vec<i64>)> {
    let mut entries = Vec::new();
    if let Some(values) = heightmaps
        .get("WORLD_SURFACE")
        .or_else(|| heightmaps.get("WORLD_SURFACE_WG"))
    {
        entries.push((1, values.clone()));
    }
    if let Some(values) = heightmaps
        .get("MOTION_BLOCKING")
        .or_else(|| heightmaps.get("WORLD_SURFACE"))
        .or_else(|| heightmaps.get("WORLD_SURFACE_WG"))
    {
        entries.push((4, values.clone()));
    }
    if let Some(values) = heightmaps
        .get("MOTION_BLOCKING_NO_LEAVES")
        .or_else(|| heightmaps.get("MOTION_BLOCKING"))
        .or_else(|| heightmaps.get("WORLD_SURFACE"))
        .or_else(|| heightmaps.get("WORLD_SURFACE_WG"))
    {
        entries.push((5, values.clone()));
    }
    entries
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

fn block_state_to_item_drop(block_state_id: i32) -> Option<i32> {
    match block_state_id {
        STONE_BLOCK_STATE_ID => Some(35),      // stone → cobblestone
        GRANITE_BLOCK_STATE_ID => Some(2),     // granite → granite
        DIORITE_BLOCK_STATE_ID => Some(4),     // diorite → diorite
        ANDESITE_BLOCK_STATE_ID => Some(6),    // andesite → andesite
        GRASS_BLOCK_STATE_ID => Some(28),      // grass block → dirt
        DIRT_BLOCK_STATE_ID => Some(28),       // dirt → dirt
        DANDELION_BLOCK_STATE_ID => Some(229), // dandelion → dandelion
        POPPY_BLOCK_STATE_ID => Some(233),     // poppy → poppy
        _ => None,
    }
}


fn save_broken_block_to_region(layout: &WorldLayout, chunk_pos: ChunkPos, bx: i32, by: i32, bz: i32) {
    let region_dir = layout.region_dir();
    let Ok(region) = RegionFile::open(&region_dir, chunk_pos.region()) else {
        return;
    };
    let mut chunk = match region.read_chunk_nbt(chunk_pos) {
        Ok(Some((_name, tag))) => {
            crate::storage::chunk::LevelChunk::from_nbt(chunk_pos, &tag).unwrap_or_else(|_| {
                generate_overworld_chunk_for_preset(chunk_pos, "normal")
                    .unwrap_or_else(|_| crate::storage::chunk::LevelChunk::empty(chunk_pos))
            })
        }
        _ => generate_overworld_chunk_for_preset(chunk_pos, "normal")
            .unwrap_or_else(|_| crate::storage::chunk::LevelChunk::empty(chunk_pos)),
    };
    chunk.set_block_state(bx, by, bz, "minecraft:air");
    let nbt = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let _ = region.write_chunk_nbt(chunk_pos, "", &nbt);
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
    write_var_i32(writer, 2)?;
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

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0] != 0)
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
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
        banner_pattern_nbt, cat_sound_variant_nbt, chat_type_nbt, chicken_sound_variant_nbt,
        chunk_batch_size, chunk_window, cow_sound_variant_nbt, encode_base64, escape_json_string,
        handle_legacy_status_connection, instrument_nbt, jukebox_song_nbt,
        legacy_disconnect_packet, legacy_version0_response, legacy_version1_response, load_favicon,
        newly_visible_chunks, pig_sound_variant_nbt, read_packet, status_json, trim_material_nbt,
        trim_pattern_nbt, vanilla_baseline_biome_nbt, visible_spawn_surface_feature_id,
        visible_spawn_surface_top_block_id, visible_spawn_terrain_block_count,
        visible_spawn_terrain_height, wait_for_configuration_packet, wolf_sound_variant_nbt,
        write_framed_packet, write_legacy_string, write_minimal_biome_registry_packet,
        write_minimal_damage_type_registry_packet, write_minimal_dimension_type_registry_packet,
        write_minimal_trim_material_registry_packet, write_status_pong_packet,
        write_vanilla_banner_pattern_registry_packet,
        write_vanilla_cat_sound_variant_registry_packet, write_vanilla_cat_variant_registry_packet,
        write_vanilla_chat_type_registry_packet,
        write_vanilla_chicken_sound_variant_registry_packet,
        write_vanilla_chicken_variant_registry_packet,
        write_vanilla_cow_sound_variant_registry_packet, write_vanilla_cow_variant_registry_packet,
        write_vanilla_frog_variant_registry_packet, write_vanilla_instrument_registry_packet,
        write_vanilla_jukebox_song_registry_packet, write_vanilla_painting_variant_registry_packet,
        write_vanilla_pig_sound_variant_registry_packet, write_vanilla_pig_variant_registry_packet,
        write_vanilla_trim_pattern_registry_packet,
        write_vanilla_wolf_sound_variant_registry_packet,
        write_vanilla_wolf_variant_registry_packet,
        write_vanilla_zombie_nautilus_variant_registry_packet,
        write_visible_spawn_terrain_block_state_container, CompressionState,
        ANDESITE_BLOCK_STATE_ID, BANNER_PATTERNS, BANNER_PATTERN_TAGS, BEDROCK_BLOCK_STATE_ID,
        BIOMES, CHAT_TYPES, DAMAGE_TYPES, DAMAGE_TYPE_TAGS, DANDELION_BLOCK_STATE_ID,
        DIORITE_BLOCK_STATE_ID, DIRT_BLOCK_STATE_ID, GRANITE_BLOCK_STATE_ID, GRASS_BLOCK_STATE_ID,
        INSTRUMENTS, JUKEBOX_SONGS, POPPY_BLOCK_STATE_ID,
        SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID,
        SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID, SHORT_GRASS_BLOCK_STATE_ID,
        STONE_BLOCK_STATE_ID, TRIM_MATERIALS, TRIM_PATTERNS, VERSION_NAME,
    };
    use crate::network::codec::write_identifier;
    use crate::network::ping::ServerboundPingRequestPacket;
    use crate::network::varint::{read_var_i32, write_var_i32};
    use crate::registry::Identifier;
    use crate::server_properties::ServerProperties;
    use crate::storage::nbt::Tag;
    use crate::{biome, damage_type, equipment_trim, presentation_data};
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::{self, Cursor, Read, Write};
    use std::path::{Path, PathBuf};

    struct SynchronizedRegistryManifestEntry {
        registry_id: &'static str,
        expected_entry_count: usize,
        java_network_shape: &'static str,
        write_packet: fn(&mut Vec<u8>) -> io::Result<()>,
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
        let properties = test_properties();
        let json = status_json(&properties, None);
        assert!(json.contains("\"name\":\"26.1.2\""));
        assert!(json.contains("\"protocol\":775"));
        assert!(json.contains("\"max\":20"));
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
    fn spawn_chunk_window_uses_configured_server_view_distance_radius() {
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

    #[test]
    fn level_chunk_packet_data_uses_vanilla_heightmap_stream_codec_not_nbt() {
        let mut heightmaps = std::collections::BTreeMap::new();
        heightmaps.insert("WORLD_SURFACE_WG".to_string(), vec![0x0102_0304_0506_0708]);
        let data = super::ClientboundLevelChunkPacketData {
            heightmaps,
            buffer: Vec::new(),
            block_entity_count: 0,
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
}
