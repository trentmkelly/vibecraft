use super::*;

const VEGETATION_PLACEMENTS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/placement/VegetationPlacements.java"
);

const VEGETATION_PLACED_FEATURE_KEYS: [&str; 89] = [
    "minecraft:bamboo_light",
    "minecraft:bamboo",
    "minecraft:vines",
    "minecraft:patch_sunflower",
    "minecraft:patch_pumpkin",
    "minecraft:patch_grass_plain",
    "minecraft:patch_grass_meadow",
    "minecraft:patch_grass_forest",
    "minecraft:patch_grass_badlands",
    "minecraft:patch_grass_savanna",
    "minecraft:patch_grass_normal",
    "minecraft:patch_grass_taiga_2",
    "minecraft:patch_grass_taiga",
    "minecraft:patch_grass_jungle",
    "minecraft:grass_bonemeal",
    "minecraft:patch_dead_bush_2",
    "minecraft:patch_dead_bush",
    "minecraft:patch_dead_bush_badlands",
    "minecraft:patch_dry_grass_badlands",
    "minecraft:patch_dry_grass_desert",
    "minecraft:patch_melon",
    "minecraft:patch_melon_sparse",
    "minecraft:patch_berry_common",
    "minecraft:patch_berry_rare",
    "minecraft:patch_waterlily",
    "minecraft:patch_tall_grass_2",
    "minecraft:patch_tall_grass",
    "minecraft:patch_large_fern",
    "minecraft:patch_bush",
    "minecraft:patch_leaf_litter",
    "minecraft:patch_cactus_desert",
    "minecraft:patch_cactus_decorated",
    "minecraft:patch_sugar_cane_swamp",
    "minecraft:patch_sugar_cane_desert",
    "minecraft:patch_sugar_cane_badlands",
    "minecraft:patch_sugar_cane",
    "minecraft:patch_firefly_bush_swamp",
    "minecraft:patch_firefly_bush_near_water_swamp",
    "minecraft:patch_firefly_bush_near_water",
    "minecraft:brown_mushroom_nether",
    "minecraft:red_mushroom_nether",
    "minecraft:brown_mushroom_normal",
    "minecraft:red_mushroom_normal",
    "minecraft:brown_mushroom_taiga",
    "minecraft:red_mushroom_taiga",
    "minecraft:brown_mushroom_old_growth",
    "minecraft:red_mushroom_old_growth",
    "minecraft:brown_mushroom_swamp",
    "minecraft:red_mushroom_swamp",
    "minecraft:flower_warm",
    "minecraft:flower_default",
    "minecraft:flower_flower_forest",
    "minecraft:flower_swamp",
    "minecraft:flower_plains",
    "minecraft:flower_meadow",
    "minecraft:flower_cherry",
    "minecraft:flower_pale_garden",
    "minecraft:wildflowers_birch_forest",
    "minecraft:wildflowers_meadow",
    "minecraft:trees_plains",
    "minecraft:dark_forest_vegetation",
    "minecraft:pale_garden_vegetation",
    "minecraft:flower_forest_flowers",
    "minecraft:forest_flowers",
    "minecraft:pale_garden_flowers",
    "minecraft:pale_moss_patch",
    "minecraft:trees_flower_forest",
    "minecraft:trees_meadow",
    "minecraft:trees_cherry",
    "minecraft:trees_taiga",
    "minecraft:trees_grove",
    "minecraft:trees_badlands",
    "minecraft:trees_snowy",
    "minecraft:trees_swamp",
    "minecraft:trees_windswept_savanna",
    "minecraft:trees_savanna",
    "minecraft:birch_tall",
    "minecraft:trees_birch",
    "minecraft:trees_windswept_forest",
    "minecraft:trees_windswept_hills",
    "minecraft:trees_water",
    "minecraft:trees_birch_and_oak_leaf_litter",
    "minecraft:trees_sparse_jungle",
    "minecraft:trees_old_growth_spruce_taiga",
    "minecraft:trees_old_growth_pine_taiga",
    "minecraft:trees_jungle",
    "minecraft:bamboo_vegetation",
    "minecraft:mushroom_island_vegetation",
    "minecraft:trees_mangrove",
];

