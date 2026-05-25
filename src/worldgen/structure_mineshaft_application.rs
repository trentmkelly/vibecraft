use super::*;

fn mineshaft_chunk_bounding_box(chunk_pos: ChunkPos) -> StructureBoundingBoxModel {
    StructureBoundingBoxModel {
        min_x: chunk_pos.x * 16,
        min_y: -64,
        min_z: chunk_pos.z * 16,
        max_x: chunk_pos.x * 16 + 15,
        max_y: 319,
        max_z: chunk_pos.z * 16 + 15,
    }
}

fn mineshaft_block_is_liquid(block: &str) -> bool {
    block.starts_with("minecraft:water") || block.starts_with("minecraft:lava")
}

fn mineshaft_piece_is_in_invalid_location(
    chunk: &LevelChunk,
    piece_box: StructureBoundingBoxModel,
    chunk_bb: StructureBoundingBoxModel,
) -> bool {
    let x0 = (piece_box.min_x - 1).max(chunk_bb.min_x);
    let y0 = (piece_box.min_y - 1).max(chunk_bb.min_y);
    let z0 = (piece_box.min_z - 1).max(chunk_bb.min_z);
    let x1 = (piece_box.max_x + 1).min(chunk_bb.max_x);
    let y1 = (piece_box.max_y + 1).min(chunk_bb.max_y);
    let z1 = (piece_box.max_z + 1).min(chunk_bb.max_z);
    for x in x0..=x1 {
        for z in z0..=z1 {
            if chunk
                .get_block_state(x, y0, z)
                .is_some_and(|block| mineshaft_block_is_liquid(&block))
                || chunk
                    .get_block_state(x, y1, z)
                    .is_some_and(|block| mineshaft_block_is_liquid(&block))
            {
                return true;
            }
        }
    }
    for x in x0..=x1 {
        for y in y0..=y1 {
            if chunk
                .get_block_state(x, y, z0)
                .is_some_and(|block| mineshaft_block_is_liquid(&block))
                || chunk
                    .get_block_state(x, y, z1)
                    .is_some_and(|block| mineshaft_block_is_liquid(&block))
            {
                return true;
            }
        }
    }
    for z in z0..=z1 {
        for y in y0..=y1 {
            if chunk
                .get_block_state(x0, y, z)
                .is_some_and(|block| mineshaft_block_is_liquid(&block))
                || chunk
                    .get_block_state(x1, y, z)
                    .is_some_and(|block| mineshaft_block_is_liquid(&block))
            {
                return true;
            }
        }
    }
    false
}

fn mineshaft_place_cave_air_blocks(
    chunk: &mut LevelChunk,
    blocks: Vec<StructurePiecePlacementBlock>,
) -> usize {
    mineshaft_place_blocks(chunk, blocks, Some("minecraft:cave_air"))
}

fn mineshaft_place_blocks(
    chunk: &mut LevelChunk,
    blocks: Vec<StructurePiecePlacementBlock>,
    force_state: Option<&'static str>,
) -> usize {
    let mut placed = 0;
    for block in blocks {
        chunk.set_block_state(
            block.world_pos.x,
            block.world_pos.y,
            block.world_pos.z,
            force_state.unwrap_or(block.state),
        );
        placed += 1;
    }
    placed
}

fn mineshaft_generate_world_box(
    chunk_bb: StructureBoundingBoxModel,
    min: BlockPos,
    max: BlockPos,
) -> Vec<StructurePiecePlacementBlock> {
    structure_piece_generate_box(
        StructurePieceBoxInput {
            bounding_box: StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 0,
                max_z: 0,
            },
            orientation: None,
            chunk_bb,
            min,
            max,
        },
        "minecraft:cave_air",
        "minecraft:cave_air",
        false,
        |_| false,
    )
}

