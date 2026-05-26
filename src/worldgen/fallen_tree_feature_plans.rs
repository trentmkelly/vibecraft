use super::*;

pub fn fallen_tree_log_length(min_length: i32, max_length: i32, sample_roll: i32) -> i32 {
    let span = (max_length - min_length + 1).max(1);
    min_length + sample_roll.rem_euclid(span) - 2
}

pub fn fallen_tree_start_pos(
    origin: BlockPos,
    direction: HorizontalDirection,
    distance_roll: i32,
    ground_probe: &[bool],
) -> Option<BlockPos> {
    let mut pos = offset_horizontal(origin, direction, 2 + distance_roll.rem_euclid(2));
    pos.y += 1;
    for can_place in ground_probe.iter().copied().take(6) {
        if can_place {
            return Some(pos);
        }
        pos.y -= 1;
    }
    None
}

pub fn fallen_tree_can_place_log(
    valid_tree_positions: &[bool],
    over_solid_ground: &[bool],
) -> bool {
    let mut ground_gap = 0;
    for (index, valid) in valid_tree_positions.iter().copied().enumerate() {
        if !valid {
            return false;
        }
        if !over_solid_ground.get(index).copied().unwrap_or(false) {
            ground_gap += 1;
            if ground_gap > 2 {
                return false;
            }
        } else {
            ground_gap = 0;
        }
    }
    true
}

pub struct FallenTreePlacementInput<'a> {
    pub origin: BlockPos,
    pub config: &'a FallenTreeConfigurationModel,
    pub direction: HorizontalDirection,
    pub log_length_roll: i32,
    pub distance_roll: i32,
    pub ground_probe: &'a [bool],
    pub valid_tree_positions: &'a [bool],
    pub over_solid_ground: &'a [bool],
}

pub fn fallen_tree_placement_plan(
    input: FallenTreePlacementInput<'_>,
) -> Option<FallenTreePlacementPlan> {
    let trunk_state = block_state_provider_sample(&input.config.trunk_provider, 0)?;
    let log_length = fallen_tree_log_length(
        input.config.min_log_length,
        input.config.max_log_length,
        input.log_length_roll,
    );
    let start = fallen_tree_start_pos(
        input.origin,
        input.direction,
        input.distance_roll,
        input.ground_probe,
    )?;
    let valid_len = log_length.max(0) as usize;
    if input.valid_tree_positions.len() < valid_len || input.over_solid_ground.len() < valid_len {
        return None;
    }
    if !fallen_tree_can_place_log(
        &input.valid_tree_positions[..valid_len],
        &input.over_solid_ground[..valid_len],
    ) {
        return None;
    }
    let mut blocks = vec![FallenTreeBlock {
        pos: input.origin,
        state: trunk_state,
        mark_above_for_post_processing: true,
    }];
    for i in 0..log_length.max(0) {
        blocks.push(FallenTreeBlock {
            pos: offset_horizontal(start, input.direction, i),
            state: rotated_log_state(trunk_state, input.direction),
            mark_above_for_post_processing: true,
        });
    }
    Some(FallenTreePlacementPlan {
        blocks,
        stump_decorators: input.config.stump_decorators.len(),
        log_decorators: input.config.log_decorators.len(),
    })
}