const VEGETATION_FEATURE_REFS: [(&str, &str); 89] = [
    ("minecraft:bamboo_light", "minecraft:bamboo_no_podzol"),
    ("minecraft:bamboo", "minecraft:bamboo_some_podzol"),
    ("minecraft:vines", "minecraft:vines"),
    ("minecraft:patch_sunflower", "minecraft:sunflower"),
    ("minecraft:patch_pumpkin", "minecraft:pumpkin"),
    ("minecraft:patch_grass_plain", "minecraft:grass"),
    ("minecraft:patch_grass_meadow", "minecraft:grass"),
    ("minecraft:patch_grass_forest", "minecraft:grass"),
    ("minecraft:patch_grass_badlands", "minecraft:grass"),
    ("minecraft:patch_grass_savanna", "minecraft:grass"),
    ("minecraft:patch_grass_normal", "minecraft:grass"),
    ("minecraft:patch_grass_taiga_2", "minecraft:taiga_grass"),
    ("minecraft:patch_grass_taiga", "minecraft:taiga_grass"),
    ("minecraft:patch_grass_jungle", "minecraft:grass_jungle"),
    ("minecraft:grass_bonemeal", "minecraft:grass"),
    ("minecraft:patch_dead_bush_2", "minecraft:dead_bush"),
    ("minecraft:patch_dead_bush", "minecraft:dead_bush"),
    ("minecraft:patch_dead_bush_badlands", "minecraft:dead_bush"),
    ("minecraft:patch_dry_grass_badlands", "minecraft:dry_grass"),
    ("minecraft:patch_dry_grass_desert", "minecraft:dry_grass"),
    ("minecraft:patch_melon", "minecraft:melon"),
    ("minecraft:patch_melon_sparse", "minecraft:melon"),
    ("minecraft:patch_berry_common", "minecraft:berry_bush"),
    ("minecraft:patch_berry_rare", "minecraft:berry_bush"),
    ("minecraft:patch_waterlily", "minecraft:waterlily"),
    ("minecraft:patch_tall_grass_2", "minecraft:tall_grass"),
    ("minecraft:patch_tall_grass", "minecraft:tall_grass"),
    ("minecraft:patch_large_fern", "minecraft:large_fern"),
    ("minecraft:patch_bush", "minecraft:bush"),
    ("minecraft:patch_leaf_litter", "minecraft:leaf_litter"),
    ("minecraft:patch_cactus_desert", "minecraft:cactus"),
    ("minecraft:patch_cactus_decorated", "minecraft:cactus"),
    ("minecraft:patch_sugar_cane_swamp", "minecraft:sugar_cane"),
    ("minecraft:patch_sugar_cane_desert", "minecraft:sugar_cane"),
    (
        "minecraft:patch_sugar_cane_badlands",
        "minecraft:sugar_cane",
    ),
    ("minecraft:patch_sugar_cane", "minecraft:sugar_cane"),
    (
        "minecraft:patch_firefly_bush_swamp",
        "minecraft:firefly_bush",
    ),
    (
        "minecraft:patch_firefly_bush_near_water_swamp",
        "minecraft:firefly_bush",
    ),
    (
        "minecraft:patch_firefly_bush_near_water",
        "minecraft:firefly_bush",
    ),
    (
        "minecraft:brown_mushroom_nether",
        "minecraft:brown_mushroom",
    ),
    ("minecraft:red_mushroom_nether", "minecraft:red_mushroom"),
    (
        "minecraft:brown_mushroom_normal",
        "minecraft:brown_mushroom",
    ),
    ("minecraft:red_mushroom_normal", "minecraft:red_mushroom"),
    ("minecraft:brown_mushroom_taiga", "minecraft:brown_mushroom"),
    ("minecraft:red_mushroom_taiga", "minecraft:red_mushroom"),
    (
        "minecraft:brown_mushroom_old_growth",
        "minecraft:brown_mushroom",
    ),
    (
        "minecraft:red_mushroom_old_growth",
        "minecraft:red_mushroom",
    ),
    ("minecraft:brown_mushroom_swamp", "minecraft:brown_mushroom"),
    ("minecraft:red_mushroom_swamp", "minecraft:red_mushroom"),
    ("minecraft:flower_warm", "minecraft:flower_default"),
    ("minecraft:flower_default", "minecraft:flower_default"),
    (
        "minecraft:flower_flower_forest",
        "minecraft:flower_flower_forest",
    ),
    ("minecraft:flower_swamp", "minecraft:flower_swamp"),
    ("minecraft:flower_plains", "minecraft:flower_plain"),
    ("minecraft:flower_meadow", "minecraft:flower_meadow"),
    ("minecraft:flower_cherry", "minecraft:flower_cherry"),
    (
        "minecraft:flower_pale_garden",
        "minecraft:flower_pale_garden",
    ),
    ("minecraft:wildflowers_birch_forest", "minecraft:wildflower"),
    ("minecraft:wildflowers_meadow", "minecraft:wildflower"),
    ("minecraft:trees_plains", "minecraft:trees_plains"),
    (
        "minecraft:dark_forest_vegetation",
        "minecraft:dark_forest_vegetation",
    ),
    (
        "minecraft:pale_garden_vegetation",
        "minecraft:pale_garden_vegetation",
    ),
    (
        "minecraft:flower_forest_flowers",
        "minecraft:forest_flowers",
    ),
    ("minecraft:forest_flowers", "minecraft:forest_flowers"),
    (
        "minecraft:pale_garden_flowers",
        "minecraft:pale_forest_flower",
    ),
    ("minecraft:pale_moss_patch", "minecraft:pale_moss_patch"),
    (
        "minecraft:trees_flower_forest",
        "minecraft:trees_flower_forest",
    ),
    ("minecraft:trees_meadow", "minecraft:meadow_trees"),
    ("minecraft:trees_cherry", "minecraft:cherry_bees_005"),
    ("minecraft:trees_taiga", "minecraft:trees_taiga"),
    ("minecraft:trees_grove", "minecraft:trees_grove"),
    ("minecraft:trees_badlands", "minecraft:trees_badlands"),
    ("minecraft:trees_snowy", "minecraft:trees_snowy"),
    ("minecraft:trees_swamp", "minecraft:swamp_oak"),
    (
        "minecraft:trees_windswept_savanna",
        "minecraft:trees_savanna",
    ),
    ("minecraft:trees_savanna", "minecraft:trees_savanna"),
    ("minecraft:birch_tall", "minecraft:birch_tall"),
    ("minecraft:trees_birch", "minecraft:trees_birch"),
    (
        "minecraft:trees_windswept_forest",
        "minecraft:trees_windswept_hills",
    ),
    (
        "minecraft:trees_windswept_hills",
        "minecraft:trees_windswept_hills",
    ),
    ("minecraft:trees_water", "minecraft:trees_water"),
    (
        "minecraft:trees_birch_and_oak_leaf_litter",
        "minecraft:trees_birch_and_oak_leaf_litter",
    ),
    (
        "minecraft:trees_sparse_jungle",
        "minecraft:trees_sparse_jungle",
    ),
    (
        "minecraft:trees_old_growth_spruce_taiga",
        "minecraft:trees_old_growth_spruce_taiga",
    ),
    (
        "minecraft:trees_old_growth_pine_taiga",
        "minecraft:trees_old_growth_pine_taiga",
    ),
    ("minecraft:trees_jungle", "minecraft:trees_jungle"),
    ("minecraft:bamboo_vegetation", "minecraft:bamboo_vegetation"),
    (
        "minecraft:mushroom_island_vegetation",
        "minecraft:mushroom_island_vegetation",
    ),
    ("minecraft:trees_mangrove", "minecraft:mangrove_vegetation"),
];

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn count_standalone_count_placement_calls(source: &str) -> usize {
    source
        .match_indices("CountPlacement.of(")
        .filter(|(index, _)| *index == 0 || !source.as_bytes()[index - 1].is_ascii_alphabetic())
        .count()
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

fn vegetation_placed_feature_keys() -> &'static [&'static str] {
    PLACED_FEATURE_BOOTSTRAP_SOURCES
        .iter()
        .find(|entry| entry.source == PlacedFeatureSource::Vegetation)
        .expect("vegetation placed-feature bootstrap source")
        .keys
}

