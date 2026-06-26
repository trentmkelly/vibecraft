use super::*;

#[test]
fn teleport_command_rejects_missing_source_and_invalid_positions() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "teleport 1 2 3"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "teleport Steve 30000000 64 0"
        ),
        Err(CommandError::TeleportInvalidPosition)
    );
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "teleport Steve -30000000 19999999.999 29999999.999",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "teleport Steve 0 20000000 0"
        ),
        Err(CommandError::TeleportInvalidPosition)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "teleport Steve -30000000.1 64 0"
        ),
        Err(CommandError::TeleportInvalidPosition)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "teleport Steve 0 64 0 facing entity Alex head"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn time_command_sets_adds_and_queries_default_clock() {
    let mut state = ServerCommandState {
        game_time_ticks: 2_147_483_650,
        world_clock_ticks: 23_000,
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("time"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "time set day"
        ),
        Err(CommandError::PermissionDenied)
    );

    let set_day = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time set day",
    )
    .unwrap();
    assert_eq!(state.world_clock_ticks, 1_000);
    assert_eq!(set_day.success_count, 1_000);
    assert_eq!(set_day.feedback_key, "commands.time.set.time_marker");
    assert!(set_day.broadcast_to_admins);

    let add = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time add 0.5d",
    )
    .unwrap();
    assert_eq!(state.world_clock_ticks, 13_000);
    assert_eq!(add.feedback_key, "commands.time.set.absolute");
    assert_eq!(add.success_count, 13_000);

    let query_time = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time query time",
    )
    .unwrap();
    assert_eq!(query_time.success_count, 13_000);
    assert_eq!(query_time.feedback_key, "commands.time.query.absolute");
    assert!(!query_time.broadcast_to_admins);

    let query_gametime = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time query gametime",
    )
    .unwrap();
    assert_eq!(query_gametime.success_count, 3);
    assert_eq!(query_gametime.feedback_key, "commands.time.query.gametime");
}

#[test]
fn time_command_tracks_pause_rate_clock_and_timeline_forms() {
    let mut state = ServerCommandState {
        world_clock_ticks: 50_000,
        ..ServerCommandState::default()
    };
    let pause = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time pause",
    )
    .unwrap();
    assert!(state.world_clock_paused);
    assert_eq!(pause.feedback_key, "commands.time.pause");

    let resume = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time resume",
    )
    .unwrap();
    assert!(!state.world_clock_paused);
    assert_eq!(resume.feedback_key, "commands.time.resume");

    let rate = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time rate 2.5",
    )
    .unwrap();
    assert_eq!(state.world_clock_rate, 2.5);
    assert_eq!(rate.feedback_key, "commands.time.rate");

    let day = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time query day",
    )
    .unwrap();
    assert_eq!(day.success_count, 2_000);
    assert_eq!(day.feedback_key, "commands.time.query.timeline");

    let repetitions = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time query moon repetition",
    )
    .unwrap();
    assert_eq!(repetitions.success_count, 0);
    assert_eq!(
        repetitions.feedback_key,
        "commands.time.query.timeline.repetitions"
    );

    let end_query = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time query of minecraft:the_end query time",
    )
    .unwrap();
    assert_eq!(end_query.success_count, 50_000);
    assert_eq!(end_query.feedback_key, "commands.time.query.absolute");
}

#[test]
fn time_command_rejects_invalid_or_clockless_forms() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "time set dawn"
        ),
        Err(CommandError::TimeNoTimeMarkerFound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "time set -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "time rate 1000.1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    state.command_source_dimension = "minecraft:the_nether".to_string();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "time query time"
        ),
        Err(CommandError::TimeNoDefaultClock)
    );
}

