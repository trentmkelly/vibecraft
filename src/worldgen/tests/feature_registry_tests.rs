use super::*;

const AQUATIC_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/AquaticFeatures.java"
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
fn feature_type_registry_matches_vanilla_feature_order() {
    assert_eq!(FEATURE_TYPES.len(), 60);
    assert_eq!(
        FEATURE_TYPES
            .iter()
            .map(|feature| feature.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:no_op",
            "minecraft:tree",
            "minecraft:fallen_tree",
            "minecraft:block_pile",
            "minecraft:spring_feature",
            "minecraft:chorus_plant",
            "minecraft:replace_single_block",
            "minecraft:void_start_platform",
            "minecraft:desert_well",
            "minecraft:fossil",
            "minecraft:huge_red_mushroom",
            "minecraft:huge_brown_mushroom",
            "minecraft:spike",
            "minecraft:glowstone_blob",
            "minecraft:freeze_top_layer",
            "minecraft:vines",
            "minecraft:block_column",
            "minecraft:vegetation_patch",
            "minecraft:waterlogged_vegetation_patch",
            "minecraft:root_system",
            "minecraft:multiface_growth",
            "minecraft:underwater_magma",
            "minecraft:monster_room",
            "minecraft:blue_ice",
            "minecraft:iceberg",
            "minecraft:block_blob",
            "minecraft:disk",
            "minecraft:lake",
            "minecraft:ore",
            "minecraft:end_platform",
            "minecraft:end_spike",
            "minecraft:end_island",
            "minecraft:end_gateway",
            "minecraft:seagrass",
            "minecraft:kelp",
            "minecraft:coral_tree",
            "minecraft:coral_mushroom",
            "minecraft:coral_claw",
            "minecraft:sea_pickle",
            "minecraft:simple_block",
            "minecraft:bamboo",
            "minecraft:huge_fungus",
            "minecraft:nether_forest_vegetation",
            "minecraft:weeping_vines",
            "minecraft:twisting_vines",
            "minecraft:basalt_columns",
            "minecraft:delta_feature",
            "minecraft:netherrack_replace_blobs",
            "minecraft:fill_layer",
            "minecraft:bonus_chest",
            "minecraft:basalt_pillar",
            "minecraft:scattered_ore",
            "minecraft:random_selector",
            "minecraft:simple_random_selector",
            "minecraft:random_boolean_selector",
            "minecraft:geode",
            "minecraft:dripstone_cluster",
            "minecraft:large_dripstone",
            "minecraft:pointed_dripstone",
            "minecraft:sculk_patch",
        ]
    );

    let tree = super::super::feature_type_by_id("tree").unwrap();
    assert_eq!(tree.configuration, FeatureConfigurationKind::Tree);
    assert_eq!(tree.family, FeatureFamily::Tree);

    let ore = super::super::feature_type_by_id("minecraft:ore").unwrap();
    assert_eq!(ore.configuration, FeatureConfigurationKind::Ore);
    assert_eq!(ore.family, FeatureFamily::Ore);

    let random_selector = super::super::feature_type_by_id("random_selector").unwrap();
    assert_eq!(
        random_selector.configuration,
        FeatureConfigurationKind::RandomFeature
    );
    assert_eq!(random_selector.family, FeatureFamily::Selector);

    let sculk_patch = super::super::feature_type_by_id("sculk_patch").unwrap();
    assert_eq!(
        sculk_patch.configuration,
        FeatureConfigurationKind::SculkPatch
    );
    assert_eq!(sculk_patch.family, FeatureFamily::Cave);
}

