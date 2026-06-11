use super::{
    banner_pattern_nbt, bug_report_server_links_packet, build_player_status, cat_sound_variant_nbt,
    chicken_sound_variant_nbt, cow_sound_variant_nbt, encode_base64, escape_json_string,
    function_permission_level_from_properties, handle_legacy_status_connection, instrument_nbt,
    legacy_disconnect_packet, legacy_version0_response, legacy_version1_response,
    load_code_of_conduct_for_language, load_favicon, login_access_disconnect_reason,
    login_compression_threshold, login_host_ip, pig_sound_variant_nbt, read_code_of_conducts,
    read_packet, resolve_status_icon_from, status_json, strip_minecraft_formatting,
    trim_material_nbt, trim_pattern_nbt, wolf_sound_variant_nbt, write_legacy_string,
    write_minimal_biome_registry_packet, write_minimal_damage_type_registry_packet,
    write_minimal_dimension_type_registry_packet, write_minimal_trim_material_registry_packet,
    write_status_pong_packet, write_transfers_disabled_disconnect,
    write_vanilla_banner_pattern_registry_packet, write_vanilla_cat_sound_variant_registry_packet,
    write_vanilla_cat_variant_registry_packet, write_vanilla_chat_type_registry_packet,
    write_vanilla_chicken_sound_variant_registry_packet,
    write_vanilla_chicken_variant_registry_packet, write_vanilla_cow_sound_variant_registry_packet,
    write_vanilla_cow_variant_registry_packet, write_vanilla_frog_variant_registry_packet,
    write_vanilla_instrument_registry_packet, write_vanilla_jukebox_song_registry_packet,
    write_vanilla_painting_variant_registry_packet,
    write_vanilla_pig_sound_variant_registry_packet, write_vanilla_pig_variant_registry_packet,
    write_vanilla_trim_pattern_registry_packet, write_vanilla_wolf_sound_variant_registry_packet,
    write_vanilla_wolf_variant_registry_packet,
    write_vanilla_zombie_nautilus_variant_registry_packet, write_world_clock_registry_packet,
    ActiveLoginRegistry, StatusPlayer, INSTRUMENTS, MAX_PACKET_SIZE, MAX_STATUS_PLAYER_SAMPLE,
    STATUS_ANONYMOUS_NAME, STATUS_ANONYMOUS_UUID, TRIM_MATERIALS,
};
use crate::command::PermissionLevel;
use crate::network::common::{ServerLinkLabel, ServerLinkType};
use crate::network::ping::ServerboundPingRequestPacket;
use crate::network::play::pack_block_position;
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::random_source::LegacyRandom;
use crate::server_properties::ServerProperties;
use crate::storage::nbt::Tag;
use crate::{biome, damage_type, equipment_trim, presentation_data};
use std::fs;
use std::io::{self, Cursor, Read};
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
    let mut properties =
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
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
    root.push(format!("vibecraft-codeofconduct-{}", std::process::id()));
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

    let mut properties =
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
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
    properties.set("motd", "VibeCraft Test");
    let json = status_json(&properties, None, &[]);
    assert!(json.contains("\"name\":\"26.1.2\""));
    assert!(json.contains("\"protocol\":775"));
    assert!(json.contains("\"max\":20"));
    assert!(json.contains("\"players\":{\"max\":20,\"online\":0,\"sample\":[]}"));
    assert!(json.contains("\"description\":{\"text\":\"VibeCraft Test\"}"));
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

    let json = status_json(&properties, None, &[]);

    assert!(json.contains("\"players\":{\"max\":37,\"online\":0,\"sample\":[]}"));
}

