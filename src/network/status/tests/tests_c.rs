use super::super::*;
use super::*;

pub fn oak_planks_recipe_map() -> RecipeMap {
    RecipeMap::create(vec![crate::recipe_system::RecipeHolder {
        id: "minecraft:oak_planks",
        recipe: crate::recipe_system::RecipeKind::Shapeless {
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
    assert!(
        state
            .inventory_menu
            .recipe_book_highlighted_recipes()
            .is_empty()
    );
}

#[test]
pub fn place_recipe_packet_moves_unlocked_recipe_ingredients_into_inventory_grid() {
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
pub fn movement_packets_accumulate_and_apply_fall_damage_on_landing() {
    let mut state = session_state_with_inventory(&[]);
    state.y = 80.0;
    state.on_ground = true;

    let mut falling = Vec::new();
    falling.extend_from_slice(&state.x.to_be_bytes());
    falling.extend_from_slice(&70.0_f64.to_be_bytes());
    falling.extend_from_slice(&state.z.to_be_bytes());
    falling.push(0);
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
        &mut Cursor::new(falling),
        &mut state,
    )
    .unwrap();
    assert!(update.position_changed);
    assert!(!update.health_changed);
    assert_eq!(state.fall_distance, 10.0);
    assert_eq!(state.health, 20.0);

    let mut landing = Vec::new();
    landing.extend_from_slice(&state.x.to_be_bytes());
    landing.extend_from_slice(&70.0_f64.to_be_bytes());
    landing.extend_from_slice(&state.z.to_be_bytes());
    landing.push(1);
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
        &mut Cursor::new(landing),
        &mut state,
    )
    .unwrap();
    assert!(update.position_changed);
    assert!(update.health_changed);
    assert_eq!(state.fall_distance, 0.0);
    assert_eq!(state.health, 13.0);
}

#[test]
pub fn fall_damage_is_suppressed_for_mayfly_players() {
    let mut state = session_state_with_inventory(&[]);
    state.y = 80.0;
    state.abilities.mayfly = true;

    let mut falling = Vec::new();
    falling.extend_from_slice(&state.x.to_be_bytes());
    falling.extend_from_slice(&60.0_f64.to_be_bytes());
    falling.extend_from_slice(&state.z.to_be_bytes());
    falling.push(0);
    super::super::update_play_session_state(
        super::super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
        &mut Cursor::new(falling),
        &mut state,
    )
    .unwrap();

    let mut landing = Vec::new();
    landing.extend_from_slice(&state.x.to_be_bytes());
    landing.extend_from_slice(&60.0_f64.to_be_bytes());
    landing.extend_from_slice(&state.z.to_be_bytes());
    landing.push(1);
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
        &mut Cursor::new(landing),
        &mut state,
    )
    .unwrap();
    assert!(!update.health_changed);
    assert_eq!(state.fall_distance, 0.0);
    assert_eq!(state.health, 20.0);
}

#[test]
pub fn player_fluid_detection_tracks_body_and_eye_water() {
    let mut state = session_state_with_inventory(&[]);
    state.x = 0.5;
    state.y = 64.0;
    state.z = 0.5;

    let shallow = super::super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
        (x == 0 && y == 64 && z == 0).then(|| "minecraft:water".to_string())
    });
    assert!(shallow.in_water);
    assert!(!shallow.eye_in_water);
    assert_eq!(shallow.water_height, 1.0);

    let submerged = super::super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
        (x == 0 && (64..=65).contains(&y) && z == 0)
            .then(|| "minecraft:water[level=0]".to_string())
    });
    assert!(submerged.in_water);
    assert!(submerged.eye_in_water);
    assert_eq!(submerged.water_height, 2.0);

    let waterlogged = super::super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
        (x == 0 && y == 64 && z == 0)
            .then(|| "minecraft:oak_fence[waterlogged=true]".to_string())
    });
    assert!(waterlogged.in_water);
}

#[test]
pub fn water_contact_suppresses_fall_distance_and_uses_water_exhaustion() {
    let mut state = session_state_with_inventory(&[]);
    state.fall_distance = 7.0;
    state.in_water = true;
    state.eye_in_water = true;
    state.input_sprinting = true;
    state.on_ground = true;

    let update = super::super::apply_player_movement(&mut state, 1.0, -1.0, 0.0, true);
    assert!(update.position_changed);
    assert!(!update.health_changed);
    assert_eq!(state.fall_distance, 0.0);
    assert_eq!(state.health, 20.0);
    assert!((state.food_exhaustion - 0.0141).abs() < f32::EPSILON);
}

