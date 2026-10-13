use super::*;

/// One cell in the monument's room graph, including its open directional edges.
///
/// `connections` uses the vanilla 3D direction order: down, up, north, south,
/// west, east. Empty cells are omitted from the graph entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OceanMonumentRoomGraphRoomModel {
    pub index: i32,
    pub connections: [Option<usize>; 6],
    pub openings: [bool; 6],
    pub claimed: bool,
    pub is_source: bool,
}

/// The sparse room graph generated before monument room fitters consume it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OceanMonumentRoomGraphModel {
    pub rooms: Vec<OceanMonumentRoomGraphRoomModel>,
    /// Room slots in the post-shuffle order returned by Java's room graph builder.
    pub room_order: Vec<usize>,
    pub source_room: usize,
    pub core_room: usize,
}

/// Room storage and sparse-grid lookup before special rooms are attached.
struct MonumentRoomGrid {
    rooms: Vec<OceanMonumentRoomGraphRoomModel>,
    grid: [Option<usize>; 75],
    source_room: usize,
}

const MONUMENT_DIRECTIONS: [OceanMonumentDirectionModel; 6] = [
    OceanMonumentDirectionModel::Down,
    OceanMonumentDirectionModel::Up,
    OceanMonumentDirectionModel::North,
    OceanMonumentDirectionModel::South,
    OceanMonumentDirectionModel::West,
    OceanMonumentDirectionModel::East,
];

/// Mirrors `MonumentBuilding.generateRoomGraph` in OceanMonumentPieces.
///
/// In particular, regular rooms are collected in backing-grid order, special
/// rooms enter fitter order after opening pruning, and edge removal consumes
/// the same shuffle and up-to-five `nextInt(6)` attempts per room as Java.
pub fn ocean_monument_room_graph(
    random: &mut RandomSourceKind,
) -> Result<OceanMonumentRoomGraphModel, String> {
    let MonumentRoomGrid {
        mut rooms,
        grid,
        source_room,
    } = create_monument_room_grid()?;
    let regular_room_count = rooms.len();
    let (roof, left_wing, right_wing) = add_monument_special_rooms(&mut rooms, &grid)?;
    let core_room = claim_monument_core_room(&mut rooms, &grid, random)?;
    initialize_monument_room_openings(&mut rooms, regular_room_count, roof);
    let mut room_order = prune_monument_room_openings(&mut rooms, &grid, random)?;
    room_order.extend([roof, left_wing, right_wing]);

    Ok(OceanMonumentRoomGraphModel {
        rooms,
        room_order,
        source_room,
        core_room,
    })
}

fn create_monument_room_grid() -> Result<MonumentRoomGrid, String> {
    let mut grid = [None; 75];

    // Java fills the sparse array in x/z/y batches, then collects non-null
    // entries by ascending array index. Store the rooms in that collected order.
    for x in 0..5 {
        for z in 0..4 {
            let index = ocean_monument_room_index(x, 0, z) as usize;
            grid[index] = Some(0);
        }
    }
    for x in 0..5 {
        for z in 0..4 {
            let index = ocean_monument_room_index(x, 1, z) as usize;
            grid[index] = Some(0);
        }
    }
    for x in 1..4 {
        for z in 0..2 {
            let index = ocean_monument_room_index(x, 2, z) as usize;
            grid[index] = Some(0);
        }
    }
    let mut rooms = Vec::with_capacity(49);
    for (index, slot) in grid.iter_mut().enumerate() {
        if slot.is_some() {
            *slot = Some(rooms.len());
            rooms.push(OceanMonumentRoomGraphRoomModel {
                index: index as i32,
                connections: [None; 6],
                openings: [false; 6],
                claimed: false,
                is_source: false,
            });
        }
    }
    let source_room = monument_grid_room(&grid, ocean_monument_room_index(2, 0, 0))
        .ok_or_else(|| "Ocean monument source room is absent from the vanilla grid".to_string())?;
    rooms[source_room].is_source = true;

    // Java iterates x, z, y and `Direction.values()`. For north/south edges it
    // reverses the requested direction based on the neighboring z coordinate;
    // preserve that detail rather than replacing it with a generic grid walk.
    for x in 0..5 {
        for z in 0..5 {
            for y in 0..3 {
                let Some(room) = grid[ocean_monument_room_index(x, y, z) as usize] else {
                    continue;
                };
                for direction in MONUMENT_DIRECTIONS {
                    let (dx, dy, dz) = monument_direction_step(direction);
                    let nx = x + dx;
                    let ny = y + dy;
                    let nz = z + dz;
                    if !(0..5).contains(&nx) || !(0..3).contains(&ny) || !(0..5).contains(&nz) {
                        continue;
                    }
                    let Some(neighbor) = grid[ocean_monument_room_index(nx, ny, nz) as usize]
                    else {
                        continue;
                    };
                    let connection_direction = if nz == z {
                        direction
                    } else {
                        ocean_monument_opposite_direction(direction)
                    };
                    connect_monument_rooms(&mut rooms, room, connection_direction, neighbor);
                }
            }
        }
    }

    Ok(MonumentRoomGrid {
        rooms,
        grid,
        source_room,
    })
}

