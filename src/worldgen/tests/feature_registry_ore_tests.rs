use super::*;

const ORE_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/OreFeatures.java"
);

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
fn ore_features_java_source_shape_matches_configured_feature_registry() {
    assert_eq!(ORE_FEATURES_JAVA.lines().count(), 155);
    assert_eq!(
        count_occurrences(ORE_FEATURES_JAVA, "FeatureUtils.createKey("),
        32
    );
    assert_eq!(
        count_occurrences(ORE_FEATURES_JAVA, "FeatureUtils.register("),
        32
    );
    assert_eq!(
        count_occurrences(ORE_FEATURES_JAVA, "new OreConfiguration"),
        32
    );
    assert_eq!(
        count_occurrences(ORE_FEATURES_JAVA, "OreConfiguration.target"),
        18
    );
    assert_eq!(
        count_occurrences(ORE_FEATURES_JAVA, "Feature.SCATTERED_ORE"),
        2
    );
    for sentinel in [
        "ORE_MAGMA = FeatureUtils.createKey(\"ore_magma\")",
        "ORE_COPPPER_SMALL = FeatureUtils.createKey(\"ore_copper_small\")",
        "RuleTest naturalStone = new TagMatchTest(BlockTags.BASE_STONE_OVERWORLD);",
        "RuleTest netherrack = new BlockMatchTest(Blocks.NETHERRACK);",
        "RuleTest netherOreReplaceables = new TagMatchTest(BlockTags.BASE_STONE_NETHER);",
        "new OreConfiguration(netherrack, Blocks.MAGMA_BLOCK.defaultBlockState(), 33)",
        "new OreConfiguration(oreCoalTargetList, 17, 0.5F)",
        "new OreConfiguration(oreDiamondTargetList, 12, 0.7F)",
        "new OreConfiguration(netherOreReplaceables, Blocks.ANCIENT_DEBRIS.defaultBlockState(), 3, 1.0F)",
        "FeatureUtils.register(context, ORE_CLAY, Feature.ORE, new OreConfiguration(naturalStone, Blocks.CLAY.defaultBlockState(), 33));",
    ] {
        assert!(
            ORE_FEATURES_JAVA.contains(sentinel),
            "missing OreFeatures sentinel {sentinel}"
        );
    }
}

#[test]
fn ore_features_keys_and_types_match_vanilla_json() {
    let keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Ore)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "minecraft:ore_magma",
            "minecraft:ore_soul_sand",
            "minecraft:ore_nether_gold",
            "minecraft:ore_quartz",
            "minecraft:ore_gravel_nether",
            "minecraft:ore_blackstone",
            "minecraft:ore_dirt",
            "minecraft:ore_gravel",
            "minecraft:ore_granite",
            "minecraft:ore_diorite",
            "minecraft:ore_andesite",
            "minecraft:ore_tuff",
            "minecraft:ore_coal",
            "minecraft:ore_coal_buried",
            "minecraft:ore_iron",
            "minecraft:ore_iron_small",
            "minecraft:ore_gold",
            "minecraft:ore_gold_buried",
            "minecraft:ore_redstone",
            "minecraft:ore_diamond_small",
            "minecraft:ore_diamond_medium",
            "minecraft:ore_diamond_large",
            "minecraft:ore_diamond_buried",
            "minecraft:ore_lapis",
            "minecraft:ore_lapis_buried",
            "minecraft:ore_infested",
            "minecraft:ore_emerald",
            "minecraft:ore_ancient_debris_large",
            "minecraft:ore_ancient_debris_small",
            "minecraft:ore_copper_small",
            "minecraft:ore_copper_large",
            "minecraft:ore_clay",
        ]
    );

    for feature in keys {
        let expected_type = if feature.starts_with("minecraft:ore_ancient_debris") {
            "minecraft:scattered_ore"
        } else {
            "minecraft:ore"
        };
        assert_eq!(
            configured_feature_json(feature)["type"],
            expected_type,
            "{feature}"
        );
    }
}

#[test]
fn ore_features_representative_configs_match_vanilla_json() {
    let magma = configured_feature_json("minecraft:ore_magma");
    assert_eq!(magma["config"]["size"], 33);
    assert_eq!(
        magma["config"]["targets"][0]["target"]["block"],
        "minecraft:netherrack"
    );
    assert_eq!(
        magma["config"]["targets"][0]["state"]["Name"],
        "minecraft:magma_block"
    );

    let coal_buried = configured_feature_json("minecraft:ore_coal_buried");
    assert_eq!(coal_buried["config"]["size"], 17);
    assert_eq!(coal_buried["config"]["discard_chance_on_air_exposure"], 0.5);
    assert_eq!(
        coal_buried["config"]["targets"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        coal_buried["config"]["targets"][1]["state"]["Name"],
        "minecraft:deepslate_coal_ore"
    );

    let redstone = configured_feature_json("minecraft:ore_redstone");
    assert_eq!(redstone["config"]["size"], 8);
    assert_eq!(
        redstone["config"]["targets"][0]["target"]["tag"],
        "minecraft:stone_ore_replaceables"
    );
    assert_eq!(
        redstone["config"]["targets"][1]["target"]["tag"],
        "minecraft:deepslate_ore_replaceables"
    );

    let diamond_large = configured_feature_json("minecraft:ore_diamond_large");
    assert_eq!(diamond_large["config"]["size"], 12);
    assert_eq!(
        diamond_large["config"]["discard_chance_on_air_exposure"],
        0.7
    );

    let ancient_large = configured_feature_json("minecraft:ore_ancient_debris_large");
    assert_eq!(ancient_large["type"], "minecraft:scattered_ore");
    assert_eq!(ancient_large["config"]["size"], 3);
    assert_eq!(
        ancient_large["config"]["targets"][0]["target"]["tag"],
        "minecraft:base_stone_nether"
    );
    assert_eq!(
        ancient_large["config"]["discard_chance_on_air_exposure"],
        1.0
    );

    let copper_large = configured_feature_json("minecraft:ore_copper_large");
    assert_eq!(copper_large["config"]["size"], 20);
    assert_eq!(
        copper_large["config"]["targets"][1]["state"]["Name"],
        "minecraft:deepslate_copper_ore"
    );
}
