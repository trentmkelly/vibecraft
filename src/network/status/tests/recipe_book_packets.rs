use super::super::*;
use super::*;

fn oak_planks_recipe_map() -> RecipeMap {
    RecipeMap::create(vec![crate::recipe_system::RecipeHolder {
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
    }])
}

fn display_shifted_oak_planks_recipe_map() -> RecipeMap {
    let mut recipes = vec![crate::recipe_system::RecipeHolder {
        id: "minecraft:repair_item",
        recipe: crate::recipe_system::RecipeKind::Special {
            kind: crate::recipe_system::SpecialRecipeKind::RepairItem,
            result_hint: None,
        },
    }];
    recipes.extend(oak_planks_recipe_map().values().iter().cloned());
    RecipeMap::create(recipes)
}

fn vanilla_recipe_manager() -> RecipeManagerModel {
    let recipe_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    load_recipe_directory(&recipe_dir).expect("vanilla recipe directory should load")
}

#[cfg_attr(
    not(vibecraft_has_decompiled_sources),
    ignore = "requires optional Java source root"
)]
#[test]
pub fn recipebook_java_source_sentinels_cover_place_recipe_parity_rules() {
    const PLACE_RECIPE_HELPER_JAVA: &str = vibecraft_java_source!("/net/minecraft/recipebook/PlaceRecipeHelper.java");
    const SERVER_PLACE_RECIPE_JAVA: &str = vibecraft_java_source!("/net/minecraft/recipebook/ServerPlaceRecipe.java");

    for sentinel in [
        "placeRecipe(gridWidth, gridHeight, shapedRecipe.getWidth(), shapedRecipe.getHeight(), entries, output);",
        "boolean shouldCenterRecipe = recipeHeight < gridHeight / 2.0F;",
        "int startPosCenterRecipe = Mth.floor(gridHeight / 2.0F - recipeHeight / 2.0F);",
        "void addItemToSlot(T item, int gridIndex, int gridXPos, int gridYPos);",
    ] {
        assert!(
            PLACE_RECIPE_HELPER_JAVA.contains(sentinel),
            "missing Java PlaceRecipeHelper sentinel {sentinel}"
        );
    }
    for sentinel in [
        "if (!allowDroppingItemsToClear && !placer.testClearGrid())",
        "boolean recipeMatchesPlaced = this.menu.recipeMatches(recipe);",
        "int amountToCraft = this.calculateAmountToCraft(biggestCraftableStack, recipeMatchesPlaced);",
        "smallestStackSize++;",
        "this.clearGrid();",
        "this.inventory.placeItemBackInInventory(itemStackCopy, false);",
        "PlaceRecipeHelper.placeRecipe(",
        "this.inventory.findSlotMatchingCraftingIngredient(itemInInventory, itemInTargetSlot);",
    ] {
        assert!(
            SERVER_PLACE_RECIPE_JAVA.contains(sentinel),
            "missing Java ServerPlaceRecipe sentinel {sentinel}"
        );
    }
}

#[test]
pub fn picking_up_oak_log_unlocks_and_sends_oak_planks_recipe() {
    let recipe_manager = vanilla_recipe_manager();
    let recipes = recipe_manager.recipe_map().clone();
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes);

    let world_items = Arc::new(Mutex::new(WorldItemEntities::restore(
        vec![DroppedItem {
            entity_id: 2,
            item: "minecraft:oak_log",
            count: 1,
            x: state.x,
            y: state.y,
            z: state.z,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            pickup_delay: 0,
            age: 0,
            target_uuid: None,
        }],
        2,
    )));

    let (mut server, mut client) = loopback_pair();
    process_item_pickups(
        &mut server,
        CompressionState::disabled(),
        &mut state,
        "00000000-0000-0000-0000-000000000000",
        &world_items,
        &recipe_manager,
    )
    .expect("pickup processing should write recipe unlock packets");
    drop(server);
    client
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();

    let known_recipes = state.inventory_menu.recipe_book_known_recipes();
    let highlighted_recipes = state.inventory_menu.recipe_book_highlighted_recipes();
    assert!(
        known_recipes.contains(&"minecraft:oak_planks"),
        "oak log pickup should unlock oak_planks; known recipes were {known_recipes:?}"
    );
    assert!(
        highlighted_recipes.contains(&"minecraft:oak_planks"),
        "oak log pickup should highlight oak_planks; highlighted recipes were {highlighted_recipes:?}"
    );
    assert!(world_items.lock().unwrap().entities.is_empty());

    let mut packet_ids = Vec::new();
    while let Ok(frame) = read_packet(&mut client) {
        let mut payload = &frame[..];
        packet_ids.push(read_var_i32(&mut payload).unwrap());
    }
    assert!(
        packet_ids.contains(&CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID),
        "oak log pickup should send recipe_book_add for oak_planks; saw {packet_ids:?}"
    );
}

