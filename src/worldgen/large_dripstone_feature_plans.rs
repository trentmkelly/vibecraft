use super::*;

pub fn validate_large_dripstone_sampled_config(
    config: LargeDripstoneSampledConfig,
) -> Result<LargeDripstoneSampledConfig, String> {
    if !(1..=512).contains(&config.floor_to_ceiling_search_range)
        || !(1..=60).contains(&config.column_radius_min)
        || !(1..=60).contains(&config.column_radius_max)
        || config.column_radius_min > config.column_radius_max
        || !(0.0..=20.0).contains(&config.height_scale)
        || !(0.1..=1.0).contains(&config.max_column_radius_to_cave_height_ratio)
        || !(0.1..=10.0).contains(&config.stalactite_bluntness)
        || !(0.1..=10.0).contains(&config.stalagmite_bluntness)
        || !(0.0..=2.0).contains(&config.wind_speed)
        || !(0.0..=std::f64::consts::PI).contains(&config.wind_direction_radians)
        || !(0..=100).contains(&config.min_radius_for_wind)
        || !(0.0..=5.0).contains(&config.min_bluntness_for_wind)
    {
        Err("large dripstone sampled fields are outside vanilla codec ranges".to_string())
    } else {
        Ok(config)
    }
}

pub fn large_dripstone_selected_radius(
    cave_height: i32,
    config: LargeDripstoneSampledConfig,
    radius_roll: i32,
) -> Option<i32> {
    if cave_height < 4 {
        return None;
    }
    let max_based_on_height =
        (cave_height as f32 * config.max_column_radius_to_cave_height_ratio) as i32;
    let max_radius = max_based_on_height.clamp(config.column_radius_min, config.column_radius_max);
    Some(inclusive_roll(
        radius_roll,
        config.column_radius_min,
        max_radius,
    ))
}

pub fn large_dripstone_height_at_radius(
    xz_distance_from_center: f64,
    dripstone_radius: i32,
    scale: f64,
    bluntness: f64,
) -> i32 {
    let xz_distance = xz_distance_from_center.max(bluntness);
    let cutoff = 0.384;
    let r = xz_distance / dripstone_radius as f64 * cutoff;
    let part1 = 0.75 * r.powf(4.0 / 3.0);
    let part2 = r.powf(2.0 / 3.0);
    let part3 = (1.0 / 3.0) * r.ln();
    let height_relative = (scale * (part1 - part2 - part3)).max(0.0);
    (height_relative / cutoff * dripstone_radius as f64) as i32
}

pub fn large_dripstone_wind_enabled(
    radius: i32,
    bluntness: f64,
    config: LargeDripstoneSampledConfig,
) -> bool {
    radius >= config.min_radius_for_wind && bluntness >= config.min_bluntness_for_wind
}

pub fn large_dripstone_wind_offset(
    pos: BlockPos,
    origin_y: i32,
    config: LargeDripstoneSampledConfig,
) -> BlockPos {
    let dy = origin_y - pos.y;
    BlockPos {
        x: pos.x
            + (config.wind_direction_radians.cos() * config.wind_speed * dy as f64).floor() as i32,
        y: pos.y,
        z: pos.z
            + (config.wind_direction_radians.sin() * config.wind_speed * dy as f64).floor() as i32,
    }
}

pub fn large_dripstone_placement_plan(
    origin: BlockPos,
    floor_y: i32,
    ceiling_y: i32,
    config: LargeDripstoneSampledConfig,
    radius_roll: i32,
    shrink_rolls: &[f32],
    shrink_factor_rolls: &[f32],
) -> Option<LargeDripstonePlacementPlan> {
    validate_large_dripstone_sampled_config(config).ok()?;
    let cave_height = ceiling_y - floor_y - 1;
    let radius = large_dripstone_selected_radius(cave_height, config, radius_roll)?;
    let stalactite = LargeDripstoneModel {
        root: BlockPos {
            x: origin.x,
            y: ceiling_y - 1,
            z: origin.z,
        },
        pointing_up: false,
        radius,
    };
    let stalagmite = LargeDripstoneModel {
        root: BlockPos {
            x: origin.x,
            y: floor_y + 1,
            z: origin.z,
        },
        pointing_up: true,
        radius,
    };
    let wind_enabled = large_dripstone_wind_enabled(radius, config.stalactite_bluntness, config)
        && large_dripstone_wind_enabled(radius, config.stalagmite_bluntness, config);

    Some(LargeDripstonePlacementPlan {
        stalactite,
        stalagmite,
        wind_enabled,
        stalactite_blocks: large_dripstone_blocks(
            stalactite,
            config.height_scale,
            config.stalactite_bluntness,
            origin.y,
            if wind_enabled { Some(config) } else { None },
            shrink_rolls,
            shrink_factor_rolls,
        ),
        stalagmite_blocks: large_dripstone_blocks(
            stalagmite,
            config.height_scale,
            config.stalagmite_bluntness,
            origin.y,
            if wind_enabled { Some(config) } else { None },
            shrink_rolls,
            shrink_factor_rolls,
        ),
    })
}

pub fn large_dripstone_blocks(
    dripstone: LargeDripstoneModel,
    scale: f64,
    bluntness: f64,
    origin_y: i32,
    wind: Option<LargeDripstoneSampledConfig>,
    shrink_rolls: &[f32],
    shrink_factor_rolls: &[f32],
) -> Vec<LargeDripstoneBlockModel> {
    let mut blocks = Vec::new();
    let mut roll_index = 0;
    let mut factor_roll_index = 0;
    for dx in -dripstone.radius..=dripstone.radius {
        for dz in -dripstone.radius..=dripstone.radius {
            let current_radius = ((dx * dx + dz * dz) as f64).sqrt();
            if current_radius > dripstone.radius as f64 {
                continue;
            }
            let mut height = large_dripstone_height_at_radius(
                current_radius,
                dripstone.radius,
                scale,
                bluntness,
            );
            if height <= 0 {
                continue;
            }
            let shrink_roll = shrink_rolls.get(roll_index).copied().unwrap_or(1.0);
            roll_index += 1;
            if shrink_roll < 0.2 {
                let factor_roll = shrink_factor_rolls
                    .get(factor_roll_index)
                    .copied()
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                factor_roll_index += 1;
                height = (height as f32 * (0.8 + 0.2 * factor_roll)) as i32;
            }
            for i in 0..height {
                let raw = BlockPos {
                    x: dripstone.root.x + dx,
                    y: if dripstone.pointing_up {
                        dripstone.root.y + i
                    } else {
                        dripstone.root.y - i
                    },
                    z: dripstone.root.z + dz,
                };
                blocks.push(LargeDripstoneBlockModel {
                    pos: wind
                        .map(|config| large_dripstone_wind_offset(raw, origin_y, config))
                        .unwrap_or(raw),
                    pointing_up: dripstone.pointing_up,
                });
            }
        }
    }
    blocks
}
