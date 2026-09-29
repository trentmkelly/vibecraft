//! `execute store (result|success) ...`: the result callbacks `ExecuteCommand.wrapStores`
//! attaches to a command source, and the NBT accessors (`DataAccessor`) they and
//! `execute if data` share.

use super::execute_args::*;
use super::*;
use crate::command_nbt_path_argument::NbtPathModel;

/// The numeric NBT type a `store ... <path> <type> <scale>` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StoreNumberKind {
    Int,
    Float,
    Short,
    Long,
    Double,
    Byte,
}

impl StoreNumberKind {
    pub(super) fn parse(input: &str) -> Option<Self> {
        Some(match input {
            "int" => Self::Int,
            "float" => Self::Float,
            "short" => Self::Short,
            "long" => Self::Long,
            "double" => Self::Double,
            "byte" => Self::Byte,
            _ => return None,
        })
    }

    /// The `IntFunction<Tag>` of `wrapStores`: `v -> XTag.valueOf((x) (v * scale))`.
    ///
    /// Java's `(short)`/`(byte)` casts of a `double` go through `int` first, so they truncate
    /// rather than saturate.
    fn tag(self, value: i32, scale: f64) -> Tag {
        let scaled = f64::from(value) * scale;
        match self {
            Self::Int => Tag::Int(scaled as i32),
            Self::Float => Tag::Float(scaled as f32),
            Self::Short => Tag::Short(scaled as i32 as i16),
            Self::Long => Tag::Long(scaled as i64),
            Self::Double => Tag::Double(scaled),
            Self::Byte => Tag::Byte(scaled as i32 as i8),
        }
    }
}

/// `DataCommands.DataProvider` of a `store` target, still unresolved (selector/coordinates).
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DataTargetSpec {
    Entity(String),
    Block(Coordinates),
    Storage(String),
}

/// A `DataAccessor`: the NBT compound a `/data`-style target exposes.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DataTarget {
    Entity(EntityRef),
    Block(BlockPos),
    Storage(String),
}

/// The parsed `store` sub-tree, before it is bound to a command source.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct StoreSpec {
    pub(super) store_result: bool,
    pub(super) target: StoreTargetSpec,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum StoreTargetSpec {
    Score {
        targets: String,
        objective: String,
    },
    BossBar {
        id: String,
        /// `false` stores into `value`, `true` into `max`.
        max: bool,
    },
    Data {
        target: DataTargetSpec,
        path: String,
        kind: StoreNumberKind,
        scale: f64,
    },
}

/// One `CommandResultCallback` added by `execute store`, bound to resolved targets.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct StoreCallback {
    store_result: bool,
    target: BoundStoreTarget,
}

#[derive(Debug, Clone, PartialEq)]
enum BoundStoreTarget {
    Score {
        holders: Vec<String>,
        objective: String,
    },
    BossBar {
        id: String,
        max: bool,
    },
    Data {
        target: DataTarget,
        path: String,
        kind: StoreNumberKind,
        scale: f64,
    },
}

impl StoreSpec {
    /// Parses the tokens after `store` (`result|success ...`). Returns the spec and the number of
    /// tokens consumed, including the `result|success` keyword.
    pub(super) fn parse(tokens: &[&str]) -> Result<(Self, usize), CommandError> {
        let store_result = match tokens.first().copied() {
            Some("result") => true,
            Some("success") => false,
            _ => return Err(CommandError::InvalidSyntax),
        };
        let (target, consumed) = match tokens.get(1).copied() {
            Some("score") => match tokens.get(2..4) {
                Some([targets, objective]) => (
                    StoreTargetSpec::Score {
                        targets: (*targets).to_string(),
                        objective: (*objective).to_string(),
                    },
                    4,
                ),
                _ => return Err(CommandError::InvalidSyntax),
            },
            Some("bossbar") => match (tokens.get(2), tokens.get(3).copied()) {
                (Some(id), Some(which @ ("value" | "max"))) => (
                    StoreTargetSpec::BossBar {
                        id: parse_resource_identifier(id)?,
                        max: which == "max",
                    },
                    4,
                ),
                _ => return Err(CommandError::InvalidSyntax),
            },
            Some(provider @ ("entity" | "block" | "storage")) => {
                let (target, used) = match provider {
                    "entity" => {
                        let selector = tokens.get(2).ok_or(CommandError::InvalidSyntax)?;
                        validate_entity_selector(selector)?;
                        (DataTargetSpec::Entity((*selector).to_string()), 3)
                    }
                    "block" => {
                        let Some([x, y, z]) = tokens.get(2..5) else {
                            return Err(CommandError::InvalidSyntax);
                        };
                        (
                            DataTargetSpec::Block(Coordinates::parse_block_pos([x, y, z])?),
                            5,
                        )
                    }
                    _ => {
                        let id = tokens.get(2).ok_or(CommandError::InvalidSyntax)?;
                        (DataTargetSpec::Storage(parse_resource_identifier(id)?), 3)
                    }
                };
                let Some([path, kind, scale]) = tokens.get(used..used + 3) else {
                    return Err(CommandError::InvalidSyntax);
                };
                NbtPathModel::of(path).map_err(|_| CommandError::InvalidSyntax)?;
                let kind = StoreNumberKind::parse(kind).ok_or(CommandError::InvalidSyntax)?;
                let scale = parse_java_double_argument(scale)?;
                (
                    StoreTargetSpec::Data {
                        target,
                        path: (*path).to_string(),
                        kind,
                        scale,
                    },
                    used + 3,
                )
            }
            _ => return Err(CommandError::InvalidSyntax),
        };
        Ok((
            Self {
                store_result,
                target,
            },
            consumed,
        ))
    }

