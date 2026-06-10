// Intentional aliases for block-state names VibeCraft can receive from world data.
// They resolve to the canonical item used by Java for inventory/protocol encoding.

use super::super::ItemCatalogEntry;

macro_rules! catalog_alias {
    ($key:literal, $canonical_key:literal, $protocol_id:expr) => {
        ItemCatalogEntry {
            key: $key,
            static_name: concat!("minecraft:", $canonical_key),
            protocol_id: $protocol_id,
        }
    };
}

pub(super) const ITEM_PROTOCOL_BLOCK_STATE_ALIASES: &[ItemCatalogEntry] = &[
    catalog_alias!("lit_redstone_ore", "redstone_ore", 72),
    catalog_alias!("lit_deepslate_redstone_ore", "deepslate_redstone_ore", 73),
];
