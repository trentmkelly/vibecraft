//! Tests for the container openers registry and its chest, barrel and shulker
//! box effects. Java references: `ContainerOpenersCounter`,
//! `ChestBlockEntity` / `BarrelBlockEntity` / `ShulkerBoxBlockEntity`
//! `startOpen` / `stopOpen` / `recheckOpen`.

use std::sync::Arc;

use super::container::ChunkWorld;
use super::lifecycle::sync_block_entity_after_set_block;
use super::open_effects::{OpenCounters, OpenOutcome};
use super::openers::ContainerOpeners;
use super::*;
use crate::network::play::{
    CLIENTBOUND_BLOCK_EVENT_PACKET_ID, CLIENTBOUND_SOUND_PACKET_ID,
};
use crate::network::varint::read_var_i32;
use crate::storage::chunk::LevelChunk;

const CHEST: BlockPos = BlockPos { x: 3, y: 64, z: 5 };

struct Fixture {
    chunks: std::collections::HashMap<ChunkPos, Arc<LevelChunk>>,
    openers: Arc<ContainerOpeners>,
    counters: OpenCounters,
    game_time: i64,
}

impl Fixture {
    fn with_block(pos: BlockPos, state: &str) -> Self {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.set_block_state(pos.x, pos.y, pos.z, state);
        sync_block_entity_after_set_block(&mut chunk, pos, None, state);
        let mut chunks = std::collections::HashMap::new();
        chunks.insert(chunk.pos, Arc::new(chunk));
        Self {
            chunks,
            openers: Arc::default(),
            counters: OpenCounters::default(),
            game_time: 0,
        }
    }

    fn chest() -> Self {
        Self::with_block(
            CHEST,
            "minecraft:chest[facing=north,type=single,waterlogged=false]",
        )
    }

    fn tick(&mut self) -> OpenOutcome {
        self.game_time += 1;
        let world = ChunkWorld::new(&mut self.chunks);
        self.counters.process(&world, &self.openers, self.game_time)
    }
}

/// `(packet id, first payload byte after the id)` for every frame.
fn packets(frames: &[u8]) -> Vec<(i32, Vec<u8>)> {
    let mut cursor = frames;
    let mut out = Vec::new();
    while !cursor.is_empty() {
        let len = read_var_i32(&mut cursor).unwrap() as usize;
        let (payload, rest) = cursor.split_at(len);
        let mut payload_cursor = payload;
        let id = read_var_i32(&mut payload_cursor).unwrap();
        out.push((id, payload_cursor.to_vec()));
        cursor = rest;
    }
    out
}

/// `(action, param)` of the block-event packets, in order.
fn block_events(frames: &[u8]) -> Vec<(u8, u8)> {
    packets(frames)
        .into_iter()
        .filter(|(id, _)| *id == CLIENTBOUND_BLOCK_EVENT_PACKET_ID)
        // BlockPos (8 bytes) then action and param.
        .map(|(_, payload)| (payload[8], payload[9]))
        .collect()
}

fn sound_count(frames: &[u8]) -> usize {
    packets(frames)
        .iter()
        .filter(|(id, _)| *id == CLIENTBOUND_SOUND_PACKET_ID)
        .count()
}

#[test]
fn opening_a_chest_sends_the_lid_event_and_sound_and_closing_undoes_them() {
    let mut world = Fixture::chest();
    let guard = world.openers.start_open(CHEST, 4.5);
    let opened = world.tick();
    assert_eq!(block_events(&opened.frames), vec![(1, 1)]);
    assert_eq!(sound_count(&opened.frames), 1, "onOpen plays the chest sound");
    assert_eq!(world.counters.open_count(CHEST), 1);

    let second = world.openers.start_open(CHEST, 4.5);
    let joined = world.tick();
    assert_eq!(block_events(&joined.frames), vec![(1, 2)]);
    assert_eq!(sound_count(&joined.frames), 0, "only the first opener plays a sound");

    drop(guard);
    let one_left = world.tick();
    assert_eq!(block_events(&one_left.frames), vec![(1, 1)]);
    assert_eq!(sound_count(&one_left.frames), 0);

    drop(second);
    let closed = world.tick();
    assert_eq!(block_events(&closed.frames), vec![(1, 0)]);
    assert_eq!(sound_count(&closed.frames), 1, "onClose plays the close sound");
    assert_eq!(world.counters.open_count(CHEST), 0);
}

#[test]
fn a_chest_rechecks_its_openers_every_five_ticks_while_open() {
    let mut world = Fixture::chest();
    let _guard = world.openers.start_open(CHEST, 4.5);
    world.tick();
    for _ in 0..3 {
        assert!(block_events(&world.tick().frames).is_empty());
    }
    // Scheduled five ticks after the open (tick 1 -> due on tick 6).
    assert!(block_events(&world.tick().frames).is_empty(), "tick 5");
    assert_eq!(
        block_events(&world.tick().frames),
        vec![(1, 1)],
        "recheckOpen re-signals the unchanged count and reschedules"
    );
    for _ in 0..4 {
        world.tick();
    }
    assert_eq!(block_events(&world.tick().frames), vec![(1, 1)]);
}

