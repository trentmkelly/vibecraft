//! Execution of a parsed [`ExecuteChain`]: the model of `BuildContexts`, the `ExecuteCommand`
//! task, `ExecutionCommandSource` result callbacks and the `ExecutionContext` cost/fork limits.
//!
//! Java queues tasks on an `ExecutionContext`; the command model runs them depth-first, which is
//! the same order the queue produces (`pushNewCommands` re-inserts new entries at the front).

use super::execute_args::*;
use super::execute_command::*;
use super::execute_condition_support::level_height;
use super::execute_conditions::*;
use super::execute_math::*;
use super::execute_store::*;
use super::*;

/// `max_command_sequence_length` / `max_command_forks` defaults if a rule is not seeded.
const DEFAULT_COMMAND_LIMIT: i32 = 65536;

/// A `CommandSourceStack` while a chain runs: the location/entity snapshot plus the chained
/// `CommandResultCallback`s (`execute store`) that fire when the command completes.
#[derive(Debug, Clone)]
pub(super) struct ExecuteSource {
    pub(super) snapshot: ExecuteSourceSnapshot,
    pub(super) callbacks: Vec<StoreCallback>,
}

impl ExecuteSource {
    /// A copy of this source whose snapshot was edited (`CommandSourceStack.withX`).
    fn map_snapshot(&self, edit: impl FnOnce(&mut ExecuteSourceSnapshot)) -> Self {
        let mut source = self.clone();
        edit(&mut source.snapshot);
        source
    }

    /// `CommandSourceStack.withEntity`.
    fn with_entity(&self, entity: EntityRef) -> Self {
        let mut source = self.clone();
        source.snapshot.entity = Some(entity);
        source
    }

    /// `ExecutionCommandSource.clearCallbacks`.
    fn clear_callbacks(&self) -> Self {
        Self {
            snapshot: self.snapshot.clone(),
            callbacks: Vec::new(),
        }
    }

    /// `CommandSourceStack.facing(Vec3)`.
    fn facing(&self, state: &ServerCommandState, target: &Vec3) -> Self {
        let from = source_anchor_position(state, &self.snapshot);
        let (xd, yd, zd) = (target.x - from.x, target.y - from.y, target.z - from.z);
        let sd = (xd * xd + zd * zd).sqrt();
        // `(float)Math.PI` widened to double, exactly as the Java expression evaluates.
        let pi = f64::from(std::f32::consts::PI);
        let x_rot = mth_wrap_degrees((-(mth_atan2(yd, sd) * 180.0 / pi)) as f32);
        let y_rot = mth_wrap_degrees((mth_atan2(zd, xd) * 180.0 / pi) as f32 - 90.0);
        let mut source = self.clone();
        source.snapshot.pitch = x_rot;
        source.snapshot.yaw = y_rot;
        source
    }
}

/// Mutable outcome accumulated while a chain runs.
#[derive(Debug, Default)]
struct RunOutcome {
    /// Sum of the results of every successful terminal execution.
    total: i32,
    /// The last successful terminal result (its feedback is what the caller reports).
    last_success: Option<CommandResult>,
    /// The error `handleError` reported (non-forked chains only).
    error: Option<CommandError>,
    /// An uncaught exception (Java `Commands.performCommand` reports `command.failed`).
    abort: Option<CommandError>,
}

struct ChainRunner<'a> {
    permissions: LevelBasedPermissionSet,
    original: &'a CommandSourceSnapshot,
    outcome: RunOutcome,
    /// `ChainModifiers.isReturn`: this chain runs under `return run`.
    returning: bool,
    /// The current frame was discarded by a `return`.
    returned: bool,
}

