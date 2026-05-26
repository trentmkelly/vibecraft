use super::*;

pub(super) fn place_simple_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    input: SimpleLeavesRowInput,
) {
    for dx in -input.radius..=input.radius {
        for dz in -input.radius..=input.radius {
            if simple_leaves_row_should_skip_corner(
                dx,
                input.y_offset,
                dz,
                input.radius,
                input.blob_shape,
                input.rand_a,
                input.rand_b,
            ) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: input.origin.x + dx,
                        y: input.origin.y + input.y_offset,
                        z: input.origin.z + dz,
                    },
                    state: input.state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) struct SimpleLeavesRowInput {
    pub(super) origin: BlockPos,
    pub(super) radius: i32,
    pub(super) y_offset: i32,
    pub(super) state: &'static str,
    pub(super) blob_shape: bool,
    pub(super) rand_a: i32,
    pub(super) rand_b: i32,
}

pub(super) fn place_live_blob_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
    random: &mut RandomSourceKind,
) {
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if live_blob_leaves_row_should_skip(dx, y_offset, dz, radius, random) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn place_acacia_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    if radius < 0 {
        return;
    }
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if acacia_leaves_row_should_skip(dx.abs(), y_offset, dz.abs(), radius) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn place_dark_oak_single_trunk_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    if radius < 0 {
        return;
    }
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if dark_oak_single_trunk_leaves_row_should_skip(dx.abs(), y_offset, dz.abs(), radius) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn place_conifer_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    if radius < 0 {
        return;
    }
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if conifer_leaves_row_should_skip(dx.abs(), dz.abs(), radius) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn place_fancy_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    if radius < 0 {
        return;
    }
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if fancy_leaves_row_should_skip(dx, dz, radius) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn place_cherry_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    input: CherryLeavesRowInput,
) {
    if input.radius < 0 {
        return;
    }
    for dx in -input.radius..=input.radius {
        for dz in -input.radius..=input.radius {
            if cherry_leaves_row_should_skip(
                CherryLeavesSkipInput {
                    dx: dx.abs(),
                    y_offset: input.y_offset,
                    dz: dz.abs(),
                    radius: input.radius,
                    wide_bottom_layer_hole_chance: input.wide_bottom_layer_hole_chance,
                    corner_hole_chance: input.corner_hole_chance,
                    rand_a: input.rand_a,
                    rand_b: input.rand_b,
                },
            ) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: input.origin.x + dx,
                        y: input.origin.y + input.y_offset,
                        z: input.origin.z + dz,
                    },
                    state: input.state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) struct CherryLeavesRowInput {
    pub(super) origin: BlockPos,
    pub(super) radius: i32,
    pub(super) y_offset: i32,
    pub(super) state: &'static str,
    pub(super) wide_bottom_layer_hole_chance: f32,
    pub(super) corner_hole_chance: f32,
    pub(super) rand_a: i32,
    pub(super) rand_b: i32,
}

pub(super) fn place_mega_pine_leaves_row(
    blocks: &mut Vec<TreePlacementBlock>,
    origin: BlockPos,
    radius: i32,
    y_offset: i32,
    state: &'static str,
) {
    if radius < 0 {
        return;
    }
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            if mega_pine_leaves_row_should_skip(dx.abs(), dz.abs(), radius) {
                continue;
            }
            push_tree_block(
                blocks,
                TreePlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: origin.y + y_offset,
                        z: origin.z + dz,
                    },
                    state,
                    kind: TreePlacementBlockKind::Leaves,
                },
            );
        }
    }
}

pub(super) fn pine_foliage_rows(offset: i32, foliage_height: i32, leaf_radius: i32) -> Vec<(i32, i32)> {
    let mut rows = Vec::new();
    let mut current_radius = 0;
    for y_offset in (offset - foliage_height..=offset).rev() {
        rows.push((y_offset, current_radius));
        if current_radius >= 1 && y_offset == offset - foliage_height + 1 {
            current_radius -= 1;
        } else if current_radius < leaf_radius {
            current_radius += 1;
        }
    }
    rows
}

pub(super) fn fancy_foliage_rows(offset: i32, foliage_height: i32, leaf_radius: i32) -> Vec<(i32, i32)> {
    let mut rows = Vec::new();
    for y_offset in (offset - foliage_height..=offset).rev() {
        let current_radius = leaf_radius
            + if y_offset != offset && y_offset != offset - foliage_height {
                1
            } else {
                0
            };
        rows.push((y_offset, current_radius));
    }
    rows
}

