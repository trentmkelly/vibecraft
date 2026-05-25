use super::*;

pub fn tree_leaf_distance_updates(
    logs: &[BlockPos],
    leaves: &[(BlockPos, i32)],
    decorations: &[BlockPos],
    roots: &[BlockPos],
) -> Vec<TreeLeafDistanceUpdate> {
    let Some(bounds) = bounding_box_for_positions(
        logs.iter()
            .chain(leaves.iter().map(|(pos, _)| pos))
            .chain(decorations.iter())
            .chain(roots.iter())
            .copied()
            .collect::<Vec<_>>()
            .as_slice(),
    ) else {
        return Vec::new();
    };

    let mut full = Vec::new();
    for pos in decorations.iter().chain(roots.iter()).copied() {
        if block_pos_inside_bounds(pos, bounds) && !full.contains(&pos) {
            full.push(pos);
        }
    }
    let mut to_check: Vec<Vec<BlockPos>> = vec![Vec::new(); 7];
    for pos in logs.iter().copied() {
        if block_pos_inside_bounds(pos, bounds) && !to_check[0].contains(&pos) {
            to_check[0].push(pos);
        }
    }

    let mut updates: Vec<TreeLeafDistanceUpdate> = Vec::new();
    let mut smallest_distance = 0_usize;
    while smallest_distance < 7 {
        if to_check[smallest_distance].is_empty() {
            smallest_distance += 1;
            continue;
        }
        let pos = to_check[smallest_distance].remove(0);
        if !block_pos_inside_bounds(pos, bounds) {
            continue;
        }
        if smallest_distance != 0 && !updates.iter().any(|update| update.pos == pos) {
            updates.push(TreeLeafDistanceUpdate {
                pos,
                distance: smallest_distance as i32,
            });
        }
        if !full.contains(&pos) {
            full.push(pos);
        }

        for neighbor in block_pos_six_neighbors(pos) {
            if !block_pos_inside_bounds(neighbor, bounds) || full.contains(&neighbor) {
                continue;
            }
            let Some((_, current_distance)) =
                leaves.iter().find(|(leaf_pos, _)| *leaf_pos == neighbor)
            else {
                continue;
            };
            let new_distance = (*current_distance).min(smallest_distance as i32 + 1);
            if new_distance < 7 {
                let bucket = new_distance as usize;
                if !to_check[bucket].contains(&neighbor) {
                    to_check[bucket].push(neighbor);
                }
                smallest_distance = smallest_distance.min(bucket);
            }
        }
    }

    updates
}

fn bounding_box_for_positions(positions: &[BlockPos]) -> Option<(BlockPos, BlockPos)> {
    let first = *positions.first()?;
    let mut min = first;
    let mut max = first;
    for pos in positions.iter().copied().skip(1) {
        min.x = min.x.min(pos.x);
        min.y = min.y.min(pos.y);
        min.z = min.z.min(pos.z);
        max.x = max.x.max(pos.x);
        max.y = max.y.max(pos.y);
        max.z = max.z.max(pos.z);
    }
    Some((min, max))
}

fn block_pos_inside_bounds(pos: BlockPos, bounds: (BlockPos, BlockPos)) -> bool {
    pos.x >= bounds.0.x
        && pos.y >= bounds.0.y
        && pos.z >= bounds.0.z
        && pos.x <= bounds.1.x
        && pos.y <= bounds.1.y
        && pos.z <= bounds.1.z
}

fn block_pos_six_neighbors(pos: BlockPos) -> [BlockPos; 6] {
    [
        BlockPos {
            x: pos.x + 1,
            ..pos
        },
        BlockPos {
            x: pos.x - 1,
            ..pos
        },
        BlockPos {
            y: pos.y + 1,
            ..pos
        },
        BlockPos {
            y: pos.y - 1,
            ..pos
        },
        BlockPos {
            z: pos.z + 1,
            ..pos
        },
        BlockPos {
            z: pos.z - 1,
            ..pos
        },
    ]
}

