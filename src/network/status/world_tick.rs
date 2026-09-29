//! Server-level world tick: the single, shared driver of scheduled block/fluid
//! ticks and per-chunk random ticks.
//!
//! Java runs these from `MinecraftServer.tickChildren` -> `ServerLevel.tick`
//! exactly once per server tick, against the level's own `LevelTicks` queues
//! and `getGameTime()`; they are properties of the *world*, not of any player
//! connection. This module owns the shared queues ([`WorldTicks`], one
//! `Arc<Mutex<..>>` in the status runtime) and the once-per-tick driver
//! ([`tick_server_world`]) that the server tick thread calls. Player sessions
//! only *schedule into* the shared queues (block placement/breaking, chunk
//! load unpacking) and receive the resulting block updates through the
//! [`WorldPacketBus`], like every other world event.
//!
//! Order matches `ServerLevel.tick`: block ticks, fluid ticks, then
//! `ServerChunkCache.tickChunks` (random ticks), then entity simulation
//! (falling blocks).

use std::io;

use super::block_placement_live::{
    apply_block_tick_outcome, process_live_block_ticks, run_live_shape_cascade, tick_falling_blocks,
    LiveBlockWorld, LiveCascade, TickTarget,
};
use super::fire_live::FireEnvironment;
use super::*;
use crate::block_scheduled_ticks::BlockTickOutcome;
use crate::block_states::block_state_entry;
use crate::block_behavior::BlockStateModel;
use crate::random_tick::LevelRandom;

/// The level's scheduled-tick queues plus the game time they are keyed to
/// (Java `ServerLevel.blockTicks` / `fluidTicks` and `getGameTime()`).
pub struct WorldTicks {
    /// Server game time (the shared `ServerClockManager.game_time`), refreshed
    /// by the tick thread at the start of every world tick. Everything that
    /// schedules a tick uses this so delays are relative to the world clock.
    pub game_time: i64,
    pub fluid: LiveFluidTicks,
    pub block: LiveBlockTicks,
}

impl WorldTicks {
    pub fn new(game_time: i64) -> Self {
        Self {
            game_time,
            fluid: LiveFluidTicks::new(),
            block: LiveBlockTicks::new(),
        }
    }
}

/// Shared handle to the level's queues.
pub type SharedWorldTicks = Arc<Mutex<WorldTicks>>;

/// Everything [`tick_server_world`] reads besides the queues themselves.
pub(super) struct ServerWorldTick<'a> {
    pub game_time: i64,
    pub layout: &'a WorldLayout,
    pub seed: i64,
    pub cache: &'a GeneratedChunkCache,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
    /// Java `DedicatedServerProperties.maxChainedNeighborUpdates`.
    pub max_chained_neighbor_updates: i32,
    /// Java `GameRules.RANDOM_TICK_SPEED`.
    pub random_tick_speed: i32,
    /// Weather/difficulty/game-rule inputs of `FireBlock.tick`.
    pub fire: FireEnvironment,
    pub bus: &'a WorldPacketBus,
}

/// Runs one `ServerLevel.tick` worth of scheduled and random ticks and
/// broadcasts the resulting block changes to every session.
pub(super) fn tick_server_world(
    ticks: &SharedWorldTicks,
    random: &mut LevelRandom,
    world: &ServerWorldTick<'_>,
) -> io::Result<()> {
    // Writers capture plain (uncompressed) frames; the bus re-frames them per
    // subscriber, as `tick_live_world_systems` did for block ticks before.
    let mut frames = Vec::new();
    {
        let mut guard = lock_status_mutex(ticks);
        guard.game_time = world.game_time;
        let WorldTicks { fluid, block, .. } = &mut *guard;
        process_live_block_ticks(
            &mut frames,
            CompressionState::disabled(),
            block,
            fluid,
            world.fire,
            world.game_time,
            world.layout,
            world.seed,
            world.cache,
            world.world_items,
            world.max_chained_neighbor_updates,
        )?;
        process_live_fluid_ticks(
            &mut frames,
            CompressionState::disabled(),
            fluid,
            world.game_time,
            world.layout,
            world.seed,
            world.cache,
        )?;
        run_random_ticks(&mut frames, random, block, fluid, world)?;
        let mut cascade = LiveCascade {
            layout: world.layout,
            seed: world.seed,
            cache: world.cache,
            fluid_ticks: fluid,
            block_ticks: block,
            game_time: world.game_time,
            random_roll: (world.game_time as i32).rem_euclid(40),
            max_chained_neighbor_updates: world.max_chained_neighbor_updates,
        };
        tick_falling_blocks(
            &mut frames,
            CompressionState::disabled(),
            &mut cascade,
            world.world_items,
        )?;
    }
    world.bus.publish_frames(&frames)
}

