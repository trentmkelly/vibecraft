//! Codecs shared by several registry element definitions: colors, text
//! components, sound events, music, int providers and particle options.

use serde_json::Value as Json;

use crate::registry_pipeline::codec::{
    self, describe, dispatch, float_codec, generic, holder_file, int_codec, list, non_negative_int,
    opt, opt_default, parse_identifier, record, req, string_codec, Codec, CodecResult, Field,
};
use crate::storage::nbt::Tag;

const SOUND_EVENT_REGISTRY: &str = "minecraft:sound_event";
const PARTICLE_TYPE_REGISTRY: &str = "minecraft:particle_type";

// ---------------------------------------------------------------------------
// Colors (ExtraCodecs)
// ---------------------------------------------------------------------------

/// `ExtraCodecs.hexColor(digits)` decoding: `#` plus exactly `digits` hex digits.
fn parse_hex_color(text: &str, digits: usize) -> CodecResult<i32> {
    let rest = text
        .strip_prefix('#')
        .ok_or_else(|| "Hex color must begin with #".to_string())?;
    if rest.len() != digits {
        return Err(format!(
            "Hex color is wrong size, expected {digits} digits but got {}",
            rest.len()
        ));
    }
    let value =
        u64::from_str_radix(rest, 16).map_err(|_| format!("Invalid color value: {text}"))?;
    Ok(value as u32 as i32)
}

/// `ARGB.colorFromFloat` for a list of 3 (`RGB`, alpha 1) or 4 (`ARGB`) floats.
fn color_from_floats(json: &Json, components: usize) -> CodecResult<i32> {
    let Json::Array(items) = json else {
        return Err(format!("Not a color: {}", describe(json)));
    };
    if items.len() != components {
        return Err(format!(
            "Input is not a list of {components} elements: {}",
            describe(json)
        ));
    }
    let mut floats = items
        .iter()
        .map(|item| {
            item.as_f64()
                .map(|value| value as f32)
                .ok_or_else(|| format!("Not a number: {}", describe(item)))
        })
        .collect::<CodecResult<Vec<f32>>>()?;
    if components == 3 {
        floats.insert(0, 1.0);
    }
    let channel = |value: f32| ((value * 255.0).floor() as i32) & 0xFF;
    Ok((channel(floats[0]) << 24)
        | (channel(floats[1]) << 16)
        | (channel(floats[2]) << 8)
        | channel(floats[3]))
}

/// Integer value or float-list alternative of `RGB_COLOR_CODEC`/`ARGB_COLOR_CODEC`.
fn color_int_alternatives(json: &Json, components: usize) -> CodecResult<i32> {
    match json {
        Json::Number(_) => codec::json_int_value(json),
        _ => color_from_floats(json, components),
    }
}

/// `ExtraCodecs.RGB_COLOR_CODEC`/`ARGB_COLOR_CODEC`: encodes as a plain int.
pub fn color_int_codec(components: usize) -> Codec {
    Codec::new(move |json, _| color_int_alternatives(json, components).map(Tag::Int))
}

/// `ExtraCodecs.STRING_RGB_COLOR`: canonical `#rrggbb`.
pub fn string_rgb_color() -> Codec {
    Codec::new(|json, _| {
        let value = match json {
            Json::String(text) => parse_hex_color(text, 6)?,
            other => color_int_alternatives(other, 3)?,
        };
        Ok(Tag::String(format!("#{:06x}", value & 0x00FF_FFFF)))
    })
}

/// `ExtraCodecs.STRING_ARGB_COLOR`: canonical `#aarrggbb`.
pub fn string_argb_color() -> Codec {
    Codec::new(|json, _| {
        let value = match json {
            Json::String(text) => parse_hex_color(text, 8)?,
            other => color_int_alternatives(other, 4)?,
        };
        Ok(Tag::String(format!("#{:08x}", value as u32)))
    })
}

// ---------------------------------------------------------------------------
// Text components (ComponentSerialization)
// ---------------------------------------------------------------------------

const NAMED_TEXT_COLORS: &[&str] = &[
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
];

