//! Codec for `worldgen/configured_carver` (`ConfiguredWorldCarver.DIRECT_CODEC`) with
//! the carver configurations, `CarverDebugSettings` and `FloatProviders`.

use serde_json::json;

use crate::registry_pipeline::codec::{
    bool_codec, either, float_codec, float_range, holder_set, opt_default, positive_int, record,
    req, Codec, Field,
};
use crate::registry_pipeline::worldgen_common::{
    block_state, height_provider, registry_dispatch, vertical_anchor, Variant,
};
use crate::storage::nbt::Tag;

const BLOCK_REGISTRY: &str = "minecraft:block";

/// `ConfiguredWorldCarver.DIRECT_CODEC`: `{type, config}`.
pub fn configured_carver() -> Codec {
    registry_dispatch("type", "minecraft:worldgen/carver", CARVERS)
}

/// `WorldCarver` registrations; each carries its configuration under `config`.
const CARVERS: &[Variant] = &[
    ("cave", cave_config_field),
    ("nether_cave", cave_config_field),
    ("canyon", canyon_config_field),
];

fn cave_config_field() -> Codec {
    record(vec![req("config", cave_configuration())])
}

fn canyon_config_field() -> Codec {
    record(vec![req("config", canyon_configuration())])
}

// ---------------------------------------------------------------------------
// Float providers
// ---------------------------------------------------------------------------

/// `FloatProviders.CODEC`: a bare float (a constant) or a typed provider.
pub fn float_provider() -> Codec {
    let typed = registry_dispatch(
        "type",
        "minecraft:float_provider_type",
        FLOAT_PROVIDER_TYPES,
    );
    either(float_codec(), typed).map_tag(|tag| {
        Ok(crate::registry_pipeline::worldgen_common::collapse_variant(
            tag, "constant", "value",
        ))
    })
}

/// `FloatProviders.bootstrap` registrations.
const FLOAT_PROVIDER_TYPES: &[Variant] = &[
    ("constant", constant_float),
    ("uniform", uniform_float),
    ("clamped_normal", clamped_normal_float),
    ("trapezoid", trapezoid_float),
];

fn constant_float() -> Codec {
    record(vec![req("value", float_codec())])
}

/// `UniformFloat.MAP_CODEC`: `max_exclusive` must exceed `min_inclusive`.
fn uniform_float() -> Codec {
    record(vec![
        req("min_inclusive", float_codec()),
        req("max_exclusive", float_codec()),
    ])
    .validate(|tag| {
        match (
            float_of(tag, "min_inclusive"),
            float_of(tag, "max_exclusive"),
        ) {
            (Some(min), Some(max)) if max <= min => Err(format!(
                "Max must be larger than min, min: {min:?}, max: {max:?}"
            )),
            _ => Ok(()),
        }
    })
}

/// `ClampedNormalFloat.MAP_CODEC`.
fn clamped_normal_float() -> Codec {
    record(vec![
        req("mean", float_codec()),
        req("deviation", float_codec()),
        req("min", float_codec()),
        req("max", float_codec()),
    ])
    .validate(|tag| match (float_of(tag, "min"), float_of(tag, "max")) {
        (Some(min), Some(max)) if max < min => {
            Err(format!("Max must be larger than min: [{min:?}, {max:?}]"))
        }
        _ => Ok(()),
    })
}

/// `TrapezoidFloat.MAP_CODEC`.
fn trapezoid_float() -> Codec {
    record(vec![
        req("min", float_codec()),
        req("max", float_codec()),
        req("plateau", float_codec()),
    ])
    .validate(|tag| {
        let (Some(min), Some(max), Some(plateau)) = (
            float_of(tag, "min"),
            float_of(tag, "max"),
            float_of(tag, "plateau"),
        ) else {
            return Ok(());
        };
        if max < min {
            Err(format!("Max must be larger than min: [{min:?}, {max:?}]"))
        } else if plateau > max - min {
            Err(format!(
                "Plateau can at most be the full span: [{min:?}, {max:?}]"
            ))
        } else {
            Ok(())
        }
    })
}

fn float_of(tag: &Tag, name: &str) -> Option<f32> {
    match tag {
        Tag::Compound(fields) => fields.iter().find_map(|(key, value)| match value {
            Tag::Float(v) if key == name => Some(*v),
            _ => None,
        }),
        _ => None,
    }
}

/// The `[min, max]` range a float provider can produce (`FloatProvider.min()/max()`).
fn float_provider_range(tag: &Tag) -> Option<(f32, f32)> {
    match tag {
        Tag::Float(value) => Some((*value, *value)),
        Tag::Compound(_) => {
            if let (Some(min), Some(max)) = (
                float_of(tag, "min_inclusive"),
                float_of(tag, "max_exclusive"),
            ) {
                Some((min, max))
            } else {
                Some((float_of(tag, "min")?, float_of(tag, "max")?))
            }
        }
        _ => None,
    }
}

/// `FloatProviders.codec(minValue, maxValue)`.
fn float_provider_in(min_value: f32, max_value: f32) -> Codec {
    float_provider().validate(move |tag| match float_provider_range(tag) {
        Some((min, max)) if min < min_value => Err(format!(
            "Value provider too low: {min_value:?} [{min:?}-{max:?}]"
        )),
        Some((min, max)) if max > max_value => Err(format!(
            "Value provider too high: {max_value:?} [{min:?}-{max:?}]"
        )),
        _ => Ok(()),
    })
}

// ---------------------------------------------------------------------------
// Configurations
// ---------------------------------------------------------------------------

/// `CarverDebugSettings.CODEC`. Every state defaults to the acacia button state
/// (`DEFAULT.getAirState()` is used for all four).
fn debug_settings() -> Codec {
    let default_state = || json!({"Name": "minecraft:acacia_button"});
    record(vec![
        opt_default("debug_mode", bool_codec(), json!(false)),
        opt_default("air_state", block_state(), default_state()),
        opt_default("water_state", block_state(), default_state()),
        opt_default("lava_state", block_state(), default_state()),
        opt_default("barrier_state", block_state(), default_state()),
    ])
}

/// `CarverConfiguration.CODEC`, flattened into the concrete configuration.
fn base_fields() -> Vec<Field> {
    vec![
        req("probability", float_range(0.0, 1.0)),
        req("y", height_provider()),
        req("yScale", float_provider()),
        req("lava_level", vertical_anchor()),
        opt_default("debug_settings", debug_settings(), json!({})),
        req("replaceable", holder_set(BLOCK_REGISTRY, false)),
    ]
}

/// `CaveCarverConfiguration.CODEC`.
fn cave_configuration() -> Codec {
    let mut fields = base_fields();
    fields.extend([
        req("horizontal_radius_multiplier", float_provider()),
        req("vertical_radius_multiplier", float_provider()),
        req("floor_level", float_provider_in(-1.0, 1.0)),
    ]);
    record(fields)
}

/// `CanyonCarverConfiguration.CODEC`.
fn canyon_configuration() -> Codec {
    let mut fields = base_fields();
    fields.extend([
        req("vertical_rotation", float_provider()),
        req("shape", canyon_shape()),
    ]);
    record(fields)
}

/// `CanyonCarverConfiguration.CanyonShapeConfiguration.CODEC`.
fn canyon_shape() -> Codec {
    record(vec![
        req("distance_factor", float_provider()),
        req("thickness", float_provider()),
        req("width_smoothness", positive_int()),
        req("horizontal_radius_factor", float_provider()),
        req("vertical_radius_default_factor", float_codec()),
        req("vertical_radius_center_factor", float_codec()),
    ])
}
