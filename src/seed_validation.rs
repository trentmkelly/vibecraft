#![allow(dead_code)]

use crate::random_source::{
    decoration_seed, feature_seed, large_feature_seed_with_salt, seed128_from_md5_digest,
    slime_chunk_seed, LegacyRandom, RandomAlgorithm, Seed128,
};
use crate::worldgen::{
    initial_spawn_position, spawn_search_candidate, InitialSpawnKind, RandomSpreadType,
    StructurePlacementKind, BUILTIN_STRUCTURE_SETS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeedParitySample {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub biome_sample_block: (i32, i32, i32),
    pub initial_spawn: InitialSpawnKind,
    pub spawn_search_candidate: (i32, i32),
    pub structure_chunks: Vec<(&'static str, ChunkCoord)>,
    pub decoration_seed: i64,
    pub feature_seed: i64,
    pub slime_chunk_seed: i64,
    pub loot_sequence_seed: Seed128,
}

pub const SEED_PARITY_MATRIX_SEEDS: &[i64] = &[0, 1, -1, 12_345, 8_675_309, i64::MAX];
pub const SEED_PARITY_MATRIX_CHUNKS: &[ChunkCoord] = &[
    ChunkCoord { x: 0, z: 0 },
    ChunkCoord { x: 1, z: 0 },
    ChunkCoord { x: 0, z: 1 },
    ChunkCoord { x: -1, z: -1 },
    ChunkCoord { x: 16, z: 16 },
    ChunkCoord { x: -32, z: 32 },
];

pub fn build_seed_parity_matrix() -> Vec<SeedParitySample> {
    SEED_PARITY_MATRIX_SEEDS
        .iter()
        .flat_map(|seed| {
            SEED_PARITY_MATRIX_CHUNKS
                .iter()
                .map(move |chunk| build_seed_parity_sample(*seed, chunk.x, chunk.z))
        })
        .collect()
}

pub fn build_seed_parity_sample(seed: i64, chunk_x: i32, chunk_z: i32) -> SeedParitySample {
    let initial_spawn = initial_spawn_position(false, false, false, chunk_x, chunk_z, 64, -64, 70);
    let Some(spawn_search_candidate) =
        spawn_search_candidate(chunk_x * 16 + 8, chunk_z * 16 + 8, 10, 0, 17)
    else {
        panic!("seed parity spawn candidate index must stay below the vanilla cap");
    };
    let structure_chunks = BUILTIN_STRUCTURE_SETS
        .iter()
        .filter_map(|set| match set.placement {
            StructurePlacementKind::RandomSpread {
                spacing,
                separation,
                salt,
                spread_type,
            } => Some((
                set.id,
                potential_random_spread_chunk(
                    seed,
                    chunk_x,
                    chunk_z,
                    spacing,
                    separation,
                    salt,
                    spread_type,
                ),
            )),
            StructurePlacementKind::ConcentricRings { .. } => None,
        })
        .take(6)
        .collect();
    let decoration = decoration_seed(seed, chunk_x, chunk_z, RandomAlgorithm::Xoroshiro);

    SeedParitySample {
        seed,
        chunk: ChunkCoord {
            x: chunk_x,
            z: chunk_z,
        },
        biome_sample_block: (chunk_x * 16 + 8, 64, chunk_z * 16 + 8),
        initial_spawn,
        spawn_search_candidate,
        structure_chunks,
        decoration_seed: decoration,
        feature_seed: feature_seed(decoration, 3, 2),
        slime_chunk_seed: slime_chunk_seed(chunk_x, chunk_z, seed, 987_234_911),
        // MD5("minecraft:chests/simple_dungeon"), matching RandomSequence.seedForKey's
        // big-endian 128-bit split without pulling the full hashing pipeline into this harness.
        loot_sequence_seed: seed128_from_md5_digest([
            0x95, 0x74, 0x12, 0x41, 0xc4, 0x91, 0x68, 0x95, 0x53, 0x8c, 0x2e, 0x10, 0x05, 0xbc,
            0x91, 0xb7,
        ]),
    }
}

pub fn potential_random_spread_chunk(
    seed: i64,
    source_x: i32,
    source_z: i32,
    spacing: i32,
    separation: i32,
    salt: i32,
    spread_type: RandomSpreadType,
) -> ChunkCoord {
    let grid_x = source_x.div_euclid(spacing);
    let grid_z = source_z.div_euclid(spacing);
    let mut random = LegacyRandom::new(large_feature_seed_with_salt(seed, grid_x, grid_z, salt));
    let limit = spacing - separation;
    let spread_x = evaluate_spread(&mut random, spread_type, limit);
    let spread_z = evaluate_spread(&mut random, spread_type, limit);
    ChunkCoord {
        x: grid_x * spacing + spread_x,
        z: grid_z * spacing + spread_z,
    }
}

fn evaluate_spread(random: &mut LegacyRandom, spread_type: RandomSpreadType, limit: i32) -> i32 {
    match spread_type {
        RandomSpreadType::Linear => random.next_i32_bound(limit),
        RandomSpreadType::Triangular => {
            (random.next_i32_bound(limit) + random.next_i32_bound(limit)) / 2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_seed_parity_matrix, build_seed_parity_sample, potential_random_spread_chunk,
        ChunkCoord, SEED_PARITY_MATRIX_CHUNKS, SEED_PARITY_MATRIX_SEEDS,
    };
    use crate::random_source::Seed128;
    use crate::worldgen::RandomSpreadType;

    #[test]
    fn random_spread_structure_chunks_are_deterministic_for_same_seed() {
        assert_eq!(
            potential_random_spread_chunk(12345, 0, 0, 34, 8, 10387312, RandomSpreadType::Linear),
            ChunkCoord { x: 21, z: 5 }
        );
        assert_eq!(
            potential_random_spread_chunk(12345, -1, -1, 32, 8, 14357618, RandomSpreadType::Linear),
            ChunkCoord { x: -19, z: -14 }
        );
        assert_eq!(
            potential_random_spread_chunk(
                12345,
                2,
                2,
                32,
                5,
                10387313,
                RandomSpreadType::Triangular
            ),
            ChunkCoord { x: 3, z: 19 }
        );
    }

    #[test]
    fn seed_parity_sample_covers_seeded_worldgen_and_loot_surfaces() {
        let first = build_seed_parity_sample(12345, 0, 0);
        let second = build_seed_parity_sample(12345, 0, 0);
        let different = build_seed_parity_sample(54321, 0, 0);

        assert_eq!(first, second);
        assert_ne!(first.structure_chunks, different.structure_chunks);
        assert_ne!(first.decoration_seed, different.decoration_seed);
        assert_ne!(first.feature_seed, different.feature_seed);
        assert_eq!(first.biome_sample_block, (8, 64, 8));
        assert_eq!(first.spawn_search_candidate, (14, 11));
        assert_eq!(first.structure_chunks.len(), 6);
        assert_eq!(
            first.loot_sequence_seed,
            Seed128 {
                lo: -7_677_491_391_079_815_019,
                hi: 6_020_237_448_238_109_111
            }
        );
    }

    #[test]
    fn seed_parity_matrix_covers_chunks_structures_loot_and_spawn_candidates() {
        let first = build_seed_parity_matrix();
        let second = build_seed_parity_matrix();

        assert_eq!(first, second);
        assert_eq!(
            first.len(),
            SEED_PARITY_MATRIX_SEEDS.len() * SEED_PARITY_MATRIX_CHUNKS.len()
        );
        assert!(first
            .iter()
            .all(|sample| sample.structure_chunks.len() == 6));
        assert!(first.iter().all(
            |sample| sample.spawn_search_candidate.0 >= sample.chunk.x * 16
                && sample.spawn_search_candidate.0 < sample.chunk.x * 16 + 24
        ));
        assert!(first.iter().all(
            |sample| sample.spawn_search_candidate.1 >= sample.chunk.z * 16
                && sample.spawn_search_candidate.1 < sample.chunk.z * 16 + 24
        ));
        assert!(first
            .iter()
            .all(|sample| sample.loot_sequence_seed.lo != 0 || sample.loot_sequence_seed.hi != 0));

        let unique_structure_sets = first
            .iter()
            .flat_map(|sample| sample.structure_chunks.iter().map(|(id, _)| *id))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique_structure_sets.len(), 6);
    }
}
