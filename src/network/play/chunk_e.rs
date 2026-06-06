use super::*;

impl ServerboundPickItemFromBlockPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            x,
            y,
            z,
            include_data: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_bool(writer, self.include_data)
    }
}

impl ServerboundPickItemFromEntityPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            entity_id: read_var_i32(reader)?,
            include_data: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_bool(writer, self.include_data)
    }
}

impl RecipeBookType {
    pub(super) fn from_id(id: i32) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Crafting),
            1 => Ok(Self::Furnace),
            2 => Ok(Self::BlastFurnace),
            3 => Ok(Self::Smoker),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid recipe book type {id}"),
            )),
        }
    }

    pub(super) fn to_id(self) -> i32 {
        match self {
            Self::Crafting => 0,
            Self::Furnace => 1,
            Self::BlastFurnace => 2,
            Self::Smoker => 3,
        }
    }
}

impl ServerboundRecipeBookChangeSettingsPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            book_type: RecipeBookType::from_id(read_var_i32(reader)?)?,
            is_open: read_bool(reader)?,
            is_filtering: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.book_type.to_id())?;
        write_bool(writer, self.is_open)?;
        write_bool(writer, self.is_filtering)
    }
}

impl ServerboundRecipeBookSeenRecipePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            recipe_index: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.recipe_index)
    }
}

impl ServerboundPlaceRecipePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            container_id: read_var_i32(reader)?,
            recipe_index: read_var_i32(reader)?,
            use_max_items: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.recipe_index)?;
        write_bool(writer, self.use_max_items)
    }
}

impl ServerboundChunkBatchReceivedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            desired_chunks_per_tick: read_f32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_f32(writer, self.desired_chunks_per_tick)
    }
}

impl ClientboundChunkBatchFinishedPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            batch_size: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.batch_size)
    }
}

impl ClientboundChangeDifficultyPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            difficulty: GameDifficulty::from_wire_index(read_var_i32(reader)?)?,
            locked: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.difficulty.to_wire_index())?;
        write_bool(writer, self.locked)
    }
}

impl ClientboundSetChunkCacheCenterPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            x: read_var_i32(reader)?,
            z: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.x)?;
        write_var_i32(writer, self.z)
    }
}

impl ClientboundSetChunkCacheRadiusPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            radius: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.radius)
    }
}

impl ClientboundSetDefaultSpawnPositionPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let dimension = read_identifier(reader)?;
        let (x, y, z) = read_block_position(reader)?;
        Ok(Self {
            respawn_data: ClientboundSetDefaultSpawnPositionData {
                dimension,
                x,
                y,
                z,
                yaw: read_f32(reader)?,
                pitch: read_f32(reader)?,
            },
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.respawn_data.dimension)?;
        write_block_position(
            writer,
            self.respawn_data.x,
            self.respawn_data.y,
            self.respawn_data.z,
        )?;
        writer.write_all(&self.respawn_data.yaw.to_be_bytes())?;
        writer.write_all(&self.respawn_data.pitch.to_be_bytes())
    }
}

impl ClientboundSetExperiencePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            experience_progress: read_f32(reader)?,
            experience_level: read_var_i32(reader)?,
            total_experience: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.experience_progress.to_be_bytes())?;
        write_var_i32(writer, self.experience_level)?;
        write_var_i32(writer, self.total_experience)
    }
}

impl ClientboundSetHealthPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            health: read_f32(reader)?,
            food: read_var_i32(reader)?,
            saturation: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.health.to_be_bytes())?;
        write_var_i32(writer, self.food)?;
        writer.write_all(&self.saturation.to_be_bytes())
    }
}

impl ClockNetworkState {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            total_ticks: read_var_i64(reader)?,
            partial_tick: read_f32(reader)?,
            rate: read_f32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i64(writer, self.total_ticks)?;
        writer.write_all(&self.partial_tick.to_be_bytes())?;
        writer.write_all(&self.rate.to_be_bytes())
    }
}

