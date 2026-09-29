//! Port of `net.minecraft.util.datafix.fixes.BlockEntityBlockStateFix`.

use crate::datafix::decode::decode;
use crate::datafix::dynamic::{get_i32_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::Tmpl;
use crate::datafix::walk::Target;

use super::block_state_data::get_tag;

/// `new BlockEntityBlockStateFix(schema, changesType)`: pistons store their moving
/// block as a block state instead of `blockId` / `blockData`.
pub fn fix() -> Fix {
    Fix::everywhere_with(
        "BlockEntityBlockStateFix",
        Target::block_entity("minecraft:piston"),
        |ctx, piston| {
            let block = get_i32_or(piston, "blockId", 0);
            remove(piston, "blockId");
            let data = get_i32_or(piston, "blockData", 0) & 15;
            remove(piston, "blockData");
            let mut block_state = get_tag((block << 4) | data);
            // The state is read with the output schema's `block_state` type.
            let _ = decode(ctx.output, &Tmpl::Ref(r::BLOCK_STATE), &mut block_state);
            set(piston, "blockState", block_state);
        },
    )
}
