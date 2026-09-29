//! Port of `net.minecraft.util.datafix.fixes.EntitySkeletonSplitFix`.

use crate::datafix::dynamic::get_i32_or;
use crate::datafix::fix::Fix;

use super::simple_entity_rename_fix::simple_entity_rename_fix;

/// `new EntitySkeletonSplitFix(schema, changesType)`.
pub fn fix() -> Fix {
    simple_entity_rename_fix("EntitySkeletonSplitFix", |name, tag| {
        if name == "Skeleton" {
            match get_i32_or(tag, "SkeletonType", 0) {
                1 => return "WitherSkeleton".to_string(),
                2 => return "Stray".to_string(),
                _ => {}
            }
        }
        name.to_string()
    })
}
