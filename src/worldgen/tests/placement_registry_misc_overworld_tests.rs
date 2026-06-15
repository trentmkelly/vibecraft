use super::*;

const MISC_OVERWORLD_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/MiscOverworldPlacements.java"
);

const MISC_OVERWORLD_PLACED_FEATURE_KEYS: [&str; 18] = [
    "minecraft:ice_spike",
    "minecraft:ice_patch",
    "minecraft:forest_rock",
    "minecraft:iceberg_packed",
    "minecraft:iceberg_blue",
    "minecraft:blue_ice",
    "minecraft:lake_lava_underground",
    "minecraft:lake_lava_surface",
    "minecraft:disk_clay",
    "minecraft:disk_gravel",
    "minecraft:disk_sand",
    "minecraft:disk_grass",
    "minecraft:freeze_top_layer",
    "minecraft:void_start_platform",
    "minecraft:desert_well",
    "minecraft:spring_lava",
    "minecraft:spring_lava_frozen",
    "minecraft:spring_water",
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

fn misc_overworld_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::MiscOverworld)
        .expect("misc overworld placed-feature bootstrap source")
        .keys
}

#[test]
fn misc_overworld_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(MISC_OVERWORLD_PLACEMENTS_JAVA.lines().count(), 191);
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        18
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_PLACEMENTS_JAVA, "PlacementUtils.register("),
        18
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_PLACEMENTS_JAVA, "CountPlacement.of"),
        9
    );
    assert_eq!(
        count_occurrences(
            MISC_OVERWORLD_PLACEMENTS_JAVA,
            "RarityFilter.onAverageOnceEvery"
        ),
        5
    );
    assert_eq!(
        count_occurrences(
            MISC_OVERWORLD_PLACEMENTS_JAVA,
            "BlockPredicateFilter.forPredicate"
        ),
        5
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_PLACEMENTS_JAVA, "BiomeFilter.biome()"),
        18
    );

    for sentinel in [
        "ICE_SPIKE = PlacementUtils.createKey(\"ice_spike\")",
        "SPRING_WATER = PlacementUtils.createKey(\"spring_water\")",
        "RandomOffsetPlacement.vertical(ConstantInt.of(-1))",
        "BlockPredicateFilter.forPredicate(BlockPredicate.matchesBlocks(Blocks.SNOW_BLOCK))",
        "RarityFilter.onAverageOnceEvery(200)",
        "HeightRangePlacement.uniform(VerticalAnchor.absolute(30), VerticalAnchor.absolute(61))",
        "EnvironmentScanPlacement.scanningFor(",
        "SurfaceRelativeThresholdFilter.of(Heightmap.Types.OCEAN_FLOOR_WG, Integer.MIN_VALUE, -5)",
        "VeryBiasedToBottomHeight.of(VerticalAnchor.bottom(), VerticalAnchor.belowTop(8), 8)",
    ] {
        assert!(
            MISC_OVERWORLD_PLACEMENTS_JAVA.contains(sentinel),
            "missing MiscOverworldPlacements sentinel {sentinel}"
        );
    }

    assert_eq!(
        misc_overworld_placed_feature_keys(),
        MISC_OVERWORLD_PLACED_FEATURE_KEYS
    );
    for key in MISC_OVERWORLD_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::MiscOverworld),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn misc_overworld_placements_ice_and_iceberg_configs_match_vanilla_json() {
    for (id, feature, count) in [
        ("minecraft:ice_spike", "minecraft:ice_spike", 3),
        ("minecraft:forest_rock", "minecraft:forest_rock", 2),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["heightmap"], "MOTION_BLOCKING",
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }

    let ice_patch = placed_feature_json("minecraft:ice_patch");
    assert_eq!(ice_patch["feature"], "minecraft:ice_patch");
    assert_eq!(ice_patch["placement"][0]["count"], 2);
    assert_eq!(ice_patch["placement"][3]["y_spread"], -1);
    assert_eq!(
        ice_patch["placement"][4]["predicate"]["blocks"],
        "minecraft:snow_block"
    );

    for (id, feature, chance) in [
        ("minecraft:iceberg_packed", "minecraft:iceberg_packed", 16),
        ("minecraft:iceberg_blue", "minecraft:iceberg_blue", 200),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(
            placed["placement"][0]["type"], "minecraft:rarity_filter",
            "{id}"
        );
        assert_eq!(placed["placement"][0]["chance"], chance, "{id}");
        assert_eq!(placed["placement"][2]["type"], "minecraft:biome", "{id}");
    }

    let blue_ice = placed_feature_json("minecraft:blue_ice");
    assert_eq!(blue_ice["feature"], "minecraft:blue_ice");
    assert_eq!(blue_ice["placement"][0]["count"]["min_inclusive"], 0);
    assert_eq!(blue_ice["placement"][0]["count"]["max_inclusive"], 19);
    assert_eq!(
        blue_ice["placement"][2]["height"]["min_inclusive"]["absolute"],
        30
    );
    assert_eq!(
        blue_ice["placement"][2]["height"]["max_inclusive"]["absolute"],
        61
    );
}

