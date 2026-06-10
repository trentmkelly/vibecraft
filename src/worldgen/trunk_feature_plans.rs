use super::*;

pub struct ForkingTrunkPlacementInput {
    pub origin: BlockPos,
    pub tree_height: i32,
    pub trunk_state: &'static str,
    pub below_trunk_state: &'static str,
    pub lean_direction: HorizontalDirection,
    pub branch_direction: HorizontalDirection,
    pub lean_height_roll: i32,
    pub lean_steps_roll: i32,
    pub branch_pos_roll: i32,
    pub branch_steps_roll: i32,
}

pub fn forking_trunk_placement_plan(input: ForkingTrunkPlacementInput) -> TrunkPlacementPlan {
    let mut blocks = vec![TreePlacementBlock {
        pos: BlockPos {
            x: input.origin.x,
            y: input.origin.y - 1,
            z: input.origin.z,
        },
        state: input.below_trunk_state,
        kind: TreePlacementBlockKind::DirtBelowTrunk,
    }];
    let mut attachments = Vec::new();
    let lean_height = input.tree_height - input.lean_height_roll.rem_euclid(4) - 1;
    let mut lean_steps = 3 - input.lean_steps_roll.rem_euclid(3);
    let mut tx = input.origin.x;
    let mut tz = input.origin.z;
    let mut last_top_y = None;

    for y_offset in 0..input.tree_height {
        let y = input.origin.y + y_offset;
        if y_offset >= lean_height && lean_steps > 0 {
            let moved = offset_horizontal(BlockPos { x: tx, y, z: tz }, input.lean_direction, 1);
            tx = moved.x;
            tz = moved.z;
            lean_steps -= 1;
        }
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos { x: tx, y, z: tz },
                state: input.trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
        last_top_y = Some(y + 1);
    }

    if let Some(y) = last_top_y {
        attachments.push(TreeFoliageAttachmentModel {
            pos: BlockPos { x: tx, y, z: tz },
            radius_offset: 1,
            double_trunk: false,
        });
    }

    if input.branch_direction != input.lean_direction {
        tx = input.origin.x;
        tz = input.origin.z;
        let branch_pos = lean_height - input.branch_pos_roll.rem_euclid(2) - 1;
        let mut branch_steps = 1 + input.branch_steps_roll.rem_euclid(3);
        last_top_y = None;
        let mut y_offset = branch_pos;
        while y_offset < input.tree_height && branch_steps > 0 {
            if y_offset >= 1 {
                let y = input.origin.y + y_offset;
                let moved =
                    offset_horizontal(BlockPos { x: tx, y, z: tz }, input.branch_direction, 1);
                tx = moved.x;
                tz = moved.z;
                push_tree_block(
                    &mut blocks,
                    TreePlacementBlock {
                        pos: BlockPos { x: tx, y, z: tz },
                        state: input.trunk_state,
                        kind: TreePlacementBlockKind::Log,
                    },
                );
                last_top_y = Some(y + 1);
            }
            y_offset += 1;
            branch_steps -= 1;
        }

        if let Some(y) = last_top_y {
            attachments.push(TreeFoliageAttachmentModel {
                pos: BlockPos { x: tx, y, z: tz },
                radius_offset: 0,
                double_trunk: false,
            });
        }
    }

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

pub struct BendingTrunkPlacementInput {
    pub origin: BlockPos,
    pub tree_height: i32,
    pub trunk_state: &'static str,
    pub below_trunk_state: &'static str,
    pub direction: HorizontalDirection,
    pub min_height_for_leaves: i32,
    pub bend_length: i32,
    pub bend_start_roll: i32,
}

pub fn bending_trunk_placement_plan(input: BendingTrunkPlacementInput) -> TrunkPlacementPlan {
    let mut blocks = vec![TreePlacementBlock {
        pos: BlockPos {
            x: input.origin.x,
            y: input.origin.y - 1,
            z: input.origin.z,
        },
        state: input.below_trunk_state,
        kind: TreePlacementBlockKind::DirtBelowTrunk,
    }];
    let mut attachments = Vec::new();
    let log_height = input.tree_height - 1;
    let mut pos = input.origin;

    for i in 0..=log_height {
        if i + 1 >= log_height + input.bend_start_roll.rem_euclid(2) {
            pos = offset_horizontal(pos, input.direction, 1);
        }
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos,
                state: input.trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
        if i >= input.min_height_for_leaves {
            attachments.push(TreeFoliageAttachmentModel {
                pos,
                radius_offset: 0,
                double_trunk: false,
            });
        }
        pos.y += 1;
    }

    for _ in 0..=input.bend_length {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos,
                state: input.trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
        attachments.push(TreeFoliageAttachmentModel {
            pos,
            radius_offset: 0,
            double_trunk: false,
        });
        pos = offset_horizontal(pos, input.direction, 1);
    }

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

pub fn giant_trunk_placement_plan(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
) -> TrunkPlacementPlan {
    let mut blocks = Vec::new();
    for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x + dx,
                    y: origin.y - 1,
                    z: origin.z + dz,
                },
                state: below_trunk_state,
                kind: TreePlacementBlockKind::DirtBelowTrunk,
            },
        );
    }

    for y_offset in 0..tree_height {
        let full_2x2_layer = y_offset < tree_height - 1;
        for (dx, dz) in [(0, 0), (1, 0), (1, 1), (0, 1)] {
            if !full_2x2_layer && (dx != 0 || dz != 0) {
                continue;
            }
            push_tree_block(
                &mut blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state: trunk_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
        }
    }

    TrunkPlacementPlan {
        blocks,
        attachments: vec![TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: origin.x,
                y: origin.y + tree_height,
                z: origin.z,
            },
            radius_offset: 0,
            double_trunk: true,
        }],
    }
}

