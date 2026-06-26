use super::*;

#[test]
fn permission_levels_clamp_and_compare_like_vanilla() {
    assert_eq!(PermissionLevel::by_id(-5), PermissionLevel::All);
    assert_eq!(PermissionLevel::by_id(0), PermissionLevel::All);
    assert_eq!(PermissionLevel::by_id(2), PermissionLevel::Gamemasters);
    assert_eq!(PermissionLevel::by_id(99), PermissionLevel::Owners);
    assert!(PermissionLevel::Admins.is_equal_or_higher_than(PermissionLevel::Gamemasters));
    assert!(!PermissionLevel::Moderators.is_equal_or_higher_than(PermissionLevel::Admins));
    assert_eq!(PermissionLevel::Owners.serialized_name(), "owners");
}

#[test]
fn level_based_permission_set_grants_command_levels_and_entity_selectors() {
    assert!(LevelBasedPermissionSet::GAMEMASTER
        .has_permission(Permission::CommandLevel(PermissionLevel::Gamemasters)));
    assert!(!LevelBasedPermissionSet::GAMEMASTER
        .has_permission(Permission::CommandLevel(PermissionLevel::Admins)));
    assert!(LevelBasedPermissionSet::GAMEMASTER.has_permission(Permission::CommandsEntitySelectors));
    assert!(!LevelBasedPermissionSet::MODERATOR.has_permission(Permission::CommandsEntitySelectors));
}

#[test]
fn command_availability_uses_root_literal_and_permission_level() {
    assert_eq!(command_required_permission("stop"), PermissionLevel::Owners);
    assert_eq!(command_required_permission("op"), PermissionLevel::Admins);
    assert_eq!(
        command_required_permission("execute"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(command_required_permission("list"), PermissionLevel::All);

    assert_eq!(
        LevelBasedPermissionSet::ALL.can_run("/list"),
        CommandAvailability::Available
    );
    assert_eq!(
        LevelBasedPermissionSet::GAMEMASTER.can_run("/stop"),
        CommandAvailability::Hidden
    );
    assert_eq!(
        LevelBasedPermissionSet::ADMIN.can_run("/op Steve"),
        CommandAvailability::Available
    );
    assert_eq!(
        LevelBasedPermissionSet::OWNER.can_run("save-all flush"),
        CommandAvailability::Available
    );
}

#[test]
fn set_idle_timeout_command_updates_minutes_and_feedback() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "/setidletimeout 5",
    )
    .unwrap();
    assert_eq!(state.player_idle_timeout_minutes, 5);
    assert_eq!(result.success_count, 5);
    assert_eq!(result.feedback_key, "commands.setidletimeout.success");

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "setidletimeout 0",
    )
    .unwrap();
    assert_eq!(state.player_idle_timeout_minutes, 0);
    assert_eq!(
        result.feedback_key,
        "commands.setidletimeout.success.disabled"
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "setidletimeout 1",
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "setidletimeout -1",
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "setidletimeout 2147483648",
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn save_commands_track_flush_and_autosave_state() {
    let mut state = ServerCommandState::default();
    let save = execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-all").unwrap();
    assert_eq!(save.success_count, 1);
    assert_eq!(save.feedback_key, "commands.save.success");
    assert!(save.broadcast_to_admins);
    assert_eq!(
        state.side_feedback,
        vec![CommandResult {
            success_count: 1,
            feedback_key: "commands.save.saving",
            broadcast_to_admins: false,
        }]
    );

    execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-all flush").unwrap();
    assert_eq!(
        state.save_all_requests,
        vec![
            SaveAllRequest { flush: false },
            SaveAllRequest { flush: true }
        ]
    );
    assert_eq!(state.side_feedback.len(), 2);

    let mut failing_save = ServerCommandState {
        save_all_should_fail: true,
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(&mut failing_save, LevelBasedPermissionSet::OWNER, "save-all"),
        Err(CommandError::SaveFailed)
    );
    assert_eq!(
        failing_save.save_all_requests,
        vec![SaveAllRequest { flush: false }]
    );
    assert_eq!(
        failing_save.side_feedback,
        vec![CommandResult {
            success_count: 1,
            feedback_key: "commands.save.saving",
            broadcast_to_admins: false,
        }]
    );

    let off =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-off").unwrap();
    assert!(!state.autosave_enabled);
    assert_eq!(off.feedback_key, "commands.save.disabled");
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-off"),
        Err(CommandError::SaveAlreadyOff)
    );

    let on =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-on").unwrap();
    assert!(state.autosave_enabled);
    assert_eq!(on.feedback_key, "commands.save.enabled");
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "save-on"),
        Err(CommandError::SaveAlreadyOn)
    );
}

