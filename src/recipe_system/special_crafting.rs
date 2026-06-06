//! Live, stack-aware implementations of the vanilla `CustomRecipe` special
//! crafting recipes (`net.minecraft.world.item.crafting.*Recipe`).
//!
//! Unlike the ordinary `RecipeKind::matches`/`assemble` path — which only sees
//! item ids — these operate on the real 3×3 grid of [`ItemStack`]s so the result
//! carries the correct data components (durability, enchantments, banner
//! patterns, …). Each function mirrors its Java recipe's `matches` + `assemble`
//! (+ `getRemainingItems`) exactly. [`special_crafting_result`] dispatches by the
//! loaded recipe's [`SpecialRecipeKind`]; [`RecipeMap::special_crafting_result`]
//! finds the first registered special recipe that matches the current grid.

use std::collections::BTreeMap;

use super::recipe_results::SpecialRecipeKind;
use super::ItemAmount;
use crate::item_properties::ItemComponent;
use crate::item_stack::ItemStack;

/// The outcome of a successful special craft: the result stack plus the grid's
/// state after one craft is taken. `grid_after[i]` is what input slot `i` becomes
/// once the result is removed — usually empty, but e.g. the patterned banner or
/// written book is left behind by the duplicate/clone recipes.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialCraftOutcome {
    pub result: ItemStack,
    pub grid_after: Vec<ItemStack>,
}

/// Evaluate one special recipe of the given `kind` against the live grid.
///
/// `result_hint` is the recipe's configured `result` template (item id + count),
/// captured from the recipe JSON — e.g. the colour-specific banner for a banner
/// duplicate, or `firework_rocket × 3`. It is `None` for the recipes whose result
/// is fully dynamic (repair, map cloning).
pub fn special_crafting_result(
    kind: SpecialRecipeKind,
    result_hint: Option<&ItemAmount>,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    match kind {
        SpecialRecipeKind::RepairItem => repair_item(grid),
        SpecialRecipeKind::ShieldDecoration => shield_decoration(result_hint?, grid),
        SpecialRecipeKind::BannerDuplicate => banner_duplicate(result_hint?, grid),
        // TODO(recipes-special): port the remaining `CustomRecipe` assemblers to
        // real `ItemStack`s — book cloning, decorated pot, dye, the three firework
        // recipes, and map cloning/extending. Until each is implemented here it
        // does not function in-game.
        _ => None,
    }
}

/// The non-empty grid stacks paired with their slot index.
fn present_indexed(grid: &[ItemStack]) -> Vec<(usize, &ItemStack)> {
    grid.iter()
        .enumerate()
        .filter(|(_, stack)| !stack.is_empty())
        .collect()
}

/// Whether an item id is one of the 16 `<colour>_banner` `BannerItem`s.
fn is_banner(item_id: &str) -> bool {
    banner_base_color(item_id).is_some()
}

/// `BannerItem.getColor()` — the intrinsic dye colour of a `<colour>_banner`, as the
/// `DyeColor` id (e.g. `"red"`). `None` for non-banner items.
fn banner_base_color(item_id: &str) -> Option<&'static str> {
    const COLORS: &[&str] = &[
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ];
    let name = item_id.strip_prefix("minecraft:")?.strip_suffix("_banner")?;
    COLORS.iter().copied().find(|color| *color == name)
}

/// The `banner_patterns` layers carried by a stack, if any.
fn banner_patterns(stack: &ItemStack) -> Option<Vec<crate::block_entity::BannerPatternLayer>> {
    match stack.component("minecraft:banner_patterns") {
        Some(ItemComponent::BannerPatterns(layers)) => Some(layers.clone()),
        _ => None,
    }
}

/// The number of `banner_patterns` layers on a stack (0 if the component is absent).
fn banner_pattern_count(stack: &ItemStack) -> usize {
    banner_patterns(stack).map_or(0, |layers| layers.len())
}

// ----------------------------------------------------------------------------
// ShieldDecorationRecipe
// ----------------------------------------------------------------------------

