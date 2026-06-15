use super::*;

const TREE_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/TreePlacements.java"
);

const TREE_PLACED_FEATURE_KEYS: [&str; 41] = [
    "minecraft:crimson_fungi",
    "minecraft:warped_fungi",
    "minecraft:oak_checked",
    "minecraft:dark_oak_checked",
    "minecraft:pale_oak_checked",
    "minecraft:pale_oak_creaking_checked",
    "minecraft:birch_checked",
    "minecraft:acacia_checked",
    "minecraft:spruce_checked",
    "minecraft:mangrove_checked",
    "minecraft:cherry_checked",
    "minecraft:pine_on_snow",
    "minecraft:spruce_on_snow",
    "minecraft:pine_checked",
    "minecraft:jungle_tree",
    "minecraft:fancy_oak_checked",
    "minecraft:mega_jungle_tree_checked",
    "minecraft:mega_spruce_checked",
    "minecraft:mega_pine_checked",
    "minecraft:tall_mangrove_checked",
    "minecraft:jungle_bush",
    "minecraft:super_birch_bees_0002",
    "minecraft:super_birch_bees",
    "minecraft:oak_bees_0002_leaf_litter",
    "minecraft:oak_bees_002",
    "minecraft:birch_bees_0002",
    "minecraft:birch_bees_0002_leaf_litter",
    "minecraft:birch_bees_002",
    "minecraft:fancy_oak_bees_0002_leaf_litter",
    "minecraft:fancy_oak_bees_002",
    "minecraft:fancy_oak_bees",
    "minecraft:cherry_bees_005",
    "minecraft:oak_leaf_litter",
    "minecraft:dark_oak_leaf_litter",
    "minecraft:birch_leaf_litter",
    "minecraft:fancy_oak_leaf_litter",
    "minecraft:fallen_oak_tree",
    "minecraft:fallen_birch_tree",
    "minecraft:fallen_super_birch_tree",
    "minecraft:fallen_spruce_tree",
    "minecraft:fallen_jungle_tree",
];

