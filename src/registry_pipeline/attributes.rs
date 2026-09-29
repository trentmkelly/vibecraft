//! Environment attributes: the attribute registry, per-type value codecs, modifier
//! libraries, and the `EnvironmentAttributeMap` codec used by biomes and dimension
//! types. Timeline tracks reuse the same per-attribute value/argument codecs.
//!
//! Mirrors `EnvironmentAttributes` (the attribute definitions), `AttributeTypes`
//! (value codecs and modifier libraries) and `EnvironmentAttributeMap` in
//! `net.minecraft.world.attribute`.

use serde_json::Value as Json;

use crate::registry::Identifier;
use crate::registry_pipeline::codec::{
    self, bool_codec, describe, double_codec, either, enum_codec, float_codec, float_range,
    int_codec, list, opt, opt_default, record, req, Codec, CodecResult,
};
use crate::registry_pipeline::shared::{
    color_int_codec, component, music, particle_options, sound_event, string_argb_color,
    string_rgb_color,
};
use crate::storage::nbt::Tag;

/// `AttributeTypes`: the value type of an attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Boolean,
    TriState,
    Float,
    AngleDegrees,
    RgbColor,
    ArgbColor,
    MoonPhase,
    Activity,
    BedRule,
    Particle,
    AmbientParticles,
    BackgroundMusic,
    AmbientSounds,
}

/// `AttributeRange`: validation applied to float attribute values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueRange {
    Any,
    Unit,
    NonNegative,
    Between(f32, f32),
}

impl ValueRange {
    fn bounds(self) -> Option<(f32, f32)> {
        match self {
            Self::Any => None,
            Self::Unit => Some((0.0, 1.0)),
            Self::NonNegative => Some((0.0, f32::INFINITY)),
            Self::Between(min, max) => Some((min, max)),
        }
    }
}

/// One entry of `BuiltInRegistries.ENVIRONMENT_ATTRIBUTE`.
#[derive(Debug, Clone, Copy)]
pub struct AttributeDef {
    /// Path inside the `minecraft` namespace, e.g. `visual/sky_color`.
    pub id: &'static str,
    /// Value type.
    pub ty: AttributeType,
    /// `EnvironmentAttribute.isSyncable`.
    pub syncable: bool,
    /// `EnvironmentAttribute.isPositional`.
    pub positional: bool,
    /// `EnvironmentAttribute.valueRange`.
    pub range: ValueRange,
}

const fn attr(
    id: &'static str,
    ty: AttributeType,
    syncable: bool,
    positional: bool,
    range: ValueRange,
) -> AttributeDef {
    AttributeDef {
        id,
        ty,
        syncable,
        positional,
        range,
    }
}

/// Every environment attribute, in `EnvironmentAttributes` declaration order.
pub const ATTRIBUTES: &[AttributeDef] = &[
    attr(
        "visual/fog_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/fog_start_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/fog_end_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::NonNegative,
    ),
    attr(
        "visual/sky_fog_end_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::NonNegative,
    ),
    attr(
        "visual/cloud_fog_end_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::NonNegative,
    ),
    attr(
        "visual/water_fog_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/water_fog_start_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/water_fog_end_distance",
        AttributeType::Float,
        true,
        true,
        ValueRange::NonNegative,
    ),
    attr(
        "visual/sky_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/sunrise_sunset_color",
        AttributeType::ArgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/cloud_color",
        AttributeType::ArgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/cloud_height",
        AttributeType::Float,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/sun_angle",
        AttributeType::AngleDegrees,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/moon_angle",
        AttributeType::AngleDegrees,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/star_angle",
        AttributeType::AngleDegrees,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/moon_phase",
        AttributeType::MoonPhase,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/star_brightness",
        AttributeType::Float,
        true,
        true,
        ValueRange::Unit,
    ),
    attr(
        "visual/block_light_tint",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/sky_light_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/sky_light_factor",
        AttributeType::Float,
        true,
        true,
        ValueRange::Unit,
    ),
    attr(
        "visual/night_vision_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/ambient_light_color",
        AttributeType::RgbColor,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/default_dripstone_particle",
        AttributeType::Particle,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "visual/ambient_particles",
        AttributeType::AmbientParticles,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "audio/background_music",
        AttributeType::BackgroundMusic,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "audio/music_volume",
        AttributeType::Float,
        true,
        true,
        ValueRange::Unit,
    ),
    attr(
        "audio/ambient_sounds",
        AttributeType::AmbientSounds,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "audio/firefly_bush_sounds",
        AttributeType::Boolean,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/sky_light_level",
        AttributeType::Float,
        true,
        false,
        ValueRange::Between(0.0, 15.0),
    ),
    attr(
        "gameplay/can_start_raid",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/water_evaporates",
        AttributeType::Boolean,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/bed_rule",
        AttributeType::BedRule,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/respawn_anchor_works",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/nether_portal_spawns_piglin",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/fast_lava",
        AttributeType::Boolean,
        true,
        false,
        ValueRange::Any,
    ),
    attr(
        "gameplay/increased_fire_burnout",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/eyeblossom_open",
        AttributeType::TriState,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/turtle_egg_hatch_chance",
        AttributeType::Float,
        false,
        true,
        ValueRange::Unit,
    ),
    attr(
        "gameplay/piglins_zombify",
        AttributeType::Boolean,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/snow_golem_melts",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/creaking_active",
        AttributeType::Boolean,
        true,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/surface_slime_spawn_chance",
        AttributeType::Float,
        false,
        true,
        ValueRange::Unit,
    ),
    attr(
        "gameplay/cat_waking_up_gift_chance",
        AttributeType::Float,
        false,
        true,
        ValueRange::Unit,
    ),
    attr(
        "gameplay/bees_stay_in_hive",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/monsters_burn",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/can_pillager_patrol_spawn",
        AttributeType::Boolean,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/villager_activity",
        AttributeType::Activity,
        false,
        true,
        ValueRange::Any,
    ),
    attr(
        "gameplay/baby_villager_activity",
        AttributeType::Activity,
        false,
        true,
        ValueRange::Any,
    ),
];

