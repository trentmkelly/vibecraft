//! The "read" half of `DataFixerUpper.update`: decoding a tag against a schema type.
//!
//! Java decodes the input `Dynamic` into a typed value before any fix runs and
//! encodes it again afterwards. The observable effects, reproduced here on the
//! raw tag, are:
//!
//! * numeric leaves declared as `intType()` become int tags,
//! * `namespacedString()` leaves (and namespaced choice keys) are normalised with
//!   `ensureNamespaced`,
//! * pre-read hooks (`DSL.hook`) rewrite the value,
//! * a structural mismatch, a missing *required* field, or a tagged choice with
//!   a missing/unknown key makes the read fail,
//! * a failing *optional* field is not an error: DFU treats the field as absent
//!   and keeps its raw data in the remainder. Such fields are marked *opaque*
//!   here so that neither normalisation nor any fix touches them.
//!
//! # Remainder copies
//!
//! DFU's `remainder` keeps a copy of *every* key of the value as it was read, and
//! typed fields are written over that copy. A field that a fix removes from the
//! typed structure therefore reappears on write with its original, unnormalised
//! value: `Equipment` (`EntityEquipmentToArmorAndHandFix`), `Riding`
//! (`EntityRidingToPassengersFix`) and the block names replaced by block states
//! (`EntityBlockStateFix`) linger in upgraded data. To reproduce this,
//! [`decode`] stashes the raw value of the [`SHADOWED_KEYS`] under a shadow key,
//! and [`restore_artifacts`] puts it back after the update if the field is gone.

use crate::storage::nbt::Tag;

use super::dynamic::{as_i32, ensure_namespaced, get, get_mut, set};
use super::schema::Schema;
use super::template::{Leaf, Tmpl};

/// Typed fields that a later type-changing fix drops from the type (see the
/// module documentation).
pub const SHADOWED_KEYS: &[&str] = &[
    // EntityEquipmentToArmorAndHandFix
    "Equipment",
    // EntityRidingToPassengersFix
    "Riding",
    // EntityBlockStateFix: block names replaced by block states
    "Block",
    "inTile",
    "DisplayTile",
    "carried",
];

const SHADOW_PREFIX: &str = "\u{0}raw:";
const OPAQUE_PREFIX: &str = "\u{0}opaque:";
/// Marks a compound that an `Or` template could only read with its right side.
const OR_RIGHT_KEY: &str = "\u{0}opaque:$or_right";

fn shadow_key(name: &str) -> String {
    format!("{SHADOW_PREFIX}{name}")
}

/// The marker key recording that `name` is opaque.
fn opaque_key(name: &str) -> String {
    format!("{OPAQUE_PREFIX}{name}")
}

/// Whether `name` of `tag` failed to decode and must be treated as untyped.
pub fn is_opaque(tag: &Tag, name: &str) -> bool {
    get(tag, &opaque_key(name)).is_some()
}

/// Whether an `Or` template read this compound with its right side (the left side
/// failed at read time, so the value is not a value of the left type).
pub fn read_as_or_right(tag: &Tag) -> bool {
    get(tag, OR_RIGHT_KEY).is_some()
}

/// Marks `name` as typed again (a fix replaced it with a well-typed value).
pub fn clear_opaque(tag: &mut Tag, name: &str) {
    if let Tag::Compound(entries) = tag {
        let key = opaque_key(name);
        entries.retain(|(entry, _)| *entry != key);
    }
}

/// Turns the bookkeeping entries of [`decode`] back into plain data: raw copies
/// of removed fields are restored, opaque markers are dropped.
pub fn restore_artifacts(tag: &mut Tag) {
    match tag {
        Tag::Compound(entries) => {
            for (name, value) in entries.iter_mut() {
                if !name.starts_with(SHADOW_PREFIX) && !name.starts_with(OPAQUE_PREFIX) {
                    restore_artifacts(value);
                }
            }
            let mut shadows = Vec::new();
            entries.retain_mut(|(name, value)| {
                if let Some(original) = name.strip_prefix(SHADOW_PREFIX) {
                    shadows.push((original.to_string(), std::mem::replace(value, Tag::End)));
                    false
                } else {
                    !name.starts_with(OPAQUE_PREFIX)
                }
            });
            for (name, raw) in shadows {
                if !entries.iter().any(|(existing, _)| *existing == name) {
                    entries.push((name, raw));
                }
            }
        }
        Tag::List(items) => items.iter_mut().for_each(restore_artifacts),
        _ => {}
    }
}

