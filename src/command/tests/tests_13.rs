use super::*;

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
        block_item_slots: vec![
            CommandBlockItemSlot {
                pos: BlockPos { x: 1, y: 64, z: 2 },
                slot: "container.0".to_string(),
                item: None,
            },
            CommandBlockItemSlot {
                pos: BlockPos { x: 1, y: 64, z: 2 },
                slot: "container.1".to_string(),
                item: None,
            },
        ],
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

    let give_multiple_players = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot give Steve,Alex loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(give_multiple_players.success_count, 4);
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
        block_item_slots: vec![CommandBlockItemSlot {
            pos: BlockPos { x: 2, y: 64, z: 2 },
            slot: "container.0".to_string(),
            item: None,
        }],
        ..ServerCommandState::default()
    };

    let replace_entity = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot replace entity Steve hotbar.0 2 mine 0 64 0 diamond_pickaxe",
    )
    .unwrap();
    assert_eq!(replace_entity.success_count, 2);
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
fn loot_command_inserts_by_java_container_distribution_rules() {
    let mut state = ServerCommandState {
        command_loot_tables: vec![CommandLootTable {
            id: "minecraft:chests/simple_dungeon".to_string(),
            drops: vec![
                CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 3,
                },
                CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 2,
                },
            ],
        }],
        block_item_slots: vec![
            CommandBlockItemSlot {
                pos: BlockPos { x: 4, y: 64, z: 4 },
                slot: "container.0".to_string(),
                item: Some(CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 62,
                }),
            },
            CommandBlockItemSlot {
                pos: BlockPos { x: 4, y: 64, z: 4 },
                slot: "container.1".to_string(),
                item: None,
            },
            CommandBlockItemSlot {
                pos: BlockPos { x: 4, y: 64, z: 4 },
                slot: "container.2".to_string(),
                item: None,
            },
        ],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot insert 4 64 4 loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert!(state.block_item_slots.iter().any(|entry| {
        entry.slot == "container.0"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 64,
                })
    }));
    assert!(state.block_item_slots.iter().any(|entry| {
        entry.slot == "container.1"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 1,
                })
    }));
    assert!(state.block_item_slots.iter().any(|entry| {
        entry.slot == "container.2"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 2,
                })
    }));
}

#[test]
fn loot_command_reports_container_failures_and_counts_empty_replacements() {
    let mut state = ServerCommandState {
        command_loot_tables: vec![CommandLootTable {
            id: "minecraft:chests/simple_dungeon".to_string(),
            drops: vec![CommandItemStack {
                item: "minecraft:iron_ingot".to_string(),
                count: 1,
            }],
        }],
        block_item_slots: vec![
            CommandBlockItemSlot {
                pos: BlockPos { x: 6, y: 64, z: 6 },
                slot: "container.0".to_string(),
                item: None,
            },
            CommandBlockItemSlot {
                pos: BlockPos { x: 6, y: 64, z: 6 },
                slot: "container.1".to_string(),
                item: Some(CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 1,
                }),
            },
        ],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot insert 9 64 9 loot chests/simple_dungeon"
        ),
        Err(CommandError::ItemTargetNotContainer)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot replace block 9 64 9 container.0 loot chests/simple_dungeon"
        ),
        Err(CommandError::ItemTargetNotContainer)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "loot replace block 6 64 6 container.2 loot chests/simple_dungeon"
        ),
        Err(CommandError::ItemTargetNoSuchSlot)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot replace block 6 64 6 container.0 2 loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert!(state
        .block_item_slots
        .iter()
        .any(|entry| { entry.slot == "container.1" && entry.item.is_none() }));

    let entity_result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "loot replace entity Steve hotbar.0 2 loot chests/simple_dungeon",
    )
    .unwrap();
    assert_eq!(entity_result.success_count, 2);
    assert!(state.entity_item_slots.iter().any(|entry| {
        entry.entity.id == "Steve" && entry.slot == "hotbar.1" && entry.item.is_none()
    }));
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
