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

#[derive(Clone, Copy)]
struct MineshaftPieceRequest {
    foot: BlockPos,
    direction: HorizontalDirection,
    gen_depth: i32,
    mineshaft_type: MineshaftTypeModel,
}

#[derive(Clone, Copy)]
enum MineshaftRoomExitSide {
    North,
    South,
    West,
    East,
}

struct MineshaftChildGenerator<'a, 'b> {
    start_box: StructureBoundingBoxModel,
    pieces: &'a mut Vec<MineshaftGeneratedPieceModel>,
    random: &'b mut RandomSourceKind,
}

fn mineshaft_create_random_piece(
    random: &mut RandomSourceKind,
    request: MineshaftPieceRequest,
    existing_pieces: &[MineshaftGeneratedPieceModel],
) -> Option<MineshaftGeneratedPieceModel> {
    let existing_boxes = mineshaft_piece_bounding_boxes(existing_pieces);
    match mineshaft_random_piece_kind(random_next_i32_bound(random, 100)).ok()? {
        MineshaftPieceKindModel::Crossing => {
            let bounding_box = mineshaft_find_crossing(
                request.foot.x,
                request.foot.y,
                request.foot.z,
                request.direction,
                random_next_i32_bound(random, 4),
                &existing_boxes,
            )
            .ok()
            .flatten()?;
            Some(MineshaftGeneratedPieceModel::Crossing {
                bounding_box,
                direction: request.direction,
                mineshaft_type: request.mineshaft_type,
                is_two_floored: bounding_box.max_y - bounding_box.min_y + 1 > 3,
                gen_depth: request.gen_depth,
            })
        }
        MineshaftPieceKindModel::Stairs => {
            let bounding_box = mineshaft_find_stairs(
                request.foot.x,
                request.foot.y,
                request.foot.z,
                request.direction,
                &existing_boxes,
            )?;
            Some(MineshaftGeneratedPieceModel::Stairs {
                bounding_box,
                direction: request.direction,
                mineshaft_type: request.mineshaft_type,
                gen_depth: request.gen_depth,
            })
        }
        MineshaftPieceKindModel::Corridor => {
            let bounding_box = mineshaft_find_corridor_size(
                request.foot.x,
                request.foot.y,
                request.foot.z,
                request.direction,
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
                request.direction,
                request.mineshaft_type,
                rails_roll,
                spider_roll,
            )
            .ok()?;
            Some(MineshaftGeneratedPieceModel::Corridor {
                model,
                gen_depth: request.gen_depth,
            })
        }
    }
}

impl<'a, 'b> MineshaftChildGenerator<'a, 'b> {
    fn generate_and_add_piece(
        &mut self,
        foot: BlockPos,
        direction: HorizontalDirection,
        depth: i32,
    ) -> Option<StructureBoundingBoxModel> {
        if depth > 8
            || (foot.x - self.start_box.min_x).abs() > 80
            || (foot.z - self.start_box.min_z).abs() > 80
        {
            return None;
        }
        let mineshaft_type = self
            .pieces
            .first()
            .map(mineshaft_generated_piece_type)
            .unwrap_or(MineshaftTypeModel::Normal);
        let request = MineshaftPieceRequest {
            foot,
            direction,
            gen_depth: depth + 1,
            mineshaft_type,
        };
        let piece = mineshaft_create_random_piece(self.random, request, self.pieces)?;
        let bounding_box = piece.bounding_box();
        self.pieces.push(piece);
        self.add_children(self.pieces.len() - 1);
        Some(bounding_box)
    }

    fn add_children(&mut self, piece_index: usize) {
        match self.pieces[piece_index].clone() {
            MineshaftGeneratedPieceModel::Room {
                bounding_box,
                gen_depth,
                ..
            } => self.add_room_children(piece_index, bounding_box, gen_depth),
            MineshaftGeneratedPieceModel::Corridor { model, gen_depth } => {
                self.add_corridor_children(model, gen_depth);
            }
            MineshaftGeneratedPieceModel::Crossing {
                bounding_box,
                direction,
                is_two_floored,
                gen_depth,
                ..
            } => self.add_crossing_children(bounding_box, direction, is_two_floored, gen_depth),
            MineshaftGeneratedPieceModel::Stairs {
                bounding_box,
                direction,
                gen_depth,
                ..
            } => self.add_stairs_child(bounding_box, direction, gen_depth),
        }
    }