fn mineshaft_apply_piece_to_chunk(
    chunk: &mut LevelChunk,
    piece: &MineshaftGeneratedPieceModel,
    chunk_bb: StructureBoundingBoxModel,
    random: &mut RandomSourceKind,
) -> usize {
    let piece_box = piece.bounding_box();
    if !piece_box.intersects(chunk_bb)
        || mineshaft_piece_is_in_invalid_location(chunk, piece_box, chunk_bb)
    {
        return 0;
    }
    match piece {
        MineshaftGeneratedPieceModel::Room {
            bounding_box,
            child_entrance_boxes,
            ..
        } => {
            let mut placed = 0;
            placed += mineshaft_place_cave_air_blocks(
                chunk,
                mineshaft_generate_world_box(
                    chunk_bb,
                    BlockPos {
                        x: bounding_box.min_x,
                        y: bounding_box.min_y + 1,
                        z: bounding_box.min_z,
                    },
                    BlockPos {
                        x: bounding_box.max_x,
                        y: (bounding_box.min_y + 3).min(bounding_box.max_y),
                        z: bounding_box.max_z,
                    },
                ),
            );
            for entrance in child_entrance_boxes {
                placed += mineshaft_place_cave_air_blocks(
                    chunk,
                    mineshaft_generate_world_box(
                        chunk_bb,
                        BlockPos {
                            x: entrance.min_x,
                            y: entrance.max_y - 2,
                            z: entrance.min_z,
                        },
                        BlockPos {
                            x: entrance.max_x,
                            y: entrance.max_y,
                            z: entrance.max_z,
                        },
                    ),
                );
            }
            placed += mineshaft_place_cave_air_blocks(
                chunk,
                structure_piece_generate_upper_half_sphere(
                    StructurePieceSphereInput {
                        box_input: StructurePieceBoxInput {
                            bounding_box: *bounding_box,
                            orientation: None,
                            chunk_bb,
                            min: BlockPos {
                                x: bounding_box.min_x,
                                y: bounding_box.min_y + 4,
                                z: bounding_box.min_z,
                            },
                            max: BlockPos {
                                x: bounding_box.max_x,
                                y: bounding_box.max_y,
                                z: bounding_box.max_z,
                            },
                        },
                        fill_block: "minecraft:cave_air",
                        skip_air: false,
                    },
                    |_| false,
                ),
            );
            placed
        }
        MineshaftGeneratedPieceModel::Corridor { model, .. } => {
            let length = model.num_sections * 5 - 1;
            let mut placed = mineshaft_place_cave_air_blocks(
                chunk,
                structure_piece_generate_box(
                    StructurePieceBoxInput {
                        bounding_box: model.bounding_box,
                        orientation: Some(model.orientation),
                        chunk_bb,
                        min: BlockPos { x: 0, y: 0, z: 0 },
                        max: BlockPos {
                            x: 2,
                            y: 1,
                            z: length,
                        },
                    },
                    "minecraft:cave_air",
                    "minecraft:cave_air",
                    false,
                    |_| false,
                ),
            );
            let ceiling_rolls = (0..=length)
                .flat_map(|_| {
                    [
                        feature_random_next_f32(random),
                        feature_random_next_f32(random),
                        feature_random_next_f32(random),
                    ]
                })
                .collect::<Vec<_>>();
            placed += mineshaft_place_cave_air_blocks(
                chunk,
                structure_piece_generate_maybe_box(
                    StructurePieceMaybeBoxInput {
                        box_input: StructurePieceBoxInput {
                            bounding_box: model.bounding_box,
                            orientation: Some(model.orientation),
                            chunk_bb,
                            min: BlockPos { x: 0, y: 2, z: 0 },
                            max: BlockPos {
                                x: 2,
                                y: 2,
                                z: length,
                            },
                        },
                        random_values: &ceiling_rolls,
                        probability: 0.8,
                        edge_block: "minecraft:cave_air",
                        fill_block: "minecraft:cave_air",
                        skip_air: false,
                        has_to_be_inside: false,
                    },
                    |_| false,
                    |_| false,
                ),
            );
            placed +=
                mineshaft_postprocess_corridor_details(chunk, model, chunk_bb, random, length);
            placed
        }
        MineshaftGeneratedPieceModel::Crossing {
            bounding_box,
            is_two_floored,
            ..
        } => {
            let mut placed = 0;
            if *is_two_floored {
                for (min, max) in [
                    (
                        BlockPos {
                            x: bounding_box.min_x + 1,
                            y: bounding_box.min_y,
                            z: bounding_box.min_z,
                        },
                        BlockPos {
                            x: bounding_box.max_x - 1,
                            y: bounding_box.min_y + 2,
                            z: bounding_box.max_z,
                        },
                    ),
                    (
                        BlockPos {
                            x: bounding_box.min_x,
                            y: bounding_box.min_y,
                            z: bounding_box.min_z + 1,
                        },
                        BlockPos {
                            x: bounding_box.max_x,
                            y: bounding_box.min_y + 2,
                            z: bounding_box.max_z - 1,
                        },
                    ),
                    (
                        BlockPos {
                            x: bounding_box.min_x + 1,
                            y: bounding_box.max_y - 2,
                            z: bounding_box.min_z,
                        },
                        BlockPos {
                            x: bounding_box.max_x - 1,
                            y: bounding_box.max_y,
                            z: bounding_box.max_z,
                        },
                    ),
                    (
                        BlockPos {
                            x: bounding_box.min_x,
                            y: bounding_box.max_y - 2,
                            z: bounding_box.min_z + 1,
                        },
                        BlockPos {
                            x: bounding_box.max_x,
                            y: bounding_box.max_y,
                            z: bounding_box.max_z - 1,
                        },
                    ),
                    (
                        BlockPos {
                            x: bounding_box.min_x + 1,
                            y: bounding_box.min_y + 3,
                            z: bounding_box.min_z + 1,
                        },
                        BlockPos {
                            x: bounding_box.max_x - 1,
                            y: bounding_box.min_y + 3,
                            z: bounding_box.max_z - 1,
                        },
                    ),
                ] {
                    placed += mineshaft_place_cave_air_blocks(
                        chunk,
                        mineshaft_generate_world_box(chunk_bb, min, max),
                    );
                }
            } else {
                for (min, max) in [
                    (
                        BlockPos {
                            x: bounding_box.min_x + 1,
                            y: bounding_box.min_y,
                            z: bounding_box.min_z,
                        },
                        BlockPos {
                            x: bounding_box.max_x - 1,
                            y: bounding_box.max_y,
                            z: bounding_box.max_z,
                        },
                    ),
                    (
                        BlockPos {
                            x: bounding_box.min_x,
                            y: bounding_box.min_y,
                            z: bounding_box.min_z + 1,
                        },
                        BlockPos {
                            x: bounding_box.max_x,
                            y: bounding_box.max_y,
                            z: bounding_box.max_z - 1,
                        },
                    ),
                ] {
                    placed += mineshaft_place_cave_air_blocks(
                        chunk,
                        mineshaft_generate_world_box(chunk_bb, min, max),
                    );
                }
            }
            placed
        }
        MineshaftGeneratedPieceModel::Stairs {
            bounding_box,
            direction,
            ..
        } => {
            let mut placed = 0;
            for (min, max) in [
                (BlockPos { x: 0, y: 5, z: 0 }, BlockPos { x: 2, y: 7, z: 1 }),
                (BlockPos { x: 0, y: 0, z: 7 }, BlockPos { x: 2, y: 2, z: 8 }),
            ] {
                placed += mineshaft_place_cave_air_blocks(
                    chunk,
                    structure_piece_generate_box(
                        StructurePieceBoxInput {
                            bounding_box: *bounding_box,
                            orientation: Some(*direction),
                            chunk_bb,
                            min,
                            max,
                        },
                        "minecraft:cave_air",
                        "minecraft:cave_air",
                        false,
                        |_| false,
                    ),
                );
            }
            for i in 0..5 {
                placed += mineshaft_place_cave_air_blocks(
                    chunk,
                    structure_piece_generate_box(
                        StructurePieceBoxInput {
                            bounding_box: *bounding_box,
                            orientation: Some(*direction),
                            chunk_bb,
                            min: BlockPos {
                                x: 0,
                                y: 5 - i - if i < 4 { 1 } else { 0 },
                                z: 2 + i,
                            },
                            max: BlockPos {
                                x: 2,
                                y: 7 - i,
                                z: 2 + i,
                            },
                        },
                        "minecraft:cave_air",
                        "minecraft:cave_air",
                        false,
                        |_| false,
                    ),
                );
            }
            placed
        }
    }
}

