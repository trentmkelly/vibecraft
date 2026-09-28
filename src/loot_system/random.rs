//! Loot-context random source.
//!
//! Java `LootContext.Builder.create` picks the context's `RandomSource` as
//! `explicit random (seed) -> server.getRandomSequence(table.random_sequence) -> level.getRandom()`.
//! The chosen `RandomSource` is shared by every condition, entry, and function
//! evaluated with that context, so [`LootRandom`] is a cheap-clone shared handle
//! rather than a copied state.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::random_sequences::SharedRandomSequences;
use crate::random_source::{
    random_source_next_bool, random_source_next_i32, RandomAlgorithm, RandomSourceKind,
};

/// Backing generator of a [`LootRandom`].
enum LootRandomSource {
    /// `RandomSource.create(seed)` (a `LegacyRandomSource`).
    Legacy(RandomSourceKind),
    /// `RandomSequences.get(key, worldSeed)`: a persisted Xoroshiro sequence whose
    /// every draw marks the saved data dirty (`DirtyMarkingRandomSource`).
    Sequence {
        sequences: SharedRandomSequences,
        id: String,
        world_seed: i64,
    },
}

/// Shared handle to the `RandomSource` of one loot context.
#[derive(Clone)]
pub struct LootRandom {
    source: Arc<Mutex<LootRandomSource>>,
}

impl LootRandom {
    /// Java `RandomSource.create(seed)`.
    pub fn from_seed(seed: u64) -> Self {
        Self::wrap(LootRandomSource::Legacy(RandomSourceKind::new(
            seed as i64,
            RandomAlgorithm::Legacy,
        )))
    }

    /// Java `MinecraftServer.getRandomSequence(key)`: creates the sequence on
    /// first use (`computeIfAbsent`) and returns a dirty-marking view of it.
    pub fn from_sequence(sequences: SharedRandomSequences, id: &str, world_seed: i64) -> Self {
        let id = normalize_sequence_id(id);
        sequences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get_or_create(&id, world_seed);
        Self::wrap(LootRandomSource::Sequence {
            sequences,
            id,
            world_seed,
        })
    }

    fn wrap(source: LootRandomSource) -> Self {
        Self {
            source: Arc::new(Mutex::new(source)),
        }
    }

    fn lock(&self) -> MutexGuard<'_, LootRandomSource> {
        self.source
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Java `RandomSource.nextInt(bound)`. Java throws for `bound <= 0`; loot data
    /// can produce such bounds from malformed providers, so those return 0
    /// without consuming randomness instead of panicking the server.
    pub fn next_i32(&self, bound: i32) -> i32 {
        if bound <= 0 {
            return 0;
        }
        match &mut *self.lock() {
            LootRandomSource::Legacy(random) => random_source_next_i32(random, bound),
            LootRandomSource::Sequence {
                sequences,
                id,
                world_seed,
            } => with_sequence(sequences, id, *world_seed, |seq| seq.next_int_bound(bound)),
        }
    }

    /// Java `RandomSource.nextFloat()`.
    pub fn next_f32(&self) -> f32 {
        match &mut *self.lock() {
            LootRandomSource::Legacy(random) => random.next_f32(),
            LootRandomSource::Sequence {
                sequences,
                id,
                world_seed,
            } => with_sequence(sequences, id, *world_seed, |seq| seq.next_f32()),
        }
    }

    /// Java `RandomSource.nextBoolean()`.
    pub fn next_bool(&self) -> bool {
        match &mut *self.lock() {
            LootRandomSource::Legacy(random) => random_source_next_bool(random),
            LootRandomSource::Sequence {
                sequences,
                id,
                world_seed,
            } => with_sequence(sequences, id, *world_seed, |seq| seq.next_bool()),
        }
    }
}

/// Runs one draw against the stored sequence and marks the saved data dirty.
fn with_sequence<T>(
    sequences: &SharedRandomSequences,
    id: &str,
    world_seed: i64,
    draw: impl FnOnce(&mut crate::random_sequences::RandomSequence) -> T,
) -> T {
    let mut guard = sequences
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let value = draw(guard.get_or_create(id, world_seed));
    guard.dirty = true;
    value
}

/// Java `Identifier.toString()` always includes the namespace.
fn normalize_sequence_id(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}
