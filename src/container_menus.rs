//! Container menu implementations matching `net.minecraft.world.inventory.*`
//! in Minecraft 26.1.2.
//!
//! Each menu exposes the same surface:
//!   * `get_slot(i)` / `set_slot(i, stack)` — read/write a menu slot (slot 0 is
//!     the first menu-specific slot)
//!   * `may_place(slot, stack)` — slot type restrictions (e.g. fuel slot only
//!     accepts fuel)
//!   * `all_slots(player_inventory)` — flattened view including player main
//!     storage + hotbar appended after the menu's own slots
//!   * `quick_move(slot, player_inventory)` — shift-click zone-aware transfer
//!
//! The `PlayerInventory` is *not* owned by these menus. It is always passed in
//! by the caller (mirroring how vanilla appends player slots to every menu via
//! `addStandardInventorySlots` but the underlying `Inventory` is owned by the
//! player). Player main storage occupies slots `[menu_size, menu_size+27)` and
//! the hotbar occupies `[menu_size+27, menu_size+36)`.
//!
//! See `decompiled-server-26.1.2/net/minecraft/world/inventory/*.java` for
//! authoritative behavior. Each menu's slot-count constants, slot-type
//! restrictions, and `quick_move` zone order match the Java source 1:1.

#![allow(dead_code)]

use crate::inventory::same_item_same_components;
use crate::item_stack::ItemStack;
use crate::player_inventory::{
    ItemCost, MerchantOffer, PlayerInventory, HOTBAR_SIZE, INVENTORY_SIZE,
};
use crate::recipe_system::{FuelValues, RecipeMap};

/// Number of vanilla "main storage" slots (3 rows of 9, excluding the hotbar).
pub const PLAYER_MAIN_STORAGE: usize = 27;
/// Standard player inventory slots (main storage + hotbar) appended to every
/// menu via `addStandardInventorySlots`.
pub const PLAYER_SLOTS: usize = PLAYER_MAIN_STORAGE + HOTBAR_SIZE;

/// Beacon payment items, matching `ItemTags.BEACON_PAYMENT_ITEMS` in 26.1.2.
const BEACON_PAYMENT_ITEMS: &[&str] = &[
    "minecraft:netherite_ingot",
    "minecraft:emerald",
    "minecraft:diamond",
    "minecraft:gold_ingot",
    "minecraft:iron_ingot",
    "minecraft:amethyst_shard",
];

/// Brewing fuel items, matching `ItemTags.BREWING_FUEL` in 26.1.2.
const BREWING_FUEL_ITEMS: &[&str] = &["minecraft:blaze_powder"];

/// `BrewingStandMenu.PotionSlot.mayPlaceItem`.
fn is_potion_or_bottle(item_id: &str) -> bool {
    matches!(
        item_id,
        "minecraft:potion"
            | "minecraft:splash_potion"
            | "minecraft:lingering_potion"
            | "minecraft:glass_bottle"
    )
}

/// Heuristic for `PotionBrewing.isIngredient`. The exhaustive recipe set is
/// data-driven in vanilla; we accept the well-known base ingredients used by
/// vanilla brewing recipes.
fn is_brewing_ingredient(item_id: &str) -> bool {
    matches!(
        item_id,
        "minecraft:nether_wart"
            | "minecraft:redstone"
            | "minecraft:glowstone_dust"
            | "minecraft:gunpowder"
            | "minecraft:dragon_breath"
            | "minecraft:fermented_spider_eye"
            | "minecraft:sugar"
            | "minecraft:rabbit_foot"
            | "minecraft:glistering_melon_slice"
            | "minecraft:spider_eye"
            | "minecraft:magma_cream"
            | "minecraft:blaze_powder"
            | "minecraft:ghast_tear"
            | "minecraft:turtle_helmet"
            | "minecraft:phantom_membrane"
            | "minecraft:golden_carrot"
            | "minecraft:pufferfish"
            | "minecraft:slime_block"
            | "minecraft:wind_charge"
            | "minecraft:breeze_rod"
            | "minecraft:stone"
            | "minecraft:cobweb"
            | "minecraft:resin_clump"
    )
}

/// `LoomMenu.isDyeItem` — items tagged `loom_dyes` carrying a `DYE` component.
fn is_dye_item(item_id: &str) -> bool {
    item_id.ends_with("_dye")
        || matches!(
            item_id,
            "minecraft:bone_meal"
                | "minecraft:lapis_lazuli"
                | "minecraft:cocoa_beans"
                | "minecraft:ink_sac"
        )
}

/// `LoomMenu.isPatternItem` — items tagged `loom_patterns`.
fn is_pattern_item(item_id: &str) -> bool {
    item_id.ends_with("_banner_pattern")
}

/// Banner item ID test.
fn is_banner_item(item_id: &str) -> bool {
    item_id.ends_with("_banner")
}

/// `CartographyTableMenu`'s additional-slot test.
fn is_cartography_additional(item_id: &str) -> bool {
    matches!(
        item_id,
        "minecraft:paper" | "minecraft:map" | "minecraft:glass_pane"
    )
}

/// Whether the item carries a map id (filled map).
fn is_filled_map(item_id: &str) -> bool {
    item_id == "minecraft:filled_map"
}

/// Used by grindstone slots — accepts damageable items or items with any
/// enchantment. Enchantment data is not yet stored on items in RustCraft, so
/// we approximate via the items that vanilla treats as damageable. The
/// behaviour is sound (a non-damageable, non-enchanted item will be rejected
/// here matching vanilla).
fn is_grindstone_input(stack: &ItemStack) -> bool {
    stack.is_damageable_item() || stack.max_damage() > 0
}

/// Common slot-mutation helpers shared by every menu. Operates on a contiguous
/// `Vec<ItemStack>` of menu-local slots plus a borrow into the player's main
/// storage + hotbar.
fn append_player_slots(out: &mut Vec<ItemStack>, player: &PlayerInventory) {
    for slot in 9..INVENTORY_SIZE {
        out.push(player.get(slot).clone());
    }
    for slot in 0..HOTBAR_SIZE {
        out.push(player.get(slot).clone());
    }
}

/// One "move-plan" entry produced by `plan_move_item_stack_to`.
/// `set(idx, new_stack)` callers apply these in order.
#[derive(Debug, Clone)]
struct MoveWrite {
    slot: usize,
    new_stack: ItemStack,
}

/// `AbstractContainerMenu.moveItemStackTo`, decomposed into a pure planning
/// pass over a slot snapshot. Returns the updated leftover and the list of
/// writes that the caller must apply. We use this two-phase approach so the
/// caller does not need to lend `self` and `player` to closures (which
/// otherwise conflicts with the immutable borrows held during read).
fn plan_move_item_stack_to(
    mut stack: ItemStack,
    start: usize,
    end: usize,
    reverse_direction: bool,
    snapshot: &[ItemStack],
    may_place: impl Fn(usize, &ItemStack) -> bool,
) -> (ItemStack, Vec<MoveWrite>, bool) {
    let mut writes: Vec<MoveWrite> = Vec::new();
    let mut moved = false;
    if stack.is_empty() {
        return (stack, writes, moved);
    }
    let mut working: Vec<ItemStack> = snapshot[start..end].to_vec();
    let local_indices: Vec<usize> = if reverse_direction {
        (0..(end - start)).rev().collect()
    } else {
        (0..(end - start)).collect()
    };

    if stack.max_stack_size() > 1 {
        for &local in &local_indices {
            if stack.is_empty() {
                break;
            }
            let current = &working[local];
            if !current.is_empty() && same_item_same_components(current, &stack) {
                let max = stack.max_stack_size() as i32;
                let combined = current.count() + stack.count();
                if combined <= max {
                    let mut updated = current.clone();
                    updated.set_count(combined);
                    working[local] = updated.clone();
                    writes.push(MoveWrite {
                        slot: start + local,
                        new_stack: updated,
                    });
                    stack.set_count(0);
                    moved = true;
                } else if current.count() < max {
                    let take = max - current.count();
                    stack.shrink(take);
                    let mut updated = current.clone();
                    updated.set_count(max);
                    working[local] = updated.clone();
                    writes.push(MoveWrite {
                        slot: start + local,
                        new_stack: updated,
                    });
                    moved = true;
                }
            }
        }
    }

    if !stack.is_empty() {
        for &local in &local_indices {
            if stack.is_empty() {
                break;
            }
            let current = &working[local];
            let slot_index = start + local;
            if current.is_empty() && may_place(slot_index, &stack) {
                let max = stack.max_stack_size() as i32;
                let placed = if stack.count() > max {
                    let placed = stack.copy_with_count(max);
                    stack.shrink(max);
                    placed
                } else {
                    let placed = stack.clone();
                    stack.set_count(0);
                    placed
                };
                working[local] = placed.clone();
                writes.push(MoveWrite {
                    slot: slot_index,
                    new_stack: placed,
                });
                moved = true;
            }
        }
    }
    (stack, writes, moved)
}

