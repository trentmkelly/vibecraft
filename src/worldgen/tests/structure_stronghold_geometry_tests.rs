use super::*;

fn box_model(
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) -> StructureBoundingBoxModel {
    StructureBoundingBoxModel {
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
    }
}

fn piece_with_box(bounding_box: StructureBoundingBoxModel) -> StructurePieceModel {
    StructurePieceModel { bounding_box }
}

#[test]
fn stronghold_random_direction_roll_matches_java_horizontal_plane_order() {
    let expected = [
        HorizontalDirection::North,
        HorizontalDirection::East,
        HorizontalDirection::South,
        HorizontalDirection::West,
    ];
    for (roll, direction) in expected.into_iter().enumerate() {
        assert_eq!(
            stronghold_horizontal_direction_from_random_roll(roll as i32),
            Ok(direction),
            "Direction.Plane.HORIZONTAL index {roll}"
        );
    }
    assert!(stronghold_horizontal_direction_from_random_roll(-1).is_err());
    assert!(stronghold_horizontal_direction_from_random_roll(4).is_err());
}

#[test]
fn stronghold_piece_factories_use_their_java_dimensions_and_anchor_offsets() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    let cases = [
        (
            StrongholdPieceKindModel::Straight,
            box_model(99, 39, 200, 103, 43, 206),
        ),
        (
            StrongholdPieceKindModel::ChestCorridor,
            box_model(99, 39, 200, 103, 43, 206),
        ),
        (
            StrongholdPieceKindModel::PrisonHall,
            box_model(99, 39, 200, 107, 43, 210),
        ),
        (
            StrongholdPieceKindModel::LeftTurn,
            box_model(99, 39, 200, 103, 43, 204),
        ),
        (
            StrongholdPieceKindModel::RightTurn,
            box_model(99, 39, 200, 103, 43, 204),
        ),
        (
            StrongholdPieceKindModel::RoomCrossing,
            box_model(96, 39, 200, 106, 45, 210),
        ),
        (
            StrongholdPieceKindModel::StraightStairsDown,
            box_model(99, 33, 200, 103, 43, 207),
        ),
        (
            StrongholdPieceKindModel::StairsDown,
            box_model(99, 33, 200, 103, 43, 204),
        ),
        (
            StrongholdPieceKindModel::FiveCrossing,
            box_model(96, 37, 200, 105, 45, 210),
        ),
        (
            StrongholdPieceKindModel::Library,
            box_model(96, 39, 200, 109, 49, 214),
        ),
        (
            StrongholdPieceKindModel::PortalRoom,
            box_model(96, 39, 200, 106, 46, 215),
        ),
    ];

    for (kind, expected) in cases {
        assert_eq!(
            stronghold_find_piece_box(kind, foot, HorizontalDirection::South, &[]),
            Some(expected),
            "unexpected south-facing factory box for {kind:?}"
        );
    }
}

#[test]
fn stronghold_straight_piece_boxes_map_all_horizontal_directions() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    let cases = [
        (
            HorizontalDirection::North,
            box_model(99, 39, 194, 103, 43, 200),
        ),
        (
            HorizontalDirection::East,
            box_model(100, 39, 199, 106, 43, 203),
        ),
        (
            HorizontalDirection::South,
            box_model(99, 39, 200, 103, 43, 206),
        ),
        (
            HorizontalDirection::West,
            box_model(94, 39, 199, 100, 43, 203),
        ),
    ];

    for (direction, expected) in cases {
        assert_eq!(
            stronghold_find_piece_box(StrongholdPieceKindModel::Straight, foot, direction, &[]),
            Some(expected),
            "unexpected straight piece box facing {direction:?}"
        );
    }
}

