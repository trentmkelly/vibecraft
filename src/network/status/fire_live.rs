//! Live wiring of [`crate::fire_block`]: the scheduled `FireBlock.tick` run by
//! the shared world tick, and the `FireBlock.onPlace` scheduling for fire
//! placed by players.
//!
//! [`LiveFireWorld`] adapts the generated-chunk cache to the
//! [`FireWorld`](crate::fire_block::FireWorld) abstraction: every
//! `Level.setBlock` / `removeBlock` the Java tick performs mutates the cache,
//! is written as a block update on the shared frame buffer when the flags ask
//! for a client update (flag 2), and is queued for the `updateNeighborShapes`
//! cascade when they ask for a neighbour update (flag 1).

use std::io::{self, Write};

use super::block_placement_live::{write_block_update, LiveBlockWorld};
use super::*;
use crate::block_behavior::BlockStateModel;
use crate::block_placement::PlacementWorld;
use crate::block_survival::SurvivalWorld;
use crate::block_update::{BlockPos, Direction};
use crate::fire_block::{fire_tick, fire_tick_delay, FireRandom, FireWorld};

/// `Level.setBlock` flag bits (`Block.UPDATE_*`).
const UPDATE_NEIGHBORS: i32 = 1;
const UPDATE_CLIENTS: i32 = 2;
const UPDATE_SKIP_ON_PLACE: i32 = 256;

/// Server state `FireBlock.tick` reads that is not stored in the chunks.
#[derive(Clone, Copy, Debug)]
pub struct FireEnvironment {
    /// `Level.isRaining()`.
    pub raining: bool,
    /// `Difficulty.getId()`.
    pub difficulty_id: i32,
    /// `GameRules.FIRE_SPREAD_RADIUS_AROUND_PLAYER`.
    pub spread_radius: i32,
}

/// The players `ChunkMap.playerIsCloseEnoughTo` counts for fire spread:
/// everyone who is not a spectator.
pub(super) fn fire_spread_players(bus: &WorldPacketBus) -> Vec<[f64; 3]> {
    bus.players()
        .into_iter()
        .filter(|(_, presence)| !presence.spectator)
        .map(|(_, presence)| presence.position)
        .collect()
}

/// `RandomSource` backed by an xorshift generator seeded per use.
pub struct LiveRandom(u64);

impl LiveRandom {
    /// Seeds from the wall clock mixed with `salt` (Java `Level.random` is
    /// unseeded, so only distribution matters).
    pub fn new(salt: u64) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as u64);
        Self((nanos ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15)) | 1)
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
}

impl FireRandom for LiveRandom {
    fn next_int(&mut self, bound: i32) -> i32 {
        (self.next_u32() % bound.max(1) as u32) as i32
    }

    fn next_float(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1 << 24) as f32
    }
}

struct LiveFireWorld<'a, 'b, W: Write> {
    inner: LiveBlockWorld<'a>,
    writer: &'b mut W,
    compression: CompressionState,
    block_ticks: &'b mut LiveBlockTicks,
    game_time: i64,
    environment: FireEnvironment,
    world_items: &'b Arc<Mutex<WorldItemEntities>>,
    /// Positions of the non-spectator players (`anyPlayerCloseEnoughTo`).
    players: &'b [[f64; 3]],
    random: LiveRandom,
    /// Positions whose neighbours need the `updateShape` cascade.
    changed: Vec<BlockPos>,
    /// First write error (the trait methods cannot fail).
    error: Option<io::Error>,
}

impl<W: Write> LiveFireWorld<'_, '_, W> {
    fn write_update(&mut self, pos: BlockPos, state: &BlockStateModel) {
        let id = crate::block_states::network_id_for_block_state(&state.state_name()).unwrap_or(0);
        if let Err(err) = write_block_update(self.writer, self.compression, pos, id) {
            self.error.get_or_insert(err);
        }
    }
}

impl<W: Write> SurvivalWorld for LiveFireWorld<'_, '_, W> {
    fn state_at(&self, pos: BlockPos) -> BlockStateModel {
        self.inner.state_at(pos)
    }

    fn raw_brightness(&self, pos: BlockPos) -> i32 {
        self.inner.raw_brightness(pos)
    }
}

impl<W: Write> PlacementWorld for LiveFireWorld<'_, '_, W> {
    fn has_neighbor_signal(&self, pos: BlockPos) -> bool {
        self.inner.has_neighbor_signal(pos)
    }
}

