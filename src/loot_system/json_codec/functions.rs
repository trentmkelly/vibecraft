//! `LootItemFunctions.ROOT_CODEC` / `LootItemFunction` types.
//!
//! Functions the runtime applies faithfully decode into their model variant. Every
//! other registered function type is validated field by field against its Java record
//! codec (see [`function_schema`]) and kept as [`LootFunction::Unmodeled`].
//! Component payloads (`DataComponentPatch`, text components, NBT) are checked for
//! presence and JSON shape only; TODO(loot-json-codec): decode them with the item
//! component codecs once those exist.

use serde_json::Value;

use super::conditions::decode_conditions;
use super::item_functions::{
    decode_copy_components, decode_copy_state, decode_enchant_randomly, decode_enchant_with_levels,
    decode_furnace_smelt, decode_set_components, decode_set_damage, decode_set_enchantments,
    decode_set_instrument, decode_set_name, decode_set_stew_effect,
};
use super::fields::{
    as_float, as_int, as_list, as_object, bool_or, identifier, int_field, int_range, list,
    required, type_name, validate_fields, Field, Kind, Object,
};
use super::numbers::decode_number;
use crate::loot_system::{LootBonusFormula, LootCondition, LootFunction};

/// The `functions` list of a pool, entry or table.
pub(super) fn decode_functions(object: &Object) -> Result<Vec<LootFunction>, String> {
    list(object, "functions")?
        .iter()
        .map(decode_function)
        .collect()
}

/// `LootItemFunctions.ROOT_CODEC`: a typed function, or a bare list which is an
/// inline `sequence` (`SequenceFunction.INLINE_CODEC`).
pub(super) fn decode_function(value: &Value) -> Result<LootFunction, String> {
    if let Value::Array(functions) = value {
        return functions
            .iter()
            .map(decode_typed_function)
            .collect::<Result<_, _>>()
            .map(LootFunction::Sequence);
    }
    decode_typed_function(value)
}

/// `LootItemFunctions.TYPED_CODEC`. A function's own `conditions`
/// (`LootItemConditionalFunction`) wrap it in a [`LootFunction::Filtered`].
fn decode_typed_function(value: &Value) -> Result<LootFunction, String> {
    let object = as_object(value, "loot function")?;
    let kind = type_name(object, "function")?;
    let conditions = decode_conditions(object)?;
    let function = decode_function_body(&kind, object, value)?;
    Ok(match conditions.len() {
        0 => function,
        _ => LootFunction::Filtered {
            condition: LootCondition::AllOf(conditions),
            function: Box::new(function),
        },
    })
}

/// The function proper, without its `conditions`.
fn decode_function_body(
    kind: &str,
    object: &Object,
    value: &Value,
) -> Result<LootFunction, String> {
    let unmodeled = || -> Result<LootFunction, String> {
        validate_unmodeled_function(kind, object)?;
        Ok(LootFunction::Unmodeled {
            kind: format!("minecraft:{kind}"),
            data: value.clone(),
        })
    };
    match kind {
        "set_count" => {
            let count = decode_number(required(object, "count")?)?;
            Ok(if bool_or(object, "add", false)? {
                LootFunction::AddCount(count)
            } else {
                LootFunction::SetCount(count)
            })
        }
        "limit_count" => match int_range(required(object, "limit")?)? {
            Some((min, max)) => Ok(LootFunction::LimitCount { min, max }),
            None => unmodeled(),
        },
        "explosion_decay" => Ok(LootFunction::ApplyExplosionDecay),
        "set_potion" => Ok(LootFunction::SetPotion(identifier(
            required(object, "id")?,
            "id",
        )?)),
        "apply_bonus" => decode_apply_bonus(object),
        "enchanted_count_increase" => Ok(LootFunction::EnchantedCountIncrease {
            enchantment: identifier(required(object, "enchantment")?, "enchantment")?,
            count: decode_number(required(object, "count")?)?,
            limit: int_field(object, "limit", 0)?,
        }),
        "furnace_smelt" => decode_furnace_smelt(object),
        "set_damage" => decode_set_damage(object),
        "set_enchantments" => decode_set_enchantments(object),
        "enchant_with_levels" => decode_enchant_with_levels(object),
        "enchant_randomly" => decode_enchant_randomly(object),
        "copy_state" => decode_copy_state(object),
        "copy_components" => decode_copy_components(object),
        "set_instrument" => decode_set_instrument(object),
        "set_stew_effect" => decode_set_stew_effect(object),
        "set_ominous_bottle_amplifier" => Ok(LootFunction::SetOminousBottleAmplifier(
            decode_number(required(object, "amplifier")?)?,
        )),
        "set_name" => match decode_set_name(object)? {
            Some(function) => Ok(function),
            None => unmodeled(),
        },
        "set_components" => match decode_set_components(object)? {
            Some(function) => Ok(function),
            None => unmodeled(),
        },
        "discard" => Ok(LootFunction::Discard),
        "reference" => Ok(LootFunction::Reference(identifier(
            required(object, "name")?,
            "name",
        )?)),
        "sequence" => Ok(LootFunction::Sequence(
            as_list(required(object, "functions")?, "functions")?
                .iter()
                .map(decode_typed_function)
                .collect::<Result<_, _>>()?,
        )),
        "set_item" => Ok(LootFunction::SetItem(identifier(
            required(object, "item")?,
            "item",
        )?)),
        _ => unmodeled(),
    }
}

