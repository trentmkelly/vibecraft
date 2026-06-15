#![allow(dead_code)]

use std::collections::{btree_map::Entry, BTreeMap};

use crate::random_source::{
    mix_stafford_13, seed128_from_hash_of, upgrade_seed_to_128bit_unmixed, Seed128, GOLDEN_RATIO_64,
    SILVER_RATIO_64,
};
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSequence {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RandomSequences {
    pub salt: i32,
    pub include_world_seed: bool,
    pub include_sequence_id: bool,
    pub sequences: BTreeMap<String, RandomSequence>,
    pub dirty: bool,
}

impl Default for RandomSequences {
    fn default() -> Self {
        Self {
            salt: 0,
            include_world_seed: true,
            include_sequence_id: true,
            sequences: BTreeMap::new(),
            dirty: false,
        }
    }
}

impl RandomSequence {
    pub fn from_seed(seed: i64, sequence_id: Option<&str>) -> Self {
        let mut seed128 = upgrade_seed_to_128bit_unmixed(seed);
        if let Some(id) = sequence_id {
            seed128 = xor_seed(seed128, seed128_from_hash_of(id));
        }
        seed128 = Seed128 {
            lo: mix_stafford_13(seed128.lo),
            hi: mix_stafford_13(seed128.hi),
        };
        let (seed_lo, seed_hi) = non_zero_xoroshiro_seed(seed128.lo, seed128.hi);
        Self { seed_lo, seed_hi }
    }

    pub fn next_long(&mut self) -> i64 {
        let s0 = self.seed_lo as u64;
        let mut s1 = self.seed_hi as u64;
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        s1 ^= s0;
        self.seed_lo = (s0.rotate_left(49) ^ s1 ^ (s1 << 21)) as i64;
        self.seed_hi = s1.rotate_left(28) as i64;
        result as i64
    }

    pub fn next_int(&mut self) -> i32 {
        self.next_long() as i32
    }

    pub fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "random bound must be positive");

        let mut random_bits = self.next_int() as u32 as u64;
        let mut multiplied_random_bits = random_bits * bound as u64;
        let mut fractional_part = multiplied_random_bits & u64::from(u32::MAX);
        if fractional_part < bound as u64 {
            let unbiased_buckets_start = (u32::MAX.wrapping_sub(bound as u32).wrapping_add(1)
                % bound as u32) as u64;
            while fractional_part < unbiased_buckets_start {
                random_bits = self.next_int() as u32 as u64;
                multiplied_random_bits = random_bits * bound as u64;
                fractional_part = multiplied_random_bits & u64::from(u32::MAX);
            }
        }

        (multiplied_random_bits >> 32) as i32
    }

    pub fn to_tag(&self) -> Tag {
        Tag::Compound(vec![(
            "source".to_string(),
            Tag::LongArray(vec![self.seed_lo, self.seed_hi]),
        )])
    }

    pub fn from_tag(tag: &Tag) -> Option<Self> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        let Some((_, Tag::LongArray(source))) = fields.iter().find(|(name, _)| name == "source")
        else {
            return None;
        };
        if source.len() != 2 {
            return None;
        }
        let (seed_lo, seed_hi) = non_zero_xoroshiro_seed(source[0], source[1]);
        Some(Self { seed_lo, seed_hi })
    }
}

