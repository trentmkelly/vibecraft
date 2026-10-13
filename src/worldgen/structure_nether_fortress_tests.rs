use super::{BlockPos, HorizontalDirection, NetherFortressPieceKindModel};

const NETHER_FORTRESS_PIECE_BOX_SPECS: [(NetherFortressPieceKindModel, i32, i32, i32, i32, i32);
    14] = [
    (
        NetherFortressPieceKindModel::BridgeCrossing,
        -8,
        -3,
        19,
        10,
        19,
    ),
    (
        NetherFortressPieceKindModel::BridgeEndFiller,
        -1,
        -3,
        5,
        10,
        8,
    ),
    (
        NetherFortressPieceKindModel::BridgeStraight,
        -1,
        -3,
        5,
        10,
        19,
    ),
    (
        NetherFortressPieceKindModel::CastleCorridorStairs,
        -1,
        -7,
        5,
        14,
        10,
    ),
    (
        NetherFortressPieceKindModel::CastleCorridorTBalcony,
        -3,
        0,
        9,
        7,
        9,
    ),
    (
        NetherFortressPieceKindModel::CastleEntrance,
        -5,
        -3,
        13,
        14,
        13,
    ),
    (
        NetherFortressPieceKindModel::CastleSmallCorridorCrossing,
        -1,
        0,
        5,
        7,
        5,
    ),
    (
        NetherFortressPieceKindModel::CastleSmallCorridorLeftTurn,
        -1,
        0,
        5,
        7,
        5,
    ),
    (
        NetherFortressPieceKindModel::CastleSmallCorridor,
        -1,
        0,
        5,
        7,
        5,
    ),
    (
        NetherFortressPieceKindModel::CastleSmallCorridorRightTurn,
        -1,
        0,
        5,
        7,
        5,
    ),
    (
        NetherFortressPieceKindModel::CastleStalkRoom,
        -5,
        -3,
        13,
        14,
        13,
    ),
    (NetherFortressPieceKindModel::MonsterThrone, -2, 0, 7, 8, 9),
    (NetherFortressPieceKindModel::RoomCrossing, -2, 0, 7, 9, 7),
    (NetherFortressPieceKindModel::StairsRoom, -2, 0, 7, 11, 7),
];

#[test]
fn nether_fortress_piece_boxes_match_vanilla_specs_in_every_orientation() {
    let directions = [
        HorizontalDirection::North,
        HorizontalDirection::South,
        HorizontalDirection::West,
        HorizontalDirection::East,
    ];
    let foot = BlockPos {
        x: 37,
        y: 64,
        z: -21,
    };

    for (kind, offset_x, offset_y, width, height, depth) in NETHER_FORTRESS_PIECE_BOX_SPECS {
        for direction in directions {
            let expected = super::structure_orient_box(
                foot,
                BlockPos {
                    x: offset_x,
                    y: offset_y,
                    z: 0,
                },
                width,
                height,
                depth,
                direction,
            );
            assert_eq!(
                super::nether_fortress_piece_box(kind, foot.x, foot.y, foot.z, direction, &[]),
                Some(expected),
                "piece {kind:?} facing {direction:?}"
            );
            if kind == NetherFortressPieceKindModel::BridgeCrossing {
                assert_eq!(
                    super::nether_fortress_bridge_crossing_box(
                        foot.x,
                        foot.y,
                        foot.z,
                        direction,
                        &[]
                    ),
                    Some(expected)
                );
            }
            if kind == NetherFortressPieceKindModel::BridgeStraight {
                assert_eq!(
                    super::nether_fortress_bridge_straight_box(
                        foot.x,
                        foot.y,
                        foot.z,
                        direction,
                        &[]
                    ),
                    Some(expected)
                );
            }
        }
    }
}

#[test]
fn nether_fortress_bridge_end_filler_boxes_match_literal_orientations() {
    use super::{HorizontalDirection, NetherFortressPieceKindModel, StructureBoundingBoxModel};

    let cases = [
        (
            HorizontalDirection::North,
            StructureBoundingBoxModel {
                min_x: 36,
                min_y: 61,
                min_z: -28,
                max_x: 40,
                max_y: 70,
                max_z: -21,
            },
        ),
        (
            HorizontalDirection::South,
            StructureBoundingBoxModel {
                min_x: 36,
                min_y: 61,
                min_z: -21,
                max_x: 40,
                max_y: 70,
                max_z: -14,
            },
        ),
        (
            HorizontalDirection::West,
            StructureBoundingBoxModel {
                min_x: 30,
                min_y: 61,
                min_z: -22,
                max_x: 37,
                max_y: 70,
                max_z: -18,
            },
        ),
        (
            HorizontalDirection::East,
            StructureBoundingBoxModel {
                min_x: 37,
                min_y: 61,
                min_z: -22,
                max_x: 44,
                max_y: 70,
                max_z: -18,
            },
        ),
    ];

    for (direction, expected) in cases {
        assert_eq!(
            super::nether_fortress_piece_box(
                NetherFortressPieceKindModel::BridgeEndFiller,
                37,
                64,
                -21,
                direction,
                &[]
            ),
            Some(expected),
            "BridgeEndFiller facing {direction:?}"
        );
    }
}

#[test]
fn nether_fortress_piece_box_rejects_low_and_colliding_boxes() {
    use super::{
        HorizontalDirection, NetherFortressPieceKindModel, StructureBoundingBoxModel,
        StructurePieceModel,
    };

    assert_eq!(
        super::nether_fortress_piece_box(
            NetherFortressPieceKindModel::BridgeEndFiller,
            0,
            13,
            0,
            HorizontalDirection::North,
            &[]
        ),
        None,
        "BridgeEndFiller minY is 10, which vanilla rejects"
    );
    let accepted = super::nether_fortress_piece_box(
        NetherFortressPieceKindModel::BridgeEndFiller,
        0,
        14,
        0,
        HorizontalDirection::North,
        &[],
    );
    assert!(accepted.is_some());
    let collision = StructurePieceModel {
        bounding_box: StructureBoundingBoxModel {
            min_x: -1,
            min_y: 11,
            min_z: -8,
            max_x: 3,
            max_y: 20,
            max_z: -1,
        },
    };
    assert_eq!(
        super::nether_fortress_piece_box(
            NetherFortressPieceKindModel::BridgeEndFiller,
            0,
            14,
            0,
            HorizontalDirection::North,
            &[collision]
        ),
        None
    );
}
