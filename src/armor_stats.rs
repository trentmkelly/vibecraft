//! Armor attribute contributions of worn armor items.
//!
//! Java references:
//! * `ArmorMaterials` — the per-material defense table (`makeDefense(boots, legs, chest,
//!   helmet, body)`), toughness and knockback resistance.
//! * `ArmorMaterial.createAttributes(ArmorType)` — each armor piece adds
//!   `Attributes.ARMOR` / `ARMOR_TOUGHNESS` modifiers (`ADD_VALUE`) while worn in its slot.
//! * `Attributes.ARMOR` (clamped `0..30`) and `Attributes.ARMOR_TOUGHNESS` (`0..20`).
//! * `Items.java` — which item uses which material (`humanoidArmor(material, type)`), which
//!   derives each piece's `MAX_DAMAGE` (`material durability * ArmorType durability`),
//!   enchantability and repair ingredient.

use std::sync::LazyLock;

use crate::item_properties::{EquipmentSlot, ItemDefinition};

/// Maximum of `Attributes.ARMOR`.
pub const MAX_ARMOR_VALUE: f64 = 30.0;
/// Maximum of `Attributes.ARMOR_TOUGHNESS`.
pub const MAX_ARMOR_TOUGHNESS: f64 = 20.0;

/// Worn-armor slot of a humanoid armor piece (`ArmorType`, without `BODY`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Boots,
    Leggings,
    Chestplate,
    Helmet,
}

/// One `ArmorMaterial`: defense per slot `[boots, leggings, chestplate, helmet]`.
struct Material {
    /// Item id prefix (`golden` for `minecraft:golden_helmet`).
    prefix: &'static str,
    durability: u32,
    defense: [u8; 4],
    enchantment_value: u32,
    toughness: f32,
    /// `ArmorMaterial.repairIngredient` item tag.
    repair_tag: &'static str,
}

const LEATHER: Material = Material {
    prefix: "leather", durability: 5, defense: [1, 2, 3, 1], enchantment_value: 15,
    toughness: 0.0, repair_tag: "#minecraft:repairs_leather_armor",
};
const COPPER: Material = Material {
    prefix: "copper", durability: 11, defense: [1, 3, 4, 2], enchantment_value: 8,
    toughness: 0.0, repair_tag: "#minecraft:repairs_copper_armor",
};
const CHAINMAIL: Material = Material {
    prefix: "chainmail", durability: 15, defense: [1, 4, 5, 2], enchantment_value: 12,
    toughness: 0.0, repair_tag: "#minecraft:repairs_chain_armor",
};
const IRON: Material = Material {
    prefix: "iron", durability: 15, defense: [2, 5, 6, 2], enchantment_value: 9,
    toughness: 0.0, repair_tag: "#minecraft:repairs_iron_armor",
};
const GOLD: Material = Material {
    prefix: "golden", durability: 7, defense: [1, 3, 5, 2], enchantment_value: 25,
    toughness: 0.0, repair_tag: "#minecraft:repairs_gold_armor",
};
const DIAMOND: Material = Material {
    prefix: "diamond", durability: 33, defense: [3, 6, 8, 3], enchantment_value: 10,
    toughness: 2.0, repair_tag: "#minecraft:repairs_diamond_armor",
};
const TURTLE_SCUTE: Material = Material {
    prefix: "turtle", durability: 25, defense: [2, 5, 6, 2], enchantment_value: 9,
    toughness: 0.0, repair_tag: "#minecraft:repairs_turtle_helmet",
};
const NETHERITE: Material = Material {
    prefix: "netherite", durability: 37, defense: [3, 6, 8, 3], enchantment_value: 15,
    toughness: 3.0, repair_tag: "#minecraft:repairs_netherite_armor",
};

/// `ArmorType` in `[boots, leggings, chestplate, helmet]` order: slot and durability factor.
const ARMOR_TYPES: [(ArmorSlot, u32, &str, EquipmentSlot); 4] = [
    (ArmorSlot::Boots, 13, "boots", EquipmentSlot::Feet),
    (ArmorSlot::Leggings, 15, "leggings", EquipmentSlot::Legs),
    (ArmorSlot::Chestplate, 16, "chestplate", EquipmentSlot::Chest),
    (ArmorSlot::Helmet, 11, "helmet", EquipmentSlot::Head),
];

/// The materials that have all four pieces (turtle scute only has a helmet).
const MATERIALS: [&Material; 8] =
    [&LEATHER, &COPPER, &CHAINMAIL, &IRON, &GOLD, &DIAMOND, &TURTLE_SCUTE, &NETHERITE];

/// One registered humanoid armor item.
struct ArmorItem {
    id: &'static str,
    slot: ArmorSlot,
    material: &'static Material,
    durability_factor: u32,
    equipment_slot: EquipmentSlot,
}

/// Every humanoid armor item of `Items.java`, ids interned once.
static ARMOR_ITEMS: LazyLock<Vec<ArmorItem>> = LazyLock::new(|| {
    let mut items = Vec::new();
    for material in MATERIALS {
        for (slot, durability_factor, name, equipment_slot) in ARMOR_TYPES {
            // Turtle scute armor only exists as a helmet.
            if std::ptr::eq(material, &TURTLE_SCUTE) && slot != ArmorSlot::Helmet {
                continue;
            }
            let id = format!("minecraft:{}_{name}", material.prefix);
            items.push(ArmorItem {
                id: Box::leak(id.into_boxed_str()),
                slot,
                material,
                durability_factor,
                equipment_slot,
            });
        }
    }
    items
});

