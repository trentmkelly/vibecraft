use super::*;

#[test]
fn return_command_requires_gamemaster_and_records_success_or_failure() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("return"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::MODERATOR, "return 7"),
        Err(CommandError::PermissionDenied)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "return 7")
            .unwrap();
    assert_eq!(result.success_count, 7);
    assert_eq!(result.feedback_key, "commands.return.success");
    assert_eq!(
        state.return_events[0],
        ReturnCommandEvent::Success {
            value: 7,
            discard_frame: true,
        }
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "return fail",
    )
    .unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, "commands.return.fail");
    assert_eq!(
        state.return_events[1],
        ReturnCommandEvent::Failure {
            discard_frame: true,
        }
    );
}

#[test]
fn return_command_records_forwarded_command_and_rejects_invalid_syntax() {
    let mut state = ServerCommandState::default();
    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "return run say hello from function",
    )
    .unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.feedback_key, "commands.return.run");
    assert_eq!(
        state.return_events,
        vec![ReturnCommandEvent::Run {
            command: "say hello from function".to_string(),
            discard_frame: true,
        }]
    );

    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "return"),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "return run"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "return nope"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn ride_command_requires_gamemaster_and_mounts_then_dismounts() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("ride"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "ride pig mount boat"
        ),
        Err(CommandError::PermissionDenied)
    );

    let mounted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "ride pig mount boat",
    )
    .unwrap();
    assert_eq!(mounted.success_count, 1);
    assert_eq!(mounted.feedback_key, "commands.ride.mount.success");
    assert_eq!(
        state.entity_mounts,
        vec![EntityMount {
            target: EntityRef {
                id: "pig".to_string(),
                display_name: "pig".to_string(),
            },
            vehicle: EntityRef {
                id: "boat".to_string(),
                display_name: "boat".to_string(),
            },
        }]
    );
    assert_eq!(
        state.ride_events[0],
        RideCommandEvent::Mount {
            target: EntityRef {
                id: "pig".to_string(),
                display_name: "pig".to_string(),
            },
            vehicle: EntityRef {
                id: "boat".to_string(),
                display_name: "boat".to_string(),
            },
        }
    );

    let dismounted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "ride pig dismount",
    )
    .unwrap();
    assert_eq!(dismounted.success_count, 1);
    assert_eq!(dismounted.feedback_key, "commands.ride.dismount.success");
    assert!(state.entity_mounts.is_empty());
    assert!(matches!(
        state.ride_events[1],
        RideCommandEvent::Dismount { .. }
    ));
}

#[test]
fn ride_command_rejects_vanilla_mount_failures() {
    let mut state = ServerCommandState {
        online_players: vec![NameAndId::create_offline("Steve")],
        entity_states: vec![
            EntityState {
                entity: EntityRef {
                    id: "pig".to_string(),
                    display_name: "pig".to_string(),
                },
                kind: EntityKind::Generic,
                dimension: "minecraft:overworld".to_string(),
            },
            EntityState {
                entity: EntityRef {
                    id: "strider".to_string(),
                    display_name: "strider".to_string(),
                },
                kind: EntityKind::Generic,
                dimension: "minecraft:the_nether".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig dismount"
        ),
        Err(CommandError::RideNotRiding)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount Steve"
        ),
        Err(CommandError::RideMountingPlayer)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount pig"
        ),
        Err(CommandError::RideMountingLoop)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount strider"
        ),
        Err(CommandError::RideWrongDimension)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "ride pig mount boat",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "ride pig mount minecart"
        ),
        Err(CommandError::RideAlreadyRiding)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "ride pig"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn spectate_command_updates_camera_for_spectators() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        online_players: vec![steve.clone(), alex.clone()],
        player_game_modes: vec![
            PlayerGameMode {
                player: steve.clone(),
                gamemode: GameMode::Spectator,
            },
            PlayerGameMode {
                player: alex.clone(),
                gamemode: GameMode::Spectator,
            },
        ],
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("spectate"),
        PermissionLevel::Gamemasters
    );

    let started = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spectate cow",
    )
    .unwrap();
    assert_eq!(started.success_count, 1);
    assert_eq!(started.feedback_key, "commands.spectate.success.started");
    assert_eq!(
        state.camera_targets[0].target,
        Some(EntityRef {
            id: "cow".to_string(),
            display_name: "cow".to_string(),
        })
    );

    let explicit = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "spectate pig Alex",
    )
    .unwrap();
    assert_eq!(explicit.feedback_key, "commands.spectate.success.started");
    assert_eq!(state.camera_targets[1].player, alex);
    assert_eq!(
        state.camera_targets[1].target,
        Some(EntityRef {
            id: "pig".to_string(),
            display_name: "pig".to_string(),
        })
    );

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "spectate")
            .unwrap();
    assert_eq!(stopped.feedback_key, "commands.spectate.success.stopped");
    assert_eq!(state.camera_targets[0].target, None);
}