pub fn validate_feature_size(size: FeatureSizeModel) -> Result<FeatureSizeModel, String> {
    let min_clipped_height = match size {
        FeatureSizeModel::TwoLayers {
            min_clipped_height, ..
        }
        | FeatureSizeModel::ThreeLayers {
            min_clipped_height, ..
        } => min_clipped_height,
    };
    if min_clipped_height.is_some_and(|height| !(0..=80).contains(&height)) {
        return Err("min_clipped_height must be in 0..=80".to_string());
    }
    let valid = match size {
        FeatureSizeModel::TwoLayers {
            limit,
            lower_size,
            upper_size,
            ..
        } => {
            (0..=81).contains(&limit)
                && (0..=16).contains(&lower_size)
                && (0..=16).contains(&upper_size)
        }
        FeatureSizeModel::ThreeLayers {
            limit,
            upper_limit,
            lower_size,
            middle_size,
            upper_size,
            ..
        } => {
            (0..=80).contains(&limit)
                && (0..=80).contains(&upper_limit)
                && (0..=16).contains(&lower_size)
                && (0..=16).contains(&middle_size)
                && (0..=16).contains(&upper_size)
        }
    };
    if valid {
        Ok(size)
    } else {
        Err("feature size fields are outside vanilla codec ranges".to_string())
    }
}

pub fn feature_size_at_height(size: FeatureSizeModel, tree_height: i32, yo: i32) -> i32 {
    match size {
        FeatureSizeModel::TwoLayers {
            limit,
            lower_size,
            upper_size,
            ..
        } => {
            if yo < limit {
                lower_size
            } else {
                upper_size
            }
        }
        FeatureSizeModel::ThreeLayers {
            limit,
            upper_limit,
            lower_size,
            middle_size,
            upper_size,
            ..
        } => {
            if yo < limit {
                lower_size
            } else if yo >= tree_height - upper_limit {
                upper_size
            } else {
                middle_size
            }
        }
    }
}

pub fn tree_decorator_type(id: &str) -> Option<&'static str> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    WORLDGEN_TYPE_REGISTRIES
        .iter()
        .find(|registry| registry.id == "minecraft:tree_decorator_type")?
        .entries
        .iter()
        .copied()
        .find(|entry| entry.strip_prefix("minecraft:") == Some(name))
}

pub fn validate_tree_decorator(
    decorator: TreeDecoratorModel,
) -> Result<TreeDecoratorModel, String> {
    let probability = match decorator {
        TreeDecoratorModel::Cocoa { probability }
        | TreeDecoratorModel::LeaveVine { probability }
        | TreeDecoratorModel::Beehive { probability }
        | TreeDecoratorModel::CreakingHeart { probability }
        | TreeDecoratorModel::AttachedToLeaves { probability }
        | TreeDecoratorModel::AttachedToLogs { probability } => Some(probability),
        TreeDecoratorModel::TrunkVine
        | TreeDecoratorModel::PaleMoss { .. }
        | TreeDecoratorModel::AlterGround
        | TreeDecoratorModel::PlaceOnGround => None,
    };
    if probability.is_some_and(|probability| !(0.0..=1.0).contains(&probability)) {
        Err("tree decorator probability must be in 0.0..=1.0".to_string())
    } else if let TreeDecoratorModel::PaleMoss {
        leaves_probability,
        trunk_probability,
        ground_probability,
    } = decorator
    {
        if [leaves_probability, trunk_probability, ground_probability]
            .iter()
            .any(|probability| !(0.0..=1.0).contains(probability))
        {
            Err("pale moss decorator probabilities must be in 0.0..=1.0".to_string())
        } else {
            Ok(decorator)
        }
    } else {
        Ok(decorator)
    }
}

pub fn validate_attached_to_leaves_decorator_fields(
    exclusion_radius_xz: i32,
    exclusion_radius_y: i32,
    required_empty_blocks: i32,
    directions_len: usize,
) -> Result<(), String> {
    if !(0..=16).contains(&exclusion_radius_xz) || !(0..=16).contains(&exclusion_radius_y) {
        return Err("attached-to-leaves exclusion radii must be in 0..=16".to_string());
    }
    if !(1..=16).contains(&required_empty_blocks) {
        return Err("attached-to-leaves required_empty_blocks must be in 1..=16".to_string());
    }
    if directions_len == 0 {
        return Err("attached-to-leaves directions list must be non-empty".to_string());
    }
    Ok(())
}

pub fn validate_place_on_ground_decorator_fields(
    tries: i32,
    radius: i32,
    height: i32,
) -> Result<(), String> {
    if tries <= 0 {
        return Err("place-on-ground tries must be positive".to_string());
    }
    if radius < 0 || height < 0 {
        return Err("place-on-ground radius and height must be non-negative".to_string());
    }
    Ok(())
}

pub fn tree_decorator_should_place(probability: f32, random_next_float: f32) -> bool {
    random_next_float < probability
}

