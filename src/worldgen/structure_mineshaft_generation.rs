use super::*;

pub fn mineshaft_type_by_id(id: i32) -> MineshaftTypeModel {
    match id {
        1 => MineshaftTypeModel::Mesa,
        _ => MineshaftTypeModel::Normal,
    }
}

pub fn mineshaft_type_id(mineshaft_type: MineshaftTypeModel) -> i32 {
    match mineshaft_type {
        MineshaftTypeModel::Normal => 0,
        MineshaftTypeModel::Mesa => 1,
    }
}

pub fn mineshaft_materials(mineshaft_type: MineshaftTypeModel) -> MineshaftMaterialModel {
    match mineshaft_type {
        MineshaftTypeModel::Normal => MineshaftMaterialModel {
            serialized_name: "normal",
            wood_state: "minecraft:oak_log",
            planks_state: "minecraft:oak_planks",
            fence_state: "minecraft:oak_fence",
        },
        MineshaftTypeModel::Mesa => MineshaftMaterialModel {
            serialized_name: "mesa",
            wood_state: "minecraft:dark_oak_log",
            planks_state: "minecraft:dark_oak_planks",
            fence_state: "minecraft:dark_oak_fence",
        },
    }
}

pub fn mineshaft_start_pos(chunk_pos: ChunkPos) -> BlockPos {
    BlockPos {
        x: chunk_pos.x * 16 + 8,
        y: 50,
        z: chunk_pos.z * 16,
    }
}

pub fn mineshaft_room(
    chunk_pos: ChunkPos,
    mineshaft_type: MineshaftTypeModel,
    x_size_roll: i32,
    y_size_roll: i32,
    z_size_roll: i32,
) -> Result<MineshaftRoomModel, String> {
    if !(0..6).contains(&x_size_roll)
        || !(0..6).contains(&y_size_roll)
        || !(0..6).contains(&z_size_roll)
    {
        return Err("Mineshaft room size rolls must match RandomSource#nextInt(6)".to_string());
    }
    let west = chunk_pos.x * 16 + 2;
    let north = chunk_pos.z * 16 + 2;
    Ok(MineshaftRoomModel {
        bounding_box: StructureBoundingBoxModel {
            min_x: west,
            min_y: 50,
            min_z: north,
            max_x: west + 7 + x_size_roll,
            max_y: 54 + y_size_roll,
            max_z: north + 7 + z_size_roll,
        },
        mineshaft_type,
    })
}

pub fn mineshaft_move_below_sea_level_dy(
    bounding_box: StructureBoundingBoxModel,
    sea_level: i32,
    min_y: i32,
    offset: i32,
    random_roll: i32,
) -> Result<i32, String> {
    let max_y = sea_level - offset;
    let mut y1_pos = (bounding_box.max_y - bounding_box.min_y + 1) + min_y + 1;
    if y1_pos < max_y {
        let span = max_y - y1_pos;
        if !(0..span).contains(&random_roll) {
            return Err(
                "Mineshaft sea-level move roll is outside RandomSource#nextInt span".to_string(),
            );
        }
        y1_pos += random_roll;
    }
    Ok(y1_pos - bounding_box.max_y)
}

pub fn mineshaft_mesa_vertical_dy(
    bounding_box: StructureBoundingBoxModel,
    sea_level: i32,
    surface_height: i32,
    random_roll: i32,
) -> Result<i32, String> {
    let center = bounding_box.center();
    let target_y_for_center = if surface_height <= sea_level {
        sea_level
    } else {
        let span = surface_height - sea_level + 1;
        if !(0..span).contains(&random_roll) {
            return Err(
                "Mineshaft mesa height roll is outside sea-level-to-surface range".to_string(),
            );
        }
        sea_level + random_roll
    };
    Ok(target_y_for_center - center.y)
}

