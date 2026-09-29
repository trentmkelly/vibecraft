//! Live wiring of [`crate::random_tick_behaviors`]: adapts the
//! generated-chunk cache to [`RandomTickWorld`] for the random-tick pass of
//! the shared world tick.
//!
//! Every `Level.setBlock` a `randomTick` performs mutates the cache, is
//! written as a block update on the shared frame buffer when the flags ask for
//! a client update (flag 2), and is queued for the `updateNeighborShapes`
//! cascade when they ask for a neighbour update (flag 1).

use std::io::{self, Write};

use super::block_placement_live::{write_block_update, LiveBlockWorld};
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::random_tick_behaviors::{RandomTickWorld, UPDATE_CLIENTS, UPDATE_NEIGHBORS};
use crate::storage::chunk::HeightmapKind;

/// `Level.isRainingAt(pos)` against the chunk cache: it must be raining and
/// the `MOTION_BLOCKING` height must not be above `pos`.
///
/// TODO(precipitation-biome): Java also requires the biome's
/// `getPrecipitationAt(pos)` to be rain (snow and dry biomes are excluded).
pub(super) fn is_raining_at(world: &LiveBlockWorld<'_>, raining: bool, pos: BlockPos) -> bool {
    if !raining {
        return false;
    }
    let chunk = world.cache.get_or_load(
        pos.x.div_euclid(16),
        pos.z.div_euclid(16),
        world.layout.root(),
        world.seed,
    );
    chunk
        .heightmap_value(
            HeightmapKind::MotionBlocking,
            pos.x.rem_euclid(16) as usize,
            pos.z.rem_euclid(16) as usize,
        )
        .is_none_or(|height| height <= pos.y)
}

/// Server state a `randomTick` reads that is not stored in the chunks.
#[derive(Clone, Copy)]
pub(super) struct RandomTickEnvironment {
    /// `Level.isRaining()`.
    pub raining: bool,
    /// `GameRules.SPREAD_VINES`.
    pub spread_vines: bool,
}

/// [`RandomTickWorld`] over the live chunk cache.
pub(super) struct LiveRandomTickWorld<'a, 'b, W: Write> {
    inner: LiveBlockWorld<'a>,
    writer: &'b mut W,
    compression: CompressionState,
    environment: RandomTickEnvironment,
    /// Positions whose neighbours need the `updateShape` cascade.
    pub changed: Vec<BlockPos>,
    /// First write error (the trait methods cannot fail).
    pub error: Option<io::Error>,
}

impl<'a, 'b, W: Write> LiveRandomTickWorld<'a, 'b, W> {
    pub fn new(
        inner: LiveBlockWorld<'a>,
        writer: &'b mut W,
        compression: CompressionState,
        environment: RandomTickEnvironment,
    ) -> Self {
        Self {
            inner,
            writer,
            compression,
            environment,
            changed: Vec::new(),
            error: None,
        }
    }
}

impl<W: Write> SurvivalWorld for LiveRandomTickWorld<'_, '_, W> {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.inner.state_at(pos)
    }

    fn raw_brightness(&self, pos: BlockPos) -> i32 {
        self.inner.raw_brightness(pos)
    }
}

impl<W: Write> RandomTickWorld for LiveRandomTickWorld<'_, '_, W> {
    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32) {
        // `Level.setBlock` returns early when the state is unchanged.
        if self.inner.state_at(pos) == state {
            return;
        }
        let name = state.state_name();
        self.inner
            .cache
            .set_block(self.inner.layout.root(), self.inner.seed, pos, &name);
        if flags & UPDATE_CLIENTS != 0 {
            let id = crate::block_states::network_id_for_block_state(&name).unwrap_or(0);
            if let Err(err) = write_block_update(self.writer, self.compression, pos, id) {
                self.error.get_or_insert(err);
            }
        }
        if flags & UPDATE_NEIGHBORS != 0 {
            self.changed.push(pos);
        }
    }

    fn max_local_raw_brightness(&self, pos: BlockPos) -> i32 {
        // TODO(live-light-query): `getMaxLocalRawBrightness` is
        // `max(skyLight - skyDarken, blockLight)`; until the lighting engine
        // is queryable from the live world this shares the constant full
        // daylight of `raw_brightness`.
        self.inner.raw_brightness(pos)
    }

    fn is_raining_at(&self, pos: BlockPos) -> bool {
        is_raining_at(&self.inner, self.environment.raining, pos)
    }

    fn spread_vines(&self) -> bool {
        self.environment.spread_vines
    }

    fn min_y(&self) -> i32 {
        crate::world::OVERWORLD_MIN_Y
    }

    fn max_y(&self) -> i32 {
        crate::world::OVERWORLD_MIN_Y + crate::world::OVERWORLD_LEVEL_HEIGHT - 1
    }
}
