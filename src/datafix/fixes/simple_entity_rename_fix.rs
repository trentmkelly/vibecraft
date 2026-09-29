//! Port of `net.minecraft.util.datafix.fixes.SimpleEntityRenameFix`.

use crate::datafix::fix::Fix;
use crate::storage::nbt::Tag;

use super::entity_rename_fix::entity_rename_fix;

/// `new SimpleEntityRenameFix(name, schema, changesType) { getNewNameAndTag }`:
/// the callback may edit the entity's remaining data and returns the new id.
pub fn simple_entity_rename_fix(
    name: &str,
    get_new_name_and_tag: impl Fn(&str, &mut Tag) -> String + Send + Sync + 'static,
) -> Fix {
    entity_rename_fix(name, move |_, entity_name, tag| {
        get_new_name_and_tag(entity_name, tag)
    })
}
