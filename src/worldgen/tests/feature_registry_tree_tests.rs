use super::*;

const TREE_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/TreeFeatures.java"
);

const TREE_FEATURE_KEYS: [&str; 50] = [
    "minecraft:crimson_fungus",
    "minecraft:crimson_fungus_planted",
    "minecraft:warped_fungus",
    "minecraft:warped_fungus_planted",
    "minecraft:huge_brown_mushroom",
    "minecraft:huge_red_mushroom",
    "minecraft:oak",
    "minecraft:dark_oak",
    "minecraft:pale_oak",
    "minecraft:pale_oak_bonemeal",
    "minecraft:pale_oak_creaking",
    "minecraft:birch",
    "minecraft:acacia",
    "minecraft:spruce",
    "minecraft:pine",
    "minecraft:jungle_tree",
    "minecraft:fancy_oak",
    "minecraft:jungle_tree_no_vine",
    "minecraft:mega_jungle_tree",
    "minecraft:mega_spruce",
    "minecraft:mega_pine",
    "minecraft:super_birch_bees_0002",
    "minecraft:super_birch_bees",
    "minecraft:swamp_oak",
    "minecraft:jungle_bush",
    "minecraft:azalea_tree",
    "minecraft:mangrove",
    "minecraft:tall_mangrove",
    "minecraft:cherry",
    "minecraft:oak_bees_0002_leaf_litter",
    "minecraft:oak_bees_002",
    "minecraft:oak_bees_005",
    "minecraft:birch_bees_0002",
    "minecraft:birch_bees_0002_leaf_litter",
    "minecraft:birch_bees_002",
    "minecraft:birch_bees_005",
    "minecraft:fancy_oak_bees_0002_leaf_litter",
    "minecraft:fancy_oak_bees_002",
    "minecraft:fancy_oak_bees_005",
    "minecraft:fancy_oak_bees",
    "minecraft:cherry_bees_005",
    "minecraft:oak_leaf_litter",
    "minecraft:dark_oak_leaf_litter",
    "minecraft:birch_leaf_litter",
    "minecraft:fancy_oak_leaf_litter",
    "minecraft:fallen_oak_tree",
    "minecraft:fallen_jungle_tree",
    "minecraft:fallen_spruce_tree",
    "minecraft:fallen_birch_tree",
    "minecraft:fallen_super_birch_tree",
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn configured_feature_json(id: &str) -> serde_json::Value {
    let path = super::vanilla_data_path(&["data", "minecraft", "worldgen", "configured_feature"])
        .join(format!("{}.json", id.trim_start_matches("minecraft:")));
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"));
    serde_json::from_str(&json).unwrap_or_else(|err| panic!("failed to parse {path:?}: {err}"))
}

fn tree_feature_keys() -> Vec<&'static str> {
    CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Tree)
        .map(|feature| feature.id)
        .collect()
}

#[test]
fn tree_features_java_bootstrap_matches_configured_feature_registry() {
    assert_eq!(TREE_FEATURES_JAVA.lines().count(), 692);
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "FeatureUtils.createKey("),
        50
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "FeatureUtils.register("),
        50
    );
    assert_eq!(
        count_occurrences(
            TREE_FEATURES_JAVA,
            "TreeConfiguration.TreeConfigurationBuilder"
        ),
        25
    );
    assert_eq!(
        count_occurrences(
            TREE_FEATURES_JAVA,
            "FallenTreeConfiguration.FallenTreeConfigurationBuilder"
        ),
        6
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "HugeFungusConfiguration"),
        5
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "HugeMushroomFeatureConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "BeehiveDecorator"),
        11
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "PlaceOnGroundDecorator"),
        5
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "PaleMossDecorator"),
        3
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "CreakingHeartDecorator"),
        2
    );
    assert_eq!(
        count_occurrences(TREE_FEATURES_JAVA, "MangroveRootPlacer"),
        3
    );

    for sentinel in [
        "CRIMSON_FUNGUS = FeatureUtils.createKey(\"crimson_fungus\")",
        "PALE_OAK_CREAKING = FeatureUtils.createKey(\"pale_oak_creaking\")",
        "OAK_BEES_0002_LEAF_LITTER = FeatureUtils.createKey(\"oak_bees_0002_leaf_litter\")",
        "FALLEN_SUPER_BIRCH_TREE = FeatureUtils.createKey(\"fallen_super_birch_tree\")",
        "new CreakingHeartDecorator(1.0F)",
        "VegetationFeatures.leafLitterPatchBuilder(1, 4)",
        "new MangroveRootPlacement(",
        "new CherryFoliagePlacer(ConstantInt.of(4), ConstantInt.of(0), ConstantInt.of(5), 0.25F, 0.5F, 0.16666667F, 0.33333334F)",
        "FeatureUtils.register(context, FALLEN_SPRUCE_TREE, Feature.FALLEN_TREE, createFallenSpruce().build())",
    ] {
        assert!(
            TREE_FEATURES_JAVA.contains(sentinel),
            "missing TreeFeatures sentinel {sentinel}"
        );
    }

    assert_eq!(tree_feature_keys(), TREE_FEATURE_KEYS);
}

