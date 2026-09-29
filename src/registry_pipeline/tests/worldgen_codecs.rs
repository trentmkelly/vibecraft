//! Java behaviours of the worldgen codecs (`noise_codecs`, `noise_settings_codecs`,
//! `structure_*_codecs`, `dimension_codecs`, ...) on hand-written inputs, plus the
//! registry-coverage checks that every registered type has a decoder.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::{builtin, field, string};
use crate::registry::Identifier;
use crate::registry_pipeline::carver_codecs::{configured_carver, float_provider};
use crate::registry_pipeline::codec::{Codec, CodecContext};
use crate::registry_pipeline::dimension_codecs::{
    biome_source, chunk_generator, flat_level_generator_preset, level_stem,
    multi_noise_parameter_list, world_preset,
};
use crate::registry_pipeline::enchantment_provider_codecs::enchantment_provider;
use crate::registry_pipeline::noise_codecs::{density_function, noise_parameters};
use crate::registry_pipeline::noise_settings_codecs::{
    climate_parameter, condition_source, noise_generator_settings, rule_source,
};
use crate::registry_pipeline::structure_codecs::{structure, structure_set};
use crate::registry_pipeline::structure_pool_codecs::{
    dimension_padding, pool_alias_binding, pool_element, template_pool,
};
use crate::registry_pipeline::structure_processor_codecs::{
    pos_rule_test, processor_list, rule_block_entity_modifier, rule_test, structure_processor,
};
use crate::registry_pipeline::trade_codecs::{trade_set, villager_trade};
use crate::registry_pipeline::trial_spawner_codecs::trial_spawner_config;
use crate::registry_pipeline::worldgen_common::{block_state, height_provider, vertical_anchor};
use crate::storage::nbt::Tag;

fn loading() -> BTreeSet<Identifier> {
    crate::resource_registry_data_loader::WORLDGEN_REGISTRIES
        .iter()
        .map(|data| Identifier::parse(data.key).expect("registry key"))
        .collect()
}

fn parse(codec: &Codec, json: Value) -> Result<Tag, String> {
    let loading = loading();
    codec.parse(&json, &CodecContext::new(builtin(), &loading))
}

fn error_of(codec: &Codec, json: Value) -> String {
    parse(codec, json).expect_err("expected the codec to reject the input")
}

fn vendored(path: &str) -> Value {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft")
        .join(path);
    serde_json::from_str(&std::fs::read_to_string(&full).expect("vendored file"))
        .expect("vendored JSON")
}

// ---------------------------------------------------------------------------
// noise and density functions
// ---------------------------------------------------------------------------

#[test]
fn noise_parameters_keep_java_types() {
    let tag = parse(
        &noise_parameters(),
        json!({"firstOctave": -7, "amplitudes": [1.0, 1, 0.5]}),
    )
    .expect("noise");
    assert_eq!(field(&tag, "firstOctave"), Some(&Tag::Int(-7)));
    assert_eq!(
        field(&tag, "amplitudes"),
        Some(&Tag::List(vec![
            Tag::Double(1.0),
            Tag::Double(1.0),
            Tag::Double(0.5)
        ]))
    );
    assert!(error_of(&noise_parameters(), json!({"amplitudes": []})).contains("firstOctave"));
}

#[test]
fn constant_density_functions_collapse_to_their_number() {
    let typed = json!({"type": "minecraft:constant", "argument": 0.5});
    assert_eq!(parse(&density_function(), typed), Ok(Tag::Double(0.5)));
    assert_eq!(parse(&density_function(), json!(0.5)), Ok(Tag::Double(0.5)));
    assert!(error_of(&density_function(), json!(2_000_000.0)).contains("outside of range"));
}

#[test]
fn density_function_graph_records_references_and_wraps_markers() {
    let function = json!({
        "type": "minecraft:interpolated",
        "argument": {
            "type": "minecraft:add",
            "argument1": 1,
            "argument2": "minecraft:overworld/base_3d_noise"
        }
    });
    let loading = loading();
    let ctx = CodecContext::new(builtin(), &loading);
    let tag = density_function().parse(&function, &ctx).expect("function");
    let inner = field(&tag, "argument").expect("argument");
    assert_eq!(field(inner, "argument1"), Some(&Tag::Double(1.0)));
    assert_eq!(
        field(inner, "argument2"),
        Some(&string("minecraft:overworld/base_3d_noise"))
    );
    let references = ctx.take_references();
    let density = Identifier::parse("minecraft:worldgen/density_function").expect("key");
    let target = Identifier::parse("minecraft:overworld/base_3d_noise").expect("id");
    assert!(references.elements.contains(&(density, target)));
}

