//! Port of `net.minecraft.util.datafix.fixes.SimplestEntityRenameFix`.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new SimplestEntityRenameFix(name, schema, changesType) { rename }`: renames
/// the entity id and every `ENTITY_NAME` reference.
pub fn simplest_entity_rename_fix(
    name: &str,
    rename: impl Fn(&str) -> String + Clone + Send + Sync + 'static,
) -> Fix {
    let rename_id = rename.clone();
    let choice_rule = Fix::everywhere(name, Target::AnyChoice(ChoiceSet::Entities), move |tag| {
        if let Some(old) = get_str(tag, "id").map(str::to_string) {
            set(tag, "id", Tag::String(rename_id(&old)));
        }
    });
    let name_rule = Fix::everywhere(
        format!("{name} for entity name"),
        Target::Type(r::ENTITY_NAME),
        move |tag| {
            if let Tag::String(value) = tag {
                *value = rename(value);
            }
        },
    );
    Fix::sequence(name, vec![choice_rule, name_rule])
}
