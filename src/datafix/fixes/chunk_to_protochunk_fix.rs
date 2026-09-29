//! Port of `net.minecraft.util.datafix.fixes.ChunkToProtochunkFix`.

use crate::datafix::dynamic::{
    as_byte_buffer, as_f64, get, get_bool_or, get_i32_or, get_mut, list_items, remove, set,
};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::write_and_read_fix::write_fix_and_read_fix;

const NUM_SECTIONS: usize = 16;

/// `new ChunkToProtochunkFix(schema, changesType)`: chunks gain a generation
/// `Status`, packed biomes and packed pending ticks.
pub fn fix() -> Fix {
    write_fix_and_read_fix("ChunkToProtoChunkFix", r::CHUNK, |chunk| {
        if let Some(level) = get_mut(chunk, "Level") {
            fix_chunk_data(level);
        }
    })
}

fn fix_chunk_data(tag: &mut Tag) {
    let terrain_populated = get_bool_or(tag, "TerrainPopulated", false);
    let light_populated = get(tag, "LightPopulated").and_then(as_f64).is_none()
        || get_bool_or(tag, "LightPopulated", false);
    let status = if terrain_populated {
        if light_populated {
            "mobs_spawned"
        } else {
            "decorated"
        }
    } else {
        "carved"
    };
    repack_biomes(tag);
    repack_ticks(tag);
    set(tag, "Status", Tag::String(status.to_string()));
    set(tag, "hasLegacyStructureData", Tag::Byte(1));
}

/// `ChunkToProtochunkFix.repackBiomes`: the 256 biome bytes become an int array.
fn repack_biomes(tag: &mut Tag) {
    let Some(biomes) = get_mut(tag, "Biomes") else {
        return;
    };
    let Some(bytes) = as_byte_buffer(biomes) else {
        return;
    };
    let repacked = (0..256)
        .map(|index| bytes.get(index).map_or(0, |byte| i32::from(*byte) & 255))
        .collect();
    *biomes = Tag::IntArray(repacked);
}

/// `ChunkToProtochunkFix.repackTicks`: `TileTicks` become per-section `ToBeTicked` lists.
fn repack_ticks(tag: &mut Tag) {
    let Some(ticks) = get(tag, "TileTicks").and_then(list_items) else {
        return;
    };
    let mut to_be_ticked: Vec<Vec<i16>> = vec![Vec::new(); NUM_SECTIONS];
    for tick in &ticks {
        let x = get_i32_or(tick, "x", 0);
        let y = get_i32_or(tick, "y", 0);
        let z = get_i32_or(tick, "z", 0);
        // Java indexes `toBeTickedTag.get(y >> 4)`, which throws outside 0..16.
        let Ok(section) = usize::try_from(y >> 4) else {
            continue;
        };
        if let Some(list) = to_be_ticked.get_mut(section) {
            list.push(pack_offset_coordinates(x, y, z));
        }
    }
    remove(tag, "TileTicks");
    let lists = to_be_ticked
        .into_iter()
        .map(|list| Tag::List(list.into_iter().map(Tag::Short).collect()))
        .collect();
    set(tag, "ToBeTicked", Tag::List(lists));
}

fn pack_offset_coordinates(x: i32, y: i32, z: i32) -> i16 {
    ((x & 15) | ((y & 15) << 4) | ((z & 15) << 8)) as i16
}
