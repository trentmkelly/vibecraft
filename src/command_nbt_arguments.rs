use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundTagArgumentModel;

impl CompoundTagArgumentModel {
    pub fn compound_tag() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<CompoundTagModel, TagParseError> {
        parse_compound_as_argument(reader)
    }

    pub fn examples(&self) -> [&'static str; 2] {
        ["{}", "{foo=bar}"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtTagArgumentModel;

impl NbtTagArgumentModel {
    pub fn nbt_tag() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<TagModel, TagParseError> {
        parse_tag(reader)
    }

    pub fn list_suggestions(&self, _remaining: &str) -> Vec<String> {
        Vec::new()
    }

    pub fn examples(&self) -> [&'static str; 7] {
        ["0", "0b", "0l", "0.0", "\"foo\"", "{foo=bar}", "[0]"]
    }
}

pub fn get_compound_tag(context: &CommandContextModel, name: &str) -> Option<CompoundTagModel> {
    context.compounds.get(name).cloned()
}

pub fn get_nbt_tag(context: &CommandContextModel, name: &str) -> Option<TagModel> {
    context.tags.get(name).cloned()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    compounds: HashMap<String, CompoundTagModel>,
    tags: HashMap<String, TagModel>,
}

impl CommandContextModel {
    pub fn with_compound(mut self, name: impl Into<String>, value: CompoundTagModel) -> Self {
        self.compounds.insert(name.into(), value);
        self
    }

    pub fn with_tag(mut self, name: impl Into<String>, value: TagModel) -> Self {
        self.tags.insert(name.into(), value);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompoundTagModel {
    fields: Vec<(String, TagModel)>,
}

impl CompoundTagModel {
    pub fn get(&self, name: &str) -> Option<&TagModel> {
        self.fields
            .iter()
            .find(|(key, _value)| key == name)
            .map(|(_key, value)| value)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagModel {
    Byte(i8),
    Int(i32),
    Long(i64),
    Double(f64),
    String(String),
    Compound(CompoundTagModel),
    List(Vec<TagModel>),
}

impl Eq for TagModel {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParseError {
    ExpectedCompound,
    ExpectedKey,
    ExpectedSeparator,
    ExpectedValue,
    UnclosedCompound,
    UnclosedList,
    InvalidNumber,
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

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.input[start..end]
    }

    fn skip_whitespace(&mut self) {
        while self.can_read() && self.peek().is_ascii_whitespace() {
            self.skip();
        }
    }
}

fn parse_compound_as_argument(
    reader: &mut StringReaderModel,
) -> Result<CompoundTagModel, TagParseError> {
    let start = reader.cursor();
    match parse_compound(reader) {
        Ok(value) => Ok(value),
        Err(error) => {
            if matches!(error, TagParseError::ExpectedCompound) {
                reader.set_cursor(start);
            }
            Err(error)
        }
    }
}

fn parse_tag(reader: &mut StringReaderModel) -> Result<TagModel, TagParseError> {
    reader.skip_whitespace();
    if !reader.can_read() {
        return Err(TagParseError::ExpectedValue);
    }

    match reader.peek() {
        '{' => parse_compound(reader).map(TagModel::Compound),
        '[' => parse_list(reader).map(TagModel::List),
        '"' | '\'' => parse_quoted_string(reader).map(TagModel::String),
        _ => parse_unquoted_tag(reader),
    }
}

fn parse_compound(reader: &mut StringReaderModel) -> Result<CompoundTagModel, TagParseError> {
    reader.skip_whitespace();
    if !reader.can_read() || reader.peek() != '{' {
        return Err(TagParseError::ExpectedCompound);
    }
    reader.skip();

    let mut fields = Vec::new();
    loop {
        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(TagParseError::UnclosedCompound);
        }
        if reader.peek() == '}' {
            reader.skip();
            return Ok(CompoundTagModel { fields });
        }

        let key = parse_key(reader)?;
        reader.skip_whitespace();
        if !reader.can_read() || !matches!(reader.peek(), ':' | '=') {
            return Err(TagParseError::ExpectedSeparator);
        }
        reader.skip();
        let value = parse_tag(reader)?;
        fields.push((key, value));

        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
}

fn parse_list(reader: &mut StringReaderModel) -> Result<Vec<TagModel>, TagParseError> {
    reader.skip_whitespace();
    if !reader.can_read() || reader.peek() != '[' {
        return Err(TagParseError::ExpectedValue);
    }
    reader.skip();

    let mut values = Vec::new();
    loop {
        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(TagParseError::UnclosedList);
        }
        if reader.peek() == ']' {
            reader.skip();
            return Ok(values);
        }

        values.push(parse_tag(reader)?);
        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
}

fn parse_key(reader: &mut StringReaderModel) -> Result<String, TagParseError> {
    reader.skip_whitespace();
    if !reader.can_read() {
        return Err(TagParseError::ExpectedKey);
    }
    if matches!(reader.peek(), '"' | '\'') {
        return parse_quoted_string(reader);
    }

    let start = reader.cursor();
    while reader.can_read() && is_allowed_unquoted(reader.peek()) {
        reader.skip();
    }
    if reader.cursor() == start {
        return Err(TagParseError::ExpectedKey);
    }
    Ok(reader.slice(start, reader.cursor()).to_string())
}

fn parse_quoted_string(reader: &mut StringReaderModel) -> Result<String, TagParseError> {
    let quote = reader.peek();
    reader.skip();
    let start = reader.cursor();
    while reader.can_read() && reader.peek() != quote {
        reader.skip();
    }
    if !reader.can_read() {
        return Err(TagParseError::ExpectedValue);
    }
    let value = reader.slice(start, reader.cursor()).to_string();
    reader.skip();
    Ok(value)
}

fn parse_unquoted_tag(reader: &mut StringReaderModel) -> Result<TagModel, TagParseError> {
    let start = reader.cursor();
    while reader.can_read() && !matches!(reader.peek(), ',' | '}' | ']' | ' ' | '\t' | '\n' | '\r')
    {
        reader.skip();
    }
    if reader.cursor() == start {
        return Err(TagParseError::ExpectedValue);
    }

    let raw = reader.slice(start, reader.cursor());
    if let Some(number) = raw.strip_suffix(['b', 'B']) {
        return number
            .parse::<i8>()
            .map(TagModel::Byte)
            .map_err(|_error| TagParseError::InvalidNumber);
    }
    if let Some(number) = raw.strip_suffix(['l', 'L']) {
        return number
            .parse::<i64>()
            .map(TagModel::Long)
            .map_err(|_error| TagParseError::InvalidNumber);
    }
    if raw.contains('.') {
        return raw
            .parse::<f64>()
            .map(TagModel::Double)
            .map_err(|_error| TagParseError::InvalidNumber);
    }
    if let Ok(value) = raw.parse::<i32>() {
        return Ok(TagModel::Int(value));
    }
    Ok(TagModel::String(raw.to_string()))
}

fn is_allowed_unquoted(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '+' | '/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_compound(input: &str) -> Result<(CompoundTagModel, usize), TagParseError> {
        let mut reader = StringReaderModel::new(input);
        let value = CompoundTagArgumentModel::compound_tag().parse(&mut reader)?;
        Ok((value, reader.cursor()))
    }

    fn parse_tag_value(input: &str) -> Result<(TagModel, usize), TagParseError> {
        let mut reader = StringReaderModel::new(input);
        let value = NbtTagArgumentModel::nbt_tag().parse(&mut reader)?;
        Ok((value, reader.cursor()))
    }

    #[test]
    fn java_compound_tag_factory_examples_and_context_getter_match_source() {
        let argument = CompoundTagArgumentModel::compound_tag();
        assert_eq!(argument.examples(), ["{}", "{foo=bar}"]);

        let compound = parse_compound("{foo=bar}").unwrap().0;
        let context = CommandContextModel::default().with_compound("tag", compound.clone());

        assert_eq!(get_compound_tag(&context, "tag"), Some(compound));
        assert_eq!(get_compound_tag(&context, "missing"), None);
    }

    #[test]
    fn java_compound_tag_parse_delegates_to_compound_parser() {
        let (empty, empty_cursor) = parse_compound("{} trailing").unwrap();
        assert!(empty.is_empty());
        assert_eq!(empty_cursor, "{}".len());

        let (compound, cursor) = parse_compound("{foo=bar,answer:42} rest").unwrap();
        assert_eq!(compound.len(), 2);
        assert_eq!(
            compound.get("foo"),
            Some(&TagModel::String("bar".to_string()))
        );
        assert_eq!(compound.get("answer"), Some(&TagModel::Int(42)));
        assert_eq!(cursor, "{foo=bar,answer:42}".len());
    }

    #[test]
    fn java_compound_tag_rejects_non_compound_without_consuming_start() {
        let mut reader = StringReaderModel::new("[0]");
        let error = CompoundTagArgumentModel::compound_tag().parse(&mut reader);

        assert_eq!(error, Err(TagParseError::ExpectedCompound));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_nbt_tag_factory_examples_and_context_getter_match_source() {
        let argument = NbtTagArgumentModel::nbt_tag();
        assert_eq!(
            argument.examples(),
            ["0", "0b", "0l", "0.0", "\"foo\"", "{foo=bar}", "[0]"]
        );

        let tag = parse_tag_value("\"foo\"").unwrap().0;
        let context = CommandContextModel::default().with_tag("tag", tag.clone());
        assert_eq!(get_nbt_tag(&context, "tag"), Some(tag));
        assert_eq!(get_nbt_tag(&context, "missing"), None);
    }

    #[test]
    fn java_nbt_tag_parse_covers_documented_examples() {
        assert_eq!(parse_tag_value("0").unwrap(), (TagModel::Int(0), 1));
        assert_eq!(parse_tag_value("0b").unwrap(), (TagModel::Byte(0), 2));
        assert_eq!(parse_tag_value("0l").unwrap(), (TagModel::Long(0), 2));
        assert_eq!(parse_tag_value("0.0").unwrap(), (TagModel::Double(0.0), 3));
        assert_eq!(
            parse_tag_value("\"foo\"").unwrap(),
            (TagModel::String("foo".to_string()), 5)
        );

        let compound = parse_tag_value("{foo=bar} trailing").unwrap();
        assert!(matches!(compound.0, TagModel::Compound(_)));
        assert_eq!(compound.1, "{foo=bar}".len());

        assert_eq!(
            parse_tag_value("[0]").unwrap(),
            (TagModel::List(vec![TagModel::Int(0)]), 3)
        );
    }

    #[test]
    fn java_nbt_tag_inherits_parser_based_suggestion_delegation() {
        assert_eq!(
            NbtTagArgumentModel::nbt_tag().list_suggestions("{fo"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn java_nbt_parser_errors_preserve_consumed_cursor_from_parser() {
        let mut reader = StringReaderModel::new("{foo=bar");
        let error = NbtTagArgumentModel::nbt_tag().parse(&mut reader);

        assert_eq!(error, Err(TagParseError::UnclosedCompound));
        assert_eq!(reader.cursor(), "{foo=bar".len());
    }
}
