#[test]
fn pool_element_structure_piece_state_and_junction_y_math_match_vanilla() {
    assert_pool_element_structure_piece_state_matches_vanilla();
    assert_rigid_jigsaw_child_y_placement_matches_vanilla();
    assert_terrain_jigsaw_child_y_placement_matches_vanilla();
}

fn assert_pool_element_structure_piece_state_matches_vanilla() {
    let element = super::super::JigsawPoolElementModel::single(
        "minecraft:bastion/starts/start",
        &[],
        super::super::JigsawProjectionModel::Rigid,
        None,
    );
    let mut piece = super::super::PoolElementStructurePieceModel::new(
        element,
        (10, 64, -8),
        1,
        super::super::StructurePieceRotation::Clockwise90,
        super::super::StructureBoundingBoxModel {
            min_x: 10,
            min_y: 64,
            min_z: -8,
            max_x: 20,
            max_y: 72,
            max_z: 2,
        },
        super::super::PoolElementStructurePieceModel::DEFAULT_LIQUID_SETTINGS,
    );
    piece.add_junction(super::super::JigsawJunctionModel {
        source_x: 21,
        source_ground_y: 66,
        source_z: -2,
        delta_y: 0,
        dest_projection: super::super::JigsawProjectionModel::Rigid,
    });
    piece.move_by(1, -2, 3);
    assert_eq!(piece.position, (11, 62, -5));
    assert_eq!(
        piece.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: 11,
            min_y: 62,
            min_z: -5,
            max_x: 21,
            max_y: 70,
            max_z: 5,
        }
    );
    assert_eq!(
        piece.save_tag(),
        super::super::PoolElementStructurePieceTagModel {
            pos_x: 11,
            pos_y: 62,
            pos_z: -5,
            ground_level_delta: 1,
            rotation: super::super::StructurePieceRotation::Clockwise90,
            junctions: vec![super::super::JigsawJunctionTagModel {
                source_x: 21,
                source_ground_y: 66,
                source_z: -2,
                delta_y: 0,
                dest_proj: "rigid",
            }],
            liquid_settings: None,
        }
    );
    piece.liquid_settings = super::super::LiquidSettingsModel::IgnoreWaterlogging;
    assert_eq!(
        piece.save_tag().liquid_settings,
        Some(super::super::LiquidSettingsModel::IgnoreWaterlogging)
    );
}

fn assert_rigid_jigsaw_child_y_placement_matches_vanilla() {
    let both_rigid =
        super::super::jigsaw_child_placement_y(super::super::JigsawChildPlacementYInput {
            source_projection: super::super::JigsawProjectionModel::Rigid,
            target_projection: super::super::JigsawProjectionModel::Rigid,
            source_box_y: 64,
            source_jigsaw_local_y: 3,
            target_jigsaw_local_y: 1,
            source_direction_step_y: 0,
            source_ground_level_delta: 5,
            target_ground_level_delta: 1,
            source_jigsaw_base_height: 90,
        });
    assert_eq!(
        both_rigid,
        super::super::JigsawChildPlacementYModel {
            target_box_y: 66,
            target_ground_level_delta: 3,
            junction_y: 67,
            case: super::super::JigsawJunctionYOffsetCase::BothRigid,
        }
    );

    let terrain_target =
        super::super::jigsaw_child_placement_y(super::super::JigsawChildPlacementYInput {
            source_projection: super::super::JigsawProjectionModel::Rigid,
            target_projection: super::super::JigsawProjectionModel::TerrainMatching,
            source_box_y: 64,
            source_jigsaw_local_y: 3,
            target_jigsaw_local_y: 1,
            source_direction_step_y: 0,
            source_ground_level_delta: 5,
            target_ground_level_delta: 1,
            source_jigsaw_base_height: 90,
        });
    assert_eq!(terrain_target.target_box_y, 89);
    assert_eq!(terrain_target.target_ground_level_delta, 1);
    assert_eq!(
        terrain_target.case,
        super::super::JigsawJunctionYOffsetCase::SourceRigid
    );
}

