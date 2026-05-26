use super::*;

#[test]
fn item_command_modifies_slots_and_clamps_to_stack_size() {
    let mut state = ServerCommandState::default();
    state.entity_item_slots.push(CommandEntityItemSlot {
        entity: EntityRef {
            id: "Steve".to_string(),
            display_name: "Steve".to_string(),
        },
        slot: "hotbar.0".to_string(),
        item: Some(CommandItemStack {
            item: "minecraft:stone".to_string(),
            count: 80,
        }),
    });

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "item modify entity Steve hotbar.0 minecraft:set_count",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(
        state.entity_item_slots[0].item,
        Some(CommandItemStack {
            item: "minecraft:stone".to_string(),
            count: 64,
        })
    );
    assert_eq!(
        state.item_modifier_events,
        vec![CommandItemModifierEvent {
            target: CommandItemTarget::Entity {
                entity: EntityRef {
                    id: "Steve".to_string(),
                    display_name: "Steve".to_string(),
                },
                slot: "hotbar.0".to_string(),
            },
            modifier: "minecraft:set_count".to_string(),
            input: Some(CommandItemStack {
                item: "minecraft:stone".to_string(),
                count: 80,
            }),
            output: Some(CommandItemStack {
                item: "minecraft:stone".to_string(),
                count: 64,
            }),
        }]
    );
}

#[test]
fn item_command_rejects_invalid_counts_slots_and_missing_sources() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "item replace entity Steve hotbar.0 with stone 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "item replace entity Steve hotbar.0 with stone 100"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "item replace entity Steve bad_slot with stone"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "item replace entity Steve hotbar.1 from entity Alex hotbar.0"
        ),
        Err(CommandError::ItemSourceNoSuchSlot)
    );
}

#[test]
fn locate_command_finds_nearest_structure_biome_and_poi() {
    let mut state = locate_test_state();
    assert_locate_permission_gate(&mut state);

    let structure = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "locate structure #minecraft:village",
    )
    .unwrap();
    assert_eq!(structure.success_count, 200);
    assert_eq!(structure.feedback_key, "commands.locate.structure.success");

    let biome = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "locate biome desert",
    )
    .unwrap();
    assert_eq!(biome.success_count, 143);
    assert_eq!(biome.feedback_key, "commands.locate.biome.success");

    let poi = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "locate poi armorer",
    )
    .unwrap();
    assert_eq!(poi.success_count, 50);
    assert_eq!(poi.feedback_key, "commands.locate.poi.success");
    assert_eq!(state.locate_results, expected_locate_results());
}

fn locate_test_state() -> ServerCommandState {
    ServerCommandState {
        command_source_position: Vec3 {
            x: 0.0,
            y: 64.0,
            z: 0.0,
        },
        locatable_entries: vec![
            CommandLocatableEntry {
                kind: LocateKind::Structure,
                id: "minecraft:village_plains".to_string(),
                tags: vec!["minecraft:village".to_string()],
                position: BlockPos {
                    x: 300,
                    y: 70,
                    z: 400,
                },
            },
            CommandLocatableEntry {
                kind: LocateKind::Structure,
                id: "minecraft:village_taiga".to_string(),
                tags: vec!["minecraft:village".to_string()],
                position: BlockPos {
                    x: 120,
                    y: 80,
                    z: 160,
                },
            },
            CommandLocatableEntry {
                kind: LocateKind::Biome,
                id: "minecraft:desert".to_string(),
                tags: vec!["minecraft:is_overworld".to_string()],
                position: BlockPos {
                    x: 0,
                    y: 128,
                    z: 128,
                },
            },
            CommandLocatableEntry {
                kind: LocateKind::Poi,
                id: "minecraft:armorer".to_string(),
                tags: vec!["minecraft:acquirable_job_site".to_string()],
                position: BlockPos {
                    x: 30,
                    y: 64,
                    z: 40,
                },
            },
        ],
        ..ServerCommandState::default()
    }
}

fn assert_locate_permission_gate(state: &mut ServerCommandState) {
    assert_eq!(
        command_required_permission("locate"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::MODERATOR,
            "locate structure #minecraft:village"
        ),
        Err(CommandError::PermissionDenied)
    );
}

fn expected_locate_results() -> Vec<CommandLocateResult> {
    vec![
        CommandLocateResult {
            kind: LocateKind::Structure,
            query: "#minecraft:village".to_string(),
            found_id: "minecraft:village_taiga".to_string(),
            position: BlockPos {
                x: 120,
                y: 80,
                z: 160,
            },
            distance: 200,
            include_y: false,
        },
        CommandLocateResult {
            kind: LocateKind::Biome,
            query: "minecraft:desert".to_string(),
            found_id: "minecraft:desert".to_string(),
            position: BlockPos {
                x: 0,
                y: 128,
                z: 128,
            },
            distance: 143,
            include_y: true,
        },
        CommandLocateResult {
            kind: LocateKind::Poi,
            query: "minecraft:armorer".to_string(),
            found_id: "minecraft:armorer".to_string(),
            position: BlockPos {
                x: 30,
                y: 64,
                z: 40,
            },
            distance: 50,
            include_y: false,
        },
    ]
}

