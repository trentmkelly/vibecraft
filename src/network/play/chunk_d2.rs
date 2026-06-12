use super::*;

impl ServerboundSetCommandBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let command = read_string(reader, 32767)?;
        let mode = CommandBlockMode::from_id(read_var_i32(reader)?)?;
        let flags = read_u8(reader)?;
        let packet = Self {
            x,
            y,
            z,
            command,
            mode,
            track_output: flags & 1 != 0,
            conditional: flags & 2 != 0,
            automatic: flags & 4 != 0,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_string(writer, &self.command, 32767)?;
        write_var_i32(writer, self.mode.to_id())?;
        let flags = (if self.track_output { 1 } else { 0 })
            | (if self.conditional { 2 } else { 0 })
            | (if self.automatic { 4 } else { 0 });
        writer.write_all(&[flags])
    }
}

impl ServerboundSetCommandMinecartPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entity_id: read_var_i32(reader)?,
            command: read_string(reader, 32767)?,
            track_output: read_bool(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_string(writer, &self.command, 32767)?;
        write_bool(writer, self.track_output)
    }
}

impl StructureBlockUpdateType {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::UpdateData),
            1 => Ok(Self::SaveArea),
            2 => Ok(Self::LoadArea),
            3 => Ok(Self::ScanArea),
            _ => Err(invalid_data("invalid structure block update type")),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::UpdateData => 0,
            Self::SaveArea => 1,
            Self::LoadArea => 2,
            Self::ScanArea => 3,
        }
    }
}

impl StructureBlockMode {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Save),
            1 => Ok(Self::Load),
            2 => Ok(Self::Corner),
            3 => Ok(Self::Data),
            _ => Err(invalid_data("invalid structure block mode")),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::Save => 0,
            Self::Load => 1,
            Self::Corner => 2,
            Self::Data => 3,
        }
    }
}

impl StructureMirror {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::LeftRight),
            2 => Ok(Self::FrontBack),
            _ => Err(invalid_data("invalid structure mirror")),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::LeftRight => 1,
            Self::FrontBack => 2,
        }
    }
}

impl StructureRotation {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::Clockwise90),
            2 => Ok(Self::Clockwise180),
            3 => Ok(Self::Counterclockwise90),
            _ => Err(invalid_data("invalid structure rotation")),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }
}

impl ServerboundSetStructureBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        let update_type = StructureBlockUpdateType::from_id(read_var_i32(reader)?)?;
        let mode = StructureBlockMode::from_id(read_var_i32(reader)?)?;
        let name = read_string(reader, 32767)?;
        let offset = [
            read_clamped_i8(reader, -48, 48)?,
            read_clamped_i8(reader, -48, 48)?,
            read_clamped_i8(reader, -48, 48)?,
        ];
        let size = [
            read_clamped_i8(reader, 0, 48)? as u8,
            read_clamped_i8(reader, 0, 48)? as u8,
            read_clamped_i8(reader, 0, 48)? as u8,
        ];
        let mirror = StructureMirror::from_id(read_var_i32(reader)?)?;
        let rotation = StructureRotation::from_id(read_var_i32(reader)?)?;
        let data = read_string(reader, 128)?;
        let integrity = read_f32(reader)?.clamp(0.0, 1.0);
        let seed = read_var_i64(reader)?;
        let flags = read_u8(reader)?;
        let packet = Self {
            x,
            y,
            z,
            update_type,
            mode,
            name,
            offset,
            size,
            mirror,
            rotation,
            data,
            integrity,
            seed,
            ignore_entities: flags & 1 != 0,
            strict: flags & 8 != 0,
            show_air: flags & 2 != 0,
            show_bounding_box: flags & 4 != 0,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.update_type.to_id())?;
        write_var_i32(writer, self.mode.to_id())?;
        write_string(writer, &self.name, 32767)?;
        for value in self.offset {
            writer.write_all(&[value as u8])?;
        }
        for value in self.size {
            writer.write_all(&[value])?;
        }
        write_var_i32(writer, self.mirror.to_id())?;
        write_var_i32(writer, self.rotation.to_id())?;
        write_string(writer, &self.data, 128)?;
        write_f32(writer, self.integrity)?;
        write_var_i64(writer, self.seed)?;
        let flags = (if self.ignore_entities { 1 } else { 0 })
            | (if self.show_air { 2 } else { 0 })
            | (if self.show_bounding_box { 4 } else { 0 })
            | (if self.strict { 8 } else { 0 });
        writer.write_all(&[flags])
    }
}

