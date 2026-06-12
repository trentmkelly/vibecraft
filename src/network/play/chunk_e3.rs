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
