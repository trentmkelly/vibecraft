//! Port of `net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix`.

use crate::datafix::dynamic::remove;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;

use super::named_entity_fix::named_entity_fix;

/// `new BlockEntityShulkerBoxColorFix(schema, changesType)`: drops `Color`.
pub fn fix() -> Fix {
    named_entity_fix(
        "BlockEntityShulkerBoxColorFix",
        r::BLOCK_ENTITY,
        "minecraft:shulker_box",
        |tag| {
            remove(tag, "Color");
        },
    )
}
