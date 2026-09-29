//! Building blocks shared by the worldgen data codecs (`worldgen/noise`,
//! `worldgen/density_function`, `worldgen/template_pool`, `worldgen/structure`, ...).
//!
//! Every item mirrors a small Java helper that several of those codecs use:
//! `BlockState.CODEC`, `VerticalAnchor.CODEC`, `HeightProvider.CODEC`,
//! `WeightedList.codec`, `ExtraCodecs.nonEmptyList`, `MapCodec.unit`, registry
//! `byNameCodec().dispatch(...)`, and so on.

use serde_json::Value as Json;

use crate::block_states::{block_state_entry, default_state_properties};
use crate::registry_pipeline::codec::{
    self, describe, dispatch, either, enum_codec, holder_fixed, int_codec, int_range, list,
    non_negative_int, opt_default, record, req, Codec, CodecResult, Field,
};
use crate::storage::nbt::Tag;

/// `DimensionType.BITS_FOR_Y` derived size: `(1 << 12) - 32`.
pub const Y_SIZE: i32 = 4064;
/// `DimensionType.MAX_Y`.
pub const MAX_Y: i32 = (Y_SIZE >> 1) - 1;
/// `DimensionType.MIN_Y`.
pub const MIN_Y: i32 = MAX_Y - Y_SIZE + 1;

/// One entry of a registry-keyed dispatch table: the type path under `minecraft:` and
/// the factory of the variant's `MapCodec`.
pub type Variant = (&'static str, fn() -> Codec);

/// `MapCodec.unit(...)`: reads nothing and writes nothing, but (like every
/// `MapCodec.codec()`) still requires the input to be a map.
pub fn unit() -> Codec {
    Codec::new(|json, _| match json {
        Json::Object(_) => Ok(Tag::Compound(Vec::new())),
        _ => Err(format!("Not a map: {}", describe(json))),
    })
}

/// `<registry>.byNameCodec().dispatch(typeKey, ...)` over a static table of
/// variants. `registry` names the Java registry in the "unknown key" message.
pub fn registry_dispatch(
    type_key: &'static str,
    registry: &'static str,
    variants: &'static [Variant],
) -> Codec {
    dispatch(type_key, move |id| {
        let full = id.to_string();
        variants
            .iter()
            .find(|(name, _)| full.strip_prefix("minecraft:") == Some(*name))
            .map(|(_, factory)| factory())
            .ok_or_else(|| {
                format!("Unknown registry key in ResourceKey[minecraft:root / {registry}]: {id}")
            })
    })
}

/// `ExtraCodecs.nonEmptyList(codec.listOf())`.
pub fn non_empty_list(element: Codec) -> Codec {
    list(element).validate(|tag| match tag {
        Tag::List(items) if items.is_empty() => Err("List must have contents".to_string()),
        _ => Ok(()),
    })
}

/// `Weighted.codec(elementCodec)`: `{data: <element>, weight: <int >= 0>}`.
pub fn weighted(element: Codec) -> Codec {
    record(vec![
        req("data", element),
        req("weight", non_negative_int()),
    ])
}

/// `WeightedList.codec(elementCodec)`: a list of [`weighted`] entries.
pub fn weighted_list(element: Codec) -> Codec {
    list(weighted(element))
}

/// `WeightedList.nonEmptyCodec(elementCodec)`.
pub fn non_empty_weighted_list(element: Codec) -> Codec {
    weighted_list_of(weighted(element))
}

/// `WeightedList.nonEmptyCodec(mapCodec)` where `entry` is the already flattened
/// weighted record.
pub fn weighted_list_of(entry: Codec) -> Codec {
    list(entry).validate(|tag| match tag {
        Tag::List(items) if items.is_empty() => {
            Err("Weighted list must contain at least one entry with non-zero weight".to_string())
        }
        _ => Ok(()),
    })
}

/// Fields of `Weighted.codec(MapCodec)`: the element's own fields plus `weight`.
pub fn flat_weighted(mut element_fields: Vec<Field>) -> Codec {
    element_fields.push(req("weight", non_negative_int()));
    record(element_fields)
}

/// `BuiltInRegistries.BLOCK.byNameCodec()`.
pub fn block_id() -> Codec {
    holder_fixed("minecraft:block")
}

/// `BlockState.CODEC` (`StateHolder.codec`): `{Name, Properties?}`.
///
/// `Properties` is a lenient optional field; each property falls back to the block's
/// default value when it is absent or not a valid value, unknown properties are
/// ignored, and the encoder writes every property of a non-singleton block in name
/// order (`StateDefinition.createCodec` appends properties sorted by name).
pub fn block_state() -> Codec {
    Codec::new(|json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let name = object
            .get("Name")
            .ok_or_else(|| format!("No key Name in MapLike[{}]", describe(json)))?;
        let Tag::String(block) = block_id().parse(name, ctx)? else {
            return Err("block name did not encode to a string".to_string());
        };
        let mut out = vec![("Name".to_string(), Tag::String(block.clone()))];
        let Some(entry) = block_state_entry(&block) else {
            return Ok(Tag::Compound(out));
        };
        if entry.properties.is_empty() {
            return Ok(Tag::Compound(out));
        }
        let requested = match object.get("Properties") {
            Some(Json::Object(properties)) => Some(properties),
            _ => None,
        };
        let defaults = default_state_properties(&block).unwrap_or_default();
        let mut properties: Vec<(String, Tag)> = entry
            .properties
            .iter()
            .map(|property| {
                let default = defaults
                    .iter()
                    .find_map(|(name, value)| (*name == property.name).then_some(*value))
                    .unwrap_or(property.values[0]);
                let value = requested
                    .and_then(|map| map.get(property.name))
                    .and_then(Json::as_str)
                    .filter(|value| property.values.contains(value))
                    .unwrap_or(default);
                (property.name.to_string(), Tag::String(value.to_string()))
            })
            .collect();
        properties.sort_by(|a, b| a.0.cmp(&b.0));
        out.push(("Properties".to_string(), Tag::Compound(properties)));
        Ok(Tag::Compound(out))
    })
}

