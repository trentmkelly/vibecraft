use super::*;

const VILLAGE_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/VillagePlacements.java"
);

const VILLAGE_PLACED_FEATURE_KEYS: [&str; 13] = [
    "minecraft:pile_hay",
    "minecraft:pile_melon",
    "minecraft:pile_snow",
    "minecraft:pile_ice",
    "minecraft:pile_pumpkin",
    "minecraft:oak",
    "minecraft:acacia",
    "minecraft:spruce",
    "minecraft:pine",
    "minecraft:patch_cactus",
    "minecraft:flower_plain",
    "minecraft:patch_taiga_grass",
    "minecraft:patch_berry_bush",
];

const VILLAGE_FEATURE_REFS: [(&str, &str); 13] = [
    ("minecraft:pile_hay", "minecraft:pile_hay"),
    ("minecraft:pile_melon", "minecraft:pile_melon"),
    ("minecraft:pile_snow", "minecraft:pile_snow"),
    ("minecraft:pile_ice", "minecraft:pile_ice"),
    ("minecraft:pile_pumpkin", "minecraft:pile_pumpkin"),
    ("minecraft:oak", "minecraft:oak"),
    ("minecraft:acacia", "minecraft:acacia"),
    ("minecraft:spruce", "minecraft:spruce"),
    ("minecraft:pine", "minecraft:pine"),
    ("minecraft:patch_cactus", "minecraft:cactus"),
    ("minecraft:flower_plain", "minecraft:flower_plain"),
    ("minecraft:patch_taiga_grass", "minecraft:taiga_grass"),
    ("minecraft:patch_berry_bush", "minecraft:berry_bush"),
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn placed_feature_json(id: &str) -> serde_json::Value {
    let path = super::vanilla_data_path(&["data", "minecraft", "worldgen", "placed_feature"])
        .join(format!("{}.json", id.trim_start_matches("minecraft:")));
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"));
    serde_json::from_str(&json).unwrap_or_else(|err| panic!("failed to parse {path:?}: {err}"))
}

fn village_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Village)
        .expect("village placed-feature bootstrap source")
        .keys
}

#[test]
fn village_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(VILLAGE_PLACEMENTS_JAVA.lines().count(), 97);
    assert_eq!(
        count_occurrences(VILLAGE_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        13
    );
    assert_eq!(
        count_occurrences(VILLAGE_PLACEMENTS_JAVA, "PlacementUtils.register("),
        13
    );
    assert_eq!(
        count_occurrences(
            VILLAGE_PLACEMENTS_JAVA,
            "PlacementUtils.filteredByBlockSurvival"
        ),
        4
    );
    assert_eq!(
        count_occurrences(VILLAGE_PLACEMENTS_JAVA, "Blocks.SPRUCE_SAPLING"),
        2
    );

    for sentinel in [
        "PILE_HAY_VILLAGE = PlacementUtils.createKey(\"pile_hay\")",
        "PATCH_BERRY_BUSH_VILLAGE = PlacementUtils.createKey(\"patch_berry_bush\")",
        "PlacementUtils.register(context, PILE_PUMPKIN_VILLAGE, pilePumpkin);",
        "PlacementUtils.register(context, PINE_VILLAGE, pine, PlacementUtils.filteredByBlockSurvival(Blocks.SPRUCE_SAPLING));",
        "RandomOffsetPlacement.ofTriangle(6, 2)",
        "BlockPredicate.matchesBlocks(Direction.DOWN.getUnitVec3i(), Blocks.GRASS_BLOCK)",
    ] {
        assert!(
            VILLAGE_PLACEMENTS_JAVA.contains(sentinel),
            "missing VillagePlacements sentinel {sentinel}"
        );
    }

    assert_eq!(village_placed_feature_keys(), VILLAGE_PLACED_FEATURE_KEYS);
    for key in VILLAGE_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Village),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn village_placements_feature_refs_match_vanilla_json() {
    for (id, feature) in VILLAGE_FEATURE_REFS {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
    }
}

#[test]
fn village_placements_pile_and_tree_configs_match_vanilla_json() {
    for id in [
        "minecraft:pile_hay",
        "minecraft:pile_melon",
        "minecraft:pile_snow",
        "minecraft:pile_ice",
        "minecraft:pile_pumpkin",
    ] {
        let placed = placed_feature_json(id);
        assert!(
            placed["placement"]
                .as_array()
                .expect("placement")
                .is_empty(),
            "{id}"
        );
    }

    for (id, sapling) in [
        ("minecraft:oak", "minecraft:oak_sapling"),
        ("minecraft:acacia", "minecraft:acacia_sapling"),
        ("minecraft:spruce", "minecraft:spruce_sapling"),
        ("minecraft:pine", "minecraft:spruce_sapling"),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"].as_array().expect("placement").len(), 1);
        assert_eq!(
            placed["placement"][0]["type"],
            "minecraft:block_predicate_filter"
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["type"],
            "minecraft:would_survive"
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["state"]["Name"],
            sapling
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["state"]["Properties"]["stage"],
            "0"
        );
    }
}

#[test]
fn village_placements_patch_configs_match_vanilla_json() {
    for (id, count, xz_spread, y_spread) in [
        ("minecraft:patch_cactus", 10, 7, 3),
        ("minecraft:flower_plain", 64, 6, 2),
        ("minecraft:patch_taiga_grass", 32, 7, 3),
        ("minecraft:patch_berry_bush", 96, 7, 3),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:random_offset",
            "{id}"
        );
        assert_eq!(
            placed["placement"][1]["xz_spread"]["min"], -xz_spread,
            "{id}"
        );
        assert_eq!(
            placed["placement"][1]["xz_spread"]["max"], xz_spread,
            "{id}"
        );
        assert_eq!(placed["placement"][1]["y_spread"]["min"], -y_spread, "{id}");
        assert_eq!(placed["placement"][1]["y_spread"]["max"], y_spread, "{id}");
        assert_eq!(
            placed["placement"][2]["type"], "minecraft:block_predicate_filter",
            "{id}"
        );
    }

    let cactus = placed_feature_json("minecraft:patch_cactus");
    assert_eq!(
        cactus["placement"][2]["predicate"]["type"],
        "minecraft:all_of"
    );
    assert_eq!(
        cactus["placement"][2]["predicate"]["predicates"][1]["state"]["Name"],
        "minecraft:cactus"
    );

    let berry = placed_feature_json("minecraft:patch_berry_bush");
    assert_eq!(
        berry["placement"][2]["predicate"]["type"],
        "minecraft:all_of"
    );
    assert_eq!(
        berry["placement"][2]["predicate"]["predicates"][1]["blocks"],
        "minecraft:grass_block"
    );
    assert_eq!(
        berry["placement"][2]["predicate"]["predicates"][1]["offset"],
        serde_json::json!([0, -1, 0])
    );
}
