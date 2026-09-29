//! `execute if|unless <condition>`: `ExecuteCommand.addConditionals`.
//!
//! A condition is parsed from tokens into an [`ExecuteCondition`], evaluated against a command
//! source into a [`ConditionOutcome`], and either filters sources (forking use, `expect`) or is
//! reported through the terminal `.executes(...)` handlers ([`terminal_condition_result`]).

use super::execute_args::*;
use super::execute_condition_support::*;
use super::execute_store::*;
use super::*;
use crate::command_nbt_path_argument::NbtPathModel;
use crate::command_slot_arguments::SlotRangeModel;
use crate::criterion_min_max_bounds::{DoublesBoundsModel, IntsBoundsModel};

/// The comparison of `if score <target> <objective> <op> <source> <sourceObjective>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScoreComparison {
    Equal,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl ScoreComparison {
    fn parse(input: &str) -> Option<Self> {
        Some(match input {
            "=" => Self::Equal,
            "<" => Self::Less,
            "<=" => Self::LessOrEqual,
            ">" => Self::Greater,
            ">=" => Self::GreaterOrEqual,
            _ => return None,
        })
    }

    fn test(self, a: i32, b: i32) -> bool {
        match self {
            Self::Equal => a == b,
            Self::Less => a < b,
            Self::LessOrEqual => a <= b,
            Self::Greater => a > b,
            Self::GreaterOrEqual => a >= b,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ScoreTest {
    Compare {
        comparison: ScoreComparison,
        source: String,
        source_objective: String,
    },
    Matches(IntsBoundsModel),
}

/// One parsed `if|unless` condition (without the expected polarity).
#[derive(Debug, Clone, PartialEq)]
pub(super) enum ExecuteCondition {
    Block {
        pos: Coordinates,
        predicate: BlockPredicate,
    },
    Biome {
        pos: Coordinates,
        predicate: BiomePredicate,
    },
    Loaded {
        pos: Coordinates,
    },
    Dimension(String),
    Score {
        target: String,
        target_objective: String,
        test: ScoreTest,
    },
    Blocks {
        start: Coordinates,
        end: Coordinates,
        destination: Coordinates,
        skip_air: bool,
    },
    Entity(String),
    Predicate(String),
    /// `if function <name>`: a custom modifier (see `ExecuteCommand.scheduleFunctionConditionsAndTest`).
    Function(String),
    ItemsEntity {
        entities: String,
        slots: SlotRangeModel,
        predicate: ItemPredicate,
    },
    ItemsBlock {
        pos: Coordinates,
        slots: SlotRangeModel,
        predicate: ItemPredicate,
    },
    Data {
        target: DataTargetSpec,
        path: String,
    },
    Stopwatch {
        id: String,
        range: DoublesBoundsModel,
    },
}

/// What a condition computed, which decides both forking and the terminal feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConditionOutcome {
    /// A `CommandPredicate` (`addConditional`).
    Bool(bool),
    /// A `CommandNumericPredicate` (`entity`, `items`, `data`).
    Count(i32),
    /// `checkRegions`: the compared block count, if the regions matched.
    Regions(Option<i32>),
}

impl ConditionOutcome {
    /// The `result` passed to `ExecuteCommand.expect(context, expected, result)`.
    pub(super) fn truthy(self) -> bool {
        match self {
            Self::Bool(value) => value,
            Self::Count(count) => count > 0,
            Self::Regions(count) => count.is_some(),
        }
    }
}

/// `BlockPosArgument.blockPos()` from three consecutive tokens starting at `start`.
fn block_pos_at(tokens: &[&str], start: usize) -> Result<Coordinates, CommandError> {
    match tokens.get(start..start + 3) {
        Some([x, y, z]) => Coordinates::parse_block_pos([x, y, z]),
        _ => Err(CommandError::InvalidSyntax),
    }
}

fn token(tokens: &[&str], index: usize) -> Result<String, CommandError> {
    tokens
        .get(index)
        .map(|token| (*token).to_string())
        .ok_or(CommandError::InvalidSyntax)
}

impl ExecuteCondition {
    /// Parses the tokens following `if`/`unless`; returns the condition and the tokens used
    /// (including the condition keyword).
    pub(super) fn parse(
        state: &ServerCommandState,
        tokens: &[&str],
    ) -> Result<(Self, usize), CommandError> {
        let keyword = *tokens.first().ok_or(CommandError::InvalidSyntax)?;
        match keyword {
            "block" => Ok((
                Self::Block {
                    pos: block_pos_at(tokens, 1)?,
                    predicate: BlockPredicate::parse(
                        tokens.get(4).ok_or(CommandError::InvalidSyntax)?,
                    )?,
                },
                5,
            )),
            "biome" => Ok((
                Self::Biome {
                    pos: block_pos_at(tokens, 1)?,
                    predicate: BiomePredicate::parse(
                        tokens.get(4).ok_or(CommandError::InvalidSyntax)?,
                    )?,
                },
                5,
            )),
            "loaded" => Ok((
                Self::Loaded {
                    pos: block_pos_at(tokens, 1)?,
                },
                4,
            )),
            "dimension" => Ok((
                Self::Dimension(parse_resource_identifier(
                    tokens.get(1).ok_or(CommandError::InvalidSyntax)?,
                )?),
                2,
            )),
            "score" => Self::parse_score(tokens),
            "blocks" => {
                let start = block_pos_at(tokens, 1)?;
                let end = block_pos_at(tokens, 4)?;
                let destination = block_pos_at(tokens, 7)?;
                let skip_air = match tokens.get(10).copied() {
                    Some("all") => false,
                    Some("masked") => true,
                    _ => return Err(CommandError::InvalidSyntax),
                };
                Ok((
                    Self::Blocks {
                        start,
                        end,
                        destination,
                        skip_air,
                    },
                    11,
                ))
            }
            "entity" => {
                let selector = tokens.get(1).ok_or(CommandError::InvalidSyntax)?;
                validate_entity_selector(selector)?;
                Ok((Self::Entity((*selector).to_string()), 2))
            }
            "predicate" => {
                let id = normalize_predicate_id(tokens.get(1).ok_or(CommandError::InvalidSyntax)?)?;
                if !state.loot_predicates.iter().any(|entry| entry.id == id) {
                    return Err(translatable("argument.predicate.unknown", [id]));
                }
                Ok((Self::Predicate(id), 2))
            }
            "function" => {
                let name = tokens.get(1).ok_or(CommandError::InvalidSyntax)?;
                parse_schedule_function(name)?;
                Ok((Self::Function((*name).to_string()), 2))
            }
            "items" => Self::parse_items(tokens),
            "data" => Self::parse_data(tokens),
            "stopwatch" => {
                let id = parse_identifier(tokens.get(1).ok_or(CommandError::InvalidSyntax)?)?;
                let range = DoublesBoundsModel::from_reader(
                    tokens.get(2).ok_or(CommandError::InvalidSyntax)?,
                )
                .map_err(|_| CommandError::InvalidSyntax)?;
                Ok((Self::Stopwatch { id, range }, 3))
            }
            _ => Err(CommandError::InvalidSyntax),
        }
    }

    fn parse_score(tokens: &[&str]) -> Result<(Self, usize), CommandError> {
        let target = token(tokens, 1)?;
        let target_objective = token(tokens, 2)?;
        let operator = *tokens.get(3).ok_or(CommandError::InvalidSyntax)?;
        if operator == "matches" {
            let range = IntsBoundsModel::from_reader(tokens.get(4).ok_or(CommandError::InvalidSyntax)?)
                .map_err(|_| CommandError::InvalidSyntax)?;
            return Ok((
                Self::Score {
                    target,
                    target_objective,
                    test: ScoreTest::Matches(range),
                },
                5,
            ));
        }
        let comparison = ScoreComparison::parse(operator).ok_or(CommandError::InvalidSyntax)?;
        Ok((
            Self::Score {
                target,
                target_objective,
                test: ScoreTest::Compare {
                    comparison,
                    source: token(tokens, 4)?,
                    source_objective: token(tokens, 5)?,
                },
            },
            6,
        ))
    }

    fn parse_items(tokens: &[&str]) -> Result<(Self, usize), CommandError> {
        match tokens.get(1).copied() {
            Some("entity") => {
                let selector = token(tokens, 2)?;
                validate_entity_selector(&selector)?;
                Ok((
                    Self::ItemsEntity {
                        entities: selector,
                        slots: parse_slot_range(tokens.get(3).ok_or(CommandError::InvalidSyntax)?)?,
                        predicate: ItemPredicate::parse(
                            tokens.get(4).ok_or(CommandError::InvalidSyntax)?,
                        )?,
                    },
                    5,
                ))
            }
            Some("block") => Ok((
                Self::ItemsBlock {
                    pos: block_pos_at(tokens, 2)?,
                    slots: parse_slot_range(tokens.get(5).ok_or(CommandError::InvalidSyntax)?)?,
                    predicate: ItemPredicate::parse(
                        tokens.get(6).ok_or(CommandError::InvalidSyntax)?,
                    )?,
                },
                7,
            )),
            _ => Err(CommandError::InvalidSyntax),
        }
    }

    fn parse_data(tokens: &[&str]) -> Result<(Self, usize), CommandError> {
        let (target, path_index) = match tokens.get(1).copied() {
            Some("entity") => {
                let selector = token(tokens, 2)?;
                validate_entity_selector(&selector)?;
                (DataTargetSpec::Entity(selector), 3)
            }
            Some("block") => (DataTargetSpec::Block(block_pos_at(tokens, 2)?), 5),
            Some("storage") => (
                DataTargetSpec::Storage(parse_resource_identifier(
                    tokens.get(2).ok_or(CommandError::InvalidSyntax)?,
                )?),
                3,
            ),
            _ => return Err(CommandError::InvalidSyntax),
        };
        let path = token(tokens, path_index)?;
        NbtPathModel::of(&path).map_err(|_| CommandError::InvalidSyntax)?;
        Ok((Self::Data { target, path }, path_index + 1))
    }

    /// Evaluates the condition for one source. `Function` conditions are scheduled by the
    /// chain runner instead and never reach this method.
    pub(super) fn evaluate(
        &self,
        state: &ServerCommandState,
        source: &ExecuteSourceSnapshot,
    ) -> Result<ConditionOutcome, CommandError> {
        Ok(match self {
            Self::Block { pos, predicate } => {
                let pos = loaded_block_pos(state, source, pos)?;
                ConditionOutcome::Bool(predicate.test(state, &source.dimension, pos))
            }
            Self::Biome { pos, predicate } => {
                let pos = loaded_block_pos(state, source, pos)?;
                let biome = biome_at_explicit_or_generated(
                    state,
                    &source.dimension,
                    quantize_biome_pos(pos),
                );
                ConditionOutcome::Bool(predicate.test(&biome))
            }
            Self::Loaded { pos } => ConditionOutcome::Bool(chunk_is_loaded(
                state,
                &source.dimension,
                pos.block_pos(state, source),
                true,
            )),
            Self::Dimension(dimension) => {
                require_known_dimension(state, dimension)?;
                ConditionOutcome::Bool(*dimension == source.dimension)
            }
            Self::Score {
                target,
                target_objective,
                test,
            } => ConditionOutcome::Bool(evaluate_score(
                state,
                source,
                target,
                target_objective,
                test,
            )?),
            Self::Blocks {
                start,
                end,
                destination,
                skip_air,
            } => ConditionOutcome::Regions(check_regions(
                state,
                &source.dimension,
                loaded_block_pos(state, source, start)?,
                loaded_block_pos(state, source, end)?,
                loaded_block_pos(state, source, destination)?,
                *skip_air,
            )?),
            Self::Entity(selector) => {
                ConditionOutcome::Count(resolve_entities(state, source, selector)?.len() as i32)
            }
            Self::Predicate(id) => ConditionOutcome::Bool(evaluate_predicate(state, source, id)?),
            Self::Function(_) => return Err(CommandError::InvalidSyntax),
            Self::ItemsEntity {
                entities,
                slots,
                predicate,
            } => ConditionOutcome::Count(count_entity_items(
                state,
                &resolve_entities_required(state, source, entities)?,
                slots,
                predicate,
            )),
            Self::ItemsBlock {
                pos,
                slots,
                predicate,
            } => {
                let pos = loaded_block_pos(state, source, pos)?;
                ConditionOutcome::Count(count_block_items(state, pos, slots, predicate)?)
            }
            Self::Data { target, path } => {
                let accessor = bind_data_target(state, source, target)?;
                let path = NbtPathModel::of(path).map_err(|_| CommandError::InvalidSyntax)?;
                ConditionOutcome::Count(path.count_matching(&accessor.get_data(state)?) as i32)
            }
            Self::Stopwatch { id, range } => {
                ConditionOutcome::Bool(evaluate_stopwatch(state, id, range)?)
            }
        })
    }
}

/// `BlockPosArgument.getLoadedBlockPos` for the source's level.
fn loaded_block_pos(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    coordinates: &Coordinates,
) -> Result<BlockPos, CommandError> {
    require_loaded_block_pos(state, &source.dimension, coordinates.block_pos(state, source))
}

/// `ExecuteCommand.checkCustomPredicate` for a registered predicate id.
fn evaluate_predicate(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    id: &str,
) -> Result<bool, CommandError> {
    let predicate = state
        .loot_predicates
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| translatable("argument.predicate.unknown", [id.to_string()]))?;
    Ok(evaluate_loot_predicate(state, source, &predicate.definition))
}

/// `ExecuteCommand.checkStopwatch`.
fn evaluate_stopwatch(
    state: &ServerCommandState,
    id: &str,
    range: &DoublesBoundsModel,
) -> Result<bool, CommandError> {
    let watch = state
        .stopwatches
        .iter()
        .find(|watch| watch.id == id)
        .ok_or(CommandError::StopwatchDoesNotExist)?;
    let elapsed_millis = watch.accumulated_elapsed_millis
        + state
            .command_time_millis
            .saturating_sub(watch.creation_time_millis);
    Ok(range.matches(elapsed_millis as f64 / 1000.0))
}

/// Resolves a predicate id argument (`ResourceOrIdArgument.lootPredicate` by id).
fn normalize_predicate_id(input: &str) -> Result<String, CommandError> {
    parse_resource_identifier(input)
}

/// `DimensionArgument.getDimension`: `argument.dimension.invalid` for a dimension the model
/// does not know (the three vanilla dimensions plus any dimension the state mentions).
pub(super) fn require_known_dimension(
    state: &ServerCommandState,
    dimension: &str,
) -> Result<(), CommandError> {
    let known = matches!(
        dimension,
        "minecraft:overworld" | "minecraft:the_nether" | "minecraft:the_end"
    ) || dimension == state.command_source_dimension
        || state.blocks.iter().any(|entry| entry.dimension == dimension)
        || state.entity_states.iter().any(|entry| entry.dimension == dimension)
        || state
            .entity_positions
            .iter()
            .any(|entry| entry.dimension == dimension);
    if known {
        Ok(())
    } else {
        Err(translatable(
            "argument.dimension.invalid",
            [dimension.to_string()],
        ))
    }
}

/// `ExecuteCommand.checkScore` (both the operator and `matches` forms).
fn evaluate_score(
    state: &ServerCommandState,
    source: &ExecuteSourceSnapshot,
    target: &str,
    target_objective: &str,
    test: &ScoreTest,
) -> Result<bool, CommandError> {
    let target = resolve_single_score_holder(state, source, target)?;
    require_scoreboard_objective(state, target_objective)?;
    match test {
        ScoreTest::Matches(range) => Ok(scoreboard_score(state, &target, target_objective)
            .is_some_and(|score| range.matches(score.value))),
        ScoreTest::Compare {
            comparison,
            source: other,
            source_objective,
        } => {
            let other = resolve_single_score_holder(state, source, other)?;
            require_scoreboard_objective(state, source_objective)?;
            let a = scoreboard_score(state, &target, target_objective);
            let b = scoreboard_score(state, &other, source_objective);
            Ok(match (a, b) {
                (Some(a), Some(b)) => comparison.test(a.value, b.value),
                _ => false,
            })
        }
    }
}

/// The terminal `.executes(...)` handler of a condition: `createNumericConditionalHandler`,
/// `addConditional`'s handler and `checkIfRegions`/`checkUnlessRegions`.
pub(super) fn terminal_condition_result(
    state: &mut ServerCommandState,
    expected: bool,
    outcome: ConditionOutcome,
) -> Result<CommandResult, CommandError> {
    let pass = |state: &mut ServerCommandState, count: Option<i32>, value: i32| {
        state.feedback_args = count.map(|count| vec![count.to_string()]).unwrap_or_default();
        Ok(CommandResult {
            success_count: value,
            feedback_key: if count.is_some() {
                "commands.execute.conditional.pass_count"
            } else {
                "commands.execute.conditional.pass"
            },
            broadcast_to_admins: false,
        })
    };
    let fail = || CommandError::ExecuteConditionFailed;
    let fail_count =
        |count: i32| translatable("commands.execute.conditional.fail_count", [count.to_string()]);
    match (outcome, expected) {
        (ConditionOutcome::Bool(value), _) => {
            if value == expected {
                pass(state, None, 1)
            } else {
                Err(fail())
            }
        }
        (ConditionOutcome::Count(count), true) | (ConditionOutcome::Regions(Some(count)), true) => {
            if count > 0 || matches!(outcome, ConditionOutcome::Regions(_)) {
                pass(state, Some(count), count)
            } else {
                Err(fail())
            }
        }
        (ConditionOutcome::Regions(None), true) => Err(fail()),
        (ConditionOutcome::Count(count), false) => {
            if count == 0 {
                pass(state, None, 1)
            } else {
                Err(fail_count(count))
            }
        }
        (ConditionOutcome::Regions(Some(count)), false) => Err(fail_count(count)),
        (ConditionOutcome::Regions(None), false) => pass(state, None, 1),
    }
}