#[test]
fn stronghold_factory_boxes_use_a_strict_min_y_gate_and_inclusive_collisions() {
    let at_straight_floor = BlockPos {
        x: 100,
        y: 11,
        z: 200,
    };
    assert_eq!(
        stronghold_find_piece_box(
            StrongholdPieceKindModel::Straight,
            at_straight_floor,
            HorizontalDirection::South,
            &[],
        ),
        None,
        "a box with minY=10 fails Java's strict minY > 10 check"
    );
    assert!(stronghold_find_piece_box(
        StrongholdPieceKindModel::Straight,
        BlockPos {
            y: 12,
            ..at_straight_floor
        },
        HorizontalDirection::South,
        &[],
    )
    .is_some());

    let at_stairs_floor = BlockPos {
        x: 100,
        y: 17,
        z: 200,
    };
    assert_eq!(
        stronghold_find_piece_box(
            StrongholdPieceKindModel::StairsDown,
            at_stairs_floor,
            HorizontalDirection::South,
            &[],
        ),
        None,
        "stairs use their -7 vertical offset before the same strict gate"
    );
    assert!(stronghold_find_piece_box(
        StrongholdPieceKindModel::StairsDown,
        BlockPos {
            y: 18,
            ..at_stairs_floor
        },
        HorizontalDirection::South,
        &[],
    )
    .is_some());

    let normal_foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    let merely_adjacent = piece_with_box(box_model(104, 39, 202, 104, 39, 202));
    assert!(stronghold_find_piece_box(
        StrongholdPieceKindModel::Straight,
        normal_foot,
        HorizontalDirection::South,
        &[merely_adjacent],
    )
    .is_some());

    let touching_inclusive_edge = piece_with_box(box_model(103, 39, 202, 103, 39, 202));
    assert_eq!(
        stronghold_find_piece_box(
            StrongholdPieceKindModel::Straight,
            normal_foot,
            HorizontalDirection::South,
            &[touching_inclusive_edge],
        ),
        None,
        "BoundingBox intersections include blocks sharing the max boundary"
    );
}

#[test]
fn stronghold_library_retries_with_six_block_height_after_tall_box_collision() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    // This piece intersects only the Java 11-high library box (which ends at
    // y=49); the six-high retry ends at y=44 and remains clear.
    let tall_only_obstacle = piece_with_box(box_model(100, 45, 201, 100, 46, 201));
    assert_eq!(
        stronghold_find_piece_box(
            StrongholdPieceKindModel::Library,
            foot,
            HorizontalDirection::South,
            &[tall_only_obstacle],
        ),
        Some(box_model(96, 39, 200, 109, 44, 214))
    );

    let obstacle_also_in_short_retry = piece_with_box(box_model(100, 42, 201, 100, 43, 201));
    assert_eq!(
        stronghold_find_piece_box(
            StrongholdPieceKindModel::Library,
            foot,
            HorizontalDirection::South,
            &[obstacle_also_in_short_retry],
        ),
        None
    );
}

#[test]
fn stronghold_filler_corridor_extends_one_block_past_the_first_clear_probe_depth() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    // Each obstacle intersects the four-block probe at its far edge, but not
    // the two-block candidate. Vanilla therefore returns a three-block box.
    let cases = [
        (
            HorizontalDirection::North,
            box_model(99, 39, 198, 99, 39, 198),
            box_model(99, 39, 198, 103, 43, 200),
        ),
        (
            HorizontalDirection::East,
            box_model(102, 39, 199, 102, 39, 199),
            box_model(100, 39, 199, 102, 43, 203),
        ),
        (
            HorizontalDirection::South,
            box_model(99, 39, 202, 99, 39, 202),
            box_model(99, 39, 200, 103, 43, 202),
        ),
        (
            HorizontalDirection::West,
            box_model(98, 39, 199, 98, 39, 199),
            box_model(98, 39, 199, 100, 43, 203),
        ),
    ];

    for (direction, obstacle, expected_box) in cases {
        let corridor = stronghold_filler_corridor(foot, direction, 17, &[piece_with_box(obstacle)])
            .expect("clear short candidate should produce a filler corridor");
        assert_eq!(
            corridor.bounding_box, expected_box,
            "box facing {direction:?}"
        );
        assert_eq!(corridor.orientation, direction);
        assert_eq!(corridor.gen_depth, 17);
        assert_eq!(corridor.steps, 3, "step span facing {direction:?}");
    }
}

#[test]
fn stronghold_filler_corridor_returns_depth_two_when_only_depth_one_is_clear() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    let collision_at_second_south_block = piece_with_box(box_model(99, 39, 201, 99, 39, 201));
    assert_eq!(
        stronghold_filler_corridor(
            foot,
            HorizontalDirection::South,
            9,
            &[collision_at_second_south_block],
        ),
        Some(StrongholdFillerCorridorModel {
            bounding_box: box_model(99, 39, 200, 103, 43, 201),
            orientation: HorizontalDirection::South,
            gen_depth: 9,
            steps: 2,
        })
    );
}

