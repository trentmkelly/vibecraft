//! Strict JSON parsing matching Minecraft's `StrictJsonParser`.

#![allow(dead_code)]

use std::io::Read;

pub struct StrictJsonParser;

impl StrictJsonParser {
    pub fn parse_reader(mut reader: impl Read) -> Result<serde_json::Value, String> {
        let mut input = String::new();
        reader
            .read_to_string(&mut input)
            .map_err(|error| error.to_string())?;
        Self::parse(&input)
    }

    pub fn parse(json: &str) -> Result<serde_json::Value, String> {
        let mut values = serde_json::Deserializer::from_str(json).into_iter::<serde_json::Value>();
        let value = values
            .next()
            .ok_or_else(|| "expected a JSON value".to_string())?
            .map_err(|error| error.to_string())?;

        // Vanilla checks for trailing input except when the parsed element is
        // JsonNull; preserve that exact Gson behavior.
        if !value.is_null() && values.next().is_some() {
            return Err("Did not consume the entire document.".to_string());
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::StrictJsonParser;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn strict_json_parser_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/StrictJsonParser.java");
        assert_eq!(JAVA.lines().count(), 36);
        for fragment in [
            "public static JsonElement parse(final Reader reader)",
            "jsonReader.setStrictness(Strictness.STRICT)",
            "if (!element.isJsonNull() && jsonReader.peek() != JsonToken.END_DOCUMENT)",
            "Did not consume the entire document.",
            "public static JsonElement parse(final String json)",
        ] {
            assert!(JAVA.contains(fragment), "missing StrictJsonParser source fragment: {fragment}");
        }
    }

    #[test]
    fn strict_json_parser_rejects_trailing_non_null_data_and_reads_streams() {
        assert_eq!(StrictJsonParser::parse("{\"value\":1}").ok(), Some(serde_json::json!({"value": 1})));
        assert_eq!(
            StrictJsonParser::parse_reader(Cursor::new("[1,2,3]")),
            Ok(serde_json::json!([1, 2, 3]))
        );
        assert!(StrictJsonParser::parse("{} {} ").is_err());
        assert!(StrictJsonParser::parse("{").is_err());
        // Gson's implementation intentionally skips the trailing-document
        // check when the first parsed element is JsonNull.
        assert_eq!(StrictJsonParser::parse("null {}"), Ok(serde_json::Value::Null));
    }
}
