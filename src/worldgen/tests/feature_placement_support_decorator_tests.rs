use super::super::*;

pub(super) fn assert_decorator_spring_and_monster_room_support() {
    assert_fallen_tree_support();
    assert_tree_decorator_validation();
    assert_lowest_trunk_or_root_positions();
    assert_trunk_vine_decorator_support();
    assert_leaf_vine_decorator_support();
    assert_cocoa_decorator_support();
    assert_beehive_and_creaking_heart_decorators();
    assert_pale_moss_decorator_support();
    assert_place_on_ground_decorator_validation();
    assert_place_on_ground_decorator_placement();
    assert_alter_ground_decorator_support();
    assert_attached_to_logs_decorator_support();
    assert_attached_to_leaves_decorator_validation();
    assert_attached_to_leaves_decorator_success();
    assert_attached_to_leaves_decorator_empty_when_probability_roll_fails();
}

fn assert_fallen_tree_support() {
    let fallen_config = super::super::FallenTreeConfigurationModel {
        trunk_provider: BlockStateProviderModel::Simple("minecraft:oak_log"),
        min_log_length: 4,
        max_log_length: 7,
        stump_decorators: vec![TreeDecoratorModel::TrunkVine],
        log_decorators: vec![TreeDecoratorModel::AttachedToLogs { probability: 0.1 }],
    };
    assert_eq!(super::super::fallen_tree_log_length(4, 7, 0), 2);
    assert_eq!(
        super::super::fallen_tree_start_pos(
            BlockPos { x: 0, y: 64, z: 0 },
            super::super::HorizontalDirection::East,
            1,
            &[false, true],
        ),
        Some(BlockPos { x: 3, y: 64, z: 0 })
    );
    assert!(super::super::fallen_tree_can_place_log(
        &[true, true, true, true],
        &[true, false, false, true],
    ));
    assert!(!super::super::fallen_tree_can_place_log(
        &[true, true, true],
        &[false, false, false],
    ));
    let fallen_plan =
        super::super::fallen_tree_placement_plan(super::super::FallenTreePlacementInput {
            origin: BlockPos { x: 0, y: 64, z: 0 },
            config: &fallen_config,
            direction: super::super::HorizontalDirection::East,
            log_length_roll: 1,
            distance_roll: 0,
            ground_probe: &[true],
            valid_tree_positions: &[true, true, true],
            over_solid_ground: &[true, true, true],
        })
        .unwrap();
    assert_eq!(fallen_plan.stump_decorators, 1);
    assert_eq!(fallen_plan.log_decorators, 1);
    assert_eq!(
        fallen_plan.blocks[0],
        super::super::FallenTreeBlock {
            pos: BlockPos { x: 0, y: 64, z: 0 },
            state: "minecraft:oak_log",
            mark_above_for_post_processing: true,
        }
    );
    assert!(fallen_plan.blocks.contains(&super::super::FallenTreeBlock {
        pos: BlockPos { x: 2, y: 65, z: 0 },
        state: "minecraft:oak_log[axis=x]",
        mark_above_for_post_processing: true,
    }));
}

fn assert_tree_decorator_validation() {
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::Cocoa { probability: 0.25 }),
        Ok(TreeDecoratorModel::Cocoa { probability: 0.25 })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::LeaveVine { probability: 1.0 }),
        Ok(TreeDecoratorModel::LeaveVine { probability: 1.0 })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::CreakingHeart {
            probability: 0.0
        }),
        Ok(TreeDecoratorModel::CreakingHeart { probability: 0.0 })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::AttachedToLeaves {
            probability: 1.0
        }),
        Ok(TreeDecoratorModel::AttachedToLeaves { probability: 1.0 })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::AttachedToLogs {
            probability: 0.0
        }),
        Ok(TreeDecoratorModel::AttachedToLogs { probability: 0.0 })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::PaleMoss {
            leaves_probability: 0.25,
            trunk_probability: 0.5,
            ground_probability: 1.0,
        }),
        Ok(TreeDecoratorModel::PaleMoss {
            leaves_probability: 0.25,
            trunk_probability: 0.5,
            ground_probability: 1.0,
        })
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::PaleMoss {
            leaves_probability: 1.1,
            trunk_probability: 0.5,
            ground_probability: 1.0,
        })
        .unwrap_err(),
        "pale moss decorator probabilities must be in 0.0..=1.0".to_string()
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::Beehive { probability: 1.5 })
            .unwrap_err(),
        "tree decorator probability must be in 0.0..=1.0".to_string()
    );
    assert_eq!(
        super::super::validate_tree_decorator(TreeDecoratorModel::AttachedToLogs {
            probability: -0.1
        })
        .unwrap_err(),
        "tree decorator probability must be in 0.0..=1.0".to_string()
    );
    assert!(super::super::tree_decorator_should_place(0.25, 0.249));
    assert!(!super::super::tree_decorator_should_place(0.25, 0.25));
}

