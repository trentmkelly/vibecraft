use super::*;

const QUICK_CRAFT_TYPE_CHARITABLE: i32 = 0;
const QUICK_CRAFT_TYPE_GREEDY: i32 = 1;
const QUICK_CRAFT_TYPE_CLONE: i32 = 2;
const QUICK_CRAFT_HEADER_START: i32 = 0;
const QUICK_CRAFT_HEADER_CONTINUE: i32 = 1;
const QUICK_CRAFT_HEADER_END: i32 = 2;

impl Menu {
    /// Mirrors Java `AbstractContainerMenu#doClick` for QUICK_CRAFT packets:
    /// START chooses the drag mode, CONTINUE records eligible slots, and END
    /// applies the split from the server-side carried stack.
    pub fn click_quick_craft(&mut self, slot_index: i32, button: i32) -> InventoryAction {
        let expected_status = self.quick_craft_status;
        self.quick_craft_status = quick_craft_header(button);
        if ((expected_status != QUICK_CRAFT_HEADER_CONTINUE
            || self.quick_craft_status != QUICK_CRAFT_HEADER_END)
            && expected_status != self.quick_craft_status)
            || self.carried.is_empty()
        {
            self.reset_quick_craft();
            return InventoryAction::Noop;
        }

        match self.quick_craft_status {
            QUICK_CRAFT_HEADER_START => self.start_quick_craft(button),
            QUICK_CRAFT_HEADER_CONTINUE => self.continue_quick_craft(slot_index),
            QUICK_CRAFT_HEADER_END => self.end_quick_craft(),
            _ => {
                self.reset_quick_craft();
                InventoryAction::Noop
            }
        }
    }

    pub fn quick_craft(&mut self, slot_indices: &[usize]) -> InventoryAction {
        if self.carried.is_empty() || slot_indices.is_empty() {
            return InventoryAction::Noop;
        }
        self.quick_craft_type = QUICK_CRAFT_TYPE_CHARITABLE;
        self.quick_craft_slots.clear();
        for &slot_index in slot_indices {
            if self.can_add_quick_craft_slot(slot_index) {
                self.quick_craft_slots.push(slot_index);
            }
        }
        self.end_quick_craft()
    }

    fn start_quick_craft(&mut self, button: i32) -> InventoryAction {
        self.quick_craft_type = quick_craft_type(button);
        if is_valid_quick_craft_type(self.quick_craft_type, self.creative) {
            self.quick_craft_status = QUICK_CRAFT_HEADER_CONTINUE;
            self.quick_craft_slots.clear();
        } else {
            self.reset_quick_craft();
        }
        InventoryAction::Noop
    }

    fn continue_quick_craft(&mut self, slot_index: i32) -> InventoryAction {
        let Ok(slot_index) = usize::try_from(slot_index) else {
            return InventoryAction::Noop;
        };
        if !self.quick_craft_slots.contains(&slot_index)
            && self.can_add_quick_craft_slot(slot_index)
        {
            self.quick_craft_slots.push(slot_index);
        }
        InventoryAction::Noop
    }

    fn end_quick_craft(&mut self) -> InventoryAction {
        if self.quick_craft_slots.is_empty() {
            self.reset_quick_craft();
            return InventoryAction::Noop;
        }
        if self.quick_craft_slots.len() == 1 {
            let slot = self.quick_craft_slots[0];
            let click_action = if self.quick_craft_type == QUICK_CRAFT_TYPE_GREEDY {
                ClickAction::Secondary
            } else {
                ClickAction::Primary
            };
            self.reset_quick_craft();
            return self.click_pickup(Some(slot), click_action);
        }

        let source = self.carried.clone();
        let each =
            quick_craft_place_count(self.quick_craft_slots.len(), self.quick_craft_type, &source);
        let mut remaining = self.carried.count();
        let mut changed = 0;
        for slot_index in self.quick_craft_slots.clone() {
            let Some(slot) = self.slots.get_mut(slot_index) else {
                continue;
            };
            if !can_item_quick_replace(slot, &source)
                || !slot.may_place
                || (self.quick_craft_type != QUICK_CRAFT_TYPE_CLONE
                    && source.count() < self.quick_craft_slots.len() as i32)
            {
                continue;
            }
            let carry = if slot.has_item() {
                slot.stack.count()
            } else {
                0
            };
            let max_size = (source.max_stack_size() as i32).min(slot.max_stack_size);
            let new_count = (each + carry).min(max_size);
            remaining -= new_count - carry;
            slot.stack = source.copy_with_count(new_count);
            changed += 1;
        }
        self.carried.set_count(remaining);
        self.reset_quick_craft();
        InventoryAction::QuickCrafted {
            slots: changed,
            each,
        }
    }

