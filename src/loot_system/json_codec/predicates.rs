//! Codecs of the loot conditions that embed advancement predicates:
//! `entity_properties`, `damage_source_properties` and `location_check`.
//!
//! The predicate documents are first decoded by the advancement condition schema
//! (`Rec::Entity`, `Rec::DamageSource`, `Rec::Location`), which mirrors the strict
//! Java codecs and re-encodes the value. The re-encoded document is then lowered into
//! the loot predicate model; a predicate carrying a field the model does not evaluate
//! (`distance`, `nbt`, `vehicle`, structures, light, ...) leaves the condition
//! `Unmodeled`.

use serde_json::Value;

use super::conditions::{decode_holder_set, decode_tool_predicate};
use super::fields::{as_bool, as_int, as_list, as_object, as_str, entity_target, identifier, Object};
use crate::advancement_condition_schema::{BuiltinLookup, ConditionCodec, Rec};
use crate::loot_system::{
    canonical_component_value, BlockPredicate, DamageSourcePredicate, DoubleBounds,
    EntityFlagsPredicate, EntityPredicate, IntBounds, LocationPredicate, LootCondition,
    LootEntityTarget, ToolPredicate, TypeSpecificPredicate,
};

/// Runs the advancement schema over `value`, returning the re-encoded predicate.
fn validated(rec: Rec, value: &Value) -> Result<Value, String> {
    ConditionCodec::new(&BuiltinLookup).decode_rec(rec, value)
}

fn unmodeled(kind: &str, value: &Value) -> LootCondition {
    LootCondition::Unmodeled {
        kind: format!("minecraft:{kind}"),
        data: value.clone(),
    }
}

/// `LootItemEntityPropertyCondition.MAP_CODEC`.
pub(super) fn decode_entity_properties(
    object: &Object,
    value: &Value,
) -> Result<LootCondition, String> {
    let name = as_str(super::fields::required(object, "entity")?, "entity")?;
    entity_target(&Value::String(name.to_string()), "entity")?;
    let target = LootEntityTarget::by_name(name).ok_or_else(|| format!("Unknown entity '{name}'"))?;
    let Some(predicate) = object.get("predicate") else {
        return Ok(LootCondition::EntityHasProperties {
            target,
            predicate: None,
        });
    };
    let encoded = validated(Rec::Entity, predicate)?;
    Ok(match lower_entity_predicate(&encoded)? {
        Some(predicate) => LootCondition::EntityHasProperties {
            target,
            predicate: Some(predicate),
        },
        None => unmodeled("entity_properties", value),
    })
}

/// `DamageSourceCondition.MAP_CODEC`.
pub(super) fn decode_damage_source_properties(
    object: &Object,
    value: &Value,
) -> Result<LootCondition, String> {
    let Some(predicate) = object.get("predicate") else {
        return Ok(LootCondition::DamageSourceMatches(None));
    };
    let encoded = validated(Rec::DamageSource, predicate)?;
    Ok(match lower_damage_source_predicate(&encoded)? {
        Some(predicate) => LootCondition::DamageSourceMatches(Some(predicate)),
        None => unmodeled("damage_source_properties", value),
    })
}

/// `LocationCheck.MAP_CODEC`: the optional predicate and the `offsetX/Y/Z` block offset.
pub(super) fn decode_location_check(
    object: &Object,
    value: &Value,
) -> Result<LootCondition, String> {
    let offset = |key: &str| -> Result<i32, String> {
        object.get(key).map_or(Ok(0), |value| as_int(value, key))
    };
    let offset = (offset("offsetX")?, offset("offsetY")?, offset("offsetZ")?);
    let Some(predicate) = object.get("predicate") else {
        return Ok(LootCondition::LocationMatches {
            predicate: None,
            offset,
        });
    };
    let encoded = validated(Rec::Location, predicate)?;
    Ok(match lower_location_predicate(&encoded)? {
        Some(predicate) => LootCondition::LocationMatches {
            predicate: Some(predicate),
            offset,
        },
        None => unmodeled("location_check", value),
    })
}

