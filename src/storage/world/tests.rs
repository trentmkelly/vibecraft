use super::{LevelStorageSource, PlayerDataStorage, WorldLayout};
use std::fs;

#[test]
fn exposes_vanilla_world_paths() {
    let layout = WorldLayout::new("world");
    assert_eq!(
        layout.level_dat(),
        std::path::PathBuf::from("world/level.dat")
    );
    assert_eq!(
        layout.level_dat_old(),
        std::path::PathBuf::from("world/level.dat_old")
    );
    assert_eq!(
        layout.session_lock(),
        std::path::PathBuf::from("world/session.lock")
    );
    assert_eq!(
        layout.region_dir(),
        std::path::PathBuf::from("world/region")
    );
    assert_eq!(
        layout.entities_dir(),
        std::path::PathBuf::from("world/entities")
    );
    assert_eq!(layout.poi_dir(), std::path::PathBuf::from("world/poi"));
    assert_eq!(
        layout.playerdata_dir(),
        std::path::PathBuf::from("world/playerdata")
    );
    assert_eq!(
        layout.advancements_dir(),
        std::path::PathBuf::from("world/advancements")
    );
    assert_eq!(layout.stats_dir(), std::path::PathBuf::from("world/stats"));
    assert_eq!(
        layout.datapacks_dir(),
        std::path::PathBuf::from("world/datapacks")
    );
    assert_eq!(layout.data_dir(), std::path::PathBuf::from("world/data"));
    assert_eq!(
        layout.dimensions_dir(),
        std::path::PathBuf::from("world/dimensions")
    );
    assert_eq!(
        layout.player_data_file("uuid"),
        std::path::PathBuf::from("world/playerdata/uuid.dat")
    );
    assert_eq!(
        layout.advancements_file("uuid"),
        std::path::PathBuf::from("world/advancements/uuid.json")
    );
    assert_eq!(
        layout.stats_file("uuid"),
        std::path::PathBuf::from("world/stats/uuid.json")
    );
    assert_eq!(
        layout.saved_data_file("scoreboard"),
        std::path::PathBuf::from("world/data/scoreboard.dat")
    );
}

