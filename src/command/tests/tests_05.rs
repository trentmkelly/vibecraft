use super::*;

#[test]
fn transfer_command_requires_admin_and_valid_port() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..Default::default()
    };
    assert_eq!(
        command_required_permission("transfer"),
        PermissionLevel::Admins
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "transfer example.org"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "transfer example.org 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "transfer example.org 65536"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn transfer_command_defaults_to_source_player_and_port() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "transfer mc.test",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.transfer.success.single");
    assert_eq!(state.transfer_requests.len(), 1);
    assert_eq!(state.transfer_requests[0].host, "mc.test");
    assert_eq!(state.transfer_requests[0].port, 25565);
    assert_eq!(state.transfer_requests[0].targets[0].name, "Steve");
}

#[test]
fn transfer_command_targets_explicit_players() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "transfer mc.test"
        ),
        Err(CommandError::NoPlayers)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "transfer mc.test 25566 Steve Alex",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(result.feedback_key, "commands.transfer.success.multiple");
    assert_eq!(state.transfer_requests.len(), 1);
    assert_eq!(state.transfer_requests[0].host, "mc.test");
    assert_eq!(state.transfer_requests[0].port, 25566);
    assert_eq!(
        state.transfer_requests[0]
            .targets
            .iter()
            .map(|target| target.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Steve", "Alex"]
    );
}

#[test]
fn weather_command_uses_gamemaster_permission_and_vanilla_feedback() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("weather"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "weather rain"
        ),
        Err(CommandError::PermissionDenied)
    );

    let rain = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "weather rain",
    )
    .unwrap();
    assert_eq!(state.weather.mode, WeatherMode::Rain);
    assert_eq!(state.weather.duration_ticks, None);
    assert_eq!(rain.success_count, -1);
    assert_eq!(rain.feedback_key, "commands.weather.set.rain");
    assert!(rain.broadcast_to_admins);
}

#[test]
fn weather_command_accepts_clear_rain_thunder_with_time_arguments() {
    let mut state = ServerCommandState::default();
    let clear = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "weather clear 1s",
    )
    .unwrap();
    assert_eq!(state.weather.mode, WeatherMode::Clear);
    assert_eq!(state.weather.duration_ticks, Some(20));
    assert_eq!(clear.success_count, 20);
    assert_eq!(clear.feedback_key, "commands.weather.set.clear");

    let thunder = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "weather thunder 2d",
    )
    .unwrap();
    assert_eq!(state.weather.mode, WeatherMode::Thunder);
    assert_eq!(state.weather.duration_ticks, Some(48_000));
    assert_eq!(thunder.success_count, 48_000);
    assert_eq!(thunder.feedback_key, "commands.weather.set.thunder");

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "weather rain 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn warden_spawn_tracker_command_sets_and_clears_source_tracker() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("warden_spawn_tracker"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "warden_spawn_tracker set 1"
        ),
        Err(CommandError::PermissionDenied)
    );

    let set = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "warden_spawn_tracker set 4",
    )
    .unwrap();
    assert_eq!(set.success_count, 1);
    assert_eq!(
        set.feedback_key,
        "commands.warden_spawn_tracker.set.success.single"
    );
    assert!(set.broadcast_to_admins);
    assert_eq!(
        state.warden_spawn_trackers,
        vec![WardenSpawnTrackerState {
            player: NameAndId::create_offline("Steve"),
            warning_level: 4,
        }]
    );

    let clear = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "warden_spawn_tracker clear",
    )
    .unwrap();
    assert_eq!(clear.success_count, 1);
    assert_eq!(
        clear.feedback_key,
        "commands.warden_spawn_tracker.clear.success.single"
    );
    assert_eq!(state.warden_spawn_trackers[0].warning_level, 0);
}

#[test]
fn warden_spawn_tracker_command_rejects_bad_levels_and_non_players() {
    let mut state = ServerCommandState {
        command_source_player: Some(NameAndId::create_offline("Steve")),
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "warden_spawn_tracker set 5"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "warden_spawn_tracker set -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    state.command_source_player = None;
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "warden_spawn_tracker clear"
        ),
        Err(CommandError::NoPlayers)
    );
}

