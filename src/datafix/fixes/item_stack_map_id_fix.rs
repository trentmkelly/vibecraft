//! Port of `net.minecraft.util.datafix.fixes.ItemStackMapIdFix`.

use crate::datafix::dynamic::{get_i32_or, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::update_typed_compound;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemStackMapIdFix(schema, changesType)`: filled maps keep their map id in
/// `tag.map` instead of the damage value.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ItemInstanceMapIdFix",
        Target::Type(r::ITEM_STACK),
        |stack| {
            if get_str(stack, "id") != Some("minecraft:filled_map") {
                return;
            }
            let map_id = get_i32_or(stack, "Damage", 0);
            update_typed_compound(stack, "tag", |tag| set(tag, "map", Tag::Int(map_id)));
        },
    )
}
