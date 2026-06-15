use super::*;

fn crafting_test_recipes() -> crate::recipe_system::RecipeMap {
    crate::recipe_system::RecipeMap::create(vec![
        crate::recipe_system::RecipeHolder {
            id: "minecraft:oak_planks",
            recipe: crate::recipe_system::RecipeKind::Shapeless {
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
                ingredients: vec![crate::recipe_system::IngredientSpec::Item(
                    "minecraft:oak_log",
                )],
                result: crate::recipe_system::ItemAmount {
                    item: "minecraft:oak_planks",
                    count: 4,
                },
            },
        },
        crate::recipe_system::RecipeHolder {
            id: "minecraft:crafting_table",
            recipe: crate::recipe_system::RecipeKind::Shaped {
                width: 2,
                height: 2,
                pattern: vec![
                    Some(crate::recipe_system::IngredientSpec::Item(
                        "minecraft:oak_planks",
                    )),
                    Some(crate::recipe_system::IngredientSpec::Item(
                        "minecraft:oak_planks",
                    )),
                    Some(crate::recipe_system::IngredientSpec::Item(
                        "minecraft:oak_planks",
                    )),
                    Some(crate::recipe_system::IngredientSpec::Item(
                        "minecraft:oak_planks",
                    )),
                ],
                result: crate::recipe_system::ItemAmount::one("minecraft:crafting_table"),
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
            },
        },
        crate::recipe_system::RecipeHolder {
            id: "minecraft:test_bucket_recipe",
            recipe: crate::recipe_system::RecipeKind::Shapeless {
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
                ingredients: vec![crate::recipe_system::IngredientSpec::Item(
                    "minecraft:water_bucket",
                )],
                result: crate::recipe_system::ItemAmount::one("minecraft:clay"),
            },
        },
        crate::recipe_system::RecipeHolder {
            id: "minecraft:torch",
            recipe: crate::recipe_system::RecipeKind::Shapeless {
                category: crate::recipe_system::CraftingBookCategoryModel::Misc,
                ingredients: vec![
                    crate::recipe_system::IngredientSpec::Item("minecraft:stick"),
                    crate::recipe_system::IngredientSpec::Item("minecraft:stick"),
                ],
                result: crate::recipe_system::ItemAmount {
                    item: "minecraft:torch",
                    count: 4,
                },
            },
        },
    ])
}

#[test]
fn player_inventory_uses_vanilla_slot_mapping_and_hotbar_validation() {
    let mut inventory = PlayerInventory::new();
    assert_eq!(inventory.container_size(), 43);
    assert_eq!(
        PlayerInventory::equipment_slot(36),
        Some(EquipmentSlot::Feet)
    );
    assert_eq!(
        PlayerInventory::equipment_slot(39),
        Some(EquipmentSlot::Head)
    );
    assert_eq!(
        PlayerInventory::equipment_slot(SLOT_OFFHAND),
        Some(EquipmentSlot::Offhand)
    );
    assert_eq!(
        PlayerInventory::equipment_slot(SLOT_BODY_ARMOR),
        Some(EquipmentSlot::Body)
    );
    assert_eq!(
        PlayerInventory::equipment_slot(SLOT_SADDLE),
        Some(EquipmentSlot::Saddle)
    );

    assert!(inventory.set_selected_slot(8).is_ok());
    assert_eq!(inventory.selected_slot(), 8);
    assert!(inventory.set_selected_slot(9).is_err());
}

#[test]
fn add_prefers_selected_then_offhand_then_existing_inventory_space() {
    let mut inventory = PlayerInventory::new();
    inventory.set_selected_slot(2).unwrap();
    inventory.set(2, ItemStack::new("minecraft:stick", 60));
    inventory.set(SLOT_OFFHAND, ItemStack::new("minecraft:stick", 10));
    inventory.set(5, ItemStack::new("minecraft:stick", 10));

    assert_eq!(
        inventory.add(ItemStack::new("minecraft:stick", 12)),
        InventoryAddResult::FullyAdded
    );
    assert_eq!(inventory.get(2).count(), 64);
    assert_eq!(inventory.get(SLOT_OFFHAND).count(), 18);
    assert_eq!(inventory.get(5).count(), 10);
    assert_eq!(inventory.get(2).pop_time(), 5);
}

