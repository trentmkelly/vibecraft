//! `LightChunkGetter` implementations over real chunk data.
//!
//! [`LevelChunkLightChunkGetter`] wraps a borrowed slice of generated chunks
//! and is the adapter used at the chunk-pipeline boundary. It is responsible
//! for:
//!
//! - Translating world `(x, y, z)` to the correct chunk's local lookup, with a
//!   Bedrock-default fallback when the requested chunk is not in the slice.
//! - Lazily computing the per-chunk [`ChunkSkyLightSources`] heightmap on
//!   first request, by walking each column top-down through the chunk's block
//!   palette and finding the lowest sky-source Y. This mirrors
//!   `ChunkSkyLightSources.fillFrom(ChunkAccess)`.
//! - Enumerating emissive block-state palette entries via
//!   [`find_block_light_sources`], matching `LightChunk.findBlockLightSources`.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::lighting::block_light_properties::light_properties_for;
use crate::lighting::chunk_sky_light_sources::ChunkSkyLightSources;
use crate::lighting::level_height::LevelHeightAccessor;
use crate::lighting::light_chunk::{
    BlockLightSourceConsumer, LightBlockProperties, LightChunkGetter,
};
use crate::storage::chunk::LevelChunk;
use crate::storage::region::ChunkPos;

/// A `LightChunkGetter` over an owned set of generated chunks.
///
/// The adapter pre-indexes the chunks by `ChunkPos` so neighbour lookups are
/// O(1), and caches the computed `ChunkSkyLightSources` heightmaps once per
/// chunk for the lifetime of the adapter.
pub struct LevelChunkLightChunkGetter<'a> {
    chunks: HashMap<ChunkPos, &'a LevelChunk>,
    level_height: LevelHeightAccessor,
    sky_sources_cache: RefCell<HashMap<ChunkPos, ChunkSkyLightSources>>,
}

impl<'a> LevelChunkLightChunkGetter<'a> {
    pub fn new(level_height: LevelHeightAccessor, chunks: &'a [&'a LevelChunk]) -> Self {
        let mut map = HashMap::new();
        for chunk in chunks {
            map.insert(chunk.pos, *chunk);
        }
        Self {
            chunks: map,
            level_height,
            sky_sources_cache: RefCell::new(HashMap::new()),
        }
    }

    fn chunk_at(&self, chunk_x: i32, chunk_z: i32) -> Option<&LevelChunk> {
        self.chunks
            .get(&ChunkPos {
                x: chunk_x,
                z: chunk_z,
            })
            .copied()
    }
}

impl<'a> LightChunkGetter for LevelChunkLightChunkGetter<'a> {
    fn light_properties_at(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> LightBlockProperties {
        let chunk_x = world_x.div_euclid(16);
        let chunk_z = world_z.div_euclid(16);
        let Some(chunk) = self.chunk_at(chunk_x, chunk_z) else {
            return LightBlockProperties::BEDROCK_FALLBACK;
        };
        chunk
            .get_block_state_name(world_x, world_y, world_z)
            .map(light_properties_for)
            .unwrap_or(LightBlockProperties::AIR)
    }

    fn find_block_light_sources(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        consumer: &mut dyn BlockLightSourceConsumer,
    ) {
        let Some(chunk) = self.chunk_at(chunk_x, chunk_z) else {
            return;
        };
        let chunk_min_x = chunk_x * 16;
        let chunk_min_z = chunk_z * 16;
        for section in &chunk.sections {
            let section_min_y = i32::from(section.y) * 16;
            for local_y in 0..16 {
                let world_y = section_min_y + local_y;
                for local_z in 0..16 {
                    let world_z = chunk_min_z + local_z;
                    for local_x in 0..16 {
                        let world_x = chunk_min_x + local_x;
                        let Some(name) = chunk.get_block_state_name(world_x, world_y, world_z)
                        else {
                            continue;
                        };
                        let props = light_properties_for(name);
                        if props.emission > 0 {
                            consumer.accept(world_x, world_y, world_z, props.emission);
                        }
                    }
                }
            }
        }
    }

    fn sky_light_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<ChunkSkyLightSources> {
        if let Some(cached) = self.sky_sources_cache.borrow().get(&ChunkPos {
            x: chunk_x,
            z: chunk_z,
        }) {
            return Some(cached.clone());
        }
        let chunk = self.chunk_at(chunk_x, chunk_z)?;
        let chunk_min_x = chunk_x * 16;
        let chunk_min_z = chunk_z * 16;
        let top_block_y = self.level_height.max_y();
        let bottom_block_y = self.level_height.min_y();
        let mut sources = ChunkSkyLightSources::new(self.level_height);
        sources.fill_from_columns(
            chunk_min_x,
            chunk_min_z,
            top_block_y,
            bottom_block_y,
            |x, y, z| {
                let name = chunk
                    .get_block_state_name(x, y, z)
                    .unwrap_or("minecraft:air");
                light_properties_for(name)
            },
        );
        self.sky_sources_cache.borrow_mut().insert(
            ChunkPos {
                x: chunk_x,
                z: chunk_z,
            },
            sources.clone(),
        );
        Some(sources)
    }

    fn level(&self) -> LevelHeightAccessor {
        self.level_height
    }
}
