//! Port of `net.minecraft.util.datafix.fixes.SignTextStrictJsonFix`.

use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::rewrite_from_lenient;
use crate::datafix::references as r;
use crate::datafix::typed::typed_mut;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

const LINE_FIELDS: [&str; 4] = ["Text1", "Text2", "Text3", "Text4"];

/// `new SignTextStrictJsonFix(schema)`: sign lines become strict JSON components.
pub fn fix() -> Fix {
    named_entity_fix("SignTextStrictJsonFix", r::BLOCK_ENTITY, "Sign", |sign| {
        for line_field in LINE_FIELDS {
            if let Some(Tag::String(line)) = typed_mut(sign, line_field) {
                *line = rewrite_from_lenient(line);
            }
        }
    })
}
