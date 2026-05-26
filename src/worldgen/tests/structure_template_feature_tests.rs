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

#[test]
fn ruined_portal_setup_template_and_vertical_rules_match_vanilla() {
    assert_ruined_portal_setup_probability_and_template_rules();
    assert_ruined_portal_initial_and_suitable_y_rules();
    assert_ruined_portal_properties_and_piece_processors();
}

fn ruined_portal_land_setup() -> super::super::RuinedPortalSetupModel {
    super::super::RuinedPortalSetupModel {
        placement: super::super::RuinedPortalVerticalPlacement::OnLandSurface,
        air_pocket_probability: 0.0,
        mossiness: 0.2,
        overgrown: false,
        vines: true,
        can_be_cold: true,
        replace_with_blackstone: false,
        weight: 1.0,
    }
}

fn ruined_portal_nether_setup() -> super::super::RuinedPortalSetupModel {
    super::super::RuinedPortalSetupModel {
        placement: super::super::RuinedPortalVerticalPlacement::InNether,
        air_pocket_probability: 1.0,
        mossiness: 0.8,
        overgrown: false,
        vines: false,
        can_be_cold: false,
        replace_with_blackstone: true,
        weight: 3.0,
    }
}

fn assert_ruined_portal_setup_probability_and_template_rules() {
    let land_setup = ruined_portal_land_setup();
    let nether_setup = ruined_portal_nether_setup();
    assert_eq!(
        super::super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.24).unwrap(),
        land_setup
    );
    assert_eq!(
        super::super::ruined_portal_choose_setup(&[land_setup, nether_setup], 0.25).unwrap(),
        nether_setup
    );
    assert_eq!(
        super::super::ruined_portal_vertical_placement_id(
            super::super::RuinedPortalVerticalPlacement::PartlyBuried
        ),
        "partly_buried"
    );
    assert!(!super::super::ruined_portal_sample_probability(0.0, 0.0).unwrap());
    assert!(super::super::ruined_portal_sample_probability(1.0, 0.99).unwrap());
    assert!(super::super::ruined_portal_sample_probability(0.3, 0.29).unwrap());
    assert!(!super::super::ruined_portal_sample_probability(0.3, 0.3).unwrap());

    assert_eq!(
        super::super::ruined_portal_template_name(0.049, 2).unwrap(),
        "minecraft:ruined_portal/giant_portal_3"
    );
    assert_eq!(
        super::super::ruined_portal_template_name(0.05, 9).unwrap(),
        "minecraft:ruined_portal/portal_10"
    );
    assert_eq!(
        super::super::ruined_portal_mirror(0.49).unwrap(),
        super::super::RuinedPortalMirrorModel::None
    );
    assert_eq!(
        super::super::ruined_portal_mirror(0.5).unwrap(),
        super::super::RuinedPortalMirrorModel::FrontBack
    );
}

fn assert_ruined_portal_initial_and_suitable_y_rules() {
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            true,
            80,
            12,
            -64,
            0.75,
            5,
        )
        .unwrap(),
        37
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            false,
            80,
            12,
            -64,
            0.49,
            2,
        )
        .unwrap(),
        29
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InNether,
            false,
            80,
            12,
            -64,
            0.5,
            0,
        )
        .unwrap(),
        29
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::InMountain,
            false,
            96,
            20,
            -64,
            0.0,
            3,
        )
        .unwrap(),
        73
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::Underground,
            false,
            50,
            20,
            -64,
            0.0,
            79,
        )
        .unwrap(),
        30
    );
    assert_eq!(
        super::super::ruined_portal_initial_y(
            super::super::RuinedPortalVerticalPlacement::PartlyBuried,
            false,
            90,
            16,
            -64,
            0.0,
            4,
        )
        .unwrap(),
        80
    );

    let suitable = super::super::ruined_portal_find_suitable_y(
        super::super::RuinedPortalVerticalPlacement::OnOceanFloor,
        -64,
        80,
        |y| {
            if y == 72 {
                3
            } else {
                2
            }
        },
    );
    assert_eq!(suitable, 72);
    assert_eq!(
        super::super::ruined_portal_find_suitable_y(
            super::super::RuinedPortalVerticalPlacement::Underground,
            -64,
            -40,
            |_| 0,
        ),
        -49
    );
}

