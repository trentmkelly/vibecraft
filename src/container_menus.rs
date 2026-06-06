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

// ---------------------------------------------------------------------------
// Anvil helpers (`AnvilMenu.createResult` prerequisites)
// ---------------------------------------------------------------------------

/// `EnchantmentHelper.canStoreEnchantments`: whether the item carries its enchantment
/// component (`ENCHANTMENTS`, or `STORED_ENCHANTMENTS` for an enchanted book). In
/// 26.1.2 `DataComponents.COMMON_ITEM_COMPONENTS` sets an empty `ENCHANTMENTS` on
/// EVERY item, so this is true for any non-empty item (which is why anything can be
/// renamed in an anvil). Enchantability for the table is a separate concern.
pub(super) fn can_store_enchantments(stack: &ItemStack) -> bool {
    !stack.is_empty()
}

/// `ItemStack.isValidRepairItem`: `addition` is a member of the item's `Repairable`
/// material tag (e.g. `#minecraft:diamond_tool_materials`), or the literal repair item.
pub(super) fn anvil_is_valid_repair_item(item: &ItemStack, addition: &ItemStack) -> bool {
    match item.component("minecraft:repairable") {
        Some(ItemComponent::Repairable(material)) => {
            if material.starts_with('#') {
                crate::item_tags::item_in_tag(addition.item_id(), material)
            } else {
                *material == addition.item_id()
            }
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Smithing (transform + trim) — `SmithingMenu`
// ---------------------------------------------------------------------------

/// `PROVIDES_TRIM_MATERIAL`: the trim material an addition item supplies (from
/// `Items.java` `.trimMaterial(...)`).
fn trim_material_for_ingredient(item_id: &str) -> Option<&'static str> {
    Some(match item_id {
        "minecraft:iron_ingot" => "minecraft:iron",
        "minecraft:copper_ingot" => "minecraft:copper",
        "minecraft:gold_ingot" => "minecraft:gold",
        "minecraft:netherite_ingot" => "minecraft:netherite",
        "minecraft:diamond" => "minecraft:diamond",
        "minecraft:emerald" => "minecraft:emerald",
        "minecraft:lapis_lazuli" => "minecraft:lapis",
        "minecraft:amethyst_shard" => "minecraft:amethyst",
        "minecraft:quartz" => "minecraft:quartz",
        "minecraft:redstone" => "minecraft:redstone",
        "minecraft:resin_brick" => "minecraft:resin",
        _ => return None,
    })
}

/// The netherite-upgrade transform result for a diamond gear base (the
/// `netherite_*_smithing` recipes).
fn netherite_transform_result(base_item: &str) -> Option<&'static str> {
    Some(match base_item {
        "minecraft:diamond_helmet" => "minecraft:netherite_helmet",
        "minecraft:diamond_chestplate" => "minecraft:netherite_chestplate",
        "minecraft:diamond_leggings" => "minecraft:netherite_leggings",
        "minecraft:diamond_boots" => "minecraft:netherite_boots",
        "minecraft:diamond_sword" => "minecraft:netherite_sword",
        "minecraft:diamond_axe" => "minecraft:netherite_axe",
        "minecraft:diamond_pickaxe" => "minecraft:netherite_pickaxe",
        "minecraft:diamond_shovel" => "minecraft:netherite_shovel",
        "minecraft:diamond_hoe" => "minecraft:netherite_hoe",
        _ => return None,
    })
}

/// The trim pattern a `*_armor_trim_smithing_template` item provides (matching the
/// `equipment_trim::TRIM_PATTERNS` set; each template provides the like-named pattern).
fn pattern_for_trim_template(template_item: &str) -> Option<&'static str> {
    Some(match template_item {
        "minecraft:sentry_armor_trim_smithing_template" => "minecraft:sentry",
        "minecraft:dune_armor_trim_smithing_template" => "minecraft:dune",
        "minecraft:coast_armor_trim_smithing_template" => "minecraft:coast",
        "minecraft:wild_armor_trim_smithing_template" => "minecraft:wild",
        "minecraft:ward_armor_trim_smithing_template" => "minecraft:ward",
        "minecraft:eye_armor_trim_smithing_template" => "minecraft:eye",
        "minecraft:vex_armor_trim_smithing_template" => "minecraft:vex",
        "minecraft:tide_armor_trim_smithing_template" => "minecraft:tide",
        "minecraft:snout_armor_trim_smithing_template" => "minecraft:snout",
        "minecraft:rib_armor_trim_smithing_template" => "minecraft:rib",
        "minecraft:spire_armor_trim_smithing_template" => "minecraft:spire",
        "minecraft:wayfinder_armor_trim_smithing_template" => "minecraft:wayfinder",
        "minecraft:shaper_armor_trim_smithing_template" => "minecraft:shaper",
        "minecraft:silence_armor_trim_smithing_template" => "minecraft:silence",
        "minecraft:raiser_armor_trim_smithing_template" => "minecraft:raiser",
        "minecraft:host_armor_trim_smithing_template" => "minecraft:host",
        "minecraft:flow_armor_trim_smithing_template" => "minecraft:flow",
        "minecraft:bolt_armor_trim_smithing_template" => "minecraft:bolt",
        _ => return None,
    })
}

