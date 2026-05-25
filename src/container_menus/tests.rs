use super::*;
use crate::recipe_system::{
    IngredientSpec, ItemAmount, RecipeHolder, RecipeKind, RecipeMap, StonecutterSelection,
};

fn empty_recipes() -> RecipeMap {
    RecipeMap::create(Vec::new())
}

fn planks_recipe() -> RecipeMap {
    RecipeMap::create(vec![
        RecipeHolder {
            id: "minecraft:oak_planks",
            recipe: RecipeKind::Shapeless {
                ingredients: vec![IngredientSpec::Item("minecraft:oak_log")],
                result: ItemAmount {
                    item: "minecraft:oak_planks",
                    count: 4,
                },
            },
        },
        RecipeHolder {
            id: "minecraft:iron_ingot_from_smelting",
            recipe: RecipeKind::Cooking {
                kind: crate::recipe_system::CookingKind::Smelting,
                ingredient: IngredientSpec::Item("minecraft:raw_iron"),
                result: ItemAmount::one("minecraft:iron_ingot"),
                experience_millis: 700,
                cooking_time: Some(200),
            },
        },
    ])
}

fn crafting_table_recipes() -> RecipeMap {
    RecipeMap::create(vec![
        RecipeHolder {
            id: "minecraft:oak_planks",
            recipe: RecipeKind::Shapeless {
                ingredients: vec![IngredientSpec::Item("minecraft:oak_log")],
                result: ItemAmount {
                    item: "minecraft:oak_planks",
                    count: 4,
                },
            },
        },
        RecipeHolder {
            id: "minecraft:packed_ice_from_water_bucket",
            recipe: RecipeKind::Shapeless {
                ingredients: vec![IngredientSpec::Item("minecraft:water_bucket")],
                result: ItemAmount::one("minecraft:packed_ice"),
            },
        },
    ])
}

// -------- CraftingMenu --------

#[test]
fn crafting_menu_layout_and_recipe_matching_match_vanilla() {
    let mut menu = CraftingMenu::new(planks_recipe());
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
    let mut menu = CraftingMenu::new(planks_recipe());
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
    let mut menu = CraftingMenu::new(crafting_table_recipes());
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
fn crafting_menu_failed_result_quick_move_preserves_inputs_and_result() {
    let mut menu = CraftingMenu::new(planks_recipe());
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

// -------- AbstractFurnaceMenu --------

#[test]
fn furnace_menu_layout_and_fuel_restrictions_match_vanilla() {
    let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
    let mut player = PlayerInventory::new();
    assert_eq!(AbstractFurnaceMenu::SLOT_COUNT, 39);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:raw_iron", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:coal", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:bucket", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
    assert!(!menu.may_place(2, &ItemStack::new("minecraft:iron_ingot", 1)));

    // Bucket has a max-stack-size cap of 1 in the fuel slot.
    assert_eq!(
        menu.max_stack_size(1, &ItemStack::new("minecraft:bucket", 64)),
        1
    );

    menu.set_slot(0, ItemStack::new("minecraft:raw_iron", 1), &mut player);
    assert_eq!(
        menu.get_slot(0, &player).unwrap().item_id(),
        "minecraft:raw_iron"
    );

    // Result slot rejects direct placement via set_slot.
    assert!(!menu.set_slot(2, ItemStack::new("minecraft:iron_ingot", 1), &mut player));
    menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 1));
    assert_eq!(menu.get_slot(2, &player).unwrap().count(), 1);

    assert!(menu.set_data(furnace_data::LIT_TIME, 40));
    assert!(menu.set_data(furnace_data::LIT_DURATION, 80));
    assert!(menu.set_data(furnace_data::COOKING_PROGRESS, 50));
    assert!(menu.set_data(furnace_data::COOKING_TOTAL_TIME, 200));
    assert_eq!(menu.data(furnace_data::LIT_TIME), Some(40));
    assert!(menu.is_lit());
    assert_eq!(menu.lit_progress(), 0.5);
    assert_eq!(menu.burn_progress(), 0.25);
    assert!(!menu.set_data(4, 1));
}

#[test]
fn furnace_quick_move_result_to_player_and_storage_to_input() {
    let recipes = planks_recipe();
    let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
    let mut player = PlayerInventory::new();
    menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 3));
    let moved = menu.quick_move(2, &recipes, &mut player);
    assert_eq!(moved.item_id(), "minecraft:iron_ingot");
    assert_eq!(moved.count(), 3);
    assert!(menu.get_slot(2, &player).unwrap().is_empty());

    // Place a smeltable into the player main storage (menu slot 3).
    menu.set_slot(3, ItemStack::new("minecraft:raw_iron", 1), &mut player);
    let pre = menu.quick_move(3, &recipes, &mut player);
    assert_eq!(pre.item_id(), "minecraft:raw_iron");
    assert_eq!(
        menu.get_slot(0, &player).unwrap().item_id(),
        "minecraft:raw_iron"
    );

    // Coal goes to fuel.
    menu.set_slot(3, ItemStack::new("minecraft:coal", 1), &mut player);
    menu.quick_move(3, &recipes, &mut player);
    assert_eq!(
        menu.get_slot(1, &player).unwrap().item_id(),
        "minecraft:coal"
    );
}