pub fn mega_jungle_trunk_placement_plan(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
    branches: &[MegaJungleBranchModel],
) -> TrunkPlacementPlan {
    let mut plan = giant_trunk_placement_plan(origin, tree_height, trunk_state, below_trunk_state);
    for branch in branches {
        let mut bx = 0;
        let mut bz = 0;
        for b in 0..5 {
            bx = (1.5 + branch.angle_radians.cos() * b as f32) as i32;
            bz = (1.5 + branch.angle_radians.sin() * b as f32) as i32;
            push_tree_block(
                &mut plan.blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + bx,
                        y: origin.y + branch.branch_height - 3 + b / 2,
                        z: origin.z + bz,
                    },
                    state: trunk_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
        }
        plan.attachments.push(TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: origin.x + bx,
                y: origin.y + branch.branch_height,
                z: origin.z + bz,
            },
            radius_offset: -2,
            double_trunk: false,
        });
    }
    plan
}

pub struct DarkOakTrunkPlacementInput<'a> {
    pub origin: BlockPos,
    pub tree_height: i32,
    pub trunk_state: &'static str,
    pub below_trunk_state: &'static str,
    pub lean_direction: HorizontalDirection,
    pub lean_height_roll: i32,
    pub lean_steps_roll: i32,
    pub branch_rolls: &'a [i32],
}

pub fn dark_oak_trunk_placement_plan(input: DarkOakTrunkPlacementInput<'_>) -> TrunkPlacementPlan {
    let mut blocks = Vec::new();
    for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: input.origin.x + dx,
                    y: input.origin.y - 1,
                    z: input.origin.z + dz,
                },
                state: input.below_trunk_state,
                kind: TreePlacementBlockKind::DirtBelowTrunk,
            },
        );
    }

    let lean_height = input.tree_height - input.lean_height_roll.rem_euclid(4);
    let mut lean_steps = 2 - input.lean_steps_roll.rem_euclid(3);
    let mut tx = input.origin.x;
    let mut tz = input.origin.z;
    let ey = input.origin.y + input.tree_height - 1;
    for y_offset in 0..input.tree_height {
        if y_offset >= lean_height && lean_steps > 0 {
            let moved = offset_horizontal(
                BlockPos {
                    x: tx,
                    y: input.origin.y + y_offset,
                    z: tz,
                },
                input.lean_direction,
                1,
            );
            tx = moved.x;
            tz = moved.z;
            lean_steps -= 1;
        }
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            push_tree_block(
                &mut blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: tx + dx,
                        y: input.origin.y + y_offset,
                        z: tz + dz,
                    },
                    state: input.trunk_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
        }
    }

    let mut attachments = vec![TreeFoliageAttachmentModel {
        pos: BlockPos {
            x: tx,
            y: ey,
            z: tz,
        },
        radius_offset: 0,
        double_trunk: true,
    }];
    append_dark_oak_side_branches(&mut blocks, &mut attachments, &input, ey);

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

