//! `LootTable.DIRECT_CODEC` for JSON: decodes the loot tables datapacks ship under
//! `loot_table/**.json` into the runtime [`LootTable`] model.
//!
//! Java decodes every loot table with the full codec and logs+skips tables that fail
//! (`SimpleJsonResourceReloadListener.scanDirectory`). The runtime model
//! ([`LootEntry`], [`LootCondition`], [`LootFunction`]) only represents a subset of the
//! vanilla entry/condition/function types, so this codec decodes exactly that subset
//! and rejects a table that uses anything else with an `unsupported ...` error rather
//! than silently dropping the unsupported part (which would change drops).
//! TODO(loot-json-codec): grow the runtime model and this codec together until every
//! `LootPoolEntries`/`LootItemConditions`/`LootItemFunctions` type decodes.

use serde_json::{Map, Value};

use super::*;

type Object = Map<String, Value>;

/// Decodes one loot table JSON document.
pub fn decode_loot_table(raw: &str) -> Result<LootTable, String> {
    let value: Value = serde_json::from_str(raw).map_err(|err| format!("invalid JSON: {err}"))?;
    let object = as_object(&value, "loot table")?;
    // `LootContextParamSets.CODEC.lenientOptionalFieldOf("type", ALL_PARAMS)`: a
    // missing or unrecognised param set decodes to the default instead of failing.
    let param_set = object
        .get("type")
        .and_then(Value::as_str)
        .and_then(decode_param_set)
        .unwrap_or(LootParamSet::AllParams);
    let random_sequence = object
        .get("random_sequence")
        .map(|value| as_str(value, "random_sequence").map(str::to_string))
        .transpose()?;
    let pools = list(object, "pools")?
        .iter()
        .map(decode_pool)
        .collect::<Result<_, _>>()?;
    let functions = decode_functions(object)?;
    Ok(LootTable {
        param_set,
        random_sequence,
        pools,
        functions,
    })
}

/// `LootContextParamSets.REGISTRY`.
fn decode_param_set(id: &str) -> Option<LootParamSet> {
    Some(match id.strip_prefix("minecraft:").unwrap_or(id) {
        "empty" => LootParamSet::Empty,
        "generic" => LootParamSet::AllParams,
        "block" => LootParamSet::Block,
        "entity" => LootParamSet::Entity,
        "chest" => LootParamSet::Chest,
        "fishing" => LootParamSet::Fishing,
        "archaeology" => LootParamSet::Archaeology,
        "advancement_reward" => LootParamSet::AdvancementReward,
        "gift" => LootParamSet::Gift,
        "barter" => LootParamSet::Barter,
        "vault" => LootParamSet::Vault,
        "command" => LootParamSet::Command,
        "selector" => LootParamSet::Selector,
        "advancement_entity" => LootParamSet::AdvancementEntity,
        "equipment" => LootParamSet::Equipment,
        _ => return None,
    })
}

fn decode_pool(value: &Value) -> Result<LootPool, String> {
    let object = as_object(value, "loot pool")?;
    let rolls = object
        .get("rolls")
        .ok_or_else(|| "loot pool is missing rolls".to_string())
        .and_then(decode_number)?;
    let bonus_rolls = match object.get("bonus_rolls") {
        Some(value) => decode_number(value)?,
        None => NumberProvider::Constant(0.0),
    };
    Ok(LootPool {
        entries: list(object, "entries")?
            .iter()
            .map(decode_entry)
            .collect::<Result<_, _>>()?,
        conditions: decode_conditions(object)?,
        functions: decode_functions(object)?,
        rolls,
        bonus_rolls,
    })
}

