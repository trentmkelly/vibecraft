use super::*;

#[test]
fn ocean_monument_generation_point_and_building_match_vanilla() {
    let chunk_pos = ChunkPos { x: -2, z: 3 };
    assert_eq!(
        super::super::ocean_monument_generation_point(chunk_pos, 63),
        super::super::OceanMonumentGenerationPointModel {
            biome_check_center: BlockPos {
                x: -23,
                y: 63,
                z: 57,
            },
            biome_check_radius: 29,
            heightmap: "OCEAN_FLOOR_WG",
        }
    );
    assert_eq!(
        super::super::ocean_monument_top_piece_origin(chunk_pos),
        BlockPos {
            x: -61,
            y: 39,
            z: 19,
        }
    );

    let building = super::super::ocean_monument_building(chunk_pos, 0, 3).unwrap();
    assert_eq!(
        building.orientation,
        super::super::HorizontalDirection::North
    );
    assert_eq!(building.source_room_index, 2);
    assert_eq!(building.core_room_index, 13);
    assert_eq!(building.top_connect_index, 52);
    assert_eq!(building.left_wing_connect_index, 25);
    assert_eq!(building.right_wing_connect_index, 29);
    assert_eq!(
        building.bounding_box,
        super::super::StructureBoundingBoxModel {
            min_x: -61,
            min_y: 39,
            min_z: 19,
            max_x: -4,
            max_y: 61,
            max_z: 76,
        }
    );
    assert_eq!(
        building.child_piece_offset,
        BlockPos {
            x: -52,
            y: 39,
            z: 54,
        }
    );
    assert_eq!(
        super::super::ocean_monument_building(chunk_pos, 0, 4).unwrap_err(),
        "Ocean monument core room roll must match RandomSource#nextInt(4)".to_string()
    );
}

#[test]
fn ocean_monument_room_primitives_match_vanilla() {
    assert_eq!(super::super::ocean_monument_room_index(2, 0, 0), 2);
    assert_eq!(super::super::ocean_monument_room_index(2, 2, 0), 52);
    assert_eq!(super::super::ocean_monument_room_index(0, 1, 0), 25);
    assert_eq!(super::super::ocean_monument_room_index(4, 1, 0), 29);
    assert_eq!(
        super::super::ocean_monument_direction_3d_value(
            super::super::OceanMonumentDirectionModel::East
        ),
        5
    );
    assert_eq!(
        super::super::ocean_monument_opposite_direction(
            super::super::OceanMonumentDirectionModel::North
        ),
        super::super::OceanMonumentDirectionModel::South
    );
    assert_eq!(
        super::super::ocean_monument_room_definition(
            1003,
            true,
            false,
            [true, false, true, false, false, true],
        ),
        super::super::OceanMonumentRoomDefinitionModel {
            index: 1003,
            claimed: true,
            is_source: false,
            is_special: true,
            opening_count: 3,
        }
    );
    assert_eq!(
        super::super::ocean_monument_room_box(
            13,
            2,
            2,
            2,
            super::super::HorizontalDirection::South
        ),
        super::super::StructureBoundingBoxModel {
            min_x: 24,
            min_y: 0,
            min_z: 16,
            max_x: 39,
            max_y: 7,
            max_z: 31,
        }
    );
    assert_eq!(
        super::super::ocean_monument_room_box(
            13,
            2,
            2,
            2,
            super::super::HorizontalDirection::North
        ),
        super::super::StructureBoundingBoxModel {
            min_x: 24,
            min_y: 0,
            min_z: -31,
            max_x: 39,
            max_y: 7,
            max_z: -16,
        }
    );
}

#[test]
fn ocean_monument_elder_spawn_positions_match_vanilla() {
    let building = super::super::ocean_monument_building(ChunkPos { x: -2, z: 3 }, 0, 3).unwrap();
    let chunk_bb = building.bounding_box;
    assert_eq!(
        super::super::ocean_monument_elder_spawn_pos(
            building.bounding_box,
            building.orientation,
            BlockPos { x: 6, y: 1, z: 6 },
            chunk_bb,
        ),
        Some(BlockPos {
            x: -55,
            y: 40,
            z: 70,
        })
    );
    assert_eq!(
        super::super::ocean_monument_elder_spawn_pos(
            building.bounding_box,
            building.orientation,
            BlockPos { x: 500, y: 1, z: 6 },
            chunk_bb,
        ),
        None
    );
}

