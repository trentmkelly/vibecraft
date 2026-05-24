use super::super::*;
use super::*;

#[test]
fn play_session_container_state_id_advances_only_after_accepted_click() {
    let mut session = PlaySession::new(7, 0);
    let mut menu = Menu::new(1);
    menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stick", 2));

    let stale = session.apply_scripted_container_click(
        &mut menu,
        &scripted_container_click(3, 0, vec![(0, ItemStack::empty())], ItemStack::empty()),
    );
    assert!(!stale.accepted);
    assert_eq!(stale.expected_state_id, 0);
    assert_eq!(session.container_state_id, 0);

    let accepted = session.apply_scripted_container_click(
        &mut menu,
        &scripted_container_click(
            0,
            0,
            vec![(0, ItemStack::empty())],
            ItemStack::new("minecraft:stick", 2),
        ),
    );
    assert!(accepted.accepted);
    assert_eq!(accepted.next_state_id, 1);
    assert_eq!(session.container_state_id, 1);

    let rejected = session.apply_scripted_container_click(
        &mut menu,
        &scripted_container_click(
            1,
            0,
            vec![(0, ItemStack::empty())],
            ItemStack::new("minecraft:stick", 99),
        ),
    );
    assert!(!rejected.accepted);
    assert_eq!(session.container_state_id, 1);
}

#[test]
fn stale_container_state_id_corrections_become_set_slot_packets() {
    let mut session = PlaySession::new(7, 0);
    session.container_state_id = 8;
    let mut menu = Menu::new(1);
    menu.slots[0] = Slot::with_stack(ItemStack::new("minecraft:stone", 2));

    let rejected = session.apply_scripted_container_click(
        &mut menu,
        &scripted_container_click(
            7,
            0,
            vec![(0, ItemStack::empty())],
            ItemStack::new("minecraft:stone", 2),
        ),
    );
    assert!(!rejected.accepted);

    let packets = slot_corrections_to_set_slot_packets(
        0,
        rejected.expected_state_id,
        &rejected.corrections,
    )
    .unwrap();
    assert_eq!(packets.len(), 2);
    assert_eq!(packets[0].container_id, 0);
    assert_eq!(packets[0].state_id, 8);
    assert_eq!(packets[0].slot, 0);
    assert_eq!(packets[0].item_stack.count, 2);
    assert_eq!(
        packets[0].item_stack.item_id,
        item_protocol_id("minecraft:stone")
    );
    assert_eq!(packets[1].slot, -1);
    assert_eq!(packets[1].item_stack.count, 0);
    assert_eq!(session.container_state_id, 8);
}

