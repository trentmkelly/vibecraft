use super::super::*;

pub(super) fn assert_provider_and_ore_support() {
    assert_worldgen_type_registry_support();
    assert_provider_type_and_basic_samples();
    assert_weighted_and_rotated_provider_samples();
    assert_randomized_int_provider_samples();
    assert_rule_based_provider_samples();
    assert_noise_provider_samples();
    assert_simple_block_placement();
    assert_replace_and_tag_rules();
    assert_configured_ore_models();
    assert_ore_rule_helpers();
    assert_ore_vein_helpers();
    super::feature_placement_support_environment_tests::assert_aquatic_placement_helpers();
    super::feature_placement_support_environment_tests::assert_coral_placement_helpers();
    super::feature_placement_support_environment_tests::assert_vegetation_patch_helpers();
    super::feature_placement_support_environment_tests::assert_lake_placement_helpers();
    super::feature_placement_support_environment_tests::assert_fossil_placement_helpers();
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

fn assert_worldgen_type_registry_support() {
    assert_eq!(
        WORLDGEN_TYPE_REGISTRIES
            .iter()
            .map(|registry| (registry.id, registry.entries.len()))
            .collect::<Vec<_>>(),
        vec![
            ("minecraft:height_provider_type", 6),
            ("minecraft:block_predicate_type", 13),
            ("minecraft:placement_modifier_type", 14),
            ("minecraft:trunk_placer_type", 9),
            ("minecraft:foliage_placer_type", 11),
            ("minecraft:block_state_provider_type", 8),
            ("minecraft:tree_decorator_type", 10),
            ("minecraft:feature_size_type", 2),
            ("minecraft:root_placer_type", 1),
        ]
    );
    assert_registry_entry(
        "minecraft:placement_modifier_type",
        "minecraft:environment_scan",
    );
    assert_registry_entry("minecraft:tree_decorator_type", "minecraft:creaking_heart");
    assert_registry_entry(
        "minecraft:trunk_placer_type",
        "minecraft:upwards_branching_trunk_placer",
    );
    assert_registry_entry(
        "minecraft:block_state_provider_type",
        "minecraft:rule_based_state_provider",
    );
}

fn assert_registry_entry(registry_id: &str, entry_id: &str) {
    assert!(WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == registry_id)
        .unwrap()
        .entries
        .contains(&entry_id));
}

fn assert_provider_type_and_basic_samples() {
    assert_eq!(
        super::super::block_state_provider_type("simple_state_provider"),
        Some("minecraft:simple_state_provider")
    );
    assert_eq!(
        super::super::block_state_provider_type("minecraft:weighted_state_provider"),
        Some("minecraft:weighted_state_provider")
    );
    assert_eq!(
        super::super::block_state_provider_type("minecraft:rotated_block_provider"),
        Some("minecraft:rotated_block_provider")
    );
    assert_eq!(super::super::block_state_provider_type("missing"), None);
    assert_eq!(
        super::super::tree_placement_filter_sapling("minecraft:trees_birch"),
        Some("minecraft:birch_sapling")
    );
    assert_eq!(
        super::super::tree_placement_filter_sapling("minecraft:trees_birch_and_oak_leaf_litter"),
        None,
        "forest leaf-litter trees use vanilla treePlacement without a sapling BlockPredicateFilter"
    );
    assert_eq!(
        super::super::block_state_provider_sample(
            &BlockStateProviderModel::Simple("minecraft:oak_log"),
            99
        ),
        Some("minecraft:oak_log")
    );
}

