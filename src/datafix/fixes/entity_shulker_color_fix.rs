//! Port of `net.minecraft.util.datafix.fixes.EntityShulkerColorFix`.

use crate::datafix::dynamic::{get, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new EntityShulkerColorFix(schema, changesType)`: shulkers without a `Color`
/// get the default colour 10.
///
/// The Java condition `get("Color").map(Dynamic::asNumber).result().isEmpty()`
/// only tests that the field is missing, not that it is numeric.
pub fn fix() -> Fix {
    named_entity_fix(
        "EntityShulkerColorFix",
        r::ENTITY,
        "minecraft:shulker",
        |tag| {
            if get(tag, "Color").is_none() {
                set(tag, "Color", Tag::Byte(10));
            }
        },
    )
}
