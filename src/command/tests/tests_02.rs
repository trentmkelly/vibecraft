use super::*;

#[test]
fn scoreboard_objectives_scores_and_display_slots_round_trip_persistence() {
    let mut state = ServerCommandState {
        scoreboard_objectives: vec![ScoreboardObjective {
            name: "kills".to_string(),
            criteria: "dummy".to_string(),
            display_name: "Kills".to_string(),
            render_type: "hearts".to_string(),
            display_auto_update: false,
            number_format: Some("fixed:!".to_string()),
        }],
        scoreboard_scores: vec![ScoreboardScore {
            owner: "Steve".to_string(),
            objective: "kills".to_string(),
            value: 7,
            locked: false,
            display_name: Some("Slayer".to_string()),
            number_format: Some("blank".to_string()),
        }],
        scoreboard_display_slots: vec![ScoreboardDisplaySlot {
            slot: "sidebar".to_string(),
            objective: "kills".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let persisted = ScoreboardPersistence::from_state(&state);
    let tag = persisted.to_nbt();
    assert!(matches!(
        &tag,
        Tag::Compound(fields)
            if fields.iter().any(|(name, _)| name == "Objectives")
                && fields.iter().any(|(name, _)| name == "PlayerScores")
                && fields.iter().any(|(name, _)| name == "DisplaySlots")
    ));

    let loaded = ScoreboardPersistence::from_nbt(&tag).unwrap();
    assert_eq!(loaded.objectives, state.scoreboard_objectives);
    assert_eq!(loaded.scores, state.scoreboard_scores);
    assert_eq!(loaded.display_slots, state.scoreboard_display_slots);

    state.scoreboard_objectives.clear();
    state.scoreboard_scores.clear();
    state.scoreboard_display_slots.clear();
    loaded.apply_to_state(&mut state);
    assert_eq!(state.scoreboard_objectives[0].criteria, "dummy");
    assert_eq!(state.scoreboard_objectives[0].render_type, "hearts");
    assert!(!state.scoreboard_objectives[0].display_auto_update);
    assert_eq!(
        state.scoreboard_objectives[0].number_format,
        Some("fixed:!".to_string())
    );
    assert_eq!(state.scoreboard_scores[0].value, 7);
    assert!(!state.scoreboard_scores[0].locked);
    assert_eq!(state.scoreboard_display_slots[0].slot, "sidebar");
}

#[test]
fn trigger_command_consumes_enabled_trigger_scores() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        scoreboard_objectives: vec![ScoreboardObjective {
            name: "quest".to_string(),
            criteria: "trigger".to_string(),
            display_name: "Quest".to_string(),
            render_type: "integer".to_string(),
            display_auto_update: true,
            number_format: None,
        }],
        scoreboard_scores: vec![ScoreboardScore {
            owner: "Steve".to_string(),
            objective: "quest".to_string(),
            value: 0,
            locked: false,
            display_name: None,
            number_format: None,
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("trigger"), PermissionLevel::All);

    let simple =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "trigger quest").unwrap();
    assert_eq!(simple.success_count, 1);
    assert_eq!(simple.feedback_key, "commands.trigger.simple.success");
    assert!(simple.broadcast_to_admins);
    assert_eq!(state.scoreboard_scores[0].value, 1);
    assert!(state.scoreboard_scores[0].locked);

    state.scoreboard_scores[0].locked = false;
    let add = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "trigger quest add 4",
    )
    .unwrap();
    assert_eq!(add.success_count, 5);
    assert_eq!(add.feedback_key, "commands.trigger.add.success");
    assert_eq!(state.scoreboard_scores[0].value, 5);
    assert!(state.scoreboard_scores[0].locked);

    state.scoreboard_scores[0].locked = false;
    let set = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "trigger quest set -3",
    )
    .unwrap();
    assert_eq!(set.success_count, -3);
    assert_eq!(set.feedback_key, "commands.trigger.set.success");
    assert_eq!(state.scoreboard_scores[0].value, -3);
    assert!(state.scoreboard_scores[0].locked);
}

