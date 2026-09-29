//! Port of `net.minecraft.util.datafix.fixes.EntityCustomNameToComponentFix`.

use crate::datafix::dynamic::{get_str_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::legacy_component_data_fix_utils::create_text_component_json;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::write_and_read_fix::write_fix_and_read;

/// `new EntityCustomNameToComponentFix(schema)`: custom names become JSON text
/// components (empty names are dropped).
pub fn fix() -> Fix {
    Fix::everywhere_with(
        "EntityCustomNameToComponentFix",
        Target::Type(r::ENTITY),
        |ctx, entity| {
            let Some(Tag::String(custom_name)) = typed(entity, "CustomName") else {
                return;
            };
            let custom_name = custom_name.clone();
            if custom_name.is_empty() {
                write_fix_and_read(ctx, r::ENTITY, entity, |written| {
                    remove(written, "CustomName");
                });
                return;
            }
            let component = if get_str_or(entity, "id", "") == "minecraft:commandblock_minecart" {
                custom_name
            } else {
                create_text_component_json(&custom_name)
            };
            set(entity, "CustomName", Tag::String(component));
        },
    )
}
