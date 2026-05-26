fn assert_tree_variant_large_tree_support() {
    let mega_pine_plan = simple_tree_plan!(
        BlockPos {
            x: 120,
            y: 64,
            z: 120,
        },
        TrunkPlacerModel {
            base_height: 6,
            height_rand_a: 0,
            height_rand_b: 0,
            kind: TrunkPlacerKind::Straight,
        },
        FoliagePlacerModel {
            radius_min: 2,
            radius_max: 2,
            offset_min: 0,
            offset_max: 0,
            kind: FoliagePlacerKind::MegaPine {
                height_min: 6,
                height_max: 6,
            },
        },
        "minecraft:spruce_log",
        "minecraft:spruce_leaves",
        "minecraft:dirt",
        0,
        0,
    )
    .unwrap();
    assert_eq!(
        super::super::mega_pine_foliage_rows(70, 0, 6, 2),
        vec![(-6, 5), (-5, 4), (-4, 5), (-3, 3), (-2, 4), (-1, 2), (0, 2)]
    );
    assert!(mega_pine_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 124,
                    y: 68,
                    z: 120,
                }
    }));
    assert!(!mega_pine_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Leaves
            && block.pos
                == BlockPos {
                    x: 124,
                    y: 68,
                    z: 124,
                }
    }));
    assert!(!super::super::mega_pine_leaves_row_should_skip(4, 0, 4));
    assert!(super::super::mega_pine_leaves_row_should_skip(4, 4, 5));
    let forking_plan =
        super::super::forking_trunk_placement_plan(super::super::ForkingTrunkPlacementInput {
            origin: BlockPos {
                x: 200,
                y: 64,
                z: 200,
            },
            tree_height: 6,
            trunk_state: "minecraft:oak_log",
            below_trunk_state: "minecraft:dirt",
            lean_direction: HorizontalDirection::East,
            branch_direction: HorizontalDirection::North,
            lean_height_roll: 1,
            lean_steps_roll: 1,
            branch_pos_roll: 0,
            branch_steps_roll: 2,
        });
    assert!(forking_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 202,
                    y: 69,
                    z: 200,
                }
    }));
    assert!(forking_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 200,
                    y: 69,
                    z: 197,
                }
    }));
    assert_eq!(
        forking_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 202,
                    y: 70,
                    z: 200,
                },
                radius_offset: 1,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 200,
                    y: 70,
                    z: 197,
                },
                radius_offset: 0,
                double_trunk: false,
            },
        ]
    );
    assert_eq!(
        super::super::forking_trunk_placement_plan(super::super::ForkingTrunkPlacementInput {
            origin: BlockPos {
                x: 200,
                y: 64,
                z: 200,
            },
            tree_height: 6,
            trunk_state: "minecraft:oak_log",
            below_trunk_state: "minecraft:dirt",
            lean_direction: HorizontalDirection::East,
            branch_direction: HorizontalDirection::East,
            lean_height_roll: 1,
            lean_steps_roll: 1,
            branch_pos_roll: 0,
            branch_steps_roll: 2,
        },)
        .attachments
        .len(),
        1
    );
    let bending_plan =
        super::super::bending_trunk_placement_plan(super::super::BendingTrunkPlacementInput {
            origin: BlockPos {
                x: 220,
                y: 64,
                z: 220,
            },
            tree_height: 5,
            trunk_state: "minecraft:oak_log",
            below_trunk_state: "minecraft:dirt",
            direction: HorizontalDirection::South,
            min_height_for_leaves: 2,
            bend_length: 2,
            bend_start_roll: 0,
        });
    assert!(bending_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 220,
                    y: 67,
                    z: 221,
                }
    }));
    assert!(bending_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 220,
                    y: 69,
                    z: 224,
                }
    }));
    assert_eq!(
        bending_plan.attachments.first().copied(),
        Some(TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: 220,
                y: 66,
                z: 220,
            },
            radius_offset: 0,
            double_trunk: false,
        })
    );
    assert_eq!(
        bending_plan.attachments.last().copied(),
        Some(TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: 220,
                y: 69,
                z: 224,
            },
            radius_offset: 0,
            double_trunk: false,
        })
    );
    let giant_plan = super::super::giant_trunk_placement_plan(
        BlockPos {
            x: 240,
            y: 64,
            z: 240,
        },
        3,
        "minecraft:jungle_log",
        "minecraft:dirt",
    );
    assert_eq!(
        giant_plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk)
            .count(),
        4
    );
    assert_eq!(
        giant_plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Log)
            .count(),
        9
    );
    assert!(giant_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::DirtBelowTrunk
            && block.pos
                == BlockPos {
                    x: 241,
                    y: 63,
                    z: 241,
                }
    }));
    assert!(giant_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 241,
                    y: 65,
                    z: 241,
                }
    }));
    assert!(!giant_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 241,
                    y: 66,
                    z: 241,
                }
    }));
    assert_eq!(
        giant_plan.attachments,
        vec![TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: 240,
                y: 67,
                z: 240,
            },
            radius_offset: 0,
            double_trunk: true,
        }]
    );
    let mega_jungle_trunk_plan = super::super::mega_jungle_trunk_placement_plan(
        BlockPos {
            x: 250,
            y: 64,
            z: 250,
        },
        6,
        "minecraft:jungle_log",
        "minecraft:dirt",
        &[super::super::MegaJungleBranchModel {
            branch_height: 4,
            angle_radians: 0.0,
        }],
    );
    assert!(mega_jungle_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 255,
                    y: 67,
                    z: 251,
                }
    }));
    assert_eq!(
        mega_jungle_trunk_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 250,
                    y: 70,
                    z: 250,
                },
                radius_offset: 0,
                double_trunk: true,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 255,
                    y: 68,
                    z: 251,
                },
                radius_offset: -2,
                double_trunk: false,
            },
        ]
    );
    let dark_oak_plan =
        super::super::dark_oak_trunk_placement_plan(super::super::DarkOakTrunkPlacementInput {
            origin: BlockPos {
                x: 260,
                y: 64,
                z: 260,
            },
            tree_height: 6,
            trunk_state: "minecraft:dark_oak_log",
            below_trunk_state: "minecraft:dirt",
            lean_direction: HorizontalDirection::East,
            lean_height_roll: 1,
            lean_steps_roll: 1,
            branch_rolls: &[1, 1, 1, 1, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1],
        });
    assert_eq!(
        dark_oak_plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::DirtBelowTrunk)
            .count(),
        4
    );
    assert!(dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 262,
                    y: 69,
                    z: 261,
                }
    }));
    assert!(dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 260,
                    y: 68,
                    z: 259,
                }
    }));
    assert!(dark_oak_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 260,
                    y: 65,
                    z: 259,
                }
    }));
    assert_eq!(
        dark_oak_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 261,
                    y: 69,
                    z: 260,
                },
                radius_offset: 0,
                double_trunk: true,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 260,
                    y: 69,
                    z: 259,
                },
                radius_offset: 0,
                double_trunk: false,
            },
        ]
    );
    let upwards_branching_plan = super::super::upwards_branching_trunk_placement_plan(
        BlockPos {
            x: 280,
            y: 64,
            z: 280,
        },
        5,
        "minecraft:spruce_log",
        &[super::super::UpwardsBranchingBranchModel {
            trunk_y_offset: 1,
            direction: HorizontalDirection::North,
            branch_pos: 1,
            branch_steps: 3,
        }],
    );
    assert_eq!(
        upwards_branching_plan
            .blocks
            .iter()
            .filter(|block| block.kind == TreePlacementBlockKind::Log)
            .count(),
        8
    );
    assert!(upwards_branching_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.pos
                == BlockPos {
                    x: 280,
                    y: 67,
                    z: 278,
                }
    }));
    assert_eq!(
        upwards_branching_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 66,
                    z: 279,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 67,
                    z: 278,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 68,
                    z: 277,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 69,
                    z: 277,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 67,
                    z: 277,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 280,
                    y: 69,
                    z: 280,
                },
                radius_offset: 0,
                double_trunk: false,
            },
        ]
    );
    let cherry_trunk_plan = super::super::cherry_trunk_placement_plan(
        BlockPos {
            x: 300,
            y: 64,
            z: 300,
        },
        6,
        "minecraft:cherry_log",
        "minecraft:dirt",
        2,
        &[
            super::super::CherryBranchModel {
                start_offset_from_origin: 3,
                direction: HorizontalDirection::East,
                horizontal_length: 2,
                end_offset_from_origin: 5,
                middle_continues_upwards: false,
                grow_vertically: vec![false, true, true],
            },
            super::super::CherryBranchModel {
                start_offset_from_origin: 2,
                direction: HorizontalDirection::West,
                horizontal_length: 2,
                end_offset_from_origin: 1,
                middle_continues_upwards: true,
                grow_vertically: vec![true, false],
            },
        ],
    );
    assert!(cherry_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::DirtBelowTrunk
            && block.pos
                == BlockPos {
                    x: 300,
                    y: 63,
                    z: 300,
                }
    }));
    assert!(cherry_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.state == "minecraft:cherry_log[axis=x]"
            && block.pos
                == BlockPos {
                    x: 302,
                    y: 67,
                    z: 300,
                }
    }));
    assert!(cherry_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.state == "minecraft:cherry_log"
            && block.pos
                == BlockPos {
                    x: 302,
                    y: 69,
                    z: 300,
                }
    }));
    assert_eq!(
        cherry_trunk_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 302,
                    y: 70,
                    z: 300,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 297,
                    y: 66,
                    z: 300,
                },
                radius_offset: 0,
                double_trunk: false,
            },
        ]
    );
    let fancy_trunk_plan = super::super::fancy_trunk_placement_plan(
        BlockPos {
            x: 320,
            y: 64,
            z: 320,
        },
        8,
        "minecraft:oak_log",
        "minecraft:dirt",
        &[super::super::FancyTrunkClusterRollModel {
            shape_float: 0.5,
            angle_float: 0.25,
        }],
    );
    assert_eq!(super::super::fancy_trunk_tree_shape(10, 2), -1.0);
    assert_eq!(super::super::fancy_trunk_tree_shape(10, 0), -1.0);
    assert_eq!(super::super::fancy_trunk_cluster_roll_count(8), 3);
    assert!(fancy_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::DirtBelowTrunk
            && block.pos
                == BlockPos {
                    x: 320,
                    y: 63,
                    z: 320,
                }
    }));
    assert!(fancy_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.state == "minecraft:oak_log"
            && block.pos
                == BlockPos {
                    x: 320,
                    y: 70,
                    z: 320,
                }
    }));
    assert!(fancy_trunk_plan.blocks.iter().any(|block| {
        block.kind == TreePlacementBlockKind::Log
            && block.state == "minecraft:oak_log[axis=x]"
            && block.pos
                == BlockPos {
                    x: 322,
                    y: 68,
                    z: 320,
                }
    }));
    assert_eq!(
        fancy_trunk_plan.attachments,
        vec![
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 320,
                    y: 69,
                    z: 320,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 322,
                    y: 68,
                    z: 320,
                },
                radius_offset: 0,
                double_trunk: false,
            },
            TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: 320,
                    y: 67,
                    z: 321,
                },
                radius_offset: 0,
                double_trunk: false,
            },
        ]
    );
}
