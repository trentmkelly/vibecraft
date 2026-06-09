use super::*;

const ORE_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/OrePlacements.java"
);

const ORE_PLACED_FEATURE_KEYS: [&str; 40] = [
    "minecraft:ore_magma",
    "minecraft:ore_soul_sand",
    "minecraft:ore_gold_deltas",
    "minecraft:ore_quartz_deltas",
    "minecraft:ore_gold_nether",
    "minecraft:ore_quartz_nether",
    "minecraft:ore_gravel_nether",
    "minecraft:ore_blackstone",
    "minecraft:ore_dirt",
    "minecraft:ore_gravel",
    "minecraft:ore_granite_upper",
    "minecraft:ore_granite_lower",
    "minecraft:ore_diorite_upper",
    "minecraft:ore_diorite_lower",
    "minecraft:ore_andesite_upper",
    "minecraft:ore_andesite_lower",
    "minecraft:ore_tuff",
    "minecraft:ore_coal_upper",
    "minecraft:ore_coal_lower",
    "minecraft:ore_iron_upper",
    "minecraft:ore_iron_middle",
    "minecraft:ore_iron_small",
    "minecraft:ore_gold_extra",
    "minecraft:ore_gold",
    "minecraft:ore_gold_lower",
    "minecraft:ore_redstone",
    "minecraft:ore_redstone_lower",
    "minecraft:ore_diamond",
    "minecraft:ore_diamond_medium",
    "minecraft:ore_diamond_large",
    "minecraft:ore_diamond_buried",
    "minecraft:ore_lapis",
    "minecraft:ore_lapis_buried",
    "minecraft:ore_infested",
    "minecraft:ore_emerald",
    "minecraft:ore_ancient_debris_large",
    "minecraft:ore_debris_small",
    "minecraft:ore_copper",
    "minecraft:ore_copper_large",
    "minecraft:ore_clay",
];

const ORE_FEATURE_REFS: [(&str, &str); 40] = [
    ("minecraft:ore_magma", "minecraft:ore_magma"),
    ("minecraft:ore_soul_sand", "minecraft:ore_soul_sand"),
    ("minecraft:ore_gold_deltas", "minecraft:ore_nether_gold"),
    ("minecraft:ore_quartz_deltas", "minecraft:ore_quartz"),
    ("minecraft:ore_gold_nether", "minecraft:ore_nether_gold"),
    ("minecraft:ore_quartz_nether", "minecraft:ore_quartz"),
    ("minecraft:ore_gravel_nether", "minecraft:ore_gravel_nether"),
    ("minecraft:ore_blackstone", "minecraft:ore_blackstone"),
    ("minecraft:ore_dirt", "minecraft:ore_dirt"),
    ("minecraft:ore_gravel", "minecraft:ore_gravel"),
    ("minecraft:ore_granite_upper", "minecraft:ore_granite"),
    ("minecraft:ore_granite_lower", "minecraft:ore_granite"),
    ("minecraft:ore_diorite_upper", "minecraft:ore_diorite"),
    ("minecraft:ore_diorite_lower", "minecraft:ore_diorite"),
    ("minecraft:ore_andesite_upper", "minecraft:ore_andesite"),
    ("minecraft:ore_andesite_lower", "minecraft:ore_andesite"),
    ("minecraft:ore_tuff", "minecraft:ore_tuff"),
    ("minecraft:ore_coal_upper", "minecraft:ore_coal"),
    ("minecraft:ore_coal_lower", "minecraft:ore_coal_buried"),
    ("minecraft:ore_iron_upper", "minecraft:ore_iron"),
    ("minecraft:ore_iron_middle", "minecraft:ore_iron"),
    ("minecraft:ore_iron_small", "minecraft:ore_iron_small"),
    ("minecraft:ore_gold_extra", "minecraft:ore_gold"),
    ("minecraft:ore_gold", "minecraft:ore_gold_buried"),
    ("minecraft:ore_gold_lower", "minecraft:ore_gold_buried"),
    ("minecraft:ore_redstone", "minecraft:ore_redstone"),
    ("minecraft:ore_redstone_lower", "minecraft:ore_redstone"),
    ("minecraft:ore_diamond", "minecraft:ore_diamond_small"),
    (
        "minecraft:ore_diamond_medium",
        "minecraft:ore_diamond_medium",
    ),
    ("minecraft:ore_diamond_large", "minecraft:ore_diamond_large"),
    (
        "minecraft:ore_diamond_buried",
        "minecraft:ore_diamond_buried",
    ),
    ("minecraft:ore_lapis", "minecraft:ore_lapis"),
    ("minecraft:ore_lapis_buried", "minecraft:ore_lapis_buried"),
    ("minecraft:ore_infested", "minecraft:ore_infested"),
    ("minecraft:ore_emerald", "minecraft:ore_emerald"),
    (
        "minecraft:ore_ancient_debris_large",
        "minecraft:ore_ancient_debris_large",
    ),
    (
        "minecraft:ore_debris_small",
        "minecraft:ore_ancient_debris_small",
    ),
    ("minecraft:ore_copper", "minecraft:ore_copper_small"),
    ("minecraft:ore_copper_large", "minecraft:ore_copper_large"),
    ("minecraft:ore_clay", "minecraft:ore_clay"),
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn placed_feature_json(id: &str) -> serde_json::Value {
    let path = format!(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/placed_feature/{}.json",
        id.trim_start_matches("minecraft:")
    );
    let json =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
    serde_json::from_str(&json).unwrap_or_else(|err| panic!("failed to parse {path}: {err}"))
}

fn ore_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Ore)
        .expect("ore placed-feature bootstrap source")
        .keys
}

