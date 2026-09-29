//! Port of `net.minecraft.util.datafix.fixes.WrittenBookPagesStrictJsonFix`.

use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::rewrite_from_lenient;
use crate::datafix::typed::typed_mut;
use crate::storage::nbt::Tag;

use super::item_stack_tag_fix::item_stack_tag_fix;

/// `new WrittenBookPagesStrictJsonFix(schema)`: book pages become strict JSON components.
pub fn fix() -> Fix {
    item_stack_tag_fix(
        "WrittenBookPagesStrictJsonFix",
        |id| id == "minecraft:written_book",
        |tag| {
            if let Some(Tag::List(pages)) = typed_mut(tag, "pages") {
                for page in pages {
                    if let Tag::String(text) = page {
                        *text = rewrite_from_lenient(text);
                    }
                }
            }
        },
    )
}
