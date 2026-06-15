use super::*;

pub(super) fn setworldspawn_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (position, yaw, pitch) = match parts {
        ["setworldspawn"] => (
            block_pos_containing(state.command_source_position),
            0.0,
            0.0,
        ),
        ["setworldspawn", x, y, z] => (parse_block_pos(x, y, z)?, 0.0, 0.0),
        ["setworldspawn", x, y, z, yaw, pitch] => (
            parse_block_pos(x, y, z)?,
            parse_f32(yaw)?,
            parse_f32(pitch)?,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    state.world_spawn = RespawnData {
        dimension: state.command_source_dimension.clone(),
        position,
        yaw,
        pitch,
    };
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.setworldspawn.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn spawnpoint_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (targets, position, yaw, pitch) = match parts {
        ["spawnpoint"] => (
            vec![state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?],
            block_pos_containing(state.command_source_position),
            0.0,
            0.0,
        ),
        ["spawnpoint", targets] => (
            parse_name_list(targets),
            block_pos_containing(state.command_source_position),
            0.0,
            0.0,
        ),
        ["spawnpoint", targets, x, y, z] => (
            parse_name_list(targets),
            parse_block_pos(x, y, z)?,
            0.0,
            0.0,
        ),
        ["spawnpoint", targets, x, y, z, yaw, pitch] => (
            parse_name_list(targets),
            parse_block_pos(x, y, z)?,
            wrap_degrees(parse_f32(yaw)?),
            parse_f32(pitch)?.clamp(-90.0, 90.0),
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };
    if targets.is_empty() {
        return Err(CommandError::NoPlayers);
    }
    let count = targets.len() as i32;
    for target in targets {
        set_player_spawn(
            state,
            target,
            RespawnData {
                dimension: state.command_source_dimension.clone(),
                position,
                yaw,
                pitch,
            },
        );
    }
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            "commands.spawnpoint.success.single"
        } else {
            "commands.spawnpoint.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn set_player_spawn(
    state: &mut ServerCommandState,
    player: NameAndId,
    respawn: RespawnData,
) {
    if let Some(existing) = state
        .player_spawns
        .iter_mut()
        .find(|spawn| spawn.player.uuid == player.uuid)
    {
        existing.respawn = respawn;
        existing.forced = true;
    } else {
        state.player_spawns.push(PlayerSpawn {
            player,
            respawn,
            forced: true,
        });
    }
}

pub(super) fn spawn_armor_trims_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    state
        .command_source_player
        .as_ref()
        .ok_or(CommandError::InvalidSyntax)?;
    let patterns: Vec<&'static str> = match parts {
        ["spawn_armor_trims", "*_lag_my_game"] => VANILLA_TRIM_PATTERNS.to_vec(),
        ["spawn_armor_trims", pattern] => {
            let pattern = parse_resource_identifier(pattern)?;
            let Some(pattern) = VANILLA_TRIM_PATTERNS
                .iter()
                .copied()
                .find(|entry| *entry == pattern)
            else {
                return Err(CommandError::InvalidArmorTrimPattern);
            };
            vec![pattern]
        }
        _ => return Err(CommandError::InvalidSyntax),
    };

    let origin = Vec3 {
        x: state.command_source_position.x.floor() + 0.5,
        y: state.command_source_position.y.floor() + 0.5,
        z: state.command_source_position.z.floor() + 5.5,
    };
    for (material_index, material) in VANILLA_TRIM_MATERIALS.iter().enumerate() {
        for (pattern_index, pattern) in patterns.iter().enumerate() {
            for (item_index, item) in TRIMMABLE_ARMOR_ITEMS.iter().enumerate() {
                state.armor_trim_spawns.push(ArmorTrimSpawn {
                    pattern: (*pattern).to_string(),
                    material: (*material).to_string(),
                    item: (*item).to_string(),
                    position: Vec3 {
                        x: origin.x - item_index as f64 * 3.0,
                        y: origin.y + material_index as f64 * 3.0,
                        z: origin.z + pattern_index as f64 * 10.0,
                    },
                    named: item_index == 0,
                    invisible: item_index != 0,
                });
            }
        }
    }

    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.spawn_armor_trims.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn spreadplayers_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (center_x, center_z, spread_distance, max_range, max_height, respect_teams, targets) =
        match parts {
            ["spreadplayers", x, z, spread, range, "under", height, respect, targets @ ..]
                if !targets.is_empty() =>
            {
                (
                    parse_f64(x)?,
                    parse_f64(z)?,
                    parse_non_negative_f32(spread)? as f64,
                    parse_positive_f32(range)? as f64,
                    height
                        .parse::<i32>()
                        .map_err(|_| CommandError::InvalidSyntax)?,
                    parse_bool(respect)?,
                    targets,
                )
            }
            ["spreadplayers", x, z, spread, range, respect, targets @ ..]
                if !targets.is_empty() =>
            {
                (
                    parse_f64(x)?,
                    parse_f64(z)?,
                    parse_non_negative_f32(spread)? as f64,
                    parse_positive_f32(range)? as f64,
                    320,
                    parse_bool(respect)?,
                    targets,
                )
            }
            _ => return Err(CommandError::InvalidSyntax),
        };
    if max_height < -64 {
        return Err(CommandError::SpreadPlayersInvalidMaxHeight);
    }
    let targets = targets
        .iter()
        .map(|target| entity_ref(target))
        .collect::<Vec<_>>();
    let groups = spread_groups(state, &targets, respect_teams);
    if groups.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    if groups.len() > 1 && max_range * 2.0 < spread_distance {
        return Err(if respect_teams {
            CommandError::SpreadPlayersFailedTeams
        } else {
            CommandError::SpreadPlayersFailedEntities
        });
    }
    let positions = spread_positions(center_x, center_z, max_range, groups.len(), max_height);
    for (group_index, group) in groups.iter().enumerate() {
        for target in group {
            upsert_entity_position(
                state,
                target.clone(),
                Vec3 {
                    x: positions[group_index].x,
                    y: positions[group_index].y,
                    z: positions[group_index].z,
                },
            );
        }
    }
    Ok(CommandResult {
        success_count: groups.len() as i32,
        feedback_key: if respect_teams {
            "commands.spreadplayers.success.teams"
        } else {
            "commands.spreadplayers.success.entities"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn spread_groups(
    state: &ServerCommandState,
    targets: &[EntityRef],
    respect_teams: bool,
) -> Vec<Vec<EntityRef>> {
    if !respect_teams {
        return targets.iter().cloned().map(|target| vec![target]).collect();
    }
    let mut groups: Vec<(Option<String>, Vec<EntityRef>)> = Vec::new();
    for target in targets {
        let team = state
            .player_teams
            .iter()
            .find(|membership| {
                membership.player.name == target.id || membership.player.uuid == target.id
            })
            .map(|membership| membership.team.clone());
        if let Some((_, members)) = groups
            .iter_mut()
            .find(|(entry_team, _)| *entry_team == team)
        {
            members.push(target.clone());
        } else {
            groups.push((team, vec![target.clone()]));
        }
    }
    groups.into_iter().map(|(_, members)| members).collect()
}

pub(super) fn spread_positions(
    center_x: f64,
    center_z: f64,
    max_range: f64,
    count: usize,
    max_height: i32,
) -> Vec<Vec3> {
    let radius = max_range.max(0.0);
    let y = (max_height + 1) as f64;
    if count == 1 {
        return vec![Vec3 {
            x: center_x.floor() + 0.5,
            y,
            z: center_z.floor() + 0.5,
        }];
    }
    (0..count)
        .map(|index| {
            let angle = (index as f64 / count as f64) * std::f64::consts::TAU;
            Vec3 {
                x: (center_x + angle.cos() * radius).floor() + 0.5,
                y,
                z: (center_z + angle.sin() * radius).floor() + 0.5,
            }
        })
        .collect()
}

pub(super) fn upsert_entity_position(
    state: &mut ServerCommandState,
    entity: EntityRef,
    position: Vec3,
) {
    let dimension = state.command_source_dimension.clone();
    if let Some(existing) = state
        .entity_positions
        .iter_mut()
        .find(|entry| entry.entity.id == entity.id)
    {
        existing.dimension = dimension;
        existing.position = position;
    } else {
        state.entity_positions.push(EntityPosition {
            entity,
            dimension,
            position,
        });
    }
}

pub(super) fn spectate_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (target, player) = match parts {
        ["spectate"] => (
            None,
            state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?,
        ),
        ["spectate", target] => (
            Some(entity_ref(target)),
            state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?,
        ),
        ["spectate", target, player] => {
            (Some(entity_ref(target)), NameAndId::create_offline(player))
        }
        _ => return Err(CommandError::InvalidSyntax),
    };
    if target
        .as_ref()
        .is_some_and(|target| target.id == player.name || target.id == player.uuid)
    {
        return Err(CommandError::SpectateSelf);
    }
    if player_gamemode(state, &player) != GameMode::Spectator {
        return Err(CommandError::SpectateNotSpectator);
    }
    if let Some(target) = &target {
        if state
            .untrackable_entities
            .iter()
            .any(|entity| entity.id == target.id)
        {
            return Err(CommandError::SpectateCannotSpectate);
        }
    }
    set_camera_target(state, player, target.clone());
    Ok(CommandResult {
        success_count: 1,
        feedback_key: if target.is_some() {
            "commands.spectate.success.started"
        } else {
            "commands.spectate.success.stopped"
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn player_gamemode(state: &ServerCommandState, player: &NameAndId) -> GameMode {
    state
        .player_game_modes
        .iter()
        .find(|entry| entry.player.uuid == player.uuid)
        .map(|entry| entry.gamemode)
        .unwrap_or(GameMode::Survival)
}

pub(super) fn set_player_gamemode(
    state: &mut ServerCommandState,
    player: NameAndId,
    gamemode: GameMode,
) {
    if let Some(existing) = state
        .player_game_modes
        .iter_mut()
        .find(|entry| entry.player.uuid == player.uuid)
    {
        existing.gamemode = gamemode;
    } else {
        state
            .player_game_modes
            .push(PlayerGameMode { player, gamemode });
    }
}

pub(super) fn default_game_rules() -> Vec<GameRuleState> {
    default_game_rules_with_features(false)
}

pub(super) fn default_game_rules_with_features(
    minecart_improvements_enabled: bool,
) -> Vec<GameRuleState> {
    VANILLA_GAME_RULES
        .iter()
        .filter(|definition| {
            !definition.requires_minecart_improvements || minecart_improvements_enabled
        })
        .map(|definition| GameRuleState {
            name: definition.name.to_string(),
            value: definition.default,
        })
        .collect()
}

pub(super) fn normalize_game_rule_name(rule: &str) -> String {
    let rule = rule.strip_prefix("minecraft:").unwrap_or(rule);
    match rule {
        "commandBlockOutput" => "command_block_output",
        "doDaylightCycle" => "advance_time",
        "doEntityDrops" => "entity_drops",
        "doImmediateRespawn" => "immediate_respawn",
        "doInsomnia" => "spawn_phantoms",
        "doLimitedCrafting" => "limited_crafting",
        "doMobLoot" => "mob_drops",
        "doMobSpawning" => "spawn_mobs",
        "doPatrolSpawning" => "spawn_patrols",
        "doTileDrops" => "block_drops",
        "doTraderSpawning" => "spawn_wandering_traders",
        "doVinesSpread" => "spread_vines",
        "doWardenSpawning" => "spawn_wardens",
        "doWeatherCycle" => "advance_weather",
        "drowningDamage" => "drowning_damage",
        "fallDamage" => "fall_damage",
        "fireDamage" => "fire_damage",
        "forgiveDeadPlayers" => "forgive_dead_players",
        "freezeDamage" => "freeze_damage",
        "globalSoundEvents" => "global_sound_events",
        "keepInventory" => "keep_inventory",
        "logAdminCommands" => "log_admin_commands",
        "maxCommandChainLength" => "max_command_sequence_length",
        "maxCommandForkCount" => "max_command_forks",
        "maxEntityCramming" => "max_entity_cramming",
        "mobGriefing" => "mob_griefing",
        "naturalRegeneration" => "natural_health_regeneration",
        "playersSleepingPercentage" => "players_sleeping_percentage",
        "randomTickSpeed" => "random_tick_speed",
        "reducedDebugInfo" => "reduced_debug_info",
        "sendCommandFeedback" => "send_command_feedback",
        "showDeathMessages" => "show_death_messages",
        "spawnRadius" => "respawn_radius",
        "spectatorsGenerateChunks" => "spectators_generate_chunks",
        "tntExplodes" => "tnt_explodes",
        "universalAnger" => "universal_anger",
        _ => rule,
    }
    .to_string()
}

pub(super) fn game_rule_value(
    state: &ServerCommandState,
    rule: &str,
) -> Result<GameRuleValue, CommandError> {
    let normalized = normalize_game_rule_name(rule);
    state
        .game_rules
        .iter()
        .find(|entry| entry.name == normalized)
        .map(|entry| entry.value)
        .ok_or(CommandError::InvalidSyntax)
}

pub(super) fn set_game_rule_value(
    state: &mut ServerCommandState,
    rule: String,
    value: GameRuleValue,
) {
    if let Some(existing) = state.game_rules.iter_mut().find(|entry| entry.name == rule) {
        existing.value = value;
    } else {
        state.game_rules.push(GameRuleState { name: rule, value });
    }
}

pub(super) fn parse_game_rule_value(
    input: &str,
    current: &GameRuleValue,
    definition: &GameRuleDefinition,
) -> Result<GameRuleValue, CommandError> {
    match current {
        GameRuleValue::Bool(_) => Ok(GameRuleValue::Bool(parse_bool(input)?)),
        GameRuleValue::Int(_) => {
            let value = input
                .parse::<i32>()
                .map_err(|_| CommandError::InvalidSyntax)?;
            if definition.min.is_some_and(|min| value < min)
                || definition.max.is_some_and(|max| value > max)
            {
                return Err(CommandError::InvalidSyntax);
            }
            Ok(GameRuleValue::Int(value))
        }
    }
}

pub(super) fn game_rule_definition(
    rule: &str,
) -> Result<&'static GameRuleDefinition, CommandError> {
    let normalized = normalize_game_rule_name(rule);
    VANILLA_GAME_RULES
        .iter()
        .find(|definition| definition.name == normalized)
        .ok_or(CommandError::InvalidSyntax)
}

impl GameRuleValue {
    pub(super) fn command_result(&self) -> i32 {
        match self {
            Self::Bool(value) => i32::from(*value),
            Self::Int(value) => *value,
        }
    }

    pub(super) fn sync_value(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
        }
    }
}

pub(super) fn set_camera_target(
    state: &mut ServerCommandState,
    player: NameAndId,
    target: Option<EntityRef>,
) {
    if let Some(existing) = state
        .camera_targets
        .iter_mut()
        .find(|entry| entry.player.uuid == player.uuid)
    {
        existing.target = target;
    } else {
        state.camera_targets.push(CameraTarget { player, target });
    }
}

pub(super) fn summon_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    let (entity_type, position, nbt, finalized_spawn) = match parts {
        ["summon", entity] => (
            parse_resource_identifier(entity)?,
            state.command_source_position,
            None,
            true,
        ),
        ["summon", entity, x, y, z] => (
            parse_resource_identifier(entity)?,
            parse_vec3(x, y, z)?,
            None,
            true,
        ),
        ["summon", entity, x, y, z, nbt] => (
            parse_resource_identifier(entity)?,
            parse_vec3(x, y, z)?,
            Some((*nbt).to_string()),
            false,
        ),
        _ => return Err(CommandError::InvalidSyntax),
    };

    if !is_in_spawnable_bounds(block_pos_containing(position)) {
        return Err(CommandError::SummonInvalidPosition);
    }
    let entity_id = summoned_entity_id(&entity_type, &position, nbt.as_deref());
    if state
        .entity_states
        .iter()
        .any(|entry| entry.entity.id == entity_id)
        || state
            .summoned_entities
            .iter()
            .any(|entry| entry.entity.id == entity_id)
    {
        return Err(CommandError::SummonDuplicateUuid);
    }

    let entity = EntityRef {
        id: entity_id,
        display_name: entity_type.clone(),
    };
    state.entity_states.push(EntityState {
        entity: entity.clone(),
        kind: EntityKind::Generic,
        dimension: state.command_source_dimension.clone(),
    });
    state.summoned_entities.push(SummonedEntity {
        entity_type,
        entity,
        position,
        nbt,
        finalized_spawn,
    });
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.summon.success",
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_vec3(x: &str, y: &str, z: &str) -> Result<Vec3, CommandError> {
    Ok(Vec3 {
        x: parse_f64(x)?,
        y: parse_f64(y)?,
        z: parse_f64(z)?,
    })
}

pub(super) fn parse_resource_identifier(input: &str) -> Result<String, CommandError> {
    let identifier = parse_identifier(input)?;
    if identifier.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(CommandError::InvalidSyntax);
    }
    if identifier.contains(':') {
        Ok(identifier)
    } else {
        Ok(format!("minecraft:{identifier}"))
    }
}

pub(super) fn is_in_spawnable_bounds(pos: BlockPos) -> bool {
    pos.x >= -30_000_000
        && pos.z >= -30_000_000
        && pos.x < 30_000_000
        && pos.z < 30_000_000
        && pos.y >= -20_000_000
        && pos.y < 20_000_000
}

pub(super) fn summoned_entity_id(entity_type: &str, position: &Vec3, nbt: Option<&str>) -> String {
    if let Some(uuid) = nbt.and_then(extract_uuid_from_nbt) {
        return uuid;
    }
    format!(
        "{}@{:.3},{:.3},{:.3}",
        entity_type, position.x, position.y, position.z
    )
}

pub(super) fn extract_uuid_from_nbt(nbt: &str) -> Option<String> {
    let marker = "UUID:";
    let start = nbt.find(marker)? + marker.len();
    let tail = &nbt[start..];
    let end = tail
        .find(|ch: char| ch == ',' || ch == '}' || ch.is_whitespace())
        .unwrap_or(tail.len());
    let uuid = tail[..end].trim_matches('"');
    if uuid.is_empty() {
        None
    } else {
        Some(uuid.to_string())
    }
}
