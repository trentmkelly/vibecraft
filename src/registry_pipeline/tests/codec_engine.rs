//! Unit tests for the JSON-to-NBT codec combinators and shared codecs.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::{builtin, field, string};
use crate::registry::Identifier;
use crate::registry_pipeline::attributes::{environment_attribute_map, AttributeMapMode};
use crate::registry_pipeline::codec::{
    bool_codec, compact_list, dispatch, either, enum_codec, float_codec, generic, holder_fixed,
    holder_set, int_range, list, non_negative_int, opt, opt_default, or_else, record, req, Codec,
    CodecContext,
};
use crate::registry_pipeline::shared::{
    component, int_provider, particle_options, sound_event, string_argb_color, string_rgb_color,
};
use crate::storage::nbt::Tag;

fn parse(codec: &Codec, json: Value) -> Result<Tag, String> {
    let loading = BTreeSet::new();
    codec.parse(&json, &CodecContext::new(builtin(), &loading))
}

fn loading_set(registries: &[&str]) -> BTreeSet<Identifier> {
    registries
        .iter()
        .map(|id| Identifier::parse(id).expect("registry id"))
        .collect()
}

#[test]
fn numeric_leaves_carry_the_java_nbt_type() {
    let codec = record(vec![req("f", float_codec()), req("i", int_range(0, 5))]);
    let tag = parse(&codec, json!({"f": 1, "i": 3})).expect("parse");
    assert_eq!(field(&tag, "f"), Some(&Tag::Float(1.0)));
    assert_eq!(field(&tag, "i"), Some(&Tag::Int(3)));
    assert_eq!(
        parse(&codec, json!({"f": 1, "i": 9})).unwrap_err(),
        "Value 9 outside of range [0:5]"
    );
}

#[test]
fn json_leniency_matches_json_ops() {
    // JsonOps.getBooleanValue accepts numbers; getNumberValue accepts booleans.
    assert_eq!(parse(&bool_codec(), json!(1)), Ok(Tag::Byte(1)));
    assert_eq!(parse(&bool_codec(), json!(0)), Ok(Tag::Byte(0)));
    assert!(parse(&bool_codec(), json!("true")).is_err());
    assert_eq!(parse(&non_negative_int(), json!(true)), Ok(Tag::Int(1)));
    assert!(parse(&non_negative_int(), json!(-1))
        .unwrap_err()
        .contains("non-negative"));
}

#[test]
fn record_field_modes_follow_dfu_semantics() {
    let codec = record(vec![
        req("required", bool_codec()),
        opt("optional", bool_codec()),
        opt_default("defaulted", enum_codec(&["a", "b"]), json!("a")),
        or_else("fallback", int_range(0, 10), json!(7)),
    ]);
    // Absent optional fields vanish, a default-valued field is not written, and
    // orElse always writes (using the default on failure).
    let tag = parse(&codec, json!({"required": true, "fallback": 99})).expect("parse");
    assert_eq!(
        tag,
        Tag::Compound(vec![
            ("required".to_string(), Tag::Byte(1)),
            ("fallback".to_string(), Tag::Int(7)),
        ])
    );
    let tag = parse(
        &codec,
        json!({"required": false, "defaulted": "b", "optional": true}),
    )
    .expect("parse");
    assert_eq!(field(&tag, "defaulted"), Some(&string("b")));
    assert_eq!(field(&tag, "optional"), Some(&Tag::Byte(1)));
    assert_eq!(field(&tag, "fallback"), Some(&Tag::Int(7)));
    let err = parse(&codec, json!({"defaulted": "a"})).unwrap_err();
    assert!(err.starts_with("No key required in MapLike["), "{err}");
    assert!(parse(&codec, json!("nope"))
        .unwrap_err()
        .starts_with("Not a map"));
}