#[test]
fn title_command_records_text_clear_reset_and_times_packets() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("title"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "title Steve clear"
        ),
        Err(CommandError::PermissionDenied)
    );

    let title = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "title Steve title {\"text\":\"Boss Incoming\"}",
    )
    .unwrap();
    assert_eq!(title.success_count, 1);
    assert_eq!(title.feedback_key, "commands.title.show.title.single");
    assert!(title.broadcast_to_admins);
    assert_eq!(state.title_events.len(), 1);
    assert_eq!(
        state.title_events[0].targets,
        vec![NameAndId::create_offline("Steve")]
    );
    assert_eq!(
        state.title_events[0].action,
        TitleCommandAction::Text {
            kind: TitleTextKind::Title,
            component: "{\"text\":\"Boss Incoming\"}".to_string(),
        }
    );

    let actionbar = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "title Steve,Alex actionbar Ready",
    )
    .unwrap();
    assert_eq!(
        actionbar.feedback_key,
        "commands.title.show.actionbar.multiple"
    );
    assert_eq!(actionbar.success_count, 2);
    assert_eq!(
        state.title_events.last().unwrap().action,
        TitleCommandAction::Text {
            kind: TitleTextKind::ActionBar,
            component: "Ready".to_string(),
        }
    );

    let times = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "title Steve times 1s 2s 3s",
    )
    .unwrap();
    assert_eq!(times.feedback_key, "commands.title.times.single");
    assert_eq!(
        state.title_events.last().unwrap().action,
        TitleCommandAction::Times {
            fade_in: 20,
            stay: 40,
            fade_out: 60,
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "title Alex reset",
    )
    .unwrap();
    assert_eq!(
        state.title_events.last().unwrap().action,
        TitleCommandAction::Clear { reset: true }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "title Alex clear",
    )
    .unwrap();
    assert_eq!(
        state.title_events.last().unwrap().action,
        TitleCommandAction::Clear { reset: false }
    );
}

#[test]
fn title_command_rejects_missing_components_and_bad_times() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "title Steve title"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "title Steve times 1s 2s"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "title Steve times -1 2s 3s"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn clone_command_copies_masked_filtered_move_and_dimension_variants() {
    let mut state = ServerCommandState {
        blocks: vec![
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 0, y: 64, z: 0 },
                block: "minecraft:stone".to_string(),
            },
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 1, y: 64, z: 0 },
                block: "minecraft:air".to_string(),
            },
            BlockStateEntry {
                dimension: "minecraft:overworld".to_string(),
                position: BlockPos { x: 2, y: 64, z: 0 },
                block: "minecraft:dirt".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };

    let cloned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clone 0 64 0 2 64 0 10 70 0 masked force",
    )
    .unwrap();
    assert_eq!(cloned.success_count, 2);
    assert_eq!(cloned.feedback_key, "commands.clone.success");
    assert_eq!(
        state
            .blocks
            .iter()
            .find(|entry| entry.position == BlockPos { x: 10, y: 70, z: 0 })
            .unwrap()
            .block,
        "minecraft:stone"
    );
    assert_eq!(state.clone_events[0].filter, CloneFilter::Masked);
    assert_eq!(state.clone_events[0].mode, CloneMode::Force);
    assert_eq!(state.clone_events[0].default_update_flags, 2);
    assert_eq!(state.clone_events[0].move_barrier_update_flags, None);
    assert!(state.clone_events[0].neighbour_updates);
    assert!(state.clone_events[0].block_ticks_copied);

    let filtered = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clone 0 64 0 2 64 0 to the_nether 0 80 0 filtered dirt",
    )
    .unwrap();
    assert_eq!(filtered.success_count, 1);
    assert_eq!(
        state
            .blocks
            .iter()
            .find(|entry| {
                entry.dimension == "minecraft:the_nether"
                    && entry.position == BlockPos { x: 2, y: 80, z: 0 }
            })
            .unwrap()
            .block,
        "minecraft:dirt"
    );

    let moved = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clone 0 64 0 0 64 0 20 70 0 replace move",
    )
    .unwrap();
    assert_eq!(moved.success_count, 1);
    assert_eq!(
        state
            .blocks
            .iter()
            .find(|entry| entry.position == BlockPos { x: 0, y: 64, z: 0 })
            .unwrap()
            .block,
        "minecraft:air"
    );
    assert_eq!(state.clone_events.last().unwrap().mode, CloneMode::Move);
    assert_eq!(
        state.clone_events.last().unwrap().move_barrier_update_flags,
        Some(818)
    );
    assert_eq!(
        state.clone_events.last().unwrap().move_air_update_flags,
        Some(3)
    );
}

#[test]
fn clone_command_records_strict_update_flags_like_java() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 64, z: 0 },
            block: "minecraft:stone".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let cloned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clone 0 64 0 0 64 0 strict 10 70 0 replace move",
    )
    .unwrap();

    let event = state.clone_events.last().unwrap();
    assert_eq!(cloned.success_count, 1);
    assert!(event.strict);
    assert_eq!(event.default_update_flags, 818);
    assert_eq!(event.move_barrier_update_flags, Some(818));
    assert_eq!(event.move_air_update_flags, Some(818));
    assert!(!event.neighbour_updates);
    assert!(event.block_ticks_copied);
}

