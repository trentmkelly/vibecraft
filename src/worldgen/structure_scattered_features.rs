use super::*;

pub fn buried_treasure_generation_piece(chunk_pos: ChunkPos) -> BuriedTreasurePieceModel {
    let pos = BlockPos {
        x: chunk_pos.x * 16 + 9,
        y: 90,
        z: chunk_pos.z * 16 + 9,
    };
    BuriedTreasurePieceModel {
        bounding_box: StructureBoundingBoxModel {
            min_x: pos.x,
            min_y: pos.y,
            min_z: pos.z,
            max_x: pos.x,
            max_y: pos.y,
            max_z: pos.z,
        },
    }
}

pub fn buried_treasure_is_liquid(state: &'static str) -> bool {
    matches!(state, "minecraft:water" | "minecraft:lava")
}

pub fn buried_treasure_is_support(state: &'static str) -> bool {
    matches!(
        state,
        "minecraft:sandstone"
            | "minecraft:stone"
            | "minecraft:andesite"
            | "minecraft:granite"
            | "minecraft:diorite"
    )
}

pub fn buried_treasure_soft_state(current_state: &'static str) -> &'static str {
    if current_state == "minecraft:air" || buried_treasure_is_liquid(current_state) {
        "minecraft:sand"
    } else {
        current_state
    }
}

pub fn buried_treasure_place(
    piece: BuriedTreasurePieceModel,
    ocean_floor_y: i32,
    min_y: i32,
    mut block_state_at: impl FnMut(BlockPos) -> &'static str,
) -> Option<BuriedTreasurePlacementModel> {
    let x = piece.bounding_box.min_x;
    let z = piece.bounding_box.min_z;
    for y in (min_y + 1..=ocean_floor_y).rev() {
        let pos = BlockPos { x, y, z };
        let below_pos = BlockPos { x, y: y - 1, z };
        let below_state = block_state_at(below_pos);
        if !buried_treasure_is_support(below_state) {
            continue;
        }

        let soft_state = buried_treasure_soft_state(block_state_at(pos));
        let directions = [
            BlockPos { x: 0, y: -1, z: 0 },
            BlockPos { x: 0, y: 1, z: 0 },
            BlockPos { x: 0, y: 0, z: -1 },
            BlockPos { x: 0, y: 0, z: 1 },
            BlockPos { x: -1, y: 0, z: 0 },
            BlockPos { x: 1, y: 0, z: 0 },
        ];
        let side_fill = directions.map(|delta| {
            let relative_pos = BlockPos {
                x: pos.x + delta.x,
                y: pos.y + delta.y,
                z: pos.z + delta.z,
            };
            let relative_state = block_state_at(relative_pos);
            let fill_state =
                if relative_state == "minecraft:air" || buried_treasure_is_liquid(relative_state) {
                    let below_relative_pos = BlockPos {
                        x: relative_pos.x,
                        y: relative_pos.y - 1,
                        z: relative_pos.z,
                    };
                    let below_relative_state = block_state_at(below_relative_pos);
                    if delta.y != 1
                        && (below_relative_state == "minecraft:air"
                            || buried_treasure_is_liquid(below_relative_state))
                    {
                        below_state
                    } else {
                        soft_state
                    }
                } else {
                    relative_state
                };
            (fill_state, relative_pos)
        });

        return Some(BuriedTreasurePlacementModel {
            chest_pos: pos,
            bounding_box: StructureBoundingBoxModel {
                min_x: pos.x,
                min_y: pos.y,
                min_z: pos.z,
                max_x: pos.x,
                max_y: pos.y,
                max_z: pos.z,
            },
            side_fill,
        });
    }
    None
}

pub fn scattered_feature_piece(
    west: i32,
    floor: i32,
    north: i32,
    width: i32,
    height: i32,
    depth: i32,
    orientation: HorizontalDirection,
) -> ScatteredFeaturePieceModel {
    ScatteredFeaturePieceModel {
        bounding_box: structure_make_bounding_box(
            west,
            floor,
            north,
            orientation,
            width,
            height,
            depth,
        ),
        orientation,
        width,
        height,
        depth,
        height_position: -1,
    }
}

pub fn scattered_feature_update_average_ground_height(
    piece: &mut ScatteredFeaturePieceModel,
    chunk_bb: StructureBoundingBoxModel,
    offset: i32,
    mut height_at: impl FnMut(i32, i32) -> i32,
) -> bool {
    if piece.height_position >= 0 {
        return true;
    }
    let mut total = 0;
    let mut count = 0;
    for z in piece.bounding_box.min_z..=piece.bounding_box.max_z {
        for x in piece.bounding_box.min_x..=piece.bounding_box.max_x {
            if chunk_bb.is_inside(BlockPos { x, y: 64, z }) {
                total += height_at(x, z);
                count += 1;
            }
        }
    }
    if count == 0 {
        return false;
    }
    piece.height_position = total / count;
    let dy = piece.height_position - piece.bounding_box.min_y + offset;
    piece.bounding_box = piece.bounding_box.moved(0, dy, 0);
    true
}

pub fn swamp_hut_generation_piece(
    chunk_pos: ChunkPos,
    orientation: HorizontalDirection,
) -> SwampHutPieceModel {
    SwampHutPieceModel {
        scattered: scattered_feature_piece(
            chunk_pos.x * 16,
            64,
            chunk_pos.z * 16,
            7,
            7,
            9,
            orientation,
        ),
        spawned_witch: false,
        spawned_cat: false,
    }
}

pub fn swamp_hut_save_tag(piece: &SwampHutPieceModel) -> SwampHutSaveTagModel {
    SwampHutSaveTagModel {
        width: piece.scattered.width,
        height: piece.scattered.height,
        depth: piece.scattered.depth,
        height_position: piece.scattered.height_position,
        witch: piece.spawned_witch,
        cat: piece.spawned_cat,
    }
}

fn swamp_hut_place_block(
    piece: &SwampHutPieceModel,
    chunk_bb: StructureBoundingBoxModel,
    local_pos: BlockPos,
    state: &'static str,
    blocks: &mut Vec<StructurePiecePlacementBlock>,
) {
    if let Some(block) = structure_piece_place_block(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        chunk_bb,
        local_pos,
        state,
        true,
    ) {
        blocks.push(block);
    }
}

fn swamp_hut_generate_box(
    piece: &SwampHutPieceModel,
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
    state: &'static str,
    blocks: &mut Vec<StructurePiecePlacementBlock>,
) {
    blocks.extend(structure_piece_generate_box(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        chunk_bb,
        min,
        max,
        state,
        state,
        false,
        |_| false,
    ));
}

pub fn swamp_hut_post_process(
    mut piece: SwampHutPieceModel,
    chunk_bb: StructureBoundingBoxModel,
    height_at: impl FnMut(i32, i32) -> i32,
) -> Option<SwampHutPostProcessModel> {
    if !scattered_feature_update_average_ground_height(&mut piece.scattered, chunk_bb, 0, height_at)
    {
        return None;
    }

    let mut blocks = Vec::new();
    swamp_hut_generate_box(
        &piece,
        chunk_bb,
        BlockPos { x: 1, y: 1, z: 1 },
        BlockPos { x: 5, y: 1, z: 7 },
        "minecraft:spruce_planks",
        &mut blocks,
    );
    swamp_hut_generate_box(
        &piece,
        chunk_bb,
        BlockPos { x: 1, y: 4, z: 2 },
        BlockPos { x: 5, y: 4, z: 7 },
        "minecraft:spruce_planks",
        &mut blocks,
    );
    for (min, max, state) in [
        (
            BlockPos { x: 2, y: 1, z: 0 },
            BlockPos { x: 4, y: 1, z: 0 },
            "minecraft:spruce_planks",
        ),
        (
            BlockPos { x: 2, y: 2, z: 2 },
            BlockPos { x: 3, y: 3, z: 2 },
            "minecraft:spruce_planks",
        ),
        (
            BlockPos { x: 1, y: 2, z: 3 },
            BlockPos { x: 1, y: 3, z: 6 },
            "minecraft:spruce_planks",
        ),
        (
            BlockPos { x: 5, y: 2, z: 3 },
            BlockPos { x: 5, y: 3, z: 6 },
            "minecraft:spruce_planks",
        ),
        (
            BlockPos { x: 2, y: 2, z: 7 },
            BlockPos { x: 4, y: 3, z: 7 },
            "minecraft:spruce_planks",
        ),
        (
            BlockPos { x: 1, y: 0, z: 2 },
            BlockPos { x: 1, y: 3, z: 2 },
            "minecraft:oak_log",
        ),
        (
            BlockPos { x: 5, y: 0, z: 2 },
            BlockPos { x: 5, y: 3, z: 2 },
            "minecraft:oak_log",
        ),
        (
            BlockPos { x: 1, y: 0, z: 7 },
            BlockPos { x: 1, y: 3, z: 7 },
            "minecraft:oak_log",
        ),
        (
            BlockPos { x: 5, y: 0, z: 7 },
            BlockPos { x: 5, y: 3, z: 7 },
            "minecraft:oak_log",
        ),
        (
            BlockPos { x: 0, y: 4, z: 1 },
            BlockPos { x: 6, y: 4, z: 1 },
            "minecraft:spruce_stairs[facing=north]",
        ),
        (
            BlockPos { x: 0, y: 4, z: 2 },
            BlockPos { x: 0, y: 4, z: 7 },
            "minecraft:spruce_stairs[facing=east]",
        ),
        (
            BlockPos { x: 6, y: 4, z: 2 },
            BlockPos { x: 6, y: 4, z: 7 },
            "minecraft:spruce_stairs[facing=west]",
        ),
        (
            BlockPos { x: 0, y: 4, z: 8 },
            BlockPos { x: 6, y: 4, z: 8 },
            "minecraft:spruce_stairs[facing=south]",
        ),
    ] {
        swamp_hut_generate_box(&piece, chunk_bb, min, max, state, &mut blocks);
    }

    for (local_pos, state) in [
        (BlockPos { x: 2, y: 3, z: 2 }, "minecraft:oak_fence"),
        (BlockPos { x: 3, y: 3, z: 7 }, "minecraft:oak_fence"),
        (BlockPos { x: 1, y: 3, z: 4 }, "minecraft:air"),
        (BlockPos { x: 5, y: 3, z: 4 }, "minecraft:air"),
        (BlockPos { x: 5, y: 3, z: 5 }, "minecraft:air"),
        (
            BlockPos { x: 1, y: 3, z: 5 },
            "minecraft:potted_red_mushroom",
        ),
        (BlockPos { x: 3, y: 2, z: 6 }, "minecraft:crafting_table"),
        (BlockPos { x: 4, y: 2, z: 6 }, "minecraft:cauldron"),
        (BlockPos { x: 1, y: 2, z: 1 }, "minecraft:oak_fence"),
        (BlockPos { x: 5, y: 2, z: 1 }, "minecraft:oak_fence"),
        (
            BlockPos { x: 0, y: 4, z: 1 },
            "minecraft:spruce_stairs[facing=north,shape=outer_right]",
        ),
        (
            BlockPos { x: 6, y: 4, z: 1 },
            "minecraft:spruce_stairs[facing=north,shape=outer_left]",
        ),
        (
            BlockPos { x: 0, y: 4, z: 8 },
            "minecraft:spruce_stairs[facing=south,shape=outer_left]",
        ),
        (
            BlockPos { x: 6, y: 4, z: 8 },
            "minecraft:spruce_stairs[facing=south,shape=outer_right]",
        ),
    ] {
        swamp_hut_place_block(&piece, chunk_bb, local_pos, state, &mut blocks);
    }

    let mut fill_columns = Vec::new();
    for z in [2, 7] {
        for x in [1, 5] {
            fill_columns.push(structure_piece_world_pos(
                piece.scattered.bounding_box,
                Some(piece.scattered.orientation),
                x,
                -1,
                z,
            ));
        }
    }

    let spawn_pos = structure_piece_world_pos(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        2,
        2,
        5,
    );
    let mut entity_spawns = Vec::new();
    if chunk_bb.is_inside(spawn_pos) {
        if !piece.spawned_witch {
            piece.spawned_witch = true;
            entity_spawns.push(SwampHutEntitySpawnModel {
                entity: "minecraft:witch",
                pos: spawn_pos,
            });
        }
        if !piece.spawned_cat {
            piece.spawned_cat = true;
            entity_spawns.push(SwampHutEntitySpawnModel {
                entity: "minecraft:cat",
                pos: spawn_pos,
            });
        }
    }

    Some(SwampHutPostProcessModel {
        piece,
        blocks,
        fill_columns,
        entity_spawns,
    })
}

pub fn desert_pyramid_generation_piece(
    chunk_pos: ChunkPos,
    orientation: HorizontalDirection,
) -> DesertPyramidPieceModel {
    DesertPyramidPieceModel {
        scattered: scattered_feature_piece(
            chunk_pos.x * 16,
            64,
            chunk_pos.z * 16,
            21,
            15,
            21,
            orientation,
        ),
        has_placed_chest: [false; 4],
        potential_suspicious_sand_world_positions: Vec::new(),
        random_collapsed_roof_pos: BlockPos { x: 0, y: 0, z: 0 },
    }
}

pub fn desert_pyramid_save_tag(piece: &DesertPyramidPieceModel) -> DesertPyramidSaveTagModel {
    DesertPyramidSaveTagModel {
        width: piece.scattered.width,
        height: piece.scattered.height,
        depth: piece.scattered.depth,
        height_position: piece.scattered.height_position,
        has_placed_chest: piece.has_placed_chest,
    }
}

pub fn desert_pyramid_place_sand(
    piece: &mut DesertPyramidPieceModel,
    local_x: i32,
    local_y: i32,
    local_z: i32,
) -> BlockPos {
    let pos = structure_piece_world_pos(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        local_x,
        local_y,
        local_z,
    );
    piece.potential_suspicious_sand_world_positions.push(pos);
    pos
}

pub fn desert_pyramid_place_sand_box(
    piece: &mut DesertPyramidPieceModel,
    min: BlockPos,
    max: BlockPos,
) {
    for y in min.y..=max.y {
        for x in min.x..=max.x {
            for z in min.z..=max.z {
                desert_pyramid_place_sand(piece, x, y, z);
            }
        }
    }
}

pub fn desert_pyramid_record_collapsed_roof(
    piece: &mut DesertPyramidPieceModel,
    local_min: BlockPos,
    local_max_x: i32,
    local_max_z: i32,
    chosen_local_x: i32,
    chosen_local_z: i32,
) -> Result<BlockPos, String> {
    if !(local_min.x..=local_max_x).contains(&chosen_local_x)
        || !(local_min.z..=local_max_z).contains(&chosen_local_z)
    {
        return Err(
            "Collapsed roof random position must be inside the requested local range".to_string(),
        );
    }
    let pos = structure_piece_world_pos(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        chosen_local_x,
        local_min.y,
        chosen_local_z,
    );
    piece.random_collapsed_roof_pos = pos;
    Ok(pos)
}

fn block_pos_java_cmp_key(pos: BlockPos) -> (i32, i32, i32) {
    (pos.y, pos.z, pos.x)
}

pub fn desert_pyramid_unique_suspicious_sand_positions(
    pieces: &[DesertPyramidPieceModel],
) -> Vec<BlockPos> {
    let mut positions = pieces
        .iter()
        .flat_map(|piece| {
            piece
                .potential_suspicious_sand_world_positions
                .iter()
                .copied()
        })
        .collect::<Vec<_>>();
    positions.sort_by_key(|pos| block_pos_java_cmp_key(*pos));
    positions.dedup();
    positions
}

pub fn desert_pyramid_after_place_archaeology(
    pieces: &[DesertPyramidPieceModel],
    chunk_bb: StructureBoundingBoxModel,
    shuffled_unique_positions: &[BlockPos],
    suspicious_sand_to_place: usize,
) -> Vec<DesertPyramidArchaeologyPlacement> {
    let unique_positions = desert_pyramid_unique_suspicious_sand_positions(pieces);
    let mut ordered_positions = shuffled_unique_positions
        .iter()
        .copied()
        .filter(|pos| unique_positions.contains(pos))
        .collect::<Vec<_>>();
    for pos in unique_positions {
        if !ordered_positions.contains(&pos) {
            ordered_positions.push(pos);
        }
    }

    let mut placements = Vec::new();
    for piece in pieces {
        if chunk_bb.is_inside(piece.random_collapsed_roof_pos) {
            placements.push(DesertPyramidArchaeologyPlacement {
                pos: piece.random_collapsed_roof_pos,
                state: "minecraft:suspicious_sand",
                loot_table: Some("minecraft:archaeology/desert_pyramid"),
                loot_seed: Some(block_pos_as_long(piece.random_collapsed_roof_pos)),
            });
        }
    }

    for (index, pos) in ordered_positions.into_iter().enumerate() {
        if !chunk_bb.is_inside(pos) {
            continue;
        }
        if index < suspicious_sand_to_place {
            placements.push(DesertPyramidArchaeologyPlacement {
                pos,
                state: "minecraft:suspicious_sand",
                loot_table: Some("minecraft:archaeology/desert_pyramid"),
                loot_seed: Some(block_pos_as_long(pos)),
            });
        } else {
            placements.push(DesertPyramidArchaeologyPlacement {
                pos,
                state: "minecraft:sand",
                loot_table: None,
                loot_seed: None,
            });
        }
    }
    placements
}

pub fn block_pos_as_long(pos: BlockPos) -> i64 {
    (((pos.x as i64) & 0x3ffffff) << 38)
        | (((pos.z as i64) & 0x3ffffff) << 12)
        | ((pos.y as i64) & 0xfff)
}

pub fn jungle_temple_generation_piece(
    chunk_pos: ChunkPos,
    orientation: HorizontalDirection,
) -> JungleTemplePieceModel {
    JungleTemplePieceModel {
        scattered: scattered_feature_piece(
            chunk_pos.x * 16,
            64,
            chunk_pos.z * 16,
            12,
            10,
            15,
            orientation,
        ),
        placed_main_chest: false,
        placed_hidden_chest: false,
        placed_trap1: false,
        placed_trap2: false,
    }
}

pub fn jungle_temple_save_tag(piece: &JungleTemplePieceModel) -> JungleTempleSaveTagModel {
    JungleTempleSaveTagModel {
        width: piece.scattered.width,
        height: piece.scattered.height,
        depth: piece.scattered.depth,
        height_position: piece.scattered.height_position,
        placed_main_chest: piece.placed_main_chest,
        placed_hidden_chest: piece.placed_hidden_chest,
        placed_trap1: piece.placed_trap1,
        placed_trap2: piece.placed_trap2,
    }
}

fn jungle_temple_try_container(
    piece: &JungleTemplePieceModel,
    chunk_bb: StructureBoundingBoxModel,
    local_pos: BlockPos,
    kind: &'static str,
    facing: Option<HorizontalDirection>,
    loot_table: &'static str,
    already_placed: bool,
) -> Option<JungleTempleContainerPlacement> {
    if already_placed {
        return None;
    }
    let pos = structure_piece_world_pos(
        piece.scattered.bounding_box,
        Some(piece.scattered.orientation),
        local_pos.x,
        local_pos.y,
        local_pos.z,
    );
    chunk_bb
        .is_inside(pos)
        .then_some(JungleTempleContainerPlacement {
            kind,
            pos,
            facing,
            loot_table,
        })
}

pub fn jungle_temple_post_process(
    mut piece: JungleTemplePieceModel,
    chunk_bb: StructureBoundingBoxModel,
    height_at: impl FnMut(i32, i32) -> i32,
) -> Option<JungleTemplePostProcessModel> {
    if !scattered_feature_update_average_ground_height(&mut piece.scattered, chunk_bb, 0, height_at)
    {
        return None;
    }

    let mut containers = Vec::new();
    if let Some(container) = jungle_temple_try_container(
        &piece,
        chunk_bb,
        BlockPos { x: 3, y: -2, z: 1 },
        "minecraft:dispenser",
        Some(HorizontalDirection::North),
        "minecraft:chests/jungle_temple_dispenser",
        piece.placed_trap1,
    ) {
        piece.placed_trap1 = true;
        containers.push(container);
    }
    if let Some(container) = jungle_temple_try_container(
        &piece,
        chunk_bb,
        BlockPos { x: 9, y: -2, z: 3 },
        "minecraft:dispenser",
        Some(HorizontalDirection::West),
        "minecraft:chests/jungle_temple_dispenser",
        piece.placed_trap2,
    ) {
        piece.placed_trap2 = true;
        containers.push(container);
    }
    if let Some(container) = jungle_temple_try_container(
        &piece,
        chunk_bb,
        BlockPos { x: 8, y: -3, z: 3 },
        "minecraft:chest",
        None,
        "minecraft:chests/jungle_temple",
        piece.placed_main_chest,
    ) {
        piece.placed_main_chest = true;
        containers.push(container);
    }
    if let Some(container) = jungle_temple_try_container(
        &piece,
        chunk_bb,
        BlockPos { x: 9, y: -3, z: 10 },
        "minecraft:chest",
        None,
        "minecraft:chests/jungle_temple",
        piece.placed_hidden_chest,
    ) {
        piece.placed_hidden_chest = true;
        containers.push(container);
    }

    Some(JungleTemplePostProcessModel { piece, containers })
}

