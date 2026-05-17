use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::network::codec::{write_identifier, write_optional};
use crate::network::login::{
    LoginSession, ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
    CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, SERVERBOUND_HELLO_PACKET_ID,
    SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID,
};
use crate::network::ping::{ClientboundPongResponsePacket, ServerboundPingRequestPacket};
use crate::network::play::{
    ClientboundLoginPacket, CommonPlayerSpawnInfo, GameMode, CLIENTBOUND_LOGIN_PACKET_ID,
    CLIENTBOUND_PLAYER_POSITION_PACKET_ID, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID,
};
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;
use crate::server_properties::ServerProperties;
use crate::storage::nbt::Tag;

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
const CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID: i32 = 11;
const CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID: i32 = 12;
const CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID: i32 = 45;
const EMPTY_SPAWN_CHUNK_SECTION_COUNT: usize = 24;
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

const BIOMES: &[&str] = &[
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

const TRIM_PATTERNS: &[&str] = &[
    "bolt",
    "coast",
    "dune",
    "eye",
    "flow",
    "host",
    "raiser",
    "rib",
    "sentry",
    "shaper",
    "silence",
    "snout",
    "spire",
    "tide",
    "vex",
    "ward",
    "wayfinder",
    "wild",
];

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
        id: "amethyst",
        asset_name: "amethyst",
        color: "#9a5cc6",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "copper",
        asset_name: "copper",
        color: "#b4684d",
        overrides: &[("minecraft:copper", "copper_darker")],
    },
    TrimMaterialEntry {
        id: "diamond",
        asset_name: "diamond",
        color: "#6eead6",
        overrides: &[("minecraft:diamond", "diamond_darker")],
    },
    TrimMaterialEntry {
        id: "emerald",
        asset_name: "emerald",
        color: "#11a036",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "gold",
        asset_name: "gold",
        color: "#decf2a",
        overrides: &[("minecraft:gold", "gold_darker")],
    },
    TrimMaterialEntry {
        id: "iron",
        asset_name: "iron",
        color: "#ececec",
        overrides: &[("minecraft:iron", "iron_darker")],
    },
    TrimMaterialEntry {
        id: "lapis",
        asset_name: "lapis",
        color: "#416e97",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "netherite",
        asset_name: "netherite",
        color: "#625859",
        overrides: &[("minecraft:netherite", "netherite_darker")],
    },
    TrimMaterialEntry {
        id: "quartz",
        asset_name: "quartz",
        color: "#e3d4bd",
        overrides: &[],
    },
    TrimMaterialEntry {
        id: "redstone",
        asset_name: "redstone",
        color: "#971607",
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
) -> Result<(), String> {
    let address = format!("{bind_ip}:{port}");
    let listener = TcpListener::bind(&address)
        .map_err(|err| format!("Failed to bind status listener on {address}: {err}"))?;
    let favicon = load_favicon(Path::new("server-icon.png"))
        .map_err(|err| format!("Failed to load server-icon.png: {err}"))?;
    println!("Status listener bound to {address}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let properties = properties.clone();
                let favicon = favicon.clone();
                thread::spawn(move || {
                    if let Err(err) =
                        handle_status_connection(stream, &properties, favicon.as_deref())
                    {
                        eprintln!("status connection error: {err}");
                    }
                });
            }
            Err(err) => eprintln!("status accept error: {err}"),
        }
    }

    Ok(())
}

fn handle_status_connection(
    mut stream: TcpStream,
    properties: &ServerProperties,
    favicon: Option<&str>,
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

    let _protocol = read_var_i32(&mut input)?;
    let _server_address = read_string(&mut input, 255)?;
    let mut port_bytes = [0u8; 2];
    input.read_exact(&mut port_bytes)?;
    let _server_port = u16::from_be_bytes(port_bytes);
    let next_state = read_var_i32(&mut input)?;
    if next_state == 2 {
        return handle_login_connection(&mut stream, properties);
    }
    if next_state != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported handshake target state",
        ));
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

