use super::*;

pub fn validate_root_system_configuration(
    config: &RootSystemConfigurationModel,
) -> Result<(), String> {
    if !(1..=64).contains(&config.required_vertical_space_for_tree)
        || !(1..=64).contains(&config.root_radius)
        || !(1..=256).contains(&config.root_placement_attempts)
        || !(1..=4096).contains(&config.root_column_max_height)
        || !(1..=64).contains(&config.hanging_root_radius)
        || !(1..=16).contains(&config.hanging_roots_vertical_span)
        || !(1..=256).contains(&config.hanging_root_placement_attempts)
        || !(1..=64).contains(&config.allowed_vertical_water_for_tree)
    {
        Err("root system configuration fields are outside vanilla codec ranges".to_string())
    } else {
        Ok(())
    }
}

pub fn mangrove_potential_root_positions(
    pos: BlockPos,
    prev_dir: HorizontalDirection,
    root_origin: BlockPos,
    max_root_width: i32,
    random_skew_chance: f32,
    skew_roll: f32,
    choose_next_to: bool,
) -> Vec<BlockPos> {
    let below = BlockPos {
        x: pos.x,
        y: pos.y - 1,
        z: pos.z,
    };
    let next_to = offset_horizontal(pos, prev_dir, 1);
    let next_to_below = BlockPos {
        x: next_to.x,
        y: next_to.y - 1,
        z: next_to.z,
    };
    let width = (pos.x - root_origin.x).abs()
        + (pos.y - root_origin.y).abs()
        + (pos.z - root_origin.z).abs();
    if width > max_root_width - 3 && width <= max_root_width {
        if skew_roll < random_skew_chance {
            vec![below, next_to_below]
        } else {
            vec![below]
        }
    } else if width > max_root_width || skew_roll < random_skew_chance {
        vec![below]
    } else if choose_next_to {
        vec![next_to]
    } else {
        vec![below]
    }
}

pub fn root_system_placement_plan(
    origin: BlockPos,
    origin_is_air: bool,
    config: &RootSystemConfigurationModel,
    tree_candidates: &[RootSystemTreeCandidateModel],
    root_rolls: &[RootSystemOffsetRoll],
    hanging_root_rolls: &[RootSystemOffsetRoll],
    root_replaceable_positions: &[BlockPos],
    hanging_root_candidates: &[BlockPos],
) -> Result<RootSystemPlacementPlan, String> {
    validate_root_system_configuration(config)?;
    if !origin_is_air {
        return Ok(RootSystemPlacementPlan {
            tree_origin: None,
            blocks: Vec::new(),
            attempted_roots: false,
        });
    }

    for y in 0..config.root_column_max_height {
        let working_pos = BlockPos {
            x: origin.x,
            y: origin.y + y + 1,
            z: origin.z,
        };
        let Some(candidate) = tree_candidates
            .iter()
            .find(|candidate| candidate.pos == working_pos)
        else {
            continue;
        };
        if !candidate.allowed_tree_position
            || !root_system_space_for_tree(
                &candidate.vertical_space_states,
                config.required_vertical_space_for_tree,
                config.allowed_vertical_water_for_tree,
            )
        {
            continue;
        }
        if root_system_below_rejects_tree(candidate.below_state) {
            return Ok(RootSystemPlacementPlan {
                tree_origin: None,
                blocks: Vec::new(),
                attempted_roots: false,
            });
        }
        if !candidate.tree_feature_places {
            continue;
        }

        let mut blocks = root_system_dirt_placements(
            origin,
            origin.y + y,
            config,
            root_rolls,
            root_replaceable_positions,
        );
        blocks.extend(root_system_hanging_root_placements(
            origin,
            config,
            hanging_root_rolls,
            hanging_root_candidates,
        ));
        return Ok(RootSystemPlacementPlan {
            tree_origin: Some(candidate.pos),
            blocks,
            attempted_roots: true,
        });
    }

    Ok(RootSystemPlacementPlan {
        tree_origin: None,
        blocks: Vec::new(),
        attempted_roots: false,
    })
}

