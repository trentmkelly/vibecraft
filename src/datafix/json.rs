//! Minimal Gson-compatible JSON handling for the text component fixes.
//!
//! `LegacyComponentDataFixUtils` parses legacy text with Gson's *lenient*
//! `JsonParser.parseString` and re-serialises it with `GsonHelper.toStableString`
//! (object keys sorted, no whitespace, numbers keep their original text). This
//! module reproduces exactly those two behaviours.

/// A JSON element (Gson `JsonElement`).
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// `JsonNull`.
    Null,
    /// A boolean primitive.
    Bool(bool),
    /// A number primitive, kept as its lexical text (`LazilyParsedNumber`).
    Number(String),
    /// A string primitive.
    String(String),
    /// A `JsonArray`.
    Array(Vec<Json>),
    /// A `JsonObject`; duplicate names keep the last value.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// `JsonElement.getAsString()` for primitives.
    pub fn primitive_as_string(&self) -> Option<String> {
        match self {
            Json::Bool(value) => Some(value.to_string()),
            Json::Number(text) | Json::String(text) => Some(text.clone()),
            _ => None,
        }
    }
}

/// A Gson syntax error (`JsonSyntaxException` / `MalformedJsonException`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonSyntaxError;

/// `JsonParser.parseString`: lenient parsing of exactly one element.
pub fn parse_lenient(text: &str) -> Result<Json, JsonSyntaxError> {
    let mut reader = Reader {
        chars: text.chars().collect(),
        pos: 0,
    };
    reader.skip_non_execute_prefix();
    let element = reader.read_value()?;
    if element != Json::Null && reader.next_non_whitespace()?.is_some() {
        // "Did not consume the entire document."
        return Err(JsonSyntaxError);
    }
    Ok(element)
}

struct Reader {
    chars: Vec<char>,
    pos: usize,
}

/// `JsonReader.isLiteral` (lenient mode).
fn is_literal(c: char) -> bool {
    !matches!(
        c,
        '/' | '\\'
            | ';'
            | '#'
            | '='
            | '{'
            | '}'
            | '['
            | ']'
            | ':'
            | ','
            | ' '
            | '\t'
            | '\u{c}'
            | '\r'
            | '\n'
    )
}

impl Reader {
    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// `consumeNonExecutePrefix`: skips a leading `)]}'` line.
    fn skip_non_execute_prefix(&mut self) {
        // The reader first skips leading whitespace, then the prefix if present.
        let start = self.pos;
        let mut probe = self.pos;
        while matches!(self.chars.get(probe), Some(' ' | '\n' | '\t' | '\r')) {
            probe += 1;
        }
        let prefix: [char; 5] = [')', ']', '}', '\'', '\n'];
        if self.chars.len() >= probe + 5 && self.chars[probe..probe + 5] == prefix {
            self.pos = probe + 5;
        } else {
            self.pos = start;
        }
    }

    /// `nextNonWhitespace(throwOnEof = false)`: skips whitespace and comments,
    /// returning the next character without consuming it.
    fn next_non_whitespace(&mut self) -> Result<Option<char>, JsonSyntaxError> {
        loop {
            match self.peek_char() {
                None => return Ok(None),
                Some(' ' | '\n' | '\t' | '\r') => self.pos += 1,
                Some('/') => match self.chars.get(self.pos + 1) {
                    Some('*') => {
                        let mut index = self.pos + 2;
                        loop {
                            if index + 1 >= self.chars.len() {
                                return Err(JsonSyntaxError);
                            }
                            if self.chars[index] == '*' && self.chars[index + 1] == '/' {
                                break;
                            }
                            index += 1;
                        }
                        self.pos = index + 2;
                    }
                    Some('/') => self.skip_to_end_of_line(),
                    _ => return Ok(Some('/')),
                },
                Some('#') => self.skip_to_end_of_line(),
                Some(c) => return Ok(Some(c)),
            }
        }
    }

