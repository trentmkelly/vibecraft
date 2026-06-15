use super::*;

impl ClientboundMoveEntityPacket {
    pub fn read_pos<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            id: read_var_i32(reader)?,
            delta: [read_i16(reader)?, read_i16(reader)?, read_i16(reader)?],
            y_rot: 0,
            x_rot: 0,
            on_ground: read_bool(reader)?,
            has_position: true,
            has_rotation: false,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn read_pos_rot<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            id: read_var_i32(reader)?,
            delta: [read_i16(reader)?, read_i16(reader)?, read_i16(reader)?],
            y_rot: read_u8(reader)?,
            x_rot: read_u8(reader)?,
            on_ground: read_bool(reader)?,
            has_position: true,
            has_rotation: true,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn read_rot<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            id: read_var_i32(reader)?,
            delta: [0, 0, 0],
            y_rot: read_u8(reader)?,
            x_rot: read_u8(reader)?,
            on_ground: read_bool(reader)?,
            has_position: false,
            has_rotation: true,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }
}
