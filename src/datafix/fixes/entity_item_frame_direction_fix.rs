//! Port of `net.minecraft.util.datafix.fixes.EntityItemFrameDirectionFix`.

use crate::datafix::dynamic::{get_i8_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

/// `new EntityItemFrameDirectionFix(schema, changesType)`: horizontal facings
/// become 3d directions.
pub fn fix() -> Fix {
    named_entity_fix(
        "EntityItemFrameDirectionFix",
        r::ENTITY,
        "minecraft:item_frame",
        |frame| {
            let facing = direction_2d_to_3d(get_i8_or(frame, "Facing", 0));
            set(frame, "Facing", Tag::Byte(facing));
        },
    )
}

fn direction_2d_to_3d(direction: i8) -> i8 {
    match direction {
        0 => 3,
        1 => 4,
        3 => 5,
        _ => 2,
    }
}
