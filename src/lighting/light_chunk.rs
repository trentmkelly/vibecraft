//! Rust counterparts to `net.minecraft.world.level.chunk.LightChunk` and
//! `net.minecraft.world.level.chunk.LightChunkGetter`.
//!
//! Vanilla splits chunk-source lookups into "fetch the chunk pointer" then
//! "query the chunk" because Java's `BlockState` reference travels through
//! multiple subsystems. In Rust there is no concrete `BlockState` value to
//! share, so we collapse the two interfaces into a single `LightChunkGetter`
//! that answers world-coordinate questions directly and falls back to a
//! Bedrock-default opaque state when no chunk is loaded — exactly mirroring
//! `LightEngine.getState(BlockPos)`.

use crate::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::lighting::data_layer::MAX_LIGHT_LEVEL;
use crate::lighting::direction::Direction;
use crate::lighting::level_height::LevelHeightAccessor;

/// The light-relevant subset of a block state. Constructed via
/// [`crate::lighting::block_light_properties::light_properties_for`] from the
/// authoritative per-state tables in [`crate::block_properties`].
///
/// Java equivalents:
/// - `opacity` -> `BlockState.getLightDampening()`.
/// - `emission` -> `BlockState.getLightEmission()`.
/// - `can_occlude` -> `BlockState.canOcclude()`.
/// - `uses_shape_for_light_occlusion` -> `BlockState.useShapeForLightOcclusion()`.
/// - `occlusion` -> the state's full occlusion shape
///   (`BlockState.getOcclusionShape()`), as a position in the occlusion-shape
///   universe of [`crate::block_properties`], whose vendored matrices evaluate
///   `Shapes.mergedFaceOccludes` / `Shapes.faceShapeOccludes` exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightBlockProperties {
    pub opacity: u8,
    pub emission: u8,
    pub can_occlude: bool,
    pub uses_shape_for_light_occlusion: bool,
    pub occlusion: u16,
}

impl LightBlockProperties {
    /// Java: `Blocks.BEDROCK.defaultBlockState()` — fallback when a chunk
    /// pointer is null. Bedrock is `canOcclude` but not
    /// `useShapeForLightOcclusion`, so its occlusion shape is empty for the
    /// light engine; full darkness comes from `opacity == 15`.
    pub const BEDROCK_FALLBACK: Self = Self {
        opacity: MAX_LIGHT_LEVEL,
        emission: 0,
        can_occlude: true,
        uses_shape_for_light_occlusion: false,
        occlusion: crate::block_properties::EMPTY_OCCLUSION_POSITION,
    };

    /// Pure air.
    pub const AIR: Self = Self {
        opacity: 0,
        emission: 0,
        can_occlude: false,
        uses_shape_for_light_occlusion: false,
        occlusion: crate::block_properties::EMPTY_OCCLUSION_POSITION,
    };

    /// Java: `LightEngine.isEmptyShape(BlockState)` —
    /// `!state.canOcclude() || !state.useShapeForLightOcclusion()`.
    #[inline]
    pub fn has_empty_occlusion_shape(self) -> bool {
        !self.can_occlude || !self.uses_shape_for_light_occlusion
    }

    /// The occlusion shape `LightEngine.getOcclusionShape(state, ...)` sees:
    /// the empty shape when [`Self::has_empty_occlusion_shape`], otherwise the
    /// state's real occlusion shape.
    #[inline]
    pub fn gated_occlusion(self) -> u16 {
        if self.has_empty_occlusion_shape() {
            crate::block_properties::EMPTY_OCCLUSION_POSITION
        } else {
            self.occlusion
        }
    }
}

/// Java: `LightChunk.findBlockLightSources` consumer.
pub trait BlockLightSourceConsumer {
    fn accept(&mut self, world_x: i32, world_y: i32, world_z: i32, emission: u8);
}

impl<F> BlockLightSourceConsumer for F
where
    F: FnMut(i32, i32, i32, u8),
{
    fn accept(&mut self, world_x: i32, world_y: i32, world_z: i32, emission: u8) {
        self(world_x, world_y, world_z, emission)
    }
}

/// Java fusion of `LightChunk` + `LightChunkGetter`.
///
/// All methods accept world coordinates and silently substitute the
/// Bedrock-default fallback for missing chunks; this keeps the engines free of
/// `null` checks scattered throughout propagation.
pub trait LightChunkGetter {
    /// Java: `LightChunk.getBlockState(BlockPos)`. Returns
    /// [`LightBlockProperties::BEDROCK_FALLBACK`] if the chunk is missing.
    fn light_properties_at(&self, world_x: i32, world_y: i32, world_z: i32)
        -> LightBlockProperties;

    /// Java: `LightChunk.findBlockLightSources(BiConsumer)` against the chunk
    /// at `(chunk_x, chunk_z)`. Implementations should silently do nothing if
    /// the chunk is missing.
    fn find_block_light_sources(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        consumer: &mut dyn BlockLightSourceConsumer,
    );

    /// Java: `LightChunk.getSkyLightSources()` for `(chunk_x, chunk_z)`.
    /// Returns `None` when the chunk is unloaded; the sky engine substitutes
    /// an empty `ChunkSkyLightSources` in that case.
    fn sky_light_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<ChunkSkyLightSources>;

    /// Java: `LightChunkGetter.getLevel()`.
    fn level(&self) -> LevelHeightAccessor;
}

/// Java: `LightEngine.getLightBlockInto(BlockState, BlockState, Direction, int)`.
///
/// `Shapes.mergedFaceOccludes` is evaluated through the vendored exact matrices
/// over full occlusion shapes (with the `isEmptyShape` substitution to
/// `Shapes.empty()` exactly as Java does at lines 57-58).
pub fn get_light_block_into(
    from: LightBlockProperties,
    to: LightBlockProperties,
    direction: Direction,
    simple_opacity: i32,
) -> i32 {
    let from_empty = from.has_empty_occlusion_shape();
    let to_empty = to.has_empty_occlusion_shape();
    if from_empty && to_empty {
        return simple_opacity;
    }
    let from_shape = if from_empty {
        crate::block_properties::EMPTY_OCCLUSION_POSITION
    } else {
        from.occlusion
    };
    let to_shape = if to_empty {
        crate::block_properties::EMPTY_OCCLUSION_POSITION
    } else {
        to.occlusion
    };
    if crate::block_properties::merged_face_occludes(from_shape, to_shape, direction.ordinal()) {
        16
    } else {
        simple_opacity
    }
}

/// Java: `LightEngine.shapeOccludes(BlockState, BlockState, Direction)` —
/// `Shapes.faceShapeOccludes(getOcclusionShape(from, dir),
/// getOcclusionShape(to, dir.opposite))`, exact via the vendored face-shape
/// matrices.
pub fn shape_occludes(
    from: LightBlockProperties,
    to: LightBlockProperties,
    direction: Direction,
) -> bool {
    crate::block_properties::face_shape_occludes(
        from.gated_occlusion(),
        direction.ordinal(),
        to.gated_occlusion(),
        direction.opposite().ordinal(),
    )
}
