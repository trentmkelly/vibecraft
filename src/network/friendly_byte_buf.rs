#![allow(dead_code)]

use std::collections::BTreeMap;
use std::io::{self, Read, Write};

use crate::network::codec::{read_string, read_uuid, write_string, write_uuid, Uuid};
use crate::network::varint::{read_var_i32, read_var_i64, write_var_i32, write_var_i64};
use crate::registry::{Identifier, Registry, ResourceKey};

pub const MAX_STRING_LENGTH: usize = 32767;
pub const MAX_COMPONENT_STRING_LENGTH: usize = 262144;
pub const MAX_PUBLIC_KEY_LENGTH: usize = 512;

#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPos {
    pub dimension: ResourceKey<()>,
    pub pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3f(pub f32, pub f32, pub f32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternionf(pub f32, pub f32, pub f32, pub f32);

#[derive(Debug, Clone, PartialEq)]
pub struct BlockHitResult {
    pub block_pos: BlockPos,
    pub direction_ordinal: i32,
    pub location: Vector3f,
    pub inside: bool,
    pub world_border: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKeyBytes(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriendlyByteBufModel {
    bytes: Vec<u8>,
    reader_index: usize,
}

impl FriendlyByteBufModel {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            reader_index: 0,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            reader_index: 0,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn readable_bytes(&self) -> usize {
        self.bytes.len().saturating_sub(self.reader_index)
    }

    pub fn read_collection<T, F>(&mut self, mut decode: F) -> io::Result<Vec<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        let count = self.read_nonnegative_len()?;
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(decode(self)?);
        }
        Ok(result)
    }

    pub fn write_collection<T, F>(&mut self, values: &[T], mut encode: F) -> io::Result<()>
    where
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        self.write_var_int(values.len() as i32)?;
        for value in values {
            encode(self, value)?;
        }
        Ok(())
    }

    pub fn read_int_id_list(&mut self) -> io::Result<Vec<i32>> {
        self.read_collection(Self::read_var_int)
    }

    pub fn write_int_id_list(&mut self, values: &[i32]) -> io::Result<()> {
        self.write_collection(values, |buf, value| buf.write_var_int(*value))
    }

    pub fn read_map<K: Ord, V, FK, FV>(
        &mut self,
        mut key_decode: FK,
        mut value_decode: FV,
    ) -> io::Result<BTreeMap<K, V>>
    where
        FK: FnMut(&mut Self) -> io::Result<K>,
        FV: FnMut(&mut Self) -> io::Result<V>,
    {
        let count = self.read_nonnegative_len()?;
        let mut result = BTreeMap::new();
        for _ in 0..count {
            result.insert(key_decode(self)?, value_decode(self)?);
        }
        Ok(result)
    }

    pub fn write_map<K: Ord, V, FK, FV>(
        &mut self,
        values: &BTreeMap<K, V>,
        mut key_encode: FK,
        mut value_encode: FV,
    ) -> io::Result<()>
    where
        FK: FnMut(&mut Self, &K) -> io::Result<()>,
        FV: FnMut(&mut Self, &V) -> io::Result<()>,
    {
        self.write_var_int(values.len() as i32)?;
        for (key, value) in values {
            key_encode(self, key)?;
            value_encode(self, value)?;
        }
        Ok(())
    }

    pub fn read_with_count<F>(&mut self, mut read: F) -> io::Result<usize>
    where
        F: FnMut(&mut Self) -> io::Result<()>,
    {
        let count = self.read_nonnegative_len()?;
        for _ in 0..count {
            read(self)?;
        }
        Ok(count)
    }

    pub fn write_enum_set(&mut self, ordinals: &[usize], enum_len: usize) -> io::Result<()> {
        let mut bits = vec![false; enum_len];
        for &ordinal in ordinals {
            if ordinal >= enum_len {
                return Err(invalid("enum ordinal outside enum constants"));
            }
            bits[ordinal] = true;
        }
        self.write_fixed_bitset(&bits, enum_len)
    }

    pub fn read_enum_set(&mut self, enum_len: usize) -> io::Result<Vec<usize>> {
        Ok(self
            .read_fixed_bitset(enum_len)?
            .into_iter()
            .enumerate()
            .filter_map(|(index, set)| set.then_some(index))
            .collect())
    }

    pub fn write_optional<T, F>(&mut self, value: Option<&T>, mut write: F) -> io::Result<()>
    where
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            write(self, value)?;
        }
        Ok(())
    }

    pub fn read_optional<T, F>(&mut self, mut read: F) -> io::Result<Option<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        if self.read_bool()? {
            Ok(Some(read(self)?))
        } else {
            Ok(None)
        }
    }

    pub fn write_either<L, R, FL, FR>(
        &mut self,
        value: &Either<L, R>,
        mut write_left: FL,
        mut write_right: FR,
    ) -> io::Result<()>
    where
        FL: FnMut(&mut Self, &L) -> io::Result<()>,
        FR: FnMut(&mut Self, &R) -> io::Result<()>,
    {
        match value {
            Either::Left(left) => {
                self.write_bool(true);
                write_left(self, left)
            }
            Either::Right(right) => {
                self.write_bool(false);
                write_right(self, right)
            }
        }
    }

    pub fn read_either<L, R, FL, FR>(
        &mut self,
        mut read_left: FL,
        mut read_right: FR,
    ) -> io::Result<Either<L, R>>
    where
        FL: FnMut(&mut Self) -> io::Result<L>,
        FR: FnMut(&mut Self) -> io::Result<R>,
    {
        if self.read_bool()? {
            Ok(Either::Left(read_left(self)?))
        } else {
            Ok(Either::Right(read_right(self)?))
        }
    }

    pub fn write_nullable<T, F>(&mut self, value: Option<&T>, write: F) -> io::Result<()>
    where
        F: FnMut(&mut Self, &T) -> io::Result<()>,
    {
        self.write_optional(value, write)
    }

    pub fn read_nullable<T, F>(&mut self, read: F) -> io::Result<Option<T>>
    where
        F: FnMut(&mut Self) -> io::Result<T>,
    {
        self.read_optional(read)
    }

    pub fn write_byte_array(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write_var_int(bytes.len() as i32)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    pub fn read_byte_array(&mut self, max_size: usize) -> io::Result<Vec<u8>> {
        let size = self.read_nonnegative_len()?;
        if size > max_size {
            return Err(invalid(format!(
                "ByteArray with size {size} is bigger than allowed {max_size}"
            )));
        }
        self.read_exact_vec(size)
    }

    pub fn write_var_int_array(&mut self, values: &[i32]) -> io::Result<()> {
        self.write_collection(values, |buf, value| buf.write_var_int(*value))
    }

    pub fn read_var_int_array(&mut self, max_size: usize) -> io::Result<Vec<i32>> {
        let size = self.read_nonnegative_len()?;
        if size > max_size {
            return Err(invalid(format!(
                "VarIntArray with size {size} is bigger than allowed {max_size}"
            )));
        }
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(self.read_var_int()?);
        }
        Ok(result)
    }

    pub fn write_long_array(&mut self, values: &[i64]) -> io::Result<()> {
        self.write_var_int(values.len() as i32)?;
        self.write_fixed_size_long_array(values)
    }

    pub fn read_long_array(&mut self) -> io::Result<Vec<i64>> {
        let size = self.read_nonnegative_len()?;
        let max_size = self.readable_bytes() / 8;
        if size > max_size {
            return Err(invalid(format!(
                "LongArray with size {size} is bigger than allowed {max_size}"
            )));
        }
        self.read_fixed_size_long_array(size)
    }

    pub fn write_fixed_size_long_array(&mut self, values: &[i64]) -> io::Result<()> {
        for value in values {
            self.write_i64(*value);
        }
        Ok(())
    }

    pub fn read_fixed_size_long_array(&mut self, size: usize) -> io::Result<Vec<i64>> {
        let mut result = Vec::with_capacity(size);
        for _ in 0..size {
            result.push(self.read_i64()?);
        }
        Ok(result)
    }

    pub fn write_block_pos(&mut self, pos: BlockPos) {
        self.write_i64(pack_block_pos(pos));
    }

    pub fn read_block_pos(&mut self) -> io::Result<BlockPos> {
        Ok(unpack_block_pos(self.read_i64()?))
    }

    pub fn write_chunk_pos(&mut self, pos: ChunkPos) {
        self.write_i64(pack_chunk_pos(pos));
    }

    pub fn read_chunk_pos(&mut self) -> io::Result<ChunkPos> {
        Ok(unpack_chunk_pos(self.read_i64()?))
    }

    pub fn write_global_pos(&mut self, global: &GlobalPos) -> io::Result<()> {
        self.write_resource_key(&global.dimension)?;
        self.write_block_pos(global.pos);
        Ok(())
    }

    pub fn read_global_pos(&mut self, registry: Identifier) -> io::Result<GlobalPos> {
        Ok(GlobalPos {
            dimension: self.read_resource_key(registry)?,
            pos: self.read_block_pos()?,
        })
    }

    pub fn write_vector3f(&mut self, value: Vector3f) {
        self.write_f32(value.0);
        self.write_f32(value.1);
        self.write_f32(value.2);
    }

    pub fn read_vector3f(&mut self) -> io::Result<Vector3f> {
        Ok(Vector3f(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn write_quaternion(&mut self, value: Quaternionf) {
        self.write_f32(value.0);
        self.write_f32(value.1);
        self.write_f32(value.2);
        self.write_f32(value.3);
    }

    pub fn read_quaternion(&mut self) -> io::Result<Quaternionf> {
        Ok(Quaternionf(
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ))
    }

    pub fn write_enum(&mut self, ordinal: i32) -> io::Result<()> {
        self.write_var_int(ordinal)
    }

    pub fn read_enum(&mut self, enum_len: usize) -> io::Result<usize> {
        let ordinal = self.read_var_int()?;
        let ordinal = usize::try_from(ordinal).map_err(|_| invalid("negative enum ordinal"))?;
        if ordinal >= enum_len {
            return Err(invalid("enum ordinal outside enum constants"));
        }
        Ok(ordinal)
    }

    pub fn write_by_id<T>(
        &mut self,
        value: &T,
        converter: impl FnOnce(&T) -> i32,
    ) -> io::Result<()> {
        self.write_var_int(converter(value))
    }

    pub fn read_by_id<T>(&mut self, converter: impl FnOnce(i32) -> T) -> io::Result<T> {
        Ok(converter(self.read_var_int()?))
    }

    pub fn write_uuid(&mut self, uuid: Uuid) -> io::Result<()> {
        write_uuid(self, uuid)
    }

    pub fn read_uuid(&mut self) -> io::Result<Uuid> {
        read_uuid(self)
    }

    pub fn write_utf(&mut self, value: &str, max_length: usize) -> io::Result<()> {
        write_string(self, value, max_length)
    }

    pub fn read_utf(&mut self, max_length: usize) -> io::Result<String> {
        read_string(self, max_length)
    }

    pub fn write_identifier(&mut self, id: &Identifier) -> io::Result<()> {
        self.write_utf(&id.to_string(), MAX_STRING_LENGTH)
    }

    pub fn read_identifier(&mut self) -> io::Result<Identifier> {
        Identifier::parse(&self.read_utf(MAX_STRING_LENGTH)?)
            .map_err(|err| invalid(format!("invalid identifier: {err}")))
    }

    pub fn write_resource_key<T>(&mut self, key: &ResourceKey<T>) -> io::Result<()> {
        self.write_identifier(key.location())
    }

    pub fn read_resource_key<T>(&mut self, registry: Identifier) -> io::Result<ResourceKey<T>> {
        Ok(ResourceKey::new(registry, self.read_identifier()?))
    }

    pub fn read_registry_key<T>(&mut self) -> io::Result<ResourceKey<Registry<T>>> {
        Ok(ResourceKey::create_registry_key(self.read_identifier()?))
    }

    pub fn write_instant_millis(&mut self, epoch_millis: i64) {
        self.write_i64(epoch_millis);
    }

    pub fn read_instant_millis(&mut self) -> io::Result<i64> {
        self.read_i64()
    }

    pub fn write_public_key(&mut self, key: &PublicKeyBytes) -> io::Result<()> {
        self.write_byte_array(&key.0)
    }

    pub fn read_public_key(&mut self) -> io::Result<PublicKeyBytes> {
        Ok(PublicKeyBytes(self.read_byte_array(MAX_PUBLIC_KEY_LENGTH)?))
    }

    pub fn write_block_hit_result(&mut self, hit: &BlockHitResult) -> io::Result<()> {
        self.write_block_pos(hit.block_pos);
        self.write_enum(hit.direction_ordinal)?;
        self.write_f32(hit.location.0 - hit.block_pos.x as f32);
        self.write_f32(hit.location.1 - hit.block_pos.y as f32);
        self.write_f32(hit.location.2 - hit.block_pos.z as f32);
        self.write_bool(hit.inside);
        self.write_bool(hit.world_border);
        Ok(())
    }

    pub fn read_block_hit_result(&mut self, direction_count: usize) -> io::Result<BlockHitResult> {
        let block_pos = self.read_block_pos()?;
        let direction_ordinal = self.read_enum(direction_count)? as i32;
        let click_x = self.read_f32()?;
        let click_y = self.read_f32()?;
        let click_z = self.read_f32()?;
        Ok(BlockHitResult {
            block_pos,
            direction_ordinal,
            location: Vector3f(
                block_pos.x as f32 + click_x,
                block_pos.y as f32 + click_y,
                block_pos.z as f32 + click_z,
            ),
            inside: self.read_bool()?,
            world_border: self.read_bool()?,
        })
    }

    pub fn write_bitset(&mut self, bits: &[bool]) -> io::Result<()> {
        self.write_long_array(&bits_to_longs(bits))
    }

    pub fn read_bitset(&mut self) -> io::Result<Vec<bool>> {
        Ok(longs_to_bits(&self.read_long_array()?))
    }

    pub fn write_fixed_bitset(&mut self, bits: &[bool], size: usize) -> io::Result<()> {
        let highest = bits.iter().rposition(|set| *set).map(|idx| idx + 1).unwrap_or(0);
        if highest > size {
            return Err(invalid(format!(
                "BitSet is larger than expected size ({highest}>{size})"
            )));
        }
        let mut bytes = vec![0u8; size.div_ceil(8)];
        for (idx, set) in bits.iter().copied().enumerate().take(size) {
            if set {
                bytes[idx / 8] |= 1 << (idx % 8);
            }
        }
        self.bytes.extend_from_slice(&bytes);
        Ok(())
    }

    pub fn read_fixed_bitset(&mut self, size: usize) -> io::Result<Vec<bool>> {
        let bytes = self.read_exact_vec(size.div_ceil(8))?;
        Ok((0..size)
            .map(|idx| bytes[idx / 8] & (1 << (idx % 8)) != 0)
            .collect())
    }

    pub fn write_container_id(&mut self, id: i32) -> io::Result<()> {
        self.write_var_int(id)
    }

    pub fn read_container_id(&mut self) -> io::Result<i32> {
        self.read_var_int()
    }

    pub fn write_bool(&mut self, value: bool) {
        self.bytes.push(u8::from(value));
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn write_var_int(&mut self, value: i32) -> io::Result<()> {
        write_var_i32(self, value)
    }

    pub fn read_var_int(&mut self) -> io::Result<i32> {
        read_var_i32(self)
    }

    pub fn write_var_long(&mut self, value: i64) -> io::Result<()> {
        write_var_i64(self, value)
    }

    pub fn read_var_long(&mut self) -> io::Result<i64> {
        read_var_i64(self)
    }

    pub fn write_i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        let bytes = self.read_exact_array::<8>()?;
        Ok(i64::from_be_bytes(bytes))
    }

    pub fn write_f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        let bytes = self.read_exact_array::<4>()?;
        Ok(f32::from_be_bytes(bytes))
    }

    fn read_u8(&mut self) -> io::Result<u8> {
        Ok(self.read_exact_array::<1>()?[0])
    }

    fn read_nonnegative_len(&mut self) -> io::Result<usize> {
        usize::try_from(self.read_var_int()?).map_err(|_| invalid("negative size"))
    }

    fn read_exact_vec(&mut self, size: usize) -> io::Result<Vec<u8>> {
        if self.readable_bytes() < size {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "buffer underflow"));
        }
        let start = self.reader_index;
        self.reader_index += size;
        Ok(self.bytes[start..start + size].to_vec())
    }

    fn read_exact_array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let bytes = self.read_exact_vec(N)?;
        bytes
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::UnexpectedEof, "buffer underflow"))
    }
}