fn append_dark_oak_side_branches(
    blocks: &mut Vec<TreePlacementBlock>,
    attachments: &mut Vec<TreeFoliageAttachmentModel>,
    input: &DarkOakTrunkPlacementInput<'_>,
    ey: i32,
) {
    let mut roll_index = 0;
    for ox in -1..=2 {
        for oz in -1..=2 {
            if (0..=1).contains(&ox) && (0..=1).contains(&oz) {
                continue;
            }
            let gate_roll = input.branch_rolls.get(roll_index).copied().unwrap_or(1);
            roll_index += 1;
            if gate_roll.rem_euclid(3) > 0 {
                continue;
            }
            let length_roll = input.branch_rolls.get(roll_index).copied().unwrap_or(0);
            roll_index += 1;
            let length = length_roll.rem_euclid(3) + 2;
            for branch_y in 0..length {
                push_tree_block(
                    blocks,
                    TreePlacementBlock {
                        pos: BlockPos {
                            x: input.origin.x + ox,
                            y: ey - branch_y - 1,
                            z: input.origin.z + oz,
                        },
                        state: input.trunk_state,
                        kind: TreePlacementBlockKind::Log,
                    },
                );
            }
            attachments.push(TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: input.origin.x + ox,
                    y: ey,
                    z: input.origin.z + oz,
                },
                radius_offset: 0,
                double_trunk: false,
            });
        }
    }
}

pub fn upwards_branching_trunk_placement_plan(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    branches: &[UpwardsBranchingBranchModel],
) -> TrunkPlacementPlan {
    let mut blocks = Vec::new();
    let mut attachments = Vec::new();

    for height_pos in 0..tree_height {
        let current_height = origin.y + height_pos;
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x,
                    y: current_height,
                    z: origin.z,
                },
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );

        for branch in branches
            .iter()
            .copied()
            .filter(|branch| branch.trunk_y_offset == height_pos && height_pos < tree_height - 1)
        {
            place_upwards_branching_branch(
                &mut blocks,
                &mut attachments,
                origin,
                tree_height,
                trunk_state,
                branch,
            );
        }

        if height_pos == tree_height - 1 {
            attachments.push(TreeFoliageAttachmentModel {
                pos: BlockPos {
                    x: origin.x,
                    y: current_height + 1,
                    z: origin.z,
                },
                radius_offset: 0,
                double_trunk: false,
            });
        }
    }

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

