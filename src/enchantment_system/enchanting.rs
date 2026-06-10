//! Enchantment helpers that are not the core registry/effect tables: curse +
//! cost queries, `canEnchant`/`isPrimaryItem`, and the enchanting-table selection
//! algorithm (`getEnchantmentCost` / `selectEnchantment`). Split out of the parent
//! to keep each file under the 1200-line limit. Re-exported from `enchantment_system`.

use super::*;

pub fn is_curse(id: &str) -> bool {
    enchantment(id).is_some_and(|def| def.groups.contains(&EnchantmentGroup::Curse))
}

/// `Enchantment.getMinCost(level)` — the minimum enchanting cost at a level, used by
/// the grindstone XP calculation and enchanting-table seeding. Unknown ids cost 0.
pub fn min_cost_for(id: &str, level: i32) -> i32 {
    enchantment(id).map_or(0, |def| def.min_cost.calculate(level))
}

/// `Enchantment.canEnchant(item)`: whether the enchantment may be applied to the item,
/// i.e. the item is a member of the enchantment's `supported_items` tag (resolved by
/// `item_tags`). Used by the anvil combine and the enchanting table. Unknown ids → false.
pub fn can_enchant(item_id: &str, enchantment_id: &str) -> bool {
    enchantment(enchantment_id)
        .is_some_and(|def| crate::item_tags::item_in_tag(item_id, def.supported_items))
}

/// The `primary_items` tag for an enchantment when it differs from `supported_items`
/// (the enchanting table offers an enchantment only on its primary items, while the
/// anvil accepts any supported item). 26.1.2 declares these for five enchantments;
/// the rest default to their `supported_items`.
fn primary_items_override(enchantment_id: &str) -> Option<&'static str> {
    match enchantment_id {
        "minecraft:sharpness"
        | "minecraft:smite"
        | "minecraft:bane_of_arthropods"
        | "minecraft:fire_aspect" => Some("#minecraft:enchantable/melee_weapon"),
        "minecraft:thorns" => Some("#minecraft:enchantable/chest_armor"),
        _ => None,
    }
}

/// `Enchantment.isPrimaryItem`: `isSupportedItem(item) && (primaryItems.isEmpty() ||
/// item in primaryItems)`. Used by the enchanting table's offer selection. Unknown
/// ids → false.
pub fn is_primary_item(item_id: &str, enchantment_id: &str) -> bool {
    let Some(def) = enchantment(enchantment_id) else {
        return false;
    };
    if !crate::item_tags::item_in_tag(item_id, def.supported_items) {
        return false;
    }
    match def
        .primary_items
        .or_else(|| primary_items_override(enchantment_id))
    {
        Some(primary) => crate::item_tags::item_in_tag(item_id, primary),
        None => true,
    }
}

/// `EnchantmentHelper.getEnchantmentCost`: the enchanting-table level for `slot` (0..3)
/// given the item's `enchantability` (the `ENCHANTABLE` component value) and the number
/// of nearby `bookcases` (capped at 15). `random` must be a freshly seeded Java RNG
/// (`LegacyRandom`) — the two `nextInt` draws here are part of the per-table sequence.
pub fn get_enchantment_cost(
    random: &mut crate::random_source::LegacyRandom,
    slot: i32,
    mut bookcases: i32,
    enchantability: i32,
) -> i32 {
    if enchantability <= 0 {
        return 0;
    }
    if bookcases > 15 {
        bookcases = 15;
    }
    let selected =
        random.next_i32_bound(8) + 1 + (bookcases >> 1) + random.next_i32_bound(bookcases + 1);
    match slot {
        0 => (selected / 3).max(1),
        1 => selected * 2 / 3 + 1,
        _ => selected.max(bookcases * 2),
    }
}

