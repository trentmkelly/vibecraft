use super::*;

const VEGETATION_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/VegetationFeatures.java"
);

const VEGETATION_FEATURE_KEYS: [&str; 57] = [
    "minecraft:bamboo_no_podzol",
    "minecraft:bamboo_some_podzol",
    "minecraft:vines",
    "minecraft:brown_mushroom",
    "minecraft:red_mushroom",
    "minecraft:sunflower",
    "minecraft:pumpkin",
    "minecraft:berry_bush",
    "minecraft:taiga_grass",
    "minecraft:grass",
    "minecraft:grass_jungle",
    "minecraft:dead_bush",
    "minecraft:dry_grass",
    "minecraft:melon",
    "minecraft:waterlily",
    "minecraft:tall_grass",
    "minecraft:large_fern",
    "minecraft:bush",
    "minecraft:leaf_litter",
    "minecraft:firefly_bush",
    "minecraft:cactus",
    "minecraft:sugar_cane",
    "minecraft:flower_default",
    "minecraft:flower_flower_forest",
    "minecraft:flower_swamp",
    "minecraft:flower_plain",
    "minecraft:flower_meadow",
    "minecraft:flower_cherry",
    "minecraft:flower_pale_garden",
    "minecraft:wildflower",
    "minecraft:forest_flowers",
    "minecraft:pale_forest_flower",
    "minecraft:dark_forest_vegetation",
    "minecraft:pale_garden_vegetation",
    "minecraft:pale_moss_vegetation",
    "minecraft:pale_moss_patch",
    "minecraft:pale_moss_patch_bonemeal",
    "minecraft:trees_flower_forest",
    "minecraft:meadow_trees",
    "minecraft:trees_taiga",
    "minecraft:trees_badlands",
    "minecraft:trees_grove",
    "minecraft:trees_savanna",
    "minecraft:trees_snowy",
    "minecraft:trees_birch",
    "minecraft:birch_tall",
    "minecraft:trees_windswept_hills",
    "minecraft:trees_water",
    "minecraft:trees_birch_and_oak_leaf_litter",
    "minecraft:trees_plains",
    "minecraft:trees_sparse_jungle",
    "minecraft:trees_old_growth_spruce_taiga",
    "minecraft:trees_old_growth_pine_taiga",
    "minecraft:trees_jungle",
    "minecraft:bamboo_vegetation",
    "minecraft:mushroom_island_vegetation",
    "minecraft:mangrove_vegetation",
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn configured_feature_json(id: &str) -> serde_json::Value {
    let path = format!(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/configured_feature/{}.json",
        id.trim_start_matches("minecraft:")
    );
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
    serde_json::from_str(&json).unwrap_or_else(|err| panic!("failed to parse {path}: {err}"))
}

fn vegetation_feature_keys() -> Vec<&'static str> {
    CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Vegetation)
        .map(|feature| feature.id)
        .collect()
}

#[test]
fn vegetation_features_java_bootstrap_matches_configured_feature_registry() {
    assert_eq!(VEGETATION_FEATURES_JAVA.lines().count(), 657);
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "FeatureUtils.createKey("),
        57
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "FeatureUtils.register("),
        57
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "SimpleBlockConfiguration"),
        32
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "RandomFeatureConfiguration"),
        24
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "WeightedPlacedFeature"),
        54
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "BlockColumnConfiguration"),
        5
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "VegetationPatchConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(VEGETATION_FEATURES_JAVA, "PlacementUtils.inlinePlaced"),
        13
    );

    for sentinel in [
        "BAMBOO_NO_PODZOL = FeatureUtils.createKey(\"bamboo_no_podzol\")",
        "MANGROVE_VEGETATION = FeatureUtils.createKey(\"mangrove_vegetation\")",
        "private static final float FALLEN_TREE_ONE_IN_CHANCE = 80.0F;",
        "new ProbabilityFeatureConfiguration(0.2F)",
        "new WeightedPlacedFeature(fallenOak, 0.0125F)",
        "new WeightedPlacedFeature(fallenSuperBirch, 0.00625F)",
        "new WeightedPlacedFeature(megaPineChecked, 0.30769232F)",
        "BlockPredicate.not(BlockPredicate.matchesBlocks(Direction.DOWN.getUnitVec3i(), Blocks.PODZOL))",
        "segmentedBlockPatchBuilder(Blocks.LEAF_LITTER, minState, maxState, LeafLitterBlock.AMOUNT, LeafLitterBlock.FACING)",
        "BlockPredicate.matchesFluids(new BlockPos(0, -1, -1), Fluids.WATER, Fluids.FLOWING_WATER)",
    ] {
        assert!(
            VEGETATION_FEATURES_JAVA.contains(sentinel),
            "missing VegetationFeatures sentinel {sentinel}"
        );
    }

    assert_eq!(vegetation_feature_keys(), VEGETATION_FEATURE_KEYS);
}

