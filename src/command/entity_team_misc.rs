use super::*;

pub(super) fn swing_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (targets, hand) = match parts {
        ["swing"] => (
            vec![state
                .command_source_entity
                .clone()
                .ok_or(CommandError::InvalidSyntax)?],
            InteractionHand::MainHand,
        ),
        ["swing", targets] => (parse_entity_list(targets), InteractionHand::MainHand),
        ["swing", targets, "mainhand"] => (parse_entity_list(targets), InteractionHand::MainHand),
        ["swing", targets, "offhand"] => (parse_entity_list(targets), InteractionHand::OffHand),
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() {
        return Err(CommandError::SwingNoLivingEntity);
    }

    let mut success = 0;
    for target in targets {
        if entity_kind(state, &target) == EntityKind::NonLiving {
            continue;
        }
        state.swing_events.push(SwingCommandEvent { target, hand });
        success += 1;
    }

    if success == 0 {
        return Err(CommandError::SwingNoLivingEntity);
    }

    Ok(CommandResult {
        success_count: success,
        feedback_key: if success == 1 {
            "commands.swing.success.single"
        } else {
            "commands.swing.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn tag_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["tag", targets, "add", name] => add_entity_tag(state, parse_entity_list(targets), name),
        ["tag", targets, "remove", name] => {
            remove_entity_tag(state, parse_entity_list(targets), name)
        }
        ["tag", targets, "list"] => list_entity_tags(state, parse_entity_list(targets)),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn add_entity_tag(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    name: &str,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() || name.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let target_count = targets.len();
    let mut success = 0;
    for target in targets {
        let index = entity_tags_index(state, target);
        if !state.entity_tags[index].tags.iter().any(|tag| tag == name) {
            state.entity_tags[index].tags.push(name.to_string());
            success += 1;
        }
    }
    if success == 0 {
        return Err(CommandError::TagAddFailed);
    }
    Ok(CommandResult {
        success_count: success,
        feedback_key: if target_count == 1 {
            "commands.tag.add.success.single"
        } else {
            "commands.tag.add.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn remove_entity_tag(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    name: &str,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() || name.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let target_count = targets.len();
    let mut success = 0;
    for target in targets {
        if let Some(index) = state
            .entity_tags
            .iter()
            .position(|entry| entry.entity.id == target.id)
        {
            let old_len = state.entity_tags[index].tags.len();
            state.entity_tags[index].tags.retain(|tag| tag != name);
            if state.entity_tags[index].tags.len() != old_len {
                success += 1;
            }
        }
    }
    if success == 0 {
        return Err(CommandError::TagRemoveFailed);
    }
    Ok(CommandResult {
        success_count: success,
        feedback_key: if target_count == 1 {
            "commands.tag.remove.success.single"
        } else {
            "commands.tag.remove.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn list_entity_tags(
    state: &ServerCommandState,
    targets: Vec<EntityRef>,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let mut tags = Vec::<String>::new();
    for target in &targets {
        if let Some(entry) = state
            .entity_tags
            .iter()
            .find(|entry| entry.entity.id == target.id)
        {
            for tag in &entry.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }
    }
    let empty = tags.is_empty();
    Ok(CommandResult {
        success_count: tags.len() as i32,
        feedback_key: match (targets.len(), empty) {
            (1, true) => "commands.tag.list.single.empty",
            (1, false) => "commands.tag.list.single.success",
            (_, true) => "commands.tag.list.multiple.empty",
            (_, false) => "commands.tag.list.multiple.success",
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn entity_tags_index(state: &mut ServerCommandState, entity: EntityRef) -> usize {
    if let Some(index) = state
        .entity_tags
        .iter()
        .position(|entry| entry.entity.id == entity.id)
    {
        index
    } else {
        state.entity_tags.push(EntityTags {
            entity,
            tags: Vec::new(),
        });
        state.entity_tags.len() - 1
    }
}

pub(super) fn team_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["team", "list"] => Ok(CommandResult {
            success_count: state.teams.len() as i32,
            feedback_key: if state.teams.is_empty() {
                "commands.team.list.teams.empty"
            } else {
                "commands.team.list.teams.success"
            },
            broadcast_to_admins: false,
        }),
        ["team", "list", team] => {
            require_team(state, team)?;
            let count = state
                .player_teams
                .iter()
                .filter(|membership| membership.team == *team)
                .count();
            Ok(CommandResult {
                success_count: count as i32,
                feedback_key: if count == 0 {
                    "commands.team.list.members.empty"
                } else {
                    "commands.team.list.members.success"
                },
                broadcast_to_admins: false,
            })
        }
        ["team", "add", team] => add_team(state, team, team),
        ["team", "add", team, display_name] => add_team(state, team, display_name),
        ["team", "remove", team] => {
            require_team(state, team)?;
            state.teams.retain(|entry| entry.name != *team);
            state
                .player_teams
                .retain(|membership| membership.team != *team);
            Ok(CommandResult {
                success_count: state.teams.len() as i32,
                feedback_key: "commands.team.remove.success",
                broadcast_to_admins: true,
            })
        }
        ["team", "empty", team] => {
            require_team(state, team)?;
            let old_len = state.player_teams.len();
            state
                .player_teams
                .retain(|membership| membership.team != *team);
            let removed = old_len - state.player_teams.len();
            if removed == 0 {
                return Err(CommandError::TeamAlreadyEmpty);
            }
            Ok(CommandResult {
                success_count: removed as i32,
                feedback_key: "commands.team.empty.success",
                broadcast_to_admins: true,
            })
        }
        ["team", "join", team] => {
            let player = state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            join_team(state, team, vec![player])
        }
        ["team", "join", team, members @ ..] if !members.is_empty() => {
            let members = members
                .iter()
                .map(|member| NameAndId::create_offline(member))
                .collect();
            join_team(state, team, members)
        }
        ["team", "leave", members @ ..] if !members.is_empty() => {
            let members: Vec<NameAndId> = members
                .iter()
                .map(|member| NameAndId::create_offline(member))
                .collect();
            for member in &members {
                state
                    .player_teams
                    .retain(|membership| membership.player.uuid != member.uuid);
            }
            Ok(CommandResult {
                success_count: members.len() as i32,
                feedback_key: if members.len() == 1 {
                    "commands.team.leave.success.single"
                } else {
                    "commands.team.leave.success.multiple"
                },
                broadcast_to_admins: true,
            })
        }
        ["team", "modify", team, option, value] => modify_team(state, team, option, value),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn add_team(
    state: &mut ServerCommandState,
    team: &str,
    display_name: &str,
) -> Result<CommandResult, CommandError> {
    parse_identifier(team)?;
    if state.teams.iter().any(|entry| entry.name == team) {
        return Err(CommandError::TeamAlreadyExists);
    }
    state
        .teams
        .push(TeamState::new(team.to_string(), display_name.to_string()));
    Ok(CommandResult {
        success_count: state.teams.len() as i32,
        feedback_key: "commands.team.add.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn join_team(
    state: &mut ServerCommandState,
    team: &str,
    members: Vec<NameAndId>,
) -> Result<CommandResult, CommandError> {
    require_team(state, team)?;
    for member in &members {
        state
            .player_teams
            .retain(|membership| membership.player.uuid != member.uuid);
        state.player_teams.push(TeamMembership {
            player: member.clone(),
            team: team.to_string(),
        });
    }
    Ok(CommandResult {
        success_count: members.len() as i32,
        feedback_key: if members.len() == 1 {
            "commands.team.join.success.single"
        } else {
            "commands.team.join.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn modify_team(
    state: &mut ServerCommandState,
    team: &str,
    option: &str,
    value: &str,
) -> Result<CommandResult, CommandError> {
    let team = state
        .teams
        .iter_mut()
        .find(|entry| entry.name == team)
        .ok_or(CommandError::TeamNotFound)?;
    let feedback_key = match option {
        "displayName" => set_team_string(
            &mut team.display_name,
            value,
            "commands.team.option.name.success",
        )?,
        "color" => set_team_string(&mut team.color, value, "commands.team.option.color.success")?,
        "friendlyFire" => {
            let value = parse_bool(value)?;
            set_team_bool(
                &mut team.friendly_fire,
                value,
                if value {
                    "commands.team.option.friendlyfire.enabled"
                } else {
                    "commands.team.option.friendlyfire.disabled"
                },
            )?
        }
        "seeFriendlyInvisibles" => {
            let value = parse_bool(value)?;
            set_team_bool(
                &mut team.see_friendly_invisibles,
                value,
                if value {
                    "commands.team.option.seeFriendlyInvisibles.enabled"
                } else {
                    "commands.team.option.seeFriendlyInvisibles.disabled"
                },
            )?
        }
        "nametagVisibility" => {
            validate_team_visibility(value)?;
            set_team_string(
                &mut team.nametag_visibility,
                value,
                "commands.team.option.nametagVisibility.success",
            )?
        }
        "deathMessageVisibility" => {
            validate_team_visibility(value)?;
            set_team_string(
                &mut team.death_message_visibility,
                value,
                "commands.team.option.deathMessageVisibility.success",
            )?
        }
        "collisionRule" => {
            validate_team_collision(value)?;
            set_team_string(
                &mut team.collision_rule,
                value,
                "commands.team.option.collisionRule.success",
            )?
        }
        "prefix" => set_team_string(
            &mut team.prefix,
            value,
            "commands.team.option.prefix.success",
        )?,
        "suffix" => set_team_string(
            &mut team.suffix,
            value,
            "commands.team.option.suffix.success",
        )?,
        _ => return Err(CommandError::InvalidSyntax),
    };
    Ok(CommandResult {
        success_count: 0,
        feedback_key,
        broadcast_to_admins: true,
    })
}

pub(super) fn require_team(state: &ServerCommandState, team: &str) -> Result<(), CommandError> {
    state
        .teams
        .iter()
        .any(|entry| entry.name == team)
        .then_some(())
        .ok_or(CommandError::TeamNotFound)
}

pub(super) fn set_team_string(
    current: &mut String,
    value: &str,
    feedback_key: &'static str,
) -> Result<&'static str, CommandError> {
    if current == value {
        return Err(CommandError::TeamOptionUnchanged);
    }
    *current = value.to_string();
    Ok(feedback_key)
}

pub(super) fn set_team_bool(
    current: &mut bool,
    value: bool,
    feedback_key: &'static str,
) -> Result<&'static str, CommandError> {
    if *current == value {
        return Err(CommandError::TeamOptionUnchanged);
    }
    *current = value;
    Ok(feedback_key)
}

pub(super) fn validate_team_visibility(value: &str) -> Result<(), CommandError> {
    match value {
        "always" | "never" | "hideForOtherTeams" | "hideForOwnTeam" => Ok(()),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn validate_team_collision(value: &str) -> Result<(), CommandError> {
    match value {
        "always" | "never" | "pushOwnTeam" | "pushOtherTeams" => Ok(()),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn particle_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 {
        return Err(CommandError::InvalidSyntax);
    }
    let name = parts[1].to_string();
    let mut index = 2;
    let position = if parts.len() >= index + 3 && is_number(parts[index]) {
        let pos = Vec3 {
            x: parse_f64(parts[index])?,
            y: parse_f64(parts[index + 1])?,
            z: parse_f64(parts[index + 2])?,
        };
        index += 3;
        pos
    } else {
        Vec3::default()
    };

    let mut delta = Vec3::default();
    let mut speed = 0.0;
    let mut count = 0;
    if parts.len() > index {
        if parts.len() < index + 5 {
            return Err(CommandError::InvalidSyntax);
        }
        delta = Vec3 {
            x: parse_f64(parts[index])?,
            y: parse_f64(parts[index + 1])?,
            z: parse_f64(parts[index + 2])?,
        };
        speed = parse_non_negative_f32(parts[index + 3])?;
        count = parts[index + 4]
            .parse::<u32>()
            .map_err(|_| CommandError::InvalidSyntax)?;
        index += 5;
    }

    let mut force = false;
    if parts
        .get(index)
        .is_some_and(|mode| *mode == "force" || *mode == "normal")
    {
        force = parts[index] == "force";
        index += 1;
    }
    let viewers = if let Some(viewers) = parts.get(index) {
        index += 1;
        parse_name_list(viewers)
    } else {
        state.online_players.clone()
    };
    if index != parts.len() {
        return Err(CommandError::InvalidSyntax);
    }
    if viewers.is_empty() {
        return Err(CommandError::ParticleFailed);
    }

    let success_count = viewers.len() as i32;
    state.particle_events.push(ParticleCommandEvent {
        name,
        viewers,
        position,
        delta,
        speed,
        count,
        force,
    });
    Ok(CommandResult {
        success_count,
        feedback_key: "commands.particle.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn server_pack_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["serverpack", "push", url] => push_server_pack(state, url, None, None),
        ["serverpack", "push", url, id] => push_server_pack(state, url, Some(*id), None),
        ["serverpack", "push", url, id, hash] => {
            push_server_pack(state, url, Some(*id), Some(*hash))
        }
        ["serverpack", "pop", id] => {
            let id = parse_uuid_string(id)?;
            state
                .server_pack_events
                .push(ServerPackCommandEvent::Pop { id });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn push_server_pack(
    state: &mut ServerCommandState,
    url: &str,
    id: Option<&str>,
    hash: Option<&str>,
) -> Result<CommandResult, CommandError> {
    let id = match id {
        Some(id) => parse_uuid_string(id)?,
        None => name_uuid_from_bytes(url.as_bytes()),
    };
    state
        .server_pack_events
        .push(ServerPackCommandEvent::Push(ServerPackPushRequest {
            id,
            url: url.to_string(),
            hash: hash.unwrap_or_default().to_string(),
            required: false,
            prompt: None,
        }));
    Ok(CommandResult {
        success_count: 0,
        feedback_key: NO_COMMAND_FEEDBACK,
        broadcast_to_admins: false,
    })
}

pub(super) fn perf_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["perf", "start"] => {
            if state.perf_recording {
                return Err(CommandError::PerfAlreadyRunning);
            }
            state.perf_recording = true;
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.started",
                broadcast_to_admins: false,
            })
        }
        ["perf", "stop"] => {
            if !state.perf_recording {
                return Err(CommandError::PerfNotRunning);
            }
            state.perf_recording = false;
            state.perf_reports.push(PerfReport {
                ticks: state.tick_time_samples_nanos.len() as u32,
                duration_nanos: state.tick_time_samples_nanos.iter().sum(),
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.perf.stopped",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn jfr_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["jfr", "start"] => {
            if state.jfr_recording {
                return Err(CommandError::JfrStartFailed);
            }
            state.jfr_recording = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.jfr.started",
                broadcast_to_admins: false,
            })
        }
        ["jfr", "stop"] => {
            if !state.jfr_recording {
                return Err(CommandError::JfrDumpFailed);
            }
            state.jfr_recording = false;
            state
                .jfr_recordings
                .push(state.next_jfr_recording_path.clone());
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.jfr.stopped",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn recipe_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (mode, targets, recipes) = match parts {
        ["recipe", mode @ ("give" | "take"), targets, "*"] => {
            (*mode, parse_name_list(targets), state.known_recipes.clone())
        }
        ["recipe", mode @ ("give" | "take"), targets, recipe] => {
            (*mode, parse_name_list(targets), vec![(*recipe).to_string()])
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() || recipes.is_empty() {
        return Err(match mode {
            "give" => CommandError::RecipeGiveFailed,
            "take" => CommandError::RecipeTakeFailed,
            _ => CommandError::InvalidSyntax,
        });
    }

    let mut success = 0;
    for target in &targets {
        let index = recipe_book_index(state, target);
        match mode {
            "give" => {
                for recipe in &recipes {
                    if !state.player_recipes[index].recipes.contains(recipe) {
                        state.player_recipes[index].recipes.push(recipe.clone());
                        success += 1;
                    }
                }
            }
            "take" => {
                for recipe in &recipes {
                    let old_len = state.player_recipes[index].recipes.len();
                    state.player_recipes[index]
                        .recipes
                        .retain(|known| known != recipe);
                    if state.player_recipes[index].recipes.len() != old_len {
                        success += 1;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    if success == 0 {
        return Err(match mode {
            "give" => CommandError::RecipeGiveFailed,
            "take" => CommandError::RecipeTakeFailed,
            _ => CommandError::InvalidSyntax,
        });
    }

    Ok(CommandResult {
        success_count: success,
        feedback_key: match (mode, targets.len()) {
            ("give", 1) => "commands.recipe.give.success.single",
            ("give", _) => "commands.recipe.give.success.multiple",
            ("take", 1) => "commands.recipe.take.success.single",
            ("take", _) => "commands.recipe.take.success.multiple",
            _ => unreachable!(),
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn recipe_book_index(state: &mut ServerCommandState, player: &NameAndId) -> usize {
    if let Some(index) = state
        .player_recipes
        .iter()
        .position(|book| book.player.uuid == player.uuid)
    {
        index
    } else {
        state.player_recipes.push(PlayerRecipeBook {
            player: player.clone(),
            recipes: Vec::new(),
        });
        state.player_recipes.len() - 1
    }
}

pub(super) fn rotate_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 4 {
        return Err(CommandError::InvalidSyntax);
    }
    let target = entity_ref(parts[1]);
    let mode = match parts {
        ["rotate", _target, "facing", "entity", facing_entity] => RotationMode::FacingEntity {
            entity: entity_ref(facing_entity),
            anchor: EntityAnchor::Feet,
        },
        ["rotate", _target, "facing", "entity", facing_entity, anchor] => {
            RotationMode::FacingEntity {
                entity: entity_ref(facing_entity),
                anchor: parse_entity_anchor(anchor)?,
            }
        }
        ["rotate", _target, "facing", x, y, z] => RotationMode::FacingPosition(Vec3 {
            x: parse_f64(x)?,
            y: parse_f64(y)?,
            z: parse_f64(z)?,
        }),
        ["rotate", _target, yaw, pitch] => {
            let yaw = parse_rotation_component(yaw)?;
            let pitch = parse_rotation_component(pitch)?;
            RotationMode::Angles {
                yaw: yaw.value,
                pitch: pitch.value,
                yaw_relative: yaw.relative,
                pitch_relative: pitch.relative,
            }
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    state
        .rotation_requests
        .push(RotationRequest { target, mode });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.rotate.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn return_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["return", "fail"] => {
            state.return_events.push(ReturnCommandEvent::Failure {
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        ["return", "run", command @ ..] if !command.is_empty() => {
            let command = command.join(" ");
            state.return_events.push(ReturnCommandEvent::Run {
                command,
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: 0,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        ["return", value] => {
            let value = parse_i32(value)?;
            state.return_events.push(ReturnCommandEvent::Success {
                value,
                discard_frame: true,
            });
            Ok(CommandResult {
                success_count: value,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn ride_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["ride", target, "mount", vehicle] => {
            mount_entity(state, entity_ref(target), entity_ref(vehicle))
        }
        ["ride", target, "dismount"] => dismount_entity(state, entity_ref(target)),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn mount_entity(
    state: &mut ServerCommandState,
    target: EntityRef,
    vehicle: EntityRef,
) -> Result<CommandResult, CommandError> {
    if mounted_vehicle(state, &target).is_some() {
        return Err(CommandError::RideAlreadyRiding);
    }
    if entity_kind(state, &vehicle) == EntityKind::Player {
        return Err(CommandError::RideMountingPlayer);
    }
    if is_self_or_passenger_of(state, &target, &vehicle) {
        return Err(CommandError::RideMountingLoop);
    }
    if entity_dimension(state, &target) != entity_dimension(state, &vehicle) {
        return Err(CommandError::RideWrongDimension);
    }
    if state
        .ride_mount_failures
        .iter()
        .any(|failure| failure.target.id == target.id && failure.vehicle.id == vehicle.id)
    {
        return Err(CommandError::RideMountFailed);
    }

    let mount = EntityMount {
        target: target.clone(),
        vehicle: vehicle.clone(),
    };
    state.entity_mounts.push(mount);
    state
        .ride_events
        .push(RideCommandEvent::Mount { target, vehicle });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.ride.mount.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn dismount_entity(
    state: &mut ServerCommandState,
    target: EntityRef,
) -> Result<CommandResult, CommandError> {
    let Some(index) = state
        .entity_mounts
        .iter()
        .position(|mount| mount.target.id == target.id)
    else {
        return Err(CommandError::RideNotRiding);
    };
    let mount = state.entity_mounts.remove(index);
    state.ride_events.push(RideCommandEvent::Dismount {
        target: mount.target,
        vehicle: mount.vehicle,
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.ride.dismount.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn mounted_vehicle<'a>(
    state: &'a ServerCommandState,
    target: &EntityRef,
) -> Option<&'a EntityRef> {
    state
        .entity_mounts
        .iter()
        .find(|mount| mount.target.id == target.id)
        .map(|mount| &mount.vehicle)
}

pub(super) fn is_self_or_passenger_of(
    state: &ServerCommandState,
    target: &EntityRef,
    candidate: &EntityRef,
) -> bool {
    if target.id == candidate.id {
        return true;
    }
    let mut current = candidate;
    let mut depth = 0;
    while let Some(vehicle) = mounted_vehicle(state, current) {
        if vehicle.id == target.id {
            return true;
        }
        current = vehicle;
        depth += 1;
        if depth > state.entity_mounts.len() {
            break;
        }
    }
    false
}

pub(super) fn entity_kind(state: &ServerCommandState, entity: &EntityRef) -> EntityKind {
    if let Some(kind) = state
        .entity_states
        .iter()
        .find(|known| known.entity.id == entity.id)
        .map(|known| known.kind)
    {
        return kind;
    }
    if state
        .online_players
        .iter()
        .any(|player| player.name == entity.id)
    {
        EntityKind::Player
    } else {
        EntityKind::Generic
    }
}

pub(super) fn entity_dimension(state: &ServerCommandState, entity: &EntityRef) -> String {
    state
        .entity_states
        .iter()
        .find(|known| known.entity.id == entity.id)
        .map(|known| known.dimension.clone())
        .unwrap_or_else(|| "minecraft:overworld".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RotationComponent {
    value: f32,
    relative: bool,
}

pub(super) fn parse_rotation_component(input: &str) -> Result<RotationComponent, CommandError> {
    if let Some(rest) = input.strip_prefix('~') {
        Ok(RotationComponent {
            value: if rest.is_empty() {
                0.0
            } else {
                rest.parse::<f32>()
                    .map_err(|_| CommandError::InvalidSyntax)?
            },
            relative: true,
        })
    } else {
        Ok(RotationComponent {
            value: input
                .parse::<f32>()
                .map_err(|_| CommandError::InvalidSyntax)?,
            relative: false,
        })
    }
}

pub(super) fn parse_entity_anchor(input: &str) -> Result<EntityAnchor, CommandError> {
    match input {
        "feet" => Ok(EntityAnchor::Feet),
        "eyes" => Ok(EntityAnchor::Eyes),
        _ => Err(CommandError::InvalidSyntax),
    }
}
