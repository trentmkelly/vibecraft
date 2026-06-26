use super::*;

#[test]
fn reload_command_requires_gamemaster_and_returns_zero() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("reload"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "reload"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "reload").unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, "commands.reload.success");
    assert!(result.broadcast_to_admins);
    assert_eq!(state.reload_requests.len(), 1);
}

#[test]
fn reload_command_discovers_new_packs_without_enabling_disabled_packs() {
    let mut state = ServerCommandState {
        available_data_packs: vec![
            "vanilla".to_string(),
            "kept".to_string(),
            "new_pack".to_string(),
            "disabled_pack".to_string(),
        ],
        selected_data_packs: vec!["vanilla".to_string(), "kept".to_string()],
        disabled_data_packs: vec!["disabled_pack".to_string()],
        ..ServerCommandState::default()
    };
    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "reload").unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(
        state.selected_data_packs,
        vec!["vanilla", "kept", "new_pack"]
    );
    assert_eq!(
        state.reload_requests,
        vec![ReloadRequest {
            selected_packs: vec![
                "vanilla".to_string(),
                "kept".to_string(),
                "new_pack".to_string(),
            ],
        }]
    );
}

#[test]
fn say_command_requires_gamemaster_and_broadcasts_to_online_players() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Console")),
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("say"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "say hello"),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "say hello all",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events.len(), 1);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Say);
    assert_eq!(state.chat_events[0].targets.len(), 2);
    assert_eq!(state.chat_events[0].message, "hello all");
}

#[test]
fn me_command_broadcasts_emote_chat_without_permission_gate() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("me"), PermissionLevel::All);
    assert_eq!(command_usage("me", LevelBasedPermissionSet::ALL), Some("/me <action>"));
    assert_eq!(command_usage("emote", LevelBasedPermissionSet::ALL), None);

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "me waves hello")
            .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.me.success");
    assert!(!result.broadcast_to_admins);
    assert_eq!(state.chat_events.len(), 1);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Emote);
    assert_eq!(
        state.chat_events[0].sender,
        Some(NameAndId::create_offline("Steve"))
    );
    assert_eq!(state.chat_events[0].targets.len(), 2);
    assert_eq!(state.chat_events[0].message, "waves hello");
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "me"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "emote waves"),
        Err(CommandError::PermissionDenied)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn me_command_source_matches_java_26_1_2() {
    const EMOTE_COMMANDS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/EmoteCommands.java");

    for sentinel in [
        "Commands.literal(\"me\")",
        "Commands.argument(\"action\", MessageArgument.message())",
        "MessageArgument.resolveChatMessage(c, \"action\", message ->",
        "source.getServer().getPlayerList()",
        "playerList.broadcastChatMessage(message, source, ChatType.bind(ChatType.EMOTE_COMMAND, source));",
        "return 1;",
    ] {
        assert!(
            EMOTE_COMMANDS_JAVA.contains(sentinel),
            "EmoteCommands.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn msg_tell_and_w_send_private_messages_without_permission_gate() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("msg"), PermissionLevel::All);

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "msg Alex secret plan",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Private);
    assert_eq!(state.chat_events[0].targets[0].name, "Alex");
    assert_eq!(state.chat_events[0].message, "secret plan");

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tell Steve hello").unwrap();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "w Alex hi").unwrap();
    assert_eq!(state.chat_events.len(), 3);

    let multi = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "msg Steve,Alex hello both",
    )
    .unwrap();
    assert_eq!(multi.success_count, 2);
    assert_eq!(state.chat_events[3].message, "hello both");
}