fn assert_weighted_and_rotated_provider_samples() {
    let weighted = BlockStateProviderModel::Weighted(vec![
        WeightedBlockState {
            state: "minecraft:stone",
            weight: 2,
        },
        WeightedBlockState {
            state: "minecraft:andesite",
            weight: 1,
        },
    ]);
    assert_eq!(
        super::super::block_state_provider_sample(&weighted, 0),
        Some("minecraft:stone")
    );
    assert_eq!(
        super::super::block_state_provider_sample(&weighted, 2),
        Some("minecraft:andesite")
    );
    assert_eq!(
        super::super::block_state_provider_sample(
            &BlockStateProviderModel::Weighted(vec![WeightedBlockState {
                state: "minecraft:air",
                weight: 0,
            }]),
            0,
        ),
        None
    );
    assert_axis_samples(
        "minecraft:hay_block",
        "minecraft:hay_block",
        "minecraft:hay_block[axis=x]",
        "minecraft:hay_block[axis=z]",
    );
    assert_axis_samples(
        "minecraft:oak_log",
        "minecraft:oak_log",
        "minecraft:oak_log[axis=x]",
        "minecraft:oak_log[axis=z]",
    );
    assert_axis_samples(
        "minecraft:basalt",
        "minecraft:basalt",
        "minecraft:basalt[axis=x]",
        "minecraft:basalt[axis=z]",
    );
}

fn assert_axis_samples(
    block_id: &'static str,
    default_state: &'static str,
    x_state: &'static str,
    z_state: &'static str,
) {
    let rotated = BlockStateProviderModel::RotatedBlock(block_id);
    assert_eq!(
        super::super::block_state_provider_sample(&rotated, 0),
        Some(x_state)
    );
    assert_eq!(
        super::super::block_state_provider_sample(&rotated, 1),
        Some(default_state)
    );
    assert_eq!(
        super::super::block_state_provider_sample(&rotated, 2),
        Some(z_state)
    );
}

fn assert_randomized_int_provider_samples() {
    let randomized_cave_vines = BlockStateProviderModel::RandomizedInt {
        source: Box::new(BlockStateProviderModel::Weighted(vec![
            WeightedBlockState {
                state: "minecraft:cave_vines[age=0,berries=false]",
                weight: 4,
            },
            WeightedBlockState {
                state: "minecraft:cave_vines[age=0,berries=true]",
                weight: 1,
            },
        ])),
        property: "age",
        min_inclusive: 23,
        max_inclusive: 25,
    };
    assert_eq!(
        super::super::block_state_provider_sample(&randomized_cave_vines, 0),
        Some("minecraft:cave_vines[age=23,berries=false]")
    );
    assert_eq!(
        super::super::block_state_provider_sample(&randomized_cave_vines, 4),
        Some("minecraft:cave_vines[age=24,berries=true]")
    );
    let randomized_propagule = BlockStateProviderModel::RandomizedInt {
        source: Box::new(BlockStateProviderModel::Simple(
            "minecraft:mangrove_propagule[age=0,hanging=true,stage=0,waterlogged=false]",
        )),
        property: "age",
        min_inclusive: 0,
        max_inclusive: 4,
    };
    assert_eq!(
        super::super::block_state_provider_sample(&randomized_propagule, 3),
        Some("minecraft:mangrove_propagule[age=3,hanging=true,stage=0,waterlogged=false]")
    );
    let missing_property = BlockStateProviderModel::RandomizedInt {
        source: Box::new(BlockStateProviderModel::Simple("minecraft:stone")),
        property: "age",
        min_inclusive: 0,
        max_inclusive: 4,
    };
    assert_eq!(
        super::super::block_state_provider_sample(&missing_property, 3),
        Some("minecraft:stone")
    );
}

fn assert_rule_based_provider_samples() {
    let rule_based_disk = sample_rule_based_disk_provider();
    assert_eq!(
        super::super::block_state_provider_sample_in_context(
            &rule_based_disk,
            0,
            predicate_context("minecraft:air", false, true),
            64,
            "minecraft:air",
        ),
        Some("minecraft:sandstone")
    );
    assert_eq!(
        super::super::block_state_provider_sample_in_context(
            &rule_based_disk,
            0,
            predicate_context("minecraft:dirt", true, false),
            64,
            "minecraft:dirt",
        ),
        Some("minecraft:sand")
    );
    assert_rule_based_random_preservation(&rule_based_disk);
    assert_rule_based_no_fallback_uses_current_block();
}

fn sample_rule_based_disk_provider() -> BlockStateProviderModel {
    BlockStateProviderModel::RuleBased {
        fallback: Some(Box::new(BlockStateProviderModel::Simple("minecraft:sand"))),
        rules: vec![RuleBasedBlockStateProviderRule {
            if_true: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:air"],
            },
            then: Box::new(BlockStateProviderModel::Simple("minecraft:sandstone")),
        }],
    }
}

