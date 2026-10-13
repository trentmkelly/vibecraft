//! Decoding and application for Java's list-editing component loot functions.

use serde_json::{Map, Value};

use super::list_operations::{LootListEdit, LootListOperation};
use super::{LootFunction, LootStack};
use crate::advancement_condition_schema::{BuiltinLookup, ConditionCodec};
use crate::loot_system::LootEntityTarget;

const LORE_COMPONENT: &str = "minecraft:lore";
const WRITTEN_BOOK_CONTENT_COMPONENT: &str = "minecraft:written_book_content";
const WRITABLE_BOOK_CONTENT_COMPONENT: &str = "minecraft:writable_book_content";
const MAX_LORE_LINES: usize = 256;
const MAX_WRITABLE_PAGES: usize = 100;
const MAX_WRITABLE_PAGE_LENGTH: usize = 1024;
const MAX_WRITTEN_PAGE_LENGTH: usize = 32767;

/// Decodes the three Java functions backed by `ListOperation`.
///
/// `Ok(None)` means the input is valid but Java needs an entity resolver that is not
/// available in the current loot runtime. Malformed codec input returns `Err`.
pub(super) fn decode(
    kind: &str,
    object: &Map<String, Value>,
) -> Result<Option<LootFunction>, String> {
    match kind {
        "set_lore" => decode_lore(object),
        "set_written_book_pages" => decode_written_pages(object),
        "set_writable_book_pages" => decode_writable_pages(object),
        _ => Ok(None),
    }
}

fn decode_lore(object: &Map<String, Value>) -> Result<Option<LootFunction>, String> {
    let lines = required_list(object, "lore")?;
    if lines.len() > MAX_LORE_LINES {
        return Err(format!(
            "lore has {} entries; maximum is {MAX_LORE_LINES}",
            lines.len()
        ));
    }
    let codec = ConditionCodec::new(&BuiltinLookup);
    for line in lines {
        validate_component(&codec, line, "lore line")?;
    }
    let operation = LootListOperation::decode(object, MAX_LORE_LINES)?;
    let entity = match object.get("entity") {
        Some(Value::String(name)) => Some(
            LootEntityTarget::by_name(name)
                .ok_or_else(|| format!("Unknown entity target '{name}'"))?,
        ),
        Some(_) => return Err("`entity` must be an entity target string".to_string()),
        None => None,
    };
    if entity.is_some_and(|_| lines.iter().any(needs_resolution)) {
        return Ok(None);
    }
    Ok(Some(LootFunction::EditLore(LootListEdit {
        values: lines.to_vec(),
        operation,
    })))
}

fn decode_written_pages(object: &Map<String, Value>) -> Result<Option<LootFunction>, String> {
    let pages = required_list(object, "pages")?;
    // WrittenBookContent.PAGES_CODEC is listOf, with no list length limit.
    let codec = ConditionCodec::new(&BuiltinLookup);
    let mut supported = true;
    for page in pages {
        // Keep validating later pages even when a previous component is unsupported.
        supported &= validate_filterable_component(&codec, page, "written book page")?;
    }
    let operation = LootListOperation::decode(object, i32::MAX as usize)?;
    if !supported {
        return Ok(None);
    }
    Ok(Some(LootFunction::EditWrittenBookPages(LootListEdit {
        values: pages.to_vec(),
        operation,
    })))
}

fn decode_writable_pages(object: &Map<String, Value>) -> Result<Option<LootFunction>, String> {
    let pages = required_list(object, "pages")?;
    if pages.len() > MAX_WRITABLE_PAGES {
        return Err(format!(
            "pages has {} entries; maximum is {MAX_WRITABLE_PAGES}",
            pages.len()
        ));
    }
    for page in pages {
        validate_filterable_string(page, "writable book page")?;
    }
    let operation = LootListOperation::decode(object, MAX_WRITABLE_PAGES)?;
    Ok(Some(LootFunction::EditWritableBookPages(LootListEdit {
        values: pages.to_vec(),
        operation,
    })))
}

fn required_list<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, String> {
    object
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("`{field}` must be a list"))
}

