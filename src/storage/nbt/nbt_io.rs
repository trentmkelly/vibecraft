#![allow(dead_code)]

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use super::tag_access::{NbtStreamTagVisitor, StreamEntryResult, StreamValueResult};
use super::tag_metadata::tag_type;
use super::{read_string, read_u8, write_string, Tag, DEFAULT_MAX_NBT_DEPTH};

pub fn read_compressed_path(path: &Path) -> io::Result<Tag> {
    read_compressed(fs::File::open(path)?)
}

pub fn read_compressed<R: Read>(reader: R) -> io::Result<Tag> {
    read_root_compound(&mut GzDecoder::new(reader))
}

pub fn parse_compressed_path<V: NbtStreamTagVisitor>(
    path: &Path,
    visitor: &mut V,
) -> io::Result<()> {
    parse_compressed(fs::File::open(path)?, visitor)
}

pub fn parse_compressed<R: Read, V: NbtStreamTagVisitor>(
    reader: R,
    visitor: &mut V,
) -> io::Result<()> {
    parse(&mut GzDecoder::new(reader), visitor)
}

pub fn write_compressed_path(tag: &Tag, path: &Path) -> io::Result<()> {
    let file = sync_output_file(path)?;
    write_compressed(tag, file)
}

pub fn write_compressed<W: Write>(tag: &Tag, writer: W) -> io::Result<()> {
    let mut encoder = GzEncoder::new(writer, Compression::default());
    write_root_compound(tag, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

pub fn write_path(tag: &Tag, path: &Path) -> io::Result<()> {
    write_root_compound(tag, &mut sync_output_file(path)?)
}

pub fn read_path(path: &Path) -> io::Result<Option<Tag>> {
    if !path.exists() {
        return Ok(None);
    }
    read_root_compound(&mut fs::File::open(path)?).map(Some)
}

pub fn read_root_compound<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let tag = read_unnamed_tag(reader)?;
    if matches!(tag, Tag::Compound(_)) {
        Ok(tag)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Root tag must be a named compound tag",
        ))
    }
}

pub fn write_root_compound<W: Write>(tag: &Tag, writer: &mut W) -> io::Result<()> {
    if !matches!(tag, Tag::Compound(_)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Root tag must be a named compound tag",
        ));
    }
    write_unnamed_tag_with_fallback(tag, writer)
}

pub fn parse<R: Read, V: NbtStreamTagVisitor>(reader: &mut R, visitor: &mut V) -> io::Result<()> {
    let id = read_u8(reader)?;
    if id == 0 {
        if visitor.visit_root_entry(tag_type(0)) == StreamValueResult::Continue {
            visitor.visit_end();
        }
        return Ok(());
    }

    let root_type = tag_type(i32::from(id));
    match visitor.visit_root_entry(root_type) {
        StreamValueResult::Halt => Ok(()),
        StreamValueResult::Break => {
            let _ = read_string(reader)?;
            Tag::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH).map(|_| ())
        }
        StreamValueResult::Continue => {
            let _ = read_string(reader)?;
            let tag = Tag::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH)?;
            visit_tag_payload(&tag, visitor).map(|_| ())
        }
    }
}

pub fn read_any_tag<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let id = read_u8(reader)?;
    if id == 0 {
        Ok(Tag::End)
    } else {
        Tag::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH)
    }
}

pub fn write_any_tag<W: Write>(tag: &Tag, writer: &mut W) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        tag.write_payload(writer)?;
    }
    Ok(())
}

pub fn read_unnamed_tag<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let id = read_u8(reader)?;
    if id == 0 {
        return Ok(Tag::End);
    }
    let _ = read_string(reader)?;
    Tag::read_payload_limited(id, reader, DEFAULT_MAX_NBT_DEPTH)
}

pub fn write_unnamed_tag<W: Write>(tag: &Tag, writer: &mut W) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        write_string(writer, "")?;
        tag.write_payload(writer)?;
    }
    Ok(())
}