fn mineshaft_postprocess_corridor_details(
    chunk: &mut LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    random: &mut RandomSourceKind,
    length: i32,
) -> usize {
    let materials = mineshaft_materials(model.mineshaft_type);
    let mut placed = 0;

    if model.spider_corridor {
        let spider_rolls = (0..6 * (length + 1))
            .map(|_| feature_random_next_f32(random))
            .collect::<Vec<_>>();
        placed += mineshaft_place_blocks(
            chunk,
            structure_piece_generate_maybe_box(
                StructurePieceMaybeBoxInput {
                    box_input: StructurePieceBoxInput {
                        bounding_box: model.bounding_box,
                        orientation: Some(model.orientation),
                        chunk_bb,
                        min: BlockPos { x: 0, y: 0, z: 0 },
                        max: BlockPos {
                            x: 2,
                            y: 1,
                            z: length,
                        },
                    },
                    random_values: &spider_rolls,
                    probability: 0.6,
                    edge_block: "minecraft:cobweb",
                    fill_block: "minecraft:cave_air",
                    skip_air: false,
                    has_to_be_inside: true,
                },
                |_| false,
                |pos| {
                    mineshaft_world_pos_is_interior(
                        chunk,
                        chunk_bb,
                        BlockPos {
                            x: pos.x,
                            y: pos.y + 1,
                            z: pos.z,
                        },
                    )
                },
            ),
            None,
        );
    }

    let mut has_placed_spider = model.has_placed_spider;
    for section in 0..model.num_sections {
        let z = 2 + section * 5;
        placed += mineshaft_place_corridor_support(chunk, model, chunk_bb, random, z, materials);

        for (probability, x, y, web_z) in [
            (0.1, 0, 2, z - 1),
            (0.1, 2, 2, z - 1),
            (0.1, 0, 2, z + 1),
            (0.1, 2, 2, z + 1),
            (0.05, 0, 2, z - 2),
            (0.05, 2, 2, z - 2),
            (0.05, 0, 2, z + 2),
            (0.05, 2, 2, z + 2),
        ] {
            if mineshaft_corridor_is_interior(chunk, model, chunk_bb, x, y, web_z) {
                let roll = feature_random_next_f32(random);
                if roll < probability
                    && mineshaft_corridor_has_sturdy_neighbors(
                        chunk, model, chunk_bb, x, y, web_z, 2,
                    )
                {
                    placed += mineshaft_place_corridor_block(
                        chunk,
                        model,
                        chunk_bb,
                        x,
                        y,
                        web_z,
                        "minecraft:cobweb",
                    );
                }
            }
        }

        if feature_random_next_i32_bound(random, 100) == 0 {
            placed += mineshaft_place_corridor_block(
                chunk,
                model,
                chunk_bb,
                2,
                0,
                z - 1,
                "minecraft:chest",
            );
        }
        if feature_random_next_i32_bound(random, 100) == 0 {
            placed += mineshaft_place_corridor_block(
                chunk,
                model,
                chunk_bb,
                0,
                0,
                z + 1,
                "minecraft:chest",
            );
        }

        if model.spider_corridor && !has_placed_spider {
            let new_z = z - 1 + feature_random_next_i32_bound(random, 3);
            let pos =
                structure_piece_world_pos(model.bounding_box, Some(model.orientation), 1, 0, new_z);
            if chunk_bb.is_inside(pos)
                && mineshaft_corridor_is_interior(chunk, model, chunk_bb, 1, 0, new_z)
            {
                has_placed_spider = true;
                placed += mineshaft_place_corridor_block(
                    chunk,
                    model,
                    chunk_bb,
                    1,
                    0,
                    new_z,
                    "minecraft:spawner",
                );
            }
        }
    }

    for x in 0..=2 {
        for z in 0..=length {
            let pos =
                structure_piece_world_pos(model.bounding_box, Some(model.orientation), x, -1, z);
            let interior_pos = BlockPos {
                x: pos.x,
                y: pos.y + 1,
                z: pos.z,
            };
            if chunk_bb.is_inside(pos)
                && mineshaft_world_pos_is_interior(chunk, chunk_bb, interior_pos)
                && !chunk
                    .get_block_state(pos.x, pos.y, pos.z)
                    .is_some_and(|block| block_blocks_motion(&block))
            {
                chunk.set_block_state(pos.x, pos.y, pos.z, materials.planks_state);
                placed += 1;
            }
        }
    }

    if model.has_rails {
        for z in 0..=length {
            let pos =
                structure_piece_world_pos(model.bounding_box, Some(model.orientation), 1, -1, z);
            let floor = chunk
                .get_block_state(pos.x, pos.y, pos.z)
                .unwrap_or_else(|| "minecraft:air".to_string());
            if floor != "minecraft:air" && floor != "minecraft:cave_air" {
                let probability = if mineshaft_corridor_is_interior(chunk, model, chunk_bb, 1, 0, z)
                {
                    0.7
                } else {
                    0.9
                };
                if feature_random_next_f32(random) < probability {
                    placed += mineshaft_place_corridor_block(
                        chunk,
                        model,
                        chunk_bb,
                        1,
                        0,
                        z,
                        "minecraft:rail",
                    );
                }
            }
        }
    }
    placed
}