const TREE_FEATURE_REFS: [(&str, &str); 41] = [
    ("minecraft:crimson_fungi", "minecraft:crimson_fungus"),
    ("minecraft:warped_fungi", "minecraft:warped_fungus"),
    ("minecraft:oak_checked", "minecraft:oak"),
    ("minecraft:dark_oak_checked", "minecraft:dark_oak"),
    ("minecraft:pale_oak_checked", "minecraft:pale_oak"),
    (
        "minecraft:pale_oak_creaking_checked",
        "minecraft:pale_oak_creaking",
    ),
    ("minecraft:birch_checked", "minecraft:birch"),
    ("minecraft:acacia_checked", "minecraft:acacia"),
    ("minecraft:spruce_checked", "minecraft:spruce"),
    ("minecraft:mangrove_checked", "minecraft:mangrove"),
    ("minecraft:cherry_checked", "minecraft:cherry"),
    ("minecraft:pine_on_snow", "minecraft:pine"),
    ("minecraft:spruce_on_snow", "minecraft:spruce"),
    ("minecraft:pine_checked", "minecraft:pine"),
    ("minecraft:jungle_tree", "minecraft:jungle_tree"),
    ("minecraft:fancy_oak_checked", "minecraft:fancy_oak"),
    (
        "minecraft:mega_jungle_tree_checked",
        "minecraft:mega_jungle_tree",
    ),
    ("minecraft:mega_spruce_checked", "minecraft:mega_spruce"),
    ("minecraft:mega_pine_checked", "minecraft:mega_pine"),
    ("minecraft:tall_mangrove_checked", "minecraft:tall_mangrove"),
    ("minecraft:jungle_bush", "minecraft:jungle_bush"),
    (
        "minecraft:super_birch_bees_0002",
        "minecraft:super_birch_bees_0002",
    ),
    ("minecraft:super_birch_bees", "minecraft:super_birch_bees"),
    (
        "minecraft:oak_bees_0002_leaf_litter",
        "minecraft:oak_bees_0002_leaf_litter",
    ),
    ("minecraft:oak_bees_002", "minecraft:oak_bees_002"),
    ("minecraft:birch_bees_0002", "minecraft:birch_bees_0002"),
    (
        "minecraft:birch_bees_0002_leaf_litter",
        "minecraft:birch_bees_0002_leaf_litter",
    ),
    ("minecraft:birch_bees_002", "minecraft:birch_bees_002"),
    (
        "minecraft:fancy_oak_bees_0002_leaf_litter",
        "minecraft:fancy_oak_bees_0002_leaf_litter",
    ),
    (
        "minecraft:fancy_oak_bees_002",
        "minecraft:fancy_oak_bees_002",
    ),
    ("minecraft:fancy_oak_bees", "minecraft:fancy_oak_bees"),
    ("minecraft:cherry_bees_005", "minecraft:cherry_bees_005"),
    ("minecraft:oak_leaf_litter", "minecraft:oak_leaf_litter"),
    (
        "minecraft:dark_oak_leaf_litter",
        "minecraft:dark_oak_leaf_litter",
    ),
    ("minecraft:birch_leaf_litter", "minecraft:birch_leaf_litter"),
    (
        "minecraft:fancy_oak_leaf_litter",
        "minecraft:fancy_oak_leaf_litter",
    ),
    ("minecraft:fallen_oak_tree", "minecraft:fallen_oak_tree"),
    ("minecraft:fallen_birch_tree", "minecraft:fallen_birch_tree"),
    (
        "minecraft:fallen_super_birch_tree",
        "minecraft:fallen_super_birch_tree",
    ),
    (
        "minecraft:fallen_spruce_tree",
        "minecraft:fallen_spruce_tree",
    ),
    (
        "minecraft:fallen_jungle_tree",
        "minecraft:fallen_jungle_tree",
    ),
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

fn tree_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Tree)
        .expect("tree placed-feature bootstrap source")
        .keys
}

#[test]
fn tree_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(TREE_PLACEMENTS_JAVA.lines().count(), 152);
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        41
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "PlacementUtils.register("),
        41
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "filteredByBlockSurvival("),
        37
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "CountOnEveryLayerPlacement.of"),
        2
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "BiomeFilter.biome()"),
        2
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "EnvironmentScanPlacement.scanningFor"),
        1
    );
    assert_eq!(
        count_occurrences(TREE_PLACEMENTS_JAVA, "BlockPredicateFilter.forPredicate"),
        1
    );

    for sentinel in [
        "CRIMSON_FUNGI = PlacementUtils.createKey(\"crimson_fungi\")",
        "JUNGLE_TREE_CHECKED = PlacementUtils.createKey(\"jungle_tree\")",
        "FALLEN_JUNGLE_TREE = PlacementUtils.createKey(\"fallen_jungle_tree\")",
        "PlacementUtils.register(context, CRIMSON_FUNGI, crimsonFungus, CountOnEveryLayerPlacement.of(8), BiomeFilter.biome());",
        "BlockPredicate.matchesBlocks(Direction.DOWN.getUnitVec3i(), Blocks.SNOW_BLOCK, Blocks.POWDER_SNOW)",
        "EnvironmentScanPlacement.scanningFor(Direction.UP, BlockPredicate.not(BlockPredicate.matchesBlocks(Blocks.POWDER_SNOW)), 8)",
        "PlacementUtils.register(context, PALE_OAK_CREAKING_CHECKED, paleOakCreaking, PlacementUtils.filteredByBlockSurvival(Blocks.PALE_OAK_SAPLING));",
    ] {
        assert!(
            TREE_PLACEMENTS_JAVA.contains(sentinel),
            "missing TreePlacements sentinel {sentinel}"
        );
    }

    assert_eq!(tree_placed_feature_keys(), TREE_PLACED_FEATURE_KEYS);
    for key in TREE_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Tree),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn tree_placements_feature_refs_match_vanilla_json() {
    for (id, feature) in TREE_FEATURE_REFS {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
    }
}

