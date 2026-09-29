//! Port of `net.minecraft.util.datafix.fixes.ItemWaterPotionFix`.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::update_typed_compound;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemWaterPotionFix(schema, changesType)`: potions without a `Potion`
/// tag become water potions.
pub fn fix() -> Fix {
    Fix::everywhere("ItemWaterPotionFix", Target::Type(r::ITEM_STACK), |stack| {
        let is_potion = matches!(
            get_str(stack, "id"),
            Some(
                "minecraft:potion"
                    | "minecraft:splash_potion"
                    | "minecraft:lingering_potion"
                    | "minecraft:tipped_arrow"
            )
        );
        if !is_potion {
            return;
        }
        update_typed_compound(stack, "tag", |tag| {
            if get_str(tag, "Potion").is_none() {
                set(tag, "Potion", Tag::String("minecraft:water".to_string()));
            }
        });
    })
}