pub fn mineshaft_find_corridor_size(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    corridor_length_roll: i32,
    existing_pieces: &[StructurePieceModel],
) -> Result<Option<StructureBoundingBoxModel>, String> {
    if !(0..3).contains(&corridor_length_roll) {
        return Err(
            "Mineshaft corridor length roll must match RandomSource#nextInt(3)".to_string(),
        );
    }
    for corridor_length in (1..=corridor_length_roll + 2).rev() {
        let block_length = corridor_length * 5;
        let mut box_ = match direction {
            HorizontalDirection::North => StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: -(block_length - 1),
                max_x: 2,
                max_y: 2,
                max_z: 0,
            },
            HorizontalDirection::South => StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: 2,
                max_y: 2,
                max_z: block_length - 1,
            },
            HorizontalDirection::West => StructureBoundingBoxModel {
                min_x: -(block_length - 1),
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 2,
                max_z: 2,
            },
            HorizontalDirection::East => StructureBoundingBoxModel {
                min_x: 0,
                min_y: 0,
                min_z: 0,
                max_x: block_length - 1,
                max_y: 2,
                max_z: 2,
            },
        };
        box_ = box_.moved(foot_x, foot_y, foot_z);
        if structure_piece_find_collision_piece(existing_pieces, box_).is_none() {
            return Ok(Some(box_));
        }
    }
    Ok(None)
}

pub fn mineshaft_find_crossing(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    two_floor_roll: i32,
    existing_pieces: &[StructurePieceModel],
) -> Result<Option<StructureBoundingBoxModel>, String> {
    if !(0..4).contains(&two_floor_roll) {
        return Err("Mineshaft crossing floor roll must match RandomSource#nextInt(4)".to_string());
    }
    let y1 = if two_floor_roll == 0 { 6 } else { 2 };
    let box_ = match direction {
        HorizontalDirection::North => StructureBoundingBoxModel {
            min_x: -1,
            min_y: 0,
            min_z: -4,
            max_x: 3,
            max_y: y1,
            max_z: 0,
        },
        HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: -1,
            min_y: 0,
            min_z: 0,
            max_x: 3,
            max_y: y1,
            max_z: 4,
        },
        HorizontalDirection::West => StructureBoundingBoxModel {
            min_x: -4,
            min_y: 0,
            min_z: -1,
            max_x: 0,
            max_y: y1,
            max_z: 3,
        },
        HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: 0,
            min_y: 0,
            min_z: -1,
            max_x: 4,
            max_y: y1,
            max_z: 3,
        },
    }
    .moved(foot_x, foot_y, foot_z);
    Ok(structure_piece_find_collision_piece(existing_pieces, box_)
        .is_none()
        .then_some(box_))
}

pub fn mineshaft_find_stairs(
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    existing_pieces: &[StructurePieceModel],
) -> Option<StructureBoundingBoxModel> {
    let box_ = match direction {
        HorizontalDirection::North => StructureBoundingBoxModel {
            min_x: 0,
            min_y: -5,
            min_z: -8,
            max_x: 2,
            max_y: 2,
            max_z: 0,
        },
        HorizontalDirection::South => StructureBoundingBoxModel {
            min_x: 0,
            min_y: -5,
            min_z: 0,
            max_x: 2,
            max_y: 2,
            max_z: 8,
        },
        HorizontalDirection::West => StructureBoundingBoxModel {
            min_x: -8,
            min_y: -5,
            min_z: 0,
            max_x: 0,
            max_y: 2,
            max_z: 2,
        },
        HorizontalDirection::East => StructureBoundingBoxModel {
            min_x: 0,
            min_y: -5,
            min_z: 0,
            max_x: 8,
            max_y: 2,
            max_z: 2,
        },
    }
    .moved(foot_x, foot_y, foot_z);
    structure_piece_find_collision_piece(existing_pieces, box_)
        .is_none()
        .then_some(box_)
}

pub fn mineshaft_random_piece_kind(selection_roll: i32) -> Result<MineshaftPieceKindModel, String> {
    if !(0..100).contains(&selection_roll) {
        return Err(
            "Mineshaft piece selection roll must match RandomSource#nextInt(100)".to_string(),
        );
    }
    Ok(if selection_roll >= 80 {
        MineshaftPieceKindModel::Crossing
    } else if selection_roll >= 70 {
        MineshaftPieceKindModel::Stairs
    } else {
        MineshaftPieceKindModel::Corridor
    })
}

