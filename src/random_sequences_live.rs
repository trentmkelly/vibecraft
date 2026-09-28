//! Server-owned `random_sequences.dat` saved data.
//!
//! Java `MinecraftServer` loads `RandomSequences` from the overworld
//! `SavedDataStorage` at startup, hands it out through `getRandomSequences()` /
//! `getRandomSequence(key)`, and writes it back on save when dirty. This module
//! owns the process-wide equivalent: [`initialize`] loads it at server start,
//! [`handle`] gives loot evaluation the shared instance, and [`save_if_dirty`]
//! persists it (autosave / shutdown).

use std::io;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::random_sequences::{RandomSequences, SharedRandomSequences};
use crate::storage::nbt::Tag;
use crate::storage::world::WorldLayout;

/// Live saved-data binding: where it persists and the seed sequences derive from.
struct LiveRandomSequences {
    layout: WorldLayout,
    world_seed: i64,
    sequences: SharedRandomSequences,
}

fn slot() -> &'static Mutex<Option<LiveRandomSequences>> {
    static SLOT: OnceLock<Mutex<Option<LiveRandomSequences>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn lock_slot() -> MutexGuard<'static, Option<LiveRandomSequences>> {
    slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Loads `data/random_sequences.dat` (missing file means a fresh, empty
/// `RandomSequences`, as with Java `SavedDataType` constructors) and installs it.
pub fn initialize(layout: WorldLayout, world_seed: i64) -> io::Result<()> {
    let sequences = load(&layout)?;
    *lock_slot() = Some(LiveRandomSequences {
        layout,
        world_seed,
        sequences: Arc::new(Mutex::new(sequences)),
    });
    Ok(())
}

/// The shared sequences plus world seed, once [`initialize`] has run.
pub fn handle() -> Option<(SharedRandomSequences, i64)> {
    lock_slot()
        .as_ref()
        .map(|live| (live.sequences.clone(), live.world_seed))
}

/// Writes the sequences if any draw or reset marked them dirty (Java
/// `SavedData.isDirty` gating in `SavedDataStorage.saveAndJoin`).
pub fn save_if_dirty() -> io::Result<()> {
    let guard = lock_slot();
    let Some(live) = guard.as_ref() else {
        return Ok(());
    };
    let mut sequences = live
        .sequences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !sequences.dirty {
        return Ok(());
    }
    live.layout.save_random_sequences(&saved_data_tag(&sequences))?;
    sequences.dirty = false;
    Ok(())
}

fn load(layout: &WorldLayout) -> io::Result<RandomSequences> {
    match layout.load_random_sequences() {
        Ok(tag) => Ok(parse_saved_data(&tag).unwrap_or_default()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(RandomSequences::default()),
        Err(err) => Err(err),
    }
}

/// `SavedDataStorage` stores the codec output under `data` next to `DataVersion`.
fn saved_data_tag(sequences: &RandomSequences) -> Tag {
    Tag::Compound(vec![("data".to_string(), sequences.to_tag())])
}

fn parse_saved_data(tag: &Tag) -> Option<RandomSequences> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    let (_, data) = fields.iter().find(|(name, _)| name == "data")?;
    RandomSequences::from_tag(data)
}

#[cfg(test)]
mod tests;