pub(super) fn mega_jungle_foliage_rows(offset: i32, leaf_height: i32, leaf_radius: i32) -> Vec<(i32, i32)> {
    let mut rows = Vec::new();
    for y_offset in (offset - leaf_height..=offset).rev() {
        rows.push((y_offset, leaf_radius + 1 - y_offset));
    }
    rows
}

pub(super) fn cherry_foliage_rows(foliage_height: i32, leaf_radius: i32) -> Vec<(i32, i32)> {
    let current_radius = leaf_radius - 1;
    let mut rows = vec![
        (foliage_height - 3, current_radius - 2),
        (foliage_height - 4, current_radius - 1),
    ];
    for y_offset in (0..=foliage_height - 5).rev() {
        rows.push((y_offset, current_radius));
    }
    rows.push((-1, current_radius));
    rows.push((-2, current_radius - 1));
    rows
}

pub(super) fn random_spread_foliage_positions(
    origin: BlockPos,
    foliage_height: i32,
    leaf_radius: i32,
    leaf_placement_attempts: i32,
    rolls: &[i32],
) -> Vec<BlockPos> {
    if foliage_height <= 0 || leaf_radius <= 0 || leaf_placement_attempts <= 0 {
        return Vec::new();
    }
    let mut positions = Vec::new();
    for attempt in 0..leaf_placement_attempts as usize {
        let base = attempt * 6;
        let sample = |index: usize, bound: i32| {
            rolls
                .get(base + index)
                .copied()
                .unwrap_or(0)
                .rem_euclid(bound)
        };
        positions.push(BlockPos {
            x: origin.x + sample(0, leaf_radius) - sample(1, leaf_radius),
            y: origin.y + sample(2, foliage_height) - sample(3, foliage_height),
            z: origin.z + sample(4, leaf_radius) - sample(5, leaf_radius),
        });
    }
    positions
}

