//! Codec for `worldgen/noise_settings` (`NoiseGeneratorSettings.DIRECT_CODEC`) with
//! `NoiseSettings`, `NoiseRouter`, `Climate.ParameterPoint` and the `SurfaceRules`
//! condition/rule sources.

use crate::registry_pipeline::codec::{
    bool_codec, double_codec, enum_codec, float_range, identifier_codec, int_codec, int_range,
    lazy, list, record, req, Codec,
};
use crate::registry_pipeline::noise_codecs::density_function_holder;
use crate::registry_pipeline::worldgen_common::{
    block_state, interval_codec, registry_dispatch, unit, vertical_anchor, Variant, MAX_Y, MIN_Y,
    Y_SIZE,
};
use crate::storage::nbt::Tag;

/// `Climate.quantizeCoord`: `(long)(coord * 10000.0F)`.
fn quantize(coord: f32) -> i64 {
    (coord * 10000.0_f32) as i64
}

/// `Climate.unquantizeCoord`: `(float)coord / 10000.0F`.
fn unquantize(coord: i64) -> f32 {
    coord as f32 / 10000.0_f32
}

/// `Climate.Parameter.CODEC`: `ExtraCodecs.intervalCodec` over `floatRange(-2, 2)`. A
/// point is a bare float, an interval a `[min, max]` list (or `{min, max}` object).
/// Both ends round-trip through the quantised `long` representation.
pub fn climate_parameter() -> Codec {
    interval_codec(float_range(-2.0, 2.0), "min", "max", |min, max| {
        let (Tag::Float(min), Tag::Float(max)) = (min, max) else {
            return Err("Climate parameter bounds must be floats".to_string());
        };
        if min > max {
            return Err(format!(
                "Cannon construct interval, min > max ({min} > {max})"
            ));
        }
        Ok((
            Tag::Float(unquantize(quantize(*min))),
            Tag::Float(unquantize(quantize(*max))),
        ))
    })
}

/// `Codec.floatRange(0, 1).xmap(Climate::quantizeCoord, Climate::unquantizeCoord)`.
fn quantized_offset() -> Codec {
    float_range(0.0, 1.0).map_tag(|tag| match tag {
        Tag::Float(value) => Ok(Tag::Float(unquantize(quantize(value)))),
        other => Ok(other),
    })
}

/// `Climate.ParameterPoint.CODEC`.
pub fn climate_parameter_point() -> Codec {
    record(vec![
        req("temperature", climate_parameter()),
        req("humidity", climate_parameter()),
        req("continentalness", climate_parameter()),
        req("erosion", climate_parameter()),
        req("depth", climate_parameter()),
        req("weirdness", climate_parameter()),
        req("offset", quantized_offset()),
    ])
}

/// `NoiseSettings.CODEC` including `guardY`.
fn noise_settings() -> Codec {
    record(vec![
        req("min_y", int_range(MIN_Y, MAX_Y)),
        req("height", int_range(0, Y_SIZE)),
        req("size_horizontal", int_range(1, 4)),
        req("size_vertical", int_range(1, 4)),
    ])
    .validate(|tag| {
        let get = |name: &str| match tag {
            Tag::Compound(fields) => fields.iter().find_map(|(key, value)| match value {
                Tag::Int(v) if key == name => Some(*v),
                _ => None,
            }),
            _ => None,
        };
        let (Some(min_y), Some(height)) = (get("min_y"), get("height")) else {
            return Ok(());
        };
        if min_y + height > MAX_Y + 1 {
            Err(format!(
                "min_y + height cannot be higher than: {}",
                MAX_Y + 1
            ))
        } else if height % 16 != 0 {
            Err("height has to be a multiple of 16".to_string())
        } else if min_y % 16 != 0 {
            Err("min_y has to be a multiple of 16".to_string())
        } else {
            Ok(())
        }
    })
}

/// `NoiseRouter.CODEC`: fifteen density function fields.
fn noise_router() -> Codec {
    const FIELDS: [&str; 15] = [
        "barrier",
        "fluid_level_floodedness",
        "fluid_level_spread",
        "lava",
        "temperature",
        "vegetation",
        "continents",
        "erosion",
        "depth",
        "ridges",
        "preliminary_surface_level",
        "final_density",
        "vein_toggle",
        "vein_ridged",
        "vein_gap",
    ];
    record(
        FIELDS
            .iter()
            .map(|name| req(name, density_function_holder()))
            .collect(),
    )
}