#[test]
fn configured_feature_bootstrap_keys_match_vanilla_sources() {
    assert_eq!(CONFIGURED_FEATURES.len(), 221);

    let source_counts = [
        (ConfiguredFeatureSource::Aquatic, 7),
        (ConfiguredFeatureSource::Cave, 24),
        (ConfiguredFeatureSource::End, 6),
        (ConfiguredFeatureSource::MiscOverworld, 18),
        (ConfiguredFeatureSource::Nether, 22),
        (ConfiguredFeatureSource::Ore, 32),
        (ConfiguredFeatureSource::Pile, 5),
        (ConfiguredFeatureSource::Tree, 50),
        (ConfiguredFeatureSource::Vegetation, 57),
    ];
    for (source, expected_count) in source_counts {
        assert_eq!(
            CONFIGURED_FEATURES
                .iter()
                .filter(|feature| feature.source == source)
                .count(),
            expected_count
        );
    }

    assert_eq!(
        CONFIGURED_FEATURES
            .iter()
            .take(7)
            .map(|feature| feature.id)
            .collect::<Vec<_>>(),
        vec![
            "minecraft:seagrass_short",
            "minecraft:seagrass_slightly_less_short",
            "minecraft:seagrass_mid",
            "minecraft:seagrass_tall",
            "minecraft:sea_pickle",
            "minecraft:kelp",
            "minecraft:warm_ocean_vegetation",
        ]
    );
    assert_eq!(
        CONFIGURED_FEATURES.last().map(|feature| feature.id),
        Some("minecraft:mangrove_vegetation")
    );

    assert_eq!(
        super::super::configured_feature("ore_diamond_buried").map(|feature| feature.source),
        Some(ConfiguredFeatureSource::Ore)
    );
    assert_eq!(
        super::super::configured_feature("minecraft:pale_oak_creaking")
            .map(|feature| feature.source),
        Some(ConfiguredFeatureSource::Tree)
    );
    assert_eq!(
        super::super::configured_feature("sculk_patch_ancient_city").map(|feature| feature.source),
        Some(ConfiguredFeatureSource::Cave)
    );
}

#[test]
fn aquatic_features_java_bootstrap_matches_configured_feature_registry() {
    assert_eq!(AQUATIC_FEATURES_JAVA.lines().count(), 43);
    assert_eq!(
        count_occurrences(AQUATIC_FEATURES_JAVA, "FeatureUtils.createKey("),
        7
    );
    assert_eq!(
        count_occurrences(AQUATIC_FEATURES_JAVA, "FeatureUtils.register("),
        7
    );
    assert_eq!(
        count_occurrences(AQUATIC_FEATURES_JAVA, "new ProbabilityFeatureConfiguration"),
        4
    );
    assert_eq!(
        count_occurrences(AQUATIC_FEATURES_JAVA, "new CountConfiguration(20)"),
        1
    );
    assert_eq!(
        count_occurrences(AQUATIC_FEATURES_JAVA, "PlacementUtils.inlinePlaced"),
        3
    );

    for sentinel in [
        "SEAGRASS_SHORT = FeatureUtils.createKey(\"seagrass_short\")",
        "SEAGRASS_SLIGHTLY_LESS_SHORT = FeatureUtils.createKey(\"seagrass_slightly_less_short\")",
        "WARM_OCEAN_VEGETATION = FeatureUtils.createKey(\"warm_ocean_vegetation\")",
        "FeatureUtils.register(context, SEAGRASS_SHORT, Feature.SEAGRASS, new ProbabilityFeatureConfiguration(0.3F));",
        "FeatureUtils.register(context, SEAGRASS_TALL, Feature.SEAGRASS, new ProbabilityFeatureConfiguration(0.8F));",
        "FeatureUtils.register(context, SEA_PICKLE, Feature.SEA_PICKLE, new CountConfiguration(20));",
        "FeatureUtils.register(context, KELP, Feature.KELP);",
        "Feature.CORAL_TREE, FeatureConfiguration.NONE",
        "Feature.CORAL_CLAW, FeatureConfiguration.NONE",
        "Feature.CORAL_MUSHROOM, FeatureConfiguration.NONE",
    ] {
        assert!(
            AQUATIC_FEATURES_JAVA.contains(sentinel),
            "missing AquaticFeatures sentinel {sentinel}"
        );
    }

    let aquatic_keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Aquatic)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        aquatic_keys,
        vec![
            "minecraft:seagrass_short",
            "minecraft:seagrass_slightly_less_short",
            "minecraft:seagrass_mid",
            "minecraft:seagrass_tall",
            "minecraft:sea_pickle",
            "minecraft:kelp",
            "minecraft:warm_ocean_vegetation",
        ]
    );

    for (id, probability) in [
        ("minecraft:seagrass_short", 0.3),
        ("minecraft:seagrass_slightly_less_short", 0.4),
        ("minecraft:seagrass_mid", 0.6),
        ("minecraft:seagrass_tall", 0.8),
    ] {
        let parsed = configured_feature_json(id);
        assert_eq!(parsed["type"], "minecraft:seagrass", "{id}");
        assert_eq!(parsed["config"]["probability"], probability, "{id}");
    }

    let sea_pickle = configured_feature_json("minecraft:sea_pickle");
    assert_eq!(sea_pickle["type"], "minecraft:sea_pickle");
    assert_eq!(sea_pickle["config"]["count"], 20);

    let kelp = configured_feature_json("minecraft:kelp");
    assert_eq!(kelp["type"], "minecraft:kelp");
    assert_eq!(kelp["config"], serde_json::json!({}));

    let warm_ocean = configured_feature_json("minecraft:warm_ocean_vegetation");
    assert_eq!(warm_ocean["type"], "minecraft:simple_random_selector");
    let features = warm_ocean["config"]["features"]
        .as_array()
        .expect("warm ocean vegetation features must be an array");
    assert_eq!(features.len(), 3);
    assert_eq!(features[0]["feature"]["type"], "minecraft:coral_tree");
    assert_eq!(features[1]["feature"]["type"], "minecraft:coral_claw");
    assert_eq!(features[2]["feature"]["type"], "minecraft:coral_mushroom");
    assert!(features
        .iter()
        .all(|feature| feature["placement"].as_array().is_some_and(Vec::is_empty)));
}

