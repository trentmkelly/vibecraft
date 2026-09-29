//! Element codecs for the synchronised registries, one function per Java class
//! (`ChatType.DIRECT_CODEC`, `WolfVariant.NETWORK_CODEC`, ...).
//!
//! Each `*_direct` function models the codec the data-pack loader decodes with;
//! `*_network` the codec `RegistrySynchronization` encodes with. Where Java uses one
//! codec for both, a single function serves both.

use serde_json::{json, Value as Json};

use crate::registry_pipeline::attributes::{
    attribute_key_codec, environment_attribute_map, find_attribute, modifier_argument_codec,
    parse_modifier, AttributeDef, AttributeMapMode, Modifier,
};
use crate::registry_pipeline::codec::{
    self, bool_codec, describe, double_range, either, enum_codec, float_codec, float_range,
    generic, holder_fixed, holder_set, identifier_codec, int_range, list, non_negative_int, opt,
    opt_default, or_else, positive_float, positive_int, record, req, resource_path_codec,
    string_codec, tag_key_hashed, unbounded_map, Codec, CodecResult, Field,
};
use crate::registry_pipeline::shared::{component, int_provider, sound_event, style};
use crate::storage::nbt::Tag;

const WORLD_CLOCK_REGISTRY: &str = "minecraft:world_clock";
const TIMELINE_REGISTRY: &str = "minecraft:timeline";
const ENTITY_TYPE_REGISTRY: &str = "minecraft:entity_type";

// ---------------------------------------------------------------------------
// Biome
// ---------------------------------------------------------------------------

const GRASS_COLOR_MODIFIERS: &[&str] = &["none", "dark_forest", "swamp"];
const TEMPERATURE_MODIFIERS: &[&str] = &["none", "frozen"];
const MOB_CATEGORIES: &[&str] = &[
    "monster",
    "creature",
    "ambient",
    "axolotls",
    "underground_water_creature",
    "water_creature",
    "water_ambient",
    "misc",
];

/// `Biome.ClimateSettings.CODEC`.
fn climate_settings() -> Vec<Field> {
    vec![
        req("has_precipitation", bool_codec()),
        req("temperature", float_codec()),
        opt_default(
            "temperature_modifier",
            enum_codec(TEMPERATURE_MODIFIERS),
            json!("none"),
        ),
        req("downfall", float_codec()),
    ]
}

/// `BiomeSpecialEffects.CODEC`.
fn biome_special_effects() -> Codec {
    use crate::registry_pipeline::shared::string_rgb_color;
    record(vec![
        req("water_color", string_rgb_color()),
        opt("foliage_color", string_rgb_color()),
        opt("dry_foliage_color", string_rgb_color()),
        opt("grass_color", string_rgb_color()),
        opt_default(
            "grass_color_modifier",
            enum_codec(GRASS_COLOR_MODIFIERS),
            json!("none"),
        ),
    ])
}

/// `Biome.NETWORK_CODEC`.
pub fn biome_network() -> Codec {
    let mut fields = climate_settings();
    fields.push(opt_default(
        "attributes",
        environment_attribute_map(AttributeMapMode::Network),
        json!({}),
    ));
    fields.push(req("effects", biome_special_effects()));
    record(fields)
}