#[test]
pub fn live_join_sends_update_recipes_after_held_slot_like_java() {
    let recipe_manager = vanilla_recipe_manager();
    let recipes = recipe_manager.recipe_map().clone();
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes);
    let properties = test_properties();
    let profile = crate::player_access::NameAndId {
        uuid: "00000000-0000-0000-0000-000000000123".to_string(),
        name: "RecipeJoinBot".to_string(),
    };
    let world_root = std::env::temp_dir().join("vibecraft-recipe-join-test");
    let (mut server, mut client) = loopback_pair();

    write_minimal_play_join(
        &mut server,
        CompressionState::disabled(),
        MinimalPlayJoinContext {
            properties: &properties,
            world_seed: 0,
            profile: &profile,
            play_state: &state,
            recipe_manager: &recipe_manager,
            world_root: &world_root,
            clock_game_time: 0,
            clock_data: &[],
            rain_level: 0.0,
            thunder_level: 0.0,
        },
    )
    .expect("join writer should emit play packets");
    drop(server);

    client
        .set_read_timeout(Some(std::time::Duration::from_millis(50)))
        .unwrap();
    let mut packet_ids = Vec::new();
    while let Ok(frame) = read_packet(&mut client) {
        let mut payload = &frame[..];
        packet_ids.push(read_var_i32(&mut payload).unwrap());
    }

    let held_slot = packet_ids
        .iter()
        .position(|id| *id == CLIENTBOUND_SET_HELD_SLOT_PACKET_ID)
        .expect("join should send held-slot packet");
    assert_eq!(
        packet_ids.get(held_slot + 1),
        Some(&crate::network::play::CLIENTBOUND_UPDATE_RECIPES_PACKET_ID),
        "Java PlayerList sends update_recipes immediately after held-slot; saw {packet_ids:?}"
    );
    assert!(
        packet_ids
            .iter()
            .position(|id| *id == CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID)
            .is_some_and(|settings| settings > held_slot + 1),
        "recipe-book settings must follow update_recipes; saw {packet_ids:?}"
    );
}

#[test]
pub fn play_session_state_nbt_round_trip_preserves_recipe_book_state() {
    let recipes = oak_planks_recipe_map();
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], ["minecraft:oak_planks"]);
    super::super::apply_recipe_book_settings_packet(
        &mut state,
        crate::network::play::ServerboundRecipeBookChangeSettingsPacket {
            book_type: crate::network::play::RecipeBookType::Crafting,
            is_open: true,
            is_filtering: true,
        },
    );

    let tag = play_session_state_to_nbt(&state);
    let restored = play_session_state_from_nbt(&tag, GameMode::Survival, &recipes).unwrap();

    assert_eq!(
        restored.inventory_menu.recipe_book_known_recipes(),
        vec!["minecraft:oak_planks"]
    );
    assert_eq!(
        restored.inventory_menu.recipe_book_highlighted_recipes(),
        vec!["minecraft:oak_planks"]
    );
    assert!(restored.recipe_book_settings.crafting.open);
    assert!(restored.recipe_book_settings.crafting.filtering);
}

#[test]
pub fn recipe_book_seen_recipe_packet_clears_highlight_for_display_id() {
    let recipes = oak_planks_recipe_map();
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(PlayerInventory::new(), recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], ["minecraft:oak_planks"]);

    super::super::apply_recipe_book_seen_recipe_packet(
        &mut state,
        crate::network::play::ServerboundRecipeBookSeenRecipePacket { recipe_index: 0 },
        &recipes,
    );

    assert_eq!(
        state.inventory_menu.recipe_book_known_recipes(),
        vec!["minecraft:oak_planks"]
    );
    assert!(state
        .inventory_menu
        .recipe_book_highlighted_recipes()
        .is_empty());

    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], ["minecraft:oak_planks"]);

    for recipe_index in [-1, 1] {
        super::super::apply_recipe_book_seen_recipe_packet(
            &mut state,
            crate::network::play::ServerboundRecipeBookSeenRecipePacket { recipe_index },
            &recipes,
        );
    }

    assert_eq!(
        state.inventory_menu.recipe_book_highlighted_recipes(),
        vec!["minecraft:oak_planks"]
    );
}

