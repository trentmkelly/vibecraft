use super::*;

#[test]
fn desert_pyramid_archaeology_state_matches_after_place_rules() {
    assert_desert_pyramid_piece_metadata_and_save();
    assert_desert_pyramid_suspicious_sand_positions();
    assert_desert_pyramid_after_place_outputs();
}

fn desert_pyramid_test_piece() -> super::super::DesertPyramidPieceModel {
    super::super::desert_pyramid_generation_piece(
        ChunkPos { x: 1, z: -1 },
        super::super::HorizontalDirection::South,
    )
}

fn desert_pyramid_piece_with_suspicious_sand() -> super::super::DesertPyramidPieceModel {
    let mut piece = desert_pyramid_test_piece();
    super::super::desert_pyramid_place_sand_box(
        &mut piece,
        BlockPos {
            x: 14,
            y: -3,
            z: 11,
        },
        BlockPos {
            x: 15,
            y: -2,
            z: 12,
        },
    );
    piece
}

fn assert_desert_pyramid_piece_metadata_and_save() {
    let mut piece = desert_pyramid_test_piece();
    assert_eq!(piece.scattered.width, 21);
    assert_eq!(piece.scattered.height, 15);
    assert_eq!(piece.scattered.depth, 21);
    assert_eq!(
        piece.scattered.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: 16,
            min_y: 64,
            min_z: -16,
            max_x: 36,
            max_y: 78,
            max_z: 4,
        }
    );
    piece.has_placed_chest[2] = true;
    assert_eq!(
        super::super::desert_pyramid_save_tag(&piece),
        super::super::DesertPyramidSaveTagModel {
            width: 21,
            height: 15,
            depth: 21,
            height_position: -1,
            has_placed_chest: [false, false, true, false],
        }
    );
}

fn assert_desert_pyramid_suspicious_sand_positions() {
    let mut piece = desert_pyramid_piece_with_suspicious_sand();
    let duplicate = super::super::desert_pyramid_place_sand(&mut piece, 14, -3, 11);
    assert_eq!(
        duplicate,
        BlockPos {
            x: 30,
            y: 61,
            z: -5
        }
    );
    assert_eq!(piece.potential_suspicious_sand_world_positions.len(), 9);
    assert_eq!(
        super::super::desert_pyramid_record_collapsed_roof(
            &mut piece,
            BlockPos { x: 14, y: 0, z: 11 },
            18,
            15,
            16,
            13,
        )
        .unwrap(),
        BlockPos {
            x: 32,
            y: 64,
            z: -3
        }
    );
    assert_eq!(
        super::super::desert_pyramid_record_collapsed_roof(
            &mut piece.clone(),
            BlockPos { x: 14, y: 0, z: 11 },
            18,
            15,
            19,
            13,
        )
        .unwrap_err(),
        "Collapsed roof random position must be inside the requested local range".to_string()
    );

    let unique = super::super::desert_pyramid_unique_suspicious_sand_positions(&[piece.clone()]);
    assert_eq!(unique.len(), 8);
    assert_eq!(
        unique[0],
        BlockPos {
            x: 30,
            y: 61,
            z: -5
        }
    );
    assert_eq!(
        unique[1],
        BlockPos {
            x: 31,
            y: 61,
            z: -5
        }
    );
    assert_eq!(
        unique[2],
        BlockPos {
            x: 30,
            y: 61,
            z: -4
        }
    );
}

fn assert_desert_pyramid_after_place_outputs() {
    let mut piece = desert_pyramid_piece_with_suspicious_sand();
    assert_eq!(
        super::super::desert_pyramid_record_collapsed_roof(
            &mut piece,
            BlockPos { x: 14, y: 0, z: 11 },
            18,
            15,
            16,
            13,
        )
        .unwrap(),
        BlockPos {
            x: 32,
            y: 64,
            z: -3
        }
    );
    let unique =
        super::super::desert_pyramid_unique_suspicious_sand_positions(std::slice::from_ref(&piece));
    let chunk_bb = super::super::StructureBoundingBoxModel {
        min_x: 16,
        min_y: i32::MIN,
        min_z: -16,
        max_x: 36,
        max_y: i32::MAX,
        max_z: 4,
    };
    let shuffled = vec![unique[3], unique[0], unique[7]];
    let placements =
        super::super::desert_pyramid_after_place_archaeology(&[piece], chunk_bb, &shuffled, 2);
    assert_eq!(
        placements[0].pos,
        BlockPos {
            x: 32,
            y: 64,
            z: -3
        }
    );
    assert_eq!(placements[0].state, "minecraft:suspicious_sand");
    assert_eq!(
        placements[0].loot_table,
        Some("minecraft:archaeology/desert_pyramid")
    );
    assert_eq!(
        placements[0].loot_seed,
        Some(super::super::block_pos_as_long(BlockPos {
            x: 32,
            y: 64,
            z: -3
        }))
    );
    assert_eq!(placements[1].pos, unique[3]);
    assert_eq!(placements[1].state, "minecraft:suspicious_sand");
    assert_eq!(placements[2].pos, unique[0]);
    assert_eq!(placements[2].state, "minecraft:suspicious_sand");
    assert_eq!(placements[3].pos, unique[7]);
    assert_eq!(placements[3].state, "minecraft:sand");
}