impl ClientboundSetTimePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        // Java: ByteBufCodecs.LONG — fixed 8-byte big-endian long, not a varint
        let game_time = read_i64(reader)?;
        let clock_updates_len = read_var_i32(reader)?;
        if clock_updates_len < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid clock update map length",
            ));
        }

        let mut clock_updates = BTreeMap::new();
        for _ in 0..clock_updates_len {
            // Java: WorldClock.STREAM_CODEC = ByteBufCodecs.holderRegistry(WORLD_CLOCK) — VarInt ID
            let key = read_var_i32(reader)?;
            let state = ClockNetworkState::read(reader)?;
            clock_updates.insert(key, state);
        }

        Ok(Self {
            game_time,
            clock_updates,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, self.game_time)?;
        write_var_i32(writer, self.clock_updates.len() as i32)?;
        for (clock_id, state) in &self.clock_updates {
            write_var_i32(writer, *clock_id)?;
            state.write(writer)?;
        }
        Ok(())
    }
}

impl ClientboundGameEventType {
    pub(super) fn from_id(id: u8) -> Self {
        match id {
            0 => Self::NoRespawnBlockAvailable,
            1 => Self::StartRaining,
            2 => Self::StopRaining,
            3 => Self::ChangeGameMode,
            4 => Self::WinGame,
            5 => Self::DemoEvent,
            6 => Self::PlayArrowHitSound,
            7 => Self::RainLevelChange,
            8 => Self::ThunderLevelChange,
            9 => Self::PufferFishSting,
            10 => Self::GuardianElderEffect,
            11 => Self::ImmediateRespawn,
            12 => Self::LimitedCrafting,
            13 => Self::LevelChunksLoadStart,
            _ => Self::Unknown(id),
        }
    }

    pub(super) fn to_id(self) -> u8 {
        match self {
            Self::NoRespawnBlockAvailable => 0,
            Self::StartRaining => 1,
            Self::StopRaining => 2,
            Self::ChangeGameMode => 3,
            Self::WinGame => 4,
            Self::DemoEvent => 5,
            Self::PlayArrowHitSound => 6,
            Self::RainLevelChange => 7,
            Self::ThunderLevelChange => 8,
            Self::PufferFishSting => 9,
            Self::GuardianElderEffect => 10,
            Self::ImmediateRespawn => 11,
            Self::LimitedCrafting => 12,
            Self::LevelChunksLoadStart => 13,
            Self::Unknown(value) => value,
        }
    }
}

impl ClientboundGameEventPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut value = [0u8; 1];
        reader.read_exact(&mut value)?;
        let packet = Self {
            event: ClientboundGameEventType::from_id(value[0]),
            param: read_f32(reader)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.event.to_id()])?;
        writer.write_all(&self.param.to_be_bytes())
    }
}

impl ClientboundSetSimulationDistancePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            simulation_distance: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.simulation_distance)
    }
}

impl ClientboundTickingStatePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            tick_rate: read_f32(reader)?,
            is_frozen: read_bool(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.tick_rate.to_be_bytes())?;
        write_bool(writer, self.is_frozen)
    }
}

impl ClientboundTickingStepPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            tick_steps: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.tick_steps)
    }
}

impl RespawnDataToKeep {
    pub const NONE: Self = Self { bits: 0 };
    pub const KEEP_ATTRIBUTE_MODIFIERS: Self = Self { bits: 1 };
    pub const KEEP_ENTITY_DATA: Self = Self { bits: 2 };
    pub const KEEP_ALL_DATA: Self = Self { bits: 3 };

    pub fn should_keep(self, mask: Self) -> bool {
        self.bits & mask.bits != 0
    }

    pub fn bits(self) -> u8 {
        self.bits
    }
}

impl ClientboundRespawnPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.spawn_info.write(writer)?;
        writer.write_all(&[self.data_to_keep.bits])
    }
}

impl ServerboundSetCarriedItemPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut bytes = [0u8; 2];
        reader.read_exact(&mut bytes)?;
        Ok(Self {
            slot: i16::from_be_bytes(bytes),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.slot.to_be_bytes())
    }
}

impl ClientboundSetHeldSlotPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            slot: read_var_i32(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)
    }
}

impl ClientboundBlockDestructionPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.id)?;
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.progress])
    }
}

impl ClientboundBlockEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        writer.write_all(&[self.action, self.param])?;
        write_var_i32(writer, self.block_id)
    }
}

impl ClientboundBlockUpdatePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.block_state_id)
    }
}

impl ClientboundBlockEntityDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.x, self.y, self.z)?;
        write_var_i32(writer, self.block_entity_type_id)?;
        write_network_compound_tag(writer, &self.tag)
    }
}

impl ClientboundLevelEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i32(writer, self.event_type)?;
        write_block_position(writer, self.x, self.y, self.z)?;
        write_i32(writer, self.data)?;
        write_bool(writer, self.global_event)
    }
}

impl ClientboundPlayerInfoRemovePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.profile_ids.len() as i32)?;
        for id in &self.profile_ids {
            write_uuid(writer, *id)?;
        }
        Ok(())
    }
}

impl ClientboundContainerClosePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)
    }
}

impl ClientboundContainerSetDataPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_i16(writer, self.id)?;
        write_i16(writer, self.value)
    }
}

impl ClientboundContainerPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.state_id)?;
        write_var_i32(writer, self.slots.len() as i32)?;
        for slot in &self.slots {
            slot.write_optional_trusted(writer)?;
        }
        self.carried_item.write_optional_trusted(writer)
    }
}

impl ClientboundContainerSetSlotPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.container_id)?;
        write_var_i32(writer, self.state_id)?;
        write_i16(writer, self.slot)?;
        self.item_stack.write_optional_trusted(writer)
    }
}

pub fn raw_item_stack_from_item_stack(stack: &ItemStack) -> io::Result<RawItemStack> {
    if stack.is_empty() {
        return Ok(RawItemStack::empty());
    }
    let item_id = item_protocol_id(stack.item_id()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown item protocol id for {}", stack.item_id()),
        )
    })?;
    Ok(RawItemStack {
        count: stack.count(),
        item_id: Some(item_id),
        components: raw_data_component_patch_from_item_stack(stack)?,
    })
}

fn raw_data_component_patch_from_item_stack(
    stack: &ItemStack,
) -> io::Result<RawDataComponentPatch> {
    let mut added = Vec::new();
    if let Some(ItemComponent::WritableBookContent(pages)) =
        stack.component("minecraft:writable_book_content")
    {
        if !pages.is_empty() {
            let mut payload = Vec::new();
            write_var_i32(&mut payload, pages.len() as i32)?;
            for page in pages {
                write_string(
                    &mut payload,
                    page,
                    ServerboundEditBookPacket::MAX_PAGE_CHARS,
                )?;
                write_bool(&mut payload, false)?;
            }
            added.push((54, payload));
        }
    }
    if let Some(ItemComponent::WrittenBookContent {
        title,
        author,
        generation,
        pages,
        resolved,
    }) = stack.component("minecraft:written_book_content")
    {
        let mut payload = Vec::new();
        write_string(
            &mut payload,
            title,
            ServerboundEditBookPacket::MAX_TITLE_CHARS,
        )?;
        write_bool(&mut payload, false)?;
        write_string(&mut payload, author, 32767)?;
        write_var_i32(&mut payload, *generation)?;
        write_var_i32(&mut payload, pages.len() as i32)?;
        for page in pages {
            let text = serde_json::to_string(page)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
            write_trusted_component(&mut payload, &ComponentJson(format!("{{\"text\":{text}}}")))?;
            write_bool(&mut payload, false)?;
        }
        write_bool(&mut payload, *resolved)?;
        added.push((55, payload));
    }
    Ok(RawDataComponentPatch {
        added,
        removed: Vec::new(),
    })
}

