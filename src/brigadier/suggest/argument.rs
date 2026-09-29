//! Argument parsing and suggestions for server-side completion.
//!
//! Brigadier's own argument types (`bool`, numbers, strings) are ports of
//! `StringReader`/`*ArgumentType.parse`. Minecraft argument types are consumed
//! by [`consume_minecraft_argument`], a structural approximation that only
//! decides how many characters an argument occupies (balanced quotes, brackets
//! and braces; fixed token counts for coordinates).
//!
//! TODO(command-argument-parsers): replace the approximation with exact ports
//! of the argument classes under `net/minecraft/commands/arguments` (entity
//! selectors, NBT, components, block/item predicates) and their
//! `listSuggestions` implementations; they need live players, registries and
//! scoreboards that are not reachable from the command tree yet.

use serde_json::Value;

pub(super) mod coordinates;

use super::SuggestionsBuilder;

/// Java `StringReader` over UTF-16 code units.
pub(crate) struct StringReader<'a> {
    input: &'a [u16],
    cursor: usize,
}

const SYNTAX_ESCAPE: u16 = b'\\' as u16;
const DOUBLE_QUOTE: u16 = b'"' as u16;
const SINGLE_QUOTE: u16 = b'\'' as u16;

impl<'a> StringReader<'a> {
    pub(crate) fn new(input: &'a [u16], cursor: usize) -> Self {
        Self { input, cursor }
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    /// `canRead(length)`.
    pub(crate) fn can_read(&self, length: usize) -> bool {
        self.cursor + length <= self.input.len()
    }

    pub(crate) fn peek(&self) -> Option<u16> {
        self.input.get(self.cursor).copied()
    }

    pub(crate) fn skip(&mut self) {
        self.cursor += 1;
    }

    /// `LiteralCommandNode.parse`: the literal must be followed by a space or
    /// the end of input; the comparison is case sensitive.
    pub(crate) fn read_literal(&mut self, literal: &str) -> bool {
        let units: Vec<u16> = literal.encode_utf16().collect();
        let end = self.cursor + units.len();
        let matches = end <= self.input.len()
            && self.input[self.cursor..end] == units[..]
            && (end == self.input.len() || self.input[end] == b' ' as u16);
        if matches {
            self.cursor = end;
        }
        matches
    }

    fn is_allowed_in_unquoted_string(unit: u16) -> bool {
        matches!(unit, 0x30..=0x39 | 0x41..=0x5a | 0x61..=0x7a)
            || unit == b'_' as u16
            || unit == b'-' as u16
            || unit == b'.' as u16
            || unit == b'+' as u16
    }

    fn is_allowed_number(unit: u16) -> bool {
        matches!(unit, 0x30..=0x39) || unit == b'.' as u16 || unit == b'-' as u16
    }

    /// `readUnquotedString`.
    fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.peek().is_some_and(Self::is_allowed_in_unquoted_string) {
            self.cursor += 1;
        }
        String::from_utf16_lossy(&self.input[start..self.cursor])
    }

    /// `readStringUntil`; `None` on an invalid escape or missing end quote.
    fn read_string_until(&mut self, terminator: u16) -> Option<String> {
        let mut result = Vec::new();
        let mut escaped = false;
        while let Some(c) = self.peek() {
            self.cursor += 1;
            if escaped {
                if c == terminator || c == SYNTAX_ESCAPE {
                    result.push(c);
                    escaped = false;
                } else {
                    self.cursor -= 1;
                    return None;
                }
            } else if c == SYNTAX_ESCAPE {
                escaped = true;
            } else if c == terminator {
                return Some(String::from_utf16_lossy(&result));
            } else {
                result.push(c);
            }
        }
        None
    }

    /// `readString`: quoted or unquoted.
    fn read_string(&mut self) -> Option<String> {
        match self.peek() {
            None => Some(String::new()),
            Some(quote @ (DOUBLE_QUOTE | SINGLE_QUOTE)) => {
                self.cursor += 1;
                self.read_string_until(quote)
            }
            Some(_) => Some(self.read_unquoted_string()),
        }
    }