pub fn network_crafting_test_recipes() -> crate::recipe_system::RecipeMap {
    crate::recipe_system::RecipeMap::create(vec![crate::recipe_system::RecipeHolder {
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

pub fn vanilla_recipe_map() -> crate::recipe_system::RecipeMap {
    let recipe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/data/minecraft/recipe");
    crate::recipe_system::load_recipe_directory(&recipe_dir)
        .expect("bundled vanilla recipe directory should load")
        .recipe_map()
        .clone()
}

#[test]
fn pending_container_click_updates_inventory_menu_result_and_unlocks_recipe() {
    let mut session = PlaySession::new(7, 0);
    let mut inventory_menu = InventoryMenu::new(
        crate::player_inventory::PlayerInventory::new(),
        network_crafting_test_recipes(),
    );
    let mut carried = ItemStack::new("minecraft:oak_log", 1);

    session.last_container_click = Some(ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0,
        slot_num: 1,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    });
    let instructions =
        session.process_pending_container_click(&mut inventory_menu, &mut carried);

    assert_eq!(session.container_state_id, 1);
    assert!(carried.is_empty());
    assert_eq!(
        inventory_menu.get_slot(1),
        Some(ItemStack::new("minecraft:oak_log", 1))
    );
    assert_eq!(
        inventory_menu.get_slot(0),
        Some(ItemStack::new("minecraft:oak_planks", 4))
    );
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::ContainerSetSlot(packet)
            if packet.container_id == 0
                && packet.state_id == 1
                && packet.slot == 0
                && packet.item_stack.count == 4
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::ContainerSetSlot(packet)
            if packet.container_id == 0
                && packet.state_id == 1
                && packet.slot == 1
                && packet.item_stack.count == 1
    )));

    session.last_container_click = Some(ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 1,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: BTreeMap::new(),
        carried_item: HashedStack::empty(),
    });
    let instructions =
        session.process_pending_container_click(&mut inventory_menu, &mut carried);

    assert_eq!(session.container_state_id, 2);
    assert_eq!(carried, ItemStack::new("minecraft:oak_planks", 4));
    assert_eq!(inventory_menu.get_slot(1), Some(ItemStack::empty()));
    assert_eq!(inventory_menu.get_slot(0), Some(ItemStack::empty()));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::ContainerSetSlot(packet)
            if packet.container_id == 0
                && packet.state_id == 2
                && packet.slot == 0
                && packet.item_stack.count == 0
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::ContainerSetSlot(packet)
            if packet.container_id == 0
                && packet.state_id == 2
                && packet.slot == 1
                && packet.item_stack.count == 0
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::SetCursorItem(packet)
            if packet.item_stack.count == 4
                && packet.item_stack.item_id == item_protocol_id("minecraft:oak_planks")
    )));
    assert!(instructions.iter().any(|instruction| matches!(
        instruction,
        PlayInstruction::RecipesUnlocked(ids) if ids == &vec!["minecraft:oak_planks"]
    )));
}

#[test]
fn death_and_respawn_flow_match_player_list_respawn_packet_order() {
    let mut session = PlaySession::new(99, 0);
    assert_eq!(
        session.death_screen("{\"translate\":\"death.attack.generic\"}"),
        PlayInstruction::CombatKill(ClientboundPlayerCombatKillPacket {
            player_id: 99,
            message: "{\"translate\":\"death.attack.generic\"}".to_string(),
        })
    );

    let flow = session.respawn_flow(RespawnRequest {
        reason: RespawnReason::Death,
        keep_all_player_data: false,
        missing_respawn_block: true,
        hardcore: true,
        active_effect_count: 2,
        respawn_anchor_depleted: true,
        spawn_info: CommonPlayerSpawnInfo::default(),
    });

    assert_eq!(
        flow,
        vec![
            PlayInstruction::NoRespawnBlockAvailable,
            PlayInstruction::Respawn(ClientboundRespawnPacket {
                spawn_info: CommonPlayerSpawnInfo::default(),
                data_to_keep: RespawnDataToKeep::NONE,
            }),
            PlayInstruction::TeleportToSpawn { teleport_id: 0 },
            PlayInstruction::SetDefaultSpawnPosition,
            PlayInstruction::ChangeDifficulty {
                difficulty: GameDifficulty::Normal,
                locked: false,
            },
            PlayInstruction::SetExperience,
            PlayInstruction::ActiveEffects { count: 2 },
            PlayInstruction::SendLevelInfo,
            PlayInstruction::UpdatePermissionLevel(0),
            PlayInstruction::AddPlayerToLevel,
            PlayInstruction::InitInventoryMenu,
            PlayInstruction::SetHealth,
            PlayInstruction::SetGameModeSpectator,
            PlayInstruction::DisableSpectatorsGenerateChunks,
            PlayInstruction::RespawnAnchorDepleteSound,
        ]
    );
}