/// `NumberProviders.CODEC`: a bare number is `ConstantValue`.
fn decode_number(value: &Value) -> Result<NumberProvider, String> {
    if let Some(number) = value.as_f64() {
        return Ok(NumberProvider::Constant(number as f32));
    }
    let object = as_object(value, "number provider")?;
    let kind = match object.get("type") {
        Some(kind) => as_str(kind, "number provider type")?,
        None => "minecraft:constant",
    };
    match kind {
        "minecraft:constant" => {
            let value = object
                .get("value")
                .and_then(Value::as_f64)
                .ok_or_else(|| "constant number provider is missing value".to_string())?;
            Ok(NumberProvider::Constant(value as f32))
        }
        "minecraft:uniform" => Ok(NumberProvider::UniformProvider {
            min: Box::new(decode_number(required(object, "min")?)?),
            max: Box::new(decode_number(required(object, "max")?)?),
        }),
        "minecraft:binomial" => Ok(NumberProvider::BinomialProvider {
            n: Box::new(decode_number(required(object, "n")?)?),
            p: Box::new(decode_number(required(object, "p")?)?),
        }),
        other => Err(format!("unsupported number provider {other}")),
    }
}

/// `LootPoolEntryContainer.CODEC`.
fn decode_entry(value: &Value) -> Result<LootEntry, String> {
    let object = as_object(value, "loot entry")?;
    let kind = as_str(required(object, "type")?, "entry type")?;
    let conditions = decode_conditions(object)?;
    let weight = int_field(object, "weight", 1)?;
    let quality = int_field(object, "quality", 0)?;
    match kind {
        "minecraft:item" => Ok(LootEntry::Item {
            item: as_str(required(object, "name")?, "name")?.to_string(),
            weight,
            quality,
            conditions,
            functions: decode_functions(object)?,
        }),
        "minecraft:empty" => Ok(LootEntry::Empty {
            weight,
            quality,
            conditions,
        }),
        "minecraft:tag" => Ok(LootEntry::Tag {
            tag: as_str(required(object, "name")?, "name")?.to_string(),
            expand: object.get("expand").and_then(Value::as_bool).unwrap_or(false),
            weight,
            quality,
            conditions,
        }),
        "minecraft:loot_table" => Ok(LootEntry::WeightedNestedTable {
            table: as_str(required(object, "value")?, "value")?.to_string(),
            weight,
            quality,
            conditions,
        }),
        "minecraft:dynamic" => Ok(LootEntry::Dynamic(
            as_str(required(object, "name")?, "name")?.to_string(),
        )),
        "minecraft:alternatives" | "minecraft:sequence" | "minecraft:group" => {
            if !conditions.is_empty() {
                return Err(format!("unsupported conditions on composite entry {kind}"));
            }
            let children = list(object, "children")?
                .iter()
                .map(decode_entry)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(match kind {
                "minecraft:alternatives" => LootEntry::Alternatives(children),
                "minecraft:sequence" => LootEntry::Sequence(children),
                _ => LootEntry::Group(children),
            })
        }
        other => Err(format!("unsupported loot entry type {other}")),
    }
}

fn decode_conditions(object: &Object) -> Result<Vec<LootCondition>, String> {
    list(object, "conditions")?
        .iter()
        .map(decode_condition)
        .collect()
}

/// `LootItemCondition.CODEC`.
fn decode_condition(value: &Value) -> Result<LootCondition, String> {
    let object = as_object(value, "loot condition")?;
    let kind = as_str(required(object, "condition")?, "condition")?;
    match kind {
        "minecraft:random_chance" => match required(object, "chance")? {
            Value::Number(chance) => Ok(LootCondition::RandomChance(
                chance.as_f64().unwrap_or_default() as f32,
            )),
            _ => Err("unsupported non-constant random_chance chance".to_string()),
        },
        "minecraft:killed_by_player" => Ok(LootCondition::KilledByPlayer),
        "minecraft:survives_explosion" => Ok(LootCondition::SurvivesExplosion),
        "minecraft:inverted" => Ok(LootCondition::Inverted(Box::new(decode_condition(
            required(object, "term")?,
        )?))),
        "minecraft:all_of" | "minecraft:any_of" => {
            let terms = list(object, "terms")?
                .iter()
                .map(decode_condition)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(if kind == "minecraft:all_of" {
                LootCondition::AllOf(terms)
            } else {
                LootCondition::AnyOf(terms)
            })
        }
        "minecraft:reference" => Ok(LootCondition::Reference(
            as_str(required(object, "name")?, "name")?.to_string(),
        )),
        "minecraft:weather_check" => Ok(LootCondition::WeatherCheck {
            raining: object.get("raining").and_then(Value::as_bool),
            thundering: object.get("thundering").and_then(Value::as_bool),
        }),
        "minecraft:block_state_property" => decode_block_state_property(object),
        other => Err(format!("unsupported loot condition {other}")),
    }
}