pub fn cherry_trunk_placement_plan(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
    branch_count: i32,
    branches: &[CherryBranchModel],
) -> TrunkPlacementPlan {
    let mut blocks = vec![TreePlacementBlock {
        pos: BlockPos {
            x: origin.x,
            y: origin.y - 1,
            z: origin.z,
        },
        state: below_trunk_state,
        kind: TreePlacementBlockKind::DirtBelowTrunk,
    }];
    let has_middle_branch = branch_count == 3;
    let trunk_height = if has_middle_branch {
        tree_height
    } else {
        branches
            .iter()
            .take(branch_count.clamp(1, 2) as usize)
            .map(|branch| branch.start_offset_from_origin + 1)
            .max()
            .unwrap_or(tree_height)
    };
    for y_offset in 0..trunk_height {
        push_tree_block(
            &mut blocks,
            TreePlacementBlock {
                pos: BlockPos {
                    x: origin.x,
                    y: origin.y + y_offset,
                    z: origin.z,
                },
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }

    let mut attachments = Vec::new();
    if has_middle_branch {
        attachments.push(TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: origin.x,
                y: origin.y + trunk_height,
                z: origin.z,
            },
            radius_offset: 0,
            double_trunk: false,
        });
    }
    for branch in branches.iter().take(branch_count.clamp(1, 3) as usize) {
        attachments.push(place_cherry_branch(
            &mut blocks,
            origin,
            tree_height,
            trunk_state,
            branch,
        ));
    }

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

pub fn fancy_trunk_placement_plan(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
    cluster_rolls: &[FancyTrunkClusterRollModel],
) -> TrunkPlacementPlan {
    fancy_trunk_placement_plan_with_limb_validator(
        origin,
        tree_height,
        trunk_state,
        below_trunk_state,
        cluster_rolls,
        |_| true,
    )
}

pub(super) fn live_fancy_trunk_placement_plan(
    block_context: &TreeDecorationBlockContext<'_>,
    previous_source_blocks: &TreeBlockOverlay,
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
    cluster_rolls: &[FancyTrunkClusterRollModel],
) -> TrunkPlacementPlan {
    fancy_trunk_placement_plan_with_limb_validator(
        origin,
        tree_height,
        trunk_state,
        below_trunk_state,
        cluster_rolls,
        |pos| {
            let world_pos = local_tree_block_to_world(block_context.source_pos, pos);
            let state = live_tree_state_with_previous_overlay(
                block_context,
                previous_source_blocks,
                world_pos,
            );
            tree_trunk_free_pos(&state)
        },
    )
}

fn fancy_trunk_placement_plan_with_limb_validator<F>(
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    below_trunk_state: &'static str,
    cluster_rolls: &[FancyTrunkClusterRollModel],
    mut limb_is_free: F,
) -> TrunkPlacementPlan
where
    F: FnMut(BlockPos) -> bool,
{
    let height = tree_height + 2;
    let trunk_height = ((height as f64) * 0.618).floor() as i32;
    let trunk_top_y = origin.y + trunk_height;
    let clusters_per_y = 1.min((1.382 + ((height as f64) / 13.0).powi(2)).floor() as i32);
    let mut blocks = vec![TreePlacementBlock {
        pos: BlockPos {
            x: origin.x,
            y: origin.y - 1,
            z: origin.z,
        },
        state: below_trunk_state,
        kind: TreePlacementBlockKind::DirtBelowTrunk,
    }];
    let foliage_coords = fancy_trunk_foliage_coords(
        FancyTrunkFoliageInput {
            origin,
            height,
            trunk_top_y,
            clusters_per_y,
            cluster_rolls,
        },
        &mut limb_is_free,
    );

    fancy_trunk_place_limb(
        &mut blocks,
        origin,
        BlockPos {
            x: origin.x,
            y: origin.y + trunk_height,
            z: origin.z,
        },
        trunk_state,
    );
    for (attachment, branch_base_y) in &foliage_coords {
        let base_coord = BlockPos {
            x: origin.x,
            y: *branch_base_y,
            z: origin.z,
        };
        if base_coord != attachment.pos
            && fancy_trunk_should_trim_branch(height, branch_base_y - origin.y)
        {
            fancy_trunk_place_limb(&mut blocks, base_coord, attachment.pos, trunk_state);
        }
    }

    let attachments = foliage_coords
        .into_iter()
        .filter_map(|(attachment, branch_base_y)| {
            if fancy_trunk_should_trim_branch(height, branch_base_y - origin.y) {
                Some(attachment)
            } else {
                None
            }
        })
        .collect();

    TrunkPlacementPlan {
        blocks,
        attachments,
    }
}

struct FancyTrunkFoliageInput<'a> {
    origin: BlockPos,
    height: i32,
    trunk_top_y: i32,
    clusters_per_y: i32,
    cluster_rolls: &'a [FancyTrunkClusterRollModel],
}

