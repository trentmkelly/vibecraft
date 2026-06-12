use super::*;

impl ClientboundLevelChunkWithLightPacket {
    pub fn from_chunk(chunk: &LevelChunk, light_data: ClientboundLightUpdatePacketData) -> Self {
        Self {
            pos: chunk.pos,
            chunk_data: Some(ClientboundLevelChunkPacketData::from_chunk(chunk)),
            light_data: Some(light_data),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.pos.x)?;
        write_i32(writer, self.pos.z)?;
        self.chunk_data
            .as_ref()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "chunk-with-light packet missing chunk data",
                )
            })?
            .write(writer)?;
        self.light_data
            .as_ref()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "chunk-with-light packet missing light data",
                )
            })?
            .write(writer)
    }
}

impl ClientboundForgetLevelChunkPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            pos: unpack_chunk_pos(read_i64(reader)?),
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, pack_chunk_pos(self.pos))
    }
}

fn pack_chunk_pos(pos: ChunkPos) -> i64 {
    (i64::from(pos.x) & 0xffff_ffff) | ((i64::from(pos.z) & 0xffff_ffff) << 32)
}

fn unpack_chunk_pos(packed: i64) -> ChunkPos {
    ChunkPos {
        x: packed as i32,
        z: (packed >> 32) as i32,
    }
}

impl ClientboundLightUpdatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.pos.x)?;
        write_var_i32(writer, self.pos.z)?;
        self.light_data.write(writer)
    }
}

impl SectionPos {
    pub fn packed_long(self) -> i64 {
        let x = (self.x as i64) & 0x3f_ffff;
        let y = (self.y as i64) & 0x0f_ffff;
        let z = (self.z as i64) & 0x3f_ffff;
        (x << 42) | y | (z << 20)
    }
}

impl ClientboundSectionBlocksUpdatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, self.section_pos.packed_long())?;
        write_var_i32(writer, self.updates.len() as i32)?;
        for update in &self.updates {
            if update.packed_pos > 0x0fff {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "section-relative block position exceeds 12 bits",
                ));
            }
            let packed_change =
                (i64::from(update.block_state_id) << 12) | i64::from(update.packed_pos);
            write_var_i64(writer, packed_change)?;
        }
        Ok(())
    }
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
}

impl Default for VecDeltaCodec {
    fn default() -> Self {
        Self { base: Vec3::ZERO }
    }
}

impl VecDeltaCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode(input: f64) -> i64 {
        (input * 4096.0 + 0.5).floor() as i64
    }

    pub fn decode_component(value: i64) -> f64 {
        value as f64 / 4096.0
    }

    pub fn decode(&self, xa: i64, ya: i64, za: i64) -> Vec3 {
        if xa == 0 && ya == 0 && za == 0 {
            return self.base;
        }
        Vec3 {
            x: if xa == 0 {
                self.base.x
            } else {
                Self::decode_component(Self::encode(self.base.x) + xa)
            },
            y: if ya == 0 {
                self.base.y
            } else {
                Self::decode_component(Self::encode(self.base.y) + ya)
            },
            z: if za == 0 {
                self.base.z
            } else {
                Self::decode_component(Self::encode(self.base.z) + za)
            },
        }
    }

    pub fn encode_x(&self, pos: Vec3) -> i64 {
        Self::encode(pos.x) - Self::encode(self.base.x)
    }

    pub fn encode_y(&self, pos: Vec3) -> i64 {
        Self::encode(pos.y) - Self::encode(self.base.y)
    }

    pub fn encode_z(&self, pos: Vec3) -> i64 {
        Self::encode(pos.z) - Self::encode(self.base.z)
    }

    pub fn delta(&self, pos: Vec3) -> Vec3 {
        Vec3 {
            x: pos.x - self.base.x,
            y: pos.y - self.base.y,
            z: pos.z - self.base.z,
        }
    }

    pub fn set_base(&mut self, base: Vec3) {
        self.base = base;
    }

    pub fn base(&self) -> Vec3 {
        self.base
    }
}