#[test]
fn unknown_density_function_types_are_rejected_like_registry_lookups() {
    let message = error_of(&density_function(), json!({"type": "minecraft:nope"}));
    // `Codec.either(NOISE_VALUE_CODEC, CODEC)` reports both alternatives.
    assert_eq!(
        message,
        "Failed to parse either. First: Not a number: {\"type\":\"minecraft:nope\"}; Second: Unknown registry key in ResourceKey[minecraft:root / minecraft:worldgen/density_function_type]: minecraft:nope"
    );
}

#[test]
fn splines_take_constants_or_non_empty_multipoints() {
    let constant = json!({"type": "minecraft:spline", "spline": 0.25});
    let tag = parse(&density_function(), constant).expect("constant spline");
    assert_eq!(field(&tag, "spline"), Some(&Tag::Float(0.25)));

    let point = |location: f64, value: Value| json!({"location": location, "value": value, "derivative": 0.0});
    let multipoint = json!({"type": "minecraft:spline", "spline": {
        "coordinate": "minecraft:overworld/continents",
        "points": [point(-1.0, json!(0.0)), point(1.0, json!({
            "coordinate": "minecraft:overworld/erosion",
            "points": [point(0.0, json!(1.0))]
        }))]
    }});
    parse(&density_function(), multipoint).expect("multipoint spline");

    let empty = json!({"type": "minecraft:spline", "spline": {
        "coordinate": "minecraft:overworld/continents", "points": []
    }});
    assert!(error_of(&density_function(), empty).ends_with("Second: List must have contents"));
}

#[test]
fn y_clamped_gradient_bounds_use_twice_the_world_height() {
    let gradient = |from: i64| {
        json!({"type": "minecraft:y_clamped_gradient", "from_y": from, "to_y": 0,
               "from_value": 0, "to_value": 1})
    };
    parse(&density_function(), gradient(-4064)).expect("in range");
    assert!(error_of(&density_function(), gradient(-4065)).contains("outside of range"));
}

// ---------------------------------------------------------------------------
// noise settings and surface rules
// ---------------------------------------------------------------------------

#[test]
fn block_states_fill_and_sort_properties_like_the_state_definition_codec() {
    let tag = parse(
        &block_state(),
        json!({"Name": "minecraft:grass_block", "Properties": {"snowy": "true", "bogus": "1"}}),
    )
    .expect("state");
    assert_eq!(
        tag,
        Tag::Compound(vec![
            ("Name".to_string(), string("minecraft:grass_block")),
            (
                "Properties".to_string(),
                Tag::Compound(vec![("snowy".to_string(), string("true"))])
            ),
        ])
    );
    // Invalid values fall back to the default; a singleton state has no Properties.
    let lit = parse(
        &block_state(),
        json!({"Name": "minecraft:furnace", "Properties": {"lit": "maybe"}}),
    )
    .expect("furnace");
    let properties = field(&lit, "Properties").expect("properties");
    assert_eq!(field(properties, "lit"), Some(&string("false")));
    assert_eq!(field(properties, "facing"), Some(&string("north")));
    assert_eq!(
        parse(
            &block_state(),
            json!({"Name": "minecraft:stone", "Properties": {"x": "y"}})
        ),
        Ok(Tag::Compound(vec![(
            "Name".to_string(),
            string("minecraft:stone")
        )]))
    );
    assert!(
        error_of(&block_state(), json!({"Name": "minecraft:not_a_block"}))
            .contains("minecraft:not_a_block")
    );
}