#[test]
fn teammsg_requires_source_team_and_targets_team_members() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        online_players: vec![steve.clone(), alex.clone()],
        player_teams: vec![
            TeamMembership {
                player: steve,
                team: "red".to_string(),
            },
            TeamMembership {
                player: alex,
                team: "red".to_string(),
            },
            TeamMembership {
                player: NameAndId::create_offline("Bob"),
                team: "blue".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("teammsg"), PermissionLevel::All);

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "teammsg push left",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Team);
    assert_eq!(state.chat_events[0].message, "push left");
    assert_eq!(
        state.chat_events[0]
            .targets
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Steve", "Alex"]
    );

    state.command_source_player = Some(NameAndId::create_offline("NoTeam"));
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tm hello"),
        Err(CommandError::TeamMsgNoTeam)
    );
}

#[test]
fn tellraw_requires_gamemaster_and_sends_system_message_to_targets() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("tellraw"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "tellraw Steve {\"text\":\"hi\"}"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tellraw Steve,Alex {\"text\":\"hi\"}",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::TellRaw);
    assert_eq!(state.chat_events[0].message, "{\"text\":\"hi\"}");
    assert_eq!(state.chat_events[0].targets.len(), 2);
}

#[test]
fn stopwatch_command_creates_queries_restarts_and_removes() {
    let mut state = ServerCommandState {
        command_time_millis: 1_000,
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("stopwatch"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "stopwatch create minecraft:test"
        ),
        Err(CommandError::PermissionDenied)
    );

    let created = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopwatch create minecraft:test",
    )
    .unwrap();
    assert_eq!(created.success_count, 1);
    assert_eq!(created.feedback_key, "commands.stopwatch.create.success");
    assert_eq!(
        state.stopwatches,
        vec![StopwatchState {
            id: "minecraft:test".to_string(),
            creation_time_millis: 1_000,
            accumulated_elapsed_millis: 0,
        }]
    );

    state.command_time_millis = 4_250;
    let queried = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopwatch query minecraft:test 10",
    )
    .unwrap();
    assert_eq!(queried.success_count, 32);
    assert_eq!(queried.feedback_key, "commands.stopwatch.query");

    let restarted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopwatch restart minecraft:test",
    )
    .unwrap();
    assert_eq!(restarted.success_count, 1);
    assert_eq!(state.stopwatches[0].creation_time_millis, 4_250);

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopwatch remove minecraft:test",
    )
    .unwrap();
    assert_eq!(removed.success_count, 1);
    assert!(state.stopwatches.is_empty());
}

#[test]
fn stopwatch_command_reports_duplicate_missing_and_bad_syntax() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopwatch create minecraft:test",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopwatch create minecraft:test"
        ),
        Err(CommandError::StopwatchAlreadyExists)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopwatch query minecraft:missing"
        ),
        Err(CommandError::StopwatchDoesNotExist)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopwatch restart minecraft:missing"
        ),
        Err(CommandError::StopwatchDoesNotExist)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopwatch remove minecraft:missing"
        ),
        Err(CommandError::StopwatchDoesNotExist)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "stopwatch create bad id"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn swing_command_defaults_to_source_entity_and_accepts_hands() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "Steve".to_string(),
            display_name: "Steve".to_string(),
        }),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("swing"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "swing"),
        Err(CommandError::PermissionDenied)
    );

    let own =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "swing").unwrap();
    assert_eq!(own.success_count, 1);
    assert_eq!(own.feedback_key, "commands.swing.success.single");
    assert_eq!(
        state.swing_events[0],
        SwingCommandEvent {
            target: EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            },
            hand: InteractionHand::MainHand,
        }
    );

    let offhand = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "swing pig,cow offhand",
    )
    .unwrap();
    assert_eq!(offhand.success_count, 2);
    assert_eq!(offhand.feedback_key, "commands.swing.success.multiple");
    assert_eq!(state.swing_events[1].hand, InteractionHand::OffHand);
    assert_eq!(state.swing_events[2].target.id, "cow");
}