#[test]
fn trigger_command_addition_wraps_like_java_int_scores() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        scoreboard_objectives: vec![ScoreboardObjective {
            name: "quest".to_string(),
            criteria: "trigger".to_string(),
            display_name: "Quest".to_string(),
            render_type: "integer".to_string(),
            display_auto_update: true,
            number_format: None,
        }],
        scoreboard_scores: vec![ScoreboardScore {
            owner: "Steve".to_string(),
            objective: "quest".to_string(),
            value: i32::MAX,
            locked: false,
            display_name: None,
            number_format: None,
        }],
        ..ServerCommandState::default()
    };

    let simple =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "trigger quest").unwrap();
    assert_eq!(simple.success_count, i32::MIN);
    assert_eq!(state.scoreboard_scores[0].value, i32::MIN);
    assert!(state.scoreboard_scores[0].locked);

    state.scoreboard_scores[0].locked = false;
    let add = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "trigger quest add -1",
    )
    .unwrap();
    assert_eq!(add.success_count, i32::MAX);
    assert_eq!(state.scoreboard_scores[0].value, i32::MAX);
}

#[test]
fn trigger_command_rejects_unprimed_non_trigger_and_non_player_sources() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        scoreboard_objectives: vec![
            ScoreboardObjective {
                name: "quest".to_string(),
                criteria: "trigger".to_string(),
                display_name: "Quest".to_string(),
                render_type: "integer".to_string(),
                display_auto_update: true,
                number_format: None,
            },
            ScoreboardObjective {
                name: "kills".to_string(),
                criteria: "dummy".to_string(),
                display_name: "Kills".to_string(),
                render_type: "integer".to_string(),
                display_auto_update: true,
                number_format: None,
            },
        ],
        scoreboard_scores: vec![ScoreboardScore {
            owner: "Steve".to_string(),
            objective: "quest".to_string(),
            value: 0,
            locked: true,
            display_name: None,
            number_format: None,
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "trigger quest"),
        Err(CommandError::TriggerNotPrimed)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "trigger kills"),
        Err(CommandError::ScoreboardNotTrigger)
    );
    state.command_source_player = None;
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "trigger quest"),
        Err(CommandError::NoPlayers)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "trigger quest add"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn spawn_armor_trims_spawns_single_pattern_grid_from_player_position() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        command_source_position: Vec3 {
            x: 10.2,
            y: 64.9,
            z: -4.1,
        },
        command_source_yaw: 90.0,
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("spawn_armor_trims"),
        PermissionLevel::Gamemasters
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawn_armor_trims sentry",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.spawn_armor_trims.success");
    assert!(result.broadcast_to_admins);
    assert_eq!(state.armor_trim_spawns.len(), 11 * 25);
    assert_eq!(state.armor_trim_spawns[0].pattern, "minecraft:sentry");
    assert_eq!(state.armor_trim_spawns[0].material, "minecraft:quartz");
    assert_eq!(state.armor_trim_spawns[0].item, "minecraft:leather_helmet");
    assert_eq!(
        state.armor_trim_spawns[0].position,
        Vec3 {
            x: 5.5,
            y: 64.5,
            z: -4.5,
        }
    );
    assert_eq!(state.armor_trim_spawns[0].y_rot, 180.0);
    assert!(state.armor_trim_spawns[0].no_gravity);
    assert!(state.armor_trim_spawns[0].named);
    assert!(state.armor_trim_spawns[0].custom_name_visible);
    assert!(!state.armor_trim_spawns[0].invisible);
    assert!(!state.armor_trim_spawns[1].named);
    assert!(!state.armor_trim_spawns[1].custom_name_visible);
    assert!(state.armor_trim_spawns[1].invisible);
}

