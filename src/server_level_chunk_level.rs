#![allow(dead_code)]

use crate::chunk_ticket::{
    FullChunkStatus, BLOCK_TICKING_LEVEL, ENTITY_TICKING_LEVEL, FULL_CHUNK_LEVEL,
};
use crate::storage::chunk::{
    chunk_generation_task_layer_radius, chunk_pyramid_accumulated_dependencies, ChunkPyramidKind,
};

pub const RADIUS_AROUND_FULL_CHUNK: i32 = 11;
pub const MAX_LEVEL: i32 = FULL_CHUNK_LEVEL + RADIUS_AROUND_FULL_CHUNK;

pub fn generation_status(level: i32) -> Option<&'static str> {
    get_status_around_full_chunk_or(level - FULL_CHUNK_LEVEL, None)
}

pub fn get_status_around_full_chunk(distance_to_full_chunk: i32) -> Option<&'static str> {
    get_status_around_full_chunk_or(distance_to_full_chunk, Some("minecraft:empty"))
}

pub fn get_status_around_full_chunk_or(
    distance_to_full_chunk: i32,
    default_value: Option<&'static str>,
) -> Option<&'static str> {
    if distance_to_full_chunk > RADIUS_AROUND_FULL_CHUNK {
        return default_value;
    }

    if distance_to_full_chunk <= 0 {
        return Some("minecraft:full");
    }

    chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Generation, "minecraft:full")
        .and_then(|dependencies| dependencies.get(distance_to_full_chunk as usize).copied())
}

pub fn by_chunk_status(status: &str) -> Option<i32> {
    chunk_generation_task_layer_radius("minecraft:full", status, true)
        .map(|radius| FULL_CHUNK_LEVEL + radius)
}

pub fn full_status(level: i32) -> FullChunkStatus {
    FullChunkStatus::by_level(level)
}

pub fn by_full_chunk_status(status: FullChunkStatus) -> i32 {
    match status {
        FullChunkStatus::Inaccessible => MAX_LEVEL,
        FullChunkStatus::Full => FULL_CHUNK_LEVEL,
        FullChunkStatus::BlockTicking => BLOCK_TICKING_LEVEL,
        FullChunkStatus::EntityTicking => ENTITY_TICKING_LEVEL,
    }
}

pub fn is_entity_ticking(level: i32) -> bool {
    level <= ENTITY_TICKING_LEVEL
}

pub fn is_block_ticking(level: i32) -> bool {
    level <= BLOCK_TICKING_LEVEL
}

