//! Port of `net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix`.

use crate::datafix::decode::is_opaque;
use crate::datafix::dynamic::{get, get_mut, get_str_or, remove};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::add_new_choices::add_new_choices;

/// `removeBlockEntity`: drops `field_name` of `tag` when it holds a block entity
/// whose id is in `ids_to_drop`.
fn remove_block_entity(tag: &mut Tag, field_name: &str, ids_to_drop: &[&str]) {
    if is_opaque(tag, field_name) {
        return;
    }
    let Some(block_entity) = get(tag, field_name) else {
        return;
    };
    if ids_to_drop.contains(&get_str_or(block_entity, "id", "")) {
        remove(tag, field_name);
    }
}

/// `new RemoveBlockEntityTagFix(schema, blockEntityIdsToDrop)`: items, falling
/// blocks and structure blocks lose the block entity data of the given ids.
pub fn fix(ids_to_drop: &'static [&'static str]) -> Fix {
    Fix::sequence(
        "RemoveBlockEntityTagFix",
        vec![
            Fix::everywhere(
                "ItemRemoveBlockEntityTagFix",
                Target::Type(r::ITEM_STACK),
                move |stack| {
                    if is_opaque(stack, "tag") {
                        return;
                    }
                    if let Some(tag) = get_mut(stack, "tag") {
                        remove_block_entity(tag, "BlockEntityTag", ids_to_drop);
                    }
                },
            ),
            Fix::everywhere(
                "FallingBlockEntityRemoveBlockEntityTagFix",
                Target::entity("minecraft:falling_block"),
                move |falling_block| {
                    remove_block_entity(falling_block, "TileEntityData", ids_to_drop)
                },
            ),
            Fix::everywhere(
                "StructureRemoveBlockEntityTagFix",
                Target::Type(r::STRUCTURE),
                move |structure| {
                    if is_opaque(structure, "blocks") {
                        return;
                    }
                    if let Some(Tag::List(blocks)) = get_mut(structure, "blocks") {
                        for block in blocks {
                            remove_block_entity(block, "nbt", ids_to_drop);
                        }
                    }
                },
            ),
            // `convertUnchecked`: matches the block entity type without changing data.
            add_new_choices(
                "ItemRemoveBlockEntityTagFix - update block entity type",
                r::BLOCK_ENTITY,
            ),
        ],
    )
}
