use super::*;

/// Java `BlueIceFeature.place` performs this many independent spread attempts.
const BLUE_ICE_SPREAD_ATTEMPTS: usize = 200;

pub fn blue_ice_can_start(
    origin_y: i32,
    sea_level: i32,
    origin_state: &str,
    below_state: &str,
    adjacent_without_down: &[&str],
) -> bool {
    origin_y < sea_level
        && (block_state_id(origin_state) == "minecraft:water"
            || block_state_id(below_state) == "minecraft:water")
        && adjacent_without_down
            .iter()
            .any(|state| block_state_id(state) == "minecraft:packed_ice")
}

pub fn blue_ice_xz_diff(y_offset: i32) -> i32 {
    let mut xz_diff = 3;
    if y_offset < 2 {
        xz_diff += y_offset / 2;
    }
    xz_diff
}

pub fn blue_ice_spread_candidate(
    origin: BlockPos,
    y_offset: i32,
    x_first_roll: i32,
    x_second_roll: i32,
    z_first_roll: i32,
    z_second_roll: i32,
) -> Option<BlockPos> {
    let xz_diff = blue_ice_xz_diff(y_offset);
    if xz_diff < 1 {
        return None;
    }
    Some(BlockPos {
        x: origin.x + x_first_roll.rem_euclid(xz_diff) - x_second_roll.rem_euclid(xz_diff),
        y: origin.y + y_offset,
        z: origin.z + z_first_roll.rem_euclid(xz_diff) - z_second_roll.rem_euclid(xz_diff),
    })
}

pub fn blue_ice_candidate_offset(
    origin: BlockPos,
    y_first_roll: i32,
    y_second_roll: i32,
    x_first_roll: i32,
    x_second_roll: i32,
    z_first_roll: i32,
    z_second_roll: i32,
) -> Option<BlockPos> {
    blue_ice_spread_candidate(
        origin,
        y_first_roll.rem_euclid(5) - y_second_roll.rem_euclid(6),
        x_first_roll,
        x_second_roll,
        z_first_roll,
        z_second_roll,
    )
}

/// Draw every candidate considered by Java `BlueIceFeature.place`, in random-call order.
///
/// The `xzDiff` guard mirrors Java's branch; Java's own Y draws are limited to -5..=4,
/// where `xzDiff` is always positive and all 200 attempts yield a candidate. This plans
/// coordinates only; the caller must still apply the feature's state gates and writes.
pub fn blue_ice_spread_candidates(
    origin: BlockPos,
    random: &mut RandomSourceKind,
) -> Vec<BlockPos> {
    let mut candidates = Vec::with_capacity(BLUE_ICE_SPREAD_ATTEMPTS);
    for _ in 0..BLUE_ICE_SPREAD_ATTEMPTS {
        let y_offset =
            worldgen_random_next_i32_bound(random, 5) - worldgen_random_next_i32_bound(random, 6);
        let xz_diff = blue_ice_xz_diff(y_offset);
        if xz_diff < 1 {
            continue;
        }

        let candidate = blue_ice_spread_candidate(
            origin,
            y_offset,
            worldgen_random_next_i32_bound(random, xz_diff),
            worldgen_random_next_i32_bound(random, xz_diff),
            worldgen_random_next_i32_bound(random, xz_diff),
            worldgen_random_next_i32_bound(random, xz_diff),
        );
        if let Some(candidate) = candidate {
            candidates.push(candidate);
        }
    }
    candidates
}

pub fn blue_ice_spread_can_place(candidate_state: &str, adjacent_states: &[&str]) -> bool {
    (block_is_air(candidate_state)
        || matches!(
            block_state_id(candidate_state),
            "minecraft:water" | "minecraft:packed_ice" | "minecraft:ice"
        ))
        && adjacent_states
            .iter()
            .any(|state| block_state_id(state) == "minecraft:blue_ice")
}

pub fn blue_ice_spread_can_replace(candidate_state: &str, adjacent_states: &[&str]) -> bool {
    blue_ice_spread_can_place(candidate_state, adjacent_states)
}

#[cfg(test)]
#[path = "blue_ice_feature_plan_regression_tests.rs"]
mod regression_tests;
