//! `SetCustomModelDataFunction` decoding and runtime behavior.
//!
//! The component is represented as canonical JSON in [`LootStack::components`].
//! Its four fields are independent lists, and every present field is transformed by
//! its `ListOperation` while absent fields retain their current values.

use serde_json::{Map, Value};

use super::list_operations::LootListEdit;
use super::{LootContext, LootFunction, LootStack, NumberProvider};

const COMPONENT: &str = "minecraft:custom_model_data";
const MAX_LIST_SIZE: usize = i32::MAX as usize;

/// The four optional list edits supported by `SetCustomModelDataFunction`.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomModelDataEdit {
    pub floats: Option<LootListEdit<NumberProvider>>,
    pub flags: Option<LootListEdit<bool>>,
    pub strings: Option<LootListEdit<String>>,
    pub colors: Option<LootListEdit<NumberProvider>>,
}

/// `SetCustomModelDataFunction.MAP_CODEC`.
pub(super) fn decode(object: &Map<String, Value>) -> Result<LootFunction, String> {
    Ok(LootFunction::EditCustomModelData(CustomModelDataEdit {
        floats: decode_edit(object, "floats", |value| {
            super::json_codec::decode_number_provider(value)
        })?,
        flags: decode_edit(object, "flags", |value| decode_bool(value, "flags"))?,
        strings: decode_edit(object, "strings", |value| decode_string(value, "strings"))?,
        colors: decode_edit(object, "colors", decode_color_provider)?,
    }))
}

fn decode_edit<T>(
    object: &Map<String, Value>,
    field: &str,
    decode_value: impl Fn(&Value) -> Result<T, String>,
) -> Result<Option<LootListEdit<T>>, String> {
    object
        .get(field)
        .map(|value| {
            let edit = value
                .as_object()
                .ok_or_else(|| format!("{field} is not an object: {value}"))?;
            let values = edit
                .get("values")
                .and_then(Value::as_array)
                .ok_or_else(|| format!("{field}.values is not a list"))?
                .as_slice();
            if values.len() > MAX_LIST_SIZE {
                return Err(format!(
                    "{field}.values has {} entries; maximum is {MAX_LIST_SIZE}",
                    values.len()
                ));
            }
            let values = values
                .iter()
                .map(decode_value)
                .collect::<Result<Vec<_>, _>>()?;
            let operation = super::list_operations::LootListOperation::decode(edit, MAX_LIST_SIZE)?;
            Ok(LootListEdit { values, operation })
        })
        .transpose()
}

fn decode_bool(value: &Value, field: &str) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| format!("{field} is not a boolean: {value}"))
}

fn decode_string(value: &Value, field: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{field} is not a string: {value}"))
}

/// `Codec.withAlternative(NumberProviders.CODEC, RGB_COLOR_CODEC, ConstantValue::new)`.
/// `RGB_COLOR_CODEC` first accepts an integer, then a normalized `[r, g, b]` vector.
fn decode_color_provider(value: &Value) -> Result<NumberProvider, String> {
    if let Ok(provider) = super::json_codec::decode_number_provider(value) {
        return Ok(provider);
    }

    let Value::Array(channels) = value else {
        return Err(format!("invalid color provider: {value}"));
    };
    if channels.len() != 3 {
        return Err(format!("color is not a three-float vector: {value}"));
    }
    let mut rgb = [0u8; 3];
    for (index, channel) in channels.iter().enumerate() {
        let value = channel
            .as_f64()
            .ok_or_else(|| format!("color channel is not a number: {channel}"))?
            as f32;
        if !value.is_finite() {
            return Err(format!("color channel is not finite: {channel}"));
        }
        // ARGB.colorFromFloat(1.0F, r, g, b) -> floor(channel * 255),
        // packed in an opaque ARGB integer. Java's packed color allows channel
        // values outside [0, 1] and masks their low eight bits.
        rgb[index] = ((value * 255.0).floor() as i32 & 0xff) as u8;
    }
    let color = i32::from_be_bytes([0xff, rgb[0], rgb[1], rgb[2]]);
    Ok(NumberProvider::Constant(color as f32))
}

