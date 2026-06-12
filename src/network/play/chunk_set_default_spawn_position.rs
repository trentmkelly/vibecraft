use super::*;

impl ClientboundSetDefaultSpawnPositionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let dimension = read_identifier(reader)?;
        let (x, y, z) = read_block_position(reader)?;
        let packet = Self {
            respawn_data: ClientboundSetDefaultSpawnPositionData {
                dimension,
                x,
                y,
                z,
                yaw: read_f32(reader)?,
                pitch: read_f32(reader)?,
            },
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.respawn_data.dimension)?;
        write_block_position(
            writer,
            self.respawn_data.x,
            self.respawn_data.y,
            self.respawn_data.z,
        )?;
        write_f32(writer, self.respawn_data.yaw)?;
        write_f32(writer, self.respawn_data.pitch)
    }
}
