use super::*;

pub(super) fn publish_server(
    state: &mut ServerCommandState,
    port: u16,
    allow_commands: bool,
    gamemode: Option<GameMode>,
) -> Result<CommandResult, CommandError> {
    if state.published_server.is_some() {
        return Err(CommandError::PublishAlreadyPublished);
    }
    if state.publish_should_fail {
        return Err(CommandError::PublishFailed);
    }
    state.published_server = Some(PublishRequest {
        port,
        allow_commands,
        gamemode,
    });
    Ok(CommandResult {
        success_count: i32::from(port),
        feedback_key: "commands.publish.started",
        broadcast_to_admins: true,
    })
}

pub(super) fn kick_players(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    reason: String,
) -> Result<CommandResult, CommandError> {
    if state.published_server.is_none() {
        return Err(CommandError::KickSingleplayer);
    }

    let mut count = 0;
    for target in targets {
        if state
            .singleplayer_owner
            .as_ref()
            .is_some_and(|owner| owner.uuid == target.uuid)
        {
            continue;
        }
        state.disconnected_players.push(PlayerDisconnect {
            player: target,
            reason: reason.clone(),
        });
        count += 1;
    }

    if count == 0 {
        Err(CommandError::KickOwner)
    } else {
        Ok(CommandResult {
            success_count: count,
            feedback_key: "commands.kick.success",
            broadcast_to_admins: true,
        })
    }
}

pub(super) fn ban_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (targets, reason) = ban_targets_and_optional_reason(&parts[1..])?;
    let mut count = 0;
    for target in targets {
        let profile = NameAndId::create_offline(target);
        if state.add_player_ban(profile.clone(), reason.clone()) {
            state
                .ban_player_feedback_events
                .push(BanPlayerFeedbackEvent {
                    player: profile.clone(),
                    feedback_key: "commands.ban.success",
                    broadcast_to_admins: true,
                });
            if state
                .online_players
                .iter()
                .any(|online| online.uuid == profile.uuid)
            {
                state.disconnected_players.push(PlayerDisconnect {
                    player: profile,
                    reason: "multiplayer.disconnect.banned".to_string(),
                });
            }
            count += 1;
        }
    }

    if count == 0 {
        Err(CommandError::BanFailed)
    } else {
        Ok(CommandResult {
            success_count: count,
            feedback_key: "commands.ban.success",
            broadcast_to_admins: true,
        })
    }
}

pub(super) fn ban_targets_and_optional_reason<'a>(
    parts: &'a [&'a str],
) -> Result<(Vec<&'a str>, Option<String>), CommandError> {
    if parts.contains(&"--") {
        return targets_and_optional_reason(parts);
    }
    match parts {
        [] => Err(CommandError::InvalidSyntax),
        [target] => Ok((vec![*target], None)),
        [target, reason @ ..] => Ok((vec![*target], Some(reason.join(" ")))),
    }
}