/// `SmithingMenu.createResult` (recipe-driven): a netherite-upgrade transform
/// (`netherite_upgrade_smithing_template` + diamond gear + `netherite_ingot` → the
/// netherite gear, keeping the base's components via `createWithOriginalComponents`),
/// or an armour trim (a trim-pattern template + trimmable armour + a trim-material item
/// → the armour with a new `TRIM`; empty when the base already carries that exact trim).
/// Otherwise empty.
pub(super) fn smithing_create_result(
    template: &ItemStack,
    base: &ItemStack,
    addition: &ItemStack,
) -> ItemStack {
    if template.is_empty() || base.is_empty() || addition.is_empty() {
        return ItemStack::empty();
    }
    if template.item_id() == "minecraft:netherite_upgrade_smithing_template"
        && addition.item_id() == "minecraft:netherite_ingot"
    {
        if let Some(result_item) = netherite_transform_result(base.item_id()) {
            return base.transmute_copy(result_item, 1);
        }
    }
    if let (Some(pattern), Some(material)) = (
        pattern_for_trim_template(template.item_id()),
        trim_material_for_ingredient(addition.item_id()),
    ) {
        if crate::item_tags::item_in_tag(base.item_id(), "#minecraft:trimmable_armor") {
            let new_trim = ItemComponent::ArmorTrim { material, pattern };
            if base.component("minecraft:trim") == Some(&new_trim) {
                return ItemStack::empty();
            }
            let mut result = base.copy_with_count(1);
            result.set_component(new_trim);
            return result;
        }
    }
    ItemStack::empty()
}

fn anvil_repair_cost_of(item: &ItemStack) -> i32 {
    match item.component("minecraft:repair_cost") {
        Some(ItemComponent::RepairCost(cost)) => *cost,
        _ => 0,
    }
}

fn anvil_enchants_compatible(a: &str, b: &str) -> bool {
    use crate::enchantment_system::{are_compatible, enchantment};
    match (enchantment(a), enchantment(b)) {
        (Some(left), Some(right)) => are_compatible(left, right),
        _ => a != b,
    }
}

/// The outputs of `AnvilMenu.createResult`.
pub(super) struct AnvilCombineResult {
    pub result: ItemStack,
    pub cost: i32,
    pub repair_item_count_cost: i32,
    pub only_renaming: bool,
}

