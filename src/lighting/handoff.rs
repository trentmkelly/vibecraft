//! Chunk-pipeline integration: run `LevelLightEngine` against a freshly
//! generated chunk and bake the resulting light arrays back onto each
//! `ChunkSection`.
//!
//! Mirrors `ThreadedLevelLightEngine.initializeLight(chunk, lighted)` followed
//! by `lightChunk(chunk, lighted)`:
//!
//! 1. For every non-empty section of `chunk`, call
//!    `LevelLightEngine.updateSectionStatus(..., false)`.
//! 2. Call `setLightEnabled(pos, lighted)`. We always invoke this with
//!    `lighted = false` because we are computing light from scratch.
//! 3. Call `propagateLightSources(pos)`.
//! 4. Run light updates to drain the queues.
//! 5. Read the visible data layers back out of the engine and write them onto
//!    the chunk's [`ChunkSection`] structs.
//! 6. Mark `chunk.light_correct = true`.

use crate::lighting::chunk_source::LevelChunkLightChunkGetter;
use crate::lighting::data_layer::DataLayer;
use crate::lighting::level_height::LevelHeightAccessor;
use crate::lighting::level_light_engine::LevelLightEngine;
use crate::lighting::light_layer::LightLayer;
use crate::storage::chunk::LevelChunk;

/// Result of running the engine over a single chunk; surfaces the per-layer
/// node counts so callers can log them as proof that propagation ran.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkLightingResult {
    pub block_nodes_propagated: u64,
    pub sky_nodes_propagated: u64,
}

impl ChunkLightingResult {
    pub fn total(self) -> u64 {
        self.block_nodes_propagated + self.sky_nodes_propagated
    }
}

/// Compute lighting for a single chunk and bake the resulting block/sky light
/// arrays back onto each `ChunkSection`.
///
/// `level_height` describes the vertical extent of the dimension the chunk
/// belongs to (overworld defaults: `min_y = -64`, `height = 384`).
///
/// `has_sky_light` mirrors Java's `DimensionType.hasSkyLight()` flag — only
/// the overworld and superflat have sky light.
pub fn compute_chunk_lighting(
    chunk: &mut LevelChunk,
    level_height: LevelHeightAccessor,
    has_sky_light: bool,
) -> ChunkLightingResult {
    let chunk_pos = chunk.pos;
    let mut engine = LevelLightEngine::new(level_height, true, has_sky_light);

    // Wrap the chunk for lookups. The borrow is dropped before we mutate
    // the chunk again.
    let result;
    {
        let chunks_slice: [&LevelChunk; 1] = [chunk];
        let chunks: &[&LevelChunk] = &chunks_slice;
        let getter = LevelChunkLightChunkGetter::new(level_height, chunks);

        // Step 1: announce non-empty sections.
        for (section_index, section_y) in
            (level_height.min_section_y()..=level_height.max_section_y()).enumerate()
        {
            let _ = section_index;
            let section_empty = !chunk
                .sections
                .iter()
                .any(|section| i32::from(section.y) == section_y && !section_has_only_air(section));
            engine.update_section_status(chunk_pos.x, section_y, chunk_pos.z, section_empty);
        }

        // Step 2: enable light. We never run with `lighted = true` here
        // because the input chunk was just generated.
        engine.set_light_enabled(&getter, chunk_pos.x, chunk_pos.z, false);

        // Step 3: seed sky-light source columns and block-light emitters.
        engine.propagate_light_sources(&getter, chunk_pos.x, chunk_pos.z);

        // Step 4: drain queues.
        engine.run_light_updates(&getter);

        result = ChunkLightingResult {
            block_nodes_propagated: engine
                .block_engine
                .as_ref()
                .map(|e| e.last_run_node_count())
                .unwrap_or(0),
            sky_nodes_propagated: engine
                .sky_engine
                .as_ref()
                .map(|e| e.last_run_node_count())
                .unwrap_or(0),
        };
    }

    // Step 5: bake per-section data layers back onto the chunk.
    // Engines store light for sections in `[min_section_y - 1, max_section_y + 1]`
    // (the `LIGHT_SECTION_PADDING = 1` rows above/below); the chunk packet only
    // ships the in-world sections, so we only copy those.
    for section in chunk.sections.iter_mut() {
        let section_y = i32::from(section.y);
        section.block_light = engine
            .data_layer_for(LightLayer::Block, chunk_pos.x, section_y, chunk_pos.z)
            .map(DataLayer::into_bytes)
            .map(byte_vec_to_signed);
        if has_sky_light {
            section.sky_light = engine
                .data_layer_for(LightLayer::Sky, chunk_pos.x, section_y, chunk_pos.z)
                .map(DataLayer::into_bytes)
                .map(byte_vec_to_signed);
        } else {
            section.sky_light = None;
        }
    }

    // Step 6: Java's `ThreadedLevelLightEngine.lightChunk` sets
    // `setLightCorrect(true)` unconditionally after `propagateLightSources`
    // and `runLightUpdates` drain successfully. Empty sections legitimately
    // leave both `block_light` and `sky_light` as `None`; the chunk packet
    // serializer reports that by emitting neither a "has data" bit nor an
    // "empty" bit for the section, which the vanilla client interprets as
    // "default for the dimension" — sky=15 above the topmost stored
    // section, sky=0 below.
    chunk.light_correct = true;

    result
}

fn section_has_only_air(section: &crate::storage::chunk::ChunkSection) -> bool {
    // Mirrors `LevelChunkSection.hasOnlyAir()` via the palette only. We avoid
    // an `unpack_palette_indices` pass over every voxel: if the palette
    // contains exclusively air-name entries, the section is air-only.
    use crate::storage::chunk::PalettedContainer;
    use crate::storage::chunk::SECTION_VOLUME;
    let Ok(container) = PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME) else {
        return true;
    };
    container.palette.iter().all(|entry| {
        let name = match entry {
            Tag::Compound(fields) => fields
                .iter()
                .find(|(k, _)| k == "Name")
                .and_then(|(_, v)| match v {
                    Tag::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("minecraft:air"),
            Tag::String(s) => s.as_str(),
            _ => "minecraft:air",
        };
        matches!(
            name,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        )
    })
}

fn byte_vec_to_signed(bytes: Vec<u8>) -> Vec<i8> {
    bytes.into_iter().map(|b| b as i8).collect()
}

use crate::storage::nbt::Tag;