fn assert_terrain_jigsaw_child_y_placement_matches_vanilla() {
    let both_terrain =
        super::super::jigsaw_child_placement_y(super::super::JigsawChildPlacementYInput {
            source_projection: super::super::JigsawProjectionModel::TerrainMatching,
            target_projection: super::super::JigsawProjectionModel::TerrainMatching,
            source_box_y: 64,
            source_jigsaw_local_y: 4,
            target_jigsaw_local_y: 1,
            source_direction_step_y: -1,
            source_ground_level_delta: 2,
            target_ground_level_delta: 1,
            source_jigsaw_base_height: 91,
        });
    let delta_y = 4 - 1 - 1;
    assert_eq!(both_terrain.target_box_y, 90);
    assert_eq!(both_terrain.junction_y, 91 + delta_y / 2);
    assert_eq!(
        super::super::jigsaw_source_junction(
            (30, 68, -4),
            both_terrain,
            4,
            2,
            delta_y,
            super::super::JigsawProjectionModel::TerrainMatching,
        ),
        super::super::JigsawJunctionModel {
            source_x: 30,
            source_ground_y: 90,
            source_z: -4,
            delta_y,
            dest_projection: super::super::JigsawProjectionModel::TerrainMatching,
        }
    );
    assert_eq!(
        super::super::jigsaw_target_junction(
            (29, 67, -4),
            both_terrain,
            1,
            1,
            delta_y,
            super::super::JigsawProjectionModel::TerrainMatching,
        ),
        super::super::JigsawJunctionModel {
            source_x: 29,
            source_ground_y: 92,
            source_z: -4,
            delta_y: -delta_y,
            dest_projection: super::super::JigsawProjectionModel::TerrainMatching,
        }
    );
}

#[test]
fn jigsaw_child_box_placement_matches_java_raw_box_and_y_offset_math() {
    let placement =
        super::super::jigsaw_child_placement_y(super::super::JigsawChildPlacementYInput {
            source_projection: super::super::JigsawProjectionModel::Rigid,
            target_projection: super::super::JigsawProjectionModel::Rigid,
            source_box_y: 64,
            source_jigsaw_local_y: 5,
            target_jigsaw_local_y: 2,
            source_direction_step_y: 1,
            source_ground_level_delta: 0,
            target_ground_level_delta: 0,
            source_jigsaw_base_height: 90,
        });
    assert_eq!(placement.target_box_y, 68);

    let box_placement = super::super::jigsaw_child_box_placement(
        super::super::BlockPos {
            x: 101,
            y: 70,
            z: -31,
        },
        super::super::BlockPos { x: 4, y: 2, z: 6 },
        super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 9,
            max_y: 7,
            max_z: 11,
        },
        placement,
    );

    assert_eq!(
        box_placement,
        super::super::JigsawChildBoxPlacementModel {
            raw_target_box_pos: super::super::BlockPos {
                x: 97,
                y: 68,
                z: -37,
            },
            raw_target_bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 97,
                min_y: 68,
                min_z: -37,
                max_x: 106,
                max_y: 75,
                max_z: -26,
            },
            y_offset: 0,
            target_box_position: super::super::BlockPos {
                x: 97,
                y: 68,
                z: -37,
            },
            target_bounding_box: super::super::StructureBoundingBoxModel {
                min_x: 97,
                min_y: 68,
                min_z: -37,
                max_x: 106,
                max_y: 75,
                max_z: -26,
            },
        }
    );

    let terrain_placement =
        super::super::jigsaw_child_placement_y(super::super::JigsawChildPlacementYInput {
            source_projection: super::super::JigsawProjectionModel::TerrainMatching,
            target_projection: super::super::JigsawProjectionModel::Rigid,
            source_box_y: 64,
            source_jigsaw_local_y: 5,
            target_jigsaw_local_y: 2,
            source_direction_step_y: -1,
            source_ground_level_delta: 0,
            target_ground_level_delta: 0,
            source_jigsaw_base_height: 91,
        });
    let moved = super::super::jigsaw_child_box_placement(
        super::super::BlockPos {
            x: 101,
            y: 70,
            z: -31,
        },
        super::super::BlockPos { x: 4, y: 2, z: 6 },
        super::super::StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 9,
            max_y: 7,
            max_z: 11,
        },
        terrain_placement,
    );
    assert_eq!(terrain_placement.target_box_y, 89);
    assert_eq!(moved.y_offset, 21);
    assert_eq!(moved.target_box_position.y, 89);
    assert_eq!(moved.target_bounding_box.min_y, 89);
    assert_eq!(moved.target_bounding_box.max_y, 96);
}

