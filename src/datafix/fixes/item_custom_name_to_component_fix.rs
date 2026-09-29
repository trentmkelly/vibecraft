//! Port of `net.minecraft.util.datafix.fixes.ItemCustomNameToComponentFix`.

use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::create_text_component_json;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemCustomNameToComponentFix(schema)`: `display.Name` becomes a JSON text
/// component.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ItemCustomNameToComponentFix",
        Target::Type(r::ITEM_STACK),
        |stack| {
            let Some(tag) = typed_mut(stack, "tag") else {
                return;
            };
            let Some(display) = typed_mut(tag, "display") else {
                return;
            };
            if let Some(Tag::String(name)) = typed_mut(display, "Name") {
                *name = create_text_component_json(name);
            }
        },
    )
}
