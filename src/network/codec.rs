#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{self, Cursor, Read, Write};
use std::sync::Arc;

use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;
use crate::storage::nbt::{read_named_tag, write_named_tag, Tag};

pub const MAX_INITIAL_COLLECTION_SIZE: usize = 65_536;

type DecodeFn<T> = dyn Fn(&mut dyn Read) -> io::Result<T>;
type EncodeFn<T> = dyn Fn(&mut dyn Write, &T) -> io::Result<()>;

pub struct StreamCodec<T> {
    decoder: Arc<DecodeFn<T>>,
    encoder: Arc<EncodeFn<T>>,
}

impl<T> Clone for StreamCodec<T> {
    fn clone(&self) -> Self {
        Self {
            decoder: Arc::clone(&self.decoder),
            encoder: Arc::clone(&self.encoder),
        }
    }
}

impl<T: 'static> StreamCodec<T> {
    pub fn of(
        encoder: impl Fn(&mut dyn Write, &T) -> io::Result<()> + 'static,
        decoder: impl Fn(&mut dyn Read) -> io::Result<T> + 'static,
    ) -> Self {
        Self {
            decoder: Arc::new(decoder),
            encoder: Arc::new(encoder),
        }
    }

    pub fn of_member(
        encoder: impl Fn(&T, &mut dyn Write) -> io::Result<()> + 'static,
        decoder: impl Fn(&mut dyn Read) -> io::Result<T> + 'static,
    ) -> Self {
        Self::of(move |output, value| encoder(value, output), decoder)
    }

    pub fn unit(instance: T) -> Self
    where
        T: Clone + Eq + std::fmt::Debug,
    {
        let decoded = instance.clone();
        Self::of(
            move |_output, value| {
                if value == &instance {
                    Ok(())
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Can't encode '{value:?}', expected '{instance:?}'"),
                    ))
                }
            },
            move |_input| Ok(decoded.clone()),
        )
    }

    pub fn decode(&self, input: &mut dyn Read) -> io::Result<T> {
        (self.decoder)(input)
    }

    pub fn encode(&self, output: &mut dyn Write, value: &T) -> io::Result<()> {
        (self.encoder)(output, value)
    }

    pub fn map<O: 'static>(
        self,
        to: impl Fn(T) -> O + 'static,
        from: impl Fn(&O) -> T + 'static,
    ) -> StreamCodec<O> {
        let encoder_codec = self.clone();
        let decoder_codec = self;
        StreamCodec::of(
            move |output, value| encoder_codec.encode(output, &from(value)),
            move |input| decoder_codec.decode(input).map(&to),
        )
    }
}

type CodecModifierFn<T, C> = dyn Fn(StreamCodec<T>, &C) -> StreamCodec<T>;

pub struct CodecModifier<T, C> {
    modifier: Box<CodecModifierFn<T, C>>,
}

impl<T, C> CodecModifier<T, C> {
    pub fn new(modifier: impl Fn(StreamCodec<T>, &C) -> StreamCodec<T> + 'static) -> Self {
        Self {
            modifier: Box::new(modifier),
        }
    }

    pub fn apply(&self, original: StreamCodec<T>, context: &C) -> StreamCodec<T> {
        (self.modifier)(original, context)
    }
}

pub struct IdDispatchCodec<V, T> {
    type_getter: Box<dyn Fn(&V) -> T>,
    by_id: Vec<IdDispatchEntry<V, T>>,
    to_id: HashMap<T, usize>,
}

struct IdDispatchEntry<V, T> {
    serializer: StreamCodec<V>,
    type_id: T,
}

