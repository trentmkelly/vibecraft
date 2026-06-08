use std::collections::HashMap;

use crate::criterion_min_max_bounds::{BoundsModel, DoublesBoundsModel, IntsBoundsModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntRangeArgumentModel;

impl IntRangeArgumentModel {
    pub fn int_range() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<IntsBoundsModel, RangeParseError> {
        let start = reader.cursor();
        match parse_bounds(reader, str::parse::<i32>) {
            Ok(bounds) if bounds.are_swapped() => {
                reader.set_cursor(start);
                Err(RangeParseError::Swapped { cursor: start })
            }
            Ok(bounds) => Ok(IntsBoundsModel::new(bounds)),
            Err(error) => {
                reader.set_cursor(start);
                Err(error.with_cursor(start))
            }
        }
    }

    pub fn examples(&self) -> [&'static str; 5] {
        ["0..5", "0", "-5", "-100..", "..100"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatRangeArgumentModel;

impl FloatRangeArgumentModel {
    pub fn float_range() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<DoublesBoundsModel, RangeParseError> {
        let start = reader.cursor();
        match parse_bounds(reader, str::parse::<f64>) {
            Ok(bounds) if bounds.are_swapped() => {
                reader.set_cursor(start);
                Err(RangeParseError::Swapped { cursor: start })
            }
            Ok(bounds) => Ok(DoublesBoundsModel::new(bounds)),
            Err(error) => {
                reader.set_cursor(start);
                Err(error.with_cursor(start))
            }
        }
    }

    pub fn examples(&self) -> [&'static str; 5] {
        ["0..5.2", "0", "-5.4", "-100.76..", "..100"]
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandContextModel {
    int_arguments: HashMap<String, IntsBoundsModel>,
    float_arguments: HashMap<String, DoublesBoundsModel>,
}

impl CommandContextModel {
    pub fn with_int_range(mut self, name: impl Into<String>, value: IntsBoundsModel) -> Self {
        self.int_arguments.insert(name.into(), value);
        self
    }

    pub fn with_float_range(mut self, name: impl Into<String>, value: DoublesBoundsModel) -> Self {
        self.float_arguments.insert(name.into(), value);
        self
    }
}

pub fn get_int_range(context: &CommandContextModel, name: &str) -> Option<IntsBoundsModel> {
    context.int_arguments.get(name).copied()
}

pub fn get_float_range(context: &CommandContextModel, name: &str) -> Option<DoublesBoundsModel> {
    context.float_arguments.get(name).copied()
}

fn parse_bounds<T, E>(
    reader: &mut StringReaderModel,
    converter: impl Fn(&str) -> Result<T, E>,
) -> Result<BoundsModel<T>, RangeParseError>
where
    T: Copy + PartialEq + PartialOrd,
{
    if !reader.can_read() {
        return Err(RangeParseError::Empty {
            cursor: reader.cursor(),
        });
    }

    let start = reader.cursor();
    let min = read_number(reader, &converter)?;
    let max = if reader.can_read_n(2) && reader.peek() == '.' && reader.peek_at(1) == '.' {
        reader.skip();
        reader.skip();
        read_number(reader, &converter)?
    } else {
        min
    };

    if reader.cursor() == start || (min.is_none() && max.is_none()) {
        Err(RangeParseError::Empty { cursor: start })
    } else {
        Ok(BoundsModel::new(min, max))
    }
}

fn read_number<T, E>(
    reader: &mut StringReaderModel,
    converter: &impl Fn(&str) -> Result<T, E>,
) -> Result<Option<T>, RangeParseError>
where
    T: Copy,
{
    let start = reader.cursor();
    while reader.can_read() && is_allowed_input_char(reader) {
        reader.skip();
    }

    let number = reader.slice(start, reader.cursor());
    if number.is_empty() {
        return Ok(None);
    }

    converter(number)
        .map(Some)
        .map_err(|_| RangeParseError::InvalidNumber { cursor: start })
}

fn is_allowed_input_char(reader: &StringReaderModel) -> bool {
    let value = reader.peek();
    value.is_ascii_digit()
        || value == '-'
        || (value == '.' && (!reader.can_read_n(2) || reader.peek_at(1) != '.'))
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

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn can_read_n(&self, count: usize) -> bool {
        self.cursor + count <= self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn peek_at(&self, offset: usize) -> char {
        self.input.as_bytes()[self.cursor + offset] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeParseError {
    Empty { cursor: usize },
    InvalidNumber { cursor: usize },
    Swapped { cursor: usize },
}

impl RangeParseError {
    fn with_cursor(self, cursor: usize) -> Self {
        match self {
            Self::Empty { .. } => Self::Empty { cursor },
            Self::InvalidNumber { .. } => Self::InvalidNumber { cursor },
            Self::Swapped { .. } => Self::Swapped { cursor },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_int(input: &str) -> Result<(IntsBoundsModel, usize), RangeParseError> {
        let mut reader = StringReaderModel::new(input);
        let range = IntRangeArgumentModel::int_range().parse(&mut reader)?;
        Ok((range, reader.cursor()))
    }

    fn parse_float(input: &str) -> Result<(DoublesBoundsModel, usize), RangeParseError> {
        let mut reader = StringReaderModel::new(input);
        let range = FloatRangeArgumentModel::float_range().parse(&mut reader)?;
        Ok((range, reader.cursor()))
    }

    #[test]
    fn java_factories_examples_and_context_lookup_match_source() {
        assert_eq!(
            IntRangeArgumentModel::int_range().examples(),
            ["0..5", "0", "-5", "-100..", "..100"]
        );
        assert_eq!(
            FloatRangeArgumentModel::float_range().examples(),
            ["0..5.2", "0", "-5.4", "-100.76..", "..100"]
        );

        let int_range = IntsBoundsModel::between(0, 5);
        let float_range = DoublesBoundsModel::at_most(3.5);
        let context = CommandContextModel::default()
            .with_int_range("i", int_range)
            .with_float_range("f", float_range);
        assert_eq!(get_int_range(&context, "i"), Some(int_range));
        assert_eq!(get_float_range(&context, "f"), Some(float_range));
        assert_eq!(get_int_range(&context, "missing"), None);
    }

    #[test]
    fn java_int_range_parse_delegates_to_min_max_bounds_ints() {
        let (exact, exact_cursor) = parse_int("5 next").unwrap();
        assert_eq!(exact.bounds(), BoundsModel::exactly(5));
        assert_eq!(exact_cursor, "5".len());

        let (closed, closed_cursor) = parse_int("-5..10 next").unwrap();
        assert_eq!(closed.bounds(), BoundsModel::between(-5, 10));
        assert_eq!(closed_cursor, "-5..10".len());

        assert_eq!(
            parse_int("-100..").unwrap().0.bounds(),
            BoundsModel::at_least(-100)
        );
        assert_eq!(
            parse_int("..100").unwrap().0.bounds(),
            BoundsModel::at_most(100)
        );
    }

    #[test]
    fn java_float_range_parse_delegates_to_min_max_bounds_doubles() {
        let (closed, cursor) = parse_float("0..5.2 next").unwrap();
        assert_eq!(closed.bounds(), BoundsModel::between(0.0, 5.2));
        assert_eq!(cursor, "0..5.2".len());

        assert_eq!(
            parse_float("-5.4").unwrap().0.bounds(),
            BoundsModel::exactly(-5.4)
        );
        assert_eq!(
            parse_float("-100.76..").unwrap().0.bounds(),
            BoundsModel::at_least(-100.76)
        );
        assert_eq!(
            parse_float("..100").unwrap().0.bounds(),
            BoundsModel::at_most(100.0)
        );
    }

    #[test]
    fn java_parse_stops_number_before_double_dot_separator() {
        let (range, cursor) = parse_float("1..x").unwrap();

        assert_eq!(range.bounds(), BoundsModel::at_least(1.0));
        assert_eq!(cursor, "1..".len());
    }

    #[test]
    fn java_empty_invalid_and_swapped_ranges_reset_cursor_to_start() {
        let mut empty = StringReaderModel::new("");
        assert_eq!(
            IntRangeArgumentModel::int_range().parse(&mut empty),
            Err(RangeParseError::Empty { cursor: 0 })
        );
        assert_eq!(empty.cursor(), 0);

        let mut dots = StringReaderModel::new(".. rest");
        assert_eq!(
            IntRangeArgumentModel::int_range().parse(&mut dots),
            Err(RangeParseError::Empty { cursor: 0 })
        );
        assert_eq!(dots.cursor(), 0);

        let mut invalid_int = StringReaderModel::new("1.2 rest");
        assert_eq!(
            IntRangeArgumentModel::int_range().parse(&mut invalid_int),
            Err(RangeParseError::InvalidNumber { cursor: 0 })
        );
        assert_eq!(invalid_int.cursor(), 0);

        let mut swapped = StringReaderModel::new("8..4 rest");
        assert_eq!(
            IntRangeArgumentModel::int_range().parse(&mut swapped),
            Err(RangeParseError::Swapped { cursor: 0 })
        );
        assert_eq!(swapped.cursor(), 0);
    }

    #[test]
    fn java_float_range_rejects_swapped_bounds_but_not_open_negative_bounds() {
        let mut swapped = StringReaderModel::new("5.2..0");
        assert_eq!(
            FloatRangeArgumentModel::float_range().parse(&mut swapped),
            Err(RangeParseError::Swapped { cursor: 0 })
        );
        assert_eq!(swapped.cursor(), 0);

        assert_eq!(
            parse_float("..-1.5").unwrap().0.bounds(),
            BoundsModel::at_most(-1.5)
        );
    }
}
