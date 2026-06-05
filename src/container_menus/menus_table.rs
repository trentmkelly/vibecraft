use super::*;

// ============================================================================
// CartographyTableMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CartographyTableMenu {
    map: ItemStack,
    additional: ItemStack,
    result: ItemStack,
}

impl CartographyTableMenu {
    pub const MAP_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;

    pub fn new() -> Self {
        Self {
            map: ItemStack::empty(),
            additional: ItemStack::empty(),
            result: ItemStack::empty(),
        }
    }

    /// `CartographyTableMenu.removed`: discard the (virtual) result slot, then
    /// `clearContainer(container)` — the map and paper/additional return to the player.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        self.result = ItemStack::empty();
        for slot in [&mut self.map, &mut self.additional] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.map.clone(),
            1 => self.additional.clone(),
            2 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
        })
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        match slot {
            0 => self.map = stack,
            1 => self.additional = stack,
            2 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => stack.is_empty() || is_filled_map(stack.item_id()),
            1 => stack.is_empty() || is_cartography_additional(stack.item_id()),
            2 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.map.clone());
        out.push(self.additional.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        if slot == 2 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            2 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0 | 1 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => {
                if is_filled_map(moving.item_id()) {
                    self.move_into_range(&mut moving, 0, 1, false, player)
                } else if is_cartography_additional(moving.item_id()) {
                    self.move_into_range(&mut moving, 1, 2, false, player)
                } else if slot < Self::INV_START + PLAYER_MAIN_STORAGE {
                    self.move_into_range(
                        &mut moving,
                        Self::INV_START + PLAYER_MAIN_STORAGE,
                        Self::HOTBAR_END,
                        false,
                        player,
                    )
                } else {
                    self.move_into_range(
                        &mut moving,
                        Self::INV_START,
                        Self::INV_START + PLAYER_MAIN_STORAGE,
                        false,
                        player,
                    )
                }
            }
        };
        if !moving.is_empty() && slot != 2 {
            self.set_slot(slot, moving, player);
        }
        if !moved {
            return ItemStack::empty();
        }
        original
    }

    fn move_into_range(
        &mut self,
        stack: &mut ItemStack,
        start: usize,
        end: usize,
        reverse: bool,
        player: &mut PlayerInventory,
    ) -> bool {
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            match w.slot {
                0 => self.map = w.new_stack,
                1 => self.additional = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for CartographyTableMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// LoomMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct LoomMenu {
    banner: ItemStack,
    dye: ItemStack,
    pattern: ItemStack,
    result: ItemStack,
    pub selected_pattern_index: i32,
}

impl LoomMenu {
    pub const BANNER_SLOT: usize = 0;
    pub const DYE_SLOT: usize = 1;
    pub const PATTERN_SLOT: usize = 2;
    pub const RESULT_SLOT: usize = 3;
    pub const MENU_SLOTS: usize = 4;
    pub const INV_START: usize = 4;
    pub const HOTBAR_END: usize = 40;
    pub const SLOT_COUNT: usize = 40;

    pub fn new() -> Self {
        Self {
            banner: ItemStack::empty(),
            dye: ItemStack::empty(),
            pattern: ItemStack::empty(),
            result: ItemStack::empty(),
            selected_pattern_index: -1,
        }
    }

    /// `LoomMenu.removed`: `clearContainer(inputContainer)` — banner, dye, and pattern
    /// return to the player. The result is computed virtually and not returned.
    pub fn removed(
        &mut self,
        player: &mut PlayerInventory,
        carried: &mut ItemStack,
        disconnected: bool,
    ) {
        drop_or_place_in_inventory(
            player,
            std::mem::replace(carried, ItemStack::empty()),
            disconnected,
        );
        for slot in [&mut self.banner, &mut self.dye, &mut self.pattern] {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
        self.result = ItemStack::empty();
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.banner.clone(),
            1 => self.dye.clone(),
            2 => self.pattern.clone(),
            3 => self.result.clone(),
            _ => read_player_slot(slot, Self::INV_START, player),
        })
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        match slot {
            0 => self.banner = stack,
            1 => self.dye = stack,
            2 => self.pattern = stack,
            3 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => stack.is_empty() || is_banner_item(stack.item_id()),
            1 => stack.is_empty() || is_dye_item(stack.item_id()),
            2 => stack.is_empty() || is_pattern_item(stack.item_id()),
            3 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.banner.clone());
        out.push(self.dye.clone());
        out.push(self.pattern.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        if slot == 3 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            3 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0..=2 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => {
                if is_banner_item(moving.item_id()) {
                    self.move_into_range(&mut moving, 0, 1, false, player)
                } else if is_dye_item(moving.item_id()) {
                    self.move_into_range(&mut moving, 1, 2, false, player)
                } else if is_pattern_item(moving.item_id()) {
                    self.move_into_range(&mut moving, 2, 3, false, player)
                } else if slot < Self::INV_START + PLAYER_MAIN_STORAGE {
                    self.move_into_range(
                        &mut moving,
                        Self::INV_START + PLAYER_MAIN_STORAGE,
                        Self::HOTBAR_END,
                        false,
                        player,
                    )
                } else {
                    self.move_into_range(
                        &mut moving,
                        Self::INV_START,
                        Self::INV_START + PLAYER_MAIN_STORAGE,
                        false,
                        player,
                    )
                }
            }
        };
        if !moving.is_empty() && slot != 3 {
            self.set_slot(slot, moving, player);
        }
        if !moved {
            return ItemStack::empty();
        }
        original
    }

    fn move_into_range(
        &mut self,
        stack: &mut ItemStack,
        start: usize,
        end: usize,
        reverse: bool,
        player: &mut PlayerInventory,
    ) -> bool {
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            match w.slot {
                0 => self.banner = w.new_stack,
                1 => self.dye = w.new_stack,
                2 => self.pattern = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for LoomMenu {
    fn default() -> Self {
        Self::new()
    }
}

