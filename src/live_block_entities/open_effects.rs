//! `ContainerOpenersCounter` callbacks for chests, trapped chests, barrels and
//! shulker boxes, driven by the menu open/close events of
//! [`super::openers::ContainerOpeners`].
//!
//! * Chest / trapped chest (`ChestBlockEntity`): `onOpen` / `onClose` play the
//!   chest sounds (`ChestBlockEntity.playSound`), `openerCountChanged` sends
//!   `level.blockEvent(pos, block, 1, current)` so clients animate the lid
//!   (`ChestLidController.shouldBeOpen`).
//! * Barrel (`BarrelBlockEntity`): `onOpen` / `onClose` play the barrel sounds and
//!   write `BarrelBlock.OPEN`.
//! * Shulker box (`ShulkerBoxBlockEntity`): keeps its own `openCount`, sends the
//!   block event on every change and plays the box sounds on the first open /
//!   last close.
//!
//! The scheduled `recheckOpen` tick (`ChestBlock.tick` / `BarrelBlock.tick`,
//! five ticks after the first open and then every five ticks while anyone has
//! the container open) is driven by [`OpenCounters::process`].
//!
//! TODO(container-game-events): `level.gameEvent(CONTAINER_OPEN/CLOSE)` has no
//! live game-event dispatcher (vibrations, sculk sensors) to notify yet.
//! TODO(trapped-chest-neighbor-updates): a changed trapped-chest viewer count
//! also calls `level.updateNeighborsAt` on the chest and the block below; there
//! is no live redstone neighbour-update system. The count is exposed through
//! [`OpenCounters::open_count`] (`ChestBlockEntity.getOpenCount`) for that
//! future consumer.
//! TODO(double-chest-menu): the live chest menu only opens the clicked half, so
//! a double chest registers one opener instead of `CompoundContainer.startOpen`
//! registering both halves.

use std::collections::BTreeMap;

use rand::Rng;

use crate::block_entity::{
    BlockEntityTypeId, ContainerOpenersCounterEffect, ContainerOpenersCounterModel,
    ContainerUserOpenState,
};
use crate::block_update::BlockPos;
use crate::network::compression::CompressionState;
use crate::network::play::{
    ClientboundBlockEventPacket, ClientboundSoundPacket, SoundEventHolder, SoundSource, Vec3,
    CLIENTBOUND_BLOCK_EVENT_PACKET_ID, CLIENTBOUND_SOUND_PACKET_ID,
};
use crate::network::status::write_framed_packet_with_compression;
use crate::storage::chunk::BlockStateEntry;

use super::container::ChunkWorld;
use super::lifecycle::{block_entity_identity, type_accepts_block};
use super::openers::{ContainerOpeners, OpenEvent};
use super::registry_ids::{block_protocol_id, sound_event_protocol_id};
use super::{format_block_state, BlockStateChange};

/// `ContainerOpenersCounter.CHECK_TICK_DELAY`.
const CHECK_TICK_DELAY: i64 = ContainerOpenersCounterModel::CHECK_TICK_DELAY as i64;

/// The `BlockEntityType`s whose menus register openers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenerKind {
    Chest,
    Barrel,
    ShulkerBox,
}

/// The world-facing result of processing open/close events.
#[derive(Debug, Default, PartialEq)]
pub struct OpenOutcome {
    /// Framed clientbound packets (`VarInt length` + payload) to broadcast.
    pub frames: Vec<u8>,
    /// Block-state writes (`BarrelBlock.OPEN`).
    pub state_changes: Vec<BlockStateChange>,
}

#[derive(Debug, Default)]
struct CounterEntry {
    model: ContainerOpenersCounterModel,
    /// Game time of the scheduled `recheckOpen` tick.
    recheck_at: Option<i64>,
}

/// The ticker-side `openersCounter` of every chest/barrel plus the shulker
/// boxes' `openCount`.
#[derive(Debug, Default)]
pub struct OpenCounters {
    counters: BTreeMap<BlockPos, CounterEntry>,
    shulker_open_counts: BTreeMap<BlockPos, i32>,
}

