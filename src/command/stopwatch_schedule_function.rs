use super::*;

pub(super) fn ensure_attribute_target_is_living(
    state: &ServerCommandState,
    target: &str,
) -> Result<(), CommandError> {
    if state
        .entity_states
        .iter()
        .any(|entry| entry.entity.id == target && entry.kind == EntityKind::NonLiving)
    {
        Err(CommandError::AttributeNotLiving)
    } else {
        Ok(())
    }
}

pub(super) fn stop_sound_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if !(2..=4).contains(&parts.len()) {
        return Err(CommandError::InvalidSyntax);
    }
    let targets = parse_name_list(parts[1]);
    let (source, sound) = match parts {
        ["stopsound", _targets] => (None, None),
        ["stopsound", _targets, "*"] => return Err(CommandError::InvalidSyntax),
        ["stopsound", _targets, "*", sound] => (None, Some(parse_resource_identifier(sound)?)),
        ["stopsound", _targets, source] => (Some(parse_sound_source(source)?), None),
        ["stopsound", _targets, source, sound] => (
            Some(parse_sound_source(source)?),
            Some(parse_resource_identifier(sound)?),
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    let count = targets.len() as i32;
    let feedback_key = match (source, sound.as_deref()) {
        (Some(_), Some(_)) => "commands.stopsound.success.source.sound",
        (Some(_), None) => "commands.stopsound.success.source.any",
        (None, Some(_)) => "commands.stopsound.success.sourceless.sound",
        (None, None) => "commands.stopsound.success.sourceless.any",
    };
    state
        .sound_events
        .push(SoundCommandEvent::Stop(StopSoundRequest {
            targets,
            source,
            sound,
        }));
    Ok(CommandResult {
        success_count: count,
        feedback_key,
        broadcast_to_admins: true,
    })
}

pub(super) fn stopwatch_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["stopwatch", "create", id] => {
            let id = parse_identifier(id)?;
            if state.stopwatches.iter().any(|watch| watch.id == id) {
                return Err(CommandError::StopwatchAlreadyExists);
            }
            state.stopwatches.push(StopwatchState {
                id,
                creation_time_millis: state.command_time_millis,
                accumulated_elapsed_millis: 0,
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.stopwatch.create.success",
                broadcast_to_admins: true,
            })
        }
        ["stopwatch", "query", id] => query_stopwatch(state, id, 1.0),
        ["stopwatch", "query", id, scale] => {
            let scale = scale
                .parse::<f64>()
                .map_err(|_| CommandError::InvalidSyntax)?;
            query_stopwatch(state, id, scale)
        }
        ["stopwatch", "restart", id] => {
            let id = parse_identifier(id)?;
            let Some(watch) = state.stopwatches.iter_mut().find(|watch| watch.id == id) else {
                return Err(CommandError::StopwatchDoesNotExist);
            };
            watch.creation_time_millis = state.command_time_millis;
            watch.accumulated_elapsed_millis = 0;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.stopwatch.restart.success",
                broadcast_to_admins: true,
            })
        }
        ["stopwatch", "remove", id] => {
            let id = parse_identifier(id)?;
            let old_len = state.stopwatches.len();
            state.stopwatches.retain(|watch| watch.id != id);
            if state.stopwatches.len() == old_len {
                return Err(CommandError::StopwatchDoesNotExist);
            }
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.stopwatch.remove.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn query_stopwatch(
    state: &ServerCommandState,
    id: &str,
    scale: f64,
) -> Result<CommandResult, CommandError> {
    let id = parse_identifier(id)?;
    let Some(watch) = state.stopwatches.iter().find(|watch| watch.id == id) else {
        return Err(CommandError::StopwatchDoesNotExist);
    };
    let elapsed_millis = watch.accumulated_elapsed_millis
        + state
            .command_time_millis
            .saturating_sub(watch.creation_time_millis);
    let elapsed_seconds = elapsed_millis as f64 / 1000.0;
    Ok(CommandResult {
        success_count: (elapsed_seconds * scale) as i32,
        feedback_key: "commands.stopwatch.query",
        broadcast_to_admins: true,
    })
}

pub(super) fn setblock_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (position, block, mode, strict) = match parts {
        ["setblock", x, y, z, block] => (
            parse_setblock_pos(state, x, y, z)?,
            parse_fill_block_state(block)?,
            SetBlockMode::Replace,
            false,
        ),
        ["setblock", x, y, z, block, "replace"] => (
            parse_setblock_pos(state, x, y, z)?,
            parse_fill_block_state(block)?,
            SetBlockMode::Replace,
            false,
        ),
        ["setblock", x, y, z, block, "destroy"] => (
            parse_setblock_pos(state, x, y, z)?,
            parse_fill_block_state(block)?,
            SetBlockMode::Destroy,
            false,
        ),
        ["setblock", x, y, z, block, "keep"] => (
            parse_setblock_pos(state, x, y, z)?,
            parse_fill_block_state(block)?,
            SetBlockMode::Keep,
            false,
        ),
        ["setblock", x, y, z, block, "strict"] => (
            parse_setblock_pos(state, x, y, z)?,
            parse_fill_block_state(block)?,
            SetBlockMode::Replace,
            true,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };

    if state.debug_world {
        return Err(CommandError::SetBlockFailed);
    }

    let dimension = state.command_source_dimension.clone();
    let existing_block = block_at(state, &dimension, position);
    if mode == SetBlockMode::Keep
        && existing_block != "minecraft:air"
    {
        return Err(CommandError::SetBlockFailed);
    }

    let block_after_destroy = if mode == SetBlockMode::Destroy {
        "minecraft:air"
    } else {
        existing_block.as_str()
    };
    let place_needed = mode != SetBlockMode::Destroy || block != "minecraft:air";
    if place_needed && block == block_after_destroy {
        return Err(CommandError::SetBlockFailed);
    }

    let final_block = if place_needed {
        block.clone()
    } else {
        "minecraft:air".to_string()
    };
    set_block_in_dimension(state, &dimension, position, final_block);
    state.setblock_events.push(SetBlockEvent {
        dimension,
        position,
        block,
        mode,
        strict,
        destroyed_block: (mode == SetBlockMode::Destroy).then_some(existing_block),
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.setblock.success",
        broadcast_to_admins: true,
    })
}

fn parse_setblock_pos(
    state: &ServerCommandState,
    x: &str,
    y: &str,
    z: &str,
) -> Result<BlockPos, CommandError> {
    if x.starts_with('^') || y.starts_with('^') || z.starts_with('^') {
        if !(x.starts_with('^') && y.starts_with('^') && z.starts_with('^')) {
            return Err(CommandError::InvalidSyntax);
        }
        let left = parse_local_coordinate(x)?;
        let up = parse_local_coordinate(y)?;
        let forwards = parse_local_coordinate(z)?;
        return Ok(block_pos_containing(apply_local_block_coordinates(
            state, left, up, forwards,
        )));
    }

    Ok(BlockPos {
        x: parse_world_block_coordinate(x, state.command_source_position.x)?,
        y: parse_world_block_coordinate(y, state.command_source_position.y)?,
        z: parse_world_block_coordinate(z, state.command_source_position.z)?,
    })
}

fn parse_world_block_coordinate(input: &str, origin: f64) -> Result<i32, CommandError> {
    if let Some(relative) = input.strip_prefix('~') {
        let offset = if relative.is_empty() {
            0.0
        } else {
            relative
                .parse::<f64>()
                .map_err(|_| CommandError::InvalidSyntax)?
        };
        Ok((origin + offset).floor() as i32)
    } else {
        input
            .parse::<i32>()
            .map_err(|_| CommandError::InvalidSyntax)
    }
}

fn parse_local_coordinate(input: &str) -> Result<f64, CommandError> {
    let value = input
        .strip_prefix('^')
        .ok_or(CommandError::InvalidSyntax)?;
    if value.is_empty() {
        Ok(0.0)
    } else {
        value.parse::<f64>().map_err(|_| CommandError::InvalidSyntax)
    }
}

fn apply_local_block_coordinates(
    state: &ServerCommandState,
    left: f64,
    up: f64,
    forwards: f64,
) -> Vec3 {
    let yaw_radians = (state.command_source_yaw + 90.0).to_radians();
    let y_cos = yaw_radians.cos();
    let y_sin = yaw_radians.sin();
    let pitch_radians = (-state.command_source_pitch).to_radians();
    let x_cos = pitch_radians.cos();
    let x_sin = pitch_radians.sin();
    let pitch_up_radians = (-state.command_source_pitch + 90.0).to_radians();
    let x_cos_up = pitch_up_radians.cos();
    let x_sin_up = pitch_up_radians.sin();

    let forwards_vec = Vec3 {
        x: f64::from(y_cos * x_cos),
        y: f64::from(x_sin),
        z: f64::from(y_sin * x_cos),
    };
    let up_vec = Vec3 {
        x: f64::from(y_cos * x_cos_up),
        y: f64::from(x_sin_up),
        z: f64::from(y_sin * x_cos_up),
    };
    let left_vec = Vec3 {
        x: -(forwards_vec.y * up_vec.z - forwards_vec.z * up_vec.y),
        y: -(forwards_vec.z * up_vec.x - forwards_vec.x * up_vec.z),
        z: -(forwards_vec.x * up_vec.y - forwards_vec.y * up_vec.x),
    };

    Vec3 {
        x: state.command_source_position.x
            + forwards_vec.x * forwards
            + up_vec.x * up
            + left_vec.x * left,
        y: state.command_source_position.y
            + forwards_vec.y * forwards
            + up_vec.y * up
            + left_vec.y * left,
        z: state.command_source_position.z
            + forwards_vec.z * forwards
            + up_vec.z * up
            + left_vec.z * left,
    }
}

pub(super) fn schedule_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["schedule", "function", function, time] => {
            schedule_function(state, function, parse_time_ticks_allow_zero(time)?, true)
        }
        ["schedule", "function", function, time, "replace"] => {
            schedule_function(state, function, parse_time_ticks_allow_zero(time)?, true)
        }
        ["schedule", "function", function, time, "append"] => {
            schedule_function(state, function, parse_time_ticks_allow_zero(time)?, false)
        }
        ["schedule", "clear", id] => {
            let old_len = state.scheduled_functions.len();
            state.scheduled_functions.retain(|event| event.id != *id);
            let removed = old_len - state.scheduled_functions.len();
            if removed == 0 {
                return Err(CommandError::ScheduleCantRemove);
            }
            Ok(CommandResult {
                success_count: removed as i32,
                feedback_key: "commands.schedule.cleared.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn function_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 2 || parts[0] != "function" {
        return Err(CommandError::InvalidSyntax);
    }
    let name = parse_schedule_function(parts[1])?;
    let mut arguments = None;
    if parts.len() > 2 {
        if parts[2] == "with" {
            arguments = Some(resolve_function_macro_nbt_source(state, &parts[3..])?.to_snbt());
        } else {
            if parts.len() != 3 {
                return Err(CommandError::InvalidSyntax);
            }
            if !looks_like_compound_tag(parts[2]) {
                return Err(CommandError::FunctionArgumentNotCompound);
            }
            arguments = Some(parts[2].to_string());
        }
    }

    let functions = resolve_command_functions(state, &name.0, name.1)?;
    if functions.is_empty() {
        return Err(CommandError::FunctionNoFunctions);
    }
    let mut queued = 0;
    for function in functions {
        let instantiated = instantiate_command_function(&function, arguments.as_deref())
            .map_err(|_| CommandError::FunctionInstantiationFailure)?;
        state.queued_functions.push(QueuedFunctionCall {
            id: function.id.clone(),
            commands: instantiated.commands,
            arguments: arguments.clone(),
            source_dimension: state.command_source_dimension.clone(),
            suppressed_output: true,
            permission_level: PermissionLevel::Gamemasters,
        });
        queued += 1;
    }
    Ok(CommandResult {
        success_count: queued,
        feedback_key: if queued == 1 {
            "commands.function.scheduled.single"
        } else {
            "commands.function.scheduled.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn resolve_command_functions(
    state: &ServerCommandState,
    id: &str,
    tag: bool,
) -> Result<Vec<CommandFunctionDefinition>, CommandError> {
    if tag {
        let tag = state
            .function_tags
            .iter()
            .find(|entry| entry.id == id)
            .ok_or(CommandError::FunctionNoFunctions)?;
        Ok(tag
            .functions
            .iter()
            .filter_map(|function_id| {
                state
                    .available_functions
                    .iter()
                    .find(|function| function.id == *function_id)
                    .cloned()
            })
            .collect())
    } else {
        Ok(state
            .available_functions
            .iter()
            .find(|function| function.id == id)
            .cloned()
            .into_iter()
            .collect())
    }
}

pub(super) fn looks_like_compound_tag(input: &str) -> bool {
    input.starts_with('{') && input.ends_with('}')
}

pub(super) fn resolve_function_macro_nbt_source(
    state: &ServerCommandState,
    parts: &[&str],
) -> Result<Tag, CommandError> {
    match parts {
        ["entity", target] => state
            .macro_entity_nbt_sources
            .iter()
            .find(|source| source.entity.id == *target || source.entity.display_name == *target)
            .map(|source| source.nbt.clone())
            .ok_or(CommandError::FunctionInstantiationFailure),
        ["block", x, y, z] => {
            let pos = parse_block_pos(x, y, z)?;
            state
                .macro_block_nbt_sources
                .iter()
                .find(|source| source.pos == pos)
                .map(|source| source.nbt.clone())
                .ok_or(CommandError::FunctionInstantiationFailure)
        }
        ["storage", id] => {
            let id = parse_resource_identifier(id)?;
            state
                .macro_storage_nbt_sources
                .iter()
                .find(|source| source.id == id)
                .map(|source| source.nbt.clone())
                .ok_or(CommandError::FunctionInstantiationFailure)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}