pub fn tree_lowest_trunk_or_root_positions(logs: &[BlockPos], roots: &[BlockPos]) -> Vec<BlockPos> {
    let mut sorted_logs = logs.to_vec();
    let mut sorted_roots = roots.to_vec();
    sorted_logs.sort_by_key(|pos| pos.y);
    sorted_roots.sort_by_key(|pos| pos.y);
    if sorted_roots.is_empty() {
        sorted_logs
    } else if sorted_logs
        .first()
        .zip(sorted_roots.first())
        .is_some_and(|(log, root)| log.y == root.y)
    {
        sorted_logs
            .into_iter()
            .chain(sorted_roots)
            .collect::<Vec<_>>()
    } else {
        sorted_roots
    }
}

pub fn trunk_vine_decorator_placement(
    logs: &[TrunkVineLogContext],
    random_next_int_3: &[i32],
) -> Vec<TreeDecoratorPlacement> {
    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|log| log.pos.y);
    let mut roll_index = 0;
    let mut placements = Vec::new();
    for log in sorted_logs {
        for (target, air, state) in [
            (
                BlockPos {
                    x: log.pos.x - 1,
                    y: log.pos.y,
                    z: log.pos.z,
                },
                log.west_air,
                "minecraft:vine[east=true]",
            ),
            (
                BlockPos {
                    x: log.pos.x + 1,
                    y: log.pos.y,
                    z: log.pos.z,
                },
                log.east_air,
                "minecraft:vine[west=true]",
            ),
            (
                BlockPos {
                    x: log.pos.x,
                    y: log.pos.y,
                    z: log.pos.z - 1,
                },
                log.north_air,
                "minecraft:vine[south=true]",
            ),
            (
                BlockPos {
                    x: log.pos.x,
                    y: log.pos.y,
                    z: log.pos.z + 1,
                },
                log.south_air,
                "minecraft:vine[north=true]",
            ),
        ] {
            let roll = random_next_int_3.get(roll_index).copied().unwrap_or(0);
            roll_index += 1;
            if roll.rem_euclid(3) > 0 && air {
                placements.push(TreeDecoratorPlacement { pos: target, state });
            }
        }
    }
    placements
}

fn add_hanging_vine_placements(
    placements: &mut Vec<TreeDecoratorPlacement>,
    pos: BlockPos,
    direction_state: &'static str,
    below_air: &[bool; 4],
) {
    placements.push(TreeDecoratorPlacement {
        pos,
        state: direction_state,
    });
    for (index, air) in below_air.iter().copied().enumerate() {
        if !air {
            break;
        }
        placements.push(TreeDecoratorPlacement {
            pos: BlockPos {
                x: pos.x,
                y: pos.y - 1 - index as i32,
                z: pos.z,
            },
            state: direction_state,
        });
    }
}

pub fn leave_vine_decorator_placement(
    leaves: &[LeaveVineLeafContext],
    probability: f32,
    side_rolls: &[f32],
) -> Vec<TreeDecoratorPlacement> {
    let mut sorted_leaves = leaves.to_vec();
    sorted_leaves.sort_by_key(|leaf| leaf.pos.y);
    let mut roll_index = 0;
    let mut placements = Vec::new();
    for leaf in sorted_leaves {
        for (target, air, below_air, state) in [
            (
                BlockPos {
                    x: leaf.pos.x - 1,
                    y: leaf.pos.y,
                    z: leaf.pos.z,
                },
                leaf.west_air,
                leaf.west_below_air,
                "minecraft:vine[east=true]",
            ),
            (
                BlockPos {
                    x: leaf.pos.x + 1,
                    y: leaf.pos.y,
                    z: leaf.pos.z,
                },
                leaf.east_air,
                leaf.east_below_air,
                "minecraft:vine[west=true]",
            ),
            (
                BlockPos {
                    x: leaf.pos.x,
                    y: leaf.pos.y,
                    z: leaf.pos.z - 1,
                },
                leaf.north_air,
                leaf.north_below_air,
                "minecraft:vine[south=true]",
            ),
            (
                BlockPos {
                    x: leaf.pos.x,
                    y: leaf.pos.y,
                    z: leaf.pos.z + 1,
                },
                leaf.south_air,
                leaf.south_below_air,
                "minecraft:vine[north=true]",
            ),
        ] {
            let roll = side_rolls.get(roll_index).copied().unwrap_or(1.0);
            roll_index += 1;
            if roll < probability && air {
                add_hanging_vine_placements(&mut placements, target, state, &below_air);
            }
        }
    }
    placements
}

