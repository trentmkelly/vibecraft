#![cfg_attr(
    all(test, not(vibecraft_has_decompiled_sources)),
    allow(dead_code)
)]

use std::io::{self, Read, Write};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

pub mod accounter;
pub mod compound_tag;
pub mod nbt_io;
pub mod nbt_ops;
pub mod nbt_utils;
pub mod numeric;
pub mod snbt_grammar;
pub mod snbt_operations;
pub mod snbt_printer;
pub mod snbt_string;
pub mod tag_access;
pub mod tag_metadata;
pub mod tag_parser;
pub mod text_component_tag_visitor;

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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NbtSizeTracker {
    pub payload_bytes: usize,
    pub nodes: usize,
    pub max_depth: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtFieldSelector {
    pub path: Vec<String>,
}

impl Tag {
    #[allow(dead_code)]
    pub fn numeric_value(&self) -> Option<numeric::NbtNumericValue> {
        match self {
            Tag::Byte(value) => Some(numeric::NbtNumericValue::Byte(*value)),
            Tag::Short(value) => Some(numeric::NbtNumericValue::Short(*value)),
            Tag::Int(value) => Some(numeric::NbtNumericValue::Int(*value)),
            Tag::Long(value) => Some(numeric::NbtNumericValue::Long(*value)),
            Tag::Float(value) => Some(numeric::NbtNumericValue::Float(*value)),
            Tag::Double(value) => Some(numeric::NbtNumericValue::Double(*value)),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Tag::Byte(_)
                | Tag::Short(_)
                | Tag::Int(_)
                | Tag::Long(_)
                | Tag::Float(_)
                | Tag::Double(_)
                | Tag::String(_)
        )
    }

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
                if element_id == 0 && len > 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Missing type on ListTag",
                    ));
                }
                let mut values = Vec::with_capacity(len);
                for _ in 0..len {
                    let value =
                        Tag::read_payload_at_depth(element_id, reader, depth + 1, max_depth)?;
                    values.push(unwrap_list_element(value));
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
                    compound_tag::put_compound_entry(&mut values, name, payload);
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
                let element_id = identify_list_raw_element_type(values);
                writer.write_all(&[element_id])?;
                write_len_i32(writer, values.len())?;
                for value in values {
                    wrap_list_element_if_needed(element_id, value).write_payload(writer)?;
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

    #[cfg(test)]
    pub fn payload_size(&self) -> usize {
        match self {
            Tag::End => 0,
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) | Tag::Float(_) => 4,
            Tag::Long(_) | Tag::Double(_) => 8,
            Tag::ByteArray(values) => 4 + values.len(),
            Tag::String(value) => 2 + modified_utf8_len(value),
            Tag::List(values) => 5 + values.iter().map(Tag::payload_size).sum::<usize>(),
            Tag::Compound(values) => {
                1 + values
                    .iter()
                    .map(|(name, value)| 1 + 2 + modified_utf8_len(name) + value.payload_size())
                    .sum::<usize>()
            }
            Tag::IntArray(values) => 4 + values.len() * 4,
            Tag::LongArray(values) => 4 + values.len() * 8,
        }
    }

    #[cfg(test)]
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
            // Suffix casing matches Java StringTagVisitor: scalar byte 'b'/short
            // 's'/long 'L'/float 'f'/double 'd', and array elements use uppercase
            // 'B'/'L' (byte/long arrays).
            // TODO(snbt-float-formatting-parity): Float/Double rendering is not yet
            // 1:1 — Java uses Float.toString/Double.toString (always keeps a
            // decimal point, e.g. "1.0f"), while Rust's `{}` prints "1f". Matching
            // requires a Java-style shortest float/double formatter.
            Tag::End => "END".to_string(),
            Tag::Byte(value) => format!("{value}b"),
            Tag::Short(value) => format!("{value}s"),
            Tag::Int(value) => value.to_string(),
            Tag::Long(value) => format!("{value}L"),
            Tag::Float(value) => format!("{value}f"),
            Tag::Double(value) => format!("{value}d"),
            Tag::ByteArray(values) => format!(
                "[B;{}]",
                values
                    .iter()
                    .map(|value| format!("{value}B"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Tag::String(value) => snbt_string::quote_and_escape_snbt_string(value),
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
                sorted_compound_entries(values)
                    .into_iter()
                    .map(|(name, value)| {
                        format!("{}:{}", snbt_string::quote_snbt_key(name), value.to_snbt())
                    })
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
                    .map(|value| format!("{value}L"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }

    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
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

    #[cfg(test)]
    pub fn to_text_component(
        &self,
        indentation: &str,
        sort_keys: bool,
    ) -> crate::chat_component::Component {
        text_component_tag_visitor::to_plain_text_component(self, indentation, sort_keys)
    }

    #[cfg(test)]
    pub fn to_text_component_plain(&self, indentation: &str, sort_keys: bool) -> String {
        text_component_tag_visitor::to_plain_text(self, indentation, sort_keys)
    }
}

fn sorted_compound_entries(values: &[(String, Tag)]) -> Vec<(&str, &Tag)> {
    let mut entries = values
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    entries
}

fn identify_list_raw_element_type(values: &[Tag]) -> u8 {
    let mut element_type = 0;
    for value in values {
        let value_type = value.id();
        if element_type == 0 {
            element_type = value_type;
        } else if element_type != value_type {
            return 10;
        }
    }
    element_type
}

fn wrap_list_element_if_needed(element_type: u8, value: &Tag) -> Tag {
    if element_type != 10 {
        return value.clone();
    }
    match value {
        Tag::Compound(entries) if !is_list_wrapper(entries) => value.clone(),
        _ => Tag::Compound(vec![("".to_string(), value.clone())]),
    }
}

fn unwrap_list_element(value: Tag) -> Tag {
    match value {
        Tag::Compound(mut entries) if is_list_wrapper(&entries) => entries.remove(0).1,
        value => value,
    }
}

fn is_list_wrapper(entries: &[(String, Tag)]) -> bool {
    entries.len() == 1 && entries[0].0.is_empty()
}

#[cfg(test)]
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

#[cfg(test)]
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
    snbt_grammar::parse_snbt(input)
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
    // NBT strings are Java "modified UTF-8" (DataInput.readUTF): an UNSIGNED u16
    // byte-length prefix followed by modified-UTF-8 bytes.
    let len = read_u16(reader)?;
    let mut bytes = vec![0u8; usize::from(len)];
    reader.read_exact(&mut bytes)?;
    decode_modified_utf8(&bytes)
}

fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let bytes = encode_modified_utf8(value);
    let len = u16::try_from(bytes.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT string too long"))?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&bytes)
}

fn read_u16<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_be_bytes(bytes))
}

fn modified_utf8_err() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid modified UTF-8 in NBT string",
    )
}

