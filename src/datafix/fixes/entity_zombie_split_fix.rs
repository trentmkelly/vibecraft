//! Port of `net.minecraft.util.datafix.fixes.EntityZombieSplitFix`.

use crate::datafix::dynamic::{get_i32_or, remove, set};
use crate::datafix::fix::Fix;
use crate::storage::nbt::Tag;

use super::entity_rename_fix::entity_rename_fix;

/// `new EntityZombieSplitFix(schema)`.
pub fn fix() -> Fix {
    entity_rename_fix("EntityZombieSplitFix", |_, name, entity| {
        if name != "Zombie" {
            return name.to_string();
        }
        let zombie_type = get_i32_or(entity, "ZombieType", 0);
        let new_name = match zombie_type {
            1..=5 => {
                set(entity, "Profession", Tag::Int(zombie_type - 1));
                "ZombieVillager"
            }
            6 => "Husk",
            _ => "Zombie",
        };
        remove(entity, "ZombieType");
        new_name.to_string()
    })
}
