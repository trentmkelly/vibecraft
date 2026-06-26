use super::*;

#[test]
fn list_command_tracks_login_replacement_and_disconnect_counts() {
    let mut state = ServerCommandState {
        max_players: 40,
        ..ServerCommandState::default()
    };

    let empty = execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(empty.success_count, 0);
    assert_eq!(empty.feedback_key, "commands.list.players");

    state.online_players = vec![NameAndId::create_offline("Steve")];
    let joined = execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(joined.success_count, 1);
    assert_eq!(joined.feedback_key, "commands.list.players");
    assert!(!state.last_list_includes_uuids);

    state.online_players = vec![NameAndId::create_offline("Steve")];
    let replaced =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list uuids").unwrap();
    assert_eq!(replaced.success_count, 1);
    assert_eq!(replaced.feedback_key, "commands.list.players");
    assert!(state.last_list_includes_uuids);

    state.online_players = vec![
        NameAndId::create_offline("Steve"),
        NameAndId::create_offline("Alex"),
    ];
    let two_players =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(two_players.success_count, 2);
    assert!(!state.last_list_includes_uuids);

    state.online_players = vec![NameAndId::create_offline("Alex")];
    let after_disconnect =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(after_disconnect.success_count, 1);
}

#[test]
#[cfg(vibecraft_has_decompiled_sources)]
fn list_players_command_source_matches_java_26_1_2() {
    const LIST_PLAYERS: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ListPlayersCommand.java");

    for sentinel in [
        "Commands.literal(\"list\").executes(c -> listPlayers((CommandSourceStack)c.getSource()))",
        "Commands.literal(\"uuids\").executes(c -> listPlayersWithUuids((CommandSourceStack)c.getSource()))",
        "return format(source, Player::getDisplayName);",
        "Component.translatable(\"commands.list.nameAndId\", player.getName(), Component.translationArg(player.getGameProfile().id()))",
        "PlayerList playerList = source.getServer().getPlayerList();",
        "List<ServerPlayer> players = playerList.getPlayers();",
        "ComponentUtils.formatList(players, formatter)",
        "Component.translatable(\"commands.list.players\", players.size(), playerList.getMaxPlayers(), listComponent)",
        "return players.size();",
    ] {
        assert!(
            LIST_PLAYERS.contains(sentinel),
            "ListPlayersCommand.java is missing sentinel: {sentinel}"
        );
    }
}

#[test]
fn command_results_keep_source_permissions_feedback_and_side_effects_consistent() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        online_players: vec![steve.clone(), alex.clone()],
        ..ServerCommandState::default()
    };

    let player_list =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "list").unwrap();
    assert_eq!(player_list.success_count, 2);
    assert_eq!(player_list.feedback_key, "commands.list.players");
    assert!(!player_list.broadcast_to_admins);

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "gamemode creative"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert!(state.player_game_modes.is_empty());

    let op_gamemode = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamemode creative",
    )
    .unwrap();
    assert_eq!(op_gamemode.success_count, 1);
    assert_eq!(op_gamemode.feedback_key, "commands.gamemode.success.self");
    assert!(op_gamemode.broadcast_to_admins);
    assert_eq!(
        state
            .player_game_modes
            .iter()
            .find(|entry| entry.player.uuid == steve.uuid)
            .map(|entry| entry.gamemode),
        Some(GameMode::Creative)
    );
    assert!(!state
        .player_game_modes
        .iter()
        .any(|entry| entry.player.uuid == alex.uuid));

    let repeated = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "gamemode creative",
    )
    .unwrap();
    assert_eq!(repeated.success_count, 0);
    assert_eq!(repeated.feedback_key, NO_COMMAND_FEEDBACK);
    assert!(repeated.broadcast_to_admins);

    state.command_source_player = None;
    let console_set = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::OWNER,
        "gamemode spectator Alex",
    )
    .unwrap();
    assert_eq!(console_set.success_count, 1);
    assert_eq!(console_set.feedback_key, "commands.gamemode.success.other");
    assert!(console_set.broadcast_to_admins);
    assert_eq!(
        state
            .player_game_modes
            .iter()
            .find(|entry| entry.player.uuid == alex.uuid)
            .map(|entry| entry.gamemode),
        Some(GameMode::Spectator)
    );
}