/// Lowers a re-encoded `EntityPredicate`; `None` when it uses an unmodeled field.
fn lower_entity_predicate(encoded: &Value) -> Result<Option<EntityPredicate>, String> {
    let mut predicate = EntityPredicate::default();
    for (key, value) in as_object(encoded, "entity predicate")? {
        match key.as_str() {
            "type" => predicate.entity_type = Some(decode_holder_set(value)?),
            "components" => {
                for (component, expected) in as_object(value, "components")? {
                    let component = identifier(&Value::String(component.clone()), "component")?;
                    let text = canonical_component_value(&component, expected);
                    predicate.components.push((component, text));
                }
            }
            "type_specific" => match lower_type_specific(value)? {
                Some(type_specific) => predicate.type_specific = Some(type_specific),
                None => return Ok(None),
            },
            "equipment" => {
                for (slot, item) in as_object(value, "equipment")? {
                    match lower_item_predicate(item)? {
                        Some(item) => predicate.equipment.push((slot.clone(), item)),
                        None => return Ok(None),
                    }
                }
            }
            "flags" => predicate.flags = Some(lower_flags(value)?),
            "vehicle" => match lower_entity_predicate(value)? {
                Some(vehicle) => predicate.vehicle = Some(Box::new(vehicle)),
                None => return Ok(None),
            },
            _ => return Ok(None),
        }
    }
    Ok(Some(predicate))
}

/// `EntitySubPredicates`: the `sheep`, `slime`, `raider` and `fishing_hook` types
/// vanilla loot uses.
fn lower_type_specific(value: &Value) -> Result<Option<TypeSpecificPredicate>, String> {
    let object = as_object(value, "type_specific")?;
    let kind = identifier(super::fields::required(object, "type")?, "type")?;
    let flag = |key: &str| object.get(key).map(|v| as_bool(v, key)).transpose();
    Ok(match kind.as_str() {
        "minecraft:sheep" => Some(TypeSpecificPredicate::Sheep {
            sheared: flag("sheared")?,
        }),
        "minecraft:slime" => Some(TypeSpecificPredicate::Slime {
            size: object.get("size").map(lower_int_bounds).transpose()?.unwrap_or_default(),
        }),
        "minecraft:raider" => Some(TypeSpecificPredicate::Raider {
            has_raid: flag("has_raid")?.unwrap_or(false),
            is_captain: flag("is_captain")?.unwrap_or(false),
        }),
        "minecraft:fishing_hook" => Some(TypeSpecificPredicate::FishingHook {
            in_open_water: flag("in_open_water")?,
        }),
        _ => None,
    })
}

/// `EntityFlagsPredicate.CODEC`.
fn lower_flags(value: &Value) -> Result<EntityFlagsPredicate, String> {
    let object = as_object(value, "flags")?;
    let flag = |key: &str| object.get(key).map(|v| as_bool(v, key)).transpose();
    Ok(EntityFlagsPredicate {
        is_on_ground: flag("is_on_ground")?,
        is_on_fire: flag("is_on_fire")?,
        is_crouching: flag("is_sneaking")?,
        is_sprinting: flag("is_sprinting")?,
        is_swimming: flag("is_swimming")?,
        is_flying: flag("is_flying")?,
        is_baby: flag("is_baby")?,
        is_in_water: flag("is_in_water")?,
        is_fall_flying: flag("is_fall_flying")?,
    })
}

/// `MinMaxBounds.Ints` as re-encoded by the schema: a number (exact) or `{min, max}`.
fn lower_int_bounds(value: &Value) -> Result<IntBounds, String> {
    if value.is_number() {
        let exact = as_int(value, "bounds")?;
        return Ok(IntBounds {
            min: Some(exact),
            max: Some(exact),
        });
    }
    let object = as_object(value, "bounds")?;
    let bound = |key: &str| object.get(key).map(|v| as_int(v, key)).transpose();
    Ok(IntBounds {
        min: bound("min")?,
        max: bound("max")?,
    })
}