#[test]
fn clone_command_rejects_overlap_too_big_debug_and_empty_selection() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:overworld".to_string(),
            position: BlockPos { x: 0, y: 64, z: 0 },
            block: "minecraft:stone".to_string(),
        }],
        max_block_modifications: 4,
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clone 0 64 0 0 64 0 0 64 0"
        ),
        Err(CommandError::CloneOverlap)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clone 0 64 0 4 64 0 10 64 0"
        ),
        Err(CommandError::CloneTooBig)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clone 0 64 0 0 64 0 10 64 0 filtered diamond"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clone 0 64 0 0 64 0 10 64 0 filtered diamond_block"
        ),
        Err(CommandError::CloneFailed)
    );
    state.debug_world = true;
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clone 0 64 0 0 64 0 10 64 0 force"
        ),
        Err(CommandError::CloneFailed)
    );
}

#[test]
fn damage_command_records_generic_typed_positioned_and_entity_sources() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "damage zombie 4"
        ),
        Err(CommandError::PermissionDenied)
    );

    let generic = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "damage zombie 4",
    )
    .unwrap();
    assert_eq!(generic.success_count, 1);
    assert_eq!(generic.feedback_key, "commands.damage.success");
    assert_eq!(state.damage_events[0].source, DamageCommandSource::Generic);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "damage zombie 1.5 wither",
    )
    .unwrap();
    assert_eq!(
        state.damage_events[1].source,
        DamageCommandSource::Type {
            damage_type: "minecraft:wither".to_string(),
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "damage zombie 2 magic at 1.5 65 -2",
    )
    .unwrap();
    assert_eq!(
        state.damage_events[2].source,
        DamageCommandSource::At {
            damage_type: "minecraft:magic".to_string(),
            location: Vec3 {
                x: 1.5,
                y: 65.0,
                z: -2.0,
            },
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "damage zombie 3 arrow by arrow_entity from skeleton",
    )
    .unwrap();
    assert_eq!(
        state.damage_events[3].source,
        DamageCommandSource::By {
            damage_type: "minecraft:arrow".to_string(),
            entity: EntityRef {
                id: "arrow_entity".to_string(),
                display_name: "arrow_entity".to_string(),
            },
            cause: Some(EntityRef {
                id: "skeleton".to_string(),
                display_name: "skeleton".to_string(),
            }),
        }
    );
}

#[test]
fn damage_command_rejects_negative_amount_bad_syntax_and_invulnerable_targets() {
    let mut state = ServerCommandState {
        invulnerable_entities: vec![EntityRef {
            id: "armor_stand".to_string(),
            display_name: "Armor Stand".to_string(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "damage zombie -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "damage zombie 1 magic at 1 2"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "damage zombie 1 definitely_not_damage"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "damage armor_stand 1 generic"
        ),
        Err(CommandError::DamageInvulnerable)
    );
    assert!(state.damage_events.is_empty());
}

#[test]
fn datapack_command_lists_enables_disables_and_reloads_selection() {
    let mut state = ServerCommandState {
        available_data_packs: vec![
            "vanilla".to_string(),
            "file/low".to_string(),
            "file/high".to_string(),
            "file/extra".to_string(),
        ],
        selected_data_packs: vec!["vanilla".to_string(), "file/low".to_string()],
        disabled_data_packs: vec!["file/high".to_string()],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("datapack"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "datapack list"
        ),
        Err(CommandError::PermissionDenied)
    );

    assert_datapack_list_reports_enabled_then_available(&mut state);

    let available = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "datapack list available",
    )
    .unwrap();
    assert_eq!(available.success_count, 2);
    assert_eq!(
        available.feedback_key,
        "commands.datapack.list.available.success"
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "datapack enable file/high before file/low",
    )
    .unwrap();
    assert_eq!(
        state.selected_data_packs,
        vec!["vanilla", "file/high", "file/low"]
    );
    assert!(state.disabled_data_packs.is_empty());
    assert_eq!(
        state.reload_requests.last().unwrap(),
        &ReloadRequest {
            selected_packs: vec![
                "vanilla".to_string(),
                "file/high".to_string(),
                "file/low".to_string(),
            ],
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "datapack enable file/extra last",
    )
    .unwrap();
    assert_eq!(
        state.selected_data_packs,
        vec!["vanilla", "file/high", "file/low", "file/extra"]
    );

    let disabled = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "datapack disable file/high",
    )
    .unwrap();
    assert_eq!(disabled.success_count, 3);
    assert_eq!(disabled.feedback_key, "commands.datapack.modify.disable");
    assert_eq!(
        state.selected_data_packs,
        vec!["vanilla", "file/low", "file/extra"]
    );
    assert_eq!(state.disabled_data_packs, vec!["file/high"]);
}

#[test]
fn datapack_command_reports_vanilla_failures_and_creates_empty_packs() {
    let mut state = ServerCommandState {
        available_data_packs: vec![
            "vanilla".to_string(),
            "feature/redstone".to_string(),
            "file/locked".to_string(),
            "file/disabled".to_string(),
        ],
        selected_data_packs: vec![
            "vanilla".to_string(),
            "feature/redstone".to_string(),
            "file/locked".to_string(),
        ],
        feature_data_packs: vec!["feature/redstone".to_string()],
        unavailable_feature_data_packs: vec!["file/locked".to_string()],
        ..ServerCommandState::default()
    };

    assert_datapack_vanilla_failure_edges(&mut state);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "datapack create test_pack Empty test pack"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "datapack create bad/name Empty"
        ),
        Err(CommandError::DataPackInvalidName)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "datapack create CON Empty"
        ),
        Err(CommandError::DataPackInvalidFullName)
    );

    let created = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::OWNER,
        "datapack create test_pack Empty test pack",
    )
    .unwrap();
    assert_eq!(created.feedback_key, "commands.datapack.create.success");
    assert_eq!(
        state.created_data_packs,
        vec![super::CreatedDataPack {
            id: "test_pack".to_string(),
            description: "Empty test pack".to_string(),
        }]
    );
    assert!(state
        .available_data_packs
        .contains(&"file/test_pack".to_string()));
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "datapack create test_pack Empty"
        ),
        Err(CommandError::DataPackAlreadyExists)
    );
}