pub fn slot_corrections_to_set_slot_packets(
    container_id: i32,
    state_id: i32,
    corrections: &[SlotCorrection],
) -> io::Result<Vec<ClientboundContainerSetSlotPacket>> {
    corrections
        .iter()
        .map(|correction| {
            Ok(ClientboundContainerSetSlotPacket {
                container_id,
                state_id,
                slot: correction.slot as i16,
                item_stack: raw_item_stack_from_item_stack(&correction.actual)?,
            })
        })
        .collect()
}

/// Convert the network-layer `ContainerInput` to the inventory-layer `ContainerInput`.
pub(super) fn play_container_input_to_inventory(
    input: ContainerInput,
) -> crate::inventory::ContainerInput {
    match input {
        ContainerInput::Pickup => crate::inventory::ContainerInput::Pickup,
        ContainerInput::QuickMove => crate::inventory::ContainerInput::QuickMove,
        ContainerInput::Swap => crate::inventory::ContainerInput::Swap,
        ContainerInput::Clone => crate::inventory::ContainerInput::Clone,
        ContainerInput::Throw => crate::inventory::ContainerInput::Throw,
        ContainerInput::QuickCraft => crate::inventory::ContainerInput::QuickCraft,
        ContainerInput::PickupAll => crate::inventory::ContainerInput::PickupAll,
    }
}

/// Flatten an `InventoryMenu` (+ separate cursor) into a generic `Menu` snapshot for use with
/// `apply_scripted_packet`.  Slot 0 (result) has `may_place = false`.
pub fn inventory_menu_to_flat_menu(inventory_menu: &InventoryMenu, carried: &ItemStack) -> Menu {
    let mut menu = Menu::new(InventoryMenu::SLOT_COUNT);
    for i in 0..InventoryMenu::SLOT_COUNT {
        let stack = inventory_menu
            .get_slot(i)
            .unwrap_or_else(crate::item_stack::ItemStack::empty);
        menu.slots[i] = Slot {
            stack,
            max_stack_size: 64,
            may_place: inventory_menu.may_place(i),
            may_pickup: true,
        };
    }
    menu.carried = carried.clone();
    menu
}

/// Build `ContainerSetSlot` correction instructions for every slot that differs between the
/// server's `InventoryMenu` state and the client's expected view.  Used when a stale state-ID
/// packet is rejected.
pub fn slot_corrections_from_inventory_menu(
    inventory_menu: &InventoryMenu,
    carried: &ItemStack,
    state_id: i32,
) -> Vec<PlayInstruction> {
    let mut instructions = Vec::new();
    for (i, stack) in inventory_menu.all_slots().iter().enumerate() {
        if let Ok(raw) = raw_item_stack_from_item_stack(stack) {
            instructions.push(PlayInstruction::ContainerSetSlot(
                ClientboundContainerSetSlotPacket {
                    container_id: 0,
                    state_id,
                    slot: i as i16,
                    item_stack: raw,
                },
            ));
        }
    }
    if let Ok(raw) = raw_item_stack_from_item_stack(carried) {
        instructions.push(PlayInstruction::SetCursorItem(
            ClientboundSetCursorItemPacket { item_stack: raw },
        ));
    }
    instructions
}

