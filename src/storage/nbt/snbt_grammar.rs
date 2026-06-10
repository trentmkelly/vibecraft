#![allow(dead_code)]

use std::io;

use super::compound_tag::put_compound_entry;
use super::snbt_operations;
use super::Tag;

// TODO(snbt-grammar-unicode-name-database): Java delegates `\N{...}` to
// `Character.codePointOf`, which uses the JDK Unicode character-name database.
// Rust std does not expose that table. Common aliases are handled below; full
// parity should be wired to a generated Unicode name table.

pub fn parse_snbt(input: &str) -> io::Result<Tag> {
    let mut parser = SnbtGrammarParser::new(input);
    let tag = parser.parse_literal()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return Err(parser.error("trailing SNBT input"));
    }
    Ok(tag)
}

pub fn escape_control_characters(ch: char) -> Option<String> {
    match ch {
        '\u{0008}' => Some("b".to_string()),
        '\t' => Some("t".to_string()),
        '\n' => Some("n".to_string()),
        '\u{000C}' => Some("f".to_string()),
        '\r' => Some("r".to_string()),
        ch if ch < ' ' => Some(format!("x{:02X}", ch as u32)),
        _ => None,
    }
}

pub fn can_start_number(ch: char) -> bool {
    matches!(ch, '+' | '-' | '.' | '0'..='9')
}

pub fn is_allowed_to_start_unquoted_string(ch: char) -> bool {
    !can_start_number(ch)
}

pub fn clean_underscores(contents: &str) -> String {
    contents.chars().filter(|ch| *ch != '_').collect()
}

