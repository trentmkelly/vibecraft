use super::*;

pub(super) fn scoreboard_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "objectives", ..] => scoreboard_objectives_command(state, parts),
        ["scoreboard", "players", ..] => scoreboard_players_command(state, parts),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_objectives_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "objectives", "modify", ..] => {
            scoreboard_objective_modify_command(state, parts)
        }
        ["scoreboard", "objectives", "setdisplay", ..] => {
            scoreboard_objective_display_command(state, parts)
        }
        ["scoreboard", "objectives", "list"] => Ok(CommandResult {
            success_count: state.scoreboard_objectives.len() as i32,
            feedback_key: if state.scoreboard_objectives.is_empty() {
                "commands.scoreboard.objectives.list.empty"
            } else {
                "commands.scoreboard.objectives.list.success"
            },
            broadcast_to_admins: false,
        }),
        ["scoreboard", "objectives", "add", objective, criteria] => {
            add_scoreboard_objective(state, objective, criteria, objective)
        }
        ["scoreboard", "objectives", "add", objective, criteria, display] => {
            add_scoreboard_objective(state, objective, criteria, display)
        }
        ["scoreboard", "objectives", "remove", objective] => {
            require_scoreboard_objective(state, objective)?;
            state
                .scoreboard_objectives
                .retain(|entry| entry.name != *objective);
            state
                .scoreboard_scores
                .retain(|entry| entry.objective != *objective);
            state
                .scoreboard_display_slots
                .retain(|entry| entry.objective != *objective);
            Ok(CommandResult {
                success_count: state.scoreboard_objectives.len() as i32,
                feedback_key: "commands.scoreboard.objectives.remove.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_objective_modify_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "objectives", "modify", objective, "displayname", display] => {
            let objective = scoreboard_objective_mut(state, objective)?;
            objective.display_name = (*display).to_string();
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.modify.displayname",
                0,
                true,
            ))
        }
        ["scoreboard", "objectives", "modify", objective, "rendertype", render_type] => {
            if !matches!(*render_type, "integer" | "hearts") {
                return Err(CommandError::InvalidSyntax);
            }
            let objective = scoreboard_objective_mut(state, objective)?;
            objective.render_type = (*render_type).to_string();
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.modify.rendertype",
                0,
                true,
            ))
        }
        ["scoreboard", "objectives", "modify", objective, "displayautoupdate", value] => {
            let value = parse_bool(value)?;
            let objective = scoreboard_objective_mut(state, objective)?;
            objective.display_auto_update = value;
            Ok(scoreboard_result(
                if value {
                    "commands.scoreboard.objectives.modify.displayAutoUpdate.enable"
                } else {
                    "commands.scoreboard.objectives.modify.displayAutoUpdate.disable"
                },
                0,
                true,
            ))
        }
        ["scoreboard", "objectives", "modify", objective, "numberformat"] => {
            let objective = scoreboard_objective_mut(state, objective)?;
            objective.number_format = None;
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.modify.objectiveFormat.clear",
                0,
                true,
            ))
        }
        ["scoreboard", "objectives", "modify", objective, "numberformat", format @ ..]
            if !format.is_empty() =>
        {
            let value = parse_score_number_format(format)?;
            let objective = scoreboard_objective_mut(state, objective)?;
            objective.number_format = Some(value);
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.modify.objectiveFormat.set",
                0,
                true,
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_objective_display_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "objectives", "setdisplay", slot] => {
            if !state
                .scoreboard_display_slots
                .iter()
                .any(|entry| entry.slot == *slot)
            {
                return Err(CommandError::ScoreboardDisplayAlreadyEmpty);
            }
            state
                .scoreboard_display_slots
                .retain(|entry| entry.slot != *slot);
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.display.cleared",
                0,
                true,
            ))
        }
        ["scoreboard", "objectives", "setdisplay", slot, objective] => {
            require_scoreboard_objective(state, objective)?;
            if state
                .scoreboard_display_slots
                .iter()
                .any(|entry| entry.slot == *slot && entry.objective == *objective)
            {
                return Err(CommandError::ScoreboardDisplayAlreadySet);
            }
            state
                .scoreboard_display_slots
                .retain(|entry| entry.slot != *slot);
            state.scoreboard_display_slots.push(ScoreboardDisplaySlot {
                slot: (*slot).to_string(),
                objective: (*objective).to_string(),
            });
            Ok(scoreboard_result(
                "commands.scoreboard.objectives.display.set",
                0,
                true,
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "list", ..] | ["scoreboard", "players", "get", ..] => {
            scoreboard_players_query_command(state, parts)
        }
        ["scoreboard", "players", "set" | "add" | "remove", ..] => {
            scoreboard_players_score_command(state, parts)
        }
        ["scoreboard", "players", "reset" | "enable", ..] => {
            scoreboard_players_state_command(state, parts)
        }
        ["scoreboard", "players", "display", ..] => {
            scoreboard_players_display_command(state, parts)
        }
        ["scoreboard", "players", "operation", ..] => {
            scoreboard_players_operation_command(state, parts)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_query_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "list"] => {
            let count = tracked_score_holders(state).len();
            Ok(scoreboard_result(
                if count == 0 {
                    "commands.scoreboard.players.list.empty"
                } else {
                    "commands.scoreboard.players.list.success"
                },
                count as i32,
                false,
            ))
        }
        ["scoreboard", "players", "list", target] => {
            let count = state
                .scoreboard_scores
                .iter()
                .filter(|entry| entry.owner == *target)
                .count();
            Ok(scoreboard_result(
                if count == 0 {
                    "commands.scoreboard.players.list.entity.empty"
                } else {
                    "commands.scoreboard.players.list.entity.success"
                },
                count as i32,
                false,
            ))
        }
        ["scoreboard", "players", "get", target, objective] => {
            require_scoreboard_objective(state, objective)?;
            let score = scoreboard_score(state, target, objective)
                .ok_or(CommandError::ScoreboardScoreNotFound)?;
            Ok(scoreboard_result(
                "commands.scoreboard.players.get.success",
                score.value,
                false,
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_score_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "set", targets, objective, value] => {
            let value = parse_i32(value)?;
            set_scores(state, targets, objective, value)
        }
        ["scoreboard", "players", "add", targets, objective, value] => {
            let value = parse_non_negative_i32(value)?;
            add_scores(state, targets, objective, value)
        }
        ["scoreboard", "players", "remove", targets, objective, value] => {
            let value = parse_non_negative_i32(value)?;
            add_scores(state, targets, objective, -value)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_state_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "reset", targets] => {
            let names = parse_score_holders(targets);
            for name in &names {
                state.scoreboard_scores.retain(|entry| entry.owner != *name);
            }
            Ok(scoreboard_result(
                if names.len() == 1 {
                    "commands.scoreboard.players.reset.all.single"
                } else {
                    "commands.scoreboard.players.reset.all.multiple"
                },
                names.len() as i32,
                true,
            ))
        }
        ["scoreboard", "players", "reset", targets, objective] => {
            require_scoreboard_objective(state, objective)?;
            let names = parse_score_holders(targets);
            for name in &names {
                state
                    .scoreboard_scores
                    .retain(|entry| entry.owner != *name || entry.objective != *objective);
            }
            Ok(scoreboard_result(
                if names.len() == 1 {
                    "commands.scoreboard.players.reset.specific.single"
                } else {
                    "commands.scoreboard.players.reset.specific.multiple"
                },
                names.len() as i32,
                true,
            ))
        }
        ["scoreboard", "players", "enable", targets, objective] => {
            if require_scoreboard_objective(state, objective)?.criteria != "trigger" {
                return Err(CommandError::ScoreboardNotTrigger);
            }
            let names = parse_score_holders(targets);
            let mut changed = 0;
            for name in &names {
                let score = scoreboard_score_mut_or_create(state, name, objective);
                if !score.locked {
                    continue;
                }
                score.locked = false;
                changed += 1;
            }
            if changed == 0 {
                return Err(CommandError::ScoreboardTriggerAlreadyEnabled);
            }
            Ok(scoreboard_result(
                if names.len() == 1 {
                    "commands.scoreboard.players.enable.success.single"
                } else {
                    "commands.scoreboard.players.enable.success.multiple"
                },
                changed,
                true,
            ))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_display_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "display", "name", targets, objective] => {
            set_score_display_name(state, targets, objective, None)
        }
        ["scoreboard", "players", "display", "name", targets, objective, name] => {
            set_score_display_name(state, targets, objective, Some((*name).to_string()))
        }
        ["scoreboard", "players", "display", "numberformat", targets, objective] => {
            set_score_number_format(state, targets, objective, None)
        }
        ["scoreboard", "players", "display", "numberformat", targets, objective, format @ ..]
            if !format.is_empty() =>
        {
            set_score_number_format(
                state,
                targets,
                objective,
                Some(parse_score_number_format(format)?),
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn scoreboard_players_operation_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["scoreboard", "players", "operation", targets, target_objective, operation, sources, source_objective] => {
            scoreboard_operation(
                state,
                targets,
                target_objective,
                operation,
                sources,
                source_objective,
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn schedule_function(
    state: &mut ServerCommandState,
    function: &str,
    delay_ticks: u32,
    replace: bool,
) -> Result<CommandResult, CommandError> {
    if delay_ticks == 0 {
        return Err(CommandError::ScheduleSameTick);
    }
    let (function, tag) = parse_schedule_function(function)?;
    if !tag && state.macro_functions.iter().any(|entry| entry == &function) {
        return Err(CommandError::ScheduleMacro);
    }
    let schedule_id = if tag {
        format!("#{function}")
    } else {
        function.clone()
    };
    if replace {
        state
            .scheduled_functions
            .retain(|event| event.id != schedule_id);
    }
    let trigger_tick = state.game_time_ticks + delay_ticks as u64;
    state.scheduled_functions.push(ScheduledFunction {
        id: schedule_id,
        function,
        tag,
        trigger_tick,
    });
    Ok(CommandResult {
        success_count: trigger_tick.rem_euclid(i32::MAX as u64) as i32,
        feedback_key: if tag {
            "commands.schedule.created.tag"
        } else {
            "commands.schedule.created.function"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_schedule_function(input: &str) -> Result<(String, bool), CommandError> {
    if let Some(tag) = input.strip_prefix('#') {
        Ok((parse_resource_identifier(tag)?, true))
    } else {
        Ok((parse_resource_identifier(input)?, false))
    }
}

pub(super) fn add_scoreboard_objective(
    state: &mut ServerCommandState,
    objective: &str,
    criteria: &str,
    display_name: &str,
) -> Result<CommandResult, CommandError> {
    parse_identifier(objective)?;
    if state
        .scoreboard_objectives
        .iter()
        .any(|entry| entry.name == objective)
    {
        return Err(CommandError::ScoreboardObjectiveAlreadyExists);
    }
    state.scoreboard_objectives.push(ScoreboardObjective {
        name: objective.to_string(),
        criteria: criteria.to_string(),
        display_name: display_name.to_string(),
        render_type: "integer".to_string(),
        display_auto_update: true,
        number_format: None,
    });
    Ok(scoreboard_result(
        "commands.scoreboard.objectives.add.success",
        state.scoreboard_objectives.len() as i32,
        true,
    ))
}

pub(super) fn scoreboard_result(
    feedback_key: &'static str,
    success_count: i32,
    broadcast_to_admins: bool,
) -> CommandResult {
    CommandResult {
        success_count,
        feedback_key,
        broadcast_to_admins,
    }
}

pub(super) fn require_scoreboard_objective<'a>(
    state: &'a ServerCommandState,
    objective: &str,
) -> Result<&'a ScoreboardObjective, CommandError> {
    state
        .scoreboard_objectives
        .iter()
        .find(|entry| entry.name == objective)
        .ok_or(CommandError::ScoreboardObjectiveNotFound)
}

pub(super) fn scoreboard_objective_mut<'a>(
    state: &'a mut ServerCommandState,
    objective: &str,
) -> Result<&'a mut ScoreboardObjective, CommandError> {
    state
        .scoreboard_objectives
        .iter_mut()
        .find(|entry| entry.name == objective)
        .ok_or(CommandError::ScoreboardObjectiveNotFound)
}

pub(super) fn scoreboard_score<'a>(
    state: &'a ServerCommandState,
    owner: &str,
    objective: &str,
) -> Option<&'a ScoreboardScore> {
    state
        .scoreboard_scores
        .iter()
        .find(|entry| entry.owner == owner && entry.objective == objective)
}

pub(super) fn scoreboard_score_mut_or_create<'a>(
    state: &'a mut ServerCommandState,
    owner: &str,
    objective: &str,
) -> &'a mut ScoreboardScore {
    let index = if let Some(index) = state
        .scoreboard_scores
        .iter()
        .position(|entry| entry.owner == owner && entry.objective == objective)
    {
        index
    } else {
        state.scoreboard_scores.push(ScoreboardScore {
            owner: owner.to_string(),
            objective: objective.to_string(),
            value: 0,
            locked: true,
            display_name: None,
            number_format: None,
        });
        state.scoreboard_scores.len() - 1
    };
    &mut state.scoreboard_scores[index]
}

pub(super) fn trigger_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["trigger", objective] => trigger_score(state, objective, TriggerMode::Simple),
        ["trigger", objective, "add", value] => {
            trigger_score(state, objective, TriggerMode::Add(parse_i32(value)?))
        }
        ["trigger", objective, "set", value] => {
            trigger_score(state, objective, TriggerMode::Set(parse_i32(value)?))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TriggerMode {
    Simple,
    Add(i32),
    Set(i32),
}

pub(super) fn trigger_score(
    state: &mut ServerCommandState,
    objective: &str,
    mode: TriggerMode,
) -> Result<CommandResult, CommandError> {
    let player = state
        .command_source_player
        .as_ref()
        .ok_or(CommandError::NoPlayers)?
        .name
        .clone();
    if require_scoreboard_objective(state, objective)?.criteria != "trigger" {
        return Err(CommandError::ScoreboardNotTrigger);
    }
    let score = state
        .scoreboard_scores
        .iter_mut()
        .find(|entry| entry.owner == player && entry.objective == objective)
        .filter(|entry| !entry.locked)
        .ok_or(CommandError::TriggerNotPrimed)?;
    let (success_count, feedback_key) = match mode {
        TriggerMode::Simple => {
            score.value = score.value.wrapping_add(1);
            (score.value, "commands.trigger.simple.success")
        }
        TriggerMode::Add(value) => {
            score.value = score.value.wrapping_add(value);
            (score.value, "commands.trigger.add.success")
        }
        TriggerMode::Set(value) => {
            score.value = value;
            (value, "commands.trigger.set.success")
        }
    };
    score.locked = true;
    Ok(CommandResult {
        success_count,
        feedback_key,
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_score_holders(input: &str) -> Vec<String> {
    input
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

pub(super) fn tracked_score_holders(state: &ServerCommandState) -> Vec<String> {
    let mut holders = Vec::new();
    for score in &state.scoreboard_scores {
        if !holders.contains(&score.owner) {
            holders.push(score.owner.clone());
        }
    }
    holders
}

pub(super) fn set_scores(
    state: &mut ServerCommandState,
    targets: &str,
    objective: &str,
    value: i32,
) -> Result<CommandResult, CommandError> {
    require_scoreboard_objective(state, objective)?;
    let names = parse_score_holders(targets);
    for name in &names {
        scoreboard_score_mut_or_create(state, name, objective).value = value;
    }
    Ok(scoreboard_result(
        if names.len() == 1 {
            "commands.scoreboard.players.set.success.single"
        } else {
            "commands.scoreboard.players.set.success.multiple"
        },
        names.len() as i32,
        true,
    ))
}

pub(super) fn add_scores(
    state: &mut ServerCommandState,
    targets: &str,
    objective: &str,
    delta: i32,
) -> Result<CommandResult, CommandError> {
    require_scoreboard_objective(state, objective)?;
    let names = parse_score_holders(targets);
    let mut last = 0;
    for name in &names {
        let score = scoreboard_score_mut_or_create(state, name, objective);
        score.value += delta;
        last = score.value;
    }
    Ok(scoreboard_result(
        if delta >= 0 {
            if names.len() == 1 {
                "commands.scoreboard.players.add.success.single"
            } else {
                "commands.scoreboard.players.add.success.multiple"
            }
        } else if names.len() == 1 {
            "commands.scoreboard.players.remove.success.single"
        } else {
            "commands.scoreboard.players.remove.success.multiple"
        },
        if names.len() == 1 {
            last
        } else {
            names.len() as i32
        },
        true,
    ))
}

pub(super) fn set_score_display_name(
    state: &mut ServerCommandState,
    targets: &str,
    objective: &str,
    display_name: Option<String>,
) -> Result<CommandResult, CommandError> {
    require_scoreboard_objective(state, objective)?;
    let names = parse_score_holders(targets);
    for name in &names {
        scoreboard_score_mut_or_create(state, name, objective).display_name = display_name.clone();
    }
    Ok(scoreboard_result(
        if display_name.is_some() {
            if names.len() == 1 {
                "commands.scoreboard.players.display.name.set.success.single"
            } else {
                "commands.scoreboard.players.display.name.set.success.multiple"
            }
        } else if names.len() == 1 {
            "commands.scoreboard.players.display.name.clear.success.single"
        } else {
            "commands.scoreboard.players.display.name.clear.success.multiple"
        },
        names.len() as i32,
        true,
    ))
}

pub(super) fn set_score_number_format(
    state: &mut ServerCommandState,
    targets: &str,
    objective: &str,
    number_format: Option<String>,
) -> Result<CommandResult, CommandError> {
    require_scoreboard_objective(state, objective)?;
    let names = parse_score_holders(targets);
    for name in &names {
        scoreboard_score_mut_or_create(state, name, objective).number_format =
            number_format.clone();
    }
    Ok(scoreboard_result(
        if number_format.is_some() {
            if names.len() == 1 {
                "commands.scoreboard.players.display.numberFormat.set.success.single"
            } else {
                "commands.scoreboard.players.display.numberFormat.set.success.multiple"
            }
        } else if names.len() == 1 {
            "commands.scoreboard.players.display.numberFormat.clear.success.single"
        } else {
            "commands.scoreboard.players.display.numberFormat.clear.success.multiple"
        },
        names.len() as i32,
        true,
    ))
}

pub(super) fn parse_score_number_format(parts: &[&str]) -> Result<String, CommandError> {
    match parts {
        ["blank"] => Ok("blank".to_string()),
        ["fixed", contents] => Ok(format!("fixed:{contents}")),
        ["styled", style] => Ok(format!("styled:{style}")),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn scoreboard_objective_to_nbt(objective: &ScoreboardObjective) -> Tag {
    let mut fields = vec![
        ("Name".to_string(), Tag::String(objective.name.clone())),
        (
            "CriteriaName".to_string(),
            Tag::String(objective.criteria.clone()),
        ),
        (
            "DisplayName".to_string(),
            Tag::String(objective.display_name.clone()),
        ),
        (
            "RenderType".to_string(),
            Tag::String(objective.render_type.clone()),
        ),
        (
            "display_auto_update".to_string(),
            Tag::Byte(i8::from(objective.display_auto_update)),
        ),
    ];
    if let Some(format) = &objective.number_format {
        fields.push(("NumberFormat".to_string(), Tag::String(format.clone())));
    }
    Tag::Compound(fields)
}

pub(super) fn scoreboard_objective_from_nbt(tag: &Tag) -> Result<ScoreboardObjective, String> {
    let fields = nbt_compound(tag)?;
    Ok(ScoreboardObjective {
        name: nbt_string(fields, "Name")?.to_string(),
        criteria: nbt_string(fields, "CriteriaName")?.to_string(),
        display_name: nbt_string(fields, "DisplayName")?.to_string(),
        render_type: nbt_string(fields, "RenderType")?.to_string(),
        display_auto_update: nbt_bool(fields, "display_auto_update").unwrap_or(true),
        number_format: nbt_optional_string(fields, "NumberFormat").map(str::to_string),
    })
}

pub(super) fn scoreboard_score_to_nbt(score: &ScoreboardScore) -> Tag {
    let mut fields = vec![
        ("Name".to_string(), Tag::String(score.owner.clone())),
        (
            "Objective".to_string(),
            Tag::String(score.objective.clone()),
        ),
        ("Score".to_string(), Tag::Int(score.value)),
        ("Locked".to_string(), Tag::Byte(i8::from(score.locked))),
    ];
    if let Some(display_name) = &score.display_name {
        fields.push(("DisplayName".to_string(), Tag::String(display_name.clone())));
    }
    if let Some(format) = &score.number_format {
        fields.push(("NumberFormat".to_string(), Tag::String(format.clone())));
    }
    Tag::Compound(fields)
}

pub(super) fn scoreboard_score_from_nbt(tag: &Tag) -> Result<ScoreboardScore, String> {
    let fields = nbt_compound(tag)?;
    Ok(ScoreboardScore {
        owner: nbt_string(fields, "Name")?.to_string(),
        objective: nbt_string(fields, "Objective")?.to_string(),
        value: nbt_int(fields, "Score")?,
        locked: nbt_bool(fields, "Locked").unwrap_or(true),
        display_name: nbt_optional_string(fields, "DisplayName").map(str::to_string),
        number_format: nbt_optional_string(fields, "NumberFormat").map(str::to_string),
    })
}

pub(super) fn nbt_compound(tag: &Tag) -> Result<&[(String, Tag)], String> {
    match tag {
        Tag::Compound(fields) => Ok(fields),
        _ => Err("expected compound".to_string()),
    }
}

pub(super) fn nbt_field<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    fields
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}

pub(super) fn nbt_string<'a>(fields: &'a [(String, Tag)], name: &str) -> Result<&'a str, String> {
    match nbt_field(fields, name) {
        Some(Tag::String(value)) => Ok(value),
        _ => Err(format!("missing string field {name}")),
    }
}

pub(super) fn nbt_optional_string<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    match nbt_field(fields, name) {
        Some(Tag::String(value)) => Some(value),
        _ => None,
    }
}

pub(super) fn nbt_int(fields: &[(String, Tag)], name: &str) -> Result<i32, String> {
    match nbt_field(fields, name) {
        Some(Tag::Int(value)) => Ok(*value),
        _ => Err(format!("missing int field {name}")),
    }
}

pub(super) fn nbt_bool(fields: &[(String, Tag)], name: &str) -> Option<bool> {
    match nbt_field(fields, name) {
        Some(Tag::Byte(value)) => Some(*value != 0),
        Some(Tag::Int(value)) => Some(*value != 0),
        _ => None,
    }
}

pub(super) fn scoreboard_operation(
    state: &mut ServerCommandState,
    targets: &str,
    target_objective: &str,
    operation: &str,
    sources: &str,
    source_objective: &str,
) -> Result<CommandResult, CommandError> {
    require_scoreboard_objective(state, target_objective)?;
    require_scoreboard_objective(state, source_objective)?;
    let targets = parse_score_holders(targets);
    let sources = parse_score_holders(sources);
    if sources.is_empty() || targets.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let source_values = sources
        .iter()
        .map(|source| {
            scoreboard_score(state, source, source_objective)
                .map(|score| score.value)
                .ok_or(CommandError::ScoreboardScoreNotFound)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut last = 0;
    for target in &targets {
        for source_value in &source_values {
            let score = scoreboard_score_mut_or_create(state, target, target_objective);
            apply_score_operation(score, operation, *source_value)?;
            last = score.value;
        }
    }
    Ok(scoreboard_result(
        if targets.len() == 1 {
            "commands.scoreboard.players.operation.success.single"
        } else {
            "commands.scoreboard.players.operation.success.multiple"
        },
        if targets.len() == 1 {
            last
        } else {
            targets.len() as i32
        },
        true,
    ))
}

pub(super) fn apply_score_operation(
    score: &mut ScoreboardScore,
    operation: &str,
    source_value: i32,
) -> Result<(), CommandError> {
    match operation {
        "=" => score.value = source_value,
        "+=" => score.value += source_value,
        "-=" => score.value -= source_value,
        "*=" => score.value *= source_value,
        "/=" => {
            if source_value == 0 {
                return Err(CommandError::InvalidSyntax);
            }
            score.value /= source_value;
        }
        "%=" => {
            if source_value == 0 {
                return Err(CommandError::InvalidSyntax);
            }
            score.value %= source_value;
        }
        "<" => score.value = score.value.min(source_value),
        ">" => score.value = score.value.max(source_value),
        _ => return Err(CommandError::InvalidSyntax),
    }
    Ok(())
}
