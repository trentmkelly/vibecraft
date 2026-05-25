use super::*;

pub fn blue_ice_can_start(
    origin_y: i32,
    sea_level: i32,
    origin_state: &str,
    below_state: &str,
    adjacent_without_down: &[&str],
) -> bool {
    origin_y < sea_level
        && (origin_state == "minecraft:water" || below_state == "minecraft:water")
        && adjacent_without_down.contains(&"minecraft:packed_ice")
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

pub fn blue_ice_spread_can_place(candidate_state: &str, adjacent_states: &[&str]) -> bool {
    matches!(
        candidate_state,
        "minecraft:air" | "minecraft:water" | "minecraft:packed_ice" | "minecraft:ice"
    ) && adjacent_states.contains(&"minecraft:blue_ice")
}

pub fn blue_ice_spread_can_replace(candidate_state: &str, adjacent_states: &[&str]) -> bool {
    blue_ice_spread_can_place(candidate_state, adjacent_states)
}
