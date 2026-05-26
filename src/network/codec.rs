#![allow(dead_code)]

use std::io::{self, Cursor, Read, Write};

use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;
use crate::storage::nbt::{read_named_tag, write_named_tag, Tag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(pub [u8; 16]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentJson(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryValueId(pub i32);

pub fn read_string<R: Read>(reader: &mut R, max_chars: usize) -> io::Result<String> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_chars * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid string length",
        ));
    }

    let mut bytes = vec![0u8; length as usize];
    reader.read_exact(&mut bytes)?;
    let string =
        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if string.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    Ok(string)
}

pub fn write_string<W: Write>(writer: &mut W, value: &str, max_chars: usize) -> io::Result<()> {
    if value.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "string too long",
        ));
    }
    write_var_i32(writer, value.len() as i32)?;
    writer.write_all(value.as_bytes())
}

pub fn read_identifier<R: Read>(reader: &mut R) -> io::Result<Identifier> {
    Identifier::parse(&read_string(reader, 32767)?)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub fn write_identifier<W: Write>(writer: &mut W, value: &Identifier) -> io::Result<()> {
    write_string(writer, &value.to_string(), 32767)
}

pub fn read_uuid<R: Read>(reader: &mut R) -> io::Result<Uuid> {
    let mut bytes = [0u8; 16];
    reader.read_exact(&mut bytes)?;
    Ok(Uuid(bytes))
}

pub fn write_uuid<W: Write>(writer: &mut W, value: Uuid) -> io::Result<()> {
    writer.write_all(&value.0)
}

pub fn read_optional<R, T, F>(reader: &mut R, read: F) -> io::Result<Option<T>>
where
    R: Read,
    F: FnOnce(&mut R) -> io::Result<T>,
{
    let mut present = [0u8; 1];
    reader.read_exact(&mut present)?;
    match present[0] {
        0 => Ok(None),
        1 => Ok(Some(read(reader)?)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid optional marker",
        )),
    }
}

pub fn write_optional<W, T, F>(writer: &mut W, value: Option<&T>, write: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut W, &T) -> io::Result<()>,
{
    match value {
        Some(value) => {
            writer.write_all(&[1])?;
            write(writer, value)
        }
        None => writer.write_all(&[0]),
    }
}

pub fn read_collection<R, T, F>(reader: &mut R, mut read: F) -> io::Result<Vec<T>>
where
    R: Read,
    F: FnMut(&mut R) -> io::Result<T>,
{
    let len = read_var_i32(reader)?;
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative collection length",
        ));
    }
    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        values.push(read(reader)?);
    }
    Ok(values)
}

pub fn write_collection<W, T, F>(writer: &mut W, values: &[T], mut write: F) -> io::Result<()>
where
    W: Write,
    F: FnMut(&mut W, &T) -> io::Result<()>,
{
    write_var_i32(writer, values.len() as i32)?;
    for value in values {
        write(writer, value)?;
    }
    Ok(())
}

pub fn read_enum_index<R: Read>(reader: &mut R, variant_count: usize) -> io::Result<usize> {
    let index = read_var_i32(reader)?;
    if index < 0 || index as usize >= variant_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "enum index out of range",
        ));
    }
    Ok(index as usize)
}

pub fn write_enum_index<W: Write>(
    writer: &mut W,
    index: usize,
    variant_count: usize,
) -> io::Result<()> {
    if index >= variant_count {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "enum index out of range",
        ));
    }
    write_var_i32(writer, index as i32)
}

pub fn read_bitset<R: Read>(reader: &mut R) -> io::Result<Vec<u64>> {
    read_collection(reader, |reader| {
        let mut bytes = [0u8; 8];
        reader.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
    })
}

pub fn write_bitset<W: Write>(writer: &mut W, values: &[u64]) -> io::Result<()> {
    write_collection(writer, values, |writer, value| {
        writer.write_all(&value.to_be_bytes())
    })
}

pub fn read_nbt<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let (_name, tag) = read_named_tag(reader)?;
    Ok(tag)
}