    fn add_room_children(
        &mut self,
        piece_index: usize,
        bounding_box: StructureBoundingBoxModel,
        gen_depth: i32,
    ) {
        let height_space = (bounding_box.max_y - bounding_box.min_y + 1 - 3 - 1).max(1);
        let mut entrance_boxes = Vec::new();
        self.add_room_exits_along_x(
            bounding_box,
            gen_depth,
            height_space,
            MineshaftRoomExitSide::North,
            &mut entrance_boxes,
        );
        self.add_room_exits_along_x(
            bounding_box,
            gen_depth,
            height_space,
            MineshaftRoomExitSide::South,
            &mut entrance_boxes,
        );
        self.add_room_exits_along_z(
            bounding_box,
            gen_depth,
            height_space,
            MineshaftRoomExitSide::West,
            &mut entrance_boxes,
        );
        self.add_room_exits_along_z(
            bounding_box,
            gen_depth,
            height_space,
            MineshaftRoomExitSide::East,
            &mut entrance_boxes,
        );
        if let MineshaftGeneratedPieceModel::Room {
            child_entrance_boxes,
            ..
        } = &mut self.pieces[piece_index]
        {
            *child_entrance_boxes = entrance_boxes;
        }
    }

    fn add_room_exits_along_x(
        &mut self,
        room_box: StructureBoundingBoxModel,
        gen_depth: i32,
        height_space: i32,
        side: MineshaftRoomExitSide,
        entrance_boxes: &mut Vec<StructureBoundingBoxModel>,
    ) {
        let span = room_box.max_x - room_box.min_x + 1;
        let mut pos = 0;
        while pos < span {
            pos += random_next_i32_bound(self.random, span);
            if pos + 3 > span {
                break;
            }
            let foot_y = room_box.min_y + random_next_i32_bound(self.random, height_space) + 1;
            let foot_z = match side {
                MineshaftRoomExitSide::North => room_box.min_z - 1,
                MineshaftRoomExitSide::South => room_box.max_z + 1,
                MineshaftRoomExitSide::West | MineshaftRoomExitSide::East => unreachable!(),
            };
            if let Some(child_box) = self.generate_and_add_piece(
                BlockPos {
                    x: room_box.min_x + pos,
                    y: foot_y,
                    z: foot_z,
                },
                room_exit_direction(side),
                gen_depth,
            ) {
                entrance_boxes.push(room_x_entrance_box(side, room_box, child_box));
            }
            pos += 4;
        }
    }

    fn add_room_exits_along_z(
        &mut self,
        room_box: StructureBoundingBoxModel,
        gen_depth: i32,
        height_space: i32,
        side: MineshaftRoomExitSide,
        entrance_boxes: &mut Vec<StructureBoundingBoxModel>,
    ) {
        let span = room_box.max_z - room_box.min_z + 1;
        let mut pos = 0;
        while pos < span {
            pos += random_next_i32_bound(self.random, span);
            if pos + 3 > span {
                break;
            }
            let foot_y = room_box.min_y + random_next_i32_bound(self.random, height_space) + 1;
            let foot_x = match side {
                MineshaftRoomExitSide::West => room_box.min_x - 1,
                MineshaftRoomExitSide::East => room_box.max_x + 1,
                MineshaftRoomExitSide::North | MineshaftRoomExitSide::South => unreachable!(),
            };
            if let Some(child_box) = self.generate_and_add_piece(
                BlockPos {
                    x: foot_x,
                    y: foot_y,
                    z: room_box.min_z + pos,
                },
                room_exit_direction(side),
                gen_depth,
            ) {
                entrance_boxes.push(room_z_entrance_box(side, room_box, child_box));
            }
            pos += 4;
        }
    }