fn predicate_context(block: &'static str, solid: bool, replaceable: bool) -> BlockPredicateContext {
    BlockPredicateContext {
        min_y: -64,
        height: 384,
        block,
        fluid: "minecraft:empty",
        solid,
        replaceable,
        unobstructed: true,
    }
}

fn assert_rule_based_random_preservation(rule_based_disk: &BlockStateProviderModel) {
    let mut disk_random = crate::random_source::RandomSourceKind::new(
        1234,
        crate::random_source::RandomAlgorithm::Xoroshiro,
    );
    let mut expected_disk_random = disk_random;
    assert_eq!(
        super::super::block_state_provider_sample_in_context_with_random(
            rule_based_disk,
            &mut disk_random,
            predicate_context("minecraft:dirt", true, false),
            64,
            "minecraft:dirt",
        ),
        Some("minecraft:sand")
    );
    assert_eq!(
        super::super::random_next_i32_bound(&mut disk_random, 10_000),
        super::super::random_next_i32_bound(&mut expected_disk_random, 10_000),
        "Java SimpleStateProvider and simple RuleBasedStateProvider branches do not consume RandomSource"
    );
}

fn assert_rule_based_no_fallback_uses_current_block() {
    let no_fallback = BlockStateProviderModel::RuleBased {
        fallback: None,
        rules: vec![RuleBasedBlockStateProviderRule {
            if_true: BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:air"],
            },
            then: Box::new(BlockStateProviderModel::Simple("minecraft:dirt")),
        }],
    };
    assert_eq!(
        super::super::block_state_provider_sample_in_context(
            &no_fallback,
            0,
            predicate_context("minecraft:stone", true, false),
            64,
            "minecraft:stone",
        ),
        Some("minecraft:stone")
    );
}

fn assert_noise_provider_samples() {
    let flower_forest_noise = BlockStateProviderModel::Noise {
        states: vec![
            "minecraft:dandelion",
            "minecraft:poppy",
            "minecraft:allium",
            "minecraft:azure_bluet",
            "minecraft:red_tulip",
            "minecraft:orange_tulip",
            "minecraft:white_tulip",
            "minecraft:pink_tulip",
            "minecraft:oxeye_daisy",
            "minecraft:cornflower",
            "minecraft:lily_of_the_valley",
        ],
    };
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, -1.5),
        Some("minecraft:dandelion")
    );
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, 0.0),
        Some("minecraft:orange_tulip")
    );
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(&flower_forest_noise, 0, 1.5),
        Some("minecraft:lily_of_the_valley")
    );
    assert_noise_threshold_provider_samples();
    assert_dual_noise_provider_samples();
}

fn assert_noise_threshold_provider_samples() {
    let plains_flower_threshold = BlockStateProviderModel::NoiseThreshold {
        threshold: -0.8,
        high_chance: 0.33333334,
        default_state: "minecraft:dandelion",
        low_states: vec![
            "minecraft:orange_tulip",
            "minecraft:red_tulip",
            "minecraft:pink_tulip",
            "minecraft:white_tulip",
        ],
        high_states: vec![
            "minecraft:poppy",
            "minecraft:azure_bluet",
            "minecraft:oxeye_daisy",
            "minecraft:cornflower",
        ],
    };
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(
            &plains_flower_threshold,
            2,
            -0.9
        ),
        Some("minecraft:pink_tulip")
    );
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(
            &plains_flower_threshold,
            200_000,
            -0.7
        ),
        Some("minecraft:poppy")
    );
    assert_eq!(
        super::super::block_state_provider_sample_with_noise_value(
            &plains_flower_threshold,
            900_000,
            -0.7
        ),
        Some("minecraft:dandelion")
    );
}