pub fn mineshaft_corridor(
    bounding_box: StructureBoundingBoxModel,
    orientation: HorizontalDirection,
    mineshaft_type: MineshaftTypeModel,
    rails_roll: i32,
    spider_roll: i32,
) -> Result<MineshaftCorridorModel, String> {
    if !(0..3).contains(&rails_roll) {
        return Err("Mineshaft rails roll must match RandomSource#nextInt(3)".to_string());
    }
    if !(0..23).contains(&spider_roll) {
        return Err("Mineshaft spider roll must match RandomSource#nextInt(23)".to_string());
    }
    let has_rails = rails_roll == 0;
    let spider_corridor = !has_rails && spider_roll == 0;
    let num_sections = if matches!(
        orientation,
        HorizontalDirection::North | HorizontalDirection::South
    ) {
        (bounding_box.max_z - bounding_box.min_z + 1) / 5
    } else {
        (bounding_box.max_x - bounding_box.min_x + 1) / 5
    };
    Ok(MineshaftCorridorModel {
        bounding_box,
        orientation,
        mineshaft_type,
        has_rails,
        spider_corridor,
        has_placed_spider: false,
        num_sections,
    })
}

pub fn mineshaft_corridor_save_tag(
    corridor: MineshaftCorridorModel,
) -> MineshaftCorridorSaveTagModel {
    MineshaftCorridorSaveTagModel {
        has_rails: corridor.has_rails,
        spider_corridor: corridor.spider_corridor,
        has_placed_spider: corridor.has_placed_spider,
        num_sections: corridor.num_sections,
        mineshaft_type_id: mineshaft_type_id(corridor.mineshaft_type),
    }
}

fn mineshaft_piece_bounding_boxes(
    pieces: &[MineshaftGeneratedPieceModel],
) -> Vec<StructurePieceModel> {
    pieces
        .iter()
        .map(MineshaftGeneratedPieceModel::as_structure_piece)
        .collect()
}

fn mineshaft_generated_piece_type(piece: &MineshaftGeneratedPieceModel) -> MineshaftTypeModel {
    match piece {
        MineshaftGeneratedPieceModel::Room { mineshaft_type, .. }
        | MineshaftGeneratedPieceModel::Crossing { mineshaft_type, .. }
        | MineshaftGeneratedPieceModel::Stairs { mineshaft_type, .. } => *mineshaft_type,
        MineshaftGeneratedPieceModel::Corridor { model, .. } => model.mineshaft_type,
    }
}

fn mineshaft_create_random_piece(
    random: &mut RandomSourceKind,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    gen_depth: i32,
    mineshaft_type: MineshaftTypeModel,
    existing_pieces: &[MineshaftGeneratedPieceModel],
) -> Option<MineshaftGeneratedPieceModel> {
    let existing_boxes = mineshaft_piece_bounding_boxes(existing_pieces);
    match mineshaft_random_piece_kind(random_next_i32_bound(random, 100)).ok()? {
        MineshaftPieceKindModel::Crossing => {
            let bounding_box = mineshaft_find_crossing(
                foot_x,
                foot_y,
                foot_z,
                direction,
                random_next_i32_bound(random, 4),
                &existing_boxes,
            )
            .ok()
            .flatten()?;
            Some(MineshaftGeneratedPieceModel::Crossing {
                bounding_box,
                direction,
                mineshaft_type,
                is_two_floored: bounding_box.max_y - bounding_box.min_y + 1 > 3,
                gen_depth,
            })
        }
        MineshaftPieceKindModel::Stairs => {
            let bounding_box =
                mineshaft_find_stairs(foot_x, foot_y, foot_z, direction, &existing_boxes)?;
            Some(MineshaftGeneratedPieceModel::Stairs {
                bounding_box,
                direction,
                mineshaft_type,
                gen_depth,
            })
        }
        MineshaftPieceKindModel::Corridor => {
            let bounding_box = mineshaft_find_corridor_size(
                foot_x,
                foot_y,
                foot_z,
                direction,
                random_next_i32_bound(random, 3),
                &existing_boxes,
            )
            .ok()
            .flatten()?;
            let rails_roll = random_next_i32_bound(random, 3);
            // Java uses short-circuit evaluation here:
            // `spiderCorridor = !hasRails && random.nextInt(23) == 0`.
            // When rails are present, the spider roll is not consumed, so the
            // remaining mineshaft piece chain must keep that PRNG state.
            let spider_roll = if rails_roll == 0 {
                1
            } else {
                random_next_i32_bound(random, 23)
            };
            let model = mineshaft_corridor(
                bounding_box,
                direction,
                mineshaft_type,
                rails_roll,
                spider_roll,
            )
            .ok()?;
            Some(MineshaftGeneratedPieceModel::Corridor { model, gen_depth })
        }
    }
}