#[test]
fn waypoint_command_lists_and_modifies_waypoint_icons() {
    let mut state = ServerCommandState {
        command_source_dimension: "minecraft:overworld".to_string(),
        waypoints: vec![
            WaypointState {
                entity: entity_ref("Steve"),
                dimension: "minecraft:overworld".to_string(),
                color: None,
                style: "minecraft:default".to_string(),
            },
            WaypointState {
                entity: entity_ref("Alex"),
                dimension: "minecraft:the_nether".to_string(),
                color: None,
                style: "minecraft:default".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("waypoint"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "waypoint list"
        ),
        Err(CommandError::PermissionDenied)
    );

    let list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint list",
    )
    .unwrap();
    assert_eq!(list.success_count, 1);
    assert_eq!(list.feedback_key, "commands.waypoint.list.success");
    assert!(!list.broadcast_to_admins);

    let color = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint modify Steve color red",
    )
    .unwrap();
    assert_eq!(color.success_count, 0);
    assert_eq!(color.feedback_key, "commands.waypoint.modify.color");
    assert_eq!(state.waypoints[0].color, Some(0xFF5555));

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint modify Steve color hex #123ABC",
    )
    .unwrap();
    assert_eq!(state.waypoints[0].color, Some(0x123ABC));

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint modify Steve style set bowtie",
    )
    .unwrap();
    assert_eq!(state.waypoints[0].style, "minecraft:bowtie");

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint modify Steve color reset",
    )
    .unwrap();
    assert_eq!(state.waypoints[0].color, None);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint modify Steve style reset",
    )
    .unwrap();
    assert_eq!(state.waypoints[0].style, "minecraft:default");
}

#[test]
fn waypoint_command_reports_empty_lists_and_rejects_invalid_waypoints() {
    let mut state = ServerCommandState {
        command_source_dimension: "minecraft:the_end".to_string(),
        waypoints: vec![WaypointState {
            entity: entity_ref("Steve"),
            dimension: "minecraft:overworld".to_string(),
            color: None,
            style: "minecraft:default".to_string(),
        }],
        ..ServerCommandState::default()
    };
    let list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "waypoint list",
    )
    .unwrap();
    assert_eq!(list.success_count, 0);
    assert_eq!(list.feedback_key, "commands.waypoint.list.empty");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "waypoint modify Missing color red"
        ),
        Err(CommandError::WaypointInvalid)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "waypoint modify Steve color hex nope"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "waypoint modify Steve color rainbow"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn worldborder_command_sets_adds_centers_and_queries_border() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("worldborder"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "worldborder set 100"
        ),
        Err(CommandError::PermissionDenied)
    );

    let set = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder set 100",
    )
    .unwrap();
    assert_eq!(set.feedback_key, "commands.worldborder.set.immediate");
    assert_eq!(state.world_border.size(), 100.0);
    assert!(set.broadcast_to_admins);

    let grow = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder add 50 2s",
    )
    .unwrap();
    assert_eq!(grow.success_count, 50);
    assert_eq!(grow.feedback_key, "commands.worldborder.set.grow");
    assert_eq!(state.world_border.lerp_target(), 150.0);
    assert_eq!(state.world_border.lerp_time(), 40);

    let center = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder center 10.5 -20.25",
    )
    .unwrap();
    assert_eq!(center.success_count, 0);
    assert_eq!(center.feedback_key, "commands.worldborder.center.success");
    assert_eq!(state.world_border.center_x, 10.5);
    assert_eq!(state.world_border.center_z, -20.25);

    let get = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder get",
    )
    .unwrap();
    assert_eq!(get.feedback_key, "commands.worldborder.get");
    assert_eq!(get.success_count, 100);
    assert!(!get.broadcast_to_admins);
}

#[test]
fn worldborder_command_updates_damage_and_warning_settings() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder damage buffer 8.5",
    )
    .unwrap();
    assert_eq!(state.world_border.safe_zone, 8.5);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder damage amount 0.75",
    )
    .unwrap();
    assert_eq!(state.world_border.damage_per_block, 0.75);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder warning distance 12",
    )
    .unwrap();
    assert_eq!(state.world_border.warning_blocks, 12);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder warning time 5s",
    )
    .unwrap();
    assert_eq!(state.world_border.warning_time, 100);
}

