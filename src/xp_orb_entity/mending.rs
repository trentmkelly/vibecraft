//! Equipment repair performed when a player absorbs an experience orb.
//!
//! This follows `ExperienceOrb.repairPlayerItems`: Mending selects one eligible
//! equipment entry, repairs it, and recursively spends the integer XP remainder
//! on another eligible entry. Candidate entries are weighted by qualifying
//! enchantments, matching `EnchantmentHelper.getRandomItemWith`.
//!
//! The current enchantment registry exposes vanilla effect hooks and slots. Only
//! Mending has the repair hook in that registry; its value is the Java bootstrap
//! `MultiplyValue(LevelBasedValue.constant(2.0F))` (Enchantments.java). Loading
//! custom repair effect values and conditions requires an effect registry.

use crate::enchantment_system::{enchantment, EnchantmentEffectHook, EquipmentSlotGroup};
use crate::item_properties::ItemComponent;
use crate::item_stack::ItemStack;

/// Equipment slots in `EquipmentSlot.VALUES` order from Java 26.1.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairEquipmentSlot {
    Mainhand,
    Offhand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl RepairEquipmentSlot {
    const fn java_order(self) -> usize {
        match self {
            Self::Mainhand => 0,
            Self::Offhand => 1,
            Self::Feet => 2,
            Self::Legs => 3,
            Self::Chest => 4,
            Self::Head => 5,
            Self::Body => 6,
            Self::Saddle => 7,
        }
    }
}

/// Spend `amount` XP repairing damaged equipped items with XP-repairing
/// enchantments. `choose_index` selects a candidate in `[0, candidate_count)`;
/// it is called even when there is only one candidate, as Java consumes a
/// random draw in that case too.
///
/// The returned amount is the XP left for the player's XP bar. Candidate
/// ordering follows Java's equipment-slot order regardless of the slice
/// order. Each qualifying enchantment adds one candidate entry, so multiple
/// effects on one item increase its selection weight as they do in Java.
pub fn repair_equipment(
    equipment: &mut [(RepairEquipmentSlot, ItemStack)],
    amount: i32,
    choose_index: &mut impl FnMut(usize) -> usize,
) -> i32 {
    repair_equipment_inner(equipment, amount, choose_index)
}

fn repair_equipment_inner(
    equipment: &mut [(RepairEquipmentSlot, ItemStack)],
    amount: i32,
    choose_index: &mut impl FnMut(usize) -> usize,
) -> i32 {
    let mut candidates = Vec::new();
    let mut equipment_order: Vec<usize> = (0..equipment.len()).collect();
    equipment_order.sort_by_key(|&index| equipment[index].0.java_order());
    for equipment_index in equipment_order {
        let (slot, item) = &equipment[equipment_index];
        if !item.is_damaged() {
            continue;
        }
        let Some(ItemComponent::Enchantments(enchantments)) =
            item.component("minecraft:enchantments")
        else {
            continue;
        };

        for id in enchantments.keys() {
            let Some(definition) = enchantment(id) else {
                continue;
            };
            if definition
                .hooks
                .contains(&EnchantmentEffectHook::RepairWithXp)
                && slot_matches(definition.slots, *slot)
            {
                candidates.push(equipment_index);
            }
        }
    }

    if candidates.is_empty() {
        return amount;
    }

    let selected = candidates[choose_index(candidates.len())];
    let item = &mut equipment[selected].1;

    // The currently modeled repair effect is Mending's factor of two. Java
    // starts with the XP amount as a float and applies registry effects before
    // truncating to int and clamping at zero.
    let to_repair = ((amount as f32) * 2.0) as i32;
    let to_repair = to_repair.max(0);
    let damage = item.damage_value() as i32;
    let repaired = to_repair.min(damage);
    item.set_damage_value((damage - repaired) as u32);

    if repaired > 0 {
        // Preserve Java int arithmetic and truncation order exactly.
        let remaining = amount.wrapping_sub(repaired.wrapping_mul(amount) / to_repair);
        if remaining > 0 {
            return repair_equipment_inner(equipment, remaining, choose_index);
        }
    }
    0
}

