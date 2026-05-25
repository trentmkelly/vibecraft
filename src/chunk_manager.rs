#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::storage::chunk::{chunk_status, chunk_status_is_or_after, LevelChunk};
use crate::storage::region::ChunkPos;

#[derive(Debug, Clone, PartialEq)]
pub struct ManagedChunk {
    pub chunk: LevelChunk,
    pub dirty: bool,
    pub loaded_tick: u64,
    pub last_access_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkLifecycleEvent {
    Loaded(ChunkPos),
    Generated { pos: ChunkPos, status: &'static str },
    Saved(ChunkPos),
    Unloaded(ChunkPos),
    Evicted(ChunkPos),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkManager {
    chunks: BTreeMap<ChunkPos, ManagedChunk>,
    saved: BTreeMap<ChunkPos, LevelChunk>,
    forced: BTreeSet<ChunkPos>,
    events: VecDeque<ChunkLifecycleEvent>,
    tick: u64,
    capacity: usize,
}

impl ChunkManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            chunks: BTreeMap::new(),
            saved: BTreeMap::new(),
            forced: BTreeSet::new(),
            events: VecDeque::new(),
            tick: 0,
            capacity,
        }
    }

    pub fn insert_saved_chunk(&mut self, chunk: LevelChunk) {
        self.saved.insert(chunk.pos, chunk);
    }

    pub fn load_or_generate(
        &mut self,
        pos: ChunkPos,
        target_status: &str,
    ) -> Result<&LevelChunk, String> {
        self.tick += 1;
        if !self.chunks.contains_key(&pos) {
            let chunk = self
                .saved
                .get(&pos)
                .cloned()
                .unwrap_or_else(|| LevelChunk::empty(pos));
            self.events.push_back(ChunkLifecycleEvent::Loaded(pos));
            self.chunks.insert(
                pos,
                ManagedChunk {
                    chunk,
                    dirty: false,
                    loaded_tick: self.tick,
                    last_access_tick: self.tick,
                },
            );
        }

        self.advance_status(pos, target_status)?;
        self.evict_if_needed();
        self.chunks
            .get(&pos)
            .map(|chunk| &chunk.chunk)
            .ok_or_else(|| format!("chunk {pos:?} was evicted before load completed"))
    }

    pub fn mark_dirty(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.chunks.get_mut(&pos) {
            chunk.dirty = true;
            chunk.last_access_tick = self.tick;
        }
    }

    pub fn save_chunk(&mut self, pos: ChunkPos) -> bool {
        let Some(chunk) = self.chunks.get_mut(&pos) else {
            return false;
        };
        self.saved.insert(pos, chunk.chunk.clone());
        chunk.dirty = false;
        self.events.push_back(ChunkLifecycleEvent::Saved(pos));
        true
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) -> bool {
        if self.forced.contains(&pos) {
            return false;
        }
        if self.chunks.get(&pos).is_some_and(|chunk| chunk.dirty) {
            self.save_chunk(pos);
        }
        let removed = self.chunks.remove(&pos).is_some();
        if removed {
            self.events.push_back(ChunkLifecycleEvent::Unloaded(pos));
        }
        removed
    }

    pub fn force_chunk(&mut self, pos: ChunkPos, forced: bool) {
        if forced {
            self.forced.insert(pos);
        } else {
            self.forced.remove(&pos);
        }
    }

    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    pub fn saved_chunk(&self, pos: ChunkPos) -> Option<&LevelChunk> {
        self.saved.get(&pos)
    }

    pub fn drain_events(&mut self) -> Vec<ChunkLifecycleEvent> {
        self.events.drain(..).collect()
    }

