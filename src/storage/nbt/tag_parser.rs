#![allow(dead_code)]

use std::io;

use super::{parse_snbt, Tag};

pub const ELEMENT_SEPARATOR: char = ',';
pub const NAME_VALUE_SEPARATOR: char = ':';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParserError {
    TrailingData,
    ExpectedCompound,
    ParseError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnbtStringReader {
    input: String,
    cursor: usize,
}

impl SnbtStringReader {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> &str {
        &self.input[self.cursor..]
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn skip_whitespace(&mut self) {
        while self
            .remaining()
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            self.cursor += self
                .remaining()
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);
        }
    }
}

pub fn parse_fully(input: &str) -> Result<Tag, TagParserError> {
    let mut reader = SnbtStringReader::new(input);
    parse_fully_reader(&mut reader)
}

pub fn parse_fully_reader(reader: &mut SnbtStringReader) -> Result<Tag, TagParserError> {
    let tag = parse_as_argument(reader)?;
    reader.skip_whitespace();
    if reader.remaining().is_empty() {
        Ok(tag)
    } else {
        Err(TagParserError::TrailingData)
    }
}

pub fn parse_as_argument(reader: &mut SnbtStringReader) -> Result<Tag, TagParserError> {
    let start = reader.cursor();
    let end = find_tag_end(reader.remaining()).ok_or_else(|| {
        TagParserError::ParseError("unable to find SNBT argument boundary".to_string())
    })?;
    let consumed = &reader.remaining()[..end];
    let tag = parse_snbt(consumed).map_err(parse_error)?;
    reader.set_cursor(start + end);
    Ok(tag)
}

pub fn parse_compound_fully(input: &str) -> Result<Tag, TagParserError> {
    let mut reader = SnbtStringReader::new(input);
    let tag = parse_fully_reader(&mut reader)?;
    cast_to_compound_or_throw(tag)
}

pub fn parse_compound_as_argument(reader: &mut SnbtStringReader) -> Result<Tag, TagParserError> {
    let start = reader.cursor();
    let tag = parse_as_argument(reader)?;
    match cast_to_compound_or_throw(tag) {
        Ok(tag) => Ok(tag),
        Err(error) => {
            reader.set_cursor(start);
            Err(error)
        }
    }
}

fn cast_to_compound_or_throw(tag: Tag) -> Result<Tag, TagParserError> {
    if matches!(tag, Tag::Compound(_)) {
        Ok(tag)
    } else {
        Err(TagParserError::ExpectedCompound)
    }
}

fn parse_error(error: io::Error) -> TagParserError {
    TagParserError::ParseError(error.to_string())
}

fn find_tag_end(input: &str) -> Option<usize> {
    let mut chars = input.char_indices().peekable();
    let first = chars.peek().copied()?;
    match first.1 {
        '{' => find_balanced_end(input, '{', '}'),
        '[' => find_balanced_end(input, '[', ']'),
        '"' | '\'' => find_quoted_end(input, first.0, first.1),
        _ => {
            let mut end = input.len();
            for (index, value) in input.char_indices() {
                if value.is_whitespace() || value == ELEMENT_SEPARATOR {
                    end = index;
                    break;
                }
            }
            Some(end)
        }
    }
}

fn find_balanced_end(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, value) in input.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == active_quote {
                quote = None;
            }
            continue;
        }

        if value == '"' || value == '\'' {
            quote = Some(value);
        } else if value == open {
            depth += 1;
        } else if value == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index + value.len_utf8());
            }
        }
    }
    None
}

fn find_quoted_end(input: &str, start_index: usize, quote: char) -> Option<usize> {
    let mut escaped = false;
    for (index, value) in input[start_index + quote.len_utf8()..].char_indices() {
        let absolute = start_index + quote.len_utf8() + index;
        if escaped {
            escaped = false;
        } else if value == '\\' {
            escaped = true;
        } else if value == quote {
            return Some(absolute + value.len_utf8());
        }
    }
    None
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const TAG_PARSER_JAVA: &str =
        include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/TagParser.java");

    #[test]
    fn tag_parser_matches_java_entry_points_and_cursor_policy() {
        for sentinel in [
            "ERROR_TRAILING_DATA",
            "ERROR_EXPECTED_COMPOUND",
            "public static final char ELEMENT_SEPARATOR = ',';",
            "public static final char NAME_VALUE_SEPARATOR = ':';",
            "private static final TagParser<Tag> NBT_OPS_PARSER = create(NbtOps.INSTANCE);",
            "public DynamicOps<T> getOps()",
            "SnbtGrammar.createParser(ops)",
            "castToCompoundOrThrow",
            "parseCompoundFully(final String input)",
            "reader.skipWhitespace();",
            "if (reader.canRead())",
            "throw ERROR_TRAILING_DATA.createWithContext(reader);",
            "parseAsArgument(final StringReader reader)",
            "parseCompoundAsArgument(final StringReader reader)",
        ] {
            assert!(
                TAG_PARSER_JAVA.contains(sentinel),
                "missing TagParser sentinel {sentinel}"
            );
        }

        assert_eq!(ELEMENT_SEPARATOR, ',');
        assert_eq!(NAME_VALUE_SEPARATOR, ':');
        assert_eq!(
            parse_fully("{answer:42}").unwrap(),
            Tag::Compound(vec![("answer".to_string(), Tag::Int(42))])
        );
        assert_eq!(
            parse_fully("{answer:42} trailing").unwrap_err(),
            TagParserError::TrailingData
        );
        assert_eq!(
            parse_compound_fully("12").unwrap_err(),
            TagParserError::ExpectedCompound
        );
        assert_eq!(
            parse_compound_fully("{flag:true}").unwrap(),
            Tag::Compound(vec![("flag".to_string(), Tag::Byte(1))])
        );

        let mut reader = SnbtStringReader::new("{name:'A B',list:[1,2]} trailing");
        assert_eq!(
            parse_as_argument(&mut reader).unwrap(),
            Tag::Compound(vec![
                ("name".to_string(), Tag::String("A B".to_string())),
                (
                    "list".to_string(),
                    Tag::List(vec![Tag::Int(1), Tag::Int(2)])
                ),
            ])
        );
        assert_eq!(reader.cursor(), "{name:'A B',list:[1,2]}".len());
        assert_eq!(reader.remaining(), " trailing");

        let mut non_compound = SnbtStringReader::new("[0] trailing");
        assert_eq!(
            parse_compound_as_argument(&mut non_compound).unwrap_err(),
            TagParserError::ExpectedCompound
        );
        assert_eq!(non_compound.cursor(), 0);

        let mut compound = SnbtStringReader::new("{foo:bar} trailing");
        assert_eq!(
            parse_compound_as_argument(&mut compound).unwrap(),
            Tag::Compound(vec![("foo".to_string(), Tag::String("bar".to_string()))])
        );
        assert_eq!(compound.cursor(), "{foo:bar}".len());
    }
}