pub fn cocoa_decorator_placement(
    logs: &[CocoaLogContext],
    probability: f32,
    global_roll: f32,
    side_rolls: &[f32],
    age_rolls: &[i32],
) -> Vec<TreeDecoratorPlacement> {
    if global_roll >= probability {
        return Vec::new();
    }
    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|log| log.pos.y);
    let Some(tree_y) = sorted_logs.first().map(|log| log.pos.y) else {
        return Vec::new();
    };
    let mut side_roll_index = 0;
    let mut age_roll_index = 0;
    let mut placements = Vec::new();
    for log in sorted_logs {
        if log.pos.y - tree_y > 2 {
            continue;
        }
        for (target, air, facing) in [
            (
                BlockPos {
                    x: log.pos.x,
                    y: log.pos.y,
                    z: log.pos.z + 1,
                },
                log.south_air,
                "north",
            ),
            (
                BlockPos {
                    x: log.pos.x - 1,
                    y: log.pos.y,
                    z: log.pos.z,
                },
                log.west_air,
                "east",
            ),
            (
                BlockPos {
                    x: log.pos.x,
                    y: log.pos.y,
                    z: log.pos.z - 1,
                },
                log.north_air,
                "south",
            ),
            (
                BlockPos {
                    x: log.pos.x + 1,
                    y: log.pos.y,
                    z: log.pos.z,
                },
                log.east_air,
                "west",
            ),
        ] {
            let side_roll = side_rolls.get(side_roll_index).copied().unwrap_or(1.0);
            side_roll_index += 1;
            if side_roll <= 0.25 && air {
                let age = age_rolls
                    .get(age_roll_index)
                    .copied()
                    .unwrap_or(0)
                    .rem_euclid(3);
                age_roll_index += 1;
                placements.push(TreeDecoratorPlacement {
                    pos: target,
                    state: match (age, facing) {
                        (0, "north") => "minecraft:cocoa[age=0,facing=north]",
                        (1, "north") => "minecraft:cocoa[age=1,facing=north]",
                        (2, "north") => "minecraft:cocoa[age=2,facing=north]",
                        (0, "east") => "minecraft:cocoa[age=0,facing=east]",
                        (1, "east") => "minecraft:cocoa[age=1,facing=east]",
                        (2, "east") => "minecraft:cocoa[age=2,facing=east]",
                        (0, "south") => "minecraft:cocoa[age=0,facing=south]",
                        (1, "south") => "minecraft:cocoa[age=1,facing=south]",
                        (2, "south") => "minecraft:cocoa[age=2,facing=south]",
                        (0, "west") => "minecraft:cocoa[age=0,facing=west]",
                        (1, "west") => "minecraft:cocoa[age=1,facing=west]",
                        _ => "minecraft:cocoa[age=2,facing=west]",
                    },
                });
            }
        }
    }
    placements
}

pub struct BeehiveDecoratorInput<'a> {
    pub logs: &'a [BlockPos],
    pub leaves: &'a [BlockPos],
    pub probability: f32,
    pub global_roll: f32,
    pub leafless_height_roll: i32,
    pub shuffled_candidate_indices: &'a [usize],
    pub candidate_air: &'a [(BlockPos, bool, bool)],
    pub bee_count_roll: i32,
    pub bee_ticks_rolls: &'a [i32],
}

