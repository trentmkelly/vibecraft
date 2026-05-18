use std::io::{self, Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

pub const DEFAULT_MAX_NBT_DEPTH: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NbtSizeTracker {
    pub payload_bytes: usize,
    pub nodes: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtFieldSelector {
    pub path: Vec<String>,
}

impl Tag {
    pub fn id(&self) -> u8 {
        match self {
            Tag::End => 0,
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::ByteArray(_) => 7,
            Tag::String(_) => 8,
            Tag::List(_) => 9,
            Tag::Compound(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }

    pub fn read_payload<R: Read>(id: u8, reader: &mut R) -> io::Result<Self> {
        Self::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH)
    }

    pub fn read_payload_limited<R: Read>(
        id: u8,
        reader: &mut R,
        max_depth: usize,
    ) -> io::Result<Self> {
        Self::read_payload_at_depth(id, reader, 0, max_depth)
    }

    fn read_payload_at_depth<R: Read>(
        id: u8,
        reader: &mut R,
        depth: usize,
        max_depth: usize,
    ) -> io::Result<Self> {
        if depth > max_depth {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NBT depth limit exceeded",
            ));
        }
        Ok(match id {
            0 => Tag::End,
            1 => Tag::Byte(read_i8(reader)?),
            2 => Tag::Short(read_i16(reader)?),
            3 => Tag::Int(read_i32(reader)?),
            4 => Tag::Long(read_i64(reader)?),
            5 => Tag::Float(f32::from_bits(read_u32(reader)?)),
            6 => Tag::Double(f64::from_bits(read_u64(reader)?)),
            7 => {
                let len = read_len_i32(reader)?;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(read_i8(reader)?);
                }
                Tag::ByteArray(values)
            }
            8 => Tag::String(read_string(reader)?),
            9 => {
                let element_id = read_u8(reader)?;
                let len = read_len_i32(reader)?;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(Tag::read_payload_at_depth(
                        element_id,
                        reader,
                        depth + 1,
                        max_depth,
                    )?);
                }
                Tag::List(values)
            }
            10 => {
                let mut values = Vec::new();
                loop {
                    let child_id = read_u8(reader)?;
                    if child_id == 0 {
                        break;
                    }
                    let name = read_string(reader)?;
                    let payload =
                        Tag::read_payload_at_depth(child_id, reader, depth + 1, max_depth)?;
                    values.push((name, payload));
                }
                Tag::Compound(values)
            }
            11 => {
                let len = read_len_i32(reader)?;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(read_i32(reader)?);
                }
                Tag::IntArray(values)
            }
            12 => {
                let len = read_len_i32(reader)?;
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    values.push(read_i64(reader)?);
                }
                Tag::LongArray(values)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unknown NBT tag id",
                ))
            }
        })
    }

    pub fn write_payload<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Tag::End => Ok(()),
            Tag::Byte(value) => writer.write_all(&[*value as u8]),
            Tag::Short(value) => writer.write_all(&value.to_be_bytes()),
            Tag::Int(value) => writer.write_all(&value.to_be_bytes()),
            Tag::Long(value) => writer.write_all(&value.to_be_bytes()),
            Tag::Float(value) => writer.write_all(&value.to_bits().to_be_bytes()),
            Tag::Double(value) => writer.write_all(&value.to_bits().to_be_bytes()),
            Tag::ByteArray(values) => {
                write_len_i32(writer, values.len())?;
                for value in values {
                    writer.write_all(&[*value as u8])?;
                }
                Ok(())
            }
            Tag::String(value) => write_string(writer, value),
            Tag::List(values) => {
                let element_id = values.first().map(Tag::id).unwrap_or(0);
                if values.iter().any(|value| value.id() != element_id) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "NBT lists must contain one tag type",
                    ));
                }
                writer.write_all(&[element_id])?;
                write_len_i32(writer, values.len())?;
                for value in values {
                    value.write_payload(writer)?;
                }
                Ok(())
            }
            Tag::Compound(values) => {
                for (name, value) in values {
                    writer.write_all(&[value.id()])?;
                    write_string(writer, name)?;
                    value.write_payload(writer)?;
                }
                writer.write_all(&[0])
            }
            Tag::IntArray(values) => {
                write_len_i32(writer, values.len())?;
                for value in values {
                    writer.write_all(&value.to_be_bytes())?;
                }
                Ok(())
            }
            Tag::LongArray(values) => {
                write_len_i32(writer, values.len())?;
                for value in values {
                    writer.write_all(&value.to_be_bytes())?;
                }
                Ok(())
            }
        }
    }

    pub fn payload_size(&self) -> usize {
        match self {
            Tag::End => 0,
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) | Tag::Float(_) => 4,
            Tag::Long(_) | Tag::Double(_) => 8,
            Tag::ByteArray(values) => 4 + values.len(),
            Tag::String(value) => 2 + value.len(),
            Tag::List(values) => 5 + values.iter().map(Tag::payload_size).sum::<usize>(),
            Tag::Compound(values) => {
                1 + values
                    .iter()
                    .map(|(name, value)| 1 + 2 + name.len() + value.payload_size())
                    .sum::<usize>()
            }
            Tag::IntArray(values) => 4 + values.len() * 4,
            Tag::LongArray(values) => 4 + values.len() * 8,
        }
    }

    pub fn tracked_size(&self) -> NbtSizeTracker {
        fn walk(tag: &Tag, depth: usize, tracker: &mut NbtSizeTracker) {
            tracker.payload_bytes += tag.payload_size();
            tracker.nodes += 1;
            tracker.max_depth = tracker.max_depth.max(depth);
            match tag {
                Tag::List(values) => {
                    for value in values {
                        walk(value, depth + 1, tracker);
                    }
                }
                Tag::Compound(values) => {
                    for (_name, value) in values {
                        walk(value, depth + 1, tracker);
                    }
                }
                _ => {}
            }
        }

        let mut tracker = NbtSizeTracker {
            payload_bytes: 0,
            nodes: 0,
            max_depth: 0,
        };
        walk(self, 0, &mut tracker);
        tracker
    }

    pub fn to_snbt(&self) -> String {
        match self {
            Tag::End => "END".to_string(),
            Tag::Byte(value) => format!("{value}b"),
            Tag::Short(value) => format!("{value}s"),
            Tag::Int(value) => value.to_string(),
            Tag::Long(value) => format!("{value}l"),
            Tag::Float(value) => format!("{value}f"),
            Tag::Double(value) => format!("{value}d"),
            Tag::ByteArray(values) => format!(
                "[B;{}]",
                values
                    .iter()
                    .map(|value| format!("{value}b"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Tag::String(value) => format!("\"{}\"", escape_snbt_string(value)),
            Tag::List(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Tag::to_snbt)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Tag::Compound(values) => format!(
                "{{{}}}",
                values
                    .iter()
                    .map(|(name, value)| format!("{}:{}", quote_snbt_key(name), value.to_snbt()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Tag::IntArray(values) => format!(
                "[I;{}]",
                values
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Tag::LongArray(values) => format!(
                "[L;{}]",
                values
                    .iter()
                    .map(|value| format!("{value}l"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }

    pub fn visit_depth_first<F>(&self, visitor: &mut F)
    where
        F: FnMut(&Tag),
    {
        visitor(self);
        match self {
            Tag::List(values) => {
                for value in values {
                    value.visit_depth_first(visitor);
                }
            }
            Tag::Compound(values) => {
                for (_name, value) in values {
                    value.visit_depth_first(visitor);
                }
            }
            _ => {}
        }
    }

    pub fn select<'a>(&'a self, selector: &NbtFieldSelector) -> Option<&'a Tag> {
        let mut current = self;
        for part in &selector.path {
            match current {
                Tag::Compound(values) => {
                    current = &values.iter().find(|(name, _)| name == part)?.1;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    pub fn visit_selected_fields<'a, F>(&'a self, selectors: &'a [NbtFieldSelector], mut visitor: F)
    where
        F: FnMut(&'a NbtFieldSelector, &'a Tag),
    {
        for selector in selectors {
            if let Some(tag) = self.select(selector) {
                visitor(selector, tag);
            }
        }
    }
}

impl NbtFieldSelector {
    pub fn dotted(path: &str) -> Self {
        Self {
            path: path
                .split('.')
                .filter(|part| !part.is_empty())
                .map(ToString::to_string)
                .collect(),
        }
    }
}

pub fn read_named_tag<R: Read>(reader: &mut R) -> io::Result<(String, Tag)> {
    let id = read_u8(reader)?;
    if id == 0 {
        return Ok((String::new(), Tag::End));
    }
    let name = read_string(reader)?;
    let payload = Tag::read_payload(id, reader)?;
    Ok((name, payload))
}

pub fn read_named_tag_limited<R: Read>(
    reader: &mut R,
    max_depth: usize,
) -> io::Result<(String, Tag)> {
    let id = read_u8(reader)?;
    if id == 0 {
        return Ok((String::new(), Tag::End));
    }
    let name = read_string(reader)?;
    let payload = Tag::read_payload_limited(id, reader, max_depth)?;
    Ok((name, payload))
}

pub fn write_named_tag<W: Write>(writer: &mut W, name: &str, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        write_string(writer, name)?;
        tag.write_payload(writer)?;
    }
    Ok(())
}

pub fn read_gzip_named_tag<R: Read>(reader: R) -> io::Result<(String, Tag)> {
    read_named_tag(&mut GzDecoder::new(reader))
}

pub fn write_gzip_named_tag<W: Write>(writer: W, name: &str, tag: &Tag) -> io::Result<()> {
    let mut encoder = GzEncoder::new(writer, Compression::default());
    write_named_tag(&mut encoder, name, tag)?;
    encoder.finish()?;
    Ok(())
}

pub fn parse_snbt(input: &str) -> io::Result<Tag> {
    let mut parser = SnbtParser::new(input);
    let tag = parser.parse_tag()?;
    parser.skip_ws();
    if parser.peek().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "trailing SNBT input",
        ));
    }
    Ok(tag)
}

fn quote_snbt_key(key: &str) -> String {
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '+'))
    {
        key.to_string()
    } else {
        format!("\"{}\"", escape_snbt_string(key))
    }
}

fn escape_snbt_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

struct SnbtParser<'a> {
    input: &'a str,
    cursor: usize,
}

impl<'a> SnbtParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    fn parse_tag(&mut self) -> io::Result<Tag> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.parse_compound(),
            Some('[') => self.parse_array_or_list(),
            Some('"') | Some('\'') => Ok(Tag::String(self.parse_quoted_string()?)),
            Some(_) => self.parse_primitive(),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "empty SNBT input",
            )),
        }
    }

    fn parse_compound(&mut self) -> io::Result<Tag> {
        self.expect('{')?;
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume('}') {
                break;
            }
            let name = self.parse_key()?;
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_tag()?;
            values.push((name, value));
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            self.expect('}')?;
            break;
        }
        Ok(Tag::Compound(values))
    }

    fn parse_array_or_list(&mut self) -> io::Result<Tag> {
        self.expect('[')?;
        self.skip_ws();
        if self.peek_type_array_prefix('B') {
            self.cursor += 2;
            return self.parse_byte_array();
        }
        if self.peek_type_array_prefix('I') {
            self.cursor += 2;
            return self.parse_int_array();
        }
        if self.peek_type_array_prefix('L') {
            self.cursor += 2;
            return self.parse_long_array();
        }

        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            values.push(self.parse_tag()?);
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(Tag::List(values))
    }

    fn parse_byte_array(&mut self) -> io::Result<Tag> {
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            match self.parse_primitive()? {
                Tag::Byte(value) => values.push(value),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "SNBT byte array values must be bytes",
                    ))
                }
            }
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(Tag::ByteArray(values))
    }

    fn parse_int_array(&mut self) -> io::Result<Tag> {
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            match self.parse_primitive()? {
                Tag::Int(value) => values.push(value),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "SNBT int array values must be ints",
                    ))
                }
            }
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(Tag::IntArray(values))
    }

    fn parse_long_array(&mut self) -> io::Result<Tag> {
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            match self.parse_primitive()? {
                Tag::Long(value) => values.push(value),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "SNBT long array values must be longs",
                    ))
                }
            }
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            self.expect(']')?;
            break;
        }
        Ok(Tag::LongArray(values))
    }

    fn parse_key(&mut self) -> io::Result<String> {
        self.skip_ws();
        match self.peek() {
            Some('"') | Some('\'') => self.parse_quoted_string(),
            Some(_) => self.parse_unquoted_token(),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing SNBT key",
            )),
        }
    }

    fn parse_primitive(&mut self) -> io::Result<Tag> {
        let token = self.parse_unquoted_token()?;
        if token.eq_ignore_ascii_case("true") {
            return Ok(Tag::Byte(1));
        }
        if token.eq_ignore_ascii_case("false") {
            return Ok(Tag::Byte(0));
        }
        if token.eq_ignore_ascii_case("END") {
            return Ok(Tag::End);
        }

        let suffix = token.chars().last().unwrap_or_default();
        let number = if matches!(
            suffix,
            'b' | 'B' | 's' | 'S' | 'l' | 'L' | 'f' | 'F' | 'd' | 'D'
        ) {
            &token[..token.len() - suffix.len_utf8()]
        } else {
            token.as_str()
        };
        match suffix {
            'b' | 'B' => number
                .parse::<i8>()
                .map(Tag::Byte)
                .map_err(invalid_snbt_number),
            's' | 'S' => number
                .parse::<i16>()
                .map(Tag::Short)
                .map_err(invalid_snbt_number),
            'l' | 'L' => number
                .parse::<i64>()
                .map(Tag::Long)
                .map_err(invalid_snbt_number),
            'f' | 'F' => number
                .parse::<f32>()
                .map(Tag::Float)
                .map_err(invalid_snbt_number),
            'd' | 'D' => number
                .parse::<f64>()
                .map(Tag::Double)
                .map_err(invalid_snbt_number),
            _ => token
                .parse::<i32>()
                .map(Tag::Int)
                .or_else(|_| token.parse::<f64>().map(Tag::Double))
                .or_else(|_| Ok(Tag::String(token))),
        }
    }

    fn parse_quoted_string(&mut self) -> io::Result<String> {
        let quote = self.next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing SNBT string quote")
        })?;
        let mut output = String::new();
        loop {
            match self.next() {
                Some(ch) if ch == quote => break,
                Some('\\') => match self.next() {
                    Some(escaped) => output.push(escaped),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "unterminated SNBT escape",
                        ))
                    }
                },
                Some(ch) => output.push(ch),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unterminated SNBT string",
                    ))
                }
            }
        }
        Ok(output)
    }

    fn parse_unquoted_token(&mut self) -> io::Result<String> {
        self.skip_ws();
        let start = self.cursor;
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || matches!(ch, ':' | ',' | ']' | '}') {
                break;
            }
            self.next();
        }
        if self.cursor == start {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "missing SNBT token",
            ));
        }
        Ok(self.input[start..self.cursor].to_string())
    }

    fn peek_type_array_prefix(&self, prefix: char) -> bool {
        self.input[self.cursor..].starts_with(prefix)
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
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("expected SNBT character {expected}"),
            ))
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

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }
}

