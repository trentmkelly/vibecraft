//! Port of `net.minecraft.util.datafix.fixes.OptionsLowerCaseLanguageFix`.

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new OptionsLowerCaseLanguageFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere(
        "OptionsLowerCaseLanguageFix",
        Target::Type(r::OPTIONS),
        |options| {
            if let Some(lang) = get_str(options, "lang").map(str::to_lowercase) {
                set(options, "lang", Tag::String(lang));
            }
        },
    )
}
