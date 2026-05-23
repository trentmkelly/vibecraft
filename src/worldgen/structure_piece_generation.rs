use super::*;

pub fn structure_make_bounding_box(
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
    width: i32,
    height: i32,
    depth: i32,
) -> StructureBoundingBoxModel {
    match direction {
        HorizontalDirection::North | HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x + width - 1,
            max_y: y + height - 1,
            max_z: z + depth - 1,
        },
        HorizontalDirection::West | HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: x,
            min_y: y,
            min_z: z,
            max_x: x + depth - 1,
            max_y: y + height - 1,
            max_z: z + width - 1,
        },
    }
}

pub fn structure_orient_box(
    foot: BlockPos,
    offset: BlockPos,
    width: i32,
    height: i32,
    depth: i32,
    direction: HorizontalDirection,
) -> StructureBoundingBoxModel {
    match direction {
        HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: foot.x + offset.x,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.z,
            max_x: foot.x + width - 1 + offset.x,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + depth - 1 + offset.z,
        },
        HorizontalDirection::North => StructureBoundingBoxModel {
            min_x: foot.x + offset.x,
            min_y: foot.y + offset.y,
            min_z: foot.z - depth + 1 + offset.z,
            max_x: foot.x + width - 1 + offset.x,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + offset.z,
        },
        HorizontalDirection::West => StructureBoundingBoxModel {
            min_x: foot.x - depth + 1 + offset.z,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.x,
            max_x: foot.x + offset.z,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + width - 1 + offset.x,
        },
        HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: foot.x + offset.z,
            min_y: foot.y + offset.y,
            min_z: foot.z + offset.x,
            max_x: foot.x + depth - 1 + offset.z,
            max_y: foot.y + height - 1 + offset.y,
            max_z: foot.z + width - 1 + offset.x,
        },
    }
}

pub fn structure_piece_world_pos(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    x: i32,
    y: i32,
    z: i32,
) -> BlockPos {
    let Some(orientation) = orientation else {
        return BlockPos { x, y, z };
    };
    let world_x = match orientation {
        HorizontalDirection::North | HorizontalDirection::South => bounding_box.min_x + x,
        HorizontalDirection::West => bounding_box.max_x - z,
        HorizontalDirection::East => bounding_box.min_x + z,
    };
    let world_z = match orientation {
        HorizontalDirection::North => bounding_box.max_z - z,
        HorizontalDirection::South => bounding_box.min_z + z,
        HorizontalDirection::West | HorizontalDirection::East => bounding_box.min_z + x,
    };
    BlockPos {
        x: world_x,
        y: y + bounding_box.min_y,
        z: world_z,
    }
}

pub fn structure_piece_is_close_to_chunk(
    piece: StructurePieceModel,
    chunk_pos: ChunkPos,
    distance: i32,
) -> bool {
    let chunk_min_x = chunk_pos.x * 16;
    let chunk_min_z = chunk_pos.z * 16;
    piece.bounding_box.intersects(StructureBoundingBoxModel {
        min_x: chunk_min_x - distance,
        min_y: i32::MIN,
        min_z: chunk_min_z - distance,
        max_x: chunk_min_x + 15 + distance,
        max_y: i32::MAX,
        max_z: chunk_min_z + 15 + distance,
    })
}

pub fn structure_piece_locator_position(piece: StructurePieceModel) -> BlockPos {
    piece.bounding_box.center()
}

pub fn structure_piece_place_block(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    local_pos: BlockPos,
    state: &'static str,
    edge: bool,
) -> Option<StructurePiecePlacementBlock> {
    let world_pos = structure_piece_world_pos(
        bounding_box,
        orientation,
        local_pos.x,
        local_pos.y,
        local_pos.z,
    );
    chunk_bb
        .is_inside(world_pos)
        .then_some(StructurePiecePlacementBlock {
            local_pos,
            world_pos,
            state,
            edge,
        })
}

pub fn structure_piece_generate_air_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
) -> Vec<StructurePiecePlacementBlock> {
    structure_piece_generate_box(
        bounding_box,
        orientation,
        chunk_bb,
        min,
        max,
        "minecraft:air",
        "minecraft:air",
        false,
        |_| false,
    )
}