#[test]
fn either_keeps_the_first_matching_branch_and_reports_both_errors() {
    let codec = either(int_range(0, 3), enum_codec(&["x"]));
    assert_eq!(parse(&codec, json!(2)), Ok(Tag::Int(2)));
    assert_eq!(parse(&codec, json!("x")), Ok(string("x")));
    let err = parse(&codec, json!(true.to_string())).unwrap_err();
    assert!(err.starts_with("Failed to parse either. First:"), "{err}");
}

#[test]
fn compact_list_collapses_single_elements() {
    let codec = compact_list(enum_codec(&["a", "b"]));
    assert_eq!(parse(&codec, json!("a")), Ok(string("a")));
    assert_eq!(parse(&codec, json!(["a"])), Ok(string("a")));
    assert_eq!(
        parse(&codec, json!(["a", "b"])),
        Ok(Tag::List(vec![string("a"), string("b")]))
    );
    assert_eq!(
        parse(&list(enum_codec(&["a"])), json!([])),
        Ok(Tag::List(Vec::new()))
    );
}

#[test]
fn dispatch_writes_the_type_first_and_rejects_unknown_types() {
    let codec = dispatch("type", |id| match id.to_string().as_str() {
        "minecraft:one" => Ok(record(vec![req("value", int_range(0, 9))])),
        other => Err(format!("unknown {other}")),
    });
    assert_eq!(
        parse(&codec, json!({"type": "one", "value": 4})),
        Ok(Tag::Compound(vec![
            ("type".to_string(), string("minecraft:one")),
            ("value".to_string(), Tag::Int(4)),
        ]))
    );
    assert_eq!(
        parse(&codec, json!({"type": "two"})).unwrap_err(),
        "unknown minecraft:two"
    );
}

#[test]
fn holders_resolve_static_registries_and_record_dynamic_references() {
    let sound = sound_event();
    assert_eq!(
        parse(&sound, json!("minecraft:ambient.cave")),
        Ok(string("minecraft:ambient.cave"))
    );
    assert!(parse(&sound, json!("minecraft:not_a_sound"))
        .unwrap_err()
        .contains("Failed to get element"));
    // Inline SoundEvent definitions are allowed by RegistryFileCodec.
    assert_eq!(
        parse(
            &sound,
            json!({"sound_id": "minecraft:ambient.cave", "range": 4})
        ),
        Ok(Tag::Compound(vec![
            ("sound_id".to_string(), string("minecraft:ambient.cave")),
            ("range".to_string(), Tag::Float(4.0)),
        ]))
    );

    let loading = loading_set(&["minecraft:world_clock"]);
    let ctx = CodecContext::new(builtin(), &loading);
    holder_fixed("minecraft:world_clock")
        .parse(&json!("minecraft:overworld"), &ctx)
        .expect("reference");
    let references = ctx.take_references();
    assert_eq!(references.elements.len(), 1);
    // Registries this pipeline does not load cannot be verified and are not recorded.
    holder_fixed("minecraft:worldgen/structure")
        .parse(&json!("minecraft:village"), &ctx)
        .expect("unloaded registry");
    assert!(ctx.take_references().elements.is_empty());
}

#[test]
fn holder_sets_accept_tags_lists_and_single_elements() {
    let loading = loading_set(&["minecraft:timeline"]);
    let ctx = CodecContext::new(builtin(), &loading);
    let set = holder_set("minecraft:timeline", false);
    assert_eq!(
        set.parse(&json!("#minecraft:in_overworld"), &ctx),
        Ok(string("#minecraft:in_overworld"))
    );
    assert_eq!(
        set.parse(&json!("minecraft:day"), &ctx),
        Ok(string("minecraft:day"))
    );
    assert_eq!(
        set.parse(&json!(["minecraft:day", "minecraft:moon"]), &ctx),
        Ok(Tag::List(vec![
            string("minecraft:day"),
            string("minecraft:moon")
        ]))
    );
    let references = ctx.take_references();
    assert_eq!(references.tags.len(), 1);
    assert_eq!(references.elements.len(), 2);
    // alwaysUseList rejects a bare element.
    assert!(holder_set("minecraft:timeline", true)
        .parse(&json!("minecraft:day"), &ctx)
        .is_err());
}