impl ServerboundSelectTradePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            item: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.item)
    }
}

impl ServerboundRenameItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, 32767)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 32767)
    }
}

impl ServerboundContainerClosePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            container_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)
    }
}

impl ServerboundContainerButtonClickPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            container_id: read_var_i32(reader)?,
            button_id: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.button_id)
    }
}

impl ServerboundContainerSlotStateChangedPacket {
    /// Java: `(input.readVarInt(), input.readContainerId(), input.readBoolean())`
    /// where `readContainerId` is a `VarInt`.
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot_id: read_var_i32(reader)?,
            container_id: read_var_i32(reader)?,
            new_state: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot_id)?;
        write_var_i32(writer, self.container_id)?;
        write_bool(writer, self.new_state)
    }
}

impl ClientboundMerchantOffersPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.offers.len() as i32)?;
        for offer in &self.offers {
            offer.write(writer)?;
        }
        write_var_i32(writer, self.villager_level)?;
        write_var_i32(writer, self.villager_xp)?;
        write_bool(writer, self.show_progress)?;
        write_bool(writer, self.can_restock)
    }
}

impl MerchantOfferData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.base_cost_a.write(writer)?;
        self.result.write_required_trusted(writer)?;
        write_optional(writer, self.cost_b.as_ref(), |writer, cost| {
            cost.write(writer)
        })?;
        write_bool(writer, self.out_of_stock)?;
        write_i32(writer, self.uses)?;
        write_i32(writer, self.max_uses)?;
        write_i32(writer, self.xp)?;
        write_i32(writer, self.special_price_diff)?;
        write_f32(writer, self.price_multiplier)?;
        write_i32(writer, self.demand)
    }
}

impl ItemCostData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.item_id)?;
        write_var_i32(writer, self.count)?;
        self.components.write(writer)
    }
}

impl RawDataComponentPatch {
    pub fn empty() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
        }
    }

    pub fn read_delimited<R: Read>(reader: &mut R) -> io::Result<Self> {
        let added_count = read_limited_len(reader, 65536, "data component add count")?;
        let removed_count = read_limited_len(reader, 65536, "data component remove count")?;
        let mut added = Vec::with_capacity(added_count);
        for _ in 0..added_count {
            let component_type_id = read_var_i32(reader)?;
            let payload = read_length_prefixed_bytes(reader, i32::MAX as usize)?;
            added.push((component_type_id, payload));
        }
        let mut removed = Vec::with_capacity(removed_count);
        for _ in 0..removed_count {
            removed.push(read_var_i32(reader)?);
        }
        Ok(Self { added, removed })
    }

    pub fn write_delimited<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.added.len() as i32)?;
        write_var_i32(writer, self.removed.len() as i32)?;
        for (component_type_id, payload) in &self.added {
            write_var_i32(writer, *component_type_id)?;
            write_length_prefixed_bytes(writer, payload, i32::MAX as usize)?;
        }
        for component_type_id in &self.removed {
            write_var_i32(writer, *component_type_id)?;
        }
        Ok(())
    }

    pub fn write_trusted<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.added.len() as i32)?;
        write_var_i32(writer, self.removed.len() as i32)?;
        for (component_type_id, payload) in &self.added {
            write_var_i32(writer, *component_type_id)?;
            writer.write_all(payload)?;
        }
        for component_type_id in &self.removed {
            write_var_i32(writer, *component_type_id)?;
        }
        Ok(())
    }
}

impl RawDataComponentExactPredicate {
    pub fn empty() -> Self {
        Self {
            expected_components: Vec::new(),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.expected_components.len() as i32)?;
        for (component_type_id, payload) in &self.expected_components {
            write_var_i32(writer, *component_type_id)?;
            writer.write_all(payload)?;
        }
        Ok(())
    }
}

impl RawItemStack {
    pub fn empty() -> Self {
        Self {
            count: 0,
            item_id: None,
            components: RawDataComponentPatch::empty(),
        }
    }

    pub fn read_optional_untrusted<R: Read>(reader: &mut R) -> io::Result<Self> {
        let count = read_var_i32(reader)?;
        if count <= 0 {
            return Ok(Self::empty());
        }
        Ok(Self {
            count,
            item_id: Some(read_var_i32(reader)?),
            components: RawDataComponentPatch::read_delimited(reader)?,
        })
    }

