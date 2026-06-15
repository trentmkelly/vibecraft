use super::*;

const NETHER_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/NetherPlacements.java"
);

const NETHER_PLACED_FEATURE_KEYS: [&str; 20] = [
    "minecraft:delta",
    "minecraft:small_basalt_columns",
    "minecraft:large_basalt_columns",
    "minecraft:basalt_blobs",
    "minecraft:blackstone_blobs",
    "minecraft:glowstone_extra",
    "minecraft:glowstone",
    "minecraft:crimson_forest_vegetation",
    "minecraft:warped_forest_vegetation",
    "minecraft:nether_sprouts",
    "minecraft:twisting_vines",
    "minecraft:weeping_vines",
    "minecraft:patch_crimson_roots",
    "minecraft:basalt_pillar",
    "minecraft:spring_delta",
    "minecraft:spring_closed",
    "minecraft:spring_closed_double",
    "minecraft:spring_open",
    "minecraft:patch_soul_fire",
    "minecraft:patch_fire",
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

fn nether_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Nether)
        .expect("nether placed-feature bootstrap source")
        .keys
}

#[test]
fn nether_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(NETHER_PLACEMENTS_JAVA.lines().count(), 140);
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        20
    );
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "PlacementUtils.register("),
        20
    );
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "CountOnEveryLayerPlacement.of"),
        6
    );
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "CountPlacement.of"),
        14
    );
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "PlacementUtils.FULL_RANGE"),
        7
    );
    assert_eq!(
        count_occurrences(NETHER_PLACEMENTS_JAVA, "BlockPredicateFilter.forPredicate"),
        2
    );

    for sentinel in [
        "DELTA = PlacementUtils.createKey(\"delta\")",
        "PATCH_FIRE = PlacementUtils.createKey(\"patch_fire\")",
        "PlacementUtils.register(context, DELTA, delta, CountOnEveryLayerPlacement.of(40), BiomeFilter.biome())",
        "CountPlacement.of(BiasedToBottomInt.of(0, 9))",
        "SPRING_CLOSED_DOUBLE, springNetherClosed, CountPlacement.of(32)",
        "PlacementUtils.register(context, PATCH_SOUL_FIRE, soulFire, firePlacement(Blocks.SOUL_SOIL));",
        "BlockPredicate.matchesBlocks(Direction.DOWN.getUnitVec3i(), onlyOnBlock)",
    ] {
        assert!(
            NETHER_PLACEMENTS_JAVA.contains(sentinel),
            "missing NetherPlacements sentinel {sentinel}"
        );
    }

    assert_eq!(nether_placed_feature_keys(), NETHER_PLACED_FEATURE_KEYS);
    for key in NETHER_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Nether),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn nether_placements_layer_and_full_range_configs_match_vanilla_json() {
    for (id, count) in [
        ("minecraft:delta", 40),
        ("minecraft:small_basalt_columns", 4),
        ("minecraft:large_basalt_columns", 2),
        ("minecraft:crimson_forest_vegetation", 6),
        ("minecraft:warped_forest_vegetation", 5),
        ("minecraft:nether_sprouts", 4),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], id, "{id}");
        assert_eq!(
            placed["placement"][0]["type"], "minecraft:count_on_every_layer",
            "{id}"
        );
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(placed["placement"][1]["type"], "minecraft:biome", "{id}");
    }

    for (id, feature, count) in [
        ("minecraft:basalt_blobs", "minecraft:basalt_blobs", 75),
        (
            "minecraft:blackstone_blobs",
            "minecraft:blackstone_blobs",
            25,
        ),
        ("minecraft:glowstone", "minecraft:glowstone_extra", 10),
        ("minecraft:twisting_vines", "minecraft:twisting_vines", 10),
        ("minecraft:weeping_vines", "minecraft:weeping_vines", 10),
        ("minecraft:basalt_pillar", "minecraft:basalt_pillar", 10),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["min_inclusive"]["above_bottom"], 0,
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["max_inclusive"]["below_top"], 0,
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }

    let extra = placed_feature_json("minecraft:glowstone_extra");
    assert_eq!(extra["feature"], "minecraft:glowstone_extra");
    assert_eq!(
        extra["placement"][0]["count"]["type"],
        "minecraft:biased_to_bottom"
    );
    assert_eq!(extra["placement"][0]["count"]["max_inclusive"], 9);
    assert_eq!(
        extra["placement"][2]["height"]["min_inclusive"]["above_bottom"],
        4
    );
    assert_eq!(
        extra["placement"][2]["height"]["max_inclusive"]["below_top"],
        4
    );
}

#[test]
fn nether_placements_spring_configs_match_vanilla_json() {
    for (id, feature, count, range) in [
        (
            "minecraft:spring_delta",
            "minecraft:spring_lava_nether",
            16,
            4,
        ),
        (
            "minecraft:spring_closed",
            "minecraft:spring_nether_closed",
            16,
            10,
        ),
        (
            "minecraft:spring_closed_double",
            "minecraft:spring_nether_closed",
            32,
            10,
        ),
        (
            "minecraft:spring_open",
            "minecraft:spring_nether_open",
            8,
            4,
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][1]["type"], "minecraft:in_square",
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["min_inclusive"]["above_bottom"], range,
            "{id}"
        );
        assert_eq!(
            placed["placement"][2]["height"]["max_inclusive"]["below_top"], range,
            "{id}"
        );
        assert_eq!(placed["placement"][3]["type"], "minecraft:biome", "{id}");
    }
}

#[test]
fn nether_placements_root_and_fire_patch_configs_match_vanilla_json() {
    let roots = placed_feature_json("minecraft:patch_crimson_roots");
    assert_eq!(roots["feature"], "minecraft:crimson_roots");
    assert_eq!(roots["placement"][0]["type"], "minecraft:height_range");
    assert_eq!(roots["placement"][1]["type"], "minecraft:biome");
    assert_eq!(roots["placement"][2]["count"], 96);
    assert_eq!(
        roots["placement"][3]["xz_spread"]["type"],
        "minecraft:trapezoid"
    );
    assert_eq!(roots["placement"][3]["xz_spread"]["max"], 7);
    assert_eq!(roots["placement"][4]["predicate"]["tag"], "minecraft:air");

    for (id, base_block) in [
        ("minecraft:patch_soul_fire", "minecraft:soul_soil"),
        ("minecraft:patch_fire", "minecraft:netherrack"),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], id, "{id}");
        assert_eq!(placed["placement"][0]["count"]["min_inclusive"], 0, "{id}");
        assert_eq!(placed["placement"][0]["count"]["max_inclusive"], 5, "{id}");
        assert_eq!(
            placed["placement"][2]["height"]["min_inclusive"]["above_bottom"], 4,
            "{id}"
        );
        assert_eq!(placed["placement"][4]["count"], 96, "{id}");
        assert_eq!(
            placed["placement"][6]["predicate"]["predicates"][1]["blocks"], base_block,
            "{id}"
        );
        assert_eq!(
            placed["placement"][6]["predicate"]["predicates"][1]["offset"],
            serde_json::json!([0, -1, 0]),
            "{id}"
        );
    }
}
