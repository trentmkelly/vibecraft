//! Port of `net.minecraft.util.datafix.fixes.ItemBannerColorFix`.

use crate::datafix::dynamic::{as_f64, as_i32, get, get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ItemBannerColorFix(schema, changesType)`: the banner base colour moves
/// from the block entity tag to the item's damage value.
///
/// The Java code calls `blockEntityRest.remove("Base")` and discards the result,
/// so `Base` (faithfully) stays in the block entity tag, and the "(+NBT" lore
/// special case returns the same stack as the regular path.
pub fn fix() -> Fix {
    Fix::everywhere("ItemBannerColorFix", Target::Type(r::ITEM_STACK), fix_stack)
}

fn fix_stack(stack: &mut Tag) {
    if get_str(stack, "id") != Some("minecraft:banner") {
        return;
    }
    let Some(base) = typed(stack, "tag")
        .and_then(|tag| typed(tag, "BlockEntityTag"))
        .and_then(|block_entity| get(block_entity, "Base"))
        .filter(|base| as_f64(base).is_some())
    else {
        return;
    };
    let damage = (as_i32(base).unwrap_or(0) & 15) as i16;
    set(stack, "Damage", Tag::Short(damage));
}
