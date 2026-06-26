use super::*;

#[test]
fn dialog_command_shows_and_clears_dialog_packets_for_players() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("dialog"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "dialog show Steve minecraft:welcome"
        ),
        Err(CommandError::PermissionDenied)
    );

    let shown = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "dialog show Steve Alex minecraft:welcome",
    )
    .unwrap();
    assert_eq!(shown.success_count, 2);
    assert_eq!(shown.feedback_key, "commands.dialog.show.multiple");
    assert!(shown.broadcast_to_admins);
    assert_eq!(
        state.dialog_events[0],
        DialogCommandEvent::Show {
            targets: vec![
                NameAndId::create_offline("Steve"),
                NameAndId::create_offline("Alex"),
            ],
            dialog: "minecraft:welcome".to_string(),
        }
    );

    let cleared = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "dialog clear Steve",
    )
    .unwrap();
    assert_eq!(cleared.success_count, 1);
    assert_eq!(cleared.feedback_key, "commands.dialog.clear.single");
    assert_eq!(
        state.dialog_events[1],
        DialogCommandEvent::Clear {
            targets: vec![NameAndId::create_offline("Steve")],
        }
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "dialog show Steve"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn effect_command_gives_default_timed_infinite_and_instant_effects() {
    let mut state = ServerCommandState {
        entity_states: vec![EntityState {
            entity: EntityRef {
                id: "armor_stand".to_string(),
                display_name: "Armor Stand".to_string(),
            },
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("effect"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "effect give Steve speed"
        ),
        Err(CommandError::PermissionDenied)
    );

    let default = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve speed",
    )
    .unwrap();
    assert_eq!(default.success_count, 1);
    assert_eq!(default.feedback_key, "commands.effect.give.success.single");
    assert_eq!(
        state.active_effects[0],
        ActiveEffect {
            target: EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            },
            effect: "minecraft:speed".to_string(),
            duration_ticks: 600,
            amplifier: 0,
            show_particles: true,
        }
    );

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve,Alex strength 5 2 true",
    )
    .unwrap();
    assert!(state.active_effects.iter().any(|effect| {
        effect.target.id == "Alex"
            && effect.effect == "minecraft:strength"
            && effect.duration_ticks == 100
            && effect.amplifier == 2
            && !effect.show_particles
    }));

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve regeneration infinite 1 false",
    )
    .unwrap();
    assert!(state.active_effects.iter().any(|effect| {
        effect.target.id == "Steve"
            && effect.effect == "minecraft:regeneration"
            && effect.duration_ticks == -1
            && effect.amplifier == 1
            && effect.show_particles
    }));

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect give Steve instant_health",
    )
    .unwrap();
    assert!(state.active_effects.iter().any(|effect| {
        effect.target.id == "Steve"
            && effect.effect == "minecraft:instant_health"
            && effect.duration_ticks == 1
    }));
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "effect give armor_stand speed"
        ),
        Err(CommandError::EffectGiveFailed)
    );
}

#[test]
fn effect_command_clears_all_or_specific_effects_and_reports_failures() {
    let mut state = ServerCommandState {
        command_source_entity: Some(EntityRef {
            id: "Steve".to_string(),
            display_name: "Steve".to_string(),
        }),
        active_effects: vec![
            ActiveEffect {
                target: super::entity_ref("Steve"),
                effect: "minecraft:speed".to_string(),
                duration_ticks: 600,
                amplifier: 0,
                show_particles: true,
            },
            ActiveEffect {
                target: super::entity_ref("Alex"),
                effect: "minecraft:strength".to_string(),
                duration_ticks: 100,
                amplifier: 0,
                show_particles: true,
            },
        ],
        ..ServerCommandState::default()
    };

    let clear_specific = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect clear Alex strength",
    )
    .unwrap();
    assert_eq!(clear_specific.success_count, 1);
    assert_eq!(
        clear_specific.feedback_key,
        "commands.effect.clear.specific.success.single"
    );
    assert_eq!(
        state.active_effects,
        vec![ActiveEffect {
            target: super::entity_ref("Steve"),
            effect: "minecraft:speed".to_string(),
            duration_ticks: 600,
            amplifier: 0,
            show_particles: true,
        }]
    );

    let clear_source = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "effect clear",
    )
    .unwrap();
    assert_eq!(clear_source.success_count, 1);
    assert_eq!(
        clear_source.feedback_key,
        "commands.effect.clear.everything.success.single"
    );
    assert!(state.active_effects.is_empty());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "effect clear Steve"
        ),
        Err(CommandError::EffectClearEverythingFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "effect clear Alex strength"
        ),
        Err(CommandError::EffectClearSpecificFailed)
    );
}

