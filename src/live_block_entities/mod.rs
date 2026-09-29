//! Live block-entity ticking for the running server.
//!
//! Java keeps every loaded `BlockEntity` on its `LevelChunk` and ticks them from
//! `Level.tickBlockEntities` through the `TickingBlockEntity` /
//! `BlockEntityTicker` pair created in `LevelChunk.updateBlockEntityTicker`.
//! VibeCraft's authoritative per-chunk block-entity store is the
//! `LevelChunk.block_entities` NBT list held by
//! [`GeneratedChunkCache`](crate::network::status::GeneratedChunkCache): it is
//! populated from chunk NBT on load, saved with the chunk, and is also the
//! container the block menus read and write. The ticker therefore runs
//! *against that store*: each server tick it walks the cached chunks, decodes
//! every tickable block entity into its typed model, runs the type's
//! `BlockEntityTicker`, and writes the new state back only when it changed.
//!
//! * [`lifecycle`] — `LevelChunk.setBlockState` block-entity create/remove.
//! * [`furnace`] — `AbstractFurnaceBlockEntity.serverTick` for the furnace,
//!   blast furnace and smoker.
//! * [`hopper`] — `HopperBlockEntity.pushItemsTick`, which needs the whole
//!   chunk map (neighbouring containers) and the world's item entities, so it
//!   runs as a world-level pass ([`LiveBlockEntityTicker::tick`]) instead of a
//!   [`TickerFn`]; [`container`] is the NBT-backed `Container` it works on.
//! * [`openers`] / [`open_effects`] — `ContainerOpenersCounter` for chests,
//!   trapped chests, barrels and shulker boxes, driven by menu open/close.
//!
//! Adding another ticking type means adding a [`TickerFn`] arm to
//! [`ticker_for`]; the surrounding walk, dirty tracking and block-state
//! broadcast are shared.
//!
//! TODO(block-entity-ticking-range): Java only ticks block entities in
//! entity-ticking chunks (near a player). The chunk cache does not track which
//! sessions have a chunk loaded, so every cached chunk is ticked.

pub mod container;
mod entity_broadcast;
pub mod experience;
pub mod furnace;
pub mod hopper;
pub mod lifecycle;
pub mod open_effects;
pub mod openers;
pub mod registry_ids;
pub mod stack;
#[cfg(test)]
mod hopper_tests;
#[cfg(test)]
mod openers_tests;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::block_entity::BlockEntityTypeId;
use crate::block_update::BlockPos;
use crate::network::compression::CompressionState;
use crate::network::play::{block_state_name_network_id, CLIENTBOUND_BLOCK_UPDATE_PACKET_ID};
use crate::network::status::{
    block_pos_as_long, write_framed_packet_with_compression, GeneratedChunkCache,
};
use crate::network::varint::write_var_i32;
use crate::network::world_broadcast::WorldPacketBus;
use crate::item_entity::WorldItemEntities;
use crate::recipe_system::{FuelValues, RecipeManagerModel, RecipeMap};
use crate::storage::chunk::{BlockStateEntry, LevelChunk};
use crate::storage::nbt::Tag;
use crate::storage::region::ChunkPos;

/// World data a `BlockEntityTicker` may read (Java `ServerLevel.recipeAccess()`
/// and `ServerLevel.fuelValues()`).
pub struct BlockEntityTickEnvironment<'a> {
    pub recipes: &'a RecipeMap,
    pub fuel_values: &'a FuelValues,
}

/// What one `BlockEntityTicker` invocation produced.
#[derive(Debug, Clone, PartialEq)]
pub struct TickOutput {
    /// The block entity's full NBT after the tick (`saveWithId`-shaped).
    pub tag: Tag,
    /// The block state the ticker wrote with `level.setBlock(pos, state, 3)`.
    pub new_block_state: Option<String>,
}

/// A `BlockEntityTicker` over the NBT-backed block entity. Returns `None` when
/// the tick left the block entity and its block untouched.
pub type TickerFn = fn(
    tag: &Tag,
    state: &BlockStateEntry,
    env: &BlockEntityTickEnvironment<'_>,
) -> Option<TickOutput>;

/// `BlockEntityType.getTicker` for the types that are ticked live.
pub fn ticker_for(ty: BlockEntityTypeId) -> Option<TickerFn> {
    match ty {
        BlockEntityTypeId::Furnace
        | BlockEntityTypeId::BlastFurnace
        | BlockEntityTypeId::Smoker => Some(furnace::tick_furnace_family),
        _ => None,
    }
}