#[test]
fn vegetation_placements_java_bootstrap_matches_placed_feature_registry() {
    assert_eq!(VEGETATION_PLACEMENTS_JAVA.lines().count(), 926);
    assert_eq!(
        count_occurrences(VEGETATION_PLACEMENTS_JAVA, "PlacementUtils.createKey("),
        89
    );
    assert_eq!(
        count_occurrences(VEGETATION_PLACEMENTS_JAVA, "PlacementUtils.register("),
        89
    );
    assert_eq!(
        count_standalone_count_placement_calls(VEGETATION_PLACEMENTS_JAVA),
        60
    );
    assert_eq!(
        count_occurrences(
            VEGETATION_PLACEMENTS_JAVA,
            "RarityFilter.onAverageOnceEvery"
        ),
        33
    );
    assert_eq!(
        count_occurrences(VEGETATION_PLACEMENTS_JAVA, "PlacementUtils.countExtra"),
        21
    );
    assert_eq!(
        count_occurrences(
            VEGETATION_PLACEMENTS_JAVA,
            "NoiseThresholdCountPlacement.of"
        ),
        6
    );
    assert_eq!(
        count_occurrences(
            VEGETATION_PLACEMENTS_JAVA,
            "SurfaceWaterDepthFilter.forMaxDepth"
        ),
        4
    );

    for sentinel in [
        "BAMBOO_LIGHT = PlacementUtils.createKey(\"bamboo_light\")",
        "TREES_MANGROVE = PlacementUtils.createKey(\"trees_mangrove\")",
        "private static final PlacementModifier TREE_THRESHOLD = SurfaceWaterDepthFilter.forMaxDepth(0);",
        "return List.of(CountPlacement.of(count), InSquarePlacement.spread(), PlacementUtils.HEIGHTMAP_WORLD_SURFACE, BiomeFilter.biome());",
        "RandomOffsetPlacement.ofTriangle(7, 3)",
        "PlacementUtils.countExtra(0, 0.05F, 1)",
        "SurfaceWaterDepthFilter.forMaxDepth(5)",
    ] {
        assert!(
            VEGETATION_PLACEMENTS_JAVA.contains(sentinel),
            "missing VegetationPlacements sentinel {sentinel}"
        );
    }

    assert_eq!(
        vegetation_placed_feature_keys(),
        VEGETATION_PLACED_FEATURE_KEYS
    );
    for key in VEGETATION_PLACED_FEATURE_KEYS {
        assert_eq!(
            super::super::placed_feature_source(key),
            Some(PlacedFeatureSource::Vegetation),
            "wrong placed-feature source for {key}"
        );
    }
}