/// Runs a parsed chain against the state's current command source.
pub(super) fn run_execute_chain(
    state: &mut ServerCommandState,
    permissions: LevelBasedPermissionSet,
    chain: &ExecuteChain,
) -> Result<CommandResult, CommandError> {
    let original = capture_command_source(state);
    let top_level = !state.execution.active;
    if top_level {
        let limit = |rule: &str| match game_rule_value(state, rule) {
            Ok(GameRuleValue::Int(value)) => value,
            _ => DEFAULT_COMMAND_LIMIT,
        };
        state.execution = CommandExecutionBudget {
            active: true,
            quota: limit("max_command_sequence_length").max(1),
            fork_limit: limit("max_command_forks"),
        };
    }
    let origin = ExecuteSource {
        snapshot: ExecuteSourceSnapshot {
            entity: state.command_source_entity.clone(),
            position: state.command_source_position,
            yaw: state.command_source_yaw,
            pitch: state.command_source_pitch,
            dimension: state.command_source_dimension.clone(),
            anchor: EntityAnchor::Feet,
        },
        callbacks: Vec::new(),
    };
    let mut runner = ChainRunner::new(permissions, &original, false);
    runner.build_contexts(state, &origin, vec![origin.clone()], chain, 0, false);
    let result = runner.finish();
    restore_command_source(state, original.clone());
    if top_level {
        state.execution = CommandExecutionBudget::default();
    }
    result
}

impl<'a> ChainRunner<'a> {
    fn new(
        permissions: LevelBasedPermissionSet,
        original: &'a CommandSourceSnapshot,
        returning: bool,
    ) -> Self {
        Self {
            permissions,
            original,
            outcome: RunOutcome::default(),
            returning,
            returned: false,
        }
    }

    /// The command result for the caller: the aggregate of every terminal execution.
    ///
    /// Java has no single return value here (each execution reports through the source's
    /// callback and feedback); the model returns the summed success count with the last
    /// success' feedback, an error only when a non-forked stage reported one, and a silent
    /// zero when nothing executed (`execute if ... run ...` that filtered every source).
    fn finish(self) -> Result<CommandResult, CommandError> {
        if let Some(error) = self.outcome.abort.or(self.outcome.error) {
            return Err(error);
        }
        Ok(match self.outcome.last_success {
            Some(last) => CommandResult {
                success_count: self.outcome.total,
                ..last
            },
            None => CommandResult {
                success_count: 0,
                feedback_key: NO_COMMAND_FEEDBACK,
                broadcast_to_admins: false,
            },
        })
    }

    /// `commandQuota <= 0`, an uncaught exception or a discarded frame end the queue.
    fn stopped(&self, state: &ServerCommandState) -> bool {
        state.execution.quota <= 0 || self.outcome.abort.is_some() || self.returned
    }

    /// `CommandSourceStack.handleError`: only non-forked chains report the failure.
    fn handle_error(&mut self, error: CommandError, forked: bool) {
        if !forked {
            self.outcome.error = Some(error);
        }
    }

    /// Fires a source's chained callbacks (`CommandResultCallback.onResult`).
    fn fire_callbacks(
        &mut self,
        state: &mut ServerCommandState,
        source: &ExecuteSource,
        success: bool,
        result: i32,
    ) {
        for callback in &source.callbacks {
            if let Err(error) = callback.on_result(state, success, result) {
                self.outcome.abort = Some(error);
                return;
            }
        }
    }

    /// `BuildContexts.execute`: applies stages `[index..]` to `sources`, then runs the terminal.
    fn build_contexts(
        &mut self,
        state: &mut ServerCommandState,
        original_source: &ExecuteSource,
        mut sources: Vec<ExecuteSource>,
        chain: &ExecuteChain,
        mut index: usize,
        mut forked: bool,
    ) {
        if self.stopped(state) {
            return;
        }
        while index < chain.stages.len() {
            let stage = &chain.stages[index];
            if stage.is_forking() {
                forked = true;
            }
            if let ExecuteStage::Condition {
                expected,
                condition: ExecuteCondition::Function(name),
            } = stage
            {
                self.schedule_function_conditions(
                    state,
                    original_source,
                    sources,
                    (chain, index),
                    forked,
                    (*expected, name),
                );
                return;
            }
            state.execution.quota -= 1; // ExecutionContext.incrementCost
            let fork_limit = state.execution.fork_limit;
            let mut next_sources: Vec<ExecuteSource> = Vec::new();
            for source in &sources {
                match self.apply_modifier(state, stage, source) {
                    Ok(new_sources) => {
                        if (next_sources.len() + new_sources.len()) as i64 >= i64::from(fork_limit) {
                            self.handle_error(
                                translatable("command.forkLimit", [fork_limit.to_string()]),
                                forked,
                            );
                            return;
                        }
                        next_sources.extend(new_sources);
                    }
                    // `ContextChain.runModifier`: a forked chain swallows the failure and drops
                    // the source; otherwise it aborts with the error message.
                    Err(error) if forked => drop(error),
                    Err(error) => {
                        self.handle_error(error, false);
                        return;
                    }
                }
            }
            sources = next_sources;
            index += 1;
        }
        if sources.is_empty() {
            if self.returning {
                self.return_failure(state);
            }
            return;
        }
        self.execute_terminal(state, chain, sources, forked);
    }