fn assert_lowest_trunk_or_root_positions() {
    assert_eq!(
        super::super::tree_lowest_trunk_or_root_positions(
            &[BlockPos { x: 0, y: 3, z: 0 }, BlockPos { x: 0, y: 1, z: 0 },],
            &[],
        ),
        vec![BlockPos { x: 0, y: 1, z: 0 }, BlockPos { x: 0, y: 3, z: 0 },]
    );
    assert_eq!(
        super::super::tree_lowest_trunk_or_root_positions(
            &[BlockPos { x: 0, y: 1, z: 0 }],
            &[
                BlockPos { x: 1, y: 0, z: 0 },
                BlockPos { x: 1, y: -1, z: 0 },
            ],
        ),
        vec![
            BlockPos { x: 1, y: -1, z: 0 },
            BlockPos { x: 1, y: 0, z: 0 },
        ]
    );
    assert_eq!(
        super::super::tree_lowest_trunk_or_root_positions(
            &[BlockPos { x: 0, y: 1, z: 0 }],
            &[BlockPos { x: 1, y: 1, z: 0 }],
        ),
        vec![BlockPos { x: 0, y: 1, z: 0 }, BlockPos { x: 1, y: 1, z: 0 },]
    );
}

fn assert_trunk_vine_decorator_support() {
    assert_eq!(
        super::super::trunk_vine_decorator_placement(
            &[
                super::super::TrunkVineLogContext {
                    pos: BlockPos {
                        x: 10,
                        y: 66,
                        z: 10
                    },
                    west_air: true,
                    east_air: true,
                    north_air: true,
                    south_air: true,
                },
                super::super::TrunkVineLogContext {
                    pos: BlockPos {
                        x: 10,
                        y: 64,
                        z: 10
                    },
                    west_air: true,
                    east_air: false,
                    north_air: true,
                    south_air: true,
                },
            ],
            &[1, 2, 0, 1, 0, 1, 2, 0],
        ),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 9, y: 64, z: 10 },
                state: "minecraft:vine[east=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 10,
                    y: 64,
                    z: 11
                },
                state: "minecraft:vine[north=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 11,
                    y: 66,
                    z: 10
                },
                state: "minecraft:vine[west=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 10, y: 66, z: 9 },
                state: "minecraft:vine[south=true]",
            },
        ]
    );
}

fn assert_leaf_vine_decorator_support() {
    assert_eq!(
        super::super::leave_vine_decorator_placement(
            &[
                super::super::LeaveVineLeafContext {
                    pos: BlockPos {
                        x: 30,
                        y: 70,
                        z: 30
                    },
                    west_air: true,
                    east_air: true,
                    north_air: true,
                    south_air: true,
                    west_below_air: [true, true, false, true],
                    east_below_air: [false, true, true, true],
                    north_below_air: [true, true, true, true],
                    south_below_air: [true, true, true, true],
                },
                super::super::LeaveVineLeafContext {
                    pos: BlockPos {
                        x: 30,
                        y: 68,
                        z: 30
                    },
                    west_air: true,
                    east_air: false,
                    north_air: true,
                    south_air: true,
                    west_below_air: [true, false, true, true],
                    east_below_air: [true, true, true, true],
                    north_below_air: [true, true, true, true],
                    south_below_air: [true, true, true, true],
                },
            ],
            0.5,
            &[0.1, 0.2, 0.6, 0.5, 0.6, 0.6, 0.0, 0.7],
        ),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 29,
                    y: 68,
                    z: 30
                },
                state: "minecraft:vine[east=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 29,
                    y: 67,
                    z: 30
                },
                state: "minecraft:vine[east=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 30,
                    y: 70,
                    z: 29
                },
                state: "minecraft:vine[south=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 30,
                    y: 69,
                    z: 29
                },
                state: "minecraft:vine[south=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 30,
                    y: 68,
                    z: 29
                },
                state: "minecraft:vine[south=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 30,
                    y: 67,
                    z: 29
                },
                state: "minecraft:vine[south=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 30,
                    y: 66,
                    z: 29
                },
                state: "minecraft:vine[south=true]",
            },
        ]
    );
}

