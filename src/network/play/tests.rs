use super::*;
use crate::item_stack::ItemStack;
use crate::network::codec::{cursor, read_identifier, read_string};

fn decoded(id: i32, payload: Vec<u8>) -> DecodedPacket {
    DecodedPacket {
        state: ProtocolState::Play,
        direction: PacketDirection::Serverbound,
        id,
        payload,
    }
}

fn stack(count: i32, item_id: i32) -> RawItemStack {
    RawItemStack {
        count,
        item_id: Some(item_id),
        components: RawDataComponentPatch::empty(),
    }
}

fn scripted_container_click(
    state_id: i32,
    slot: i32,
    changed_slots: Vec<(i32, ItemStack)>,
    carried: ItemStack,
) -> ScriptedContainerClickPacket {
    ScriptedContainerClickPacket {
        container_id: 0,
        state_id,
        slot,
        button: 0,
        mode: crate::inventory::ContainerInput::Pickup,
        changed_slots,
        carried,
    }
}

#[test]
fn play_packet_registry_matches_game_protocol_order_and_counts() {
    let registry = PlayProtocolRegistry::new();
    assert_play_registry_counts_and_serverbound_names(&registry);
    assert_play_registry_clientbound_core_names(&registry);
    assert_play_registry_clientbound_system_names(&registry);
    assert_play_registry_tail_names(&registry);
}

fn assert_play_registry_counts_and_serverbound_names(registry: &PlayProtocolRegistry) {
    assert_eq!(
        registry.serverbound().len(),
        SERVERBOUND_PLAY_PACKET_COUNT_26_1_2
    );
    assert_eq!(
        registry.clientbound().len(),
        CLIENTBOUND_PLAY_PACKET_COUNT_26_1_2
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID),
        Some("accept_teleportation")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_PLAYER_LOADED_PACKET_ID),
        Some("player_loaded")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID),
        Some("chunk_batch_received")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_ACK_PACKET_ID),
        Some("chat_ack")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_COMMAND_PACKET_ID),
        Some("chat_command")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_COMMAND_SIGNED_PACKET_ID),
        Some("chat_command_signed")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_PACKET_ID),
        Some("chat")
    );
    assert_eq!(
        registry.serverbound_name(SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID),
        Some("chat_session_update")
    );
}

fn assert_play_registry_clientbound_core_names(registry: &PlayProtocolRegistry) {
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID),
        Some("bundle")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID),
        Some("chunk_batch_finished")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID),
        Some("chunk_batch_start")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID),
        Some("command_suggestions")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_DELETE_CHAT_PACKET_ID),
        Some("delete_chat")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT_PACKET_ID),
        Some("level_chunk_with_light")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LIGHT_UPDATE_PACKET_ID),
        Some("light_update")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID),
        Some("map_item_data")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_GAME_RULE_VALUES_PACKET_ID),
        Some("game_rule_values")
    );
}

fn assert_play_registry_clientbound_system_names(registry: &PlayProtocolRegistry) {
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_ADD_ENTITY_PACKET_ID),
        Some("add_entity")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID),
        Some("remove_entities")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID),
        Some("set_entity_data")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID),
        Some("update_attributes")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID),
        Some("teleport_entity")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID),
        Some("update_mob_effect")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID),
        Some("container_set_content")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID),
        Some("recipe_book_add")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID),
        Some("update_advancements")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID),
        Some("set_objective")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_BOSS_EVENT_PACKET_ID),
        Some("boss_event")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SOUND_PACKET_ID),
        Some("sound")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID),
        Some("level_particles")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_COMMANDS_PACKET_ID),
        Some("commands")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_DEBUG_SAMPLE_PACKET_ID),
        Some("debug_sample")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_LOGIN_PACKET_ID),
        Some("login")
    );
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_START_CONFIGURATION_PACKET_ID),
        Some("start_configuration")
    );
}

fn assert_play_registry_tail_names(registry: &PlayProtocolRegistry) {
    assert_eq!(registry.serverbound().last(), Some(&"custom_click_action"));
    assert_eq!(registry.clientbound().last(), Some(&"show_dialog"));
}

#[test]
fn recipe_book_add_packet_matches_vanilla_display_and_slot_stream_order() {
    let payload = recipe_book_add_payload();
    assert_eq!(payload, expected_recipe_book_add_payload());
}

