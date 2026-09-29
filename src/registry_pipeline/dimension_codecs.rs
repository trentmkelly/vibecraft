//! Codecs for the world creation registries: `worldgen/world_preset`
//! (`WorldPreset.DIRECT_CODEC`), `LevelStem`, the chunk generators and biome
//! sources they use, `worldgen/flat_level_generator_preset` and
//! `worldgen/multi_noise_biome_source_parameter_list`.

use serde_json::{json, Value as Json};

use crate::registry_pipeline::codec::{
    bool_codec, custom_field, either, holder_file, holder_fixed, holder_set, identifier_codec,
    int_range, lenient_opt, list, or_else, record, req, unbounded_map, Codec, CodecContext, Field,
};
use crate::registry_pipeline::element_codecs::{biome_direct, dimension_type_direct};
use crate::registry_pipeline::noise_settings_codecs::{
    climate_parameter_point, noise_generator_settings,
};
use crate::registry_pipeline::worldgen_common::{
    block_id, non_empty_list, registry_dispatch, Variant, Y_SIZE,
};
use crate::storage::nbt::Tag;

const BIOME_REGISTRY: &str = "minecraft:worldgen/biome";
const DIMENSION_TYPE_REGISTRY: &str = "minecraft:dimension_type";
const NOISE_SETTINGS_REGISTRY: &str = "minecraft:worldgen/noise_settings";
const STRUCTURE_SET_REGISTRY: &str = "minecraft:worldgen/structure_set";
const PLACED_FEATURE_REGISTRY: &str = "minecraft:worldgen/placed_feature";
const MULTI_NOISE_REGISTRY: &str = "minecraft:worldgen/multi_noise_biome_source_parameter_list";
const ITEM_REGISTRY: &str = "minecraft:item";

/// `Biome.CODEC` (`RegistryFileCodec`).
fn biome_holder() -> Codec {
    holder_file(BIOME_REGISTRY, biome_direct())
}

/// `RegistryOps.retrieveElement(key)`: reads nothing from the input but requires the
/// element to exist, and writes nothing.
fn retrieve_element(registry: &'static str, id: &'static str) -> Field {
    custom_field(move |_, ctx, _| {
        holder_fixed(registry)
            .parse(&Json::String(id.to_string()), ctx)
            .map(|_| ())
    })
}

// ---------------------------------------------------------------------------
// Multi-noise biome source parameter lists
// ---------------------------------------------------------------------------

/// `MultiNoiseBiomeSourceParameterList.DIRECT_CODEC`: `{preset}` naming a built-in
/// preset (`Preset.CODEC`).
pub fn multi_noise_parameter_list() -> Codec {
    record(vec![req("preset", multi_noise_preset())])
}

/// `MultiNoiseBiomeSourceParameterList.Preset.CODEC`.
fn multi_noise_preset() -> Codec {
    Codec::new(|json, ctx| {
        let Tag::String(name) = identifier_codec().parse(json, ctx)? else {
            return Err("preset did not encode to a string".to_string());
        };
        match name.as_str() {
            "minecraft:overworld" | "minecraft:nether" => Ok(Tag::String(name)),
            _ => Err(format!("Unknown preset: {name}")),
        }
    })
}

// ---------------------------------------------------------------------------
// Biome sources
// ---------------------------------------------------------------------------

/// `BiomeSource.CODEC`.
pub fn biome_source() -> Codec {
    registry_dispatch("type", "minecraft:worldgen/biome_source", BIOME_SOURCES)
}

/// `BiomeSources.bootstrap` registrations.
const BIOME_SOURCES: &[Variant] = &[
    ("fixed", fixed_biome_source),
    ("multi_noise", multi_noise_biome_source),
    ("checkerboard", checkerboard_biome_source),
    ("the_end", the_end_biome_source),
];

fn fixed_biome_source() -> Codec {
    record(vec![req("biome", biome_holder())])
}

/// `CheckerboardColumnBiomeSource.CODEC`.
fn checkerboard_biome_source() -> Codec {
    record(vec![
        req("biomes", holder_set(BIOME_REGISTRY, false)),
        or_else("scale", int_range(0, 62), json!(2)),
    ])
}

/// `MultiNoiseBiomeSource.CODEC`: `Codec.mapEither(DIRECT_CODEC, PRESET_CODEC)`, an
/// explicit `biomes` list or a `preset` reference.
fn multi_noise_biome_source() -> Codec {
    let parameters = record(vec![req(
        "biomes",
        non_empty_list(record(vec![
            req("parameters", climate_parameter_point()),
            req("biome", biome_holder()),
        ])),
    )]);
    let preset = record(vec![req(
        "preset",
        holder_file(MULTI_NOISE_REGISTRY, multi_noise_parameter_list()),
    )]);
    either(parameters, preset)
}

/// `TheEndBiomeSource.CODEC`: five `retrieveElement` entries, no data.
fn the_end_biome_source() -> Codec {
    record(vec![
        retrieve_element(BIOME_REGISTRY, "minecraft:the_end"),
        retrieve_element(BIOME_REGISTRY, "minecraft:end_highlands"),
        retrieve_element(BIOME_REGISTRY, "minecraft:end_midlands"),
        retrieve_element(BIOME_REGISTRY, "minecraft:small_end_islands"),
        retrieve_element(BIOME_REGISTRY, "minecraft:end_barrens"),
    ])
}