#[test]
fn add_to_empty_slot_moves_full_stack_not_single_item() {
    let mut inventory = PlayerInventory::new();

    assert_eq!(
        inventory.add(ItemStack::new("minecraft:stone", 64)),
        InventoryAddResult::FullyAdded
    );
    assert_eq!(inventory.get(0).item_id(), "minecraft:stone");
    assert_eq!(inventory.get(0).count(), 64);
    assert_eq!(inventory.get(0).pop_time(), 5);
}

#[test]
fn inventory_save_load_tracks_only_non_equipment_storage_slots() {
    let mut inventory = PlayerInventory::new();
    inventory.set(0, ItemStack::new("minecraft:apple", 3));
    inventory.set(37, ItemStack::new("minecraft:diamond_helmet", 1));
    inventory.set(SLOT_OFFHAND, ItemStack::new("minecraft:shield", 1));

    let saved = inventory.saved_items();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].0, 0);

    let mut loaded = PlayerInventory::new();
    loaded.load_items(&saved);
    assert_eq!(loaded.get(0).item_id(), "minecraft:apple");
    assert!(loaded.get(37).is_empty());
    assert!(loaded.get(SLOT_OFFHAND).is_empty());
}

#[test]
fn place_item_back_drops_when_inventory_is_full() {
    let mut inventory = PlayerInventory::new();
    for slot in 0..INVENTORY_SIZE {
        inventory.set(slot, ItemStack::new("minecraft:apple", 64));
    }

    assert_eq!(
        inventory.place_item_back_in_inventory(ItemStack::new("minecraft:stick", 3)),
        InventoryAddResult::Dropped { count: 3 }
    );
    assert_eq!(inventory.dropped()[0].item_id(), "minecraft:stick");
}

#[test]
fn ender_chest_and_generic_containers_have_fixed_slot_behavior() {
    let mut ender_chest = SimpleContainer::ender_chest();
    assert_eq!(ender_chest.size(), 27);
    ender_chest.set(26, ItemStack::new("minecraft:apple", 70));
    assert_eq!(ender_chest.get(26).count(), 64);
    assert_eq!(ender_chest.remove(26, 8).count(), 8);
    assert_eq!(ender_chest.get(26).count(), 56);
}

#[test]
fn horse_inventory_layout_matches_equipment_and_chest_slots() {
    let horse = HorseInventoryLayout::new(true, true, false, 5);
    assert!(horse.saddle_slot_active);
    assert!(horse.armor_slot_active);
    assert_eq!(horse.chest_slots, 15);
    assert_eq!(horse.player_inventory_start, 17);

    let llama = HorseInventoryLayout::new(false, false, true, 3);
    assert!(!llama.saddle_slot_active);
    assert!(llama.armor_slot_active);
    assert_eq!(llama.chest_slots, 9);
}

