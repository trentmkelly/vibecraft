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
        1 => HorizontalDirection::East,
        2 => HorizontalDirection::South,
        _ => HorizontalDirection::West,
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
    let bounding_box = stronghold_find_piece_box(
        StrongholdPieceKindModel::PortalRoom,
        BlockPos {
            x: foot_x,
            y: foot_y,
            z: foot_z,
        },
        direction,
        existing_pieces,
    )?;
    Some(StrongholdPortalRoomModel {
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

/// Factory geometry from `StrongholdPieces.*.createPiece`. Libraries try the
/// eleven-block-high box first, then the six-block-high box after a collision.
/// No factory consumes randomness until its bounding box has been accepted.
pub fn stronghold_find_piece_box(
    kind: StrongholdPieceKindModel,
    foot: BlockPos,
    direction: HorizontalDirection,
    existing_pieces: &[StructurePieceModel],
) -> Option<StructureBoundingBoxModel> {
    let (offset, width, height, depth) = match kind {
        StrongholdPieceKindModel::Straight | StrongholdPieceKindModel::ChestCorridor => {
            (BlockPos { x: -1, y: -1, z: 0 }, 5, 5, 7)
        }
        StrongholdPieceKindModel::PrisonHall => (BlockPos { x: -1, y: -1, z: 0 }, 9, 5, 11),
        StrongholdPieceKindModel::LeftTurn | StrongholdPieceKindModel::RightTurn => {
            (BlockPos { x: -1, y: -1, z: 0 }, 5, 5, 5)
        }
        StrongholdPieceKindModel::RoomCrossing => (BlockPos { x: -4, y: -1, z: 0 }, 11, 7, 11),
        StrongholdPieceKindModel::StraightStairsDown => (BlockPos { x: -1, y: -7, z: 0 }, 5, 11, 8),
        StrongholdPieceKindModel::StairsDown => (BlockPos { x: -1, y: -7, z: 0 }, 5, 11, 5),
        StrongholdPieceKindModel::FiveCrossing => (BlockPos { x: -4, y: -3, z: 0 }, 10, 9, 11),
        StrongholdPieceKindModel::Library => (BlockPos { x: -4, y: -1, z: 0 }, 14, 11, 15),
        StrongholdPieceKindModel::PortalRoom => (BlockPos { x: -4, y: -1, z: 0 }, 11, 8, 16),
    };
    let mut bounding_box = structure_orient_box(foot, offset, width, height, depth, direction);
    if kind == StrongholdPieceKindModel::Library
        && (!stronghold_is_ok_box(bounding_box)
            || structure_piece_find_collision_piece(existing_pieces, bounding_box).is_some())
    {
        bounding_box = structure_orient_box(foot, offset, width, 6, depth, direction);
    }
    (stronghold_is_ok_box(bounding_box)
        && structure_piece_find_collision_piece(existing_pieces, bounding_box).is_none())
    .then_some(bounding_box)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdFillerCorridorModel {
    pub bounding_box: StructureBoundingBoxModel,
    pub orientation: HorizontalDirection,
    pub gen_depth: i32,
    /// Vanilla persists this as `Steps`; it is the box's span along orientation.
    pub steps: i32,
}

/// Implements `FillerCorridor.findPieceBox` followed by the small-door factory's
/// min-Y gate. Only the first piece intersecting the four-block probe is used:
/// vanilla intentionally does not reject a returned box against other pieces.
/// The Java method accepts a random source but never draws from it.
pub fn stronghold_filler_corridor(
    foot: BlockPos,
    direction: HorizontalDirection,
    gen_depth: i32,
    existing_pieces: &[StructurePieceModel],
) -> Option<StrongholdFillerCorridorModel> {
    let offset = BlockPos { x: -1, y: -1, z: 0 };
    let probe = structure_orient_box(foot, offset, 5, 5, 4, direction);
    let collision = structure_piece_find_collision_piece(existing_pieces, probe)?;
    if collision.bounding_box.min_y != probe.min_y {
        return None;
    }
    for depth in (1..=2).rev() {
        let candidate = structure_orient_box(foot, offset, 5, 5, depth, direction);
        if !collision.bounding_box.intersects(candidate) {
            let bounding_box = structure_orient_box(foot, offset, 5, 5, depth + 1, direction);
            if bounding_box.min_y <= 1 {
                return None;
            }
            let steps = match direction {
                HorizontalDirection::North | HorizontalDirection::South => {
                    bounding_box.max_z - bounding_box.min_z + 1
                }
                HorizontalDirection::West | HorizontalDirection::East => {
                    bounding_box.max_x - bounding_box.min_x + 1
                }
            };
            return Some(StrongholdFillerCorridorModel {
                bounding_box,
                orientation: direction,
                gen_depth,
                steps,
            });
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrongholdChildDoorModel {
    /// The first argument of Java's forward helper is the horizontal offset.
    Forward {
        horizontal_offset: i32,
        y_offset: i32,
    },
    /// Side helpers use `(yOff, zOff)`, independent of the parent's facing sign.
    Left {
        y_offset: i32,
        lateral_offset: i32,
    },
    Right {
        y_offset: i32,
        lateral_offset: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrongholdChildAttachmentModel {
    pub foot: BlockPos,
    pub orientation: HorizontalDirection,
    pub gen_depth: i32,
}

/// Pure geometry for `generateSmallDoorChild*` and `generateAndAddPiece`.
/// Callers must pass the parent's generation depth: vanilla tests it against
/// fifty before incrementing, so a child at depth fifty-one is permitted.
pub fn stronghold_child_attachment(
    start_box: StructureBoundingBoxModel,
    parent_box: StructureBoundingBoxModel,
    parent_orientation: Option<HorizontalDirection>,
    parent_depth: i32,
    door: StrongholdChildDoorModel,
) -> Option<StrongholdChildAttachmentModel> {
    let direction = parent_orientation?;
    if parent_depth > 50 {
        return None;
    }
    let (foot, orientation) = match door {
        StrongholdChildDoorModel::Forward {
            horizontal_offset,
            y_offset,
        } => {
            let y = parent_box.min_y.wrapping_add(y_offset);
            let (x, z) = match direction {
                HorizontalDirection::North => (
                    parent_box.min_x.wrapping_add(horizontal_offset),
                    parent_box.min_z.wrapping_sub(1),
                ),
                HorizontalDirection::South => (
                    parent_box.min_x.wrapping_add(horizontal_offset),
                    parent_box.max_z.wrapping_add(1),
                ),
                HorizontalDirection::West => (
                    parent_box.min_x.wrapping_sub(1),
                    parent_box.min_z.wrapping_add(horizontal_offset),
                ),
                HorizontalDirection::East => (
                    parent_box.max_x.wrapping_add(1),
                    parent_box.min_z.wrapping_add(horizontal_offset),
                ),
            };
            (BlockPos { x, y, z }, direction)
        }
        StrongholdChildDoorModel::Left {
            y_offset,
            lateral_offset,
        } => {
            let y = parent_box.min_y.wrapping_add(y_offset);
            match direction {
                HorizontalDirection::North | HorizontalDirection::South => (
                    BlockPos {
                        x: parent_box.min_x.wrapping_sub(1),
                        y,
                        z: parent_box.min_z.wrapping_add(lateral_offset),
                    },
                    HorizontalDirection::West,
                ),
                HorizontalDirection::West | HorizontalDirection::East => (
                    BlockPos {
                        x: parent_box.min_x.wrapping_add(lateral_offset),
                        y,
                        z: parent_box.min_z.wrapping_sub(1),
                    },
                    HorizontalDirection::North,
                ),
            }
        }
        StrongholdChildDoorModel::Right {
            y_offset,
            lateral_offset,
        } => {
            let y = parent_box.min_y.wrapping_add(y_offset);
            match direction {
                HorizontalDirection::North | HorizontalDirection::South => (
                    BlockPos {
                        x: parent_box.max_x.wrapping_add(1),
                        y,
                        z: parent_box.min_z.wrapping_add(lateral_offset),
                    },
                    HorizontalDirection::East,
                ),
                HorizontalDirection::West | HorizontalDirection::East => (
                    BlockPos {
                        x: parent_box.min_x.wrapping_add(lateral_offset),
                        y,
                        z: parent_box.max_z.wrapping_add(1),
                    },
                    HorizontalDirection::South,
                ),
            }
        }
    };
    // Java uses int subtraction followed by Math.abs; preserve its overflow
    // behavior for the full coordinate domain rather than panicking in debug.
    if foot.x.wrapping_sub(start_box.min_x).wrapping_abs() > 112
        || foot.z.wrapping_sub(start_box.min_z).wrapping_abs() > 112
    {
        return None;
    }
    Some(StrongholdChildAttachmentModel {
        foot,
        orientation,
        gen_depth: parent_depth.wrapping_add(1),
    })
}

#[cfg(test)]
#[path = "tests/structure_stronghold_geometry_tests.rs"]
mod structure_stronghold_geometry_tests;