fn invalid_snbt_number(err: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}

fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_i8<R: Read>(reader: &mut R) -> io::Result<i8> {
    Ok(read_u8(reader)? as i8)
}

fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

fn read_u64<R: Read>(reader: &mut R) -> io::Result<u64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_be_bytes(bytes))
}

fn read_len_i32<R: Read>(reader: &mut R) -> io::Result<usize> {
    let len = read_i32(reader)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative NBT length",
        ));
    }
    Ok(len as usize)
}

fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = read_i16(reader)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative NBT string length",
        ));
    }
    let mut bytes = vec![0u8; len as usize];
    reader.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let len = i16::try_from(value.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT string too long"))?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(value.as_bytes())
}

fn write_len_i32<W: Write>(writer: &mut W, len: usize) -> io::Result<()> {
    let len = i32::try_from(len)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT collection too long"))?;
    writer.write_all(&len.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        parse_snbt, read_gzip_named_tag, read_named_tag, read_named_tag_limited,
        write_gzip_named_tag, write_named_tag, NbtFieldSelector, Tag,
    };
    use std::io::Cursor;

    #[test]
    fn exposes_all_vanilla_tag_ids() {
        assert_eq!(Tag::End.id(), 0);
        assert_eq!(Tag::Byte(0).id(), 1);
        assert_eq!(Tag::Short(0).id(), 2);
        assert_eq!(Tag::Int(0).id(), 3);
        assert_eq!(Tag::Long(0).id(), 4);
        assert_eq!(Tag::Float(0.0).id(), 5);
        assert_eq!(Tag::Double(0.0).id(), 6);
        assert_eq!(Tag::ByteArray(vec![]).id(), 7);
        assert_eq!(Tag::String(String::new()).id(), 8);
        assert_eq!(Tag::List(vec![]).id(), 9);
        assert_eq!(Tag::Compound(vec![]).id(), 10);
        assert_eq!(Tag::IntArray(vec![]).id(), 11);
        assert_eq!(Tag::LongArray(vec![]).id(), 12);
    }

    #[test]
    fn round_trips_named_compound_with_nested_values() {
        let tag = Tag::Compound(vec![
            ("name".to_string(), Tag::String("RustCraft".to_string())),
            ("health".to_string(), Tag::Float(20.0)),
            (
                "pos".to_string(),
                Tag::List(vec![Tag::Double(1.0), Tag::Double(2.0)]),
            ),
            ("ints".to_string(), Tag::IntArray(vec![1, 2, 3])),
            ("longs".to_string(), Tag::LongArray(vec![4, 5, 6])),
        ]);

        let mut bytes = Vec::new();
        write_named_tag(&mut bytes, "root", &tag).unwrap();
        let (name, decoded) = read_named_tag(&mut Cursor::new(bytes)).unwrap();

        assert_eq!(name, "root");
        assert_eq!(decoded, tag);
    }

    #[test]
    fn rejects_mixed_type_lists() {
        let mut bytes = Vec::new();
        let err = Tag::List(vec![Tag::Int(1), Tag::String("bad".to_string())])
            .write_payload(&mut bytes)
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn compressed_nbt_round_trips_named_tags() {
        let tag = Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4790))]);
        let mut bytes = Vec::new();
        write_gzip_named_tag(&mut bytes, "level", &tag).unwrap();

        assert_eq!(&bytes[..2], &[0x1f, 0x8b]);
        let (name, decoded) = read_gzip_named_tag(Cursor::new(bytes)).unwrap();
        assert_eq!(name, "level");
        assert_eq!(decoded, tag);
    }

    #[test]
    fn snbt_printer_size_accounting_and_traversal_cover_nested_tags() {
        let tag = Tag::Compound(vec![
            (
                "name".to_string(),
                Tag::String("A \"quoted\" name".to_string()),
            ),
            ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
            (
                "nested".to_string(),
                Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
            ),
        ]);

        assert_eq!(
            tag.to_snbt(),
            "{name:\"A \\\"quoted\\\" name\",bytes:[B;1b,2b],nested:[1,2]}"
        );
        assert_eq!(tag.payload_size(), 61);

        let mut ids = Vec::new();
        tag.visit_depth_first(&mut |visited| ids.push(visited.id()));
        assert_eq!(ids, vec![10, 8, 7, 9, 3, 3]);
    }

    #[test]
    fn bounded_nbt_reads_reject_excessive_recursion() {
        let tag = Tag::Compound(vec![(
            "outer".to_string(),
            Tag::Compound(vec![("inner".to_string(), Tag::Int(1))]),
        )]);
        let mut bytes = Vec::new();
        write_named_tag(&mut bytes, "root", &tag).unwrap();

        let err = read_named_tag_limited(&mut Cursor::new(bytes), 1).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("depth limit"));
    }

    #[test]
    fn nbt_size_tracker_and_field_selectors_cover_streaming_use_cases() {
        let tag = Tag::Compound(vec![
            (
                "Data".to_string(),
                Tag::Compound(vec![
                    ("DataVersion".to_string(), Tag::Int(4790)),
                    ("LevelName".to_string(), Tag::String("world".to_string())),
                ]),
            ),
            ("Other".to_string(), Tag::Byte(1)),
        ]);

        let tracked = tag.tracked_size();
        assert_eq!(tracked.nodes, 5);
        assert_eq!(tracked.max_depth, 2);
        assert!(tracked.payload_bytes >= tag.payload_size());

        let selectors = [
            NbtFieldSelector::dotted("Data.DataVersion"),
            NbtFieldSelector::dotted("Data.Missing"),
            NbtFieldSelector::dotted("Other"),
        ];
        let mut selected = Vec::new();
        tag.visit_selected_fields(&selectors, |selector, value| {
            selected.push((selector.path.join("."), value.clone()));
        });
        assert_eq!(
            selected,
            vec![
                ("Data.DataVersion".to_string(), Tag::Int(4790)),
                ("Other".to_string(), Tag::Byte(1)),
            ]
        );
    }

    #[test]
    fn snbt_parser_round_trips_printer_shapes_and_reports_errors() {
        let tag = Tag::Compound(vec![
            (
                "name".to_string(),
                Tag::String("A \"quoted\" name".to_string()),
            ),
            ("bytes".to_string(), Tag::ByteArray(vec![1, 2])),
            ("ints".to_string(), Tag::IntArray(vec![3, 4])),
            ("longs".to_string(), Tag::LongArray(vec![5, 6])),
            (
                "nested".to_string(),
                Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
            ),
            ("enabled".to_string(), Tag::Byte(1)),
        ]);

        let printed = tag.to_snbt();
        assert_eq!(parse_snbt(&printed).unwrap(), tag);
        assert_eq!(parse_snbt("{flag:true}").unwrap().to_snbt(), "{flag:1b}");
        assert_eq!(parse_snbt("12s").unwrap(), Tag::Short(12));
        assert_eq!(parse_snbt("3.5f").unwrap(), Tag::Float(3.5));
        assert!(parse_snbt("{broken")
            .unwrap_err()
            .to_string()
            .contains("expected"));
    }
}
