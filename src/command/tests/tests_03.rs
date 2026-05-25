use super::*;

#[test]
fn forceload_command_reports_range_and_noop_failures() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "forceload add -16 -16",
    )
    .unwrap();
    assert_eq!(state.forced_chunks[0].chunk, ChunkPos { x: -1, z: -1 });
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "forceload add -16 -16"
        ),
        Err(CommandError::ForceLoadAlreadyAdded)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "forceload remove 32 32"
        ),
        Err(CommandError::ForceLoadNotForced)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "forceload query 32 32"
        ),
        Err(CommandError::ForceLoadNotForced)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "forceload add 0 0 4096 0"
        ),
        Err(CommandError::ForceLoadTooBig)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "forceload add -30000001 0"
        ),
        Err(CommandError::ForceLoadOutOfWorld)
    );
}

#[test]
fn serverpack_push_generates_java_name_uuid_or_uses_explicit_uuid() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("serverpack"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "serverpack push https://example.invalid/pack.zip"
        ),
        Err(CommandError::PermissionDenied)
    );

    let generated = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "serverpack push https://example.invalid/pack.zip",
    )
    .unwrap();
    assert_eq!(generated.success_count, 0);
    assert_eq!(generated.feedback_key, "commands.serverpack.push");
    assert_eq!(
        state.server_pack_events[0],
        ServerPackCommandEvent::Push(ServerPackPushRequest {
            id: "f2dfd86d-3bee-3650-9f80-4317b330dccc".to_string(),
            url: "https://example.invalid/pack.zip".to_string(),
            hash: String::new(),
            required: false,
            prompt: None,
        })
    );

    execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack push https://example.invalid/pack2.zip 00000000-0000-3000-8000-000000000001 abc123",
        )
        .unwrap();
    assert_eq!(
        state.server_pack_events[1],
        ServerPackCommandEvent::Push(ServerPackPushRequest {
            id: "00000000-0000-3000-8000-000000000001".to_string(),
            url: "https://example.invalid/pack2.zip".to_string(),
            hash: "abc123".to_string(),
            required: false,
            prompt: None,
        })
    );
}

#[test]
fn serverpack_pop_records_uuid_and_rejects_bad_syntax() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "serverpack pop AAAAAAAA-BBBB-3CCC-8DDD-EEEEEEEEEEEE",
    )
    .unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, "commands.serverpack.pop");
    assert_eq!(
        state.server_pack_events,
        vec![ServerPackCommandEvent::Pop {
            id: "aaaaaaaa-bbbb-3ccc-8ddd-eeeeeeeeeeee".to_string(),
        }]
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack pop not-a-uuid"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "serverpack"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn version_command_reports_26_1_2_metadata() {
    let mut state = ServerCommandState::default();
    let lines = VersionInfo::CURRENT_26_1_2.command_lines();
    assert!(lines.contains(&"commands.version.id 26.1.2".to_string()));
    assert!(lines.contains(&"commands.version.data 4790".to_string()));
    assert!(lines.contains(&"commands.version.protocol 775 0x307".to_string()));
    assert!(lines.contains(&"commands.version.pack.resource 84".to_string()));
    assert!(lines.contains(&"commands.version.pack.data 101.1".to_string()));
    assert!(lines.contains(&"commands.version.stable.yes".to_string()));

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "/version").unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.version.header");
    assert!(!result.broadcast_to_admins);
}

#[test]
fn whitelist_command_requires_admin_permission() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "whitelist on"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on")
            .unwrap();
    assert!(state.whitelist_enabled);
    assert_eq!(state.kick_unlisted_requests, 1);
    assert_eq!(result.feedback_key, "commands.whitelist.enabled");
}

#[test]
fn whitelist_on_off_match_vanilla_errors_and_feedback() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist off"),
        Err(CommandError::WhitelistAlreadyOff)
    );

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on").unwrap();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist on"),
        Err(CommandError::WhitelistAlreadyOn)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist off")
            .unwrap();
    assert!(!state.whitelist_enabled);
    assert_eq!(state.kick_unlisted_requests, 1);
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.whitelist.disabled");
}

