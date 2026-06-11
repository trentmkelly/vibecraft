use super::*;
use crate::player_inventory::{
    biggest_placeable_craft_count, crafting_recipe_placement, find_player_slot_matching,
};

// ============================================================================
// CraftingMenu (3x3 crafting table)
// ============================================================================

/// 3×3 crafting table. Layout matches `CraftingMenu`:
/// `result=0`, `grid=1..10`, `player main=10..37`, `hotbar=37..46`.
#[derive(Debug, Clone, PartialEq)]
pub struct CraftingMenu {
    grid: [ItemStack; 9],
    result: ItemStack,
    recipe_id: Option<&'static str>,
    /// For a matched special (`CustomRecipe`) result, the grid state to apply when
    /// the result is taken (see [`SpecialCraftOutcome::grid_after`]). `None` for an
    /// ordinary recipe, whose inputs are consumed via `get_remaining_items`.
    special_grid_after: Option<Vec<ItemStack>>,
    /// The level's map saved-data (scale/exploration) the `MapExtendingRecipe` needs.
    /// Empty by default; the server populates it when constructing the menu.
    map_data: crate::recipe_system::MapDataStore,
    recipes: RecipeMap,
    unlocked_recipes: BTreeSet<&'static str>,
    highlighted_recipes: BTreeSet<&'static str>,
    recipe_unlock_events: Vec<&'static str>,
}

impl CraftingMenu {
    pub const RESULT_SLOT: usize = 0;
    pub const GRID_START: usize = 1;
    pub const GRID_END: usize = 10;
    pub const INV_START: usize = 10;
    pub const INV_END: usize = 37;
    pub const HOTBAR_START: usize = 37;
    pub const HOTBAR_END: usize = 46;
    pub const SLOT_COUNT: usize = 46;
    pub const MENU_SLOTS: usize = 10;

    pub fn new(recipes: RecipeMap) -> Self {
        Self {
            grid: std::array::from_fn(|_| ItemStack::empty()),
            result: ItemStack::empty(),
            recipe_id: None,
            special_grid_after: None,
            map_data: crate::recipe_system::MapDataStore::default(),
            recipes,
            unlocked_recipes: BTreeSet::new(),
            highlighted_recipes: BTreeSet::new(),
            recipe_unlock_events: Vec::new(),
        }
    }

    /// Supply the level's map saved-data so the `MapExtendingRecipe` can read the
    /// centre map's scale/exploration (the server calls this when opening the menu).
    pub fn set_map_data(&mut self, map_data: crate::recipe_system::MapDataStore) {
        self.map_data = map_data;
        self.slots_changed();
    }

    /// `CraftingMenu.removed`: route the carried cursor item, then
    /// `clearContainer(craftSlots)` — the 3x3 grid returns to the player on a normal
    /// close and drops in-world on disconnect. The (virtual) result is not returned.
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
        for slot in &mut self.grid {
            drop_or_place_in_inventory(
                player,
                std::mem::replace(slot, ItemStack::empty()),
                disconnected,
            );
        }
        self.result = ItemStack::empty();
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn recipe_id(&self) -> Option<&'static str> {
        self.recipe_id
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

    pub fn recipe_book_type(&self) -> RecipeBookType {
        RecipeBookType::Crafting
    }

    pub fn place_recipe_from_inventory(
        &mut self,
        recipe_id: &str,
        use_max_items: bool,
        player: &mut PlayerInventory,
    ) -> bool {
        let Some(holder) = self.recipes.by_key(recipe_id) else {
            return false;
        };
        let Some(placement) = crafting_recipe_placement(&holder.recipe, 3, 3) else {
            return false;
        };

        let mut next_menu = self.clone();
        let mut next_player = player.clone();
        for slot in &mut next_menu.grid {
            next_player.place_item_back_in_inventory(std::mem::replace(slot, ItemStack::empty()));
        }
        next_menu.slots_changed();

        let amount = if use_max_items {
            biggest_placeable_craft_count(&next_player, &placement).min(64)
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
                find_player_slot_matching(&next_player, ingredient, amount)
            else {
                return false;
            };
            next_player.remove(player_slot, amount);
            placed[grid_index] = ItemStack::new(item_id, amount);
        }
        for (grid_index, stack) in placed.into_iter().enumerate() {
            next_menu.grid[grid_index] = stack;
        }
        next_menu.slots_changed();
        *self = next_menu;
        *player = next_player;
        true
    }

