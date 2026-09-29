//! `pack.mcmeta` parsing against Java's `PackFormat`, `PackMetadataSection`,
//! `FeatureFlagsMetadataSection`, `OverlayMetadataSection` and `ResourceFilterSection`.

use serde_json::json;

use super::metadata_parser::parse_pack_section;
use super::*;

/// Validates `pack` as the `pack` section of `pack.mcmeta`.
fn pack_section(pack: serde_json::Value) -> Result<PackFormatRange, String> {
    parse_pack_section(&pack).map(|(_, range)| range)
}

fn format(major: u32, minor: u32) -> PackFormat {
    PackFormat { major, minor }
}

#[test]
fn new_style_formats_default_the_missing_minor_like_the_bottom_and_top_codecs() {
    let range = pack_section(json!({
        "description": "x", "min_format": [101, 1], "max_format": 101
    }))
    .unwrap();
    assert_eq!(range.min, format(101, 1));
    // `TOP_CODEC` fills the missing max minor with Integer.MAX_VALUE.
    assert_eq!(range.max, format(101, i32::MAX as u32));
    assert_eq!(range.max.to_string(), "101.*");
    assert_eq!(range.min.to_string(), "101.1");

    let single =
        pack_section(json!({"description": "x", "min_format": 101, "max_format": 101})).unwrap();
    assert_eq!(single.min, format(101, 0));
}

#[test]
fn old_style_formats_require_matching_pack_format_and_supported_formats() {
    let range = pack_section(json!({
        "description": "x", "pack_format": 48, "supported_formats": [45, 48]
    }))
    .unwrap();
    assert_eq!((range.min, range.max), (format(45, 0), format(48, 0)));
    // A bare integer `supported_formats` is a one-value range.
    let single = pack_section(json!({
        "description": "x", "pack_format": 48, "supported_formats": 48
    }))
    .unwrap();
    assert_eq!((single.min, single.max), (format(48, 0), format(48, 0)));
    // The `{min_inclusive, max_inclusive}` object form is accepted too.
    let object = pack_section(json!({
        "description": "x", "pack_format": 48,
        "supported_formats": {"min_inclusive": 45, "max_inclusive": 48}
    }))
    .unwrap();
    assert_eq!((object.min, object.max), (format(45, 0), format(48, 0)));
}

#[test]
fn invalid_format_declarations_report_the_java_messages() {
    let error = |pack: serde_json::Value| pack_section(pack).unwrap_err();
    assert_eq!(
        error(json!({"description": "x", "min_format": [101, 1]})),
        "Pack missing field, must declare both min_format and max_format"
    );
    assert_eq!(
        error(json!({"description": "x", "min_format": [101, 5], "max_format": [101, 2]})),
        "Pack min_format (101.5) is greater than max_format (101.2)"
    );
    assert_eq!(
        error(json!({"description": "x", "pack_format": 101})),
        "Pack declares support for version newer than 81, but is missing mandatory fields min_format and max_format"
    );
    assert_eq!(
        error(json!({"description": "x"})),
        "Pack could not be parsed, missing format version information"
    );
    assert_eq!(
        error(json!({
            "description": "x", "min_format": [101, 0], "max_format": 101,
            "supported_formats": [101, 101]
        })),
        "Pack key supported_formats is deprecated starting from pack format 82. Remove supported_formats from your pack.mcmeta."
    );
    assert_eq!(
        error(json!({"description": "x", "min_format": 70, "max_format": 101})),
        "Pack declares support for format 70, but game versions supporting formats 17 to 81 require a supported_formats field. Add \"supported_formats\": [70, 81] or require a version greater or equal to 82.0."
    );
    assert_eq!(
        error(json!({"description": "x", "supported_formats": [45, 48]})),
        "Pack declares support for formats up to 81, but game versions supporting formats 17 to 81 require a pack_format field. Add \"pack_format\": 45 or require a version greater or equal to 82.0."
    );
    assert_eq!(
        error(json!({"description": "x", "pack_format": 10, "supported_formats": [10, 12]})),
        "Multi-version packs cannot support minimum version of less than 15, since this will leave versions in range unable to load pack."
    );
    assert_eq!(
        error(json!({"description": "x", "pack_format": 60, "supported_formats": [45, 48]})),
        "Pack declared support for versions 45 to 48 but declared main format is 60"
    );
    assert_eq!(
        error(json!({"description": "x", "supported_formats": [5, 1], "pack_format": 3})),
        "min_inclusive must be less than or equal to max_inclusive"
    );
    assert!(pack_section(json!({"min_format": 101, "max_format": 101})).is_err());
}

