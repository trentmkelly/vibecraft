use super::{
    BanEntry, BlockPos, IpLogPolicy, NameAndId, OpEntry, PlayerAccess, ProxyConnectionDecision,
    SpawnProtection,
};
use std::fs;

#[cfg(vibecraft_has_decompiled_sources)]
const JAVA_SOURCE: &str =
    vibecraft_java_source!("/net/minecraft/server/players/NameAndId.java");

#[test]
fn name_and_id_matches_java_legacy_json_and_codec_shapes() {
    let profile = NameAndId {
        uuid: "069A79F4-44E9-4726-A5BE-FCA90E38AAF5".to_string(),
        name: "Notch\"".to_string(),
    };
    let legacy = profile.to_json_value();
    assert_eq!(legacy["uuid"], "069a79f4-44e9-4726-a5be-fca90e38aaf5");
    assert_eq!(legacy["name"], "Notch\"");
    assert_eq!(NameAndId::from_json(&legacy), Some(NameAndId {
        uuid: "069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string(),
        name: "Notch\"".to_string(),
    }));

    let codec = profile.to_codec_value().expect("valid UUID codec value");
    assert_eq!(codec["id"], "069a79f4-44e9-4726-a5be-fca90e38aaf5");
    assert_eq!(codec["name"], "Notch\"");
    assert_eq!(NameAndId::from_codec_value(&codec), Ok(NameAndId {
        uuid: "069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string(),
        name: "Notch\"".to_string(),
    }));

    #[cfg(vibecraft_has_decompiled_sources)]
    {
        for source_fragment in [
            "public record NameAndId(UUID id, String name)",
            "UUIDUtil.STRING_CODEC.fieldOf(\"id\")",
            "object.has(\"uuid\") && object.has(\"name\")",
            "output.addProperty(\"uuid\", this.id().toString())",
            "UUIDUtil.createOfflinePlayerUUID(name)",
        ] {
            assert!(JAVA_SOURCE.contains(source_fragment), "missing: {source_fragment}");
        }
    }
}

#[test]
fn name_and_id_rejects_invalid_uuid_and_missing_codec_fields() {
    let invalid_legacy = serde_json::json!({"uuid": "not-a-uuid", "name": "Steve"});
    assert_eq!(NameAndId::from_json(&invalid_legacy), None);
    assert_eq!(
        NameAndId::from_codec_value(&serde_json::json!({"id": "not-a-uuid", "name": "Steve"})),
        Err("Invalid UUID not-a-uuid".to_string())
    );
    assert_eq!(
        NameAndId::from_codec_value(&serde_json::json!({"id": "00000000-0000-0000-0000-000000000001"})),
        Err("NameAndId.name must be a string".to_string())
    );
    assert_eq!(
        (NameAndId {
            uuid: "not-a-uuid".to_string(),
            name: "Steve".to_string(),
        })
        .to_codec_value(),
        Err("Invalid UUID not-a-uuid".to_string())
    );
}

fn player() -> NameAndId {
    NameAndId {
        uuid: "00000000-0000-0000-0000-000000000001".to_string(),
        name: "Steve".to_string(),
    }
}

#[test]
fn checks_bans_whitelist_and_op_level() {
    let mut access = PlayerAccess::default();
    let player = player();
    access.ban_player(BanEntry {
        user: player.clone(),
        created: "2026-05-16 00:00:00 +0000".to_string(),
        source: "Server".to_string(),
        expires: None,
        reason: Some("test".to_string()),
    });
    access.ban_ip(BanEntry {
        user: "127.0.0.1".to_string(),
        created: "2026-05-16 00:00:00 +0000".to_string(),
        source: "Server".to_string(),
        expires: None,
        reason: None,
    });
    access.whitelist(player.clone());
    access.op(OpEntry {
        user: player.clone(),
        level: 4,
        bypasses_player_limit: true,
    });

    assert!(access.is_player_banned(&player.uuid));
    assert!(access.is_ip_banned("127.0.0.1"));
    assert!(access.is_whitelisted(&player.uuid));
    assert_eq!(access.op_level(&player.uuid), Some(4));
}