#[test]
fn ore_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(ORE_PLACEMENTS_JAVA.lines().count(), 255);
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        40
    );
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "PlacementUtils.register("),
        40
    );
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "commonOrePlacement("),
        34
    );
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "rareOrePlacement("),
        5
    );
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "HeightRangePlacement.uniform"),
        21
    );
    assert_eq!(
        count_occurrences(ORE_PLACEMENTS_JAVA, "HeightRangePlacement.triangle"),
        13
    );

    for sentinel in [
        "ORE_MAGMA = PlacementUtils.createKey(\"ore_magma\")",
        "ORE_ANCIENT_DEBRIS_SMALL = PlacementUtils.createKey(\"ore_debris_small\")",
        "ORE_COPPER = PlacementUtils.createKey(\"ore_copper\")",
        "return List.of(frequencyModifier, InSquarePlacement.spread(), heightRange, BiomeFilter.biome());",
        "orePlacement(CountPlacement.of(UniformInt.of(0, 1)), HeightRangePlacement.uniform(VerticalAnchor.absolute(-64), VerticalAnchor.absolute(-48)))",
        "HeightRangePlacement.triangle(VerticalAnchor.aboveBottom(-80), VerticalAnchor.aboveBottom(80))",
        "PlacementUtils.RANGE_BOTTOM_TO_MAX_TERRAIN_HEIGHT",
    ] {
        assert!(
            ORE_PLACEMENTS_JAVA.contains(sentinel),
            "missing OrePlacements sentinel {sentinel}"
        );
    }

    assert_eq!(ore_placed_feature_keys(), ORE_PLACED_FEATURE_KEYS);
    for key in ORE_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Ore),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn ore_placements_feature_refs_match_vanilla_json() {
    for (id, feature) in ORE_FEATURE_REFS {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
        assert_eq!(
            placed["placement"]
                .as_array()
                .expect("ore placement modifiers")
                .last()
                .expect("final ore placement modifier")["type"],
            "minecraft:biome",
            "{id}"
        );
    }
}

#[test]
fn ore_placements_nether_and_stone_configs_match_vanilla_json() {
    for (id, count, min_key, min_value, max_key, max_value) in [
        ("minecraft:ore_magma", 4, "absolute", 27, "absolute", 36),
        (
            "minecraft:ore_soul_sand",
            12,
            "above_bottom",
            0,
            "absolute",
            31,
        ),
        (
            "minecraft:ore_gold_deltas",
            20,
            "above_bottom",
            10,
            "below_top",
            10,
        ),
        (
            "minecraft:ore_quartz_deltas",
            32,
            "above_bottom",
            10,
            "below_top",
            10,
        ),
        (
            "minecraft:ore_granite_lower",
            2,
            "absolute",
            0,
            "absolute",
            60,
        ),
        ("minecraft:ore_tuff", 2, "above_bottom", 0, "absolute", 0),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"][0]["type"], "minecraft:count", "{id}");
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
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
    }

    let granite_upper = placed_feature_json("minecraft:ore_granite_upper");
    assert_eq!(
        granite_upper["placement"][0]["type"],
        "minecraft:rarity_filter"
    );
    assert_eq!(granite_upper["placement"][0]["chance"], 6);
    assert_eq!(
        granite_upper["placement"][2]["height"]["min_inclusive"]["absolute"],
        64
    );
    assert_eq!(
        granite_upper["placement"][2]["height"]["max_inclusive"]["absolute"],
        128
    );
}