fn validate_filterable_component(
    codec: &ConditionCodec<'_>,
    value: &Value,
    description: &str,
) -> Result<bool, String> {
    if let Some(map) = value.as_object().filter(|map| map.contains_key("raw")) {
        let raw = map
            .get("raw")
            .ok_or_else(|| format!("{description} filterable value is missing `raw`"))?;
        let raw_supported = validate_component(codec, raw, description)?;
        if let Some(filtered) = map.get("filtered") {
            return Ok(validate_component(codec, filtered, description)? && raw_supported);
        }
        Ok(raw_supported)
    } else {
        validate_component(codec, value, description)
    }
}

fn validate_filterable_string(value: &Value, description: &str) -> Result<(), String> {
    let check = |value: &Value| -> Result<(), String> {
        let text = value
            .as_str()
            .ok_or_else(|| format!("{description} must be a string"))?;
        let length = text.encode_utf16().count();
        if length > MAX_WRITABLE_PAGE_LENGTH {
            return Err(format!(
                "{description} has {length} UTF-16 code units; maximum is {MAX_WRITABLE_PAGE_LENGTH}"
            ));
        }
        Ok(())
    };
    if let Some(map) = value.as_object().filter(|map| map.contains_key("raw")) {
        check(
            map.get("raw")
                .ok_or_else(|| format!("{description} is missing `raw`"))?,
        )?;
        if let Some(filtered) = map.get("filtered") {
            check(filtered)?;
        }
        Ok(())
    } else {
        check(value)
    }
}

fn validate_component(
    codec: &ConditionCodec<'_>,
    value: &Value,
    description: &str,
) -> Result<bool, String> {
    if matches!(value, Value::Array(parts) if parts.is_empty()) {
        return Err(format!(
            "invalid {description}: text component arrays cannot be empty"
        ));
    }
    codec
        .component(value)
        .map_err(|error| format!("invalid {description}: {error}"))?;
    if description == "written book page" {
        // Java's Component codec may canonicalize objects and arrays before
        // `flatRestrictedCodec` counts their encoded JSON. This model can prove the
        // exact boundary only for literal strings, which serialize as JSON strings.
        let Some(text) = value.as_str() else {
            return Ok(false);
        };
        let encoded_len = Value::String(text.to_string())
            .to_string()
            .encode_utf16()
            .count()
            + 5 * text
                .chars()
                .filter(|ch| matches!(*ch, '\u{2028}' | '\u{2029}'))
                .count();
        if encoded_len > MAX_WRITTEN_PAGE_LENGTH {
            return Err(format!(
                "written book page encodes to {encoded_len} UTF-16 code units; maximum is {MAX_WRITTEN_PAGE_LENGTH}"
            ));
        }
    }
    Ok(true)
}

/// `ComponentUtils.resolve` can change selectors, scores and NBT components when an
/// entity target is configured. Such functions remain valid but cannot be evaluated
/// without Java's command source and world/registry context.
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

