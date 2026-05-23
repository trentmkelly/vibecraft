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
        "msg Alex -- secret plan",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::Private);
    assert_eq!(state.chat_events[0].targets[0].name, "Alex");
    assert_eq!(state.chat_events[0].message, "secret plan");

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "tell Steve hello").unwrap();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "w Alex hi").unwrap();
    assert_eq!(state.chat_events.len(), 3);
}

#[test]
fn teammsg_requires_source_team_and_targets_team_members() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
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
        "tellraw Steve Alex -- {\"text\":\"hi\"}",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(state.chat_events[0].kind, ChatCommandKind::TellRaw);
    assert_eq!(state.chat_events[0].message, "{\"text\":\"hi\"}");
    assert_eq!(state.chat_events[0].targets.len(), 2);
}

#[test]
fn playsound_requires_gamemaster_and_defaults_to_source_player() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("playsound"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "playsound minecraft:block.note_block.harp"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "playsound minecraft:block.note_block.harp",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.playsound.success.single");
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Play(PlaySoundRequest {
            sound: "minecraft:block.note_block.harp".to_string(),
            source: SoundSource::Master,
            targets: vec![NameAndId::create_offline("Steve")],
            position: Vec3::default(),
            volume: 1.0,
            pitch: 1.0,
            min_volume: 0.0,
        })
    );
}

#[test]
fn playsound_accepts_source_targets_position_volume_pitch_and_min_volume() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "playsound minecraft:entity.arrow.hit player Steve,Alex 1.5 2.0 3.5 4.0 0.75 0.25",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.playsound.success.multiple");
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Play(PlaySoundRequest {
            sound: "minecraft:entity.arrow.hit".to_string(),
            source: SoundSource::Player,
            targets: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3 {
                x: 1.5,
                y: 2.0,
                z: 3.5,
            },
            volume: 4.0,
            pitch: 0.75,
            min_volume: 0.25,
        })
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:bad player Steve 0 0 0 1 2.5"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn playsound_fails_when_no_target_receives_sound() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "playsound minecraft:empty"
        ),
        Err(CommandError::PlaySoundTooFar)
    );
}

#[test]
fn stopsound_queues_source_and_sound_filters() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("stopsound"),
        PermissionLevel::Gamemasters
    );
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve,Alex player minecraft:entity.arrow.hit",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(
        result.feedback_key,
        "commands.stopsound.success.source.sound"
    );
    assert_eq!(
        state.sound_events[0],
        SoundCommandEvent::Stop(StopSoundRequest {
            targets: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            source: Some(SoundSource::Player),
            sound: Some("minecraft:entity.arrow.hit".to_string()),
        })
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "stopsound Steve * minecraft:music.menu",
    )
    .unwrap();
    assert_eq!(
        result.feedback_key,
        "commands.stopsound.success.sourceless.sound"
    );
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
fn particle_command_requires_gamemaster_and_defaults_to_all_online_players() {
    let mut state = ServerCommandState {
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("particle"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "particle flame"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "particle flame",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.particle.success");
    assert_eq!(
        state.particle_events[0],
        ParticleCommandEvent {
            name: "flame".to_string(),
            viewers: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3::default(),
            delta: Vec3::default(),
            speed: 0.0,
            count: 0,
            force: false,
        }
    );
}

#[test]
fn particle_command_parses_position_delta_speed_count_mode_and_viewers() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "particle minecraft:dust 1 2 3 0.1 0.2 0.3 0.4 12 force Steve,Alex",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(
        state.particle_events[0],
        ParticleCommandEvent {
            name: "minecraft:dust".to_string(),
            viewers: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex")
            ],
            position: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            delta: Vec3 {
                x: 0.1,
                y: 0.2,
                z: 0.3,
            },
            speed: 0.4,
            count: 12,
            force: true,
        }
    );
}

#[test]
fn particle_command_fails_when_no_players_receive_particles() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame"
        ),
        Err(CommandError::ParticleFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "particle flame 0 0 0 0 0 0 -1 1"
        ),
        Err(CommandError::InvalidSyntax)
    );
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
    assert_eq!(stopped.feedback_key, "commands.perf.stopped");
    assert_eq!(
        state.perf_reports,
        vec![PerfReport {
            ticks: 2,
            duration_nanos: 110_000_000,
        }]
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
