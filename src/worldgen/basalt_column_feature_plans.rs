use super::*;

pub fn basalt_columns_cannot_place_on(state: &str) -> bool {
    matches!(
        block_state_id(state),
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
    block_is_air(state) || (block_state_id(state) == "minecraft:lava" && y <= lava_sea_level)
}

pub fn basalt_columns_can_place_at(
    state: &str,
    below_state: &str,
    y: i32,
    lava_sea_level: i32,
) -> bool {
    basalt_columns_is_air_or_lava_ocean(state, y, lava_sea_level)
        && !block_is_air(below_state)
        && !basalt_columns_cannot_place_on(below_state)
}

/// Model Java's downward `findSurface` scan over explicitly sampled block states.
///
/// Each `(y, state)` pair describes one block in the same X/Z column. Samples
/// must cover every queried height: Java's world always supplies those states,
/// so an incomplete snapshot cannot establish a surface and returns `None`.
/// The Java loop only examines positions strictly above `min_y + 1`, and a
/// zero or negative limit performs no queries.
pub fn basalt_columns_find_surface_y(
    start_y: i32,
    min_y: i32,
    limit: i32,
    states: &[(i32, &str)],
    lava_sea_level: i32,
) -> Option<i32> {
    let mut y = start_y;
    let mut remaining = limit;
    while y > min_y + 1 && remaining > 0 {
        remaining -= 1;
        let current = basalt_column_state_at(states, y)?;
        let below = basalt_column_state_at(states, y - 1)?;
        if basalt_columns_can_place_at(current, below, y, lava_sea_level) {
            return Some(y);
        }
        y -= 1;
    }
    None
}

/// Model Java's upward `findAir` scan over explicit states in one X/Z column.
///
/// `max_y` is inclusive. A forbidden block ends the search immediately, even
/// if an air block exists farther above, matching the Java loop's check order.
/// Missing samples return `None` because the scan cannot safely be continued.
pub fn basalt_columns_find_air_y(
    start_y: i32,
    max_y: i32,
    limit: i32,
    states: &[(i32, &str)],
) -> Option<i32> {
    let mut y = start_y;
    let mut remaining = limit;
    while y <= max_y && remaining > 0 {
        remaining -= 1;
        let state = basalt_column_state_at(states, y)?;
        if basalt_columns_cannot_place_on(state) {
            return None;
        }
        if block_is_air(state) {
            return Some(y);
        }
        y += 1;
    }
    None
}

fn basalt_column_state_at<'a>(states: &'a [(i32, &'a str)], y: i32) -> Option<&'a str> {
    states
        .iter()
        .find_map(|(sample_y, state)| (*sample_y == y).then_some(*state))
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

/// Plan the upward writes after Java's `findSurface` / `findAir` search.
///
/// `origin` is the column-cluster center, and `surface_pos` retains the sampled
/// X/Z coordinate. Java measures taper from that sample at `origin.y`, before
/// the vertical search moves its cursor. The reach bounds a square, including
/// corners whose Manhattan distance exceeds reach.
pub fn basalt_column_blocks_from_surface(
    surface_pos: BlockPos,
    origin: BlockPos,
    column_height: i32,
    reach: i32,
    air_or_lava_ocean_above: &[bool],
    already_basalt_above: &[bool],
) -> Vec<BasaltColumnPlacementBlock> {
    let mut blocks = Vec::new();
    let x_distance = (surface_pos.x - origin.x).abs();
    let z_distance = (surface_pos.z - origin.z).abs();
    if x_distance > reach || z_distance > reach {
        return blocks;
    }
    let step_limit = x_distance + z_distance;
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

#[cfg(test)]
#[path = "basalt_column_feature_plan_regression_tests.rs"]
mod regression_tests;
