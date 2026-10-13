use super::*;

pub fn twisting_vines_valid_ground(state: &str) -> bool {
    matches!(
        state,
        "minecraft:netherrack" | "minecraft:warped_nylium" | "minecraft:warped_wart_block"
    )
}

pub fn weeping_vines_valid_ceiling(state: &str) -> bool {
    matches!(
        state,
        "minecraft:netherrack" | "minecraft:nether_wart_block"
    )
}

pub fn vine_height(base_roll: i32, max_height: i32, double_roll: i32, single_roll: i32) -> i32 {
    let mut height = 1 + base_roll.rem_euclid(max_height.max(1));
    if double_roll.rem_euclid(6) == 0 {
        height *= 2;
    }
    if single_roll.rem_euclid(5) == 0 {
        height = 1;
    }
    height
}

pub fn vine_age(min_age: i32, max_age: i32, roll: i32) -> i32 {
    let span = (max_age - min_age + 1).max(1);
    min_age + roll.rem_euclid(span)
}

/// World operations needed by the Java `placeWeepingVinesColumn` helpers.
///
/// Implementations must make `is_empty_block` observe earlier calls to
/// `set_block`, since Java checks the world again after each plant placement.
pub trait NetherVineColumnWorld {
    fn is_empty_block(&mut self, pos: BlockPos) -> bool;

    /// Java writes vine body and head blocks with update flag 2.
    fn set_block(&mut self, block: VineColumnBlock, flags: i32);
}

/// Execute `TwistingVinesFeature.placeWeepingVinesColumn` against a mutable
/// world. `cursor` is advanced in place just like Java's `MutableBlockPos`: it
/// moves upward after each non-head iteration, including occupied positions,
/// and remains at the head if a head placement ends the loop.
pub fn place_twisting_vines_column(
    world: &mut impl NetherVineColumnWorld,
    random: &mut RandomSourceKind,
    cursor: &mut BlockPos,
    total_height: i32,
    min_age: i32,
    max_age: i32,
) {
    for height in 1..=total_height {
        if world.is_empty_block(*cursor) {
            if height == total_height || !world.is_empty_block(vine_vertical_offset(*cursor, 1)) {
                let age = random_age(random, min_age, max_age);
                world.set_block(
                    VineColumnBlock {
                        pos: *cursor,
                        state: "minecraft:twisting_vines",
                        kind: VineColumnBlockKind::Head,
                        age: Some(age),
                    },
                    2,
                );
                break;
            }

            world.set_block(
                VineColumnBlock {
                    pos: *cursor,
                    state: "minecraft:twisting_vines_plant",
                    kind: VineColumnBlockKind::Plant,
                    age: None,
                },
                2,
            );
        }

        cursor.y = cursor.y.wrapping_add(1);
    }
}

/// Execute `WeepingVinesFeature.placeWeepingVinesColumn` against a mutable
/// world, preserving its zero-based loop and mutable cursor behavior.
pub fn place_weeping_vines_column(
    world: &mut impl NetherVineColumnWorld,
    random: &mut RandomSourceKind,
    cursor: &mut BlockPos,
    total_height: i32,
    min_age: i32,
    max_age: i32,
) {
    for height in 0..=total_height {
        if world.is_empty_block(*cursor) {
            if height == total_height || !world.is_empty_block(vine_vertical_offset(*cursor, -1)) {
                let age = random_age(random, min_age, max_age);
                world.set_block(
                    VineColumnBlock {
                        pos: *cursor,
                        state: "minecraft:weeping_vines",
                        kind: VineColumnBlockKind::Head,
                        age: Some(age),
                    },
                    2,
                );
                break;
            }

            world.set_block(
                VineColumnBlock {
                    pos: *cursor,
                    state: "minecraft:weeping_vines_plant",
                    kind: VineColumnBlockKind::Plant,
                    age: None,
                },
                2,
            );
        }

        cursor.y = cursor.y.wrapping_sub(1);
    }
}

fn vine_vertical_offset(pos: BlockPos, dy: i32) -> BlockPos {
    BlockPos {
        y: pos.y.wrapping_add(dy),
        ..pos
    }
}

fn random_age(random: &mut RandomSourceKind, min_age: i32, max_age: i32) -> i32 {
    if min_age >= max_age {
        min_age
    } else {
        min_age + worldgen_random_next_i32_bound(random, max_age - min_age + 1)
    }
}

/// Snapshot-only shape helper. Its occupancy slices cannot observe prior
/// writes; use `place_twisting_vines_column` when reads must query a live world.
pub fn twisting_vines_column(
    origin: BlockPos,
    total_height: i32,
    empty_up: &[bool],
    blocked_above: &[bool],
    age_roll: i32,
) -> Vec<VineColumnBlock> {
    let mut blocks = Vec::new();
    for height in 1..=total_height {
        let step = (height - 1) as usize;
        if !empty_up.get(step).copied().unwrap_or(false) {
            continue;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y + step as i32,
            z: origin.z,
        };
        if height == total_height || blocked_above.get(step).copied().unwrap_or(false) {
            blocks.push(VineColumnBlock {
                pos,
                state: "minecraft:twisting_vines",
                kind: VineColumnBlockKind::Head,
                age: Some(vine_age(17, 25, age_roll)),
            });
            break;
        }
        blocks.push(VineColumnBlock {
            pos,
            state: "minecraft:twisting_vines_plant",
            kind: VineColumnBlockKind::Plant,
            age: None,
        });
    }
    blocks
}

/// Snapshot-only shape helper. Its occupancy slices cannot observe prior
/// writes; use `place_weeping_vines_column` when reads must query a live world.
pub fn weeping_vines_column(
    origin: BlockPos,
    total_height: i32,
    empty_down: &[bool],
    blocked_below: &[bool],
    age_roll: i32,
) -> Vec<VineColumnBlock> {
    let mut blocks = Vec::new();
    for height in 0..=total_height {
        let step = height as usize;
        if !empty_down.get(step).copied().unwrap_or(false) {
            continue;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y - height,
            z: origin.z,
        };
        if height == total_height || blocked_below.get(step).copied().unwrap_or(false) {
            blocks.push(VineColumnBlock {
                pos,
                state: "minecraft:weeping_vines",
                kind: VineColumnBlockKind::Head,
                age: Some(vine_age(17, 25, age_roll)),
            });
            break;
        }
        blocks.push(VineColumnBlock {
            pos,
            state: "minecraft:weeping_vines_plant",
            kind: VineColumnBlockKind::Plant,
            age: None,
        });
    }
    blocks
}

pub fn weeping_vines_wart_can_grow(
    candidate_empty: bool,
    wart_or_netherrack_neighbors: i32,
) -> bool {
    candidate_empty && wart_or_netherrack_neighbors == 1
}

#[cfg(test)]
#[path = "nether_vine_execution_tests.rs"]
mod execution_tests;