/// Process one container click packet against the player's `InventoryMenu`.
///
/// This is the core click-processing logic extracted from
/// `PlaySession::process_pending_container_click` so the game-loop in
/// `network/status.rs` can call it directly without holding a `PlaySession`.
///
/// Matches Java: `ServerGamePacketListenerImpl.handleContainerClick` +
///               `AbstractContainerMenu.clicked`.
pub fn handle_container_click(
    packet: &ServerboundContainerClickPacket,
    container_state_id: &mut i32,
    inventory_menu: &mut InventoryMenu,
    carried: &mut ItemStack,
) -> Vec<PlayInstruction> {
    use crate::inventory::InventoryAction;

    if !is_valid_inventory_menu_click_slot(packet.slot_num) {
        return Vec::new();
    }
    if packet.slot_num == -1 {
        if packet.state_id != *container_state_id {
            return slot_corrections_from_inventory_menu(
                inventory_menu,
                carried,
                *container_state_id,
            );
        }
        return Vec::new();
    }

    let full_resync_needed = packet.state_id != *container_state_id;
    let before_slots = inventory_menu.all_slots();
    let before_carried = carried.clone();
    let slot_idx = usize::try_from(packet.slot_num).ok();

    if packet.container_input == ContainerInput::QuickMove {
        if let Some(slot) = slot_idx {
            inventory_menu.quick_move(slot);
        }
        *container_state_id = (*container_state_id).wrapping_add(1);
    } else {
        let mut snapshot = inventory_menu_to_flat_menu(inventory_menu, carried);
        let dry_run = apply_scripted_packet(
            &mut snapshot.clone(),
            packet.state_id,
            &ScriptedContainerClickPacket {
                container_id: 0,
                state_id: packet.state_id,
                slot: packet.slot_num as i32,
                button: packet.button_num as i32,
                mode: play_container_input_to_inventory(packet.container_input),
                changed_slots: Vec::new(),
                carried: ItemStack::empty(),
            },
        );
        let scripted = ScriptedContainerClickPacket {
            container_id: 0,
            state_id: packet.state_id,
            slot: packet.slot_num as i32,
            button: packet.button_num as i32,
            mode: play_container_input_to_inventory(packet.container_input),
            changed_slots: Vec::new(),
            carried: dry_run.carried,
        };
        let result = apply_scripted_packet(&mut snapshot, packet.state_id, &scripted);
        if result.accepted {
            *container_state_id = (*container_state_id).wrapping_add(1);
            let result_taken = slot_idx == Some(0)
                && matches!(result.action, InventoryAction::PickedUp { slot: 0, .. });
            if result_taken {
                let taken = inventory_menu.take_result();
                *carried = taken;
            } else {
                for (i, slot) in snapshot.slots.iter().enumerate().skip(1) {
                    inventory_menu.set_slot(i, slot.stack.clone());
                }
                *carried = snapshot.carried.clone();
            }
        }
    }

    if full_resync_needed {
        let mut instructions =
            slot_corrections_from_inventory_menu(inventory_menu, carried, *container_state_id);
        let unlock_events = inventory_menu.drain_recipe_unlock_events();
        if !unlock_events.is_empty() {
            instructions.push(PlayInstruction::RecipesUnlocked(unlock_events));
        }
        return instructions;
    }

    let mut instructions: Vec<PlayInstruction> = Vec::new();
    let after_slots = inventory_menu.all_slots();
    for (i, (before, after)) in before_slots.iter().zip(after_slots.iter()).enumerate() {
        if before != after {
            if let Ok(raw) = raw_item_stack_from_item_stack(after) {
                instructions.push(PlayInstruction::ContainerSetSlot(
                    ClientboundContainerSetSlotPacket {
                        container_id: 0,
                        state_id: *container_state_id,
                        slot: i as i16,
                        item_stack: raw,
                    },
                ));
            }
        }
    }
    if *carried != before_carried {
        if let Ok(raw) = raw_item_stack_from_item_stack(carried) {
            instructions.push(PlayInstruction::SetCursorItem(
                ClientboundSetCursorItemPacket { item_stack: raw },
            ));
        }
    }
    let unlock_events = inventory_menu.drain_recipe_unlock_events();
    if !unlock_events.is_empty() {
        instructions.push(PlayInstruction::RecipesUnlocked(unlock_events));
    }
    instructions
}

fn is_valid_inventory_menu_click_slot(slot_num: i16) -> bool {
    slot_num == -999 || slot_num == -1 || (0..InventoryMenu::SLOT_COUNT as i16).contains(&slot_num)
}

