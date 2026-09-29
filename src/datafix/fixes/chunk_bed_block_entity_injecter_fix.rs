//! Port of `net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix`.

use crate::datafix::dynamic::{as_i32, compound, get, get_i32_or};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::ChoiceSet;
use crate::datafix::typed::{set_typed, typed};
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// Legacy block id of beds (416) as compared by the Java code: `(id & 0xFF) << 4`.
const BED_ID_SHIFTED: i32 = 416;

/// `new ChunkBedBlockEntityInjecterFix(schema, changesType)`: adds the block
/// entities that 1.12 beds gained, one per bed block found in the chunk sections.
pub fn fix() -> Fix {
    Fix::sequence(
        "ChunkBedBlockEntityInjecterFix",
        vec![
            Fix::everywhere(
                "InjectBedBlockEntityType",
                Target::AnyChoice(ChoiceSet::BlockEntities),
                |_| {},
            ),
            Fix::everywhere(
                "BedBlockEntityInjecter",
                Target::Type(r::CHUNK),
                inject_beds,
            ),
        ],
    )
}

fn inject_beds(chunk: &mut Tag) {
    let Some(level) = crate::datafix::dynamic::get_mut(chunk, "Level") else {
        return;
    };
    let chunk_x = get_i32_or(level, "xPos", 0);
    let chunk_z = get_i32_or(level, "zPos", 0);
    let mut tile_entities = match typed(level, "TileEntities") {
        Some(Tag::List(existing)) => existing.clone(),
        _ => Vec::new(),
    };
    let sections = match get(level, "Sections") {
        Some(Tag::List(sections)) => sections.clone(),
        _ => Vec::new(),
    };
    for section in &sections {
        let section_y = get_i32_or(section, "Y", 0);
        let Some(blocks) = block_ids(section) else {
            continue;
        };
        for (index, block) in blocks.into_iter().enumerate() {
            if BED_ID_SHIFTED != (block & 0xFF) << 4 {
                continue;
            }
            let position = index as i32;
            let x = position & 15;
            let y = (position >> 8) & 15;
            let z = (position >> 4) & 15;
            tile_entities.push(compound(vec![
                ("id", Tag::String("minecraft:bed".to_string())),
                ("x", Tag::Int(x + (chunk_x << 4))),
                ("y", Tag::Int(y + (section_y << 4))),
                ("z", Tag::Int(z + (chunk_z << 4))),
                ("color", Tag::Short(14)),
            ]));
        }
    }
    if !tile_entities.is_empty() {
        set_typed(level, "TileEntities", Tag::List(tile_entities));
    }
}

/// `sectionTag.get("Blocks").asIntStream()`.
fn block_ids(section: &Tag) -> Option<Vec<i32>> {
    let blocks = get(section, "Blocks")?;
    crate::datafix::dynamic::list_items(blocks)
        .map(|items| items.iter().filter_map(as_i32).collect())
}
