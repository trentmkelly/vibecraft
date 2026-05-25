use super::*;

// ============================================================================
// HorseInventoryMenu (uses AbstractMountInventoryMenu layout)
// ============================================================================

/// Saddle / armor / mount-chest inventory menu used by horse, llama, etc.
/// Slot 0 = saddle (only saddle item), slot 1 = body armor (only horse armor
/// or carpet for llamas).
#[derive(Debug, Clone, PartialEq)]
pub struct HorseInventoryMenu {
    saddle: ItemStack,
    armor: ItemStack,
    chest_slots: Vec<ItemStack>,
    layout: HorseLayout,
}

/// Geometry describing whether saddle/armor are active and how many chest
/// slots are present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorseLayout {
    pub saddle_active: bool,
    pub armor_active: bool,
    pub is_llama: bool,
    pub inventory_columns: usize,
}

impl HorseLayout {
    pub fn chest_size(&self) -> usize {
        self.inventory_columns * 3
    }

    pub fn player_start(&self) -> usize {
        2 + self.chest_size()
    }

    pub fn slot_count(&self) -> usize {
        self.player_start() + PLAYER_SLOTS
    }
}

impl HorseInventoryMenu {
    pub const SLOT_SADDLE: usize = 0;
    pub const SLOT_ARMOR: usize = 1;
    pub const SLOT_INVENTORY_START: usize = 2;

    pub fn new(layout: HorseLayout) -> Self {
        Self {
            saddle: ItemStack::empty(),
            armor: ItemStack::empty(),
            chest_slots: vec![ItemStack::empty(); layout.chest_size()],
            layout,
        }
    }

    pub fn layout(&self) -> &HorseLayout {
        &self.layout
    }

    pub fn slot_count(&self) -> usize {
        self.layout.slot_count()
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= self.slot_count() {
            return None;
        }
        Some(match slot {
            0 => self.saddle.clone(),
            1 => self.armor.clone(),
            s if s < self.layout.player_start() => self.chest_slots[s - 2].clone(),
            s => read_player_slot(s, self.layout.player_start(), player),
        })
    }

