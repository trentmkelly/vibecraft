use super::*;

const AQUATIC_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/AquaticFeatures.java"
);
const CAVE_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/CaveFeatures.java"
);
const END_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/EndFeatures.java"
);
const FEATURE_UTILS_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/FeatureUtils.java"
);
const MISC_OVERWORLD_FEATURES_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/MiscOverworldFeatures.java"
);
const DESERT_WELL_FEATURE_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/world/level/levelgen/feature/DesertWellFeature.java"
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
fn feature_utils_java_bootstrap_dispatch_matches_configured_feature_registry() {
    assert_eq!(FEATURE_UTILS_JAVA.lines().count(), 40);
    assert_eq!(
        count_occurrences(FEATURE_UTILS_JAVA, "Features.bootstrap(context);"),
        9
    );
    assert_eq!(count_occurrences(FEATURE_UTILS_JAVA, "void register("), 2);
    for sentinel in [
        "AquaticFeatures.bootstrap(context);",
        "CaveFeatures.bootstrap(context);",
        "EndFeatures.bootstrap(context);",
        "MiscOverworldFeatures.bootstrap(context);",
        "NetherFeatures.bootstrap(context);",
        "OreFeatures.bootstrap(context);",
        "PileFeatures.bootstrap(context);",
        "TreeFeatures.bootstrap(context);",
        "VegetationFeatures.bootstrap(context);",
        "ResourceKey.create(Registries.CONFIGURED_FEATURE, Identifier.withDefaultNamespace(name))",
        "register(context, id, feature, FeatureConfiguration.NONE);",
        "context.register(id, new ConfiguredFeature(feature, config));",
    ] {
        assert!(
            FEATURE_UTILS_JAVA.contains(sentinel),
            "missing FeatureUtils sentinel {sentinel}"
        );
    }

    assert_eq!(CONFIGURED_FEATURES.len(), 221);
    assert_eq!(
        CONFIGURED_FEATURES
            .iter()
            .take_while(|feature| feature.source == ConfiguredFeatureSource::Aquatic)
            .count(),
        7
    );
    assert_eq!(
        CONFIGURED_FEATURES
            .iter()
            .skip_while(|feature| feature.source == ConfiguredFeatureSource::Aquatic)
            .take_while(|feature| feature.source == ConfiguredFeatureSource::Cave)
            .count(),
        24
    );
    assert_eq!(
        super::super::configured_feature("kelp").map(|feature| feature.id),
        Some("minecraft:kelp")
    );
    assert_eq!(
        super::super::configured_feature("minecraft:end_island").map(|feature| feature.source),
        Some(ConfiguredFeatureSource::End)
    );
    assert!(super::super::configured_feature("minecraft:missing_feature").is_none());
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
fn cave_features_java_source_shape_matches_configured_feature_registry() {
    assert_eq!(CAVE_FEATURES_JAVA.lines().count(), 494);
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "FeatureUtils.createKey("),
        24
    );
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "FeatureUtils.register("),
        24
    );
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "PlacementUtils.inlinePlaced"),
        12
    );
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "new VegetationPatchConfiguration"),
        5
    );
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "new BlockColumnConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(CAVE_FEATURES_JAVA, "new SculkPatchConfiguration"),
        2
    );

    for sentinel in [
        "MONSTER_ROOM = FeatureUtils.createKey(\"monster_room\")",
        "SCULK_VEIN = FeatureUtils.createKey(\"sculk_vein\")",
        "HolderGetter<ConfiguredFeature<?, ?>> configuredFeatures = context.lookup(Registries.CONFIGURED_FEATURE);",
        "HolderGetter<StructureProcessorList> processorLists = context.lookup(Registries.PROCESSOR_LIST);",
        "FeatureUtils.register(context, MONSTER_ROOM, Feature.MONSTER_ROOM);",
        "new FossilFeatureConfiguration(fossilStructures, fossilCoalStructures, fossilRot, processorLists.getOrThrow(ProcessorLists.FOSSIL_COAL), 4)",
        "new UnderwaterMagmaConfiguration(5, 1, 0.5F)",
        "new SculkPatchConfiguration(10, 32, 64, 0, 1, ConstantInt.of(0), 0.5F)",
        "new SculkPatchConfiguration(10, 32, 64, 0, 1, UniformInt.of(1, 3), 0.5F)",
        "new GeodeLayerSettings(1.7, 2.2, 3.2, 4.2)",
    ] {
        assert!(
            CAVE_FEATURES_JAVA.contains(sentinel),
            "missing CaveFeatures sentinel {sentinel}"
        );
    }
}