#[test]
fn locate_command_reports_invalid_or_missing_targets() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "locate structure not_a_structure"
        ),
        Err(CommandError::LocateStructureInvalid)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "locate structure stronghold"
        ),
        Err(CommandError::LocateStructureNotFound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "locate biome desert"
        ),
        Err(CommandError::LocateBiomeNotFound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "locate poi armorer"
        ),
        Err(CommandError::LocatePoiNotFound)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "locate"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn loot_command_gives_spawns_and_inserts_generated_drops() {
    let mut state = ServerCommandState {
        command_loot_tables: vec![CommandLootTable {
            id: "minecraft:chests/simple_dungeon".to_string(),
            drops: vec![
                CommandItemStack {
                    item: "minecraft:iron_ingot".to_string(),
                    count: 3,
                },
                CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 1,
                },
            ],
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("loot"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "loot give Steve loot chests/simple_dungeon"
        ),
        Err(CommandError::PermissionDenied)
    );

    let give = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot give Steve loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(give.success_count, 2);
    assert_eq!(give.feedback_key, "commands.drop.success.multiple");
    assert_eq!(state.player_inventories[0].items.len(), 2);

    let insert = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot insert 1 64 2 loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(insert.success_count, 2);
    assert!(state
        .block_item_slots
        .iter()
        .any(|entry| entry.pos == BlockPos { x: 1, y: 64, z: 2 }
            && entry.slot == "container.0"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:iron_ingot".to_string(),
                    count: 3,
                })));

    let spawn = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot spawn 4 65 6 loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(spawn.success_count, 2);
    assert!(matches!(
        state.loot_events.last().unwrap().target,
        CommandLootTarget::Spawn { .. }
    ));
}

#[test]
fn loot_command_replaces_entity_and_block_slots_from_mine_and_kill_sources() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 64, z: 0 },
            block: "minecraft:diamond_ore".to_string(),
        }],
        entity_loot_tables: vec![CommandEntityLootTable {
            entity: EntityRef {
                id: "Zombie".to_string(),
                display_name: "Zombie".to_string(),
            },
            table: "minecraft:entities/zombie".to_string(),
            drops: vec![CommandItemStack {
                item: "minecraft:rotten_flesh".to_string(),
                count: 2,
            }],
        }],
        ..ServerCommandState::default()
    };

    let replace_entity = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot replace entity Steve hotbar.0 2 mine 0 64 0 diamond_pickaxe",
    )
    .unwrap();
    assert_eq!(replace_entity.success_count, 1);
    assert_eq!(
        state.entity_item_slots[0].item,
        Some(CommandItemStack {
            item: "minecraft:diamond_ore".to_string(),
            count: 1,
        })
    );
    assert_eq!(state.entity_item_slots[1].slot, "hotbar.1");
    assert_eq!(state.entity_item_slots[1].item, None);

    let replace_block = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot replace block 2 64 2 container.0 kill Zombie",
    )
    .unwrap();
    assert_eq!(replace_block.success_count, 1);
    assert!(state
        .block_item_slots
        .iter()
        .any(|entry| entry.pos == BlockPos { x: 2, y: 64, z: 2 }
            && entry.slot == "container.0"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:rotten_flesh".to_string(),
                    count: 2,
                })));
    assert_eq!(
        state.loot_events.last().unwrap().source,
        CommandLootSource::Kill {
            entity: EntityRef {
                id: "Zombie".to_string(),
                display_name: "Zombie".to_string(),
            },
            table: "minecraft:entities/zombie".to_string(),
        }
    );
}

#[test]
fn loot_command_reports_missing_held_items_blocks_and_entity_tables() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot give Steve fish gameplay/fishing 0 64 0 mainhand"
        ),
        Err(CommandError::LootNoHeldItems)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot give Steve mine 0 64 0"
        ),
        Err(CommandError::LootNoBlockLootTable)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot give Steve kill Zombie"
        ),
        Err(CommandError::LootNoEntityLootTable)
    );
}