pub fn beehive_decorator_placement(
    input: BeehiveDecoratorInput<'_>,
) -> Option<BeehiveDecoratorPlacement> {
    if input.logs.is_empty() || input.global_roll >= input.probability {
        return None;
    }
    let mut sorted_logs = input.logs.to_vec();
    sorted_logs.sort_by_key(|pos| pos.y);
    let mut sorted_leaves = input.leaves.to_vec();
    sorted_leaves.sort_by_key(|pos| pos.y);
    let first_log_y = sorted_logs.first()?.y;
    let last_log_y = sorted_logs.last()?.y;
    let hive_y = if let Some(first_leaf) = sorted_leaves.first() {
        (first_leaf.y - 1).max(first_log_y + 1)
    } else {
        (first_log_y + 1 + input.leafless_height_roll.rem_euclid(3)).min(last_log_y)
    };
    let mut candidates = Vec::new();
    for log in sorted_logs.iter().filter(|pos| pos.y == hive_y) {
        candidates.extend([
            BlockPos {
                x: log.x + 1,
                y: log.y,
                z: log.z,
            },
            BlockPos {
                x: log.x,
                y: log.y,
                z: log.z + 1,
            },
            BlockPos {
                x: log.x - 1,
                y: log.y,
                z: log.z,
            },
        ]);
    }
    if candidates.is_empty() {
        return None;
    }
    let mut candidate_order = input
        .shuffled_candidate_indices
        .iter()
        .copied()
        .filter_map(|index| candidates.get(index).copied())
        .collect::<Vec<_>>();
    if candidate_order.len() < candidates.len() {
        for candidate in candidates {
            if !candidate_order.contains(&candidate) {
                candidate_order.push(candidate);
            }
        }
    }
    let hive_pos = candidate_order.into_iter().find(|candidate| {
        input
            .candidate_air
            .iter()
            .find(|(pos, _, _)| pos == candidate)
            .is_some_and(|(_, self_air, front_air)| *self_air && *front_air)
    })?;
    let bee_count = 2 + input.bee_count_roll.rem_euclid(2);
    let bee_ticks_in_hive = (0..bee_count as usize)
        .map(|index| {
            input
                .bee_ticks_rolls
                .get(index)
                .copied()
                .unwrap_or(0)
                .rem_euclid(599)
        })
        .collect();
    Some(BeehiveDecoratorPlacement {
        pos: hive_pos,
        state: "minecraft:bee_nest[facing=south,honey_level=0]",
        bee_ticks_in_hive,
    })
}

pub fn creaking_heart_decorator_placement(
    logs: &[BlockPos],
    probability: f32,
    global_roll: f32,
    shuffled_log_indices: &[usize],
    adjacent_log_checks: &[(BlockPos, [bool; 6])],
) -> Option<TreeDecoratorPlacement> {
    if logs.is_empty() || global_roll >= probability {
        return None;
    }
    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|pos| pos.y);
    let mut log_order = shuffled_log_indices
        .iter()
        .copied()
        .filter_map(|index| sorted_logs.get(index).copied())
        .collect::<Vec<_>>();
    if log_order.len() < sorted_logs.len() {
        for log in sorted_logs {
            if !log_order.contains(&log) {
                log_order.push(log);
            }
        }
    }
    let pos = log_order.into_iter().find(|candidate| {
        adjacent_log_checks
            .iter()
            .find(|(pos, _)| pos == candidate)
            .is_some_and(|(_, checks)| checks.iter().all(|is_log| *is_log))
    })?;
    Some(TreeDecoratorPlacement {
        pos,
        state: "minecraft:creaking_heart[active=false,axis=y,natural=true]",
    })
}

pub fn place_on_ground_decorator_placement(
    lowest_trunk_or_root_positions: &[BlockPos],
    tries: i32,
    radius: i32,
    height: i32,
    block_state: &'static str,
    attempts: &[PlaceOnGroundAttemptContext],
) -> Result<Vec<TreeDecoratorPlacement>, String> {
    validate_place_on_ground_decorator_fields(tries, radius, height)?;
    let Some(origin) = lowest_trunk_or_root_positions.first().copied() else {
        return Ok(Vec::new());
    };
    let min_y = origin.y;
    let mut min_x = origin.x;
    let mut max_x = origin.x;
    let mut min_z = origin.z;
    let mut max_z = origin.z;
    for position in lowest_trunk_or_root_positions {
        if position.y == min_y {
            min_x = min_x.min(position.x);
            max_x = max_x.max(position.x);
            min_z = min_z.min(position.z);
            max_z = max_z.max(position.z);
        }
    }
    let bounds_min_x = min_x - radius;
    let bounds_max_x = max_x + radius;
    let bounds_min_y = min_y - height;
    let bounds_max_y = min_y + height;
    let bounds_min_z = min_z - radius;
    let bounds_max_z = max_z + radius;
    let mut placements = Vec::new();
    for attempt in attempts.iter().take(tries as usize) {
        if attempt.pos.x < bounds_min_x
            || attempt.pos.x > bounds_max_x
            || attempt.pos.y < bounds_min_y
            || attempt.pos.y > bounds_max_y
            || attempt.pos.z < bounds_min_z
            || attempt.pos.z > bounds_max_z
        {
            continue;
        }
        let above_pos = BlockPos {
            x: attempt.pos.x,
            y: attempt.pos.y + 1,
            z: attempt.pos.z,
        };
        if attempt.above_is_air_or_vine
            && attempt.pos_is_solid_render
            && attempt.motion_blocking_no_leaves_height <= above_pos.y
        {
            placements.push(TreeDecoratorPlacement {
                pos: above_pos,
                state: block_state,
            });
        }
    }
    Ok(placements)
}