    pub fn slots_changed(&mut self) {
        let items: Vec<Option<&'static str>> = self
            .grid
            .iter()
            .map(|s| (!s.is_empty()).then(|| s.item_id()))
            .collect();
        if let Some(holder) = self.recipes.get_recipe_for("crafting", 3, 3, &items) {
            self.recipe_id = Some(holder.id);
            // Transmute/Imbue preserve input components; all others build a plain
            // result from the recipe's output item + count.
            self.result = holder
                .recipe
                .component_aware_result(&self.grid)
                .or_else(|| {
                    holder
                        .recipe
                        .assemble()
                        .map(|r| ItemStack::new(r.item, r.count as i32))
                })
                .unwrap_or_else(ItemStack::empty);
            self.special_grid_after = None;
        } else if let Some((id, outcome)) = self.recipes.special_crafting_result(&self.grid) {
            // A `CustomRecipe` (repair, banner duplicate, fireworks, …) whose result
            // depends on input components — evaluated with full stacks.
            self.recipe_id = Some(id);
            self.result = outcome.result;
            self.special_grid_after = Some(outcome.grid_after);
        } else if let Some((id, outcome)) = self
            .recipes
            .map_extending_result(&self.grid, &self.map_data)
        {
            // `MapExtendingRecipe` needs the centre map's saved data (scale/exploration).
            self.recipe_id = Some(id);
            self.result = outcome.result;
            self.special_grid_after = Some(outcome.grid_after);
        } else {
            self.recipe_id = None;
            self.result = ItemStack::empty();
            self.special_grid_after = None;
        }
    }

    pub fn take_result(&mut self) -> ItemStack {
        let Some(recipe_id) = self.recipe_id else {
            return ItemStack::empty();
        };
        if self.result.is_empty() {
            return ItemStack::empty();
        }
        let result = self.result.clone();
        self.consume_inputs_and_refresh();
        if self.unlocked_recipes.insert(recipe_id) {
            self.highlighted_recipes.insert(recipe_id);
            self.recipe_unlock_events.push(recipe_id);
        }
        result
    }