/// A block-state write made by a ticker, to be applied and broadcast once the
/// chunk map lock is released.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateChange {
    pub pos: BlockPos,
    pub state: String,
}

/// Formats a block state as `name[prop=value,...]`, the form accepted by
/// `GeneratedChunkCache::set_block`.
pub fn format_block_state(entry: &BlockStateEntry) -> String {
    if entry.properties.is_empty() {
        return entry.name.clone();
    }
    let properties: Vec<String> = entry
        .properties
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    format!("{}[{}]", entry.name, properties.join(","))
}

/// `Level.tickBlockEntities` over every cached chunk: ticks each block entity
/// whose type has a [`ticker_for`], persists changed block entities into their
/// chunk and returns the block-state writes the tickers requested.
///
/// Chunks are only cloned-on-write (`Arc::make_mut`) when a block entity really
/// changed, so idle furnaces cost a decode and a comparison.
pub fn tick_cached_block_entities(
    cache: &GeneratedChunkCache,
    env: &BlockEntityTickEnvironment<'_>,
) -> Vec<BlockStateChange> {
    let mut changes = Vec::new();
    let mut dirtied: Vec<ChunkPos> = Vec::new();
    {
        let mut chunks = cache.chunks.lock().unwrap_or_else(|e| e.into_inner());
        for (chunk_pos, chunk) in chunks.iter_mut() {
            let updates = collect_chunk_updates(chunk, env);
            if updates.is_empty() {
                continue;
            }
            let chunk = Arc::make_mut(chunk);
            for (pos, output) in updates {
                chunk.set_block_entity_nbt(output.tag);
                if let Some(state) = output.new_block_state {
                    changes.push(BlockStateChange { pos, state });
                }
            }
            dirtied.push(*chunk_pos);
        }
    }
    if !dirtied.is_empty() {
        cache
            .dirty
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(dirtied);
    }
    changes
}

/// Runs the tickers of one chunk against a read-only view.
fn collect_chunk_updates(
    chunk: &LevelChunk,
    env: &BlockEntityTickEnvironment<'_>,
) -> Vec<(BlockPos, TickOutput)> {
    let mut updates = Vec::new();
    for tag in &chunk.block_entities {
        let Some((pos, ty)) = lifecycle::block_entity_identity(tag) else {
            continue;
        };
        let Some(ticker) = ticker_for(ty) else {
            continue;
        };
        // `BoundTickingBlockEntity.tick`: a block entity whose block no longer
        // accepts its type is skipped (removal happens in `setBlockState`).
        let Some(state) = chunk.get_block_state_model(pos.x, pos.y, pos.z) else {
            continue;
        };
        if !lifecycle::type_accepts_block(ty, &state.name) {
            continue;
        }
        if let Some(output) = ticker(tag, &state, env) {
            updates.push((pos, output));
        }
    }
    updates
}

/// Applies the ticker-requested block-state writes (`level.setBlock(pos, state,
/// 3)`) and broadcasts each as a `ClientboundBlockUpdatePacket` to every
/// connected player through the world packet bus.
pub fn apply_block_state_changes(
    cache: &GeneratedChunkCache,
    world_root: &Path,
    world_seed: i64,
    bus: &WorldPacketBus,
    changes: &[BlockStateChange],
) {
    for change in changes {
        cache.set_block(world_root, world_seed, change.pos, &change.state);
        let Some(state_id) = block_state_name_network_id(&change.state) else {
            continue;
        };
        let mut frame = Vec::new();
        let written = write_framed_packet_with_compression(
            &mut frame,
            CompressionState::disabled(),
            CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
            |payload| {
                let packed = block_pos_as_long(change.pos.x, change.pos.y, change.pos.z);
                payload.extend_from_slice(&packed.to_be_bytes());
                write_var_i32(payload, state_id)
            },
        );
        if written.is_ok() {
            // Frames are `VarInt length` + plain payload, the bus' input shape.
            let _ = bus.publish_frames(&frame);
        }
    }
}

/// The server's block-entity ticker: owns the shared world handles and the
/// production [`FuelValues`], and runs one `Level.tickBlockEntities` per call to
/// [`tick`](Self::tick) from the server tick thread.
pub struct LiveBlockEntityTicker {
    cache: GeneratedChunkCache,
    recipes: Arc<RecipeManagerModel>,
    world_root: Arc<PathBuf>,
    world_seed: i64,
    bus: WorldPacketBus,
    world_items: Arc<Mutex<WorldItemEntities>>,
    fuel_values: &'static FuelValues,
    /// `Level.getGameTime()` as seen by this ticker: ticks run so far.
    game_time: i64,
    hoppers: hopper::HopperTicker,
    open_counters: open_effects::OpenCounters,
}

