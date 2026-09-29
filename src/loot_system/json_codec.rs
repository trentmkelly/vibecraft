//! `LootTable.DIRECT_CODEC` for JSON: decodes the loot tables datapacks ship under
//! `loot_table/**.json` into the runtime [`LootTable`] model.
//!
//! Java decodes every loot table with the full codec and logs+skips tables that fail
//! (`SimpleJsonResourceReloadListener.scanDirectory`). This codec accepts exactly the
//! documents the Java codec accepts at the structural level: every registered entry,
//! condition, function and number provider type is decoded and its record fields are
//! validated. Parts of the document the runtime cannot evaluate yet decode into
//! `Unmodeled` model variants (see [`LootTable::is_fully_modeled`]) instead of being
//! dropped, so nothing is silently lost.
//! TODO(loot-json-codec): grow the runtime model until no `Unmodeled` variant is
//! needed, and decode item components/predicates with their own codecs.

use serde_json::Value;

use super::*;

mod conditions;
mod fields;
mod functions;
mod numbers;

use conditions::decode_conditions;
use fields::{as_bool, as_list, as_object, identifier, int_field, list, required, type_name, Object};
use functions::decode_functions;
use numbers::decode_number;

/// Decodes one loot table JSON document.
pub fn decode_loot_table(raw: &str) -> Result<LootTable, String> {
    let value: Value = serde_json::from_str(raw).map_err(|err| format!("invalid JSON: {err}"))?;
    decode_table(&value)
}

/// `LootTable.DIRECT_CODEC`.
fn decode_table(value: &Value) -> Result<LootTable, String> {
    let object = as_object(value, "loot table")?;
    // `LootContextParamSets.CODEC.lenientOptionalFieldOf("type", ALL_PARAMS)`: a
    // missing or unrecognised param set decodes to the default instead of failing.
    let param_set = object
        .get("type")
        .and_then(Value::as_str)
        .and_then(decode_param_set)
        .unwrap_or(LootParamSet::AllParams);
    let random_sequence = object
        .get("random_sequence")
        .map(|value| identifier(value, "random_sequence"))
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
        "villager_trade" => LootParamSet::VillagerTrade,
        "advancement_location" => LootParamSet::AdvancementLocation,
        "block_use" => LootParamSet::BlockUse,
        "shearing" => LootParamSet::Shearing,
        "entity_interact" => LootParamSet::EntityInteract,
        "block_interact" => LootParamSet::BlockInteract,
        "enchanted_damage" => LootParamSet::EnchantedDamage,
        "enchanted_item" => LootParamSet::EnchantedItem,
        "enchanted_location" => LootParamSet::EnchantedLocation,
        "enchanted_entity" => LootParamSet::EnchantedEntity,
        "hit_block" => LootParamSet::HitBlock,
        _ => return None,
    })
}

/// `LootPool.CODEC`.
fn decode_pool(value: &Value) -> Result<LootPool, String> {
    let object = as_object(value, "loot pool")?;
    let entries = as_list(required(object, "entries")?, "entries")?
        .iter()
        .map(decode_entry)
        .collect::<Result<_, _>>()?;
    let rolls = decode_number(required(object, "rolls")?)?;
    let bonus_rolls = match object.get("bonus_rolls") {
        Some(value) => decode_number(value)?,
        None => NumberProvider::Constant(0.0),
    };
    Ok(LootPool {
        entries,
        conditions: decode_conditions(object)?,
        functions: decode_functions(object)?,
        rolls,
        bonus_rolls,
    })
}

/// The `singletonFields` shared by item-producing entries.
struct Singleton {
    weight: i32,
    quality: i32,
    conditions: Vec<LootCondition>,
    functions: Vec<LootFunction>,
}

impl Singleton {
    /// `LootPoolSingletonContainer.singletonFields`.
    fn decode(object: &Object) -> Result<Self, String> {
        Ok(Self {
            weight: int_field(object, "weight", 1)?,
            quality: int_field(object, "quality", 0)?,
            conditions: decode_conditions(object)?,
            functions: decode_functions(object)?,
        })
    }
}

/// `LootPoolEntries.CODEC`: dispatches on the entry `type`.
fn decode_entry(value: &Value) -> Result<LootEntry, String> {
    let object = as_object(value, "loot entry")?;
    let kind = type_name(object, "type")?;
    match kind.as_str() {
        "alternatives" | "sequence" | "group" => decode_composite(&kind, object),
        "empty" | "item" | "tag" | "loot_table" | "dynamic" | "slots" => {
            decode_singleton(&kind, object, value)
        }
        other => Err(format!("Unknown loot entry type 'minecraft:{other}'")),
    }
}

/// `CompositeEntryBase.createCodec`: children plus the container's own conditions.
fn decode_composite(kind: &str, object: &Object) -> Result<LootEntry, String> {
    let children = list(object, "children")?
        .iter()
        .map(decode_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let entry = match kind {
        "alternatives" => LootEntry::Alternatives(children),
        "sequence" => LootEntry::Sequence(children),
        _ => LootEntry::Group(children),
    };
    let conditions = decode_conditions(object)?;
    Ok(if conditions.is_empty() {
        entry
    } else {
        LootEntry::Conditional {
            conditions,
            entry: Box::new(entry),
        }
    })
}

/// The `LootPoolSingletonContainer` entry types.
fn decode_singleton(kind: &str, object: &Object, value: &Value) -> Result<LootEntry, String> {
    let Singleton {
        weight,
        quality,
        conditions,
        functions,
    } = Singleton::decode(object)?;
    Ok(match kind {
        "empty" => LootEntry::Empty {
            weight,
            quality,
            conditions,
            functions,
        },
        "item" => LootEntry::Item {
            item: identifier(required(object, "name")?, "name")?,
            weight,
            quality,
            conditions,
            functions,
        },
        "tag" => LootEntry::Tag {
            tag: identifier(required(object, "name")?, "name")?,
            expand: as_bool(required(object, "expand")?, "expand")?,
            weight,
            quality,
            conditions,
            functions,
        },
        "dynamic" => LootEntry::Dynamic {
            name: identifier(required(object, "name")?, "name")?,
            weight,
            quality,
            conditions,
            functions,
        },
        "loot_table" => match required(object, "value")? {
            // `Codec.either(LootTable.KEY_CODEC, LootTable.DIRECT_CODEC)`.
            Value::String(_) => LootEntry::WeightedNestedTable {
                table: identifier(required(object, "value")?, "value")?,
                weight,
                quality,
                conditions,
                functions,
            },
            inline => {
                decode_table(inline)?;
                unmodeled_entry(kind, value)
            }
        },
        _ => {
            as_object(required(object, "slot_source")?, "slot_source")?;
            unmodeled_entry(kind, value)
        }
    })
}

/// A validated entry the runtime cannot evaluate yet.
fn unmodeled_entry(kind: &str, value: &Value) -> LootEntry {
    LootEntry::Unmodeled {
        kind: format!("minecraft:{kind}"),
        data: value.clone(),
    }
}

#[cfg(test)]
mod tests;