fn mineshaft_generate_and_add_piece(
    start_box: StructureBoundingBoxModel,
    pieces: &mut Vec<MineshaftGeneratedPieceModel>,
    random: &mut RandomSourceKind,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: HorizontalDirection,
    depth: i32,
) -> Option<StructureBoundingBoxModel> {
    if depth > 8 || (foot_x - start_box.min_x).abs() > 80 || (foot_z - start_box.min_z).abs() > 80 {
        return None;
    }
    let mineshaft_type = pieces
        .first()
        .map(mineshaft_generated_piece_type)
        .unwrap_or(MineshaftTypeModel::Normal);
    let piece = mineshaft_create_random_piece(
        random,
        foot_x,
        foot_y,
        foot_z,
        direction,
        depth + 1,
        mineshaft_type,
        pieces,
    )?;
    let bounding_box = piece.bounding_box();
    pieces.push(piece);
    let piece_index = pieces.len() - 1;
    mineshaft_add_children(piece_index, start_box, pieces, random);
    Some(bounding_box)
}

fn mineshaft_add_children(
    piece_index: usize,
    start_box: StructureBoundingBoxModel,
    pieces: &mut Vec<MineshaftGeneratedPieceModel>,
    random: &mut RandomSourceKind,
) {
    let piece = pieces[piece_index].clone();
    match piece {
        MineshaftGeneratedPieceModel::Room {
            bounding_box,
            gen_depth,
            ..
        } => {
            let mut entrance_boxes = Vec::new();
            let mut height_space = bounding_box.max_y - bounding_box.min_y + 1 - 3 - 1;
            if height_space <= 0 {
                height_space = 1;
            }

            let mut pos = 0;
            while pos < bounding_box.max_x - bounding_box.min_x + 1 {
                pos += random_next_i32_bound(random, bounding_box.max_x - bounding_box.min_x + 1);
                if pos + 3 > bounding_box.max_x - bounding_box.min_x + 1 {
                    break;
                }
                let foot_y = bounding_box.min_y + random_next_i32_bound(random, height_space) + 1;
                if let Some(child_box) = mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x + pos,
                    foot_y,
                    bounding_box.min_z - 1,
                    HorizontalDirection::North,
                    gen_depth,
                ) {
                    entrance_boxes.push(StructureBoundingBoxModel {
                        min_x: child_box.min_x,
                        min_y: child_box.min_y,
                        min_z: bounding_box.min_z,
                        max_x: child_box.max_x,
                        max_y: child_box.max_y,
                        max_z: bounding_box.min_z + 1,
                    });
                }
                pos += 4;
            }

            pos = 0;
            while pos < bounding_box.max_x - bounding_box.min_x + 1 {
                pos += random_next_i32_bound(random, bounding_box.max_x - bounding_box.min_x + 1);
                if pos + 3 > bounding_box.max_x - bounding_box.min_x + 1 {
                    break;
                }
                let foot_y = bounding_box.min_y + random_next_i32_bound(random, height_space) + 1;
                if let Some(child_box) = mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x + pos,
                    foot_y,
                    bounding_box.max_z + 1,
                    HorizontalDirection::South,
                    gen_depth,
                ) {
                    entrance_boxes.push(StructureBoundingBoxModel {
                        min_x: child_box.min_x,
                        min_y: child_box.min_y,
                        min_z: bounding_box.max_z - 1,
                        max_x: child_box.max_x,
                        max_y: child_box.max_y,
                        max_z: bounding_box.max_z,
                    });
                }
                pos += 4;
            }

            pos = 0;
            while pos < bounding_box.max_z - bounding_box.min_z + 1 {
                pos += random_next_i32_bound(random, bounding_box.max_z - bounding_box.min_z + 1);
                if pos + 3 > bounding_box.max_z - bounding_box.min_z + 1 {
                    break;
                }
                let foot_y = bounding_box.min_y + random_next_i32_bound(random, height_space) + 1;
                if let Some(child_box) = mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x - 1,
                    foot_y,
                    bounding_box.min_z + pos,
                    HorizontalDirection::West,
                    gen_depth,
                ) {
                    entrance_boxes.push(StructureBoundingBoxModel {
                        min_x: bounding_box.min_x,
                        min_y: child_box.min_y,
                        min_z: child_box.min_z,
                        max_x: bounding_box.min_x + 1,
                        max_y: child_box.max_y,
                        max_z: child_box.max_z,
                    });
                }
                pos += 4;
            }

            pos = 0;
            while pos < bounding_box.max_z - bounding_box.min_z + 1 {
                pos += random_next_i32_bound(random, bounding_box.max_z - bounding_box.min_z + 1);
                if pos + 3 > bounding_box.max_z - bounding_box.min_z + 1 {
                    break;
                }
                let foot_y = bounding_box.min_y + random_next_i32_bound(random, height_space) + 1;
                if let Some(child_box) = mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.max_x + 1,
                    foot_y,
                    bounding_box.min_z + pos,
                    HorizontalDirection::East,
                    gen_depth,
                ) {
                    entrance_boxes.push(StructureBoundingBoxModel {
                        min_x: bounding_box.max_x - 1,
                        min_y: child_box.min_y,
                        min_z: child_box.min_z,
                        max_x: bounding_box.max_x,
                        max_y: child_box.max_y,
                        max_z: child_box.max_z,
                    });
                }
                pos += 4;
            }

            if let MineshaftGeneratedPieceModel::Room {
                child_entrance_boxes,
                ..
            } = &mut pieces[piece_index]
            {
                *child_entrance_boxes = entrance_boxes;
            }
        }
        MineshaftGeneratedPieceModel::Corridor { model, gen_depth } => {
            let end_selection = random_next_i32_bound(random, 4);
            match model.orientation {
                HorizontalDirection::North => {
                    if end_selection <= 1 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x,
                            foot_y,
                            model.bounding_box.min_z - 1,
                            model.orientation,
                            gen_depth,
                        );
                    } else if end_selection == 2 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x - 1,
                            foot_y,
                            model.bounding_box.min_z,
                            HorizontalDirection::West,
                            gen_depth,
                        );
                    } else {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.max_x + 1,
                            foot_y,
                            model.bounding_box.min_z,
                            HorizontalDirection::East,
                            gen_depth,
                        );
                    }
                }
                HorizontalDirection::South => {
                    if end_selection <= 1 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x,
                            foot_y,
                            model.bounding_box.max_z + 1,
                            model.orientation,
                            gen_depth,
                        );
                    } else if end_selection == 2 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x - 1,
                            foot_y,
                            model.bounding_box.max_z - 3,
                            HorizontalDirection::West,
                            gen_depth,
                        );
                    } else {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.max_x + 1,
                            foot_y,
                            model.bounding_box.max_z - 3,
                            HorizontalDirection::East,
                            gen_depth,
                        );
                    }
                }
                HorizontalDirection::West => {
                    if end_selection <= 1 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x - 1,
                            foot_y,
                            model.bounding_box.min_z,
                            model.orientation,
                            gen_depth,
                        );
                    } else if end_selection == 2 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x,
                            foot_y,
                            model.bounding_box.min_z - 1,
                            HorizontalDirection::North,
                            gen_depth,
                        );
                    } else {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.min_x,
                            foot_y,
                            model.bounding_box.max_z + 1,
                            HorizontalDirection::South,
                            gen_depth,
                        );
                    }
                }
                HorizontalDirection::East => {
                    if end_selection <= 1 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.max_x + 1,
                            foot_y,
                            model.bounding_box.min_z,
                            model.orientation,
                            gen_depth,
                        );
                    } else if end_selection == 2 {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.max_x - 3,
                            foot_y,
                            model.bounding_box.min_z - 1,
                            HorizontalDirection::North,
                            gen_depth,
                        );
                    } else {
                        let foot_y =
                            model.bounding_box.min_y - 1 + random_next_i32_bound(random, 3);
                        mineshaft_generate_and_add_piece(
                            start_box,
                            pieces,
                            random,
                            model.bounding_box.max_x - 3,
                            foot_y,
                            model.bounding_box.max_z + 1,
                            HorizontalDirection::South,
                            gen_depth,
                        );
                    }
                }
            }

            if gen_depth < 8 {
                if !matches!(
                    model.orientation,
                    HorizontalDirection::North | HorizontalDirection::South
                ) {
                    let mut x = model.bounding_box.min_x + 3;
                    while x + 3 <= model.bounding_box.max_x {
                        match random_next_i32_bound(random, 5) {
                            0 => {
                                mineshaft_generate_and_add_piece(
                                    start_box,
                                    pieces,
                                    random,
                                    x,
                                    model.bounding_box.min_y,
                                    model.bounding_box.min_z - 1,
                                    HorizontalDirection::North,
                                    gen_depth + 1,
                                );
                            }
                            1 => {
                                mineshaft_generate_and_add_piece(
                                    start_box,
                                    pieces,
                                    random,
                                    x,
                                    model.bounding_box.min_y,
                                    model.bounding_box.max_z + 1,
                                    HorizontalDirection::South,
                                    gen_depth + 1,
                                );
                            }
                            _ => {}
                        }
                        x += 5;
                    }
                } else {
                    let mut z = model.bounding_box.min_z + 3;
                    while z + 3 <= model.bounding_box.max_z {
                        match random_next_i32_bound(random, 5) {
                            0 => {
                                mineshaft_generate_and_add_piece(
                                    start_box,
                                    pieces,
                                    random,
                                    model.bounding_box.min_x - 1,
                                    model.bounding_box.min_y,
                                    z,
                                    HorizontalDirection::West,
                                    gen_depth + 1,
                                );
                            }
                            1 => {
                                mineshaft_generate_and_add_piece(
                                    start_box,
                                    pieces,
                                    random,
                                    model.bounding_box.max_x + 1,
                                    model.bounding_box.min_y,
                                    z,
                                    HorizontalDirection::East,
                                    gen_depth + 1,
                                );
                            }
                            _ => {}
                        }
                        z += 5;
                    }
                }
            }
        }
        MineshaftGeneratedPieceModel::Crossing {
            bounding_box,
            direction,
            is_two_floored,
            gen_depth,
            ..
        } => {
            match direction {
                HorizontalDirection::North => {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z - 1,
                        HorizontalDirection::North,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x - 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::West,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.max_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::East,
                        gen_depth,
                    );
                }
                HorizontalDirection::South => {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.max_z + 1,
                        HorizontalDirection::South,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x - 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::West,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.max_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::East,
                        gen_depth,
                    );
                }
                HorizontalDirection::West => {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z - 1,
                        HorizontalDirection::North,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.max_z + 1,
                        HorizontalDirection::South,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x - 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::West,
                        gen_depth,
                    );
                }
                HorizontalDirection::East => {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z - 1,
                        HorizontalDirection::North,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y,
                        bounding_box.max_z + 1,
                        HorizontalDirection::South,
                        gen_depth,
                    );
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.max_x + 1,
                        bounding_box.min_y,
                        bounding_box.min_z + 1,
                        HorizontalDirection::East,
                        gen_depth,
                    );
                }
            }

            if is_two_floored {
                if crate::random_source::random_source_next_bool(random) {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y + 4,
                        bounding_box.min_z - 1,
                        HorizontalDirection::North,
                        gen_depth,
                    );
                }
                if crate::random_source::random_source_next_bool(random) {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x - 1,
                        bounding_box.min_y + 4,
                        bounding_box.min_z + 1,
                        HorizontalDirection::West,
                        gen_depth,
                    );
                }
                if crate::random_source::random_source_next_bool(random) {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.max_x + 1,
                        bounding_box.min_y + 4,
                        bounding_box.min_z + 1,
                        HorizontalDirection::East,
                        gen_depth,
                    );
                }
                if crate::random_source::random_source_next_bool(random) {
                    mineshaft_generate_and_add_piece(
                        start_box,
                        pieces,
                        random,
                        bounding_box.min_x + 1,
                        bounding_box.min_y + 4,
                        bounding_box.max_z + 1,
                        HorizontalDirection::South,
                        gen_depth,
                    );
                }
            }
        }
        MineshaftGeneratedPieceModel::Stairs {
            bounding_box,
            direction,
            gen_depth,
            ..
        } => match direction {
            HorizontalDirection::North => {
                mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x,
                    bounding_box.min_y,
                    bounding_box.min_z - 1,
                    HorizontalDirection::North,
                    gen_depth,
                );
            }
            HorizontalDirection::South => {
                mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x,
                    bounding_box.min_y,
                    bounding_box.max_z + 1,
                    HorizontalDirection::South,
                    gen_depth,
                );
            }
            HorizontalDirection::West => {
                mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.min_x - 1,
                    bounding_box.min_y,
                    bounding_box.min_z,
                    HorizontalDirection::West,
                    gen_depth,
                );
            }
            HorizontalDirection::East => {
                mineshaft_generate_and_add_piece(
                    start_box,
                    pieces,
                    random,
                    bounding_box.max_x + 1,
                    bounding_box.min_y,
                    bounding_box.min_z,
                    HorizontalDirection::East,
                    gen_depth,
                );
            }
        },
    }
}

