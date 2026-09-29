//! Codecs of the loot functions that edit enchantments, durability and components:
//! `set_damage`, `set_enchantments`, `enchant_with_levels`, `enchant_randomly`,
//! `furnace_smelt`, `copy_state`, `copy_components`, `set_instrument`,
//! `set_stew_effect`, `set_name` and `set_components`.

use serde_json::Value;

use super::conditions::decode_holder_set;
use super::fields::{
    as_list, as_object, as_str, bool_or, entity_target, identifier, list, required, Object,
};
use super::numbers::decode_number;
use crate::loot_system::{
    ComponentEdit, ComponentSource, LootFunction, NameTarget, StewEffect,
};
use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;

/// `SetItemDamageFunction.MAP_CODEC`.
pub(super) fn decode_set_damage(object: &Object) -> Result<LootFunction, String> {
    Ok(LootFunction::SetDamage {
        damage: decode_number(required(object, "damage")?)?,
        add: bool_or(object, "add", false)?,
    })
}

/// `SmeltItemFunction.MAP_CODEC`.
pub(super) fn decode_furnace_smelt(object: &Object) -> Result<LootFunction, String> {
    Ok(LootFunction::SmeltItem {
        use_input_count: bool_or(object, "use_input_count", true)?,
    })
}

/// `SetEnchantmentsFunction.MAP_CODEC`: `Codec.unboundedMap(Enchantment.CODEC,
/// NumberProviders.CODEC)`.
pub(super) fn decode_set_enchantments(object: &Object) -> Result<LootFunction, String> {
    let mut enchantments = Vec::new();
    if let Some(map) = object.get("enchantments") {
        for (enchantment, level) in as_object(map, "enchantments")? {
            enchantments.push((
                identifier(&Value::String(enchantment.clone()), "enchantment")?,
                decode_number(level)?,
            ));
        }
    }
    Ok(LootFunction::SetEnchantments {
        enchantments,
        add: bool_or(object, "add", false)?,
    })
}

/// `EnchantWithLevelsFunction.MAP_CODEC`.
pub(super) fn decode_enchant_with_levels(object: &Object) -> Result<LootFunction, String> {
    Ok(LootFunction::EnchantWithLevels {
        levels: decode_number(required(object, "levels")?)?,
        options: object.get("options").map(decode_holder_set).transpose()?,
        include_additional_cost_component: bool_or(
            object,
            "include_additional_cost_component",
            false,
        )?,
    })
}

/// `EnchantRandomlyFunction.MAP_CODEC`.
pub(super) fn decode_enchant_randomly(object: &Object) -> Result<LootFunction, String> {
    Ok(LootFunction::EnchantRandomly {
        options: object.get("options").map(decode_holder_set).transpose()?,
        only_compatible: bool_or(object, "only_compatible", true)?,
        include_additional_cost_component: bool_or(
            object,
            "include_additional_cost_component",
            false,
        )?,
    })
}

/// `SetInstrumentFunction.MAP_CODEC`.
pub(super) fn decode_set_instrument(object: &Object) -> Result<LootFunction, String> {
    Ok(LootFunction::SetInstrument(decode_holder_set(required(
        object, "options",
    )?)?))
}

/// `CopyBlockState.MAP_CODEC`: the property names that do not exist on the block's
/// state definition are dropped (`getStateDefinition().getProperty(name)` is null).
pub(super) fn decode_copy_state(object: &Object) -> Result<LootFunction, String> {
    let block = identifier(required(object, "block")?, "block")?;
    let definition = crate::block_states::block_state_entry(&block)
        .ok_or_else(|| format!("Unknown block '{block}'"))?;
    let mut properties = Vec::new();
    for name in as_list(required(object, "properties")?, "properties")? {
        let name = as_str(name, "properties")?;
        if definition
            .properties
            .iter()
            .any(|property| property.name == name)
        {
            properties.push(name.to_string());
        }
    }
    Ok(LootFunction::CopyState { block, properties })
}

/// Whether `DataComponentType.CODEC` knows `id`.
fn validate_component_type(id: &str) -> Result<(), String> {
    let registry = Identifier::parse("minecraft:data_component_type")?;
    let element = Identifier::parse(id)?;
    let known = BuiltinRegistries::vanilla()
        .map(|registries| registries.contains_element(&registry, &element))
        .unwrap_or(false);
    if known {
        Ok(())
    } else {
        Err(format!("Unknown data component type '{id}'"))
    }
}

/// `DataComponentType.CODEC.listOf().optionalFieldOf(key)`.
fn component_type_list(object: &Object, key: &str) -> Result<Option<Vec<String>>, String> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let mut types = Vec::new();
    for entry in as_list(value, key)? {
        let id = identifier(entry, key)?;
        validate_component_type(&id)?;
        types.push(id);
    }
    Ok(Some(types))
}