pub fn structure_piece_generate_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
    edge_block: &'static str,
    fill_block: &'static str,
    skip_air: bool,
    mut is_existing_air: impl FnMut(BlockPos) -> bool,
) -> Vec<StructurePiecePlacementBlock> {
    let mut blocks = Vec::new();
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                let local_pos = BlockPos { x, y, z };
                let world_pos = structure_piece_world_pos(bounding_box, orientation, x, y, z);
                if skip_air && is_existing_air(world_pos) {
                    continue;
                }
                let edge = y == min.y
                    || y == max.y
                    || x == min.x
                    || x == max.x
                    || z == min.z
                    || z == max.z;
                if let Some(block) = structure_piece_place_block(
                    bounding_box,
                    orientation,
                    chunk_bb,
                    local_pos,
                    if edge { edge_block } else { fill_block },
                    edge,
                ) {
                    blocks.push(block);
                }
            }
        }
    }
    blocks
}

pub fn structure_piece_generate_maybe_box(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    random_values: &[f32],
    probability: f32,
    min: BlockPos,
    max: BlockPos,
    edge_block: &'static str,
    fill_block: &'static str,
    skip_air: bool,
    has_to_be_inside: bool,
    mut is_existing_air: impl FnMut(BlockPos) -> bool,
    mut is_interior: impl FnMut(BlockPos) -> bool,
) -> Vec<StructurePiecePlacementBlock> {
    let mut blocks = Vec::new();
    let mut random_index = 0usize;
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                let random_value = random_values.get(random_index).copied().unwrap_or(1.0);
                random_index += 1;
                if random_value > probability {
                    continue;
                }
                let local_pos = BlockPos { x, y, z };
                let world_pos = structure_piece_world_pos(bounding_box, orientation, x, y, z);
                if skip_air && is_existing_air(world_pos) {
                    continue;
                }
                if has_to_be_inside && !is_interior(world_pos) {
                    continue;
                }
                let edge = y == min.y
                    || y == max.y
                    || x == min.x
                    || x == max.x
                    || z == min.z
                    || z == max.z;
                if let Some(block) = structure_piece_place_block(
                    bounding_box,
                    orientation,
                    chunk_bb,
                    local_pos,
                    if edge { edge_block } else { fill_block },
                    edge,
                ) {
                    blocks.push(block);
                }
            }
        }
    }
    blocks
}

pub fn structure_piece_maybe_generate_block(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    random_value: f32,
    probability: f32,
    local_pos: BlockPos,
    state: &'static str,
) -> Option<StructurePiecePlacementBlock> {
    (random_value < probability)
        .then(|| {
            structure_piece_place_block(bounding_box, orientation, chunk_bb, local_pos, state, true)
        })
        .flatten()
}

pub fn structure_piece_generate_upper_half_sphere(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
    fill_block: &'static str,
    skip_air: bool,
    mut is_existing_air: impl FnMut(BlockPos) -> bool,
) -> Vec<StructurePiecePlacementBlock> {
    let diag_x = (max.x - min.x + 1) as f32;
    let diag_y = (max.y - min.y + 1) as f32;
    let diag_z = (max.z - min.z + 1) as f32;
    let center_x = min.x as f32 + diag_x / 2.0;
    let center_z = min.z as f32 + diag_z / 2.0;
    let mut blocks = Vec::new();

    for y in min.y..=max.y {
        let normalized_y = (y - min.y) as f32 / diag_y;
        for x in min.x..=max.x {
            let normalized_x = (x as f32 - center_x) / (diag_x * 0.5);
            for z in min.z..=max.z {
                let normalized_z = (z as f32 - center_z) / (diag_z * 0.5);
                let local_pos = BlockPos { x, y, z };
                let world_pos = structure_piece_world_pos(bounding_box, orientation, x, y, z);
                if skip_air && is_existing_air(world_pos) {
                    continue;
                }
                let dist = normalized_x * normalized_x
                    + normalized_y * normalized_y
                    + normalized_z * normalized_z;
                if dist <= 1.05 {
                    if let Some(block) = structure_piece_place_block(
                        bounding_box,
                        orientation,
                        chunk_bb,
                        local_pos,
                        fill_block,
                        false,
                    ) {
                        blocks.push(block);
                    }
                }
            }
        }
    }
    blocks
}

pub fn structure_piece_is_replaceable_by_structures(state: &'static str) -> bool {
    matches!(
        state,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:glow_lichen"
            | "minecraft:seagrass"
            | "minecraft:tall_seagrass"
            | "minecraft:tall_seagrass[half=lower]"
            | "minecraft:tall_seagrass[half=upper]"
    )
}

