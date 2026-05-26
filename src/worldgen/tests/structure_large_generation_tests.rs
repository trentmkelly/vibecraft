use super::*;

#[test]
fn mineshaft_start_materials_and_piece_boxes_match_vanilla() {
    assert_mineshaft_type_materials_and_start_pos();
    assert_mineshaft_room_and_vertical_adjustment();
    assert_mineshaft_piece_box_selection();
    assert_mineshaft_corridor_save_state();
}

fn assert_mineshaft_type_materials_and_start_pos() {
    assert_eq!(
        super::super::mineshaft_type_by_id(-1),
        super::super::MineshaftTypeModel::Normal
    );
    assert_eq!(
        super::super::mineshaft_type_by_id(0),
        super::super::MineshaftTypeModel::Normal
    );
    assert_eq!(
        super::super::mineshaft_type_by_id(1),
        super::super::MineshaftTypeModel::Mesa
    );
    assert_eq!(
        super::super::mineshaft_type_id(super::super::MineshaftTypeModel::Mesa),
        1
    );
    assert_eq!(
        super::super::mineshaft_materials(super::super::MineshaftTypeModel::Normal),
        super::super::MineshaftMaterialModel {
            serialized_name: "normal",
            wood_state: "minecraft:oak_log",
            planks_state: "minecraft:oak_planks",
            fence_state: "minecraft:oak_fence",
        }
    );
    assert_eq!(
        super::super::mineshaft_materials(super::super::MineshaftTypeModel::Mesa).planks_state,
        "minecraft:dark_oak_planks"
    );
    assert_eq!(
        super::super::mineshaft_start_pos(ChunkPos { x: -2, z: 3 }),
        BlockPos {
            x: -24,
            y: 50,
            z: 48
        }
    );
}

fn assert_mineshaft_room_and_vertical_adjustment() {
    let room = super::super::mineshaft_room(
        ChunkPos { x: -2, z: 3 },
        super::super::MineshaftTypeModel::Mesa,
        5,
        4,
        3,
    )
    .unwrap();
    assert_eq!(
        room.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: -30,
            min_y: 50,
            min_z: 50,
            max_x: -18,
            max_y: 58,
            max_z: 60,
        }
    );
    assert_eq!(
        super::super::mineshaft_room(
            ChunkPos { x: 0, z: 0 },
            super::super::MineshaftTypeModel::Normal,
            6,
            0,
            0,
        )
        .unwrap_err(),
        "Mineshaft room size rolls must match RandomSource#nextInt(6)".to_string()
    );

    let aggregate = super::super::StructureBoundingBoxModel {
        min_x: -30,
        min_y: 50,
        min_z: 50,
        max_x: -18,
        max_y: 58,
        max_z: 60,
    };
    assert_eq!(
        super::super::mineshaft_move_below_sea_level_dy(aggregate, 63, -64, 10, 20).unwrap(),
        -92
    );
    assert_eq!(
        super::super::mineshaft_mesa_vertical_dy(aggregate, 63, 80, 7).unwrap(),
        16
    );
    assert_eq!(
        super::super::mineshaft_mesa_vertical_dy(aggregate, 63, 60, 0).unwrap(),
        9
    );
}