fn assert_dual_noise_provider_samples() {
    let meadow_dual_noise = BlockStateProviderModel::DualNoise {
        variety_min: 1,
        variety_max: 3,
        states: vec![
            "minecraft:tall_grass[half=lower]",
            "minecraft:allium",
            "minecraft:poppy",
            "minecraft:azure_bluet",
            "minecraft:dandelion",
            "minecraft:cornflower",
            "minecraft:oxeye_daisy",
            "minecraft:short_grass",
        ],
    };
    assert_eq!(
        super::super::block_state_provider_sample_dual_noise_values(
            &meadow_dual_noise,
            -1.0,
            &[1.0],
            0.0,
        ),
        Some("minecraft:short_grass")
    );
    assert_eq!(
        super::super::block_state_provider_sample_dual_noise_values(
            &meadow_dual_noise,
            1.0,
            &[-1.0, -0.5, 0.0, 1.0],
            0.9999,
        ),
        Some("minecraft:short_grass")
    );
    assert_eq!(
        super::super::block_state_provider_sample_dual_noise_values(
            &meadow_dual_noise,
            0.0,
            &[-1.0, 1.0],
            -1.0,
        ),
        Some("minecraft:tall_grass[half=lower]")
    );
}

fn assert_simple_block_placement() {
    let simple_config = super::super::SimpleBlockConfigurationModel {
        to_place: BlockStateProviderModel::Simple("minecraft:dandelion"),
        schedule_tick: false,
    };
    assert_simple_block_plan(
        &simple_config,
        simple_block_context("minecraft:air", "minecraft:grass_block", "minecraft:air"),
        Some(super::super::SimpleBlockPlacementPlan {
            state: "minecraft:dandelion",
            upper_state: None,
            schedule_tick: false,
        }),
    );
    assert_simple_block_plan(
        &simple_config,
        simple_block_context(
            "minecraft:leaf_litter",
            "minecraft:grass_block",
            "minecraft:air",
        ),
        Some(super::super::SimpleBlockPlacementPlan {
            state: "minecraft:dandelion",
            upper_state: None,
            schedule_tick: false,
        }),
    );
    assert_simple_block_plan(
        &simple_config,
        simple_block_context("minecraft:air", "minecraft:stone", "minecraft:air"),
        None,
    );
    assert_sunflower_placement();
}

fn simple_block_context(
    origin_block: &'static str,
    below_block: &'static str,
    above_block: &'static str,
) -> super::super::SimpleBlockPlacementContext {
    super::super::SimpleBlockPlacementContext {
        origin_block,
        below_block,
        above_block,
    }
}

fn assert_simple_block_plan(
    config: &super::super::SimpleBlockConfigurationModel,
    context: super::super::SimpleBlockPlacementContext,
    expected: Option<super::super::SimpleBlockPlacementPlan>,
) {
    let mut random = crate::random_source::RandomSourceKind::new(
        42,
        crate::random_source::RandomAlgorithm::Xoroshiro,
    );
    assert_eq!(
        super::super::simple_block_placement_plan(config, context, &mut random),
        expected
    );
}

fn assert_sunflower_placement() {
    let sunflower_config = super::super::SimpleBlockConfigurationModel {
        to_place: BlockStateProviderModel::Simple("minecraft:sunflower"),
        schedule_tick: true,
    };
    assert_simple_block_plan(
        &sunflower_config,
        simple_block_context("minecraft:air", "minecraft:grass_block", "minecraft:air"),
        Some(super::super::SimpleBlockPlacementPlan {
            state: "minecraft:sunflower",
            upper_state: Some("minecraft:sunflower"),
            schedule_tick: true,
        }),
    );
    assert_simple_block_plan(
        &sunflower_config,
        simple_block_context(
            "minecraft:air",
            "minecraft:grass_block",
            "minecraft:oak_leaves",
        ),
        None,
    );
}

