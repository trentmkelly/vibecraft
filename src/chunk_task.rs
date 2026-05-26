#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::storage::region::ChunkPos;

pub const PRIORITY_LEVEL_COUNT: usize = 34 + 2;
pub const DISPATCHER_PRIORITY_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkTaskId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksForChunk {
    pub chunk: ChunkPos,
    pub tasks: Vec<ChunkTaskId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatcherOp {
    LevelChange {
        chunk: ChunkPos,
        old_level: usize,
        new_level: usize,
    },
    Release {
        chunk: ChunkPos,
        clear_queue: bool,
    },
    Submit {
        chunk: ChunkPos,
        task: ChunkTaskId,
        level: usize,
    },
    Poll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTaskPriorityQueue {
    queues_per_priority: Vec<BTreeMap<ChunkPos, VecDeque<ChunkTaskId>>>,
    insertion_order: VecDeque<(usize, ChunkPos)>,
    top_priority_queue_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkTaskDispatcher {
    queue: ChunkTaskPriorityQueue,
    dispatcher_ops: Vec<VecDeque<DispatcherOp>>,
    sleeping: bool,
    executed: Vec<TasksForChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrottlingChunkTaskDispatcher {
    inner: ChunkTaskDispatcher,
    chunks_in_execution: BTreeSet<ChunkPos>,
    max_chunks_in_execution: usize,
}

impl ChunkTaskPriorityQueue {
    pub fn new() -> Self {
        Self {
            queues_per_priority: vec![BTreeMap::new(); PRIORITY_LEVEL_COUNT],
            insertion_order: VecDeque::new(),
            top_priority_queue_index: PRIORITY_LEVEL_COUNT,
        }
    }

    pub fn submit(&mut self, task: ChunkTaskId, chunk: ChunkPos, level: usize) {
        let Some(queue) = self.queues_per_priority.get_mut(level) else {
            return;
        };
        let is_new_chunk_at_level = !queue.contains_key(&chunk);
        queue.entry(chunk).or_default().push_back(task);
        if is_new_chunk_at_level {
            self.insertion_order.push_back((level, chunk));
        }
        self.top_priority_queue_index = self.top_priority_queue_index.min(level);
    }

    pub fn resort_chunk_tasks(
        &mut self,
        old_priority: usize,
        chunk: ChunkPos,
        new_priority: usize,
    ) {
        if old_priority >= PRIORITY_LEVEL_COUNT || new_priority >= PRIORITY_LEVEL_COUNT {
            return;
        }
        let Some(tasks) = self.queues_per_priority[old_priority].remove(&chunk) else {
            self.recompute_top_priority();
            return;
        };
        if !tasks.is_empty() {
            let Some(target_queue) = self.queues_per_priority.get_mut(new_priority) else {
                self.recompute_top_priority();
                return;
            };
            let target_was_empty = !target_queue.contains_key(&chunk);
            target_queue.entry(chunk).or_default().extend(tasks);
            if target_was_empty {
                self.insertion_order.push_back((new_priority, chunk));
            }
        }
        self.remove_order_entry(old_priority, chunk);
        self.recompute_top_priority();
    }

    pub fn release(&mut self, chunk: ChunkPos, unschedule: bool) {
        for level in 0..PRIORITY_LEVEL_COUNT {
            let remove = if let Some(tasks) = self.queues_per_priority[level].get_mut(&chunk) {
                if unschedule {
                    tasks.clear();
                }
                tasks.is_empty()
            } else {
                false
            };
            if remove {
                self.queues_per_priority[level].remove(&chunk);
                self.remove_order_entry(level, chunk);
            }
        }
        self.recompute_top_priority();
    }

    pub fn pop(&mut self) -> Option<TasksForChunk> {
        if !self.has_work() {
            return None;
        }
        let level = self.top_priority_queue_index;
        let chunk = self.first_chunk_at_priority(level)?;
        let tasks = self.queues_per_priority[level].remove(&chunk)?;
        self.remove_order_entry(level, chunk);
        self.recompute_top_priority();
        Some(TasksForChunk {
            chunk,
            tasks: tasks.into_iter().collect(),
        })
    }

    pub fn has_work(&self) -> bool {
        self.top_priority_queue_index < PRIORITY_LEVEL_COUNT
    }

    pub fn top_priority_queue_index(&self) -> usize {
        self.top_priority_queue_index
    }

    fn first_chunk_at_priority(&self, level: usize) -> Option<ChunkPos> {
        self.insertion_order
            .iter()
            .find_map(|(entry_level, chunk)| {
                (*entry_level == level && self.queues_per_priority[level].contains_key(chunk))
                    .then_some(*chunk)
            })
    }

    fn remove_order_entry(&mut self, level: usize, chunk: ChunkPos) {
        if let Some(index) = self
            .insertion_order
            .iter()
            .position(|entry| *entry == (level, chunk))
        {
            self.insertion_order.remove(index);
        }
    }

    fn recompute_top_priority(&mut self) {
        self.top_priority_queue_index = self
            .queues_per_priority
            .iter()
            .position(|queue| !queue.is_empty())
            .unwrap_or(PRIORITY_LEVEL_COUNT);
    }
}

impl ChunkTaskDispatcher {
    pub fn new() -> Self {
        Self {
            queue: ChunkTaskPriorityQueue::new(),
            dispatcher_ops: vec![VecDeque::new(); DISPATCHER_PRIORITY_COUNT],
            sleeping: true,
            executed: Vec::new(),
        }
    }

    pub fn on_level_change(&mut self, chunk: ChunkPos, old_level: usize, new_level: usize) {
        self.dispatcher_ops[0].push_back(DispatcherOp::LevelChange {
            chunk,
            old_level,
            new_level,
        });
    }

    pub fn release(&mut self, chunk: ChunkPos, clear_queue: bool) {
        self.dispatcher_ops[1].push_back(DispatcherOp::Release { chunk, clear_queue });
    }

    pub fn submit(&mut self, chunk: ChunkPos, task: ChunkTaskId, level: usize) {
        self.dispatcher_ops[2].push_back(DispatcherOp::Submit { chunk, task, level });
    }

    pub fn poll_task(&mut self) {
        self.dispatcher_ops[3].push_back(DispatcherOp::Poll);
    }

    pub fn run_one_dispatcher_op(&mut self) -> Option<DispatcherOp> {
        let priority = self.dispatcher_ops.iter().position(|ops| !ops.is_empty())?;
        let op = self.dispatcher_ops[priority].pop_front()?;
        self.apply_op(op.clone());
        Some(op)
    }

    pub fn drain_dispatcher(&mut self) -> Vec<DispatcherOp> {
        let mut ops = Vec::new();
        while let Some(op) = self.run_one_dispatcher_op() {
            ops.push(op);
        }
        ops
    }

    pub fn has_work(&self) -> bool {
        self.queue.has_work() || self.dispatcher_ops.iter().any(|ops| !ops.is_empty())
    }

    pub fn executed(&self) -> &[TasksForChunk] {
        &self.executed
    }

    pub fn sleeping(&self) -> bool {
        self.sleeping
    }

    fn apply_op(&mut self, op: DispatcherOp) {
        match op {
            DispatcherOp::LevelChange {
                chunk,
                old_level,
                new_level,
            } => self.queue.resort_chunk_tasks(old_level, chunk, new_level),
            DispatcherOp::Release { chunk, clear_queue } => {
                self.queue.release(chunk, clear_queue);
                if self.sleeping {
                    self.sleeping = false;
                    self.poll_task();
                }
            }
            DispatcherOp::Submit { chunk, task, level } => {
                self.queue.submit(task, chunk, level);
                if self.sleeping {
                    self.sleeping = false;
                    self.poll_task();
                }
            }
            DispatcherOp::Poll => match self.pop_tasks() {
                Some(tasks) => self.schedule_for_execution(tasks),
                None => self.sleeping = true,
            },
        }
    }

    fn pop_tasks(&mut self) -> Option<TasksForChunk> {
        self.queue.pop()
    }

    fn schedule_for_execution(&mut self, tasks: TasksForChunk) {
        self.executed.push(tasks);
        self.poll_task();
    }
}

impl ThrottlingChunkTaskDispatcher {
    pub fn new(max_chunks_in_execution: usize) -> Self {
        Self {
            inner: ChunkTaskDispatcher::new(),
            chunks_in_execution: BTreeSet::new(),
            max_chunks_in_execution,
        }
    }

    pub fn submit(&mut self, chunk: ChunkPos, task: ChunkTaskId, level: usize) {
        self.inner.submit(chunk, task, level);
    }

    pub fn release(&mut self, chunk: ChunkPos, clear_queue: bool) {
        self.inner.dispatcher_ops[1].push_back(DispatcherOp::Release { chunk, clear_queue });
    }

    pub fn drain_dispatcher(&mut self) -> Vec<DispatcherOp> {
        let mut ops = Vec::new();
        while let Some(op) = self.run_one_dispatcher_op() {
            ops.push(op);
        }
        ops
    }

    pub fn run_one_dispatcher_op(&mut self) -> Option<DispatcherOp> {
        let priority = self
            .inner
            .dispatcher_ops
            .iter()
            .position(|ops| !ops.is_empty())?;
        let op = self.inner.dispatcher_ops[priority].pop_front()?;
        self.apply_op(op.clone());
        Some(op)
    }

    pub fn executed(&self) -> &[TasksForChunk] {
        self.inner.executed()
    }

    pub fn chunks_in_execution(&self) -> &BTreeSet<ChunkPos> {
        &self.chunks_in_execution
    }

    fn apply_op(&mut self, op: DispatcherOp) {
        match op {
            DispatcherOp::Release { chunk, clear_queue } => {
                self.inner.queue.release(chunk, clear_queue);
                self.chunks_in_execution.remove(&chunk);
                if self.inner.sleeping {
                    self.inner.sleeping = false;
                    self.inner.poll_task();
                }
            }
            DispatcherOp::Poll => {
                if self.chunks_in_execution.len() < self.max_chunks_in_execution {
                    if let Some(tasks) = self.inner.queue.pop() {
                        self.chunks_in_execution.insert(tasks.chunk);
                        self.inner.schedule_for_execution(tasks);
                    } else {
                        self.inner.sleeping = true;
                    }
                } else {
                    self.inner.sleeping = true;
                }
            }
            other => self.inner.apply_op(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChunkTaskDispatcher, ChunkTaskId, ChunkTaskPriorityQueue, DispatcherOp,
        ThrottlingChunkTaskDispatcher, PRIORITY_LEVEL_COUNT,
    };
    use crate::storage::region::ChunkPos;

    #[test]
    fn priority_queue_pops_lowest_ticket_level_first_and_preserves_chunk_insertion_order() {
        let a = ChunkPos { x: 0, z: 0 };
        let b = ChunkPos { x: 1, z: 0 };
        let c = ChunkPos { x: 2, z: 0 };
        let mut queue = ChunkTaskPriorityQueue::new();
        queue.submit(ChunkTaskId(1), a, 10);
        queue.submit(ChunkTaskId(2), b, 5);
        queue.submit(ChunkTaskId(3), c, 5);
        queue.submit(ChunkTaskId(4), b, 5);

        assert_eq!(queue.top_priority_queue_index(), 5);
        assert_eq!(queue.pop().unwrap().chunk, b);
        assert_eq!(queue.pop().unwrap().chunk, c);
        assert_eq!(queue.pop().unwrap().chunk, a);
        assert_eq!(queue.top_priority_queue_index(), PRIORITY_LEVEL_COUNT);
    }

    #[test]
    fn resort_moves_all_pending_tasks_to_new_priority() {
        let chunk = ChunkPos { x: -3, z: 7 };
        let mut queue = ChunkTaskPriorityQueue::new();
        queue.submit(ChunkTaskId(1), chunk, 20);
        queue.submit(ChunkTaskId(2), chunk, 20);

        queue.resort_chunk_tasks(20, chunk, 4);
        let tasks = queue.pop().unwrap();
        assert_eq!(tasks.chunk, chunk);
        assert_eq!(tasks.tasks, vec![ChunkTaskId(1), ChunkTaskId(2)]);
        assert!(!queue.has_work());
    }

    #[test]
    fn release_can_clear_or_keep_tasks() {
        let chunk = ChunkPos { x: 2, z: 2 };
        let mut queue = ChunkTaskPriorityQueue::new();
        queue.submit(ChunkTaskId(1), chunk, 6);
        queue.release(chunk, false);
        assert_eq!(queue.pop().unwrap().tasks, vec![ChunkTaskId(1)]);

        queue.submit(ChunkTaskId(2), chunk, 6);
        queue.release(chunk, true);
        assert!(!queue.has_work());
    }

    #[test]
    fn dispatcher_operation_priorities_match_vanilla_zero_to_three_order() {
        let chunk = ChunkPos { x: 0, z: 0 };
        let mut dispatcher = ChunkTaskDispatcher::new();
        dispatcher.submit(chunk, ChunkTaskId(1), 8);
        dispatcher.release(chunk, false);
        dispatcher.on_level_change(chunk, 8, 3);
        dispatcher.poll_task();

        let ops = dispatcher.drain_dispatcher();
        assert!(matches!(ops[0], DispatcherOp::LevelChange { .. }));
        assert!(matches!(ops[1], DispatcherOp::Release { .. }));
        assert!(matches!(ops[2], DispatcherOp::Submit { .. }));
        assert!(ops.iter().any(|op| matches!(op, DispatcherOp::Poll)));
    }

    #[test]
    fn dispatcher_wakes_on_submit_executes_chunk_tasks_and_sleeps_when_empty() {
        let chunk = ChunkPos { x: 5, z: -1 };
        let mut dispatcher = ChunkTaskDispatcher::new();
        assert!(dispatcher.sleeping());

        dispatcher.submit(chunk, ChunkTaskId(9), 2);
        dispatcher.drain_dispatcher();

        assert_eq!(dispatcher.executed()[0].chunk, chunk);
        assert_eq!(dispatcher.executed()[0].tasks, vec![ChunkTaskId(9)]);
        assert!(dispatcher.sleeping());
        assert!(!dispatcher.has_work());
    }

    #[test]
    fn throttling_dispatcher_limits_chunks_in_execution_until_release() {
        let a = ChunkPos { x: 0, z: 0 };
        let b = ChunkPos { x: 1, z: 0 };
        let mut dispatcher = ThrottlingChunkTaskDispatcher::new(1);
        dispatcher.submit(a, ChunkTaskId(1), 1);
        dispatcher.submit(b, ChunkTaskId(2), 1);

        dispatcher.drain_dispatcher();
        assert_eq!(dispatcher.executed().len(), 1);
        assert!(dispatcher.chunks_in_execution().contains(&a));

        dispatcher.release(a, false);
        dispatcher.drain_dispatcher();
        assert_eq!(dispatcher.executed().len(), 2);
        assert!(dispatcher.chunks_in_execution().contains(&b));
    }
}