fn assert_mineshaft_piece_box_selection() {
    let collision = super::super::StructurePieceModel {
        bounding_box: super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 50,
            min_z: -14,
            max_x: 2,
            max_y: 52,
            max_z: -10,
        },
    };
    assert_eq!(
        super::super::mineshaft_find_corridor_size(
            0,
            50,
            0,
            super::super::HorizontalDirection::North,
            1,
            &[collision],
        )
        .unwrap(),
        Some(super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 50,
            min_z: -9,
            max_x: 2,
            max_y: 52,
            max_z: 0,
        })
    );
    assert_eq!(
        super::super::mineshaft_find_crossing(
            10,
            40,
            -5,
            super::super::HorizontalDirection::East,
            0,
            &[],
        )
        .unwrap(),
        Some(super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 40,
            min_z: -6,
            max_x: 14,
            max_y: 46,
            max_z: -2,
        })
    );
    assert_eq!(
        super::super::mineshaft_find_stairs(
            10,
            40,
            -5,
            super::super::HorizontalDirection::West,
            &[],
        ),
        Some(super::super::StructureBoundingBoxModel {
            min_x: 2,
            min_y: 35,
            min_z: -5,
            max_x: 10,
            max_y: 42,
            max_z: -3,
        })
    );

    assert_eq!(
        super::super::mineshaft_random_piece_kind(69).unwrap(),
        super::super::MineshaftPieceKindModel::Corridor
    );
    assert_eq!(
        super::super::mineshaft_random_piece_kind(70).unwrap(),
        super::super::MineshaftPieceKindModel::Stairs
    );
    assert_eq!(
        super::super::mineshaft_random_piece_kind(80).unwrap(),
        super::super::MineshaftPieceKindModel::Crossing
    );
}

fn assert_mineshaft_corridor_save_state() {
    let corridor_box = super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: 40,
        min_z: 0,
        max_x: 2,
        max_y: 42,
        max_z: 14,
    };
    let corridor = super::super::mineshaft_corridor(
        corridor_box,
        super::super::HorizontalDirection::South,
        super::super::MineshaftTypeModel::Normal,
        2,
        0,
    )
    .unwrap();
    assert!(!corridor.has_rails);
    assert!(corridor.spider_corridor);
    assert_eq!(corridor.num_sections, 3);
    assert_eq!(
        super::super::mineshaft_corridor_save_tag(corridor),
        super::super::MineshaftCorridorSaveTagModel {
            has_rails: false,
            spider_corridor: true,
            has_placed_spider: false,
            num_sections: 3,
            mineshaft_type_id: 0,
        }
    );
    assert!(
        !super::super::mineshaft_corridor(
            corridor_box,
            super::super::HorizontalDirection::South,
            super::super::MineshaftTypeModel::Normal,
            0,
            0,
        )
        .unwrap()
        .spider_corridor
    );
}

const EXPECTED_MINESHAFT_CHAIN_PREFIX: [(&str, i32, super::super::StructureBoundingBoxModel); 10] = [
    (
        "room",
        0,
        super::super::StructureBoundingBoxModel {
            min_x: -14,
            min_y: -49,
            min_z: 66,
            max_x: -5,
            max_y: -41,
            max_z: 75,
        },
    ),
    (
        "corridor",
        1,
        super::super::StructureBoundingBoxModel {
            min_x: -7,
            min_y: -45,
            min_z: 46,
            max_x: -5,
            max_y: -43,
            max_z: 65,
        },
    ),
    (
        "stairs",
        2,
        super::super::StructureBoundingBoxModel {
            min_x: -4,
            min_y: -49,
            min_z: 46,
            max_x: 4,
            max_y: -42,
            max_z: 48,
        },
    ),
    (
        "corridor",
        3,
        super::super::StructureBoundingBoxModel {
            min_x: 5,
            min_y: -49,
            min_z: 46,
            max_x: 14,
            max_y: -47,
            max_z: 48,
        },
    ),
    (
        "corridor",
        4,
        super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: -50,
            min_z: 31,
            max_x: 13,
            max_y: -48,
            max_z: 45,
        },
    ),
    (
        "corridor",
        5,
        super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: -51,
            min_z: 11,
            max_x: 13,
            max_y: -49,
            max_z: 30,
        },
    ),
    (
        "corridor",
        6,
        super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: -52,
            min_z: -9,
            max_x: 13,
            max_y: -50,
            max_z: 10,
        },
    ),
    (
        "corridor",
        7,
        super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: -51,
            min_z: -29,
            max_x: 13,
            max_y: -49,
            max_z: -10,
        },
    ),
    (
        "corridor",
        8,
        super::super::StructureBoundingBoxModel {
            min_x: 1,
            min_y: -52,
            min_z: -1,
            max_x: 10,
            max_y: -50,
            max_z: 1,
        },
    ),
    (
        "corridor",
        9,
        super::super::StructureBoundingBoxModel {
            min_x: 1,
            min_y: -51,
            min_z: 2,
            max_x: 3,
            max_y: -49,
            max_z: 21,
        },
    ),
];

