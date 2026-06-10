//! Shared infrastructure for the block- and sky-light propagators.
//!
//! Mirrors `net.minecraft.world.level.lighting.LightEngine` — the abstract
//! base class that owns the increase/decrease queues, the
//! `blockNodesToCheck` set, and the orchestration of `runLightUpdates`.
//!
//! Each layer engine (block / sky) embeds a [`LightEngineBase`] and supplies
//! its own `check_node` / `propagate_increase` / `propagate_decrease`
//! callbacks through [`run_light_updates`].
//!
//! Java's `lastChunkPos` / `lastChunk` two-entry cache is intentionally
//! omitted: in RustCraft `LightChunkGetter::light_properties_at` is a hot but
//! cheap HashMap lookup, and bringing back the cache requires a borrow-checker
//! dance that would obscure the engine's intent. The cache is purely a
//! microoptimisation in Java and does not affect correctness — adding one
//! later (over `light_properties_at`) is a localized change.

use std::collections::{HashSet, VecDeque};

use crate::lighting::data_layer::MAX_LIGHT_LEVEL;
use crate::lighting::direction::Direction;
use crate::lighting::light_chunk::{LightBlockProperties, LightChunkGetter};
use crate::lighting::positions::{block_pos_as_long, block_pos_offset};
use crate::lighting::queue_entry::{get_from_level, is_increase_from_emission};
use crate::lighting::storage::LayerLightSectionStorage;

/// Java: `LightEngine` abstract base.
#[derive(Debug)]
pub struct LightEngineBase {
    pub storage: LayerLightSectionStorage,
    pub block_nodes_to_check: HashSet<i64>,
    pub increase_queue: VecDeque<(i64, i64)>,
    pub decrease_queue: VecDeque<(i64, i64)>,
    /// Total propagation work performed by the last `run_light_updates` call.
    /// Surfaces in the worldgen log line so future no-op regressions are
    /// caught even when wall-clock timing rounds to zero.
    last_run_node_count: u64,
}

impl LightEngineBase {
    pub fn new(storage: LayerLightSectionStorage) -> Self {
        Self {
            storage,
            block_nodes_to_check: HashSet::new(),
            increase_queue: VecDeque::new(),
            decrease_queue: VecDeque::new(),
            last_run_node_count: 0,
        }
    }

    /// Java: `enqueueIncrease(long, long)`.
    #[inline]
    pub fn enqueue_increase(&mut self, from_node: i64, increase_data: i64) {
        self.increase_queue.push_back((from_node, increase_data));
    }

    /// Java: `enqueueDecrease(long, long)`.
    #[inline]
    pub fn enqueue_decrease(&mut self, from_node: i64, decrease_data: i64) {
        self.decrease_queue.push_back((from_node, decrease_data));
    }

    /// Java: `checkBlock(BlockPos)`.
    #[inline]
    pub fn check_block(&mut self, block_node: i64) {
        self.block_nodes_to_check.insert(block_node);
    }

    /// Java: `hasLightWork`.
    pub fn has_light_work(&self) -> bool {
        self.storage.has_inconsistencies()
            || !self.block_nodes_to_check.is_empty()
            || !self.decrease_queue.is_empty()
            || !self.increase_queue.is_empty()
    }

    pub fn last_run_node_count(&self) -> u64 {
        self.last_run_node_count
    }

    pub fn record_run_count(&mut self, count: u64) {
        self.last_run_node_count = count;
    }
}

/// Java: `LightEngine.getOpacity(BlockState)` — `max(1, lightDampening)`.
#[inline]
pub fn light_engine_get_opacity(state: LightBlockProperties) -> i32 {
    state.opacity.max(1) as i32
}

/// Java: `LightEngine.hasDifferentLightProperties`.
pub fn has_different_light_properties(
    old_state: LightBlockProperties,
    new_state: LightBlockProperties,
) -> bool {
    if old_state == new_state {
        return false;
    }
    old_state.opacity != new_state.opacity
        || old_state.emission != new_state.emission
        || new_state.uses_shape_for_light_occlusion
        || old_state.uses_shape_for_light_occlusion
}

/// Java: `runLightUpdates`. The body is identical across block- and
/// sky-light engines — they only differ in the inner `check_node` /
/// `propagate_*` callbacks.
///
/// Returns the total number of nodes processed across the two queues (Java's
/// `int` return). Stored on the base so the worldgen log line can report it.
pub fn run_light_updates<C, I, D>(
    base: &mut LightEngineBase,
    chunk_source: &dyn LightChunkGetter,
    mut check_node: C,
    mut propagate_increase: I,
    mut propagate_decrease: D,
) -> u64
where
    C: FnMut(&mut LightEngineBase, &dyn LightChunkGetter, i64),
    I: FnMut(&mut LightEngineBase, &dyn LightChunkGetter, i64, i64, i32),
    D: FnMut(&mut LightEngineBase, &dyn LightChunkGetter, i64, i64),
{
    let nodes: Vec<i64> = base.block_nodes_to_check.iter().copied().collect();
    for node in &nodes {
        check_node(base, chunk_source, *node);
    }
    base.block_nodes_to_check.clear();
    let mut count: u64 = 0;
    while let Some((from_node, decrease_data)) = base.decrease_queue.pop_front() {
        propagate_decrease(base, chunk_source, from_node, decrease_data);
        count += 1;
    }
    while let Some((from_node, increase_data)) = base.increase_queue.pop_front() {
        let from_level = base.storage.get_stored_level(from_node);
        let from_target_level = get_from_level(increase_data);
        let mut effective_level = from_level;
        if is_increase_from_emission(increase_data) && from_level < from_target_level {
            base.storage.set_stored_level(from_node, from_target_level);
            effective_level = from_target_level;
        }
        if effective_level == from_target_level {
            propagate_increase(
                base,
                chunk_source,
                from_node,
                increase_data,
                effective_level,
            );
        }
        count += 1;
    }
    base.storage.mark_new_inconsistencies();
    base.storage.swap_section_map();
    base.record_run_count(count);
    count
}

/// Java: `LightEngine.MAX_LEVEL = 15`.
pub const MAX_LEVEL: i32 = MAX_LIGHT_LEVEL as i32;

#[inline]
pub fn neighbour_block_node(from_node: i64, direction: Direction) -> i64 {
    block_pos_offset(
        from_node,
        direction.step_x(),
        direction.step_y(),
        direction.step_z(),
    )
}

#[inline]
pub fn make_block_node(x: i32, y: i32, z: i32) -> i64 {
    block_pos_as_long(x, y, z)
}
