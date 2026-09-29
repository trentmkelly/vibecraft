use super::*;

pub(super) fn capture_command_source(state: &ServerCommandState) -> CommandSourceSnapshot {
    CommandSourceSnapshot {
        entity: state.command_source_entity.clone(),
        player: state.command_source_player.clone(),
        position: state.command_source_position,
        yaw: state.command_source_yaw,
        pitch: state.command_source_pitch,
        dimension: state.command_source_dimension.clone(),
    }
}

pub(super) fn restore_command_source(
    state: &mut ServerCommandState,
    source: CommandSourceSnapshot,
) {
    state.command_source_entity = source.entity;
    state.command_source_player = source.player;
    state.command_source_position = source.position;
    state.command_source_yaw = source.yaw;
    state.command_source_pitch = source.pitch;
    state.command_source_dimension = source.dimension;
}

pub(super) fn apply_execute_source(
    state: &mut ServerCommandState,
    original: &CommandSourceSnapshot,
    source: &ExecuteSourceSnapshot,
) {
    state.command_source_entity = source.entity.clone();
    state.command_source_player = source
        .entity
        .as_ref()
        .map(|entity| NameAndId::create_offline(&entity.id))
        .or_else(|| original.player.clone());
    state.command_source_position = source.position;
    state.command_source_yaw = source.yaw;
    state.command_source_pitch = source.pitch;
    state.command_source_dimension = source.dimension.clone();
}

pub(super) fn entity_position<'a>(
    state: &'a ServerCommandState,
    entity: &EntityRef,
) -> Option<&'a EntityPosition> {
    state
        .entity_positions
        .iter()
        .find(|position| position.entity.id == entity.id)
}

pub(super) fn entity_state<'a>(
    state: &'a ServerCommandState,
    entity: &EntityRef,
) -> Option<&'a EntityState> {
    state
        .entity_states
        .iter()
        .find(|state| state.entity.id == entity.id)
}

