//! Port of `net.minecraft.util.datafix.fixes.ItemStackTheFlatteningFix`:
//! item id + damage pairs become flattened item ids.

use crate::datafix::dynamic::{get_i32_or, get_str, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::update_typed_compound;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_stack_the_flattening_fix_tables::{DAMAGE_IDS, MAP};

fn lookup(key: &str) -> Option<&'static str> {
    MAP.iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| *value)
}

/// `ItemStackTheFlatteningFix.updateItem`: the flattened id for `name` + `data`,
/// or `None` when `name` is not one of the split items.
pub fn update_item(name: &str, data: i32) -> Option<String> {
    let is_split_item = MAP
        .iter()
        .any(|(key, _)| key.rsplit_once('.').is_some_and(|(id, _)| id == name));
    if !is_split_item {
        return None;
    }
    let new_name = lookup(&format!("{name}.{data}")).or_else(|| lookup(&format!("{name}.0")));
    new_name.map(str::to_string)
}

/// `new ItemStackTheFlatteningFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ItemInstanceTheFlatteningFix",
        Target::Type(r::ITEM_STACK),
        fix_stack,
    )
}

fn fix_stack(stack: &mut Tag) {
    let Some(id) = get_str(stack, "id").map(str::to_string) else {
        return;
    };
    let data = get_i32_or(stack, "Damage", 0);
    if let Some(new_value) = update_item(&id, data) {
        set(stack, "id", Tag::String(new_value));
    }
    if DAMAGE_IDS.contains(&id.as_str()) {
        update_typed_compound(stack, "tag", |tag| set(tag, "Damage", Tag::Int(data)));
    }
    remove(stack, "Damage");
}
