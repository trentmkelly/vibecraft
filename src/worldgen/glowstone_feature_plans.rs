use super::*;

pub fn glowstone_can_start(origin_empty: bool, above_state: &str) -> bool {
    origin_empty
        && matches!(
            above_state,
            "minecraft:netherrack" | "minecraft:basalt" | "minecraft:blackstone"
        )
}

pub fn glowstone_candidate_offset(
    x_roll_a: i32,
    x_roll_b: i32,
    y_roll: i32,
    z_roll_a: i32,
    z_roll_b: i32,
) -> BlockPos {
    BlockPos {
        x: x_roll_a.rem_euclid(8) - x_roll_b.rem_euclid(8),
        y: -y_roll.rem_euclid(12),
        z: z_roll_a.rem_euclid(8) - z_roll_b.rem_euclid(8),
    }
}

pub fn glowstone_can_grow(candidate_empty: bool, glowstone_neighbors: i32) -> bool {
    candidate_empty && glowstone_neighbors == 1
}
