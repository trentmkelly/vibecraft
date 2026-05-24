use super::*;


pub fn block_state_name_network_id(name: &str) -> Option<i32> {
    if let Some(id) = liquid_block_state_name_network_id(name) {
        return Some(id);
    }
    Some(match name {
        "minecraft:air" => 0,
        "minecraft:stone" => 1,
        "minecraft:granite" => 2,
        "minecraft:diorite" => 4,
        "minecraft:andesite" => 6,
        "minecraft:grass_block" => 9,
        "minecraft:dirt" => 10,
        "minecraft:coarse_dirt" => 11,
        "minecraft:podzol" => 13,
        "minecraft:cobblestone" => 14,
        "minecraft:lava" => 102,
        "minecraft:sand" => 118,
        "minecraft:red_sand" => 123,
        "minecraft:gravel" => 124,
        "minecraft:gold_ore" => 129,
        "minecraft:deepslate_gold_ore" => 130,
        "minecraft:iron_ore" => 131,
        "minecraft:deepslate_iron_ore" => 132,
        "minecraft:coal_ore" => 133,
        "minecraft:deepslate_coal_ore" => 134,
        "minecraft:sandstone" => 578,
        "minecraft:lapis_ore" => 563,
        "minecraft:deepslate_lapis_ore" => 564,
        "minecraft:obsidian" => 3369,
        "minecraft:diamond_ore" => 5307,
        "minecraft:deepslate_diamond_ore" => 5308,
        "minecraft:redstone_ore" => 6882,
        "minecraft:deepslate_redstone_ore" => 6884,
        "minecraft:snow" => 6919,
        "minecraft:ice" => 6927,
        "minecraft:snow_block" => 6928,
        "minecraft:clay" => 6946,
        "minecraft:mycelium" => 8919,
        "minecraft:emerald_ore" => 9573,
        "minecraft:deepslate_emerald_ore" => 9574,
        "minecraft:water" => 86,
        "minecraft:white_terracotta" => 11444,
        "minecraft:orange_terracotta" => 11445,
        "minecraft:yellow_terracotta" => 11448,
        "minecraft:light_gray_terracotta" => 11452,
        "minecraft:brown_terracotta" => 11456,
        "minecraft:red_terracotta" => 11458,
        "minecraft:terracotta" => 12912,
        "minecraft:packed_ice" => 12914,
        "minecraft:red_sandstone" => 13247,
        "minecraft:magma_block" => 14845,
        "minecraft:void_air" => 15292,
        "minecraft:cave_air" => 15293,
        "minecraft:blue_ice" => 15275,
        "minecraft:oak_log" => 137,
        "minecraft:oak_leaves" => 279,
        "minecraft:bedrock" => 85,
        "minecraft:tuff" => 23452,
        "minecraft:calcite" => 24687,
        "minecraft:copper_ore" => 25313,
        "minecraft:deepslate_copper_ore" => 25314,
        "minecraft:dripstone_block" => 27755,
        "minecraft:mud" => 27922,
        "minecraft:deepslate" => 27924,
        "minecraft:raw_iron_block" => 29577,
        "minecraft:raw_copper_block" => 29578,
        "minecraft:raw_gold_block" => 29579,
        "minecraft:short_grass" => 2248,
        "minecraft:dandelion" => 2321,
        "minecraft:poppy" => 2324,
        "minecraft:birch_log" => 143,
        "minecraft:birch_leaves" => 335,
        "minecraft:sunflower" => 12916,
        _ => return None,
    })
}

pub(super) fn liquid_block_state_name_network_id(name: &str) -> Option<i32> {
    let (base, properties) = name.split_once('[').unwrap_or((name, ""));
    let start = match base {
        "minecraft:water" => 86,
        "minecraft:lava" => 102,
        _ => return None,
    };
    let level = properties
        .trim_end_matches(']')
        .split(',')
        .find_map(|property| property.strip_prefix("level="))
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0)
        .clamp(0, 15);
    Some(start + level)
}

pub(super) fn biome_name_network_id(name: &str) -> Option<i32> {
    let key = name.strip_prefix("minecraft:").unwrap_or(name);
    crate::network::status::BIOMES
        .iter()
        .position(|biome| *biome == key)
        .map(|index| index as i32)
}

impl ServerboundAcceptTeleportationPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            teleport_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.teleport_id)
    }
}

impl ServerboundChangeDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            difficulty: GameDifficulty::from_wire_index(read_var_i32(reader)?)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.difficulty.to_wire_index())
    }
}

impl ServerboundChatAckPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            offset: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.offset)
    }
}

impl MessageSignature {
    pub const BYTES: usize = 256;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; Self::BYTES];
        reader.read_exact(&mut bytes)?;
        Ok(Self(bytes))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.0)
    }
}

impl PackedMessageSignature {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::CacheId(id) => write_var_i32(writer, id + 1),
            Self::Full(signature) => {
                write_var_i32(writer, 0)?;
                signature.write(writer)
            }
        }
    }
}

impl ClientboundDeleteChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.message_signature.write(writer)
    }
}

impl LastSeenMessagesUpdate {
    pub const ACKNOWLEDGED_BITS: usize = 20;
    pub const ACKNOWLEDGED_BYTES: usize = 3;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let offset = read_var_i32(reader)?;
        let mut acknowledged = vec![0; Self::ACKNOWLEDGED_BYTES];
        reader.read_exact(&mut acknowledged)?;
        Ok(Self {
            offset,
            acknowledged,
            checksum: read_u8(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.acknowledged.len() != Self::ACKNOWLEDGED_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "last-seen acknowledged bitset must be 3 bytes",
            ));
        }
        if self.acknowledged[2] & !0x0f != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "last-seen acknowledged bitset exceeds 20 bits",
            ));
        }
        write_var_i32(writer, self.offset)?;
        writer.write_all(&self.acknowledged)?;
        writer.write_all(&[self.checksum])
    }
}

impl ArgumentSignature {
    pub const MAX_ARGUMENT_NAME_CHARS: usize = 16;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, Self::MAX_ARGUMENT_NAME_CHARS)?,
            signature: MessageSignature::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, Self::MAX_ARGUMENT_NAME_CHARS)?;
        self.signature.write(writer)
    }
}

impl ServerboundChatPacket {
    pub const MAX_MESSAGE_CHARS: usize = 256;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            message: read_string(reader, Self::MAX_MESSAGE_CHARS)?,
            timestamp_epoch_millis: read_i64(reader)?,
            salt: read_i64(reader)?,
            signature: read_nullable_signature(reader)?,
            last_seen_messages: LastSeenMessagesUpdate::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.message, Self::MAX_MESSAGE_CHARS)?;
        write_i64(writer, self.timestamp_epoch_millis)?;
        write_i64(writer, self.salt)?;
        write_nullable_signature(writer, self.signature.as_ref())?;
        self.last_seen_messages.write(writer)
    }
}

impl ServerboundChatCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            command: read_string(reader, 32767)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.command, 32767)
    }
}

impl ServerboundChatCommandSignedPacket {
    pub const MAX_ARGUMENT_SIGNATURES: usize = 8;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            command: read_string(reader, 32767)?,
            timestamp_epoch_millis: read_i64(reader)?,
            salt: read_i64(reader)?,
            argument_signatures: read_limited_collection(
                reader,
                Self::MAX_ARGUMENT_SIGNATURES,
                ArgumentSignature::read,
            )?,
            last_seen_messages: LastSeenMessagesUpdate::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.argument_signatures.len() > Self::MAX_ARGUMENT_SIGNATURES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many argument signatures",
            ));
        }
        write_string(writer, &self.command, 32767)?;
        write_i64(writer, self.timestamp_epoch_millis)?;
        write_i64(writer, self.salt)?;
        write_collection(writer, &self.argument_signatures, |writer, entry| {
            entry.write(writer)
        })?;
        self.last_seen_messages.write(writer)
    }
}