#[test]
fn cave_features_configured_feature_keys_match_java_bootstrap() {
    let cave_keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::Cave)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        cave_keys,
        vec![
            "minecraft:monster_room",
            "minecraft:fossil_coal",
            "minecraft:fossil_diamonds",
            "minecraft:dripstone_cluster",
            "minecraft:large_dripstone",
            "minecraft:pointed_dripstone",
            "minecraft:underwater_magma",
            "minecraft:glow_lichen",
            "minecraft:rooted_azalea_tree",
            "minecraft:cave_vine",
            "minecraft:cave_vine_in_moss",
            "minecraft:moss_vegetation",
            "minecraft:moss_patch",
            "minecraft:moss_patch_bonemeal",
            "minecraft:dripleaf",
            "minecraft:clay_with_dripleaves",
            "minecraft:clay_pool_with_dripleaves",
            "minecraft:lush_caves_clay",
            "minecraft:moss_patch_ceiling",
            "minecraft:spore_blossom",
            "minecraft:amethyst_geode",
            "minecraft:sculk_patch_deep_dark",
            "minecraft:sculk_patch_ancient_city",
            "minecraft:sculk_vein",
        ]
    );
}

#[test]
fn cave_features_configured_feature_types_match_vanilla_json() {
    for (id, feature_type) in [
        ("minecraft:monster_room", "minecraft:monster_room"),
        ("minecraft:fossil_coal", "minecraft:fossil"),
        ("minecraft:fossil_diamonds", "minecraft:fossil"),
        ("minecraft:dripstone_cluster", "minecraft:dripstone_cluster"),
        ("minecraft:large_dripstone", "minecraft:large_dripstone"),
        (
            "minecraft:pointed_dripstone",
            "minecraft:simple_random_selector",
        ),
        ("minecraft:underwater_magma", "minecraft:underwater_magma"),
        ("minecraft:glow_lichen", "minecraft:multiface_growth"),
        ("minecraft:rooted_azalea_tree", "minecraft:root_system"),
        ("minecraft:cave_vine", "minecraft:block_column"),
        ("minecraft:cave_vine_in_moss", "minecraft:block_column"),
        ("minecraft:moss_vegetation", "minecraft:simple_block"),
        ("minecraft:moss_patch", "minecraft:vegetation_patch"),
        (
            "minecraft:moss_patch_bonemeal",
            "minecraft:vegetation_patch",
        ),
        ("minecraft:dripleaf", "minecraft:simple_random_selector"),
        (
            "minecraft:clay_with_dripleaves",
            "minecraft:vegetation_patch",
        ),
        (
            "minecraft:clay_pool_with_dripleaves",
            "minecraft:waterlogged_vegetation_patch",
        ),
        (
            "minecraft:lush_caves_clay",
            "minecraft:random_boolean_selector",
        ),
        ("minecraft:moss_patch_ceiling", "minecraft:vegetation_patch"),
        ("minecraft:spore_blossom", "minecraft:simple_block"),
        ("minecraft:amethyst_geode", "minecraft:geode"),
        ("minecraft:sculk_patch_deep_dark", "minecraft:sculk_patch"),
        (
            "minecraft:sculk_patch_ancient_city",
            "minecraft:sculk_patch",
        ),
        ("minecraft:sculk_vein", "minecraft:multiface_growth"),
    ] {
        assert_eq!(configured_feature_json(id)["type"], feature_type, "{id}");
    }
}