/// `TextColor.CODEC`: a color name or `#RRGGBB` (re-serialised upper-case).
fn text_color() -> Codec {
    Codec::new(|json, _| {
        let text = match json {
            Json::String(text) => text,
            other => return Err(format!("Not a string: {}", describe(other))),
        };
        if let Some(hex) = text.strip_prefix('#') {
            let value =
                i64::from_str_radix(hex, 16).map_err(|_| format!("Invalid color value: {text}"))?;
            if (0..=0x00FF_FFFF).contains(&value) {
                Ok(Tag::String(format!("#{value:06X}")))
            } else {
                Err(format!("Color value out of range: {text}"))
            }
        } else if NAMED_TEXT_COLORS.contains(&text.as_str()) {
            Ok(Tag::String(text.clone()))
        } else {
            Err(format!("Invalid color name: {text}"))
        }
    })
}

/// `Style.Serializer.MAP_CODEC` fields.
fn style_fields() -> Vec<Field> {
    vec![
        opt("color", text_color()),
        opt("shadow_color", color_int_codec(4)),
        opt("bold", codec::bool_codec()),
        opt("italic", codec::bool_codec()),
        opt("underlined", codec::bool_codec()),
        opt("strikethrough", codec::bool_codec()),
        opt("obfuscated", codec::bool_codec()),
        opt("click_event", generic()),
        opt("hover_event", generic()),
        opt("insertion", string_codec()),
        opt("font", codec::identifier_codec()),
    ]
}

/// `Style.Serializer.CODEC`.
///
/// TODO(registry-pipeline-component): `click_event`/`hover_event` are converted
/// generically instead of through `ClickEvent.CODEC`/`HoverEvent.CODEC`.
pub fn style() -> Codec {
    record(style_fields())
}

/// Content and sibling keys of a text component (`ComponentContents` codecs).
fn component_fields() -> Vec<Field> {
    let mut fields = vec![
        opt("type", string_codec()),
        opt("text", string_codec()),
        opt("translate", string_codec()),
        opt("fallback", string_codec()),
        opt("with", generic()),
        opt("keybind", string_codec()),
        opt("selector", string_codec()),
        opt("separator", codec::lazy(component)),
        opt("score", generic()),
        opt("nbt", string_codec()),
        opt("source", string_codec()),
        opt("interpret", codec::bool_codec()),
        opt("block", string_codec()),
        opt("entity", string_codec()),
        opt("storage", string_codec()),
    ];
    fields.extend(style_fields());
    fields.push(opt("extra", list(codec::lazy(component))));
    fields
}

/// `ComponentSerialization.CODEC`.
///
/// A bare string is a literal; an object is validated field by field. A literal
/// object with nothing but `text` collapses to its string form exactly like the
/// Java encoder's compact literal.
///
/// TODO(registry-pipeline-component): `with`, `score`, and the click/hover events
/// are converted generically instead of through their typed codecs, and the
/// list-of-components sibling form is not re-shaped into an object; no vanilla
/// registry element uses those forms.
pub fn component() -> Codec {
    let object = record(component_fields());
    Codec::new(move |json, ctx| match json {
        Json::String(text) => Ok(Tag::String(text.clone())),
        Json::Array(items) if !items.is_empty() => list(codec::lazy(component)).parse(json, ctx),
        Json::Object(_) => match object.parse(json, ctx)? {
            Tag::Compound(fields)
                if fields.iter().all(|(key, _)| key == "text")
                    && fields.iter().any(|(key, _)| key == "text") =>
            {
                Ok(fields.into_iter().next().map_or(Tag::End, |(_, text)| text))
            }
            other => Ok(other),
        },
        other => Err(format!("Not a string, list or map: {}", describe(other))),
    })
}

// ---------------------------------------------------------------------------
// Sounds and music
// ---------------------------------------------------------------------------

/// `SoundEvent.DIRECT_CODEC`.
fn sound_event_direct() -> Codec {
    record(vec![
        req("sound_id", codec::identifier_codec()),
        codec::lenient_opt("range", float_codec()),
    ])
}

/// `SoundEvent.CODEC`: a `Holder<SoundEvent>` by id or inline.
pub fn sound_event() -> Codec {
    holder_file(SOUND_EVENT_REGISTRY, sound_event_direct())
}