fn recipe_book_add_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    ClientboundRecipeBookAddPacket {
        entries: vec![
            shapeless_recipe_book_entry(),
            shaped_recipe_book_entry(),
            furnace_recipe_book_entry(),
            stonecutter_recipe_book_entry(),
            smithing_recipe_book_entry(),
        ],
        replace: true,
    }
    .write(&mut payload)
    .unwrap();
    payload
}

fn shapeless_recipe_book_entry() -> RecipeBookAddEntry {
    RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: 0,
            display: RecipeDisplayData::CraftingShapeless {
                ingredients: vec![
                    SlotDisplayData::Item { item_id: 1 },
                    SlotDisplayData::Tag {
                        tag: Identifier::parse("minecraft:planks").unwrap(),
                    },
                ],
                result: SlotDisplayData::ItemStack { stack: stack(1, 2) },
                crafting_station: SlotDisplayData::Item { item_id: 3 },
            },
            group: None,
            category_id: 0,
            crafting_requirements: None,
        },
        false,
        false,
    )
}

fn shaped_recipe_book_entry() -> RecipeBookAddEntry {
    RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: 1,
            display: RecipeDisplayData::CraftingShaped {
                width: 2,
                height: 2,
                ingredients: vec![
                    SlotDisplayData::Empty,
                    SlotDisplayData::Item { item_id: 4 },
                    SlotDisplayData::WithRemainder {
                        input: Box::new(SlotDisplayData::Item { item_id: 5 }),
                        remainder: Box::new(SlotDisplayData::ItemStack { stack: stack(1, 6) }),
                    },
                    SlotDisplayData::Composite(vec![
                        SlotDisplayData::Item { item_id: 7 },
                        SlotDisplayData::Tag {
                            tag: Identifier::parse("minecraft:logs").unwrap(),
                        },
                    ]),
                ],
                result: SlotDisplayData::Item { item_id: 8 },
                crafting_station: SlotDisplayData::Item { item_id: 9 },
            },
            group: Some(0),
            category_id: 1,
            crafting_requirements: Some(vec![
                RecipeIngredientData::DirectItems(vec![4]),
                RecipeIngredientData::Tag(Identifier::parse("minecraft:wool").unwrap()),
            ]),
        },
        true,
        true,
    )
}

#[test]
fn update_recipes_packet_writes_property_sets_and_stonecutter_entries() {
    let raw_iron_id = item_protocol_id("minecraft:raw_iron").unwrap();
    let stone_id = item_protocol_id("minecraft:stone").unwrap();
    let stone_slab_id = item_protocol_id("minecraft:stone_slab").unwrap();
    let manager = crate::recipe_system::RecipeManagerModel::new(vec![
        crate::recipe_system::RecipeHolder {
            id: "minecraft:iron_ingot_from_smelting_raw_iron",
            recipe: crate::recipe_system::RecipeKind::Cooking {
                kind: crate::recipe_system::CookingKind::Smelting,
                category: crate::recipe_system::CookingBookCategory::Misc,
                ingredient: crate::recipe_system::IngredientSpec::Item("minecraft:raw_iron"),
                result: crate::recipe_system::ItemAmount {
                    item: "minecraft:iron_ingot",
                    count: 1,
                },
                experience_millis: 700,
                cooking_time: Some(200),
            },
        },
        crate::recipe_system::RecipeHolder {
            id: "minecraft:stone_slab_from_stone_stonecutting",
            recipe: crate::recipe_system::RecipeKind::Stonecutting {
                ingredient: crate::recipe_system::IngredientSpec::Item("minecraft:stone"),
                result: crate::recipe_system::ItemAmount {
                    item: "minecraft:stone_slab",
                    count: 2,
                },
            },
        },
    ]);

    let mut bytes = Vec::new();
    write_clientbound_update_recipes_packet(&mut bytes, &manager).unwrap();
    let mut input = cursor(bytes);

    let property_set_count = read_var_i32(&mut input).unwrap();
    assert_eq!(property_set_count, 7);
    let mut saw_furnace_raw_iron = false;
    for _ in 0..property_set_count {
        let key = read_identifier(&mut input).unwrap();
        let item_count = read_var_i32(&mut input).unwrap();
        let items = (0..item_count)
            .map(|_| read_var_i32(&mut input).unwrap())
            .collect::<Vec<_>>();
        if key.to_string() == "minecraft:furnace_input" {
            saw_furnace_raw_iron = items == vec![raw_iron_id];
        }
    }
    assert!(saw_furnace_raw_iron);

    assert_eq!(read_var_i32(&mut input).unwrap(), 1);
    assert_eq!(read_var_i32(&mut input).unwrap(), 2);
    assert_eq!(read_var_i32(&mut input).unwrap(), stone_id);
    assert_eq!(read_var_i32(&mut input).unwrap(), 5);
    assert_eq!(read_var_i32(&mut input).unwrap(), stone_slab_id);
    assert_eq!(read_var_i32(&mut input).unwrap(), 2);
    assert_eq!(read_var_i32(&mut input).unwrap(), 0);
    assert_eq!(read_var_i32(&mut input).unwrap(), 0);
}

