//! Port of `net.minecraft.util.datafix.fixes.EntityPufferfishRenameFix`.

use crate::datafix::fix::Fix;

use super::simplest_entity_rename_fix::simplest_entity_rename_fix;

/// `EntityPufferfishRenameFix.RENAMED_IDS`.
pub const RENAMED_IDS: &[(&str, &str)] = &[(
    "minecraft:puffer_fish_spawn_egg",
    "minecraft:pufferfish_spawn_egg",
)];

/// `new EntityPufferfishRenameFix(schema, changesType)`.
pub fn fix() -> Fix {
    simplest_entity_rename_fix("EntityPufferfishRenameFix", |name| {
        if name == "minecraft:puffer_fish" {
            "minecraft:pufferfish".to_string()
        } else {
            name.to_string()
        }
    })
}