/// Java `BlockBehaviour.randomTick` for the block types with a port. `None`
/// means the state has no random-tick behavior implemented yet.
///
/// TODO(random-tick-behaviors): crops/saplings/grass spread/fire-on-lava/
/// vine growth/ice melt and the rest of the `randomTick` overrides are not
/// ported; only states the authoritative table flags as randomly ticking AND
/// listed here do anything.
fn random_tick_outcome(state: &BlockStateModel) -> Option<BlockTickOutcome> {
    let entry = block_state_entry(&state.registry_id)?;
    match entry.block_type {
        // LeavesBlock.randomTick: `if (decaying(state)) { dropResources;
        // removeBlock(pos, false); }`
        "leaves" | "mangrove_leaves" | "tinted_particle_leaves" | "untinted_particle_leaves" => {
            matches!(
                crate::plant::leaves_decay(state),
                crate::plant::PlantAction::Decay
            )
            .then_some(BlockTickOutcome::Destroy { drop: true })
        }
        _ => None,
    }
}

/// Java `ServerLevel.tickChunk` block/fluid part, for every cached chunk:
/// per section, `tickSpeed` uniformly random positions
/// (`Level.getBlockRandomPos`), each ticked when its state
/// `isRandomlyTicking`.
///
/// TODO(random-tick-range): Java only ticks chunks within the players'
/// simulation distance (`ChunkMap.forEachBlockTickingChunk`); the chunk cache
/// does not track which sessions load which chunk, so every cached chunk
/// ticks (same limitation as the block-entity ticker).
/// TODO(precipitation-tick): the `random.nextInt(48) == 0`
/// `tickPrecipitation` roll (snow/ice/cauldron fill) is not ported.
fn run_random_ticks<W: io::Write>(
    writer: &mut W,
    random: &mut LevelRandom,
    block_ticks: &mut LiveBlockTicks,
    fluid_ticks: &mut LiveFluidTicks,
    world: &ServerWorldTick<'_>,
) -> io::Result<()> {
    if world.random_tick_speed <= 0 {
        return Ok(());
    }
    let mut chunks: Vec<_> = lock_status_mutex(&world.cache.chunks)
        .iter()
        .map(|(pos, chunk)| (*pos, Arc::clone(chunk)))
        .collect();
    chunks.sort_by_key(|(pos, _)| (pos.x, pos.z));
    let live_world = LiveBlockWorld {
        layout: world.layout,
        seed: world.seed,
        cache: world.cache,
    };
    let mut changed = Vec::new();
    for (chunk_pos, chunk) in &chunks {
        let (min_x, min_z) = (chunk_pos.x * 16, chunk_pos.z * 16);
        for section in &chunk.sections {
            let min_y = i32::from(section.y) * 16;
            for _ in 0..world.random_tick_speed {
                let pos = random.block_random_pos(min_x, min_y, min_z, 15);
                let Some(entry) = chunk.get_block_state_model(pos.x, pos.y, pos.z) else {
                    continue;
                };
                let mut state = BlockStateModel::new(entry.name);
                for (key, value) in entry.properties {
                    state = state.with_property(&key, value);
                }
                let randomly_ticking = crate::block_properties::state_physics_by_name(
                    &crate::fluid::block_state_model_name(&state),
                )
                .is_some_and(|physics| physics.is_randomly_ticking);
                if !randomly_ticking {
                    continue;
                }
                if let Some(outcome) = random_tick_outcome(&state) {
                    apply_block_tick_outcome(
                        writer,
                        CompressionState::disabled(),
                        block_ticks,
                        &TickTarget {
                            pos,
                            state: &state,
                            game_time: world.game_time,
                        },
                        outcome,
                        &live_world,
                        world.world_items,
                        &mut changed,
                    )?;
                }
            }
        }
    }
    if !changed.is_empty() {
        let mut cascade = LiveCascade {
            layout: world.layout,
            seed: world.seed,
            cache: world.cache,
            fluid_ticks,
            block_ticks,
            game_time: world.game_time,
            random_roll: (world.game_time as i32).rem_euclid(40),
            max_chained_neighbor_updates: world.max_chained_neighbor_updates,
        };
        run_live_shape_cascade(writer, CompressionState::disabled(), &mut cascade, changed)?;
    }
    Ok(())
}