/// `CompoundTag.CODEC`: any map, kept as an NBT compound.
pub fn compound_tag() -> Codec {
    Codec::new(|json, _| match json {
        Json::Object(_) => Ok(codec::json_to_tag(json)),
        _ => Err(format!("Not a compound tag: {json}")),
    })
}

/// `ExtraCodecs.intervalCodec(point, lower, upper, makeInterval, ...)`.
///
/// Decodes a bare point (min = max), a `[min, max]` list or a `{lower, upper}`
/// object, passes the bounds through `make_interval` (which validates them and yields
/// the stored bounds) and encodes equal bounds as the bare point and everything else
/// as the list form.
pub fn interval_codec(
    point: Codec,
    lower: &'static str,
    upper: &'static str,
    make_interval: impl Fn(&Tag, &Tag) -> CodecResult<(Tag, Tag)> + 'static,
) -> Codec {
    let make = move |min: &Tag, max: &Tag| -> CodecResult<Tag> {
        let (min, max) = make_interval(min, max)?;
        Ok(if min == max {
            min
        } else {
            Tag::List(vec![min, max])
        })
    };
    Codec::new(move |json, ctx| match json {
        Json::Array(items) => {
            if items.len() != 2 {
                return Err(format!(
                    "List {} should have size 2 but it was {}.",
                    describe(json),
                    items.len()
                ));
            }
            make(&point.parse(&items[0], ctx)?, &point.parse(&items[1], ctx)?)
        }
        Json::Object(_) => {
            let bounds = record(vec![req(lower, point.clone()), req(upper, point.clone())]);
            match bounds.parse(json, ctx)? {
                Tag::Compound(fields) if fields.len() == 2 => make(&fields[0].1, &fields[1].1),
                _ => Err("interval did not decode to two bounds".to_string()),
            }
        }
        _ => {
            let value = point.parse(json, ctx)?;
            make(&value, &value)
        }
    })
}

/// `NOISE_VALUE_CODEC`-style `Codec.doubleRange`.
pub fn double_in(min: f64, max: f64) -> Codec {
    codec::double_range(min, max)
}

/// `Codec.xor(...)` over single-key records: exactly one alternative may succeed.
fn xor_keys(keys: &'static [&'static str], value: fn() -> Codec) -> Codec {
    Codec::new(move |json, ctx| {
        let mut parsed: Vec<Tag> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for key in keys {
            match record(vec![req(key, value())]).parse(json, ctx) {
                Ok(tag) => parsed.push(tag),
                Err(err) => errors.push(err),
            }
        }
        match parsed.len() {
            1 => Ok(parsed.remove(0)),
            0 => Err(format!(
                "Failed to parse either. First: {}; Second: {}",
                errors.first().cloned().unwrap_or_default(),
                errors.last().cloned().unwrap_or_default()
            )),
            _ => Err(format!(
                "Both alternatives read successfully, can not pick the correct one; input: {}",
                describe(json)
            )),
        }
    })
}