/// `EnchantmentTags.IN_ENCHANTING_TABLE` (= `#non_treasure`) in tag order. The order is
/// authoritative because `WeightedRandom` walks the candidate list in source order, so
/// it must match `data/minecraft/tags/enchantment/non_treasure.json`.
pub const IN_ENCHANTING_TABLE: &[&str] = &[
    "minecraft:protection",
    "minecraft:fire_protection",
    "minecraft:feather_falling",
    "minecraft:blast_protection",
    "minecraft:projectile_protection",
    "minecraft:respiration",
    "minecraft:aqua_affinity",
    "minecraft:thorns",
    "minecraft:depth_strider",
    "minecraft:sharpness",
    "minecraft:smite",
    "minecraft:bane_of_arthropods",
    "minecraft:knockback",
    "minecraft:fire_aspect",
    "minecraft:looting",
    "minecraft:sweeping_edge",
    "minecraft:efficiency",
    "minecraft:silk_touch",
    "minecraft:unbreaking",
    "minecraft:fortune",
    "minecraft:power",
    "minecraft:punch",
    "minecraft:flame",
    "minecraft:infinity",
    "minecraft:luck_of_the_sea",
    "minecraft:lure",
    "minecraft:loyalty",
    "minecraft:impaling",
    "minecraft:riptide",
    "minecraft:channeling",
    "minecraft:multishot",
    "minecraft:quick_charge",
    "minecraft:piercing",
    "minecraft:density",
    "minecraft:breach",
    "minecraft:lunge",
];

fn enchants_compatible_ids(a: &str, b: &str) -> bool {
    match (enchantment(a), enchantment(b)) {
        (Some(left), Some(right)) => are_compatible(left, right),
        _ => a != b,
    }
}

/// `EnchantmentHelper.getAvailableEnchantmentResults`: every `IN_ENCHANTING_TABLE`
/// enchantment that is a primary item for the stack (or the stack is a plain book), at
/// the highest level whose `[minCost, maxCost]` range contains `value`. Preserves tag
/// order. Returns `(id, level, weight)`.
pub fn available_enchantment_results(value: i32, item_id: &str) -> Vec<(&'static str, i32, i32)> {
    let is_book = item_id == "minecraft:book";
    let mut results = Vec::new();
    for &id in IN_ENCHANTING_TABLE {
        let Some(def) = enchantment(id) else { continue };
        if !(is_book || is_primary_item(item_id, id)) {
            continue;
        }
        for level in (1..=def.max_level).rev() {
            if value >= def.min_cost.calculate(level) && value <= def.max_cost.calculate(level) {
                results.push((id, level, def.weight));
                break;
            }
        }
    }
    results
}

/// `WeightedRandom.getRandomItem` over `(id, level, weight)` candidates: draw
/// `nextInt(totalWeight)` and walk the list subtracting weights. Returns the index.
fn weighted_pick(
    random: &mut crate::random_source::LegacyRandom,
    items: &[(&'static str, i32, i32)],
) -> Option<usize> {
    let total: i32 = items.iter().map(|(_, _, w)| *w).sum();
    if total <= 0 {
        return None;
    }
    let mut selection = random.next_i32_bound(total);
    for (index, (_, _, weight)) in items.iter().enumerate() {
        selection -= *weight;
        if selection < 0 {
            return Some(index);
        }
    }
    None
}

/// `EnchantmentHelper.selectEnchantment`: the seeded list of `(id, level)` the enchanting
/// table applies for a base `cost`, given the item's `enchantability`. Mirrors the Java
/// call sequence exactly (so the verified Java-compatible `LegacyRandom` reproduces it).
pub fn select_enchantment(
    random: &mut crate::random_source::LegacyRandom,
    item_id: &str,
    mut cost: i32,
    enchantability: i32,
) -> Vec<(&'static str, i32)> {
    let mut results: Vec<(&'static str, i32)> = Vec::new();
    if enchantability <= 0 {
        return results;
    }
    cost += 1
        + random.next_i32_bound(enchantability / 4 + 1)
        + random.next_i32_bound(enchantability / 4 + 1);
    let random_span = (random.next_f32() + random.next_f32() - 1.0) * 0.15;
    cost = ((cost as f32 + cost as f32 * random_span).round() as i32).max(1);

    let mut candidates = available_enchantment_results(cost, item_id);
    if candidates.is_empty() {
        return results;
    }
    if let Some(i) = weighted_pick(random, &candidates) {
        results.push((candidates[i].0, candidates[i].1));
    }
    while random.next_i32_bound(50) <= cost {
        if let Some((last_id, _)) = results.last() {
            // filterCompatibleEnchantments: keep only those compatible with the last
            // pick (which also drops the last pick itself, since are_compatible(x, x)
            // is false).
            let last = *last_id;
            candidates.retain(|(id, _, _)| enchants_compatible_ids(last, id));
        }
        if candidates.is_empty() {
            break;
        }
        if let Some(i) = weighted_pick(random, &candidates) {
            results.push((candidates[i].0, candidates[i].1));
        }
        cost /= 2;
    }
    results
}
