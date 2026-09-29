//! Port of `net.minecraft.util.datafix.fixes.ItemShulkerBoxColorFix`.

use crate::datafix::dynamic::{get_i32_or, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `ItemShulkerBoxColorFix.NAMES_BY_COLOR`.
pub const NAMES_BY_COLOR: [&str; 16] = [
    "minecraft:white_shulker_box",
    "minecraft:orange_shulker_box",
    "minecraft:magenta_shulker_box",
    "minecraft:light_blue_shulker_box",
    "minecraft:yellow_shulker_box",
    "minecraft:lime_shulker_box",
    "minecraft:pink_shulker_box",
    "minecraft:gray_shulker_box",
    "minecraft:silver_shulker_box",
    "minecraft:cyan_shulker_box",
    "minecraft:purple_shulker_box",
    "minecraft:blue_shulker_box",
    "minecraft:brown_shulker_box",
    "minecraft:green_shulker_box",
    "minecraft:red_shulker_box",
    "minecraft:black_shulker_box",
];

/// `new ItemShulkerBoxColorFix(schema, changesType)`: the block entity colour
/// selects the coloured shulker box item.
///
/// The Java code calls `blockEntityRest.remove("Color")` and discards the
/// result, so `Color` (faithfully) stays in the block entity tag.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ItemShulkerBoxColorFix",
        Target::Type(r::ITEM_STACK),
        |stack| {
            if get_str(stack, "id") != Some("minecraft:shulker_box") {
                return;
            }
            let Some(block_entity) =
                typed(stack, "tag").and_then(|tag| typed(tag, "BlockEntityTag"))
            else {
                return;
            };
            // Java indexes with `color % 16`, which throws for negative colours.
            let Ok(index) = usize::try_from(get_i32_or(block_entity, "Color", 0) % 16) else {
                return;
            };
            set(stack, "id", Tag::String(NAMES_BY_COLOR[index].to_string()));
        },
    )
}
