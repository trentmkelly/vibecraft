use super::*;

pub fn woodland_mansion_generation_start(start_pos: BlockPos) -> Option<BlockPos> {
    (start_pos.y >= 60).then_some(start_pos)
}

pub fn woodland_mansion_template_id(template_name: &'static str) -> String {
    format!("minecraft:woodland_mansion/{template_name}")
}

pub fn woodland_mansion_template_piece(
    template_name: &'static str,
    position: BlockPos,
    rotation: StructureRotation,
    mirror: WoodlandMansionMirrorModel,
) -> WoodlandMansionTemplatePieceModel {
    WoodlandMansionTemplatePieceModel {
        template_name,
        template_id: match template_name {
            "entrance" => "minecraft:woodland_mansion/entrance",
            "wall_flat" => "minecraft:woodland_mansion/wall_flat",
            "wall_window" => "minecraft:woodland_mansion/wall_window",
            "wall_corner" => "minecraft:woodland_mansion/wall_corner",
            "small_wall" => "minecraft:woodland_mansion/small_wall",
            "small_wall_corner" => "minecraft:woodland_mansion/small_wall_corner",
            "roof_front" => "minecraft:woodland_mansion/roof_front",
            "roof_corner" => "minecraft:woodland_mansion/roof_corner",
            "roof_inner_corner" => "minecraft:woodland_mansion/roof_inner_corner",
            "1x1_a1" => "minecraft:woodland_mansion/1x1_a1",
            "1x1_as1" => "minecraft:woodland_mansion/1x1_as1",
            "1x2_a1" => "minecraft:woodland_mansion/1x2_a1",
            "1x2_b1" => "minecraft:woodland_mansion/1x2_b1",
            "1x2_s1" => "minecraft:woodland_mansion/1x2_s1",
            "2x2_a1" => "minecraft:woodland_mansion/2x2_a1",
            "2x2_s1" => "minecraft:woodland_mansion/2x2_s1",
            other => {
                let _ = other;
                "minecraft:woodland_mansion/unknown"
            }
        },
        position,
        rotation,
        mirror,
        processor: "STRUCTURE_BLOCK",
    }
}

pub fn woodland_mansion_initial_placement(
    origin: BlockPos,
    rotation: StructureRotation,
) -> (
    WoodlandMansionTemplatePieceModel,
    WoodlandMansionPlacementDataModel,
) {
    let west = structure_rotation_rotate_direction(rotation, HorizontalDirection::West);
    let entrance_pos = block_pos_relative(origin, west, 9);
    let next_pos = block_pos_relative(
        origin,
        structure_rotation_rotate_direction(rotation, HorizontalDirection::South),
        16,
    );
    (
        woodland_mansion_template_piece(
            "entrance",
            entrance_pos,
            rotation,
            WoodlandMansionMirrorModel::None,
        ),
        WoodlandMansionPlacementDataModel {
            position: next_pos,
            rotation,
            wall_type: "wall_flat",
        },
    )
}

pub fn woodland_mansion_second_floor_data(
    data: WoodlandMansionPlacementDataModel,
) -> WoodlandMansionPlacementDataModel {
    WoodlandMansionPlacementDataModel {
        position: BlockPos {
            y: data.position.y + 8,
            ..data.position
        },
        wall_type: "wall_window",
        ..data
    }
}

pub fn woodland_mansion_traverse_wall_piece(
    data: WoodlandMansionPlacementDataModel,
) -> (
    WoodlandMansionTemplatePieceModel,
    WoodlandMansionPlacementDataModel,
) {
    let east = structure_rotation_rotate_direction(data.rotation, HorizontalDirection::East);
    let south = structure_rotation_rotate_direction(data.rotation, HorizontalDirection::South);
    let piece_pos = block_pos_relative(data.position, east, 7);
    let next_pos = block_pos_relative(data.position, south, 8);
    (
        woodland_mansion_template_piece(
            data.wall_type,
            piece_pos,
            data.rotation,
            WoodlandMansionMirrorModel::None,
        ),
        WoodlandMansionPlacementDataModel {
            position: next_pos,
            ..data
        },
    )
}

pub fn woodland_mansion_add_room_1x1(
    room_pos: BlockPos,
    rotation: StructureRotation,
    door_dir: Option<HorizontalDirection>,
    room_type: &'static str,
) -> WoodlandMansionTemplatePieceModel {
    let mut piece_rot = StructureRotation::None;
    let mut selected_room = room_type;
    match door_dir {
        Some(HorizontalDirection::East) => {}
        Some(HorizontalDirection::North) => {
            piece_rot = structure_rotation_add(piece_rot, StructureRotation::Counterclockwise90);
        }
        Some(HorizontalDirection::West) => {
            piece_rot = structure_rotation_add(piece_rot, StructureRotation::Clockwise180);
        }
        Some(HorizontalDirection::South) => {
            piece_rot = structure_rotation_add(piece_rot, StructureRotation::Clockwise90);
        }
        None => selected_room = "1x1_as1",
    }
    let piece_rot = structure_rotation_add(piece_rot, rotation);
    woodland_mansion_template_piece(
        selected_room,
        room_pos,
        piece_rot,
        WoodlandMansionMirrorModel::None,
    )
}

pub fn woodland_mansion_marker_action(
    marker_id: &'static str,
    position: BlockPos,
    rotation: StructureRotation,
    chunk_bb: StructureBoundingBoxModel,
    allay_roll: i32,
) -> Result<Option<WoodlandMansionMarkerActionModel>, String> {
    if marker_id.starts_with("Chest") {
        let chest_facing = match marker_id {
            "ChestWest" => Some(structure_rotation_rotate_direction(
                rotation,
                HorizontalDirection::West,
            )),
            "ChestEast" => Some(structure_rotation_rotate_direction(
                rotation,
                HorizontalDirection::East,
            )),
            "ChestSouth" => Some(structure_rotation_rotate_direction(
                rotation,
                HorizontalDirection::South,
            )),
            "ChestNorth" => Some(structure_rotation_rotate_direction(
                rotation,
                HorizontalDirection::North,
            )),
            _ => None,
        };
        return Ok(chunk_bb
            .is_inside(position)
            .then_some(WoodlandMansionMarkerActionModel {
                marker_id,
                target_pos: position,
                loot_table: Some("minecraft:chests/woodland_mansion"),
                chest_facing,
                spawned_entity: None,
                spawn_count: 0,
                clears_marker_block: false,
            }));
    }
    let (spawned_entity, spawn_count) = match marker_id {
        "Mage" => (Some("minecraft:evoker"), 1),
        "Warrior" => (Some("minecraft:vindicator"), 1),
        "Group of Allays" => {
            if !(0..3).contains(&allay_roll) {
                return Err(
                    "Woodland mansion allay group roll must match RandomSource#nextInt(3)"
                        .to_string(),
                );
            }
            (Some("minecraft:allay"), allay_roll + 1)
        }
        _ => return Ok(None),
    };
    Ok(Some(WoodlandMansionMarkerActionModel {
        marker_id,
        target_pos: position,
        loot_table: None,
        chest_facing: None,
        spawned_entity,
        spawn_count,
        clears_marker_block: true,
    }))
}

pub fn woodland_mansion_support_column_y_values(
    y_start: i32,
    min_y: i32,
    is_inside_piece: bool,
    first_solid_or_liquid_y: Option<i32>,
) -> Vec<i32> {
    if !is_inside_piece {
        return Vec::new();
    }
    let stop_y = first_solid_or_liquid_y.unwrap_or(min_y);
    ((stop_y + 1)..y_start).rev().collect()
}
