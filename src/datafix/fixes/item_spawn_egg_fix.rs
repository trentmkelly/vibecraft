//! Port of `net.minecraft.util.datafix.fixes.ItemSpawnEggFix`: spawn egg damage
//! values become an `EntityTag.id`.

use crate::datafix::decode::decode;
use crate::datafix::dynamic::{get_i16_or, get_str, set};
use crate::datafix::fix::{Fix, FixContext};
use crate::datafix::references as r;
use crate::datafix::template::Tmpl;
use crate::datafix::typed::{set_typed, typed, update_typed_compound};
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::item_spawn_egg_fix_entities::ID_TO_ENTITY;

/// `new ItemSpawnEggFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere_with("ItemSpawnEggFix", Target::Type(r::ITEM_STACK), fix_stack)
}

fn fix_stack(ctx: &FixContext<'_>, stack: &mut Tag) {
    if get_str(stack, "id") != Some("minecraft:spawn_egg") {
        return;
    }
    let damage = i32::from(get_i16_or(stack, "Damage", 0));
    let old_id = typed(stack, "tag")
        .and_then(|tag| typed(tag, "EntityTag"))
        .and_then(|entity| get_str(entity, "id"))
        .map(str::to_string);
    if let Some(entity_name) = ID_TO_ENTITY[(damage & 0xFF) as usize] {
        if old_id.as_deref() != Some(entity_name) {
            let mut entity_tag = match typed(stack, "tag").and_then(|tag| typed(tag, "EntityTag")) {
                Some(existing) => existing.clone(),
                None => Tag::Compound(Vec::new()),
            };
            set(&mut entity_tag, "id", Tag::String(entity_name.to_string()));
            // `ExtraDataFixUtils.readAndSet` reads the new value with the entity tree type.
            let mut attempt = entity_tag.clone();
            if decode(ctx.input, &Tmpl::Ref(r::ENTITY_TREE), &mut attempt).is_ok() {
                entity_tag = attempt;
            }
            update_typed_compound(stack, "tag", |tag| set_typed(tag, "EntityTag", entity_tag));
        }
    }
    if damage != 0 {
        set(stack, "Damage", Tag::Short(0));
    }
}
