use super::super::*;
use super::*;

#[test]
pub fn chat_validation_matches_java_allowed_chat_characters() {
    assert!(!chat_message_is_illegal("hello world"));
    assert!(!chat_message_is_illegal("snowman \u{2603}"));
    assert!(chat_message_is_illegal("bad\nline"));
    assert!(chat_message_is_illegal("bad\u{7f}delete"));
    assert!(chat_message_is_illegal("bad\u{00a7}format"));
}

#[test]
pub fn literal_chat_component_uses_network_nbt_shape() {
    assert_eq!(
        literal_component_tag("<Steve> hello"),
        Tag::Compound(vec![(
            "text".to_string(),
            Tag::String("<Steve> hello".to_string())
        )])
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
pub fn serverbound_player_abilities_updates_flying_only_when_mayfly() {
    let mut state = session_state_with_inventory(&[]);
    state.abilities.flying = true;
    let update = super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_ABILITIES_PACKET_ID,
        &mut Cursor::new(vec![0x02]),
        &mut state,
    )
    .unwrap();
    assert_eq!(update, super::super::PlaySessionUpdate::default());
    assert!(!state.abilities.flying);

    state.abilities.mayfly = true;
    super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_ABILITIES_PACKET_ID,
        &mut Cursor::new(vec![0x02]),
        &mut state,
    )
    .unwrap();
    assert!(state.abilities.flying);

    super::super::update_play_session_state(
        super::super::SERVERBOUND_PLAYER_ABILITIES_PACKET_ID,
        &mut Cursor::new(vec![0x00]),
        &mut state,
    )
    .unwrap();
    assert!(!state.abilities.flying);
}

#[test]
pub fn creative_mode_slot_packet_applies_java_slot_and_ability_gates() {
    let mut state = session_state_with_inventory(&[]);
    let packet = super::super::ServerboundSetCreativeModeSlotPacket {
        slot_num: 36,
        item_stack: super::super::RawItemStack {
            count: 64,
            item_id: crate::item_catalog::item_protocol_id("minecraft:stone"),
            components: super::super::RawDataComponentPatch::empty(),
        },
    };
    assert!(
        super::super::player_creative_packets::apply_set_creative_mode_slot_packet(
            &mut state,
            packet.clone()
        )
        .is_none()
    );
    assert_eq!(state.inventory_menu.get_slot(36), Some(ItemStack::empty()));

    state.abilities.instabuild = true;
    let slot_update = super::super::player_creative_packets::apply_set_creative_mode_slot_packet(
        &mut state, packet,
    )
    .expect("creative slot packet should be accepted with instabuild");
    assert_eq!(
        state.inventory_menu.get_slot(36),
        Some(ItemStack::new("minecraft:stone", 64))
    );
    assert_eq!(state.container_state_id, 1);
    assert_eq!(slot_update.container_id, 0);
    assert_eq!(slot_update.state_id, 1);
    assert_eq!(slot_update.slot, 36);
    assert_eq!(slot_update.item_stack.count, 64);

    let invalid_result_slot = super::super::ServerboundSetCreativeModeSlotPacket {
        slot_num: 0,
        item_stack: super::super::RawItemStack::empty(),
    };
    assert!(
        super::super::player_creative_packets::apply_set_creative_mode_slot_packet(
            &mut state,
            invalid_result_slot
        )
        .is_none()
    );
    assert_eq!(state.container_state_id, 1);

    let clear = super::super::ServerboundSetCreativeModeSlotPacket {
        slot_num: 36,
        item_stack: super::super::RawItemStack::empty(),
    };
    let clear_update = super::super::player_creative_packets::apply_set_creative_mode_slot_packet(
        &mut state, clear,
    )
    .expect("creative slot clear should be accepted");
    assert_eq!(state.inventory_menu.get_slot(36), Some(ItemStack::empty()));
    assert_eq!(state.container_state_id, 2);
    assert_eq!(clear_update.state_id, 2);
    assert_eq!(clear_update.slot, 36);
    assert_eq!(clear_update.item_stack.count, 0);
}

#[test]
pub fn fresh_creative_session_accepts_and_persists_creative_picker_items() {
    let mut properties = crate::server_properties::ServerProperties::load_or_default(
        std::path::Path::new("/tmp/vibecraft-missing-server.properties"),
    )
    .unwrap();
    properties.game_mode = "creative".to_string();

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-creative-session-{unique}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&world_root).unwrap();

    let mut state = super::super::load_play_session_state(
        &world_root,
        "00000000-0000-4000-8000-000000000333",
        &properties,
        &RecipeMap::default(),
        0,
    );
    let _ = std::fs::remove_dir_all(&world_root);

    assert_eq!(state.game_mode, GameMode::Creative);
    assert!(state.abilities.instabuild);
    assert!(state.abilities.mayfly);
    assert!(state.abilities.invulnerable);

    let packet = super::super::ServerboundSetCreativeModeSlotPacket {
        slot_num: 36,
        item_stack: super::super::RawItemStack {
            count: 64,
            item_id: crate::item_catalog::item_protocol_id("minecraft:stone"),
            components: super::super::RawDataComponentPatch::empty(),
        },
    };
    super::super::player_creative_packets::apply_set_creative_mode_slot_packet(&mut state, packet)
        .expect("creative picker item should be accepted by a creative player");

    let tag = play_session_state_to_nbt(&state);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Creative, &RecipeMap::default()).unwrap();
    assert_eq!(
        restored.inventory_menu.get_slot(36),
        Some(ItemStack::new("minecraft:stone", 64))
    );
}