#[test]
fn kick_command_requires_admin_published_server_and_non_owner_target() {
    let mut state = ServerCommandState::default();
    assert_eq!(command_required_permission("kick"), PermissionLevel::Admins);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "kick Steve"
        ),
        Err(CommandError::PermissionDenied)
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve"),
        Err(CommandError::KickSingleplayer)
    );

    let owner = NameAndId::create_offline("Steve");
    state.singleplayer_owner = Some(owner);
    state.published_server = Some(PublishRequest {
        port: 25565,
        allow_commands: false,
        gamemode: None,
    });
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve"),
        Err(CommandError::KickOwner)
    );
}

#[test]
fn kick_command_disconnects_targets_with_default_or_custom_reason() {
    let mut state = ServerCommandState {
        published_server: Some(PublishRequest {
            port: 25565,
            allow_commands: false,
            gamemode: None,
        }),
        ..ServerCommandState::default()
    };
    let kicked =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick Steve").unwrap();
    assert_eq!(kicked.success_count, 1);
    assert_eq!(kicked.feedback_key, "commands.kick.success");
    assert_eq!(
        state.disconnected_players[0].reason,
        "multiplayer.disconnect.kicked"
    );

    let kicked = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "kick Alex maintenance window",
    )
    .unwrap();
    assert_eq!(kicked.success_count, 1);
    assert_eq!(state.disconnected_players[1].player.name, "Alex");
    assert_eq!(state.disconnected_players[1].reason, "maintenance window");

    let kicked = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "kick Alex,Steve server restart",
    )
    .unwrap();
    assert_eq!(kicked.success_count, 2);
    assert_eq!(state.disconnected_players[2].player.name, "Alex");
    assert_eq!(state.disconnected_players[3].player.name, "Steve");
    assert_eq!(state.disconnected_players[2].reason, "server restart");

    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "kick ,"),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn ban_and_pardon_commands_track_profiles_and_disconnect_online_players() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        online_players: vec![steve.clone()],
        ..ServerCommandState::default()
    };
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "ban Steve"),
        Err(CommandError::PermissionDenied)
    );

    let banned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "ban Steve Alex -- repeated griefing",
    )
    .unwrap();
    assert_eq!(banned.success_count, 2);
    assert_eq!(banned.feedback_key, "commands.ban.success");
    assert_eq!(state.banned_player_names(), vec!["Steve", "Alex"]);
    assert_eq!(
        state.ban_player_feedback_events,
        vec![
            BanPlayerFeedbackEvent {
                player: steve.clone(),
                feedback_key: "commands.ban.success",
                broadcast_to_admins: true,
            },
            BanPlayerFeedbackEvent {
                player: alex,
                feedback_key: "commands.ban.success",
                broadcast_to_admins: true,
            },
        ]
    );
    assert_eq!(
        state.banned_players[0].reason.as_deref(),
        Some("repeated griefing")
    );
    assert_eq!(state.disconnected_players.len(), 1);
    assert_eq!(
        state.disconnected_players[0].reason,
        "multiplayer.disconnect.banned"
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "ban Steve"),
        Err(CommandError::BanFailed)
    );

    let list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "banlist players",
    )
    .unwrap();
    assert_eq!(list.success_count, 2);
    assert_eq!(list.feedback_key, "commands.banlist.list");
    assert!(!list.broadcast_to_admins);

    let pardoned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "pardon Steve Alex",
    )
    .unwrap();
    assert_eq!(pardoned.success_count, 2);
    assert_eq!(pardoned.feedback_key, "commands.pardon.success");
    assert_eq!(
        state.side_feedback,
        vec![CommandResult {
            success_count: 1,
            feedback_key: "commands.pardon.success",
            broadcast_to_admins: true,
        }]
    );
    assert!(state.banned_players.is_empty());
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "pardon Steve"),
        Err(CommandError::PardonFailed)
    );
}

#[test]
fn ban_command_treats_words_after_plain_target_as_vanilla_message_reason() {
    let mut state = ServerCommandState::default();

    let banned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "ban Steve repeated griefing",
    )
    .unwrap();

    assert_eq!(banned.success_count, 1);
    assert_eq!(state.banned_player_names(), vec!["Steve"]);
    assert_eq!(
        state.banned_players[0].reason.as_deref(),
        Some("repeated griefing")
    );
}

