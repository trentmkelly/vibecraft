//! Port of `net.minecraft.util.datafix.fixes.ScoreboardDisplayNameFix`.

use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::create_text_component_json;
use crate::datafix::references::TypeReference;
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new ScoreboardDisplayNameFix(schema, name, type)`: display names become JSON
/// text components.
pub fn scoreboard_display_name_fix(name: &str, reference: TypeReference) -> Fix {
    Fix::everywhere(name, Target::Type(reference), |node| {
        if let Some(Tag::String(display_name)) = typed_mut(node, "DisplayName") {
            *display_name = create_text_component_json(display_name);
        }
    })
}
