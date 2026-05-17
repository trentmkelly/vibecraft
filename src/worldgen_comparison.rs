#![allow(dead_code)]

use crate::seed_validation::{build_seed_parity_sample, ChunkCoord, SeedParitySample};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenChunkComparison {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub fingerprint: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldgenComparisonDiff {
    pub seed: i64,
    pub chunk: ChunkCoord,
    pub left_fingerprint: u64,
    pub right_fingerprint: u64,
}

pub fn build_worldgen_chunk_comparisons(
    seeds: &[i64],
    chunks: &[ChunkCoord],
) -> Vec<WorldgenChunkComparison> {
    seeds
        .iter()
        .flat_map(|seed| {
            chunks.iter().map(move |chunk| {
                let sample = build_seed_parity_sample(*seed, chunk.x, chunk.z);
                WorldgenChunkComparison {
                    seed: *seed,
                    chunk: *chunk,
                    fingerprint: fingerprint_sample(&sample),
                }
            })
        })
        .collect()
}

pub fn diff_worldgen_comparisons(
    left: &[WorldgenChunkComparison],
    right: &[WorldgenChunkComparison],
) -> Vec<WorldgenComparisonDiff> {
    left.iter()
        .zip(right)
        .filter_map(|(left, right)| {
            if left.seed == right.seed
                && left.chunk == right.chunk
                && left.fingerprint != right.fingerprint
            {
                Some(WorldgenComparisonDiff {
                    seed: left.seed,
                    chunk: left.chunk,
                    left_fingerprint: left.fingerprint,
                    right_fingerprint: right.fingerprint,
                })
            } else {
                None
            }
        })
        .collect()
}

fn fingerprint_sample(sample: &SeedParitySample) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    mix_i64(&mut hash, sample.seed);
    mix_i32(&mut hash, sample.chunk.x);
    mix_i32(&mut hash, sample.chunk.z);
    mix_i32(&mut hash, sample.biome_sample_block.0);
    mix_i32(&mut hash, sample.biome_sample_block.1);
    mix_i32(&mut hash, sample.biome_sample_block.2);
    mix_i32(&mut hash, sample.spawn_search_candidate.0);
    mix_i32(&mut hash, sample.spawn_search_candidate.1);
    mix_i64(&mut hash, sample.decoration_seed);
    mix_i64(&mut hash, sample.feature_seed);
    mix_i64(&mut hash, sample.slime_chunk_seed);
    mix_i64(&mut hash, sample.loot_sequence_seed.lo);
    mix_i64(&mut hash, sample.loot_sequence_seed.hi);
    for (id, chunk) in &sample.structure_chunks {
        mix_bytes(&mut hash, id.as_bytes());
        mix_i32(&mut hash, chunk.x);
        mix_i32(&mut hash, chunk.z);
    }
    hash
}

fn mix_i32(hash: &mut u64, value: i32) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_i64(hash: &mut u64, value: i64) {
    mix_bytes(hash, &value.to_le_bytes());
}

fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: &[i64] = &[0, 1, -1, 12_345, 8_675_309, i64::MIN + 1, i64::MAX];
    const CHUNKS: &[ChunkCoord] = &[
        ChunkCoord { x: 0, z: 0 },
        ChunkCoord { x: 1, z: -1 },
        ChunkCoord { x: -12, z: 34 },
        ChunkCoord { x: 1875, z: -1875 },
        ChunkCoord {
            x: i32::MIN / 1024,
            z: i32::MAX / 1024,
        },
    ];

    #[test]
    fn worldgen_chunk_comparison_matrix_is_deterministic_across_many_seeds_and_coordinates() {
        let first = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);
        let second = build_worldgen_chunk_comparisons(SEEDS, CHUNKS);

        assert_eq!(first, second);
        assert_eq!(first.len(), SEEDS.len() * CHUNKS.len());
        assert!(first.iter().all(|entry| entry.fingerprint != 0));
    }

    #[test]
    fn worldgen_chunk_comparison_fingerprints_change_with_seed_or_coordinate() {
        let baseline =
            build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
        let different_seed =
            build_worldgen_chunk_comparisons(&[54_321], &[ChunkCoord { x: 0, z: 0 }])[0].clone();
        let different_chunk =
            build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 2, z: 3 }])[0].clone();

        assert_ne!(baseline.fingerprint, different_seed.fingerprint);
        assert_ne!(baseline.fingerprint, different_chunk.fingerprint);
    }

    #[test]
    fn worldgen_comparison_diff_reports_matching_seed_chunk_fingerprint_changes() {
        let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
        let mut right = left.clone();
        right[0].fingerprint ^= 0xfeed;

        assert_eq!(
            diff_worldgen_comparisons(&left, &right),
            vec![WorldgenComparisonDiff {
                seed: 12_345,
                chunk: ChunkCoord { x: 0, z: 0 },
                left_fingerprint: left[0].fingerprint,
                right_fingerprint: right[0].fingerprint,
            }]
        );
    }

    #[test]
    fn worldgen_comparison_diff_ignores_mismatched_rows_so_matrices_fail_closed_elsewhere() {
        let left = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 0, z: 0 }]);
        let right = build_worldgen_chunk_comparisons(&[12_345], &[ChunkCoord { x: 1, z: 0 }]);

        assert!(diff_worldgen_comparisons(&left, &right).is_empty());
    }
}
