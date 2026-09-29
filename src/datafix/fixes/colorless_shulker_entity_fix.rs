//! Port of `net.minecraft.util.datafix.fixes.ColorlessShulkerEntityFix`.

use crate::datafix::dynamic::{get_i32_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new ColorlessShulkerEntityFix(schema, changesType)`: the old default colour
/// 10 becomes the colourless value 16.
pub fn fix() -> Fix {
    named_entity_fix(
        "Colorless shulker entity fix",
        r::ENTITY,
        "minecraft:shulker",
        |shulker| {
            if get_i32_or(shulker, "Color", 0) == 10 {
                set(shulker, "Color", Tag::Byte(16));
            }
        },
    )
}