pub(super) fn experience_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let parts = if parts.first() == Some(&"xp") {
        let mut redirected = parts.to_vec();
        redirected[0] = "experience";
        redirected
    } else {
        parts.to_vec()
    };
    match parts.as_slice() {
        ["experience", "add", targets, amount] => experience_add(
            state,
            parse_name_list(targets),
            parse_i32(amount)?,
            ExperienceType::Points,
        ),
        ["experience", "add", targets, amount, ty] => experience_add(
            state,
            parse_name_list(targets),
            parse_i32(amount)?,
            parse_experience_type(ty)?,
        ),
        ["experience", "set", targets, amount] => experience_set(
            state,
            parse_name_list(targets),
            parse_non_negative_i32(amount)?,
            ExperienceType::Points,
        ),
        ["experience", "set", targets, amount, ty] => experience_set(
            state,
            parse_name_list(targets),
            parse_non_negative_i32(amount)?,
            parse_experience_type(ty)?,
        ),
        ["experience", "query", target, ty] => {
            if target.contains(',') {
                return Err(CommandError::InvalidSyntax);
            }
            let player = NameAndId::create_offline(target);
            let ty = parse_experience_type(ty)?;
            let state = player_experience(state, &player).clone();
            Ok(CommandResult {
                success_count: ty.query(&state),
                feedback_key: match ty {
                    ExperienceType::Points => "commands.experience.query.points",
                    ExperienceType::Levels => "commands.experience.query.levels",
                },
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn experience_add(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    amount: i32,
    ty: ExperienceType,
) -> Result<CommandResult, CommandError> {
    for target in &targets {
        let xp = player_experience_mut(state, target);
        match ty {
            ExperienceType::Points => give_experience_points(xp, amount),
            ExperienceType::Levels => give_experience_levels(xp, amount),
        }
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: match (ty, targets.len()) {
            (ExperienceType::Points, 1) => "commands.experience.add.points.success.single",
            (ExperienceType::Points, _) => "commands.experience.add.points.success.multiple",
            (ExperienceType::Levels, 1) => "commands.experience.add.levels.success.single",
            (ExperienceType::Levels, _) => "commands.experience.add.levels.success.multiple",
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn experience_set(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    amount: i32,
    ty: ExperienceType,
) -> Result<CommandResult, CommandError> {
    let mut success = 0;
    for target in &targets {
        let xp = player_experience_mut(state, target);
        let changed = match ty {
            ExperienceType::Points => set_experience_points(xp, amount),
            ExperienceType::Levels => {
                set_experience_levels(xp, amount);
                true
            }
        };
        if changed {
            success += 1;
        }
    }
    if success == 0 {
        return Err(CommandError::ExperienceSetPointsInvalid);
    }
    Ok(CommandResult {
        success_count: targets.len() as i32,
        feedback_key: match (ty, targets.len()) {
            (ExperienceType::Points, 1) => "commands.experience.set.points.success.single",
            (ExperienceType::Points, _) => "commands.experience.set.points.success.multiple",
            (ExperienceType::Levels, 1) => "commands.experience.set.levels.success.single",
            (ExperienceType::Levels, _) => "commands.experience.set.levels.success.multiple",
        },
        broadcast_to_admins: true,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExperienceType {
    Points,
    Levels,
}

impl ExperienceType {
    pub(super) fn query(self, state: &PlayerExperienceState) -> i32 {
        match self {
            ExperienceType::Points => {
                (state.progress * xp_needed_for_next_level(state.level) as f32).floor() as i32
            }
            ExperienceType::Levels => state.level,
        }
    }
}

pub(super) fn parse_experience_type(input: &str) -> Result<ExperienceType, CommandError> {
    match input {
        "points" => Ok(ExperienceType::Points),
        "levels" => Ok(ExperienceType::Levels),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn player_experience<'a>(
    state: &'a ServerCommandState,
    player: &NameAndId,
) -> &'a PlayerExperienceState {
    static ZERO_XP: std::sync::OnceLock<PlayerExperienceState> = std::sync::OnceLock::new();
    state
        .player_experience
        .iter()
        .find(|xp| xp.player.uuid == player.uuid)
        .unwrap_or_else(|| {
            ZERO_XP.get_or_init(|| PlayerExperienceState {
                player: NameAndId::create_offline(""),
                level: 0,
                progress: 0.0,
                total: 0,
            })
        })
}

pub(super) fn player_experience_mut<'a>(
    state: &'a mut ServerCommandState,
    player: &NameAndId,
) -> &'a mut PlayerExperienceState {
    if let Some(index) = state
        .player_experience
        .iter()
        .position(|xp| xp.player.uuid == player.uuid)
    {
        &mut state.player_experience[index]
    } else {
        state.player_experience.push(PlayerExperienceState {
            player: player.clone(),
            level: 0,
            progress: 0.0,
            total: 0,
        });
        let index = state.player_experience.len() - 1;
        &mut state.player_experience[index]
    }
}

pub(super) fn give_experience_points(state: &mut PlayerExperienceState, amount: i32) {
    state.progress += amount as f32 / xp_needed_for_next_level(state.level) as f32;
    state.total = state.total.saturating_add(amount).max(0);
    while state.progress < 0.0 {
        let remaining = state.progress * xp_needed_for_next_level(state.level) as f32;
        if state.level > 0 {
            give_experience_levels(state, -1);
            state.progress = 1.0 + remaining / xp_needed_for_next_level(state.level) as f32;
        } else {
            give_experience_levels(state, -1);
            state.progress = 0.0;
        }
    }
    while state.progress >= 1.0 {
        state.progress = (state.progress - 1.0) * xp_needed_for_next_level(state.level) as f32;
        give_experience_levels(state, 1);
        state.progress /= xp_needed_for_next_level(state.level) as f32;
    }
}

pub(super) fn give_experience_levels(state: &mut PlayerExperienceState, amount: i32) {
    state.level = state.level.saturating_add(amount);
    if state.level < 0 {
        state.level = 0;
        state.progress = 0.0;
        state.total = 0;
    }
}

pub(super) fn set_experience_points(state: &mut PlayerExperienceState, amount: i32) -> bool {
    let needed = xp_needed_for_next_level(state.level);
    if amount >= needed {
        return false;
    }
    let max = (needed - 1) as f32 / needed as f32;
    state.progress = (amount as f32 / needed as f32).clamp(0.0, max);
    true
}

pub(super) fn set_experience_levels(state: &mut PlayerExperienceState, amount: i32) {
    state.level = amount;
}

pub(super) fn xp_needed_for_next_level(level: i32) -> i32 {
    if level >= 30 {
        112 + (level - 30) * 9
    } else if level >= 15 {
        37 + (level - 15) * 5
    } else {
        7 + level * 2
    }
}

pub(super) fn fetch_profile_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["fetchprofile", "name", name @ ..] if !name.is_empty() => {
            let name = name.join(" ");
            let profile = profile_by_name(state, &name);
            finish_async_profile_fetch(
                state,
                FetchProfileQuery::Name(name),
                profile,
                "commands.fetchprofile.name.success",
                "commands.fetchprofile.name.failure",
            );
            Ok(CommandResult {
                success_count: 1,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        ["fetchprofile", "id", id] => {
            let id = parse_uuid_string(id)?;
            let profile = profile_by_uuid(state, &id);
            finish_async_profile_fetch(
                state,
                FetchProfileQuery::Id(id),
                profile,
                "commands.fetchprofile.id.success",
                "commands.fetchprofile.id.failure",
            );
            Ok(CommandResult {
                success_count: 1,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        ["fetchprofile", "entity", entity] => {
            let entity = entity_ref(entity);
            let profile = state
                .avatar_profiles
                .iter()
                .find(|avatar| avatar.entity.id == entity.id)
                .map(|avatar| avatar.profile.clone())
                .ok_or(CommandError::FetchProfileNotFound)?;
            record_fetched_profile(
                state,
                FetchProfileQuery::Entity(entity),
                profile,
            );
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.fetchprofile.entity.success",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn finish_async_profile_fetch(
    state: &mut ServerCommandState,
    query: FetchProfileQuery,
    profile: Option<NameAndId>,
    success_key: &'static str,
    failure_key: &'static str,
) {
    if let Some(profile) = profile {
        record_fetched_profile(state, query, profile);
        state.side_feedback.push(CommandResult {
            success_count: 1,
            feedback_key: success_key,
            broadcast_to_admins: false,
        });
    } else {
        state.side_feedback.push(CommandResult {
            success_count: 0,
            feedback_key: failure_key,
            broadcast_to_admins: false,
        });
    }
}

pub(super) fn record_fetched_profile(
    state: &mut ServerCommandState,
    query: FetchProfileQuery,
    profile: NameAndId,
) {
    state.fetched_profiles.push(FetchProfileEvent {
        query,
        encoded_profile: encoded_profile(&profile),
        encoded_head_component: encoded_head_component(&profile),
        profile,
    });
}

pub(super) fn profile_by_name(state: &ServerCommandState, name: &str) -> Option<NameAndId> {
    known_profiles(state)
        .into_iter()
        .find(|profile| profile.name == name)
}

pub(super) fn profile_by_uuid(state: &ServerCommandState, uuid: &str) -> Option<NameAndId> {
    known_profiles(state)
        .into_iter()
        .find(|profile| profile.uuid == uuid)
}

pub(super) fn known_profiles(state: &ServerCommandState) -> Vec<NameAndId> {
    state
        .online_players
        .iter()
        .chain(state.operator_players.iter())
        .chain(state.whitelisted_players.iter())
        .chain(state.banned_players.iter().map(|entry| &entry.user))
        .chain(
            state
                .player_inventories
                .iter()
                .map(|inventory| &inventory.player),
        )
        .chain(state.player_experience.iter().map(|xp| &xp.player))
        .cloned()
        .collect()
}

pub(super) fn encoded_profile(profile: &NameAndId) -> String {
    format!(
        "{{name:\"{}\",id:\"{}\"}}",
        escape_command_string(&profile.name),
        profile.uuid
    )
}

pub(super) fn encoded_head_component(profile: &NameAndId) -> String {
    format!(
        "{{type:\"object\",contents:{{type:\"player\",profile:{}}}}}",
        encoded_profile(profile)
    )
}

pub(super) fn escape_command_string(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