#[test]
fn end_city_generation_start_and_template_ids_match_vanilla() {
    assert_eq!(
        super::super::end_city_rotation_from_roll(0).unwrap(),
        super::super::StructureRotation::None
    );
    assert_eq!(
        super::super::end_city_rotation_from_roll(3).unwrap(),
        super::super::StructureRotation::Counterclockwise90
    );
    assert_eq!(
        super::super::end_city_rotation_from_roll(4).unwrap_err(),
        "End city rotation roll must match RandomSource#nextInt(4)".to_string()
    );
    assert_eq!(
        super::super::end_city_generation_start(BlockPos { x: 8, y: 59, z: 8 }),
        None
    );
    assert_eq!(
        super::super::end_city_generation_start(BlockPos { x: 8, y: 60, z: 8 }),
        Some(BlockPos { x: 8, y: 60, z: 8 })
    );
    assert_eq!(
        super::super::end_city_template_id("ship"),
        "minecraft:end_city/ship".to_string()
    );
}

#[test]
fn end_city_tower_seed_and_bridge_candidates_match_vanilla() {
    let pieces = super::super::end_city_start_house_tower_seed(
        BlockPos {
            x: 16,
            y: 70,
            z: -8,
        },
        super::super::StructureRotation::Clockwise90,
    );
    assert_eq!(pieces.len(), 4);
    assert_eq!(pieces[0].template_name, "base_floor");
    assert_eq!(pieces[0].template_id, "minecraft:end_city/base_floor");
    assert!(pieces[0].overwrite);
    assert_eq!(pieces[0].processor, "STRUCTURE_BLOCK");
    assert_eq!(pieces[1].template_name, "second_floor_1");
    assert!(!pieces[1].overwrite);
    assert_eq!(pieces[1].processor, "STRUCTURE_AND_AIR");
    assert_eq!(
        pieces[3],
        super::super::EndCityTemplatePieceModel {
            template_name: "third_roof",
            template_id: "minecraft:end_city/third_roof",
            position: BlockPos {
                x: 13,
                y: 82,
                z: -11,
            },
            rotation: super::super::StructureRotation::Clockwise90,
            overwrite: true,
            processor: "STRUCTURE_BLOCK",
            gen_depth: 0,
        }
    );

    assert_eq!(
        super::super::end_city_tower_bridge_candidates(),
        vec![
            super::super::EndCityBridgeCandidateModel {
                rotation: super::super::StructureRotation::None,
                offset: BlockPos { x: 1, y: -1, z: 0 },
            },
            super::super::EndCityBridgeCandidateModel {
                rotation: super::super::StructureRotation::Clockwise90,
                offset: BlockPos { x: 6, y: -1, z: 1 },
            },
            super::super::EndCityBridgeCandidateModel {
                rotation: super::super::StructureRotation::Counterclockwise90,
                offset: BlockPos { x: 0, y: -1, z: 5 },
            },
            super::super::EndCityBridgeCandidateModel {
                rotation: super::super::StructureRotation::Clockwise180,
                offset: BlockPos { x: 5, y: -1, z: 6 },
            },
        ]
    );
    assert_eq!(
        super::super::end_city_fat_tower_bridge_candidates()[1],
        super::super::EndCityBridgeCandidateModel {
            rotation: super::super::StructureRotation::Clockwise90,
            offset: BlockPos { x: 12, y: -1, z: 4 },
        }
    );
}

#[test]
fn end_city_markers_match_vanilla() {
    let chunk_bb = test_chunk_bounding_box();
    let marker_pos = BlockPos {
        x: 10,
        y: 20,
        z: 10,
    };
    assert_eq!(
        super::super::end_city_marker_action(
            "ChestLoot",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            true,
        ),
        Some(super::super::EndCityMarkerActionModel {
            marker_id: "ChestLoot",
            target_pos: BlockPos {
                x: 10,
                y: 19,
                z: 10,
            },
            loot_table: Some("minecraft:chests/end_city_treasure"),
            spawned_entity: None,
            item_frame_facing: None,
            item: None,
        })
    );
    assert_eq!(
        super::super::end_city_marker_action(
            "Sentry",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            true,
        )
        .unwrap()
        .spawned_entity,
        Some("minecraft:shulker")
    );
    assert_eq!(
        super::super::end_city_marker_action(
            "Elytra",
            marker_pos,
            super::super::StructureRotation::Clockwise90,
            chunk_bb,
            true,
        ),
        Some(super::super::EndCityMarkerActionModel {
            marker_id: "Elytra",
            target_pos: marker_pos,
            loot_table: None,
            spawned_entity: Some("minecraft:item_frame"),
            item_frame_facing: Some(super::super::HorizontalDirection::West),
            item: Some("minecraft:elytra"),
        })
    );
    assert_eq!(
        super::super::end_city_marker_action(
            "Sentry",
            BlockPos {
                x: 100,
                y: 20,
                z: 10,
            },
            super::super::StructureRotation::None,
            chunk_bb,
            true,
        ),
        None
    );
}

fn test_chunk_bounding_box() -> super::super::StructureBoundingBoxModel {
    super::super::StructureBoundingBoxModel {
        min_x: 0,
        min_y: 0,
        min_z: 0,
        max_x: 32,
        max_y: 100,
        max_z: 32,
    }
}