#[test]
fn tree_features_keys_and_types_match_vanilla_json() {
    let huge_fungi = [
        "minecraft:crimson_fungus",
        "minecraft:crimson_fungus_planted",
        "minecraft:warped_fungus",
        "minecraft:warped_fungus_planted",
    ];
    let fallen_trees = [
        "minecraft:fallen_oak_tree",
        "minecraft:fallen_jungle_tree",
        "minecraft:fallen_spruce_tree",
        "minecraft:fallen_birch_tree",
        "minecraft:fallen_super_birch_tree",
    ];

    for key in tree_feature_keys() {
        let expected_type = if huge_fungi.contains(&key) {
            "minecraft:huge_fungus"
        } else if key == "minecraft:huge_brown_mushroom" {
            "minecraft:huge_brown_mushroom"
        } else if key == "minecraft:huge_red_mushroom" {
            "minecraft:huge_red_mushroom"
        } else if fallen_trees.contains(&key) {
            "minecraft:fallen_tree"
        } else {
            "minecraft:tree"
        };
        assert_eq!(
            configured_feature_json(key)["type"],
            expected_type,
            "wrong configured-feature type for {key}"
        );
    }
}

#[test]
fn tree_features_fungus_and_mushroom_configs_match_vanilla_json() {
    let crimson = configured_feature_json("minecraft:crimson_fungus");
    assert_eq!(
        crimson["config"]["valid_base_block"]["Name"],
        "minecraft:crimson_nylium"
    );
    assert_eq!(
        crimson["config"]["stem_state"]["Name"],
        "minecraft:crimson_stem"
    );
    assert_eq!(
        crimson["config"]["hat_state"]["Name"],
        "minecraft:nether_wart_block"
    );
    assert_eq!(
        crimson["config"]["decor_state"]["Name"],
        "minecraft:shroomlight"
    );
    assert_eq!(crimson["config"]["planted"], false);
    assert_eq!(
        crimson["config"]["replaceable_blocks"]["blocks"]
            .as_array()
            .expect("crimson fungus replaceable block list")
            .len(),
        59
    );
    assert_eq!(
        crimson["config"]["replaceable_blocks"]["blocks"][7],
        "minecraft:pale_oak_sapling"
    );
    assert_eq!(
        crimson["config"]["replaceable_blocks"]["blocks"][55],
        "minecraft:wildflowers"
    );
    assert_eq!(
        crimson["config"]["replaceable_blocks"]["blocks"][58],
        "minecraft:small_dripleaf"
    );

    let planted = configured_feature_json("minecraft:warped_fungus_planted");
    assert_eq!(
        planted["config"]["valid_base_block"]["Name"],
        "minecraft:warped_nylium"
    );
    assert_eq!(
        planted["config"]["stem_state"]["Name"],
        "minecraft:warped_stem"
    );
    assert_eq!(
        planted["config"]["hat_state"]["Name"],
        "minecraft:warped_wart_block"
    );
    assert_eq!(planted["config"]["planted"], true);

    let brown = configured_feature_json("minecraft:huge_brown_mushroom");
    assert_eq!(
        brown["config"]["cap_provider"]["state"]["Name"],
        "minecraft:brown_mushroom_block"
    );
    assert_eq!(brown["config"]["foliage_radius"], 3);
    assert_eq!(
        brown["config"]["can_place_on"]["tag"],
        "minecraft:huge_brown_mushroom_can_place_on"
    );

    let red = configured_feature_json("minecraft:huge_red_mushroom");
    assert_eq!(
        red["config"]["cap_provider"]["state"]["Name"],
        "minecraft:red_mushroom_block"
    );
    assert_eq!(red["config"]["foliage_radius"], 2);
    assert_eq!(
        red["config"]["stem_provider"]["state"]["Properties"]["up"],
        "false"
    );
}

