//! Mirrors `net.minecraft.world.level.lighting.ChunkSkyLightSources`.
//!
//! Per-column heightmap of the lowest Y at which sky-light reaches level 15 in
//! a chunk. Java uses a `SimpleBitStorage` of `ceillog2(maxY - minY + 1)` bits
//! per cell × 256 cells; here we simply use a `Vec<i32>` because the engine
//! doesn't read the underlying packed storage shape — only `set` / `get` /
//! `getLowestSourceY` / `getHighestLowestSourceY` are observed.
//!
//! Values are stored in absolute world-block-Y units, with `minY` defined as
//! `levelHeightAccessor.getMinY() - 1` (so the sentinel "no occluder reached"
//! value is one below the world bottom).
//!
//! The implementation mirrors `findLowestSourceY` and `update` from the Java
//! class for parity with vanilla; in practice the chunk-pipeline handoff fills
//! the sources via [`fill_from_columns`], which is the equivalent of Java's
//! `fillFrom(ChunkAccess)` walking each column top-down.

use crate::lighting::level_height::LevelHeightAccessor;

const SIZE: usize = 16;
const COLUMN_COUNT: usize = SIZE * SIZE;
/// Java: `ChunkSkyLightSources.NEGATIVE_INFINITY`. Indicates "no occluder ever
/// reached" for the purpose of `getLowestSourceY`.
pub const NEGATIVE_INFINITY: i32 = i32::MIN;

/// Java: `ChunkSkyLightSources`.
#[derive(Debug, Clone)]
pub struct ChunkSkyLightSources {
    min_y: i32,
    heights: Vec<i32>,
}

impl ChunkSkyLightSources {
    /// Java: `new ChunkSkyLightSources(LevelHeightAccessor)`.
    pub fn new(level: LevelHeightAccessor) -> Self {
        let min_y = level.min_y() - 1;
        Self {
            min_y,
            // Java fills with `0` (= minY); we mirror that so `getLowestSourceY`
            // for an unfilled column returns NEGATIVE_INFINITY.
            heights: vec![0; COLUMN_COUNT],
        }
    }

    /// Total number of columns (always 256).
    pub fn len(&self) -> usize {
        self.heights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heights.is_empty()
    }

    /// Java: `getLowestSourceY(int x, int z)`.
    pub fn get_lowest_source_y(&self, x: i32, z: i32) -> i32 {
        let value = self.heights[Self::index(x, z)] + self.min_y;
        self.extend_sources_below_world(value)
    }

    /// Java: `getHighestLowestSourceY()`.
    pub fn get_highest_lowest_source_y(&self) -> i32 {
        let mut max_value = i32::MIN;
        for &value in &self.heights {
            if value > max_value {
                max_value = value;
            }
        }
        self.extend_sources_below_world(max_value + self.min_y)
    }

    /// Java: private `set(int index, int value)`.
    pub fn set(&mut self, x: i32, z: i32, lowest_source_y: i32) {
        self.heights[Self::index(x, z)] = lowest_source_y - self.min_y;
    }

    /// Java: private `fill(int lowestSourceY)`.
    pub fn fill(&mut self, lowest_source_y: i32) {
        let value = lowest_source_y - self.min_y;
        for cell in self.heights.iter_mut() {
            *cell = value;
        }
    }

    /// Vanilla `fillFrom(ChunkAccess)` walks each column top-down from the
    /// highest non-empty section, looking for the first occluded edge (using
    /// `LightDampening != 0` or merged occlusion shapes).
    ///
    /// This helper accepts a closure that returns the
    /// [`LightBlockProperties`](super::light_chunk::LightBlockProperties) at
    /// the given world coordinates, decoupling this module from any specific
    /// chunk type.
    pub fn fill_from_columns<F>(
        &mut self,
        chunk_min_block_x: i32,
        chunk_min_block_z: i32,
        top_block_y: i32,
        bottom_block_y: i32,
        mut state_at: F,
    ) where
        F: FnMut(i32, i32, i32) -> super::light_chunk::LightBlockProperties,
    {
        if top_block_y < bottom_block_y {
            self.fill(self.min_y);
            return;
        }
        for local_z in 0..16_i32 {
            for local_x in 0..16_i32 {
                let world_x = chunk_min_block_x + local_x;
                let world_z = chunk_min_block_z + local_z;
                let mut top_state = super::light_chunk::LightBlockProperties::AIR;
                let mut top_y = top_block_y + 1;
                let mut bottom_y = top_block_y;
                let mut found = None;
                while bottom_y >= bottom_block_y {
                    let bottom_state = state_at(world_x, bottom_y, world_z);
                    if is_edge_occluded(top_state, bottom_state) {
                        found = Some(top_y);
                        break;
                    }
                    top_state = bottom_state;
                    top_y = bottom_y;
                    bottom_y -= 1;
                }
                let lowest_source_y = found.unwrap_or(self.min_y);
                let clamped = lowest_source_y.max(self.min_y);
                self.set(local_x, local_z, clamped);
            }
        }
    }

    /// Java: `extendSourcesBelowWorld(int)`.
    fn extend_sources_below_world(&self, value: i32) -> i32 {
        if value == self.min_y {
            NEGATIVE_INFINITY
        } else {
            value
        }
    }

    /// Java: `index(int, int)`.
    fn index(x: i32, z: i32) -> usize {
        debug_assert!((0..SIZE as i32).contains(&x));
        debug_assert!((0..SIZE as i32).contains(&z));
        (x + z * SIZE as i32) as usize
    }
}

/// Java: `isEdgeOccluded(BlockState top, BlockState bottom)`.
///
/// Vanilla checks `getLightDampening() != 0` first (i.e. opacity > 0), then
/// falls back to merged occlusion shapes when the bottom has empty opacity
/// (mostly fluids/leaves edge cases). Without voxel shapes we approximate the
/// shape merge using the `occlusion_shape_occludes_full_face` flag.
fn is_edge_occluded(
    top: super::light_chunk::LightBlockProperties,
    bottom: super::light_chunk::LightBlockProperties,
) -> bool {
    if bottom.opacity != 0 {
        return true;
    }
    // Both faces full -> edge sealed.
    let top_face =
        !top.has_empty_occlusion_shape() && top.occlusion_shape_occludes_full_face;
    let bottom_face = !bottom.has_empty_occlusion_shape()
        && bottom.occlusion_shape_occludes_full_face;
    top_face && bottom_face
}