#[test]
fn jungle_temple_piece_container_flags_match_vanilla() {
    assert_jungle_temple_piece_metadata_and_save();
    assert_jungle_temple_full_post_process_flags();
    assert_jungle_temple_partial_and_outside_chunk_gates();
}

fn jungle_temple_test_piece() -> super::super::JungleTemplePieceModel {
    super::super::jungle_temple_generation_piece(
        ChunkPos { x: 0, z: 0 },
        super::super::HorizontalDirection::South,
    )
}

fn jungle_temple_full_chunk() -> super::super::StructureBoundingBoxModel {
    super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: i32::MIN,
        min_z: 0,
        max_x: 15,
        max_y: i32::MAX,
        max_z: 15,
    }
}

fn assert_jungle_temple_piece_metadata_and_save() {
    let piece = jungle_temple_test_piece();
    assert_eq!(piece.scattered.width, 12);
    assert_eq!(piece.scattered.height, 10);
    assert_eq!(piece.scattered.depth, 15);
    assert_eq!(
        piece.scattered.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 64,
            min_z: 0,
            max_x: 11,
            max_y: 73,
            max_z: 14,
        }
    );
    assert_eq!(
        super::super::jungle_temple_save_tag(&piece),
        super::super::JungleTempleSaveTagModel {
            width: 12,
            height: 10,
            depth: 15,
            height_position: -1,
            placed_main_chest: false,
            placed_hidden_chest: false,
            placed_trap1: false,
            placed_trap2: false,
        }
    );
}

fn assert_jungle_temple_full_post_process_flags() {
    let piece = jungle_temple_test_piece();
    let chunk_bb = jungle_temple_full_chunk();
    let processed =
        super::super::jungle_temple_post_process(piece.clone(), chunk_bb, |_, _| 70).unwrap();
    assert_eq!(processed.piece.scattered.height_position, 70);
    assert_eq!(processed.piece.scattered.bounding_box.min_y, 70);
    assert!(processed.piece.placed_trap1);
    assert!(processed.piece.placed_trap2);
    assert!(processed.piece.placed_main_chest);
    assert!(processed.piece.placed_hidden_chest);
    assert_eq!(
        processed.containers,
        vec![
            super::super::JungleTempleContainerPlacement {
                kind: "minecraft:dispenser",
                pos: BlockPos { x: 3, y: 68, z: 1 },
                facing: Some(super::super::HorizontalDirection::North),
                loot_table: "minecraft:chests/jungle_temple_dispenser",
            },
            super::super::JungleTempleContainerPlacement {
                kind: "minecraft:dispenser",
                pos: BlockPos { x: 9, y: 68, z: 3 },
                facing: Some(super::super::HorizontalDirection::West),
                loot_table: "minecraft:chests/jungle_temple_dispenser",
            },
            super::super::JungleTempleContainerPlacement {
                kind: "minecraft:chest",
                pos: BlockPos { x: 8, y: 67, z: 3 },
                facing: None,
                loot_table: "minecraft:chests/jungle_temple",
            },
            super::super::JungleTempleContainerPlacement {
                kind: "minecraft:chest",
                pos: BlockPos { x: 9, y: 67, z: 10 },
                facing: None,
                loot_table: "minecraft:chests/jungle_temple",
            },
        ]
    );

    let no_repeat =
        super::super::jungle_temple_post_process(processed.piece.clone(), chunk_bb, |_, _| 70)
            .unwrap();
    assert!(no_repeat.containers.is_empty());
}

fn assert_jungle_temple_partial_and_outside_chunk_gates() {
    let piece = jungle_temple_test_piece();
    let partial_chunk = super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: i32::MIN,
        min_z: 0,
        max_x: 5,
        max_y: i32::MAX,
        max_z: 5,
    };
    let partial =
        super::super::jungle_temple_post_process(piece.clone(), partial_chunk, |_, _| 70).unwrap();
    assert_eq!(partial.containers.len(), 1);
    assert_eq!(partial.containers[0].pos, BlockPos { x: 3, y: 68, z: 1 });
    assert!(partial.piece.placed_trap1);
    assert!(!partial.piece.placed_trap2);
    assert!(!partial.piece.placed_main_chest);
    assert!(!partial.piece.placed_hidden_chest);

    let outside_chunk = super::super::StructureBoundingBoxModel {
        min_x: 100,
        min_y: i32::MIN,
        min_z: 100,
        max_x: 115,
        max_y: i32::MAX,
        max_z: 115,
    };
    assert!(super::super::jungle_temple_post_process(piece, outside_chunk, |_, _| 70).is_none());
}

