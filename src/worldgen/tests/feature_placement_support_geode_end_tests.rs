use super::super::*;

pub(super) fn assert_geode_nether_and_end_support() {
        let geode_config = super::super::GeodeConfigurationModel {
            filling_provider: BlockStateProviderModel::Simple("minecraft:air"),
            inner_layer_provider: BlockStateProviderModel::Simple("minecraft:amethyst_block"),
            alternate_inner_layer_provider: BlockStateProviderModel::Simple(
                "minecraft:budding_amethyst",
            ),
            middle_layer_provider: BlockStateProviderModel::Simple("minecraft:calcite"),
            outer_layer_provider: BlockStateProviderModel::Simple("minecraft:smooth_basalt"),
            inner_placements: &[
                "minecraft:small_amethyst_bud",
                "minecraft:medium_amethyst_bud",
                "minecraft:large_amethyst_bud",
                "minecraft:amethyst_cluster",
            ],
            cannot_replace: &["minecraft:bedrock"],
            invalid_blocks: &["minecraft:water", "minecraft:lava"],
            layers: super::super::GeodeLayerSettingsModel {
                filling: 1.7,
                inner_layer: 2.2,
                middle_layer: 3.2,
                outer_layer: 4.2,
            },
            crack: super::super::GeodeCrackSettingsModel {
                generate_crack_chance: 0.95,
                base_crack_size: 2.0,
                crack_point_offset: 2,
            },
            use_potential_placements_chance: 0.35,
            use_alternate_layer0_chance: 0.083,
            placements_require_layer0_alternate: true,
            outer_wall_distance_max: 6,
            invalid_blocks_threshold: 1,
        };
        assert_eq!(super::super::validate_geode_config(&geode_config), Ok(()));
        assert!(super::super::geode_can_place(
            &geode_config,
            &["minecraft:stone", "minecraft:water"]
        ));
        assert!(!super::super::geode_can_place(
            &geode_config,
            &["minecraft:air", "minecraft:water"]
        ));
        let geode_thresholds = super::super::geode_layer_thresholds(
            geode_config.layers,
            geode_config.crack,
            4,
            geode_config.outer_wall_distance_max,
            0.0,
        );
        assert!(geode_thresholds.inner_air > geode_thresholds.innermost_block_layer);
        assert!(geode_thresholds.innermost_block_layer > geode_thresholds.inner_crust);
        assert!(geode_thresholds.inner_crust > geode_thresholds.outer_crust);
        assert!(super::super::geode_should_generate_crack(geode_config.crack, 0.94));
        let crack_points = super::super::geode_crack_points(BlockPos { x: 0, y: 0, z: 0 }, 4, 2);
        assert_eq!(crack_points[0], BlockPos { x: 9, y: 7, z: 9 });
        let distribution_points = [super::super::GeodeDistributionPoint {
            pos: BlockPos { x: 0, y: 0, z: 0 },
            offset: 1,
        }];
        assert!(
            super::super::geode_shell_density(BlockPos { x: 0, y: 0, z: 0 }, &distribution_points, 0.0)
                > super::super::geode_shell_density(
                    BlockPos { x: 16, y: 0, z: 0 },
                    &distribution_points,
                    0.0
                )
        );
        assert_eq!(
            super::super::geode_layer_for_density(
                geode_thresholds.inner_air + 0.1,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::super::GeodeLayer::Filling)
        );
        assert_eq!(
            super::super::geode_layer_for_density(
                (geode_thresholds.innermost_block_layer + geode_thresholds.inner_air) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.0,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::super::GeodeLayer::AlternateInner)
        );
        assert_eq!(
            super::super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                0.0,
                geode_thresholds,
                false,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::super::GeodeLayer::Outer)
        );
        assert_eq!(
            super::super::geode_layer_for_density(
                (geode_thresholds.outer_crust + geode_thresholds.inner_crust) / 2.0,
                geode_thresholds.crack_size,
                geode_thresholds,
                true,
                0.5,
                geode_config.use_alternate_layer0_chance,
            ),
            Some(super::super::GeodeLayer::CrackAir)
        );
        assert_eq!(
            super::super::geode_placement_block(
                &geode_config,
                BlockPos { x: 1, y: 2, z: 3 },
                super::super::GeodeLayer::AlternateInner,
                0,
                0.0,
            ),
            Some(super::super::GeodePlacementBlock {
                pos: BlockPos { x: 1, y: 2, z: 3 },
                state: "minecraft:budding_amethyst",
                layer: super::super::GeodeLayer::AlternateInner,
                potential_crystal_source: true,
            })
        );
        assert_eq!(
            super::super::geode_inner_placement(&geode_config, 3),
            Some("minecraft:amethyst_cluster")
        );
        let iceberg_shape = super::super::iceberg_shape_model(0.8, 0.25, 0, 2, 0.8, 5, 0.0, 0, 10, 6, 0);
        assert_eq!(iceberg_shape.shape_ellipse_a, 11);
        assert_eq!(iceberg_shape.shape_ellipse_c, 5);
        assert!(iceberg_shape.is_ellipse);
        assert_eq!(iceberg_shape.over_water_height, 11);
        assert_eq!(iceberg_shape.under_water_height, 18);
        assert_eq!(iceberg_shape.width, 11);
        assert_eq!(super::super::iceberg_ellipse_c(9, 11, 5), 3);
        assert!(
            super::super::iceberg_signed_distance_circle(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 5, 0.5,)
                < 0.0
        );
        assert!(
            super::super::iceberg_signed_distance_ellipse(0, 0, BlockPos { x: 0, y: 0, z: 0 }, 11, 5, 0.0,)
                < 0.0
        );
        assert_eq!(super::super::iceberg_height_radius_ellipse(0, 11, 11), 6);
        assert_eq!(super::super::iceberg_height_radius_steep(1, 11, 11, 0.0), 5);
        assert!(super::super::iceberg_height_radius_round(0, 11, 11, 0.5, 0, 0) > 0);
        assert_eq!(
            super::super::iceberg_set_block_action("minecraft:air", 1, 11, true, true, 0, 0.1,),
            super::super::IcebergBlockAction::SnowBlock
        );
        assert_eq!(
            super::super::iceberg_set_block_action("minecraft:stone", 1, 11, true, true, 0, 0.1,),
            super::super::IcebergBlockAction::Keep
        );
        assert!(super::super::iceberg_should_skip_surface_noise(-0.25, true, 0.95));
        assert_eq!(
            super::super::iceberg_carve_action("minecraft:packed_ice", true),
            super::super::IcebergBlockAction::Water
        );
        assert_eq!(
            super::super::iceberg_carve_action("minecraft:blue_ice", false),
            super::super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::super::iceberg_smooth_action("minecraft:packed_ice", false, 3),
            super::super::IcebergBlockAction::Air
        );
        assert_eq!(
            super::super::iceberg_smooth_action("minecraft:snow", true, 0),
            super::super::IcebergBlockAction::Air
        );
        assert!(super::super::blue_ice_can_start(
            62,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::super::blue_ice_can_start(
            63,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::super::blue_ice_can_start(
            62,
            63,
            "minecraft:air",
            "minecraft:stone",
            &["minecraft:packed_ice"]
        ));
        assert!(!super::super::blue_ice_can_start(
            62,
            63,
            "minecraft:water",
            "minecraft:air",
            &["minecraft:ice"]
        ));
        assert_eq!(super::super::blue_ice_xz_diff(1), 3);
        assert_eq!(super::super::blue_ice_xz_diff(-5), 1);
        assert_eq!(super::super::blue_ice_xz_diff(-6), 0);
        assert_eq!(
            super::super::blue_ice_spread_candidate(
                BlockPos {
                    x: 10,
                    y: 64,
                    z: 10
                },
                -1,
                2,
                0,
                1,
                0
            ),
            Some(BlockPos {
                x: 12,
                y: 63,
                z: 11
            })
        );
        assert_eq!(
            super::super::blue_ice_spread_candidate(
                BlockPos {
                    x: 10,
                    y: 64,
                    z: 10
                },
                -6,
                0,
                0,
                0,
                0
            ),
            None
        );
        assert!(super::super::blue_ice_spread_can_place(
            "minecraft:water",
            &["minecraft:blue_ice"]
        ));
        assert!(super::super::blue_ice_spread_can_place(
            "minecraft:ice",
            &["minecraft:blue_ice"]
        ));
        assert!(!super::super::blue_ice_spread_can_place(
            "minecraft:stone",
            &["minecraft:blue_ice"]
        ));
        assert!(!super::super::blue_ice_spread_can_place(
            "minecraft:water",
            &["minecraft:packed_ice"]
        ));
        let random_feature = super::super::RandomFeatureConfigurationModel {
            features: vec![
                super::super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_tulip",
                    chance: 0.2,
                },
                super::super::WeightedPlacedFeatureModel {
                    feature: "minecraft:patch_grass",
                    chance: 0.5,
                },
            ],
            default_feature: "minecraft:flower_default",
        };
        assert_eq!(
            super::super::validate_weighted_placed_feature(random_feature.features[0]),
            Ok(random_feature.features[0])
        );
        assert_eq!(
            super::super::validate_weighted_placed_feature(super::super::WeightedPlacedFeatureModel {
                feature: "minecraft:bad",
                chance: 1.1,
            }),
            Err("weighted placed feature chance must be in 0.0..=1.0")
        );
        assert_eq!(
            super::super::random_selector_feature(&random_feature, &[0.3, 0.25]),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::super::random_selector_feature(&random_feature, &[0.3, 0.6]),
            Some("minecraft:flower_default")
        );
        let simple_random_feature = super::super::SimpleRandomFeatureConfigurationModel {
            features: vec![
                "minecraft:flower_plain",
                "minecraft:patch_grass",
                "minecraft:patch_sunflower",
            ],
        };
        assert_eq!(
            super::super::simple_random_selector_feature(&simple_random_feature, 4),
            Some("minecraft:patch_grass")
        );
        assert_eq!(
            super::super::simple_random_selector_feature(
                &super::super::SimpleRandomFeatureConfigurationModel { features: vec![] },
                0,
            ),
            None
        );
        let random_boolean_feature = super::super::RandomBooleanFeatureConfigurationModel {
            feature_true: "minecraft:flower_cherry",
            feature_false: "minecraft:patch_grass",
        };
        assert_eq!(
            super::super::random_boolean_selector_feature(random_boolean_feature, true),
            "minecraft:flower_cherry"
        );
        assert_eq!(
            super::super::random_boolean_selector_feature(random_boolean_feature, false),
            "minecraft:patch_grass"
        );
        let fill_layer_config = super::super::FillLayerConfigurationModel {
            height: 32,
            state: "minecraft:lava",
        };
        assert_eq!(
            super::super::validate_fill_layer_config(fill_layer_config, 384),
            Ok(fill_layer_config)
        );
        assert_eq!(
            super::super::validate_fill_layer_config(
                super::super::FillLayerConfigurationModel {
                    height: 385,
                    state: "minecraft:lava",
                },
                384,
            ),
            Err("fill layer height must be in 0..=dimension_y_size")
        );
        let mut fill_air = vec![false; 256];
        fill_air[0] = true;
        fill_air[17] = true;
        assert_eq!(
            super::super::fill_layer_placement_plan(
                BlockPos { x: 16, y: 0, z: 32 },
                -64,
                fill_layer_config,
                &fill_air,
            ),
            vec![
                BlockPos {
                    x: 16,
                    y: -32,
                    z: 32
                },
                BlockPos {
                    x: 17,
                    y: -32,
                    z: 33
                },
            ]
        );
        assert_eq!(super::super::end_island_layer_radius(4.0), 4);
        assert_eq!(super::super::end_island_next_size(4.0, 1), 2.5);
        let end_island =
            super::super::end_island_placement_plan(BlockPos { x: 0, y: 80, z: 0 }, 0, &[1, 1, 1]);
        assert!(end_island.contains(&super::super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 80, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(end_island.contains(&super::super::EndIslandPlacementBlock {
            pos: BlockPos { x: 0, y: 79, z: 0 },
            state: "minecraft:end_stone",
        }));
        assert!(!end_island.contains(&super::super::EndIslandPlacementBlock {
            pos: BlockPos { x: 5, y: 80, z: 5 },
            state: "minecraft:end_stone",
        }));
        let replace_sphere = super::super::ReplaceSphereConfigurationModel {
            target_state: "minecraft:netherrack",
            replace_state: "minecraft:basalt",
            radius_min: 3,
            radius_max: 7,
        };
        assert_eq!(
            super::super::validate_replace_sphere_config(replace_sphere),
            Ok(replace_sphere)
        );
        assert_eq!(super::super::replace_sphere_radius(replace_sphere, 5), 3);
        assert_eq!(
            super::super::validate_replace_sphere_config(super::super::ReplaceSphereConfigurationModel {
                target_state: "minecraft:netherrack",
                replace_state: "minecraft:basalt",
                radius_min: 8,
                radius_max: 7,
            }),
            Err("replace sphere radius bounds must be ordered in 0..=12")
        );
        assert_eq!(
            super::super::replace_sphere_find_target(
                BlockPos { x: 4, y: 70, z: 8 },
                -64,
                320,
                &["minecraft:air", "minecraft:netherrack"],
                "minecraft:netherrack",
            ),
            Some(BlockPos { x: 4, y: 69, z: 8 })
        );
        let sphere_positions =
            super::super::replace_sphere_positions(BlockPos { x: 0, y: 0, z: 0 }, 1, 2, 3);
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: 0, z: 0 }));
        assert!(sphere_positions.contains(&BlockPos { x: 0, y: -2, z: 0 }));
        assert!(!sphere_positions.contains(&BlockPos { x: 1, y: 2, z: 3 }));
        assert!(super::super::basalt_pillar_can_start(true, false));
        assert!(!super::super::basalt_pillar_can_start(true, true));
        assert!(super::super::basalt_pillar_hangoff_places(9));
        assert!(!super::super::basalt_pillar_hangoff_places(10));
        assert!(super::super::basalt_pillar_base_places(1, 2, 7));
        assert!(!super::super::basalt_pillar_base_places(3, 3, 1));
        let base_drop = vec![&[][..]; 49];
        let base_supported = vec![true; 49];
        let basalt_pillar = super::super::basalt_pillar_placement_plan(
            BlockPos { x: 0, y: 64, z: 0 },
            &[true, true, false],
            &[false, false],
            &[(1, 1, 10, 1), (1, 10, 1, 1)],
            &[0; 49],
            &base_drop,
            &base_supported,
        );
        assert!(basalt_pillar.contains(&super::super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            kind: super::super::BasaltPillarBlockKind::Core,
        }));
        assert!(basalt_pillar.contains(&super::super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: -1 },
            kind: super::super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(!basalt_pillar.contains(&super::super::BasaltPillarPlacementBlock {
            pos: BlockPos { x: -1, y: 64, z: 0 },
            kind: super::super::BasaltPillarBlockKind::HangOff,
        }));
        assert!(basalt_pillar.iter().any(|block| {
            block.kind == super::super::BasaltPillarBlockKind::Base && block.pos.y == 62
        }));
        assert!(super::super::basalt_columns_cannot_place_on(
            "minecraft:magma_block"
        ));
        assert!(!super::super::basalt_columns_cannot_place_on(
            "minecraft:netherrack"
        ));
        assert!(super::super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            31,
            32,
        ));
        assert!(!super::super::basalt_columns_is_air_or_lava_ocean(
            "minecraft:lava",
            33,
            32,
        ));
        assert!(super::super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:netherrack",
            64,
            32,
        ));
        assert!(!super::super::basalt_columns_can_place_at(
            "minecraft:air",
            "minecraft:magma_block",
            64,
            32,
        ));
        let column_config = super::super::ColumnFeatureConfigurationModel {
            reach_min: 1,
            reach_max: 3,
            height_min: 5,
            height_max: 10,
        };
        assert_eq!(
            super::super::validate_column_feature_config(column_config),
            Ok(column_config)
        );
        assert_eq!(
            super::super::validate_column_feature_config(super::super::ColumnFeatureConfigurationModel {
                reach_min: 4,
                reach_max: 3,
                height_min: 5,
                height_max: 10,
            }),
            Err("column reach bounds must be ordered in 0..=3")
        );
        assert_eq!(
            super::super::basalt_columns_cluster_parameters(7, 0.5),
            (true, 5, 50)
        );
        assert_eq!(
            super::super::basalt_columns_cluster_parameters(7, 0.95),
            (false, 7, 15)
        );
        let column_blocks = super::super::basalt_column_blocks_from_surface(
            BlockPos { x: 1, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            4,
            3,
            &[true, true, false, true],
            &[false, true, false, false],
        );
        assert!(column_blocks.contains(&super::super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
        }));
        assert!(column_blocks.contains(&super::super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 65, z: 0 },
        }));
        assert!(!column_blocks.contains(&super::super::BasaltColumnPlacementBlock {
            pos: BlockPos { x: 1, y: 67, z: 0 },
        }));
        let delta_config = super::super::DeltaFeatureConfigurationModel {
            contents: "minecraft:lava",
            rim: "minecraft:magma_block",
            size_min: 3,
            size_max: 7,
            rim_size_min: 0,
            rim_size_max: 2,
        };
        assert_eq!(super::super::validate_delta_config(delta_config), Ok(delta_config));
        assert_eq!(
            super::super::validate_delta_config(super::super::DeltaFeatureConfigurationModel {
                contents: "minecraft:lava",
                rim: "minecraft:magma_block",
                size_min: 17,
                size_max: 17,
                rim_size_min: 0,
                rim_size_max: 2,
            }),
            Err("delta size bounds must be ordered in 0..=16")
        );
        assert!(super::super::delta_cannot_replace("minecraft:bedrock"));
        assert!(!super::super::delta_cannot_replace("minecraft:netherrack"));
        assert!(super::super::delta_is_clear(super::super::DeltaClearInput {
            state: "minecraft:netherrack",
            contents: "minecraft:lava",
            up_air: false,
            down_air: false,
            north_air: false,
            south_air: false,
            west_air: false,
            east_air: false,
        }));
        assert!(!super::super::delta_is_clear(super::super::DeltaClearInput {
            state: "minecraft:netherrack",
            contents: "minecraft:lava",
            up_air: false,
            down_air: false,
            north_air: true,
            south_air: false,
            west_air: false,
            east_air: false,
        }));
        assert!(super::super::delta_has_rim(0.5, 1, 2));
        assert!(!super::super::delta_has_rim(0.95, 1, 2));
        let delta_offsets = super::super::delta_candidate_offsets(2, 1);
        assert!(delta_offsets.contains(&(0, 0)));
        assert!(delta_offsets.contains(&(2, 0)));
        assert!(!delta_offsets.contains(&(2, 1)));
        assert!(super::super::glowstone_can_start(true, "minecraft:netherrack"));
        assert!(!super::super::glowstone_can_start(true, "minecraft:air"));
        assert_eq!(
            super::super::glowstone_candidate_offset(7, 1, 11, 2, 6),
            BlockPos {
                x: 6,
                y: -11,
                z: -4
            }
        );
        assert!(super::super::glowstone_can_grow(true, 1));
        assert!(!super::super::glowstone_can_grow(true, 2));
        let nether_vegetation = super::super::NetherForestVegetationConfigModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:crimson_roots"),
            spread_width: 8,
            spread_height: 4,
        };
        assert_eq!(
            super::super::validate_nether_forest_vegetation_config(&nether_vegetation),
            Ok(())
        );
        assert!(super::super::nether_forest_vegetation_can_start(
            "minecraft:crimson_nylium",
            64,
            -64,
            320,
        ));
        assert!(!super::super::nether_forest_vegetation_can_start(
            "minecraft:netherrack",
            64,
            -64,
            320,
        ));
        assert_eq!(super::super::nether_forest_vegetation_attempts(8), 64);
        assert_eq!(
            super::super::nether_forest_vegetation_offset(
                super::super::NetherForestVegetationOffsetInput {
                    spread_width: 8,
                    spread_height: 4,
                    x_rolls: (7, 1),
                    y_rolls: (3, 1),
                    z_rolls: (2, 6),
                },
            ),
            BlockPos { x: 6, y: 2, z: -4 }
        );
        assert!(super::super::twisting_vines_valid_ground(
            "minecraft:warped_nylium"
        ));
        assert!(super::super::weeping_vines_valid_ceiling(
            "minecraft:nether_wart_block"
        ));
        assert_eq!(super::super::vine_height(2, 8, 6, 1), 6);
        assert_eq!(super::super::vine_height(2, 8, 1, 5), 1);
        assert_eq!(super::super::vine_age(17, 25, 9), 17);
        let twisting_column = super::super::twisting_vines_column(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            &[true, true, true],
            &[false, true, false],
            1,
        );
        assert_eq!(
            twisting_column,
            vec![
                super::super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    state: "minecraft:twisting_vines_plant",
                    kind: super::super::VineColumnBlockKind::Plant,
                    age: None,
                },
                super::super::VineColumnBlock {
                    pos: BlockPos { x: 0, y: 65, z: 0 },
                    state: "minecraft:twisting_vines",
                    kind: super::super::VineColumnBlockKind::Head,
                    age: Some(18),
                },
            ]
        );
        let weeping_column = super::super::weeping_vines_column(
            BlockPos { x: 0, y: 70, z: 0 },
            2,
            &[true, true, true],
            &[false, false, true],
            2,
        );
        assert_eq!(
            weeping_column.last(),
            Some(&super::super::VineColumnBlock {
                pos: BlockPos { x: 0, y: 68, z: 0 },
                state: "minecraft:weeping_vines",
                kind: super::super::VineColumnBlockKind::Head,
                age: Some(19),
            })
        );
        assert!(super::super::weeping_vines_wart_can_grow(true, 1));
        assert!(!super::super::weeping_vines_wart_can_grow(true, 2));
        let end_platform = super::super::end_platform_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(end_platform.len(), 100);
        assert_eq!(
            end_platform
                .iter()
                .filter(|block| block.state == "minecraft:obsidian")
                .count(),
            25
        );
        assert!(end_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos {
                x: -2,
                y: 63,
                z: -2
            },
            state: "minecraft:obsidian",
        }));
        assert!(end_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 2, y: 66, z: 2 },
            state: "minecraft:air",
        }));
        assert_eq!(
            super::super::void_start_platform_origin(64),
            BlockPos { x: 8, y: 67, z: 8 }
        );
        assert!(super::super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 1,
            z: 1
        }));
        assert!(!super::super::void_start_platform_applies_to_chunk(ChunkPos {
            x: 2,
            z: 0
        }));
        let void_platform = super::super::void_start_platform_blocks(ChunkPos { x: 0, z: 0 }, 64);
        assert!(void_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 8, y: 67, z: 8 },
            state: "minecraft:cobblestone",
        }));
        assert!(void_platform.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 67, z: 0 },
            state: "minecraft:stone",
        }));
        assert_eq!(
            super::super::void_start_platform_blocks(ChunkPos { x: 2, z: 0 }, 64),
            Vec::new()
        );
        assert_eq!(
            super::super::end_gateway_known_exit(BlockPos { x: 1, y: 2, z: 3 }, true),
            super::super::EndGatewayConfigurationModel {
                exit: Some(BlockPos { x: 1, y: 2, z: 3 }),
                exact: true,
            }
        );
        assert_eq!(
            super::super::end_gateway_delayed_exit_search(),
            super::super::EndGatewayConfigurationModel {
                exit: None,
                exact: false,
            }
        );
        let gateway = super::super::end_gateway_blocks(BlockPos { x: 0, y: 64, z: 0 });
        assert_eq!(gateway.len(), 45);
        assert!(gateway.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            state: "minecraft:end_gateway",
        }));
        assert!(gateway.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            state: "minecraft:bedrock",
        }));
        assert!(gateway.contains(&super::super::FeaturePlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 0 },
            state: "minecraft:air",
        }));
        assert_eq!(
            gateway
                .iter()
                .filter(|block| block.state == "minecraft:bedrock")
                .count(),
            12
        );
        assert!(super::super::chorus_plant_can_start(true, "minecraft:end_stone"));
        assert!(!super::super::chorus_plant_can_start(false, "minecraft:end_stone"));
        assert!(!super::super::chorus_plant_can_start(true, "minecraft:stone"));
        assert!(super::super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            Some(1),
        ));
        assert!(!super::super::chorus_all_horizontal_neighbors_empty(
            [true, false, true, true],
            None,
        ));
        assert!(super::super::chorus_branch_target_within_spread(
            BlockPos { x: 7, y: 68, z: -7 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert!(!super::super::chorus_branch_target_within_spread(
            BlockPos { x: 8, y: 68, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            8,
        ));
        assert_eq!(super::super::chorus_trunk_height(0, 0), 2);
        assert_eq!(super::super::chorus_trunk_height(1, 3), 4);
        assert_eq!(super::super::chorus_stem_attempts(0, 0), 1);
        assert_eq!(super::super::chorus_stem_attempts(1, 3), 3);
        let chorus_trunk =
            super::super::chorus_trunk_and_terminal_flower(BlockPos { x: 0, y: 64, z: 0 }, 0, 0, false);
        assert_eq!(
            chorus_trunk.last(),
            Some(&super::super::ChorusPlantPlacementBlock {
                pos: BlockPos { x: 0, y: 66, z: 0 },
                kind: super::super::ChorusPlantPlacementKind::Flower,
                age: Some(5),
            })
        );
        assert_eq!(
            chorus_trunk
                .iter()
                .filter(|block| block.kind == super::super::ChorusPlantPlacementKind::Plant)
                .count(),
            3
        );
        assert_eq!(
            super::super::end_podium_location(BlockPos { x: 1, y: 2, z: 3 }),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert!(super::super::end_podium_inside_rim(
            BlockPos { x: 2, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(!super::super::end_podium_inside_rim(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        assert!(super::super::end_podium_inside_body(
            BlockPos { x: 3, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
        ));
        let inactive_podium = super::super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, false);
        let active_podium = super::super::end_podium_blocks(BlockPos { x: 0, y: 64, z: 0 }, true);
        assert!(inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 63, z: 0 },
            kind: super::super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 63, z: 0 },
            kind: super::super::EndPodiumBlockKind::EndStone,
        }));
        assert!(inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 3, y: 64, z: 0 },
            kind: super::super::EndPodiumBlockKind::Bedrock,
        }));
        assert!(inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 0, y: 66, z: -1 },
            kind: super::super::EndPodiumBlockKind::WallTorch(super::super::HorizontalDirection::North),
        }));
        assert!(active_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::super::EndPodiumBlockKind::EndPortal,
        }));
        assert!(inactive_podium.contains(&super::super::EndPodiumPlacementBlock {
            pos: BlockPos { x: 1, y: 64, z: 1 },
            kind: super::super::EndPodiumBlockKind::Air,
        }));
        assert_eq!(
            active_podium
                .iter()
                .filter(|block| block.kind
                    == super::super::EndPodiumBlockKind::WallTorch(super::super::HorizontalDirection::East))
                .count(),
            1
        );
        let spike = super::super::end_spike_from_size(0, 2);
        assert_eq!(
            spike,
            super::super::EndSpikeModel {
                center_x: 42,
                center_z: 0,
                radius: 2,
                height: 82,
                guarded: true,
            }
        );
        assert!(super::super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 32, y: 0, z: 0 },
        ));
        assert!(!super::super::end_spike_is_center_within_chunk(
            spike,
            BlockPos { x: 16, y: 0, z: 0 },
        ));
        assert_eq!(
            super::super::end_spike_top_bounding_box(spike, -64, 320),
            (
                BlockPos {
                    x: 40,
                    y: -64,
                    z: -2
                },
                BlockPos {
                    x: 44,
                    y: 320,
                    z: 2
                },
            )
        );
        let spike_blocks = super::super::end_spike_cylinder_and_air_blocks(spike, 64);
        assert!(spike_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 64, z: 0 },
            kind: super::super::EndSpikeBlockKind::Obsidian,
        }));
        assert!(spike_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos {
                x: 40,
                y: 82,
                z: -2
            },
            kind: super::super::EndSpikeBlockKind::Air,
        }));
        let cage_blocks = super::super::end_spike_guard_cage_blocks(spike);
        assert!(cage_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos { x: 40, y: 82, z: 0 },
            kind: super::super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: false,
                east: false,
            },
        }));
        assert!(cage_blocks.contains(&super::super::EndSpikePlacementBlock {
            pos: BlockPos { x: 42, y: 85, z: 0 },
            kind: super::super::EndSpikeBlockKind::IronBars {
                north: true,
                south: true,
                west: true,
                east: true,
            },
        }));
        assert_eq!(
            super::super::end_spike_guard_cage_blocks(super::super::EndSpikeModel {
                guarded: false,
                ..spike
            }),
            Vec::new()
        );
        let spike_config = super::super::EndSpikeConfigurationModel {
            crystal_invulnerable: true,
            spikes: vec![spike],
            crystal_beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
        };
        assert_eq!(
            super::super::end_crystal_for_spike(spike, &spike_config, 0.25),
            super::super::EndCrystalPlacement {
                x: 42.5,
                y: 83.0,
                z: 0.5,
                yaw: 90.0,
                beam_target: Some(BlockPos { x: 0, y: 80, z: 0 }),
                invulnerable: true,
            }
        );
        assert_eq!(
            super::super::end_spike_crystal_support_blocks(spike),
            vec![
                super::super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 82, z: 0 },
                    kind: super::super::EndSpikeBlockKind::Bedrock,
                },
                super::super::EndSpikePlacementBlock {
                    pos: BlockPos { x: 42, y: 83, z: 0 },
                    kind: super::super::EndSpikeBlockKind::Fire,
                },
            ]
        );
        let huge_fungus_config = super::super::HugeFungusConfigurationModel {
            valid_base_state: "minecraft:crimson_nylium",
            stem_state: "minecraft:crimson_stem",
            hat_state: "minecraft:nether_wart_block",
            decor_state: "minecraft:shroomlight",
            planted: false,
        };
        assert!(super::super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:crimson_nylium",
        ));
        assert!(!super::super::huge_fungus_can_start(
            &huge_fungus_config,
            "minecraft:warped_nylium",
        ));
        assert_eq!(super::super::huge_fungus_total_height(0, 1), 4);
        assert_eq!(super::super::huge_fungus_total_height(9, 0), 26);
        assert!(super::super::huge_fungus_fits_height(64, 10, 80, false));
        assert!(!super::super::huge_fungus_fits_height(64, 15, 80, false));
        assert!(super::super::huge_fungus_fits_height(64, 15, 80, true));
        assert!(super::super::huge_fungus_is_huge(false, 0.05));
        assert!(!super::super::huge_fungus_is_huge(true, 0.05));
        let stem_blocks = super::super::huge_fungus_stem_blocks(
            BlockPos { x: 0, y: 64, z: 0 },
            3,
            true,
            &[0.0, 1.0, 1.0, 1.0],
        );
        assert!(stem_blocks.contains(&super::super::HugeFungusStemBlock {
            pos: BlockPos {
                x: -1,
                y: 64,
                z: -1
            },
            kind: super::super::HugeFungusStemKind::CornerStem,
        }));
        assert!(!stem_blocks.contains(&super::super::HugeFungusStemBlock {
            pos: BlockPos { x: -1, y: 64, z: 1 },
            kind: super::super::HugeFungusStemKind::CornerStem,
        }));
        assert!(stem_blocks.contains(&super::super::HugeFungusStemBlock {
            pos: BlockPos { x: 0, y: 66, z: 0 },
            kind: super::super::HugeFungusStemKind::Stem,
        }));
        assert_eq!(super::super::huge_fungus_hat_height(12, 0), 5);
        assert_eq!(super::super::huge_fungus_hat_radius(7, 12, 5, false, 0), 2);
        assert_eq!(super::super::huge_fungus_hat_radius(10, 12, 5, false, 2), 1);
        assert_eq!(super::super::huge_fungus_hat_radius(4, 12, 9, true, 0), 4);
        let hat_cells =
            super::super::huge_fungus_hat_cells(BlockPos { x: 0, y: 64, z: 0 }, 8, 0, &[0; 9], false);
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::super::HugeFungusHatRole::Bottom));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::super::HugeFungusHatRole::Inside));
        assert!(hat_cells
            .iter()
            .any(|cell| cell.role == super::super::HugeFungusHatRole::Corner));
        assert_eq!(
            super::super::huge_fungus_hat_drop_outcome(false, 0.1, 0, true),
            super::super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::super::huge_fungus_hat_drop_outcome(false, 0.2, 0, true),
            super::super::HugeFungusHatPlacement::None
        );
        assert_eq!(
            super::super::huge_fungus_hat_block_outcome(0.05, 0.0, 0.0, 0.1, 0.2, 0.1),
            super::super::HugeFungusHatPlacement::Decor
        );
        assert_eq!(
            super::super::huge_fungus_hat_block_outcome(0.2, 0.1, 0.05, 0.1, 0.2, 0.1),
            super::super::HugeFungusHatPlacement::HatWithWeepingVines
        );
        assert_eq!(
            super::super::huge_fungus_hat_probabilities(super::super::HugeFungusHatRole::Edge, true),
            Some((0.0005, 0.98, 0.07))
        );
        assert_eq!(super::super::huge_fungus_weeping_vine_height(4, 7), 10);

        let pile_config = super::super::BlockPileConfigurationModel {
            state_provider: BlockStateProviderModel::Simple("minecraft:hay_block"),
        };
        let all_shape_rolls = vec![(1.0, 0.0, 1.0); 7 * 7 * 2];
        let pile_positions = super::super::block_pile_placement_candidates(
            BlockPos { x: 0, y: 64, z: 0 },
            -64,
            1,
            1,
            &all_shape_rolls,
        );
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 64, z: 0 }));
        assert!(pile_positions.contains(&BlockPos { x: 0, y: 65, z: 0 }));
        assert!(pile_positions.len() > 20);
        assert!(super::super::block_pile_placement_candidates(
            BlockPos { x: 0, y: -60, z: 0 },
            -64,
            0,
            0,
            &all_shape_rolls,
        )
        .is_empty());
        assert_eq!(
            super::super::block_pile_try_place(
                &pile_config,
                true,
                "minecraft:grass_block",
                true,
                false,
                0,
            ),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", true, false, 0,),
            None
        );
        assert_eq!(
            super::super::block_pile_try_place(&pile_config, true, "minecraft:dirt_path", false, true, 0,),
            Some("minecraft:hay_block")
        );
        assert_eq!(
            super::super::block_pile_try_place(
                &pile_config,
                false,
                "minecraft:grass_block",
                true,
                true,
                0,
            ),
            None
        );

}
