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

pub struct FossilPlacementInput<'a> {
    pub config: &'a FossilFeatureConfigurationModel,
    pub origin: BlockPos,
    pub rotated_size_x: i32,
    pub rotated_size_z: i32,
    pub lowest_surface_y: i32,
    pub min_y: i32,
    pub rotation_roll: i32,
    pub fossil_index_roll: i32,
    pub depth_roll: i32,
    pub empty_corners: i32,
}

pub fn fossil_placement_plan(input: FossilPlacementInput<'_>) -> Option<FossilPlacementPlan> {
    validate_fossil_config(input.config).ok()?;
    if input.empty_corners > input.config.max_empty_corners_allowed {
        return None;
    }
    let index = input
        .fossil_index_roll
        .rem_euclid(input.config.fossil_structures.len() as i32) as usize;
    let low_corner = fossil_low_corner(input.origin, input.rotated_size_x, input.rotated_size_z);
    Some(FossilPlacementPlan {
        fossil_structure: input.config.fossil_structures[index],
        overlay_structure: input.config.overlay_structures[index],
        rotation: fossil_rotation(input.rotation_roll),
        target_pos: BlockPos {
            x: low_corner.x,
            y: fossil_target_y(input.lowest_surface_y, input.min_y, input.depth_roll),
            z: low_corner.z,
        },
        fossil_processors: input.config.fossil_processors,
        overlay_processors: input.config.overlay_processors,
    })
}
