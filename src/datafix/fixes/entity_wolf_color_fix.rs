//! Port of `net.minecraft.util.datafix.fixes.EntityWolfColorFix`.

use crate::datafix::dynamic::{as_i32, get_mut};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new EntityWolfColorFix(schema, changesType)`: collar colours switch from the
/// wool scale to the dye scale.
pub fn fix() -> Fix {
    named_entity_fix("EntityWolfColorFix", r::ENTITY, "minecraft:wolf", |wolf| {
        if let Some(color) = get_mut(wolf, "CollarColor") {
            let old = as_i32(color).unwrap_or(0);
            *color = Tag::Byte((15 - old) as i8);
        }
    })
}
