//! Port of `net.minecraft.util.datafix.fixes.EntityCodSalmonFix`.

use crate::datafix::fix::Fix;

use super::simplest_entity_rename_fix::simplest_entity_rename_fix;

/// `EntityCodSalmonFix.RENAMED_IDS`.
pub const RENAMED_IDS: &[(&str, &str)] = &[
    ("minecraft:salmon_mob", "minecraft:salmon"),
    ("minecraft:cod_mob", "minecraft:cod"),
];

/// `EntityCodSalmonFix.RENAMED_EGG_IDS`.
pub const RENAMED_EGG_IDS: &[(&str, &str)] = &[
    (
        "minecraft:salmon_mob_spawn_egg",
        "minecraft:salmon_spawn_egg",
    ),
    ("minecraft:cod_mob_spawn_egg", "minecraft:cod_spawn_egg"),
];

/// `new EntityCodSalmonFix(schema, changesType)`.
pub fn fix() -> Fix {
    simplest_entity_rename_fix("EntityCodSalmonFix", |name| {
        RENAMED_IDS
            .iter()
            .find(|(old, _)| *old == name)
            .map_or(name, |(_, new)| *new)
            .to_string()
    })
}
