use super::{
    banner_pattern_nbt, bug_report_server_links_packet, cat_sound_variant_nbt, chat_type_nbt,
    chicken_sound_variant_nbt, chunk_batch_size, chunk_has_non_air_blocks, chunk_window,
    cow_sound_variant_nbt, day_timeline_nbt, early_game_timeline_nbt, encode_base64,
    escape_json_string, function_permission_level_from_properties, handle_legacy_status_connection,
    instrument_nbt, inventory_internal_slot, jukebox_song_nbt, legacy_disconnect_packet,
    legacy_version0_response, legacy_version1_response, load_code_of_conduct_for_language,
    load_favicon, login_access_disconnect_reason, login_compression_threshold, login_host_ip,
    moon_timeline_nbt, newly_visible_chunks, overworld_dimension_type_nbt, packed_chunk_pos,
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
use crate::command::PermissionLevel;
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
pub fn bug_report_link_becomes_known_server_link_when_valid() {
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

    properties.set("bug-report-link", "ftp://example.invalid/bugs");
    assert!(bug_report_server_links_packet(&properties).is_none());

    properties.set("bug-report-link", "https://example.invalid/bug report");
    assert!(bug_report_server_links_packet(&properties).is_none());
}

#[test]
pub fn code_of_conduct_loader_strips_formatting_and_applies_language_fallback() {
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
pub fn read_packet_rejects_negative_and_oversized_lengths_before_allocating() {
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

pub fn test_properties() -> ServerProperties {
    ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
        .unwrap()
}

#[test]
pub fn escapes_status_description() {
    assert_eq!(
        escape_json_string("A \"quoted\" server"),
        "A \\\"quoted\\\" server"
    );
}

#[test]
pub fn includes_26_1_2_protocol_in_status_json() {
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
pub fn login_compression_threshold_matches_java_negative_disable_gate() {
    let mut properties = test_properties();
    assert_eq!(login_compression_threshold(&properties), Some(256));

    properties.set("network-compression-threshold", "-1");
    assert_eq!(login_compression_threshold(&properties), None);

    properties.set("network-compression-threshold", "0");
    assert_eq!(login_compression_threshold(&properties), Some(0));
}

#[test]
pub fn function_permission_level_clamps_java_permission_ids() {
    let mut properties = test_properties();
    assert_eq!(
        function_permission_level_from_properties(&properties),
        PermissionLevel::Gamemasters
    );

    properties.set("function-permission-level", "-1");
    assert_eq!(
        function_permission_level_from_properties(&properties),
        PermissionLevel::All
    );

    properties.set("function-permission-level", "4");
    assert_eq!(
        function_permission_level_from_properties(&properties),
        PermissionLevel::Owners
    );

    properties.set("function-permission-level", "99");
    assert_eq!(
        function_permission_level_from_properties(&properties),
        PermissionLevel::Owners
    );
}

#[test]
pub fn hidden_online_players_preserves_counts_and_omits_sample_entries() {
    let mut properties = test_properties();
    properties.set("hide-online-players", "true");
    properties.set("max-players", "37");

    let json = status_json(&properties, None);

    assert!(json.contains("\"players\":{\"max\":37,\"online\":0,\"sample\":[]}"));
}

#[test]
pub fn prevent_proxy_connections_rejects_mismatched_handshake_ip() {
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
pub fn includes_favicon_when_present() {
    let properties = test_properties();
    let json = status_json(&properties, Some("data:image/png;base64,iVBORw0KGgo="));
    assert!(json.contains("\"favicon\":\"data:image/png;base64,iVBORw0KGgo=\""));
}

#[test]
pub fn server_icon_loader_requires_64_by_64_png_and_encodes_data_uri() {
    let path = temp_status_test_path("server-icon-64.png");
    fs::write(&path, png_header(64, 64)).unwrap();

    let favicon = load_favicon(&path).unwrap().unwrap();
    assert!(favicon.starts_with("data:image/png;base64,"));

    fs::remove_file(path).unwrap();
}

#[test]
pub fn server_icon_loader_rejects_wrong_png_dimensions() {
    let path = temp_status_test_path("server-icon-32.png");
    fs::write(&path, png_header(32, 64)).unwrap();

    let error = load_favicon(&path).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("64x64"));

    fs::remove_file(path).unwrap();
}

#[test]
pub fn encodes_base64_padding_cases() {
    assert_eq!(encode_base64(b""), "");
    assert_eq!(encode_base64(b"f"), "Zg==");
    assert_eq!(encode_base64(b"fo"), "Zm8=");
    assert_eq!(encode_base64(b"foo"), "Zm9v");
}

pub fn temp_status_test_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("rustcraft-status-{}-{name}", std::process::id()))
}

pub fn png_header(width: u32, height: u32) -> Vec<u8> {
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
pub fn formats_legacy_status_responses_like_vanilla() {
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
pub fn handles_legacy_1_6_ping_host_payload() {
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
pub fn status_ping_packet_echoes_payload_for_client_latency_measurement() {
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
pub fn animal_sound_variant_payloads_match_nested_26_1_2_codecs() {
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
pub fn trim_material_registry_payload_includes_redstone_component_data() {
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
pub fn vanilla_animal_variant_registry_payloads_include_client_referenced_entries() {
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
pub fn synced_registry_payloads_include_expected_counts_and_fields() {
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


mod tests_a2;
pub use tests_a2::*;
mod tests_b;
pub use tests_b::*;
mod tests_c;
pub use tests_c::*;
mod tests_d;
pub use tests_d::*;
mod tests_e;
pub use tests_e::*;