#[test]
pub fn water_movement_does_not_seed_velocity_from_client_air_motion() {
    let mut state = session_state_with_inventory(&[]);
    state.in_water = true;
    state.eye_in_water = true;
    state.water_velocity_x = 0.01;
    state.water_velocity_y = -0.02;
    state.water_velocity_z = 0.03;

    super::super::apply_player_movement(&mut state, 0.3, -0.8, 0.3, true);

    assert_eq!(state.water_velocity_x, 0.01);
    assert_eq!(state.water_velocity_y, -0.02);
    assert_eq!(state.water_velocity_z, 0.03);
}

#[test]
pub fn water_tick_depletes_refills_air_and_drowns_like_java() {
    let mut state = session_state_with_inventory(&[]);

    let update = super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: true,
            water_height: 2.0,
        },
    );
    assert!(update.air_changed);
    assert!(!update.health_changed);
    assert!(update.motion_changed);
    assert_eq!(state.air_supply, 299);
    assert_eq!(state.water_velocity_y, -0.005);
    assert!(state.in_water);
    assert!(state.eye_in_water);

    state.air_supply = -19;
    state.health = 20.0;
    let update = super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: true,
            water_height: 2.0,
        },
    );
    assert!(update.air_changed);
    assert!(update.health_changed);
    assert_eq!(state.air_supply, 0);
    assert_eq!(state.health, 18.0);

    let update = super::super::tick_play_session_water(&mut state, super::super::PlayerFluidState::DRY);
    assert!(update.air_changed);
    assert!(!update.health_changed);
    assert_eq!(state.air_supply, 4);

    state.air_supply = 298;
    super::super::tick_play_session_water(&mut state, super::super::PlayerFluidState::DRY);
    assert_eq!(state.air_supply, super::super::MAX_AIR_SUPPLY);
}

#[test]
pub fn water_tick_applies_jump_impulse_and_drag_like_java() {
    let mut state = session_state_with_inventory(&[]);
    state.on_ground = false;
    state.water_velocity_y = -0.4;

    let update = super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!(update.motion_changed);
    assert!((state.water_velocity_y - (-0.325)).abs() < 1.0e-12);

    state.water_velocity_y = 0.0;
    state.input_jumping = true;
    let update = super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!(update.motion_changed);
    assert!((state.water_velocity_y - 0.027).abs() < 1.0e-12);

    state.water_velocity_y = 0.1;
    super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!((state.water_velocity_y - 0.107).abs() < 1.0e-12);

    state.input_jumping = false;
    state.input_shift = true;
    state.water_velocity_y = 0.0;
    super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!((state.water_velocity_y - (-0.037)).abs() < 1.0e-12);
}

#[test]
pub fn water_tick_preserves_horizontal_input_like_java() {
    let mut state = session_state_with_inventory(&[]);
    state.input_forward = true;
    state.input_jumping = true;
    state.yaw = 0.0;

    let update = super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!(update.motion_changed);
    assert_eq!(state.water_velocity_x, 0.0);
    assert!((state.water_velocity_z - 0.016).abs() < f64::EPSILON);
    assert!((state.water_velocity_y - 0.027).abs() < 1.0e-12);

    state.input_sprinting = true;
    super::super::tick_play_session_water(
        &mut state,
        super::super::PlayerFluidState {
            in_water: true,
            eye_in_water: false,
            water_height: 1.0,
        },
    );
    assert!((state.water_velocity_z - ((0.016 + 0.02) * 0.9)).abs() < 1.0e-12);
}

#[test]
pub fn air_supply_metadata_packet_uses_vanilla_entity_data_index() {
    let mut state = session_state_with_inventory(&[]);
    state.air_supply = 247;

    let packet = super::super::play_state_air_supply_metadata_packet(&state).unwrap();
    assert_eq!(packet.id, super::super::PLAYER_ENTITY_ID);
    assert_eq!(
        packet.packed_items,
        vec![EntityDataValue::typed(1, EntityMetadataValue::VarInt(247)).unwrap()]
    );
}

