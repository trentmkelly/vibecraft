//! Port of `net.minecraft.util.datafix.fixes.BlockEntityJukeboxFix`.

use crate::datafix::decode::decode;
use crate::datafix::dynamic::{compound, get_i32_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::Tmpl;
use crate::storage::nbt::Tag;

use super::item_id_fix::get_item;
use super::item_stack_the_flattening_fix::update_item;

/// `new BlockEntityJukeboxFix(schema, changesType)`: the numeric `Record` id
/// becomes a `RecordItem` stack.
///
/// The Java code discards the result of `tag.remove("Record")`, so `Record`
/// (faithfully) stays in the block entity.
pub fn fix() -> Fix {
    Fix::everywhere_with(
        "BlockEntityJukeboxFix",
        crate::datafix::walk::Target::block_entity("minecraft:jukebox"),
        |ctx, jukebox| {
            let record_id = get_i32_or(jukebox, "Record", 0);
            if record_id <= 0 {
                return;
            }
            let Some(id) = update_item(get_item(record_id), 0) else {
                return;
            };
            let mut record_item = compound(vec![("id", Tag::String(id)), ("Count", Tag::Byte(1))]);
            // `itemStackType.readTyped(..)` with the input schema's stack type.
            let _ = decode(ctx.input, &Tmpl::Ref(r::ITEM_STACK), &mut record_item);
            set(jukebox, "RecordItem", record_item);
        },
    )
}
