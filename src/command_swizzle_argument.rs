use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DirectionAxisModel {
    X,
    Y,
    Z,
}

impl DirectionAxisModel {
    fn parse(value: char) -> Option<Self> {
        match value {
            'x' => Some(Self::X),
            'y' => Some(Self::Y),
            'z' => Some(Self::Z),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwizzleArgumentModel;

impl SwizzleArgumentModel {
    pub fn swizzle() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<BTreeSet<DirectionAxisModel>, SwizzleParseError> {
        let mut result = BTreeSet::new();
        while reader.can_read() && reader.peek() != ' ' {
            let cursor_before = reader.cursor();
            let axis = reader
                .read()
                .and_then(DirectionAxisModel::parse)
                .ok_or_else(|| SwizzleParseError::Invalid {
                    cursor: reader.cursor(),
                })?;
            if result.contains(&axis) {
                return Err(SwizzleParseError::Invalid {
                    cursor: reader.cursor(),
                });
            }
            debug_assert_eq!(reader.cursor(), cursor_before + 1);
            result.insert(axis);
        }
        Ok(result)
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["xyz", "x"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, BTreeSet<DirectionAxisModel>>,
}

impl CommandContextModel {
    pub fn with_swizzle(
        mut self,
        name: impl Into<String>,
        value: BTreeSet<DirectionAxisModel>,
    ) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_swizzle(
    context: &CommandContextModel,
    name: &str,
) -> Option<BTreeSet<DirectionAxisModel>> {
    context.arguments.get(name).cloned()
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

    fn read(&mut self) -> Option<char> {
        if self.can_read() {
            let result = self.peek();
            self.cursor += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwizzleParseError {
    Invalid { cursor: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axes(values: &[DirectionAxisModel]) -> BTreeSet<DirectionAxisModel> {
        values.iter().copied().collect()
    }

    fn parse(input: &str) -> Result<(BTreeSet<DirectionAxisModel>, usize), SwizzleParseError> {
        let mut reader = StringReaderModel::new(input);
        let result = SwizzleArgumentModel::swizzle().parse(&mut reader)?;
        Ok((result, reader.cursor()))
    }

    #[test]
    fn java_factory_and_examples_match_source() {
        assert_eq!(SwizzleArgumentModel::swizzle().examples(), ["xyz", "x"]);
    }

    #[test]
    fn java_get_swizzle_returns_typed_context_argument() {
        let value = axes(&[DirectionAxisModel::X, DirectionAxisModel::Z]);
        let context = CommandContextModel::default().with_swizzle("axes", value.clone());

        assert_eq!(get_swizzle(&context, "axes"), Some(value));
        assert_eq!(get_swizzle(&context, "missing"), None);
    }

    #[test]
    fn java_parse_accepts_unique_axes_until_literal_space() {
        assert_eq!(
            parse("xyz rest"),
            Ok((
                axes(&[
                    DirectionAxisModel::X,
                    DirectionAxisModel::Y,
                    DirectionAxisModel::Z
                ]),
                3
            ))
        );
        assert_eq!(
            parse("zyx"),
            Ok((
                axes(&[
                    DirectionAxisModel::X,
                    DirectionAxisModel::Y,
                    DirectionAxisModel::Z
                ]),
                3
            ))
        );
        assert_eq!(parse("x tail"), Ok((axes(&[DirectionAxisModel::X]), 1)));
    }

    #[test]
    fn java_parse_allows_empty_swizzle_when_no_characters_are_read() {
        assert_eq!(parse(""), Ok((BTreeSet::new(), 0)));
        assert_eq!(parse(" tail"), Ok((BTreeSet::new(), 0)));
    }

    #[test]
    fn java_parse_rejects_invalid_character_after_consuming_it() {
        assert_eq!(parse("w"), Err(SwizzleParseError::Invalid { cursor: 1 }));
        assert_eq!(parse("xw"), Err(SwizzleParseError::Invalid { cursor: 2 }));
        assert_eq!(parse("x\tz"), Err(SwizzleParseError::Invalid { cursor: 2 }));
    }

    #[test]
    fn java_parse_rejects_duplicate_axis_after_consuming_duplicate() {
        assert_eq!(parse("xx"), Err(SwizzleParseError::Invalid { cursor: 2 }));
        assert_eq!(parse("zyz"), Err(SwizzleParseError::Invalid { cursor: 3 }));
    }
}
