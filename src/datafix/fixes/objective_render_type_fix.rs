//! Port of `net.minecraft.util.datafix.fixes.ObjectiveRenderTypeFix`.

use crate::datafix::dynamic::{get_str, get_str_or, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `ObjectiveRenderTypeFix.getRenderType`.
fn render_type(criteria_name: &str) -> &'static str {
    if criteria_name == "health" {
        "hearts"
    } else {
        "integer"
    }
}

/// `new ObjectiveRenderTypeFix(schema)`: objectives get a default render type.
pub fn fix() -> Fix {
    Fix::everywhere(
        "ObjectiveRenderTypeFix",
        Target::Type(r::OBJECTIVE),
        |objective| {
            if get_str(objective, "RenderType").is_none() {
                let default = render_type(get_str_or(objective, "CriteriaName", ""));
                set(objective, "RenderType", Tag::String(default.to_string()));
            }
        },
    )
}