fn assert_ruined_portal_properties_and_piece_processors() {
    let properties =
        super::super::ruined_portal_make_properties(ruined_portal_nether_setup(), 0.0, true)
            .unwrap();
    assert_eq!(
        properties,
        super::super::RuinedPortalPropertiesModel {
            cold: false,
            mossiness: 0.8,
            air_pocket: true,
            overgrown: false,
            vines: false,
            replace_with_blackstone: true,
        }
    );
    let piece = super::super::ruined_portal_make_piece(
        "minecraft:ruined_portal/portal_1",
        BlockPos {
            x: 16,
            y: 37,
            z: -16,
        },
        super::super::RuinedPortalVerticalPlacement::InNether,
        properties,
        super::super::StructureRotation::Clockwise90,
        super::super::RuinedPortalMirrorModel::FrontBack,
        BlockPos { x: 4, y: 0, z: 5 },
    );
    assert_eq!(
        piece.ignore_processor,
        "minecraft:block_ignore_structure_block"
    );
    assert_eq!(piece.lava_replacement, "minecraft:magma_block@0.2");
    assert!(piece.include_blackstone_replace_processor);
    assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 5 });

    let ocean_piece = super::super::ruined_portal_make_piece(
        "minecraft:ruined_portal/portal_2",
        BlockPos { x: 0, y: 0, z: 0 },
        super::super::RuinedPortalVerticalPlacement::OnOceanFloor,
        super::super::RuinedPortalPropertiesModel {
            cold: true,
            mossiness: 0.0,
            air_pocket: false,
            overgrown: false,
            vines: false,
            replace_with_blackstone: false,
        },
        super::super::StructureRotation::None,
        super::super::RuinedPortalMirrorModel::None,
        BlockPos { x: 0, y: 0, z: 0 },
    );
    assert_eq!(
        ocean_piece.ignore_processor,
        "minecraft:block_ignore_structure_and_air"
    );
    assert_eq!(ocean_piece.lava_replacement, "minecraft:magma_block");
}

#[test]
fn shipwreck_template_height_and_loot_rules_match_vanilla() {
    assert_shipwreck_template_catalog_and_heightmap_rules();
    assert_shipwreck_piece_save_and_worldgen_region_rules();
    assert_shipwreck_height_adjustment_and_loot_rules();
}

fn shipwreck_test_piece() -> super::super::ShipwreckPieceModel {
    super::super::shipwreck_make_piece(
        ChunkPos { x: -1, z: 4 },
        super::super::StructureRotation::Clockwise180,
        7,
        false,
    )
    .unwrap()
}

fn assert_shipwreck_template_catalog_and_heightmap_rules() {
    assert_eq!(super::super::SHIPWRECK_BEACHED_TEMPLATES.len(), 11);
    assert_eq!(super::super::SHIPWRECK_OCEAN_TEMPLATES.len(), 20);
    assert_eq!(
        super::super::SHIPWRECK_BEACHED_TEMPLATES[0],
        "minecraft:shipwreck/with_mast"
    );
    assert_eq!(
        super::super::SHIPWRECK_OCEAN_TEMPLATES[19],
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert_eq!(
        super::super::shipwreck_heightmap_type(true),
        "minecraft:world_surface_wg"
    );
    assert_eq!(
        super::super::shipwreck_heightmap_type(false),
        "minecraft:ocean_floor_wg"
    );
    assert_eq!(
        super::super::shipwreck_template_name(true, 10).unwrap(),
        "minecraft:shipwreck/rightsideup_backhalf_degraded"
    );
    assert_eq!(
        super::super::shipwreck_template_name(false, 11).unwrap(),
        "minecraft:shipwreck/upsidedown_full_degraded"
    );
    assert_eq!(
        super::super::shipwreck_template_name(true, 11).unwrap_err(),
        "Shipwreck template index must match Util.getRandom template list".to_string()
    );
}

fn assert_shipwreck_piece_save_and_worldgen_region_rules() {
    let piece = shipwreck_test_piece();
    assert_eq!(piece.template_name, "minecraft:shipwreck/rightsideup_full");
    assert_eq!(
        piece.template_position,
        BlockPos {
            x: -16,
            y: 90,
            z: 64
        }
    );
    assert_eq!(piece.pivot, BlockPos { x: 4, y: 0, z: 15 });
    assert_eq!(piece.processor, "minecraft:block_ignore_structure_and_air");
    assert!(!piece.height_adjusted);
    assert!(!piece.is_beached);
    assert_eq!(
        super::super::shipwreck_save_tag(piece),
        super::super::ShipwreckSaveTagModel {
            is_beached: false,
            rotation: super::super::StructureRotation::Clockwise180,
            height_adjusted: false,
        }
    );

    assert!(
        !super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 32,
            y: 32,
            z: 80
        })
    );
    assert!(
        super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 33,
            y: 12,
            z: 12
        })
    );
    assert!(
        super::super::shipwreck_is_too_big_to_fit_in_worldgen_region(BlockPos {
            x: 12,
            y: 33,
            z: 12
        })
    );
}

