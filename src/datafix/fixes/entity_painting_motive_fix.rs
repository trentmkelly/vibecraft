//! Port of `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix`.

use crate::datafix::dynamic::{ensure_namespaced, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `EntityPaintingMotiveFix.MAP`.
const MAP: &[(&str, &str)] = &[
    ("donkeykong", "donkey_kong"),
    ("burningskull", "burning_skull"),
    ("skullandroses", "skull_and_roses"),
];

/// `new EntityPaintingMotiveFix(schema, changesType)`: motives become lower case
/// namespaced ids.
pub fn fix() -> Fix {
    named_entity_fix(
        "EntityPaintingMotiveFix",
        r::ENTITY,
        "minecraft:painting",
        |painting| {
            let Some(motive) = get_str(painting, "Motive").map(str::to_lowercase) else {
                return;
            };
            let renamed = MAP
                .iter()
                .find(|(old, _)| *old == motive)
                .map_or(motive.as_str(), |(_, new)| *new);
            set(painting, "Motive", Tag::String(ensure_namespaced(renamed)));
        },
    )
}