fn alter_ground_scan_context_at(
    scan_contexts: &[AlterGroundScanContext],
    pos: BlockPos,
) -> AlterGroundScanContext {
    scan_contexts
        .iter()
        .copied()
        .find(|context| context.pos == pos)
        .unwrap_or(AlterGroundScanContext {
            pos,
            provider_state: None,
            is_air: true,
        })
}

fn alter_ground_place_block_at(
    placements: &mut Vec<TreeDecoratorPlacement>,
    pos: BlockPos,
    scan_contexts: &[AlterGroundScanContext],
) {
    for dy in (-3..=2).rev() {
        let cursor = BlockPos {
            x: pos.x,
            y: pos.y + dy,
            z: pos.z,
        };
        let context = alter_ground_scan_context_at(scan_contexts, cursor);
        if let Some(state) = context.provider_state {
            placements.push(TreeDecoratorPlacement { pos: cursor, state });
            break;
        }
        if !context.is_air && dy < 0 {
            break;
        }
    }
}

fn alter_ground_place_circle(
    placements: &mut Vec<TreeDecoratorPlacement>,
    pos: BlockPos,
    scan_contexts: &[AlterGroundScanContext],
) {
    for dx in -2i32..=2 {
        for dz in -2i32..=2 {
            if dx.abs() == 2 && dz.abs() == 2 {
                continue;
            }
            alter_ground_place_block_at(
                placements,
                BlockPos {
                    x: pos.x + dx,
                    y: pos.y,
                    z: pos.z + dz,
                },
                scan_contexts,
            );
        }
    }
}

pub fn alter_ground_decorator_placement(
    lowest_trunk_or_root_positions: &[BlockPos],
    random_next_int_64: &[i32],
    scan_contexts: &[AlterGroundScanContext],
) -> Vec<TreeDecoratorPlacement> {
    let mut sorted_positions = lowest_trunk_or_root_positions.to_vec();
    sorted_positions.sort_by_key(|pos| pos.y);
    let Some(min_y) = sorted_positions.first().map(|pos| pos.y) else {
        return Vec::new();
    };
    let roots = sorted_positions
        .into_iter()
        .filter(|pos| pos.y == min_y)
        .collect::<Vec<_>>();
    let mut placements = Vec::new();
    let mut roll_index = 0;
    for pos in roots {
        for anchor in [
            BlockPos {
                x: pos.x - 1,
                y: pos.y,
                z: pos.z - 1,
            },
            BlockPos {
                x: pos.x + 2,
                y: pos.y,
                z: pos.z - 1,
            },
            BlockPos {
                x: pos.x - 1,
                y: pos.y,
                z: pos.z + 2,
            },
            BlockPos {
                x: pos.x + 2,
                y: pos.y,
                z: pos.z + 2,
            },
        ] {
            alter_ground_place_circle(&mut placements, anchor, scan_contexts);
        }
        for _ in 0..5 {
            let placement = random_next_int_64
                .get(roll_index)
                .copied()
                .unwrap_or(0)
                .rem_euclid(64);
            roll_index += 1;
            let x_offset = placement % 8;
            let z_offset = placement / 8;
            if x_offset == 0 || x_offset == 7 || z_offset == 0 || z_offset == 7 {
                alter_ground_place_circle(
                    &mut placements,
                    BlockPos {
                        x: pos.x - 3 + x_offset,
                        y: pos.y,
                        z: pos.z - 3 + z_offset,
                    },
                    scan_contexts,
                );
            }
        }
    }
    placements
}

fn add_pale_moss_hanger_placements(
    placements: &mut Vec<TreeDecoratorPlacement>,
    mut pos: BlockPos,
    below_air: &[bool],
    hanger_rolls: &[f32],
    hanger_roll_index: &mut usize,
) {
    let mut below_index = 0;
    while below_air.get(below_index).copied().unwrap_or(false) {
        let roll = hanger_rolls.get(*hanger_roll_index).copied().unwrap_or(0.0);
        *hanger_roll_index += 1;
        if roll < 0.5 {
            break;
        }
        placements.push(TreeDecoratorPlacement {
            pos,
            state: "minecraft:pale_hanging_moss[tip=false]",
        });
        pos = BlockPos {
            x: pos.x,
            y: pos.y - 1,
            z: pos.z,
        };
        below_index += 1;
    }
    placements.push(TreeDecoratorPlacement {
        pos,
        state: "minecraft:pale_hanging_moss[tip=true]",
    });
}

