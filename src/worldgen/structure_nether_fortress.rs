use super::*;

pub fn nether_fortress_piece_weights(
    pool: NetherFortressPiecePoolModel,
) -> Vec<NetherFortressPieceWeightModel> {
    match pool {
        NetherFortressPiecePoolModel::Bridge => vec![
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::BridgeStraight,
                weight: 30,
                max_place_count: 0,
                place_count: 0,
                allow_in_row: true,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::BridgeCrossing,
                weight: 10,
                max_place_count: 4,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::RoomCrossing,
                weight: 10,
                max_place_count: 4,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::StairsRoom,
                weight: 10,
                max_place_count: 3,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::MonsterThrone,
                weight: 5,
                max_place_count: 2,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleEntrance,
                weight: 5,
                max_place_count: 1,
                place_count: 0,
                allow_in_row: false,
            },
        ],
        NetherFortressPiecePoolModel::Castle => vec![
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleSmallCorridor,
                weight: 25,
                max_place_count: 0,
                place_count: 0,
                allow_in_row: true,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleSmallCorridorCrossing,
                weight: 15,
                max_place_count: 5,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleSmallCorridorRightTurn,
                weight: 5,
                max_place_count: 10,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleSmallCorridorLeftTurn,
                weight: 5,
                max_place_count: 10,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleCorridorStairs,
                weight: 10,
                max_place_count: 3,
                place_count: 0,
                allow_in_row: true,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleCorridorTBalcony,
                weight: 7,
                max_place_count: 2,
                place_count: 0,
                allow_in_row: false,
            },
            NetherFortressPieceWeightModel {
                kind: NetherFortressPieceKindModel::CastleStalkRoom,
                weight: 5,
                max_place_count: 2,
                place_count: 0,
                allow_in_row: false,
            },
        ],
    }
}

pub fn nether_fortress_piece_weight_can_place(
    piece: NetherFortressPieceWeightModel,
    previous_piece: Option<NetherFortressPieceKindModel>,
) -> bool {
    (piece.max_place_count == 0 || piece.place_count < piece.max_place_count)
        && (piece.allow_in_row || previous_piece != Some(piece.kind))
}

pub fn nether_fortress_piece_weight_is_valid(piece: NetherFortressPieceWeightModel) -> bool {
    piece.max_place_count == 0 || piece.place_count < piece.max_place_count
}

pub fn nether_fortress_update_piece_weight(pieces: &[NetherFortressPieceWeightModel]) -> i32 {
    if pieces
        .iter()
        .any(|piece| piece.max_place_count > 0 && piece.place_count < piece.max_place_count)
    {
        pieces.iter().map(|piece| piece.weight).sum()
    } else {
        -1
    }
}

pub fn nether_fortress_select_piece(
    pieces: &[NetherFortressPieceWeightModel],
    previous_piece: Option<NetherFortressPieceKindModel>,
    depth: i32,
    weight_rolls: &[i32],
) -> Result<NetherFortressPieceSelectionModel, String> {
    let total_weight = nether_fortress_update_piece_weight(pieces);
    if total_weight <= 0 || depth > 30 {
        return Ok(NetherFortressPieceSelectionModel {
            selected_kind: NetherFortressPieceKindModel::BridgeEndFiller,
            fallback_to_end_filler: true,
        });
    }
    for &roll in weight_rolls.iter().take(5) {
        if !(0..total_weight).contains(&roll) {
            return Err(
                "Nether fortress piece roll must match RandomSource#nextInt(totalWeight)"
                    .to_string(),
            );
        }
        let mut weight_selection = roll;
        for piece in pieces {
            weight_selection -= piece.weight;
            if weight_selection < 0 {
                if nether_fortress_piece_weight_can_place(*piece, previous_piece) {
                    return Ok(NetherFortressPieceSelectionModel {
                        selected_kind: piece.kind,
                        fallback_to_end_filler: false,
                    });
                }
                break;
            }
        }
    }
    Ok(NetherFortressPieceSelectionModel {
        selected_kind: NetherFortressPieceKindModel::BridgeEndFiller,
        fallback_to_end_filler: true,
    })
}

pub fn nether_fortress_start_piece(
    chunk_pos: ChunkPos,
    direction_roll: i32,
) -> Result<NetherFortressStartPieceModel, String> {
    let orientation = stronghold_horizontal_direction_from_random_roll(direction_roll)?;
    let west = chunk_pos.x * 16 + 2;
    let north = chunk_pos.z * 16 + 2;
    Ok(NetherFortressStartPieceModel {
        bounding_box: structure_make_bounding_box(west, 64, north, orientation, 19, 10, 19),
        orientation,
        previous_piece: None,
        bridge_piece_count: nether_fortress_piece_weights(NetherFortressPiecePoolModel::Bridge)
            .len(),
        castle_piece_count: nether_fortress_piece_weights(NetherFortressPiecePoolModel::Castle)
            .len(),
        pending_children: 0,
    })
}

