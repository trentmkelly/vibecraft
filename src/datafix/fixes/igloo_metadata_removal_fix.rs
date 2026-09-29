//! Port of `net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix`.

use crate::datafix::dynamic::{get, get_mut, get_str_or, list_items, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new IglooMetadataRemovalFix(schema, changesType)`: igloo pieces are folded
/// into the structure id.
pub fn fix() -> Fix {
    Fix::everywhere(
        "IglooMetadataRemovalFix",
        Target::Type(r::STRUCTURE_FEATURE),
        fix_tag,
    )
}

fn is_igloo_piece(tag: &Tag) -> bool {
    get_str_or(tag, "id", "") == "Iglu"
}

fn fix_tag(input: &mut Tag) {
    let children = get(input, "Children").and_then(list_items);
    let is_igloo_only = children
        .as_ref()
        .is_some_and(|pieces| pieces.iter().all(is_igloo_piece));
    if is_igloo_only {
        set(input, "id", Tag::String("Igloo".to_string()));
        remove(input, "Children");
    } else if let (Some(pieces), Some(slot)) = (children, get_mut(input, "Children")) {
        *slot = Tag::List(
            pieces
                .into_iter()
                .filter(|piece| !is_igloo_piece(piece))
                .collect(),
        );
    }
}
