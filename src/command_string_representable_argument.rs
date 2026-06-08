use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringRepresentableArgumentModel {
    values: Vec<TestStringRepresentableModel>,
    convert_id: ConvertIdModel,
}

impl StringRepresentableArgumentModel {
    pub fn new(values: impl Into<Vec<TestStringRepresentableModel>>) -> Self {
        Self {
            values: values.into(),
            convert_id: ConvertIdModel::Identity,
        }
    }

    pub fn with_convert_id(
        values: impl Into<Vec<TestStringRepresentableModel>>,
        convert_id: ConvertIdModel,
    ) -> Self {
        Self {
            values: values.into(),
            convert_id,
        }
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<TestStringRepresentableModel, StringRepresentableParseError> {
        let id = reader.read_unquoted_string();
        self.values
            .iter()
            .copied()
            .find(|value| value.serialized_name() == id)
            .ok_or(StringRepresentableParseError::InvalidValue { value: id })
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(
            self.values
                .iter()
                .map(|value| self.convert_id.convert(value.serialized_name())),
            builder,
        );
        builder.clone().build()
    }

    pub fn examples(&self) -> Vec<String> {
        self.values
            .iter()
            .take(2)
            .map(|value| self.convert_id.convert(value.serialized_name()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertIdModel {
    Identity,
    Lowercase,
    PrefixHash,
}

impl ConvertIdModel {
    fn convert(self, id: &str) -> String {
        match self {
            Self::Identity => id.to_string(),
            Self::Lowercase => id.to_lowercase(),
            Self::PrefixHash => format!("#{id}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStringRepresentableModel {
    First,
    Second,
    Third,
}

impl TestStringRepresentableModel {
    pub const VALUES: [Self; 3] = [Self::First, Self::Second, Self::Third];

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::First => "Alpha.Value",
            Self::Second => "beta_value",
            Self::Third => "gamma/value",
        }
    }
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
pub enum StringRepresentableParseError {
    InvalidValue { value: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_argument() -> StringRepresentableArgumentModel {
        StringRepresentableArgumentModel::new(TestStringRepresentableModel::VALUES)
    }

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_parse_reads_one_unquoted_string_and_uses_exact_codec_id() {
        let mut reader = StringReaderModel::new("beta_value trailing");
        let parsed = default_argument().parse(&mut reader);

        assert_eq!(parsed, Ok(TestStringRepresentableModel::Second));
        assert_eq!(reader.cursor(), "beta_value".len());
    }

    #[test]
    fn java_parse_accepts_punctuation_until_whitespace() {
        let mut reader = StringReaderModel::new("gamma/value tail");
        let parsed = default_argument().parse(&mut reader);

        assert_eq!(parsed, Ok(TestStringRepresentableModel::Third));
        assert_eq!(reader.cursor(), "gamma/value".len());
    }

    #[test]
    fn java_invalid_value_keeps_cursor_after_consumed_token() {
        let mut reader = StringReaderModel::new("missing next");
        let error = default_argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(StringRepresentableParseError::InvalidValue {
                value: "missing".to_string()
            })
        );
        assert_eq!(reader.cursor(), "missing".len());
    }

    #[test]
    fn java_empty_input_is_invalid_empty_value_without_cursor_movement() {
        let mut reader = StringReaderModel::new("");
        let error = default_argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(StringRepresentableParseError::InvalidValue {
                value: String::new()
            })
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_examples_are_first_two_values_after_convert_id() {
        assert_eq!(
            default_argument().examples(),
            vec!["Alpha.Value".to_string(), "beta_value".to_string()]
        );

        let converted = StringRepresentableArgumentModel::with_convert_id(
            TestStringRepresentableModel::VALUES,
            ConvertIdModel::Lowercase,
        );
        assert_eq!(
            converted.examples(),
            vec!["alpha.value".to_string(), "beta_value".to_string()]
        );
    }

    #[test]
    fn java_suggestions_apply_convert_id_before_shared_filtering() {
        let argument = StringRepresentableArgumentModel::with_convert_id(
            TestStringRepresentableModel::VALUES,
            ConvertIdModel::Lowercase,
        );
        let mut builder = SuggestionsBuilderModel::new("alp");

        assert_eq!(
            suggestion_values(argument.list_suggestions(&mut builder)),
            vec!["alpha.value".to_string()]
        );
    }

    #[test]
    fn java_shared_suggestion_matching_sees_converted_ids() {
        let argument = StringRepresentableArgumentModel::with_convert_id(
            TestStringRepresentableModel::VALUES,
            ConvertIdModel::PrefixHash,
        );
        let mut builder = SuggestionsBuilderModel::new("#g");

        assert_eq!(
            suggestion_values(argument.list_suggestions(&mut builder)),
            vec!["#gamma/value".to_string()]
        );
    }

    #[test]
    fn java_convert_id_does_not_affect_parsing() {
        let argument = StringRepresentableArgumentModel::with_convert_id(
            TestStringRepresentableModel::VALUES,
            ConvertIdModel::Lowercase,
        );

        let mut converted_reader = StringReaderModel::new("alpha.value");
        assert_eq!(
            argument.parse(&mut converted_reader),
            Err(StringRepresentableParseError::InvalidValue {
                value: "alpha.value".to_string()
            })
        );
        assert_eq!(converted_reader.cursor(), "alpha.value".len());

        let mut raw_reader = StringReaderModel::new("Alpha.Value");
        assert_eq!(
            argument.parse(&mut raw_reader),
            Ok(TestStringRepresentableModel::First)
        );
        assert_eq!(raw_reader.cursor(), "Alpha.Value".len());
    }
}