#[test]
fn mineshaft_corridor_rail_roll_preserves_java_piece_chain_prng_alignment() {
    let pieces = super::super::mineshaft_generate_pieces_for_start(
        8_675_309,
        ChunkPos { x: -1, z: 4 },
        super::super::MineshaftTypeModel::Normal,
        63,
        -64,
    );
    assert_eq!(pieces.len(), 235);

    for (index, (expected_kind, expected_depth, expected_box)) in
        EXPECTED_MINESHAFT_CHAIN_PREFIX.into_iter().enumerate()
    {
        let piece = &pieces[index];
        let actual_kind = match piece {
            super::super::MineshaftGeneratedPieceModel::Room { .. } => "room",
            super::super::MineshaftGeneratedPieceModel::Corridor { .. } => "corridor",
            super::super::MineshaftGeneratedPieceModel::Crossing { .. } => "crossing",
            super::super::MineshaftGeneratedPieceModel::Stairs { .. } => "stairs",
        };
        assert_eq!(actual_kind, expected_kind, "piece {index} kind drifted");
        assert_eq!(
            piece.gen_depth(),
            expected_depth,
            "piece {index} generation depth drifted"
        );
        assert_eq!(
            piece.bounding_box(),
            expected_box,
            "piece {index} bounding box drifted"
        );
    }
}

#[test]
fn stronghold_start_weights_and_portal_room_match_vanilla() {
    assert_stronghold_weights_and_availability();
    assert_stronghold_random_direction_and_door_rolls();
    assert_stronghold_start_piece_and_portal_room();
    assert_stronghold_portal_room_alternate_and_rejections();
}

fn stronghold_test_start_piece() -> super::super::StrongholdStartPieceModel {
    super::super::stronghold_start_piece(ChunkPos { x: -2, z: 3 }, 1).unwrap()
}

fn stronghold_test_portal_room() -> super::super::StrongholdPortalRoomModel {
    super::super::stronghold_portal_room(
        10,
        40,
        -5,
        super::super::HorizontalDirection::South,
        7,
        &[],
    )
    .unwrap()
}

fn assert_stronghold_weights_and_availability() {
    let weights = super::super::stronghold_piece_weights();
    assert_eq!(weights.len(), 11);
    assert_eq!(
        weights[0],
        super::super::StrongholdPieceWeightModel {
            kind: super::super::StrongholdPieceKindModel::Straight,
            weight: 40,
            max_place_count: 0,
            place_count: 0,
            min_depth: 0,
        }
    );
    assert_eq!(
        weights[9],
        super::super::StrongholdPieceWeightModel {
            kind: super::super::StrongholdPieceKindModel::Library,
            weight: 10,
            max_place_count: 2,
            place_count: 0,
            min_depth: 5,
        }
    );
    assert_eq!(
        weights[10],
        super::super::StrongholdPieceWeightModel {
            kind: super::super::StrongholdPieceKindModel::PortalRoom,
            weight: 20,
            max_place_count: 1,
            place_count: 0,
            min_depth: 6,
        }
    );
    assert_eq!(super::super::stronghold_total_weight(&weights), 145);
    assert!(super::super::stronghold_has_limited_piece_available(
        &weights
    ));
    assert!(!super::super::stronghold_piece_weight_can_place(
        weights[9], 4
    ));
    assert!(super::super::stronghold_piece_weight_can_place(
        weights[9], 5
    ));
    let exhausted_portal = super::super::StrongholdPieceWeightModel {
        place_count: 1,
        ..weights[10]
    };
    assert!(!super::super::stronghold_piece_weight_can_place(
        exhausted_portal,
        6
    ));
    assert!(!super::super::stronghold_piece_weight_is_valid(
        exhausted_portal
    ));
}