#[test]
fn whitelist_add_remove_and_list_track_profiles() {
    let mut state = ServerCommandState::default();
    let empty =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist list")
            .unwrap();
    assert_eq!(empty.success_count, 0);
    assert_eq!(empty.feedback_key, "commands.whitelist.none");

    let added = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "whitelist add Steve Alex",
    )
    .unwrap();
    assert_eq!(added.success_count, 2);
    assert_eq!(added.feedback_key, "commands.whitelist.add.success");
    assert_eq!(state.whitelist_names(), vec!["Steve", "Alex"]);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "whitelist add Steve"
        ),
        Err(CommandError::AlreadyWhitelisted)
    );

    let list =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "whitelist list")
            .unwrap();
    assert_eq!(list.success_count, 2);
    assert_eq!(list.feedback_key, "commands.whitelist.list");
    assert!(!list.broadcast_to_admins);

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "whitelist remove Alex",
    )
    .unwrap();
    assert_eq!(removed.success_count, 1);
    assert_eq!(removed.feedback_key, "commands.whitelist.remove.success");
    assert_eq!(state.whitelist_names(), vec!["Steve"]);
    assert_eq!(state.kick_unlisted_requests, 1);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "whitelist remove Alex"
        ),
        Err(CommandError::NotWhitelisted)
    );
}

#[test]
fn whitelist_reload_requests_storage_reload_and_kick() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "whitelist reload",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.whitelist.reloaded");
    assert_eq!(state.whitelist_reload_requests, 1);
    assert_eq!(state.kick_unlisted_requests, 1);
}

#[test]
fn op_command_requires_admin_and_tracks_operator_profiles() {
    let mut state = ServerCommandState::default();
    assert_eq!(command_required_permission("op"), PermissionLevel::Admins);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "op Steve"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "op Steve Alex")
            .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.op.success");
    assert!(result.broadcast_to_admins);
    assert_eq!(state.operator_names(), vec!["Steve", "Alex"]);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "op Steve"),
        Err(CommandError::OpFailed)
    );
}

#[test]
fn deop_command_removes_ops_and_requests_unlisted_player_kick() {
    let mut state = ServerCommandState {
        operator_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("deop"), PermissionLevel::Admins);

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "deop Steve").unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.deop.success");
    assert_eq!(state.operator_names(), vec!["Alex"]);
    assert_eq!(state.kick_unlisted_requests, 1);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "deop Steve"),
        Err(CommandError::DeOpFailed)
    );
}

#[test]
fn operator_permission_surface_covers_visibility_feedback_and_reload_effects() {
    let mut state = ServerCommandState::default();
    assert!(!visible_command_usages(LevelBasedPermissionSet::ALL).contains(&"/op <targets>"));
    assert!(visible_command_usages(LevelBasedPermissionSet::ADMIN).contains(&"/op <targets>"));
    assert!(visible_command_usages(LevelBasedPermissionSet::ADMIN).contains(&"/deop <targets>"));
    assert_eq!(command_usage("op", LevelBasedPermissionSet::ALL), None);
    assert_eq!(
        command_usage("op", LevelBasedPermissionSet::ADMIN),
        Some("/op <targets>")
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "op Steve"),
        Err(CommandError::PermissionDenied)
    );

    let op_result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "op Steve").unwrap();
    assert_eq!(op_result.feedback_key, "commands.op.success");
    assert_eq!(state.operator_names(), vec!["Steve"]);

    let deop_result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "deop Steve").unwrap();
    assert_eq!(deop_result.feedback_key, "commands.deop.success");
    assert!(state.operator_names().is_empty());
    assert_eq!(state.kick_unlisted_requests, 1);
}

