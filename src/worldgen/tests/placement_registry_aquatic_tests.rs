use super::*;

const AQUATIC_PLACEMENTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/placement/AquaticPlacements.java");

const AQUATIC_PLACED_FEATURE_KEYS: [&str; 12] = [
    "minecraft:seagrass_warm",
    "minecraft:seagrass_normal",
    "minecraft:seagrass_cold",
    "minecraft:seagrass_river",
    "minecraft:seagrass_swamp",
    "minecraft:seagrass_deep_warm",
    "minecraft:seagrass_deep",
    "minecraft:seagrass_deep_cold",
    "minecraft:sea_pickle",
    "minecraft:kelp_cold",
    "minecraft:kelp_warm",
    "minecraft:warm_ocean_vegetation",
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

fn aquatic_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Aquatic)
        .expect("aquatic placed-feature bootstrap source")
        .keys
}

#[test]
fn aquatic_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(AQUATIC_PLACEMENTS_JAVA.lines().count(), 91);
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        12
    );
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "PlacementUtils.register("),
        12
    );
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "seagrassPlacement("),
        9
    );
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "NoiseBasedCountPlacement.of"),
        3
    );
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "RarityFilter.onAverageOnceEvery"),
        1
    );
    assert_eq!(
        count_occurrences(AQUATIC_PLACEMENTS_JAVA, "CountPlacement.of"),
        4
    );

    for sentinel in [
        "SEAGRASS_WARM = PlacementUtils.createKey(\"seagrass_warm\")",
        "WARM_OCEAN_VEGETATION = PlacementUtils.createKey(\"warm_ocean_vegetation\")",
        "return List.of(InSquarePlacement.spread(), PlacementUtils.HEIGHTMAP_TOP_SOLID, CountPlacement.of(count), BiomeFilter.biome());",
        "PlacementUtils.register(context, SEAGRASS_DEEP_COLD, seagrassTall, seagrassPlacement(40));",
        "RarityFilter.onAverageOnceEvery(16)",
        "NoiseBasedCountPlacement.of(120, 80.0, 0.0)",
        "NoiseBasedCountPlacement.of(20, 400.0, 0.0)",
    ] {
        assert!(
            AQUATIC_PLACEMENTS_JAVA.contains(sentinel),
            "missing AquaticPlacements sentinel {sentinel}"
        );
    }

    assert_eq!(aquatic_placed_feature_keys(), AQUATIC_PLACED_FEATURE_KEYS);
    for key in AQUATIC_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Aquatic),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn aquatic_placements_seagrass_configs_match_vanilla_json() {
    for (id, feature, count) in [
        ("minecraft:seagrass_warm", "minecraft:seagrass_short", 80),
        ("minecraft:seagrass_normal", "minecraft:seagrass_short", 48),
        ("minecraft:seagrass_cold", "minecraft:seagrass_short", 32),
        (
            "minecraft:seagrass_river",
            "minecraft:seagrass_slightly_less_short",
            48,
        ),
        ("minecraft:seagrass_swamp", "minecraft:seagrass_mid", 64),
        (
            "minecraft:seagrass_deep_warm",
            "minecraft:seagrass_tall",
            80,
        ),
        ("minecraft:seagrass_deep", "minecraft:seagrass_tall", 48),
        (
            "minecraft:seagrass_deep_cold",
            "minecraft:seagrass_tall",
            40,
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(
            placed["placement"]
                .as_array()
                .expect("seagrass placement modifiers")
                .len(),
            4,
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][1]["heightmap"], "OCEAN_FLOOR_WG",
            "{id}"
        );
        assert_eq!(placed["placement"][2]["type"], "minecraft:count", "{id}");
        assert_eq!(placed["placement"][2]["count"], count, "{id}");
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }
}

#[test]
fn aquatic_placements_pickle_kelp_and_warm_ocean_configs_match_vanilla_json() {
    let sea_pickle = placed_feature_json("minecraft:sea_pickle");
    assert_eq!(sea_pickle["feature"], "minecraft:sea_pickle");
    assert_eq!(
        sea_pickle["placement"][0]["type"],
        "minecraft:rarity_filter"
    );
    assert_eq!(sea_pickle["placement"][0]["chance"], 16);
    assert_eq!(sea_pickle["placement"][1]["type"], "minecraft:in_square");
    assert_eq!(sea_pickle["placement"][2]["heightmap"], "OCEAN_FLOOR_WG");
    assert_eq!(sea_pickle["placement"][3]["type"], "minecraft:biome");

    for (id, ratio, factor) in [
        ("minecraft:kelp_cold", 120, 80.0),
        ("minecraft:kelp_warm", 80, 80.0),
        ("minecraft:warm_ocean_vegetation", 20, 400.0),
    ] {
        let placed = placed_feature_json(id);
        let expected_feature = if id == "minecraft:warm_ocean_vegetation" {
            "minecraft:warm_ocean_vegetation"
        } else {
            "minecraft:kelp"
        };
        assert_eq!(placed["feature"], expected_feature, "{id}");
        assert_eq!(
            placed["placement"][0]["type"], "minecraft:noise_based_count",
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["noise_to_count_ratio"], ratio,
            "{id}"
        );
        assert_eq!(placed["placement"][0]["noise_factor"], factor, "{id}");
        assert_eq!(placed["placement"][0]["noise_offset"], 0.0, "{id}");
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["heightmap"], "OCEAN_FLOOR_WG",
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }
}