fn assert_stronghold_random_direction_and_door_rolls() {
    assert_eq!(
        super::super::stronghold_horizontal_direction_from_random_roll(0).unwrap(),
        super::super::HorizontalDirection::North
    );
    assert_eq!(
        super::super::stronghold_horizontal_direction_from_random_roll(1).unwrap(),
        super::super::HorizontalDirection::South
    );
    assert_eq!(
        super::super::stronghold_horizontal_direction_from_random_roll(2).unwrap(),
        super::super::HorizontalDirection::West
    );
    assert_eq!(
        super::super::stronghold_horizontal_direction_from_random_roll(3).unwrap(),
        super::super::HorizontalDirection::East
    );
    assert_eq!(
        super::super::stronghold_random_small_door(0).unwrap(),
        super::super::StrongholdSmallDoorTypeModel::Opening
    );
    assert_eq!(
        super::super::stronghold_random_small_door(2).unwrap(),
        super::super::StrongholdSmallDoorTypeModel::WoodDoor
    );
    assert_eq!(
        super::super::stronghold_random_small_door(3).unwrap(),
        super::super::StrongholdSmallDoorTypeModel::Grates
    );
    assert_eq!(
        super::super::stronghold_random_small_door(4).unwrap(),
        super::super::StrongholdSmallDoorTypeModel::IronDoor
    );
    assert_eq!(
        super::super::stronghold_random_small_door(5).unwrap_err(),
        "Stronghold small-door roll must match RandomSource#nextInt(5)".to_string()
    );
}

fn assert_stronghold_start_piece_and_portal_room() {
    let start = stronghold_test_start_piece();
    assert_eq!(start.orientation, super::super::HorizontalDirection::South);
    assert_eq!(
        start.entry_door,
        super::super::StrongholdSmallDoorTypeModel::Opening
    );
    assert!(start.is_source);
    assert_eq!(
        start.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: -30,
            min_y: 64,
            min_z: 50,
            max_x: -26,
            max_y: 74,
            max_z: 54,
        }
    );

    let portal_room = stronghold_test_portal_room();
    assert_eq!(
        portal_room.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: 6,
            min_y: 39,
            min_z: -5,
            max_x: 16,
            max_y: 46,
            max_z: 10,
        }
    );
    assert_eq!(
        portal_room.orientation,
        super::super::HorizontalDirection::South
    );
    assert_eq!(portal_room.gen_depth, 7);
    assert_eq!(
        super::super::stronghold_portal_room_save_tag(portal_room),
        super::super::StrongholdPortalRoomSaveTagModel {
            has_placed_spawner: false,
        }
    );
    assert_eq!(
        super::super::stronghold_attach_portal_room(start, portal_room).portal_room_piece,
        Some(portal_room)
    );
}

fn assert_stronghold_portal_room_alternate_and_rejections() {
    let portal_room = stronghold_test_portal_room();
    let west_portal = super::super::stronghold_portal_room(
        10,
        40,
        -5,
        super::super::HorizontalDirection::West,
        7,
        &[],
    )
    .unwrap();
    assert_eq!(
        west_portal.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: -5,
            min_y: 39,
            min_z: -9,
            max_x: 10,
            max_y: 46,
            max_z: 1,
        }
    );
    assert_eq!(
        super::super::stronghold_portal_room(
            10,
            10,
            -5,
            super::super::HorizontalDirection::South,
            7,
            &[]
        ),
        None
    );
    assert_eq!(
        super::super::stronghold_portal_room(
            10,
            40,
            -5,
            super::super::HorizontalDirection::South,
            7,
            &[super::super::StructurePieceModel {
                bounding_box: portal_room.bounding_box,
            }],
        ),
        None
    );
}

#[test]
fn nether_fortress_start_weights_and_child_anchors_match_vanilla() {
    assert_nether_fortress_piece_weight_tables();
    assert_nether_fortress_piece_weight_selection();
    assert_nether_fortress_start_and_bridge_boxes();
    assert_nether_fortress_child_anchors_and_height_adjustment();
}