#[test]
fn vegetation_features_keys_and_types_match_vanilla_json() {
    for key in VEGETATION_FEATURE_KEYS {
        let expected_type = match key {
            "minecraft:bamboo_no_podzol" | "minecraft:bamboo_some_podzol" => "minecraft:bamboo",
            "minecraft:vines" => "minecraft:vines",
            "minecraft:cactus" | "minecraft:sugar_cane" => "minecraft:block_column",
            "minecraft:forest_flowers" => "minecraft:simple_random_selector",
            "minecraft:pale_moss_patch" | "minecraft:pale_moss_patch_bonemeal" => {
                "minecraft:vegetation_patch"
            }
            "minecraft:pale_moss_vegetation" => "minecraft:simple_block",
            "minecraft:mushroom_island_vegetation" => "minecraft:random_boolean_selector",
            key if key.contains("vegetation")
                || key.starts_with("minecraft:trees_")
                || key == "minecraft:meadow_trees"
                || key == "minecraft:birch_tall" =>
            {
                "minecraft:random_selector"
            }
            _ => "minecraft:simple_block",
        };
        assert_eq!(
            configured_feature_json(key)["type"],
            expected_type,
            "wrong configured-feature type for {key}"
        );
    }
}

#[test]
fn vegetation_features_simple_and_column_configs_match_vanilla_json() {
    let bamboo = configured_feature_json("minecraft:bamboo_some_podzol");
    assert_eq!(bamboo["config"]["probability"], 0.2);

    let berry = configured_feature_json("minecraft:berry_bush");
    assert_eq!(
        berry["config"]["to_place"]["state"]["Name"],
        "minecraft:sweet_berry_bush"
    );
    assert_eq!(
        berry["config"]["to_place"]["state"]["Properties"]["age"],
        "3"
    );

    let taiga_grass = configured_feature_json("minecraft:taiga_grass");
    assert_eq!(
        taiga_grass["config"]["to_place"]["entries"][0]["data"]["Name"],
        "minecraft:short_grass"
    );
    assert_eq!(
        taiga_grass["config"]["to_place"]["entries"][1]["data"]["Name"],
        "minecraft:fern"
    );
    assert_eq!(taiga_grass["config"]["to_place"]["entries"][1]["weight"], 4);

    let leaf_litter = configured_feature_json("minecraft:leaf_litter");
    assert_eq!(
        leaf_litter["config"]["to_place"]["entries"]
            .as_array()
            .expect("leaf litter entries")
            .len(),
        12
    );
    assert_eq!(
        leaf_litter["config"]["to_place"]["entries"][11]["data"]["Properties"]["segment_amount"],
        "3"
    );

    let cactus = configured_feature_json("minecraft:cactus");
    assert_eq!(cactus["config"]["direction"], "up");
    assert_eq!(
        cactus["config"]["layers"][0]["height"]["type"],
        "minecraft:biased_to_bottom"
    );
    assert_eq!(cactus["config"]["layers"][0]["height"]["max_inclusive"], 3);
    assert_eq!(
        cactus["config"]["layers"][1]["provider"]["state"]["Name"],
        "minecraft:cactus_flower"
    );
    assert_eq!(
        cactus["config"]["layers"][1]["height"]["distribution"][0]["weight"],
        3
    );

    let sugar_cane = configured_feature_json("minecraft:sugar_cane");
    assert_eq!(
        sugar_cane["config"]["layers"][0]["provider"]["state"]["Name"],
        "minecraft:sugar_cane"
    );
    assert_eq!(
        sugar_cane["config"]["layers"][0]["height"]["min_inclusive"],
        2
    );
    assert_eq!(
        sugar_cane["config"]["layers"][0]["height"]["max_inclusive"],
        4
    );
}

#[test]
fn vegetation_features_flower_configs_match_vanilla_json() {
    let flower_plain = configured_feature_json("minecraft:flower_plain");
    assert_eq!(
        flower_plain["config"]["to_place"]["type"],
        "minecraft:noise_threshold_provider"
    );
    assert_eq!(flower_plain["config"]["to_place"]["seed"], 2345);
    assert_eq!(flower_plain["config"]["to_place"]["scale"], 0.005);
    assert_eq!(flower_plain["config"]["to_place"]["threshold"], -0.8);
    assert_eq!(
        flower_plain["config"]["to_place"]["low_states"]
            .as_array()
            .expect("plain flower low states")
            .len(),
        4
    );
    assert_eq!(
        flower_plain["config"]["to_place"]["high_states"][3]["Name"],
        "minecraft:cornflower"
    );

    let meadow = configured_feature_json("minecraft:flower_meadow");
    assert_eq!(
        meadow["config"]["to_place"]["type"],
        "minecraft:dual_noise_provider"
    );
    assert_eq!(
        meadow["config"]["to_place"]["variety"],
        serde_json::json!([1, 3])
    );
    assert_eq!(
        meadow["config"]["to_place"]["slow_noise"]["firstOctave"],
        -10
    );
    assert_eq!(meadow["config"]["to_place"]["noise"]["firstOctave"], -3);
    assert_eq!(
        meadow["config"]["to_place"]["states"][7]["Name"],
        "minecraft:short_grass"
    );

    let cherry = configured_feature_json("minecraft:flower_cherry");
    assert_eq!(
        cherry["config"]["to_place"]["entries"]
            .as_array()
            .expect("cherry flower bed entries")
            .len(),
        16
    );
    assert_eq!(
        cherry["config"]["to_place"]["entries"][15]["data"]["Properties"]["flower_amount"],
        "4"
    );

    let forest = configured_feature_json("minecraft:forest_flowers");
    assert_eq!(
        forest["config"]["features"]
            .as_array()
            .expect("forest flower features")
            .len(),
        4
    );
    assert_eq!(
        forest["config"]["features"][0]["feature"]["config"]["to_place"]["state"]["Name"],
        "minecraft:lilac"
    );
    assert_eq!(forest["config"]["features"][0]["placement"][0]["count"], 96);
    assert_eq!(
        forest["config"]["features"][3]["feature"]["config"]["to_place"]["state"]["Name"],
        "minecraft:lily_of_the_valley"
    );
}