fn anchor_offset() -> Codec {
    int_range(MIN_Y, MAX_Y)
}

/// `VerticalAnchor.CODEC`: exactly one of `absolute`, `above_bottom`, `below_top`.
pub fn vertical_anchor() -> Codec {
    xor_keys(&["absolute", "above_bottom", "below_top"], anchor_offset)
}

/// Height provider types (`HeightProviderType`).
const HEIGHT_PROVIDERS: &[Variant] = &[
    ("constant", height_constant),
    ("uniform", height_uniform),
    ("biased_to_bottom", height_biased_to_bottom),
    ("very_biased_to_bottom", height_biased_to_bottom),
    ("trapezoid", height_trapezoid),
    ("weighted_list", height_weighted_list),
];

fn height_constant() -> Codec {
    record(vec![req("value", vertical_anchor())])
}

fn height_uniform() -> Codec {
    record(vec![
        req("min_inclusive", vertical_anchor()),
        req("max_inclusive", vertical_anchor()),
    ])
}

/// `BiasedToBottomHeight`/`VeryBiasedToBottomHeight`: `inner` is `intRange(1, MAX)`.
fn height_biased_to_bottom() -> Codec {
    record(vec![
        req("min_inclusive", vertical_anchor()),
        req("max_inclusive", vertical_anchor()),
        opt_default("inner", int_range(1, i32::MAX), Json::from(1)),
    ])
}

fn height_trapezoid() -> Codec {
    record(vec![
        req("min_inclusive", vertical_anchor()),
        req("max_inclusive", vertical_anchor()),
        opt_default("plateau", int_codec(), Json::from(0)),
    ])
}

fn height_weighted_list() -> Codec {
    record(vec![req(
        "distribution",
        non_empty_weighted_list(codec::lazy(height_provider)),
    )])
}

/// `HeightProvider.CODEC`: a bare [`vertical_anchor`] (a constant) or a typed provider.
/// The constant provider is written back as its anchor.
pub fn height_provider() -> Codec {
    let typed = registry_dispatch("type", "minecraft:height_provider_type", HEIGHT_PROVIDERS);
    either(vertical_anchor(), typed).map_tag(|tag| Ok(collapse_constant_height(tag)))
}

/// `xmap` encode side of "a variant that collapses to its single field": when `tag` is
/// a compound of type `minecraft:<variant>`, returns the value of `field` instead.
pub fn collapse_variant(tag: Tag, variant: &str, field: &str) -> Tag {
    if let Tag::Compound(fields) = &tag {
        let full = format!("minecraft:{variant}");
        let is_variant = fields
            .iter()
            .any(|(key, value)| key == "type" && *value == Tag::String(full.clone()));
        if is_variant {
            if let Some((_, value)) = fields.iter().find(|(key, _)| key == field) {
                return value.clone();
            }
        }
    }
    tag
}

fn collapse_constant_height(tag: Tag) -> Tag {
    collapse_variant(tag, "constant", "value")
}

/// `Heightmap.Types.CODEC` (serialised names are upper case).
pub fn heightmap_types() -> Codec {
    enum_codec(&[
        "WORLD_SURFACE_WG",
        "WORLD_SURFACE",
        "OCEAN_FLOOR_WG",
        "OCEAN_FLOOR",
        "MOTION_BLOCKING",
        "MOTION_BLOCKING_NO_LEAVES",
    ])
}

/// `Vec3i.offsetCodec(maxOffsetPerAxis)`: a three-int list with `|axis| < max`.
pub fn vec3i_offset(max_offset_per_axis: i32) -> Codec {
    list(int_codec()).validate(move |tag| {
        let Tag::List(items) = tag else {
            return Ok(());
        };
        if items.len() != 3 {
            return Err(format!(
                "Input does not have enough elements; expected 3, got {}",
                items.len()
            ));
        }
        let ints: Vec<i32> = items
            .iter()
            .filter_map(|item| match item {
                Tag::Int(value) => Some(*value),
                _ => None,
            })
            .collect();
        if ints
            .iter()
            .any(|v| v.unsigned_abs() as i64 >= i64::from(max_offset_per_axis))
        {
            return Err(format!(
                "Position out of range, expected at most {max_offset_per_axis}: {ints:?}"
            ));
        }
        Ok(())
    })
}