fn furnace_recipe_book_entry() -> RecipeBookAddEntry {
    RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: 2,
            display: RecipeDisplayData::Furnace {
                ingredient: SlotDisplayData::WithAnyPotion(Box::new(SlotDisplayData::Item {
                    item_id: 10,
                })),
                fuel: SlotDisplayData::AnyFuel,
                result: SlotDisplayData::ItemStack {
                    stack: stack(2, 11),
                },
                crafting_station: SlotDisplayData::Item { item_id: 12 },
                duration: 200,
                experience_bits: 1.0f32.to_bits(),
            },
            group: None,
            category_id: 2,
            crafting_requirements: None,
        },
        false,
        false,
    )
}

fn stonecutter_recipe_book_entry() -> RecipeBookAddEntry {
    RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: 3,
            display: RecipeDisplayData::Stonecutter {
                ingredient: SlotDisplayData::OnlyWithComponent {
                    contents: Box::new(SlotDisplayData::Item { item_id: 13 }),
                    component_type_id: 14,
                },
                result: SlotDisplayData::Dyed {
                    dye: Box::new(SlotDisplayData::Item { item_id: 15 }),
                    target: Box::new(SlotDisplayData::Item { item_id: 16 }),
                },
                crafting_station: SlotDisplayData::Item { item_id: 17 },
            },
            group: None,
            category_id: 3,
            crafting_requirements: None,
        },
        false,
        false,
    )
}

fn smithing_recipe_book_entry() -> RecipeBookAddEntry {
    RecipeBookAddEntry::new(
        RecipeDisplayEntryData {
            id: 4,
            display: RecipeDisplayData::Smithing {
                template: SlotDisplayData::Item { item_id: 18 },
                base: SlotDisplayData::SmithingTrim {
                    base: Box::new(SlotDisplayData::Item { item_id: 19 }),
                    material: Box::new(SlotDisplayData::Item { item_id: 20 }),
                    pattern_id: 21,
                },
                addition: SlotDisplayData::Item { item_id: 22 },
                result: SlotDisplayData::Item { item_id: 24 },
                crafting_station: SlotDisplayData::Item { item_id: 23 },
            },
            group: None,
            category_id: 4,
            crafting_requirements: None,
        },
        false,
        false,
    )
}

fn expected_recipe_book_add_payload() -> Vec<u8> {
    [
        vec![5],
        vec![0, 0, 2, 4, 1, 6, 16],
        b"minecraft:planks".to_vec(),
        vec![5, 2, 1, 0, 0, 4, 3, 0, 0, 0, 0],
        vec![
            1, 1, 2, 2, 4, 0, 4, 4, 9, 4, 5, 5, 6, 1, 0, 0, 10, 2, 4, 7, 6, 14,
        ],
        b"minecraft:logs".to_vec(),
        vec![4, 8, 4, 9, 1, 1, 1, 2, 2, 4, 0, 14],
        b"minecraft:wool".to_vec(),
        vec![3],
        vec![
            2, 2, 2, 4, 10, 1, 5, 11, 2, 0, 0, 4, 12, 0xc8, 0x01, 0x3f, 0x80, 0, 0, 0, 2, 0, 0,
        ],
        vec![3, 3, 3, 4, 13, 14, 7, 4, 15, 4, 16, 4, 17, 0, 3, 0, 0],
        vec![
            4, 4, 4, 18, 8, 4, 19, 4, 20, 21, 4, 22, 4, 24, 4, 23, 0, 4, 0, 0,
        ],
        vec![1],
    ]
    .concat()
}

#[test]
fn broad_play_packet_families_are_represented_as_distinct_instructions() {
    let instructions = representative_play_instructions();
    assert_representative_play_instruction_kinds(&instructions);
}

fn representative_play_instructions() -> Vec<PlayInstruction> {
    [
        representative_inventory_and_progress_instructions(),
        representative_feedback_instructions(),
        representative_world_and_debug_instructions(),
    ]
    .concat()
}