#[test]
fn dimension_return_respawn_keeps_attribute_modifiers_like_vanilla_keep_all_path() {
    let mut session = PlaySession::new(99, 0);
    let flow = session.respawn_flow(RespawnRequest {
        reason: RespawnReason::WonGameReturnToOverworld,
        keep_all_player_data: true,
        missing_respawn_block: false,
        hardcore: false,
        active_effect_count: 0,
        respawn_anchor_depleted: false,
        spawn_info: CommonPlayerSpawnInfo {
            dimension: Identifier::parse("minecraft:overworld").unwrap(),
            previous_game_mode: Some(GameMode::Survival),
            ..CommonPlayerSpawnInfo::default()
        },
    });

    let PlayInstruction::Respawn(packet) = &flow[0] else {
        panic!("respawn packet should be first");
    };
    assert_eq!(packet.data_to_keep.bits(), 1);
    assert!(packet
        .data_to_keep
        .should_keep(RespawnDataToKeep::KEEP_ATTRIBUTE_MODIFIERS));
    assert!(!packet
        .data_to_keep
        .should_keep(RespawnDataToKeep::KEEP_ENTITY_DATA));
    assert!(!flow.contains(&PlayInstruction::SetGameModeSpectator));
}

#[test]
fn player_loaded_packet_moves_session_to_playing() {
    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, Vec::new())),
        DispatchOutcome::Handled
    );
    assert_eq!(session.state, PlayState::Playing);
    assert!(session.loaded);
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, vec![0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert_eq!(session.state, PlayState::Playing);
    assert!(session.loaded);
}

#[test]
fn malformed_serverbound_scalar_packets_disconnect_session() {
    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::Playing;

    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CLIENT_COMMAND_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CLIENT_COMMAND_PACKET_ID, vec![3])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CLIENT_COMMAND_PACKET_ID, vec![1, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CLIENT_TICK_END_PACKET_ID, vec![0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_LOCK_DIFFICULTY_PACKET_ID, vec![1, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PADDLE_BOAT_PACKET_ID, vec![1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PADDLE_BOAT_PACKET_ID, vec![1, 0, 1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_INPUT_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_INPUT_PACKET_ID, vec![0x55, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, vec![0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
            vec![0xac, 0x02, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SWING_PACKET_ID, vec![3])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SWING_PACKET_ID, vec![1, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_COMMAND_PACKET_ID, vec![37, 7, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
            vec![37, 3, 0x80, 0x01, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_INTERACT_PACKET_ID, vec![128, 1, 1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_INTERACT_PACKET_ID,
            vec![128, 1, 1, 0, 1, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_VEHICLE_PACKET_ID, vec![0; 32])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID, vec![0; 24])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_POS_PACKET_ID, vec![0; 26])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
            vec![0; 32]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
            vec![0; 34]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID, vec![0; 8])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID, vec![0; 10])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID,
            Vec::new()
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_MOVE_PLAYER_STATUS_ONLY_PACKET_ID,
            vec![3, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_VEHICLE_PACKET_ID, vec![0; 34])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_ACTION_PACKET_ID, vec![8])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_PLAYER_ACTION_PACKET_ID,
            vec![2, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 4, 0xac, 0x02, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_PACKET_ID, vec![2])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_USE_ITEM_PACKET_ID,
            vec![1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00, 0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_ON_PACKET_ID, vec![2])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_USE_ITEM_ON_PACKET_ID,
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 6]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_USE_ITEM_ON_PACKET_ID,
            vec![
                0, 0xff, 0xff, 0xfd, 0x00, 0x00, 0x02, 0x20, 0x40, 1, 0x3e, 0x80, 0x00,
                0x00, 0x3f, 0x00, 0x00, 0x00, 0x3f, 0x40, 0x00, 0x00, 1, 0, 0xad, 0x02, 0
            ]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHANGE_DIFFICULTY_PACKET_ID, vec![1, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_ACK_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_PACKET_ID, vec![1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID, vec![1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_CONTAINER_CLICK_PACKET_ID, vec![1, 2])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
            vec![0]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID,
            vec![0; 24]
        )),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_RESOURCE_PACK_PACKET_ID, vec![0; 16])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID, vec![1])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID, vec![0; 8])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_EDIT_BOOK_PACKET_ID, vec![0, 101])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_INTERACT_PACKET_ID, vec![1, 0])),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID,
            vec![0; 8]
        )),
        DispatchOutcome::Disconnect(_)
    ));
}

