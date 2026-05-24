use super::*;

    #[test]
    fn structure_piece_bounding_box_and_world_coordinates_match_vanilla_orientation() {
        assert_eq!(
            super::super::structure_make_bounding_box(
                10,
                20,
                30,
                super::super::HorizontalDirection::South,
                3,
                4,
                5
            ),
            super::super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 12,
                max_y: 23,
                max_z: 34,
            }
        );
        assert_eq!(
            super::super::structure_make_bounding_box(
                10,
                20,
                30,
                super::super::HorizontalDirection::East,
                3,
                4,
                5
            ),
            super::super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 20,
                min_z: 30,
                max_x: 14,
                max_y: 23,
                max_z: 32,
            }
        );

        let foot = BlockPos {
            x: 100,
            y: 40,
            z: 200,
        };
        let offset = BlockPos { x: 2, y: 3, z: 4 };
        assert_eq!(
            super::super::structure_orient_box(foot, offset, 5, 6, 7, super::super::HorizontalDirection::North),
            super::super::StructureBoundingBoxModel {
                min_x: 102,
                min_y: 43,
                min_z: 198,
                max_x: 106,
                max_y: 48,
                max_z: 204,
            }
        );
        assert_eq!(
            super::super::structure_orient_box(foot, offset, 5, 6, 7, super::super::HorizontalDirection::West),
            super::super::StructureBoundingBoxModel {
                min_x: 98,
                min_y: 43,
                min_z: 202,
                max_x: 104,
                max_y: 48,
                max_z: 206,
            }
        );
        assert_eq!(
            super::super::structure_orient_box(foot, offset, 5, 6, 7, super::super::HorizontalDirection::East),
            super::super::StructureBoundingBoxModel {
                min_x: 104,
                min_y: 43,
                min_z: 202,
                max_x: 110,
                max_y: 48,
                max_z: 206,
            }
        );

        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 50,
            min_y: 60,
            min_z: 70,
            max_x: 59,
            max_y: 69,
            max_z: 79,
        };
        assert_eq!(
            super::super::structure_piece_world_pos(bounding_box, None, 1, 2, 3),
            BlockPos { x: 1, y: 2, z: 3 }
        );
        assert_eq!(
            super::super::structure_piece_world_pos(
                bounding_box,
                Some(super::super::HorizontalDirection::North),
                1,
                2,
                3
            ),
            BlockPos {
                x: 51,
                y: 62,
                z: 76,
            }
        );
        assert_eq!(
            super::super::structure_piece_world_pos(
                bounding_box,
                Some(super::super::HorizontalDirection::West),
                1,
                2,
                3
            ),
            BlockPos {
                x: 56,
                y: 62,
                z: 71,
            }
        );
        assert_eq!(
            super::super::structure_piece_world_pos(
                bounding_box,
                Some(super::super::HorizontalDirection::East),
                1,
                2,
                3
            ),
            BlockPos {
                x: 53,
                y: 62,
                z: 71,
            }
        );
    }

    #[test]
    fn mineshaft_corridor_ceiling_uses_java_maybe_box_probability() {
        let chunk_bb = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 15,
            max_y: 15,
            max_z: 15,
        };
        let piece_box = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 2,
            max_y: 2,
            max_z: 4,
        };
        let rolls = [
            0.0, 0.79, 0.8, 0.81, 0.2, 0.95, 0.1, 0.7, 0.99, 0.3, 0.4, 0.5, 0.6, 0.85, 0.75,
        ];

        let blocks = super::super::structure_piece_generate_maybe_box(
            piece_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            &rolls,
            0.8,
            BlockPos { x: 0, y: 2, z: 0 },
            BlockPos { x: 2, y: 2, z: 4 },
            "minecraft:cave_air",
            "minecraft:cave_air",
            false,
            false,
            |_| false,
            |_| true,
        );

        assert_eq!(
            blocks.len(),
            rolls.iter().filter(|roll| **roll <= 0.8).count(),
            "Java StructurePiece#generateMaybeBox places when random.nextFloat() <= probability"
        );
        assert!(!blocks
            .iter()
            .any(|block| block.world_pos == BlockPos { x: 0, y: 2, z: 3 }));
    }

    #[test]
    fn structure_piece_chunk_proximity_and_locator_position_match_vanilla() {
        let even_sized_piece = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 32,
                min_y: 20,
                min_z: -16,
                max_x: 47,
                max_y: 35,
                max_z: -1,
            },
        };
        assert_eq!(
            super::super::structure_piece_locator_position(even_sized_piece),
            BlockPos {
                x: 40,
                y: 28,
                z: -8,
            }
        );

        let piece = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 20,
                min_y: -10,
                min_z: 20,
                max_x: 25,
                max_y: 120,
                max_z: 25,
            },
        };
        assert!(super::super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 1, z: 1 },
            0
        ));
        assert!(!super::super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            0
        ));
        assert!(super::super::structure_piece_is_close_to_chunk(
            piece,
            ChunkPos { x: 2, z: 1 },
            7
        ));
    }

    #[test]
    fn structure_piece_box_generation_uses_vanilla_loop_edges_and_chunk_clipping() {
        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let chunk_bb = super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: 20,
            min_z: 30,
            max_x: 12,
            max_y: 22,
            max_z: 32,
        };
        let blocks = super::super::structure_piece_generate_box(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 2, y: 2, z: 2 },
            "minecraft:cobblestone",
            "minecraft:mossy_cobblestone",
            false,
            |_| false,
        );

        assert_eq!(blocks.len(), 18);
        assert_eq!(
            blocks.first().copied(),
            Some(super::super::StructurePiecePlacementBlock {
                local_pos: BlockPos { x: 1, y: 0, z: 0 },
                world_pos: BlockPos {
                    x: 11,
                    y: 20,
                    z: 30,
                },
                state: "minecraft:cobblestone",
                edge: true,
            })
        );
        assert!(blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 1, y: 1, z: 1 }
                && block.world_pos
                    == BlockPos {
                        x: 11,
                        y: 21,
                        z: 31,
                    }
                && block.state == "minecraft:mossy_cobblestone"
                && !block.edge
        }));
        assert!(blocks
            .iter()
            .all(|block| chunk_bb.is_inside(block.world_pos)));
    }

    #[test]
    fn structure_piece_air_box_and_skip_air_match_vanilla_generation_rules() {
        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 10,
            max_y: 74,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let air_blocks = super::super::structure_piece_generate_air_box(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
        );
        assert_eq!(air_blocks.len(), 8);
        assert!(air_blocks
            .iter()
            .all(|block| block.state == "minecraft:air" && block.edge));

        let skipped_world_pos = BlockPos { x: 1, y: 66, z: 3 };
        let blocks = super::super::structure_piece_generate_box(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 2, y: 3, z: 4 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            |world_pos| world_pos == skipped_world_pos,
        );
        assert_eq!(blocks.len(), 7);
        assert!(!blocks
            .iter()
            .any(|block| block.world_pos == skipped_world_pos));
    }

    #[test]
    fn structure_piece_maybe_box_and_single_block_use_vanilla_probability_edges() {
        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 10,
            min_z: 0,
            max_x: 10,
            max_y: 20,
            max_z: 10,
        };
        let chunk_bb = bounding_box;
        let blocks = super::super::structure_piece_generate_maybe_box(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            &[0.75, 0.76, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60],
            0.75,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 1, y: 1, z: 1 },
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            true,
            true,
            |world_pos| world_pos == BlockPos { x: 0, y: 10, z: 0 },
            |world_pos| world_pos.y >= 11,
        );

        assert_eq!(blocks.len(), 4);
        assert!(blocks
            .iter()
            .all(|block| block.state == "minecraft:stone_bricks" && block.edge));
        assert!(!blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 0, y: 0, z: 0 }
                || block.local_pos == BlockPos { x: 0, y: 0, z: 1 }
                || block.local_pos == BlockPos { x: 1, y: 0, z: 0 }
        }));

        assert_eq!(
            super::super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::super::HorizontalDirection::South),
                chunk_bb,
                0.49,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            )
            .map(|block| block.world_pos),
            Some(BlockPos { x: 2, y: 13, z: 4 })
        );
        assert_eq!(
            super::super::structure_piece_maybe_generate_block(
                bounding_box,
                Some(super::super::HorizontalDirection::South),
                chunk_bb,
                0.5,
                0.5,
                BlockPos { x: 2, y: 3, z: 4 },
                "minecraft:lantern"
            ),
            None
        );
    }

    #[test]
    fn structure_piece_upper_half_sphere_matches_vanilla_shape_and_clipping() {
        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let full_chunk = super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 20,
            min_z: 30,
            max_x: 20,
            max_y: 30,
            max_z: 40,
        };
        let blocks = super::super::structure_piece_generate_upper_half_sphere(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            full_chunk,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 4, y: 4, z: 4 },
            "minecraft:smooth_sandstone",
            false,
            |_| false,
        );

        assert_eq!(blocks.len(), 76);
        assert!(blocks.iter().any(|block| {
            block.local_pos == BlockPos { x: 2, y: 0, z: 2 }
                && block.world_pos
                    == BlockPos {
                        x: 12,
                        y: 20,
                        z: 32,
                    }
        }));
        assert!(!blocks
            .iter()
            .any(|block| block.local_pos == BlockPos { x: 0, y: 0, z: 0 }));
        assert!(blocks
            .iter()
            .all(|block| block.state == "minecraft:smooth_sandstone" && !block.edge));

        let clipped_chunk = super::super::StructureBoundingBoxModel {
            min_x: 12,
            min_y: 20,
            min_z: 32,
            max_x: 14,
            max_y: 24,
            max_z: 34,
        };
        let skipped = BlockPos {
            x: 12,
            y: 20,
            z: 32,
        };
        let clipped = super::super::structure_piece_generate_upper_half_sphere(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            clipped_chunk,
            BlockPos { x: 0, y: 0, z: 0 },
            BlockPos { x: 4, y: 4, z: 4 },
            "minecraft:smooth_sandstone",
            true,
            |world_pos| world_pos == skipped,
        );
        assert!(clipped.len() < blocks.len());
        assert!(clipped
            .iter()
            .all(|block| clipped_chunk.is_inside(block.world_pos)));
        assert!(!clipped.iter().any(|block| block.world_pos == skipped));
    }

    #[test]
    fn structure_piece_fill_column_down_matches_vanilla_replaceable_loop() {
        assert!(super::super::structure_piece_is_replaceable_by_structures(
            "minecraft:air"
        ));
        assert!(super::super::structure_piece_is_replaceable_by_structures(
            "minecraft:water"
        ));
        assert!(super::super::structure_piece_is_replaceable_by_structures(
            "minecraft:glow_lichen"
        ));
        assert!(super::super::structure_piece_is_replaceable_by_structures(
            "minecraft:tall_seagrass[half=upper]"
        ));
        assert!(!super::super::structure_piece_is_replaceable_by_structures(
            "minecraft:stone"
        ));

        let bounding_box = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 15,
            max_y: 80,
            max_z: 15,
        };
        let chunk_bb = super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: -64,
            min_z: 0,
            max_x: 15,
            max_y: 320,
            max_z: 15,
        };
        let blocks = super::super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            3,
            4,
            5,
            64,
            "minecraft:sandstone",
            |pos| {
                if pos.y >= 66 {
                    "minecraft:air"
                } else {
                    "minecraft:stone"
                }
            },
        );
        assert_eq!(
            blocks
                .iter()
                .map(|block| block.world_pos)
                .collect::<Vec<_>>(),
            vec![
                BlockPos { x: 3, y: 68, z: 5 },
                BlockPos { x: 3, y: 67, z: 5 },
                BlockPos { x: 3, y: 66, z: 5 },
            ]
        );

        let min_limited = super::super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            chunk_bb,
            3,
            2,
            5,
            64,
            "minecraft:sandstone",
            |_| "minecraft:air",
        );
        assert_eq!(min_limited.len(), 1);
        assert_eq!(min_limited[0].world_pos.y, 66);

        let outside_chunk = super::super::structure_piece_fill_column_down(
            bounding_box,
            Some(super::super::HorizontalDirection::South),
            super::super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: -64,
                min_z: 10,
                max_x: 15,
                max_y: 320,
                max_z: 15,
            },
            3,
            4,
            5,
            64,
            "minecraft:sandstone",
            |_| "minecraft:air",
        );
        assert!(outside_chunk.is_empty());
    }

    #[test]
    fn structure_piece_reorient_facing_matches_vanilla_neighbor_rules() {
        use super::super::HorizontalDirection::{East, North, South, West};

        assert_eq!(
            super::super::structure_piece_reorient_facing(
                North,
                &[super::super::StructurePieceNeighborState {
                    direction: East,
                    chest: true,
                    solid_render: true,
                }]
            ),
            North
        );
        assert_eq!(
            super::super::structure_piece_reorient_facing(
                North,
                &[super::super::StructurePieceNeighborState {
                    direction: West,
                    chest: false,
                    solid_render: true,
                }]
            ),
            East
        );
        assert_eq!(
            super::super::structure_piece_reorient_facing(
                North,
                &[
                    super::super::StructurePieceNeighborState {
                        direction: West,
                        chest: false,
                        solid_render: true,
                    },
                    super::super::StructurePieceNeighborState {
                        direction: East,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            North
        );
        assert_eq!(
            super::super::structure_piece_reorient_facing(
                North,
                &[
                    super::super::StructurePieceNeighborState {
                        direction: North,
                        chest: false,
                        solid_render: true,
                    },
                    super::super::StructurePieceNeighborState {
                        direction: South,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            West
        );
        assert_eq!(
            super::super::structure_piece_reorient_facing(
                North,
                &[
                    super::super::StructurePieceNeighborState {
                        direction: North,
                        chest: false,
                        solid_render: true,
                    },
                    super::super::StructurePieceNeighborState {
                        direction: South,
                        chest: false,
                        solid_render: true,
                    },
                    super::super::StructurePieceNeighborState {
                        direction: West,
                        chest: false,
                        solid_render: true,
                    },
                ]
            ),
            East
        );
    }

    #[test]
    fn structure_piece_move_aggregate_box_and_collision_match_vanilla_helpers() {
        let first = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 10,
                min_z: 0,
                max_x: 4,
                max_y: 14,
                max_z: 4,
            },
        };
        let second = super::super::StructurePieceModel {
            bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 10,
                min_y: 8,
                min_z: -2,
                max_x: 12,
                max_y: 18,
                max_z: 2,
            },
        };

        assert_eq!(
            super::super::structure_piece_moved(first, 3, -2, 5),
            super::super::StructurePieceModel {
                bounding_box: super::super::StructureBoundingBoxModel {
                    min_x: 3,
                    min_y: 8,
                    min_z: 5,
                    max_x: 7,
                    max_y: 12,
                    max_z: 9,
                },
            }
        );
        assert_eq!(
            super::super::structure_piece_create_bounding_box(&[first, second]),
            Ok(super::super::StructureBoundingBoxModel {
                min_x: 0,
                min_y: 8,
                min_z: -2,
                max_x: 12,
                max_y: 18,
                max_z: 4,
            })
        );
        assert_eq!(
            super::super::structure_piece_create_bounding_box(&[]),
            Err("Unable to calculate boundingbox without pieces".to_string())
        );
        assert_eq!(
            super::super::structure_piece_find_collision_piece(
                &[first, second],
                super::super::StructureBoundingBoxModel {
                    min_x: 11,
                    min_y: 0,
                    min_z: 0,
                    max_x: 20,
                    max_y: 20,
                    max_z: 10,
                },
            ),
            Some(second)
        );
        assert_eq!(
            super::super::structure_piece_find_collision_piece(
                &[first, second],
                super::super::StructureBoundingBoxModel {
                    min_x: 5,
                    min_y: 0,
                    min_z: 5,
                    max_x: 9,
                    max_y: 20,
                    max_z: 9,
                },
            ),
            None
        );
    }

    #[test]
    fn structure_piece_orientation_sets_mirror_and_rotation_like_vanilla() {
        use super::super::HorizontalDirection::{East, North, South, West};

        assert_eq!(
            super::super::structure_piece_orientation_state(None),
            super::super::StructurePieceOrientationState {
                orientation: None,
                mirror: super::super::StructurePieceMirror::None,
                rotation: super::super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::super::structure_piece_orientation_state(Some(North)),
            super::super::StructurePieceOrientationState {
                orientation: Some(North),
                mirror: super::super::StructurePieceMirror::None,
                rotation: super::super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::super::structure_piece_orientation_state(Some(South)),
            super::super::StructurePieceOrientationState {
                orientation: Some(South),
                mirror: super::super::StructurePieceMirror::LeftRight,
                rotation: super::super::StructurePieceRotation::None,
            }
        );
        assert_eq!(
            super::super::structure_piece_orientation_state(Some(West)),
            super::super::StructurePieceOrientationState {
                orientation: Some(West),
                mirror: super::super::StructurePieceMirror::LeftRight,
                rotation: super::super::StructurePieceRotation::Clockwise90,
            }
        );
        assert_eq!(
            super::super::structure_piece_orientation_state(Some(East)),
            super::super::StructurePieceOrientationState {
                orientation: Some(East),
                mirror: super::super::StructurePieceMirror::None,
                rotation: super::super::StructurePieceRotation::Clockwise90,
            }
        );
    }