pub struct PaleMossDecoratorInput<'a> {
    pub logs: &'a [PaleMossAttachmentContext],
    pub leaves: &'a [PaleMossAttachmentContext],
    pub leaves_probability: f32,
    pub trunk_probability: f32,
    pub ground_probability: f32,
    pub ground_roll: f32,
    pub trunk_rolls: &'a [f32],
    pub leaf_rolls: &'a [f32],
    pub hanger_rolls: &'a [f32],
}

pub fn pale_moss_decorator_placement(
    input: PaleMossDecoratorInput<'_>,
) -> Vec<TreeDecoratorPlacement> {
    if input.logs.is_empty() {
        return Vec::new();
    }
    let mut sorted_logs = input.logs.to_vec();
    sorted_logs.sort_by_key(|log| log.pos.y);
    let mut sorted_leaves = input.leaves.to_vec();
    sorted_leaves.sort_by_key(|leaf| leaf.pos.y);
    let mut placements = Vec::new();
    if input.ground_roll < input.ground_probability {
        if let Some(origin) = sorted_logs.first().map(|log| log.pos) {
            placements.push(TreeDecoratorPlacement {
                pos: BlockPos {
                    x: origin.x,
                    y: origin.y + 1,
                    z: origin.z,
                },
                state: "minecraft:configured_feature/pale_moss_patch",
            });
        }
    }
    let mut hanger_roll_index = 0;
    for (index, log) in sorted_logs.iter().enumerate() {
        let roll = input.trunk_rolls.get(index).copied().unwrap_or(1.0);
        if roll < input.trunk_probability && log.down_air {
            add_pale_moss_hanger_placements(
                &mut placements,
                BlockPos {
                    x: log.pos.x,
                    y: log.pos.y - 1,
                    z: log.pos.z,
                },
                &log.below_air,
                input.hanger_rolls,
                &mut hanger_roll_index,
            );
        }
    }
    for (index, leaf) in sorted_leaves.iter().enumerate() {
        let roll = input.leaf_rolls.get(index).copied().unwrap_or(1.0);
        if roll < input.leaves_probability && leaf.down_air {
            add_pale_moss_hanger_placements(
                &mut placements,
                BlockPos {
                    x: leaf.pos.x,
                    y: leaf.pos.y - 1,
                    z: leaf.pos.z,
                },
                &leaf.below_air,
                input.hanger_rolls,
                &mut hanger_roll_index,
            );
        }
    }
    placements
}

fn relative_direction(pos: BlockPos, direction: &str) -> BlockPos {
    match direction {
        "down" => BlockPos {
            x: pos.x,
            y: pos.y - 1,
            z: pos.z,
        },
        "up" => BlockPos {
            x: pos.x,
            y: pos.y + 1,
            z: pos.z,
        },
        "north" => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z - 1,
        },
        "south" => BlockPos {
            x: pos.x,
            y: pos.y,
            z: pos.z + 1,
        },
        "west" => BlockPos {
            x: pos.x - 1,
            y: pos.y,
            z: pos.z,
        },
        "east" => BlockPos {
            x: pos.x + 1,
            y: pos.y,
            z: pos.z,
        },
        _ => pos,
    }
}

pub fn attached_to_logs_decorator_placement(
    logs: &[BlockPos],
    probability: f32,
    block_state: &'static str,
    shuffled_log_indices: &[usize],
    direction_choices: &[&str],
    probability_rolls: &[f32],
    placement_air: &[(BlockPos, bool)],
) -> Vec<TreeDecoratorPlacement> {
    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|pos| pos.y);
    let mut log_order = shuffled_log_indices
        .iter()
        .copied()
        .filter_map(|index| sorted_logs.get(index).copied())
        .collect::<Vec<_>>();
    if log_order.len() < sorted_logs.len() {
        for log in sorted_logs {
            if !log_order.contains(&log) {
                log_order.push(log);
            }
        }
    }
    let mut placements = Vec::new();
    for (index, log) in log_order.into_iter().enumerate() {
        let direction = direction_choices.get(index).copied().unwrap_or("north");
        let placement_pos = relative_direction(log, direction);
        let roll = probability_rolls.get(index).copied().unwrap_or(1.0);
        let is_air = placement_air
            .iter()
            .find(|(pos, _)| *pos == placement_pos)
            .is_some_and(|(_, air)| *air);
        if roll <= probability && is_air {
            placements.push(TreeDecoratorPlacement {
                pos: placement_pos,
                state: block_state,
            });
        }
    }
    placements
}