fn assert_shipwreck_height_adjustment_and_loot_rules() {
    assert_eq!(
        super::super::shipwreck_calculate_beached_position(72, 15, 2).unwrap(),
        63
    );
    assert_eq!(
        super::super::shipwreck_calculate_beached_position(72, 15, 3).unwrap_err(),
        "Shipwreck beached height roll must match RandomSource#nextInt(3)".to_string()
    );
    let piece = shipwreck_test_piece();
    let adjusted = super::super::shipwreck_adjust_position_height(piece, 48);
    assert_eq!(adjusted.template_position.y, 48);
    assert!(adjusted.height_adjusted);
    assert_eq!(
        super::super::shipwreck_save_tag(adjusted),
        super::super::ShipwreckSaveTagModel {
            is_beached: false,
            rotation: super::super::StructureRotation::Clockwise180,
            height_adjusted: true,
        }
    );

    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("map_chest"),
        Some("minecraft:chests/shipwreck_map")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("treasure_chest"),
        Some("minecraft:chests/shipwreck_treasure")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("supply_chest"),
        Some("minecraft:chests/shipwreck_supply")
    );
    assert_eq!(
        super::super::shipwreck_loot_table_for_marker("unknown"),
        None
    );
}

#[test]
fn ocean_ruin_piece_templates_markers_and_height_match_vanilla() {
    assert_ocean_ruin_template_catalog_and_probability_rules();
    assert_ocean_ruin_piece_selection_and_save_tags();
    assert_ocean_ruin_marker_actions();
    assert_ocean_ruin_floor_adjustment_rules();
}

fn ocean_ruin_warm_config() -> super::super::OceanRuinStructureConfigModel {
    super::super::OceanRuinStructureConfigModel {
        biome_type: super::super::OceanRuinBiomeType::Warm,
        large_probability: 0.3,
        cluster_probability: 0.9,
    }
}

fn ocean_ruin_cold_config() -> super::super::OceanRuinStructureConfigModel {
    super::super::OceanRuinStructureConfigModel {
        biome_type: super::super::OceanRuinBiomeType::Cold,
        large_probability: 1.0,
        cluster_probability: 0.0,
    }
}

fn ocean_ruin_warm_piece() -> super::super::OceanRuinPieceModel {
    super::super::ocean_ruin_add_piece(
        ocean_ruin_warm_config(),
        BlockPos {
            x: 16,
            y: 90,
            z: -32,
        },
        super::super::StructureRotation::Clockwise90,
        true,
        0.9,
        2,
    )
    .unwrap()[0]
}

fn ocean_ruin_cold_pieces() -> Vec<super::super::OceanRuinPieceModel> {
    super::super::ocean_ruin_add_piece(
        ocean_ruin_cold_config(),
        BlockPos { x: 0, y: 90, z: 0 },
        super::super::StructureRotation::None,
        false,
        0.8,
        4,
    )
    .unwrap()
}

fn assert_ocean_ruin_template_catalog_and_probability_rules() {
    assert_eq!(super::super::OCEAN_RUIN_WARM_TEMPLATES.len(), 8);
    assert_eq!(super::super::OCEAN_RUIN_BIG_WARM_TEMPLATES.len(), 4);
    assert_eq!(
        super::super::OCEAN_RUIN_BRICK_TEMPLATES[0],
        "minecraft:underwater_ruin/brick_1"
    );
    assert_eq!(
        super::super::OCEAN_RUIN_BIG_CRACKED_TEMPLATES[3],
        "minecraft:underwater_ruin/big_cracked_8"
    );
    assert_eq!(
        super::super::ocean_ruin_biome_type_id(super::super::OceanRuinBiomeType::Warm),
        "warm"
    );
    assert_eq!(
        super::super::ocean_ruin_biome_type_id(super::super::OceanRuinBiomeType::Cold),
        "cold"
    );
    assert!(super::super::ocean_ruin_is_large(0.4, 0.4).unwrap());
    assert!(!super::super::ocean_ruin_is_large(0.4, 0.4001).unwrap());
    assert!(super::super::ocean_ruin_should_add_cluster(0.2, 0.2).unwrap());
    assert!(!super::super::ocean_ruin_should_add_cluster(0.2, 0.21).unwrap());
}