#[test]
fn climate_parameters_quantise_and_collapse_points() {
    assert_eq!(parse(&climate_parameter(), json!(0.5)), Ok(Tag::Float(0.5)));
    assert_eq!(
        parse(&climate_parameter(), json!([0.1, 0.5])),
        Ok(Tag::List(vec![Tag::Float(0.1), Tag::Float(0.5)]))
    );
    assert_eq!(
        parse(&climate_parameter(), json!([0.5, 0.5])),
        Ok(Tag::Float(0.5))
    );
    assert_eq!(
        parse(&climate_parameter(), json!({"min": -1.0, "max": 1.0})),
        Ok(Tag::List(vec![Tag::Float(-1.0), Tag::Float(1.0)]))
    );
    assert!(error_of(&climate_parameter(), json!([0.6, 0.5]))
        .starts_with("Cannon construct interval, min > max"));
    assert!(error_of(&climate_parameter(), json!(2.5)).contains("outside of range"));
    assert!(error_of(&climate_parameter(), json!([0.1])).contains("should have size 2"));
}

#[test]
fn surface_rules_dispatch_conditions_and_invert_nested_sources() {
    let rule = json!({
        "type": "minecraft:condition",
        "if_true": {"type": "minecraft:not", "invert": {"type": "minecraft:hole"}},
        "then_run": {"type": "minecraft:block", "result_state": {"Name": "minecraft:stone"}}
    });
    parse(&rule_source(), rule).expect("rule");
    let bad_depth = json!({"type": "minecraft:water", "offset": 0,
        "surface_depth_multiplier": 21, "add_stone_depth": false});
    let condition = json!({"type": "minecraft:condition", "if_true": bad_depth,
        "then_run": {"type": "minecraft:bandlands"}});
    assert!(error_of(&rule_source(), condition).contains("outside of range [-20:20]"));
}

#[test]
fn vertical_anchors_accept_exactly_one_key() {
    assert_eq!(
        parse(&vertical_anchor(), json!({"absolute": 5})),
        Ok(Tag::Compound(vec![("absolute".to_string(), Tag::Int(5))]))
    );
    assert!(parse(&vertical_anchor(), json!({"absolute": 5, "below_top": 1})).is_err());
    assert!(parse(&vertical_anchor(), json!({})).is_err());
    assert!(error_of(&vertical_anchor(), json!({"absolute": 4000})).contains("outside of range"));
}

#[test]
fn height_providers_collapse_constants_to_their_anchor() {
    let constant = json!({"type": "minecraft:constant", "value": {"absolute": 3}});
    assert_eq!(
        parse(&height_provider(), constant),
        Ok(Tag::Compound(vec![("absolute".to_string(), Tag::Int(3))]))
    );
    parse(
        &height_provider(),
        json!({"type": "minecraft:trapezoid", "min_inclusive": {"absolute": 0},
               "max_inclusive": {"above_bottom": 10}}),
    )
    .expect("trapezoid");
}

#[test]
fn noise_settings_reject_unaligned_or_oversized_worlds() {
    let mut overworld = vendored("worldgen/noise_settings/overworld.json");
    overworld["noise"]["height"] = json!(4064);
    assert_eq!(
        error_of(&noise_generator_settings(), overworld.clone()),
        "min_y + height cannot be higher than: 2032"
    );
    overworld["noise"]["height"] = json!(100);
    assert_eq!(
        error_of(&noise_generator_settings(), overworld),
        "height has to be a multiple of 16"
    );
}

// ---------------------------------------------------------------------------
// structure templates
// ---------------------------------------------------------------------------

#[test]
fn processor_lists_accept_both_forms_and_encode_the_object_form() {
    let bare = json!([{"processor_type": "minecraft:nop"}]);
    let wrapped = json!({"processors": [{"processor_type": "minecraft:nop"}]});
    let expected = Tag::Compound(vec![(
        "processors".to_string(),
        Tag::List(vec![Tag::Compound(vec![(
            "processor_type".to_string(),
            string("minecraft:nop"),
        )])]),
    )]);
    assert_eq!(parse(&processor_list(), bare), Ok(expected.clone()));
    assert_eq!(parse(&processor_list(), wrapped), Ok(expected));
}

