use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleArgumentModel;

impl StyleArgumentModel {
    pub fn style(_context: &CommandBuildContextModel) -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<StyleModel, StyleParseError> {
        let cursor = reader.cursor();
        let tag = parse_compound(reader).map_err(StyleParseError::TagParser)?;
        decode_style(tag).map_err(|message| {
            reader.set_cursor(cursor);
            StyleParseError::InvalidStyle(message)
        })
    }

    pub fn list_suggestions(&self, _remaining: &str) -> Vec<String> {
        Vec::new()
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["{bold: true}", "{color: 'red'}", "{}"]
    }
}

pub fn get_style(context: &CommandContextModel, name: &str) -> Option<StyleModel> {
    context.arguments.get(name).cloned()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    arguments: HashMap<String, StyleModel>,
}

impl CommandContextModel {
    pub fn with_style(mut self, name: impl Into<String>, style: StyleModel) -> Self {
        self.arguments.insert(name.into(), style);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleModel {
    color: Option<String>,
    shadow_color: Option<i32>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    click_event: Option<String>,
    hover_event: Option<String>,
    insertion: Option<String>,
    font: Option<String>,
}

impl StyleModel {
    const DEFAULT_FONT: &'static str = "minecraft:default";

    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn shadow_color(&self) -> Option<i32> {
        self.shadow_color
    }

    pub fn is_bold(&self) -> bool {
        self.bold == Some(true)
    }

    pub fn is_italic(&self) -> bool {
        self.italic == Some(true)
    }

    pub fn is_underlined(&self) -> bool {
        self.underlined == Some(true)
    }

    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough == Some(true)
    }

    pub fn is_obfuscated(&self) -> bool {
        self.obfuscated == Some(true)
    }

    pub fn insertion(&self) -> Option<&str> {
        self.insertion.as_deref()
    }

    pub fn font(&self) -> &str {
        self.font.as_deref().unwrap_or(Self::DEFAULT_FONT)
    }

    pub fn click_event(&self) -> Option<&str> {
        self.click_event.as_deref()
    }

    pub fn hover_event(&self) -> Option<&str> {
        self.hover_event.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleParseError {
    TagParser(TagParseError),
    InvalidStyle(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParseError {
    ExpectedOpeningBrace,
    UnclosedCompound,
    InvalidKey,
    ExpectedColon,
    InvalidValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagValueModel {
    Bool(bool),
    String(String),
    Int(i32),
    Compound(HashMap<String, TagValueModel>),
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

fn decode_style(fields: HashMap<String, TagValueModel>) -> Result<StyleModel, String> {
    let mut style = StyleModel::default();

    for (key, value) in fields {
        match key.as_str() {
            "color" => style.color = Some(decode_color(value)?),
            "shadow_color" => style.shadow_color = Some(decode_int(value, "shadow_color")?),
            "bold" => style.bold = Some(decode_bool(value, "bold")?),
            "italic" => style.italic = Some(decode_bool(value, "italic")?),
            "underlined" => style.underlined = Some(decode_bool(value, "underlined")?),
            "strikethrough" => style.strikethrough = Some(decode_bool(value, "strikethrough")?),
            "obfuscated" => style.obfuscated = Some(decode_bool(value, "obfuscated")?),
            "click_event" => {
                style.click_event = Some(decode_compound_summary(value, "click_event")?)
            }
            "hover_event" => {
                style.hover_event = Some(decode_compound_summary(value, "hover_event")?)
            }
            "insertion" => style.insertion = Some(decode_string(value, "insertion")?),
            "font" => style.font = Some(decode_string(value, "font")?),
            _unknown => {}
        }
    }

    Ok(style)
}

fn decode_color(value: TagValueModel) -> Result<String, String> {
    let color = decode_string(value, "color")?;
    if is_legacy_color(&color) || is_hex_color(&color) {
        Ok(color)
    } else {
        Err(format!("invalid color '{color}'"))
    }
}

fn is_legacy_color(value: &str) -> bool {
    matches!(
        value,
        "black"
            | "dark_blue"
            | "dark_green"
            | "dark_aqua"
            | "dark_red"
            | "dark_purple"
            | "gold"
            | "gray"
            | "dark_gray"
            | "blue"
            | "green"
            | "aqua"
            | "red"
            | "light_purple"
            | "yellow"
            | "white"
    )
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn decode_bool(value: TagValueModel, field: &str) -> Result<bool, String> {
    match value {
        TagValueModel::Bool(value) => Ok(value),
        other => Err(format!("{field} expected boolean, got {other:?}")),
    }
}

fn decode_int(value: TagValueModel, field: &str) -> Result<i32, String> {
    match value {
        TagValueModel::Int(value) => Ok(value),
        other => Err(format!("{field} expected int, got {other:?}")),
    }
}

fn decode_string(value: TagValueModel, field: &str) -> Result<String, String> {
    match value {
        TagValueModel::String(value) => Ok(value),
        other => Err(format!("{field} expected string, got {other:?}")),
    }
}

fn decode_compound_summary(value: TagValueModel, field: &str) -> Result<String, String> {
    match value {
        TagValueModel::Compound(fields) => Ok(format!("{field}:{}", fields.len())),
        other => Err(format!("{field} expected compound, got {other:?}")),
    }
}

fn parse_compound(
    reader: &mut StringReaderModel,
) -> Result<HashMap<String, TagValueModel>, TagParseError> {
    reader.skip_whitespace();
    if !reader.can_read() || reader.peek() != '{' {
        return Err(TagParseError::ExpectedOpeningBrace);
    }
    reader.skip();

    let mut fields = HashMap::new();
    loop {
        reader.skip_whitespace();
        if !reader.can_read() {
            return Err(TagParseError::UnclosedCompound);
        }
        if reader.peek() == '}' {
            reader.skip();
            return Ok(fields);
        }

        let key = parse_key(reader)?;
        reader.skip_whitespace();
        if !reader.can_read() || reader.peek() != ':' {
            return Err(TagParseError::ExpectedColon);
        }
        reader.skip();
        reader.skip_whitespace();
        let value = parse_value(reader)?;
        fields.insert(key, value);

        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
}

fn parse_key(reader: &mut StringReaderModel) -> Result<String, TagParseError> {
    if !reader.can_read() {
        return Err(TagParseError::InvalidKey);
    }
    if matches!(reader.peek(), '\'' | '"') {
        return parse_quoted_string(reader);
    }

    let start = reader.cursor();
    while reader.can_read() && is_allowed_key_char(reader.peek()) {
        reader.skip();
    }
    if reader.cursor() == start {
        return Err(TagParseError::InvalidKey);
    }
    Ok(reader.slice(start, reader.cursor()).to_string())
}

fn parse_value(reader: &mut StringReaderModel) -> Result<TagValueModel, TagParseError> {
    if !reader.can_read() {
        return Err(TagParseError::InvalidValue);
    }
    match reader.peek() {
        '{' => parse_compound(reader).map(TagValueModel::Compound),
        '\'' | '"' => parse_quoted_string(reader).map(TagValueModel::String),
        '-' | '0'..='9' => parse_int(reader).map(TagValueModel::Int),
        _ => parse_unquoted_value(reader),
    }
}

fn parse_quoted_string(reader: &mut StringReaderModel) -> Result<String, TagParseError> {
    let quote = reader.peek();
    reader.skip();
    let start = reader.cursor();
    while reader.can_read() && reader.peek() != quote {
        reader.skip();
    }
    if !reader.can_read() {
        return Err(TagParseError::InvalidValue);
    }
    let value = reader.slice(start, reader.cursor()).to_string();
    reader.skip();
    Ok(value)
}

fn parse_int(reader: &mut StringReaderModel) -> Result<i32, TagParseError> {
    let start = reader.cursor();
    if reader.peek() == '-' {
        reader.skip();
    }
    while reader.can_read() && reader.peek().is_ascii_digit() {
        reader.skip();
    }
    reader
        .slice(start, reader.cursor())
        .parse()
        .map_err(|_error| TagParseError::InvalidValue)
}

fn parse_unquoted_value(reader: &mut StringReaderModel) -> Result<TagValueModel, TagParseError> {
    let start = reader.cursor();
    while reader.can_read() && !matches!(reader.peek(), ',' | '}' | ' ' | '\t' | '\n' | '\r') {
        reader.skip();
    }
    if reader.cursor() == start {
        return Err(TagParseError::InvalidValue);
    }
    match reader.slice(start, reader.cursor()) {
        "true" => Ok(TagValueModel::Bool(true)),
        "false" => Ok(TagValueModel::Bool(false)),
        value => Ok(TagValueModel::String(value.to_string())),
    }
}

fn is_allowed_key_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument() -> StyleArgumentModel {
        StyleArgumentModel::style(&CommandBuildContextModel)
    }

    fn parse(input: &str) -> Result<(StyleModel, usize), StyleParseError> {
        let mut reader = StringReaderModel::new(input);
        let style = argument().parse(&mut reader)?;
        Ok((style, reader.cursor()))
    }

    #[test]
    fn java_factory_examples_and_context_getter_match_source() {
        let argument = argument();
        assert_eq!(
            argument.examples(),
            ["{bold: true}", "{color: 'red'}", "{}"]
        );

        let style = parse("{bold: true}").unwrap().0;
        let context = CommandContextModel::default().with_style("style", style.clone());
        assert_eq!(get_style(&context, "style"), Some(style));
        assert_eq!(get_style(&context, "missing"), None);
    }

    #[test]
    fn java_parse_empty_compound_returns_canonical_empty_style() {
        let (style, cursor) = parse("{} trailing").unwrap();

        assert!(style.is_empty());
        assert_eq!(style.font(), "minecraft:default");
        assert_eq!(cursor, "{}".len());
    }

    #[test]
    fn java_parse_decodes_style_serializer_fields() {
        let input = "{color:'red',shadow_color:-1,bold:true,italic:false,underlined:true,strikethrough:true,obfuscated:false,insertion:'copy',font:'minecraft:uniform'}";
        let (style, cursor) = parse(input).unwrap();

        assert_eq!(style.color(), Some("red"));
        assert_eq!(style.shadow_color(), Some(-1));
        assert!(style.is_bold());
        assert!(!style.is_italic());
        assert!(style.is_underlined());
        assert!(style.is_strikethrough());
        assert!(!style.is_obfuscated());
        assert_eq!(style.insertion(), Some("copy"));
        assert_eq!(style.font(), "minecraft:uniform");
        assert_eq!(cursor, input.len());
    }

    #[test]
    fn java_parse_accepts_nested_click_and_hover_event_compounds() {
        let input = "{click_event:{action:'open_url',url:'https://example.invalid'},hover_event:{action:'show_text'}}";
        let (style, cursor) = parse(input).unwrap();

        assert_eq!(style.click_event(), Some("click_event:2"));
        assert_eq!(style.hover_event(), Some("hover_event:1"));
        assert_eq!(cursor, input.len());
    }

    #[test]
    fn java_codec_error_resets_cursor_and_uses_invalid_style_error() {
        let mut reader = StringReaderModel::new("{bold:'yes'} trailing");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(StyleParseError::InvalidStyle(
                "bold expected boolean, got String(\"yes\")".to_string()
            ))
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_invalid_color_is_codec_error_and_resets_cursor() {
        let mut reader = StringReaderModel::new("{color:'not_a_color'}");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(StyleParseError::InvalidStyle(
                "invalid color 'not_a_color'".to_string()
            ))
        );
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn java_tag_parser_error_is_not_wrapped_as_style_codec_error() {
        let mut reader = StringReaderModel::new("{bold:true");
        let error = argument().parse(&mut reader);

        assert_eq!(
            error,
            Err(StyleParseError::TagParser(TagParseError::UnclosedCompound))
        );
        assert_eq!(reader.cursor(), "{bold:true".len());
    }

    #[test]
    fn java_suggestions_delegate_to_underlying_parser() {
        assert_eq!(argument().list_suggestions("{bo"), Vec::<String>::new());
    }
}