#[test]
fn ban_ip_banlist_and_pardon_ip_follow_vanilla_resolution_failures() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        online_player_addresses: vec![
            PlayerIpAddress {
                player: steve.clone(),
                ip: "203.0.113.7".to_string(),
            },
            PlayerIpAddress {
                player: alex.clone(),
                ip: "203.0.113.7".to_string(),
            },
        ],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "ban-ip missingPlayer"
        ),
        Err(CommandError::BanIpInvalid)
    );
    let banned =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "ban-ip Steve")
            .unwrap();
    assert_eq!(banned.success_count, 2);
    assert_eq!(banned.feedback_key, "commands.banip.info");
    assert_eq!(
        state.ban_ip_feedback_events,
        vec![BanIpFeedbackEvent {
            feedback_key: "commands.banip.success",
            broadcast_to_admins: true,
        }]
    );
    assert_eq!(state.banned_ip_names(), vec!["203.0.113.7"]);
    assert_eq!(state.disconnected_players.len(), 2);
    assert_eq!(
        state.disconnected_players[1].reason,
        "multiplayer.disconnect.ip_banned"
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "ban-ip 203.0.113.7"
        ),
        Err(CommandError::BanIpFailed)
    );

    let list =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ADMIN, "banlist").unwrap();
    assert_eq!(list.success_count, 1);
    let pardoned = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ADMIN,
        "pardon-ip 203.0.113.7",
    )
    .unwrap();
    assert_eq!(pardoned.feedback_key, "commands.pardonip.success");
    assert!(state.banned_ips.is_empty());
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "pardon-ip not-an-ip"
        ),
        Err(CommandError::PardonIpInvalid)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ADMIN,
            "pardon-ip 203.0.113.8"
        ),
        Err(CommandError::PardonIpFailed)
    );
}

#[test]
fn bossbar_add_list_get_remove_and_permission_match_vanilla_surface() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "bossbar add raid Raid Warning"
        ),
        Err(CommandError::PermissionDenied)
    );

    let created = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar add raid Raid Warning",
    )
    .unwrap();
    assert_eq!(created.success_count, 1);
    assert_eq!(created.feedback_key, "commands.bossbar.create.success");
    assert_eq!(state.bossbars[0].id, "minecraft:raid");
    assert_eq!(state.bossbars[0].name, "Raid Warning");
    assert_eq!(state.bossbars[0].color, BossBarCommandColor::White);
    assert_eq!(state.bossbars[0].overlay, BossBarCommandOverlay::Progress);
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar add raid Duplicate"
        ),
        Err(CommandError::BossBarAlreadyExists)
    );

    let list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar list",
    )
    .unwrap();
    assert_eq!(list.success_count, 1);
    assert_eq!(list.feedback_key, "commands.bossbar.list.bars.some");
    assert!(!list.broadcast_to_admins);
    let value = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar get raid value",
    )
    .unwrap();
    assert_eq!(value.success_count, 0);
    assert_eq!(value.feedback_key, "commands.bossbar.get.value");
    let max = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar get raid max",
    )
    .unwrap();
    assert_eq!(max.success_count, 100);
    assert_eq!(max.feedback_key, "commands.bossbar.get.max");

    let visible = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar get raid visible",
    )
    .unwrap();
    assert_eq!(visible.success_count, 1);
    assert_eq!(visible.feedback_key, "commands.bossbar.get.visible.visible");

    let removed = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar remove raid",
    )
    .unwrap();
    assert_eq!(removed.success_count, 0);
    assert!(state.bossbars.is_empty());
    let empty_list = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar list",
    )
    .unwrap();
    assert_eq!(empty_list.success_count, 0);
    assert_eq!(empty_list.feedback_key, "commands.bossbar.list.bars.none");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar remove raid"
        ),
        Err(CommandError::BossBarUnknown)
    );
}

#[test]
fn bossbar_set_mutates_values_players_and_reports_unchanged_errors() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        online_players: vec![steve.clone(), alex.clone()],
        ..ServerCommandState::default()
    };
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar add event Event",
    )
    .unwrap();

    let (max, value, hidden, players) = set_bossbar_event_values(&mut state);
    assert_bossbar_event_values(&state, max, value, hidden, players);
    assert_bossbar_event_unchanged_errors(&mut state);
    assert_bossbar_event_players_clear(&mut state);
}

fn set_bossbar_event_values(
    state: &mut ServerCommandState,
) -> (CommandResult, CommandResult, CommandResult, CommandResult) {
    execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event name Dragon Fight",
    )
    .unwrap();
    execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event color purple",
    )
    .unwrap();
    execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event style notched_10",
    )
    .unwrap();
    let max = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event max 250",
    )
    .unwrap();
    let value = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event value 125",
    )
    .unwrap();
    let hidden = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event visible false",
    )
    .unwrap();
    let players = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event players Steve Alex",
    )
    .unwrap();
    (max, value, hidden, players)
}