/// `Biome.DIRECT_CODEC`.
///
/// The generation settings' `carvers` and `features` resolve into the
/// `configured_carver`/`placed_feature` registries, which are loaded by
/// [`worldgen_reference_target`] so unknown references fail the load. Inline
/// (non-registered) carvers/features are not supported.
pub fn biome_direct() -> Codec {
    let mut fields = climate_settings();
    fields.push(opt_default(
        "attributes",
        environment_attribute_map(AttributeMapMode::OnlyPositional),
        json!({}),
    ));
    fields.push(req("effects", biome_special_effects()));
    // BiomeGenerationSettings.CODEC
    fields.push(req(
        "carvers",
        holder_set("minecraft:worldgen/configured_carver", false),
    ));
    fields.push(req(
        "features",
        list(holder_set("minecraft:worldgen/placed_feature", false)),
    ));
    // MobSpawnSettings.CODEC
    fields.push(opt_default(
        "creature_spawn_probability",
        float_range(0.0, 0.999_999_9),
        json!(0.1),
    ));
    fields.push(req(
        "spawners",
        unbounded_map(
            enum_codec(MOB_CATEGORIES),
            list(record(vec![
                req("type", holder_fixed(ENTITY_TYPE_REGISTRY)),
                req("minCount", positive_int()),
                req("maxCount", positive_int()),
                req("weight", non_negative_int()),
            ])),
        ),
    ));
    fields.push(req(
        "spawn_costs",
        unbounded_map(
            holder_fixed(ENTITY_TYPE_REGISTRY),
            record(vec![
                req("energy_budget", codec::double_codec()),
                req("charge", codec::double_codec()),
            ]),
        ),
    ));
    record(fields)
}

// ---------------------------------------------------------------------------
// Chat, trims, banner patterns, damage types, jukebox songs, instruments, clocks
// ---------------------------------------------------------------------------

const CHAT_PARAMETERS: &[&str] = &["sender", "target", "content"];

/// `ChatTypeDecoration.CODEC`.
fn chat_type_decoration() -> Codec {
    record(vec![
        req("translation_key", string_codec()),
        req("parameters", list(enum_codec(CHAT_PARAMETERS))),
        opt_default("style", style(), json!({})),
    ])
}

/// `ChatType.DIRECT_CODEC`.
pub fn chat_type() -> Codec {
    record(vec![
        req("chat", chat_type_decoration()),
        req("narration", chat_type_decoration()),
    ])
}

/// `TrimPattern.DIRECT_CODEC`.
pub fn trim_pattern() -> Codec {
    record(vec![
        req("asset_id", identifier_codec()),
        req("description", component()),
        or_else("decal", bool_codec(), json!(false)),
    ])
}

/// `TrimMaterial.DIRECT_CODEC` (`MaterialAssetGroup.MAP_CODEC` + description).
pub fn trim_material() -> Codec {
    record(vec![
        req("asset_name", resource_path_codec()),
        opt_default(
            "override_armor_assets",
            unbounded_map(identifier_codec(), resource_path_codec()),
            json!({}),
        ),
        req("description", component()),
    ])
}

/// `BannerPattern.DIRECT_CODEC`.
pub fn banner_pattern() -> Codec {
    record(vec![
        req("asset_id", identifier_codec()),
        req("translation_key", string_codec()),
    ])
}

const DAMAGE_SCALING: &[&str] = &["never", "when_caused_by_living_non_player", "always"];
const DAMAGE_EFFECTS: &[&str] = &[
    "hurt", "thorns", "drowning", "burning", "poking", "freezing",
];
const DEATH_MESSAGE_TYPES: &[&str] = &["default", "fall_variants", "intentional_game_design"];

/// `DamageType.DIRECT_CODEC`.
pub fn damage_type() -> Codec {
    record(vec![
        req("message_id", string_codec()),
        req("scaling", enum_codec(DAMAGE_SCALING)),
        req("exhaustion", float_codec()),
        opt_default("effects", enum_codec(DAMAGE_EFFECTS), json!("hurt")),
        opt_default(
            "death_message_type",
            enum_codec(DEATH_MESSAGE_TYPES),
            json!("default"),
        ),
    ])
}

/// `JukeboxSong.DIRECT_CODEC`.
pub fn jukebox_song() -> Codec {
    record(vec![
        req("sound_event", sound_event()),
        req("description", component()),
        req("length_in_seconds", positive_float()),
        req("comparator_output", int_range(0, 15)),
    ])
}

/// `Instrument.DIRECT_CODEC`.
pub fn instrument() -> Codec {
    record(vec![
        req("sound_event", sound_event()),
        req("use_duration", positive_float()),
        req("range", positive_float()),
        req("description", component()),
    ])
}

