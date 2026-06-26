use super::*;

type DispatchResult = Option<Result<CommandResult, CommandError>>;

pub fn execute_builtin_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    input: &str,
) -> Result<CommandResult, CommandError> {
    let input = input
        .trim_start()
        .strip_prefix('/')
        .unwrap_or(input.trim_start());
    let parts = input.split_whitespace().collect::<Vec<_>>();
    let Some(command) = parts.first().copied() else {
        return Err(CommandError::InvalidSyntax);
    };
    if permissions.can_run(command) == CommandAvailability::Hidden {
        return Err(CommandError::PermissionDenied);
    }

    dispatch_builtin_command(state, permissions, command, &parts)
}

fn dispatch_builtin_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    command: &str,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if let Some(result) = dispatch_server_command(state, permissions, command, parts) {
        return result;
    }
    if let Some(result) = dispatch_delegated_command(state, permissions, command, parts) {
        return result;
    }
    if let Some(result) = dispatch_chat_command(state, permissions, command, parts) {
        return result;
    }
    if let Some(result) = dispatch_admin_command(state, command, parts) {
        return result;
    }
    if let Some(result) = dispatch_runtime_command(state, permissions, command, parts) {
        return result;
    }
    Err(CommandError::InvalidSyntax)
}

fn dispatch_server_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    command: &str,
    parts: &[&str],
) -> DispatchResult {
    Some(match command {
        "setidletimeout" => set_idle_timeout_command(state, parts),
        "save-all" => save_all_command(state, parts),
        "save-off" => save_off_command(state, parts),
        "save-on" => save_on_command(state, parts),
        "stop" => stop_command(state, parts),
        "help" => help_command(permissions, parts),
        "jfr" => jfr_command(state, parts),
        "list" => list_command(state, parts),
        "seed" => seed_command(state, parts),
        "version" => version_command(parts),
        "reload" => reload_command(state, parts),
        "publish" => publish_command(state, parts),
        _ => return None,
    })
}

fn dispatch_delegated_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    command: &str,
    parts: &[&str],
) -> DispatchResult {
    Some(match command {
        "advancement" => advancement_command(state, parts),
        "attribute" => attribute_command(state, parts),
        "biome" => biome_command(state, parts),
        "bossbar" => bossbar_command(state, parts),
        "chase" => chase_command(state, parts),
        "clear" => clear_command(state, parts),
        "clone" => clone_command(state, parts),
        "damage" => damage_command(state, parts),
        "datapack" => datapack_command(state, parts, permissions),
        "debug" => debug_command(state, parts),
        "debugconfig" => debug_config_command(state, parts),
        "debugmobspawning" => debug_mob_spawning_command(state, parts),
        "debugpath" => debug_path_command(state, parts),
        "defaultgamemode" => default_gamemode_command(state, parts),
        "difficulty" => difficulty_command(state, parts),
        "dialog" => dialog_command(state, parts),
        "effect" => effect_command(state, parts),
        "enchant" => enchant_command(state, parts),
        "execute" => execute_command(state, permissions, parts),
        "experience" | "xp" => experience_command(state, parts),
        "fetchprofile" => fetch_profile_command(state, parts),
        "fill" => fill_command(state, parts),
        "fillbiome" => fill_biome_command(state, parts),
        "forceload" => forceload_command(state, parts),
        "function" => function_command(state, parts),
        "gamemode" => gamemode_command(state, parts),
        "gamerule" => gamerule_command(state, parts),
        "give" => give_command(state, parts),
        "item" => item_command(state, parts),
        "locate" => locate_command(state, parts),
        "loot" => loot_command(state, parts),
        "particle" => particle_command(state, parts),
        "perf" => perf_command(state, parts),
        "place" => place_command(state, parts),
        "raid" => raid_command(state, parts),
        "recipe" => recipe_command(state, parts),
        "return" => return_command(state, parts),
        "ride" => ride_command(state, parts),
        "rotate" => rotate_command(state, parts),
        "schedule" => schedule_command(state, parts),
        "scoreboard" => scoreboard_command(state, parts),
        "serverpack" => server_pack_command(state, parts),
        "setblock" => setblock_command(state, parts),
        "setworldspawn" => setworldspawn_command(state, parts),
        "spawn_armor_trims" => spawn_armor_trims_command(state, parts),
        "spawnpoint" => spawnpoint_command(state, parts),
        "spreadplayers" => spreadplayers_command(state, parts),
        "spectate" => spectate_command(state, parts),
        "stopwatch" => stopwatch_command(state, parts),
        "summon" => summon_command(state, parts),
        "swing" => swing_command(state, parts),
        "tag" => tag_command(state, parts),
        "team" => team_command(state, parts),
        "teleport" | "tp" => teleport_command(state, parts),
        "time" => time_command(state, parts),
        "title" => title_command(state, parts),
        "trigger" => trigger_command(state, parts),
        "warden_spawn_tracker" => warden_spawn_tracker_command(state, parts),
        "waypoint" => waypoint_command(state, parts),
        "worldborder" => worldborder_command(state, parts),
        _ => return None,
    })
}