pub fn structure_piece_fill_column_down(
    bounding_box: StructureBoundingBoxModel,
    orientation: Option<HorizontalDirection>,
    chunk_bb: StructureBoundingBoxModel,
    x: i32,
    start_y: i32,
    z: i32,
    min_y: i32,
    block_state: &'static str,
    mut block_at: impl FnMut(BlockPos) -> &'static str,
) -> Vec<StructurePiecePlacementBlock> {
    let mut world_pos = structure_piece_world_pos(bounding_box, orientation, x, start_y, z);
    if !chunk_bb.is_inside(world_pos) {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    while structure_piece_is_replaceable_by_structures(block_at(world_pos))
        && world_pos.y > min_y + 1
    {
        blocks.push(StructurePiecePlacementBlock {
            local_pos: BlockPos {
                x,
                y: world_pos.y - bounding_box.min_y,
                z,
            },
            world_pos,
            state: block_state,
            edge: false,
        });
        world_pos.y -= 1;
    }
    blocks
}

pub fn structure_piece_reorient_facing(
    current_facing: HorizontalDirection,
    neighbors: &[StructurePieceNeighborState],
) -> HorizontalDirection {
    let mut solid_neighbor = None;
    for direction in [
        HorizontalDirection::North,
        HorizontalDirection::South,
        HorizontalDirection::West,
        HorizontalDirection::East,
    ] {
        if let Some(neighbor) = neighbors
            .iter()
            .find(|neighbor| neighbor.direction == direction)
        {
            if neighbor.chest {
                return current_facing;
            }
            if neighbor.solid_render {
                if solid_neighbor.is_some() {
                    solid_neighbor = None;
                    break;
                }
                solid_neighbor = Some(direction);
            }
        }
    }

    if let Some(direction) = solid_neighbor {
        return direction.opposite();
    }

    let mut lock_dir = current_facing;
    if structure_piece_neighbor_is_solid(neighbors, lock_dir) {
        lock_dir = lock_dir.opposite();
    }
    if structure_piece_neighbor_is_solid(neighbors, lock_dir) {
        lock_dir = lock_dir.clockwise();
    }
    if structure_piece_neighbor_is_solid(neighbors, lock_dir) {
        lock_dir = lock_dir.opposite();
    }
    lock_dir
}

fn structure_piece_neighbor_is_solid(
    neighbors: &[StructurePieceNeighborState],
    direction: HorizontalDirection,
) -> bool {
    neighbors
        .iter()
        .find(|neighbor| neighbor.direction == direction)
        .is_some_and(|neighbor| neighbor.solid_render)
}

pub fn structure_piece_moved(
    piece: StructurePieceModel,
    dx: i32,
    dy: i32,
    dz: i32,
) -> StructurePieceModel {
    StructurePieceModel {
        bounding_box: piece.bounding_box.moved(dx, dy, dz),
    }
}

pub fn structure_piece_create_bounding_box(
    pieces: &[StructurePieceModel],
) -> Result<StructureBoundingBoxModel, String> {
    pieces
        .iter()
        .map(|piece| piece.bounding_box)
        .reduce(StructureBoundingBoxModel::union)
        .ok_or_else(|| "Unable to calculate boundingbox without pieces".to_string())
}

pub fn structure_piece_find_collision_piece(
    pieces: &[StructurePieceModel],
    bounding_box: StructureBoundingBoxModel,
) -> Option<StructurePieceModel> {
    pieces
        .iter()
        .copied()
        .find(|piece| piece.bounding_box.intersects(bounding_box))
}

pub fn structure_piece_orientation_state(
    orientation: Option<HorizontalDirection>,
) -> StructurePieceOrientationState {
    let (mirror, rotation) = match orientation {
        None | Some(HorizontalDirection::North) => {
            (StructurePieceMirror::None, StructurePieceRotation::None)
        }
        Some(HorizontalDirection::South) => (
            StructurePieceMirror::LeftRight,
            StructurePieceRotation::None,
        ),
        Some(HorizontalDirection::West) => (
            StructurePieceMirror::LeftRight,
            StructurePieceRotation::Clockwise90,
        ),
        Some(HorizontalDirection::East) => (
            StructurePieceMirror::None,
            StructurePieceRotation::Clockwise90,
        ),
    };
    StructurePieceOrientationState {
        orientation,
        mirror,
        rotation,
    }
}