fn assert_datapack_list_reports_enabled_then_available(state: &mut ServerCommandState) {
    let listed = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "datapack list",
    )
    .unwrap();
    assert_eq!(listed.success_count, 4);
    assert_eq!(
        state.side_feedback,
        vec![
            CommandResult {
                success_count: 2,
                feedback_key: "commands.datapack.list.enabled.success",
                broadcast_to_admins: false,
            },
            CommandResult {
                success_count: 2,
                feedback_key: "commands.datapack.list.available.success",
                broadcast_to_admins: false,
            },
        ]
    );
    state.side_feedback.clear();
}

fn assert_datapack_vanilla_failure_edges(state: &mut ServerCommandState) {
    for (command, error) in [
        ("datapack enable missing", CommandError::DataPackUnknown),
        ("datapack enable vanilla", CommandError::DataPackAlreadyEnabled),
        (
            "datapack disable file/locked",
            CommandError::DataPackFeaturesNotEnabled,
        ),
        (
            "datapack disable file/disabled",
            CommandError::DataPackAlreadyDisabled,
        ),
        (
            "datapack disable feature/redstone",
            CommandError::DataPackCannotDisableFeature,
        ),
    ] {
        assert_eq!(
            execute_builtin_command(state, LevelBasedPermissionSet::GAMEMASTER, command),
            Err(error)
        );
    }
}

#[test]
fn kill_command_requires_gamemaster_and_defaults_to_source_entity() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "source".to_string(),
            display_name: "Source".to_string(),
        }),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("kill"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "kill"),
        Err(CommandError::PermissionDenied)
    );

    let killed =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "kill").unwrap();
    assert_eq!(killed.success_count, 1);
    assert_eq!(killed.feedback_key, "commands.kill.success.single");
    assert_eq!(state.killed_entities[0].id, "source");
}

#[test]
fn kill_command_accepts_multiple_targets() {
    let mut state = ServerCommandState::default();
    let killed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "kill zombie creeper",
    )
    .unwrap();
    assert_eq!(killed.success_count, 2);
    assert_eq!(killed.feedback_key, "commands.kill.success.multiple");
    assert_eq!(
        state
            .killed_entities
            .iter()
            .map(|entity| entity.id.as_str())
            .collect::<Vec<_>>(),
        vec!["zombie", "creeper"]
    );
}

#[test]
fn help_command_lists_visible_root_usages() {
    let mut state = ServerCommandState::default();
    assert_eq!(command_required_permission("help"), PermissionLevel::All);

    let result = execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help").unwrap();
    let all_usages = visible_command_usages(LevelBasedPermissionSet::ALL);
    assert_eq!(result.success_count, all_usages.len() as i32);
    assert!(all_usages.contains(&"/help [command]"));
    assert!(all_usages.contains(&"/list [uuids]"));
    assert!(!all_usages.contains(&"/stop"));
    assert_eq!(result.feedback_key, "commands.help.success");
    assert!(!result.broadcast_to_admins);
}

