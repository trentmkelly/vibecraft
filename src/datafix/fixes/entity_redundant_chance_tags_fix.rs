//! Port of `net.minecraft.util.datafix.fixes.EntityRedundantChanceTagsFix`.

use crate::datafix::dynamic::{as_f32, get, list_items, remove};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new EntityRedundantChanceTagsFix(schema, changesType)`: drops all-zero drop chance lists.
pub fn fix() -> Fix {
    Fix::everywhere(
        "EntityRedundantChanceTagsFix",
        Target::Type(r::ENTITY),
        |tag| {
            if is_zero_list(get(tag, "HandDropChances"), 2) {
                remove(tag, "HandDropChances");
            }
            if is_zero_list(get(tag, "ArmorDropChances"), 4) {
                remove(tag, "ArmorDropChances");
            }
        },
    )
}

/// `EntityRedundantChanceTagsFix.isZeroList`: a list of exactly `size` numbers, all zero.
fn is_zero_list(element: Option<&Tag>, size: usize) -> bool {
    let Some(items) = element.and_then(list_items) else {
        return false;
    };
    let floats: Option<Vec<f32>> = items.iter().map(as_f32).collect();
    floats.is_some_and(|floats| floats.len() == size && floats.iter().all(|f| *f == 0.0))
}