#[test]
fn processor_rules_default_the_optional_predicates() {
    let rule = json!({"processor_type": "minecraft:rule", "rules": [{
        "input_predicate": {"predicate_type": "minecraft:always_true"},
        "location_predicate": {"predicate_type": "minecraft:always_true"},
        "position_predicate": {"predicate_type": "minecraft:always_true"},
        "output_state": {"Name": "minecraft:stone"},
        "block_entity_modifier": {"type": "minecraft:passthrough"}
    }]});
    let tag = parse(&structure_processor(), rule).expect("rule");
    let Some(Tag::List(rules)) = field(&tag, "rules") else {
        panic!("rules");
    };
    // Equal to the defaults, so the encoder drops them.
    assert!(field(&rules[0], "position_predicate").is_none());
    assert!(field(&rules[0], "block_entity_modifier").is_none());
    // A position predicate that fails to decode is ignored (lenient optional field).
    let lenient = json!({"processor_type": "minecraft:rule", "rules": [{
        "input_predicate": {"predicate_type": "minecraft:always_true"},
        "location_predicate": {"predicate_type": "minecraft:always_true"},
        "position_predicate": {"predicate_type": "minecraft:nope"},
        "output_state": {"Name": "minecraft:stone"}
    }]});
    parse(&structure_processor(), lenient).expect("lenient position predicate");
}

#[test]
fn rule_tests_validate_their_registries() {
    assert!(parse(
        &rule_test(),
        json!({"predicate_type": "minecraft:tag_match", "tag": "minecraft:logs"})
    )
    .is_ok());
    assert!(error_of(
        &rule_test(),
        json!({"predicate_type": "minecraft:block_match", "block": "minecraft:nope"})
    )
    .contains("minecraft:nope"));
}

#[test]
fn protected_blocks_take_a_hashed_tag_and_block_ignore_drops_properties() {
    let protected = json!({"processor_type": "minecraft:protected_blocks", "value": "#minecraft:features_cannot_replace"});
    parse(&structure_processor(), protected).expect("protected");
    assert!(parse(
        &structure_processor(),
        json!({"processor_type": "minecraft:protected_blocks", "value": "minecraft:x"})
    )
    .is_err());
    let ignore = json!({"processor_type": "minecraft:block_ignore",
        "blocks": [{"Name": "minecraft:grass_block", "Properties": {"snowy": "true"}}]});
    let tag = parse(&structure_processor(), ignore).expect("ignore");
    let Some(Tag::List(blocks)) = field(&tag, "blocks") else {
        panic!("blocks");
    };
    let properties = field(&blocks[0], "Properties").expect("properties");
    assert_eq!(field(properties, "snowy"), Some(&string("false")));
}

#[test]
fn pool_elements_validate_weights_projection_and_lists() {
    let single = |location: &str| {
        json!({"element_type": "minecraft:single_pool_element", "location": location,
               "processors": "minecraft:empty", "projection": "rigid"})
    };
    parse(&pool_element(), single("minecraft:village/x")).expect("single");
    assert!(parse(&pool_element(), single("Not Valid")).is_err());
    let mut bad_projection = single("minecraft:a");
    bad_projection["projection"] = json!("sideways");
    assert_eq!(
        error_of(&pool_element(), bad_projection),
        "Unknown element name:sideways"
    );
    assert_eq!(
        error_of(
            &pool_element(),
            json!({"element_type": "minecraft:list_pool_element", "elements": [], "projection": "rigid"})
        ),
        "Elements are empty"
    );
    parse(
        &pool_element(),
        json!({"element_type": "minecraft:empty_pool_element"}),
    )
    .expect("empty");
}

#[test]
fn template_pool_weights_are_one_to_one_fifty() {
    let pool = |weight: i64| {
        json!({"fallback": "minecraft:empty", "elements": [{
            "weight": weight,
            "element": {"element_type": "minecraft:empty_pool_element"}}]})
    };
    parse(&template_pool(), pool(150)).expect("weight 150");
    assert!(error_of(&template_pool(), pool(151)).contains("outside of range [1:150]"));
    assert!(error_of(&template_pool(), pool(0)).contains("outside of range [1:150]"));
}

#[test]
fn pool_aliases_and_dimension_padding_follow_their_codecs() {
    parse(
        &pool_alias_binding(),
        json!({"type": "minecraft:random", "alias": "minecraft:a",
               "targets": [{"data": "minecraft:b", "weight": 1}]}),
    )
    .expect("random alias");
    assert!(error_of(
        &pool_alias_binding(),
        json!({"type": "minecraft:random", "alias": "minecraft:a", "targets": []})
    )
    .contains("at least one entry"));
    assert_eq!(parse(&dimension_padding(), json!(3)), Ok(Tag::Int(3)));
    assert_eq!(
        parse(&dimension_padding(), json!({"bottom": 2, "top": 2})),
        Ok(Tag::Int(2))
    );
    assert_eq!(
        parse(&dimension_padding(), json!({"bottom": 2})),
        Ok(Tag::Compound(vec![("bottom".to_string(), Tag::Int(2))]))
    );
}

