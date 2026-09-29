//! Port of `net.minecraft.util.datafix.LegacyComponentDataFixUtils`.

use crate::storage::nbt::Tag;

use super::json::{parse_lenient, to_stable_string, Json};

/// `LegacyComponentDataFixUtils.createTextComponentJson`: `{"text":"<text>"}`.
pub fn create_text_component_json(text: &str) -> String {
    to_stable_string(&Json::Object(vec![(
        "text".to_string(),
        Json::String(text.to_string()),
    )]))
}

/// `LegacyComponentDataFixUtils.createTranslatableComponentJson`.
pub fn create_translatable_component_json(key: &str) -> String {
    to_stable_string(&Json::Object(vec![(
        "translate".to_string(),
        Json::String(key.to_string()),
    )]))
}

/// `createPlainTextComponent` for NBT.
pub fn create_plain_text_component(text: &str) -> Tag {
    Tag::String(create_text_component_json(text))
}

/// `createEmptyComponent` for NBT (`{"text":""}`).
pub fn create_empty_component() -> Tag {
    Tag::String(create_text_component_json(""))
}

/// `createTranslatableComponent` for NBT.
pub fn create_translatable_component(key: &str) -> Tag {
    Tag::String(create_translatable_component_json(key))
}

/// `LegacyComponentDataFixUtils.rewriteFromLenient`: turns legacy, possibly
/// non-JSON text into a strict JSON text component.
pub fn rewrite_from_lenient(string: &str) -> String {
    if string.is_empty() || string == "null" {
        return create_text_component_json("");
    }
    let first = string.chars().next();
    let last = string.chars().last();
    let looks_like_json = matches!(
        (first, last),
        (Some('"'), Some('"')) | (Some('{'), Some('}')) | (Some('['), Some(']'))
    );
    if looks_like_json {
        if let Ok(json) = parse_lenient(string) {
            return match json.primitive_as_string() {
                Some(text) => create_text_component_json(&text),
                None => to_stable_string(&json),
            };
        }
    }
    create_text_component_json(string)
}

/// `LegacyComponentDataFixUtils.extractTranslationString`.
pub fn extract_translation_string(component: &str) -> Option<String> {
    match parse_lenient(component).ok()? {
        Json::Object(members) => members.into_iter().find_map(|(name, value)| {
            if name == "translate" {
                value.primitive_as_string()
            } else {
                None
            }
        }),
        _ => None,
    }
}

/// `LegacyComponentDataFixUtils.isStrictlyValidJson` for a string tag: parses
/// with the strict parser (`StrictJsonParser`).
pub fn is_strictly_valid_json(component: &Tag) -> bool {
    match component {
        Tag::String(text) => serde_json::from_str::<serde_json::Value>(text).is_ok(),
        _ => false,
    }
}