#[test]
fn spawn_armor_trims_spawns_all_patterns_and_rejects_invalid_sources_or_patterns() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spawn_armor_trims *_lag_my_game",
    )
    .unwrap();
    assert_eq!(state.armor_trim_spawns.len(), 18 * 11 * 25);
    assert_eq!(
        state.armor_trim_spawns.last().unwrap().pattern,
        "minecraft:bolt"
    );

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spawn_armor_trims nope"
        ),
        Err(CommandError::InvalidArmorTrimPattern)
    );
    state.command_source_player = None;
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spawn_armor_trims sentry"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn spawn_armor_trims_command_source_matches_java_26_1_2() {
    const SPAWN_ARMOR_TRIMS_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SpawnArmorTrimsCommand.java");

    for sentinel in [
        "Commands.literal(\"spawn_armor_trims\")",
        "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
        "Commands.literal(\"*_lag_my_game\")",
        "ResourceKeyArgument.key(Registries.TRIM_PATTERN)",
        "ResourceKeyArgument.getRegistryKey(c, \"pattern\", Registries.TRIM_PATTERN, ERROR_INVALID_PATTERN)",
        "((CommandSourceStack)c.getSource()).getPlayerOrException()",
        "patterns.sorted(Comparator.comparing(h -> TRIM_PATTERN_ORDER.applyAsInt(h.key()))).toList()",
        "sorted(Comparator.comparing(h -> TRIM_MATERIAL_ORDER.applyAsInt(h.key())))",
        "findEquippableItemsWithAssets(level.registryAccess().lookupOrThrow(Registries.ITEM))",
        "BlockPos origin = player.blockPosition().relative(player.getDirection(), 5);",
        "armorStand.setYRot(180.0F);",
        "armorStand.setNoGravity(true);",
        "stack.set(DataComponents.TRIM, trim);",
        "armorStand.setItemSlot(equippable.slot(), stack);",
        "armorStand.setCustomNameVisible(true);",
        "armorStand.setInvisible(true);",
        "source.sendSuccess(() -> Component.literal(\"Armorstands with trimmed armor spawned around you\"), true);",
        "return 1;",
    ] {
        assert!(
            SPAWN_ARMOR_TRIMS_COMMAND_JAVA.contains(sentinel),
            "SpawnArmorTrimsCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn spreadplayers_places_entities_or_team_groups() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_dimension: "minecraft:the_nether".to_string(),
        command_source_position: Vec3 {
            x: 100.25,
            y: 70.0,
            z: -25.75,
        },
        world_seed: 7,
        player_teams: vec![
            TeamMembership {
                player: steve.clone(),
                team: "red".to_string(),
            },
            TeamMembership {
                player: alex.clone(),
                team: "red".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("spreadplayers"),
        PermissionLevel::Gamemasters
    );

    let separate = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spreadplayers 0 0 2 10 false Steve Alex",
    )
    .unwrap();
    assert_eq!(separate.success_count, 2);
    assert_eq!(
        separate.feedback_key,
        "commands.spreadplayers.success.entities"
    );
    assert_eq!(state.entity_positions.len(), 2);
    let steve_pos = state
        .entity_positions
        .iter()
        .find(|entry| entry.entity.id == "Steve")
        .unwrap()
        .position;
    let alex_pos = state
        .entity_positions
        .iter()
        .find(|entry| entry.entity.id == "Alex")
        .unwrap()
        .position;
    assert!(
        ((steve_pos.x - alex_pos.x).powi(2) + (steve_pos.z - alex_pos.z).powi(2)).sqrt()
            >= 2.0
    );
    assert_eq!(steve_pos.y, 1.0);
    assert_eq!(alex_pos.y, 1.0);
    assert_eq!(
        state.entity_positions[0].dimension,
        "minecraft:the_nether".to_string()
    );

    let teams = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spreadplayers ~ ~ 2 10 under 80 true Steve Alex",
    )
    .unwrap();
    assert_eq!(teams.success_count, 1);
    assert_eq!(teams.feedback_key, "commands.spreadplayers.success.teams");
    assert_eq!(
        state
            .entity_positions
            .iter()
            .find(|entry| entry.entity.id == "Steve")
            .unwrap()
            .position,
        state
            .entity_positions
            .iter()
            .find(|entry| entry.entity.id == "Alex")
            .unwrap()
            .position
    );
    assert_eq!(
        state
            .entity_positions
            .iter()
            .find(|entry| entry.entity.id == "Steve")
            .unwrap()
            .position
            .y,
        1.0
    );
}