fn mineshaft_place_corridor_support(
    chunk: &mut LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    random: &mut RandomSourceKind,
    z: i32,
    materials: MineshaftMaterialModel,
) -> usize {
    if !mineshaft_corridor_supporting_box(chunk, model, chunk_bb, 0, 2, 2, z) {
        return 0;
    }
    let mut placed = 0;
    for y in 0..=1 {
        placed +=
            mineshaft_place_corridor_block(chunk, model, chunk_bb, 0, y, z, materials.fence_state);
        placed +=
            mineshaft_place_corridor_block(chunk, model, chunk_bb, 2, y, z, materials.fence_state);
    }
    if feature_random_next_i32_bound(random, 4) == 0 {
        placed +=
            mineshaft_place_corridor_block(chunk, model, chunk_bb, 0, 2, z, materials.planks_state);
        placed +=
            mineshaft_place_corridor_block(chunk, model, chunk_bb, 2, 2, z, materials.planks_state);
    } else {
        for x in 0..=2 {
            placed += mineshaft_place_corridor_block(
                chunk,
                model,
                chunk_bb,
                x,
                2,
                z,
                materials.planks_state,
            );
        }
        if feature_random_next_f32(random) < 0.05 {
            placed += mineshaft_place_corridor_block(
                chunk,
                model,
                chunk_bb,
                1,
                2,
                z - 1,
                "minecraft:wall_torch",
            );
        }
        if feature_random_next_f32(random) < 0.05 {
            placed += mineshaft_place_corridor_block(
                chunk,
                model,
                chunk_bb,
                1,
                2,
                z + 1,
                "minecraft:wall_torch",
            );
        }
    }
    placed
}