#[test]
fn merchant_offer_applies_special_price_demand_stock_and_payment_consumption() {
    let java_source = vibecraft_java_source!("/net/minecraft/world/item/trading/MerchantOffer.java");
    for sentinel in [
        "ItemCost.CODEC.fieldOf(\"buy\")",
        "ItemCost.CODEC.lenientOptionalFieldOf(\"buyB\")",
        "ItemStack.CODEC.fieldOf(\"sell\")",
        "Codec.INT.lenientOptionalFieldOf(\"uses\", 0)",
        "Codec.INT.lenientOptionalFieldOf(\"maxUses\", 4)",
        "Codec.BOOL.lenientOptionalFieldOf(\"rewardExp\", true)",
        "Codec.INT.lenientOptionalFieldOf(\"specialPrice\", 0)",
        "Codec.INT.lenientOptionalFieldOf(\"demand\", 0)",
        "Codec.FLOAT.lenientOptionalFieldOf(\"priceMultiplier\", 0.0F)",
        "Codec.INT.lenientOptionalFieldOf(\"xp\", 1)",
        "public boolean shouldRewardExp()",
        "output.writeInt(offer.getSpecialPriceDiff())",
        "output.writeFloat(offer.getPriceMultiplier())",
        "output.writeInt(offer.getDemand())",
    ] {
        assert!(
            java_source.contains(sentinel),
            "missing MerchantOffer Java sentinel {sentinel}"
        );
    }
    assert!(
        !java_source.contains("ignoreDiscount"),
        "Java 26.1.2 MerchantOffer does not define an ignoreDiscount codec field"
    );

    let mut offer = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 5),
        Some(ItemCost::new("minecraft:book", 1)),
        ItemStack::new("minecraft:written_book", 1),
        2,
        7,
        0.2,
    );
    assert_eq!(offer.base_cost_a, ItemCost::new("minecraft:emerald", 5));
    assert_eq!(offer.cost_b, Some(ItemCost::new("minecraft:book", 1)));
    assert_eq!(offer.result, ItemStack::new("minecraft:written_book", 1));
    assert_eq!(offer.uses, 0);
    assert_eq!(offer.max_uses, 2);
    assert!(offer.reward_exp);
    assert_eq!(offer.special_price_diff, 0);
    assert_eq!(offer.demand, 0);
    assert_eq!(offer.price_multiplier, 0.2);
    assert_eq!(offer.xp, 7);
    assert!(!offer.ignore_discount);

    offer.demand = 3;
    offer.special_price_diff = -1;
    assert_eq!(offer.cost_a_count(), 7);
    offer.special_price_diff = -100;
    assert_eq!(offer.cost_a_count(), 1);
    offer.demand = 100;
    offer.special_price_diff = 0;
    assert_eq!(offer.cost_a_count(), 64);
    offer.demand = 3;
    offer.special_price_diff = -1;

    let mut emeralds = ItemStack::new("minecraft:emerald", 8);
    let mut book = ItemStack::new("minecraft:book", 1);
    assert!(offer.take(&mut emeralds, &mut book));
    assert_eq!(emeralds.count(), 1);
    assert!(book.is_empty());

    offer.increase_uses();
    assert!(offer.needs_restock());
    offer.increase_uses();
    assert!(offer.is_out_of_stock());
    offer.update_demand();
    assert_eq!(offer.demand, 5);

    offer.demand = 6;
    offer.uses = 1;
    offer.max_uses = 4;
    offer.update_demand();
    assert_eq!(offer.demand, 4);
}

#[test]
fn merchant_container_selects_active_offer_and_clears_out_of_stock_results() {
    let offer = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 2),
        None,
        ItemStack::new("minecraft:apple", 4),
        1,
        3,
        0.0,
    );
    let mut container = MerchantContainer::new(vec![offer]);
    container.set_payment(0, ItemStack::new("minecraft:emerald", 2));

    assert!(container.can_trade());
    assert_eq!(container.result().item_id(), "minecraft:apple");
    assert_eq!(container.future_xp(), 3);
    assert_eq!(container.take_result().count(), 4);
    assert!(!container.can_trade());
    assert!(container.result().is_empty());
    assert!(container.offer(0).unwrap().is_out_of_stock());

    container.prepare_trade();
    assert!(!container.can_trade());
    assert!(container.result().is_empty());
}

#[test]
fn merchant_result_take_increments_uses_and_villager_xp() {
    let offer = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 2),
        None,
        ItemStack::new("minecraft:apple", 4),
        3,
        7,
        0.0,
    );
    let mut container = MerchantContainer::new(vec![offer]);
    container.set_payment(0, ItemStack::new("minecraft:emerald", 2));

    assert_eq!(container.take_result().item_id(), "minecraft:apple");
    assert_eq!(container.offers()[0].uses, 1);
    assert_eq!(container.villager_xp(), 7);
    assert!(container.result().is_empty());
}

