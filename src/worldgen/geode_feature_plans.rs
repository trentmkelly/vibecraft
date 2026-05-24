use super::*;

pub fn validate_geode_config(config: &GeodeConfigurationModel) -> Result<(), &'static str> {
    let layer_values = [
        config.layers.filling,
        config.layers.inner_layer,
        config.layers.middle_layer,
        config.layers.outer_layer,
    ];
    if layer_values
        .iter()
        .any(|layer| !(0.01..=50.0).contains(layer))
    {
        Err("geode layer values must be in 0.01..=50.0")
    } else if !(0.0..=1.0).contains(&config.crack.generate_crack_chance)
        || !(0.0..=1.0).contains(&config.use_potential_placements_chance)
        || !(0.0..=1.0).contains(&config.use_alternate_layer0_chance)
    {
        Err("geode chances must be in 0.0..=1.0")
    } else if !(0.0..=5.0).contains(&config.crack.base_crack_size) {
        Err("geode crack base size must be in 0.0..=5.0")
    } else if !(0..=10).contains(&config.crack.crack_point_offset) {
        Err("geode crack point offset must be in 0..=10")
    } else if !(1..=20).contains(&config.outer_wall_distance_max) {
        Err("geode outer wall distance max must be in 1..=20")
    } else if config.inner_placements.is_empty() {
        Err("geode inner placements need at least one state")
    } else {
        Ok(())
    }
}

pub fn geode_invalid_point_count(config: &GeodeConfigurationModel, sampled_states: &[&str]) -> i32 {
    sampled_states
        .iter()
        .filter(|state| **state == "minecraft:air" || config.invalid_blocks.contains(state))
        .count() as i32
}

pub fn geode_can_place(config: &GeodeConfigurationModel, sampled_states: &[&str]) -> bool {
    geode_invalid_point_count(config, sampled_states) <= config.invalid_blocks_threshold
}

pub fn geode_layer_thresholds(
    layers: GeodeLayerSettingsModel,
    crack: GeodeCrackSettingsModel,
    num_points: i32,
    outer_wall_distance_max: i32,
    crack_roll: f64,
) -> GeodeLayerThresholds {
    let crack_size_adjustment = f64::from(num_points) / f64::from(outer_wall_distance_max.max(1));
    GeodeLayerThresholds {
        inner_air: 1.0 / layers.filling.sqrt(),
        innermost_block_layer: 1.0 / (layers.inner_layer + crack_size_adjustment).sqrt(),
        inner_crust: 1.0 / (layers.middle_layer + crack_size_adjustment).sqrt(),
        outer_crust: 1.0 / (layers.outer_layer + crack_size_adjustment).sqrt(),
        crack_size: 1.0
            / (crack.base_crack_size
                + crack_roll / 2.0
                + if num_points > 3 {
                    crack_size_adjustment
                } else {
                    0.0
                })
            .sqrt(),
    }
}

pub fn geode_should_generate_crack(crack: GeodeCrackSettingsModel, roll: f32) -> bool {
    roll < crack.generate_crack_chance
}

pub fn geode_crack_points(origin: BlockPos, num_points: i32, offset_roll: i32) -> Vec<BlockPos> {
    let crack_offset = num_points * 2 + 1;
    let offsets = match offset_roll.rem_euclid(4) {
        0 => [
            (crack_offset, 7, 0),
            (crack_offset, 5, 0),
            (crack_offset, 1, 0),
        ],
        1 => [
            (0, 7, crack_offset),
            (0, 5, crack_offset),
            (0, 1, crack_offset),
        ],
        2 => [
            (crack_offset, 7, crack_offset),
            (crack_offset, 5, crack_offset),
            (crack_offset, 1, crack_offset),
        ],
        _ => [(0, 7, 0), (0, 5, 0), (0, 1, 0)],
    };
    offsets
        .into_iter()
        .map(|(x, y, z)| BlockPos {
            x: origin.x + x,
            y: origin.y + y,
            z: origin.z + z,
        })
        .collect()
}