fn assert_ocean_ruin_piece_selection_and_save_tags() {
    let warm_piece = super::super::ocean_ruin_add_piece(
        ocean_ruin_warm_config(),
        BlockPos {
            x: 16,
            y: 90,
            z: -32,
        },
        super::super::StructureRotation::Clockwise90,
        true,
        0.9,
        2,
    )
    .unwrap();
    assert_eq!(warm_piece.len(), 1);
    assert_eq!(
        warm_piece[0].template_name,
        "minecraft:underwater_ruin/big_warm_6"
    );
    assert_eq!(warm_piece[0].integrity, 0.9);
    assert_eq!(warm_piece[0].suspicious_block, "minecraft:suspicious_sand");
    assert_eq!(
        warm_piece[0].suspicious_loot_table,
        "minecraft:archaeology/ocean_ruin_warm"
    );

    let cold_pieces = super::super::ocean_ruin_add_piece(
        ocean_ruin_cold_config(),
        BlockPos { x: 0, y: 90, z: 0 },
        super::super::StructureRotation::None,
        false,
        0.8,
        4,
    )
    .unwrap();
    assert_eq!(cold_pieces.len(), 3);
    assert_eq!(
        cold_pieces
            .iter()
            .map(|piece| (piece.template_name, piece.integrity))
            .collect::<Vec<_>>(),
        vec![
            ("minecraft:underwater_ruin/brick_5", 0.8),
            ("minecraft:underwater_ruin/cracked_5", 0.7),
            ("minecraft:underwater_ruin/mossy_5", 0.5),
        ]
    );
    assert_eq!(
        cold_pieces[0].suspicious_block,
        "minecraft:suspicious_gravel"
    );
    assert_eq!(
        super::super::ocean_ruin_save_tag(cold_pieces[1]),
        super::super::OceanRuinSaveTagModel {
            rotation: super::super::StructureRotation::None,
            integrity: 0.7,
            biome_type: super::super::OceanRuinBiomeType::Cold,
            is_large: false,
        }
    );
}

fn assert_ocean_ruin_marker_actions() {
    let warm_piece = ocean_ruin_warm_piece();
    let cold_pieces = ocean_ruin_cold_pieces();
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            warm_piece,
            "chest",
            BlockPos { x: 1, y: 50, z: 2 },
            63,
            true
        ),
        Some(super::super::OceanRuinMarkerActionModel {
            marker_id: "chest",
            pos: BlockPos { x: 1, y: 50, z: 2 },
            placed_block: "minecraft:chest[waterlogged=true]",
            loot_table: Some("minecraft:chests/underwater_ruin_big"),
            spawned_entity: None,
        })
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "drowned",
            BlockPos { x: 1, y: 70, z: 2 },
            63,
            false
        ),
        Some(super::super::OceanRuinMarkerActionModel {
            marker_id: "drowned",
            pos: BlockPos { x: 1, y: 70, z: 2 },
            placed_block: "minecraft:air",
            loot_table: None,
            spawned_entity: Some("minecraft:drowned"),
        })
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "drowned",
            BlockPos { x: 1, y: 50, z: 2 },
            63,
            false
        )
        .unwrap()
        .placed_block,
        "minecraft:water"
    );
    assert_eq!(
        super::super::ocean_ruin_marker_action(
            cold_pieces[0],
            "unknown",
            BlockPos { x: 0, y: 0, z: 0 },
            63,
            false
        ),
        None
    );
}

fn assert_ocean_ruin_floor_adjustment_rules() {
    let warm_piece = ocean_ruin_warm_piece();
    let adjusted = super::super::ocean_ruin_adjust_to_ocean_floor(
        warm_piece,
        70,
        BlockPos { x: 4, y: 6, z: 4 },
        |x, z| {
            if x == 16 && z == -32 {
                69
            } else {
                64
            }
        },
    );
    assert_eq!(adjusted.template_position.y, 65);
    let flat = super::super::ocean_ruin_adjust_to_ocean_floor(
        warm_piece,
        70,
        BlockPos { x: 4, y: 6, z: 4 },
        |_, _| 69,
    );
    assert_eq!(flat.template_position.y, 70);
}