    /// `ExecuteCommand` tasks for every source that reached the end of the chain.
    fn execute_terminal(
        &mut self,
        state: &mut ServerCommandState,
        chain: &ExecuteChain,
        mut sources: Vec<ExecuteSource>,
        forked: bool,
    ) {
        // `BuildContexts`: under `return run` only the first source executes, and its result
        // also completes the returning frame.
        if self.returning {
            sources.truncate(1);
        }
        let snapshots: Vec<ExecuteSourceSnapshot> =
            sources.iter().map(|source| source.snapshot.clone()).collect();
        let total_before = self.outcome.total;
        for source in &sources {
            if self.stopped(state) {
                break;
            }
            state.execution.quota -= 1; // ExecutionContext.incrementCost
            match &chain.terminal {
                ExecuteTerminal::Command(command) => {
                    self.run_command(state, command, source, forked)
                }
                ExecuteTerminal::Condition {
                    expected,
                    condition,
                } => self.run_condition(state, *expected, condition, source, forked),
            }
        }
        if let ExecuteTerminal::Command(command) = &chain.terminal {
            let result = self.outcome.total - total_before;
            state.execute_events.push(ExecuteCommandEvent {
                sources: snapshots,
                command: command.clone(),
                result,
                success: result > 0,
            });
        }
    }

    /// Records one finished terminal execution and completes the callbacks
    /// (`ContextChain.runExecutable`).
    fn complete(
        &mut self,
        state: &mut ServerCommandState,
        source: &ExecuteSource,
        result: Result<CommandResult, CommandError>,
        forked: bool,
    ) {
        match result {
            Ok(result) => {
                self.fire_callbacks(state, source, true, result.success_count);
                if self.returning {
                    self.return_value(state, true, result.success_count);
                }
                self.outcome.total = self.outcome.total.saturating_add(result.success_count);
                self.outcome.last_success = Some(result);
            }
            Err(error) => {
                self.fire_callbacks(state, source, false, 0);
                if self.returning {
                    self.return_value(state, false, 0);
                }
                self.handle_error(error, forked);
            }
        }
    }

    /// `frame.returnSuccess` / `frame.returnFailure` for the returning frame.
    fn return_value(&mut self, state: &mut ServerCommandState, success: bool, value: i32) {
        if self.returned {
            return;
        }
        state.return_events.push(if success {
            ReturnCommandEvent::Success {
                value,
                discard_frame: true,
            }
        } else {
            ReturnCommandEvent::Failure {
                discard_frame: true,
            }
        });
        self.returned = true;
    }

    fn return_failure(&mut self, state: &mut ServerCommandState) {
        self.return_value(state, false, 0);
    }

    fn run_condition(
        &mut self,
        state: &mut ServerCommandState,
        expected: bool,
        condition: &ExecuteCondition,
        source: &ExecuteSource,
        forked: bool,
    ) {
        apply_execute_source(state, self.original, &source.snapshot);
        let result = condition
            .evaluate(state, &source.snapshot)
            .and_then(|outcome| terminal_condition_result(state, expected, outcome));
        self.complete(state, source, result, forked);
    }