/// `CopyComponentsFunction.MAP_CODEC`.
pub(super) fn decode_copy_components(object: &Object) -> Result<LootFunction, String> {
    let name = as_str(required(object, "source")?, "source")?;
    let source = ComponentSource::by_name(name)
        .ok_or_else(|| format!("Unknown context source '{name}'"))?;
    Ok(LootFunction::CopyComponents {
        source,
        include: component_type_list(object, "include")?,
        exclude: component_type_list(object, "exclude")?,
    })
}

/// `SetStewEffectFunction.MAP_CODEC`: effects must be distinct.
pub(super) fn decode_set_stew_effect(object: &Object) -> Result<LootFunction, String> {
    let mut effects: Vec<StewEffect> = Vec::new();
    for entry in list(object, "effects")? {
        let entry = as_object(entry, "effect")?;
        let effect = identifier(required(entry, "type")?, "type")?;
        if effects.iter().any(|seen| seen.effect == effect) {
            return Err(format!("Encountered duplicate mob effect: '{effect}'"));
        }
        effects.push(StewEffect {
            effect,
            duration: decode_number(required(entry, "duration")?)?,
        });
    }
    Ok(LootFunction::SetStewEffects(effects))
}

/// Whether a text component (or any nested part) needs an entity to resolve:
/// `selector`, `score` and `nbt` contents (`ComponentUtils.resolve`).
fn needs_resolution(component: &Value) -> bool {
    match component {
        Value::Object(map) => {
            ["selector", "score", "nbt"]
                .iter()
                .any(|key| map.contains_key(*key))
                || map.values().any(needs_resolution)
        }
        Value::Array(parts) => parts.iter().any(needs_resolution),
        _ => false,
    }
}

/// `SetNameFunction.MAP_CODEC`. The name resolver is the identity unless an entity
/// is named and the component has contents that resolve against it; that case needs
/// a command source and stays unmodeled (`None`).
pub(super) fn decode_set_name(object: &Object) -> Result<Option<LootFunction>, String> {
    let name = match object.get("name") {
        Some(name) => {
            let codec = crate::advancement_condition_schema::ConditionCodec::new(
                &crate::advancement_condition_schema::BuiltinLookup,
            );
            codec.component(name)?;
            Some(name)
        }
        None => None,
    };
    if let Some(entity) = object.get("entity") {
        entity_target(entity, "entity")?;
        if name.is_some_and(needs_resolution) {
            return Ok(None);
        }
    }
    let target = match object.get("target") {
        None => NameTarget::CustomName,
        Some(target) => {
            let target = as_str(target, "target")?;
            NameTarget::by_name(target)
                .ok_or_else(|| format!("Unknown set_name target '{target}'"))?
        }
    };
    Ok(Some(LootFunction::SetName {
        name: name.map(Value::to_string),
        target,
    }))
}

/// `SetComponentsFunction.MAP_CODEC`: `DataComponentPatch.CODEC`. Only components
/// whose value codec is modeled decode; `None` keeps the function unmodeled.
pub(super) fn decode_set_components(object: &Object) -> Result<Option<LootFunction>, String> {
    let mut edits = Vec::new();
    for (key, value) in as_object(required(object, "components")?, "components")? {
        let (removal, id) = match key.strip_prefix('!') {
            Some(id) => (true, id),
            None => (false, key.as_str()),
        };
        let id = identifier(&Value::String(id.to_string()), "component")?;
        validate_component_type(&id)?;
        if removal {
            // `"!id": {}` removes the component.
            if !as_object(value, key)?.is_empty() {
                return Err(format!("removal marker for {id} must be an empty object"));
            }
            edits.push(ComponentEdit {
                component: id,
                value: None,
            });
        } else if id == "minecraft:trim" {
            edits.push(ComponentEdit {
                component: id,
                value: Some(decode_trim(value)?),
            });
        } else {
            return Ok(None);
        }
    }
    Ok(Some(LootFunction::SetComponents(edits)))
}

/// `ArmorTrim.CODEC`: a trim material and pattern (registry holders), kept as the
/// compact JSON of the re-encoded record.
fn decode_trim(value: &Value) -> Result<String, String> {
    let trim = as_object(value, "trim")?;
    let mut encoded = serde_json::Map::new();
    for key in ["material", "pattern"] {
        encoded.insert(
            key.to_string(),
            Value::String(identifier(required(trim, key)?, key)?),
        );
    }
    Ok(Value::Object(encoded).to_string())
}