fn mineshaft_move_pieces(pieces: &mut [MineshaftGeneratedPieceModel], dy: i32) {
    for piece in pieces {
        match piece {
            MineshaftGeneratedPieceModel::Room {
                bounding_box,
                child_entrance_boxes,
                ..
            } => {
                *bounding_box = bounding_box.moved(0, dy, 0);
                for entrance in child_entrance_boxes {
                    *entrance = entrance.moved(0, dy, 0);
                }
            }
            MineshaftGeneratedPieceModel::Corridor { model, .. } => {
                model.bounding_box = model.bounding_box.moved(0, dy, 0);
            }
            MineshaftGeneratedPieceModel::Crossing { bounding_box, .. }
            | MineshaftGeneratedPieceModel::Stairs { bounding_box, .. } => {
                *bounding_box = bounding_box.moved(0, dy, 0);
            }
        }
    }
}

pub fn mineshaft_generate_pieces_for_start(
    seed: i64,
    chunk_pos: ChunkPos,
    mineshaft_type: MineshaftTypeModel,
    sea_level: i32,
    min_y: i32,
) -> Vec<MineshaftGeneratedPieceModel> {
    let mut random = RandomSourceKind::Legacy(LegacyRandom::new(
        crate::random_source::large_feature_seed(seed, chunk_pos.x, chunk_pos.z),
    ));
    let _generation_point_roll = random_next_f64(&mut random);
    let room = mineshaft_room(
        chunk_pos,
        mineshaft_type,
        random_next_i32_bound(&mut random, 6),
        random_next_i32_bound(&mut random, 6),
        random_next_i32_bound(&mut random, 6),
    )
    .expect("RandomSource#nextInt(6) room rolls should be valid");
    let start_box = room.bounding_box;
    let mut pieces = vec![MineshaftGeneratedPieceModel::Room {
        bounding_box: room.bounding_box,
        mineshaft_type,
        child_entrance_boxes: Vec::new(),
        gen_depth: 0,
    }];
    mineshaft_add_children(0, start_box, &mut pieces, &mut random);
    let aggregate = pieces
        .iter()
        .map(MineshaftGeneratedPieceModel::bounding_box)
        .reduce(StructureBoundingBoxModel::union)
        .unwrap_or(room.bounding_box);
    let random_roll = {
        let max_y = sea_level - 10;
        let y1_pos = (aggregate.max_y - aggregate.min_y + 1) + min_y + 1;
        if y1_pos < max_y {
            random_next_i32_bound(&mut random, max_y - y1_pos)
        } else {
            0
        }
    };
    let dy = mineshaft_move_below_sea_level_dy(aggregate, sea_level, min_y, 10, random_roll)
        .expect("computed mineshaft sea-level move roll should be valid");
    mineshaft_move_pieces(&mut pieces, dy);
    pieces
}