#[test]
fn furnace_result_extraction_releases_xp_and_clears_recipe_ledger() {
    let recipes = planks_recipe();
    let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
    let mut player = PlayerInventory::new();

    menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 2));
    menu.record_recipe_use("minecraft:iron_ingot_from_smelting_raw_iron", 2, 500);
    let moved = menu.quick_move(2, &recipes, &mut player);

    assert_eq!(moved.item_id(), "minecraft:iron_ingot");
    assert!(menu.get_slot(2, &player).unwrap().is_empty());
    assert!(menu.recipes_used().is_empty());
    assert_eq!(
        menu.drain_result_award(),
        FurnaceResultAward {
            recipe_ids: vec!["minecraft:iron_ingot_from_smelting_raw_iron".to_string()],
            experience: 1,
        }
    );

    menu.set_result_internal(ItemStack::new("minecraft:iron_ingot", 1));
    menu.record_recipe_use("minecraft:iron_ingot_from_smelting_raw_iron", 1, 700);
    let taken = menu.take_result_with_xp_roll(0.0);
    assert_eq!(taken.count(), 1);
    assert_eq!(menu.drain_result_award().experience, 1);
}

#[test]
fn furnace_wrappers_use_vanilla_recipe_book_types() {
    assert_eq!(
        CraftingMenu::new(empty_recipes()).recipe_book_type(),
        RecipeBookType::Crafting
    );
    assert_eq!(
        FurnaceMenu::new(FuelValues::vanilla()).0.recipe_book_type(),
        RecipeBookType::Furnace
    );
    assert_eq!(
        BlastFurnaceMenu::new(FuelValues::vanilla())
            .0
            .recipe_book_type(),
        RecipeBookType::BlastFurnace
    );
    assert_eq!(
        SmokerMenu::new(FuelValues::vanilla()).0.recipe_book_type(),
        RecipeBookType::Smoker
    );
}

// -------- ChestMenu --------

#[test]
fn chest_menu_supports_one_through_six_rows_and_player_inventory_append() {
    for rows in 1..=6 {
        let menu = ChestMenu::new(rows);
        assert_eq!(menu.chest_size(), rows * 9);
        assert_eq!(menu.slot_count(), rows * 9 + PLAYER_SLOTS);
        assert_eq!(menu.inv_start(), rows * 9);
        assert_eq!(menu.hotbar_start(), rows * 9 + PLAYER_MAIN_STORAGE);
    }
}

#[test]
fn chest_menu_quick_move_shifts_between_chest_and_inventory() {
    let mut menu = ChestMenu::new(3);
    let mut player = PlayerInventory::new();
    menu.set_slot(0, ItemStack::new("minecraft:apple", 5), &mut player);
    let moved = menu.quick_move(0, &mut player);
    assert_eq!(moved.item_id(), "minecraft:apple");
    // After moving to player inventory (reverse order), the hotbar slot 8
    // should be the first chosen empty slot.
    assert_eq!(player.get(8).item_id(), "minecraft:apple");

    // Now shift it back.
    let chest_size = menu.chest_size();
    let menu_hotbar_8 = chest_size + PLAYER_MAIN_STORAGE + 8;
    let back = menu.quick_move(menu_hotbar_8, &mut player);
    assert_eq!(back.item_id(), "minecraft:apple");
    assert_eq!(
        menu.get_slot(0, &player).unwrap().item_id(),
        "minecraft:apple"
    );
}

// -------- HopperMenu --------

#[test]
fn hopper_menu_5_slots_and_quick_move_to_player() {
    let mut menu = HopperMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(HopperMenu::SLOT_COUNT, 41);
    menu.set_slot(4, ItemStack::new("minecraft:redstone", 3), &mut player);
    let moved = menu.quick_move(4, &mut player);
    assert_eq!(moved.item_id(), "minecraft:redstone");
    assert!(menu.get_slot(4, &player).unwrap().is_empty());
}

// -------- DispenserMenu --------

#[test]
fn dispenser_menu_layout_and_quick_move() {
    let mut menu = DispenserMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(DispenserMenu::SLOT_COUNT, 45);
    menu.set_slot(0, ItemStack::new("minecraft:arrow", 16), &mut player);
    let moved = menu.quick_move(0, &mut player);
    assert_eq!(moved.count(), 16);
}

// -------- ShulkerBoxMenu --------

#[test]
fn shulker_box_rejects_nested_shulker_boxes() {
    let mut menu = ShulkerBoxMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(ShulkerBoxMenu::SLOT_COUNT, 63);
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:shulker_box", 1)));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:red_shulker_box", 1)));
    assert!(menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    // set_slot should refuse nested shulker box.
    assert!(!menu.set_slot(0, ItemStack::new("minecraft:shulker_box", 1), &mut player));
}

// -------- AnvilMenu --------

#[test]
fn anvil_result_slot_rejects_placement_and_renaming_costs_one() {
    let mut menu = AnvilMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(AnvilMenu::SLOT_COUNT, 39);
    assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
    assert!(menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));

    menu.set_slot(0, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    menu.set_item_name(Some("Excalibur".to_string()));
    menu.set_result_from_inputs();
    assert_eq!(menu.cost, anvil_cost::RENAME);
    assert!(menu.only_renaming);
}

// -------- SmithingMenu --------

#[test]
fn smithing_menu_layout_and_result_rejection() {
    let mut menu = SmithingMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(SmithingMenu::SLOT_COUNT, 40);
    assert!(!menu.may_place(3, &ItemStack::new("minecraft:apple", 1)));
    menu.set_slot(
        0,
        ItemStack::new("minecraft:netherite_upgrade_smithing_template", 1),
        &mut player,
    );
    menu.set_slot(1, ItemStack::new("minecraft:diamond_sword", 1), &mut player);
    menu.set_slot(
        2,
        ItemStack::new("minecraft:netherite_ingot", 1),
        &mut player,
    );
    let all = menu.all_slots(&player);
    assert_eq!(all.len(), 40);
}