// ---------------------------------------------------------------------------
// structures and structure sets
// ---------------------------------------------------------------------------

#[test]
fn jigsaw_structures_verify_the_horizontal_range() {
    let mut village = vendored("worldgen/structure/village_plains.json");
    village["max_distance_from_center"] = json!(120);
    village["terrain_adaptation"] = json!("beard_thin");
    assert_eq!(
        error_of(&structure(), village.clone()),
        "Horizontal structure size including terrain adaptation must not exceed 128"
    );
    village["terrain_adaptation"] = json!("none");
    parse(&structure(), village.clone()).expect("no adaptation fits");
    village["max_distance_from_center"] = json!({"horizontal": 80, "vertical": 80});
    let tag = parse(&structure(), village).expect("collapsed distance");
    assert_eq!(field(&tag, "max_distance_from_center"), Some(&Tag::Int(80)));
}

#[test]
fn structure_spawn_overrides_and_types_are_checked() {
    let mut fortress = vendored("worldgen/structure/fortress.json");
    parse(&structure(), fortress.clone()).expect("fortress");
    fortress["spawn_overrides"]["bogus"] = json!({"bounding_box": "piece", "spawns": []});
    assert!(error_of(&structure(), fortress.clone()).contains("Unknown element name:bogus"));
    fortress["type"] = json!("minecraft:nope");
    assert!(error_of(&structure(), fortress).contains("Unknown registry key"));
}

#[test]
fn random_spread_requires_spacing_larger_than_separation() {
    let placement = |spacing: i64, separation: i64| {
        json!({"structures": [{"structure": "minecraft:igloo", "weight": 1}], "placement": {
            "type": "minecraft:random_spread", "salt": 1, "spacing": spacing, "separation": separation}})
    };
    parse(&structure_set(), placement(32, 8)).expect("valid");
    assert_eq!(
        error_of(&structure_set(), placement(8, 8)),
        "Spacing has to be larger than separation"
    );
    assert!(error_of(&structure_set(), placement(5000, 8)).contains("outside of range [0:4096]"));
}

// ---------------------------------------------------------------------------
// world creation
// ---------------------------------------------------------------------------

#[test]
fn world_presets_need_an_overworld_and_valid_generators() {
    let mut preset = vendored("worldgen/world_preset/normal.json");
    parse(&world_preset(), preset.clone()).expect("normal");
    let nether = preset["dimensions"]["minecraft:the_nether"].clone();
    preset["dimensions"] = json!({"minecraft:the_nether": nether});
    assert_eq!(
        error_of(&world_preset(), preset),
        "Missing overworld dimension"
    );
}

#[test]
fn level_stems_dispatch_generators_and_biome_sources() {
    let stem = |source: Value| {
        json!({"type": "minecraft:overworld", "generator": {
            "type": "minecraft:noise", "settings": "minecraft:overworld", "biome_source": source}})
    };
    parse(
        &level_stem(),
        stem(json!({"type": "minecraft:fixed", "biome": "minecraft:plains"})),
    )
    .expect("fixed");
    parse(
        &level_stem(),
        stem(json!({"type": "minecraft:checkerboard", "biomes": ["minecraft:plains"]})),
    )
    .expect("checkerboard");
    let tag = parse(&level_stem(), stem(json!({"type": "minecraft:the_end"}))).expect("the_end");
    let generator = field(&tag, "generator").expect("generator");
    assert_eq!(
        field(field(generator, "biome_source").expect("source"), "type"),
        Some(&string("minecraft:the_end"))
    );
    // A bare identifier is a registry reference that only fails when the registry
    // freezes; an inline preset with an unknown name fails immediately.
    parse(
        &level_stem(),
        stem(json!({"type": "minecraft:multi_noise", "preset": "minecraft:nope"})),
    )
    .expect("reference to a not-yet-bound preset");
    assert!(parse(
        &level_stem(),
        stem(json!({"type": "minecraft:multi_noise", "preset": {"preset": "minecraft:nope"}}))
    )
    .is_err());
}