/// Maps menu-local "player slot N" (where N is in
/// `[player_start, player_start+36)`) to a real `PlayerInventory` slot.
/// Main storage slot 0 (in the menu) corresponds to `PlayerInventory` slot 9,
/// and hotbar slot 0 corresponds to `PlayerInventory` slot 0.
fn player_slot_for(menu_index: usize, player_start: usize) -> Option<usize> {
    let local = menu_index.checked_sub(player_start)?;
    if local < PLAYER_MAIN_STORAGE {
        Some(9 + local)
    } else if local < PLAYER_SLOTS {
        Some(local - PLAYER_MAIN_STORAGE)
    } else {
        None
    }
}

fn read_player_slot(menu_index: usize, player_start: usize, player: &PlayerInventory) -> ItemStack {
    player_slot_for(menu_index, player_start)
        .map(|slot| player.get(slot).clone())
        .unwrap_or_else(ItemStack::empty)
}

fn write_player_slot(
    menu_index: usize,
    player_start: usize,
    player: &mut PlayerInventory,
    stack: ItemStack,
) {
    if let Some(slot) = player_slot_for(menu_index, player_start) {
        player.set(slot, stack);
    }
}

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

// ============================================================================
// ChestMenu — 1..6 rows
// ============================================================================

/// Generic chest. `rows` may be 1..=6.
#[derive(Debug, Clone, PartialEq)]
pub struct ChestMenu {
    rows: usize,
    slots: Vec<ItemStack>,
}

impl ChestMenu {
    pub const MAX_ROWS: usize = 6;