/// `WorldClock.DIRECT_CODEC` (`MapCodec.unitCodec`).
pub fn world_clock() -> Codec {
    Codec::new(|json, _| match json {
        Json::Object(_) => Ok(Tag::Compound(Vec::new())),
        other => Err(format!("Not a map: {}", describe(other))),
    })
}

/// Registries the server loads only so other registries' references into them can
/// be resolved (`configured_carver`, `placed_feature`, `structure`). Elements are
/// accepted as any JSON value; none of these registries is sent to clients.
///
/// TODO(registry-pipeline-worldgen-configured-carver): `ConfiguredWorldCarver.DIRECT_CODEC`.
/// TODO(registry-pipeline-worldgen-placed-feature): `PlacedFeature.DIRECT_CODEC`.
/// TODO(registry-pipeline-worldgen-structure): `Structure.DIRECT_CODEC`.
pub fn worldgen_reference_target() -> Codec {
    generic()
}

/// Registries whose Java codec is not ported yet are decoded with the generic
/// converter (see [`codec::json_to_tag`]).
///
/// TODO(registry-pipeline-enchantment): `Enchantment.DIRECT_CODEC`.
/// TODO(registry-pipeline-dialog): `Dialog.DIRECT_CODEC`.
pub fn unported() -> Codec {
    generic()
}

// ---------------------------------------------------------------------------
// Entity variants
// ---------------------------------------------------------------------------

/// `ClientAsset.ResourceTexture.DEFAULT_FIELD_CODEC`.
fn asset_id() -> Field {
    req("asset_id", identifier_codec())
}

/// `ModelAndTexture.codec(modelCodec, defaultModel)`.
fn model_and_texture(models: &'static [&'static str]) -> Vec<Field> {
    vec![
        opt_default("model", enum_codec(models), json!("normal")),
        asset_id(),
    ]
}

/// `SpawnPrioritySelectors.CODEC`.
///
/// Biome and structure holder sets inside spawn conditions resolve into the
/// `worldgen/biome` and `worldgen/structure` registries.
fn spawn_conditions() -> Codec {
    let condition = codec::dispatch("type", |id| {
        match id.to_string().as_str() {
        "minecraft:biome" => Ok(record(vec![req(
            "biomes",
            holder_set("minecraft:worldgen/biome", false),
        )])),
        "minecraft:structure" => Ok(record(vec![req(
            "structures",
            holder_set("minecraft:worldgen/structure", false),
        )])),
        "minecraft:moon_brightness" => Ok(record(vec![req("range", generic())])),
        other => Err(format!(
            "Unknown registry key in ResourceKey[minecraft:root / minecraft:spawn_condition_type]: {other}"
        )),
    }
    });
    list(record(vec![
        opt("condition", condition),
        req("priority", codec::int_codec()),
    ]))
}

/// A variant whose direct codec appends `spawn_conditions` to the network fields.
fn with_spawn_conditions(mut network_fields: Vec<Field>) -> Codec {
    network_fields.push(req("spawn_conditions", spawn_conditions()));
    record(network_fields)
}

fn wolf_asset_info() -> Codec {
    record(vec![
        req("wild", identifier_codec()),
        req("tame", identifier_codec()),
        req("angry", identifier_codec()),
    ])
}

fn wolf_variant_fields() -> Vec<Field> {
    vec![
        req("assets", wolf_asset_info()),
        req("baby_assets", wolf_asset_info()),
    ]
}

/// `WolfVariant.NETWORK_CODEC`.
pub fn wolf_variant_network() -> Codec {
    record(wolf_variant_fields())
}

/// `WolfVariant.DIRECT_CODEC`.
pub fn wolf_variant_direct() -> Codec {
    with_spawn_conditions(wolf_variant_fields())
}

