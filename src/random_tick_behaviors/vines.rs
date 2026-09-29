//! `VineBlock.randomTick`: the `spreadVines`-gated growth of vines across
//! walls, over ledges and downwards.

use super::{
    bool_property, default_state, direction_name, is_air, is_block, is_empty_block, offset,
    vertical, with_bool, RandomTickWorld, HORIZONTAL, UPDATE_CLIENTS,
};
use crate::block_behavior::BlockStateModel;
use crate::block_survival::multiface_can_attach_to;
use crate::block_update::{BlockPos, Direction};
use crate::random_source::LegacyRandom;

const VINE: &str = "minecraft:vine";

/// `Direction.VALUES` (`values()` ordinal order) for `Direction.getRandom`.
const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `Direction.getClockWise()` for a horizontal direction.
fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        _ => Direction::North,
    }
}

/// `Direction.getCounterClockWise()` for a horizontal direction.
fn counter_clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        _ => Direction::North,
    }
}

fn is_horizontal(direction: Direction) -> bool {
    HORIZONTAL.contains(&direction)
}

/// `VineBlock.getPropertyForFace(direction)`.
fn face_property(direction: Direction) -> &'static str {
    direction_name(direction)
}

/// `VineBlock.isAcceptableNeighbour(level, neighbourPos, directionToNeighbour)`.
fn is_acceptable_neighbour(
    world: &impl RandomTickWorld,
    neighbour_pos: BlockPos,
    direction_to_neighbour: Direction,
) -> bool {
    multiface_can_attach_to(&world.state_at(neighbour_pos), direction_to_neighbour)
}

/// `VineBlock.canSupportAtFace` restricted to `Direction.UP`, the only face
/// the random tick asks about (`DOWN` never supports; the horizontal branch
/// is not reachable from `randomTick`).
fn can_support_at_top(world: &impl RandomTickWorld, pos: BlockPos) -> bool {
    is_acceptable_neighbour(world, vertical(pos, 1), Direction::Up)
}

/// `VineBlock.hasHorizontalConnection`.
fn has_horizontal_connection(state: &BlockStateModel) -> bool {
    HORIZONTAL
        .iter()
        .any(|direction| bool_property(state, face_property(*direction)))
}

/// `VineBlock.canSpread`: fewer than five vines within a 9x3x9 box.
fn can_spread(world: &impl RandomTickWorld, pos: BlockPos) -> bool {
    let mut max = 5;
    for x in -4..=4 {
        for y in -1..=1 {
            for z in -4..=4 {
                if is_block(&world.state_at(offset(pos, x, y, z)), VINE) {
                    max -= 1;
                    if max <= 0 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// `VineBlock.copyRandomFaces`.
fn copy_random_faces(
    from: &BlockStateModel,
    mut to: BlockStateModel,
    random: &mut LegacyRandom,
) -> BlockStateModel {
    for direction in HORIZONTAL {
        if random.next_bits(1) != 0 {
            let property = face_property(direction);
            if bool_property(from, property) {
                to = with_bool(&to, property, true);
            }
        }
    }
    to
}

/// A fresh vine with only `direction` set.
fn vine_with_face(direction: Direction) -> BlockStateModel {
    with_bool(&default_state(VINE), face_property(direction), true)
}

/// `VineBlock.randomTick`.
pub(super) fn vine_random_tick(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    random: &mut LegacyRandom,
) {
    if !world.spread_vines() || random.next_i32_bound(4) != 0 {
        return;
    }
    let test_direction = ALL_DIRECTIONS[random.next_i32_bound(6) as usize];
    if is_horizontal(test_direction) && !bool_property(state, face_property(test_direction)) {
        if can_spread(world, pos) {
            spread_horizontally(world, state, pos, test_direction, random);
        }
        return;
    }

    let above_pos = vertical(pos, 1);
    if test_direction == Direction::Up && pos.y < world.max_y() {
        if can_support_at_top(world, pos) {
            world.set_block(pos, with_bool(state, "up", true), UPDATE_CLIENTS);
            return;
        }
        if is_empty_block(world, above_pos) {
            if !can_spread(world, pos) {
                return;
            }
            let mut above_state = state.clone();
            for direction in HORIZONTAL {
                if random.next_bits(1) != 0
                    || !is_acceptable_neighbour(world, above_pos.relative(direction), direction)
                {
                    above_state = with_bool(&above_state, face_property(direction), false);
                }
            }
            if has_horizontal_connection(&above_state) {
                world.set_block(above_pos, above_state, UPDATE_CLIENTS);
            }
            return;
        }
    }

    if pos.y > world.min_y() {
        let below_pos = vertical(pos, -1);
        let below_state = world.state_at(below_pos);
        if is_air(&below_state) || is_block(&below_state, VINE) {
            let before = if is_air(&below_state) {
                default_state(VINE)
            } else {
                below_state
            };
            let after = copy_random_faces(state, before.clone(), random);
            if before != after && has_horizontal_connection(&after) {
                world.set_block(below_pos, after, UPDATE_CLIENTS);
            }
        }
    }
}

/// The `isHorizontal && !state.getValue(face)` branch of `VineBlock.randomTick`
/// once `canSpread` passed.
fn spread_horizontally(
    world: &mut impl RandomTickWorld,
    state: &BlockStateModel,
    pos: BlockPos,
    test_direction: Direction,
    random: &mut LegacyRandom,
) {
    let test_pos = pos.relative(test_direction);
    let edge_state = world.state_at(test_pos);
    if !is_air(&edge_state) {
        if is_acceptable_neighbour(world, test_pos, test_direction) {
            world.set_block(
                pos,
                with_bool(state, face_property(test_direction), true),
                UPDATE_CLIENTS,
            );
        }
        return;
    }

    let cw = clockwise(test_direction);
    let ccw = counter_clockwise(test_direction);
    let cw_has_connecting_face = bool_property(state, face_property(cw));
    let ccw_has_connecting_face = bool_property(state, face_property(ccw));
    let cw_test_pos = test_pos.relative(cw);
    let ccw_test_pos = test_pos.relative(ccw);
    if cw_has_connecting_face && is_acceptable_neighbour(world, cw_test_pos, cw) {
        world.set_block(test_pos, vine_with_face(cw), UPDATE_CLIENTS);
    } else if ccw_has_connecting_face && is_acceptable_neighbour(world, ccw_test_pos, ccw) {
        world.set_block(test_pos, vine_with_face(ccw), UPDATE_CLIENTS);
    } else {
        let opposite = test_direction.opposite();
        if cw_has_connecting_face
            && is_empty_block(world, cw_test_pos)
            && is_acceptable_neighbour(world, pos.relative(cw), opposite)
        {
            world.set_block(cw_test_pos, vine_with_face(opposite), UPDATE_CLIENTS);
        } else if ccw_has_connecting_face
            && is_empty_block(world, ccw_test_pos)
            && is_acceptable_neighbour(world, pos.relative(ccw), opposite)
        {
            world.set_block(ccw_test_pos, vine_with_face(opposite), UPDATE_CLIENTS);
        } else if f64::from(random.next_f32()) < 0.05
            && is_acceptable_neighbour(world, vertical(test_pos, 1), Direction::Up)
        {
            world.set_block(test_pos, vine_with_face(Direction::Up), UPDATE_CLIENTS);
        }
    }
}
