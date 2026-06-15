use super::*;

pub const CLIENTBOUND_CHUNKS_BIOMES_MAX_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundChunksBiomesPacket {
    pub chunk_biome_data: Vec<ChunkBiomeData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkBiomeData {
    pub pos: ChunkPos,
    pub buffer: Vec<u8>,
}

impl ClientboundChunksBiomesPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            chunk_biome_data: read_collection(reader, ChunkBiomeData::read)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.chunk_biome_data, |writer, data| {
            data.write(writer)
        })
    }
}

impl ChunkBiomeData {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            pos: unpack_chunk_pos_from_long(read_i64(reader)?),
            buffer: read_sized_byte_array(reader, CLIENTBOUND_CHUNKS_BIOMES_MAX_BYTES)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, pack_chunk_pos_as_long(self.pos))?;
        write_sized_byte_array(writer, &self.buffer)
    }
}

fn read_sized_byte_array<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative byte array length",
        ));
    }

    let length = length as usize;
    if length > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("byte array length {length} exceeds maximum {max_size}"),
        ));
    }

    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_sized_byte_array<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    write_var_i32(writer, bytes.len() as i32)?;
    writer.write_all(bytes)
}
