//! Port of `net.minecraft.util.datafix.fixes.OptionsForceVBOFix`.

use crate::datafix::dynamic::set;
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new OptionsForceVBOFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere("OptionsForceVBOFix", Target::Type(r::OPTIONS), |options| {
        set(options, "useVbo", Tag::String("true".to_string()))
    })
}