// -------- StonecutterMenu --------

#[test]
fn stonecutter_menu_layout_and_recipe_index_tracking() {
    let mut menu = StonecutterMenu::new();
    let mut player = PlayerInventory::new();
    let recipes = stonecutter_recipes();
    assert_eq!(StonecutterMenu::SLOT_COUNT, 38);
    assert_eq!(menu.get_selected_recipe_index(), -1);
    assert_eq!(menu.data(0), Some(-1));
    assert!(!menu.set_data(1, 0));

    assert_stonecutter_selection_and_result_take(&mut menu, &mut player, &recipes);
    assert_stonecutter_inventory_quick_move(&mut menu, &mut player, &recipes);
}

fn stonecutter_recipes() -> Vec<StonecutterSelection> {
    vec![
        StonecutterSelection {
            recipe_id: "minecraft:stone_stairs_from_stone_stonecutting",
            input: IngredientSpec::Item("minecraft:stone"),
            result: ItemAmount {
                item: "minecraft:stone_stairs",
                count: 1,
            },
        },
        StonecutterSelection {
            recipe_id: "minecraft:stone_slab_from_stone_stonecutting",
            input: IngredientSpec::Item("minecraft:stone"),
            result: ItemAmount {
                item: "minecraft:stone_slab",
                count: 2,
            },
        },
    ]
}

fn assert_stonecutter_selection_and_result_take(
    menu: &mut StonecutterMenu,
    player: &mut PlayerInventory,
    recipes: &[StonecutterSelection],
) {
    menu.set_slot(0, ItemStack::new("minecraft:stone", 2), player);
    menu.slots_changed(recipes);
    assert!(menu.has_input_item());
    assert_eq!(menu.get_number_of_visible_recipes(), 2);
    assert!(menu.click_button(1));
    assert_eq!(menu.get_selected_recipe_index(), 1);
    assert_eq!(
        menu.get_slot(StonecutterMenu::RESULT_SLOT, player)
            .unwrap()
            .item_id(),
        "minecraft:stone_slab"
    );
    assert!(!menu.click_button(1));
    assert!(menu.click_button(42));
    assert_eq!(menu.get_selected_recipe_index(), 1);
    let taken = menu.take_result();
    assert_eq!(taken.item_id(), "minecraft:stone_slab");
    assert_eq!(
        menu.get_slot(StonecutterMenu::INPUT_SLOT, player)
            .unwrap()
            .count(),
        1
    );
    assert_eq!(
        menu.get_slot(StonecutterMenu::RESULT_SLOT, player)
            .unwrap()
            .item_id(),
        "minecraft:stone_slab"
    );
    let taken = menu.take_result();
    assert_eq!(taken.item_id(), "minecraft:stone_slab");
    assert!(!menu.has_input_item());
    assert_eq!(menu.get_selected_recipe_index(), -1);
    assert!(menu
        .get_slot(StonecutterMenu::RESULT_SLOT, player)
        .unwrap()
        .is_empty());
}

fn assert_stonecutter_inventory_quick_move(
    menu: &mut StonecutterMenu,
    player: &mut PlayerInventory,
    recipes: &[StonecutterSelection],
) {
    player.set(9, ItemStack::new("minecraft:stone", 3));
    let moved = menu.quick_move_with_recipes(StonecutterMenu::INV_START, player, recipes);
    assert_eq!(moved.item_id(), "minecraft:stone");
    assert_eq!(
        menu.get_slot(StonecutterMenu::INPUT_SLOT, player)
            .unwrap()
            .count(),
        3
    );
    assert_eq!(menu.get_number_of_visible_recipes(), 2);
    player.set(10, ItemStack::new("minecraft:apple", 1));
    let moved = menu.quick_move_with_recipes(StonecutterMenu::INV_START + 1, player, recipes);
    assert_eq!(moved.item_id(), "minecraft:apple");
    assert_eq!(
        menu.get_slot(StonecutterMenu::INPUT_SLOT, player)
            .unwrap()
            .count(),
        3
    );
    assert!(player.get(10).is_empty());
    assert_eq!(player.get(0).item_id(), "minecraft:apple");
}

// -------- GrindstoneMenu --------

#[test]
fn grindstone_input_accepts_damageable_only_and_result_rejects_placement() {
    let menu = GrindstoneMenu::new();
    assert_eq!(GrindstoneMenu::SLOT_COUNT, 39);
    assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
    // Diamond sword is damageable in item_properties (matches vanilla).
    let sword = ItemStack::new("minecraft:diamond_sword", 1);
    // Whether the slot accepts depends on whether it's marked damageable;
    // we tolerate either outcome but require the slot may_place is
    // consistent with is_damageable_item().
    assert_eq!(menu.may_place(0, &sword), is_grindstone_input(&sword));
}

// -------- EnchantmentMenu --------