    /// `run <command>` for one source. `return` is a custom executor in Java (it completes the
    /// callbacks itself and discards the frame), everything else is a plain command.
    fn run_command(
        &mut self,
        state: &mut ServerCommandState,
        command: &str,
        source: &ExecuteSource,
        forked: bool,
    ) {
        let tokens: Vec<&str> = command.split_whitespace().collect();
        if tokens.first() == Some(&"return") {
            self.run_return(state, &tokens, source, forked);
            return;
        }
        apply_execute_source(state, self.original, &source.snapshot);
        let result = execute_builtin_command(state, self.permissions, command);
        self.complete(state, source, result, forked);
    }

    /// `ReturnCommand`: `return <value>`, `return fail` and `return run <command>`.
    fn run_return(
        &mut self,
        state: &mut ServerCommandState,
        tokens: &[&str],
        source: &ExecuteSource,
        forked: bool,
    ) {
        if self.permissions.can_run("return") == CommandAvailability::Hidden {
            self.complete(state, source, Err(CommandError::PermissionDenied), forked);
            return;
        }
        match tokens {
            ["return", "fail"] => {
                self.fire_callbacks(state, source, false, 0);
                state.return_events.push(ReturnCommandEvent::Failure {
                    discard_frame: true,
                });
                self.returned = true;
            }
            ["return", "run", rest @ ..] if !rest.is_empty() => {
                self.run_return_run(state, rest, source, forked);
            }
            ["return", value] => match value.parse::<i32>() {
                Ok(value) => {
                    self.fire_callbacks(state, source, true, value);
                    state.return_events.push(ReturnCommandEvent::Success {
                        value,
                        discard_frame: true,
                    });
                    self.returned = true;
                }
                Err(_) => self.complete(state, source, Err(CommandError::InvalidSyntax), forked),
            },
            _ => self.complete(state, source, Err(CommandError::InvalidSyntax), forked),
        }
    }

    /// `ReturnFromCommandCustomModifier`: discards the current frame, then runs the command
    /// with `ChainModifiers.setReturn()` so its first result becomes the frame's return value.
    fn run_return_run(
        &mut self,
        state: &mut ServerCommandState,
        command: &[&str],
        source: &ExecuteSource,
        forked: bool,
    ) {
        let chain = if command.first() == Some(&"execute") {
            match parse_execute_chain(state, command) {
                Ok(chain) => chain,
                Err(error) => {
                    self.handle_error(error, forked);
                    return;
                }
            }
        } else {
            ExecuteChain {
                stages: Vec::new(),
                terminal: ExecuteTerminal::Command(command.join(" ")),
            }
        };
        let mut nested = ChainRunner::new(self.permissions, self.original, true);
        nested.build_contexts(state, source, vec![source.clone()], &chain, 0, forked);
        self.outcome.total = self.outcome.total.saturating_add(nested.outcome.total);
        if nested.outcome.last_success.is_some() {
            self.outcome.last_success = nested.outcome.last_success.take();
        }
        if let Some(abort) = nested.outcome.abort {
            self.outcome.abort = Some(abort);
        }
        if let Some(error) = nested.outcome.error {
            self.handle_error(error, forked);
        }
        self.returned = true;
    }