use Kind::{Any, Bool, Function, Id, IdSet, Int, List, ListMode, Long, Number, Obj, Str, Target};

/// The record fields (besides `conditions`) of each function type the runtime does
/// not model, or `None` for an unregistered type.
fn function_schema(kind: &str) -> Option<&'static [Field]> {
    Some(match kind {
        "limit_count" => &[("limit", Kind::Range, true)],
        "set_custom_data" => &[("tag", Any, true)],
        "set_components" => &[("components", Obj, true)],
        "set_attributes" => &[("modifiers", List, true), ("replace", Bool, false)],
        "set_name" => &[
            ("name", Any, false),
            ("entity", Target, false),
            ("target", Str, false),
        ],
        "exploration_map" => &[
            ("destination", Id, false),
            ("decoration", Id, false),
            ("zoom", Int, false),
            ("search_radius", Int, false),
            ("skip_existing_chunks", Bool, false),
        ],
        "copy_name" => &[("source", Str, true)],
        "set_contents" => &[("component", Id, true), ("entries", List, true)],
        "modify_contents" => &[("component", Id, true), ("modifier", Function, true)],
        "filtered" => &[
            ("item_filter", Obj, true),
            ("on_pass", Function, false),
            ("on_fail", Function, false),
        ],
        "set_loot_table" => &[
            ("name", Id, true),
            ("seed", Long, false),
            ("type", Id, true),
        ],
        "set_lore" => &[
            ("lore", List, true),
            ("mode", ListMode, true),
            ("entity", Target, false),
        ],
        "fill_player_head" => &[("entity", Target, true)],
        "copy_custom_data" => &[("source", Any, true), ("ops", List, true)],
        "set_banner_pattern" => &[("patterns", List, true), ("append", Bool, true)],
        "set_random_dyes" => &[("number_of_dyes", Number, true)],
        "set_random_potion" => &[("options", IdSet, false)],
        "set_fireworks" => &[("explosions", Obj, false), ("flight_duration", Int, false)],
        "set_firework_explosion" => &[
            ("shape", Str, false),
            ("colors", List, false),
            ("fade_colors", List, false),
            ("trail", Bool, false),
            ("twinkle", Bool, false),
        ],
        "set_book_cover" => &[
            ("title", Any, false),
            ("author", Str, false),
            ("generation", Int, false),
        ],
        "set_written_book_pages" | "set_writable_book_pages" => {
            &[("pages", List, true), ("mode", ListMode, true)]
        }
        "toggle_tooltips" => &[("toggles", Obj, true)],
        "set_custom_model_data" => &[
            ("floats", Obj, false),
            ("flags", Obj, false),
            ("strings", Obj, false),
            ("colors", Obj, false),
        ],
        _ => return None,
    })
}

/// Validates the record fields of a function the runtime does not model, plus the
/// nested structures the schema table cannot express.
fn validate_unmodeled_function(kind: &str, object: &Object) -> Result<(), String> {
    let schema =
        function_schema(kind).ok_or_else(|| format!("Unknown function type 'minecraft:{kind}'"))?;
    validate_fields(object, schema)?;
    match kind {
        "set_contents" => as_list(required(object, "entries")?, "entries")?
            .iter()
            .try_for_each(|entry| super::decode_entry(entry).map(drop)),
        "set_attributes" => as_list(required(object, "modifiers")?, "modifiers")?
            .iter()
            .try_for_each(validate_attribute_modifier),
        _ => Ok(()),
    }
}

/// `ApplyBonusCount.FORMULA_CODEC`: `formula` picks the parameter record.
fn decode_apply_bonus(object: &Object) -> Result<LootFunction, String> {
    let formula = identifier(required(object, "formula")?, "formula")?;
    let parameters = || {
        object
            .get("parameters")
            .ok_or_else(|| "missing field parameters".to_string())
            .and_then(|value| as_object(value, "parameters"))
    };
    let formula = match formula.as_str() {
        "minecraft:ore_drops" => LootBonusFormula::OreDrops,
        "minecraft:uniform_bonus_count" => LootBonusFormula::UniformBonusCount {
            bonus_multiplier: int_field_required(parameters()?, "bonusMultiplier")?,
        },
        "minecraft:binomial_with_bonus_count" => LootBonusFormula::BinomialWithBonusCount {
            extra: int_field_required(parameters()?, "extra")?,
            probability: as_float(required(parameters()?, "probability")?, "probability")?,
        },
        other => return Err(format!("No formula type with id: '{other}'")),
    };
    Ok(LootFunction::ApplyBonus {
        enchantment: identifier(required(object, "enchantment")?, "enchantment")?,
        formula,
    })
}

/// `Codec.INT.fieldOf(key)`.
fn int_field_required(object: &Object, key: &str) -> Result<i32, String> {
    as_int(required(object, key)?, key)
}

/// `SetAttributesFunction.Modifier.CODEC`.
fn validate_attribute_modifier(modifier: &Value) -> Result<(), String> {
    let modifier = as_object(modifier, "attribute modifier")?;
    validate_fields(
        modifier,
        &[
            ("id", Id, true),
            ("attribute", Id, true),
            ("operation", Str, true),
            ("amount", Number, true),
            ("slot", Any, true),
        ],
    )
}