/// `ShieldDecorationRecipe` — a banner + a pattern-free shield yields a shield
/// carrying the banner's pattern layers and base colour.
fn shield_decoration(
    result_hint: &ItemAmount,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    let present = present_indexed(grid);
    if present.len() != 2 {
        return None;
    }
    let mut banner = None;
    let mut target = None;
    for (_, stack) in &present {
        if is_banner(stack.item_id()) {
            if banner.replace(*stack).is_some() {
                return None;
            }
        } else if stack.item_id() == "minecraft:shield" && banner_pattern_count(stack) == 0 {
            if target.replace(*stack).is_some() {
                return None;
            }
        } else {
            return None;
        }
    }
    let banner = banner?;
    let target = target?;

    // createWithOriginalComponents(result, target) + BANNER_PATTERNS + BASE_COLOR.
    let mut result = target.transmute_copy(result_hint.item, 1);
    match banner_patterns(banner) {
        Some(layers) => {
            result.set_component(ItemComponent::BannerPatterns(layers));
        }
        // A pattern-free banner clears any patterns and contributes only the colour.
        None => {
            result.remove_component("minecraft:banner_patterns");
        }
    }
    if let Some(color) = banner_base_color(banner.item_id()) {
        result.set_component(ItemComponent::BaseColor(color));
    }
    Some(SpecialCraftOutcome {
        result,
        grid_after: vec![ItemStack::empty(); grid.len()],
    })
}

// ----------------------------------------------------------------------------
// BannerDuplicateRecipe
// ----------------------------------------------------------------------------

/// `BannerDuplicateRecipe` — a patterned banner + a same-colour blank banner copies
/// the patterns onto the blank one; the patterned source banner is left behind.
fn banner_duplicate(
    result_hint: &ItemAmount,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    // This recipe is colour-specific (one per dye), so the inputs must be the same
    // colour as the configured result banner.
    let recipe_color = banner_base_color(result_hint.item)?;
    let present = present_indexed(grid);
    if present.len() != 2 {
        return None;
    }
    let mut source = None;
    let mut target = false;
    for (_, stack) in &present {
        if banner_base_color(stack.item_id()) != Some(recipe_color) {
            return None;
        }
        let count = banner_pattern_count(stack);
        if count > 6 {
            return None;
        }
        if count > 0 {
            if source.replace(*stack).is_some() {
                return None;
            }
        } else if std::mem::replace(&mut target, true) {
            return None;
        }
    }
    let source = source?;
    if !target {
        return None;
    }

    let result = source.transmute_copy(result_hint.item, 1);
    // getRemainingItems: the patterned source banner stays (copyWithCount 1), the
    // blank one is consumed.
    let grid_after = grid
        .iter()
        .map(|stack| {
            if banner_pattern_count(stack) > 0 {
                stack.copy_with_count(1)
            } else {
                ItemStack::empty()
            }
        })
        .collect();
    Some(SpecialCraftOutcome { result, grid_after })
}

// ----------------------------------------------------------------------------
// RepairItemRecipe
// ----------------------------------------------------------------------------