#[test]
fn merchant_container_respects_selection_hint_and_payment_slots() {
    let cheap = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 1),
        None,
        ItemStack::new("minecraft:apple", 1),
        8,
        1,
        0.0,
    );
    let specific = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 1),
        Some(ItemCost::new("minecraft:book", 1)),
        ItemStack::new("minecraft:written_book", 1),
        8,
        5,
        0.0,
    );
    let mut container = MerchantContainer::new(vec![cheap, specific]);

    container.set_selection_hint(1);
    container.set_payment(0, ItemStack::new("minecraft:emerald", 1));
    container.set_payment(1, ItemStack::new("minecraft:book", 1));
    assert!(container.can_trade());
    assert_eq!(container.active_offer(), container.offer(1));
    assert_eq!(container.result().item_id(), "minecraft:written_book");
    assert_eq!(container.future_xp(), 5);

    container.set_payment(1, ItemStack::empty());
    container.set_payment(0, ItemStack::empty());
    container.set_payment(1, ItemStack::new("minecraft:emerald", 1));
    container.prepare_trade();
    assert!(container.can_trade());
    assert_eq!(container.active_offer(), container.offer(0));
    assert_eq!(container.result().item_id(), "minecraft:apple");

    container.set_payment(1, ItemStack::empty());
    container.set_payment(0, ItemStack::new("minecraft:dirt", 1));
    assert!(!container.can_trade());
    assert!(container.result().is_empty());
    assert_eq!(container.future_xp(), 0);
}

#[test]
fn merchant_container_take_result_consumes_swapped_payment_slot() {
    let offer = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 2),
        None,
        ItemStack::new("minecraft:apple", 4),
        3,
        1,
        0.0,
    );
    let mut container = MerchantContainer::new(vec![offer]);

    // Java MerchantResultSlot.onTake tries offer.take(buyA, buyB) and then
    // offer.take(buyB, buyA), so a payment placed in slot 1 is consumed too.
    container.set_payment(1, ItemStack::new("minecraft:emerald", 2));
    assert!(container.can_trade());
    assert_eq!(container.result().item_id(), "minecraft:apple");

    let result = container.take_result();
    assert_eq!(result.item_id(), "minecraft:apple");
    assert!(container.result().is_empty());
    assert!(container.offer(0).unwrap().needs_restock());
    assert!(!container.can_trade());
}

#[test]
fn merchant_offer_demand_increases_after_purchase_and_resets_after_restock() {
    // Java parity: trading consumes uses; updateDemand uses the formula
    // demand += uses - (maxUses - uses); restock then clears uses.
    let mut container = MerchantContainer::new(vec![MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 5),
        None,
        ItemStack::new("minecraft:written_book", 1),
        2,    // max_uses
        1,    // xp
        0.05, // price_multiplier
    )]);

    // Place enough emeralds to trade
    container.set_payment(0, ItemStack::new("minecraft:emerald", 5));
    assert!(container.can_trade());

    // First trade — uses becomes 1.
    let result = container.take_result();
    assert_eq!(result.item_id(), "minecraft:written_book");
    assert_eq!(container.offers()[0].uses, 1);

    // Second trade — uses becomes 2, offer goes out of stock.
    container.set_payment(0, ItemStack::new("minecraft:emerald", 5));
    let _ = container.take_result();
    assert_eq!(container.offers()[0].uses, 2);
    assert!(container.offers()[0].is_out_of_stock());

    // Restock: demand += uses - (maxUses - uses) = 0 + 2 - 0 = 2, then
    // every offer's uses counter resets to 0.
    container.restock();
    assert_eq!(container.offers()[0].uses, 0);
    assert_eq!(container.offers()[0].demand, 2);

    // With a higher price multiplier (0.5) and demand=2 the cost shifts
    // from 5 to 10 emeralds: 5 + floor(5 * 2 * 0.5) = 5 + 5 = 10.
    let mut offer2 = MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 5),
        None,
        ItemStack::new("minecraft:written_book", 1),
        2,
        1,
        0.5,
    );
    offer2.demand = 2;
    assert_eq!(offer2.cost_a_count(), 10);
}

