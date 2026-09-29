//! Port of `net.minecraft.util.datafix.fixes.ItemPotionFix`: potion damage values
//! become the `Potion` tag.

use crate::datafix::dynamic::{get_i16_or, get_mut, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_potion_fix_potions::POTIONS;

const SPLASH: i32 = 16384;

/// `new ItemPotionFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere("ItemPotionFix", Target::Type(r::ITEM_STACK), fix_stack)
}

fn fix_stack(stack: &mut Tag) {
    if get_str(stack, "id") != Some("minecraft:potion") {
        return;
    }
    let damage = i32::from(get_i16_or(stack, "Damage", 0));
    // The item's `tag` compound is an optional typed field.
    if typed(stack, "tag").is_none() {
        return;
    }
    let mut splash = false;
    if let Some(tag) = get_mut(stack, "tag") {
        if get_str(tag, "Potion").is_none() {
            let potion = POTIONS[(damage & 127) as usize].unwrap_or("minecraft:water");
            set(tag, "Potion", Tag::String(potion.to_string()));
            splash = (damage & SPLASH) == SPLASH;
        }
    }
    if splash {
        set(
            stack,
            "id",
            Tag::String("minecraft:splash_potion".to_string()),
        );
    }
    if damage != 0 {
        set(stack, "Damage", Tag::Short(0));
    }
}