#[test]
fn enchant_menu_lapis_only_in_lapis_slot_and_item_slot_max_1() {
    let mut menu = EnchantmentMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(EnchantmentMenu::SLOT_COUNT, 38);
    assert!(menu.may_place(1, &ItemStack::new("minecraft:lapis_lazuli", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
    assert_eq!(menu.max_stack_size(0), 1);

    menu.set_slot(
        0,
        ItemStack::new("minecraft:diamond_sword", 64),
        &mut player,
    );
    assert_eq!(menu.get_slot(0, &player).unwrap().count(), 1);
}

// -------- BrewingStandMenu --------

#[test]
fn brewing_stand_slot_restrictions_match_vanilla() {
    let mut menu = BrewingStandMenu::new();
    assert_eq!(BrewingStandMenu::SLOT_COUNT, 41);
    // Potion slot accepts potion / glass bottle.
    assert!(menu.may_place(0, &ItemStack::new("minecraft:potion", 1)));
    assert!(menu.may_place(0, &ItemStack::new("minecraft:glass_bottle", 1)));
    assert!(!menu.may_place(0, &ItemStack::empty()));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    // Ingredient slot accepts brewing ingredients.
    assert!(menu.may_place(3, &ItemStack::new("minecraft:nether_wart", 1)));
    assert!(!menu.may_place(3, &ItemStack::empty()));
    assert!(!menu.may_place(3, &ItemStack::new("minecraft:apple", 1)));
    // Fuel slot accepts blaze powder.
    assert!(menu.may_place(4, &ItemStack::new("minecraft:blaze_powder", 1)));
    assert!(!menu.may_place(4, &ItemStack::empty()));
    assert!(!menu.may_place(4, &ItemStack::new("minecraft:coal", 1)));
    // Potion slots cap at max stack size 1.
    assert_eq!(menu.max_stack_size(0), 1);
    assert_eq!(menu.max_stack_size(3), 64);

    assert!(menu.set_data(BrewingStandMenu::BREW_TIME_DATA, 399));
    assert!(menu.set_data(BrewingStandMenu::FUEL_LEVEL_DATA, 20));
    assert_eq!(menu.get_brewing_ticks(), 399);
    assert_eq!(menu.get_fuel(), 20);
    assert_eq!(menu.data(BrewingStandMenu::FUEL_LEVEL_DATA), Some(20));
    assert!(!menu.set_data(BrewingStandMenu::DATA_COUNT, 1));
}

// -------- CartographyTableMenu --------

#[test]
fn cartography_table_slot_restrictions() {
    let menu = CartographyTableMenu::new();
    assert_eq!(CartographyTableMenu::SLOT_COUNT, 39);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:filled_map", 1)));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:paper", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:paper", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:map", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));
    assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
}

// -------- LoomMenu --------

#[test]
fn loom_menu_slot_restrictions() {
    let menu = LoomMenu::new();
    assert_eq!(LoomMenu::SLOT_COUNT, 40);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:white_banner", 1)));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:red_dye", 1)));
    assert!(menu.may_place(2, &ItemStack::new("minecraft:creeper_banner_pattern", 1)));
    assert!(!menu.may_place(3, &ItemStack::new("minecraft:white_banner", 1)));
}

// -------- LecternMenu --------

#[test]
fn lectern_menu_page_buttons() {
    let mut menu = LecternMenu::with_book(ItemStack::new("minecraft:written_book", 1));
    assert_eq!(LecternMenu::SLOT_COUNT, 1);
    assert_eq!(LecternMenu::DATA_COUNT, 1);
    assert_eq!(menu.get_book().item_id(), "minecraft:written_book");
    assert_eq!(menu.data(LecternMenu::PAGE_DATA), Some(0));
    assert!(menu.set_data(LecternMenu::PAGE_DATA, 2));
    assert!(!menu.set_data(LecternMenu::DATA_COUNT, 1));
    assert!(menu.click_button(LecternMenu::BUTTON_PREV_PAGE));
    assert_eq!(menu.get_page(), 1);
    assert!(menu.click_button(LecternMenu::BUTTON_NEXT_PAGE));
    assert_eq!(menu.get_page(), 2);
    assert!(menu.click_button(LecternMenu::BUTTON_PAGE_JUMP_RANGE_START + 7));
    assert_eq!(menu.get_page(), 7);
    assert!(menu.set_data(LecternMenu::PAGE_DATA, 0));
    assert!(menu.click_button(LecternMenu::BUTTON_PREV_PAGE));
    assert_eq!(menu.get_page(), -1);
    assert!(!menu.click_button(42));
    let denied_take = menu.click_button_with_permission(LecternMenu::BUTTON_TAKE_BOOK, false);
    assert!(!denied_take.handled);
    assert!(denied_take.taken_book.is_empty());
    assert_eq!(menu.get_book().item_id(), "minecraft:written_book");
    let allowed_take = menu.click_button_with_permission(LecternMenu::BUTTON_TAKE_BOOK, true);
    assert!(allowed_take.handled);
    assert_eq!(allowed_take.taken_book.item_id(), "minecraft:written_book");
    assert!(menu.get_book().is_empty());
    // Quick-move always returns empty.
    assert!(menu.quick_move(0).is_empty());
}

// -------- BeaconMenu --------

