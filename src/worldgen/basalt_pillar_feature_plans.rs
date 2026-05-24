use super::*;

pub fn basalt_pillar_can_start(origin_empty: bool, above_empty: bool) -> bool {
    origin_empty && !above_empty
}

pub fn basalt_pillar_hangoff_places(roll: i32) -> bool {
    roll.rem_euclid(10) != 0
}

pub fn basalt_pillar_base_places(dx: i32, dz: i32, roll: i32) -> bool {
    let probability = dx.abs() * dz.abs();
    roll.rem_euclid(10) < 10 - probability
}

pub fn basalt_pillar_placement_plan(
    origin: BlockPos,
    empty_down: &[bool],
    outside_build_height: &[bool],
    hangoff_rolls: &[(i32, i32, i32, i32)],
    base_rolls: &[i32],
    base_drop_empty_below: &[&[bool]],
    base_supported_below: &[bool],
) -> Vec<BasaltPillarPlacementBlock> {
    let mut blocks = Vec::new();
    let mut y_steps = 0usize;
    let mut north_active = true;
    let mut south_active = true;
    let mut west_active = true;
    let mut east_active = true;
    while empty_down.get(y_steps).copied().unwrap_or(false) {
        if outside_build_height.get(y_steps).copied().unwrap_or(false) {
            return blocks;
        }
        let pos = BlockPos {
            x: origin.x,
            y: origin.y - y_steps as i32,
            z: origin.z,
        };
        blocks.push(BasaltPillarPlacementBlock {
            pos,
            kind: BasaltPillarBlockKind::Core,
        });
        let rolls = hangoff_rolls.get(y_steps).copied().unwrap_or((0, 0, 0, 0));
        if north_active && basalt_pillar_hangoff_places(rolls.0) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::North, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            north_active = false;
        }
        if south_active && basalt_pillar_hangoff_places(rolls.1) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::South, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            south_active = false;
        }
        if west_active && basalt_pillar_hangoff_places(rolls.2) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::West, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            west_active = false;
        }
        if east_active && basalt_pillar_hangoff_places(rolls.3) {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(pos, HorizontalDirection::East, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        } else {
            east_active = false;
        }
        y_steps += 1;
    }

    if y_steps == 0 {
        return blocks;
    }
    let base_y = origin.y - y_steps as i32;
    let mut base_index = 0usize;
    for dx in -3..=3 {
        for dz in -3..=3 {
            let roll = base_rolls.get(base_index).copied().unwrap_or(10);
            let drop_empty = base_drop_empty_below
                .get(base_index)
                .copied()
                .unwrap_or(&[]);
            let supported = base_supported_below
                .get(base_index)
                .copied()
                .unwrap_or(false);
            base_index += 1;
            if !basalt_pillar_base_places(dx, dz, roll) {
                continue;
            }
            let drop = drop_empty
                .iter()
                .take(3)
                .take_while(|empty| **empty)
                .count() as i32;
            if supported {
                blocks.push(BasaltPillarPlacementBlock {
                    pos: BlockPos {
                        x: origin.x + dx,
                        y: base_y - drop,
                        z: origin.z + dz,
                    },
                    kind: BasaltPillarBlockKind::Base,
                });
            }
        }
    }
    blocks
}
