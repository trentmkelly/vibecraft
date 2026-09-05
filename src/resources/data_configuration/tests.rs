use super::*;
use crate::storage::nbt::Tag;

#[test]
fn lenient_fields_default_independently_in_json_and_nbt() {
    let registry = FeatureFlagRegistry::main_26_1_2().unwrap();
    let json = r#"{"DataPacks":{"Enabled":["custom"],"Disabled":[]},"enabled_features":["minecraft:trade_rebalance","missing"]}"#;
    let parsed = WorldDataConfiguration::from_json(json, &registry).unwrap();
    assert_eq!(parsed.data_packs.enabled, ["custom"]);
    assert_eq!(
        parsed.enabled_features,
        feature_flags::default_flags_26_1_2()
    );
    let malformed = Tag::Compound(vec![
        ("DataPacks".to_owned(), Tag::String("bad".to_owned())),
        (
            "enabled_features".to_owned(),
            Tag::List(vec![Tag::String("trade_rebalance".to_owned())]),
        ),
    ]);
    let parsed = WorldDataConfiguration::from_nbt(&malformed, &registry).unwrap();
    assert_eq!(parsed.data_packs, DataPackConfig::default_26_1_2());
    assert_eq!(
        parsed.enabled_features,
        FeatureFlagSet::of(&[feature_flags::TRADE_REBALANCE])
    );
    for json in [
        "{}",
        r#"{"DataPacks":null,"enabled_features":42}"#,
        r#"{"DataPacks":{"Enabled":[]},"enabled_features":[false]}"#,
    ] {
        assert_eq!(
            WorldDataConfiguration::from_json(json, &registry).unwrap(),
            WorldDataConfiguration::default_26_1_2()
        );
    }
    assert!(WorldDataConfiguration::from_nbt(&Tag::Int(0), &registry).is_err());
}

#[test]
fn optional_encoding_preserves_java_default_identity_boundary() {
    let registry = FeatureFlagRegistry::main_26_1_2().unwrap();
    let default = WorldDataConfiguration::default_26_1_2();
    assert_eq!(default.to_json(&registry).unwrap(), "{}");
    assert_eq!(default.to_nbt(&registry), Tag::Compound(vec![]));
    let explicit = WorldDataConfiguration {
        data_packs: DataPackConfig::new(["vanilla"], std::iter::empty::<&str>()),
        enabled_features: default.enabled_features,
    };
    assert_eq!(
        explicit.to_json(&registry).unwrap(),
        r#"{"DataPacks":{"Enabled":["vanilla"],"Disabled":[]}}"#
    );
    assert_eq!(
        WorldDataConfiguration::from_nbt(&explicit.to_nbt(&registry), &registry)
            .unwrap()
            .to_json(&registry)
            .unwrap(),
        explicit.to_json(&registry).unwrap()
    );
    let empty = WorldDataConfiguration {
        enabled_features: FeatureFlagSet::empty(),
        ..default
    };
    assert_eq!(
        empty.to_json(&registry).unwrap(),
        r#"{"enabled_features":[]}"#
    );
    assert_eq!(
        WorldDataConfiguration::from_nbt(&empty.to_nbt(&registry), &registry).unwrap(),
        empty
    );
}

#[test]
fn datapack_nbt_codec_requires_both_lists_and_preserves_order_and_duplicates() {
    let config = DataPackConfig::new(["a", "", "a", "b"], ["c"]);
    assert_eq!(DataPackConfig::from_nbt(&config.to_nbt()).unwrap(), config);
    assert!(DataPackConfig::from_nbt(&Tag::Compound(vec![(
        "Enabled".to_owned(),
        Tag::List(vec![])
    )]))
    .is_err());
    let bad = Tag::Compound(vec![
        ("Enabled".to_owned(), Tag::List(vec![Tag::Int(1)])),
        ("Disabled".to_owned(), Tag::List(vec![])),
    ]);
    assert!(DataPackConfig::from_nbt(&bad).is_err());
}

#[test]
fn primary_level_data_preserves_enabled_features_and_selected_packs() {
    use crate::storage::world::PrimaryLevelData;
    let registry = FeatureFlagRegistry::main_26_1_2().unwrap();
    let expected = WorldDataConfiguration {
        data_packs: DataPackConfig::new(["vanilla", "file/world"], ["file/disabled"]),
        enabled_features: FeatureFlagSet::of(&[
            feature_flags::VANILLA,
            feature_flags::MINECART_IMPROVEMENTS,
        ]),
    };
    let Tag::Compound(mut fields) = expected.to_nbt(&registry) else {
        panic!("compound")
    };
    fields.extend([
        (
            "DataVersion".to_owned(),
            Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        (
            "Version".to_owned(),
            Tag::Compound(vec![
                (
                    "Id".to_owned(),
                    Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                ),
                ("Name".to_owned(), Tag::String("26.1.2".to_owned())),
            ]),
        ),
    ]);
    let loaded = PrimaryLevelData::from_level_dat(&Tag::Compound(vec![(
        "Data".to_owned(),
        Tag::Compound(fields),
    )]))
    .unwrap();
    assert_eq!(loaded.data_configuration, expected);
    let reloaded = PrimaryLevelData::from_level_dat(&loaded.to_level_dat().unwrap()).unwrap();
    assert_eq!(reloaded.data_configuration, expected);
}