#[test]
fn vegetation_features_patch_and_selector_configs_match_vanilla_json() {
    let pale_moss = configured_feature_json("minecraft:pale_moss_vegetation");
    assert_eq!(
        pale_moss["config"]["to_place"]["entries"][0]["data"]["Name"],
        "minecraft:pale_moss_carpet"
    );
    assert_eq!(pale_moss["config"]["to_place"]["entries"][0]["weight"], 25);
    assert_eq!(
        pale_moss["config"]["to_place"]["entries"][2]["data"]["Name"],
        "minecraft:tall_grass"
    );
    assert_eq!(pale_moss["config"]["to_place"]["entries"][2]["weight"], 10);

    let patch = configured_feature_json("minecraft:pale_moss_patch");
    assert_eq!(
        patch["config"]["replaceable"],
        "#minecraft:moss_replaceable"
    );
    assert_eq!(
        patch["config"]["ground_state"]["state"]["Name"],
        "minecraft:pale_moss_block"
    );
    assert_eq!(patch["config"]["vegetation_chance"], 0.3);
    assert_eq!(patch["config"]["xz_radius"]["min_inclusive"], 2);
    assert_eq!(patch["config"]["xz_radius"]["max_inclusive"], 4);

    let bonemeal = configured_feature_json("minecraft:pale_moss_patch_bonemeal");
    assert_eq!(bonemeal["config"]["vegetation_chance"], 0.6);
    assert_eq!(bonemeal["config"]["xz_radius"]["min_inclusive"], 1);
    assert_eq!(bonemeal["config"]["xz_radius"]["max_inclusive"], 2);

    let dark_forest = configured_feature_json("minecraft:dark_forest_vegetation");
    assert_eq!(
        dark_forest["config"]["default"],
        "minecraft:oak_leaf_litter"
    );
    assert_eq!(
        dark_forest["config"]["features"]
            .as_array()
            .expect("dark forest weighted features")
            .len(),
        7
    );
    assert_eq!(
        dark_forest["config"]["features"][0]["feature"]["feature"],
        "minecraft:huge_brown_mushroom"
    );
    assert_eq!(dark_forest["config"]["features"][3]["chance"], 0.0025);
    assert_eq!(
        dark_forest["config"]["features"][6]["feature"],
        "minecraft:fancy_oak_leaf_litter"
    );
}

#[test]
fn vegetation_features_tree_selector_configs_match_vanilla_json() {
    let old_pine = configured_feature_json("minecraft:trees_old_growth_pine_taiga");
    assert_eq!(old_pine["config"]["default"], "minecraft:spruce_checked");
    assert_eq!(old_pine["config"]["features"][0]["chance"], 0.025641026);
    assert_eq!(
        old_pine["config"]["features"][1]["feature"],
        "minecraft:mega_pine_checked"
    );
    assert_eq!(old_pine["config"]["features"][3]["chance"], 0.0125);

    let bamboo = configured_feature_json("minecraft:bamboo_vegetation");
    assert_eq!(
        bamboo["config"]["features"][2]["feature"],
        "minecraft:mega_jungle_tree_checked"
    );
    assert_eq!(bamboo["config"]["features"][2]["chance"], 0.7);
    assert_eq!(
        bamboo["config"]["default"]["feature"],
        "minecraft:grass_jungle"
    );
    assert_eq!(bamboo["config"]["default"]["placement"][0]["count"], 32);
    assert_eq!(
        bamboo["config"]["default"]["placement"][2]["predicate"]["predicates"][1]["predicate"]
            ["blocks"],
        "minecraft:podzol"
    );

    let mushroom = configured_feature_json("minecraft:mushroom_island_vegetation");
    assert_eq!(
        mushroom["config"]["feature_true"]["feature"],
        "minecraft:huge_red_mushroom"
    );
    assert_eq!(
        mushroom["config"]["feature_false"]["feature"],
        "minecraft:huge_brown_mushroom"
    );

    let mangrove = configured_feature_json("minecraft:mangrove_vegetation");
    assert_eq!(mangrove["config"]["default"], "minecraft:mangrove_checked");
    assert_eq!(
        mangrove["config"]["features"][0]["feature"],
        "minecraft:tall_mangrove_checked"
    );
    assert_eq!(mangrove["config"]["features"][0]["chance"], 0.85);
}
