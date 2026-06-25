use super::*;

impl DebugSubscriptionUpdate {
    const VALUE_SUBSCRIPTION_RANGE: std::ops::Range<i32> = 1..16;

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_debug_subscription_id(writer, self.subscription_id)?;
        match self.value_payload.as_deref() {
            Some(payload) => {
                write_bool(writer, true)?;
                writer.write_all(payload)
            }
            None => write_bool(writer, false),
        }
    }
}

impl DebugSubscriptionEvent {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_debug_subscription_id(writer, self.subscription_id)?;
        writer.write_all(&self.value_payload)
    }
}

impl ClientboundDebugBlockValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.block_pos.x, self.block_pos.y, self.block_pos.z)?;
        self.update.write(writer)
    }
}

impl ClientboundDebugChunkValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, pack_chunk_pos_as_long(self.chunk_pos))?;
        self.update.write(writer)
    }
}

impl ClientboundDebugEntityValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        self.update.write(writer)
    }
}

impl ClientboundDebugEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.event.write(writer)
    }
}

fn write_debug_subscription_id<W: Write>(writer: &mut W, subscription_id: i32) -> io::Result<()> {
    if !DebugSubscriptionUpdate::VALUE_SUBSCRIPTION_RANGE.contains(&subscription_id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "debug subscription has no value stream codec",
        ));
    }
    write_var_i32(writer, subscription_id)
}