#[test]
fn swing_command_fails_when_no_living_entity_swings() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "minecart".to_string(),
            display_name: "minecart".to_string(),
        }),
        entity_states: vec![EntityState {
            entity: EntityRef {
                id: "minecart".to_string(),
                display_name: "minecart".to_string(),
            },
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "swing minecart"
        ),
        Err(CommandError::SwingNoLivingEntity)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "swing"),
        Err(CommandError::SwingNoLivingEntity)
    );
    state.command_source_entity = None;
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "swing"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "swing pig wronghand"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn tag_command_adds_removes_and_lists_entity_tags() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("tag"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "tag pig add angry"
        ),
        Err(CommandError::PermissionDenied)
    );

    let added = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig,cow add angry",
    )
    .unwrap();
    assert_eq!(added.success_count, 2);
    assert_eq!(added.feedback_key, "commands.tag.add.success.multiple");
    assert_eq!(
        state.entity_tags,
        vec![
            EntityTags {
                entity: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                tags: vec!["angry".to_string()],
            },
            EntityTags {
                entity: EntityRef {
                    id: "cow".to_string(),
                    display_name: "cow".to_string(),
                },
                tags: vec!["angry".to_string()],
            },
        ]
    );

    let listed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig,cow list",
    )
    .unwrap();
    assert_eq!(listed.success_count, 1);
    assert_eq!(listed.feedback_key, "commands.tag.list.multiple.success");
    assert!(!listed.broadcast_to_admins);

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig remove angry",
    )
    .unwrap();
    assert_eq!(removed.success_count, 1);
    assert_eq!(removed.feedback_key, "commands.tag.remove.success.single");
    assert!(state.entity_tags[0].tags.is_empty());
}

#[test]
fn tag_command_uses_java_entity_tag_capacity_limit_per_target() {
    let mut state = ServerCommandState {
        entity_tags: vec![EntityTags {
            entity: EntityRef {
                id: "pig".to_string(),
                display_name: "pig".to_string(),
            },
            tags: (0..1024).map(|index| format!("tag{index}")).collect(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "tag pig add overflow"
        ),
        Err(CommandError::TagAddFailed)
    );

    let mixed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig,cow add overflow",
    )
    .unwrap();
    assert_eq!(mixed.success_count, 1);
    assert_eq!(mixed.feedback_key, "commands.tag.add.success.multiple");
    assert_eq!(state.entity_tags[0].tags.len(), 1024);
    assert_eq!(
        state
            .entity_tags
            .iter()
            .find(|entry| entry.entity.id == "cow")
            .map(|entry| entry.tags.as_slice()),
        Some(["overflow".to_string()].as_slice())
    );
}

#[test]
fn tag_command_reports_empty_lists_and_failed_mutations() {
    let mut state = ServerCommandState::default();
    let empty = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig list",
    )
    .unwrap();
    assert_eq!(empty.success_count, 0);
    assert_eq!(empty.feedback_key, "commands.tag.list.single.empty");

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "tag pig add angry",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "tag pig add angry"
        ),
        Err(CommandError::TagAddFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "tag cow remove angry"
        ),
        Err(CommandError::TagRemoveFailed)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "tag pig"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn tag_command_source_matches_java_26_1_2() {
    const TAG_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/TagCommand.java");
    const ENTITY_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/entity/Entity.java");

    for sentinel in [
        "Commands.literal(\"tag\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.argument(\"targets\", EntityArgument.entities())",
        "Commands.literal(\"add\")",
        "Commands.literal(\"remove\")",
        "SharedSuggestionProvider.suggest(getTags(EntityArgument.getEntities(c, \"targets\")), p)",
        "Commands.literal(\"list\")",
        "throw ERROR_ADD_FAILED.create();",
        "throw ERROR_REMOVE_FAILED.create();",
        "commands.tag.add.success.single",
        "commands.tag.add.success.multiple",
        "commands.tag.remove.success.single",
        "commands.tag.remove.success.multiple",
        "commands.tag.list.single.empty",
        "commands.tag.list.single.success",
        "commands.tag.list.multiple.empty",
        "commands.tag.list.multiple.success",
    ] {
        assert!(
            TAG_COMMAND_JAVA.contains(sentinel),
            "TagCommand.java is missing sentinel: {sentinel}"
        );
    }

    for sentinel in [
        "public Set<String> entityTags()",
        "return this.tags.size() >= 1024 ? false : this.tags.add(tag);",
        "return this.tags.remove(tag);",
    ] {
        assert!(
            ENTITY_JAVA.contains(sentinel),
            "Entity.java is missing tag sentinel: {sentinel}"
        );
    }
}