/// Item definitions of every humanoid armor piece
/// (`Item.Properties.humanoidArmor(material, type)`): durability, repair tag, enchantability,
/// equippable slot with `damage_on_hurt`. `minecraft:diamond_helmet` is defined with the
/// representative items and skipped here.
pub fn armor_item_definitions() -> Vec<ItemDefinition> {
    ARMOR_ITEMS
        .iter()
        .filter(|item| item.id != "minecraft:diamond_helmet")
        .map(|item| {
            ItemDefinition::new(item.id)
                .durability(item.material.durability * item.durability_factor)
                .repairable(item.material.repair_tag)
                .enchantable(item.material.enchantment_value)
                .equippable(item.equipment_slot, true, true)
                .custom("minecraft:attribute_modifiers")
        })
        .collect()
}

/// The armor slot and material of `item_id`, or `None` when it is not humanoid armor.
fn armor_piece(item_id: &str) -> Option<(ArmorSlot, &'static Material)> {
    ARMOR_ITEMS
        .iter()
        .find(|item| item.id == item_id)
        .map(|item| (item.slot, item.material))
}

/// `(armor, toughness)` an armor `item_id` contributes when worn in `worn_in`; zero when the
/// item is not armor or sits in the wrong slot (the modifier's `EquipmentSlotGroup` fails).
pub fn worn_armor_contribution(item_id: &str, worn_in: ArmorSlot) -> (f64, f64) {
    match armor_piece(item_id) {
        Some((slot, material)) if slot == worn_in => {
            let index = match slot {
                ArmorSlot::Boots => 0,
                ArmorSlot::Leggings => 1,
                ArmorSlot::Chestplate => 2,
                ArmorSlot::Helmet => 3,
            };
            (f64::from(material.defense[index]), f64::from(material.toughness))
        }
        _ => (0.0, 0.0),
    }
}

/// `(getArmorValue(), ARMOR_TOUGHNESS)` for the four worn pieces, each clamped to its
/// attribute range like `AttributeInstance.getValue()`.
pub fn total_armor<'a>(worn: impl IntoIterator<Item = (ArmorSlot, &'a str)>) -> (f32, f32) {
    let (mut armor, mut toughness) = (0.0, 0.0);
    for (slot, item_id) in worn {
        let (piece_armor, piece_toughness) = worn_armor_contribution(item_id, slot);
        armor += piece_armor;
        toughness += piece_toughness;
    }
    (armor.clamp(0.0, MAX_ARMOR_VALUE) as f32, toughness.clamp(0.0, MAX_ARMOR_TOUGHNESS) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_diamond_and_netherite_sets_match_armor_materials() {
        let set = |material: &str| {
            [
                (ArmorSlot::Boots, format!("minecraft:{material}_boots")),
                (ArmorSlot::Leggings, format!("minecraft:{material}_leggings")),
                (ArmorSlot::Chestplate, format!("minecraft:{material}_chestplate")),
                (ArmorSlot::Helmet, format!("minecraft:{material}_helmet")),
            ]
        };
        let diamond = set("diamond");
        let (armor, toughness) =
            total_armor(diamond.iter().map(|(slot, id)| (*slot, id.as_str())));
        assert_eq!((armor, toughness), (20.0, 8.0));
        let netherite = set("netherite");
        let (armor, toughness) =
            total_armor(netherite.iter().map(|(slot, id)| (*slot, id.as_str())));
        assert_eq!((armor, toughness), (20.0, 12.0));
    }

    #[test]
    fn turtle_helmet_and_wrong_slot_pieces() {
        assert_eq!(worn_armor_contribution("minecraft:turtle_helmet", ArmorSlot::Helmet), (2.0, 0.0));
        assert_eq!(worn_armor_contribution("minecraft:iron_helmet", ArmorSlot::Chestplate), (0.0, 0.0));
        assert_eq!(worn_armor_contribution("minecraft:elytra", ArmorSlot::Chestplate), (0.0, 0.0));
        assert_eq!(worn_armor_contribution("minecraft:golden_chestplate", ArmorSlot::Chestplate), (5.0, 0.0));
    }

    #[test]
    fn armor_definitions_match_items_java_durability_and_enchantability() {
        let by_id = |id: &str| {
            crate::item_properties::item_definition(id).unwrap().effective()
        };
        // Items.java: durability = material durability * ArmorType factor.
        assert_eq!(by_id("minecraft:leather_helmet").durability, Some(55));
        assert_eq!(by_id("minecraft:iron_chestplate").durability, Some(240));
        assert_eq!(by_id("minecraft:golden_leggings").durability, Some(105));
        assert_eq!(by_id("minecraft:diamond_boots").durability, Some(429));
        assert_eq!(by_id("minecraft:netherite_helmet").durability, Some(407));
        assert_eq!(by_id("minecraft:turtle_helmet").durability, Some(275));
        assert_eq!(by_id("minecraft:copper_boots").durability, Some(143));
        assert_eq!(by_id("minecraft:golden_boots").enchantability, Some(25));
        assert!(crate::item_properties::item_definition("minecraft:turtle_chestplate").is_none());
        assert_eq!(armor_item_definitions().len(), 8 * 4 - 3 - 1);
    }
}
