use super::*;

pub fn stronghold_horizontal_direction_from_random_roll(
    roll: i32,
) -> Result<HorizontalDirection, String> {
    if !(0..4).contains(&roll) {
        return Err(
            "Stronghold horizontal direction roll must match RandomSource#nextInt(4)".to_string(),
        );
    }
    Ok(match roll {
        0 => HorizontalDirection::North,
        1 => HorizontalDirection::South,
        2 => HorizontalDirection::West,
        _ => HorizontalDirection::East,
    })
}

pub fn stronghold_piece_weights() -> Vec<StrongholdPieceWeightModel> {
    vec![
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::Straight,
            weight: 40,
            max_place_count: 0,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::PrisonHall,
            weight: 5,
            max_place_count: 5,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::LeftTurn,
            weight: 20,
            max_place_count: 0,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::RightTurn,
            weight: 20,
            max_place_count: 0,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::RoomCrossing,
            weight: 10,
            max_place_count: 6,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::StraightStairsDown,
            weight: 5,
            max_place_count: 5,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::StairsDown,
            weight: 5,
            max_place_count: 5,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::FiveCrossing,
            weight: 5,
            max_place_count: 4,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::ChestCorridor,
            weight: 5,
            max_place_count: 4,
            place_count: 0,
            min_depth: 0,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::Library,
            weight: 10,
            max_place_count: 2,
            place_count: 0,
            min_depth: 5,
        },
        StrongholdPieceWeightModel {
            kind: StrongholdPieceKindModel::PortalRoom,
            weight: 20,
            max_place_count: 1,
            place_count: 0,
            min_depth: 6,
        },
    ]
}

pub fn stronghold_piece_weight_can_place(piece: StrongholdPieceWeightModel, depth: i32) -> bool {
    (piece.max_place_count == 0 || piece.place_count < piece.max_place_count)
        && depth >= piece.min_depth
}

pub fn stronghold_piece_weight_is_valid(piece: StrongholdPieceWeightModel) -> bool {
    piece.max_place_count == 0 || piece.place_count < piece.max_place_count
}

pub fn stronghold_total_weight(pieces: &[StrongholdPieceWeightModel]) -> i32 {
    pieces.iter().map(|piece| piece.weight).sum()
}

pub fn stronghold_has_limited_piece_available(pieces: &[StrongholdPieceWeightModel]) -> bool {
    pieces
        .iter()
        .any(|piece| piece.max_place_count > 0 && piece.place_count < piece.max_place_count)
}

pub fn stronghold_random_small_door(
    selection_roll: i32,
) -> Result<StrongholdSmallDoorTypeModel, String> {
    if !(0..5).contains(&selection_roll) {
        return Err("Stronghold small-door roll must match RandomSource#nextInt(5)".to_string());
    }
    Ok(match selection_roll {
        2 => StrongholdSmallDoorTypeModel::WoodDoor,
        3 => StrongholdSmallDoorTypeModel::Grates,
        4 => StrongholdSmallDoorTypeModel::IronDoor,
        _ => StrongholdSmallDoorTypeModel::Opening,
    })
}

pub fn stronghold_start_piece(
    chunk_pos: ChunkPos,
    direction_roll: i32,
) -> Result<StrongholdStartPieceModel, String> {
    let orientation = stronghold_horizontal_direction_from_random_roll(direction_roll)?;
    let west = chunk_pos.x * 16 + 2;
    let north = chunk_pos.z * 16 + 2;
    Ok(StrongholdStartPieceModel {
        bounding_box: structure_make_bounding_box(west, 64, north, orientation, 5, 11, 5),
        orientation,
        entry_door: StrongholdSmallDoorTypeModel::Opening,
        is_source: true,
        previous_piece: None,
        portal_room_piece: None,
        pending_children: 0,
    })
}

pub fn stronghold_is_ok_box(bounding_box: StructureBoundingBoxModel) -> bool {
    bounding_box.min_y > 10
}

pub fn stronghold_portal_room(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    gen_depth: i32,
    existing_pieces: &[StructurePieceModel],
) -> Option<StrongholdPortalRoomModel> {
    let bounding_box = structure_orient_box(
        BlockPos {
            x: foot_x,
            y: foot_y,
            z: foot_z,
        },
        BlockPos { x: -4, y: -1, z: 0 },
        11,
        8,
        16,
        direction,
    );
    (stronghold_is_ok_box(bounding_box)
        && structure_piece_find_collision_piece(existing_pieces, bounding_box).is_none())
    .then_some(StrongholdPortalRoomModel {
        bounding_box,
        orientation: direction,
        gen_depth,
        has_placed_spawner: false,
    })
}

pub fn stronghold_portal_room_save_tag(
    portal_room: StrongholdPortalRoomModel,
) -> StrongholdPortalRoomSaveTagModel {
    StrongholdPortalRoomSaveTagModel {
        has_placed_spawner: portal_room.has_placed_spawner,
    }
}

pub fn stronghold_attach_portal_room(
    mut start_piece: StrongholdStartPieceModel,
    portal_room: StrongholdPortalRoomModel,
) -> StrongholdStartPieceModel {
    start_piece.portal_room_piece = Some(portal_room);
    start_piece
}