#[test]
pub fn place_recipe_packet_moves_unlocked_recipe_ingredients_into_inventory_grid() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    assert_eq!(
        state.inventory_menu.recipe_book_type(),
        crate::recipe_system::RecipeBookType::Crafting
    );
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], []);

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(changed);
    assert_eq!(state.container_state_id, 1);
    assert_eq!(
        state.inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(
        state.inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4))
    );
    assert_eq!(
        state.inventory_menu.player_inventory().get(0),
        &ItemStack::new("minecraft:oak_log", 2)
    );
}

#[test]
pub fn place_recipe_packet_increments_matching_grid_like_java_server_place_recipe() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 2))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], []);
    assert!(state
        .inventory_menu
        .set_slot(1, ItemStack::new("minecraft:oak_log", 1)));

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(changed);
    assert_eq!(
        state.inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 2)),
        "Java ServerPlaceRecipe increments an already-matching grid by one craft"
    );
    let remaining_log = state.inventory_menu.player_inventory().get(0);
    assert_eq!(remaining_log.item_id(), "minecraft:oak_log");
    assert_eq!(remaining_log.count(), 1);
}

#[test]
pub fn place_recipe_packet_refuses_matching_grid_that_cannot_grow_like_java() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 64))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], []);
    assert!(state
        .inventory_menu
        .set_slot(1, ItemStack::new("minecraft:oak_log", 64)));

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(!changed);
    assert_eq!(
        state.inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 64))
    );
    assert_eq!(
        state.inventory_menu.player_inventory().get(0),
        &ItemStack::new("minecraft:oak_log", 64)
    );
}

#[test]
pub fn place_recipe_packet_refuses_to_clear_grid_when_inventory_is_full_like_java() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    for slot in 0..INVENTORY_SIZE {
        inventory.set(slot, ItemStack::new("minecraft:cobblestone", 64));
    }
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], []);
    assert!(state
        .inventory_menu
        .set_slot(1, ItemStack::new("minecraft:oak_log", 1)));

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(!changed);
    assert_eq!(state.container_state_id, 0);
    assert_eq!(
        state.inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(
        state.inventory_menu.player_inventory().get(0),
        &ItemStack::new("minecraft:cobblestone", 64)
    );
    assert!(state.inventory_menu.player_inventory().dropped().is_empty());
}

#[test]
pub fn place_recipe_packet_rejects_wrong_container_id_without_mutating_inventory() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    state
        .inventory_menu
        .load_recipe_book(["minecraft:oak_planks"], []);

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 7,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(!changed);
    assert_eq!(state.container_state_id, 0);
    assert_eq!(state.inventory_menu.get_slot(0), Some(ItemStack::empty()));
    assert_eq!(state.inventory_menu.get_slot(1), Some(ItemStack::empty()));
    assert_eq!(
        state.inventory_menu.player_inventory().get(0),
        &ItemStack::new("minecraft:oak_log", 3)
    );
}

#[test]
pub fn place_recipe_packet_rejects_locked_recipe_without_mutating_inventory() {
    let recipes = oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(!changed);
    assert_eq!(state.container_state_id, 0);
    assert_eq!(state.inventory_menu.get_slot(1), Some(ItemStack::empty()));
    assert_eq!(
        state.inventory_menu.player_inventory().get(0),
        &ItemStack::new("minecraft:oak_log", 3)
    );
}

#[test]
pub fn place_recipe_packet_uses_java_display_id_not_raw_recipe_index() {
    let recipes = display_shifted_oak_planks_recipe_map();
    let mut inventory = PlayerInventory::new();
    inventory.load_items(&[(0, ItemStack::new("minecraft:oak_log", 3))]);
    let mut state = session_state_with_inventory(&[]);
    state.inventory_menu = InventoryMenu::new(inventory, recipes.clone());
    state.inventory_menu.load_recipe_book(["minecraft:oak_planks"], []);

    let changed = super::super::apply_place_recipe_packet(
        &mut state,
        crate::network::play::ServerboundPlaceRecipePacket {
            container_id: 0,
            recipe_index: 0,
            use_max_items: false,
        },
        &recipes,
    );

    assert!(changed);
    assert_eq!(
        state.inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4)),
        "Java RecipeManager display ids are compact over synchronized recipe displays"
    );
    assert_eq!(
        state.inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
}