    /// `ExecuteCommand.scheduleFunctionConditionsAndTest` for `if|unless function <name>`.
    fn schedule_function_conditions(
        &mut self,
        state: &mut ServerCommandState,
        original_source: &ExecuteSource,
        sources: Vec<ExecuteSource>,
        (chain, index): (&ExecuteChain, usize),
        forked: bool,
        (expected, name): (bool, &str),
    ) {
        let functions = match resolve_condition_functions(state, name) {
            Ok(functions) => functions,
            Err(error) => {
                self.handle_error(error, forked);
                return;
            }
        };
        if functions.is_empty() {
            return;
        }
        // Instantiation failures are reported, but the chain continues with whatever was
        // instantiated before the failure (Java only `handleError`s, it does not return).
        let mut instantiated: Vec<Vec<String>> = Vec::with_capacity(functions.len());
        for function in &functions {
            match instantiate_command_function(function, None) {
                Ok(function) => instantiated.push(function.commands),
                Err(reason) => {
                    self.handle_error(
                        translatable(
                            "commands.execute.function.instantiationFailure",
                            [function.id.clone(), reason],
                        ),
                        forked,
                    );
                    break;
                }
            }
        }
        let mut filtered: Vec<ExecuteSource> = Vec::with_capacity(sources.len());
        for source in sources {
            if state.execution.quota <= 0 || self.outcome.abort.is_some() {
                break;
            }
            let function_source = source.clear_callbacks();
            let result = self.call_functions(state, &instantiated, &function_source);
            let value = result.map_or(0, |(_, value)| value);
            // `check`: `if` passes on a non-zero result, `unless` on zero.
            if (value != 0) == expected {
                filtered.push(source);
            }
        }
        self.build_contexts(state, original_source, filtered, chain, index + 1, forked);
    }

    /// The `IsolatedCall` of a function condition: runs the functions in order until one
    /// returns, otherwise falls through as a failure. Returns the frame's result, if any.
    fn call_functions(
        &mut self,
        state: &mut ServerCommandState,
        functions: &[Vec<String>],
        source: &ExecuteSource,
    ) -> Option<(bool, i32)> {
        for commands in functions {
            state.execution.quota -= 1; // CallFunction.execute -> incrementCost
            for line in commands {
                if state.execution.quota <= 0 {
                    return None;
                }
                let mark = state.return_events.len();
                self.run_function_line(state, line, source);
                if state.return_events.len() > mark {
                    return match state.return_events.last() {
                        Some(ReturnCommandEvent::Success { value, .. }) => Some((true, *value)),
                        _ => Some((false, 0)),
                    };
                }
            }
        }
        None
    }