#[test]
fn enchant_command_applies_compatible_held_item_enchantments() {
    let mut state = ServerCommandState {
        player_inventories: vec![
            CommandPlayerInventory {
                player: NameAndId::create_offline("Steve"),
                items: vec![CommandItemStack {
                    item: "minecraft:diamond_sword".to_string(),
                    count: 1,
                }],
            },
            CommandPlayerInventory {
                player: NameAndId::create_offline("Alex"),
                items: vec![CommandItemStack {
                    item: "minecraft:diamond_pickaxe".to_string(),
                    count: 1,
                }],
            },
        ],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("enchant"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "enchant Steve sharpness"
        ),
        Err(CommandError::PermissionDenied)
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "enchant Steve sharpness 5",
    )
    .unwrap();
    assert_eq!(result.success_count, 1);
    assert_eq!(result.feedback_key, "commands.enchant.success.single");
    assert_eq!(
        state.item_enchantments,
        vec![CommandItemEnchantment {
            target: EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            },
            item: "minecraft:diamond_sword".to_string(),
            enchantment: "minecraft:sharpness".to_string(),
            level: 5,
        }]
    );

    let multi = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "enchant Steve,Alex fortune 3",
    )
    .unwrap();
    assert_eq!(multi.success_count, 1);
    assert_eq!(multi.feedback_key, "commands.enchant.success.multiple");
    assert!(state.item_enchantments.iter().any(|enchantment| {
        enchantment.target.id == "Alex"
            && enchantment.item == "minecraft:diamond_pickaxe"
            && enchantment.enchantment == "minecraft:fortune"
            && enchantment.level == 3
    }));
}

#[test]
fn enchant_command_reports_level_item_entity_and_compatibility_failures() {
    let mut state = ServerCommandState {
        entity_states: vec![EntityState {
            entity: EntityRef {
                id: "armor_stand".to_string(),
                display_name: "Armor Stand".to_string(),
            },
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        player_inventories: vec![
            CommandPlayerInventory {
                player: NameAndId::create_offline("Steve"),
                items: vec![CommandItemStack {
                    item: "minecraft:diamond_sword".to_string(),
                    count: 1,
                }],
            },
            CommandPlayerInventory {
                player: NameAndId::create_offline("Alex"),
                items: Vec::new(),
            },
        ],
        item_enchantments: vec![CommandItemEnchantment {
            target: EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            },
            item: "minecraft:diamond_sword".to_string(),
            enchantment: "minecraft:sharpness".to_string(),
            level: 5,
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant Steve sharpness 6"
        ),
        Err(CommandError::EnchantLevelTooHigh)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant armor_stand sharpness"
        ),
        Err(CommandError::EnchantNotLivingEntity)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant Alex sharpness"
        ),
        Err(CommandError::EnchantNoItem)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant Steve fortune"
        ),
        Err(CommandError::EnchantIncompatible)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "enchant Alex,armor_stand sharpness"
        ),
        Err(CommandError::EnchantFailed)
    );
}

