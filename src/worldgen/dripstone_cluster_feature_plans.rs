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
    let water = dripstone_cluster_water_plan(origin, config, input, rolls);
    let mut state = dripstone_cluster_initial_column_state(config, input, rolls, water.floor_y);
    dripstone_cluster_resolve_overlapping_tips(input, rolls, water.floor_y, &mut state);
    state.merge_tips = dripstone_cluster_should_merge_tips(input, rolls, water.floor_y, state);
    dripstone_cluster_materialize_column_plan(origin, config, input, water, state)
}

#[derive(Clone, Copy)]
struct DripstoneClusterWaterPlan {
    floor_y: Option<i32>,
    water_pos: Option<BlockPos>,
}

#[derive(Clone, Copy)]
struct DripstoneClusterColumnState {
    want_stalactite: bool,
    want_stalagmite: bool,
    stalactite_height: i32,
    stalagmite_height: i32,
    merge_tips: bool,
}

fn dripstone_cluster_water_plan(
    origin: BlockPos,
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
) -> DripstoneClusterWaterPlan {
    let mut floor_y = input.floor_y;
    let water_pos = if rolls.water_roll < config.wetness && input.floor_pool_supported {
        input.floor_y.map(|base_floor_y| {
            floor_y = Some(base_floor_y - 1);
            BlockPos {
                x: origin.x + input.dx,
                y: base_floor_y,
                z: origin.z + input.dz,
            }
        })
    } else {
        None
    };
    DripstoneClusterWaterPlan { floor_y, water_pos }
}

fn dripstone_cluster_initial_column_state(
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
    floor_y: Option<i32>,
) -> DripstoneClusterColumnState {
    let chance = dripstone_cluster_chance_of_column(
        config.x_radius,
        config.z_radius,
        input.dx,
        input.dz,
        config,
    );
    let want_stalactite = rolls.stalactite_roll < chance;
    let stalactite_height =
        dripstone_cluster_stalactite_height(config, input, rolls, floor_y, want_stalactite);
    let want_stalagmite = rolls.stalagmite_roll < chance;
    let stalagmite_height = dripstone_cluster_stalagmite_height(
        config,
        input,
        rolls,
        floor_y,
        want_stalagmite,
        stalactite_height,
    );
    DripstoneClusterColumnState {
        want_stalactite,
        want_stalagmite,
        stalactite_height,
        stalagmite_height,
        merge_tips: false,
    }
}

fn dripstone_cluster_stalactite_height(
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
    floor_y: Option<i32>,
    want_stalactite: bool,
) -> i32 {
    let Some(ceiling_y) = input.ceiling_y else {
        return 0;
    };
    if !want_stalactite || input.ceiling_is_lava {
        return 0;
    }
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
}

fn dripstone_cluster_stalagmite_height(
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
    floor_y: Option<i32>,
    want_stalagmite: bool,
    stalactite_height: i32,
) -> i32 {
    if floor_y.is_none() || !want_stalagmite || input.floor_is_lava {
        return 0;
    }
    if input.ceiling_y.is_some() {
        return (stalactite_height
            + inclusive_roll(
                rolls.stalagmite_height_diff_roll,
                -config.max_stalagmite_stalactite_height_diff,
                config.max_stalagmite_stalactite_height_diff,
            ))
        .max(0);
    }
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

fn dripstone_cluster_resolve_overlapping_tips(
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
    floor_y: Option<i32>,
    state: &mut DripstoneClusterColumnState,
) {
    let (Some(ceiling_y), Some(floor_y_value)) = (input.ceiling_y, floor_y) else {
        return;
    };
    if ceiling_y - state.stalactite_height > floor_y_value + state.stalagmite_height {
        return;
    }
    let lowest_stalactite_bottom = (ceiling_y - state.stalactite_height).max(floor_y_value + 1);
    let highest_stalagmite_top = (floor_y_value + state.stalagmite_height).min(ceiling_y - 1);
    let actual_stalactite_bottom = inclusive_roll(
        rolls.overlap_split_roll,
        lowest_stalactite_bottom,
        highest_stalagmite_top + 1,
    );
    let actual_stalagmite_top = actual_stalactite_bottom - 1;
    state.stalactite_height = ceiling_y - actual_stalactite_bottom;
    state.stalagmite_height = actual_stalagmite_top - floor_y_value;
}

fn dripstone_cluster_should_merge_tips(
    input: DripstoneClusterColumnInput,
    rolls: DripstoneClusterColumnRolls,
    floor_y: Option<i32>,
    state: DripstoneClusterColumnState,
) -> bool {
    let column_height = input
        .ceiling_y
        .zip(floor_y)
        .map(|(ceiling, floor)| ceiling - floor);
    rolls.merge_tips_roll
        && state.stalactite_height > 0
        && state.stalagmite_height > 0
        && column_height
            .is_some_and(|height| state.stalactite_height + state.stalagmite_height == height)
}

fn dripstone_cluster_materialize_column_plan(
    origin: BlockPos,
    config: DripstoneClusterSampledConfig,
    input: DripstoneClusterColumnInput,
    water: DripstoneClusterWaterPlan,
    state: DripstoneClusterColumnState,
) -> DripstoneClusterColumnPlan {
    let column_xz = BlockPos {
        x: origin.x + input.dx,
        y: origin.y,
        z: origin.z + input.dz,
    };

    DripstoneClusterColumnPlan {
        water_pos: water.water_pos,
        ceiling_dripstone_blocks: input
            .ceiling_y
            .filter(|_| state.want_stalactite && !input.ceiling_is_lava)
            .map(|ceiling_y| {
                dripstone_block_layer_positions(
                    column_xz,
                    ceiling_y,
                    config.dripstone_block_layer_thickness,
                    1,
                )
            })
            .unwrap_or_default(),
        floor_dripstone_blocks: water
            .floor_y
            .filter(|_| state.want_stalagmite && !input.floor_is_lava)
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
                    state.stalactite_height,
                    state.merge_tips,
                )
            })
            .unwrap_or_default(),
        stalagmite: water
            .floor_y
            .map(|floor_y| {
                pointed_dripstone_column(
                    BlockPos {
                        x: column_xz.x,
                        y: floor_y + 1,
                        z: column_xz.z,
                    },
                    PointedDripstoneDirection::Up,
                    state.stalagmite_height,
                    state.merge_tips,
                )
            })
            .unwrap_or_default(),
        merge_tips: state.merge_tips,
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