fn assert_replace_and_tag_rules() {
    let replace_targets = [
        super::super::TargetBlockStateModel {
            target: super::super::RuleTestModel::BlockMatch("minecraft:stone"),
            state: "minecraft:granite",
        },
        super::super::TargetBlockStateModel {
            target: super::super::RuleTestModel::TagMatch(&[
                "minecraft:dirt",
                "minecraft:grass_block",
            ]),
            state: "minecraft:coarse_dirt",
        },
        super::super::TargetBlockStateModel {
            target: super::super::RuleTestModel::AlwaysTrue,
            state: "minecraft:air",
        },
    ];
    assert_eq!(
        super::super::replace_block_result("minecraft:stone", &replace_targets),
        Some("minecraft:granite")
    );
    assert_eq!(
        super::super::replace_block_result("minecraft:grass_block", &replace_targets),
        Some("minecraft:coarse_dirt")
    );
    assert_eq!(
        super::super::replace_block_result("minecraft:deepslate", &replace_targets),
        Some("minecraft:air")
    );
    assert_eq!(
        super::super::replace_block_result("minecraft:stone", &[]),
        None
    );
    assert_block_tag_rules();
}

fn assert_block_tag_rules() {
    assert!(super::super::block_matches_tag(
        "minecraft:stone",
        "minecraft:stone_ore_replaceables"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:andesite",
        "minecraft:stone_ore_replaceables"
    ));
    assert!(!super::super::block_matches_tag(
        "minecraft:tuff",
        "minecraft:stone_ore_replaceables"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:deepslate",
        "minecraft:deepslate_ore_replaceables"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:tuff",
        "minecraft:deepslate_ore_replaceables"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:granite",
        "minecraft:base_stone_overworld"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:tuff",
        "minecraft:base_stone_overworld"
    ));
    assert!(!super::super::block_matches_tag(
        "minecraft:netherrack",
        "minecraft:base_stone_overworld"
    ));
    assert!(super::super::block_matches_tag(
        "minecraft:blackstone",
        "minecraft:base_stone_nether"
    ));
    assert!(super::super::rule_test_matches(
        super::super::RuleTestModel::BlockTag("minecraft:base_stone_overworld"),
        "minecraft:deepslate"
    ));
    assert!(!super::super::rule_test_matches(
        super::super::RuleTestModel::BlockTag("minecraft:base_stone_overworld"),
        "minecraft:netherrack"
    ));
}

fn sample_ore_config() -> super::super::OreConfigurationModel {
    super::super::OreConfigurationModel {
        target_states: vec![super::super::TargetBlockStateModel {
            target: super::super::RuleTestModel::BlockMatch("minecraft:stone"),
            state: "minecraft:iron_ore",
        }],
        size: 9,
        discard_chance_on_air_exposure: 0.5,
    }
}

fn assert_configured_ore_models() {
    let tuff_config = super::super::configured_ore_configuration("minecraft:ore_tuff").unwrap();
    assert_eq!(tuff_config.size, 64);
    assert_eq!(tuff_config.discard_chance_on_air_exposure, 0.0);
    assert_eq!(tuff_config.target_states.len(), 1);
    assert_eq!(tuff_config.target_states[0].state, "minecraft:tuff");
    assert!(super::super::rule_test_matches(
        tuff_config.target_states[0].target,
        "minecraft:deepslate"
    ));
    let diamond_config =
        super::super::configured_ore_configuration("minecraft:ore_diamond_large").unwrap();
    assert_eq!(diamond_config.size, 12);
    assert_eq!(diamond_config.discard_chance_on_air_exposure, 0.7);
    assert_eq!(
        diamond_config
            .target_states
            .iter()
            .map(|target| target.state)
            .collect::<Vec<_>>(),
        vec!["minecraft:diamond_ore", "minecraft:deepslate_diamond_ore"]
    );
    assert!(super::super::rule_test_matches(
        diamond_config.target_states[0].target,
        "minecraft:granite"
    ));
    assert!(super::super::rule_test_matches(
        diamond_config.target_states[1].target,
        "minecraft:tuff"
    ));
    assert_configured_debris_ore_model();
}

fn assert_configured_debris_ore_model() {
    let debris_config =
        super::super::configured_ore_configuration("minecraft:ore_ancient_debris_large").unwrap();
    assert_eq!(debris_config.size, 3);
    assert_eq!(debris_config.discard_chance_on_air_exposure, 1.0);
    assert!(super::super::rule_test_matches(
        debris_config.target_states[0].target,
        "minecraft:blackstone"
    ));
    assert_eq!(
        super::super::configured_ore_configuration("minecraft:not_ore"),
        None
    );
}