#[test]
fn cave_features_representative_configs_match_vanilla_json() {
    let fossil_coal = configured_feature_json("minecraft:fossil_coal");
    assert_eq!(
        fossil_coal["config"]["fossil_structures"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(
        fossil_coal["config"]["overlay_structures"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(
        fossil_coal["config"]["fossil_processors"],
        "minecraft:fossil_rot"
    );
    assert_eq!(
        fossil_coal["config"]["overlay_processors"],
        "minecraft:fossil_coal"
    );
    assert_eq!(fossil_coal["config"]["max_empty_corners_allowed"], 4);

    let dripstone_cluster = configured_feature_json("minecraft:dripstone_cluster");
    assert_eq!(
        dripstone_cluster["config"]["floor_to_ceiling_search_range"],
        12
    );
    assert_eq!(dripstone_cluster["config"]["height"]["min_inclusive"], 3);
    assert_eq!(dripstone_cluster["config"]["height"]["max_inclusive"], 6);
    assert_eq!(dripstone_cluster["config"]["radius"]["min_inclusive"], 2);
    assert_eq!(dripstone_cluster["config"]["radius"]["max_inclusive"], 8);

    let underwater_magma = configured_feature_json("minecraft:underwater_magma");
    assert_eq!(underwater_magma["config"]["floor_search_range"], 5);
    assert_eq!(
        underwater_magma["config"]["placement_radius_around_floor"],
        1
    );
    assert_eq!(
        underwater_magma["config"]["placement_probability_per_valid_position"],
        0.5
    );

    let moss_patch = configured_feature_json("minecraft:moss_patch");
    assert_eq!(
        moss_patch["config"]["replaceable"],
        "#minecraft:moss_replaceable"
    );
    assert_eq!(moss_patch["config"]["surface"], "floor");
    assert_eq!(
        moss_patch["config"]["vegetation_feature"]["feature"],
        "minecraft:moss_vegetation"
    );

    let lush_caves_clay = configured_feature_json("minecraft:lush_caves_clay");
    assert_eq!(
        lush_caves_clay["config"]["feature_true"]["feature"],
        "minecraft:clay_with_dripleaves"
    );
    assert_eq!(
        lush_caves_clay["config"]["feature_false"]["feature"],
        "minecraft:clay_pool_with_dripleaves"
    );

    let deep_dark = configured_feature_json("minecraft:sculk_patch_deep_dark");
    assert_eq!(deep_dark["config"]["charge_count"], 10);
    assert_eq!(deep_dark["config"]["extra_rare_growths"], 0);

    let ancient_city = configured_feature_json("minecraft:sculk_patch_ancient_city");
    assert_eq!(
        ancient_city["config"]["extra_rare_growths"]["min_inclusive"],
        1
    );
    assert_eq!(
        ancient_city["config"]["extra_rare_growths"]["max_inclusive"],
        3
    );
}

#[test]
fn end_features_java_bootstrap_matches_configured_feature_registry() {
    assert_eq!(END_FEATURES_JAVA.lines().count(), 28);
    assert_eq!(
        count_occurrences(END_FEATURES_JAVA, "FeatureUtils.createKey("),
        6
    );
    assert_eq!(
        count_occurrences(END_FEATURES_JAVA, "FeatureUtils.register("),
        6
    );
    for sentinel in [
        "END_PLATFORM = FeatureUtils.createKey(\"end_platform\")",
        "END_GATEWAY_DELAYED = FeatureUtils.createKey(\"end_gateway_delayed\")",
        "FeatureUtils.register(context, END_PLATFORM, Feature.END_PLATFORM);",
        "new EndSpikeConfiguration(false, ImmutableList.of(), null)",
        "EndGatewayConfiguration.knownExit(ServerLevel.END_SPAWN_POINT, true)",
        "EndGatewayConfiguration.delayedExitSearch()",
        "FeatureUtils.register(context, CHORUS_PLANT, Feature.CHORUS_PLANT);",
        "FeatureUtils.register(context, END_ISLAND, Feature.END_ISLAND);",
    ] {
        assert!(
            END_FEATURES_JAVA.contains(sentinel),
            "missing EndFeatures sentinel {sentinel}"
        );
    }

    let end_keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::End)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        end_keys,
        vec![
            "minecraft:end_platform",
            "minecraft:end_spike",
            "minecraft:end_gateway_return",
            "minecraft:end_gateway_delayed",
            "minecraft:chorus_plant",
            "minecraft:end_island",
        ]
    );

    for (id, feature_type) in [
        ("minecraft:end_platform", "minecraft:end_platform"),
        ("minecraft:end_spike", "minecraft:end_spike"),
        ("minecraft:end_gateway_return", "minecraft:end_gateway"),
        ("minecraft:end_gateway_delayed", "minecraft:end_gateway"),
        ("minecraft:chorus_plant", "minecraft:chorus_plant"),
        ("minecraft:end_island", "minecraft:end_island"),
    ] {
        assert_eq!(configured_feature_json(id)["type"], feature_type, "{id}");
    }

    let end_spike = configured_feature_json("minecraft:end_spike");
    assert_eq!(end_spike["config"]["crystal_invulnerable"], false);
    assert_eq!(end_spike["config"]["spikes"].as_array().unwrap().len(), 0);

    let gateway_return = configured_feature_json("minecraft:end_gateway_return");
    assert_eq!(gateway_return["config"]["exact"], true);
    assert_eq!(
        gateway_return["config"]["exit"],
        serde_json::json!([100, 50, 0])
    );

    let gateway_delayed = configured_feature_json("minecraft:end_gateway_delayed");
    assert_eq!(gateway_delayed["config"]["exact"], false);
    assert!(gateway_delayed["config"].get("exit").is_none());
}

#[test]
fn misc_overworld_features_java_source_shape_matches_configured_feature_registry() {
    assert_eq!(MISC_OVERWORLD_FEATURES_JAVA.lines().count(), 201);
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_FEATURES_JAVA, "FeatureUtils.createKey("),
        18
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_FEATURES_JAVA, "FeatureUtils.register("),
        18
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_FEATURES_JAVA, "new DiskConfiguration"),
        5
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_FEATURES_JAVA, "new SpringConfiguration"),
        3
    );
    assert_eq!(
        count_occurrences(MISC_OVERWORLD_FEATURES_JAVA, "new RuleBasedStateProvider("),
        2
    );
    for sentinel in [
        "ICE_SPIKE = FeatureUtils.createKey(\"ice_spike\")",
        "SPRING_WATER = FeatureUtils.createKey(\"spring_water\")",
        "new SpikeConfiguration(",
        "new BlockBlobConfiguration(Blocks.MOSSY_COBBLESTONE.defaultBlockState(), BlockPredicate.matchesTag(BlockTags.FOREST_ROCK_CAN_PLACE_ON))",
        "FeatureUtils.register(context, BLUE_ICE, Feature.BLUE_ICE);",
        "FeatureUtils.register(context, FREEZE_TOP_LAYER, Feature.FREEZE_TOP_LAYER);",
        "FeatureUtils.register(context, DESERT_WELL, Feature.DESERT_WELL);",
        "new SpringConfiguration(",
        "Fluids.WATER.defaultFluidState()",
    ] {
        assert!(
            MISC_OVERWORLD_FEATURES_JAVA.contains(sentinel),
            "missing MiscOverworldFeatures sentinel {sentinel}"
        );
    }
}