#[test]
fn flat_presets_default_layers_and_biome() {
    let preset = json!({"display": "minecraft:stone", "settings": {
        "layers": [{"height": 2}]}});
    let tag = parse(&flat_level_generator_preset(), preset).expect("flat preset");
    let settings = field(&tag, "settings").expect("settings");
    assert_eq!(field(settings, "biome"), Some(&string("minecraft:plains")));
    assert_eq!(field(settings, "lakes"), Some(&Tag::Byte(0)));
    let Some(Tag::List(layers)) = field(settings, "layers") else {
        panic!("layers");
    };
    assert_eq!(field(&layers[0], "block"), Some(&string("minecraft:air")));
    let too_tall = json!({"display": "minecraft:stone", "settings": {
        "layers": [{"height": 4000}, {"height": 100}]}});
    assert_eq!(
        error_of(&flat_level_generator_preset(), too_tall),
        "Sum of layer heights is > 4064"
    );
}

#[test]
fn multi_noise_presets_are_the_two_built_in_names() {
    parse(
        &multi_noise_parameter_list(),
        json!({"preset": "minecraft:nether"}),
    )
    .expect("nether");
    assert_eq!(
        error_of(
            &multi_noise_parameter_list(),
            json!({"preset": "minecraft:end"})
        ),
        "Unknown preset: minecraft:end"
    );
}

// ---------------------------------------------------------------------------
// enchantment providers and trial spawners
// ---------------------------------------------------------------------------

#[test]
fn enchantment_providers_check_cost_ranges() {
    let by_cost = |min: i64| {
        json!({"type": "minecraft:by_cost_with_difficulty", "enchantments": "#minecraft:in_enchanting_table",
               "min_cost": min, "max_cost_span": 5})
    };
    parse(&enchantment_provider(), by_cost(1)).expect("min cost 1");
    assert!(error_of(&enchantment_provider(), by_cost(0)).contains("outside of range [1:10000]"));
    let single =
        json!({"type": "minecraft:single", "enchantment": "minecraft:piercing", "level": 1});
    let tag = parse(&enchantment_provider(), single).expect("single");
    assert_eq!(field(&tag, "level"), Some(&Tag::Int(1)));
}

#[test]
fn trial_spawner_defaults_are_omitted_and_slot_chances_collapse() {
    assert_eq!(
        parse(
            &trial_spawner_config(),
            json!({"spawn_range": 4, "total_mobs": 6.0})
        ),
        Ok(Tag::Compound(Vec::new()))
    );
    assert!(error_of(&trial_spawner_config(), json!({"spawn_range": 0}))
        .contains("outside of range [1:128]"));
    let spawn = json!({"spawn_potentials": [{"weight": 1, "data": {
        "entity": {"id": "zombie"},
        "custom_spawn_rules": {"block_light_limit": [0, 4]},
        "equipment": {"loot_table": "minecraft:equipment/x", "slot_drop_chances": {
            "mainhand": 0.0, "offhand": 0.0, "feet": 0.0, "legs": 0.0,
            "chest": 0.0, "head": 0.0, "body": 0.0, "saddle": 0.0}}}}]});
    let tag = parse(&trial_spawner_config(), spawn).expect("spawn potentials");
    let Some(Tag::List(entries)) = field(&tag, "spawn_potentials") else {
        panic!("spawn_potentials");
    };
    let data = field(&entries[0], "data").expect("data");
    assert_eq!(
        field(field(data, "entity").expect("entity"), "id"),
        Some(&string("minecraft:zombie"))
    );
    let equipment = field(data, "equipment").expect("equipment");
    assert_eq!(
        field(equipment, "slot_drop_chances"),
        Some(&Tag::Float(0.0))
    );
    let too_bright = json!({"spawn_potentials": [{"weight": 1, "data": {
        "entity": {}, "custom_spawn_rules": {"sky_light_limit": [0, 16]}}}]});
    assert!(parse(&trial_spawner_config(), too_bright).is_err());
}

// ---------------------------------------------------------------------------
// villager trades
// ---------------------------------------------------------------------------