#[test]
fn execute_command_runs_nested_command_with_derived_sources() {
    let mut state = ServerCommandState {
        online_players: vec![
            NameAndId::create_offline("Steve"),
            NameAndId::create_offline("Alex"),
        ],
        command_source_position: Vec3 {
            x: 1.0,
            y: 64.0,
            z: 1.0,
        },
        command_source_dimension: "minecraft:overworld".to_string(),
        entity_positions: vec![EntityPosition {
            entity: EntityRef {
                id: "Alex".to_string(),
                display_name: "Alex".to_string(),
            },
            dimension: "minecraft:the_nether".to_string(),
            position: Vec3 {
                x: 8.0,
                y: 70.0,
                z: -3.0,
            },
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("execute"),
        PermissionLevel::Gamemasters
    );

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "execute as Steve,Alex positioned 4 65 9 run say hello",
    )
    .unwrap();

    assert_eq!(result.success_count, 2);
    assert_eq!(state.chat_events.len(), 2);
    assert_eq!(
        state
            .chat_events
            .iter()
            .map(|event| event.sender.as_ref().unwrap().name.as_str())
            .collect::<Vec<_>>(),
        vec!["Steve", "Alex"]
    );
    assert_eq!(state.execute_events.len(), 1);
    assert_eq!(state.execute_events[0].command, "say hello");
    assert_eq!(state.execute_events[0].result, 2);
    assert!(state.execute_events[0].success);
    assert_eq!(state.execute_events[0].sources.len(), 2);
    assert_eq!(
        state.execute_events[0].sources[0],
        ExecuteSourceSnapshot {
            entity: Some(EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            }),
            position: Vec3 {
                x: 4.0,
                y: 65.0,
                z: 9.0,
            },
            dimension: "minecraft:overworld".to_string(),
            anchor: EntityAnchor::Feet,
        }
    );
    assert_eq!(state.command_source_entity, None);
    assert_eq!(state.command_source_position.x, 1.0);
    assert_eq!(state.command_source_dimension, "minecraft:overworld");

    let at = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "execute at Alex anchored eyes run particle minecraft:dust 0 70 0 0 0 0 0 1",
    )
    .unwrap();
    assert_eq!(at.success_count, 2);
    assert_eq!(
        state.execute_events.last().unwrap().sources[0],
        ExecuteSourceSnapshot {
            entity: Some(EntityRef {
                id: "Alex".to_string(),
                display_name: "Alex".to_string(),
            }),
            position: Vec3 {
                x: 8.0,
                y: 70.0,
                z: -3.0,
            },
            dimension: "minecraft:the_nether".to_string(),
            anchor: EntityAnchor::Eyes,
        }
    );
}

#[test]
fn execute_command_applies_dimension_and_conditions() {
    let mut state = ServerCommandState {
        blocks: vec![BlockStateEntry {
            dimension: "minecraft:the_nether".to_string(),
            position: BlockPos { x: 1, y: 2, z: 3 },
            block: "minecraft:gold_block".to_string(),
        }],
        ..ServerCommandState::default()
    };

    let success = execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "execute in minecraft:the_nether if block 1 2 3 gold_block run setblock 4 5 6 diamond_block",
        )
        .unwrap();
    assert_eq!(success.success_count, 1);
    assert!(state.blocks.iter().any(|entry| {
        entry.dimension == "minecraft:the_nether"
            && entry.position == BlockPos { x: 4, y: 5, z: 6 }
            && entry.block == "minecraft:diamond_block"
    }));

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "execute unless entity Steve run say hidden"
        ),
        Err(CommandError::ExecuteConditionFailed)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "execute run say denied"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "execute if block 1 2 3 diamond_block run say no"
        ),
        Err(CommandError::ExecuteConditionFailed)
    );
}

#[test]
fn experience_command_adds_sets_queries_points_and_levels() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("experience"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "experience add Steve 7"
        ),
        Err(CommandError::PermissionDenied)
    );

    let added = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience add Steve 16 points",
    )
    .unwrap();
    assert_eq!(added.success_count, 1);
    assert_eq!(
        added.feedback_key,
        "commands.experience.add.points.success.single"
    );
    assert_eq!(
        state.player_experience[0].player,
        NameAndId::create_offline("Steve")
    );
    assert_eq!(state.player_experience[0].level, 2);
    assert!(state.player_experience[0].progress.abs() < 0.0001);
    assert_eq!(state.player_experience[0].total, 16);

    let levels = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "xp add Steve,Alex 3 levels",
    )
    .unwrap();
    assert_eq!(levels.success_count, 2);
    assert_eq!(
        levels.feedback_key,
        "commands.experience.add.levels.success.multiple"
    );
    assert_eq!(state.player_experience[0].level, 5);
    assert_eq!(
        state
            .player_experience
            .iter()
            .find(|xp| xp.player.name == "Alex")
            .unwrap()
            .level,
        3
    );

    let set_points = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience set Steve 5 points",
    )
    .unwrap();
    assert_eq!(
        set_points.feedback_key,
        "commands.experience.set.points.success.single"
    );
    let queried_points = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience query Steve points",
    )
    .unwrap();
    assert_eq!(queried_points.success_count, 5);
    assert_eq!(
        queried_points.feedback_key,
        "commands.experience.query.points"
    );

    let queried_levels = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience query Steve levels",
    )
    .unwrap();
    assert_eq!(queried_levels.success_count, 5);
    assert_eq!(
        queried_levels.feedback_key,
        "commands.experience.query.levels"
    );
}