    pub fn write_optional_untrusted<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.count <= 0 {
            return write_var_i32(writer, 0);
        }
        let item_id = self.item_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "non-empty item stack missing item id",
            )
        })?;
        write_var_i32(writer, self.count)?;
        write_var_i32(writer, item_id)?;
        self.components.write_delimited(writer)
    }

    pub fn write_optional_trusted<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.count <= 0 {
            return write_var_i32(writer, 0);
        }
        let item_id = self.item_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "non-empty item stack missing item id",
            )
        })?;
        write_var_i32(writer, self.count)?;
        write_var_i32(writer, item_id)?;
        self.components.write_trusted(writer)
    }

    pub fn write_required_trusted<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.count <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "required item stack cannot be empty",
            ));
        }
        let item_id = self.item_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "non-empty item stack missing item id",
            )
        })?;
        write_var_i32(writer, self.count)?;
        write_var_i32(writer, item_id)?;
        self.components.write_trusted(writer)
    }

    pub fn write_template<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.count <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "item stack template cannot be empty",
            ));
        }
        let item_id = self.item_id.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "non-empty item stack template missing item id",
            )
        })?;
        // Java: ItemStackTemplate.STREAM_CODEC = Item.STREAM_CODEC, count, DataComponentPatch.
        // This differs from inventory ItemStack codecs, which write count before item ID.
        write_var_i32(writer, item_id)?;
        write_var_i32(writer, self.count)?;
        self.components.write_trusted(writer)
    }
}

impl HashedPatchMap {
    pub const MAX_HASHED_COMPONENTS: usize = 256;

    pub fn empty() -> Self {
        Self {
            added_component_hashes: Vec::new(),
            removed_components: Vec::new(),
        }
    }

    pub fn create(
        patch: &RawDataComponentPatch,
        mut hasher: impl FnMut(i32, &[u8]) -> i32,
    ) -> Self {
        let mut added = BTreeMap::new();
        for (component_type_id, payload) in &patch.added {
            added.insert(*component_type_id, hasher(*component_type_id, payload));
        }
        Self {
            added_component_hashes: added.into_iter().collect(),
            removed_components: sorted_unique_i32(&patch.removed),
        }
    }

    pub fn matches(
        &self,
        patch: &RawDataComponentPatch,
        mut hasher: impl FnMut(i32, &[u8]) -> i32,
    ) -> bool {
        if sorted_unique_i32(&self.removed_components) != sorted_unique_i32(&patch.removed) {
            return false;
        }
        if self.added_component_hashes.len() != patch.added.len() {
            return false;
        }

        let expected: BTreeMap<_, _> = self.added_component_hashes.iter().copied().collect();
        for (component_type_id, payload) in &patch.added {
            let Some(expected_hash) = expected.get(component_type_id) else {
                return false;
            };
            if *expected_hash != hasher(*component_type_id, payload) {
                return false;
            }
        }
        true
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let added_len = read_limited_len(
            reader,
            Self::MAX_HASHED_COMPONENTS,
            "hashed patch add count",
        )?;
        let mut added_component_hashes = Vec::with_capacity(added_len);
        for _ in 0..added_len {
            added_component_hashes.push((read_var_i32(reader)?, read_i32(reader)?));
        }
        let removed_len = read_limited_len(
            reader,
            Self::MAX_HASHED_COMPONENTS,
            "hashed patch remove count",
        )?;
        let mut removed_components = Vec::with_capacity(removed_len);
        for _ in 0..removed_len {
            removed_components.push(read_var_i32(reader)?);
        }
        Ok(Self {
            added_component_hashes,
            removed_components,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.added_component_hashes.len() > Self::MAX_HASHED_COMPONENTS
            || self.removed_components.len() > Self::MAX_HASHED_COMPONENTS
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many hashed patch components",
            ));
        }
        write_var_i32(writer, self.added_component_hashes.len() as i32)?;
        for (component_type_id, hash) in &self.added_component_hashes {
            write_var_i32(writer, *component_type_id)?;
            write_i32(writer, *hash)?;
        }
        write_var_i32(writer, self.removed_components.len() as i32)?;
        for component_type_id in &self.removed_components {
            write_var_i32(writer, *component_type_id)?;
        }
        Ok(())
    }
}

impl HashedStack {
    pub fn empty() -> Self {
        Self {
            item_id: None,
            count: 0,
            components: HashedPatchMap::empty(),
        }
    }

