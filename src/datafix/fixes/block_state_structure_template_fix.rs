//! Port of `net.minecraft.util.datafix.fixes.BlockStateStructureTemplateFix`.

use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;

use super::block_state_data::upgrade_block_state_tag;

/// `new BlockStateStructureTemplateFix(schema, changesType)`: structure template
/// palettes get flattened block states.
pub fn fix() -> Fix {
    Fix::everywhere(
        "BlockStateStructureTemplateFix",
        Target::Type(r::BLOCK_STATE),
        |state| *state = upgrade_block_state_tag(state),
    )
}
