use super::*;

#[test]
fn crafting_menu_layout_and_recipe_matching_match_vanilla() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    assert_eq!(CraftingMenu::SLOT_COUNT, 46);
    assert!(!menu.may_place(CraftingMenu::RESULT_SLOT));
    assert!(menu.may_place(1));
    assert!(menu.may_place(10));

    assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player));
    assert_eq!(menu.result().item_id(), "minecraft:oak_planks");
    assert_eq!(menu.result().count(), 4);

    let taken = menu.take_result();
    assert_eq!(taken.count(), 4);
    assert!(menu.get_slot(1, &player).unwrap().is_empty());
}

#[test]
fn crafting_menu_quick_move_result_goes_to_player_inventory() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player);
    let moved = menu.quick_move(0, &mut player);
    assert_eq!(moved.item_id(), "minecraft:oak_planks");
    assert_eq!(moved.count(), 4);
    // Java `moveItemStackTo(stack, 10, 46, true)` iterates 45..10 reversed,
    // so the last menu slot (45 = hotbar 8 = player main slot 8) gets the
    // result first.
    assert_eq!(player.get(8).item_id(), "minecraft:oak_planks");
    assert_eq!(player.get(8).count(), 4);
}

#[test]
fn crafting_menu_result_take_applies_remainders_and_unlocks_recipe_once() {
    let mut menu = CraftingMenu::new(tests::crafting_table_recipes());
    let mut player = PlayerInventory::new();
    menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1), &mut player);

    let taken = menu.take_result();
    assert_eq!(taken.item_id(), "minecraft:packed_ice");
    assert_eq!(
        menu.get_slot(1, &player).unwrap().item_id(),
        "minecraft:bucket"
    );
    assert_eq!(
        menu.recipe_book_known_recipes(),
        vec!["minecraft:packed_ice_from_water_bucket"]
    );
    assert_eq!(
        menu.recipe_book_highlighted_recipes(),
        vec!["minecraft:packed_ice_from_water_bucket"]
    );
    assert_eq!(
        menu.drain_recipe_unlock_events(),
        vec!["minecraft:packed_ice_from_water_bucket"]
    );

    menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1), &mut player);
    assert_eq!(menu.take_result().item_id(), "minecraft:packed_ice");
    assert!(menu.drain_recipe_unlock_events().is_empty());
}

#[test]
fn crafting_menu_recipe_book_placement_increments_matching_grid_like_java() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    player.set(0, ItemStack::new("minecraft:oak_log", 2));
    assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player));

    assert!(menu.place_recipe_from_inventory("minecraft:oak_planks", false, &mut player));

    assert_eq!(
        menu.get_slot(1, &player),
        Some(ItemStack::new("minecraft:oak_log", 2))
    );
    assert_eq!(player.get(0).item_id(), "minecraft:oak_log");
    assert_eq!(player.get(0).count(), 1);
}

#[test]
fn crafting_menu_recipe_book_refuses_matching_grid_that_cannot_grow_like_java() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    player.set(0, ItemStack::new("minecraft:oak_log", 64));
    assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 64), &mut player));

    assert!(!menu.place_recipe_from_inventory("minecraft:oak_planks", false, &mut player));

    assert_eq!(
        menu.get_slot(1, &player),
        Some(ItemStack::new("minecraft:oak_log", 64))
    );
    assert_eq!(player.get(0), &ItemStack::new("minecraft:oak_log", 64));
}

#[test]
fn crafting_menu_recipe_book_refuses_to_clear_grid_when_inventory_is_full_like_java() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    for slot in 0..INVENTORY_SIZE {
        player.set(slot, ItemStack::new("minecraft:cobblestone", 64));
    }
    assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player));

    assert!(!menu.place_recipe_from_inventory("minecraft:oak_planks", false, &mut player));

    assert_eq!(
        menu.get_slot(1, &player),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(player.get(0), &ItemStack::new("minecraft:cobblestone", 64));
    assert!(player.dropped().is_empty());
}

#[test]
fn crafting_menu_failed_result_quick_move_preserves_inputs_and_result() {
    let mut menu = CraftingMenu::new(tests::planks_recipe());
    let mut player = PlayerInventory::new();
    for slot in 0..INVENTORY_SIZE {
        player.set(slot, ItemStack::new("minecraft:cobblestone", 64));
    }
    menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1), &mut player);

    let moved = menu.quick_move(0, &mut player);
    assert!(moved.is_empty());
    assert_eq!(menu.result().item_id(), "minecraft:oak_planks");
    assert_eq!(
        menu.get_slot(1, &player).unwrap().item_id(),
        "minecraft:oak_log"
    );
    assert!(menu.recipe_unlock_events().is_empty());
}
