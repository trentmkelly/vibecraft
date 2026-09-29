//! JSON field access shared by the loot table codec modules: the primitive
//! `Codec.INT`/`Codec.BOOL`/`Identifier.CODEC` decoders and the small
//! record-field schema used to validate loot function and condition parameters
//! whose runtime behaviour is not modeled yet.

use serde_json::{Map, Value};

use crate::registry::Identifier;

/// A decoded JSON object.
pub(super) type Object = Map<String, Value>;

/// `RecordCodecBuilder` `fieldOf`: the field must be present.
pub(super) fn required<'a>(object: &'a Object, key: &str) -> Result<&'a Value, String> {
    object
        .get(key)
        .ok_or_else(|| format!("missing field {key}"))
}

/// `listOf().optionalFieldOf(key, List.of())`.
pub(super) fn list<'a>(object: &'a Object, key: &str) -> Result<&'a [Value], String> {
    match object.get(key) {
        None => Ok(&[]),
        Some(value) => as_list(value, key),
    }
}

/// A JSON array.
pub(super) fn as_list<'a>(value: &'a Value, what: &str) -> Result<&'a [Value], String> {
    match value {
        Value::Array(values) => Ok(values),
        other => Err(format!("{what} is not a list: {other}")),
    }
}

/// `Codec.INT.optionalFieldOf(key, default)`.
pub(super) fn int_field(object: &Object, key: &str, default: i32) -> Result<i32, String> {
    match object.get(key) {
        None => Ok(default),
        Some(value) => as_int(value, key),
    }
}

/// `Codec.INT`: an integral number that fits in 32 bits.
pub(super) fn as_int(value: &Value, what: &str) -> Result<i32, String> {
    value
        .as_i64()
        .and_then(|number| i32::try_from(number).ok())
        .ok_or_else(|| format!("{what} is not an integer: {value}"))
}

/// `Codec.FLOAT`: any JSON number.
pub(super) fn as_float(value: &Value, what: &str) -> Result<f32, String> {
    value
        .as_f64()
        .map(|number| number as f32)
        .ok_or_else(|| format!("{what} is not a number: {value}"))
}

/// `Codec.BOOL`.
pub(super) fn as_bool(value: &Value, what: &str) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("{what} is not a boolean: {value}"))
}

/// `Codec.BOOL.optionalFieldOf(key, default)`.
pub(super) fn bool_or(object: &Object, key: &str, default: bool) -> Result<bool, String> {
    object
        .get(key)
        .map_or(Ok(default), |value| as_bool(value, key))
}

/// `Codec.BOOL.optionalFieldOf(key)`.
pub(super) fn optional_bool(object: &Object, key: &str) -> Result<Option<bool>, String> {
    object.get(key).map(|value| as_bool(value, key)).transpose()
}

pub(super) fn as_object<'a>(value: &'a Value, what: &str) -> Result<&'a Object, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{what} is not a JSON object"))
}

pub(super) fn as_str<'a>(value: &'a Value, what: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{what} is not a string"))
}

/// `Identifier.CODEC`: a `namespace:path` string (the namespace defaults to
/// `minecraft`), returned in its canonical form.
pub(super) fn identifier(value: &Value, what: &str) -> Result<String, String> {
    let text = as_str(value, what)?;
    Identifier::parse(text)
        .map(|id| id.to_string())
        .map_err(|err| format!("{what}: {err}"))
}

/// A registry type name (`function`, `condition`, `type` fields): a valid
/// identifier in the `minecraft` namespace, returned as its bare path.
pub(super) fn type_name(object: &Object, key: &str) -> Result<String, String> {
    let id = identifier(required(object, key)?, key)?;
    id.strip_prefix("minecraft:")
        .map(str::to_string)
        .ok_or_else(|| format!("Unknown {key} '{id}'"))
}

/// `LootContext.EntityTarget.CODEC`.
pub(super) fn entity_target(value: &Value, what: &str) -> Result<String, String> {
    let name = as_str(value, what)?;
    const TARGETS: [&str; 6] = [
        "this",
        "attacker",
        "direct_attacker",
        "attacking_player",
        "target_entity",
        "interacting_entity",
    ];
    if TARGETS.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(format!("Unknown {what} '{name}'"))
    }
}