fn assert_cocoa_decorator_support() {
    assert_eq!(
        super::super::cocoa_decorator_placement(
            &[
                super::super::CocoaLogContext {
                    pos: BlockPos {
                        x: 20,
                        y: 67,
                        z: 20
                    },
                    north_air: true,
                    east_air: true,
                    south_air: true,
                    west_air: true,
                },
                super::super::CocoaLogContext {
                    pos: BlockPos {
                        x: 20,
                        y: 64,
                        z: 20
                    },
                    north_air: true,
                    east_air: true,
                    south_air: false,
                    west_air: true,
                },
            ],
            0.5,
            0.49,
            &[0.1, 0.2, 0.3, 0.25, 0.1, 0.1, 0.1, 0.1],
            &[1, 2, 4, 0, 1, 2],
        ),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 19,
                    y: 64,
                    z: 20
                },
                state: "minecraft:cocoa[age=1,facing=east]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 21,
                    y: 64,
                    z: 20
                },
                state: "minecraft:cocoa[age=2,facing=west]",
            },
        ]
    );
    assert!(super::super::cocoa_decorator_placement(
        &[super::super::CocoaLogContext {
            pos: BlockPos {
                x: 20,
                y: 64,
                z: 20
            },
            north_air: true,
            east_air: true,
            south_air: true,
            west_air: true,
        }],
        0.5,
        0.5,
        &[0.0; 4],
        &[0; 4],
    )
    .is_empty());
}

fn assert_beehive_and_creaking_heart_decorators() {
    assert_eq!(
        super::super::beehive_decorator_placement(super::super::BeehiveDecoratorInput {
            logs: &[
                BlockPos { x: 0, y: 64, z: 0 },
                BlockPos { x: 0, y: 65, z: 0 },
                BlockPos { x: 0, y: 66, z: 0 },
            ],
            leaves: &[BlockPos { x: 0, y: 66, z: 0 }],
            probability: 1.0,
            global_roll: 0.0,
            leafless_height_roll: 0,
            shuffled_candidate_indices: &[2, 0, 1],
            candidate_air: &[
                (BlockPos { x: -1, y: 65, z: 0 }, false, true),
                (BlockPos { x: 1, y: 65, z: 0 }, true, false),
                (BlockPos { x: 0, y: 65, z: 1 }, true, true),
            ],
            bee_count_roll: 1,
            bee_ticks_rolls: &[598, 599, 600],
        }),
        Some(super::super::BeehiveDecoratorPlacement {
            pos: BlockPos { x: 0, y: 65, z: 1 },
            state: "minecraft:bee_nest[facing=south,honey_level=0]",
            bee_ticks_in_hive: vec![598, 0, 1],
        })
    );
    assert!(
        super::super::beehive_decorator_placement(super::super::BeehiveDecoratorInput {
            logs: &[BlockPos { x: 0, y: 64, z: 0 }],
            leaves: &[],
            probability: 0.5,
            global_roll: 0.5,
            leafless_height_roll: 0,
            shuffled_candidate_indices: &[],
            candidate_air: &[],
            bee_count_roll: 0,
            bee_ticks_rolls: &[],
        })
        .is_none()
    );
    assert_eq!(
        super::super::creaking_heart_decorator_placement(
            &[
                BlockPos { x: 5, y: 66, z: 5 },
                BlockPos { x: 5, y: 64, z: 5 },
                BlockPos { x: 5, y: 65, z: 5 },
            ],
            0.75,
            0.5,
            &[2, 0, 1],
            &[
                (
                    BlockPos { x: 5, y: 66, z: 5 },
                    [true, true, false, true, true, true]
                ),
                (
                    BlockPos { x: 5, y: 64, z: 5 },
                    [true, true, true, true, true, true]
                ),
                (
                    BlockPos { x: 5, y: 65, z: 5 },
                    [true, true, true, true, false, true]
                ),
            ],
        ),
        Some(super::super::TreeDecoratorPlacement {
            pos: BlockPos { x: 5, y: 64, z: 5 },
            state: "minecraft:creaking_heart[active=false,axis=y,natural=true]",
        })
    );
    assert!(super::super::creaking_heart_decorator_placement(
        &[BlockPos { x: 5, y: 64, z: 5 }],
        0.75,
        0.75,
        &[0],
        &[(BlockPos { x: 5, y: 64, z: 5 }, [true; 6])],
    )
    .is_none());
}