#[test]
fn unvalidatable_formats_fall_back_to_the_description_and_flag_the_pack_unknown() {
    // A pre-minor pack that declares only `pack_format` is a valid one-value range...
    let old = parse_pack_metadata(r#"{"pack":{"description":"Legacy","pack_format":3}}"#).unwrap();
    assert_eq!(old.compatibility, PackCompatibility::TooOld);
    // ...but a new-style major without min/max_format cannot be validated.
    let metadata =
        parse_pack_metadata(r#"{"pack":{"description":"Legacy","pack_format":101}}"#).unwrap();
    assert_eq!(metadata.description, "Legacy");
    assert_eq!(metadata.compatibility, PackCompatibility::Unknown);
    assert!(!metadata.compatibility.is_compatible());
    // Without a description even the fallback fails and the pack is dropped.
    assert!(parse_pack_metadata(r#"{"pack":{"pack_format":3}}"#).is_err());
    assert!(parse_pack_metadata("not json").is_err());
    assert!(parse_pack_metadata("[]").is_err());
}

#[test]
fn compatibility_follows_the_declared_range_against_101_1() {
    let compat = |min: &str, max: &str| {
        parse_pack_metadata(&format!(
            r#"{{"pack":{{"description":"x","min_format":{min},"max_format":{max}}}}}"#
        ))
        .unwrap()
        .compatibility
    };
    assert_eq!(compat("[101, 1]", "101"), PackCompatibility::Compatible);
    assert_eq!(compat("101", "101"), PackCompatibility::Compatible);
    assert_eq!(compat("[101, 2]", "102"), PackCompatibility::TooNew);
    assert_eq!(compat("100", "[101, 0]"), PackCompatibility::TooOld);
}

#[test]
fn feature_sections_resolve_known_flags_and_reject_unknown_ones() {
    let ok = parse_pack_metadata(
        r#"{"pack":{"description":"x","min_format":101,"max_format":101},
            "features":{"enabled":["minecraft:trade_rebalance","redstone_experiments"]}}"#,
    )
    .unwrap();
    assert!(ok
        .requested_features
        .contains(feature_flags::TRADE_REBALANCE));
    assert!(ok
        .requested_features
        .contains(feature_flags::REDSTONE_EXPERIMENTS));
    assert!(!ok.requested_features.contains(feature_flags::VANILLA));
    // An unknown flag id makes `getMetadataSection(FeatureFlagsMetadataSection.TYPE)`
    // throw, so the pack is dropped.
    assert!(parse_pack_metadata(
        r#"{"pack":{"description":"x","min_format":101,"max_format":101},
            "features":{"enabled":["example:nope"]}}"#
    )
    .is_err());
}

#[test]
fn overlays_apply_for_the_current_format_and_validate_their_declarations() {
    let with_overlays = |entries: &str| {
        parse_pack_metadata(&format!(
            r#"{{"pack":{{"description":"x","min_format":[101,1],"max_format":101}},
                "overlays":{{"entries":{entries}}}}}"#
        ))
    };
    let metadata = with_overlays(
        r#"[{"directory":"now","min_format":101,"max_format":101},
            {"directory":"later","min_format":[101,2],"max_format":102},
            {"directory":"earlier","min_format":90,"max_format":100},
            {"directory":"undeclared"}]"#,
    )
    .unwrap();
    assert_eq!(metadata.overlays, vec!["now"]);
    assert!(
        with_overlays(r#"[{"directory":"a/b","min_format":101,"max_format":101}]"#)
            .unwrap_err()
            .contains("a/b is not accepted directory name")
    );
    assert!(with_overlays(r#"[{"directory":"x","min_format":101}]"#)
        .unwrap_err()
        .contains("Overlay \"x\" missing field, must declare both min_format and max_format"));
}

#[test]
fn filter_sections_use_partial_regex_matches_and_default_to_everything() {
    let filter = parse_filter_section(
        r#"{"pack":{},"filter":{"block":[{"namespace":"^mine","path":"tags/.*"},{"path":"x"}]}}"#,
    )
    .unwrap();
    assert!(filter.is_namespace_filtered("minecraft"));
    // The second pattern has no namespace part, which matches every namespace.
    assert!(filter.is_namespace_filtered("example"));
    assert!(filter.is_path_filtered("tags/item/logs.json"));
    assert!(filter.is_path_filtered("has_x_inside"));
    assert!(!filter.is_path_filtered("loot_table/a.json"));
    assert!(parse_filter_section(r#"{"pack":{}}"#).is_none());
    assert!(parse_filter_section(r#"{"filter":{"block":[{"path":"("}]}}"#).is_none());
}

#[test]
fn feature_names_come_out_in_java_hash_set_order() {
    // `FeatureFlagRegistry.toNames` builds a `HashSet<Identifier>`; its iteration
    // order (bucket of `Identifier.hashCode`) is what clients and `level.dat` see.
    let registry = FeatureFlagRegistry::main_26_1_2().unwrap();
    let all = FeatureFlagSet::of(&[
        feature_flags::VANILLA,
        feature_flags::TRADE_REBALANCE,
        feature_flags::REDSTONE_EXPERIMENTS,
        feature_flags::MINECART_IMPROVEMENTS,
    ]);
    let names: Vec<String> = registry
        .to_names(all)
        .iter()
        .map(|name| name.to_string())
        .collect();
    assert_eq!(
        names,
        [
            "minecraft:redstone_experiments",
            "minecraft:vanilla",
            "minecraft:trade_rebalance",
            "minecraft:minecart_improvements"
        ]
    );
}
