use super::*;

// ============================================================================
// BrewingStandMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct BrewingStandMenu {
    potions: [ItemStack; 3],
    ingredient: ItemStack,
    fuel: ItemStack,
    /// `[brewTime, fuelLevel]`.
    pub data: [i32; 2],
}

impl BrewingStandMenu {
    pub const POTION0_SLOT: usize = 0;
    pub const POTION1_SLOT: usize = 1;
    pub const POTION2_SLOT: usize = 2;
    pub const INGREDIENT_SLOT: usize = 3;
    pub const FUEL_SLOT: usize = 4;
    pub const MENU_SLOTS: usize = 5;
    pub const DATA_COUNT: usize = 2;
    pub const BREW_TIME_DATA: usize = 0;
    pub const FUEL_LEVEL_DATA: usize = 1;
    pub const INV_START: usize = 5;
    pub const HOTBAR_END: usize = 41;
    pub const SLOT_COUNT: usize = 41;

    pub fn new() -> Self {
        Self {
            potions: std::array::from_fn(|_| ItemStack::empty()),
            ingredient: ItemStack::empty(),
            fuel: ItemStack::empty(),
            data: [0, 0],
        }
    }

    /// `BrewingStandMenu` has no `removed()` override in Java — the brewing stand is
    /// a placed block entity whose container persists across closes, so only the base
    /// `AbstractContainerMenu.removed` runs (routing the carried cursor item).
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
    }

    pub fn data(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }

    pub fn set_data(&mut self, index: usize, value: i32) -> bool {
        let Some(slot) = self.data.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }

    pub fn get_brewing_ticks(&self) -> i32 {
        self.data[Self::BREW_TIME_DATA]
    }

    pub fn get_fuel(&self) -> i32 {
        self.data[Self::FUEL_LEVEL_DATA]
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0..=2 => self.potions[slot].clone(),
            3 => self.ingredient.clone(),
            4 => self.fuel.clone(),
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
            0..=2 => {
                let mut s = stack;
                s.limit_size(1);
                self.potions[slot] = s;
            }
            3 => self.ingredient = stack,
            4 => self.fuel = stack,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0..=2 => !stack.is_empty() && is_potion_or_bottle(stack.item_id()),
            3 => !stack.is_empty() && is_brewing_ingredient(stack.item_id()),
            4 => !stack.is_empty() && BREWING_FUEL_ITEMS.contains(&stack.item_id()),
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn max_stack_size(&self, slot: usize) -> i32 {
        if slot <= 2 {
            1
        } else {
            64
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for p in &self.potions {
            out.push(p.clone());
        }
        out.push(self.ingredient.clone());
        out.push(self.fuel.clone());
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
        self.set_slot(slot, ItemStack::empty(), player);

        let moved = if slot <= 4 {
            // Brewing slot → player inventory.
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else {
            // Player → brewing slots in fuel/ingredient/potion order.
            if BREWING_FUEL_ITEMS.contains(&moving.item_id()) {
                if self.move_into_range(&mut moving, 4, 5, false, player) {
                    true
                } else if is_brewing_ingredient(moving.item_id()) {
                    self.move_into_range(&mut moving, 3, 4, false, player)
                } else {
                    false
                }
            } else if is_brewing_ingredient(moving.item_id()) {
                self.move_into_range(&mut moving, 3, 4, false, player)
            } else if is_potion_or_bottle(moving.item_id()) {
                self.move_into_range(&mut moving, 0, 3, false, player)
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
        };
        if !moving.is_empty() {
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
                0..=2 => self.potions[w.slot] = w.new_stack,
                3 => self.ingredient = w.new_stack,
                4 => self.fuel = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for BrewingStandMenu {
    fn default() -> Self {
        Self::new()
    }
}
