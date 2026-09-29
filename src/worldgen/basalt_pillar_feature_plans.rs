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

/// Horizontal neighbours in the order `BasaltPillarFeature` visits them (N, S, W, E).
const BASALT_PILLAR_DIRECTIONS: [HorizontalDirection; 4] = [
    HorizontalDirection::North,
    HorizontalDirection::South,
    HorizontalDirection::West,
    HorizontalDirection::East,
];

/// Pre-evaluated world lookups and random draws for one `BasaltPillarFeature.place` call.
pub struct BasaltPillarInput<'a> {
    pub origin: BlockPos,
    /// `level.isEmptyBlock` for each step down from the origin.
    pub empty_down: &'a [bool],
    /// `level.isOutsideBuildHeight` for each step down from the origin.
    pub outside_build_height: &'a [bool],
    /// Per-step `nextInt(10)` draws for the N/S/W/E hang-offs.
    pub hangoff_rolls: &'a [(i32, i32, i32, i32)],
    /// `placeBaseHangOff` `nextBoolean()` draws for N/S/W/E on the lowest core block.
    pub base_hangoff_rolls: [bool; 4],
    /// `nextInt(10)` draw for each of the 7x7 base cells (dx outer, dz inner).
    pub base_rolls: &'a [i32],
    /// Per base cell: emptiness of up to three blocks below it.
    pub base_drop_empty_below: &'a [&'a [bool]],
    /// Per base cell: whether the block below the dropped position is non-empty.
    pub base_supported_below: &'a [bool],
}

pub fn basalt_pillar_placement_plan(
    input: BasaltPillarInput<'_>,
) -> Vec<BasaltPillarPlacementBlock> {
    let (mut blocks, y_steps) = basalt_pillar_core_blocks(&input);
    let Some(y_steps) = y_steps else {
        return blocks;
    };
    let origin = input.origin;
    // Java `placeBaseHangOff`: after the loop the cursor steps back up to the lowest placed core
    // block and rolls `nextBoolean()` for its N/S/W/E neighbours (in that order).
    let lowest_core = BlockPos {
        y: origin.y - (y_steps as i32 - 1),
        ..origin
    };
    for (direction, places) in BASALT_PILLAR_DIRECTIONS
        .into_iter()
        .zip(input.base_hangoff_rolls)
    {
        if places {
            blocks.push(BasaltPillarPlacementBlock {
                pos: offset_horizontal(lowest_core, direction, 1),
                kind: BasaltPillarBlockKind::HangOff,
            });
        }
    }
    blocks.extend(basalt_pillar_base_blocks(&input, origin.y - y_steps as i32));
    blocks
}

/// The core column and its hang-offs. Returns the blocks and the number of steps walked, or `None`
/// for the step count when the walk left the build height (Java returns `true` immediately) or
/// never started.
fn basalt_pillar_core_blocks(
    input: &BasaltPillarInput<'_>,
) -> (Vec<BasaltPillarPlacementBlock>, Option<usize>) {
    let mut blocks = Vec::new();
    let mut y_steps = 0usize;
    // Java `placeXHangoff = placeXHangoff && placeHangOff(...)`: once a side fails it stays off.
    let mut active = [true; 4];
    while input.empty_down.get(y_steps).copied().unwrap_or(false) {
        if input
            .outside_build_height
            .get(y_steps)
            .copied()
            .unwrap_or(false)
        {
            return (blocks, None);
        }
        let pos = BlockPos {
            y: input.origin.y - y_steps as i32,
            ..input.origin
        };
        blocks.push(BasaltPillarPlacementBlock {
            pos,
            kind: BasaltPillarBlockKind::Core,
        });
        let (north, south, west, east) = input
            .hangoff_rolls
            .get(y_steps)
            .copied()
            .unwrap_or((0, 0, 0, 0));
        for ((direction, side_active), roll) in BASALT_PILLAR_DIRECTIONS
            .into_iter()
            .zip(active.iter_mut())
            .zip([north, south, west, east])
        {
            if *side_active && basalt_pillar_hangoff_places(roll) {
                blocks.push(BasaltPillarPlacementBlock {
                    pos: offset_horizontal(pos, direction, 1),
                    kind: BasaltPillarBlockKind::HangOff,
                });
            } else {
                *side_active = false;
            }
        }
        y_steps += 1;
    }
    (blocks, (y_steps > 0).then_some(y_steps))
}

/// The 7x7 base disc below the pillar (`nextInt(10) < 10 - |dx * dz|`, dropping up to three
/// blocks while empty and only placing when the dropped position is supported).
fn basalt_pillar_base_blocks(
    input: &BasaltPillarInput<'_>,
    base_y: i32,
) -> Vec<BasaltPillarPlacementBlock> {
    let mut blocks = Vec::new();
    let mut base_index = 0usize;
    for dx in -3..=3 {
        for dz in -3..=3 {
            let roll = input.base_rolls.get(base_index).copied().unwrap_or(10);
            let drop_empty = input
                .base_drop_empty_below
                .get(base_index)
                .copied()
                .unwrap_or(&[]);
            let supported = input
                .base_supported_below
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
                        x: input.origin.x + dx,
                        y: base_y - drop,
                        z: input.origin.z + dz,
                    },
                    kind: BasaltPillarBlockKind::Base,
                });
            }
        }
    }
    blocks
}