/// Resolves an attribute by its full identifier (`EnvironmentAttributes.CODEC`).
pub fn find_attribute(id: &Identifier) -> Option<&'static AttributeDef> {
    if id.namespace() != "minecraft" {
        return None;
    }
    ATTRIBUTES.iter().find(|def| def.id == id.path())
}

/// `AttributeModifier.OperationId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Override,
    AlphaBlend,
    Add,
    Subtract,
    Multiply,
    BlendToGray,
    Minimum,
    Maximum,
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}

impl Modifier {
    /// `OperationId.getSerializedName`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Override => "override",
            Self::AlphaBlend => "alpha_blend",
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::BlendToGray => "blend_to_gray",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
            Self::And => "and",
            Self::Nand => "nand",
            Self::Or => "or",
            Self::Nor => "nor",
            Self::Xor => "xor",
            Self::Xnor => "xnor",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        const ALL: [Modifier; 14] = [
            Modifier::Override,
            Modifier::AlphaBlend,
            Modifier::Add,
            Modifier::Subtract,
            Modifier::Multiply,
            Modifier::BlendToGray,
            Modifier::Minimum,
            Modifier::Maximum,
            Modifier::And,
            Modifier::Nand,
            Modifier::Or,
            Modifier::Nor,
            Modifier::Xor,
            Modifier::Xnor,
        ];
        ALL.into_iter().find(|modifier| modifier.name() == name)
    }
}

/// `AttributeModifier.*_LIBRARY` for a type (override is always allowed).
fn modifier_library(ty: AttributeType) -> &'static [Modifier] {
    use Modifier::*;
    match ty {
        AttributeType::Boolean => &[And, Nand, Or, Nor, Xor, Xnor],
        AttributeType::Float | AttributeType::AngleDegrees => {
            &[AlphaBlend, Add, Subtract, Multiply, Minimum, Maximum]
        }
        AttributeType::RgbColor | AttributeType::ArgbColor => {
            &[AlphaBlend, Add, Subtract, Multiply, BlendToGray]
        }
        _ => &[],
    }
}

const ACTIVITY_REGISTRY: &str = "minecraft:activity";

const MOON_PHASES: &[&str] = &[
    "full_moon",
    "waning_gibbous",
    "third_quarter",
    "waning_crescent",
    "new_moon",
    "waxing_crescent",
    "first_quarter",
    "waxing_gibbous",
];

const BED_RULES: &[&str] = &["always", "when_dark", "never"];

/// `TriState.CODEC`: booleans collapse to bytes, `default` stays a string.
fn tri_state() -> Codec {
    Codec::new(|json, _| match json {
        Json::Bool(flag) => Ok(Tag::Byte(i8::from(*flag))),
        Json::String(text) => match text.as_str() {
            "true" => Ok(Tag::Byte(1)),
            "false" => Ok(Tag::Byte(0)),
            "default" => Ok(Tag::String("default".to_string())),
            other => Err(format!("Unknown element name:{other}")),
        },
        other => Err(format!("Not a boolean or string: {}", describe(other))),
    })
}

