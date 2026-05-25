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

pub fn nether_forest_vegetation_offset(
    spread_width: i32,
    spread_height: i32,
    x_a: i32,
    x_b: i32,
    y_a: i32,
    y_b: i32,
    z_a: i32,
    z_b: i32,
) -> BlockPos {
    BlockPos {
        x: x_a.rem_euclid(spread_width) - x_b.rem_euclid(spread_width),
        y: y_a.rem_euclid(spread_height) - y_b.rem_euclid(spread_height),
        z: z_a.rem_euclid(spread_width) - z_b.rem_euclid(spread_width),
    }
}

pub fn nether_forest_vegetation_attempts(spread_width: i32) -> i32 {
    spread_width * spread_width
}
