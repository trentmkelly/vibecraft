#![allow(dead_code, unused_imports)]
//! Cross-section, cross-chunk lighting engine.
//!
//! This module ports the 26.1.2 vanilla light pipeline from Java:
//!
//! ```text
//! +-----------------------+        +-------------------------------+
//! | LevelLightEngine      |  ───>  | LightEngineBase (block / sky) |
//! +-----------------------+        +-------------------------------+
//!          │                                  │
//!          │                                  ▼
//!          │                       +-------------------------------+
//!          │                       | LayerLightSectionStorage      |
//!          │                       | (block + sky behaviours via   |
//!          │                       |  the LightLayer enum)         |
//!          │                       +-------------------------------+
//!          ▼
//! +-----------------------+        +-------------------------------+
//! | LightChunkGetter      |  ───>  | LevelChunk (real chunk data)  |
//! +-----------------------+        +-------------------------------+
//! ```
//!
//! Entry point for the chunk pipeline is
//! [`handoff::compute_chunk_lighting`], which mirrors
//! `ThreadedLevelLightEngine.initializeLight(chunk, lighted)` followed by
//! `lightChunk(chunk, lighted)` and writes the resulting `DataLayer`s back
//! onto each [`crate::storage::chunk::ChunkSection`].

pub mod block_light_engine;
pub mod block_light_properties;
pub mod chunk_sky_light_sources;
pub mod chunk_source;
pub mod data_layer;
pub mod direction;
pub mod handoff;
pub mod level_height;
pub mod level_light_engine;
pub mod light_chunk;
pub mod light_engine;
pub mod light_layer;
pub mod positions;
pub mod queue_entry;
pub mod sky_light_engine;
pub mod storage;

#[cfg(test)]
pub mod tests;

pub use block_light_engine::BlockLightEngine;
pub use chunk_sky_light_sources::ChunkSkyLightSources;
pub use data_layer::DataLayer;
pub use handoff::{compute_chunk_lighting, ChunkLightingResult};
pub use level_height::LevelHeightAccessor;
pub use level_light_engine::LevelLightEngine;
pub use light_chunk::{LightBlockProperties, LightChunkGetter};
pub use light_layer::LightLayer;
pub use sky_light_engine::SkyLightEngine;