    /// One function line as its own top-level `BuildContexts.Unbound` chain in the shared
    /// function frame; failures inside functions never propagate to the caller.
    fn run_function_line(&mut self, state: &mut ServerCommandState, line: &str, source: &ExecuteSource) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let chain = if tokens.first() == Some(&"execute") {
            match parse_execute_chain(state, &tokens) {
                Ok(chain) => chain,
                Err(_) => return,
            }
        } else {
            ExecuteChain {
                stages: Vec::new(),
                terminal: ExecuteTerminal::Command(line.to_string()),
            }
        };
        let mut nested = ChainRunner::new(self.permissions, self.original, false);
        nested.build_contexts(state, source, vec![source.clone()], &chain, 0, false);
        if let Some(abort) = nested.outcome.abort {
            self.outcome.abort = Some(abort);
        }
    }

    /// The modifier body of a stage for one source (`RedirectModifier.apply`).
    fn apply_modifier(
        &mut self,
        state: &mut ServerCommandState,
        stage: &ExecuteStage,
        source: &ExecuteSource,
    ) -> Result<Vec<ExecuteSource>, CommandError> {
        let snapshot = &source.snapshot;
        Ok(match stage {
            // The `fork(...)` nodes that expand a selector or relation into several sources.
            ExecuteStage::As(targets) => resolve_entities(state, snapshot, targets)?
                .into_iter()
                .map(|entity| source.with_entity(entity))
                .collect(),
            ExecuteStage::At(targets) => resolve_entities(state, snapshot, targets)?
                .into_iter()
                .map(|entity| self.at_entity(state, source, &entity))
                .collect(),
            ExecuteStage::PositionedAs(targets) => resolve_entities(state, snapshot, targets)?
                .into_iter()
                .map(|entity| {
                    source.map_snapshot(|next| {
                        if let Some((_, position)) = locate_entity(state, &entity) {
                            next.position = position;
                        }
                    })
                })
                .collect(),
            ExecuteStage::RotatedAs(targets) => resolve_entities(state, snapshot, targets)?
                .into_iter()
                .map(|entity| {
                    let (pitch, yaw) = entity_rotation(state, &entity);
                    source.map_snapshot(|next| {
                        next.pitch = pitch;
                        next.yaw = yaw;
                    })
                })
                .collect(),
            ExecuteStage::FacingEntity { targets, anchor } => {
                resolve_entities(state, snapshot, targets)?
                    .into_iter()
                    .map(|entity| {
                        let target = entity_anchor_position(state, &entity, *anchor);
                        source.facing(state, &target)
                    })
                    .collect()
            }
            ExecuteStage::On(relation) => {
                related_entities(state, snapshot.entity.as_ref(), *relation)
                    .into_iter()
                    .map(|entity| source.with_entity(entity))
                    .collect()
            }
            ExecuteStage::Condition {
                expected,
                condition,
            } => {
                if condition.evaluate(state, snapshot)?.truthy() == *expected {
                    vec![source.clone()]
                } else {
                    Vec::new()
                }
            }
            // The `redirect(...)` nodes: exactly one source comes out.
            _ => vec![self.redirect_modifier(state, stage, source)?],
        })
    }

    /// The single-source (`redirect`) modifiers.
    fn redirect_modifier(
        &mut self,
        state: &mut ServerCommandState,
        stage: &ExecuteStage,
        source: &ExecuteSource,
    ) -> Result<ExecuteSource, CommandError> {
        let snapshot = &source.snapshot;
        Ok(match stage {
            ExecuteStage::PositionedPos(position) => {
                let position = position.position(state, snapshot);
                source.map_snapshot(|next| {
                    next.position = position;
                    next.anchor = EntityAnchor::Feet;
                })
            }
            ExecuteStage::PositionedOver(heightmap) => {
                let position = snapshot.position;
                let pos = block_pos_containing(position);
                if !chunk_is_loaded(state, &snapshot.dimension, pos, false) {
                    return Err(translatable("argument.pos.unloaded", []));
                }
                let height = level_height(state, &snapshot.dimension, *heightmap, pos.x, pos.z);
                source.map_snapshot(|next| next.position.y = f64::from(height))
            }
            ExecuteStage::RotatedRot(rotation) => {
                let (pitch, yaw) = resolve_rotation(*rotation, snapshot);
                source.map_snapshot(|next| {
                    next.pitch = pitch;
                    next.yaw = yaw;
                })
            }
            ExecuteStage::FacingPos(position) => {
                source.facing(state, &position.position(state, snapshot))
            }
            ExecuteStage::Align(axes) => source.map_snapshot(|next| {
                let position = &mut next.position;
                if axes.x {
                    position.x = mth_floor(position.x);
                }
                if axes.y {
                    position.y = mth_floor(position.y);
                }
                if axes.z {
                    position.z = mth_floor(position.z);
                }
            }),
            ExecuteStage::Anchored(anchor) => source.map_snapshot(|next| next.anchor = *anchor),
            ExecuteStage::In(dimension) => {
                require_known_dimension(state, dimension)?;
                with_level(source, dimension)
            }
            ExecuteStage::Summon(entity_type) => self.summon(state, source, entity_type)?,
            ExecuteStage::Store(spec) => {
                let callback = spec.bind(state, snapshot)?;
                let mut next = source.clone();
                next.callbacks.push(callback);
                next
            }
            // Forking stages are handled by `apply_modifier`.
            _ => return Err(CommandError::InvalidSyntax),
        })
    }

    /// `execute at <entity>`: `withLevel(entity.level()).withPosition(entity.position())
    /// .withRotation(entity.getRotationVector())`.
    fn at_entity(
        &self,
        state: &ServerCommandState,
        source: &ExecuteSource,
        entity: &EntityRef,
    ) -> ExecuteSource {
        let mut next = source.clone();
        match locate_entity(state, entity) {
            Some((dimension, position)) => {
                next = with_level(&next, &dimension);
                next.snapshot.position = position;
            }
            None => {
                if let Some(entry) = entity_state(state, entity) {
                    next = with_level(&next, &entry.dimension);
                }
            }
        }
        let (pitch, yaw) = entity_rotation(state, entity);
        next.snapshot.pitch = pitch;
        next.snapshot.yaw = yaw;
        next
    }

    /// `spawnEntityAndRedirect`: `SummonCommand.createEntity` at the source position, then
    /// `source.withEntity(entity)`.
    fn summon(
        &self,
        state: &mut ServerCommandState,
        source: &ExecuteSource,
        entity_type: &str,
    ) -> Result<ExecuteSource, CommandError> {
        apply_execute_source(state, self.original, &source.snapshot);
        let position = &source.snapshot.position;
        let command = format!(
            "summon {entity_type} {} {} {}",
            position.x, position.y, position.z
        );
        let parts: Vec<&str> = command.split_whitespace().collect();
        summon_command(state, &parts)?;
        let entity = state
            .summoned_entities
            .last()
            .map(|summoned| summoned.entity.clone())
            .ok_or(CommandError::SummonFailed)?;
        Ok(source.with_entity(entity))
    }
}