/// `Music.CODEC`.
pub fn music() -> Codec {
    record(vec![
        req("sound", sound_event()),
        req("min_delay", non_negative_int()),
        req("max_delay", non_negative_int()),
        opt_default(
            "replace_current_music",
            codec::bool_codec(),
            Json::Bool(false),
        ),
    ])
}

// ---------------------------------------------------------------------------
// Int providers
// ---------------------------------------------------------------------------

fn int_field(tag: &Tag, name: &str) -> Option<i32> {
    match tag {
        Tag::Compound(fields) => fields.iter().find_map(|(key, value)| match value {
            Tag::Int(v) if key == name => Some(*v),
            _ => None,
        }),
        _ => None,
    }
}

/// `IntProvider.minInclusive()`/`maxInclusive()` of an encoded provider.
fn int_provider_bounds(tag: &Tag) -> Option<(i32, i32)> {
    match tag {
        Tag::Int(value) => Some((*value, *value)),
        Tag::Compound(fields) => {
            let kind = fields.iter().find_map(|(key, value)| match value {
                Tag::String(name) if key == "type" => Some(name.as_str()),
                _ => None,
            })?;
            match kind {
                "minecraft:uniform"
                | "minecraft:biased_to_bottom"
                | "minecraft:clamped"
                | "minecraft:clamped_normal" => Some((
                    int_field(tag, "min_inclusive")?,
                    int_field(tag, "max_inclusive")?,
                )),
                "minecraft:trapezoid" => Some((int_field(tag, "min")?, int_field(tag, "max")?)),
                "minecraft:weighted_list" => {
                    let distribution = fields.iter().find_map(|(key, value)| match value {
                        Tag::List(items) if key == "distribution" => Some(items),
                        _ => None,
                    })?;
                    distribution
                        .iter()
                        .filter_map(|entry| match entry {
                            Tag::Compound(entry_fields) => entry_fields
                                .iter()
                                .find(|(key, _)| key == "data")
                                .and_then(|(_, data)| int_provider_bounds(data)),
                            _ => None,
                        })
                        .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn ordered_bounds(
    min_key: &'static str,
    max_key: &'static str,
) -> impl Fn(&Tag) -> CodecResult<()> {
    move |tag| match (int_field(tag, min_key), int_field(tag, max_key)) {
        (Some(min), Some(max)) if max < min => Err(format!(
            "Max must be at least min, min_inclusive: {min}, max_inclusive: {max}"
        )),
        _ => Ok(()),
    }
}

fn int_provider_variant(kind: &str) -> CodecResult<Codec> {
    let ints = || codec::int_codec();
    Ok(match kind {
        "minecraft:constant" => record(vec![req("value", ints())]),
        "minecraft:uniform" | "minecraft:biased_to_bottom" => record(vec![
            req("min_inclusive", ints()),
            req("max_inclusive", ints()),
        ])
        .validate(ordered_bounds("min_inclusive", "max_inclusive")),
        "minecraft:clamped" => record(vec![
            req("source", codec::lazy(int_provider_any)),
            req("min_inclusive", ints()),
            req("max_inclusive", ints()),
        ])
        .validate(ordered_bounds("min_inclusive", "max_inclusive")),
        "minecraft:clamped_normal" => record(vec![
            req("mean", float_codec()),
            req("deviation", float_codec()),
            req("min_inclusive", ints()),
            req("max_inclusive", ints()),
        ])
        .validate(|tag| {
            match (
                int_field(tag, "min_inclusive"),
                int_field(tag, "max_inclusive"),
            ) {
                (Some(min), Some(max)) if max < min => {
                    Err(format!("Max must be larger than min: [{min}, {max}]"))
                }
                _ => Ok(()),
            }
        }),
        "minecraft:trapezoid" => record(vec![
            req("min", ints()),
            req("max", ints()),
            req("plateau", ints()),
        ])
        .validate(|tag| {
            let (Some(min), Some(max), Some(plateau)) = (
                int_field(tag, "min"),
                int_field(tag, "max"),
                int_field(tag, "plateau"),
            ) else {
                return Ok(());
            };
            if min > max {
                Err(format!("Max must be larger than min: [{min}, {max}]"))
            } else if plateau > max - min {
                Err(format!(
                    "Plateau can at most be the full span: [{min}, {max}]"
                ))
            } else {
                Ok(())
            }
        }),
        "minecraft:weighted_list" => record(vec![req(
            "distribution",
            list(record(vec![
                req("data", codec::lazy(int_provider_any)),
                req("weight", non_negative_int()),
            ]))
            .validate(|tag| match tag {
                Tag::List(items) if items.is_empty() => Err(
                    "Weighted list must contain at least one entry with non-zero weight"
                        .to_string(),
                ),
                _ => Ok(()),
            }),
        )]),
        other => return Err(format!("Unknown int provider type: {other}")),
    })
}

/// `IntProviders.CODEC`: a bare int, or a typed provider (`constant` collapses to
/// its int like the Java encoder).
fn int_provider_any() -> Codec {
    Codec::new(|json, ctx| match json {
        Json::Number(_) => int_codec().parse(json, ctx),
        Json::Object(_) => {
            let typed = dispatch("type", |id| int_provider_variant(&id.to_string()));
            match typed.parse(json, ctx)? {
                Tag::Compound(fields) => {
                    let is_constant = fields.iter().any(|(key, value)| {
                        key == "type" && *value == Tag::String("minecraft:constant".to_string())
                    });
                    if is_constant {
                        fields
                            .into_iter()
                            .find_map(|(key, value)| (key == "value").then_some(value))
                            .ok_or_else(|| "constant provider has no value".to_string())
                    } else {
                        Ok(Tag::Compound(fields))
                    }
                }
                other => Ok(other),
            }
        }
        other => Err(format!("Not a number or map: {}", describe(other))),
    })
}

/// `IntProviders.codec(min, max)`.
pub fn int_provider(min: i32, max: i32) -> Codec {
    int_provider_any().validate(move |tag| match int_provider_bounds(tag) {
        Some((low, _)) if low < min => {
            let (low, high) = int_provider_bounds(tag).unwrap_or((low, low));
            Err(format!("Value provider too low: {min} [{low}-{high}]"))
        }
        Some((low, high)) if high > max => {
            Err(format!("Value provider too high: {max} [{low}-{high}]"))
        }
        _ => Ok(()),
    })
}

// ---------------------------------------------------------------------------
// Particles
// ---------------------------------------------------------------------------

/// Particle types that carry option fields (`ParticleTypes.register(name, ..,
/// codec, streamCodec)`); all other types are `SimpleParticleType`s.
const PARTICLES_WITH_OPTIONS: &[&str] = &[
    "block",
    "block_marker",
    "dragon_breath",
    "dust",
    "dust_color_transition",
    "effect",
    "entity_effect",
    "falling_dust",
    "tinted_leaves",
    "sculk_charge",
    "flash",
    "instant_effect",
    "item",
    "vibration",
    "trail",
    "shriek",
    "dust_pillar",
    "block_crumble",
];

/// `ParticleTypes.CODEC`: `{"type": <particle_type>, ...options}`.
///
/// TODO(registry-pipeline-particle-options): the option fields of the 18 particle
/// types that have them are converted generically rather than through the typed
/// per-particle codecs (`DustParticleOptions.CODEC`, ...).
pub fn particle_options() -> Codec {
    Codec::new(|json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let raw = object
            .get("type")
            .ok_or_else(|| format!("No key type in MapLike[{}]", describe(json)))?;
        let type_id = parse_identifier(raw)?;
        let registry = crate::registry::Identifier::parse(PARTICLE_TYPE_REGISTRY)
            .map_err(|err| format!("invalid particle registry name: {err}"))?;
        if !ctx.builtin().contains_element(&registry, &type_id) {
            return Err(format!(
                "Unknown registry key in ResourceKey[minecraft:root / {registry}]: {type_id}"
            ));
        }
        let mut fields = vec![("type".to_string(), Tag::String(type_id.to_string()))];
        if PARTICLES_WITH_OPTIONS.contains(&type_id.path()) {
            for (key, value) in object {
                if key != "type" {
                    fields.push((key.clone(), codec::json_to_tag(value)));
                }
            }
        }
        Ok(Tag::Compound(fields))
    })
}