#[test]
pub fn fresh_player_inventory_menu_uses_loaded_recipes_for_manual_crafting() -> Result<(), String> {
    let recipe_map = RecipeMap::create(vec![crate::recipe_system::RecipeHolder {
        id: "minecraft:oak_planks",
        recipe: crate::recipe_system::RecipeKind::Shapeless {
            category: crate::recipe_system::CraftingBookCategoryModel::Misc,
            ingredients: vec![crate::recipe_system::IngredientSpec::Item("minecraft:oak_log")],
            result: crate::recipe_system::ItemAmount {
                item: "minecraft:oak_planks",
                count: 4,
            },
        },
    }]);
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("system clock before Unix epoch: {error}"))?
        .as_nanos();
    let world_root = std::env::temp_dir().join(format!(
        "vibecraft-fresh-player-recipes-{unique}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&world_root)
        .map_err(|error| format!("failed to create fresh-player test world: {error}"))?;
    let properties = crate::server_properties::ServerProperties::load_or_default(
        &world_root.join("server.properties"),
    )?;

    let mut state = super::super::load_play_session_state(
        &world_root,
        "00000000-0000-4000-8000-000000000444",
        &properties,
        &recipe_map,
        0,
    );
    std::fs::remove_dir_all(&world_root)
        .map_err(|error| format!("failed to remove fresh-player test world: {error}"))?;

    assert!(state.inventory_menu.set_slot(
        1,
        ItemStack::new("minecraft:oak_log", 1)
    ));
    assert_eq!(
        state.inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4))
    );
    Ok(())
}

#[test]
pub fn container_close_returns_cursor_stack_to_inventory_before_save() {
    let mut state = session_state_with_inventory(&[]);
    state.carried_item = ItemStack::new("minecraft:stone", 64);

    let mut payload = Vec::new();
    ServerboundContainerClosePacket { container_id: 0 }
        .write(&mut payload)
        .unwrap();
    update_play_session_state(
        SERVERBOUND_CONTAINER_CLOSE_PACKET_ID,
        &mut Cursor::new(payload),
        &mut state,
    )
    .unwrap();

    assert!(
        state.carried_item.is_empty(),
        "container close must clear the server-side cursor stack"
    );
    let slot = state.inventory_menu.get_slot(36).unwrap();
    assert_eq!(slot.item_id(), "minecraft:stone");
    assert_eq!(
        slot.count(),
        64,
        "cursor stack should be placed into the first hotbar slot"
    );

    let tag = play_session_state_to_nbt(&state);
    let restored =
        play_session_state_from_nbt(&tag, GameMode::Creative, &RecipeMap::default()).unwrap();
    assert_eq!(
        restored.inventory_menu.get_slot(36),
        Some(ItemStack::new("minecraft:stone", 64)),
        "cursor stack returned on close must survive playerdata reload"
    );

    let mut state = session_state_with_inventory(&[]);
    state.carried_item = ItemStack::new("minecraft:dirt", 32);

    let mut payload = Vec::new();
    ServerboundContainerClosePacket { container_id: 128 }
        .write(&mut payload)
        .unwrap();
    update_play_session_state(
        SERVERBOUND_CONTAINER_CLOSE_PACKET_ID,
        &mut Cursor::new(payload),
        &mut state,
    )
    .unwrap();

    assert!(state.carried_item.is_empty());
    let slot = state.inventory_menu.get_slot(36).unwrap();
    assert_eq!(slot.item_id(), "minecraft:dirt");
    assert_eq!(
        slot.count(),
        32,
        "serverbound close intentionally ignores the packet container ID like Java"
    );
}

#[test]
pub fn creative_mode_debug_packets_do_not_fall_through_to_unexpected_disconnect() {
    // Java 26.1.2 GameProtocols registers serverbound player_abilities at
    // 40, pick_item_from_block/entity at 36/37, and set_creative_mode_slot at 56.
    // These are common in creative mode: double-tap space toggles packet 40,
    // middle-click pick-block sends 36/37, and taking an item from the creative
    // inventory sends packet 56.
    assert_eq!(super::super::SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID, 36);
    assert_eq!(
        super::super::SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID,
        37
    );
    assert_eq!(super::super::SERVERBOUND_PLAYER_ABILITIES_PACKET_ID, 40);
    assert_eq!(
        super::super::SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
        56
    );
    assert!(super::super::play_packet_has_live_status_handler(
        super::super::SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID
    ));
    assert!(super::super::play_packet_has_live_status_handler(
        super::super::SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID
    ));
    assert!(super::super::play_packet_is_handled_after_state_update(
        super::super::SERVERBOUND_PLAYER_ABILITIES_PACKET_ID
    ));
    assert!(super::super::play_packet_has_live_status_handler(
        super::super::SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID
    ));
    assert!(!super::super::play_packet_has_live_status_handler(999));
}