/// A record of sound-event fields.
fn sound_set(fields: &[&'static str]) -> Codec {
    record(fields.iter().map(|name| req(name, sound_event())).collect())
}

fn adult_baby(set: Codec) -> Codec {
    record(vec![
        req("adult_sounds", set.clone()),
        req("baby_sounds", set),
    ])
}

/// `WolfSoundVariant` codec (direct == network).
pub fn wolf_sound_variant() -> Codec {
    adult_baby(sound_set(&[
        "ambient_sound",
        "death_sound",
        "growl_sound",
        "hurt_sound",
        "pant_sound",
        "whine_sound",
        "step_sound",
    ]))
}

/// `PigSoundVariant` codec.
pub fn pig_sound_variant() -> Codec {
    adult_baby(sound_set(&[
        "ambient_sound",
        "hurt_sound",
        "death_sound",
        "step_sound",
        "eat_sound",
    ]))
}

/// `CatSoundVariant` codec.
pub fn cat_sound_variant() -> Codec {
    adult_baby(sound_set(&[
        "ambient_sound",
        "stray_ambient_sound",
        "hiss_sound",
        "hurt_sound",
        "death_sound",
        "eat_sound",
        "beg_for_food_sound",
        "purr_sound",
        "purreow_sound",
    ]))
}

/// `ChickenSoundVariant` codec.
pub fn chicken_sound_variant() -> Codec {
    adult_baby(sound_set(&[
        "ambient_sound",
        "hurt_sound",
        "death_sound",
        "step_sound",
    ]))
}

/// `CowSoundVariant.DIRECT_CODEC` (used for both directions).
pub fn cow_sound_variant() -> Codec {
    sound_set(&["ambient_sound", "hurt_sound", "death_sound", "step_sound"])
}

const PIG_MODELS: &[&str] = &["normal", "cold"];
const COW_MODELS: &[&str] = &["normal", "cold", "warm"];
const CHICKEN_MODELS: &[&str] = &["normal", "cold"];
const ZOMBIE_NAUTILUS_MODELS: &[&str] = &["normal", "warm"];

fn baby_texture_fields(models: &'static [&'static str]) -> Vec<Field> {
    let mut fields = model_and_texture(models);
    fields.push(req("baby_asset_id", identifier_codec()));
    fields
}

/// `PigVariant.NETWORK_CODEC`.
pub fn pig_variant_network() -> Codec {
    record(baby_texture_fields(PIG_MODELS))
}

/// `PigVariant.DIRECT_CODEC`.
pub fn pig_variant_direct() -> Codec {
    with_spawn_conditions(baby_texture_fields(PIG_MODELS))
}

/// `CowVariant.NETWORK_CODEC`.
pub fn cow_variant_network() -> Codec {
    record(baby_texture_fields(COW_MODELS))
}

/// `CowVariant.DIRECT_CODEC`.
pub fn cow_variant_direct() -> Codec {
    with_spawn_conditions(baby_texture_fields(COW_MODELS))
}

/// `ChickenVariant.NETWORK_CODEC`.
pub fn chicken_variant_network() -> Codec {
    record(baby_texture_fields(CHICKEN_MODELS))
}

/// `ChickenVariant.DIRECT_CODEC`.
pub fn chicken_variant_direct() -> Codec {
    with_spawn_conditions(baby_texture_fields(CHICKEN_MODELS))
}

/// `ZombieNautilusVariant.NETWORK_CODEC`.
pub fn zombie_nautilus_variant_network() -> Codec {
    record(model_and_texture(ZOMBIE_NAUTILUS_MODELS))
}

/// `ZombieNautilusVariant.DIRECT_CODEC`.
pub fn zombie_nautilus_variant_direct() -> Codec {
    with_spawn_conditions(model_and_texture(ZOMBIE_NAUTILUS_MODELS))
}

/// `FrogVariant.NETWORK_CODEC`.
pub fn frog_variant_network() -> Codec {
    record(vec![asset_id()])
}

/// `FrogVariant.DIRECT_CODEC`.
pub fn frog_variant_direct() -> Codec {
    with_spawn_conditions(vec![asset_id()])
}

fn cat_variant_fields() -> Vec<Field> {
    vec![asset_id(), req("baby_asset_id", identifier_codec())]
}