impl ServerboundChatSessionUpdatePacket {
    pub const MAX_PUBLIC_KEY_BYTES: usize = 512;
    pub const MAX_SIGNATURE_BYTES: usize = 4096;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            session_id: read_uuid(reader)?,
            expires_at_epoch_millis: read_i64(reader)?,
            public_key: read_length_prefixed_bytes(reader, Self::MAX_PUBLIC_KEY_BYTES)?,
            key_signature: read_length_prefixed_bytes(reader, Self::MAX_SIGNATURE_BYTES)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, self.session_id)?;
        write_i64(writer, self.expires_at_epoch_millis)?;
        write_length_prefixed_bytes(writer, &self.public_key, Self::MAX_PUBLIC_KEY_BYTES)?;
        write_length_prefixed_bytes(writer, &self.key_signature, Self::MAX_SIGNATURE_BYTES)
    }
}

impl ServerboundClientCommandAction {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::PerformRespawn),
            1 => Ok(Self::RequestStats),
            2 => Ok(Self::RequestGameruleValues),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "client command action index out of range",
            )),
        }
    }

    pub(super) fn to_id(self) -> io::Result<i32> {
        match self {
            Self::PerformRespawn => Ok(0),
            Self::RequestStats => Ok(1),
            Self::RequestGameruleValues => Ok(2),
        }
    }
}

impl ServerboundClientCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            action: ServerboundClientCommandAction::from_id(read_var_i32(reader)?)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.action.to_id()?)
    }
}

impl ServerboundClientTickEndPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundLockDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            locked: read_bool(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.locked)
    }
}

impl ServerboundPaddleBoatPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            left: read_bool(reader)?,
            right: read_bool(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.left)?;
        write_bool(writer, self.right)
    }
}

impl ServerboundPlayerInput {
    pub(super) fn from_flags(flags: u8) -> Self {
        Self {
            forward: flags & 1 != 0,
            backward: flags & 2 != 0,
            left: flags & 4 != 0,
            right: flags & 8 != 0,
            jump: flags & 16 != 0,
            shift: flags & 32 != 0,
            sprint: flags & 64 != 0,
        }
    }

    pub(super) fn to_flags(self) -> u8 {
        (if self.forward { 1 } else { 0 })
            | (if self.backward { 2 } else { 0 })
            | (if self.left { 4 } else { 0 })
            | (if self.right { 8 } else { 0 })
            | (if self.jump { 16 } else { 0 })
            | (if self.shift { 32 } else { 0 })
            | (if self.sprint { 64 } else { 0 })
    }
}

impl ServerboundPlayerInputPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let flags = read_u8(reader)?;
        let packet = Self {
            input: ServerboundPlayerInput::from_flags(flags),
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.input.to_flags()])
    }
}

impl ServerboundPlayerLoadedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundPlayerCommandAction {
    pub(super) fn from_id(id: i32) -> Self {
        match id {
            0 => Self::StopSleeping,
            1 => Self::StartSprinting,
            2 => Self::StopSprinting,
            3 => Self::StartRidingJump,
            4 => Self::StopRidingJump,
            5 => Self::OpenInventory,
            6 => Self::StartFallFlying,
            _ => Self::Unknown(id),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::StopSleeping => 0,
            Self::StartSprinting => 1,
            Self::StopSprinting => 2,
            Self::StartRidingJump => 3,
            Self::StopRidingJump => 4,
            Self::OpenInventory => 5,
            Self::StartFallFlying => 6,
            Self::Unknown(value) => value,
        }
    }
}

impl ServerboundPlayerCommandPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            action: ServerboundPlayerCommandAction::from_id(read_var_i32(reader)?),
            data: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.action.to_id())?;
        write_var_i32(writer, self.data)
    }
}

impl Direction3d {
    pub(super) fn from_id(id: u8) -> Self {
        match id % 6 {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            _ => Self::East,
        }
    }

    pub(super) fn to_id(self) -> u8 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    pub(super) fn from_enum_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Down),
            1 => Ok(Self::Up),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "direction enum index out of range",
            )),
        }
    }

    pub(super) fn to_enum_id(self) -> i32 {
        i32::from(self.to_id())
    }
}

impl ServerboundPlayerAction {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::StartDestroyBlock),
            1 => Ok(Self::AbortDestroyBlock),
            2 => Ok(Self::StopDestroyBlock),
            3 => Ok(Self::DropAllItems),
            4 => Ok(Self::DropItem),
            5 => Ok(Self::ReleaseUseItem),
            6 => Ok(Self::SwapItemWithOffhand),
            7 => Ok(Self::Stab),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "player action index out of range",
            )),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::StartDestroyBlock => 0,
            Self::AbortDestroyBlock => 1,
            Self::StopDestroyBlock => 2,
            Self::DropAllItems => 3,
            Self::DropItem => 4,
            Self::ReleaseUseItem => 5,
            Self::SwapItemWithOffhand => 6,
            Self::Stab => 7,
        }
    }
}