#[test]
pub fn transfer_intent_disabled_sends_vanilla_transfers_disabled_disconnect() {
    // Java ServerHandshakePacketListenerImpl TRANSFER case with acceptsTransfers()
    // false: a login-state disconnect with `multiplayer.disconnect.transfers_disabled`
    // (no translation args).
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        write_transfers_disabled_disconnect(&mut stream).unwrap();
    });

    let mut client = std::net::TcpStream::connect(addr).unwrap();
    let frame = read_packet(&mut client).unwrap();
    let mut payload = &frame[..];
    assert_eq!(
        read_var_i32(&mut payload).unwrap(),
        crate::network::login::CLIENTBOUND_LOGIN_DISCONNECT_PACKET_ID
    );
    let packet =
        crate::network::login::ClientboundLoginDisconnectPacket::read(&mut payload).unwrap();
    assert_eq!(
        packet.reason.0,
        "{\"translate\":\"multiplayer.disconnect.transfers_disabled\"}"
    );
    handle.join().unwrap();
}

/// A connected loopback (server_end, client_end) pair for exercising packet
/// writers against a real socket.
fn loopback_pair() -> (std::net::TcpStream, std::net::TcpStream) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client = std::net::TcpStream::connect(addr).unwrap();
    let (server, _) = listener.accept().unwrap();
    (server, client)
}

#[test]
pub fn keep_alive_tick_sends_challenge_after_the_vanilla_interval() {
    let (mut server, mut client) = loopback_pair();
    let clock = Arc::new(Mutex::new(crate::world_time::ServerClockManager::default()));
    let mut keep_alive = crate::network::common::KeepAliveState::new(0, 0);
    // Back-date the epoch so `now_ms` ≈ 16 s > the 15 s interval → a challenge sends.
    let epoch = std::time::Instant::now() - std::time::Duration::from_secs(16);
    let mut last_time_sync = std::time::Instant::now();

    let alive = super::tick_keep_alive_and_time(
        &mut server,
        crate::network::compression::CompressionState::disabled(),
        &clock,
        &mut keep_alive,
        epoch,
        &mut last_time_sync,
    )
    .unwrap();

    assert!(alive);
    assert!(keep_alive.is_pending());
    let frame = read_packet(&mut client).unwrap();
    let mut payload = &frame[..];
    assert_eq!(
        read_var_i32(&mut payload).unwrap(),
        crate::network::play::CLIENTBOUND_KEEP_ALIVE_PACKET_ID
    );
}

#[test]
pub fn keep_alive_tick_disconnects_when_previous_challenge_unanswered() {
    let (mut server, mut client) = loopback_pair();
    let clock = Arc::new(Mutex::new(crate::world_time::ServerClockManager::default()));
    // Pre-seed a pending challenge at t=15 s (via the model), then run the live tick
    // at ≈32 s — still pending one interval later → `disconnect.timeout`.
    let mut keep_alive = crate::network::common::KeepAliveState::new(0, 0);
    assert!(matches!(
        keep_alive.tick(15_000, false),
        crate::network::common::KeepAliveTick::Send(_)
    ));
    let epoch = std::time::Instant::now() - std::time::Duration::from_secs(32);
    let mut last_time_sync = std::time::Instant::now();

    let alive = super::tick_keep_alive_and_time(
        &mut server,
        crate::network::compression::CompressionState::disabled(),
        &clock,
        &mut keep_alive,
        epoch,
        &mut last_time_sync,
    )
    .unwrap();

    assert!(!alive);
    let frame = read_packet(&mut client).unwrap();
    let mut payload = &frame[..];
    assert_eq!(
        read_var_i32(&mut payload).unwrap(),
        crate::network::play::CLIENTBOUND_DISCONNECT_PACKET_ID
    );
    let rest = String::from_utf8_lossy(payload);
    assert!(rest.contains("disconnect.timeout"));
}

fn status_player(name: &str, allows_listing: bool) -> StatusPlayer {
    StatusPlayer {
        uuid: crate::player_access::NameAndId::create_offline(name).uuid,
        name: name.to_string(),
        allows_listing,
    }
}

#[test]
pub fn status_json_reports_live_online_count_and_listed_player_sample() {
    let properties = test_properties();
    let players = [status_player("Alex", true), status_player("Steve", true)];

    let json = status_json(&properties, None, &players);

    // Online count reflects the live in-play set, max from properties.
    assert!(json.contains("\"online\":2"));
    assert!(json.contains("\"max\":20"));
    // Both listed players appear by name + dashed UUID (order is shuffled, so
    // assert membership rather than a fixed sequence).
    assert!(json.contains("\"name\":\"Alex\""));
    assert!(json.contains("\"name\":\"Steve\""));
    assert!(json.contains(&format!(
        "\"id\":\"{}\"",
        crate::player_access::NameAndId::create_offline("Alex").uuid
    )));
}

