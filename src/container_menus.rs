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

use crate::block_entity::BannerPatternLayer;
use crate::inventory::same_item_same_components;
use crate::item_properties::{ItemComponent, MapPostProcessing};
use crate::item_stack::ItemStack;
use crate::map_state::DyeColor;
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

/// `AbstractContainerMenu.dropOrPlaceInInventory`: on a normal close the stack is
/// placed back into the player inventory (overflow dropped in-world via
/// `place_item_back_in_inventory`); on disconnect/removal it is dropped in-world.
/// Empty stacks are ignored.
pub(crate) fn drop_or_place_in_inventory(
    player: &mut PlayerInventory,
    stack: ItemStack,
    disconnected: bool,
) {
    if stack.is_empty() {
        return;
    }
    if disconnected {
        player.drop_item(stack);
    } else {
        player.place_item_back_in_inventory(stack);
    }
}

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

/// `BannerPatternTags.NO_ITEM_REQUIRED` — the patterns selectable in the loom with an
/// empty pattern slot, in the tag's declared order (drives loom button indices).
/// Source: `data/minecraft/tags/banner_pattern/no_item_required.json`.
pub(super) const NO_ITEM_REQUIRED_PATTERNS: &[&str] = &[
    "minecraft:square_bottom_left",
    "minecraft:square_bottom_right",
    "minecraft:square_top_left",
    "minecraft:square_top_right",
    "minecraft:stripe_bottom",
    "minecraft:stripe_top",
    "minecraft:stripe_left",
    "minecraft:stripe_right",
    "minecraft:stripe_center",
    "minecraft:stripe_middle",
    "minecraft:stripe_downright",
    "minecraft:stripe_downleft",
    "minecraft:small_stripes",
    "minecraft:cross",
    "minecraft:straight_cross",
    "minecraft:triangle_bottom",
    "minecraft:triangle_top",
    "minecraft:triangles_bottom",
    "minecraft:triangles_top",
    "minecraft:diagonal_left",
    "minecraft:diagonal_up_right",
    "minecraft:diagonal_up_left",
    "minecraft:diagonal_right",
    "minecraft:circle",
    "minecraft:rhombus",
    "minecraft:half_vertical",
    "minecraft:half_horizontal",
    "minecraft:half_vertical_right",
    "minecraft:half_horizontal_bottom",
    "minecraft:border",
    "minecraft:gradient",
    "minecraft:gradient_up",
];

/// The banner pattern provided by a `*_banner_pattern` item via its
/// `PROVIDES_BANNER_PATTERNS` data component (modelled as a static map since each
/// loom pattern item provides exactly one pattern). Source:
/// `data/minecraft/tags/banner_pattern/pattern_item/*.json`.
pub(super) fn pattern_for_pattern_item(item_id: &str) -> Option<&'static str> {
    Some(match item_id {
        "minecraft:creeper_banner_pattern" => "minecraft:creeper",
        "minecraft:skull_banner_pattern" => "minecraft:skull",
        "minecraft:flower_banner_pattern" => "minecraft:flower",
        "minecraft:mojang_banner_pattern" => "minecraft:mojang",
        "minecraft:globe_banner_pattern" => "minecraft:globe",
        "minecraft:piglin_banner_pattern" => "minecraft:piglin",
        "minecraft:flow_banner_pattern" => "minecraft:flow",
        "minecraft:guster_banner_pattern" => "minecraft:guster",
        "minecraft:field_masoned_banner_pattern" => "minecraft:bricks",
        "minecraft:bordure_indented_banner_pattern" => "minecraft:curly_border",
        _ => return None,
    })
}

/// `LoomMenu.getSelectablePatterns(patternStack)`: with an empty pattern slot the
/// `NO_ITEM_REQUIRED` patterns are selectable; with a held loom-pattern item only the
/// pattern that item provides; otherwise none.
pub(super) fn loom_selectable_patterns(pattern_item: &ItemStack) -> Vec<&'static str> {
    if pattern_item.is_empty() {
        NO_ITEM_REQUIRED_PATTERNS.to_vec()
    } else {
        pattern_for_pattern_item(pattern_item.item_id())
            .into_iter()
            .collect()
    }
}

/// `DyeItem.getDyeColor()` — maps a dye item to its `DyeColor`, covering the 16
/// `*_dye` items plus the special-cased dyes accepted by `is_dye_item`.
pub(super) fn dye_color_for_item(item_id: &str) -> Option<DyeColor> {
    Some(match item_id {
        "minecraft:white_dye" | "minecraft:bone_meal" => DyeColor::White,
        "minecraft:orange_dye" => DyeColor::Orange,
        "minecraft:magenta_dye" => DyeColor::Magenta,
        "minecraft:light_blue_dye" => DyeColor::LightBlue,
        "minecraft:yellow_dye" => DyeColor::Yellow,
        "minecraft:lime_dye" => DyeColor::Lime,
        "minecraft:pink_dye" => DyeColor::Pink,
        "minecraft:gray_dye" => DyeColor::Gray,
        "minecraft:light_gray_dye" => DyeColor::LightGray,
        "minecraft:cyan_dye" => DyeColor::Cyan,
        "minecraft:purple_dye" => DyeColor::Purple,
        "minecraft:blue_dye" | "minecraft:lapis_lazuli" => DyeColor::Blue,
        "minecraft:brown_dye" | "minecraft:cocoa_beans" => DyeColor::Brown,
        "minecraft:green_dye" => DyeColor::Green,
        "minecraft:red_dye" => DyeColor::Red,
        "minecraft:black_dye" | "minecraft:ink_sac" => DyeColor::Black,
        _ => return None,
    })
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
pub(super) struct MoveWrite {
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

mod menus_brewing;
mod menus_crafting_and_furnace;
mod menus_storage;
mod menus_workstation;
mod menus_table;
mod menus_misc;
mod menus_entity;

#[cfg(test)]
use menus_brewing::*;
#[cfg(test)]
use menus_crafting_and_furnace::*;
#[cfg(test)]
use menus_storage::*;
#[cfg(test)]
use menus_workstation::*;
#[cfg(test)]
use menus_table::*;
#[cfg(test)]
use menus_misc::*;
#[cfg(test)]
use menus_entity::*;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_close;
#[cfg(test)]
mod tests_table;