#[test]
fn experience_command_defaults_to_points_and_query_requires_single_player() {
    let mut state = ServerCommandState {
        player_experience: vec![PlayerExperienceState {
            player: NameAndId::create_offline("Steve"),
            level: 5,
            progress: 0.0,
            total: 0,
        }],
        ..ServerCommandState::default()
    };

    let default_set_points = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience set Steve 6",
    )
    .unwrap();
    assert_eq!(default_set_points.success_count, 1);
    assert_eq!(
        default_set_points.feedback_key,
        "commands.experience.set.points.success.single"
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "experience query Steve,Alex points"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn experience_command_rejects_invalid_set_points_and_clamps_negative_levels() {
    let mut state = ServerCommandState {
        player_experience: vec![PlayerExperienceState {
            player: NameAndId::create_offline("Steve"),
            level: 1,
            progress: 0.5,
            total: 10,
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "experience set Steve 9 points"
        ),
        Err(CommandError::ExperienceSetPointsInvalid)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "experience set Steve -1 levels"
        ),
        Err(CommandError::InvalidSyntax)
    );

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience add Steve -20 points",
    )
    .unwrap();
    assert_eq!(removed.success_count, 1);
    assert_eq!(state.player_experience[0].level, 0);
    assert_eq!(state.player_experience[0].progress, 0.0);
    assert_eq!(state.player_experience[0].total, 0);

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience set Steve 30 levels",
    )
    .unwrap();
    assert_eq!(state.player_experience[0].level, 30);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "experience set Steve 112 points"
        ),
        Err(CommandError::ExperienceSetPointsInvalid)
    );
}

#[test]
fn experience_command_set_points_matches_java_partial_success() {
    let mut state = ServerCommandState {
        player_experience: vec![
            PlayerExperienceState {
                player: NameAndId::create_offline("Steve"),
                level: 0,
                progress: 0.0,
                total: 0,
            },
            PlayerExperienceState {
                player: NameAndId::create_offline("Alex"),
                level: 30,
                progress: 0.0,
                total: 0,
            },
        ],
        ..ServerCommandState::default()
    };

    let result = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "experience set Steve,Alex 50 points",
    )
    .unwrap();
    assert_eq!(result.success_count, 2);
    assert_eq!(
        result.feedback_key,
        "commands.experience.set.points.success.multiple"
    );
    let steve = state
        .player_experience
        .iter()
        .find(|xp| xp.player.name == "Steve")
        .unwrap();
    assert_eq!(steve.progress, 0.0);
    let alex = state
        .player_experience
        .iter()
        .find(|xp| xp.player.name == "Alex")
        .unwrap();
    assert!((alex.progress - (50.0 / 112.0)).abs() < 0.0001);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn experience_command_source_matches_java_26_1_2() {
    const EXPERIENCE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ExperienceCommand.java");

    for sentinel in [
        "Commands.literal(\"experience\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"add\")",
        "Commands.argument(\"target\", EntityArgument.players())",
        "Commands.argument(\"amount\", IntegerArgumentType.integer())",
        "ExperienceCommand.Type.POINTS",
        "ExperienceCommand.Type.LEVELS",
        "Commands.literal(\"set\")",
        "Commands.argument(\"amount\", IntegerArgumentType.integer(0))",
        "Commands.literal(\"query\")",
        "Commands.argument(\"target\", EntityArgument.player())",
        "Commands.literal(\"xp\").requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        ".redirect(command)",
        "source.sendSuccess(() -> Component.translatable(\"commands.experience.query.\" + type.name",
        "source.sendSuccess(\n            () -> Component.translatable(\"commands.experience.add.\" + type.name + \".success.single\"",
        "throw ERROR_SET_POINTS_INVALID.create();",
        "return players.size();",
        "POINTS(\"points\", Player::giveExperiencePoints",
        "if (a >= p.getXpNeededForNextLevel())",
        "p.setExperiencePoints(a);",
        "LEVELS(\"levels\", ServerPlayer::giveExperienceLevels",
        "p.setExperienceLevels(a);",
        "Mth.floor(p.experienceProgress * p.getXpNeededForNextLevel())",
    ] {
        assert!(
            EXPERIENCE_COMMAND_JAVA.contains(sentinel),
            "ExperienceCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn fetchprofile_command_resolves_name_id_and_avatar_entity_profiles() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mannequin_profile = NameAndId::create_offline("DisplayAlex");
    let mannequin = EntityRef {
        id: "mannequin".to_string(),
        display_name: "mannequin".to_string(),
    };
    let mut state = ServerCommandState {
        online_players: vec![steve.clone()],
        whitelisted_players: vec![alex.clone()],
        avatar_profiles: vec![AvatarProfile {
            entity: mannequin.clone(),
            profile: mannequin_profile.clone(),
        }],
        ..ServerCommandState::default()
    };
    assert_eq!(
        command_required_permission("fetchprofile"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "fetchprofile name Steve"
        ),
        Err(CommandError::PermissionDenied)
    );

    let by_name = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fetchprofile name Steve",
    )
    .unwrap();
    assert_eq!(by_name.success_count, 1);
    assert_eq!(by_name.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        state.side_feedback[0],
        CommandResult {
            success_count: 1,
            feedback_key: "commands.fetchprofile.name.success",
            broadcast_to_admins: false,
        }
    );
    assert_eq!(
        state.fetched_profiles[0].query,
        FetchProfileQuery::Name("Steve".to_string())
    );
    assert_eq!(state.fetched_profiles[0].profile, steve);
    assert!(state.fetched_profiles[0]
        .encoded_profile
        .contains("5627dd98-e6be-3c21-b8a8-e92344183641"));
    assert!(state.fetched_profiles[0]
        .encoded_head_component
        .contains("type:\"player\""));

    let by_id = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        &format!("fetchprofile id {}", alex.uuid),
    )
    .unwrap();
    assert_eq!(by_id.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        state.side_feedback[1],
        CommandResult {
            success_count: 1,
            feedback_key: "commands.fetchprofile.id.success",
            broadcast_to_admins: false,
        }
    );
    assert_eq!(
        state.fetched_profiles[1].query,
        FetchProfileQuery::Id(alex.uuid.clone())
    );
    assert_eq!(state.fetched_profiles[1].profile, alex);

    let by_entity = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fetchprofile entity mannequin",
    )
    .unwrap();
    assert_eq!(
        by_entity.feedback_key,
        "commands.fetchprofile.entity.success"
    );
    assert_eq!(
        state.fetched_profiles[2].query,
        FetchProfileQuery::Entity(mannequin)
    );
    assert_eq!(state.fetched_profiles[2].profile, mannequin_profile);
}