fn assert_pale_moss_decorator_support() {
    assert_eq!(
        super::super::pale_moss_decorator_placement(super::super::PaleMossDecoratorInput {
            logs: &[
                super::super::PaleMossAttachmentContext {
                    pos: BlockPos { x: 6, y: 66, z: 6 },
                    down_air: true,
                    below_air: vec![true, true, false],
                },
                super::super::PaleMossAttachmentContext {
                    pos: BlockPos { x: 6, y: 64, z: 6 },
                    down_air: true,
                    below_air: vec![true, true, true],
                },
                super::super::PaleMossAttachmentContext {
                    pos: BlockPos { x: 6, y: 65, z: 6 },
                    down_air: false,
                    below_air: vec![true],
                },
            ],
            leaves: &[super::super::PaleMossAttachmentContext {
                pos: BlockPos { x: 7, y: 68, z: 7 },
                down_air: true,
                below_air: vec![true, false],
            }],
            leaves_probability: 0.5,
            trunk_probability: 0.75,
            ground_probability: 0.25,
            ground_roll: 0.1,
            trunk_rolls: &[0.2, 0.9, 0.8],
            leaf_rolls: &[0.25],
            hanger_rolls: &[0.8, 0.3, 0.6],
        }),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 6, y: 65, z: 6 },
                state: "minecraft:configured_feature/pale_moss_patch",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 6, y: 63, z: 6 },
                state: "minecraft:pale_hanging_moss[tip=false]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 6, y: 62, z: 6 },
                state: "minecraft:pale_hanging_moss[tip=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 7, y: 67, z: 7 },
                state: "minecraft:pale_hanging_moss[tip=false]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 7, y: 66, z: 7 },
                state: "minecraft:pale_hanging_moss[tip=true]",
            },
        ]
    );
    assert!(
        super::super::pale_moss_decorator_placement(super::super::PaleMossDecoratorInput {
            logs: &[],
            leaves: &[],
            leaves_probability: 1.0,
            trunk_probability: 1.0,
            ground_probability: 1.0,
            ground_roll: 0.0,
            trunk_rolls: &[],
            leaf_rolls: &[],
            hanger_rolls: &[],
        })
        .is_empty()
    );
}

fn assert_place_on_ground_decorator_validation() {
    assert_eq!(
        super::super::validate_place_on_ground_decorator_fields(1, 0, 0),
        Ok(())
    );
    assert_eq!(
        super::super::validate_place_on_ground_decorator_fields(0, 0, 0).unwrap_err(),
        "place-on-ground tries must be positive".to_string()
    );
    assert_eq!(
        super::super::validate_place_on_ground_decorator_fields(1, -1, 0).unwrap_err(),
        "place-on-ground radius and height must be non-negative".to_string()
    );
}