fn assert_bossbar_event_values(
    state: &ServerCommandState,
    max: CommandResult,
    value: CommandResult,
    hidden: CommandResult,
    players: CommandResult,
) {
    assert_eq!(max.success_count, 250);
    assert_eq!(value.success_count, 125);
    assert_eq!(
        hidden.feedback_key,
        "commands.bossbar.set.visible.success.hidden"
    );
    assert_eq!(players.success_count, 2);
    assert_eq!(state.bossbars[0].name, "Dragon Fight");
    assert_eq!(state.bossbars[0].color, BossBarCommandColor::Purple);
    assert_eq!(state.bossbars[0].overlay, BossBarCommandOverlay::Notched10);
    assert!(!state.bossbars[0].visible);
    assert_eq!(state.bossbars[0].players.len(), 2);
    assert_eq!(state.bossbars[0].players[0].name, "Steve");
}

fn assert_bossbar_event_unchanged_errors(state: &mut ServerCommandState) {
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar get event players"
        )
        .unwrap()
        .feedback_key,
        "commands.bossbar.get.players.some"
    );

    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event name Dragon Fight"
        ),
        Err(CommandError::BossBarNameUnchanged)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event color purple"
        ),
        Err(CommandError::BossBarColorUnchanged)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event style notched_10"
        ),
        Err(CommandError::BossBarStyleUnchanged)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event max 250"
        ),
        Err(CommandError::BossBarMaxUnchanged)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event value 125"
        ),
        Err(CommandError::BossBarValueUnchanged)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event visible false"
        ),
        Err(CommandError::BossBarAlreadyHidden)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event visible true"
        )
        .unwrap()
        .feedback_key,
        "commands.bossbar.set.visible.success.visible"
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event visible true"
        ),
        Err(CommandError::BossBarAlreadyVisible)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event players Herobrine"
        ),
        Err(CommandError::NoPlayers)
    );
    assert_eq!(
        execute_builtin_command(
            state,
            LevelBasedPermissionSet::GAMEMASTER,
            "bossbar set event players Steve Alex"
        ),
        Err(CommandError::BossBarPlayersUnchanged)
    );
}

fn assert_bossbar_event_players_clear(state: &mut ServerCommandState) {
    let cleared = execute_builtin_command(
        state,
        LevelBasedPermissionSet::GAMEMASTER,
        "bossbar set event players",
    )
    .unwrap();
    assert_eq!(cleared.success_count, 0);
    assert_eq!(
        cleared.feedback_key,
        "commands.bossbar.set.players.success.none"
    );
}

#[test]
fn chase_command_starts_follow_lead_and_stop_sessions_with_vanilla_defaults() {
    let mut state = ServerCommandState::default();
    let follow =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "chase follow").unwrap();
    assert_eq!(follow.success_count, 0);
    assert_eq!(follow.feedback_key, "commands.chase.follow.success");
    assert!(!follow.broadcast_to_admins);
    assert_eq!(
        state.chase_session,
        Some(ChaseSession::Following {
            host: "localhost".to_string(),
            port: 10000,
        })
    );
    assert_eq!(
        state.chase_events,
        vec![ChaseEvent::FollowStarted {
            host: "localhost".to_string(),
            port: 10000,
        }]
    );
    assert_eq!(
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "chase lead"),
        Err(CommandError::ChaseAlreadyRunning)
    );

    let stopped =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "chase stop").unwrap();
    assert_eq!(stopped.feedback_key, "commands.chase.stop");
    assert_eq!(state.chase_session, None);
    assert_eq!(state.chase_events[1], ChaseEvent::FollowStopped);

    let lead =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "chase lead").unwrap();
    assert_eq!(lead.feedback_key, "commands.chase.lead.success");
    assert_eq!(
        state.chase_session,
        Some(ChaseSession::Leading {
            bind_address: "0.0.0.0".to_string(),
            port: 10000,
        })
    );
}