/// `LootItemBlockStatePropertyCondition`: the block plus every exact property value.
fn decode_block_state_property(object: &Object) -> Result<LootCondition, String> {
    let mut terms = vec![LootCondition::BlockState {
        block: as_str(required(object, "block")?, "block")?.to_string(),
    }];
    if let Some(properties) = object.get("properties") {
        for (property, value) in as_object(properties, "properties")? {
            let Value::String(value) = value else {
                return Err(format!("unsupported ranged block state property {property}"));
            };
            terms.push(LootCondition::BlockStateProperty {
                property: property.clone(),
                value: value.clone(),
            });
        }
    }
    Ok(LootCondition::AllOf(terms))
}

fn decode_functions(object: &Object) -> Result<Vec<LootFunction>, String> {
    list(object, "functions")?
        .iter()
        .map(decode_function)
        .collect()
}

/// `LootItemFunction.CODEC`.
fn decode_function(value: &Value) -> Result<LootFunction, String> {
    let object = as_object(value, "loot function")?;
    let kind = as_str(required(object, "function")?, "function")?;
    if !list(object, "conditions")?.is_empty() {
        return Err(format!("unsupported conditions on function {kind}"));
    }
    match kind {
        "minecraft:set_count" => {
            if object.get("add").and_then(Value::as_bool).unwrap_or(false) {
                return Err("unsupported set_count with add".to_string());
            }
            Ok(LootFunction::SetCount(decode_number(required(
                object, "count",
            )?)?))
        }
        "minecraft:limit_count" => decode_limit_count(object),
        "minecraft:explosion_decay" => Ok(LootFunction::ApplyExplosionDecay),
        "minecraft:furnace_smelt" => Ok(LootFunction::SmeltItem),
        other => Err(format!("unsupported loot function {other}")),
    }
}

/// `LimitCount` with an `IntRange` that has both bounds as constants.
fn decode_limit_count(object: &Object) -> Result<LootFunction, String> {
    let limit = required(object, "limit")?;
    let (min, max) = match limit {
        Value::Number(bound) => {
            let bound = bound.as_i64().ok_or("limit is not an integer")? as i32;
            (bound, bound)
        }
        Value::Object(range) => (
            int_field(range, "min", i32::MIN)?,
            int_field(range, "max", i32::MAX)?,
        ),
        _ => return Err("limit is not an int range".to_string()),
    };
    Ok(LootFunction::LimitCount { min, max })
}

fn required<'a>(object: &'a Object, key: &str) -> Result<&'a Value, String> {
    object.get(key).ok_or_else(|| format!("missing field {key}"))
}

fn list<'a>(object: &'a Object, key: &str) -> Result<&'a [Value], String> {
    match object.get(key) {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        Some(other) => Err(format!("{key} is not a list: {other}")),
    }
}

fn int_field(object: &Object, key: &str, default: i32) -> Result<i32, String> {
    match object.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_i64()
            .and_then(|number| i32::try_from(number).ok())
            .ok_or_else(|| format!("{key} is not an integer: {value}")),
    }
}

fn as_object<'a>(value: &'a Value, what: &str) -> Result<&'a Object, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{what} is not a JSON object"))
}

fn as_str<'a>(value: &'a Value, what: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{what} is not a string"))
}

#[cfg(test)]
mod tests;
