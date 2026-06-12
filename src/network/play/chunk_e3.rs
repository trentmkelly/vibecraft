use super::*;

impl ServerboundSeenAdvancementsAction {
    fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::OpenedTab),
            1 => Ok(Self::ClosedScreen),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid seen advancements action {id}"),
            )),
        }
    }

    fn to_id(&self) -> i32 {
        match self {
            Self::OpenedTab => 0,
            Self::ClosedScreen => 1,
        }
    }
}

impl ServerboundSeenAdvancementsPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let action = ServerboundSeenAdvancementsAction::from_id(read_var_i32(reader)?)?;
        let tab = if action == ServerboundSeenAdvancementsAction::OpenedTab {
            Some(read_identifier(reader)?)
        } else {
            None
        };
        let packet = Self { action, tab };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.action.to_id())?;
        if self.action == ServerboundSeenAdvancementsAction::OpenedTab {
            let tab = self.tab.as_ref().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "opened advancement tab packet missing tab identifier",
                )
            })?;
            write_identifier(writer, tab)?;
        }
        Ok(())
    }
}

impl ServerboundSelectBundleItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            slot_id: read_var_i32(reader)?,
            selected_item_index: read_var_i32(reader)?,
        };
        if packet.selected_item_index < 0 && packet.selected_item_index != -1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid selectedItemIndex: {}", packet.selected_item_index),
            ));
        }
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.selected_item_index < 0 && self.selected_item_index != -1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid selectedItemIndex: {}", self.selected_item_index),
            ));
        }
        write_var_i32(writer, self.slot_id)?;
        write_var_i32(writer, self.selected_item_index)
    }
}

impl ServerboundSetGameRulePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entries: read_collection(reader, ServerboundSetGameRuleEntry::read)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.entries, |writer, entry| entry.write(writer))
    }
}

impl ServerboundSetGameRuleEntry {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            game_rule_key: read_identifier(reader)?,
            value: read_string(reader, 32767)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.game_rule_key)?;
        write_string(writer, &self.value, 32767)
    }
}

impl JigsawJointType {
    fn from_name(name: &str) -> Self {
        match name {
            "rollable" => Self::Rollable,
            "aligned" => Self::Aligned,
            _ => Self::Aligned,
        }
    }

    fn serialized_name(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }
}

impl ServerboundSetJigsawBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let packet = Self {
            x,
            y,
            z,
            name: read_identifier(reader)?,
            target: read_identifier(reader)?,
            pool: read_identifier(reader)?,
            final_state: read_string(reader, 32767)?,
            joint: JigsawJointType::from_name(&read_string(reader, 32767)?),
            selection_priority: read_var_i32(reader)?,
            placement_priority: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_identifier(writer, &self.name)?;
        write_identifier(writer, &self.target)?;
        write_identifier(writer, &self.pool)?;
        write_string(writer, &self.final_state, 32767)?;
        write_string(writer, self.joint.serialized_name(), 32767)?;
        write_var_i32(writer, self.selection_priority)?;
        write_var_i32(writer, self.placement_priority)
    }
}

impl TestBlockMode {
    fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Log,
            2 => Self::Fail,
            3 => Self::Accept,
            _ => Self::Start,
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Start => 0,
            Self::Log => 1,
            Self::Fail => 2,
            Self::Accept => 3,
        }
    }
}

impl ServerboundSetTestBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let packet = Self {
            x,
            y,
            z,
            mode: TestBlockMode::from_id(read_var_i32(reader)?),
            message: read_string(reader, 32767)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.mode.to_id())?;
        write_string(writer, &self.message, 32767)
    }
}

impl ServerboundSpectateEntityPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entity_id: read_var_i32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)
    }
}

impl TestInstanceBlockAction {
    fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Query,
            2 => Self::Set,
            3 => Self::Reset,
            4 => Self::Save,
            5 => Self::Export,
            6 => Self::Run,
            _ => Self::Init,
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Init => 0,
            Self::Query => 1,
            Self::Set => 2,
            Self::Reset => 3,
            Self::Save => 4,
            Self::Export => 5,
            Self::Run => 6,
        }
    }
}

impl TestInstanceBlockStatus {
    fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Running,
            2 => Self::Finished,
            _ => Self::Cleared,
        }
    }

    fn to_id(self) -> i32 {
        match self {
            Self::Cleared => 0,
            Self::Running => 1,
            Self::Finished => 2,
        }
    }
}

impl ServerboundTestInstanceBlockActionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let packet = Self {
            x,
            y,
            z,
            action: TestInstanceBlockAction::from_id(read_var_i32(reader)?),
            data: TestInstanceBlockData::read(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.action.to_id())?;
        self.data.write(writer)
    }
}

impl TestInstanceBlockData {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            test: read_optional(reader, read_identifier)?,
            size: [
                read_var_i32(reader)?,
                read_var_i32(reader)?,
                read_var_i32(reader)?,
            ],
            rotation: StructureRotation::from_id(read_var_i32(reader)?.rem_euclid(4))?,
            ignore_entities: read_bool(reader)?,
            status: TestInstanceBlockStatus::from_id(read_var_i32(reader)?),
            error_message: read_optional(reader, read_trusted_component)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_optional(writer, self.test.as_ref(), write_identifier)?;
        for component in self.size {
            write_var_i32(writer, component)?;
        }
        write_var_i32(writer, self.rotation.to_id())?;
        write_bool(writer, self.ignore_entities)?;
        write_var_i32(writer, self.status.to_id())?;
        write_optional(writer, self.error_message.as_ref(), write_trusted_component)
    }
}

impl ServerboundTeleportToEntityPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            uuid: read_uuid(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, self.uuid)
    }
}
