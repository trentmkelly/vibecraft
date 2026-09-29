//! Port of `net.minecraft.util.datafix.fixes.NamedEntityFix`.
//!
//! Rewrites the entities (or block entities) of one specific id. The Java
//! `fix(Typed)` receives the choice branch value, which does not contain the
//! `id` discriminator, so the id is stripped for the callback and restored after.

use crate::datafix::dynamic::{set, take_id};
use crate::datafix::fix::Fix;
use crate::datafix::references::{self as r, TypeReference};
use crate::datafix::template::ChoiceSet;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// The tagged-choice registry that backs a reference (`ENTITY` or `BLOCK_ENTITY`).
pub fn choice_set_of(reference: TypeReference) -> ChoiceSet {
    match reference {
        r::ENTITY => ChoiceSet::Entities,
        r::BLOCK_ENTITY => ChoiceSet::BlockEntities,
        other => panic!("{other} is not a tagged-choice reference"),
    }
}

/// `new NamedEntityFix(schema, changesType, name, type, entityName) { fix }`.
pub fn named_entity_fix(
    name: &str,
    reference: TypeReference,
    entity_name: &str,
    fix: impl Fn(&mut Tag) + Send + Sync + 'static,
) -> Fix {
    let target = Target::Choice(choice_set_of(reference), entity_name.to_string());
    Fix::everywhere(name, target, move |tag| {
        let id = take_id(tag);
        fix(tag);
        if let Some(id) = id {
            set(tag, "id", Tag::String(id));
        }
    })
}