#[test]
fn fetchprofile_command_reports_missing_and_invalid_profiles() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fetchprofile id not-a-uuid"
        ),
        Err(CommandError::InvalidSyntax)
    );
    let missing_id = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fetchprofile id 00000000-0000-0000-0000-000000000001",
    )
    .unwrap();
    assert_eq!(missing_id.success_count, 1);
    assert_eq!(missing_id.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        state.side_feedback[0],
        CommandResult {
            success_count: 0,
            feedback_key: "commands.fetchprofile.id.failure",
            broadcast_to_admins: false,
        }
    );
    let missing_name = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "fetchprofile name Missing Player",
    )
    .unwrap();
    assert_eq!(missing_name.success_count, 1);
    assert_eq!(missing_name.feedback_key, NO_COMMAND_FEEDBACK);
    assert_eq!(
        state.side_feedback[1],
        CommandResult {
            success_count: 0,
            feedback_key: "commands.fetchprofile.name.failure",
            broadcast_to_admins: false,
        }
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fetchprofile entity pig"
        ),
        Err(CommandError::FetchProfileNotFound)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "fetchprofile"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn fetchprofile_command_source_matches_java_26_1_2() {
    const FETCH_PROFILE_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/FetchProfileCommand.java");

    for sentinel in [
        "Commands.literal(\"fetchprofile\")",
        ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
        "Commands.literal(\"name\")",
        "Commands.argument(\"name\", StringArgumentType.greedyString())",
        "Commands.literal(\"id\")",
        "Commands.argument(\"id\", UuidArgument.uuid())",
        "Commands.literal(\"entity\")",
        "Commands.argument(\"entity\", EntityArgument.entity())",
        "reportResolvedProfile(source, profile, \"commands.fetchprofile.name.success\", nameComponent)",
        "source.sendFailure(Component.translatable(\"commands.fetchprofile.name.failure\", nameComponent))",
        "return 1;",
        "reportResolvedProfile(source, profile, \"commands.fetchprofile.id.success\", idComponent)",
        "source.sendFailure(Component.translatable(\"commands.fetchprofile.id.failure\", idComponent))",
        "if (entity instanceof Avatar avatar)",
        "throw NO_PROFILE.create(entity.getDisplayName());",
        "reportResolvedProfile(source, \"commands.fetchprofile.entity.success\", avatar.getDisplayName(), avatar.getProfile())",
        "commands.fetchprofile.failed_to_serialize",
    ] {
        assert!(
            FETCH_PROFILE_JAVA.contains(sentinel),
            "FetchProfileCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn tick_rate_command_matches_admin_range_and_feedback() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "tick rate 40"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick rate 0.5"),
        Err(CommandError::InvalidSyntax)
    );

    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick rate 40.5")
            .unwrap();
    assert_eq!(result.success_count, 40);
    assert_eq!(result.feedback_key, "commands.tick.rate.success");
    assert_eq!(state.tick_rate.tick_rate(), 40.5);
}