    /// The `storeValue`/`storeData` modifier body: resolves the targets against `source` and
    /// returns the callback `CommandSourceStack.withCallback(cb, CommandResultCallback::chain)`
    /// appends.
    pub(super) fn bind(
        &self,
        state: &ServerCommandState,
        source: &ExecuteSourceSnapshot,
    ) -> Result<StoreCallback, CommandError> {
        let target = match &self.target {
            StoreTargetSpec::Score { targets, objective } => {
                require_scoreboard_objective(state, objective)?;
                BoundStoreTarget::Score {
                    holders: resolve_score_holders_with_wildcard(state, source, targets)?,
                    objective: objective.clone(),
                }
            }
            StoreTargetSpec::BossBar { id, max } => {
                bossbar(state, id)?;
                BoundStoreTarget::BossBar {
                    id: id.clone(),
                    max: *max,
                }
            }
            StoreTargetSpec::Data {
                target,
                path,
                kind,
                scale,
            } => BoundStoreTarget::Data {
                target: bind_data_target(state, source, target)?,
                path: path.clone(),
                kind: *kind,
                scale: *scale,
            },
        };
        Ok(StoreCallback {
            store_result: self.store_result,
            target,
        })
    }
}

/// Brigadier `DoubleArgumentType.doubleArg()` token (`StringReader.readDouble`).
pub(super) fn parse_java_double_argument(token: &str) -> Result<f64, CommandError> {
    if token.is_empty()
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.' || byte == b'-')
    {
        return Err(CommandError::InvalidSyntax);
    }
    token.parse::<f64>().map_err(|_| CommandError::InvalidSyntax)
}

/// `DataProvider.access(context)` for a bound source.
pub(super) fn bind_data_target(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    spec: &DataTargetSpec,
) -> Result<DataTarget, CommandError> {
    Ok(match spec {
        DataTargetSpec::Entity(selector) => {
            DataTarget::Entity(resolve_single_entity(state, source, selector)?)
        }
        DataTargetSpec::Block(coordinates) => {
            let pos = coordinates.block_pos(state, source);
            if !block_entity_exists(state, pos) {
                return Err(translatable("commands.data.block.invalid", []));
            }
            DataTarget::Block(pos)
        }
        DataTargetSpec::Storage(id) => DataTarget::Storage(id.clone()),
    })
}

/// Whether the command model holds a block entity (its NBT source) at `pos`.
pub(super) fn block_entity_exists(state: &ServerCommandState, pos: BlockPos) -> bool {
    state
        .macro_block_nbt_sources
        .iter()
        .any(|source| source.pos == pos)
}

impl DataTarget {
    /// `DataAccessor.getData()`.
    pub(super) fn get_data(&self, state: &ServerCommandState) -> Result<Tag, CommandError> {
        let empty = || Tag::Compound(Vec::new());
        Ok(match self {
            Self::Entity(entity) => state
                .macro_entity_nbt_sources
                .iter()
                .find(|source| source.entity.id == entity.id)
                .map_or_else(empty, |source| source.nbt.clone()),
            Self::Block(pos) => state
                .macro_block_nbt_sources
                .iter()
                .find(|source| source.pos == *pos)
                .map(|source| source.nbt.clone())
                .ok_or_else(|| translatable("commands.data.block.invalid", []))?,
            Self::Storage(id) => state
                .macro_storage_nbt_sources
                .iter()
                .find(|source| source.id == *id)
                .map_or_else(empty, |source| source.nbt.clone()),
        })
    }