#[test]
fn command_like_packets_wait_for_player_loaded_boundary() {
    let mut session = PlaySession::new(1, 0);
    session.state = PlayState::WaitingForPlayerLoaded;

    for id in [
        SERVERBOUND_CHAT_COMMAND_PACKET_ID,
        SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
        SERVERBOUND_CHAT_PACKET_ID,
        SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
    ] {
        assert!(matches!(
            session.handle_decoded(decoded(id, Vec::new())),
            DispatchOutcome::Disconnect(reason)
                if reason == format!("command packet {id} before player_loaded")
        ));
    }

    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_LOADED_PACKET_ID, Vec::new())),
        DispatchOutcome::Handled
    );
    let mut command_suggestion = Vec::new();
    ServerboundCommandSuggestionPacket {
        id: 7,
        command: "/ti".to_string(),
    }
    .write(&mut command_suggestion)
    .unwrap();
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
            command_suggestion
        )),
        DispatchOutcome::Handled
    );
}

#[test]
fn movement_packets_decode_flags_position_and_rotation_by_shape() {
    let movement = ServerboundMovePlayerPacket {
        x: 1.25,
        y: 65.0,
        z: -2.5,
        y_rot: 90.0,
        x_rot: 30.0,
        on_ground: true,
        horizontal_collision: true,
        has_position: true,
        has_rotation: true,
    };
    let mut payload = Vec::new();
    movement.write_pos_rot(&mut payload).unwrap();

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    let decoded = session.last_move.unwrap();
    assert_eq!(decoded.x, 1.25);
    assert_eq!(decoded.z, -2.5);
    assert_eq!(decoded.y_rot, 90.0);
    assert!(decoded.on_ground);
    assert!(decoded.horizontal_collision);
    assert!(decoded.has_position);
    assert!(decoded.has_rotation);
}