#[test]
fn loot_command_uses_fishing_source_with_optional_tool() {
    let mut state = ServerCommandState {
        command_loot_tables: vec![CommandLootTable {
            id: "minecraft:gameplay/fishing".to_string(),
            drops: vec![CommandItemStack {
                item: "minecraft:cod".to_string(),
                count: 1,
            }],
        }],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot give Steve fish gameplay/fishing 1 62 3 fishing_rod",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.drop.success.single");
    assert_eq!(
        state.player_inventories[0].items,
        vec![CommandItemStack {
            item: "minecraft:cod".to_string(),
            count: 1,
        }]
    );
    assert_eq!(
        state.loot_events.last().unwrap().source,
        CommandLootSource::Fish {
            table: "minecraft:gameplay/fishing".to_string(),
            pos: BlockPos { x: 1, y: 62, z: 3 },
            tool: Some("minecraft:fishing_rod".to_string()),
        }
    );
}

#[test]
fn place_command_records_feature_jigsaw_structure_and_template_placements() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 10.8,
            y: 64.0,
            z: -3.2,
        },
        available_templates: vec![
            "minecraft:village/plains/houses/plains_small_house_1".to_string()
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("place"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "place feature oak"
        ),
        Err(CommandError::PermissionDenied)
    );

    let feature = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "place feature oak",
    )
    .unwrap();
    assert_eq!(feature.feedback_key, "commands.place.feature.success");
    assert_eq!(state.place_events[0].kind, PlaceKind::Feature);
    assert_eq!(
        state.place_events[0].position,
        BlockPos {
            x: 10,
            y: 64,
            z: -4,
        }
    );

    let jigsaw = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "place jigsaw village/plains/town_centers minecraft:bottom 4 0 65 0",
    )
    .unwrap();
    assert_eq!(jigsaw.feedback_key, "commands.place.jigsaw.success");

    let structure = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "place structure stronghold 32 70 48",
    )
    .unwrap();
    assert_eq!(structure.feedback_key, "commands.place.structure.success");

    let template = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place template village/plains/houses/plains_small_house_1 1 64 2 clockwise_90 left_right 0.75 42 strict",
        )
        .unwrap();
    assert_eq!(template.feedback_key, "commands.place.template.success");
    assert_eq!(state.place_events.len(), 4);
    assert_eq!(state.place_events[3].kind, PlaceKind::Template);
    assert_eq!(
        state.place_events[3].id,
        "minecraft:village/plains/houses/plains_small_house_1"
    );
    assert_eq!(
        state.place_events[3].rotation.as_deref(),
        Some("clockwise_90")
    );
    assert_eq!(state.place_events[3].mirror.as_deref(), Some("left_right"));
    assert_eq!(state.place_events[3].integrity, Some(0.75));
    assert_eq!(state.place_events[3].seed, Some(42));
    assert!(state.place_events[3].strict);
}

#[test]
fn place_command_reports_vanilla_failure_paths() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place feature not_a_feature"
        ),
        Err(CommandError::PlaceFeatureFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place jigsaw village/plains/town_centers minecraft:bottom 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place structure not_a_structure"
        ),
        Err(CommandError::PlaceStructureFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place template missing_template"
        ),
        Err(CommandError::PlaceTemplateInvalid)
    );
    state
        .available_templates
        .push("minecraft:house".to_string());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place template house 0 64 0 bad_rotation"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "place template house 0 64 0 none none 1.5"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn raid_command_starts_checks_updates_and_stops_raids() {
    let player = NameAndId::create_offline("Steve");
    let mut state = ServerCommandState {
        command_source_player: Some(player),
        command_source_position: Vec3 {
            x: 10.0,
            y: 64.0,
            z: 10.0,
        },
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("raid"), PermissionLevel::Admins);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "raid start 2"
        ),
        Err(CommandError::PermissionDenied)
    );

    let start = execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid start 2")
        .unwrap();
    assert_eq!(start.success_count, 1);
    assert_eq!(start.feedback_key, "commands.raid.start.success");
    assert_eq!(
        state.raids,
        vec![CommandRaidState {
            center: BlockPos {
                x: 10,
                y: 64,
                z: 10
            },
            omen_level: 2,
            groups_spawned: 0,
            raiders_alive: 0,
            health: 0,
            total_health: 0,
            stopped: false,
            glowing: false,
        }]
    );

    let duplicate =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid start 1")
            .unwrap();
    assert_eq!(duplicate.success_count, -1);
    assert_eq!(duplicate.feedback_key, "commands.raid.already_started");

    let check =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid check").unwrap();
    assert_eq!(check.success_count, 1);
    assert_eq!(check.feedback_key, "commands.raid.check.success");

    let setomen =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid setomen 4")
            .unwrap();
    assert_eq!(setomen.feedback_key, "commands.raid.omen.changed");
    assert_eq!(state.raids[0].omen_level, 4);

    let glow =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid glow").unwrap();
    assert_eq!(glow.success_count, 1);
    assert!(state.raids[0].glowing);

    let stop =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid stop").unwrap();
    assert_eq!(stop.success_count, 1);
    assert!(state.raids[0].stopped);
    let none =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid check").unwrap();
    assert_eq!(none.success_count, 0);
}

