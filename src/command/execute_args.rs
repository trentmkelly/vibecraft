//! Argument helpers shared by the `/execute` implementation: coordinates, entity selectors and
//! the entity queries (`position`, rotation, anchors) that `CommandSourceStack` performs.

use super::execute_math::*;
use super::*;
use crate::command_selector::{
    EntityRecord, EntityWithPosition, Selector, Vec3 as SelectorVec3,
};
use std::collections::BTreeMap;

/// One `WorldCoordinate`: an absolute value or an offset from the source (`~`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WorldCoordinate {
    relative: bool,
    value: f64,
}

impl WorldCoordinate {
    /// `WorldCoordinate.get(double original)`.
    fn get(self, original: f64) -> f64 {
        if self.relative {
            self.value + original
        } else {
            self.value
        }
    }

    /// `WorldCoordinate.parseDouble(reader, center)` for one whitespace-delimited token.
    fn parse_double(token: &str, center: bool) -> Result<Self, CommandError> {
        if token.starts_with('^') {
            // `Vec3Argument.ERROR_MIXED_TYPE`
            return Err(CommandError::InvalidSyntax);
        }
        let (relative, number) = match token.strip_prefix('~') {
            Some(rest) => (true, rest),
            None => (false, token),
        };
        if number.is_empty() {
            // Relative `~` is a zero offset; an empty absolute token cannot occur.
            return if relative {
                Ok(Self {
                    relative: true,
                    value: 0.0,
                })
            } else {
                Err(CommandError::InvalidSyntax)
            };
        }
        let mut value = read_java_double(number)?;
        if !number.contains('.') && !relative && center {
            value += 0.5;
        }
        Ok(Self { relative, value })
    }

    /// `WorldCoordinate.parseInt(reader)`.
    fn parse_int(token: &str) -> Result<Self, CommandError> {
        if token.starts_with('^') {
            return Err(CommandError::InvalidSyntax);
        }
        let (relative, number) = match token.strip_prefix('~') {
            Some(rest) => (true, rest),
            None => (false, token),
        };
        let value = if number.is_empty() {
            0.0
        } else if relative {
            read_java_double(number)?
        } else {
            f64::from(number.parse::<i32>().map_err(|_| CommandError::InvalidSyntax)?)
        };
        Ok(Self { relative, value })
    }
}

/// Brigadier `StringReader.readDouble`: only `0-9`, `.` and `-` are number characters.
fn read_java_double(token: &str) -> Result<f64, CommandError> {
    if token.is_empty()
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.' || byte == b'-')
    {
        return Err(CommandError::InvalidSyntax);
    }
    token.parse::<f64>().map_err(|_| CommandError::InvalidSyntax)
}

/// The `Coordinates` implementations: `WorldCoordinates` (`x y z` / `~ ~ ~`) or
/// `LocalCoordinates` (`^left ^up ^forwards`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Coordinates {
    World([WorldCoordinate; 3]),
    Local { left: f64, up: f64, forwards: f64 },
}

impl Coordinates {
    /// `Vec3Argument.parse` with `centerCorrect == true` (`Vec3Argument.vec3()`).
    pub(super) fn parse_vec3(tokens: [&str; 3]) -> Result<Self, CommandError> {
        if tokens[0].starts_with('^') {
            let read = |token: &str| -> Result<f64, CommandError> {
                let rest = token.strip_prefix('^').ok_or(CommandError::InvalidSyntax)?;
                if rest.is_empty() {
                    Ok(0.0)
                } else {
                    read_java_double(rest)
                }
            };
            return Ok(Self::Local {
                left: read(tokens[0])?,
                up: read(tokens[1])?,
                forwards: read(tokens[2])?,
            });
        }
        Ok(Self::World([
            WorldCoordinate::parse_double(tokens[0], true)?,
            WorldCoordinate::parse_double(tokens[1], false)?,
            WorldCoordinate::parse_double(tokens[2], true)?,
        ]))
    }

    /// `BlockPosArgument.blockPos()` (`WorldCoordinates.parseInt` per axis, or local).
    pub(super) fn parse_block_pos(tokens: [&str; 3]) -> Result<Self, CommandError> {
        if tokens[0].starts_with('^') {
            return Self::parse_vec3(tokens);
        }
        Ok(Self::World([
            WorldCoordinate::parse_int(tokens[0])?,
            WorldCoordinate::parse_int(tokens[1])?,
            WorldCoordinate::parse_int(tokens[2])?,
        ]))
    }

