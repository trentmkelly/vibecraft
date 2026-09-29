//! Port of `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix`.

use crate::datafix::dynamic::{as_f64, get, get_i32_or, get_i8_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;
use crate::datafix::references as r;

const DIRECTIONS: [[i32; 3]; 4] = [[0, 0, 1], [-1, 0, 0], [0, 0, -1], [1, 0, 0]];

/// `new EntityPaintingItemFrameDirectionFix(schema, changesType)`.
pub fn fix() -> Fix {
    // Both rules in the Java code look the entity up by choice name.
    let _ = Target::entity;
    Fix::sequence(
        "EntityPaintingItemFrameDirectionFix",
        vec![
            named_entity_fix("EntityPaintingFix", r::ENTITY, "Painting", |tag| {
                do_fix(tag, true, false)
            }),
            named_entity_fix("EntityItemFrameFix", r::ENTITY, "ItemFrame", |tag| {
                do_fix(tag, false, true)
            }),
        ],
    )
}

/// `EntityPaintingItemFrameDirectionFix.doFix`.
fn do_fix(input: &mut Tag, is_painting: bool, is_item_frame: bool) {
    if !(is_painting || is_item_frame) || get(input, "Facing").and_then(as_f64).is_some() {
        return;
    }
    let direction;
    if get(input, "Direction").and_then(as_f64).is_some() {
        // Java indexes with the (possibly negative) remainder and would throw.
        let Some(index) = usize::try_from(get_i8_or(input, "Direction", 0) % 4).ok() else {
            return;
        };
        direction = index;
        let steps = DIRECTIONS[direction];
        for (axis, step) in ["TileX", "TileY", "TileZ"].into_iter().zip(steps) {
            let value = get_i32_or(input, axis, 0).wrapping_add(step);
            set(input, axis, Tag::Int(value));
        }
        remove(input, "Direction");
        if is_item_frame && get(input, "ItemRotation").and_then(as_f64).is_some() {
            let rotation = i32::from(get_i8_or(input, "ItemRotation", 0)) * 2;
            set(input, "ItemRotation", Tag::Byte(rotation as i8));
        }
    } else {
        let Some(index) = usize::try_from(get_i8_or(input, "Dir", 0) % 4).ok() else {
            return;
        };
        direction = index;
        remove(input, "Dir");
    }
    set(input, "Facing", Tag::Byte(direction as i8));
}