impl ServerboundPlayerActionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let action = ServerboundPlayerAction::from_id(read_var_i32(reader)?)?;
        let (x, y, z) = read_block_position(reader)?;
        let packet = Self {
            action,
            x,
            y,
            z,
            direction: Direction3d::from_id(read_u8(reader)?),
            sequence: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.action.to_id())?;
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.direction.to_id()])?;
        write_var_i32(writer, self.sequence)
    }
}

impl ServerboundSwingHand {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::MainHand),
            1 => Ok(Self::OffHand),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "interaction hand index out of range",
            )),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
        }
    }
}

impl ServerboundSwingPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())
    }
}

impl ServerboundUseItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?)?,
            sequence: read_var_i32(reader)?,
            y_rot: read_f32(reader)?,
            x_rot: read_f32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())?;
        write_var_i32(writer, self.sequence)?;
        write_f32(writer, self.y_rot)?;
        write_f32(writer, self.x_rot)
    }
}

impl BlockHitResultPacketData {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            direction: Direction3d::from_enum_id(read_var_i32(reader)?)?,
            click_x: read_f32(reader)?,
            click_y: read_f32(reader)?,
            click_z: read_f32(reader)?,
            inside: read_bool(reader)?,
            world_border_hit: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.direction.to_enum_id())?;
        write_f32(writer, self.click_x)?;
        write_f32(writer, self.click_y)?;
        write_f32(writer, self.click_z)?;
        write_bool(writer, self.inside)?;
        write_bool(writer, self.world_border_hit)
    }
}

impl ServerboundUseItemOnPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            hand: ServerboundSwingHand::from_id(read_var_i32(reader)?)?,
            block_hit: BlockHitResultPacketData::read(reader)?,
            sequence: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.hand.to_id())?;
        self.block_hit.write(writer)?;
        write_var_i32(writer, self.sequence)
    }
}

impl ServerboundPongPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            id: i32::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ServerboundConfigurationAcknowledgedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        expect_empty_payload(reader)?;
        Ok(Self)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ServerboundJigsawGeneratePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            levels: read_var_i32(reader)?,
            keep_jigsaws: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.levels)?;
        write_bool(writer, self.keep_jigsaws)
    }
}

impl ServerboundSignUpdatePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let is_front_text = read_bool(reader)?;
        Ok(Self {
            x,
            y,
            z,
            is_front_text,
            lines: [
                read_string(reader, 384)?,
                read_string(reader, 384)?,
                read_string(reader, 384)?,
                read_string(reader, 384)?,
            ],
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_bool(writer, self.is_front_text)?;
        for line in &self.lines {
            write_string(writer, line, 384)?;
        }
        Ok(())
    }
}

impl ServerboundSetBeaconPacket {
    pub(super) fn read_optional_mob_effect<R: Read>(reader: &mut R) -> io::Result<Option<i32>> {
        if read_bool(reader)? {
            Ok(Some(read_var_i32(reader)?))
        } else {
            Ok(None)
        }
    }

    pub(super) fn write_optional_mob_effect<W: Write>(
        writer: &mut W,
        effect_id: Option<i32>,
    ) -> io::Result<()> {
        write_bool(writer, effect_id.is_some())?;
        if let Some(effect_id) = effect_id {
            write_var_i32(writer, effect_id)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            primary_effect_id: Self::read_optional_mob_effect(reader)?,
            secondary_effect_id: Self::read_optional_mob_effect(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        Self::write_optional_mob_effect(writer, self.primary_effect_id)?;
        Self::write_optional_mob_effect(writer, self.secondary_effect_id)
    }
}

impl CommandBlockMode {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Sequence),
            1 => Ok(Self::Auto),
            2 => Ok(Self::Redstone),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid command block mode {id}"),
            )),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::Sequence => 0,
            Self::Auto => 1,
            Self::Redstone => 2,
        }
    }
}
