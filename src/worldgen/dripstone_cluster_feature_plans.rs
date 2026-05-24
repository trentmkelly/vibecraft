use super::*;

pub fn validate_dripstone_cluster_sampled_config(
    config: DripstoneClusterSampledConfig,
) -> Result<DripstoneClusterSampledConfig, String> {
    if !(1..=512).contains(&config.floor_to_ceiling_search_range)
        || !(1..=128).contains(&config.height)
        || !(1..=128).contains(&config.x_radius)
        || !(1..=128).contains(&config.z_radius)
        || !(0..=64).contains(&config.max_stalagmite_stalactite_height_diff)
        || !(1..=64).contains(&config.height_deviation)
        || !(0..=128).contains(&config.dripstone_block_layer_thickness)
        || !(0.0..=2.0).contains(&config.density)
        || !(0.0..=2.0).contains(&config.wetness)
        || !(0.0..=1.0).contains(&config.chance_of_dripstone_column_at_max_distance_from_center)
        || !(1..=64).contains(&config.max_distance_from_edge_affecting_chance_of_dripstone_column)
        || !(1..=64).contains(&config.max_distance_from_center_affecting_height_bias)
    {
        Err("dripstone cluster sampled fields are outside vanilla codec ranges".to_string())
    } else {
        Ok(config)
    }
}

pub fn dripstone_cluster_chance_of_column(
    x_radius: i32,
    z_radius: i32,
    dx: i32,
    dz: i32,
    config: DripstoneClusterSampledConfig,
) -> f64 {
    let x_distance_from_edge = x_radius - dx.abs();
    let z_distance_from_edge = z_radius - dz.abs();
    let distance_from_edge = x_distance_from_edge.min(z_distance_from_edge);
    clamped_map_f64(
        distance_from_edge as f64,
        0.0,
        config.max_distance_from_edge_affecting_chance_of_dripstone_column as f64,
        config.chance_of_dripstone_column_at_max_distance_from_center as f64,
        1.0,
    )
}

pub fn dripstone_cluster_height_for_column(
    dx: i32,
    dz: i32,
    density: f32,
    max_height: i32,
    config: DripstoneClusterSampledConfig,
    density_roll: f32,
    biased_height_sample: f32,
) -> i32 {
    if density_roll > density {
        return 0;
    }
    let distance_from_center = dx.abs() + dz.abs();
    let _height_mean = clamped_map_f64(
        distance_from_center as f64,
        0.0,
        config.max_distance_from_center_affecting_height_bias as f64,
        max_height as f64 / 2.0,
        0.0,
    );
    biased_height_sample.clamp(0.0, max_height as f32) as i32
}