fn add_monument_special_rooms(
    rooms: &mut Vec<OceanMonumentRoomGraphRoomModel>,
    grid: &[Option<usize>; 75],
) -> Result<(usize, usize, usize), String> {
    let roof = add_monument_special_room(rooms, 1003);
    let left_wing = add_monument_special_room(rooms, 1001);
    let right_wing = add_monument_special_room(rooms, 1002);
    connect_monument_rooms(
        rooms,
        monument_grid_room(grid, ocean_monument_room_index(2, 2, 0)).ok_or_else(|| {
            "Ocean monument roof connector is absent from the vanilla grid".to_string()
        })?,
        OceanMonumentDirectionModel::Up,
        roof,
    );
    connect_monument_rooms(
        rooms,
        monument_grid_room(grid, ocean_monument_room_index(0, 1, 0)).ok_or_else(|| {
            "Ocean monument left-wing connector is absent from the vanilla grid".to_string()
        })?,
        OceanMonumentDirectionModel::South,
        left_wing,
    );
    connect_monument_rooms(
        rooms,
        monument_grid_room(grid, ocean_monument_room_index(4, 1, 0)).ok_or_else(|| {
            "Ocean monument right-wing connector is absent from the vanilla grid".to_string()
        })?,
        OceanMonumentDirectionModel::South,
        right_wing,
    );
    rooms[roof].claimed = true;
    rooms[left_wing].claimed = true;
    rooms[right_wing].claimed = true;
    Ok((roof, left_wing, right_wing))
}

fn claim_monument_core_room(
    rooms: &mut [OceanMonumentRoomGraphRoomModel],
    grid: &[Option<usize>; 75],
    random: &mut RandomSourceKind,
) -> Result<usize, String> {
    let core_room_index =
        ocean_monument_room_index(worldgen_random_next_i32_bound(random, 4), 0, 2);
    let core_room = monument_grid_room(grid, core_room_index)
        .ok_or_else(|| "Ocean monument core room is absent from the vanilla grid".to_string())?;
    rooms[core_room].claimed = true;
    let east = monument_connected_room(rooms, core_room, 5)
        .ok_or_else(|| "Ocean monument core east room is missing".to_string())?;
    let north = monument_connected_room(rooms, core_room, 2)
        .ok_or_else(|| "Ocean monument core north room is missing".to_string())?;
    rooms[east].claimed = true;
    rooms[north].claimed = true;
    let east_north = monument_connected_room(rooms, east, 2)
        .ok_or_else(|| "Ocean monument core east-north room is missing".to_string())?;
    rooms[east_north].claimed = true;
    let up = monument_connected_room(rooms, core_room, 1)
        .ok_or_else(|| "Ocean monument core upper room is missing".to_string())?;
    let east_up = monument_connected_room(rooms, east, 1)
        .ok_or_else(|| "Ocean monument core east-upper room is missing".to_string())?;
    let north_up = monument_connected_room(rooms, north, 1)
        .ok_or_else(|| "Ocean monument core north-upper room is missing".to_string())?;
    rooms[up].claimed = true;
    rooms[east_up].claimed = true;
    rooms[north_up].claimed = true;
    let east_north_up = monument_connected_room(rooms, east_north, 1)
        .ok_or_else(|| "Ocean monument core east-north-upper room is missing".to_string())?;
    rooms[east_north_up].claimed = true;
    Ok(core_room)
}

fn initialize_monument_room_openings(
    rooms: &mut [OceanMonumentRoomGraphRoomModel],
    regular_room_count: usize,
    roof: usize,
) {
    for room in rooms.iter_mut().take(regular_room_count) {
        room.openings = room.connections.map(|connection| connection.is_some());
    }
    rooms[roof].openings = rooms[roof]
        .connections
        .map(|connection| connection.is_some());
}

