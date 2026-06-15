use super::*;

const END_PLACEMENTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/placement/EndPlacements.java");

const END_PLACED_FEATURE_KEYS: [&str; 5] = [
    "minecraft:end_platform",
    "minecraft:end_spike",
    "minecraft:end_gateway_return",
    "minecraft:chorus_plant",
    "minecraft:end_island_decorated",
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

fn end_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::End)
        .expect("end placed-feature bootstrap source")
        .keys
}

#[test]
fn end_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(END_PLACEMENTS_JAVA.lines().count(), 62);
    assert_eq!(
        count_occurrences(END_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        5
    );
    assert_eq!(
        count_occurrences(END_PLACEMENTS_JAVA, "PlacementUtils.register("),
        5
    );
    assert_eq!(
        count_occurrences(END_PLACEMENTS_JAVA, "RarityFilter.onAverageOnceEvery"),
        2
    );
    assert_eq!(
        count_occurrences(END_PLACEMENTS_JAVA, "BiomeFilter.biome()"),
        5
    );
    assert_eq!(
        count_occurrences(END_PLACEMENTS_JAVA, "PlacementUtils.HEIGHTMAP"),
        2
    );

    for sentinel in [
        "END_PLATFORM = PlacementUtils.createKey(\"end_platform\")",
        "END_ISLAND_DECORATED = PlacementUtils.createKey(\"end_island_decorated\")",
        "FixedPlacement.of(ServerLevel.END_SPAWN_POINT.below())",
        "RarityFilter.onAverageOnceEvery(700)",
        "RandomOffsetPlacement.vertical(UniformInt.of(3, 9))",
        "CountPlacement.of(UniformInt.of(0, 4))",
        "PlacementUtils.countExtra(1, 0.25F, 1)",
        "HeightRangePlacement.uniform(VerticalAnchor.absolute(55), VerticalAnchor.absolute(70))",
    ] {
        assert!(
            END_PLACEMENTS_JAVA.contains(sentinel),
            "missing EndPlacements sentinel {sentinel}"
        );
    }

    assert_eq!(end_placed_feature_keys(), END_PLACED_FEATURE_KEYS);
    for key in END_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::End),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn end_placements_platform_spike_and_gateway_configs_match_vanilla_json() {
    let platform = placed_feature_json("minecraft:end_platform");
    assert_eq!(platform["feature"], "minecraft:end_platform");
    assert_eq!(
        platform["placement"][0]["type"],
        "minecraft:fixed_placement"
    );
    assert_eq!(
        platform["placement"][0]["positions"][0],
        serde_json::json!([100, 49, 0])
    );
    assert_eq!(platform["placement"][1]["type"], "minecraft:biome");

    let spike = placed_feature_json("minecraft:end_spike");
    assert_eq!(spike["feature"], "minecraft:end_spike");
    assert_eq!(
        spike["placement"]
            .as_array()
            .expect("end spike placement modifiers")
            .len(),
        1
    );
    assert_eq!(spike["placement"][0]["type"], "minecraft:biome");

    let gateway = placed_feature_json("minecraft:end_gateway_return");
    assert_eq!(gateway["feature"], "minecraft:end_gateway_return");
    assert_eq!(gateway["placement"][0]["type"], "minecraft:rarity_filter");
    assert_eq!(gateway["placement"][0]["chance"], 700);
    assert_eq!(gateway["placement"][1]["type"], "minecraft:in_square");
    assert_eq!(gateway["placement"][2]["heightmap"], "MOTION_BLOCKING");
    assert_eq!(
        gateway["placement"][3]["y_spread"]["type"],
        "minecraft:uniform"
    );
    assert_eq!(gateway["placement"][3]["y_spread"]["min_inclusive"], 3);
    assert_eq!(gateway["placement"][3]["y_spread"]["max_inclusive"], 9);
    assert_eq!(gateway["placement"][4]["type"], "minecraft:biome");
}

#[test]
fn end_placements_chorus_and_island_configs_match_vanilla_json() {
    let chorus = placed_feature_json("minecraft:chorus_plant");
    assert_eq!(chorus["feature"], "minecraft:chorus_plant");
    assert_eq!(chorus["placement"][0]["type"], "minecraft:count");
    assert_eq!(chorus["placement"][0]["count"]["min_inclusive"], 0);
    assert_eq!(chorus["placement"][0]["count"]["max_inclusive"], 4);
    assert_eq!(chorus["placement"][1]["type"], "minecraft:in_square");
    assert_eq!(chorus["placement"][2]["heightmap"], "MOTION_BLOCKING");
    assert_eq!(chorus["placement"][3]["type"], "minecraft:biome");

    let island = placed_feature_json("minecraft:end_island_decorated");
    assert_eq!(island["feature"], "minecraft:end_island");
    assert_eq!(island["placement"][0]["type"], "minecraft:rarity_filter");
    assert_eq!(island["placement"][0]["chance"], 14);
    assert_eq!(
        island["placement"][1]["count"]["type"],
        "minecraft:weighted_list"
    );
    assert_eq!(
        island["placement"][1]["count"]["distribution"][0]["data"],
        1
    );
    assert_eq!(
        island["placement"][1]["count"]["distribution"][0]["weight"],
        3
    );
    assert_eq!(
        island["placement"][1]["count"]["distribution"][1]["data"],
        2
    );
    assert_eq!(
        island["placement"][1]["count"]["distribution"][1]["weight"],
        1
    );
    assert_eq!(island["placement"][2]["type"], "minecraft:in_square");
    assert_eq!(
        island["placement"][3]["height"]["min_inclusive"]["absolute"],
        55
    );
    assert_eq!(
        island["placement"][3]["height"]["max_inclusive"]["absolute"],
        70
    );
    assert_eq!(island["placement"][4]["type"], "minecraft:biome");
}