pub fn dripstone_cluster_column_plan(
    origin: BlockPos,
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
) -> DripstoneClusterColumnPlan {
    let chance = dripstone_cluster_chance_of_column(
        config.x_radius,
        config.z_radius,
        input.dx,
        input.dz,
        config,
    );
    let mut floor_y = input.floor_y;
    let water_pos = if rolls.water_roll < config.wetness
        && input.floor_y.is_some()
        && input.floor_pool_supported
    {
        let base_floor_y = input.floor_y.unwrap();
        floor_y = Some(base_floor_y - 1);
        Some(BlockPos {
            x: origin.x + input.dx,
            y: base_floor_y,
            z: origin.z + input.dz,
        })
    } else {
        None
    };

    let want_stalactite = rolls.stalactite_roll < chance;
    let mut stalactite_height = if let Some(ceiling_y) = input.ceiling_y {
        if want_stalactite && !input.ceiling_is_lava {
            let max_height = floor_y
                .map(|floor| config.height.min(ceiling_y - floor))
                .unwrap_or(config.height);
            dripstone_cluster_height_for_column(
                input.dx,
                input.dz,
                config.density,
                max_height,
                config,
                rolls.stalactite_density_roll,
                rolls.stalactite_biased_height,
            )
        } else {
            0
        }
    } else {
        0
    };

    let want_stalagmite = rolls.stalagmite_roll < chance;
    let mut stalagmite_height = if floor_y.is_some() && want_stalagmite && !input.floor_is_lava {
        if input.ceiling_y.is_some() {
            (stalactite_height
                + inclusive_roll(
                    rolls.stalagmite_height_diff_roll,
                    -config.max_stalagmite_stalactite_height_diff,
                    config.max_stalagmite_stalactite_height_diff,
                ))
            .max(0)
        } else {
            dripstone_cluster_height_for_column(
                input.dx,
                input.dz,
                config.density,
                config.height,
                config,
                rolls.stalagmite_density_roll,
                rolls.stalagmite_biased_height,
            )
        }
    } else {
        0
    };

    if let (Some(ceiling_y), Some(floor_y_value)) = (input.ceiling_y, floor_y) {
        if ceiling_y - stalactite_height <= floor_y_value + stalagmite_height {
            let lowest_stalactite_bottom = (ceiling_y - stalactite_height).max(floor_y_value + 1);
            let highest_stalagmite_top = (floor_y_value + stalagmite_height).min(ceiling_y - 1);
            let actual_stalactite_bottom = inclusive_roll(
                rolls.overlap_split_roll,
                lowest_stalactite_bottom,
                highest_stalagmite_top + 1,
            );
            let actual_stalagmite_top = actual_stalactite_bottom - 1;
            stalactite_height = ceiling_y - actual_stalactite_bottom;
            stalagmite_height = actual_stalagmite_top - floor_y_value;
        }
    }

    let column_height = input
        .ceiling_y
        .zip(floor_y)
        .map(|(ceiling, floor)| ceiling - floor);
    let merge_tips = rolls.merge_tips_roll
        && stalactite_height > 0
        && stalagmite_height > 0
        && column_height.is_some_and(|height| stalactite_height + stalagmite_height == height);
    let column_xz = BlockPos {
        x: origin.x + input.dx,
        y: origin.y,
        z: origin.z + input.dz,
    };

    DripstoneClusterColumnPlan {
        water_pos,
        ceiling_dripstone_blocks: input
            .ceiling_y
            .filter(|_| want_stalactite && !input.ceiling_is_lava)
            .map(|ceiling_y| {
                dripstone_block_layer_positions(
                    column_xz,
                    ceiling_y,
                    config.dripstone_block_layer_thickness,
                    1,
                )
            })
            .unwrap_or_default(),
        floor_dripstone_blocks: floor_y
            .filter(|_| want_stalagmite && !input.floor_is_lava)
            .map(|floor_y| {
                dripstone_block_layer_positions(
                    column_xz,
                    floor_y,
                    config.dripstone_block_layer_thickness,
                    -1,
                )
            })
            .unwrap_or_default(),
        stalactite: input
            .ceiling_y
            .map(|ceiling_y| {
                pointed_dripstone_column(
                    BlockPos {
                        x: column_xz.x,
                        y: ceiling_y - 1,
                        z: column_xz.z,
                    },
                    PointedDripstoneDirection::Down,
                    stalactite_height,
                    merge_tips,
                )
            })
            .unwrap_or_default(),
        stalagmite: floor_y
            .map(|floor_y| {
                pointed_dripstone_column(
                    BlockPos {
                        x: column_xz.x,
                        y: floor_y + 1,
                        z: column_xz.z,
                    },
                    PointedDripstoneDirection::Up,
                    stalagmite_height,
                    merge_tips,
                )
            })
            .unwrap_or_default(),
        merge_tips,
    }
}

fn dripstone_block_layer_positions(
    column_xz: BlockPos,
    start_y: i32,
    max_count: i32,
    y_step: i32,
) -> Vec<BlockPos> {
    (0..max_count)
        .map(|i| BlockPos {
            x: column_xz.x,
            y: start_y + i * y_step,
            z: column_xz.z,
        })
        .collect()
}
