use super::block_menu_open::LiveBlockMenuKind;
use super::*;
use crate::inventory::{Menu, Slot};
use crate::inventory_transactions::{apply_scripted_packet, ScriptedContainerClickPacket};
use crate::network::play::{
    ClientboundContainerPacket, ClientboundSetCursorItemPacket, ContainerInput,
    CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
};

const PLAYER_MAIN_STORAGE: usize = 27;
const PLAYER_SLOTS: usize = 36;
const OUTSIDE_SLOT: i16 = -999;

#[derive(Debug, Clone, PartialEq)]
pub(in crate::network::status) struct ActiveBlockMenu {
    container_id: i32,
    state_id: i32,
    pos: crate::block_update::BlockPos,
    kind: ActiveBlockMenuKind,
    slots: Vec<ItemStack>,
}

#[derive(Debug, Clone, PartialEq)]
enum ActiveBlockMenuKind {
    Persistent {
        block_entity_id: &'static str,
        result_slot: Option<usize>,
    },
    Ephemeral {
        result_slot: Option<usize>,
    },
}

impl ActiveBlockMenu {
    pub(in crate::network::status) fn open(
        container_id: i32,
        pos: crate::block_update::BlockPos,
        live_kind: LiveBlockMenuKind,
        world_layout: &WorldLayout,
        world_seed: i64,
        chunk_cache: &GeneratedChunkCache,
    ) -> Self {
        match live_kind {
            LiveBlockMenuKind::Generic9x3 { block_entity_id } => Self::persistent(
                container_id,
                pos,
                block_entity_id,
                27,
                None,
                world_layout,
                world_seed,
                chunk_cache,
            ),
            LiveBlockMenuKind::Furnace { block_entity_id } => Self::persistent(
                container_id,
                pos,
                block_entity_id,
                3,
                Some(2),
                world_layout,
                world_seed,
                chunk_cache,
            ),
            LiveBlockMenuKind::Ephemeral {
                slot_count,
                result_slot,
            } => Self {
                container_id,
                state_id: 0,
                pos,
                kind: ActiveBlockMenuKind::Ephemeral { result_slot },
                slots: vec![ItemStack::empty(); slot_count],
            },
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the live open-menu inputs at one call site"
    )]
    fn persistent(
        container_id: i32,
        pos: crate::block_update::BlockPos,
        block_entity_id: &'static str,
        slot_count: usize,
        result_slot: Option<usize>,
        world_layout: &WorldLayout,
        world_seed: i64,
        chunk_cache: &GeneratedChunkCache,
    ) -> Self {
        let mut slots = vec![ItemStack::empty(); slot_count];
        if let Some(tag) = chunk_cache.block_entity_nbt_at(world_layout.root(), world_seed, pos) {
            load_items_from_block_entity_tag(&tag, &mut slots);
        }
        Self {
            container_id,
            state_id: 0,
            pos,
            kind: ActiveBlockMenuKind::Persistent {
                block_entity_id,
                result_slot,
            },
            slots,
        }
    }

    pub(in crate::network::status) fn container_id(&self) -> i32 {
        self.container_id
    }

