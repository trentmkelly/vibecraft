#![allow(dead_code)]

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDestructionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockDestructionPos {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone)]
pub struct BlockDestructionProgressModel {
    id: i32,
    pos: BlockDestructionPos,
    progress: i32,
    updated_render_tick: i32,
}

impl BlockDestructionProgressModel {
    pub fn new(id: i32, pos: BlockDestructionPos) -> Self {
        Self {
            id,
            pos,
            progress: 0,
            updated_render_tick: 0,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn pos(&self) -> BlockDestructionPos {
        self.pos
    }

    pub fn set_progress(&mut self, progress: i32) {
        self.progress = progress.min(10);
    }

    pub fn progress(&self) -> i32 {
        self.progress
    }

    pub fn update_tick(&mut self, tick: i32) {
        self.updated_render_tick = tick;
    }

    pub fn updated_render_tick(&self) -> i32 {
        self.updated_render_tick
    }

    pub fn compare_to(&self, other: &Self) -> Ordering {
        match self.progress.cmp(&other.progress) {
            Ordering::Equal => self.id.cmp(&other.id),
            ordering => ordering,
        }
    }

    pub fn java_hash_code(&self) -> i32 {
        self.id
    }
}

impl PartialEq for BlockDestructionProgressModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BlockDestructionProgressModel {}

impl Hash for BlockDestructionProgressModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_destruction_progress_stores_id_position_progress_and_tick_like_java() {
        let pos = BlockDestructionPos::new(1, 64, -2);
        let mut progress = BlockDestructionProgressModel::new(7, pos);

        assert_eq!(progress.id(), 7);
        assert_eq!(progress.pos(), pos);
        assert_eq!(progress.progress(), 0);
        assert_eq!(progress.updated_render_tick(), 0);

        progress.set_progress(6);
        progress.update_tick(42);
        assert_eq!(progress.progress(), 6);
        assert_eq!(progress.updated_render_tick(), 42);

        progress.set_progress(11);
        assert_eq!(progress.progress(), 10);

        progress.set_progress(-3);
        assert_eq!(progress.progress(), -3);
    }

    #[test]
    fn equality_and_hash_code_are_id_only_even_when_other_fields_differ() {
        let mut first = BlockDestructionProgressModel::new(9, BlockDestructionPos::new(1, 2, 3));
        let mut same_id =
            BlockDestructionProgressModel::new(9, BlockDestructionPos::new(4, 5, 6));
        let different_id =
            BlockDestructionProgressModel::new(10, BlockDestructionPos::new(1, 2, 3));

        first.set_progress(1);
        first.update_tick(20);
        same_id.set_progress(9);
        same_id.update_tick(40);

        assert_eq!(first, same_id);
        assert_ne!(first, different_id);
        assert_eq!(first.java_hash_code(), 9);
        assert_eq!(same_id.java_hash_code(), 9);
    }

    #[test]
    fn compare_to_orders_by_progress_then_id_like_java() {
        let mut low = BlockDestructionProgressModel::new(100, BlockDestructionPos::ZERO);
        let mut same_progress_lower_id =
            BlockDestructionProgressModel::new(1, BlockDestructionPos::ZERO);
        let mut high = BlockDestructionProgressModel::new(2, BlockDestructionPos::ZERO);
        low.set_progress(2);
        same_progress_lower_id.set_progress(5);
        high.set_progress(5);

        let mut entries = vec![high.clone(), low.clone(), same_progress_lower_id.clone()];
        entries.sort_by(BlockDestructionProgressModel::compare_to);

        assert_eq!(entries, vec![low, same_progress_lower_id, high]);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn block_destruction_progress_source_matches_java_26_1_2() {
        const BLOCK_DESTRUCTION_PROGRESS: &str =
            vibecraft_java_source!("/net/minecraft/server/level/BlockDestructionProgress.java");

        for sentinel in [
            "public class BlockDestructionProgress implements Comparable<BlockDestructionProgress>",
            "private final int id;",
            "private final BlockPos pos;",
            "private int progress;",
            "private int updatedRenderTick;",
            "public BlockDestructionProgress(final int id, final BlockPos pos)",
            "public int getId()",
            "public BlockPos getPos()",
            "if (progress > 10)",
            "this.progress = progress;",
            "public void updateTick(final int tick)",
            "return this.updatedRenderTick;",
            "return this.id == that.id;",
            "return Integer.hashCode(this.id);",
            "return this.progress != o.progress ? Integer.compare(this.progress, o.progress) : Integer.compare(this.id, o.id);",
        ] {
            assert!(
                BLOCK_DESTRUCTION_PROGRESS.contains(sentinel),
                "BlockDestructionProgress.java is missing sentinel: {sentinel}"
            );
        }
    }
}