#[test]
pub fn player_input_tracks_sprint_jump_exhaustion() {
    let mut state = session_state_with_inventory(&[]);
    state.on_ground = true;

    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
        &mut Cursor::new(vec![16 | 64]),
        &mut state,
    )
    .unwrap();
    assert!(!update.health_changed);
    assert!(state.input_jumping);
    assert!(state.input_sprinting);
    assert!(!state.input_forward);
    assert!(!state.input_shift);
    assert_eq!(state.food_exhaustion, SPRINT_JUMP_EXHAUSTION);

    super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
        &mut Cursor::new(vec![16 | 64]),
        &mut state,
    )
    .unwrap();
    assert_eq!(
        state.food_exhaustion, SPRINT_JUMP_EXHAUSTION,
        "holding jump should not charge jump exhaustion every packet"
    );

    super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_INPUT_PACKET_ID,
        &mut Cursor::new(vec![1 | 32]),
        &mut state,
    )
    .unwrap();
    assert!(state.input_forward);
    assert!(state.input_shift);
    assert!(!state.input_jumping);
    assert!(!state.input_sprinting);
}

#[test]
pub fn sprint_movement_accumulates_food_exhaustion() {
    let mut state = session_state_with_inventory(&[]);
    state.input_sprinting = true;

    let mut movement = Vec::new();
    movement.extend_from_slice(&1.5_f64.to_be_bytes());
    movement.extend_from_slice(&64.0_f64.to_be_bytes());
    movement.extend_from_slice(&(-1.0_f64).to_be_bytes());
    movement.push(1);
    super::super::update_play_session_state(
        super::super::SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID,
        &mut Cursor::new(movement),
        &mut state,
    )
    .unwrap();
    assert_eq!(state.food_exhaustion, 0.05);
}

#[test]
pub fn food_tick_fast_regen_heals_and_adds_exhaustion() {
    let mut state = session_state_with_inventory(&[]);
    state.health = 18.0;
    state.food_saturation = 5.0;
    state.food_tick_timer = 9;

    let changed = super::super::tick_play_session_food(&mut state, FoodDifficulty::Normal, true, 10);
    assert!(changed);
    assert_eq!(state.health, 18.833334);
    assert_eq!(state.food_tick_timer, 0);
    assert_eq!(state.food_exhaustion, 5.0);
}

#[test]
pub fn food_tick_slow_regen_and_starvation_match_java_thresholds() {
    let mut state = session_state_with_inventory(&[]);
    state.health = 12.0;
    state.food_level = 18;
    state.food_saturation = 0.0;
    state.food_tick_timer = 79;

    assert!(super::super::tick_play_session_food(
        &mut state,
        FoodDifficulty::Normal,
        true,
        80
    ));
    assert_eq!(state.health, 13.0);
    assert_eq!(state.food_exhaustion, 6.0);

    state.health = 10.0;
    state.food_level = 0;
    state.food_tick_timer = 79;
    assert!(!super::super::tick_play_session_food(
        &mut state,
        FoodDifficulty::Easy,
        true,
        160
    ));
    assert_eq!(state.health, 10.0);

    state.health = 10.0;
    state.food_tick_timer = 79;
    assert!(super::super::tick_play_session_food(
        &mut state,
        FoodDifficulty::Hard,
        true,
        240
    ));
    assert_eq!(state.health, 9.0);
}

#[test]
pub fn peaceful_tick_restores_health_saturation_and_food() {
    let mut state = session_state_with_inventory(&[]);
    state.health = 19.0;
    state.food_level = 19;
    state.food_saturation = 4.0;

    assert!(super::super::tick_play_session_food(
        &mut state,
        FoodDifficulty::Peaceful,
        true,
        20
    ));
    assert_eq!(state.health, 20.0);
    assert_eq!(state.food_level, 20);
    assert_eq!(state.food_saturation, 5.0);
}

#[test]
pub fn client_respawn_command_only_requests_respawn_when_dead() {
    let mut alive = session_state_with_inventory(&[]);
    let mut action = Vec::new();
    write_var_i32(&mut action, 0).unwrap();
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
        &mut Cursor::new(action),
        &mut alive,
    )
    .unwrap();
    assert!(!update.respawn_requested);

    let mut dead = session_state_with_inventory(&[]);
    dead.health = 0.0;
    let mut action = Vec::new();
    write_var_i32(&mut action, 0).unwrap();
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
        &mut Cursor::new(action),
        &mut dead,
    )
    .unwrap();
    assert!(update.respawn_requested);

    let mut stats = Vec::new();
    write_var_i32(&mut stats, 1).unwrap();
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
        &mut Cursor::new(stats),
        &mut dead,
    )
    .unwrap();
    assert!(!update.respawn_requested);
}