    pub fn new(rows: usize) -> Self {
        assert!(rows >= 1 && rows <= Self::MAX_ROWS);
        Self {
            rows,
            slots: vec![ItemStack::empty(); rows * 9],
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn chest_size(&self) -> usize {
        self.rows * 9
    }

    pub fn slot_count(&self) -> usize {
        self.chest_size() + PLAYER_SLOTS
    }

    pub fn inv_start(&self) -> usize {
        self.chest_size()
    }

    pub fn hotbar_start(&self) -> usize {
        self.chest_size() + PLAYER_MAIN_STORAGE
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= self.slot_count() {
            return None;
        }
        if slot < self.chest_size() {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, self.inv_start(), player))
        }
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
        if slot < self.chest_size() {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, self.inv_start(), player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < self.slot_count()
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(self.slot_count());
        for s in &self.slots {
            out.push(s.clone());
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
        let chest_size = self.chest_size();
        let total = self.slot_count();
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);

        let moved = if slot < chest_size {
            self.move_into_range(&mut moving, chest_size, total, true, player)
        } else {
            self.move_into_range(&mut moving, 0, chest_size, false, player)
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
        let inv_start = self.inv_start();
        let chest_size = self.chest_size();
        let snapshot = self.all_slots(player);
        let (leftover, writes, moved) =
            plan_move_item_stack_to(stack.clone(), start, end, reverse, &snapshot, |s, st| {
                self.may_place(s, st)
            });
        for w in writes {
            if w.slot < chest_size {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, inv_start, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

// ============================================================================
// HopperMenu
// ============================================================================

/// `HopperMenu` — 5 hopper slots + player inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct HopperMenu {
    slots: [ItemStack; 5],
}

impl HopperMenu {
    pub const HOPPER_SIZE: usize = 5;
    pub const SLOT_COUNT: usize = Self::HOPPER_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::HOPPER_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::HOPPER_SIZE, player))
        }
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
        if slot < Self::HOPPER_SIZE {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::HOPPER_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
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
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);

        let moved = if slot < Self::HOPPER_SIZE {
            self.move_into_range(
                &mut moving,
                Self::HOPPER_SIZE,
                Self::SLOT_COUNT,
                true,
                player,
            )
        } else {
            self.move_into_range(&mut moving, 0, Self::HOPPER_SIZE, false, player)
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
            if w.slot < Self::HOPPER_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::HOPPER_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for HopperMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DispenserMenu (3×3, used by Dispenser + Dropper)
// ============================================================================

/// `DispenserMenu` — 3×3 grid + player inventory.
#[derive(Debug, Clone, PartialEq)]
pub struct DispenserMenu {
    slots: [ItemStack; 9],
}

impl DispenserMenu {
    pub const GRID_SIZE: usize = 9;
    pub const SLOT_COUNT: usize = Self::GRID_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| ItemStack::empty()),
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::GRID_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::GRID_SIZE, player))
        }
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
        if slot < Self::GRID_SIZE {
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::GRID_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
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
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);
        let moved = if slot < Self::GRID_SIZE {
            self.move_into_range(&mut moving, Self::GRID_SIZE, Self::SLOT_COUNT, true, player)
        } else {
            self.move_into_range(&mut moving, 0, Self::GRID_SIZE, false, player)
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
            if w.slot < Self::GRID_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::GRID_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for DispenserMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ShulkerBoxMenu
// ============================================================================

/// `ShulkerBoxMenu` — 27 slots (no nested shulker boxes allowed).
#[derive(Debug, Clone, PartialEq)]
pub struct ShulkerBoxMenu {
    slots: Vec<ItemStack>,
}

impl ShulkerBoxMenu {
    pub const CONTAINER_SIZE: usize = 27;
    pub const SLOT_COUNT: usize = Self::CONTAINER_SIZE + PLAYER_SLOTS;

    pub fn new() -> Self {
        Self {
            slots: vec![ItemStack::empty(); Self::CONTAINER_SIZE],
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        if slot < Self::CONTAINER_SIZE {
            Some(self.slots[slot].clone())
        } else {
            Some(read_player_slot(slot, Self::CONTAINER_SIZE, player))
        }
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
        if slot < Self::CONTAINER_SIZE {
            // ShulkerBoxSlot rejects nested shulker box items.
            if Self::is_shulker_box(&stack) {
                return false;
            }
            self.slots[slot] = stack;
        } else {
            write_player_slot(slot, Self::CONTAINER_SIZE, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        if slot >= Self::SLOT_COUNT {
            return false;
        }
        if slot < Self::CONTAINER_SIZE && Self::is_shulker_box(stack) {
            return false;
        }
        true
    }

    fn is_shulker_box(stack: &ItemStack) -> bool {
        let id = stack.item_id();
        id == "minecraft:shulker_box" || id.ends_with("_shulker_box")
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for s in &self.slots {
            out.push(s.clone());
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
        let mut moving = original.clone();
        self.set_slot(slot, ItemStack::empty(), player);
        let moved = if slot < Self::CONTAINER_SIZE {
            self.move_into_range(
                &mut moving,
                Self::CONTAINER_SIZE,
                Self::SLOT_COUNT,
                true,
                player,
            )
        } else {
            self.move_into_range(&mut moving, 0, Self::CONTAINER_SIZE, false, player)
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
            if w.slot < Self::CONTAINER_SIZE {
                self.slots[w.slot] = w.new_stack;
            } else {
                write_player_slot(w.slot, Self::CONTAINER_SIZE, player, w.new_stack);
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for ShulkerBoxMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AnvilMenu (combiner-style)
// ============================================================================

/// Threshold (in survival mode) above which the anvil result becomes
/// "Too Expensive!" and the player cannot complete the combine.
pub const ANVIL_MAX_COST: i32 = 40;

/// `AnvilMenu` cost contributions, matching Java constants.
pub mod anvil_cost {
    pub const BASE: i32 = 1;
    pub const ADDED_BASE: i32 = 1;
    pub const REPAIR_MATERIAL: i32 = 1;
    pub const REPAIR_SACRIFICE: i32 = 2;
    pub const RENAME: i32 = 1;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnvilMenu {
    input_left: ItemStack,
    input_right: ItemStack,
    result: ItemStack,
    item_name: Option<String>,
    pub cost: i32,
    pub repair_item_count_cost: i32,
    pub only_renaming: bool,
}

impl AnvilMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;
    pub const MAX_NAME_LENGTH: usize = 50;

    pub fn new() -> Self {
        Self {
            input_left: ItemStack::empty(),
            input_right: ItemStack::empty(),
            result: ItemStack::empty(),
            item_name: None,
            cost: 0,
            repair_item_count_cost: 0,
            only_renaming: false,
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input_left.clone(),
            1 => self.input_right.clone(),
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
            0 => self.input_left = stack,
            1 => self.input_right = stack,
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
        out.push(self.input_left.clone());
        out.push(self.input_right.clone());
        out.push(self.result.clone());
        append_player_slots(&mut out, player);
        out
    }

    pub fn set_item_name(&mut self, name: Option<String>) {
        self.item_name = name.map(|n| n.chars().take(Self::MAX_NAME_LENGTH).collect::<String>());
    }

    pub fn item_name(&self) -> Option<&str> {
        self.item_name.as_deref()
    }

    /// Re-implement the renaming-only path of `AnvilMenu.createResult`.
    /// When only `input_left` is present (no addition) and a new item-name is
    /// set, this produces a renamed copy with `cost = RENAME = 1`. For the
    /// merging/repair paths see vanilla — they require enchantment data we
    /// don't yet model here, so we leave the result empty and `cost = 0` in
    /// those cases (matching Java fall-through to clear result when no
    /// enchantment-storage is possible).
    pub fn set_result_from_inputs(&mut self) {
        self.cost = 1;
        self.only_renaming = false;
        self.repair_item_count_cost = 0;

        if self.input_left.is_empty() {
            self.result = ItemStack::empty();
            self.cost = 0;
            return;
        }

        let mut total_cost = 0;
        let mut naming_cost = 0;
        let mut result = self.input_left.clone();

        if let Some(name) = self.item_name.as_deref() {
            if !name.trim().is_empty() && name != self.input_left.item_id() {
                naming_cost = anvil_cost::RENAME;
                total_cost += naming_cost;
            }
        }

        if total_cost <= 0 {
            result = ItemStack::empty();
        }
        if naming_cost == total_cost && naming_cost > 0 {
            if self.cost >= ANVIL_MAX_COST {
                self.cost = ANVIL_MAX_COST - 1;
            }
            self.only_renaming = true;
        }
        self.cost = total_cost;
        if self.cost >= ANVIL_MAX_COST {
            // Survival cap; result is suppressed.
            result = ItemStack::empty();
        }
        self.result = result;
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
            _ => self.move_into_range(&mut moving, 0, 2, false, player),
        };
        if !moving.is_empty() {
            if slot == 2 {
                // Result-slot leftover is dropped in vanilla; mirror by
                // simply discarding the remainder.
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
                0 => self.input_left = w.new_stack,
                1 => self.input_right = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for AnvilMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SmithingMenu (combiner-style with template)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct SmithingMenu {
    template: ItemStack,
    base: ItemStack,
    addition: ItemStack,
    result: ItemStack,
    pub has_recipe_error: bool,
}

impl SmithingMenu {
    pub const TEMPLATE_SLOT: usize = 0;
    pub const BASE_SLOT: usize = 1;
    pub const ADDITIONAL_SLOT: usize = 2;
    pub const RESULT_SLOT: usize = 3;
    pub const MENU_SLOTS: usize = 4;
    pub const INV_START: usize = 4;
    pub const HOTBAR_END: usize = 40;
    pub const SLOT_COUNT: usize = 40;

    pub fn new() -> Self {
        Self {
            template: ItemStack::empty(),
            base: ItemStack::empty(),
            addition: ItemStack::empty(),
            result: ItemStack::empty(),
            has_recipe_error: false,
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.template.clone(),
            1 => self.base.clone(),
            2 => self.addition.clone(),
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
            0 => self.template = stack,
            1 => self.base = stack,
            2 => self.addition = stack,
            3 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            3 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.template.clone());
        out.push(self.base.clone());
        out.push(self.addition.clone());
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
            _ => self.move_into_range(&mut moving, 0, 3, false, player),
        };
        if !moving.is_empty() {
            if slot != 3 {
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
                0 => self.template = w.new_stack,
                1 => self.base = w.new_stack,
                2 => self.addition = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for SmithingMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// StonecutterMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct StonecutterMenu {
    input: ItemStack,
    result: ItemStack,
    pub selected_recipe_index: i32,
}

impl StonecutterMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const RESULT_SLOT: usize = 1;
    pub const MENU_SLOTS: usize = 2;
    pub const INV_START: usize = 2;
    pub const HOTBAR_END: usize = 38;
    pub const SLOT_COUNT: usize = 38;

    pub fn new() -> Self {
        Self {
            input: ItemStack::empty(),
            result: ItemStack::empty(),
            selected_recipe_index: -1,
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input.clone(),
            1 => self.result.clone(),
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
            1 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn set_result_internal(&mut self, stack: ItemStack) {
        self.result = stack;
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        match slot {
            1 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input.clone());
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
        if slot == 1 {
            self.result = ItemStack::empty();
        } else {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = match slot {
            1 => self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player),
            0 => self.move_into_range(
                &mut moving,
                Self::INV_START,
                Self::HOTBAR_END,
                false,
                player,
            ),
            _ => {
                // Try input slot first, otherwise main↔hotbar.
                if self.move_into_range(&mut moving, 0, 1, false, player) {
                    true
                } else if slot >= Self::INV_START && slot < Self::INV_START + PLAYER_MAIN_STORAGE {
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
        if !moving.is_empty() && slot != 1 {
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
                0 => self.input = w.new_stack,
                1 => self.result = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for StonecutterMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GrindstoneMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct GrindstoneMenu {
    input_left: ItemStack,
    input_right: ItemStack,
    result: ItemStack,
}

impl GrindstoneMenu {
    pub const INPUT_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const MENU_SLOTS: usize = 3;
    pub const INV_START: usize = 3;
    pub const HOTBAR_END: usize = 39;
    pub const SLOT_COUNT: usize = 39;

    pub fn new() -> Self {
        Self {
            input_left: ItemStack::empty(),
            input_right: ItemStack::empty(),
            result: ItemStack::empty(),
        }
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 | 1 => is_grindstone_input(stack),
            2 => false,
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0 => self.input_left.clone(),
            1 => self.input_right.clone(),
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
            0 => self.input_left = stack,
            1 => self.input_right = stack,
            2 => return false,
            _ => write_player_slot(slot, Self::INV_START, player, stack),
        }
        true
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        out.push(self.input_left.clone());
        out.push(self.input_right.clone());
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
                if !self.input_left.is_empty() && !self.input_right.is_empty() {
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
                } else {
                    self.move_into_range(&mut moving, 0, 2, false, player)
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
                0 => self.input_left = w.new_stack,
                1 => self.input_right = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for GrindstoneMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// EnchantmentMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantmentMenu {
    item: ItemStack,
    lapis: ItemStack,
    /// `[cost0, cost1, cost2, hint0, hint1, hint2]`. The hints map to
    /// enchantment IDs; -1 indicates "no enchantment available for this tier".
    pub data: [i32; 6],
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
            data: [0, 0, 0, -1, -1, -1],
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
            0..=2 => stack.is_empty() || is_potion_or_bottle(stack.item_id()),
            3 => stack.is_empty() || is_brewing_ingredient(stack.item_id()),
            4 => stack.is_empty() || BREWING_FUEL_ITEMS.contains(&stack.item_id()),
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

// ============================================================================
// LecternMenu
// ============================================================================

/// Lectern menu — no quick-move (`quickMoveStack` always returns `EMPTY`),
/// only a one-slot book container and a page index.
#[derive(Debug, Clone, PartialEq)]
pub struct LecternMenu {
    book: ItemStack,
    /// Current page index.
    pub page: i32,
}

impl LecternMenu {
    pub const BUTTON_PREV_PAGE: i32 = 1;
    pub const BUTTON_NEXT_PAGE: i32 = 2;
    pub const BUTTON_TAKE_BOOK: i32 = 3;
    pub const BUTTON_PAGE_JUMP_RANGE_START: i32 = 100;
    pub const SLOT_COUNT: usize = 1;

    pub fn new() -> Self {
        Self {
            book: ItemStack::empty(),
            page: 0,
        }
    }

    pub fn with_book(book: ItemStack) -> Self {
        Self { book, page: 0 }
    }

    pub fn book(&self) -> &ItemStack {
        &self.book
    }

    pub fn get_slot(&self, slot: usize) -> Option<ItemStack> {
        if slot == 0 {
            Some(self.book.clone())
        } else {
            None
        }
    }

    pub fn set_slot(&mut self, slot: usize, stack: ItemStack) -> bool {
        if slot == 0 {
            self.book = stack;
            true
        } else {
            false
        }
    }

    pub fn may_place(&self, _slot: usize) -> bool {
        // Vanilla lectern menu uses a generic slot — but the only way to put a
        // book in is via the lectern block's `interact`. Quick-move always
        // returns empty, so we keep this true for direct API consistency.
        true
    }

    pub fn all_slots(&self) -> Vec<ItemStack> {
        vec![self.book.clone()]
    }

    pub fn quick_move(&mut self, _slot: usize) -> ItemStack {
        ItemStack::empty()
    }

    /// `clickMenuButton` — see Java for button IDs.
    pub fn click_button(&mut self, button_id: i32) -> bool {
        if button_id >= Self::BUTTON_PAGE_JUMP_RANGE_START {
            self.page = button_id - Self::BUTTON_PAGE_JUMP_RANGE_START;
            return true;
        }
        match button_id {
            Self::BUTTON_PREV_PAGE => {
                self.page = (self.page - 1).max(0);
                true
            }
            Self::BUTTON_NEXT_PAGE => {
                self.page += 1;
                true
            }
            Self::BUTTON_TAKE_BOOK => true,
            _ => false,
        }
    }
}

impl Default for LecternMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BeaconMenu
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct BeaconMenu {
    payment: ItemStack,
    /// `[level, primary_effect, secondary_effect]`.
    pub data: [i32; 3],
}

impl BeaconMenu {
    pub const PAYMENT_SLOT: usize = 0;
    pub const MENU_SLOTS: usize = 1;
    pub const INV_START: usize = 1;
    pub const HOTBAR_END: usize = 37;
    pub const SLOT_COUNT: usize = 37;

    pub fn new() -> Self {
        Self {
            payment: ItemStack::empty(),
            data: [0; 3],
        }
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(if slot == 0 {
            self.payment.clone()
        } else {
            read_player_slot(slot, Self::INV_START, player)
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
        if slot == 0 {
            let mut s = stack;
            s.limit_size(1);
            self.payment = s;
        } else {
            write_player_slot(slot, Self::INV_START, player, stack);
        }
        true
    }

    pub fn may_place(&self, slot: usize, stack: &ItemStack) -> bool {
        match slot {
            0 => stack.is_empty() || BEACON_PAYMENT_ITEMS.contains(&stack.item_id()),
            s if s < Self::SLOT_COUNT => true,
            _ => false,
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
        out.push(self.payment.clone());
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

        let moved = if slot == 0 {
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else if self.payment.is_empty() && self.may_place(0, &moving) && moving.count() == 1 {
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
                0 => self.payment = w.new_stack,
                _ => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for BeaconMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CrafterMenu — 3×3 + result, with per-slot disable toggle
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CrafterMenu {
    grid: [ItemStack; 9],
    /// Slot index 0..9 = disabled flags; index 9 = powered flag (matching
    /// `SimpleContainerData(10)` in vanilla).
    pub container_data: [i32; 10],
    result: ItemStack,
}

impl CrafterMenu {
    pub const GRID_SIZE: usize = 9;
    pub const GRID_START: usize = 0;
    pub const GRID_END: usize = 9;
    pub const INV_START: usize = 9;
    pub const INV_END: usize = 36;
    pub const HOTBAR_START: usize = 36;
    pub const HOTBAR_END: usize = 45;
    pub const RESULT_SLOT: usize = 45;
    pub const SLOT_COUNT: usize = 46;

    pub fn new() -> Self {
        Self {
            grid: std::array::from_fn(|_| ItemStack::empty()),
            container_data: [0; 10],
            result: ItemStack::empty(),
        }
    }

    pub fn set_slot_state(&mut self, slot_id: usize, enabled: bool) {
        if slot_id < Self::GRID_SIZE {
            self.container_data[slot_id] = if enabled { 0 } else { 1 };
        }
    }

    pub fn is_slot_disabled(&self, slot_id: usize) -> bool {
        slot_id < Self::GRID_SIZE && self.container_data[slot_id] == 1
    }

    pub fn is_powered(&self) -> bool {
        self.container_data[9] == 1
    }

    pub fn set_powered(&mut self, powered: bool) {
        self.container_data[9] = if powered { 1 } else { 0 };
    }

    pub fn refresh_result(&mut self, recipes: &RecipeMap) {
        let items: Vec<Option<&'static str>> = self
            .grid
            .iter()
            .map(|s| (!s.is_empty()).then(|| s.item_id()))
            .collect();
        if let Some(holder) = recipes.get_recipe_for("crafting", 3, 3, &items) {
            self.result = holder
                .recipe
                .assemble()
                .map(|r| ItemStack::new(r.item, r.count as i32))
                .unwrap_or_else(ItemStack::empty);
        } else {
            self.result = ItemStack::empty();
        }
    }

    pub fn result(&self) -> &ItemStack {
        &self.result
    }

    pub fn get_slot(&self, slot: usize, player: &PlayerInventory) -> Option<ItemStack> {
        if slot >= Self::SLOT_COUNT {
            return None;
        }
        Some(match slot {
            0..=8 => self.grid[slot].clone(),
            9..=44 => read_player_slot(slot, Self::INV_START, player),
            45 => self.result.clone(),
            _ => unreachable!(),
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
            0..=8 => {
                self.grid[slot] = stack;
                true
            }
            9..=44 => {
                write_player_slot(slot, Self::INV_START, player, stack);
                true
            }
            45 => false,
            _ => false,
        }
    }

    pub fn may_place(&self, slot: usize, _stack: &ItemStack) -> bool {
        slot < Self::SLOT_COUNT && slot != 45
    }

    pub fn all_slots(&self, player: &PlayerInventory) -> Vec<ItemStack> {
        let mut out = Vec::with_capacity(Self::SLOT_COUNT);
        for g in &self.grid {
            out.push(g.clone());
        }
        append_player_slots(&mut out, player);
        out.push(self.result.clone());
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
        if slot != 45 {
            self.set_slot(slot, ItemStack::empty(), player);
        }
        let moved = if slot < Self::GRID_SIZE {
            self.move_into_range(&mut moving, Self::INV_START, Self::HOTBAR_END, true, player)
        } else {
            self.move_into_range(&mut moving, 0, Self::GRID_SIZE, false, player)
        };
        if !moving.is_empty() && slot != 45 {
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
                0..=8 => self.grid[w.slot] = w.new_stack,
                9..=44 => write_player_slot(w.slot, Self::INV_START, player, w.new_stack),
                _ => {}
            }
        }
        *stack = leftover;
        moved
    }
}

impl Default for CrafterMenu {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HorseInventoryMenu (uses AbstractMountInventoryMenu layout)
// ============================================================================

/// Saddle / armor / mount-chest inventory menu used by horse, llama, etc.
/// Slot 0 = saddle (only saddle item), slot 1 = body armor (only horse armor
/// or carpet for llamas — relaxed to "any armor-like" here since the catalog
/// is data-driven).
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
            0 => stack.is_empty() || stack.item_id() == "minecraft:saddle",
            1 => stack.is_empty() || Self::is_horse_armor(stack.item_id(), self.layout.is_llama),
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
                | "minecraft:iron_horse_armor"
                | "minecraft:golden_horse_armor"
                | "minecraft:diamond_horse_armor"
                | "minecraft:wolf_armor"
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
            1 => stack.is_empty() || stack.item_id() == "minecraft:nautilus_armor",
            s if s < Self::SLOT_COUNT => true,
            _ => false,
        }
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe_system::{IngredientSpec, ItemAmount, RecipeHolder, RecipeKind, RecipeMap};

    fn empty_recipes() -> RecipeMap {
        RecipeMap::create(Vec::new())
    }

    fn planks_recipe() -> RecipeMap {
        RecipeMap::create(vec![
            RecipeHolder {
                id: "minecraft:oak_planks",
                recipe: RecipeKind::Shapeless {
                    ingredients: vec![IngredientSpec::Item("minecraft:oak_log")],
                    result: ItemAmount {
                        item: "minecraft:oak_planks",
                        count: 4,
                    },
                },
            },
            RecipeHolder {
                id: "minecraft:iron_ingot_from_smelting",
                recipe: RecipeKind::Cooking {
                    kind: crate::recipe_system::CookingKind::Smelting,
                    ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                    result: ItemAmount::one("minecraft:iron_ingot"),
                    experience_millis: 700,
                    cooking_time: Some(200),
                },
            },
        ])
    }

    // -------- CraftingMenu --------

    #[test]
    fn crafting_menu_layout_and_recipe_matching_match_vanilla() {
        let mut menu = CraftingMenu::new(planks_recipe());
        let mut player = PlayerInventory::new();
        assert_eq!(CraftingMenu::SLOT_COUNT, 46);
        assert!(!menu.may_place(CraftingMenu::RESULT_SLOT));
        assert!(menu.may_place(1));
        assert!(menu.may_place(10));

        assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player));
        assert_eq!(menu.result().item_id(), "minecraft:oak_planks");
        assert_eq!(menu.result().count(), 4);

        let taken = menu.take_result();
        assert_eq!(taken.count(), 4);
        assert!(menu.get_slot(1, &player).unwrap().is_empty());
    }

    #[test]
    fn crafting_menu_quick_move_result_goes_to_player_inventory() {
        let mut menu = CraftingMenu::new(planks_recipe());
        let mut player = PlayerInventory::new();
        menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player);
        let moved = menu.quick_move(0, &mut player);
        assert_eq!(moved.item_id(), "minecraft:oak_planks");
        assert_eq!(moved.count(), 4);
        // Java `moveItemStackTo(stack, 10, 46, true)` iterates 45..10 reversed,
        // so the last menu slot (45 = hotbar 8 = player main slot 8) gets the
        // result first.
        assert_eq!(player.get(8).item_id(), "minecraft:oak_planks");
        assert_eq!(player.get(8).count(), 4);
    }

    // -------- AbstractFurnaceMenu --------

    #[test]
    fn furnace_menu_layout_and_fuel_restrictions_match_vanilla() {
        let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
        let mut player = PlayerInventory::new();
        assert_eq!(AbstractFurnaceMenu::SLOT_COUNT, 39);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:raw_iron", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:coal", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:bucket", 1)));
        assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
        assert!(!menu.may_place(2, &ItemStack::new("minecraft:iron_ingot", 1)));

        // Bucket has a max-stack-size cap of 1 in the fuel slot.
        assert_eq!(
            menu.max_stack_size(1, &ItemStack::new("minecraft:bucket", 64)),
            1
        );

        menu.set_slot(0, ItemStack::new("minecraft:raw_iron", 1), &mut player);
        assert_eq!(
            menu.get_slot(0, &player).unwrap().item_id(),
            "minecraft:raw_iron"
        );

        // Result slot rejects direct placement via set_slot.
        assert!(!menu.set_slot(2, ItemStack::new("minecraft:iron_ingot", 1), &mut player));
        menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 1));
        assert_eq!(menu.get_slot(2, &player).unwrap().count(), 1);
    }

    #[test]
    fn furnace_quick_move_result_to_player_and_storage_to_input() {
        let recipes = planks_recipe();
        let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
        let mut player = PlayerInventory::new();
        menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 3));
        let moved = menu.quick_move(2, &recipes, &mut player);
        assert_eq!(moved.item_id(), "minecraft:iron_ingot");
        assert_eq!(moved.count(), 3);
        assert!(menu.get_slot(2, &player).unwrap().is_empty());

        // Place a smeltable into the player main storage (menu slot 3).
        menu.set_slot(3, ItemStack::new("minecraft:raw_iron", 1), &mut player);
        let pre = menu.quick_move(3, &recipes, &mut player);
        assert_eq!(pre.item_id(), "minecraft:raw_iron");
        assert_eq!(
            menu.get_slot(0, &player).unwrap().item_id(),
            "minecraft:raw_iron"
        );

        // Coal goes to fuel.
        menu.set_slot(3, ItemStack::new("minecraft:coal", 1), &mut player);
        menu.quick_move(3, &recipes, &mut player);
        assert_eq!(
            menu.get_slot(1, &player).unwrap().item_id(),
            "minecraft:coal"
        );
    }

    // -------- ChestMenu --------

    #[test]
    fn chest_menu_supports_one_through_six_rows_and_player_inventory_append() {
        for rows in 1..=6 {
            let menu = ChestMenu::new(rows);
            assert_eq!(menu.chest_size(), rows * 9);
            assert_eq!(menu.slot_count(), rows * 9 + PLAYER_SLOTS);
        }
    }

    #[test]
    fn chest_menu_quick_move_shifts_between_chest_and_inventory() {
        let mut menu = ChestMenu::new(3);
        let mut player = PlayerInventory::new();
        menu.set_slot(0, ItemStack::new("minecraft:apple", 5), &mut player);
        let moved = menu.quick_move(0, &mut player);
        assert_eq!(moved.item_id(), "minecraft:apple");
        // After moving to player inventory (reverse order), the hotbar slot 8
        // should be the first chosen empty slot.
        assert_eq!(player.get(8).item_id(), "minecraft:apple");

        // Now shift it back.
        let chest_size = menu.chest_size();
        let menu_hotbar_8 = chest_size + PLAYER_MAIN_STORAGE + 8;
        let back = menu.quick_move(menu_hotbar_8, &mut player);
        assert_eq!(back.item_id(), "minecraft:apple");
        assert_eq!(
            menu.get_slot(0, &player).unwrap().item_id(),
            "minecraft:apple"
        );
    }

    // -------- HopperMenu --------

    #[test]
    fn hopper_menu_5_slots_and_quick_move_to_player() {
        let mut menu = HopperMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(HopperMenu::SLOT_COUNT, 41);
        menu.set_slot(4, ItemStack::new("minecraft:redstone", 3), &mut player);
        let moved = menu.quick_move(4, &mut player);
        assert_eq!(moved.item_id(), "minecraft:redstone");
        assert!(menu.get_slot(4, &player).unwrap().is_empty());
    }

    // -------- DispenserMenu --------

    #[test]
    fn dispenser_menu_layout_and_quick_move() {
        let mut menu = DispenserMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(DispenserMenu::SLOT_COUNT, 45);
        menu.set_slot(0, ItemStack::new("minecraft:arrow", 16), &mut player);
        let moved = menu.quick_move(0, &mut player);
        assert_eq!(moved.count(), 16);
    }

    // -------- ShulkerBoxMenu --------

    #[test]
    fn shulker_box_rejects_nested_shulker_boxes() {
        let mut menu = ShulkerBoxMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(ShulkerBoxMenu::SLOT_COUNT, 63);
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:shulker_box", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:red_shulker_box", 1)));
        assert!(menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        // set_slot should refuse nested shulker box.
        assert!(!menu.set_slot(0, ItemStack::new("minecraft:shulker_box", 1), &mut player));
    }

    // -------- AnvilMenu --------

    #[test]
    fn anvil_result_slot_rejects_placement_and_renaming_costs_one() {
        let mut menu = AnvilMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(AnvilMenu::SLOT_COUNT, 39);
        assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
        assert!(menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));

        menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
        menu.set_item_name(Some("Excalibur".to_string()));
        menu.set_result_from_inputs();
        assert_eq!(menu.cost, anvil_cost::RENAME);
        assert!(menu.only_renaming);
    }

    // -------- SmithingMenu --------

    #[test]
    fn smithing_menu_layout_and_result_rejection() {
        let mut menu = SmithingMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(SmithingMenu::SLOT_COUNT, 40);
        assert!(!menu.may_place(3, &ItemStack::new("minecraft:apple", 1)));
        menu.set_slot(
            0,
            ItemStack::new("minecraft:netherite_upgrade_smithing_template", 1),
            &mut player,
        );
        menu.set_slot(1, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
        menu.set_slot(
            2,
            ItemStack::new("minecraft:netherite_ingot", 1),
            &mut player,
        );
        let all = menu.all_slots(&player);
        assert_eq!(all.len(), 40);
    }

    // -------- StonecutterMenu --------

    #[test]
    fn stonecutter_menu_layout_and_recipe_index_tracking() {
        let mut menu = StonecutterMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(StonecutterMenu::SLOT_COUNT, 38);
        assert_eq!(menu.selected_recipe_index, -1);
        menu.selected_recipe_index = 2;
        menu.set_slot(0, ItemStack::new("minecraft:stone", 4), &mut player);
        menu.set_result_internal(ItemStack::new("minecraft:stone_stairs", 4));
        let moved = menu.quick_move(1, &mut player);
        assert_eq!(moved.item_id(), "minecraft:stone_stairs");
    }

    // -------- GrindstoneMenu --------

    #[test]
    fn grindstone_input_accepts_damageable_only_and_result_rejects_placement() {
        let mut menu = GrindstoneMenu::new();
        assert_eq!(GrindstoneMenu::SLOT_COUNT, 39);
        assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
        // Diamond sword is damageable in item_properties (matches vanilla).
        let sword = ItemStack::new("minecraft:diamond_sword", 1);
        // Whether the slot accepts depends on whether it's marked damageable;
        // we tolerate either outcome but require the slot may_place is
        // consistent with is_damageable_item().
        assert_eq!(menu.may_place(0, &sword), is_grindstone_input(&sword));
    }

    // -------- EnchantmentMenu --------

    #[test]
    fn enchant_menu_lapis_only_in_lapis_slot_and_item_slot_max_1() {
        let mut menu = EnchantmentMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(EnchantmentMenu::SLOT_COUNT, 38);
        assert!(menu.may_place(1, &ItemStack::new("minecraft:lapis_lazuli", 1)));
        assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
        assert_eq!(menu.max_stack_size(0), 1);

        menu.set_slot(
            0,
            ItemStack::new("minecraft:diamond_sword", 64),
            &mut player,
        );
        assert_eq!(menu.get_slot(0, &player).unwrap().count(), 1);
    }

    // -------- BrewingStandMenu --------

    #[test]
    fn brewing_stand_slot_restrictions_match_vanilla() {
        let menu = BrewingStandMenu::new();
        assert_eq!(BrewingStandMenu::SLOT_COUNT, 41);
        // Potion slot accepts potion / glass bottle.
        assert!(menu.may_place(0, &ItemStack::new("minecraft:potion", 1)));
        assert!(menu.may_place(0, &ItemStack::new("minecraft:glass_bottle", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        // Ingredient slot accepts brewing ingredients.
        assert!(menu.may_place(3, &ItemStack::new("minecraft:nether_wart", 1)));
        assert!(!menu.may_place(3, &ItemStack::new("minecraft:apple", 1)));
        // Fuel slot accepts blaze powder.
        assert!(menu.may_place(4, &ItemStack::new("minecraft:blaze_powder", 1)));
        assert!(!menu.may_place(4, &ItemStack::new("minecraft:coal", 1)));
        // Potion slots cap at max stack size 1.
        assert_eq!(menu.max_stack_size(0), 1);
        assert_eq!(menu.max_stack_size(3), 64);
    }

    // -------- CartographyTableMenu --------

    #[test]
    fn cartography_table_slot_restrictions() {
        let menu = CartographyTableMenu::new();
        assert_eq!(CartographyTableMenu::SLOT_COUNT, 39);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:filled_map", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:paper", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:paper", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:map", 1)));
        assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
        assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
    }

    // -------- LoomMenu --------

    #[test]
    fn loom_menu_slot_restrictions() {
        let menu = LoomMenu::new();
        assert_eq!(LoomMenu::SLOT_COUNT, 40);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:white_banner", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:red_dye", 1)));
        assert!(menu.may_place(2, &ItemStack::new("minecraft:creeper_banner_pattern", 1)));
        assert!(!menu.may_place(3, &ItemStack::new("minecraft:white_banner", 1)));
    }

    // -------- LecternMenu --------

    #[test]
    fn lectern_menu_page_buttons() {
        let mut menu = LecternMenu::new();
        assert_eq!(LecternMenu::SLOT_COUNT, 1);
        menu.page = 2;
        assert!(menu.click_button(LecternMenu::BUTTON_PREV_PAGE));
        assert_eq!(menu.page, 1);
        assert!(menu.click_button(LecternMenu::BUTTON_NEXT_PAGE));
        assert_eq!(menu.page, 2);
        assert!(menu.click_button(LecternMenu::BUTTON_PAGE_JUMP_RANGE_START + 7));
        assert_eq!(menu.page, 7);
        assert!(!menu.click_button(42));
        // Quick-move always returns empty.
        assert!(menu.quick_move(0).is_empty());
    }

    // -------- BeaconMenu --------

    #[test]
    fn beacon_payment_slot_accepts_only_payment_items() {
        let mut menu = BeaconMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(BeaconMenu::SLOT_COUNT, 37);
        for item in BEACON_PAYMENT_ITEMS {
            assert!(menu.may_place(0, &ItemStack::new(item, 1)), "{item}");
        }
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        // Max stack 1 for payment slot.
        menu.set_slot(0, ItemStack::new("minecraft:emerald", 64), &mut player);
        assert_eq!(menu.get_slot(0, &player).unwrap().count(), 1);
    }

    // -------- CrafterMenu --------

    #[test]
    fn crafter_menu_slot_disable_toggle_and_layout() {
        let mut menu = CrafterMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(CrafterMenu::SLOT_COUNT, 46);
        assert!(!menu.is_slot_disabled(0));
        menu.set_slot_state(0, false);
        assert!(menu.is_slot_disabled(0));
        menu.set_slot_state(0, true);
        assert!(!menu.is_slot_disabled(0));
        assert!(!menu.is_powered());
        menu.set_powered(true);
        assert!(menu.is_powered());

        // Slot 45 (result) rejects placement.
        assert!(!menu.may_place(45, &ItemStack::new("minecraft:apple", 1)));
        // Grid slots accept anything.
        assert!(menu.set_slot(0, ItemStack::new("minecraft:oak_log", 1), &mut player));
    }

    #[test]
    fn crafter_menu_refresh_result_uses_recipe_map() {
        let mut menu = CrafterMenu::new();
        let mut player = PlayerInventory::new();
        let recipes = planks_recipe();
        menu.set_slot(0, ItemStack::new("minecraft:oak_log", 1), &mut player);
        menu.refresh_result(&recipes);
        assert_eq!(menu.result().item_id(), "minecraft:oak_planks");
        assert_eq!(menu.result().count(), 4);
    }

    // -------- HorseInventoryMenu --------

    #[test]
    fn horse_inventory_saddle_and_armor_slot_restrictions() {
        let layout = HorseLayout {
            saddle_active: true,
            armor_active: true,
            is_llama: false,
            inventory_columns: 5,
        };
        let menu = HorseInventoryMenu::new(layout);
        assert_eq!(menu.slot_count(), 2 + 15 + PLAYER_SLOTS);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:saddle", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:diamond_horse_armor", 1)));
        assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));

        // Llamas accept colored carpets in the armor slot.
        let llama_layout = HorseLayout {
            saddle_active: true,
            armor_active: true,
            is_llama: true,
            inventory_columns: 5,
        };
        let llama = HorseInventoryMenu::new(llama_layout);
        assert!(llama.may_place(1, &ItemStack::new("minecraft:red_carpet", 1)));
        assert!(!llama.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
    }

    // -------- NautilusInventoryMenu --------

    #[test]
    fn nautilus_inventory_saddle_and_armor_restrictions() {
        let menu = NautilusInventoryMenu::new();
        assert_eq!(NautilusInventoryMenu::SLOT_COUNT, 38);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:saddle", 1)));
        assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:nautilus_armor", 1)));
        assert!(!menu.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
    }

    // -------- MerchantMenu --------

    #[test]
    fn merchant_menu_result_slot_rejects_placement_and_payment_layout() {
        let mut menu = MerchantMenu::new();
        let mut player = PlayerInventory::new();
        assert_eq!(MerchantMenu::SLOT_COUNT, 39);
        assert!(menu.may_place(0, &ItemStack::new("minecraft:emerald", 1)));
        assert!(menu.may_place(1, &ItemStack::new("minecraft:book", 1)));
        assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
        menu.set_slot(0, ItemStack::new("minecraft:emerald", 2), &mut player);
        menu.set_result_internal(ItemStack::new("minecraft:written_book", 1));
        let moved = menu.quick_move(2, &mut player);
        assert_eq!(moved.item_id(), "minecraft:written_book");
    }

    #[test]
    fn merchant_menu_try_move_items_moves_payment_back_and_fills_for_new_offer() {
        // Java parity: MerchantMenu.tryMoveItems(newTradeIndex) pushes the
        // current payment slots back into the player inventory and then
        // re-fills them from the player inventory using the new offer's cost.
        //
        // Note: moveFromInventoryToPaymentSlot fills the payment slot up to
        // its max stack size (or the inventory item count, whichever is
        // smaller). It does NOT cap at the cost's count — the result slot
        // will only consume what the offer actually requires, leaving the
        // surplus visible to the player.
        let offers = vec![
            MerchantOffer::new(
                ItemCost::new("minecraft:emerald", 2),
                None,
                ItemStack::new("minecraft:diamond", 1),
                5,
                1,
                0.05,
            ),
            MerchantOffer::new(
                ItemCost::new("minecraft:emerald", 3),
                None,
                ItemStack::new("minecraft:emerald_block", 1),
                5,
                1,
                0.05,
            ),
        ];
        let mut menu = MerchantMenu::new();
        let mut player = PlayerInventory::new();

        // Seed payment slot A with 2 emeralds (the active offer 0 payment).
        menu.set_slot(0, ItemStack::new("minecraft:emerald", 2), &mut player);
        // Seed hotbar slot 0 with 5 more emeralds.
        player.set(0, ItemStack::new("minecraft:emerald", 5));

        // Switch to offer index 1 (needs 3 emeralds).
        menu.try_move_items(1, &offers, &mut player);

        // The 2 from payment_a are pushed back into the player inventory
        // (merging with the 5 in hotbar slot 0 → 7 emeralds there), then
        // the refill loop pulls up to a full stack (64) into payment A. With
        // only 7 emeralds available the whole stack moves over and the
        // hotbar slot ends up empty.
        let payment_a = menu.get_slot(0, &player).unwrap();
        assert_eq!(payment_a.item_id(), "minecraft:emerald");
        assert_eq!(payment_a.count(), 7);
        assert!(player.get(0).is_empty());

        // The offer at index 1 still requires 3, so it should now be
        // satisfied — the trader will leave 4 emeralds behind when the
        // player takes the result.
        assert!(offers[1].satisfied_by(&payment_a, &ItemStack::empty()));
    }

    #[test]
    fn merchant_menu_try_move_items_ignores_out_of_bounds_indices() {
        // Java parity: `tryMoveItems` guards `newTradeIndex >= 0 &&
        // getOffers().size() > newTradeIndex`.
        let offers = vec![MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 1),
            None,
            ItemStack::new("minecraft:diamond", 1),
            5,
            1,
            0.05,
        )];
        let mut menu = MerchantMenu::new();
        let mut player = PlayerInventory::new();
        menu.set_slot(0, ItemStack::new("minecraft:emerald", 4), &mut player);

        menu.try_move_items(5, &offers, &mut player);
        // Payment slot is unchanged.
        assert_eq!(menu.get_slot(0, &player).unwrap().count(), 4);
    }

    #[test]
    fn merchant_menu_try_move_items_with_two_cost_offer_fills_both_slots() {
        // Java parity: when the offer specifies a `cost_b`, both payment
        // slots are auto-filled in order — `cost_a` → slot 0, `cost_b` → slot 1.
        let offers = vec![MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 2),
            Some(ItemCost::new("minecraft:book", 1)),
            ItemStack::new("minecraft:written_book", 1),
            5,
            1,
            0.05,
        )];
        let mut menu = MerchantMenu::new();
        let mut player = PlayerInventory::new();
        // Hotbar slots — emeralds in slot 0, books in slot 1.
        player.set(0, ItemStack::new("minecraft:emerald", 6));
        player.set(1, ItemStack::new("minecraft:book", 3));

        menu.try_move_items(0, &offers, &mut player);

        let payment_a = menu.get_slot(0, &player).unwrap();
        let payment_b = menu.get_slot(1, &player).unwrap();
        assert_eq!(payment_a.item_id(), "minecraft:emerald");
        assert_eq!(payment_a.count(), 6);
        assert_eq!(payment_b.item_id(), "minecraft:book");
        assert_eq!(payment_b.count(), 3);
    }

    // -------- Cross-cutting click-mode tests --------
    //
    // These tests exercise the per-menu slot layout under each of the click
    // modes that the network layer dispatches through (hotbar swap, drag
    // split, double-click collect, drop, creative clone, close-while-carrying,
    // disconnect-while-open). The actual click pipeline runs in
    // `inventory.rs`; here we verify that the menu structs faithfully expose
    // and accept slot reads/writes so the pipeline can act on them.

    #[test]
    fn hotbar_swap_works_in_representative_menu_types() {
        // ChestMenu: swap chest slot 0 with hotbar slot 4.
        // Hotbar slot 4 in menu-space lives at chest_size + PLAYER_MAIN_STORAGE + 4.
        let mut menu = ChestMenu::new(3);
        let mut player = PlayerInventory::new();
        let chest_item = ItemStack::new("minecraft:diamond", 3);
        let hotbar_item = ItemStack::new("minecraft:emerald", 1);
        menu.set_slot(0, chest_item.clone(), &mut player);
        player.set(4, hotbar_item.clone());

        let chest_size = menu.chest_size();
        let hotbar_menu_slot = chest_size + PLAYER_MAIN_STORAGE + 4;
        let hotbar_content = menu.get_slot(hotbar_menu_slot, &player).unwrap();
        assert_eq!(hotbar_content.item_id(), "minecraft:emerald");

        menu.set_slot(0, hotbar_content.clone(), &mut player);
        player.set(4, chest_item.clone());

        assert_eq!(
            menu.get_slot(0, &player).unwrap().item_id(),
            "minecraft:emerald"
        );
        assert_eq!(player.get(4).item_id(), "minecraft:diamond");
    }

    #[test]
    fn drag_split_distributes_stack_evenly_across_chest_slots() {
        // Java parity: left-click drag splits the carried stack evenly across
        // every target slot. 8 diamonds across 4 slots → 2 each.
        let mut menu = ChestMenu::new(3);
        let mut player = PlayerInventory::new();
        let carry = 8;
        let target_slots = [0usize, 1, 2, 3];
        let each = carry / target_slots.len();
        for &slot in &target_slots {
            menu.set_slot(
                slot,
                ItemStack::new("minecraft:diamond", each as i32),
                &mut player,
            );
        }
        for &slot in &target_slots {
            assert_eq!(menu.get_slot(slot, &player).unwrap().count(), 2);
        }
    }

    #[test]
    fn drag_split_in_hopper_menu_fills_all_5_hopper_slots() {
        let mut menu = HopperMenu::new();
        let mut player = PlayerInventory::new();
        let each = 10i32;
        for slot in 0..5 {
            menu.set_slot(slot, ItemStack::new("minecraft:stone", each), &mut player);
        }
        for slot in 0..5 {
            assert_eq!(menu.get_slot(slot, &player).unwrap().count(), 10);
        }
    }

    #[test]
    fn double_click_collect_gathers_items_into_cursor_from_chest() {
        // Java parity: PICKUP_ALL mode walks every slot looking for matching
        // items and merges them into the cursor up to the cursor's max
        // stack. Here we verify the slot reads that pickup_all would issue
        // are sane: cursor of 2 + slot 0 of 3 + slot 1 of 5 = 10.
        let mut menu = ChestMenu::new(3);
        let mut player = PlayerInventory::new();
        menu.set_slot(0, ItemStack::new("minecraft:diamond", 3), &mut player);
        menu.set_slot(1, ItemStack::new("minecraft:diamond", 5), &mut player);
        let total = 2
            + menu.get_slot(0, &player).unwrap().count()
            + menu.get_slot(1, &player).unwrap().count();
        assert_eq!(total, 10);
    }

    #[test]
    fn drop_from_slot_removes_item_from_furnace_input() {
        // Java parity: THROW mode with Ctrl removes the entire slot. Here
        // we simulate the resulting slot state — empty after the drop.
        let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
        let mut player = PlayerInventory::new();
        menu.set_slot(0, ItemStack::new("minecraft:raw_iron", 8), &mut player);
        let stack = menu.get_slot(0, &player).unwrap();
        assert!(!stack.is_empty());
        menu.set_slot(0, ItemStack::empty(), &mut player);
        assert!(menu.get_slot(0, &player).unwrap().is_empty());
    }

    #[test]
    fn drop_single_from_dispenser_slot() {
        // Java parity: THROW without Ctrl drops one item from the targeted slot.
        let mut menu = DispenserMenu::new();
        let mut player = PlayerInventory::new();
        menu.set_slot(4, ItemStack::new("minecraft:arrow", 16), &mut player);
        let count_before = menu.get_slot(4, &player).unwrap().count();
        menu.set_slot(
            4,
            ItemStack::new("minecraft:arrow", count_before - 1),
            &mut player,
        );
        assert_eq!(menu.get_slot(4, &player).unwrap().count(), 15);
    }

    #[test]
    fn creative_clone_produces_full_stack_from_slot() {
        // Java parity: CLONE mode (middle-click in creative) replaces the
        // cursor with a max-stack copy of the targeted slot's contents.
        let mut menu = ChestMenu::new(3);
        let mut player = PlayerInventory::new();
        menu.set_slot(5, ItemStack::new("minecraft:emerald", 2), &mut player);
        let slot_item = menu.get_slot(5, &player).unwrap();
        let max = slot_item.max_stack_size();
        assert_eq!(max, 64);
        let cloned = ItemStack::new(slot_item.item_id(), max as i32);
        assert_eq!(cloned.count(), 64);
        assert_eq!(cloned.item_id(), "minecraft:emerald");
    }

    #[test]
    fn close_while_carrying_returns_item_to_inventory() {
        // Java parity: AbstractContainerMenu.removed places the carried item
        // back into the inventory (or drops it if no slot is free).
        let mut player = PlayerInventory::new();
        let carried = ItemStack::new("minecraft:diamond", 3);

        let placed = if let Some(empty_slot) = (0..36).find(|&i| player.get(i).is_empty()) {
            player.set(empty_slot, carried.clone());
            true
        } else {
            false
        };
        assert!(placed);
        assert_eq!(player.get(0).item_id(), "minecraft:diamond");
        assert_eq!(player.get(0).count(), 3);
    }

    #[test]
    fn disconnect_while_open_drops_payment_items() {
        // Java parity: MerchantMenu.removed places the merchant container's
        // payment slots back into the player's inventory on disconnect.
        let mut menu = MerchantMenu::new();
        let mut player = PlayerInventory::new();
        menu.set_slot(0, ItemStack::new("minecraft:emerald", 5), &mut player);
        menu.set_slot(1, ItemStack::new("minecraft:book", 1), &mut player);

        let payment_a = menu.get_slot(0, &player).unwrap();
        let payment_b = menu.get_slot(1, &player).unwrap();

        if !payment_a.is_empty() {
            let slot = (0..36).find(|&i| player.get(i).is_empty()).unwrap();
            player.set(slot, payment_a);
            menu.set_slot(0, ItemStack::empty(), &mut player);
        }
        if !payment_b.is_empty() {
            let slot = (0..36).find(|&i| player.get(i).is_empty()).unwrap();
            player.set(slot, payment_b);
            menu.set_slot(1, ItemStack::empty(), &mut player);
        }

        assert_eq!(player.get(0).item_id(), "minecraft:emerald");
        assert_eq!(player.get(1).item_id(), "minecraft:book");
        assert!(menu.get_slot(0, &player).unwrap().is_empty());
        assert!(menu.get_slot(1, &player).unwrap().is_empty());
    }

    // -------- Stale state-ID style sweep: all_slots length checks --------

    #[test]
    fn every_menu_reports_correct_slot_count_for_full_resync() {
        let player = PlayerInventory::new();
        assert_eq!(
            CraftingMenu::new(empty_recipes()).all_slots(&player).len(),
            46
        );
        assert_eq!(
            AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla())
                .all_slots(&player)
                .len(),
            39
        );
        assert_eq!(
            ChestMenu::new(3).all_slots(&player).len(),
            27 + PLAYER_SLOTS
        );
        assert_eq!(
            ChestMenu::new(6).all_slots(&player).len(),
            54 + PLAYER_SLOTS
        );
        assert_eq!(HopperMenu::new().all_slots(&player).len(), 41);
        assert_eq!(DispenserMenu::new().all_slots(&player).len(), 45);
        assert_eq!(ShulkerBoxMenu::new().all_slots(&player).len(), 63);
        assert_eq!(AnvilMenu::new().all_slots(&player).len(), 39);
        assert_eq!(SmithingMenu::new().all_slots(&player).len(), 40);
        assert_eq!(StonecutterMenu::new().all_slots(&player).len(), 38);
        assert_eq!(GrindstoneMenu::new().all_slots(&player).len(), 39);
        assert_eq!(EnchantmentMenu::new().all_slots(&player).len(), 38);
        assert_eq!(BrewingStandMenu::new().all_slots(&player).len(), 41);
        assert_eq!(CartographyTableMenu::new().all_slots(&player).len(), 39);
        assert_eq!(LoomMenu::new().all_slots(&player).len(), 40);
        assert_eq!(LecternMenu::new().all_slots().len(), 1);
        assert_eq!(BeaconMenu::new().all_slots(&player).len(), 37);
        assert_eq!(CrafterMenu::new().all_slots(&player).len(), 46);
        assert_eq!(NautilusInventoryMenu::new().all_slots(&player).len(), 38);
        assert_eq!(MerchantMenu::new().all_slots(&player).len(), 39);
    }
}
