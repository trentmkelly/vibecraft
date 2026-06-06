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
pub fn special_crafting_result(
    kind: SpecialRecipeKind,
    grid: &[ItemStack],
) -> Option<SpecialCraftOutcome> {
    match kind {
        SpecialRecipeKind::RepairItem => repair_item(grid),
        // TODO(recipes-special): port the remaining `CustomRecipe` assemblers to
        // real `ItemStack`s — banner duplicate, book cloning, decorated pot, dye,
        // the three firework recipes, map cloning/extending, and shield
        // decoration. Until each is implemented here it does not function in-game.
        _ => None,
    }
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
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, &grid)
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
        let outcome = special_crafting_result(SpecialRecipeKind::RepairItem, &grid).unwrap();
        match outcome.result.component("minecraft:enchantments") {
            Some(ItemComponent::Enchantments(map)) => {
                assert_eq!(map.get("minecraft:binding_curse"), Some(&1));
                assert_eq!(map.get("minecraft:vanishing_curse"), Some(&1));
                assert_eq!(map.get("minecraft:efficiency"), None, "non-curse dropped");
            }
            other => panic!("expected curse enchantments, got {other:?}"),
        }
    }

    #[test]
    fn repair_item_rejects_mismatched_or_multi_count_inputs() {
        // Different items.
        let grid = grid_with(&[
            (0, damaged("minecraft:diamond_pickaxe", 100, 80)),
            (1, damaged("minecraft:iron_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, &grid).is_none());

        // A stack of two is rejected.
        let mut pair = damaged("minecraft:diamond_pickaxe", 100, 80);
        pair.set_count(2);
        let grid = grid_with(&[
            (0, pair),
            (1, damaged("minecraft:diamond_pickaxe", 100, 80)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, &grid).is_none());

        // A non-damageable item is rejected.
        let grid = grid_with(&[
            (0, ItemStack::new("minecraft:stone", 1)),
            (1, ItemStack::new("minecraft:stone", 1)),
        ]);
        assert!(special_crafting_result(SpecialRecipeKind::RepairItem, &grid).is_none());
    }
}
