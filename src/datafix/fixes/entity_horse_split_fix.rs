//! Port of `net.minecraft.util.datafix.fixes.EntityHorseSplitFix`.

use crate::datafix::dynamic::{get_i32_or, remove};
use crate::datafix::fix::Fix;

use super::entity_rename_fix::entity_rename_fix;

/// `new EntityHorseSplitFix(schema, changesType)`.
pub fn fix() -> Fix {
    entity_rename_fix("EntityHorseSplitFix", |_, name, entity| {
        if name != "EntityHorse" {
            return name.to_string();
        }
        let new_name = match get_i32_or(entity, "Type", 0) {
            1 => "Donkey",
            2 => "Mule",
            3 => "ZombieHorse",
            4 => "SkeletonHorse",
            _ => "Horse",
        };
        remove(entity, "Type");
        new_name.to_string()
    })
}