/// `CatVariant.NETWORK_CODEC`.
pub fn cat_variant_network() -> Codec {
    record(cat_variant_fields())
}

/// `CatVariant.DIRECT_CODEC`.
pub fn cat_variant_direct() -> Codec {
    with_spawn_conditions(cat_variant_fields())
}

/// `PaintingVariant.DIRECT_CODEC`.
pub fn painting_variant() -> Codec {
    record(vec![
        req("width", int_range(1, 16)),
        req("height", int_range(1, 16)),
        req("asset_id", identifier_codec()),
        opt("title", component()),
        opt("author", component()),
    ])
}

// ---------------------------------------------------------------------------
// Dimension type
// ---------------------------------------------------------------------------

/// `BlockPos.PACKED_Y_LENGTH`-derived bounds of `DimensionType`.
const Y_SIZE: i32 = (1 << 12) - 32;
const MAX_Y: i32 = (Y_SIZE >> 1) - 1;
const MIN_Y: i32 = MAX_Y - Y_SIZE + 1;

const SKYBOXES: &[&str] = &["none", "overworld", "end"];
const CARDINAL_LIGHT_TYPES: &[&str] = &["default", "nether"];

fn tag_int(tag: &Tag, name: &str) -> Option<i32> {
    match tag {
        Tag::Compound(fields) => fields.iter().find_map(|(key, value)| match value {
            Tag::Int(v) if key == name => Some(*v),
            _ => None,
        }),
        _ => None,
    }
}

/// The compact constructor checks of `DimensionType`, reported as decode errors
/// (`ExtraCodecs.catchDecoderException`).
fn validate_dimension_type(tag: &Tag) -> CodecResult<()> {
    let (Some(min_y), Some(height), Some(logical_height)) = (
        tag_int(tag, "min_y"),
        tag_int(tag, "height"),
        tag_int(tag, "logical_height"),
    ) else {
        return Ok(());
    };
    if height < 16 {
        return Err("height has to be at least 16".to_string());
    }
    if min_y + height > MAX_Y + 1 {
        return Err(format!(
            "min_y + height cannot be higher than: {}",
            MAX_Y + 1
        ));
    }
    if logical_height > height {
        return Err("logical_height cannot be higher than height".to_string());
    }
    if height % 16 != 0 {
        return Err("height has to be multiple of 16".to_string());
    }
    if min_y % 16 != 0 {
        return Err("min_y has to be a multiple of 16".to_string());
    }
    Ok(())
}

fn dimension_type(mode: AttributeMapMode) -> Codec {
    record(vec![
        opt_default("has_fixed_time", bool_codec(), json!(false)),
        req("has_skylight", bool_codec()),
        req("has_ceiling", bool_codec()),
        req("has_ender_dragon_fight", bool_codec()),
        req(
            "coordinate_scale",
            double_range(f64::from(1.0e-5_f32), 3.0e7),
        ),
        req("min_y", int_range(MIN_Y, MAX_Y)),
        req("height", int_range(16, Y_SIZE)),
        req("logical_height", int_range(0, Y_SIZE)),
        req("infiniburn", tag_key_hashed("minecraft:block")),
        req("ambient_light", float_codec()),
        req("monster_spawn_light_level", int_provider(0, 15)),
        req("monster_spawn_block_light_limit", int_range(0, 15)),
        opt_default("skybox", enum_codec(SKYBOXES), json!("overworld")),
        opt_default(
            "cardinal_light",
            enum_codec(CARDINAL_LIGHT_TYPES),
            json!("default"),
        ),
        opt_default("attributes", environment_attribute_map(mode), json!({})),
        opt_default("timelines", holder_set(TIMELINE_REGISTRY, false), json!([])),
        opt("default_clock", holder_fixed(WORLD_CLOCK_REGISTRY)),
    ])
    .validate(validate_dimension_type)
}

/// `DimensionType.DIRECT_CODEC`.
pub fn dimension_type_direct() -> Codec {
    dimension_type(AttributeMapMode::Direct)
}

