//! Port of `net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix`:
//! numeric enchantment ids become namespaced names.

use crate::datafix::dynamic::{get, get_i32_or, list_items, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_stack_enchantment_names_fix_map::MAP;

fn enchantment_name(id: i32) -> &'static str {
    MAP.iter()
        .find(|(key, _)| *key == id)
        .map_or("null", |(_, name)| *name)
}

/// Replaces the numeric `id` of each enchantment compound by its name.
fn rename_ids(list: &[Tag]) -> Tag {
    Tag::List(
        list.iter()
            .map(|element| {
                let mut element = element.clone();
                let name = enchantment_name(get_i32_or(&element, "id", 0));
                set(&mut element, "id", Tag::String(name.to_string()));
                element
            })
            .collect(),
    )
}

/// `new ItemStackEnchantmentNamesFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ItemStackEnchantmentFix",
        Target::Type(r::ITEM_STACK),
        |stack| {
            let Some(tag) = typed_mut(stack, "tag") else {
                return;
            };
            if let Some(list) = get(tag, "ench").and_then(list_items) {
                let enchantments = rename_ids(&list);
                remove(tag, "ench");
                set(tag, "Enchantments", enchantments);
            }
            if let Some(list) = get(tag, "StoredEnchantments").and_then(list_items) {
                set(tag, "StoredEnchantments", rename_ids(&list));
            }
        },
    )
}