#[test]
pub fn pick_item_from_block_selects_existing_or_creative_cloned_block() {
    let chunk_pos = crate::storage::region::ChunkPos { x: 0, z: 0 };
    let mut chunk = crate::storage::chunk::LevelChunk::empty(chunk_pos);
    chunk.set_block_state(1, 64, 0, "minecraft:stone");
    let cache = super::super::GeneratedChunkCache::default();
    cache
        .chunks
        .lock()
        .unwrap()
        .insert(chunk_pos, std::sync::Arc::new(chunk));
    let world_root =
        std::env::temp_dir().join(format!("vibecraft-pick-item-block-{}", std::process::id()));
    let layout = super::super::WorldLayout::new(&world_root);

    let mut state = PlaySessionState {
        x: 0.5,
        y: 64.0,
        z: 0.5,
        game_mode: GameMode::Creative,
        abilities: super::super::PlayerNbtAbilities::for_game_mode(GameMode::Creative),
        ..PlaySessionState::default()
    };

    let packet = super::super::ServerboundPickItemFromBlockPacket {
        x: 1,
        y: 64,
        z: 0,
        include_data: true,
    };
    assert_eq!(
        super::super::player_creative_packets::apply_pick_item_from_block_packet(
            &mut state, packet, &layout, &cache
        ),
        super::super::player_creative_packets::PickItemOutcome::Picked {
            inventory_changed: true
        }
    );
    assert_eq!(state.selected_slot, 0);
    assert_eq!(
        state.inventory_menu.get_slot(36),
        Some(ItemStack::new("minecraft:stone", 1))
    );

    state
        .inventory_menu
        .set_slot(36, ItemStack::new("minecraft:dirt", 1));
    state
        .inventory_menu
        .set_slot(9, ItemStack::new("minecraft:stone", 1));
    state.selected_slot = 0;
    assert_eq!(
        super::super::player_creative_packets::apply_pick_item_from_block_packet(
            &mut state, packet, &layout, &cache
        ),
        super::super::player_creative_packets::PickItemOutcome::Picked {
            inventory_changed: true
        }
    );
    assert_eq!(state.selected_slot, 1);
    assert_eq!(
        state.inventory_menu.get_slot(37),
        Some(ItemStack::new("minecraft:stone", 1))
    );

    let far_packet = super::super::ServerboundPickItemFromBlockPacket {
        x: 100,
        y: 64,
        z: 0,
        include_data: false,
    };
    assert_eq!(
        super::super::player_creative_packets::apply_pick_item_from_block_packet(
            &mut state, far_packet, &layout, &cache
        ),
        super::super::player_creative_packets::PickItemOutcome::NoItem
    );
    let _ = std::fs::remove_dir_all(&world_root);
}

#[test]
pub fn log_ips_uses_java_loggable_address_redaction_and_login_shape() {
    assert_eq!(
        super::super::loggable_remote_address(true, "203.0.113.10:25565"),
        "203.0.113.10:25565"
    );
    assert_eq!(
        super::super::loggable_remote_address(false, "203.0.113.10:25565"),
        "IP hidden"
    );
    assert_eq!(
        super::super::player_login_log_message("Steve", "IP hidden", 1, 0.5, 64.0, -12.25),
        "Steve[IP hidden] logged in with entity id 1 at (0.5, 64, -12.25)"
    );
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
        (x == 0 && (64..=65).contains(&y) && z == 0).then(|| "minecraft:water[level=0]".to_string())
    });
    assert!(submerged.in_water);
    assert!(submerged.eye_in_water);
    assert_eq!(submerged.water_height, 2.0);

    let waterlogged =
        super::super::detect_play_session_fluid_state_with_lookup(&state, |x, y, z| {
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

    let update =
        super::super::tick_play_session_water(&mut state, super::super::PlayerFluidState::DRY);
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
    assert!(
        (state.food_exhaustion - 0.05).abs() < f32::EPSILON,
        "sprint exhaustion should be ~0.05, got {}",
        state.food_exhaustion
    );
}

#[test]
pub fn food_tick_fast_regen_heals_and_adds_exhaustion() {
    let mut state = session_state_with_inventory(&[]);
    state.health = 18.0;
    state.food_saturation = 5.0;
    state.food_tick_timer = 9;

    let changed =
        super::super::tick_play_session_food(&mut state, FoodDifficulty::Normal, true, 10);
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
    assert_eq!(
        super::super::overworld_respawn_pos_in_chunk(&chunk, 0, 0),
        None
    );
}

mod playerdata;
