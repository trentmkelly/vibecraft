mod block_items;
mod block_state_aliases;
mod non_block_items;

use super::ItemCatalogEntry;

pub(super) const PRIMARY_ITEM_PROTOCOL_CATALOG: &[&[ItemCatalogEntry]] = &[
    block_items::ITEM_PROTOCOL_BLOCK_ITEMS,
    non_block_items::ITEM_PROTOCOL_NON_BLOCK_ITEMS,
];

pub(super) const ITEM_PROTOCOL_CATALOG: &[&[ItemCatalogEntry]] = &[
    block_items::ITEM_PROTOCOL_BLOCK_ITEMS,
    non_block_items::ITEM_PROTOCOL_NON_BLOCK_ITEMS,
    block_state_aliases::ITEM_PROTOCOL_BLOCK_STATE_ALIASES,
];

#[cfg(test)]
pub(super) const PRIMARY_ITEM_PROTOCOL_CATALOG_LEN: usize = block_items::ITEM_PROTOCOL_BLOCK_ITEMS
    .len()
    + non_block_items::ITEM_PROTOCOL_NON_BLOCK_ITEMS.len();