/// `BedRule.CODEC`.
fn bed_rule() -> Codec {
    record(vec![
        req("can_sleep", enum_codec(BED_RULES)),
        req("can_set_spawn", enum_codec(BED_RULES)),
        opt_default("explodes", bool_codec(), Json::Bool(false)),
        opt("error_message", component()),
    ])
}

/// `AmbientSounds.CODEC`.
fn ambient_sounds() -> Codec {
    record(vec![
        opt("loop", sound_event()),
        opt(
            "mood",
            record(vec![
                req("sound", sound_event()),
                req("tick_delay", int_codec()),
                req("block_search_extent", int_codec()),
                req("offset", double_codec()),
            ]),
        ),
        opt_default(
            "additions",
            codec::compact_list(record(vec![
                req("sound", sound_event()),
                req("tick_chance", double_codec()),
            ])),
            Json::Array(Vec::new()),
        ),
    ])
}

/// `BackgroundMusic.CODEC`.
fn background_music() -> Codec {
    record(vec![
        opt("default", music()),
        opt("creative", music()),
        opt("underwater", music()),
    ])
}

/// The unvalidated `AttributeType.valueCodec()`.
fn type_value_codec(ty: AttributeType) -> Codec {
    match ty {
        AttributeType::Boolean => bool_codec(),
        AttributeType::TriState => tri_state(),
        AttributeType::Float | AttributeType::AngleDegrees => float_codec(),
        AttributeType::RgbColor => string_rgb_color(),
        AttributeType::ArgbColor => string_argb_color(),
        AttributeType::MoonPhase => enum_codec(MOON_PHASES),
        AttributeType::Activity => codec::holder_fixed(ACTIVITY_REGISTRY),
        AttributeType::BedRule => bed_rule(),
        AttributeType::Particle => particle_options(),
        AttributeType::AmbientParticles => list(record(vec![
            req("particle", particle_options()),
            req("probability", float_range(0.0, 1.0)),
        ])),
        AttributeType::BackgroundMusic => background_music(),
        AttributeType::AmbientSounds => ambient_sounds(),
    }
}

/// `EnvironmentAttribute.valueCodec()`: the type codec validated by the range.
pub fn attribute_value_codec(def: &AttributeDef) -> Codec {
    let value = type_value_codec(def.ty);
    match (def.ty, def.range.bounds()) {
        (AttributeType::Float | AttributeType::AngleDegrees, Some((min, max))) => {
            value.validate(move |tag| match tag {
                Tag::Float(v) if !(*v >= min && *v <= max) => {
                    Err(format!("{v:?} is not in range [{min:?}; {max:?}]"))
                }
                _ => Ok(()),
            })
        }
        _ => value,
    }
}

/// `FloatWithAlpha.CODEC`.
fn float_with_alpha() -> Codec {
    Codec::new(|json, ctx| match json {
        Json::Object(_) => {
            let full = record(vec![
                req("value", float_codec()),
                opt_default("alpha", float_range(0.0, 1.0), serde_json::json!(1.0)),
            ]);
            let tag = full.parse(json, ctx)?;
            // alpha == 1.0 collapses to the bare float.
            match &tag {
                Tag::Compound(fields) if fields.len() == 1 => Ok(fields[0].1.clone()),
                _ => Ok(tag),
            }
        }
        _ => float_codec().parse(json, ctx),
    })
}

/// `ColorModifier.ArgbModifier` argument codec.
fn argb_modifier_argument() -> Codec {
    Codec::new(|json, ctx| {
        let decoded = either(string_argb_color(), color_int_codec(3)).parse(json, ctx)?;
        // Alpha 255 is written as a plain int, anything else as `#aarrggbb`.
        let argb = match &decoded {
            Tag::String(text) => i64::from_str_radix(text.trim_start_matches('#'), 16).ok(),
            Tag::Int(value) => Some(i64::from(*value as u32)),
            _ => None,
        };
        match argb {
            Some(value) if (value >> 24) & 0xFF == 255 => Ok(Tag::Int(value as u32 as i32)),
            Some(value) => Ok(Tag::String(format!("#{value:08x}"))),
            None => Ok(decoded),
        }
    })
}

