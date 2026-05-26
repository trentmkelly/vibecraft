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

pub fn delta_is_clear(input: DeltaClearInput<'_>) -> bool {
    input.state != input.contents
        && !delta_cannot_replace(input.state)
        && !input.up_air
        && !input.down_air
        && !input.north_air
        && !input.south_air
        && !input.west_air
        && !input.east_air
}

pub fn delta_has_rim(spawn_roll: f64, rim_x: i32, rim_z: i32) -> bool {
    spawn_roll < 0.9 && rim_x != 0 && rim_z != 0
}

pub fn delta_candidate_offsets(radius_x: i32, radius_z: i32) -> Vec<(i32, i32)> {
    let radius_limit = radius_x.max(radius_z);
    let mut offsets = Vec::new();
    for dz in -radius_z..=radius_z {
        for dx in -radius_x..=radius_x {
            if dx.abs() + dz.abs() <= radius_limit {
                offsets.push((dx, dz));
            }
        }
    }
    offsets
}
