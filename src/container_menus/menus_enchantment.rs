//! `EnchantmentMenu` — the enchanting table (offers, costs, and the enchant
//! action). Split out of `menus_workstation` to respect the 1200-line limit.

use super::*;

// ============================================================================
// EnchantmentMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentMenu {
    item: ItemStack,
    lapis: ItemStack,
    /// The required level for each of the 3 enchanting slots (`costs`); 0 = unavailable.
    pub costs: [i32; 3],
    /// The previewed enchantment id for each slot (`enchantClue`); `None` = unknown.
    pub enchant_clue: [Option<&'static str>; 3],
    /// The previewed enchantment level for each slot (`levelClue`); -1 = none.
    pub level_clue: [i32; 3],
    /// The per-table enchantment seed (`enchantmentSeed`).
    pub enchantment_seed: i32,
}

/// The outcome of `EnchantmentMenu::click_button`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnchantOutcome {
    /// The click was rejected (bad slot, insufficient lapis/levels, or no offer).
    Rejected,
    /// The item was enchanted; `xp_levels` is the `enchantmentCost` (slot+1) spent.
    Enchanted { xp_levels: i32 },
}

impl EnchantmentMenu {
    pub const ITEM_SLOT: usize = 0;
    pub const LAPIS_SLOT: usize = 1;
    pub const MENU_SLOTS: usize = 2;
    pub const INV_START: usize = 2;
    pub const HOTBAR_END: usize = 38;
    pub const SLOT_COUNT: usize = 38;

    pub fn new() -> Self {
        Self {
            item: ItemStack::empty(),
            lapis: ItemStack::empty(),
            costs: [0; 3],
            enchant_clue: [None; 3],
            level_clue: [-1; 3],
            enchantment_seed: 0,
        }
    }

    /// `ItemStack.isEnchantable`: has the `ENCHANTABLE` component and is not already
    /// enchanted (empty `ENCHANTMENTS`).
    fn is_enchantable(stack: &ItemStack) -> bool {
        stack.component("minecraft:enchantable").is_some()
            && stack.component("minecraft:enchantments").is_none()
    }

    fn enchantability(stack: &ItemStack) -> i32 {
        match stack.component("minecraft:enchantable") {
            Some(ItemComponent::Enchantable(value)) => *value as i32,
            _ => 0,
        }
    }

    /// `EnchantmentMenu.getEnchantmentList`: re-seed to `enchantmentSeed + slot`, run
    /// `selectEnchantment` over the enchanting-table set, and (for a plain book with
    /// more than one offer) drop one at random.
    fn enchantment_list(
        &self,
        random: &mut crate::random_source::LegacyRandom,
        slot: i32,
        cost: i32,
        enchantability: i32,
    ) -> Vec<(&'static str, i32)> {
        random.set_seed(i64::from(self.enchantment_seed) + i64::from(slot));
        let mut list = crate::enchantment_system::select_enchantment(
            random,
            self.item.item_id(),
            cost,
            enchantability,
        );
        if self.item.item_id() == "minecraft:book" && list.len() > 1 {
            let remove = random.next_i32_bound(list.len() as i32) as usize;
            list.remove(remove);
        }
        list
    }

    /// `EnchantmentMenu.slotsChanged`: recompute the 3 costs and preview clues from the
    /// item's enchantability and the number of nearby `bookcases` (supplied by the
    /// caller, since it depends on the world). Clears everything when the item is empty
    /// or not enchantable.
    pub fn slots_changed(&mut self, bookcases: i32) {
        if self.item.is_empty() || !Self::is_enchantable(&self.item) {
            self.costs = [0; 3];
            self.enchant_clue = [None; 3];
            self.level_clue = [-1; 3];
            return;
        }
        let enchantability = Self::enchantability(&self.item);
        let mut random = crate::random_source::LegacyRandom::new(i64::from(self.enchantment_seed));
        for slot in 0..3 {
            self.costs[slot] = crate::enchantment_system::get_enchantment_cost(
                &mut random,
                slot as i32,
                bookcases,
                enchantability,
            );
            self.enchant_clue[slot] = None;
            self.level_clue[slot] = -1;
            if self.costs[slot] < slot as i32 + 1 {
                self.costs[slot] = 0;
            }
        }
        for slot in 0..3 {
            if self.costs[slot] > 0 {
                let list = self.enchantment_list(
                    &mut random,
                    slot as i32,
                    self.costs[slot],
                    enchantability,
                );
                if !list.is_empty() {
                    let pick = list[random.next_i32_bound(list.len() as i32) as usize];
                    self.enchant_clue[slot] = Some(pick.0);
                    self.level_clue[slot] = pick.1;
                }
            }
        }
    }

