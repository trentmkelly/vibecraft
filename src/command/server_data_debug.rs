use super::*;

pub(super) fn damage_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (target, amount, source) = match parts {
        ["damage", target, amount] => (
            entity_ref(target),
            parse_damage_amount(amount)?,
            DamageCommandSource::Generic,
        ),
        ["damage", target, amount, damage_type] => (
            entity_ref(target),
            parse_damage_amount(amount)?,
            DamageCommandSource::Type {
                damage_type: parse_resource_identifier(damage_type)?,
            },
        ),
        ["damage", target, amount, damage_type, "at", x, y, z] => (
            entity_ref(target),
            parse_damage_amount(amount)?,
            DamageCommandSource::At {
                damage_type: parse_resource_identifier(damage_type)?,
                location: Vec3 {
                    x: parse_f64(x)?,
                    y: parse_f64(y)?,
                    z: parse_f64(z)?,
                },
            },
        ),
        ["damage", target, amount, damage_type, "by", entity] => (
            entity_ref(target),
            parse_damage_amount(amount)?,
            DamageCommandSource::By {
                damage_type: parse_resource_identifier(damage_type)?,
                entity: entity_ref(entity),
                cause: None,
            },
        ),
        ["damage", target, amount, damage_type, "by", entity, "from", cause] => (
            entity_ref(target),
            parse_damage_amount(amount)?,
            DamageCommandSource::By {
                damage_type: parse_resource_identifier(damage_type)?,
                entity: entity_ref(entity),
                cause: Some(entity_ref(cause)),
            },
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };

    if state
        .invulnerable_entities
        .iter()
        .any(|entity| entity.id == target.id)
    {
        return Err(CommandError::DamageInvulnerable);
    }

    state.damage_events.push(DamageCommandEvent {
        target,
        amount,
        source,
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.damage.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_damage_amount(input: &str) -> Result<f32, CommandError> {
    let amount = parse_f32(input)?;
    if amount < 0.0 {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(amount)
    }
}

pub(super) fn datapack_command(
    state: &mut ServerCommandState,
    parts: &[&str],
    permissions: LevelBasedPermissionSet,
) -> Result<CommandResult, CommandError> {
    match parts {
        ["datapack", "list"] => {
            let enabled = datapack_enabled_count(state);
            let available = datapack_available_count(state);
            Ok(CommandResult {
                success_count: enabled + available,
                feedback_key: "commands.datapack.list.success",
                broadcast_to_admins: false,
            })
        }
        ["datapack", "list", "enabled"] => {
            let count = datapack_enabled_count(state);
            Ok(CommandResult {
                success_count: count,
                feedback_key: if count == 0 {
                    "commands.datapack.list.enabled.none"
                } else {
                    "commands.datapack.list.enabled.success"
                },
                broadcast_to_admins: false,
            })
        }
        ["datapack", "list", "available"] => {
            let count = datapack_available_count(state);
            Ok(CommandResult {
                success_count: count,
                feedback_key: if count == 0 {
                    "commands.datapack.list.available.none"
                } else {
                    "commands.datapack.list.available.success"
                },
                broadcast_to_admins: false,
            })
        }
        ["datapack", "enable", id] => datapack_enable(state, id, DataPackInsert::Default),
        ["datapack", "enable", id, "first"] => datapack_enable(state, id, DataPackInsert::First),
        ["datapack", "enable", id, "last"] => datapack_enable(state, id, DataPackInsert::Last),
        ["datapack", "enable", id, "before", existing] => {
            datapack_enable(state, id, DataPackInsert::Before(existing))
        }
        ["datapack", "enable", id, "after", existing] => {
            datapack_enable(state, id, DataPackInsert::After(existing))
        }
        ["datapack", "disable", id] => datapack_disable(state, id),
        ["datapack", "create", id, description @ ..] => {
            if !permissions.has_permission(Permission::CommandLevel(PermissionLevel::Owners)) {
                return Err(CommandError::PermissionDenied);
            }
            if description.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            datapack_create(state, id, &description.join(" "))
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataPackInsert<'a> {
    Default,
    First,
    Last,
    Before(&'a str),
    After(&'a str),
}

pub(super) fn datapack_enabled_count(state: &ServerCommandState) -> i32 {
    state
        .selected_data_packs
        .iter()
        .filter(|id| state.available_data_packs.contains(id))
        .count() as i32
}

pub(super) fn datapack_available_count(state: &ServerCommandState) -> i32 {
    state
        .available_data_packs
        .iter()
        .filter(|id| {
            !state.selected_data_packs.contains(id)
                && !state.unavailable_feature_data_packs.contains(id)
        })
        .count() as i32
}

pub(super) fn datapack_enable(
    state: &mut ServerCommandState,
    id: &str,
    insert: DataPackInsert<'_>,
) -> Result<CommandResult, CommandError> {
    datapack_check_known(state, id)?;
    if state
        .selected_data_packs
        .iter()
        .any(|selected| selected == id)
    {
        return Err(CommandError::DataPackAlreadyEnabled);
    }
    if state
        .unavailable_feature_data_packs
        .iter()
        .any(|pack| pack == id)
    {
        return Err(CommandError::DataPackFeaturesNotEnabled);
    }

    let index = match insert {
        DataPackInsert::Default | DataPackInsert::Last => state.selected_data_packs.len(),
        DataPackInsert::First => 0,
        DataPackInsert::Before(existing) => selected_pack_index(state, existing)?,
        DataPackInsert::After(existing) => selected_pack_index(state, existing)? + 1,
    };
    state.selected_data_packs.insert(index, id.to_string());
    state.disabled_data_packs.retain(|disabled| disabled != id);
    state.reload_requests.push(ReloadRequest {
        selected_packs: state.selected_data_packs.clone(),
    });
    Ok(CommandResult {
        success_count: state.selected_data_packs.len() as i32,
        feedback_key: "commands.datapack.modify.enable",
        broadcast_to_admins: true,
    })
}

pub(super) fn datapack_disable(
    state: &mut ServerCommandState,
    id: &str,
) -> Result<CommandResult, CommandError> {
    datapack_check_known(state, id)?;
    if state
        .unavailable_feature_data_packs
        .iter()
        .any(|pack| pack == id)
    {
        return Err(CommandError::DataPackFeaturesNotEnabled);
    }
    if state
        .feature_data_packs
        .iter()
        .any(|feature_pack| feature_pack == id)
    {
        return Err(CommandError::DataPackCannotDisableFeature);
    }
    if !state
        .selected_data_packs
        .iter()
        .any(|selected| selected == id)
    {
        return Err(CommandError::DataPackAlreadyDisabled);
    }

    state.selected_data_packs.retain(|selected| selected != id);
    if !state
        .disabled_data_packs
        .iter()
        .any(|disabled| disabled == id)
    {
        state.disabled_data_packs.push(id.to_string());
    }
    state.reload_requests.push(ReloadRequest {
        selected_packs: state.selected_data_packs.clone(),
    });
    Ok(CommandResult {
        success_count: state.selected_data_packs.len() as i32,
        feedback_key: "commands.datapack.modify.disable",
        broadcast_to_admins: true,
    })
}

pub(super) fn datapack_create(
    state: &mut ServerCommandState,
    id: &str,
    description: &str,
) -> Result<CommandResult, CommandError> {
    if !is_valid_datapack_name(id) {
        return Err(CommandError::DataPackInvalidName);
    }
    if !is_portable_datapack_name(id) {
        return Err(CommandError::DataPackInvalidFullName);
    }
    let pack_id = format!("file/{id}");
    if state
        .available_data_packs
        .iter()
        .any(|pack| pack == &pack_id)
        || state.created_data_packs.iter().any(|pack| pack.id == id)
    {
        return Err(CommandError::DataPackAlreadyExists);
    }

    state.created_data_packs.push(CreatedDataPack {
        id: id.to_string(),
        description: description.to_string(),
    });
    state.available_data_packs.push(pack_id);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.datapack.create.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn datapack_check_known(
    state: &ServerCommandState,
    id: &str,
) -> Result<(), CommandError> {
    if state.available_data_packs.iter().any(|pack| pack == id) {
        Ok(())
    } else {
        Err(CommandError::DataPackUnknown)
    }
}

pub(super) fn selected_pack_index(
    state: &ServerCommandState,
    id: &str,
) -> Result<usize, CommandError> {
    datapack_check_known(state, id)?;
    state
        .selected_data_packs
        .iter()
        .position(|selected| selected == id)
        .ok_or(CommandError::DataPackAlreadyDisabled)
}

pub(super) fn is_valid_datapack_name(id: &str) -> bool {
    !id.is_empty()
        && !id.contains('/')
        && !id.contains('\\')
        && id != "."
        && id != ".."
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

pub(super) fn is_portable_datapack_name(id: &str) -> bool {
    let upper = id.to_ascii_uppercase();
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    !RESERVED.contains(&upper.as_str()) && !id.ends_with('.') && !id.ends_with(' ')
}

pub(super) fn debug_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["debug", "start"] => {
            if state.debug_profiler_running {
                return Err(CommandError::DebugAlreadyRunning);
            }
            state.debug_profiler_running = true;
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.debug.started",
                broadcast_to_admins: true,
            })
        }
        ["debug", "stop"] => {
            if !state.debug_profiler_running {
                return Err(CommandError::DebugNotRunning);
            }
            state.debug_profiler_running = false;
            let result = state
                .debug_profiler_results
                .pop()
                .unwrap_or(DebugProfilerResult {
                    duration_nanos: 1_000_000_000,
                    tick_duration: 20,
                });
            let tps = if result.duration_nanos == 0 {
                0
            } else {
                ((result.tick_duration as f64) / ((result.duration_nanos as f64) / 1_000_000_000.0))
                    .round() as i32
            };
            Ok(CommandResult {
                success_count: tps,
                feedback_key: "commands.debug.stopped",
                broadcast_to_admins: true,
            })
        }
        ["debug", "function", "return"] => Err(CommandError::DebugNoReturnRun),
        ["debug", "function", "recursive"] => Err(CommandError::DebugNoRecursiveTraces),
        ["debug", "function", function] => {
            state.debug_trace_events.push(DebugTraceEvent {
                function: parse_resource_identifier(function)?,
                output: format!("debug-trace-{}.txt", state.debug_trace_events.len() + 1),
                command_count: state
                    .macro_functions
                    .iter()
                    .filter(|known| known.as_str() == *function)
                    .count()
                    .max(1),
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.debug.function.success.single",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn debug_config_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["debugconfig", "config", target] => {
            let profile = NameAndId::create_offline(target);
            if !state
                .config_players
                .iter()
                .any(|known| known.uuid == profile.uuid)
            {
                state.config_players.push(profile);
            }
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.debugconfig.config",
                broadcast_to_admins: false,
            })
        }
        ["debugconfig", "unconfig", target] => {
            let old_len = state.config_players.len();
            state
                .config_players
                .retain(|known| known.uuid != *target && known.name != *target);
            Ok(CommandResult {
                success_count: if old_len == state.config_players.len() {
                    0
                } else {
                    1
                },
                feedback_key: if old_len == state.config_players.len() {
                    "commands.debugconfig.missing"
                } else {
                    "commands.debugconfig.unconfig"
                },
                broadcast_to_admins: false,
            })
        }
        ["debugconfig", "dialog", target, dialog] => {
            if !state
                .config_players
                .iter()
                .any(|known| known.uuid == *target || known.name == *target)
            {
                return Ok(CommandResult {
                    success_count: 0,
                    feedback_key: "commands.debugconfig.missing",
                    broadcast_to_admins: false,
                });
            }
            state.config_dialog_events.push(DebugConfigDialogEvent {
                target: (*target).to_string(),
                dialog: parse_resource_identifier(dialog)?,
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: "commands.debugconfig.dialog",
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn debug_mob_spawning_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["debugmobspawning", category, x, y, z] => {
            if mob_category(category).is_none() {
                return Err(CommandError::InvalidSyntax);
            }
            let position = parse_block_pos(x, y, z)?;
            state.mob_spawning_events.push(DebugMobSpawningEvent {
                category: (*category).to_string(),
                position,
            });
            Ok(CommandResult {
                success_count: 1,
                feedback_key: crate::command::NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn debug_path_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let ["debugpath", x, y, z] = parts else {
        return Err(CommandError::InvalidSyntax);
    };
    let source = state
        .command_source_entity
        .clone()
        .ok_or(CommandError::DebugPathNotMob)?;
    if matches!(
        entity_kind(state, &source),
        EntityKind::Player | EntityKind::NonLiving
    ) {
        return Err(CommandError::DebugPathNotMob);
    }
    let target = parse_block_pos(x, y, z)?;
    if state.unreachable_debug_paths.contains(&target) {
        return Err(CommandError::DebugPathNoPath);
    }
    if state.incomplete_debug_paths.contains(&target) {
        return Err(CommandError::DebugPathNotComplete);
    }
    state
        .debug_path_events
        .push(DebugPathEvent { source, target });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: crate::command::DEBUG_PATH_SUCCESS_FEEDBACK,
        broadcast_to_admins: true,
    })
}

pub(super) fn default_gamemode_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let ["defaultgamemode", mode] = parts else {
        return Err(CommandError::InvalidSyntax);
    };
    let mode = parse_gamemode(mode)?;
    state.default_game_mode = mode;
    if let Some(force_mode) = state.force_game_mode {
        for player in state.online_players.clone() {
            set_player_gamemode(state, player, force_mode);
        }
    }
    Ok(CommandResult {
        success_count: if state.force_game_mode.is_some() {
            state.online_players.len() as i32
        } else {
            0
        },
        feedback_key: "commands.defaultgamemode.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn difficulty_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["difficulty"] => Ok(CommandResult {
            success_count: state.difficulty.id(),
            feedback_key: "commands.difficulty.query",
            broadcast_to_admins: false,
        }),
        ["difficulty", difficulty] => {
            let difficulty = parse_difficulty(difficulty)?;
            if state.difficulty == difficulty {
                return Err(CommandError::DifficultyAlreadySame);
            }
            state.difficulty = difficulty;
            Ok(CommandResult {
                success_count: 0,
                feedback_key: "commands.difficulty.success",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn dialog_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["dialog", "show", targets, dialog] => {
            let targets = parse_name_list(targets);
            if targets.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            state.dialog_events.push(DialogCommandEvent::Show {
                targets: targets.clone(),
                dialog: parse_dialog_argument(dialog)?,
            });
            Ok(CommandResult {
                success_count: targets.len() as i32,
                feedback_key: if targets.len() == 1 {
                    "commands.dialog.show.single"
                } else {
                    "commands.dialog.show.multiple"
                },
                broadcast_to_admins: true,
            })
        }
        ["dialog", "clear", targets] => {
            let targets = parse_name_list(targets);
            if targets.is_empty() {
                return Err(CommandError::InvalidSyntax);
            }
            state.dialog_events.push(DialogCommandEvent::Clear {
                targets: targets.clone(),
            });
            Ok(CommandResult {
                success_count: targets.len() as i32,
                feedback_key: if targets.len() == 1 {
                    "commands.dialog.clear.single"
                } else {
                    "commands.dialog.clear.multiple"
                },
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn parse_dialog_argument(input: &str) -> Result<DialogCommandDialog, CommandError> {
    if input.starts_with('{') {
        let Tag::Compound(fields) = parse_snbt(input).map_err(|_| CommandError::InvalidSyntax)?
        else {
            return Err(CommandError::InvalidSyntax);
        };
        let title = fields
            .iter()
            .find_map(|(key, value)| match (key.as_str(), value) {
                ("title", Tag::String(title)) => Some(title.clone()),
                _ => None,
            })
            .unwrap_or_default();
        Ok(DialogCommandDialog::Inline { title })
    } else {
        Ok(DialogCommandDialog::Reference(parse_resource_identifier(input)?))
    }
}

pub(super) fn effect_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["effect", "give", targets, effect] => {
            give_effect(state, parse_name_list(targets), effect, None, 0, true)
        }
        ["effect", "give", targets, effect, "infinite"] => {
            give_effect(state, parse_name_list(targets), effect, Some(-1), 0, true)
        }
        ["effect", "give", targets, effect, "infinite", amplifier] => give_effect(
            state,
            parse_name_list(targets),
            effect,
            Some(-1),
            parse_effect_amplifier(amplifier)?,
            true,
        ),
        ["effect", "give", targets, effect, "infinite", amplifier, hide_particles] => give_effect(
            state,
            parse_name_list(targets),
            effect,
            Some(-1),
            parse_effect_amplifier(amplifier)?,
            !parse_bool(hide_particles)?,
        ),
        ["effect", "give", targets, effect, seconds] => give_effect(
            state,
            parse_name_list(targets),
            effect,
            Some(parse_effect_seconds(seconds)?),
            0,
            true,
        ),
        ["effect", "give", targets, effect, seconds, amplifier] => give_effect(
            state,
            parse_name_list(targets),
            effect,
            Some(parse_effect_seconds(seconds)?),
            parse_effect_amplifier(amplifier)?,
            true,
        ),
        ["effect", "give", targets, effect, seconds, amplifier, hide_particles] => give_effect(
            state,
            parse_name_list(targets),
            effect,
            Some(parse_effect_seconds(seconds)?),
            parse_effect_amplifier(amplifier)?,
            !parse_bool(hide_particles)?,
        ),
        ["effect", "clear"] => {
            let source = state
                .command_source_entity
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            clear_all_effects(state, vec![source])
        }
        ["effect", "clear", targets] => clear_all_effects(
            state,
            parse_name_list(targets)
                .into_iter()
                .map(|profile| entity_ref(&profile.name))
                .collect(),
        ),
        ["effect", "clear", targets, effect] => clear_specific_effect(
            state,
            parse_name_list(targets)
                .into_iter()
                .map(|profile| entity_ref(&profile.name))
                .collect(),
            &parse_resource_identifier(effect)?,
        ),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn give_effect(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    effect: &str,
    seconds: Option<i32>,
    amplifier: u8,
    show_particles: bool,
) -> Result<CommandResult, CommandError> {
    let effect = parse_resource_identifier(effect)?;
    let duration_ticks = effect_duration_ticks(&effect, seconds);
    let mut count = 0;
    for target in targets.iter().map(|profile| entity_ref(&profile.name)) {
        if matches!(entity_kind(state, &target), EntityKind::NonLiving) {
            continue;
        }
        upsert_active_effect(
            state,
            ActiveEffect {
                target,
                effect: effect.clone(),
                duration_ticks,
                amplifier,
                show_particles,
            },
        );
        count += 1;
    }
    if count == 0 {
        return Err(CommandError::EffectGiveFailed);
    }
    Ok(CommandResult {
        success_count: count,
        feedback_key: if targets.len() == 1 {
            "commands.effect.give.success.single"
        } else {
            "commands.effect.give.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn clear_all_effects(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
) -> Result<CommandResult, CommandError> {
    let mut count = 0;
    for target in &targets {
        if matches!(entity_kind(state, target), EntityKind::NonLiving) {
            continue;
        }
        let before = state.active_effects.len();
        state
            .active_effects
            .retain(|effect| effect.target.id != target.id);
        if state.active_effects.len() != before {
            count += 1;
        }
    }
    if count == 0 {
        return Err(CommandError::EffectClearEverythingFailed);
    }
    Ok(CommandResult {
        success_count: count,
        feedback_key: if targets.len() == 1 {
            "commands.effect.clear.everything.success.single"
        } else {
            "commands.effect.clear.everything.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn clear_specific_effect(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
    effect: &str,
) -> Result<CommandResult, CommandError> {
    let mut count = 0;
    for target in &targets {
        if matches!(entity_kind(state, target), EntityKind::NonLiving) {
            continue;
        }
        let before = state.active_effects.len();
        state
            .active_effects
            .retain(|active| active.target.id != target.id || active.effect != effect);
        if state.active_effects.len() != before {
            count += 1;
        }
    }
    if count == 0 {
        return Err(CommandError::EffectClearSpecificFailed);
    }
    Ok(CommandResult {
        success_count: count,
        feedback_key: if targets.len() == 1 {
            "commands.effect.clear.specific.success.single"
        } else {
            "commands.effect.clear.specific.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn upsert_active_effect(state: &mut ServerCommandState, effect: ActiveEffect) {
    if let Some(existing) = state
        .active_effects
        .iter_mut()
        .find(|active| active.target.id == effect.target.id && active.effect == effect.effect)
    {
        *existing = effect;
    } else {
        state.active_effects.push(effect);
    }
}

pub(super) fn effect_duration_ticks(effect: &str, seconds: Option<i32>) -> i32 {
    match seconds {
        Some(-1) => -1,
        Some(seconds) if is_instant_effect(effect) => seconds,
        Some(seconds) => seconds * 20,
        None if is_instant_effect(effect) => 1,
        None => 600,
    }
}

pub(super) fn is_instant_effect(effect: &str) -> bool {
    matches!(
        effect,
        "minecraft:instant_health" | "minecraft:instant_damage" | "minecraft:saturation"
    )
}

pub(super) fn parse_effect_seconds(input: &str) -> Result<i32, CommandError> {
    let seconds = input
        .parse::<i32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if (1..=1_000_000).contains(&seconds) {
        Ok(seconds)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_effect_amplifier(input: &str) -> Result<u8, CommandError> {
    input.parse::<u8>().map_err(|_| CommandError::InvalidSyntax)
}