    pub(in crate::network::status) fn write_full_content<W: Write>(
        &self,
        writer: &mut W,
        compression: CompressionState,
        state: &PlaySessionState,
    ) -> io::Result<()> {
        let packet = ClientboundContainerPacket {
            container_id: self.container_id,
            state_id: self.state_id,
            slots: self
                .flattened_slots(state)
                .iter()
                .map(raw_item_stack_from_item_stack)
                .collect::<io::Result<Vec<_>>>()?,
            carried_item: raw_item_stack_from_item_stack(&state.carried_item)?,
        };
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
            |payload| packet.write(payload),
        )
    }

    pub(in crate::network::status) fn handle_click(
        &mut self,
        packet: &ServerboundContainerClickPacket,
        state: &mut PlaySessionState,
        world_layout: &WorldLayout,
        world_seed: i64,
        chunk_cache: &GeneratedChunkCache,
    ) -> Vec<PlayInstruction> {
        if packet.container_id != self.container_id || !self.valid_slot(packet.slot_num) {
            return Vec::new();
        }

        let before_slots = self.flattened_slots(state);
        let before_carried = state.carried_item.clone();
        let mut menu = self.to_flat_menu(state);
        let dry_run = apply_scripted_packet(
            &mut menu.clone(),
            self.state_id,
            &ScriptedContainerClickPacket {
                container_id: packet.container_id,
                state_id: packet.state_id,
                slot: packet.slot_num as i32,
                button: packet.button_num as i32,
                mode: container_input_to_inventory(packet.container_input),
                changed_slots: Vec::new(),
                carried: ItemStack::empty(),
            },
        );
        let result = apply_scripted_packet(
            &mut menu,
            self.state_id,
            &ScriptedContainerClickPacket {
                container_id: packet.container_id,
                state_id: packet.state_id,
                slot: packet.slot_num as i32,
                button: packet.button_num as i32,
                mode: container_input_to_inventory(packet.container_input),
                changed_slots: Vec::new(),
                carried: dry_run.carried,
            },
        );
        if !result.accepted {
            return self.full_slot_resync(state);
        }

        self.state_id = self.state_id.wrapping_add(1);
        self.apply_flat_menu(menu, state);
        self.persist_if_needed(world_layout, world_seed, chunk_cache);

        let mut instructions = Vec::new();
        let after_slots = self.flattened_slots(state);
        for (slot, (before, after)) in before_slots.iter().zip(after_slots.iter()).enumerate() {
            if before != after {
                if let Ok(item_stack) = raw_item_stack_from_item_stack(after) {
                    instructions.push(PlayInstruction::ContainerSetSlot(
                        ClientboundContainerSetSlotPacket {
                            container_id: self.container_id,
                            state_id: self.state_id,
                            slot: slot as i16,
                            item_stack,
                        },
                    ));
                }
            }
        }
        if state.carried_item != before_carried {
            if let Ok(item_stack) = raw_item_stack_from_item_stack(&state.carried_item) {
                instructions.push(PlayInstruction::SetCursorItem(
                    ClientboundSetCursorItemPacket { item_stack },
                ));
            }
        }
        instructions
    }

    pub(in crate::network::status) fn close(mut self, state: &mut PlaySessionState) {
        if !state.carried_item.is_empty() {
            let stack = std::mem::replace(&mut state.carried_item, ItemStack::empty());
            let _ = state
                .inventory_menu
                .player_inventory_mut()
                .place_item_back_in_inventory(stack);
        }
        if matches!(self.kind, ActiveBlockMenuKind::Ephemeral { .. }) {
            let inventory = state.inventory_menu.player_inventory_mut();
            for slot in &mut self.slots {
                if !slot.is_empty() {
                    let stack = std::mem::replace(slot, ItemStack::empty());
                    let _ = inventory.place_item_back_in_inventory(stack);
                }
            }
        }
    }

    fn valid_slot(&self, slot: i16) -> bool {
        slot == OUTSIDE_SLOT || (0..self.total_slot_count() as i16).contains(&slot)
    }

    fn total_slot_count(&self) -> usize {
        self.slots.len() + PLAYER_SLOTS
    }

    fn flattened_slots(&self, state: &PlaySessionState) -> Vec<ItemStack> {
        let mut slots = self.slots.clone();
        let inventory = state.inventory_menu.player_inventory();
        for menu_slot in self.slots.len()..self.total_slot_count() {
            slots.push(read_player_menu_slot(menu_slot, self.slots.len(), inventory));
        }
        slots
    }

    fn to_flat_menu(&self, state: &PlaySessionState) -> Menu {
        let result_slot = match self.kind {
            ActiveBlockMenuKind::Persistent { result_slot, .. }
            | ActiveBlockMenuKind::Ephemeral { result_slot } => result_slot,
        };
        let mut menu = Menu::new(self.total_slot_count());
        for (slot, stack) in self.flattened_slots(state).into_iter().enumerate() {
            menu.slots[slot] = Slot {
                stack,
                max_stack_size: 64,
                may_place: result_slot != Some(slot),
                may_pickup: true,
            };
        }
        menu.carried = state.carried_item.clone();
        menu
    }

    fn apply_flat_menu(&mut self, menu: Menu, state: &mut PlaySessionState) {
        for slot in 0..self.slots.len() {
            self.slots[slot] = menu.slots[slot].stack.clone();
        }
        let player_start = self.slots.len();
        let inventory = state.inventory_menu.player_inventory_mut();
        for menu_slot in player_start..self.total_slot_count() {
            write_player_menu_slot(
                menu_slot,
                player_start,
                inventory,
                menu.slots[menu_slot].stack.clone(),
            );
        }
        state.carried_item = menu.carried;
    }

    fn full_slot_resync(&self, state: &PlaySessionState) -> Vec<PlayInstruction> {
        self.flattened_slots(state)
            .into_iter()
            .enumerate()
            .filter_map(|(slot, stack)| {
                Some(PlayInstruction::ContainerSetSlot(
                    ClientboundContainerSetSlotPacket {
                        container_id: self.container_id,
                        state_id: self.state_id,
                        slot: slot as i16,
                        item_stack: raw_item_stack_from_item_stack(&stack).ok()?,
                    },
                ))
            })
            .chain(
                raw_item_stack_from_item_stack(&state.carried_item)
                    .ok()
                    .map(|item_stack| {
                        PlayInstruction::SetCursorItem(ClientboundSetCursorItemPacket {
                            item_stack,
                        })
                    }),
            )
            .collect()
    }

    fn persist_if_needed(
        &self,
        world_layout: &WorldLayout,
        world_seed: i64,
        chunk_cache: &GeneratedChunkCache,
    ) {
        let ActiveBlockMenuKind::Persistent {
            block_entity_id, ..
        } = self.kind
        else {
            return;
        };
        let existing = chunk_cache.block_entity_nbt_at(world_layout.root(), world_seed, self.pos);
        let tag = block_entity_tag(block_entity_id, self.pos, &self.slots, existing.as_ref());
        chunk_cache.set_block_entity_nbt(world_layout.root(), world_seed, self.pos, tag);
    }
}