#[test]
fn stronghold_filler_corridor_requires_a_same_level_probe_and_min_y_above_one() {
    let foot = BlockPos {
        x: 100,
        y: 40,
        z: 200,
    };
    assert_eq!(
        stronghold_filler_corridor(foot, HorizontalDirection::South, 1, &[]),
        None
    );

    let probe_intersection = box_model(99, 39, 202, 99, 39, 202);
    let wrong_floor = piece_with_box(box_model(99, 38, 202, 99, 40, 202));
    assert_eq!(
        stronghold_filler_corridor(foot, HorizontalDirection::South, 1, &[wrong_floor],),
        None,
        "the first intersecting piece must have exactly the probe minY"
    );

    let low_foot = BlockPos { y: 2, ..foot };
    let low_probe_obstacle = piece_with_box(box_model(99, 1, 202, 99, 1, 202));
    assert_eq!(
        stronghold_filler_corridor(
            low_foot,
            HorizontalDirection::South,
            1,
            &[low_probe_obstacle],
        ),
        None,
        "the returned full corridor is rejected when minY is at most one"
    );

    let blocked_short_candidates = piece_with_box(box_model(99, 39, 200, 99, 39, 201));
    assert_eq!(
        stronghold_filler_corridor(
            foot,
            HorizontalDirection::South,
            1,
            &[blocked_short_candidates],
        ),
        None
    );

    // The first intersecting piece wins. A later same-level piece cannot
    // replace an earlier intersecting piece on a different floor.
    let later_same_level = piece_with_box(probe_intersection);
    assert_eq!(
        stronghold_filler_corridor(
            foot,
            HorizontalDirection::South,
            1,
            &[wrong_floor, later_same_level],
        ),
        None
    );

    // Although this later piece blocks the depth-two candidate, Java keeps
    // consulting only the first collision object, which permits depth three.
    let later_short_candidate_collision = piece_with_box(box_model(99, 39, 200, 99, 39, 200));
    let result = stronghold_filler_corridor(
        foot,
        HorizontalDirection::South,
        1,
        &[
            piece_with_box(probe_intersection),
            later_short_candidate_collision,
        ],
    );
    assert_eq!(
        result.map(|corridor| corridor.bounding_box),
        Some(box_model(99, 39, 200, 103, 43, 202))
    );
}

fn assert_child_door_anchors(
    parent_direction: HorizontalDirection,
    forward: BlockPos,
    left: BlockPos,
    left_direction: HorizontalDirection,
    right: BlockPos,
    right_direction: HorizontalDirection,
) {
    let start_box = box_model(0, 0, 0, 4, 4, 4);
    let parent_box = box_model(10, 20, 30, 14, 24, 36);
    for (door, expected_foot, expected_orientation) in [
        (
            StrongholdChildDoorModel::Forward {
                horizontal_offset: 2,
                y_offset: 3,
            },
            forward,
            parent_direction,
        ),
        (
            StrongholdChildDoorModel::Left {
                y_offset: 3,
                lateral_offset: 2,
            },
            left,
            left_direction,
        ),
        (
            StrongholdChildDoorModel::Right {
                y_offset: 3,
                lateral_offset: 2,
            },
            right,
            right_direction,
        ),
    ] {
        assert_eq!(
            stronghold_child_attachment(start_box, parent_box, Some(parent_direction), 7, door,),
            Some(StrongholdChildAttachmentModel {
                foot: expected_foot,
                orientation: expected_orientation,
                gen_depth: 8,
            }),
            "door {door:?} from {parent_direction:?}"
        );
    }
}