impl Read for FriendlyByteBufModel {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        let len = out.len().min(self.readable_bytes());
        out[..len].copy_from_slice(&self.bytes[self.reader_index..self.reader_index + len]);
        self.reader_index += len;
        Ok(len)
    }
}

impl Write for FriendlyByteBufModel {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn limit_value<T>(
    original: impl FnOnce(usize) -> T,
    value: usize,
    limit: usize,
) -> io::Result<T> {
    if value > limit {
        Err(invalid(format!("Value {value} is larger than limit {limit}")))
    } else {
        Ok(original(value))
    }
}

pub fn delegated_bytebuf_methods() -> &'static [&'static str] {
    &[
        "capacity", "alloc", "order", "unwrap", "isDirect", "isReadOnly", "readerIndex",
        "writerIndex", "setIndex", "readableBytes", "writableBytes", "isReadable", "isWritable",
        "clear", "markReaderIndex", "resetReaderIndex", "markWriterIndex", "resetWriterIndex",
        "discardReadBytes", "ensureWritable", "getBoolean", "getByte", "getShort", "getInt",
        "getLong", "getFloat", "getDouble", "getBytes", "setBoolean", "setByte", "setShort",
        "setInt", "setLong", "setFloat", "setDouble", "setBytes", "readBoolean", "readByte",
        "readShort", "readInt", "readLong", "readFloat", "readDouble", "readBytes",
        "readSlice", "skipBytes", "writeBoolean", "writeByte", "writeShort", "writeInt",
        "writeLong", "writeFloat", "writeDouble", "writeBytes", "writeZero", "indexOf",
        "bytesBefore", "forEachByte", "copy", "slice", "duplicate", "nioBuffer", "hasArray",
        "array", "memoryAddress", "toString", "hashCode", "equals", "compareTo", "retain",
        "touch", "refCnt", "release",
    ]
}