#[test]
fn writes_vanilla_access_control_files() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vibecraft-access-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let mut access = PlayerAccess::default();
    let player = player();
    access.cache_user(player.clone());
    access.whitelist(player.clone());
    access.op(OpEntry {
        user: player,
        level: 3,
        bypasses_player_limit: false,
    });
    access.save_all(&dir).unwrap();

    assert!(dir.join("ops.json").is_file());
    assert!(dir.join("whitelist.json").is_file());
    assert!(dir.join("banned-players.json").is_file());
    assert!(dir.join("banned-ips.json").is_file());
    assert!(dir.join("usercache.json").is_file());
    assert!(fs::read_to_string(dir.join("ops.json"))
        .unwrap()
        .contains("\"level\":3"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn loads_vanilla_access_control_files() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vibecraft-access-load-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("whitelist.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Steve\"}]",
    )
    .unwrap();
    fs::write(
        dir.join("ops.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\",\"level\":3,\"bypassesPlayerLimit\":true}]",
    )
    .unwrap();
    fs::write(
        dir.join("banned-players.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"Griefer\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"test\"}]",
    )
    .unwrap();
    fs::write(
        dir.join("banned-ips.json"),
        "[{\"ip\":\"127.0.0.1\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"test\"}]",
    )
    .unwrap();
    fs::write(
        dir.join("usercache.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000004\",\"name\":\"Cached\",\"expiresOn\":\"2999-01-01 00:00:00 +0000\"}]",
    )
    .unwrap();

    let mut access = PlayerAccess::load_from_dir(&dir).unwrap();
    assert!(access.is_whitelisted("00000000-0000-0000-0000-000000000001"));
    assert_eq!(
        access.op_level("00000000-0000-0000-0000-000000000002"),
        Some(3)
    );
    assert!(access.is_player_banned("00000000-0000-0000-0000-000000000003"));
    assert!(access.is_ip_banned("127.0.0.1"));
    assert_eq!(
        access
            .lookup_cached_profile("cached", std::time::SystemTime::now())
            .unwrap()
            .name,
        "Cached"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn reload_from_dir_reflects_hot_edited_operator_whitelist_and_ban_files() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("vibecraft-access-hot-edit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("whitelist.json"), "[]").unwrap();
    fs::write(
        dir.join("ops.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Steve\",\"level\":4,\"bypassesPlayerLimit\":false}]",
    )
    .unwrap();
    fs::write(dir.join("banned-players.json"), "[]").unwrap();
    fs::write(dir.join("banned-ips.json"), "[]").unwrap();
    fs::write(dir.join("usercache.json"), "[]").unwrap();

    let first = PlayerAccess::load_from_dir(&dir).unwrap();
    assert_eq!(
        first.op_level("00000000-0000-0000-0000-000000000001"),
        Some(4)
    );
    assert!(!first.is_whitelisted("00000000-0000-0000-0000-000000000002"));
    assert!(!first.is_player_banned("00000000-0000-0000-0000-000000000003"));

    fs::write(
        dir.join("ops.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\",\"level\":2,\"bypassesPlayerLimit\":true}]",
    )
    .unwrap();
    fs::write(
        dir.join("whitelist.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Alex\"}]",
    )
    .unwrap();
    fs::write(
        dir.join("banned-players.json"),
        "[{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"Griefer\",\"created\":\"2026-05-17 00:00:00 +0000\",\"source\":\"Server\",\"expires\":\"forever\",\"reason\":\"reload\"}]",
    )
    .unwrap();

    let reloaded = PlayerAccess::load_from_dir(&dir).unwrap();
    assert!(!reloaded.is_op("00000000-0000-0000-0000-000000000001"));
    assert_eq!(
        reloaded.op_level("00000000-0000-0000-0000-000000000002"),
        Some(2)
    );
    assert!(reloaded.is_whitelisted("00000000-0000-0000-0000-000000000002"));
    assert!(reloaded.is_player_banned("00000000-0000-0000-0000-000000000003"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn usercache_loader_ignores_malformed_missing_and_expired_entries() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "vibecraft-usercache-corrupt-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    fs::write(
        dir.join("usercache.json"),
        concat!(
            "[",
            "{\"uuid\":\"00000000-0000-0000-0000-000000000001\",\"name\":\"Valid\",\"expiresOn\":\"2999-01-01 00:00:00 +0000\"},",
            "{\"uuid\":\"00000000-0000-0000-0000-000000000002\",\"name\":\"Expired\",\"expiresOn\":\"2000-01-01 00:00:00 +0000\"},",
            "{\"uuid\":\"00000000-0000-0000-0000-000000000003\",\"name\":\"MissingDate\"},",
            "{\"uuid\":\"00000000-0000-0000-0000-000000000004\",\"name\":\"MalformedDate\",\"expiresOn\":\"not a date\"}",
            "]"
        ),
    )
    .unwrap();

    let mut access = PlayerAccess::load_from_dir(&dir).unwrap();
    let now = std::time::SystemTime::now();
    assert_eq!(
        access.lookup_cached_profile("valid", now).unwrap().name,
        "Valid"
    );
    assert!(access.lookup_cached_profile("expired", now).is_none());
    assert!(access.lookup_cached_profile("missingdate", now).is_none());
    assert!(access.lookup_cached_profile("malformeddate", now).is_none());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn profile_cache_expires_entries() {
    let mut access = PlayerAccess::default();
    let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
    let player = player();
    access.profile_cache.insert(player.clone(), now);

    assert_eq!(
        access.lookup_cached_profile("steve", now + std::time::Duration::from_secs(60)),
        Some(player)
    );
    assert_eq!(
        access.lookup_cached_profile(
            "steve",
            now + std::time::Duration::from_secs(60 * 60 * 24 * 31)
        ),
        None
    );
}

#[test]
fn derives_offline_mode_uuid() {
    let player = NameAndId::create_offline("Steve");
    assert_eq!(player.name, "Steve");
    assert_eq!(player.uuid, "5627dd98-e6be-3c21-b8a8-e92344183641");
}

#[test]
fn prevent_proxy_connections_rejects_host_socket_ip_mismatch() {
    let access = PlayerAccess::default();
    assert_eq!(
        access.check_proxy_connection(true, "203.0.113.10", "198.51.100.20"),
        ProxyConnectionDecision::RejectPreventProxyConnections
    );
    assert_eq!(
        access.check_proxy_connection(true, "203.0.113.10", "203.0.113.10"),
        ProxyConnectionDecision::Allow
    );
    assert_eq!(
        access.check_proxy_connection(false, "203.0.113.10", "198.51.100.20"),
        ProxyConnectionDecision::Allow
    );
}

#[test]
fn ip_log_policy_redacts_remote_addresses_when_disabled() {
    let access = PlayerAccess::default();
    assert_eq!(access.ip_log_policy(true), IpLogPolicy::Include);
    assert_eq!(access.ip_log_policy(false), IpLogPolicy::Redact);
    assert_eq!(
        access.ip_log_policy(true).format_remote("203.0.113.10"),
        "203.0.113.10"
    );
    assert_eq!(
        access.ip_log_policy(false).format_remote("203.0.113.10"),
        "IP hidden"
    );
}

#[test]
fn spawn_protection_matches_dedicated_server_rules() {
    let mut access = PlayerAccess::default();
    let player = player();
    let op = NameAndId {
        uuid: "00000000-0000-0000-0000-000000000002".to_string(),
        name: "Alex".to_string(),
    };
    let protection = SpawnProtection {
        radius: 16,
        spawn_dimension: "minecraft:overworld".to_string(),
        spawn_pos: BlockPos { x: 0, y: 64, z: 0 },
    };

    assert!(!access.is_under_spawn_protection(
        &protection,
        "minecraft:overworld",
        BlockPos { x: 0, y: 64, z: 0 },
        &player.uuid
    ));

    access.op(OpEntry {
        user: op.clone(),
        level: 4,
        bypasses_player_limit: true,
    });

    assert!(access.is_under_spawn_protection(
        &protection,
        "minecraft:overworld",
        BlockPos {
            x: 16,
            y: -64,
            z: 0
        },
        &player.uuid
    ));
    assert!(!access.is_under_spawn_protection(
        &protection,
        "minecraft:overworld",
        BlockPos { x: 17, y: 64, z: 0 },
        &player.uuid
    ));
    assert!(!access.is_under_spawn_protection(
        &protection,
        "minecraft:the_nether",
        BlockPos { x: 0, y: 64, z: 0 },
        &player.uuid
    ));
    assert!(!access.is_under_spawn_protection(
        &protection,
        "minecraft:overworld",
        BlockPos { x: 0, y: 64, z: 0 },
        &op.uuid
    ));
    assert!(!access.is_under_spawn_protection(
        &SpawnProtection {
            radius: 0,
            ..protection
        },
        "minecraft:overworld",
        BlockPos { x: 0, y: 64, z: 0 },
        &player.uuid
    ));
}