#[test]
fn recheck_drops_openers_whose_menu_disappeared_without_a_stop() {
    let mut world = Fixture::chest();
    let guard = world.openers.start_open(CHEST, 4.5);
    world.tick();
    // The registry loses the user (as `hasContainerOpen` turning false would)
    // while the counter still counts it: forge the drift by forgetting the guard
    // without dropping it.
    std::mem::forget(guard);
    assert_eq!(world.counters.open_count(CHEST), 1);
    for _ in 0..4 {
        world.tick();
    }
    let recheck = world.tick();
    assert_eq!(block_events(&recheck.frames), vec![(1, 1)], "the leaked user is still open");
}

#[test]
fn the_left_half_of_a_double_chest_is_silent_and_the_right_half_offsets_the_sound() {
    let mut left = Fixture::with_block(
        CHEST,
        "minecraft:chest[facing=north,type=left,waterlogged=false]",
    );
    let _guard = left.openers.start_open(CHEST, 4.5);
    let outcome = left.tick();
    assert_eq!(sound_count(&outcome.frames), 0);
    assert_eq!(block_events(&outcome.frames), vec![(1, 1)], "the lid still animates");

    let mut right = Fixture::with_block(
        CHEST,
        "minecraft:chest[facing=north,type=right,waterlogged=false]",
    );
    let _guard = right.openers.start_open(CHEST, 4.5);
    assert_eq!(sound_count(&right.tick().frames), 1);
}

#[test]
fn barrels_toggle_their_open_state_and_play_a_sound() {
    let mut world = Fixture::with_block(CHEST, "minecraft:barrel[facing=up,open=false]");
    let guard = world.openers.start_open(CHEST, 4.5);
    let opened = world.tick();
    assert_eq!(sound_count(&opened.frames), 1);
    assert!(block_events(&opened.frames).is_empty(), "barrels send no block event");
    assert_eq!(
        opened.state_changes,
        vec![BlockStateChange {
            pos: CHEST,
            state: "minecraft:barrel[facing=up,open=true]".to_string()
        }]
    );
    drop(guard);
    let closed = world.tick();
    assert_eq!(
        closed.state_changes[0].state,
        "minecraft:barrel[facing=up,open=false]"
    );
    assert_eq!(sound_count(&closed.frames), 1);
}

#[test]
fn shulker_boxes_signal_every_count_change_and_play_sounds_at_the_ends() {
    let mut world = Fixture::with_block(CHEST, "minecraft:shulker_box[facing=up]");
    let first = world.openers.start_open(CHEST, 4.5);
    let second = world.openers.start_open(CHEST, 4.5);
    let opened = world.tick();
    assert_eq!(block_events(&opened.frames), vec![(1, 1), (1, 2)]);
    assert_eq!(sound_count(&opened.frames), 1, "only the first open plays a sound");
    drop(first);
    drop(second);
    let closed = world.tick();
    assert_eq!(block_events(&closed.frames), vec![(1, 1), (1, 0)]);
    assert_eq!(sound_count(&closed.frames), 1, "only the last close plays a sound");
}

#[test]
fn a_removed_block_entity_forgets_its_openers() {
    let mut world = Fixture::chest();
    let guard = world.openers.start_open(CHEST, 4.5);
    world.tick();
    world
        .chunks
        .values_mut()
        .for_each(|chunk| Arc::make_mut(chunk).block_entities.clear());
    drop(guard);
    let outcome = world.tick();
    assert!(outcome.frames.is_empty());
    assert_eq!(world.counters.open_count(CHEST), 0);
}

#[test]
fn only_containers_with_an_openers_counter_react() {
    let mut world = Fixture::with_block(
        CHEST,
        "minecraft:furnace[facing=north,lit=false]",
    );
    let _guard = world.openers.start_open(CHEST, 4.5);
    assert_eq!(world.tick(), OpenOutcome::default());
}

/// The full path a menu takes: a session registers through the cache's shared
/// registry, the production ticker processes it and the lid packet reaches the
/// world bus.
#[test]
fn the_live_ticker_broadcasts_lid_events_for_registered_menus() {
    let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    let state = "minecraft:chest[facing=north,type=single,waterlogged=false]";
    chunk.set_block_state(CHEST.x, CHEST.y, CHEST.z, state);
    sync_block_entity_after_set_block(&mut chunk, CHEST, None, state);
    let cache = GeneratedChunkCache::default();
    cache.chunks.lock().unwrap().insert(chunk.pos, Arc::new(chunk));
    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(1);
    let mut ticker = LiveBlockEntityTicker::new(
        cache.clone(),
        Arc::new(
            crate::recipe_system::load_recipe_directory(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("vanilla-data/data/minecraft/recipe"),
            )
            .expect("recipes"),
        ),
        Arc::new(std::env::temp_dir().join("vibecraft-live-openers")),
        0,
        bus,
        Arc::new(Mutex::new(WorldItemEntities::new())),
    );

    let guard = cache.container_openers.start_open(CHEST, 4.5);
    ticker.tick();

    let mut out = Vec::new();
    subscription
        .drain_into(&mut out, crate::network::compression::CompressionState::disabled())
        .unwrap();
    let mut cursor = &out[..];
    let mut ids = Vec::new();
    while !cursor.is_empty() {
        let len = read_var_i32(&mut cursor).unwrap() as usize;
        let (payload, rest) = cursor.split_at(len);
        ids.push(read_var_i32(&mut &payload[..]).unwrap());
        cursor = rest;
    }
    assert!(ids.contains(&CLIENTBOUND_BLOCK_EVENT_PACKET_ID), "{ids:?}");
    assert!(ids.contains(&CLIENTBOUND_SOUND_PACKET_ID), "{ids:?}");
    drop(guard);
}