    /// `Coordinates.getPosition(source)`.
    pub(super) fn position(
        self,
        state: &ServerCommandState,
        source: &ExecuteSourceSnapshot,
    ) -> Vec3 {
        match self {
            Self::World([x, y, z]) => Vec3 {
                x: x.get(source.position.x),
                y: y.get(source.position.y),
                z: z.get(source.position.z),
            },
            Self::Local {
                left,
                up,
                forwards,
            } => {
                let origin = source_anchor_position(state, source);
                let offset =
                    apply_local_coordinates_to_rotation(source.pitch, source.yaw, left, up, forwards);
                Vec3 {
                    x: offset.x + origin.x,
                    y: offset.y + origin.y,
                    z: offset.z + origin.z,
                }
            }
        }
    }

    /// `Coordinates.getBlockPos(source)` (`BlockPos.containing(getPosition)`).
    pub(super) fn block_pos(
        self,
        state: &ServerCommandState,
        source: &ExecuteSourceSnapshot,
    ) -> BlockPos {
        block_pos_containing(self.position(state, source))
    }
}

/// `RotationArgument.parse`: `<yaw> <pitch>`, each optionally relative. Returns
/// `(pitch, yaw)` coordinates in `Vec2(x = pitch, y = yaw)` order.
pub(super) fn parse_rotation(tokens: [&str; 2]) -> Result<[WorldCoordinate; 2], CommandError> {
    let yaw = WorldCoordinate::parse_double(tokens[0], false)?;
    let pitch = WorldCoordinate::parse_double(tokens[1], false)?;
    Ok([pitch, yaw])
}

/// `WorldCoordinates.getRotation(source)` -> `(x_rot, y_rot)`.
pub(super) fn resolve_rotation(
    coordinates: [WorldCoordinate; 2],
    source: &ExecuteSourceSnapshot,
) -> (f32, f32) {
    (
        coordinates[0].get(f64::from(source.pitch)) as f32,
        coordinates[1].get(f64::from(source.yaw)) as f32,
    )
}

/// `CommandSourceStack.getEntity().getEyeHeight()` stand-in for the entity model, which only
/// distinguishes players, non-living entities and everything else.
pub(super) fn entity_eye_height(state: &ServerCommandState, entity: &EntityRef) -> f64 {
    match entity_state(state, entity).map(|state| state.kind) {
        Some(EntityKind::Player) => 1.62,
        Some(EntityKind::NonLiving) => 0.0,
        _ => 1.0,
    }
}

/// `EntityAnchorArgument.Anchor.apply(CommandSourceStack)`.
pub(super) fn source_anchor_position(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
) -> Vec3 {
    match (&source.entity, source.anchor) {
        (Some(entity), EntityAnchor::Eyes) => Vec3 {
            x: source.position.x,
            y: source.position.y + entity_eye_height(state, entity),
            z: source.position.z,
        },
        _ => source.position,
    }
}

/// `EntityAnchorArgument.Anchor.apply(Entity)`.
pub(super) fn entity_anchor_position(
    state: &ServerCommandState,
    entity: &EntityRef,
    anchor: EntityAnchor,
) -> Vec3 {
    let position = locate_entity(state, entity)
        .map(|(_, position)| position)
        .unwrap_or_default();
    match anchor {
        EntityAnchor::Feet => position,
        EntityAnchor::Eyes => Vec3 {
            x: position.x,
            y: position.y + entity_eye_height(state, entity),
            z: position.z,
        },
    }
}

/// `Entity.level()` + `Entity.position()`: the dimension and position of an entity.
///
/// Positions come from [`EntityPosition`] records, then from spawned entities; an entity that
/// only has an [`EntityState`] reports no position.
pub(super) fn locate_entity(
    state: &ServerCommandState,
    entity: &EntityRef,
) -> Option<(String, Vec3)> {
    if let Some(entry) = entity_position(state, entity) {
        return Some((entry.dimension.clone(), entry.position));
    }
    let spawned = state
        .summoned_entities
        .iter()
        .find(|spawned| spawned.entity.id == entity.id)?;
    let dimension = entity_state(state, entity)
        .map(|entry| entry.dimension.clone())
        .unwrap_or_else(|| state.command_source_dimension.clone());
    Some((dimension, spawned.position))
}