impl ClientboundAddEntityPacket {
    pub fn new(input: AddEntityPacketInput) -> Self {
        Self {
            id: input.id,
            uuid: input.uuid,
            entity_type: input.entity_type,
            position: input.position,
            movement: input.movement,
            x_rot: pack_degrees(input.rotation.0),
            y_rot: pack_degrees(input.rotation.1),
            y_head_rot: pack_degrees(input.y_head_rot),
            data: input.data,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_uuid(writer, self.uuid)?;
        write_var_i32(writer, self.entity_type)?;
        write_f64(writer, self.position.x)?;
        write_f64(writer, self.position.y)?;
        write_f64(writer, self.position.z)?;
        write_lp_vec3(writer, self.movement)?;
        writer.write_all(&[self.x_rot, self.y_rot, self.y_head_rot])?;
        write_var_i32(writer, self.data)
    }
}

impl ClientboundRemoveEntitiesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_ids.len() as i32)?;
        for id in &self.entity_ids {
            write_var_i32(writer, *id)?;
        }
        Ok(())
    }
}

impl ClientboundTakeItemEntityPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.item_entity_id)?;
        write_var_i32(writer, self.collector_entity_id)?;
        write_var_i32(writer, self.amount)
    }
}

impl ClientboundSetPlayerInventoryPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)?;
        self.contents.write_optional_trusted(writer)
    }
}

impl ClientboundSetEntityDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        for item in &self.packed_items {
            item.write(writer)?;
        }
        writer.write_all(&[0xff])
    }
}

impl EntityDataValue {
    pub fn typed(index: u8, value: EntityMetadataValue) -> io::Result<Self> {
        let serializer_id = value.serializer_id();
        let mut encoded_payload = Vec::new();
        value.write_payload(&mut encoded_payload)?;
        Ok(Self {
            index,
            serializer_id,
            encoded_payload,
        })
    }

    pub fn raw(index: u8, serializer_id: i32, encoded_payload: Vec<u8>) -> Self {
        Self {
            index,
            serializer_id,
            encoded_payload,
        }
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.index == 0xff {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "entity metadata index 255 is reserved as EOF",
            ));
        }
        writer.write_all(&[self.index])?;
        write_var_i32(writer, self.serializer_id)?;
        writer.write_all(&self.encoded_payload)
    }
}

