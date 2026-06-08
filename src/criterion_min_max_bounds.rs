#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeShape<T> {
    All,
    AtLeast(T),
    AtMost(T),
    Closed(T, T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecShape<T> {
    Range(BoundsModel<T>),
    Point(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    Empty,
    Swapped,
    InvalidNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamEncoded<T> {
    flags: u8,
    min: Option<T>,
    max: Option<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundsModel<T> {
    min: Option<T>,
    max: Option<T>,
}

impl<T> BoundsModel<T>
where
    T: Copy + PartialEq + PartialOrd,
{
    pub fn new(min: Option<T>, max: Option<T>) -> Self {
        Self { min, max }
    }

    pub fn any() -> Self {
        Self::new(None, None)
    }

    pub fn exactly(value: T) -> Self {
        Self::new(Some(value), Some(value))
    }

    pub fn between(min: T, max: T) -> Self {
        Self::new(Some(min), Some(max))
    }

    pub fn at_least(value: T) -> Self {
        Self::new(Some(value), None)
    }

    pub fn at_most(value: T) -> Self {
        Self::new(None, Some(value))
    }

    pub fn min(&self) -> Option<T> {
        self.min
    }

    pub fn max(&self) -> Option<T> {
        self.max
    }

    pub fn is_any(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }

    pub fn are_swapped(&self) -> bool {
        self.min.zip(self.max).is_some_and(|(min, max)| min > max)
    }

    pub fn validate_swapped_bounds_in_codec(&self) -> Result<Self, String> {
        if self.are_swapped() {
            Err("Swapped bounds in range".to_string())
        } else {
            Ok(*self)
        }
    }

    pub fn as_range(&self) -> RangeShape<T> {
        match (self.min, self.max) {
            (Some(min), Some(max)) => RangeShape::Closed(min, max),
            (Some(min), None) => RangeShape::AtLeast(min),
            (None, Some(max)) => RangeShape::AtMost(max),
            (None, None) => RangeShape::All,
        }
    }

    pub fn as_point(&self) -> Option<T> {
        (self.min == self.max).then_some(self.min).flatten()
    }

    pub fn map<U>(self, mapper: impl Fn(T) -> U) -> BoundsModel<U>
    where
        U: Copy + PartialEq + PartialOrd,
    {
        BoundsModel::new(self.min.map(&mapper), self.max.map(mapper))
    }

    pub fn codec_shape(&self) -> CodecShape<T> {
        self.as_point()
            .map_or(CodecShape::Range(*self), CodecShape::Point)
    }

    pub fn stream_encode(&self) -> StreamEncoded<T> {
        StreamEncoded {
            flags: u8::from(self.min.is_some()) | (u8::from(self.max.is_some()) << 1),
            min: self.min,
            max: self.max,
        }
    }

    pub fn stream_decode(encoded: StreamEncoded<T>) -> Self {
        Self::new(
            ((encoded.flags & 1) != 0).then_some(encoded.min).flatten(),
            ((encoded.flags & 2) != 0).then_some(encoded.max).flatten(),
        )
    }

    pub fn contains_bounds(&self, target: &BoundsModel<T>) -> bool {
        self.min
            .zip(target.min)
            .is_none_or(|(allowed, target)| allowed <= target)
            && self
                .max
                .zip(target.max)
                .is_none_or(|(allowed, target)| allowed >= target)
            && !(self.min.is_some() && target.min.is_none())
            && !(self.max.is_some() && target.max.is_none())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoublesBoundsModel {
    bounds: BoundsModel<f64>,
    bounds_sqr: BoundsModel<f64>,
}

impl DoublesBoundsModel {
    pub const ANY: Self = Self {
        bounds: BoundsModel {
            min: None,
            max: None,
        },
        bounds_sqr: BoundsModel {
            min: None,
            max: None,
        },
    };

    pub fn new(bounds: BoundsModel<f64>) -> Self {
        Self {
            bounds,
            bounds_sqr: bounds.map(|value| value * value),
        }
    }

    pub fn exactly(value: f64) -> Self {
        Self::new(BoundsModel::exactly(value))
    }

    pub fn between(min: f64, max: f64) -> Self {
        Self::new(BoundsModel::between(min, max))
    }

    pub fn at_least(value: f64) -> Self {
        Self::new(BoundsModel::at_least(value))
    }

    pub fn at_most(value: f64) -> Self {
        Self::new(BoundsModel::at_most(value))
    }

    pub fn bounds(&self) -> BoundsModel<f64> {
        self.bounds
    }

    pub fn matches(&self, value: f64) -> bool {
        self.bounds.min.is_none_or(|min| min <= value)
            && self.bounds.max.is_none_or(|max| max >= value)
    }

    pub fn matches_sqr(&self, value_sqr: f64) -> bool {
        self.bounds_sqr.min.is_none_or(|min| min <= value_sqr)
            && self.bounds_sqr.max.is_none_or(|max| max >= value_sqr)
    }

    pub fn from_reader(input: &str) -> Result<Self, ParseError> {
        let bounds = parse_bounds(input, str::parse::<f64>)?;
        if bounds.are_swapped() {
            Err(ParseError {
                kind: ParseErrorKind::Swapped,
                cursor: 0,
            })
        } else {
            Ok(Self::new(bounds))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatDegreesBoundsModel {
    bounds: BoundsModel<f32>,
}

impl FloatDegreesBoundsModel {
    pub const ANY: Self = Self {
        bounds: BoundsModel {
            min: None,
            max: None,
        },
    };

    pub fn from_reader(input: &str) -> Result<Self, ParseError> {
        Ok(Self {
            bounds: parse_bounds(input, str::parse::<f32>)?,
        })
    }

    pub fn bounds(&self) -> BoundsModel<f32> {
        self.bounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntsBoundsModel {
    bounds: BoundsModel<i32>,
    bounds_sqr: BoundsModel<i64>,
}

impl IntsBoundsModel {
    pub const ANY: Self = Self {
        bounds: BoundsModel {
            min: None,
            max: None,
        },
        bounds_sqr: BoundsModel {
            min: None,
            max: None,
        },
    };

    pub fn new(bounds: BoundsModel<i32>) -> Self {
        Self {
            bounds,
            bounds_sqr: bounds.map(|value| {
                let value = i64::from(value);
                value * value
            }),
        }
    }

    pub fn exactly(value: i32) -> Self {
        Self::new(BoundsModel::exactly(value))
    }

    pub fn between(min: i32, max: i32) -> Self {
        Self::new(BoundsModel::between(min, max))
    }

    pub fn at_least(value: i32) -> Self {
        Self::new(BoundsModel::at_least(value))
    }

    pub fn at_most(value: i32) -> Self {
        Self::new(BoundsModel::at_most(value))
    }

    pub fn bounds(&self) -> BoundsModel<i32> {
        self.bounds
    }

    pub fn matches(&self, value: i32) -> bool {
        self.bounds.min.is_none_or(|min| min <= value)
            && self.bounds.max.is_none_or(|max| max >= value)
    }

    pub fn matches_sqr(&self, value_sqr: i64) -> bool {
        self.bounds_sqr.min.is_none_or(|min| min <= value_sqr)
            && self.bounds_sqr.max.is_none_or(|max| max >= value_sqr)
    }

    pub fn from_reader(input: &str) -> Result<Self, ParseError> {
        let bounds = parse_bounds(input, str::parse::<i32>)?;
        if bounds.are_swapped() {
            Err(ParseError {
                kind: ParseErrorKind::Swapped,
                cursor: 0,
            })
        } else {
            Ok(Self::new(bounds))
        }
    }
}

fn parse_bounds<T, E>(
    input: &str,
    converter: impl Fn(&str) -> Result<T, E>,
) -> Result<BoundsModel<T>, ParseError>
where
    T: Copy + PartialEq + PartialOrd,
    E: std::fmt::Debug,
{
    if input.is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::Empty,
            cursor: 0,
        });
    }

    let start = 0;
    let (min, cursor) = read_number(input, start, &converter)?;
    let (max, cursor) = if input[cursor..].starts_with("..") {
        read_number(input, cursor + 2, &converter)?
    } else {
        (min, cursor)
    };

    if cursor == start || (min.is_none() && max.is_none()) {
        Err(ParseError {
            kind: ParseErrorKind::Empty,
            cursor: start,
        })
    } else {
        Ok(BoundsModel::new(min, max))
    }
}

fn read_number<T, E>(
    input: &str,
    mut cursor: usize,
    converter: &impl Fn(&str) -> Result<T, E>,
) -> Result<(Option<T>, usize), ParseError>
where
    T: Copy,
    E: std::fmt::Debug,
{
    let start = cursor;
    while let Some(ch) = input[cursor..].chars().next() {
        if !is_allowed_input_char(input, cursor, ch) {
            break;
        }
        cursor += ch.len_utf8();
    }

    let number = &input[start..cursor];
    if number.is_empty() {
        return Ok((None, cursor));
    }

    converter(number)
        .map(|value| (Some(value), cursor))
        .map_err(|_| ParseError {
            kind: ParseErrorKind::InvalidNumber,
            cursor: start,
        })
}

fn is_allowed_input_char(input: &str, cursor: usize, ch: char) -> bool {
    if ch.is_ascii_digit() || ch == '-' {
        true
    } else {
        ch == '.' && !input[cursor..].starts_with("..")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_bounds_cover_any_factories_ranges_points_and_mapping() {
        assert!(BoundsModel::<i32>::any().is_any());
        assert_eq!(BoundsModel::exactly(4).min(), Some(4));
        assert_eq!(BoundsModel::exactly(4).max(), Some(4));
        assert_eq!(
            BoundsModel::between(2, 6).as_range(),
            RangeShape::Closed(2, 6)
        );
        assert_eq!(BoundsModel::at_least(2).as_range(), RangeShape::AtLeast(2));
        assert_eq!(BoundsModel::at_most(6).as_range(), RangeShape::AtMost(6));
        assert_eq!(BoundsModel::exactly(7).as_point(), Some(7));
        assert_eq!(BoundsModel::between(2, 6).as_point(), None);
        assert_eq!(
            BoundsModel::between(2, 6).map(i64::from),
            BoundsModel::between(2_i64, 6_i64)
        );
    }

    #[test]
    fn codec_validation_and_shape_match_java_bounds_codec() {
        assert!(BoundsModel::between(1, 3)
            .validate_swapped_bounds_in_codec()
            .is_ok());
        assert!(BoundsModel::between(3, 1)
            .validate_swapped_bounds_in_codec()
            .is_err());
        assert_eq!(BoundsModel::exactly(5).codec_shape(), CodecShape::Point(5));
        assert_eq!(
            BoundsModel::at_least(5).codec_shape(),
            CodecShape::Range(BoundsModel::at_least(5))
        );
    }

    #[test]
    fn contained_range_validation_matches_guava_range_encloses_behavior() {
        let allowed = BoundsModel::between(0, 10);

        assert!(allowed.contains_bounds(&BoundsModel::between(2, 8)));
        assert!(allowed.contains_bounds(&BoundsModel::exactly(10)));
        assert!(!allowed.contains_bounds(&BoundsModel::at_least(2)));
        assert!(!allowed.contains_bounds(&BoundsModel::at_most(8)));
        assert!(!allowed.contains_bounds(&BoundsModel::between(-1, 8)));
        assert!(!allowed.contains_bounds(&BoundsModel::between(2, 11)));
    }

    #[test]
    fn stream_codec_flags_encode_optional_min_and_max() {
        let exact = BoundsModel::exactly(5).stream_encode();
        let lower = BoundsModel::at_least(5).stream_encode();
        let upper = BoundsModel::at_most(5).stream_encode();
        let any = BoundsModel::<i32>::any().stream_encode();

        assert_eq!(exact.flags, 3);
        assert_eq!(lower.flags, 1);
        assert_eq!(upper.flags, 2);
        assert_eq!(any.flags, 0);
        assert_eq!(BoundsModel::stream_decode(exact), BoundsModel::exactly(5));
    }

    #[test]
    fn command_reader_parses_points_and_open_ranges() {
        assert_eq!(
            IntsBoundsModel::from_reader("4").unwrap().bounds(),
            BoundsModel::exactly(4)
        );
        assert_eq!(
            IntsBoundsModel::from_reader("4..8").unwrap().bounds(),
            BoundsModel::between(4, 8)
        );
        assert_eq!(
            IntsBoundsModel::from_reader("4..").unwrap().bounds(),
            BoundsModel::at_least(4)
        );
        assert_eq!(
            IntsBoundsModel::from_reader("..8").unwrap().bounds(),
            BoundsModel::at_most(8)
        );
        assert_eq!(
            DoublesBoundsModel::from_reader("-1.5..2.25")
                .unwrap()
                .bounds(),
            BoundsModel::between(-1.5, 2.25)
        );
    }

    #[test]
    fn command_reader_reports_empty_invalid_and_swapped_inputs() {
        assert_eq!(
            IntsBoundsModel::from_reader("").unwrap_err().kind,
            ParseErrorKind::Empty
        );
        assert_eq!(
            IntsBoundsModel::from_reader("..").unwrap_err().kind,
            ParseErrorKind::Empty
        );
        assert_eq!(
            IntsBoundsModel::from_reader("x").unwrap_err().kind,
            ParseErrorKind::Empty
        );
        assert_eq!(
            IntsBoundsModel::from_reader("1.2").unwrap_err().kind,
            ParseErrorKind::InvalidNumber
        );
        assert_eq!(
            IntsBoundsModel::from_reader("8..4").unwrap_err().kind,
            ParseErrorKind::Swapped
        );
    }

    #[test]
    fn doubles_match_values_and_precomputed_squared_bounds() {
        let ranged = DoublesBoundsModel::between(2.0, 4.0);

        assert!(DoublesBoundsModel::ANY.matches(-100.0));
        assert!(DoublesBoundsModel::exactly(3.0).matches(3.0));
        assert!(DoublesBoundsModel::at_least(3.0).matches(4.0));
        assert!(DoublesBoundsModel::at_most(3.0).matches(2.0));
        assert!(ranged.matches(3.0));
        assert!(!ranged.matches(5.0));
        assert!(ranged.matches_sqr(9.0));
        assert!(!ranged.matches_sqr(3.0));
        assert!(!ranged.matches_sqr(17.0));
    }

    #[test]
    fn ints_match_values_and_precomputed_squared_long_bounds() {
        let ranged = IntsBoundsModel::between(2, 4);

        assert!(IntsBoundsModel::ANY.matches(-100));
        assert!(IntsBoundsModel::exactly(3).matches(3));
        assert!(IntsBoundsModel::at_least(3).matches(4));
        assert!(IntsBoundsModel::at_most(3).matches(2));
        assert!(ranged.matches(3));
        assert!(!ranged.matches(5));
        assert!(ranged.matches_sqr(9));
        assert!(!ranged.matches_sqr(3));
        assert!(!ranged.matches_sqr(17));
    }

    #[test]
    fn float_degrees_reader_preserves_java_lack_of_swapped_validation() {
        assert_eq!(
            FloatDegreesBoundsModel::from_reader("10.0..-10.0")
                .unwrap()
                .bounds(),
            BoundsModel::between(10.0, -10.0)
        );
        assert_eq!(FloatDegreesBoundsModel::ANY.bounds(), BoundsModel::any());
    }
}