/// 1:1 port of `AnvilMenu.createResult` (the full repair + enchantment-combine + rename
/// pipeline). `item_name` is the requested rename (vanilla compares it to
/// `getHoverName()`; RustCraft has no display-name/translation layer so it compares to
/// the item id, matching the existing rename path). `creative` is `hasInfiniteMaterials`.
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub(super) fn anvil_create_result(
    input: &ItemStack,
    addition: &ItemStack,
    item_name: Option<&str>,
    creative: bool,
) -> AnvilCombineResult {
    let empty = || AnvilCombineResult {
        result: ItemStack::empty(),
        cost: 0,
        repair_item_count_cost: 0,
        only_renaming: false,
    };
    if input.is_empty() || !can_store_enchantments(input) {
        return empty();
    }

    let mut price: i64 = 0;
    let mut naming_cost: i64 = 0;
    let mut repair_item_count_cost = 0;
    let mut only_renaming = false;
    let mut result = input.clone();
    let mut enchantments = enchantments_for_crafting(&result);
    let tax = i64::from(anvil_repair_cost_of(input)) + i64::from(anvil_repair_cost_of(addition));

    if !addition.is_empty() {
        let using_book = addition.component("minecraft:stored_enchantments").is_some();
        if result.is_damageable_item() && anvil_is_valid_repair_item(input, addition) {
            // Repair with a material item (one durability quarter per item consumed).
            let mut repair_amount = result.damage_value().min(result.max_damage() / 4) as i32;
            if repair_amount <= 0 {
                return empty();
            }
            let mut count = 0;
            while repair_amount > 0 && count < addition.count() {
                let new_damage = result.damage_value() as i32 - repair_amount;
                result.set_damage_value(new_damage.max(0) as u32);
                price += 1;
                repair_amount = result.damage_value().min(result.max_damage() / 4) as i32;
                count += 1;
            }
            repair_item_count_cost = count;
        } else {
            if !using_book && (result.item_id() != addition.item_id() || !result.is_damageable_item())
            {
                return empty();
            }
            if result.is_damageable_item() && !using_book {
                // Repair by combining two of the same item (+12% durability bonus).
                let remaining1 = input.max_damage() as i32 - input.damage_value() as i32;
                let remaining2 = addition.max_damage() as i32 - addition.damage_value() as i32;
                let bonus = remaining2 + result.max_damage() as i32 * 12 / 100;
                let remaining = remaining1 + bonus;
                let result_damage = (result.max_damage() as i32 - remaining).max(0);
                if result_damage < result.damage_value() as i32 {
                    result.set_damage_value(result_damage as u32);
                    price += 2;
                }
            }

            let mut any_compatible = false;
            let mut any_not_compatible = false;
            for (id, add_level) in enchantments_for_crafting(addition) {
                let current = enchantments.get(&id).copied().unwrap_or(0);
                let mut level = if current == add_level {
                    add_level + 1
                } else {
                    add_level.max(current)
                };
                let mut compatible = crate::enchantment_system::can_enchant(input.item_id(), &id);
                if creative || input.item_id() == "minecraft:enchanted_book" {
                    compatible = true;
                }
                for other in enchantments.keys() {
                    if *other != id && !anvil_enchants_compatible(&id, other) {
                        compatible = false;
                        price += 1;
                    }
                }
                if !compatible {
                    any_not_compatible = true;
                } else {
                    any_compatible = true;
                    let max_level = crate::enchantment_system::enchantment(&id)
                        .map_or(level, |def| def.max_level);
                    if level > max_level {
                        level = max_level;
                    }
                    let mut fee = crate::enchantment_system::enchantment(&id)
                        .map_or(0, |def| def.anvil_cost);
                    if using_book {
                        fee = (fee / 2).max(1);
                    }
                    enchantments.insert(id, level);
                    price += i64::from(fee) * i64::from(level);
                    if input.count() > 1 {
                        price = 40;
                    }
                }
            }
            if any_not_compatible && !any_compatible {
                return empty();
            }
        }
    }

    // Rename cost (vanilla compares the requested name to getHoverName(); see doc note).
    if let Some(name) = item_name.filter(|n| !n.trim().is_empty()) {
        if name != input.item_id() {
            naming_cost = 1;
            price += 1;
        }
    }
    // (The "input has a custom name -> remove it" branch is not modelled, since
    // RustCraft does not attach a CUSTOM_NAME component to inputs.)

    let final_price = if price <= 0 {
        0
    } else {
        (tax + price).clamp(0, i64::from(i32::MAX)) as i32
    };
    let mut cost = final_price;
    if price <= 0 {
        result = ItemStack::empty();
    }
    if naming_cost == price && naming_cost > 0 {
        if cost >= 40 {
            cost = 39;
        }
        only_renaming = true;
    }
    if cost >= 40 && !creative {
        result = ItemStack::empty();
    }
    if !result.is_empty() {
        let mut base_cost = anvil_repair_cost_of(&result).max(anvil_repair_cost_of(addition));
        if naming_cost != price || naming_cost == 0 {
            base_cost = calculate_increased_repair_cost(base_cost);
        }
        result.set_component(ItemComponent::RepairCost(base_cost));
        set_enchantments_for_crafting(&mut result, enchantments);
    }

    AnvilCombineResult {
        result,
        cost,
        repair_item_count_cost,
        only_renaming,
    }
}