impl OpenCounters {
    /// `ChestBlockEntity.getOpenCount(level, pos)`.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "TODO(trapped-chest-neighbor-updates) redstone consumer")
    )]
    pub fn open_count(&self, pos: BlockPos) -> i32 {
        self.counters
            .get(&pos)
            .map_or(0, |entry| entry.model.open_count)
    }

    /// Applies every recorded menu start/stop, then runs the rechecks that came
    /// due at `game_time`.
    pub fn process(
        &mut self,
        world: &ChunkWorld<'_>,
        openers: &ContainerOpeners,
        game_time: i64,
    ) -> OpenOutcome {
        let mut outcome = OpenOutcome::default();
        for event in openers.drain_events() {
            match event {
                OpenEvent::Start { pos, range, .. } => {
                    self.start_open(world, pos, range, game_time, &mut outcome);
                }
                OpenEvent::Stop { pos, .. } => self.stop_open(world, pos, &mut outcome),
            }
        }
        self.run_due_rechecks(world, openers, game_time, &mut outcome);
        outcome
    }

    /// `startOpen` for the block entity at `pos`.
    fn start_open(
        &mut self,
        world: &ChunkWorld<'_>,
        pos: BlockPos,
        range: f64,
        game_time: i64,
        outcome: &mut OpenOutcome,
    ) {
        let Some((kind, state)) = opener_at(world, pos) else {
            return;
        };
        if kind == OpenerKind::ShulkerBox {
            self.shulker_start_open(pos, &state, outcome);
            return;
        }
        let entry = self.counters.entry(pos).or_default();
        let effect = entry.model.increment_openers(range);
        if effect.schedule_recheck_delay.is_some() {
            entry.recheck_at = Some(game_time + CHECK_TICK_DELAY);
        }
        apply_effect(kind, pos, &state, &effect, outcome);
    }

    /// `stopOpen` for the block entity at `pos`.
    fn stop_open(&mut self, world: &ChunkWorld<'_>, pos: BlockPos, outcome: &mut OpenOutcome) {
        let Some((kind, state)) = opener_at(world, pos) else {
            // `!this.remove` failed: the block entity is gone, so is its counter.
            self.counters.remove(&pos);
            self.shulker_open_counts.remove(&pos);
            return;
        };
        if kind == OpenerKind::ShulkerBox {
            self.shulker_stop_open(pos, &state, outcome);
            return;
        }
        let Some(entry) = self.counters.get_mut(&pos) else {
            return;
        };
        let effect = entry.model.decrement_openers();
        apply_effect(kind, pos, &state, &effect, outcome);
    }

    /// `ChestBlock.tick` / `BarrelBlock.tick` -> `recheckOpen`.
    fn run_due_rechecks(
        &mut self,
        world: &ChunkWorld<'_>,
        openers: &ContainerOpeners,
        game_time: i64,
        outcome: &mut OpenOutcome,
    ) {
        let due: Vec<BlockPos> = self
            .counters
            .iter()
            .filter(|(_, entry)| entry.recheck_at.is_some_and(|at| at <= game_time))
            .map(|(pos, _)| *pos)
            .collect();
        for pos in due {
            let Some((kind, state)) = opener_at(world, pos) else {
                self.counters.remove(&pos);
                continue;
            };
            let users: Vec<ContainerUserOpenState> = openers
                .users_at(pos)
                .into_iter()
                .map(|user| ContainerUserOpenState {
                    has_container_open: true,
                    spectator: false,
                    interaction_range: user.range,
                })
                .collect();
            let Some(entry) = self.counters.get_mut(&pos) else {
                continue;
            };
            let effect = entry.model.recheck_openers(&users);
            entry.recheck_at = effect
                .schedule_recheck_delay
                .map(|delay| game_time + i64::from(delay));
            apply_effect(kind, pos, &state, &effect, outcome);
            if entry.model.open_count == 0 && entry.recheck_at.is_none() {
                self.counters.remove(&pos);
            }
        }
    }

    /// `ShulkerBoxBlockEntity.startOpen`.
    fn shulker_start_open(
        &mut self,
        pos: BlockPos,
        state: &BlockStateEntry,
        outcome: &mut OpenOutcome,
    ) {
        let count = self.shulker_open_counts.entry(pos).or_insert(0);
        *count = (*count).max(0) + 1;
        let count = *count;
        push_block_event(outcome, pos, &state.name, count);
        if count == 1 {
            push_sound(outcome, "minecraft:block.shulker_box.open", block_center(pos));
        }
    }

    /// `ShulkerBoxBlockEntity.stopOpen`.
    fn shulker_stop_open(
        &mut self,
        pos: BlockPos,
        state: &BlockStateEntry,
        outcome: &mut OpenOutcome,
    ) {
        let count = self.shulker_open_counts.entry(pos).or_insert(0);
        *count -= 1;
        let count = *count;
        push_block_event(outcome, pos, &state.name, count);
        if count <= 0 {
            push_sound(outcome, "minecraft:block.shulker_box.close", block_center(pos));
        }
    }
}

/// The opener kind of the block entity at `pos` with its block state.
fn opener_at(world: &ChunkWorld<'_>, pos: BlockPos) -> Option<(OpenerKind, BlockStateEntry)> {
    let state = world.block_state(pos)?;
    let (_, ty) = block_entity_identity(&world.block_entity(pos)?)?;
    if !type_accepts_block(ty, &state.name) {
        return None;
    }
    let kind = match ty {
        BlockEntityTypeId::Chest | BlockEntityTypeId::TrappedChest => OpenerKind::Chest,
        BlockEntityTypeId::Barrel => OpenerKind::Barrel,
        BlockEntityTypeId::ShulkerBox => OpenerKind::ShulkerBox,
        _ => return None,
    };
    Some((kind, state))
}