fn nether_bridge_weights() -> Vec<super::super::NetherFortressPieceWeightModel> {
    super::super::nether_fortress_piece_weights(super::super::NetherFortressPiecePoolModel::Bridge)
}

fn nether_fortress_test_start() -> super::super::NetherFortressStartPieceModel {
    super::super::nether_fortress_start_piece(ChunkPos { x: -2, z: 3 }, 0).unwrap()
}

fn nether_fortress_test_crossing_box() -> super::super::StructureBoundingBoxModel {
    super::super::nether_fortress_bridge_crossing_box(
        10,
        40,
        -5,
        super::super::HorizontalDirection::South,
        &[],
    )
    .unwrap()
}

fn assert_nether_fortress_piece_weight_tables() {
    let bridge_weights = nether_bridge_weights();
    assert_eq!(bridge_weights.len(), 6);
    assert_eq!(
        bridge_weights[0],
        super::super::NetherFortressPieceWeightModel {
            kind: super::super::NetherFortressPieceKindModel::BridgeStraight,
            weight: 30,
            max_place_count: 0,
            place_count: 0,
            allow_in_row: true,
        }
    );
    assert_eq!(
        bridge_weights[5],
        super::super::NetherFortressPieceWeightModel {
            kind: super::super::NetherFortressPieceKindModel::CastleEntrance,
            weight: 5,
            max_place_count: 1,
            place_count: 0,
            allow_in_row: false,
        }
    );

    let castle_weights = super::super::nether_fortress_piece_weights(
        super::super::NetherFortressPiecePoolModel::Castle,
    );
    assert_eq!(castle_weights.len(), 7);
    assert_eq!(
        castle_weights[0],
        super::super::NetherFortressPieceWeightModel {
            kind: super::super::NetherFortressPieceKindModel::CastleSmallCorridor,
            weight: 25,
            max_place_count: 0,
            place_count: 0,
            allow_in_row: true,
        }
    );
    assert_eq!(
        castle_weights[4],
        super::super::NetherFortressPieceWeightModel {
            kind: super::super::NetherFortressPieceKindModel::CastleCorridorStairs,
            weight: 10,
            max_place_count: 3,
            place_count: 0,
            allow_in_row: true,
        }
    );
}

fn assert_nether_fortress_piece_weight_selection() {
    let bridge_weights = nether_bridge_weights();
    assert_eq!(
        super::super::nether_fortress_update_piece_weight(&bridge_weights),
        70
    );
    assert_eq!(
        super::super::nether_fortress_update_piece_weight(&[
            super::super::NetherFortressPieceWeightModel {
                place_count: 1,
                ..bridge_weights[5]
            },
            super::super::NetherFortressPieceWeightModel {
                place_count: 2,
                ..bridge_weights[4]
            },
        ]),
        -1
    );
    assert!(!super::super::nether_fortress_piece_weight_can_place(
        bridge_weights[1],
        Some(super::super::NetherFortressPieceKindModel::BridgeCrossing)
    ));
    assert!(super::super::nether_fortress_piece_weight_can_place(
        bridge_weights[0],
        Some(super::super::NetherFortressPieceKindModel::BridgeStraight)
    ));
    assert!(!super::super::nether_fortress_piece_weight_is_valid(
        super::super::NetherFortressPieceWeightModel {
            place_count: 1,
            ..bridge_weights[5]
        }
    ));
    assert_eq!(
        super::super::nether_fortress_select_piece(&bridge_weights, None, 1, &[29])
            .unwrap()
            .selected_kind,
        super::super::NetherFortressPieceKindModel::BridgeStraight
    );
    assert_eq!(
        super::super::nether_fortress_select_piece(&bridge_weights, None, 1, &[30])
            .unwrap()
            .selected_kind,
        super::super::NetherFortressPieceKindModel::BridgeCrossing
    );
    assert!(
        super::super::nether_fortress_select_piece(&bridge_weights, None, 31, &[0])
            .unwrap()
            .fallback_to_end_filler
    );
    assert_eq!(
        super::super::nether_fortress_select_piece(&bridge_weights, None, 1, &[70]).unwrap_err(),
        "Nether fortress piece roll must match RandomSource#nextInt(totalWeight)".to_string()
    );
}

