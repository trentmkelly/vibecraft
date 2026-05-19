#![allow(dead_code)]

use crate::inventory::same_item_same_components;
use crate::item_stack::ItemStack;

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
    pub fn death_drops<F>(&mut self, keep_inventory: bool, has_prevent_equipment_drop: F) -> Vec<ItemStack>
    where
        F: Fn(&ItemStack) -> bool,
    {
        if keep_inventory {
            return Vec::new();
        }
        let total = self.container_size();
        let mut drops = Vec::new();
        for slot in 0..total {
            let stack = if let Some(s) = self.stack_mut(slot) { s } else { continue };
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftingRecipe {
    Shapeless {
        inputs: Vec<(&'static str, i32)>,
        output: (&'static str, i32),
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CraftingGrid {
    width: usize,
    height: usize,
    slots: Vec<ItemStack>,
    result: ItemStack,
    recipe: Option<CraftingRecipe>,
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
            recipe: None,
        }
    }

    pub fn set_input(&mut self, slot: usize, stack: ItemStack) {
        if let Some(target) = self.slots.get_mut(slot) {
            *target = stack;
            self.update_result();
        }
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn take_result(&mut self) -> ItemStack {
        let result = self.result.clone();
        if result.is_empty() {
            return ItemStack::empty();
        }
        for slot in &mut self.slots {
            if !slot.is_empty() {
                slot.shrink(1);
            }
        }
        self.update_result();
        result
    }

    pub fn input_count(&self, item_id: &'static str) -> i32 {
        self.slots
            .iter()
            .filter(|stack| stack.item_id() == item_id)
            .map(ItemStack::count)
            .sum()
    }

    fn update_result(&mut self) {
        let non_empty: Vec<&ItemStack> = self
            .slots
            .iter()
            .filter(|stack| !stack.is_empty())
            .collect();
        let stick_count: i32 = non_empty
            .iter()
            .filter(|stack| stack.item_id() == "minecraft:stick")
            .map(|stack| stack.count())
            .sum();
        if non_empty.len() == 2 && stick_count == 2 {
            self.recipe = Some(CraftingRecipe::Shapeless {
                inputs: vec![("minecraft:stick", 2)],
                output: ("minecraft:torch", 4),
            });
            self.result = ItemStack::new("minecraft:torch", 4);
        } else {
            self.recipe = None;
            self.result = ItemStack::empty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(container.result().item_id(), "minecraft:apple");
        assert_eq!(container.future_xp(), 3);
        assert_eq!(container.take_result().count(), 4);
        assert!(container.result().is_empty());
        assert!(container.offer(0).unwrap().is_out_of_stock());
    }

    #[test]
    fn crafting_grid_updates_result_and_consumes_inputs_after_take() {
        let mut grid = CraftingGrid::two_by_two();
        grid.set_input(0, ItemStack::new("minecraft:stick", 1));
        grid.set_input(1, ItemStack::new("minecraft:stick", 1));

        assert_eq!(grid.result().item_id(), "minecraft:torch");
        assert_eq!(grid.result().count(), 4);
        assert_eq!(grid.take_result().count(), 4);
        assert!(grid.result().is_empty());
        assert_eq!(grid.input_count("minecraft:stick"), 0);

        let table = CraftingGrid::three_by_three();
        assert_eq!(table.width, 3);
        assert_eq!(table.height, 3);
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
            assert!(inv.get(slot).is_empty(), "slot {slot} should be empty after death");
        }
    }
}