pub fn write_nbt<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    write_named_tag(writer, "", tag)
}

pub fn read_component<R: Read>(reader: &mut R) -> io::Result<ComponentJson> {
    Ok(ComponentJson(read_string(reader, 262144)?))
}

pub fn write_component<W: Write>(writer: &mut W, component: &ComponentJson) -> io::Result<()> {
    write_string(writer, &component.0, 262144)
}

pub fn read_trusted_component<R: Read>(reader: &mut R) -> io::Result<ComponentJson> {
    let tag = read_network_nbt(reader)?;
    let value = component_tag_to_json(&tag);
    serde_json::to_string(&value)
        .map(ComponentJson)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub fn write_trusted_component<W: Write>(
    writer: &mut W,
    component: &ComponentJson,
) -> io::Result<()> {
    let tag = component_json_to_network_tag(&component.0);
    write_network_nbt(writer, &tag)
}

fn read_network_nbt<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let mut id = [0u8; 1];
    reader.read_exact(&mut id)?;
    if id[0] == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected non-null NBT tag",
        ));
    }
    Tag::read_payload(id[0], reader)
}

fn write_network_nbt<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    if matches!(tag, Tag::End) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected non-null NBT tag",
        ));
    }
    writer.write_all(&[tag.id()])?;
    tag.write_payload(writer)
}

fn component_json_to_network_tag(json: &str) -> Tag {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Tag::String(json.to_string());
    };

    if let Some(text) = collapsible_literal_text(&value) {
        return Tag::String(text.to_string());
    }

    json_value_to_nbt(&value).unwrap_or_else(|| Tag::String(json.to_string()))
}

fn collapsible_literal_text(value: &serde_json::Value) -> Option<&str> {
    let serde_json::Value::Object(object) = value else {
        return value.as_str();
    };
    if object.len() == 1 {
        object.get("text").and_then(serde_json::Value::as_str)
    } else {
        None
    }
}

fn json_value_to_nbt(value: &serde_json::Value) -> Option<Tag> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(value) => Some(Tag::Byte(i8::from(*value))),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                i32::try_from(value)
                    .map(Tag::Int)
                    .ok()
                    .or(Some(Tag::Long(value)))
            } else {
                value.as_f64().map(Tag::Double)
            }
        }
        serde_json::Value::String(value) => Some(Tag::String(value.clone())),
        serde_json::Value::Array(values) => {
            let mut tags = Vec::with_capacity(values.len());
            for value in values {
                tags.push(json_value_to_nbt(value)?);
            }
            Some(Tag::List(tags))
        }
        serde_json::Value::Object(object) => Some(Tag::Compound(
            object
                .iter()
                .filter_map(|(key, value)| {
                    json_value_to_nbt(value).map(|value| (key.clone(), value))
                })
                .collect(),
        )),
    }
}

fn component_tag_to_json(tag: &Tag) -> serde_json::Value {
    match tag {
        Tag::String(value) => {
            let mut object = serde_json::Map::new();
            object.insert(
                "text".to_string(),
                serde_json::Value::String(value.to_string()),
            );
            serde_json::Value::Object(object)
        }
        Tag::Compound(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), nbt_tag_to_json(value)))
                .collect(),
        ),
        other => nbt_tag_to_json(other),
    }
}

fn nbt_tag_to_json(tag: &Tag) -> serde_json::Value {
    match tag {
        Tag::End => serde_json::Value::Null,
        Tag::Byte(value) => serde_json::Value::Bool(*value != 0),
        Tag::Short(value) => serde_json::Value::Number(i64::from(*value).into()),
        Tag::Int(value) => serde_json::Value::Number(i64::from(*value).into()),
        Tag::Long(value) => serde_json::Value::Number((*value).into()),
        Tag::Float(value) => serde_json::Number::from_f64(f64::from(*value))
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Tag::Double(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Tag::ByteArray(values) => serde_json::Value::Array(
            values
                .iter()
                .map(|value| serde_json::Value::Number(i64::from(*value).into()))
                .collect(),
        ),
        Tag::String(value) => serde_json::Value::String(value.clone()),
        Tag::List(values) => serde_json::Value::Array(values.iter().map(nbt_tag_to_json).collect()),
        Tag::Compound(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), nbt_tag_to_json(value)))
                .collect(),
        ),
        Tag::IntArray(values) => serde_json::Value::Array(
            values
                .iter()
                .map(|value| serde_json::Value::Number(i64::from(*value).into()))
                .collect(),
        ),
        Tag::LongArray(values) => serde_json::Value::Array(
            values
                .iter()
                .map(|value| serde_json::Value::Number((*value).into()))
                .collect(),
        ),
    }
}

