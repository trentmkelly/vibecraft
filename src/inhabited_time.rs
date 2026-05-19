#![allow(dead_code)]

use crate::storage::chunk::LevelChunk;
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawningChunkTick {
    pub chunk: ChunkPos,
    pub time_diff: i64,
    pub entity_ticking: bool,
    pub can_spawn_entities: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InhabitedTickOutcome {
    pub chunk: ChunkPos,
    pub inhabited_time: i64,
    pub tick_thunder: bool,
    pub run_natural_spawner: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DifficultyInputs {
    pub local_time: i64,
    pub moon_brightness: f32,
}

pub fn increment_inhabited_time(chunk: &mut LevelChunk, time_diff: i64) {
    chunk.inhabited_time += time_diff;
}

pub fn tick_spawning_chunk(
    chunk: &mut LevelChunk,
    tick: SpawningChunkTick,
    has_spawning_categories: bool,
) -> InhabitedTickOutcome {
    increment_inhabited_time(chunk, tick.time_diff);
    InhabitedTickOutcome {
        chunk: tick.chunk,
        inhabited_time: chunk.inhabited_time,
        tick_thunder: tick.entity_ticking,
        run_natural_spawner: has_spawning_categories && tick.can_spawn_entities,
    }
}

pub fn difficulty_inputs_for_chunk(
    chunk: Option<&LevelChunk>,
    moon_brightness: f32,
) -> DifficultyInputs {
    match chunk {
        Some(chunk) => DifficultyInputs {
            local_time: chunk.inhabited_time,
            moon_brightness,
        },
        None => DifficultyInputs {
            local_time: 0,
            moon_brightness: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        difficulty_inputs_for_chunk, increment_inhabited_time, tick_spawning_chunk,
        SpawningChunkTick,
    };
    use crate::storage::chunk::LevelChunk;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;

    #[test]
    fn spawning_chunk_tick_adds_time_diff_like_server_chunk_cache() {
        let pos = ChunkPos { x: 2, z: -4 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.inhabited_time = 120;

        let outcome = tick_spawning_chunk(
            &mut chunk,
            SpawningChunkTick {
                chunk: pos,
                time_diff: 3,
                entity_ticking: true,
                can_spawn_entities: true,
            },
            true,
        );

        assert_eq!(chunk.inhabited_time, 123);
        assert_eq!(outcome.inhabited_time, 123);
        assert!(outcome.tick_thunder);
        assert!(outcome.run_natural_spawner);
    }

    #[test]
    fn natural_spawning_requires_categories_and_spawnable_chunk() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        let no_categories = tick_spawning_chunk(
            &mut chunk,
            SpawningChunkTick {
                chunk: pos,
                time_diff: 1,
                entity_ticking: false,
                can_spawn_entities: true,
            },
            false,
        );
        assert!(!no_categories.tick_thunder);
        assert!(!no_categories.run_natural_spawner);

        let blocked_chunk = tick_spawning_chunk(
            &mut chunk,
            SpawningChunkTick {
                chunk: pos,
                time_diff: 1,
                entity_ticking: true,
                can_spawn_entities: false,
            },
            true,
        );
        assert!(blocked_chunk.tick_thunder);
        assert!(!blocked_chunk.run_natural_spawner);
    }

    #[test]
    fn difficulty_inputs_use_chunk_inhabited_time_or_zero_when_missing() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 1, z: 1 });
        chunk.inhabited_time = 99;

        assert_eq!(
            difficulty_inputs_for_chunk(Some(&chunk), 0.75).local_time,
            99
        );
        assert_eq!(
            difficulty_inputs_for_chunk(Some(&chunk), 0.75).moon_brightness,
            0.75
        );
        assert_eq!(difficulty_inputs_for_chunk(None, 0.75).local_time, 0);
        assert_eq!(difficulty_inputs_for_chunk(None, 0.75).moon_brightness, 0.0);
    }

    #[test]
    fn inhabited_time_round_trips_through_chunk_nbt() {
        let pos = ChunkPos { x: -7, z: 8 };
        let mut chunk = LevelChunk::empty(pos);
        increment_inhabited_time(&mut chunk, 42);
        increment_inhabited_time(&mut chunk, 5);

        let encoded = chunk.to_nbt(crate::storage::datafix::TARGET_DATA_VERSION);
        let decoded = LevelChunk::from_nbt(pos, &encoded).expect("chunk decodes");
        assert_eq!(decoded.inhabited_time, 47);
        match encoded {
            Tag::Compound(fields) => assert!(fields
                .iter()
                .any(|(name, value)| name == "InhabitedTime" && *value == Tag::Long(47))),
            other => panic!("expected compound, got {other:?}"),
        }
    }
}
