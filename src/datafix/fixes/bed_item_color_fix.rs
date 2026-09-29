//! Port of `net.minecraft.util.datafix.fixes.BedItemColorFix`.

use crate::datafix::dynamic::{get_i32_or, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new BedItemColorFix(schema, changesType)`: plain beds become red (damage 14).
pub fn fix() -> Fix {
    Fix::everywhere("BedItemColorFix", Target::Type(r::ITEM_STACK), |stack| {
        if get_str(stack, "id") == Some("minecraft:bed") && get_i32_or(stack, "Damage", 0) == 0 {
            set(stack, "Damage", Tag::Short(14));
        }
    })
}
