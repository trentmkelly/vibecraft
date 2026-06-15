use super::*;

const NETHER_FEATURES_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/features/NetherFeatures.java");

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

#[test]
fn nether_features_java_source_shape_matches_configured_feature_registry() {
    assert_eq!(NETHER_FEATURES_JAVA.lines().count(), 135);
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "FeatureUtils.createKey("),
        22
    );
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "FeatureUtils.register("),
        22
    );
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "new NetherForestVegetationConfig"),
        6
    );
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "new SpringConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "new SimpleBlockConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(NETHER_FEATURES_JAVA, "new TwistingVinesConfig"),
        2
    );
    for sentinel in [
        "DELTA = FeatureUtils.createKey(\"delta\")",
        "SOUL_FIRE = FeatureUtils.createKey(\"patch_soul_fire\")",
        "WARPED_FOREST_VEGETION = FeatureUtils.createKey(\"warped_forest_vegetation\")",
        "new DeltaFeatureConfiguration(Blocks.LAVA.defaultBlockState(), Blocks.MAGMA_BLOCK.defaultBlockState(), UniformInt.of(3, 7), UniformInt.of(0, 2))",
        "new ColumnFeatureConfiguration(ConstantInt.of(1), UniformInt.of(1, 4))",
        "new ReplaceSphereConfiguration(Blocks.NETHERRACK.defaultBlockState(), Blocks.BLACKSTONE.defaultBlockState(), UniformInt.of(3, 7))",
        "new NetherForestVegetationConfig(crimsonVegetationProvider, 8, 4)",
        "new NetherForestVegetationConfig(warpedVegetationProvider, 3, 1)",
        "new TwistingVinesConfig(8, 4, 8)",
        "new SpringConfiguration(Fluids.LAVA.defaultFluidState(), false, 5, 0, HolderSet.direct(Block::builtInRegistryHolder, Blocks.NETHERRACK))",
        "FeatureUtils.register(context, WEEPING_VINES, Feature.WEEPING_VINES);",
        "FeatureUtils.register(context, BASALT_PILLAR, Feature.BASALT_PILLAR);",
    ] {
        assert!(
            NETHER_FEATURES_JAVA.contains(sentinel),
            "missing NetherFeatures sentinel {sentinel}"
        );
    }
}

#[test]
fn nether_features_keys_and_types_match_vanilla_json() {
    let keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Nether)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "minecraft:delta",
            "minecraft:small_basalt_columns",
            "minecraft:large_basalt_columns",
            "minecraft:basalt_blobs",
            "minecraft:blackstone_blobs",
            "minecraft:glowstone_extra",
            "minecraft:crimson_forest_vegetation",
            "minecraft:crimson_forest_vegetation_bonemeal",
            "minecraft:warped_forest_vegetation",
            "minecraft:warped_forest_vegetation_bonemeal",
            "minecraft:nether_sprouts",
            "minecraft:nether_sprouts_bonemeal",
            "minecraft:twisting_vines",
            "minecraft:twisting_vines_bonemeal",
            "minecraft:weeping_vines",
            "minecraft:crimson_roots",
            "minecraft:basalt_pillar",
            "minecraft:spring_lava_nether",
            "minecraft:spring_nether_closed",
            "minecraft:spring_nether_open",
            "minecraft:patch_fire",
            "minecraft:patch_soul_fire",
        ]
    );

    for (id, feature_type) in [
        ("minecraft:delta", "minecraft:delta_feature"),
        ("minecraft:small_basalt_columns", "minecraft:basalt_columns"),
        ("minecraft:large_basalt_columns", "minecraft:basalt_columns"),
        (
            "minecraft:basalt_blobs",
            "minecraft:netherrack_replace_blobs",
        ),
        (
            "minecraft:blackstone_blobs",
            "minecraft:netherrack_replace_blobs",
        ),
        ("minecraft:glowstone_extra", "minecraft:glowstone_blob"),
        (
            "minecraft:crimson_forest_vegetation",
            "minecraft:nether_forest_vegetation",
        ),
        (
            "minecraft:crimson_forest_vegetation_bonemeal",
            "minecraft:nether_forest_vegetation",
        ),
        (
            "minecraft:warped_forest_vegetation",
            "minecraft:nether_forest_vegetation",
        ),
        (
            "minecraft:warped_forest_vegetation_bonemeal",
            "minecraft:nether_forest_vegetation",
        ),
        (
            "minecraft:nether_sprouts",
            "minecraft:nether_forest_vegetation",
        ),
        (
            "minecraft:nether_sprouts_bonemeal",
            "minecraft:nether_forest_vegetation",
        ),
        ("minecraft:twisting_vines", "minecraft:twisting_vines"),
        (
            "minecraft:twisting_vines_bonemeal",
            "minecraft:twisting_vines",
        ),
        ("minecraft:weeping_vines", "minecraft:weeping_vines"),
        ("minecraft:crimson_roots", "minecraft:simple_block"),
        ("minecraft:basalt_pillar", "minecraft:basalt_pillar"),
        ("minecraft:spring_lava_nether", "minecraft:spring_feature"),
        ("minecraft:spring_nether_closed", "minecraft:spring_feature"),
        ("minecraft:spring_nether_open", "minecraft:spring_feature"),
        ("minecraft:patch_fire", "minecraft:simple_block"),
        ("minecraft:patch_soul_fire", "minecraft:simple_block"),
    ] {
        assert_eq!(configured_feature_json(id)["type"], feature_type, "{id}");
    }
}