    /// The digits/`.`/`-` run shared by `readInt`, `readLong`, `readFloat`, `readDouble`.
    fn read_number_text(&mut self) -> String {
        let start = self.cursor;
        while self.peek().is_some_and(Self::is_allowed_number) {
            self.cursor += 1;
        }
        String::from_utf16_lossy(&self.input[start..self.cursor])
    }

    /// `readInt`; `None` on a syntax error (cursor is restored).
    pub(crate) fn read_int(&mut self) -> Option<i32> {
        let start = self.cursor;
        let value = self.read_number_text().parse::<i32>().ok();
        if value.is_none() {
            self.cursor = start;
        }
        value
    }

    /// `readDouble`; `None` on a syntax error (cursor is restored).
    pub(crate) fn read_double(&mut self) -> Option<f64> {
        let start = self.cursor;
        let value = self.read_number_text().parse::<f64>().ok();
        if value.is_none() {
            self.cursor = start;
        }
        value
    }

    /// `readFloat`; `None` on a syntax error (cursor is restored).
    pub(crate) fn read_float(&mut self) -> Option<f32> {
        let start = self.cursor;
        let value = self.read_number_text().parse::<f32>().ok();
        if value.is_none() {
            self.cursor = start;
        }
        value
    }

    /// Consumes to the next space outside quotes/brackets (see [`consume_minecraft_argument`]).
    fn read_balanced_token(&mut self) -> bool {
        let start = self.cursor;
        let mut quote: Option<u16> = None;
        let mut escaped = false;
        let mut depth = 0usize;
        while let Some(c) = self.peek() {
            if let Some(q) = quote {
                if escaped {
                    escaped = false;
                } else if c == SYNTAX_ESCAPE {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            } else if c == DOUBLE_QUOTE || c == SINGLE_QUOTE {
                quote = Some(c);
            } else if c == b'[' as u16 || c == b'{' as u16 {
                depth += 1;
            } else if (c == b']' as u16 || c == b'}' as u16) && depth > 0 {
                depth -= 1;
            } else if c == b' ' as u16 && depth == 0 {
                break;
            }
            self.cursor += 1;
        }
        quote.is_none() && depth == 0 && self.cursor > start
    }
}

/// `ArgumentType.parse` for the node's parser; `false` on a syntax error.
pub(crate) fn parse(parser: &str, properties: &Value, reader: &mut StringReader<'_>) -> bool {
    match parser {
        "brigadier:bool" => {
            let start = reader.cursor();
            let ok = matches!(reader.read_string().as_deref(), Some("true" | "false"));
            if !ok {
                reader.set_cursor(start);
            }
            ok
        }
        "brigadier:integer" => parse_integral::<i32>(properties, reader),
        "brigadier:long" => parse_integral::<i64>(properties, reader),
        "brigadier:float" => {
            parse_floating(properties, reader, |s| s.parse::<f32>().ok().map(f64::from))
        }
        "brigadier:double" => parse_floating(properties, reader, |s| s.parse::<f64>().ok()),
        "brigadier:string" => match properties["type"].as_str() {
            Some("greedy") => {
                reader.set_cursor(reader.input.len());
                true
            }
            Some("phrase") => reader.read_string().is_some(),
            _ => {
                reader.read_unquoted_string();
                true
            }
        },
        other => consume_minecraft_argument(other, reader),
    }
}

fn parse_integral<T>(properties: &Value, reader: &mut StringReader<'_>) -> bool
where
    T: std::str::FromStr + Into<i64>,
{
    let start = reader.cursor();
    let text = reader.read_number_text();
    let value = text.parse::<T>().ok().map(Into::into);
    let in_range = value.is_some_and(|v: i64| {
        properties["min"].as_i64().is_none_or(|min| v >= min)
            && properties["max"].as_i64().is_none_or(|max| v <= max)
    });
    if !in_range {
        reader.set_cursor(start);
    }
    in_range
}

fn parse_floating(
    properties: &Value,
    reader: &mut StringReader<'_>,
    convert: impl Fn(&str) -> Option<f64>,
) -> bool {
    let start = reader.cursor();
    let text = reader.read_number_text();
    let in_range = convert(&text).is_some_and(|v| {
        properties["min"].as_f64().is_none_or(|min| v >= min)
            && properties["max"].as_f64().is_none_or(|max| v <= max)
    });
    if !in_range {
        reader.set_cursor(start);
    }
    in_range
}

/// Structural consumption of a Minecraft argument type (see the module docs).
fn consume_minecraft_argument(parser: &str, reader: &mut StringReader<'_>) -> bool {
    let id = parser.strip_prefix("minecraft:").unwrap_or(parser);
    match id {
        // Consume everything that is left (`MessageArgument`).
        "message" => {
            reader.set_cursor(reader.input.len());
            true
        }
        // Positions take one whitespace separated coordinate each.
        "vec3" | "block_pos" => consume_tokens(reader, 3),
        "vec2" | "column_pos" | "rotation" => consume_tokens(reader, 2),
        _ => reader.read_balanced_token(),
    }
}

fn consume_tokens(reader: &mut StringReader<'_>, count: usize) -> bool {
    for index in 0..count {
        if index > 0 {
            if reader.peek() != Some(b' ' as u16) {
                return false;
            }
            reader.skip();
        }
        if !reader.read_balanced_token() {
            return false;
        }
    }
    true
}

/// `SharedSuggestionProvider.matchesSubStr`: the pattern must prefix the input
/// or any segment following one of `._/`.
pub(crate) fn matches_sub_str(pattern: &str, input: &str) -> bool {
    let mut index = 0;
    while !input[index..].starts_with(pattern) {
        match input[index..].find(['.', '_', '/']) {
            Some(offset) => index += offset + 1,
            None => return false,
        }
    }
    true
}

/// `SharedSuggestionProvider.suggest(Iterable<String>, builder)`.
pub(crate) fn suggest_matching<'v>(
    values: impl IntoIterator<Item = &'v str>,
    builder: &mut SuggestionsBuilder<'_>,
) {
    let lower_prefix = builder.remaining_lower_case();
    for name in values {
        if matches_sub_str(&lower_prefix, &name.to_lowercase()) {
            builder.suggest(name);
        }
    }
}

/// `ArgumentType.listSuggestions` for the parsers whose suggestions do not
/// depend on live server state.
pub(crate) fn list_suggestions(
    parser: &str,
    _properties: &Value,
    builder: &mut SuggestionsBuilder<'_>,
) {
    match parser {
        // `BoolArgumentType.listSuggestions`: plain prefix match.
        "brigadier:bool" => {
            let remaining = builder.remaining_lower_case();
            for value in ["true", "false"] {
                if value.starts_with(&remaining) {
                    builder.suggest(value);
                }
            }
        }
        "minecraft:gamemode" => {
            suggest_matching(["survival", "creative", "adventure", "spectator"], builder)
        }
        "minecraft:entity_anchor" => suggest_matching(["feet", "eyes"], builder),
        "minecraft:time" => suggest_time_units(builder),
        "minecraft:vec3" => coordinates::suggest(builder, coordinates::Kind::Vec3),
        "minecraft:block_pos" => coordinates::suggest(builder, coordinates::Kind::BlockPos),
        "minecraft:vec2" => coordinates::suggest(builder, coordinates::Kind::Vec2),
        "minecraft:column_pos" => coordinates::suggest(builder, coordinates::Kind::ColumnPos),
        _ => {}
    }
}

/// `TimeArgument.listSuggestions`: once a number has been typed, offer the
/// unit suffixes (`d`, `s`, `t`) at the position after the number.
fn suggest_time_units(builder: &mut SuggestionsBuilder<'_>) {
    let remaining: Vec<u16> = builder.remaining().encode_utf16().collect();
    let mut reader = StringReader::new(&remaining, 0);
    if reader.read_float().is_none() {
        return;
    }
    let mut offset = SuggestionsBuilder::new(builder.input, builder.start + reader.cursor());
    suggest_matching(["d", "s", "t"], &mut offset);
    builder.result.extend(offset.result);
}
