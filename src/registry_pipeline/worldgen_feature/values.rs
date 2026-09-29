//! Value providers and small shared codecs used across worldgen features.

use serde_json::{json, Value as Json};

use super::{cached_codec, typed, Variant};
use crate::registry_pipeline::codec::{
    self, describe, either, enum_codec, float_codec, int_codec, int_range, json_int_value, lazy,
    list, non_negative_int, opt_default, record, req, Codec,
};
use crate::registry_pipeline::shared::int_provider;
use crate::storage::nbt::Tag;

/// `DimensionType.MIN_Y` (`MAX_Y - Y_SIZE + 1` with 12 packed Y bits).
pub const MIN_Y: i32 = -2032;
/// `DimensionType.MAX_Y`.
pub const MAX_Y: i32 = 2031;
/// `DimensionType.Y_SIZE`.
pub const Y_SIZE: i32 = 4064;

/// `IntProviders.CODEC` (no bounds).
pub fn any_int_provider() -> Codec {
    int_provider(i32::MIN, i32::MAX)
}

/// `IntProviders.NON_NEGATIVE_CODEC`.
pub fn non_negative_int_provider() -> Codec {
    int_provider(0, i32::MAX)
}

/// `IntProviders.POSITIVE_CODEC`.
pub fn positive_int_provider() -> Codec {
    int_provider(1, i32::MAX)
}

/// `Vec3i.CODEC`/`BlockPos.CODEC`: a three element int list (an `IntArrayTag` in NBT).
pub fn block_pos() -> Codec {
    Codec::new(|json, _| {
        let Json::Array(items) = json else {
            return Err(format!("Not a list: {}", describe(json)));
        };
        if items.len() != 3 {
            return Err(format!(
                "Input is not a list of 3 ints, was {}",
                describe(json)
            ));
        }
        items
            .iter()
            .map(json_int_value)
            .collect::<Result<Vec<_>, _>>()
            .map(Tag::IntArray)
    })
}

/// `Vec3i.offsetCodec(max_offset_per_axis)`.
pub fn offset_codec(max_offset_per_axis: i32) -> Codec {
    block_pos().validate(move |tag| match tag {
        Tag::IntArray(v) if v.iter().all(|c| c.abs() < max_offset_per_axis) => Ok(()),
        Tag::IntArray(v) => Err(format!(
            "Position out of range, expected at most {max_offset_per_axis}: Vec3i{{x={}, y={}, z={}}}",
            v[0], v[1], v[2]
        )),
        _ => Ok(()),
    })
}

/// `Vec3i.offsetCodec(16).optionalFieldOf("offset", Vec3i.ZERO)`.
pub fn offset_field() -> codec::Field {
    opt_default("offset", offset_codec(16), json!([0, 0, 0]))
}

const DIRECTIONS: &[&str] = &["down", "up", "north", "south", "west", "east"];

/// `Direction.CODEC`.
pub fn direction() -> Codec {
    enum_codec(DIRECTIONS)
}

/// `Direction.VERTICAL_CODEC`.
pub fn vertical_direction() -> Codec {
    direction().validate(|tag| match tag {
        Tag::String(name) if name == "up" || name == "down" => Ok(()),
        Tag::String(name) => Err(format!("Expected a vertical direction, found {name}")),
        _ => Ok(()),
    })
}

const HEIGHTMAP_TYPES: &[&str] = &[
    "WORLD_SURFACE_WG",
    "WORLD_SURFACE",
    "OCEAN_FLOOR_WG",
    "OCEAN_FLOOR",
    "MOTION_BLOCKING",
    "MOTION_BLOCKING_NO_LEAVES",
];

/// `Heightmap.Types.CODEC`.
pub fn heightmap_type() -> Codec {
    enum_codec(HEIGHTMAP_TYPES)
}

/// `Codec.floatRange(0, 1)`, the probability codec used throughout.
pub fn probability() -> Codec {
    codec::float_range(0.0, 1.0)
}

/// `Codec.xor(first, second)`: exactly one alternative must parse.
fn xor(alternatives: Vec<Codec>) -> Codec {
    Codec::new(move |json, ctx| {
        let mut parsed = Vec::new();
        let mut errors = Vec::new();
        for alternative in &alternatives {
            match alternative.parse(json, ctx) {
                Ok(tag) => parsed.push(tag),
                Err(error) => errors.push(error),
            }
        }
        match parsed.len() {
            1 => Ok(parsed.remove(0)),
            0 => Err(format!(
                "Failed to parse either. First: {}; Second: {}",
                errors[0],
                errors.get(1).map_or("", String::as_str)
            )),
            _ => Err(format!(
                "Both alternatives read successfully, can not pick the correct one; input: {}",
                describe(json)
            )),
        }
    })
}