fn handle_login_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
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
    write_framed_packet(stream, CLIENTBOUND_LOGIN_FINISHED_PACKET_ID, |payload| {
        finished.write(payload)
    })?;

    let packet = read_packet(stream)?;
    let mut input = Cursor::new(packet);
    let packet_id = read_var_i32(&mut input)?;
    if packet_id != SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected login acknowledgement",
        ));
    }
    login.acknowledge(ServerboundLoginAcknowledgedPacket::read(&mut input)?);

    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_UPDATE_ENABLED_FEATURES_PACKET_ID,
        |payload| {
            write_var_i32(payload, 1)?;
            write_identifier(payload, &Identifier::parse("minecraft:vanilla").unwrap())
        },
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_biome_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chat_type_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_trim_pattern_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_trim_material_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_wolf_sound_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_pig_sound_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_frog_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cat_sound_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_sound_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_cow_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_sound_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_chicken_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_zombie_nautilus_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_painting_variant_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_dimension_type_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_minimal_damage_type_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_banner_pattern_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_jukebox_song_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_REGISTRY_DATA_PACKET_ID,
        write_vanilla_instrument_registry_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_UPDATE_TAGS_PACKET_ID,
        write_minimal_update_tags_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        write_vanilla_known_packs_packet,
    )?;
    wait_for_configuration_packet(
        stream,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID,
        "selected known packs",
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_CONFIGURATION_FINISH_PACKET_ID,
        |_payload| Ok(()),
    )?;

    wait_for_configuration_packet(
        stream,
        SERVERBOUND_CONFIGURATION_FINISH_PACKET_ID,
        "finish configuration",
    )?;

    write_minimal_play_join(stream, properties)?;
    loop {
        match read_packet(stream) {
            Ok(_packet) => {}
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::UnexpectedEof
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::TimedOut
                ) =>
            {
                return Ok(())
            }
            Err(err) => return Err(err),
        }
    }
}

fn wait_for_configuration_packet<R: Read>(
    reader: &mut R,
    expected_packet_id: i32,
    expected_name: &'static str,
) -> io::Result<()> {
    for _ in 0..32 {
        let packet = read_packet(reader)?;
        let mut input = Cursor::new(packet);
        let packet_id = read_var_i32(&mut input)?;
        if packet_id == expected_packet_id {
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
    properties: &ServerProperties,
) -> io::Result<()> {
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
        spawn_info: CommonPlayerSpawnInfo::default(),
        enforces_secure_chat: false,
    };
    write_framed_packet(stream, CLIENTBOUND_LOGIN_PACKET_ID, |payload| {
        write_clientbound_login_packet(payload, &login)
    })?;
    write_framed_packet(stream, CLIENTBOUND_SET_HELD_SLOT_PACKET_ID, |payload| {
        write_var_i32(payload, 0)
    })?;
    write_framed_packet(stream, CLIENTBOUND_PLAYER_POSITION_PACKET_ID, |payload| {
        write_var_i32(payload, 0)?;
        write_vec3(payload, 0.5, 80.0, 0.5)?;
        write_vec3(payload, 0.0, 0.0, 0.0)?;
        payload.write_all(&0.0f32.to_be_bytes())?;
        payload.write_all(&0.0f32.to_be_bytes())?;
        payload.write_all(&0_i32.to_be_bytes())
    })?;
    write_framed_packet(
        stream,
        CLIENTBOUND_PLAY_CHUNK_BATCH_START_PACKET_ID,
        |_payload| Ok(()),
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_PLAY_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID,
        write_empty_spawn_chunk_packet,
    )?;
    write_framed_packet(
        stream,
        CLIENTBOUND_PLAY_CHUNK_BATCH_FINISHED_PACKET_ID,
        |payload| write_var_i32(payload, 1),
    )
}

fn write_empty_spawn_chunk_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all(&0_i32.to_be_bytes())?;
    writer.write_all(&0_i32.to_be_bytes())?;
    write_var_i32(writer, 0)?;

    let mut section_buffer = Vec::with_capacity(EMPTY_SPAWN_CHUNK_SECTION_COUNT * 10);
    for _ in 0..EMPTY_SPAWN_CHUNK_SECTION_COUNT {
        section_buffer.write_all(&0_i16.to_be_bytes())?;
        section_buffer.write_all(&0_i16.to_be_bytes())?;
        write_single_value_paletted_container(&mut section_buffer, 0)?;
        write_single_value_paletted_container(&mut section_buffer, 0)?;
    }
    write_var_i32(writer, section_buffer.len() as i32)?;
    writer.write_all(&section_buffer)?;
    write_var_i32(writer, 0)?;

    write_empty_bitset(writer)?;
    write_empty_bitset(writer)?;
    write_empty_bitset(writer)?;
    write_empty_bitset(writer)?;
    write_var_i32(writer, 0)?;
    write_var_i32(writer, 0)
}

fn write_single_value_paletted_container<W: Write>(writer: &mut W, id: i32) -> io::Result<()> {
    writer.write_all(&[0])?;
    write_var_i32(writer, id)?;
    write_var_i32(writer, 0)
}

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

fn write_vanilla_zombie_nautilus_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const VARIANTS: &[&str] = &["temperate", "warm"];
    write_variant_registry(writer, "minecraft:zombie_nautilus_variant", VARIANTS, |id| {
        zombie_nautilus_variant_nbt(id)
    })
}

