//! Port of `net.minecraft.util.datafix.fixes.BlockEntityCustomNameToComponentFix`.

use crate::datafix::dynamic::{get_str, get_str_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::create_plain_text_component;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::write_and_read_fix::write_fix_and_read;

/// `BlockEntityCustomNameToComponentFix.NAMEABLE_BLOCK_ENTITIES`.
const NAMEABLE_BLOCK_ENTITIES: &[&str] = &[
    "minecraft:beacon",
    "minecraft:banner",
    "minecraft:brewing_stand",
    "minecraft:chest",
    "minecraft:trapped_chest",
    "minecraft:dispenser",
    "minecraft:dropper",
    "minecraft:enchanting_table",
    "minecraft:furnace",
    "minecraft:hopper",
    "minecraft:shulker_box",
];

/// `BlockEntityCustomNameToComponentFix.fixTagCustomName`: an empty (or missing)
/// name is dropped, any other becomes a plain text component.
pub fn fix_tag_custom_name(tag: &mut Tag) {
    let name = get_str_or(tag, "CustomName", "").to_string();
    if name.is_empty() {
        remove(tag, "CustomName");
    } else {
        set(tag, "CustomName", create_plain_text_component(&name));
    }
}

/// `new BlockEntityCustomNameToComponentFix(schema)`.
pub fn fix() -> Fix {
    Fix::everywhere_with(
        "BlockEntityCustomNameToComponentFix",
        Target::Type(r::BLOCK_ENTITY),
        |ctx, block_entity| {
            let nameable = match get_str(block_entity, "id") {
                Some(id) => NAMEABLE_BLOCK_ENTITIES.contains(&id),
                None => true,
            };
            if nameable {
                write_fix_and_read(ctx, r::BLOCK_ENTITY, block_entity, fix_tag_custom_name);
            }
        },
    )
}
