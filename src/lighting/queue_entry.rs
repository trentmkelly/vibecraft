//! Mirrors `net.minecraft.world.level.lighting.LightEngine.QueueEntry`.
//!
//! The two propagation queues in `LightEngine` store pairs of longs: the first
//! is the packed block node, the second is a `QueueEntry` mask. The mask is
//! packed identically to Java so that bit-for-bit verification against the
//! decompiled source is straightforward:
//!
//! | Bits  | Field                              |
//! |-------|------------------------------------|
//! | 0..3  | `from_level` (0..=15)              |
//! | 4..9  | `directions` (one bit per `Direction` ordinal) |
//! | 10    | `FLAG_FROM_EMPTY_SHAPE`            |
//! | 11    | `FLAG_INCREASE_FROM_EMISSION`      |
//!
//! Java masks are `LEVEL_MASK = 15L`, `DIRECTIONS_MASK = 1008L`,
//! `FLAG_FROM_EMPTY_SHAPE = 1024L`, `FLAG_INCREASE_FROM_EMISSION = 2048L`.

use crate::lighting::direction::Direction;

pub const LEVEL_MASK: i64 = 15;
pub const DIRECTIONS_MASK: i64 = 1008; // 0b1111110000
pub const FLAG_FROM_EMPTY_SHAPE: i64 = 1024;
pub const FLAG_INCREASE_FROM_EMISSION: i64 = 2048;

/// Java: `LightEngine.QueueEntry.decreaseAllDirections(int)`.
#[inline]
pub fn decrease_all_directions(old_from_level: i32) -> i64 {
    with_level(DIRECTIONS_MASK, old_from_level)
}

/// Java: `LightEngine.PULL_LIGHT_IN_ENTRY = QueueEntry.decreaseAllDirections(1)`.
#[inline]
pub fn pull_light_in_entry() -> i64 {
    decrease_all_directions(1)
}

/// Java: `LightEngine.QueueEntry.decreaseSkipOneDirection(int, Direction)`.
#[inline]
pub fn decrease_skip_one_direction(old_from_level: i32, skip: Direction) -> i64 {
    let data = without_direction(DIRECTIONS_MASK, skip);
    with_level(data, old_from_level)
}

/// Java: `LightEngine.QueueEntry.increaseLightFromEmission(int, boolean)`.
#[inline]
pub fn increase_light_from_emission(new_from_level: i32, from_empty_shape: bool) -> i64 {
    let mut data = DIRECTIONS_MASK | FLAG_INCREASE_FROM_EMISSION;
    if from_empty_shape {
        data |= FLAG_FROM_EMPTY_SHAPE;
    }
    with_level(data, new_from_level)
}

/// Java: `LightEngine.QueueEntry.increaseSkipOneDirection(int, boolean, Direction)`.
#[inline]
pub fn increase_skip_one_direction(
    new_from_level: i32,
    from_empty_shape: bool,
    skip: Direction,
) -> i64 {
    let mut data = without_direction(DIRECTIONS_MASK, skip);
    if from_empty_shape {
        data |= FLAG_FROM_EMPTY_SHAPE;
    }
    with_level(data, new_from_level)
}

/// Java: `LightEngine.QueueEntry.increaseOnlyOneDirection(int, boolean, Direction)`.
#[inline]
pub fn increase_only_one_direction(
    new_from_level: i32,
    from_empty_shape: bool,
    direction: Direction,
) -> i64 {
    let mut data = 0i64;
    if from_empty_shape {
        data |= FLAG_FROM_EMPTY_SHAPE;
    }
    data = with_direction(data, direction);
    with_level(data, new_from_level)
}

/// Java: `LightEngine.QueueEntry.increaseSkySourceInDirections(boolean down, boolean north, boolean south, boolean west, boolean east)`.
///
/// Sky sources are always level 15, never from an empty shape.
#[inline]
pub fn increase_sky_source_in_directions(
    down: bool,
    north: bool,
    south: bool,
    west: bool,
    east: bool,
) -> i64 {
    let mut data = with_level(0, 15);
    if down {
        data = with_direction(data, Direction::Down);
    }
    if north {
        data = with_direction(data, Direction::North);
    }
    if south {
        data = with_direction(data, Direction::South);
    }
    if west {
        data = with_direction(data, Direction::West);
    }
    if east {
        data = with_direction(data, Direction::East);
    }
    data
}

/// Java: `LightEngine.QueueEntry.getFromLevel(long)`.
#[inline]
pub fn get_from_level(entry: i64) -> i32 {
    (entry & LEVEL_MASK) as i32
}

/// Java: `LightEngine.QueueEntry.isFromEmptyShape(long)`.
#[inline]
pub fn is_from_empty_shape(entry: i64) -> bool {
    entry & FLAG_FROM_EMPTY_SHAPE != 0
}

/// Java: `LightEngine.QueueEntry.isIncreaseFromEmission(long)`.
#[inline]
pub fn is_increase_from_emission(entry: i64) -> bool {
    entry & FLAG_INCREASE_FROM_EMISSION != 0
}

/// Java: `LightEngine.QueueEntry.shouldPropagateInDirection(long, Direction)`.
#[inline]
pub fn should_propagate_in_direction(entry: i64, direction: Direction) -> bool {
    let mask = 1i64 << (direction.ordinal() + 4);
    entry & mask != 0
}

#[inline]
fn with_level(entry: i64, level: i32) -> i64 {
    (entry & !LEVEL_MASK) | ((level as i64) & LEVEL_MASK)
}

#[inline]
fn with_direction(entry: i64, direction: Direction) -> i64 {
    entry | (1i64 << (direction.ordinal() + 4))
}

#[inline]
fn without_direction(entry: i64, direction: Direction) -> i64 {
    entry & !(1i64 << (direction.ordinal() + 4))
}