#[test]
fn creates_base_dirs() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-world-layout-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    layout.ensure_base_dirs().unwrap();

    assert!(layout.region_dir().is_dir());
    assert!(layout.entities_dir().is_dir());
    assert!(layout.dimensions_dir().is_dir());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn level_storage_source_enumerates_and_validates_world_folders() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-source-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let source = LevelStorageSource::new(&path, path.join("../backups")).unwrap();
    assert_eq!(source.name(), "Anvil");
    assert!(source.is_new_level_id_acceptable("new_world"));
    assert!(!source.is_new_level_id_acceptable("../escape"));

    let world = WorldLayout::new(path.join("world_one"));
    world
        .save_level_dat(&crate::storage::nbt::Tag::Compound(vec![(
            "LevelName".to_string(),
            crate::storage::nbt::Tag::String("World One".to_string()),
        )]))
        .unwrap();
    fs::create_dir_all(path.join("not_a_world")).unwrap();

    let candidates = source.find_level_candidates().unwrap();
    assert_eq!(candidates.levels().len(), 1);
    assert_eq!(candidates.levels()[0].directory_name(), "world_one");
    let summaries = candidates.summaries();
    assert_eq!(summaries.len(), 1);
    let summary = summaries[0].as_ref().unwrap();
    assert_eq!(summary.directory_name, "world_one");
    assert_eq!(summary.level_name, "World One");
    assert_eq!(summary.game_type, super::LevelGameType::Survival);
    assert!(!summary.locked);
    assert!(source.level_exists("world_one"));
    assert!(!source.level_exists("../escape"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(world.level_dat(), path.join("world_one/linked.dat")).unwrap();
        let err = source.validate_and_create_access("world_one").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        fs::remove_file(path.join("world_one/linked.dat")).unwrap();
    }

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn level_storage_access_locks_saves_renames_backs_up_and_deletes() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-access-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let source = LevelStorageSource::new(&path, path.join("backups")).unwrap();
    let access = source.create_access("world_two").unwrap();
    assert_eq!(access.level_id(), "world_two");
    assert_eq!(
        access.get_dimension_path("minecraft:the_nether").unwrap(),
        path.join("world_two/dimensions/minecraft/the_nether")
    );

    let tag = crate::storage::nbt::Tag::Compound(vec![
        (
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        (
            "LevelName".to_string(),
            crate::storage::nbt::Tag::String("Old Name".to_string()),
        ),
    ]);
    access.save_level_data(&tag).unwrap();
    assert!(access.has_world_data());
    assert_eq!(access.read_level_data().unwrap(), tag);

    access.rename_level("  New Name  ").unwrap();
    let renamed = access.read_level_data().unwrap();
    let crate::storage::nbt::Tag::Compound(values) = renamed else {
        panic!("expected compound");
    };
    assert_eq!(
        values.iter().find(|(name, _)| name == "LevelName"),
        Some(&(
            "LevelName".to_string(),
            crate::storage::nbt::Tag::String("New Name".to_string())
        ))
    );

    fs::write(access.layout().root().join("kept.txt"), b"backup").unwrap();
    let backup = access.make_world_backup().unwrap();
    assert!(backup.join("level.dat").is_file());
    assert!(backup.join("kept.txt").is_file());
    assert!(!backup.join("session.lock").exists());

    access.delete_level().unwrap();
    assert!(!path.join("world_two").exists());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn derived_level_data_matches_java_delegation_semantics() {
    let world_data = super::WorldDataView {
        level_name: "Shared World".to_string(),
        game_type: super::LevelGameType::Creative,
        hardcore: true,
        allow_commands: true,
        difficulty: super::LevelDifficulty::Hard,
        difficulty_locked: true,
        seed: 12345,
    };
    let wrapped = super::ServerLevelDataView {
        respawn_data: super::LevelRespawnData {
            dimension: "minecraft:the_nether".to_string(),
            x: 8,
            y: 70,
            z: -4,
            angle: 90.0,
        },
        game_time: 24000,
        initialized: false,
        seed: 67890,
    };
    let mut derived = super::DerivedLevelData::new(world_data, wrapped);

    assert_eq!(derived.level_name(), "Shared World");
    assert_eq!(derived.game_type(), super::LevelGameType::Creative);
    assert!(derived.is_hardcore());
    assert!(derived.allow_commands());
    assert_eq!(derived.difficulty(), super::LevelDifficulty::Hard);
    assert!(derived.difficulty_locked());
    assert_eq!(derived.game_time(), 24000);
    assert!(!derived.initialized());
    assert_eq!(derived.world_seed(), 12345);
    assert_eq!(derived.dimension_seed(), 67890);

    derived.set_spawn(super::LevelRespawnData {
        dimension: "minecraft:overworld".to_string(),
        x: 0,
        y: 64,
        z: 0,
        angle: 0.0,
    });
    assert_eq!(derived.respawn_data().dimension, "minecraft:overworld");
    assert_eq!(derived.respawn_data().y, 64);

    derived.set_game_time(1);
    derived.set_game_type(super::LevelGameType::Survival);
    derived.set_initialized(true);
    assert_eq!(derived.game_time(), 24000);
    assert_eq!(derived.game_type(), super::LevelGameType::Creative);
    assert!(!derived.initialized());
}

#[test]
fn dimension_paths_match_26_1_2_identifier_storage_folder() {
    let layout = WorldLayout::new("world");
    assert_eq!(
        layout.dimension_path("minecraft:overworld").unwrap(),
        std::path::PathBuf::from("world/dimensions/minecraft/overworld")
    );
    assert_eq!(
        layout.dimension_path("minecraft:the_nether").unwrap(),
        std::path::PathBuf::from("world/dimensions/minecraft/the_nether")
    );
    assert_eq!(
        layout.dimension_path("minecraft:the_end").unwrap(),
        std::path::PathBuf::from("world/dimensions/minecraft/the_end")
    );
    assert_eq!(
        layout.dimension_path("custom:sky/islands").unwrap(),
        std::path::PathBuf::from("world/dimensions/custom/sky/islands")
    );
    assert!(layout.dimension_path("../escape").is_err());
    assert!(layout.dimension_path("Bad:overworld").is_err());
}

#[test]
fn session_lock_matches_vanilla_marker_and_enforces_exclusive_lock() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-session-lock-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    assert!(!layout.is_session_locked().unwrap());

    let lock = layout.acquire_session_lock().unwrap();
    assert_eq!(
        fs::read(layout.session_lock()).unwrap(),
        "\u{2603}".as_bytes()
    );
    assert!(layout.is_session_locked().unwrap());

    let err = layout.acquire_session_lock().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::WouldBlock);
    assert!(err.to_string().contains("already locked"));

    drop(lock);
    assert!(!layout.is_session_locked().unwrap());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn saves_level_dat_and_rotates_old_copy() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-dat-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let first = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(4790),
    )]);
    let second = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(4791),
    )]);

    layout.save_level_dat(&first).unwrap();
    assert_eq!(layout.load_level_dat().unwrap(), first);
    layout.save_level_dat(&second).unwrap();
    assert_eq!(layout.load_level_dat().unwrap(), second);
    assert!(layout.level_dat_old().is_file());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn level_dat_is_gzip_with_empty_root_name_like_vanilla_and_reads_legacy_uncompressed() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-dat-gzip-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    let layout = WorldLayout::new(&path);

    let data = crate::storage::nbt::Tag::Compound(vec![(
        "Data".to_string(),
        crate::storage::nbt::Tag::Compound(vec![(
            "LevelName".to_string(),
            crate::storage::nbt::Tag::String("hi".to_string()),
        )]),
    )]);
    layout.save_level_dat(&data).unwrap();

    // Java NbtIo.writeCompressed => gzip (magic 1f 8b) with an empty root name
    // whose payload is { "Data": <leveldata> }.
    let raw = fs::read(layout.level_dat()).unwrap();
    assert_eq!(&raw[..2], &[0x1f, 0x8b]);
    let (root_name, _) = crate::storage::nbt::read_gzip_named_tag(raw.as_slice()).unwrap();
    assert_eq!(root_name, "");
    assert_eq!(layout.load_level_dat().unwrap(), data);

    // Legacy uncompressed level.dat still loads (migration path).
    let mut legacy = Vec::new();
    crate::storage::nbt::write_named_tag(&mut legacy, "Data", &data).unwrap();
    fs::write(layout.level_dat(), &legacy).unwrap();
    assert_eq!(layout.load_level_dat().unwrap(), data);

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn saves_player_data_and_json_sidecars() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-player-storage-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let uuid = "00000000-0000-0000-0000-000000000001";
    let player = crate::storage::nbt::Tag::Compound(vec![(
        "Health".to_string(),
        crate::storage::nbt::Tag::Float(20.0),
    )]);
    layout.save_player_data(uuid, &player).unwrap();
    layout.save_advancements(uuid, "{\"done\":true}").unwrap();
    layout
        .save_stats(uuid, "{\"minecraft:custom\":{}}")
        .unwrap();

    assert_eq!(
        layout.load_player_data(uuid).unwrap(),
        super::tag_with_data_version(&player)
    );
    assert_eq!(
        layout.load_advancements(uuid).unwrap(),
        format!(
            "{{\"DataVersion\":{},\"done\":true}}",
            crate::storage::datafix::TARGET_DATA_VERSION
        )
    );
    assert_eq!(
        layout.load_stats(uuid).unwrap(),
        format!(
            "{{\"DataVersion\":{},\"minecraft:custom\":{{}}}}",
            crate::storage::datafix::TARGET_DATA_VERSION
        )
    );

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn json_sidecars_stamp_and_validate_data_versions() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-json-version-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let uuid = "00000000-0000-0000-0000-000000000005";
    layout
        .save_advancements(uuid, "{\"DataVersion\":1,\"advancements\":{}}")
        .unwrap();
    assert!(layout.load_advancements(uuid).unwrap().contains(&format!(
        "\"DataVersion\":{}",
        crate::storage::datafix::TARGET_DATA_VERSION
    )));

    layout
        .save_stats(uuid, "{\"stats\":{},\"DataVersion\":1}")
        .unwrap();
    assert!(layout.load_stats(uuid).unwrap().contains(&format!(
        "\"DataVersion\":{}",
        crate::storage::datafix::TARGET_DATA_VERSION
    )));

    fs::write(layout.advancements_file(uuid), "{\"advancements\":{}}").unwrap();
    let err = layout.load_advancements(uuid).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("missing DataVersion"));

    fs::write(
        layout.stats_file(uuid),
        format!(
            "{{\"stats\":{{}},\"DataVersion\":{}}}",
            crate::storage::datafix::TARGET_DATA_VERSION - 1
        ),
    )
    .unwrap();
    let err = layout.load_stats(uuid).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Unsupported world DataVersion"));

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn player_data_uses_dat_old_and_corrupt_backup_like_vanilla_storage() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-player-corrupt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let uuid = "00000000-0000-0000-0000-000000000002";
    let first = crate::storage::nbt::Tag::Compound(vec![(
        "Pos".to_string(),
        crate::storage::nbt::Tag::List(vec![
            crate::storage::nbt::Tag::Double(1.0),
            crate::storage::nbt::Tag::Double(80.0),
            crate::storage::nbt::Tag::Double(1.0),
        ]),
    )]);
    let second = crate::storage::nbt::Tag::Compound(vec![(
        "Pos".to_string(),
        crate::storage::nbt::Tag::List(vec![
            crate::storage::nbt::Tag::Double(2.0),
            crate::storage::nbt::Tag::Double(81.0),
            crate::storage::nbt::Tag::Double(2.0),
        ]),
    )]);

    layout.save_player_data(uuid, &first).unwrap();
    layout.save_player_data(uuid, &second).unwrap();
    assert_eq!(
        layout.load_player_data(uuid).unwrap(),
        super::tag_with_data_version(&second)
    );
    assert!(layout.player_data_old_file(uuid).is_file());

    fs::write(layout.player_data_file(uuid), b"corrupt playerdata").unwrap();
    assert_eq!(
        layout.load_player_data(uuid).unwrap(),
        super::tag_with_data_version(&first)
    );
    let corrupt_backups = fs::read_dir(layout.playerdata_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(&format!("{uuid}_corrupted_"))
        })
        .count();
    assert_eq!(corrupt_backups, 1);

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn player_data_storage_wraps_layout_save_load_backup_paths() {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "vibecraft-player-data-storage-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let storage = PlayerDataStorage::new(layout.clone());
    let uuid = "00000000-0000-0000-0000-000000000003";
    let player = crate::storage::nbt::Tag::Compound(vec![(
        "Score".to_string(),
        crate::storage::nbt::Tag::Int(5),
    )]);

    storage.save(uuid, &player).unwrap();
    assert_eq!(
        storage.load(uuid).unwrap(),
        super::tag_with_data_version(&player)
    );
    assert_eq!(
        storage.player_data_file(uuid),
        layout.player_data_file(uuid)
    );
    assert_eq!(
        storage.player_data_old_file(uuid),
        layout.player_data_old_file(uuid)
    );

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn saved_nbt_loaders_refuse_missing_or_unsupported_data_versions() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-saved-version-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let uuid = "00000000-0000-0000-0000-000000000004";

    let missing = crate::storage::nbt::Tag::Compound(vec![(
        "Health".to_string(),
        crate::storage::nbt::Tag::Float(20.0),
    )]);
    let mut bytes = Vec::new();
    crate::storage::nbt::write_gzip_named_tag(&mut bytes, "", &missing).unwrap();
    fs::create_dir_all(layout.playerdata_dir()).unwrap();
    fs::write(layout.player_data_file(uuid), bytes).unwrap();
    let err = layout.load_player_data(uuid).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("missing DataVersion"));

    let unsupported = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION - 1),
    )]);
    let mut bytes = Vec::new();
    crate::storage::nbt::write_named_tag(&mut bytes, "", &unsupported).unwrap();
    fs::create_dir_all(layout.data_dir()).unwrap();
    fs::write(layout.saved_data_file("scoreboard"), bytes).unwrap();
    let err = layout.load_scoreboard().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err
        .to_string()
        .contains("Unsupported scoreboard DataVersion"));

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn saves_vanilla_named_data_files() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-saved-data-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let tag = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(4790),
    )]);

    layout.save_scoreboard(&tag).unwrap();
    layout.save_raids("", &tag).unwrap();
    layout.save_map_data(0, &tag).unwrap();
    layout.save_forced_chunks(&tag).unwrap();
    layout.save_command_storage("minecraft", &tag).unwrap();
    layout.save_custom_bossbars(&tag).unwrap();
    layout.save_random_sequences(&tag).unwrap();

    assert_eq!(layout.load_scoreboard().unwrap(), tag);
    assert_eq!(layout.load_raids("").unwrap(), tag);
    assert_eq!(layout.load_map_data(0).unwrap(), tag);
    assert_eq!(layout.load_forced_chunks().unwrap(), tag);
    assert_eq!(layout.load_command_storage("minecraft").unwrap(), tag);
    assert_eq!(layout.load_custom_bossbars().unwrap(), tag);
    assert_eq!(layout.load_random_sequences().unwrap(), tag);
    assert!(layout.saved_data_file("scoreboard").is_file());
    assert!(layout.map_data_file(0).is_file());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn saves_entity_and_poi_region_files_separate_from_block_regions() {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "vibecraft-entity-poi-regions-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let pos = crate::storage::region::ChunkPos { x: 2, z: -3 };
    let entity_tag = crate::storage::entities::ChunkEntities {
        pos,
        entities: vec![crate::storage::entities::StoredEntity {
            uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            entity_type: "minecraft:pig".to_string(),
            data: crate::storage::nbt::Tag::Compound(vec![(
                "Health".to_string(),
                crate::storage::nbt::Tag::Float(10.0),
            )]),
        }],
    }
    .to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
    let poi_tag = crate::storage::poi::PoiSection::new(true).to_nbt();

    layout.save_entity_region_chunk(pos, &entity_tag).unwrap();
    layout.save_poi_region_chunk(pos, &poi_tag).unwrap();

    assert_eq!(
        layout.load_entity_region_chunk(pos).unwrap(),
        Some(entity_tag)
    );
    assert_eq!(layout.load_poi_region_chunk(pos).unwrap(), Some(poi_tag));
    assert!(layout.entities_dir().join("r.0.-1.mca").is_file());
    assert!(layout.poi_dir().join("r.0.-1.mca").is_file());
    assert!(!layout.region_dir().join("r.0.-1.mca").exists());

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn validates_world_relative_paths_and_rejects_symlinks() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-path-safety-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let layout = WorldLayout::new(&path);
    assert_eq!(
        layout
            .validate_relative_path(std::path::Path::new("data/scoreboard.dat"))
            .unwrap(),
        path.join("data/scoreboard.dat")
    );
    assert!(layout
        .validate_relative_path(std::path::Path::new("../outside.dat"))
        .is_err());
    assert!(layout
        .validate_relative_path(std::path::Path::new("/outside.dat"))
        .is_err());

    let normal = path.join("normal.dat");
    fs::write(&normal, b"ok").unwrap();
    assert!(layout.reject_symlink(&normal).is_ok());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let target = path.join("target.dat");
        let link = path.join("link.dat");
        fs::write(&target, b"target").unwrap();
        symlink(&target, &link).unwrap();
        assert!(layout.reject_symlink(&link).is_err());
    }

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn falls_back_to_level_dat_old_when_primary_is_corrupt() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-corrupt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let first = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(4790),
    )]);
    let second = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(4791),
    )]);

    layout.save_level_dat(&first).unwrap();
    layout.save_level_dat(&second).unwrap();
    fs::write(layout.level_dat(), b"corrupt").unwrap();

    assert_eq!(layout.load_level_dat_with_backup().unwrap(), first);

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn refuses_world_metadata_when_primary_and_backup_are_corrupt() {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "vibecraft-level-both-corrupt-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();

    let layout = WorldLayout::new(&path);
    fs::write(layout.level_dat(), b"corrupt primary").unwrap();
    fs::write(layout.level_dat_old(), b"corrupt backup").unwrap();

    let err = layout.load_level_dat_with_backup().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn checked_level_dat_refuses_incompatible_or_missing_data_version() {
    let mut path = std::env::temp_dir();
    path.push(format!("vibecraft-level-version-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);

    let layout = WorldLayout::new(&path);
    let valid = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
    )]);
    layout.save_level_dat(&valid).unwrap();
    assert_eq!(layout.load_level_dat_checked().unwrap(), valid);

    let invalid = crate::storage::nbt::Tag::Compound(vec![(
        "DataVersion".to_string(),
        crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION - 1),
    )]);
    layout.save_level_dat(&invalid).unwrap();
    let err = layout.load_level_dat_checked().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Unsupported world DataVersion"));

    let missing = crate::storage::nbt::Tag::Compound(vec![]);
    layout.save_level_dat(&missing).unwrap();
    let err = layout.load_level_dat_checked().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("missing DataVersion"));

    let _ = fs::remove_dir_all(&path);
}