impl LiveBlockEntityTicker {
    pub fn new(
        cache: GeneratedChunkCache,
        recipes: Arc<RecipeManagerModel>,
        world_root: Arc<PathBuf>,
        world_seed: i64,
        bus: WorldPacketBus,
        world_items: Arc<Mutex<WorldItemEntities>>,
    ) -> Self {
        Self {
            cache,
            recipes,
            world_root,
            world_seed,
            bus,
            world_items,
            fuel_values: FuelValues::shared_vanilla(),
            game_time: 0,
            hoppers: hopper::HopperTicker::default(),
            open_counters: open_effects::OpenCounters::default(),
        }
    }

    /// Ticks every block entity once and broadcasts the resulting block changes.
    pub fn tick(&mut self) {
        self.game_time += 1;
        // A `/reload` swaps the server's recipe manager; furnaces and campfires must
        // cook with the recipes now in force.
        if let Some(live) = crate::registry_pipeline::server_resources::installed_recipe_manager() {
            self.recipes = live;
        }
        let env = BlockEntityTickEnvironment {
            recipes: self.recipes.recipe_map(),
            fuel_values: self.fuel_values,
        };
        let mut changes = tick_cached_block_entities(&self.cache, &env);
        changes.extend(tick_world_level(
            WorldLevelParts {
                cache: &self.cache,
                world_items: &self.world_items,
                bus: &self.bus,
                hoppers: &mut self.hoppers,
                open_counters: &mut self.open_counters,
            },
            &env,
            self.game_time,
        ));
        apply_block_state_changes(
            &self.cache,
            &self.world_root,
            self.world_seed,
            &self.bus,
            &changes,
        );
    }
}

/// The ticker state the world-level pass touches.
struct WorldLevelParts<'a> {
    cache: &'a GeneratedChunkCache,
    world_items: &'a Mutex<WorldItemEntities>,
    bus: &'a WorldPacketBus,
    hoppers: &'a mut hopper::HopperTicker,
    open_counters: &'a mut open_effects::OpenCounters,
}

/// The block-entity tickers that need the whole chunk map: menu open/close
/// counters and hoppers. Returns the block-state writes they requested.
fn tick_world_level(
    parts: WorldLevelParts<'_>,
    env: &BlockEntityTickEnvironment<'_>,
    game_time: i64,
) -> Vec<BlockStateChange> {
    // Lock order: item entities, then chunks (no path nests them the other way
    // round).
    let mut items = parts.world_items.lock().unwrap_or_else(|e| e.into_inner());
    let mut chunks = parts.cache.chunks.lock().unwrap_or_else(|e| e.into_inner());
    let mut world = container::ChunkWorld::new(&mut chunks);
    let opened = parts
        .open_counters
        .process(&world, &parts.cache.container_openers, game_time);
    let effects = parts.hoppers.tick(&mut world, &mut items, env, game_time);
    let dirtied = world.into_dirtied();
    drop(chunks);
    let orb_frames = spawn_awarded_experience(&mut items, &parts.cache.experience_awards);
    drop(items);
    if !dirtied.is_empty() {
        parts
            .cache
            .dirty
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(dirtied);
    }
    let _ = parts.bus.publish_frames(&opened.frames);
    let _ = parts.bus.publish_frames(&orb_frames);
    let _ = entity_broadcast::publish_item_effects(parts.bus, &effects);
    opened.state_changes
}

/// `ExperienceOrb.award` for every recorded award; returns the spawn packets.
fn spawn_awarded_experience(
    items: &mut WorldItemEntities,
    awards: &experience::ExperienceAwards,
) -> Vec<u8> {
    use crate::network::status::xp_orb_live::write_xp_orb_spawn_packets;
    use crate::xp_orb_entity::XpOrbRandom;

    let mut frames = Vec::new();
    for award in awards.drain() {
        let mut random = XpOrbRandom::new(rand::random());
        for orb in items.award_experience(award.pos, award.amount, &mut random) {
            let _ = write_xp_orb_spawn_packets(&mut frames, CompressionState::disabled(), &orb);
        }
    }
    frames
}