    fn consume_inputs_and_refresh(&mut self) {
        // A special (`CustomRecipe`) result carries its own post-craft grid state
        // (e.g. the patterned banner / written book stays, the copy is consumed).
        if let Some(grid_after) = self.special_grid_after.take() {
            for (slot, after) in self.grid.iter_mut().zip(grid_after) {
                *slot = after;
            }
            self.slots_changed();
            return;
        }

        let remaining_items = self
            .recipe_id
            .and_then(|recipe_id| self.recipes.by_key(recipe_id))
            .map(|holder| {
                let input = self
                    .grid
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
            .unwrap_or_else(|| vec![None; self.grid.len()]);

        for (slot, remainder) in self.grid.iter_mut().zip(remaining_items) {
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
        self.slots_changed();
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.result.clone(),
            1..=9 => self.grid[slot - 1].clone(),
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
            0 => false,
            1..=9 => {
                self.grid[slot - 1] = stack;
                self.slots_changed();
                true
            }
            _ => {
                write_player_slot(slot, Self::INV_START, player, stack);
                true
            }
        }
    }

    pub fn may_place(&self, slot: usize) -> bool {
        slot != 0 && slot < Self::SLOT_COUNT
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.result.clone());
        for stack in &self.grid {
            out.push(stack.clone());
        }
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
        let mut moving = if slot == 0 {
            original.clone()
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
            original.clone()
        };

        let moved = if slot == 0 {
            // Result → player inventory (reverse order matches Java
            // `moveItemStackTo(stack, 10, 46, true)`).
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else if slot >= Self::INV_START {
            // Player inventory → grid → main/hotbar mirror.
            if self.move_into_range(&mut moving, Self::GRID_START, Self::GRID_END, false, player) {
                true
            } else if slot < Self::HOTBAR_START {
                self.move_into_range(
                    &mut moving,
                    Self::HOTBAR_START,
                    Self::HOTBAR_END,
                    false,
                    player,
                )
            } else {
                self.move_into_range(
                    &mut moving,
                    Self::INV_START,
                    Self::HOTBAR_START,
                    false,
                    player,
                )
            }
        } else {
            // Grid slot → player inventory.
            self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            )
        };

        if !moving.is_empty() {
            // Put leftover back where it came from (not for result-slot).
            if slot != 0 {
                self.set_slot(slot, moving, player);
            }
        }
        if !moved {
            return ItemStack::empty();
        }
        if slot == 0 {
            self.take_result();
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
        let (leftover, writes, moved) = plan_move_item_stack_to(
            stack.clone(),
            start,
            end,
            reverse,
            &snapshot,
            |s, _stack| self.may_place(s),
        );
        for w in writes {
            self.set_slot(w.slot, w.new_stack, player);
        }
        *stack = leftover;
        moved
    }
}

// ============================================================================
// AbstractFurnaceMenu (base for Furnace, BlastFurnace, Smoker)
// ============================================================================

/// `RecipeBookType` matching Java enum values used by furnaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeBookType {
    Crafting,
    Furnace,
    BlastFurnace,
    Smoker,
}

/// Distinguishes the three furnace variants for recipe lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnaceKind {
    Furnace,
    BlastFurnace,
    Smoker,
}

impl FurnaceKind {
    pub fn recipe_book_type(self) -> RecipeBookType {
        match self {
            FurnaceKind::Furnace => RecipeBookType::Furnace,
            FurnaceKind::BlastFurnace => RecipeBookType::BlastFurnace,
            FurnaceKind::Smoker => RecipeBookType::Smoker,
        }
    }
}

/// Container data indices for `AbstractFurnaceMenu`.
pub mod furnace_data {
    pub const LIT_TIME: usize = 0;
    pub const LIT_DURATION: usize = 1;
    pub const COOKING_PROGRESS: usize = 2;
    pub const COOKING_TOTAL_TIME: usize = 3;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FurnaceResultAward {
    pub recipe_ids: Vec<String>,
    pub experience: i32,
}

/// Implementation shared by `FurnaceMenu`, `BlastFurnaceMenu`, `SmokerMenu`.
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractFurnaceMenu {
    kind: FurnaceKind,
    input: ItemStack,
    fuel: ItemStack,
    result: ItemStack,
    /// `[litTime, litDuration, cookingProgress, cookingTotalTime]`.
    pub data: [i16; 4],
    fuel_values: FuelValues,
    recipes_used: BTreeMap<String, (i32, i32)>,
    pending_result_award: FurnaceResultAward,
}

impl AbstractFurnaceMenu {
    pub const INGREDIENT_SLOT: usize = 0;
    pub const FUEL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const INV_END: usize = 30;
    pub const HOTBAR_START: usize = 30;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;

    pub fn new(kind: FurnaceKind, fuel_values: FuelValues) -> Self {
        Self {
            kind,
            input: ItemStack::empty(),
            fuel: ItemStack::empty(),
            result: ItemStack::empty(),
            data: [0; 4],
            fuel_values,
            recipes_used: BTreeMap::new(),
            pending_result_award: FurnaceResultAward::default(),
        }
    }

    pub fn kind(&self) -> FurnaceKind {
        self.kind
    }

    pub fn recipe_book_type(&self) -> RecipeBookType {
        self.kind.recipe_book_type()
    }

    pub fn is_fuel(&self, stack: &ItemStack) -> bool {
        !stack.is_empty() && self.fuel_values.is_fuel(stack.item_id())
    }