fn representative_inventory_and_progress_instructions() -> Vec<PlayInstruction> {
    vec![
        PlayInstruction::Container(ClientboundContainerPacket {
            container_id: 1,
            state_id: 2,
            slots: vec![
                RawItemStack {
                    count: 1,
                    item_id: Some(5),
                    components: RawDataComponentPatch::empty(),
                },
                RawItemStack::empty(),
            ],
            carried_item: RawItemStack::empty(),
        }),
        PlayInstruction::Recipes(ClientboundRecipePacket {
            recipes: vec![Identifier::parse("minecraft:stone").unwrap()],
        }),
        PlayInstruction::Advancements(ClientboundAdvancementsPacket {
            reset: true,
            added: vec![AdvancementHolderData::minimal(
                Identifier::parse("minecraft:story/root").unwrap(),
                None,
                vec![vec!["tick".to_string()]],
                true,
            )],
            removed: Vec::new(),
            progress: Vec::new(),
            show_advancements: true,
        }),
        PlayInstruction::AwardStats(ClientboundAwardStatsPacket {
            stats: vec![AwardedStat {
                stat_type_id: 8,
                stat_value_id: 23,
                value: 3,
            }],
        }),
        PlayInstruction::GameRuleValues(ClientboundGameRuleValuesPacket {
            values: BTreeMap::from([(
                Identifier::parse("minecraft:keep_inventory").unwrap(),
                "true".to_string(),
            )]),
        }),
    ]
}

fn representative_feedback_instructions() -> Vec<PlayInstruction> {
    vec![
        PlayInstruction::Scoreboard(ClientboundScoreboardPacket {
            objective: "sidebar".to_string(),
            owner: Some("Steve".to_string()),
            score: Some(10),
        }),
        PlayInstruction::BossEvent(ClientboundBossEventPacket {
            event_id: Uuid([2; 16]),
            operation: BossEventOperation::UpdateProgress { progress: 0.5 },
        }),
        PlayInstruction::Title(ClientboundTitlePacket {
            kind: TitlePacketKind::Times,
            text: None,
            fade_in: Some(10),
            stay: Some(70),
            fade_out: Some(20),
        }),
        PlayInstruction::Sound(ClientboundSoundPacket {
            sound: SoundEventHolder::Registered { id: 1 },
            source_id: 2,
            position: Vec3::ZERO,
            volume: 1.0,
            pitch: 1.0,
            seed: 99,
            entity_id: None,
        }),
        PlayInstruction::Particle(ClientboundParticlePacket {
            particle_id: 1,
            override_limiter: false,
            always_show: true,
            position: Vec3::ZERO,
            offset: Vec3::ZERO,
            max_speed: 0.0,
            count: 1,
            particle_data: Vec::new(),
        }),
        PlayInstruction::Explode(ClientboundExplodePacket {
            center: Vec3::ZERO,
            radius: 2.0,
            block_count: 0,
            player_knockback: None,
            explosion_particle: RawParticleOptions {
                particle_id: 1,
                data: Vec::new(),
            },
            explosion_sound: SoundEventHolder::Registered { id: 1 },
            block_particles: Vec::new(),
        }),
    ]
}

fn representative_world_and_debug_instructions() -> Vec<PlayInstruction> {
    vec![
        PlayInstruction::MapItemData(ClientboundMapItemDataPacket {
            map_id: 1,
            scale: 2,
            locked: false,
            decorations: Some(vec![MapDecorationData {
                decoration_type_id: 0,
                x: 1,
                y: 2,
                rotation: 3,
                name: None,
            }]),
            color_patch: Some(MapPatch {
                width: 1,
                height: 1,
                start_x: 0,
                start_y: 0,
                colors: vec![5],
            }),
        }),
        PlayInstruction::WorldBorder(ClientboundWorldBorderPacket {
            kind: WorldBorderPacketKind::Initialize,
            center: Some((0.0, 0.0)),
            old_size: Some(6.0e7),
            new_size: Some(6.0e7),
            lerp_time_ms: Some(0),
            warning_blocks: Some(5),
            warning_time: Some(15),
        }),
        PlayInstruction::Commands(ClientboundCommandsPacket::root_only()),
        PlayInstruction::CommandSuggestions(ClientboundCommandSuggestionsPacket {
            transaction_id: 4,
            start: 0,
            length: 2,
            suggestions: vec![CommandSuggestionEntry {
                text: "help".to_string(),
                tooltip: None,
            }],
        }),
        PlayInstruction::Debug(ClientboundDebugPacket {
            kind: DebugPacketKind::Sample,
            payload_size: 8,
        }),
    ]
}