/// Build a `ClientboundRecipeBookAddPacket` announcing newly-unlocked recipes.
///
/// Called by the server runtime when `PlayInstruction::RecipesUnlocked` is emitted.
/// Uses sequential recipe index as the display ID.
/// Java: `RecipeManager` assigns `RecipeDisplay` IDs during server reload.
pub fn build_recipe_book_add(
    recipe_ids: &[&str],
    recipe_map: &crate::recipe_system::RecipeMap,
) -> Option<ClientboundRecipeBookAddPacket> {
    build_recipe_book_add_with_flags(recipe_ids, recipe_map, true, true, false, None)
}

pub fn build_recipe_book_add_with_flags(
    recipe_ids: &[&str],
    recipe_map: &crate::recipe_system::RecipeMap,
    notification: bool,
    highlight: bool,
    replace: bool,
    highlighted_recipe_ids: Option<&[&str]>,
) -> Option<ClientboundRecipeBookAddPacket> {
    let all_holders = recipe_map.values();
    let entries = recipe_ids
        .iter()
        .filter_map(|recipe_id| {
            recipe_book_add_entry(
                recipe_id,
                recipe_map,
                all_holders,
                notification,
                highlight,
                highlighted_recipe_ids,
            )
        })
        .collect::<Vec<_>>();

    (!entries.is_empty()).then_some(ClientboundRecipeBookAddPacket { entries, replace })
}

struct RecipeBookDisplayBuild {
    display: RecipeDisplayData,
    crafting_requirements: Option<Vec<RecipeIngredientData>>,
    category_id: i32,
}

fn recipe_book_add_entry(
    recipe_id: &str,
    recipe_map: &crate::recipe_system::RecipeMap,
    all_holders: &[crate::recipe_system::RecipeHolder],
    notification: bool,
    highlight: bool,
    highlighted_recipe_ids: Option<&[&str]>,
) -> Option<RecipeBookAddEntry> {
    let holder = recipe_map.by_key(recipe_id)?;
    let display_id = recipe_display_id(holder, all_holders);
    let display = recipe_book_display(holder)?;
    let entry_highlight = highlighted_recipe_ids
        .map(|ids| ids.contains(&recipe_id))
        .unwrap_or(highlight);
    Some(RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: display_id,
            display: display.display,
            group: None,
            category_id: display.category_id,
            crafting_requirements: display.crafting_requirements,
        },
        notification,
        entry_highlight,
    ))
}

fn recipe_display_id(
    holder: &crate::recipe_system::RecipeHolder,
    all_holders: &[crate::recipe_system::RecipeHolder],
) -> i32 {
    all_holders
        .iter()
        .position(|candidate| candidate.id == holder.id)
        .unwrap_or(0) as i32
}

fn recipe_book_display(
    holder: &crate::recipe_system::RecipeHolder,
) -> Option<RecipeBookDisplayBuild> {
    use crate::recipe_system::RecipeKind;

    match &holder.recipe {
        RecipeKind::Shapeless {
            ingredients,
            result,
            ..
        } => Some(recipe_book_shapeless_display(ingredients, result)),
        RecipeKind::Shaped {
            width,
            height,
            pattern,
            result,
            ..
        } => Some(recipe_book_shaped_display(*width, *height, pattern, result)),
        RecipeKind::Cooking {
            kind,
            ingredient,
            result,
            experience_millis,
            cooking_time,
            ..
        } => Some(recipe_book_cooking_display(
            *kind,
            ingredient,
            result,
            *experience_millis,
            *cooking_time,
        )),
        RecipeKind::Stonecutting { ingredient, result } => {
            Some(recipe_book_stonecutting_display(ingredient, result))
        }
        RecipeKind::SmithingTransform {
            template,
            base,
            addition,
            result,
        } => Some(recipe_book_smithing_transform_display(
            template, base, addition, result,
        )),
        RecipeKind::SmithingTrim {
            template,
            base,
            addition,
            ..
        } => Some(recipe_book_smithing_trim_display(template, base, addition)),
        // Special/transmute/imbue recipes — omit from recipe book for now.
        // Java: These use dedicated server-side logic, not generic RecipeDisplay.
        RecipeKind::Special { .. } | RecipeKind::Transmute { .. } | RecipeKind::Imbue { .. } => {
            None
        }
    }
}