fn read_player_menu_slot(
    menu_index: usize,
    player_start: usize,
    player: &PlayerInventory,
) -> ItemStack {
    player_slot_for(menu_index, player_start)
        .map(|slot| player.get(slot).clone())
        .unwrap_or_else(ItemStack::empty)
}

fn write_player_menu_slot(
    menu_index: usize,
    player_start: usize,
    player: &mut PlayerInventory,
    stack: ItemStack,
) {
    if let Some(slot) = player_slot_for(menu_index, player_start) {
        player.set(slot, stack);
    }
}

fn player_slot_for(menu_index: usize, player_start: usize) -> Option<usize> {
    let local = menu_index.checked_sub(player_start)?;
    if local < PLAYER_MAIN_STORAGE {
        Some(9 + local)
    } else if local < PLAYER_SLOTS {
        Some(local - PLAYER_MAIN_STORAGE)
    } else {
        None
    }
}

fn container_input_to_inventory(input: ContainerInput) -> crate::inventory::ContainerInput {
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

fn load_items_from_block_entity_tag(tag: &Tag, slots: &mut [ItemStack]) {
    let Some(entries) = compound_entries(tag) else {
        return;
    };
    let Some(Tag::List(items)) = entries
        .iter()
        .find(|(name, _)| name == "Items")
        .map(|(_, tag)| tag)
    else {
        return;
    };
    for item in items {
        let Some(item_entries) = compound_entries(item) else {
            continue;
        };
        let Some(slot) = tag_byte_field(item_entries, "Slot").and_then(|slot| {
            (0..slots.len() as i8)
                .contains(&slot)
                .then_some(slot as usize)
        }) else {
            continue;
        };
        let Some(id) = tag_string_field(item_entries, "id").and_then(item_static_name) else {
            continue;
        };
        let count = tag_int_field(item_entries, "count").unwrap_or(1);
        slots[slot] = ItemStack::new(id, count);
    }
}

fn block_entity_tag(
    block_entity_id: &'static str,
    pos: crate::block_update::BlockPos,
    slots: &[ItemStack],
    existing: Option<&Tag>,
) -> Tag {
    let mut fields = existing
        .and_then(compound_entries)
        .map(|entries| {
            entries
                .iter()
                .filter(|(key, _)| !matches!(key.as_str(), "id" | "x" | "y" | "z" | "Items"))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    fields.insert(0, ("id".to_string(), Tag::String(block_entity_id.to_string())));
    fields.insert(1, ("x".to_string(), Tag::Int(pos.x)));
    fields.insert(2, ("y".to_string(), Tag::Int(pos.y)));
    fields.insert(3, ("z".to_string(), Tag::Int(pos.z)));
    fields.push(("Items".to_string(), Tag::List(item_tags(slots))));
    Tag::Compound(fields)
}

fn item_tags(slots: &[ItemStack]) -> Vec<Tag> {
    slots
        .iter()
        .enumerate()
        .filter(|(_, stack)| !stack.is_empty())
        .map(|(slot, stack)| {
            Tag::Compound(vec![
                ("Slot".to_string(), Tag::Byte(slot as i8)),
                ("id".to_string(), Tag::String(stack.item_id().to_string())),
                ("count".to_string(), Tag::Int(stack.count())),
            ])
        })
        .collect()
}

fn compound_entries(tag: &Tag) -> Option<&[(String, Tag)]> {
    match tag {
        Tag::Compound(entries) => Some(entries),
        _ => None,
    }
}

fn tag_byte_field(entries: &[(String, Tag)], key: &str) -> Option<i8> {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, tag)| match tag {
            Tag::Byte(value) => Some(*value),
            _ => None,
        })
}

fn tag_int_field(entries: &[(String, Tag)], key: &str) -> Option<i32> {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, tag)| match tag {
            Tag::Int(value) => Some(*value),
            _ => None,
        })
}

