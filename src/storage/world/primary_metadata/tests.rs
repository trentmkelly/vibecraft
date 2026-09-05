use super::*;

fn loaded_metadata() -> PrimaryLevelData {
    PrimaryLevelData::from_level_dat(&Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(vec![
            ("DataVersion".to_owned(), Tag::Int(123)),
            ("version".to_owned(), Tag::Int(19132)),
            (
                "Version".to_owned(),
                Tag::Compound(vec![
                    ("Id".to_owned(), Tag::Int(123)),
                    ("Name".to_owned(), Tag::String("old".to_owned())),
                    ("Series".to_owned(), Tag::String("other".to_owned())),
                    ("Snapshot".to_owned(), Tag::Byte(1)),
                ]),
            ),
            (
                "ServerBrands".to_owned(),
                Tag::List(vec![
                    Tag::String("first".to_owned()),
                    Tag::String("second".to_owned()),
                    Tag::String("first".to_owned()),
                    Tag::Int(3),
                ]),
            ),
        ]),
    )]))
    .unwrap()
}

fn saved_fields(tag: &Tag) -> &[(String, Tag)] {
    let Tag::Compound(root) = tag else {
        panic!("root compound")
    };
    let Tag::Compound(fields) = &root.iter().find(|(key, _)| key == "Data").unwrap().1 else {
        panic!("Data compound")
    };
    fields
}

#[test]
fn every_save_stamps_current_version_without_mutating_loaded_metadata() {
    let loaded = loaded_metadata();
    let saved = loaded.to_level_dat_at(1_234_567).unwrap();
    let fields = saved_fields(&saved);
    assert_eq!(compound_i64(fields, "LastPlayed"), Some(1_234_567));
    let reloaded = PrimaryLevelData::from_level_dat(&saved).unwrap();
    assert_eq!(
        reloaded.data_version,
        crate::storage::datafix::TARGET_DATA_VERSION
    );
    assert_eq!(reloaded.level_data_version, 19133);
    assert_eq!(
        reloaded.version,
        LevelVersionInfo {
            id: crate::storage::datafix::TARGET_DATA_VERSION,
            name: CURRENT_VERSION_NAME.to_owned(),
            series: CURRENT_VERSION_SERIES.to_owned(),
            snapshot: false,
        }
    );
    assert_eq!(loaded.data_version, 123);
    assert_eq!(loaded.version.name, "old");
    assert_eq!(
        compound_i64(
            saved_fields(&loaded.to_level_dat_at(-1).unwrap()),
            "LastPlayed"
        ),
        Some(-1)
    );
}

#[test]
fn public_save_uses_current_epoch_milliseconds() {
    let loaded = loaded_metadata();
    let before = chrono::Utc::now().timestamp_millis();
    let saved = loaded.to_level_dat().unwrap();
    let after = chrono::Utc::now().timestamp_millis();
    let timestamp = compound_i64(saved_fields(&saved), "LastPlayed").unwrap();
    assert!((before..=after).contains(&timestamp));
}

#[test]
fn brands_are_unique_ordered_snapshots_and_modded_status_is_sticky() {
    let mut loaded = loaded_metadata();
    assert_eq!(loaded.server_brands, ["first", "second"]);
    let snapshot = loaded.known_server_brands();
    loaded.set_modded_info("first", false);
    assert!(!loaded.was_modded);
    loaded.set_modded_info("third", true);
    loaded.set_modded_info("second", false);
    assert!(loaded.was_modded);
    assert_eq!(snapshot, ["first", "second"]);
    assert_eq!(loaded.known_server_brands(), ["first", "second", "third"]);
    loaded.server_brands.push("third".to_owned());
    let reloaded = PrimaryLevelData::from_level_dat(&loaded.to_level_dat_at(0).unwrap()).unwrap();
    assert_eq!(reloaded.server_brands, ["first", "second", "third"]);
    assert!(reloaded.was_modded);
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn save_metadata_contract_matches_java_source() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/PrimaryLevelData.java");
    for fragment in [
        "writeVersionTag(tag)",
        "NbtUtils.addCurrentDataVersion(tag)",
        "tag.putInt(\"version\", 19133)",
        "tag.putLong(\"LastPlayed\", Util.getEpochMillis())",
        "SharedConstants.getCurrentVersion().name()",
        "SharedConstants.getCurrentVersion().dataVersion().version()",
        "Collectors.toCollection(Sets::newLinkedHashSet)",
        "this.knownServerBrands.add(serverBrand)",
        "this.wasModded |= isModded",
    ] {
        assert!(JAVA.contains(fragment), "missing Java contract: {fragment}");
    }
}

