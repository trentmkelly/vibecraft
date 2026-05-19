#![allow(dead_code)]

use crate::item_stack::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerInput {
    Pickup,
    QuickMove,
    Swap,
    Clone,
    Throw,
    QuickCraft,
    PickupAll,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub stack: ItemStack,
    pub max_stack_size: i32,
    pub may_place: bool,
    pub may_pickup: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub slots: Vec<Slot>,
    pub last_slots: Vec<ItemStack>,
    pub remote_slots: Vec<ItemStack>,
    pub data_slots: Vec<DataSlot>,
    pub remote_data_slots: Vec<i32>,
    pub listeners: Vec<ContainerListener>,
    pub synchronizer: Option<ContainerSynchronizer>,
    pub carried: ItemStack,
    pub remote_carried: ItemStack,
    pub hotbar: Vec<ItemStack>,
    pub creative: bool,
    pub dropped: Vec<ItemStack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotChange {
    pub slot: usize,
    pub stack: ItemStack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSlot {
    value: i32,
    previous_value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerData {
    values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataChange {
    pub id: usize,
    pub value: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerListenerEvent {
    SlotChanged(SlotChange),
    DataChanged(DataChange),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerListener {
    pub events: Vec<ContainerListenerEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerSynchronizerEvent {
    InitialData(MenuDataSync),
    SlotChanged(SlotChange),
    CarriedChanged(ItemStack),
    DataChanged(DataChange),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerSynchronizer {
    pub events: Vec<ContainerSynchronizerEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuDataSync {
    pub slots: Vec<SlotChange>,
    pub data: Vec<DataChange>,
    pub carried: Option<ItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryAction {
    Noop,
    PickedUp { slot: usize, count: i32 },
    Placed { slot: usize, count: i32 },
    Swapped { slot: usize, hotbar_slot: usize },
    QuickMoved { from: usize, to: usize, count: i32 },
    Cloned { slot: usize },
    Dropped { slot: Option<usize>, count: i32 },
    QuickCrafted { slots: usize, each: i32 },
    PickedUpAll { count: i32 },
}

impl Slot {
    pub fn empty() -> Self {
        Self {
            stack: ItemStack::empty(),
            max_stack_size: 64,
            may_place: true,
            may_pickup: true,
        }
    }

    pub fn with_stack(stack: ItemStack) -> Self {
        Self {
            stack,
            max_stack_size: 64,
            may_place: true,
            may_pickup: true,
        }
    }

    pub fn has_item(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn safe_take(&mut self, amount: i32) -> ItemStack {
        if !self.may_pickup || self.stack.is_empty() {
            return ItemStack::empty();
        }
        self.stack.split(amount)
    }

    pub fn safe_insert(&mut self, carried: &mut ItemStack, amount: i32) -> i32 {
        if !self.may_place || carried.is_empty() || amount <= 0 {
            return 0;
        }
        if self.stack.is_empty() {
            let inserted = amount.min(carried.count()).min(self.max_stack_size);
            self.stack = carried.split(inserted);
            inserted
        } else if same_item_same_components(&self.stack, carried) {
            let room =
                self.max_stack_size.min(self.stack.max_stack_size() as i32) - self.stack.count();
            let inserted = amount.min(carried.count()).min(room.max(0));
            self.stack.grow(inserted);
            carried.shrink(inserted);
            inserted
        } else {
            0
        }
    }
}

impl DataSlot {
    pub fn standalone() -> Self {
        Self {
            value: 0,
            previous_value: 0,
        }
    }

    pub fn with_value(value: i32) -> Self {
        Self {
            value,
            previous_value: value,
        }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
    }

    pub fn check_and_clear_update_flag(&mut self) -> bool {
        let changed = self.value != self.previous_value;
        self.previous_value = self.value;
        changed
    }
}

impl ContainerData {
    pub fn new(count: usize) -> Self {
        Self {
            values: vec![0; count],
        }
    }

    pub fn from_values(values: Vec<i32>) -> Self {
        Self { values }
    }

    pub fn get(&self, data_id: usize) -> Option<i32> {
        self.values.get(data_id).copied()
    }

    pub fn set(&mut self, data_id: usize, value: i32) -> bool {
        let Some(slot) = self.values.get_mut(data_id) else {
            return false;
        };
        *slot = value;
        true
    }

    pub fn get_count(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        self.values.iter().copied()
    }
}

impl ContainerListener {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn slot_changed(&mut self, slot: usize, stack: ItemStack) {
        self.events
            .push(ContainerListenerEvent::SlotChanged(SlotChange {
                slot,
                stack,
            }));
    }

    pub fn data_changed(&mut self, id: usize, value: i32) {
        self.events
            .push(ContainerListenerEvent::DataChanged(DataChange {
                id,
                value,
            }));
    }
}

impl Default for ContainerListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerSynchronizer {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn send_initial_data(&mut self, sync: MenuDataSync) {
        self.events
            .push(ContainerSynchronizerEvent::InitialData(sync));
    }

    pub fn send_slot_change(&mut self, slot: usize, stack: ItemStack) {
        self.events
            .push(ContainerSynchronizerEvent::SlotChanged(SlotChange {
                slot,
                stack,
            }));
    }

    pub fn send_carried_change(&mut self, stack: ItemStack) {
        self.events
            .push(ContainerSynchronizerEvent::CarriedChanged(stack));
    }

    pub fn send_data_change(&mut self, id: usize, value: i32) {
        self.events
            .push(ContainerSynchronizerEvent::DataChanged(DataChange {
                id,
                value,
            }));
    }
}

impl Default for ContainerSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![Slot::empty(); slot_count],
            last_slots: vec![ItemStack::empty(); slot_count],
            remote_slots: vec![ItemStack::empty(); slot_count],
            data_slots: Vec::new(),
            remote_data_slots: Vec::new(),
            listeners: Vec::new(),
            synchronizer: None,
            carried: ItemStack::empty(),
            remote_carried: ItemStack::empty(),
            hotbar: vec![ItemStack::empty(); 9],
            creative: false,
            dropped: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: ContainerListener) {
        self.listeners.push(listener);
        self.broadcast_changes();
    }

    pub fn remove_listener(&mut self, index: usize) -> Option<ContainerListener> {
        if index >= self.listeners.len() {
            return None;
        }
        Some(self.listeners.remove(index))
    }

    pub fn set_synchronizer(&mut self, synchronizer: ContainerSynchronizer) {
        self.synchronizer = Some(synchronizer);
        let sync = self.send_all_data_to_remote();
        if let Some(synchronizer) = &mut self.synchronizer {
            synchronizer.send_initial_data(sync);
        }
    }

    pub fn add_data_slot(&mut self, data_slot: DataSlot) -> usize {
        let index = self.data_slots.len();
        self.data_slots.push(data_slot);
        self.remote_data_slots.push(0);
        index
    }

    pub fn add_data_slots(&mut self, container: &ContainerData) {
        for value in container.iter() {
            self.add_data_slot(DataSlot::with_value(value));
        }
    }

    pub fn send_all_data_to_remote(&mut self) -> MenuDataSync {
        let slots = self
            .slots
            .iter()
            .enumerate()
            .map(|(slot, slot_data)| {
                self.remote_slots[slot] = slot_data.stack.clone();
                SlotChange {
                    slot,
                    stack: slot_data.stack.clone(),
                }
            })
            .collect();
        let data = self
            .data_slots
            .iter()
            .enumerate()
            .map(|(id, data_slot)| {
                let value = data_slot.get();
                self.remote_data_slots[id] = value;
                DataChange { id, value }
            })
            .collect();
        self.remote_carried = self.carried.clone();
        MenuDataSync {
            slots,
            data,
            carried: Some(self.carried.clone()),
        }
    }

    pub fn send_slot_change(&mut self, slot: usize) -> Option<SlotChange> {
        let current = self.slots.get(slot)?.stack.clone();
        let remote = self.remote_slots.get_mut(slot)?;
        if same_item_same_components(remote, &current) && remote.count() == current.count() {
            return None;
        }
        *remote = current.clone();
        Some(SlotChange {
            slot,
            stack: current,
        })
    }

    pub fn send_carried_change(&mut self) -> Option<ItemStack> {
        if same_item_same_components(&self.remote_carried, &self.carried)
            && self.remote_carried.count() == self.carried.count()
        {
            return None;
        }
        self.remote_carried = self.carried.clone();
        Some(self.carried.clone())
    }

    pub fn send_data_change(&mut self, id: usize) -> Option<DataChange> {
        let current = self.data_slots.get(id)?.get();
        let remote = self.remote_data_slots.get_mut(id)?;
        if *remote == current {
            return None;
        }
        *remote = current;
        Some(DataChange { id, value: current })
    }

    pub fn send_data_changes(&mut self) -> Vec<DataChange> {
        let mut changes = Vec::new();
        for id in 0..self.data_slots.len() {
            if self.data_slots[id].check_and_clear_update_flag() {
                if let Some(change) = self.send_data_change(id) {
                    changes.push(change);
                }
            }
        }
        changes
    }

    pub fn broadcast_changes(&mut self) -> MenuDataSync {
        let mut slot_changes = Vec::new();
        for slot in 0..self.slots.len() {
            let current = self.slots[slot].stack.clone();
            if !same_stack(&self.last_slots[slot], &current) {
                self.last_slots[slot] = current.clone();
                for listener in &mut self.listeners {
                    listener.slot_changed(slot, current.clone());
                }
            }
            if let Some(change) = self.send_slot_change(slot) {
                if let Some(synchronizer) = &mut self.synchronizer {
                    synchronizer.send_slot_change(change.slot, change.stack.clone());
                }
                slot_changes.push(change);
            }
        }

        let carried = self.send_carried_change();
        if let Some(stack) = &carried {
            if let Some(synchronizer) = &mut self.synchronizer {
                synchronizer.send_carried_change(stack.clone());
            }
        }

        let mut data = Vec::new();
        for id in 0..self.data_slots.len() {
            let current = self.data_slots[id].get();
            if self.data_slots[id].check_and_clear_update_flag() {
                for listener in &mut self.listeners {
                    listener.data_changed(id, current);
                }
            }
            if let Some(change) = self.send_data_change(id) {
                if let Some(synchronizer) = &mut self.synchronizer {
                    synchronizer.send_data_change(change.id, change.value);
                }
                data.push(change);
            }
        }

        MenuDataSync {
            slots: slot_changes,
            data,
            carried,
        }
    }

    pub fn broadcast_full_state(&mut self) -> MenuDataSync {
        for slot in 0..self.slots.len() {
            let current = self.slots[slot].stack.clone();
            self.last_slots[slot] = current.clone();
            for listener in &mut self.listeners {
                listener.slot_changed(slot, current.clone());
            }
        }
        for id in 0..self.data_slots.len() {
            let current = self.data_slots[id].get();
            self.data_slots[id].check_and_clear_update_flag();
            for listener in &mut self.listeners {
                listener.data_changed(id, current);
            }
        }
        let sync = self.send_all_data_to_remote();
        if let Some(synchronizer) = &mut self.synchronizer {
            synchronizer.send_initial_data(sync.clone());
        }
        sync
    }

    pub fn click_pickup(
        &mut self,
        slot_index: Option<usize>,
        action: ClickAction,
    ) -> InventoryAction {
        let Some(slot_index) = slot_index else {
            if self.carried.is_empty() {
                return InventoryAction::Noop;
            }
            let amount = match action {
                ClickAction::Primary => self.carried.count(),
                ClickAction::Secondary => 1,
            };
            let dropped = self.carried.split(amount);
            self.dropped.push(dropped);
            return InventoryAction::Dropped {
                slot: None,
                count: amount,
            };
        };

        let slot = &mut self.slots[slot_index];
        if self.carried.is_empty() {
            let amount = match action {
                ClickAction::Primary => slot.stack.count(),
                ClickAction::Secondary => (slot.stack.count() + 1) / 2,
            };
            self.carried = slot.safe_take(amount);
            InventoryAction::PickedUp {
                slot: slot_index,
                count: self.carried.count(),
            }
        } else if slot.stack.is_empty() || same_item_same_components(&slot.stack, &self.carried) {
            let amount = match action {
                ClickAction::Primary => self.carried.count(),
                ClickAction::Secondary => 1,
            };
            let inserted = slot.safe_insert(&mut self.carried, amount);
            InventoryAction::Placed {
                slot: slot_index,
                count: inserted,
            }
        } else if slot.may_place {
            std::mem::swap(&mut slot.stack, &mut self.carried);
            InventoryAction::Swapped {
                slot: slot_index,
                hotbar_slot: usize::MAX,
            }
        } else {
            InventoryAction::Noop
        }
    }

    pub fn quick_move(&mut self, slot_index: usize) -> InventoryAction {
        if slot_index >= self.slots.len() || self.slots[slot_index].stack.is_empty() {
            return InventoryAction::Noop;
        }
        let mut moving = self.slots[slot_index].stack.clone();
        let original = moving.count();
        let mut first_target = None;
        for (target_index, target) in self.slots.iter_mut().enumerate() {
            if target_index == slot_index {
                continue;
            }
            let moving_count = moving.count();
            if target.safe_insert(&mut moving, moving_count) > 0 && first_target.is_none() {
                first_target = Some(target_index);
            }
            if moving.is_empty() {
                break;
            }
        }
        let moved = original - moving.count();
        self.slots[slot_index].stack.set_count(moving.count());
        if moved > 0 {
            InventoryAction::QuickMoved {
                from: slot_index,
                to: first_target.unwrap_or(slot_index),
                count: moved,
            }
        } else {
            InventoryAction::Noop
        }
    }

    pub fn swap_hotbar(&mut self, slot_index: usize, hotbar_slot: usize) -> InventoryAction {
        if slot_index >= self.slots.len() || hotbar_slot >= self.hotbar.len() {
            return InventoryAction::Noop;
        }
        std::mem::swap(
            &mut self.slots[slot_index].stack,
            &mut self.hotbar[hotbar_slot],
        );
        InventoryAction::Swapped {
            slot: slot_index,
            hotbar_slot,
        }
    }

    pub fn clone_slot(&mut self, slot_index: usize) -> InventoryAction {
        if !self.creative || !self.carried.is_empty() || slot_index >= self.slots.len() {
            return InventoryAction::Noop;
        }
        let slot = &self.slots[slot_index];
        if slot.stack.is_empty() {
            return InventoryAction::Noop;
        }
        self.carried = slot
            .stack
            .copy_with_count(slot.stack.max_stack_size() as i32);
        InventoryAction::Cloned { slot: slot_index }
    }

    pub fn throw_from_slot(&mut self, slot_index: usize, full_stack: bool) -> InventoryAction {
        if slot_index >= self.slots.len() || !self.carried.is_empty() {
            return InventoryAction::Noop;
        }
        let amount = if full_stack {
            self.slots[slot_index].stack.count()
        } else {
            1
        };
        let dropped = self.slots[slot_index].safe_take(amount);
        let count = dropped.count();
        if count > 0 {
            self.dropped.push(dropped);
            InventoryAction::Dropped {
                slot: Some(slot_index),
                count,
            }
        } else {
            InventoryAction::Noop
        }
    }

    pub fn quick_craft(&mut self, slot_indices: &[usize]) -> InventoryAction {
        if self.carried.is_empty() || slot_indices.is_empty() {
            return InventoryAction::Noop;
        }
        let each = self.carried.count() / slot_indices.len() as i32;
        if each <= 0 {
            return InventoryAction::Noop;
        }
        let mut changed = 0;
        for &slot_index in slot_indices {
            if let Some(slot) = self.slots.get_mut(slot_index) {
                if slot.safe_insert(&mut self.carried, each) > 0 {
                    changed += 1;
                }
            }
        }
        InventoryAction::QuickCrafted {
            slots: changed,
            each,
        }
    }

    pub fn pickup_all(&mut self, start_slot: usize) -> InventoryAction {
        if self.carried.is_empty() || start_slot >= self.slots.len() {
            return InventoryAction::Noop;
        }
        let mut moved = 0;
        for slot in self.slots.iter_mut().rev() {
            if self.carried.count() >= self.carried.max_stack_size() as i32 {
                break;
            }
            if slot.has_item() && same_item_same_components(&slot.stack, &self.carried) {
                let room = self.carried.max_stack_size() as i32 - self.carried.count();
                let taken = slot.safe_take(room);
                moved += taken.count();
                self.carried.grow(taken.count());
            }
        }
        InventoryAction::PickedUpAll { count: moved }
    }
}

pub fn same_item_same_components(a: &ItemStack, b: &ItemStack) -> bool {
    (a.is_empty() && b.is_empty())
        || (!a.is_empty()
            && !b.is_empty()
            && a.item_id() == b.item_id()
            && a.components_patch() == b.components_patch())
}

pub fn same_stack(a: &ItemStack, b: &ItemStack) -> bool {
    same_item_same_components(a, b) && a.count() == b.count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_and_secondary_pickup_follow_carried_stack_rules() {
        let mut menu = Menu::new(2);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 5));
        assert_eq!(
            menu.click_pickup(Some(0), ClickAction::Secondary),
            InventoryAction::PickedUp { slot: 0, count: 3 }
        );
        assert_eq!(menu.carried.count(), 3);
        assert_eq!(menu.slots[0].stack.count(), 2);
        assert_eq!(
            menu.click_pickup(Some(1), ClickAction::Primary),
            InventoryAction::Placed { slot: 1, count: 3 }
        );
        assert!(menu.carried.is_empty());
    }

    #[test]
    fn pickup_swaps_different_items_when_slot_accepts_carried() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:apple", 1));
        menu.carried = ItemStack::new("minecraft:stick", 4);
        assert_eq!(
            menu.click_pickup(Some(0), ClickAction::Primary),
            InventoryAction::Swapped {
                slot: 0,
                hotbar_slot: usize::MAX
            }
        );
        assert_eq!(menu.slots[0].stack.item_id(), "minecraft:stick");
        assert_eq!(menu.carried.item_id(), "minecraft:apple");
    }

    #[test]
    fn carried_click_outside_drops_primary_or_secondary_amount() {
        let mut menu = Menu::new(0);
        menu.carried = ItemStack::new("minecraft:stick", 5);
        assert_eq!(
            menu.click_pickup(None, ClickAction::Secondary),
            InventoryAction::Dropped {
                slot: None,
                count: 1
            }
        );
        assert_eq!(menu.carried.count(), 4);
        assert_eq!(menu.dropped[0].count(), 1);
    }

    #[test]
    fn hotbar_swap_exchanges_slot_and_hotbar_stack() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:apple", 1));
        menu.hotbar[2] = ItemStack::new("minecraft:stick", 8);
        assert_eq!(
            menu.swap_hotbar(0, 2),
            InventoryAction::Swapped {
                slot: 0,
                hotbar_slot: 2
            }
        );
        assert_eq!(menu.slots[0].stack.item_id(), "minecraft:stick");
        assert_eq!(menu.hotbar[2].item_id(), "minecraft:apple");
    }

    #[test]
    fn creative_clone_copies_slot_to_max_stack_size() {
        let mut menu = Menu::new(1);
        menu.creative = true;
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:ender_pearl", 3));
        assert_eq!(menu.clone_slot(0), InventoryAction::Cloned { slot: 0 });
        assert_eq!(menu.carried.count(), 16);
    }

    #[test]
    fn throw_from_slot_removes_one_or_full_stack() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 3));
        assert_eq!(
            menu.throw_from_slot(0, false),
            InventoryAction::Dropped {
                slot: Some(0),
                count: 1
            }
        );
        assert_eq!(menu.slots[0].stack.count(), 2);
        assert_eq!(
            menu.throw_from_slot(0, true),
            InventoryAction::Dropped {
                slot: Some(0),
                count: 2
            }
        );
        assert!(menu.slots[0].stack.is_empty());
    }

    #[test]
    fn quick_craft_distributes_carried_stack_evenly() {
        let mut menu = Menu::new(3);
        menu.carried = ItemStack::new("minecraft:stick", 9);
        assert_eq!(
            menu.quick_craft(&[0, 1, 2]),
            InventoryAction::QuickCrafted { slots: 3, each: 3 }
        );
        assert!(menu.carried.is_empty());
        assert_eq!(menu.slots[0].stack.count(), 3);
        assert_eq!(menu.slots[1].stack.count(), 3);
        assert_eq!(menu.slots[2].stack.count(), 3);
    }

    #[test]
    fn pickup_all_collects_matching_items_into_carried_stack() {
        let mut menu = Menu::new(3);
        menu.carried = ItemStack::new("minecraft:stick", 60);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 2));
        menu.slots[1] = Slot::with_stack(ItemStack::new("minecraft:apple", 2));
        menu.slots[2] = Slot::with_stack(ItemStack::new("minecraft:stick", 5));
        assert_eq!(
            menu.pickup_all(0),
            InventoryAction::PickedUpAll { count: 4 }
        );
        assert_eq!(menu.carried.count(), 64);
        assert_eq!(menu.slots[2].stack.count(), 1);
    }

    #[test]
    fn quick_move_merges_stack_into_existing_compatible_slot() {
        let mut menu = Menu::new(3);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 5));
        menu.slots[1] = Slot::with_stack(ItemStack::new("minecraft:stick", 60));
        assert_eq!(
            menu.quick_move(0),
            InventoryAction::QuickMoved {
                from: 0,
                to: 1,
                count: 5
            }
        );
        assert_eq!(menu.slots[1].stack.count(), 64);
        assert!(menu.slots[0].stack.is_empty());
        assert_eq!(menu.slots[2].stack.count(), 1);
    }

    #[test]
    fn remote_slot_shadow_copies_only_report_changed_slots_and_carried() {
        let mut menu = Menu::new(2);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 2));
        menu.carried = ItemStack::new("minecraft:apple", 1);

        let full = menu.send_all_data_to_remote();
        assert_eq!(full.slots.len(), 2);
        assert_eq!(full.slots[0].stack.item_id(), "minecraft:stick");
        assert!(full.data.is_empty());
        assert_eq!(full.carried.unwrap().item_id(), "minecraft:apple");
        assert!(menu.send_slot_change(0).is_none());
        assert!(menu.send_carried_change().is_none());

        menu.slots[0].stack.grow(1);
        let change = menu.send_slot_change(0).unwrap();
        assert_eq!(change.slot, 0);
        assert_eq!(change.stack.count(), 3);
        assert!(menu.send_slot_change(0).is_none());

        menu.carried = ItemStack::new("minecraft:diamond", 1);
        let carried = menu.send_carried_change().unwrap();
        assert_eq!(carried.item_id(), "minecraft:diamond");
        assert!(menu.send_carried_change().is_none());
    }

    #[test]
    fn data_slots_sync_all_values_then_only_changed_values() {
        let mut menu = Menu::new(0);
        menu.add_data_slots(&ContainerData::from_values(vec![20, 0, 80]));

        let full = menu.send_all_data_to_remote();
        assert_eq!(
            full.data,
            vec![
                DataChange { id: 0, value: 20 },
                DataChange { id: 1, value: 0 },
                DataChange { id: 2, value: 80 }
            ]
        );
        assert!(menu.send_data_changes().is_empty());

        menu.data_slots[1].set(7);
        let changes = menu.send_data_changes();
        assert_eq!(changes, vec![DataChange { id: 1, value: 7 }]);
        assert!(menu.send_data_changes().is_empty());
    }

    #[test]
    fn synchronizer_receives_initial_data_and_incremental_remote_changes() {
        let mut menu = Menu::new(1);
        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 2));
        menu.add_data_slot(DataSlot::with_value(5));
        menu.set_synchronizer(ContainerSynchronizer::new());

        let synchronizer = menu.synchronizer.as_ref().unwrap();
        assert_eq!(synchronizer.events.len(), 1);
        match &synchronizer.events[0] {
            ContainerSynchronizerEvent::InitialData(sync) => {
                assert_eq!(sync.slots[0].stack.item_id(), "minecraft:stick");
                assert_eq!(sync.data, vec![DataChange { id: 0, value: 5 }]);
            }
            event => panic!("unexpected synchronizer event: {event:?}"),
        }

        menu.slots[0].stack.grow(1);
        menu.carried = ItemStack::new("minecraft:apple", 1);
        menu.data_slots[0].set(9);
        let sync = menu.broadcast_changes();
        assert_eq!(sync.slots[0].stack.count(), 3);
        assert_eq!(sync.carried.unwrap().item_id(), "minecraft:apple");
        assert_eq!(sync.data, vec![DataChange { id: 0, value: 9 }]);

        let events = &menu.synchronizer.as_ref().unwrap().events;
        assert!(matches!(
            &events[1],
            ContainerSynchronizerEvent::SlotChanged(SlotChange { slot: 0, stack })
                if stack.count() == 3
        ));
        assert!(matches!(
            &events[2],
            ContainerSynchronizerEvent::CarriedChanged(stack) if stack.item_id() == "minecraft:apple"
        ));
        assert_eq!(
            events[3],
            ContainerSynchronizerEvent::DataChanged(DataChange { id: 0, value: 9 })
        );
        assert!(menu.broadcast_changes().slots.is_empty());
    }

    #[test]
    fn listeners_receive_local_slot_and_data_changes() {
        let mut menu = Menu::new(1);
        menu.add_data_slot(DataSlot::standalone());
        menu.add_listener(ContainerListener::new());
        menu.listeners[0].events.clear();

        menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stone", 4));
        menu.data_slots[0].set(12);
        menu.broadcast_changes();

        assert_eq!(
            menu.listeners[0].events,
            vec![
                ContainerListenerEvent::SlotChanged(SlotChange {
                    slot: 0,
                    stack: ItemStack::new("minecraft:stone", 4)
                }),
                ContainerListenerEvent::DataChanged(DataChange { id: 0, value: 12 })
            ]
        );
    }
}
