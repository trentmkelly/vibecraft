use super::*;

#[test]
    fn height_provider_types_match_vanilla_registry_order() {
        assert_eq!(
            HEIGHT_PROVIDER_TYPES
                .iter()
                .map(|provider_type| provider_type.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:constant",
                "minecraft:uniform",
                "minecraft:biased_to_bottom",
                "minecraft:very_biased_to_bottom",
                "minecraft:trapezoid",
                "minecraft:weighted_list",
            ]
        );
        assert!(super::super::height_provider_type("constant").is_some());
        assert!(super::super::height_provider_type("minecraft:weighted_list").is_some());
        assert!(super::super::height_provider_type("clamped").is_none());
    }

    #[test]
    fn height_provider_sampling_envelopes_follow_vanilla_edge_cases() {
        let context = WorldGenerationHeightContext {
            min_y: -64,
            height: 384,
        };
        assert_eq!(VerticalAnchor::Absolute(12).resolve_y(context), 12);
        assert_eq!(VerticalAnchor::AboveBottom(8).resolve_y(context), -56);
        assert_eq!(VerticalAnchor::BelowTop(1).resolve_y(context), 318);

        let constant = HeightProvider::Constant {
            value: VerticalAnchor::AboveBottom(8),
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(constant, context),
            (-56, -56)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(constant, context, 99, 0, 0),
            -56
        );

        let uniform = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::Absolute(10),
            max_inclusive: VerticalAnchor::Absolute(14),
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(uniform, context),
            (10, 14)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(uniform, context, 7, 0, 0),
            12
        );

        let empty_uniform = HeightProvider::Uniform {
            min_inclusive: VerticalAnchor::Absolute(20),
            max_inclusive: VerticalAnchor::Absolute(10),
        };
        assert_eq!(
            super::super::height_provider_sample_with_rolls(empty_uniform, context, 0, 0, 0),
            20
        );

        let biased = HeightProvider::BiasedToBottom {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            inner: 2,
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(biased, context),
            (0, 9)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(biased, context, 4, 6, 0),
            0
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(biased, context, 8, 9, 0),
            9
        );

        let very_biased = HeightProvider::VeryBiasedToBottom {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            inner: 2,
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(very_biased, context),
            (0, 9)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(very_biased, context, 8, 9, 9),
            9
        );

        let trapezoid = HeightProvider::Trapezoid {
            min_inclusive: VerticalAnchor::Absolute(0),
            max_inclusive: VerticalAnchor::Absolute(10),
            plateau: 2,
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(trapezoid, context),
            (0, 10)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(trapezoid, context, 6, 4, 0),
            10
        );

        static DISTRIBUTION: &[WeightedHeightProvider] = &[
            WeightedHeightProvider {
                weight: 2,
                provider: HeightProvider::Constant {
                    value: VerticalAnchor::Absolute(4),
                },
            },
            WeightedHeightProvider {
                weight: 3,
                provider: HeightProvider::Uniform {
                    min_inclusive: VerticalAnchor::Absolute(20),
                    max_inclusive: VerticalAnchor::Absolute(22),
                },
            },
        ];
        let weighted = HeightProvider::WeightedList {
            distribution: DISTRIBUTION,
        };
        assert_eq!(
            super::super::height_provider_sample_bounds(weighted, context),
            (4, 22)
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(weighted, context, 1, 99, 0),
            4
        );
        assert_eq!(
            super::super::height_provider_sample_with_rolls(weighted, context, 4, 5, 0),
            22
        );
    }

    #[test]
    fn generation_decoration_steps_match_vanilla_serialized_order() {
        assert_eq!(
            GenerationDecorationStep::VALUES
                .iter()
                .map(|step| step.serialized_name())
                .collect::<Vec<_>>(),
            vec![
                "raw_generation",
                "lakes",
                "local_modifications",
                "underground_structures",
                "surface_structures",
                "strongholds",
                "underground_ores",
                "underground_decoration",
                "fluid_springs",
                "vegetal_decoration",
                "top_layer_modification",
            ]
        );
    }

    #[test]
    fn block_predicate_types_match_vanilla_registry_order() {
        assert_eq!(
            BLOCK_PREDICATE_TYPES
                .iter()
                .map(|predicate_type| predicate_type.id)
                .collect::<Vec<_>>(),
            vec![
                "minecraft:matching_blocks",
                "minecraft:matching_block_tag",
                "minecraft:matching_fluids",
                "minecraft:has_sturdy_face",
                "minecraft:solid",
                "minecraft:replaceable",
                "minecraft:would_survive",
                "minecraft:inside_world_bounds",
                "minecraft:any_of",
                "minecraft:all_of",
                "minecraft:not",
                "minecraft:true",
                "minecraft:unobstructed",
            ]
        );
        assert!(super::super::block_predicate_type("matching_blocks").is_some());
        assert!(super::super::block_predicate_type("minecraft:not").is_some());
        assert!(super::super::block_predicate_type("height_range").is_none());
    }

    #[test]
    fn block_predicate_core_evaluators_follow_vanilla_boolean_rules() {
        let grass = BlockPredicateContext {
            min_y: -64,
            height: 384,
            block: "minecraft:grass_block",
            fluid: "minecraft:empty",
            solid: true,
            replaceable: false,
            unobstructed: true,
        };
        assert!(super::super::block_predicate_test(
            BlockPredicate::True,
            grass,
            320
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::MatchingBlocks {
                blocks: &["minecraft:dirt", "minecraft:grass_block"],
            },
            grass,
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::MatchingBlockTag {
                tag: "minecraft:logs",
            },
            BlockPredicateContext {
                block: "minecraft:stripped_oak_log",
                ..grass
            },
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::MatchingBlockTag {
                tag: "minecraft:air",
            },
            BlockPredicateContext {
                block: "minecraft:cave_air",
                ..grass
            },
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::MatchingBlockTag {
                tag: "minecraft:leaves",
            },
            BlockPredicateContext {
                block: "minecraft:azalea_leaves",
                ..grass
            },
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::MatchingBlockTag {
                tag: "minecraft:replaceable",
            },
            BlockPredicateContext {
                block: "minecraft:void_air",
                ..grass
            },
            64
        ));
        assert!(!super::super::block_predicate_test(
            BlockPredicate::MatchingBlockTag {
                tag: "minecraft:leaves",
            },
            grass,
            64
        ));
        assert!(!super::super::block_predicate_test(
            BlockPredicate::MatchingFluids {
                fluids: &["minecraft:water"],
            },
            grass,
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::Solid,
            grass,
            64
        ));
        assert!(!super::super::block_predicate_test(
            BlockPredicate::Replaceable,
            grass,
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::InsideWorldBounds { offset_y: -1 },
            grass,
            320
        ));
        assert!(!super::super::block_predicate_test(
            BlockPredicate::InsideWorldBounds { offset_y: 0 },
            grass,
            320
        ));

        static SOLID: BlockPredicate = BlockPredicate::Solid;
        static REPLACEABLE: BlockPredicate = BlockPredicate::Replaceable;
        static UNOBSTRUCTED: BlockPredicate = BlockPredicate::Unobstructed;
        static ALL_PREDICATES: &[BlockPredicate] = &[SOLID, UNOBSTRUCTED];
        static ANY_PREDICATES: &[BlockPredicate] = &[REPLACEABLE, UNOBSTRUCTED];
        assert!(super::super::block_predicate_test(
            BlockPredicate::AllOf {
                predicates: ALL_PREDICATES,
            },
            grass,
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::AnyOf {
                predicates: ANY_PREDICATES,
            },
            grass,
            64
        ));
        assert!(super::super::block_predicate_test(
            BlockPredicate::Not {
                predicate: &REPLACEABLE,
            },
            grass,
            64
        ));

        let water_below = BlockPredicateContext {
            block: "minecraft:water",
            fluid: "minecraft:water",
            solid: false,
            replaceable: true,
            unobstructed: false,
            ..grass
        };
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::SolidAt { offset_y: -1 },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::MatchingFluidsAt {
                offset_y: -1,
                fluids: &["minecraft:water"],
            },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::ReplaceableAt { offset_y: -1 },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::WouldSurvive {
                offset_y: -1,
                state: "minecraft:oak_sapling",
                survives: true,
            },
            grass,
            water_below,
            64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::WouldSurvive {
                offset_y: -1,
                state: "minecraft:oak_sapling",
                survives: true,
            },
            grass,
            water_below,
            -64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::WouldSurvive {
                offset_y: -1,
                state: "minecraft:oak_sapling",
                survives: false,
            },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::HasSturdyFace {
                offset_y: -1,
                direction: "up",
                sturdy: true,
            },
            grass,
            water_below,
            64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::HasSturdyFace {
                offset_y: -1,
                direction: "up",
                sturdy: true,
            },
            grass,
            water_below,
            -64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::HasSturdyFace {
                offset_y: -1,
                direction: "north",
                sturdy: false,
            },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::MatchingBlocksAt {
                offset_y: -1,
                blocks: &["minecraft:water"],
            },
            grass,
            water_below,
            64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::MatchingFluidsAt {
                offset_y: -1,
                fluids: &["minecraft:water"],
            },
            grass,
            water_below,
            -64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::MatchingBlocksAt {
                offset_y: -1,
                blocks: &["minecraft:water"],
            },
            grass,
            water_below,
            -64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::SolidAt { offset_y: -1 },
            grass,
            water_below,
            -64
        ));
        assert!(!super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::ReplaceableAt { offset_y: -1 },
            grass,
            water_below,
            -64
        ));

        static OFFSET_WATER: BlockPredicate = BlockPredicate::MatchingFluidsAt {
            offset_y: -1,
            fluids: &["minecraft:water"],
        };
        static OFFSET_SOLID: BlockPredicate = BlockPredicate::SolidAt { offset_y: -1 };
        static OFFSET_ANY_PREDICATES: &[BlockPredicate] = &[OFFSET_SOLID, OFFSET_WATER];
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::AnyOf {
                predicates: OFFSET_ANY_PREDICATES,
            },
            grass,
            water_below,
            64
        ));
        assert!(super::super::block_predicate_test_with_vertical_context(
            BlockPredicate::Not {
                predicate: &OFFSET_SOLID,
            },
            grass,
            water_below,
            64
        ));
    }

    #[test]
    fn placement_modifier_registry_and_core_positions_follow_vanilla_rules() {
        assert!(super::super::placement_modifier_type("rarity_filter").is_some());
        assert!(super::super::placement_modifier_type("minecraft:fixed_placement").is_some());
        assert!(super::super::placement_modifier_type("matching_blocks").is_none());

        let origin = BlockPos {
            x: 32,
            y: 70,
            z: -16,
        };
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::RarityFilter { chance: 4 },
                origin,
                8,
                0,
                0
            ),
            vec![origin]
        );
        assert!(super::super::placement_modifier_positions(
            PlacementModifier::RarityFilter { chance: 4 },
            origin,
            9,
            0,
            0
        )
        .is_empty());
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::Count { count: 3 },
                origin,
                0,
                0,
                0
            ),
            vec![origin, origin, origin]
        );
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::NoiseBasedCount {
                    noise_to_count_ratio: 4,
                    noise_factor: 200.0,
                    noise_offset: 0.25,
                    sampled_noise: 0.26,
                },
                origin,
                0,
                0,
                0,
            ),
            vec![origin, origin, origin]
        );
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::NoiseThresholdCount {
                    noise_level: 0.4,
                    below_noise: 2,
                    above_noise: 5,
                    sampled_noise: 0.4,
                },
                origin,
                0,
                0,
                0,
            ),
            vec![origin; 5]
        );
        assert_eq!(
            super::super::placement_modifier_positions(PlacementModifier::InSquare, origin, 19, 31, 0),
            vec![BlockPos {
                x: 35,
                y: 70,
                z: -1,
            }]
        );
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::RandomOffset {
                    xz_spread: 4,
                    y_spread: 2,
                },
                origin,
                8,
                4,
                0
            ),
            vec![BlockPos {
                x: 36,
                y: 72,
                z: -20,
            }]
        );

        static FIXED_POSITIONS: &[BlockPos] = &[
            BlockPos {
                x: 34,
                y: 70,
                z: -8,
            },
            BlockPos {
                x: 48,
                y: 70,
                z: -8,
            },
            BlockPos {
                x: 35,
                y: 71,
                z: -1,
            },
        ];
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::Fixed {
                    positions: FIXED_POSITIONS,
                },
                origin,
                0,
                0,
                0
            ),
            vec![
                BlockPos {
                    x: 34,
                    y: 70,
                    z: -8
                },
                BlockPos {
                    x: 35,
                    y: 71,
                    z: -1
                },
            ]
        );
        static EVERY_LAYER_POSITIONS: &[BlockPos] = &[
            BlockPos {
                x: 33,
                y: 70,
                z: -15,
            },
            BlockPos {
                x: 34,
                y: 80,
                z: -15,
            },
        ];
        assert_eq!(
            super::super::placement_modifier_positions(
                PlacementModifier::CountOnEveryLayer {
                    positions: EVERY_LAYER_POSITIONS,
                },
                origin,
                0,
                0,
                0,
            ),
            EVERY_LAYER_POSITIONS.to_vec()
        );

        let placement_context = PlacementContextModel {
            min_y: -64,
            world_surface_height: 81,
            ocean_floor_height: 63,
            biome_allows_feature: true,
            block_predicate: BlockPredicateContext {
                min_y: -64,
                height: 384,
                block: "minecraft:grass_block",
                fluid: "minecraft:empty",
                solid: true,
                replaceable: false,
                unobstructed: true,
            },
        };
        assert_eq!(
            super::super::placement_modifier_positions_with_context(
                PlacementModifier::Heightmap {
                    heightmap: HeightmapKind::WorldSurface,
                },
                origin,
                placement_context,
                0,
                0,
                0,
            ),
            vec![BlockPos {
                x: 32,
                y: 81,
                z: -16,
            }]
        );
        assert_eq!(
            super::super::placement_modifier_positions_with_context(
                PlacementModifier::HeightRange {
                    height: HeightProvider::Uniform {
                        min_inclusive: VerticalAnchor::Absolute(64),
                        max_inclusive: VerticalAnchor::Absolute(68),
                    },
                },
                origin,
                placement_context,
                3,
                0,
                0,
            ),
            vec![BlockPos {
                x: 32,
                y: 67,
                z: -16,
            }]
        );
        assert_eq!(
            super::super::placement_modifier_positions_with_context(
                PlacementModifier::SurfaceRelativeThresholdFilter {
                    heightmap: HeightmapKind::WorldSurface,
                    min_inclusive: -16,
                    max_inclusive: 0,
                },
                BlockPos {
                    x: 32,
                    y: 70,
                    z: -16,
                },
                placement_context,
                0,
                0,
                0,
            ),
            vec![origin]
        );
        static SCAN_STATES: &[BlockPredicateContext] = &[
            BlockPredicateContext {
                min_y: -64,
                height: 384,
                block: "minecraft:air",
                fluid: "minecraft:empty",
                solid: false,
                replaceable: true,
                unobstructed: true,
            },
            BlockPredicateContext {
                min_y: -64,
                height: 384,
                block: "minecraft:air",
                fluid: "minecraft:empty",
                solid: false,
                replaceable: true,
                unobstructed: true,
            },
            BlockPredicateContext {
                min_y: -64,
                height: 384,
                block: "minecraft:grass_block",
                fluid: "minecraft:empty",
                solid: true,
                replaceable: false,
                unobstructed: true,
            },
        ];
        static SCAN_ALLOWED_PREDICATES: &[BlockPredicate] =
            &[BlockPredicate::Replaceable, BlockPredicate::Solid];
        assert_eq!(
            super::super::placement_modifier_positions_with_context(
                PlacementModifier::EnvironmentScan {
                    direction_y: -1,
                    target_condition: BlockPredicate::Solid,
                    allowed_search_condition: BlockPredicate::AnyOf {
                        predicates: SCAN_ALLOWED_PREDICATES,
                    },
                    max_steps: 3,
                    states: SCAN_STATES,
                },
                BlockPos {
                    x: 32,
                    y: 72,
                    z: -16,
                },
                placement_context,
                0,
                0,
                0,
            ),
            vec![BlockPos {
                x: 32,
                y: 70,
                z: -16,
            }]
        );
        assert!(super::super::placement_modifier_positions_with_context(
            PlacementModifier::SurfaceWaterDepthFilter { max_water_depth: 8 },
            origin,
            placement_context,
            0,
            0,
            0,
        )
        .is_empty());
        assert_eq!(
            super::super::placement_modifier_positions_with_context(
                PlacementModifier::BlockPredicateFilter {
                    predicate: BlockPredicate::Solid,
                },
                origin,
                placement_context,
                0,
                0,
                0,
            ),
            vec![origin]
        );
        assert_eq!(
            super::super::placed_feature_positions(
                &[
                    PlacementModifier::BiomeFilter,
                    PlacementModifier::Count { count: 2 },
                    PlacementModifier::InSquare,
                    PlacementModifier::Heightmap {
                        heightmap: HeightmapKind::WorldSurface,
                    },
                ],
                BlockPos {
                    x: 32,
                    y: 0,
                    z: -16
                },
                placement_context,
                &[(0, 0, 0), (0, 0, 0), (3, 4, 0), (0, 0, 0)],
            ),
            vec![
                BlockPos {
                    x: 35,
                    y: 81,
                    z: -12,
                },
                BlockPos {
                    x: 35,
                    y: 81,
                    z: -12,
                },
            ]
        );
        assert!(super::super::placed_feature_positions(
            &[PlacementModifier::BiomeFilter],
            origin,
            PlacementContextModel {
                biome_allows_feature: false,
                ..placement_context
            },
            &[(0, 0, 0)],
        )
        .is_empty());

        assert_eq!(
            super::super::placed_feature_invocations(
                "minecraft:oak",
                &[
                    PlacementModifier::BiomeFilter,
                    PlacementModifier::Count { count: 2 },
                    PlacementModifier::InSquare,
                    PlacementModifier::Heightmap {
                        heightmap: HeightmapKind::WorldSurface,
                    },
                ],
                BlockPos {
                    x: 32,
                    y: 0,
                    z: -16
                },
                placement_context,
                &[(0, 0, 0), (0, 0, 0), (3, 4, 0), (0, 0, 0)],
            )
            .unwrap(),
            vec![
                super::super::PlacedFeatureInvocation {
                    feature: "minecraft:oak",
                    source: super::super::ConfiguredFeatureSource::Tree,
                    pos: BlockPos {
                        x: 35,
                        y: 81,
                        z: -12,
                    },
                },
                super::super::PlacedFeatureInvocation {
                    feature: "minecraft:oak",
                    source: super::super::ConfiguredFeatureSource::Tree,
                    pos: BlockPos {
                        x: 35,
                        y: 81,
                        z: -12,
                    },
                },
            ]
        );
        assert_eq!(
            super::super::placed_feature_invocations(
                "minecraft:not_a_feature",
                &[],
                origin,
                placement_context,
                &[],
            )
            .unwrap_err(),
            "unknown configured feature minecraft:not_a_feature"
        );
    }