fn assert_place_on_ground_decorator_placement() {
    assert_eq!(
        super::super::place_on_ground_decorator_placement(
            &[
                BlockPos { x: 0, y: 64, z: 0 },
                BlockPos { x: 2, y: 64, z: 1 },
                BlockPos {
                    x: 10,
                    y: 65,
                    z: 10
                },
            ],
            5,
            2,
            1,
            "minecraft:pale_moss_carpet",
            &[
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos {
                        x: -2,
                        y: 64,
                        z: -2
                    },
                    above_is_air_or_vine: true,
                    pos_is_solid_render: true,
                    motion_blocking_no_leaves_height: 65,
                },
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos { x: 4, y: 65, z: 3 },
                    above_is_air_or_vine: true,
                    pos_is_solid_render: true,
                    motion_blocking_no_leaves_height: 67,
                },
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos { x: 0, y: 63, z: 0 },
                    above_is_air_or_vine: false,
                    pos_is_solid_render: true,
                    motion_blocking_no_leaves_height: 64,
                },
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    above_is_air_or_vine: true,
                    pos_is_solid_render: false,
                    motion_blocking_no_leaves_height: 65,
                },
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos { x: 2, y: 64, z: 1 },
                    above_is_air_or_vine: true,
                    pos_is_solid_render: true,
                    motion_blocking_no_leaves_height: 64,
                },
                super::super::PlaceOnGroundAttemptContext {
                    pos: BlockPos { x: 0, y: 64, z: 0 },
                    above_is_air_or_vine: true,
                    pos_is_solid_render: true,
                    motion_blocking_no_leaves_height: 65,
                },
            ],
        )
        .unwrap(),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: -2,
                    y: 65,
                    z: -2
                },
                state: "minecraft:pale_moss_carpet",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos { x: 2, y: 65, z: 1 },
                state: "minecraft:pale_moss_carpet",
            },
        ]
    );
    assert!(super::super::place_on_ground_decorator_placement(
        &[],
        1,
        0,
        0,
        "minecraft:pale_moss_carpet",
        &[],
    )
    .unwrap()
    .is_empty());
    assert!(!super::super::tree_decorator_solid_render(
        "minecraft:leaf_litter"
    ));
    assert!(!super::super::tree_decorator_solid_render(
        "minecraft:oak_leaves"
    ));
    assert!(super::super::tree_decorator_solid_render(
        "minecraft:grass_block"
    ));
}

fn assert_alter_ground_decorator_support() {
    let alter_ground = super::super::alter_ground_decorator_placement(
        &[
            BlockPos { x: 0, y: 64, z: 0 },
            BlockPos { x: 2, y: 65, z: 0 },
        ],
        &[9, 18, 27, 36, 63],
        &[
            super::super::AlterGroundScanContext {
                pos: BlockPos {
                    x: -1,
                    y: 66,
                    z: -1,
                },
                provider_state: Some("minecraft:rooted_dirt"),
                is_air: true,
            },
            super::super::AlterGroundScanContext {
                pos: BlockPos {
                    x: -3,
                    y: 66,
                    z: -3,
                },
                provider_state: Some("minecraft:moss_block"),
                is_air: true,
            },
            super::super::AlterGroundScanContext {
                pos: BlockPos { x: 4, y: 66, z: 4 },
                provider_state: Some("minecraft:podzol"),
                is_air: true,
            },
            super::super::AlterGroundScanContext {
                pos: BlockPos { x: 1, y: 63, z: -1 },
                provider_state: None,
                is_air: false,
            },
            super::super::AlterGroundScanContext {
                pos: BlockPos { x: 1, y: 62, z: -1 },
                provider_state: Some("minecraft:coarse_dirt"),
                is_air: true,
            },
            super::super::AlterGroundScanContext {
                pos: BlockPos { x: 2, y: 61, z: 2 },
                provider_state: Some("minecraft:coarse_dirt"),
                is_air: true,
            },
        ],
    );
    assert!(
        alter_ground.contains(&super::super::TreeDecoratorPlacement {
            pos: BlockPos {
                x: -1,
                y: 66,
                z: -1
            },
            state: "minecraft:rooted_dirt",
        })
    );
    assert!(
        alter_ground.contains(&super::super::TreeDecoratorPlacement {
            pos: BlockPos { x: 4, y: 66, z: 4 },
            state: "minecraft:podzol",
        })
    );
    assert!(
        alter_ground.contains(&super::super::TreeDecoratorPlacement {
            pos: BlockPos { x: 2, y: 61, z: 2 },
            state: "minecraft:coarse_dirt",
        })
    );
    assert!(
        !alter_ground.contains(&super::super::TreeDecoratorPlacement {
            pos: BlockPos {
                x: -3,
                y: 66,
                z: -3
            },
            state: "minecraft:moss_block",
        })
    );
    assert!(
        !alter_ground.contains(&super::super::TreeDecoratorPlacement {
            pos: BlockPos { x: 1, y: 62, z: -1 },
            state: "minecraft:coarse_dirt",
        })
    );
    assert!(super::super::alter_ground_decorator_placement(&[], &[], &[]).is_empty());
}

