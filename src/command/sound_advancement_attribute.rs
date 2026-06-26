use super::*;

pub(super) fn play_sound_command(
    state: &mut ServerCommandState,
    parts: &[&str],
    _permissions: LevelBasedPermissionSet,
) -> Result<CommandResult, CommandError> {
    let sound = parts.get(1).ok_or(CommandError::InvalidSyntax)?;
    let sound = parse_resource_identifier(sound)?;
    let source_position = state.command_source_position;
    let source_player = || state.command_source_player.clone().into_iter().collect();
    let (source, targets, position, volume, pitch, min_volume) = match parts {
        ["playsound", _sound] => (
            SoundSource::Master,
            source_player(),
            source_position,
            1.0,
            1.0,
            0.0,
        ),
        ["playsound", _sound, source] => (
            parse_sound_source(source)?,
            source_player(),
            source_position,
            1.0,
            1.0,
            0.0,
        ),
        ["playsound", _sound, source, targets] => (
            parse_sound_source(source)?,
            parse_name_list(targets),
            source_position,
            1.0,
            1.0,
            0.0,
        ),
        ["playsound", _sound, source, targets, x, y, z] => (
            parse_sound_source(source)?,
            parse_name_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            1.0,
            1.0,
            0.0,
        ),
        ["playsound", _sound, source, targets, x, y, z, volume] => (
            parse_sound_source(source)?,
            parse_name_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            parse_non_negative_f32(volume)?,
            1.0,
            0.0,
        ),
        ["playsound", _sound, source, targets, x, y, z, volume, pitch] => (
            parse_sound_source(source)?,
            parse_name_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            parse_non_negative_f32(volume)?,
            parse_bounded_f32(pitch, 0.0, 2.0)?,
            0.0,
        ),
        ["playsound", _sound, source, targets, x, y, z, volume, pitch, min_volume] => (
            parse_sound_source(source)?,
            parse_name_list(targets),
            parse_teleport_vec3(state, x, y, z)?,
            parse_non_negative_f32(volume)?,
            parse_bounded_f32(pitch, 0.0, 2.0)?,
            parse_bounded_f32(min_volume, 0.0, 1.0)?,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() {
        return Err(CommandError::PlaySoundTooFar);
    }

    let count = targets.len() as i32;
    state
        .sound_events
        .push(SoundCommandEvent::Play(PlaySoundRequest {
            sound,
            source,
            targets,
            position,
            volume,
            pitch,
            min_volume,
        }));
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            "commands.playsound.success.single"
        } else {
            "commands.playsound.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn advancement_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let action = match parts.get(1).copied() {
        Some("grant") => AdvancementAction::Grant,
        Some("revoke") => AdvancementAction::Revoke,
        _ => return Err(CommandError::InvalidSyntax),
    };
    let targets = parts.get(2).ok_or(CommandError::InvalidSyntax)?;
    let targets = parse_name_list(targets);
    if targets.is_empty() {
        return Err(CommandError::NoPlayers);
    }
    match parts.get(3).copied() {
        Some("everything") if parts.len() == 4 => {
            let advancement_ids = state
                .advancements
                .iter()
                .map(|advancement| advancement.id.clone())
                .collect::<Vec<_>>();
            perform_advancement_action(state, action, &targets, &advancement_ids, None, false)
        }
        Some(mode @ ("only" | "from" | "until" | "through")) => {
            let advancement = parts.get(4).ok_or(CommandError::InvalidSyntax)?;
            let criterion = if mode == "only" && parts.len() > 5 {
                Some(parts[5..].join(" "))
            } else if parts.len() == 5 {
                None
            } else {
                return Err(CommandError::InvalidSyntax);
            };
            let advancement_ids = advancement_selection(state, advancement, mode)?;
            perform_advancement_action(
                state,
                action,
                &targets,
                &advancement_ids,
                criterion.as_deref(),
                true,
            )
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AdvancementAction {
    Grant,
    Revoke,
}

pub(super) fn advancement_selection(
    state: &ServerCommandState,
    target: &str,
    mode: &str,
) -> Result<Vec<String>, CommandError> {
    let target = parse_resource_identifier(target)?;
    if state.advancements.iter().all(|entry| entry.id != target) {
        return Ok(vec![target]);
    }
    let mut output = Vec::new();
    if matches!(mode, "until" | "through") {
        let mut parent = state
            .advancements
            .iter()
            .find(|entry| entry.id == target)
            .and_then(|entry| entry.parent.clone());
        let mut parents = Vec::new();
        while let Some(parent_id) = parent {
            parents.push(parent_id.clone());
            parent = state
                .advancements
                .iter()
                .find(|entry| entry.id == parent_id)
                .and_then(|entry| entry.parent.clone());
        }
        parents.reverse();
        output.extend(parents);
    }
    output.push(target.clone());
    if matches!(mode, "from" | "through") {
        add_advancement_children(state, &target, &mut output);
    }
    Ok(output)
}

pub(super) fn add_advancement_children(
    state: &ServerCommandState,
    parent: &str,
    output: &mut Vec<String>,
) {
    for child in state
        .advancements
        .iter()
        .filter(|entry| entry.parent.as_deref() == Some(parent))
    {
        output.push(child.id.clone());
        add_advancement_children(state, &child.id, output);
    }
}

pub(super) fn perform_advancement_action(
    state: &mut ServerCommandState,
    action: AdvancementAction,
    targets: &[NameAndId],
    advancements: &[String],
    criterion: Option<&str>,
    show_advancements: bool,
) -> Result<CommandResult, CommandError> {
    let mut count = 0;
    for target in targets {
        if !show_advancements {
            state.advancement_flush_events.push(AdvancementFlushEvent {
                player: target.clone(),
                hide_advancement_toasts: true,
            });
        }
        for advancement in advancements {
            if let Some(criterion) = criterion {
                let definition = state
                    .advancements
                    .iter()
                    .find(|entry| entry.id == *advancement)
                    .ok_or(CommandError::AdvancementCriterionNotFound)?;
                if !definition.criteria.iter().any(|entry| entry == criterion) {
                    return Err(CommandError::AdvancementCriterionNotFound);
                }
                if perform_advancement_criterion(state, action, target, advancement, criterion) {
                    count += 1;
                }
            } else if perform_advancement(state, action, target, advancement) {
                count += 1;
            }
        }
        if !show_advancements {
            state.advancement_flush_events.push(AdvancementFlushEvent {
                player: target.clone(),
                hide_advancement_toasts: false,
            });
        }
    }
    if count == 0 {
        return Err(CommandError::AdvancementNoAction);
    }
    Ok(CommandResult {
        success_count: count,
        feedback_key: match (
            action,
            criterion.is_some(),
            advancements.len(),
            targets.len(),
        ) {
            (AdvancementAction::Grant, true, _, 1) => {
                "commands.advancement.grant.criterion.to.one.success"
            }
            (AdvancementAction::Grant, true, _, _) => {
                "commands.advancement.grant.criterion.to.many.success"
            }
            (AdvancementAction::Revoke, true, _, 1) => {
                "commands.advancement.revoke.criterion.to.one.success"
            }
            (AdvancementAction::Revoke, true, _, _) => {
                "commands.advancement.revoke.criterion.to.many.success"
            }
            (AdvancementAction::Grant, false, 1, 1) => {
                "commands.advancement.grant.one.to.one.success"
            }
            (AdvancementAction::Grant, false, 1, _) => {
                "commands.advancement.grant.one.to.many.success"
            }
            (AdvancementAction::Grant, false, _, 1) => {
                "commands.advancement.grant.many.to.one.success"
            }
            (AdvancementAction::Grant, false, _, _) => {
                "commands.advancement.grant.many.to.many.success"
            }
            (AdvancementAction::Revoke, false, 1, 1) => {
                "commands.advancement.revoke.one.to.one.success"
            }
            (AdvancementAction::Revoke, false, 1, _) => {
                "commands.advancement.revoke.one.to.many.success"
            }
            (AdvancementAction::Revoke, false, _, 1) => {
                "commands.advancement.revoke.many.to.one.success"
            }
            (AdvancementAction::Revoke, false, _, _) => {
                "commands.advancement.revoke.many.to.many.success"
            }
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn perform_advancement(
    state: &mut ServerCommandState,
    action: AdvancementAction,
    target: &NameAndId,
    advancement: &str,
) -> bool {
    let criteria = state
        .advancements
        .iter()
        .find(|entry| entry.id == advancement)
        .map(|entry| entry.criteria.clone())
        .unwrap_or_else(|| vec!["impossible".to_string()]);
    let progress = player_advancement_progress_mut(state, target, advancement);
    match action {
        AdvancementAction::Grant => {
            let missing = criteria
                .into_iter()
                .filter(|criterion| !progress.completed_criteria.contains(criterion))
                .collect::<Vec<_>>();
            if missing.is_empty() {
                return false;
            }
            progress.completed_criteria.extend(missing);
            true
        }
        AdvancementAction::Revoke => {
            if progress.completed_criteria.is_empty() {
                return false;
            }
            progress.completed_criteria.clear();
            true
        }
    }
}

pub(super) fn perform_advancement_criterion(
    state: &mut ServerCommandState,
    action: AdvancementAction,
    target: &NameAndId,
    advancement: &str,
    criterion: &str,
) -> bool {
    let progress = player_advancement_progress_mut(state, target, advancement);
    match action {
        AdvancementAction::Grant => {
            if progress
                .completed_criteria
                .iter()
                .any(|entry| entry == criterion)
            {
                false
            } else {
                progress.completed_criteria.push(criterion.to_string());
                true
            }
        }
        AdvancementAction::Revoke => {
            let old_len = progress.completed_criteria.len();
            progress
                .completed_criteria
                .retain(|entry| entry != criterion);
            progress.completed_criteria.len() != old_len
        }
    }
}

pub(super) fn player_advancement_progress_mut<'a>(
    state: &'a mut ServerCommandState,
    player: &NameAndId,
    advancement: &str,
) -> &'a mut PlayerAdvancementProgress {
    if let Some(index) = state
        .player_advancements
        .iter()
        .position(|entry| entry.player.uuid == player.uuid && entry.advancement == advancement)
    {
        return &mut state.player_advancements[index];
    }
    state.player_advancements.push(PlayerAdvancementProgress {
        player: player.clone(),
        advancement: advancement.to_string(),
        completed_criteria: Vec::new(),
    });
    let index = state.player_advancements.len() - 1;
    &mut state.player_advancements[index]
}

pub(super) fn attribute_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    if parts.len() < 4 {
        return Err(CommandError::InvalidSyntax);
    }
    let target = parts[1];
    let attribute = parse_resource_identifier(parts[2])?;
    match parts[3] {
        "get" => {
            let scale = parts
                .get(4)
                .map(|value| parse_f64(value))
                .transpose()?
                .unwrap_or(1.0);
            if parts.len() > 5 {
                return Err(CommandError::InvalidSyntax);
            }
            let value = entity_attribute(state, target, &attribute)?.computed_value();
            Ok(CommandResult {
                success_count: (value * scale) as i32,
                feedback_key: "commands.attribute.value.get.success",
                broadcast_to_admins: false,
            })
        }
        "base" => attribute_base_command(state, target, &attribute, &parts[4..]),
        "modifier" => attribute_modifier_command(state, target, &attribute, &parts[4..]),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn attribute_base_command(
    state: &mut ServerCommandState,
    target: &str,
    attribute: &str,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["get"] | ["get", _] => {
            let scale = parts
                .get(1)
                .map(|value| parse_f64(value))
                .transpose()?
                .unwrap_or(1.0);
            let value = entity_attribute(state, target, attribute)?.base;
            Ok(CommandResult {
                success_count: (value * scale) as i32,
                feedback_key: "commands.attribute.base_value.get.success",
                broadcast_to_admins: false,
            })
        }
        ["set", value] => {
            let value = parse_f64(value)?;
            entity_attribute_mut(state, target, attribute)?.base = value;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.attribute.base_value.set.success",
                broadcast_to_admins: false,
            })
        }
        ["reset"] => {
            let attribute = entity_attribute_mut(state, target, attribute)?;
            attribute.base = attribute.default_base;
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.attribute.base_value.reset.success",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn attribute_modifier_command(
    state: &mut ServerCommandState,
    target: &str,
    attribute: &str,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["add", id, value, operation] => {
            let id = parse_resource_identifier(id)?;
            let value = parse_f64(value)?;
            let operation = match *operation {
                "add_value" => AttributeOperation::Value,
                "add_multiplied_base" => AttributeOperation::MultipliedBase,
                "add_multiplied_total" => AttributeOperation::MultipliedTotal,
                _ => return Err(CommandError::InvalidSyntax),
            };
            let attribute = entity_attribute_mut(state, target, attribute)?;
            if attribute.modifiers.iter().any(|modifier| modifier.id == id) {
                return Err(CommandError::AttributeModifierAlreadyPresent);
            }
            attribute.modifiers.push(AttributeModifierState {
                id,
                value,
                operation,
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.attribute.modifier.add.success",
                broadcast_to_admins: false,
            })
        }
        ["remove", id] => {
            let id = parse_resource_identifier(id)?;
            let attribute = entity_attribute_mut(state, target, attribute)?;
            let old_len = attribute.modifiers.len();
            attribute.modifiers.retain(|modifier| modifier.id != id);
            if attribute.modifiers.len() == old_len {
                return Err(CommandError::AttributeNoSuchModifier);
            }
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.attribute.modifier.remove.success",
                broadcast_to_admins: false,
            })
        }
        ["value", "get", id] | ["value", "get", id, _] => {
            let id = parse_resource_identifier(id)?;
            let scale = parts
                .get(3)
                .map(|value| parse_f64(value))
                .transpose()?
                .unwrap_or(1.0);
            let modifier = entity_attribute(state, target, attribute)?
                .modifiers
                .iter()
                .find(|modifier| modifier.id == id)
                .ok_or(CommandError::AttributeNoSuchModifier)?;
            Ok(CommandResult {
                success_count: (modifier.value * scale) as i32,
                feedback_key: "commands.attribute.modifier.value.get.success",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

impl EntityAttributeState {
    pub(super) fn computed_value(&self) -> f64 {
        let add_value = self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation == AttributeOperation::Value)
            .map(|modifier| modifier.value)
            .sum::<f64>();
        let base = self.base + add_value;
        let multiplied_base = self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation == AttributeOperation::MultipliedBase)
            .fold(base, |value, modifier| value + self.base * modifier.value);
        self.modifiers
            .iter()
            .filter(|modifier| modifier.operation == AttributeOperation::MultipliedTotal)
            .fold(multiplied_base, |value, modifier| {
                value * (1.0 + modifier.value)
            })
    }
}

pub(super) fn entity_attribute<'a>(
    state: &'a ServerCommandState,
    target: &str,
    attribute: &str,
) -> Result<&'a EntityAttributeState, CommandError> {
    ensure_attribute_target_is_living(state, target)?;
    state
        .entity_attributes
        .iter()
        .find(|entry| entry.target == target && entry.attribute == attribute)
        .ok_or(CommandError::AttributeNoSuchAttribute)
}

pub(super) fn entity_attribute_mut<'a>(
    state: &'a mut ServerCommandState,
    target: &str,
    attribute: &str,
) -> Result<&'a mut EntityAttributeState, CommandError> {
    ensure_attribute_target_is_living(state, target)?;
    state
        .entity_attributes
        .iter_mut()
        .find(|entry| entry.target == target && entry.attribute == attribute)
        .ok_or(CommandError::AttributeNoSuchAttribute)
}