struct SnbtGrammarParser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> SnbtGrammarParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn parse_literal(&mut self) -> io::Result<Tag> {
        self.skip_ws();
        match self.peek() {
            Some(ch) if can_start_number(ch) => self.parse_number(),
            Some('"') | Some('\'') => self.parse_quoted_string().map(Tag::String),
            Some('{') => self.parse_map(),
            Some('[') => self.parse_list_or_array(),
            Some(_) => self.parse_unquoted_or_builtin(),
            None => Err(self.error("empty SNBT input")),
        }
    }

    fn parse_map(&mut self) -> io::Result<Tag> {
        self.expect('{')?;
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            if self.consume('}') {
                break;
            }
            let key = self.parse_map_key()?;
            if key.is_empty() {
                return Err(self.error("empty SNBT key"));
            }
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_literal()?;
            put_compound_entry(&mut entries, key, value);
            self.skip_ws();
            if self.consume(',') {
                self.skip_ws();
                if self.consume('}') {
                    break;
                }
                continue;
            }
            self.expect('}')?;
            break;
        }
        Ok(Tag::Compound(entries))
    }

    fn parse_map_key(&mut self) -> io::Result<String> {
        self.skip_ws();
        match self.peek() {
            Some('"') | Some('\'') => self.parse_quoted_string(),
            Some(_) => self.parse_unquoted_string(),
            None => Err(self.error("missing SNBT map key")),
        }
    }

    fn parse_list_or_array(&mut self) -> io::Result<Tag> {
        self.expect('[')?;
        self.skip_ws();
        if self.peek_array_prefix('B') {
            self.next();
            self.expect(';')?;
            return self.parse_integer_array(ArrayPrefix::Byte);
        }
        if self.peek_array_prefix('I') {
            self.next();
            self.expect(';')?;
            return self.parse_integer_array(ArrayPrefix::Int);
        }
        if self.peek_array_prefix('L') {
            self.next();
            self.expect(';')?;
            return self.parse_integer_array(ArrayPrefix::Long);
        }

        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            values.push(self.parse_literal()?);
            self.skip_ws();
            if self.consume(',') {
                self.skip_ws();
                if self.consume(']') {
                    break;
                }
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(Tag::List(values))
    }

    fn parse_integer_array(&mut self, prefix: ArrayPrefix) -> io::Result<Tag> {
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            let literal = self.parse_integer_literal()?;
            values.push(prefix.coerce(literal)?);
            self.skip_ws();
            if self.consume(',') {
                self.skip_ws();
                if self.consume(']') {
                    break;
                }
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(match prefix {
            ArrayPrefix::Byte => {
                Tag::ByteArray(values.into_iter().map(|value| value as u8 as i8).collect())
            }
            ArrayPrefix::Int => {
                Tag::IntArray(values.into_iter().map(|value| value as i32).collect())
            }
            ArrayPrefix::Long => Tag::LongArray(values),
        })
    }

    fn parse_unquoted_or_builtin(&mut self) -> io::Result<Tag> {
        let contents = self.parse_unquoted_string()?;
        let Some(first) = contents.chars().next() else {
            return Err(self.error("invalid unquoted SNBT string start"));
        };
        if !is_allowed_to_start_unquoted_string(first) {
            return Err(self.error("invalid unquoted SNBT string start"));
        }
        self.skip_ws();
        if self.consume('(') {
            let arguments = self.parse_argument_list()?;
            self.expect(')')?;
            return snbt_operations::run_builtin(&contents, &arguments)
                .map_err(|err| self.error(format!("SNBT builtin failed: {err:?}")));
        }
        if contents.eq_ignore_ascii_case("true") {
            Ok(Tag::Byte(1))
        } else if contents.eq_ignore_ascii_case("false") {
            Ok(Tag::Byte(0))
        } else {
            Ok(Tag::String(contents))
        }
    }

    fn parse_argument_list(&mut self) -> io::Result<Vec<Tag>> {
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(')') {
                break;
            }
            values.push(self.parse_literal()?);
            self.skip_ws();
            if self.consume(',') {
                self.skip_ws();
                if self.peek() == Some(')') {
                    break;
                }
                continue;
            }
            break;
        }
        Ok(values)
    }

    fn parse_number(&mut self) -> io::Result<Tag> {
        let start = self.cursor;
        let integer = self.try_parse_integer_literal();
        if let Ok(literal) = integer {
            let after_integer = self.cursor;
            if self.peek().is_some_and(is_number_continuation_for_float) {
                self.cursor = start;
                return self.parse_float_literal();
            }
            return literal
                .create()
                .map_err(|message| self.error_at(after_integer, message));
        }
        self.cursor = start;
        self.parse_float_literal()
    }

    fn parse_float_literal(&mut self) -> io::Result<Tag> {
        let start = self.cursor;
        self.consume_sign();
        let whole_start = self.cursor;
        self.consume_decimal_digits();
        let has_whole = self.cursor != whole_start;
        let mut has_fraction = false;
        if self.consume('.') {
            let fraction_start = self.cursor;
            self.consume_decimal_digits();
            has_fraction = self.cursor != fraction_start;
        }
        let mut has_exponent = false;
        if matches!(self.peek(), Some('e' | 'E')) {
            self.next();
            self.consume_sign();
            let exponent_start = self.cursor;
            self.consume_decimal_digits();
            has_exponent = self.cursor != exponent_start;
            if !has_exponent {
                return Err(self.error("expected decimal exponent"));
            }
        }
        let suffix = self.peek().filter(|ch| matches!(ch, 'f' | 'F' | 'd' | 'D'));
        if suffix.is_some() {
            self.next();
        }
        if !has_fraction && !has_exponent && suffix.is_none() {
            return Err(self.error("expected SNBT number"));
        }
        if !has_whole && !has_fraction {
            return Err(self.error("expected float digits"));
        }
        let contents =
            self.input[start..self.cursor - suffix.map_or(0, char::len_utf8)].replace('_', "");
        match suffix {
            Some('f' | 'F') => {
                let value = contents.parse::<f32>().map_err(invalid_snbt_number)?;
                if !value.is_finite() {
                    return Err(self.error("infinity not allowed"));
                }
                Ok(Tag::Float(value))
            }
            _ => {
                let value = contents.parse::<f64>().map_err(invalid_snbt_number)?;
                if !value.is_finite() {
                    return Err(self.error("infinity not allowed"));
                }
                Ok(Tag::Double(value))
            }
        }
    }

    fn try_parse_integer_literal(&mut self) -> Result<IntegerLiteral, String> {
        let sign = self.consume_sign();
        let (base, digits) = if self.consume('0') {
            if matches!(self.peek(), Some('x' | 'X')) {
                self.next();
                (Base::Hex, self.consume_run(is_hex_digit_or_underscore))
            } else if matches!(self.peek(), Some('b' | 'B')) {
                self.next();
                (
                    Base::Binary,
                    self.consume_run(is_binary_digit_or_underscore),
                )
            } else {
                let more_digits = self.consume_run(is_decimal_digit_or_underscore);
                if !more_digits.is_empty() {
                    return Err("leading zero not allowed".to_string());
                }
                (Base::Decimal, "0".to_string())
            }
        } else {
            (
                Base::Decimal,
                self.consume_run(is_decimal_digit_or_underscore),
            )
        };
        if digits.is_empty() {
            return Err("expected integer digits".to_string());
        }
        let suffix = self.consume_integer_suffix();
        Ok(IntegerLiteral {
            sign,
            base,
            digits,
            suffix,
        })
    }

    fn parse_integer_literal(&mut self) -> io::Result<IntegerLiteral> {
        self.try_parse_integer_literal()
            .map_err(|message| self.error(message))
    }

    fn parse_quoted_string(&mut self) -> io::Result<String> {
        let quote = self
            .next()
            .ok_or_else(|| self.error("missing SNBT string quote"))?;
        let mut output = String::new();
        loop {
            match self.next() {
                Some(ch) if ch == quote => break,
                Some('\\') => output.push_str(&self.parse_escape_sequence()?),
                Some(ch) => output.push(ch),
                None => return Err(self.error("unterminated SNBT string")),
            }
        }
        Ok(output)
    }

    fn parse_escape_sequence(&mut self) -> io::Result<String> {
        match self.next() {
            Some('b') => Ok("\u{0008}".to_string()),
            Some('s') => Ok(" ".to_string()),
            Some('t') => Ok("\t".to_string()),
            Some('n') => Ok("\n".to_string()),
            Some('f') => Ok("\u{000C}".to_string()),
            Some('r') => Ok("\r".to_string()),
            Some('\\') => Ok("\\".to_string()),
            Some('\'') => Ok("'".to_string()),
            Some('"') => Ok("\"".to_string()),
            Some('x') => self.parse_hex_escape(2),
            Some('u') => self.parse_hex_escape(4),
            Some('U') => self.parse_hex_escape(8),
            Some('N') => self.parse_unicode_name_escape(),
            Some(ch) => Err(self.error(format!("invalid SNBT escape {ch}"))),
            None => Err(self.error("unterminated SNBT escape")),
        }
    }

    fn parse_hex_escape(&mut self, len: usize) -> io::Result<String> {
        let start = self.cursor;
        for _ in 0..len {
            if !self.next().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                return Err(self.error(format!("expected {len}-digit hex escape")));
            }
        }
        let value = u32::from_str_radix(&self.input[start..self.cursor], 16)
            .map_err(invalid_snbt_number)?;
        char::from_u32(value)
            .map(|ch| ch.to_string())
            .ok_or_else(|| self.error(format!("invalid codepoint U+{value:08X}")))
    }

    fn parse_unicode_name_escape(&mut self) -> io::Result<String> {
        self.expect('{')?;
        let start = self.cursor;
        while let Some(ch) = self.peek() {
            if ch == '}' {
                break;
            }
            if !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | ' ')) {
                return Err(self.error("invalid character name"));
            }
            self.next();
        }
        let name = self.input[start..self.cursor].to_ascii_uppercase();
        self.expect('}')?;
        match unicode_name_codepoint(&name) {
            Some(ch) => Ok(ch.to_string()),
            None => Err(self.error("invalid character name")),
        }
    }

    fn parse_unquoted_string(&mut self) -> io::Result<String> {
        self.skip_ws();
        let start = self.cursor;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || matches!(ch, ':' | ',' | ']' | '}' | ')' | '(') {
                break;
            }
            if matches!(ch, '"' | '\'') {
                break;
            }
            self.next();
        }
        if self.cursor == start {
            return Err(self.error("missing SNBT token"));
        }
        Ok(self.input[start..self.cursor].to_string())
    }

    fn consume_integer_suffix(&mut self) -> IntegerSuffix {
        let signed = match (self.peek(), self.peek_nth(1)) {
            (Some('u' | 'U'), Some(kind @ ('b' | 'B' | 's' | 'S' | 'i' | 'I' | 'l' | 'L'))) => {
                self.next();
                self.next();
                return IntegerSuffix {
                    signed: Some(SignedPrefix::Unsigned),
                    ty: Some(TypeSuffix::from_integer_char(kind)),
                };
            }
            (Some('s' | 'S'), Some(kind @ ('b' | 'B' | 's' | 'S' | 'i' | 'I' | 'l' | 'L'))) => {
                self.next();
                self.next();
                return IntegerSuffix {
                    signed: Some(SignedPrefix::Signed),
                    ty: Some(TypeSuffix::from_integer_char(kind)),
                };
            }
            _ => None,
        };
        let ty = self.peek().and_then(TypeSuffix::try_from_bare_integer_char);
        if ty.is_some() {
            self.next();
        }
        IntegerSuffix { signed, ty }
    }

    fn consume_sign(&mut self) -> Sign {
        if self.consume('-') {
            Sign::Minus
        } else {
            self.consume('+');
            Sign::Plus
        }
    }

    fn consume_decimal_digits(&mut self) {
        self.consume_run(is_decimal_digit_or_underscore);
    }

    fn consume_run(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let start = self.cursor;
        while self.peek().is_some_and(&predicate) {
            self.next();
        }
        self.input[start..self.cursor].to_string()
    }

    fn peek_array_prefix(&self, prefix: char) -> bool {
        self.peek() == Some(prefix)
            && self.input[self.cursor + prefix.len_utf8()..].starts_with(';')
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.next();
        }
    }

    fn expect(&mut self, expected: char) -> io::Result<()> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(self.error(format!("expected SNBT character {expected}")))
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.next();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.cursor..].chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.input[self.cursor..].chars().nth(n)
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    fn error(&self, message: impl Into<String>) -> io::Error {
        self.error_at(self.cursor, message)
    }

    fn error_at(&self, cursor: usize, message: impl Into<String>) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} at byte {cursor}", message.into()),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayPrefix {
    Byte,
    Int,
    Long,
}