cached_codec! {
    /// `VerticalAnchor.CODEC`: one of `{absolute}`, `{above_bottom}`, `{below_top}`.
    pub fn vertical_anchor() -> Codec {
        let anchor = |name: &'static str| record(vec![req(name, int_range(MIN_Y, MAX_Y))]);
        xor(vec![anchor("absolute"), anchor("above_bottom"), anchor("below_top")])
    }
}

/// `WeightedList.nonEmptyCodec(element)`: `[{data, weight}, ...]`.
pub fn non_empty_weighted_list(element: Codec) -> Codec {
    list(record(vec![
        req("data", element),
        req("weight", non_negative_int()),
    ]))
    .validate(|tag| match tag {
        Tag::List(items) if items.is_empty() => {
            Err("Weighted list must contain at least one entry with non-zero weight".to_string())
        }
        _ => Ok(()),
    })
}

/// `ExtraCodecs.nonEmptyList(codec.listOf())`.
pub fn non_empty_list(element: Codec) -> Codec {
    list(element).validate(|tag| match tag {
        Tag::List(items) if items.is_empty() => Err("List must have contents".to_string()),
        _ => Ok(()),
    })
}

// ---------------------------------------------------------------------------
// Float providers
// ---------------------------------------------------------------------------

fn float_field(tag: &Tag, name: &str) -> Option<f32> {
    match tag {
        Tag::Compound(fields) => fields.iter().find_map(|(key, value)| match value {
            Tag::Float(v) if key == name => Some(*v),
            _ => None,
        }),
        _ => None,
    }
}

const FLOAT_PROVIDER_TYPES: &[Variant] = &[
    ("constant", || record(vec![req("value", float_codec())])),
    ("uniform", || {
        record(vec![
            req("min_inclusive", float_codec()),
            req("max_exclusive", float_codec()),
        ])
        .validate(|tag| {
            match (float_field(tag, "min_inclusive"), float_field(tag, "max_exclusive")) {
                (Some(min), Some(max)) if max <= min => {
                    Err(format!("Max must be larger than min, min: {min}, max: {max}"))
                }
                _ => Ok(()),
            }
        })
    }),
    ("clamped_normal", || {
        record(vec![
            req("mean", float_codec()),
            req("deviation", float_codec()),
            req("min", float_codec()),
            req("max", float_codec()),
        ])
        .validate(|tag| match (float_field(tag, "min"), float_field(tag, "max")) {
            (Some(min), Some(max)) if max < min => {
                Err(format!("Max must be larger than min: [{min}, {max}]"))
            }
            _ => Ok(()),
        })
    }),
    ("trapezoid", || {
        record(vec![
            req("min", float_codec()),
            req("max", float_codec()),
            req("plateau", float_codec()),
        ])
        .validate(|tag| {
            let (Some(min), Some(max), Some(plateau)) = (
                float_field(tag, "min"),
                float_field(tag, "max"),
                float_field(tag, "plateau"),
            ) else {
                return Ok(());
            };
            if max < min {
                Err(format!("Max must be larger than min: [{min}, {max}]"))
            } else if plateau > max - min {
                Err(format!("Plateau can at most be the full span: [{min}, {max}]"))
            } else {
                Ok(())
            }
        })
    }),
];

