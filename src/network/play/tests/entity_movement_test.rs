use super::super::*;
use super::*;

#[test]
fn entity_movement_mount_link_and_animation_packets_capture_vanilla_shapes() {
    assert_eq!(
        ClientboundMoveEntityPacket::pos(7, [1, -2, 3], true),
        ClientboundMoveEntityPacket {
            id: 7,
            delta: [1, -2, 3],
            y_rot: 0,
            x_rot: 0,
            on_ground: true,
            has_position: true,
            has_rotation: false,
        }
    );
    assert_eq!(
        ClientboundMoveEntityPacket::pos_rot(7, [1, 2, 3], 90.0, 45.0, false).y_rot,
        64
    );
    assert_eq!(
        ClientboundMoveEntityPacket::rot(7, 180.0, 45.0, true).x_rot,
        32
    );
    let mut move_pos = Vec::new();
    ClientboundMoveEntityPacket::pos(300, [1, -2, 3], true)
        .write_pos(&mut move_pos)
        .unwrap();
    assert_eq!(
        move_pos,
        vec![0xac, 0x02, 0x00, 0x01, 0xff, 0xfe, 0x00, 0x03, 0x01]
    );

    let mut move_pos_rot = Vec::new();
    ClientboundMoveEntityPacket::pos_rot(300, [1, 2, 3], 90.0, 45.0, false)
        .write_pos_rot(&mut move_pos_rot)
        .unwrap();
    assert_eq!(
        move_pos_rot,
        vec![0xac, 0x02, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x40, 0x20, 0x00]
    );

    let mut move_rot = Vec::new();
    ClientboundMoveEntityPacket::rot(300, 180.0, 45.0, true)
        .write_rot(&mut move_rot)
        .unwrap();
    assert_eq!(move_rot, vec![0xac, 0x02, 0x80, 0x20, 0x01]);
    assert_eq!(ClientboundRotateHeadPacket::new(7, 180.0).y_head_rot, 128);
    let motion = ClientboundSetEntityMotionPacket::new(
        7,
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );
    let mut motion_payload = Vec::new();
    motion.write(&mut motion_payload).unwrap();
    assert_eq!(motion_payload, vec![7, 0]);

    let add_entity = ClientboundAddEntityPacket::new(AddEntityPacketInput {
        id: 300,
        uuid: Uuid([4; 16]),
        entity_type: 5,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3::ZERO,
        rotation: (90.0, 45.0),
        y_head_rot: 180.0,
        data: 123,
    });
    let mut add_entity_payload = Vec::new();
    add_entity.write(&mut add_entity_payload).unwrap();
    assert_eq!(
        &add_entity_payload[..19],
        &[vec![0xac, 0x02], vec![4; 16], vec![5]].concat()
    );
    assert_eq!(&add_entity_payload[19..27], &1.25_f64.to_be_bytes());
    assert_eq!(&add_entity_payload[27..35], &64.0_f64.to_be_bytes());
    assert_eq!(&add_entity_payload[35..43], &(-2.5_f64).to_be_bytes());
    assert_eq!(&add_entity_payload[43..], &[0, 64, 32, 128, 123]);

    let move_vehicle = ClientboundMoveVehiclePacket {
        position: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        y_rot: 90.0,
        x_rot: 45.0,
    };
    let mut move_vehicle_payload = Vec::new();
    move_vehicle.write(&mut move_vehicle_payload).unwrap();
    assert_eq!(&move_vehicle_payload[..8], &1.0_f64.to_be_bytes());
    assert_eq!(&move_vehicle_payload[8..16], &2.0_f64.to_be_bytes());
    assert_eq!(&move_vehicle_payload[16..24], &3.0_f64.to_be_bytes());
    assert_eq!(&move_vehicle_payload[24..28], &90.0_f32.to_be_bytes());
    assert_eq!(&move_vehicle_payload[28..32], &45.0_f32.to_be_bytes());
    let mut rotate_head_payload = Vec::new();
    ClientboundRotateHeadPacket::new(7, 180.0)
        .write(&mut rotate_head_payload)
        .unwrap();
    assert_eq!(rotate_head_payload, vec![7, 128]);
    assert_eq!(
        ClientboundSetEntityLinkPacket::new(7, None),
        ClientboundSetEntityLinkPacket {
            source_id: 7,
            dest_id: 0,
        }
    );
    assert_eq!(
        ClientboundSetPassengersPacket {
            vehicle: 7,
            passengers: vec![8, 9],
        }
        .passengers,
        vec![8, 9]
    );
    let mut passengers_payload = Vec::new();
    ClientboundSetPassengersPacket {
        vehicle: 7,
        passengers: vec![8, 9],
    }
    .write(&mut passengers_payload)
    .unwrap();
    assert_eq!(passengers_payload, vec![7, 2, 8, 9]);
    assert_eq!(
        ClientboundAnimatePacket {
            id: 7,
            action: EntityAnimation::SwingOffHand,
        }
        .action as i32,
        3
    );
    let mut animate_payload = Vec::new();
    ClientboundAnimatePacket {
        id: 7,
        action: EntityAnimation::SwingOffHand,
    }
    .write(&mut animate_payload)
    .unwrap();
    assert_eq!(animate_payload, vec![7, 3]);
    let mut entity_event_payload = Vec::new();
    ClientboundEntityEventPacket {
        entity_id: 7,
        event_id: 3,
    }
    .write(&mut entity_event_payload)
    .unwrap();
    assert_eq!(entity_event_payload, vec![0, 0, 0, 7, 3]);
    assert_eq!(
        ClientboundRemoveEntitiesPacket {
            entity_ids: vec![7, 8]
        }
        .entity_ids,
        vec![7, 8]
    );
    let mut remove_payload = Vec::new();
    ClientboundRemoveEntitiesPacket {
        entity_ids: vec![7, 8],
    }
    .write(&mut remove_payload)
    .unwrap();
    assert_eq!(remove_payload, vec![2, 7, 8]);

    let border = ClientboundInitializeBorderPacket {
        new_center_x: 1.0,
        new_center_z: 2.0,
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
        new_absolute_max_size: 400,
        warning_blocks: 5,
        warning_time: 6,
    };
    let mut border_payload = Vec::new();
    border.write(&mut border_payload).unwrap();
    assert_eq!(&border_payload[..8], &1.0_f64.to_be_bytes());
    assert_eq!(&border_payload[8..16], &2.0_f64.to_be_bytes());
    assert_eq!(&border_payload[16..24], &100.0_f64.to_be_bytes());
    assert_eq!(&border_payload[24..32], &200.0_f64.to_be_bytes());
    assert_eq!(&border_payload[32..], &[0xac, 0x02, 0x90, 0x03, 5, 6]);

    let mut border_center = Vec::new();
    ClientboundSetBorderCenterPacket {
        new_center_x: 1.0,
        new_center_z: 2.0,
    }
    .write(&mut border_center)
    .unwrap();
    assert_eq!(&border_center[..8], &1.0_f64.to_be_bytes());
    assert_eq!(&border_center[8..], &2.0_f64.to_be_bytes());

    let mut border_lerp = Vec::new();
    ClientboundSetBorderLerpSizePacket {
        old_size: 100.0,
        new_size: 200.0,
        lerp_time: 300,
    }
    .write(&mut border_lerp)
    .unwrap();
    assert_eq!(&border_lerp[..8], &100.0_f64.to_be_bytes());
    assert_eq!(&border_lerp[8..16], &200.0_f64.to_be_bytes());
    assert_eq!(&border_lerp[16..], &[0xac, 0x02]);

    let mut border_size = Vec::new();
    ClientboundSetBorderSizePacket { size: 200.0 }
        .write(&mut border_size)
        .unwrap();
    assert_eq!(border_size, 200.0_f64.to_be_bytes());

    let mut warning_delay = Vec::new();
    ClientboundSetBorderWarningDelayPacket { warning_delay: 6 }
        .write(&mut warning_delay)
        .unwrap();
    assert_eq!(warning_delay, vec![6]);

    let mut warning_distance = Vec::new();
    ClientboundSetBorderWarningDistancePacket { warning_blocks: 5 }
        .write(&mut warning_distance)
        .unwrap();
    assert_eq!(warning_distance, vec![5]);

    let mut clear_titles = Vec::new();
    ClientboundClearTitlesPacket { reset_times: true }
        .write(&mut clear_titles)
        .unwrap();
    assert_eq!(clear_titles, vec![1]);

    let mut title_times = Vec::new();
    ClientboundSetTitlesAnimationPacket {
        fade_in: 10,
        stay: 70,
        fade_out: 20,
    }
    .write(&mut title_times)
    .unwrap();
    assert_eq!(
        title_times,
        [
            10_i32.to_be_bytes(),
            70_i32.to_be_bytes(),
            20_i32.to_be_bytes()
        ]
        .concat()
    );

    let mut clientbound_close = Vec::new();
    ClientboundContainerClosePacket { container_id: 128 }
        .write(&mut clientbound_close)
        .unwrap();
    assert_eq!(clientbound_close, vec![0x80, 0x01]);

    let mut set_data = Vec::new();
    ClientboundContainerSetDataPacket {
        container_id: 2,
        id: -3,
        value: 400,
    }
    .write(&mut set_data)
    .unwrap();
    assert_eq!(set_data, vec![2, 0xff, 0xfd, 0x01, 0x90]);

    let mut mount_screen = Vec::new();
    ClientboundMountScreenOpenPacket {
        container_id: 2,
        inventory_columns: 5,
        entity_id: 300,
    }
    .write(&mut mount_screen)
    .unwrap();
    assert_eq!(
        mount_screen,
        [vec![2, 5], 300_i32.to_be_bytes().to_vec()].concat()
    );

    let mut cooldown = Vec::new();
    ClientboundCooldownPacket {
        cooldown_group: Identifier::parse("minecraft:ender_pearl").unwrap(),
        duration: 20,
    }
    .write(&mut cooldown)
    .unwrap();
    assert_eq!(
        cooldown,
        [vec![21], b"minecraft:ender_pearl".to_vec(), vec![20]].concat()
    );

    let abilities = ClientboundPlayerAbilitiesPacket {
        invulnerable: true,
        flying: false,
        can_fly: true,
        instant_build: true,
        flying_speed: 0.05,
        walking_speed: 0.1,
    };
    let mut abilities_payload = Vec::new();
    abilities.write(&mut abilities_payload).unwrap();
    assert_eq!(abilities_payload[0], 0b1101);
    assert_eq!(&abilities_payload[1..5], &0.05_f32.to_be_bytes());
    assert_eq!(&abilities_payload[5..9], &0.1_f32.to_be_bytes());

    let mut block_destruction = Vec::new();
    ClientboundBlockDestructionPacket {
        id: 99,
        x: -12,
        y: 64,
        z: 34,
        progress: 9,
    }
    .write(&mut block_destruction)
    .unwrap();
    assert_eq!(block_destruction[0], 99);
    assert_eq!(block_destruction.len(), 10);
    assert_eq!(*block_destruction.last().unwrap(), 9);

    let mut block_event = Vec::new();
    ClientboundBlockEventPacket {
        x: -12,
        y: 64,
        z: 34,
        action: 1,
        param: 2,
        block_id: 300,
    }
    .write(&mut block_event)
    .unwrap();
    assert_eq!(block_event.len(), 12);
    assert_eq!(&block_event[8..10], &[1, 2]);
    assert_eq!(&block_event[10..], &[0xac, 0x02]);

    let mut block_update = Vec::new();
    ClientboundBlockUpdatePacket {
        x: -12,
        y: 64,
        z: 34,
        block_state_id: 300,
    }
    .write(&mut block_update)
    .unwrap();
    assert_eq!(block_update.len(), 10);
    assert_eq!(&block_update[8..], &[0xac, 0x02]);

    let mut level_event = Vec::new();
    ClientboundLevelEventPacket {
        event_type: 2001,
        x: -12,
        y: 64,
        z: 34,
        data: 300,
        global_event: true,
    }
    .write(&mut level_event)
    .unwrap();
    assert_eq!(&level_event[..4], &2001_i32.to_be_bytes());
    assert_eq!(&level_event[12..16], &300_i32.to_be_bytes());
    assert_eq!(level_event[16], 1);

    let mut player_info_remove = Vec::new();
    ClientboundPlayerInfoRemovePacket {
        profile_ids: vec![Uuid([1; 16]), Uuid([2; 16])],
    }
    .write(&mut player_info_remove)
    .unwrap();
    assert_eq!(player_info_remove[0], 2);
    assert_eq!(&player_info_remove[1..17], &[1; 16]);
    assert_eq!(&player_info_remove[17..33], &[2; 16]);

    let filled_stack = RawItemStack {
        count: 2,
        item_id: Some(5),
        components: RawDataComponentPatch::empty(),
    };
    let mut container_content = Vec::new();
    ClientboundContainerPacket {
        container_id: 3,
        state_id: 4,
        slots: vec![filled_stack.clone(), RawItemStack::empty()],
        carried_item: RawItemStack::empty(),
    }
    .write(&mut container_content)
    .unwrap();
    assert_eq!(container_content, vec![3, 4, 2, 2, 5, 0, 0, 0, 0]);

    let mut container_slot = Vec::new();
    ClientboundContainerSetSlotPacket {
        container_id: 3,
        state_id: 4,
        slot: -1,
        item_stack: filled_stack.clone(),
    }
    .write(&mut container_slot)
    .unwrap();
    assert_eq!(&container_slot[..5], &[3, 4, 0xff, 0xff, 2]);
    assert_eq!(&container_slot[5..], &[5, 0, 0]);

    let mut cursor_item = Vec::new();
    ClientboundSetCursorItemPacket {
        item_stack: filled_stack,
    }
    .write(&mut cursor_item)
    .unwrap();
    assert_eq!(cursor_item, vec![2, 5, 0, 0]);

    let mut merchant_offers = Vec::new();
    ClientboundMerchantOffersPacket {
        container_id: 2,
        offers: vec![MerchantOfferData {
            base_cost_a: ItemCostData {
                item_id: 5,
                count: 3,
                components: RawDataComponentExactPredicate::empty(),
            },
            result: RawItemStack {
                count: 1,
                item_id: Some(6),
                components: RawDataComponentPatch::empty(),
            },
            cost_b: Some(ItemCostData {
                item_id: 7,
                count: 2,
                components: RawDataComponentExactPredicate::empty(),
            }),
            out_of_stock: true,
            uses: 1,
            max_uses: 12,
            xp: 4,
            special_price_diff: -2,
            price_multiplier: 0.05,
            demand: 9,
        }],
        villager_level: 3,
        villager_xp: 120,
        show_progress: true,
        can_restock: false,
    }
    .write(&mut merchant_offers)
    .unwrap();
    assert_eq!(
        &merchant_offers[..13],
        &[2, 1, 5, 3, 0, 1, 6, 0, 0, 1, 7, 2, 0]
    );
    assert_eq!(merchant_offers[13], 1);
    assert_eq!(&merchant_offers[14..18], &1_i32.to_be_bytes());
    assert_eq!(&merchant_offers[18..22], &12_i32.to_be_bytes());
    assert_eq!(&merchant_offers[22..26], &4_i32.to_be_bytes());
    assert_eq!(&merchant_offers[26..30], &(-2_i32).to_be_bytes());
    assert_eq!(&merchant_offers[30..34], &0.05_f32.to_be_bytes());
    assert_eq!(&merchant_offers[34..38], &9_i32.to_be_bytes());
    assert_eq!(&merchant_offers[38..], &[3, 120, 1, 0]);

    let mut player_info = Vec::new();
    ClientboundPlayerInfoUpdatePacket::player_initializing(vec![PlayerInfoUpdateEntry {
        profile_id: Uuid([7; 16]),
        profile: Some(PlayerInfoProfile {
            name: "Steve".to_string(),
            properties: vec![GameProfileProperty {
                name: "textures".to_string(),
                value: "abc".to_string(),
                signature: Some("sig".to_string()),
            }],
        }),
        chat_session_payload: None,
        game_mode: 1,
        listed: true,
        latency: 20,
        display_name_payload: None,
        list_order: 3,
        show_hat: true,
    }])
    .write(&mut player_info)
    .unwrap();
    assert_eq!(
        player_info,
        [
            vec![0xff, 1],
            vec![7; 16],
            vec![5],
            b"Steve".to_vec(),
            vec![1, 8],
            b"textures".to_vec(),
            vec![3],
            b"abc".to_vec(),
            vec![1, 3],
            b"sig".to_vec(),
            vec![0, 1, 1, 20, 0, 3, 1],
        ]
        .concat()
    );

    let mut player_chat = Vec::new();
    ClientboundPlayerChatPacket {
        global_index: 1,
        sender: Uuid([8; 16]),
        index: 2,
        signature: Some(vec![9; 256]),
        body: SignedMessageBodyPacked {
            content: "hi".to_string(),
            timestamp_epoch_millis: 1000,
            salt: -2,
            last_seen: vec![
                MessageSignaturePackedData::Id(2),
                MessageSignaturePackedData::Full(vec![7; 256]),
            ],
        },
        unsigned_content_payload: None,
        filter_mask: FilterMaskData::PartiallyFiltered(vec![5]),
        chat_type: BoundChatTypeData {
            chat_type_id: 0,
            name_payload: vec![0],
            target_name_payload: None,
        },
    }
    .write(&mut player_chat)
    .unwrap();
    assert_eq!(
        &player_chat[..19],
        &[vec![1], vec![8; 16], vec![2, 1]].concat()
    );
    assert_eq!(&player_chat[19..275], &[9; 256]);
    assert_eq!(&player_chat[275..278], &[2, b'h', b'i']);
    assert_eq!(&player_chat[278..286], &1000_i64.to_be_bytes());
    assert_eq!(&player_chat[286..294], &(-2_i64).to_be_bytes());
    assert_eq!(&player_chat[294..297], &[2, 3, 0]);
    assert_eq!(&player_chat[297..553], &[7; 256]);
    assert_eq!(
        &player_chat[553..],
        &[0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 5, 1, 0, 0]
    );

    let mut recipe_add = Vec::new();
    ClientboundRecipeBookAddPacket {
        entries: vec![RecipeBookAddEntry::new(
            RecipeDisplayEntryData {
                id: 3,
                display: RecipeDisplayData::Stonecutter {
                    ingredient: SlotDisplayData::Item { item_id: 5 },
                    result: SlotDisplayData::Item { item_id: 6 },
                    crafting_station: SlotDisplayData::Empty,
                },
                group: Some(7),
                category_id: 10,
                crafting_requirements: Some(vec![
                    RecipeIngredientData::DirectItems(vec![5, 6]),
                    RecipeIngredientData::Tag(Identifier::parse("minecraft:logs").unwrap()),
                ]),
            },
            true,
            true,
        )],
        replace: true,
    }
    .write(&mut recipe_add)
    .unwrap();
    assert_eq!(
        recipe_add,
        [
            vec![1, 3, 3, 4, 5, 4, 6, 0, 8, 10, 1, 2, 3, 5, 6, 0, 14],
            b"minecraft:logs".to_vec(),
            vec![3, 1],
        ]
        .concat()
    );

    let mut recipe_remove = Vec::new();
    ClientboundRecipeBookRemovePacket {
        recipe_display_ids: vec![1, 128],
    }
    .write(&mut recipe_remove)
    .unwrap();
    assert_eq!(recipe_remove, vec![2, 1, 0x80, 0x01]);

    let mut recipe_settings = Vec::new();
    ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings {
            open: true,
            filtering: false,
        },
        furnace: RecipeBookTypeSettings {
            open: false,
            filtering: true,
        },
        blast_furnace: RecipeBookTypeSettings::CLOSED_UNFILTERED,
        smoker: RecipeBookTypeSettings {
            open: true,
            filtering: true,
        },
    }
    .write(&mut recipe_settings)
    .unwrap();
    assert_eq!(recipe_settings, vec![1, 0, 0, 1, 0, 0, 1, 1]);

    let root_id = Identifier::parse("minecraft:story/root").unwrap();
    let hidden_id = Identifier::parse("minecraft:story/hidden").unwrap();
    let mut advancements = Vec::new();
    ClientboundAdvancementsPacket {
        reset: true,
        added: vec![AdvancementHolderData::minimal(
            root_id.clone(),
            None,
            vec![vec!["tick".to_string()]],
            true,
        )],
        removed: vec![hidden_id],
        progress: vec![(
            root_id,
            AdvancementProgressData {
                criteria: vec![
                    (
                        "tick".to_string(),
                        CriterionProgressData {
                            obtained_epoch_millis: Some(1000),
                        },
                    ),
                    (
                        "stone".to_string(),
                        CriterionProgressData {
                            obtained_epoch_millis: None,
                        },
                    ),
                ],
            },
        )],
        show_advancements: true,
    }
    .write(&mut advancements)
    .unwrap();
    assert_eq!(
        advancements,
        [
            vec![
                1, 1, 20, // reset, added count, holder id length
            ],
            b"minecraft:story/root".to_vec(),
            vec![
                0, 0, 1, 1,
                4, // no parent, no display, requirements count/group/string length
            ],
            b"tick".to_vec(),
            vec![1, 1, 22], // telemetry, removed count, removed id length
            b"minecraft:story/hidden".to_vec(),
            vec![1, 20], // progress map count, progress id length
            b"minecraft:story/root".to_vec(),
            vec![2, 4], // criteria count, first criterion length
            b"tick".to_vec(),
            vec![
                1, 0, 0, 0, 0, 0, 0, 3, 0xe8,
                5, // done + epoch millis, second criterion length
            ],
            b"stone".to_vec(),
            vec![0, 1], // not done, show advancements
        ]
        .concat()
    );

    let mut commands = Vec::new();
    ClientboundCommandsPacket {
        root_index: 0,
        entries: vec![
            CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: None,
                children: vec![1, 2],
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Literal {
                    name: "help".to_string(),
                },
                executable: true,
                restricted: false,
                redirect: None,
                children: Vec::new(),
            },
            CommandNodeEntryData {
                stub: CommandNodeStubData::Argument {
                    name: "target".to_string(),
                    parser_type_id: 5,
                    parser_payload: vec![0x03],
                    suggestion_id: Some(Identifier::parse("minecraft:ask_server").unwrap()),
                },
                executable: false,
                restricted: true,
                redirect: Some(1),
                children: Vec::new(),
            },
        ],
    }
    .write(&mut commands)
    .unwrap();
    assert_eq!(
        commands,
        [
            vec![3, 0, 2, 1, 2, 5, 0, 4],
            b"help".to_vec(),
            vec![58, 0, 1, 6],
            b"target".to_vec(),
            vec![5, 0x03, 20],
            b"minecraft:ask_server".to_vec(),
            vec![0],
        ]
        .concat()
    );

    let mut command_suggestions = Vec::new();
    ClientboundCommandSuggestionsPacket {
        transaction_id: 4,
        start: 1,
        length: 2,
        suggestions: vec![
            CommandSuggestionEntry {
                text: "help".to_string(),
                tooltip: None,
            },
            CommandSuggestionEntry {
                text: "hello".to_string(),
                tooltip: Some(Tag::Compound(vec![(
                    "text".to_string(),
                    Tag::String("tooltip".to_string()),
                )])),
            },
        ],
    }
    .write(&mut command_suggestions)
    .unwrap();
    assert_eq!(&command_suggestions[..6], &[4, 1, 2, 2, 4, b'h']);
    assert!(command_suggestions.ends_with(&[0]));
    assert!(command_suggestions
        .windows(4)
        .any(|window| window == [1, 10, 8, 0]));

    let mut debug_sample = Vec::new();
    ClientboundDebugSamplePacket {
        sample: vec![10, -20],
        sample_type: RemoteDebugSampleType::TickTime,
    }
    .write(&mut debug_sample)
    .unwrap();
    assert_eq!(debug_sample[0], 2);
    assert_eq!(&debug_sample[1..9], &10_i64.to_be_bytes());
    assert_eq!(&debug_sample[9..17], &(-20_i64).to_be_bytes());
    assert_eq!(debug_sample[17], 0);

    let mut start_configuration = Vec::new();
    ClientboundStartConfigurationPacket
        .write(&mut start_configuration)
        .unwrap();
    assert!(start_configuration.is_empty());

    let mut remove_effect = Vec::new();
    ClientboundRemoveMobEffectPacket {
        entity_id: 129,
        effect_id: 5,
    }
    .write(&mut remove_effect)
    .unwrap();
    assert_eq!(remove_effect, vec![0x81, 0x01, 5]);

    let mut update_effect = Vec::new();
    ClientboundUpdateMobEffectPacket {
        entity_id: 129,
        effect_id: 5,
        amplifier: 2,
        duration_ticks: 600,
        flags: MobEffectFlags::from_parts(true, false, true, true),
    }
    .write(&mut update_effect)
    .unwrap();
    assert_eq!(update_effect, vec![0x81, 0x01, 5, 2, 0xd8, 0x04, 0x0d]);

    let mut award_stats = Vec::new();
    ClientboundAwardStatsPacket {
        stats: vec![AwardedStat {
            stat_type_id: 8,
            stat_value_id: 23,
            value: 300,
        }],
    }
    .write(&mut award_stats)
    .unwrap();
    assert_eq!(award_stats, vec![1, 8, 23, 0xac, 0x02]);

    let mut update_attributes = Vec::new();
    ClientboundUpdateAttributesPacket {
        entity_id: 300,
        attributes: vec![AttributeSnapshot {
            attribute_id: 4,
            base: 20.0,
            modifiers: vec![AttributeModifierSnapshot {
                id: Identifier::parse("minecraft:generic.movement_speed").unwrap(),
                amount: 0.5,
                operation: AttributeModifierOperation::MultipliedTotal,
            }],
        }],
    }
    .write(&mut update_attributes)
    .unwrap();
    assert_eq!(&update_attributes[..3], &[0xac, 0x02, 1]);
    assert_eq!(update_attributes[3], 4);
    assert_eq!(&update_attributes[4..12], &20.0_f64.to_be_bytes());
    assert_eq!(update_attributes[12], 1);
    assert_eq!(update_attributes[13], 32);
    assert_eq!(
        &update_attributes[14..46],
        b"minecraft:generic.movement_speed"
    );
    assert_eq!(&update_attributes[46..54], &0.5_f64.to_be_bytes());
    assert_eq!(update_attributes[54], 2);

    let mut section_blocks = Vec::new();
    ClientboundSectionBlocksUpdatePacket {
        section_pos: SectionPos { x: 1, y: -2, z: 3 },
        updates: vec![SectionBlockUpdate {
            packed_pos: 0x0abc,
            block_state_id: 118,
        }],
    }
    .write(&mut section_blocks)
    .unwrap();
    assert_eq!(
        &section_blocks[..8],
        &0x0000_0400_003f_fffe_i64.to_be_bytes()
    );
    assert_eq!(&section_blocks[8..], &[1, 0xbc, 0xd5, 0x1d]);

    let mut block_entity = Vec::new();
    ClientboundBlockEntityDataPacket {
        x: 1,
        y: 64,
        z: -2,
        block_entity_type_id: 1,
        tag: Tag::Compound(Vec::new()),
    }
    .write(&mut block_entity)
    .unwrap();
    assert_eq!(
        block_entity,
        [
            pack_block_position(1, 64, -2).to_be_bytes().to_vec(),
            vec![1, 10, 0],
        ]
        .concat()
    );

    let mut named_like_network_nbt = Vec::new();
    ClientboundBlockEntityDataPacket {
        x: 0,
        y: 0,
        z: 0,
        block_entity_type_id: 1,
        tag: Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:chest".to_string()),
        )]),
    }
    .write(&mut named_like_network_nbt)
    .unwrap();
    assert_eq!(named_like_network_nbt[9], 10);
    assert_eq!(
        &named_like_network_nbt[10..13],
        &[8, 0, 2],
        "network NBT uses writeAnyTag and must not include a root name"
    );

    let title_tag = Tag::Compound(vec![("text".to_string(), Tag::String("Title".to_string()))]);
    let mut title = Vec::new();
    ClientboundSetTitleTextPacket {
        text: title_tag.clone(),
    }
    .write(&mut title)
    .unwrap();
    assert_eq!(&title[..4], &[10, 8, 0, 4]);

    let mut subtitle = Vec::new();
    ClientboundSetSubtitleTextPacket {
        text: title_tag.clone(),
    }
    .write(&mut subtitle)
    .unwrap();
    assert_eq!(subtitle, title);

    let mut action_bar = Vec::new();
    ClientboundSetActionBarTextPacket {
        text: title_tag.clone(),
    }
    .write(&mut action_bar)
    .unwrap();
    assert_eq!(action_bar, title);

    let mut system_chat = Vec::new();
    ClientboundSystemChatPacket {
        content: title_tag.clone(),
        overlay: true,
    }
    .write(&mut system_chat)
    .unwrap();
    assert!(system_chat.starts_with(&title));
    assert_eq!(*system_chat.last().unwrap(), 1);

    let mut disguised_chat = Vec::new();
    ClientboundDisguisedChatPacket {
        message: title_tag.clone(),
        chat_type: ChatTypeBound {
            chat_type_id: 0,
            name: Tag::Compound(vec![("text".to_string(), Tag::String("Steve".to_string()))]),
            target_name: Some(Tag::Compound(vec![(
                "text".to_string(),
                Tag::String("Alex".to_string()),
            )])),
        },
    }
    .write(&mut disguised_chat)
    .unwrap();
    assert!(disguised_chat.starts_with(&title));
    assert!(disguised_chat.windows(3).any(|window| window == [0, 1, 10]));
    assert!(disguised_chat.ends_with(&[0]));

    let footer_tag = Tag::Compound(vec![(
        "text".to_string(),
        Tag::String("Footer".to_string()),
    )]);
    let mut tab_list = Vec::new();
    ClientboundTabListPacket {
        header: title_tag,
        footer: footer_tag,
    }
    .write(&mut tab_list)
    .unwrap();
    assert!(tab_list.starts_with(&title));
    assert_eq!(tab_list.iter().filter(|byte| **byte == 10).count(), 2);

    let mut reset_score = Vec::new();
    ClientboundResetScorePacket {
        owner: "Alex".to_string(),
        objective_name: Some("kills".to_string()),
    }
    .write(&mut reset_score)
    .unwrap();
    assert_eq!(
        reset_score,
        [vec![4], b"Alex".to_vec(), vec![1, 5], b"kills".to_vec()].concat()
    );

    let mut display_objective = Vec::new();
    ClientboundSetDisplayObjectivePacket {
        slot: 1,
        objective_name: "sidebar".to_string(),
    }
    .write(&mut display_objective)
    .unwrap();
    assert_eq!(
        display_objective,
        [vec![1, 7], b"sidebar".to_vec()].concat()
    );

    let score_name = Tag::Compound(vec![("text".to_string(), Tag::String("Kills".to_string()))]);
    let mut set_objective = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Add {
            display_name: score_name.clone(),
            render_type: ObjectiveRenderType::Hearts,
            number_format: Some(NumberFormat::Fixed {
                value: score_name.clone(),
            }),
        },
    }
    .write(&mut set_objective)
    .unwrap();
    assert_eq!(&set_objective[..7], &[5, b'k', b'i', b'l', b'l', b's', 0]);
    assert_eq!(set_objective[7], 10);
    assert!(set_objective.windows(3).any(|window| window == [1, 1, 2]));
    assert!(set_objective.ends_with(&[0]));

    let mut remove_objective = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Remove,
    }
    .write(&mut remove_objective)
    .unwrap();
    assert_eq!(
        remove_objective,
        [vec![5], b"kills".to_vec(), vec![1]].concat()
    );

    let mut set_score = Vec::new();
    ClientboundSetScorePacket {
        owner: "Alex".to_string(),
        objective_name: "kills".to_string(),
        score: 300,
        display: Some(score_name),
        number_format: Some(NumberFormat::Blank),
    }
    .write(&mut set_score)
    .unwrap();
    assert_eq!(
        &set_score[..12],
        &[4, b'A', b'l', b'e', b'x', 5, b'k', b'i', b'l', b'l', b's', 0xac]
    );
    assert!(set_score.windows(3).any(|window| window == [0x02, 1, 10]));
    assert_eq!(&set_score[set_score.len() - 3..], &[0, 1, 0]);

    let team_params = TeamPacketParameters {
        display_name: Tag::Compound(vec![("text".to_string(), Tag::String("Red".to_string()))]),
        options: 0b11,
        nametag_visibility: TeamVisibility::HideForOtherTeams,
        collision_rule: TeamCollisionRule::PushOwnTeam,
        color_id: 12,
        prefix: Tag::Compound(vec![("text".to_string(), Tag::String("[".to_string()))]),
        suffix: Tag::Compound(vec![("text".to_string(), Tag::String("]".to_string()))]),
    };
    let mut team_create = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Create {
            parameters: team_params.clone(),
            players: vec!["Alex".to_string(), "Steve".to_string()],
        },
    }
    .write(&mut team_create)
    .unwrap();
    assert_eq!(&team_create[..5], &[3, b'r', b'e', b'd', 0]);
    assert!(team_create.windows(4).any(|window| window == [3, 2, 3, 12]));
    assert!(team_create.ends_with(b"\x05Steve"));

    let mut team_update = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::Update {
            parameters: team_params,
        },
    }
    .write(&mut team_update)
    .unwrap();
    assert_eq!(&team_update[..5], &[3, b'r', b'e', b'd', 2]);
    assert!(!team_update.ends_with(b"Steve"));

    let mut team_remove_players = Vec::new();
    ClientboundSetPlayerTeamPacket {
        name: "red".to_string(),
        method: TeamPacketMethod::RemovePlayers {
            players: vec!["Alex".to_string()],
        },
    }
    .write(&mut team_remove_players)
    .unwrap();
    assert_eq!(
        team_remove_players,
        [vec![3], b"red".to_vec(), vec![4, 1, 4], b"Alex".to_vec()].concat()
    );

    let mut open_screen = Vec::new();
    ClientboundOpenScreenPacket {
        container_id: 300,
        menu_type_id: 9,
        title: Tag::Compound(vec![("text".to_string(), Tag::String("Chest".to_string()))]),
    }
    .write(&mut open_screen)
    .unwrap();
    assert_eq!(&open_screen[..3], &[0xac, 0x02, 9]);
    assert_eq!(open_screen[3], 10);
    assert!(open_screen.ends_with(&[0]));

    let mut boss_add = Vec::new();
    ClientboundBossEventPacket {
        event_id: Uuid([9; 16]),
        operation: BossEventOperation::Add {
            name: Tag::Compound(vec![("text".to_string(), Tag::String("Boss".to_string()))]),
            progress: 0.75,
            color: BossBarColor::Purple,
            overlay: BossBarOverlay::Notched10,
            flags: BossEventFlags {
                darken_screen: true,
                play_music: false,
                create_world_fog: true,
            },
        },
    }
    .write(&mut boss_add)
    .unwrap();
    assert_eq!(&boss_add[..17], &[vec![9; 16], vec![0]].concat());
    assert!(boss_add
        .windows(4)
        .any(|window| window == 0.75_f32.to_be_bytes()));
    assert_eq!(&boss_add[boss_add.len() - 3..], &[5, 2, 5]);

    let mut boss_progress = Vec::new();
    ClientboundBossEventPacket {
        event_id: Uuid([8; 16]),
        operation: BossEventOperation::UpdateProgress { progress: 0.25 },
    }
    .write(&mut boss_progress)
    .unwrap();
    assert_eq!(&boss_progress[..17], &[vec![8; 16], vec![2]].concat());
    assert_eq!(&boss_progress[17..], &0.25_f32.to_be_bytes());

    let mut boss_style = Vec::new();
    ClientboundBossEventPacket {
        event_id: Uuid([7; 16]),
        operation: BossEventOperation::UpdateStyle {
            color: BossBarColor::Red,
            overlay: BossBarOverlay::Notched20,
        },
    }
    .write(&mut boss_style)
    .unwrap();
    assert_eq!(&boss_style[16..], &[4, 2, 4]);

    let mut map_item = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 300,
        scale: 2,
        locked: true,
        decorations: Some(vec![MapDecorationData {
            decoration_type_id: 7,
            x: -1,
            y: 2,
            rotation: 19,
            name: Some(Tag::Compound(vec![(
                "text".to_string(),
                Tag::String("Home".to_string()),
            )])),
        }]),
        color_patch: Some(MapPatch {
            width: 2,
            height: 1,
            start_x: 4,
            start_y: 5,
            colors: vec![6, 7],
        }),
    }
    .write(&mut map_item)
    .unwrap();
    assert_eq!(&map_item[..10], &[0xac, 0x02, 2, 1, 1, 1, 7, 0xff, 2, 3]);
    assert!(map_item.windows(2).any(|window| window == [1, 10]));
    assert_eq!(&map_item[map_item.len() - 7..], &[2, 1, 4, 5, 2, 6, 7]);

    let mut map_no_patch = Vec::new();
    ClientboundMapItemDataPacket {
        map_id: 1,
        scale: 0,
        locked: false,
        decorations: None,
        color_patch: None,
    }
    .write(&mut map_no_patch)
    .unwrap();
    assert_eq!(map_no_patch, vec![1, 0, 0, 0, 0]);

    let mut pack_pop = Vec::new();
    ClientboundResourcePackPopPacket {
        id: Some(Uuid([3; 16])),
    }
    .write(&mut pack_pop)
    .unwrap();
    assert_eq!(pack_pop[0], 1);
    assert_eq!(&pack_pop[1..], &[3; 16]);

    let teleport = ClientboundTeleportEntityPacket {
        id: 7,
        position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        movement: Vec3 {
            x: 0.1,
            y: -0.2,
            z: 0.3,
        },
        y_rot: 90.0,
        x_rot: 30.0,
        relative_flags: 0b1_0010_0011,
        on_ground: true,
    };
    let mut teleport_payload = Vec::new();
    teleport.write(&mut teleport_payload).unwrap();
    assert_eq!(teleport_payload[0], 7);
    assert_eq!(&teleport_payload[1..9], &1.25_f64.to_be_bytes());
    assert_eq!(&teleport_payload[25..33], &0.1_f64.to_be_bytes());
    assert_eq!(&teleport_payload[49..53], &90.0_f32.to_be_bytes());
    assert_eq!(&teleport_payload[53..57], &30.0_f32.to_be_bytes());
    assert_eq!(&teleport_payload[57..61], &0b1_0010_0011_i32.to_be_bytes());
    assert_eq!(teleport_payload[61], 1);

    let mut position_sync = Vec::new();
    ClientboundEntityPositionSyncPacket {
        id: 8,
        position: teleport.position,
        movement: teleport.movement,
        y_rot: teleport.y_rot,
        x_rot: teleport.x_rot,
        on_ground: false,
    }
    .write(&mut position_sync)
    .unwrap();
    assert_eq!(position_sync[0], 8);
    assert_eq!(position_sync.len(), 58);
    assert_eq!(*position_sync.last().unwrap(), 0);

    let mut player_position = Vec::new();
    ClientboundPlayerPositionPacket {
        id: 9,
        position: teleport.position,
        movement: teleport.movement,
        y_rot: teleport.y_rot,
        x_rot: teleport.x_rot,
        relative_flags: 0b1_0010_0011,
    }
    .write(&mut player_position)
    .unwrap();
    assert_eq!(player_position[0], 9);
    assert_eq!(player_position.len(), 61);
    assert_eq!(&player_position[57..61], &0b1_0010_0011_i32.to_be_bytes());

    let mut look_at = Vec::new();
    ClientboundPlayerLookAtPacket {
        from_anchor: EntityAnchor::Eyes,
        x: 10.0,
        y: 64.5,
        z: -7.25,
        target_entity: Some((33, EntityAnchor::Feet)),
    }
    .write(&mut look_at)
    .unwrap();
    assert_eq!(look_at[0], 1);
    assert_eq!(&look_at[1..9], &10.0_f64.to_be_bytes());
    assert_eq!(&look_at[9..17], &64.5_f64.to_be_bytes());
    assert_eq!(&look_at[17..25], &(-7.25_f64).to_be_bytes());
    assert_eq!(&look_at[25..], &[1, 33, 0]);

    let mut sound_position = Vec::new();
    ClientboundSoundPacket {
        sound: SoundEventHolder::Registered { id: 5 },
        source_id: SoundSource::Blocks as i32,
        position: Vec3 {
            x: 1.25,
            y: -2.5,
            z: 3.0,
        },
        volume: 0.75,
        pitch: 1.25,
        seed: -9,
        entity_id: None,
    }
    .write_position(&mut sound_position)
    .unwrap();
    assert_eq!(&sound_position[..2], &[6, 4]);
    assert_eq!(&sound_position[2..6], &10_i32.to_be_bytes());
    assert_eq!(&sound_position[6..10], &(-20_i32).to_be_bytes());
    assert_eq!(&sound_position[10..14], &24_i32.to_be_bytes());
    assert_eq!(&sound_position[14..18], &0.75_f32.to_be_bytes());
    assert_eq!(&sound_position[18..22], &1.25_f32.to_be_bytes());
    assert_eq!(&sound_position[22..30], &(-9_i64).to_be_bytes());

    let mut direct_sound_entity = Vec::new();
    ClientboundSoundPacket {
        sound: SoundEventHolder::Direct {
            location: Identifier::parse("minecraft:test.sound").unwrap(),
            fixed_range: Some(16.0),
        },
        source_id: SoundSource::Players as i32,
        position: Vec3::ZERO,
        volume: 1.0,
        pitch: 0.5,
        seed: 42,
        entity_id: Some(300),
    }
    .write_entity(&mut direct_sound_entity)
    .unwrap();
    assert_eq!(&direct_sound_entity[..3], &[0, 20, b'm']);
    assert!(direct_sound_entity
        .windows(5)
        .any(|window| window == [1, 0x41, 0x80, 0, 0]));
    assert!(direct_sound_entity
        .windows(2)
        .any(|window| window == [0xac, 0x02]));

    let mut stop_sound = Vec::new();
    ClientboundStopSoundPacket {
        source: Some(SoundSource::Blocks),
        name: Some(Identifier::parse("minecraft:block.note_block.harp").unwrap()),
    }
    .write(&mut stop_sound)
    .unwrap();
    assert_eq!(
        stop_sound,
        [vec![3, 4, 31], b"minecraft:block.note_block.harp".to_vec()].concat()
    );

    let mut particle = Vec::new();
    ClientboundParticlePacket {
        particle_id: 300,
        override_limiter: true,
        always_show: false,
        position: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        offset: Vec3 {
            x: 0.25,
            y: 0.5,
            z: 0.75,
        },
        max_speed: 1.25,
        count: 4,
        particle_data: vec![0xaa, 0xbb],
    }
    .write(&mut particle)
    .unwrap();
    assert_eq!(&particle[..2], &[1, 0]);
    assert_eq!(&particle[2..10], &1.0_f64.to_be_bytes());
    assert_eq!(&particle[10..18], &2.0_f64.to_be_bytes());
    assert_eq!(&particle[18..26], &3.0_f64.to_be_bytes());
    assert_eq!(&particle[26..30], &0.25_f32.to_be_bytes());
    assert_eq!(&particle[30..34], &0.5_f32.to_be_bytes());
    assert_eq!(&particle[34..38], &0.75_f32.to_be_bytes());
    assert_eq!(&particle[38..42], &1.25_f32.to_be_bytes());
    assert_eq!(&particle[42..46], &4_i32.to_be_bytes());
    assert_eq!(&particle[46..], &[0xac, 0x02, 0xaa, 0xbb]);

    let mut explode = Vec::new();
    ClientboundExplodePacket {
        center: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        radius: 4.5,
        block_count: 6,
        player_knockback: Some(Vec3 {
            x: 0.25,
            y: 0.5,
            z: 0.75,
        }),
        explosion_particle: RawParticleOptions {
            particle_id: 300,
            data: vec![0xaa],
        },
        explosion_sound: SoundEventHolder::Registered { id: 5 },
        block_particles: vec![WeightedExplosionParticle {
            value: ExplosionParticleInfo {
                particle: RawParticleOptions {
                    particle_id: 1,
                    data: Vec::new(),
                },
                scaling: 2.0,
                speed: 3.0,
            },
            weight: 7,
        }],
    }
    .write(&mut explode)
    .unwrap();
    assert_eq!(&explode[..8], &1.0_f64.to_be_bytes());
    assert_eq!(&explode[8..16], &2.0_f64.to_be_bytes());
    assert_eq!(&explode[16..24], &3.0_f64.to_be_bytes());
    assert_eq!(&explode[24..28], &4.5_f32.to_be_bytes());
    assert_eq!(&explode[28..32], &6_i32.to_be_bytes());
    assert_eq!(explode[32], 1);
    assert_eq!(&explode[33..41], &0.25_f64.to_be_bytes());
    assert_eq!(&explode[41..49], &0.5_f64.to_be_bytes());
    assert_eq!(&explode[49..57], &0.75_f64.to_be_bytes());
    assert_eq!(&explode[57..63], &[0xac, 0x02, 0xaa, 6, 1, 1]);
    assert_eq!(&explode[63..67], &2.0_f32.to_be_bytes());
    assert_eq!(&explode[67..71], &3.0_f32.to_be_bytes());
    assert_eq!(explode[71], 7);

    let mut cached_delete_chat = Vec::new();
    ClientboundDeleteChatPacket {
        message_signature: PackedMessageSignature::CacheId(7),
    }
    .write(&mut cached_delete_chat)
    .unwrap();
    assert_eq!(cached_delete_chat, vec![8]);

    let mut full_delete_chat = Vec::new();
    ClientboundDeleteChatPacket {
        message_signature: PackedMessageSignature::Full(Box::new(MessageSignature([9; 256]))),
    }
    .write(&mut full_delete_chat)
    .unwrap();
    assert_eq!(full_delete_chat[0], 0);
    assert_eq!(&full_delete_chat[1..], &[9; 256]);
}
