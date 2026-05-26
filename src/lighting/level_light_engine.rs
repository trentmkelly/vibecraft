//! Mirrors `net.minecraft.world.level.lighting.LevelLightEngine`.
//!
//! Top-level driver owning an optional block-light engine and an optional
//! sky-light engine, plus a reference to the `LightChunkGetter`. Mirrors the
//! Java API surface that the chunk-status pipeline (`ThreadedLevelLightEngine`)
//! uses: `setLightEnabled`, `propagateLightSources`, `updateSectionStatus`,
//! `runLightUpdates`, `getDataLayerData`, `getRawBrightness`.

use crate::lighting::block_light_engine::BlockLightEngine;
use crate::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::lighting::data_layer::DataLayer;
use crate::lighting::level_height::LevelHeightAccessor;
use crate::lighting::light_chunk::LightChunkGetter;
use crate::lighting::light_layer::LightLayer;
use crate::lighting::positions::{
    block_pos_as_long, section_pos_as_long, section_pos_zero_node,
};
use crate::lighting::sky_light_engine::SkyLightEngine;

/// Java: `LevelLightEngine.LIGHT_SECTION_PADDING = 1`.
pub const LIGHT_SECTION_PADDING: i32 = 1;

/// Java: `LevelLightEngine`.
#[derive(Debug)]
pub struct LevelLightEngine {
    pub block_engine: Option<BlockLightEngine>,
    pub sky_engine: Option<SkyLightEngine>,
    pub level_height: LevelHeightAccessor,
}

impl LevelLightEngine {
    /// Java: `LevelLightEngine(LightChunkGetter, boolean hasBlockLight, boolean hasSkyLight)`.
    pub fn new(level_height: LevelHeightAccessor, has_block_light: bool, has_sky_light: bool) -> Self {
        Self {
            block_engine: has_block_light.then(BlockLightEngine::new),
            sky_engine: has_sky_light
                .then(|| SkyLightEngine::new(ChunkSkyLightSources::new(level_height))),
            level_height,
        }
    }

    /// Java: `checkBlock(BlockPos)`.
    pub fn check_block(&mut self, world_x: i32, world_y: i32, world_z: i32) {
        let node = block_pos_as_long(world_x, world_y, world_z);
        if let Some(engine) = &mut self.block_engine {
            engine.check_block(node);
        }
        if let Some(engine) = &mut self.sky_engine {
            engine.check_block(node);
        }
    }

    /// Java: `hasLightWork`.
    pub fn has_light_work(&self) -> bool {
        self.block_engine
            .as_ref()
            .map(|e| e.has_light_work())
            .unwrap_or(false)
            || self
                .sky_engine
                .as_ref()
                .map(|e| e.has_light_work())
                .unwrap_or(false)
    }

    /// Java: `runLightUpdates`.
    pub fn run_light_updates(&mut self, chunk_source: &dyn LightChunkGetter) -> u64 {
        let mut count = 0;
        if let Some(engine) = &mut self.block_engine {
            count += engine.run_light_updates(chunk_source);
        }
        if let Some(engine) = &mut self.sky_engine {
            count += engine.run_light_updates(chunk_source);
        }
        count
    }

    /// Java: `updateSectionStatus(SectionPos, boolean)`.
    pub fn update_section_status(
        &mut self,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        section_empty: bool,
    ) {
        let node = section_pos_as_long(section_x, section_y, section_z);
        if let Some(engine) = &mut self.block_engine {
            engine.update_section_status(node, section_empty);
        }
        if let Some(engine) = &mut self.sky_engine {
            engine.update_section_status(node, section_empty);
        }
    }

    /// Java: `setLightEnabled(ChunkPos, boolean)`.
    pub fn set_light_enabled(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
        enable: bool,
    ) {
        if let Some(engine) = &mut self.block_engine {
            engine.set_light_enabled(chunk_x, chunk_z, enable);
        }
        if let Some(engine) = &mut self.sky_engine {
            engine.set_light_enabled(chunk_source, chunk_x, chunk_z, enable);
        }
    }

    /// Java: `propagateLightSources(ChunkPos)`.
    pub fn propagate_light_sources(
        &mut self,
        chunk_source: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
    ) {
        if let Some(engine) = &mut self.block_engine {
            engine.propagate_light_sources(chunk_source, chunk_x, chunk_z);
        }
        if let Some(engine) = &mut self.sky_engine {
            engine.propagate_light_sources(chunk_source, chunk_x, chunk_z);
        }
    }

    /// Java: `retainData(ChunkPos, boolean)`.
    pub fn retain_data(&mut self, chunk_x: i32, chunk_z: i32, retain: bool) {
        let zero = section_pos_zero_node(chunk_x, chunk_z);
        if let Some(engine) = &mut self.block_engine {
            engine.base.storage.retain_data(zero, retain);
        }
        if let Some(engine) = &mut self.sky_engine {
            engine.base.storage.retain_data(zero, retain);
        }
    }

    /// Java: `queueSectionData(LightLayer, SectionPos, DataLayer)`.
    pub fn queue_section_data(
        &mut self,
        layer: LightLayer,
        section_x: i32,
        section_y: i32,
        section_z: i32,
        data: Option<DataLayer>,
    ) {
        let node = section_pos_as_long(section_x, section_y, section_z);
        match layer {
            LightLayer::Block => {
                if let Some(engine) = &mut self.block_engine {
                    engine.base.storage.queue_section_data(node, data);
                }
            }
            LightLayer::Sky => {
                if let Some(engine) = &mut self.sky_engine {
                    engine.base.storage.queue_section_data(node, data);
                }
            }
        }
    }

    /// Java: `getRawBrightness(BlockPos, int skyDampen)`.
    pub fn get_raw_brightness(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        sky_dampen: i32,
    ) -> i32 {
        let node = block_pos_as_long(world_x, world_y, world_z);
        let sky_light = self
            .sky_engine
            .as_ref()
            .map(|e| e.get_light_value(node) - sky_dampen)
            .unwrap_or(0);
        let block_light = self
            .block_engine
            .as_ref()
            .map(|e| e.get_light_value(node))
            .unwrap_or(0);
        sky_light.max(block_light)
    }

    /// Visible data layer for a section (used by chunk packet serialization).
    pub fn data_layer_for(
        &self,
        layer: LightLayer,
        section_x: i32,
        section_y: i32,
        section_z: i32,
    ) -> Option<DataLayer> {
        let node = section_pos_as_long(section_x, section_y, section_z);
        match layer {
            LightLayer::Block => self
                .block_engine
                .as_ref()
                .and_then(|e| e.base.storage.snapshot_visible_layer(node)),
            LightLayer::Sky => self
                .sky_engine
                .as_ref()
                .and_then(|e| e.base.storage.snapshot_visible_layer(node)),
        }
    }

    /// Total propagation work performed by the most recent `run_light_updates`
    /// call across both layers; used by the worldgen log line as proof that
    /// the engine ran.
    pub fn last_run_node_count(&self) -> u64 {
        let block = self
            .block_engine
            .as_ref()
            .map(|e| e.last_run_node_count())
            .unwrap_or(0);
        let sky = self
            .sky_engine
            .as_ref()
            .map(|e| e.last_run_node_count())
            .unwrap_or(0);
        block + sky
    }
}