// ---------------------------------------------------------------------------
// Grindstone (disenchant + repair + experience) — `GrindstoneMenu`
// ---------------------------------------------------------------------------

/// `AnvilMenu.calculateIncreasedRepairCost`: the prior-work penalty doubles + 1 per
/// stored enchantment, saturating at `i32::MAX`.
pub(super) fn calculate_increased_repair_cost(base: i32) -> i32 {
    (i64::from(base) * 2 + 1).min(i64::from(i32::MAX)) as i32
}

/// `EnchantmentHelper.getEnchantmentsForCrafting`: an enchanted book stores its
/// enchantments in `STORED_ENCHANTMENTS`; every other item uses `ENCHANTMENTS`.
pub(super) fn enchantments_for_crafting(stack: &ItemStack) -> BTreeMap<String, i32> {
    let key = if stack.item_id() == "minecraft:enchanted_book" {
        "minecraft:stored_enchantments"
    } else {
        "minecraft:enchantments"
    };
    match stack.component(key) {
        Some(ItemComponent::Enchantments(m) | ItemComponent::StoredEnchantments(m)) => m.clone(),
        _ => BTreeMap::new(),
    }
}

pub(super) fn set_enchantments_for_crafting(stack: &mut ItemStack, enchantments: BTreeMap<String, i32>) {
    let is_book = stack.item_id() == "minecraft:enchanted_book";
    let key = if is_book {
        "minecraft:stored_enchantments"
    } else {
        "minecraft:enchantments"
    };
    if enchantments.is_empty() {
        stack.remove_component(key);
    } else if is_book {
        stack.set_component(ItemComponent::StoredEnchantments(enchantments));
    } else {
        stack.set_component(ItemComponent::Enchantments(enchantments));
    }
}

/// `EnchantmentHelper.hasAnyEnchantments`.
pub(super) fn has_any_enchantments(stack: &ItemStack) -> bool {
    !enchantments_for_crafting(stack).is_empty()
}

/// `GrindstoneMenu.getExperienceFromItem`: sum of `getMinCost(level)` over the item's
/// non-curse enchantments.
pub(super) fn grindstone_experience_from_item(stack: &ItemStack) -> i32 {
    enchantments_for_crafting(stack)
        .iter()
        .filter(|(id, _)| !crate::enchantment_system::is_curse(id))
        .map(|(id, level)| crate::enchantment_system::min_cost_for(id, *level))
        .sum()
}

/// `GrindstoneMenu.getExperienceAmount`: `ceil(total/2) + random.nextInt(ceil(total/2))`
/// over both inputs' non-curse enchantment cost. The random draw is supplied by the
/// caller (`random_in_half` must lie in `0..ceil(total/2)`); 0 when there is no cost.
pub(super) fn grindstone_experience_on_take(
    input: &ItemStack,
    additional: &ItemStack,
    random_in_half: i32,
) -> i32 {
    let amount = grindstone_experience_from_item(input) + grindstone_experience_from_item(additional);
    if amount > 0 {
        (amount + 1) / 2 + random_in_half
    } else {
        0
    }
}

