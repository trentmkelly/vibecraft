use std::collections::HashMap;

use crate::command_shared_suggestion_provider::{
    suggest, SuggestionModel, SuggestionsBuilderModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationArgumentModel;

impl OperationArgumentModel {
    pub fn operation() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<OperationModel, OperationArgumentError> {
        if !reader.can_read() {
            return Err(OperationArgumentError::InvalidOperation);
        }

        let start = reader.cursor();
        while reader.can_read() && reader.peek() != ' ' {
            reader.skip();
        }
        OperationModel::parse(reader.slice(start, reader.cursor()))
    }

    pub fn list_suggestions(&self, builder: &mut SuggestionsBuilderModel) -> Vec<SuggestionModel> {
        suggest(
            ["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"]
                .into_iter()
                .map(str::to_string),
            builder,
        );
        builder.clone().build()
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["=", ">", "<"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, OperationModel>,
}

impl CommandContextModel {
    pub fn with_operation(mut self, name: impl Into<String>, value: OperationModel) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_operation(context: &CommandContextModel, name: &str) -> Option<OperationModel> {
    context.arguments.get(name).copied()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationModel {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    Min,
    Max,
    Swap,
}

impl OperationModel {
    fn parse(value: &str) -> Result<Self, OperationArgumentError> {
        match value {
            "=" => Ok(Self::Assign),
            "+=" => Ok(Self::AddAssign),
            "-=" => Ok(Self::SubtractAssign),
            "*=" => Ok(Self::MultiplyAssign),
            "/=" => Ok(Self::DivideAssign),
            "%=" => Ok(Self::ModuloAssign),
            "<" => Ok(Self::Min),
            ">" => Ok(Self::Max),
            "><" => Ok(Self::Swap),
            _ => Err(OperationArgumentError::InvalidOperation),
        }
    }

    pub fn apply(
        self,
        left: &mut ScoreAccessModel,
        right: &mut ScoreAccessModel,
    ) -> Result<(), OperationArgumentError> {
        if self == Self::Swap {
            let swap = left.get();
            left.set(right.get());
            right.set(swap);
            return Ok(());
        }

        let value = self.apply_simple(left.get(), right.get())?;
        left.set(value);
        Ok(())
    }

    fn apply_simple(self, left: i32, right: i32) -> Result<i32, OperationArgumentError> {
        match self {
            Self::Assign => Ok(right),
            Self::AddAssign => Ok(left.wrapping_add(right)),
            Self::SubtractAssign => Ok(left.wrapping_sub(right)),
            Self::MultiplyAssign => Ok(left.wrapping_mul(right)),
            Self::DivideAssign => {
                if right == 0 {
                    Err(OperationArgumentError::DivideByZero)
                } else {
                    Ok(java_floor_div(left, right))
                }
            }
            Self::ModuloAssign => {
                if right == 0 {
                    Err(OperationArgumentError::DivideByZero)
                } else {
                    Ok(java_floor_mod(left, right))
                }
            }
            Self::Min => Ok(left.min(right)),
            Self::Max => Ok(left.max(right)),
            Self::Swap => unreachable!("swap is handled before simple operation application"),
        }
    }
}

fn java_floor_div(left: i32, right: i32) -> i32 {
    let quotient = left.wrapping_div(right);
    let remainder = left.wrapping_rem(right);
    if remainder != 0 && ((left ^ right) < 0) {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
}

fn java_floor_mod(left: i32, right: i32) -> i32 {
    left.wrapping_sub(java_floor_div(left, right).wrapping_mul(right))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreAccessModel {
    value: i32,
}

impl ScoreAccessModel {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
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

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationArgumentError {
    InvalidOperation,
    DivideByZero,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(OperationModel, usize), OperationArgumentError> {
        let mut reader = StringReaderModel::new(input);
        let operation = OperationArgumentModel::operation().parse(&mut reader)?;
        Ok((operation, reader.cursor()))
    }

    fn apply(
        operation: OperationModel,
        left: i32,
        right: i32,
    ) -> Result<i32, OperationArgumentError> {
        let mut left = ScoreAccessModel::new(left);
        let mut right = ScoreAccessModel::new(right);
        operation.apply(&mut left, &mut right)?;
        Ok(left.get())
    }

    fn suggestion_values(suggestions: Vec<SuggestionModel>) -> Vec<String> {
        suggestions
            .into_iter()
            .map(|suggestion| suggestion.value)
            .collect()
    }

    #[test]
    fn java_factory_examples_and_context_lookup_match_source() {
        assert_eq!(
            OperationArgumentModel::operation().examples(),
            ["=", ">", "<"]
        );

        let context =
            CommandContextModel::default().with_operation("op", OperationModel::MultiplyAssign);
        assert_eq!(
            get_operation(&context, "op"),
            Some(OperationModel::MultiplyAssign)
        );
        assert_eq!(get_operation(&context, "missing"), None);
    }

    #[test]
    fn java_parse_reads_until_space_and_keeps_cursor_on_valid_operation() {
        let (operation, cursor) = parse("+= trailing").unwrap();

        assert_eq!(operation, OperationModel::AddAssign);
        assert_eq!(cursor, "+=".len());
    }

    #[test]
    fn java_parse_rejects_empty_input_without_cursor_movement() {
        let mut reader = StringReaderModel::new("");
        let error = OperationArgumentModel::operation().parse(&mut reader);

        assert_eq!(error, Err(OperationArgumentError::InvalidOperation));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_parse_rejects_unknown_operation_without_cursor_reset() {
        let mut reader = StringReaderModel::new("?? trailing");
        let error = OperationArgumentModel::operation().parse(&mut reader);

        assert_eq!(error, Err(OperationArgumentError::InvalidOperation));
        assert_eq!(reader.cursor(), "??".len());
    }

    #[test]
    fn java_suggestions_include_all_operation_tokens_in_source_order() {
        let mut builder = SuggestionsBuilderModel::new("");
        assert_eq!(
            suggestion_values(OperationArgumentModel::operation().list_suggestions(&mut builder)),
            vec!["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"]
        );

        let mut filtered = SuggestionsBuilderModel::new(">");
        assert_eq!(
            suggestion_values(OperationArgumentModel::operation().list_suggestions(&mut filtered)),
            vec![">", "><"]
        );
    }

    #[test]
    fn java_simple_operations_mutate_left_score_from_right_score() {
        assert_eq!(apply(OperationModel::Assign, 7, 3), Ok(3));
        assert_eq!(apply(OperationModel::AddAssign, 7, 3), Ok(10));
        assert_eq!(apply(OperationModel::SubtractAssign, 7, 3), Ok(4));
        assert_eq!(apply(OperationModel::MultiplyAssign, 7, 3), Ok(21));
        assert_eq!(apply(OperationModel::Min, 7, 3), Ok(3));
        assert_eq!(apply(OperationModel::Max, 7, 3), Ok(7));
    }

    #[test]
    fn java_arithmetic_uses_i32_wrapping_like_int_operations() {
        assert_eq!(apply(OperationModel::AddAssign, i32::MAX, 1), Ok(i32::MIN));
        assert_eq!(apply(OperationModel::MultiplyAssign, i32::MAX, 2), Ok(-2));
    }

    #[test]
    fn java_division_and_modulo_use_floor_semantics() {
        assert_eq!(apply(OperationModel::DivideAssign, -7, 3), Ok(-3));
        assert_eq!(apply(OperationModel::DivideAssign, 7, -3), Ok(-3));
        assert_eq!(
            apply(OperationModel::DivideAssign, i32::MIN, -1),
            Ok(i32::MIN)
        );
        assert_eq!(apply(OperationModel::ModuloAssign, -7, 3), Ok(2));
        assert_eq!(apply(OperationModel::ModuloAssign, 7, -3), Ok(-2));
        assert_eq!(apply(OperationModel::ModuloAssign, i32::MIN, -1), Ok(0));
    }

    #[test]
    fn java_division_and_modulo_reject_zero_divisor_without_mutating_left() {
        let mut left = ScoreAccessModel::new(7);
        let mut right = ScoreAccessModel::new(0);
        assert_eq!(
            OperationModel::DivideAssign.apply(&mut left, &mut right),
            Err(OperationArgumentError::DivideByZero)
        );
        assert_eq!(left.get(), 7);

        assert_eq!(
            OperationModel::ModuloAssign.apply(&mut left, &mut right),
            Err(OperationArgumentError::DivideByZero)
        );
        assert_eq!(left.get(), 7);
    }

    #[test]
    fn java_swap_operation_exchanges_both_scores() {
        let mut left = ScoreAccessModel::new(5);
        let mut right = ScoreAccessModel::new(9);

        OperationModel::Swap.apply(&mut left, &mut right).unwrap();

        assert_eq!(left.get(), 9);
        assert_eq!(right.get(), 5);
    }
}