/// `FloatProvider.getMinValue()`/`getMaxValue()` of an encoded provider.
fn float_provider_bounds(tag: &Tag) -> Option<(f32, f32)> {
    match tag {
        Tag::Float(v) => Some((*v, *v)),
        Tag::Compound(_) => {
            let Tag::Compound(fields) = tag else { return None };
            let kind = fields.iter().find_map(|(key, value)| match value {
                Tag::String(name) if key == "type" => Some(name.as_str()),
                _ => None,
            })?;
            match kind {
                "minecraft:uniform" => Some((
                    float_field(tag, "min_inclusive")?,
                    float_field(tag, "max_exclusive")?,
                )),
                "minecraft:clamped_normal" | "minecraft:trapezoid" => {
                    Some((float_field(tag, "min")?, float_field(tag, "max")?))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

cached_codec! {
    /// `FloatProviders.CODEC`: a bare float or a typed provider (`constant`
    /// collapses to its float like the Java encoder).
    pub fn any_float_provider() -> Codec {
        let typed_provider = typed("type", "minecraft:float_provider_type", FLOAT_PROVIDER_TYPES)
            .map_tag(|tag| match tag {
                Tag::Compound(fields)
                    if fields.iter().any(|(key, value)| {
                        key == "type" && *value == Tag::String("minecraft:constant".to_string())
                    }) =>
                {
                    fields
                        .into_iter()
                        .find_map(|(key, value)| (key == "value").then_some(value))
                        .ok_or_else(|| "constant provider has no value".to_string())
                }
                other => Ok(other),
            });
        either(float_codec(), typed_provider)
    }
}

/// `FloatProviders.codec(min, max)`.
pub fn float_provider(min: f32, max: f32) -> Codec {
    any_float_provider().validate(move |tag| match float_provider_bounds(tag) {
        Some((low, high)) if low < min => {
            Err(format!("Value provider too low: {min} [{low}-{high}]"))
        }
        Some((low, high)) if high > max => {
            Err(format!("Value provider too high: {max} [{low}-{high}]"))
        }
        _ => Ok(()),
    })
}

// ---------------------------------------------------------------------------
// Height providers
// ---------------------------------------------------------------------------

/// `Codec.intRange(1, Integer.MAX_VALUE).optionalFieldOf("inner", 1)`.
fn inner_field() -> codec::Field {
    opt_default("inner", int_range(1, i32::MAX), json!(1))
}

fn height_bounds() -> Vec<codec::Field> {
    vec![
        req("min_inclusive", vertical_anchor()),
        req("max_inclusive", vertical_anchor()),
    ]
}

const HEIGHT_PROVIDER_TYPES: &[Variant] = &[
    ("constant", || record(vec![req("value", vertical_anchor())])),
    ("uniform", || record(height_bounds())),
    ("biased_to_bottom", || {
        let mut fields = height_bounds();
        fields.push(inner_field());
        record(fields)
    }),
    ("very_biased_to_bottom", || {
        let mut fields = height_bounds();
        fields.push(inner_field());
        record(fields)
    }),
    ("trapezoid", || {
        let mut fields = height_bounds();
        fields.push(opt_default("plateau", int_codec(), json!(0)));
        record(fields)
    }),
    ("weighted_list", || {
        record(vec![req(
            "distribution",
            non_empty_weighted_list(lazy(height_provider)),
        )])
    }),
];

cached_codec! {
    /// `HeightProvider.CODEC`: a bare vertical anchor (a constant provider) or a typed
    /// provider.
    pub fn height_provider() -> Codec {
        let typed_provider = typed(
            "type",
            "minecraft:height_provider_type",
            HEIGHT_PROVIDER_TYPES,
        )
        .map_tag(|tag| match tag {
            Tag::Compound(fields)
                if fields.iter().any(|(key, value)| {
                    key == "type" && *value == Tag::String("minecraft:constant".to_string())
                }) =>
            {
                fields
                    .into_iter()
                    .find_map(|(key, value)| (key == "value").then_some(value))
                    .ok_or_else(|| "constant height has no value".to_string())
            }
            other => Ok(other),
        });
        either(vertical_anchor(), typed_provider)
    }
}

// ---------------------------------------------------------------------------
// Inclusive int ranges
// ---------------------------------------------------------------------------

/// `InclusiveRange.create` over parsed bounds, encoded like `intervalCodec`: a single
/// value when both bounds are equal, otherwise the two element array.
fn make_interval(min: i32, max: i32) -> Result<Tag, String> {
    if min > max {
        return Err("min_inclusive must be less than or equal to max_inclusive".to_string());
    }
    Ok(if min == max {
        Tag::Int(min)
    } else {
        Tag::List(vec![Tag::Int(min), Tag::Int(max)])
    })
}

/// `InclusiveRange.codec(Codec.INT, min_allowed, max_allowed)`: a lone int, a
/// `[min, max]` array or a `{min_inclusive, max_inclusive}` object.
pub fn inclusive_int_range(min_allowed: i32, max_allowed: i32) -> Codec {
    let object = record(vec![
        req("min_inclusive", int_codec()),
        req("max_inclusive", int_codec()),
    ]);
    Codec::new(move |json, ctx| {
        let (min, max) = match json {
            Json::Array(items) if items.len() == 2 => {
                (json_int_value(&items[0])?, json_int_value(&items[1])?)
            }
            Json::Array(_) => {
                return Err(format!("Input is not a list of 2 elements: {}", describe(json)))
            }
            Json::Object(_) => match object.parse(json, ctx)? {
                Tag::Compound(fields) => match fields.as_slice() {
                    [(_, Tag::Int(min)), (_, Tag::Int(max))] => (*min, *max),
                    _ => return Err("Malformed interval".to_string()),
                },
                _ => return Err("Malformed interval".to_string()),
            },
            _ => {
                let value = json_int_value(json)?;
                (value, value)
            }
        };
        let tag = make_interval(min, max)?;
        if min < min_allowed {
            return Err(format!(
                "Range limit too low, expected at least {min_allowed} [{min}-{max}]"
            ));
        }
        if max > max_allowed {
            return Err(format!(
                "Range limit too high, expected at most {max_allowed} [{min}-{max}]"
            ));
        }
        Ok(tag)
    })
}
