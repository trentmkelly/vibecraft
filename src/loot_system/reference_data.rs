//! Static vanilla facts the item functions read.
//!
//! Loot functions consult item data components (`max_damage`, `enchantable`,
//! `max_stack_size`, whether an item can carry `enchantments`) that the string-typed
//! [`LootStack`] does not carry. They come from
//! `vanilla-data/reports/item_loot_properties_26_1_2.json`, which
//! `tools/DumpItemLootProperties.java` dumps from the official server's bootstrapped
//! item registry (`Item.components()` after `DataComponentInitializers` ran), so the
//! values are the real ones rather than a hand-maintained table.

use std::collections::HashMap;
use std::sync::LazyLock;

/// The item data components loot functions read (`Item.components()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemLootProperties {
    /// `DataComponents.MAX_STACK_SIZE` (`Item.getDefaultMaxStackSize`).
    pub max_stack_size: i32,
    /// `DataComponents.MAX_DAMAGE`; present for damageable items.
    pub max_damage: Option<i32>,
    /// `DataComponents.ENCHANTABLE` (`Enchantable.value`).
    pub enchantable: Option<i32>,
    /// Whether the item's defaults hold `DataComponents.ENCHANTMENTS`.
    pub can_hold_enchantments: bool,
    /// Whether the item's defaults hold `DataComponents.STORED_ENCHANTMENTS`.
    pub can_hold_stored_enchantments: bool,
}

const ITEM_PROPERTIES_JSON: &str =
    include_str!("../../vanilla-data/reports/item_loot_properties_26_1_2.json");

static ITEM_PROPERTIES: LazyLock<HashMap<String, ItemLootProperties>> = LazyLock::new(|| {
    // The bundled report is validated by `item_properties_report_lists_every_item`.
    let Ok(serde_json::Value::Object(document)) = serde_json::from_str(ITEM_PROPERTIES_JSON)
    else {
        return HashMap::new();
    };
    let int = |entry: &serde_json::Value, key: &str| {
        entry
            .get(key)
            .and_then(serde_json::Value::as_i64)
            .map(|value| value as i32)
    };
    let flag = |entry: &serde_json::Value, key: &str| {
        entry
            .get(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    document
        .iter()
        .map(|(id, entry)| {
            let properties = ItemLootProperties {
                max_stack_size: int(entry, "max_stack_size").unwrap_or(1),
                max_damage: int(entry, "max_damage"),
                enchantable: int(entry, "enchantable"),
                can_hold_enchantments: flag(entry, "enchantments"),
                can_hold_stored_enchantments: flag(entry, "stored_enchantments"),
            };
            (id.clone(), properties)
        })
        .collect()
});

/// The loot-relevant item components of `item` (a namespaced registry id).
pub fn item_loot_properties(item: &str) -> Option<&'static ItemLootProperties> {
    ITEM_PROPERTIES.get(item)
}

/// The interned `'static` id of a vanilla item (recipes and the item registry key items
/// by `&'static str`); `None` for ids the registry does not define.
pub fn vanilla_item_id(item: &str) -> Option<&'static str> {
    ITEM_PROPERTIES.get_key_value(item).map(|(id, _)| id.as_str())
}

/// Every vanilla item id (unordered), for exhaustive checks.
pub fn vanilla_item_ids() -> impl Iterator<Item = &'static str> {
    ITEM_PROPERTIES.keys().map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_properties_report_lists_every_item() {
        assert!(vanilla_item_ids().count() > 1400);
        let sword = item_loot_properties("minecraft:diamond_sword").expect("sword");
        assert_eq!(sword.max_damage, Some(1561));
        assert_eq!(sword.enchantable, Some(10));
        assert_eq!(sword.max_stack_size, 1);
        let book = item_loot_properties("minecraft:enchanted_book").expect("book");
        assert!(book.can_hold_stored_enchantments);
        assert_eq!(vanilla_item_id("minecraft:stick"), Some("minecraft:stick"));
        assert_eq!(vanilla_item_id("minecraft:nope"), None);
    }
}