pub(super) fn ban_ip_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let target = parts[1];
    let reason = if parts.len() > 2 {
        Some(parts[2..].join(" "))
    } else {
        None
    };
    let ip = if is_ip_address(target) {
        target.to_string()
    } else {
        state
            .online_ip_for_name(target)
            .ok_or(CommandError::BanIpInvalid)?
    };
    if !state.add_ip_ban(ip.clone(), reason) {
        return Err(CommandError::BanIpFailed);
    }
    state.ban_ip_feedback_events.push(BanIpFeedbackEvent {
        feedback_key: "commands.banip.success",
        broadcast_to_admins: true,
    });
    let players = state.players_with_ip(&ip);
    for player in &players {
        state.disconnected_players.push(PlayerDisconnect {
            player: player.clone(),
            reason: "multiplayer.disconnect.ip_banned".to_string(),
        });
    }
    Ok(CommandResult {
        success_count: players.len() as i32,
        feedback_key: if players.is_empty() {
            "commands.banip.success"
        } else {
            "commands.banip.info"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn banlist_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let count = match parts {
        ["banlist"] => state.banned_players.len() + state.banned_ips.len(),
        ["banlist", "players"] => state.banned_players.len(),
        ["banlist", "ips"] => state.banned_ips.len(),
        _ => return Err(CommandError::InvalidSyntax),
    };
    Ok(CommandResult {
        success_count: count as i32,
        feedback_key: if count == 0 {
            "commands.banlist.none"
        } else {
            "commands.banlist.list"
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn pardon_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let pardoned = parts[1..]
        .iter()
        .map(|target| NameAndId::create_offline(target))
        .filter(|profile| state.remove_player_ban(profile))
        .collect::<Vec<_>>();
    if pardoned.is_empty() {
        Err(CommandError::PardonFailed)
    } else {
        push_extra_admin_feedback(state, pardoned.len(), "commands.pardon.success");
        Ok(CommandResult {
            success_count: pardoned.len() as i32,
            feedback_key: "commands.pardon.success",
            broadcast_to_admins: true,
        })
    }
}

fn push_extra_admin_feedback(
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

pub(super) fn pardon_ip_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["pardon-ip", ip] if is_ip_address(ip) => {
            if !state.remove_ip_ban(ip) {
                return Err(CommandError::PardonIpFailed);
            }
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.pardonip.success",
                broadcast_to_admins: true,
            })
        }
        ["pardon-ip", _] => Err(CommandError::PardonIpInvalid),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn targets_and_optional_reason<'a>(
    parts: &'a [&'a str],
) -> Result<(Vec<&'a str>, Option<String>), CommandError> {
    if parts.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let split = parts
        .iter()
        .position(|part| *part == "--")
        .unwrap_or(parts.len());
    let targets = parts[..split].to_vec();
    if targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let reason = if split < parts.len() {
        if split + 1 >= parts.len() {
            return Err(CommandError::InvalidSyntax);
        }
        Some(parts[split + 1..].join(" "))
    } else {
        None
    };
    Ok((targets, reason))
}

pub(super) fn is_ip_address(value: &str) -> bool {
    value.parse::<IpAddr>().is_ok()
}

pub(super) fn bossbar_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["bossbar", "add", id, name @ ..] if !name.is_empty() => {
            let id = parse_resource_identifier(id)?;
            if state.bossbars.iter().any(|bar| bar.id == id) {
                return Err(CommandError::BossBarAlreadyExists);
            }
            state.bossbars.push(CustomBossBar {
                id,
                name: name.join(" "),
                color: BossBarCommandColor::White,
                overlay: BossBarCommandOverlay::Progress,
                value: 0,
                max: 100,
                visible: true,
                players: Vec::new(),
            });
            Ok(CommandResult {
                success_count: state.bossbars.len() as i32,
                feedback_key: "commands.bossbar.create.success",
                broadcast_to_admins: true,
            })
        }
        ["bossbar", "remove", id] => {
            let id = parse_resource_identifier(id)?;
            let index = state
                .bossbars
                .iter()
                .position(|bar| bar.id == id)
                .ok_or(CommandError::BossBarUnknown)?;
            state.bossbars.remove(index);
            Ok(CommandResult {
                success_count: state.bossbars.len() as i32,
                feedback_key: "commands.bossbar.remove.success",
                broadcast_to_admins: true,
            })
        }
        ["bossbar", "list"] => Ok(CommandResult {
            success_count: state.bossbars.len() as i32,
            feedback_key: if state.bossbars.is_empty() {
                "commands.bossbar.list.bars.none"
            } else {
                "commands.bossbar.list.bars.some"
            },
            broadcast_to_admins: false,
        }),
        ["bossbar", "get", id, property] => {
            let id = parse_resource_identifier(id)?;
            let bar = bossbar(state, &id)?;
            match *property {
                "value" => Ok(CommandResult {
                    success_count: bar.value,
                    feedback_key: "commands.bossbar.get.value",
                    broadcast_to_admins: true,
                }),
                "max" => Ok(CommandResult {
                    success_count: bar.max,
                    feedback_key: "commands.bossbar.get.max",
                    broadcast_to_admins: true,
                }),
                "visible" => Ok(CommandResult {
                    success_count: i32::from(bar.visible),
                    feedback_key: if bar.visible {
                        "commands.bossbar.get.visible.visible"
                    } else {
                        "commands.bossbar.get.visible.hidden"
                    },
                    broadcast_to_admins: true,
                }),
                "players" => Ok(CommandResult {
                    success_count: bar.players.len() as i32,
                    feedback_key: if bar.players.is_empty() {
                        "commands.bossbar.get.players.none"
                    } else {
                        "commands.bossbar.get.players.some"
                    },
                    broadcast_to_admins: true,
                }),
                _ => Err(CommandError::InvalidSyntax),
            }
        }
        ["bossbar", "set", id, property, rest @ ..] => {
            let id = parse_resource_identifier(id)?;
            bossbar_set_command(state, &id, property, rest)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn bossbar_set_command(
    state: &mut ServerCommandState,
    id: &str,
    property: &str,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match property {
        "name" if !parts.is_empty() => {
            let bar = bossbar_mut(state, id)?;
            let name = parts.join(" ");
            if bar.name == name {
                return Err(CommandError::BossBarNameUnchanged);
            }
            bar.name = name;
            Ok(bossbar_set_result(0, "commands.bossbar.set.name.success"))
        }
        "color" if parts.len() == 1 => {
            let color = parse_bossbar_color(parts[0])?;
            let bar = bossbar_mut(state, id)?;
            if bar.color == color {
                return Err(CommandError::BossBarColorUnchanged);
            }
            bar.color = color;
            Ok(bossbar_set_result(0, "commands.bossbar.set.color.success"))
        }
        "style" if parts.len() == 1 => {
            let overlay = parse_bossbar_overlay(parts[0])?;
            let bar = bossbar_mut(state, id)?;
            if bar.overlay == overlay {
                return Err(CommandError::BossBarStyleUnchanged);
            }
            bar.overlay = overlay;
            Ok(bossbar_set_result(0, "commands.bossbar.set.style.success"))
        }
        "value" if parts.len() == 1 => {
            let value = parse_i32(parts[0])?;
            if value < 0 {
                return Err(CommandError::InvalidSyntax);
            }
            let bar = bossbar_mut(state, id)?;
            if bar.value == value {
                return Err(CommandError::BossBarValueUnchanged);
            }
            bar.value = value;
            Ok(bossbar_set_result(
                value,
                "commands.bossbar.set.value.success",
            ))
        }
        "max" if parts.len() == 1 => {
            let max = parse_i32(parts[0])?;
            if max < 1 {
                return Err(CommandError::InvalidSyntax);
            }
            let bar = bossbar_mut(state, id)?;
            if bar.max == max {
                return Err(CommandError::BossBarMaxUnchanged);
            }
            bar.max = max;
            Ok(bossbar_set_result(max, "commands.bossbar.set.max.success"))
        }
        "visible" if parts.len() == 1 => {
            let visible = parse_bool(parts[0])?;
            let bar = bossbar_mut(state, id)?;
            if bar.visible == visible {
                return Err(if visible {
                    CommandError::BossBarAlreadyVisible
                } else {
                    CommandError::BossBarAlreadyHidden
                });
            }
            bar.visible = visible;
            Ok(bossbar_set_result(
                0,
                if visible {
                    "commands.bossbar.set.visible.success.visible"
                } else {
                    "commands.bossbar.set.visible.success.hidden"
                },
            ))
        }
        "players" => {
            let players = resolve_bossbar_players(&state.online_players, parts)?;
            let bar = bossbar_mut(state, id)?;
            if same_players(&bar.players, &players) {
                return Err(CommandError::BossBarPlayersUnchanged);
            }
            bar.players = players;
            Ok(bossbar_set_result(
                bar.players.len() as i32,
                if bar.players.is_empty() {
                    "commands.bossbar.set.players.success.none"
                } else {
                    "commands.bossbar.set.players.success.some"
                },
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn resolve_bossbar_players(
    online_players: &[NameAndId],
    names: &[&str],
) -> Result<Vec<NameAndId>, CommandError> {
    let mut players = Vec::with_capacity(names.len());
    for name in names {
        let player = online_players
            .iter()
            .find(|player| player.name == *name)
            .ok_or(CommandError::NoPlayers)?;
        players.push(player.clone());
    }
    Ok(players)
}

pub(super) fn bossbar_set_result(success_count: i32, feedback_key: &'static str) -> CommandResult {
    CommandResult {
        success_count,
        feedback_key,
        broadcast_to_admins: true,
    }
}

pub(super) fn bossbar<'a>(
    state: &'a ServerCommandState,
    id: &str,
) -> Result<&'a CustomBossBar, CommandError> {
    state
        .bossbars
        .iter()
        .find(|bar| bar.id == id)
        .ok_or(CommandError::BossBarUnknown)
}

pub(super) fn bossbar_mut<'a>(
    state: &'a mut ServerCommandState,
    id: &str,
) -> Result<&'a mut CustomBossBar, CommandError> {
    state
        .bossbars
        .iter_mut()
        .find(|bar| bar.id == id)
        .ok_or(CommandError::BossBarUnknown)
}

pub(super) fn parse_bossbar_color(input: &str) -> Result<BossBarCommandColor, CommandError> {
    match input {
        "pink" => Ok(BossBarCommandColor::Pink),
        "blue" => Ok(BossBarCommandColor::Blue),
        "red" => Ok(BossBarCommandColor::Red),
        "green" => Ok(BossBarCommandColor::Green),
        "yellow" => Ok(BossBarCommandColor::Yellow),
        "purple" => Ok(BossBarCommandColor::Purple),
        "white" => Ok(BossBarCommandColor::White),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_bossbar_overlay(input: &str) -> Result<BossBarCommandOverlay, CommandError> {
    match input {
        "progress" => Ok(BossBarCommandOverlay::Progress),
        "notched_6" => Ok(BossBarCommandOverlay::Notched6),
        "notched_10" => Ok(BossBarCommandOverlay::Notched10),
        "notched_12" => Ok(BossBarCommandOverlay::Notched12),
        "notched_20" => Ok(BossBarCommandOverlay::Notched20),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn same_players(left: &[NameAndId], right: &[NameAndId]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.uuid == right.uuid)
}

pub(super) fn chase_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["chase", "follow"] => start_chase_follow(state, "localhost", 10000),
        ["chase", "follow", host] => start_chase_follow(state, host, 10000),
        ["chase", "follow", host, port] => {
            start_chase_follow(state, host, parse_chase_port(port, 1)?)
        }
        ["chase", "lead"] => start_chase_lead(state, "0.0.0.0", 10000),
        ["chase", "lead", bind_address] => start_chase_lead(state, bind_address, 10000),
        ["chase", "lead", bind_address, port] => {
            start_chase_lead(state, bind_address, parse_chase_port(port, 1024)?)
        }
        ["chase", "stop"] => {
            if let Some(session) = state.chase_session.take() {
                match session {
                    ChaseSession::Leading { .. } => {
                        state.chase_events.push(ChaseEvent::LeadStopped)
                    }
                    ChaseSession::Following { .. } => {
                        state.chase_events.push(ChaseEvent::FollowStopped)
                    }
                }
            }
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.chase.stop",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn start_chase_follow(
    state: &mut ServerCommandState,
    host: &str,
    port: u16,
) -> Result<CommandResult, CommandError> {
    if state.chase_session.is_some() {
        return Err(CommandError::ChaseAlreadyRunning);
    }
    state.chase_session = Some(ChaseSession::Following {
        host: host.to_string(),
        port,
    });
    state.chase_events.push(ChaseEvent::FollowStarted {
        host: host.to_string(),
        port,
    });
    Ok(CommandResult {
        success_count: 0,
        feedback_key: "commands.chase.follow.success",
        broadcast_to_admins: false,
    })
}

pub(super) fn start_chase_lead(
    state: &mut ServerCommandState,
    bind_address: &str,
    port: u16,
) -> Result<CommandResult, CommandError> {
    if state.chase_session.is_some() {
        return Err(CommandError::ChaseAlreadyRunning);
    }
    state.chase_session = Some(ChaseSession::Leading {
        bind_address: bind_address.to_string(),
        port,
    });
    state.chase_events.push(ChaseEvent::LeadStarted {
        bind_address: bind_address.to_string(),
        port,
    });
    Ok(CommandResult {
        success_count: 0,
        feedback_key: "commands.chase.lead.success",
        broadcast_to_admins: false,
    })
}

pub(super) fn parse_chase_port(input: &str, min: u16) -> Result<u16, CommandError> {
    let port = input
        .parse::<u16>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if port < min {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(port)
    }
}
