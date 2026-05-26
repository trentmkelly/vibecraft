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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointedDripstoneFeatureInput<'a> {
    pub origin: BlockPos,
    pub config: PointedDripstoneConfigurationModel,
    pub can_place_above: bool,
    pub can_place_below: bool,
    pub choose_down_when_both: bool,
    pub taller_roll: f32,
    pub next_position_empty_or_water: bool,
    pub spread_rolls: &'a [PointedDripstoneSpreadRoll],
}

pub fn pointed_dripstone_feature_plan(
    input: PointedDripstoneFeatureInput<'_>,
) -> Result<PointedDripstoneFeaturePlan, String> {
    validate_pointed_dripstone_configuration(input.config)?;
    let Some(tip_direction) = pointed_dripstone_tip_direction(
        input.can_place_above,
        input.can_place_below,
        input.choose_down_when_both,
    )
    else {
        return Ok(PointedDripstoneFeaturePlan {
            tip_direction: None,
            dripstone_blocks: Vec::new(),
            pointed_blocks: Vec::new(),
        });
    };
    let root_pos = match tip_direction {
        PointedDripstoneDirection::Down => BlockPos {
            x: input.origin.x,
            y: input.origin.y + 1,
            z: input.origin.z,
        },
        PointedDripstoneDirection::Up => BlockPos {
            x: input.origin.x,
            y: input.origin.y - 1,
            z: input.origin.z,
        },
    };
    let height = if input.taller_roll < input.config.chance_of_taller_dripstone
        && input.next_position_empty_or_water
    {
        2
    } else {
        1
    };

    Ok(PointedDripstoneFeaturePlan {
        tip_direction: Some(tip_direction),
        dripstone_blocks: pointed_dripstone_patch_positions(
            root_pos,
            input.config,
            input.spread_rolls,
        ),
        pointed_blocks: pointed_dripstone_column(input.origin, tip_direction, height, false),
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