fn geode_distance_sqr(a: BlockPos, b: BlockPos) -> f64 {
    let dx = f64::from(a.x - b.x);
    let dy = f64::from(a.y - b.y);
    let dz = f64::from(a.z - b.z);
    dx * dx + dy * dy + dz * dz
}

pub fn geode_shell_density(
    pos: BlockPos,
    points: &[GeodeDistributionPoint],
    noise_offset: f64,
) -> f64 {
    points
        .iter()
        .map(|point| 1.0 / (geode_distance_sqr(pos, point.pos) + f64::from(point.offset)).sqrt())
        .sum::<f64>()
        + noise_offset * points.len() as f64
}

pub fn geode_crack_density(
    pos: BlockPos,
    crack_points: &[BlockPos],
    crack_point_offset: i32,
    noise_offset: f64,
) -> f64 {
    crack_points
        .iter()
        .map(|point| 1.0 / (geode_distance_sqr(pos, *point) + f64::from(crack_point_offset)).sqrt())
        .sum::<f64>()
        + noise_offset * crack_points.len() as f64
}

pub fn geode_layer_for_density(
    shell_density: f64,
    crack_density: f64,
    thresholds: GeodeLayerThresholds,
    should_generate_crack: bool,
    alternate_inner_roll: f32,
    use_alternate_layer0_chance: f32,
) -> Option<GeodeLayer> {
    if shell_density < thresholds.outer_crust {
        None
    } else if should_generate_crack
        && crack_density >= thresholds.crack_size
        && shell_density < thresholds.inner_air
    {
        Some(GeodeLayer::CrackAir)
    } else if shell_density >= thresholds.inner_air {
        Some(GeodeLayer::Filling)
    } else if shell_density >= thresholds.innermost_block_layer {
        if alternate_inner_roll < use_alternate_layer0_chance {
            Some(GeodeLayer::AlternateInner)
        } else {
            Some(GeodeLayer::Inner)
        }
    } else if shell_density >= thresholds.inner_crust {
        Some(GeodeLayer::Middle)
    } else {
        Some(GeodeLayer::Outer)
    }
}

pub fn geode_placement_block(
    config: &GeodeConfigurationModel,
    pos: BlockPos,
    layer: GeodeLayer,
    provider_roll: i32,
    potential_roll: f32,
) -> Option<GeodePlacementBlock> {
    let state = match layer {
        GeodeLayer::CrackAir => "minecraft:air",
        GeodeLayer::Filling => {
            block_state_provider_sample(&config.filling_provider, provider_roll)?
        }
        GeodeLayer::Inner => {
            block_state_provider_sample(&config.inner_layer_provider, provider_roll)?
        }
        GeodeLayer::AlternateInner => {
            block_state_provider_sample(&config.alternate_inner_layer_provider, provider_roll)?
        }
        GeodeLayer::Middle => {
            block_state_provider_sample(&config.middle_layer_provider, provider_roll)?
        }
        GeodeLayer::Outer => {
            block_state_provider_sample(&config.outer_layer_provider, provider_roll)?
        }
    };
    let potential_crystal_source = matches!(layer, GeodeLayer::Inner | GeodeLayer::AlternateInner)
        && (!config.placements_require_layer0_alternate || layer == GeodeLayer::AlternateInner)
        && potential_roll < config.use_potential_placements_chance;
    Some(GeodePlacementBlock {
        pos,
        state,
        layer,
        potential_crystal_source,
    })
}

pub fn geode_inner_placement(config: &GeodeConfigurationModel, roll: i32) -> Option<&'static str> {
    if config.inner_placements.is_empty() {
        None
    } else {
        Some(
            config.inner_placements[roll.rem_euclid(config.inner_placements.len() as i32) as usize],
        )
    }
}