#[test]
fn move_player_packet_shapes_match_vanilla_field_layouts() {
    let movement = ServerboundMovePlayerPacket {
        x: 1.25,
        y: 65.0,
        z: -2.5,
        y_rot: 90.0,
        x_rot: 30.0,
        on_ground: true,
        horizontal_collision: true,
        has_position: true,
        has_rotation: true,
    };

    let mut pos = Vec::new();
    movement.write_pos(&mut pos).unwrap();
    assert_eq!(pos.len(), 25);
    assert_eq!(&pos[0..8], &1.25_f64.to_be_bytes());
    assert_eq!(&pos[8..16], &65.0_f64.to_be_bytes());
    assert_eq!(&pos[16..24], &(-2.5_f64).to_be_bytes());
    assert_eq!(pos[24], 3);
    let decoded_pos =
        ServerboundMovePlayerPacket::read_shape(&mut cursor(pos.clone()), MoveShape::Pos)
            .unwrap();
    assert_eq!(decoded_pos.x, 1.25);
    assert_eq!(decoded_pos.y, 65.0);
    assert_eq!(decoded_pos.z, -2.5);
    assert_eq!(decoded_pos.y_rot, 0.0);
    assert!(decoded_pos.on_ground);
    assert!(decoded_pos.horizontal_collision);
    assert!(decoded_pos.has_position);
    assert!(!decoded_pos.has_rotation);
    let mut truncated_pos = pos.clone();
    truncated_pos.pop();
    assert!(
        ServerboundMovePlayerPacket::read_shape(&mut cursor(truncated_pos), MoveShape::Pos)
            .is_err()
    );
    let mut trailing_pos = pos.clone();
    trailing_pos.push(0);
    assert!(
        ServerboundMovePlayerPacket::read_shape(&mut cursor(trailing_pos), MoveShape::Pos)
            .is_err()
    );
    let mut high_flags_pos = pos.clone();
    high_flags_pos[24] = 0x83;
    let decoded_high_flags =
        ServerboundMovePlayerPacket::read_shape(&mut cursor(high_flags_pos), MoveShape::Pos)
            .unwrap();
    assert!(decoded_high_flags.on_ground);
    assert!(decoded_high_flags.horizontal_collision);

    let mut pos_rot = Vec::new();
    movement.write_pos_rot(&mut pos_rot).unwrap();
    assert_eq!(pos_rot.len(), 33);
    assert_eq!(&pos_rot[0..8], &1.25_f64.to_be_bytes());
    assert_eq!(&pos_rot[24..28], &90.0_f32.to_be_bytes());
    assert_eq!(&pos_rot[28..32], &30.0_f32.to_be_bytes());
    assert_eq!(pos_rot[32], 3);
    let decoded_pos_rot =
        ServerboundMovePlayerPacket::read_shape(&mut cursor(pos_rot.clone()), MoveShape::PosRot)
            .unwrap();
    assert_eq!(decoded_pos_rot.x, 1.25);
    assert_eq!(decoded_pos_rot.y, 65.0);
    assert_eq!(decoded_pos_rot.z, -2.5);
    assert_eq!(decoded_pos_rot.y_rot, 90.0);
    assert_eq!(decoded_pos_rot.x_rot, 30.0);
    assert!(decoded_pos_rot.on_ground);
    assert!(decoded_pos_rot.horizontal_collision);
    assert!(decoded_pos_rot.has_position);
    assert!(decoded_pos_rot.has_rotation);
    let mut truncated_pos_rot = pos_rot.clone();
    truncated_pos_rot.pop();
    assert!(
        ServerboundMovePlayerPacket::read_shape(
            &mut cursor(truncated_pos_rot),
            MoveShape::PosRot
        )
        .is_err()
    );
    let mut trailing_pos_rot = pos_rot.clone();
    trailing_pos_rot.push(0);
    assert!(
        ServerboundMovePlayerPacket::read_shape(
            &mut cursor(trailing_pos_rot),
            MoveShape::PosRot
        )
        .is_err()
    );

    let mut rot = Vec::new();
    movement.write_rot(&mut rot).unwrap();
    assert_eq!(rot.len(), 9);
    assert_eq!(&rot[0..4], &90.0_f32.to_be_bytes());
    assert_eq!(&rot[4..8], &30.0_f32.to_be_bytes());
    assert_eq!(rot[8], 3);
    let decoded_rot =
        ServerboundMovePlayerPacket::read_shape(&mut cursor(rot.clone()), MoveShape::Rot)
            .unwrap();
    assert_eq!(decoded_rot.x, 0.0);
    assert_eq!(decoded_rot.y, 0.0);
    assert_eq!(decoded_rot.z, 0.0);
    assert_eq!(decoded_rot.y_rot, 90.0);
    assert_eq!(decoded_rot.x_rot, 30.0);
    assert!(decoded_rot.on_ground);
    assert!(decoded_rot.horizontal_collision);
    assert!(!decoded_rot.has_position);
    assert!(decoded_rot.has_rotation);
    let mut truncated_rot = rot.clone();
    truncated_rot.pop();
    assert!(
        ServerboundMovePlayerPacket::read_shape(&mut cursor(truncated_rot), MoveShape::Rot)
            .is_err()
    );
    let mut trailing_rot = rot.clone();
    trailing_rot.push(0);
    assert!(
        ServerboundMovePlayerPacket::read_shape(&mut cursor(trailing_rot), MoveShape::Rot)
            .is_err()
    );

    let mut status_only = Vec::new();
    movement.write_status_only(&mut status_only).unwrap();
    assert_eq!(status_only, vec![3]);
    let decoded_status = ServerboundMovePlayerPacket::read_shape(
        &mut cursor(status_only.clone()),
        MoveShape::StatusOnly,
    )
    .unwrap();
    assert_eq!(decoded_status.x, 0.0);
    assert_eq!(decoded_status.y, 0.0);
    assert_eq!(decoded_status.z, 0.0);
    assert_eq!(decoded_status.y_rot, 0.0);
    assert_eq!(decoded_status.x_rot, 0.0);
    assert!(decoded_status.on_ground);
    assert!(decoded_status.horizontal_collision);
    assert!(!decoded_status.has_position);
    assert!(!decoded_status.has_rotation);
    assert!(
        ServerboundMovePlayerPacket::read_shape(&mut cursor(Vec::new()), MoveShape::StatusOnly)
            .is_err()
    );
    let mut trailing_status = status_only.clone();
    trailing_status.push(0);
    assert!(
        ServerboundMovePlayerPacket::read_shape(
            &mut cursor(trailing_status),
            MoveShape::StatusOnly
        )
        .is_err()
    );
    let decoded_high_status =
        ServerboundMovePlayerPacket::read_shape(&mut cursor(vec![0x83]), MoveShape::StatusOnly)
            .unwrap();
    assert!(decoded_high_status.on_ground);
    assert!(decoded_high_status.horizontal_collision);
}

