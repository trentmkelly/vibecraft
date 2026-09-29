//! `LootItemCondition.DIRECT_CODEC` / `LootItemConditions`.
//!
//! Conditions the runtime evaluates faithfully decode into their model variant. The
//! rest (entity, item, location and damage source predicates, enchantment based
//! chances, scores, clocks and environment attributes) are validated field by field
//! against the Java record codec and kept as [`LootCondition::Unmodeled`].

use serde_json::Value;

use super::fields::{
    as_bool, as_list, as_object, identifier, int_range, list, optional_bool, required, type_name,
    validate_fields, Kind, Object,
};
use super::numbers::{decode_level_based, decode_number};
use crate::loot_system::{
    EnchantmentBound, HolderSet, LootCondition, NumberProvider, ToolPredicate,
};

/// The `conditions` list of a pool, entry or function.
pub(super) fn decode_conditions(object: &Object) -> Result<Vec<LootCondition>, String> {
    list(object, "conditions")?
        .iter()
        .map(decode_condition)
        .collect()
}

/// `LootItemCondition.DIRECT_CODEC`: a typed condition, or a bare list which is an
/// inline `all_of` (`AllOfCondition.INLINE_CODEC`).
pub(super) fn decode_condition(value: &Value) -> Result<LootCondition, String> {
    if let Value::Array(terms) = value {
        return terms
            .iter()
            .map(decode_condition)
            .collect::<Result<_, _>>()
            .map(LootCondition::AllOf);
    }
    let object = as_object(value, "loot condition")?;
    let kind = type_name(object, "condition")?;
    match kind.as_str() {
        "inverted" => Ok(LootCondition::Inverted(Box::new(decode_condition(
            required(object, "term")?,
        )?))),
        "all_of" | "any_of" => {
            let terms = as_list(required(object, "terms")?, "terms")?
                .iter()
                .map(decode_condition)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(if kind == "all_of" {
                LootCondition::AllOf(terms)
            } else {
                LootCondition::AnyOf(terms)
            })
        }
        "random_chance" => decode_random_chance(object, value),
        "random_chance_with_enchanted_bonus" => decode_enchanted_bonus_chance(object),
        "table_bonus" => decode_table_bonus(object),
        "match_tool" => decode_match_tool(object, value),
        "killed_by_player" => Ok(LootCondition::KilledByPlayer),
        "survives_explosion" => Ok(LootCondition::SurvivesExplosion),
        "block_state_property" => decode_block_state_property(object, value),
        "weather_check" => Ok(LootCondition::WeatherCheck {
            raining: optional_bool(object, "raining")?,
            thundering: optional_bool(object, "thundering")?,
        }),
        "reference" => Ok(LootCondition::Reference(identifier(
            required(object, "name")?,
            "name",
        )?)),
        "enchantment_active_check" => Ok(LootCondition::EnchantmentActiveCheck {
            active: as_bool(required(object, "active")?, "active")?,
        }),
        _ => {
            validate_unmodeled_condition(&kind, object)?;
            Ok(LootCondition::Unmodeled {
                kind: format!("minecraft:{kind}"),
                data: value.clone(),
            })
        }
    }
}

/// `LootItemRandomChanceCondition`: a constant chance is modeled, a computed one
/// (`score`, `sum`, ...) is not.
fn decode_random_chance(object: &Object, value: &Value) -> Result<LootCondition, String> {
    match decode_number(required(object, "chance")?)? {
        NumberProvider::Constant(chance) => Ok(LootCondition::RandomChance(chance)),
        _ => Ok(LootCondition::Unmodeled {
            kind: "minecraft:random_chance".to_string(),
            data: value.clone(),
        }),
    }
}

/// `LootItemRandomChanceWithEnchantedBonusCondition`.
fn decode_enchanted_bonus_chance(object: &Object) -> Result<LootCondition, String> {
    let unenchanted = super::fields::as_float(
        required(object, "unenchanted_chance")?,
        "unenchanted_chance",
    )?;
    if !(0.0..=1.0).contains(&unenchanted) {
        return Err(format!("unenchanted_chance {unenchanted} is not in [0, 1]"));
    }
    Ok(LootCondition::RandomChanceWithEnchantedBonus {
        unenchanted_chance: unenchanted,
        enchanted_chance: decode_level_based(required(object, "enchanted_chance")?)?,
        enchantment: identifier(required(object, "enchantment")?, "enchantment")?,
    })
}

/// `BonusLevelTableCondition`: a non-empty float list per enchantment level.
fn decode_table_bonus(object: &Object) -> Result<LootCondition, String> {
    let chances = as_list(required(object, "chances")?, "chances")?
        .iter()
        .map(|chance| super::fields::as_float(chance, "chances"))
        .collect::<Result<Vec<_>, _>>()?;
    if chances.is_empty() {
        return Err("chances must be a non-empty list".to_string());
    }
    Ok(LootCondition::TableBonus {
        enchantment: identifier(required(object, "enchantment")?, "enchantment")?,
        chances,
    })
}

