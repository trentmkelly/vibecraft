//! Port of `net.minecraft.util.datafix.fixes.ItemIdFix`: numeric item ids become
//! namespaced names.

use crate::datafix::dynamic::get_mut;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_id_fix_names::ITEM_NAMES;

/// `ItemIdFix.getItem`: the name of a numeric item id (`minecraft:air` when the
/// id is unknown, the map's default return value).
pub fn get_item(id: i32) -> &'static str {
    match ITEM_NAMES.binary_search_by_key(&id, |(key, _)| *key) {
        Ok(index) => ITEM_NAMES[index].1,
        Err(_) => "minecraft:air",
    }
}

/// `new ItemIdFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere("ItemIdFix", Target::Type(r::ITEM_STACK), |stack| {
        if let Some(id) = get_mut(stack, "id") {
            if let Tag::Int(numeric) = id {
                *id = Tag::String(get_item(*numeric).to_string());
            }
        }
    })
}
