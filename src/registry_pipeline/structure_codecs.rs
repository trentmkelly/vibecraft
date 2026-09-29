//! Codecs for `worldgen/structure` (`Structure.DIRECT_CODEC` and every registered
//! `StructureType`) and `worldgen/structure_set` (`StructureSet.DIRECT_CODEC` with its
//! `StructurePlacement` types).

use serde_json::json;

use crate::registry_pipeline::codec::{
    bool_codec, either, enum_codec, float_range, holder_file, holder_fixed, holder_set,
    identifier_codec, int_range, list, non_negative_int, opt, opt_default, positive_float,
    positive_int, record, req, unbounded_map, Codec, Field,
};
use crate::registry_pipeline::structure_pool_codecs::{
    dimension_padding, liquid_settings, pool_alias_binding, template_pool_holder,
};
use crate::registry_pipeline::worldgen_common::{
    flat_weighted, height_provider, heightmap_types, non_empty_list, registry_dispatch,
    vec3i_offset, Variant, Y_SIZE,
};
use crate::storage::nbt::Tag;

/// `Registries.STRUCTURE`.
pub const STRUCTURE_REGISTRY: &str = "minecraft:worldgen/structure";
/// `Registries.STRUCTURE_SET`.
pub const STRUCTURE_SET_REGISTRY: &str = "minecraft:worldgen/structure_set";
const BIOME_REGISTRY: &str = "minecraft:worldgen/biome";
const ENTITY_TYPE_REGISTRY: &str = "minecraft:entity_type";

/// `MobCategory` serialised names.
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

/// `GenerationStep.Decoration` serialised names.
const DECORATION_STEPS: &[&str] = &[
    "raw_generation",
    "lakes",
    "local_modifications",
    "underground_structures",
    "surface_structures",
    "strongholds",
    "underground_ores",
    "underground_decoration",
    "fluid_springs",
    "vegetal_decoration",
    "top_layer_modification",
];

/// `TerrainAdjustment` serialised names.
const TERRAIN_ADJUSTMENTS: &[&str] = &["none", "bury", "beard_thin", "beard_box", "encapsulate"];

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// `Structure.CODEC` (`RegistryFileCodec`): a `worldgen/structure` reference or an
/// inline structure.
pub fn structure_holder() -> Codec {
    holder_file(STRUCTURE_REGISTRY, structure())
}

/// `Structure.DIRECT_CODEC`.
pub fn structure() -> Codec {
    registry_dispatch("type", "minecraft:worldgen/structure_type", STRUCTURE_TYPES)
}

/// `StructureType` registrations.
const STRUCTURE_TYPES: &[Variant] = &[
    ("buried_treasure", settings_only),
    ("desert_pyramid", settings_only),
    ("end_city", settings_only),
    ("fortress", settings_only),
    ("igloo", settings_only),
    ("jigsaw", jigsaw),
    ("jungle_temple", settings_only),
    ("mineshaft", mineshaft),
    ("nether_fossil", nether_fossil),
    ("ocean_monument", settings_only),
    ("ocean_ruin", ocean_ruin),
    ("ruined_portal", ruined_portal),
    ("shipwreck", shipwreck),
    ("stronghold", settings_only),
    ("swamp_hut", settings_only),
    ("woodland_mansion", settings_only),
];

/// `StructureSpawnOverride.CODEC`.
fn spawn_override() -> Codec {
    record(vec![
        req("bounding_box", enum_codec(&["piece", "full"])),
        req(
            "spawns",
            list(flat_weighted(vec![
                req("type", holder_fixed(ENTITY_TYPE_REGISTRY)),
                req("minCount", positive_int()),
                req("maxCount", positive_int()),
            ])),
        ),
    ])
}