    fn add_corridor_children(&mut self, model: MineshaftCorridorModel, gen_depth: i32) {
        self.add_corridor_end_child(model, gen_depth);
        if gen_depth < 8 {
            self.add_corridor_side_children(model, gen_depth);
        }
    }

    fn add_corridor_end_child(&mut self, model: MineshaftCorridorModel, gen_depth: i32) {
        let end_selection = random_next_i32_bound(self.random, 4);
        let foot_y = model.bounding_box.min_y - 1 + random_next_i32_bound(self.random, 3);
        let (x, z, direction) = corridor_end_child(model, end_selection);
        self.generate_and_add_piece(BlockPos { x, y: foot_y, z }, direction, gen_depth);
    }

    fn add_corridor_side_children(&mut self, model: MineshaftCorridorModel, gen_depth: i32) {
        if matches!(
            model.orientation,
            HorizontalDirection::North | HorizontalDirection::South
        ) {
            self.add_corridor_side_children_along_z(model, gen_depth);
        } else {
            self.add_corridor_side_children_along_x(model, gen_depth);
        }
    }

    fn add_corridor_side_children_along_x(
        &mut self,
        model: MineshaftCorridorModel,
        gen_depth: i32,
    ) {
        let mut x = model.bounding_box.min_x + 3;
        while x + 3 <= model.bounding_box.max_x {
            match random_next_i32_bound(self.random, 5) {
                0 => {
                    self.generate_and_add_piece(
                        BlockPos {
                            x,
                            y: model.bounding_box.min_y,
                            z: model.bounding_box.min_z - 1,
                        },
                        HorizontalDirection::North,
                        gen_depth + 1,
                    );
                }
                1 => {
                    self.generate_and_add_piece(
                        BlockPos {
                            x,
                            y: model.bounding_box.min_y,
                            z: model.bounding_box.max_z + 1,
                        },
                        HorizontalDirection::South,
                        gen_depth + 1,
                    );
                }
                _ => {}
            }
            x += 5;
        }
    }

    fn add_corridor_side_children_along_z(
        &mut self,
        model: MineshaftCorridorModel,
        gen_depth: i32,
    ) {
        let mut z = model.bounding_box.min_z + 3;
        while z + 3 <= model.bounding_box.max_z {
            match random_next_i32_bound(self.random, 5) {
                0 => {
                    self.generate_and_add_piece(
                        BlockPos {
                            x: model.bounding_box.min_x - 1,
                            y: model.bounding_box.min_y,
                            z,
                        },
                        HorizontalDirection::West,
                        gen_depth + 1,
                    );
                }
                1 => {
                    self.generate_and_add_piece(
                        BlockPos {
                            x: model.bounding_box.max_x + 1,
                            y: model.bounding_box.min_y,
                            z,
                        },
                        HorizontalDirection::East,
                        gen_depth + 1,
                    );
                }
                _ => {}
            }
            z += 5;
        }
    }

    fn add_crossing_children(
        &mut self,
        bounding_box: StructureBoundingBoxModel,
        direction: HorizontalDirection,
        is_two_floored: bool,
        gen_depth: i32,
    ) {
        for (foot, direction) in crossing_base_children(bounding_box, direction) {
            self.generate_and_add_piece(foot, direction, gen_depth);
        }
        if is_two_floored {
            self.add_crossing_upper_children(bounding_box, gen_depth);
        }
    }

    fn add_crossing_upper_children(
        &mut self,
        bounding_box: StructureBoundingBoxModel,
        gen_depth: i32,
    ) {
        for (foot, direction) in crossing_upper_children(bounding_box) {
            if crate::random_source::random_source_next_bool(self.random) {
                self.generate_and_add_piece(foot, direction, gen_depth);
            }
        }
    }