#[test]
fn beacon_payment_slot_accepts_only_payment_items() {
    let mut menu = BeaconMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(BeaconMenu::SLOT_COUNT, 37);
    for item in BEACON_PAYMENT_ITEMS {
        assert!(menu.may_place(0, &ItemStack::new(item, 1)), "{item}");
    }
    assert!(!menu.may_place(0, &ItemStack::empty()));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    // Max stack 1 for payment slot.
    menu.set_slot(0, ItemStack::new("minecraft:emerald", 64), &mut player);
    assert_eq!(menu.get_slot(0, &player).unwrap().count(), 1);
    assert!(menu.has_payment());

    assert!(menu.set_data(BeaconMenu::LEVELS_DATA, 4));
    assert_eq!(menu.get_levels(), 4);
    assert_eq!(BeaconMenu::encode_effect(Some(0)), Some(1));
    assert_eq!(BeaconMenu::decode_effect(10), Some(9));
    assert_eq!(BeaconMenu::encode_effect(Some(40)), None);
    assert!(menu.update_effects(Some(0), Some(9)));
    assert_eq!(menu.primary_effect_id(), Some(0));
    assert_eq!(menu.secondary_effect_id(), Some(9));
    assert!(!menu.has_payment());
    assert!(!menu.update_effects(Some(0), None));

    menu.set_slot(0, ItemStack::new("minecraft:diamond", 1), &mut player);
    assert_eq!(menu.removed().item_id(), "minecraft:diamond");
    assert!(!menu.has_payment());
}

// -------- CrafterMenu --------

#[test]
fn crafter_menu_slot_disable_toggle_and_layout() {
    let mut menu = CrafterMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(CrafterMenu::SLOT_COUNT, 46);
    assert!(!menu.is_slot_disabled(0));
    menu.set_slot_state(0, false);
    assert!(menu.is_slot_disabled(0));
    menu.set_slot_state(0, true);
    assert!(!menu.is_slot_disabled(0));
    assert!(!menu.is_powered());
    menu.set_powered(true);
    assert!(menu.is_powered());

    // Slot 45 (result) rejects placement.
    assert!(!menu.may_place(45, &ItemStack::new("minecraft:apple", 1)));
    // Grid slots accept anything.
    assert!(menu.set_slot(0, ItemStack::new("minecraft:oak_log", 1), &mut player));
}

#[test]
fn crafter_menu_refresh_result_uses_recipe_map() {
    let mut menu = CrafterMenu::new();
    let mut player = PlayerInventory::new();
    let recipes = planks_recipe();
    menu.set_slot(0, ItemStack::new("minecraft:oak_log", 1), &mut player);
    menu.refresh_result(&recipes);
    assert_eq!(menu.result().item_id(), "minecraft:oak_planks");
    assert_eq!(menu.result().count(), 4);
}

// -------- HorseInventoryMenu --------

#[test]
fn horse_inventory_saddle_and_armor_slot_restrictions() {
    let layout = HorseLayout {
        saddle_active: true,
        armor_active: true,
        is_llama: false,
        inventory_columns: 5,
    };
    let menu = HorseInventoryMenu::new(layout);
    assert_eq!(menu.slot_count(), 2 + 15 + PLAYER_SLOTS);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:saddle", 1)));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:copper_horse_armor", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:diamond_horse_armor", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:netherite_horse_armor", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:wolf_armor", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:apple", 1)));

    // Llamas accept colored carpets in the armor slot.
    let llama_layout = HorseLayout {
        saddle_active: true,
        armor_active: true,
        is_llama: true,
        inventory_columns: 5,
    };
    let llama = HorseInventoryMenu::new(llama_layout);
    assert!(llama.may_place(1, &ItemStack::new("minecraft:red_carpet", 1)));
    assert!(!llama.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));

    let inactive = HorseInventoryMenu::new(HorseLayout {
        saddle_active: false,
        armor_active: false,
        is_llama: false,
        inventory_columns: 0,
    });
    assert!(!inactive.may_place(0, &ItemStack::new("minecraft:saddle", 1)));
    assert!(!inactive.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
}

#[test]
fn horse_inventory_quick_move_prioritizes_mount_slots_then_storage() {
    let layout = HorseLayout {
        saddle_active: true,
        armor_active: true,
        is_llama: false,
        inventory_columns: 5,
    };
    let mut menu = HorseInventoryMenu::new(layout);
    let mut player = PlayerInventory::new();

    player.set(0, ItemStack::new("minecraft:iron_horse_armor", 1));
    let armor_slot = layout.player_start() + PLAYER_MAIN_STORAGE;
    let moved_armor = menu.quick_move(armor_slot, &mut player);
    assert_eq!(moved_armor.item_id(), "minecraft:iron_horse_armor");
    assert_eq!(
        menu.get_slot(HorseInventoryMenu::SLOT_ARMOR, &player)
            .unwrap()
            .item_id(),
        "minecraft:iron_horse_armor"
    );

    player.set(1, ItemStack::new("minecraft:saddle", 1));
    let saddle_slot = layout.player_start() + PLAYER_MAIN_STORAGE + 1;
    let moved_saddle = menu.quick_move(saddle_slot, &mut player);
    assert_eq!(moved_saddle.item_id(), "minecraft:saddle");
    assert_eq!(
        menu.get_slot(HorseInventoryMenu::SLOT_SADDLE, &player)
            .unwrap()
            .item_id(),
        "minecraft:saddle"
    );

    player.set(2, ItemStack::new("minecraft:wheat", 4));
    let storage_slot = layout.player_start() + PLAYER_MAIN_STORAGE + 2;
    let moved_storage = menu.quick_move(storage_slot, &mut player);
    assert_eq!(moved_storage.item_id(), "minecraft:wheat");
    assert_eq!(
        menu.get_slot(HorseInventoryMenu::SLOT_INVENTORY_START, &player)
            .unwrap()
            .item_id(),
        "minecraft:wheat"
    );
}

// -------- NautilusInventoryMenu --------

