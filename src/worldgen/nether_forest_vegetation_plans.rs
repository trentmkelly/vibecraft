use super::*;

pub fn nether_forest_vegetation_can_start(
    below_state: &str,
    y: i32,
    min_y: i32,
    max_y: i32,
) -> bool {
    matches!(
        below_state,
        "minecraft:crimson_nylium" | "minecraft:warped_nylium"
    ) && y > min_y
        && y < max_y
}

pub fn validate_nether_forest_vegetation_config(
    config: &NetherForestVegetationConfigModel,
) -> Result<(), &'static str> {
    if config.spread_width <= 0 || config.spread_height <= 0 {
        Err("nether forest vegetation spread values must be positive")
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherForestVegetationOffsetInput {
    pub spread_width: i32,
    pub spread_height: i32,
    pub x_rolls: (i32, i32),
    pub y_rolls: (i32, i32),
    pub z_rolls: (i32, i32),
}

pub fn nether_forest_vegetation_offset(input: NetherForestVegetationOffsetInput) -> BlockPos {
    BlockPos {
        x: input.x_rolls.0.rem_euclid(input.spread_width)
            - input.x_rolls.1.rem_euclid(input.spread_width),
        y: input.y_rolls.0.rem_euclid(input.spread_height)
            - input.y_rolls.1.rem_euclid(input.spread_height),
        z: input.z_rolls.0.rem_euclid(input.spread_width)
            - input.z_rolls.1.rem_euclid(input.spread_width),
    }
}

pub fn nether_forest_vegetation_attempts(spread_width: i32) -> i32 {
    spread_width * spread_width
}
