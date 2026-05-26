//! Rust counterpart to `net.minecraft.world.level.LevelHeightAccessor`,
//! reduced to the fields the lighting engines consult.

/// Java: `LevelHeightAccessor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelHeightAccessor {
    /// Smallest block-Y stored by this dimension (e.g. -64 for the overworld).
    pub min_y: i32,
    /// Vertical block height (e.g. 384 for the overworld -> max_y = 319).
    pub height: i32,
}

impl LevelHeightAccessor {
    pub const fn new(min_y: i32, height: i32) -> Self {
        Self { min_y, height }
    }

    /// Java: `getMinY()`.
    #[inline]
    pub const fn min_y(self) -> i32 {
        self.min_y
    }

    /// Java: `getMaxY()` — inclusive top block of the dimension.
    #[inline]
    pub const fn max_y(self) -> i32 {
        self.min_y + self.height - 1
    }

    /// Java: `getMinSectionY()`.
    #[inline]
    pub const fn min_section_y(self) -> i32 {
        self.min_y >> 4
    }

    /// Java: `getMaxSectionY()` (inclusive section index of the top section).
    #[inline]
    pub const fn max_section_y(self) -> i32 {
        self.max_y() >> 4
    }

    /// Java: `getSectionsCount()`.
    #[inline]
    pub const fn sections_count(self) -> i32 {
        self.max_section_y() - self.min_section_y() + 1
    }
}
