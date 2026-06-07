#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierExceptionModel {
    message: String,
    cause: Option<String>,
}

impl IdentifierExceptionModel {
    pub fn new(message: &str) -> Self {
        Self {
            message: escape_java(message),
            cause: None,
        }
    }

    pub fn with_cause(message: &str, cause: impl Into<String>) -> Self {
        Self {
            message: escape_java(message),
            cause: Some(cause.into()),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn cause(&self) -> Option<&str> {
        self.cause.as_deref()
    }
}

impl fmt::Display for IdentifierExceptionModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

pub fn escape_java(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            '"' => escaped.push_str("\\\""),
            '\'' => escaped.push('\''),
            '\\' => escaped.push_str("\\\\"),
            character if character < ' ' || character as u32 > 0x7e => {
                append_java_unicode_escape(&mut escaped, character);
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn append_java_unicode_escape(output: &mut String, character: char) {
    let mut buffer = [0_u16; 2];
    for code_unit in character.encode_utf16(&mut buffer) {
        output.push_str("\\u");
        output.push_str(&format!("{code_unit:04X}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_java, IdentifierExceptionModel};

    #[test]
    fn identifier_exception_escapes_message_like_apache_escape_java() {
        assert_eq!(
            IdentifierExceptionModel::new("bad\nid\t\"x\"\\path").message(),
            "bad\\nid\\t\\\"x\\\"\\\\path"
        );
        assert_eq!(
            IdentifierExceptionModel::new("snowman ☃").message(),
            "snowman \\u2603"
        );
        assert_eq!(
            IdentifierExceptionModel::new("emoji 😀").message(),
            "emoji \\uD83D\\uDE00"
        );
    }

    #[test]
    fn identifier_exception_preserves_cause_and_display_message() {
        let exception = IdentifierExceptionModel::with_cause("bad\rid", "root cause");

        assert_eq!(exception.message(), "bad\\rid");
        assert_eq!(exception.cause(), Some("root cause"));
        assert_eq!(exception.to_string(), "bad\\rid");
    }

    #[test]
    fn escape_java_matches_java_character_special_cases() {
        assert_eq!(escape_java("a\u{08}b\u{0C}c"), "a\\bb\\fc");
        assert_eq!(
            escape_java("'single quotes stay raw'"),
            "'single quotes stay raw'"
        );
        assert_eq!(escape_java("\u{0001}\u{007F}"), "\\u0001\\u007F");
    }
}
