#![allow(dead_code)]

use std::time::Duration;

use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkOperationKind {
    Load,
    Generate,
    Light,
    Save,
    Unload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkOperation {
    pub chunk: ChunkPos,
    pub kind: ChunkOperationKind,
    pub elapsed: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkOperationDecision {
    Continue,
    YieldToNextTick {
        may_have_delayed_tasks: bool,
    },
    Crash {
        exceeded_by: Duration,
        stats: ChunkWatchdogStats,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkWatchdogStats {
    pub players: usize,
    pub entities: usize,
    pub block_entities: usize,
    pub block_ticks: usize,
    pub fluid_ticks: usize,
    pub chunk_source: String,
    pub slow_chunk: Option<ChunkPos>,
    pub slow_operation: Option<ChunkOperationKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkOperationWatchdog {
    max_tick_time: Duration,
    cooperative_yield_after: Duration,
}

impl ChunkOperationWatchdog {
    pub fn new(max_tick_time: Duration, cooperative_yield_after: Duration) -> Self {
        Self {
            max_tick_time,
            cooperative_yield_after,
        }
    }

    pub fn check_operation(
        &self,
        operation: ChunkOperation,
        stats: ChunkWatchdogStats,
    ) -> ChunkOperationDecision {
        if self.max_tick_time.is_zero() {
            return ChunkOperationDecision::Continue;
        }
        if operation.elapsed > self.max_tick_time {
            return ChunkOperationDecision::Crash {
                exceeded_by: operation.elapsed - self.max_tick_time,
                stats: stats.with_slow_operation(operation),
            };
        }
        if operation.elapsed >= self.cooperative_yield_after {
            return ChunkOperationDecision::YieldToNextTick {
                may_have_delayed_tasks: true,
            };
        }
        ChunkOperationDecision::Continue
    }
}

impl ChunkWatchdogStats {
    pub fn watchdog_line(&self) -> String {
        format!(
            "players: {}, entities: {}, block_entities: {}, block_ticks: {}, fluid_ticks: {}, chunk_source: {}",
            self.players,
            self.entities,
            self.block_entities,
            self.block_ticks,
            self.fluid_ticks,
            self.chunk_source
        )
    }

    fn with_slow_operation(mut self, operation: ChunkOperation) -> Self {
        self.slow_chunk = Some(operation.chunk);
        self.slow_operation = Some(operation.kind);
        self
    }
}

pub fn should_run_delayed_chunk_task(task_tick: u64, current_tick: u64, have_time: bool) -> bool {
    task_tick + 3 < current_tick || have_time
}

pub fn next_chunk_operation_slice(
    remaining: Duration,
    max_slice: Duration,
    time_until_watchdog: Duration,
) -> Duration {
    remaining.min(max_slice).min(time_until_watchdog)
}

#[cfg(test)]
mod tests {
    use super::{
        next_chunk_operation_slice, should_run_delayed_chunk_task, ChunkOperation,
        ChunkOperationDecision, ChunkOperationKind, ChunkOperationWatchdog, ChunkWatchdogStats,
    };
    use crate::storage::region::ChunkPos;
    use std::time::Duration;

    fn stats() -> ChunkWatchdogStats {
        ChunkWatchdogStats {
            players: 2,
            entities: 20,
            block_entities: 3,
            block_ticks: 7,
            fluid_ticks: 5,
            chunk_source: "visible=8 pending=2".to_string(),
            slow_chunk: None,
            slow_operation: None,
        }
    }

    #[test]
    fn slow_chunk_operations_yield_before_watchdog_limit() {
        let watchdog =
            ChunkOperationWatchdog::new(Duration::from_secs(60), Duration::from_millis(45));
        let decision = watchdog.check_operation(
            ChunkOperation {
                chunk: ChunkPos { x: 1, z: 2 },
                kind: ChunkOperationKind::Generate,
                elapsed: Duration::from_millis(45),
            },
            stats(),
        );
        assert_eq!(
            decision,
            ChunkOperationDecision::YieldToNextTick {
                may_have_delayed_tasks: true
            }
        );
    }

    #[test]
    fn chunk_operation_watchdog_crashes_after_max_tick_time_with_level_stats() {
        let watchdog =
            ChunkOperationWatchdog::new(Duration::from_secs(60), Duration::from_millis(45));
        let chunk = ChunkPos { x: -3, z: 4 };
        let decision = watchdog.check_operation(
            ChunkOperation {
                chunk,
                kind: ChunkOperationKind::Light,
                elapsed: Duration::from_secs(61),
            },
            stats(),
        );
        match decision {
            ChunkOperationDecision::Crash { exceeded_by, stats } => {
                assert_eq!(exceeded_by, Duration::from_secs(1));
                assert_eq!(stats.slow_chunk, Some(chunk));
                assert_eq!(stats.slow_operation, Some(ChunkOperationKind::Light));
                assert_eq!(
                    stats.watchdog_line(),
                    "players: 2, entities: 20, block_entities: 3, block_ticks: 7, fluid_ticks: 5, chunk_source: visible=8 pending=2"
                );
            }
            other => panic!("expected crash decision, got {other:?}"),
        }
    }

    #[test]
    fn disabled_watchdog_allows_long_chunk_operations() {
        let watchdog = ChunkOperationWatchdog::new(Duration::ZERO, Duration::from_millis(1));
        assert_eq!(
            watchdog.check_operation(
                ChunkOperation {
                    chunk: ChunkPos { x: 0, z: 0 },
                    kind: ChunkOperationKind::Save,
                    elapsed: Duration::from_secs(999),
                },
                stats(),
            ),
            ChunkOperationDecision::Continue
        );
    }

    #[test]
    fn delayed_chunk_tasks_follow_minecraft_server_three_tick_rule() {
        assert!(should_run_delayed_chunk_task(10, 14, false));
        assert!(!should_run_delayed_chunk_task(10, 13, false));
        assert!(should_run_delayed_chunk_task(10, 13, true));
    }

    #[test]
    fn chunk_operation_slices_never_exceed_remaining_slice_or_watchdog_window() {
        assert_eq!(
            next_chunk_operation_slice(
                Duration::from_millis(100),
                Duration::from_millis(20),
                Duration::from_millis(50)
            ),
            Duration::from_millis(20)
        );
        assert_eq!(
            next_chunk_operation_slice(
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(50)
            ),
            Duration::from_millis(10)
        );
        assert_eq!(
            next_chunk_operation_slice(
                Duration::from_millis(100),
                Duration::from_millis(20),
                Duration::from_millis(5)
            ),
            Duration::from_millis(5)
        );
    }
}