#[test]
fn ore_placements_overworld_trapezoid_configs_match_vanilla_json() {
    for (id, count, min_key, min_value, max_key, max_value) in [
        (
            "minecraft:ore_coal_lower",
            20,
            "absolute",
            0,
            "absolute",
            192,
        ),
        (
            "minecraft:ore_iron_upper",
            90,
            "absolute",
            80,
            "absolute",
            384,
        ),
        (
            "minecraft:ore_redstone_lower",
            8,
            "above_bottom",
            -32,
            "above_bottom",
            32,
        ),
        (
            "minecraft:ore_diamond",
            7,
            "above_bottom",
            -80,
            "above_bottom",
            80,
        ),
        ("minecraft:ore_lapis", 2, "absolute", -32, "absolute", 32),
        (
            "minecraft:ore_emerald",
            100,
            "absolute",
            -16,
            "absolute",
            480,
        ),
    ] {
        let placed = placed_feature_json(id);
        assert_eq!(placed["placement"][0]["count"], count, "{id}");
        assert_eq!(
            placed["placement"][2]["height"]["type"], "minecraft:trapezoid",
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
    }

    let diamond_large = placed_feature_json("minecraft:ore_diamond_large");
    assert_eq!(
        diamond_large["placement"][0]["type"],
        "minecraft:rarity_filter"
    );
    assert_eq!(diamond_large["placement"][0]["chance"], 9);
}

#[test]
fn ore_placements_special_configs_match_vanilla_json() {
    let gold_lower = placed_feature_json("minecraft:ore_gold_lower");
    assert_eq!(
        gold_lower["placement"][0]["count"]["type"],
        "minecraft:uniform"
    );
    assert_eq!(gold_lower["placement"][0]["count"]["min_inclusive"], 0);
    assert_eq!(gold_lower["placement"][0]["count"]["max_inclusive"], 1);
    assert_eq!(
        gold_lower["placement"][2]["height"]["min_inclusive"]["absolute"],
        -64
    );
    assert_eq!(
        gold_lower["placement"][2]["height"]["max_inclusive"]["absolute"],
        -48
    );

    let debris_large = placed_feature_json("minecraft:ore_ancient_debris_large");
    assert_eq!(debris_large["placement"][0]["type"], "minecraft:in_square");
    assert_eq!(
        debris_large["placement"][1]["height"]["type"],
        "minecraft:trapezoid"
    );
    assert_eq!(
        debris_large["placement"][1]["height"]["min_inclusive"]["absolute"],
        8
    );
    assert_eq!(
        debris_large["placement"][1]["height"]["max_inclusive"]["absolute"],
        24
    );

    let debris_small = placed_feature_json("minecraft:ore_debris_small");
    assert_eq!(
        debris_small["placement"][1]["height"]["min_inclusive"]["above_bottom"],
        8
    );
    assert_eq!(
        debris_small["placement"][1]["height"]["max_inclusive"]["below_top"],
        8
    );

    let copper = placed_feature_json("minecraft:ore_copper");
    assert_eq!(copper["feature"], "minecraft:ore_copper_small");
    assert_eq!(copper["placement"][0]["count"], 16);
    assert_eq!(
        copper["placement"][2]["height"]["max_inclusive"]["absolute"],
        112
    );

    let clay = placed_feature_json("minecraft:ore_clay");
    assert_eq!(clay["placement"][0]["count"], 46);
    assert_eq!(
        clay["placement"][2]["height"]["max_inclusive"]["absolute"],
        256
    );
}