impl EntityMetadataValue {
    pub(crate) fn serializer_id(&self) -> i32 {
        match self {
            Self::Byte(_) => 0,
            Self::VarInt(_) => 1,
            Self::VarLong(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::Component(_) => 5,
            Self::OptionalComponent(_) => 6,
            Self::ItemStack(_) => 7,
            Self::Boolean(_) => 8,
            Self::Rotations(_) => 9,
            Self::BlockPos(_) => 10,
            Self::OptionalBlockPos(_) => 11,
            Self::Direction(_) => 12,
            Self::OptionalLivingEntityReference(_) => 13,
            Self::BlockState(_) => 14,
            Self::OptionalBlockState(_) => 15,
            Self::Particle(_) => 16,
            Self::Particles(_) => 17,
            Self::VillagerData(_) => 18,
            Self::OptionalUnsignedInt(_) => 19,
            Self::Pose(_) => 20,
            Self::CatVariant(_) => 21,
            Self::CatSoundVariant(_) => 22,
            Self::CowVariant(_) => 23,
            Self::CowSoundVariant(_) => 24,
            Self::WolfVariant(_) => 25,
            Self::WolfSoundVariant(_) => 26,
            Self::FrogVariant(_) => 27,
            Self::PigVariant(_) => 28,
            Self::PigSoundVariant(_) => 29,
            Self::ChickenVariant(_) => 30,
            Self::ChickenSoundVariant(_) => 31,
            Self::ZombieNautilusVariant(_) => 32,
            Self::OptionalGlobalPos(_) => 33,
            Self::PaintingVariant(_) => 34,
            Self::SnifferState(_) => 35,
            Self::ArmadilloState(_) => 36,
            Self::CopperGolemState(_) => 37,
            Self::WeatheringCopperState(_) => 38,
            Self::Vector3f(_) => 39,
            Self::Quaternionf(_) => 40,
            Self::ResolvableProfile(_) => 41,
            Self::HumanoidArm(_) => 42,
            Self::Raw { serializer_id, .. } => *serializer_id,
        }
    }

    pub(crate) fn write_payload<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Byte(value) => write_i8(writer, *value),
            Self::VarInt(value) => write_var_i32(writer, *value),
            Self::VarLong(value) => write_var_i64(writer, *value),
            Self::Float(value) => write_f32(writer, *value),
            Self::String(value) => write_string(writer, value, 32767),
            Self::Component(payload) | Self::ResolvableProfile(payload) => {
                writer.write_all(payload)
            }
            Self::OptionalComponent(value) => write_optional_raw_payload(writer, value.as_deref()),
            Self::ItemStack(stack) => stack.write_optional_trusted(writer),
            Self::Boolean(value) => write_bool(writer, *value),
            Self::Rotations(value) => {
                write_f32(writer, value.x)?;
                write_f32(writer, value.y)?;
                write_f32(writer, value.z)
            }
            Self::BlockPos(pos) => write_metadata_block_pos(writer, *pos),
            Self::OptionalBlockPos(pos) => write_optional(writer, pos.as_ref(), |writer, pos| {
                write_metadata_block_pos(writer, *pos)
            }),
            Self::Direction(direction) => write_var_i32(writer, direction.id()),
            Self::OptionalLivingEntityReference(entity_id) => {
                write_optional_entity_reference(writer, *entity_id)
            }
            Self::BlockState(state_id) => write_var_i32(writer, *state_id),
            Self::OptionalBlockState(state_id) => write_var_i32(writer, state_id.unwrap_or(0)),
            Self::Particle(particle) => particle.write(writer),
            Self::Particles(particles) => {
                write_var_i32(writer, particles.len() as i32)?;
                for particle in particles {
                    particle.write(writer)?;
                }
                Ok(())
            }
            Self::VillagerData(data) => {
                write_var_i32(writer, data.villager_type)?;
                write_var_i32(writer, data.profession)?;
                write_var_i32(writer, data.level)
            }
            Self::OptionalUnsignedInt(value) => write_var_i32(writer, value.map_or(0, |v| v + 1)),
            Self::Pose(pose) => write_var_i32(writer, pose.id()),
            Self::CatVariant(id)
            | Self::CatSoundVariant(id)
            | Self::CowVariant(id)
            | Self::CowSoundVariant(id)
            | Self::WolfVariant(id)
            | Self::WolfSoundVariant(id)
            | Self::FrogVariant(id)
            | Self::PigVariant(id)
            | Self::PigSoundVariant(id)
            | Self::ChickenVariant(id)
            | Self::ChickenSoundVariant(id)
            | Self::ZombieNautilusVariant(id)
            | Self::PaintingVariant(id) => write_var_i32(writer, *id),
            Self::OptionalGlobalPos(global_pos) => {
                write_optional(writer, global_pos.as_ref(), |writer, global_pos| {
                    write_identifier(writer, &global_pos.dimension)?;
                    write_metadata_block_pos(writer, global_pos.pos)
                })
            }
            Self::SnifferState(state) => write_var_i32(writer, state.id()),
            Self::ArmadilloState(state) => write_var_i32(writer, state.id()),
            Self::CopperGolemState(state) => write_var_i32(writer, state.id()),
            Self::WeatheringCopperState(state) => write_var_i32(writer, state.id()),
            Self::Vector3f(value) => {
                write_f32(writer, value.x)?;
                write_f32(writer, value.y)?;
                write_f32(writer, value.z)
            }
            Self::Quaternionf(value) => {
                write_f32(writer, value.x)?;
                write_f32(writer, value.y)?;
                write_f32(writer, value.z)?;
                write_f32(writer, value.w)
            }
            Self::HumanoidArm(arm) => write_var_i32(writer, arm.id()),
            Self::Raw {
                encoded_payload, ..
            } => writer.write_all(encoded_payload),
        }
    }
}