#[test]
fn operator_command_smoke_matrix_covers_permissions_feedback_and_state_changes() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        online_players: vec![steve.clone(), alex.clone()],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "op Steve"),
        Err(CommandError::PermissionDenied)
    );

    let op =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "op Steve").unwrap();
    assert_eq!(op.success_count, 1);
    assert_eq!(op.feedback_key, "commands.op.success");
    assert!(state.operator_names().contains(&"Steve"));

    let whitelist = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "whitelist add Alex",
    )
    .unwrap();
    assert_eq!(whitelist.feedback_key, "commands.whitelist.add.success");
    assert_eq!(state.whitelist_names(), vec!["Alex"]);

    let ban = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "ban Alex -- smoke",
    )
    .unwrap();
    assert_eq!(ban.feedback_key, "commands.ban.success");
    assert_eq!(state.banned_player_names(), vec!["Alex"]);
    assert_eq!(
        state.disconnected_players.last().unwrap().reason,
        "multiplayer.disconnect.banned"
    );

    let pardon =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "pardon Alex").unwrap();
    assert_eq!(pardon.feedback_key, "commands.pardon.success");
    assert!(state.banned_players.is_empty());

    let gamemode = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamemode creative",
    )
    .unwrap();
    assert_eq!(gamemode.feedback_key, "commands.gamemode.success.self");
    assert_eq!(
        state
            .player_game_modes
            .iter()
            .find(|entry| entry.player.uuid == steve.uuid)
            .map(|entry| entry.gamemode),
        Some(GameMode::Creative)
    );

    let teleport = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tp Alex 1 80 2",
    )
    .unwrap();
    assert_eq!(
        teleport.feedback_key,
        "commands.teleport.success.location.single"
    );
    assert_eq!(state.entity_positions.last().unwrap().entity.id, "Alex");

    let give = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve stone 2",
    )
    .unwrap();
    assert_eq!(give.feedback_key, "commands.give.success.single");
    assert_eq!(state.player_inventories[0].items[0].count, 2);

    let effect = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve speed",
    )
    .unwrap();
    assert_eq!(effect.feedback_key, "commands.effect.give.success.single");
    assert_eq!(state.active_effects[0].effect, "minecraft:speed");

    let deop =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "deop Steve").unwrap();
    assert_eq!(deop.feedback_key, "commands.deop.success");
    assert!(state.operator_players.is_empty());
}

#[test]
fn debug_command_starts_stops_and_records_function_traces() {
    let mut state = ServerCommandState {
        debug_profiler_results: vec![super::DebugProfilerResult {
            duration_nanos: 2_000_000_000,
            tick_duration: 40,
        }],
        macro_functions: vec!["minecraft:test".to_string(), "minecraft:test".to_string()],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("debug"),
        PermissionLevel::Admins
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "debug start"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "debug stop"),
        Err(CommandError::DebugNotRunning)
    );

    let started =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "debug start").unwrap();
    assert_eq!(started.success_count, 0);
    assert_eq!(started.feedback_key, "commands.debug.started");
    assert!(state.debug_profiler_running);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "debug start"),
        Err(CommandError::DebugAlreadyRunning)
    );

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "debug stop").unwrap();
    assert_eq!(stopped.success_count, 20);
    assert_eq!(stopped.feedback_key, "commands.debug.stopped");
    assert!(!state.debug_profiler_running);

    let traced = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "debug function minecraft:test",
    )
    .unwrap();
    assert_eq!(
        traced.feedback_key,
        "commands.debug.function.success.single"
    );
    assert_eq!(
        state.debug_trace_events,
        vec![super::DebugTraceEvent {
            function: "minecraft:test".to_string(),
            output: "debug-trace-1.txt".to_string(),
            command_count: 2,
        }]
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "debug function return"
        ),
        Err(CommandError::DebugNoReturnRun)
    );
}

#[test]
fn debugconfig_moves_players_through_configuration_and_dialogs() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("debugconfig"),
        PermissionLevel::Admins
    );

    let config = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "debugconfig config Steve",
    )
    .unwrap();
    assert_eq!(config.success_count, 1);
    assert_eq!(
        state.config_players,
        vec![NameAndId::create_offline("Steve")]
    );

    let dialog = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "debugconfig dialog Steve minecraft:test_dialog",
    )
    .unwrap();
    assert_eq!(dialog.success_count, 1);
    assert_eq!(
        state.config_dialog_events,
        vec![super::DebugConfigDialogEvent {
            target: "Steve".to_string(),
            dialog: "minecraft:test_dialog".to_string(),
        }]
    );

    let unconfig = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "debugconfig unconfig Steve",
    )
    .unwrap();
    assert_eq!(unconfig.success_count, 1);
    assert!(state.config_players.is_empty());
    let missing = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "debugconfig dialog Steve minecraft:test_dialog",
    )
    .unwrap();
    assert_eq!(missing.success_count, 0);
    assert_eq!(missing.feedback_key, "commands.debugconfig.missing");
}

