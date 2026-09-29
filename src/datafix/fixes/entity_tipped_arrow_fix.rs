//! Port of `net.minecraft.util.datafix.fixes.EntityTippedArrowFix`.

use crate::datafix::fix::Fix;

use super::simplest_entity_rename_fix::simplest_entity_rename_fix;

/// `new EntityTippedArrowFix(schema, changesType)`.
pub fn fix() -> Fix {
    simplest_entity_rename_fix("EntityTippedArrowFix", |name| {
        if name == "TippedArrow" {
            "Arrow".to_string()
        } else {
            name.to_string()
        }
    })
}