#[test]
pub fn status_json_anonymises_players_who_opt_out_of_listing() {
    let properties = test_properties();
    // Steve opted out (allowsListing=false) → shown as the anonymous nil profile.
    let players = [status_player("Steve", false)];

    let json = status_json(&properties, None, &players);

    assert!(json.contains("\"online\":1"));
    assert!(json.contains(&format!("\"name\":\"{STATUS_ANONYMOUS_NAME}\"")));
    assert!(json.contains(&format!("\"id\":\"{STATUS_ANONYMOUS_UUID}\"")));
    assert!(!json.contains("\"name\":\"Steve\""));
}

#[test]
pub fn build_player_status_includes_every_player_when_under_sample_cap() {
    // For online <= 12 the whole roster is included (Mth.nextInt(0,0) offset),
    // only reordered by the shuffle.
    let players: Vec<StatusPlayer> = (0..MAX_STATUS_PLAYER_SAMPLE)
        .map(|index| status_player(&format!("Player{index}"), true))
        .collect();
    let mut random = LegacyRandom::new(1);

    let (online, sample) = build_player_status(&players, false, &mut random);

    assert_eq!(online, MAX_STATUS_PLAYER_SAMPLE);
    assert_eq!(sample.len(), MAX_STATUS_PLAYER_SAMPLE);
    let names: std::collections::BTreeSet<&str> =
        sample.iter().map(|(_, name)| name.as_str()).collect();
    let expected: std::collections::BTreeSet<&str> =
        players.iter().map(|player| player.name.as_str()).collect();
    assert_eq!(names, expected);
}

#[test]
pub fn build_player_status_caps_sample_at_twelve_and_keeps_full_count() {
    // 20 players online → online=20 but the sample is capped at 12 (Java
    // MAX_STATUS_PLAYER_SAMPLE), drawn from a contiguous window.
    let players: Vec<StatusPlayer> = (0..20)
        .map(|index| status_player(&format!("Player{index:02}"), true))
        .collect();
    let mut random = LegacyRandom::new(42);

    let (online, sample) = build_player_status(&players, false, &mut random);

    assert_eq!(online, 20);
    assert_eq!(sample.len(), MAX_STATUS_PLAYER_SAMPLE);
}

#[test]
pub fn build_player_status_hidden_returns_count_only() {
    let players = [status_player("Alex", true), status_player("Steve", true)];
    let mut random = LegacyRandom::new(7);

    let (online, sample) = build_player_status(&players, true, &mut random);

    assert_eq!(online, 2);
    assert!(sample.is_empty());
}

/// Open a real loopback TCP connection and return the client end, so registry
/// sessions (which hold a cloned `TcpStream`) can be exercised without a socket
/// mock. The accepted server end is dropped — only the handle matters here.
fn loopback_stream() -> std::net::TcpStream {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client = std::net::TcpStream::connect(addr).unwrap();
    let _server = listener.accept().unwrap();
    client
}

#[test]
pub fn active_login_registry_only_counts_in_play_sessions_and_tracks_listing() {
    let registry = ActiveLoginRegistry::default();
    let alex = crate::player_access::NameAndId::create_offline("Alex");
    let (alex_guard, replaced) = registry
        .register_replacing(&alex.uuid, "Alex", &loopback_stream())
        .unwrap();
    assert!(replaced.is_none());

    // Still in login/config — not counted toward the status online total yet
    // (Java only counts players added to PlayerList on play entry).
    assert_eq!(registry.online_count(), 0);
    assert!(registry.status_players().is_empty());

    // Entering PLAY makes the player visible; listing defaults to false
    // (ClientInformation.createDefault), so the sample anonymises them.
    alex_guard.mark_in_play();
    assert_eq!(registry.online_count(), 1);
    assert_eq!(registry.online_player_names(), vec!["Alex".to_string()]);
    let sampled = registry.status_players();
    assert_eq!(sampled.len(), 1);
    assert!(!sampled[0].allows_listing);

    // A ClientInformation opt-in flips the listing flag for the sample.
    alex_guard.set_allows_listing(true);
    assert!(registry.status_players()[0].allows_listing);
}

