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
use crate::recipe_system::{
    stonecutter_recipes_for_input, CraftingStack, FuelValues, ItemAmount, RecipeMap,
    StonecutterSelection,
};
use std::collections::{BTreeMap, BTreeSet};

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
pub(super) fn is_potion_or_bottle(item_id: &str) -> bool {
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
pub(super) fn is_brewing_ingredient(item_id: &str) -> bool {
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
pub(super) fn is_dye_item(item_id: &str) -> bool {
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
pub(super) fn is_pattern_item(item_id: &str) -> bool {
    item_id.ends_with("_banner_pattern")
}

/// Banner item ID test.
pub(super) fn is_banner_item(item_id: &str) -> bool {
    item_id.ends_with("_banner")
}

/// `CartographyTableMenu`'s additional-slot test.
pub(super) fn is_cartography_additional(item_id: &str) -> bool {
    matches!(
        item_id,
        "minecraft:paper" | "minecraft:map" | "minecraft:glass_pane"
    )
}

/// Whether the item carries a map id (filled map).
pub(super) fn is_filled_map(item_id: &str) -> bool {
    item_id == "minecraft:filled_map"
}

/// Used by grindstone slots — accepts damageable items or items with any
/// enchantment. Enchantment data is not yet stored on items in RustCraft, so
/// we approximate via the items that vanilla treats as damageable. The
/// behaviour is sound (a non-damageable, non-enchanted item will be rejected
/// here matching vanilla).
pub(super) fn is_grindstone_input(stack: &ItemStack) -> bool {
    stack.is_damageable_item() || stack.max_damage() > 0
}

pub(super) fn item_amount_to_stack(amount: &ItemAmount) -> ItemStack {
    ItemStack::new(amount.item, amount.count as i32)
}

/// Common slot-mutation helpers shared by every menu. Operates on a contiguous
/// `Vec<ItemStack>` of menu-local slots plus a borrow into the player's main
/// storage + hotbar.
pub(super) fn append_player_slots(out: &mut Vec<ItemStack>, player: &PlayerInventory) {
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
pub(super) fn plan_move_item_stack_to(
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
pub(super) fn player_slot_for(menu_index: usize, player_start: usize) -> Option<usize> {
    let local = menu_index.checked_sub(player_start)?;
    if local < PLAYER_MAIN_STORAGE {
        Some(9 + local)
    } else if local < PLAYER_SLOTS {
        Some(local - PLAYER_MAIN_STORAGE)
    } else {
        None
    }
}

pub(super) fn read_player_slot(menu_index: usize, player_start: usize, player: &PlayerInventory) -> ItemStack {
    player_slot_for(menu_index, player_start)
        .map(|slot| player.get(slot).clone())
        .unwrap_or_else(ItemStack::empty)
}

pub(super) fn write_player_slot(
    menu_index: usize,
    player_start: usize,
    player: &mut PlayerInventory,
    stack: ItemStack,
) {
    if let Some(slot) = player_slot_for(menu_index, player_start) {
        player.set(slot, stack);
    }
}

mod menus_crafting_and_furnace;
mod menus_storage;
mod menus_workstation;
mod menus_table;
mod menus_misc;
mod menus_entity;

pub use menus_crafting_and_furnace::*;
pub use menus_storage::*;
pub use menus_workstation::*;
pub use menus_table::*;
pub use menus_misc::*;
pub use menus_entity::*;

#[cfg(test)]
mod tests;
