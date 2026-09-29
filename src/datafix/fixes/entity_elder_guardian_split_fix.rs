//! Port of `net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix`.

use crate::datafix::dynamic::get_bool_or;
use crate::datafix::fix::Fix;

use super::simple_entity_rename_fix::simple_entity_rename_fix;

/// `new EntityElderGuardianSplitFix(schema, changesType)`.
pub fn fix() -> Fix {
    simple_entity_rename_fix("EntityElderGuardianSplitFix", |name, tag| {
        if name == "Guardian" && get_bool_or(tag, "Elder", false) {
            "ElderGuardian".to_string()
        } else {
            name.to_string()
        }
    })
}
