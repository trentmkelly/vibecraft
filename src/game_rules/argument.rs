//! Brigadier-exact parsing of game-rule values.
//!
//! Java `GameRule.deserialize` (used by `ServerboundSetGameRulePacket`) and the
//! `/gamerule <rule> <value>` command both parse the value with the rule's
//! `ArgumentType` (`BoolArgumentType.bool()` / `IntegerArgumentType.integer(min, max)`),
//! so the failure messages below are the literal Brigadier `CommandSyntaxException` texts.

use super::{GameRuleDefinition, GameRuleType, GameRuleValue};

/// A Brigadier parse failure for a game-rule value argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameRuleArgumentError {
    /// `BuiltInExceptions.readerExpectedBool`
    ExpectedBool,
    /// `BuiltInExceptions.readerInvalidBool`
    InvalidBool(String),
    /// `BuiltInExceptions.readerExpectedInt`
    ExpectedInteger,
    /// `BuiltInExceptions.readerInvalidInt`
    InvalidInteger(String),
    /// `BuiltInExceptions.integerTooLow`
    IntegerTooLow { found: i32, min: i32 },
    /// `BuiltInExceptions.integerTooHigh`
    IntegerTooHigh { found: i32, max: i32 },
    /// `BuiltInExceptions.readerExpectedEndOfQuote` (also unsupported escapes).
    ExpectedEndOfQuote,
    /// `CommandDispatcher`: text directly follows a parsed argument without a space.
    ExpectedArgumentSeparator,
    /// Text remained after the argument (`StringReader.canRead()` in `GameRule.deserialize`).
    TrailingData,
}

impl GameRuleArgumentError {
    /// The English Brigadier message for this failure.
    pub fn message(&self) -> String {
        match self {
            Self::ExpectedBool => "Expected bool".to_string(),
            Self::InvalidBool(value) => {
                format!("Invalid bool, expected true or false but found '{value}'")
            }
            Self::ExpectedInteger => "Expected integer".to_string(),
            Self::InvalidInteger(value) => format!("Invalid integer '{value}'"),
            Self::IntegerTooLow { found, min } => {
                format!("Integer must not be less than {min}, found {found}")
            }
            Self::IntegerTooHigh { found, max } => {
                format!("Integer must not be more than {max}, found {found}")
            }
            Self::ExpectedEndOfQuote => "Unclosed quoted string".to_string(),
            Self::ExpectedArgumentSeparator => {
                "Expected whitespace to end one argument".to_string()
            }
            Self::TrailingData => "Incorrect argument for command".to_string(),
        }
    }
}

/// `StringReader.isAllowedInUnquotedString`
fn is_unquoted_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+')
}

/// `StringReader.isAllowedNumber`
fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '.' || ch == '-'
}

/// `StringReader.readString`: returns the string and the unread remainder.
fn read_string(input: &str) -> Result<(String, &str), GameRuleArgumentError> {
    match input.chars().next() {
        Some(quote @ ('"' | '\'')) => {
            let mut value = String::new();
            let mut escaped = false;
            for (index, ch) in input.char_indices().skip(1) {
                if escaped {
                    if ch != quote && ch != '\\' {
                        return Err(GameRuleArgumentError::ExpectedEndOfQuote);
                    }
                    value.push(ch);
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    return Ok((value, &input[index + ch.len_utf8()..]));
                } else {
                    value.push(ch);
                }
            }
            Err(GameRuleArgumentError::ExpectedEndOfQuote)
        }
        _ => {
            let end = input
                .char_indices()
                .find(|(_, ch)| !is_unquoted_char(*ch))
                .map_or(input.len(), |(index, _)| index);
            Ok((input[..end].to_string(), &input[end..]))
        }
    }
}

/// Parses one argument token and returns the value plus the unread remainder,
/// exactly like `ArgumentType.parse(StringReader)`.
pub fn parse_argument_prefix<'a>(
    input: &'a str,
    definition: &GameRuleDefinition,
) -> Result<(GameRuleValue, &'a str), GameRuleArgumentError> {
    match definition.rule_type {
        GameRuleType::Bool => {
            let (value, rest) = read_string(input)?;
            match value.as_str() {
                "" => Err(GameRuleArgumentError::ExpectedBool),
                "true" => Ok((GameRuleValue::Bool(true), rest)),
                "false" => Ok((GameRuleValue::Bool(false), rest)),
                _ => Err(GameRuleArgumentError::InvalidBool(value)),
            }
        }
        GameRuleType::Int => {
            let end = input
                .char_indices()
                .find(|(_, ch)| !is_number_char(*ch))
                .map_or(input.len(), |(index, _)| index);
            let number = &input[..end];
            if number.is_empty() {
                return Err(GameRuleArgumentError::ExpectedInteger);
            }
            let value = number
                .parse::<i32>()
                .map_err(|_| GameRuleArgumentError::InvalidInteger(number.to_string()))?;
            if let Some(min) = definition.min.filter(|min| value < *min) {
                return Err(GameRuleArgumentError::IntegerTooLow { found: value, min });
            }
            if let Some(max) = definition.max.filter(|max| value > *max) {
                return Err(GameRuleArgumentError::IntegerTooHigh { found: value, max });
            }
            Ok((GameRuleValue::Int(value), &input[end..]))
        }
    }
}

/// `GameRule.deserialize`: the whole string must be one valid value.
pub fn deserialize_game_rule_value(
    raw: &str,
    definition: &GameRuleDefinition,
) -> Result<GameRuleValue, GameRuleArgumentError> {
    let (value, rest) = parse_argument_prefix(raw, definition)?;
    if rest.is_empty() {
        Ok(value)
    } else {
        Err(GameRuleArgumentError::TrailingData)
    }
}
