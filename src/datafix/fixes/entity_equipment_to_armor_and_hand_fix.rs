//! Port of `net.minecraft.util.datafix.fixes.EntityEquipmentToArmorAndHandFix`.

use crate::datafix::dynamic::{as_f32, get, list_items, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new EntityEquipmentToArmorAndHandFix(schema)` (registered with `changesType`).
pub fn fix() -> Fix {
    Fix::sequence(
        "EntityEquipmentToArmorAndHandFix",
        vec![
            Fix::everywhere(
                "EntityEquipmentToArmorAndHandFix - drop chances",
                Target::Type(r::ENTITY),
                fix_drop_chances,
            ),
            Fix::everywhere(
                "EntityEquipmentToArmorAndHandFix - equipment",
                Target::Type(r::ENTITY_EQUIPMENT),
                fix_equipment,
            ),
        ],
    )
}

/// `EntityEquipmentToArmorAndHandFix.fixDropChances`.
fn fix_drop_chances(tag: &mut Tag) {
    let drop_chances = remove(tag, "DropChances");
    let Some(values) = drop_chances.as_ref().and_then(list_items) else {
        return;
    };
    let mut chances = values
        .iter()
        .map(|value| as_f32(value).unwrap_or(0.0))
        .chain(std::iter::repeat(0.0));
    let hand_chance = chances.next().unwrap_or(0.0);
    if get(tag, "HandDropChances").is_none() {
        set(
            tag,
            "HandDropChances",
            Tag::List(vec![Tag::Float(hand_chance), Tag::Float(0.0)]),
        );
    }
    if get(tag, "ArmorDropChances").is_none() {
        let armor = (0..4)
            .map(|_| Tag::Float(chances.next().unwrap_or(0.0)))
            .collect();
        set(tag, "ArmorDropChances", Tag::List(armor));
    }
}

/// The `entity_equipment` rewrite: the legacy five slot `Equipment` list becomes
/// `HandItems` / `ArmorItems`.
fn fix_equipment(tag: &mut Tag) {
    // An unreadable `Equipment` field is absent from the typed value.
    let items = match typed(tag, "Equipment") {
        Some(Tag::List(items)) => items.clone(),
        _ => Vec::new(),
    };
    remove(tag, "Equipment");
    // Empty stack: the new item stack type read from an empty map.
    let empty_stack = || Tag::Compound(Vec::new());
    if !items.is_empty() {
        set(
            tag,
            "HandItems",
            Tag::List(vec![items[0].clone(), empty_stack()]),
        );
    }
    if items.len() > 1 {
        let mut armor = vec![empty_stack(), empty_stack(), empty_stack(), empty_stack()];
        let count = items.len().min(5) - 1;
        armor[..count].clone_from_slice(&items[1..=count]);
        set(tag, "ArmorItems", Tag::List(armor));
    }
}