    /// `EnchantmentMenu.clickMenuButton`: enchant the item from slot `button` (0..3).
    /// Requires `lapis >= button+1` and `xp_level >= max(button+1, costs[button])`
    /// unless `creative`. On success, applies the selected enchantments (transmuting a
    /// plain book to an enchanted book), consumes the lapis, and returns the XP spent.
    /// The caller is responsible for deducting the player's XP and re-rolling the seed.
    pub fn click_button(&mut self, button: usize, xp_level: i32, creative: bool) -> EnchantOutcome {
        if button >= 3 {
            return EnchantOutcome::Rejected;
        }
        let xp_levels = button as i32 + 1;
        if (self.lapis.is_empty() || self.lapis.count() < xp_levels) && !creative {
            return EnchantOutcome::Rejected;
        }
        if self.costs[button] <= 0
            || self.item.is_empty()
            || ((xp_level < xp_levels || xp_level < self.costs[button]) && !creative)
        {
            return EnchantOutcome::Rejected;
        }
        let enchantability = Self::enchantability(&self.item);
        let mut random = crate::random_source::LegacyRandom::new(i64::from(self.enchantment_seed));
        let list = self.enchantment_list(
            &mut random,
            button as i32,
            self.costs[button],
            enchantability,
        );
        if list.is_empty() {
            return EnchantOutcome::Rejected;
        }
        if self.item.item_id() == "minecraft:book" {
            self.item = self
                .item
                .transmute_copy("minecraft:enchanted_book", self.item.count());
        }
        let mut enchantments = enchantments_for_crafting(&self.item);
        for (id, level) in &list {
            let existing = enchantments.get(*id).copied().unwrap_or(0);
            enchantments.insert((*id).to_string(), (*level).max(existing));
        }
        set_enchantments_for_crafting(&mut self.item, enchantments);
        if !creative {
            self.lapis.shrink(xp_levels);
        }
        EnchantOutcome::Enchanted { xp_levels }
    }

    /// The previewed item (after `set_slot` / `click_button`).
    pub fn item(&self) -> &ItemStack {
        &self.item
    }

    /// `EnchantmentMenu.removed`: `clearContainer(enchantSlots)` — the item and lapis
    /// return to the player.
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
        for slot in [&mut self.item, &mut self.lapis] {
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
            0 => self.item.clone(),
            1 => self.lapis.clone(),
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
            0 => {
                let mut s = stack;
                s.limit_size(1);
                self.item = s;
            }
            1 => self.lapis = stack,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => slot < Self::SLOT_COUNT,
            1 => stack.is_empty() || stack.item_id() == "minecraft:lapis_lazuli",
            _ => slot < Self::SLOT_COUNT,
        }
    }

    pub fn max_stack_size(&self, slot: usize) -> i32 {
        if slot == 0 {
            1
        } else {
            64
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.item.clone());
        out.push(self.lapis.clone());
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
        let moved = match slot {
            0 | 1 => {
                self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
            }
            _ if moving.item_id() == "minecraft:lapis_lazuli" => {
                self.move_into_range(&mut moving, 1, 2, true, player)
            }
            _ => {
                if self.item.is_empty() && self.may_place(0, &moving) {
                    let one = moving.copy_with_count(1);
                    moving.shrink(1);
                    self.item = one;
                    true
                } else {
                    false
                }
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
                0 => self.item = w.new_stack,
                1 => self.lapis = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for EnchantmentMenu {
    fn default() -> Self {
        Self::new()
    }
}
