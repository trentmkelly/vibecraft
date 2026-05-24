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