#[test]
fn vegetation_placements_feature_refs_match_vanilla_json() {
    for (id, feature) in VEGETATION_FEATURE_REFS {
        let placed = placed_feature_json(id);
        assert_eq!(placed["feature"], feature, "{id}");
    }
}

#[test]
fn vegetation_placements_basic_patch_configs_match_vanilla_json() {
    let bamboo_light = placed_feature_json("minecraft:bamboo_light");
    assert_eq!(
        bamboo_light["placement"][0]["type"],
        "minecraft:rarity_filter"
    );
    assert_eq!(bamboo_light["placement"][0]["chance"], 4);
    assert_eq!(bamboo_light["placement"][2]["heightmap"], "MOTION_BLOCKING");

    let bamboo = placed_feature_json("minecraft:bamboo");
    assert_eq!(
        bamboo["placement"][0]["type"],
        "minecraft:noise_based_count"
    );
    assert_eq!(bamboo["placement"][0]["noise_to_count_ratio"], 160);
    assert_eq!(bamboo["placement"][0]["noise_factor"], 80.0);
    assert_eq!(bamboo["placement"][0]["noise_offset"], 0.3);
    assert_eq!(bamboo["placement"][2]["heightmap"], "WORLD_SURFACE_WG");

    let vines = placed_feature_json("minecraft:vines");
    assert_eq!(vines["placement"][0]["count"], 127);
    assert_eq!(vines["placement"][2]["type"], "minecraft:height_range");
    assert_eq!(
        vines["placement"][2]["height"]["min_inclusive"]["absolute"],
        64
    );
    assert_eq!(
        vines["placement"][2]["height"]["max_inclusive"]["absolute"],
        100
    );

    let pumpkin = placed_feature_json("minecraft:patch_pumpkin");
    assert_eq!(pumpkin["placement"][0]["chance"], 300);
    assert_eq!(pumpkin["placement"][4]["count"], 96);
    assert_eq!(
        pumpkin["placement"][5]["xz_spread"],
        serde_json::json!({"type": "minecraft:trapezoid", "min": -7, "max": 7, "plateau": 0})
    );
    assert_eq!(
        pumpkin["placement"][6]["predicate"]["predicates"][1]["blocks"],
        "minecraft:grass_block"
    );
}

