use std::collections::BTreeMap;

use crate::worldgen::{RandomSpreadType, StructurePlacementKind, BUILTIN_STRUCTURE_SETS};

const STRUCTURE_SETS_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/worldgen/StructureSets.java");
const STRUCTURE_SET_ROOT: &str =
    "../decompiled-server-26.1.2/data/minecraft/worldgen/structure_set";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedStructureSet {
    structures: Vec<(String, i32)>,
    placement: ParsedStructureSetPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedStructureSetPlacement {
    RandomSpread {
        spacing: i32,
        separation: i32,
        salt: i32,
        spread_type: String,
        frequency_reduction_method: Option<String>,
        frequency_millionths: Option<i32>,
        exclusion_zone: Option<(String, i32)>,
        locate_offset: Option<(i32, i32, i32)>,
    },
    ConcentricRings {
        distance: i32,
        spread: i32,
        count: i32,
        preferred_biomes: String,
    },
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "StructureSets.java is missing sentinel: {sentinel}"
        );
    }
}

fn json_object<'a>(
    value: &'a serde_json::Value,
    label: &str,
) -> &'a serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{label} should be an object"))
}

fn json_array<'a>(value: &'a serde_json::Value, label: &str) -> &'a [serde_json::Value] {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("{label} should be an array"))
}

fn string_field(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> String {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("missing string field {key}"))
        .to_string()
}

fn i32_field(object: &serde_json::Map<String, serde_json::Value>, key: &str) -> i32 {
    let value = object
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_else(|| panic!("missing integer field {key}"));
    i32::try_from(value).unwrap_or_else(|_| panic!("integer field {key} overflows i32"))
}

fn parse_frequency_millionths(object: &serde_json::Map<String, serde_json::Value>) -> Option<i32> {
    object.get("frequency").map(|value| {
        let number = value
            .as_f64()
            .unwrap_or_else(|| panic!("frequency should be numeric"));
        (number * 1_000_000.0).round() as i32
    })
}

fn parse_locate_offset(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Option<(i32, i32, i32)> {
    object.get("locate_offset").map(|value| {
        let offset = json_array(value, "locate_offset");
        assert_eq!(offset.len(), 3);
        let coord = |index: usize| {
            let raw = offset[index]
                .as_i64()
                .unwrap_or_else(|| panic!("locate_offset coordinate should be integer"));
            i32::try_from(raw).unwrap_or_else(|_| panic!("locate_offset coordinate overflows i32"))
        };
        (coord(0), coord(1), coord(2))
    })
}

fn parse_structure_set_value(value: &serde_json::Value) -> ParsedStructureSet {
    let object = json_object(value, "structure set");
    let structures = json_array(
        object
            .get("structures")
            .unwrap_or_else(|| panic!("structure set missing structures")),
        "structures",
    )
    .iter()
    .map(|entry| {
        let entry = json_object(entry, "structure set entry");
        (string_field(entry, "structure"), i32_field(entry, "weight"))
    })
    .collect::<Vec<_>>();

    let placement = json_object(
        object
            .get("placement")
            .unwrap_or_else(|| panic!("structure set missing placement")),
        "placement",
    );
    let placement = match string_field(placement, "type").as_str() {
        "minecraft:random_spread" => ParsedStructureSetPlacement::RandomSpread {
            spacing: i32_field(placement, "spacing"),
            separation: i32_field(placement, "separation"),
            salt: i32_field(placement, "salt"),
            spread_type: placement
                .get("spread_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("linear")
                .to_string(),
            frequency_reduction_method: placement
                .get("frequency_reduction_method")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            frequency_millionths: parse_frequency_millionths(placement),
            exclusion_zone: placement.get("exclusion_zone").map(|value| {
                let exclusion = json_object(value, "exclusion_zone");
                (
                    string_field(exclusion, "other_set"),
                    i32_field(exclusion, "chunk_count"),
                )
            }),
            locate_offset: parse_locate_offset(placement),
        },
        "minecraft:concentric_rings" => ParsedStructureSetPlacement::ConcentricRings {
            distance: i32_field(placement, "distance"),
            spread: i32_field(placement, "spread"),
            count: i32_field(placement, "count"),
            preferred_biomes: string_field(placement, "preferred_biomes"),
        },
        other => panic!("unsupported structure-set placement type {other}"),
    };

    ParsedStructureSet {
        structures,
        placement,
    }
}

fn load_vanilla_structure_sets() -> BTreeMap<String, ParsedStructureSet> {
    let root = std::path::Path::new(STRUCTURE_SET_ROOT);
    let mut sets = BTreeMap::new();
    for entry in std::fs::read_dir(root)
        .unwrap_or_else(|err| panic!("failed to read structure_set dir {root:?}: {err}"))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read structure_set entry: {err}"));
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_else(|| panic!("invalid structure_set path {path:?}"));
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read structure_set JSON {path:?}: {err}"));
        let value = serde_json::from_str::<serde_json::Value>(&raw)
            .unwrap_or_else(|err| panic!("invalid structure_set JSON {path:?}: {err}"));
        sets.insert(format!("minecraft:{id}"), parse_structure_set_value(&value));
    }
    sets
}

fn vanilla_structure_set<'a>(
    sets: &'a BTreeMap<String, ParsedStructureSet>,
    id: &str,
) -> &'a ParsedStructureSet {
    sets.get(id)
        .unwrap_or_else(|| panic!("missing vanilla structure set {id}"))
}