#[test]
fn spreadplayers_rejects_invalid_height_impossible_spacing_and_syntax() {
    let mut state = ServerCommandState::default();

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spreadplayers 0 0 1 10 under -65 false Steve"
        ),
        Err(CommandError::SpreadPlayersInvalidMaxHeight)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spreadplayers 0 0 5 1 false Steve Alex"
        ),
        Err(CommandError::SpreadPlayersFailedEntities)
    );
    state.player_teams = vec![
        TeamMembership {
            player: NameAndId::create_offline("Steve"),
            team: "red".to_string(),
        },
        TeamMembership {
            player: NameAndId::create_offline("Alex"),
            team: "blue".to_string(),
        },
    ];
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spreadplayers 0 0 5 1 true Steve Alex"
        ),
        Err(CommandError::SpreadPlayersFailedTeams)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spreadplayers 0 0 1 0 false Steve"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn spreadplayers_uses_java_team_identity_for_players_only() {
    let mut state = ServerCommandState {
        world_seed: 11,
        online_players: vec![NameAndId::create_offline("Steve")],
        player_teams: vec![TeamMembership {
            player: NameAndId::create_offline("Steve"),
            team: "red".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spreadplayers 0 0 1 10 true Steve Zombie Skeleton",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    let zombie = state
        .entity_positions
        .iter()
        .find(|entry| entry.entity.id == "Zombie")
        .unwrap()
        .position;
    let skeleton = state
        .entity_positions
        .iter()
        .find(|entry| entry.entity.id == "Skeleton")
        .unwrap()
        .position;
    assert_eq!(zombie, skeleton);
    assert_ne!(
        state
            .entity_positions
            .iter()
            .find(|entry| entry.entity.id == "Steve")
            .unwrap()
            .position,
        zombie
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn spreadplayers_command_source_matches_java_26_1_2() {
    const SPREADPLAYERS_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SpreadPlayersCommand.java");

    for sentinel in [
        "Commands.literal(\"spreadplayers\")",
        "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
        "Commands.argument(\"center\", Vec2Argument.vec2())",
        "FloatArgumentType.floatArg(0.0F)",
        "FloatArgumentType.floatArg(1.0F)",
        "Commands.literal(\"under\")",
        "Commands.argument(\"maxHeight\", IntegerArgumentType.integer())",
        "Commands.argument(\"respectTeams\", BoolArgumentType.bool())",
        "Commands.argument(\"targets\", EntityArgument.entities())",
        "((CommandSourceStack)c.getSource()).getLevel().getMaxY() + 1",
        "throw ERROR_INVALID_MAX_HEIGHT.create(maxHeight, minY);",
        "RandomSource random = RandomSource.createThreadLocalInstance();",
        "createInitialPositions(",
        "spreadPositions(center, spreadDistance, level, random, minX, minZ, maxX, maxZ, maxHeight, positions, respectTeams);",
        "setPlayerPositions(entities, level, positions, maxHeight, respectTeams);",
        "commands.spreadplayers.success.",
        "return positions.length;",
        "private static int getNumberOfTeams",
        "if (player instanceof Player) {",
        "teams.add(player.getTeam());",
        "for (iteration = 0; iteration < 10000 && hasCollisions; iteration++)",
        "if (dist < spreadDist) {",
        "position.moveAway(averageNeighbourPos);",
        "position.randomize(random, minX, minZ, maxX, maxZ);",
        "if (!position.isSafe(level, maxHeight))",
        "throw ERROR_FAILED_TO_SPREAD_TEAMS.create",
        "throw ERROR_FAILED_TO_SPREAD_ENTITIES.create",
        "Mth.floor(position.x) + 0.5",
        "position.getSpawnY(level, maxHeight)",
        "pos.getY() < maxHeight && !state.liquid() && !state.is(BlockTags.FIRE)",
    ] {
        assert!(
            SPREADPLAYERS_COMMAND_JAVA.contains(sentinel),
            "SpreadPlayersCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn setblock_command_places_replaces_and_destroys_blocks() {
    let mut state = ServerCommandState {
        command_source_dimension: "minecraft:the_end".to_string(),
        command_source_position: Vec3 {
            x: 10.5,
            y: 64.0,
            z: -2.5,
        },
        command_source_yaw: 0.0,
        command_source_pitch: 0.0,
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("setblock"),
        PermissionLevel::Gamemasters
    );

    let placed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setblock 1 64 2 stone",
    )
    .unwrap();
    assert_eq!(placed.success_count, 1);
    assert_eq!(placed.feedback_key, "commands.setblock.success");
    assert!(placed.broadcast_to_admins);
    assert_eq!(
        state.blocks[0],
        BlockStateEntry {
            dimension: "minecraft:the_end".to_string(),
            position: BlockPos { x: 1, y: 64, z: 2 },
            block: "minecraft:stone".to_string(),
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setblock 1 64 2 minecraft:dirt destroy",
    )
    .unwrap();
    assert_eq!(state.blocks[0].block, "minecraft:dirt");
    assert_eq!(state.setblock_events[1].mode, SetBlockMode::Destroy);
    assert_eq!(
        state.setblock_events[1].destroyed_block,
        Some("minecraft:stone".to_string())
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setblock ~1 ~ ~-2 oak_leaves[persistent=true] strict",
    )
    .unwrap();
    assert!(state.setblock_events[2].strict);
    assert_eq!(
        state.setblock_events[2].position,
        BlockPos { x: 11, y: 64, z: -5 }
    );
    assert_eq!(
        state.setblock_events[2].block,
        "minecraft:oak_leaves[persistent=true]"
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "setblock ^1 ^ ^2 gold_block",
    )
    .unwrap();
    assert_eq!(
        state.setblock_events[3].position,
        BlockPos { x: 11, y: 64, z: -1 }
    );
}

#[test]
fn setblock_command_rejects_keep_debug_and_bad_syntax() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 64, z: 0 },
            block: "minecraft:stone".to_string(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 0 64 0 dirt keep"
        ),
        Err(CommandError::SetBlockFailed)
    );
    state.debug_world = true;
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 1 64 0 dirt"
        ),
        Err(CommandError::SetBlockFailed)
    );
    state.debug_world = false;
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 1 64 dirt"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 1 64 0 BadBlock"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 2 64 0 air"
        ),
        Err(CommandError::SetBlockFailed)
    );
    state.blocks.push(BlockStateEntry {
        dimension: "minecraft:overworld".to_string(),
        position: BlockPos { x: 3, y: 64, z: 0 },
        block: "minecraft:stone".to_string(),
    });
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 3 64 0 stone"
        ),
        Err(CommandError::SetBlockFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setblock 4 64 0 stone[axis=x]"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn setblock_command_source_matches_java_26_1_2() {
    const SET_BLOCK_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SetBlockCommand.java");

    for sentinel in [
        "Commands.literal(\"setblock\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"pos\", BlockPosArgument.blockPos())",
        "BlockPosArgument.getLoadedBlockPos(c, \"pos\")",
        "Commands.argument(\n                                       \"block\", BlockStateArgument.block(context)",
        "Commands.literal(\"destroy\")",
        "Commands.literal(\"keep\")",
        "Commands.literal(\"replace\")",
        "Commands.literal(\"strict\")",
        "if (level.isDebug())",
        "predicate != null && !predicate.test(new BlockInWorld(level, pos, true))",
        "level.destroyBlock(pos, true)",
        "placeNeeded = !block.getState().isAir() || !level.getBlockState(pos).isAir();",
        "block.place(level, pos, 2 | (strict ? 816 : 256))",
        "if (!strict)",
        "level.updateNeighboursOnBlockSet(pos, oldState)",
        "Component.translatable(\"commands.setblock.success\", pos.getX(), pos.getY(), pos.getZ())",
        "return 1;",
    ] {
        assert!(
            SET_BLOCK_COMMAND_JAVA.contains(sentinel),
            "SetBlockCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn fill_command_replaces_outlines_hollows_and_keeps_blocks() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 1, y: 1, z: 1 },
            block: "minecraft:stone".to_string(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("fill"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "fill 0 0 0 0 0 0 stone"
        ),
        Err(CommandError::PermissionDenied)
    );

    let filled = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 1 1 1 dirt",
    )
    .unwrap();
    assert_eq!(filled.success_count, 8);
    assert_eq!(filled.feedback_key, "commands.fill.success");
    assert_eq!(state.fill_events[0].mode, FillMode::Replace);
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 1, y: 1, z: 1 } && entry.block == "minecraft:dirt"
    }));

    let hollow = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 2 2 2 glass hollow",
    )
    .unwrap();
    assert_eq!(hollow.success_count, 27);
    assert_eq!(state.fill_events.last().unwrap().mode, FillMode::Hollow);
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 1, y: 1, z: 1 } && entry.block == "minecraft:air"
    }));

    let kept = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 1 1 1 1 1 1 gold_block keep",
    )
    .unwrap();
    assert_eq!(kept.success_count, 1);
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 1, y: 1, z: 1 } && entry.block == "minecraft:gold_block"
    }));
}

