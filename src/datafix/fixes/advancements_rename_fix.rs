//! Port of `net.minecraft.util.datafix.fixes.AdvancementsRenameFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new AdvancementsRenameFix(schema, changesType, name, renamer)`: renames the
/// advancement ids (the keys of the advancements compound).
pub fn advancements_rename_fix(
    name: &str,
    renamer: impl Fn(&str) -> String + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere(name, Target::Type(r::ADVANCEMENTS), move |advancements| {
        if let Tag::Compound(entries) = advancements {
            for (id, _) in entries.iter_mut() {
                *id = renamer(id);
            }
        }
    })
}