/// Applies one of the three list-editing component functions to the canonical JSON
/// component values stored by `LootStack`.
pub(super) fn apply(mut stack: LootStack, function: &LootFunction) -> LootStack {
    match function {
        LootFunction::EditLore(edit) => {
            let original = read_array_component(&stack, LORE_COMPONENT);
            let result = edit
                .operation
                .apply(&original, &edit.values, MAX_LORE_LINES);
            write_component(&mut stack, LORE_COMPONENT, Value::Array(result));
        }
        LootFunction::EditWritableBookPages(edit) => {
            let mut content = read_object_component(&stack, WRITABLE_BOOK_CONTENT_COMPONENT);
            let original = content
                .get("pages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let result = edit
                .operation
                .apply(&original, &edit.values, MAX_WRITABLE_PAGES);
            content.insert("pages".to_string(), Value::Array(result));
            write_component(
                &mut stack,
                WRITABLE_BOOK_CONTENT_COMPONENT,
                Value::Object(content),
            );
        }
        LootFunction::EditWrittenBookPages(edit) => {
            let mut content = read_object_component(&stack, WRITTEN_BOOK_CONTENT_COMPONENT);
            let original = content
                .get("pages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let result = edit
                .operation
                .apply(&original, &edit.values, i32::MAX as usize);
            // WrittenBookContent.withReplacedPages preserves title/author/generation
            // and always clears `resolved`, even when the selected pages compare equal.
            content.insert("pages".to_string(), Value::Array(result));
            content.insert("resolved".to_string(), Value::Bool(false));
            write_component(
                &mut stack,
                WRITTEN_BOOK_CONTENT_COMPONENT,
                Value::Object(content),
            );
        }
        _ => {}
    }
    stack
}

fn read_array_component(stack: &LootStack, key: &str) -> Vec<Value> {
    stack
        .components
        .get(key)
        .and_then(|text| serde_json::from_str::<Value>(text).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
}

fn read_object_component(stack: &LootStack, key: &str) -> Map<String, Value> {
    let mut content = stack
        .components
        .get(key)
        .and_then(|text| serde_json::from_str::<Value>(text).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    if key == WRITTEN_BOOK_CONTENT_COMPONENT {
        content
            .entry("title".to_string())
            .or_insert_with(|| Value::String(String::new()));
        content
            .entry("author".to_string())
            .or_insert_with(|| Value::String(String::new()));
        content
            .entry("generation".to_string())
            .or_insert_with(|| Value::Number(0.into()));
        content
            .entry("resolved".to_string())
            .or_insert(Value::Bool(true));
    }
    content
}

fn write_component(stack: &mut LootStack, key: &str, value: Value) {
    stack.components.insert(key.to_string(), value.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loot_system::{decode_loot_table, LootContext, LootParamSet};

    #[test]
    fn decodes_components_and_retains_filterable_page_payloads() {
        let lore = serde_json::json!({"mode":"append", "lore":["line"]});
        assert!(matches!(
            decode("set_lore", lore.as_object().unwrap()),
            Ok(Some(LootFunction::EditLore(_)))
        ));

        let pages = serde_json::json!({
            "mode":"replace_all",
            "pages":[{"raw":"raw text", "filtered":"filtered text"}]
        });
        let Some(LootFunction::EditWritableBookPages(edit)) =
            decode("set_writable_book_pages", pages.as_object().unwrap()).unwrap()
        else {
            panic!("writable page edit should decode")
        };
        assert_eq!(
            edit.values,
            [serde_json::json!({"raw":"raw text", "filtered":"filtered text"})]
        );
    }

    #[test]
    fn rejects_bad_page_codecs_and_marks_dynamic_lore_unmodeled() {
        let bad_writable = serde_json::json!({"mode":"append", "pages":[{"raw":12}]});
        assert!(decode("set_writable_book_pages", bad_writable.as_object().unwrap()).is_err());
        let empty_component = serde_json::json!({"mode":"append", "lore":[[]]});
        assert!(decode("set_lore", empty_component.as_object().unwrap()).is_err());

        let dynamic_lore = serde_json::json!({
            "mode":"append", "entity":"this", "lore":[{"selector":"@s"}]
        });
        assert!(decode("set_lore", dynamic_lore.as_object().unwrap())
            .unwrap()
            .is_none());
    }

    #[test]
    fn writable_page_length_uses_java_utf16_code_units() {
        let at_limit = "😀".repeat(MAX_WRITABLE_PAGE_LENGTH / 2);
        let over_limit = format!("{at_limit}😀");
        let valid = serde_json::json!({"mode":"append", "pages":[at_limit]});
        let invalid = serde_json::json!({"mode":"append", "pages":[over_limit]});
        assert!(decode("set_writable_book_pages", valid.as_object().unwrap()).is_ok());
        assert!(decode("set_writable_book_pages", invalid.as_object().unwrap()).is_err());
    }

    #[test]
    fn written_plain_text_page_checks_flat_restricted_encoded_length() {
        let oversized = "x".repeat(MAX_WRITTEN_PAGE_LENGTH + 1);
        let pages = serde_json::json!({"mode":"append", "pages":[oversized]});
        assert!(decode("set_written_book_pages", pages.as_object().unwrap()).is_err());

        // The quotes are part of the JSON size counted by Java's GsonHelper.
        let boundary = "x".repeat(MAX_WRITTEN_PAGE_LENGTH - 2);
        let at_limit = serde_json::json!({"mode":"append", "pages":[boundary]});
        assert!(decode("set_written_book_pages", at_limit.as_object().unwrap()).is_ok());
        let over_boundary = "x".repeat(MAX_WRITTEN_PAGE_LENGTH - 1);
        let over = serde_json::json!({"mode":"append", "pages":[over_boundary]});
        assert!(decode("set_written_book_pages", over.as_object().unwrap()).is_err());

        let separator = "\u{2028}".repeat((MAX_WRITTEN_PAGE_LENGTH - 2) / 6 + 1);
        let escaped = serde_json::json!({"mode":"append", "pages":[separator]});
        assert!(decode("set_written_book_pages", escaped.as_object().unwrap()).is_err());

        // This valid component remains unmodeled because the runtime cannot prove
        // ComponentSerialization's canonical encoded-size check for object values.
        let compact_literal = serde_json::json!({
            "mode":"append", "pages":[{"text":"short"}]
        });
        assert!(decode(
            "set_written_book_pages",
            compact_literal.as_object().unwrap()
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn unsupported_written_components_still_validate_later_pages_and_mode() {
        let bad_page = serde_json::json!({"mode":"append", "pages":[{"text":"first"}, 12]});
        assert!(decode("set_written_book_pages", bad_page.as_object().unwrap()).is_err());
        let bad_mode = serde_json::json!({"mode":"unknown", "pages":[{"text":"first"}]});
        assert!(decode("set_written_book_pages", bad_mode.as_object().unwrap()).is_err());
    }

    #[test]
    fn written_book_edit_preserves_metadata_and_clears_resolution() {
        let mut stack = LootStack::new("minecraft:written_book", 1);
        stack.components.insert(
            WRITTEN_BOOK_CONTENT_COMPONENT.to_string(),
            serde_json::json!({
                "title":"Title", "author":"Author", "generation":2,
                "pages":["old"], "resolved":true
            })
            .to_string(),
        );
        let function = LootFunction::EditWrittenBookPages(LootListEdit {
            values: vec![serde_json::json!({"raw":"new", "filtered":"masked"})],
            operation: LootListOperation::Append,
        });
        let result = apply(stack, &function);
        let content: Value =
            serde_json::from_str(&result.components[WRITTEN_BOOK_CONTENT_COMPONENT]).unwrap();
        assert_eq!(content["title"], "Title");
        assert_eq!(content["author"], "Author");
        assert_eq!(content["generation"], 2);
        assert_eq!(content["resolved"], false);
        assert_eq!(
            content["pages"],
            serde_json::json!(["old", {"raw":"new", "filtered":"masked"}])
        );
    }

    #[test]
    fn decoded_list_functions_are_fully_modeled_and_evaluate_to_vanilla_components() {
        let table = decode_loot_table(
            r#"{
                "pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:book",
                    "functions":[
                        {"function":"minecraft:set_lore","mode":"append","lore":["Line"]},
                        {"function":"minecraft:set_writable_book_pages","mode":"replace_all","pages":[{"raw":"Draft","filtered":"Filtered"}]},
                        {"function":"minecraft:set_written_book_pages","mode":"insert","offset":0,"pages":[{"raw":"Page","filtered":"Masked"}]}
                    ]
                }]}]
            }"#,
        )
        .expect("Java-shaped table decodes");
        assert!(table.is_fully_modeled());
        let mut context = LootContext::new(LootParamSet::AllParams, 1);
        let stacks = table.evaluate(&mut context);
        assert_eq!(stacks.len(), 1);
        let stack = &stacks[0];
        let lore: Value = serde_json::from_str(&stack.components[LORE_COMPONENT]).unwrap();
        assert_eq!(lore, serde_json::json!(["Line"]));
        let writable: Value =
            serde_json::from_str(&stack.components[WRITABLE_BOOK_CONTENT_COMPONENT]).unwrap();
        assert_eq!(
            writable["pages"],
            serde_json::json!([{"raw":"Draft", "filtered":"Filtered"}])
        );
        let written: Value =
            serde_json::from_str(&stack.components[WRITTEN_BOOK_CONTENT_COMPONENT]).unwrap();
        assert_eq!(
            written["pages"],
            serde_json::json!([{"raw":"Page", "filtered":"Masked"}])
        );
        assert_eq!(written["resolved"], false);
    }
}