    fn skip_to_end_of_line(&mut self) {
        while let Some(c) = self.peek_char() {
            self.pos += 1;
            if c == '\n' || c == '\r' {
                break;
            }
        }
    }

    fn read_value(&mut self) -> Result<Json, JsonSyntaxError> {
        match self.next_non_whitespace()? {
            None => Err(JsonSyntaxError),
            Some('[') => {
                self.pos += 1;
                self.read_array()
            }
            Some('{') => {
                self.pos += 1;
                self.read_object()
            }
            Some(quote @ ('"' | '\'')) => {
                self.pos += 1;
                Ok(Json::String(self.read_quoted(quote)?))
            }
            Some(_) => self.read_unquoted_value(),
        }
    }

    fn read_array(&mut self) -> Result<Json, JsonSyntaxError> {
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let c = self.next_non_whitespace()?.ok_or(JsonSyntaxError)?;
            if !first {
                match c {
                    ']' => {
                        self.pos += 1;
                        return Ok(Json::Array(items));
                    }
                    ';' | ',' => self.pos += 1,
                    _ => return Err(JsonSyntaxError),
                }
            }
            let c = self.next_non_whitespace()?.ok_or(JsonSyntaxError)?;
            match c {
                ']' => {
                    self.pos += 1;
                    if first {
                        return Ok(Json::Array(items));
                    }
                    // `[1,]`: a trailing separator yields an implicit null.
                    items.push(Json::Null);
                    return Ok(Json::Array(items));
                }
                ';' | ',' => {
                    // A zero-length literal in an array is `null`.
                    items.push(Json::Null);
                }
                _ => items.push(self.read_value()?),
            }
            first = false;
        }
    }

    fn read_object(&mut self) -> Result<Json, JsonSyntaxError> {
        let mut members: Vec<(String, Json)> = Vec::new();
        let mut first = true;
        loop {
            let mut c = self.next_non_whitespace()?.ok_or(JsonSyntaxError)?;
            if !first {
                match c {
                    '}' => {
                        self.pos += 1;
                        return Ok(Json::Object(members));
                    }
                    ';' | ',' => {
                        self.pos += 1;
                        c = self.next_non_whitespace()?.ok_or(JsonSyntaxError)?;
                    }
                    _ => return Err(JsonSyntaxError),
                }
            } else if c == '}' {
                self.pos += 1;
                return Ok(Json::Object(members));
            }
            let name = match c {
                '"' | '\'' => {
                    self.pos += 1;
                    self.read_quoted(c)?
                }
                _ if is_literal(c) => self.read_unquoted_literal(),
                _ => return Err(JsonSyntaxError),
            };
            match self.next_non_whitespace()? {
                Some(':') => self.pos += 1,
                Some('=') => {
                    self.pos += 1;
                    if self.peek_char() == Some('>') {
                        self.pos += 1;
                    }
                }
                _ => return Err(JsonSyntaxError),
            }
            let value = self.read_value()?;
            match members.iter_mut().find(|(existing, _)| *existing == name) {
                Some((_, slot)) => *slot = value,
                None => members.push((name, value)),
            }
            first = false;
        }
    }

    /// Reads a quoted string; the opening quote is already consumed.
    fn read_quoted(&mut self, quote: char) -> Result<String, JsonSyntaxError> {
        let mut out = String::new();
        loop {
            let c = self.peek_char().ok_or(JsonSyntaxError)?;
            self.pos += 1;
            if c == quote {
                return Ok(out);
            }
            if c != '\\' {
                out.push(c);
                continue;
            }
            let escaped = self.peek_char().ok_or(JsonSyntaxError)?;
            self.pos += 1;
            match escaped {
                'u' => {
                    let hex: String = self
                        .chars
                        .get(self.pos..self.pos + 4)
                        .ok_or(JsonSyntaxError)?
                        .iter()
                        .collect();
                    let unit = u16::from_str_radix(&hex, 16).map_err(|_| JsonSyntaxError)?;
                    if hex.chars().any(|d| !d.is_ascii_hexdigit()) {
                        return Err(JsonSyntaxError);
                    }
                    self.pos += 4;
                    push_utf16_unit(&mut out, unit, &mut self.chars, &mut self.pos)?;
                }
                't' => out.push('\t'),
                'b' => out.push('\u{8}'),
                'n' | '\n' => out.push('\n'),
                'r' => out.push('\r'),
                'f' => out.push('\u{c}'),
                '\'' | '"' | '\\' | '/' => out.push(escaped),
                _ => return Err(JsonSyntaxError),
            }
        }
    }

    fn read_unquoted_literal(&mut self) -> String {
        let start = self.pos;
        while self.peek_char().is_some_and(is_literal) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// A keyword, number or unquoted string.
    fn read_unquoted_value(&mut self) -> Result<Json, JsonSyntaxError> {
        let literal = self.read_unquoted_literal();
        if literal.is_empty() {
            return Err(JsonSyntaxError);
        }
        if matches_keyword(&literal, "true") {
            return Ok(Json::Bool(true));
        }
        if matches_keyword(&literal, "false") {
            return Ok(Json::Bool(false));
        }
        if matches_keyword(&literal, "null") {
            return Ok(Json::Null);
        }
        if is_json_number(&literal) {
            return Ok(Json::Number(literal));
        }
        Ok(Json::String(literal))
    }
}