fn dispatch_chat_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    command: &str,
    parts: &[&str],
) -> DispatchResult {
    Some(match command {
        "say" => chat_broadcast_command(state, parts, ChatCommandKind::Say),
        "me" => chat_broadcast_command(state, parts, ChatCommandKind::Emote),
        "msg" | "tell" | "w" => private_message_command(state, parts, ChatCommandKind::Private),
        "tellraw" => private_message_command(state, parts, ChatCommandKind::TellRaw),
        "teammsg" | "tm" => team_message_command(state, parts),
        "playsound" => play_sound_command(state, parts, permissions),
        "stopsound" => stop_sound_command(state, parts),
        _ => return None,
    })
}

fn dispatch_admin_command(
    state: &mut ServerCommandState,
    command: &str,
    parts: &[&str],
) -> DispatchResult {
    Some(match command {
        "ban" => ban_command(state, parts),
        "ban-ip" => ban_ip_command(state, parts),
        "banlist" => banlist_command(state, parts),
        "deop" => deop_command(state, parts),
        "kick" => kick_command(state, parts),
        "kill" => kill_command(state, parts),
        "op" => op_command(state, parts),
        "pardon" => pardon_command(state, parts),
        "pardon-ip" => pardon_ip_command(state, parts),
        "whitelist" => whitelist_command(state, parts),
        _ => return None,
    })
}

fn dispatch_runtime_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    command: &str,
    parts: &[&str],
) -> DispatchResult {
    Some(match command {
        "random" => random_command(state, permissions, parts),
        "tick" => tick_command(state, parts),
        "transfer" => transfer_command(state, parts),
        "weather" => weather_command(state, parts),
        _ => return None,
    })
}

fn set_idle_timeout_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let minutes = parts[1]
        .parse::<i32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if minutes < 0 {
        return Err(CommandError::InvalidSyntax);
    }
    state.player_idle_timeout_minutes = minutes as u32;
    Ok(CommandResult {
        success_count: minutes,
        feedback_key: if minutes > 0 {
            "commands.setidletimeout.success"
        } else {
            "commands.setidletimeout.success.disabled"
        },
        broadcast_to_admins: true,
    })
}

fn save_all_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let flush = match parts {
        ["save-all"] => false,
        ["save-all", "flush"] => true,
        _ => return Err(CommandError::InvalidSyntax),
    };
    state
        .side_feedback
        .push(CommandResult {
            success_count: 1,
            feedback_key: "commands.save.saving",
            broadcast_to_admins: false,
        });
    state.save_all_requests.push(SaveAllRequest { flush });
    if state.save_all_should_fail {
        return Err(CommandError::SaveFailed);
    }
    Ok(success_result("commands.save.success", true))
}

fn save_off_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    if !state.autosave_enabled {
        return Err(CommandError::SaveAlreadyOff);
    }
    state.autosave_enabled = false;
    Ok(success_result("commands.save.disabled", true))
}

fn save_on_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    if state.autosave_enabled {
        return Err(CommandError::SaveAlreadyOn);
    }
    state.autosave_enabled = true;
    Ok(success_result("commands.save.enabled", true))
}

fn stop_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    state.halt_requested = true;
    Ok(success_result("commands.stop.stopping", true))
}