#[test]
fn move_vehicle_packet_matches_vanilla_field_layout() {
    let vehicle = ServerboundMoveVehiclePacket {
        position: Vec3 {
            x: 1.25,
            y: 65.0,
            z: -2.5,
        },
        y_rot: 90.0,
        x_rot: 30.0,
        on_ground: true,
    };
    let mut payload = Vec::new();
    vehicle.write(&mut payload).unwrap();
    assert_eq!(payload.len(), 33);
    assert_eq!(&payload[0..8], &1.25_f64.to_be_bytes());
    assert_eq!(&payload[8..16], &65.0_f64.to_be_bytes());
    assert_eq!(&payload[16..24], &(-2.5_f64).to_be_bytes());
    assert_eq!(&payload[24..28], &90.0_f32.to_be_bytes());
    assert_eq!(&payload[28..32], &30.0_f32.to_be_bytes());
    assert_eq!(payload[32], 1);

    let decoded_vehicle =
        ServerboundMoveVehiclePacket::read(&mut cursor(payload.clone())).unwrap();
    assert_eq!(decoded_vehicle, vehicle);

    let mut truncated = payload.clone();
    truncated.pop();
    assert!(ServerboundMoveVehiclePacket::read(&mut cursor(truncated)).is_err());
    let mut trailing = payload.clone();
    trailing.push(0);
    assert!(ServerboundMoveVehiclePacket::read(&mut cursor(trailing)).is_err());

    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_MOVE_VEHICLE_PACKET_ID, payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_vehicle_move, Some(vehicle));
}

#[test]
fn teleport_ack_and_held_slot_follow_play_state_validation() {
    let mut session = PlaySession::new(1, 0);
    session.pending_teleports.insert(7);

    let mut ack = Vec::new();
    ServerboundAcceptTeleportationPacket { teleport_id: 7 }
        .write(&mut ack)
        .unwrap();
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, ack)),
        DispatchOutcome::Handled
    );
    assert!(session.pending_teleports.is_empty());

    let mut held = Vec::new();
    ServerboundSetCarriedItemPacket { slot: 8 }
        .write(&mut held)
        .unwrap();
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, held)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.selected_slot, 8);

    let mut invalid = Vec::new();
    ServerboundSetCarriedItemPacket { slot: 9 }
        .write(&mut invalid)
        .unwrap();
    assert!(matches!(
        session.handle_decoded(decoded(SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, invalid)),
        DispatchOutcome::Disconnect(_)
    ));
}

#[test]
fn play_session_rejects_wrong_state_or_unknown_packets_and_can_reconfigure() {
    let mut session = PlaySession::new(1, 0);
    let wrong_state = DecodedPacket {
        state: ProtocolState::Configuration,
        direction: PacketDirection::Serverbound,
        id: SERVERBOUND_PLAYER_LOADED_PACKET_ID,
        payload: Vec::new(),
    };
    assert!(matches!(
        session.handle_decoded(wrong_state),
        DispatchOutcome::Disconnect(_)
    ));
    assert!(matches!(
        session.handle_decoded(decoded(999, Vec::new())),
        DispatchOutcome::Disconnect(_)
    ));
    assert_eq!(
        session.request_reconfiguration(),
        PlayInstruction::StartConfiguration
    );
    assert_eq!(session.state, PlayState::Reconfiguring);
}
