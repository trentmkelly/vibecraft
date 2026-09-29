//! Port of `net.minecraft.util.datafix.fixes.ItemStackTagFix`.

use crate::datafix::dynamic::{get_mut, get_str};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemStackTagFix(schema, name, idFilter) { fixItemStackTag }`: runs `fix`
/// on the `tag` compound of every stack whose id passes `id_filter`.
pub fn item_stack_tag_fix(
    name: &str,
    id_filter: impl Fn(&str) -> bool + Send + Sync + 'static,
    fix: impl Fn(&mut Tag) + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere(name, Target::Type(r::ITEM_STACK), move |stack| {
        let matches = get_str(stack, "id").is_some_and(&id_filter);
        if matches {
            if let Some(tag) = get_mut(stack, "tag") {
                fix(tag);
            }
        }
    })
}
