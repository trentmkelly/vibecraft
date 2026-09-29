//! `worldgen/configured_feature` and `worldgen/placed_feature`: every vendored file
//! decodes with the typed Java codecs and re-encodes to semantically equal JSON,
//! and the registries resolve every holder reference between them.
//!
//! `tools/vanilla_data_verified_features.txt` lists the checklist paths this
//! verifies; `tools/tick_vanilla_data_items.py` ticks exactly those lines of
//! `CHECKLIST_VANILLA_DATA_RESOURCES.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::{builtin, registries};
use crate::registry::Identifier;
use crate::registry_pipeline::codec::{Codec, CodecContext};
use crate::registry_pipeline::worldgen_feature::{
    configured_feature, json_semantically_equal, placed_feature, tag_to_json,
};

const CONFIGURED_DIR: &str = "worldgen/configured_feature";
const PLACED_DIR: &str = "worldgen/placed_feature";

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Extension-less file names of the vendored directory `dir`, sorted.
fn vendored(dir: &str) -> Vec<String> {
    let base = manifest_path("vanilla-data/data/minecraft").join(dir);
    let mut names: Vec<String> = fs::read_dir(&base)
        .unwrap_or_else(|err| panic!("{}: {err}", base.display()))
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            path.file_stem()
                .expect("stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

fn loading_registries() -> BTreeSet<Identifier> {
    ["minecraft:worldgen/configured_feature", "minecraft:worldgen/placed_feature"]
        .iter()
        .map(|id| Identifier::parse(id).expect("registry id"))
        .collect()
}

fn decode(codec: &Codec, json: &Value) -> Result<Value, String> {
    let loading = loading_registries();
    let ctx = CodecContext::new(builtin(), &loading);
    codec.parse(json, &ctx).map(|tag| tag_to_json(&tag))
}

/// Decodes and re-encodes the vendored file, returning the failure description.
fn round_trip(dir: &str, name: &str, codec: &Codec) -> Result<(), String> {
    let path = manifest_path("vanilla-data/data/minecraft")
        .join(dir)
        .join(format!("{name}.json"));
    let text = fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
    let source: Value = serde_json::from_str(&text).map_err(|err| format!("{name}: {err}"))?;
    let encoded = decode(codec, &source).map_err(|err| format!("{name}: {err}"))?;
    if json_semantically_equal(&source, &encoded) {
        Ok(())
    } else {
        Err(format!("{name}: re-encoded JSON differs\n{encoded}\nvs\n{source}"))
    }
}

/// Checklist paths of the files that decode and round-trip.
fn verified_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut problems = Vec::new();
    for (dir, codec) in [
        (CONFIGURED_DIR, configured_feature()),
        (PLACED_DIR, placed_feature()),
    ] {
        for name in vendored(dir) {
            match round_trip(dir, &name, &codec) {
                Ok(()) => {
                    paths.insert(format!(
                        "decompiled-server-26.1.2/data/minecraft/{dir}/{name}.json"
                    ));
                }
                Err(problem) => problems.push(problem),
            }
        }
    }
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
    paths
}

#[test]
fn every_vendored_feature_file_round_trips_through_the_typed_codecs() {
    let paths = verified_paths();
    assert_eq!(paths.len(), 221 + 258);
}

#[test]
fn the_feature_registries_load_every_vendored_file_with_resolved_references() {
    // `registries()` fails the whole load on unbound references, so this also proves
    // every placed feature resolves its configured feature, every biome its placed
    // features and every selector its sub-features.
    for (key, dir) in [
        ("minecraft:worldgen/configured_feature", CONFIGURED_DIR),
        ("minecraft:worldgen/placed_feature", PLACED_DIR),
    ] {
        let registry = registries()
            .lookup(&Identifier::parse(key).expect("registry id"))
            .unwrap_or_else(|| panic!("{key} is not loaded"));
        let loaded: BTreeSet<String> = registry
            .elements()
            .iter()
            .map(|element| element.key.path().to_string())
            .collect();
        assert_eq!(loaded, vendored(dir).into_iter().collect(), "{key}");
    }
}

#[test]
fn verified_feature_list_matches_and_covers_every_ticked_row() {
    let paths = verified_paths();
    let list = manifest_path("tools/vanilla_data_verified_features.txt");
    let text: String = paths.iter().map(|path| format!("{path}\n")).collect();
    if std::env::var_os("VIBECRAFT_WRITE_VERIFIED_PATHS").is_some() {
        fs::write(&list, &text).unwrap_or_else(|err| panic!("write list: {err}"));
    }
    let committed = fs::read_to_string(&list).unwrap_or_else(|err| panic!("read list: {err}"));
    assert_eq!(committed, text, "regenerate with VIBECRAFT_WRITE_VERIFIED_PATHS=1");

    let checklist = fs::read_to_string(manifest_path("CHECKLIST_VANILLA_DATA_RESOURCES.md"))
        .unwrap_or_else(|err| panic!("read checklist: {err}"));
    for line in checklist
        .lines()
        .filter(|line| line.starts_with("- [x] Audit vanilla data resource `"))
    {
        let path = line.split('`').nth(1).unwrap_or_default();
        let is_feature = [CONFIGURED_DIR, PLACED_DIR].iter().any(|dir| {
            path.starts_with(&format!("decompiled-server-26.1.2/data/minecraft/{dir}/"))
        });
        if is_feature {
            assert!(paths.contains(path), "ticked but not verified: {path}");
        }
    }
}

// ---------------------------------------------------------------------------
// Codec strictness
// ---------------------------------------------------------------------------

fn configured(json: Value) -> Result<Value, String> {
    decode(&configured_feature(), &json)
}

fn placed(json: Value) -> Result<Value, String> {
    decode(&placed_feature(), &json)
}

#[test]
fn unknown_feature_and_type_ids_are_rejected() {
    let error = configured(json!({"type": "minecraft:nope", "config": {}})).unwrap_err();
    assert!(error.contains("Unknown registry key"), "{error}");
    let error = placed(json!({
        "feature": "minecraft:oak",
        "placement": [{"type": "minecraft:nope"}]
    }))
    .unwrap_err();
    assert!(error.contains("Unknown registry key"), "{error}");
}

#[test]
fn missing_required_fields_are_rejected_and_extra_fields_ignored() {
    let error = configured(json!({"type": "minecraft:kelp"})).unwrap_err();
    assert!(error.starts_with("No key config"), "{error}");
    let encoded = configured(json!({"type": "minecraft:kelp", "config": {}, "extra": 1}))
        .expect("unknown fields are ignored like RecordCodecBuilder");
    assert_eq!(encoded, json!({"type": "minecraft:kelp", "config": {}}));
}

#[test]
fn block_states_reject_unknown_blocks_and_default_missing_properties() {
    let config = |state: Value| json!({"type": "minecraft:iceberg", "config": {"state": state}});
    let error = configured(config(json!({"Name": "minecraft:nope"}))).unwrap_err();
    assert!(error.contains("Unknown registry key"), "{error}");
    // Lenient properties: unknown names and invalid values fall back to the default,
    // and every property is written back.
    let encoded = configured(config(
        json!({"Name": "minecraft:oak_log", "Properties": {"axis": "q", "bogus": "1"}}),
    ))
    .expect("lenient properties");
    assert_eq!(
        encoded["config"]["state"],
        json!({"Name": "minecraft:oak_log", "Properties": {"axis": "y"}})
    );
}

#[test]
fn value_providers_enforce_their_bounds() {
    let count = |count: Value| json!({
        "feature": "minecraft:oak",
        "placement": [{"type": "minecraft:count", "count": count}]
    });
    assert!(placed(count(json!(4096))).is_ok());
    let error = placed(count(json!(4097))).unwrap_err();
    assert!(error.contains("Value provider too high: 4096"), "{error}");
    let error =
        placed(count(json!({"type": "minecraft:uniform", "min_inclusive": 5, "max_inclusive": 1})))
            .unwrap_err();
    assert!(error.contains("Max must be at least min"), "{error}");
}

#[test]
fn height_providers_collapse_constants_and_validate_anchors() {
    let range = |height: Value| json!({
        "feature": "minecraft:oak",
        "placement": [{"type": "minecraft:height_range", "height": height}]
    });
    let encoded = placed(range(
        json!({"type": "minecraft:constant", "value": {"absolute": 5}}),
    ))
    .expect("constant height");
    assert_eq!(encoded["placement"][0]["height"], json!({"absolute": 5}));
    let error = placed(range(json!({"absolute": 5000}))).unwrap_err();
    assert!(error.contains("outside of range"), "{error}");
    let error = placed(range(json!({"absolute": 1, "below_top": 2}))).unwrap_err();
    assert!(error.contains("Both alternatives"), "{error}");
}

#[test]
fn placed_features_accept_inline_configured_features() {
    let encoded = placed(json!({
        "feature": {"type": "minecraft:kelp", "config": {}},
        "placement": [{"type": "minecraft:in_square"}, {"type": "minecraft:biome"}]
    }))
    .expect("inline feature");
    assert_eq!(encoded["feature"], json!({"type": "minecraft:kelp", "config": {}}));
}

#[test]
fn rule_tests_dispatch_on_predicate_type() {
    let ore = |target: Value| json!({
        "type": "minecraft:ore",
        "config": {
            "targets": [{"target": target, "state": {"Name": "minecraft:stone"}}],
            "size": 4,
            "discard_chance_on_air_exposure": 0.0
        }
    });
    assert!(configured(ore(json!({
        "predicate_type": "minecraft:tag_match", "tag": "minecraft:stone_ore_replaceables"
    })))
    .is_ok());
    let error = configured(ore(json!({"predicate_type": "minecraft:nope"}))).unwrap_err();
    assert!(error.contains("Unknown registry key"), "{error}");
}

#[test]
fn multiface_growth_only_accepts_spreadeable_blocks() {
    let growth = |block: &str| json!({
        "type": "minecraft:multiface_growth",
        "config": {"block": block, "can_be_placed_on": ["minecraft:stone"]}
    });
    let encoded = configured(growth("minecraft:sculk_vein")).expect("sculk vein");
    assert_eq!(encoded["config"]["block"], json!("minecraft:sculk_vein"));
    // `orElse(glow lichen)`: an invalid block silently falls back to the default.
    let fallback = configured(growth("minecraft:stone")).expect("orElse");
    assert_eq!(fallback["config"]["block"], json!("minecraft:glow_lichen"));
}

#[test]
fn simple_random_selector_requires_a_non_empty_feature_list() {
    let selector = |features: Value| json!({
        "type": "minecraft:simple_random_selector", "config": {"features": features}
    });
    let error = configured(selector(json!([]))).unwrap_err();
    assert!(error.contains("List must have contents"), "{error}");
    let error = configured(selector(json!(["minecraft:oak", {
        "feature": {"type": "minecraft:kelp", "config": {}}, "placement": []
    }])))
    .unwrap_err();
    assert!(error.contains("Mixed type list"), "{error}");
}
