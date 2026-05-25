use super::super::*;
use super::*;

#[test]
fn load_static_reads_id_components_and_custom_payload() {
    let tag = Tag::Compound(vec![
        (
            "id".to_string(),
            Tag::String("minecraft:campfire".to_string()),
        ),
        ("x".to_string(), Tag::Int(18)),
        ("y".to_string(), Tag::Int(64)),
        ("z".to_string(), Tag::Int(35)),
        ("CookingTimes".to_string(), Tag::List(vec![Tag::Int(10)])),
        (
            "components".to_string(),
            Tag::Compound(vec![(
                "minecraft:lore".to_string(),
                Tag::String("[]".to_string()),
            )]),
        ),
    ]);
    let entity = load_static(pos(), "minecraft:campfire", &tag).unwrap();
    assert_eq!(entity.ty, BlockEntityTypeId::Campfire);
    assert!(entity.custom_data.contains_key("CookingTimes"));
    assert!(entity.components.contains_key("minecraft:lore"));
}

#[test]
fn load_static_with_data_version_refuses_unsafe_migrations() {
    let tag = Tag::Compound(vec![(
        "id".to_string(),
        Tag::String("minecraft:campfire".to_string()),
    )]);

    let entity = load_static_with_data_version(
        pos(),
        "minecraft:campfire",
        &tag,
        crate::storage::datafix::TARGET_DATA_VERSION,
    )
    .unwrap();
    assert_eq!(entity.ty, BlockEntityTypeId::Campfire);

    assert!(matches!(
        load_static_with_data_version(
            pos(),
            "minecraft:campfire",
            &tag,
            crate::storage::datafix::TARGET_DATA_VERSION - 1,
        ),
        Err(BlockEntityError::UnsupportedDataVersion(message))
            if message.contains("unsafe migrations")
    ));
}

#[test]
fn save_load_round_trip_preserves_generic_fields_for_every_type() {
    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
        entity
            .custom_data
            .insert("CustomInt".to_string(), Tag::Int(42));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Round Trip\"".to_string()),
        );

        let tag = entity.save_with_full_metadata();
        let loaded = load_static(pos(), block_state, &tag).unwrap();

        assert_eq!(loaded.ty, entity.ty, "type failed for {}", info.key);
        assert_eq!(loaded.pos, entity.pos, "position failed for {}", info.key);
        assert_eq!(
            loaded.block_state, entity.block_state,
            "block state failed for {}",
            info.key
        );
        assert_eq!(
            loaded.custom_data, entity.custom_data,
            "custom data failed for {}",
            info.key
        );
        assert_eq!(
            loaded.components, entity.components,
            "components failed for {}",
            info.key
        );
        assert!(
            !loaded.has_level,
            "level attachment leaked for {}",
            info.key
        );
        assert!(!loaded.removed, "removed flag leaked for {}", info.key);
        assert!(!loaded.changed, "changed flag leaked for {}", info.key);
        assert_eq!(loaded.tick_count, 0, "tick count leaked for {}", info.key);
    }
}

#[test]
fn placement_then_chunk_unload_reload_preserves_block_entity_nbt_for_every_type() {
    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let placed_pos = BlockPos {
            x: 18,
            y: 73,
            z: -29,
        };
        let mut placed = BlockEntity::new(info.id, placed_pos, block_state).unwrap();
        placed.set_level();
        placed.set_changed();
        placed.tick_count = 99;
        placed
            .custom_data
            .insert("ChunkUnloadProbe".to_string(), Tag::Long(123_456));
        placed.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String(format!("\"{}\"", info.key)),
        );

        let saved_at_unload = placed.save_with_full_metadata();
        let reloaded = load_static(placed_pos, block_state, &saved_at_unload).unwrap();
        let saved_after_reload = reloaded.save_with_full_metadata();

        assert_eq!(
            saved_after_reload, saved_at_unload,
            "chunk unload/reload NBT identity failed for {}",
            info.key
        );
        assert!(
            !reloaded.has_level && !reloaded.changed && !reloaded.removed,
            "runtime placement flags leaked through chunk reload for {}",
            info.key
        );
        assert_eq!(
            reloaded.tick_count, 0,
            "scheduler tick state leaked through chunk reload for {}",
            info.key
        );
    }
}

