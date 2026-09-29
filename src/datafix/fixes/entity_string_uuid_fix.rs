//! Port of `net.minecraft.util.datafix.fixes.EntityStringUuidFix`.

use crate::datafix::dynamic::{get_str, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::uuid::JavaUuid;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new EntityStringUuidFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere("EntityStringUuidFix", Target::Type(r::ENTITY), |entity| {
        // Java throws on an unparsable UUID (aborting the chunk upgrade); such
        // entities are left untouched here.
        let Some(uuid) = get_str(entity, "UUID").and_then(JavaUuid::from_string) else {
            return;
        };
        remove(entity, "UUID");
        set(entity, "UUIDMost", Tag::Long(uuid.most));
        set(entity, "UUIDLeast", Tag::Long(uuid.least));
    })
}