impl RandomSequences {
    pub fn get_or_create(&mut self, id: &str, world_seed: i64) -> &mut RandomSequence {
        let salt = self.salt;
        let include_world_seed = self.include_world_seed;
        let include_sequence_id = self.include_sequence_id;
        match self.sequences.entry(id.to_string()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let seed = (if include_world_seed { world_seed } else { 0 }) ^ i64::from(salt);
                let sequence = RandomSequence::from_seed(seed, include_sequence_id.then_some(id));
                self.dirty = true;
                entry.insert(sequence)
            }
        }
    }

    pub fn next_long(&mut self, id: &str, world_seed: i64) -> i64 {
        let value = self.get_or_create(id, world_seed).next_long();
        self.dirty = true;
        value
    }

    pub fn next_int_bound(&mut self, id: &str, world_seed: i64, bound: i32) -> i32 {
        let value = self.get_or_create(id, world_seed).next_int_bound(bound);
        self.dirty = true;
        value
    }

    pub fn set_seed_defaults(
        &mut self,
        salt: i32,
        include_world_seed: bool,
        include_sequence_id: bool,
    ) {
        self.salt = salt;
        self.include_world_seed = include_world_seed;
        self.include_sequence_id = include_sequence_id;
    }

    pub fn clear(&mut self) -> usize {
        let count = self.sequences.len();
        self.sequences.clear();
        count
    }

    pub fn reset(&mut self, id: &str, world_seed: i64) {
        let sequence = self.create_sequence_with_options(
            id,
            world_seed,
            self.salt,
            self.include_world_seed,
            self.include_sequence_id,
        );
        self.sequences.insert(id.to_string(), sequence);
    }

    pub fn reset_with_options(
        &mut self,
        id: &str,
        world_seed: i64,
        salt: i32,
        include_world_seed: bool,
        include_sequence_id: bool,
    ) {
        let sequence = self.create_sequence_with_options(
            id,
            world_seed,
            salt,
            include_world_seed,
            include_sequence_id,
        );
        self.sequences.insert(id.to_string(), sequence);
    }

    pub fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            ("salt".to_string(), Tag::Int(self.salt)),
            (
                "include_world_seed".to_string(),
                Tag::Byte(i8::from(self.include_world_seed)),
            ),
            (
                "include_sequence_id".to_string(),
                Tag::Byte(i8::from(self.include_sequence_id)),
            ),
            (
                "sequences".to_string(),
                Tag::Compound(
                    self.sequences
                        .iter()
                        .map(|(id, sequence)| (id.clone(), sequence.to_tag()))
                        .collect(),
                ),
            ),
        ])
    }

    pub fn from_tag(tag: &Tag) -> Option<Self> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        let mut model = Self {
            salt: int_field(fields, "salt").unwrap_or(0),
            include_world_seed: bool_field(fields, "include_world_seed").unwrap_or(true),
            include_sequence_id: bool_field(fields, "include_sequence_id").unwrap_or(true),
            ..Self::default()
        };
        if let Some((_, Tag::Compound(sequences))) =
            fields.iter().find(|(name, _)| name == "sequences")
        {
            model.sequences = sequences
                .iter()
                .filter_map(|(id, tag)| RandomSequence::from_tag(tag).map(|sequence| (id.clone(), sequence)))
                .collect();
        }
        model.dirty = false;
        Some(model)
    }

    fn create_sequence(&self, id: &str, world_seed: i64) -> RandomSequence {
        self.create_sequence_with_options(
            id,
            world_seed,
            self.salt,
            self.include_world_seed,
            self.include_sequence_id,
        )
    }

    fn create_sequence_with_options(
        &self,
        id: &str,
        world_seed: i64,
        salt: i32,
        include_world_seed: bool,
        include_sequence_id: bool,
    ) -> RandomSequence {
        let seed = (if include_world_seed { world_seed } else { 0 }) ^ i64::from(salt);
        RandomSequence::from_seed(seed, include_sequence_id.then_some(id))
    }
}

fn xor_seed(left: Seed128, right: Seed128) -> Seed128 {
    Seed128 {
        lo: left.lo ^ right.lo,
        hi: left.hi ^ right.hi,
    }
}

fn non_zero_xoroshiro_seed(seed_lo: i64, seed_hi: i64) -> (i64, i64) {
    if (seed_lo | seed_hi) == 0 {
        (GOLDEN_RATIO_64, SILVER_RATIO_64)
    } else {
        (seed_lo, seed_hi)
    }
}

fn int_field(fields: &[(String, Tag)], name: &str) -> Option<i32> {
    match fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::Int(value) => Some(value),
        _ => None,
    }
}

fn bool_field(fields: &[(String, Tag)], name: &str) -> Option<bool> {
    match fields.iter().find(|(field, _)| field == name)?.1 {
        Tag::Byte(value) => Some(value != 0),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_sequences_saved_data_codec_shape_matches_java() {
        let mut sequences = RandomSequences::default();
        sequences.reset("minecraft:chests/simple_dungeon", 12_345);
        sequences.set_seed_defaults(7, false, true);
        sequences.reset_with_options("minecraft:test", 12_345, 7, false, true);

        let tag = sequences.to_tag();
        let loaded = RandomSequences::from_tag(&tag).expect("saved data should decode");

        assert_eq!(loaded.salt, 7);
        assert!(!loaded.include_world_seed);
        assert!(loaded.include_sequence_id);
        assert_eq!(loaded.sequences.len(), 2);
        assert!(loaded
            .sequences
            .contains_key("minecraft:chests/simple_dungeon"));
        assert_eq!(loaded.to_tag(), tag);
        assert!(!loaded.dirty);
    }

    #[test]
    fn random_sequences_create_reset_clear_and_dirty_like_java_saved_data() {
        let mut sequences = RandomSequences::default();
        let first = sequences.next_long("minecraft:test", 9);
        assert!(sequences.dirty);
        sequences.dirty = false;

        let second = sequences.next_long("minecraft:test", 9);
        assert_ne!(first, second);
        assert!(sequences.dirty);

        sequences.dirty = false;
        sequences.reset("minecraft:test", 9);
        assert!(!sequences.dirty);
        let reset_first = sequences.next_long("minecraft:test", 9);
        assert_eq!(first, reset_first);
        sequences.dirty = false;
        assert_eq!(sequences.clear(), 1);
        assert!(sequences.sequences.is_empty());
        assert!(!sequences.dirty);
    }

    #[test]
    fn random_sequence_seed_for_key_matches_known_java_md5_fixture() {
        let key_seed = seed128_from_hash_of("minecraft:chests/simple_dungeon");
        assert_eq!(
            key_seed,
            crate::random_source::seed128_from_md5_digest([
                0x95, 0x74, 0x12, 0x41, 0xc4, 0x91, 0x68, 0x95, 0x53, 0x8c, 0x2e, 0x10, 0x05,
                0xbc, 0x91, 0xb7,
            ])
        );

        let without_id = RandomSequence::from_seed(0, None);
        let with_id = RandomSequence::from_seed(0, Some("minecraft:chests/simple_dungeon"));
        assert_ne!(without_id, with_id);
    }
}
