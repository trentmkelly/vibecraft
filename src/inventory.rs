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
    pub carried: ItemStack,
    pub hotbar: Vec<ItemStack>,
    pub creative: bool,
    pub dropped: Vec<ItemStack>,
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

impl Menu {
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![Slot::empty(); slot_count],
            carried: ItemStack::empty(),
            hotbar: vec![ItemStack::empty(); 9],
            creative: false,
            dropped: Vec::new(),
        }
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
}
