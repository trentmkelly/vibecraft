use super::*;

pub fn aquatic_feature_offset(
    origin: BlockPos,
    x_rolls: (i32, i32),
    z_rolls: (i32, i32),
) -> (i32, i32) {
    (
        origin.x + x_rolls.0 - x_rolls.1,
        origin.z + z_rolls.0 - z_rolls.1,
    )
}

pub fn seagrass_placement_plan(
    pos: BlockPos,
    current_block: &'static str,
    above_block: &'static str,
    can_survive: bool,
    tall_probability: f64,
    tall_roll: f64,
) -> Vec<AquaticPlacementBlock> {
    if current_block != "minecraft:water" || !can_survive {
        return Vec::new();
    }
    if tall_roll < tall_probability {
        if above_block == "minecraft:water" {
            vec![
                AquaticPlacementBlock {
                    pos,
                    state: "minecraft:tall_seagrass",
                },
                AquaticPlacementBlock {
                    pos: BlockPos {
                        x: pos.x,
                        y: pos.y + 1,
                        z: pos.z,
                    },
                    state: "minecraft:tall_seagrass[half=upper]",
                },
            ]
        } else {
            Vec::new()
        }
    } else {
        vec![AquaticPlacementBlock {
            pos,
            state: "minecraft:seagrass",
        }]
    }
}

pub fn sea_pickle_placement_plan(
    pos: BlockPos,
    current_block: &'static str,
    can_survive: bool,
    pickle_roll: i32,
) -> Option<AquaticPlacementBlock> {
    if current_block == "minecraft:water" && can_survive {
        Some(AquaticPlacementBlock {
            pos,
            state: match pickle_roll.rem_euclid(4) + 1 {
                1 => "minecraft:sea_pickle[pickles=1]",
                2 => "minecraft:sea_pickle[pickles=2]",
                3 => "minecraft:sea_pickle[pickles=3]",
                _ => "minecraft:sea_pickle[pickles=4]",
            },
        })
    } else {
        None
    }
}

pub fn kelp_placement_plan(
    origin: BlockPos,
    water_column: &[bool],
    survival_column: &[bool],
    height_roll: i32,
    age_rolls: &[i32],
    below_is_kelp: bool,
) -> Vec<AquaticPlacementBlock> {
    if !water_column.first().copied().unwrap_or(false) {
        return Vec::new();
    }
    let height = 1 + height_roll.rem_euclid(10);
    let mut blocks = Vec::new();
    for h in 0..=height {
        let current_water = water_column.get(h as usize).copied().unwrap_or(false);
        let above_water = water_column.get(h as usize + 1).copied().unwrap_or(false);
        let can_survive = survival_column.get(h as usize).copied().unwrap_or(false);
        let pos = BlockPos {
            x: origin.x,
            y: origin.y + h,
            z: origin.z,
        };
        if current_water && above_water && can_survive {
            if h == height {
                let age = 20 + age_rolls.first().copied().unwrap_or(0).rem_euclid(4);
                blocks.push(AquaticPlacementBlock {
                    pos,
                    state: kelp_state_for_age(age),
                });
            } else {
                blocks.push(AquaticPlacementBlock {
                    pos,
                    state: "minecraft:kelp_plant",
                });
            }
        } else if h > 0 {
            let below_index = h as usize - 1;
            if survival_column.get(below_index).copied().unwrap_or(false) && !below_is_kelp {
                let age = 20 + age_rolls.first().copied().unwrap_or(0).rem_euclid(4);
                blocks.pop();
                blocks.push(AquaticPlacementBlock {
                    pos: BlockPos {
                        x: origin.x,
                        y: origin.y + h - 1,
                        z: origin.z,
                    },
                    state: kelp_state_for_age(age),
                });
            }
            break;
        }
    }
    blocks
}

const fn kelp_state_for_age(age: i32) -> &'static str {
    match age {
        20 => "minecraft:kelp[age=20]",
        21 => "minecraft:kelp[age=21]",
        22 => "minecraft:kelp[age=22]",
        _ => "minecraft:kelp[age=23]",
    }
}

pub fn coral_block_can_place(current_block: &'static str, above_block: &'static str) -> bool {
    (current_block == "minecraft:water" || block_is_coral(current_block))
        && above_block == "minecraft:water"
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoralBlockPlacementInput<'a> {
    pub pos: BlockPos,
    pub current_block: &'static str,
    pub above_block: &'static str,
    pub coral_state: &'static str,
    pub coral_roll: f32,
    pub sea_pickle_roll: f32,
    pub pickle_count_roll: i32,
    pub wall_fan_rolls: &'a [(HorizontalDirection, f32, bool)],
}