#[test]
fn igloo_template_piece_generation_and_surface_shift_match_vanilla() {
    assert_igloo_basement_piece_stack();
    assert_igloo_top_surface_shift_and_trapdoor();
    assert_igloo_no_basement_and_roll_errors();
}

fn igloo_basement_pieces() -> Vec<super::super::IglooPieceModel> {
    super::super::igloo_generation_pieces(
        ChunkPos { x: 2, z: -3 },
        super::super::StructureRotation::Clockwise90,
        0.25,
        2,
    )
    .unwrap()
}

fn assert_igloo_basement_piece_stack() {
    let pieces = igloo_basement_pieces();
    assert_eq!(pieces.len(), 7);
    assert_eq!(pieces[0].template, super::super::IglooTemplateKind::Bottom);
    assert_eq!(pieces[0].template_name, "minecraft:igloo/bottom");
    assert_eq!(pieces[0].depth, 18);
    assert_eq!(pieces[0].offset, BlockPos { x: 0, y: -3, z: -2 });
    assert_eq!(pieces[0].pivot, BlockPos { x: 3, y: 6, z: 7 });
    assert_eq!(
        pieces[0].template_position,
        BlockPos {
            x: 32,
            y: 69,
            z: -50
        }
    );
    assert_eq!(pieces[1].template, super::super::IglooTemplateKind::Middle);
    assert_eq!(pieces[1].depth, 0);
    assert_eq!(
        pieces[1].template_position,
        BlockPos {
            x: 34,
            y: 87,
            z: -44
        }
    );
    assert_eq!(pieces[5].template, super::super::IglooTemplateKind::Middle);
    assert_eq!(pieces[5].depth, 12);
    assert_eq!(pieces[6].template, super::super::IglooTemplateKind::Top);
    assert_eq!(
        pieces[6].template_position,
        BlockPos {
            x: 32,
            y: 90,
            z: -48
        }
    );
}

fn assert_igloo_top_surface_shift_and_trapdoor() {
    let top = igloo_basement_pieces()[6];
    let processed = super::super::igloo_post_process(
        top,
        |x, z| 75 + (x == 40 && z == -43) as i32,
        |_| "minecraft:snow_block",
    );
    assert_eq!(
        processed.entrance_pos,
        BlockPos {
            x: 40,
            y: 90,
            z: -43
        }
    );
    assert_eq!(
        processed.adjusted_template_position,
        BlockPos {
            x: 32,
            y: 75,
            z: -48
        }
    );
    assert_eq!(
        processed.trapdoor_pos,
        Some(BlockPos {
            x: 35,
            y: 75,
            z: -43
        })
    );
    assert!(processed.should_cover_trapdoor);

    let ladder_below = super::super::igloo_post_process(top, |_, _| 76, |_| "minecraft:ladder");
    assert!(!ladder_below.should_cover_trapdoor);
}

fn assert_igloo_no_basement_and_roll_errors() {
    let no_basement = super::super::igloo_generation_pieces(
        ChunkPos { x: 2, z: -3 },
        super::super::StructureRotation::None,
        0.5,
        7,
    )
    .unwrap();
    assert_eq!(no_basement.len(), 1);
    assert_eq!(
        no_basement[0].template,
        super::super::IglooTemplateKind::Top
    );

    assert_eq!(
        super::super::igloo_generation_pieces(
            ChunkPos { x: 0, z: 0 },
            super::super::StructureRotation::None,
            1.0,
            0,
        )
        .unwrap_err(),
        "Igloo basement roll must be in [0.0, 1.0)".to_string()
    );
    assert_eq!(
        super::super::igloo_generation_pieces(
            ChunkPos { x: 0, z: 0 },
            super::super::StructureRotation::None,
            0.0,
            8,
        )
        .unwrap_err(),
        "Igloo depth roll must match RandomSource#nextInt(8)".to_string()
    );
}

#[test]
fn nether_fossil_generation_scan_and_dried_ghast_match_vanilla() {
    assert_nether_fossil_template_catalog();
    assert_nether_fossil_generation_scan_rules();
    assert_nether_fossil_generation_rejections();
    assert_nether_fossil_dried_ghast_placement_rules();
}