/// `IntRange.CODEC`: a bare int or `{min?, max?}` number providers. Returns the
/// bounds when both are absent-or-constant integers, so callers can lower the
/// range to the runtime model.
pub(super) fn int_range(value: &Value) -> Result<Option<(i32, i32)>, String> {
    if value.is_number() {
        // `Codec.INT` exact range.
        let exact = as_int(value, "range")?;
        return Ok(Some((exact, exact)));
    }
    let object = as_object(value, "int range")?;
    let mut bounds = [Some(i32::MIN), Some(i32::MAX)];
    for (slot, key) in bounds.iter_mut().zip(["min", "max"]) {
        let Some(bound) = object.get(key) else {
            continue;
        };
        // Every bound must be a valid number provider; a constant one rounds like
        // `ConstantValue.getInt` (`Math.round`).
        *slot = match super::numbers::decode_number(bound)? {
            crate::loot_system::NumberProvider::Constant(value) => {
                Some((value + 0.5).floor() as i32)
            }
            _ => None,
        };
    }
    Ok(match bounds {
        [Some(min), Some(max)] => Some((min, max)),
        _ => None,
    })
}

/// The JSON type a schema field must have.
#[derive(Clone, Copy)]
pub(super) enum Kind {
    /// `NumberProviders.CODEC`.
    Number,
    /// `LevelBasedValue.CODEC`.
    LevelBased,
    /// `Identifier.CODEC` (registry holders and keys).
    Id,
    /// A registry holder set: an id, a `#tag` or a list of ids.
    IdSet,
    /// `Codec.INT`.
    Int,
    /// `Codec.LONG`.
    Long,
    /// `Codec.FLOAT`.
    Float,
    /// `Codec.BOOL`.
    Bool,
    /// `Codec.STRING`.
    Str,
    /// A JSON object (predicates and other nested records).
    Obj,
    /// A JSON list.
    List,
    /// `LootContext.EntityTarget`.
    Target,
    /// `IntRange.CODEC`.
    Range,
    /// Any value; only its presence is checked (text components, NBT, ...).
    Any,
    /// `LootItemFunctions.ROOT_CODEC`.
    Function,
    /// `ListOperation.codec`: the `mode` discriminator of a list operation.
    ListMode,
}

/// One record field: its name, type and whether `fieldOf` (rather than
/// `optionalFieldOf`) declares it.
pub(super) type Field = (&'static str, Kind, bool);

/// Validates `object` against a record schema. Unknown keys are ignored, like
/// `RecordCodecBuilder`.
pub(super) fn validate_fields(object: &Object, schema: &[Field]) -> Result<(), String> {
    for (key, kind, is_required) in schema {
        match object.get(*key) {
            None if *is_required => return Err(format!("missing field {key}")),
            None => {}
            Some(value) => validate_kind(value, *kind, key)?,
        }
    }
    Ok(())
}

/// Validates one JSON value against a [`Kind`].
pub(super) fn validate_kind(value: &Value, kind: Kind, what: &str) -> Result<(), String> {
    match kind {
        Kind::Number => super::numbers::decode_number(value).map(drop),
        Kind::LevelBased => super::numbers::validate_level_based(value),
        Kind::Id => identifier(value, what).map(drop),
        Kind::IdSet => validate_id_set(value, what),
        Kind::Int => as_int(value, what).map(drop),
        Kind::Long => value
            .as_i64()
            .map(drop)
            .ok_or_else(|| format!("{what} is not a long: {value}")),
        Kind::Float => as_float(value, what).map(drop),
        Kind::Bool => as_bool(value, what).map(drop),
        Kind::Str => as_str(value, what).map(drop),
        Kind::Obj => as_object(value, what).map(drop),
        Kind::List => as_list(value, what).map(drop),
        Kind::Target => entity_target(value, what).map(drop),
        Kind::Range => int_range(value).map(drop),
        Kind::Any => Ok(()),
        Kind::Function => super::functions::decode_function(value).map(drop),
        Kind::ListMode => validate_list_mode(value, what),
    }
}

/// `RegistryCodecs.homogeneousList`: a single id, a `#tag` string or a list of ids.
fn validate_id_set(value: &Value, what: &str) -> Result<(), String> {
    match value {
        Value::String(text) => {
            let text = text.strip_prefix('#').unwrap_or(text);
            Identifier::parse(text)
                .map(drop)
                .map_err(|err| format!("{what}: {err}"))
        }
        Value::Array(values) => values
            .iter()
            .try_for_each(|entry| identifier(entry, what).map(drop)),
        other => Err(format!("{what} is not an id, tag or list: {other}")),
    }
}

/// `ListOperation.Type.CODEC`.
fn validate_list_mode(value: &Value, what: &str) -> Result<(), String> {
    match as_str(value, what)? {
        "replace_all" | "replace_section" | "insert" | "append" => Ok(()),
        other => Err(format!("Unknown list operation mode '{other}'")),
    }
}
