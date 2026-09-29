//! Port of `net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix`:
//! chest block entities standing where a section's palette has a trapped chest
//! block become `minecraft:trapped_chest` block entities.

use std::collections::HashSet;

use crate::datafix::dynamic::{get, get_i32_or, get_str_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::{typed, typed_mut};
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::add_new_choices::add_new_choices;
use super::leaves_fix::get_index;
use super::leaves_fix::section::{Section, SIZE};

/// `SIZE_BITS`.
const SIZE_BITS: i32 = 12;

/// `new TrappedChestBlockEntityFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::sequence(
        "TrappedChestBlockEntityFix",
        vec![
            add_new_choices("AddTrappedChestFix", r::BLOCK_ENTITY),
            Fix::everywhere("Trapped Chest fix", Target::Type(r::CHUNK), |chunk| {
                if let Some(level) = typed_mut(chunk, "Level") {
                    fix_level(level);
                }
            }),
        ],
    )
}

/// `TrappedChestSection`: reads a section, skippable unless its palette holds a
/// trapped chest. Returns the section and the palette ids of trapped chests, or
/// `None` where the Java constructor throws.
fn read_section(tag: &Tag) -> Option<(Section, HashSet<usize>)> {
    let mut chest_ids = HashSet::new();
    let section = Section::read(tag, |palette| {
        for (i, palette_tag) in palette.iter().enumerate() {
            if get_str_or(palette_tag, "Name", "") == "minecraft:trapped_chest" {
                chest_ids.insert(i);
            }
        }
        chest_ids.is_empty()
    })?;
    Some((section, chest_ids))
}

/// The positions (`Y << 12 | index`) of all trapped chest blocks, or `None` where
/// the Java code would throw.
fn trapped_chest_locations(level: &Tag) -> Option<HashSet<i32>> {
    let Some(Tag::List(section_tags)) = typed(level, "Sections") else {
        return Some(HashSet::new());
    };
    let mut locations = HashSet::new();
    for tag in section_tags {
        let (section, chest_ids) = read_section(tag)?;
        if section.is_skippable() {
            continue;
        }
        for i in 0..SIZE {
            if chest_ids.contains(&section.block(i)) {
                locations.insert(section.index() << SIZE_BITS | i as i32);
            }
        }
    }
    Some(locations)
}

fn fix_level(level: &mut Tag) {
    if typed(level, "Sections").is_none() {
        return;
    }
    let Some(chest_locations) = trapped_chest_locations(level) else {
        return;
    };
    let chunk_x = get_i32_or(level, "xPos", 0);
    let chunk_z = get_i32_or(level, "zPos", 0);
    let Some(Tag::List(tile_entities)) = typed_mut(level, "TileEntities") else {
        return;
    };
    for tile_entity in tile_entities {
        let x = get_i32_or(tile_entity, "x", 0).wrapping_sub(chunk_x << 4);
        let y = get_i32_or(tile_entity, "y", 0);
        let z = get_i32_or(tile_entity, "z", 0).wrapping_sub(chunk_z << 4);
        if get(tile_entity, "id").is_some() && chest_locations.contains(&get_index(x, y, z)) {
            set(
                tile_entity,
                "id",
                Tag::String("minecraft:trapped_chest".to_string()),
            );
        }
    }
}