fn rust_structure_set(id: &str) -> &'static crate::worldgen::StructureSetEntry {
    BUILTIN_STRUCTURE_SETS
        .iter()
        .find(|set| set.id == id)
        .unwrap_or_else(|| panic!("missing Rust structure set {id}"))
}

#[test]
fn structure_sets_java_bootstrap_shape_matches_decompilation() {
    assert_eq!(STRUCTURE_SETS_JAVA.lines().count(), 177);
    assert_source_contains_all(
        STRUCTURE_SETS_JAVA,
        &[
            "HolderGetter<Structure> structures = context.lookup(Registries.STRUCTURE);",
            "HolderGetter<Biome> biomes = context.lookup(Registries.BIOME);",
            "Holder.Reference<StructureSet> villages = context.register(",
            "BuiltinStructureSets.VILLAGES",
            "StructureSet.entry(structures.getOrThrow(BuiltinStructures.VILLAGE_TAIGA))",
            "StructurePlacement.FrequencyReductionMethod.LEGACY_TYPE_1",
            "Optional.of(new StructurePlacement.ExclusionZone(villages, 10))",
            "new Vec3i(9, 0, 9), StructurePlacement.FrequencyReductionMethod.LEGACY_TYPE_2, 0.01F, 0, Optional.empty(), 1, 0, RandomSpreadType.LINEAR",
            "StructureSet.entry(structures.getOrThrow(BuiltinStructures.FORTRESS), 2)",
            "StructureSet.entry(structures.getOrThrow(BuiltinStructures.BASTION_REMNANT), 3)",
            "new ConcentricRingsStructurePlacement(32, 3, 128, biomes.getOrThrow(BiomeTags.STRONGHOLD_BIASED_TO))",
            "new RandomSpreadStructurePlacement(34, 12, RandomSpreadType.LINEAR, 94251327)",
        ],
    );
    assert_eq!(
        count_occurrences(STRUCTURE_SETS_JAVA, "context.register("),
        20
    );
    assert_eq!(
        count_occurrences(STRUCTURE_SETS_JAVA, "new RandomSpreadStructurePlacement"),
        19
    );
    assert_eq!(
        count_occurrences(STRUCTURE_SETS_JAVA, "new ConcentricRingsStructurePlacement"),
        1
    );
    assert_eq!(
        count_occurrences(STRUCTURE_SETS_JAVA, "StructureSet.entry"),
        20
    );
    assert_eq!(
        count_occurrences(STRUCTURE_SETS_JAVA, "BuiltinStructures."),
        34
    );
}

#[test]
fn rust_structure_set_order_and_placements_match_java_surface() {
    assert_eq!(BUILTIN_STRUCTURE_SETS.len(), 20);
    assert_eq!(
        BUILTIN_STRUCTURE_SETS
            .iter()
            .map(|set| set.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:villages",
            "minecraft:desert_pyramids",
            "minecraft:igloos",
            "minecraft:jungle_temples",
            "minecraft:swamp_huts",
            "minecraft:pillager_outposts",
            "minecraft:ancient_cities",
            "minecraft:ocean_monuments",
            "minecraft:woodland_mansions",
            "minecraft:buried_treasures",
            "minecraft:mineshafts",
            "minecraft:ruined_portals",
            "minecraft:shipwrecks",
            "minecraft:ocean_ruins",
            "minecraft:nether_complexes",
            "minecraft:nether_fossils",
            "minecraft:end_cities",
            "minecraft:strongholds",
            "minecraft:trail_ruins",
            "minecraft:trial_chambers",
        ]
    );

    assert_eq!(
        rust_structure_set("minecraft:villages").placement,
        StructurePlacementKind::RandomSpread {
            spacing: 34,
            separation: 8,
            salt: 10387312,
            spread_type: RandomSpreadType::Linear,
        }
    );
    assert_eq!(
        rust_structure_set("minecraft:ocean_monuments").placement,
        StructurePlacementKind::RandomSpread {
            spacing: 32,
            separation: 5,
            salt: 10387313,
            spread_type: RandomSpreadType::Triangular,
        }
    );
    assert_eq!(
        rust_structure_set("minecraft:strongholds").placement,
        StructurePlacementKind::ConcentricRings {
            distance: 32,
            spread: 3,
            count: 128,
        }
    );
}

