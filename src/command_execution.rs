#[cfg(test)]
use std::collections::VecDeque;

#[cfg(test)]
pub const EXECUTION_PACKAGE_NULL_MARKED: bool = true;
#[cfg(test)]
pub const EXECUTION_TASKS_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSourceStackModel {
    pub source: String,
    pub entity: Option<String>,
    pub position: Vec3,
    pub rotation: (f32, f32),
    pub level: String,
    pub permission_level: u8,
    pub silent: bool,
    pub anchor: EntityAnchor,
    pub callbacks: Vec<ResultCallback>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet,
    #[cfg(test)]
    Eyes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCallback {
    Empty,
    #[cfg(test)]
    StoreSuccess,
    #[cfg(test)]
    StoreResult,
    #[cfg(test)]
    ReturnFrame,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandOutcome {
    pub success: bool,
    pub result: i32,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandTask {
    Command {
        command: String,
        source: String,
        returning: bool,
    },
    FunctionCall {
        id: String,
        source: String,
        returning: bool,
    },
    Fallthrough,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainModifiersModel {
    flags: u8,
}

#[cfg(test)]
impl ChainModifiersModel {
    pub const DEFAULT: Self = Self { flags: 0 };
    const FLAG_FORKED: u8 = 1;
    const FLAG_RETURN: u8 = 2;

    pub fn flags(self) -> u8 {
        self.flags
    }

    pub fn is_forked(self) -> bool {
        self.flags & Self::FLAG_FORKED != 0
    }

    pub fn set_forked(self) -> Self {
        self.set_flag(Self::FLAG_FORKED)
    }

    pub fn is_return(self) -> bool {
        self.flags & Self::FLAG_RETURN != 0
    }

    pub fn set_return(self) -> Self {
        self.set_flag(Self::FLAG_RETURN)
    }

    fn set_flag(self, flag: u8) -> Self {
        let new_flags = self.flags | flag;
        if new_flags == self.flags {
            self
        } else {
            Self { flags: new_flags }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFrame {
    pub depth: usize,
    pub return_value: Option<CommandOutcome>,
    pub discarded: bool,
}

#[cfg(test)]
impl CommandFrame {
    pub fn return_success(&mut self, value: i32) {
        self.return_value = Some(CommandOutcome {
            success: true,
            result: value,
        });
    }

    pub fn return_failure(&mut self) {
        self.return_value = Some(CommandOutcome {
            success: false,
            result: 0,
        });
    }

    pub fn discard(&mut self) {
        self.discarded = true;
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceEventModel {
    Command {
        depth: usize,
        command: String,
    },
    Return {
        depth: usize,
        command: String,
        result: i32,
    },
    Error(String),
    Call {
        depth: usize,
        function: String,
        size: usize,
    },
    Closed,
}

#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceCallbacksModel {
    pub events: Vec<TraceEventModel>,
}

#[cfg(test)]
impl TraceCallbacksModel {
    pub fn on_command(&mut self, depth: usize, command: impl Into<String>) {
        self.events.push(TraceEventModel::Command {
            depth,
            command: command.into(),
        });
    }

    pub fn on_return(&mut self, depth: usize, command: impl Into<String>, result: i32) {
        self.events.push(TraceEventModel::Return {
            depth,
            command: command.into(),
            result,
        });
    }

    pub fn on_error(&mut self, message: impl Into<String>) {
        self.events.push(TraceEventModel::Error(message.into()));
    }

    pub fn on_call(&mut self, depth: usize, function: impl Into<String>, size: usize) {
        self.events.push(TraceEventModel::Call {
            depth,
            function: function.into(),
            size,
        });
    }

    pub fn close(&mut self) {
        self.events.push(TraceEventModel::Closed);
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryActionModel {
    Task(CommandTask),
    Fallthrough,
    Isolated(Vec<CommandTask>),
    CallFunction {
        id: String,
        entries: Vec<CommandTask>,
        return_parent_frame: bool,
    },
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandQueueEntryModel {
    pub frame: CommandFrame,
    pub action: EntryActionModel,
}

#[cfg(test)]
impl CommandQueueEntryModel {
    pub fn new(frame: CommandFrame, action: EntryActionModel) -> Self {
        Self { frame, action }
    }

    pub fn execute(self, context: &mut ExecutionContextModel) {
        self.action.execute(context, self.frame);
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContextModel {
    pub command_limit: usize,
    pub fork_limit: usize,
    pub max_queue_depth: usize,
    pub quota: usize,
    pub current_depth: usize,
    pub queue: VecDeque<(CommandFrame, CommandTask)>,
    pub queue_overflow: bool,
    pub fork_limit_reached: bool,
    pub callbacks: Vec<CommandOutcome>,
    pub tracer: Option<TraceCallbacksModel>,
}

impl CommandSourceStackModel {
    pub fn new(source: impl Into<String>, level: impl Into<String>, permission_level: u8) -> Self {
        Self {
            source: source.into(),
            entity: None,
            position: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: (0.0, 0.0),
            level: level.into(),
            permission_level,
            silent: false,
            anchor: EntityAnchor::Feet,
            callbacks: vec![ResultCallback::Empty],
        }
    }

    #[cfg(test)]
    pub fn with_entity(mut self, entity: impl Into<String>) -> Self {
        let entity = entity.into();
        self.source = entity.clone();
        self.entity = Some(entity);
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    #[cfg(test)]
    pub fn with_rotation(mut self, rotation: (f32, f32)) -> Self {
        self.rotation = rotation;
        self
    }

    #[cfg(test)]
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    #[cfg(test)]
    pub fn with_callback(mut self, callback: ResultCallback) -> Self {
        self.callbacks = chain_callbacks(&self.callbacks, callback);
        self
    }
}

#[cfg(test)]
impl ExecutionContextModel {
    pub const MAX_QUEUE_DEPTH: usize = 10_000_000;

    pub fn new(command_limit: usize, fork_limit: usize) -> Self {
        Self {
            command_limit,
            fork_limit,
            max_queue_depth: Self::MAX_QUEUE_DEPTH,
            quota: command_limit,
            current_depth: 0,
            queue: VecDeque::new(),
            queue_overflow: false,
            fork_limit_reached: false,
            callbacks: Vec::new(),
            tracer: None,
        }
    }

    pub fn with_max_queue_depth(mut self, max_queue_depth: usize) -> Self {
        self.max_queue_depth = max_queue_depth;
        self
    }

    pub fn tracer(&self) -> Option<&TraceCallbacksModel> {
        self.tracer.as_ref()
    }

    pub fn set_tracer(&mut self, tracer: Option<TraceCallbacksModel>) {
        self.tracer = tracer;
    }

    pub fn close(&mut self) {
        if let Some(tracer) = &mut self.tracer {
            tracer.close();
        }
    }

    pub fn increment_cost(&mut self) {
        self.quota = self.quota.saturating_sub(1);
    }

    pub fn queue_initial_command(
        &mut self,
        command: impl Into<String>,
        source: &CommandSourceStackModel,
    ) {
        self.queue_next(
            CommandFrame {
                depth: self.current_depth,
                return_value: None,
                discarded: false,
            },
            CommandTask::Command {
                command: command.into(),
                source: source.source.clone(),
                returning: false,
            },
        );
    }

    pub fn queue_initial_function(
        &mut self,
        id: impl Into<String>,
        source: &CommandSourceStackModel,
    ) {
        self.queue_next(
            CommandFrame {
                depth: self.current_depth,
                return_value: None,
                discarded: false,
            },
            CommandTask::FunctionCall {
                id: id.into(),
                source: source.source.clone(),
                returning: false,
            },
        );
    }

    pub fn queue_continuation(
        &mut self,
        command: impl Into<String>,
        original: &CommandSourceStackModel,
        current_sources: &[CommandSourceStackModel],
        returning: bool,
    ) {
        let command = command.into();
        if current_sources.is_empty() {
            if returning {
                self.queue_next(current_frame(self.current_depth), CommandTask::Fallthrough);
            }
            return;
        }
        if current_sources.len() >= self.fork_limit {
            self.fork_limit_reached = true;
            return;
        }
        for source in current_sources {
            self.queue_next(
                current_frame(self.current_depth + 1),
                CommandTask::Command {
                    command: command.clone(),
                    source: source
                        .entity
                        .clone()
                        .unwrap_or_else(|| original.source.clone()),
                    returning,
                },
            );
        }
    }

    pub fn queue_next(&mut self, frame: CommandFrame, task: CommandTask) {
        if self.queue.len() > self.max_queue_depth {
            self.queue.clear();
            self.queue_overflow = true;
            return;
        }
        if !self.queue_overflow {
            self.queue.push_front((frame, task));
        }
    }

    pub fn run<F>(&mut self, mut executor: F) -> Vec<CommandTask>
    where
        F: FnMut(&CommandTask) -> CommandOutcome,
    {
        let mut executed = Vec::new();
        while self.quota > 0 {
            let Some((mut frame, task)) = self.queue.pop_front() else {
                break;
            };
            self.current_depth = frame.depth;
            self.quota -= 1;
            if frame.discarded {
                continue;
            }
            let outcome = executor(&task);
            apply_callback(&mut self.callbacks, outcome);
            if matches!(
                task,
                CommandTask::Command {
                    returning: true,
                    ..
                }
            ) || matches!(
                task,
                CommandTask::FunctionCall {
                    returning: true,
                    ..
                }
            ) {
                frame.return_value = Some(outcome);
                self.discard_at_depth_or_higher(frame.depth);
            }
            executed.push(task);
            if self.queue_overflow {
                break;
            }
        }
        self.current_depth = 0;
        executed
    }

    pub fn discard_at_depth_or_higher(&mut self, depth: usize) {
        self.queue.retain(|(frame, _)| frame.depth < depth);
    }
}

#[cfg(test)]
impl EntryActionModel {
    pub fn execute(self, context: &mut ExecutionContextModel, mut frame: CommandFrame) {
        match self {
            Self::Task(task) => context.queue_next(frame, task),
            Self::Fallthrough => {
                frame.return_failure();
                frame.discard();
                context
                    .callbacks
                    .push(frame.return_value.expect("fallthrough result"));
                context.discard_at_depth_or_higher(frame.depth);
            }
            Self::Isolated(tasks) => {
                let new_frame = current_frame(frame.depth + 1);
                for task in tasks {
                    context.queue_next(new_frame.clone(), task);
                }
            }
            Self::CallFunction {
                id,
                entries,
                return_parent_frame,
            } => {
                context.increment_cost();
                if let Some(tracer) = &mut context.tracer {
                    tracer.on_call(frame.depth, id, entries.len());
                }
                let new_depth = frame.depth + 1;
                let new_frame = if return_parent_frame {
                    CommandFrame {
                        depth: new_depth,
                        ..frame
                    }
                } else {
                    current_frame(new_depth)
                };
                ContinuationTaskModel::schedule(context, new_frame, entries);
            }
        }
    }
}

#[cfg(test)]
pub struct ExecutionControlModel<'a> {
    context: &'a mut ExecutionContextModel,
    frame: CommandFrame,
}

#[cfg(test)]
impl<'a> ExecutionControlModel<'a> {
    pub fn create(context: &'a mut ExecutionContextModel, frame: CommandFrame) -> Self {
        Self { context, frame }
    }

    pub fn queue_next(&mut self, action: EntryActionModel) {
        CommandQueueEntryModel::new(self.frame.clone(), action).execute(self.context);
    }

    pub fn set_tracer(&mut self, tracer: Option<TraceCallbacksModel>) {
        self.context.set_tracer(tracer);
    }

    pub fn tracer(&self) -> Option<&TraceCallbacksModel> {
        self.context.tracer()
    }

    pub fn current_frame(&self) -> &CommandFrame {
        &self.frame
    }
}

#[cfg(test)]
pub struct UnboundEntryActionModel {
    task: CommandTask,
}

#[cfg(test)]
impl UnboundEntryActionModel {
    pub fn new(task: CommandTask) -> Self {
        Self { task }
    }

    pub fn bind(&self, sender: &CommandSourceStackModel) -> EntryActionModel {
        let task = match &self.task {
            CommandTask::Command {
                command, returning, ..
            } => CommandTask::Command {
                command: command.clone(),
                source: sender.source.clone(),
                returning: *returning,
            },
            CommandTask::FunctionCall { id, returning, .. } => CommandTask::FunctionCall {
                id: id.clone(),
                source: sender.source.clone(),
                returning: *returning,
            },
            CommandTask::Fallthrough => CommandTask::Fallthrough,
        };
        EntryActionModel::Task(task)
    }
}

#[cfg(test)]
pub struct ContinuationTaskModel;

#[cfg(test)]
impl ContinuationTaskModel {
    pub fn schedule(
        context: &mut ExecutionContextModel,
        frame: CommandFrame,
        arguments: Vec<CommandTask>,
    ) {
        match arguments.len() {
            0 => {}
            1 | 2 => {
                for argument in arguments {
                    context.queue_next(frame.clone(), argument);
                }
            }
            _ => {
                for argument in arguments.into_iter().rev() {
                    context.queue_next(frame.clone(), argument);
                }
            }
        }
    }
}

#[cfg(test)]
pub fn execute_as(
    source: &CommandSourceStackModel,
    entities: &[&str],
) -> Vec<CommandSourceStackModel> {
    entities
        .iter()
        .map(|entity| source.clone().with_entity(*entity))
        .collect()
}

#[cfg(test)]
pub fn execute_positioned(
    source: &CommandSourceStackModel,
    position: Vec3,
) -> CommandSourceStackModel {
    source.clone().with_position(position)
}

#[cfg(test)]
pub fn propagate_results(
    callbacks: &[ResultCallback],
    outcome: CommandOutcome,
) -> Vec<CommandOutcome> {
    callbacks
        .iter()
        .filter(|callback| **callback != ResultCallback::Empty)
        .map(|_| outcome)
        .collect()
}

#[cfg(test)]
pub fn run_guarded_custom_command(
    callbacks: &mut Vec<CommandOutcome>,
    tracer: Option<&mut TraceCallbacksModel>,
    modifiers: ChainModifiersModel,
    guarded: Result<CommandOutcome, &str>,
) -> Option<CommandOutcome> {
    match guarded {
        Ok(outcome) => {
            callbacks.push(outcome);
            Some(outcome)
        }
        Err(message) => {
            if let Some(tracer) = tracer {
                let suffix = if modifiers.is_forked() {
                    " forked"
                } else {
                    " single"
                };
                tracer.on_error(format!("{message}{suffix}"));
            }
            callbacks.push(CommandOutcome {
                success: false,
                result: 0,
            });
            None
        }
    }
}

#[cfg(test)]
fn chain_callbacks(existing: &[ResultCallback], next: ResultCallback) -> Vec<ResultCallback> {
    if existing == [ResultCallback::Empty] {
        vec![next]
    } else if next == ResultCallback::Empty {
        existing.to_vec()
    } else {
        let mut chained = existing.to_vec();
        chained.push(next);
        chained
    }
}

#[cfg(test)]
fn apply_callback(callbacks: &mut Vec<CommandOutcome>, outcome: CommandOutcome) {
    callbacks.push(outcome);
}

#[cfg(test)]
fn current_frame(depth: usize) -> CommandFrame {
    CommandFrame {
        depth,
        return_value: None,
        discarded: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_stack_mutators_preserve_context_like_command_source_stack() {
        const { assert!(EXECUTION_PACKAGE_NULL_MARKED) };
        const { assert!(EXECUTION_TASKS_PACKAGE_NULL_MARKED) };
        let source = CommandSourceStackModel::new("server", "overworld", 4)
            .with_position(Vec3 {
                x: 1.0,
                y: 64.0,
                z: 2.0,
            })
            .with_rotation((90.0, 20.0))
            .with_level("nether")
            .with_entity("Steve")
            .with_callback(ResultCallback::StoreResult)
            .with_callback(ResultCallback::ReturnFrame);
        let mut source = source;
        source.anchor = EntityAnchor::Eyes;
        assert_eq!(source.source, "Steve");
        assert_eq!(source.entity, Some("Steve".to_string()));
        assert_eq!(source.level, "nether");
        assert_eq!(source.anchor, EntityAnchor::Eyes);
        assert_eq!(
            source.callbacks,
            vec![ResultCallback::StoreResult, ResultCallback::ReturnFrame]
        );
    }

    #[test]
    fn chain_modifiers_frame_trace_and_queue_entry_match_execution_records() {
        let modifiers = ChainModifiersModel::DEFAULT.set_forked().set_return();
        assert!(modifiers.is_forked());
        assert!(modifiers.is_return());
        assert_eq!(modifiers.flags(), 3);
        assert_eq!(modifiers.set_return(), modifiers);

        let mut frame = current_frame(2);
        frame.return_success(7);
        assert_eq!(
            frame.return_value,
            Some(CommandOutcome {
                success: true,
                result: 7
            })
        );
        frame.discard();
        assert!(frame.discarded);

        let mut tracer = TraceCallbacksModel::default();
        tracer.on_command(1, "say hi");
        tracer.on_return(1, "say hi", 1);
        tracer.on_error("bad command");
        tracer.on_call(2, "minecraft:tick", 3);
        tracer.close();
        assert_eq!(
            tracer.events,
            vec![
                TraceEventModel::Command {
                    depth: 1,
                    command: "say hi".to_string()
                },
                TraceEventModel::Return {
                    depth: 1,
                    command: "say hi".to_string(),
                    result: 1
                },
                TraceEventModel::Error("bad command".to_string()),
                TraceEventModel::Call {
                    depth: 2,
                    function: "minecraft:tick".to_string(),
                    size: 3
                },
                TraceEventModel::Closed
            ]
        );

        let mut context = ExecutionContextModel::new(10, 8);
        CommandQueueEntryModel::new(
            current_frame(0),
            EntryActionModel::Task(CommandTask::Command {
                command: "say queued".to_string(),
                source: "server".to_string(),
                returning: false,
            }),
        )
        .execute(&mut context);
        assert_eq!(context.queue.len(), 1);

        context.set_tracer(Some(TraceCallbacksModel::default()));
        context.close();
        assert_eq!(
            context.tracer.as_ref().unwrap().events,
            vec![TraceEventModel::Closed]
        );
    }

    #[test]
    fn execution_control_unbound_action_and_isolated_call_bind_like_java_interfaces() {
        let source = CommandSourceStackModel::new("server", "overworld", 4).with_entity("Alex");
        let mut context = ExecutionContextModel::new(10, 8);
        let frame = current_frame(3);
        let mut control = ExecutionControlModel::create(&mut context, frame.clone());
        control.set_tracer(Some(TraceCallbacksModel::default()));
        assert_eq!(control.current_frame(), &frame);
        assert!(control.tracer().is_some());
        control.queue_next(EntryActionModel::Task(CommandTask::Command {
            command: "say control".to_string(),
            source: "server".to_string(),
            returning: false,
        }));
        assert_eq!(context.queue.len(), 1);

        let unbound = UnboundEntryActionModel::new(CommandTask::Command {
            command: "say bound".to_string(),
            source: "old".to_string(),
            returning: false,
        });
        let action = unbound.bind(&source);
        let mut bound_context = ExecutionContextModel::new(10, 8);
        action.execute(&mut bound_context, current_frame(0));
        assert!(matches!(
            bound_context.queue.front().unwrap().1,
            CommandTask::Command { ref source, .. } if source == "Alex"
        ));

        EntryActionModel::Isolated(vec![CommandTask::Command {
            command: "say isolated".to_string(),
            source: "server".to_string(),
            returning: false,
        }])
        .execute(&mut bound_context, current_frame(1));
        assert!(bound_context.queue.iter().any(|(frame, task)| {
            frame.depth == 2
                && matches!(
                    task,
                    CommandTask::Command { command, .. } if command == "say isolated"
                )
        }));
    }

    #[test]
    fn fallthrough_call_function_and_continuation_tasks_match_java_scheduling() {
        let mut context = ExecutionContextModel::new(10, 8);
        context.set_tracer(Some(TraceCallbacksModel::default()));
        EntryActionModel::CallFunction {
            id: "minecraft:tick".to_string(),
            entries: vec![
                CommandTask::Command {
                    command: "say one".to_string(),
                    source: "server".to_string(),
                    returning: false,
                },
                CommandTask::Command {
                    command: "say two".to_string(),
                    source: "server".to_string(),
                    returning: false,
                },
                CommandTask::Command {
                    command: "say three".to_string(),
                    source: "server".to_string(),
                    returning: false,
                },
            ],
            return_parent_frame: false,
        }
        .execute(&mut context, current_frame(0));
        assert_eq!(context.quota, 9);
        assert_eq!(context.queue.len(), 3);
        assert!(context.queue.iter().all(|(frame, _)| frame.depth == 1));
        assert_eq!(
            context.tracer.as_ref().unwrap().events,
            vec![TraceEventModel::Call {
                depth: 0,
                function: "minecraft:tick".to_string(),
                size: 3
            }]
        );

        let mut fallthrough = ExecutionContextModel::new(10, 8);
        fallthrough.queue_next(
            CommandFrame {
                depth: 2,
                return_value: None,
                discarded: false,
            },
            CommandTask::Command {
                command: "say skipped".to_string(),
                source: "server".to_string(),
                returning: false,
            },
        );
        EntryActionModel::Fallthrough.execute(&mut fallthrough, current_frame(2));
        assert_eq!(
            fallthrough.callbacks,
            vec![CommandOutcome {
                success: false,
                result: 0
            }]
        );
        assert!(fallthrough.queue.is_empty());
    }

    #[test]
    fn custom_command_error_handling_traces_errors_and_fires_failure_callback() {
        let mut callbacks = Vec::new();
        let mut tracer = TraceCallbacksModel::default();
        assert_eq!(
            run_guarded_custom_command(
                &mut callbacks,
                Some(&mut tracer),
                ChainModifiersModel::DEFAULT,
                Ok(CommandOutcome {
                    success: true,
                    result: 4
                })
            ),
            Some(CommandOutcome {
                success: true,
                result: 4
            })
        );
        assert_eq!(
            run_guarded_custom_command(
                &mut callbacks,
                Some(&mut tracer),
                ChainModifiersModel::DEFAULT.set_forked(),
                Err("syntax")
            ),
            None
        );
        assert_eq!(
            callbacks,
            vec![
                CommandOutcome {
                    success: true,
                    result: 4
                },
                CommandOutcome {
                    success: false,
                    result: 0
                }
            ]
        );
        assert_eq!(
            tracer.events,
            vec![TraceEventModel::Error("syntax forked".to_string())]
        );
    }

    #[test]
    fn execute_as_and_positioned_create_forked_sources() {
        let source = CommandSourceStackModel::new("server", "overworld", 2);
        let forked = execute_as(&source, &["Alex", "Steve"]);
        assert_eq!(forked.len(), 2);
        assert_eq!(forked[0].source, "Alex");
        assert_eq!(forked[1].entity, Some("Steve".to_string()));
        let positioned = execute_positioned(
            &forked[0],
            Vec3 {
                x: 8.0,
                y: 70.0,
                z: -3.0,
            },
        );
        assert_eq!(positioned.source, "Alex");
        assert_eq!(positioned.position.x, 8.0);
    }

    #[test]
    fn execution_context_runs_commands_with_quota_and_result_callbacks() {
        let source = CommandSourceStackModel::new("server", "overworld", 4);
        let mut context = ExecutionContextModel::new(2, 8);
        context.queue_initial_command("say first", &source);
        context.queue_initial_command("say second", &source);
        context.queue_initial_command("say third", &source);
        let executed = context.run(|task| CommandOutcome {
            success: matches!(task, CommandTask::Command { .. }),
            result: 1,
        });
        assert_eq!(executed.len(), 2);
        assert_eq!(context.quota, 0);
        assert_eq!(context.callbacks.len(), 2);
        assert_eq!(
            propagate_results(
                &[ResultCallback::StoreSuccess],
                CommandOutcome {
                    success: true,
                    result: 5
                }
            ),
            vec![CommandOutcome {
                success: true,
                result: 5
            }]
        );
    }

    #[test]
    fn return_value_discards_same_depth_continuations() {
        let source = CommandSourceStackModel::new("server", "overworld", 4);
        let mut context = ExecutionContextModel::new(10, 8);
        context.queue_next(
            CommandFrame {
                depth: 1,
                return_value: None,
                discarded: false,
            },
            CommandTask::Command {
                command: "say skipped".to_string(),
                source: source.source.clone(),
                returning: false,
            },
        );
        context.queue_next(
            CommandFrame {
                depth: 1,
                return_value: None,
                discarded: false,
            },
            CommandTask::Command {
                command: "return 7".to_string(),
                source: source.source.clone(),
                returning: true,
            },
        );
        let executed = context.run(|task| CommandOutcome {
            success: true,
            result: if matches!(
                task,
                CommandTask::Command {
                    returning: true,
                    ..
                }
            ) {
                7
            } else {
                1
            },
        });
        assert_eq!(executed.len(), 1);
        assert!(matches!(
            executed[0],
            CommandTask::Command {
                returning: true,
                ..
            }
        ));
        assert!(context.queue.is_empty());
        assert_eq!(
            context.callbacks[0],
            CommandOutcome {
                success: true,
                result: 7
            }
        );
    }

    #[test]
    fn function_continuations_fork_with_limit_and_fallthrough_when_returning_empty() {
        let source = CommandSourceStackModel::new("server", "overworld", 4);
        let sources = execute_as(&source, &["a"]);
        let mut context = ExecutionContextModel::new(10, 2);
        context.queue_initial_function("minecraft:tick", &source);
        context.queue_continuation("say hi", &source, &sources, false);
        assert_eq!(context.queue.len(), 2);
        let executed = context.run(|_| CommandOutcome {
            success: true,
            result: 1,
        });
        assert_eq!(executed.len(), 2);
        assert!(executed.iter().any(|task| matches!(
            task,
            CommandTask::FunctionCall {
                id,
                returning: false,
                ..
            } if id == "minecraft:tick"
        )));
        assert!(!context.fork_limit_reached);

        let mut empty_returning = ExecutionContextModel::new(10, 2);
        empty_returning.queue_continuation("say no targets", &source, &[], true);
        assert_eq!(empty_returning.queue.len(), 1);
        assert!(matches!(
            empty_returning.queue.front().unwrap().1,
            CommandTask::Fallthrough
        ));
    }

    #[test]
    fn execution_context_matches_vanilla_fork_limit_and_queue_overflow_boundary() {
        let source = CommandSourceStackModel::new("server", "overworld", 4);
        let sources = execute_as(&source, &["a", "b", "c", "d"]);
        let mut context = ExecutionContextModel::new(10, 3);
        context.queue_continuation("say fork", &source, &sources, false);
        assert!(context.queue.is_empty());
        assert!(context.fork_limit_reached);

        let allowed_sources = execute_as(&source, &["a", "b"]);
        let mut allowed = ExecutionContextModel::new(10, 3);
        allowed.queue_continuation("say fork", &source, &allowed_sources, false);
        assert_eq!(allowed.queue.len(), 2);
        assert!(!allowed.fork_limit_reached);
        assert!(allowed.queue.iter().all(|(_, task)| matches!(
            task,
            CommandTask::Command {
                command,
                returning: false,
                ..
            } if command == "say fork"
        )));

        let mut overflow = ExecutionContextModel::new(10, 8).with_max_queue_depth(2);
        overflow.queue_initial_command("say one", &source);
        overflow.queue_initial_command("say two", &source);
        overflow.queue_initial_command("say three", &source);
        assert_eq!(overflow.queue.len(), 3);
        assert!(!overflow.queue_overflow);
        overflow.queue_initial_command("say four", &source);
        assert!(overflow.queue.is_empty());
        assert!(overflow.queue_overflow);
    }
}
