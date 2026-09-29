//! Port of `net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_stack_spawn_egg_fix_map::MAP;

/// The id of `TaggedChoiceType.point` for the entity registry of the 1451.5 schema:
/// DFU returns the first entry of a hash map, which is the wither there (verified
/// against the real fixer).
const DEFAULT_ENTITY_POINT: &str = "minecraft:wither";

/// `new ItemStackSpawnEggFix(schema, changesType, itemType)`: the generic spawn
/// egg item becomes the egg of the entity in its `EntityTag`.
pub fn fix(item_type: &'static str) -> Fix {
    Fix::everywhere(
        "ItemInstanceSpawnEggFix",
        Target::Type(r::ITEM_STACK),
        move |stack| {
            if get_str(stack, "id") != Some(item_type) {
                return;
            }
            // `getOrCreateTyped` builds the default entity tree when there is none,
            // and the default point of the entity choice is its first entry.
            let entity_id = typed(stack, "tag")
                .and_then(|tag| typed(tag, "EntityTag"))
                .and_then(|entity| get_str(entity, "id"))
                .unwrap_or(DEFAULT_ENTITY_POINT);
            let egg = MAP
                .iter()
                .find(|(entity, _)| *entity == entity_id)
                .map_or("minecraft:pig_spawn_egg", |(_, egg)| *egg);
            set(stack, "id", Tag::String(egg.to_string()));
        },
    )
}