#[test]
fn world_state_commands_update_client_observable_runtime_state() {
    let mut state = ServerCommandState {
        world_clock_ticks: 23_500,
        ..ServerCommandState::default()
    };

    let set_midnight = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time set midnight",
    )
    .unwrap();
    assert_eq!(state.world_clock_ticks, 18_000);
    assert_eq!(set_midnight.success_count, 18_000);
    assert_eq!(set_midnight.feedback_key, "commands.time.set.time_marker");
    assert!(set_midnight.broadcast_to_admins);

    let pause = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "time pause",
    )
    .unwrap();
    assert!(state.world_clock_paused);
    assert_eq!(pause.feedback_key, "commands.time.pause");

    let thunder = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "weather thunder 30s",
    )
    .unwrap();
    assert_eq!(state.weather.mode, WeatherMode::Thunder);
    assert_eq!(state.weather.duration_ticks, Some(600));
    assert_eq!(thunder.success_count, 600);
    assert_eq!(thunder.feedback_key, "commands.weather.set.thunder");
    assert!(thunder.broadcast_to_admins);

    let initial_border_size = state.world_border.size();
    let border = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder set 128 10s",
    )
    .unwrap();
    assert_eq!(state.world_border.lerp_target(), 128.0);
    assert_eq!(state.world_border.lerp_time(), 200);
    assert_eq!(border.success_count, (128.0 - initial_border_size) as i32);
    assert_eq!(border.feedback_key, "commands.worldborder.set.shrink");
    assert!(border.broadcast_to_admins);

    let warning = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "worldborder warning distance 8",
    )
    .unwrap();
    assert_eq!(state.world_border.warning_blocks, 8);
    assert_eq!(warning.success_count, 8);
    assert_eq!(
        warning.feedback_key,
        "commands.worldborder.warning.distance.success"
    );
}

#[test]
fn worldborder_command_rejects_vanilla_failure_paths() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder set 0.5"
        ),
        Err(CommandError::WorldBorderTooSmall)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder set 60000000"
        ),
        Err(CommandError::WorldBorderTooBig)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder center 30000000 0"
        ),
        Err(CommandError::WorldBorderTooFarOut)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder center 0 0"
        ),
        Err(CommandError::WorldBorderSameCenter)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder damage buffer 5"
        ),
        Err(CommandError::WorldBorderSameDamageBuffer)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "worldborder warning distance 5"
        ),
        Err(CommandError::WorldBorderSameWarningDistance)
    );
}

#[test]
fn publish_command_requires_owner_and_records_default_publish_request() {
    let mut state = ServerCommandState {
        next_available_publish_port: 24454,
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("publish"),
        PermissionLevel::Owners
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "publish"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "publish").unwrap();
    assert_eq!(result.success_count, 24454);
    assert_eq!(result.feedback_key, "commands.publish.started");
    assert!(result.broadcast_to_admins);
    assert_eq!(
        state.published_server,
        Some(super::PublishRequest {
            port: 24454,
            allow_commands: false,
            gamemode: None,
        })
    );
}

#[test]
fn publish_command_parses_allow_commands_gamemode_and_port() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::OWNER,
        "publish true creative 0",
    )
    .unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(
        state.published_server,
        Some(super::PublishRequest {
            port: 0,
            allow_commands: true,
            gamemode: Some(GameMode::Creative),
        })
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "publish false survival"
        ),
        Err(CommandError::PublishAlreadyPublished)
    );
}

#[test]
fn publish_command_rejects_invalid_bool_gamemode_or_port() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "publish yes"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "publish true builder"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::OWNER,
            "publish true creative 65536"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn publish_command_reports_server_publish_failure_without_recording_request() {
    let mut state = ServerCommandState {
        publish_should_fail: true,
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, "publish"),
        Err(CommandError::PublishFailed)
    );
    assert_eq!(state.published_server, None);
}

#[test]
fn random_value_and_roll_sample_ranges_without_permission() {
    let mut state = ServerCommandState {
        world_seed: 123,
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("random"), PermissionLevel::All);

    let value = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "random value 1..6",
    )
    .unwrap();
    assert!((1..=6).contains(&value.success_count));
    assert_eq!(value.feedback_key, "commands.random.sample.success");
    assert!(!value.broadcast_to_admins);
    assert!(!state.random_broadcasts[0].announced);

    let roll = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "random roll -2..2",
    )
    .unwrap();
    assert!((-2..=2).contains(&roll.success_count));
    assert_eq!(roll.feedback_key, "commands.random.roll");
    assert!(state.random_broadcasts[1].announced);
}