#[test]
fn data_get_block_exposes_full_nbt_for_every_block_entity_type() {
    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let probe_pos = BlockPos {
            x: -12,
            y: 81,
            z: 44,
        };
        let mut entity = BlockEntity::new(info.id, probe_pos, block_state).unwrap();
        entity
            .custom_data
            .insert("DataProbe".to_string(), Tag::String(info.key.to_string()));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Data Probe\"".to_string()),
        );

        let tag = entity.data_get_block_nbt();
        let entries = compound_entries(&tag).expect("/data get block result is compound");

        assert_eq!(
            get_string(entries, "id"),
            Some(info.key),
            "/data id failed for {}",
            info.key
        );
        assert_eq!(get_int(entries, "x"), Some(probe_pos.x));
        assert_eq!(get_int(entries, "y"), Some(probe_pos.y));
        assert_eq!(get_int(entries, "z"), Some(probe_pos.z));
        assert!(
            entries
                .iter()
                .any(|(key, value)| key == "DataProbe"
                    && value == &Tag::String(info.key.to_string())),
            "/data custom field missing for {}",
            info.key
        );
        assert!(
            entries.iter().any(|(key, value)| {
                key == "components"
                    && matches!(value, Tag::Compound(values) if values.iter().any(
                        |(component_key, component_value)| component_key == "minecraft:custom_name"
                            && component_value == &Tag::String("\"Data Probe\"".to_string())
                    ))
            }),
            "/data components missing for {}",
            info.key
        );

        let loaded = load_static(probe_pos, block_state, &tag).unwrap();
        assert_eq!(
            loaded.custom_data, entity.custom_data,
            "{} custom data",
            info.key
        );
        assert_eq!(
            loaded.components, entity.components,
            "{} components",
            info.key
        );
    }
}

#[test]
fn destruction_drops_cover_tool_silk_explosion_gamerule_and_stored_items_for_every_type() {
    let correct_tool = BlockEntityDestructionContext {
        correct_tool: true,
        silk_touch: false,
        explosion_survives: true,
        do_tile_drops: true,
    };
    let silk_touch = BlockEntityDestructionContext {
        correct_tool: false,
        silk_touch: true,
        explosion_survives: true,
        do_tile_drops: true,
    };
    let wrong_tool = BlockEntityDestructionContext {
        correct_tool: false,
        silk_touch: false,
        explosion_survives: true,
        do_tile_drops: true,
    };
    let explosion_consumed = BlockEntityDestructionContext {
        correct_tool: true,
        silk_touch: false,
        explosion_survives: false,
        do_tile_drops: true,
    };
    let tile_drops_disabled = BlockEntityDestructionContext {
        correct_tool: true,
        silk_touch: false,
        explosion_survives: true,
        do_tile_drops: false,
    };

    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
        entity.custom_data.insert(
            "Items".to_string(),
            Tag::List(vec![stack("minecraft:apple", 2).to_tag()]),
        );
        entity
            .custom_data
            .insert("item".to_string(), stack("minecraft:diamond", 1).to_tag());
        entity.custom_data.insert(
            "Book".to_string(),
            stack("minecraft:written_book", 1).to_tag(),
        );
        entity.custom_data.insert(
            "RecordItem".to_string(),
            stack("minecraft:music_disc_13", 1).to_tag(),
        );

        assert_eq!(
            entity.destruction_drops(correct_tool).block_item.as_deref(),
            Some(block_item_from_state(block_state)),
            "{} correct-tool block drop",
            info.key
        );
        assert_eq!(
            entity.destruction_drops(silk_touch).block_item.as_deref(),
            Some(block_item_from_state(block_state)),
            "{} silk-touch block drop",
            info.key
        );

        let wrong_tool_drops = entity.destruction_drops(wrong_tool);
        assert_eq!(
            wrong_tool_drops.block_item, None,
            "{} wrong-tool block drop",
            info.key
        );
        assert_eq!(
            wrong_tool_drops.stored_items,
            vec![
                stack("minecraft:apple", 2),
                stack("minecraft:diamond", 1),
                stack("minecraft:written_book", 1),
                stack("minecraft:music_disc_13", 1),
            ],
            "{} stored item drops",
            info.key
        );

        assert_eq!(
            entity.destruction_drops(explosion_consumed),
            BlockEntityDestructionDrops {
                block_item: None,
                stored_items: Vec::new(),
            },
            "{} explosion consumed drops",
            info.key
        );
        assert_eq!(
            entity.destruction_drops(tile_drops_disabled),
            BlockEntityDestructionDrops {
                block_item: None,
                stored_items: Vec::new(),
            },
            "{} doTileDrops=false drops",
            info.key
        );
    }
}