#[test]
fn villager_trades_default_their_optional_numbers_and_reject_air() {
    let trade = json!({"wants": {"id": "minecraft:emerald", "count": 2},
        "gives": {"id": "minecraft:bread", "count": 6}, "max_uses": 12, "xp": 1});
    let tag = parse(&villager_trade(), trade).expect("trade");
    // `xp` equals the default (1.0) and is not written; `max_uses` is a float.
    assert!(field(&tag, "xp").is_none());
    assert_eq!(field(&tag, "max_uses"), Some(&Tag::Float(12.0)));
    let wants = field(&tag, "wants").expect("wants");
    assert_eq!(field(wants, "count"), Some(&Tag::Float(2.0)));
    let gives = field(&tag, "gives").expect("gives");
    assert_eq!(field(gives, "count"), Some(&Tag::Int(6)));

    let air = json!({"wants": {"id": "minecraft:emerald"}, "gives": {"id": "minecraft:air"}});
    assert_eq!(error_of(&villager_trade(), air), "Item must be non-empty");
    let too_many = json!({"wants": {"id": "minecraft:emerald"},
        "gives": {"id": "minecraft:bread", "count": 100}});
    assert!(error_of(&villager_trade(), too_many).contains("[1;99]"));
    let missing = json!({"wants": {"id": "minecraft:emerald"}});
    assert!(error_of(&villager_trade(), missing).contains("No key gives"));
}

#[test]
fn villager_trade_components_and_loot_parts_are_validated() {
    let with_component = |name: &str| {
        json!({"wants": {"id": "minecraft:emerald", "components": {name: 1}},
               "gives": {"id": "minecraft:bread"}})
    };
    parse(&villager_trade(), with_component("minecraft:damage")).expect("known component");
    assert!(parse(&villager_trade(), with_component("minecraft:nope")).is_err());

    // An unparseable lenient number falls back to its default.
    let lenient = json!({"wants": {"id": "minecraft:emerald"},
        "gives": {"id": "minecraft:bread"}, "max_uses": "many"});
    assert!(field(
        &parse(&villager_trade(), lenient).expect("lenient"),
        "max_uses"
    )
    .is_none());

    let bad_function = json!({"wants": {"id": "minecraft:emerald"},
        "gives": {"id": "minecraft:bread"},
        "given_item_modifiers": [{"function": "minecraft:nope"}]});
    assert!(parse(&villager_trade(), bad_function).is_err());
    let predicate = json!({"wants": {"id": "minecraft:emerald"},
        "gives": {"id": "minecraft:bread"},
        "merchant_predicate": {"condition": "minecraft:entity_properties", "entity": "this",
                               "predicate": {}}});
    parse(&villager_trade(), predicate).expect("merchant predicate");
}

#[test]
fn trade_sets_reference_trades_and_default_duplicates() {
    let set = json!({"trades": "#minecraft:armorer/level_1", "amount": 2});
    let tag = parse(&trade_set(), set).expect("set");
    assert_eq!(field(&tag, "amount"), Some(&Tag::Float(2.0)));
    assert!(field(&tag, "allow_duplicates").is_none());
    let listed = json!({"trades": ["minecraft:a", "minecraft:b"],
        "amount": {"type": "minecraft:uniform", "min": 1, "max": 3},
        "allow_duplicates": true, "random_sequence": "minecraft:trade"});
    let tag = parse(&trade_set(), listed).expect("listed set");
    assert_eq!(field(&tag, "allow_duplicates"), Some(&Tag::Byte(1)));
    assert!(error_of(&trade_set(), json!({"amount": 1})).contains("No key trades"));
}

// ---------------------------------------------------------------------------
// carvers
// ---------------------------------------------------------------------------

#[test]
fn carver_configurations_validate_float_providers() {
    let mut cave = vendored("worldgen/configured_carver/cave.json");
    parse(&configured_carver(), cave.clone()).expect("cave");
    cave["config"]["floor_level"] = json!(2.0);
    assert_eq!(
        error_of(&configured_carver(), cave.clone()),
        "Value provider too high: 1.0 [2.0-2.0]"
    );
    cave["config"]["floor_level"] =
        json!({"type": "minecraft:uniform", "min_inclusive": 1.0, "max_exclusive": 1.0});
    assert_eq!(
        error_of(&configured_carver(), cave),
        "Failed to parse either. First: Not a number: {\"max_exclusive\":1.0,\"min_inclusive\":1.0,\"type\":\"minecraft:uniform\"}; Second: Max must be larger than min, min: 1.0, max: 1.0"
    );
}