fn fancy_trunk_foliage_coords<F>(
    input: FancyTrunkFoliageInput<'_>,
    limb_is_free: &mut F,
) -> Vec<(TreeFoliageAttachmentModel, i32)>
where
    F: FnMut(BlockPos) -> bool,
{
    let mut foliage_coords = vec![(
        TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: input.origin.x,
                y: input.origin.y + input.height - 5,
                z: input.origin.z,
            },
            radius_offset: 0,
            double_trunk: false,
        },
        input.trunk_top_y,
    )];

    let mut roll_index = 0;
    for relative_y in (0..=(input.height - 5)).rev() {
        let tree_shape = fancy_trunk_tree_shape(input.height, relative_y);
        if tree_shape < 0.0 {
            continue;
        }
        append_fancy_trunk_foliage_row(
            &mut foliage_coords,
            &input,
            relative_y,
            tree_shape,
            &mut roll_index,
            limb_is_free,
        );
    }

    foliage_coords
}

fn append_fancy_trunk_foliage_row<F>(
    foliage_coords: &mut Vec<(TreeFoliageAttachmentModel, i32)>,
    input: &FancyTrunkFoliageInput<'_>,
    relative_y: i32,
    tree_shape: f32,
    roll_index: &mut usize,
    limb_is_free: &mut F,
) where
    F: FnMut(BlockPos) -> bool,
{
    for _ in 0..input.clusters_per_y {
        let roll =
            input
                .cluster_rolls
                .get(*roll_index)
                .copied()
                .unwrap_or(FancyTrunkClusterRollModel {
                    shape_float: 0.0,
                    angle_float: 0.0,
                });
        *roll_index += 1;
        let check_start =
            fancy_trunk_foliage_check_start(input.origin, relative_y, tree_shape, roll);
        let branch_top_y = fancy_trunk_branch_top_y(input.origin, check_start, input.trunk_top_y);
        let check_end = BlockPos {
            x: check_start.x,
            y: check_start.y + 5,
            z: check_start.z,
        };
        let branch_base = BlockPos {
            x: input.origin.x,
            y: branch_top_y,
            z: input.origin.z,
        };
        if fancy_trunk_limb_can_place(check_start, check_end, limb_is_free)
            && fancy_trunk_limb_can_place(branch_base, check_start, limb_is_free)
        {
            foliage_coords.push((
                TreeFoliageAttachmentModel {
                    pos: check_start,
                    radius_offset: 0,
                    double_trunk: false,
                },
                branch_top_y,
            ));
        }
    }
}

fn fancy_trunk_foliage_check_start(
    origin: BlockPos,
    relative_y: i32,
    tree_shape: f32,
    roll: FancyTrunkClusterRollModel,
) -> BlockPos {
    let radius = f64::from(tree_shape) * f64::from(roll.shape_float + 0.328);
    let angle = f64::from(roll.angle_float) * 2.0 * std::f64::consts::PI;
    BlockPos {
        x: origin.x + (radius * angle.sin() + 0.5).floor() as i32,
        y: origin.y + relative_y - 1,
        z: origin.z + (radius * angle.cos() + 0.5).floor() as i32,
    }
}

fn fancy_trunk_branch_top_y(origin: BlockPos, check_start: BlockPos, trunk_top_y: i32) -> i32 {
    let dx = origin.x - check_start.x;
    let dz = origin.z - check_start.z;
    let branch_height = check_start.y as f64 - f64::from(dx * dx + dz * dz).sqrt() * 0.381;
    if branch_height > f64::from(trunk_top_y) {
        trunk_top_y
    } else {
        branch_height as i32
    }
}

fn fancy_trunk_limb_can_place<F>(start_pos: BlockPos, end_pos: BlockPos, is_free: &mut F) -> bool
where
    F: FnMut(BlockPos) -> bool,
{
    if start_pos == end_pos {
        return true;
    }
    fancy_trunk_limb_positions(start_pos, end_pos)
        .into_iter()
        .all(is_free)
}

