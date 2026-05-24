use super::*;

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
    recipes: RecipeMap,
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
            recipes,
        }
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn recipe_id(&self) -> Option<&'static str> {
        self.recipe_id
    }

    pub fn slots_changed(&mut self) {
        let items: Vec<Option<&'static str>> = self
            .grid
            .iter()
            .map(|s| (!s.is_empty()).then(|| s.item_id()))
            .collect();
        if let Some(holder) = self.recipes.get_recipe_for("crafting", 3, 3, &items) {
            self.recipe_id = Some(holder.id);
            self.result = holder
                .recipe
                .assemble()
                .map(|r| ItemStack::new(r.item, r.count as i32))
                .unwrap_or_else(ItemStack::empty);
        } else {
            self.recipe_id = None;
            self.result = ItemStack::empty();
        }
    }

    pub fn take_result(&mut self) -> ItemStack {
        if self.result.is_empty() {
            return ItemStack::empty();
        }
        let result = self.result.clone();
        for slot in &mut self.grid {
            if !slot.is_empty() {
                slot.shrink(1);
            }
        }
        self.slots_changed();
        result
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
            self.take_result()
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

    pub fn take_result(&mut self) -> ItemStack {
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
                } else if slot >= Self::INV_START && slot < Self::HOTBAR_START {
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