fn contains_required_empty_blocks(
    leaf_pos: BlockPos,
    direction: &str,
    required_empty_blocks: i32,
    air_checks: &[(BlockPos, bool)],
) -> bool {
    (1..=required_empty_blocks).all(|distance| {
        let check_pos = match direction {
            "down" => BlockPos {
                x: leaf_pos.x,
                y: leaf_pos.y - distance,
                z: leaf_pos.z,
            },
            "up" => BlockPos {
                x: leaf_pos.x,
                y: leaf_pos.y + distance,
                z: leaf_pos.z,
            },
            "north" => BlockPos {
                x: leaf_pos.x,
                y: leaf_pos.y,
                z: leaf_pos.z - distance,
            },
            "south" => BlockPos {
                x: leaf_pos.x,
                y: leaf_pos.y,
                z: leaf_pos.z + distance,
            },
            "west" => BlockPos {
                x: leaf_pos.x - distance,
                y: leaf_pos.y,
                z: leaf_pos.z,
            },
            "east" => BlockPos {
                x: leaf_pos.x + distance,
                y: leaf_pos.y,
                z: leaf_pos.z,
            },
            _ => leaf_pos,
        };
        air_checks
            .iter()
            .find(|(pos, _)| *pos == check_pos)
            .is_some_and(|(_, air)| *air)
    })
}

pub struct AttachedToLeavesDecoratorInput<'a> {
    pub leaves: &'a [BlockPos],
    pub probability: f32,
    pub exclusion_radius_xz: i32,
    pub exclusion_radius_y: i32,
    pub required_empty_blocks: i32,
    pub block_state: &'static str,
    pub shuffled_leaf_indices: &'a [usize],
    pub direction_choices: &'a [&'a str],
    pub probability_rolls: &'a [f32],
    pub air_checks: &'a [(BlockPos, bool)],
}

pub fn attached_to_leaves_decorator_placement(
    input: AttachedToLeavesDecoratorInput<'_>,
) -> Result<Vec<TreeDecoratorPlacement>, String> {
    validate_attached_to_leaves_decorator_fields(
        input.exclusion_radius_xz,
        input.exclusion_radius_y,
        input.required_empty_blocks,
        input.direction_choices.len(),
    )?;
    let mut sorted_leaves = input.leaves.to_vec();
    sorted_leaves.sort_by_key(|pos| pos.y);
    let mut leaf_order = input
        .shuffled_leaf_indices
        .iter()
        .copied()
        .filter_map(|index| sorted_leaves.get(index).copied())
        .collect::<Vec<_>>();
    if leaf_order.len() < sorted_leaves.len() {
        for leaf in sorted_leaves {
            if !leaf_order.contains(&leaf) {
                leaf_order.push(leaf);
            }
        }
    }
    let mut blacklist = Vec::new();
    let mut placements = Vec::new();
    for (index, leaf_pos) in leaf_order.into_iter().enumerate() {
        let direction = input
            .direction_choices
            .get(index)
            .copied()
            .unwrap_or("down");
        let placement_pos = relative_direction(leaf_pos, direction);
        let roll = input.probability_rolls.get(index).copied().unwrap_or(1.0);
        if blacklist.contains(&(placement_pos.x, placement_pos.y, placement_pos.z))
            || roll >= input.probability
        {
            continue;
        }
        if !contains_required_empty_blocks(
            leaf_pos,
            direction,
            input.required_empty_blocks,
            input.air_checks,
        ) {
            continue;
        }
        for x in placement_pos.x - input.exclusion_radius_xz
            ..=placement_pos.x + input.exclusion_radius_xz
        {
            for y in placement_pos.y - input.exclusion_radius_y
                ..=placement_pos.y + input.exclusion_radius_y
            {
                for z in placement_pos.z - input.exclusion_radius_xz
                    ..=placement_pos.z + input.exclusion_radius_xz
                {
                    blacklist.push((x, y, z));
                }
            }
        }
        placements.push(TreeDecoratorPlacement {
            pos: placement_pos,
            state: input.block_state,
        });
    }
    Ok(placements)
}