/// `DimensionType.NETWORK_CODEC`.
pub fn dimension_type_network() -> Codec {
    dimension_type(AttributeMapMode::Network)
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

const SIMPLE_EASINGS: &[&str] = &[
    "constant",
    "linear",
    "in_back",
    "in_bounce",
    "in_circ",
    "in_cubic",
    "in_elastic",
    "in_expo",
    "in_quad",
    "in_quart",
    "in_quint",
    "in_sine",
    "in_out_back",
    "in_out_bounce",
    "in_out_circ",
    "in_out_cubic",
    "in_out_elastic",
    "in_out_expo",
    "in_out_quad",
    "in_out_quart",
    "in_out_quint",
    "in_out_sine",
    "out_back",
    "out_bounce",
    "out_circ",
    "out_cubic",
    "out_elastic",
    "out_expo",
    "out_quad",
    "out_quart",
    "out_quint",
    "out_sine",
];

/// `EasingType.CODEC`.
fn easing() -> Codec {
    let cubic_bezier = record(vec![req(
        "cubic_bezier",
        list(float_codec()).validate(|tag| match tag {
            Tag::List(items) if items.len() != 4 => Err(format!(
                "List is too short/long: expected 4 elements but got {}",
                items.len()
            )),
            Tag::List(items) => {
                let value = |index: usize| match items.get(index) {
                    Some(Tag::Float(v)) => *v,
                    _ => 0.0,
                };
                if !(0.0..=1.0).contains(&value(0)) {
                    Err("x1 must be in range [0; 1]".to_string())
                } else if !(0.0..=1.0).contains(&value(2)) {
                    Err("x2 must be in range [0; 1]".to_string())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }),
    )]);
    either(enum_codec(SIMPLE_EASINGS), cubic_bezier)
}

/// `KeyframeTrack.validateKeyframes` on an encoded keyframe list.
fn validate_keyframes(tag: &Tag) -> CodecResult<()> {
    let Tag::List(keyframes) = tag else {
        return Ok(());
    };
    if keyframes.is_empty() {
        return Err("Keyframes must not be empty".to_string());
    }
    let ticks: Vec<i32> = keyframes
        .iter()
        .map(|keyframe| tag_int(keyframe, "ticks").unwrap_or(0))
        .collect();
    if ticks.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err("Keyframes must be ordered by ticks field".to_string());
    }
    if ticks.len() > 1 {
        let mut repeat_count = 0;
        let mut last = ticks[ticks.len() - 1];
        for tick in &ticks {
            if *tick == last {
                repeat_count += 1;
                if repeat_count > 2 {
                    return Err(format!("More than 2 keyframes on same tick: {tick}"));
                }
            } else {
                repeat_count = 0;
            }
            last = *tick;
        }
    }
    Ok(())
}

/// `AttributeTrack.createCodec(attribute)`.
fn attribute_track(def: &'static AttributeDef) -> Codec {
    Codec::new(move |json, ctx| {
        let Json::Object(object) = json else {
            return Err(format!("Not a map: {}", describe(json)));
        };
        let modifier = match object.get("modifier") {
            None => Modifier::Override,
            Some(raw) => {
                let name = raw
                    .as_str()
                    .ok_or_else(|| format!("Not a string: {}", describe(raw)))?;
                parse_modifier(def, name)?
            }
        };
        let keyframe = record(vec![
            req("ticks", non_negative_int()),
            req("value", modifier_argument_codec(def, modifier)),
        ]);
        let track = record(vec![
            req("keyframes", list(keyframe).validate(validate_keyframes)),
            opt_default("ease", easing(), json!("linear")),
        ]);
        let mut fields = Vec::new();
        if modifier != Modifier::Override {
            fields.push((
                "modifier".to_string(),
                Tag::String(modifier.name().to_string()),
            ));
        }
        match track.parse(json, ctx)? {
            Tag::Compound(rest) => fields.extend(rest),
            other => return Err(format!("Track produced a non-map: {other:?}")),
        }
        Ok(Tag::Compound(fields))
    })
}

/// `Timeline.TimeMarkerInfo.CODEC`.
fn time_marker_info() -> Codec {
    Codec::new(|json, ctx| match json {
        Json::Object(_) => {
            let full = record(vec![
                req("ticks", non_negative_int()),
                opt_default("show_in_commands", bool_codec(), json!(false)),
            ]);
            match full.parse(json, ctx)? {
                Tag::Compound(fields) if fields.len() == 1 => Ok(fields[0].1.clone()),
                other => Ok(other),
            }
        }
        _ => non_negative_int().parse(json, ctx),
    })
}

fn timeline(syncable_only: bool) -> Codec {
    let tracks = codec::dispatched_map(attribute_key_codec(), |key| {
        let id = crate::registry::Identifier::parse(key)?;
        let def = find_attribute(&id).ok_or_else(|| format!("Unknown attribute {key}"))?;
        Ok(attribute_track(def))
    });
    record(vec![
        req("clock", holder_fixed(WORLD_CLOCK_REGISTRY)),
        opt("period_ticks", positive_int()),
        opt_default("tracks", tracks, json!({})),
        opt_default(
            "time_markers",
            unbounded_map(identifier_codec(), time_marker_info()),
            json!({}),
        ),
    ])
    .validate(validate_timeline)
    .map_tag(move |tag| {
        if !syncable_only {
            return Ok(tag);
        }
        let Tag::Compound(fields) = tag else {
            return Ok(tag);
        };
        // `Timeline.filterSyncableTracks`: drop non-syncable tracks; a track map that
        // becomes empty equals the `optionalFieldOf("tracks", Map.of())` default and is
        // therefore not written.
        Ok(Tag::Compound(
            fields
                .into_iter()
                .filter_map(|(key, value)| match (key.as_str(), value) {
                    ("tracks", Tag::Compound(tracks)) => {
                        let syncable: Vec<(String, Tag)> = tracks
                            .into_iter()
                            .filter(|(attribute, _)| {
                                crate::registry::Identifier::parse(attribute)
                                    .ok()
                                    .and_then(|id| find_attribute(&id))
                                    .is_some_and(|def| def.syncable)
                            })
                            .collect();
                        (!syncable.is_empty()).then_some((key, Tag::Compound(syncable)))
                    }
                    (_, value) => Some((key, value)),
                })
                .collect(),
        ))
    })
}

/// `Timeline.validateInternal` on an encoded timeline.
fn validate_timeline(tag: &Tag) -> CodecResult<()> {
    let Some(period) = tag_int(tag, "period_ticks") else {
        return Ok(());
    };
    let Tag::Compound(fields) = tag else {
        return Ok(());
    };
    for (key, value) in fields {
        match (key.as_str(), value) {
            ("time_markers", Tag::Compound(markers)) => {
                for (marker, info) in markers {
                    let ticks = match info {
                        Tag::Int(ticks) => Some(*ticks),
                        other => tag_int(other, "ticks"),
                    };
                    if let Some(ticks) = ticks {
                        if ticks < 0 || ticks >= period {
                            return Err(format!(
                                "Time Marker ResourceKey[minecraft:clock_time_marker / {marker}] must be in range [0; {period})"
                            ));
                        }
                    }
                }
            }
            ("tracks", Tag::Compound(tracks)) => {
                for (_, track) in tracks {
                    let Tag::Compound(track_fields) = track else {
                        continue;
                    };
                    for (name, keyframes) in track_fields {
                        let (true, Tag::List(keyframes)) = (name == "keyframes", keyframes) else {
                            continue;
                        };
                        for keyframe in keyframes {
                            let tick = tag_int(keyframe, "ticks").unwrap_or(0);
                            if tick < 0 || tick > period {
                                return Err(format!(
                                    "Keyframe at tick {tick} must be in range [0; {period}]"
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// `Timeline.DIRECT_CODEC`.
pub fn timeline_direct() -> Codec {
    timeline(false)
}

/// `Timeline.NETWORK_CODEC`.
pub fn timeline_network() -> Codec {
    timeline(true)
}
