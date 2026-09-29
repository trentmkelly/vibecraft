//! Codecs of the game-test registries: `TestEnvironmentDefinition.DIRECT_CODEC`
//! (`minecraft:test_environment`) and `GameTestInstance.DIRECT_CODEC`
//! (`minecraft:test_instance`). Both registries are synchronised to clients, so the
//! same codec decodes packs and encodes the network form.

use serde_json::json;

use crate::game_rules::{game_rule_definition, GameRuleType};
use crate::registry_pipeline::codec::{
    bool_codec, dispatch, dispatched_map, enum_codec, holder_file, holder_fixed, identifier_codec,
    int_range, lazy, list, non_negative_int, opt, opt_default, parse_identifier, positive_int,
    record, req, Codec,
};
use crate::storage::nbt::Tag;

const TEST_ENVIRONMENT_REGISTRY: &str = "minecraft:test_environment";
const WORLD_CLOCK_REGISTRY: &str = "minecraft:world_clock";
const TIMELINE_REGISTRY: &str = "minecraft:timeline";

/// `TestEnvironmentDefinition.Weather.Type.CODEC` (`StringRepresentable` ids).
const WEATHER_TYPES: &[&str] = &["clear", "rain", "thunder"];
/// `Rotation.CODEC` (`StringRepresentable` ids).
const ROTATIONS: &[&str] = &["none", "clockwise_90", "180", "counterclockwise_90"];

/// `BuiltInRegistries.GAME_RULE.byNameCodec()`: a known game rule id, written back
/// as the fully qualified identifier.
fn game_rule_key() -> Codec {
    Codec::new(|json, _| {
        let id = parse_identifier(json)?;
        match (id.namespace(), game_rule_definition(id.path())) {
            ("minecraft", Some(_)) => Ok(Tag::String(id.to_string())),
            _ => Err(format!(
                "Unknown registry key in ResourceKey[minecraft:root / minecraft:game_rule]: {id}"
            )),
        }
    })
}

/// `GameRule.valueCodec` of the rule named by the canonical key.
fn game_rule_value(canonical_key: &str) -> Result<Codec, String> {
    let definition = canonical_key
        .strip_prefix("minecraft:")
        .and_then(game_rule_definition)
        .ok_or_else(|| format!("Unknown game rule {canonical_key}"))?;
    Ok(match definition.rule_type {
        GameRuleType::Bool => bool_codec(),
        GameRuleType::Int => int_range(
            definition.min.unwrap_or(i32::MIN),
            definition.max.unwrap_or(i32::MAX),
        ),
    })
}

/// `TestEnvironmentDefinition.DIRECT_CODEC`: dispatched on the environment `type`.
pub fn test_environment() -> Codec {
    dispatch("type", |id| {
        Ok(match id.to_string().as_str() {
            "minecraft:all_of" => record(vec![req(
                "definitions",
                list(environment_holder()),
            )]),
            "minecraft:game_rules" => record(vec![req(
                "rules",
                dispatched_map(game_rule_key(), game_rule_value),
            )]),
            "minecraft:clock_time" => record(vec![
                req("clock", holder_fixed(WORLD_CLOCK_REGISTRY)),
                req("time", non_negative_int()),
            ]),
            "minecraft:timeline_attributes" => record(vec![req(
                "timelines",
                list(holder_fixed(TIMELINE_REGISTRY)),
            )]),
            "minecraft:weather" => record(vec![req("weather", enum_codec(WEATHER_TYPES))]),
            "minecraft:function" => record(vec![
                opt("setup", identifier_codec()),
                opt("teardown", identifier_codec()),
            ]),
            other => {
                return Err(format!(
                    "Unknown registry key in ResourceKey[minecraft:root / minecraft:test_environment_definition_type]: {other}"
                ))
            }
        })
    })
}

/// `TestEnvironmentDefinition.CODEC`: a registered environment or an inline one.
fn environment_holder() -> Codec {
    holder_file(TEST_ENVIRONMENT_REGISTRY, lazy(test_environment))
}

/// `TestData.CODEC`: the fields shared by every test instance type.
fn test_data() -> Vec<crate::registry_pipeline::codec::Field> {
    vec![
        req("environment", environment_holder()),
        req("structure", identifier_codec()),
        req("max_ticks", positive_int()),
        opt_default("setup_ticks", non_negative_int(), json!(0)),
        opt_default("required", bool_codec(), json!(true)),
        opt_default("rotation", enum_codec(ROTATIONS), json!("none")),
        opt_default("manual_only", bool_codec(), json!(false)),
        opt_default("max_attempts", positive_int(), json!(1)),
        opt_default("required_successes", positive_int(), json!(1)),
        opt_default("sky_access", bool_codec(), json!(false)),
        opt_default("padding", int_range(0, 128), json!(0)),
    ]
}

/// `GameTestInstance.DIRECT_CODEC`: dispatched on the instance `type`.
pub fn test_instance() -> Codec {
    dispatch("type", |id| {
        match id.to_string().as_str() {
        // `BlockBasedTestInstance.CODEC`.
        "minecraft:block_based" => Ok(record(test_data())),
        // `FunctionGameTestInstance.CODEC`: a `ResourceKey<TestFunction>` then the data.
        "minecraft:function" => {
            let mut fields = vec![req("function", identifier_codec())];
            fields.extend(test_data());
            Ok(record(fields))
        }
        other => Err(format!(
            "Unknown registry key in ResourceKey[minecraft:root / minecraft:test_instance_type]: {other}"
        )),
    }
    })
}
