use super::super::*;
use super::*;

#[test]
fn small_play_packets_round_trip_vanilla_codecs() {
    let mut bytes = Vec::new();
    ClientboundSetHeldSlotPacket { slot: 4 }
        .write(&mut bytes)
        .unwrap();
    assert_eq!(
        ClientboundSetHeldSlotPacket::read(&mut cursor(bytes)).unwrap(),
        ClientboundSetHeldSlotPacket { slot: 4 }
    );

    let mut carried = Vec::new();
    ServerboundSetCarriedItemPacket { slot: 5 }
        .write(&mut carried)
        .unwrap();
    assert_eq!(
        ServerboundSetCarriedItemPacket::read(&mut cursor(carried)).unwrap(),
        ServerboundSetCarriedItemPacket { slot: 5 }
    );

    let mut server_command = Vec::new();
    ServerboundChangeDifficultyPacket {
        difficulty: GameDifficulty::Easy,
    }
    .write(&mut server_command)
    .unwrap();
    assert_eq!(
        ServerboundChangeDifficultyPacket::read(&mut cursor(server_command)).unwrap(),
        ServerboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Easy,
        }
    );

    let mut client_command = Vec::new();
    ServerboundClientCommandPacket {
        action: ServerboundClientCommandAction::RequestStats,
    }
    .write(&mut client_command)
    .unwrap();
    assert_eq!(client_command, vec![1]);
    let parsed_client_command =
        ServerboundClientCommandPacket::read(&mut cursor(client_command)).unwrap();
    assert!(matches!(
        parsed_client_command.action,
        ServerboundClientCommandAction::RequestStats
    ));

    let mut client_tick_end = Vec::new();
    ServerboundClientTickEndPacket
        .write(&mut client_tick_end)
        .unwrap();
    assert_eq!(
        ServerboundClientTickEndPacket::read(&mut cursor(client_tick_end)).unwrap(),
        ServerboundClientTickEndPacket
    );

    let mut lock_difficulty = Vec::new();
    ServerboundLockDifficultyPacket { locked: true }
        .write(&mut lock_difficulty)
        .unwrap();
    assert_eq!(
        ServerboundLockDifficultyPacket::read(&mut cursor(lock_difficulty)).unwrap(),
        ServerboundLockDifficultyPacket { locked: true }
    );

    let mut paddle_boat = Vec::new();
    ServerboundPaddleBoatPacket {
        left: true,
        right: false,
    }
    .write(&mut paddle_boat)
    .unwrap();
    assert_eq!(
        ServerboundPaddleBoatPacket::read(&mut cursor(paddle_boat)).unwrap(),
        ServerboundPaddleBoatPacket {
            left: true,
            right: false,
        }
    );

    let mut player_input = Vec::new();
    ServerboundPlayerInputPacket {
        input: ServerboundPlayerInput {
            forward: true,
            backward: false,
            left: true,
            right: false,
            jump: true,
            shift: false,
            sprint: true,
        },
    }
    .write(&mut player_input)
    .unwrap();
    assert_eq!(
        ServerboundPlayerInputPacket::read(&mut cursor(player_input)).unwrap(),
        ServerboundPlayerInputPacket {
            input: ServerboundPlayerInput {
                forward: true,
                backward: false,
                left: true,
                right: false,
                jump: true,
                shift: false,
                sprint: true,
            },
        }
    );

    let mut player_command = Vec::new();
    ServerboundPlayerCommandPacket {
        entity_id: 37,
        action: ServerboundPlayerCommandAction::StartRidingJump,
        data: 128,
    }
    .write(&mut player_command)
    .unwrap();
    assert_eq!(player_command, vec![37, 3, 0x80, 0x01]);
    assert_eq!(
        ServerboundPlayerCommandPacket::read(&mut cursor(player_command.clone())).unwrap(),
        ServerboundPlayerCommandPacket {
            entity_id: 37,
            action: ServerboundPlayerCommandAction::StartRidingJump,
            data: 128,
        }
    );
    let mut session = PlaySession::new(1, 0);
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_PLAYER_COMMAND_PACKET_ID,
            player_command
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_player_command,
        Some(ServerboundPlayerCommandPacket {
            entity_id: 37,
            action: ServerboundPlayerCommandAction::StartRidingJump,
            data: 128,
        })
    );

    let mut player_action = Vec::new();
    ServerboundPlayerActionPacket {
        action: ServerboundPlayerAction::StopDestroyBlock,
        x: -12,
        y: 64,
        z: 34,
        direction: Direction3d::West,
        sequence: 300,
    }
    .write(&mut player_action)
    .unwrap();
    assert_eq!(player_action[0], 2);
    assert_eq!(player_action[9], 4);
    assert_eq!(&player_action[10..], &[0xac, 0x02]);
    let parsed_action =
        ServerboundPlayerActionPacket::read(&mut cursor(player_action.clone())).unwrap();
    assert_eq!(
        parsed_action,
        ServerboundPlayerActionPacket {
            action: ServerboundPlayerAction::StopDestroyBlock,
            x: -12,
            y: 64,
            z: 34,
            direction: Direction3d::West,
            sequence: 300,
        }
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_PLAYER_ACTION_PACKET_ID, player_action)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_player_action, Some(parsed_action));

    let mut player_loaded = Vec::new();
    ServerboundPlayerLoadedPacket
        .write(&mut player_loaded)
        .unwrap();
    assert_eq!(
        ServerboundPlayerLoadedPacket::read(&mut cursor(player_loaded)).unwrap(),
        ServerboundPlayerLoadedPacket
    );

    let mut swing = Vec::new();
    ServerboundSwingPacket {
        hand: ServerboundSwingHand::OffHand,
    }
    .write(&mut swing)
    .unwrap();
    assert_eq!(swing, vec![1]);
    assert_eq!(
        ServerboundSwingPacket::read(&mut cursor(swing)).unwrap(),
        ServerboundSwingPacket {
            hand: ServerboundSwingHand::OffHand,
        }
    );

    let use_item = ServerboundUseItemPacket {
        hand: ServerboundSwingHand::OffHand,
        sequence: 300,
        y_rot: 45.0,
        x_rot: -10.5,
    };
    let mut use_item_payload = Vec::new();
    use_item.write(&mut use_item_payload).unwrap();
    assert_eq!(
        use_item_payload,
        vec![1, 0xac, 0x02, 0x42, 0x34, 0x00, 0x00, 0xc1, 0x28, 0x00, 0x00]
    );
    assert_eq!(
        ServerboundUseItemPacket::read(&mut cursor(use_item_payload.clone())).unwrap(),
        use_item
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_USE_ITEM_PACKET_ID, use_item_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_use_item, Some(use_item));

    let use_item_on = ServerboundUseItemOnPacket {
        hand: ServerboundSwingHand::MainHand,
        block_hit: BlockHitResultPacketData {
            x: -12,
            y: 64,
            z: 34,
            direction: Direction3d::Up,
            click_x: 0.25,
            click_y: 0.5,
            click_z: 0.75,
            inside: true,
            world_border_hit: false,
        },
        sequence: 301,
    };
    let mut use_item_on_payload = Vec::new();
    use_item_on.write(&mut use_item_on_payload).unwrap();
    assert_eq!(use_item_on_payload[0], 0);
    assert_eq!(use_item_on_payload[9], 1);
    assert_eq!(&use_item_on_payload[22..24], &[1, 0]);
    assert_eq!(&use_item_on_payload[24..], &[0xad, 0x02]);
    assert_eq!(
        ServerboundUseItemOnPacket::read(&mut cursor(use_item_on_payload.clone())).unwrap(),
        use_item_on
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_USE_ITEM_ON_PACKET_ID,
            use_item_on_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_use_item_on, Some(use_item_on));

    let mut pong = Vec::new();
    ServerboundPongPacket { id: 0x01020304 }
        .write(&mut pong)
        .unwrap();
    assert_eq!(pong, vec![1, 2, 3, 4]);
    assert_eq!(
        ServerboundPongPacket::read(&mut cursor(pong.clone())).unwrap(),
        ServerboundPongPacket { id: 0x01020304 }
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_PONG_PACKET_ID, pong)),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_pong,
        Some(ServerboundPongPacket { id: 0x01020304 })
    );

    let mut ping = Vec::new();
    ClientboundPingPacket { id: -0x01020304 }
        .write(&mut ping)
        .unwrap();
    assert_eq!(ping, (-0x01020304_i32).to_be_bytes());
    assert_eq!(
        ClientboundPingPacket::read(&mut cursor(ping)).unwrap(),
        ClientboundPingPacket { id: -0x01020304 }
    );

    let mut configuration_ack = Vec::new();
    ServerboundConfigurationAcknowledgedPacket
        .write(&mut configuration_ack)
        .unwrap();
    assert!(configuration_ack.is_empty());
    assert_eq!(
        ServerboundConfigurationAcknowledgedPacket::read(&mut cursor(
            configuration_ack.clone()
        ))
        .unwrap(),
        ServerboundConfigurationAcknowledgedPacket
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
            configuration_ack
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.state, PlayState::Reconfiguring);

    let mut jigsaw_generate = Vec::new();
    let jigsaw_packet = ServerboundJigsawGeneratePacket {
        x: -12,
        y: 64,
        z: 34,
        levels: 7,
        keep_jigsaws: true,
    };
    jigsaw_packet.write(&mut jigsaw_generate).unwrap();
    assert_eq!(jigsaw_generate.len(), 10);
    assert_eq!(&jigsaw_generate[8..], &[7, 1]);
    assert_eq!(
        ServerboundJigsawGeneratePacket::read(&mut cursor(jigsaw_generate.clone())).unwrap(),
        jigsaw_packet
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_JIGSAW_GENERATE_PACKET_ID,
            jigsaw_generate
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_jigsaw_generate, Some(jigsaw_packet));

    let sign_update = ServerboundSignUpdatePacket {
        x: -12,
        y: 64,
        z: 34,
        is_front_text: false,
        lines: [
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
            "four".to_string(),
        ],
    };
    let mut sign_payload = Vec::new();
    sign_update.write(&mut sign_payload).unwrap();
    assert_eq!(sign_payload[8], 0);
    assert_eq!(
        ServerboundSignUpdatePacket::read(&mut cursor(sign_payload.clone())).unwrap(),
        sign_update
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_SIGN_UPDATE_PACKET_ID, sign_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_sign_update, Some(sign_update));

    let set_beacon = ServerboundSetBeaconPacket {
        primary_effect_id: Some(1),
        secondary_effect_id: Some(39),
    };
    let mut set_beacon_payload = Vec::new();
    set_beacon.write(&mut set_beacon_payload).unwrap();
    assert_eq!(set_beacon_payload, vec![1, 1, 1, 39]);
    assert_eq!(
        ServerboundSetBeaconPacket::read(&mut cursor(set_beacon_payload.clone())).unwrap(),
        set_beacon
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_BEACON_PACKET_ID,
            set_beacon_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_set_beacon, Some(set_beacon));

    let mut empty_beacon_payload = Vec::new();
    ServerboundSetBeaconPacket {
        primary_effect_id: None,
        secondary_effect_id: None,
    }
    .write(&mut empty_beacon_payload)
    .unwrap();
    assert_eq!(empty_beacon_payload, vec![0, 0]);
    assert!(ServerboundSetBeaconPacket::read(&mut cursor(vec![1, 40, 0])).is_err());
    assert!(ServerboundSetBeaconPacket {
        primary_effect_id: Some(40),
        secondary_effect_id: None,
    }
    .write(&mut Vec::new())
    .is_err());

    let mut select_trade_payload = Vec::new();
    let select_trade = ServerboundSelectTradePacket { item: 128 };
    select_trade.write(&mut select_trade_payload).unwrap();
    assert_eq!(select_trade_payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundSelectTradePacket::read(&mut cursor(select_trade_payload.clone())).unwrap(),
        select_trade
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SELECT_TRADE_PACKET_ID,
            select_trade_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_select_trade, Some(select_trade));

    let rename_item = ServerboundRenameItemPacket {
        name: "Sharp Thing".to_string(),
    };
    let mut rename_payload = Vec::new();
    rename_item.write(&mut rename_payload).unwrap();
    assert_eq!(rename_payload[0], 11);
    assert_eq!(
        ServerboundRenameItemPacket::read(&mut cursor(rename_payload.clone())).unwrap(),
        rename_item
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_RENAME_ITEM_PACKET_ID, rename_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_rename_item, Some(rename_item));
    assert!(ServerboundRenameItemPacket {
        name: "a".repeat(32768),
    }
    .write(&mut Vec::new())
    .is_err());
    let mut oversized_rename_payload = Vec::new();
    write_var_i32(&mut oversized_rename_payload, 32768).unwrap();
    oversized_rename_payload.extend(vec![b'a'; 32768]);
    assert!(ServerboundRenameItemPacket::read(&mut cursor(oversized_rename_payload)).is_err());

    let container_close = ServerboundContainerClosePacket { container_id: 128 };
    let mut container_close_payload = Vec::new();
    container_close.write(&mut container_close_payload).unwrap();
    assert_eq!(container_close_payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundContainerClosePacket::read(&mut cursor(container_close_payload.clone()))
            .unwrap(),
        container_close
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONTAINER_CLOSE_PACKET_ID,
            container_close_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_container_close, Some(container_close));

    let container_button_click = ServerboundContainerButtonClickPacket {
        container_id: 128,
        button_id: 7,
    };
    let mut button_click_payload = Vec::new();
    container_button_click
        .write(&mut button_click_payload)
        .unwrap();
    assert_eq!(button_click_payload, vec![0x80, 0x01, 7]);
    assert_eq!(
        ServerboundContainerButtonClickPacket::read(&mut cursor(button_click_payload.clone()))
            .unwrap(),
        container_button_click
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONTAINER_BUTTON_CLICK_PACKET_ID,
            button_click_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_container_button_click,
        Some(container_button_click)
    );

    let creative_slot = ServerboundSetCreativeModeSlotPacket {
        slot_num: -1,
        item_stack: RawItemStack {
            count: 3,
            item_id: Some(42),
            components: RawDataComponentPatch {
                added: vec![(7, vec![0xaa, 0xbb])],
                removed: vec![9],
            },
        },
    };
    let mut creative_slot_payload = Vec::new();
    creative_slot.write(&mut creative_slot_payload).unwrap();
    assert_eq!(
        creative_slot_payload,
        vec![0xff, 0xff, 3, 42, 1, 1, 7, 2, 0xaa, 0xbb, 9]
    );
    assert_eq!(
        ServerboundSetCreativeModeSlotPacket::read(&mut cursor(creative_slot_payload.clone()))
            .unwrap(),
        creative_slot
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
            creative_slot_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_set_creative_mode_slot, Some(creative_slot));

    let mut changed_slots = BTreeMap::new();
    changed_slots.insert(
        5,
        HashedStack {
            item_id: Some(42),
            count: 3,
            components: HashedPatchMap {
                added_component_hashes: vec![(7, 0x01020304)],
                removed_components: vec![9],
            },
        },
    );
    let container_click = ServerboundContainerClickPacket {
        container_id: 1,
        state_id: 2,
        slot_num: -1,
        button_num: -2,
        container_input: ContainerInput::Throw,
        changed_slots,
        carried_item: HashedStack::empty(),
    };
    let mut container_click_payload = Vec::new();
    container_click.write(&mut container_click_payload).unwrap();
    assert_eq!(
        container_click_payload,
        vec![1, 2, 0xff, 0xff, 0xfe, 4, 1, 0, 5, 1, 42, 3, 1, 7, 1, 2, 3, 4, 1, 9, 0]
    );
    assert_eq!(
        ServerboundContainerClickPacket::read(&mut cursor(container_click_payload.clone()))
            .unwrap(),
        container_click
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
            container_click_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_container_click, Some(container_click));
    let mut too_many_changed_slots = vec![1, 2, 0, 0, 0, 0];
    write_var_i32(&mut too_many_changed_slots, 129).unwrap();
    assert!(
        ServerboundContainerClickPacket::read(&mut cursor(too_many_changed_slots)).is_err()
    );
    assert!(ServerboundContainerClickPacket {
        container_id: 0,
        state_id: 0,
        slot_num: 0,
        button_num: 0,
        container_input: ContainerInput::Pickup,
        changed_slots: (0..129).map(|slot| (slot, HashedStack::empty())).collect(),
        carried_item: HashedStack::empty(),
    }
    .write(&mut Vec::new())
    .is_err());

    let edit_book = ServerboundEditBookPacket {
        slot: 1,
        pages: vec!["page one".to_string(), "page two".to_string()],
        title: Some("Title".to_string()),
    };
    let mut edit_book_payload = Vec::new();
    edit_book.write(&mut edit_book_payload).unwrap();
    assert_eq!(
        edit_book_payload,
        vec![
            1, 2, 8, b'p', b'a', b'g', b'e', b' ', b'o', b'n', b'e', 8, b'p', b'a', b'g', b'e',
            b' ', b't', b'w', b'o', 1, 5, b'T', b'i', b't', b'l', b'e'
        ]
    );
    assert_eq!(
        ServerboundEditBookPacket::read(&mut cursor(edit_book_payload.clone())).unwrap(),
        edit_book
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_EDIT_BOOK_PACKET_ID, edit_book_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_edit_book, Some(edit_book));
    assert!(ServerboundEditBookPacket {
        slot: 0,
        pages: vec!["x".to_string(); 101],
        title: None,
    }
    .write(&mut Vec::new())
    .is_err());

    let interact = ServerboundInteractPacket {
        entity_id: 128,
        hand: ServerboundInteractionHand::OffHand,
        location: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        using_secondary_action: true,
    };
    let mut interact_payload = Vec::new();
    interact.write(&mut interact_payload).unwrap();
    assert_eq!(interact_payload, vec![0x80, 0x01, 1, 0, 1]);
    assert_eq!(
        ServerboundInteractPacket::read(&mut cursor(interact_payload.clone())).unwrap(),
        interact
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_INTERACT_PACKET_ID, interact_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_interact, Some(interact));
    let invalid_hand_payload = vec![1, 7, 0, 0];
    assert_eq!(
        ServerboundInteractPacket::read(&mut cursor(invalid_hand_payload)).unwrap(),
        ServerboundInteractPacket {
            entity_id: 1,
            hand: ServerboundInteractionHand::MainHand,
            location: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            using_secondary_action: false,
        }
    );

    let chat_ack = ServerboundChatAckPacket { offset: 128 };
    let mut chat_ack_payload = Vec::new();
    chat_ack.write(&mut chat_ack_payload).unwrap();
    assert_eq!(chat_ack_payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundChatAckPacket::read(&mut cursor(chat_ack_payload.clone())).unwrap(),
        chat_ack
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_ACK_PACKET_ID, chat_ack_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_ack, Some(chat_ack));

    let last_seen = LastSeenMessagesUpdate {
        offset: 2,
        acknowledged: vec![0b1010_0001, 0, 0b0000_1000],
        checksum: 5,
    };
    let chat = ServerboundChatPacket {
        message: "hi".to_string(),
        timestamp_epoch_millis: 100,
        salt: -7,
        signature: Some(MessageSignature([7; MessageSignature::BYTES])),
        last_seen_messages: last_seen.clone(),
    };
    let mut chat_payload = Vec::new();
    chat.write(&mut chat_payload).unwrap();
    assert_eq!(&chat_payload[..2], &[2, b'h']);
    assert_eq!(chat_payload[2], b'i');
    assert_eq!(&chat_payload[3..11], &100_i64.to_be_bytes());
    assert_eq!(&chat_payload[11..19], &(-7_i64).to_be_bytes());
    assert_eq!(chat_payload[19], 1);
    assert_eq!(&chat_payload[276..], &[2, 0b1010_0001, 0, 0b0000_1000, 5]);
    assert_eq!(
        ServerboundChatPacket::read(&mut cursor(chat_payload.clone())).unwrap(),
        chat
    );
    assert_eq!(
        session.handle_decoded(decoded(SERVERBOUND_CHAT_PACKET_ID, chat_payload)),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat, Some(chat));

    let chat_command = ServerboundChatCommandPacket {
        command: "seed".to_string(),
    };
    let mut chat_command_payload = Vec::new();
    chat_command.write(&mut chat_command_payload).unwrap();
    assert_eq!(chat_command_payload, vec![4, b's', b'e', b'e', b'd']);
    assert_eq!(
        ServerboundChatCommandPacket::read(&mut cursor(chat_command_payload.clone())).unwrap(),
        chat_command
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CHAT_COMMAND_PACKET_ID,
            chat_command_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_command, Some(chat_command));

    let signed_command = ServerboundChatCommandSignedPacket {
        command: "msg Notch hello".to_string(),
        timestamp_epoch_millis: 101,
        salt: 9,
        argument_signatures: vec![ArgumentSignature {
            name: "message".to_string(),
            signature: MessageSignature([8; MessageSignature::BYTES]),
        }],
        last_seen_messages: last_seen,
    };
    let mut signed_command_payload = Vec::new();
    signed_command.write(&mut signed_command_payload).unwrap();
    assert_eq!(signed_command_payload[0], 15);
    assert_eq!(&signed_command_payload[16..24], &101_i64.to_be_bytes());
    assert_eq!(&signed_command_payload[24..32], &9_i64.to_be_bytes());
    assert_eq!(signed_command_payload[32], 1);
    assert_eq!(
        &signed_command_payload[33..41],
        &[7, b'm', b'e', b's', b's', b'a', b'g', b'e']
    );
    assert_eq!(
        &signed_command_payload[signed_command_payload.len() - 5..],
        &[2, 0b1010_0001, 0, 0b0000_1000, 5]
    );
    assert_eq!(
        ServerboundChatCommandSignedPacket::read(&mut cursor(signed_command_payload.clone()))
            .unwrap(),
        signed_command
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID,
            signed_command_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_signed_chat_command, Some(signed_command));
    assert!(LastSeenMessagesUpdate {
        offset: 0,
        acknowledged: vec![0, 0, 0x10],
        checksum: 0,
    }
    .write(&mut Vec::new())
    .is_err());
    assert!(ServerboundChatCommandSignedPacket {
        command: String::new(),
        timestamp_epoch_millis: 0,
        salt: 0,
        argument_signatures: vec![
            ArgumentSignature {
                name: String::new(),
                signature: MessageSignature([0; MessageSignature::BYTES]),
            };
            9
        ],
        last_seen_messages: LastSeenMessagesUpdate {
            offset: 0,
            acknowledged: vec![0, 0, 0],
            checksum: 0,
        },
    }
    .write(&mut Vec::new())
    .is_err());

    let chat_session_update = ServerboundChatSessionUpdatePacket {
        session_id: Uuid([4; 16]),
        expires_at_epoch_millis: 1_234_567_890,
        public_key: vec![1, 2, 3],
        key_signature: vec![4, 5],
    };
    let mut chat_session_payload = Vec::new();
    chat_session_update
        .write(&mut chat_session_payload)
        .unwrap();
    let mut expected_chat_session = vec![4; 16];
    expected_chat_session.extend_from_slice(&1_234_567_890_i64.to_be_bytes());
    expected_chat_session.extend_from_slice(&[3, 1, 2, 3, 2, 4, 5]);
    assert_eq!(chat_session_payload, expected_chat_session);
    assert_eq!(
        ServerboundChatSessionUpdatePacket::read(&mut cursor(chat_session_payload.clone()))
            .unwrap(),
        chat_session_update
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID,
            chat_session_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_chat_session_update, Some(chat_session_update));
    assert!(ServerboundChatSessionUpdatePacket {
        session_id: Uuid([0; 16]),
        expires_at_epoch_millis: 0,
        public_key: vec![0; 513],
        key_signature: Vec::new(),
    }
    .write(&mut Vec::new())
    .is_err());

    let resource_pack_response = ServerboundResourcePackPacket {
        id: Uuid([9; 16]),
        action: crate::network::common::ResourcePackAction::Accepted,
    };
    let mut resource_pack_payload = Vec::new();
    resource_pack_response
        .write(&mut resource_pack_payload)
        .unwrap();
    let mut expected_resource_pack = vec![9; 16];
    expected_resource_pack.push(3);
    assert_eq!(resource_pack_payload, expected_resource_pack);
    assert_eq!(
        ServerboundResourcePackPacket::read(&mut cursor(resource_pack_payload.clone()))
            .unwrap(),
        resource_pack_response
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_RESOURCE_PACK_PACKET_ID,
            resource_pack_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_resource_pack_response,
        Some(resource_pack_response)
    );

    let command_block = ServerboundSetCommandBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        command: "say hi".to_string(),
        mode: CommandBlockMode::Redstone,
        track_output: true,
        conditional: false,
        automatic: true,
    };
    let mut command_block_payload = Vec::new();
    command_block.write(&mut command_block_payload).unwrap();
    assert_eq!(command_block_payload.len(), 17);
    assert_eq!(
        &command_block_payload[8..],
        &[6, b's', b'a', b'y', b' ', b'h', b'i', 2, 5]
    );
    assert_eq!(
        ServerboundSetCommandBlockPacket::read(&mut cursor(command_block_payload.clone()))
            .unwrap(),
        command_block
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID,
            command_block_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_set_command_block, Some(command_block));
    assert!(ServerboundSetCommandBlockPacket::read(&mut cursor(vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0
    ]))
    .is_err());

    let structure_block = ServerboundSetStructureBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        update_type: StructureBlockUpdateType::LoadArea,
        mode: StructureBlockMode::Load,
        name: "demo:house".to_string(),
        offset: [-2, 3, 4],
        size: [5, 6, 7],
        mirror: StructureMirror::FrontBack,
        rotation: StructureRotation::Counterclockwise90,
        data: "metadata".to_string(),
        integrity: 0.75,
        seed: 128,
        ignore_entities: true,
        strict: true,
        show_air: false,
        show_bounding_box: true,
    };
    let mut structure_block_payload = Vec::new();
    structure_block.write(&mut structure_block_payload).unwrap();
    assert_eq!(
        &structure_block_payload[8..],
        &[
            2, 1, 10, b'd', b'e', b'm', b'o', b':', b'h', b'o', b'u', b's', b'e', 0xfe, 3, 4,
            5, 6, 7, 2, 3, 8, b'm', b'e', b't', b'a', b'd', b'a', b't', b'a', 0x3f, 0x40, 0, 0,
            0x80, 0x01, 13
        ]
    );
    assert_eq!(
        ServerboundSetStructureBlockPacket::read(&mut cursor(structure_block_payload.clone()))
            .unwrap(),
        structure_block
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID,
            structure_block_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_set_structure_block, Some(structure_block));

    let clamped_structure = ServerboundSetStructureBlockPacket::read(&mut cursor(vec![
        0, 0, 0, 0, 0, 0, 0, 0, // BlockPos
        0, 0, 0, // update type, mode, empty name
        200, 60, 255, // offset clamps to -48, 48, -1
        255, 60, 10, // size clamps to 0, 48, 10
        0, 3, 0, // mirror, rotation, empty data
        0x3f, 0xc0, 0, 0, // integrity 1.5 clamps to 1.0
        0, 15,
    ]))
    .unwrap();
    assert_eq!(clamped_structure.offset, [-48, 48, -1]);
    assert_eq!(clamped_structure.size, [0, 48, 10]);
    assert_eq!(
        clamped_structure.rotation,
        StructureRotation::Counterclockwise90
    );
    assert_eq!(clamped_structure.integrity, 1.0);
    assert!(clamped_structure.ignore_entities);
    assert!(clamped_structure.strict);
    assert!(clamped_structure.show_air);
    assert!(clamped_structure.show_bounding_box);

    let command_minecart = ServerboundSetCommandMinecartPacket {
        entity_id: 128,
        command: "say hi".to_string(),
        track_output: true,
    };
    let mut command_minecart_payload = Vec::new();
    command_minecart
        .write(&mut command_minecart_payload)
        .unwrap();
    assert_eq!(
        command_minecart_payload,
        vec![0x80, 0x01, 6, b's', b'a', b'y', b' ', b'h', b'i', 1]
    );
    assert_eq!(
        ServerboundSetCommandMinecartPacket::read(&mut cursor(
            command_minecart_payload.clone()
        ))
        .unwrap(),
        command_minecart
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID,
            command_minecart_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_set_command_minecart, Some(command_minecart));

    let command_suggestion = ServerboundCommandSuggestionPacket {
        id: 128,
        command: "/time set day".to_string(),
    };
    let mut command_suggestion_payload = Vec::new();
    command_suggestion
        .write(&mut command_suggestion_payload)
        .unwrap();
    assert_eq!(
        command_suggestion_payload,
        vec![
            0x80, 0x01, 13, b'/', b't', b'i', b'm', b'e', b' ', b's', b'e', b't', b' ', b'd',
            b'a', b'y'
        ]
    );
    assert_eq!(
        ServerboundCommandSuggestionPacket::read(&mut cursor(
            command_suggestion_payload.clone()
        ))
        .unwrap(),
        command_suggestion
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_COMMAND_SUGGESTION_PACKET_ID,
            command_suggestion_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_command_suggestion, Some(command_suggestion));

    let pick_item_from_block = ServerboundPickItemFromBlockPacket {
        x: -12,
        y: 64,
        z: 34,
        include_data: true,
    };
    let mut pick_block_payload = Vec::new();
    pick_item_from_block.write(&mut pick_block_payload).unwrap();
    assert_eq!(pick_block_payload.len(), 9);
    assert_eq!(pick_block_payload[8], 1);
    assert_eq!(
        ServerboundPickItemFromBlockPacket::read(&mut cursor(pick_block_payload.clone()))
            .unwrap(),
        pick_item_from_block
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_PICK_ITEM_FROM_BLOCK_PACKET_ID,
            pick_block_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_pick_item_from_block,
        Some(pick_item_from_block)
    );

    let pick_item_from_entity = ServerboundPickItemFromEntityPacket {
        entity_id: 128,
        include_data: false,
    };
    let mut pick_entity_payload = Vec::new();
    pick_item_from_entity
        .write(&mut pick_entity_payload)
        .unwrap();
    assert_eq!(pick_entity_payload, vec![0x80, 0x01, 0]);
    assert_eq!(
        ServerboundPickItemFromEntityPacket::read(&mut cursor(pick_entity_payload.clone()))
            .unwrap(),
        pick_item_from_entity
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_PICK_ITEM_FROM_ENTITY_PACKET_ID,
            pick_entity_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_pick_item_from_entity,
        Some(pick_item_from_entity)
    );

    let recipe_book_settings = ServerboundRecipeBookChangeSettingsPacket {
        book_type: RecipeBookType::BlastFurnace,
        is_open: true,
        is_filtering: false,
    };
    let mut recipe_book_settings_payload = Vec::new();
    recipe_book_settings
        .write(&mut recipe_book_settings_payload)
        .unwrap();
    assert_eq!(recipe_book_settings_payload, vec![2, 1, 0]);
    assert_eq!(
        ServerboundRecipeBookChangeSettingsPacket::read(&mut cursor(
            recipe_book_settings_payload.clone()
        ))
        .unwrap(),
        recipe_book_settings
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS_PACKET_ID,
            recipe_book_settings_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(
        session.last_recipe_book_change_settings,
        Some(recipe_book_settings)
    );
    assert!(
        ServerboundRecipeBookChangeSettingsPacket::read(&mut cursor(vec![4, 0, 0])).is_err()
    );

    let seen_recipe = ServerboundRecipeBookSeenRecipePacket { recipe_index: 128 };
    let mut seen_recipe_payload = Vec::new();
    seen_recipe.write(&mut seen_recipe_payload).unwrap();
    assert_eq!(seen_recipe_payload, vec![0x80, 0x01]);
    assert_eq!(
        ServerboundRecipeBookSeenRecipePacket::read(&mut cursor(seen_recipe_payload.clone()))
            .unwrap(),
        seen_recipe
    );
    assert_eq!(
        session.handle_decoded(decoded(
            SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE_PACKET_ID,
            seen_recipe_payload
        )),
        DispatchOutcome::Handled
    );
    assert_eq!(session.last_recipe_book_seen_recipe, Some(seen_recipe));

    let mut change_difficulty = Vec::new();
    ClientboundChangeDifficultyPacket {
        difficulty: GameDifficulty::Hard,
        locked: true,
    }
    .write(&mut change_difficulty)
    .unwrap();
    assert_eq!(
        ClientboundChangeDifficultyPacket::read(&mut cursor(change_difficulty)).unwrap(),
        ClientboundChangeDifficultyPacket {
            difficulty: GameDifficulty::Hard,
            locked: true,
        }
    );

    let mut chunk_cache_center = Vec::new();
    ClientboundSetChunkCacheCenterPacket { x: 12, z: -34 }
        .write(&mut chunk_cache_center)
        .unwrap();
    assert_eq!(
        ClientboundSetChunkCacheCenterPacket::read(&mut cursor(chunk_cache_center)).unwrap(),
        ClientboundSetChunkCacheCenterPacket { x: 12, z: -34 }
    );

    let mut chunk_cache_radius = Vec::new();
    ClientboundSetChunkCacheRadiusPacket { radius: 5 }
        .write(&mut chunk_cache_radius)
        .unwrap();
    assert_eq!(
        ClientboundSetChunkCacheRadiusPacket::read(&mut cursor(chunk_cache_radius)).unwrap(),
        ClientboundSetChunkCacheRadiusPacket { radius: 5 }
    );

    let mut spawn_position = Vec::new();
    ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: ClientboundSetDefaultSpawnPositionData {
            dimension: Identifier::parse("minecraft:the_end").unwrap(),
            x: 1,
            y: 2,
            z: 3,
            yaw: 45.0,
            pitch: -23.5,
        },
    }
    .write(&mut spawn_position)
    .unwrap();
    assert_eq!(
        ClientboundSetDefaultSpawnPositionPacket::read(&mut cursor(spawn_position)).unwrap(),
        ClientboundSetDefaultSpawnPositionPacket {
            respawn_data: ClientboundSetDefaultSpawnPositionData {
                dimension: Identifier::parse("minecraft:the_end").unwrap(),
                x: 1,
                y: 2,
                z: 3,
                yaw: 45.0,
                pitch: -23.5,
            },
        }
    );

    let mut experience = Vec::new();
    ClientboundSetExperiencePacket {
        experience_progress: 0.75,
        experience_level: 3,
        total_experience: 42,
    }
    .write(&mut experience)
    .unwrap();
    assert_eq!(
        ClientboundSetExperiencePacket::read(&mut cursor(experience)).unwrap(),
        ClientboundSetExperiencePacket {
            experience_progress: 0.75,
            experience_level: 3,
            total_experience: 42,
        }
    );

    let mut health = Vec::new();
    ClientboundSetHealthPacket {
        health: 14.5,
        food: 19,
        saturation: 2.3,
    }
    .write(&mut health)
    .unwrap();
    assert_eq!(
        ClientboundSetHealthPacket::read(&mut cursor(health)).unwrap(),
        ClientboundSetHealthPacket {
            health: 14.5,
            food: 19,
            saturation: 2.3,
        }
    );

    let mut set_time = Vec::new();
    ClientboundSetTimePacket {
        game_time: 900_000,
        clock_updates: BTreeMap::from([(
            OVERWORLD_CLOCK_ID,
            ClockNetworkState {
                total_ticks: 12345,
                partial_tick: 0.25,
                rate: 1.0,
            },
        )]),
    }
    .write(&mut set_time)
    .unwrap();
    assert_eq!(
        ClientboundSetTimePacket::read(&mut cursor(set_time)).unwrap(),
        ClientboundSetTimePacket {
            game_time: 900_000,
            clock_updates: BTreeMap::from([(
                OVERWORLD_CLOCK_ID,
                ClockNetworkState {
                    total_ticks: 12345,
                    partial_tick: 0.25,
                    rate: 1.0,
                },
            )]),
        }
    );

    let mut game_event = Vec::new();
    ClientboundGameEventPacket {
        event: ClientboundGameEventType::RainLevelChange,
        param: 0.5,
    }
    .write(&mut game_event)
    .unwrap();
    assert_eq!(
        ClientboundGameEventPacket::read(&mut cursor(game_event)).unwrap(),
        ClientboundGameEventPacket {
            event: ClientboundGameEventType::RainLevelChange,
            param: 0.5,
        }
    );

    let mut simulation = Vec::new();
    ClientboundSetSimulationDistancePacket {
        simulation_distance: 8,
    }
    .write(&mut simulation)
    .unwrap();
    assert_eq!(
        ClientboundSetSimulationDistancePacket::read(&mut cursor(simulation)).unwrap(),
        ClientboundSetSimulationDistancePacket {
            simulation_distance: 8
        }
    );

    let mut ticking_state = Vec::new();
    ClientboundTickingStatePacket {
        tick_rate: 0.5,
        is_frozen: true,
    }
    .write(&mut ticking_state)
    .unwrap();
    assert_eq!(
        ClientboundTickingStatePacket::read(&mut cursor(ticking_state)).unwrap(),
        ClientboundTickingStatePacket {
            tick_rate: 0.5,
            is_frozen: true,
        }
    );

    let mut ticking_step = Vec::new();
    ClientboundTickingStepPacket { tick_steps: 7 }
        .write(&mut ticking_step)
        .unwrap();
    assert_eq!(
        ClientboundTickingStepPacket::read(&mut cursor(ticking_step)).unwrap(),
        ClientboundTickingStepPacket { tick_steps: 7 }
    );
}