fn assert_nether_fossil_template_catalog() {
    assert_eq!(super::super::NETHER_FOSSIL_TEMPLATES.len(), 14);
    assert_eq!(
        super::super::NETHER_FOSSIL_TEMPLATES[0],
        "minecraft:nether_fossils/fossil_1"
    );
    assert_eq!(
        super::super::NETHER_FOSSIL_TEMPLATES[13],
        "minecraft:nether_fossils/fossil_14"
    );
}

fn assert_nether_fossil_generation_scan_rules() {
    let generation = super::super::nether_fossil_find_generation_point(
        super::super::NetherFossilGenerationInput {
            chunk_pos: ChunkPos { x: -2, z: 3 },
            block_x_roll: 5,
            block_z_roll: 11,
            sampled_y: 72,
            sea_level: 31,
            template_index: 6,
            rotation: super::super::StructureRotation::Counterclockwise90,
        },
        |y| {
            if y == 70 {
                "minecraft:air"
            } else if y == 69 {
                "minecraft:soul_sand"
            } else {
                "minecraft:netherrack"
            }
        },
        |_| false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        generation.position,
        BlockPos {
            x: -27,
            y: 69,
            z: 59
        }
    );
    assert_eq!(generation.piece.template_index, 6);
    assert_eq!(
        generation.piece.template_name,
        "minecraft:nether_fossils/fossil_7"
    );
    assert_eq!(generation.piece.template_position, generation.position);
    assert_eq!(
        generation.piece.rotation,
        super::super::StructureRotation::Counterclockwise90
    );
    assert_eq!(
        generation.piece.processor,
        "minecraft:block_ignore_structure_and_air"
    );

    let sturdy_generation = super::super::nether_fossil_find_generation_point(
        super::super::NetherFossilGenerationInput {
            chunk_pos: ChunkPos { x: 0, z: 0 },
            block_x_roll: 0,
            block_z_roll: 0,
            sampled_y: 50,
            sea_level: 31,
            template_index: 0,
            rotation: super::super::StructureRotation::None,
        },
        |y| {
            if y == 45 {
                "minecraft:air"
            } else {
                "minecraft:netherrack"
            }
        },
        |y| y == 44,
    )
    .unwrap()
    .unwrap();
    assert_eq!(sturdy_generation.position, BlockPos { x: 0, y: 44, z: 0 });
}

fn assert_nether_fossil_generation_rejections() {
    let too_low = super::super::nether_fossil_find_generation_point(
        super::super::NetherFossilGenerationInput {
            chunk_pos: ChunkPos { x: 0, z: 0 },
            block_x_roll: 0,
            block_z_roll: 0,
            sampled_y: 33,
            sea_level: 31,
            template_index: 0,
            rotation: super::super::StructureRotation::None,
        },
        |_| "minecraft:netherrack",
        |_| false,
    )
    .unwrap();
    assert!(too_low.is_none());

    assert_eq!(
        super::super::nether_fossil_make_piece(
            BlockPos { x: 0, y: 0, z: 0 },
            14,
            super::super::StructureRotation::None,
        )
        .unwrap_err(),
        "Nether fossil template index must match Util.getRandom(FOSSILS)".to_string()
    );
}

fn assert_nether_fossil_dried_ghast_placement_rules() {
    let fossil_bb = super::super::StructureBoundingBoxModel {
        min_x: -10,
        min_y: 41,
        min_z: 20,
        max_x: -5,
        max_y: 45,
        max_z: 27,
    };
    let chunk_bb = super::super::StructureBoundingBoxModel {
        min_x: -16,
        min_y: i32::MIN,
        min_z: 16,
        max_x: -1,
        max_y: i32::MAX,
        max_z: 31,
    };
    assert_eq!(
        super::super::nether_fossil_dried_ghast_placement(
            fossil_bb,
            chunk_bb,
            0.49,
            2,
            3,
            super::super::StructureRotation::Clockwise180,
            "minecraft:air",
        )
        .unwrap(),
        Some(super::super::DriedGhastPlacementModel {
            pos: BlockPos {
                x: -8,
                y: 41,
                z: 23
            },
            rotation: super::super::StructureRotation::Clockwise180,
        })
    );
    assert!(super::super::nether_fossil_dried_ghast_placement(
        fossil_bb,
        chunk_bb,
        0.5,
        2,
        3,
        super::super::StructureRotation::Clockwise180,
        "minecraft:air",
    )
    .unwrap()
    .is_none());
    assert!(super::super::nether_fossil_dried_ghast_placement(
        fossil_bb,
        chunk_bb,
        0.49,
        2,
        3,
        super::super::StructureRotation::Clockwise180,
        "minecraft:netherrack",
    )
    .unwrap()
    .is_none());
}

include!("structure_template_feature_ruined_portal_tests.rs");