#[test]
fn spectate_command_rejects_vanilla_failures() {
    let steve = NameAndId::create_offline("Steve");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        player_game_modes: vec![PlayerGameMode {
            player: steve.clone(),
            gamemode: GameMode::Spectator,
        }],
        untrackable_entities: vec![EntityRef {
            id: "marker".to_string(),
            display_name: "marker".to_string(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spectate Steve"
        ),
        Err(CommandError::SpectateSelf)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spectate marker"
        ),
        Err(CommandError::SpectateCannotSpectate)
    );

    state.player_game_modes.clear();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "spectate pig"
        ),
        Err(CommandError::SpectateNotSpectator)
    );
    state.command_source_player = None;
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "spectate"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn summon_command_records_entity_spawn_with_defaults_position_and_nbt() {
    let mut state = ServerCommandState {
        command_source_position: Vec3 {
            x: 1.25,
            y: 64.0,
            z: -2.5,
        },
        command_source_dimension: "minecraft:the_nether".to_string(),
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("summon"),
        PermissionLevel::Gamemasters
    );

    let defaulted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "summon pig",
    )
    .unwrap();
    assert_eq!(defaulted.success_count, 1);
    assert_eq!(defaulted.feedback_key, "commands.summon.success");
    assert!(defaulted.broadcast_to_admins);
    assert_eq!(state.summoned_entities[0].entity_type, "minecraft:pig");
    assert_eq!(
        state.summoned_entities[0].position,
        state.command_source_position
    );
    assert!(state.summoned_entities[0].finalized_spawn);
    assert_eq!(
        state.entity_states[0].dimension,
        "minecraft:the_nether".to_string()
    );

    let with_nbt = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "summon minecraft:cow 4.5 70 -8 {NoAI:1b}",
    )
    .unwrap();
    assert_eq!(with_nbt.feedback_key, "commands.summon.success");
    assert_eq!(state.summoned_entities[1].entity_type, "minecraft:cow");
    assert_eq!(
        state.summoned_entities[1].position,
        Vec3 {
            x: 4.5,
            y: 70.0,
            z: -8.0,
        }
    );
    assert_eq!(
        state.summoned_entities[1].nbt,
        Some("{NoAI:1b}".to_string())
    );
    assert!(!state.summoned_entities[1].finalized_spawn);
}

#[test]
fn summon_command_rejects_invalid_position_duplicate_uuid_and_syntax() {
    let mut state = ServerCommandState::default();

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "summon pig 30000000 64 0"
        ),
        Err(CommandError::SummonInvalidPosition)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "summon pig 0 20000000 0"
        ),
        Err(CommandError::SummonInvalidPosition)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "summon NotValid 0 64 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "summon"),
        Err(CommandError::InvalidSyntax)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "summon pig 0 64 0 {UUID:fixed-id}",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "summon cow 1 64 1 {UUID:fixed-id}"
        ),
        Err(CommandError::SummonDuplicateUuid)
    );
}

#[test]
fn team_command_manages_teams_and_memberships() {
    let steve = NameAndId::create_offline("Steve");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        ..ServerCommandState::default()
    };

    assert_eq!(
        command_required_permission("team"),
        PermissionLevel::Gamemasters
    );

    let add = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team add red",
    )
    .unwrap();
    assert_eq!(add.success_count, 1);
    assert_eq!(add.feedback_key, "commands.team.add.success");
    assert_eq!(state.teams[0].display_name, "red");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "team add red"
        ),
        Err(CommandError::TeamAlreadyExists)
    );

    let joined = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team join red Steve Alex",
    )
    .unwrap();
    assert_eq!(joined.success_count, 2);
    assert_eq!(joined.feedback_key, "commands.team.join.success.multiple");
    assert_eq!(state.player_teams.len(), 2);

    let listed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team list red",
    )
    .unwrap();
    assert_eq!(listed.success_count, 2);
    assert_eq!(listed.feedback_key, "commands.team.list.members.success");

    let left = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team leave Alex",
    )
    .unwrap();
    assert_eq!(left.feedback_key, "commands.team.leave.success.single");
    assert_eq!(state.player_teams.len(), 1);

    let emptied = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team empty red",
    )
    .unwrap();
    assert_eq!(emptied.success_count, 1);
    assert!(state.player_teams.is_empty());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "team empty red"
        ),
        Err(CommandError::TeamAlreadyEmpty)
    );

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team remove red",
    )
    .unwrap();
    assert_eq!(removed.feedback_key, "commands.team.remove.success");
    assert!(state.teams.is_empty());
}