impl<W: Write> FireWorld for LiveFireWorld<'_, '_, W> {
    fn can_spread_fire_around(&self, pos: BlockPos) -> bool {
        // `ServerLevel.canSpreadFireAround`: -1 is unlimited, else
        // `ChunkMap.anyPlayerCloseEnoughTo(pos, radius)`.
        let radius = self.environment.spread_radius;
        radius == -1
            || self.players.iter().any(|player| {
                let d = |axis: usize, block: i32| player[axis] - f64::from(block);
                (d(0, pos.x).powi(2) + d(1, pos.y).powi(2) + d(2, pos.z).powi(2)).sqrt()
                    < f64::from(radius)
            })
    }

    fn is_raining(&self) -> bool {
        self.environment.raining
    }

    fn is_raining_at(&self, pos: BlockPos) -> bool {
        super::random_tick_live::is_raining_at(&self.inner, self.environment.raining, pos)
    }

    fn is_infiniburn(&self, state: &BlockStateModel) -> bool {
        crate::block_tags::block_tag_contains("infiniburn_overworld", &state.registry_id)
    }

    fn increased_fire_burnout(&self, _pos: BlockPos) -> bool {
        // TODO(environment-attributes-live): `gameplay/increased_fire_burnout`
        // is biome-driven; the vanilla default is false.
        false
    }

    fn difficulty_id(&self) -> i32 {
        self.environment.difficulty_id
    }

    fn is_face_sturdy_up(&self, pos: BlockPos) -> bool {
        crate::block_placement::attached::face_sturdy_at(&self.inner, pos, Direction::Up)
    }

    fn set_block(&mut self, pos: BlockPos, state: BlockStateModel, flags: i32) {
        let old = self.state_at(pos);
        self.inner
            .cache
            .set_block(self.inner.layout.root(), self.inner.seed, pos, &state.state_name());
        if flags & UPDATE_CLIENTS != 0 {
            self.write_update(pos, &state);
        }
        if flags & UPDATE_NEIGHBORS != 0 {
            self.changed.push(pos);
        }
        // `FireBlock.onPlace`, skipped by UPDATE_SKIP_ON_PLACE and by an
        // unchanged block type.
        if flags & UPDATE_SKIP_ON_PLACE == 0
            && state.registry_id == "minecraft:fire"
            && old.registry_id != state.registry_id
        {
            let delay = fire_tick_delay(&mut self.random);
            self.schedule_fire_tick(pos, delay);
        }
    }

    fn remove_block(&mut self, pos: BlockPos) {
        self.inner
            .cache
            .set_block(self.inner.layout.root(), self.inner.seed, pos, "minecraft:air");
        self.write_update(pos, &BlockStateModel::air());
        self.changed.push(pos);
    }

    fn prime_tnt(&mut self, pos: BlockPos) {
        // `TntBlock.prime(level, pos)`; the burnt block was already removed by
        // `checkBurnOut`, so the result (`tnt_explodes`) only gates the entity.
        super::tnt_live::prime_tnt(
            self.world_items,
            &self.inner.cache.level_random,
            pos,
            None,
        );
    }

    fn schedule_fire_tick(&mut self, pos: BlockPos, delay: i32) {
        self.block_ticks
            .schedule(self.game_time, pos, "minecraft:fire", delay);
    }
}

/// Runs `FireBlock.tick` for the fire at `pos` and returns the positions whose
/// neighbours need the shape-update cascade.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_live_fire_tick<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    world: LiveBlockWorld<'_>,
    block_ticks: &mut LiveBlockTicks,
    game_time: i64,
    environment: FireEnvironment,
    world_items: &Arc<Mutex<WorldItemEntities>>,
    players: &[[f64; 3]],
    state: BlockStateModel,
    pos: BlockPos,
) -> io::Result<Vec<BlockPos>> {
    let salt = (game_time as u64) ^ (pos.x as u64).rotate_left(21) ^ (pos.z as u64).rotate_left(42);
    let mut fire_world = LiveFireWorld {
        inner: world,
        writer,
        compression,
        block_ticks,
        game_time,
        environment,
        world_items,
        players,
        random: LiveRandom::new(salt),
        changed: Vec::new(),
        error: None,
    };
    // `FireBlock.tick`'s `random` argument and the level RNG used by `onPlace`
    // are separate streams.
    let mut tick_random = LiveRandom::new(salt.rotate_left(7));
    fire_tick(&mut fire_world, state, pos, &mut tick_random);
    match fire_world.error {
        Some(err) => Err(err),
        None => Ok(fire_world.changed),
    }
}

/// `FireBlock.onPlace` for fire placed outside a tick: schedules the first
/// spread tick `30 + nextInt(10)` game ticks out.
pub(super) fn schedule_placed_fire(
    block_ticks: &mut LiveBlockTicks,
    game_time: i64,
    pos: BlockPos,
) {
    let mut random = LiveRandom::new((game_time as u64) ^ (pos.x as u64) ^ ((pos.z as u64) << 32));
    block_ticks.schedule(game_time, pos, "minecraft:fire", fire_tick_delay(&mut random));
}