fn assert_representative_play_instruction_kinds(instructions: &[PlayInstruction]) {
    assert_eq!(instructions.len(), 16);
    assert!(matches!(instructions[0], PlayInstruction::Container(_)));
    assert!(matches!(
        instructions[4],
        PlayInstruction::GameRuleValues(_)
    ));
    assert!(matches!(instructions[5], PlayInstruction::Scoreboard(_)));
    assert!(matches!(instructions[11], PlayInstruction::MapItemData(_)));
    assert!(matches!(instructions[12], PlayInstruction::WorldBorder(_)));
    assert!(matches!(instructions[15], PlayInstruction::Debug(_)));
}

#[test]
fn game_rule_values_packet_writes_registry_key_string_map() {
    let packet = ClientboundGameRuleValuesPacket {
        values: BTreeMap::from([
            (
                Identifier::parse("minecraft:keep_inventory").unwrap(),
                "true".to_string(),
            ),
            (
                Identifier::parse("minecraft:random_tick_speed").unwrap(),
                "3".to_string(),
            ),
        ]),
    };
    let mut bytes = Vec::new();
    packet.write(&mut bytes).unwrap();
    let mut input = cursor(bytes);

    assert_eq!(read_var_i32(&mut input).unwrap(), 2);
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:keep_inventory").unwrap()
    );
    assert_eq!(read_string(&mut input, 32767).unwrap(), "true");
    assert_eq!(
        read_identifier(&mut input).unwrap(),
        Identifier::parse("minecraft:random_tick_speed").unwrap()
    );
    assert_eq!(read_string(&mut input, 32767).unwrap(), "3");
}

#[test]
fn entity_spawn_bundle_preserves_vanilla_spawn_then_state_update_order() {
    let spawn = test_add_entity_packet();
    assert_entity_spawn_rotation_and_velocity(&spawn);
    let instructions = test_entity_spawn_bundle_instructions(spawn);
    assert_entity_spawn_bundle_instruction_order(&instructions);
    assert_entity_spawn_bundle_payloads(instructions);
}

fn test_add_entity_packet() -> ClientboundAddEntityPacket {
    ClientboundAddEntityPacket::new(AddEntityPacketInput {
        id: 7,
        uuid: Uuid([1; 16]),
        entity_type: 42,
        position: Vec3 {
            x: 1.0,
            y: 65.0,
            z: -2.0,
        },
        movement: Vec3 {
            x: 4.5,
            y: -4.5,
            z: 0.25,
        },
        rotation: (45.0, 90.0),
        y_head_rot: 180.0,
        data: 3,
    })
}

fn assert_entity_spawn_rotation_and_velocity(spawn: &ClientboundAddEntityPacket) {
    assert_eq!(spawn.x_rot, 32);
    assert_eq!(spawn.y_rot, 64);
    assert_eq!(spawn.y_head_rot, 128);

    // 26.1.2 uses LpVec3 (ABS_MAX ~1.7e10), not the legacy ±3.9 clamp.
    let velocity = ClientboundSetEntityMotionPacket::new(7, spawn.movement);
    assert_eq!(
        velocity.movement,
        Vec3 {
            x: 4.5,
            y: -4.5,
            z: 0.25,
        }
    );
}

fn test_entity_spawn_bundle_instructions(
    spawn: ClientboundAddEntityPacket,
) -> Vec<PlayInstruction> {
    let velocity = ClientboundSetEntityMotionPacket::new(7, spawn.movement);
    EntitySpawnBundle {
        spawn: spawn.clone(),
        metadata: Some(ClientboundSetEntityDataPacket {
            id: 7,
            packed_items: vec![EntityDataValue {
                index: 0,
                serializer_id: 0,
                encoded_payload: vec![0x20],
            }],
        }),
        velocity: Some(velocity),
        equipment: Some(ClientboundSetEquipmentPacket {
            entity: 7,
            slots: vec![
                EquipmentEntry {
                    slot: EquipmentSlotKind::MainHand,
                    item_stack: RawItemStack {
                        count: 1,
                        item_id: Some(1),
                        components: RawDataComponentPatch::empty(),
                    },
                },
                EquipmentEntry {
                    slot: EquipmentSlotKind::Head,
                    item_stack: RawItemStack {
                        count: 1,
                        item_id: Some(2),
                        components: RawDataComponentPatch::empty(),
                    },
                },
            ],
        }),
        attributes: Some(ClientboundUpdateAttributesPacket {
            entity_id: 7,
            attributes: vec![AttributeSnapshot {
                attribute_id: 0,
                base: 20.0,
                modifiers: Vec::new(),
            }],
        }),
        effects: vec![ClientboundUpdateMobEffectPacket {
            entity_id: 7,
            effect_id: 1,
            amplifier: 0,
            duration_ticks: 200,
            flags: MobEffectFlags::from_parts(false, true, true, true),
        }],
    }
    .instructions()
}