#[test]
fn debugmobspawning_and_debugpath_record_debug_actions() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "zombie".to_string(),
            display_name: "Zombie".to_string(),
        }),
        unreachable_debug_paths: vec![BlockPos { x: 2, y: 64, z: 2 }],
        incomplete_debug_paths: vec![BlockPos { x: 3, y: 64, z: 3 }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("debugmobspawning"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "debugmobspawning monster 0 64 0"
        ),
        Err(CommandError::PermissionDenied)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "debugmobspawning monster 0 64 0",
    )
    .unwrap();
    assert_eq!(
        state.mob_spawning_events,
        vec![super::DebugMobSpawningEvent {
            category: "monster".to_string(),
            position: BlockPos { x: 0, y: 64, z: 0 },
        }]
    );

    let path = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "debugpath 1 64 1",
    )
    .unwrap();
    assert_eq!(path.success_count, 1);
    assert_eq!(path.feedback_key, "commands.debugpath.success");
    assert_eq!(
        state.debug_path_events[0].target,
        BlockPos { x: 1, y: 64, z: 1 }
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "debugpath 2 64 2"
        ),
        Err(CommandError::DebugPathNoPath)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "debugpath 3 64 3"
        ),
        Err(CommandError::DebugPathNotComplete)
    );

    state.entity_states.push(EntityState {
        entity: EntityRef {
            id: "zombie".to_string(),
            display_name: "Zombie".to_string(),
        },
        kind: EntityKind::NonLiving,
        dimension: "minecraft:overworld".to_string(),
    });
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "debugpath 4 64 4"
        ),
        Err(CommandError::DebugPathNotMob)
    );
}

#[test]
fn gamemode_commands_update_defaults_players_and_forced_modes() {
    let mut state = ServerCommandState {
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        force_game_mode: Some(GameMode::Adventure),
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("defaultgamemode"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "defaultgamemode creative"
        ),
        Err(CommandError::PermissionDenied)
    );

    let defaulted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "defaultgamemode creative",
    )
    .unwrap();
    assert_eq!(defaulted.success_count, 2);
    assert_eq!(defaulted.feedback_key, "commands.defaultgamemode.success");
    assert_eq!(state.default_game_mode, GameMode::Creative);
    assert_eq!(
        super::player_gamemode(&state, &NameAndId::create_offline("Steve")),
        GameMode::Adventure
    );

    let self_mode = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamemode spectator",
    )
    .unwrap();
    assert_eq!(self_mode.success_count, 1);
    assert_eq!(self_mode.feedback_key, "commands.gamemode.success.self");
    assert_eq!(
        super::player_gamemode(&state, &NameAndId::create_offline("Steve")),
        GameMode::Spectator
    );

    let others = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamemode creative Steve Alex",
    )
    .unwrap();
    assert_eq!(others.success_count, 2);
    assert_eq!(others.feedback_key, "commands.gamemode.success.other");
    assert_eq!(
        super::player_gamemode(&state, &NameAndId::create_offline("Alex")),
        GameMode::Creative
    );
}