    pub fn create_from_raw(
        item_stack: &RawItemStack,
        hasher: impl FnMut(i32, &[u8]) -> i32,
    ) -> Self {
        if item_stack.count <= 0 || item_stack.item_id.is_none() {
            Self::empty()
        } else {
            Self {
                item_id: item_stack.item_id,
                count: item_stack.count,
                components: HashedPatchMap::create(&item_stack.components, hasher),
            }
        }
    }

    pub fn matches_raw(
        &self,
        item_stack: &RawItemStack,
        hasher: impl FnMut(i32, &[u8]) -> i32,
    ) -> bool {
        if self.item_id.is_none() {
            return item_stack.count <= 0 || item_stack.item_id.is_none();
        }
        self.count == item_stack.count
            && self.item_id == item_stack.item_id
            && self.components.matches(&item_stack.components, hasher)
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        if !read_bool(reader)? {
            return Ok(Self::empty());
        }
        Ok(Self {
            item_id: Some(read_var_i32(reader)?),
            count: read_var_i32(reader)?,
            components: HashedPatchMap::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.item_id {
            Some(item_id) => {
                write_bool(writer, true)?;
                write_var_i32(writer, item_id)?;
                write_var_i32(writer, self.count)?;
                self.components.write(writer)
            }
            None => write_bool(writer, false),
        }
    }
}

fn sorted_unique_i32(values: &[i32]) -> Vec<i32> {
    values
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl ContainerInput {
    pub(super) fn from_wire_id(id: i32) -> Self {
        match id {
            1 => Self::QuickMove,
            2 => Self::Swap,
            3 => Self::Clone,
            4 => Self::Throw,
            5 => Self::QuickCraft,
            6 => Self::PickupAll,
            _ => Self::Pickup,
        }
    }

    pub(super) fn to_wire_id(self) -> i32 {
        match self {
            Self::Pickup => 0,
            Self::QuickMove => 1,
            Self::Swap => 2,
            Self::Clone => 3,
            Self::Throw => 4,
            Self::QuickCraft => 5,
            Self::PickupAll => 6,
        }
    }
}

impl ServerboundContainerClickPacket {
    pub const MAX_CHANGED_SLOTS: usize = 128;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let container_id = read_var_i32(reader)?;
        let state_id = read_var_i32(reader)?;
        let slot_num = read_i16(reader)?;
        let button_num = read_i8(reader)?;
        let container_input = ContainerInput::from_wire_id(read_var_i32(reader)?);
        let changed_len = read_limited_len(reader, Self::MAX_CHANGED_SLOTS, "changed slot count")?;
        let mut changed_slots = BTreeMap::new();
        for _ in 0..changed_len {
            changed_slots.insert(read_i16(reader)? as i32, HashedStack::read(reader)?);
        }
        Ok(Self {
            container_id,
            state_id,
            slot_num,
            button_num,
            container_input,
            changed_slots,
            carried_item: HashedStack::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.changed_slots.len() > Self::MAX_CHANGED_SLOTS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many changed slots",
            ));
        }
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.state_id)?;
        write_i16(writer, self.slot_num)?;
        write_i8(writer, self.button_num)?;
        write_var_i32(writer, self.container_input.to_wire_id())?;
        write_var_i32(writer, self.changed_slots.len() as i32)?;
        for (slot, stack) in &self.changed_slots {
            write_i16(writer, *slot as i16)?;
            stack.write(writer)?;
        }
        self.carried_item.write(writer)
    }
}

impl ServerboundSetCreativeModeSlotPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot_num: read_i16(reader)?,
            item_stack: RawItemStack::read_optional_untrusted(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i16(writer, self.slot_num)?;
        self.item_stack.write_optional_untrusted(writer)
    }
}

impl ServerboundPlayerAbilitiesPacket {
    const FLAG_FLYING: u8 = 0x02;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            is_flying: read_u8(reader)? & Self::FLAG_FLYING != 0,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[if self.is_flying { Self::FLAG_FLYING } else { 0 }])
    }
}

impl ServerboundCommandSuggestionPacket {
    pub const MAX_COMMAND_CHARS: usize = 32500;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_var_i32(reader)?,
            command: read_string(reader, Self::MAX_COMMAND_CHARS)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_string(writer, &self.command, Self::MAX_COMMAND_CHARS)
    }
}

