//! `NumberFormat` model strings and their `NumberFormatTypes.CODEC` NBT form.
//!
//! The command model keeps a score/objective number format as text: `blank`, `fixed:<text>`
//! or `styled:<snbt style compound>` (`ScoreboardCommand`'s `numberformat` argument). Java stores
//! them as `{type:"minecraft:blank"}`, `{type:"minecraft:fixed", value:<component>}` and
//! `{type:"minecraft:styled", <style fields>}`.

use crate::storage::nbt::{parse_snbt, Tag};

/// A model number-format string split into its Java `NumberFormatType`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedNumberFormat {
    Blank,
    Fixed(String),
    Styled(String),
}

/// Splits a model string; `None` for text that is none of the three formats.
pub(super) fn parse_number_format(format: &str) -> Option<ParsedNumberFormat> {
    if format == "blank" {
        Some(ParsedNumberFormat::Blank)
    } else if let Some(text) = format.strip_prefix("fixed:") {
        Some(ParsedNumberFormat::Fixed(text.to_string()))
    } else {
        format
            .strip_prefix("styled:")
            .map(|style| ParsedNumberFormat::Styled(style.to_string()))
    }
}

/// `NumberFormatTypes.CODEC` encoding of a model number format.
pub(super) fn number_format_to_tag(format: &str) -> Option<Tag> {
    let type_field = |name: &str| ("type".to_string(), Tag::String(format!("minecraft:{name}")));
    Some(match parse_number_format(format)? {
        ParsedNumberFormat::Blank => Tag::Compound(vec![type_field("blank")]),
        ParsedNumberFormat::Fixed(text) => Tag::Compound(vec![
            type_field("fixed"),
            ("value".to_string(), Tag::String(text)),
        ]),
        ParsedNumberFormat::Styled(style) => {
            let mut fields = vec![type_field("styled")];
            if let Ok(Tag::Compound(style_fields)) = parse_snbt(&style) {
                fields.extend(style_fields);
            }
            Tag::Compound(fields)
        }
    })
}

/// Decodes `NumberFormatTypes.CODEC` back to the model string; `None` for an unknown type.
pub(super) fn number_format_from_tag(tag: &Tag) -> Option<String> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    let field = |name: &str| fields.iter().find(|(key, _)| key == name).map(|(_, tag)| tag);
    let kind = match field("type")? {
        Tag::String(kind) => kind.strip_prefix("minecraft:").unwrap_or(kind),
        _ => return None,
    };
    match kind {
        "blank" => Some("blank".to_string()),
        "fixed" => Some(format!("fixed:{}", plain_text(field("value")?))),
        "styled" => {
            let style: Vec<(String, Tag)> = fields
                .iter()
                .filter(|(key, _)| key != "type")
                .cloned()
                .collect();
            Some(format!("styled:{}", Tag::Compound(style).to_snbt()))
        }
        _ => None,
    }
}

/// Plain text of a component tag (`"text"` or `{text:"..."}`), the only shapes the plain-text
/// scoreboard model can represent.
// TODO(scoreboard-rich-components): the command model keeps display names, prefixes, suffixes
// and fixed number text as plain strings, so styled/translatable components lose their styling
// when loaded from disk.
pub(super) fn plain_text(tag: &Tag) -> String {
    match tag {
        Tag::String(text) => text.clone(),
        Tag::Compound(fields) => fields
            .iter()
            .find_map(|(key, value)| match (key.as_str(), value) {
                ("text", Tag::String(text)) => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default(),
        _ => String::new(),
    }
}