#[test]
fn chase_command_accepts_explicit_endpoints_and_rejects_bad_ports() {
    let mut state = ServerCommandState::default();
    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "chase follow example.test 25565",
    )
    .unwrap();
    assert_eq!(
        state.chase_session,
        Some(ChaseSession::Following {
            host: "example.test".to_string(),
            port: 25565,
        })
    );
    execute_builtin_command(&mut state, LevelBasedPermissionSet::ALL, "chase stop").unwrap();

    execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::ALL,
        "chase lead 127.0.0.1 12000",
    )
    .unwrap();
    assert_eq!(
        state.chase_session,
        Some(ChaseSession::Leading {
            bind_address: "127.0.0.1".to_string(),
            port: 12000,
        })
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "chase lead host 1024"
        ),
        Err(CommandError::ChaseAlreadyRunning)
    );

    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "chase follow host 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::ALL,
            "chase lead host 1023"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(command_required_permission("chase"), PermissionLevel::All);
}

#[test]
fn clear_command_defaults_to_source_and_removes_matching_items() {
    let steve = NameAndId::create_offline("Steve");
    let mut state = ServerCommandState {
        command_source_player: Some(steve.clone()),
        player_inventories: vec![CommandPlayerInventory {
            player: steve,
            items: vec![
                CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 32,
                },
                CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 5,
                },
            ],
        }],
        ..ServerCommandState::default()
    };

    let cleared =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::GAMEMASTER, "clear").unwrap();
    assert_eq!(cleared.success_count, 37);
    assert_eq!(cleared.feedback_key, "commands.clear.success.single");
    assert!(cleared.broadcast_to_admins);
    assert!(state.player_inventories[0].items.is_empty());
}

#[test]
fn clear_command_supports_item_predicate_test_mode_limits_and_failures() {
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let mut state = ServerCommandState {
        online_players: vec![steve.clone(), alex.clone()],
        player_inventories: vec![
            CommandPlayerInventory {
                player: steve,
                items: vec![
                    CommandItemStack {
                        item: "minecraft:stone".to_string(),
                        count: 32,
                    },
                    CommandItemStack {
                        item: "minecraft:apple".to_string(),
                        count: 5,
                    },
                ],
            },
            CommandPlayerInventory {
                player: alex,
                items: vec![CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 12,
                }],
            },
        ],
        ..ServerCommandState::default()
    };

    let counted = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clear Steve,Alex stone 0",
    )
    .unwrap();
    assert_eq!(counted.success_count, 44);
    assert_eq!(counted.feedback_key, "commands.clear.test.multiple");
    assert_eq!(state.player_inventories[0].items[0].count, 32);

    let limited = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "clear Steve stone 10",
    )
    .unwrap();
    assert_eq!(limited.success_count, 10);
    assert_eq!(limited.feedback_key, "commands.clear.success.single");
    assert_eq!(state.player_inventories[0].items[0].count, 22);

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clear Steve diamond"
        ),
        Err(CommandError::ClearFailedSingle)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clear Steve,Alex diamond"
        ),
        Err(CommandError::ClearFailedMultiple)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clear Herobrine stone"
        ),
        Err(CommandError::NoPlayers)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clear Steve definitely_not_an_item"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "clear Steve stone -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn give_command_adds_items_to_single_and_multiple_player_inventories() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("give"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "give Steve stone"
        ),
        Err(CommandError::PermissionDenied)
    );

    let single = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve stone 65",
    )
    .unwrap();
    assert_eq!(single.success_count, 1);
    assert_eq!(single.feedback_key, "commands.give.success.single");
    assert_eq!(
        state.player_inventories[0],
        CommandPlayerInventory {
            player: NameAndId::create_offline("Steve"),
            items: vec![
                CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 64,
                },
                CommandItemStack {
                    item: "minecraft:stone".to_string(),
                    count: 1,
                },
            ],
        }
    );

    let multiple = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve,Alex diamond_sword 2",
    )
    .unwrap();
    assert_eq!(multiple.success_count, 2);
    assert_eq!(multiple.feedback_key, "commands.give.success.single");
    let alex = state
        .player_inventories
        .iter()
        .find(|inventory| inventory.player.name == "Alex")
        .unwrap();
    assert_eq!(
        alex.items,
        vec![
            CommandItemStack {
                item: "minecraft:diamond_sword".to_string(),
                count: 1,
            },
            CommandItemStack {
                item: "minecraft:diamond_sword".to_string(),
                count: 1,
            },
        ]
    );
}