#[test]
fn jfr_command_requires_owner_and_records_start_stop_path() {
    let mut state = ServerCommandState {
        next_jfr_recording_path: "debug/test-recording.jfr".to_string(),
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("jfr"), PermissionLevel::Owners);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "jfr start"),
        Err(CommandError::PermissionDenied)
    );

    let started =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr start").unwrap();
    assert!(state.jfr_recording);
    assert_eq!(started.success_count, 1);
    assert_eq!(started.feedback_key, "commands.jfr.started");
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr start"),
        Err(CommandError::JfrStartFailed)
    );

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr stop").unwrap();
    assert!(!state.jfr_recording);
    assert_eq!(stopped.success_count, 1);
    assert_eq!(stopped.feedback_key, "commands.jfr.stopped");
    assert_eq!(state.jfr_recordings, vec!["debug/test-recording.jfr"]);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr stop"),
        Err(CommandError::JfrDumpFailed)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "jfr"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn jfr_command_source_matches_java_26_1_2() {
    const JFR_COMMAND_JAVA: &str = vibecraft_java_source!("/net/minecraft/server/commands/JfrCommand.java");

    for sentinel in [
        "Commands.literal(\"jfr\")", ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
        "Commands.literal(\"start\").executes(c -> startJfr((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"stop\").executes(c -> stopJfr((CommandSourceStack)c.getSource()))",
        "Environment env = Environment.from(source.getServer());", "if (!JvmProfiler.INSTANCE.start(env))",
        "throw START_FAILED.create();", "Component.translatable(\"commands.jfr.started\")", "return 1;",
        "Paths.get(\".\").relativize(JvmProfiler.INSTANCE.stop().normalize())",
        "source.getServer().isPublished() && !SharedConstants.IS_RUNNING_IN_IDE",
        "savedRecording.toAbsolutePath()", "ClickEvent.CopyToClipboard", "clipboardPath.toString()",
        "HoverEvent.ShowText(Component.translatable(\"chat.copy.click\"))",
        "Component.translatable(\"commands.jfr.stopped\", fileText)",
        "throw DUMP_FAILED.create(t.getMessage());",
    ] {
        assert!(
            JFR_COMMAND_JAVA.contains(sentinel),
            "JfrCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn perf_command_requires_owner_and_toggles_metrics_recording() {
    let mut state = ServerCommandState {
        tick_time_samples_nanos: vec![50_000_000, 60_000_000],
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("perf"), PermissionLevel::Owners);
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "perf start"),
        Err(CommandError::PermissionDenied)
    );

    let started =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf start").unwrap();
    assert!(state.perf_recording);
    assert_eq!(started.success_count, 0);
    assert_eq!(started.feedback_key, "commands.perf.started");
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf start"),
        Err(CommandError::PerfAlreadyRunning)
    );

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf stop").unwrap();
    assert!(!state.perf_recording);
    assert_eq!(stopped.success_count, 0);
    assert_eq!(stopped.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        state.perf_reports,
        vec![PerfReport {
            ticks: 2,
            duration_nanos: 110_000_000,
        }]
    );
    assert_eq!(
        state.side_feedback,
        vec![
            CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.stopped",
                broadcast_to_admins: false,
            },
            CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.reportSaved",
                broadcast_to_admins: false,
            }
        ]
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf stop"),
        Err(CommandError::PerfNotRunning)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "perf"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn perf_command_models_empty_results_and_report_failure_callbacks() {
    let mut empty = ServerCommandState {
        perf_recording: true,
        tick_time_samples_nanos: Vec::new(),
        ..ServerCommandState::default()
    };
    let stopped =
        execute_builtin_command(&mut empty, LevelBasedPermissionSet::OWNER, "perf stop").unwrap();
    assert_eq!(stopped.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        empty.perf_reports,
        vec![PerfReport {
            ticks: 0,
            duration_nanos: 0,
        }]
    );
    assert_eq!(
        empty.side_feedback,
        vec![CommandResult {
            success_count: 0,
            feedback_key: "commands.perf.reportSaved",
            broadcast_to_admins: false,
        }]
    );

    let mut failing = ServerCommandState {
        perf_recording: true,
        perf_report_should_fail: true,
        tick_time_samples_nanos: vec![50_000_000],
        ..ServerCommandState::default()
    };
    execute_builtin_command(&mut failing, LevelBasedPermissionSet::OWNER, "perf stop").unwrap();
    assert_eq!(
        failing.side_feedback,
        vec![
            CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.stopped",
                broadcast_to_admins: false,
            },
            CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.reportFailed",
                broadcast_to_admins: false,
            }
        ]
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn perf_command_source_matches_java_26_1_2() {
    const PERF_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/PerfCommand.java");

    for sentinel in [
        "Commands.literal(\"perf\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_OWNERS))",
        "Commands.literal(\"start\").executes(c -> startProfilingDedicatedServer((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"stop\").executes(c -> stopProfilingDedicatedServer((CommandSourceStack)c.getSource()))",
        "if (server.isRecordingMetrics())",
        "throw ERROR_ALREADY_RUNNING.create();",
        "server.startRecordingMetrics(onStopped, onReportFinished);",
        "Component.translatable(\"commands.perf.started\")",
        "return 0;",
        "if (!server.isRecordingMetrics())",
        "throw ERROR_NOT_RUNNING.create();",
        "server.finishRecordingMetrics();",
        "Component.translatable(\"commands.perf.reportFailed\")",
        "Component.translatable(\"commands.perf.reportSaved\", zipFile)",
        "if (results != EmptyProfileResults.EMPTY)",
        "Component.translatable(\n               \"commands.perf.stopped\"",
    ] {
        assert!(
            PERF_COMMAND_JAVA.contains(sentinel),
            "PerfCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn rotate_command_requires_gamemaster_and_records_absolute_or_relative_angles() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("rotate"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "rotate pig 90 15"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "rotate pig 90 ~-15",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.rotate.success");
    assert_eq!(
        state.rotation_requests[0],
        RotationRequest {
            target: EntityRef {
                id: "pig".to_string(),
                display_name: "pig".to_string(),
            },
            mode: RotationMode::Angles {
                yaw: 90.0,
                pitch: -15.0,
                yaw_relative: false,
                pitch_relative: true,
            },
        }
    );
}

#[test]
fn rotate_command_records_facing_entity_or_position() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "rotate pig facing entity cow eyes",
    )
    .unwrap();
    assert_eq!(
        state.rotation_requests[0],
        RotationRequest {
            target: EntityRef {
                id: "pig".to_string(),
                display_name: "pig".to_string(),
            },
            mode: RotationMode::FacingEntity {
                entity: EntityRef {
                    id: "cow".to_string(),
                    display_name: "cow".to_string(),
                },
                anchor: EntityAnchor::Eyes,
            },
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "rotate pig facing 1.5 64 -2",
    )
    .unwrap();
    assert_eq!(
        state.rotation_requests[1].mode,
        RotationMode::FacingPosition(Vec3 {
            x: 1.5,
            y: 64.0,
            z: -2.0,
        })
    );

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "rotate pig facing entity cow head"
        ),
        Err(CommandError::InvalidSyntax)
    );
}