/// `RepairItemRecipe` — combine two damaged items of the same type into one with
/// the summed remaining durability plus a 5%-of-max bonus, carrying over only the
/// items' curse enchantments (at the higher of the two levels).
fn repair_item(grid: &[ItemStack]) -> Option<SpecialCraftOutcome> {
    // `getItemsToCombine`: exactly two non-empty inputs that `canCombine`.
    let present: Vec<(usize, &ItemStack)> = grid
        .iter()
        .enumerate()
        .filter(|(_, stack)| !stack.is_empty())
        .collect();
    let [(first_idx, first), (second_idx, second)] = present.as_slice() else {
        return None;
    };
    if !repair_can_combine(first, second) {
        return None;
    }

    // `assemble`.
    let durability = first.max_damage().max(second.max_damage());
    let remaining = (first.max_damage() - first.damage_value())
        + (second.max_damage() - second.damage_value())
        + durability * 5 / 100;
    let mut result = ItemStack::new(first.item_id(), 1);
    result.set_component(ItemComponent::MaxDamage(durability));
    result.set_damage_value(durability.saturating_sub(remaining));

    // Only `EnchantmentTags.CURSE` enchantments survive (max level of the two).
    let mut curses: BTreeMap<String, i32> = BTreeMap::new();
    for stack in [first, second] {
        for (id, level) in crafting_enchantments(stack) {
            if crate::enchantment_system::is_curse(&id) {
                let entry = curses.entry(id).or_insert(0);
                *entry = (*entry).max(level);
            }
        }
    }
    if !curses.is_empty() {
        result.set_component(ItemComponent::Enchantments(curses));
    }

    // Both single-count inputs are consumed.
    let mut grid_after = grid.to_vec();
    grid_after[*first_idx] = ItemStack::empty();
    grid_after[*second_idx] = ItemStack::empty();
    Some(SpecialCraftOutcome { result, grid_after })
}

/// `RepairItemRecipe.canCombine`: same item, both single, both damageable.
fn repair_can_combine(first: &ItemStack, second: &ItemStack) -> bool {
    second.item_id() == first.item_id()
        && first.count() == 1
        && second.count() == 1
        && first.component("minecraft:max_damage").is_some()
        && second.component("minecraft:max_damage").is_some()
        && first.component("minecraft:damage").is_some()
        && second.component("minecraft:damage").is_some()
}