pub(super) fn deterministic_random_spread_rolls(
    rand_a: i32,
    rand_b: i32,
    leaf_radius: i32,
    foliage_height: i32,
    leaf_placement_attempts: i32,
) -> Vec<i32> {
    if foliage_height <= 0 || leaf_radius <= 0 || leaf_placement_attempts <= 0 {
        return Vec::new();
    }
    let mut seed = (rand_a as i64 as u64)
        ^ (rand_b as i64 as u64).rotate_left(29)
        ^ (leaf_radius as i64 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (foliage_height as i64 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let mut rolls = Vec::with_capacity(leaf_placement_attempts as usize * 6);
    for _ in 0..leaf_placement_attempts * 6 {
        seed = splitmix64(seed);
        rolls.push((seed >> 1) as i32);
    }
    rolls
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut mixed = value;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^ (mixed >> 31)
}

pub(super) fn mega_pine_foliage_rows(
    origin_y: i32,
    offset: i32,
    foliage_height: i32,
    leaf_radius: i32,
) -> Vec<(i32, i32)> {
    if foliage_height <= 0 {
        return vec![(offset, leaf_radius)];
    }
    let mut rows = Vec::new();
    let mut prev_radius = 0;
    for y_offset in (offset - foliage_height)..=offset {
        let yo = -y_offset;
        let smooth_radius =
            leaf_radius + ((yo as f32 / foliage_height as f32) * 3.5).floor() as i32;
        let jagged_radius =
            if yo > 0 && smooth_radius == prev_radius && (origin_y + y_offset).rem_euclid(2) == 0 {
                smooth_radius + 1
            } else {
                smooth_radius
            };
        rows.push((y_offset, jagged_radius));
        prev_radius = smooth_radius;
    }
    rows
}

pub(super) fn spruce_foliage_rows(
    offset: i32,
    foliage_height: i32,
    leaf_radius: i32,
    initial_radius: i32,
) -> Vec<(i32, i32)> {
    let mut rows = Vec::new();
    let mut current_radius = initial_radius.clamp(0, 1);
    let mut max_radius = 1;
    let mut min_radius = 0;
    for y_offset in (-foliage_height..=offset).rev() {
        rows.push((y_offset, current_radius));
        if current_radius >= max_radius {
            current_radius = min_radius;
            min_radius = 1;
            max_radius = (max_radius + 1).min(leaf_radius);
        } else {
            current_radius += 1;
        }
    }
    rows
}

pub(super) fn sample_inclusive_i32(min: i32, max: i32, roll: i32) -> i32 {
    if max <= min {
        min
    } else {
        min + roll.rem_euclid(max - min + 1)
    }
}

fn simple_leaves_row_should_skip_corner(
    dx: i32,
    y_offset: i32,
    dz: i32,
    radius: i32,
    blob_shape: bool,
    rand_a: i32,
    rand_b: i32,
) -> bool {
    if dx.abs() != radius || dz.abs() != radius {
        return false;
    }
    if blob_shape && y_offset == 0 {
        return true;
    }
    simple_tree_corner_roll(dx, y_offset, dz, rand_a, rand_b) == 0
}

fn live_blob_leaves_row_should_skip(
    dx: i32,
    y_offset: i32,
    dz: i32,
    radius: i32,
    random: &mut RandomSourceKind,
) -> bool {
    if dx.abs() != radius || dz.abs() != radius {
        return false;
    }
    random_next_i32_bound(random, 2) == 0 || y_offset == 0
}

fn acacia_leaves_row_should_skip(dx: i32, y_offset: i32, dz: i32, radius: i32) -> bool {
    if y_offset == 0 {
        (dx > 1 || dz > 1) && dx != 0 && dz != 0
    } else {
        dx == radius && dz == radius && radius > 0
    }
}

fn dark_oak_single_trunk_leaves_row_should_skip(
    dx: i32,
    y_offset: i32,
    dz: i32,
    radius: i32,
) -> bool {
    y_offset == -1 && dx == radius && dz == radius
}

fn conifer_leaves_row_should_skip(dx: i32, dz: i32, radius: i32) -> bool {
    dx == radius && dz == radius && radius > 0
}

pub(super) fn fancy_leaves_row_should_skip(dx: i32, dz: i32, radius: i32) -> bool {
    let dx = dx.abs() as f32 + 0.5;
    let dz = dz.abs() as f32 + 0.5;
    dx * dx + dz * dz > (radius * radius) as f32
}

pub(super) struct CherryLeavesSkipInput {
    pub(super) dx: i32,
    pub(super) y_offset: i32,
    pub(super) dz: i32,
    pub(super) radius: i32,
    pub(super) wide_bottom_layer_hole_chance: f32,
    pub(super) corner_hole_chance: f32,
    pub(super) rand_a: i32,
    pub(super) rand_b: i32,
}

pub(super) fn cherry_leaves_row_should_skip(input: CherryLeavesSkipInput) -> bool {
    if input.y_offset == -1
        && (input.dx == input.radius || input.dz == input.radius)
        && deterministic_chance_roll(
            input.dx,
            input.y_offset,
            input.dz,
            input.rand_a,
            input.rand_b,
        ) < input.wide_bottom_layer_hole_chance
    {
        return true;
    }

    let corner = input.dx == input.radius && input.dz == input.radius;
    let wide_layer = input.radius > 2;
    if wide_layer {
        corner
            || (input.dx + input.dz > input.radius * 2 - 2
                && deterministic_chance_roll(
                    input.dx,
                    input.y_offset,
                    input.dz,
                    input.rand_b,
                    input.rand_a,
                ) < input.corner_hole_chance)
    } else {
        corner
            && deterministic_chance_roll(
                input.dx,
                input.y_offset,
                input.dz,
                input.rand_b,
                input.rand_a,
            ) < input.corner_hole_chance
    }
}

fn deterministic_chance_roll(dx: i32, y_offset: i32, dz: i32, rand_a: i32, rand_b: i32) -> f32 {
    (simple_tree_corner_roll(dx, y_offset, dz, rand_a, rand_b) as f32) * 0.5
}

pub(super) fn mega_pine_leaves_row_should_skip(dx: i32, dz: i32, radius: i32) -> bool {
    dx + dz >= 7 || dx * dx + dz * dz > radius * radius
}

fn simple_tree_corner_roll(dx: i32, y_offset: i32, dz: i32, rand_a: i32, rand_b: i32) -> i32 {
    let mut value = (rand_a as i64 as u64)
        ^ (rand_b as i64 as u64).rotate_left(17)
        ^ (dx as i64 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (y_offset as i64 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ (dz as i64 as u64).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    ((value ^ (value >> 31)) & 1) as i32
}

pub(super) fn push_tree_block(blocks: &mut Vec<TreePlacementBlock>, block: TreePlacementBlock) {
    if !blocks.iter().any(|existing| existing.pos == block.pos) {
        blocks.push(block);
    }
}