pub fn is_loaded(level: i32) -> bool {
    level <= MAX_LEVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_level_generation_status_matches_java_full_chunk_step() {
        assert_eq!(RADIUS_AROUND_FULL_CHUNK, 11);
        assert_eq!(MAX_LEVEL, 44);
        assert_eq!(generation_status(31), Some("minecraft:full"));
        assert_eq!(generation_status(33), Some("minecraft:full"));
        assert_eq!(generation_status(34), Some("minecraft:initialize_light"));
        assert_eq!(generation_status(35), Some("minecraft:carvers"));
        assert_eq!(generation_status(36), Some("minecraft:biomes"));
        assert_eq!(generation_status(44), Some("minecraft:structure_starts"));
        assert_eq!(generation_status(45), None);
    }

    #[test]
    fn status_around_full_chunk_overloads_match_java_defaults() {
        assert_eq!(
            get_status_around_full_chunk_or(12, Some("minecraft:noise")),
            Some("minecraft:noise")
        );
        assert_eq!(get_status_around_full_chunk_or(12, None), None);
        assert_eq!(
            get_status_around_full_chunk(12),
            Some("minecraft:empty")
        );
        assert_eq!(
            get_status_around_full_chunk(-4),
            Some("minecraft:full")
        );
    }

    #[test]
    fn chunk_level_status_level_conversions_match_java_thresholds() {
        assert_eq!(by_chunk_status("minecraft:full"), Some(33));
        assert_eq!(by_chunk_status("minecraft:spawn"), Some(33));
        assert_eq!(by_chunk_status("minecraft:light"), Some(33));
        assert_eq!(by_chunk_status("minecraft:initialize_light"), Some(34));
        assert_eq!(by_chunk_status("minecraft:features"), Some(34));
        assert_eq!(by_chunk_status("minecraft:carvers"), Some(35));
        assert_eq!(by_chunk_status("minecraft:biomes"), Some(36));
        assert_eq!(by_chunk_status("minecraft:structure_starts"), Some(44));
        assert_eq!(by_chunk_status("minecraft:unknown"), None);

        assert_eq!(full_status(31), FullChunkStatus::EntityTicking);
        assert_eq!(full_status(32), FullChunkStatus::BlockTicking);
        assert_eq!(full_status(33), FullChunkStatus::Full);
        assert_eq!(full_status(34), FullChunkStatus::Inaccessible);
        assert_eq!(
            by_full_chunk_status(FullChunkStatus::Inaccessible),
            MAX_LEVEL
        );
        assert_eq!(by_full_chunk_status(FullChunkStatus::Full), 33);
        assert_eq!(by_full_chunk_status(FullChunkStatus::BlockTicking), 32);
        assert_eq!(by_full_chunk_status(FullChunkStatus::EntityTicking), 31);
    }

    #[test]
    fn ticking_and_loaded_predicates_match_java_thresholds() {
        assert!(is_entity_ticking(31));
        assert!(!is_entity_ticking(32));
        assert!(is_block_ticking(32));
        assert!(!is_block_ticking(33));
        assert!(is_loaded(44));
        assert!(!is_loaded(45));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn chunk_level_source_matches_java_26_1_2() {
        const CHUNK_LEVEL: &str =
            vibecraft_java_source!("/net/minecraft/server/level/ChunkLevel.java");

        for sentinel in [
            "public class ChunkLevel",
            "private static final int FULL_CHUNK_LEVEL = 33;",
            "private static final int BLOCK_TICKING_LEVEL = 32;",
            "private static final int ENTITY_TICKING_LEVEL = 31;",
            "public static final int RADIUS_AROUND_FULL_CHUNK = FULL_CHUNK_STEP.accumulatedDependencies().getRadius();",
            "public static final int MAX_LEVEL = 33 + RADIUS_AROUND_FULL_CHUNK;",
            "public static @Nullable ChunkStatus generationStatus(final int level)",
            "return getStatusAroundFullChunk(level - 33, null);",
            "public static @Nullable ChunkStatus getStatusAroundFullChunk(final int distanceToFullChunk, final @Nullable ChunkStatus defaultValue)",
            "if (distanceToFullChunk > RADIUS_AROUND_FULL_CHUNK)",
            "return distanceToFullChunk <= 0 ? ChunkStatus.FULL : FULL_CHUNK_STEP.accumulatedDependencies().get(distanceToFullChunk);",
            "public static ChunkStatus getStatusAroundFullChunk(final int distanceToFullChunk)",
            "return getStatusAroundFullChunk(distanceToFullChunk, ChunkStatus.EMPTY);",
            "public static int byStatus(final ChunkStatus status)",
            "return 33 + FULL_CHUNK_STEP.getAccumulatedRadiusOf(status);",
            "public static FullChunkStatus fullStatus(final int level)",
            "public static int byStatus(final FullChunkStatus status)",
            "case INACCESSIBLE -> MAX_LEVEL;",
            "case FULL -> 33;",
            "case BLOCK_TICKING -> 32;",
            "case ENTITY_TICKING -> 31;",
            "public static boolean isEntityTicking(final int level)",
            "return level <= 31;",
            "public static boolean isBlockTicking(final int level)",
            "return level <= 32;",
            "public static boolean isLoaded(final int level)",
            "return level <= MAX_LEVEL;",
        ] {
            assert!(
                CHUNK_LEVEL.contains(sentinel),
                "ChunkLevel.java is missing sentinel: {sentinel}"
            );
        }
    }
}
