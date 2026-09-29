//! Port of `net.minecraft.util.datafix.fixes.NamespacedTypeRenameFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references::TypeReference;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new NamespacedTypeRenameFix(schema, name, type, renamer)`: renames the
/// namespaced string values of a reference type (recipes, ...).
pub fn namespaced_type_rename_fix(
    name: &str,
    reference: TypeReference,
    renamer: impl Fn(&str) -> String + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere(name, Target::Type(reference), move |value| {
        if let Tag::String(text) = value {
            *text = renamer(text);
        }
    })
}