    fn advance_status(&mut self, pos: ChunkPos, target_status: &str) -> Result<(), String> {
        let target = chunk_status(target_status)
            .ok_or_else(|| format!("unknown chunk status {target_status}"))?;
        let Some(chunk) = self.chunks.get_mut(&pos) else {
            return Err(format!("chunk {pos:?} is not loaded"));
        };
        if chunk_status_is_or_after(&chunk.chunk.status, target.id) == Some(true) {
            chunk.last_access_tick = self.tick;
            return Ok(());
        }
        for status in crate::storage::chunk::CHUNK_STATUS_PIPELINE {
            if status.index
                <= chunk_status(&chunk.chunk.status)
                    .map(|s| s.index)
                    .unwrap_or(0)
            {
                continue;
            }
            chunk.chunk.status = status.id.to_string();
            chunk.dirty = true;
            self.events.push_back(ChunkLifecycleEvent::Generated {
                pos,
                status: status.id,
            });
            if status.index == target.index {
                break;
            }
        }
        chunk.last_access_tick = self.tick;
        Ok(())
    }

    fn evict_if_needed(&mut self) {
        while self.chunks.len() > self.capacity {
            let Some((&pos, _)) = self
                .chunks
                .iter()
                .filter(|(pos, _)| !self.forced.contains(pos))
                .min_by_key(|(_, chunk)| (chunk.last_access_tick, chunk.loaded_tick))
            else {
                break;
            };
            if self.chunks.get(&pos).is_some_and(|chunk| chunk.dirty) {
                self.save_chunk(pos);
            }
            self.chunks.remove(&pos);
            self.events.push_back(ChunkLifecycleEvent::Evicted(pos));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkLifecycleEvent, ChunkManager};
    use crate::storage::chunk::LevelChunk;
    use crate::storage::region::ChunkPos;

    #[test]
    fn chunk_manager_loads_saved_chunks_and_generates_to_target_status() {
        let mut manager = ChunkManager::new(8);
        let pos = ChunkPos { x: 1, z: 2 };
        let mut saved = LevelChunk::empty(pos);
        saved.status = "minecraft:surface".to_string();
        manager.insert_saved_chunk(saved);

        let chunk = manager.load_or_generate(pos, "features").unwrap();
        assert_eq!(chunk.status, "minecraft:features");
        let events = manager.drain_events();
        assert_eq!(events[0], ChunkLifecycleEvent::Loaded(pos));
        assert!(events.contains(&ChunkLifecycleEvent::Generated {
            pos,
            status: "minecraft:carvers"
        }));
        assert!(events.contains(&ChunkLifecycleEvent::Generated {
            pos,
            status: "minecraft:features"
        }));
    }

    #[test]
    fn dirty_chunks_save_before_unload_and_remain_in_saved_store() {
        let mut manager = ChunkManager::new(8);
        let pos = ChunkPos { x: -2, z: 5 };
        manager.load_or_generate(pos, "full").unwrap();
        manager.mark_dirty(pos);
        assert!(manager.unload_chunk(pos));
        assert!(!manager.is_loaded(pos));
        assert_eq!(manager.saved_chunk(pos).unwrap().status, "minecraft:full");
        let events = manager.drain_events();
        assert!(events.contains(&ChunkLifecycleEvent::Saved(pos)));
        assert!(events.contains(&ChunkLifecycleEvent::Unloaded(pos)));
    }

    #[test]
    fn eviction_uses_lru_but_keeps_forced_chunks_loaded() {
        let mut manager = ChunkManager::new(2);
        let forced = ChunkPos { x: 0, z: 0 };
        let second = ChunkPos { x: 1, z: 0 };
        let third = ChunkPos { x: 2, z: 0 };
        manager.force_chunk(forced, true);
        manager.load_or_generate(forced, "full").unwrap();
        manager.load_or_generate(second, "full").unwrap();
        manager.load_or_generate(third, "full").unwrap();

        assert!(manager.is_loaded(forced));
        assert!(!manager.is_loaded(second));
        assert!(manager.is_loaded(third));
        assert!(manager
            .drain_events()
            .contains(&ChunkLifecycleEvent::Evicted(second)));
    }
}