/// A failed read (Java `DataResult.error`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError(pub String);

/// Decodes `tag` in place against `tmpl` as resolved by `schema`.
pub fn decode(schema: &Schema, tmpl: &Tmpl, tag: &mut Tag) -> Result<(), DecodeError> {
    match tmpl {
        Tmpl::Remainder => Ok(()),
        Tmpl::Leaf(leaf) => decode_leaf(*leaf, tag),
        Tmpl::Ref(reference) => {
            let target = schema
                .type_of(reference)
                .ok_or_else(|| DecodeError(format!("unknown type reference {reference}")))?;
            decode(schema, target, tag)
        }
        Tmpl::Fields(fields) => {
            if !matches!(tag, Tag::Compound(_)) {
                return Err(DecodeError("expected a compound".into()));
            }
            for field in fields {
                decode_field(schema, tag, field.name, &field.tmpl, field.optional)?;
            }
            Ok(())
        }
        Tmpl::List(element) => match tag {
            Tag::List(items) => items
                .iter_mut()
                .try_for_each(|item| decode(schema, element, item)),
            _ => Err(DecodeError("expected a list".into())),
        },
        Tmpl::CompoundList(key, element) => match tag {
            Tag::Compound(entries) => entries.iter_mut().try_for_each(|(name, value)| {
                let mut key_tag = Tag::String(std::mem::take(name));
                let result = decode(schema, key, &mut key_tag);
                if let Tag::String(decoded) = key_tag {
                    *name = decoded;
                }
                result?;
                decode(schema, element, value)
            }),
            _ => Err(DecodeError("expected a compound".into())),
        },
        Tmpl::And(parts) => parts.iter().try_for_each(|part| decode(schema, part, tag)),
        Tmpl::Or(left, right) => {
            let mut attempt = tag.clone();
            if decode(schema, left, &mut attempt).is_ok() {
                *tag = attempt;
                Ok(())
            } else {
                decode(schema, right, tag)?;
                // Later schemas may accept the value on the left, but the typed
                // value stays a right-hand value.
                set(tag, OR_RIGHT_KEY, Tag::Byte(0));
                Ok(())
            }
        }
        Tmpl::Choice {
            key,
            namespaced_key,
            set,
        } => {
            let id = match get_mut(tag, key) {
                Some(Tag::String(id)) => id,
                _ => return Err(DecodeError(format!("missing choice key {key}"))),
            };
            if *namespaced_key {
                *id = ensure_namespaced(id);
            }
            let branch = schema
                .choice(*set, id)
                .ok_or_else(|| DecodeError(format!("Unsupported key: {id}")))?;
            decode(schema, branch, tag)
        }
        Tmpl::Hook(element, pre_read, _) => {
            pre_read(tag);
            decode(schema, element, tag)
        }
    }
}

/// Decodes one named field of a compound.
fn decode_field(
    schema: &Schema,
    tag: &mut Tag,
    name: &'static str,
    tmpl: &Tmpl,
    optional: bool,
) -> Result<(), DecodeError> {
    let Some(raw) = get(tag, name).cloned() else {
        return if optional {
            Ok(())
        } else {
            Err(DecodeError(format!("missing field {name}")))
        };
    };
    let mut attempt = raw.clone();
    match decode(schema, tmpl, &mut attempt) {
        Ok(()) => set(tag, name, attempt),
        Err(error) if !optional => return Err(error),
        // Lenient optional field: keep the raw data, untyped.
        Err(_) => set(tag, &opaque_key(name), Tag::Byte(0)),
    }
    if SHADOWED_KEYS.contains(&name) {
        set(tag, &shadow_key(name), raw);
    }
    Ok(())
}

fn decode_leaf(leaf: Leaf, tag: &mut Tag) -> Result<(), DecodeError> {
    match leaf {
        Leaf::Int => {
            let value = as_i32(tag).ok_or_else(|| DecodeError("expected a number".into()))?;
            *tag = Tag::Int(value);
            Ok(())
        }
        Leaf::Str => match tag {
            Tag::String(_) => Ok(()),
            _ => Err(DecodeError("expected a string".into())),
        },
        Leaf::NsStr => match tag {
            Tag::String(value) => {
                *value = ensure_namespaced(value);
                Ok(())
            }
            _ => Err(DecodeError("expected a string".into())),
        },
    }
}
