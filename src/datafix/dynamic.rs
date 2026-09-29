//! `Dynamic`-style helpers over NBT [`Tag`] trees.
//!
//! Java fixes are written against `com.mojang.serialization.Dynamic<Tag>` with
//! `NbtOps`. These helpers reproduce the `NbtOps` semantics the fixes rely on
//! (string access only succeeds for string tags, numeric access accepts any
//! numeric tag and narrows like `Number.intValue()`, and so on) so each fix can
//! stay a close, readable transcription of its Java counterpart.

use crate::registry::Identifier;
use crate::storage::nbt::Tag;

/// `NamespacedSchema.ensureNamespaced`: adds the `minecraft:` namespace to a
/// valid identifier and leaves anything unparseable untouched.
pub fn ensure_namespaced(input: &str) -> String {
    match Identifier::try_parse(input) {
        Some(identifier) => identifier.to_string(),
        None => input.to_string(),
    }
}

/// `Dynamic.get(key)` on a compound.
pub fn get<'a>(tag: &'a Tag, key: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(entries) => entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value),
        _ => None,
    }
}

/// Mutable variant of [`get`].
pub fn get_mut<'a>(tag: &'a mut Tag, key: &str) -> Option<&'a mut Tag> {
    match tag {
        Tag::Compound(entries) => entries
            .iter_mut()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value),
        _ => None,
    }
}

/// `Dynamic.set(key, value)`: replaces in place or appends. No-op on non-compounds.
pub fn set(tag: &mut Tag, key: &str, value: Tag) {
    if let Tag::Compound(entries) = tag {
        match entries.iter_mut().find(|(name, _)| name == key) {
            Some((_, slot)) => *slot = value,
            None => entries.push((key.to_string(), value)),
        }
    }
}

/// `Dynamic.remove(key)`: returns the removed value.
pub fn remove(tag: &mut Tag, key: &str) -> Option<Tag> {
    match tag {
        Tag::Compound(entries) => {
            let index = entries.iter().position(|(name, _)| name == key)?;
            Some(entries.remove(index).1)
        }
        _ => None,
    }
}

/// `Dynamic.renameField(from, to)`: moves a field when present.
pub fn rename_field(tag: &mut Tag, from: &str, to: &str) {
    if let Some(value) = remove(tag, from) {
        set(tag, to, value);
    }
}

/// `Dynamic.asString().result()`: only string tags convert.
pub fn as_str(tag: &Tag) -> Option<&str> {
    match tag {
        Tag::String(value) => Some(value),
        _ => None,
    }
}

/// `Dynamic.get(key).asString().result()`.
pub fn get_str<'a>(tag: &'a Tag, key: &str) -> Option<&'a str> {
    get(tag, key).and_then(as_str)
}

/// `Dynamic.get(key).asString("")`.
pub fn get_str_or<'a>(tag: &'a Tag, key: &str, default: &'a str) -> &'a str {
    get_str(tag, key).unwrap_or(default)
}

/// `Dynamic.asNumber().result()` widened to `f64` (Java `Number.doubleValue()`).
pub fn as_f64(tag: &Tag) -> Option<f64> {
    match tag {
        Tag::Byte(v) => Some(f64::from(*v)),
        Tag::Short(v) => Some(f64::from(*v)),
        Tag::Int(v) => Some(f64::from(*v)),
        Tag::Long(v) => Some(*v as f64),
        Tag::Float(v) => Some(f64::from(*v)),
        Tag::Double(v) => Some(*v),
        _ => None,
    }
}

/// `Dynamic.asNumber().result()` narrowed to `long` (Java `Number.longValue()`).
pub fn as_i64(tag: &Tag) -> Option<i64> {
    match tag {
        Tag::Byte(v) => Some(i64::from(*v)),
        Tag::Short(v) => Some(i64::from(*v)),
        Tag::Int(v) => Some(i64::from(*v)),
        Tag::Long(v) => Some(*v),
        Tag::Float(v) => Some(*v as i64),
        Tag::Double(v) => Some(*v as i64),
        _ => None,
    }
}

/// `Dynamic.asNumber().result()` narrowed to `int` (Java `Number.intValue()`).
pub fn as_i32(tag: &Tag) -> Option<i32> {
    match tag {
        Tag::Long(v) => Some(*v as i32),
        Tag::Float(v) => Some(*v as i32),
        Tag::Double(v) => Some(*v as i32),
        other => as_i64(other).map(|v| v as i32),
    }
}

/// `Dynamic.get(key).asInt(default)`.
pub fn get_i32_or(tag: &Tag, key: &str, default: i32) -> i32 {
    get(tag, key).and_then(as_i32).unwrap_or(default)
}

/// `Dynamic.get(key).asLong(default)`.
pub fn get_i64_or(tag: &Tag, key: &str, default: i64) -> i64 {
    get(tag, key).and_then(as_i64).unwrap_or(default)
}

