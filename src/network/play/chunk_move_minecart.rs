#![allow(dead_code)]

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundMoveMinecartPacket {
    pub entity_id: i32,
    pub lerp_steps: Vec<MinecartLerpStep>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartLerpStep {
    pub position: Vec3,
    pub movement: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub weight: f32,
}

impl ClientboundMoveMinecartPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entity_id: read_var_i32(reader)?,
            lerp_steps: read_collection(reader, MinecartLerpStep::read)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_collection(writer, &self.lerp_steps, |writer, step| step.write(writer))
    }
}

impl MinecartLerpStep {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            position: read_vec3(reader)?,
            movement: read_vec3(reader)?,
            y_rot: unpack_degrees(read_u8(reader)?),
            x_rot: unpack_degrees(read_u8(reader)?),
            weight: read_f32(reader)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_vec3(writer, self.position)?;
        write_vec3(writer, self.movement)?;
        writer.write_all(&[pack_degrees(self.y_rot)])?;
        writer.write_all(&[pack_degrees(self.x_rot)])?;
        write_f32(writer, self.weight)
    }
}

fn read_vec3<R: Read>(reader: &mut R) -> io::Result<Vec3> {
    Ok(Vec3 {
        x: read_f64(reader)?,
        y: read_f64(reader)?,
        z: read_f64(reader)?,
    })
}

fn unpack_degrees(packed: u8) -> f32 {
    f32::from(packed as i8) * 360.0 / 256.0
}