#[test]
fn give_command_rejects_invalid_counts_and_too_many_stacks() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "give Steve stone 0"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "give Steve stone -1"
        ),
        Err(CommandError::InvalidSyntax)
    );
    let too_many_stone = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve stone 6401",
    )
    .unwrap();
    assert_eq!(too_many_stone.success_count, 0);
    assert_eq!(
        too_many_stone.feedback_key,
        "commands.give.failed.toomanyitems"
    );
    assert!(!too_many_stone.broadcast_to_admins);

    let too_many_sword = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve diamond_sword 101",
    )
    .unwrap();
    assert_eq!(too_many_sword.success_count, 0);
    assert_eq!(
        too_many_sword.feedback_key,
        "commands.give.failed.toomanyitems"
    );
    let too_many_egg = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "give Steve egg 1601",
    )
    .unwrap();
    assert_eq!(too_many_egg.success_count, 0);
    assert_eq!(too_many_egg.feedback_key, "commands.give.failed.toomanyitems");
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "give Steve BadItem"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "give Steve definitely_not_a_real_item"
        ),
        Err(CommandError::InvalidSyntax)
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "give Steve lit_redstone_ore"
        ),
        Err(CommandError::InvalidSyntax)
    );
}

#[test]
fn item_command_replaces_entity_and_block_slots() {
    let mut state = ServerCommandState::default();
    assert_eq!(
        command_required_permission("item"),
        PermissionLevel::Gamemasters
    );
    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::MODERATOR,
            "item replace entity Steve weapon.mainhand with diamond_sword"
        ),
        Err(CommandError::PermissionDenied)
    );

    let entity = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "item replace entity Steve weapon.mainhand with diamond_sword",
    )
    .unwrap();
    assert_eq!(entity.success_count, 1);
    assert_eq!(
        entity.feedback_key,
        "commands.item.entity.set.success.single"
    );
    assert_eq!(
        state.entity_item_slots,
        vec![CommandEntityItemSlot {
            entity: EntityRef {
                id: "Steve".to_string(),
                display_name: "Steve".to_string(),
            },
            slot: "weapon.mainhand".to_string(),
            item: Some(CommandItemStack {
                item: "minecraft:diamond_sword".to_string(),
                count: 1,
            }),
        }]
    );

    state.block_item_slots.push(CommandBlockItemSlot {
        pos: BlockPos { x: 1, y: 64, z: 2 },
        slot: "container.0".to_string(),
        item: None,
    });

    let block = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "item replace block 1 64 2 container.0 with stone 32",
    )
    .unwrap();
    assert_eq!(block.success_count, 1);
    assert_eq!(block.feedback_key, "commands.item.block.set.success");
    assert_eq!(
        state.block_item_slots,
        vec![CommandBlockItemSlot {
            pos: BlockPos { x: 1, y: 64, z: 2 },
            slot: "container.0".to_string(),
            item: Some(CommandItemStack {
                item: "minecraft:stone".to_string(),
                count: 32,
            }),
        }]
    );
}

#[test]
fn item_command_copies_between_block_and_entity_sources() {
    let mut state = ServerCommandState::default();
    state.block_item_slots.push(CommandBlockItemSlot {
        pos: BlockPos { x: 0, y: 64, z: 0 },
        slot: "container.2".to_string(),
        item: Some(CommandItemStack {
            item: "minecraft:apple".to_string(),
            count: 9,
        }),
    });

    let to_entities = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "item replace entity Steve,Alex hotbar.0 from block 0 64 0 container.2",
    )
    .unwrap();
    assert_eq!(to_entities.success_count, 2);
    assert_eq!(
        to_entities.feedback_key,
        "commands.item.entity.set.success.multiple"
    );
    assert_eq!(state.entity_item_slots.len(), 2);
    assert!(state.entity_item_slots.iter().all(|entry| entry.item
        == Some(CommandItemStack {
            item: "minecraft:apple".to_string(),
            count: 9,
        })));

    state.block_item_slots.push(CommandBlockItemSlot {
        pos: BlockPos { x: 2, y: 64, z: 2 },
        slot: "container.1".to_string(),
        item: None,
    });
    let to_block = execute_builtin_command(
        &mut state,
        LevelBasedPermissionSet::GAMEMASTER,
        "item replace block 2 64 2 container.1 from entity Steve hotbar.0",
    )
    .unwrap();
    assert_eq!(to_block.success_count, 1);
    assert!(state
        .block_item_slots
        .iter()
        .any(|entry| entry.pos == BlockPos { x: 2, y: 64, z: 2 }
            && entry.slot == "container.1"
            && entry.item
                == Some(CommandItemStack {
                    item: "minecraft:apple".to_string(),
                    count: 9,
                })));
}
