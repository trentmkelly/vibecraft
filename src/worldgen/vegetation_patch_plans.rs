use super::*;

pub fn vegetation_patch_radius(min_radius: i32, max_radius: i32, roll: i32) -> i32 {
    let span = (max_radius - min_radius + 1).max(1);
    min_radius + roll.rem_euclid(span) + 1
}

pub fn vegetation_patch_should_try_column(
    dx: i32,
    dz: i32,
    x_radius: i32,
    z_radius: i32,
    extra_edge_column_chance: f32,
    edge_roll: f32,
) -> bool {
    let is_x_edge = dx == -x_radius || dx == x_radius;
    let is_z_edge = dz == -z_radius || dz == z_radius;
    let is_corner = is_x_edge && is_z_edge;
    let is_edge_but_not_corner = (is_x_edge || is_z_edge) && !is_corner;
    !is_corner
        && (!is_edge_but_not_corner
            || (extra_edge_column_chance != 0.0 && edge_roll <= extra_edge_column_chance))
}

pub fn vegetation_patch_depth(
    min_depth: i32,
    max_depth: i32,
    depth_roll: i32,
    extra_bottom_block_chance: f32,
    extra_roll: f32,
) -> i32 {
    let span = (max_depth - min_depth + 1).max(1);
    min_depth
        + depth_roll.rem_euclid(span)
        + i32::from(extra_bottom_block_chance > 0.0 && extra_roll < extra_bottom_block_chance)
}

pub fn vegetation_patch_place_ground(
    config: &VegetationPatchConfigurationModel,
    start: BlockPos,
    existing_blocks: &[&'static str],
    depth: i32,
    random_roll: i32,
) -> Option<Vec<VegetationPatchBlock>> {
    let state = block_state_provider_sample(&config.ground_state, random_roll)?;
    let mut blocks = Vec::new();
    for i in 0..depth.max(0) {
        let existing = existing_blocks
            .get(i as usize)
            .copied()
            .unwrap_or("minecraft:air");
        if existing == state {
            continue;
        }
        if !config.replaceable.contains(&existing) {
            return (!blocks.is_empty()).then_some(blocks);
        }
        blocks.push(VegetationPatchBlock {
            pos: offset_vertical(start, config.surface, i),
            state,
        });
    }
    Some(blocks)
}

pub fn vegetation_patch_plan(
    config: &VegetationPatchConfigurationModel,
    columns: &[VegetationPatchGroundColumn],
    existing_blocks: &[&[&'static str]],
    vegetation_rolls: &[f32],
) -> VegetationPatchPlan {
    let mut ground = Vec::new();
    let mut vegetation_origins = Vec::new();
    for (index, column) in columns.iter().enumerate() {
        if let Some(mut column_blocks) = vegetation_patch_place_ground(
            config,
            column.ground_start,
            existing_blocks.get(index).copied().unwrap_or(&[]),
            column.depth,
            index as i32,
        ) {
            if !column_blocks.is_empty() {
                ground.append(&mut column_blocks);
                if config.vegetation_chance > 0.0
                    && vegetation_rolls.get(index).copied().unwrap_or(1.0)
                        < config.vegetation_chance
                {
                    vegetation_origins.push(offset_vertical(
                        column.surface_pos,
                        config.surface.opposite(),
                        1,
                    ));
                }
            }
        }
    }
    VegetationPatchPlan {
        ground,
        vegetation_origins,
    }
}
