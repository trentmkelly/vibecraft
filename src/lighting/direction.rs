//! The six axis-aligned cardinal directions, in the same ordinal order as
//! `net.minecraft.core.Direction`.
//!
//! The ordinal indices are load-bearing: `QueueEntry` packs propagation
//! direction bits at `1 << (ordinal + 4)`, so any reorder here would silently
//! break propagation. Java order is `DOWN, UP, NORTH, SOUTH, WEST, EAST`.

/// Java: `net.minecraft.core.Direction` (subset of the values used by lighting).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

/// Java: `LightEngine.PROPAGATION_DIRECTIONS = Direction.values()`.
pub const PROPAGATION_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

impl Direction {
    /// Java: `ordinal()`.
    #[inline]
    pub const fn ordinal(self) -> usize {
        self as usize
    }

    /// Java: `getStepX()`.
    #[inline]
    pub const fn step_x(self) -> i32 {
        match self {
            Direction::West => -1,
            Direction::East => 1,
            _ => 0,
        }
    }

    /// Java: `getStepY()`.
    #[inline]
    pub const fn step_y(self) -> i32 {
        match self {
            Direction::Down => -1,
            Direction::Up => 1,
            _ => 0,
        }
    }

    /// Java: `getStepZ()`.
    #[inline]
    pub const fn step_z(self) -> i32 {
        match self {
            Direction::North => -1,
            Direction::South => 1,
            _ => 0,
        }
    }

    /// Java: `getOpposite()`.
    #[inline]
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
}
