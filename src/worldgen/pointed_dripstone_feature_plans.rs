use super::*;

pub fn pointed_dripstone_column(
    start_pos: BlockPos,
    direction: PointedDripstoneDirection,
    total_length: i32,
    merged_tip: bool,
) -> Vec<PointedDripstoneBlockModel> {
    let mut thicknesses = Vec::new();
    if total_length >= 3 {
        thicknesses.push(PointedDripstoneThickness::Base);
        for _ in 0..total_length - 3 {
            thicknesses.push(PointedDripstoneThickness::Middle);
        }
    }
    if total_length >= 2 {
        thicknesses.push(PointedDripstoneThickness::Frustum);
    }
    if total_length >= 1 {
        thicknesses.push(if merged_tip {
            PointedDripstoneThickness::TipMerge
        } else {
            PointedDripstoneThickness::Tip
        });
    }

    thicknesses
        .into_iter()
        .enumerate()
        .map(|(index, thickness)| PointedDripstoneBlockModel {
            pos: BlockPos {
                x: start_pos.x,
                y: match direction {
                    PointedDripstoneDirection::Up => start_pos.y + index as i32,
                    PointedDripstoneDirection::Down => start_pos.y - index as i32,
                },
                z: start_pos.z,
            },
            direction,
            thickness,
        })
        .collect()
}

pub fn validate_pointed_dripstone_configuration(
    config: PointedDripstoneConfigurationModel,
) -> Result<PointedDripstoneConfigurationModel, String> {
    if [
        config.chance_of_taller_dripstone,
        config.chance_of_directional_spread,
        config.chance_of_spread_radius2,
        config.chance_of_spread_radius3,
    ]
    .into_iter()
    .all(|chance| (0.0..=1.0).contains(&chance))
    {
        Ok(config)
    } else {
        Err("pointed dripstone chances must be in 0.0..=1.0".to_string())
    }
}

pub fn pointed_dripstone_tip_direction(
    can_place_above: bool,
    can_place_below: bool,
    choose_down_when_both: bool,
) -> Option<PointedDripstoneDirection> {
    match (can_place_above, can_place_below) {
        (true, true) => Some(if choose_down_when_both {
            PointedDripstoneDirection::Down
        } else {
            PointedDripstoneDirection::Up
        }),
        (true, false) => Some(PointedDripstoneDirection::Down),
        (false, true) => Some(PointedDripstoneDirection::Up),
        (false, false) => None,
    }
}

pub fn pointed_dripstone_feature_plan(
    origin: BlockPos,
    config: PointedDripstoneConfigurationModel,
    can_place_above: bool,
    can_place_below: bool,
    choose_down_when_both: bool,
    taller_roll: f32,
    next_position_empty_or_water: bool,
    spread_rolls: &[PointedDripstoneSpreadRoll],
) -> Result<PointedDripstoneFeaturePlan, String> {
    validate_pointed_dripstone_configuration(config)?;
    let Some(tip_direction) =
        pointed_dripstone_tip_direction(can_place_above, can_place_below, choose_down_when_both)
    else {
        return Ok(PointedDripstoneFeaturePlan {
            tip_direction: None,
            dripstone_blocks: Vec::new(),
            pointed_blocks: Vec::new(),
        });
    };
    let root_pos = match tip_direction {
        PointedDripstoneDirection::Down => BlockPos {
            x: origin.x,
            y: origin.y + 1,
            z: origin.z,
        },
        PointedDripstoneDirection::Up => BlockPos {
            x: origin.x,
            y: origin.y - 1,
            z: origin.z,
        },
    };
    let height = if taller_roll < config.chance_of_taller_dripstone && next_position_empty_or_water
    {
        2
    } else {
        1
    };

    Ok(PointedDripstoneFeaturePlan {
        tip_direction: Some(tip_direction),
        dripstone_blocks: pointed_dripstone_patch_positions(root_pos, config, spread_rolls),
        pointed_blocks: pointed_dripstone_column(origin, tip_direction, height, false),
    })
}

pub fn pointed_dripstone_patch_positions(
    root_pos: BlockPos,
    config: PointedDripstoneConfigurationModel,
    rolls: &[PointedDripstoneSpreadRoll],
) -> Vec<BlockPos> {
    let mut positions = vec![root_pos];
    for roll in rolls {
        if roll.direction_roll > config.chance_of_directional_spread {
            continue;
        }
        let pos1 = offset_horizontal(root_pos, roll.direction, 1);
        positions.push(pos1);
        if roll.radius2_roll > config.chance_of_spread_radius2 {
            continue;
        }
        let pos2 = offset_horizontal(pos1, roll.radius2_direction, 1);
        positions.push(pos2);
        if roll.radius3_roll > config.chance_of_spread_radius3 {
            continue;
        }
        positions.push(offset_horizontal(pos2, roll.radius3_direction, 1));
    }
    positions
}
