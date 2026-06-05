use super::*;
use crate::enchantment_system::min_cost_for;
use crate::item_properties::ItemComponent;
use std::collections::BTreeMap;

fn enchants(pairs: &[(&str, i32)]) -> BTreeMap<String, i32> {
    pairs.iter().map(|(id, l)| ((*id).to_string(), *l)).collect()
}

#[test]
fn grindstone_disenchant_keeps_curses_and_awards_xp() {
    let mut player = PlayerInventory::new();
    let mut sword = ItemStack::new("minecraft:diamond_sword", 1);
    sword.set_component(ItemComponent::Enchantments(enchants(&[
        ("minecraft:sharpness", 3),
        ("minecraft:binding_curse", 1),
    ])));

    let mut menu = GrindstoneMenu::new();
    menu.set_slot(0, sword, &mut player);
    let result = menu.get_slot(2, &player).unwrap();

    // Non-curse (sharpness) stripped, curse kept.
    match result.component("minecraft:enchantments") {
        Some(ItemComponent::Enchantments(m)) => {
            assert_eq!(m.len(), 1);
            assert_eq!(m.get("minecraft:binding_curse"), Some(&1));
        }
        other => panic!("expected enchantments component, got {other:?}"),
    }
    // One remaining enchantment -> prior-work repair cost 1.
    assert_eq!(
        result.component("minecraft:repair_cost"),
        Some(&ItemComponent::RepairCost(1))
    );
    // XP = ceil(min_cost(sharpness,3)/2) (curse contributes nothing); +0 random.
    let cost = min_cost_for("minecraft:sharpness", 3);
    assert!(cost > 0);
    assert_eq!(menu.experience_on_take(0), (cost + 1) / 2);
    assert_eq!(menu.experience_on_take(2), (cost + 1) / 2 + 2);
}

#[test]
fn grindstone_empty_enchanted_book_transmutes_to_book() {
    let mut player = PlayerInventory::new();
    let mut book = ItemStack::new("minecraft:enchanted_book", 1);
    book.set_component(ItemComponent::StoredEnchantments(enchants(&[(
        "minecraft:sharpness",
        2,
    )])));

    let mut menu = GrindstoneMenu::new();
    menu.set_slot(1, book, &mut player);
    let result = menu.get_slot(2, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:book");
    assert!(result.component("minecraft:stored_enchantments").is_none());
}

#[test]
fn grindstone_repairs_two_matching_damaged_items() {
    let mut player = PlayerInventory::new();
    let mut a = ItemStack::new("minecraft:diamond_pickaxe", 1);
    a.set_component(ItemComponent::MaxDamage(1561));
    a.set_damage_value(500);
    let mut b = ItemStack::new("minecraft:diamond_pickaxe", 1);
    b.set_component(ItemComponent::MaxDamage(1561));
    b.set_damage_value(300);

    let mut menu = GrindstoneMenu::new();
    menu.set_slot(0, a, &mut player);
    menu.set_slot(1, b, &mut player);
    let result = menu.get_slot(2, &player).unwrap();
    assert_eq!(result.item_id(), "minecraft:diamond_pickaxe");
    assert_eq!(result.count(), 1);
    assert_eq!(result.max_damage(), 1561);
    // remaining = (1561-500)+(1561-300)+1561*5/100 = 2400 > 1561 -> fully repaired.
    assert_eq!(result.damage_value(), 0);
}

#[test]
fn grindstone_no_result_for_invalid_inputs() {
    let mut player = PlayerInventory::new();

    // Unenchanted single item -> no result.
    let mut menu = GrindstoneMenu::new();
    menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Two different items -> merge rejects -> no result.
    let mut menu = GrindstoneMenu::new();
    menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:iron_sword", 1), &mut player);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // A stack of >1 -> no result.
    let mut menu = GrindstoneMenu::new();
    let mut book = ItemStack::new("minecraft:enchanted_book", 2);
    book.set_component(ItemComponent::StoredEnchantments(enchants(&[(
        "minecraft:sharpness",
        1,
    )])));
    menu.set_slot(0, book, &mut player);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());
}