#[test]
fn misc_overworld_placements_lake_and_disk_configs_match_vanilla_json() {
    let underground = placed_feature_json("minecraft:lake_lava_underground");
    assert_eq!(underground["feature"], "minecraft:lake_lava");
    assert_eq!(underground["placement"][0]["chance"], 9);
    assert_eq!(
        underground["placement"][2]["height"]["max_inclusive"]["below_top"],
        0
    );
    assert_eq!(
        underground["placement"][3]["target_condition"]["predicates"][1]["offset"],
        serde_json::json!([0, -5, 0])
    );
    assert_eq!(underground["placement"][4]["heightmap"], "OCEAN_FLOOR_WG");
    assert_eq!(underground["placement"][4]["max_inclusive"], -5);

    let surface = placed_feature_json("minecraft:lake_lava_surface");
    assert_eq!(surface["feature"], "minecraft:lake_lava");
    assert_eq!(surface["placement"][0]["chance"], 200);
    assert_eq!(surface["placement"][2]["heightmap"], "WORLD_SURFACE_WG");

    for (id, feature, count) in [
        ("minecraft:disk_clay", "minecraft:disk_clay", None),
        ("minecraft:disk_gravel", "minecraft:disk_gravel", None),
        ("minecraft:disk_sand", "minecraft:disk_sand", Some(3)),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        let filter_index = if let Some(count) = count {
            assert_eq!(placed["placement"][0]["count"], count, "{id}");
            assert_eq!(
                placed["placement"][2]["heightmap"], "OCEAN_FLOOR_WG",
                "{id}"
            );
            3
        } else {
            assert_eq!(
                placed["placement"][1]["heightmap"], "OCEAN_FLOOR_WG",
                "{id}"
            );
            2
        };
        assert_eq!(
            placed["placement"][filter_index]["predicate"]["fluids"], "minecraft:water",
            "{id}"
        );
    }

    let grass = placed_feature_json("minecraft:disk_grass");
    assert_eq!(grass["feature"], "minecraft:disk_grass");
    assert_eq!(grass["placement"][0]["count"], 1);
    assert_eq!(grass["placement"][3]["y_spread"], -1);
    assert_eq!(
        grass["placement"][4]["predicate"]["blocks"],
        "minecraft:mud"
    );
}

#[test]
fn misc_overworld_placements_misc_and_spring_configs_match_vanilla_json() {
    for id in [
        "minecraft:freeze_top_layer",
        "minecraft:void_start_platform",
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], id);
        assert_eq!(
            placed["placement"]
                .as_array()
                .expect("biome-only misc placement")
                .len(),
            1,
            "{id}"
        );
        assert_eq!(placed["placement"][0]["type"], "minecraft:biome", "{id}");
    }

    let well = placed_feature_json("minecraft:desert_well");
    assert_eq!(well["feature"], "minecraft:desert_well");
    assert_eq!(well["placement"][0]["chance"], 1000);
    assert_eq!(well["placement"][2]["heightmap"], "MOTION_BLOCKING");

    for (id, feature, count, height_type) in [
        (
            "minecraft:spring_lava",
            "minecraft:spring_lava_overworld",
            20,
            "minecraft:very_biased_to_bottom",
        ),
        (
            "minecraft:spring_lava_frozen",
            "minecraft:spring_lava_frozen",
            20,
            "minecraft:very_biased_to_bottom",
        ),
        (
            "minecraft:spring_water",
            "minecraft:spring_water",
            25,
            "minecraft:uniform",
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][2]["height"]["type"], height_type,
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }

    let lava = placed_feature_json("minecraft:spring_lava");
    assert_eq!(lava["placement"][2]["height"]["inner"], 8);
    assert_eq!(
        lava["placement"][2]["height"]["max_inclusive"]["below_top"],
        8
    );

    let water = placed_feature_json("minecraft:spring_water");
    assert_eq!(
        water["placement"][2]["height"]["max_inclusive"]["absolute"],
        192
    );
}