#[test]
fn colors_are_canonicalised_like_extra_codecs() {
    assert_eq!(
        parse(&string_rgb_color(), json!("#78A7FF")),
        Ok(string("#78a7ff"))
    );
    // The int alternative decodes but is re-encoded as a hex string.
    assert_eq!(parse(&string_rgb_color(), json!(-1)), Ok(string("#ffffff")));
    assert_eq!(
        parse(&string_rgb_color(), json!([1.0, 0.0, 0.0])),
        Ok(string("#ff0000"))
    );
    assert_eq!(
        parse(&string_argb_color(), json!("#CCFFFFFF")),
        Ok(string("#ccffffff"))
    );
    assert_eq!(
        parse(&string_rgb_color(), json!("78a7ff")).unwrap_err(),
        "Hex color must begin with #"
    );
    assert!(parse(&string_rgb_color(), json!("#12345"))
        .unwrap_err()
        .contains("wrong size"));
    assert!(parse(&string_argb_color(), json!("#12345678z")).is_err());
}

#[test]
fn int_providers_collapse_constants_and_enforce_bounds() {
    let codec = int_provider(0, 15);
    assert_eq!(parse(&codec, json!(7)), Ok(Tag::Int(7)));
    assert_eq!(
        parse(&codec, json!({"type": "minecraft:constant", "value": 7})),
        Ok(Tag::Int(7))
    );
    let uniform = json!({"type": "minecraft:uniform", "min_inclusive": 0, "max_inclusive": 7});
    let tag = parse(&codec, uniform).expect("uniform");
    assert_eq!(field(&tag, "type"), Some(&string("minecraft:uniform")));
    assert_eq!(field(&tag, "max_inclusive"), Some(&Tag::Int(7)));
    assert_eq!(
        parse(&codec, json!(16)).unwrap_err(),
        "Value provider too high: 15 [16-16]"
    );
    assert_eq!(
        parse(&codec, json!(-1)).unwrap_err(),
        "Value provider too low: 0 [-1--1]"
    );
    assert_eq!(
        parse(
            &codec,
            json!({"type": "minecraft:uniform", "min_inclusive": 5, "max_inclusive": 2})
        )
        .unwrap_err(),
        "Max must be at least min, min_inclusive: 5, max_inclusive: 2"
    );
    let weighted = json!({"type": "minecraft:weighted_list", "distribution": [
        {"data": 3, "weight": 1}, {"data": {"type": "minecraft:uniform", "min_inclusive": 1, "max_inclusive": 9}, "weight": 2}
    ]});
    assert!(parse(&codec, weighted).is_ok());
    let empty = json!({"type": "minecraft:weighted_list", "distribution": []});
    assert!(parse(&codec, empty)
        .unwrap_err()
        .contains("at least one entry"));
}

#[test]
fn components_keep_translate_and_color_and_collapse_plain_text() {
    let codec = component();
    assert_eq!(parse(&codec, json!("plain")), Ok(string("plain")));
    assert_eq!(parse(&codec, json!({"text": "plain"})), Ok(string("plain")));
    let tag = parse(
        &codec,
        json!({"translate": "a.b", "color": "#9a5cc6", "bold": true}),
    )
    .expect("component");
    assert_eq!(field(&tag, "translate"), Some(&string("a.b")));
    assert_eq!(field(&tag, "color"), Some(&string("#9A5CC6")));
    assert_eq!(field(&tag, "bold"), Some(&Tag::Byte(1)));
    assert!(
        parse(&codec, json!({"translate": "a", "color": "chartreuse"}))
            .unwrap_err()
            .contains("Invalid color name")
    );
    assert!(parse(&codec, json!(5)).is_err());
}

