//! Port of `net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix`.

use crate::datafix::dynamic::{get_i32_or, remove};
use crate::datafix::fix::Fix;

use super::entity_rename_fix::entity_rename_fix;

/// `new EntityMinecartIdentifiersFix(schema)`.
pub fn fix() -> Fix {
    entity_rename_fix("EntityMinecartIdentifiersFix", |_, name, entity| {
        if name != "Minecart" {
            return name.to_string();
        }
        let new_name = match get_i32_or(entity, "Type", 0) {
            1 => "MinecartChest",
            2 => "MinecartFurnace",
            _ => "MinecartRideable",
        };
        remove(entity, "Type");
        new_name.to_string()
    })
}