pub fn read_registry_value_id<R: Read>(reader: &mut R) -> io::Result<RegistryValueId> {
    Ok(RegistryValueId(read_var_i32(reader)?))
}

pub fn write_registry_value_id<W: Write>(writer: &mut W, value: RegistryValueId) -> io::Result<()> {
    write_var_i32(writer, value.0)
}

pub fn packet_payload<F>(write: F) -> io::Result<Vec<u8>>
where
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write(&mut payload)?;
    Ok(payload)
}

pub fn cursor(bytes: Vec<u8>) -> Cursor<Vec<u8>> {
    Cursor::new(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_string_identifier_and_uuid() {
        let id = Identifier::parse("minecraft:stone").unwrap();
        let uuid = Uuid([1; 16]);
        let mut bytes = Vec::new();
        write_string(&mut bytes, "hello", 16).unwrap();
        write_identifier(&mut bytes, &id).unwrap();
        write_uuid(&mut bytes, uuid).unwrap();

        let mut input = cursor(bytes);
        assert_eq!(read_string(&mut input, 16).unwrap(), "hello");
        assert_eq!(read_identifier(&mut input).unwrap(), id);
        assert_eq!(read_uuid(&mut input).unwrap(), uuid);
    }

    #[test]
    fn round_trips_optional_collection_enum_and_bitset() {
        let mut bytes = Vec::new();
        write_optional(&mut bytes, Some(&7), |writer, value| {
            write_var_i32(writer, *value)
        })
        .unwrap();
        write_optional::<_, i32, _>(&mut bytes, None, |writer, value| {
            write_var_i32(writer, *value)
        })
        .unwrap();
        write_collection(&mut bytes, &[1, 2, 3], |writer, value| {
            write_var_i32(writer, *value)
        })
        .unwrap();
        write_enum_index(&mut bytes, 2, 3).unwrap();
        write_bitset(&mut bytes, &[0xFF, 0xAA]).unwrap();

        let mut input = cursor(bytes);
        assert_eq!(read_optional(&mut input, read_var_i32).unwrap(), Some(7));
        assert_eq!(read_optional(&mut input, read_var_i32).unwrap(), None);
        assert_eq!(
            read_collection(&mut input, read_var_i32).unwrap(),
            vec![1, 2, 3]
        );
        assert_eq!(read_enum_index(&mut input, 3).unwrap(), 2);
        assert_eq!(read_bitset(&mut input).unwrap(), vec![0xFF, 0xAA]);
    }

    #[test]
    fn round_trips_nbt_component_and_registry_value_id() {
        let mut bytes = Vec::new();
        write_nbt(
            &mut bytes,
            &Tag::Compound(vec![("x".to_string(), Tag::Int(1))]),
        )
        .unwrap();
        write_component(&mut bytes, &ComponentJson("{\"text\":\"hi\"}".to_string())).unwrap();
        write_registry_value_id(&mut bytes, RegistryValueId(42)).unwrap();

        let mut input = cursor(bytes);
        assert_eq!(
            read_nbt(&mut input).unwrap(),
            Tag::Compound(vec![("x".to_string(), Tag::Int(1))])
        );
        assert_eq!(
            read_component(&mut input).unwrap(),
            ComponentJson("{\"text\":\"hi\"}".to_string())
        );
        assert_eq!(
            read_registry_value_id(&mut input).unwrap(),
            RegistryValueId(42)
        );
    }
}