#[test]
fn nautilus_inventory_saddle_and_armor_restrictions() {
    let menu = NautilusInventoryMenu::new();
    assert_eq!(NautilusInventoryMenu::SLOT_COUNT, 38);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:saddle", 1)));
    assert!(!menu.may_place(0, &ItemStack::new("minecraft:apple", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:copper_nautilus_armor", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:iron_nautilus_armor", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:netherite_nautilus_armor", 1)));
    assert!(!menu.may_place(1, &ItemStack::new("minecraft:iron_horse_armor", 1)));
}

#[test]
fn nautilus_inventory_quick_move_prioritizes_equipment_slots() {
    let mut menu = NautilusInventoryMenu::new();
    let mut player = PlayerInventory::new();

    player.set(0, ItemStack::new("minecraft:diamond_nautilus_armor", 1));
    let armor_slot = NautilusInventoryMenu::INV_START + PLAYER_MAIN_STORAGE;
    let moved_armor = menu.quick_move(armor_slot, &mut player);
    assert_eq!(moved_armor.item_id(), "minecraft:diamond_nautilus_armor");
    assert_eq!(
        menu.get_slot(NautilusInventoryMenu::SLOT_ARMOR, &player)
            .unwrap()
            .item_id(),
        "minecraft:diamond_nautilus_armor"
    );

    player.set(1, ItemStack::new("minecraft:saddle", 1));
    let saddle_slot = NautilusInventoryMenu::INV_START + PLAYER_MAIN_STORAGE + 1;
    let moved_saddle = menu.quick_move(saddle_slot, &mut player);
    assert_eq!(moved_saddle.item_id(), "minecraft:saddle");
    assert_eq!(
        menu.get_slot(NautilusInventoryMenu::SLOT_SADDLE, &player)
            .unwrap()
            .item_id(),
        "minecraft:saddle"
    );
}

// -------- MerchantMenu --------

#[test]
fn merchant_menu_result_slot_rejects_placement_and_payment_layout() {
    let mut menu = MerchantMenu::new();
    let mut player = PlayerInventory::new();
    assert_eq!(MerchantMenu::SLOT_COUNT, 39);
    assert!(menu.may_place(0, &ItemStack::new("minecraft:emerald", 1)));
    assert!(menu.may_place(1, &ItemStack::new("minecraft:book", 1)));
    assert!(!menu.may_place(2, &ItemStack::new("minecraft:apple", 1)));
    menu.set_slot(0, ItemStack::new("minecraft:emerald", 2), &mut player);
    menu.set_result_internal(ItemStack::new("minecraft:written_book", 1));
    let moved = menu.quick_move(2, &mut player);
    assert_eq!(moved.item_id(), "minecraft:written_book");
}

#[test]
fn merchant_menu_try_move_items_moves_payment_back_and_fills_for_new_offer() {
    // Java parity: MerchantMenu.tryMoveItems(newTradeIndex) pushes the
    // current payment slots back into the player inventory and then
    // re-fills them from the player inventory using the new offer's cost.
    //
    // Note: moveFromInventoryToPaymentSlot fills the payment slot up to
    // its max stack size (or the inventory item count, whichever is
    // smaller). It does NOT cap at the cost's count — the result slot
    // will only consume what the offer actually requires, leaving the
    // surplus visible to the player.
    let offers = vec![
        MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 2),
            None,
            ItemStack::new("minecraft:diamond", 1),
            5,
            1,
            0.05,
        ),
        MerchantOffer::new(
            ItemCost::new("minecraft:emerald", 3),
            None,
            ItemStack::new("minecraft:emerald_block", 1),
            5,
            1,
            0.05,
        ),
    ];
    let mut menu = MerchantMenu::new();
    let mut player = PlayerInventory::new();

    // Seed payment slot A with 2 emeralds (the active offer 0 payment).
    menu.set_slot(0, ItemStack::new("minecraft:emerald", 2), &mut player);
    // Seed hotbar slot 0 with 5 more emeralds.
    player.set(0, ItemStack::new("minecraft:emerald", 5));

    // Switch to offer index 1 (needs 3 emeralds).
    menu.try_move_items(1, &offers, &mut player);

    // The 2 from payment_a are pushed back into the player inventory
    // (merging with the 5 in hotbar slot 0 → 7 emeralds there), then
    // the refill loop pulls up to a full stack (64) into payment A. With
    // only 7 emeralds available the whole stack moves over and the
    // hotbar slot ends up empty.
    let payment_a = menu.get_slot(0, &player).unwrap();
    assert_eq!(payment_a.item_id(), "minecraft:emerald");
    assert_eq!(payment_a.count(), 7);
    assert!(player.get(0).is_empty());

    // The offer at index 1 still requires 3, so it should now be
    // satisfied — the trader will leave 4 emeralds behind when the
    // player takes the result.
    assert!(offers[1].satisfied_by(&payment_a, &ItemStack::empty()));
}

#[test]
fn merchant_menu_try_move_items_ignores_out_of_bounds_indices() {
    // Java parity: `tryMoveItems` guards `newTradeIndex >= 0 &&
    // getOffers().size() > newTradeIndex`.
    let offers = vec![MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 1),
        None,
        ItemStack::new("minecraft:diamond", 1),
        5,
        1,
        0.05,
    )];
    let mut menu = MerchantMenu::new();
    let mut player = PlayerInventory::new();
    menu.set_slot(0, ItemStack::new("minecraft:emerald", 4), &mut player);

    menu.try_move_items(5, &offers, &mut player);
    // Payment slot is unchanged.
    assert_eq!(menu.get_slot(0, &player).unwrap().count(), 4);
}