    /// `DataAccessor.setData(tag)`; players cannot be modified (`commands.data.entity.invalid`).
    pub(super) fn set_data(
        &self,
        state: &mut ServerCommandState,
        data: Tag,
    ) -> Result<(), CommandError> {
        match self {
            Self::Entity(entity) => {
                if entity_state(state, entity).map(|entry| entry.kind) == Some(EntityKind::Player)
                    || state
                        .online_players
                        .iter()
                        .any(|player| player.name == entity.id)
                {
                    return Err(translatable("commands.data.entity.invalid", []));
                }
                match state
                    .macro_entity_nbt_sources
                    .iter_mut()
                    .find(|source| source.entity.id == entity.id)
                {
                    Some(source) => source.nbt = data,
                    None => state.macro_entity_nbt_sources.push(CommandEntityNbtSource {
                        entity: entity.clone(),
                        nbt: data,
                    }),
                }
            }
            Self::Block(pos) => {
                let source = state
                    .macro_block_nbt_sources
                    .iter_mut()
                    .find(|source| source.pos == *pos)
                    .ok_or_else(|| translatable("commands.data.block.invalid", []))?;
                source.nbt = data;
            }
            Self::Storage(id) => match state
                .macro_storage_nbt_sources
                .iter_mut()
                .find(|source| source.id == *id)
            {
                Some(source) => source.nbt = data,
                None => state.macro_storage_nbt_sources.push(CommandStorageNbtSource {
                    id: id.clone(),
                    nbt: data,
                }),
            },
        }
        Ok(())
    }
}

/// `ScoreHolderArgument.getNamesWithDefaultWildcard`: `*` expands to every tracked holder,
/// selectors to the matching entities' scoreboard names, anything else to literal names.
pub(super) fn resolve_score_holders_with_wildcard(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    input: &str,
) -> Result<Vec<String>, CommandError> {
    if input == "*" {
        return Ok(tracked_score_holders(state));
    }
    if input.starts_with('@') {
        return Ok(resolve_entities(state, source, input)?
            .iter()
            .map(|entity| score_holder_name(state, entity))
            .collect());
    }
    Ok(parse_score_holders(input))
}

/// `ScoreHolderArgument.getName`: the first holder, `argument.scoreHolder.empty` if none.
pub(super) fn resolve_single_score_holder(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    input: &str,
) -> Result<String, CommandError> {
    let holders = resolve_score_holders_with_wildcard(state, source, input)?;
    holders
        .into_iter()
        .next()
        .ok_or_else(|| translatable("argument.scoreHolder.empty", []))
}

impl StoreCallback {
    /// Runs `CommandResultCallback.onResult(success, result)` for this store.
    ///
    /// Data-store failures are swallowed exactly like the `catch (CommandSyntaxException)` in
    /// `storeData`; the only error returned is Java's uncaught `IllegalStateException` for
    /// writing a read-only score, which aborts the whole command as `command.failed`.
    pub(super) fn on_result(
        &self,
        state: &mut ServerCommandState,
        success: bool,
        result: i32,
    ) -> Result<(), CommandError> {
        let value = if self.store_result {
            result
        } else {
            i32::from(success)
        };
        match &self.target {
            BoundStoreTarget::Score { holders, objective } => {
                for holder in holders {
                    if require_writable_objective(state, objective).is_err() {
                        return Err(translatable("command.failed", []));
                    }
                    scoreboard_score_mut_or_create(state, holder, objective).value = value;
                }
            }
            BoundStoreTarget::BossBar { id, max } => {
                if let Ok(bar) = bossbar_mut(state, id) {
                    if *max {
                        bar.max = value;
                    } else {
                        bar.value = value;
                    }
                }
            }
            BoundStoreTarget::Data {
                target,
                path,
                kind,
                scale,
            } => {
                let Ok(mut data) = target.get_data(state) else {
                    return Ok(());
                };
                let Ok(path) = NbtPathModel::of(path) else {
                    return Ok(());
                };
                if path.set(&mut data, kind.tag(value, *scale)).is_ok() {
                    let _ = target.set_data(state, data);
                }
            }
        }
        Ok(())
    }
}

/// `ScoreHolder.getScoreboardName()` of an entity: a player's name, otherwise its UUID/id.
pub(super) fn score_holder_name(state: &ServerCommandState, entity: &EntityRef) -> String {
    state
        .online_players
        .iter()
        .find(|player| player.name == entity.id)
        .map_or_else(|| entity.id.clone(), |player| player.name.clone())
}