    fn add_stairs_child(
        &mut self,
        bounding_box: StructureBoundingBoxModel,
        direction: HorizontalDirection,
        gen_depth: i32,
    ) {
        let foot = match direction {
            HorizontalDirection::North => BlockPos {
                x: bounding_box.min_x,
                y: bounding_box.min_y,
                z: bounding_box.min_z - 1,
            },
            HorizontalDirection::South => BlockPos {
                x: bounding_box.min_x,
                y: bounding_box.min_y,
                z: bounding_box.max_z + 1,
            },
            HorizontalDirection::West => BlockPos {
                x: bounding_box.min_x - 1,
                y: bounding_box.min_y,
                z: bounding_box.min_z,
            },
            HorizontalDirection::East => BlockPos {
                x: bounding_box.max_x + 1,
                y: bounding_box.min_y,
                z: bounding_box.min_z,
            },
        };
        self.generate_and_add_piece(foot, direction, gen_depth);
    }
}

fn room_exit_direction(side: MineshaftRoomExitSide) -> HorizontalDirection {
    match side {
        MineshaftRoomExitSide::North => HorizontalDirection::North,
        MineshaftRoomExitSide::South => HorizontalDirection::South,
        MineshaftRoomExitSide::West => HorizontalDirection::West,
        MineshaftRoomExitSide::East => HorizontalDirection::East,
    }
}

fn room_x_entrance_box(
    side: MineshaftRoomExitSide,
    room_box: StructureBoundingBoxModel,
    child_box: StructureBoundingBoxModel,
) -> StructureBoundingBoxModel {
    let (min_z, max_z) = match side {
        MineshaftRoomExitSide::North => (room_box.min_z, room_box.min_z + 1),
        MineshaftRoomExitSide::South => (room_box.max_z - 1, room_box.max_z),
        MineshaftRoomExitSide::West | MineshaftRoomExitSide::East => unreachable!(),
    };
    StructureBoundingBoxModel {
        min_x: child_box.min_x,
        min_y: child_box.min_y,
        min_z,
        max_x: child_box.max_x,
        max_y: child_box.max_y,
        max_z,
    }
}

fn room_z_entrance_box(
    side: MineshaftRoomExitSide,
    room_box: StructureBoundingBoxModel,
    child_box: StructureBoundingBoxModel,
) -> StructureBoundingBoxModel {
    let (min_x, max_x) = match side {
        MineshaftRoomExitSide::West => (room_box.min_x, room_box.min_x + 1),
        MineshaftRoomExitSide::East => (room_box.max_x - 1, room_box.max_x),
        MineshaftRoomExitSide::North | MineshaftRoomExitSide::South => unreachable!(),
    };
    StructureBoundingBoxModel {
        min_x,
        min_y: child_box.min_y,
        min_z: child_box.min_z,
        max_x,
        max_y: child_box.max_y,
        max_z: child_box.max_z,
    }
}

fn corridor_end_child(
    model: MineshaftCorridorModel,
    end_selection: i32,
) -> (i32, i32, HorizontalDirection) {
    let box_ = model.bounding_box;
    match (model.orientation, end_selection) {
        (HorizontalDirection::North, 0 | 1) => (box_.min_x, box_.min_z - 1, model.orientation),
        (HorizontalDirection::North, 2) => (box_.min_x - 1, box_.min_z, HorizontalDirection::West),
        (HorizontalDirection::North, _) => (box_.max_x + 1, box_.min_z, HorizontalDirection::East),
        (HorizontalDirection::South, 0 | 1) => (box_.min_x, box_.max_z + 1, model.orientation),
        (HorizontalDirection::South, 2) => {
            (box_.min_x - 1, box_.max_z - 3, HorizontalDirection::West)
        }
        (HorizontalDirection::South, _) => {
            (box_.max_x + 1, box_.max_z - 3, HorizontalDirection::East)
        }
        (HorizontalDirection::West, 0 | 1) => (box_.min_x - 1, box_.min_z, model.orientation),
        (HorizontalDirection::West, 2) => (box_.min_x, box_.min_z - 1, HorizontalDirection::North),
        (HorizontalDirection::West, _) => (box_.min_x, box_.max_z + 1, HorizontalDirection::South),
        (HorizontalDirection::East, 0 | 1) => (box_.max_x + 1, box_.min_z, model.orientation),
        (HorizontalDirection::East, 2) => {
            (box_.max_x - 3, box_.min_z - 1, HorizontalDirection::North)
        }
        (HorizontalDirection::East, _) => {
            (box_.max_x - 3, box_.max_z + 1, HorizontalDirection::South)
        }
    }
}

