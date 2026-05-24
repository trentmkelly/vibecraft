use super::*;

pub fn validate_replace_sphere_config(
    config: ReplaceSphereConfigurationModel,
) -> Result<ReplaceSphereConfigurationModel, &'static str> {
    if config.target_state.is_empty() || config.replace_state.is_empty() {
        Err("replace sphere target and replacement states must not be empty")
    } else if !(0..=12).contains(&config.radius_min)
        || !(0..=12).contains(&config.radius_max)
        || config.radius_min > config.radius_max
    {
        Err("replace sphere radius bounds must be ordered in 0..=12")
    } else {
        Ok(config)
    }
}

pub fn replace_sphere_radius(config: ReplaceSphereConfigurationModel, roll: i32) -> i32 {
    let span = (config.radius_max - config.radius_min + 1).max(1);
    config.radius_min + roll.rem_euclid(span)
}

pub fn replace_sphere_find_target(
    origin: BlockPos,
    min_y: i32,
    max_y: i32,
    column_states: &[&str],
    target_state: &str,
) -> Option<BlockPos> {
    let mut y = origin.y.clamp(min_y + 1, max_y);
    while y > min_y + 1 {
        let offset = origin.y.clamp(min_y + 1, max_y) - y;
        if column_states.get(offset as usize).copied() == Some(target_state) {
            return Some(BlockPos {
                x: origin.x,
                y,
                z: origin.z,
            });
        }
        y -= 1;
    }
    None
}

pub fn replace_sphere_positions(
    center: BlockPos,
    radius_x: i32,
    radius_y: i32,
    radius_z: i32,
) -> Vec<BlockPos> {
    let maximum_radius = radius_x.max(radius_y).max(radius_z);
    let mut positions = Vec::new();
    for dy in -radius_y..=radius_y {
        for dz in -radius_z..=radius_z {
            for dx in -radius_x..=radius_x {
                if dx.abs() + dy.abs() + dz.abs() <= maximum_radius {
                    positions.push(BlockPos {
                        x: center.x + dx,
                        y: center.y + dy,
                        z: center.z + dz,
                    });
                }
            }
        }
    }
    positions
}
