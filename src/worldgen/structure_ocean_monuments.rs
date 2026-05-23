use super::*;

pub fn ocean_monument_generation_point(
    chunk_pos: ChunkPos,
    sea_level: i32,
) -> OceanMonumentGenerationPointModel {
    OceanMonumentGenerationPointModel {
        biome_check_center: BlockPos {
            x: chunk_pos.x * 16 + 9,
            y: sea_level,
            z: chunk_pos.z * 16 + 9,
        },
        biome_check_radius: 29,
        heightmap: "OCEAN_FLOOR_WG",
    }
}

pub fn ocean_monument_top_piece_origin(chunk_pos: ChunkPos) -> BlockPos {
    BlockPos {
        x: chunk_pos.x * 16 - 29,
        y: 39,
        z: chunk_pos.z * 16 - 29,
    }
}

pub fn ocean_monument_room_index(room_x: i32, room_y: i32, room_z: i32) -> i32 {
    room_y * 25 + room_z * 5 + room_x
}

pub fn ocean_monument_direction_3d_value(direction: OceanMonumentDirectionModel) -> i32 {
    match direction {
        OceanMonumentDirectionModel::Down => 0,
        OceanMonumentDirectionModel::Up => 1,
        OceanMonumentDirectionModel::North => 2,
        OceanMonumentDirectionModel::South => 3,
        OceanMonumentDirectionModel::West => 4,
        OceanMonumentDirectionModel::East => 5,
    }
}

pub fn ocean_monument_opposite_direction(
    direction: OceanMonumentDirectionModel,
) -> OceanMonumentDirectionModel {
    match direction {
        OceanMonumentDirectionModel::Down => OceanMonumentDirectionModel::Up,
        OceanMonumentDirectionModel::Up => OceanMonumentDirectionModel::Down,
        OceanMonumentDirectionModel::North => OceanMonumentDirectionModel::South,
        OceanMonumentDirectionModel::South => OceanMonumentDirectionModel::North,
        OceanMonumentDirectionModel::West => OceanMonumentDirectionModel::East,
        OceanMonumentDirectionModel::East => OceanMonumentDirectionModel::West,
    }
}

pub fn ocean_monument_building(
    chunk_pos: ChunkPos,
    direction_roll: i32,
    core_room_x_roll: i32,
) -> Result<OceanMonumentBuildingModel, String> {
    if !(0..4).contains(&core_room_x_roll) {
        return Err("Ocean monument core room roll must match RandomSource#nextInt(4)".to_string());
    }
    let orientation = stronghold_horizontal_direction_from_random_roll(direction_roll)?;
    let origin = ocean_monument_top_piece_origin(chunk_pos);
    let bounding_box =
        structure_make_bounding_box(origin.x, origin.y, origin.z, orientation, 58, 23, 58);
    Ok(OceanMonumentBuildingModel {
        bounding_box,
        orientation,
        source_room_index: ocean_monument_room_index(2, 0, 0),
        core_room_index: ocean_monument_room_index(core_room_x_roll, 0, 2),
        top_connect_index: ocean_monument_room_index(2, 2, 0),
        left_wing_connect_index: ocean_monument_room_index(0, 1, 0),
        right_wing_connect_index: ocean_monument_room_index(4, 1, 0),
        child_piece_offset: structure_piece_world_pos(bounding_box, Some(orientation), 9, 0, 22),
    })
}

pub fn ocean_monument_room_definition(
    index: i32,
    claimed: bool,
    is_source: bool,
    openings: [bool; 6],
) -> OceanMonumentRoomDefinitionModel {
    OceanMonumentRoomDefinitionModel {
        index,
        claimed,
        is_source,
        is_special: index >= 75,
        opening_count: openings.iter().filter(|&&opening| opening).count() as i32,
    }
}

pub fn ocean_monument_room_box(
    room_index: i32,
    room_width: i32,
    room_height: i32,
    room_depth: i32,
    orientation: HorizontalDirection,
) -> StructureBoundingBoxModel {
    let room_x = room_index % 5;
    let room_z = room_index / 5 % 5;
    let room_y = room_index / 25;
    let base = structure_make_bounding_box(
        0,
        0,
        0,
        orientation,
        room_width * 8,
        room_height * 4,
        room_depth * 8,
    );
    match orientation {
        HorizontalDirection::North => {
            base.moved(room_x * 8, room_y * 4, -(room_z + room_depth) * 8 + 1)
        }
        HorizontalDirection::South => base.moved(room_x * 8, room_y * 4, room_z * 8),
        HorizontalDirection::West => {
            base.moved(-(room_z + room_depth) * 8 + 1, room_y * 4, room_x * 8)
        }
        HorizontalDirection::East => base.moved(room_z * 8, room_y * 4, room_x * 8),
    }
}

pub fn ocean_monument_elder_spawn_pos(
    piece_box: StructureBoundingBoxModel,
    orientation: HorizontalDirection,
    local_pos: BlockPos,
    chunk_bb: StructureBoundingBoxModel,
) -> Option<BlockPos> {
    let world_pos = structure_piece_world_pos(
        piece_box,
        Some(orientation),
        local_pos.x,
        local_pos.y,
        local_pos.z,
    );
    chunk_bb.is_inside(world_pos).then_some(world_pos)
}

