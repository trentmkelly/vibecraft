//! Port of `net.minecraft.util.datafix.fixes.NamedEntityWriteReadFix`.
//!
//! Unlike [`NamedEntityFix`](super::named_entity_fix) the callback works on the
//! *serialised* entity (`id` included), which is then read back with the output
//! schema's type of the reference.

use crate::datafix::fix::Fix;
use crate::datafix::references::TypeReference;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::named_entity_fix::choice_set_of;
use super::write_and_read_fix::write_fix_and_read;

/// `new NamedEntityWriteReadFix(schema, changesType, name, type, entityName) { fix }`.
pub fn named_entity_write_read_fix(
    name: &str,
    reference: TypeReference,
    entity_name: &str,
    fix: impl Fn(&mut Tag) + Send + Sync + 'static,
) -> Fix {
    let target = Target::Choice(choice_set_of(reference), entity_name.to_string());
    Fix::everywhere_with(name, target, move |ctx, entity| {
        write_fix_and_read(ctx, reference, entity, &fix)
    })
}