#[test]
fn team_command_modifies_options_and_rejects_unchanged_values() {
    let mut state = ServerCommandState {
        teams: vec![TeamState::new("red".to_string(), "Red Team".to_string())],
        ..ServerCommandState::default()
    };

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red color blue",
    )
    .unwrap();
    assert_eq!(state.teams[0].color, "blue");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "team modify red color blue"
        ),
        Err(CommandError::TeamOptionUnchanged)
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red friendlyFire false",
    )
    .unwrap();
    assert!(!state.teams[0].friendly_fire);
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red seeFriendlyInvisibles false",
    )
    .unwrap();
    assert_eq!(state.teams[0].packed_options(), 0);
    state.teams[0].apply_packed_options(3);
    assert!(state.teams[0].friendly_fire);
    assert!(state.teams[0].see_friendly_invisibles);
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red nametagVisibility never",
    )
    .unwrap();
    assert_eq!(state.teams[0].nametag_visibility, "never");
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red deathMessageVisibility hideForOtherTeams",
    )
    .unwrap();
    assert_eq!(state.teams[0].death_message_visibility, "hideForOtherTeams");
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red collisionRule pushOwnTeam",
    )
    .unwrap();
    assert_eq!(state.teams[0].collision_rule, "pushOwnTeam");
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red prefix <",
    )
    .unwrap();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "team modify red suffix >",
    )
    .unwrap();
    assert_eq!(
        state.teams[0].formatted_member_name("Steve"),
        "blue:<Steve>"
    );
    assert_eq!(state.teams[0].formatted_display_name(), "blue:[Red Team]");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "team modify red collisionRule bad"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "team list blue"
        ),
        Err(CommandError::TeamNotFound)
    );
}

#[test]
fn team_runtime_rules_apply_alliance_visibility_collision_and_friendly_fire() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let blake = NameAndId::create_offline("Blake");
    let teams = vec![TeamState {
        name: "red".to_string(),
        display_name: "Red".to_string(),
        color: "red".to_string(),
        friendly_fire: false,
        see_friendly_invisibles: true,
        nametag_visibility: "hideForOtherTeams".to_string(),
        death_message_visibility: "hideForOwnTeam".to_string(),
        collision_rule: "pushOtherTeams".to_string(),
        prefix: String::new(),
        suffix: String::new(),
    }];
    let memberships = vec![
        TeamMembership {
            player: steve.clone(),
            team: "red".to_string(),
        },
        TeamMembership {
            player: alex.clone(),
            team: "red".to_string(),
        },
    ];

    assert_eq!(
        player_team(&teams, &memberships, &steve).map(|team| team.name.as_str()),
        Some("red")
    );
    assert!(players_allied(&memberships, &steve, &alex));
    assert!(!players_allied(&memberships, &steve, &blake));
    assert!(!team_allows_friendly_damage(
        &teams,
        &memberships,
        &steve,
        &alex
    ));
    assert!(team_allows_friendly_damage(
        &teams,
        &memberships,
        &steve,
        &blake
    ));
    assert!(!team_allows_collision(&teams, &memberships, &steve, &alex));
    assert!(team_allows_collision(&teams, &memberships, &steve, &blake));
    assert!(team_allows_visibility(
        &teams,
        &memberships,
        &alex,
        &steve,
        false
    ));
    assert!(!team_allows_visibility(
        &teams,
        &memberships,
        &blake,
        &steve,
        false
    ));
    assert!(!team_allows_visibility(
        &teams,
        &memberships,
        &alex,
        &steve,
        true
    ));
    assert!(team_allows_visibility(
        &teams,
        &memberships,
        &blake,
        &steve,
        true
    ));
}

#[test]
fn advancement_grant_revoke_modes_walk_parent_child_tree() {
    let mut state = ServerCommandState {
        advancements: vec![
            AdvancementDefinition {
                id: "minecraft:story/root".to_string(),
                parent: None,
                criteria: vec!["tick".to_string()],
            },
            AdvancementDefinition {
                id: "minecraft:story/mine_stone".to_string(),
                parent: Some("minecraft:story/root".to_string()),
                criteria: vec!["stone".to_string()],
            },
            AdvancementDefinition {
                id: "minecraft:story/iron_tools".to_string(),
                parent: Some("minecraft:story/mine_stone".to_string()),
                criteria: vec!["iron".to_string()],
            },
        ],
        ..ServerCommandState::default()
    };

    let granted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "advancement grant Steve through minecraft:story/mine_stone",
    )
    .unwrap();
    assert_eq!(granted.success_count, 3);
    assert_eq!(
        granted.feedback_key,
        "commands.advancement.grant.many.to.one.success"
    );
    let player = NameAndId::create_offline("Steve");
    assert!(state.player_advancements.iter().any(|progress| {
        progress.player.uuid == player.uuid
            && progress.advancement == "minecraft:story/root"
            && progress.completed_criteria == vec!["tick".to_string()]
    }));

    let revoked = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "advancement revoke Steve from minecraft:story/mine_stone",
    )
    .unwrap();
    assert_eq!(revoked.success_count, 2);
    assert!(state.player_advancements.iter().any(|progress| {
        progress.advancement == "minecraft:story/root"
            && progress.completed_criteria == vec!["tick".to_string()]
    }));
}