fn fancy_trunk_limb_positions(start_pos: BlockPos, end_pos: BlockPos) -> Vec<BlockPos> {
    let delta = BlockPos {
        x: end_pos.x - start_pos.x,
        y: end_pos.y - start_pos.y,
        z: end_pos.z - start_pos.z,
    };
    let steps = delta.x.abs().max(delta.y.abs()).max(delta.z.abs());
    if steps == 0 {
        return vec![start_pos];
    }

    let dx = delta.x as f32 / steps as f32;
    let dy = delta.y as f32 / steps as f32;
    let dz = delta.z as f32 / steps as f32;
    (0..=steps)
        .map(|i| BlockPos {
            x: start_pos.x + (0.5 + i as f32 * dx).floor() as i32,
            y: start_pos.y + (0.5 + i as f32 * dy).floor() as i32,
            z: start_pos.z + (0.5 + i as f32 * dz).floor() as i32,
        })
        .collect()
}

fn place_cherry_branch(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    _tree_height: i32,
    trunk_state: &'static str,
    branch: &CherryBranchModel,
) -> TreeFoliageAttachmentModel {
    let extend_branch_away_from_trunk = branch.middle_continues_upwards
        || branch.end_offset_from_origin < branch.start_offset_from_origin;
    let distance_to_trunk = branch.horizontal_length + i32::from(extend_branch_away_from_trunk);
    let branch_end_pos = offset_horizontal(
        BlockPos {
            x: origin.x,
            y: origin.y + branch.end_offset_from_origin,
            z: origin.z,
        },
        branch.direction,
        distance_to_trunk,
    );
    let mut log_pos = BlockPos {
        x: origin.x,
        y: origin.y + branch.start_offset_from_origin,
        z: origin.z,
    };
    let horizontal_state = rotated_log_state(trunk_state, branch.direction);
    for _ in 0..if extend_branch_away_from_trunk { 2 } else { 1 } {
        log_pos = offset_horizontal(log_pos, branch.direction, 1);
        push_tree_block(
            blocks,
            TreePlacementBlock {
                pos: log_pos,
                state: horizontal_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
    }

    let mut choice_index = 0;
    while log_pos != branch_end_pos {
        let vertical_delta = branch_end_pos.y - log_pos.y;
        let horizontal_delta = match branch.direction {
            HorizontalDirection::East => branch_end_pos.x - log_pos.x,
            HorizontalDirection::West => log_pos.x - branch_end_pos.x,
            HorizontalDirection::South => branch_end_pos.z - log_pos.z,
            HorizontalDirection::North => log_pos.z - branch_end_pos.z,
        };
        let grow_vertically = if horizontal_delta == 0 {
            true
        } else if vertical_delta == 0 {
            false
        } else {
            let choice = branch
                .grow_vertically
                .get(choice_index)
                .copied()
                .unwrap_or(true);
            choice_index += 1;
            choice
        };
        if grow_vertically {
            log_pos.y += vertical_delta.signum();
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: log_pos,
                    state: trunk_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
        } else {
            log_pos = offset_horizontal(log_pos, branch.direction, 1);
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: log_pos,
                    state: horizontal_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
        }
    }

    TreeFoliageAttachmentModel {
        pos: BlockPos {
            x: branch_end_pos.x,
            y: branch_end_pos.y + 1,
            z: branch_end_pos.z,
        },
        radius_offset: 0,
        double_trunk: false,
    }
}

pub fn fancy_trunk_tree_shape(height: i32, y: i32) -> f32 {
    if (y as f32) < height as f32 * 0.3 {
        return -1.0;
    }
    let radius = height as f32 / 2.0;
    let adjacent = radius - y as f32;
    if adjacent.abs() >= radius {
        return 0.0;
    }
    let distance = if adjacent == 0.0 {
        radius
    } else {
        (radius * radius - adjacent * adjacent).sqrt()
    };
    distance * 0.5
}

fn fancy_trunk_should_trim_branch(height: i32, local_y: i32) -> bool {
    (local_y as f64) >= (height as f64) * 0.2
}

pub(super) fn fancy_trunk_cluster_roll_count(tree_height: i32) -> i32 {
    let height = tree_height + 2;
    let clusters_per_y = 1.min((1.382 + ((height as f64) / 13.0).powi(2)).floor() as i32);
    (0..=(height - 5))
        .rev()
        .filter(|&relative_y| fancy_trunk_tree_shape(height, relative_y) >= 0.0)
        .count() as i32
        * clusters_per_y
}

fn fancy_trunk_place_limb(
    blocks: &mut Vec<TreePlacementBlock>,
    start_pos: BlockPos,
    end_pos: BlockPos,
    trunk_state: &'static str,
) {
    let delta = BlockPos {
        x: end_pos.x - start_pos.x,
        y: end_pos.y - start_pos.y,
        z: end_pos.z - start_pos.z,
    };
    let steps = delta.x.abs().max(delta.y.abs()).max(delta.z.abs());
    if steps == 0 {
        push_tree_block(
            blocks,
            TreePlacementBlock {
                pos: start_pos,
                state: trunk_state,
                kind: TreePlacementBlockKind::Log,
            },
        );
        return;
    }

    let dx = delta.x as f32 / steps as f32;
    let dy = delta.y as f32 / steps as f32;
    let dz = delta.z as f32 / steps as f32;
    for i in 0..=steps {
        let pos = BlockPos {
            x: start_pos.x + (0.5 + i as f32 * dx).floor() as i32,
            y: start_pos.y + (0.5 + i as f32 * dy).floor() as i32,
            z: start_pos.z + (0.5 + i as f32 * dz).floor() as i32,
        };
        push_tree_block(
            blocks,
            TreePlacementBlock {
                pos,
                state: fancy_trunk_log_state(trunk_state, start_pos, pos),
                kind: TreePlacementBlockKind::Log,
            },
        );
    }
}

fn fancy_trunk_log_state(
    trunk_state: &'static str,
    start_pos: BlockPos,
    block_pos: BlockPos,
) -> &'static str {
    let xdiff = (block_pos.x - start_pos.x).abs();
    let zdiff = (block_pos.z - start_pos.z).abs();
    let maxdiff = xdiff.max(zdiff);
    if maxdiff == 0 {
        trunk_state
    } else if xdiff == maxdiff {
        rotated_log_state(trunk_state, HorizontalDirection::East)
    } else {
        rotated_log_state(trunk_state, HorizontalDirection::South)
    }
}

fn place_upwards_branching_branch(
    blocks: &mut Vec<TreePlacementBlock>,
    attachments: &mut Vec<TreeFoliageAttachmentModel>,
    origin: BlockPos,
    tree_height: i32,
    trunk_state: &'static str,
    branch: UpwardsBranchingBranchModel,
) {
    let current_height = origin.y + branch.trunk_y_offset;
    let mut height_along_branch = current_height + branch.branch_pos;
    let mut log_x = origin.x;
    let mut log_z = origin.z;
    let mut branch_placement_index = branch.branch_pos;
    let mut branch_steps = branch.branch_steps;

    while branch_placement_index < tree_height && branch_steps > 0 {
        if branch_placement_index >= 1 {
            let placement_height = current_height + branch_placement_index;
            let moved = offset_horizontal(
                BlockPos {
                    x: log_x,
                    y: placement_height,
                    z: log_z,
                },
                branch.direction,
                1,
            );
            log_x = moved.x;
            log_z = moved.z;
            height_along_branch = placement_height + 1;
            let log_pos = BlockPos {
                x: log_x,
                y: placement_height,
                z: log_z,
            };
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: log_pos,
                    state: trunk_state,
                    kind: TreePlacementBlockKind::Log,
                },
            );
            attachments.push(TreeFoliageAttachmentModel {
                pos: log_pos,
                radius_offset: 0,
                double_trunk: false,
            });
        }

        branch_placement_index += 1;
        branch_steps -= 1;
    }

    if height_along_branch - current_height > 1 {
        let foliage_pos = BlockPos {
            x: log_x,
            y: height_along_branch,
            z: log_z,
        };
        attachments.push(TreeFoliageAttachmentModel {
            pos: foliage_pos,
            radius_offset: 0,
            double_trunk: false,
        });
        attachments.push(TreeFoliageAttachmentModel {
            pos: BlockPos {
                x: foliage_pos.x,
                y: foliage_pos.y - 2,
                z: foliage_pos.z,
            },
            radius_offset: 0,
            double_trunk: false,
        });
    }
}