pub fn root_system_space_for_tree(
    vertical_space_states: &[&str],
    required_vertical_space_for_tree: i32,
    allowed_vertical_water_for_tree: i32,
) -> bool {
    (1..=required_vertical_space_for_tree).all(|blocks_above_origin| {
        let state = vertical_space_states
            .get((blocks_above_origin - 1) as usize)
            .copied()
            .unwrap_or("minecraft:air");
        root_system_is_allowed_tree_space(
            state,
            blocks_above_origin,
            allowed_vertical_water_for_tree,
        )
    })
}

pub fn root_system_is_allowed_tree_space(
    state: &str,
    blocks_above_origin: i32,
    allowed_vertical_water_for_tree: i32,
) -> bool {
    if matches!(
        state,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    ) {
        return true;
    }
    let blocks_above_ground = blocks_above_origin + 1;
    blocks_above_ground <= allowed_vertical_water_for_tree && state == "minecraft:water"
}

fn root_system_below_rejects_tree(below_state: &str) -> bool {
    below_state == "minecraft:lava" || !root_system_solid_state(below_state)
}

fn root_system_solid_state(state: &str) -> bool {
    !matches!(
        state,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
    )
}

fn root_system_dirt_placements(
    origin: BlockPos,
    target_height: i32,
    config: &RootSystemConfigurationModel,
    rolls: &[RootSystemOffsetRoll],
    replaceable_positions: &[BlockPos],
) -> Vec<RootSystemPlacementBlock> {
    let mut placements = Vec::new();
    let mut roll_index = 0;
    for y in origin.y..target_height {
        let working_base = BlockPos {
            x: origin.x,
            y,
            z: origin.z,
        };
        for _ in 0..config.root_placement_attempts {
            let roll = rolls.get(roll_index).copied().unwrap_or_default();
            roll_index += 1;
            let pos = BlockPos {
                x: working_base.x + roll.positive_x.rem_euclid(config.root_radius)
                    - roll.negative_x.rem_euclid(config.root_radius),
                y: working_base.y,
                z: working_base.z + roll.positive_z.rem_euclid(config.root_radius)
                    - roll.negative_z.rem_euclid(config.root_radius),
            };
            if replaceable_positions.contains(&pos) {
                placements.push(RootSystemPlacementBlock {
                    pos,
                    state: block_state_provider_sample(
                        &config.root_state_provider,
                        roll_index as i32,
                    )
                    .unwrap_or("minecraft:air"),
                    kind: RootSystemPlacementKind::RootedDirt,
                });
            }
        }
    }
    placements
}

fn root_system_hanging_root_placements(
    origin: BlockPos,
    config: &RootSystemConfigurationModel,
    rolls: &[RootSystemOffsetRoll],
    candidates: &[BlockPos],
) -> Vec<RootSystemPlacementBlock> {
    let mut placements = Vec::new();
    for attempt in 0..config.hanging_root_placement_attempts {
        let roll = rolls.get(attempt as usize).copied().unwrap_or_default();
        let pos = BlockPos {
            x: origin.x + roll.positive_x.rem_euclid(config.hanging_root_radius)
                - roll.negative_x.rem_euclid(config.hanging_root_radius),
            y: origin.y
                + roll
                    .positive_y
                    .rem_euclid(config.hanging_roots_vertical_span)
                - roll
                    .negative_y
                    .rem_euclid(config.hanging_roots_vertical_span),
            z: origin.z + roll.positive_z.rem_euclid(config.hanging_root_radius)
                - roll.negative_z.rem_euclid(config.hanging_root_radius),
        };
        if candidates.contains(&pos) {
            placements.push(RootSystemPlacementBlock {
                pos,
                state: block_state_provider_sample(&config.hanging_root_state_provider, attempt)
                    .unwrap_or("minecraft:air"),
                kind: RootSystemPlacementKind::HangingRoot,
            });
        }
    }
    placements
}