#[test]
pub fn respawn_application_restores_health_and_clears_fall_state() {
    let mut state = session_state_with_inventory(&[]);
    state.health = 0.0;
    state.food_level = 3;
    state.food_saturation = 0.0;
    state.food_exhaustion = 12.0;
    state.air_supply = 7;
    state.in_water = true;
    state.eye_in_water = true;
    state.water_fluid_height = 2.0;
    state.fall_distance = 48.0;
    state.on_ground = false;
    state.xp_level = 9;
    state.xp_total = 123;
    state.score = 77;

    super::super::apply_spawn_placement_to_state(
        &mut state,
        super::super::PlayerSpawnPlacement {
            x: 12.5,
            y: 70.0,
            z: -3.5,
            yaw: 90.0,
            pitch: 0.0,
        },
    );
    super::super::reset_play_state_after_death_respawn(&mut state);

    assert_eq!((state.x, state.y, state.z), (12.5, 70.0, -3.5));
    assert_eq!(state.health, 20.0);
    assert_eq!(state.food_level, 20);
    assert_eq!(state.food_saturation, 5.0);
    assert_eq!(state.food_exhaustion, 0.0);
    assert_eq!(state.food_tick_timer, 0);
    assert!(!state.input_sprinting);
    assert!(!state.input_jumping);
    assert_eq!(state.air_supply, super::super::MAX_AIR_SUPPLY);
    assert!(!state.in_water);
    assert!(!state.eye_in_water);
    assert_eq!(state.water_fluid_height, 0.0);
    assert_eq!(state.fall_distance, 0.0);
    assert!(state.on_ground);
    assert_eq!(state.xp_level, 0);
    assert_eq!(state.xp_total, 0);
    assert_eq!(state.score, 0);
}

#[test]
pub fn overworld_respawn_pos_uses_motion_blocking_surface_like_java() {
    let mut chunk = LevelChunk::empty(crate::storage::region::ChunkPos { x: 0, z: 0 });
    chunk.set_block_state(0, 63, 0, "minecraft:grass_block");
    assert_eq!(
        super::super::overworld_respawn_pos_in_chunk(&chunk, 0, 0),
        Some((0, 64, 0))
    );

    chunk.set_block_state(0, 64, 0, "minecraft:water");
    assert_eq!(super::super::overworld_respawn_pos_in_chunk(&chunk, 0, 0), None);
}

#[test]
pub fn play_session_state_nbt_round_trip_preserves_hotbar_and_main_inventory() {
    // Slot 0 (hotbar), slot 9 (main inventory row 1), slot 35 (last main slot).
    // Java: Inventory saves slots 0-35 — all three must survive the round-trip.
    let original_items = &[
        ("minecraft:dirt", 64, 0usize),
        ("minecraft:stone", 32, 9),
        ("minecraft:sand", 16, 35),
    ];
    let state = session_state_with_inventory(original_items);
    let tag = play_session_state_to_nbt(&state);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();

    let saved = restored.inventory_menu.player_inventory().saved_items();
    assert_eq!(saved.len(), 3, "expected exactly 3 items after round-trip");

    for (id, count, slot) in original_items {
        let found = saved.iter().find(|(s, _)| s == slot);
        let found = found.unwrap_or_else(|| panic!("slot {slot} missing after round-trip"));
        assert_eq!(found.1.item_id(), *id, "item id mismatch at slot {slot}");
        assert_eq!(found.1.count(), *count, "count mismatch at slot {slot}");
    }
}

#[test]
pub fn play_session_state_nbt_round_trip_ignores_out_of_range_slots() {
    // Slots >= 36 (armour, offhand, etc.) are outside the main inventory range
    // and must be silently dropped during deserialisation.
    // Java: Inventory.load() only writes to slots 0-35.
    let tag = Tag::Compound(vec![
        ("DataVersion".to_string(), Tag::Int(4791)),
        (
            "Pos".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("OnGround".to_string(), Tag::Byte(1)),
        ("Health".to_string(), Tag::Float(20.0)),
        ("foodLevel".to_string(), Tag::Int(20)),
        ("foodSaturationLevel".to_string(), Tag::Float(5.0)),
        ("XpLevel".to_string(), Tag::Int(0)),
        ("XpP".to_string(), Tag::Float(0.0)),
        ("XpTotal".to_string(), Tag::Int(0)),
        ("SelectedItemSlot".to_string(), Tag::Int(0)),
        ("playerGameType".to_string(), Tag::Int(0)),
        (
            "Inventory".to_string(),
            Tag::List(vec![
                // Valid slot
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(0)),
                    ("id".to_string(), Tag::String("minecraft:dirt".to_string())),
                    ("count".to_string(), Tag::Int(1)),
                ]),
                // Out-of-range slot (armour slot 100) — must be ignored
                Tag::Compound(vec![
                    ("Slot".to_string(), Tag::Byte(100u8 as i8)),
                    ("id".to_string(), Tag::String("minecraft:stone".to_string())),
                    ("count".to_string(), Tag::Int(1)),
                ]),
            ]),
        ),
    ]);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    let saved = restored.inventory_menu.player_inventory().saved_items();
    assert_eq!(saved.len(), 1, "only the in-range slot should survive");
    assert_eq!(saved[0].0, 0);
    assert_eq!(saved[0].1.item_id(), "minecraft:dirt");
}