fn write_vanilla_painting_variant_registry_packet<W: Write>(writer: &mut W) -> io::Result<()> {
    const PAINTINGS: &[&str] = &[
        "alban", "aztec", "aztec2", "backyard", "baroque", "bomb", "bouquet",
        "burning_skull", "bust", "cavebird", "changing", "cotan", "courbet", "creebet",
        "dennis", "donkey_kong", "earth", "endboss", "fern", "fighters", "finding",
        "fire", "graham", "humble", "kebab", "lowmist", "match", "meditative", "orb",
        "owlemons", "passage", "pigscene", "plant", "pointer", "pond", "pool",
        "prairie_ride", "sea", "skeleton", "skull_and_roses", "stage", "sunflowers",
        "sunset", "tides", "unpacked", "void", "wanderer", "wasteland", "water",
        "wind", "wither",
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
    write_identifier(writer, &Identifier::parse("minecraft:worldgen/biome").unwrap())?;
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
        | "frozen_ocean" | "deep_frozen_ocean" | "grove" | "snowy_slopes"
        | "frozen_peaks" | "jagged_peaks" => (true, 0.0, 0.5, 4_020_182),
        "desert" | "savanna" | "savanna_plateau" | "windswept_savanna" | "badlands"
        | "eroded_badlands" | "wooded_badlands" | "nether_wastes" | "warped_forest"
        | "crimson_forest" | "soul_sand_valley" | "basalt_deltas" => {
            (false, 2.0, 0.0, 4_159_204)
        }
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
        ("coordinate_scale".to_string(), Tag::Double(coordinate_scale)),
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
    Ok(Some(format!(
        "data:image/png;base64,{}",
        encode_base64(&bytes)
    )))
}

pub fn status_json(properties: &ServerProperties, favicon: Option<&str>) -> String {
    let players = if properties.hide_online_players {
        "\"players\":{\"max\":0,\"online\":0}".to_string()
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
        cow_sound_variant_nbt, encode_base64, escape_json_string, handle_legacy_status_connection,
        instrument_nbt, jukebox_song_nbt, legacy_disconnect_packet, legacy_version0_response,
        legacy_version1_response, pig_sound_variant_nbt, read_packet, status_json,
        trim_material_nbt, trim_pattern_nbt, wolf_sound_variant_nbt, write_legacy_string,
        vanilla_baseline_biome_nbt,
        write_minimal_damage_type_registry_packet, write_minimal_dimension_type_registry_packet,
        write_minimal_biome_registry_packet, write_minimal_trim_material_registry_packet,
        write_status_pong_packet,
        write_vanilla_banner_pattern_registry_packet, write_vanilla_cat_variant_registry_packet,
        write_vanilla_cat_sound_variant_registry_packet,
        write_vanilla_chat_type_registry_packet, write_vanilla_chicken_variant_registry_packet,
        write_vanilla_chicken_sound_variant_registry_packet,
        write_vanilla_cow_variant_registry_packet, write_vanilla_cow_sound_variant_registry_packet,
        write_vanilla_frog_variant_registry_packet,
        write_vanilla_instrument_registry_packet, write_vanilla_jukebox_song_registry_packet,
        write_vanilla_painting_variant_registry_packet,
        write_vanilla_pig_variant_registry_packet, write_vanilla_pig_sound_variant_registry_packet,
        write_vanilla_trim_pattern_registry_packet, write_vanilla_wolf_sound_variant_registry_packet,
        write_vanilla_wolf_variant_registry_packet, write_vanilla_zombie_nautilus_variant_registry_packet,
        BANNER_PATTERNS, BANNER_PATTERN_TAGS, BIOMES,
        wait_for_configuration_packet, write_framed_packet, CHAT_TYPES, DAMAGE_TYPE_TAGS,
        INSTRUMENTS, JUKEBOX_SONGS, SERVERBOUND_CONFIGURATION_CLIENT_INFORMATION_PACKET_ID,
        SERVERBOUND_CONFIGURATION_CUSTOM_PAYLOAD_PACKET_ID,
        SERVERBOUND_CONFIGURATION_SELECT_KNOWN_PACKS_PACKET_ID, TRIM_MATERIALS, VERSION_NAME,
    };
    use crate::network::codec::write_identifier;
    use crate::network::ping::ServerboundPingRequestPacket;
    use crate::network::varint::{read_var_i32, write_var_i32};
    use crate::registry::Identifier;
    use crate::server_properties::ServerProperties;
    use crate::storage::nbt::Tag;
    use std::io::{self, Cursor, Read, Write};
    use std::path::Path;

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
    fn includes_favicon_when_present() {
        let properties = test_properties();
        let json = status_json(&properties, Some("data:image/png;base64,iVBORw0KGgo="));
        assert!(json.contains("\"favicon\":\"data:image/png;base64,iVBORw0KGgo=\""));
    }

    #[test]
    fn encodes_base64_padding_cases() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
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
            assert!(matches!(field_value(&tag, "temperature"), Some(Tag::Float(_))));
            assert!(matches!(field_value(&tag, "downfall"), Some(Tag::Float(_))));
            assert!(matches!(field_value(&tag, "effects"), Some(Tag::Compound(_))));

            let effects = compound_field(&tag, "effects");
            assert!(matches!(field_value(effects, "water_color"), Some(Tag::Int(_))));
        }
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