/// `CommandSourceStack.withLevel`: switching dimension rescales x/z by the coordinate scales.
fn with_level(source: &ExecuteSource, dimension: &str) -> ExecuteSource {
    let mut next = source.clone();
    if next.snapshot.dimension != dimension {
        let scale = dimension_coordinate_scale(&next.snapshot.dimension)
            / dimension_coordinate_scale(dimension);
        next.snapshot.position.x *= scale;
        next.snapshot.position.z *= scale;
        next.snapshot.dimension = dimension.to_string();
    }
    next
}

/// The entities an `execute on <relation>` step expands to (`expandOneToOneEntityRelation` /
/// `expandOneToManyEntityRelation`), dropping removed entities.
fn related_entities(
    state: &ServerCommandState,
    entity: Option<&EntityRef>,
    relation: OnRelation,
) -> Vec<EntityRef> {
    let Some(entity) = entity else {
        return Vec::new();
    };
    let linked = |kind: EntityRelationKind| {
        state
            .entity_relations
            .iter()
            .find(|entry| entry.entity.id == entity.id && entry.kind == kind)
            .map(|entry| entry.target.clone())
    };
    let candidates: Vec<EntityRef> = match relation {
        OnRelation::Owner => linked(EntityRelationKind::Owner).into_iter().collect(),
        OnRelation::Leasher => linked(EntityRelationKind::Leasher).into_iter().collect(),
        OnRelation::Target => linked(EntityRelationKind::Target).into_iter().collect(),
        OnRelation::Attacker => linked(EntityRelationKind::Attacker).into_iter().collect(),
        OnRelation::Controller => linked(EntityRelationKind::Controller).into_iter().collect(),
        OnRelation::Origin => linked(EntityRelationKind::Origin).into_iter().collect(),
        OnRelation::Vehicle => state
            .entity_mounts
            .iter()
            .find(|mount| mount.target.id == entity.id)
            .map(|mount| mount.vehicle.clone())
            .into_iter()
            .collect(),
        OnRelation::Passengers => state
            .entity_mounts
            .iter()
            .filter(|mount| mount.vehicle.id == entity.id)
            .map(|mount| mount.target.clone())
            .collect(),
    };
    candidates
        .into_iter()
        .filter(|candidate| !entity_is_removed(state, candidate))
        .collect()
}

/// `FunctionArgument.getFunctions`: a function id or a `#tag` (which may be empty).
fn resolve_condition_functions(
    state: &ServerCommandState,
    name: &str,
) -> Result<Vec<CommandFunctionDefinition>, CommandError> {
    let (id, tag) = parse_schedule_function(name)?;
    if tag {
        let entry = state
            .function_tags
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| translatable("arguments.function.tag.unknown", [id.clone()]))?;
        return Ok(entry
            .functions
            .iter()
            .filter_map(|function_id| {
                state
                    .available_functions
                    .iter()
                    .find(|function| function.id == *function_id)
                    .cloned()
            })
            .collect());
    }
    state
        .available_functions
        .iter()
        .find(|function| function.id == id)
        .cloned()
        .map(|function| vec![function])
        .ok_or_else(|| translatable("arguments.function.unknown", [id]))
}