fn help_command(
    permissions: LevelBasedPermissionSet,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["help"] => Ok(CommandResult {
            success_count: visible_command_usages(permissions).len() as i32,
            feedback_key: "commands.help.success",
            broadcast_to_admins: false,
        }),
        ["help", command @ ..] if !command.is_empty() => {
            let command = command.join(" ");
            let root = command.split_whitespace().next().unwrap_or_default();
            if let Some(usage) = command_usage(root, permissions) {
                Ok(CommandResult {
                    success_count: help_smart_usage_count(root, usage),
                    feedback_key: "commands.help.success",
                    broadcast_to_admins: false,
                })
            } else {
                Err(CommandError::HelpFailed)
            }
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn help_smart_usage_count(root: &str, usage: &str) -> i32 {
    let root_usage = format!("/{root}");
    if usage == root_usage { 0 } else { 1 }
}

fn list_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["list"] | ["list", "uuids"] => {
            state.last_list_includes_uuids = matches!(parts, ["list", "uuids"]);
            Ok(CommandResult {
                success_count: state.online_players.len() as i32,
                feedback_key: "commands.list.players",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn seed_command(state: &ServerCommandState, parts: &[&str]) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(CommandResult {
        success_count: state.world_seed as i32,
        feedback_key: "commands.seed.success",
        broadcast_to_admins: false,
    })
}

fn version_command(parts: &[&str]) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(success_result("commands.version.header", false))
}

fn reload_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() != 1 {
        return Err(CommandError::InvalidSyntax);
    }
    let selected = discover_reload_packs(state);
    state.selected_data_packs = selected.clone();
    state.reload_requests.push(ReloadRequest {
        selected_packs: selected,
    });
    Ok(CommandResult {
        success_count: 0,
        feedback_key: "commands.reload.success",
        broadcast_to_admins: true,
    })
}

fn publish_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["publish"] => publish_server(state, state.next_available_publish_port, false, None),
        ["publish", allow_commands] => publish_server(
            state,
            state.next_available_publish_port,
            parse_bool(allow_commands)?,
            None,
        ),
        ["publish", allow_commands, gamemode] => publish_server(
            state,
            state.next_available_publish_port,
            parse_bool(allow_commands)?,
            Some(parse_gamemode(gamemode)?),
        ),
        ["publish", allow_commands, gamemode, port] => publish_server(
            state,
            parse_publish_port(port)?,
            parse_bool(allow_commands)?,
            Some(parse_gamemode(gamemode)?),
        ),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn chat_broadcast_command(
    state: &mut ServerCommandState,
    parts: &[&str],
    kind: ChatCommandKind,
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    state.chat_events.push(ChatCommandEvent {
        kind,
        sender: state.command_source_player.clone(),
        targets: state.online_players.clone(),
        message: parts[1..].join(" "),
    });
    let feedback_key = match kind {
        ChatCommandKind::Say => NO_COMMAND_FEEDBACK,
        ChatCommandKind::Emote => "commands.me.success",
        ChatCommandKind::Private | ChatCommandKind::Team | ChatCommandKind::TellRaw => {
            return Err(CommandError::InvalidSyntax);
        }
    };
    Ok(success_result(feedback_key, false))
}

fn private_message_command(
    state: &mut ServerCommandState,
    parts: &[&str],
    kind: ChatCommandKind,
) -> Result<CommandResult, CommandError> {
    if parts.len() < 3 {
        return Err(CommandError::InvalidSyntax);
    }
    let targets = parse_name_list(parts[1]);
    if targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let count = targets.len() as i32;
    state.chat_events.push(ChatCommandEvent {
        kind,
        sender: state.command_source_player.clone(),
        targets,
        message: parts[2..].join(" "),
    });
    Ok(CommandResult {
        success_count: count,
        feedback_key: match kind {
            ChatCommandKind::Private | ChatCommandKind::TellRaw => NO_COMMAND_FEEDBACK,
            ChatCommandKind::Say | ChatCommandKind::Emote | ChatCommandKind::Team => {
                return Err(CommandError::InvalidSyntax);
            }
        },
        broadcast_to_admins: false,
    })
}

fn team_message_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let sender = state
        .command_source_player
        .clone()
        .ok_or(CommandError::InvalidSyntax)?;
    let team = state
        .team_for_player(&sender)
        .ok_or(CommandError::TeamMsgNoTeam)?
        .to_string();
    let targets = state
        .online_players
        .iter()
        .filter(|player| state.team_for_player(player) == Some(team.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let count = targets.len() as i32;
    state.chat_events.push(ChatCommandEvent {
        kind: ChatCommandKind::Team,
        sender: Some(sender),
        targets,
        message: parts[1..].join(" "),
    });
    Ok(CommandResult {
        success_count: count,
        feedback_key: NO_COMMAND_FEEDBACK,
        broadcast_to_admins: false,
    })
}

fn kick_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["kick", targets @ ..] if !targets.is_empty() => {
            let split = targets
                .iter()
                .position(|part| *part == "--")
                .unwrap_or(targets.len());
            let (targets, reason_parts) = targets.split_at(split);
            let reason = if reason_parts.is_empty() {
                "multiplayer.disconnect.kicked".to_string()
            } else {
                reason_parts[1..].join(" ")
            };
            kick_players(state, targets, reason)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

// TODO(op-deop-live-wiring): the command model mirrors Java's PlayerList mutation,
// feedback, and kick request semantics, but live command side effects still need
// PlayerAccess ops.json persistence plus PlayerList.op's target permission/command
// packet resend once cross-player command effects are wired.
fn op_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["op", targets @ ..] if !targets.is_empty() => {
            let changed = targets
                .iter()
                .filter_map(|target| {
                    let profile = NameAndId::create_offline(target);
                    state.add_operator(profile.clone()).then_some(profile)
                })
                .collect::<Vec<_>>();
            if changed.is_empty() {
                return Err(CommandError::OpFailed);
            }
            push_extra_player_feedback(state, changed.len(), "commands.op.success");
            Ok(CommandResult {
                success_count: changed.len() as i32,
                feedback_key: "commands.op.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn deop_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["deop", targets @ ..] if !targets.is_empty() => {
            let changed = targets
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .filter(|profile| state.remove_operator(profile))
                .collect::<Vec<_>>();
            if changed.is_empty() {
                return Err(CommandError::DeOpFailed);
            }
            push_extra_player_feedback(state, changed.len(), "commands.deop.success");
            state.kick_unlisted_requests += 1;
            Ok(CommandResult {
                success_count: changed.len() as i32,
                feedback_key: "commands.deop.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn push_extra_player_feedback(
    state: &mut ServerCommandState,
    changed_count: usize,
    feedback_key: &'static str,
) {
    state
        .side_feedback
        .extend((1..changed_count).map(|_| CommandResult {
            success_count: 1,
            feedback_key,
            broadcast_to_admins: true,
        }));
}

fn kill_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["kill"] => {
            let source = state
                .command_source_entity
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            kill_entities(state, vec![source])
        }
        ["kill", targets] => kill_entities(state, parse_entity_list(targets)),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn whitelist_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["whitelist", "on"] => whitelist_on_command(state),
        ["whitelist", "off"] => whitelist_off_command(state),
        ["whitelist", "list"] => whitelist_list_command(state),
        ["whitelist", "reload"] => whitelist_reload_command(state),
        ["whitelist", "add", targets @ ..] if !targets.is_empty() => {
            whitelist_add_command(state, targets)
        }
        ["whitelist", "remove", targets @ ..] if !targets.is_empty() => {
            whitelist_remove_command(state, targets)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn whitelist_on_command(state: &mut ServerCommandState) -> Result<CommandResult, CommandError> {
    if state.whitelist_enabled {
        return Err(CommandError::WhitelistAlreadyOn);
    }
    state.whitelist_enabled = true;
    state.kick_unlisted_requests += 1;
    Ok(success_result("commands.whitelist.enabled", true))
}

fn whitelist_off_command(state: &mut ServerCommandState) -> Result<CommandResult, CommandError> {
    if !state.whitelist_enabled {
        return Err(CommandError::WhitelistAlreadyOff);
    }
    state.whitelist_enabled = false;
    Ok(success_result("commands.whitelist.disabled", true))
}

fn whitelist_list_command(state: &ServerCommandState) -> Result<CommandResult, CommandError> {
    Ok(CommandResult {
        success_count: state.whitelisted_players.len() as i32,
        feedback_key: if state.whitelisted_players.is_empty() {
            "commands.whitelist.none"
        } else {
            "commands.whitelist.list"
        },
        broadcast_to_admins: false,
    })
}

fn whitelist_reload_command(state: &mut ServerCommandState) -> Result<CommandResult, CommandError> {
    state.whitelist_reload_requests += 1;
    state.kick_unlisted_requests += 1;
    Ok(success_result("commands.whitelist.reloaded", true))
}

fn whitelist_add_command(
    state: &mut ServerCommandState,
    targets: &[&str],
) -> Result<CommandResult, CommandError> {
    let success = targets
        .iter()
        .filter(|target| state.add_whitelisted(NameAndId::create_offline(target)))
        .count() as i32;
    if success == 0 {
        return Err(CommandError::AlreadyWhitelisted);
    }
    Ok(CommandResult {
        success_count: success,
        feedback_key: "commands.whitelist.add.success",
        broadcast_to_admins: true,
    })
}

fn whitelist_remove_command(
    state: &mut ServerCommandState,
    targets: &[&str],
) -> Result<CommandResult, CommandError> {
    let success = targets
        .iter()
        .map(|target| NameAndId::create_offline(target))
        .filter(|profile| state.remove_whitelisted(profile))
        .count() as i32;
    if success == 0 {
        return Err(CommandError::NotWhitelisted);
    }
    state.kick_unlisted_requests += 1;
    Ok(CommandResult {
        success_count: success,
        feedback_key: "commands.whitelist.remove.success",
        broadcast_to_admins: true,
    })
}

fn tick_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["tick", "query"] => Ok(CommandResult {
            success_count: state.tick_rate.tick_rate() as i32,
            feedback_key: tick_query_status_key(state),
            broadcast_to_admins: false,
        }),
        ["tick", "rate", rate] => tick_rate_command(state, rate),
        ["tick", "freeze"] => {
            if state.tick_rate.is_sprinting() {
                state.tick_rate.stop_sprinting();
            }
            if state.tick_rate.is_stepping_forward() {
                state.tick_rate.stop_stepping();
            }
            state.tick_rate.set_frozen(true);
            Ok(success_result("commands.tick.status.frozen", true))
        }
        ["tick", "unfreeze"] => {
            state.tick_rate.set_frozen(false);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.tick.status.running",
                broadcast_to_admins: true,
            })
        }
        ["tick", "step"] => tick_step(state, 1),
        ["tick", "step", "stop"] => tick_step_stop_command(state),
        ["tick", "step", time] => tick_step(state, parse_time_ticks(time)?),
        ["tick", "sprint", "stop"] => tick_sprint_stop_command(state),
        ["tick", "sprint", time] => tick_sprint_command(state, time),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn tick_rate_command(
    state: &mut ServerCommandState,
    rate: &str,
) -> Result<CommandResult, CommandError> {
    let rate = rate
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if !(MIN_TICK_RATE..=MAX_TICK_RATE).contains(&rate) {
        return Err(CommandError::InvalidSyntax);
    }
    state.tick_rate.set_tick_rate(rate);
    Ok(CommandResult {
        success_count: rate as i32,
        feedback_key: "commands.tick.rate.success",
        broadcast_to_admins: true,
    })
}

fn tick_step_stop_command(state: &mut ServerCommandState) -> Result<CommandResult, CommandError> {
    let stopped = state.tick_rate.stop_stepping();
    Ok(CommandResult {
        success_count: i32::from(stopped),
        feedback_key: if stopped {
            "commands.tick.step.stop.success"
        } else {
            "commands.tick.step.stop.fail"
        },
        broadcast_to_admins: stopped,
    })
}

fn tick_sprint_stop_command(state: &mut ServerCommandState) -> Result<CommandResult, CommandError> {
    let stopped = state.tick_rate.stop_sprinting();
    Ok(CommandResult {
        success_count: i32::from(stopped),
        feedback_key: if stopped {
            "commands.tick.sprint.stop.success"
        } else {
            "commands.tick.sprint.stop.fail"
        },
        broadcast_to_admins: stopped,
    })
}

fn tick_sprint_command(
    state: &mut ServerCommandState,
    time: &str,
) -> Result<CommandResult, CommandError> {
    let ticks = parse_time_ticks(time)? as u64;
    let interrupted = state.tick_rate.request_game_to_sprint(ticks);
    if interrupted {
        state.tick_feedback_events.push(TickCommandFeedbackEvent {
            feedback_key: "commands.tick.sprint.stop.success",
            broadcast_to_admins: true,
        });
    }
    Ok(success_result("commands.tick.status.sprinting", true))
}

fn transfer_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["transfer", host] => transfer_source_player(state, host, 25565),
        ["transfer", host, port] => transfer_source_player(state, host, parse_port(port)?),
        ["transfer", host, port, players @ ..] if !players.is_empty() => {
            let targets = players
                .iter()
                .map(|player| NameAndId::create_offline(player))
                .collect::<Vec<_>>();
            queue_transfer(state, host, parse_port(port)?, targets)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn transfer_source_player(
    state: &mut ServerCommandState,
    host: &str,
    port: u16,
) -> Result<CommandResult, CommandError> {
    let source = state
        .command_source_player
        .clone()
        .ok_or(CommandError::NoPlayers)?;
    queue_transfer(state, host, port, vec![source])
}

fn random_command(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["random", "value", range] => random_sample(state, range, None, false),
        ["random", "roll", range] => random_sample(state, range, None, true),
        ["random", "value", range, sequence] => {
            require_gamemaster(permissions)?;
            random_sample(state, range, Some(*sequence), false)
        }
        ["random", "roll", range, sequence] => {
            require_gamemaster(permissions)?;
            random_sample(state, range, Some(*sequence), true)
        }
        ["random", "reset", "*"] => random_reset_all_default(state, permissions),
        ["random", "reset", "*", seed] => {
            random_reset_all_checked(state, permissions, seed, true, true)
        }
        ["random", "reset", "*", seed, include_world_seed] => random_reset_all_checked(
            state,
            permissions,
            seed,
            parse_bool(include_world_seed)?,
            true,
        ),
        ["random", "reset", "*", seed, include_world_seed, include_sequence_id] => {
            random_reset_all_checked(
                state,
                permissions,
                seed,
                parse_bool(include_world_seed)?,
                parse_bool(include_sequence_id)?,
            )
        }
        ["random", "reset", sequence] => {
            reset_random_sequence_checked(state, permissions, sequence, None, true, true)
        }
        ["random", "reset", sequence, seed] => reset_random_sequence_checked(
            state,
            permissions,
            sequence,
            Some(parse_i32(seed)?),
            true,
            true,
        ),
        ["random", "reset", sequence, seed, include_world_seed] => reset_random_sequence_checked(
            state,
            permissions,
            sequence,
            Some(parse_i32(seed)?),
            parse_bool(include_world_seed)?,
            true,
        ),
        ["random", "reset", sequence, seed, include_world_seed, include_sequence_id] => {
            reset_random_sequence_checked(
                state,
                permissions,
                sequence,
                Some(parse_i32(seed)?),
                parse_bool(include_world_seed)?,
                parse_bool(include_sequence_id)?,
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn random_reset_all_default(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
) -> Result<CommandResult, CommandError> {
    require_gamemaster(permissions)?;
    let count = state.random_sequences.clear() as i32;
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.random.reset.all.success",
        broadcast_to_admins: false,
    })
}

fn random_reset_all_checked(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    seed: &str,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    require_gamemaster(permissions)?;
    random_reset_all(state, seed, include_world_seed, include_sequence_id)
}

fn reset_random_sequence_checked(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    sequence: &str,
    seed: Option<i32>,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    require_gamemaster(permissions)?;
    reset_random_sequence(
        state,
        sequence,
        seed,
        include_world_seed,
        include_sequence_id,
    )
}

fn weather_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["weather", "clear"] => set_weather(state, WeatherMode::Clear, None),
        ["weather", "clear", duration] => {
            set_weather(state, WeatherMode::Clear, Some(parse_time_ticks(duration)?))
        }
        ["weather", "rain"] => set_weather(state, WeatherMode::Rain, None),
        ["weather", "rain", duration] => {
            set_weather(state, WeatherMode::Rain, Some(parse_time_ticks(duration)?))
        }
        ["weather", "thunder"] => set_weather(state, WeatherMode::Thunder, None),
        ["weather", "thunder", duration] => set_weather(
            state,
            WeatherMode::Thunder,
            Some(parse_time_ticks(duration)?),
        ),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn success_result(feedback_key: &'static str, broadcast_to_admins: bool) -> CommandResult {
    CommandResult {
        success_count: 1,
        feedback_key,
        broadcast_to_admins,
    }
}
