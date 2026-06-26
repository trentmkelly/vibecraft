use super::*;
use crate::random_source::LegacyRandom;

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
        ["setworldspawn", x, y, z] => (parse_spawnable_block_pos(state, x, y, z)?, 0.0, 0.0),
        ["setworldspawn", x, y, z, yaw, pitch] => (
            parse_spawnable_block_pos(state, x, y, z)?,
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
            parse_spawnable_block_pos(state, x, y, z)?,
            0.0,
            0.0,
        ),
        ["spawnpoint", targets, x, y, z, yaw, pitch] => (
            parse_name_list(targets),
            parse_spawnable_block_pos(state, x, y, z)?,
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

    let (step_x, step_z) = horizontal_direction_step_from_y_rot(state.command_source_yaw);
    let origin = Vec3 {
        x: state.command_source_position.x.floor() + f64::from(step_x * 5) + 0.5,
        y: state.command_source_position.y.floor() + 0.5,
        z: state.command_source_position.z.floor() + f64::from(step_z * 5) + 0.5,
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
                    y_rot: 180.0,
                    no_gravity: true,
                    named: item_index == 0,
                    custom_name_visible: item_index == 0,
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

fn horizontal_direction_step_from_y_rot(y_rot: f32) -> (i32, i32) {
    match ((y_rot / 90.0 + 0.5).floor() as i32).rem_euclid(4) {
        0 => (0, 1),
        1 => (-1, 0),
        2 => (0, -1),
        _ => (1, 0),
    }
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
                    parse_spreadplayers_coordinate(x, state.command_source_position.x)?,
                    parse_spreadplayers_coordinate(z, state.command_source_position.z)?,
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
                    parse_spreadplayers_coordinate(x, state.command_source_position.x)?,
                    parse_spreadplayers_coordinate(z, state.command_source_position.z)?,
                    parse_non_negative_f32(spread)? as f64,
                    parse_positive_f32(range)? as f64,
                    321,
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
    let mut random = LegacyRandom::new(state.world_seed ^ state.game_time_ticks as i64);
    let mut positions = create_initial_spread_positions(
        &mut random,
        groups.len(),
        center_x - max_range,
        center_z - max_range,
        center_x + max_range,
        center_z + max_range,
    );
    spread_positions_java(
        state,
        SpreadConfig {
            center_x,
            center_z,
            spread_distance,
            max_range,
            max_height,
            respect_teams,
        },
        &mut random,
        &mut positions,
    )?;
    for (group_index, group) in groups.iter().enumerate() {
        for target in group {
            upsert_entity_position(
                state,
                target.clone(),
                positions[group_index].teleport_position(state, max_height),
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
        let player_team = state.player_teams.iter().find_map(|membership| {
            (membership.player.name == target.id || membership.player.uuid == target.id)
                .then(|| membership.team.clone())
        });
        let team = if player_team.is_some()
            || state
                .online_players
                .iter()
                .any(|player| player.name == target.id || player.uuid == target.id)
        {
            player_team
        } else {
            None
        };
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

#[derive(Debug, Clone, Copy)]
struct SpreadPosition {
    x: f64,
    z: f64,
}

impl SpreadPosition {
    fn new() -> Self {
        Self { x: 0.0, z: 0.0 }
    }

    fn dist(self, target: Self) -> f64 {
        let dx = self.x - target.x;
        let dz = self.z - target.z;
        (dx * dx + dz * dz).sqrt()
    }

    fn length(self) -> f64 {
        (self.x * self.x + self.z * self.z).sqrt()
    }

    fn normalize(&mut self) {
        let distance = self.length();
        self.x /= distance;
        self.z /= distance;
    }

    fn move_away(&mut self, pos: Self) {
        self.x -= pos.x;
        self.z -= pos.z;
    }

    fn clamp(&mut self, min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> bool {
        let mut changed = false;
        if self.x < min_x {
            self.x = min_x;
            changed = true;
        } else if self.x > max_x {
            self.x = max_x;
            changed = true;
        }
        if self.z < min_z {
            self.z = min_z;
            changed = true;
        } else if self.z > max_z {
            self.z = max_z;
            changed = true;
        }
        changed
    }

    fn randomize(
        &mut self,
        random: &mut LegacyRandom,
        min_x: f64,
        min_z: f64,
        max_x: f64,
        max_z: f64,
    ) {
        self.x = next_double_between(random, min_x, max_x);
        self.z = next_double_between(random, min_z, max_z);
    }

    fn spawn_y(self, state: &ServerCommandState, max_height: i32) -> i32 {
        let x = self.x.floor() as i32;
        let z = self.z.floor() as i32;
        let mut y = max_height + 1;
        let mut air_two_above = spread_block_is_air(state, x, y, z);
        y -= 1;
        let mut air_one_above = spread_block_is_air(state, x, y, z);
        while y > SPREADPLAYERS_MIN_Y {
            y -= 1;
            let current_is_air = spread_block_is_air(state, x, y, z);
            if !current_is_air && air_one_above && air_two_above {
                return y + 1;
            }
            air_two_above = air_one_above;
            air_one_above = current_is_air;
        }
        max_height + 1
    }

    fn is_safe(self, state: &ServerCommandState, max_height: i32) -> bool {
        let y = self.spawn_y(state, max_height) - 1;
        let block = spread_block_at(state, self.x.floor() as i32, y, self.z.floor() as i32);
        y < max_height && !spread_block_is_liquid(block) && !spread_block_is_fire(block)
    }

    fn teleport_position(self, state: &ServerCommandState, max_height: i32) -> Vec3 {
        Vec3 {
            x: self.x.floor() + 0.5,
            y: self.spawn_y(state, max_height) as f64,
            z: self.z.floor() + 0.5,
        }
    }
}

const SPREADPLAYERS_MAX_ITERATION_COUNT: usize = 10_000;
const SPREADPLAYERS_MIN_Y: i32 = -64;

#[derive(Debug, Clone, Copy)]
struct SpreadConfig {
    center_x: f64,
    center_z: f64,
    spread_distance: f64,
    max_range: f64,
    max_height: i32,
    respect_teams: bool,
}

fn parse_spreadplayers_coordinate(input: &str, source: f64) -> Result<f64, CommandError> {
    if input == "~" {
        Ok(source)
    } else if let Some(offset) = input.strip_prefix('~') {
        Ok(source + parse_f64(if offset.is_empty() { "0" } else { offset })?)
    } else {
        parse_f64(input)
    }
}

fn create_initial_spread_positions(
    random: &mut LegacyRandom,
    count: usize,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
) -> Vec<SpreadPosition> {
    let mut positions = Vec::with_capacity(count);
    for _ in 0..count {
        let mut position = SpreadPosition::new();
        position.randomize(random, min_x, min_z, max_x, max_z);
        positions.push(position);
    }
    positions
}

fn spread_positions_java(
    state: &ServerCommandState,
    config: SpreadConfig,
    random: &mut LegacyRandom,
    positions: &mut [SpreadPosition],
) -> Result<(), CommandError> {
    let min_x = config.center_x - config.max_range;
    let min_z = config.center_z - config.max_range;
    let max_x = config.center_x + config.max_range;
    let max_z = config.center_z + config.max_range;
    let mut has_collisions = true;
    let mut min_distance = f64::from(f32::MAX);
    let mut iteration = 0;

    while iteration < SPREADPLAYERS_MAX_ITERATION_COUNT && has_collisions {
        has_collisions = false;
        min_distance = f64::from(f32::MAX);

        for i in 0..positions.len() {
            let mut neighbour_count = 0;
            let mut average_neighbour_pos = SpreadPosition::new();

            for j in 0..positions.len() {
                if i != j {
                    let dist = positions[i].dist(positions[j]);
                    min_distance = min_distance.min(dist);
                    if dist < config.spread_distance {
                        neighbour_count += 1;
                        average_neighbour_pos.x += positions[j].x - positions[i].x;
                        average_neighbour_pos.z += positions[j].z - positions[i].z;
                    }
                }
            }

            if neighbour_count > 0 {
                average_neighbour_pos.x /= f64::from(neighbour_count);
                average_neighbour_pos.z /= f64::from(neighbour_count);
                if average_neighbour_pos.length() > 0.0 {
                    average_neighbour_pos.normalize();
                    positions[i].move_away(average_neighbour_pos);
                } else {
                    positions[i].randomize(random, min_x, min_z, max_x, max_z);
                }
                has_collisions = true;
            }

            if positions[i].clamp(min_x, min_z, max_x, max_z) {
                has_collisions = true;
            }
        }

        if !has_collisions {
            for position in positions.iter_mut() {
                if !position.is_safe(state, config.max_height) {
                    position.randomize(random, min_x, min_z, max_x, max_z);
                    has_collisions = true;
                }
            }
        }

        iteration += 1;
    }

    if min_distance == f64::from(f32::MAX) {
        min_distance = 0.0;
    }

    if iteration >= SPREADPLAYERS_MAX_ITERATION_COUNT {
        return Err(if config.respect_teams {
            CommandError::SpreadPlayersFailedTeams
        } else {
            CommandError::SpreadPlayersFailedEntities
        });
    }
    let _recommended_distance = min_distance;
    Ok(())
}

fn next_double_between(random: &mut LegacyRandom, min: f64, max: f64) -> f64 {
    min + random.next_f64() * (max - min)
}

fn spread_block_at(state: &ServerCommandState, x: i32, y: i32, z: i32) -> &str {
    state
        .blocks
        .iter()
        .rev()
        .find(|entry| {
            entry.dimension == state.command_source_dimension
                && entry.position
                    == BlockPos {
                        x,
                        y,
                        z,
                    }
        })
        .map_or_else(
            || {
                if y == 0 {
                    "minecraft:grass_block"
                } else {
                    "minecraft:air"
                }
            },
            |entry| entry.block.as_str(),
        )
}

fn spread_block_is_air(state: &ServerCommandState, x: i32, y: i32, z: i32) -> bool {
    spread_block_at(state, x, y, z) == "minecraft:air"
}

fn spread_block_is_liquid(block: &str) -> bool {
    matches!(block, "minecraft:water" | "minecraft:lava")
}

fn spread_block_is_fire(block: &str) -> bool {
    matches!(
        block,
        "minecraft:fire" | "minecraft:soul_fire" | "minecraft:campfire" | "minecraft:soul_campfire"
    )
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

    let allowed_in_peaceful = validate_summon_entity_type(&entity_type)?;
    if state.difficulty == Difficulty::Peaceful && !allowed_in_peaceful {
        return Err(CommandError::SummonFailedPeaceful);
    }
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

pub(super) fn validate_summon_entity_type(entity_type: &str) -> Result<bool, CommandError> {
    if !contains_whitespace_token(SUMMON_ENTITY_TYPES_26_1_2, entity_type)
        || contains_whitespace_token(NON_SUMMONABLE_ENTITY_TYPES_26_1_2, entity_type)
    {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(!contains_whitespace_token(
        NOT_IN_PEACEFUL_ENTITY_TYPES_26_1_2,
        entity_type,
    ))
}

fn contains_whitespace_token(haystack: &str, needle: &str) -> bool {
    haystack.split_whitespace().any(|token| token == needle)
}

pub(super) const SUMMON_ENTITY_TYPES_26_1_2: &str = "
minecraft:acacia_boat minecraft:acacia_chest_boat minecraft:allay minecraft:area_effect_cloud
minecraft:armadillo minecraft:armor_stand minecraft:arrow minecraft:axolotl minecraft:bamboo_chest_raft
minecraft:bamboo_raft minecraft:bat minecraft:bee minecraft:birch_boat minecraft:birch_chest_boat
minecraft:blaze minecraft:block_display minecraft:bogged minecraft:breeze minecraft:breeze_wind_charge
minecraft:camel minecraft:camel_husk minecraft:cat minecraft:cave_spider minecraft:cherry_boat
minecraft:cherry_chest_boat minecraft:chest_minecart minecraft:chicken minecraft:cod minecraft:copper_golem
minecraft:command_block_minecart minecraft:cow minecraft:creaking minecraft:creeper minecraft:dark_oak_boat
minecraft:dark_oak_chest_boat minecraft:dolphin minecraft:donkey minecraft:dragon_fireball minecraft:drowned
minecraft:egg minecraft:elder_guardian minecraft:enderman minecraft:endermite minecraft:ender_dragon
minecraft:ender_pearl minecraft:end_crystal minecraft:evoker minecraft:evoker_fangs minecraft:experience_bottle
minecraft:experience_orb minecraft:eye_of_ender minecraft:falling_block minecraft:fireball minecraft:firework_rocket
minecraft:fox minecraft:frog minecraft:furnace_minecart minecraft:ghast minecraft:happy_ghast minecraft:giant
minecraft:glow_item_frame minecraft:glow_squid minecraft:goat minecraft:guardian minecraft:hoglin minecraft:hopper_minecart
minecraft:horse minecraft:husk minecraft:illusioner minecraft:interaction minecraft:iron_golem minecraft:item minecraft:item_display
minecraft:item_frame minecraft:jungle_boat minecraft:jungle_chest_boat minecraft:leash_knot minecraft:lightning_bolt minecraft:llama
minecraft:llama_spit minecraft:magma_cube minecraft:mangrove_boat minecraft:mangrove_chest_boat minecraft:mannequin
minecraft:marker minecraft:minecart minecraft:mooshroom minecraft:mule minecraft:nautilus minecraft:oak_boat minecraft:oak_chest_boat
minecraft:ocelot minecraft:ominous_item_spawner minecraft:painting minecraft:pale_oak_boat minecraft:pale_oak_chest_boat
minecraft:panda minecraft:parched minecraft:parrot minecraft:phantom minecraft:pig minecraft:piglin minecraft:piglin_brute
minecraft:pillager minecraft:polar_bear minecraft:splash_potion minecraft:lingering_potion minecraft:pufferfish minecraft:rabbit
minecraft:ravager minecraft:salmon minecraft:sheep minecraft:shulker minecraft:shulker_bullet minecraft:silverfish minecraft:skeleton
minecraft:skeleton_horse minecraft:slime minecraft:small_fireball minecraft:sniffer minecraft:snowball minecraft:snow_golem
minecraft:spawner_minecart minecraft:spectral_arrow minecraft:spider minecraft:spruce_boat minecraft:spruce_chest_boat minecraft:squid
minecraft:stray minecraft:strider minecraft:tadpole minecraft:text_display minecraft:tnt minecraft:tnt_minecart minecraft:trader_llama
minecraft:trident minecraft:tropical_fish minecraft:turtle minecraft:vex minecraft:villager minecraft:vindicator minecraft:wandering_trader
minecraft:warden minecraft:wind_charge minecraft:witch minecraft:wither minecraft:wither_skeleton minecraft:wither_skull minecraft:wolf
minecraft:zoglin minecraft:zombie minecraft:zombie_horse minecraft:zombie_nautilus minecraft:zombie_villager minecraft:zombified_piglin
minecraft:player minecraft:fishing_bobber
";

pub(super) const NON_SUMMONABLE_ENTITY_TYPES_26_1_2: &str =
    "minecraft:player minecraft:fishing_bobber";

pub(super) const NOT_IN_PEACEFUL_ENTITY_TYPES_26_1_2: &str = "
minecraft:blaze minecraft:bogged minecraft:breeze minecraft:cave_spider minecraft:creaking minecraft:creeper
minecraft:drowned minecraft:elder_guardian minecraft:enderman minecraft:endermite minecraft:evoker minecraft:ghast
minecraft:giant minecraft:guardian minecraft:husk minecraft:illusioner minecraft:magma_cube minecraft:parched
minecraft:phantom minecraft:piglin_brute minecraft:pillager minecraft:ravager minecraft:silverfish minecraft:skeleton
minecraft:slime minecraft:spider minecraft:stray minecraft:vex minecraft:vindicator minecraft:warden minecraft:witch
minecraft:wither minecraft:wither_skeleton minecraft:zoglin minecraft:zombie minecraft:zombie_villager minecraft:zombified_piglin
";

pub(super) fn parse_vec3(x: &str, y: &str, z: &str) -> Result<Vec3, CommandError> {
    Ok(Vec3 {
        x: parse_f64(x)?,
        y: parse_f64(y)?,
        z: parse_f64(z)?,
    })
}

pub(super) fn parse_resource_identifier(input: &str) -> Result<String, CommandError> {
    crate::registry::Identifier::parse(input)
        .map(|id| id.to_string())
        .map_err(|_| CommandError::InvalidSyntax)
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
