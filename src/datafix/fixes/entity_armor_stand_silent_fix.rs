//! Port of `net.minecraft.util.datafix.fixes.EntityArmorStandSilentFix`.

use crate::datafix::dynamic::{get_bool_or, remove};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;

use super::named_entity_fix::named_entity_fix;

/// `new EntityArmorStandSilentFix(schema, changesType)`.
pub fn fix() -> Fix {
    named_entity_fix(
        "EntityArmorStandSilentFix",
        r::ENTITY,
        "ArmorStand",
        |tag| {
            if get_bool_or(tag, "Silent", false) && !get_bool_or(tag, "Marker", false) {
                remove(tag, "Silent");
            }
        },
    )
}
