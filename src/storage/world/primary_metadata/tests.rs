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