#[test]
fn tree_features_oak_pale_oak_and_cherry_configs_match_vanilla_json() {
    let oak = configured_feature_json("minecraft:oak");
    assert_eq!(
        oak["config"]["trunk_provider"]["state"]["Name"],
        "minecraft:oak_log"
    );
    assert_eq!(
        oak["config"]["foliage_provider"]["state"]["Name"],
        "minecraft:oak_leaves"
    );
    assert_eq!(
        oak["config"]["trunk_placer"]["type"],
        "minecraft:straight_trunk_placer"
    );
    assert_eq!(oak["config"]["trunk_placer"]["base_height"], 4);
    assert_eq!(
        oak["config"]["foliage_placer"]["type"],
        "minecraft:blob_foliage_placer"
    );
    assert_eq!(oak["config"]["foliage_placer"]["radius"], 2);
    assert_eq!(oak["config"]["ignore_vines"], true);

    let pale_oak = configured_feature_json("minecraft:pale_oak");
    assert_eq!(
        pale_oak["config"]["trunk_provider"]["state"]["Name"],
        "minecraft:pale_oak_log"
    );
    assert_eq!(
        pale_oak["config"]["decorators"][0]["type"],
        "minecraft:pale_moss"
    );
    assert_eq!(
        pale_oak["config"]["decorators"][0]["ground_probability"],
        0.8
    );
    assert_eq!(
        pale_oak["config"]["minimum_size"]["type"],
        "minecraft:three_layers_feature_size"
    );

    let creaking = configured_feature_json("minecraft:pale_oak_creaking");
    assert_eq!(
        creaking["config"]["decorators"][1]["type"],
        "minecraft:creaking_heart"
    );
    assert_eq!(creaking["config"]["decorators"][1]["probability"], 1.0);

    let cherry = configured_feature_json("minecraft:cherry");
    assert_eq!(
        cherry["config"]["trunk_placer"]["type"],
        "minecraft:cherry_trunk_placer"
    );
    assert_eq!(
        cherry["config"]["trunk_placer"]["branch_count"]["distribution"]
            .as_array()
            .expect("cherry branch-count distribution")
            .len(),
        3
    );
    assert_eq!(
        cherry["config"]["foliage_placer"]["type"],
        "minecraft:cherry_foliage_placer"
    );
    assert_eq!(cherry["config"]["foliage_placer"]["height"], 5);
    assert_eq!(
        cherry["config"]["foliage_placer"]["hanging_leaves_extension_chance"],
        0.33333334
    );
}