#[test]
fn primary_level_data_round_trips_vanilla_level_dat_fields() {
    let data = super::PrimaryLevelData {
        data_version: crate::storage::datafix::TARGET_DATA_VERSION,
        level_data_version: 19133,
        version: super::LevelVersionInfo {
            id: crate::storage::datafix::TARGET_DATA_VERSION,
            name: "26.1.2".to_string(),
            series: "main".to_string(),
            snapshot: false,
        },
        level_name: "Round Trip".to_string(),
        spawn: super::LevelSpawnData {
            x: 12,
            y: 80,
            z: -9,
            angle: 45.0,
        },
        game_type: super::LevelGameType::Creative,
        difficulty: super::LevelDifficulty::Hard,
        day_time: 24000,
        time: 123456,
        generator_name: "minecraft:noise".to_string(),
        generator_settings: crate::storage::nbt::Tag::Compound(vec![(
            "seed".to_string(),
            crate::storage::nbt::Tag::Long(99),
        )]),
        allow_commands: true,
        hardcore: true,
        initialized: true,
        was_modded: true,
        data_packs: super::DataPackSelection {
            enabled: vec!["vanilla".to_string(), "file/example".to_string()],
            disabled: vec!["file/disabled".to_string()],
        },
        scheduled_events: crate::storage::nbt::Tag::List(vec![crate::storage::nbt::Tag::Compound(
            vec![(
                "Name".to_string(),
                crate::storage::nbt::Tag::String("minecraft:raid".to_string()),
            )],
        )]),
        server_brands: vec!["vanilla".to_string(), "vibecraft".to_string()],
        custom_boss_events: crate::storage::nbt::Tag::Compound(vec![(
            "minecraft:boss".to_string(),
            crate::storage::nbt::Tag::Compound(vec![]),
        )]),
        dragon_fight: crate::storage::nbt::Tag::Compound(vec![(
            "DragonKilled".to_string(),
            crate::storage::nbt::Tag::Byte(1),
        )]),
        scoreboard: crate::storage::nbt::Tag::Compound(vec![(
            "Objectives".to_string(),
            crate::storage::nbt::Tag::List(vec![]),
        )]),
        game_rules: crate::storage::nbt::Tag::Compound(vec![(
            "doDaylightCycle".to_string(),
            crate::storage::nbt::Tag::String("true".to_string()),
        )]),
    };

    let encoded = data.to_level_dat();
    let crate::storage::nbt::Tag::Compound(root) = &encoded else {
        panic!("expected level.dat root compound");
    };
    let Some(crate::storage::nbt::Tag::Compound(values)) = root
        .iter()
        .find(|(name, _)| name == "Data")
        .map(|(_, value)| value)
    else {
        panic!("expected Data compound");
    };
    assert!(values.iter().any(|(name, _)| name == "Version"));
    assert!(values.iter().any(|(name, _)| name == "DataPacks"));
    assert!(values.iter().any(|(name, _)| name == "ScheduledEvents"));
    assert!(values.iter().any(|(name, _)| name == "ServerBrands"));
    assert!(values.iter().any(|(name, _)| name == "CustomBossEvents"));
    assert!(values.iter().any(|(name, _)| name == "DragonFight"));
    assert!(values.iter().any(|(name, _)| name == "scoreboard"));
    assert!(values.iter().any(|(name, _)| name == "GameRules"));

    let decoded = super::PrimaryLevelData::from_level_dat(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn default_level_dat_round_trip_matches_vanilla_generated_field_shape() {
    let data = super::PrimaryLevelData {
        data_version: crate::storage::datafix::TARGET_DATA_VERSION,
        level_data_version: 19133,
        version: super::LevelVersionInfo {
            id: crate::storage::datafix::TARGET_DATA_VERSION,
            name: "26.1.2".to_string(),
            series: "main".to_string(),
            snapshot: false,
        },
        level_name: "New World".to_string(),
        spawn: super::LevelSpawnData {
            x: 0,
            y: 64,
            z: 0,
            angle: 0.0,
        },
        game_type: super::LevelGameType::Survival,
        difficulty: super::LevelDifficulty::Normal,
        day_time: 0,
        time: 0,
        generator_name: "minecraft:normal".to_string(),
        generator_settings: crate::storage::nbt::Tag::Compound(vec![]),
        allow_commands: false,
        hardcore: false,
        initialized: true,
        was_modded: false,
        data_packs: super::DataPackSelection {
            enabled: vec!["vanilla".to_string()],
            disabled: Vec::new(),
        },
        scheduled_events: crate::storage::nbt::Tag::List(vec![]),
        server_brands: vec!["vanilla".to_string()],
        custom_boss_events: crate::storage::nbt::Tag::Compound(vec![]),
        dragon_fight: crate::storage::nbt::Tag::Compound(vec![]),
        scoreboard: crate::storage::nbt::Tag::Compound(vec![]),
        game_rules: crate::storage::nbt::Tag::Compound(vec![]),
    };

    let encoded = data.to_level_dat();
    let crate::storage::nbt::Tag::Compound(root) = &encoded else {
        panic!("expected level.dat root compound");
    };
    let Some(crate::storage::nbt::Tag::Compound(values)) = root
        .iter()
        .find(|(name, _)| name == "Data")
        .map(|(_, value)| value)
    else {
        panic!("expected Data compound");
    };

    let expected_fields = [
        "DataVersion",
        "version",
        "Version",
        "LevelName",
        "SpawnX",
        "SpawnY",
        "SpawnZ",
        "SpawnAngle",
        "GameType",
        "Difficulty",
        "DayTime",
        "Time",
        "generatorName",
        "generatorSettings",
        "allowCommands",
        "hardcore",
        "initialized",
        "WasModded",
        "DataPacks",
        "ScheduledEvents",
        "ServerBrands",
        "CustomBossEvents",
        "DragonFight",
        "scoreboard",
        "GameRules",
    ];
    for field in expected_fields {
        assert!(
            values.iter().any(|(name, _)| name == field),
            "missing vanilla level.dat field {field}"
        );
    }

    let decoded = super::PrimaryLevelData::from_level_dat(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn parses_level_version_like_vanilla_summary_data() {
    let tag = crate::storage::nbt::Tag::Compound(vec![(
        "Data".to_string(),
        crate::storage::nbt::Tag::Compound(vec![
            ("version".to_string(), crate::storage::nbt::Tag::Int(19133)),
            (
                "DataVersion".to_string(),
                crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
            ),
            (
                "LastPlayed".to_string(),
                crate::storage::nbt::Tag::Long(123456789),
            ),
            (
                "Version".to_string(),
                crate::storage::nbt::Tag::Compound(vec![
                    (
                        "Name".to_string(),
                        crate::storage::nbt::Tag::String("26.1.2".to_string()),
                    ),
                    (
                        "Id".to_string(),
                        crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                    ),
                    (
                        "Series".to_string(),
                        crate::storage::nbt::Tag::String("main".to_string()),
                    ),
                    ("Snapshot".to_string(), crate::storage::nbt::Tag::Byte(0)),
                ]),
            ),
        ]),
    )]);

    let version = super::LevelVersion::parse_level_dat(&tag).unwrap();
    assert_eq!(version.level_data_version, 19133);
    assert_eq!(
        version.data_version,
        Some(crate::storage::datafix::TARGET_DATA_VERSION)
    );
    assert_eq!(version.last_played, 123456789);
    assert_eq!(version.minecraft_version_name, "26.1.2");
    assert_eq!(
        version.minecraft_version.id,
        crate::storage::datafix::TARGET_DATA_VERSION
    );
    assert_eq!(version.minecraft_version.series, "main");
    assert!(!version.snapshot);
}

#[test]
fn parses_legacy_level_version_defaults_without_version_compound() {
    let tag = crate::storage::nbt::Tag::Compound(vec![
        ("version".to_string(), crate::storage::nbt::Tag::Int(19132)),
        (
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
    ]);

    let version = super::LevelVersion::parse_level_dat(&tag).unwrap();
    assert_eq!(version.level_data_version, 19132);
    assert_eq!(version.minecraft_version_name, "26.1.2");
    assert_eq!(version.minecraft_version.series, "main");
    assert!(!version.snapshot);
}