#[test]
fn fill_command_filters_destroys_strict_and_reports_failures() {
    let mut state = ServerCommandState {
        blocks: vec![
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 0, y: 0, z: 0 },
                block: "minecraft:stone".to_string(),
            },
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 1, y: 0, z: 0 },
                block: "minecraft:dirt".to_string(),
            },
        ],
        max_block_modifications: 2,
        ..ServerCommandState::default()
    };

    let filtered = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 1 0 0 diamond_block replace stone",
    )
    .unwrap();
    assert_eq!(filtered.success_count, 1);
    assert_eq!(
        state.fill_events.last().unwrap().filter,
        Some("minecraft:stone".to_string())
    );
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 0, y: 0, z: 0 } && entry.block == "minecraft:diamond_block"
    }));

    let destroyed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 1 0 0 1 0 0 air destroy",
    )
    .unwrap();
    assert_eq!(destroyed.success_count, 1);
    assert_eq!(state.fill_events.last().unwrap().mode, FillMode::Destroy);

    let strict = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 0 0 0 emerald_block strict",
    )
    .unwrap();
    assert_eq!(strict.success_count, 1);
    assert!(state.fill_events.last().unwrap().strict);

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 2 0 0 stone"
        ),
        Err(CommandError::FillTooBig)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 0 0 0 emerald_block replace stone"
        ),
        Err(CommandError::FillFailed)
    );
}

