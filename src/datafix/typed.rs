//! Access to *typed* fields of a decoded value.
//!
//! Java fixes reach nested values through `Typed` optics (`getOptionalTyped`,
//! `updateTyped`, `getOrCreateTyped`, ...) or through the `remainder`
//! `Dynamic`. The remainder holds raw data and is reachable no matter what, but
//! typed fields only exist when they were read successfully: an optional field
//! that failed to decode is absent from the typed value (see
//! [`decode`](super::decode)). These helpers give fixes that typed view: a field
//! marked opaque behaves as if it was missing.

use crate::storage::nbt::Tag;

use super::decode::{clear_opaque, is_opaque};
use super::dynamic::{get, get_mut, set};

/// `Typed.getOptionalTyped(field)`: the field's value, unless it is absent or was
/// not readable with the schema type.
pub fn typed<'a>(tag: &'a Tag, key: &str) -> Option<&'a Tag> {
    if is_opaque(tag, key) {
        None
    } else {
        get(tag, key)
    }
}

/// Mutable variant of [`typed`].
pub fn typed_mut<'a>(tag: &'a mut Tag, key: &str) -> Option<&'a mut Tag> {
    if is_opaque(tag, key) {
        None
    } else {
        get_mut(tag, key)
    }
}

/// `Typed.set(field, value)` for a value written by a fix: the field is typed
/// from now on.
pub fn set_typed(tag: &mut Tag, key: &str, value: Tag) {
    clear_opaque(tag, key);
    set(tag, key, value);
}

/// `Typed.getOrCreateTyped(field)` followed by an update, for a compound field:
/// an absent or unreadable field is replaced by an empty compound first.
pub fn update_typed_compound(tag: &mut Tag, key: &str, update: impl FnOnce(&mut Tag)) {
    if typed(tag, key).is_none() {
        set_typed(tag, key, Tag::Compound(Vec::new()));
    }
    if let Some(field) = get_mut(tag, key) {
        update(field);
    }
}
