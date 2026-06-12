use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundTrackedWaypointPacket {
    pub operation: TrackedWaypointOperation,
    pub waypoint: TrackedWaypoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedWaypointOperation {
    Track,
    Untrack,
    Update,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaypointIcon {
    pub style: Identifier,
    pub color: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackedWaypoint {
    pub identifier: TrackedWaypointIdentifier,
    pub icon: WaypointIcon,
    pub kind: TrackedWaypointKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackedWaypointIdentifier {
    Uuid(Uuid),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackedWaypointKind {
    Empty,
    Vec3i { x: i32, y: i32, z: i32 },
    Chunk { x: i32, z: i32 },
    Azimuth { angle: f32 },
}

impl ClientboundTrackedWaypointPacket {
    pub fn remove_waypoint(identifier: Uuid) -> Self {
        Self {
            operation: TrackedWaypointOperation::Untrack,
            waypoint: TrackedWaypoint::empty(identifier),
        }
    }

    pub fn add_waypoint_position(identifier: Uuid, icon: WaypointIcon, x: i32, y: i32, z: i32) -> Self {
        Self {
            operation: TrackedWaypointOperation::Track,
            waypoint: TrackedWaypoint::set_position(identifier, icon, x, y, z),
        }
    }

    pub fn update_waypoint_position(
        identifier: Uuid,
        icon: WaypointIcon,
        x: i32,
        y: i32,
        z: i32,
    ) -> Self {
        Self {
            operation: TrackedWaypointOperation::Update,
            waypoint: TrackedWaypoint::set_position(identifier, icon, x, y, z),
        }
    }

    pub fn add_waypoint_chunk(identifier: Uuid, icon: WaypointIcon, x: i32, z: i32) -> Self {
        Self {
            operation: TrackedWaypointOperation::Track,
            waypoint: TrackedWaypoint::set_chunk(identifier, icon, x, z),
        }
    }

    pub fn update_waypoint_chunk(identifier: Uuid, icon: WaypointIcon, x: i32, z: i32) -> Self {
        Self {
            operation: TrackedWaypointOperation::Update,
            waypoint: TrackedWaypoint::set_chunk(identifier, icon, x, z),
        }
    }

    pub fn add_waypoint_azimuth(identifier: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self {
            operation: TrackedWaypointOperation::Track,
            waypoint: TrackedWaypoint::set_azimuth(identifier, icon, angle),
        }
    }

    pub fn update_waypoint_azimuth(identifier: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self {
            operation: TrackedWaypointOperation::Update,
            waypoint: TrackedWaypoint::set_azimuth(identifier, icon, angle),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            operation: TrackedWaypointOperation::read(reader)?,
            waypoint: TrackedWaypoint::read(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.operation.write(writer)?;
        self.waypoint.write(writer)
    }
}

impl TrackedWaypointOperation {
    const VARIANT_COUNT: usize = 3;

    fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Untrack,
            2 => Self::Update,
            _ => Self::Track,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Track => 0,
            Self::Untrack => 1,
            Self::Update => 2,
        }
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let index = read_var_i32(reader)?.rem_euclid(Self::VARIANT_COUNT as i32) as usize;
        Ok(Self::from_index(index))
    }

    fn write<W: Write>(self, writer: &mut W) -> io::Result<()> {
        write_enum_index(writer, self.index(), Self::VARIANT_COUNT)
    }
}

impl WaypointIcon {
    pub fn null() -> Self {
        Self {
            style: match Identifier::parse("minecraft:default") {
                Ok(identifier) => identifier,
                Err(err) => panic!("invalid static waypoint style id: {err}"),
            },
            color: None,
        }
    }

    pub fn new(style: Identifier, color: Option<i32>) -> Self {
        Self { style, color }
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            style: read_identifier(reader)?,
            color: read_optional(reader, read_rgb_color)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.style)?;
        write_optional(writer, self.color.as_ref(), |writer, color| {
            write_rgb_color(writer, *color)
        })
    }
}

impl TrackedWaypoint {
    pub fn empty(identifier: Uuid) -> Self {
        Self {
            identifier: TrackedWaypointIdentifier::Uuid(identifier),
            icon: WaypointIcon::null(),
            kind: TrackedWaypointKind::Empty,
        }
    }

    pub fn set_position(identifier: Uuid, icon: WaypointIcon, x: i32, y: i32, z: i32) -> Self {
        Self {
            identifier: TrackedWaypointIdentifier::Uuid(identifier),
            icon,
            kind: TrackedWaypointKind::Vec3i { x, y, z },
        }
    }

    pub fn set_chunk(identifier: Uuid, icon: WaypointIcon, x: i32, z: i32) -> Self {
        Self {
            identifier: TrackedWaypointIdentifier::Uuid(identifier),
            icon,
            kind: TrackedWaypointKind::Chunk { x, z },
        }
    }

    pub fn set_azimuth(identifier: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        Self {
            identifier: TrackedWaypointIdentifier::Uuid(identifier),
            icon,
            kind: TrackedWaypointKind::Azimuth { angle },
        }
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let identifier = TrackedWaypointIdentifier::read(reader)?;
        let icon = WaypointIcon::read(reader)?;
        let kind = TrackedWaypointKind::read(reader)?;
        Ok(Self {
            identifier,
            icon,
            kind,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.identifier.write(writer)?;
        self.icon.write(writer)?;
        self.kind.write(writer)
    }
}

impl TrackedWaypointIdentifier {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        if read_bool(reader)? {
            Ok(Self::Uuid(read_uuid(reader)?))
        } else {
            Ok(Self::String(read_string(reader, 32767)?))
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Uuid(value) => {
                write_bool(writer, true)?;
                write_uuid(writer, *value)
            }
            Self::String(value) => {
                write_bool(writer, false)?;
                write_string(writer, value, 32767)
            }
        }
    }
}

impl TrackedWaypointKind {
    const VARIANT_COUNT: usize = 4;

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        match read_enum_index(reader, Self::VARIANT_COUNT)? {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Vec3i {
                x: read_var_i32(reader)?,
                y: read_var_i32(reader)?,
                z: read_var_i32(reader)?,
            }),
            2 => Ok(Self::Chunk {
                x: read_var_i32(reader)?,
                z: read_var_i32(reader)?,
            }),
            3 => Ok(Self::Azimuth {
                angle: read_f32(reader)?,
            }),
            _ => unreachable!("read_enum_index returns an in-range index"),
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_enum_index(writer, self.index(), Self::VARIANT_COUNT)?;
        match self {
            Self::Empty => Ok(()),
            Self::Vec3i { x, y, z } => {
                write_var_i32(writer, *x)?;
                write_var_i32(writer, *y)?;
                write_var_i32(writer, *z)
            }
            Self::Chunk { x, z } => {
                write_var_i32(writer, *x)?;
                write_var_i32(writer, *z)
            }
            Self::Azimuth { angle } => write_f32(writer, *angle),
        }
    }

    fn index(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Vec3i { .. } => 1,
            Self::Chunk { .. } => 2,
            Self::Azimuth { .. } => 3,
        }
    }
}

fn read_rgb_color<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut rgb = [0u8; 3];
    reader.read_exact(&mut rgb)?;
    Ok(i32::from_be_bytes([0xff, rgb[0], rgb[1], rgb[2]]))
}

fn write_rgb_color<W: Write>(writer: &mut W, value: i32) -> io::Result<()> {
    let bytes = value.to_be_bytes();
    writer.write_all(&bytes[1..])
}

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0] != 0)
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

fn write_f32<W: Write>(writer: &mut W, value: f32) -> io::Result<()> {
    writer.write_all(&value.to_be_bytes())
}