impl<V: 'static, T> IdDispatchCodec<V, T>
where
    T: Clone + Eq + Hash + std::fmt::Debug + 'static,
{
    pub fn builder(type_getter: impl Fn(&V) -> T + 'static) -> IdDispatchCodecBuilder<V, T> {
        IdDispatchCodecBuilder {
            entries: Vec::new(),
            type_getter: Box::new(type_getter),
        }
    }

    pub fn decode(&self, input: &mut dyn Read) -> io::Result<V> {
        let id = read_var_i32(input)?;
        let Some(entry) = usize::try_from(id).ok().and_then(|id| self.by_id.get(id)) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Received unknown packet id {id}"),
            ));
        };

        entry.serializer.decode(input).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("Failed to decode packet '{:?}': {err}", entry.type_id),
            )
        })
    }

    pub fn encode(&self, output: &mut dyn Write, value: &V) -> io::Result<()> {
        let type_id = (self.type_getter)(value);
        let Some(&id) = self.to_id.get(&type_id) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Sending unknown packet '{type_id:?}'"),
            ));
        };
        write_var_i32(output, id as i32)?;

        let entry = &self.by_id[id];
        entry.serializer.encode(output, value).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("Failed to encode packet '{type_id:?}': {err}"),
            )
        })
    }
}

pub struct IdDispatchCodecBuilder<V, T> {
    entries: Vec<IdDispatchEntry<V, T>>,
    type_getter: Box<dyn Fn(&V) -> T>,
}

impl<V: 'static, T> IdDispatchCodecBuilder<V, T>
where
    T: Clone + Eq + Hash + std::fmt::Debug + 'static,
{
    pub fn add(mut self, type_id: T, serializer: StreamCodec<V>) -> Self {
        self.entries.push(IdDispatchEntry {
            serializer,
            type_id,
        });
        self
    }

    pub fn build(self) -> io::Result<IdDispatchCodec<V, T>> {
        let mut to_id = HashMap::new();
        for (id, entry) in self.entries.iter().enumerate() {
            if to_id.insert(entry.type_id.clone(), id).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Duplicate registration for type {:?}", entry.type_id),
                ));
            }
        }

        Ok(IdDispatchCodec {
            type_getter: self.type_getter,
            by_id: self.entries,
            to_id,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(pub [u8; 16]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentJson(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryValueId(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryFriendlyByteBuf {
    source: Vec<u8>,
    registry_access: String,
}

impl RegistryFriendlyByteBuf {
    pub fn new(source: Vec<u8>, registry_access: impl Into<String>) -> Self {
        Self {
            source,
            registry_access: registry_access.into(),
        }
    }

    pub fn source(&self) -> &[u8] {
        &self.source
    }

    pub fn registry_access(&self) -> &str {
        &self.registry_access
    }

    pub fn decorator(
        registry_access: impl Into<String>,
    ) -> impl Fn(Vec<u8>) -> RegistryFriendlyByteBuf {
        let registry_access = registry_access.into();
        move |source| RegistryFriendlyByteBuf::new(source, registry_access.clone())
    }
}

pub fn read_string<R: Read>(reader: &mut R, max_chars: usize) -> io::Result<String> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > utf8_max_bytes(max_chars) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid string length",
        ));
    }

    let mut bytes = vec![0u8; length as usize];
    reader.read_exact(&mut bytes)?;
    let string =
        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    // Java `Utf8String.read` bounds the decoded string by `result.length()`, i.e.
    // the UTF-16 code-unit count (astral chars count as 2), NOT the code-point
    // count — so use `encode_utf16().count()` to match exactly.
    if string.encode_utf16().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    Ok(string)
}

pub fn write_string<W: Write>(writer: &mut W, value: &str, max_chars: usize) -> io::Result<()> {
    // Java `Utf8String.write`: reject when the UTF-16 length exceeds `maxLength`,
    // then when the UTF-8 byte length exceeds Netty's `utf8MaxBytes(maxLength)`.
    if value.encode_utf16().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "string too long",
        ));
    }
    let bytes = value.as_bytes();
    if bytes.len() > utf8_max_bytes(max_chars) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "string too long",
        ));
    }
    write_var_i32(writer, bytes.len() as i32)?;
    writer.write_all(bytes)
}

fn utf8_max_bytes(char_sequence_length: usize) -> usize {
    char_sequence_length * 3
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
    // Java `FriendlyByteBuf.readNullable`/`readOptional` use `readBoolean()` for the
    // presence marker, i.e. any non-zero byte means present — match that exactly.
    let mut present = [0u8; 1];
    reader.read_exact(&mut present)?;
    if present[0] != 0 {
        Ok(Some(read(reader)?))
    } else {
        Ok(None)
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

pub fn read_count<R: Read>(reader: &mut R, max_size: usize) -> io::Result<usize> {
    let count = read_var_i32(reader)?;
    if count < 0 || count as usize > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{count} elements exceeded max size of: {max_size}"),
        ));
    }
    Ok(count as usize)
}