/// `Dynamic.get(key).asDouble(default)`.
pub fn get_f64_or(tag: &Tag, key: &str, default: f64) -> f64 {
    get(tag, key).and_then(as_f64).unwrap_or(default)
}

/// `Dynamic.get(key).asBoolean(default)`: numeric tags are true when non-zero.
pub fn get_bool_or(tag: &Tag, key: &str, default: bool) -> bool {
    get(tag, key)
        .and_then(as_i64)
        .map_or(default, |value| value != 0)
}

/// `Dynamic.asStream()` on a list tag (also accepts the primitive arrays that
/// `NbtOps.getStream` treats as collections).
pub fn list_items(tag: &Tag) -> Option<Vec<Tag>> {
    match tag {
        Tag::List(items) => Some(items.clone()),
        Tag::ByteArray(items) => Some(items.iter().map(|v| Tag::Byte(*v)).collect()),
        Tag::IntArray(items) => Some(items.iter().map(|v| Tag::Int(*v)).collect()),
        Tag::LongArray(items) => Some(items.iter().map(|v| Tag::Long(*v)).collect()),
        _ => None,
    }
}

/// Builds a compound from `(name, value)` pairs (`Dynamic.emptyMap()` + `set`).
pub fn compound(entries: Vec<(&str, Tag)>) -> Tag {
    Tag::Compound(
        entries
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
    )
}

/// `Number.shortValue()` of a numeric tag.
pub fn as_i16(tag: &Tag) -> Option<i16> {
    as_i32(tag).map(|v| v as i16)
}

/// `Number.byteValue()` of a numeric tag.
pub fn as_i8(tag: &Tag) -> Option<i8> {
    as_i32(tag).map(|v| v as i8)
}

/// `Number.floatValue()` of a numeric tag.
pub fn as_f32(tag: &Tag) -> Option<f32> {
    match tag {
        Tag::Byte(v) => Some(f32::from(*v)),
        Tag::Short(v) => Some(f32::from(*v)),
        Tag::Int(v) => Some(*v as f32),
        Tag::Long(v) => Some(*v as f32),
        Tag::Float(v) => Some(*v),
        Tag::Double(v) => Some(*v as f32),
        _ => None,
    }
}

/// `Dynamic.get(key).asShort(default)`.
pub fn get_i16_or(tag: &Tag, key: &str, default: i16) -> i16 {
    get(tag, key).and_then(as_i16).unwrap_or(default)
}

/// `Dynamic.get(key).asByte(default)`.
pub fn get_i8_or(tag: &Tag, key: &str, default: i8) -> i8 {
    get(tag, key).and_then(as_i8).unwrap_or(default)
}

/// `Dynamic.get(key).asFloat(default)`.
pub fn get_f32_or(tag: &Tag, key: &str, default: f32) -> f32 {
    get(tag, key).and_then(as_f32).unwrap_or(default)
}

/// `Dynamic.createBoolean(value)`: NBT booleans are byte tags.
pub fn boolean(value: bool) -> Tag {
    Tag::Byte(i8::from(value))
}

/// Removes and returns the `id` key of an entity/block entity compound, the
/// way a tagged choice hands the discriminator to the surrounding pair and keeps
/// only the branch value.
pub fn take_id(tag: &mut Tag) -> Option<String> {
    match remove(tag, "id") {
        Some(Tag::String(id)) => Some(id),
        Some(other) => {
            set(tag, "id", other);
            None
        }
        None => None,
    }
}

/// `Dynamic.asByteBufferOpt()`: byte arrays as they are, other numeric
/// collections (lists, int/long arrays) narrowed element by element.
pub fn as_byte_buffer(tag: &Tag) -> Option<Vec<i8>> {
    match tag {
        Tag::ByteArray(bytes) => Some(bytes.clone()),
        other => list_items(other)?
            .iter()
            .map(|item| as_i32(item).map(|value| value as i8))
            .collect(),
    }
}

/// `Integer.parseInt` (optional sign, ASCII digits, must fit an `int`); `None`
/// where Java throws `NumberFormatException`.
pub fn parse_java_int(text: &str) -> Option<i32> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.strip_prefix('+').unwrap_or(text).parse().ok()
}

/// Structural equality that ignores compound key order, matching Java's
/// `HashMap`-backed `CompoundTag.equals`.
pub fn equal_unordered(a: &Tag, b: &Tag) -> bool {
    canonical(a) == canonical(b)
}

/// Recursively sorts compound keys, giving a canonical form for comparison.
pub fn canonical(tag: &Tag) -> Tag {
    match tag {
        Tag::Compound(entries) => {
            let mut sorted: Vec<(String, Tag)> = entries
                .iter()
                .map(|(name, value)| (name.clone(), canonical(value)))
                .collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            Tag::Compound(sorted)
        }
        Tag::List(items) => Tag::List(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}