#[test]
pub fn active_login_guard_in_play_profiles_enumerates_full_online_roster() {
    // Two distinct players online — the roster snapshot a guard exposes (used to
    // seed `/list`) must contain BOTH, not just the guard's own player. Matches
    // Java PlayerList.getPlayers being shared across all connections.
    let registry = ActiveLoginRegistry::default();
    let alex = crate::player_access::NameAndId::create_offline("Alex");
    let steve = crate::player_access::NameAndId::create_offline("Steve");
    let (alex_guard, _) = registry
        .register_replacing(&alex.uuid, "Alex", &loopback_stream())
        .unwrap();
    let (steve_guard, _) = registry
        .register_replacing(&steve.uuid, "Steve", &loopback_stream())
        .unwrap();
    alex_guard.mark_in_play();
    steve_guard.mark_in_play();

    // Either guard sees the whole roster (the session map is shared).
    let mut names: Vec<String> = alex_guard
        .in_play_profiles()
        .into_iter()
        .map(|profile| profile.name)
        .collect();
    names.sort();
    assert_eq!(names, vec!["Alex".to_string(), "Steve".to_string()]);
    assert_eq!(steve_guard.in_play_profiles().len(), 2);
}

#[test]
pub fn active_login_guard_captures_client_language_for_code_of_conduct() {
    let registry = ActiveLoginRegistry::default();
    let alex = crate::player_access::NameAndId::create_offline("Alex");
    let (guard, _) = registry
        .register_replacing(&alex.uuid, "Alex", &loopback_stream())
        .unwrap();

    // Defaults to en_us before any ClientInformation arrives.
    assert_eq!(guard.language(), "en_us");

    // The client locale is captured (lowercased, like Java's
    // codeOfConducts.get(language.toLowerCase())).
    guard.set_language("FR_FR");
    assert_eq!(guard.language(), "fr_fr");
}