/// `SetCustomModelDataFunction.run`.
pub(super) fn apply(
    mut stack: LootStack,
    edit: &CustomModelDataEdit,
    context: &mut LootContext,
) -> LootStack {
    let current = stack
        .components
        .get(COMPONENT)
        .and_then(|text| serde_json::from_str::<Value>(text).ok())
        .unwrap_or_else(|| Value::Object(Map::new()));

    let current = current.as_object().cloned().unwrap_or_default();
    let old_floats = float_list(&current, "floats");
    let old_flags = bool_list(&current, "flags");
    let old_strings = string_list(&current, "strings");
    let old_colors = int_list(&current, "colors");
    let mut floats = old_floats.clone();
    let mut flags = old_flags.clone();
    let mut strings = old_strings.clone();
    let mut colors = old_colors.clone();

    if let Some(operation) = &edit.floats {
        let replacement = operation
            .values
            .iter()
            .map(|provider| provider.float(context))
            .collect::<Vec<_>>();
        floats = operation
            .operation
            .apply(&old_floats, &replacement, MAX_LIST_SIZE);
    }
    if let Some(operation) = &edit.flags {
        flags = operation
            .operation
            .apply(&old_flags, &operation.values, MAX_LIST_SIZE);
    }
    if let Some(operation) = &edit.strings {
        strings = operation
            .operation
            .apply(&old_strings, &operation.values, MAX_LIST_SIZE);
    }
    if let Some(operation) = &edit.colors {
        // Java maps all replacement providers before applying the list operation;
        // preserve provider order and random draw order here.
        let replacement = operation
            .values
            .iter()
            .map(|provider| provider.int(context))
            .collect::<Vec<_>>();
        colors = operation
            .operation
            .apply(&old_colors, &replacement, MAX_LIST_SIZE);
    }

    // `CustomModelData.CODEC` encodes each empty list as its optional-field default,
    // so that field is omitted in canonical component JSON.
    let mut result = Map::new();
    insert_non_empty(&mut result, "floats", serde_json::json!(floats));
    insert_non_empty(&mut result, "flags", serde_json::json!(flags));
    insert_non_empty(&mut result, "strings", serde_json::json!(strings));
    insert_non_empty(&mut result, "colors", serde_json::json!(colors));
    stack
        .components
        .insert(COMPONENT.to_string(), Value::Object(result).to_string());
    stack
}

fn insert_non_empty(object: &mut Map<String, Value>, field: &str, values: Value) {
    if values.as_array().is_some_and(|values| !values.is_empty()) {
        object.insert(field.to_string(), values);
    }
}

fn float_list(object: &Map<String, Value>, field: &str) -> Vec<f32> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_f64)
        .map(|value| value as f32)
        .collect()
}

fn bool_list(object: &Map<String, Value>, field: &str) -> Vec<bool> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_bool)
        .collect()
}

