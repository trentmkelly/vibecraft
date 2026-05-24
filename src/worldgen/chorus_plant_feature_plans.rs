use super::*;

pub fn supports_chorus_plant(state: &str) -> bool {
    matches!(state, "minecraft:end_stone")
}

pub fn chorus_plant_can_start(origin_empty: bool, below_state: &str) -> bool {
    origin_empty && supports_chorus_plant(below_state)
}

pub fn chorus_all_horizontal_neighbors_empty(neighbors: [bool; 4], ignored: Option<usize>) -> bool {
    neighbors
        .into_iter()
        .enumerate()
        .all(|(index, empty)| ignored == Some(index) || empty)
}

pub fn chorus_branch_target_within_spread(
    target: BlockPos,
    start: BlockPos,
    max_horizontal_spread: i32,
) -> bool {
    (target.x - start.x).abs() < max_horizontal_spread
        && (target.z - start.z).abs() < max_horizontal_spread
}

pub fn chorus_trunk_height(depth: i32, height_roll: i32) -> i32 {
    let mut height = height_roll.rem_euclid(4) + 1;
    if depth == 0 {
        height += 1;
    }
    height
}

pub fn chorus_stem_attempts(depth: i32, stem_roll: i32) -> i32 {
    let mut stems = stem_roll.rem_euclid(4);
    if depth == 0 {
        stems += 1;
    }
    stems
}

pub fn chorus_trunk_and_terminal_flower(
    current: BlockPos,
    depth: i32,
    height_roll: i32,
    placed_stem: bool,
) -> Vec<ChorusPlantPlacementBlock> {
    let height = chorus_trunk_height(depth, height_roll);
    let mut blocks = Vec::new();
    blocks.push(ChorusPlantPlacementBlock {
        pos: current,
        kind: ChorusPlantPlacementKind::Plant,
        age: None,
    });
    for i in 0..height {
        blocks.push(ChorusPlantPlacementBlock {
            pos: BlockPos {
                x: current.x,
                y: current.y + i + 1,
                z: current.z,
            },
            kind: ChorusPlantPlacementKind::Plant,
            age: None,
        });
    }
    if !placed_stem {
        blocks.push(ChorusPlantPlacementBlock {
            pos: BlockPos {
                x: current.x,
                y: current.y + height,
                z: current.z,
            },
            kind: ChorusPlantPlacementKind::Flower,
            age: Some(5),
        });
    }
    blocks
}