/// `EnchantmentHelper.getEnchantmentsForCrafting`: the enchantments used for
/// crafting carry-over (the `enchantments` component for ordinary items).
fn crafting_enchantments(stack: &ItemStack) -> BTreeMap<String, i32> {
    match stack.component("minecraft:enchantments") {
        Some(ItemComponent::Enchantments(map)) => map.clone(),
        _ => BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn damaged(item: &'static str, max: u32, damage: u32) -> ItemStack {
        let mut stack = ItemStack::new(item, 1);
        stack.set_component(ItemComponent::MaxDamage(max));
        stack.set_damage_value(damage);
        stack
    }

    fn grid_with(entries: &[(usize, ItemStack)]) -> Vec<ItemStack> {
        let mut grid = vec![ItemStack::empty(); 9];
        for (index, stack) in entries {
            grid[*index] = stack.clone();
        }
        grid
    }

    #[test]
    fn repair_item_sums_remaining_durability_with_five_percent_bonus() {
        // remaining = (100-80) + (100-90) + 100*5/100 = 20 + 10 + 5 = 35
        // result damage = max(100 - 35, 0) = 65   (matches RepairItemRecipe.assemble)
        let grid = grid_with(&[
            (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
            (4, damaged("minecraft:diamond_pickaxe", 100, 90)),
        ]);
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid)
            .expect("two damaged matching tools should repair");
        assert_eq!(outcome.result.item_id(), "minecraft:diamond_pickaxe");
        assert_eq!(outcome.result.count(), 1);
        assert_eq!(outcome.result.max_damage(), 100);
        assert_eq!(outcome.result.damage_value(), 65);
        // Both single-count inputs are consumed.
        assert!(outcome.grid_after[0].is_empty());
        assert!(outcome.grid_after[4].is_empty());
    }

    #[test]
    fn repair_item_carries_only_curse_enchantments() {
        let mut a = damaged("minecraft:diamond_pickaxe", 100, 80);
        a.set_component(ItemComponent::Enchantments(BTreeMap::from([
            ("minecraft:binding_curse".to_string(), 1),
            ("minecraft:efficiency".to_string(), 5),
        ])));
        let mut b = damaged("minecraft:diamond_pickaxe", 100, 90);
        b.set_component(ItemComponent::Enchantments(BTreeMap::from([(
            "minecraft:vanishing_curse".to_string(),
            1,
        )])));
        let grid = grid_with(&[(0, a), (1, b)]);
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).unwrap();
        match outcome.result.component("minecraft:enchantments") {
            Some(ItemComponent::Enchantments(map)) => {
                assert_eq!(map.get("minecraft:binding_curse"), Some(&1));
                assert_eq!(map.get("minecraft:vanishing_curse"), Some(&1));
                assert_eq!(map.get("minecraft:efficiency"), None, "non-curse dropped");
            }
            other => panic!("expected curse enchantments, got {other:?}"),
        }
    }

    fn banner(item: &'static str, pattern_count: usize) -> ItemStack {
        use crate::block_entity::BannerPatternLayer;
        use crate::map_state::DyeColor;
        let mut stack = ItemStack::new(item, 1);
        if pattern_count > 0 {
            let layers = (0..pattern_count)
                .map(|_| BannerPatternLayer {
                    pattern: "minecraft:stripe_top".to_string(),
                    color: DyeColor::White,
                })
                .collect();
            stack.set_component(ItemComponent::BannerPatterns(layers));
        }
        stack
    }

    #[test]
    fn shield_decoration_copies_banner_patterns_and_base_color() {
        let hint = ItemAmount::one("minecraft:shield");
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 3)),
            (1, ItemStack::new("minecraft:shield", 1)),
        ]);
        let outcome = special_crafting_result(
            SpecialRecipeKind::ShieldDecoration,
            Some(&hint),
            &grid,
        )
        .expect("banner + clear shield should decorate");
        assert_eq!(outcome.result.item_id(), "minecraft:shield");
        assert_eq!(
            outcome.result.component("minecraft:base_color"),
            Some(&ItemComponent::BaseColor("red"))
        );
        match outcome.result.component("minecraft:banner_patterns") {
            Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 3),
            other => panic!("expected 3 pattern layers, got {other:?}"),
        }
        // A shield already carrying patterns is rejected (not a clear target).
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 3)),
            (1, banner("minecraft:shield", 1)),
        ]);
        assert!(
            special_crafting_result(SpecialRecipeKind::ShieldDecoration, Some(&hint), &grid)
                .is_none()
        );
    }

    #[test]
    fn banner_duplicate_copies_patterns_and_keeps_source() {
        let hint = ItemAmount::one("minecraft:red_banner");
        let grid = grid_with(&[
            (0, banner("minecraft:red_banner", 4)), // source
            (1, banner("minecraft:red_banner", 0)), // blank target
        ]);
        let outcome =
            special_crafting_result(SpecialRecipeKind::BannerDuplicate, Some(&hint), &grid)
                .expect("patterned + blank same-colour banner should duplicate");
        match outcome.result.component("minecraft:banner_patterns") {
            Some(ItemComponent::BannerPatterns(layers)) => assert_eq!(layers.len(), 4),
            other => panic!("expected copied patterns, got {other:?}"),
        }
        // The patterned source banner is left behind; the blank one is consumed.
        assert_eq!(banner_pattern_count(&outcome.grid_after[0]), 4);
        assert!(outcome.grid_after[1].is_empty());

        // A mismatched recipe colour does not match these inputs.
        let other_hint = ItemAmount::one("minecraft:blue_banner");
        assert!(special_crafting_result(
            SpecialRecipeKind::BannerDuplicate,
            Some(&other_hint),
            &grid
        )
        .is_none());
    }

    #[test]
    fn repair_item_rejects_mismatched_or_multi_count_inputs() {
        // Different items.
        let grid = grid_with(&[
            (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
            (1, damaged("minecraft:iron_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

        // A stack of two is rejected.
        let mut pair = damaged("minecraft:diamond_pickaxe", 100, 80);
        pair.set_count(2);
        let grid = grid_with(&[
            (0, pair),
            (1, damaged("minecraft:diamond_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());

        // A non-damageable item is rejected.
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:stone", 1)),
            (1, ItemStack::new("minecraft:stone", 1)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, None, &grid).is_none());
    }
}