/// Encode a string as Java "modified UTF-8" (1:1 with `DataOutput.writeUTF`),
/// iterating over UTF-16 code units: U+0001..=U+007F → 1 byte; U+0000 and
/// U+0080..=U+07FF → 2 bytes; U+0800..=U+FFFF (including surrogates) → 3 bytes —
/// so a supplementary char becomes a 6-byte surrogate pair.
fn encode_modified_utf8(value: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for unit in value.encode_utf16() {
        if (0x0001..=0x007F).contains(&unit) {
            out.push(unit as u8);
        } else if unit == 0 || (0x0080..=0x07FF).contains(&unit) {
            out.push(0xC0 | (unit >> 6) as u8);
            out.push(0x80 | (unit as u8 & 0x3F));
        } else {
            out.push(0xE0 | (unit >> 12) as u8);
            out.push(0x80 | ((unit >> 6) as u8 & 0x3F));
            out.push(0x80 | (unit as u8 & 0x3F));
        }
    }
    out
}

/// Byte length of the modified-UTF-8 encoding without allocating.
#[cfg(test)]
fn modified_utf8_len(value: &str) -> usize {
    value
        .encode_utf16()
        .map(|unit| {
            if (0x0001..=0x007F).contains(&unit) {
                1
            } else if unit == 0 || (0x0080..=0x07FF).contains(&unit) {
                2
            } else {
                3
            }
        })
        .sum()
}

/// Decode Java "modified UTF-8" (1:1 with `DataInput.readUTF`) into a String,
/// reconstructing supplementary chars from UTF-16 surrogate pairs.
fn decode_modified_utf8(bytes: &[u8]) -> io::Result<String> {
    let mut units = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let a = bytes[i];
        if a & 0x80 == 0 {
            units.push(u16::from(a));
            i += 1;
        } else if a & 0xE0 == 0xC0 {
            let b = *bytes.get(i + 1).ok_or_else(modified_utf8_err)?;
            if b & 0xC0 != 0x80 {
                return Err(modified_utf8_err());
            }
            units.push((u16::from(a & 0x1F) << 6) | u16::from(b & 0x3F));
            i += 2;
        } else if a & 0xF0 == 0xE0 {
            let b = *bytes.get(i + 1).ok_or_else(modified_utf8_err)?;
            let c = *bytes.get(i + 2).ok_or_else(modified_utf8_err)?;
            if b & 0xC0 != 0x80 || c & 0xC0 != 0x80 {
                return Err(modified_utf8_err());
            }
            units.push(
                (u16::from(a & 0x0F) << 12) | (u16::from(b & 0x3F) << 6) | u16::from(c & 0x3F),
            );
            i += 3;
        } else {
            return Err(modified_utf8_err());
        }
    }
    String::from_utf16(&units).map_err(|_| modified_utf8_err())
}

fn write_len_i32<W: Write>(writer: &mut W, len: usize) -> io::Result<()> {
    let len = i32::try_from(len)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT collection too long"))?;
    writer.write_all(&len.to_be_bytes())
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests;
#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests_visitors;