fn pack_block_pos(pos: BlockPos) -> i64 {
    const HORIZONTAL_BITS: i32 = 26;
    const Y_BITS: i32 = 12;
    const X_OFFSET: i32 = Y_BITS + HORIZONTAL_BITS;
    const Z_OFFSET: i32 = Y_BITS;
    let x = (pos.x as i64 & ((1_i64 << HORIZONTAL_BITS) - 1)) << X_OFFSET;
    let y = pos.y as i64 & ((1_i64 << Y_BITS) - 1);
    let z = (pos.z as i64 & ((1_i64 << HORIZONTAL_BITS) - 1)) << Z_OFFSET;
    x | y | z
}

fn unpack_block_pos(value: i64) -> BlockPos {
    BlockPos {
        x: (value >> 38) as i32,
        y: ((value << 52) >> 52) as i32,
        z: ((value << 26) >> 38) as i32,
    }
}

fn pack_chunk_pos(pos: ChunkPos) -> i64 {
    (pos.x as u32 as i64) | ((pos.z as u32 as i64) << 32)
}

fn unpack_chunk_pos(value: i64) -> ChunkPos {
    ChunkPos {
        x: value as u32 as i32,
        z: ((value >> 32) as u32) as i32,
    }
}

fn bits_to_longs(bits: &[bool]) -> Vec<i64> {
    let mut longs = vec![0_i64; bits.len().div_ceil(64)];
    for (idx, set) in bits.iter().copied().enumerate() {
        if set {
            longs[idx / 64] |= 1_i64 << (idx % 64);
        }
    }
    longs
}

