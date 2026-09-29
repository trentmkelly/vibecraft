//! Port of `net.minecraft.util.datafix.fixes.BlockEntityKeepPacked`.

use crate::datafix::dynamic::{boolean, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;

use super::named_entity_fix::named_entity_fix;

/// `new BlockEntityKeepPacked(schema, changesType)`: the `DUMMY` block entity keeps
/// its chunk packed.
pub fn fix() -> Fix {
    named_entity_fix("BlockEntityKeepPacked", r::BLOCK_ENTITY, "DUMMY", |tag| {
        set(tag, "keepPacked", boolean(true))
    })
}