/// Gson accepts each keyword letter in either case.
fn matches_keyword(literal: &str, keyword: &str) -> bool {
    literal.len() == keyword.len()
        && literal
            .chars()
            .zip(keyword.chars())
            .all(|(c, k)| c == k || c == k.to_ascii_uppercase())
}

/// `JsonReader.peekNumber`'s grammar: `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
fn is_json_number(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    if bytes.first() == Some(&b'-') {
        i += 1;
    }
    let int_start = i;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let int_len = i - int_start;
    if int_len == 0 || (int_len > 1 && bytes[int_start] == b'0') {
        return false;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let frac_start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == frac_start {
            return false;
        }
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let exp_start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == exp_start {
            return false;
        }
    }
    i == bytes.len()
}

/// Appends a UTF-16 code unit read from a `\u` escape, joining surrogate pairs
/// that arrive as two consecutive escapes; lone surrogates become U+FFFD.
fn push_utf16_unit(
    out: &mut String,
    unit: u16,
    chars: &mut [char],
    pos: &mut usize,
) -> Result<(), JsonSyntaxError> {
    if (0xD800..0xDC00).contains(&unit) {
        if chars.get(*pos) == Some(&'\\') && chars.get(*pos + 1) == Some(&'u') {
            let hex: String = chars
                .get(*pos + 2..*pos + 6)
                .ok_or(JsonSyntaxError)?
                .iter()
                .collect();
            if let Ok(low) = u16::from_str_radix(&hex, 16) {
                if (0xDC00..0xE000).contains(&low) {
                    *pos += 6;
                    let code =
                        0x10000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(low) - 0xDC00);
                    out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                    return Ok(());
                }
            }
        }
        out.push('\u{fffd}');
    } else {
        out.push(char::from_u32(u32::from(unit)).unwrap_or('\u{fffd}'));
    }
    Ok(())
}

/// `GsonHelper.toStableString`: compact output with object keys in natural
/// (UTF-16) order.
pub fn to_stable_string(json: &Json) -> String {
    let mut out = String::new();
    write_value(&mut out, json);
    out
}

fn write_value(out: &mut String, json: &Json) {
    match json {
        Json::Null => out.push_str("null"),
        Json::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Json::Number(text) => out.push_str(text),
        Json::String(text) => write_string(out, text),
        Json::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        Json::Object(members) => {
            let mut sorted: Vec<&(String, Json)> = members.iter().collect();
            sorted.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (index, (name, value)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(out, name);
                out.push(':');
                write_value(out, value);
            }
            out.push('}');
        }
    }
}

/// `JsonWriter.string` with `htmlSafe = false`.
fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{c}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