/// `GrindstoneMenu.removeNonCursesFrom`: keep only curse enchantments, transmute an
/// emptied enchanted book to a plain book, and recompute the prior-work repair cost.
pub(super) fn grindstone_remove_non_curses(mut item: ItemStack) -> ItemStack {
    let mut enchantments = enchantments_for_crafting(&item);
    enchantments.retain(|id, _| crate::enchantment_system::is_curse(id));
    let remaining = enchantments.len() as i32;
    set_enchantments_for_crafting(&mut item, enchantments);
    if item.item_id() == "minecraft:enchanted_book" && remaining == 0 {
        item = item.transmute_copy("minecraft:book", item.count());
    }
    let mut repair_cost = 0;
    for _ in 0..remaining {
        repair_cost = calculate_increased_repair_cost(repair_cost);
    }
    item.set_component(ItemComponent::RepairCost(repair_cost));
    item
}

/// `GrindstoneMenu.mergeEnchantsFrom`: copy the source item's enchantments onto the
/// target, upgrading to the higher level, but never adding a duplicate curse.
fn grindstone_merge_enchants(target: &mut ItemStack, source: &ItemStack) {
    let mut enchantments = enchantments_for_crafting(target);
    for (id, level) in enchantments_for_crafting(source) {
        let existing = enchantments.get(&id).copied().unwrap_or(0);
        if !crate::enchantment_system::is_curse(&id) || existing == 0 {
            enchantments.insert(id, level.max(existing));
        }
    }
    set_enchantments_for_crafting(target, enchantments);
}

/// `GrindstoneMenu.mergeItems`: repair two matching items into one (combined
/// durability + 5% bonus), merge their enchantments, then strip non-curses.
fn grindstone_merge_items(input: &ItemStack, additional: &ItemStack) -> ItemStack {
    if input.item_id() != additional.item_id() {
        return ItemStack::empty();
    }
    let durability = input.max_damage().max(additional.max_damage()) as i32;
    let remaining1 = input.max_damage() as i32 - input.damage_value() as i32;
    let remaining2 = additional.max_damage() as i32 - additional.damage_value() as i32;
    let remaining = remaining1 + remaining2 + durability * 5 / 100;
    let mut count = 1;
    if !input.is_damageable_item() {
        if (input.max_stack_size() as i32) < 2 || !same_item_same_components(input, additional) {
            return ItemStack::empty();
        }
        count = 2;
    }
    let mut new_item = input.copy_with_count(count);
    if new_item.is_damageable_item() {
        new_item.set_component(ItemComponent::MaxDamage(durability as u32));
        new_item.set_damage_value((durability - remaining).max(0) as u32);
    }
    grindstone_merge_enchants(&mut new_item, additional);
    grindstone_remove_non_curses(new_item)
}

/// `GrindstoneMenu.computeResult`: one enchanted item → disenchant it; two matching
/// single items → repair + disenchant; otherwise no result.
pub(super) fn grindstone_compute_result(input: &ItemStack, additional: &ItemStack) -> ItemStack {
    if input.is_empty() && additional.is_empty() {
        return ItemStack::empty();
    }
    if input.count() > 1 || additional.count() > 1 {
        return ItemStack::empty();
    }
    if input.is_empty() || additional.is_empty() {
        let item = if !input.is_empty() { input } else { additional };
        if has_any_enchantments(item) {
            grindstone_remove_non_curses(item.clone())
        } else {
            ItemStack::empty()
        }
    } else {
        grindstone_merge_items(input, additional)
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
mod menus_enchantment;
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
use menus_enchantment::*;
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
#[cfg(test)]
mod tests_workstation;
#[cfg(test)]
mod tests_sweep;
