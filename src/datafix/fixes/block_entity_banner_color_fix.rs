//! Port of `net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix`.

use crate::datafix::dynamic::{as_i32, get_mut};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new BlockEntityBannerColorFix(schema, changesType)`: banner colours switch
/// from the wool scale to the dye scale.
pub fn fix() -> Fix {
    named_entity_fix(
        "BlockEntityBannerColorFix",
        r::BLOCK_ENTITY,
        "minecraft:banner",
        |banner| {
            if let Some(base) = get_mut(banner, "Base") {
                *base = Tag::Int(15 - as_i32(base).unwrap_or(0));
            }
            if let Some(Tag::List(patterns)) = get_mut(banner, "Patterns") {
                for pattern in patterns {
                    if let Some(color) = get_mut(pattern, "Color") {
                        *color = Tag::Int(15 - as_i32(color).unwrap_or(0));
                    }
                }
            }
        },
    )
}