/// `onOpen` / `onClose` / `openerCountChanged` of the chest and barrel
/// counters, in Java's call order.
fn apply_effect(
    kind: OpenerKind,
    pos: BlockPos,
    state: &BlockStateEntry,
    effect: &ContainerOpenersCounterEffect,
    outcome: &mut OpenOutcome,
) {
    if effect.on_open || effect.on_close {
        match kind {
            OpenerKind::Chest => chest_sound(pos, state, effect.on_open, outcome),
            OpenerKind::Barrel => barrel_open_close(pos, state, effect.on_open, outcome),
            OpenerKind::ShulkerBox => {}
        }
    }
    if kind == OpenerKind::Chest {
        // `ChestBlockEntity.signalOpenCount`.
        push_block_event(outcome, pos, &state.name, effect.opener_count_changed.1);
    }
}

/// `ChestBlockEntity.playSound`.
fn chest_sound(pos: BlockPos, state: &BlockStateEntry, open: bool, outcome: &mut OpenOutcome) {
    let chest_type = state.properties.get("type").map(String::as_str);
    if chest_type == Some("left") {
        return;
    }
    let mut at = block_center(pos);
    if chest_type == Some("right") {
        let (step_x, step_z) = connected_step(state);
        at.x += f64::from(step_x) * 0.5;
        at.z += f64::from(step_z) * 0.5;
    }
    let event = if open {
        "minecraft:block.chest.open"
    } else {
        "minecraft:block.chest.close"
    };
    push_sound(outcome, event, at);
}

/// The horizontal step of `ChestBlock.getConnectedDirection` for a RIGHT chest
/// (`facing.getCounterClockWise()`).
fn connected_step(state: &BlockStateEntry) -> (i32, i32) {
    match state.properties.get("facing").map(String::as_str) {
        Some("north") => (-1, 0),
        Some("west") => (0, 1),
        Some("south") => (1, 0),
        Some("east") => (0, -1),
        _ => (0, 0),
    }
}

/// `BarrelBlockEntity`'s `onOpen` / `onClose`.
fn barrel_open_close(
    pos: BlockPos,
    state: &BlockStateEntry,
    open: bool,
    outcome: &mut OpenOutcome,
) {
    let (step_x, step_y, step_z) = match state.properties.get("facing").map(String::as_str) {
        Some("north") => (0.0, 0.0, -1.0),
        Some("south") => (0.0, 0.0, 1.0),
        Some("west") => (-1.0, 0.0, 0.0),
        Some("east") => (1.0, 0.0, 0.0),
        Some("down") => (0.0, -1.0, 0.0),
        _ => (0.0, 1.0, 0.0),
    };
    let mut at = block_center(pos);
    at.x += step_x / 2.0;
    at.y += step_y / 2.0;
    at.z += step_z / 2.0;
    let event = if open {
        "minecraft:block.barrel.open"
    } else {
        "minecraft:block.barrel.close"
    };
    push_sound(outcome, event, at);
    let mut opened = state.clone();
    opened
        .properties
        .insert("open".to_string(), open.to_string());
    outcome.state_changes.push(BlockStateChange {
        pos,
        state: format_block_state(&opened),
    });
}

fn block_center(pos: BlockPos) -> Vec3 {
    Vec3 {
        x: f64::from(pos.x) + 0.5,
        y: f64::from(pos.y) + 0.5,
        z: f64::from(pos.z) + 0.5,
    }
}

/// `level.blockEvent(pos, block, 1, count)` as a `ClientboundBlockEventPacket`.
fn push_block_event(outcome: &mut OpenOutcome, pos: BlockPos, block: &str, count: i32) {
    let Some(block_id) = block_protocol_id(block) else {
        return;
    };
    let packet = ClientboundBlockEventPacket {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        action: 1,
        param: count.clamp(0, i32::from(u8::MAX)) as u8,
        block_id,
    };
    let _ = write_framed_packet_with_compression(
        &mut outcome.frames,
        CompressionState::disabled(),
        CLIENTBOUND_BLOCK_EVENT_PACKET_ID,
        |payload| packet.write(payload),
    );
}

/// `level.playSound(null, x, y, z, event, SoundSource.BLOCKS, 0.5F,
/// random.nextFloat() * 0.1F + 0.9F)`.
fn push_sound(outcome: &mut OpenOutcome, event: &str, position: Vec3) {
    let Some(id) = sound_event_protocol_id(event) else {
        return;
    };
    let mut rng = rand::thread_rng();
    let packet = ClientboundSoundPacket {
        sound: SoundEventHolder::Registered { id },
        source_id: SoundSource::Blocks as i32,
        position,
        volume: 0.5,
        pitch: rng.gen::<f32>() * 0.1 + 0.9,
        seed: rng.gen(),
        entity_id: None,
    };
    let _ = write_framed_packet_with_compression(
        &mut outcome.frames,
        CompressionState::disabled(),
        CLIENTBOUND_SOUND_PACKET_ID,
        |payload| packet.write_position(payload),
    );
}