impl DirectionData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }
}

impl PoseData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Standing => 0,
            Self::FallFlying => 1,
            Self::Sleeping => 2,
            Self::Swimming => 3,
            Self::SpinAttack => 4,
            Self::Crouching => 5,
            Self::LongJumping => 6,
            Self::Dying => 7,
            Self::Croaking => 8,
            Self::UsingTongue => 9,
            Self::Sitting => 10,
            Self::Roaring => 11,
            Self::Sniffing => 12,
            Self::Emerging => 13,
            Self::Digging => 14,
            Self::Sliding => 15,
            Self::Shooting => 16,
            Self::Inhaling => 17,
        }
    }
}

impl SnifferStateData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Idling => 0,
            Self::FeelingHappy => 1,
            Self::Scenting => 2,
            Self::Sniffing => 3,
            Self::Searching => 4,
            Self::Digging => 5,
            Self::Rising => 6,
        }
    }
}

impl ArmadilloStateData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 1,
            Self::Scared => 2,
            Self::Unrolling => 3,
        }
    }
}

impl CopperGolemStateData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Unoxidized => 0,
            Self::Exposed => 1,
            Self::Weathered => 2,
            Self::Oxidized => 3,
        }
    }
}

impl WeatheringCopperStateData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Unaffected => 0,
            Self::Exposed => 1,
            Self::Weathered => 2,
            Self::Oxidized => 3,
        }
    }
}

impl HumanoidArmData {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}

pub(super) fn write_metadata_block_pos<W: Write>(
    writer: &mut W,
    pos: BlockPosition,
) -> io::Result<()> {
    write_block_position(writer, pos.x, pos.y, pos.z)
}

pub(super) fn write_optional_raw_payload<W: Write>(
    writer: &mut W,
    payload: Option<&[u8]>,
) -> io::Result<()> {
    match payload {
        Some(payload) => {
            write_bool(writer, true)?;
            writer.write_all(payload)
        }
        None => write_bool(writer, false),
    }
}

pub(super) fn write_optional_entity_reference<W: Write>(
    writer: &mut W,
    entity_id: Option<i32>,
) -> io::Result<()> {
    match entity_id {
        Some(entity_id) => {
            write_bool(writer, true)?;
            write_var_i32(writer, entity_id)
        }
        None => write_bool(writer, false),
    }
}

impl ClientboundMoveEntityPacket {
    pub fn pos(id: i32, delta: [i16; 3], on_ground: bool) -> Self {
        Self {
            id,
            delta,
            y_rot: 0,
            x_rot: 0,
            on_ground,
            has_position: true,
            has_rotation: false,
        }
    }

    pub fn pos_rot(id: i32, delta: [i16; 3], y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self {
            id,
            delta,
            y_rot: pack_degrees(y_rot),
            x_rot: pack_degrees(x_rot),
            on_ground,
            has_position: true,
            has_rotation: true,
        }
    }

    pub fn rot(id: i32, y_rot: f32, x_rot: f32, on_ground: bool) -> Self {
        Self {
            id,
            delta: [0, 0, 0],
            y_rot: pack_degrees(y_rot),
            x_rot: pack_degrees(x_rot),
            on_ground,
            has_position: false,
            has_rotation: true,
        }
    }

    pub(super) fn write_delta<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i16(writer, self.delta[0])?;
        write_i16(writer, self.delta[1])?;
        write_i16(writer, self.delta[2])
    }

    pub fn write_pos<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        self.write_delta(writer)?;
        write_bool(writer, self.on_ground)
    }

    pub fn write_pos_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        self.write_delta(writer)?;
        writer.write_all(&[self.y_rot, self.x_rot])?;
        write_bool(writer, self.on_ground)
    }

    pub fn write_rot<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        writer.write_all(&[self.y_rot, self.x_rot])?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundMoveVehiclePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            position: Vec3 {
                x: read_f64(reader)?,
                y: read_f64(reader)?,
                z: read_f64(reader)?,
            },
            y_rot: read_f32(reader)?,
            x_rot: read_f32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_vec3(writer, self.position)?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)
    }
}

