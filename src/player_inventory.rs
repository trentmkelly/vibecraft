#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::inventory::same_item_same_components;
use crate::item_stack::ItemStack;
use crate::recipe_system::{CraftingStack, IngredientSpec, RecipeKind, RecipeMap};

pub const INVENTORY_SIZE: usize = 36;
pub const HOTBAR_SIZE: usize = 9;
pub const SLOT_OFFHAND: usize = 40;
pub const SLOT_BODY_ARMOR: usize = 41;
pub const SLOT_SADDLE: usize = 42;
pub const ENDER_CHEST_SIZE: usize = 27;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentSlot {
    Feet,
    Legs,
    Chest,
    Head,
    Offhand,
    Body,
    Saddle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInventory {
    items: Vec<ItemStack>,
    feet: ItemStack,
    legs: ItemStack,
    chest: ItemStack,
    head: ItemStack,
    offhand: ItemStack,
    body: ItemStack,
    saddle: ItemStack,
    selected: usize,
    times_changed: u32,
    dropped: Vec<ItemStack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryAddResult {
    FullyAdded,
    PartiallyAdded { remaining: i32 },
    Dropped { count: i32 },
    Rejected,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            items: vec![ItemStack::empty(); INVENTORY_SIZE],
            feet: ItemStack::empty(),
            legs: ItemStack::empty(),
            chest: ItemStack::empty(),
            head: ItemStack::empty(),
            offhand: ItemStack::empty(),
            body: ItemStack::empty(),
            saddle: ItemStack::empty(),
            selected: 0,
            times_changed: 0,
            dropped: Vec::new(),
        }
    }

    pub fn container_size(&self) -> usize {
        INVENTORY_SIZE + 7
    }

    pub fn selected_slot(&self) -> usize {
        self.selected
    }

    pub fn set_selected_slot(&mut self, slot: usize) -> Result<(), &'static str> {
        if slot < HOTBAR_SIZE {
            self.selected = slot;
            Ok(())
        } else {
            Err("selected slot must be in the hotbar")
        }
    }

    pub fn get(&self, slot: usize) -> &ItemStack {
        match Self::equipment_slot(slot) {
            Some(EquipmentSlot::Feet) => &self.feet,
            Some(EquipmentSlot::Legs) => &self.legs,
            Some(EquipmentSlot::Chest) => &self.chest,
            Some(EquipmentSlot::Head) => &self.head,
            Some(EquipmentSlot::Offhand) => &self.offhand,
            Some(EquipmentSlot::Body) => &self.body,
            Some(EquipmentSlot::Saddle) => &self.saddle,
            None if slot < self.items.len() => &self.items[slot],
            None => panic!("invalid player inventory slot {slot}"),
        }
    }

    pub fn set(&mut self, slot: usize, stack: ItemStack) {
        match Self::equipment_slot(slot) {
            Some(EquipmentSlot::Feet) => self.feet = stack,
            Some(EquipmentSlot::Legs) => self.legs = stack,
            Some(EquipmentSlot::Chest) => self.chest = stack,
            Some(EquipmentSlot::Head) => self.head = stack,
            Some(EquipmentSlot::Offhand) => self.offhand = stack,
            Some(EquipmentSlot::Body) => self.body = stack,
            Some(EquipmentSlot::Saddle) => self.saddle = stack,
            None if slot < self.items.len() => self.items[slot] = stack,
            None => return,
        }
        self.set_changed();
    }

    pub fn remove(&mut self, slot: usize, count: i32) -> ItemStack {
        let removed = if let Some(stack) = self.stack_mut(slot) {
            stack.split(count)
        } else {
            ItemStack::empty()
        };
        if !removed.is_empty() {
            self.set_changed();
        }
        removed
    }

    pub fn remove_no_update(&mut self, slot: usize) -> ItemStack {
        let Some(stack) = self.stack_mut(slot) else {
            return ItemStack::empty();
        };
        std::mem::replace(stack, ItemStack::empty())
    }

    pub fn get_free_slot(&self) -> Option<usize> {
        self.items.iter().position(ItemStack::is_empty)
    }

    pub fn get_slot_with_remaining_space(&self, stack: &ItemStack) -> Option<usize> {
        [self.selected, SLOT_OFFHAND]
            .into_iter()
            .chain(0..self.items.len())
            .find(|&slot| self.has_remaining_space_for_item(slot, stack))
    }

    pub fn add(&mut self, mut stack: ItemStack) -> InventoryAddResult {
        if stack.is_empty() {
            return InventoryAddResult::Rejected;
        }

        while !stack.is_empty() {
            let before = stack.count();
            let slot = self
                .get_slot_with_remaining_space(&stack)
                .or_else(|| self.get_free_slot());
            let Some(slot) = slot else {
                return if stack.count() == before {
                    InventoryAddResult::PartiallyAdded {
                        remaining: stack.count(),
                    }
                } else {
                    InventoryAddResult::FullyAdded
                };
            };
            self.add_resource(slot, &mut stack);
            if stack.count() == before {
                break;
            }
        }

        if stack.is_empty() {
            InventoryAddResult::FullyAdded
        } else {
            InventoryAddResult::PartiallyAdded {
                remaining: stack.count(),
            }
        }
    }

    pub fn place_item_back_in_inventory(&mut self, mut stack: ItemStack) -> InventoryAddResult {
        while !stack.is_empty() {
            let slot = self
                .get_slot_with_remaining_space(&stack)
                .or_else(|| self.get_free_slot());
            let Some(slot) = slot else {
                let count = stack.count();
                self.dropped.push(stack);
                return InventoryAddResult::Dropped { count };
            };
            let room = (stack.max_stack_size() as i32 - self.get(slot).count()).max(1);
            let mut split = stack.split(room);
            self.add_resource(slot, &mut split);
        }
        InventoryAddResult::FullyAdded
    }

    pub fn pick_slot(&mut self, slot: usize) {
        let selected = self.get_suitable_hotbar_slot();
        self.selected = selected;
        if slot < self.items.len() {
            self.items.swap(selected, slot);
            self.set_changed();
        }
    }

    pub fn add_and_pick_item(&mut self, stack: ItemStack) {
        let selected = self.get_suitable_hotbar_slot();
        self.selected = selected;
        if !self.items[selected].is_empty() {
            if let Some(free) = self.get_free_slot() {
                self.items[free] = std::mem::replace(&mut self.items[selected], ItemStack::empty());
            }
        }
        self.items[selected] = stack;
        self.set_changed();
    }

    pub fn get_suitable_hotbar_slot(&self) -> usize {
        for offset in 0..HOTBAR_SIZE {
            let index = (self.selected + offset) % HOTBAR_SIZE;
            if self.items[index].is_empty() {
                return index;
            }
        }
        self.selected
    }

    pub fn clear_or_count_matching_items<F>(&mut self, predicate: F, amount: i32) -> i32
    where
        F: Fn(&ItemStack) -> bool,
    {
        let counting_only = amount == 0;
        let mut seen = 0;
        for slot in 0..self.items.len() {
            if !predicate(&self.items[slot]) {
                continue;
            }
            if counting_only {
                seen += self.items[slot].count();
                continue;
            }
            let removed = (amount - seen).min(self.items[slot].count());
            self.items[slot].shrink(removed);
            seen += removed;
            if seen >= amount {
                break;
            }
        }
        if !counting_only && seen > 0 {
            self.set_changed();
        }
        seen
    }

    pub fn saved_items(&self) -> Vec<(usize, ItemStack)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, stack)| !stack.is_empty())
            .map(|(slot, stack)| (slot, stack.clone()))
            .collect()
    }

    pub fn load_items(&mut self, input: &[(usize, ItemStack)]) {
        self.items = vec![ItemStack::empty(); INVENTORY_SIZE];
        for (slot, stack) in input {
            if *slot < INVENTORY_SIZE {
                self.items[*slot] = stack.clone();
            }
        }
        self.set_changed();
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(ItemStack::is_empty)
            && [
                &self.feet,
                &self.legs,
                &self.chest,
                &self.head,
                &self.offhand,
                &self.body,
                &self.saddle,
            ]
            .iter()
            .all(|stack| stack.is_empty())
    }

    pub fn times_changed(&self) -> u32 {
        self.times_changed
    }

    pub fn dropped(&self) -> &[ItemStack] {
        &self.dropped
    }

    pub fn equipment_slot(slot: usize) -> Option<EquipmentSlot> {
        match slot {
            36 => Some(EquipmentSlot::Feet),
            37 => Some(EquipmentSlot::Legs),
            38 => Some(EquipmentSlot::Chest),
            39 => Some(EquipmentSlot::Head),
            SLOT_OFFHAND => Some(EquipmentSlot::Offhand),
            SLOT_BODY_ARMOR => Some(EquipmentSlot::Body),
            SLOT_SADDLE => Some(EquipmentSlot::Saddle),
            _ => None,
        }
    }

    fn has_remaining_space_for_item(&self, slot: usize, stack: &ItemStack) -> bool {
        let current = self.get(slot);
        !current.is_empty()
            && same_item_same_components(current, stack)
            && current.max_stack_size() > 1
            && current.count() < current.max_stack_size() as i32
    }

    fn add_resource(&mut self, slot: usize, stack: &mut ItemStack) {
        if stack.is_empty() {
            return;
        }
        if self.get(slot).is_empty() {
            self.set(slot, stack.copy_with_count(0));
        }
        let current = self.stack_mut(slot).expect("slot was just initialized");
        let room = current.max_stack_size() as i32 - current.count();
        let moved = stack.count().min(room.max(0));
        current.grow(moved);
        current.set_pop_time(5);
        stack.shrink(moved);
        if moved > 0 {
            self.set_changed();
        }
    }

    fn stack_mut(&mut self, slot: usize) -> Option<&mut ItemStack> {
        match Self::equipment_slot(slot) {
            Some(EquipmentSlot::Feet) => Some(&mut self.feet),
            Some(EquipmentSlot::Legs) => Some(&mut self.legs),
            Some(EquipmentSlot::Chest) => Some(&mut self.chest),
            Some(EquipmentSlot::Head) => Some(&mut self.head),
            Some(EquipmentSlot::Offhand) => Some(&mut self.offhand),
            Some(EquipmentSlot::Body) => Some(&mut self.body),
            Some(EquipmentSlot::Saddle) => Some(&mut self.saddle),
            None if slot < self.items.len() => Some(&mut self.items[slot]),
            None => None,
        }
    }

    /// Compute items to drop and destroy on player death, matching Java
    /// `Player.dropEquipment` + `destroyVanishingCursedItems` + `Inventory.dropAll`.
    ///
    /// * `keep_inventory` — `keepInventory` gamerule; when true nothing is dropped.
    /// * `has_prevent_equipment_drop` — returns true if the given item stack has the
    ///   Curse of Vanishing (`PREVENT_EQUIPMENT_DROP`) enchantment; those items are
    ///   destroyed (not returned).
    ///
    /// Returns the items to be spawned as item entities.
    pub fn death_drops<F>(
        &mut self,
        keep_inventory: bool,
        has_prevent_equipment_drop: F,
    ) -> Vec<ItemStack>
    where
        F: Fn(&ItemStack) -> bool,
    {
        if keep_inventory {
            return Vec::new();
        }
        let total = self.container_size();
        let mut drops = Vec::new();
        for slot in 0..total {
            let stack = if let Some(s) = self.stack_mut(slot) {
                s
            } else {
                continue;
            };
            if stack.is_empty() {
                continue;
            }
            if has_prevent_equipment_drop(stack) {
                *stack = ItemStack::empty();
            } else {
                drops.push(std::mem::replace(stack, ItemStack::empty()));
            }
            self.times_changed += 1;
        }
        drops
    }

    fn set_changed(&mut self) {
        self.times_changed += 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleContainer {
    slots: Vec<ItemStack>,
}

impl SimpleContainer {
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![ItemStack::empty(); size],
        }
    }

    pub fn ender_chest() -> Self {
        Self::new(ENDER_CHEST_SIZE)
    }

    pub fn size(&self) -> usize {
        self.slots.len()
    }

    pub fn get(&self, slot: usize) -> &ItemStack {
        self.slots
            .get(slot)
            .unwrap_or_else(|| panic!("invalid container slot {slot}"))
    }

    pub fn set(&mut self, slot: usize, stack: ItemStack) {
        if let Some(target) = self.slots.get_mut(slot) {
            *target = stack;
            target.limit_size(64);
        }
    }

    pub fn remove(&mut self, slot: usize, count: i32) -> ItemStack {
        self.slots
            .get_mut(slot)
            .map(|stack| stack.split(count))
            .unwrap_or_else(ItemStack::empty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HorseInventoryLayout {
    pub saddle_slot_active: bool,
    pub armor_slot_active: bool,
    pub inventory_columns: usize,
    pub chest_slots: usize,
    pub player_inventory_start: usize,
}

impl HorseInventoryLayout {
    pub fn new(
        can_saddle: bool,
        can_body_armor: bool,
        is_llama: bool,
        inventory_columns: usize,
    ) -> Self {
        Self {
            saddle_slot_active: can_saddle,
            armor_slot_active: can_body_armor || is_llama,
            inventory_columns,
            chest_slots: inventory_columns * 3,
            player_inventory_start: 2 + inventory_columns * 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemCost {
    pub item_id: &'static str,
    pub count: i32,
}

impl ItemCost {
    pub fn new(item_id: &'static str, count: i32) -> Self {
        Self { item_id, count }
    }

    pub fn matches(&self, stack: &ItemStack) -> bool {
        !stack.is_empty() && stack.item_id() == self.item_id
    }

    pub fn item_stack(&self) -> ItemStack {
        ItemStack::new(self.item_id, self.count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOffer {
    pub base_cost_a: ItemCost,
    pub cost_b: Option<ItemCost>,
    pub result: ItemStack,
    pub uses: i32,
    pub max_uses: i32,
    pub reward_exp: bool,
    pub special_price_diff: i32,
    pub demand: i32,
    pub price_multiplier: f32,
    pub xp: i32,
    pub ignore_discount: bool,
}

impl MerchantOffer {
    pub fn new(
        base_cost_a: ItemCost,
        cost_b: Option<ItemCost>,
        result: ItemStack,
        max_uses: i32,
        xp: i32,
        price_multiplier: f32,
    ) -> Self {
        Self {
            base_cost_a,
            cost_b,
            result,
            uses: 0,
            max_uses,
            reward_exp: true,
            special_price_diff: 0,
            demand: 0,
            price_multiplier,
            xp,
            ignore_discount: false,
        }
    }

    pub fn cost_a_count(&self) -> i32 {
        let base = self.base_cost_a.count;
        let demand_diff =
            ((base as f32 * self.demand as f32 * self.price_multiplier).floor() as i32).max(0);
        (base + demand_diff + self.special_price_diff)
            .clamp(1, self.base_cost_a.item_stack().max_stack_size() as i32)
    }

    pub fn get_cost_a(&self) -> ItemStack {
        ItemStack::new(self.base_cost_a.item_id, self.cost_a_count())
    }

    pub fn assemble(&self) -> ItemStack {
        self.result.clone()
    }

    pub fn update_demand(&mut self) {
        self.demand = self.demand + self.uses - (self.max_uses - self.uses);
    }

    pub fn increase_uses(&mut self) {
        self.uses += 1;
    }

    pub fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    pub fn needs_restock(&self) -> bool {
        self.uses > 0
    }

    pub fn reset_uses(&mut self) {
        self.uses = 0;
    }

    pub fn satisfied_by(&self, buy_a: &ItemStack, buy_b: &ItemStack) -> bool {
        self.base_cost_a.matches(buy_a)
            && buy_a.count() >= self.cost_a_count()
            && match &self.cost_b {
                Some(cost_b) => cost_b.matches(buy_b) && buy_b.count() >= cost_b.count,
                None => buy_b.is_empty(),
            }
    }

    pub fn take(&self, buy_a: &mut ItemStack, buy_b: &mut ItemStack) -> bool {
        if !self.satisfied_by(buy_a, buy_b) {
            return false;
        }
        buy_a.shrink(self.cost_a_count());
        if let Some(cost_b) = &self.cost_b {
            buy_b.shrink(cost_b.count);
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantContainer {
    slots: [ItemStack; 3],
    offers: Vec<MerchantOffer>,
    active_offer: Option<usize>,
    selection_hint: usize,
    future_xp: i32,
}

impl MerchantContainer {
    pub fn new(offers: Vec<MerchantOffer>) -> Self {
        Self {
            slots: [ItemStack::empty(), ItemStack::empty(), ItemStack::empty()],
            offers,
            active_offer: None,
            selection_hint: 0,
            future_xp: 0,
        }
    }

    pub fn set_selection_hint(&mut self, selection_hint: usize) {
        self.selection_hint = selection_hint;
        self.update_sell_item();
    }

    pub fn set_payment(&mut self, slot: usize, stack: ItemStack) {
        if slot <= 1 {
            self.slots[slot] = stack;
            self.update_sell_item();
        }
    }

    pub fn can_trade(&self) -> bool {
        self.active_offer.is_some()
    }

    pub fn prepare_trade(&mut self) {
        self.update_sell_item();
    }

    pub fn result(&self) -> &ItemStack {
        &self.slots[2]
    }

    pub fn active_offer(&self) -> Option<&MerchantOffer> {
        self.active_offer.and_then(|index| self.offers.get(index))
    }

    pub fn offer(&self, index: usize) -> Option<&MerchantOffer> {
        self.offers.get(index)
    }

    pub fn future_xp(&self) -> i32 {
        self.future_xp
    }

    pub fn take_result(&mut self) -> ItemStack {
        let result = std::mem::replace(&mut self.slots[2], ItemStack::empty());
        if !result.is_empty() {
            if let Some(index) = self.active_offer {
                let (payment_a, rest) = self.slots.split_at_mut(1);
                if self.offers[index].take(&mut payment_a[0], &mut rest[0]) {
                    self.offers[index].increase_uses();
                }
            }
        }
        self.update_sell_item();
        result
    }

    pub fn update_sell_item(&mut self) {
        self.active_offer = None;
        let (buy_a, buy_b, swapped) = if self.slots[0].is_empty() {
            (&self.slots[1], &ItemStack::empty(), true)
        } else {
            (&self.slots[0], &self.slots[1], false)
        };
        if buy_a.is_empty() {
            self.slots[2] = ItemStack::empty();
            self.future_xp = 0;
            return;
        }

        let hinted = self.find_matching_offer(buy_a, buy_b, self.selection_hint);
        let fallback = if hinted.is_some() {
            hinted
        } else if swapped {
            None
        } else {
            self.find_matching_offer(&self.slots[1], &self.slots[0], self.selection_hint)
        };
        if let Some(index) = fallback.filter(|index| !self.offers[*index].is_out_of_stock()) {
            self.active_offer = Some(index);
            self.slots[2] = self.offers[index].assemble();
            self.future_xp = self.offers[index].xp;
        } else {
            self.slots[2] = ItemStack::empty();
            self.future_xp = 0;
        }
    }

    fn find_matching_offer(
        &self,
        buy_a: &ItemStack,
        buy_b: &ItemStack,
        hint: usize,
    ) -> Option<usize> {
        if let Some(offer) = self.offers.get(hint) {
            if offer.satisfied_by(buy_a, buy_b) {
                return Some(hint);
            }
        }
        self.offers
            .iter()
            .position(|offer| offer.satisfied_by(buy_a, buy_b))
    }

    /// Number of offers exposed by the merchant. Java: `MerchantOffers.size()`.
    pub fn offer_count(&self) -> usize {
        self.offers.len()
    }

    /// Read-only access to the offers. Used by `MerchantMenu::try_move_items`
    /// and by clients persisting trade state.
    pub fn offers(&self) -> &[MerchantOffer] {
        &self.offers
    }

    /// Run the full restock cycle for this merchant.
    ///
    /// Java: `Villager.restock()` calls `updateDemand()` over every offer
    /// *before* resetting uses so the next demand tick observes the trades
    /// that have happened during this restock window. After that, every
    /// offer's `uses` counter is cleared and the result slot is refreshed
    /// so the player immediately sees the restored stock.
    pub fn restock(&mut self) {
        self.update_demand();
        for offer in &mut self.offers {
            offer.reset_uses();
        }
        self.update_sell_item();
    }

    /// Update demand for every offer.
    ///
    /// Java: `Villager.updateDemand()` iterates the offers and delegates to
    /// `MerchantOffer.updateDemand()`, which uses the formula
    /// `demand = demand + uses - (maxUses - uses)`.
    pub fn update_demand(&mut self) {
        for offer in &mut self.offers {
            offer.update_demand();
        }
    }

    /// Apply the hero-of-the-village price reduction to every offer.
    ///
    /// Java: `Villager.updateSpecialPrices()` computes
    /// `modifier = 0.3 + 0.0625 * amplifier`, then for every offer subtracts
    /// `max(floor(modifier * baseCostA.count), 1)` from `specialPriceDiff`.
    /// `amplifier` is the 0-based effect amplifier from
    /// `MobEffectInstance.getAmplifier()`.
    pub fn apply_hero_discount(&mut self, amplifier: i32) {
        let modifier = 0.3 + 0.0625 * amplifier as f64;
        for offer in &mut self.offers {
            let cost_reduction =
                ((modifier * offer.base_cost_a.count as f64).floor() as i32).max(1);
            offer.special_price_diff -= cost_reduction;
        }
    }

    /// Clear the hero-of-the-village discount for every offer.
    ///
    /// Called when the effect wears off and after restock so prices return to
    /// their base values plus any demand modifier. Java: `Villager` re-runs
    /// `updateSpecialPrices()` on each interaction; this matches the side
    /// effect of an empty reputation/effect combination.
    pub fn reset_special_prices(&mut self) {
        for offer in &mut self.offers {
            offer.special_price_diff = 0;
        }
    }
}

// ============================================================================
// MerchantRestockTracker
// ============================================================================

/// Tracks how many times a villager has restocked today and at what game time.
///
/// Java: `Villager` holds two persistent fields: `numberOfRestocksToday` and
/// `lastRestockGameTime`. The merchant is allowed to restock if it has not
/// restocked yet today, or it has restocked once and at least 2400 ticks have
/// passed since that restock. `Villager.shouldRestock()` resets the counter
/// when a new game day starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerchantRestockTracker {
    pub restocks_today: i32,
    pub last_restock_game_time: i64,
}

impl MerchantRestockTracker {
    /// Build a fresh tracker for a villager that has never restocked.
    pub fn new() -> Self {
        Self {
            restocks_today: 0,
            last_restock_game_time: 0,
        }
    }

    /// Java: `Villager.allowedToRestock()`.
    ///
    /// A villager may restock either if it has not restocked at all today, or
    /// it has restocked exactly once and the configured cooldown (2400 ticks
    /// = 2 minutes of game time) has elapsed since the last restock.
    pub fn allowed_to_restock(&self, game_time: i64) -> bool {
        self.restocks_today == 0
            || (self.restocks_today < 2 && game_time > self.last_restock_game_time + 2400)
    }

    /// Record a restock at the given game time.
    ///
    /// Java: at the end of `Villager.restock()` the villager assigns
    /// `lastRestockGameTime = level.getGameTime()` and increments
    /// `numberOfRestocksToday`.
    pub fn record_restock(&mut self, game_time: i64) {
        self.last_restock_game_time = game_time;
        self.restocks_today += 1;
    }

    /// Java: `Villager.resetNumberOfRestocks()`. Called when a new day begins.
    pub fn reset_for_new_day(&mut self) {
        self.restocks_today = 0;
    }
}

impl Default for MerchantRestockTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CraftingGrid {
    width: usize,
    height: usize,
    slots: Vec<ItemStack>,
    result: ItemStack,
    recipe_id: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryMenuSlot {
    Result,
    CraftingInput(usize),
    Armor(EquipmentSlot),
    Storage(usize),
    Hotbar(usize),
    Offhand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryMenu {
    player: PlayerInventory,
    crafting: CraftingGrid,
    recipes: RecipeMap,
    unlocked_recipes: BTreeSet<&'static str>,
    highlighted_recipes: BTreeSet<&'static str>,
    recipe_unlock_events: Vec<&'static str>,
}

impl CraftingGrid {
    pub fn two_by_two() -> Self {
        Self::new(2, 2)
    }

    pub fn three_by_three() -> Self {
        Self::new(3, 3)
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            slots: vec![ItemStack::empty(); width * height],
            result: ItemStack::empty(),
            recipe_id: None,
        }
    }

    pub fn set_input(&mut self, slot: usize, stack: ItemStack, recipes: &RecipeMap) {
        if let Some(target) = self.slots.get_mut(slot) {
            *target = stack;
            self.update_result(recipes);
        }
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn input(&self, slot: usize) -> Option<&ItemStack> {
        self.slots.get(slot)
    }

    pub fn recipe_id(&self) -> Option<&'static str> {
        self.recipe_id
    }

    pub fn take_result(&mut self, recipes: &RecipeMap) -> ItemStack {
        let result = self.result.clone();
        if result.is_empty() {
            return ItemStack::empty();
        }
        self.consume_inputs_and_refresh(recipes);
        result
    }

    /// Drains every non-empty input slot and returns the items.  The grid's result
    /// and recipe ID are cleared.  Mirrors `InventoryMenu.removed()` + `clearContainer()`
    /// in Java, which is called when the player disconnects.
    pub fn take_all_inputs(&mut self) -> Vec<ItemStack> {
        let mut out = Vec::new();
        for slot in &mut self.slots {
            if !slot.is_empty() {
                out.push(std::mem::replace(slot, ItemStack::empty()));
            }
        }
        if !out.is_empty() {
            self.result = ItemStack::empty();
            self.recipe_id = None;
        }
        out
    }

    pub fn input_count(&self, item_id: &'static str) -> i32 {
        self.slots
            .iter()
            .filter(|stack| stack.item_id() == item_id)
            .map(ItemStack::count)
            .sum()
    }

    fn update_result(&mut self, recipes: &RecipeMap) {
        let items = self
            .slots
            .iter()
            .map(|stack| {
                if stack.is_empty() {
                    None
                } else {
                    Some(stack.item_id())
                }
            })
            .collect::<Vec<_>>();
        if let Some(holder) = recipes.get_recipe_for("crafting", self.width, self.height, &items) {
            self.recipe_id = Some(holder.id);
            self.result = holder
                .recipe
                .assemble()
                .map(|result| ItemStack::new(result.item, result.count as i32))
                .unwrap_or_else(ItemStack::empty);
        } else {
            self.recipe_id = None;
            self.result = ItemStack::empty();
        }
    }

    fn consume_inputs_and_refresh(&mut self, recipes: &RecipeMap) {
        let remaining_items = self
            .recipe_id
            .and_then(|recipe_id| recipes.by_key(recipe_id))
            .map(|holder| {
                let input = self
                    .slots
                    .iter()
                    .map(|stack| {
                        (!stack.is_empty()).then(|| CraftingStack {
                            item: stack.item_id(),
                            count: stack.count() as u32,
                        })
                    })
                    .collect::<Vec<_>>();
                holder.recipe.get_remaining_items(&input)
            })
            .unwrap_or_else(|| vec![None; self.slots.len()]);

        for (slot, remainder) in self.slots.iter_mut().zip(remaining_items) {
            if !slot.is_empty() {
                slot.shrink(1);
                if slot.is_empty() {
                    *slot = ItemStack::empty();
                }
            }
            if let Some(remainder) = remainder {
                let remainder_stack = ItemStack::new(remainder.item, remainder.count as i32);
                if slot.is_empty() {
                    *slot = remainder_stack;
                } else if same_item_same_components(slot, &remainder_stack) {
                    slot.grow(remainder_stack.count());
                }
            }
        }
        self.update_result(recipes);
    }
}

impl InventoryMenuSlot {
    pub fn from_vanilla_slot(slot: usize) -> Option<Self> {
        match slot {
            0 => Some(Self::Result),
            1..=4 => Some(Self::CraftingInput(slot - 1)),
            5 => Some(Self::Armor(EquipmentSlot::Head)),
            6 => Some(Self::Armor(EquipmentSlot::Chest)),
            7 => Some(Self::Armor(EquipmentSlot::Legs)),
            8 => Some(Self::Armor(EquipmentSlot::Feet)),
            9..=35 => Some(Self::Storage(slot)),
            36..=44 => Some(Self::Hotbar(slot - 36)),
            45 => Some(Self::Offhand),
            _ => None,
        }
    }

    pub fn may_place(self) -> bool {
        !matches!(self, Self::Result)
    }

    fn player_slot(self) -> Option<usize> {
        match self {
            Self::Armor(EquipmentSlot::Head) => Some(39),
            Self::Armor(EquipmentSlot::Chest) => Some(38),
            Self::Armor(EquipmentSlot::Legs) => Some(37),
            Self::Armor(EquipmentSlot::Feet) => Some(36),
            Self::Armor(_) => None,
            Self::Storage(slot) => Some(slot),
            Self::Hotbar(slot) => Some(slot),
            Self::Offhand => Some(SLOT_OFFHAND),
            _ => None,
        }
    }
}

impl InventoryMenu {
    pub const SLOT_COUNT: usize = 46;

    pub fn new(player: PlayerInventory, recipes: RecipeMap) -> Self {
        Self {
            player,
            crafting: CraftingGrid::two_by_two(),
            recipes,
            unlocked_recipes: BTreeSet::new(),
            highlighted_recipes: BTreeSet::new(),
            recipe_unlock_events: Vec::new(),
        }
    }

    pub fn player_inventory(&self) -> &PlayerInventory {
        &self.player
    }

    /// Mutable access to the underlying `PlayerInventory` for operations that bypass the
    /// crafting grid (block placement consumption, item pickup, serialisation).
    pub fn player_inventory_mut(&mut self) -> &mut PlayerInventory {
        &mut self.player
    }

    /// Returns all items currently in the 2×2 crafting grid back to the player's
    /// main inventory.  Items that do not fit are pushed to the player inventory's
    /// `dropped` list so callers can spawn them as world entities.
    ///
    /// Java equivalent: `InventoryMenu.removed(player)` → `clearContainer(player, craftSlots)`
    /// → `Inventory.placeItemBackInInventory(item)` (for a non-dead player).
    pub fn clear_crafting_to_inventory(&mut self) {
        for item in self.crafting.take_all_inputs() {
            // Use `add` rather than `place_item_back_in_inventory` so that items
            // which cannot fit don't silently pollute `dropped` — the caller can
            // inspect `player_inventory().dropped()` separately if needed.
            let _ = self.player.add(item);
        }
    }

    /// Consume this `InventoryMenu` and return the underlying `PlayerInventory`.
    ///
    /// Before transferring ownership, any items remaining in the 2×2 crafting grid are
    /// moved back into the player's inventory (mirroring `InventoryMenu.removed()` in Java).
    /// Items that cannot fit are appended to `PlayerInventory::dropped()`.
    pub fn into_player_inventory(mut self) -> PlayerInventory {
        self.clear_crafting_to_inventory();
        self.player
    }

    pub fn crafting_grid(&self) -> &CraftingGrid {
        &self.crafting
    }

    pub fn recipe_unlock_events(&self) -> &[&'static str] {
        &self.recipe_unlock_events
    }

    pub fn drain_recipe_unlock_events(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.recipe_unlock_events)
    }

    pub fn recipe_book_known_recipes(&self) -> Vec<&'static str> {
        self.unlocked_recipes.iter().copied().collect()
    }

    pub fn recipe_book_highlighted_recipes(&self) -> Vec<&'static str> {
        self.highlighted_recipes.iter().copied().collect()
    }

    pub fn load_recipe_book(
        &mut self,
        known_recipes: impl IntoIterator<Item = &'static str>,
        highlighted_recipes: impl IntoIterator<Item = &'static str>,
    ) {
        self.unlocked_recipes.clear();
        self.highlighted_recipes.clear();
        for recipe_id in known_recipes {
            if self.recipes.by_key(recipe_id).is_some() {
                self.unlocked_recipes.insert(recipe_id);
            }
        }
        for recipe_id in highlighted_recipes {
            if self.unlocked_recipes.contains(recipe_id) {
                self.highlighted_recipes.insert(recipe_id);
            }
        }
    }

    pub fn mark_recipe_seen(&mut self, recipe_id: &str) {
        self.highlighted_recipes.remove(recipe_id);
    }

    pub fn place_recipe_from_inventory(&mut self, recipe_id: &str, use_max_items: bool) -> bool {
        if !self.unlocked_recipes.contains(recipe_id) {
            return false;
        }
        let Some(holder) = self.recipes.by_key(recipe_id) else {
            return false;
        };
        let Some(placement) = crafting_recipe_placement(&holder.recipe, self.crafting.width, self.crafting.height) else {
            return false;
        };

        let mut next = self.clone();
        next.clear_crafting_to_inventory();
        let amount = if use_max_items {
            biggest_placeable_craft_count(&next.player, &placement).min(64)
        } else {
            1
        };
        if amount <= 0 {
            return false;
        }

        let mut placed = vec![ItemStack::empty(); placement.len()];
        for (grid_index, ingredient) in placement.iter().enumerate() {
            let Some(ingredient) = ingredient else {
                continue;
            };
            let Some((player_slot, item_id)) = find_player_slot_matching(&next.player, ingredient, amount) else {
                return false;
            };
            next.player.remove(player_slot, amount);
            placed[grid_index] = ItemStack::new(item_id, amount);
        }
        for (grid_index, stack) in placed.into_iter().enumerate() {
            next.crafting.set_input(grid_index, stack, &next.recipes);
        }
        *self = next;
        true
    }

    pub fn get_slot(&self, slot: usize) -> Option<ItemStack> {
        match InventoryMenuSlot::from_vanilla_slot(slot)? {
            InventoryMenuSlot::Result => Some(self.crafting.result().clone()),
            InventoryMenuSlot::CraftingInput(index) => self.crafting.input(index).cloned(),
            mapped => mapped
                .player_slot()
                .map(|slot| self.player.get(slot).clone()),
        }
    }

    pub fn set_slot(&mut self, slot: usize, stack: ItemStack) -> bool {
        match InventoryMenuSlot::from_vanilla_slot(slot) {
            Some(InventoryMenuSlot::Result) | None => false,
            Some(InventoryMenuSlot::CraftingInput(index)) => {
                self.crafting.set_input(index, stack, &self.recipes);
                true
            }
            Some(mapped) => {
                if let Some(player_slot) = mapped.player_slot() {
                    self.player.set(player_slot, stack);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn may_place(&self, slot: usize) -> bool {
        InventoryMenuSlot::from_vanilla_slot(slot).is_some_and(InventoryMenuSlot::may_place)
    }

    pub fn all_slots(&self) -> Vec<ItemStack> {
        (0..Self::SLOT_COUNT)
            .map(|slot| self.get_slot(slot).unwrap_or_else(ItemStack::empty))
            .collect()
    }

    pub fn take_result(&mut self) -> ItemStack {
        let Some(recipe_id) = self.crafting.recipe_id() else {
            return ItemStack::empty();
        };
        let result = self.crafting.result().clone();
        if result.is_empty() {
            return ItemStack::empty();
        }
        self.crafting.consume_inputs_and_refresh(&self.recipes);
        if self.unlocked_recipes.insert(recipe_id) {
            self.highlighted_recipes.insert(recipe_id);
            self.recipe_unlock_events.push(recipe_id);
        }
        result
    }

    pub fn quick_move(&mut self, slot: usize) -> ItemStack {
        let Some(source) = InventoryMenuSlot::from_vanilla_slot(slot) else {
            return ItemStack::empty();
        };
        let mut moving = if source == InventoryMenuSlot::Result {
            self.take_result()
        } else {
            let stack = self.get_slot(slot).unwrap_or_else(ItemStack::empty);
            if stack.is_empty() {
                return ItemStack::empty();
            }
            self.set_slot(slot, ItemStack::empty());
            stack
        };
        let original = moving.clone();

        match source {
            InventoryMenuSlot::Result => {
                self.insert_into_ranges(&mut moving, &[36..45, 9..36]);
            }
            InventoryMenuSlot::Hotbar(_) => {
                self.insert_into_ranges(&mut moving, &[9..36]);
            }
            InventoryMenuSlot::Storage(_) => {
                self.insert_into_armor_slot_if_possible(&mut moving);
                self.insert_into_ranges(&mut moving, &[36..45]);
            }
            _ => {
                self.insert_into_ranges(&mut moving, &[9..36, 36..45]);
            }
        }

        if !moving.is_empty() && source != InventoryMenuSlot::Result {
            self.set_slot(slot, moving);
        }
        original
    }

    fn insert_into_ranges(&mut self, stack: &mut ItemStack, ranges: &[std::ops::Range<usize>]) {
        for range in ranges {
            for slot in range.clone() {
                if stack.is_empty() {
                    return;
                }
                self.merge_into_slot(slot, stack);
            }
        }
        for range in ranges {
            for slot in range.clone() {
                if stack.is_empty() {
                    return;
                }
                self.move_into_empty_slot(slot, stack);
            }
        }
    }

    fn insert_into_armor_slot_if_possible(&mut self, stack: &mut ItemStack) {
        let Some(slot) = matching_armor_menu_slot(stack.item_id()) else {
            return;
        };
        self.move_into_empty_slot(slot, stack);
    }

    fn merge_into_slot(&mut self, slot: usize, stack: &mut ItemStack) {
        let Some(mut target) = self.get_slot(slot) else {
            return;
        };
        if target.is_empty() || !same_item_same_components(&target, stack) {
            return;
        }
        let room = target.max_stack_size() as i32 - target.count();
        let moved = room.max(0).min(stack.count());
        if moved > 0 {
            target.grow(moved);
            stack.shrink(moved);
            self.set_slot(slot, target);
        }
    }

    fn move_into_empty_slot(&mut self, slot: usize, stack: &mut ItemStack) {
        if stack.is_empty()
            || !self.may_place(slot)
            || self.get_slot(slot).is_none_or(|target| !target.is_empty())
        {
            return;
        }
        let moved = stack.count().min(stack.max_stack_size() as i32);
        self.set_slot(slot, stack.split(moved));
    }
}

fn crafting_recipe_placement(
    recipe: &RecipeKind,
    grid_width: usize,
    grid_height: usize,
) -> Option<Vec<Option<IngredientSpec>>> {
    match recipe {
        RecipeKind::Shaped {
            width,
            height,
            pattern,
            ..
        } => {
            if *width > grid_width || *height > grid_height {
                return None;
            }
            let mut placement = vec![None; grid_width * grid_height];
            let x_offset = centered_recipe_offset(grid_width, *width);
            let y_offset = centered_recipe_offset(grid_height, *height);
            for y in 0..*height {
                for x in 0..*width {
                    placement[(y + y_offset) * grid_width + x + x_offset] =
                        pattern[y * *width + x].clone();
                }
            }
            Some(placement)
        }
        RecipeKind::Shapeless { ingredients, .. } => {
            if ingredients.len() > grid_width * grid_height {
                return None;
            }
            let mut placement = vec![None; grid_width * grid_height];
            for (index, ingredient) in ingredients.iter().enumerate() {
                placement[index] = Some(ingredient.clone());
            }
            Some(placement)
        }
        _ => None,
    }
}

fn centered_recipe_offset(grid_size: usize, recipe_size: usize) -> usize {
    if (recipe_size as f32) < (grid_size as f32 / 2.0) {
        ((grid_size as f32 / 2.0) - (recipe_size as f32 / 2.0)).floor() as usize
    } else {
        0
    }
}

fn biggest_placeable_craft_count(
    inventory: &PlayerInventory,
    placement: &[Option<IngredientSpec>],
) -> i32 {
    let mut trial = 0;
    for amount in 1..=64 {
        if can_satisfy_placement(inventory, placement, amount) {
            trial = amount;
        } else {
            break;
        }
    }
    trial
}

fn can_satisfy_placement(
    inventory: &PlayerInventory,
    placement: &[Option<IngredientSpec>],
    amount: i32,
) -> bool {
    let mut counts: Vec<(&'static str, i32)> = (0..INVENTORY_SIZE)
        .filter_map(|slot| {
            let stack = inventory.get(slot);
            (!stack.is_empty()).then_some((stack.item_id(), stack.count()))
        })
        .collect();

    for ingredient in placement.iter().flatten() {
        let Some((_, count)) = counts
            .iter_mut()
            .find(|(item_id, count)| *count >= amount && ingredient.matches(item_id))
        else {
            return false;
        };
        *count -= amount;
    }
    true
}

fn find_player_slot_matching(
    inventory: &PlayerInventory,
    ingredient: &IngredientSpec,
    amount: i32,
) -> Option<(usize, &'static str)> {
    (0..INVENTORY_SIZE).find_map(|slot| {
        let stack = inventory.get(slot);
        (!stack.is_empty() && stack.count() >= amount && ingredient.matches(stack.item_id()))
            .then_some((slot, stack.item_id()))
    })
}

fn matching_armor_menu_slot(item_id: &str) -> Option<usize> {
    if item_id.ends_with("_helmet") || item_id == "minecraft:turtle_helmet" {
        Some(5)
    } else if item_id.ends_with("_chestplate") || item_id == "minecraft:elytra" {
        Some(6)
    } else if item_id.ends_with("_leggings") {
        Some(7)
    } else if item_id.ends_with("_boots") {
        Some(8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crafting_test_recipes() -> crate::recipe_system::RecipeMap {
        crate::recipe_system::RecipeMap::create(vec![
            crate::recipe_system::RecipeHolder {
                id: "minecraft:oak_planks",
                recipe: crate::recipe_system::RecipeKind::Shapeless {
                    ingredients: vec![crate::recipe_system::IngredientSpec::Item(
                        "minecraft:oak_log",
                    )],
                    result: crate::recipe_system::ItemAmount {
                        item: "minecraft:oak_planks",
                        count: 4,
                    },
                },
            },
            crate::recipe_system::RecipeHolder {
                id: "minecraft:crafting_table",
                recipe: crate::recipe_system::RecipeKind::Shaped {
                    width: 2,
                    height: 2,
                    pattern: vec![
                        Some(crate::recipe_system::IngredientSpec::Item(
                            "minecraft:oak_planks",
                        )),
                        Some(crate::recipe_system::IngredientSpec::Item(
                            "minecraft:oak_planks",
                        )),
                        Some(crate::recipe_system::IngredientSpec::Item(
                            "minecraft:oak_planks",
                        )),
                        Some(crate::recipe_system::IngredientSpec::Item(
                            "minecraft:oak_planks",
                        )),
                    ],
                    result: crate::recipe_system::ItemAmount::one("minecraft:crafting_table"),
                },
            },
            crate::recipe_system::RecipeHolder {
                id: "minecraft:test_bucket_recipe",
                recipe: crate::recipe_system::RecipeKind::Shapeless {
                    ingredients: vec![crate::recipe_system::IngredientSpec::Item(
                        "minecraft:water_bucket",
                    )],
                    result: crate::recipe_system::ItemAmount::one("minecraft:clay"),
                },
            },
            crate::recipe_system::RecipeHolder {
                id: "minecraft:torch",
                recipe: crate::recipe_system::RecipeKind::Shapeless {
                    ingredients: vec![
                        crate::recipe_system::IngredientSpec::Item("minecraft:stick"),
                        crate::recipe_system::IngredientSpec::Item("minecraft:stick"),
                    ],
                    result: crate::recipe_system::ItemAmount {
                        item: "minecraft:torch",
                        count: 4,
                    },
                },
            },
        ])
    }

    #[test]
    fn player_inventory_uses_vanilla_slot_mapping_and_hotbar_validation() {
        let mut inventory = PlayerInventory::new();
        assert_eq!(inventory.container_size(), 43);
        assert_eq!(
            PlayerInventory::equipment_slot(36),
            Some(EquipmentSlot::Feet)
        );
        assert_eq!(
            PlayerInventory::equipment_slot(39),
            Some(EquipmentSlot::Head)
        );
        assert_eq!(
            PlayerInventory::equipment_slot(SLOT_OFFHAND),
            Some(EquipmentSlot::Offhand)
        );
        assert_eq!(
            PlayerInventory::equipment_slot(SLOT_BODY_ARMOR),
            Some(EquipmentSlot::Body)
        );
        assert_eq!(
            PlayerInventory::equipment_slot(SLOT_SADDLE),
            Some(EquipmentSlot::Saddle)
        );

        assert!(inventory.set_selected_slot(8).is_ok());
        assert_eq!(inventory.selected_slot(), 8);
        assert!(inventory.set_selected_slot(9).is_err());
    }

    #[test]
    fn add_prefers_selected_then_offhand_then_existing_inventory_space() {
        let mut inventory = PlayerInventory::new();
        inventory.set_selected_slot(2).unwrap();
        inventory.set(2, ItemStack::new("minecraft:stick", 60));
        inventory.set(SLOT_OFFHAND, ItemStack::new("minecraft:stick", 10));
        inventory.set(5, ItemStack::new("minecraft:stick", 10));

        assert_eq!(
            inventory.add(ItemStack::new("minecraft:stick", 12)),
            InventoryAddResult::FullyAdded
        );
        assert_eq!(inventory.get(2).count(), 64);
        assert_eq!(inventory.get(SLOT_OFFHAND).count(), 18);
        assert_eq!(inventory.get(5).count(), 10);
        assert_eq!(inventory.get(2).pop_time(), 5);
    }

    #[test]
    fn inventory_save_load_tracks_only_non_equipment_storage_slots() {
        let mut inventory = PlayerInventory::new();
        inventory.set(0, ItemStack::new("minecraft:apple", 3));
        inventory.set(37, ItemStack::new("minecraft:diamond_helmet", 1));
        inventory.set(SLOT_OFFHAND, ItemStack::new("minecraft:shield", 1));

        let saved = inventory.saved_items();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].0, 0);

        let mut loaded = PlayerInventory::new();
        loaded.load_items(&saved);
        assert_eq!(loaded.get(0).item_id(), "minecraft:apple");
        assert!(loaded.get(37).is_empty());
        assert!(loaded.get(SLOT_OFFHAND).is_empty());
    }

    #[test]
    fn place_item_back_drops_when_inventory_is_full() {
        let mut inventory = PlayerInventory::new();
        for slot in 0..INVENTORY_SIZE {
            inventory.set(slot, ItemStack::new("minecraft:apple", 64));
        }

        assert_eq!(
            inventory.place_item_back_in_inventory(ItemStack::new("minecraft:stick", 3)),
            InventoryAddResult::Dropped { count: 3 }
        );
        assert_eq!(inventory.dropped()[0].item_id(), "minecraft:stick");
    }

    #[test]
    fn ender_chest_and_generic_containers_have_fixed_slot_behavior() {
        let mut ender_chest = SimpleContainer::ender_chest();
        assert_eq!(ender_chest.size(), 27);
        ender_chest.set(26, ItemStack::new("minecraft:apple", 70));
        assert_eq!(ender_chest.get(26).count(), 64);
        assert_eq!(ender_chest.remove(26, 8).count(), 8);
        assert_eq!(ender_chest.get(26).count(), 56);
    }

    #[test]
    fn horse_inventory_layout_matches_equipment_and_chest_slots() {
        let horse = HorseInventoryLayout::new(true, true, false, 5);
        assert!(horse.saddle_slot_active);
        assert!(horse.armor_slot_active);
        assert_eq!(horse.chest_slots, 15);
        assert_eq!(horse.player_inventory_start, 17);

        let llama = HorseInventoryLayout::new(false, false, true, 3);
        assert!(!llama.saddle_slot_active);
        assert!(llama.armor_slot_active);
        assert_eq!(llama.chest_slots, 9);
    }

    #[test]
    fn merchant_offer_applies_special_price_demand_stock_and_payment_consumption() {
        let mut offer = MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 5),
            Some(ItemCost::new("minecraft:book", 1)),
            ItemStack::new("minecraft:written_book", 1),
            2,
            7,
            0.2,
        );
        assert_eq!(offer.base_cost_a, ItemCost::new("minecraft:emerald", 5));
        assert_eq!(offer.cost_b, Some(ItemCost::new("minecraft:book", 1)));
        assert_eq!(offer.result, ItemStack::new("minecraft:written_book", 1));
        assert_eq!(offer.uses, 0);
        assert_eq!(offer.max_uses, 2);
        assert!(offer.reward_exp);
        assert_eq!(offer.special_price_diff, 0);
        assert_eq!(offer.demand, 0);
        assert_eq!(offer.price_multiplier, 0.2);
        assert_eq!(offer.xp, 7);
        assert!(!offer.ignore_discount);

        offer.demand = 3;
        offer.special_price_diff = -1;
        assert_eq!(offer.cost_a_count(), 7);

        let mut emeralds = ItemStack::new("minecraft:emerald", 8);
        let mut book = ItemStack::new("minecraft:book", 1);
        assert!(offer.take(&mut emeralds, &mut book));
        assert_eq!(emeralds.count(), 1);
        assert!(book.is_empty());

        offer.increase_uses();
        assert!(offer.needs_restock());
        offer.increase_uses();
        assert!(offer.is_out_of_stock());
        offer.update_demand();
        assert_eq!(offer.demand, 5);

        offer.demand = 6;
        offer.uses = 1;
        offer.max_uses = 4;
        offer.update_demand();
        assert_eq!(offer.demand, 4);
    }

    #[test]
    fn merchant_container_selects_active_offer_and_clears_out_of_stock_results() {
        let offer = MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 2),
            None,
            ItemStack::new("minecraft:apple", 4),
            1,
            3,
            0.0,
        );
        let mut container = MerchantContainer::new(vec![offer]);
        container.set_payment(0, ItemStack::new("minecraft:emerald", 2));

        assert!(container.can_trade());
        assert_eq!(container.result().item_id(), "minecraft:apple");
        assert_eq!(container.future_xp(), 3);
        assert_eq!(container.take_result().count(), 4);
        assert!(!container.can_trade());
        assert!(container.result().is_empty());
        assert!(container.offer(0).unwrap().is_out_of_stock());

        container.prepare_trade();
        assert!(!container.can_trade());
        assert!(container.result().is_empty());
    }

    #[test]
    fn merchant_container_respects_selection_hint_and_payment_slots() {
        let cheap = MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 1),
            None,
            ItemStack::new("minecraft:apple", 1),
            8,
            1,
            0.0,
        );
        let specific = MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 1),
            Some(ItemCost::new("minecraft:book", 1)),
            ItemStack::new("minecraft:written_book", 1),
            8,
            5,
            0.0,
        );
        let mut container = MerchantContainer::new(vec![cheap, specific]);

        container.set_selection_hint(1);
        container.set_payment(0, ItemStack::new("minecraft:emerald", 1));
        container.set_payment(1, ItemStack::new("minecraft:book", 1));
        assert!(container.can_trade());
        assert_eq!(container.active_offer(), container.offer(1));
        assert_eq!(container.result().item_id(), "minecraft:written_book");
        assert_eq!(container.future_xp(), 5);

        container.set_payment(1, ItemStack::empty());
        container.set_payment(0, ItemStack::empty());
        container.set_payment(1, ItemStack::new("minecraft:emerald", 1));
        container.prepare_trade();
        assert!(container.can_trade());
        assert_eq!(container.active_offer(), container.offer(0));
        assert_eq!(container.result().item_id(), "minecraft:apple");

        container.set_payment(1, ItemStack::empty());
        container.set_payment(0, ItemStack::new("minecraft:dirt", 1));
        assert!(!container.can_trade());
        assert!(container.result().is_empty());
        assert_eq!(container.future_xp(), 0);
    }

    #[test]
    fn merchant_offer_demand_increases_after_purchase_and_resets_after_restock() {
        // Java parity: trading consumes uses; updateDemand uses the formula
        // demand += uses - (maxUses - uses); restock then clears uses.
        let mut container = MerchantContainer::new(vec![MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 5),
            None,
            ItemStack::new("minecraft:written_book", 1),
            2,    // max_uses
            1,    // xp
            0.05, // price_multiplier
        )]);

        // Place enough emeralds to trade
        container.set_payment(0, ItemStack::new("minecraft:emerald", 5));
        assert!(container.can_trade());

        // First trade — uses becomes 1.
        let result = container.take_result();
        assert_eq!(result.item_id(), "minecraft:written_book");
        assert_eq!(container.offers()[0].uses, 1);

        // Second trade — uses becomes 2, offer goes out of stock.
        container.set_payment(0, ItemStack::new("minecraft:emerald", 5));
        let _ = container.take_result();
        assert_eq!(container.offers()[0].uses, 2);
        assert!(container.offers()[0].is_out_of_stock());

        // Restock: demand += uses - (maxUses - uses) = 0 + 2 - 0 = 2, then
        // every offer's uses counter resets to 0.
        container.restock();
        assert_eq!(container.offers()[0].uses, 0);
        assert_eq!(container.offers()[0].demand, 2);

        // With a higher price multiplier (0.5) and demand=2 the cost shifts
        // from 5 to 10 emeralds: 5 + floor(5 * 2 * 0.5) = 5 + 5 = 10.
        let mut offer2 = MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 5),
            None,
            ItemStack::new("minecraft:written_book", 1),
            2,
            1,
            0.5,
        );
        offer2.demand = 2;
        assert_eq!(offer2.cost_a_count(), 10);
    }

    #[test]
    fn merchant_restock_tracker_allows_two_restocks_per_day_2400_ticks_apart() {
        // Java parity: Villager.allowedToRestock() returns true when the
        // villager has never restocked today, or it has restocked once and
        // 2400 ticks have passed since that restock.
        let mut tracker = MerchantRestockTracker::new();

        // Initially allowed (first restock of the day).
        assert!(tracker.allowed_to_restock(0));
        tracker.record_restock(100);
        assert_eq!(tracker.restocks_today, 1);

        // Second restock requires 2400 ticks of cooldown to elapse.
        assert!(!tracker.allowed_to_restock(100));
        assert!(!tracker.allowed_to_restock(2499));
        assert!(tracker.allowed_to_restock(2501));

        tracker.record_restock(2501);
        assert_eq!(tracker.restocks_today, 2);

        // After two restocks no further restock is allowed regardless of
        // how much time has passed — the cap resets only on a new day.
        assert!(!tracker.allowed_to_restock(9999));

        // resetForNewDay clears the daily counter.
        tracker.reset_for_new_day();
        assert!(tracker.allowed_to_restock(9999));
    }

    #[test]
    fn merchant_hero_discount_reduces_special_price_diff() {
        // Java parity: Villager.updateSpecialPrices subtracts
        // max(floor((0.3 + 0.0625 * amplifier) * baseCostA.count), 1) from
        // each offer's specialPriceDiff when the player has the
        // hero-of-the-village effect.
        let mut container = MerchantContainer::new(vec![MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 10),
            None,
            ItemStack::new("minecraft:diamond", 1),
            5,
            1,
            0.05,
        )]);

        // amplifier 0: modifier = 0.3, costReduction = floor(0.3 * 10) = 3.
        container.apply_hero_discount(0);
        assert_eq!(container.offers()[0].special_price_diff, -3);

        // Reset and verify amplifier 1 yields the same rounded discount
        // (modifier = 0.3625, floor(0.3625 * 10) = 3).
        container.reset_special_prices();
        assert_eq!(container.offers()[0].special_price_diff, 0);
        container.apply_hero_discount(1);
        assert_eq!(container.offers()[0].special_price_diff, -3);
    }

    #[test]
    fn crafting_grid_updates_result_and_consumes_inputs_after_take() {
        let recipes = crafting_test_recipes();
        let mut grid = CraftingGrid::two_by_two();
        grid.set_input(0, ItemStack::new("minecraft:oak_log", 1), &recipes);
        assert_eq!(grid.recipe_id(), Some("minecraft:oak_planks"));
        assert_eq!(grid.result().item_id(), "minecraft:oak_planks");
        assert_eq!(grid.result().count(), 4);
        assert_eq!(grid.take_result(&recipes).count(), 4);
        assert!(grid.result().is_empty());

        grid.set_input(0, ItemStack::new("minecraft:stick", 1), &recipes);
        grid.set_input(1, ItemStack::new("minecraft:stick", 1), &recipes);

        assert_eq!(grid.recipe_id(), Some("minecraft:torch"));
        assert_eq!(grid.result().item_id(), "minecraft:torch");
        assert_eq!(grid.result().count(), 4);
        assert_eq!(grid.take_result(&recipes).count(), 4);
        assert!(grid.result().is_empty());
        assert_eq!(grid.input_count("minecraft:stick"), 0);

        grid.set_input(0, ItemStack::new("minecraft:stick", 1), &recipes);
        grid.set_input(3, ItemStack::new("minecraft:oak_log", 1), &recipes);
        assert!(grid.recipe_id().is_none());
        assert!(grid.result().is_empty());

        let mut wrong_shaped = CraftingGrid::two_by_two();
        wrong_shaped.set_input(0, ItemStack::new("minecraft:oak_planks", 1), &recipes);
        wrong_shaped.set_input(1, ItemStack::new("minecraft:oak_planks", 1), &recipes);
        wrong_shaped.set_input(2, ItemStack::new("minecraft:oak_planks", 1), &recipes);
        assert!(wrong_shaped.recipe_id().is_none());
        assert!(wrong_shaped.result().is_empty());

        let table = CraftingGrid::three_by_three();
        assert_eq!(table.width, 3);
        assert_eq!(table.height, 3);
    }

    #[test]
    fn inventory_menu_maps_vanilla_slots_to_backing_inventory_and_crafting_grid() {
        let mut player = PlayerInventory::new();
        player.set(0, ItemStack::new("minecraft:stick", 9));
        player.set(9, ItemStack::new("minecraft:cobblestone", 8));
        player.set(36, ItemStack::new("minecraft:leather_boots", 1));
        player.set(37, ItemStack::new("minecraft:iron_leggings", 1));
        player.set(38, ItemStack::new("minecraft:iron_chestplate", 1));
        player.set(39, ItemStack::new("minecraft:iron_helmet", 1));
        player.set(SLOT_OFFHAND, ItemStack::new("minecraft:shield", 1));

        let mut menu = InventoryMenu::new(player, crafting_test_recipes());
        assert_eq!(InventoryMenu::SLOT_COUNT, 46);
        assert!(!menu.may_place(0));
        assert!(menu.may_place(1));
        assert_eq!(menu.get_slot(36).unwrap().item_id(), "minecraft:stick");
        assert_eq!(menu.get_slot(9).unwrap().item_id(), "minecraft:cobblestone");
        assert_eq!(menu.get_slot(5).unwrap().item_id(), "minecraft:iron_helmet");
        assert_eq!(
            menu.get_slot(6).unwrap().item_id(),
            "minecraft:iron_chestplate"
        );
        assert_eq!(
            menu.get_slot(7).unwrap().item_id(),
            "minecraft:iron_leggings"
        );
        assert_eq!(
            menu.get_slot(8).unwrap().item_id(),
            "minecraft:leather_boots"
        );
        assert_eq!(menu.get_slot(45).unwrap().item_id(), "minecraft:shield");

        assert!(!menu.set_slot(0, ItemStack::new("minecraft:diamond", 1)));
        assert!(menu.get_slot(0).unwrap().is_empty());
        assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1)));
        assert_eq!(menu.get_slot(0).unwrap().item_id(), "minecraft:oak_planks");
        assert_eq!(menu.get_slot(0).unwrap().count(), 4);
        assert_eq!(
            menu.crafting_grid().recipe_id(),
            Some("minecraft:oak_planks")
        );

        assert!(menu.set_slot(36, ItemStack::new("minecraft:apple", 2)));
        assert_eq!(
            menu.player_inventory().get(0),
            &ItemStack::new("minecraft:apple", 2)
        );
        assert_eq!(menu.all_slots().len(), 46);
    }

    #[test]
    fn inventory_menu_result_take_consumes_inputs_remainders_and_unlocks_recipe_once() {
        let mut menu = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());

        assert!(menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1)));
        assert_eq!(menu.get_slot(0).unwrap().item_id(), "minecraft:clay");
        let result = menu.take_result();
        assert_eq!(result.item_id(), "minecraft:clay");
        assert_eq!(menu.get_slot(1).unwrap().item_id(), "minecraft:bucket");
        assert!(menu.get_slot(0).unwrap().is_empty());
        assert_eq!(
            menu.recipe_unlock_events(),
            &["minecraft:test_bucket_recipe"]
        );

        menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1));
        let second = menu.take_result();
        assert_eq!(second.item_id(), "minecraft:clay");
        assert_eq!(
            menu.recipe_unlock_events(),
            &["minecraft:test_bucket_recipe"],
            "recipe-book unlock should be emitted only for first craft"
        );
    }

    #[test]
    fn inventory_menu_quick_move_uses_vanilla_inventory_zones() {
        let mut menu = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());
        menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1));

        let crafted = menu.quick_move(0);
        assert_eq!(crafted.item_id(), "minecraft:oak_planks");
        assert_eq!(crafted.count(), 4);
        assert_eq!(menu.get_slot(36).unwrap().item_id(), "minecraft:oak_planks");
        assert!(menu.get_slot(1).unwrap().is_empty());
        assert!(menu.get_slot(0).unwrap().is_empty());

        for slot in 36..45 {
            menu.set_slot(slot, ItemStack::new("minecraft:cobblestone", 64));
        }
        menu.set_slot(36, ItemStack::new("minecraft:stick", 8));
        let hotbar = menu.quick_move(36);
        assert_eq!(hotbar.item_id(), "minecraft:stick");
        assert!(menu.get_slot(36).unwrap().is_empty());
        assert_eq!(menu.get_slot(9).unwrap().item_id(), "minecraft:stick");

        menu.set_slot(9, ItemStack::new("minecraft:apple", 3));
        let storage = menu.quick_move(9);
        assert_eq!(storage.item_id(), "minecraft:apple");
        assert_eq!(menu.get_slot(36).unwrap().item_id(), "minecraft:apple");

        menu.set_slot(10, ItemStack::new("minecraft:leather_boots", 1));
        let boots = menu.quick_move(10);
        assert_eq!(boots.item_id(), "minecraft:leather_boots");
        assert_eq!(
            menu.get_slot(8).unwrap().item_id(),
            "minecraft:leather_boots"
        );
    }

    #[test]
    fn death_drops_clears_all_slots_except_prevent_equipment_drop_and_respects_keep_inventory() {
        let mut inv = PlayerInventory::new();
        inv.set(0, ItemStack::new("minecraft:apple", 4));
        inv.set(1, ItemStack::new("minecraft:diamond_sword", 1)); // "has curse of vanishing"
        inv.set(36, ItemStack::new("minecraft:leather_boots", 1)); // feet armor slot

        // With keepInventory: nothing drops, nothing destroyed.
        let drops = inv.death_drops(true, |_| false);
        assert!(drops.is_empty());
        assert!(!inv.get(0).is_empty());

        // Without keepInventory: cursed item destroyed, others dropped.
        let curse_id = "minecraft:diamond_sword";
        let drops = inv.death_drops(false, |s| s.item_id() == curse_id);
        let dropped_ids: Vec<_> = drops.iter().map(|s| s.item_id()).collect();
        assert!(dropped_ids.contains(&"minecraft:apple"));
        assert!(!dropped_ids.contains(&curse_id)); // destroyed, not dropped
        assert!(dropped_ids.contains(&"minecraft:leather_boots"));
        // All slots now empty.
        for slot in 0..43 {
            assert!(
                inv.get(slot).is_empty(),
                "slot {slot} should be empty after death"
            );
        }
    }
}