fn mineshaft_place_corridor_block(
    chunk: &mut LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    x: i32,
    y: i32,
    z: i32,
    state: &'static str,
) -> usize {
    let pos = structure_piece_world_pos(model.bounding_box, Some(model.orientation), x, y, z);
    if !chunk_bb.is_inside(pos) {
        return 0;
    }
    chunk.set_block_state(pos.x, pos.y, pos.z, state);
    1
}

fn mineshaft_corridor_supporting_box(
    chunk: &LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    x0: i32,
    x1: i32,
    y1: i32,
    z: i32,
) -> bool {
    (x0..=x1).all(|x| {
        let pos =
            structure_piece_world_pos(model.bounding_box, Some(model.orientation), x, y1 + 1, z);
        chunk_bb.is_inside(pos)
            && !matches!(
                chunk.get_block_state(pos.x, pos.y, pos.z).as_deref(),
                None | Some("minecraft:air") | Some("minecraft:cave_air")
            )
    })
}

fn mineshaft_corridor_is_interior(
    chunk: &LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    x: i32,
    y: i32,
    z: i32,
) -> bool {
    let pos = structure_piece_world_pos(model.bounding_box, Some(model.orientation), x, y + 1, z);
    mineshaft_world_pos_is_interior(chunk, chunk_bb, pos)
}