fn string_list(object: &Map<String, Value>, field: &str) -> Vec<String> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn int_list(object: &Map<String, Value>, field: &str) -> Vec<i32> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_i64)
        .filter_map(|value| i32::try_from(value).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loot_system::{decode_loot_table, LootContext, LootParamSet, LootStack};

    fn context(seed: u64) -> LootContext {
        LootContext::new(LootParamSet::AllParams, seed)
    }

    fn component(stack: &LootStack) -> Value {
        serde_json::from_str(&stack.components[COMPONENT]).expect("valid custom model data")
    }

    #[test]
    fn list_operations_replace_insert_append_and_replace_section() {
        let mut stack = LootStack::new("minecraft:stone", 1);
        stack.components.insert(
            COMPONENT.to_string(),
            r#"{"floats":[1.0,2.0,3.0],"flags":[false],"strings":["old"],"colors":[1,2,3]}"#
                .to_string(),
        );
        let raw = serde_json::json!({
            "floats":{"values":[7.0],"mode":"replace_section","offset":1,"size":1},
            "flags":{"values":[true],"mode":"append"},
            "strings":{"values":["new"],"mode":"insert","offset":1},
            "colors":{"values":[9],"mode":"replace_all"}
        });
        let function = decode(raw.as_object().unwrap()).unwrap();
        let LootFunction::EditCustomModelData(edit) = function else {
            panic!("wrong decoded custom-model-data function variant");
        };
        let out = apply(stack, &edit, &mut context(3));
        let data = component(&out);
        assert_eq!(data["floats"], serde_json::json!([1.0, 7.0, 3.0]));
        assert_eq!(data["flags"], serde_json::json!([false, true]));
        assert_eq!(data["strings"], serde_json::json!(["old", "new"]));
        assert_eq!(data["colors"], serde_json::json!([9]));
    }

    #[test]
    fn color_vectors_are_opaque_floor_quantized_rgb_values() {
        let provider = decode_color_provider(&serde_json::json!([0.5, 1.0, 0.0])).unwrap();
        assert_eq!(
            provider,
            NumberProvider::Constant(0xff7fff00u32 as i32 as f32)
        );
        assert!(decode_color_provider(&serde_json::json!([0.5, 0.5])).is_err());
        assert!(decode_color_provider(&serde_json::json!([0.5, "green", 0.5])).is_err());
    }

    #[test]
    fn decoded_table_applies_function_and_preserves_rng_order() {
        let table = decode_loot_table(
            r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone","functions":[{"function":"minecraft:set_custom_model_data","floats":{"values":[{"type":"minecraft:uniform","min":1,"max":2},{"type":"minecraft:uniform","min":3,"max":4}],"mode":"append"},"colors":{"values":[{"type":"minecraft:uniform","min":16711680,"max":16711681}],"mode":"append"}}]}]}]}"#,
        )
        .expect("function decodes");
        assert!(table.is_fully_modeled());

        let mut actual_context = context(1234);
        let actual = table
            .evaluate(&mut actual_context)
            .into_iter()
            .next()
            .expect("item entry emits one stack");
        let actual_next = actual_context.random.next_i32(1_000_000);

        let mut expected_context = context(1234);
        let first = NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(1.0)),
            max: Box::new(NumberProvider::Constant(2.0)),
        }
        .float(&mut expected_context);
        let second = NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(3.0)),
            max: Box::new(NumberProvider::Constant(4.0)),
        }
        .float(&mut expected_context);
        let color = NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(16_711_680.0)),
            max: Box::new(NumberProvider::Constant(16_711_681.0)),
        }
        .int(&mut expected_context);
        let expected_next = expected_context.random.next_i32(1_000_000);
        let data = component(&actual);
        assert_eq!(data["floats"], serde_json::json!([first, second]));
        assert_eq!(data["colors"], serde_json::json!([color]));
        assert_eq!(actual_next, expected_next);
    }

    #[test]
    fn rejected_offset_still_evaluates_all_replacement_providers_in_field_order() {
        let function = decode(
            serde_json::json!({
                "floats":{"values":[{"type":"minecraft:uniform","min":1,"max":2}],"mode":"insert","offset":9},
                "colors":{"values":[{"type":"minecraft:uniform","min":16711680,"max":16711681}],"mode":"append"}
            })
            .as_object()
            .unwrap(),
        )
        .unwrap();
        let mut actual_context = context(993);
        let mut stack = LootStack::new("minecraft:stone", 1);
        stack.components.insert(
            COMPONENT.to_string(),
            r#"{"floats":[5.0],"flags":[],"strings":[],"colors":[0]}"#.to_string(),
        );
        let actual = function.apply(stack, &mut actual_context).unwrap();
        let actual_next = actual_context.random.next_i32(1_000_000);

        let mut expected_context = context(993);
        let _ = NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(1.0)),
            max: Box::new(NumberProvider::Constant(2.0)),
        }
        .float(&mut expected_context);
        let color = NumberProvider::UniformProvider {
            min: Box::new(NumberProvider::Constant(16_711_680.0)),
            max: Box::new(NumberProvider::Constant(16_711_681.0)),
        }
        .int(&mut expected_context);
        let expected_next = expected_context.random.next_i32(1_000_000);
        assert_eq!(actual_next, expected_next);
        let data = component(&actual);
        assert_eq!(data["floats"], serde_json::json!([5.0]));
        assert_eq!(data["colors"], serde_json::json!([0, color]));
    }

    #[test]
    fn decoded_color_constants_use_java_rounding_for_negative_halves() {
        let table = decode_loot_table(r#"{"functions":[{"function":"minecraft:set_custom_model_data","colors":{"values":[-0.5,-1.5,8388609],"mode":"replace_all"}}],"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:stone"}]}]}"#).unwrap();
        assert!(table.is_fully_modeled());
        let output = table.evaluate(&mut context(1));
        assert_eq!(
            component(&output[0])["colors"],
            serde_json::json!([0, -1, 8388609])
        );
    }

    #[test]
    fn absent_fields_are_preserved_and_signed_integer_color_providers_stay_signed() {
        let function = decode(
            serde_json::json!({
                "floats":{"values":[2.5],"mode":"replace_all"},
                "colors":{"values":[-1],"mode":"append"}
            })
            .as_object()
            .unwrap(),
        )
        .unwrap();
        let mut stack = LootStack::new("minecraft:stone", 1);
        stack.components.insert(
            COMPONENT.to_string(),
            r#"{"floats":[1.0],"flags":[true],"strings":["keep"],"colors":[4]}"#.to_string(),
        );
        let out = function.apply(stack, &mut context(8)).unwrap();
        let data = component(&out);
        assert_eq!(data["floats"], serde_json::json!([2.5]));
        assert_eq!(data["flags"], serde_json::json!([true]));
        assert_eq!(data["strings"], serde_json::json!(["keep"]));
        assert_eq!(data["colors"], serde_json::json!([4, -1]));
    }
}
