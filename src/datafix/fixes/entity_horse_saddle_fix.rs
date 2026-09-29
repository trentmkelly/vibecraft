//! Port of `net.minecraft.util.datafix.fixes.EntityHorseSaddleFix`.

use crate::datafix::dynamic::{compound, get_bool_or};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::{set_typed, typed};
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new EntityHorseSaddleFix(schema, changesType)`: the `Saddle` flag becomes a
/// `SaddleItem` stack.
///
/// The Java code calls `tag.remove("Saddle")` and discards the result, so the
/// flag is (faithfully) left in place.
pub fn fix() -> Fix {
    named_entity_fix("EntityHorseSaddleFix", r::ENTITY, "EntityHorse", |entity| {
        if typed(entity, "SaddleItem").is_none() && get_bool_or(entity, "Saddle", false) {
            let saddle = compound(vec![
                ("id", Tag::String("minecraft:saddle".to_string())),
                ("Count", Tag::Byte(1)),
                ("Damage", Tag::Short(0)),
            ]);
            set_typed(entity, "SaddleItem", saddle);
        }
    })
}