fn recipe_book_shapeless_display(
    ingredients: &[crate::recipe_system::IngredientSpec],
    result: &crate::recipe_system::ItemAmount,
) -> RecipeBookDisplayBuild {
    let requirement_slots = ingredients
        .iter()
        .filter_map(ingredient_to_req)
        .collect::<Vec<_>>();
    RecipeBookDisplayBuild {
        display: RecipeDisplayData::CraftingShapeless {
            ingredients: ingredients.iter().map(ingredient_to_slot).collect(),
            result: item_amount_to_slot(result.item, result.count),
            crafting_station: station_slot("minecraft:crafting_table"),
        },
        crafting_requirements: recipe_requirements(requirement_slots),
        category_id: 3,
    }
}

fn recipe_book_shaped_display(
    width: usize,
    height: usize,
    pattern: &[Option<crate::recipe_system::IngredientSpec>],
    result: &crate::recipe_system::ItemAmount,
) -> RecipeBookDisplayBuild {
    let requirement_slots = pattern
        .iter()
        .filter_map(|ingredient| ingredient.as_ref().and_then(ingredient_to_req))
        .collect::<Vec<_>>();
    RecipeBookDisplayBuild {
        display: RecipeDisplayData::CraftingShaped {
            width: width as i32,
            height: height as i32,
            ingredients: pattern
                .iter()
                .map(|ingredient| {
                    ingredient
                        .as_ref()
                        .map_or(SlotDisplayData::Empty, ingredient_to_slot)
                })
                .collect(),
            result: item_amount_to_slot(result.item, result.count),
            crafting_station: station_slot("minecraft:crafting_table"),
        },
        crafting_requirements: recipe_requirements(requirement_slots),
        category_id: 3,
    }
}

fn recipe_book_cooking_display(
    kind: crate::recipe_system::CookingKind,
    ingredient: &crate::recipe_system::IngredientSpec,
    result: &crate::recipe_system::ItemAmount,
    experience_millis: i32,
    cooking_time: Option<i32>,
) -> RecipeBookDisplayBuild {
    let station_name = match kind {
        crate::recipe_system::CookingKind::Smelting => "minecraft:furnace",
        crate::recipe_system::CookingKind::Blasting => "minecraft:blast_furnace",
        crate::recipe_system::CookingKind::Smoking => "minecraft:smoker",
        crate::recipe_system::CookingKind::CampfireCooking => "minecraft:campfire",
    };
    let default_time = match kind {
        crate::recipe_system::CookingKind::Smelting => 200,
        _ => 100,
    };
    RecipeBookDisplayBuild {
        display: RecipeDisplayData::Furnace {
            ingredient: ingredient_to_slot(ingredient),
            fuel: SlotDisplayData::AnyFuel,
            result: item_amount_to_slot(result.item, result.count),
            crafting_station: station_slot(station_name),
            duration: cooking_time.unwrap_or(default_time),
            experience_bits: experience_millis.unsigned_abs(),
        },
        crafting_requirements: None,
        category_id: 3,
    }
}

fn recipe_book_stonecutting_display(
    ingredient: &crate::recipe_system::IngredientSpec,
    result: &crate::recipe_system::ItemAmount,
) -> RecipeBookDisplayBuild {
    RecipeBookDisplayBuild {
        display: RecipeDisplayData::Stonecutter {
            ingredient: ingredient_to_slot(ingredient),
            result: item_amount_to_slot(result.item, result.count),
            crafting_station: station_slot("minecraft:stonecutter"),
        },
        crafting_requirements: ingredient_to_req(ingredient).map(|requirement| vec![requirement]),
        category_id: 3,
    }
}

