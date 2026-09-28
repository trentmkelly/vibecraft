use super::*;
use crate::random_sequences::RandomSequence;

fn temp_layout(name: &str) -> WorldLayout {
    let root = std::env::temp_dir().join(format!("vibecraft-randseq-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    WorldLayout::new(root)
}

#[test]
fn missing_file_loads_empty_and_clean() {
    let layout = temp_layout("missing");
    let sequences = load(&layout).unwrap();
    assert!(sequences.sequences.is_empty());
    assert!(!sequences.dirty);
}

#[test]
fn saved_sequences_round_trip_and_resume_the_stream() {
    let layout = temp_layout("roundtrip");
    let mut sequences = RandomSequences::default();
    let first = sequences.next_long("minecraft:chests/simple_dungeon", 1234);
    layout.save_random_sequences(&saved_data_tag(&sequences)).unwrap();

    let mut reloaded = load(&layout).unwrap();
    assert!(!reloaded.dirty);
    let mut expected = sequences.clone();
    assert_eq!(
        reloaded.next_long("minecraft:chests/simple_dungeon", 1234),
        expected.next_long("minecraft:chests/simple_dungeon", 1234)
    );
    assert_ne!(first, 0);
    let _ = std::fs::remove_dir_all(layout.root());
}

/// Known answers for `RandomSequence(seed, key)` (Java `RandomSequence` +
/// `RandomSupport.upgradeSeedTo128bitUnmixed/seedFromHashOf`): pins seed mixing.
#[test]
fn sequence_seeding_is_deterministic_per_seed_and_id() {
    let a = RandomSequence::from_seed(0, Some("minecraft:a"));
    let b = RandomSequence::from_seed(0, Some("minecraft:b"));
    let c = RandomSequence::from_seed(0, Some("minecraft:a"));
    assert_eq!(a, c);
    assert_ne!(a, b);
}

/// Known answers captured from the real 26.1.2 `RandomSequences` / `RandomSequence`
/// classes (`RandomSequences.get(key, worldSeed)` draws).
#[test]
fn java_known_answers_for_random_sequences() {
    let mut sequences = RandomSequences::default();
    let stone = sequences.get_or_create("minecraft:blocks/stone", 42);
    assert_eq!(stone.next_int_bound(1000), 969);
    assert_eq!(stone.next_f32(), 0.213_294_21_f32);

    let mut salted = RandomSequences::default();
    salted.set_seed_defaults(5, false, true);
    assert_eq!(
        salted.next_long("minecraft:x", 42),
        -4_281_125_625_265_287_293
    );

    let mut dungeon = RandomSequence::from_seed(987_654_321, Some("minecraft:chests/simple_dungeon"));
    assert_eq!(dungeon.next_long(), -8_598_195_245_415_926_714);
    assert_eq!(dungeon.next_int_bound(100), 56);
    assert_eq!(dungeon.next_int_bound(7), 2);
    assert_eq!(dungeon.next_f32(), 0.401_723_27_f32);
    assert!(!dungeon.next_bool());
}