#[test]
fn vanilla_structure_set_json_ids_and_rust_static_entries_match() {
    let registry = load_vanilla_structure_sets();
    assert_eq!(registry.len(), BUILTIN_STRUCTURE_SETS.len());
    for set in BUILTIN_STRUCTURE_SETS {
        let vanilla = vanilla_structure_set(&registry, set.id);
        assert_eq!(
            vanilla
                .structures
                .iter()
                .map(|(structure, _)| structure.as_str())
                .collect::<Vec<_>>(),
            set.structures
        );
        match (&vanilla.placement, set.placement) {
            (
                ParsedStructureSetPlacement::RandomSpread {
                    spacing,
                    separation,
                    salt,
                    spread_type,
                    ..
                },
                StructurePlacementKind::RandomSpread {
                    spacing: rust_spacing,
                    separation: rust_separation,
                    salt: rust_salt,
                    spread_type: rust_spread_type,
                },
            ) => {
                assert_eq!(
                    (*spacing, *separation, *salt),
                    (rust_spacing, rust_separation, rust_salt)
                );
                assert_eq!(
                    spread_type.as_str(),
                    match rust_spread_type {
                        RandomSpreadType::Linear => "linear",
                        RandomSpreadType::Triangular => "triangular",
                    }
                );
            }
            (
                ParsedStructureSetPlacement::ConcentricRings {
                    distance,
                    spread,
                    count,
                    ..
                },
                StructurePlacementKind::ConcentricRings {
                    distance: rust_distance,
                    spread: rust_spread,
                    count: rust_count,
                },
            ) => assert_eq!(
                (*distance, *spread, *count),
                (rust_distance, rust_spread, rust_count)
            ),
            (vanilla, rust) => panic!("placement kind mismatch: {vanilla:?} vs {rust:?}"),
        }
    }
}

#[test]
fn vanilla_structure_set_json_special_cases_match_java_bootstrap() {
    let registry = load_vanilla_structure_sets();

    let outposts = vanilla_structure_set(&registry, "minecraft:pillager_outposts");
    assert_eq!(
        outposts.placement,
        ParsedStructureSetPlacement::RandomSpread {
            spacing: 32,
            separation: 8,
            salt: 165745296,
            spread_type: "linear".to_string(),
            frequency_reduction_method: Some("legacy_type_1".to_string()),
            frequency_millionths: Some(200_000),
            exclusion_zone: Some(("minecraft:villages".to_string(), 10)),
            locate_offset: None,
        }
    );

    let buried = vanilla_structure_set(&registry, "minecraft:buried_treasures");
    assert_eq!(
        buried.placement,
        ParsedStructureSetPlacement::RandomSpread {
            spacing: 1,
            separation: 0,
            salt: 0,
            spread_type: "linear".to_string(),
            frequency_reduction_method: Some("legacy_type_2".to_string()),
            frequency_millionths: Some(10_000),
            exclusion_zone: None,
            locate_offset: Some((9, 0, 9)),
        }
    );

    let mineshafts = vanilla_structure_set(&registry, "minecraft:mineshafts");
    assert_eq!(
        mineshafts.placement,
        ParsedStructureSetPlacement::RandomSpread {
            spacing: 1,
            separation: 0,
            salt: 0,
            spread_type: "linear".to_string(),
            frequency_reduction_method: Some("legacy_type_3".to_string()),
            frequency_millionths: Some(4_000),
            exclusion_zone: None,
            locate_offset: None,
        }
    );

    let nether_complexes = vanilla_structure_set(&registry, "minecraft:nether_complexes");
    assert_eq!(
        nether_complexes.structures,
        vec![
            ("minecraft:fortress".to_string(), 2),
            ("minecraft:bastion_remnant".to_string(), 3),
        ]
    );

    let strongholds = vanilla_structure_set(&registry, "minecraft:strongholds");
    assert_eq!(
        strongholds.placement,
        ParsedStructureSetPlacement::ConcentricRings {
            distance: 32,
            spread: 3,
            count: 128,
            preferred_biomes: "#minecraft:stronghold_biased_to".to_string(),
        }
    );
}
