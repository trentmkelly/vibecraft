//! Port of `net.minecraft.util.datafix.fixes.BlockNameFlatteningFix`.

use crate::datafix::dynamic::ensure_namespaced;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::block_state_data::{upgrade_block_id, upgrade_block_name};

/// `new BlockNameFlatteningFix(schema, changesType)`: numeric and legacy block
/// names become flattened block names.
pub fn fix() -> Fix {
    Fix::everywhere(
        "BlockNameFlatteningFix",
        Target::Type(r::BLOCK_NAME),
        |block| {
            let upgraded = match block {
                Tag::Int(id) => upgrade_block_id(*id),
                Tag::String(name) => upgrade_block_name(&ensure_namespaced(name)),
                _ => return,
            };
            *block = Tag::String(upgraded);
        },
    )
}
