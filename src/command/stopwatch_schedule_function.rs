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
            parse_block_pos(x, y, z)?,
            parse_resource_identifier(block)?,
            SetBlockMode::Replace,
            false,
        ),
        ["setblock", x, y, z, block, "replace"] => (
            parse_block_pos(x, y, z)?,
            parse_resource_identifier(block)?,
            SetBlockMode::Replace,
            false,
        ),
        ["setblock", x, y, z, block, "destroy"] => (
            parse_block_pos(x, y, z)?,
            parse_resource_identifier(block)?,
            SetBlockMode::Destroy,
            false,
        ),
        ["setblock", x, y, z, block, "keep"] => (
            parse_block_pos(x, y, z)?,
            parse_resource_identifier(block)?,
            SetBlockMode::Keep,
            false,
        ),
        ["setblock", x, y, z, block, "strict"] => (
            parse_block_pos(x, y, z)?,
            parse_resource_identifier(block)?,
            SetBlockMode::Replace,
            true,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };

    if state.debug_world {
        return Err(CommandError::SetBlockFailed);
    }

    let existing_index = state.blocks.iter().position(|entry| {
        entry.dimension == state.command_source_dimension && entry.position == position
    });
    let existing_block = existing_index.map(|index| state.blocks[index].block.clone());
    if mode == SetBlockMode::Keep
        && existing_block.as_deref().unwrap_or("minecraft:air") != "minecraft:air"
    {
        return Err(CommandError::SetBlockFailed);
    }

    if let Some(index) = existing_index {
        state.blocks[index].block = block.clone();
    } else {
        state.blocks.push(BlockStateEntry {
            dimension: state.command_source_dimension.clone(),
            position,
            block: block.clone(),
        });
    }
    state.setblock_events.push(SetBlockEvent {
        dimension: state.command_source_dimension.clone(),
        position,
        block,
        mode,
        strict,
        destroyed_block: (mode == SetBlockMode::Destroy)
            .then_some(existing_block.unwrap_or_else(|| "minecraft:air".to_string())),
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.setblock.success",
        broadcast_to_admins: true,
    })
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