#[test]
fn vegetation_placements_mushroom_and_flower_configs_match_vanilla_json() {
    let mushroom = placed_feature_json("minecraft:brown_mushroom_normal");
    assert_eq!(mushroom["placement"][0]["type"], "minecraft:rarity_filter");
    assert_eq!(mushroom["placement"][0]["chance"], 256);
    assert_eq!(mushroom["placement"][4]["count"], 96);
    assert_eq!(
        mushroom["placement"][6]["predicate"]["tag"],
        "minecraft:air"
    );

    let old_growth_red = placed_feature_json("minecraft:red_mushroom_old_growth");
    assert_eq!(old_growth_red["placement"][0]["chance"], 171);

    let flower_plains = placed_feature_json("minecraft:flower_plains");
    assert_eq!(
        flower_plains["placement"][0]["type"],
        "minecraft:noise_threshold_count"
    );
    assert_eq!(flower_plains["placement"][0]["below_noise"], 15);
    assert_eq!(flower_plains["placement"][0]["above_noise"], 4);
    assert_eq!(flower_plains["placement"][1]["chance"], 32);
    assert_eq!(flower_plains["placement"][5]["count"], 64);

    let wildflowers = placed_feature_json("minecraft:wildflowers_meadow");
    assert_eq!(wildflowers["placement"][0]["below_noise"], 5);
    assert_eq!(wildflowers["placement"][0]["above_noise"], 10);
    assert_eq!(wildflowers["placement"][4]["count"], 8);
}

#[test]
fn vegetation_placements_tree_and_water_configs_match_vanilla_json() {
    let plains = placed_feature_json("minecraft:trees_plains");
    assert_eq!(
        plains["placement"][0]["count"]["type"],
        "minecraft:weighted_list"
    );
    assert_eq!(
        plains["placement"][0]["count"]["distribution"][0]["data"],
        0
    );
    assert_eq!(
        plains["placement"][0]["count"]["distribution"][0]["weight"],
        19
    );
    assert_eq!(
        plains["placement"][0]["count"]["distribution"][1]["data"],
        1
    );
    assert_eq!(
        plains["placement"][0]["count"]["distribution"][1]["weight"],
        1
    );
    assert_eq!(
        plains["placement"][2]["type"],
        "minecraft:surface_water_depth_filter"
    );
    assert_eq!(plains["placement"][2]["max_water_depth"], 0);
    assert_eq!(plains["placement"][3]["heightmap"], "OCEAN_FLOOR");
    assert_eq!(
        plains["placement"][4]["predicate"]["state"]["Name"],
        "minecraft:oak_sapling"
    );

    let cherry = placed_feature_json("minecraft:trees_cherry");
    assert_eq!(
        cherry["placement"][0]["count"]["distribution"][0]["data"],
        10
    );
    assert_eq!(
        cherry["placement"][0]["count"]["distribution"][0]["weight"],
        9
    );
    assert_eq!(
        cherry["placement"][0]["count"]["distribution"][1]["data"],
        11
    );
    assert_eq!(
        cherry["placement"][5]["predicate"]["state"]["Name"],
        "minecraft:cherry_sapling"
    );

    let swamp = placed_feature_json("minecraft:trees_swamp");
    assert_eq!(swamp["placement"][2]["max_water_depth"], 2);

    let mangrove = placed_feature_json("minecraft:trees_mangrove");
    assert_eq!(mangrove["placement"][0]["count"], 25);
    assert_eq!(mangrove["placement"][2]["max_water_depth"], 5);
    assert_eq!(mangrove["placement"][3]["heightmap"], "OCEAN_FLOOR");
}

#[test]
fn vegetation_placements_near_water_predicates_match_vanilla_json() {
    for id in [
        "minecraft:patch_sugar_cane_swamp",
        "minecraft:patch_firefly_bush_near_water",
    ] {
        let placed = placed_feature_json(id);
        let predicate = if id.ends_with("sugar_cane_swamp") {
            &placed["placement"][6]["predicate"]
        } else {
            &placed["placement"][4]["predicate"]
        };
        assert_eq!(predicate["type"], "minecraft:all_of", "{id}");
        assert_eq!(predicate["predicates"][0]["tag"], "minecraft:air", "{id}");
        assert_eq!(
            predicate["predicates"][2]["type"], "minecraft:any_of",
            "{id}"
        );
        assert_eq!(
            predicate["predicates"][2]["predicates"]
                .as_array()
                .expect("water adjacency predicates")
                .len(),
            4,
            "{id}"
        );
        assert_eq!(
            predicate["predicates"][2]["predicates"][0]["fluids"],
            serde_json::json!(["minecraft:water", "minecraft:flowing_water"]),
            "{id}"
        );
    }
}