fn mineshaft_world_pos_is_interior(
    chunk: &LevelChunk,
    chunk_bb: StructureBoundingBoxModel,
    pos: BlockPos,
) -> bool {
    chunk_bb.is_inside(pos)
        && chunk
            .heightmap_value(
                HeightmapKind::OceanFloorWg,
                pos.x.rem_euclid(16) as usize,
                pos.z.rem_euclid(16) as usize,
            )
            .is_some_and(|height| pos.y < height)
}

fn mineshaft_corridor_has_sturdy_neighbors(
    chunk: &LevelChunk,
    model: &MineshaftCorridorModel,
    chunk_bb: StructureBoundingBoxModel,
    x: i32,
    y: i32,
    z: i32,
    count: i32,
) -> bool {
    const OFFSETS: [(i32, i32, i32); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];
    let mut sturdy = 0;
    let center = structure_piece_world_pos(model.bounding_box, Some(model.orientation), x, y, z);
    for (dx, dy, dz) in OFFSETS {
        let pos = BlockPos {
            x: center.x + dx,
            y: center.y + dy,
            z: center.z + dz,
        };
        if chunk_bb.is_inside(pos)
            && chunk
                .get_block_state(pos.x, pos.y, pos.z)
                .is_some_and(|block| block_blocks_motion(&block))
        {
            sturdy += 1;
            if sturdy >= count {
                return true;
            }
        }
    }
    false
}

pub fn apply_mineshaft_underground_structures_to_chunk(chunk: &mut LevelChunk, seed: i64) -> usize {
    let target_pos = ChunkPos {
        x: chunk.pos.x,
        z: chunk.pos.z,
    };
    let decoration_seed = crate::random_source::decoration_seed(
        seed,
        target_pos.x * 16,
        target_pos.z * 16,
        RandomAlgorithm::Xoroshiro,
    );
    let mut postprocess_random = RandomSourceKind::new(
        crate::random_source::feature_seed(
            decoration_seed,
            0,
            GenerationDecorationStep::UndergroundStructures as i32,
        ),
        RandomAlgorithm::Xoroshiro,
    );
    let chunk_bb = mineshaft_chunk_bounding_box(target_pos);
    let mut placed = 0;
    for source_x in target_pos.x - 8..=target_pos.x + 8 {
        for source_z in target_pos.z - 8..=target_pos.z + 8 {
            if !structure_frequency_reducer_should_generate(
                FrequencyReductionMethod::LegacyType3,
                seed,
                0,
                source_x,
                source_z,
                0.004,
            )
            .unwrap_or(false)
            {
                continue;
            }
            let pieces = mineshaft_generate_pieces_for_start(
                seed,
                ChunkPos {
                    x: source_x,
                    z: source_z,
                },
                MineshaftTypeModel::Normal,
                63,
                -64,
            );
            for piece in &pieces {
                placed +=
                    mineshaft_apply_piece_to_chunk(chunk, piece, chunk_bb, &mut postprocess_random);
            }
        }
    }
    placed
}
