//! Port of `net.minecraft.util.datafix.fixes.ItemRenameFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `ItemRenameFix.create(schema, name, fixItem)`: renames every `ITEM_NAME`.
pub fn item_rename_fix(
    name: &str,
    fix_item: impl Fn(&str) -> String + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere(name, Target::Type(r::ITEM_NAME), move |tag| {
        if let Tag::String(value) = tag {
            *value = fix_item(value);
        }
    })
}