#[test]
fn difficulty_and_gamerule_commands_query_set_and_reject_noops() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "difficulty"
        )
        .unwrap()
        .success_count,
        1
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "difficulty easy"
        ),
        Err(CommandError::DifficultyAlreadySame)
    );
    let difficulty = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "difficulty hard",
    )
    .unwrap();
    assert_eq!(difficulty.success_count, 0);
    assert_eq!(difficulty.feedback_key, "commands.difficulty.success");
    assert_eq!(state.difficulty, super::Difficulty::Hard);

    let query = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule doDaylightCycle",
    )
    .unwrap();
    assert_eq!(query.success_count, 1);
    assert_eq!(query.feedback_key, "commands.gamerule.query");

    let set_bool = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule doDaylightCycle false",
    )
    .unwrap();
    assert_eq!(set_bool.success_count, 0);
    assert_eq!(
        super::game_rule_value(&state, "minecraft:doDaylightCycle").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        state.game_rule_syncs[0],
        super::GameRuleSyncEvent {
            rule: "minecraft:advance_time".to_string(),
            value: "false".to_string(),
        }
    );

    let set_int = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule randomTickSpeed 12",
    )
    .unwrap();
    assert_eq!(set_int.success_count, 12);
    assert_eq!(
        super::game_rule_value(&state, "randomTickSpeed").unwrap(),
        super::GameRuleValue::Int(12)
    );
    assert_eq!(
        state.game_rule_syncs[1],
        super::GameRuleSyncEvent {
            rule: "minecraft:random_tick_speed".to_string(),
            value: "12".to_string(),
        }
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule randomTickSpeed true"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn gamerule_defaults_cover_26_1_2_registry_names_and_bounds() {
    let state = ServerCommandState::default();
    let names: Vec<&str> = state
        .game_rules
        .iter()
        .map(|rule| rule.name.as_str())
        .collect();
    assert_eq!(names.len(), super::VANILLA_GAME_RULES.len());
    assert_eq!(
        names,
        super::VANILLA_GAME_RULES
            .iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        super::game_rule_value(&state, "advance_weather").unwrap(),
        super::GameRuleValue::Bool(true)
    );
    assert_eq!(
        super::game_rule_value(&state, "keep_inventory").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        super::game_rule_value(&state, "max_block_modifications").unwrap(),
        super::GameRuleValue::Int(32768)
    );
    assert_eq!(
        super::game_rule_value(&state, "players_nether_portal_default_delay").unwrap(),
        super::GameRuleValue::Int(80)
    );
    assert_eq!(
        super::game_rule_value(&state, "random_tick_speed").unwrap(),
        super::GameRuleValue::Int(3)
    );
}

#[test]
fn gamerule_command_supports_canonical_names_aliases_and_integer_ranges() {
    let mut state = ServerCommandState::default();
    let canonical = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule keep_inventory true",
    )
    .unwrap();
    assert_eq!(canonical.success_count, 1);
    assert_eq!(
        super::game_rule_value(&state, "keepInventory").unwrap(),
        super::GameRuleValue::Bool(true)
    );

    let bounded = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamerule max_snow_accumulation_height 8",
    )
    .unwrap();
    assert_eq!(bounded.success_count, 8);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule max_snow_accumulation_height 9"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule random_tick_speed -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "gamerule fire_spread_radius_around_player -1"
        )
        .unwrap()
        .success_count,
        -1
    );
}

#[test]
fn gamerule_command_toggles_named_client_observable_rules() {
    let mut state = ServerCommandState::default();
    let cases = [
        ("keepInventory", "true", "minecraft:keep_inventory", 1),
        (
            "doImmediateRespawn",
            "true",
            "minecraft:immediate_respawn",
            1,
        ),
        (
            "sendCommandFeedback",
            "false",
            "minecraft:send_command_feedback",
            0,
        ),
        ("doDaylightCycle", "false", "minecraft:advance_time", 0),
        ("mobGriefing", "false", "minecraft:mob_griefing", 0),
    ];

    for (rule, value, sync_rule, success_count) in cases {
        let result = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            &format!("gamerule {rule} {value}"),
        )
        .unwrap();
        assert_eq!(result.success_count, success_count);
        assert_eq!(result.feedback_key, "commands.gamerule.set");
        assert!(result.broadcast_to_admins);
        assert_eq!(
            state.game_rule_syncs.last().unwrap(),
            &super::GameRuleSyncEvent {
                rule: sync_rule.to_string(),
                value: value.to_string(),
            }
        );
    }

    assert_eq!(
        super::game_rule_value(&state, "keep_inventory").unwrap(),
        super::GameRuleValue::Bool(true)
    );
    assert_eq!(
        super::game_rule_value(&state, "immediate_respawn").unwrap(),
        super::GameRuleValue::Bool(true)
    );
    assert_eq!(
        super::game_rule_value(&state, "send_command_feedback").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        super::game_rule_value(&state, "advance_time").unwrap(),
        super::GameRuleValue::Bool(false)
    );
    assert_eq!(
        super::game_rule_value(&state, "mob_griefing").unwrap(),
        super::GameRuleValue::Bool(false)
    );
}
