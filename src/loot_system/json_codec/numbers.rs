//! `NumberProviders.CODEC`, `ScoreboardNameProviders.CODEC` and
//! `LevelBasedValue.CODEC`.

use serde_json::Value;

use super::fields::{
    as_float, as_list, as_object, as_str, entity_target, identifier, required, type_name,
    validate_fields, Kind, Object,
};
use crate::loot_system::{LevelBasedValue, NumberProvider};

/// `NumberProviders.CODEC`: a bare number is a `ConstantValue`, an object with a
/// `type` dispatches on the number provider registry, and an object without one is
/// a `UniformGenerator` (`Codec.withAlternative(TYPED_CODEC, UniformGenerator)`).
pub(super) fn decode_number(value: &Value) -> Result<NumberProvider, String> {
    if let Some(number) = value.as_f64() {
        return Ok(NumberProvider::Constant(number as f32));
    }
    let object = as_object(value, "number provider")?;
    if !object.contains_key("type") {
        return decode_uniform(object);
    }
    match type_name(object, "type")?.as_str() {
        "constant" => Ok(NumberProvider::Constant(as_float(
            required(object, "value")?,
            "value",
        )?)),
        "uniform" => decode_uniform(object),
        "binomial" => Ok(NumberProvider::BinomialProvider {
            n: Box::new(decode_number(required(object, "n")?)?),
            p: Box::new(decode_number(required(object, "p")?)?),
        }),
        "sum" => Ok(NumberProvider::Sum(
            as_list(required(object, "summands")?, "summands")?
                .iter()
                .map(decode_number)
                .collect::<Result<_, _>>()?,
        )),
        "environment_attribute" => Ok(NumberProvider::EnvironmentAttribute {
            attribute: identifier(required(object, "attribute")?, "attribute")?,
        }),
        kind @ ("score" | "storage" | "enchantment_level") => {
            validate_unmodeled_provider(kind, object)?;
            Ok(NumberProvider::Unmodeled {
                kind: format!("minecraft:{kind}"),
                data: value.clone(),
            })
        }
        other => Err(format!("Unknown number provider type 'minecraft:{other}'")),
    }
}

/// `UniformGenerator.MAP_CODEC`.
fn decode_uniform(object: &Object) -> Result<NumberProvider, String> {
    Ok(NumberProvider::UniformProvider {
        min: Box::new(decode_number(required(object, "min")?)?),
        max: Box::new(decode_number(required(object, "max")?)?),
    })
}

/// Field validation of the providers the runtime does not evaluate faithfully.
fn validate_unmodeled_provider(kind: &str, object: &Object) -> Result<(), String> {
    match kind {
        "score" => {
            validate_score_provider(required(object, "target")?)?;
            as_str(required(object, "score")?, "score")?;
            validate_fields(object, &[("scale", Kind::Float, false)])
        }
        "storage" => validate_fields(
            object,
            &[("storage", Kind::Id, true), ("path", Kind::Str, true)],
        ),
        _ => validate_level_based(required(object, "amount")?),
    }
}

/// `ScoreboardNameProviders.CODEC`: an entity target string (context provider) or a
/// typed `fixed`/`context` object.
fn validate_score_provider(value: &Value) -> Result<(), String> {
    if let Value::String(_) = value {
        return entity_target(value, "target").map(drop);
    }
    let object = as_object(value, "score provider")?;
    match type_name(object, "type")?.as_str() {
        "fixed" => as_str(required(object, "name")?, "name").map(drop),
        "context" => entity_target(required(object, "target")?, "target").map(drop),
        other => Err(format!("Unknown score provider type 'minecraft:{other}'")),
    }
}

/// `LevelBasedValue.CODEC`: a bare number (`Constant`) or a typed object.
pub(super) fn decode_level_based(value: &Value) -> Result<LevelBasedValue, String> {
    if let Some(number) = value.as_f64() {
        return Ok(LevelBasedValue::Constant(number as f32));
    }
    let object = as_object(value, "level based value")?;
    let nested = |key: &str| -> Result<Box<LevelBasedValue>, String> {
        decode_level_based(required(object, key)?).map(Box::new)
    };
    let float = |key: &str| as_float(required(object, key)?, key);
    match type_name(object, "type")?.as_str() {
        "constant" => Ok(LevelBasedValue::Constant(float("value")?)),
        "clamped" => Ok(LevelBasedValue::Clamped {
            value: nested("value")?,
            min: float("min")?,
            max: float("max")?,
        }),
        "fraction" => Ok(LevelBasedValue::Fraction {
            numerator: nested("numerator")?,
            denominator: nested("denominator")?,
        }),
        "levels_squared" => Ok(LevelBasedValue::LevelsSquared(float("added")?)),
        "linear" => Ok(LevelBasedValue::Linear {
            base: float("base")?,
            per_level_above_first: float("per_level_above_first")?,
        }),
        "exponent" => Ok(LevelBasedValue::Exponent {
            base: nested("base")?,
            power: nested("power")?,
        }),
        "lookup" => Ok(LevelBasedValue::Lookup {
            values: as_list(required(object, "values")?, "values")?
                .iter()
                .map(|entry| as_float(entry, "values"))
                .collect::<Result<_, _>>()?,
            fallback: nested("fallback")?,
        }),
        other => Err(format!(
            "Unknown level based value type 'minecraft:{other}'"
        )),
    }
}

/// Validation-only form of [`decode_level_based`].
pub(super) fn validate_level_based(value: &Value) -> Result<(), String> {
    decode_level_based(value).map(drop)
}
