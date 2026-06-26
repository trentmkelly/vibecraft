use super::*;

pub(super) fn teleport_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let parts = if parts.first() == Some(&"tp") {
        let mut redirected = parts.to_vec();
        redirected[0] = "teleport";
        redirected
    } else {
        parts.to_vec()
    };
    match parts.as_slice() {
        ["teleport", x, y, z] => {
            let target = state
                .command_source_entity
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            teleport_to_pos(
                state,
                vec![target],
                parse_teleport_vec3(state, x, y, z)?,
                None,
            )
        }
        ["teleport", destination] => {
            let target = state
                .command_source_entity
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            let destination = entity_ref(destination);
            teleport_to_entity(state, vec![target], destination)
        }
        ["teleport", targets, x, y, z] => teleport_to_pos(
            state,
            parse_entity_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            None,
        ),
        ["teleport", targets, x, y, z, yaw, pitch] => teleport_to_pos(
            state,
            parse_entity_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            Some(parse_teleport_rotation(yaw, pitch)?),
        ),
        ["teleport", targets, x, y, z, "facing", "entity", facing] => teleport_to_pos_with_facing(
            state,
            parse_entity_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            RotationMode::FacingEntity {
                entity: entity_ref(facing),
                anchor: EntityAnchor::Feet,
            },
        ),
        ["teleport", targets, x, y, z, "facing", "entity", facing, anchor] => {
            teleport_to_pos_with_facing(
                state,
                parse_entity_list(targets),
                parse_teleport_vec3(state, x, y, z)?,
                RotationMode::FacingEntity {
                    entity: entity_ref(facing),
                    anchor: parse_entity_anchor(anchor)?,
                },
            )
        }
        ["teleport", targets, x, y, z, "facing", fx, fy, fz] => teleport_to_pos_with_facing(
            state,
            parse_entity_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            RotationMode::FacingPosition(parse_vec3(fx, fy, fz)?),
        ),
        ["teleport", targets, destination] => {
            teleport_to_entity(state, parse_entity_list(targets), entity_ref(destination))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn teleport_to_entity(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    destination: EntityRef,
) -> Result<CommandResult, CommandError> {
    let destination_position =
        entity_position(state, &destination)
            .cloned()
            .unwrap_or(EntityPosition {
                entity: destination.clone(),
                dimension: state.command_source_dimension.clone(),
                position: state.command_source_position,
            });
    for target in &targets {
        upsert_entity_position(state, target.clone(), destination_position.position);
        set_entity_dimension(state, target, &destination_position.dimension);
        record_teleport_side_effect(state, target.clone());
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: if targets.len() == 1 {
            "commands.teleport.success.entity.single"
        } else {
            "commands.teleport.success.entity.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn teleport_to_pos(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    position: Vec3,
    rotation: Option<(f32, f32, bool, bool)>,
) -> Result<CommandResult, CommandError> {
    validate_teleport_position(position)?;
    for target in &targets {
        upsert_entity_position(state, target.clone(), position);
        set_entity_dimension(state, target, &state.command_source_dimension.clone());
        record_teleport_side_effect(state, target.clone());
        if let Some((yaw, pitch, yaw_relative, pitch_relative)) = rotation {
            state.rotation_requests.push(RotationRequest {
                target: target.clone(),
                mode: RotationMode::Angles {
                    yaw,
                    pitch,
                    yaw_relative,
                    pitch_relative,
                },
            });
        }
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: if targets.len() == 1 {
            "commands.teleport.success.location.single"
        } else {
            "commands.teleport.success.location.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn teleport_to_pos_with_facing(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    position: Vec3,
    facing: RotationMode,
) -> Result<CommandResult, CommandError> {
    let result = teleport_to_pos(state, targets.clone(), position, None)?;
    for target in targets {
        state.rotation_requests.push(RotationRequest {
            target,
            mode: facing.clone(),
        });
    }
    Ok(result)
}

pub(super) fn record_teleport_side_effect(state: &mut ServerCommandState, target: EntityRef) {
    let kind = entity_kind(state, &target);
    let clear_vertical_motion_and_set_on_ground = kind != EntityKind::NonLiving;
    state.teleport_side_effects.push(TeleportSideEffect {
        target,
        clear_vertical_motion_and_set_on_ground,
        stop_pathfinding_navigation: kind == EntityKind::Generic,
    });
}

pub(super) fn parse_teleport_vec3(
    state: &ServerCommandState,
    x: &str,
    y: &str,
    z: &str,
) -> Result<Vec3, CommandError> {
    Ok(Vec3 {
        x: parse_coordinate(x, state.command_source_position.x)?,
        y: parse_coordinate(y, state.command_source_position.y)?,
        z: parse_coordinate(z, state.command_source_position.z)?,
    })
}

pub(super) fn parse_coordinate(input: &str, base: f64) -> Result<f64, CommandError> {
    if input == "~" {
        Ok(base)
    } else if let Some(offset) = input.strip_prefix('~') {
        Ok(base + parse_f64(offset)?)
    } else {
        parse_f64(input)
    }
}

pub(super) fn parse_teleport_rotation(
    yaw: &str,
    pitch: &str,
) -> Result<(f32, f32, bool, bool), CommandError> {
    let (yaw, yaw_relative) = parse_teleport_rotation_component(yaw)?;
    let (pitch, pitch_relative) = parse_teleport_rotation_component(pitch)?;
    Ok((yaw, pitch, yaw_relative, pitch_relative))
}

pub(super) fn parse_teleport_rotation_component(input: &str) -> Result<(f32, bool), CommandError> {
    if input == "~" {
        Ok((0.0, true))
    } else if let Some(offset) = input.strip_prefix('~') {
        Ok((parse_f32(offset)?, true))
    } else {
        Ok((parse_f32(input)?, false))
    }
}

pub(super) fn validate_teleport_position(position: Vec3) -> Result<(), CommandError> {
    let block_x = position.x.floor();
    let block_y = position.y.floor();
    let block_z = position.z.floor();
    if !(-30_000_000.0..30_000_000.0).contains(&block_x)
        || !(-30_000_000.0..30_000_000.0).contains(&block_z)
        || !(-20_000_000.0..20_000_000.0).contains(&block_y)
    {
        Err(CommandError::TeleportInvalidPosition)
    } else {
        Ok(())
    }
}

pub(super) fn set_entity_dimension(
    state: &mut ServerCommandState,
    entity: &EntityRef,
    dimension: &str,
) {
    if let Some(entry) = state
        .entity_positions
        .iter_mut()
        .find(|entry| entry.entity.id == entity.id)
    {
        entry.dimension = dimension.to_string();
    }
}

pub(super) fn time_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["time", "query", "gametime"] => Ok(CommandResult {
            success_count: wrap_time_result(state.game_time_ticks as i64),
            feedback_key: "commands.time.query.gametime",
            broadcast_to_admins: false,
        }),
        ["time", "query", "of", clock, rest @ ..] => time_clock_command(state, clock, rest),
        ["time", rest @ ..] => {
            let clock = default_clock_for_dimension(&state.command_source_dimension)?;
            time_clock_command(state, clock, rest)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn time_clock_command(
    state: &mut ServerCommandState,
    clock: &str,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let clock = normalize_resource_id(clock);
    if !matches!(clock.as_str(), "minecraft:overworld" | "minecraft:the_end") {
        return Err(CommandError::TimeNoDefaultClock);
    }
    match parts {
        ["set", value] => set_clock_time(state, &clock, value),
        ["add", value] => {
            let ticks = parse_time_ticks_i32(value, i32::MIN)?;
            state.world_clock_ticks = state.world_clock_ticks.saturating_add(i64::from(ticks));
            Ok(CommandResult {
                success_count: wrap_time_result(state.world_clock_ticks),
                feedback_key: "commands.time.set.absolute",
                broadcast_to_admins: true,
            })
        }
        ["pause"] => {
            state.world_clock_paused = true;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.time.pause",
                broadcast_to_admins: true,
            })
        }
        ["resume"] => {
            state.world_clock_paused = false;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.time.resume",
                broadcast_to_admins: true,
            })
        }
        ["rate", rate] => {
            let rate = parse_clock_rate(rate)?;
            state.world_clock_rate = rate;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.time.rate",
                broadcast_to_admins: true,
            })
        }
        ["query", "time"] => Ok(CommandResult {
            success_count: wrap_time_result(state.world_clock_ticks),
            feedback_key: "commands.time.query.absolute",
            broadcast_to_admins: false,
        }),
        ["query", timeline] => query_timeline_time(state, &clock, timeline, false),
        ["query", timeline, "repetition"] => query_timeline_time(state, &clock, timeline, true),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn set_clock_time(
    state: &mut ServerCommandState,
    clock: &str,
    value: &str,
) -> Result<CommandResult, CommandError> {
    let (ticks, feedback_key) = match parse_time_ticks_i32(value, 0) {
        Ok(ticks) => (ticks, "commands.time.set.absolute"),
        Err(error)
            if value
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'-' | b'.')) =>
        {
            return Err(error);
        }
        Err(CommandError::InvalidSyntax) => (
            time_marker_ticks(clock, value).ok_or(CommandError::TimeNoTimeMarkerFound)?,
            "commands.time.set.time_marker",
        ),
        Err(error) => return Err(error),
    };
    state.world_clock_ticks = i64::from(ticks);
    Ok(CommandResult {
        success_count: ticks,
        feedback_key,
        broadcast_to_admins: true,
    })
}

pub(super) fn query_timeline_time(
    state: &ServerCommandState,
    clock: &str,
    timeline: &str,
    repetitions: bool,
) -> Result<CommandResult, CommandError> {
    let timeline = normalize_resource_id(timeline);
    let period = match timeline.as_str() {
        "minecraft:day" | "minecraft:villager_schedule" => Some(24_000),
        "minecraft:moon" => Some(192_000),
        "minecraft:early_game" => None,
        _ => return Err(CommandError::TimeWrongTimeline),
    };
    if clock != "minecraft:overworld" {
        return Err(CommandError::TimeWrongTimeline);
    }
    let ticks = if repetitions {
        period
            .map(|period| state.world_clock_ticks.div_euclid(i64::from(period)))
            .unwrap_or(0)
    } else {
        period
            .map(|period| state.world_clock_ticks.rem_euclid(i64::from(period)))
            .unwrap_or(state.world_clock_ticks)
    };
    Ok(CommandResult {
        success_count: wrap_time_result(ticks),
        feedback_key: if repetitions {
            "commands.time.query.timeline.repetitions"
        } else {
            "commands.time.query.timeline"
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn time_marker_ticks(clock: &str, marker: &str) -> Option<i32> {
    if clock != "minecraft:overworld" {
        return None;
    }
    match normalize_resource_id(marker).as_str() {
        "minecraft:day" => Some(1_000),
        "minecraft:noon" => Some(6_000),
        "minecraft:night" => Some(13_000),
        "minecraft:midnight" => Some(18_000),
        _ => None,
    }
}

pub(super) fn default_clock_for_dimension(dimension: &str) -> Result<&'static str, CommandError> {
    match dimension {
        "minecraft:overworld" => Ok("minecraft:overworld"),
        "minecraft:the_end" => Ok("minecraft:the_end"),
        _ => Err(CommandError::TimeNoDefaultClock),
    }
}

pub(super) fn normalize_resource_id(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        format!("minecraft:{input}")
    }
}

pub(super) fn parse_clock_rate(input: &str) -> Result<f32, CommandError> {
    let rate = input
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if (0.00001..=1000.0).contains(&rate) {
        Ok(rate)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_time_ticks_i32(input: &str, minimum: i32) -> Result<i32, CommandError> {
    let (number, multiplier) = match input.as_bytes().last().copied() {
        Some(b't') => (&input[..input.len() - 1], 1.0_f32),
        Some(b's') => (&input[..input.len() - 1], 20.0_f32),
        Some(b'd') => (&input[..input.len() - 1], 24_000.0_f32),
        Some(last) if last.is_ascii_alphabetic() => return Err(CommandError::InvalidSyntax),
        _ => (input, 1.0_f32),
    };
    let ticks = number
        .parse::<f32>()
        .map(|value| (value * multiplier).round() as i32)
        .map_err(|_| CommandError::InvalidSyntax)?;
    if ticks < minimum {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(ticks)
    }
}

pub(super) fn wrap_time_result(ticks: i64) -> i32 {
    (ticks % i64::from(i32::MAX)) as i32
}

pub(super) fn title_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["title", targets, "clear"] => title_event(
            state,
            title_targets(targets),
            TitleCommandAction::Clear { reset: false },
            "commands.title.cleared",
        ),
        ["title", targets, "reset"] => title_event(
            state,
            title_targets(targets),
            TitleCommandAction::Clear { reset: true },
            "commands.title.reset",
        ),
        ["title", targets, "times", fade_in, stay, fade_out] => title_event(
            state,
            title_targets(targets),
            TitleCommandAction::Times {
                fade_in: parse_time_ticks_i32(fade_in, 0)?,
                stay: parse_time_ticks_i32(stay, 0)?,
                fade_out: parse_time_ticks_i32(fade_out, 0)?,
            },
            "commands.title.times",
        ),
        ["title", targets, kind, component @ ..]
            if matches!(*kind, "title" | "subtitle" | "actionbar") && !component.is_empty() =>
        {
            let kind = match *kind {
                "title" => TitleTextKind::Title,
                "subtitle" => TitleTextKind::Subtitle,
                "actionbar" => TitleTextKind::ActionBar,
                _ => unreachable!(),
            };
            title_event(
                state,
                title_targets(targets),
                TitleCommandAction::Text {
                    kind,
                    component: component.join(" "),
                },
                match kind {
                    TitleTextKind::Title => "commands.title.show.title",
                    TitleTextKind::Subtitle => "commands.title.show.subtitle",
                    TitleTextKind::ActionBar => "commands.title.show.actionbar",
                },
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn title_event(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    action: TitleCommandAction,
    feedback_prefix: &'static str,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let count = targets.len() as i32;
    state
        .title_events
        .push(TitleCommandEvent { targets, action });
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            title_feedback_single(feedback_prefix)
        } else {
            title_feedback_multiple(feedback_prefix)
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn title_targets(input: &str) -> Vec<NameAndId> {
    input
        .split(',')
        .filter(|name| !name.is_empty())
        .map(NameAndId::create_offline)
        .collect()
}

pub(super) fn title_feedback_single(prefix: &str) -> &'static str {
    match prefix {
        "commands.title.cleared" => "commands.title.cleared.single",
        "commands.title.reset" => "commands.title.reset.single",
        "commands.title.times" => "commands.title.times.single",
        "commands.title.show.title" => "commands.title.show.title.single",
        "commands.title.show.subtitle" => "commands.title.show.subtitle.single",
        "commands.title.show.actionbar" => "commands.title.show.actionbar.single",
        _ => "commands.title.show.title.single",
    }
}

pub(super) fn title_feedback_multiple(prefix: &str) -> &'static str {
    match prefix {
        "commands.title.cleared" => "commands.title.cleared.multiple",
        "commands.title.reset" => "commands.title.reset.multiple",
        "commands.title.times" => "commands.title.times.multiple",
        "commands.title.show.title" => "commands.title.show.title.multiple",
        "commands.title.show.subtitle" => "commands.title.show.subtitle.multiple",
        "commands.title.show.actionbar" => "commands.title.show.actionbar.multiple",
        _ => "commands.title.show.title.multiple",
    }
}

pub(super) fn warden_spawn_tracker_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["warden_spawn_tracker", "clear"] => {
            let player = state
                .command_source_player
                .clone()
                .ok_or(CommandError::NoPlayers)?;
            set_warden_warning_level(state, player, 0);
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.warden_spawn_tracker.clear.success.single",
                broadcast_to_admins: true,
            })
        }
        ["warden_spawn_tracker", "set", warning_level] => {
            let warning_level = parse_i32(warning_level)?;
            if !(0..=4).contains(&warning_level) {
                return Err(CommandError::InvalidSyntax);
            }
            let player = state
                .command_source_player
                .clone()
                .ok_or(CommandError::NoPlayers)?;
            set_warden_warning_level(state, player, warning_level);
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.warden_spawn_tracker.set.success.single",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn set_warden_warning_level(
    state: &mut ServerCommandState,
    player: NameAndId,
    warning_level: i32,
) {
    if let Some(existing) = state
        .warden_spawn_trackers
        .iter_mut()
        .find(|entry| entry.player.uuid == player.uuid)
    {
        existing.warning_level = warning_level;
    } else {
        state.warden_spawn_trackers.push(WardenSpawnTrackerState {
            player,
            warning_level,
        });
    }
}

pub(super) fn waypoint_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["waypoint", "list"] => {
            let count = state
                .waypoints
                .iter()
                .filter(|waypoint| waypoint.dimension == state.command_source_dimension)
                .count() as i32;
            Ok(CommandResult {
                success_count: count,
                feedback_key: if count == 0 {
                    "commands.waypoint.list.empty"
                } else {
                    "commands.waypoint.list.success"
                },
                broadcast_to_admins: false,
            })
        }
        ["waypoint", "modify", entity, "color", "reset"] => {
            waypoint_mut(state, entity)?.color = None;
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.waypoint.modify.color.reset",
                broadcast_to_admins: false,
            })
        }
        ["waypoint", "modify", entity, "color", "hex", color] => {
            waypoint_mut(state, entity)?.color = Some(parse_hex_color(color)?);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.waypoint.modify.color",
                broadcast_to_admins: false,
            })
        }
        ["waypoint", "modify", entity, "color", color] => {
            waypoint_mut(state, entity)?.color = Some(parse_named_color(color)?);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.waypoint.modify.color",
                broadcast_to_admins: false,
            })
        }
        ["waypoint", "modify", entity, "style", "reset"] => {
            waypoint_mut(state, entity)?.style = "minecraft:default".to_string();
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.waypoint.modify.style",
                broadcast_to_admins: false,
            })
        }
        ["waypoint", "modify", entity, "style", "set", style] => {
            waypoint_mut(state, entity)?.style = normalize_resource_id(style);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.waypoint.modify.style",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn waypoint_mut<'a>(
    state: &'a mut ServerCommandState,
    entity: &str,
) -> Result<&'a mut WaypointState, CommandError> {
    state
        .waypoints
        .iter_mut()
        .find(|waypoint| waypoint.entity.id == entity)
        .ok_or(CommandError::WaypointInvalid)
}