fn crossing_base_children(
    bounding_box: StructureBoundingBoxModel,
    direction: HorizontalDirection,
) -> [(BlockPos, HorizontalDirection); 3] {
    let box_ = bounding_box;
    match direction {
        HorizontalDirection::North => [
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.min_z - 1,
                HorizontalDirection::North,
            ),
            mineshaft_child(
                box_.min_x - 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::West,
            ),
            mineshaft_child(
                box_.max_x + 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::East,
            ),
        ],
        HorizontalDirection::South => [
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.max_z + 1,
                HorizontalDirection::South,
            ),
            mineshaft_child(
                box_.min_x - 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::West,
            ),
            mineshaft_child(
                box_.max_x + 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::East,
            ),
        ],
        HorizontalDirection::West => [
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.min_z - 1,
                HorizontalDirection::North,
            ),
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.max_z + 1,
                HorizontalDirection::South,
            ),
            mineshaft_child(
                box_.min_x - 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::West,
            ),
        ],
        HorizontalDirection::East => [
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.min_z - 1,
                HorizontalDirection::North,
            ),
            mineshaft_child(
                box_.min_x + 1,
                box_.min_y,
                box_.max_z + 1,
                HorizontalDirection::South,
            ),
            mineshaft_child(
                box_.max_x + 1,
                box_.min_y,
                box_.min_z + 1,
                HorizontalDirection::East,
            ),
        ],
    }
}

fn crossing_upper_children(
    bounding_box: StructureBoundingBoxModel,
) -> [(BlockPos, HorizontalDirection); 4] {
    let box_ = bounding_box;
    [
        mineshaft_child(
            box_.min_x + 1,
            box_.min_y + 4,
            box_.min_z - 1,
            HorizontalDirection::North,
        ),
        mineshaft_child(
            box_.min_x - 1,
            box_.min_y + 4,
            box_.min_z + 1,
            HorizontalDirection::West,
        ),
        mineshaft_child(
            box_.max_x + 1,
            box_.min_y + 4,
            box_.min_z + 1,
            HorizontalDirection::East,
        ),
        mineshaft_child(
            box_.min_x + 1,
            box_.min_y + 4,
            box_.max_z + 1,
            HorizontalDirection::South,
        ),
    ]
}

const fn mineshaft_child(
    x: i32,
    y: i32,
    z: i32,
    direction: HorizontalDirection,
) -> (BlockPos, HorizontalDirection) {
    (BlockPos { x, y, z }, direction)
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
    let Ok(room) = mineshaft_room(
        chunk_pos,
        mineshaft_type,
        random_next_i32_bound(&mut random, 6),
        random_next_i32_bound(&mut random, 6),
        random_next_i32_bound(&mut random, 6),
    ) else {
        panic!("RandomSource#nextInt(6) room rolls should be valid");
    };
    let start_box = room.bounding_box;
    let mut pieces = vec![MineshaftGeneratedPieceModel::Room {
        bounding_box: room.bounding_box,
        mineshaft_type,
        child_entrance_boxes: Vec::new(),
        gen_depth: 0,
    }];
    MineshaftChildGenerator {
        start_box,
        pieces: &mut pieces,
        random: &mut random,
    }
    .add_children(0);
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
    let Ok(dy) = mineshaft_move_below_sea_level_dy(aggregate, sea_level, min_y, 10, random_roll)
    else {
        panic!("computed mineshaft sea-level move roll should be valid");
    };
    mineshaft_move_pieces(&mut pieces, dy);
    pieces
}