#[test]
fn stop_command_requests_halt_and_requires_owner() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "stop"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "/stop").unwrap();
    assert!(state.halt_requested);
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.stop.stopping");
}

#[test]
fn schedule_command_creates_replaces_appends_and_clears_events() {
    let mut state = ServerCommandState {
        game_time_ticks: 100,
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("schedule"),
        PermissionLevel::Gamemasters
    );

    let created = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "schedule function tick/foo 5s",
    )
    .unwrap();
    assert_eq!(created.success_count, 200);
    assert_eq!(created.feedback_key, "commands.schedule.created.function");
    assert_eq!(
        state.scheduled_functions,
        vec![ScheduledFunction {
            id: "minecraft:tick/foo".to_string(),
            function: "minecraft:tick/foo".to_string(),
            tag: false,
            trigger_tick: 200,
        }]
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "schedule function tick/foo 10t replace",
    )
    .unwrap();
    assert_eq!(state.scheduled_functions.len(), 1);
    assert_eq!(state.scheduled_functions[0].trigger_tick, 110);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "schedule function tick/foo 20t append",
    )
    .unwrap();
    assert_eq!(state.scheduled_functions.len(), 2);

    let cleared = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "schedule clear minecraft:tick/foo",
    )
    .unwrap();
    assert_eq!(cleared.success_count, 2);
    assert_eq!(cleared.feedback_key, "commands.schedule.cleared.success");
    assert!(state.scheduled_functions.is_empty());
}

#[test]
fn schedule_command_handles_tags_and_vanilla_failures() {
    let mut state = ServerCommandState {
        game_time_ticks: i32::MAX as u64 - 2,
        macro_functions: vec!["minecraft:macro".to_string()],
        ..ServerCommandState::default()
    };

    let tag = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "schedule function #tick/load 5t",
    )
    .unwrap();
    assert_eq!(tag.success_count, 3);
    assert_eq!(tag.feedback_key, "commands.schedule.created.tag");
    assert_eq!(state.scheduled_functions[0].id, "#minecraft:tick/load");
    assert!(state.scheduled_functions[0].tag);

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "schedule function tick/load 0t"
        ),
        Err(CommandError::ScheduleSameTick)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "schedule function macro 1t"
        ),
        Err(CommandError::ScheduleMacro)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "schedule clear minecraft:none"
        ),
        Err(CommandError::ScheduleCantRemove)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "schedule function Bad 1t"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn function_command_queues_single_function_tags_and_arguments() {
    let mut state = ServerCommandState {
        command_source_dimension: "minecraft:the_nether".to_string(),
        function_permission_level: PermissionLevel::Admins,
        available_functions: vec![
            CommandFunctionDefinition {
                id: "minecraft:tick/foo".to_string(),
                commands: vec!["say one".to_string(), "return 4".to_string()],
                macro_parameters: Vec::new(),
            },
            CommandFunctionDefinition {
                id: "minecraft:tick/bar".to_string(),
                commands: vec!["say $(name)".to_string()],
                macro_parameters: vec!["name".to_string()],
            },
        ],
        function_tags: vec![CommandFunctionTag {
            id: "minecraft:tick/load".to_string(),
            functions: vec![
                "minecraft:tick/foo".to_string(),
                "minecraft:tick/bar".to_string(),
            ],
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("function"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "function tick/foo"
        ),
        Err(CommandError::PermissionDenied)
    );

    let single = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "function tick/foo",
    )
    .unwrap();
    assert_eq!(single.success_count, 1);
    assert_eq!(single.feedback_key, "commands.function.scheduled.single");
    assert_eq!(
        state.queued_functions[0],
        QueuedFunctionCall {
            id: "minecraft:tick/foo".to_string(),
            commands: vec!["say one".to_string(), "return 4".to_string()],
            arguments: None,
            source_dimension: "minecraft:the_nether".to_string(),
            suppressed_output: true,
            permission_level: PermissionLevel::Gamemasters,
        }
    );

    let tagged = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "function #tick/load {name:\"Steve\"}",
    )
    .unwrap();
    assert_eq!(tagged.success_count, 2);
    assert_eq!(tagged.feedback_key, "commands.function.scheduled.multiple");
    assert_eq!(state.queued_functions.len(), 3);
    assert_eq!(
        state.queued_functions[2].arguments,
        Some("{name:\"Steve\"}".to_string())
    );
    assert_eq!(
        state.queued_functions[2].commands,
        vec!["say Steve".to_string()]
    );
}

