#![allow(dead_code)]

use std::collections::BTreeSet;

use crate::inventory::same_item_same_components;
use crate::item_stack::ItemStack;
use crate::recipe_system::{CraftingStack, IngredientSpec, RecipeBookType, RecipeKind, RecipeMap};

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

    /// Drop a stack in the world near the player (`Player.drop(stack, false)`), used
    /// by `AbstractContainerMenu.dropOrPlaceInInventory` on disconnect/removal. The
    /// stack is recorded in `dropped` (the same channel as inventory overflow).
    pub fn drop_item(&mut self, stack: ItemStack) {
        if !stack.is_empty() {
            self.dropped.push(stack);
        }
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
            let moved = stack.count().min(stack.max_stack_size() as i32);
            self.set(slot, stack.split(moved));
            if let Some(current) = self.stack_mut(slot) {
                current.set_pop_time(5);
            }
            return;
        }
        if let Some(current) = self.stack_mut(slot) {
            let room = current.max_stack_size() as i32 - current.count();
            let moved = stack.count().min(room.max(0));
            current.grow(moved);
            current.set_pop_time(5);
            stack.shrink(moved);
            if moved > 0 {
                self.set_changed();
            }
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

mod merchant;
pub use merchant::*;

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

    /// Close the player inventory menu, returning the cursor stack and 2×2 crafting inputs
    /// to the backing player inventory.
    ///
    /// Java: `InventoryMenu.removed(player)` first delegates to
    /// `AbstractContainerMenu.removed(player)`, which calls
    /// `Inventory.placeItemBackInInventory(carried)` for a connected server player, then clears
    /// the 2×2 crafting grid back into the player inventory.
    pub fn removed(&mut self, carried: &mut ItemStack) {
        if !carried.is_empty() {
            let stack = std::mem::replace(carried, ItemStack::empty());
            let _ = self.player.place_item_back_in_inventory(stack);
        }
        self.clear_crafting_to_inventory();
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

    pub fn unlock_recipe(&mut self, recipe_id: &'static str) -> bool {
        if self.recipes.by_key(recipe_id).is_none() {
            return false;
        }
        if self.unlocked_recipes.insert(recipe_id) {
            self.highlighted_recipes.insert(recipe_id);
            self.recipe_unlock_events.push(recipe_id);
            true
        } else {
            false
        }
    }

    pub fn recipe_book_type(&self) -> RecipeBookType {
        RecipeBookType::Crafting
    }

    pub fn place_recipe_from_inventory(&mut self, recipe_id: &str, use_max_items: bool) -> bool {
        if !self.unlocked_recipes.contains(recipe_id) {
            return false;
        }
        let Some(holder) = self.recipes.by_key(recipe_id) else {
            return false;
        };
        let Some(placement) =
            crafting_recipe_placement(&holder.recipe, self.crafting.width, self.crafting.height)
        else {
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
            let Some((player_slot, item_id)) =
                find_player_slot_matching(&next.player, ingredient, amount)
            else {
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
        self.unlock_recipe(recipe_id);
        result
    }

    pub fn quick_move(&mut self, slot: usize) -> ItemStack {
        let Some(source) = InventoryMenuSlot::from_vanilla_slot(slot) else {
            return ItemStack::empty();
        };
        let original = self.get_slot(slot).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();

        match source {
            InventoryMenuSlot::Result => {
                self.insert_into_range(&mut moving, 9..45, true);
            }
            InventoryMenuSlot::Hotbar(_) => {
                self.insert_into_range(&mut moving, 9..36, false);
            }
            InventoryMenuSlot::Storage(_) => {
                self.insert_into_equipment_slot_if_possible(&mut moving);
                self.insert_into_range(&mut moving, 36..45, false);
            }
            _ => {
                self.insert_into_range(&mut moving, 9..45, false);
            }
        }

        if moving.count() == original.count() {
            return ItemStack::empty();
        }
        if source == InventoryMenuSlot::Result {
            return self.take_result();
        }
        self.set_slot(slot, moving);
        original
    }

    fn insert_into_range(
        &mut self,
        stack: &mut ItemStack,
        range: std::ops::Range<usize>,
        backwards: bool,
    ) {
        let mut slots: Vec<usize> = range.collect();
        if backwards {
            slots.reverse();
        }
        for &slot in &slots {
            if stack.is_empty() {
                return;
            }
            self.merge_into_slot(slot, stack);
        }
        for &slot in &slots {
            if stack.is_empty() {
                return;
            }
            self.move_into_empty_slot(slot, stack);
        }
    }

    fn insert_into_equipment_slot_if_possible(&mut self, stack: &mut ItemStack) {
        let Some(slot) = matching_equipment_menu_slot(stack) else {
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

pub(crate) fn crafting_recipe_placement(
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

pub(crate) fn biggest_placeable_craft_count(
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

pub(crate) fn find_player_slot_matching(
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

fn matching_equipment_menu_slot(stack: &ItemStack) -> Option<usize> {
    match stack.component("minecraft:equippable") {
        Some(crate::item_properties::ItemComponent::Equippable { slot, .. }) => match slot {
            crate::item_properties::EquipmentSlot::Head => Some(5),
            crate::item_properties::EquipmentSlot::Chest => Some(6),
            crate::item_properties::EquipmentSlot::Legs => Some(7),
            crate::item_properties::EquipmentSlot::Feet => Some(8),
            crate::item_properties::EquipmentSlot::OffHand => Some(45),
            _ => None,
        },
        _ => matching_armor_menu_slot(stack.item_id()),
    }
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
mod tests;