    pub fn set_slot(
        &mut self,
        slot: usize,
        stack: ItemStack,
        player: &mut PlayerInventory,
    ) -> bool {
        if slot >= self.slot_count() {
            return false;
        }
        match slot {
            0 => self.saddle = stack,
            1 => self.armor = stack,
            s if s < self.layout.player_start() => self.chest_slots[s - 2] = stack,
            s => write_player_slot(s, self.layout.player_start(), player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => {
                stack.is_empty()
                    || (self.layout.saddle_active && stack.item_id() == "minecraft:saddle")
            }
            1 => {
                stack.is_empty()
                    || (self.layout.armor_active
                        && Self::is_horse_armor(stack.item_id(), self.layout.is_llama))
            }
            s if s < self.slot_count() => true,
            _ => false,
        }
    }

    fn is_horse_armor(item_id: &str, is_llama: bool) -> bool {
        if is_llama {
            return item_id.ends_with("_carpet");
        }
        matches!(
            item_id,
            "minecraft:leather_horse_armor"
                | "minecraft:copper_horse_armor"
                | "minecraft:iron_horse_armor"
                | "minecraft:golden_horse_armor"
                | "minecraft:diamond_horse_armor"
                | "minecraft:netherite_horse_armor"
        )
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(self.slot_count());
        out.push(self.saddle.clone());
        out.push(self.armor.clone());
        for c in &self.chest_slots {
            out.push(c.clone());
        }
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(&mut self, slot: usize, player: &mut PlayerInventory) -> ItemStack {
        if slot >= self.slot_count() {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);

        let player_start = self.layout.player_start();
        let total = self.slot_count();
        let chest_size = self.layout.chest_size();

        let moved = if slot < player_start {
            // Saddle / armor / chest → player inventory.
            self.move_into_range(&mut moving, player_start, total, true, player)
        } else if self.may_place(1, &moving) && self.armor.is_empty() {
            self.move_into_range(&mut moving, 1, 2, false, player)
        } else if self.may_place(0, &moving) && self.saddle.is_empty() {
            self.move_into_range(&mut moving, 0, 1, false, player)
        } else if chest_size > 0
            && self.move_into_range(&mut moving, 2, player_start, false, player)
        {
            true
        } else {
            // Bounce between main storage and hotbar.
            let main_end = player_start + PLAYER_MAIN_STORAGE;
            if slot >= main_end {
                self.move_into_range(&mut moving, player_start, main_end, false, player)
            } else {
                self.move_into_range(&mut moving, main_end, total, false, player)
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
        let player_start = self.layout.player_start();
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            match w.slot {
                0 => self.saddle = w.new_stack,
                1 => self.armor = w.new_stack,
                s if s < player_start => self.chest_slots[s - 2] = w.new_stack,
                s => write_player_slot(s, player_start, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

// ============================================================================
// NautilusInventoryMenu (saddle + body armor; no chest slots)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct NautilusInventoryMenu {
    saddle: ItemStack,
    armor: ItemStack,
}

impl NautilusInventoryMenu {
    pub const SLOT_SADDLE: usize = 0;
    pub const SLOT_ARMOR: usize = 1;
    pub const MENU_SLOTS: usize = 2;
    pub const INV_START: usize = 2;
    pub const HOTBAR_END: usize = 38;
    pub const SLOT_COUNT: usize = 38;

    pub fn new() -> Self {
        Self {
            saddle: ItemStack::empty(),
            armor: ItemStack::empty(),
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.saddle.clone(),
            1 => self.armor.clone(),
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
            0 => self.saddle = stack,
            1 => self.armor = stack,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => stack.is_empty() || stack.item_id() == "minecraft:saddle",
            1 => stack.is_empty() || Self::is_nautilus_armor(stack.item_id()),
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    fn is_nautilus_armor(item_id: &str) -> bool {
        matches!(
            item_id,
            "minecraft:copper_nautilus_armor"
                | "minecraft:iron_nautilus_armor"
                | "minecraft:golden_nautilus_armor"
                | "minecraft:diamond_nautilus_armor"
                | "minecraft:netherite_nautilus_armor"
        )
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.saddle.clone());
        out.push(self.armor.clone());
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
        let moved = if slot < Self::INV_START {
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else if self.may_place(1, &moving) && self.armor.is_empty() {
            self.move_into_range(&mut moving, 1, 2, false, player)
        } else if self.may_place(0, &moving) && self.saddle.is_empty() {
            self.move_into_range(&mut moving, 0, 1, false, player)
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
                0 => self.saddle = w.new_stack,
                1 => self.armor = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for NautilusInventoryMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// MerchantMenu
// ============================================================================

/// Merchant trading window. Slot layout:
///   * `0` — payment A
///   * `1` — payment B
///   * `2` — result (filled when active offer matches payment)
///   * `3..30` — player main storage
///   * `30..39` — hotbar
///
/// The actual merchant offer logic lives in `MerchantContainer` in
/// `player_inventory.rs`. This struct owns just the slot view + quick-move.
#[derive(Debug, Clone, PartialEq)]
pub struct MerchantMenu {
    payment_a: ItemStack,
    payment_b: ItemStack,
    result: ItemStack,
}

impl MerchantMenu {
    pub const PAYMENT1_SLOT: usize = 0;
    pub const PAYMENT2_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;

    pub fn new() -> Self {
        Self {
            payment_a: ItemStack::empty(),
            payment_b: ItemStack::empty(),
            result: ItemStack::empty(),
        }
    }

    pub fn payment(&self, index: usize) -> Option<&ItemStack> {
        match index {
            0 => Some(&self.payment_a),
            1 => Some(&self.payment_b),
            _ => None,
        }
    }

    pub fn set_result_internal(&mut self, stack: ItemStack) {
        self.result = stack;
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.payment_a.clone(),
            1 => self.payment_b.clone(),
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
            0 => self.payment_a = stack,
            1 => self.payment_b = stack,
            2 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            2 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.payment_a.clone());
        out.push(self.payment_b.clone());
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
                if slot < Self::INV_START + PLAYER_MAIN_STORAGE {
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
                0 => self.payment_a = w.new_stack,
                1 => self.payment_b = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }

    /// Apply a `SelectTradePacket` for the given offer index.
    ///
    /// Java parity: `MerchantMenu.tryMoveItems(newTradeIndex)`. Out-of-bounds
    /// indices are ignored. Items currently in the payment slots are pushed
    /// back into the player's inventory (preferring the hotbar end, matching
    /// vanilla's `reverseDirection = true` for `moveItemStackTo(3, 39, true)`)
    /// before refilling the payment slots from the player's inventory using
    /// the new offer's cost items.
    pub fn try_move_items(
        &mut self,
        new_trade_index: usize,
        offers: &[MerchantOffer],
        player: &mut PlayerInventory,
    ) {
        if new_trade_index >= offers.len() {
            return;
        }

        // Push payment slot A back into the player's inventory. If we cannot
        // move all of it we abort the whole operation — vanilla bails out
        // entirely so the original payment stays in the slot.
        let mut old_cost_a = self.payment_a.clone();
        if !old_cost_a.is_empty() {
            self.payment_a = ItemStack::empty();
            let moved = self.move_into_range(
                &mut old_cost_a,
                Self::INV_START,
                Self::HOTBAR_END,
                true,
                player,
            );
            if !moved && !old_cost_a.is_empty() {
                // Restore and abort.
                self.payment_a = old_cost_a;
                return;
            }
            // Java sets `tradeContainer.setItem(0, oldCostA)` with whatever
            // remains; this preserves residue if the inventory was nearly
            // full.
            self.payment_a = old_cost_a;
        }

        let mut old_cost_b = self.payment_b.clone();
        if !old_cost_b.is_empty() {
            self.payment_b = ItemStack::empty();
            let moved = self.move_into_range(
                &mut old_cost_b,
                Self::INV_START,
                Self::HOTBAR_END,
                true,
                player,
            );
            if !moved && !old_cost_b.is_empty() {
                self.payment_b = old_cost_b;
                return;
            }
            self.payment_b = old_cost_b;
        }

        // Only auto-fill if both payment slots are empty after the push-back.
        if self.payment_a.is_empty() && self.payment_b.is_empty() {
            let offer = &offers[new_trade_index];
            self.move_from_inventory_to_payment_slot(0, &offer.base_cost_a, player);
            if let Some(cost_b) = &offer.cost_b {
                self.move_from_inventory_to_payment_slot(1, cost_b, player);
            }
        }
    }

    /// Java: `MerchantMenu.moveFromInventoryToPaymentSlot(paymentSlot, cost)`.
    ///
    /// Walks the player's storage + hotbar slots (`3..39` in menu space) in
    /// order, moving stacks that match `cost` into the indicated payment slot
    /// until either the inventory is exhausted or the payment slot reaches a
    /// full stack. We match on item id only, mirroring the current
    /// `ItemCost::matches` predicate (components are not modelled yet).
    fn move_from_inventory_to_payment_slot(
        &mut self,
        payment_slot: usize,
        cost: &ItemCost,
        player: &mut PlayerInventory,
    ) {
        for menu_slot in Self::INV_START..Self::HOTBAR_END {
            let inventory_item = read_player_slot(menu_slot, Self::INV_START, player);
            if inventory_item.is_empty() || !cost.matches(&inventory_item) {
                continue;
            }
            let current_payment = match payment_slot {
                0 => self.payment_a.clone(),
                1 => self.payment_b.clone(),
                _ => return,
            };
            if !current_payment.is_empty()
                && !same_item_same_components(&inventory_item, &current_payment)
            {
                continue;
            }
            let max_stack_size = inventory_item.max_stack_size() as i32;
            let move_count = (max_stack_size - current_payment.count()).min(inventory_item.count());
            if move_count <= 0 {
                // Payment slot already full — nothing left to merge into it.
                break;
            }
            let new_payment_count = current_payment.count() + move_count;
            let new_payment = inventory_item.copy_with_count(new_payment_count);

            // Shrink the source inventory slot by exactly the moved amount.
            let mut updated_source = inventory_item.clone();
            updated_source.shrink(move_count);
            write_player_slot(menu_slot, Self::INV_START, player, updated_source);

            match payment_slot {
                0 => self.payment_a = new_payment.clone(),
                1 => self.payment_b = new_payment.clone(),
                _ => unreachable!(),
            }

            if new_payment_count >= max_stack_size {
                break;
            }
        }
    }
}

impl Default for MerchantMenu {
    fn default() -> Self {
        Self::new()
    }
}