    fn can_add_quick_craft_slot(&self, slot_index: usize) -> bool {
        let Some(slot) = self.slots.get(slot_index) else {
            return false;
        };
        can_item_quick_replace(slot, &self.carried)
            && slot.may_place
            && (self.quick_craft_type == QUICK_CRAFT_TYPE_CLONE
                || self.carried.count() > self.quick_craft_slots.len() as i32)
    }

    fn reset_quick_craft(&mut self) {
        self.quick_craft_status = 0;
        self.quick_craft_slots.clear();
    }
}

fn quick_craft_type(mask: i32) -> i32 {
    mask >> 2 & 3
}

fn quick_craft_header(mask: i32) -> i32 {
    mask & 3
}

pub fn quick_craft_mask(header: i32, quick_craft_type: i32) -> i32 {
    header & 3 | (quick_craft_type & 3) << 2
}

fn is_valid_quick_craft_type(quick_craft_type: i32, creative: bool) -> bool {
    quick_craft_type == QUICK_CRAFT_TYPE_CHARITABLE
        || quick_craft_type == QUICK_CRAFT_TYPE_GREEDY
        || (quick_craft_type == QUICK_CRAFT_TYPE_CLONE && creative)
}

fn can_item_quick_replace(slot: &Slot, stack: &ItemStack) -> bool {
    slot.stack.is_empty() || same_item_same_components(stack, &slot.stack)
}

pub fn quick_craft_place_count(
    quick_craft_slots_size: usize,
    quick_craft_type: i32,
    stack: &ItemStack,
) -> i32 {
    match quick_craft_type {
        QUICK_CRAFT_TYPE_CHARITABLE => stack.count() / quick_craft_slots_size as i32,
        QUICK_CRAFT_TYPE_GREEDY => 1,
        QUICK_CRAFT_TYPE_CLONE => stack.max_stack_size() as i32,
        _ => stack.count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_craft_place_count_matches_java_modes() {
        let stack = ItemStack::new("minecraft:stick", 10);
        assert_eq!(quick_craft_place_count(3, 0, &stack), 3);
        assert_eq!(quick_craft_place_count(3, 1, &stack), 1);
        assert_eq!(quick_craft_place_count(3, 2, &stack), 64);
        assert_eq!(quick_craft_place_count(3, 99, &stack), 10);
    }

    #[test]
    fn quick_craft_tracks_start_continue_end_packets() {
        let mut menu = Menu::new(3);
        menu.carried = ItemStack::new("minecraft:stick", 9);

        assert_eq!(
            menu.click_quick_craft(-999, quick_craft_mask(0, 0)),
            InventoryAction::Noop
        );
        assert_eq!(
            menu.click_quick_craft(0, quick_craft_mask(1, 0)),
            InventoryAction::Noop
        );
        assert_eq!(
            menu.click_quick_craft(1, quick_craft_mask(1, 0)),
            InventoryAction::Noop
        );
        assert_eq!(
            menu.click_quick_craft(-999, quick_craft_mask(2, 0)),
            InventoryAction::QuickCrafted { slots: 2, each: 4 }
        );
        assert_eq!(menu.slots[0].stack.count(), 4);
        assert_eq!(menu.slots[1].stack.count(), 4);
        assert_eq!(menu.carried.count(), 1);
    }

    #[test]
    fn quick_craft_single_slot_falls_back_to_pickup_click() {
        let mut menu = Menu::new(1);
        menu.carried = ItemStack::new("minecraft:stick", 3);

        menu.click_quick_craft(-999, quick_craft_mask(0, 1));
        menu.click_quick_craft(0, quick_craft_mask(1, 1));
        assert_eq!(
            menu.click_quick_craft(-999, quick_craft_mask(2, 1)),
            InventoryAction::Placed { slot: 0, count: 1 }
        );
        assert_eq!(menu.slots[0].stack.count(), 1);
        assert_eq!(menu.carried.count(), 2);
    }

    #[test]
    fn quick_craft_clone_mode_requires_creative() {
        let mut menu = Menu::new(2);
        menu.carried = ItemStack::new("minecraft:stick", 1);
        menu.click_quick_craft(-999, quick_craft_mask(0, 2));
        menu.click_quick_craft(0, quick_craft_mask(1, 2));
        assert_eq!(
            menu.click_quick_craft(-999, quick_craft_mask(2, 2)),
            InventoryAction::Noop
        );

        menu.creative = true;
        menu.carried = ItemStack::new("minecraft:stick", 1);
        menu.click_quick_craft(-999, quick_craft_mask(0, 2));
        menu.click_quick_craft(0, quick_craft_mask(1, 2));
        menu.click_quick_craft(1, quick_craft_mask(1, 2));
        assert_eq!(
            menu.click_quick_craft(-999, quick_craft_mask(2, 2)),
            InventoryAction::QuickCrafted { slots: 2, each: 64 }
        );
        assert_eq!(menu.slots[0].stack.count(), 64);
        assert_eq!(menu.slots[1].stack.count(), 64);
    }
}