pub(super) fn write_position_move_rotation<W: Write>(
    writer: &mut W,
    position: Vec3,
    movement: Vec3,
    y_rot: f32,
    x_rot: f32,
) -> io::Result<()> {
    write_vec3(writer, position)?;
    write_vec3(writer, movement)?;
    write_f32(writer, y_rot)?;
    write_f32(writer, x_rot)
}

pub(super) fn read_position_move_rotation<R: Read>(
    reader: &mut R,
) -> io::Result<(Vec3, Vec3, f32, f32)> {
    Ok((
        Vec3 {
            x: read_f64(reader)?,
            y: read_f64(reader)?,
            z: read_f64(reader)?,
        },
        Vec3 {
            x: read_f64(reader)?,
            y: read_f64(reader)?,
            z: read_f64(reader)?,
        },
        read_f32(reader)?,
        read_f32(reader)?,
    ))
}

const RELATIVE_FLAGS_MASK: u32 = 0x1ff;

impl ClientboundTeleportEntityPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_i32(writer, (self.relative_flags & RELATIVE_FLAGS_MASK) as i32)?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundEntityPositionSyncPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_bool(writer, self.on_ground)
    }
}

impl ClientboundPlayerPositionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let id = read_var_i32(reader)?;
        let (position, movement, y_rot, x_rot) = read_position_move_rotation(reader)?;
        let relative_flags = read_i32(reader)? as u32 & RELATIVE_FLAGS_MASK;
        expect_empty_payload(reader)?;
        Ok(Self {
            id,
            position,
            movement,
            y_rot,
            x_rot,
            relative_flags,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_position_move_rotation(writer, self.position, self.movement, self.y_rot, self.x_rot)?;
        write_i32(writer, (self.relative_flags & RELATIVE_FLAGS_MASK) as i32)
    }
}

impl ClientboundSetEntityMotionPacket {
    pub fn new(id: i32, movement: Vec3) -> Self {
        Self { id, movement }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_lp_vec3(writer, self.movement)
    }
}

impl ClientboundRotateHeadPacket {
    pub fn new(id: i32, y_head_rot: f32) -> Self {
        Self {
            id,
            y_head_rot: pack_degrees(y_head_rot),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_i8(writer, self.y_head_rot as i8)
    }
}

impl ClientboundSetPassengersPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.vehicle)?;
        write_var_i32(writer, self.passengers.len() as i32)?;
        for passenger in &self.passengers {
            write_var_i32(writer, *passenger)?;
        }
        Ok(())
    }
}

impl ClientboundEntityEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.entity_id)?;
        write_i8(writer, self.event_id)
    }
}

impl ClientboundAnimatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        writer.write_all(&[self.action as u8])
    }
}

impl ClientboundInitializeBorderPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.new_center_x)?;
        write_f64(writer, self.new_center_z)?;
        write_f64(writer, self.old_size)?;
        write_f64(writer, self.new_size)?;
        write_var_i64(writer, self.lerp_time)?;
        write_var_i32(writer, self.new_absolute_max_size)?;
        write_var_i32(writer, self.warning_blocks)?;
        write_var_i32(writer, self.warning_time)
    }
}

impl ClientboundSetBorderCenterPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            new_center_x: read_f64(reader)?,
            new_center_z: read_f64(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.new_center_x)?;
        write_f64(writer, self.new_center_z)
    }
}

impl ClientboundSetBorderLerpSizePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            old_size: read_f64(reader)?,
            new_size: read_f64(reader)?,
            lerp_time: read_var_i64(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.old_size)?;
        write_f64(writer, self.new_size)?;
        write_var_i64(writer, self.lerp_time)
    }
}

impl ClientboundSetBorderSizePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            size: read_f64(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f64(writer, self.size)
    }
}

impl ClientboundSetBorderWarningDelayPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            warning_delay: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.warning_delay)
    }
}

impl ClientboundSetBorderWarningDistancePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            warning_blocks: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.warning_blocks)
    }
}

impl ClientboundSetCameraPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            camera_id: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.camera_id)
    }
}

impl ClientboundClearTitlesPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.reset_times)
    }
}

impl ClientboundSetTitlesAnimationPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.fade_in)?;
        write_i32(writer, self.stay)?;
        write_i32(writer, self.fade_out)
    }
}

impl SoundEventHolder {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Registered { id } => write_var_i32(writer, id + 1),
            Self::Direct {
                location,
                fixed_range,
            } => {
                write_var_i32(writer, 0)?;
                write_identifier(writer, location)?;
                write_optional(writer, fixed_range.as_ref(), |writer, range| {
                    write_f32(writer, *range)
                })
            }
        }
    }
}

impl ClientboundSoundPacket {
    pub fn write_position<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.sound.write(writer)?;
        write_var_i32(writer, self.source_id)?;
        write_i32(writer, (self.position.x * 8.0) as i32)?;
        write_i32(writer, (self.position.y * 8.0) as i32)?;
        write_i32(writer, (self.position.z * 8.0) as i32)?;
        write_f32(writer, self.volume)?;
        write_f32(writer, self.pitch)?;
        write_i64(writer, self.seed)
    }

    pub fn write_entity<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let entity_id = self.entity_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "sound entity packet requires an entity id",
            )
        })?;
        self.sound.write(writer)?;
        write_var_i32(writer, self.source_id)?;
        write_var_i32(writer, entity_id)?;
        write_f32(writer, self.volume)?;
        write_f32(writer, self.pitch)?;
        write_i64(writer, self.seed)
    }
}

impl ClientboundParticlePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.override_limiter)?;
        write_bool(writer, self.always_show)?;
        write_vec3(writer, self.position)?;
        write_f32(writer, self.offset.x as f32)?;
        write_f32(writer, self.offset.y as f32)?;
        write_f32(writer, self.offset.z as f32)?;
        write_f32(writer, self.max_speed)?;
        write_i32(writer, self.count)?;
        write_var_i32(writer, self.particle_id)?;
        writer.write_all(&self.particle_data)
    }
}

impl RawParticleOptions {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.particle_id)?;
        writer.write_all(&self.data)
    }
}

impl ExplosionParticleInfo {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.particle.write(writer)?;
        write_f32(writer, self.scaling)?;
        write_f32(writer, self.speed)
    }
}

impl WeightedExplosionParticle {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.value.write(writer)?;
        write_var_i32(writer, self.weight)
    }
}

impl ClientboundExplodePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_vec3(writer, self.center)?;
        write_f32(writer, self.radius)?;
        write_i32(writer, self.block_count)?;
        write_optional(
            writer,
            self.player_knockback.as_ref(),
            |writer, knockback| write_vec3(writer, *knockback),
        )?;
        self.explosion_particle.write(writer)?;
        self.explosion_sound.write(writer)?;
        write_var_i32(writer, self.block_particles.len() as i32)?;
        for particle in &self.block_particles {
            particle.write(writer)?;
        }
        Ok(())
    }
}

impl ClientboundSetEntityLinkPacket {
    pub fn new(source_id: i32, dest_id: Option<i32>) -> Self {
        Self {
            source_id,
            dest_id: dest_id.unwrap_or(0),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.source_id)?;
        write_i32(writer, self.dest_id)
    }
}

impl ClientboundSetEquipmentPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity)?;
        for (index, entry) in self.slots.iter().enumerate() {
            let slot = entry.slot as u8;
            let encoded_slot = if index + 1 == self.slots.len() {
                slot
            } else {
                slot | 0x80
            };
            writer.write_all(&[encoded_slot])?;
            entry.item_stack.write_optional_trusted(writer)?;
        }
        Ok(())
    }

    pub fn encoded_slot_bytes(&self) -> Vec<u8> {
        self.slots
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let slot = entry.slot as u8;
                if index + 1 == self.slots.len() {
                    slot
                } else {
                    slot | 0x80
                }
            })
            .collect()
    }
}

impl ClientboundRecipeBookAddPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.entries, |writer, entry| entry.write(writer))?;
        write_bool(writer, self.replace)
    }
}

impl ClientboundDamageEventPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entity_id: read_var_i32(reader)?,
            source_type_id: read_var_i32(reader)?,
            source_cause_id: read_var_i32(reader)? - 1,
            source_direct_id: read_var_i32(reader)? - 1,
            source_position: read_optional(reader, |reader| {
                Ok(Vec3 {
                    x: read_f64(reader)?,
                    y: read_f64(reader)?,
                    z: read_f64(reader)?,
                })
            })?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.source_type_id)?;
        write_var_i32(writer, self.source_cause_id + 1)?;
        write_var_i32(writer, self.source_direct_id + 1)?;
        write_optional(writer, self.source_position.as_ref(), |writer, position| {
            write_vec3(writer, *position)
        })
    }
}

impl ClientboundPlaceGhostRecipePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        self.recipe_display.write(writer)
    }
}

impl RecipeBookAddEntry {
    pub const FLAG_NOTIFICATION: u8 = 1;
    pub const FLAG_HIGHLIGHT: u8 = 2;

    pub fn new(contents: RecipeDisplayEntryData, notification: bool, highlight: bool) -> Self {
        Self {
            contents,
            flags: u8::from(notification) | (u8::from(highlight) << 1),
        }
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.contents.write(writer)?;
        writer.write_all(&[self.flags])
    }
}

impl RecipeDisplayEntryData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        self.display.write(writer)?;
        write_optional_var_i32(writer, self.group)?;
        write_var_i32(writer, self.category_id)?;
        write_optional(
            writer,
            self.crafting_requirements.as_ref(),
            |writer, requirements| {
                write_collection(writer, requirements, |writer, ingredient| {
                    ingredient.write(writer)
                })
            },
        )
    }
}

impl RecipeDisplayData {
    const CRAFTING_SHAPELESS_TYPE_ID: i32 = 0;
    const CRAFTING_SHAPED_TYPE_ID: i32 = 1;
    const FURNACE_TYPE_ID: i32 = 2;
    const STONECUTTER_TYPE_ID: i32 = 3;
    const SMITHING_TYPE_ID: i32 = 4;

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::CraftingShapeless {
                ingredients,
                result,
                crafting_station,
            } => {
                write_var_i32(writer, Self::CRAFTING_SHAPELESS_TYPE_ID)?;
                write_collection(writer, ingredients, |writer, ingredient| {
                    ingredient.write(writer)
                })?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::CraftingShaped {
                width,
                height,
                ingredients,
                result,
                crafting_station,
            } => {
                if ingredients.len() != (*width as usize).saturating_mul(*height as usize) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "shaped recipe display ingredients must match width * height",
                    ));
                }
                write_var_i32(writer, Self::CRAFTING_SHAPED_TYPE_ID)?;
                write_var_i32(writer, *width)?;
                write_var_i32(writer, *height)?;
                write_collection(writer, ingredients, |writer, ingredient| {
                    ingredient.write(writer)
                })?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::Furnace {
                ingredient,
                fuel,
                result,
                crafting_station,
                duration,
                experience_bits,
            } => {
                write_var_i32(writer, Self::FURNACE_TYPE_ID)?;
                ingredient.write(writer)?;
                fuel.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)?;
                write_var_i32(writer, *duration)?;
                writer.write_all(&experience_bits.to_be_bytes())
            }
            Self::Stonecutter {
                ingredient,
                result,
                crafting_station,
            } => {
                write_var_i32(writer, Self::STONECUTTER_TYPE_ID)?;
                ingredient.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
            Self::Smithing {
                template,
                base,
                addition,
                result,
                crafting_station,
            } => {
                write_var_i32(writer, Self::SMITHING_TYPE_ID)?;
                template.write(writer)?;
                base.write(writer)?;
                addition.write(writer)?;
                result.write(writer)?;
                crafting_station.write(writer)
            }
        }
    }
}