#[test]
fn random_command_rejects_vanilla_range_errors() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "random value 5"),
        Err(CommandError::RandomRangeTooSmall)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "random value .."),
        Err(CommandError::RandomRangeTooLarge)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "random value 6..1"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn random_named_sequences_and_resets_require_gamemaster() {
    let mut state = ServerCommandState {
        world_seed: 99,
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "random value 1..10 minecraft:test"
        ),
        Err(CommandError::PermissionDenied)
    );

    let sample = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "random value 1..10 minecraft:test",
    )
    .unwrap();
    assert!((1..=10).contains(&sample.success_count));
    assert_eq!(state.random_sequences.sequences.len(), 1);
    assert!(state.random_sequences.sequences.contains_key("minecraft:test"));
    assert_eq!(
        state.random_broadcasts[0].sequence.as_deref(),
        Some("minecraft:test")
    );

    let before_reset = state
        .random_sequences
        .sequences
        .get("minecraft:test")
        .cloned()
        .unwrap();
    let reset = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "random reset minecraft:test 42 false true",
    )
    .unwrap();
    assert_eq!(reset.success_count, 1);
    assert_eq!(reset.feedback_key, "commands.random.reset.success");
    assert_eq!(state.random_sequences.sequences.len(), 1);
    assert_ne!(
        state
            .random_sequences
            .sequences
            .get("minecraft:test")
            .unwrap(),
        &before_reset
    );

    let reset_all = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "random reset * 7 true false",
    )
    .unwrap();
    assert_eq!(reset_all.success_count, 1);
    assert!(state.random_sequences.sequences.is_empty());
    assert_eq!(
        state.random_seed_defaults,
        super::RandomSeedDefaults {
            salt: 7,
            include_world_seed: true,
            include_sequence_id: false,
        }
    );
}

#[test]
fn recipe_command_gives_and_takes_single_or_all_recipes() {
    let mut state = ServerCommandState {
        known_recipes: vec![
            "minecraft:planks".to_string(),
            "minecraft:stick".to_string(),
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("recipe"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "recipe give Steve minecraft:planks"
        ),
        Err(CommandError::PermissionDenied)
    );

    let given = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "recipe give Steve minecraft:planks",
    )
    .unwrap();
    assert_eq!(given.success_count, 1);
    assert_eq!(given.feedback_key, "commands.recipe.give.success.single");
    assert_eq!(
        state.player_recipes,
        vec![PlayerRecipeBook {
            player: NameAndId::create_offline("Steve"),
            recipes: vec!["minecraft:planks".to_string()],
        }]
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe give Steve minecraft:planks"
        ),
        Err(CommandError::RecipeGiveFailed)
    );

    let all = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "recipe give Steve,Alex *",
    )
    .unwrap();
    assert_eq!(all.success_count, 3);
    assert_eq!(all.feedback_key, "commands.recipe.give.success.multiple");

    let taken = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "recipe take Steve minecraft:stick",
    )
    .unwrap();
    assert_eq!(taken.success_count, 1);
    assert_eq!(taken.feedback_key, "commands.recipe.take.success.single");

    let taken_all = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "recipe take Steve,Alex *",
    )
    .unwrap();
    assert_eq!(taken_all.success_count, 3);
    assert_eq!(
        taken_all.feedback_key,
        "commands.recipe.take.success.multiple"
    );
}

#[test]
fn recipe_command_fails_when_no_recipes_change() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe give Steve *"
        ),
        Err(CommandError::RecipeGiveFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "recipe take Steve minecraft:stick"
        ),
        Err(CommandError::RecipeTakeFailed)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "recipe"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn list_command_reports_online_player_count_for_all_sources() {
    let mut state = ServerCommandState {
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        max_players: 40,
        ..ServerCommandState::default()
    };
    assert_eq!(command_required_permission("list"), PermissionLevel::All);

    let list = execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(list.success_count, 2);
    assert_eq!(list.feedback_key, "commands.list.players");
    assert!(!list.broadcast_to_admins);

    let list_uuids =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list uuids").unwrap();
    assert_eq!(list_uuids.success_count, 2);
    assert_eq!(list_uuids.feedback_key, "commands.list.players");
}