fn assert_attached_to_logs_decorator_support() {
    assert_eq!(
        super::super::attached_to_logs_decorator_placement(
            &[
                BlockPos {
                    x: 40,
                    y: 66,
                    z: 40
                },
                BlockPos {
                    x: 40,
                    y: 64,
                    z: 40
                },
                BlockPos {
                    x: 40,
                    y: 65,
                    z: 40
                },
            ],
            0.5,
            "minecraft:glow_lichen",
            &[2, 0, 1],
            &["north", "east", "south"],
            &[0.5, 0.49, 0.1],
            &[
                (
                    BlockPos {
                        x: 40,
                        y: 66,
                        z: 39
                    },
                    true,
                ),
                (
                    BlockPos {
                        x: 41,
                        y: 64,
                        z: 40
                    },
                    false,
                ),
                (
                    BlockPos {
                        x: 40,
                        y: 65,
                        z: 41
                    },
                    true,
                ),
            ],
        ),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 40,
                    y: 66,
                    z: 39
                },
                state: "minecraft:glow_lichen",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 40,
                    y: 65,
                    z: 41
                },
                state: "minecraft:glow_lichen",
            },
        ]
    );
}

fn assert_attached_to_leaves_decorator_validation() {
    assert_eq!(
        super::super::validate_attached_to_leaves_decorator_fields(16, 0, 1, 1),
        Ok(())
    );
    assert_eq!(
        super::super::validate_attached_to_leaves_decorator_fields(17, 0, 1, 1).unwrap_err(),
        "attached-to-leaves exclusion radii must be in 0..=16".to_string()
    );
    assert_eq!(
        super::super::validate_attached_to_leaves_decorator_fields(0, 0, 0, 1).unwrap_err(),
        "attached-to-leaves required_empty_blocks must be in 1..=16".to_string()
    );
    assert_eq!(
        super::super::validate_attached_to_leaves_decorator_fields(0, 0, 1, 0).unwrap_err(),
        "attached-to-leaves directions list must be non-empty".to_string()
    );
}

fn assert_attached_to_leaves_decorator_success() {
    assert_eq!(
        super::super::attached_to_leaves_decorator_placement(
            super::super::AttachedToLeavesDecoratorInput {
                leaves: &[
                    BlockPos {
                        x: 50,
                        y: 64,
                        z: 50
                    },
                    BlockPos {
                        x: 51,
                        y: 64,
                        z: 50
                    },
                    BlockPos {
                        x: 50,
                        y: 65,
                        z: 50
                    },
                ],
                probability: 0.5,
                exclusion_radius_xz: 1,
                exclusion_radius_y: 0,
                required_empty_blocks: 2,
                block_state: "minecraft:mangrove_propagule[hanging=true]",
                shuffled_leaf_indices: &[0, 1, 2],
                direction_choices: &["down", "down", "east"],
                probability_rolls: &[0.1, 0.1, 0.49],
                air_checks: &[
                    (
                        BlockPos {
                            x: 50,
                            y: 63,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 50,
                            y: 62,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 63,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 62,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 51,
                            y: 65,
                            z: 50
                        },
                        true,
                    ),
                    (
                        BlockPos {
                            x: 52,
                            y: 65,
                            z: 50
                        },
                        true,
                    ),
                ],
            },
        )
        .unwrap(),
        vec![
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 50,
                    y: 63,
                    z: 50
                },
                state: "minecraft:mangrove_propagule[hanging=true]",
            },
            super::super::TreeDecoratorPlacement {
                pos: BlockPos {
                    x: 51,
                    y: 65,
                    z: 50
                },
                state: "minecraft:mangrove_propagule[hanging=true]",
            },
        ]
    );
}

fn assert_attached_to_leaves_decorator_empty_when_probability_roll_fails() {
    assert!(super::super::attached_to_leaves_decorator_placement(
        super::super::AttachedToLeavesDecoratorInput {
            leaves: &[BlockPos { x: 0, y: 64, z: 0 }],
            probability: 0.5,
            exclusion_radius_xz: 0,
            exclusion_radius_y: 0,
            required_empty_blocks: 1,
            block_state: "minecraft:mangrove_propagule[hanging=true]",
            shuffled_leaf_indices: &[0],
            direction_choices: &["down"],
            probability_rolls: &[0.5],
            air_checks: &[(BlockPos { x: 0, y: 63, z: 0 }, true)],
        },
    )
    .unwrap()
    .is_empty());
}
