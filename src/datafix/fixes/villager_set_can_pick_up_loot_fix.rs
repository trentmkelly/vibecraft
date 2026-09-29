//! Port of `net.minecraft.util.datafix.fixes.VillagerSetCanPickUpLootFix`.

use crate::datafix::dynamic::{boolean, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;

use super::named_entity_fix::named_entity_fix;

/// `new VillagerSetCanPickUpLootFix(schema)`.
pub fn fix() -> Fix {
    named_entity_fix(
        "Villager CanPickUpLoot default value",
        r::ENTITY,
        "Villager",
        |tag| set(tag, "CanPickUpLoot", boolean(true)),
    )
}