#[test]
fn tick_freeze_unfreeze_step_and_stop_use_tick_controller() {
    let mut state = ServerCommandState::default();
    let failed_step =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step").unwrap();
    assert_eq!(failed_step.success_count, 1);
    assert_eq!(failed_step.feedback_key, "commands.tick.step.fail");
    assert!(!failed_step.broadcast_to_admins);

    let frozen =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze").unwrap();
    assert!(state.tick_rate.is_frozen());
    assert_eq!(frozen.success_count, 1);
    assert_eq!(frozen.feedback_key, "commands.tick.status.frozen");

    let stepped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step 1s")
            .unwrap();
    assert_eq!(stepped.feedback_key, "commands.tick.step.success");
    assert_eq!(state.tick_rate.frozen_ticks_to_run(), 20);

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step stop")
            .unwrap();
    assert_eq!(stopped.success_count, 1);
    assert_eq!(stopped.feedback_key, "commands.tick.step.stop.success");
    assert_eq!(state.tick_rate.frozen_ticks_to_run(), 0);

    let unfrozen =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick unfreeze")
            .unwrap();
    assert!(!state.tick_rate.is_frozen());
    assert_eq!(unfrozen.success_count, 0);
    assert_eq!(unfrozen.feedback_key, "commands.tick.status.running");
}

#[test]
fn tick_freeze_stops_active_step_and_sprint_like_java_command() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze").unwrap();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick step 5t").unwrap();
    assert_eq!(state.tick_rate.frozen_ticks_to_run(), 5);
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick unfreeze").unwrap();
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 10t").unwrap();
    assert!(state.tick_rate.is_sprinting());

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze").unwrap();

    assert!(state.tick_rate.is_frozen());
    assert_eq!(state.tick_rate.frozen_ticks_to_run(), 0);
    assert!(!state.tick_rate.is_sprinting());
}

#[test]
fn tick_sprint_start_and_stop_use_time_arguments() {
    let mut state = ServerCommandState::default();
    let started =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 3d")
            .unwrap();
    assert!(state.tick_rate.is_sprinting());
    assert_eq!(started.success_count, 1);
    assert_eq!(started.feedback_key, "commands.tick.status.sprinting");

    let stopped = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "tick sprint stop",
    )
    .unwrap();
    assert!(!state.tick_rate.is_sprinting());
    assert_eq!(stopped.success_count, 1);
    assert_eq!(stopped.feedback_key, "commands.tick.sprint.stop.success");

    let stopped_again = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "tick sprint stop",
    )
    .unwrap();
    assert_eq!(stopped_again.success_count, 0);
    assert_eq!(stopped_again.feedback_key, "commands.tick.sprint.stop.fail");
}

#[test]
fn tick_query_reports_sprinting_frozen_lagging_or_running_status() {
    let mut state = ServerCommandState::default();
    let running =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query").unwrap();
    assert_eq!(running.success_count, 20);
    assert_eq!(running.feedback_key, "commands.tick.status.running");
    assert!(!running.broadcast_to_admins);

    state.average_tick_time_nanos = 60_000_000;
    let lagging =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query").unwrap();
    assert_eq!(lagging.feedback_key, "commands.tick.status.lagging");

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick freeze").unwrap();
    let frozen =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query").unwrap();
    assert_eq!(frozen.feedback_key, "commands.tick.status.frozen");

    execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick sprint 1t").unwrap();
    let sprinting =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "tick query").unwrap();
    assert_eq!(sprinting.feedback_key, "commands.tick.status.sprinting");
}