/// Lowers an `ItemPredicate` (`items` and enchantment predicates only).
fn lower_item_predicate(value: &Value) -> Result<Option<ToolPredicate>, String> {
    decode_tool_predicate(as_object(value, "item predicate")?)
}

/// Lowers a re-encoded `DamageSourcePredicate`.
fn lower_damage_source_predicate(encoded: &Value) -> Result<Option<DamageSourcePredicate>, String> {
    let mut predicate = DamageSourcePredicate::default();
    for (key, value) in as_object(encoded, "damage source predicate")? {
        match key.as_str() {
            "tags" => {
                for tag in as_list(value, "tags")? {
                    let tag = as_object(tag, "tag predicate")?;
                    predicate.tags.push((
                        identifier(super::fields::required(tag, "id")?, "id")?,
                        as_bool(super::fields::required(tag, "expected")?, "expected")?,
                    ));
                }
            }
            "direct_entity" | "source_entity" => {
                let Some(entity) = lower_entity_predicate(value)? else {
                    return Ok(None);
                };
                if key == "direct_entity" {
                    predicate.direct_entity = Some(entity);
                } else {
                    predicate.source_entity = Some(entity);
                }
            }
            "is_direct" => predicate.is_direct = Some(as_bool(value, "is_direct")?),
            _ => return Ok(None),
        }
    }
    Ok(Some(predicate))
}

/// `MinMaxBounds.Doubles` as re-encoded by the schema: a number (exact) or `{min, max}`.
fn lower_bounds(value: &Value) -> Result<DoubleBounds, String> {
    if let Some(exact) = value.as_f64() {
        return Ok(DoubleBounds {
            min: Some(exact),
            max: Some(exact),
        });
    }
    let object = as_object(value, "bounds")?;
    let bound = |key: &str| object.get(key).and_then(Value::as_f64);
    Ok(DoubleBounds {
        min: bound("min"),
        max: bound("max"),
    })
}

/// Lowers a re-encoded `LocationPredicate` (`position`, `biomes`, `dimension`, `block`).
fn lower_location_predicate(encoded: &Value) -> Result<Option<LocationPredicate>, String> {
    let mut predicate = LocationPredicate::default();
    for (key, value) in as_object(encoded, "location predicate")? {
        match key.as_str() {
            "position" => {
                let position = as_object(value, "position")?;
                let axis = |name: &str| position.get(name).map(lower_bounds).transpose();
                predicate.position = Some([
                    axis("x")?.unwrap_or_default(),
                    axis("y")?.unwrap_or_default(),
                    axis("z")?.unwrap_or_default(),
                ]);
            }
            "biomes" => predicate.biomes = Some(decode_holder_set(value)?),
            "dimension" => predicate.dimension = Some(identifier(value, "dimension")?),
            "block" => match lower_block_predicate(value)? {
                Some(block) => predicate.block = Some(block),
                None => return Ok(None),
            },
            _ => return Ok(None),
        }
    }
    Ok(Some(predicate))
}

/// Lowers a re-encoded `BlockPredicate`: `blocks` and exact `state` values.
fn lower_block_predicate(value: &Value) -> Result<Option<BlockPredicate>, String> {
    let mut predicate = BlockPredicate::default();
    for (key, value) in as_object(value, "block predicate")? {
        match key.as_str() {
            "blocks" => predicate.blocks = Some(decode_holder_set(value)?),
            "state" => {
                for (property, expected) in as_object(value, "state")? {
                    match expected {
                        Value::String(expected) => {
                            predicate.state.push((property.clone(), expected.clone()));
                        }
                        // A `{min, max}` range matcher is not evaluated.
                        _ => return Ok(None),
                    }
                }
            }
            _ => return Ok(None),
        }
    }
    Ok(Some(predicate))
}
