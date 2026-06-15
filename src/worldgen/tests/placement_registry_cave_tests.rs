use super::*;

const CAVE_PLACEMENTS_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/worldgen/placement/CavePlacements.java");

const CAVE_PLACED_FEATURE_KEYS: [&str; 20] = [
    "minecraft:monster_room",
    "minecraft:monster_room_deep",
    "minecraft:fossil_upper",
    "minecraft:fossil_lower",
    "minecraft:dripstone_cluster",
    "minecraft:large_dripstone",
    "minecraft:pointed_dripstone",
    "minecraft:underwater_magma",
    "minecraft:glow_lichen",
    "minecraft:rooted_azalea_tree",
    "minecraft:cave_vines",
    "minecraft:lush_caves_vegetation",
    "minecraft:lush_caves_clay",
    "minecraft:lush_caves_ceiling_vegetation",
    "minecraft:spore_blossom",
    "minecraft:classic_vines_cave_feature",
    "minecraft:amethyst_geode",
    "minecraft:sculk_patch_deep_dark",
    "minecraft:sculk_patch_ancient_city",
    "minecraft:sculk_vein",
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

fn cave_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Cave)
        .expect("cave placed-feature bootstrap source")
        .keys
}

#[test]
fn cave_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(CAVE_PLACEMENTS_JAVA.lines().count(), 260);
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        20
    );
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "PlacementUtils.register("),
        20
    );
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "CountPlacement.of"),
        17
    );
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "RarityFilter.onAverageOnceEvery"),
        3
    );
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "EnvironmentScanPlacement.scanningFor"),
        6
    );
    assert_eq!(
        count_occurrences(CAVE_PLACEMENTS_JAVA, "SurfaceRelativeThresholdFilter.of"),
        2
    );

    for sentinel in [
        "MONSTER_ROOM = PlacementUtils.createKey(\"monster_room\")",
        "SCULK_VEIN = PlacementUtils.createKey(\"sculk_vein\")",
        "HeightRangePlacement.uniform(VerticalAnchor.aboveBottom(6), VerticalAnchor.absolute(-1))",
        "CountPlacement.of(UniformInt.of(192, 256))",
        "RandomOffsetPlacement.of(ClampedNormalInt.of(0.0F, 3.0F, -10, 10), ClampedNormalInt.of(0.0F, 0.6F, -2, 2))",
        "EnvironmentScanPlacement.scanningFor(Direction.UP, BlockPredicate.hasSturdyFace(Direction.DOWN), BlockPredicate.ONLY_IN_AIR_PREDICATE, 12)",
        "PlacementUtils.register(context, SCULK_PATCH_ANCIENT_CITY, sculkPatchAncientCity);",
    ] {
        assert!(
            CAVE_PLACEMENTS_JAVA.contains(sentinel),
            "missing CavePlacements sentinel {sentinel}"
        );
    }

    assert_eq!(cave_placed_feature_keys(), CAVE_PLACED_FEATURE_KEYS);
    for key in CAVE_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Cave),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn cave_placements_monster_room_and_fossil_configs_match_vanilla_json() {
    for (id, feature, first_type, first_value, min_key, min_value, max_key, max_value) in [
        (
            "minecraft:monster_room",
            "minecraft:monster_room",
            "minecraft:count",
            10,
            "absolute",
            0,
            "below_top",
            0,
        ),
        (
            "minecraft:monster_room_deep",
            "minecraft:monster_room",
            "minecraft:count",
            4,
            "above_bottom",
            6,
            "absolute",
            -1,
        ),
        (
            "minecraft:fossil_upper",
            "minecraft:fossil_coal",
            "minecraft:rarity_filter",
            64,
            "absolute",
            0,
            "below_top",
            0,
        ),
        (
            "minecraft:fossil_lower",
            "minecraft:fossil_diamonds",
            "minecraft:rarity_filter",
            64,
            "above_bottom",
            0,
            "absolute",
            -8,
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"][0]["type"], first_type, "{id}");
        assert_eq!(
            placed["placement"][0]["count"]
                .as_i64()
                .or_else(|| placed["placement"][0]["chance"].as_i64()),
            Some(first_value),
            "{id}"
        );
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["min_inclusive"][min_key], min_value,
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["max_inclusive"][max_key], max_value,
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }
}

#[test]
fn cave_placements_dripstone_magma_and_lichen_configs_match_vanilla_json() {
    for (id, min_count, max_count) in [
        ("minecraft:dripstone_cluster", 48, 96),
        ("minecraft:large_dripstone", 10, 48),
        ("minecraft:underwater_magma", 44, 52),
        ("minecraft:glow_lichen", 104, 157),
        ("minecraft:sculk_vein", 204, 250),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"][0]["type"], "minecraft:count", "{id}");
        assert_eq!(
            placed["placement"][0]["count"]["min_inclusive"], min_count,
            "{id}"
        );
        assert_eq!(
            placed["placement"][0]["count"]["max_inclusive"], max_count,
            "{id}"
        );
        let modifiers = placed["placement"]
            .as_array()
            .expect("cave placement modifiers");
        assert!(
            modifiers
                .iter()
                .any(|modifier| modifier["type"] == "minecraft:height_range"),
            "missing height range for {id}"
        );
        assert_eq!(
            modifiers.last().expect("final cave placement modifier")["type"],
            "minecraft:biome",
            "{id}"
        );
    }

    let pointed = placed_feature_json("minecraft:pointed_dripstone");
    assert_eq!(pointed["placement"][3]["count"]["min_inclusive"], 1);
    assert_eq!(pointed["placement"][3]["count"]["max_inclusive"], 5);
    assert_eq!(
        pointed["placement"][4]["xz_spread"]["type"],
        "minecraft:clamped_normal"
    );
    assert_eq!(pointed["placement"][4]["xz_spread"]["max_inclusive"], 10);
    assert_eq!(pointed["placement"][4]["y_spread"]["max_inclusive"], 2);

    let magma = placed_feature_json("minecraft:underwater_magma");
    assert_eq!(
        magma["placement"][3]["type"],
        "minecraft:surface_relative_threshold_filter"
    );
    assert_eq!(magma["placement"][3]["heightmap"], "OCEAN_FLOOR_WG");
    assert_eq!(magma["placement"][3]["max_inclusive"], -2);

    let lichen = placed_feature_json("minecraft:glow_lichen");
    assert_eq!(lichen["placement"][1]["type"], "minecraft:height_range");
    assert_eq!(lichen["placement"][2]["type"], "minecraft:in_square");
    assert_eq!(lichen["placement"][3]["max_inclusive"], -13);
}

#[test]
fn cave_placements_lush_cave_scan_configs_match_vanilla_json() {
    for (id, feature, count, direction, target_type, offset) in [
        (
            "minecraft:rooted_azalea_tree",
            "minecraft:rooted_azalea_tree",
            None,
            "up",
            "minecraft:solid",
            -1,
        ),
        (
            "minecraft:cave_vines",
            "minecraft:cave_vine",
            Some(188),
            "up",
            "minecraft:has_sturdy_face",
            -1,
        ),
        (
            "minecraft:lush_caves_vegetation",
            "minecraft:moss_patch",
            Some(125),
            "down",
            "minecraft:solid",
            1,
        ),
        (
            "minecraft:lush_caves_clay",
            "minecraft:lush_caves_clay",
            Some(62),
            "down",
            "minecraft:solid",
            1,
        ),
        (
            "minecraft:lush_caves_ceiling_vegetation",
            "minecraft:moss_patch_ceiling",
            Some(125),
            "up",
            "minecraft:solid",
            -1,
        ),
        (
            "minecraft:spore_blossom",
            "minecraft:spore_blossom",
            Some(25),
            "up",
            "minecraft:solid",
            -1,
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        if let Some(count) = count {
            assert_eq!(placed["placement"][0]["count"], count, "{id}");
        } else {
            assert_eq!(placed["placement"][0]["count"]["min_inclusive"], 1, "{id}");
            assert_eq!(placed["placement"][0]["count"]["max_inclusive"], 2, "{id}");
        }
        assert_eq!(
            placed["placement"][3]["type"], "minecraft:environment_scan",
            "{id}"
        );
        assert_eq!(
            placed["placement"][3]["direction_of_search"], direction,
            "{id}"
        );
        assert_eq!(placed["placement"][3]["max_steps"], 12, "{id}");
        assert_eq!(
            placed["placement"][3]["target_condition"]["type"], target_type,
            "{id}"
        );
        assert_eq!(placed["placement"][4]["y_spread"], offset, "{id}");
    }
}

#[test]
fn cave_placements_vines_amethyst_and_sculk_configs_match_vanilla_json() {
    let vines = placed_feature_json("minecraft:classic_vines_cave_feature");
    assert_eq!(vines["feature"], "minecraft:vines");
    assert_eq!(vines["placement"][0]["count"], 256);
    assert_eq!(
        vines["placement"][2]["height"]["max_inclusive"]["absolute"],
        256
    );

    let geode = placed_feature_json("minecraft:amethyst_geode");
    assert_eq!(geode["feature"], "minecraft:amethyst_geode");
    assert_eq!(geode["placement"][0]["type"], "minecraft:rarity_filter");
    assert_eq!(geode["placement"][0]["chance"], 24);
    assert_eq!(
        geode["placement"][2]["height"]["min_inclusive"]["above_bottom"],
        6
    );
    assert_eq!(
        geode["placement"][2]["height"]["max_inclusive"]["absolute"],
        30
    );

    let deep_dark = placed_feature_json("minecraft:sculk_patch_deep_dark");
    assert_eq!(deep_dark["feature"], "minecraft:sculk_patch_deep_dark");
    assert_eq!(deep_dark["placement"][0]["count"], 256);

    let ancient_city = placed_feature_json("minecraft:sculk_patch_ancient_city");
    assert_eq!(
        ancient_city["feature"],
        "minecraft:sculk_patch_ancient_city"
    );
    assert!(ancient_city["placement"]
        .as_array()
        .expect("ancient-city sculk placement")
        .is_empty());
}