#[test]
fn tree_features_mangrove_configs_match_vanilla_json() {
    let mangrove = configured_feature_json("minecraft:mangrove");
    assert_eq!(
        mangrove["config"]["trunk_placer"]["type"],
        "minecraft:upwards_branching_trunk_placer"
    );
    assert_eq!(mangrove["config"]["trunk_placer"]["base_height"], 2);
    assert_eq!(
        mangrove["config"]["trunk_placer"]["extra_branch_steps"]["max_inclusive"],
        4
    );
    assert_eq!(
        mangrove["config"]["root_placer"]["type"],
        "minecraft:mangrove_root_placer"
    );
    assert_eq!(
        mangrove["config"]["root_placer"]["trunk_offset_y"]["min_inclusive"],
        1
    );
    assert_eq!(
        mangrove["config"]["root_placer"]["mangrove_root_placement"]["can_grow_through"],
        "#minecraft:mangrove_roots_can_grow_through"
    );
    assert_eq!(
        mangrove["config"]["decorators"][1]["type"],
        "minecraft:attached_to_leaves"
    );
    assert_eq!(
        mangrove["config"]["decorators"][1]["block_provider"]["values"]["max_inclusive"],
        4
    );
    assert_eq!(mangrove["config"]["decorators"][2]["probability"], 0.01);

    let tall = configured_feature_json("minecraft:tall_mangrove");
    assert_eq!(tall["config"]["trunk_placer"]["base_height"], 4);
    assert_eq!(tall["config"]["trunk_placer"]["height_rand_b"], 9);
    assert_eq!(
        tall["config"]["root_placer"]["trunk_offset_y"]["max_inclusive"],
        7
    );
    assert_eq!(tall["config"]["minimum_size"]["limit"], 3);
}

#[test]
fn tree_features_leaf_litter_and_fallen_tree_configs_match_vanilla_json() {
    let oak_litter = configured_feature_json("minecraft:oak_bees_0002_leaf_litter");
    assert_eq!(
        oak_litter["config"]["decorators"][0]["type"],
        "minecraft:beehive"
    );
    assert_eq!(oak_litter["config"]["decorators"][0]["probability"], 0.002);
    assert_eq!(
        oak_litter["config"]["decorators"][1]["type"],
        "minecraft:place_on_ground"
    );
    assert_eq!(oak_litter["config"]["decorators"][1]["tries"], 96);
    assert_eq!(oak_litter["config"]["decorators"][1]["radius"], 4);
    assert_eq!(
        oak_litter["config"]["decorators"][1]["block_state_provider"]["entries"]
            .as_array()
            .expect("sparse leaf-litter entries")
            .len(),
        12
    );
    assert_eq!(oak_litter["config"]["decorators"][2]["tries"], 150);
    assert_eq!(oak_litter["config"]["decorators"][2]["radius"], 2);
    assert_eq!(
        oak_litter["config"]["decorators"][2]["block_state_provider"]["entries"]
            .as_array()
            .expect("thick leaf-litter entries")
            .len(),
        16
    );

    let oak = configured_feature_json("minecraft:fallen_oak_tree");
    assert_eq!(
        oak["config"]["trunk_provider"]["state"]["Name"],
        "minecraft:oak_log"
    );
    assert_eq!(oak["config"]["log_length"]["min_inclusive"], 4);
    assert_eq!(oak["config"]["log_length"]["max_inclusive"], 7);
    assert_eq!(
        oak["config"]["log_decorators"][0]["type"],
        "minecraft:attached_to_logs"
    );
    assert_eq!(oak["config"]["log_decorators"][0]["probability"], 0.1);
    assert_eq!(
        oak["config"]["log_decorators"][0]["block_provider"]["entries"][0]["data"]["Name"],
        "minecraft:red_mushroom"
    );
    assert_eq!(
        oak["config"]["stump_decorators"][0]["type"],
        "minecraft:trunk_vine"
    );

    let spruce = configured_feature_json("minecraft:fallen_spruce_tree");
    assert_eq!(
        spruce["config"]["trunk_provider"]["state"]["Name"],
        "minecraft:spruce_log"
    );
    assert_eq!(spruce["config"]["log_length"]["min_inclusive"], 6);
    assert_eq!(spruce["config"]["log_length"]["max_inclusive"], 10);
    assert!(spruce["config"]["stump_decorators"]
        .as_array()
        .expect("fallen spruce stump decorators")
        .is_empty());

    let super_birch = configured_feature_json("minecraft:fallen_super_birch_tree");
    assert_eq!(
        super_birch["config"]["trunk_provider"]["state"]["Name"],
        "minecraft:birch_log"
    );
    assert_eq!(super_birch["config"]["log_length"]["min_inclusive"], 5);
    assert_eq!(super_birch["config"]["log_length"]["max_inclusive"], 15);
}