#[test]
pub fn play_session_state_from_nbt_clamps_vanilla_playerdata_bounds() {
    let tag = Tag::Compound(vec![
        ("DataVersion".to_string(), Tag::Int(4790)),
        (
            "Pos".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(64.0), Tag::Double(0.0)]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(0.0), Tag::Float(0.0)]),
        ),
        ("Health".to_string(), Tag::Float(200.0)),
        ("foodLevel".to_string(), Tag::Int(99)),
        ("foodSaturationLevel".to_string(), Tag::Float(99.0)),
        ("XpP".to_string(), Tag::Float(9.0)),
        ("XpLevel".to_string(), Tag::Int(-7)),
        ("XpTotal".to_string(), Tag::Int(-12)),
        ("SelectedItemSlot".to_string(), Tag::Int(99)),
        ("playerGameType".to_string(), Tag::Int(99)),
    ]);

    let restored =
        play_session_state_from_nbt(&tag, GameMode::Creative, &RecipeMap::default()).unwrap();
    assert_eq!(restored.health, 20.0);
    assert_eq!(restored.food_level, 20);
    assert_eq!(restored.food_saturation, 20.0);
    assert_eq!(restored.air_supply, super::super::MAX_AIR_SUPPLY);
    assert_eq!(restored.xp_progress, 1.0);
    assert_eq!(restored.xp_level, 0);
    assert_eq!(restored.xp_total, 0);
    assert_eq!(restored.selected_slot, 0);
    assert_eq!(restored.game_mode, GameMode::Survival);
}

#[test]
pub fn play_session_state_nbt_round_trip_preserves_full_playerdata_surface() {
    let mut state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
    state.fall_distance = 6.25;
    state.food_exhaustion = 3.5;
    state.food_tick_timer = 72;
    state.air_supply = 123;
    state.xp_progress = 0.75;
    state.xp_level = 12;
    state.xp_total = 345;
    state.xp_seed = 98_765;
    state.score = 42;
    state.previous_game_mode = Some(GameMode::Adventure);
    state.spawn = Some(PlayerSpawnData {
        dimension: "minecraft:the_nether".to_string(),
        x: 11,
        y: 72,
        z: -13,
        forced: true,
    });
    state.seen_credits = true;
    state.entered_nether_position = Some((1.25, 64.0, -2.5));
    state.last_death_location = Some(PlayerGlobalPosData {
        dimension: "minecraft:overworld".to_string(),
        x: 3,
        y: 65,
        z: 4,
    });
    state.root_vehicle = Some(Tag::Compound(vec![(
        "Entity".to_string(),
        Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:boat".to_string()),
        )]),
    )]));
    state.active_effects = vec![Tag::Compound(vec![
        ("id".to_string(), Tag::String("minecraft:speed".to_string())),
        ("amplifier".to_string(), Tag::Int(1)),
    ])];
    state.ender_items = vec![Tag::Compound(vec![
        ("Slot".to_string(), Tag::Byte(0)),
        (
            "id".to_string(),
            Tag::String("minecraft:diamond".to_string()),
        ),
        ("count".to_string(), Tag::Int(2)),
    ])];
    state.abilities = PlayerNbtAbilities {
        invulnerable: true,
        flying: true,
        mayfly: true,
        instabuild: true,
        may_build: false,
        fly_speed: 0.08,
        walk_speed: 0.12,
    };

    let tag = play_session_state_to_nbt(&state);
    for field in [
        "Pos",
        "Rotation",
        "Motion",
        "Air",
        "fall_distance",
        "Health",
        "foodLevel",
        "foodSaturationLevel",
        "foodExhaustionLevel",
        "foodTickTimer",
        "XpP",
        "XpLevel",
        "XpTotal",
        "XpSeed",
        "Score",
        "SelectedItemSlot",
        "Inventory",
        "EnderItems",
        "playerGameType",
        "previousPlayerGameType",
        "SpawnX",
        "SpawnY",
        "SpawnZ",
        "SpawnForced",
        "SpawnDimension",
        "seenCredits",
        "recipeBook",
        "LastDeathLocation",
        "enteredNetherPosition",
        "RootVehicle",
        "abilities",
        "active_effects",
    ] {
        assert!(
            field_value(&tag, field).is_some(),
            "{field} missing from player NBT"
        );
    }

    let restored =
        play_session_state_from_nbt(&tag, GameMode::Survival, &RecipeMap::default()).unwrap();
    assert_eq!(restored.fall_distance, 6.25);
    assert_eq!(restored.food_exhaustion, 3.5);
    assert_eq!(restored.food_tick_timer, 72);
    assert_eq!(restored.air_supply, 123);
    assert_eq!(restored.xp_seed, 98_765);
    assert_eq!(restored.score, 42);
    assert_eq!(restored.previous_game_mode, Some(GameMode::Adventure));
    assert_eq!(restored.spawn, state.spawn);
    assert!(restored.seen_credits);
    assert_eq!(
        restored.entered_nether_position,
        state.entered_nether_position
    );
    assert_eq!(restored.last_death_location, state.last_death_location);
    assert_eq!(restored.root_vehicle, state.root_vehicle);
    assert_eq!(restored.active_effects, state.active_effects);
    assert_eq!(restored.ender_items, state.ender_items);
    assert_eq!(restored.abilities, state.abilities);
}

