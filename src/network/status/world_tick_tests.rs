//! Tests for the shared server-level world tick ([`super::tick_server_world`]).

use super::fire_live::FireEnvironment;
use super::*;
use crate::block_update::BlockPos;
use crate::random_tick::LevelRandom;

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "vibecraft-world-tick-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

struct Harness {
    layout: WorldLayout,
    cache: GeneratedChunkCache,
    items: Arc<Mutex<WorldItemEntities>>,
    bus: WorldPacketBus,
    ticks: SharedWorldTicks,
    random: LevelRandom,
}

impl Harness {
    fn new(name: &str) -> Self {
        Self {
            layout: WorldLayout::new(temp_root(name)),
            cache: GeneratedChunkCache::default(),
            items: Arc::new(Mutex::new(WorldItemEntities::default())),
            bus: WorldPacketBus::default(),
            ticks: Arc::new(Mutex::new(WorldTicks::new(0))),
            random: LevelRandom::new(7),
        }
    }

    fn tick(&mut self, game_time: i64, random_tick_speed: i32) {
        tick_server_world(
            &self.ticks,
            &mut self.random,
            &ServerWorldTick {
                game_time,
                layout: &self.layout,
                seed: 42,
                cache: &self.cache,
                world_items: &self.items,
                max_chained_neighbor_updates: 1_000_000,
                random_tick_speed,
                spread_vines: true,
                fire: FireEnvironment {
                    raining: false,
                    difficulty_id: 2,
                    spread_radius: -1,
                },
                explosion_rules: crate::server_explosion::ExplosionRules::default(),
                bus: &self.bus,
            },
        )
        .unwrap();
    }

    fn block(&self, pos: BlockPos) -> String {
        read_live_block_model_at(&self.cache, &self.layout, 42, pos).registry_id
    }
}

/// A scheduled block tick fires from the server tick alone (no player
/// session exists), exactly once, and every subscriber gets the update.
#[test]
fn scheduled_block_tick_runs_once_and_broadcasts_to_all_sessions() {
    let mut world = Harness::new("broadcast");
    let stone = BlockPos { x: 1, y: 64, z: 1 };
    let cane = BlockPos { x: 1, y: 65, z: 1 };
    world.cache.set_block(world.layout.root(), 42, stone, "minecraft:stone");
    world.cache.set_block(world.layout.root(), 42, cane, "minecraft:sugar_cane");
    let first = world.bus.subscribe(1);
    let second = world.bus.subscribe(2);

    lock_status_mutex(&world.ticks)
        .block
        .schedule(10, cane, "minecraft:sugar_cane", 5);

    // Not due before game time 15.
    world.tick(14, 0);
    assert_eq!(world.block(cane), "minecraft:sugar_cane");
    let mut sink = Vec::new();
    assert_eq!(first.drain_into(&mut sink, CompressionState::disabled()).unwrap(), 0);

    world.tick(15, 0);
    assert_eq!(world.block(cane), "minecraft:air");
    for subscriber in [&first, &second] {
        let mut out = Vec::new();
        assert!(subscriber.drain_into(&mut out, CompressionState::disabled()).unwrap() >= 1);
    }

    // The queue entry was consumed: a later tick changes nothing.
    world.tick(16, 0);
    assert_eq!(lock_status_mutex(&world.ticks).block.queues.count(), 0);
}

/// The shared queue is keyed to the server game time the tick thread
/// publishes, so sessions scheduling later see a consistent clock.
#[test]
fn tick_publishes_game_time_into_shared_queue() {
    let mut world = Harness::new("clock");
    world.tick(123, 0);
    assert_eq!(lock_status_mutex(&world.ticks).game_time, 123);
}

/// `LeavesBlock.randomTick`: only decaying (non-persistent, distance 7)
/// leaves are removed by the random-tick pass, persistent ones survive.
#[test]
fn random_tick_pass_decays_only_decaying_leaves() {
    let mut world = Harness::new("leaves");
    let decaying = "minecraft:oak_leaves[distance=7,persistent=false,waterlogged=false]";
    let persistent = "minecraft:oak_leaves[distance=7,persistent=true,waterlogged=false]";
    let mut positions = Vec::new();
    for y in 200..216 {
        for z in 0..16 {
            for x in 0..16 {
                let pos = BlockPos { x, y, z };
                let name = if x < 8 { decaying } else { persistent };
                world.cache.set_block(world.layout.root(), 42, pos, name);
                positions.push(pos);
            }
        }
    }
    for game_time in 1..=20 {
        world.tick(game_time, 3);
    }
    let decayed = positions
        .iter()
        .filter(|pos| pos.x < 8 && world.block(**pos) == "minecraft:air")
        .count();
    assert!(decayed > 0, "no decaying leaf was random ticked");
    assert!(positions
        .iter()
        .filter(|pos| pos.x >= 8)
        .all(|pos| world.block(*pos) == "minecraft:oak_leaves"));
}

/// `randomTickSpeed = 0` disables the pass entirely.
#[test]
fn zero_random_tick_speed_skips_random_ticks() {
    let mut world = Harness::new("speed-zero");
    let pos = BlockPos { x: 3, y: 210, z: 3 };
    let decaying = "minecraft:oak_leaves[distance=7,persistent=false,waterlogged=false]";
    world.cache.set_block(world.layout.root(), 42, pos, decaying);
    for game_time in 1..=50 {
        world.tick(game_time, 0);
    }
    assert_eq!(world.block(pos), "minecraft:oak_leaves");
}

/// The random-tick pass runs the ported plant behaviors: nether wart ages
/// (`NetherWartBlock.randomTick`, one in ten per pick, capped at age 3) and
/// the changes are broadcast to subscribed sessions.
#[test]
fn random_tick_pass_grows_nether_wart_and_broadcasts_it() {
    let mut world = Harness::new("nether-wart");
    world.cache.level_random.lock().set_seed(99);
    let wart = "minecraft:nether_wart[age=0]";
    let mut positions = Vec::new();
    // A full section, so every pick of the section lands on a wart.
    for y in 192..208 {
        for z in 0..16 {
            for x in 0..16 {
                let pos = BlockPos { x, y, z };
                world.cache.set_block(world.layout.root(), 42, pos, wart);
                positions.push(pos);
            }
        }
    }
    let subscriber = world.bus.subscribe(1);
    for game_time in 1..=60 {
        world.tick(game_time, 3);
    }
    let ages: Vec<i32> = positions
        .iter()
        .map(|pos| {
            read_live_block_model_at(&world.cache, &world.layout, 42, *pos)
                .property("age")
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    assert!(ages.iter().any(|age| *age > 0), "no wart was random ticked");
    assert!(ages.iter().all(|age| *age <= 3), "nether wart ages past 3");
    let mut out = Vec::new();
    assert!(subscriber.drain_into(&mut out, CompressionState::disabled()).unwrap() >= 1);
}
