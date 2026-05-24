use super::*;

pub fn validate_fossil_config(
    config: &FossilFeatureConfigurationModel,
) -> Result<(), &'static str> {
    if config.fossil_structures.is_empty() {
        Err("Fossil structure lists need at least one entry")
    } else if config.fossil_structures.len() != config.overlay_structures.len() {
        Err("Fossil structure lists must be equal lengths")
    } else if !(0..=7).contains(&config.max_empty_corners_allowed) {
        Err("max_empty_corners_allowed must be in 0..=7")
    } else {
        Ok(())
    }
}

pub fn fossil_rotation(rotation_roll: i32) -> StructureRotation {
    match rotation_roll.rem_euclid(4) {
        0 => StructureRotation::None,
        1 => StructureRotation::Clockwise90,
        2 => StructureRotation::Clockwise180,
        _ => StructureRotation::Counterclockwise90,
    }
}

pub fn fossil_target_y(lowest_surface_y: i32, min_y: i32, depth_roll: i32) -> i32 {
    (lowest_surface_y - 15 - depth_roll.rem_euclid(10)).max(min_y + 10)
}

pub fn fossil_low_corner(origin: BlockPos, rotated_size_x: i32, rotated_size_z: i32) -> BlockPos {
    BlockPos {
        x: origin.x - rotated_size_x / 2,
        y: origin.y,
        z: origin.z - rotated_size_z / 2,
    }
}

pub fn fossil_placement_plan(
    config: &FossilFeatureConfigurationModel,
    origin: BlockPos,
    rotated_size_x: i32,
    rotated_size_z: i32,
    lowest_surface_y: i32,
    min_y: i32,
    rotation_roll: i32,
    fossil_index_roll: i32,
    depth_roll: i32,
    empty_corners: i32,
) -> Option<FossilPlacementPlan> {
    validate_fossil_config(config).ok()?;
    if empty_corners > config.max_empty_corners_allowed {
        return None;
    }
    let index = fossil_index_roll.rem_euclid(config.fossil_structures.len() as i32) as usize;
    let low_corner = fossil_low_corner(origin, rotated_size_x, rotated_size_z);
    Some(FossilPlacementPlan {
        fossil_structure: config.fossil_structures[index],
        overlay_structure: config.overlay_structures[index],
        rotation: fossil_rotation(rotation_roll),
        target_pos: BlockPos {
            x: low_corner.x,
            y: fossil_target_y(lowest_surface_y, min_y, depth_roll),
            z: low_corner.z,
        },
        fossil_processors: config.fossil_processors,
        overlay_processors: config.overlay_processors,
    })
}
