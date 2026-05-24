use super::*;

pub fn validate_fill_layer_config(
    config: FillLayerConfigurationModel,
    dimension_y_size: i32,
) -> Result<FillLayerConfigurationModel, &'static str> {
    if !(0..=dimension_y_size).contains(&config.height) {
        Err("fill layer height must be in 0..=dimension_y_size")
    } else if config.state.is_empty() {
        Err("fill layer state must not be empty")
    } else {
        Ok(config)
    }
}

pub fn fill_layer_placement_plan(
    origin: BlockPos,
    min_y: i32,
    config: FillLayerConfigurationModel,
    air_columns: &[bool],
) -> Vec<BlockPos> {
    let mut placements = Vec::new();
    let y = min_y + config.height;
    for dx in 0..16 {
        for dz in 0..16 {
            let index = (dx * 16 + dz) as usize;
            if air_columns.get(index).copied().unwrap_or(false) {
                placements.push(BlockPos {
                    x: origin.x + dx,
                    y,
                    z: origin.z + dz,
                });
            }
        }
    }
    placements
}
