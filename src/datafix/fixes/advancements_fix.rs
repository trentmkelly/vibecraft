//! Port of `net.minecraft.util.datafix.fixes.AdvancementsFix`: renamed recipe
//! advancements.

use crate::datafix::fix::Fix;

use super::advancements_fix_renames::RENAMES;
use super::advancements_rename_fix::advancements_rename_fix;

/// `new AdvancementsFix(schema, changesType)`.
pub fn fix() -> Fix {
    advancements_rename_fix("AdvancementsFix", |name| {
        RENAMES
            .iter()
            .find(|(old, _)| *old == name)
            .map_or(name, |(_, new)| *new)
            .to_string()
    })
}
