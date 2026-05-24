use super::*;

pub fn basalt_columns_cannot_place_on(state: &str) -> bool {
    matches!(
        state,
        "minecraft:lava"
            | "minecraft:bedrock"
            | "minecraft:magma_block"
            | "minecraft:soul_sand"
            | "minecraft:nether_bricks"
            | "minecraft:nether_brick_fence"
            | "minecraft:nether_brick_stairs"
            | "minecraft:nether_wart"
            | "minecraft:chest"
            | "minecraft:spawner"
    )
}

pub fn basalt_columns_is_air_or_lava_ocean(state: &str, y: i32, lava_sea_level: i32) -> bool {
    state == "minecraft:air" || (state == "minecraft:lava" && y <= lava_sea_level)
}

pub fn basalt_columns_can_place_at(
    state: &str,
    below_state: &str,
    y: i32,
    lava_sea_level: i32,
) -> bool {
    basalt_columns_is_air_or_lava_ocean(state, y, lava_sea_level)
        && below_state != "minecraft:air"
        && !basalt_columns_cannot_place_on(below_state)
}

pub fn validate_column_feature_config(
    config: ColumnFeatureConfigurationModel,
) -> Result<ColumnFeatureConfigurationModel, &'static str> {
    if !(0..=3).contains(&config.reach_min)
        || !(0..=3).contains(&config.reach_max)
        || config.reach_min > config.reach_max
    {
        Err("column reach bounds must be ordered in 0..=3")
    } else if !(1..=10).contains(&config.height_min)
        || !(1..=10).contains(&config.height_max)
        || config.height_min > config.height_max
    {
        Err("column height bounds must be ordered in 1..=10")
    } else {
        Ok(config)
    }
}

pub fn basalt_columns_cluster_parameters(
    column_height: i32,
    clustered_roll: f32,
) -> (bool, i32, i32) {
    let clustered = clustered_roll < 0.9;
    let reach = column_height.min(if clustered { 5 } else { 8 });
    let count = if clustered { 50 } else { 15 };
    (clustered, reach, count)
}

pub fn basalt_column_blocks_from_surface(
    surface_pos: BlockPos,
    origin: BlockPos,
    column_height: i32,
    reach: i32,
    air_or_lava_ocean_above: &[bool],
    already_basalt_above: &[bool],
) -> Vec<BasaltColumnPlacementBlock> {
    let mut blocks = Vec::new();
    let step_limit = (surface_pos.x - origin.x).abs()
        + (surface_pos.y - origin.y).abs()
        + (surface_pos.z - origin.z).abs();
    if step_limit > reach {
        return blocks;
    }
    let mut blocks_y = column_height - step_limit / 2;
    let mut step = 0usize;
    while blocks_y >= 0 {
        if air_or_lava_ocean_above.get(step).copied().unwrap_or(false) {
            blocks.push(BasaltColumnPlacementBlock {
                pos: BlockPos {
                    x: surface_pos.x,
                    y: surface_pos.y + step as i32,
                    z: surface_pos.z,
                },
            });
        } else if !already_basalt_above.get(step).copied().unwrap_or(false) {
            break;
        }
        blocks_y -= 1;
        step += 1;
    }
    blocks
}