#[test]
fn misc_overworld_features_keys_and_types_match_vanilla_json() {
    let keys = CONFIGURED_FEATURES
        .iter()
        .filter(|feature| feature.source == ConfiguredFeatureSource::MiscOverworld)
        .map(|feature| feature.id)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "minecraft:ice_spike",
            "minecraft:ice_patch",
            "minecraft:forest_rock",
            "minecraft:iceberg_packed",
            "minecraft:iceberg_blue",
            "minecraft:blue_ice",
            "minecraft:lake_lava",
            "minecraft:disk_clay",
            "minecraft:disk_gravel",
            "minecraft:disk_sand",
            "minecraft:freeze_top_layer",
            "minecraft:disk_grass",
            "minecraft:bonus_chest",
            "minecraft:void_start_platform",
            "minecraft:desert_well",
            "minecraft:spring_lava_overworld",
            "minecraft:spring_lava_frozen",
            "minecraft:spring_water",
        ]
    );

    for (id, feature_type) in [
        ("minecraft:ice_spike", "minecraft:spike"),
        ("minecraft:ice_patch", "minecraft:disk"),
        ("minecraft:forest_rock", "minecraft:block_blob"),
        ("minecraft:iceberg_packed", "minecraft:iceberg"),
        ("minecraft:iceberg_blue", "minecraft:iceberg"),
        ("minecraft:blue_ice", "minecraft:blue_ice"),
        ("minecraft:lake_lava", "minecraft:lake"),
        ("minecraft:disk_clay", "minecraft:disk"),
        ("minecraft:disk_gravel", "minecraft:disk"),
        ("minecraft:disk_sand", "minecraft:disk"),
        ("minecraft:freeze_top_layer", "minecraft:freeze_top_layer"),
        ("minecraft:disk_grass", "minecraft:disk"),
        ("minecraft:bonus_chest", "minecraft:bonus_chest"),
        (
            "minecraft:void_start_platform",
            "minecraft:void_start_platform",
        ),
        ("minecraft:desert_well", "minecraft:desert_well"),
        (
            "minecraft:spring_lava_overworld",
            "minecraft:spring_feature",
        ),
        ("minecraft:spring_lava_frozen", "minecraft:spring_feature"),
        ("minecraft:spring_water", "minecraft:spring_feature"),
    ] {
        assert_eq!(configured_feature_json(id)["type"], feature_type, "{id}");
    }
}