#[test]
fn float_providers_collapse_constants_and_check_trapezoids() {
    let constant = json!({"type": "minecraft:constant", "value": 3.0});
    assert_eq!(parse(&float_provider(), constant), Ok(Tag::Float(3.0)));
    let trapezoid = |plateau: f64| json!({"type": "minecraft:trapezoid", "min": 0.0, "max": 4.0, "plateau": plateau});
    parse(&float_provider(), trapezoid(4.0)).expect("plateau equal to the span");
    assert!(error_of(&float_provider(), trapezoid(4.5))
        .contains("Plateau can at most be the full span: [0.0, 4.0]"));
}

// ---------------------------------------------------------------------------
// registry coverage
// ---------------------------------------------------------------------------

/// Every entry of a built-in type registry must have a decoder: a probe with just the
/// type never fails with "Unknown registry key", while a made-up type always does.
fn assert_covers_registry(registry: &str, type_key: &str, codec: &Codec) {
    let key = Identifier::parse(registry).expect("registry key");
    let entries = builtin().get(&key).unwrap_or_else(|| panic!("{registry}"));
    assert!(!entries.elements().is_empty(), "{registry}");
    for entry in entries.elements() {
        let probe = json!({ type_key: entry.to_string() });
        let outcome = parse(codec, probe);
        assert!(
            outcome
                .as_ref()
                .err()
                .is_none_or(|err| !err.contains("Unknown registry key")),
            "{registry}: {entry} has no decoder ({outcome:?})"
        );
    }
    let unknown = error_of(codec, json!({ type_key: "minecraft:vibecraft_missing" }));
    assert!(
        unknown.contains("Unknown registry key"),
        "{registry}: {unknown}"
    );
}

#[test]
fn every_registered_worldgen_type_has_a_decoder() {
    assert_covers_registry(
        "minecraft:worldgen/density_function_type",
        "type",
        &density_function(),
    );
    assert_covers_registry("minecraft:worldgen/structure_type", "type", &structure());
    assert_covers_registry("minecraft:worldgen/carver", "type", &configured_carver());
    assert_covers_registry("minecraft:float_provider_type", "type", &float_provider());
    assert_covers_registry(
        "minecraft:worldgen/structure_processor",
        "processor_type",
        &structure_processor(),
    );
    assert_covers_registry(
        "minecraft:worldgen/structure_pool_element",
        "element_type",
        &pool_element(),
    );
    assert_covers_registry(
        "minecraft:worldgen/pool_alias_binding",
        "type",
        &pool_alias_binding(),
    );
    assert_covers_registry("minecraft:rule_test", "predicate_type", &rule_test());
    assert_covers_registry("minecraft:worldgen/material_rule", "type", &rule_source());
    assert_covers_registry("minecraft:height_provider_type", "type", &height_provider());
    assert_covers_registry(
        "minecraft:enchantment_provider_type",
        "type",
        &enchantment_provider(),
    );
    assert_covers_registry(
        "minecraft:pos_rule_test",
        "predicate_type",
        &pos_rule_test(),
    );
    assert_covers_registry(
        "minecraft:rule_block_entity_modifier",
        "type",
        &rule_block_entity_modifier(),
    );
    assert_covers_registry(
        "minecraft:worldgen/material_condition",
        "type",
        &condition_source(),
    );
    assert_covers_registry(
        "minecraft:worldgen/chunk_generator",
        "type",
        &chunk_generator(),
    );
    assert_covers_registry("minecraft:worldgen/biome_source", "type", &biome_source());
    let placement = structure_set();
    let key = Identifier::parse("minecraft:worldgen/structure_placement").expect("key");
    for entry in builtin().get(&key).expect("placements").elements() {
        let probe = json!({"structures": [], "placement": {"type": entry.to_string()}});
        let outcome = parse(&placement, probe);
        assert!(
            outcome
                .err()
                .is_none_or(|err| !err.contains("Unknown registry key")),
            "{entry}"
        );
    }
}