#[test]
fn particle_options_require_a_known_type_and_keep_options_only_where_they_exist() {
    let codec = particle_options();
    assert_eq!(
        parse(&codec, json!({"type": "minecraft:ash", "ignored": 1})),
        Ok(Tag::Compound(vec![(
            "type".to_string(),
            string("minecraft:ash")
        )]))
    );
    let dust = parse(
        &codec,
        json!({"type": "minecraft:dust", "scale": 1.5, "color": 16711680}),
    )
    .expect("dust");
    assert_eq!(field(&dust, "scale"), Some(&Tag::Float(1.5)));
    assert!(parse(&codec, json!({"type": "minecraft:not_a_particle"})).is_err());
}

#[test]
fn attribute_maps_filter_syncable_and_validate_positional_and_modifiers() {
    let map = |mode| environment_attribute_map(mode);
    let input = json!({
        "minecraft:visual/sky_color": "#ff0000",
        "minecraft:gameplay/increased_fire_burnout": true,
        "minecraft:visual/fog_end_distance": {"modifier": "multiply", "argument": 0.5},
        "minecraft:visual/fog_start_distance": {"modifier": "override", "argument": 4.0},
    });
    let network = parse(&map(AttributeMapMode::Network), input.clone()).expect("network");
    assert_eq!(
        field(&network, "minecraft:visual/sky_color"),
        Some(&string("#ff0000"))
    );
    assert!(field(&network, "minecraft:gameplay/increased_fire_burnout").is_none());
    // Explicit override collapses to the bare value; other modifiers keep the record.
    assert_eq!(
        field(&network, "minecraft:visual/fog_start_distance"),
        Some(&Tag::Float(4.0))
    );
    assert_eq!(
        field(
            field(&network, "minecraft:visual/fog_end_distance").expect("fog"),
            "modifier"
        ),
        Some(&string("multiply"))
    );

    let direct = parse(&map(AttributeMapMode::Direct), input.clone()).expect("direct");
    assert!(field(&direct, "minecraft:gameplay/increased_fire_burnout").is_some());

    // Biomes only allow positional attributes.
    let non_positional = json!({"minecraft:gameplay/fast_lava": true});
    assert!(
        parse(&map(AttributeMapMode::OnlyPositional), non_positional)
            .unwrap_err()
            .contains("cannot be positional")
    );

    // Modifiers must belong to the attribute type's library, ranges are enforced.
    let bad_modifier = json!({"minecraft:visual/sky_color": {"modifier": "and", "argument": true}});
    assert!(parse(&map(AttributeMapMode::Direct), bad_modifier).is_err());
    let out_of_range = json!({"minecraft:visual/star_brightness": 2.0});
    assert!(parse(&map(AttributeMapMode::Direct), out_of_range)
        .unwrap_err()
        .contains("is not in range"));
    let unknown = json!({"minecraft:visual/nope": 1});
    assert!(parse(&map(AttributeMapMode::Direct), unknown)
        .unwrap_err()
        .contains("Unknown registry key"));
}

#[test]
fn generic_conversion_types_numbers_by_representability() {
    let tag = parse(&generic(), json!({"a": 1, "b": 0.5, "c": 0.1234567890123, "g": 0.15, "d": true, "e": [1, 2], "f": null}))
        .expect("generic");
    assert_eq!(field(&tag, "a"), Some(&Tag::Int(1)));
    assert_eq!(field(&tag, "b"), Some(&Tag::Float(0.5)));
    assert_eq!(field(&tag, "c"), Some(&Tag::Double(0.1234567890123)));
    // The shortest f32 literal is a float field even though it is not exact in f32.
    assert_eq!(field(&tag, "g"), Some(&Tag::Float(0.15)));
    assert_eq!(field(&tag, "d"), Some(&Tag::Byte(1)));
    assert!(field(&tag, "f").is_none());
}
