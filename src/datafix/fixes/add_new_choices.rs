//! Port of `net.minecraft.util.datafix.fixes.AddNewChoices`.
//!
//! The Java rule only converts the tagged-choice type to the output schema's
//! (larger) choice type after checking that every id is known there; the data
//! itself is untouched. Its effect on a value is therefore only that the rule
//! matches, which makes the update decode/normalise the value.

use crate::datafix::fix::Fix;
use crate::datafix::references::TypeReference;
use crate::datafix::walk::Target;

use super::named_entity_fix::choice_set_of;

/// `new AddNewChoices(schema, name, type)`.
pub fn add_new_choices(name: &str, reference: TypeReference) -> Fix {
    Fix::everywhere(name, Target::AnyChoice(choice_set_of(reference)), |_| {})
}
