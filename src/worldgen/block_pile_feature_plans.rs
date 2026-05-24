use super::*;

pub fn block_pile_placement_candidates(
    origin: BlockPos,
    min_y: i32,
    x_radius_roll: i32,
    z_radius_roll: i32,
    shape_rolls: &[(f32, f32, f32)],
) -> Vec<BlockPos> {
    if origin.y < min_y + 5 {
        return Vec::new();
    }
    let x_radius = 2 + x_radius_roll.rem_euclid(2);
    let z_radius = 2 + z_radius_roll.rem_euclid(2);
    let mut positions = Vec::new();
    let mut roll_index = 0;
    for y_offset in 0..=1 {
        for z in origin.z - z_radius..=origin.z + z_radius {
            for x in origin.x - x_radius..=origin.x + x_radius {
                let (first, second, sparse) = shape_rolls
                    .get(roll_index)
                    .copied()
                    .unwrap_or((0.0, 0.0, 1.0));
                roll_index += 1;
                let dx = origin.x - x;
                let dz = origin.z - z;
                let in_blob = (dx * dx + dz * dz) as f32 <= first * 10.0 - second * 6.0;
                if in_blob || sparse < 0.031 {
                    positions.push(BlockPos {
                        x,
                        y: origin.y + y_offset,
                        z,
                    });
                }
            }
        }
    }
    positions
}

pub fn block_pile_try_place(
    config: &BlockPileConfigurationModel,
    candidate_empty: bool,
    below_block: &'static str,
    below_sturdy: bool,
    dirt_path_random: bool,
    provider_roll: i32,
) -> Option<&'static str> {
    if !candidate_empty {
        return None;
    }
    let may_place = if below_block == "minecraft:dirt_path" {
        dirt_path_random
    } else {
        below_sturdy
    };
    if may_place {
        block_state_provider_sample(&config.state_provider, provider_roll)
    } else {
        None
    }
}
