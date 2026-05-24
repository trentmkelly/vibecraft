use super::*;

pub fn end_island_layer_radius(size: f32) -> i32 {
    size.ceil() as i32
}

pub fn end_island_next_size(size: f32, shrink_roll: i32) -> f32 {
    size - (shrink_roll.rem_euclid(2) as f32 + 0.5)
}

pub fn end_island_placement_plan(
    origin: BlockPos,
    size_roll: i32,
    shrink_rolls: &[i32],
) -> Vec<EndIslandPlacementBlock> {
    let mut blocks = Vec::new();
    let mut size = size_roll.rem_euclid(3) as f32 + 4.0;
    let mut y_offset = 0;
    let mut shrink_index = 0;
    while size > 0.5 {
        let min = (-size).floor() as i32;
        let max = end_island_layer_radius(size);
        for x in min..=max {
            for z in min..=max {
                if (x * x + z * z) as f32 <= (size + 1.0) * (size + 1.0) {
                    blocks.push(EndIslandPlacementBlock {
                        pos: BlockPos {
                            x: origin.x + x,
                            y: origin.y + y_offset,
                            z: origin.z + z,
                        },
                        state: "minecraft:end_stone",
                    });
                }
            }
        }
        let shrink_roll = shrink_rolls.get(shrink_index).copied().unwrap_or(0);
        shrink_index += 1;
        size = end_island_next_size(size, shrink_roll);
        y_offset -= 1;
    }
    blocks
}
