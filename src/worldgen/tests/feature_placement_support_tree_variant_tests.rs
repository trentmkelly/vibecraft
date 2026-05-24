use super::super::*;

pub(super) fn assert_tree_variant_and_trunk_support() {
        let bush_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 20,
                y: 64,
                z: 20,
            },
            TrunkPlacerModel {
                base_height: 3,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 1,
                offset_max: 1,
                kind: FoliagePlacerKind::Bush { height: 2 },
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            2,
            4,
        )
        .unwrap();
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 20,
                        y: 68,
                        z: 20,
                    }
        }));
        assert!(!bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 21,
                        y: 68,
                        z: 20,
                    }
        }));
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 21,
                        y: 67,
                        z: 20,
                    }
        }));
        assert!(!bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 22,
                        y: 67,
                        z: 20,
                    }
        }));
        assert!(bush_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 22,
                        y: 66,
                        z: 20,
                    }
        }));
        let acacia_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 40,
                y: 64,
                z: 40,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Acacia,
            },
            "minecraft:acacia_log",
            "minecraft:acacia_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 42,
                        y: 67,
                        z: 40,
                    }
        }));
        assert!(!acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 42,
                        y: 67,
                        z: 42,
                    }
        }));
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 41,
                        y: 68,
                        z: 41,
                    }
        }));
        assert!(acacia_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 41,
                        y: 68,
                        z: 40,
                    }
        }));
        let dark_oak_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 60,
                y: 64,
                z: 60,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::DarkOak,
            },
            "minecraft:dark_oak_log",
            "minecraft:dark_oak_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 64,
                        y: 67,
                        z: 60,
                    }
        }));
        assert!(!dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 64,
                        y: 67,
                        z: 64,
                    }
        }));
        assert!(dark_oak_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 63,
                        y: 68,
                        z: 63,
                    }
        }));
        let fancy_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 70,
                y: 64,
                z: 70,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Fancy { height: 2 },
            },
            "minecraft:oak_log",
            "minecraft:oak_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::super::fancy_foliage_rows(0, 2, 2),
            vec![(0, 2), (-1, 3), (-2, 2)]
        );
        assert!(fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 72,
                        y: 67,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 73,
                        y: 67,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 68,
                        y: 68,
                        z: 70,
                    }
        }));
        assert!(!fancy_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 72,
                        y: 68,
                        z: 70,
                    }
        }));
        assert!(super::super::fancy_leaves_row_should_skip(-2, 0, 2));
        assert!(super::super::fancy_leaves_row_should_skip(2, 0, 2));
        let mega_jungle_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 140,
                y: 64,
                z: 140,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Jungle { height: 4 },
            },
            "minecraft:jungle_log",
            "minecraft:jungle_leaves",
            "minecraft:dirt",
            0,
            1,
        )
        .unwrap();
        assert_eq!(
            super::super::mega_jungle_foliage_rows(0, 2, 2),
            vec![(0, 3), (-1, 4), (-2, 5)]
        );
        assert!(mega_jungle_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 145,
                        y: 66,
                        z: 140,
                    }
        }));
        assert!(!mega_jungle_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 145,
                        y: 66,
                        z: 145,
                    }
        }));
        let random_spread_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 160,
                y: 64,
                z: 160,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 3,
                radius_max: 3,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::RandomSpread {
                    foliage_height_min: 4,
                    foliage_height_max: 4,
                    leaf_placement_attempts: 4,
                },
            },
            "minecraft:azalea_log",
            "minecraft:azalea_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::super::random_spread_foliage_positions(
                BlockPos {
                    x: 10,
                    y: 20,
                    z: 30
                },
                4,
                3,
                2,
                &[2, 0, 3, 1, 1, 0, 0, 2, 0, 3, 0, 1],
            ),
            vec![
                BlockPos {
                    x: 12,
                    y: 22,
                    z: 31
                },
                BlockPos { x: 8, y: 17, z: 29 },
            ]
        );
        assert_eq!(
            random_spread_plan
                .blocks
                .iter()
                .filter(|block| block.kind == TreePlacementBlockKind::Leaves)
                .count(),
            4
        );
        let cherry_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 180,
                y: 64,
                z: 180,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 4,
                radius_max: 4,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Cherry {
                    height: 5,
                    wide_bottom_layer_hole_chance: 0.0,
                    corner_hole_chance: 0.0,
                    hanging_leaves_chance: 0.0,
                    hanging_leaves_extension_chance: 0.0,
                },
            },
            "minecraft:cherry_log",
            "minecraft:cherry_leaves",
            "minecraft:dirt",
            0,
            0,
        )
        .unwrap();
        assert_eq!(
            super::super::cherry_foliage_rows(5, 4),
            vec![(2, 1), (1, 2), (0, 3), (-1, 3), (-2, 2)]
        );
        assert!(cherry_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 183,
                        y: 68,
                        z: 182,
                    }
        }));
        assert!(!cherry_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 183,
                        y: 68,
                        z: 183,
                    }
        }));
        assert!(!super::super::cherry_leaves_row_should_skip(
            3, -1, 0, 3, 0.0, 0.0, 0, 0
        ));
        assert!(super::super::cherry_leaves_row_should_skip(
            3, -1, 0, 3, 1.0, 0.0, 0, 0
        ));
        let pine_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 80,
                y: 64,
                z: 80,
            },
            TrunkPlacerModel {
                base_height: 4,
                height_rand_a: 0,
                height_rand_b: 0,
                kind: TrunkPlacerKind::Straight,
            },
            FoliagePlacerModel {
                radius_min: 2,
                radius_max: 2,
                offset_min: 0,
                offset_max: 0,
                kind: FoliagePlacerKind::Pine {
                    height_min: 3,
                    height_max: 3,
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
            super::super::pine_foliage_rows(0, 3, 2),
            vec![(0, 0), (-1, 1), (-2, 2), (-3, 1)]
        );
        assert!(pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 82,
                        y: 66,
                        z: 80,
                    }
        }));
        assert!(!pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 82,
                        y: 66,
                        z: 82,
                    }
        }));
        assert!(pine_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 81,
                        y: 65,
                        z: 80,
                    }
        }));
        let spruce_plan = super::super::simple_tree_placement_plan(
            BlockPos {
                x: 100,
                y: 64,
                z: 100,
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
                kind: FoliagePlacerKind::Spruce {
                    height_min: 2,
                    height_max: 2,
                },
            },
            "minecraft:spruce_log",
            "minecraft:spruce_leaves",
            "minecraft:dirt",
            0,
            1,
        )
        .unwrap();
        assert_eq!(
            super::super::spruce_foliage_rows(0, 4, 2, 1),
            vec![(0, 1), (-1, 0), (-2, 1), (-3, 2), (-4, 1)]
        );
        assert!(spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 102,
                        y: 67,
                        z: 100,
                    }
        }));
        assert!(!spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 102,
                        y: 67,
                        z: 102,
                    }
        }));
        assert!(spruce_plan.blocks.iter().any(|block| {
            block.kind == TreePlacementBlockKind::Leaves
                && block.pos
                    == BlockPos {
                        x: 101,
                        y: 66,
                        z: 100,
                    }
        }));
        let mega_pine_plan = super::super::simple_tree_placement_plan(
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
        let forking_plan = super::super::forking_trunk_placement_plan(
            BlockPos {
                x: 200,
                y: 64,
                z: 200,
            },
            6,
            "minecraft:oak_log",
            "minecraft:dirt",
            HorizontalDirection::East,
            HorizontalDirection::North,
            1,
            1,
            0,
            2,
        );
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
            super::super::forking_trunk_placement_plan(
                BlockPos {
                    x: 200,
                    y: 64,
                    z: 200,
                },
                6,
                "minecraft:oak_log",
                "minecraft:dirt",
                HorizontalDirection::East,
                HorizontalDirection::East,
                1,
                1,
                0,
                2,
            )
            .attachments
            .len(),
            1
        );
        let bending_plan = super::super::bending_trunk_placement_plan(
            BlockPos {
                x: 220,
                y: 64,
                z: 220,
            },
            5,
            "minecraft:oak_log",
            "minecraft:dirt",
            HorizontalDirection::South,
            2,
            2,
            0,
        );
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
        let dark_oak_plan = super::super::dark_oak_trunk_placement_plan(
            BlockPos {
                x: 260,
                y: 64,
                z: 260,
            },
            6,
            "minecraft:dark_oak_log",
            "minecraft:dirt",
            HorizontalDirection::East,
            1,
            1,
            &[1, 1, 1, 1, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1],
        );
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