#[test]
fn woodland_mansion_generation_start_and_template_ids_match_vanilla() {
    assert_eq!(
        super::super::woodland_mansion_generation_start(BlockPos { x: 0, y: 59, z: 0 }),
        None
    );
    assert_eq!(
        super::super::woodland_mansion_generation_start(BlockPos { x: 0, y: 60, z: 0 }),
        Some(BlockPos { x: 0, y: 60, z: 0 })
    );
    assert_eq!(
        super::super::woodland_mansion_template_id("entrance"),
        "minecraft:woodland_mansion/entrance".to_string()
    );
}

#[test]
fn woodland_mansion_initial_placement_and_room_rules_match_vanilla() {
    let (entrance, data) = super::super::woodland_mansion_initial_placement(
        BlockPos {
            x: 100,
            y: 70,
            z: -40,
        },
        super::super::StructureRotation::Clockwise90,
    );
    assert_eq!(
        entrance,
        super::super::WoodlandMansionTemplatePieceModel {
            template_name: "entrance",
            template_id: "minecraft:woodland_mansion/entrance",
            position: BlockPos {
                x: 100,
                y: 70,
                z: -49,
            },
            rotation: super::super::StructureRotation::Clockwise90,
            mirror: super::super::WoodlandMansionMirrorModel::None,
            processor: "STRUCTURE_BLOCK",
        }
    );
    assert_eq!(
        data,
        super::super::WoodlandMansionPlacementDataModel {
            position: BlockPos {
                x: 84,
                y: 70,
                z: -40,
            },
            rotation: super::super::StructureRotation::Clockwise90,
            wall_type: "wall_flat",
        }
    );

    let second = super::super::woodland_mansion_second_floor_data(data);
    assert_eq!(second.position.y, 78);
    assert_eq!(second.wall_type, "wall_window");
    let (wall, next_data) = super::super::woodland_mansion_traverse_wall_piece(data);
    assert_eq!(wall.template_name, "wall_flat");
    assert_eq!(
        wall.position,
        BlockPos {
            x: 84,
            y: 70,
            z: -33,
        }
    );
    assert_eq!(
        next_data.position,
        BlockPos {
            x: 76,
            y: 70,
            z: -40,
        }
    );

    let room = super::super::woodland_mansion_add_room_1x1(
        BlockPos { x: 0, y: 80, z: 0 },
        super::super::StructureRotation::Clockwise90,
        Some(super::super::HorizontalDirection::South),
        "1x1_a1",
    );
    assert_eq!(room.rotation, super::super::StructureRotation::Clockwise180);
    assert_eq!(room.template_id, "minecraft:woodland_mansion/1x1_a1");
    assert_eq!(
        super::super::woodland_mansion_add_room_1x1(
            BlockPos { x: 0, y: 80, z: 0 },
            super::super::StructureRotation::None,
            None,
            "1x1_a1",
        )
        .template_name,
        "1x1_as1"
    );
}

#[test]
fn woodland_mansion_markers_match_vanilla() {
    let chunk_bb = test_chunk_bounding_box();
    let marker_pos = BlockPos {
        x: 10,
        y: 20,
        z: 10,
    };
    assert_eq!(
        super::super::woodland_mansion_marker_action(
            "ChestWest",
            marker_pos,
            super::super::StructureRotation::Clockwise90,
            chunk_bb,
            0,
        )
        .unwrap(),
        Some(super::super::WoodlandMansionMarkerActionModel {
            marker_id: "ChestWest",
            target_pos: marker_pos,
            loot_table: Some("minecraft:chests/woodland_mansion"),
            chest_facing: Some(super::super::HorizontalDirection::North),
            spawned_entity: None,
            spawn_count: 0,
            clears_marker_block: false,
        })
    );
    assert_eq!(
        super::super::woodland_mansion_marker_action(
            "Mage",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            0,
        )
        .unwrap()
        .unwrap()
        .spawned_entity,
        Some("minecraft:evoker")
    );
    assert_eq!(
        super::super::woodland_mansion_marker_action(
            "Group of Allays",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            2,
        )
        .unwrap()
        .unwrap()
        .spawn_count,
        3
    );
    assert_eq!(
        super::super::woodland_mansion_marker_action(
            "Group of Allays",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            3,
        )
        .unwrap_err(),
        "Woodland mansion allay group roll must match RandomSource#nextInt(3)".to_string()
    );
    assert_eq!(
        super::super::woodland_mansion_marker_action(
            "Unknown",
            marker_pos,
            super::super::StructureRotation::None,
            chunk_bb,
            0,
        )
        .unwrap(),
        None
    );
}

#[test]
fn woodland_mansion_support_columns_match_vanilla() {
    assert_eq!(
        super::super::woodland_mansion_support_column_y_values(70, 60, true, Some(66)),
        vec![69, 68, 67]
    );
    assert!(
        super::super::woodland_mansion_support_column_y_values(70, 60, false, Some(66)).is_empty()
    );
}
