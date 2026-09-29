//! Port of `net.minecraft.util.datafix.fixes.EntityBlockStateFix`: block names
//! and ids of entities become block states.
//!
//! The typed fields that are replaced (`Block`, `inTile`, `DisplayTile`,
//! `carried`) stay in the data as raw leftovers, see
//! [`SHADOWED_KEYS`](crate::datafix::decode::SHADOWED_KEYS).

use crate::datafix::decode::is_opaque;
use crate::datafix::dynamic::{as_i32, get, get_i32_or, get_i8_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::block_state_data::get_tag;
use super::entity_block_state_fix_ids::MAP;

/// `EntityBlockStateFix.getBlockId`: legacy numeric id of a block name (0 if unknown).
pub fn get_block_id(name: &str) -> i32 {
    MAP.iter()
        .find(|(block, _)| *block == name)
        .map_or(0, |(_, id)| *id)
}

/// `new EntityBlockStateFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "EntityBlockStateFix",
        Target::AnyChoice(ChoiceSet::Entities),
        fix_entity,
    )
}

fn fix_entity(entity: &mut Tag) {
    let id = match get(entity, "id") {
        Some(Tag::String(id)) => id.clone(),
        _ => return,
    };
    match id.as_str() {
        "minecraft:falling_block" => update_falling_block(entity),
        "minecraft:enderman" => {
            update_block_to_block_state(entity, "carried", "carriedData", "carriedBlockState")
        }
        "minecraft:arrow" | "minecraft:spectral_arrow" => {
            update_block_to_block_state(entity, "inTile", "inData", "inBlockState")
        }
        // `removeInTile` only drops the typed `inTile` field; its raw copy stays.
        "minecraft:egg"
        | "minecraft:ender_pearl"
        | "minecraft:fireball"
        | "minecraft:potion"
        | "minecraft:small_fireball"
        | "minecraft:snowball"
        | "minecraft:wither_skull"
        | "minecraft:xp_bottle" => {}
        "minecraft:commandblock_minecart"
        | "minecraft:minecart"
        | "minecraft:chest_minecart"
        | "minecraft:furnace_minecart"
        | "minecraft:tnt_minecart"
        | "minecraft:hopper_minecart"
        | "minecraft:spawner_minecart" => {
            update_block_to_block_state(entity, "DisplayTile", "DisplayData", "DisplayState")
        }
        _ => {}
    }
}

/// The legacy block id stored in a typed `BLOCK_NAME` value (`or(int, name)`).
fn block_of(value: &Tag) -> i32 {
    match value {
        Tag::String(name) => get_block_id(name),
        other => as_i32(other).unwrap_or(0),
    }
}

/// `EntityBlockStateFix.updateFallingBlock`.
fn update_falling_block(entity: &mut Tag) {
    let typed_block = match get(entity, "Block") {
        Some(value) if !is_opaque(entity, "Block") => Some(block_of(value)),
        _ => None,
    };
    let block = typed_block.unwrap_or_else(|| match get(entity, "TileID").and_then(as_i32) {
        Some(tile_id) => tile_id,
        None => i32::from(get_i8_or(entity, "Tile", 0)) & 0xFF,
    });
    let data = get_i32_or(entity, "Data", 0) & 15;
    set(entity, "BlockState", get_tag((block << 4) | data));
    remove(entity, "Data");
    remove(entity, "TileID");
    remove(entity, "Tile");
}

/// `EntityBlockStateFix.updateBlockToBlockState`.
fn update_block_to_block_state(
    entity: &mut Tag,
    old_field: &str,
    data_name: &str,
    new_field: &str,
) {
    if let Some(value) = get(entity, old_field).filter(|_| !is_opaque(entity, old_field)) {
        let block = block_of(value);
        let data = get_i32_or(entity, data_name, 0) & 15;
        set(entity, new_field, get_tag((block << 4) | data));
    }
    remove(entity, data_name);
}
