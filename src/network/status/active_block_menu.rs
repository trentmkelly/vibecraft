use super::block_menu_open::LiveBlockMenuKind;
use super::*;
use crate::container_menus::CraftingMenu;
use crate::inventory::{Menu, Slot};
use crate::inventory_transactions::{apply_scripted_packet, ScriptedContainerClickPacket};
use crate::network::play::{
    ClientboundContainerPacket, ClientboundSetCursorItemPacket, ContainerInput,
    CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
};
use crate::recipe_system::RecipeMap;

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
    Crafting { menu: Box<CraftingMenu> },
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
        recipes: &RecipeMap,
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
            LiveBlockMenuKind::Crafting => Self {
                container_id,
                state_id: 0,
                pos,
                kind: ActiveBlockMenuKind::Crafting {
                    menu: Box::new(CraftingMenu::new(recipes.clone())),
                },
                slots: Vec::new(),
            },
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
        &mut self,
        writer: &mut W,
        compression: CompressionState,
        state: &PlaySessionState,
    ) -> io::Result<()> {
        self.increment_state_id();
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
        if matches!(self.kind, ActiveBlockMenuKind::Crafting { .. })
            && packet.container_input == ContainerInput::Pickup
            && packet.slot_num == CraftingMenu::RESULT_SLOT as i16
        {
            return self.handle_crafting_result_pickup(packet, state);
        }
        if matches!(self.kind, ActiveBlockMenuKind::Crafting { .. })
            && packet.container_input == ContainerInput::QuickMove
        {
            return self.handle_crafting_quick_move(packet, state);
        }

        let full_resync_needed = packet.state_id != self.state_id;
        let before_slots = self.flattened_slots(state);
        let before_carried = state.carried_item.clone();
        let mut menu = self.to_flat_menu(state);
        // Java `ServerGamePacketListenerImpl.handleContainerClick` computes
        // `fullResyncNeeded` from the packet state, but still calls
        // `containerMenu.clicked(...)` before broadcasting the full state. Apply
        // the click against the current server state here, then resync below if
        // the client's state id lagged behind.
        let mut dry_run_packet = scripted_click_packet(packet, Vec::new(), ItemStack::empty());
        dry_run_packet.state_id = self.state_id;
        let dry_run = apply_scripted_packet(&mut menu.clone(), self.state_id, &dry_run_packet);
        let mut scripted_packet = scripted_click_packet(packet, Vec::new(), dry_run.carried);
        scripted_packet.state_id = self.state_id;
        apply_scripted_packet(&mut menu, self.state_id, &scripted_packet);

        let result_slot_changed = self.result_slot_changed_after_click(&menu);
        self.increment_state_id();
        self.apply_flat_menu(menu, state);
        self.apply_result_slot_take_if_needed(result_slot_changed);
        self.persist_if_needed(world_layout, world_seed, chunk_cache);

        if full_resync_needed {
            let mut instructions = self.full_content_resync(state);
            self.push_recipe_unlocks(&mut instructions, state);
            return instructions;
        }

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
        self.push_recipe_unlocks(&mut instructions, state);
        instructions
    }

    fn handle_crafting_result_pickup(
        &mut self,
        packet: &ServerboundContainerClickPacket,
        state: &mut PlaySessionState,
    ) -> Vec<PlayInstruction> {
        let full_resync_needed = packet.state_id != self.state_id;
        let before_slots = self.flattened_slots(state);
        let before_carried = state.carried_item.clone();

        if let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind {
            // Java `ResultSlot` is fake: taking from it calls `onTake`, consumes
            // the craft grid, and never treats slot 0 as a normal mutable slot.
            // Stale-state result clicks are still applied by Java before the full
            // resync, same as ordinary slot clicks.
            let result = menu.result().clone();
            if can_carry_crafting_result(&state.carried_item, &result) {
                let taken = menu.take_result();
                if state.carried_item.is_empty() {
                    state.carried_item = taken;
                } else {
                    state.carried_item.grow(taken.count());
                }
            }
        }
        self.increment_state_id();

        if full_resync_needed {
            let mut instructions = self.full_content_resync(state);
            self.push_recipe_unlocks(&mut instructions, state);
            return instructions;
        }

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
        self.push_recipe_unlocks(&mut instructions, state);
        instructions
    }

    fn handle_crafting_quick_move(
        &mut self,
        packet: &ServerboundContainerClickPacket,
        state: &mut PlaySessionState,
    ) -> Vec<PlayInstruction> {
        let Ok(slot) = usize::try_from(packet.slot_num) else {
            return Vec::new();
        };
        let full_resync_needed = packet.state_id != self.state_id;
        let before_slots = self.flattened_slots(state);
        let before_carried = state.carried_item.clone();

        if let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind {
            // Java `CraftingMenu.quickMoveStack` handles the result slot specially:
            // the crafted item moves to player inventory and `ResultSlot.onTake`
            // consumes/remainders the 3x3 grid. The generic slot bridge would try
            // to insert the result into the first accepting slot, including empty
            // crafting-grid cells.
            menu.quick_move(slot, state.inventory_menu.player_inventory_mut());
        }
        self.increment_state_id();

        if full_resync_needed {
            let mut instructions = self.full_content_resync(state);
            self.push_recipe_unlocks(&mut instructions, state);
            return instructions;
        }

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
        self.push_recipe_unlocks(&mut instructions, state);
        instructions
    }

    pub(in crate::network::status) fn handle_place_recipe(
        &mut self,
        packet: &ServerboundPlaceRecipePacket,
        state: &mut PlaySessionState,
        recipes: &RecipeMap,
    ) -> bool {
        if packet.container_id != self.container_id || packet.recipe_index < 0 {
            return false;
        }
        let Some(holder) = recipes.holder_for_display_index(packet.recipe_index) else {
            return false;
        };
        if !state
            .inventory_menu
            .recipe_book_known_recipes()
            .contains(&holder.id)
        {
            return false;
        }
        let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind else {
            return false;
        };
        menu.place_recipe_from_inventory(
            holder.id,
            packet.use_max_items,
            state.inventory_menu.player_inventory_mut(),
        )
    }

    #[cfg(test)]
    pub(in crate::network::status) fn state_id(&self) -> i32 {
        self.state_id
    }

    pub(in crate::network::status) fn close(mut self, state: &mut PlaySessionState) {
        if !state.carried_item.is_empty() {
            let stack = std::mem::replace(&mut state.carried_item, ItemStack::empty());
            let _ = state
                .inventory_menu
                .player_inventory_mut()
                .place_item_back_in_inventory(stack);
        }
        if let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind {
            menu.removed(
                state.inventory_menu.player_inventory_mut(),
                &mut state.carried_item,
                false,
            );
        } else if matches!(self.kind, ActiveBlockMenuKind::Ephemeral { .. }) {
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
        match self.kind {
            ActiveBlockMenuKind::Crafting { .. } => CraftingMenu::SLOT_COUNT,
            _ => self.slots.len() + PLAYER_SLOTS,
        }
    }

    fn flattened_slots(&self, state: &PlaySessionState) -> Vec<ItemStack> {
        if let ActiveBlockMenuKind::Crafting { menu } = &self.kind {
            return menu.all_slots(state.inventory_menu.player_inventory());
        }
        let mut slots = self.slots.clone();
        let inventory = state.inventory_menu.player_inventory();
        for menu_slot in self.slots.len()..self.total_slot_count() {
            slots.push(read_player_menu_slot(menu_slot, self.slots.len(), inventory));
        }
        slots
    }

    fn to_flat_menu(&self, state: &PlaySessionState) -> Menu {
        let result_slot = match self.kind {
            ActiveBlockMenuKind::Crafting { .. } => Some(CraftingMenu::RESULT_SLOT),
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
        if let ActiveBlockMenuKind::Crafting {
            menu: crafting_menu,
        } = &mut self.kind
        {
            let inventory = state.inventory_menu.player_inventory_mut();
            for slot in CraftingMenu::GRID_START..CraftingMenu::GRID_END {
                let _ = crafting_menu.set_slot(slot, menu.slots[slot].stack.clone(), inventory);
            }
            for menu_slot in CraftingMenu::INV_START..CraftingMenu::SLOT_COUNT {
                let _ = crafting_menu.set_slot(menu_slot, menu.slots[menu_slot].stack.clone(), inventory);
            }
            state.carried_item = menu.carried;
            return;
        }
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

    fn result_slot_changed_after_click(&self, menu: &Menu) -> bool {
        matches!(self.kind, ActiveBlockMenuKind::Crafting { .. })
            && !self
                .flattened_result_slot()
                .is_empty()
            && menu
                .slots
                .get(CraftingMenu::RESULT_SLOT)
                .is_some_and(|slot| slot.stack != self.flattened_result_slot())
    }

    fn flattened_result_slot(&self) -> ItemStack {
        match &self.kind {
            ActiveBlockMenuKind::Crafting { menu } => menu.result().clone(),
            _ => ItemStack::empty(),
        }
    }

    fn apply_result_slot_take_if_needed(&mut self, result_slot_changed: bool) {
        if !result_slot_changed {
            return;
        }
        if let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind {
            let _ = menu.take_result();
        }
    }

    fn full_content_resync(&self, state: &PlaySessionState) -> Vec<PlayInstruction> {
        let Ok(slots) = self
            .flattened_slots(state)
            .iter()
            .map(raw_item_stack_from_item_stack)
            .collect::<io::Result<Vec<_>>>()
        else {
            return Vec::new();
        };
        let Ok(carried_item) = raw_item_stack_from_item_stack(&state.carried_item) else {
            return Vec::new();
        };
        vec![PlayInstruction::Container(ClientboundContainerPacket {
            container_id: self.container_id,
            state_id: self.state_id,
            slots,
            carried_item,
        })]
    }

    fn increment_state_id(&mut self) {
        // Java `AbstractContainerMenu.incrementStateId`: `(stateId + 1) & 32767`.
        self.state_id = (self.state_id + 1) & 0x7fff;
    }

    fn push_recipe_unlocks(
        &mut self,
        instructions: &mut Vec<PlayInstruction>,
        state: &mut PlaySessionState,
    ) {
        if let ActiveBlockMenuKind::Crafting { menu } = &mut self.kind {
            let unlocks = menu
                .drain_recipe_unlock_events()
                .into_iter()
                .filter(|recipe_id| state.inventory_menu.unlock_recipe(recipe_id))
                .collect::<Vec<_>>();
            if !unlocks.is_empty() {
                instructions.push(PlayInstruction::RecipesUnlocked(unlocks));
            }
        }
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

fn can_carry_crafting_result(carried: &ItemStack, result: &ItemStack) -> bool {
    if result.is_empty() {
        return false;
    }
    carried.is_empty()
        || (same_item_same_components(carried, result)
            && carried.count() + result.count() <= carried.max_stack_size() as i32)
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

fn scripted_click_packet(
    packet: &ServerboundContainerClickPacket,
    changed_slots: Vec<(i32, ItemStack)>,
    carried: ItemStack,
) -> ScriptedContainerClickPacket {
    ScriptedContainerClickPacket {
        container_id: packet.container_id,
        state_id: packet.state_id,
        slot: packet.slot_num as i32,
        button: packet.button_num as i32,
        mode: container_input_to_inventory(packet.container_input),
        changed_slots,
        carried,
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
#[path = "active_block_menu_tests.rs"]
mod tests;