#[test]
fn update_tag_subset_is_stable_for_every_type() {
    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
        entity
            .custom_data
            .insert("CustomInt".to_string(), Tag::Int(42));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Update Tag\"".to_string()),
        );

        let tag = entity.get_update_tag();
        let entries = compound_entries(&tag).expect("update tag is compound");

        assert!(
            entries
                .iter()
                .all(|(key, _)| key != "id" && key != "x" && key != "y" && key != "z"),
            "metadata leaked into update tag for {}",
            info.key
        );

        match info.id {
            BlockEntityTypeId::Chest
            | BlockEntityTypeId::TrappedChest
            | BlockEntityTypeId::Barrel
            | BlockEntityTypeId::Hopper
            | BlockEntityTypeId::Dispenser
            | BlockEntityTypeId::Dropper => {
                assert!(
                    entries.is_empty(),
                    "container inventory data leaked into update tag for {}",
                    info.key
                );
            }
            _ => {
                assert!(
                    entries
                        .iter()
                        .any(|(key, value)| key == "CustomInt" && *value == Tag::Int(42)),
                    "custom data missing from update tag for {}",
                    info.key
                );
                assert!(
                    entries.iter().any(|(key, _)| key == "components"),
                    "components missing from update tag for {}",
                    info.key
                );
            }
        }
    }
}

#[test]
fn block_entity_data_packets_preserve_update_tag_subset_for_every_type() {
    for info in BLOCK_ENTITY_TYPES {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has at least one valid block");
        let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
        entity
            .custom_data
            .insert("PacketProbe".to_string(), Tag::String(info.key.to_string()));
        entity.components.insert(
            "minecraft:custom_name".to_string(),
            Tag::String("\"Packet Probe\"".to_string()),
        );

        let update_tag = entity.get_update_tag();
        let packet = entity.get_update_packet();
        assert_eq!(packet.pos, pos(), "{} packet pos", info.key);
        assert_eq!(packet.ty, info.id, "{} packet type", info.key);
        assert_eq!(packet.tag, update_tag, "{} packet tag", info.key);

        let (packed_xz, y, ty, chunk_tag) = block_entity_packet_from_chunk(&entity, -64);
        assert_eq!(packed_xz, 0x23, "{} chunk packet local x/z", info.key);
        assert_eq!(y, 128, "{} chunk packet y", info.key);
        assert_eq!(ty, info.id, "{} chunk packet type", info.key);
        assert_eq!(chunk_tag, update_tag, "{} chunk packet tag", info.key);

        let entries = compound_entries(&chunk_tag).expect("block entity data tag is compound");
        assert!(
            entries
                .iter()
                .all(|(key, _)| key != "id" && key != "x" && key != "y" && key != "z"),
            "metadata leaked into block entity data packet for {}",
            info.key
        );
    }
}

#[test]
fn wrong_chunk_positions_are_corrected_like_vanilla() {
    let tag = Tag::Compound(vec![
        ("x".to_string(), Tag::Int(34)),
        ("y".to_string(), Tag::Int(-20)),
        ("z".to_string(), Tag::Int(-17)),
    ]);
    assert_eq!(
        corrected_pos_from_chunk(0, 0, &tag),
        BlockPos {
            x: 2,
            y: -20,
            z: 15
        }
    );
}

#[test]
fn ticking_requires_level_side_match_and_not_removed() {
    let mut furnace =
        BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
    assert!(!furnace.tick(false));
    furnace.set_level();
    assert!(furnace.tick(false));
    assert!(!furnace.tick(true));
    furnace.set_removed();
    assert!(!furnace.tick(false));

    let mut conduit =
        BlockEntity::new(BlockEntityTypeId::Conduit, pos(), "minecraft:conduit").unwrap();
    conduit.set_level();
    assert!(conduit.tick(false));
    assert!(conduit.tick(true));
}