pub fn write_unnamed_tag_with_fallback<W: Write>(tag: &Tag, writer: &mut W) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    if !matches!(tag, Tag::End) {
        write_string_with_fallback(writer, "")?;
        write_payload_with_string_fallback(tag, writer)?;
    }
    Ok(())
}

pub fn visit_tag_payload<V: NbtStreamTagVisitor>(
    tag: &Tag,
    visitor: &mut V,
) -> io::Result<StreamValueResult> {
    Ok(match tag {
        Tag::End => visitor.visit_end(),
        Tag::Byte(value) => visitor.visit_byte(*value),
        Tag::Short(value) => visitor.visit_short(*value),
        Tag::Int(value) => visitor.visit_int(*value),
        Tag::Long(value) => visitor.visit_long(*value),
        Tag::Float(value) => visitor.visit_float(*value),
        Tag::Double(value) => visitor.visit_double(*value),
        Tag::ByteArray(values) => visitor.visit_byte_array(values),
        Tag::String(value) => visitor.visit_string(value),
        Tag::IntArray(values) => visitor.visit_int_array(values),
        Tag::LongArray(values) => visitor.visit_long_array(values),
        Tag::List(values) => visit_list_payload(values, visitor)?,
        Tag::Compound(values) => visit_compound_payload(values, visitor)?,
    })
}

fn visit_list_payload<V: NbtStreamTagVisitor>(
    values: &[Tag],
    visitor: &mut V,
) -> io::Result<StreamValueResult> {
    let element_id = super::identify_list_raw_element_type(values);
    match visitor.visit_list(tag_type(i32::from(element_id)), values.len()) {
        StreamValueResult::Halt => return Ok(StreamValueResult::Halt),
        StreamValueResult::Break => return Ok(visitor.visit_container_end()),
        StreamValueResult::Continue => {}
    }

    for (index, value) in values.iter().enumerate() {
        let value = super::wrap_list_element_if_needed(element_id, value);
        match visitor.visit_element(tag_type(i32::from(value.id())), index) {
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            StreamEntryResult::Skip => continue,
            StreamEntryResult::Break => return Ok(visitor.visit_container_end()),
            StreamEntryResult::Enter => match visit_tag_payload(&value, visitor)? {
                StreamValueResult::Halt => return Ok(StreamValueResult::Halt),
                StreamValueResult::Break => return Ok(visitor.visit_container_end()),
                StreamValueResult::Continue => {}
            },
        }
    }

    Ok(visitor.visit_container_end())
}

fn visit_compound_payload<V: NbtStreamTagVisitor>(
    values: &[(String, Tag)],
    visitor: &mut V,
) -> io::Result<StreamValueResult> {
    for (name, value) in values {
        let value_type = tag_type(i32::from(value.id()));
        match visitor.visit_entry(value_type.clone()) {
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            StreamEntryResult::Skip => continue,
            StreamEntryResult::Break => return Ok(visitor.visit_container_end()),
            StreamEntryResult::Enter => {}
        }
        match visitor.visit_named_entry(value_type, name) {
            StreamEntryResult::Halt => return Ok(StreamValueResult::Halt),
            StreamEntryResult::Skip => continue,
            StreamEntryResult::Break => return Ok(visitor.visit_container_end()),
            StreamEntryResult::Enter => match visit_tag_payload(value, visitor)? {
                StreamValueResult::Halt => return Ok(StreamValueResult::Halt),
                StreamValueResult::Break => return Ok(visitor.visit_container_end()),
                StreamValueResult::Continue => {}
            },
        }
    }

    Ok(visitor.visit_container_end())
}

fn write_payload_with_string_fallback<W: Write>(tag: &Tag, writer: &mut W) -> io::Result<()> {
    match tag {
        Tag::String(value) => write_string_with_fallback(writer, value),
        Tag::List(values) => {
            let element_id = super::identify_list_raw_element_type(values);
            writer.write_all(&[element_id])?;
            write_i32_len(writer, values.len())?;
            for value in values {
                let value = super::wrap_list_element_if_needed(element_id, value);
                write_payload_with_string_fallback(&value, writer)?;
            }
            Ok(())
        }
        Tag::Compound(values) => {
            for (name, value) in values {
                writer.write_all(&[value.id()])?;
                write_string_with_fallback(writer, name)?;
                write_payload_with_string_fallback(value, writer)?;
            }
            writer.write_all(&[0])
        }
        _ => tag.write_payload(writer),
    }
}

