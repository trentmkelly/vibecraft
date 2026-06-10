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

#[test]
pub fn picking_up_oak_log_unlocks_and_sends_oak_planks_recipe() {
    let recipe_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/recipe");
    let recipe_manager =
        load_recipe_directory(&recipe_dir).expect("vanilla recipe directory should load");
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
