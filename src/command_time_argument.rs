use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeArgumentModel {
    minimum: i32,
}

impl TimeArgumentModel {
    pub fn time() -> Self {
        Self { minimum: 0 }
    }

    pub fn time_minimum(minimum: i32) -> Self {
        Self { minimum }
    }

    pub fn minimum(self) -> i32 {
        self.minimum
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<i32, TimeParseError> {
        let value = reader.read_float()?;
        let unit = reader.read_unquoted_string();
        let factor = unit_factor(&unit).unwrap_or(0);
        if factor == 0 {
            return Err(TimeParseError::InvalidUnit);
        }

        let ticks = java_round(value * factor as f32);
        if ticks < self.minimum {
            Err(TimeParseError::TickCountTooLow {
                value: ticks,
                minimum: self.minimum,
            })
        } else {
            Ok(ticks)
        }
    }

    pub fn list_suggestions(&self, remaining: &str) -> TimeSuggestionsModel {
        let mut reader = StringReaderModel::new(remaining);
        if reader.read_float().is_err() {
            return TimeSuggestionsModel {
                offset: 0,
                suggestions: Vec::new(),
            };
        }

        let offset = reader.cursor();
        let prefix = &remaining[offset..];
        let suggestions = units()
            .into_iter()
            .filter(|unit| unit.starts_with(prefix))
            .map(str::to_string)
            .collect();
        TimeSuggestionsModel {
            offset,
            suggestions,
        }
    }

    pub fn examples(&self) -> [&'static str; 4] {
        ["0d", "0s", "0t", "0"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSuggestionsModel {
    pub offset: usize,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeArgumentInfoModel;

impl TimeArgumentInfoModel {
    pub fn serialize_to_network(template: TimeArgumentTemplateModel) -> [u8; 4] {
        template.min.to_be_bytes()
    }

    pub fn deserialize_from_network(bytes: [u8; 4]) -> TimeArgumentTemplateModel {
        TimeArgumentTemplateModel {
            min: i32::from_be_bytes(bytes),
        }
    }

    pub fn serialize_to_json(template: TimeArgumentTemplateModel) -> String {
        format!("{{\"min\":{}}}", template.min)
    }

    pub fn unpack(argument: TimeArgumentModel) -> TimeArgumentTemplateModel {
        TimeArgumentTemplateModel {
            min: argument.minimum,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeArgumentTemplateModel {
    min: i32,
}

impl TimeArgumentTemplateModel {
    pub fn instantiate(self) -> TimeArgumentModel {
        TimeArgumentModel::time_minimum(self.min)
    }

    pub fn min(self) -> i32 {
        self.min
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, i32>,
}

impl CommandContextModel {
    pub fn with_time(mut self, name: impl Into<String>, value: i32) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_time(context: &CommandContextModel, name: &str) -> Option<i32> {
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

    fn read_float(&mut self) -> Result<f32, TimeParseError> {
        let start = self.cursor;
        while self.can_read() && is_allowed_number(self.peek()) {
            self.cursor += 1;
        }
        if start == self.cursor {
            return Err(TimeParseError::ExpectedFloat);
        }
        self.input[start..self.cursor]
            .parse::<f32>()
            .map_err(|_| TimeParseError::InvalidFloat)
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
pub enum TimeParseError {
    ExpectedFloat,
    InvalidFloat,
    InvalidUnit,
    TickCountTooLow { value: i32, minimum: i32 },
}

fn is_allowed_number(character: char) -> bool {
    matches!(character, '0'..='9' | '.' | '-' | '+')
}

fn java_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

fn unit_factor(unit: &str) -> Option<i32> {
    match unit {
        "d" => Some(24_000),
        "s" => Some(20),
        "t" | "" => Some(1),
        _ => None,
    }
}

fn units() -> [&'static str; 4] {
    ["d", "s", "t", ""]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(i32, usize), TimeParseError> {
        let mut reader = StringReaderModel::new(input);
        let ticks = TimeArgumentModel::time().parse(&mut reader)?;
        Ok((ticks, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(
            TimeArgumentModel::time().examples(),
            ["0d", "0s", "0t", "0"]
        );
    }

    #[test]
    fn get_time_returns_typed_context_argument() {
        let context = CommandContextModel::default().with_time("duration", 24000);

        assert_eq!(get_time(&context, "duration"), Some(24000));
        assert_eq!(get_time(&context, "missing"), None);
    }

    #[test]
    fn parse_applies_java_unit_factors_and_default_ticks() {
        assert_eq!(parse("2d rest"), Ok((48_000, 2)));
        assert_eq!(parse("3s rest"), Ok((60, 2)));
        assert_eq!(parse("4t rest"), Ok((4, 2)));
        assert_eq!(parse("5 rest"), Ok((5, 1)));
    }

    #[test]
    fn parse_rounds_float_value_times_factor_like_java_math_round() {
        assert_eq!(parse("1.5s rest"), Ok((30, 4)));
        assert_eq!(parse("0.4t rest"), Ok((0, 4)));
        assert_eq!(parse("0.5t rest"), Ok((1, 4)));
    }

    #[test]
    fn parse_rejects_invalid_units_and_bad_floats() {
        assert_eq!(
            TimeArgumentModel::time().parse(&mut StringReaderModel::new("1x")),
            Err(TimeParseError::InvalidUnit)
        );
        assert_eq!(
            TimeArgumentModel::time().parse(&mut StringReaderModel::new("x")),
            Err(TimeParseError::ExpectedFloat)
        );
        assert_eq!(
            TimeArgumentModel::time().parse(&mut StringReaderModel::new("--1s")),
            Err(TimeParseError::InvalidFloat)
        );
    }

    #[test]
    fn parse_enforces_configured_minimum_after_rounding() {
        let mut too_low = StringReaderModel::new("4t");
        assert_eq!(
            TimeArgumentModel::time_minimum(5).parse(&mut too_low),
            Err(TimeParseError::TickCountTooLow {
                value: 4,
                minimum: 5,
            })
        );

        let mut exact = StringReaderModel::new("5t");
        assert_eq!(TimeArgumentModel::time_minimum(5).parse(&mut exact), Ok(5));
    }

    #[test]
    fn suggestions_are_empty_until_float_parses_then_offset_to_unit_suffix() {
        assert_eq!(
            TimeArgumentModel::time().list_suggestions("x"),
            TimeSuggestionsModel {
                offset: 0,
                suggestions: Vec::new(),
            }
        );

        let suggestions = TimeArgumentModel::time().list_suggestions("10");
        assert_eq!(suggestions.offset, 2);
        let mut values = suggestions.suggestions;
        values.sort();
        assert_eq!(values, vec!["", "d", "s", "t"]);

        assert_eq!(
            TimeArgumentModel::time().list_suggestions("10s"),
            TimeSuggestionsModel {
                offset: 2,
                suggestions: vec!["s".to_string()],
            }
        );
    }

    #[test]
    fn argument_info_template_serializes_min_and_instantiates_argument() {
        let argument = TimeArgumentModel::time_minimum(12);
        let template = TimeArgumentInfoModel::unpack(argument);

        assert_eq!(template.min(), 12);
        assert_eq!(
            TimeArgumentInfoModel::serialize_to_network(template),
            12_i32.to_be_bytes()
        );
        assert_eq!(
            TimeArgumentInfoModel::deserialize_from_network(7_i32.to_be_bytes()).instantiate(),
            TimeArgumentModel::time_minimum(7)
        );
        assert_eq!(
            TimeArgumentInfoModel::deserialize_from_network(7_i32.to_be_bytes())
                .instantiate()
                .minimum(),
            7
        );
        assert_eq!(
            TimeArgumentInfoModel::serialize_to_json(template),
            "{\"min\":12}"
        );
    }
}
