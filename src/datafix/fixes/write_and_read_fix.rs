//! Port of `net.minecraft.util.datafix.fixes.WriteAndReadFix`.
//!
//! `DataFix.writeAndRead` serialises the typed value with the old type and reads
//! it back with the new one. On NBT that is exactly a decode against the output
//! schema (normalising leaves, running pre-read hooks); a read failure throws
//! in Java, here the value is left as it was.

use crate::datafix::decode::{decode, restore_artifacts};
use crate::datafix::fix::{Fix, FixContext};
use crate::datafix::references::TypeReference;
use crate::datafix::template::Tmpl;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `Util.writeAndReadTypedOrThrow(typed, newType, fix)` / `DataFix.writeFixAndRead`:
/// writes the value, applies `fix` to the serialised form and reads it with the
/// output schema's type of `reference`, keeping the original on a read failure.
pub fn write_fix_and_read(
    ctx: &FixContext<'_>,
    reference: TypeReference,
    tag: &mut Tag,
    fix: impl FnOnce(&mut Tag),
) {
    // Writing merges the typed value over its remainder (bringing back fields
    // the old type dropped) and reading starts from a fresh remainder.
    let mut attempt = tag.clone();
    restore_artifacts(&mut attempt);
    fix(&mut attempt);
    if decode(ctx.output, &Tmpl::Ref(reference), &mut attempt).is_ok() {
        *tag = attempt;
    }
}

/// Reads `tag` with the output schema's type of `reference`
/// ([`write_fix_and_read`] without a fix).
pub fn read_with_output_type(ctx: &FixContext<'_>, reference: TypeReference, tag: &mut Tag) {
    write_fix_and_read(ctx, reference, tag, |_| {});
}

/// `new WriteAndReadFix(schema, name, type)`.
pub fn write_and_read_fix(name: &str, reference: TypeReference) -> Fix {
    Fix::everywhere_with(name, Target::Type(reference), move |ctx, tag| {
        read_with_output_type(ctx, reference, tag)
    })
}

/// `DataFix.writeFixAndRead(name, type, newType, fix)` as used by
/// `ChunkToProtochunkFix`: like [`write_and_read_fix`] but with a fix applied to
/// the serialised value.
pub fn write_fix_and_read_fix(
    name: &str,
    reference: TypeReference,
    fix: impl Fn(&mut Tag) + Send + Sync + 'static,
) -> Fix {
    Fix::everywhere_with(name, Target::Type(reference), move |ctx, tag| {
        write_fix_and_read(ctx, reference, tag, &fix)
    })
}
