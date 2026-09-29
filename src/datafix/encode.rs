//! The "write" half of `DataFixerUpper.update`: encoding with the final schema.
//!
//! Writing a typed value back to a `Dynamic` runs the post-write half of every
//! `DSL.hook` in the target type (children first). Only a handful of types carry
//! non-identity post-write hooks (the scoreboard objective criteria repacker); all
//! other encoding effects are already covered by [`decode`](super::decode).
//!
//! Encoding can also fail: a tagged choice whose key is not registered in the
//! target schema (for example a `minecraft:flower_pot` block entity that
//! survived into a schema that no longer knows it) cannot be written, and
//! `DataFixerUpper.update` then returns its input unchanged.

/// A value that the target schema's type cannot encode (Java `DataResult.error`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodeError(pub String);

use crate::storage::nbt::Tag;

use super::decode::{decode, is_opaque, read_as_or_right};
use super::dynamic::{get_mut, get_str};
use super::schema::Schema;
use super::template::Tmpl;

/// Encodes `tag` with `tmpl` (as resolved by `schema`): applies the post-write
/// hooks and fails on values the type cannot represent.
pub fn apply_post_write_hooks(
    schema: &Schema,
    tmpl: &Tmpl,
    tag: &mut Tag,
) -> Result<(), EncodeError> {
    match tmpl {
        Tmpl::Remainder | Tmpl::Leaf(_) => Ok(()),
        Tmpl::Ref(reference) => match schema.type_of(reference) {
            Some(inner) => apply_post_write_hooks(schema, inner, tag),
            None => Ok(()),
        },
        Tmpl::Fields(fields) => {
            for field in fields {
                if is_opaque(tag, field.name) {
                    continue;
                }
                if let Some(value) = get_mut(tag, field.name) {
                    apply_post_write_hooks(schema, &field.tmpl, value)?;
                }
            }
            Ok(())
        }
        Tmpl::List(inner) => match tag {
            Tag::List(items) => items
                .iter_mut()
                .try_for_each(|item| apply_post_write_hooks(schema, inner, item)),
            _ => Ok(()),
        },
        Tmpl::CompoundList(_, inner) => match tag {
            Tag::Compound(entries) => entries
                .iter_mut()
                .try_for_each(|(_, value)| apply_post_write_hooks(schema, inner, value)),
            _ => Ok(()),
        },
        Tmpl::And(parts) => parts
            .iter()
            .try_for_each(|part| apply_post_write_hooks(schema, part, tag)),
        Tmpl::Or(left, right) => {
            // Compound values remember the side they were read with; scalars are
            // told apart by their tag type.
            let on_right = if matches!(tag, Tag::Compound(_)) {
                read_as_or_right(tag)
            } else {
                let mut attempt = tag.clone();
                decode(schema, left, &mut attempt).is_err()
            };
            apply_post_write_hooks(schema, if on_right { right } else { left }, tag)
        }
        Tmpl::Choice { key, set, .. } => {
            let Some(id) = get_str(tag, key).map(str::to_string) else {
                return Err(EncodeError(format!("missing choice key {key}")));
            };
            match schema.choice(*set, &id) {
                Some(branch) => apply_post_write_hooks(schema, branch, tag),
                None => Err(EncodeError(format!("Unsupported key: {id}"))),
            }
        }
        Tmpl::Hook(inner, _, post_write) => {
            apply_post_write_hooks(schema, inner, tag)?;
            if let Some(post_write) = post_write {
                post_write(tag);
            }
            Ok(())
        }
    }
}