fn assert_entity_spawn_bundle_instruction_order(instructions: &[PlayInstruction]) {
    assert!(matches!(instructions[0], PlayInstruction::AddEntity(_)));
    assert!(matches!(instructions[1], PlayInstruction::SetEntityData(_)));
    assert!(matches!(
        instructions[2],
        PlayInstruction::SetEntityMotion(_)
    ));
    assert!(matches!(instructions[3], PlayInstruction::SetEquipment(_)));
    assert!(matches!(
        instructions[4],
        PlayInstruction::UpdateAttributes(_)
    ));
    assert!(matches!(
        instructions[5],
        PlayInstruction::UpdateMobEffect(_)
    ));
}

fn assert_entity_spawn_bundle_payloads(instructions: Vec<PlayInstruction>) {
    let PlayInstruction::SetEntityData(metadata) = &instructions[1] else {
        panic!("expected set entity data instruction");
    };
    let mut metadata_payload = Vec::new();
    metadata.write(&mut metadata_payload).unwrap();
    assert_eq!(metadata_payload, vec![7, 0, 0, 0x20, 0xff]);
    let PlayInstruction::SetEquipment(equipment) = &instructions[3] else {
        panic!("expected equipment packet");
    };
    assert_eq!(equipment.encoded_slot_bytes(), vec![0x80, 5]);
    let mut equipment_payload = Vec::new();
    equipment.write(&mut equipment_payload).unwrap();
    assert_eq!(
        equipment_payload,
        vec![7, 0x80, 1, 1, 0, 0, 5, 1, 2, 0, 0],
        "entity id, continued main-hand item stack, final head item stack"
    );
    let PlayInstruction::UpdateMobEffect(effect) = instructions[5] else {
        panic!("expected effect packet");
    };
    assert_eq!(effect.flags, MobEffectFlags(14));
}