#[test]
pub fn active_login_registry_replacement_uses_new_session_and_token_guards_old() {
    let registry = ActiveLoginRegistry::default();
    let steve = crate::player_access::NameAndId::create_offline("Steve");
    let (old_guard, _) = registry
        .register_replacing(&steve.uuid, "Steve", &loopback_stream())
        .unwrap();
    old_guard.mark_in_play();

    // A second login for the same UUID replaces the session and returns the old
    // stream so the caller can disconnect it ("logged in from another location").
    let (new_guard, replaced) = registry
        .register_replacing(&steve.uuid, "Steve", &loopback_stream())
        .unwrap();
    assert!(replaced.is_some());
    new_guard.mark_in_play();
    assert_eq!(registry.online_count(), 1);

    // The stale guard's token no longer matches, so dropping/updating it cannot
    // clobber the live session.
    old_guard.set_allows_listing(true);
    assert!(!registry.status_players()[0].allows_listing);
    new_guard.set_allows_listing(true);
    assert!(registry.status_players()[0].allows_listing);
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
pub fn login_access_gate_matches_java_ban_whitelist_and_op_order() {
    let mut properties = test_properties();
    properties.set("enforce-whitelist", "true");
    let steve = crate::player_access::NameAndId::create_offline("Steve");
    let alex = crate::player_access::NameAndId::create_offline("Alex");
    let griefer = crate::player_access::NameAndId::create_offline("Griefer");
    let op = crate::player_access::NameAndId::create_offline("Operator");
    let mut access = crate::player_access::PlayerAccess::default();
    access.ban_player(crate::player_access::BanEntry {
        user: griefer.clone(),
        created: "2026-05-25 00:00:00 +0000".to_string(),
        source: "Server".to_string(),
        expires: None,
        reason: Some("test".to_string()),
    });
    access.ban_ip(crate::player_access::BanEntry {
        user: "203.0.113.7".to_string(),
        created: "2026-05-25 00:00:00 +0000".to_string(),
        source: "Server".to_string(),
        expires: None,
        reason: Some("test".to_string()),
    });
    access.whitelist(alex.clone());
    access.op(crate::player_access::OpEntry {
        user: op.clone(),
        level: 4,
        bypasses_player_limit: true,
    });
    let access = Arc::new(Mutex::new(access));

    assert_eq!(
        login_access_disconnect_reason(&properties, &access, &griefer, "203.0.113.7", None)
            .unwrap(),
        Some("multiplayer.disconnect.banned")
    );
    assert_eq!(
        login_access_disconnect_reason(&properties, &access, &steve, "203.0.113.7", None).unwrap(),
        Some("multiplayer.disconnect.not_whitelisted")
    );
    assert_eq!(
        login_access_disconnect_reason(&properties, &access, &alex, "203.0.113.7", None).unwrap(),
        Some("multiplayer.disconnect.ip_banned")
    );
    assert_eq!(
        login_access_disconnect_reason(&properties, &access, &op, "198.51.100.4", None).unwrap(),
        None
    );
}

#[test]
pub fn includes_favicon_when_present() {
    let properties = test_properties();
    let json = status_json(&properties, Some("data:image/png;base64,iVBORw0KGgo="), &[]);
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
pub fn status_icon_prefers_server_icon_falls_back_to_world_icon_and_tolerates_bad_icons() {
    let pid = std::process::id();
    let server_icon = std::env::temp_dir().join(format!("vibecraft-resolve-{pid}-server-icon.png"));
    let world_icon = std::env::temp_dir().join(format!("vibecraft-resolve-{pid}-world-icon.png"));
    let _ = fs::remove_file(&server_icon);
    let _ = fs::remove_file(&world_icon);

    // Neither present -> no favicon.
    assert_eq!(resolve_status_icon_from(&server_icon, &world_icon), None);

    // Only the world icon present -> fallback is used.
    fs::write(&world_icon, png_header(64, 64)).unwrap();
    assert!(resolve_status_icon_from(&server_icon, &world_icon)
        .unwrap()
        .starts_with("data:image/png;base64,"));

    // server-icon.png present and valid -> it wins over the world icon.
    fs::write(&server_icon, png_header(64, 64)).unwrap();
    assert!(resolve_status_icon_from(&server_icon, &world_icon).is_some());

    // server-icon.png present but WRONG size -> Java picks it, fails validation,
    // logs, and returns no icon WITHOUT falling back or aborting startup.
    fs::write(&server_icon, png_header(32, 64)).unwrap();
    assert_eq!(resolve_status_icon_from(&server_icon, &world_icon), None);

    fs::remove_file(&server_icon).unwrap();
    fs::remove_file(&world_icon).unwrap();
}

#[test]
pub fn encodes_base64_padding_cases() {
    assert_eq!(encode_base64(b""), "");
    assert_eq!(encode_base64(b"f"), "Zg==");
    assert_eq!(encode_base64(b"fo"), "Zm8=");
    assert_eq!(encode_base64(b"foo"), "Zm9v");
}

pub fn temp_status_test_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vibecraft-status-{}-{name}", std::process::id()))
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
pub fn formats_legacy_status_responses_like_vanilla() -> io::Result<()> {
    let properties = test_properties();

    assert_eq!(
        legacy_version0_response(&properties, 0),
        "A Minecraft Server§0§20"
    );
    assert_eq!(
        legacy_version1_response(&properties, 0),
        "§1\x00127\x0026.1.2\0A Minecraft Server\x000\x0020"
    );

    let packet = legacy_disconnect_packet("hello")?;
    assert_eq!(packet[0], 255);
    assert_eq!(u16::from_be_bytes([packet[1], packet[2]]), 5);
    Ok(())
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
    handle_legacy_status_connection(&mut stream, &properties, 0).unwrap();

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
        cow_sound_variant_nbt("classic"),
        &["ambient_sound", "hurt_sound", "death_sound", "step_sound"],
    );
    assert_nested_sound_variant_fields(
        chicken_sound_variant_nbt("classic"),
        &["ambient_sound", "hurt_sound", "death_sound", "step_sound"],
    );
    assert_nested_sound_variant_fields(
        pig_sound_variant_nbt("classic"),
        &[
            "ambient_sound",
            "hurt_sound",
            "death_sound",
            "step_sound",
            "eat_sound",
        ],
    );
    assert_nested_sound_variant_fields(
        cat_sound_variant_nbt("classic"),
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
        wolf_sound_variant_nbt("classic"),
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
pub fn trim_material_registry_payloads_match_vanilla_colors_and_overrides() {
    // (id, description color as serialized by TextColor #%06X, override_armor_assets)
    // Values from data/minecraft/trim_material/*.json and MaterialAssetGroup.
    type TrimExpectation = (
        &'static str,
        &'static str,
        &'static [(&'static str, &'static str)],
    );
    let expected: &[TrimExpectation] = &[
        ("quartz", "#E3D4C4", &[]),
        ("iron", "#ECECEC", &[("minecraft:iron", "iron_darker")]),
        (
            "netherite",
            "#625859",
            &[("minecraft:netherite", "netherite_darker")],
        ),
        ("redstone", "#971607", &[]),
        (
            "copper",
            "#B4684D",
            &[("minecraft:copper", "copper_darker")],
        ),
        ("gold", "#DEB12D", &[("minecraft:gold", "gold_darker")]),
        ("emerald", "#11A036", &[]),
        (
            "diamond",
            "#6EECD2",
            &[("minecraft:diamond", "diamond_darker")],
        ),
        ("lapis", "#416E97", &[]),
        ("amethyst", "#9A5CC6", &[]),
        ("resin", "#FC7812", &[]),
    ];
    assert_eq!(TRIM_MATERIALS.len(), expected.len());

    for (id, color, overrides) in expected {
        let material = TRIM_MATERIALS
            .iter()
            .find(|material| material.id == *id)
            .unwrap_or_else(|| panic!("trim material {id} should be sent"));
        let tag = trim_material_nbt(material);

        assert!(
            matches!(field_value(&tag, "asset_name"), Some(Tag::String(value)) if value == id),
            "{id} asset_name"
        );
        let description = compound_field(&tag, "description");
        assert!(
            matches!(field_value(description, "translate"), Some(Tag::String(value)) if value == &format!("trim_material.minecraft.{id}")),
            "{id} translate"
        );
        assert!(
            matches!(field_value(description, "color"), Some(Tag::String(value)) if value == color),
            "{id} color should be {color}"
        );

        if overrides.is_empty() {
            assert!(
                field_value(&tag, "override_armor_assets").is_none(),
                "{id} should have no override_armor_assets"
            );
        } else {
            let group = compound_field(&tag, "override_armor_assets");
            for (asset, suffix) in *overrides {
                assert!(
                    matches!(field_value(group, asset), Some(Tag::String(value)) if value == suffix),
                    "{id} override {asset} should be {suffix}"
                );
            }
        }
    }
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
    assert_core_synced_registry_payload_counts();
    assert_trim_pattern_registry_payload_fields();
    assert_banner_pattern_registry_payload_fields();
    assert_goat_horn_instrument_registry_payload_fields();
}

fn assert_core_synced_registry_payload_counts() {
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
}

fn assert_trim_pattern_registry_payload_fields() {
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
}

fn assert_banner_pattern_registry_payload_fields() {
    let banner = banner_pattern_nbt("flower");
    assert!(matches!(
        field_value(&banner, "asset_id"),
        Some(Tag::String(value)) if value == "minecraft:flower"
    ));
    assert!(matches!(
        field_value(&banner, "translation_key"),
        Some(Tag::String(value)) if value == "block.minecraft.banner.flower"
    ));
}

fn assert_goat_horn_instrument_registry_payload_fields() {
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
mod tests_b;
pub use tests_b::*;
mod recipe_book_packets;
mod resource_pack_properties;
mod tests_c;
mod tests_d;
pub use tests_d::*;
mod tests_e;