#[test]
fn placed_feature_bootstrap_keys_match_vanilla_sources() {
    assert_eq!(PLACED_FEATURE_BOOTSTRAP_SOURCES.len(), 9);
    assert_eq!(
        PLACED_FEATURE_BOOTSTRAP_SOURCES
            .iter()
            .map(|entry| (entry.source, entry.keys.len()))
            .collect::<Vec<_>>(),
        vec![
            (PlacedFeatureSource::Aquatic, 12),
            (PlacedFeatureSource::Cave, 20),
            (PlacedFeatureSource::End, 5),
            (PlacedFeatureSource::MiscOverworld, 18),
            (PlacedFeatureSource::Nether, 20),
            (PlacedFeatureSource::Ore, 40),
            (PlacedFeatureSource::Tree, 41),
            (PlacedFeatureSource::Vegetation, 89),
            (PlacedFeatureSource::Village, 13),
        ]
    );
    assert_eq!(
        PLACED_FEATURE_BOOTSTRAP_SOURCES
            .iter()
            .map(|entry| entry.keys.len())
            .sum::<usize>(),
        258
    );
    assert_eq!(
        PLACED_FEATURE_BOOTSTRAP_SOURCES[0].keys.first().copied(),
        Some("minecraft:seagrass_warm")
    );
    assert_eq!(
        PLACED_FEATURE_BOOTSTRAP_SOURCES
            .last()
            .and_then(|entry| entry.keys.last())
            .copied(),
        Some("minecraft:patch_berry_bush")
    );
    assert_eq!(
        super::super::placed_feature_source("ore_diamond"),
        Some(PlacedFeatureSource::Ore)
    );
    assert_eq!(
        super::super::placed_feature_source("minecraft:pale_oak_creaking_checked"),
        Some(PlacedFeatureSource::Tree)
    );
    assert_eq!(
        super::super::placed_feature_source("trees_mangrove"),
        Some(PlacedFeatureSource::Vegetation)
    );
}

#[test]
fn placed_ore_feature_models_follow_vanilla_ore_placements() {
    assert_tuff_ore_placement_matches_vanilla();
    assert_granite_ore_placement_matches_vanilla();
    assert_diamond_ore_placement_matches_vanilla();
    assert_lower_gold_ore_placement_matches_vanilla();
    assert_debris_and_copper_ore_placements_match_vanilla();
    assert_disk_sand_placement_matches_vanilla();
}

fn require_placed_ore_feature(id: &'static str) -> super::super::PlacedOreFeatureModel {
    match super::super::placed_ore_feature(id) {
        Some(feature) => feature,
        None => panic!("expected placed ore feature {id}"),
    }
}

fn require_placed_disk_feature(id: &'static str) -> super::super::PlacedDiskFeatureModel {
    match super::super::placed_disk_feature(id) {
        Some(feature) => feature,
        None => panic!("expected placed disk feature {id}"),
    }
}

fn require_disk_configuration(id: &'static str) -> super::super::DiskConfigurationModel {
    match super::super::configured_disk_configuration(id) {
        Some(configuration) => configuration,
        None => panic!("expected disk configuration {id}"),
    }
}