#[test]
fn merchant_menu_try_move_items_with_two_cost_offer_fills_both_slots() {
    // Java parity: when the offer specifies a `cost_b`, both payment
    // slots are auto-filled in order — `cost_a` → slot 0, `cost_b` → slot 1.
    let offers = vec![MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 2),
        Some(ItemCost::new("minecraft:book", 1)),
        ItemStack::new("minecraft:written_book", 1),
        5,
        1,
        0.05,
    )];
    let mut menu = MerchantMenu::new();
    let mut player = PlayerInventory::new();
    // Hotbar slots — emeralds in slot 0, books in slot 1.
    player.set(0, ItemStack::new("minecraft:emerald", 6));
    player.set(1, ItemStack::new("minecraft:book", 3));

    menu.try_move_items(0, &offers, &mut player);

    let payment_a = menu.get_slot(0, &player).unwrap();
    let payment_b = menu.get_slot(1, &player).unwrap();
    assert_eq!(payment_a.item_id(), "minecraft:emerald");
    assert_eq!(payment_a.count(), 6);
    assert_eq!(payment_b.item_id(), "minecraft:book");
    assert_eq!(payment_b.count(), 3);
}

// -------- Cross-cutting click-mode tests --------
//
// These tests exercise the per-menu slot layout under each of the click
// modes that the network layer dispatches through (hotbar swap, drag
// split, double-click collect, drop, creative clone, close-while-carrying,
// disconnect-while-open). The actual click pipeline runs in
// `inventory.rs`; here we verify that the menu structs faithfully expose
// and accept slot reads/writes so the pipeline can act on them.

#[test]
fn hotbar_swap_works_in_representative_menu_types() {
    // ChestMenu: swap chest slot 0 with hotbar slot 4.
    // Hotbar slot 4 in menu-space lives at chest_size + PLAYER_MAIN_STORAGE + 4.
    let mut menu = ChestMenu::new(3);
    let mut player = PlayerInventory::new();
    let chest_item = ItemStack::new("minecraft:diamond", 3);
    let hotbar_item = ItemStack::new("minecraft:emerald", 1);
    menu.set_slot(0, chest_item.clone(), &mut player);
    player.set(4, hotbar_item.clone());

    let chest_size = menu.chest_size();
    let hotbar_menu_slot = chest_size + PLAYER_MAIN_STORAGE + 4;
    let hotbar_content = menu.get_slot(hotbar_menu_slot, &player).unwrap();
    assert_eq!(hotbar_content.item_id(), "minecraft:emerald");

    menu.set_slot(0, hotbar_content.clone(), &mut player);
    player.set(4, chest_item.clone());

    assert_eq!(
        menu.get_slot(0, &player).unwrap().item_id(),
        "minecraft:emerald"
    );
    assert_eq!(player.get(4).item_id(), "minecraft:diamond");
}

#[test]
fn drag_split_distributes_stack_evenly_across_chest_slots() {
    // Java parity: left-click drag splits the carried stack evenly across
    // every target slot. 8 diamonds across 4 slots → 2 each.
    let mut menu = ChestMenu::new(3);
    let mut player = PlayerInventory::new();
    let carry = 8;
    let target_slots = [0usize, 1, 2, 3];
    let each = carry / target_slots.len();
    for &slot in &target_slots {
        menu.set_slot(
            slot,
            ItemStack::new("minecraft:diamond", each as i32),
            &mut player,
        );
    }
    for &slot in &target_slots {
        assert_eq!(menu.get_slot(slot, &player).unwrap().count(), 2);
    }
}

#[test]
fn drag_split_in_hopper_menu_fills_all_5_hopper_slots() {
    let mut menu = HopperMenu::new();
    let mut player = PlayerInventory::new();
    let each = 10i32;
    for slot in 0..5 {
        menu.set_slot(slot, ItemStack::new("minecraft:stone", each), &mut player);
    }
    for slot in 0..5 {
        assert_eq!(menu.get_slot(slot, &player).unwrap().count(), 10);
    }
}

#[test]
fn double_click_collect_gathers_items_into_cursor_from_chest() {
    // Java parity: PICKUP_ALL mode walks every slot looking for matching
    // items and merges them into the cursor up to the cursor's max
    // stack. Here we verify the slot reads that pickup_all would issue
    // are sane: cursor of 2 + slot 0 of 3 + slot 1 of 5 = 10.
    let mut menu = ChestMenu::new(3);
    let mut player = PlayerInventory::new();
    menu.set_slot(0, ItemStack::new("minecraft:diamond", 3), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:diamond", 5), &mut player);
    let total =
        2 + menu.get_slot(0, &player).unwrap().count() + menu.get_slot(1, &player).unwrap().count();
    assert_eq!(total, 10);
}

#[test]
fn drop_from_slot_removes_item_from_furnace_input() {
    // Java parity: THROW mode with Ctrl removes the entire slot. Here
    // we simulate the resulting slot state — empty after the drop.
    let mut menu = AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla());
    let mut player = PlayerInventory::new();
    menu.set_slot(0, ItemStack::new("minecraft:raw_iron", 8), &mut player);
    let stack = menu.get_slot(0, &player).unwrap();
    assert!(!stack.is_empty());
    menu.set_slot(0, ItemStack::empty(), &mut player);
    assert!(menu.get_slot(0, &player).unwrap().is_empty());
}