#[test]
fn entity_metadata_values_use_vanilla_26_1_2_serializer_ids_and_payloads() {
    let values = metadata_serializer_test_values();

    assert_eq!(
        values
            .iter()
            .map(|value| value.serializer_id)
            .collect::<Vec<_>>(),
        (0..=42).collect::<Vec<_>>()
    );

    let mut payload = Vec::new();
    ClientboundSetEntityDataPacket {
        id: 99,
        packed_items: values,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(&payload[..3], &[99, 0, 0]);
    assert_eq!(payload.last(), Some(&0xff));
    assert!(payload.windows(3).any(|window| window == [33, 33, 1]));
    assert!(payload.windows(3).any(|window| window == [40, 40, 0x3f]));
    assert!(payload.windows(3).any(|window| window == [42, 42, 1]));
}

fn metadata_serializer_test_values() -> Vec<EntityDataValue> {
    let component = vec![0x08, b'{', b'}'];
    let stack = RawItemStack {
        count: 2,
        item_id: Some(5),
        components: RawDataComponentPatch::empty(),
    };
    [
        metadata_scalar_values(component, stack),
        metadata_variant_values(),
        metadata_geometry_and_profile_values(),
    ]
    .concat()
}

fn metadata_scalar_values(component: Vec<u8>, stack: RawItemStack) -> Vec<EntityDataValue> {
    vec![
        EntityDataValue::typed(0, EntityMetadataValue::Byte(-1)).unwrap(),
        EntityDataValue::typed(1, EntityMetadataValue::VarInt(300)).unwrap(),
        EntityDataValue::typed(2, EntityMetadataValue::VarLong(300)).unwrap(),
        EntityDataValue::typed(3, EntityMetadataValue::Float(1.5)).unwrap(),
        EntityDataValue::typed(4, EntityMetadataValue::String("abc".to_string())).unwrap(),
        EntityDataValue::typed(5, EntityMetadataValue::Component(component.clone())).unwrap(),
        EntityDataValue::typed(6, EntityMetadataValue::OptionalComponent(Some(component))).unwrap(),
        EntityDataValue::typed(7, EntityMetadataValue::ItemStack(stack)).unwrap(),
        EntityDataValue::typed(8, EntityMetadataValue::Boolean(true)).unwrap(),
        EntityDataValue::typed(
            9,
            EntityMetadataValue::Rotations(Rotations {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }),
        )
        .unwrap(),
        EntityDataValue::typed(
            10,
            EntityMetadataValue::BlockPos(BlockPosition { x: 1, y: 2, z: 3 }),
        )
        .unwrap(),
        EntityDataValue::typed(11, EntityMetadataValue::OptionalBlockPos(None)).unwrap(),
        EntityDataValue::typed(12, EntityMetadataValue::Direction(DirectionData::East)).unwrap(),
        EntityDataValue::typed(
            13,
            EntityMetadataValue::OptionalLivingEntityReference(Some(42)),
        )
        .unwrap(),
        EntityDataValue::typed(14, EntityMetadataValue::BlockState(9)).unwrap(),
        EntityDataValue::typed(15, EntityMetadataValue::OptionalBlockState(None)).unwrap(),
        EntityDataValue::typed(
            16,
            EntityMetadataValue::Particle(RawParticleOptions {
                particle_id: 3,
                data: vec![0xaa],
            }),
        )
        .unwrap(),
        EntityDataValue::typed(
            17,
            EntityMetadataValue::Particles(vec![RawParticleOptions {
                particle_id: 4,
                data: vec![0xbb],
            }]),
        )
        .unwrap(),
    ]
}

fn metadata_variant_values() -> Vec<EntityDataValue> {
    vec![
        EntityDataValue::typed(
            18,
            EntityMetadataValue::VillagerData(VillagerData {
                villager_type: 1,
                profession: 2,
                level: 3,
            }),
        )
        .unwrap(),
        EntityDataValue::typed(19, EntityMetadataValue::OptionalUnsignedInt(Some(4))).unwrap(),
        EntityDataValue::typed(20, EntityMetadataValue::Pose(PoseData::Crouching)).unwrap(),
        EntityDataValue::typed(21, EntityMetadataValue::CatVariant(5)).unwrap(),
        EntityDataValue::typed(22, EntityMetadataValue::CatSoundVariant(6)).unwrap(),
        EntityDataValue::typed(23, EntityMetadataValue::CowVariant(7)).unwrap(),
        EntityDataValue::typed(24, EntityMetadataValue::CowSoundVariant(8)).unwrap(),
        EntityDataValue::typed(25, EntityMetadataValue::WolfVariant(9)).unwrap(),
        EntityDataValue::typed(26, EntityMetadataValue::WolfSoundVariant(10)).unwrap(),
        EntityDataValue::typed(27, EntityMetadataValue::FrogVariant(11)).unwrap(),
        EntityDataValue::typed(28, EntityMetadataValue::PigVariant(12)).unwrap(),
        EntityDataValue::typed(29, EntityMetadataValue::PigSoundVariant(13)).unwrap(),
        EntityDataValue::typed(30, EntityMetadataValue::ChickenVariant(14)).unwrap(),
        EntityDataValue::typed(31, EntityMetadataValue::ChickenSoundVariant(15)).unwrap(),
        EntityDataValue::typed(32, EntityMetadataValue::ZombieNautilusVariant(16)).unwrap(),
        EntityDataValue::typed(
            33,
            EntityMetadataValue::OptionalGlobalPos(Some(GlobalPosData {
                dimension: Identifier::parse("minecraft:overworld").unwrap(),
                pos: BlockPosition { x: 1, y: 2, z: 3 },
            })),
        )
        .unwrap(),
        EntityDataValue::typed(34, EntityMetadataValue::PaintingVariant(17)).unwrap(),
        EntityDataValue::typed(
            35,
            EntityMetadataValue::SnifferState(SnifferStateData::Digging),
        )
        .unwrap(),
        EntityDataValue::typed(
            36,
            EntityMetadataValue::ArmadilloState(ArmadilloStateData::Rolling),
        )
        .unwrap(),
        EntityDataValue::typed(
            37,
            EntityMetadataValue::CopperGolemState(CopperGolemStateData::Weathered),
        )
        .unwrap(),
        EntityDataValue::typed(
            38,
            EntityMetadataValue::WeatheringCopperState(WeatheringCopperStateData::Oxidized),
        )
        .unwrap(),
    ]
}

fn metadata_geometry_and_profile_values() -> Vec<EntityDataValue> {
    vec![
        EntityDataValue::typed(
            39,
            EntityMetadataValue::Vector3f(Vector3fData {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }),
        )
        .unwrap(),
        EntityDataValue::typed(
            40,
            EntityMetadataValue::Quaternionf(QuaternionfData {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                w: 4.0,
            }),
        )
        .unwrap(),
        EntityDataValue::typed(41, EntityMetadataValue::ResolvableProfile(vec![0])).unwrap(),
        EntityDataValue::typed(42, EntityMetadataValue::HumanoidArm(HumanoidArmData::Right))
            .unwrap(),
    ]
}

mod add_entity_packet_test;
mod block_entity_data_packet_test;
mod boss_event_packet_test;
mod chat_ack_packet_test;
mod chat_command_packet_test;
mod chat_command_signed_packet_test;
mod chat_packet_test;
mod chat_session_update_packet_test;
mod clientbound_tag_query_packet_test;
mod clientbound_game_test_highlight_pos_packet_test;
mod clientbound_move_vehicle_packet_test;
mod clientbound_player_abilities_packet_test;
mod clientbound_player_combat_packets_test;
mod clientbound_player_rotation_packet_test;
mod clientbound_projectile_power_packet_test;
mod clientbound_select_advancements_tab_packet_test;
mod clientbound_server_data_packet_test;
mod clientbound_set_camera_packet_test;
mod clientbound_set_chunk_cache_packets_test;
mod clientbound_tracked_waypoint_packet_test;
mod command_suggestion_packet_test;
mod configuration_acknowledged_packet_test;
#[cfg(vibecraft_has_decompiled_sources)]
mod container_click_packet_test;
mod container_close_packet_test;
mod container_set_content_packet_test;
mod container_set_data_packet_test;
mod container_set_slot_packet_test;
mod container_slot_state_changed_packet_test;
mod cooldown_packet_test;
mod custom_payload_packet_test;
mod delete_chat_packet_test;
mod disguised_chat_packet_test;
mod entity_inventory_clientbound_packets_test;
mod entity_movement_test;
mod entity_position_sync_packet_test;
mod explode_packet_test;
mod initialize_border_packet_test;
mod inventory_packet_item_stack_test;
mod level_particles_packet_test;
mod merchant_offers_packet_test;
mod mid_clientbound_game_packets_test;
mod mount_screen_open_packet_test;
mod named_sound_effect_absence_test;
mod open_screen_packet_test;
mod player_chat_packet_test;
mod player_info_remove_packet_test;
mod player_info_update_packet_test;
mod clientbound_player_look_at_packet_test;
mod player_position_packet_test;
mod recipe_book_remove_packet_test;
mod recipe_book_settings_packet_test;
mod scoreboard_display_clientbound_packets_test;
mod ui_world_clientbound_packets_test;
mod reset_score_packet_test;
mod resource_pack_packet_test;
mod section_blocks_update_packet_test;
mod set_action_bar_text_packet_test;
mod set_beacon_packet_test;
mod set_border_center_packet_test;
mod set_border_lerp_size_packet_test;
mod set_border_size_packet_test;
mod set_border_warning_delay_packet_test;
mod set_border_warning_distance_packet_test;
mod set_display_objective_packet_test;
mod set_entity_data_packet_test;
mod set_equipment_packet_test;
mod set_objective_packet_test;
mod set_player_team_packet_test;
mod set_score_packet_test;
mod set_subtitle_text_packet_test;
mod set_title_text_packet_test;
mod set_titles_animation_packet_test;
mod serverbound_advancement_misc_packets_source_test;
mod serverbound_block_edit_packets_source_test;
mod serverbound_block_entity_tag_query_packet_test;
mod serverbound_change_game_mode_packet_test;
mod serverbound_debug_subscription_request_packet_test;
mod serverbound_simple_packets_test;
mod serverbound_container_misc_packets_test;
mod serverbound_entity_tag_query_packet_test;
mod serverbound_hand_item_packets_source_test;
mod serverbound_jigsaw_test_packets_source_test;
mod serverbound_pick_rename_trade_packets_source_test;
mod serverbound_player_action_source_test;
mod serverbound_player_loaded_packet_test;
mod simple_clientbound_game_packets_test;
mod small_play_packets_test;
mod sound_entity_packet_test;
mod sound_packet_test;
mod start_configuration_packet_test;
mod stop_sound_packet_test;
mod system_chat_packet_test;
mod tab_list_packet_test;
mod ticking_packets_test;
mod tests_middle;
mod tests_middle2;
mod tests_middle3;
mod update_advancements_packet_test;
mod update_mob_effect_packet_test;
pub use tests_middle2::*;
mod tests_end;
