use super::*;

pub fn underwater_magma_placement_plan(
    origin: BlockPos,
    floor_y: Option<i32>,
    config: UnderwaterMagmaConfigurationModel,
    candidates: &[UnderwaterMagmaCandidate],
    probability_rolls: &[f32],
) -> Vec<BlockPos> {
    let Some(floor_y) = floor_y else {
        return Vec::new();
    };
    let radius = config.placement_radius_around_floor.clamp(0, 64);
    let probability = config
        .placement_probability_per_valid_position
        .clamp(0.0, 1.0);
    let floor_pos = BlockPos {
        x: origin.x,
        y: floor_y,
        z: origin.z,
    };
    let mut placements = Vec::new();
    let mut roll_index = 0;
    for y in floor_pos.y - radius..=floor_pos.y + radius {
        for z in floor_pos.z - radius..=floor_pos.z + radius {
            for x in floor_pos.x - radius..=floor_pos.x + radius {
                let roll = probability_rolls.get(roll_index).copied().unwrap_or(0.0);
                roll_index += 1;
                if roll >= probability {
                    continue;
                }
                let pos = BlockPos { x, y, z };
                if candidates
                    .iter()
                    .find(|candidate| candidate.pos == pos)
                    .is_some_and(underwater_magma_is_valid_placement)
                {
                    placements.push(pos);
                }
            }
        }
    }
    placements
}

pub fn underwater_magma_is_valid_placement(candidate: &UnderwaterMagmaCandidate) -> bool {
    !matches!(candidate.block, "minecraft:water" | "minecraft:air")
        && !candidate.below_visible_from_above
        && !candidate.horizontal_visible_from_outside
}