    pub fn data(&self, index: usize) -> Option<i16> {
        self.data.get(index).copied()
    }

    pub fn set_data(&mut self, index: usize, value: i16) -> bool {
        let Some(slot) = self.data.get_mut(index) else {
            return false;
        };
        *slot = value;
        true
    }

    pub fn burn_progress(&self) -> f32 {
        let current = self.data[furnace_data::COOKING_PROGRESS];
        let total = self.data[furnace_data::COOKING_TOTAL_TIME];
        if current == 0 || total == 0 {
            0.0
        } else {
            (current as f32 / total as f32).clamp(0.0, 1.0)
        }
    }

    pub fn lit_progress(&self) -> f32 {
        let lit_duration = match self.data[furnace_data::LIT_DURATION] {
            0 => 200,
            value => value,
        };
        (self.data[furnace_data::LIT_TIME] as f32 / lit_duration as f32).clamp(0.0, 1.0)
    }

    pub fn is_lit(&self) -> bool {
        self.data[furnace_data::LIT_TIME] > 0
    }

    /// `AbstractFurnaceMenu.canSmelt` — datapack-driven set of input items the
    /// furnace will accept. We delegate to the recipe map.
    pub fn can_smelt(&self, stack: &ItemStack, recipes: &RecipeMap) -> bool {
        if stack.is_empty() {
            return false;
        }
        let recipe_type = match self.kind {
            FurnaceKind::Furnace => "smelting",
            FurnaceKind::BlastFurnace => "blasting",
            FurnaceKind::Smoker => "smoking",
        };
        recipes
            .get_recipe_for(recipe_type, 1, 1, &[Some(stack.item_id())])
            .is_some()
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input.clone(),
            1 => self.fuel.clone(),
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
            0 => self.input = stack,
            1 => self.fuel = stack,
            2 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    /// Allows overwriting the result slot internally (e.g. when smelting
    /// completes or when shift-clicking removes the smelted item).
    pub fn set_result_internal(&mut self, stack: ItemStack) {
        self.result = stack;
    }

    pub fn record_recipe_use(
        &mut self,
        recipe_id: impl Into<String>,
        times_used: i32,
        experience_millis: i32,
    ) {
        if times_used <= 0 {
            return;
        }
        let entry = self
            .recipes_used
            .entry(recipe_id.into())
            .or_insert((0, experience_millis.max(0)));
        entry.0 += times_used;
        entry.1 = experience_millis.max(0);
    }

    pub fn recipes_used(&self) -> &BTreeMap<String, (i32, i32)> {
        &self.recipes_used
    }

    /// Java `FurnaceResultSlot.checkTakeAchievements` delegates to
    /// `AbstractFurnaceBlockEntity.awardUsedRecipesAndPopExperience` whenever
    /// the result slot is taken. The Rust menu has no world object to spawn XP
    /// orbs into, so it exposes the same side effect as a drainable award.
    pub fn drain_result_award(&mut self) -> FurnaceResultAward {
        std::mem::take(&mut self.pending_result_award)
    }

    pub fn take_result(&mut self) -> ItemStack {
        self.take_result_with_xp_roll(1.0)
    }

    pub fn take_result_with_xp_roll(&mut self, fraction_roll: f32) -> ItemStack {
        if !self.result.is_empty() {
            self.queue_result_award(fraction_roll);
        }
        std::mem::replace(&mut self.result, ItemStack::empty())
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => slot < Self::SLOT_COUNT,
            1 => self.is_fuel(stack) || stack.item_id() == "minecraft:bucket",
            2 => false,
            _ => slot < Self::SLOT_COUNT,
        }
    }

