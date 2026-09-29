use super::*;

pub fn validate_delta_config(
    config: DeltaFeatureConfigurationModel,
) -> Result<DeltaFeatureConfigurationModel, &'static str> {
    let valid_size = (0..=16).contains(&config.size_min)
        && (0..=16).contains(&config.size_max)
        && config.size_min <= config.size_max;
    let valid_rim = (0..=16).contains(&config.rim_size_min)
        && (0..=16).contains(&config.rim_size_max)
        && config.rim_size_min <= config.rim_size_max;
    if config.contents.is_empty() || config.rim.is_empty() {
        Err("delta contents and rim states must not be empty")
    } else if !valid_size {
        Err("delta size bounds must be ordered in 0..=16")
    } else if !valid_rim {
        Err("delta rim size bounds must be ordered in 0..=16")
    } else {
        Ok(config)
    }
}

pub fn delta_cannot_replace(state: &str) -> bool {
    matches!(
        state,
        "minecraft:bedrock"
            | "minecraft:nether_bricks"
            | "minecraft:nether_brick_fence"
            | "minecraft:nether_brick_stairs"
            | "minecraft:nether_wart"
            | "minecraft:chest"
            | "minecraft:spawner"
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaClearInput<'a> {
    pub state: &'a str,
    pub contents: &'a str,
    pub up_air: bool,
    pub down_air: bool,
    pub north_air: bool,
    pub south_air: bool,
    pub west_air: bool,
    pub east_air: bool,
}

/// Java `DeltaFeature.isClear`: for every direction, `isAir && d != UP || !isAir && d == UP`
/// rejects the position, i.e. the block above must be air while the down/horizontal neighbours
/// must all be non-air.
pub fn delta_is_clear(input: DeltaClearInput<'_>) -> bool {
    input.state != input.contents
        && !delta_cannot_replace(input.state)
        && input.up_air
        && !input.down_air
        && !input.north_air
        && !input.south_air
        && !input.west_air
        && !input.east_air
}

pub fn delta_has_rim(spawn_roll: f64, rim_x: i32, rim_z: i32) -> bool {
    spawn_roll < 0.9 && rim_x != 0 && rim_z != 0
}

/// Candidate `(dx, dz)` offsets in Java iteration order: `BlockPos.withinManhattan(origin, radiusX,
/// 0, radiusZ)` visits positions by increasing Manhattan depth (not raster order), emitting each
/// `+z` position followed by its `-z` mirror, and `DeltaFeature.place` `break`s at the first
/// position farther than `max(radiusX, radiusZ)`. Order matters because earlier placements change
/// what later `isClear` checks observe.
pub fn delta_candidate_offsets(radius_x: i32, radius_z: i32) -> Vec<(i32, i32)> {
    let radius_limit = radius_x.max(radius_z);
    let mut offsets = Vec::new();
    for depth in 0..=radius_limit.min(radius_x + radius_z) {
        let max_x = radius_x.min(depth);
        for dx in -max_x..=max_x {
            let dz = depth - dx.abs();
            if dz <= radius_z {
                offsets.push((dx, dz));
                if dz != 0 {
                    offsets.push((dx, -dz));
                }
            }
        }
    }
    offsets
}
