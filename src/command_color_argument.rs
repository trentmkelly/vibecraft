use std::collections::HashMap;

use crate::chat_formatting::ChatFormatting;
use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorArgumentModel;

impl ColorArgumentModel {
    pub fn color() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<ChatFormatting, ColorParseError> {
        let id = reader.read_unquoted_string();
        match ChatFormatting::get_by_name(Some(&id)) {
            Some(result) if !result.is_format() => Ok(result),
            _ => Err(ColorParseError::InvalidValue { value: id }),
        }
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(ChatFormatting::get_names(true, false), builder);
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["red", "green"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, ChatFormatting>,
}

impl CommandContextModel {
    pub fn with_color(mut self, name: impl Into<String>, value: ChatFormatting) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_color(context: &CommandContextModel, name: &str) -> Option<ChatFormatting> {
    context.arguments.get(name).copied()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && !self.peek().is_ascii_whitespace() {
            self.cursor += 1;
        }
        self.input[start..self.cursor].to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(ChatFormatting, usize), ColorParseError> {
        let mut reader = StringReaderModel::new(input);
        let color = ColorArgumentModel::color().parse(&mut reader)?;
        Ok((color, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(ColorArgumentModel::color().examples(), ["red", "green"]);
    }

    #[test]
    fn get_color_returns_typed_context_argument() {
        let context = CommandContextModel::default().with_color("shade", ChatFormatting::DarkRed);

        assert_eq!(get_color(&context, "shade"), Some(ChatFormatting::DarkRed));
        assert_eq!(get_color(&context, "missing"), None);
    }

    #[test]
    fn parse_reads_one_unquoted_token_and_accepts_non_format_values() {
        let (color, cursor) = parse("red rest").unwrap();

        assert_eq!(color, ChatFormatting::Red);
        assert_eq!(cursor, 3);
    }

    #[test]
    fn parse_uses_chat_formatting_cleaned_name_lookup() {
        let (color, cursor) = parse("dark-blue next").unwrap();

        assert_eq!(color, ChatFormatting::DarkBlue);
        assert_eq!(cursor, 9);
    }

    #[test]
    fn parse_rejects_formatting_codes_and_unknown_or_empty_values() {
        assert_eq!(
            parse("bold"),
            Err(ColorParseError::InvalidValue {
                value: "bold".to_string(),
            })
        );
        assert_eq!(
            parse("missing"),
            Err(ColorParseError::InvalidValue {
                value: "missing".to_string(),
            })
        );
        assert_eq!(
            parse(""),
            Err(ColorParseError::InvalidValue {
                value: String::new(),
            })
        );
    }

    #[test]
    fn reset_is_accepted_because_java_rejects_formats_not_non_colors() {
        let (color, cursor) = parse("reset tail").unwrap();

        assert_eq!(color, ChatFormatting::Reset);
        assert_eq!(cursor, 5);
    }

    #[test]
    fn list_suggestions_uses_color_names_and_shared_prefix_matching() {
        let mut red = SuggestionsBuilderModel::new("r");
        assert_eq!(
            ColorArgumentModel::color().list_suggestions(&mut red),
            vec![
                SuggestionModel {
                    value: "dark_red".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "red".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "reset".to_string(),
                    tooltip: None,
                }
            ]
        );

        let mut dark = SuggestionsBuilderModel::new("dark_");
        assert_eq!(
            ColorArgumentModel::color()
                .list_suggestions(&mut dark)
                .into_iter()
                .map(|suggestion| suggestion.value)
                .collect::<Vec<_>>(),
            vec![
                "dark_blue",
                "dark_green",
                "dark_aqua",
                "dark_red",
                "dark_purple",
                "dark_gray"
            ]
        );
    }

    #[test]
    fn suggestions_include_reset_but_not_formatting_names() {
        let mut builder = SuggestionsBuilderModel::new("");
        let values = ColorArgumentModel::color()
            .list_suggestions(&mut builder)
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect::<Vec<_>>();

        assert!(values.contains(&"reset".to_string()));
        assert!(!values.contains(&"bold".to_string()));
        assert!(!values.contains(&"italic".to_string()));
    }
}
