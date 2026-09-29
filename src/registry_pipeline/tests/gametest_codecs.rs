//! `TestEnvironmentDefinition` and `GameTestInstance` codecs.

use std::collections::BTreeSet;

use serde_json::json;

use super::builtin;
use crate::registry_pipeline::codec::CodecContext;
use crate::registry_pipeline::registry_data::element_codecs;
use crate::storage::nbt::Tag;

/// Decodes `value` with the direct codec of `registry`.
fn parse(registry: &str, value: serde_json::Value) -> Result<Tag, String> {
    let codecs = element_codecs(registry).unwrap_or_else(|| panic!("no codecs for {registry}"));
    let loading = BTreeSet::new();
    codecs
        .direct
        .parse(&value, &CodecContext::new(builtin(), &loading))
}

fn field<'a>(tag: &'a Tag, name: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value)),
        _ => None,
    }
}

fn names(tag: &Tag) -> Vec<&str> {
    match tag {
        Tag::Compound(fields) => fields.iter().map(|(key, _)| key.as_str()).collect(),
        _ => Vec::new(),
    }
}

#[test]
fn function_instances_encode_in_java_field_order_and_omit_default_values() {
    let tag = parse(
        "minecraft:test_instance",
        json!({
            "structure": "minecraft:empty",
            "max_ticks": 5,
            "environment": "minecraft:default",
            "type": "minecraft:function",
            "function": "minecraft:always_pass",
            "setup_ticks": 0,
            "required": true,
            "rotation": "180",
            "max_attempts": 3
        }),
    )
    .unwrap();
    // `type`, the function key, then `TestData` in declaration order; fields equal to
    // their default (`setup_ticks: 0`, `required: true`) are not encoded.
    assert_eq!(
        names(&tag),
        vec![
            "type",
            "function",
            "environment",
            "structure",
            "max_ticks",
            "rotation",
            "max_attempts"
        ]
    );
    assert_eq!(field(&tag, "max_ticks"), Some(&Tag::Int(5)));
    assert_eq!(
        field(&tag, "rotation"),
        Some(&Tag::String("180".to_string()))
    );
}

#[test]
fn test_data_validates_ranges_and_types() {
    let base = |patch: serde_json::Value| {
        let mut value = json!({
            "type": "minecraft:block_based",
            "environment": "minecraft:default",
            "structure": "minecraft:empty",
            "max_ticks": 1
        });
        for (key, patched) in patch.as_object().unwrap_or_else(|| panic!("object")) {
            value[key] = patched.clone();
        }
        parse("minecraft:test_instance", value)
    };
    assert!(base(json!({})).is_ok());
    assert!(base(json!({"max_ticks": 0})).is_err());
    assert!(base(json!({"setup_ticks": -1})).is_err());
    assert!(base(json!({"padding": 129})).is_err());
    assert!(base(json!({"padding": 128})).is_ok());
    assert!(base(json!({"rotation": "sideways"})).is_err());
    assert!(base(json!({"required": "yes"})).is_err());
    assert!(base(json!({"max_attempts": 0})).is_err());
    let unknown = parse("minecraft:test_instance", json!({"type": "minecraft:nope"})).unwrap_err();
    assert!(unknown.contains("test_instance_type"), "{unknown}");
    assert!(parse(
        "minecraft:test_instance",
        json!({"type": "minecraft:function"})
    )
    .is_err());
}

#[test]
fn environments_dispatch_on_type_and_nest_inline_definitions() {
    let tag = parse(
        "minecraft:test_environment",
        json!({
            "type": "minecraft:all_of",
            "definitions": [
                "minecraft:default",
                {"type": "minecraft:weather", "weather": "thunder"},
                {"type": "minecraft:function", "setup": "example:up"},
                {"type": "minecraft:clock_time", "clock": "minecraft:overworld", "time": 6000},
                {"type": "minecraft:timeline_attributes", "timelines": ["minecraft:day"]},
                {"type": "minecraft:game_rules", "rules": {
                    "minecraft:keep_inventory": true,
                    "max_entity_cramming": 5
                }}
            ]
        }),
    );
    // Bare game rule names are not identifiers in the registry codec's namespace.
    assert!(tag.is_ok(), "{tag:?}");
    let Some(Tag::List(definitions)) =
        field(&tag.unwrap_or_else(|e| panic!("{e}")), "definitions").cloned()
    else {
        panic!("definitions");
    };
    assert_eq!(definitions[0], Tag::String("minecraft:default".to_string()));
    let rules = field(&definitions[5], "rules").cloned().unwrap_or(Tag::End);
    assert_eq!(
        field(&rules, "minecraft:keep_inventory"),
        Some(&Tag::Byte(1))
    );
    assert_eq!(
        field(&rules, "minecraft:max_entity_cramming"),
        Some(&Tag::Int(5))
    );
}

#[test]
fn environment_fields_are_validated() {
    let bad = |value: serde_json::Value| parse("minecraft:test_environment", value).is_err();
    assert!(bad(json!({"type": "minecraft:weather", "weather": "hail"})));
    assert!(bad(
        json!({"type": "minecraft:clock_time", "clock": "minecraft:overworld", "time": -1})
    ));
    assert!(bad(
        json!({"type": "minecraft:game_rules", "rules": {"minecraft:nope": true}})
    ));
    assert!(bad(
        json!({"type": "minecraft:game_rules", "rules": {"minecraft:keep_inventory": "yes"}})
    ));
    assert!(bad(
        json!({"type": "minecraft:game_rules", "rules": {"minecraft:max_entity_cramming": -1}})
    ));
    assert!(bad(json!({"type": "minecraft:mystery"})));
    assert!(!bad(json!({"type": "minecraft:function"})));
}