    pub fn max_stack_size(&self, slot: usize, stack: &ItemStack) -> i32 {
        if slot == 1 && stack.item_id() == "minecraft:bucket" {
            1
        } else {
            stack.max_stack_size() as i32
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input.clone());
        out.push(self.fuel.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn quick_move(
        &mut self,
        slot: usize,
        recipes: &RecipeMap,
        player: &mut PlayerInventory,
    ) -> ItemStack {
        if slot >= Self::SLOT_COUNT {
            return ItemStack::empty();
        }
        let original = self.get_slot(slot, player).unwrap_or_else(ItemStack::empty);
        if original.is_empty() {
            return ItemStack::empty();
        }
        let mut moving = original.clone();
        // Always remove from source first to mirror Java's `slot.getItem()`
        // being a live reference that `moveItemStackTo` mutates.
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
                if self.can_smelt(&moving, recipes) {
                    self.move_into_range(&mut moving, 0, 1, false, player)
                } else if self.is_fuel(&moving) {
                    self.move_into_range(&mut moving, 1, 2, false, player)
                } else if (Self::INV_START..Self::HOTBAR_START).contains(&slot) {
                    self.move_into_range(
                        &mut moving,
                        Self::HOTBAR_START,
                        Self::HOTBAR_END,
                        false,
                        player,
                    )
                } else {
                    self.move_into_range(
                        &mut moving,
                        Self::INV_START,
                        Self::HOTBAR_START,
                        false,
                        player,
                    )
                }
            }
        };

        let result_count_changed = slot == Self::RESULT_SLOT && moving.count() != original.count();
        if !moving.is_empty() {
            if slot == 2 {
                self.result = moving;
            } else {
                self.set_slot(slot, moving, player);
            }
        }
        if !moved {
            return ItemStack::empty();
        }
        if result_count_changed {
            self.queue_result_award(1.0);
        }
        original
    }

    fn queue_result_award(&mut self, fraction_roll: f32) {
        let award = self.award_used_recipes_and_pop_experience(fraction_roll);
        if award.recipe_ids.is_empty() && award.experience == 0 {
            return;
        }
        self.pending_result_award
            .recipe_ids
            .extend(award.recipe_ids);
        self.pending_result_award.experience += award.experience;
    }

    fn award_used_recipes_and_pop_experience(&mut self, fraction_roll: f32) -> FurnaceResultAward {
        let recipe_ids = self.recipes_used.keys().cloned().collect();
        let experience = self
            .recipes_used
            .values()
            .map(|(times_used, experience_millis)| {
                if *times_used <= 0 || *experience_millis <= 0 {
                    return 0;
                }
                let total_millis = *times_used * *experience_millis;
                let whole = total_millis / 1000;
                let fraction = (total_millis % 1000) as f32 / 1000.0;
                if fraction != 0.0 && fraction_roll < fraction {
                    whole + 1
                } else {
                    whole
                }
            })
            .sum();
        self.recipes_used.clear();
        FurnaceResultAward {
            recipe_ids,
            experience,
        }
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
                0 => self.input = w.new_stack,
                1 => self.fuel = w.new_stack,
                2 => self.result = w.new_stack,
                s => write_player_slot(s, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

/// `FurnaceMenu`.
#[derive(Debug, Clone, PartialEq)]
pub struct FurnaceMenu(pub AbstractFurnaceMenu);

impl FurnaceMenu {
    pub fn new(fuel_values: FuelValues) -> Self {
        Self(AbstractFurnaceMenu::new(FurnaceKind::Furnace, fuel_values))
    }
}

/// `BlastFurnaceMenu`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlastFurnaceMenu(pub AbstractFurnaceMenu);

impl BlastFurnaceMenu {
    pub fn new(fuel_values: FuelValues) -> Self {
        Self(AbstractFurnaceMenu::new(
            FurnaceKind::BlastFurnace,
            fuel_values,
        ))
    }
}

/// `SmokerMenu`.
#[derive(Debug, Clone, PartialEq)]
pub struct SmokerMenu(pub AbstractFurnaceMenu);

impl SmokerMenu {
    pub fn new(fuel_values: FuelValues) -> Self {
        Self(AbstractFurnaceMenu::new(FurnaceKind::Smoker, fuel_values))
    }
}