#[test]
fn help_command_reports_specific_visible_command_or_failure() {
    let mut state = ServerCommandState::default();
    let help =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help list").unwrap();
    assert_eq!(help.success_count, 1);
    assert_eq!(
        command_usage("list", LevelBasedPermissionSet::ALL),
        Some("/list [uuids]")
    );

    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "help stop"),
        Err(CommandError::HelpFailed)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "help stop"),
        Ok(super::CommandResult {
            success_count: 0,
            feedback_key: "commands.help.success",
            broadcast_to_admins: false,
        })
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "help missing"),
        Err(CommandError::HelpFailed)
    );
}

#[test]
fn chat_command_model_covers_public_private_team_raw_and_permission_feedback() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        online_players: vec![steve.clone(), alex.clone()],
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

    let say_denied = execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "say hello");
    assert_eq!(say_denied, Err(CommandError::PermissionDenied));
    assert!(state.chat_events.is_empty());

    let say = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "say hello everyone",
    )
    .unwrap();
    assert_eq!(say.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Say);
    assert_eq!(
        state.chat_events[0].targets,
        vec![steve.clone(), alex.clone()]
    );

    let emote =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "me waves").unwrap();
    assert_eq!(emote.feedback_key, "commands.me.success");
    assert_eq!(state.chat_events[1].kind, ChatCommandKind::Emote);

    let private =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tell Alex hi").unwrap();
    assert_eq!(private.success_count, 1);
    assert_eq!(private.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[2].kind, ChatCommandKind::Private);
    assert_eq!(state.chat_events[2].targets, vec![alex.clone()]);

    let team =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "teammsg ready").unwrap();
    assert_eq!(team.success_count, 2);
    assert_eq!(team.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[3].kind, ChatCommandKind::Team);
    assert_eq!(
        state.chat_events[3].targets,
        vec![steve.clone(), alex.clone()]
    );

    let tellraw = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tellraw Alex {\"text\":\"raw\"}",
    )
    .unwrap();
    assert_eq!(tellraw.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[4].kind, ChatCommandKind::TellRaw);
    assert_eq!(state.chat_events[4].message, "{\"text\":\"raw\"}");
}

#[test]
fn command_visibility_surface_filters_by_permission_tier_in_stable_order() {
    let all = visible_command_usages(LevelBasedPermissionSet::ALL);
    let moderator = visible_command_usages(LevelBasedPermissionSet::MODERATOR);
    let gamemaster = visible_command_usages(LevelBasedPermissionSet::GAMEMASTER);
    let admin = visible_command_usages(LevelBasedPermissionSet::ADMIN);
    let owner = visible_command_usages(LevelBasedPermissionSet::OWNER);

    assert!(all.starts_with(&[
        "/biome",
        "/chase <follow|lead|stop> [host|bind_address] [port]",
        "/help [command]",
    ]));
    assert!(all.contains(&"/msg <targets> <message>"));
    assert!(!all.contains(&"/gamemode <gamemode> [target]"));
    assert!(!all.contains(&"/op <targets>"));
    assert!(!all.contains(&"/stop"));

    assert_eq!(moderator, all);
    assert!(gamemaster.starts_with(&[
        "/advancement <grant|revoke> <targets> <everything|only|from|until|through>",
        "/attribute <target> <attribute> get|base|get|set|reset|modifier",
    ]));
    assert!(gamemaster.contains(&"/gamemode <gamemode> [target]"));
    assert!(!gamemaster.contains(&"/op <targets>"));
    assert!(!gamemaster.contains(&"/stop"));

    assert!(admin.contains(&"/op <targets>"));
    assert!(admin.contains(&"/whitelist <on|off|list|add|remove|reload>"));
    assert!(!admin.contains(&"/stop"));

    assert!(owner.contains(&"/op <targets>"));
    assert!(owner.contains(&"/stop"));
    assert_eq!(owner.last(), Some(&"/deop <targets>"));
    assert_eq!(
        command_usage("gamemode", LevelBasedPermissionSet::ALL),
        None
    );
    assert_eq!(
        command_usage("gamemode", LevelBasedPermissionSet::GAMEMASTER),
        Some("/gamemode <gamemode> [target]")
    );
}