fn recipe_book_smithing_transform_display(
    template: &crate::recipe_system::IngredientSpec,
    base: &crate::recipe_system::IngredientSpec,
    addition: &crate::recipe_system::IngredientSpec,
    result: &crate::recipe_system::ItemAmount,
) -> RecipeBookDisplayBuild {
    RecipeBookDisplayBuild {
        display: smithing_display(
            template,
            base,
            addition,
            item_amount_to_slot(result.item, result.count),
        ),
        crafting_requirements: None,
        category_id: 3,
    }
}

fn recipe_book_smithing_trim_display(
    template: &crate::recipe_system::IngredientSpec,
    base: &crate::recipe_system::IngredientSpec,
    addition: &crate::recipe_system::IngredientSpec,
) -> RecipeBookDisplayBuild {
    RecipeBookDisplayBuild {
        display: smithing_display(template, base, addition, SlotDisplayData::Empty),
        crafting_requirements: None,
        category_id: 3,
    }
}

fn smithing_display(
    template: &crate::recipe_system::IngredientSpec,
    base: &crate::recipe_system::IngredientSpec,
    addition: &crate::recipe_system::IngredientSpec,
    result: SlotDisplayData,
) -> RecipeDisplayData {
    RecipeDisplayData::Smithing {
        template: ingredient_to_slot(template),
        base: ingredient_to_slot(base),
        addition: ingredient_to_slot(addition),
        result,
        crafting_station: station_slot("minecraft:smithing_table"),
    }
}

fn station_slot(item: &str) -> SlotDisplayData {
    item_protocol_id(item)
        .map(|item_id| SlotDisplayData::Item { item_id })
        .unwrap_or(SlotDisplayData::Empty)
}

fn ingredient_to_slot(spec: &crate::recipe_system::IngredientSpec) -> SlotDisplayData {
    match spec {
        crate::recipe_system::IngredientSpec::Empty => SlotDisplayData::Empty,
        crate::recipe_system::IngredientSpec::Item(name) => station_slot(name),
        crate::recipe_system::IngredientSpec::AnyOf(names) => any_of_ingredient_to_slot(names),
    }
}

fn any_of_ingredient_to_slot(names: &[&'static str]) -> SlotDisplayData {
    let items = names
        .iter()
        .filter_map(|name| item_protocol_id(name).map(|item_id| SlotDisplayData::Item { item_id }))
        .collect::<Vec<_>>();
    match items.as_slice() {
        [] => SlotDisplayData::Empty,
        [item] => item.clone(),
        _ => SlotDisplayData::Composite(items),
    }
}

fn item_amount_to_slot(item: &str, count: u32) -> SlotDisplayData {
    let Some(item_id) = item_protocol_id(item) else {
        return SlotDisplayData::Empty;
    };
    if count == 1 {
        SlotDisplayData::Item { item_id }
    } else {
        SlotDisplayData::ItemStack {
            stack: RawItemStack {
                count: count as i32,
                item_id: Some(item_id),
                components: RawDataComponentPatch::empty(),
            },
        }
    }
}

fn ingredient_to_req(spec: &crate::recipe_system::IngredientSpec) -> Option<RecipeIngredientData> {
    match spec {
        crate::recipe_system::IngredientSpec::Empty => None,
        crate::recipe_system::IngredientSpec::Item(name) => {
            item_protocol_id(name).map(|item_id| RecipeIngredientData::DirectItems(vec![item_id]))
        }
        crate::recipe_system::IngredientSpec::AnyOf(names) => {
            let item_ids = names
                .iter()
                .filter_map(|name| item_protocol_id(name))
                .collect::<Vec<_>>();
            recipe_requirements(item_ids).map(RecipeIngredientData::DirectItems)
        }
    }
}

fn recipe_requirements<T>(requirements: Vec<T>) -> Option<Vec<T>> {
    (!requirements.is_empty()).then_some(requirements)
}