pub fn write_count<W: Write>(writer: &mut W, count: usize, max_size: usize) -> io::Result<()> {
    if count > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{count} elements exceeded max size of: {max_size}"),
        ));
    }
    write_var_i32(writer, count as i32)
}

pub fn read_collection<R, T, F>(reader: &mut R, read: F) -> io::Result<Vec<T>>
where
    R: Read,
    F: FnMut(&mut R) -> io::Result<T>,
{
    read_collection_limited(reader, usize::MAX, read)
}

pub fn read_collection_limited<R, T, F>(
    reader: &mut R,
    max_size: usize,
    mut read: F,
) -> io::Result<Vec<T>>
where
    R: Read,
    F: FnMut(&mut R) -> io::Result<T>,
{
    let len = read_count(reader, max_size)?;
    let mut values = Vec::with_capacity(len.min(MAX_INITIAL_COLLECTION_SIZE));
    for _ in 0..len {
        values.push(read(reader)?);
    }
    Ok(values)
}

pub fn write_collection<W, T, F>(writer: &mut W, values: &[T], write: F) -> io::Result<()>
where
    W: Write,
    F: FnMut(&mut W, &T) -> io::Result<()>,
{
    write_collection_limited(writer, values, usize::MAX, write)
}

pub fn write_collection_limited<W, T, F>(
    writer: &mut W,
    values: &[T],
    max_size: usize,
    mut write: F,
) -> io::Result<()>
where
    W: Write,
    F: FnMut(&mut W, &T) -> io::Result<()>,
{
    write_count(writer, values.len(), max_size)?;
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
    fn registry_friendly_byte_buf_matches_java_wrapper_contract() {
        const REGISTRY_FRIENDLY_BYTE_BUF_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/RegistryFriendlyByteBuf.java"
        );

        for sentinel in [
            "public class RegistryFriendlyByteBuf extends FriendlyByteBuf",
            "private final RegistryAccess registryAccess;",
            "public RegistryFriendlyByteBuf(final ByteBuf source, final RegistryAccess registryAccess)",
            "super(source);",
            "this.registryAccess = registryAccess;",
            "public RegistryAccess registryAccess()",
            "return this.registryAccess;",
            "public static Function<ByteBuf, RegistryFriendlyByteBuf> decorator(final RegistryAccess registryAccess)",
            "return buf -> new RegistryFriendlyByteBuf(buf, registryAccess);",
        ] {
            assert!(
                REGISTRY_FRIENDLY_BYTE_BUF_JAVA.contains(sentinel),
                "missing RegistryFriendlyByteBuf sentinel {sentinel}"
            );
        }

        let wrapped = RegistryFriendlyByteBuf::new(vec![1, 2, 3], "minecraft:registry_access");
        assert_eq!(wrapped.source(), &[1, 2, 3]);
        assert_eq!(wrapped.registry_access(), "minecraft:registry_access");

        let decorate = RegistryFriendlyByteBuf::decorator("frozen_access");
        let first = decorate(vec![4]);
        let second = decorate(vec![5, 6]);
        assert_eq!(first.source(), &[4]);
        assert_eq!(second.source(), &[5, 6]);
        assert_eq!(first.registry_access(), "frozen_access");
        assert_eq!(second.registry_access(), "frozen_access");
    }

    #[test]
    fn optional_presence_marker_is_lenient_like_java_read_boolean() {
        // Java `readNullable` uses `readBoolean()` (any non-zero → present), so a
        // marker byte of 2 must be treated as `Some`, not rejected.
        let mut input = cursor(vec![2, 0x07]);
        let value = read_optional(&mut input, read_var_i32).unwrap();
        assert_eq!(value, Some(7));

        let mut absent = cursor(vec![0]);
        assert_eq!(read_optional(&mut absent, read_var_i32).unwrap(), None);
    }

    #[test]
    fn stream_codec_model_matches_java_functional_surface() {
        const STREAM_CODEC_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/StreamCodec.java"
        );
        const STREAM_DECODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/StreamDecoder.java"
        );
        const STREAM_ENCODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/StreamEncoder.java"
        );
        const STREAM_MEMBER_ENCODER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/StreamMemberEncoder.java"
        );

        for sentinel in [
            "static <B, V> StreamCodec<B, V> of(final StreamEncoder<B, V> encoder, final StreamDecoder<B, V> decoder)",
            "static <B, V> StreamCodec<B, V> ofMember(final StreamMemberEncoder<B, V> encoder, final StreamDecoder<B, V> decoder)",
            "static <B, V> StreamCodec<B, V> unit(final V instance)",
            "if (!value.equals(instance))",
            "default <O> StreamCodec<B, O> map(final Function<? super V, ? extends O> to, final Function<? super O, ? extends V> from)",
            "default <U> StreamCodec<B, U> dispatch(",
            "static <B, C, T1, T2> StreamCodec<B, C> composite(",
            "static <B, T> StreamCodec<B, T> recursive(final UnaryOperator<StreamCodec<B, T>> factory)",
            "interface CodecOperation<B, S, T>",
        ] {
            assert!(
                STREAM_CODEC_JAVA.contains(sentinel),
                "missing StreamCodec sentinel {sentinel}"
            );
        }
        assert!(STREAM_DECODER_JAVA.contains("T decode(I input);"));
        assert!(STREAM_ENCODER_JAVA.contains("void encode(O output, T value);"));
        assert!(STREAM_MEMBER_ENCODER_JAVA.contains("void encode(T value, O output);"));

        let byte_codec = StreamCodec::of(
            |output, value: &u8| output.write_all(&[*value]),
            |input| {
                let mut byte = [0];
                input.read_exact(&mut byte)?;
                Ok(byte[0])
            },
        );
        let plus_one = byte_codec.map(|value| value + 1, |value| value - 1);
        let mut bytes = Vec::new();
        plus_one.encode(&mut bytes, &6).unwrap();
        assert_eq!(bytes, vec![5]);
        assert_eq!(plus_one.decode(&mut cursor(bytes)).unwrap(), 6);

        let member = StreamCodec::of_member(
            |value: &u8, output| output.write_all(&[*value + 1]),
            |input| {
                let mut byte = [0];
                input.read_exact(&mut byte)?;
                Ok(byte[0] - 1)
            },
        );
        let mut bytes = Vec::new();
        member.encode(&mut bytes, &8).unwrap();
        assert_eq!(bytes, vec![9]);
        assert_eq!(member.decode(&mut cursor(bytes)).unwrap(), 8);

        let unit = StreamCodec::unit("minecraft:unit");
        assert_eq!(
            unit.decode(&mut cursor(Vec::new())).unwrap(),
            "minecraft:unit"
        );
        assert!(unit.encode(&mut Vec::new(), &"minecraft:unit").is_ok());
        assert!(unit.encode(&mut Vec::new(), &"minecraft:other").is_err());
    }

    #[test]
    fn codec_modifier_wraps_stream_codec_with_context_like_java() {
        const CODEC_MODIFIER_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/CodecModifier.java"
        );

        for sentinel in [
            "@FunctionalInterface",
            "public interface CodecModifier<B, V, C>",
            "StreamCodec<? super B, V> apply(StreamCodec<? super B, V> original, C context);",
        ] {
            assert!(
                CODEC_MODIFIER_JAVA.contains(sentinel),
                "missing CodecModifier sentinel {sentinel}"
            );
        }

        let varint = StreamCodec::of(
            |output, value: &i32| write_var_i32(output, *value),
            |input| read_var_i32(input),
        );
        let add_context = CodecModifier::new(|original, offset: &i32| {
            let offset = *offset;
            original.map(move |value| value + offset, move |value| value - offset)
        });

        let modified = add_context.apply(varint, &5);
        let mut bytes = Vec::new();
        modified.encode(&mut bytes, &15).unwrap();
        assert_eq!(bytes, vec![10]);
        assert_eq!(modified.decode(&mut cursor(bytes)).unwrap(), 15);
    }

    #[test]
    fn string_length_limit_counts_utf16_units_like_java() {
        const UTF8_STRING_JAVA: &str =
            include_str!("../../../decompiled-server-26.1.2/net/minecraft/network/Utf8String.java");

        for sentinel in [
            "int maxEncodedLength = ByteBufUtil.utf8MaxBytes(maxLength);",
            "int bufferLength = VarInt.read(input);",
            "if (bufferLength > maxEncodedLength)",
            "if (bufferLength > availableBytes)",
            "String result = input.toString(input.readerIndex(), bufferLength, StandardCharsets.UTF_8);",
            "if (result.length() > maxLength)",
            "if (value.length() > maxLength)",
            "int bytesWritten = ByteBufUtil.writeUtf8(tmp, value);",
            "int maxAllowedEncodedLength = ByteBufUtil.utf8MaxBytes(maxLength);",
            "VarInt.write(output, bytesWritten);",
        ] {
            assert!(
                UTF8_STRING_JAVA.contains(sentinel),
                "missing Utf8String sentinel {sentinel}"
            );
        }

        // An astral-plane char (😀, U+1F600) is ONE Rust code point but TWO UTF-16
        // code units — Java `String.length()` counts 2, so a max of 1 must reject it
        // and a max of 2 must accept it (round-tripping the 4 UTF-8 bytes).
        let emoji = "\u{1F600}";
        assert_eq!(emoji.chars().count(), 1);
        assert_eq!(emoji.encode_utf16().count(), 2);

        assert!(write_string(&mut Vec::new(), emoji, 1).is_err());

        let mut bytes = Vec::new();
        write_string(&mut bytes, emoji, 2).unwrap();
        // VarInt(4) + 4 UTF-8 bytes.
        assert_eq!(bytes[0], 4);
        assert_eq!(read_string(&mut cursor(bytes.clone()), 2).unwrap(), emoji);
        // The same wire bytes must be rejected when the reader's limit is 1.
        assert!(read_string(&mut cursor(bytes), 1).is_err());

        let three_byte_char = "\u{20AC}";
        assert_eq!(three_byte_char.encode_utf16().count(), 1);
        assert_eq!(three_byte_char.len(), 3);
        let mut bytes = Vec::new();
        write_string(&mut bytes, three_byte_char, 1).unwrap();
        assert_eq!(read_string(&mut cursor(bytes), 1).unwrap(), three_byte_char);

        let four_ascii_bytes = "abcd";
        assert_eq!(four_ascii_bytes.encode_utf16().count(), 4);
        assert!(write_string(&mut Vec::new(), four_ascii_bytes, 1).is_err());

        let mut encoded_too_long = Vec::new();
        write_var_i32(&mut encoded_too_long, 4).unwrap();
        encoded_too_long.extend_from_slice(b"abcd");
        assert!(read_string(&mut cursor(encoded_too_long), 1).is_err());

        let mut truncated = Vec::new();
        write_var_i32(&mut truncated, 3).unwrap();
        truncated.extend_from_slice(b"ab");
        assert!(read_string(&mut cursor(truncated), 1).is_err());
    }

    #[test]
    fn byte_buf_codecs_count_limits_match_java_collections() {
        const BYTE_BUF_CODECS_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/ByteBufCodecs.java"
        );

        for sentinel in [
            "int MAX_INITIAL_COLLECTION_SIZE = 65536;",
            "static int readCount(final ByteBuf input, final int maxSize)",
            "if (count > maxSize)",
            "static void writeCount(final ByteBuf output, final int count, final int maxSize)",
            "constructor.apply(Math.min(count, 65536))",
            "StreamCodec.CodecOperation<B, V, List<V>> list(final int maxSize)",
            "return input.readBoolean() ? Optional.of(original.decode(input)) : Optional.empty();",
            "return input.readBoolean() ? Either.left(leftCodec.decode(input)) : Either.right(rightCodec.decode(input));",
            "static <V> StreamCodec.CodecOperation<ByteBuf, V, V> lengthPrefixed(final int maxSize)",
        ] {
            assert!(
                BYTE_BUF_CODECS_JAVA.contains(sentinel),
                "missing ByteBufCodecs sentinel {sentinel}"
            );
        }

        assert_eq!(MAX_INITIAL_COLLECTION_SIZE, 65_536);

        let mut bytes = Vec::new();
        write_collection_limited(&mut bytes, &[1, 2], 2, |writer, value| {
            write_var_i32(writer, *value)
        })
        .unwrap();
        assert_eq!(
            read_collection_limited(&mut cursor(bytes), 2, read_var_i32).unwrap(),
            vec![1, 2]
        );

        assert!(
            write_collection_limited(&mut Vec::new(), &[1, 2, 3], 2, |writer, value| {
                write_var_i32(writer, *value)
            })
            .is_err()
        );

        let mut too_many = Vec::new();
        write_var_i32(&mut too_many, 3).unwrap();
        too_many.extend_from_slice(&[1, 2, 3]);
        assert!(read_collection_limited(&mut cursor(too_many), 2, |reader| {
            let mut byte = [0];
            reader.read_exact(&mut byte)?;
            Ok(byte[0])
        })
        .is_err());
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

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestPacket {
        Ping(i32),
        Pong(i32),
        Unknown,
    }

    fn packet_type(packet: &TestPacket) -> &'static str {
        match packet {
            TestPacket::Ping(_) => "ping",
            TestPacket::Pong(_) => "pong",
            TestPacket::Unknown => "unknown",
        }
    }

    fn packet_codec(expected: &'static str) -> StreamCodec<TestPacket> {
        StreamCodec::of(
            move |output, packet| {
                let value = match (expected, packet) {
                    ("ping", TestPacket::Ping(value)) | ("pong", TestPacket::Pong(value)) => value,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "wrong packet type",
                        ));
                    }
                };
                write_var_i32(output, *value)
            },
            move |input| {
                let value = read_var_i32(input)?;
                match expected {
                    "ping" => Ok(TestPacket::Ping(value)),
                    "pong" => Ok(TestPacket::Pong(value)),
                    _ => unreachable!(),
                }
            },
        )
    }

    #[test]
    fn id_dispatch_codec_matches_java_registration_and_error_contracts() {
        const ID_DISPATCH_CODEC_JAVA: &str = include_str!(
            "../../../decompiled-server-26.1.2/net/minecraft/network/codec/IdDispatchCodec.java"
        );

        for sentinel in [
            "private static final int UNKNOWN_TYPE = -1;",
            "int id = VarInt.read(input);",
            "throw new DecoderException(\"Received unknown packet id \" + id);",
            "throw new EncoderException(\"Sending unknown packet '\" + type + \"'\");",
            "throw new DecoderException(\"Failed to decode packet '\" + entry.type + \"'\", e);",
            "throw new EncoderException(\"Failed to encode packet '\" + type + \"'\", e);",
            "toId.defaultReturnValue(-2);",
            "throw new IllegalStateException(\"Duplicate registration for type \" + entry.type);",
            "public interface DontDecorateException",
        ] {
            assert!(
                ID_DISPATCH_CODEC_JAVA.contains(sentinel),
                "missing IdDispatchCodec sentinel {sentinel}"
            );
        }

        let codec = IdDispatchCodec::builder(packet_type)
            .add("ping", packet_codec("ping"))
            .add("pong", packet_codec("pong"))
            .build()
            .unwrap();

        let mut bytes = Vec::new();
        codec.encode(&mut bytes, &TestPacket::Pong(300)).unwrap();
        assert_eq!(bytes, vec![1, 0xac, 0x02]);
        assert_eq!(
            codec.decode(&mut cursor(vec![0, 42])).unwrap(),
            TestPacket::Ping(42)
        );

        assert!(codec.decode(&mut cursor(vec![2])).is_err());
        assert!(codec.encode(&mut Vec::new(), &TestPacket::Unknown).is_err());

        let duplicate = IdDispatchCodec::builder(packet_type)
            .add("ping", packet_codec("ping"))
            .add("ping", packet_codec("ping"))
            .build();
        assert!(duplicate.is_err());
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