#[test]
fn merchant_restock_tracker_allows_two_restocks_per_day_2400_ticks_apart() {
    // Java parity: Villager.allowedToRestock() returns true when the
    // villager has never restocked today, or it has restocked once and
    // 2400 ticks have passed since that restock.
    let mut tracker = MerchantRestockTracker::new();

    // Initially allowed (first restock of the day).
    assert!(tracker.allowed_to_restock(0));
    tracker.record_restock(100);
    assert_eq!(tracker.restocks_today, 1);

    // Second restock requires 2400 ticks of cooldown to elapse.
    assert!(!tracker.allowed_to_restock(100));
    assert!(!tracker.allowed_to_restock(2499));
    assert!(tracker.allowed_to_restock(2501));

    tracker.record_restock(2501);
    assert_eq!(tracker.restocks_today, 2);

    // After two restocks no further restock is allowed regardless of
    // how much time has passed — the cap resets only on a new day.
    assert!(!tracker.allowed_to_restock(9999));

    // resetForNewDay clears the daily counter.
    tracker.reset_for_new_day();
    assert!(tracker.allowed_to_restock(9999));
}

#[test]
fn merchant_hero_discount_reduces_special_price_diff() {
    // Java parity: Villager.updateSpecialPrices subtracts
    // max(floor((0.3 + 0.0625 * amplifier) * baseCostA.count), 1) from
    // each offer's specialPriceDiff when the player has the
    // hero-of-the-village effect.
    let mut container = MerchantContainer::new(vec![MerchantOffer::new(
        ItemCost::new("minecraft:emerald", 10),
        None,
        ItemStack::new("minecraft:diamond", 1),
        5,
        1,
        0.05,
    )]);

    // amplifier 0: modifier = 0.3, costReduction = floor(0.3 * 10) = 3.
    container.apply_hero_discount(0);
    assert_eq!(container.offers()[0].special_price_diff, -3);

    // Reset and verify amplifier 1 yields the same rounded discount
    // (modifier = 0.3625, floor(0.3625 * 10) = 3).
    container.reset_special_prices();
    assert_eq!(container.offers()[0].special_price_diff, 0);
    container.apply_hero_discount(1);
    assert_eq!(container.offers()[0].special_price_diff, -3);
}

#[test]
fn crafting_grid_updates_result_and_consumes_inputs_after_take() {
    let recipes = crafting_test_recipes();
    let mut grid = CraftingGrid::two_by_two();
    grid.set_input(0, ItemStack::new("minecraft:oak_log", 1), &recipes);
    assert_eq!(grid.recipe_id(), Some("minecraft:oak_planks"));
    assert_eq!(grid.result().item_id(), "minecraft:oak_planks");
    assert_eq!(grid.result().count(), 4);
    assert_eq!(grid.take_result(&recipes).count(), 4);
    assert!(grid.result().is_empty());

    grid.set_input(0, ItemStack::new("minecraft:stick", 1), &recipes);
    grid.set_input(1, ItemStack::new("minecraft:stick", 1), &recipes);

    assert_eq!(grid.recipe_id(), Some("minecraft:torch"));
    assert_eq!(grid.result().item_id(), "minecraft:torch");
    assert_eq!(grid.result().count(), 4);
    assert_eq!(grid.take_result(&recipes).count(), 4);
    assert!(grid.result().is_empty());
    assert_eq!(grid.input_count("minecraft:stick"), 0);

    grid.set_input(0, ItemStack::new("minecraft:stick", 1), &recipes);
    grid.set_input(3, ItemStack::new("minecraft:oak_log", 1), &recipes);
    assert!(grid.recipe_id().is_none());
    assert!(grid.result().is_empty());

    let mut wrong_shaped = CraftingGrid::two_by_two();
    wrong_shaped.set_input(0, ItemStack::new("minecraft:oak_planks", 1), &recipes);
    wrong_shaped.set_input(1, ItemStack::new("minecraft:oak_planks", 1), &recipes);
    wrong_shaped.set_input(2, ItemStack::new("minecraft:oak_planks", 1), &recipes);
    assert!(wrong_shaped.recipe_id().is_none());
    assert!(wrong_shaped.result().is_empty());

    let table = CraftingGrid::three_by_three();
    assert_eq!(table.width, 3);
    assert_eq!(table.height, 3);
}

