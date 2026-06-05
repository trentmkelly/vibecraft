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

#[test]
fn anvil_repair_material_and_store_enchantment_helpers_match_java() {
    use super::{anvil_is_valid_repair_item, can_store_enchantments};
    use crate::item_properties::ItemComponent;

    // isValidRepairItem: a diamond tool (Repairable = #diamond_tool_materials) is
    // repaired by a diamond, not by an iron ingot.
    let mut pick = ItemStack::new("minecraft:diamond_pickaxe", 1);
    pick.set_component(ItemComponent::Repairable("#minecraft:diamond_tool_materials"));
    assert!(anvil_is_valid_repair_item(&pick, &ItemStack::new("minecraft:diamond", 1)));
    assert!(!anvil_is_valid_repair_item(&pick, &ItemStack::new("minecraft:iron_ingot", 1)));

    // Wooden tools (Repairable = #wooden_tool_materials -> #planks) accept any plank.
    let mut wpick = ItemStack::new("minecraft:wooden_pickaxe", 1);
    wpick.set_component(ItemComponent::Repairable("#minecraft:wooden_tool_materials"));
    assert!(anvil_is_valid_repair_item(&wpick, &ItemStack::new("minecraft:birch_planks", 1)));
    assert!(!anvil_is_valid_repair_item(&wpick, &ItemStack::new("minecraft:stick", 1)));

    // A literal (non-tag) repair item is matched directly.
    let mut mace = ItemStack::new("minecraft:mace", 1);
    mace.set_component(ItemComponent::Repairable("minecraft:breeze_rod"));
    assert!(anvil_is_valid_repair_item(&mace, &ItemStack::new("minecraft:breeze_rod", 1)));

    // An item with no Repairable component cannot be repaired by material.
    assert!(!anvil_is_valid_repair_item(
        &ItemStack::new("minecraft:apple", 1),
        &ItemStack::new("minecraft:diamond", 1)
    ));

    // canStoreEnchantments: every non-empty item has the default (empty) ENCHANTMENTS
    // component (COMMON_ITEM_COMPONENTS), so all can store — even an apple (you can
    // rename anything in an anvil) — while an empty stack cannot.
    assert!(can_store_enchantments(&ItemStack::new("minecraft:diamond_sword", 1)));
    assert!(can_store_enchantments(&ItemStack::new("minecraft:enchanted_book", 1)));
    assert!(can_store_enchantments(&ItemStack::new("minecraft:apple", 1)));
    assert!(!can_store_enchantments(&ItemStack::empty()));
}

#[test]
fn anvil_create_result_full_pipeline_matches_java() {
    use crate::item_properties::ItemComponent;

    fn damaged_sword(max: u32, dmg: u32, repair_cost: i32) -> ItemStack {
        let mut s = ItemStack::new("minecraft:diamond_sword", 1);
        s.set_component(ItemComponent::MaxDamage(max));
        s.set_damage_value(dmg);
        if repair_cost != 0 {
            s.set_component(ItemComponent::RepairCost(repair_cost));
        }
        s
    }

    // --- Repair by combining two of the same damaged item (+12% bonus). ---
    let mut menu = AnvilMenu::new();
    let mut player = PlayerInventory::new();
    menu.set_slot(0, damaged_sword(1561, 1000, 0), &mut player);
    menu.set_slot(1, damaged_sword(1561, 800, 0), &mut player);
    menu.set_result_from_inputs();
    let r = menu.get_slot(2, &player).unwrap();
    // remaining1=561, remaining2=761, bonus=561+187=948, remaining=1509, resultDmg=52.
    assert_eq!(r.item_id(), "minecraft:diamond_sword");
    assert_eq!(r.damage_value(), 52);
    assert_eq!(menu.cost, 2); // price += 2 for a durability combine
    assert_eq!(r.component("minecraft:repair_cost"), Some(&ItemComponent::RepairCost(1)));

    // --- Repair with a material item (a diamond). ---
    let mut menu = AnvilMenu::new();
    let mut pick = ItemStack::new("minecraft:diamond_pickaxe", 1);
    pick.set_component(ItemComponent::MaxDamage(1561));
    pick.set_damage_value(1000);
    pick.set_component(ItemComponent::Repairable("#minecraft:diamond_tool_materials"));
    menu.set_slot(0, pick, &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:diamond", 2), &mut player);
    menu.set_result_from_inputs();
    let r = menu.get_slot(2, &player).unwrap();
    // Each diamond repairs maxDamage/4 = 390; two diamonds -> 1000-390-390 = 220.
    assert_eq!(r.damage_value(), 220);
    assert_eq!(menu.repair_item_count_cost, 2);
    assert_eq!(menu.cost, 2);

    // --- Enchant-combine: sword + an enchanted book (sharpness II). ---
    let mut menu = AnvilMenu::new();
    let book = {
        let mut b = ItemStack::new("minecraft:enchanted_book", 1);
        b.set_component(ItemComponent::StoredEnchantments(enchants(&[(
            "minecraft:sharpness",
            2,
        )])));
        b
    };
    menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    menu.set_slot(1, book, &mut player);
    menu.set_result_from_inputs();
    let r = menu.get_slot(2, &player).unwrap();
    match r.component("minecraft:enchantments") {
        Some(ItemComponent::Enchantments(m)) => assert_eq!(m.get("minecraft:sharpness"), Some(&2)),
        other => panic!("expected sharpness on result, got {other:?}"),
    }
    assert_eq!(menu.cost, 2); // book fee max(1, anvil_cost/2)=1, * level 2 = 2

    // A book whose enchantment can't apply to the item (sharpness on a pickaxe) ->
    // incompatible-only -> empty result.
    let mut menu = AnvilMenu::new();
    let book = {
        let mut b = ItemStack::new("minecraft:enchanted_book", 1);
        b.set_component(ItemComponent::StoredEnchantments(enchants(&[(
            "minecraft:sharpness",
            1,
        )])));
        b
    };
    menu.set_slot(0, ItemStack::new("minecraft:diamond_pickaxe", 1), &mut player);
    menu.set_slot(1, book, &mut player);
    menu.set_result_from_inputs();
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // --- "Too expensive" (cost >= 40) is suppressed in survival but allowed in creative. ---
    let mut menu = AnvilMenu::new();
    menu.set_slot(0, damaged_sword(1561, 1000, 31), &mut player);
    menu.set_slot(1, damaged_sword(1561, 1000, 31), &mut player);
    menu.set_result_from_inputs(); // survival
    assert!(menu.get_slot(2, &player).unwrap().is_empty());
    assert_eq!(menu.cost, 64); // tax 62 + price 2
    menu.set_result_from_inputs_with_mode(true); // creative
    assert!(!menu.get_slot(2, &player).unwrap().is_empty());

    // --- Rename only -> cost 1, only_renaming. ---
    let mut menu = AnvilMenu::new();
    menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    menu.set_item_name(Some("Excalibur".to_string()));
    menu.set_result_from_inputs();
    assert_eq!(menu.cost, 1);
    assert!(menu.only_renaming);
    assert!(!menu.get_slot(2, &player).unwrap().is_empty());
}
