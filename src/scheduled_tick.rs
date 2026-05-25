#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::block_update::BlockPos;
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TickPriority {
    ExtremelyHigh = -3,
    VeryHigh = -2,
    High = -1,
    Normal = 0,
    Low = 1,
    VeryLow = 2,
    ExtremelyLow = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTick {
    pub ty: String,
    pub pos: BlockPos,
    pub trigger_tick: i64,
    pub priority: TickPriority,
    pub sub_tick_order: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedTick {
    pub ty: String,
    pub pos: BlockPos,
    pub delay: i32,
    pub priority: TickPriority,
}

#[derive(Debug, Clone)]
pub struct TickContainer {
    queue: BinaryHeap<QueuedTick>,
    unique: BTreeSet<(BlockPos, String)>,
}

#[derive(Debug, Clone)]
pub struct LevelTickQueues {
    containers: BTreeMap<ChunkPos, TickContainer>,
    next_sub_tick: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueuedTick(ScheduledTick);

impl TickPriority {
    pub fn by_value(value: i32) -> Self {
        match value {
            i32::MIN..=-4 => Self::ExtremelyHigh,
            -3 => Self::ExtremelyHigh,
            -2 => Self::VeryHigh,
            -1 => Self::High,
            0 => Self::Normal,
            1 => Self::Low,
            2 => Self::VeryLow,
            3..=i32::MAX => Self::ExtremelyLow,
        }
    }

    pub const fn value(self) -> i32 {
        self as i32
    }
}

impl ScheduledTick {
    pub fn create(
        game_time: i64,
        sub_tick_order: i64,
        pos: BlockPos,
        ty: impl Into<String>,
        delay: i32,
        priority: TickPriority,
    ) -> Self {
        Self {
            ty: ty.into(),
            pos,
            trigger_tick: game_time + delay as i64,
            priority,
            sub_tick_order,
        }
    }

    pub fn to_saved_tick(&self, current_tick: i64) -> SavedTick {
        SavedTick {
            ty: self.ty.clone(),
            pos: self.pos,
            delay: (self.trigger_tick - current_tick) as i32,
            priority: self.priority,
        }
    }
}

impl SavedTick {
    pub fn unpack(&self, current_tick: i64, sub_tick_order: i64) -> ScheduledTick {
        ScheduledTick {
            ty: self.ty.clone(),
            pos: self.pos,
            trigger_tick: current_tick + self.delay as i64,
            priority: self.priority,
            sub_tick_order,
        }
    }
}

impl TickContainer {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            unique: BTreeSet::new(),
        }
    }

    pub fn schedule(&mut self, tick: ScheduledTick) -> bool {
        if self.unique.insert((tick.pos, tick.ty.clone())) {
            self.queue.push(QueuedTick(tick));
            true
        } else {
            false
        }
    }

    pub fn peek(&self) -> Option<&ScheduledTick> {
        self.queue.peek().map(|tick| &tick.0)
    }

    pub fn poll(&mut self) -> Option<ScheduledTick> {
        let tick = self.queue.pop()?.0;
        self.unique.remove(&(tick.pos, tick.ty.clone()));
        Some(tick)
    }

    pub fn has_scheduled_tick(&self, pos: BlockPos, ty: &str) -> bool {
        self.unique.contains(&(pos, ty.to_string()))
    }

    pub fn count(&self) -> usize {
        self.queue.len()
    }

    pub fn pack(&self, current_tick: i64) -> Vec<SavedTick> {
        self.queue
            .iter()
            .map(|tick| tick.0.to_saved_tick(current_tick))
            .collect()
    }
}

impl LevelTickQueues {
    pub fn new() -> Self {
        Self {
            containers: BTreeMap::new(),
            next_sub_tick: 0,
        }
    }

    pub fn add_container(&mut self, chunk: ChunkPos) {
        self.containers
            .entry(chunk)
            .or_insert_with(TickContainer::new);
    }

    pub fn create_tick(
        &mut self,
        game_time: i64,
        pos: BlockPos,
        ty: impl Into<String>,
        delay: i32,
        priority: TickPriority,
    ) -> ScheduledTick {
        let sub_tick_order = self.next_sub_tick;
        self.next_sub_tick += 1;
        ScheduledTick::create(game_time, sub_tick_order, pos, ty, delay, priority)
    }

    pub fn schedule(&mut self, tick: ScheduledTick) -> bool {
        let chunk = chunk_pos_for_block(tick.pos);
        match self.containers.get_mut(&chunk) {
            Some(container) => container.schedule(tick),
            None => false,
        }
    }

    pub fn has_scheduled_tick(&self, pos: BlockPos, ty: &str) -> bool {
        self.containers
            .get(&chunk_pos_for_block(pos))
            .is_some_and(|container| container.has_scheduled_tick(pos, ty))
    }

    pub fn tick(
        &mut self,
        current_tick: i64,
        max_ticks_to_process: usize,
        can_tick_chunk: impl Fn(ChunkPos) -> bool,
    ) -> Vec<ScheduledTick> {
        let mut runnable = Vec::new();
        loop {
            if runnable.len() >= max_ticks_to_process {
                break;
            }
            let Some(chunk) = self.next_runnable_container(current_tick, &can_tick_chunk) else {
                break;
            };
            let Some(tick) = self
                .containers
                .get_mut(&chunk)
                .and_then(TickContainer::poll)
            else {
                break;
            };
            runnable.push(tick);
        }
        runnable
    }

    pub fn count(&self) -> usize {
        self.containers.values().map(TickContainer::count).sum()
    }

    fn next_runnable_container(
        &self,
        current_tick: i64,
        can_tick_chunk: &impl Fn(ChunkPos) -> bool,
    ) -> Option<ChunkPos> {
        self.containers
            .iter()
            .filter_map(|(chunk, container)| {
                let tick = container.peek()?;
                (tick.trigger_tick <= current_tick && can_tick_chunk(*chunk))
                    .then_some((*chunk, tick))
            })
            .min_by(|(_, a), (_, b)| intra_tick_order(a, b))
            .map(|(chunk, _)| chunk)
    }
}

impl Ord for QueuedTick {
    fn cmp(&self, other: &Self) -> Ordering {
        drain_order(&self.0, &other.0).reverse()
    }
}

impl PartialOrd for QueuedTick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn drain_order(a: &ScheduledTick, b: &ScheduledTick) -> Ordering {
    a.trigger_tick
        .cmp(&b.trigger_tick)
        .then_with(|| a.priority.cmp(&b.priority))
        .then_with(|| a.sub_tick_order.cmp(&b.sub_tick_order))
}

pub fn intra_tick_order(a: &ScheduledTick, b: &ScheduledTick) -> Ordering {
    a.priority
        .cmp(&b.priority)
        .then_with(|| a.sub_tick_order.cmp(&b.sub_tick_order))
}

pub fn chunk_pos_for_block(pos: BlockPos) -> ChunkPos {
    ChunkPos {
        x: pos.x.div_euclid(16),
        z: pos.z.div_euclid(16),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_pos_for_block, drain_order, LevelTickQueues, SavedTick, ScheduledTick, TickContainer,
        TickPriority,
    };
    use crate::block_update::BlockPos;
    use crate::storage::region::ChunkPos;
    use std::cmp::Ordering;

    #[test]
    fn tick_priority_by_value_clamps_like_vanilla_codec() {
        assert_eq!(TickPriority::by_value(-99), TickPriority::ExtremelyHigh);
        assert_eq!(TickPriority::by_value(-2), TickPriority::VeryHigh);
        assert_eq!(TickPriority::by_value(0), TickPriority::Normal);
        assert_eq!(TickPriority::by_value(99), TickPriority::ExtremelyLow);
        assert_eq!(TickPriority::High.value(), -1);
    }

    #[test]
    fn scheduled_tick_drain_order_uses_time_priority_then_sub_tick() {
        let pos = BlockPos { x: 0, y: 64, z: 0 };
        let later = ScheduledTick::create(10, 0, pos, "minecraft:stone", 5, TickPriority::Normal);
        let high = ScheduledTick::create(10, 2, pos, "minecraft:water", 1, TickPriority::High);
        let normal = ScheduledTick::create(10, 1, pos, "minecraft:lava", 1, TickPriority::Normal);

        assert_eq!(drain_order(&high, &later), Ordering::Less);
        assert_eq!(drain_order(&high, &normal), Ordering::Less);
        assert_eq!(drain_order(&normal, &high), Ordering::Greater);
    }

    #[test]
    fn chunk_tick_container_deduplicates_by_type_and_position() {
        let pos = BlockPos { x: 4, y: 70, z: 4 };
        let mut container = TickContainer::new();

        assert!(container.schedule(ScheduledTick::create(
            100,
            0,
            pos,
            "minecraft:redstone_wire",
            2,
            TickPriority::Normal
        )));
        assert!(!container.schedule(ScheduledTick::create(
            100,
            1,
            pos,
            "minecraft:redstone_wire",
            4,
            TickPriority::High
        )));
        assert!(container.has_scheduled_tick(pos, "minecraft:redstone_wire"));
        assert_eq!(container.count(), 1);
        assert_eq!(container.poll().unwrap().trigger_tick, 102);
        assert!(!container.has_scheduled_tick(pos, "minecraft:redstone_wire"));
    }

    #[test]
    fn saved_ticks_store_delay_relative_to_current_tick_and_unpack_with_negative_subtick_base() {
        let pos = BlockPos {
            x: -1,
            y: 12,
            z: 31,
        };
        let tick = ScheduledTick::create(200, 7, pos, "minecraft:water", 20, TickPriority::Low);
        let saved = tick.to_saved_tick(205);
        assert_eq!(
            saved,
            SavedTick {
                ty: "minecraft:water".to_string(),
                pos,
                delay: 15,
                priority: TickPriority::Low
            }
        );

        let unpacked = saved.unpack(300, -1);
        assert_eq!(unpacked.trigger_tick, 315);
        assert_eq!(unpacked.sub_tick_order, -1);
    }

    #[test]
    fn level_ticks_run_due_ticks_from_ticking_chunks_in_intra_tick_order() {
        let mut queues = LevelTickQueues::new();
        queues.add_container(ChunkPos { x: 0, z: 0 });
        queues.add_container(ChunkPos { x: 1, z: 0 });
        let a = queues.create_tick(
            100,
            BlockPos { x: 2, y: 64, z: 2 },
            "minecraft:stone",
            0,
            TickPriority::Normal,
        );
        let b = queues.create_tick(
            100,
            BlockPos { x: 17, y: 64, z: 2 },
            "minecraft:water",
            0,
            TickPriority::High,
        );
        queues.schedule(a);
        queues.schedule(b);

        let ran = queues.tick(100, 10, |_| true);
        assert_eq!(
            ran.iter().map(|tick| tick.ty.as_str()).collect::<Vec<_>>(),
            vec!["minecraft:water", "minecraft:stone"]
        );
        assert_eq!(queues.count(), 0);
    }

    #[test]
    fn level_ticks_respect_loaded_chunk_check_and_max_ticks() {
        let mut queues = LevelTickQueues::new();
        queues.add_container(ChunkPos { x: 0, z: 0 });
        queues.add_container(ChunkPos { x: 1, z: 0 });
        for x in [1, 2, 17] {
            let tick = queues.create_tick(
                50,
                BlockPos { x, y: 64, z: 0 },
                format!("type_{x}"),
                0,
                TickPriority::Normal,
            );
            assert!(queues.schedule(tick));
        }

        let ran = queues.tick(50, 1, |chunk| chunk == ChunkPos { x: 0, z: 0 });
        assert_eq!(ran.len(), 1);
        assert_eq!(chunk_pos_for_block(ran[0].pos), ChunkPos { x: 0, z: 0 });
        assert_eq!(queues.count(), 2);
        assert!(queues.has_scheduled_tick(BlockPos { x: 17, y: 64, z: 0 }, "type_17"));
    }
}