#[test]
fn jigsaw_expansion_hack_box_encapsulation_matches_java_height_math() {
    let target_box = super::super::StructureBoundingBoxModel {
        min_x: 10,
        min_y: 64,
        min_z: -4,
        max_x: 18,
        max_y: 71,
        max_z: 4,
    };
    assert_eq!(
        super::super::jigsaw_apply_expansion_hack_to_target_box(target_box, 0),
        target_box
    );
    assert_eq!(
        super::super::jigsaw_apply_expansion_hack_to_target_box(target_box, 2),
        target_box
    );
    assert_eq!(
        super::super::jigsaw_apply_expansion_hack_to_target_box(target_box, 9),
        super::super::StructureBoundingBoxModel {
            max_y: 74,
            ..target_box
        }
    );
    assert_eq!(
        super::super::jigsaw_apply_expansion_hack_to_target_box(target_box, -1),
        target_box
    );
}

#[test]
fn jigsaw_child_free_shape_selection_matches_java_source_inside_check() {
    let source_box = super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: 64,
        min_z: 0,
        max_x: 15,
        max_y: 79,
        max_z: 15,
    };
    assert_eq!(
        super::super::jigsaw_child_free_shape_selection(
            source_box,
            super::super::BlockPos {
                x: 15,
                y: 79,
                z: 15
            },
            false,
        ),
        super::super::JigsawChildFreeShapeSelectionModel {
            scope: super::super::JigsawChildFreeShapeScope::SourcePiece,
            initialized_source_shape: Some(source_box),
        }
    );
    assert_eq!(
        super::super::jigsaw_child_free_shape_selection(
            source_box,
            super::super::BlockPos { x: 4, y: 70, z: 4 },
            true,
        ),
        super::super::JigsawChildFreeShapeSelectionModel {
            scope: super::super::JigsawChildFreeShapeScope::SourcePiece,
            initialized_source_shape: None,
        }
    );
    assert_eq!(
        super::super::jigsaw_child_free_shape_selection(
            source_box,
            super::super::BlockPos { x: 16, y: 70, z: 4 },
            false,
        ),
        super::super::JigsawChildFreeShapeSelectionModel {
            scope: super::super::JigsawChildFreeShapeScope::Context,
            initialized_source_shape: None,
        }
    );
}

#[test]
fn jigsaw_accepted_child_scheduling_matches_java_depth_and_priority_rule() {
    assert_eq!(
        super::super::jigsaw_accepted_child_scheduling(0, 1, 7),
        super::super::JigsawAcceptedChildSchedulingModel {
            child_depth: 1,
            queue_for_expansion: true,
            placement_priority: 7,
        }
    );
    assert_eq!(
        super::super::jigsaw_accepted_child_scheduling(1, 1, -3),
        super::super::JigsawAcceptedChildSchedulingModel {
            child_depth: 2,
            queue_for_expansion: false,
            placement_priority: -3,
        }
    );
    assert_eq!(
        super::super::jigsaw_accepted_child_scheduling(19, 20, i32::MAX),
        super::super::JigsawAcceptedChildSchedulingModel {
            child_depth: 20,
            queue_for_expansion: true,
            placement_priority: i32::MAX,
        }
    );
}