fn slot_matches(groups: &[EquipmentSlotGroup], slot: RepairEquipmentSlot) -> bool {
    groups.iter().any(|group| match group {
        EquipmentSlotGroup::Any => true,
        EquipmentSlotGroup::Armor => matches!(
            slot,
            RepairEquipmentSlot::Feet
                | RepairEquipmentSlot::Legs
                | RepairEquipmentSlot::Chest
                | RepairEquipmentSlot::Head
                | RepairEquipmentSlot::Body
        ),
        EquipmentSlotGroup::Head => slot == RepairEquipmentSlot::Head,
        EquipmentSlotGroup::Chest => slot == RepairEquipmentSlot::Chest,
        EquipmentSlotGroup::Legs => slot == RepairEquipmentSlot::Legs,
        EquipmentSlotGroup::Feet => slot == RepairEquipmentSlot::Feet,
        EquipmentSlotGroup::Hand => matches!(
            slot,
            RepairEquipmentSlot::Mainhand | RepairEquipmentSlot::Offhand
        ),
        EquipmentSlotGroup::MainHand => slot == RepairEquipmentSlot::Mainhand,
        EquipmentSlotGroup::OffHand => slot == RepairEquipmentSlot::Offhand,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn mending_item(damage: u32) -> ItemStack {
        let mut item = ItemStack::new("minecraft:diamond_sword", 1);
        item.set_damage_value(damage);
        item.set_component(ItemComponent::Enchantments(BTreeMap::from([(
            "minecraft:mending".to_owned(),
            1,
        )])));
        item
    }

    #[test]
    fn mending_spends_only_the_xp_needed_to_repair_selected_item() {
        let mut equipment = [(RepairEquipmentSlot::Mainhand, mending_item(5))];
        let mut draws = Vec::new();

        let remaining = repair_equipment(&mut equipment, 5, &mut |size| {
            draws.push(size);
            0
        });

        assert_eq!(remaining, 3);
        assert_eq!(equipment[0].1.damage_value(), 0);
        assert_eq!(draws, [1]);
    }

    #[test]
    fn remaining_xp_can_repair_another_eligible_equipment_item() {
        let mut equipment = [
            (RepairEquipmentSlot::Mainhand, mending_item(4)),
            (RepairEquipmentSlot::Head, mending_item(8)),
        ];
        let mut picks = [0, 0].into_iter();
        let mut draws = Vec::new();

        let remaining = repair_equipment(&mut equipment, 6, &mut |size| {
            draws.push(size);
            picks.next().unwrap()
        });

        // First item spends two XP repairing four durability. Four XP remain,
        // enough for the second item's eight durability.
        assert_eq!(remaining, 0);
        assert_eq!(equipment[0].1.damage_value(), 0);
        assert_eq!(equipment[1].1.damage_value(), 0);
        assert_eq!(draws, [2, 1]);
    }

    #[test]
    fn undamaged_or_non_mending_items_leave_all_xp_unspent() {
        let undamaged = mending_item(0);
        let mut non_mending = ItemStack::new("minecraft:diamond_sword", 1);
        non_mending.set_damage_value(10);
        let mut equipment = [
            (RepairEquipmentSlot::Mainhand, undamaged.clone()),
            (RepairEquipmentSlot::Head, non_mending.clone()),
        ];
        let mut called = false;

        assert_eq!(
            repair_equipment(&mut equipment, 7, &mut |_| {
                called = true;
                0
            }),
            7
        );
        assert!(!called);
        assert_eq!(equipment[0].1, undamaged);
        assert_eq!(equipment[1].1, non_mending);
    }

    #[test]
    fn stored_enchantments_and_unbreakable_items_are_ineligible() {
        let mut book_enchantment = ItemStack::new("minecraft:diamond_sword", 1);
        book_enchantment.set_damage_value(10);
        book_enchantment.set_component(ItemComponent::StoredEnchantments(BTreeMap::from([(
            "minecraft:mending".to_owned(),
            1,
        )])));
        let mut unbreakable = mending_item(10);
        unbreakable.set_component(ItemComponent::Custom("minecraft:unbreakable"));
        let mut equipment = [
            (RepairEquipmentSlot::Mainhand, book_enchantment),
            (RepairEquipmentSlot::Offhand, unbreakable),
        ];
        assert_eq!(
            repair_equipment(&mut equipment, 3, &mut |_| panic!("no eligible equipment")),
            3
        );
    }

    #[test]
    fn armor_slot_group_includes_animal_body_armor_but_excludes_saddle() {
        // EquipmentSlotGroup.ARMOR delegates to EquipmentSlot.isArmor(), which
        // includes both HUMANOID_ARMOR and ANIMAL_ARMOR in Java 26.1.2.
        assert!(slot_matches(
            &[EquipmentSlotGroup::Armor],
            RepairEquipmentSlot::Body
        ));
        assert!(slot_matches(
            &[EquipmentSlotGroup::Armor],
            RepairEquipmentSlot::Head
        ));
        assert!(!slot_matches(
            &[EquipmentSlotGroup::Armor],
            RepairEquipmentSlot::Saddle
        ));
        assert!(!slot_matches(
            &[EquipmentSlotGroup::Armor],
            RepairEquipmentSlot::Mainhand
        ));
    }

    #[test]
    fn candidate_selection_follows_java_equipment_order() {
        let mut equipment = [
            (RepairEquipmentSlot::Head, mending_item(2)),
            (RepairEquipmentSlot::Mainhand, mending_item(2)),
        ];
        let mut sizes = Vec::new();

        repair_equipment(&mut equipment, 1, &mut |size| {
            sizes.push(size);
            1
        });

        assert_eq!(sizes, [2]);
        assert_eq!(equipment[0].1.damage_value(), 0);
        assert_eq!(equipment[1].1.damage_value(), 2);
    }

    #[test]
    fn large_xp_amount_uses_java_float_conversion_instead_of_integer_overflow() {
        let mut item = mending_item(0);
        item.set_component(ItemComponent::MaxDamage(2_000_000_000));
        item.set_damage_value(2_000_000_000);
        let mut equipment = [(RepairEquipmentSlot::Mainhand, item)];

        let remaining = repair_equipment(&mut equipment, i32::MAX, &mut |_| 0);

        assert_eq!(equipment[0].1.damage_value(), 0);
        assert!(remaining > 0);
    }
}
