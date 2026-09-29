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
use super::random_tick_live::{LiveRandomTickWorld, RandomTickEnvironment};
use super::explosion_live::{DirectedPackets, ExplosionEnv, ExplosionSinks};
use super::fire_live::{fire_spread_players, FireEnvironment};
use super::tnt_live::tick_primed_tnts;
use crate::server_explosion::ExplosionRules;
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_scheduled_ticks::BlockTickOutcome;
use crate::block_states::block_state_entry;
use crate::block_survival::SurvivalWorld;
use crate::block_update::BlockPos;
use crate::random_source::LegacyRandom;
use crate::random_tick::LevelRandom;

/// `Level.random`: the level's `RandomSource` (`LegacyRandomSource`), shared
/// by the random-tick pass and every other world-level consumer such as bone
/// meal, so their draws interleave in one sequence like Java's.
pub struct LevelRandomSource(Mutex<LegacyRandom>);

impl Default for LevelRandomSource {
    /// Unseeded in Java (`RandomSource.create()`), so seeded from the clock.
    fn default() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        Self::seeded((nanos >> 3) as i64 ^ 0x5DEE_CE66D)
    }
}

impl LevelRandomSource {
    pub fn seeded(seed: i64) -> Self {
        Self(Mutex::new(LegacyRandom::new(seed)))
    }

    /// Locks the shared random for a run of draws.
    pub fn lock(&self) -> std::sync::MutexGuard<'_, LegacyRandom> {
        lock_status_mutex(&self.0)
    }
}

/// The game rules the server tick thread consumes each tick.
pub(super) struct TickGameRules {
    /// `GameRules.ADVANCE_TIME`.
    pub advance_time: bool,
    /// `GameRules.ADVANCE_WEATHER`.
    pub advance_weather: bool,
    /// `GameRules.RANDOM_TICK_SPEED`.
    pub random_tick_speed: i32,
    /// `GameRules.FIRE_SPREAD_RADIUS_AROUND_PLAYER`.
    pub fire_spread_radius: i32,
    /// `GameRules.SPREAD_VINES`.
    pub spread_vines: bool,
    /// Game rules of `ServerLevel.explode` / `TntBlock`.
    pub explosion_rules: ExplosionRules,
}

impl TickGameRules {
    pub fn read(rules: &crate::game_rules::LiveGameRules) -> Self {
        let int = |name: &str, default: i32| match rules.get(name) {
            Some(crate::game_rules::GameRuleValue::Int(value)) => value,
            _ => default,
        };
        Self {
            advance_time: rules.bool("advance_time"),
            advance_weather: rules.bool("advance_weather"),
            random_tick_speed: int("random_tick_speed", 0),
            fire_spread_radius: int("fire_spread_radius_around_player", 128),
            spread_vines: rules.bool("spread_vines"),
            explosion_rules: ExplosionRules::read(rules),
        }
    }
}

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
    /// Java `GameRules.SPREAD_VINES`.
    pub spread_vines: bool,
    /// Weather/difficulty/game-rule inputs of `FireBlock.tick`.
    pub fire: FireEnvironment,
    /// Explosion-related game rules (`ServerLevel.explode`, `TntBlock`).
    pub explosion_rules: ExplosionRules,
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
    let mut directed = DirectedPackets::default();
    // Only a finite `fire_spread_radius_around_player` needs the positions.
    let fire_players = if world.fire.spread_radius == -1 {
        Vec::new()
    } else {
        fire_spread_players(world.bus)
    };
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
            &fire_players,
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
            fluid_ticks: &mut *fluid,
            block_ticks: &mut *block,
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
        // `PrimedTnt.tick` and the explosions it triggers.
        tick_primed_tnts(
            &mut frames,
            &ExplosionEnv {
                layout: world.layout,
                seed: world.seed,
                cache: world.cache,
                world_items: world.world_items,
                bus: world.bus,
                rules: world.explosion_rules,
                game_time: world.game_time,
                max_chained_neighbor_updates: world.max_chained_neighbor_updates,
            },
            &mut ExplosionSinks {
                block,
                fluid,
                packets: &mut directed,
            },
        )?;
    }
    world.bus.publish_frames(&frames)?;
    directed.publish(world.bus);
    Ok(())
}

/// Java `LeavesBlock.randomTick`, the one `randomTick` that ends in a
/// `dropResources` + `removeBlock` outcome instead of block-state writes. Every
/// other ported override lives in [`crate::random_tick_behaviors`].
fn leaves_random_tick_outcome(state: &BlockStateModel) -> Option<BlockTickOutcome> {
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
/// `tickPrecipitation` roll consumes its RNG (and advances `randValue`) in
/// Java order, but the snow/ice/cauldron effect itself is not ported.
/// TODO(random-tick-sections): Java skips sections without a randomly ticking
/// state (`LevelChunkSection.isRandomlyTicking`); every section draws here.
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
    let mut level_random = world.cache.level_random.lock();
    let mut changed = Vec::new();
    for (chunk_pos, chunk) in &chunks {
        let (min_x, min_z) = (chunk_pos.x * 16, chunk_pos.z * 16);
        for _ in 0..world.random_tick_speed {
            if level_random.next_i32_bound(48) == 0 {
                // tickPrecipitation(getBlockRandomPos(minX, 0, minZ, 15))
                let _precipitation_pos = random.block_random_pos(min_x, 0, min_z, 15);
            }
        }
        for section in &chunk.sections {
            let min_y = i32::from(section.y) * 16;
            for _ in 0..world.random_tick_speed {
                let pos = random.block_random_pos(min_x, min_y, min_z, 15);
                tick_random_position(
                    writer,
                    &mut level_random,
                    block_ticks,
                    pos,
                    world,
                    &mut changed,
                )?;
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

/// Java `blockState.randomTick(level, pos, level.random)` for one picked
/// position, when its state `isRandomlyTicking`.
fn tick_random_position<W: io::Write>(
    writer: &mut W,
    level_random: &mut LegacyRandom,
    block_ticks: &mut LiveBlockTicks,
    pos: BlockPos,
    world: &ServerWorldTick<'_>,
    changed: &mut Vec<BlockPos>,
) -> io::Result<()> {
    let live_world = LiveBlockWorld {
        layout: world.layout,
        seed: world.seed,
        cache: world.cache,
    };
    // Read the live state: earlier ticks of this pass may have changed it.
    let state = live_world.state_at(pos);
    let randomly_ticking = crate::block_properties::state_physics_by_name(&state.state_name())
        .is_some_and(|physics| physics.is_randomly_ticking);
    if !randomly_ticking {
        return Ok(());
    }
    if let Some(outcome) = leaves_random_tick_outcome(&state) {
        return apply_block_tick_outcome(
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
            changed,
        );
    }
    let mut tick_world = LiveRandomTickWorld::new(
        live_world,
        writer,
        CompressionState::disabled(),
        RandomTickEnvironment {
            raining: world.fire.raining,
            spread_vines: world.spread_vines,
        },
    );
    crate::random_tick_behaviors::random_tick(&mut tick_world, &state, pos, level_random);
    changed.append(&mut tick_world.changed);
    tick_world.error.map_or(Ok(()), Err)
}