#[test]
fn raid_command_records_sound_and_spawnleader_debug_actions() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Alex")),
        command_source_position: Vec3 {
            x: 1.0,
            y: 65.0,
            z: 2.0,
        },
        ..ServerCommandState::default()
    };

    let sound = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "raid sound local",
    )
    .unwrap();
    assert_eq!(sound.success_count, 1);
    assert_eq!(
        state.raid_events[0],
        CommandRaidEvent::Sound {
            local: true,
            position: Vec3 {
                x: 6.0,
                y: 65.0,
                z: 2.0,
            },
        }
    );

    let leader = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "raid spawnleader",
    )
    .unwrap();
    assert_eq!(leader.feedback_key, "commands.raid.spawnleader.success");
    assert_eq!(
        state.raid_events[1],
        CommandRaidEvent::SpawnLeader {
            position: Vec3 {
                x: 1.0,
                y: 65.0,
                z: 2.0,
            },
        }
    );
}

#[test]
fn raid_command_rejects_missing_player_and_bad_omen_levels() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid start 1"),
        Err(CommandError::InvalidSyntax)
    );
    state.command_source_player = Some(NameAndId::create_offline("Steve"));
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid start -1"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "raid setomen -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid start 1").unwrap();
    let too_high =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "raid setomen 6")
            .unwrap();
    assert_eq!(too_high.feedback_key, "commands.raid.omen.too_high");
    assert_eq!(state.raids[0].omen_level, 1);
}

#[test]
fn teleport_command_moves_targets_to_locations_and_entities() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "Steve".to_string(),
            display_name: "Steve".to_string(),
        }),
        command_source_position: Vec3 {
            x: 10.0,
            y: 64.0,
            z: 10.0,
        },
        entity_positions: vec![EntityPosition {
            entity: EntityRef {
                id: "Alex".to_string(),
                display_name: "Alex".to_string(),
            },
            dimension: "minecraft:the_nether".to_string(),
            position: Vec3 {
                x: 1.0,
                y: 70.0,
                z: 2.0,
            },
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("teleport"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "teleport Steve 1 2 3"
        ),
        Err(CommandError::PermissionDenied)
    );

    let self_tp = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "teleport ~1 65 ~-2",
    )
    .unwrap();
    assert_eq!(self_tp.success_count, 1);
    assert_eq!(
        self_tp.feedback_key,
        "commands.teleport.success.location.single"
    );
    assert_eq!(
        entity_position(&state, &entity_ref("Steve"))
            .unwrap()
            .position,
        Vec3 {
            x: 11.0,
            y: 65.0,
            z: 8.0,
        }
    );

    let to_entity = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tp Steve Alex",
    )
    .unwrap();
    assert_eq!(
        to_entity.feedback_key,
        "commands.teleport.success.entity.single"
    );
    let steve = entity_position(&state, &entity_ref("Steve")).unwrap();
    assert_eq!(steve.dimension, "minecraft:the_nether");
    assert_eq!(
        steve.position,
        Vec3 {
            x: 1.0,
            y: 70.0,
            z: 2.0,
        }
    );
}

#[test]
fn teleport_command_records_rotation_and_facing_requests() {
    let mut state = ServerCommandState::default();
    let rotated = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "teleport Steve,Alex 0 64 0 90 ~-30",
    )
    .unwrap();
    assert_eq!(rotated.success_count, 2);
    assert_eq!(
        rotated.feedback_key,
        "commands.teleport.success.location.multiple"
    );
    assert_eq!(state.entity_positions.len(), 2);
    assert_eq!(state.rotation_requests.len(), 2);
    assert_eq!(
        state.rotation_requests[0].mode,
        RotationMode::Angles {
            yaw: 90.0,
            pitch: -30.0,
            yaw_relative: false,
            pitch_relative: true,
        }
    );

    let facing = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "teleport Steve 1 65 2 facing entity Alex eyes",
    )
    .unwrap();
    assert_eq!(facing.success_count, 1);
    assert_eq!(
        state.rotation_requests.last().unwrap().mode,
        RotationMode::FacingEntity {
            entity: entity_ref("Alex"),
            anchor: EntityAnchor::Eyes,
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "teleport Steve 1 65 2 facing 4 65 6",
    )
    .unwrap();
    assert_eq!(
        state.rotation_requests.last().unwrap().mode,
        RotationMode::FacingPosition(Vec3 {
            x: 4.0,
            y: 65.0,
            z: 6.0,
        })
    );
}
