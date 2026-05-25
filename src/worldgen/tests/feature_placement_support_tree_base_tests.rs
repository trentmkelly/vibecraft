use super::super::*;

pub(super) fn assert_tree_size_and_basic_tree_support() {
        let two = FeatureSizeModel::TwoLayers {
            limit: 2,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: None,
        };
        assert_eq!(super::super::validate_feature_size(two), Ok(two));
        assert_eq!(super::super::feature_size_at_height(two, 7, 1), 0);
        assert_eq!(super::super::feature_size_at_height(two, 7, 2), 1);

        let three = FeatureSizeModel::ThreeLayers {
            limit: 1,
            upper_limit: 2,
            lower_size: 0,
            middle_size: 1,
            upper_size: 2,
            min_clipped_height: Some(80),
        };
        assert_eq!(super::super::validate_feature_size(three), Ok(three));
        assert_eq!(super::super::feature_size_at_height(three, 8, 0), 0);
        assert_eq!(super::super::feature_size_at_height(three, 8, 5), 1);
        assert_eq!(super::super::feature_size_at_height(three, 8, 6), 2);
        let leaf_updates = super::super::tree_leaf_distance_updates(
            &[BlockPos { x: 0, y: 0, z: 0 }],
            &[
                (BlockPos { x: 1, y: 0, z: 0 }, 7),
                (BlockPos { x: 2, y: 0, z: 0 }, 7),
                (BlockPos { x: 3, y: 0, z: 0 }, 7),
                (BlockPos { x: 4, y: 0, z: 0 }, 7),
                (BlockPos { x: 5, y: 0, z: 0 }, 7),
                (BlockPos { x: 6, y: 0, z: 0 }, 7),
                (BlockPos { x: 7, y: 0, z: 0 }, 7),
                (BlockPos { x: 1, y: 1, z: 0 }, 1),
            ],
            &[],
            &[],
        );
        assert_eq!(
            leaf_updates,
            vec![
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 1, y: 0, z: 0 },
                    distance: 1,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 1, y: 1, z: 0 },
                    distance: 1,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 2, y: 0, z: 0 },
                    distance: 2,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 3, y: 0, z: 0 },
                    distance: 3,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 4, y: 0, z: 0 },
                    distance: 4,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 5, y: 0, z: 0 },
                    distance: 5,
                },
                super::super::TreeLeafDistanceUpdate {
                    pos: BlockPos { x: 6, y: 0, z: 0 },
                    distance: 6,
                },
            ]
        );
        assert!(super::super::tree_leaf_distance_updates(
            &[],
            &[(BlockPos { x: 1, y: 0, z: 0 }, 7)],
            &[],
            &[]
        )
        .is_empty());
        assert_eq!(
            super::super::validate_feature_size(FeatureSizeModel::TwoLayers {
                limit: 82,
                lower_size: 0,
                upper_size: 1,
                min_clipped_height: None,
            })
            .unwrap_err(),
            "feature size fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::super::validate_feature_size(FeatureSizeModel::ThreeLayers {
                limit: 1,
                upper_limit: 1,
                lower_size: 0,
                middle_size: 1,
                upper_size: 2,
                min_clipped_height: Some(81),
            })
            .unwrap_err(),
            "min_clipped_height must be in 0..=80".to_string()
        );
        assert_eq!(
            [
                "trunk_vine",
                "leave_vine",
                "pale_moss",
                "creaking_heart",
                "cocoa",
                "beehive",
                "alter_ground",
                "attached_to_leaves",
                "place_on_ground",
                "attached_to_logs",
            ]
            .iter()
            .filter_map(|decorator_type| super::super::tree_decorator_type(decorator_type))
            .collect::<Vec<_>>(),
            vec![
                "minecraft:trunk_vine",
                "minecraft:leave_vine",
                "minecraft:pale_moss",
                "minecraft:creaking_heart",
                "minecraft:cocoa",
                "minecraft:beehive",
                "minecraft:alter_ground",
                "minecraft:attached_to_leaves",
                "minecraft:place_on_ground",
                "minecraft:attached_to_logs",
            ]
        );
        assert_eq!(
            super::super::tree_decorator_type("minecraft:attached_to_logs"),
            Some("minecraft:attached_to_logs")
        );
        assert_eq!(super::super::tree_decorator_type("missing"), None);
        assert_eq!(
            super::super::trunk_placer_type("straight_trunk_placer"),
            Some("minecraft:straight_trunk_placer")
        );
        assert_eq!(
            super::super::foliage_placer_type("minecraft:cherry_foliage_placer"),
            Some("minecraft:cherry_foliage_placer")
        );
        assert_eq!(
            super::super::root_placer_type("mangrove_root_placer"),
            Some("minecraft:mangrove_root_placer")
        );
        let straight_trunk = TrunkPlacerModel {
            base_height: 5,
            height_rand_a: 2,
            height_rand_b: 1,
            kind: TrunkPlacerKind::Straight,
        };
        assert_eq!(
            super::super::validate_trunk_placer(straight_trunk),
            Ok(straight_trunk)
        );
        assert_eq!(super::super::trunk_placer_height(straight_trunk, 1, 1), 7);
        assert!(super::super::tree_valid_pos("minecraft:air"));
        assert!(super::super::tree_valid_pos("minecraft:oak_leaves"));
        assert!(super::super::tree_valid_pos("minecraft:dandelion"));
        assert!(super::super::tree_valid_pos("minecraft:water"));
        assert!(super::super::tree_valid_pos("minecraft:vine"));
        assert!(super::super::tree_valid_pos("minecraft:hanging_roots"));
        assert!(super::super::tree_valid_pos("minecraft:bush"));
        assert!(super::super::tree_valid_pos("minecraft:pale_moss_carpet"));
        assert!(super::super::tree_valid_pos("minecraft:short_dry_grass"));
        assert!(super::super::block_matches_tag(
            "minecraft:dirt",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(super::super::block_matches_tag(
            "minecraft:podzol",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(super::super::block_matches_tag(
            "minecraft:moss_block",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::super::block_matches_tag(
            "minecraft:grass_block",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::super::block_matches_tag(
            "minecraft:farmland",
            "minecraft:cannot_replace_below_tree_trunk"
        ));
        assert!(!super::super::tree_valid_pos("minecraft:oak_sapling"));
        assert!(!super::super::tree_valid_pos("minecraft:stone"));
        assert!(super::super::tree_trunk_free_pos("minecraft:oak_log"));
        assert!(super::super::tree_trunk_free_pos("minecraft:birch_log[axis=y]"));
        assert!(!super::super::tree_valid_pos("minecraft:oak_log"));
        let min_size = FeatureSizeModel::TwoLayers {
            limit: 1,
            lower_size: 0,
            upper_size: 1,
            min_clipped_height: Some(3),
        };
        let free_row = ["minecraft:air"; 9];
        let vine_row = ["minecraft:vine"; 9];
        let log_row = ["minecraft:oak_log"; 9];
        let stone_row = ["minecraft:stone"; 9];
        assert_eq!(
            super::super::tree_max_free_height(
                5,
                min_size,
                &[&free_row, &log_row, &log_row, &log_row, &log_row, &log_row, &log_row],
                true,
            ),
            5
        );
        assert_eq!(
            super::super::tree_max_free_height(
                5,
                min_size,
                &[&free_row, &free_row, &free_row, &stone_row],
                true,
            ),
            1
        );
        assert_eq!(
            super::super::tree_max_free_height(5, min_size, &[&vine_row], false),
            -2
        );
        assert!(super::super::tree_can_place(
            BlockPos { x: 0, y: 64, z: 0 },
            BlockPos { x: 0, y: 64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row, &free_row, &free_row, &free_row, &free_row, &free_row, &free_row,],
            true,
        ));
        assert!(!super::super::tree_can_place(
            BlockPos { x: 0, y: -64, z: 0 },
            BlockPos { x: 0, y: -64, z: 0 },
            5,
            min_size,
            Some(3),
            -64,
            320,
            &[&free_row],
            true,
        ));
        assert_eq!(
            super::super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 33,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            })
            .unwrap_err(),
            "trunk placer base fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::super::validate_trunk_placer(TrunkPlacerModel {
                base_height: 5,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Cherry {
                    branch_count_min: 1,
                    branch_count_max: 3,
                    branch_horizontal_length_min: 2,
                    branch_horizontal_length_max: 16,
                    branch_start_offset_from_top_min: -1,
                    branch_start_offset_from_top_max: -1,
                    branch_end_offset_from_top_min: -16,
                    branch_end_offset_from_top_max: 16,
                },
            })
            .unwrap_err(),
            "trunk placer variant fields are outside vanilla codec ranges".to_string()
        );
        let blob_foliage = FoliagePlacerModel {
            radius_min: 1,
            radius_max: 2,
            offset_min: 0,
            offset_max: 1,
            kind: FoliagePlacerKind::Blob { height: 3 },
        };
        assert_eq!(
            super::super::validate_foliage_placer(blob_foliage),
            Ok(blob_foliage)
        );
        let max_height_pine = FoliagePlacerModel {
            radius_min: 0,
            radius_max: 16,
            offset_min: 0,
            offset_max: 16,
            kind: FoliagePlacerKind::Pine {
                height_min: 0,
                height_max: 24,
            },
        };
        assert_eq!(
            super::super::validate_foliage_placer(max_height_pine),
            Ok(max_height_pine)
        );
        assert_eq!(
            super::super::validate_foliage_placer(FoliagePlacerModel {
                kind: FoliagePlacerKind::Spruce {
                    height_min: 0,
                    height_max: 25,
                },
                ..max_height_pine
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::super::validate_foliage_placer(FoliagePlacerModel {
                radius_min: 0,
                radius_max: 16,
                offset_min: 0,
                offset_max: 16,
                kind: FoliagePlacerKind::RandomSpread {
                    foliage_height_min: 0,
                    foliage_height_max: 1,
                    leaf_placement_attempts: 1,
                },
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        assert_eq!(
            super::super::validate_foliage_placer(FoliagePlacerModel {
                radius_min: 0,
                radius_max: 16,
                offset_min: 0,
                offset_max: 16,
                kind: FoliagePlacerKind::Cherry {
                    height: 3,
                    wide_bottom_layer_hole_chance: 0.0,
                    corner_hole_chance: 0.0,
                    hanging_leaves_chance: 0.0,
                    hanging_leaves_extension_chance: 0.0,
                },
            })
            .unwrap_err(),
            "foliage placer variant fields are outside vanilla codec ranges".to_string()
        );
        let mangrove_root = RootPlacerModel {
            above_root_placement_chance: Some(0.5),
            mangrove_root_placement: MangroveRootPlacementModel {
                max_root_width: 8,
                max_root_length: 15,
                random_skew_chance: 0.2,
            },
        };
        assert_eq!(
            super::super::validate_root_placer(mangrove_root),
            Ok(mangrove_root)
        );
        assert_eq!(
            super::super::validate_root_placer(RootPlacerModel {
                above_root_placement_chance: Some(1.25),
                mangrove_root_placement: MangroveRootPlacementModel {
                    max_root_width: 8,
                    max_root_length: 15,
                    random_skew_chance: 0.2,
                },
            })
            .unwrap_err(),
            "root placer fields are outside vanilla codec ranges".to_string()
        );
        let root_system_config = super::super::RootSystemConfigurationModel {
            tree_feature: "minecraft:azalea_tree",
            required_vertical_space_for_tree: 3,
            root_radius: 3,
            root_replaceable: "#minecraft:dirt",
            root_state_provider: BlockStateProviderModel::Simple("minecraft:rooted_dirt"),
            root_placement_attempts: 2,
            root_column_max_height: 4,
            hanging_root_radius: 3,
            hanging_roots_vertical_span: 2,
            hanging_root_state_provider: BlockStateProviderModel::Simple("minecraft:hanging_roots"),
            hanging_root_placement_attempts: 2,
            allowed_vertical_water_for_tree: 2,
        };
        assert_eq!(
            super::super::validate_root_system_configuration(&root_system_config),
            Ok(())
        );
        assert_eq!(
            super::super::mangrove_potential_root_positions(
                BlockPos {
                    x: 404,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                0.5,
                0.25,
                false,
            ),
            vec![
                BlockPos {
                    x: 404,
                    y: 63,
                    z: 400,
                },
                BlockPos {
                    x: 405,
                    y: 63,
                    z: 400,
                },
            ]
        );
        assert_eq!(
            super::super::mangrove_potential_root_positions(
                BlockPos {
                    x: 401,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                0.0,
                0.5,
                true,
            ),
            vec![BlockPos {
                x: 402,
                y: 64,
                z: 400,
            }]
        );
        assert_eq!(
            super::super::mangrove_potential_root_positions(
                BlockPos {
                    x: 406,
                    y: 64,
                    z: 400,
                },
                HorizontalDirection::East,
                BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                4,
                1.0,
                0.0,
                true,
            ),
            vec![BlockPos {
                x: 406,
                y: 63,
                z: 400,
            }]
        );
        assert!(super::super::root_system_is_allowed_tree_space(
            "minecraft:water",
            1,
            2
        ));
        assert!(!super::super::root_system_is_allowed_tree_space(
            "minecraft:water",
            2,
            2
        ));
        let root_plan = super::super::root_system_placement_plan(
            super::super::RootSystemPlacementInput {
                origin: BlockPos {
                    x: 400,
                    y: 64,
                    z: 400,
                },
                origin_is_air: true,
                config: &root_system_config,
                tree_candidates: &[
                    super::super::RootSystemTreeCandidateModel {
                        pos: BlockPos {
                            x: 400,
                            y: 65,
                            z: 400,
                        },
                        allowed_tree_position: true,
                        vertical_space_states: vec![
                            "minecraft:air",
                            "minecraft:water",
                            "minecraft:air",
                        ],
                        below_state: "minecraft:stone",
                        tree_feature_places: false,
                    },
                    super::super::RootSystemTreeCandidateModel {
                        pos: BlockPos {
                            x: 400,
                            y: 66,
                            z: 400,
                        },
                        allowed_tree_position: true,
                        vertical_space_states: vec![
                            "minecraft:air",
                            "minecraft:air",
                            "minecraft:air",
                        ],
                        below_state: "minecraft:dirt",
                        tree_feature_places: true,
                    },
                ],
                root_rolls: &[
                    super::super::RootSystemOffsetRoll {
                        positive_x: 2,
                        negative_x: 1,
                        positive_z: 1,
                        ..Default::default()
                    },
                    super::super::RootSystemOffsetRoll {
                        positive_x: 0,
                        negative_x: 0,
                        positive_z: 0,
                        negative_z: 1,
                        ..Default::default()
                    },
                ],
                hanging_root_rolls: &[
                    super::super::RootSystemOffsetRoll {
                        positive_x: 1,
                        positive_y: 1,
                        positive_z: 0,
                        ..Default::default()
                    },
                    super::super::RootSystemOffsetRoll {
                        negative_x: 1,
                        negative_y: 1,
                        negative_z: 1,
                        ..Default::default()
                    },
                ],
                root_replaceable_positions: &[
                    BlockPos {
                        x: 401,
                        y: 64,
                        z: 401,
                    },
                    BlockPos {
                        x: 400,
                        y: 64,
                        z: 399,
                    },
                ],
                hanging_root_candidates: &[BlockPos {
                    x: 401,
                    y: 65,
                    z: 400,
                }],
            },
        )
        .unwrap();
        assert_eq!(
            root_plan.tree_origin,
            Some(BlockPos {
                x: 400,
                y: 66,
                z: 400,
            })
        );
        assert_eq!(
            root_plan.blocks,
            vec![
                super::super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 401,
                        y: 64,
                        z: 401,
                    },
                    state: "minecraft:rooted_dirt",
                    kind: super::super::RootSystemPlacementKind::RootedDirt,
                },
                super::super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 400,
                        y: 64,
                        z: 399,
                    },
                    state: "minecraft:rooted_dirt",
                    kind: super::super::RootSystemPlacementKind::RootedDirt,
                },
                super::super::RootSystemPlacementBlock {
                    pos: BlockPos {
                        x: 401,
                        y: 65,
                        z: 400,
                    },
                    state: "minecraft:hanging_roots",
                    kind: super::super::RootSystemPlacementKind::HangingRoot,
                },
            ]
        );
        assert!(
            !super::super::root_system_placement_plan(
                super::super::RootSystemPlacementInput {
                    origin: BlockPos {
                        x: 400,
                        y: 64,
                        z: 400,
                    },
                    origin_is_air: false,
                    config: &root_system_config,
                    tree_candidates: &[],
                    root_rolls: &[],
                    hanging_root_rolls: &[],
                    root_replaceable_positions: &[],
                    hanging_root_candidates: &[],
                },
            )
            .unwrap()
            .attempted_roots
        );
        let tree_plan = super::super::simple_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            straight_trunk,
            FoliagePlacerModel {
                offset_min: 0,
                offset_max: 0,
                ..blob_foliage
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            1,
            1,
        )
        .unwrap();
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk
                && block.pos == BlockPos { x: 8, y: 63, z: 8 }
                && block.state == "minecraft:dirt"
        }));
        assert_eq!(
            tree_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Log)
                .count(),
            7
        );
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 8, y: 71, z: 8 }
                && block.state == "minecraft:oak_leaves"
        }));
        assert!(!tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 7, y: 71, z: 7 }
        }));
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 6, y: 68, z: 8 }
        }));
        assert!(tree_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos == BlockPos { x: 10, y: 68, z: 8 }
        }));
        let tree_config = super::super::TreeConfigurationModel {
            trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
            foliage_provider: BlockStateProviderModel::Simple("minecraft:oak_leaves"),
            dirt_provider: BlockStateProviderModel::Simple("minecraft:dirt"),
            trunk_placer: straight_trunk,
            foliage_placer: FoliagePlacerModel {
                offset_min: 0,
                offset_max: 0,
                ..blob_foliage
            },
            minimum_size: min_size,
            root_placer: Some(mangrove_root),
            decorators: vec![TreeDecoratorModel::Cocoa { probability: 0.25 }],
            ignore_vines: false,
        };
        assert_eq!(super::super::validate_tree_configuration(&tree_config), Ok(()));
        let full_tree_rows = [
            &free_row[..],
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &free_row,
        ];
        let configured_plan = super::super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &full_tree_rows,
            1,
            1,
        )
        .unwrap()
        .expect("valid tree config should place");
        assert!(configured_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::DirtBelowTrunk && block.state == "minecraft:dirt"
        }));
        assert!(super::super::configured_tree_placement_plan(
            BlockPos { x: 8, y: -64, z: 8 },
            &tree_config,
            -64,
            320,
            &full_tree_rows,
            1,
            1,
        )
        .unwrap()
        .is_none());
        let vine_blocked_rows = [&vine_row[..]];
        assert!(super::super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &vine_blocked_rows,
            1,
            1,
        )
        .unwrap()
        .is_none());
        let clipped_rows = [
            &free_row[..],
            &free_row,
            &free_row,
            &free_row,
            &free_row,
            &stone_row,
        ];
        assert!(super::super::configured_tree_placement_plan(
            BlockPos { x: 8, y: 64, z: 8 },
            &tree_config,
            -64,
            320,
            &clipped_rows,
            1,
            1,
        )
        .unwrap()
        .is_some());
        assert_eq!(
            super::super::validate_tree_configuration(&super::super::TreeConfigurationModel {
                decorators: vec![TreeDecoratorModel::Cocoa { probability: 1.25 }],
                ..tree_config.clone()
            })
            .unwrap_err(),
            "tree decorator probability must be in 0.0..=1.0".to_string()
        );

        assert_eq!(
            super::super::simple_tree_placement_plan(
                BlockPos { x: 8, y: 64, z: 8 },
                TrunkPlacerModel {
                    base_height: 5,
                    height_rand_a: 0,
                    height_rand_b: 0,
                    kind: TrunkPlacerKind::Forking,
                },
                blob_foliage,
                "minecraft:oak_log",
                "minecraft:oak_leaves",
                "minecraft:dirt",
                0,
                0,
            )
            .unwrap_err(),
            "only straight trunk placement is modeled by simple_tree_placement_plan".to_string()
        );
}