#[test]
fn optional_uuid_falls_back_or_overrides_without_mutating_loaded_identity() {
    use crate::network::codec::Uuid;
    let stored = Uuid([0xa5; 16]);
    let supplied = Uuid([0x5a; 16]);
    let mut level = loaded_metadata();
    assert!(level.singleplayer_uuid.is_none());
    let without = level.to_level_dat_at(0).unwrap();
    assert!(!saved_fields(&without)
        .iter()
        .any(|(key, _)| key == "singleplayer_uuid"));
    level.singleplayer_uuid = Some(stored);
    for (override_uuid, expected) in [(None, stored), (Some(supplied), supplied)] {
        let saved = level.to_level_dat_with_player_uuid(override_uuid).unwrap();
        assert_eq!(
            PrimaryLevelData::from_level_dat(&saved)
                .unwrap()
                .singleplayer_uuid,
            Some(expected)
        );
        assert_eq!(level.singleplayer_uuid, Some(stored));
    }
    level.singleplayer_uuid = None;
    let saved = level.to_level_dat_with_player_uuid(Some(supplied)).unwrap();
    assert_eq!(
        PrimaryLevelData::from_level_dat(&saved)
            .unwrap()
            .singleplayer_uuid,
        Some(supplied)
    );
}

#[test]
fn malformed_uuid_defaults_and_removed_features_round_trip_as_a_set() {
    let mut data = saved_fields(&loaded_metadata().to_level_dat_at(0).unwrap()).to_vec();
    assert!(!data.iter().any(|(key, _)| key == "removed_features"));
    data.push((
        "singleplayer_uuid".to_owned(),
        Tag::String("invalid".to_owned()),
    ));
    data.push((
        "removed_features".to_owned(),
        Tag::List(vec![
            Tag::String("old:a".to_owned()),
            Tag::String("old:b".to_owned()),
            Tag::String("old:a".to_owned()),
            Tag::Byte(1),
        ]),
    ));
    let loaded = PrimaryLevelData::from_level_dat(&Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(data),
    )]))
    .unwrap();
    assert!(loaded.singleplayer_uuid.is_none());
    assert_eq!(
        loaded.removed_features,
        ["old:a".to_owned(), "old:b".to_owned()]
            .into_iter()
            .collect()
    );
    let saved = loaded.to_level_dat_at(0).unwrap();
    let reloaded = PrimaryLevelData::from_level_dat(&saved).unwrap();
    assert_eq!(reloaded.removed_features, loaded.removed_features);
    assert_eq!(
        compound_string_list(saved_fields(&saved), "removed_features").len(),
        2
    );
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn optional_world_metadata_matches_java_codec_and_override_contract() {
    const JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/level/storage/PrimaryLevelData.java");
    for fragment in [
        "input.get(\"singleplayer_uuid\").flatMap(UUIDUtil.CODEC::parse).result().orElse(null)",
        "if (singlePlayerUUID == null)",
        "singlePlayerUUID = this.singlePlayerUUID",
        "if (!this.removedFeatureFlags.isEmpty())",
        "tag.put(\"removed_features\", stringCollectionToTag(this.removedFeatureFlags))",
    ] {
        assert!(JAVA.contains(fragment), "missing Java contract: {fragment}");
    }
    const UUID_JAVA: &str = vibecraft_java_source!("/net/minecraft/core/UUIDUtil.java");
    assert!(UUID_JAVA.contains("Util.fixedSize(list, 4).map(UUIDUtil::uuidFromIntArray)"));
}