// ---------------------------------------------------------------------------
// Chunk generators and level stems
// ---------------------------------------------------------------------------

/// `ChunkGenerator.CODEC`.
pub fn chunk_generator() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:worldgen/chunk_generator",
        CHUNK_GENERATORS,
    )
}

/// `ChunkGenerators.bootstrap` registrations.
const CHUNK_GENERATORS: &[Variant] = &[
    ("noise", noise_chunk_generator),
    ("flat", flat_chunk_generator),
    ("debug", debug_chunk_generator),
];

/// `NoiseBasedChunkGenerator.CODEC`.
fn noise_chunk_generator() -> Codec {
    record(vec![
        req("biome_source", biome_source()),
        req(
            "settings",
            holder_file(NOISE_SETTINGS_REGISTRY, noise_generator_settings()),
        ),
    ])
}

/// `FlatLevelSource.CODEC`.
fn flat_chunk_generator() -> Codec {
    record(vec![req("settings", flat_level_generator_settings())])
}

/// `DebugLevelSource.CODEC`.
fn debug_chunk_generator() -> Codec {
    record(vec![retrieve_element(BIOME_REGISTRY, "minecraft:plains")])
}

/// `LevelStem.CODEC`.
pub fn level_stem() -> Codec {
    record(vec![
        req(
            "type",
            holder_file(DIMENSION_TYPE_REGISTRY, dimension_type_direct()),
        ),
        req("generator", chunk_generator()),
    ])
}

/// `WorldPreset.DIRECT_CODEC`: a `dimensions` map keyed by level stem, which must
/// contain the overworld.
pub fn world_preset() -> Codec {
    record(vec![req(
        "dimensions",
        unbounded_map(identifier_codec(), level_stem()),
    )])
    .validate(|tag| {
        let has_overworld = match tag {
            Tag::Compound(fields) => fields.iter().any(|(key, value)| {
                key == "dimensions"
                    && matches!(value, Tag::Compound(dimensions)
                        if dimensions.iter().any(|(id, _)| id == "minecraft:overworld"))
            }),
            _ => false,
        };
        if has_overworld {
            Ok(())
        } else {
            Err("Missing overworld dimension".to_string())
        }
    })
}

// ---------------------------------------------------------------------------
// Flat world presets
// ---------------------------------------------------------------------------

/// `FlatLevelGeneratorPreset.DIRECT_CODEC`.
pub fn flat_level_generator_preset() -> Codec {
    record(vec![
        req("display", holder_fixed(ITEM_REGISTRY)),
        req("settings", flat_level_generator_settings()),
    ])
}

/// `FlatLayerInfo.CODEC`: the block falls back to air.
fn flat_layer_info() -> Codec {
    record(vec![
        req("height", int_range(0, Y_SIZE)),
        or_else("block", block_id(), json!("minecraft:air")),
    ])
}

/// `FlatLevelGeneratorSettings.CODEC`. The biome is a lenient optional that defaults
/// to plains (with an error log in Java), and is always written back.
fn flat_level_generator_settings() -> Codec {
    let fields = vec![
        lenient_opt(
            "structure_overrides",
            holder_set(STRUCTURE_SET_REGISTRY, false),
        ),
        req("layers", list(flat_layer_info())),
        or_else("lakes", bool_codec(), json!(false)),
        or_else("features", bool_codec(), json!(false)),
        flat_biome(),
        retrieve_element(BIOME_REGISTRY, "minecraft:plains"),
        retrieve_element(PLACED_FEATURE_REGISTRY, "minecraft:lake_lava_underground"),
        retrieve_element(PLACED_FEATURE_REGISTRY, "minecraft:lake_lava_surface"),
    ];
    record(fields).validate(|tag| {
        let total: i64 = match tag {
            Tag::Compound(fields) => fields
                .iter()
                .filter_map(|(key, value)| match value {
                    Tag::List(layers) if key == "layers" => Some(layers),
                    _ => None,
                })
                .flatten()
                .filter_map(|layer| match layer {
                    Tag::Compound(inner) => inner.iter().find_map(|(key, value)| match value {
                        Tag::Int(height) if key == "height" => Some(i64::from(*height)),
                        _ => None,
                    }),
                    _ => None,
                })
                .sum(),
            _ => 0,
        };
        if total > i64::from(Y_SIZE) {
            Err(format!("Sum of layer heights is > {Y_SIZE}"))
        } else {
            Ok(())
        }
    })
}

/// `Biome.CODEC.lenientOptionalFieldOf("biome")` with the plains fallback: an absent or
/// undecodable biome becomes `minecraft:plains`, and the biome is always encoded.
fn flat_biome() -> Field {
    custom_field(|object, ctx: &CodecContext<'_>, out| {
        let parsed = object
            .get("biome")
            .and_then(|value| biome_holder().parse(value, ctx).ok());
        let tag = match parsed {
            Some(tag) => tag,
            None => holder_fixed(BIOME_REGISTRY).parse(&json!("minecraft:plains"), ctx)?,
        };
        out.push(("biome".to_string(), tag));
        Ok(())
    })
}