fn assert_tuff_ore_placement_matches_vanilla() {
    let tuff = require_placed_ore_feature("minecraft:ore_tuff");
    assert_eq!(tuff.configured_feature, "minecraft:ore_tuff");
    assert_eq!(
        tuff.placement,
        vec![
            PlacementModifier::Count { count: 2 },
            PlacementModifier::InSquare,
            PlacementModifier::HeightRange {
                height: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::AboveBottom(0),
                    max_inclusive: VerticalAnchor::Absolute(0),
                }
            },
            PlacementModifier::BiomeFilter,
        ]
    );
}

fn assert_granite_ore_placement_matches_vanilla() {
    let granite_upper = require_placed_ore_feature("ore_granite_upper");
    assert_eq!(granite_upper.configured_feature, "minecraft:ore_granite");
    assert_eq!(
        granite_upper.placement,
        vec![
            PlacementModifier::RarityFilter { chance: 6 },
            PlacementModifier::InSquare,
            PlacementModifier::HeightRange {
                height: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::Absolute(64),
                    max_inclusive: VerticalAnchor::Absolute(128),
                }
            },
            PlacementModifier::BiomeFilter,
        ]
    );
}

fn assert_diamond_ore_placement_matches_vanilla() {
    let diamond = require_placed_ore_feature("minecraft:ore_diamond");
    assert_eq!(diamond.configured_feature, "minecraft:ore_diamond_small");
    assert_eq!(
        diamond.placement,
        vec![
            PlacementModifier::Count { count: 7 },
            PlacementModifier::InSquare,
            PlacementModifier::HeightRange {
                height: HeightProvider::Trapezoid {
                    min_inclusive: VerticalAnchor::AboveBottom(-80),
                    max_inclusive: VerticalAnchor::AboveBottom(80),
                    plateau: 0,
                }
            },
            PlacementModifier::BiomeFilter,
        ]
    );
}

fn assert_lower_gold_ore_placement_matches_vanilla() {
    let gold_lower = require_placed_ore_feature("ore_gold_lower");
    assert_eq!(gold_lower.configured_feature, "minecraft:ore_gold_buried");
    assert_eq!(
        gold_lower.placement[0],
        PlacementModifier::CountProvider {
            provider: super::super::IntProviderModel::Uniform {
                min_inclusive: 0,
                max_inclusive: 1,
            },
            sampled_count: 0,
        }
    );
    assert_eq!(
        gold_lower.placement[2],
        PlacementModifier::HeightRange {
            height: HeightProvider::Uniform {
                min_inclusive: VerticalAnchor::Absolute(-64),
                max_inclusive: VerticalAnchor::Absolute(-48),
            }
        }
    );
}

fn assert_debris_and_copper_ore_placements_match_vanilla() {
    let debris_small = require_placed_ore_feature("ore_debris_small");
    assert_eq!(
        debris_small.configured_feature,
        "minecraft:ore_ancient_debris_small"
    );
    assert_eq!(
        debris_small.placement,
        vec![
            PlacementModifier::InSquare,
            PlacementModifier::HeightRange {
                height: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::AboveBottom(8),
                    max_inclusive: VerticalAnchor::BelowTop(8),
                }
            },
            PlacementModifier::BiomeFilter,
        ]
    );

    let copper = require_placed_ore_feature("ore_copper");
    assert_eq!(copper.configured_feature, "minecraft:ore_copper_small");
    assert_eq!(super::super::placed_ore_feature("minecraft:not_ore"), None);
}

fn assert_disk_sand_placement_matches_vanilla() {
    let disk_sand = require_placed_disk_feature("minecraft:disk_sand");
    assert_eq!(disk_sand.configured_feature, "minecraft:disk_sand");
    assert_eq!(
        disk_sand.placement,
        vec![
            PlacementModifier::Count { count: 3 },
            PlacementModifier::InSquare,
            PlacementModifier::Heightmap {
                heightmap: HeightmapKind::OceanFloorWg,
            },
            PlacementModifier::BlockPredicateFilter {
                predicate: BlockPredicate::MatchingFluids {
                    fluids: &["minecraft:water"],
                },
            },
            PlacementModifier::BiomeFilter,
        ]
    );
    let disk_sand_config = require_disk_configuration(disk_sand.configured_feature);
    assert_eq!(
        disk_sand_config.radius,
        super::super::IntProviderModel::Uniform {
            min_inclusive: 2,
            max_inclusive: 6,
        }
    );
    assert_eq!(disk_sand_config.half_height, 2);
    assert_eq!(
        super::super::placed_disk_feature("minecraft:not_disk"),
        None
    );
}