fn assert_ore_rule_helpers() {
    let ore_config = sample_ore_config();
    let ore_target = ore_config.target_states[0];
    assert!(super::super::ore_should_skip_air_check(0.0, 0.0));
    assert!(!super::super::ore_should_skip_air_check(1.0, 1.0));
    assert!(!super::super::ore_should_skip_air_check(0.5, 0.49));
    assert!(super::super::ore_should_skip_air_check(0.5, 0.5));
    assert!(super::super::ore_can_place(
        "minecraft:stone",
        false,
        &ore_config,
        ore_target,
        0.0,
    ));
    assert!(!super::super::ore_can_place(
        "minecraft:dirt",
        false,
        &ore_config,
        ore_target,
        1.0,
    ));
    assert!(!super::super::ore_can_place(
        "minecraft:stone",
        true,
        &ore_config,
        ore_target,
        0.0,
    ));
    assert_scattered_ore_helpers(&ore_config);
}

fn assert_scattered_ore_helpers(ore_config: &super::super::OreConfigurationModel) {
    assert_eq!(
        super::super::scattered_ore_offset(
            pos(10, 20, 30),
            9,
            [(1.0, 0.0), (0.0, 1.0), (0.75, 0.25)],
        ),
        pos(17, 13, 34)
    );
    assert_eq!(
        super::super::scattered_ore_attempt(
            pos(10, 20, 30),
            2,
            [(1.0, 0.0), (0.0, 1.0), (0.5, 0.5)],
            "minecraft:stone",
            false,
            ore_config,
            0.0,
        ),
        Some(super::super::ScatteredOreAttempt {
            pos: pos(12, 18, 30),
            state: "minecraft:iron_ore",
        })
    );
}

fn assert_ore_vein_helpers() {
    let ore_config = sample_ore_config();
    let ore_spheres = super::super::ore_vein_spheres(pos(8, 32, 8), 8, 0.0, &[(2, 2)], &[1.0; 8]);
    assert!(!ore_spheres.is_empty());
    assert!(ore_spheres
        .iter()
        .any(|sphere| (sphere.center_z - 8.5).abs() < f64::EPSILON));
    let ore_candidates =
        super::super::ore_vein_position_candidates(&ore_spheres, 6, 28, 6, 6, 6, 0..384);
    assert!(!ore_candidates.is_empty());
    assert_ore_vein_boundary_candidate();
    let sampled_candidate = ore_candidates[0];
    assert_eq!(
        ore_candidates
            .iter()
            .filter(|pos| **pos == sampled_candidate)
            .count(),
        1
    );
    assert_ore_placement_plan(&ore_config);
}

fn assert_ore_vein_boundary_candidate() {
    let boundary_candidates = super::super::ore_vein_position_candidates(
        &[super::super::OreVeinSphere {
            center_x: 6.5,
            center_y: 28.5,
            center_z: 12.5,
            radius: 0.51,
        }],
        6,
        28,
        6,
        6,
        6,
        0..384,
    );
    assert!(
        boundary_candidates.contains(&pos(6, 28, 12)),
        "OreFeature's Java BitSet expands for inclusive high-z boundary candidates"
    );
}

fn assert_ore_placement_plan(ore_config: &super::super::OreConfigurationModel) {
    let ore_plan = super::super::ore_placement_plan(
        ore_config,
        &[
            ore_context(pos(8, 32, 8), "minecraft:stone", false, 0.0),
            ore_context(pos(8, 33, 8), "minecraft:stone", true, 0.0),
            ore_context(pos(8, 34, 8), "minecraft:dirt", false, 1.0),
        ],
    );
    assert_eq!(
        ore_plan,
        vec![super::super::OrePlacementBlock {
            pos: pos(8, 32, 8),
            state: "minecraft:iron_ore",
        }]
    );
}

fn ore_context(
    pos: BlockPos,
    current_block: &'static str,
    adjacent_to_air: bool,
    air_check_roll: f32,
) -> super::super::OrePlacementContext {
    super::super::OrePlacementContext {
        pos,
        current_block,
        adjacent_to_air,
        air_check_roll,
    }
}