pub fn coral_block_placement_plan(
    input: CoralBlockPlacementInput<'_>,
) -> Vec<AquaticPlacementBlock> {
    if !coral_block_can_place(input.current_block, input.above_block) {
        return Vec::new();
    }
    let mut blocks = vec![AquaticPlacementBlock {
        pos: input.pos,
        state: input.coral_state,
    }];
    if input.coral_roll < 0.25 {
        blocks.push(AquaticPlacementBlock {
            pos: BlockPos {
                x: input.pos.x,
                y: input.pos.y + 1,
                z: input.pos.z,
            },
            state: "minecraft:tube_coral",
        });
    } else if input.sea_pickle_roll < 0.05 {
        blocks.push(AquaticPlacementBlock {
            pos: BlockPos {
                x: input.pos.x,
                y: input.pos.y + 1,
                z: input.pos.z,
            },
            state: match input.pickle_count_roll.rem_euclid(4) + 1 {
                1 => "minecraft:sea_pickle[pickles=1]",
                2 => "minecraft:sea_pickle[pickles=2]",
                3 => "minecraft:sea_pickle[pickles=3]",
                _ => "minecraft:sea_pickle[pickles=4]",
            },
        });
    }
    for (direction, roll, side_is_water) in input.wall_fan_rolls {
        if *roll < 0.2 && *side_is_water {
            blocks.push(AquaticPlacementBlock {
                pos: offset_horizontal(input.pos, *direction, 1),
                state: coral_wall_fan_state(*direction),
            });
        }
    }
    blocks
}

pub fn coral_tree_positions(
    origin: BlockPos,
    trunk_height_roll: i32,
    branch_directions: &[HorizontalDirection],
    branch_height_rolls: &[i32],
    branch_step_rolls: &[f32],
) -> Vec<BlockPos> {
    let trunk_height = trunk_height_roll.rem_euclid(3) + 1;
    let mut positions = Vec::new();
    for y in 0..trunk_height {
        positions.push(BlockPos {
            x: origin.x,
            y: origin.y + y,
            z: origin.z,
        });
    }
    let trunk_top = BlockPos {
        x: origin.x,
        y: origin.y + trunk_height,
        z: origin.z,
    };
    for (branch_index, direction) in branch_directions.iter().take(4).enumerate() {
        let mut pos = offset_horizontal(trunk_top, *direction, 1);
        let branch_height = branch_height_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(5)
            + 2;
        let mut segment_length = 0;
        for j in 0..branch_height {
            positions.push(pos);
            segment_length += 1;
            pos.y += 1;
            let roll = branch_step_rolls
                .get(branch_index * 5 + j as usize)
                .copied()
                .unwrap_or(1.0);
            if j == 0 || (segment_length >= 2 && roll < 0.25) {
                pos = offset_horizontal(pos, *direction, 1);
                segment_length = 0;
            }
        }
    }
    positions
}

pub fn coral_mushroom_positions(
    origin: BlockPos,
    height_roll: i32,
    width_roll: i32,
    length_roll: i32,
    sink_roll: i32,
    skip_rolls: &[f32],
) -> Vec<BlockPos> {
    let height = height_roll.rem_euclid(3) + 3;
    let width = width_roll.rem_euclid(3) + 3;
    let length = length_roll.rem_euclid(3) + 3;
    let sink = sink_roll.rem_euclid(3) + 1;
    let mut positions = Vec::new();
    let mut roll_index = 0;
    for x in 0..=width {
        for y in 0..=height {
            for z in 0..=length {
                let not_x_edge_or_y_edge = (x != 0 && x != width) || (y != 0 && y != height);
                let not_z_edge_or_y_edge = (z != 0 && z != length) || (y != 0 && y != height);
                let not_x_edge_or_z_edge = (x != 0 && x != width) || (z != 0 && z != length);
                let on_shell =
                    x == 0 || x == width || y == 0 || y == height || z == 0 || z == length;
                if not_x_edge_or_y_edge && not_z_edge_or_y_edge && not_x_edge_or_z_edge && on_shell
                {
                    let roll = skip_rolls.get(roll_index).copied().unwrap_or(1.0);
                    roll_index += 1;
                    if roll >= 0.1 {
                        positions.push(BlockPos {
                            x: origin.x + x,
                            y: origin.y + y - sink,
                            z: origin.z + z,
                        });
                    }
                }
            }
        }
    }
    positions
}

pub fn coral_claw_positions(
    origin: BlockPos,
    claw_direction: HorizontalDirection,
    branch_directions: &[HorizontalDirection],
    sideway_rolls: &[i32],
    inway_rolls: &[i32],
    up_rolls: &[f32],
) -> Vec<BlockPos> {
    let mut positions = vec![origin];
    for (branch_index, direction) in branch_directions.iter().take(3).enumerate() {
        let sideway_length = sideway_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(2)
            + 1;
        let inway_length = inway_rolls
            .get(branch_index)
            .copied()
            .unwrap_or(0)
            .rem_euclid(3)
            + if *direction == claw_direction { 2 } else { 3 };
        let mut pos = offset_horizontal(origin, *direction, 1);
        if *direction != claw_direction {
            pos.y += 1;
        }
        for _ in 0..sideway_length {
            positions.push(pos);
            pos = offset_horizontal(pos, *direction, 1);
        }
        pos = offset_horizontal(pos, direction.opposite(), 1);
        pos.y += 1;
        for i in 0..inway_length {
            pos = offset_horizontal(pos, claw_direction, 1);
            positions.push(pos);
            if up_rolls
                .get(branch_index * 5 + i as usize)
                .copied()
                .unwrap_or(1.0)
                < 0.25
            {
                pos.y += 1;
            }
        }
    }
    positions
}