#[test]
fn function_command_reports_missing_and_argument_failures() {
    let mut state = ServerCommandState {
        available_functions: vec![CommandFunctionDefinition {
            id: "minecraft:macro".to_string(),
            commands: vec!["say $(name)".to_string()],
            macro_parameters: vec!["name".to_string()],
        }],
        function_tags: vec![CommandFunctionTag {
            id: "minecraft:empty".to_string(),
            functions: Vec::new(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function missing"
        ),
        Err(CommandError::FunctionNoFunctions)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function #empty"
        ),
        Err(CommandError::FunctionNoFunctions)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function macro"
        ),
        Err(CommandError::FunctionInstantiationFailure)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function macro not_compound"
        ),
        Err(CommandError::FunctionArgumentNotCompound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function Bad"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn macro_function_instantiation_substitutes_compound_arguments_like_vanilla() {
    let function = CommandFunctionDefinition {
        id: "minecraft:macro".to_string(),
        commands: vec![
            "say plain".to_string(),
            "say $(name) $(count) $(ratio) $(payload)".to_string(),
        ],
        macro_parameters: vec![
            "name".to_string(),
            "count".to_string(),
            "ratio".to_string(),
            "payload".to_string(),
        ],
    };
    let instantiated = instantiate_command_function(
        &function,
        Some(r#"{name:"Steve",count:7l,ratio:1.500000f,payload:{ok:1b}}"#),
    )
    .unwrap();
    assert_eq!(
        instantiated.commands,
        vec![
            "say plain".to_string(),
            "say Steve 7 1.5 {ok:1b}".to_string()
        ]
    );
    assert!(instantiate_command_function(&function, Some(r#"{name:"Steve"}"#)).is_err());
    assert!(instantiate_command_function(&function, None).is_err());
}

#[test]
fn string_template_model_scans_and_substitutes_like_java() {
    const { assert!(COMMAND_FUNCTIONS_PACKAGE_NULL_MARKED) };
    let template = StringTemplateModel::from_string("say $(name) $$ $(count)").unwrap();
    assert_eq!(
        template.segments,
        vec!["say ".to_string(), " $$ ".to_string()]
    );
    assert_eq!(
        template.variables,
        vec!["name".to_string(), "count".to_string()]
    );
    assert_eq!(
        template
            .substitute(&["Alex".to_string(), "3".to_string()])
            .unwrap(),
        "say Alex $$ 3"
    );
    assert!(StringTemplateModel::from_string("say $name")
        .unwrap_err()
        .contains("No variables"));
    assert!(StringTemplateModel::from_string("say $(name")
        .unwrap_err()
        .contains("Unterminated"));
    assert!(StringTemplateModel::from_string("say $(bad-name)")
        .unwrap_err()
        .contains("Invalid macro variable name"));
}

#[test]
fn function_builder_model_converts_plain_entries_like_java() {
    let mut plain_builder = FunctionBuilderModel::new();
    plain_builder.add_command("say before");
    let mut plain_function = plain_builder.build("example:plain");
    assert_eq!(plain_function.id(), "example:plain");
    assert_eq!(
        plain_function
            .instantiate(Some(r#"{ignored:"argument"}"#))
            .unwrap(),
        InstantiatedFunctionModel {
            id: "example:plain".to_string(),
            entries: vec!["say before".to_string()],
        }
    );

    let mut function = macro_function_model_fixture();
    assert_eq!(function.id(), "example:macro");
    let CommandFunctionModel::Macro(macro_function) = &function else {
        panic!("builder should produce a macro function after add_macro");
    };
    assert_eq!(macro_function.parameters, vec!["name", "count"]);
    assert_eq!(
        macro_function.entries,
        vec![
            FunctionEntryModel::Plain("say before".to_string()),
            FunctionEntryModel::Macro {
                template: StringTemplateModel::from_string("say $(name)").unwrap(),
                parameter_indices: vec![0],
            },
            FunctionEntryModel::Plain("say after".to_string()),
            FunctionEntryModel::Macro {
                template: StringTemplateModel::from_string("say $(count) $(name)").unwrap(),
                parameter_indices: vec![1, 0],
            },
        ]
    );

    assert_eq!(
        function
            .instantiate(Some(r#"{name:"Steve",count:7l}"#))
            .unwrap(),
        InstantiatedFunctionModel {
            id: instantiated_function_id(
                "example:macro",
                &["name".to_string(), "count".to_string()]
            ),
            entries: vec![
                "say before".to_string(),
                "say Steve".to_string(),
                "say after".to_string(),
                "say 7 Steve".to_string(),
            ],
        }
    );
}

#[test]
fn macro_function_model_uses_java_move_to_last_cache() {
    let mut function = macro_function_model_fixture();
    function
        .instantiate(Some(r#"{name:"Steve",count:7l}"#))
        .unwrap();
    for count in 8..16 {
        function
            .instantiate(Some(&format!(r#"{{name:"Name{count}",count:{count}l}}"#)))
            .unwrap();
    }
    let CommandFunctionModel::Macro(macro_function) = &mut function else {
        panic!("builder should remain a macro function");
    };
    assert_eq!(macro_function.cached_keys().len(), 8);
    assert!(!macro_function
        .cached_keys()
        .contains(&vec!["Steve".to_string(), "7".to_string()]));
    assert_eq!(
        macro_function.cached_keys().last().unwrap(),
        &vec!["Name15".to_string(), "15".to_string()]
    );

    macro_function
        .instantiate(Some(r#"{name:"Name8",count:8l}"#))
        .unwrap();
    assert_eq!(
        macro_function.cached_keys().last().unwrap(),
        &vec!["Name8".to_string(), "8".to_string()]
    );
}

fn macro_function_model_fixture() -> CommandFunctionModel {
    let mut builder = FunctionBuilderModel::new();
    builder.add_command("say before");
    builder.add_macro("say $(name)", 2).unwrap();
    builder.add_command("say after");
    builder.add_macro("say $(count) $(name)", 4).unwrap();
    builder.build("example:macro")
}

#[test]
fn function_with_entity_block_and_storage_sources_instantiates_macros() {
    let function = CommandFunctionDefinition {
        id: "minecraft:macro".to_string(),
        commands: vec!["say $(name) $(value)".to_string()],
        macro_parameters: vec!["name".to_string(), "value".to_string()],
    };
    let mut state = ServerCommandState {
        available_functions: vec![function],
        macro_functions: vec!["minecraft:macro".to_string()],
        macro_entity_nbt_sources: vec![CommandEntityNbtSource {
            entity: EntityRef {
                id: "entity-1".to_string(),
                display_name: "Pig".to_string(),
            },
            nbt: Tag::Compound(vec![
                ("name".to_string(), Tag::String("entity".to_string())),
                ("value".to_string(), Tag::Int(1)),
            ]),
        }],
        macro_block_nbt_sources: vec![CommandBlockNbtSource {
            pos: BlockPos { x: 1, y: 2, z: 3 },
            nbt: Tag::Compound(vec![
                ("name".to_string(), Tag::String("block".to_string())),
                ("value".to_string(), Tag::Int(2)),
            ]),
        }],
        macro_storage_nbt_sources: vec![CommandStorageNbtSource {
            id: "minecraft:test".to_string(),
            nbt: Tag::Compound(vec![
                ("name".to_string(), Tag::String("storage".to_string())),
                ("value".to_string(), Tag::Int(3)),
            ]),
        }],
        ..ServerCommandState::default()
    };

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "function minecraft:macro with entity Pig",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "function minecraft:macro with block 1 2 3",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "function minecraft:macro with storage minecraft:test",
    )
    .unwrap();

    assert_eq!(
        state
            .queued_functions
            .iter()
            .map(|call| call.commands[0].clone())
            .collect::<Vec<_>>(),
        vec![
            "say entity 1".to_string(),
            "say block 2".to_string(),
            "say storage 3".to_string(),
        ]
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "function minecraft:macro with storage missing:value",
        ),
        Err(CommandError::FunctionInstantiationFailure)
    );
}

#[test]
fn command_function_resources_parse_vanilla_lines_macros_and_tags() {
    let functions = load_command_functions_from_resources([
        (
            "data/example/function/tools/start.mcfunction",
            "  # comment\nsay one\\\n two\n$give @s minecraft:$(item)\n",
        ),
        (
            "data/example/tags/function/load.json",
            r##"{"values":["example:tools/start",{"id":"#example:nested","required":false}]}"##,
        ),
        ("data/example/function/ignored.txt", "say ignored"),
    ])
    .unwrap();
    assert_eq!(
        functions,
        vec![CommandFunctionDefinition {
            id: "example:tools/start".to_string(),
            commands: vec![
                "say onetwo".to_string(),
                "give @s minecraft:$(item)".to_string()
            ],
            macro_parameters: vec!["item".to_string()],
        }]
    );

    let tags = load_command_function_tags_from_resources([(
        "data/example/tags/function/load.json",
        r##"{"values":["example:tools/start",{"id":"#example:nested","required":false}]}"##,
    )])
    .unwrap();
    assert_eq!(
        tags,
        vec![CommandFunctionTag {
            id: "example:load".to_string(),
            functions: vec![
                "example:tools/start".to_string(),
                "#example:nested".to_string()
            ],
        }]
    );
}

#[test]
fn command_function_resources_reject_vanilla_parse_errors() {
    assert!(load_command_functions_from_resources([(
        "data/example/function/slash.mcfunction",
        "/say no"
    )])
    .unwrap_err()
    .contains("Invalid leading slash"));
    assert!(load_command_functions_from_resources([(
        "data/example/function/bad_macro.mcfunction",
        "$say no variables"
    )])
    .unwrap_err()
    .contains("has no variables"));
    assert!(load_command_functions_from_resources([(
        "data/example/function/unterminated.mcfunction",
        "say one\\"
    )])
    .unwrap_err()
    .contains("Line continuation at end"));
}

#[test]
fn server_function_tick_queues_load_once_then_tick_when_running() {
    let mut command_state = ServerCommandState {
        command_source_dimension: "minecraft:overworld".to_string(),
        available_functions: vec![
            CommandFunctionDefinition {
                id: "minecraft:load/init".to_string(),
                commands: vec!["say load".to_string()],
                macro_parameters: Vec::new(),
            },
            CommandFunctionDefinition {
                id: "minecraft:tick/loop".to_string(),
                commands: vec!["say tick".to_string()],
                macro_parameters: Vec::new(),
            },
        ],
        function_tags: vec![
            CommandFunctionTag {
                id: "minecraft:load".to_string(),
                functions: vec!["minecraft:load/init".to_string()],
            },
            CommandFunctionTag {
                id: "minecraft:tick".to_string(),
                functions: vec![
                    "minecraft:tick/loop".to_string(),
                    "#minecraft:nested".to_string(),
                ],
            },
        ],
        ..ServerCommandState::default()
    };
    let mut function_state = ServerFunctionTickState::default();

    assert!(queue_server_function_tick(&mut command_state, &mut function_state, false).is_empty());
    assert!(function_state.post_reload);

    assert_eq!(
        queue_server_function_tick(&mut command_state, &mut function_state, true),
        vec![
            "minecraft:load/init".to_string(),
            "minecraft:tick/loop".to_string()
        ]
    );
    assert!(!function_state.post_reload);
    assert_eq!(command_state.queued_functions.len(), 2);
    assert!(command_state.queued_functions.iter().all(
        |call| call.suppressed_output && call.permission_level == PermissionLevel::Gamemasters
    ));

    assert_eq!(
        queue_server_function_tick(&mut command_state, &mut function_state, true),
        vec!["minecraft:tick/loop".to_string()]
    );
    assert_eq!(command_state.queued_functions.len(), 3);

    command_state.available_functions.extend([
        CommandFunctionDefinition {
            id: "minecraft:load/reloaded".to_string(),
            commands: vec!["say reload".to_string()],
            macro_parameters: Vec::new(),
        },
        CommandFunctionDefinition {
            id: "minecraft:tick/reloaded".to_string(),
            commands: vec!["say tick reload".to_string()],
            macro_parameters: Vec::new(),
        },
    ]);
    command_state.function_tags = vec![
        CommandFunctionTag {
            id: "minecraft:load".to_string(),
            functions: vec!["minecraft:load/reloaded".to_string()],
        },
        CommandFunctionTag {
            id: "minecraft:tick".to_string(),
            functions: vec!["minecraft:tick/reloaded".to_string()],
        },
    ];
    function_state.post_reload = true;

    assert_eq!(
        queue_server_function_tick(&mut command_state, &mut function_state, true),
        vec![
            "minecraft:load/reloaded".to_string(),
            "minecraft:tick/reloaded".to_string()
        ]
    );
    assert_eq!(
        command_state.queued_functions[3..]
            .iter()
            .map(|call| call.id.as_str())
            .collect::<Vec<_>>(),
        vec!["minecraft:load/reloaded", "minecraft:tick/reloaded"]
    );
    assert_eq!(
        queue_server_function_tick(&mut command_state, &mut function_state, true),
        vec!["minecraft:tick/reloaded".to_string()]
    );
}

#[test]
fn seed_command_reports_level_seed_with_gamemaster_permission() {
    let mut state = ServerCommandState {
        world_seed: 8_675_309,
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "seed"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "/seed").unwrap();
    assert_eq!(result.success_count, 8_675_309);
    assert_eq!(result.feedback_key, "commands.seed.success");
    assert!(!result.broadcast_to_admins);

    state.world_seed = i32::MAX as i64 + 1;
    let wrapped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "seed").unwrap();
    assert_eq!(wrapped.success_count, i32::MIN);
}

#[test]
fn scoreboard_objectives_and_display_slots_follow_vanilla_feedbacks() {
    let mut state = ServerCommandState::default();

    assert_eq!(
        command_required_permission("scoreboard"),
        PermissionLevel::Gamemasters
    );
    let empty = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives list",
    )
    .unwrap();
    assert_eq!(
        empty.feedback_key,
        "commands.scoreboard.objectives.list.empty"
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives add kills dummy Kills",
    )
    .unwrap();
    assert_eq!(
        state.scoreboard_objectives[0],
        ScoreboardObjective {
            name: "kills".to_string(),
            criteria: "dummy".to_string(),
            display_name: "Kills".to_string(),
            render_type: "integer".to_string(),
            display_auto_update: true,
            number_format: None,
        }
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard objectives add kills dummy"
        ),
        Err(CommandError::ScoreboardObjectiveAlreadyExists)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives modify kills rendertype hearts",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives modify kills displayautoupdate false",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives modify kills numberformat fixed !",
    )
    .unwrap();
    assert_eq!(state.scoreboard_objectives[0].render_type, "hearts");
    assert!(!state.scoreboard_objectives[0].display_auto_update);
    assert_eq!(
        state.scoreboard_objectives[0].number_format,
        Some("fixed:!".to_string())
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives setdisplay sidebar kills",
    )
    .unwrap();
    assert_eq!(state.scoreboard_display_slots[0].slot, "sidebar");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard objectives setdisplay sidebar kills"
        ),
        Err(CommandError::ScoreboardDisplayAlreadySet)
    );
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard objectives setdisplay sidebar",
    )
    .unwrap();
    assert!(state.scoreboard_display_slots.is_empty());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard objectives setdisplay sidebar"
        ),
        Err(CommandError::ScoreboardDisplayAlreadyEmpty)
    );
}

#[test]
fn scoreboard_players_set_get_arithmetic_reset_and_display_overrides() {
    let mut state = ServerCommandState {
        scoreboard_objectives: vec![ScoreboardObjective {
            name: "kills".to_string(),
            criteria: "dummy".to_string(),
            display_name: "Kills".to_string(),
            render_type: "integer".to_string(),
            display_auto_update: true,
            number_format: None,
        }],
        ..ServerCommandState::default()
    };

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players set Steve kills 5",
    )
    .unwrap();
    assert_eq!(state.scoreboard_scores[0].value, 5);
    let get = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players get Steve kills",
    )
    .unwrap();
    assert_eq!(get.success_count, 5);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players add Steve kills 2",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players remove Steve kills 3",
    )
    .unwrap();
    assert_eq!(state.scoreboard_scores[0].value, 4);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players display name Steve kills Slayer",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players display numberformat Steve kills blank",
    )
    .unwrap();
    assert_eq!(
        state.scoreboard_scores[0].display_name,
        Some("Slayer".to_string())
    );
    assert_eq!(
        state.scoreboard_scores[0].number_format,
        Some("blank".to_string())
    );

    let list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players list",
    )
    .unwrap();
    assert_eq!(list.success_count, 1);
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players reset Steve kills",
    )
    .unwrap();
    assert!(state.scoreboard_scores.is_empty());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard players get Steve kills"
        ),
        Err(CommandError::ScoreboardScoreNotFound)
    );
}

#[test]
fn scoreboard_players_trigger_and_operations_match_core_rules() {
    let mut state = ServerCommandState {
        scoreboard_objectives: vec![
            ScoreboardObjective {
                name: "triggered".to_string(),
                criteria: "trigger".to_string(),
                display_name: "Triggered".to_string(),
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
        ..ServerCommandState::default()
    };

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players enable Steve triggered",
    )
    .unwrap();
    assert!(!state.scoreboard_scores[0].locked);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard players enable Steve triggered"
        ),
        Err(CommandError::ScoreboardTriggerAlreadyEnabled)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard players enable Steve kills"
        ),
        Err(CommandError::ScoreboardNotTrigger)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players set Steve kills 4",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players set Alex kills 3",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "scoreboard players operation Steve kills += Alex kills",
    )
    .unwrap();
    assert_eq!(
        state
            .scoreboard_scores
            .iter()
            .find(|score| score.owner == "Steve" && score.objective == "kills")
            .unwrap()
            .value,
        7
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "scoreboard players operation Steve kills /= Alex missing"
        ),
        Err(CommandError::ScoreboardObjectiveNotFound)
    );
}
