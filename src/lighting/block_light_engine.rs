//! Mirrors `net.minecraft.world.level.lighting.BlockLightEngine`.
//!
//! Block-light propagation: increase BFS from emissive sources (torches,
//! glowstone, …) and decrease BFS when an emitter is removed. Opacity is read
//! from each visited block's `LightBlockProperties` via `getLightBlockInto`.

use crate::lighting::data_layer::MAX_LIGHT_LEVEL;
use crate::lighting::direction::PROPAGATION_DIRECTIONS;
use crate::lighting::light_chunk::{shape_occludes, LightBlockProperties, LightChunkGetter};
use crate::lighting::light_engine::{
    light_engine_get_opacity, neighbour_block_node, run_light_updates, LightEngineBase,
};
use crate::lighting::positions::block_to_section;
use crate::lighting::queue_entry::{
    decrease_all_directions, decrease_skip_one_direction, get_from_level,
    increase_light_from_emission, increase_only_one_direction, increase_skip_one_direction,
    is_from_empty_shape, pull_light_in_entry, should_propagate_in_direction,
};
use crate::lighting::storage::LayerLightSectionStorage;

/// Java: `BlockLightEngine`.
#[derive(Debug)]
pub struct BlockLightEngine {
    pub base: LightEngineBase,
}

impl BlockLightEngine {
    pub fn new() -> Self {
        Self {
            base: LightEngineBase::new(LayerLightSectionStorage::new_block()),
        }
    }

    pub fn check_block(&mut self, block_node: i64) {
        self.base.check_block(block_node);
    }

    pub fn has_light_work(&self) -> bool {
        self.base.has_light_work()
    }

    pub fn last_run_node_count(&self) -> u64 {
        self.base.last_run_node_count()
    }

    pub fn run_light_updates(&mut self, chunk_source: &dyn LightChunkGetter) -> u64 {
        run_light_updates(
            &mut self.base,
            chunk_source,
            Self::check_node,
            Self::propagate_increase,
            Self::propagate_decrease,
        )
    }

    /// Java: `BlockLightEngine.propagateLightSources(ChunkPos)`.
    pub fn propagate_light_sources(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
    ) {
        // Java enables light first; this mirrors `setLightEnabled(pos, true)`.
        let zero_node = crate::lighting::positions::section_pos_zero_node(chunk_x, chunk_z);
        self.base.storage.set_light_enabled(zero_node, true);
        chunk_source.find_block_light_sources(
            chunk_x,
            chunk_z,
            &mut |wx, wy, wz, emission| {
                if emission == 0 {
                    return;
                }
                let block_node = crate::lighting::positions::block_pos_as_long(wx, wy, wz);
                let props = chunk_source.light_properties_at(wx, wy, wz);
                let entry =
                    increase_light_from_emission(emission as i32, props.has_empty_occlusion_shape());
                self.base.enqueue_increase(block_node, entry);
            },
        );
    }

    fn emission_for(
        _chunk_source: &dyn LightChunkGetter,
        storage: &LayerLightSectionStorage,
        block_node: i64,
        state: LightBlockProperties,
    ) -> i32 {
        let emission = state.emission as i32;
        if emission > 0 && storage.light_on_in_section(block_to_section(block_node)) {
            emission
        } else {
            0
        }
    }

    fn check_node(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        block_node: i64,
    ) {
        let section_node = block_to_section(block_node);
        if !base.storage.storing_light_for_section(section_node) {
            return;
        }
        let state = chunk_source.light_properties_at(
            crate::lighting::positions::block_pos_x(block_node),
            crate::lighting::positions::block_pos_y(block_node),
            crate::lighting::positions::block_pos_z(block_node),
        );
        let emission = Self::emission_for(chunk_source, &base.storage, block_node, state);
        let old_level = base.storage.get_stored_level(block_node);
        if emission < old_level {
            base.storage.set_stored_level(block_node, 0);
            base.enqueue_decrease(block_node, decrease_all_directions(old_level));
        } else {
            base.enqueue_decrease(block_node, pull_light_in_entry());
        }
        if emission > 0 {
            base.enqueue_increase(
                block_node,
                increase_light_from_emission(emission, state.has_empty_occlusion_shape()),
            );
        }
    }

