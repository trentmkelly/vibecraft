//! Loot random-source parity: `LootContext` must draw from Java's real
//! `RandomSource` implementations (known answers captured from the 26.1.2 jar).

use std::sync::{Arc, Mutex};

use super::*;
use crate::random_sequences::RandomSequences;

#[test]
fn seeded_context_uses_java_legacy_random_source() {
    // RandomSource.create(12345L): nextInt(100), nextInt(64), nextInt(7), nextFloat,
    // nextBoolean, nextInt(1).
    let random = LootRandom::from_seed(12345);
    assert_eq!(random.next_i32(100), 51);
    assert_eq!(random.next_i32(64), 32);
    assert_eq!(random.next_i32(7), 4);
    assert_eq!(random.next_f32(), 0.917_114_7_f32);
    assert!(random.next_bool());
    assert_eq!(random.next_i32(1), 0);
}

#[test]
fn cloned_handles_share_one_stream() {
    let random = LootRandom::from_seed(12345);
    let alias = random.clone();
    assert_eq!(alias.next_i32(100), 51);
    assert_eq!(random.next_i32(64), 32);
}

#[test]
fn sequence_random_matches_java_and_marks_saved_data_dirty() {
    let sequences = Arc::new(Mutex::new(RandomSequences::default()));
    let random = LootRandom::from_sequence(sequences.clone(), "minecraft:chests/simple_dungeon", 987_654_321);
    sequences.lock().unwrap().dirty = false;
    // First Java draw is nextLong; nextInt(bound) consumes from the same stream, so
    // compare against an independent sequence advanced identically.
    let mut reference = RandomSequences::default();
    let expected = reference
        .get_or_create("minecraft:chests/simple_dungeon", 987_654_321)
        .next_int_bound(100);
    assert_eq!(random.next_i32(100), expected);
    assert!(sequences.lock().unwrap().dirty);
}

fn one_roll_table() -> LootTable {
    let mut table = LootTable::empty();
    table.param_set = LootParamSet::Chest;
    table.random_sequence = Some("minecraft:chests/simple_dungeon".to_string());
    let mut pool = LootPool::single(LootEntry::item("minecraft:coal", 1));
    pool.rolls = NumberProvider::Constant(6.0);
    table.pools = vec![pool];
    table
}

#[test]
fn table_random_sequence_advances_shared_sequences_across_evaluations() {
    let mut table = one_roll_table();
    table.pools[0].entries.push(LootEntry::item("minecraft:iron_ingot", 1));
    let sequences = Arc::new(Mutex::new(RandomSequences::default()));
    let eval = |sequences: &Arc<Mutex<RandomSequences>>| {
        let mut context = LootContext::new(LootParamSet::Chest, 0)
            .with_random_sequences(sequences.clone(), 77);
        table.evaluate(&mut context)
    };
    let first = eval(&sequences);
    let second = eval(&sequences);

    // A second store replays the identical persisted stream: evaluation N continues
    // where N-1 left the sequence rather than reseeding.
    let replay_store = Arc::new(Mutex::new(RandomSequences::default()));
    assert_eq!(first, eval(&replay_store));
    assert_eq!(second, eval(&replay_store));
    assert!(sequences.lock().unwrap().dirty);
}

#[test]
fn explicit_seed_takes_precedence_over_random_sequence() {
    let table = one_roll_table();
    let sequences = Arc::new(Mutex::new(RandomSequences::default()));
    let mut seeded = LootContext::new(LootParamSet::Chest, 99)
        .with_random_sequences(sequences.clone(), 77);
    table.evaluate(&mut seeded);
    // Java Builder.create: explicit random wins, so the sequence is never created.
    assert!(sequences.lock().unwrap().sequences.is_empty());
}
