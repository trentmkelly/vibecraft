    use super::*;
    use crate::recipe_system::{
        CraftingBookCategoryModel, IngredientSpec, ItemAmount, RecipeHolder, RecipeKind, RecipeMap,
    };

    #[test]
    fn active_chest_menu_click_moves_inventory_item_into_persisted_block_entity_slot() {
        let pos = crate::block_update::BlockPos { x: 1, y: 64, z: 2 };
        let mut state = PlaySessionState::default();
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:stone", 16));
        let mut menu = ActiveBlockMenu {
            container_id: 7,
            state_id: 0,
            pos,
            kind: ActiveBlockMenuKind::Persistent {
                block_entity_id: "minecraft:chest",
                result_slot: None,
            },
            slots: vec![ItemStack::empty(); 27],
        };
        let world_root =
            std::env::temp_dir().join(format!("vibecraft-active-menu-{}", std::process::id()));
        let layout = WorldLayout::new(&world_root);
        let cache = GeneratedChunkCache::default();

        let pickup = ServerboundContainerClickPacket {
            container_id: 7,
            state_id: 0,
            slot_num: (27 + 27) as i16,
            button_num: 0,
            container_input: ContainerInput::Pickup,
            changed_slots: BTreeMap::new(),
            carried_item: crate::network::play::HashedStack::empty(),
        };
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        let place = ServerboundContainerClickPacket {
            state_id: 1,
            slot_num: 0,
            ..pickup
        };
        menu.handle_click(&place, &mut state, &layout, 0, &cache);

        assert_eq!(menu.slots[0], ItemStack::new("minecraft:stone", 16));
        assert!(state.inventory_menu.player_inventory().get(0).is_empty());
        let tag = block_entity_tag("minecraft:chest", pos, &menu.slots, None);
        let Tag::Compound(entries) = tag else {
            panic!("block entity tag should be compound");
        };
        assert!(matches!(
            entries.iter().find(|(name, _)| name == "Items"),
            Some((_, Tag::List(items))) if items.len() == 1
        ));
    }

    #[test]
    fn closing_ephemeral_menu_returns_input_slots_to_player_inventory() {
        let mut state = PlaySessionState::default();
        let menu = ActiveBlockMenu {
            container_id: 3,
            state_id: 0,
            pos: crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            kind: ActiveBlockMenuKind::Ephemeral {
                result_slot: Some(1),
            },
            slots: vec![ItemStack::new("minecraft:stone", 4), ItemStack::empty()],
        };
        menu.close(&mut state);

        assert_eq!(
            state.inventory_menu.player_inventory().get(0).item_id(),
            "minecraft:stone"
        );
        assert_eq!(
            state.inventory_menu.player_inventory().get(0).count(),
            4
        );
    }

    #[test]
    fn active_crafting_table_initial_sync_increments_state_id_like_java() {
        let recipes = vanilla_recipe_map();
        let state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let mut bytes = Vec::new();

        menu.write_full_content(&mut bytes, CompressionState::disabled(), &state)
            .unwrap();

        assert_eq!(menu.state_id(), 1);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn active_crafting_table_clicks_update_result_consume_inputs_and_unlock_recipe() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let mut state_id = 0;
        let pickup = click_packet(9, state_id, CraftingMenu::HOTBAR_START as i16, 0, ContainerInput::Pickup);
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        state_id += 1;
        assert_eq!(state.carried_item, ItemStack::new("minecraft:oak_planks", 4));

        let mut last_instructions = Vec::new();
        for slot in [1_i16, 2, 4, 5] {
            let place_one = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            last_instructions = menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
            state_id += 1;
        }
        assert!(state.carried_item.is_empty());
        assert!(last_instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == CraftingMenu::RESULT_SLOT as i16
                        && packet.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
                        && packet.item_stack.count == 1
            )
        }));

        let take_result = click_packet(9, state_id, CraftingMenu::RESULT_SLOT as i16, 0, ContainerInput::Pickup);
        let take_instructions = menu.handle_click(&take_result, &mut state, &layout, 0, &cache);
        assert_eq!(state.carried_item, ItemStack::new("minecraft:crafting_table", 1));
        assert!(take_instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::RecipesUnlocked(ids)
                    if ids == &vec!["minecraft:crafting_table"]
            )
        }));
        assert!(
            state
                .inventory_menu
                .recipe_book_known_recipes()
                .contains(&"minecraft:crafting_table"),
            "crafting-table ResultSlot awards recipe knowledge to the player recipe book"
        );
        assert!(menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT].is_empty());
        for slot in [1_usize, 2, 4, 5] {
            assert!(menu.flattened_slots(&state)[slot].is_empty());
        }
    }

    #[test]
    fn active_crafting_table_result_slot_is_fake_like_java_for_secondary_pickup() {
        let recipes = vanilla_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let ActiveBlockMenuKind::Crafting { menu: crafting } = &mut menu.kind else {
            panic!("expected crafting menu");
        };
        crafting.set_slot(
            1,
            ItemStack::new("minecraft:oak_log", 1),
            state.inventory_menu.player_inventory_mut(),
        );
        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:oak_planks", 4)
        );

        let instructions = menu.handle_click(
            &click_packet(
                9,
                0,
                CraftingMenu::RESULT_SLOT as i16,
                1,
                ContainerInput::Pickup,
            ),
            &mut state,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
        );

        assert_eq!(state.carried_item, ItemStack::new("minecraft:oak_planks", 4));
        assert!(menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT].is_empty());
        assert!(menu.flattened_slots(&state)[CraftingMenu::GRID_START].is_empty());
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::RecipesUnlocked(ids) if ids == &vec!["minecraft:oak_planks"]
            )
        }));
    }

    #[test]
    fn active_crafting_table_live_client_slot_shadows_update_three_by_three_result() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            carried_item: ItemStack::new("minecraft:oak_planks", 4),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let mut instructions = Vec::new();
        for (state_id, slot, carried_count) in [(0, 1, 3), (1, 2, 2), (2, 4, 1), (3, 5, 0)] {
            let mut packet = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            packet.changed_slots.insert(
                slot as i32,
                hashed_stack("minecraft:oak_planks", 1),
            );
            packet.carried_item = hashed_stack("minecraft:oak_planks", carried_count);
            instructions = menu.handle_click(&packet, &mut state, &layout, 0, &cache);
        }

        assert!(state.carried_item.is_empty());
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == CraftingMenu::RESULT_SLOT as i16
                        && packet.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
                        && packet.item_stack.count == 1
            )
        }));
    }

    #[test]
    fn active_crafting_table_uses_authoritative_stacks_not_client_hash_shadows() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            carried_item: ItemStack::new("minecraft:oak_planks", 4),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let mut instructions = Vec::new();
        for (state_id, slot) in [(0, 1_i16), (1, 2), (2, 4), (3, 5)] {
            let mut packet = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            // Java stores these as remote HashedStack shadows after applying the
            // authoritative click; they must not become the real menu contents.
            packet
                .changed_slots
                .insert(slot as i32, hashed_stack("minecraft:apple", 64));
            packet.carried_item = hashed_stack("minecraft:diamond", 64);
            instructions = menu.handle_click(&packet, &mut state, &layout, 0, &cache);
        }

        assert!(state.carried_item.is_empty());
        for slot in [1_usize, 2, 4, 5] {
            assert_eq!(
                menu.flattened_slots(&state)[slot],
                ItemStack::new("minecraft:oak_planks", 1)
            );
        }
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == CraftingMenu::RESULT_SLOT as i16
                        && packet.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
                        && packet.item_stack.count == 1
            )
        }));
    }

    #[test]
    fn active_menu_click_applies_before_correcting_mismatched_client_shadows_like_java() {
        let pos = crate::block_update::BlockPos { x: 1, y: 64, z: 2 };
        let mut state = PlaySessionState {
            carried_item: ItemStack::new("minecraft:stone", 16),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu {
            container_id: 7,
            state_id: 0,
            pos,
            kind: ActiveBlockMenuKind::Persistent {
                block_entity_id: "minecraft:chest",
                result_slot: None,
            },
            slots: vec![ItemStack::empty(); 27],
        };
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();
        let mut packet = click_packet(7, 0, 0, 0, ContainerInput::Pickup);
        packet.changed_slots.insert(0, hashed_stack("minecraft:apple", 1));
        packet.carried_item = hashed_stack("minecraft:stone", 16);

        let instructions = menu.handle_click(&packet, &mut state, &layout, 0, &cache);

        assert_eq!(menu.slots[0], ItemStack::new("minecraft:stone", 16));
        assert!(state.carried_item.is_empty());
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == 0
                        && packet.item_stack.item_id == item_protocol_id("minecraft:stone")
                        && packet.item_stack.count == 16
            )
        }));
    }

    #[test]
    fn active_crafting_table_known_recipe_take_does_not_resend_recipe_unlock() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        assert!(state.inventory_menu.unlock_recipe("minecraft:crafting_table"));
        let _ = state.inventory_menu.drain_recipe_unlock_events();
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let mut state_id = 0;
        menu.handle_click(
            &click_packet(
                9,
                state_id,
                CraftingMenu::HOTBAR_START as i16,
                0,
                ContainerInput::Pickup,
            ),
            &mut state,
            &layout,
            0,
            &cache,
        );
        state_id += 1;
        for slot in [1_i16, 2, 4, 5] {
            menu.handle_click(
                &click_packet(9, state_id, slot, 1, ContainerInput::Pickup),
                &mut state,
                &layout,
                0,
                &cache,
            );
            state_id += 1;
        }

        let instructions = menu.handle_click(
            &click_packet(
                9,
                state_id,
                CraftingMenu::RESULT_SLOT as i16,
                0,
                ContainerInput::Pickup,
            ),
            &mut state,
            &layout,
            0,
            &cache,
        );

        assert_eq!(
            state.inventory_menu.recipe_book_known_recipes(),
            vec!["minecraft:crafting_table"]
        );
        assert!(
            instructions
                .iter()
                .all(|instruction| !matches!(instruction, PlayInstruction::RecipesUnlocked(_))),
            "Java ServerRecipeBook.add only sends newly-known recipes"
        );
    }

    #[test]
    fn active_crafting_table_stale_grid_click_applies_then_full_resyncs_like_java() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let pickup = click_packet(
            9,
            0,
            CraftingMenu::HOTBAR_START as i16,
            0,
            ContainerInput::Pickup,
        );
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        for (expected_state_id, slot) in [(1, 1_i16), (2, 2), (3, 4)] {
            let place_one = click_packet(9, expected_state_id, slot, 1, ContainerInput::Pickup);
            menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
        }

        let stale_place = click_packet(9, 3, 5, 1, ContainerInput::Pickup);
        let instructions = menu.handle_click(&stale_place, &mut state, &layout, 0, &cache);

        assert_eq!(menu.state_id(), 5);
        assert!(state.carried_item.is_empty());
        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );
        for slot in [1_usize, 2, 4, 5] {
            assert_eq!(
                menu.flattened_slots(&state)[slot],
                ItemStack::new("minecraft:oak_planks", 1)
            );
        }
        let full_sync = instructions
            .iter()
            .find_map(|instruction| match instruction {
                PlayInstruction::Container(packet) => Some(packet),
                _ => None,
            })
            .expect("Java broadcastFullState sends one ContainerSetContent packet");
        assert_eq!(full_sync.container_id, 9);
        assert_eq!(full_sync.state_id, 5);
        assert_eq!(full_sync.slots.len(), CraftingMenu::SLOT_COUNT);
        assert_eq!(
            full_sync.slots[CraftingMenu::RESULT_SLOT].item_id,
            item_protocol_id("minecraft:crafting_table")
        );
        assert_eq!(full_sync.slots[CraftingMenu::RESULT_SLOT].count, 1);
        assert!(full_sync.carried_item.item_id.is_none());
    }

    #[test]
    fn active_crafting_table_shift_click_result_uses_java_quick_move_semantics() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();

        let mut state_id = 0;
        let pickup = click_packet(
            9,
            state_id,
            CraftingMenu::HOTBAR_START as i16,
            0,
            ContainerInput::Pickup,
        );
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        state_id += 1;
        for slot in [1_i16, 2, 4, 5] {
            let place_one = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
            state_id += 1;
        }
        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );

        let quick_move_result = click_packet(
            9,
            state_id,
            CraftingMenu::RESULT_SLOT as i16,
            0,
            ContainerInput::QuickMove,
        );
        let instructions = menu.handle_click(&quick_move_result, &mut state, &layout, 0, &cache);

        assert!(state.carried_item.is_empty());
        assert!(menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT].is_empty());
        for slot in CraftingMenu::GRID_START..CraftingMenu::GRID_END {
            assert!(
                menu.flattened_slots(&state)[slot].is_empty(),
                "crafted result must not be inserted into crafting-grid slot {slot}"
            );
        }
        assert_eq!(
            state.inventory_menu.player_inventory().get(8),
            &ItemStack::new("minecraft:crafting_table", 1)
        );
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::RecipesUnlocked(ids)
                    if ids == &vec!["minecraft:crafting_table"]
            )
        }));
        assert!(
            state
                .inventory_menu
                .recipe_book_known_recipes()
                .contains(&"minecraft:crafting_table")
        );
    }

    #[test]
    fn active_crafting_table_recipe_book_places_known_recipe_into_grid() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        assert!(state.inventory_menu.unlock_recipe("minecraft:crafting_table"));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );

        assert!(menu.handle_place_recipe(
            &ServerboundPlaceRecipePacket {
                container_id: 9,
                recipe_index: 0,
                use_max_items: false,
            },
            &mut state,
            &recipes,
        ));

        assert_eq!(
            menu.state_id(),
            0,
            "ServerboundPlaceRecipe mutates the CraftingMenu; the subsequent content broadcast owns the state increment like Java CraftingMenu.finishPlacingRecipe"
        );
        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );
        for slot in [1_usize, 2, 4, 5] {
            assert_eq!(
                menu.flattened_slots(&state)[slot],
                ItemStack::new("minecraft:oak_planks", 1)
            );
        }
        assert!(state.inventory_menu.player_inventory().get(0).is_empty());
    }

    #[test]
    fn active_crafting_table_place_recipe_uses_java_display_id_not_raw_recipe_index() {
        let recipes = display_shifted_crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        assert!(state.inventory_menu.unlock_recipe("minecraft:crafting_table"));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );

        assert!(menu.handle_place_recipe(
            &ServerboundPlaceRecipePacket {
                container_id: 9,
                recipe_index: 0,
                use_max_items: false,
            },
            &mut state,
            &recipes,
        ));

        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1),
            "Java RecipeManager indexes ServerboundPlaceRecipe by display table; special recipes without synchronized displays do not consume ids"
        );
    }

    #[test]
    fn active_crafting_table_recipe_book_full_sync_uses_single_java_state_increment() {
        let recipes = crafting_table_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        assert!(state.inventory_menu.unlock_recipe("minecraft:crafting_table"));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );

        let mut initial_sync = Vec::new();
        menu.write_full_content(&mut initial_sync, CompressionState::disabled(), &state)
            .unwrap();
        assert_eq!(menu.state_id(), 1);

        assert!(menu.handle_place_recipe(
            &ServerboundPlaceRecipePacket {
                container_id: 9,
                recipe_index: 0,
                use_max_items: false,
            },
            &mut state,
            &recipes,
        ));
        assert_eq!(
            menu.state_id(),
            1,
            "placing the recipe must not skip the next menu state id before broadcasting"
        );

        let mut placement_sync = Vec::new();
        menu.write_full_content(&mut placement_sync, CompressionState::disabled(), &state)
            .unwrap();
        assert_eq!(menu.state_id(), 2);
        assert!(!placement_sync.is_empty());
        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );
    }

    #[test]
    fn active_crafting_table_matches_real_vanilla_three_by_three_recipe() {
        let recipes = vanilla_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 3));
        state
            .inventory_menu
            .player_inventory_mut()
            .set(1, ItemStack::new("minecraft:stick", 2));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();
        let mut bytes = Vec::new();
        menu.write_full_content(&mut bytes, CompressionState::disabled(), &state)
            .unwrap();

        let mut state_id = menu.state_id();
        let pickup_planks = click_packet(
            9,
            state_id,
            CraftingMenu::HOTBAR_START as i16,
            0,
            ContainerInput::Pickup,
        );
        menu.handle_click(&pickup_planks, &mut state, &layout, 0, &cache);
        state_id += 1;
        for slot in [1_i16, 2, 3] {
            let place_one = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
            state_id += 1;
        }
        let pickup_sticks = click_packet(
            9,
            state_id,
            (CraftingMenu::HOTBAR_START + 1) as i16,
            0,
            ContainerInput::Pickup,
        );
        menu.handle_click(&pickup_sticks, &mut state, &layout, 0, &cache);
        state_id += 1;
        let mut last_instructions = Vec::new();
        for slot in [5_i16, 8] {
            let place_one = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            last_instructions = menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
            state_id += 1;
        }

        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:wooden_pickaxe", 1)
        );
        assert!(last_instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == CraftingMenu::RESULT_SLOT as i16
                        && packet.item_stack.item_id == item_protocol_id("minecraft:wooden_pickaxe")
                        && packet.item_stack.count == 1
            )
        }));
        let take_result = click_packet(
            9,
            state_id,
            CraftingMenu::RESULT_SLOT as i16,
            0,
            ContainerInput::Pickup,
        );
        let instructions = menu.handle_click(&take_result, &mut state, &layout, 0, &cache);
        assert_eq!(state.carried_item, ItemStack::new("minecraft:wooden_pickaxe", 1));
        assert!(instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::RecipesUnlocked(ids)
                    if ids == &vec!["minecraft:wooden_pickaxe"]
            )
        }));
    }

    #[test]
    fn active_crafting_table_matches_vanilla_recipe_away_from_top_left() {
        let recipes = vanilla_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let ActiveBlockMenuKind::Crafting { menu: crafting } = &mut menu.kind else {
            panic!("expected crafting menu");
        };
        let player = state.inventory_menu.player_inventory_mut();
        for slot in [5_usize, 6, 8, 9] {
            assert!(crafting.set_slot(slot, ItemStack::new("minecraft:oak_planks", 1), player));
        }

        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );
    }

    #[test]
    fn active_crafting_table_clicks_vanilla_planks_after_initial_content_sync() {
        let recipes = vanilla_recipe_map();
        let mut state = PlaySessionState {
            inventory_menu: InventoryMenu::new(PlayerInventory::new(), recipes.clone()),
            ..Default::default()
        };
        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:oak_planks", 4));
        let mut menu = ActiveBlockMenu::open(
            9,
            crate::block_update::BlockPos { x: 0, y: 64, z: 0 },
            LiveBlockMenuKind::Crafting,
            &WorldLayout::new(std::env::temp_dir()),
            0,
            &GeneratedChunkCache::default(),
            &recipes,
        );
        let layout = WorldLayout::new(std::env::temp_dir());
        let cache = GeneratedChunkCache::default();
        menu.write_full_content(&mut Vec::new(), CompressionState::disabled(), &state)
            .unwrap();

        let mut state_id = menu.state_id();
        let pickup = click_packet(
            9,
            state_id,
            CraftingMenu::HOTBAR_START as i16,
            0,
            ContainerInput::Pickup,
        );
        menu.handle_click(&pickup, &mut state, &layout, 0, &cache);
        state_id += 1;

        let mut last_instructions = Vec::new();
        for slot in [1_i16, 2, 4, 5] {
            let place_one = click_packet(9, state_id, slot, 1, ContainerInput::Pickup);
            last_instructions = menu.handle_click(&place_one, &mut state, &layout, 0, &cache);
            state_id += 1;
        }

        assert_eq!(
            menu.flattened_slots(&state)[CraftingMenu::RESULT_SLOT],
            ItemStack::new("minecraft:crafting_table", 1)
        );
        assert!(last_instructions.iter().any(|instruction| {
            matches!(
                instruction,
                PlayInstruction::ContainerSetSlot(packet)
                    if packet.slot == CraftingMenu::RESULT_SLOT as i16
                        && packet.item_stack.item_id == item_protocol_id("minecraft:crafting_table")
                        && packet.item_stack.count == 1
            )
        }));
    }

    fn vanilla_recipe_map() -> RecipeMap {
        let recipe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla-data")
            .join("data")
            .join("minecraft")
            .join("recipe");
        crate::recipe_system::load_recipe_directory(&recipe_dir)
            .unwrap_or_else(|err| panic!("failed to load vanilla recipes: {err}"))
            .recipe_map()
            .clone()
    }

    fn crafting_table_recipe_map() -> RecipeMap {
        RecipeMap::create(vec![RecipeHolder {
            id: "minecraft:crafting_table",
            recipe: RecipeKind::Shaped {
                width: 2,
                height: 2,
                pattern: vec![
                    Some(IngredientSpec::Item("minecraft:oak_planks")),
                    Some(IngredientSpec::Item("minecraft:oak_planks")),
                    Some(IngredientSpec::Item("minecraft:oak_planks")),
                    Some(IngredientSpec::Item("minecraft:oak_planks")),
                ],
                result: ItemAmount::one("minecraft:crafting_table"),
                category: CraftingBookCategoryModel::Misc,
            },
        }])
    }

    fn display_shifted_crafting_table_recipe_map() -> RecipeMap {
        let mut recipes = vec![RecipeHolder {
            id: "minecraft:repair_item",
            recipe: RecipeKind::Special {
                kind: crate::recipe_system::SpecialRecipeKind::RepairItem,
                result_hint: None,
            },
        }];
        recipes.extend(crafting_table_recipe_map().values().iter().cloned());
        RecipeMap::create(recipes)
    }

    fn click_packet(
        container_id: i32,
        state_id: i32,
        slot_num: i16,
        button_num: i8,
        container_input: ContainerInput,
    ) -> ServerboundContainerClickPacket {
        ServerboundContainerClickPacket {
            container_id,
            state_id,
            slot_num,
            button_num,
            container_input,
            changed_slots: BTreeMap::new(),
            carried_item: crate::network::play::HashedStack::empty(),
        }
    }

    fn hashed_stack(item: &str, count: i32) -> crate::network::play::HashedStack {
        if count <= 0 {
            return crate::network::play::HashedStack::empty();
        }
        crate::network::play::HashedStack {
            item_id: item_protocol_id(item),
            count,
            components: crate::network::play::HashedPatchMap::empty(),
        }
    }