impl ArrayPrefix {
    fn coerce(self, literal: IntegerLiteral) -> io::Result<i64> {
        let ty = literal.suffix.ty.unwrap_or(match self {
            Self::Byte => TypeSuffix::Byte,
            Self::Int => TypeSuffix::Int,
            Self::Long => TypeSuffix::Long,
        });
        let allowed = match self {
            Self::Byte => matches!(ty, TypeSuffix::Byte),
            Self::Int => matches!(ty, TypeSuffix::Int | TypeSuffix::Byte | TypeSuffix::Short),
            Self::Long => matches!(
                ty,
                TypeSuffix::Long | TypeSuffix::Byte | TypeSuffix::Short | TypeSuffix::Int
            ),
        };
        if !allowed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid array element type",
            ));
        }
        literal
            .create_integer_value(ty)
            .map_err(invalid_snbt_number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntegerLiteral {
    sign: Sign,
    base: Base,
    digits: String,
    suffix: IntegerSuffix,
}

impl IntegerLiteral {
    fn create(self) -> Result<Tag, String> {
        let ty = self.suffix.ty.unwrap_or(TypeSuffix::Int);
        let value = self.create_integer_value(ty)?;
        Ok(match ty {
            TypeSuffix::Byte => Tag::Byte(value as u8 as i8),
            TypeSuffix::Short => Tag::Short(value as u16 as i16),
            TypeSuffix::Int => Tag::Int(value as u32 as i32),
            TypeSuffix::Long => Tag::Long(value),
        })
    }

    fn create_integer_value(&self, ty: TypeSuffix) -> Result<i64, String> {
        let is_signed = self.signed_or_default() == SignedPrefix::Signed;
        if !is_signed && self.sign == Sign::Minus {
            return Err("expected non-negative number".to_string());
        }
        let digits = clean_underscores(&self.digits);
        let radix = self.base.radix();
        if is_signed {
            let mut fixed = String::new();
            self.sign.append(&mut fixed);
            fixed.push_str(&digits);
            parse_signed(&fixed, radix, ty)
        } else {
            parse_unsigned(&digits, radix, ty)
        }
    }

    fn signed_or_default(&self) -> SignedPrefix {
        self.suffix.signed.unwrap_or(match self.base {
            Base::Binary | Base::Hex => SignedPrefix::Unsigned,
            Base::Decimal => SignedPrefix::Signed,
        })
    }
}

fn parse_signed(value: &str, radix: u32, ty: TypeSuffix) -> Result<i64, String> {
    match ty {
        TypeSuffix::Byte => i8::from_str_radix(value, radix).map(i64::from),
        TypeSuffix::Short => i16::from_str_radix(value, radix).map(i64::from),
        TypeSuffix::Int => i32::from_str_radix(value, radix).map(i64::from),
        TypeSuffix::Long => i64::from_str_radix(value, radix),
    }
    .map_err(|err| err.to_string())
}

fn parse_unsigned(value: &str, radix: u32, ty: TypeSuffix) -> Result<i64, String> {
    match ty {
        TypeSuffix::Byte => u8::from_str_radix(value, radix).map(i64::from),
        TypeSuffix::Short => u16::from_str_radix(value, radix).map(i64::from),
        TypeSuffix::Int => u32::from_str_radix(value, radix).map(|value| value as i64),
        TypeSuffix::Long => u64::from_str_radix(value, radix).map(|value| value as i64),
    }
    .map_err(|err| err.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Base {
    Binary,
    Decimal,
    Hex,
}

impl Base {
    fn radix(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Decimal => 10,
            Self::Hex => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntegerSuffix {
    signed: Option<SignedPrefix>,
    ty: Option<TypeSuffix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn append(self, out: &mut String) {
        if self == Self::Minus {
            out.push('-');
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignedPrefix {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeSuffix {
    Byte,
    Short,
    Int,
    Long,
}

impl TypeSuffix {
    fn from_integer_char(ch: char) -> Self {
        match ch {
            'b' | 'B' => Self::Byte,
            's' | 'S' => Self::Short,
            'i' | 'I' => Self::Int,
            'l' | 'L' => Self::Long,
            _ => unreachable!(),
        }
    }

    fn try_from_bare_integer_char(ch: char) -> Option<Self> {
        match ch {
            'b' | 'B' => Some(Self::Byte),
            's' | 'S' => Some(Self::Short),
            'i' | 'I' => Some(Self::Int),
            'l' | 'L' => Some(Self::Long),
            _ => None,
        }
    }
}

fn is_binary_digit_or_underscore(ch: char) -> bool {
    matches!(ch, '0' | '1' | '_')
}

fn is_decimal_digit_or_underscore(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '_'
}

fn is_hex_digit_or_underscore(ch: char) -> bool {
    ch.is_ascii_hexdigit() || ch == '_'
}

fn is_number_continuation_for_float(ch: char) -> bool {
    matches!(ch, '.' | 'e' | 'E' | 'f' | 'F' | 'd' | 'D')
}

fn unicode_name_codepoint(name: &str) -> Option<char> {
    match name {
        "SPACE" => Some(' '),
        "LINE FEED" => Some('\n'),
        "CHARACTER TABULATION" | "HORIZONTAL TABULATION" => Some('\t'),
        "CARRIAGE RETURN" => Some('\r'),
        "FORM FEED" => Some('\u{000C}'),
        "BACKSPACE" => Some('\u{0008}'),
        "QUOTATION MARK" => Some('"'),
        "APOSTROPHE" => Some('\''),
        "REVERSE SOLIDUS" => Some('\\'),
        _ => None,
    }
}

fn invalid_snbt_number(err: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNBT_GRAMMAR_JAVA: &str =
        include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/SnbtGrammar.java");

    #[test]
    fn snbt_grammar_matches_java_constants_and_control_escapes() {
        for sentinel in [
            "ERROR_EXPECTED_HEX_ESCAPE",
            "ERROR_INVALID_CODEPOINT",
            "ERROR_NO_SUCH_OPERATION",
            "NumberRunParseRule(ERROR_EXPECTED_BINARY_NUMERAL",
            "Pattern.compile(\"[-a-zA-Z0-9 ]+\")",
            "public static @Nullable String escapeControlCharacters",
            "case '\\b' -> \"b\";",
            "case '\\n' -> \"n\";",
            "default -> c < ' ' ? \"x\" + HEX_ESCAPE.toHexDigits((byte)c) : null;",
        ] {
            assert!(
                SNBT_GRAMMAR_JAVA.contains(sentinel),
                "missing SnbtGrammar sentinel {sentinel}"
            );
        }

        assert_eq!(escape_control_characters('\n'), Some("n".to_string()));
        assert_eq!(
            escape_control_characters('\u{0001}'),
            Some("x01".to_string())
        );
        assert_eq!(escape_control_characters('a'), None);
        assert!(can_start_number('-'));
        assert!(!is_allowed_to_start_unquoted_string('.'));
    }

    #[test]
    fn snbt_grammar_parses_modern_integer_and_float_forms() {
        for sentinel in [
            "StringReaderTerms.characters('u', 'U')",
            "StringReaderTerms.characters('s', 'S')",
            "ERROR_LEADING_ZERO_NOT_ALLOWED",
            "ERROR_INFINITY_NOT_ALLOWED",
            "case BINARY -> 2;",
            "case HEX -> 16;",
            "UnsignedBytes.parseUnsignedByte",
            "Long.parseUnsignedLong",
        ] {
            assert!(
                SNBT_GRAMMAR_JAVA.contains(sentinel),
                "missing SnbtGrammar sentinel {sentinel}"
            );
        }

        assert_eq!(parse_snbt("0b1010").unwrap(), Tag::Int(10));
        assert_eq!(parse_snbt("0xFFub").unwrap(), Tag::Byte(-1));
        assert_eq!(parse_snbt("0xFFFFus").unwrap(), Tag::Short(-1));
        assert_eq!(parse_snbt("0xFFFF_FFFFi").unwrap(), Tag::Int(-1));
        assert_eq!(parse_snbt("-12sl").unwrap(), Tag::Long(-12));
        assert_eq!(parse_snbt(".5f").unwrap(), Tag::Float(0.5));
        assert_eq!(parse_snbt("1e2").unwrap(), Tag::Double(100.0));
        assert!(parse_snbt("01").is_err());
        assert!(parse_snbt("1e9999").is_err());
    }

    #[test]
    fn snbt_grammar_parses_strings_builtins_maps_lists_and_arrays() {
        for sentinel in [
            "Term.sequence(StringReaderTerms.character('x'), rules.named(stringHex2))",
            "Term.sequence(StringReaderTerms.character('u'), rules.named(stringHex4))",
            "Term.sequence(StringReaderTerms.character('U'), rules.named(stringHex8))",
            "Term.sequence(StringReaderTerms.character('N'), StringReaderTerms.character('{')",
            "SnbtOperations.BUILTIN_OPERATIONS.get(key)",
            "builder.buildKeepingLast()",
            "Term.repeatedWithTrailingSeparator",
            "ArrayPrefix.BYTE",
            "ERROR_INVALID_ARRAY_ELEMENT_TYPE",
        ] {
            assert!(
                SNBT_GRAMMAR_JAVA.contains(sentinel),
                "missing SnbtGrammar sentinel {sentinel}"
            );
        }

        assert_eq!(
            parse_snbt(r#""a\s\x21\u0021\U00000021\N{SPACE}b""#).unwrap(),
            Tag::String("a !!! b".to_string())
        );
        assert_eq!(parse_snbt("true").unwrap(), Tag::Byte(1));
        assert_eq!(parse_snbt("bool(0)").unwrap(), Tag::Byte(0));
        assert_eq!(
            parse_snbt("{a:1,a:2,}").unwrap(),
            Tag::Compound(vec![("a".to_string(), Tag::Int(2))])
        );
        assert_eq!(
            parse_snbt("[1,2,]").unwrap(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)])
        );
        assert_eq!(
            parse_snbt("[B;0xFFub,1b]").unwrap(),
            Tag::ByteArray(vec![-1, 1])
        );
        assert_eq!(
            parse_snbt("[I;1b,2s,3]").unwrap(),
            Tag::IntArray(vec![1, 2, 3])
        );
        assert_eq!(
            parse_snbt("[L;1b,2s,3i,4l]").unwrap(),
            Tag::LongArray(vec![1, 2, 3, 4])
        );
        assert!(parse_snbt("[B;1s]").is_err());
    }
}