#[test]
fn fill_command_validates_block_states_and_tag_predicates_like_java() {
    let mut state = ServerCommandState {
        blocks: vec![
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 0, y: 0, z: 0 },
                block: "minecraft:oak_log[axis=x]".to_string(),
            },
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 1, y: 0, z: 0 },
                block: "minecraft:oak_log[axis=z]".to_string(),
            },
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 2, y: 0, z: 0 },
                block: "minecraft:stone".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };

    let changed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 2 0 0 stripped_oak_log[axis=x] replace #minecraft:logs[axis=x]",
    )
    .unwrap();
    assert_eq!(changed.success_count, 1);
    let event = state.fill_events.last().unwrap();
    assert_eq!(event.block, "minecraft:stripped_oak_log[axis=x]");
    assert_eq!(event.filter, Some("#minecraft:logs[axis=x]".to_string()));
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 0, y: 0, z: 0 }
            && entry.block == "minecraft:stripped_oak_log[axis=x]"
    }));
    assert!(state.blocks.iter().any(|entry| {
        entry.position == BlockPos { x: 1, y: 0, z: 0 }
            && entry.block == "minecraft:oak_log[axis=z]"
    }));

    let changed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fill 0 0 0 1 0 0 birch_log replace oak_log",
    )
    .unwrap();
    assert_eq!(changed.success_count, 1);
    assert_eq!(
        state.fill_events.last().unwrap().filter,
        Some("minecraft:oak_log".to_string())
    );

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 0 0 0 not_a_block"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 0 0 0 stone[axis=x]"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 0 0 0 stone replace #minecraft:not_a_real_block_tag"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fill 0 0 0 0 0 0 stone replace outline"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn fillbiome_command_quantizes_replaces_and_filters_biomes() {
    let mut state = ServerCommandState {
        biomes: vec![
            BiomeEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 0, y: 0, z: 0 },
                biome: "minecraft:plains".to_string(),
            },
            BiomeEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 4, y: 0, z: 0 },
                biome: "minecraft:forest".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("fillbiome"),
        PermissionLevel::Gamemasters
    );

    let changed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fillbiome 1 0 0 7 0 0 desert replace plains",
    )
    .unwrap();
    assert_eq!(changed.success_count, 1);
    assert_eq!(changed.feedback_key, "commands.fillbiome.success.count");
    assert_eq!(
        state.fill_biome_events[0].begin,
        BlockPos { x: 0, y: 0, z: 0 }
    );
    assert_eq!(
        state.fill_biome_events[0].end,
        BlockPos { x: 4, y: 0, z: 0 }
    );
    assert_eq!(
        state.fill_biome_events[0].filter,
        Some("minecraft:plains".to_string())
    );
    assert!(state.biomes.iter().any(|entry| {
        entry.position == BlockPos { x: 0, y: 0, z: 0 } && entry.biome == "minecraft:desert"
    }));
    assert!(state.biomes.iter().any(|entry| {
        entry.position == BlockPos { x: 4, y: 0, z: 0 } && entry.biome == "minecraft:forest"
    }));

    let changed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fillbiome 0 0 0 4 0 0 swamp replace #minecraft:is_forest",
    )
    .unwrap();
    assert_eq!(changed.success_count, 1);
    assert_eq!(
        state.fill_biome_events[1].filter,
        Some("#minecraft:is_forest".to_string())
    );
    assert!(state.biomes.iter().any(|entry| {
        entry.position == BlockPos { x: 4, y: 0, z: 0 } && entry.biome == "minecraft:swamp"
    }));

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fillbiome 0 0 0 4 0 0 plains replace #minecraft:not_a_real_biome_tag"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn biome_debug_command_reports_quantized_source_biome() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 7.9,
            y: 64.0,
            z: -1.0,
        },
        biomes: vec![BiomeEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 4, y: 64, z: -4 },
            biome: "minecraft:forest".to_string(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(command_required_permission("biome"), PermissionLevel::All);
    assert_eq!(
        command_usage("biome", LevelBasedPermissionSet::ALL),
        Some("/biome")
    );
    assert_eq!(debug_biome_at_command_source(&state), "minecraft:forest");

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "biome").unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.vibecraft.debug.biome");
    assert!(!result.broadcast_to_admins);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "biome extra"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn fillbiome_command_reports_volume_and_syntax_failures() {
    let mut state = ServerCommandState {
        max_block_modifications: 1,
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fillbiome 0 0 0 4 0 0 desert"
        ),
        Err(CommandError::FillBiomeTooBig)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fillbiome 0 0 0 0 0 0 desert unless plains"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn forceload_command_adds_queries_lists_and_removes_chunks() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("forceload"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "forceload add 0 0"
        ),
        Err(CommandError::PermissionDenied)
    );

    let added = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload add 0 0 31 15",
    )
    .unwrap();
    assert_eq!(added.success_count, 2);
    assert_eq!(added.feedback_key, "commands.forceload.added.multiple");
    assert_eq!(
        state.forced_chunks,
        vec![
            ForcedChunk {
                dimension: "minecraft:overworld".to_string(),
                chunk: ChunkPos { x: 0, z: 0 },
            },
            ForcedChunk {
                dimension: "minecraft:overworld".to_string(),
                chunk: ChunkPos { x: 1, z: 0 },
            },
        ]
    );

    let listed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload query",
    )
    .unwrap();
    assert_eq!(listed.success_count, 2);
    assert_eq!(listed.feedback_key, "commands.forceload.list.multiple");

    let queried = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload query 16 0",
    )
    .unwrap();
    assert_eq!(queried.success_count, 1);
    assert_eq!(queried.feedback_key, "commands.forceload.query.success");

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload remove 0 0",
    )
    .unwrap();
    assert_eq!(removed.success_count, 1);
    assert_eq!(removed.feedback_key, "commands.forceload.removed.single");
    assert_eq!(state.forced_chunks.len(), 1);

    let all = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload remove all",
    )
    .unwrap();
    assert_eq!(all.success_count, 0);
    assert!(state.forced_chunks.is_empty());
}