/// `Structure.StructureSettings.CODEC`, flattened into the structure's own map.
fn settings_fields() -> Vec<Field> {
    vec![
        req("biomes", holder_set(BIOME_REGISTRY, false)),
        req(
            "spawn_overrides",
            unbounded_map(enum_codec(MOB_CATEGORIES), spawn_override()),
        ),
        req("step", enum_codec(DECORATION_STEPS)),
        opt_default(
            "terrain_adaptation",
            enum_codec(TERRAIN_ADJUSTMENTS),
            json!("none"),
        ),
    ]
}

/// `Structure.simpleCodec`: only the settings.
fn settings_only() -> Codec {
    record(settings_fields())
}

/// A structure record: the shared settings followed by `extra` fields.
fn with_settings(extra: Vec<Field>) -> Codec {
    let mut fields = settings_fields();
    fields.extend(extra);
    record(fields)
}

/// `JigsawStructure.CODEC` including `verifyRange`.
fn jigsaw() -> Codec {
    with_settings(vec![
        req("start_pool", template_pool_holder()),
        opt("start_jigsaw_name", identifier_codec()),
        req("size", int_range(0, 20)),
        req("start_height", height_provider()),
        req("use_expansion_hack", bool_codec()),
        opt("project_start_to_heightmap", heightmap_types()),
        req("max_distance_from_center", max_distance()),
        opt_default("pool_aliases", list(pool_alias_binding()), json!([])),
        opt_default("dimension_padding", dimension_padding(), json!(0)),
        opt_default(
            "liquid_settings",
            liquid_settings(),
            json!("apply_waterlogging"),
        ),
    ])
    .validate(|tag| {
        let Tag::Compound(fields) = tag else {
            return Ok(());
        };
        let find = |name: &str| fields.iter().find(|(key, _)| key == name).map(|(_, v)| v);
        let horizontal = match find("max_distance_from_center") {
            Some(Tag::Int(value)) => *value,
            Some(Tag::Compound(inner)) => inner
                .iter()
                .find_map(|(key, value)| match value {
                    Tag::Int(v) if key == "horizontal" => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            _ => return Ok(()),
        };
        let edge_needed = match find("terrain_adaptation") {
            Some(Tag::String(name)) if name != "none" => 12,
            _ => 0,
        };
        if horizontal + edge_needed > 128 {
            Err(
                "Horizontal structure size including terrain adaptation must not exceed 128"
                    .to_string(),
            )
        } else {
            Ok(())
        }
    })
}

/// `JigsawStructure.MaxDistance.CODEC`: `{horizontal, vertical?}` or a bare int used
/// for both. Equal horizontal and vertical distances are written as the bare int.
fn max_distance() -> Codec {
    let full = record(vec![
        req("horizontal", int_range(1, 128)),
        opt_default("vertical", int_range(1, Y_SIZE), json!(Y_SIZE)),
    ]);
    either(full, int_range(1, 128)).map_tag(|tag| {
        Ok(match &tag {
            Tag::Compound(fields) => {
                let horizontal = fields.iter().find_map(|(key, value)| match value {
                    Tag::Int(v) if key == "horizontal" => Some(*v),
                    _ => None,
                });
                let vertical = fields.iter().find_map(|(key, value)| match value {
                    Tag::Int(v) if key == "vertical" => Some(*v),
                    _ => None,
                });
                match (horizontal, vertical) {
                    (Some(h), Some(v)) if h == v => Tag::Int(h),
                    _ => tag,
                }
            }
            _ => tag,
        })
    })
}

/// `MineshaftStructure.CODEC`.
fn mineshaft() -> Codec {
    with_settings(vec![req("mineshaft_type", enum_codec(&["normal", "mesa"]))])
}

/// `NetherFossilStructure.CODEC`.
fn nether_fossil() -> Codec {
    with_settings(vec![req("height", height_provider())])
}

/// `OceanRuinStructure.CODEC`.
fn ocean_ruin() -> Codec {
    with_settings(vec![
        req("biome_temp", enum_codec(&["warm", "cold"])),
        req("large_probability", float_range(0.0, 1.0)),
        req("cluster_probability", float_range(0.0, 1.0)),
    ])
}

/// `ShipwreckStructure.CODEC`.
fn shipwreck() -> Codec {
    with_settings(vec![req("is_beached", bool_codec())])
}

/// `RuinedPortalStructure.CODEC`.
fn ruined_portal() -> Codec {
    let unit_range = || float_range(0.0, 1.0);
    let setup = record(vec![
        req(
            "placement",
            enum_codec(&[
                "on_land_surface",
                "partly_buried",
                "on_ocean_floor",
                "in_mountain",
                "underground",
                "in_nether",
            ]),
        ),
        req("air_pocket_probability", unit_range()),
        req("mossiness", unit_range()),
        req("overgrown", bool_codec()),
        req("vines", bool_codec()),
        req("can_be_cold", bool_codec()),
        req("replace_with_blackstone", bool_codec()),
        req("weight", positive_float()),
    ]);
    with_settings(vec![req("setups", non_empty_list(setup))])
}

// ---------------------------------------------------------------------------
// Structure sets and placements
// ---------------------------------------------------------------------------

/// `StructureSet.DIRECT_CODEC`.
pub fn structure_set() -> Codec {
    record(vec![
        req(
            "structures",
            list(record(vec![
                req("structure", structure_holder()),
                req("weight", positive_int()),
            ])),
        ),
        req("placement", structure_placement()),
    ])
}

/// `StructurePlacement.CODEC`.
fn structure_placement() -> Codec {
    registry_dispatch(
        "type",
        "minecraft:worldgen/structure_placement",
        STRUCTURE_PLACEMENTS,
    )
}

/// `StructurePlacementType` registrations.
const STRUCTURE_PLACEMENTS: &[Variant] = &[
    ("random_spread", random_spread),
    ("concentric_rings", concentric_rings),
];

/// `StructurePlacement.placementCodec`.
fn placement_fields() -> Vec<Field> {
    vec![
        opt_default("locate_offset", vec3i_offset(16), json!([0, 0, 0])),
        opt_default(
            "frequency_reduction_method",
            enum_codec(&["default", "legacy_type_1", "legacy_type_2", "legacy_type_3"]),
            json!("default"),
        ),
        opt_default("frequency", float_range(0.0, 1.0), json!(1.0)),
        req("salt", non_negative_int()),
        opt("exclusion_zone", exclusion_zone()),
    ]
}

/// `StructurePlacement.ExclusionZone.CODEC`: `other_set` is a reference only
/// (`RegistryFileCodec.create(.., false)`).
fn exclusion_zone() -> Codec {
    record(vec![
        req("other_set", holder_fixed(STRUCTURE_SET_REGISTRY)),
        req("chunk_count", int_range(1, 16)),
    ])
}

/// `RandomSpreadStructurePlacement.CODEC`.
fn random_spread() -> Codec {
    let mut fields = placement_fields();
    fields.extend([
        req("spacing", int_range(0, 4096)),
        req("separation", int_range(0, 4096)),
        opt_default(
            "spread_type",
            enum_codec(&["linear", "triangular"]),
            json!("linear"),
        ),
    ]);
    record(fields).validate(|tag| {
        let Tag::Compound(fields) = tag else {
            return Ok(());
        };
        let get = |name: &str| {
            fields.iter().find_map(|(key, value)| match value {
                Tag::Int(v) if key == name => Some(*v),
                _ => None,
            })
        };
        match (get("spacing"), get("separation")) {
            (Some(spacing), Some(separation)) if spacing <= separation => {
                Err("Spacing has to be larger than separation".to_string())
            }
            _ => Ok(()),
        }
    })
}

/// `ConcentricRingsStructurePlacement.CODEC`.
fn concentric_rings() -> Codec {
    let mut fields = placement_fields();
    fields.extend([
        req("distance", int_range(0, 1023)),
        req("spread", int_range(0, 1023)),
        req("count", int_range(1, 4095)),
        req("preferred_biomes", holder_set(BIOME_REGISTRY, false)),
    ]);
    record(fields)
}
