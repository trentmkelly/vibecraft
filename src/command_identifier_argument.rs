use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierArgumentModel;

impl IdentifierArgumentModel {
    pub fn id() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<CommandIdentifierModel, IdentifierArgumentParseError> {
        CommandIdentifierModel::read(reader)
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["foo", "foo:bar", "012"]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, CommandIdentifierModel>,
}

impl CommandContextModel {
    pub fn with_identifier(
        mut self,
        name: impl Into<String>,
        value: CommandIdentifierModel,
    ) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }
}

pub fn get_id(context: &CommandContextModel, name: &str) -> Option<CommandIdentifierModel> {
    context.arguments.get(name).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandIdentifierModel {
    namespace: String,
    path: String,
}

impl CommandIdentifierModel {
    const DEFAULT_NAMESPACE: &'static str = "minecraft";

    pub fn parse(value: &str) -> Result<Self, IdentifierArgumentParseError> {
        if let Some(separator_index) = value.find(':') {
            let path = &value[separator_index + 1..];
            if separator_index == 0 {
                Self::with_default_namespace(path)
            } else {
                Self::from_namespace_and_path(&value[..separator_index], path)
            }
        } else {
            Self::with_default_namespace(value)
        }
    }

    pub fn read(reader: &mut StringReaderModel) -> Result<Self, IdentifierArgumentParseError> {
        let start = reader.cursor();
        let raw = Self::read_greedy(reader);
        match Self::parse(&raw) {
            Ok(identifier) => Ok(identifier),
            Err(_error) => {
                reader.set_cursor(start);
                Err(IdentifierArgumentParseError::InvalidIdentifier)
            }
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn to_short_string(&self) -> String {
        if self.namespace == Self::DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_string()
        }
    }

    fn with_default_namespace(path: &str) -> Result<Self, IdentifierArgumentParseError> {
        Self::from_namespace_and_path(Self::DEFAULT_NAMESPACE, path)
    }

    fn from_namespace_and_path(
        namespace: &str,
        path: &str,
    ) -> Result<Self, IdentifierArgumentParseError> {
        if !Self::is_valid_namespace(namespace) || !Self::is_valid_path(path) {
            return Err(IdentifierArgumentParseError::InvalidIdentifier);
        }
        Ok(Self {
            namespace: namespace.to_string(),
            path: path.to_string(),
        })
    }

    fn read_greedy(reader: &mut StringReaderModel) -> String {
        let start = reader.cursor();
        while reader.can_read() && Self::is_allowed_in_identifier(reader.peek()) {
            reader.skip();
        }
        reader.slice(start, reader.cursor()).to_string()
    }

    fn is_allowed_in_identifier(value: char) -> bool {
        value.is_ascii_digit()
            || value.is_ascii_lowercase()
            || matches!(value, '_' | ':' | '/' | '.' | '-')
    }

    fn is_valid_namespace(namespace: &str) -> bool {
        namespace != ".."
            && namespace.chars().all(|value| {
                value.is_ascii_digit()
                    || value.is_ascii_lowercase()
                    || matches!(value, '_' | '.' | '-')
            })
    }

    fn is_valid_path(path: &str) -> bool {
        path.chars().all(|value| {
            value.is_ascii_digit()
                || value.is_ascii_lowercase()
                || matches!(value, '_' | '-' | '/' | '.')
        })
    }
}

impl fmt::Display for CommandIdentifierModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierArgumentParseError {
    InvalidIdentifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(CommandIdentifierModel, usize), IdentifierArgumentParseError> {
        let mut reader = StringReaderModel::new(input);
        let identifier = IdentifierArgumentModel::id().parse(&mut reader)?;
        Ok((identifier, reader.cursor()))
    }

    #[test]
    fn java_identifier_argument_factory_examples_and_context_lookup_match_source() {
        let argument = IdentifierArgumentModel::id();
        assert_eq!(argument.examples(), ["foo", "foo:bar", "012"]);

        let value = CommandIdentifierModel::parse("custom:thing").unwrap();
        let context = CommandContextModel::default().with_identifier("target", value.clone());
        assert_eq!(get_id(&context, "target"), Some(value));
        assert_eq!(get_id(&context, "missing"), None);
    }

    #[test]
    fn java_parse_defaults_namespace_without_colon() {
        let (identifier, cursor) = parse("foo trailing").unwrap();

        assert_eq!(identifier.namespace(), "minecraft");
        assert_eq!(identifier.path(), "foo");
        assert_eq!(identifier.to_string(), "minecraft:foo");
        assert_eq!(identifier.to_short_string(), "foo");
        assert_eq!(cursor, "foo".len());
    }

    #[test]
    fn java_parse_preserves_explicit_namespace_and_complex_path() {
        let input = "foo:bar/baz.012-_ rest";
        let (identifier, cursor) = parse(input).unwrap();

        assert_eq!(identifier.namespace(), "foo");
        assert_eq!(identifier.path(), "bar/baz.012-_");
        assert_eq!(identifier.to_string(), "foo:bar/baz.012-_");
        assert_eq!(identifier.to_short_string(), "foo:bar/baz.012-_");
        assert_eq!(cursor, "foo:bar/baz.012-_".len());
    }

    #[test]
    fn java_parse_colon_at_start_uses_default_namespace() {
        let (identifier, cursor) = parse(":bar").unwrap();

        assert_eq!(identifier.to_string(), "minecraft:bar");
        assert_eq!(identifier.to_short_string(), "bar");
        assert_eq!(cursor, ":bar".len());
    }

    #[test]
    fn java_parse_allows_empty_path_from_empty_or_trailing_colon() {
        let (empty_identifier, empty_cursor) = parse("").unwrap();
        assert_eq!(empty_identifier.to_string(), "minecraft:");
        assert_eq!(empty_identifier.to_short_string(), "");
        assert_eq!(empty_cursor, 0);

        let (colon_identifier, colon_cursor) = parse("foo:").unwrap();
        assert_eq!(colon_identifier.to_string(), "foo:");
        assert_eq!(colon_cursor, "foo:".len());
    }

    #[test]
    fn java_read_greedy_stops_before_disallowed_characters() {
        let (identifier, cursor) = parse("foo@bar").unwrap();

        assert_eq!(identifier.to_string(), "minecraft:foo");
        assert_eq!(cursor, "foo".len());
    }

    #[test]
    fn java_uppercase_at_start_consumes_nothing_and_parses_empty_default_path() {
        let (identifier, cursor) = parse("Foo").unwrap();

        assert_eq!(identifier.to_string(), "minecraft:");
        assert_eq!(cursor, 0);
    }

    #[test]
    fn java_invalid_namespace_resets_cursor() {
        let mut reader = StringReaderModel::new("..:path next");
        let error = IdentifierArgumentModel::id().parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_invalid_path_resets_cursor() {
        let mut reader = StringReaderModel::new("foo::bar next");
        let error = IdentifierArgumentModel::id().parse(&mut reader);

        assert_eq!(error, Err(IdentifierArgumentParseError::InvalidIdentifier));
        assert_eq!(reader.cursor(), 0);
    }
}