#[test]
fn inventory_menu_maps_vanilla_slots_to_backing_inventory_and_crafting_grid() {
    let mut player = PlayerInventory::new();
    player.set(0, ItemStack::new("minecraft:stick", 9));
    player.set(9, ItemStack::new("minecraft:cobblestone", 8));
    player.set(36, ItemStack::new("minecraft:leather_boots", 1));
    player.set(37, ItemStack::new("minecraft:iron_leggings", 1));
    player.set(38, ItemStack::new("minecraft:iron_chestplate", 1));
    player.set(39, ItemStack::new("minecraft:iron_helmet", 1));
    player.set(SLOT_OFFHAND, ItemStack::new("minecraft:shield", 1));

    let mut menu = InventoryMenu::new(player, crafting_test_recipes());
    assert_eq!(InventoryMenu::SLOT_COUNT, 46);
    assert!(!menu.may_place(0));
    assert!(menu.may_place(1));
    assert_eq!(menu.get_slot(36).unwrap().item_id(), "minecraft:stick");
    assert_eq!(menu.get_slot(9).unwrap().item_id(), "minecraft:cobblestone");
    assert_eq!(menu.get_slot(5).unwrap().item_id(), "minecraft:iron_helmet");
    assert_eq!(
        menu.get_slot(6).unwrap().item_id(),
        "minecraft:iron_chestplate"
    );
    assert_eq!(
        menu.get_slot(7).unwrap().item_id(),
        "minecraft:iron_leggings"
    );
    assert_eq!(
        menu.get_slot(8).unwrap().item_id(),
        "minecraft:leather_boots"
    );
    assert_eq!(menu.get_slot(45).unwrap().item_id(), "minecraft:shield");

    assert!(!menu.set_slot(0, ItemStack::new("minecraft:diamond", 1)));
    assert!(menu.get_slot(0).unwrap().is_empty());
    assert!(menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1)));
    assert_eq!(menu.get_slot(0).unwrap().item_id(), "minecraft:oak_planks");
    assert_eq!(menu.get_slot(0).unwrap().count(), 4);
    assert_eq!(
        menu.crafting_grid().recipe_id(),
        Some("minecraft:oak_planks")
    );

    assert!(menu.set_slot(36, ItemStack::new("minecraft:apple", 2)));
    assert_eq!(
        menu.player_inventory().get(0),
        &ItemStack::new("minecraft:apple", 2)
    );
    assert_eq!(menu.all_slots().len(), 46);
}

#[test]
fn inventory_menu_result_take_consumes_inputs_remainders_and_unlocks_recipe_once() {
    let mut menu = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());

    assert!(menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1)));
    assert_eq!(menu.get_slot(0).unwrap().item_id(), "minecraft:clay");
    let result = menu.take_result();
    assert_eq!(result.item_id(), "minecraft:clay");
    assert_eq!(menu.get_slot(1).unwrap().item_id(), "minecraft:bucket");
    assert!(menu.get_slot(0).unwrap().is_empty());
    assert_eq!(
        menu.recipe_unlock_events(),
        &["minecraft:test_bucket_recipe"]
    );

    menu.set_slot(1, ItemStack::new("minecraft:water_bucket", 1));
    let second = menu.take_result();
    assert_eq!(second.item_id(), "minecraft:clay");
    assert_eq!(
        menu.recipe_unlock_events(),
        &["minecraft:test_bucket_recipe"],
        "recipe-book unlock should be emitted only for first craft"
    );
}

#[test]
fn inventory_menu_direct_recipe_unlock_highlights_and_deduplicates() {
    let mut menu = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());

    assert!(menu.unlock_recipe("minecraft:oak_planks"));
    assert_eq!(menu.recipe_unlock_events(), &["minecraft:oak_planks"]);
    assert_eq!(
        menu.recipe_book_known_recipes(),
        vec!["minecraft:oak_planks"]
    );
    assert_eq!(
        menu.recipe_book_highlighted_recipes(),
        vec!["minecraft:oak_planks"]
    );

    assert!(!menu.unlock_recipe("minecraft:oak_planks"));
    assert_eq!(
        menu.recipe_unlock_events(),
        &["minecraft:oak_planks"],
        "duplicate acquisition must not emit another recipe-book add event"
    );
    assert!(!menu.unlock_recipe("minecraft:missing"));
}

