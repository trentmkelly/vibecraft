use std::collections::VecDeque;

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
    Eyes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultCallback {
    Empty,
    StoreSuccess,
    StoreResult,
    ReturnFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandOutcome {
    pub success: bool,
    pub result: i32,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFrame {
    pub depth: usize,
    pub return_value: Option<CommandOutcome>,
    pub discarded: bool,
}

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

    pub fn with_rotation(mut self, rotation: (f32, f32)) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }

    pub fn with_callback(mut self, callback: ResultCallback) -> Self {
        self.callbacks = chain_callbacks(&self.callbacks, callback);
        self
    }
}

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
        }
    }

    pub fn with_max_queue_depth(mut self, max_queue_depth: usize) -> Self {
        self.max_queue_depth = max_queue_depth;
        self
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

pub fn execute_as(
    source: &CommandSourceStackModel,
    entities: &[&str],
) -> Vec<CommandSourceStackModel> {
    entities
        .iter()
        .map(|entity| source.clone().with_entity(*entity))
        .collect()
}

pub fn execute_positioned(
    source: &CommandSourceStackModel,
    position: Vec3,
) -> CommandSourceStackModel {
    source.clone().with_position(position)
}

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

fn apply_callback(callbacks: &mut Vec<CommandOutcome>, outcome: CommandOutcome) {
    callbacks.push(outcome);
}

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