/// `NoiseGeneratorSettings.DIRECT_CODEC`.
pub fn noise_generator_settings() -> Codec {
    record(vec![
        req("noise", noise_settings()),
        req("default_block", block_state()),
        req("default_fluid", block_state()),
        req("noise_router", noise_router()),
        req("surface_rule", rule_source()),
        req("spawn_target", list(climate_parameter_point())),
        req("sea_level", int_codec()),
        req("disable_mob_generation", bool_codec()),
        req("aquifers_enabled", bool_codec()),
        req("ore_veins_enabled", bool_codec()),
        req("legacy_random_source", bool_codec()),
    ])
}

// ---------------------------------------------------------------------------
// Surface rules
// ---------------------------------------------------------------------------

/// `SurfaceRules.ConditionSource.bootstrap` registrations.
const CONDITION_SOURCES: &[Variant] = &[
    ("biome", biome_condition),
    ("noise_threshold", noise_threshold),
    ("vertical_gradient", vertical_gradient),
    ("y_above", y_above),
    ("water", water),
    ("temperature", unit),
    ("steep", unit),
    ("not", not_condition),
    ("hole", unit),
    ("above_preliminary_surface", unit),
    ("stone_depth", stone_depth),
];

/// `SurfaceRules.RuleSource.bootstrap` registrations.
const RULE_SOURCES: &[Variant] = &[
    ("bandlands", unit),
    ("block", block_rule),
    ("sequence", sequence_rule),
    ("condition", test_rule),
];

/// `SurfaceRules.ConditionSource.CODEC`.
pub fn condition_source() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:worldgen/material_condition",
        CONDITION_SOURCES,
    )
}

/// `SurfaceRules.RuleSource.CODEC`.
pub fn rule_source() -> Codec {
    registry_dispatch("type", "minecraft:worldgen/material_rule", RULE_SOURCES)
}

/// `SurfaceRules.BiomeConditionSource.CODEC`: `biome_is` is a list of biome keys.
fn biome_condition() -> Codec {
    record(vec![req("biome_is", list(identifier_codec()))])
}

/// `SurfaceRules.NoiseThresholdConditionSource.CODEC`.
fn noise_threshold() -> Codec {
    record(vec![
        req("noise", identifier_codec()),
        req("min_threshold", double_codec()),
        req("max_threshold", double_codec()),
    ])
}

/// `SurfaceRules.VerticalGradientConditionSource.CODEC`.
fn vertical_gradient() -> Codec {
    record(vec![
        req("random_name", identifier_codec()),
        req("true_at_and_below", vertical_anchor()),
        req("false_at_and_above", vertical_anchor()),
    ])
}

/// `SurfaceRules.YConditionSource.CODEC`.
fn y_above() -> Codec {
    record(vec![
        req("anchor", vertical_anchor()),
        req("surface_depth_multiplier", int_range(-20, 20)),
        req("add_stone_depth", bool_codec()),
    ])
}

/// `SurfaceRules.WaterConditionSource.CODEC`.
fn water() -> Codec {
    record(vec![
        req("offset", int_codec()),
        req("surface_depth_multiplier", int_range(-20, 20)),
        req("add_stone_depth", bool_codec()),
    ])
}

/// `SurfaceRules.NotConditionSource.CODEC`: the inverted source is under `invert`.
fn not_condition() -> Codec {
    record(vec![req("invert", lazy(condition_source))])
}

/// `SurfaceRules.StoneDepthCheck.CODEC`.
fn stone_depth() -> Codec {
    record(vec![
        req("offset", int_codec()),
        req("add_surface_depth", bool_codec()),
        req("secondary_depth_range", int_codec()),
        req("surface_type", enum_codec(&["ceiling", "floor"])),
    ])
}

/// `SurfaceRules.BlockRuleSource.CODEC`.
fn block_rule() -> Codec {
    record(vec![req("result_state", block_state())])
}

/// `SurfaceRules.SequenceRuleSource.CODEC`.
fn sequence_rule() -> Codec {
    record(vec![req("sequence", list(lazy(rule_source)))])
}

/// `SurfaceRules.TestRuleSource.CODEC`.
fn test_rule() -> Codec {
    record(vec![
        req("if_true", condition_source()),
        req("then_run", lazy(rule_source)),
    ])
}