impl ServerboundEditBookPacket {
    pub const MAX_PAGES: usize = 100;
    pub const MAX_PAGE_CHARS: usize = 1024;
    pub const MAX_TITLE_CHARS: usize = 32;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let slot = read_var_i32(reader)?;
        let page_count = read_var_i32(reader)?;
        if page_count < 0 || page_count as usize > Self::MAX_PAGES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid edit book page count",
            ));
        }
        let mut pages = Vec::with_capacity(page_count as usize);
        for _ in 0..page_count {
            pages.push(read_string(reader, Self::MAX_PAGE_CHARS)?);
        }
        let title = if read_bool(reader)? {
            Some(read_string(reader, Self::MAX_TITLE_CHARS)?)
        } else {
            None
        };
        Ok(Self { slot, pages, title })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.pages.len() > Self::MAX_PAGES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many edit book pages",
            ));
        }
        write_var_i32(writer, self.slot)?;
        write_var_i32(writer, self.pages.len() as i32)?;
        for page in &self.pages {
            write_string(writer, page, Self::MAX_PAGE_CHARS)?;
        }
        write_bool(writer, self.title.is_some())?;
        if let Some(title) = &self.title {
            write_string(writer, title, Self::MAX_TITLE_CHARS)?;
        }
        Ok(())
    }
}

impl ServerboundInteractionHand {
    pub(super) fn from_id(id: i32) -> Self {
        match id {
            1 => Self::OffHand,
            _ => Self::MainHand,
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::MainHand => 0,
            Self::OffHand => 1,
        }
    }
}

impl ServerboundAttackPacket {
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

impl ServerboundInteractPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            entity_id: read_var_i32(reader)?,
            hand: ServerboundInteractionHand::from_id(read_var_i32(reader)?),
            location: read_lp_vec3(reader)?,
            using_secondary_action: read_bool(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.hand.to_id())?;
        write_lp_vec3(writer, self.location)?;
        write_bool(writer, self.using_secondary_action)
    }
}

pub(super) fn read_lp_vec3<R: Read>(reader: &mut R) -> io::Result<Vec3> {
    let lowest = read_u8(reader)?;
    if lowest == 0 {
        return Ok(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
    }

    let middle = read_u8(reader)?;
    let mut highest_bytes = [0u8; 4];
    reader.read_exact(&mut highest_bytes)?;
    let highest = u32::from_be_bytes(highest_bytes) as u64;
    let buffer = (highest << 16) | ((middle as u64) << 8) | lowest as u64;
    let mut scale = (lowest & 3) as u64;
    if lowest & 4 == 4 {
        scale |= (read_var_i32(reader)? as u32 as u64) << 2;
    }
    let scale = scale as f64;

    Ok(Vec3 {
        x: unpack_lp_vec3_component(buffer >> 3) * scale,
        y: unpack_lp_vec3_component(buffer >> 18) * scale,
        z: unpack_lp_vec3_component(buffer >> 33) * scale,
    })
}

pub(super) fn write_lp_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    const ABS_MAX_VALUE: f64 = 1.7179869183E10;
    const ABS_MIN_VALUE: f64 = 3.051944088384301E-5;

    let x = sanitize_lp_vec3_component(value.x, ABS_MAX_VALUE);
    let y = sanitize_lp_vec3_component(value.y, ABS_MAX_VALUE);
    let z = sanitize_lp_vec3_component(value.z, ABS_MAX_VALUE);
    let chessboard_length = x.abs().max(y.abs()).max(z.abs());
    if chessboard_length < ABS_MIN_VALUE {
        return writer.write_all(&[0]);
    }

    let scale = chessboard_length.ceil() as u64;
    let is_partial = (scale & 3) != scale;
    let markers = if is_partial { (scale & 3) | 4 } else { scale };
    let buffer = markers
        | (pack_lp_vec3_component(x / scale as f64) << 3)
        | (pack_lp_vec3_component(y / scale as f64) << 18)
        | (pack_lp_vec3_component(z / scale as f64) << 33);
    writer.write_all(&[(buffer & 0xff) as u8, ((buffer >> 8) & 0xff) as u8])?;
    writer.write_all(&((buffer >> 16) as u32).to_be_bytes())?;
    if is_partial {
        write_var_i32(writer, (scale >> 2) as i32)?;
    }
    Ok(())
}

pub(super) fn sanitize_lp_vec3_component(value: f64, abs_max: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-abs_max, abs_max)
    }
}

pub(super) fn pack_lp_vec3_component(value: f64) -> u64 {
    ((value * 0.5 + 0.5) * 32766.0).round() as u64
}

pub(super) fn unpack_lp_vec3_component(value: u64) -> f64 {
    (value & 32767).min(32766) as f64 * 2.0 / 32766.0 - 1.0
}