#[test]
fn advancement_everything_and_criterion_paths_match_vanilla_outcomes() {
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        advancements: vec![AdvancementDefinition {
            id: "minecraft:adventure/root".to_string(),
            parent: None,
            criteria: vec!["a".to_string(), "b".to_string()],
        }],
        player_advancements: vec![PlayerAdvancementProgress {
            player: alex.clone(),
            advancement: "minecraft:adventure/root".to_string(),
            completed_criteria: vec!["a".to_string()],
        }],
        ..ServerCommandState::default()
    };

    let criterion = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "advancement grant Alex only minecraft:adventure/root b",
    )
    .unwrap();
    assert_eq!(criterion.success_count, 1);
    assert_eq!(
        criterion.feedback_key,
        "commands.advancement.grant.criterion.to.one.success"
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "advancement grant Alex only minecraft:adventure/root missing"
        ),
        Err(CommandError::AdvancementCriterionNotFound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "advancement grant Alex only minecraft:adventure/root b"
        ),
        Err(CommandError::AdvancementNoAction)
    );

    let revoked = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "advancement revoke Alex everything",
    )
    .unwrap();
    assert_eq!(revoked.success_count, 1);
    assert!(!revoked.broadcast_to_admins);
    assert!(state.player_advancements[0].completed_criteria.is_empty());
}

#[test]
fn attribute_command_gets_sets_resets_and_computes_modifier_values() {
    let mut state = ServerCommandState {
        entity_attributes: vec![EntityAttributeState {
            target: "Steve".to_string(),
            attribute: "minecraft:max_health".to_string(),
            default_base: 20.0,
            base: 20.0,
            modifiers: vec![AttributeModifierState {
                id: "minecraft:bonus".to_string(),
                value: 2.0,
                operation: AttributeOperation::Value,
            }],
        }],
        ..ServerCommandState::default()
    };

    let value = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Steve minecraft:max_health get 10",
    )
    .unwrap();
    assert_eq!(value.success_count, 220);
    assert_eq!(value.feedback_key, "commands.attribute.value.get.success");

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Steve minecraft:max_health base set 30",
    )
    .unwrap();
    assert_eq!(state.entity_attributes[0].base, 30.0);
    let base = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Steve minecraft:max_health base get",
    )
    .unwrap();
    assert_eq!(base.success_count, 30);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Steve minecraft:max_health base reset",
    )
    .unwrap();
    assert_eq!(state.entity_attributes[0].base, 20.0);
}

#[test]
fn attribute_command_adds_removes_and_reports_modifier_failures() {
    let mut state = ServerCommandState {
        entity_attributes: vec![EntityAttributeState {
            target: "Alex".to_string(),
            attribute: "minecraft:movement_speed".to_string(),
            default_base: 0.1,
            base: 0.1,
            modifiers: Vec::new(),
        }],
        ..ServerCommandState::default()
    };

    execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute Alex minecraft:movement_speed modifier add minecraft:sprint 0.2 add_multiplied_total",
        )
        .unwrap();
    assert_eq!(
        state.entity_attributes[0].modifiers[0].operation,
        AttributeOperation::MultipliedTotal
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute Alex minecraft:movement_speed modifier add minecraft:sprint 0.2 add_value",
        ),
        Err(CommandError::AttributeModifierAlreadyPresent)
    );
    let modifier = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Alex minecraft:movement_speed modifier value get minecraft:sprint 1000",
    )
    .unwrap();
    assert_eq!(modifier.success_count, 200);
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "attribute Alex minecraft:movement_speed modifier remove minecraft:sprint",
    )
    .unwrap();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute Alex minecraft:movement_speed modifier remove minecraft:sprint",
        ),
        Err(CommandError::AttributeNoSuchModifier)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute Alex minecraft:attack_damage get",
        ),
        Err(CommandError::AttributeNoSuchAttribute)
    );
}
