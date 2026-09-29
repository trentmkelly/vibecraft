//! Port of `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix`:
//! renamed structure piece templates.

use crate::datafix::dynamic::{get, get_str_or, list_items, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::chunk_structures_template_rename_fix_renames::RENAMES;

/// `new ChunkStructuresTemplateRenameFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ChunkStructuresTemplateRenameFix",
        Target::Type(r::STRUCTURE_FEATURE),
        |structure| {
            if get(structure, "Children").is_none() {
                return;
            }
            // `children.asStream()` is empty for anything but a list.
            let children: Vec<Tag> = get(structure, "Children")
                .and_then(list_items)
                .unwrap_or_default();
            let id = get_str_or(structure, "id", "").to_string();
            let fixed = children
                .into_iter()
                .map(|child| fix_child(&id, child))
                .collect();
            set(structure, "Children", Tag::List(fixed));
        },
    )
}

fn fix_child(structure_id: &str, mut child: Tag) -> Tag {
    let Some((_, piece_id, templates)) = RENAMES.iter().find(|(id, _, _)| *id == structure_id)
    else {
        return child;
    };
    if get_str_or(&child, "id", "") == *piece_id {
        let template = get_str_or(&child, "Template", "").to_string();
        let renamed = templates
            .iter()
            .find(|(old, _)| *old == template)
            .map_or(template.as_str(), |(_, new)| *new)
            .to_string();
        set(&mut child, "Template", Tag::String(renamed));
    }
    child
}
