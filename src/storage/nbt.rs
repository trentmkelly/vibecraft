use std::io::{self, Read, Write};

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
                    values.push(Tag::read_payload(element_id, reader)?);
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
                    let payload = Tag::read_payload(child_id, reader)?;
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

pub fn write_named_tag<W: Write>(writer: &mut W, name: &str, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        write_string(writer, name)?;
        tag.write_payload(writer)?;
    }
    Ok(())
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
    use super::{read_named_tag, write_named_tag, Tag};
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
}