#[test]
fn tree_placements_fungi_configs_match_vanilla_json() {
    for id in ["minecraft:crimson_fungi", "minecraft:warped_fungi"] {
        let placed = placed_feature_json(id);
        assert_eq!(
            placed["placement"][0]["type"],
            "minecraft:count_on_every_layer"
        );
        assert_eq!(placed["placement"][0]["count"], 8);
        assert_eq!(placed["placement"][1]["type"], "minecraft:biome");
        assert_eq!(placed["placement"].as_array().expect("placement").len(), 2);
    }
}

#[test]
fn tree_placements_checked_tree_survival_filters_match_vanilla_json() {
    for (id, sapling) in [
        ("minecraft:oak_checked", "minecraft:oak_sapling"),
        ("minecraft:dark_oak_checked", "minecraft:dark_oak_sapling"),
        (
            "minecraft:pale_oak_creaking_checked",
            "minecraft:pale_oak_sapling",
        ),
        ("minecraft:mangrove_checked", "minecraft:mangrove_propagule"),
        ("minecraft:jungle_tree", "minecraft:jungle_sapling"),
        ("minecraft:mega_pine_checked", "minecraft:spruce_sapling"),
        ("minecraft:super_birch_bees", "minecraft:birch_sapling"),
        (
            "minecraft:fancy_oak_bees_0002_leaf_litter",
            "minecraft:oak_sapling",
        ),
        ("minecraft:cherry_bees_005", "minecraft:cherry_sapling"),
        ("minecraft:fallen_spruce_tree", "minecraft:spruce_sapling"),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"].as_array().expect("placement").len(), 1);
        assert_eq!(
            placed["placement"][0]["type"], "minecraft:block_predicate_filter",
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["type"], "minecraft:would_survive",
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["state"]["Name"], sapling,
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["predicate"]["state"]["Properties"]["stage"], "0",
            "{id}"
        );
    }

    let survival_filter_count = TREE_PLACED_FEATURE_KEYS
        .iter()
        .map(|id| placed_feature_json(id))
        .filter(|placed| {
            placed["placement"][0]["type"] == "minecraft:block_predicate_filter"
                && placed["placement"][0]["predicate"]["type"] == "minecraft:would_survive"
        })
        .count();
    assert_eq!(survival_filter_count, 37);
}

#[test]
fn tree_placements_snow_tree_configs_match_vanilla_json() {
    for (id, feature) in [
        ("minecraft:pine_on_snow", "minecraft:pine"),
        ("minecraft:spruce_on_snow", "minecraft:spruce"),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"].as_array().expect("placement").len(), 2);
        assert_eq!(placed["placement"][0]["type"], "minecraft:environment_scan");
        assert_eq!(placed["placement"][0]["direction_of_search"], "up");
        assert_eq!(placed["placement"][0]["max_steps"], 8);
        assert_eq!(
            placed["placement"][0]["target_condition"]["type"],
            "minecraft:not"
        );
        assert_eq!(
            placed["placement"][0]["target_condition"]["predicate"]["type"],
            "minecraft:matching_blocks"
        );
        assert_eq!(
            placed["placement"][0]["target_condition"]["predicate"]["blocks"],
            "minecraft:powder_snow"
        );
        assert_eq!(
            placed["placement"][1]["type"],
            "minecraft:block_predicate_filter"
        );
        assert_eq!(
            placed["placement"][1]["predicate"]["type"],
            "minecraft:matching_blocks"
        );
        assert_eq!(
            placed["placement"][1]["predicate"]["blocks"],
            serde_json::json!(["minecraft:snow_block", "minecraft:powder_snow"])
        );
        assert_eq!(
            placed["placement"][1]["predicate"]["offset"],
            serde_json::json!([0, -1, 0])
        );
    }
}
