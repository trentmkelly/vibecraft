//! Port of `net.minecraft.util.datafix.fixes.EntityTheRenameningFix`: the 1.13
//! entity, block and item renames.

use crate::datafix::fix::Fix;

use super::simplest_entity_rename_fix::simplest_entity_rename_fix;

pub use super::entity_the_renamening_fix_ids::{RENAMED_BLOCKS, RENAMED_IDS, RENAMED_ITEMS_EXTRA};

const MINECRAFT_BRED: &str = "minecraft:bred_";

/// `EntityTheRenameningFix.RENAMED_ITEMS`: `RENAMED_BLOCKS` plus the item-only entries.
pub fn renamed_items() -> Vec<(&'static str, &'static str)> {
    RENAMED_BLOCKS
        .iter()
        .chain(RENAMED_ITEMS_EXTRA.iter())
        .copied()
        .collect()
}

/// `new EntityTheRenameningFix(schema, changesType)`.
pub fn fix() -> Fix {
    simplest_entity_rename_fix("EntityTheRenameningBlock", |name| {
        let name = match name.strip_prefix(MINECRAFT_BRED) {
            Some(rest) => format!("minecraft:{rest}"),
            None => name.to_string(),
        };
        RENAMED_IDS
            .iter()
            .find(|(old, _)| *old == name)
            .map_or(name.as_str(), |(_, new)| *new)
            .to_string()
    })
}