/// `Entity.getRotationVector()` as `(x_rot, y_rot)`.
pub(super) fn entity_rotation(state: &ServerCommandState, entity: &EntityRef) -> (f32, f32) {
    state
        .entity_rotations
        .iter()
        .find(|entry| entry.entity.id == entity.id)
        .map_or((0.0, 0.0), |entry| (entry.x_rot, entry.y_rot))
}

/// `entity.isRemoved()` for the command model (killed entities are removed).
pub(super) fn entity_is_removed(state: &ServerCommandState, entity: &EntityRef) -> bool {
    state
        .killed_entities
        .iter()
        .any(|killed| killed.id == entity.id)
}

/// Every entity the command model knows about, as selector records with positions.
///
/// The model has no entity types for anonymous entities, so a generic entity reports the type
/// `minecraft:<id>` (model entities are conventionally named after their type).
fn selector_universe(state: &ServerCommandState) -> Vec<(EntityRef, EntityWithPosition)> {
    let mut refs: Vec<EntityRef> = Vec::new();
    let mut push = |entity: &EntityRef| {
        if !refs.iter().any(|known| known.id == entity.id) {
            refs.push(entity.clone());
        }
    };
    for player in &state.online_players {
        push(&entity_ref(&player.name));
    }
    for entry in &state.entity_states {
        push(&entry.entity);
    }
    for entry in &state.entity_positions {
        push(&entry.entity);
    }
    for entry in &state.summoned_entities {
        push(&entry.entity);
    }
    for entry in &state.entity_tags {
        push(&entry.entity);
    }
    refs.into_iter()
        .filter(|entity| !entity_is_removed(state, entity))
        .map(|entity| {
            let record = selector_record(state, &entity);
            let position = locate_entity(state, &entity)
                .map(|(_, position)| position)
                .unwrap_or_else(|| state.command_source_position);
            (
                entity,
                EntityWithPosition {
                    entity: record,
                    position: SelectorVec3 {
                        x: position.x,
                        y: position.y,
                        z: position.z,
                    },
                },
            )
        })
        .collect()
}

fn selector_record(state: &ServerCommandState, entity: &EntityRef) -> EntityRecord {
    let online = state
        .online_players
        .iter()
        .find(|player| player.name == entity.id);
    let kind = entity_state(state, entity).map(|entry| entry.kind);
    let player = online.is_some() || kind == Some(EntityKind::Player);
    let spawned = state
        .summoned_entities
        .iter()
        .find(|spawned| spawned.entity.id == entity.id);
    let entity_type = if player {
        "minecraft:player".to_string()
    } else if let Some(spawned) = spawned {
        spawned.entity_type.clone()
    } else if entity.id.contains(':') {
        entity.id.clone()
    } else {
        format!("minecraft:{}", entity.id)
    };
    let (x_rot, y_rot) = entity_rotation(state, entity);
    let dimension = locate_entity(state, entity)
        .map(|(dimension, _)| dimension)
        .or_else(|| entity_state(state, entity).map(|entry| entry.dimension.clone()))
        .unwrap_or_else(|| state.command_source_dimension.clone());
    let name = online.map_or_else(|| entity.display_name.clone(), |player| player.name.clone());
    let scores: BTreeMap<String, i32> = state
        .scoreboard_scores
        .iter()
        .filter(|score| score.owner == entity.id)
        .map(|score| (score.objective.clone(), score.value))
        .collect();
    EntityRecord {
        uuid: online.map_or_else(|| entity.id.clone(), |player| player.uuid.clone()),
        name: name.clone(),
        entity_type,
        player,
        level: dimension,
        team: state
            .player_teams
            .iter()
            .find(|membership| membership.player.name == name)
            .map(|membership| membership.team.clone()),
        tags: state
            .entity_tags
            .iter()
            .filter(|tags| tags.entity.id == entity.id)
            .flat_map(|tags| tags.tags.iter().cloned())
            .collect(),
        scores,
        nbt: BTreeMap::new(),
        predicates: Vec::new(),
        gamemode: state
            .player_game_modes
            .iter()
            .find(|mode| mode.player.name == name)
            .map(|mode| format!("{:?}", mode.gamemode).to_lowercase()),
        experience_level: state
            .player_experience
            .iter()
            .find(|xp| xp.player.name == name)
            .map_or(0, |xp| xp.level),
        x_rotation: f64::from(x_rot),
        y_rotation: f64::from(y_rot),
        advancements: BTreeMap::new(),
    }
}

