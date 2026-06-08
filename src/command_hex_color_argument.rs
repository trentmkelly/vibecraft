use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexColorArgumentModel;

impl HexColorArgumentModel {
    pub fn hex_color() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<i32, HexColorParseError> {
        let color_string = reader.read_unquoted_string();
        match color_string.len() {
            3 => Ok(argb_color(
                duplicate_digit(parse_java_hex_component(&color_string, 0, 1)?),
                duplicate_digit(parse_java_hex_component(&color_string, 1, 2)?),
                duplicate_digit(parse_java_hex_component(&color_string, 2, 3)?),
            )),
            6 => Ok(argb_color(
                parse_java_hex_component(&color_string, 0, 2)?,
                parse_java_hex_component(&color_string, 2, 4)?,
                parse_java_hex_component(&color_string, 4, 6)?,
            )),
            _ => Err(HexColorParseError::InvalidHex {
                value: color_string,
            }),
        }
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(Self::examples().into_iter().map(str::to_string), builder);
        builder.clone().build()
    }

    pub fn examples() -> [&'static str; 2] {
        ["F00", "FF0000"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, i32>,
}

impl CommandContextModel {
    pub fn with_hex_color(mut self, name: impl Into<String>, value: i32) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_hex_color(context: &CommandContextModel, name: &str) -> Option<i32> {
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
pub enum HexColorParseError {
    InvalidHex { value: String },
    NumberFormat { value: String },
}

fn duplicate_digit(digit: i32) -> i32 {
    digit * 17
}

fn argb_color(red: i32, green: i32, blue: i32) -> i32 {
    i32::from_be_bytes([255, red as u8, green as u8, blue as u8])
}

fn parse_java_hex_component(
    input: &str,
    begin: usize,
    end: usize,
) -> Result<i32, HexColorParseError> {
    let component = &input[begin..end];
    let (sign, digits) = match component.as_bytes().first().copied() {
        Some(b'+') => (1, &component[1..]),
        Some(b'-') => (-1, &component[1..]),
        _ => (1, component),
    };
    if digits.is_empty() {
        return Err(HexColorParseError::NumberFormat {
            value: component.to_string(),
        });
    }

    i32::from_str_radix(digits, 16)
        .map(|value| value * sign)
        .map_err(|_| HexColorParseError::NumberFormat {
            value: component.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(i32, usize), HexColorParseError> {
        let mut reader = StringReaderModel::new(input);
        let color = HexColorArgumentModel::hex_color().parse(&mut reader)?;
        Ok((color, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(HexColorArgumentModel::examples(), ["F00", "FF0000"]);
    }

    #[test]
    fn get_hex_color_returns_typed_context_argument() {
        let context = CommandContextModel::default().with_hex_color("color", -65_536);

        assert_eq!(get_hex_color(&context, "color"), Some(-65_536));
        assert_eq!(get_hex_color(&context, "missing"), None);
    }

    #[test]
    fn parse_three_digit_color_duplicates_each_nibble() {
        let (color, cursor) = parse("F0a rest").unwrap();

        assert_eq!(color, -65_366);
        assert_eq!(cursor, 3);
    }

    #[test]
    fn parse_six_digit_color_reads_two_hex_digits_per_channel() {
        let (color, cursor) = parse("FF0000 tail").unwrap();

        assert_eq!(color, -65_536);
        assert_eq!(cursor, 6);
    }

    #[test]
    fn invalid_length_uses_command_syntax_error_value() {
        assert_eq!(
            parse("FFFF"),
            Err(HexColorParseError::InvalidHex {
                value: "FFFF".to_string(),
            })
        );
        assert_eq!(
            parse(""),
            Err(HexColorParseError::InvalidHex {
                value: String::new(),
            })
        );
    }

    #[test]
    fn invalid_hex_digit_escapes_as_number_format_equivalent() {
        assert_eq!(
            parse("GGG"),
            Err(HexColorParseError::NumberFormat {
                value: "G".to_string(),
            })
        );
    }

    #[test]
    fn six_character_channels_match_java_signed_slice_parse_and_argb_masking() {
        let (color, cursor) = parse("FF-1+0 rest").unwrap();

        assert_eq!(color, -256);
        assert_eq!(cursor, 6);
    }

    #[test]
    fn list_suggestions_uses_java_examples_and_shared_matching() {
        let mut all = SuggestionsBuilderModel::new("");
        assert_eq!(
            HexColorArgumentModel::hex_color().list_suggestions(&mut all),
            vec![
                SuggestionModel {
                    value: "F00".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "FF0000".to_string(),
                    tooltip: None,
                }
            ]
        );

        let mut six_digit = SuggestionsBuilderModel::new("ff");
        assert_eq!(
            HexColorArgumentModel::hex_color().list_suggestions(&mut six_digit),
            vec![SuggestionModel {
                value: "FF0000".to_string(),
                tooltip: None,
            }]
        );
    }
}