fn write_string_with_fallback<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    if write_string(writer, value).is_err() {
        write_string(writer, "")
    } else {
        Ok(())
    }
}

fn write_i32_len<W: Write>(writer: &mut W, len: usize) -> io::Result<()> {
    let len = i32::try_from(len)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT collection too long"))?;
    writer.write_all(&len.to_be_bytes())
}

fn sync_output_file(path: &Path) -> io::Result<fs::File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::storage::nbt::tag_access::{
        CollectFieldsVisitor, NbtFieldSelectorSpec, NbtStreamTagVisitor, StreamEntryResult,
        StreamValueResult,
    };

    const NBT_IO_JAVA: &str =
        include_str!("../../../../decompiled-server-26.1.2/net/minecraft/nbt/NbtIo.java");

    #[test]
    fn nbt_io_matches_java_unnamed_any_compressed_and_parse_contracts() {
        for sentinel in [
            "StandardOpenOption.SYNC",
            "readCompressed(final InputStream in, final NbtAccounter accounter)",
            "parseCompressed(final InputStream in, final StreamTagVisitor output, final NbtAccounter accounter)",
            "writeCompressed(final CompoundTag tag, final OutputStream out)",
            "public static @Nullable CompoundTag read(final Path file)",
            "Root tag must be a named compound tag",
            "public static Tag readAnyTag(final DataInput input, final NbtAccounter accounter)",
            "public static void writeAnyTag(final Tag tag, final DataOutput output)",
            "public static void writeUnnamedTag(final Tag tag, final DataOutput output)",
            "writeUnnamedTagWithFallback(final Tag tag, final DataOutput output)",
            "StringTag.skipString(input);",
            "case CONTINUE:",
            "case BREAK:",
            "case HALT:",
            "Failed to write NBT String",
        ] {
            assert!(NBT_IO_JAVA.contains(sentinel), "missing NbtIo sentinel {sentinel}");
        }

        let tag = Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(4790)),
            (
                "Data".to_string(),
                Tag::Compound(vec![(
                    "LevelName".to_string(),
                    Tag::String("world".to_string()),
                )]),
            ),
        ]);

        let mut unnamed = Vec::new();
        write_unnamed_tag(&tag, &mut unnamed).unwrap();
        assert_eq!(unnamed[0], 10);
        assert_eq!(&unnamed[1..3], &[0, 0]);
        assert_eq!(read_unnamed_tag(&mut unnamed.as_slice()).unwrap(), tag);

        let mut any = Vec::new();
        write_any_tag(&Tag::String("value".to_string()), &mut any).unwrap();
        assert_eq!(
            read_any_tag(&mut any.as_slice()).unwrap(),
            Tag::String("value".to_string())
        );
        let mut end = Vec::new();
        write_any_tag(&Tag::End, &mut end).unwrap();
        assert_eq!(read_any_tag(&mut end.as_slice()).unwrap(), Tag::End);

        let mut root = Vec::new();
        write_root_compound(&tag, &mut root).unwrap();
        assert_eq!(read_root_compound(&mut root.as_slice()).unwrap(), tag);
        assert!(write_root_compound(&Tag::Int(1), &mut Vec::new()).is_err());
        let mut non_compound = Vec::new();
        write_unnamed_tag(&Tag::Int(1), &mut non_compound).unwrap();
        assert!(read_root_compound(&mut non_compound.as_slice())
            .unwrap_err()
            .to_string()
            .contains("Root tag must be a named compound tag"));

        let mut compressed = Vec::new();
        write_compressed(&tag, &mut compressed).unwrap();
        assert_eq!(&compressed[..2], &[0x1f, 0x8b]);
        assert_eq!(
            read_compressed(Cursor::new(compressed.clone())).unwrap(),
            tag
        );

        let mut collector = CollectFieldsVisitor::new(&[
            NbtFieldSelectorSpec::root(tag_type(3), "DataVersion"),
            NbtFieldSelectorSpec::child("Data", tag_type(8), "LevelName"),
        ]);
        parse_compressed(Cursor::new(compressed), &mut collector).unwrap();
        assert_eq!(collector.get_result(), Some(&tag));

        let mut fallback = Vec::new();
        write_unnamed_tag_with_fallback(
            &Tag::Compound(vec![("name".to_string(), Tag::String("x".repeat(70_000)))]),
            &mut fallback,
        )
        .unwrap();
        assert_eq!(
            read_unnamed_tag(&mut fallback.as_slice()).unwrap(),
            Tag::Compound(vec![("name".to_string(), Tag::String(String::new()))])
        );

        let mixed = Tag::Compound(vec![(
            "mixed".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::String("two".to_string())]),
        )]);
        let mut mixed_bytes = Vec::new();
        write_unnamed_tag_with_fallback(&mixed, &mut mixed_bytes).unwrap();
        assert_eq!(
            read_unnamed_tag(&mut mixed_bytes.as_slice()).unwrap(),
            mixed
        );
    }

    #[test]
    fn nbt_io_file_helpers_match_java_path_behavior() {
        let tag = Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4790))]);
        let base = std::env::temp_dir().join(format!(
            "vibecraft-nbt-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let plain = base.with_extension("nbt");
        let compressed = base.with_extension("nbt.gz");
        let missing = base.with_extension("missing");

        assert_eq!(read_path(&missing).unwrap(), None);
        write_path(&tag, &plain).unwrap();
        assert_eq!(read_path(&plain).unwrap(), Some(tag.clone()));
        write_compressed_path(&tag, &compressed).unwrap();
        assert_eq!(read_compressed_path(&compressed).unwrap(), tag);

        let _ = std::fs::remove_file(plain);
        let _ = std::fs::remove_file(compressed);
    }

    #[derive(Default)]
    struct BreakingRootVisitor {
        root_entries: usize,
    }

    impl NbtStreamTagVisitor for BreakingRootVisitor {
        fn visit_end(&mut self) -> StreamValueResult {
            StreamValueResult::Continue
        }

        fn visit_string(&mut self, _value: &str) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_byte(&mut self, _value: i8) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_short(&mut self, _value: i16) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_int(&mut self, _value: i32) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_long(&mut self, _value: i64) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_float(&mut self, _value: f32) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_double(&mut self, _value: f64) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_byte_array(&mut self, _value: &[i8]) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_int_array(&mut self, _value: &[i32]) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_long_array(&mut self, _value: &[i64]) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_list(
            &mut self,
            _element_type: super::super::tag_metadata::NbtTagTypeLookup,
            _size: usize,
        ) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_entry(
            &mut self,
            _tag_type: super::super::tag_metadata::NbtTagTypeLookup,
        ) -> StreamEntryResult {
            panic!("payload should be skipped")
        }

        fn visit_named_entry(
            &mut self,
            _tag_type: super::super::tag_metadata::NbtTagTypeLookup,
            _id: &str,
        ) -> StreamEntryResult {
            panic!("payload should be skipped")
        }

        fn visit_element(
            &mut self,
            _tag_type: super::super::tag_metadata::NbtTagTypeLookup,
            _index: usize,
        ) -> StreamEntryResult {
            panic!("payload should be skipped")
        }

        fn visit_container_end(&mut self) -> StreamValueResult {
            panic!("payload should be skipped")
        }

        fn visit_root_entry(
            &mut self,
            _tag_type: super::super::tag_metadata::NbtTagTypeLookup,
        ) -> StreamValueResult {
            self.root_entries += 1;
            StreamValueResult::Break
        }
    }

    #[test]
    fn nbt_io_parse_break_skips_payload_without_visiting_children() {
        let tag = Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(4790))]);
        let mut bytes = Vec::new();
        write_unnamed_tag(&tag, &mut bytes).unwrap();
        let mut visitor = BreakingRootVisitor::default();
        parse(&mut bytes.as_slice(), &mut visitor).unwrap();
        assert_eq!(visitor.root_entries, 1);
    }
}