/// `AttributeModifier.argumentCodec(attribute)`.
pub fn modifier_argument_codec(def: &AttributeDef, modifier: Modifier) -> Codec {
    match (def.ty, modifier) {
        (_, Modifier::Override) => attribute_value_codec(def),
        (AttributeType::Boolean, _) => bool_codec(),
        (AttributeType::Float | AttributeType::AngleDegrees, Modifier::AlphaBlend) => {
            float_with_alpha()
        }
        (AttributeType::Float | AttributeType::AngleDegrees, _) => float_codec(),
        (AttributeType::RgbColor | AttributeType::ArgbColor, Modifier::AlphaBlend) => {
            string_argb_color()
        }
        (AttributeType::RgbColor | AttributeType::ArgbColor, Modifier::BlendToGray) => {
            record(vec![
                req("brightness", float_range(0.0, 1.0)),
                req("factor", float_range(0.0, 1.0)),
            ])
        }
        (AttributeType::ArgbColor, Modifier::Multiply) => argb_modifier_argument(),
        (AttributeType::RgbColor | AttributeType::ArgbColor, _) => string_rgb_color(),
        // Types without a modifier library only allow override (handled above).
        _ => attribute_value_codec(def),
    }
}

/// `AttributeType.modifierCodec()` for `def`: resolves an operation id and checks it
/// is in the type's library.
pub fn parse_modifier(def: &AttributeDef, name: &str) -> CodecResult<Modifier> {
    let modifier =
        Modifier::from_name(name).ok_or_else(|| format!("Unknown element name:{name}"))?;
    if modifier == Modifier::Override || modifier_library(def.ty).contains(&modifier) {
        Ok(modifier)
    } else {
        Err(format!("Unknown modifier {name}"))
    }
}

/// `EnvironmentAttributeMap.Entry.createCodec(attribute)`: a bare value (override)
/// or `{modifier, argument}`.
pub fn attribute_entry_codec(def: &'static AttributeDef) -> Codec {
    let value = attribute_value_codec(def);
    let full = Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let raw = object
            .get("modifier")
            .ok_or_else(|| format!("No key modifier in MapLike[{}]", describe(json)))?;
        let name = raw
            .as_str()
            .ok_or_else(|| format!("Not a string: {}", describe(raw)))?;
        let modifier = parse_modifier(def, name)?;
        let argument = object
            .get("argument")
            .ok_or_else(|| format!("No key argument in MapLike[{}]", describe(json)))?;
        let argument = modifier_argument_codec(def, modifier).parse(argument, ctx)?;
        if modifier == Modifier::Override {
            return Ok(argument);
        }
        Ok(Tag::Compound(vec![
            (
                "modifier".to_string(),
                Tag::String(modifier.name().to_string()),
            ),
            ("argument".to_string(), argument),
        ]))
    });
    either(value, full)
}

/// Which `EnvironmentAttributeMap` codec to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeMapMode {
    /// `EnvironmentAttributeMap.CODEC`.
    Direct,
    /// `EnvironmentAttributeMap.NETWORK_CODEC`: only syncable attributes survive.
    Network,
    /// `EnvironmentAttributeMap.CODEC_ONLY_POSITIONAL`: rejects non-positional
    /// attributes.
    OnlyPositional,
}

/// The `EnvironmentAttributes.CODEC` key codec (a registry id lookup).
pub fn attribute_key_codec() -> Codec {
    Codec::new(|json, _| {
        let id = codec::parse_identifier(json)?;
        if find_attribute(&id).is_some() {
            Ok(Tag::String(id.to_string()))
        } else {
            Err(format!(
                "Unknown registry key in ResourceKey[minecraft:root / minecraft:environment_attribute]: {id}"
            ))
        }
    })
}

/// `EnvironmentAttributeMap.CODEC` and its network/positional variants.
pub fn environment_attribute_map(mode: AttributeMapMode) -> Codec {
    let map = codec::dispatched_map(attribute_key_codec(), |key| {
        let id = Identifier::parse(key)?;
        let def = find_attribute(&id).ok_or_else(|| format!("Unknown attribute {key}"))?;
        Ok(attribute_entry_codec(def))
    });
    map.map_tag(move |tag| {
        let Tag::Compound(entries) = tag else {
            return Ok(tag);
        };
        let defs = |key: &str| {
            Identifier::parse(key)
                .ok()
                .and_then(|id| find_attribute(&id))
        };
        match mode {
            AttributeMapMode::Direct => Ok(Tag::Compound(entries)),
            AttributeMapMode::Network => Ok(Tag::Compound(
                entries
                    .into_iter()
                    .filter(|(key, _)| defs(key).is_some_and(|def| def.syncable))
                    .collect(),
            )),
            AttributeMapMode::OnlyPositional => {
                let illegal: Vec<String> = entries
                    .iter()
                    .filter(|(key, _)| defs(key).is_some_and(|def| !def.positional))
                    .map(|(key, _)| key.clone())
                    .collect();
                if illegal.is_empty() {
                    Ok(Tag::Compound(entries))
                } else {
                    Err(format!(
                        "The following attributes cannot be positional: [{}]",
                        illegal.join(", ")
                    ))
                }
            }
        }
    })
}