#[test]
fn nether_features_delta_and_basalt_configs_match_vanilla_json() {
    let delta = configured_feature_json("minecraft:delta");
    assert_eq!(delta["config"]["contents"]["Name"], "minecraft:lava");
    assert_eq!(delta["config"]["rim"]["Name"], "minecraft:magma_block");
    assert_eq!(delta["config"]["size"]["min_inclusive"], 3);
    assert_eq!(delta["config"]["size"]["max_inclusive"], 7);
    assert_eq!(delta["config"]["rim_size"]["max_inclusive"], 2);

    let small_columns = configured_feature_json("minecraft:small_basalt_columns");
    assert_eq!(small_columns["config"]["reach"], 1);
    assert_eq!(small_columns["config"]["height"]["max_inclusive"], 4);

    let large_columns = configured_feature_json("minecraft:large_basalt_columns");
    assert_eq!(large_columns["config"]["reach"]["min_inclusive"], 2);
    assert_eq!(large_columns["config"]["height"]["min_inclusive"], 5);
    assert_eq!(large_columns["config"]["height"]["max_inclusive"], 10);

    let basalt_blobs = configured_feature_json("minecraft:basalt_blobs");
    assert_eq!(
        basalt_blobs["config"]["target"]["Name"],
        "minecraft:netherrack"
    );
    assert_eq!(basalt_blobs["config"]["state"]["Name"], "minecraft:basalt");
    assert_eq!(basalt_blobs["config"]["radius"]["max_inclusive"], 7);
}

#[test]
fn nether_features_vegetation_configs_match_vanilla_json() {
    let crimson = configured_feature_json("minecraft:crimson_forest_vegetation");
    assert_eq!(crimson["config"]["spread_width"], 8);
    assert_eq!(crimson["config"]["spread_height"], 4);
    assert_eq!(
        crimson["config"]["state_provider"]["entries"][0]["data"]["Name"],
        "minecraft:crimson_roots"
    );
    assert_eq!(
        crimson["config"]["state_provider"]["entries"][0]["weight"],
        87
    );

    let warped = configured_feature_json("minecraft:warped_forest_vegetation");
    assert_eq!(
        warped["config"]["state_provider"]["entries"][0]["data"]["Name"],
        "minecraft:warped_roots"
    );
    assert_eq!(
        warped["config"]["state_provider"]["entries"][2]["weight"],
        13
    );

    let sprouts = configured_feature_json("minecraft:nether_sprouts");
    assert_eq!(
        sprouts["config"]["state_provider"]["state"]["Name"],
        "minecraft:nether_sprouts"
    );
}

#[test]
fn nether_features_vine_configs_match_vanilla_json() {
    let twisting = configured_feature_json("minecraft:twisting_vines");
    assert_eq!(twisting["config"]["spread_width"], 8);
    assert_eq!(twisting["config"]["spread_height"], 4);
    assert_eq!(twisting["config"]["max_height"], 8);

    let twisting_bonemeal = configured_feature_json("minecraft:twisting_vines_bonemeal");
    assert_eq!(twisting_bonemeal["config"]["spread_width"], 3);
    assert_eq!(twisting_bonemeal["config"]["spread_height"], 1);
    assert_eq!(twisting_bonemeal["config"]["max_height"], 2);
}

#[test]
fn nether_features_spring_configs_match_vanilla_json() {
    for (id, requires_block_below, rock_count, hole_count, valid_blocks) in [
        ("minecraft:spring_lava_nether", true, 4, 1, 5),
        ("minecraft:spring_nether_closed", false, 5, 0, 1),
        ("minecraft:spring_nether_open", false, 4, 1, 1),
    ] {
        let spring = configured_feature_json(id);
        assert_eq!(spring["config"]["state"]["Name"], "minecraft:lava", "{id}");
        assert_eq!(
            spring["config"]["requires_block_below"], requires_block_below,
            "{id}"
        );
        assert_eq!(spring["config"]["rock_count"], rock_count, "{id}");
        assert_eq!(spring["config"]["hole_count"], hole_count, "{id}");
        let valid_blocks_value = &spring["config"]["valid_blocks"];
        let count = valid_blocks_value.as_array().map_or(1, std::vec::Vec::len);
        assert_eq!(count, valid_blocks, "{id}");
    }
}