pub fn nether_fortress_is_ok_box(bounding_box: StructureBoundingBoxModel) -> bool {
    bounding_box.min_y > 10
}

pub fn nether_fortress_bridge_crossing_box(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    existing_pieces: &[StructurePieceModel],
) -> Option<StructureBoundingBoxModel> {
    let bounding_box = structure_orient_box(
        BlockPos {
            x: foot_x,
            y: foot_y,
            z: foot_z,
        },
        BlockPos { x: -8, y: -3, z: 0 },
        19,
        10,
        19,
        direction,
    );
    (nether_fortress_is_ok_box(bounding_box)
        && structure_piece_find_collision_piece(existing_pieces, bounding_box).is_none())
    .then_some(bounding_box)
}

pub fn nether_fortress_bridge_straight_box(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    existing_pieces: &[StructurePieceModel],
) -> Option<StructureBoundingBoxModel> {
    let bounding_box = structure_orient_box(
        BlockPos {
            x: foot_x,
            y: foot_y,
            z: foot_z,
        },
        BlockPos { x: -1, y: -3, z: 0 },
        5,
        10,
        19,
        direction,
    );
    (nether_fortress_is_ok_box(bounding_box)
        && structure_piece_find_collision_piece(existing_pieces, bounding_box).is_none())
    .then_some(bounding_box)
}

pub fn nether_fortress_child_anchor(
    start_box: StructureBoundingBoxModel,
    piece_box: StructureBoundingBoxModel,
    piece_orientation: HorizontalDirection,
    piece_depth: i32,
    child_direction: NetherFortressChildDirectionModel,
    x_or_y_off: i32,
    y_or_z_off: i32,
    is_castle: bool,
) -> NetherFortressChildAnchorModel {
    let (foot, direction) = match child_direction {
        NetherFortressChildDirectionModel::Forward => match piece_orientation {
            HorizontalDirection::North => (
                BlockPos {
                    x: piece_box.min_x + x_or_y_off,
                    y: piece_box.min_y + y_or_z_off,
                    z: piece_box.min_z - 1,
                },
                piece_orientation,
            ),
            HorizontalDirection::South => (
                BlockPos {
                    x: piece_box.min_x + x_or_y_off,
                    y: piece_box.min_y + y_or_z_off,
                    z: piece_box.max_z + 1,
                },
                piece_orientation,
            ),
            HorizontalDirection::West => (
                BlockPos {
                    x: piece_box.min_x - 1,
                    y: piece_box.min_y + y_or_z_off,
                    z: piece_box.min_z + x_or_y_off,
                },
                piece_orientation,
            ),
            HorizontalDirection::East => (
                BlockPos {
                    x: piece_box.max_x + 1,
                    y: piece_box.min_y + y_or_z_off,
                    z: piece_box.min_z + x_or_y_off,
                },
                piece_orientation,
            ),
        },
        NetherFortressChildDirectionModel::Left => match piece_orientation {
            HorizontalDirection::North | HorizontalDirection::South => (
                BlockPos {
                    x: piece_box.min_x - 1,
                    y: piece_box.min_y + x_or_y_off,
                    z: piece_box.min_z + y_or_z_off,
                },
                HorizontalDirection::West,
            ),
            HorizontalDirection::West | HorizontalDirection::East => (
                BlockPos {
                    x: piece_box.min_x + y_or_z_off,
                    y: piece_box.min_y + x_or_y_off,
                    z: piece_box.min_z - 1,
                },
                HorizontalDirection::North,
            ),
        },
        NetherFortressChildDirectionModel::Right => match piece_orientation {
            HorizontalDirection::North | HorizontalDirection::South => (
                BlockPos {
                    x: piece_box.max_x + 1,
                    y: piece_box.min_y + x_or_y_off,
                    z: piece_box.min_z + y_or_z_off,
                },
                HorizontalDirection::East,
            ),
            HorizontalDirection::West | HorizontalDirection::East => (
                BlockPos {
                    x: piece_box.min_x + y_or_z_off,
                    y: piece_box.min_y + x_or_y_off,
                    z: piece_box.max_z + 1,
                },
                HorizontalDirection::South,
            ),
        },
    };
    NetherFortressChildAnchorModel {
        foot,
        direction,
        next_depth: piece_depth + 1,
        is_castle,
        within_start_range: (foot.x - start_box.min_x).abs() <= 112
            && (foot.z - start_box.min_z).abs() <= 112,
    }
}