/// Validates an `EntityArgument.entities()` token at parse time.
pub(super) fn validate_entity_selector(input: &str) -> Result<(), CommandError> {
    for part in split_selector_list(input) {
        Selector::parse(part).map_err(|_| CommandError::InvalidSyntax)?;
    }
    Ok(())
}

/// A comma list of plain names is accepted for the model's multi-target convenience form
/// (`execute as Steve,Alex`); selectors keep their own commas inside `[...]`.
fn split_selector_list(input: &str) -> Vec<&str> {
    if input.starts_with('@') {
        vec![input]
    } else {
        input.split(',').filter(|part| !part.is_empty()).collect()
    }
}

/// `EntityArgument.getOptionalEntities`: the entities a selector resolves to for `source`
/// (possibly none).
pub(super) fn resolve_entities(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    input: &str,
) -> Result<Vec<EntityRef>, CommandError> {
    let universe = selector_universe(state);
    let records: Vec<EntityWithPosition> =
        universe.iter().map(|(_, record)| record.clone()).collect();
    let origin = SelectorVec3 {
        x: source.position.x,
        y: source.position.y,
        z: source.position.z,
    };
    let current = source
        .entity
        .as_ref()
        .map(|entity| selector_record(state, entity).uuid);
    let mut resolved: Vec<EntityRef> = Vec::new();
    for part in split_selector_list(input) {
        let selector = Selector::parse(part).map_err(|_| CommandError::InvalidSyntax)?;
        for record in selector.select(&records, origin, &source.dimension, current.as_deref()) {
            if let Some((entity, _)) = universe
                .iter()
                .find(|(_, candidate)| candidate.entity.uuid == record.uuid)
            {
                if !resolved.iter().any(|known| known.id == entity.id) {
                    resolved.push(entity.clone());
                }
            }
        }
    }
    Ok(resolved)
}

/// `EntityArgument.getEntities`: like [`resolve_entities`] but an empty result is
/// `argument.entity.notfound.entity`.
pub(super) fn resolve_entities_required(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    input: &str,
) -> Result<Vec<EntityRef>, CommandError> {
    let entities = resolve_entities(state, source, input)?;
    if entities.is_empty() {
        return Err(translatable("argument.entity.notfound.entity", []));
    }
    Ok(entities)
}

/// `EntityArgument.getEntity`: exactly one entity, `argument.entity.toomany` otherwise.
pub(super) fn resolve_single_entity(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    input: &str,
) -> Result<EntityRef, CommandError> {
    let mut entities = resolve_entities_required(state, source, input)?;
    if entities.len() > 1 {
        return Err(translatable("argument.entity.toomany", []));
    }
    Ok(entities.remove(0))
}

/// Builds a [`CommandError::Translatable`].
pub(super) fn translatable<const N: usize>(
    key: &'static str,
    args: [String; N],
) -> CommandError {
    CommandError::Translatable {
        key,
        args: args.into(),
    }
}

/// `BlockPosArgument.getLoadedBlockPos`: `argument.pos.unloaded` / `argument.pos.outofworld`.
pub(super) fn require_loaded_block_pos(
    state: &ServerCommandState,
    dimension: &str,
    pos: BlockPos,
) -> Result<BlockPos, CommandError> {
    if !chunk_is_loaded(state, dimension, pos, false) {
        return Err(translatable("argument.pos.unloaded", []));
    }
    if !is_in_spawnable_bounds(pos) {
        return Err(translatable("argument.pos.outofworld", []));
    }
    Ok(pos)
}

/// `Level.hasChunkAt(pos)` (`entity_ticking == false`) or `ExecuteCommand.isChunkLoaded`
/// (`entity_ticking == true`) for the model's chunk residency.
pub(super) fn chunk_is_loaded(
    state: &ServerCommandState,
    dimension: &str,
    pos: BlockPos,
    entity_ticking: bool,
) -> bool {
    let Some(chunks) = &state.loaded_chunks else {
        return true;
    };
    let (chunk_x, chunk_z) = (pos.x >> 4, pos.z >> 4);
    chunks.iter().any(|chunk| {
        chunk.dimension == dimension
            && chunk.chunk_x == chunk_x
            && chunk.chunk_z == chunk_z
            && (!entity_ticking || chunk.entity_ticking)
    })
}