fn prune_monument_room_openings(
    rooms: &mut [OceanMonumentRoomGraphRoomModel],
    grid: &[Option<usize>; 75],
    random: &mut RandomSourceKind,
) -> Result<Vec<usize>, String> {
    // Util.shuffle is a descending Fisher–Yates shuffle, sampling [0, i).
    let mut shuffled: Vec<usize> = grid.iter().flatten().copied().collect();
    for i in (2..=shuffled.len()).rev() {
        let swap_to = worldgen_random_next_i32_bound(random, i as i32) as usize;
        shuffled.swap(i - 1, swap_to);
    }

    let mut scan_index = 1_i32;
    let mut scan_markers = vec![0_i32; rooms.len()];
    for &room in &shuffled {
        let mut close_count = 0;
        let mut attempt_count = 0;
        while close_count < 2 && attempt_count < 5 {
            attempt_count += 1;
            let direction = worldgen_random_next_i32_bound(random, 6) as usize;
            if !rooms[room].openings[direction] {
                continue;
            }
            let neighbor = monument_connected_room(rooms, room, direction)
                .ok_or_else(|| "Open ocean monument room edge has no connection".to_string())?;
            let opposite = direction ^ 1;
            rooms[room].openings[direction] = false;
            rooms[neighbor].openings[opposite] = false;
            let first_scan = scan_index;
            scan_index = scan_index.wrapping_add(1);
            if monument_room_reaches_source(rooms, room, first_scan, &mut scan_markers) {
                let second_scan = scan_index;
                scan_index = scan_index.wrapping_add(1);
                if !monument_room_reaches_source(rooms, neighbor, second_scan, &mut scan_markers) {
                    rooms[room].openings[direction] = true;
                    rooms[neighbor].openings[opposite] = true;
                    continue;
                }
                close_count += 1;
            } else {
                rooms[room].openings[direction] = true;
                rooms[neighbor].openings[opposite] = true;
            }
        }
    }
    Ok(shuffled)
}

fn monument_direction_step(direction: OceanMonumentDirectionModel) -> (i32, i32, i32) {
    match direction {
        OceanMonumentDirectionModel::Down => (0, -1, 0),
        OceanMonumentDirectionModel::Up => (0, 1, 0),
        OceanMonumentDirectionModel::North => (0, 0, -1),
        OceanMonumentDirectionModel::South => (0, 0, 1),
        OceanMonumentDirectionModel::West => (-1, 0, 0),
        OceanMonumentDirectionModel::East => (1, 0, 0),
    }
}

fn monument_grid_room(grid: &[Option<usize>; 75], index: i32) -> Option<usize> {
    grid.get(index as usize).copied().flatten()
}

fn monument_connected_room(
    rooms: &[OceanMonumentRoomGraphRoomModel],
    room: usize,
    direction: usize,
) -> Option<usize> {
    rooms
        .get(room)
        .and_then(|definition| definition.connections[direction])
}

fn connect_monument_rooms(
    rooms: &mut [OceanMonumentRoomGraphRoomModel],
    room: usize,
    direction: OceanMonumentDirectionModel,
    neighbor: usize,
) {
    let direction = ocean_monument_direction_3d_value(direction) as usize;
    rooms[room].connections[direction] = Some(neighbor);
    rooms[neighbor].connections[direction ^ 1] = Some(room);
}

fn add_monument_special_room(
    rooms: &mut Vec<OceanMonumentRoomGraphRoomModel>,
    index: i32,
) -> usize {
    let slot = rooms.len();
    rooms.push(OceanMonumentRoomGraphRoomModel {
        index,
        connections: [None; 6],
        openings: [false; 6],
        claimed: false,
        is_source: false,
    });
    slot
}

fn monument_room_reaches_source(
    rooms: &[OceanMonumentRoomGraphRoomModel],
    room: usize,
    scan_index: i32,
    scan_markers: &mut [i32],
) -> bool {
    if rooms[room].is_source {
        return true;
    }
    scan_markers[room] = scan_index;
    for direction in 0..6 {
        if rooms[room].openings[direction] {
            if let Some(neighbor) = rooms[room].connections[direction] {
                if scan_markers[neighbor] != scan_index
                    && monument_room_reaches_source(rooms, neighbor, scan_index, scan_markers)
                {
                    return true;
                }
            }
        }
    }
    false
}

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

#[cfg(test)]
#[path = "structure_ocean_monuments_tests.rs"]
mod tests;
