use super::*;

const PILE_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/PileFeatures.java"
);

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

#[test]
fn pile_features_java_bootstrap_matches_configured_feature_registry() {
    assert_eq!(PILE_FEATURES_JAVA.lines().count(), 47);
    assert_eq!(
        count_occurrences(PILE_FEATURES_JAVA, "FeatureUtils.createKey("),
        5
    );
    assert_eq!(
        count_occurrences(PILE_FEATURES_JAVA, "FeatureUtils.register("),
        5
    );
    assert_eq!(
        count_occurrences(PILE_FEATURES_JAVA, "new BlockPileConfiguration"),
        5
    );
    assert_eq!(
        count_occurrences(PILE_FEATURES_JAVA, "new WeightedStateProvider"),
        2
    );
    for sentinel in [
        "PILE_HAY = FeatureUtils.createKey(\"pile_hay\")",
        "PILE_PUMPKIN = FeatureUtils.createKey(\"pile_pumpkin\")",
        "new RotatedBlockProvider(Blocks.HAY_BLOCK)",
        "BlockStateProvider.simple(Blocks.MELON)",
        "BlockStateProvider.simple(Blocks.SNOW)",
        "WeightedList.<BlockState>builder().add(Blocks.BLUE_ICE.defaultBlockState(), 1).add(Blocks.PACKED_ICE.defaultBlockState(), 5)",
        "WeightedList.<BlockState>builder().add(Blocks.PUMPKIN.defaultBlockState(), 19).add(Blocks.JACK_O_LANTERN.defaultBlockState(), 1)",
    ] {
        assert!(
            PILE_FEATURES_JAVA.contains(sentinel),
            "missing PileFeatures sentinel {sentinel}"
        );
    }

    let keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Pile)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "minecraft:pile_hay",
            "minecraft:pile_melon",
            "minecraft:pile_snow",
            "minecraft:pile_ice",
            "minecraft:pile_pumpkin",
        ]
    );

    for key in keys {
        assert_eq!(configured_feature_json(key)["type"], "minecraft:block_pile");
    }
}

#[test]
fn pile_features_state_providers_match_vanilla_json() {
    let hay = configured_feature_json("minecraft:pile_hay");
    assert_eq!(
        hay["config"]["state_provider"]["type"],
        "minecraft:rotated_block_provider"
    );
    assert_eq!(
        hay["config"]["state_provider"]["state"]["Name"],
        "minecraft:hay_block"
    );

    let melon = configured_feature_json("minecraft:pile_melon");
    assert_eq!(
        melon["config"]["state_provider"]["state"]["Name"],
        "minecraft:melon"
    );

    let snow = configured_feature_json("minecraft:pile_snow");
    assert_eq!(
        snow["config"]["state_provider"]["state"]["Properties"]["layers"],
        "1"
    );

    let ice = configured_feature_json("minecraft:pile_ice");
    assert_eq!(
        ice["config"]["state_provider"]["entries"][0]["data"]["Name"],
        "minecraft:blue_ice"
    );
    assert_eq!(ice["config"]["state_provider"]["entries"][0]["weight"], 1);
    assert_eq!(
        ice["config"]["state_provider"]["entries"][1]["data"]["Name"],
        "minecraft:packed_ice"
    );
    assert_eq!(ice["config"]["state_provider"]["entries"][1]["weight"], 5);

    let pumpkin = configured_feature_json("minecraft:pile_pumpkin");
    assert_eq!(
        pumpkin["config"]["state_provider"]["entries"][0]["data"]["Name"],
        "minecraft:pumpkin"
    );
    assert_eq!(
        pumpkin["config"]["state_provider"]["entries"][1]["data"]["Name"],
        "minecraft:jack_o_lantern"
    );
    assert_eq!(
        pumpkin["config"]["state_provider"]["entries"][1]["weight"],
        1
    );
}
