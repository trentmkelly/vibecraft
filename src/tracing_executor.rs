#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TracingExecutorAction {
    ServiceExecute(TracingRunnablePlan),
    ReturnDirectService,
    Shutdown,
    AwaitTermination { timeout: u64, unit: TimeUnitModel },
    ShutdownNow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingRunnablePlan {
    pub before_run: Vec<TracingRunnableAction>,
    pub run: TracingRunnableAction,
    pub on_command_throwable: Vec<TracingRunnableAction>,
    pub after_success: Vec<TracingRunnableAction>,
    pub finally: Vec<TracingRunnableAction>,
}

impl TracingRunnablePlan {
    pub fn original_command() -> Self {
        Self {
            before_run: Vec::new(),
            run: TracingRunnableAction::RunCommand,
            on_command_throwable: Vec::new(),
            after_success: Vec::new(),
            finally: Vec::new(),
        }
    }

    pub fn traced(name: &str, running_in_ide: bool, rename_thread: bool) -> Self {
        let mut before_run = Vec::new();
        let mut finally = Vec::new();

        if rename_thread {
            before_run.push(TracingRunnableAction::CaptureCurrentThreadName);
            before_run.push(TracingRunnableAction::SetThreadName(name.to_string()));
            finally.push(TracingRunnableAction::RestoreThreadName);
        }

        before_run.push(TracingRunnableAction::BeginZone {
            name: name.to_string(),
            running_in_ide,
        });

        Self {
            before_run,
            run: TracingRunnableAction::RunCommand,
            on_command_throwable: vec![
                TracingRunnableAction::CloseZoneIfPresentAddingSuppressedCloseThrowable,
                TracingRunnableAction::RethrowCommandThrowable,
            ],
            after_success: vec![TracingRunnableAction::CloseZoneIfPresent],
            finally,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TracingRunnableAction {
    CaptureCurrentThreadName,
    SetThreadName(String),
    BeginZone { name: String, running_in_ide: bool },
    RunCommand,
    CloseZoneIfPresent,
    CloseZoneIfPresentAddingSuppressedCloseThrowable,
    RethrowCommandThrowable,
    RestoreThreadName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnitModel {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwaitTerminationOutcome {
    Terminated,
    TimedOut,
    Interrupted,
}

pub fn tracing_executor_for_name_actions(
    name: &str,
    running_in_ide: bool,
    tracy_available: bool,
) -> Vec<TracingExecutorAction> {
    if running_in_ide {
        return vec![TracingExecutorAction::ServiceExecute(
            TracingRunnablePlan::traced(name, running_in_ide, true),
        )];
    }

    if tracy_available {
        vec![TracingExecutorAction::ServiceExecute(
            TracingRunnablePlan::traced(name, running_in_ide, false),
        )]
    } else {
        vec![TracingExecutorAction::ReturnDirectService]
    }
}

pub fn tracing_executor_execute_actions(
    tracy_available: bool,
    running_in_ide: bool,
) -> Vec<TracingExecutorAction> {
    vec![TracingExecutorAction::ServiceExecute(
        tracing_executor_wrap_unnamed_plan(tracy_available, running_in_ide),
    )]
}

pub fn tracing_executor_wrap_unnamed_plan(
    tracy_available: bool,
    running_in_ide: bool,
) -> TracingRunnablePlan {
    if tracy_available {
        TracingRunnablePlan::traced("task", running_in_ide, false)
    } else {
        TracingRunnablePlan::original_command()
    }
}

pub fn tracing_executor_shutdown_and_await_actions(
    timeout: u64,
    unit: TimeUnitModel,
    await_outcome: AwaitTerminationOutcome,
) -> Vec<TracingExecutorAction> {
    let mut actions = vec![
        TracingExecutorAction::Shutdown,
        TracingExecutorAction::AwaitTermination { timeout, unit },
    ];

    if await_outcome != AwaitTerminationOutcome::Terminated {
        actions.push(TracingExecutorAction::ShutdownNow);
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_executor_for_name_ide_branch_matches_java_thread_rename_zone_order() {
        assert_eq!(
            tracing_executor_for_name_actions("Server thread", true, false),
            vec![TracingExecutorAction::ServiceExecute(TracingRunnablePlan {
                before_run: vec![
                    TracingRunnableAction::CaptureCurrentThreadName,
                    TracingRunnableAction::SetThreadName("Server thread".to_string()),
                    TracingRunnableAction::BeginZone {
                        name: "Server thread".to_string(),
                        running_in_ide: true,
                    },
                ],
                run: TracingRunnableAction::RunCommand,
                on_command_throwable: vec![
                    TracingRunnableAction::CloseZoneIfPresentAddingSuppressedCloseThrowable,
                    TracingRunnableAction::RethrowCommandThrowable,
                ],
                after_success: vec![TracingRunnableAction::CloseZoneIfPresent],
                finally: vec![TracingRunnableAction::RestoreThreadName],
            })]
        );
    }

    #[test]
    fn tracing_executor_for_name_non_ide_branches_match_java_tracy_gate() {
        assert_eq!(
            tracing_executor_for_name_actions("IO", false, true),
            vec![TracingExecutorAction::ServiceExecute(TracingRunnablePlan {
                before_run: vec![TracingRunnableAction::BeginZone {
                    name: "IO".to_string(),
                    running_in_ide: false,
                }],
                run: TracingRunnableAction::RunCommand,
                on_command_throwable: vec![
                    TracingRunnableAction::CloseZoneIfPresentAddingSuppressedCloseThrowable,
                    TracingRunnableAction::RethrowCommandThrowable,
                ],
                after_success: vec![TracingRunnableAction::CloseZoneIfPresent],
                finally: Vec::new(),
            })]
        );
        assert_eq!(
            tracing_executor_for_name_actions("IO", false, false),
            vec![TracingExecutorAction::ReturnDirectService]
        );
    }

    #[test]
    fn tracing_executor_execute_wraps_unnamed_like_java() {
        assert_eq!(
            tracing_executor_execute_actions(true, true),
            vec![TracingExecutorAction::ServiceExecute(TracingRunnablePlan {
                before_run: vec![TracingRunnableAction::BeginZone {
                    name: "task".to_string(),
                    running_in_ide: true,
                }],
                run: TracingRunnableAction::RunCommand,
                on_command_throwable: vec![
                    TracingRunnableAction::CloseZoneIfPresentAddingSuppressedCloseThrowable,
                    TracingRunnableAction::RethrowCommandThrowable,
                ],
                after_success: vec![TracingRunnableAction::CloseZoneIfPresent],
                finally: Vec::new(),
            })]
        );
        assert_eq!(
            tracing_executor_wrap_unnamed_plan(false, true),
            TracingRunnablePlan::original_command()
        );
    }

    #[test]
    fn tracing_executor_shutdown_and_await_matches_java_termination_gate() {
        assert_eq!(
            tracing_executor_shutdown_and_await_actions(
                30,
                TimeUnitModel::Seconds,
                AwaitTerminationOutcome::Terminated,
            ),
            vec![
                TracingExecutorAction::Shutdown,
                TracingExecutorAction::AwaitTermination {
                    timeout: 30,
                    unit: TimeUnitModel::Seconds,
                },
            ]
        );

        for outcome in [
            AwaitTerminationOutcome::TimedOut,
            AwaitTerminationOutcome::Interrupted,
        ] {
            assert_eq!(
                tracing_executor_shutdown_and_await_actions(1, TimeUnitModel::Minutes, outcome),
                vec![
                    TracingExecutorAction::Shutdown,
                    TracingExecutorAction::AwaitTermination {
                        timeout: 1,
                        unit: TimeUnitModel::Minutes,
                    },
                    TracingExecutorAction::ShutdownNow,
                ]
            );
        }
    }
}