fn tag_string_field<'a>(entries: &'a [(String, Tag)], key: &str) -> Option<&'a str> {
    entries
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, tag)| match tag {
            Tag::String(value) => Some(value.as_str()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_chest_menu_click_moves_inventory_item_into_persisted_block_entity_slot() {
        let pos = crate::block_update::BlockPos { x: 1, y: 64, z: 2 };
        let mut state = PlaySessionState::default();
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:stone", 16));
        let mut menu = ActiveBlockMenu {
            container_id: 7,
            state_id: 0,
            pos,
            kind: ActiveBlockMenuKind::Persistent {
                block_entity_id: "minecraft:chest",
                result_slot: None,
            },
            slots: vec![ItemStack::empty(); 27],
        };
        let world_root =
            std::env::temp_dir().join(format!("vibecraft-active-menu-{}", std::process::id()));
        let layout = WorldLayout::new(&world_root);
        let cache = GeneratedChunkCache::default();

        let pickup = ServerboundContainerClickPacket {
            container_id: 7,
            state_id: 0,
            slot_num: (27 + 27) as i16,
            button_num: 0,
            container_input: ContainerInput::Pickup,
            changed_slots: BTreeMap::new(),
            carried_item: crate::network::play::HashedStack::empty(),
        };
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        let place = ServerboundContainerClickPacket {
            state_id: 1,
            slot_num: 0,
            ..pickup
        };
        menu.handle_click(&place, &mut state, &layout, 0, &cache);

        assert_eq!(menu.slots[0], ItemStack::new("minecraft:stone", 16));
        assert!(state.inventory_menu.player_inventory().get(0).is_empty());
        let tag = block_entity_tag("minecraft:chest", pos, &menu.slots, None);
        let Tag::Compound(entries) = tag else {
            panic!("block entity tag should be compound");
        };
        assert!(matches!(
            entries.iter().find(|(name, _)| name == "Items"),
            Some((_, Tag::List(items))) if items.len() == 1
        ));
    }

    #[test]
    fn closing_ephemeral_menu_returns_input_slots_to_player_inventory() {
        let mut state = PlaySessionState::default();
        let menu = ActiveBlockMenu {
            container_id: 3,
            state_id: 0,
            pos: crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            kind: ActiveBlockMenuKind::Ephemeral {
                result_slot: Some(1),
            },
            slots: vec![ItemStack::new("minecraft:stone", 4), ItemStack::empty()],
        };
        menu.close(&mut state);

        assert_eq!(
            state.inventory_menu.player_inventory().get(0).item_id(),
            "minecraft:stone"
        );
        assert_eq!(
            state.inventory_menu.player_inventory().get(0).count(),
            4
        );
    }
}