fn longs_to_bits(longs: &[i64]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(longs.len() * 64);
    for value in longs {
        for idx in 0..64 {
            bits.push(value & (1_i64 << idx) != 0);
        }
    }
    bits
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const FRIENDLY_BYTE_BUF_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/network/FriendlyByteBuf.java");
    const BLOCK_POS_JAVA: &str = vibecraft_java_source!("/net/minecraft/core/BlockPos.java");
    const CHUNK_POS_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/level/ChunkPos.java");

    #[test]
    fn java_source_sentinels_cover_custom_friendly_byte_buf_surface() {
        assert_java_contains(
            FRIENDLY_BYTE_BUF_JAVA,
            &[
                "public class FriendlyByteBuf extends ByteBuf",
                "public static final short MAX_STRING_LENGTH = 32767;",
                "public static final int MAX_COMPONENT_STRING_LENGTH = 262144;",
                "public static <T> IntFunction<T> limitValue",
                "public <T, C extends Collection<T>> C readCollection",
                "public <K, V, M extends Map<K, V>> M readMap",
                "public <T> void writeOptional",
                "public <L, R> Either<L, R> readEither",
                "public static byte[] readByteArray(final ByteBuf input, final int maxSize)",
                "public FriendlyByteBuf writeVarIntArray",
                "public static long[] readLongArray",
                "public static void writeBlockPos",
                "public void writeBlockHitResult",
                "public static int readContainerId",
            ],
        );
    }

    #[test]
    fn collections_maps_optionals_either_and_nullable_use_java_presence_shapes() {
        let mut buf = FriendlyByteBufModel::new();
        buf.write_collection(&[1, 2, 3], |buf, value| buf.write_var_int(*value))
            .unwrap();
        let map = BTreeMap::from([("a".to_string(), 7), ("b".to_string(), 9)]);
        buf.write_map(
            &map,
            |buf, key| buf.write_utf(key, MAX_STRING_LENGTH),
            |buf, value| buf.write_var_int(*value),
        )
        .unwrap();
        buf.write_optional(Some(&42), |buf, value| buf.write_var_int(*value))
            .unwrap();
        buf.write_optional::<i32, _>(None, |buf, value| buf.write_var_int(*value))
            .unwrap();
        buf.write_either(
            &Either::<i32, &str>::Left(5),
            |buf, value| buf.write_var_int(*value),
            |buf, value| buf.write_utf(value, MAX_STRING_LENGTH),
        )
        .unwrap();
        buf.write_nullable(Some(&11), |buf, value| buf.write_var_int(*value))
            .unwrap();
        buf.write_nullable::<i32, _>(None, |buf, value| buf.write_var_int(*value))
            .unwrap();

        let mut input = FriendlyByteBufModel::from_bytes(buf.into_bytes());
        assert_eq!(
            input
                .read_collection(FriendlyByteBufModel::read_var_int)
                .unwrap(),
            vec![1, 2, 3]
        );
        assert_eq!(
            input
                .read_map(
                    |buf| buf.read_utf(MAX_STRING_LENGTH),
                    FriendlyByteBufModel::read_var_int
                )
                .unwrap(),
            map
        );
        assert_eq!(
            input
                .read_optional(FriendlyByteBufModel::read_var_int)
                .unwrap(),
            Some(42)
        );
        assert_eq!(
            input
                .read_optional(FriendlyByteBufModel::read_var_int)
                .unwrap(),
            None
        );
        assert_eq!(
            input
                .read_either(
                    FriendlyByteBufModel::read_var_int,
                    |buf| buf.read_utf(MAX_STRING_LENGTH)
                )
                .unwrap(),
            Either::Left(5)
        );
        assert_eq!(
            input
                .read_nullable(FriendlyByteBufModel::read_var_int)
                .unwrap(),
            Some(11)
        );
        assert_eq!(
            input
                .read_nullable(FriendlyByteBufModel::read_var_int)
                .unwrap(),
            None
        );
    }

    #[test]
    fn arrays_bitsets_and_enum_sets_match_varint_size_prefixes_and_fixed_bit_layout() {
        let mut buf = FriendlyByteBufModel::new();
        buf.write_byte_array(&[9, 8, 7]).unwrap();
        buf.write_var_int_array(&[1, 300, -1]).unwrap();
        buf.write_long_array(&[0x0102_0304_0506_0708, -2]).unwrap();
        buf.write_enum_set(&[0, 3, 5], 6).unwrap();
        buf.write_fixed_bitset(&[true, false, true, false, false, true], 6)
            .unwrap();

        let mut input = FriendlyByteBufModel::from_bytes(buf.into_bytes());
        assert_eq!(input.read_byte_array(3).unwrap(), vec![9, 8, 7]);
        assert_eq!(input.read_var_int_array(3).unwrap(), vec![1, 300, -1]);
        assert_eq!(
            input.read_long_array().unwrap(),
            vec![0x0102_0304_0506_0708, -2]
        );
        assert_eq!(input.read_enum_set(6).unwrap(), vec![0, 3, 5]);
        assert_eq!(
            input.read_fixed_bitset(6).unwrap(),
            vec![true, false, true, false, false, true]
        );

        let mut over = FriendlyByteBufModel::new();
        over.write_byte_array(&[1, 2, 3, 4]).unwrap();
        assert!(FriendlyByteBufModel::from_bytes(over.into_bytes())
            .read_byte_array(3)
            .unwrap_err()
            .to_string()
            .contains("ByteArray with size 4"));
    }

    #[test]
    fn position_vector_uuid_identifier_resource_key_and_hit_result_round_trip() {
        assert_java_contains(
            BLOCK_POS_JAVA,
            &[
                "PACKED_HORIZONTAL_LENGTH = 1 + Mth.log2(Mth.smallestEncompassingPowerOfTwo(30000000))",
                "node |= (x & PACKED_X_MASK) << X_OFFSET;",
                "return node | (z & PACKED_Z_MASK) << Z_OFFSET;",
            ],
        );
        assert_java_contains(
            CHUNK_POS_JAVA,
            &["return x & 4294967295L | (z & 4294967295L) << 32;"],
        );

        let dimension_registry = Identifier::parse("minecraft:dimension").unwrap();
        let dimension = ResourceKey::<()>::new(
            dimension_registry.clone(),
            Identifier::parse("minecraft:overworld").unwrap(),
        );
        let hit = BlockHitResult {
            block_pos: BlockPos {
                x: -12,
                y: 64,
                z: 300,
            },
            direction_ordinal: 2,
            location: Vector3f(-11.25, 64.5, 300.75),
            inside: true,
            world_border: false,
        };
        let uuid = Uuid([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let mut buf = FriendlyByteBufModel::new();
        buf.write_block_pos(hit.block_pos);
        buf.write_chunk_pos(ChunkPos { x: -3, z: 9 });
        buf.write_global_pos(&GlobalPos {
            dimension: dimension.clone(),
            pos: hit.block_pos,
        })
        .unwrap();
        buf.write_vector3f(Vector3f(1.25, -2.5, 3.75));
        buf.write_quaternion(Quaternionf(0.0, 0.25, 0.5, 1.0));
        buf.write_uuid(uuid).unwrap();
        buf.write_identifier(&Identifier::parse("minecraft:stone").unwrap())
            .unwrap();
        buf.write_resource_key(&dimension).unwrap();
        buf.write_instant_millis(123456789);
        buf.write_public_key(&PublicKeyBytes(vec![7; 256])).unwrap();
        buf.write_block_hit_result(&hit).unwrap();

        let mut input = FriendlyByteBufModel::from_bytes(buf.into_bytes());
        assert_eq!(input.read_block_pos().unwrap(), hit.block_pos);
        assert_eq!(input.read_chunk_pos().unwrap(), ChunkPos { x: -3, z: 9 });
        assert_eq!(
            input.read_global_pos(dimension_registry.clone()).unwrap(),
            GlobalPos {
                dimension: dimension.clone(),
                pos: hit.block_pos,
            }
        );
        assert_eq!(input.read_vector3f().unwrap(), Vector3f(1.25, -2.5, 3.75));
        assert_eq!(
            input.read_quaternion().unwrap(),
            Quaternionf(0.0, 0.25, 0.5, 1.0)
        );
        assert_eq!(input.read_uuid().unwrap(), uuid);
        assert_eq!(
            input.read_identifier().unwrap(),
            Identifier::parse("minecraft:stone").unwrap()
        );
        assert_eq!(
            input.read_resource_key::<()>(dimension_registry).unwrap(),
            dimension
        );
        assert_eq!(input.read_instant_millis().unwrap(), 123456789);
        assert_eq!(input.read_public_key().unwrap(), PublicKeyBytes(vec![7; 256]));
        assert_eq!(input.read_block_hit_result(6).unwrap(), hit);
    }

    #[test]
    fn delegated_bytebuf_surface_is_documented_as_passthrough() {
        if FRIENDLY_BYTE_BUF_JAVA.is_empty() {
            return;
        }
        for method in delegated_bytebuf_methods() {
            assert!(
                FRIENDLY_BYTE_BUF_JAVA.contains(&format!("{}(", method))
                    || FRIENDLY_BYTE_BUF_JAVA.contains(&format!("{}final", method)),
                "missing delegated ByteBuf method {method}"
            );
        }
        assert_java_contains(
            FRIENDLY_BYTE_BUF_JAVA,
            &[
                "return this.source.readerIndex();",
                "this.source.readerIndex(readerIndex);",
                "return this.source.readableBytes();",
                "this.source.writeInt(value);",
                "return this.source.release(decrement);",
            ],
        );
    }

    #[test]
    fn limit_value_and_container_id_match_java_error_and_varint_shape() {
        assert_eq!(limit_value(|value| value * 2, 5, 8).unwrap(), 10);
        assert!(limit_value(|value| value, 9, 8)
            .unwrap_err()
            .to_string()
            .contains("Value 9 is larger than limit 8"));

        let mut buf = FriendlyByteBufModel::new();
        buf.write_container_id(127).unwrap();
        buf.write_container_id(128).unwrap();
        let bytes = buf.clone().into_bytes();
        assert_eq!(bytes, vec![0x7f, 0x80, 0x01]);
        assert_eq!(buf.read_container_id().unwrap(), 127);
        assert_eq!(buf.read_container_id().unwrap(), 128);
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }

    impl FriendlyByteBufModel {
        fn clone(&self) -> Self {
            Self {
                bytes: self.bytes.clone(),
                reader_index: self.reader_index,
            }
        }
    }
}