pub(super) fn parse_hex_color(input: &str) -> Result<i32, CommandError> {
    let hex = input.strip_prefix('#').unwrap_or(input);
    if hex.len() != 6 || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(CommandError::InvalidSyntax);
    }
    i32::from_str_radix(hex, 16).map_err(|_| CommandError::InvalidSyntax)
}

pub(super) fn parse_named_color(input: &str) -> Result<i32, CommandError> {
    match input {
        "black" => Ok(0x000000),
        "dark_blue" => Ok(0x0000AA),
        "dark_green" => Ok(0x00AA00),
        "dark_aqua" => Ok(0x00AAAA),
        "dark_red" => Ok(0xAA0000),
        "dark_purple" => Ok(0xAA00AA),
        "gold" => Ok(0xFFAA00),
        "gray" => Ok(0xAAAAAA),
        "dark_gray" => Ok(0x555555),
        "blue" => Ok(0x5555FF),
        "green" => Ok(0x55FF55),
        "aqua" => Ok(0x55FFFF),
        "red" => Ok(0xFF5555),
        "light_purple" => Ok(0xFF55FF),
        "yellow" => Ok(0xFFFF55),
        "white" => Ok(0xFFFFFF),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn worldborder_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["worldborder", "get"] => Ok(CommandResult {
            success_count: (state.world_border.size() + 0.5).floor() as i32,
            feedback_key: "commands.worldborder.get",
            broadcast_to_admins: false,
        }),
        ["worldborder", "set", distance] => worldborder_set_size(state, parse_f64(distance)?, 0),
        ["worldborder", "set", distance, time] => worldborder_set_size(
            state,
            parse_f64(distance)?,
            i64::from(parse_time_ticks_allow_zero(time)?),
        ),
        ["worldborder", "add", distance] => {
            let distance = state.world_border.size() + parse_f64(distance)?;
            worldborder_set_size(state, distance, 0)
        }
        ["worldborder", "add", distance, time] => {
            let distance = state.world_border.size() + parse_f64(distance)?;
            let ticks = state
                .world_border
                .lerp_time()
                .saturating_add(i64::from(parse_time_ticks_allow_zero(time)?));
            worldborder_set_size(state, distance, ticks)
        }
        ["worldborder", "center", x, z] => {
            let x = parse_f64(x)?;
            let z = parse_f64(z)?;
            if state.world_border.center_x == x && state.world_border.center_z == z {
                return Err(CommandError::WorldBorderSameCenter);
            }
            if x.abs() > WORLD_BORDER_MAX_CENTER_COORDINATE
                || z.abs() > WORLD_BORDER_MAX_CENTER_COORDINATE
            {
                return Err(CommandError::WorldBorderTooFarOut);
            }
            state.world_border.set_center(x, z);
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.worldborder.center.success",
                broadcast_to_admins: true,
            })
        }
        ["worldborder", "damage", "buffer", distance] => {
            let distance = parse_non_negative_f32(distance)? as f64;
            if state.world_border.safe_zone == distance {
                return Err(CommandError::WorldBorderSameDamageBuffer);
            }
            state.world_border.safe_zone = distance;
            Ok(CommandResult {
                success_count: distance as i32,
                feedback_key: "commands.worldborder.damage.buffer.success",
                broadcast_to_admins: true,
            })
        }
        ["worldborder", "damage", "amount", amount] => {
            let amount = parse_non_negative_f32(amount)? as f64;
            if state.world_border.damage_per_block == amount {
                return Err(CommandError::WorldBorderSameDamageAmount);
            }
            state.world_border.damage_per_block = amount;
            Ok(CommandResult {
                success_count: amount as i32,
                feedback_key: "commands.worldborder.damage.amount.success",
                broadcast_to_admins: true,
            })
        }
        ["worldborder", "warning", "distance", distance] => {
            let distance = parse_non_negative_i32(distance)?;
            if state.world_border.warning_blocks == distance {
                return Err(CommandError::WorldBorderSameWarningDistance);
            }
            state.world_border.warning_blocks = distance;
            Ok(CommandResult {
                success_count: distance,
                feedback_key: "commands.worldborder.warning.distance.success",
                broadcast_to_admins: true,
            })
        }
        ["worldborder", "warning", "time", time] => {
            let ticks = parse_time_ticks_allow_zero(time)? as i32;
            if state.world_border.warning_time == ticks {
                return Err(CommandError::WorldBorderSameWarningTime);
            }
            state.world_border.warning_time = ticks;
            Ok(CommandResult {
                success_count: ticks,
                feedback_key: "commands.worldborder.warning.time.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn worldborder_set_size(
    state: &mut ServerCommandState,
    distance: f64,
    ticks: i64,
) -> Result<CommandResult, CommandError> {
    let current = state.world_border.size();
    if current == distance {
        return Err(CommandError::WorldBorderSameSize);
    }
    if distance < 1.0 {
        return Err(CommandError::WorldBorderTooSmall);
    }
    if distance > WORLD_BORDER_MAX_SIZE {
        return Err(CommandError::WorldBorderTooBig);
    }
    if ticks > 0 {
        state
            .world_border
            .lerp_size_between(current, distance, ticks);
    } else {
        state.world_border.set_size(distance);
    }
    Ok(CommandResult {
        success_count: (distance - current) as i32,
        feedback_key: if ticks > 0 {
            if distance > current {
                "commands.worldborder.set.grow"
            } else {
                "commands.worldborder.set.shrink"
            }
        } else {
            "commands.worldborder.set.immediate"
        },
        broadcast_to_admins: true,
    })
}
