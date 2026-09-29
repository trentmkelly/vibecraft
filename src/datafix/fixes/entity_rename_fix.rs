//! Port of `net.minecraft.util.datafix.fixes.EntityRenameFix`.
//!
//! Visits every entity, hands the id and the entity value (without the `id`
//! discriminator) to the callback and stores the id it returns.

use crate::datafix::dynamic::{set, take_id};
use crate::datafix::fix::{Fix, FixContext};
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new EntityRenameFix(name, schema, changesType) { fix(name, entity) }`.
pub fn entity_rename_fix(
    name: &str,
    fix: impl Fn(&FixContext<'_>, &str, &mut Tag) -> String + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere_with(
        name,
        Target::AnyChoice(ChoiceSet::Entities),
        move |ctx, tag| {
            let Some(old_name) = take_id(tag) else {
                return;
            };
            let new_name = fix(ctx, &old_name, tag);
            set(tag, "id", Tag::String(new_name));
        },
    )
}