#[test]
fn stronghold_child_door_anchors_follow_java_for_each_parent_facing() {
    let cases = [
        (
            HorizontalDirection::North,
            BlockPos {
                x: 12,
                y: 23,
                z: 29,
            },
            BlockPos { x: 9, y: 23, z: 32 },
            HorizontalDirection::West,
            BlockPos {
                x: 15,
                y: 23,
                z: 32,
            },
            HorizontalDirection::East,
        ),
        (
            HorizontalDirection::East,
            BlockPos {
                x: 15,
                y: 23,
                z: 32,
            },
            BlockPos {
                x: 12,
                y: 23,
                z: 29,
            },
            HorizontalDirection::North,
            BlockPos {
                x: 12,
                y: 23,
                z: 37,
            },
            HorizontalDirection::South,
        ),
        (
            HorizontalDirection::South,
            BlockPos {
                x: 12,
                y: 23,
                z: 37,
            },
            BlockPos { x: 9, y: 23, z: 32 },
            HorizontalDirection::West,
            BlockPos {
                x: 15,
                y: 23,
                z: 32,
            },
            HorizontalDirection::East,
        ),
        (
            HorizontalDirection::West,
            BlockPos { x: 9, y: 23, z: 32 },
            BlockPos {
                x: 12,
                y: 23,
                z: 29,
            },
            HorizontalDirection::North,
            BlockPos {
                x: 12,
                y: 23,
                z: 37,
            },
            HorizontalDirection::South,
        ),
    ];

    for (parent_direction, forward, left, left_direction, right, right_direction) in cases {
        assert_child_door_anchors(
            parent_direction,
            forward,
            left,
            left_direction,
            right,
            right_direction,
        );
    }
}

#[test]
fn stronghold_child_attachments_keep_vanilla_depth_and_inclusive_112_block_range() {
    let start_box = box_model(0, 0, 0, 4, 4, 4);
    let forward = StrongholdChildDoorModel::Forward {
        horizontal_offset: 0,
        y_offset: 1,
    };
    let at_positive_x_limit = box_model(112, 20, 20, 116, 24, 26);
    assert_eq!(
        stronghold_child_attachment(
            start_box,
            at_positive_x_limit,
            Some(HorizontalDirection::South),
            50,
            forward,
        ),
        Some(StrongholdChildAttachmentModel {
            foot: BlockPos {
                x: 112,
                y: 21,
                z: 27
            },
            orientation: HorizontalDirection::South,
            gen_depth: 51,
        }),
        "the range bound is inclusive and a depth-50 parent can create depth 51"
    );

    let outside_positive_x = box_model(113, 20, 20, 117, 24, 26);
    let outside_negative_x = box_model(-113, 20, 20, -109, 24, 26);
    let outside_positive_z = box_model(20, 20, 113, 24, 24, 119);
    for parent_box in [outside_positive_x, outside_negative_x, outside_positive_z] {
        assert_eq!(
            stronghold_child_attachment(
                start_box,
                parent_box,
                Some(HorizontalDirection::South),
                4,
                forward,
            ),
            None,
            "child anchor beyond 112 blocks from the start min corner must be rejected"
        );
    }

    assert_eq!(
        stronghold_child_attachment(
            start_box,
            at_positive_x_limit,
            Some(HorizontalDirection::South),
            51,
            forward,
        ),
        None,
        "vanilla rejects parent depths above 50 before incrementing"
    );
    assert_eq!(
        stronghold_child_attachment(start_box, at_positive_x_limit, None, 1, forward),
        None
    );
}

#[test]
fn stronghold_child_range_preserves_java_i32_overflow_behavior() {
    let parent_box = box_model(i32::MAX, 20, 20, i32::MAX, 24, 26);
    let overflowing_forward_offset = StrongholdChildDoorModel::Forward {
        horizontal_offset: 1,
        y_offset: 1,
    };

    // Java int addition wraps: minX + 1 becomes MIN_VALUE. The subsequent
    // subtraction from an equally positioned start corner is zero.
    let start_at_wrapped_anchor = box_model(i32::MIN, 0, 0, i32::MIN, 4, 4);
    assert_eq!(
        stronghold_child_attachment(
            start_at_wrapped_anchor,
            parent_box,
            Some(HorizontalDirection::South),
            3,
            overflowing_forward_offset,
        ),
        Some(StrongholdChildAttachmentModel {
            foot: BlockPos {
                x: i32::MIN,
                y: 21,
                z: 27,
            },
            orientation: HorizontalDirection::South,
            gen_depth: 4,
        })
    );

    // Java Math.abs(Integer.MIN_VALUE) stays negative, so this range check
    // also accepts the wrapped distance when measured from coordinate zero.
    let start_at_zero = box_model(0, 0, 0, 4, 4, 4);
    assert_eq!(
        stronghold_child_attachment(
            start_at_zero,
            parent_box,
            Some(HorizontalDirection::South),
            3,
            overflowing_forward_offset,
        ),
        Some(StrongholdChildAttachmentModel {
            foot: BlockPos {
                x: i32::MIN,
                y: 21,
                z: 27,
            },
            orientation: HorizontalDirection::South,
            gen_depth: 4,
        })
    );
}