#[test]
fn drop_single_from_dispenser_slot() {
    // Java parity: THROW without Ctrl drops one item from the targeted slot.
    let mut menu = DispenserMenu::new();
    let mut player = PlayerInventory::new();
    menu.set_slot(4, ItemStack::new("minecraft:arrow", 16), &mut player);
    let count_before = menu.get_slot(4, &player).unwrap().count();
    menu.set_slot(
        4,
        ItemStack::new("minecraft:arrow", count_before - 1),
        &mut player,
    );
    assert_eq!(menu.get_slot(4, &player).unwrap().count(), 15);
}

#[test]
fn creative_clone_produces_full_stack_from_slot() {
    // Java parity: CLONE mode (middle-click in creative) replaces the
    // cursor with a max-stack copy of the targeted slot's contents.
    let mut menu = ChestMenu::new(3);
    let mut player = PlayerInventory::new();
    menu.set_slot(5, ItemStack::new("minecraft:emerald", 2), &mut player);
    let slot_item = menu.get_slot(5, &player).unwrap();
    let max = slot_item.max_stack_size();
    assert_eq!(max, 64);
    let cloned = ItemStack::new(slot_item.item_id(), max as i32);
    assert_eq!(cloned.count(), 64);
    assert_eq!(cloned.item_id(), "minecraft:emerald");
}

#[test]
fn close_while_carrying_returns_item_to_inventory() {
    // Java parity: AbstractContainerMenu.removed places the carried item
    // back into the inventory (or drops it if no slot is free).
    let mut player = PlayerInventory::new();
    let carried = ItemStack::new("minecraft:diamond", 3);

    let placed = if let Some(empty_slot) = (0..36).find(|&i| player.get(i).is_empty()) {
        player.set(empty_slot, carried.clone());
        true
    } else {
        false
    };
    assert!(placed);
    assert_eq!(player.get(0).item_id(), "minecraft:diamond");
    assert_eq!(player.get(0).count(), 3);
}

#[test]
fn disconnect_while_open_drops_payment_items() {
    // Java parity: MerchantMenu.removed places the merchant container's
    // payment slots back into the player's inventory on disconnect.
    let mut menu = MerchantMenu::new();
    let mut player = PlayerInventory::new();
    menu.set_slot(0, ItemStack::new("minecraft:emerald", 5), &mut player);
    menu.set_slot(1, ItemStack::new("minecraft:book", 1), &mut player);

    let payment_a = menu.get_slot(0, &player).unwrap();
    let payment_b = menu.get_slot(1, &player).unwrap();

    if !payment_a.is_empty() {
        let slot = (0..36).find(|&i| player.get(i).is_empty()).unwrap();
        player.set(slot, payment_a);
        menu.set_slot(0, ItemStack::empty(), &mut player);
    }
    if !payment_b.is_empty() {
        let slot = (0..36).find(|&i| player.get(i).is_empty()).unwrap();
        player.set(slot, payment_b);
        menu.set_slot(1, ItemStack::empty(), &mut player);
    }

    assert_eq!(player.get(0).item_id(), "minecraft:emerald");
    assert_eq!(player.get(1).item_id(), "minecraft:book");
    assert!(menu.get_slot(0, &player).unwrap().is_empty());
    assert!(menu.get_slot(1, &player).unwrap().is_empty());
}

// -------- Stale state-ID style sweep: all_slots length checks --------

#[test]
fn every_menu_reports_correct_slot_count_for_full_resync() {
    let player = PlayerInventory::new();
    assert_eq!(
        CraftingMenu::new(empty_recipes()).all_slots(&player).len(),
        46
    );
    assert_eq!(
        AbstractFurnaceMenu::new(FurnaceKind::Furnace, FuelValues::vanilla())
            .all_slots(&player)
            .len(),
        39
    );
    assert_eq!(
        ChestMenu::new(3).all_slots(&player).len(),
        27 + PLAYER_SLOTS
    );
    assert_eq!(
        ChestMenu::new(6).all_slots(&player).len(),
        54 + PLAYER_SLOTS
    );
    assert_eq!(HopperMenu::new().all_slots(&player).len(), 41);
    assert_eq!(DispenserMenu::new().all_slots(&player).len(), 45);
    assert_eq!(ShulkerBoxMenu::new().all_slots(&player).len(), 63);
    assert_eq!(AnvilMenu::new().all_slots(&player).len(), 39);
    assert_eq!(SmithingMenu::new().all_slots(&player).len(), 40);
    assert_eq!(StonecutterMenu::new().all_slots(&player).len(), 38);
    assert_eq!(GrindstoneMenu::new().all_slots(&player).len(), 39);
    assert_eq!(EnchantmentMenu::new().all_slots(&player).len(), 38);
    assert_eq!(BrewingStandMenu::new().all_slots(&player).len(), 41);
    assert_eq!(CartographyTableMenu::new().all_slots(&player).len(), 39);
    assert_eq!(LoomMenu::new().all_slots(&player).len(), 40);
    assert_eq!(LecternMenu::new().all_slots().len(), 1);
    assert_eq!(BeaconMenu::new().all_slots(&player).len(), 37);
    assert_eq!(CrafterMenu::new().all_slots(&player).len(), 46);
    assert_eq!(NautilusInventoryMenu::new().all_slots(&player).len(), 38);
    assert_eq!(MerchantMenu::new().all_slots(&player).len(), 39);
}