#[test]
fn tick_dispatch_advances_scheduler_state_for_every_tickable_block_entity_type() {
    let tickable: Vec<&BlockEntityTypeInfo> = BLOCK_ENTITY_TYPES
        .iter()
        .filter(|info| info.tick_kind != BlockEntityTickKind::None)
        .collect();
    assert_eq!(tickable.len(), 32);

    for info in tickable {
        let block_state = info
            .valid_blocks
            .first()
            .expect("every block entity type has a valid block");
        let mut entity = BlockEntity::new(info.id, pos(), block_state).unwrap();
        entity.set_level();

        let server_ticks = matches!(
            info.tick_kind,
            BlockEntityTickKind::Server | BlockEntityTickKind::Both
        );
        let client_ticks = matches!(
            info.tick_kind,
            BlockEntityTickKind::Client | BlockEntityTickKind::Both
        );

        assert_eq!(
            entity.tick(false),
            server_ticks,
            "{} server tick dispatch",
            info.key
        );
        assert_eq!(
            entity.tick_count,
            u64::from(server_ticks),
            "{} server tick count",
            info.key
        );
        assert_eq!(
            entity.tick(true),
            client_ticks,
            "{} client tick dispatch",
            info.key
        );
        assert_eq!(
            entity.tick_count,
            u64::from(server_ticks) + u64::from(client_ticks),
            "{} client tick count",
            info.key
        );

        entity.set_removed();
        assert!(
            !entity.tick(false) && !entity.tick(true),
            "{} removed entity ticked",
            info.key
        );
    }
}

#[test]
fn ticking_block_entity_wrapper_exposes_scheduler_shape() {
    let mut furnace =
        BlockEntity::new(BlockEntityTypeId::Furnace, pos(), "minecraft:furnace").unwrap();
    furnace.set_level();
    let mut ticker = TickingBlockEntity::new(furnace, false);

    assert_eq!(ticker.pos(), pos());
    assert_eq!(ticker.type_key(), "furnace");
    assert!(!ticker.is_removed());
    assert!(ticker.tick());
    assert_eq!(ticker.entity.tick_count, 1);

    ticker.entity.set_removed();
    assert!(ticker.is_removed());
    assert!(!ticker.tick());

    ticker.entity.clear_removed();
    assert!(!ticker.is_removed());
    assert!(ticker.tick());
    assert_eq!(ticker.entity.tick_count, 2);
}

#[test]
fn changed_flag_only_sets_when_attached_to_level() {
    let mut entity = BlockEntity::new(BlockEntityTypeId::Bell, pos(), "minecraft:bell").unwrap();
    entity.set_changed();
    assert!(!entity.changed);
    entity.set_level();
    entity.set_changed();
    assert!(entity.changed);
}

#[test]
fn update_packets_use_position_type_and_update_tag() {
    let mut sign = BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
    sign.custom_data
        .insert("front_text".to_string(), Tag::String("hi".to_string()));
    let packet = sign.get_update_packet();
    assert_eq!(packet.pos, pos());
    assert_eq!(packet.ty, BlockEntityTypeId::Sign);
    assert!(
        matches!(packet.tag, Tag::Compound(values) if values.iter().any(|(k, _)| k == "front_text"))
    );

    let chest = BlockEntity::new(BlockEntityTypeId::Chest, pos(), "minecraft:chest").unwrap();
    assert_eq!(chest.get_update_tag(), Tag::Compound(Vec::new()));
}