    fn propagate_increase(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        increase_data: i64,
        from_level: i32,
    ) {
        let mut from_state: Option<LightBlockProperties> = None;
        for direction in PROPAGATION_DIRECTIONS {
            if !should_propagate_in_direction(increase_data, direction) {
                continue;
            }
            let to_node = neighbour_block_node(from_node, direction);
            let to_section = block_to_section(to_node);
            if !base.storage.storing_light_for_section(to_section) {
                continue;
            }
            let to_level = base.storage.get_stored_level(to_node);
            let max_possible_new_to_level = from_level - 1;
            if max_possible_new_to_level <= to_level {
                continue;
            }
            let to_x = crate::lighting::positions::block_pos_x(to_node);
            let to_y = crate::lighting::positions::block_pos_y(to_node);
            let to_z = crate::lighting::positions::block_pos_z(to_node);
            let to_state = chunk_source.light_properties_at(to_x, to_y, to_z);
            let to_opacity = light_engine_get_opacity(to_state);
            let new_to_level = from_level - to_opacity;
            if new_to_level <= to_level {
                continue;
            }
            let from_state = *from_state.get_or_insert_with(|| {
                if is_from_empty_shape(increase_data) {
                    LightBlockProperties::AIR
                } else {
                    let fx = crate::lighting::positions::block_pos_x(from_node);
                    let fy = crate::lighting::positions::block_pos_y(from_node);
                    let fz = crate::lighting::positions::block_pos_z(from_node);
                    chunk_source.light_properties_at(fx, fy, fz)
                }
            });
            // Java's `propagateIncrease` performs a separate `shapeOccludes`
            // check independent of `getLightBlockInto`; if the merged
            // occlusion shapes seal the face, the propagation is blocked even
            // though `getOpacity` reported a finite cost.
            if shape_occludes(from_state, to_state, direction) {
                continue;
            }
            base.storage.set_stored_level(to_node, new_to_level);
            if new_to_level > 1 {
                base.enqueue_increase(
                    to_node,
                    increase_skip_one_direction(
                        new_to_level,
                        to_state.has_empty_occlusion_shape(),
                        direction.opposite(),
                    ),
                );
            }
        }
    }

    fn propagate_decrease(
        base: &mut LightEngineBase,
        chunk_source: &dyn LightChunkGetter,
        from_node: i64,
        decrease_data: i64,
    ) {
        let old_from_level = get_from_level(decrease_data);
        for direction in PROPAGATION_DIRECTIONS {
            if !should_propagate_in_direction(decrease_data, direction) {
                continue;
            }
            let to_node = neighbour_block_node(from_node, direction);
            let to_section = block_to_section(to_node);
            if !base.storage.storing_light_for_section(to_section) {
                continue;
            }
            let to_level = base.storage.get_stored_level(to_node);
            if to_level == 0 {
                continue;
            }
            if to_level < old_from_level {
                let to_x = crate::lighting::positions::block_pos_x(to_node);
                let to_y = crate::lighting::positions::block_pos_y(to_node);
                let to_z = crate::lighting::positions::block_pos_z(to_node);
                let to_state = chunk_source.light_properties_at(to_x, to_y, to_z);
                let to_emission =
                    Self::emission_for(chunk_source, &base.storage, to_node, to_state);
                base.storage.set_stored_level(to_node, 0);
                if to_emission < to_level {
                    base.enqueue_decrease(
                        to_node,
                        decrease_skip_one_direction(to_level, direction.opposite()),
                    );
                }
                if to_emission > 0 {
                    base.enqueue_increase(
                        to_node,
                        increase_light_from_emission(
                            to_emission,
                            to_state.has_empty_occlusion_shape(),
                        ),
                    );
                }
            } else {
                base.enqueue_increase(
                    to_node,
                    increase_only_one_direction(to_level, false, direction.opposite()),
                );
            }
        }
    }

    /// Java: `LayerLightEventListener.getLightValue(BlockPos)` for block light.
    pub fn get_light_value(&self, block_node: i64) -> i32 {
        self.base.storage.get_light_value(block_node)
    }

    /// Java: `updateSectionStatus(SectionPos, boolean)`.
    pub fn update_section_status(&mut self, section_node: i64, section_empty: bool) {
        self.base
            .storage
            .update_section_status(section_node, section_empty);
    }

    /// Java: `setLightEnabled(ChunkPos, boolean)`.
    pub fn set_light_enabled(&mut self, chunk_x: i32, chunk_z: i32, enable: bool) {
        let zero = crate::lighting::positions::section_pos_zero_node(chunk_x, chunk_z);
        self.base.storage.set_light_enabled(zero, enable);
    }
}

impl Default for BlockLightEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximum block-light level emitted by any source (= 15).
pub const BLOCK_MAX_LIGHT_LEVEL: u8 = MAX_LIGHT_LEVEL;