/// `MatchTool`: the `items` + enchantment-level subset of `ItemPredicate` is
/// modeled; any other predicate component keeps the condition unmodeled.
fn decode_match_tool(object: &Object, value: &Value) -> Result<LootCondition, String> {
    let unmodeled = || LootCondition::Unmodeled {
        kind: "minecraft:match_tool".to_string(),
        data: value.clone(),
    };
    let Some(predicate) = object.get("predicate") else {
        return Ok(unmodeled());
    };
    let predicate = as_object(predicate, "predicate")?;
    if predicate
        .keys()
        .any(|key| key != "items" && key != "predicates")
    {
        return Ok(unmodeled());
    }
    let items = predicate.get("items").map(decode_holder_set).transpose()?;
    let mut enchantments = Vec::new();
    if let Some(components) = predicate.get("predicates") {
        let components = as_object(components, "predicates")?;
        if components.keys().any(|key| key != "minecraft:enchantments") {
            return Ok(unmodeled());
        }
        if let Some(list) = components.get("minecraft:enchantments") {
            for bound in as_list(list, "minecraft:enchantments")? {
                match decode_enchantment_bound(bound)? {
                    Some(bound) => enchantments.push(bound),
                    None => return Ok(unmodeled()),
                }
            }
        }
    }
    Ok(LootCondition::ToolMatches(ToolPredicate {
        items,
        enchantments,
    }))
}

/// `RegistryCodecs.homogeneousList`: an id, a `#tag` or a list of ids.
fn decode_holder_set(value: &Value) -> Result<HolderSet, String> {
    match value {
        Value::String(text) => match text.strip_prefix('#') {
            Some(tag) => Ok(HolderSet::Tag(identifier(
                &Value::String(tag.into()),
                "tag",
            )?)),
            None => Ok(HolderSet::Ids(vec![identifier(value, "id")?])),
        },
        Value::Array(ids) => Ok(HolderSet::Ids(
            ids.iter()
                .map(|id| identifier(id, "id"))
                .collect::<Result<_, _>>()?,
        )),
        other => Err(format!("not an id, tag or list: {other}")),
    }
}

/// `EnchantmentPredicate`; `None` when it carries fields this model lacks.
fn decode_enchantment_bound(value: &Value) -> Result<Option<EnchantmentBound>, String> {
    let object = as_object(value, "enchantment predicate")?;
    if object
        .keys()
        .any(|key| key != "enchantments" && key != "levels")
    {
        return Ok(None);
    }
    let enchantments = object
        .get("enchantments")
        .map(decode_holder_set)
        .transpose()?;
    let (mut min_level, mut max_level) = (None, None);
    match object.get("levels") {
        None => {}
        Some(Value::Number(_)) => {
            let exact = super::fields::as_int(&object["levels"], "levels")?;
            (min_level, max_level) = (Some(exact), Some(exact));
        }
        Some(levels) => {
            let levels = as_object(levels, "levels")?;
            min_level = levels
                .get("min")
                .map(|v| super::fields::as_int(v, "min"))
                .transpose()?;
            max_level = levels
                .get("max")
                .map(|v| super::fields::as_int(v, "max"))
                .transpose()?;
        }
    }
    Ok(Some(EnchantmentBound {
        enchantments,
        min_level,
        max_level,
    }))
}

/// `LootItemBlockStatePropertyCondition`: the block plus every exact property value.
/// A ranged property (`{min, max}`) has no runtime representation and keeps the
/// whole condition as unmodeled.
fn decode_block_state_property(object: &Object, value: &Value) -> Result<LootCondition, String> {
    let block = identifier(required(object, "block")?, "block")?;
    let mut terms = vec![LootCondition::BlockState { block }];
    let mut ranged = false;
    if let Some(properties) = object.get("properties") {
        for (property, expected) in as_object(properties, "properties")? {
            match expected {
                Value::String(expected) => terms.push(LootCondition::BlockStateProperty {
                    property: property.clone(),
                    value: expected.clone(),
                }),
                Value::Object(_) => ranged = true,
                other => return Err(format!("property {property} is not a matcher: {other}")),
            }
        }
    }
    if ranged {
        return Ok(LootCondition::Unmodeled {
            kind: "minecraft:block_state_property".to_string(),
            data: value.clone(),
        });
    }
    Ok(LootCondition::AllOf(terms))
}

/// Field validation for the condition types with no runtime model.
fn validate_unmodeled_condition(kind: &str, object: &Object) -> Result<(), String> {
    match kind {
        "entity_properties" => validate_fields(
            object,
            &[
                ("predicate", Kind::Obj, false),
                ("entity", Kind::Target, true),
            ],
        ),
        "entity_scores" => {
            for range in as_object(required(object, "scores")?, "scores")?.values() {
                int_range(range)?;
            }
            validate_fields(object, &[("entity", Kind::Target, true)])
        }
        "damage_source_properties" => validate_fields(object, &[("predicate", Kind::Obj, false)]),
        "location_check" => validate_fields(
            object,
            &[
                ("predicate", Kind::Obj, false),
                ("offsetX", Kind::Int, false),
                ("offsetY", Kind::Int, false),
                ("offsetZ", Kind::Int, false),
            ],
        ),
        "time_check" => validate_fields(
            object,
            &[
                ("clock", Kind::Id, true),
                ("period", Kind::Long, false),
                ("value", Kind::Range, true),
            ],
        ),
        "value_check" => validate_fields(
            object,
            &[("value", Kind::Number, true), ("range", Kind::Range, true)],
        ),
        "environment_attribute_check" => validate_fields(object, &[("attribute", Kind::Id, true)]),
        other => Err(format!("Unknown condition type 'minecraft:{other}'")),
    }
}