#[test]
fn handle_update_tag_applies_network_subset_without_metadata() {
    let mut entity =
        BlockEntity::new(BlockEntityTypeId::Sign, pos(), "minecraft:oak_sign").unwrap();
    entity
        .custom_data
        .insert("old_text".to_string(), Tag::String("stale".to_string()));
    entity
        .components
        .insert("old_component".to_string(), Tag::Int(1));

    entity.handle_update_tag(&Tag::Compound(vec![
        ("x".to_string(), Tag::Int(999)),
        ("id".to_string(), Tag::String("minecraft:chest".to_string())),
        ("front_text".to_string(), Tag::String("hello".to_string())),
        (
            "components".to_string(),
            Tag::Compound(vec![(
                "minecraft:custom_name".to_string(),
                Tag::String("Sign".to_string()),
            )]),
        ),
    ]));

    assert_eq!(entity.ty, BlockEntityTypeId::Sign);
    assert_eq!(entity.pos, pos());
    assert!(!entity.custom_data.contains_key("old_text"));
    assert_eq!(
        entity.custom_data.get("front_text"),
        Some(&Tag::String("hello".to_string()))
    );
    assert!(!entity.components.contains_key("old_component"));
    assert_eq!(
        entity.components.get("minecraft:custom_name"),
        Some(&Tag::String("Sign".to_string()))
    );
}

#[test]
fn test_block_entity_state_saves_loads_and_tracks_triggers_like_java() {
    let mut state = TestBlockEntityState {
        mode: TestBlockMode::Start,
        message: "begin".to_string(),
        powered: false,
        triggered: true,
    };
    assert_eq!(
        state.save_additional(),
        Tag::Compound(vec![
            ("mode".to_string(), Tag::String("start".to_string())),
            ("message".to_string(), Tag::String("begin".to_string())),
            ("powered".to_string(), Tag::Byte(0)),
        ])
    );

    state.trigger();
    assert!(state.powered);
    assert!(state.triggered);
    state.reset();
    assert!(!state.powered);
    assert!(!state.triggered);

    let loaded = TestBlockEntityState::load_additional(&Tag::Compound(vec![
        ("mode".to_string(), Tag::String("accept".to_string())),
        ("message".to_string(), Tag::String("done".to_string())),
        ("powered".to_string(), Tag::Byte(1)),
    ]));
    assert_eq!(loaded.mode, TestBlockMode::Accept);
    assert_eq!(loaded.message, "done");
    assert!(loaded.powered);
    assert!(!loaded.triggered);
    assert_eq!(
        TestBlockEntityState::load_additional(&Tag::Compound(Vec::new())).mode,
        TestBlockMode::Fail
    );
}

#[test]
fn test_instance_block_entity_state_saves_loads_status_and_errors() {
    let mut state = TestInstanceBlockEntityState {
        data: TestInstanceBlockEntityData {
            test: Some("minecraft:always_pass".to_string()),
            size: (3, 4, 5),
            rotation: "clockwise_90".to_string(),
            ignore_entities: true,
            status: TestInstanceStatus::Cleared,
            error_message: None,
        },
        errors: Vec::new(),
    };
    state.set_running();
    state.mark_error(BlockPos { x: 1, y: 2, z: 3 }, "bad block");
    state.set_error_message("failed");

    let saved = state.save_additional();
    let loaded = TestInstanceBlockEntityState::load_additional(&saved);
    assert_eq!(loaded.data.test.as_deref(), Some("minecraft:always_pass"));
    assert_eq!(loaded.data.size, (3, 4, 5));
    assert_eq!(loaded.data.rotation, "clockwise_90");
    assert!(loaded.data.ignore_entities);
    assert_eq!(loaded.data.status, TestInstanceStatus::Finished);
    assert_eq!(loaded.data.error_message.as_deref(), Some("failed"));
    assert_eq!(
        loaded.errors,
        vec![TestInstanceErrorMarker {
            pos: BlockPos { x: 1, y: 2, z: 3 },
            text: "bad block".to_string(),
        }]
    );

    let mut success = loaded.clone();
    success.set_success();
    assert_eq!(success.data.status, TestInstanceStatus::Finished);
    assert_eq!(success.data.error_message, None);
    success.clear_error_markers();
    assert!(success.errors.is_empty());
}

#[test]
fn chunk_packet_data_packs_local_xz_y_type_and_tag() {
    let entity = BlockEntity::new(BlockEntityTypeId::Vault, pos(), "minecraft:vault").unwrap();
    let (packed_xz, y, ty, tag) = block_entity_packet_from_chunk(&entity, -64);
    assert_eq!(packed_xz, 0x23);
    assert_eq!(y, 128);
    assert_eq!(ty, BlockEntityTypeId::Vault);
    assert_eq!(
        tag,
        Tag::Compound(vec![("components".to_string(), Tag::Compound(Vec::new()))])
    );
}
