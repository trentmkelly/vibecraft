use super::*;

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

    match command {
        "setidletimeout" => {
            if parts.len() != 2 {
                return Err(CommandError::InvalidSyntax);
            }
            let minutes = parts[1]
                .parse::<u32>()
                .map_err(|_| CommandError::InvalidSyntax)?;
            state.player_idle_timeout_minutes = minutes;
            Ok(CommandResult {
                success_count: minutes as i32,
                feedback_key: if minutes > 0 {
                    "commands.setidletimeout.success"
                } else {
                    "commands.setidletimeout.success.disabled"
                },
                broadcast_to_admins: true,
            })
        }
        "save-all" => {
            let flush = match parts.as_slice() {
                ["save-all"] => false,
                ["save-all", "flush"] => true,
                _ => return Err(CommandError::InvalidSyntax),
            };
            state.save_all_requests.push(SaveAllRequest { flush });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.success",
                broadcast_to_admins: true,
            })
        }
        "save-off" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            if !state.autosave_enabled {
                return Err(CommandError::SaveAlreadyOff);
            }
            state.autosave_enabled = false;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.disabled",
                broadcast_to_admins: true,
            })
        }
        "save-on" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            if state.autosave_enabled {
                return Err(CommandError::SaveAlreadyOn);
            }
            state.autosave_enabled = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.save.enabled",
                broadcast_to_admins: true,
            })
        }
        "stop" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            state.halt_requested = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.stop.stopping",
                broadcast_to_admins: true,
            })
        }
        "help" => match parts.as_slice() {
            ["help"] => Ok(CommandResult {
                success_count: visible_command_usages(permissions).len() as i32,
                feedback_key: "commands.help.success",
                broadcast_to_admins: false,
            }),
            ["help", command @ ..] if !command.is_empty() => {
                let command = command.join(" ");
                let root = command.split_whitespace().next().unwrap_or_default();
                if command_usage(root, permissions).is_some() {
                    Ok(CommandResult {
                        success_count: 1,
                        feedback_key: "commands.help.success",
                        broadcast_to_admins: false,
                    })
                } else {
                    Err(CommandError::HelpFailed)
                }
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "jfr" => jfr_command(state, &parts),
        "advancement" => advancement_command(state, &parts),
        "attribute" => attribute_command(state, &parts),
        "biome" => biome_command(state, &parts),
        "bossbar" => bossbar_command(state, &parts),
        "chase" => chase_command(state, &parts),
        "clear" => clear_command(state, &parts),
        "clone" => clone_command(state, &parts),
        "damage" => damage_command(state, &parts),
        "datapack" => datapack_command(state, &parts, permissions),
        "debug" => debug_command(state, &parts),
        "debugconfig" => debug_config_command(state, &parts),
        "debugmobspawning" => debug_mob_spawning_command(state, &parts),
        "debugpath" => debug_path_command(state, &parts),
        "defaultgamemode" => default_gamemode_command(state, &parts),
        "difficulty" => difficulty_command(state, &parts),
        "dialog" => dialog_command(state, &parts),
        "effect" => effect_command(state, &parts),
        "enchant" => enchant_command(state, &parts),
        "execute" => execute_command(state, permissions, &parts),
        "experience" | "xp" => experience_command(state, &parts),
        "fetchprofile" => fetch_profile_command(state, &parts),
        "fill" => fill_command(state, &parts),
        "fillbiome" => fill_biome_command(state, &parts),
        "forceload" => forceload_command(state, &parts),
        "function" => function_command(state, &parts),
        "gamemode" => gamemode_command(state, &parts),
        "give" => give_command(state, &parts),
        "item" => item_command(state, &parts),
        "locate" => locate_command(state, &parts),
        "loot" => loot_command(state, &parts),
        "place" => place_command(state, &parts),
        "raid" => raid_command(state, &parts),
        "gamerule" => gamerule_command(state, &parts),
        "say" => {
            if parts.len() < 2 {
                return Err(CommandError::InvalidSyntax);
            }
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Say,
                sender: state.command_source_player.clone(),
                targets: state.online_players.clone(),
                message: parts[1..].join(" "),
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.say.success",
                broadcast_to_admins: false,
            })
        }
        "me" => {
            if parts.len() < 2 {
                return Err(CommandError::InvalidSyntax);
            }
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Emote,
                sender: state.command_source_player.clone(),
                targets: state.online_players.clone(),
                message: parts[1..].join(" "),
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.me.success",
                broadcast_to_admins: false,
            })
        }
        "msg" | "tell" | "w" => {
            if parts.len() < 3 {
                return Err(CommandError::InvalidSyntax);
            }
            let split = parts[1..]
                .iter()
                .position(|part| *part == "--")
                .map(|index| index + 1)
                .unwrap_or(2);
            let target_names = &parts[1..split];
            let message_parts = if split < parts.len() && parts[split] == "--" {
                &parts[split + 1..]
            } else {
                &parts[split..]
            };
            if target_names.is_empty() || message_parts.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            let targets = target_names
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .collect::<Vec<_>>();
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Private,
                sender: state.command_source_player.clone(),
                targets,
                message: message_parts.join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.message.display",
                broadcast_to_admins: false,
            })
        }
        "teammsg" | "tm" => {
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
            let targets = state.players_on_team(&team);
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::Team,
                sender: Some(sender),
                targets,
                message: parts[1..].join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.teammsg.success",
                broadcast_to_admins: false,
            })
        }
        "tellraw" => {
            if parts.len() < 3 {
                return Err(CommandError::InvalidSyntax);
            }
            let split = parts[1..]
                .iter()
                .position(|part| *part == "--")
                .map(|index| index + 1)
                .unwrap_or(2);
            let target_names = &parts[1..split];
            let message_parts = if split < parts.len() && parts[split] == "--" {
                &parts[split + 1..]
            } else {
                &parts[split..]
            };
            if target_names.is_empty() || message_parts.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            let targets = target_names
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .collect::<Vec<_>>();
            let count = targets.len() as i32;
            state.chat_events.push(ChatCommandEvent {
                kind: ChatCommandKind::TellRaw,
                sender: state.command_source_player.clone(),
                targets,
                message: message_parts.join(" "),
            });
            Ok(CommandResult {
                success_count: count,
                feedback_key: "commands.tellraw.success",
                broadcast_to_admins: false,
            })
        }
        "playsound" => play_sound_command(state, &parts, permissions),
        "schedule" => schedule_command(state, &parts),
        "scoreboard" => scoreboard_command(state, &parts),
        "stopsound" => stop_sound_command(state, &parts),
        "stopwatch" => stopwatch_command(state, &parts),
        "summon" => summon_command(state, &parts),
        "swing" => swing_command(state, &parts),
        "tag" => tag_command(state, &parts),
        "teleport" | "tp" => teleport_command(state, &parts),
        "team" => team_command(state, &parts),
        "time" => time_command(state, &parts),
        "title" => title_command(state, &parts),
        "particle" => particle_command(state, &parts),
        "perf" => perf_command(state, &parts),
        "rotate" => rotate_command(state, &parts),
        "return" => return_command(state, &parts),
        "ride" => ride_command(state, &parts),
        "seed" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            Ok(CommandResult {
                success_count: state.world_seed as i32,
                feedback_key: "commands.seed.success",
                broadcast_to_admins: false,
            })
        }
        "serverpack" => server_pack_command(state, &parts),
        "setblock" => setblock_command(state, &parts),
        "setworldspawn" => setworldspawn_command(state, &parts),
        "spectate" => spectate_command(state, &parts),
        "spawnpoint" => spawnpoint_command(state, &parts),
        "spawn_armor_trims" => spawn_armor_trims_command(state, &parts),
        "spreadplayers" => spreadplayers_command(state, &parts),
        "version" => {
            if parts.len() != 1 {
                return Err(CommandError::InvalidSyntax);
            }
            Ok(CommandResult {
                success_count: state.version.command_lines().len() as i32,
                feedback_key: "commands.version.header",
                broadcast_to_admins: false,
            })
        }
        "warden_spawn_tracker" => warden_spawn_tracker_command(state, &parts),
        "waypoint" => waypoint_command(state, &parts),
        "worldborder" => worldborder_command(state, &parts),
        "list" => match parts.as_slice() {
            ["list"] => Ok(CommandResult {
                success_count: state.online_players.len() as i32,
                feedback_key: "commands.list.players",
                broadcast_to_admins: false,
            }),
            ["list", "uuids"] => Ok(CommandResult {
                success_count: state.online_players.len() as i32,
                feedback_key: "commands.list.players",
                broadcast_to_admins: false,
            }),
            _ => Err(CommandError::InvalidSyntax),
        },
        "kick" => match parts.as_slice() {
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
        },
        "ban" => ban_command(state, &parts),
        "ban-ip" => ban_ip_command(state, &parts),
        "banlist" => banlist_command(state, &parts),
        "pardon" => pardon_command(state, &parts),
        "pardon-ip" => pardon_ip_command(state, &parts),
        "op" => match parts.as_slice() {
            ["op", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    if state.add_operator(NameAndId::create_offline(target)) {
                        success += 1;
                    }
                }
                if success == 0 {
                    return Err(CommandError::OpFailed);
                }
                Ok(CommandResult {
                    success_count: success,
                    feedback_key: "commands.op.success",
                    broadcast_to_admins: true,
                })
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "deop" => match parts.as_slice() {
            ["deop", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    let profile = NameAndId::create_offline(target);
                    if state.remove_operator(&profile) {
                        success += 1;
                    }
                }
                if success == 0 {
                    return Err(CommandError::DeOpFailed);
                }
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: success,
                    feedback_key: "commands.deop.success",
                    broadcast_to_admins: true,
                })
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "kill" => match parts.as_slice() {
            ["kill"] => {
                let source = state
                    .command_source_entity
                    .clone()
                    .ok_or(CommandError::InvalidSyntax)?;
                kill_entities(state, vec![source])
            }
            ["kill", targets @ ..] if !targets.is_empty() => {
                let targets = targets
                    .iter()
                    .map(|target| EntityRef {
                        id: (*target).to_string(),
                        display_name: (*target).to_string(),
                    })
                    .collect::<Vec<_>>();
                kill_entities(state, targets)
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "whitelist" => match parts.as_slice() {
            ["whitelist", "on"] => {
                if state.whitelist_enabled {
                    return Err(CommandError::WhitelistAlreadyOn);
                }
                state.whitelist_enabled = true;
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.enabled",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "off"] => {
                if !state.whitelist_enabled {
                    return Err(CommandError::WhitelistAlreadyOff);
                }
                state.whitelist_enabled = false;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.disabled",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "list"] => Ok(CommandResult {
                success_count: state.whitelisted_players.len() as i32,
                feedback_key: if state.whitelisted_players.is_empty() {
                    "commands.whitelist.none"
                } else {
                    "commands.whitelist.list"
                },
                broadcast_to_admins: false,
            }),
            ["whitelist", "reload"] => {
                state.whitelist_reload_requests += 1;
                state.kick_unlisted_requests += 1;
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.whitelist.reloaded",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "add", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    if state.add_whitelisted(NameAndId::create_offline(target)) {
                        success += 1;
                    }
                }
                if success == 0 {
                    return Err(CommandError::AlreadyWhitelisted);
                }
                Ok(CommandResult {
                    success_count: success,
                    feedback_key: "commands.whitelist.add.success",
                    broadcast_to_admins: true,
                })
            }
            ["whitelist", "remove", targets @ ..] if !targets.is_empty() => {
                let mut success = 0;
                for target in targets {
                    let profile = NameAndId::create_offline(target);
                    if state.remove_whitelisted(&profile) {
                        success += 1;
                    }
                }
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
            _ => Err(CommandError::InvalidSyntax),
        },
        "tick" => match parts.as_slice() {
            ["tick", "query"] => Ok(CommandResult {
                success_count: state.tick_rate.tick_rate() as i32,
                feedback_key: tick_query_status_key(state),
                broadcast_to_admins: false,
            }),
            ["tick", "rate", rate] => {
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
            ["tick", "freeze"] => {
                state.tick_rate.set_frozen(true);
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.tick.status.frozen",
                    broadcast_to_admins: true,
                })
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
            ["tick", "step", "stop"] => {
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
            ["tick", "step", time] => tick_step(state, parse_time_ticks(time)?),
            ["tick", "sprint", "stop"] => {
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
            ["tick", "sprint", time] => {
                let ticks = parse_time_ticks(time)? as u64;
                let _interrupted = state.tick_rate.request_game_to_sprint(ticks);
                Ok(CommandResult {
                    success_count: 1,
                    feedback_key: "commands.tick.status.sprinting",
                    broadcast_to_admins: true,
                })
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "trigger" => trigger_command(state, &parts),
        "transfer" => match parts.as_slice() {
            ["transfer", host] => {
                let source = state
                    .command_source_player
                    .clone()
                    .ok_or(CommandError::NoPlayers)?;
                queue_transfer(state, host, 25565, vec![source])
            }
            ["transfer", host, port] => {
                let source = state
                    .command_source_player
                    .clone()
                    .ok_or(CommandError::NoPlayers)?;
                queue_transfer(state, host, parse_port(port)?, vec![source])
            }
            ["transfer", host, port, players @ ..] if !players.is_empty() => {
                let targets = players
                    .iter()
                    .map(|player| NameAndId::create_offline(player))
                    .collect::<Vec<_>>();
                queue_transfer(state, host, parse_port(port)?, targets)
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "publish" => match parts.as_slice() {
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
        },
        "random" => match parts.as_slice() {
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
            ["random", "reset", "*"] => {
                require_gamemaster(permissions)?;
                let count = state.random_sequences.len() as i32;
                state.random_sequences.clear();
                Ok(CommandResult {
                    success_count: count,
                    feedback_key: "commands.random.reset.all.success",
                    broadcast_to_admins: false,
                })
            }
            ["random", "reset", "*", seed] => {
                require_gamemaster(permissions)?;
                random_reset_all(state, seed, true, true)
            }
            ["random", "reset", "*", seed, include_world_seed] => {
                require_gamemaster(permissions)?;
                random_reset_all(state, seed, parse_bool(include_world_seed)?, true)
            }
            ["random", "reset", "*", seed, include_world_seed, include_sequence_id] => {
                require_gamemaster(permissions)?;
                random_reset_all(
                    state,
                    seed,
                    parse_bool(include_world_seed)?,
                    parse_bool(include_sequence_id)?,
                )
            }
            ["random", "reset", sequence] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(state, sequence, None, true, true)
            }
            ["random", "reset", sequence, seed] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(state, sequence, Some(parse_i32(seed)?), true, true)
            }
            ["random", "reset", sequence, seed, include_world_seed] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(
                    state,
                    sequence,
                    Some(parse_i32(seed)?),
                    parse_bool(include_world_seed)?,
                    true,
                )
            }
            ["random", "reset", sequence, seed, include_world_seed, include_sequence_id] => {
                require_gamemaster(permissions)?;
                reset_random_sequence(
                    state,
                    sequence,
                    Some(parse_i32(seed)?),
                    parse_bool(include_world_seed)?,
                    parse_bool(include_sequence_id)?,
                )
            }
            _ => Err(CommandError::InvalidSyntax),
        },
        "recipe" => recipe_command(state, &parts),
        "reload" => {
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
        "weather" => match parts.as_slice() {
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
        },
        _ => Err(CommandError::InvalidSyntax),
    }
}
