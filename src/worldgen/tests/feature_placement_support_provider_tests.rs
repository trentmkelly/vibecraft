use super::super::*;

pub(super) fn assert_provider_and_ore_support() {
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
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:placement_modifier_type")
            .unwrap()
            .entries
            .contains(&"minecraft:environment_scan"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:tree_decorator_type")
            .unwrap()
            .entries
            .contains(&"minecraft:creaking_heart"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:trunk_placer_type")
            .unwrap()
            .entries
            .contains(&"minecraft:upwards_branching_trunk_placer"));
        assert!(WORLDGEN_TYPE_REGISTRIES
            .iter()
            .find(|registry| registry.id == "minecraft:block_state_provider_type")
            .unwrap()
            .entries
            .contains(&"minecraft:rule_based_state_provider"));
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
        let rotated = BlockStateProviderModel::RotatedBlock("minecraft:hay_block");
        assert_eq!(
            super::super::block_state_provider_sample(&rotated, 0),
            Some("minecraft:hay_block[axis=x]")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated, 1),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated, 2),
            Some("minecraft:hay_block[axis=z]")
        );
        let rotated_log = BlockStateProviderModel::RotatedBlock("minecraft:oak_log");
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_log, 0),
            Some("minecraft:oak_log[axis=x]")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_log, 1),
            Some("minecraft:oak_log")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_log, 2),
            Some("minecraft:oak_log[axis=z]")
        );
        let rotated_basalt = BlockStateProviderModel::RotatedBlock("minecraft:basalt");
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_basalt, 0),
            Some("minecraft:basalt[axis=x]")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_basalt, 1),
            Some("minecraft:basalt")
        );
        assert_eq!(
            super::super::block_state_provider_sample(&rotated_basalt, 2),
            Some("minecraft:basalt[axis=z]")
        );
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
        let rule_based_disk = BlockStateProviderModel::RuleBased {
            fallback: Some(Box::new(BlockStateProviderModel::Simple("minecraft:sand"))),
            rules: vec![RuleBasedBlockStateProviderRule {
                if_true: BlockPredicate::MatchingBlocks {
                    blocks: &["minecraft:air"],
                },
                then: Box::new(BlockStateProviderModel::Simple("minecraft:sandstone")),
            }],
        };
        assert_eq!(
            super::super::block_state_provider_sample_in_context(
                &rule_based_disk,
                0,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:air",
                    fluid: "minecraft:empty",
                    solid: false,
                    replaceable: true,
                    unobstructed: true,
                },
                64,
                "minecraft:air",
            ),
            Some("minecraft:sandstone")
        );
        assert_eq!(
            super::super::block_state_provider_sample_in_context(
                &rule_based_disk,
                0,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
                64,
                "minecraft:dirt",
            ),
            Some("minecraft:sand")
        );
        let mut disk_random = crate::random_source::RandomSourceKind::new(
            1234,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        let mut expected_disk_random = disk_random;
        assert_eq!(
            super::super::block_state_provider_sample_in_context_with_random(
                &rule_based_disk,
                &mut disk_random,
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:dirt",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
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
                BlockPredicateContext {
                    min_y: -64,
                    height: 384,
                    block: "minecraft:stone",
                    fluid: "minecraft:empty",
                    solid: true,
                    replaceable: false,
                    unobstructed: true,
                },
                64,
                "minecraft:stone",
            ),
            Some("minecraft:stone")
        );
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
            super::super::block_state_provider_sample_with_noise_value(&plains_flower_threshold, 2, -0.9),
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
        let flower_provider = BlockStateProviderModel::Simple("minecraft:dandelion");
        let simple_config = super::super::SimpleBlockConfigurationModel {
            to_place: flower_provider,
            schedule_tick: false,
        };
        let mut simple_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::super::simple_block_placement_plan(
                &simple_config,
                super::super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut simple_random,
            ),
            Some(super::super::SimpleBlockPlacementPlan {
                state: "minecraft:dandelion",
                upper_state: None,
                schedule_tick: false,
            })
        );
        let mut replace_leaf_litter_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::super::simple_block_placement_plan(
                &simple_config,
                super::super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:leaf_litter",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut replace_leaf_litter_random,
            ),
            Some(super::super::SimpleBlockPlacementPlan {
                state: "minecraft:dandelion",
                upper_state: None,
                schedule_tick: false,
            })
        );
        let mut bad_support_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::super::simple_block_placement_plan(
                &simple_config,
                super::super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:stone",
                    above_block: "minecraft:air",
                },
                &mut bad_support_random,
            ),
            None
        );

        let sunflower_provider = BlockStateProviderModel::Simple("minecraft:sunflower");
        let sunflower_config = super::super::SimpleBlockConfigurationModel {
            to_place: sunflower_provider,
            schedule_tick: true,
        };
        let mut sunflower_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::super::simple_block_placement_plan(
                &sunflower_config,
                super::super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:air",
                },
                &mut sunflower_random,
            ),
            Some(super::super::SimpleBlockPlacementPlan {
                state: "minecraft:sunflower",
                upper_state: Some("minecraft:sunflower"),
                schedule_tick: true,
            })
        );
        let mut obstructed_sunflower_random = crate::random_source::RandomSourceKind::new(
            42,
            crate::random_source::RandomAlgorithm::Xoroshiro,
        );
        assert_eq!(
            super::super::simple_block_placement_plan(
                &sunflower_config,
                super::super::SimpleBlockPlacementContext {
                    origin_block: "minecraft:air",
                    below_block: "minecraft:grass_block",
                    above_block: "minecraft:oak_leaves",
                },
                &mut obstructed_sunflower_random,
            ),
            None
        );

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
        assert_eq!(super::super::replace_block_result("minecraft:stone", &[]), None);
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

        let ore_config = super::super::OreConfigurationModel {
            target_states: vec![super::super::TargetBlockStateModel {
                target: super::super::RuleTestModel::BlockMatch("minecraft:stone"),
                state: "minecraft:iron_ore",
            }],
            size: 9,
            discard_chance_on_air_exposure: 0.5,
        };
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
        assert_eq!(
            super::super::scattered_ore_offset(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                9,
                [(1.0, 0.0), (0.0, 1.0), (0.75, 0.25)],
            ),
            BlockPos {
                x: 17,
                y: 13,
                z: 34,
            }
        );
        assert_eq!(
            super::super::scattered_ore_attempt(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                2,
                [(1.0, 0.0), (0.0, 1.0), (0.5, 0.5)],
                "minecraft:stone",
                false,
                &ore_config,
                0.0,
            ),
            Some(super::super::ScatteredOreAttempt {
                pos: BlockPos {
                    x: 12,
                    y: 18,
                    z: 30,
                },
                state: "minecraft:iron_ore",
            })
        );
        let ore_spheres =
            super::super::ore_vein_spheres(BlockPos { x: 8, y: 32, z: 8 }, 8, 0.0, &[(2, 2)], &[1.0; 8]);
        assert!(!ore_spheres.is_empty());
        assert!(ore_spheres
            .iter()
            .any(|sphere| (sphere.center_z - 8.5).abs() < f64::EPSILON));
        let ore_candidates =
            super::super::ore_vein_position_candidates(&ore_spheres, 6, 28, 6, 6, 6, 0..384);
        assert!(!ore_candidates.is_empty());
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
            boundary_candidates.contains(&BlockPos { x: 6, y: 28, z: 12 }),
            "OreFeature's Java BitSet expands for inclusive high-z boundary candidates"
        );
        let sampled_candidate = ore_candidates[0];
        assert_eq!(
            ore_candidates
                .iter()
                .filter(|pos| **pos == sampled_candidate)
                .count(),
            1
        );
        let ore_plan = super::super::ore_placement_plan(
            &ore_config,
            &[
                super::super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 32, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: false,
                    air_check_roll: 0.0,
                },
                super::super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 33, z: 8 },
                    current_block: "minecraft:stone",
                    adjacent_to_air: true,
                    air_check_roll: 0.0,
                },
                super::super::OrePlacementContext {
                    pos: BlockPos { x: 8, y: 34, z: 8 },
                    current_block: "minecraft:dirt",
                    adjacent_to_air: false,
                    air_check_roll: 1.0,
                },
            ],
        );
        assert_eq!(
            ore_plan,
            vec![super::super::OrePlacementBlock {
                pos: BlockPos { x: 8, y: 32, z: 8 },
                state: "minecraft:iron_ore",
            }]
        );
        assert_eq!(
            super::super::aquatic_feature_offset(
                BlockPos {
                    x: 20,
                    y: 60,
                    z: 30
                },
                (7, 3),
                (1, 6)
            ),
            (24, 25)
        );
        assert_eq!(
            super::super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:water",
                true,
                0.7,
                0.1,
            ),
            vec![
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 62, z: 1 },
                    state: "minecraft:tall_seagrass",
                },
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 1, y: 63, z: 1 },
                    state: "minecraft:tall_seagrass[half=upper]",
                },
            ]
        );
        assert_eq!(
            super::super::seagrass_placement_plan(
                BlockPos { x: 1, y: 62, z: 1 },
                "minecraft:water",
                "minecraft:air",
                true,
                0.7,
                0.1,
            ),
            Vec::new()
        );
        assert_eq!(
            super::super::sea_pickle_placement_plan(
                BlockPos { x: 2, y: 61, z: 2 },
                "minecraft:water",
                true,
                2,
            ),
            Some(super::super::AquaticPlacementBlock {
                pos: BlockPos { x: 2, y: 61, z: 2 },
                state: "minecraft:sea_pickle[pickles=3]",
            })
        );
        assert_eq!(
            super::super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, true, true],
                &[true, true, true],
                1,
                &[2],
                false,
            ),
            vec![
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 50, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 51, z: 3 },
                    state: "minecraft:kelp_plant",
                },
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 3, y: 52, z: 3 },
                    state: "minecraft:kelp[age=22]",
                },
            ]
        );
        assert_eq!(
            super::super::kelp_placement_plan(
                BlockPos { x: 3, y: 50, z: 3 },
                &[true, true, false],
                &[true, true],
                5,
                &[0],
                false,
            ),
            vec![super::super::AquaticPlacementBlock {
                pos: BlockPos { x: 3, y: 50, z: 3 },
                state: "minecraft:kelp[age=20]",
            }]
        );
        assert_eq!(
            super::super::coral_block_placement_plan(
                BlockPos { x: 4, y: 55, z: 4 },
                "minecraft:water",
                "minecraft:water",
                "minecraft:brain_coral_block",
                0.9,
                0.01,
                1,
                &[(super::super::HorizontalDirection::East, 0.1, true)],
            ),
            vec![
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 55, z: 4 },
                    state: "minecraft:brain_coral_block",
                },
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 4, y: 56, z: 4 },
                    state: "minecraft:sea_pickle[pickles=2]",
                },
                super::super::AquaticPlacementBlock {
                    pos: BlockPos { x: 5, y: 55, z: 4 },
                    state: "minecraft:tube_coral_wall_fan[facing=east]",
                },
            ]
        );
        assert!(super::super::coral_block_placement_plan(
            BlockPos { x: 4, y: 55, z: 4 },
            "minecraft:stone",
            "minecraft:water",
            "minecraft:brain_coral_block",
            0.0,
            0.0,
            0,
            &[],
        )
        .is_empty());
        let coral_tree = super::super::coral_tree_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            1,
            &[
                super::super::HorizontalDirection::North,
                super::super::HorizontalDirection::East,
            ],
            &[0, 1],
            &[1.0; 10],
        );
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_tree.contains(&BlockPos { x: 0, y: 62, z: -1 }));
        assert!(coral_tree.contains(&BlockPos { x: 1, y: 62, z: 0 }));
        let coral_mushroom = super::super::coral_mushroom_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            0,
            0,
            0,
            0,
            &[1.0; 128],
        );
        assert!(coral_mushroom.contains(&BlockPos { x: 1, y: 59, z: 1 }));
        assert!(!coral_mushroom.contains(&BlockPos { x: 0, y: 59, z: 0 }));
        let coral_claw = super::super::coral_claw_positions(
            BlockPos { x: 0, y: 60, z: 0 },
            super::super::HorizontalDirection::North,
            &[
                super::super::HorizontalDirection::North,
                super::super::HorizontalDirection::East,
            ],
            &[0, 0],
            &[0, 0],
            &[1.0; 10],
        );
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: 0 }));
        assert!(coral_claw.contains(&BlockPos { x: 0, y: 60, z: -1 }));
        assert!(coral_claw.contains(&BlockPos { x: 1, y: 61, z: 0 }));
        let vegetation_config = super::super::VegetationPatchConfigurationModel {
            replaceable: &["minecraft:dirt", "minecraft:grass_block"],
            ground_state: BlockStateProviderModel::Simple("minecraft:moss_block"),
            vegetation_feature: "minecraft:patch_grass",
            surface: CaveSurface::Floor,
            depth_min: 1,
            depth_max: 2,
            extra_bottom_block_chance: 0.5,
            vertical_range: 5,
            vegetation_chance: 0.75,
            xz_radius_min: 1,
            xz_radius_max: 2,
            extra_edge_column_chance: 0.25,
        };
        assert_eq!(super::super::vegetation_patch_radius(1, 2, 1), 3);
        assert!(!super::super::vegetation_patch_should_try_column(
            3, 3, 3, 3, 1.0, 0.0
        ));
        assert!(!super::super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.5
        ));
        assert!(super::super::vegetation_patch_should_try_column(
            3, 0, 3, 3, 0.25, 0.25
        ));
        assert_eq!(super::super::vegetation_patch_depth(1, 2, 0, 0.5, 0.25), 2);
        assert_eq!(
            super::super::vegetation_patch_place_ground(
                &vegetation_config,
                BlockPos { x: 5, y: 63, z: 5 },
                &["minecraft:dirt", "minecraft:stone"],
                3,
                0,
            ),
            Some(vec![super::super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }])
        );
        let vegetation_plan = super::super::vegetation_patch_plan(
            &vegetation_config,
            &[super::super::VegetationPatchGroundColumn {
                surface_pos: BlockPos { x: 5, y: 64, z: 5 },
                ground_start: BlockPos { x: 5, y: 63, z: 5 },
                depth: 1,
            }],
            &[&["minecraft:dirt"]],
            &[0.25],
        );
        assert_eq!(
            vegetation_plan.ground,
            vec![super::super::VegetationPatchBlock {
                pos: BlockPos { x: 5, y: 63, z: 5 },
                state: "minecraft:moss_block",
            }]
        );
        assert_eq!(
            vegetation_plan.vegetation_origins,
            vec![BlockPos { x: 5, y: 65, z: 5 }]
        );
        let lake_config = super::super::LakeConfigurationModel {
            fluid: BlockStateProviderModel::Simple("minecraft:water"),
            barrier: BlockStateProviderModel::Simple("minecraft:stone"),
        };
        let mut lake_grid = vec![false; 2048];
        lake_grid[super::super::lake_grid_index(8, 3, 8)] = true;
        assert!(super::super::lake_is_boundary(&lake_grid, 8, 4, 8));
        assert_eq!(
            super::super::lake_grid_index(8, 3, 8),
            ((8 * 16 + 8) * 8 + 3) as usize
        );
        let lake_boundary = [
            super::super::LakeBoundaryBlock {
                x: 8,
                y: 4,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: true,
            },
            super::super::LakeBoundaryBlock {
                x: 8,
                y: 2,
                z: 8,
                state: "minecraft:stone",
                solid: true,
                liquid: false,
                cannot_replace: false,
                should_freeze: false,
            },
        ];
        assert!(super::super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &lake_boundary,
            "minecraft:water"
        ));
        let invalid_lake_boundary = [super::super::LakeBoundaryBlock {
            x: 8,
            y: 4,
            z: 8,
            state: "minecraft:water",
            solid: false,
            liquid: true,
            cannot_replace: false,
            should_freeze: false,
        }];
        assert!(!super::super::lake_can_place(
            -64,
            70,
            &lake_grid,
            &invalid_lake_boundary,
            "minecraft:water"
        ));
        let lake_plan = super::super::lake_placement_plan(
            BlockPos { x: 0, y: 60, z: 0 },
            &lake_config,
            &lake_grid,
            &lake_boundary,
            &[1; 2048],
            true,
        )
        .unwrap();
        assert!(lake_plan.contains(&super::super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 63, z: 8 },
            state: "minecraft:water",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        assert!(lake_plan.contains(&super::super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:stone",
            schedule_tick: false,
            mark_above_for_post_processing: true,
        }));
        assert!(lake_plan.contains(&super::super::LakePlacementBlock {
            pos: BlockPos { x: 8, y: 64, z: 8 },
            state: "minecraft:ice",
            schedule_tick: false,
            mark_above_for_post_processing: false,
        }));
        let fossil_config = super::super::FossilFeatureConfigurationModel {
            fossil_structures: vec!["minecraft:fossil/spine_1", "minecraft:fossil/skull_1"],
            overlay_structures: vec![
                "minecraft:fossil/spine_1_coal",
                "minecraft:fossil/skull_1_coal",
            ],
            fossil_processors: "minecraft:fossil_rot",
            overlay_processors: "minecraft:fossil_coal",
            max_empty_corners_allowed: 4,
        };
        assert_eq!(super::super::validate_fossil_config(&fossil_config), Ok(()));
        assert_eq!(
            super::super::validate_fossil_config(&super::super::FossilFeatureConfigurationModel {
                fossil_structures: Vec::new(),
                overlay_structures: Vec::new(),
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
                max_empty_corners_allowed: 4,
            }),
            Err("Fossil structure lists need at least one entry")
        );
        assert_eq!(
            super::super::fossil_rotation(3),
            super::super::StructureRotation::Counterclockwise90
        );
        assert_eq!(super::super::fossil_target_y(50, -64, 9), 26);
        assert_eq!(
            super::super::fossil_low_corner(
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8
            ),
            BlockPos {
                x: 94,
                y: 40,
                z: 196
            }
        );
        assert_eq!(
            super::super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                4,
            ),
            Some(super::super::FossilPlacementPlan {
                fossil_structure: "minecraft:fossil/skull_1",
                overlay_structure: "minecraft:fossil/skull_1_coal",
                rotation: super::super::StructureRotation::Clockwise90,
                target_pos: BlockPos {
                    x: 94,
                    y: 35,
                    z: 196
                },
                fossil_processors: "minecraft:fossil_rot",
                overlay_processors: "minecraft:fossil_coal",
            })
        );
        assert_eq!(
            super::super::fossil_placement_plan(
                &fossil_config,
                BlockPos {
                    x: 100,
                    y: 40,
                    z: 200
                },
                12,
                8,
                50,
                -64,
                1,
                1,
                0,
                5,
            ),
            None
        );
}