fn assert_nether_fortress_start_and_bridge_boxes() {
    let start = nether_fortress_test_start();
    assert_eq!(start.orientation, super::super::HorizontalDirection::North);
    assert_eq!(start.bridge_piece_count, 6);
    assert_eq!(start.castle_piece_count, 7);
    assert_eq!(
        start.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: -30,
            min_y: 64,
            min_z: 50,
            max_x: -12,
            max_y: 73,
            max_z: 68,
        }
    );

    let crossing_box = nether_fortress_test_crossing_box();
    assert_eq!(
        crossing_box,
        super::super::StructureBoundingBoxModel {
            min_x: 2,
            min_y: 37,
            min_z: -5,
            max_x: 20,
            max_y: 46,
            max_z: 13,
        }
    );
    assert_eq!(
        super::super::nether_fortress_bridge_crossing_box(
            10,
            10,
            -5,
            super::super::HorizontalDirection::South,
            &[],
        ),
        None
    );
    assert_eq!(
        super::super::nether_fortress_bridge_crossing_box(
            10,
            40,
            -5,
            super::super::HorizontalDirection::South,
            &[super::super::StructurePieceModel {
                bounding_box: crossing_box,
            }],
        ),
        None
    );

    assert_eq!(
        super::super::nether_fortress_bridge_straight_box(
            10,
            40,
            -5,
            super::super::HorizontalDirection::West,
            &[],
        )
        .unwrap(),
        super::super::StructureBoundingBoxModel {
            min_x: -8,
            min_y: 37,
            min_z: -6,
            max_x: 10,
            max_y: 46,
            max_z: -2,
        }
    );
}

fn assert_nether_fortress_child_anchors_and_height_adjustment() {
    let start = nether_fortress_test_start();
    let crossing_box = nether_fortress_test_crossing_box();
    assert_eq!(
        super::super::nether_fortress_child_anchor(super::super::NetherFortressChildAnchorInput {
            start_box: start.bounding_box,
            piece_box: crossing_box,
            piece_orientation: super::super::HorizontalDirection::South,
            piece_depth: 0,
            child_direction: super::super::NetherFortressChildDirectionModel::Forward,
            x_or_y_off: 8,
            y_or_z_off: 3,
            is_castle: false,
        },),
        super::super::NetherFortressChildAnchorModel {
            foot: BlockPos {
                x: 10,
                y: 40,
                z: 14,
            },
            direction: super::super::HorizontalDirection::South,
            next_depth: 1,
            is_castle: false,
            within_start_range: true,
        }
    );
    assert_eq!(
        super::super::nether_fortress_child_anchor(super::super::NetherFortressChildAnchorInput {
            start_box: start.bounding_box,
            piece_box: crossing_box,
            piece_orientation: super::super::HorizontalDirection::South,
            piece_depth: 0,
            child_direction: super::super::NetherFortressChildDirectionModel::Left,
            x_or_y_off: 3,
            y_or_z_off: 8,
            is_castle: false,
        },)
        .direction,
        super::super::HorizontalDirection::West
    );
    assert!(
        !super::super::nether_fortress_child_anchor(super::super::NetherFortressChildAnchorInput {
            start_box: start.bounding_box,
            piece_box: crossing_box.moved(300, 0, 0),
            piece_orientation: super::super::HorizontalDirection::South,
            piece_depth: 0,
            child_direction: super::super::NetherFortressChildDirectionModel::Right,
            x_or_y_off: 3,
            y_or_z_off: 8,
            is_castle: true,
        },)
        .within_start_range
    );

    assert_eq!(
        super::super::structure_pieces_move_inside_heights_dy(crossing_box, 48, 70, 5).unwrap(),
        16
    );
    assert_eq!(
        super::super::structure_pieces_move_inside_heights_dy(crossing_box, 48, 50, 99).unwrap(),
        11
    );
}