#[test]
fn inventory_menu_quick_move_uses_vanilla_inventory_zones() {
    let mut menu = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());
    menu.set_slot(1, ItemStack::new("minecraft:oak_log", 1));

    let crafted = menu.quick_move(0);
    assert_eq!(crafted.item_id(), "minecraft:oak_planks");
    assert_eq!(crafted.count(), 4);
    assert_eq!(menu.get_slot(44).unwrap().item_id(), "minecraft:oak_planks");
    assert!(menu.get_slot(1).unwrap().is_empty());
    assert!(menu.get_slot(0).unwrap().is_empty());

    for slot in 36..45 {
        menu.set_slot(slot, ItemStack::new("minecraft:cobblestone", 64));
    }
    menu.set_slot(36, ItemStack::new("minecraft:stick", 8));
    let hotbar = menu.quick_move(36);
    assert_eq!(hotbar.item_id(), "minecraft:stick");
    assert!(menu.get_slot(36).unwrap().is_empty());
    assert_eq!(menu.get_slot(9).unwrap().item_id(), "minecraft:stick");

    menu.set_slot(9, ItemStack::new("minecraft:apple", 3));
    let storage = menu.quick_move(9);
    assert_eq!(storage.item_id(), "minecraft:apple");
    assert_eq!(menu.get_slot(36).unwrap().item_id(), "minecraft:apple");

    menu.set_slot(11, ItemStack::new("minecraft:shield", 1));
    let shield = menu.quick_move(11);
    assert_eq!(shield.item_id(), "minecraft:shield");
    assert_eq!(menu.get_slot(45).unwrap().item_id(), "minecraft:shield");

    menu.set_slot(10, ItemStack::new("minecraft:leather_boots", 1));
    let boots = menu.quick_move(10);
    assert_eq!(boots.item_id(), "minecraft:leather_boots");
    assert_eq!(
        menu.get_slot(8).unwrap().item_id(),
        "minecraft:leather_boots"
    );

    let mut full = InventoryMenu::new(PlayerInventory::new(), crafting_test_recipes());
    full.set_slot(1, ItemStack::new("minecraft:oak_log", 1));
    for slot in 9..45 {
        full.set_slot(slot, ItemStack::new("minecraft:cobblestone", 64));
    }
    assert!(full.quick_move(0).is_empty());
    assert_eq!(full.get_slot(1).unwrap().item_id(), "minecraft:oak_log");
    assert_eq!(full.get_slot(0).unwrap().item_id(), "minecraft:oak_planks");
}

#[test]
fn death_drops_clears_all_slots_except_prevent_equipment_drop_and_respects_keep_inventory() {
    let mut inv = PlayerInventory::new();
    inv.set(0, ItemStack::new("minecraft:apple", 4));
    inv.set(1, ItemStack::new("minecraft:diamond_sword", 1)); // "has curse of vanishing"
    inv.set(36, ItemStack::new("minecraft:leather_boots", 1)); // feet armor slot

    // With keepInventory: nothing drops, nothing destroyed.
    let drops = inv.death_drops(true, |_| false);
    assert!(drops.is_empty());
    assert!(!inv.get(0).is_empty());

    // Without keepInventory: cursed item destroyed, others dropped.
    let curse_id = "minecraft:diamond_sword";
    let drops = inv.death_drops(false, |s| s.item_id() == curse_id);
    let dropped_ids: Vec<_> = drops.iter().map(|s| s.item_id()).collect();
    assert!(dropped_ids.contains(&"minecraft:apple"));
    assert!(!dropped_ids.contains(&curse_id)); // destroyed, not dropped
    assert!(dropped_ids.contains(&"minecraft:leather_boots"));
    // All slots now empty.
    for slot in 0..43 {
        assert!(
            inv.get(slot).is_empty(),
            "slot {slot} should be empty after death"
        );
    }
}