#[test]
fn desert_well_suspicious_sand_assigns_archaeology_loot_like_java() {
    for sentinel in [
        "List.of(waterCenter, waterCenter.east(), waterCenter.south(), waterCenter.west(), waterCenter.north())",
        "placeSusSand(level, Util.getRandom(waterPositions, random).below(1));",
        "placeSusSand(level, Util.getRandom(waterPositions, random).below(2));",
        "Blocks.SUSPICIOUS_SAND.defaultBlockState()",
        "e.setLootTable(BuiltInLootTables.DESERT_WELL_ARCHAEOLOGY, pos.asLong())",
    ] {
        assert!(
            DESERT_WELL_FEATURE_JAVA.contains(sentinel),
            "missing DesertWellFeature sentinel {sentinel}"
        );
    }

    let origin = BlockPos { x: 10, y: 64, z: -3 };
    let placements = super::super::desert_well_suspicious_sand_placements(origin, 1, 4);

    assert_eq!(
        placements[0],
        super::super::DesertWellSuspiciousSandPlacement {
            pos: BlockPos { x: 11, y: 63, z: -3 },
            state: "minecraft:suspicious_sand",
            loot_table: "minecraft:archaeology/desert_well",
            loot_seed: crate::lighting::positions::block_pos_as_long(11, 63, -3),
        }
    );
    assert_eq!(
        placements[1],
        super::super::DesertWellSuspiciousSandPlacement {
            pos: BlockPos { x: 10, y: 62, z: -4 },
            state: "minecraft:suspicious_sand",
            loot_table: "minecraft:archaeology/desert_well",
            loot_seed: crate::lighting::positions::block_pos_as_long(10, 62, -4),
        }
    );
}

#[test]
fn misc_overworld_features_representative_configs_match_vanilla_json() {
    let ice_spike = configured_feature_json("minecraft:ice_spike");
    assert_eq!(ice_spike["config"]["state"]["Name"], "minecraft:packed_ice");
    assert_eq!(
        ice_spike["config"]["can_replace"]["tag"],
        "minecraft:ice_spike_replaceable"
    );

    let disk_sand = configured_feature_json("minecraft:disk_sand");
    assert_eq!(
        disk_sand["config"]["state_provider"]["fallback"]["state"]["Name"],
        "minecraft:sand"
    );
    assert_eq!(
        disk_sand["config"]["state_provider"]["rules"][0]["then"]["state"]["Name"],
        "minecraft:sandstone"
    );
    assert_eq!(disk_sand["config"]["radius"]["max_inclusive"], 6);
    assert_eq!(disk_sand["config"]["half_height"], 2);

    let disk_grass = configured_feature_json("minecraft:disk_grass");
    assert_eq!(
        disk_grass["config"]["state_provider"]["rules"][0]["then"]["state"]["Name"],
        "minecraft:grass_block"
    );
    assert_eq!(
        disk_grass["config"]["target"]["blocks"],
        serde_json::json!(["minecraft:dirt", "minecraft:mud"])
    );

    for (id, state_name, valid_block_count) in [
        ("minecraft:spring_lava_overworld", "minecraft:lava", 8),
        ("minecraft:spring_lava_frozen", "minecraft:lava", 3),
        ("minecraft:spring_water", "minecraft:water", 11),
    ] {
        let spring = configured_feature_json(id);
        assert_eq!(spring["config"]["state"]["Name"], state_name, "{id}");
        assert_eq!(spring["config"]["requires_block_below"], true, "{id}");
        assert_eq!(spring["config"]["rock_count"], 4, "{id}");
        assert_eq!(spring["config"]["hole_count"], 1, "{id}");
        assert_eq!(
            spring["config"]["valid_blocks"].as_array().unwrap().len(),
            valid_block_count,
            "{id}"
        );
    }
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