#[test]
pub fn play_session_state_nbt_inventory_tag_matches_vanilla_format() {
    // Verify the serialised TAG_List contains TAG_Compound entries with the
    // exact field names used by vanilla: "Slot" (TAG_Byte), "id" (TAG_String),
    // "count" (TAG_Int).  This is the wire format read back by the Java server
    // when loading player data.
    let state = session_state_with_inventory(&[("minecraft:stone", 5, 3)]);
    let tag = play_session_state_to_nbt(&state);
    let Tag::Compound(fields) = &tag else {
        panic!("expected compound tag");
    };
    let inventory_tag = fields
        .iter()
        .find_map(|(name, value)| (name == "Inventory").then_some(value))
        .expect("Inventory tag missing");
    let Tag::List(items) = inventory_tag else {
        panic!("Inventory must be a TAG_List");
    };
    assert_eq!(items.len(), 1);
    let Tag::Compound(item_fields) = &items[0] else {
        panic!("inventory entry must be TAG_Compound");
    };
    let slot = item_fields
        .iter()
        .find_map(|(n, v)| (n == "Slot").then_some(v))
        .expect("Slot missing");
    assert!(matches!(slot, Tag::Byte(3)), "Slot must be TAG_Byte(3)");
    let id = item_fields
        .iter()
        .find_map(|(n, v)| (n == "id").then_some(v))
        .expect("id missing");
    assert!(
        matches!(id, Tag::String(s) if s == "minecraft:stone"),
        "id must be TAG_String"
    );
    let count = item_fields
        .iter()
        .find_map(|(n, v)| (n == "count").then_some(v))
        .expect("count missing");
    assert!(matches!(count, Tag::Int(5)), "count must be TAG_Int(5)");
}

// ─── write_lp_vec3 ───────────────────────────────────────────────────────

#[test]
pub fn write_lp_vec3_zero_writes_single_zero_byte() {
    // Java: LpVec3.write — chessboard length below threshold → single 0x00 byte.
    let mut buf = Vec::new();
    write_lp_vec3(&mut buf, 0.0, 0.0, 0.0).unwrap();
    assert_eq!(buf, &[0u8]);
}

#[test]
pub fn write_lp_vec3_nonzero_writes_six_bytes_for_unit_scale() {
    // For velocity magnitude ≤ 1.0 the scale is 1 and isPartial=false → exactly 6 bytes.
    let mut buf = Vec::new();
    write_lp_vec3(&mut buf, 0.3, 0.1, -0.3).unwrap();
    assert_eq!(
        buf.len(),
        6,
        "scale=1 non-zero velocity should encode to 6 bytes"
    );
}
