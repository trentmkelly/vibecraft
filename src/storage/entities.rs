#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::datafix::require_current_tag_data_version;
use super::nbt::Tag;
use super::region::ChunkPos;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredEntity {
    pub uuid: String,
    pub entity_type: String,
    pub data: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkEntities {
    pub pos: ChunkPos,
    pub entities: Vec<StoredEntity>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityStorage {
    chunks: BTreeMap<ChunkPos, Tag>,
    empty_chunks: BTreeSet<ChunkPos>,
}

impl StoredEntity {
    pub fn to_nbt(&self) -> Tag {
        let mut values = vec![
            ("UUID".to_string(), Tag::String(self.uuid.clone())),
            ("id".to_string(), Tag::String(self.entity_type.clone())),
        ];
        if let Tag::Compound(extra) = &self.data {
            values.extend(extra.clone());
        } else {
            values.push(("data".to_string(), self.data.clone()));
        }
        Tag::Compound(values)
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let compound = compound(tag)?;
        let uuid = string_field(compound, "UUID")?.to_string();
        let entity_type = string_field(compound, "id")?.to_string();
        Ok(Self {
            uuid,
            entity_type,
            data: tag.clone(),
        })
    }
}

impl ChunkEntities {
    pub fn empty(pos: ChunkPos) -> Self {
        Self {
            pos,
            entities: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn to_nbt(&self, data_version: i32) -> Tag {
        Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(data_version)),
            (
                "Position".to_string(),
                Tag::IntArray(vec![self.pos.x, self.pos.z]),
            ),
            (
                "Entities".to_string(),
                Tag::List(self.entities.iter().map(StoredEntity::to_nbt).collect()),
            ),
        ])
    }

    pub fn from_nbt(expected_pos: ChunkPos, tag: &Tag) -> Result<Self, String> {
        require_current_tag_data_version("entity chunk", tag)?;
        let compound = compound(tag)?;
        let stored_pos = match field(compound, "Position")? {
            Tag::IntArray(values) if values.len() == 2 => ChunkPos {
                x: values[0],
                z: values[1],
            },
            _ => return Err("entity chunk Position must be an int array of length 2".to_string()),
        };
        if stored_pos != expected_pos {
            return Err(format!(
                "entity chunk stored at wrong position: expected {:?}, got {:?}",
                expected_pos, stored_pos
            ));
        }
        let entities = match field(compound, "Entities")? {
            Tag::List(values) => values
                .iter()
                .map(StoredEntity::from_nbt)
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err("entity chunk Entities must be a list".to_string()),
        };
        Ok(Self {
            pos: expected_pos,
            entities,
        })
    }
}

impl EntityStorage {
    pub fn load_entities(&mut self, pos: ChunkPos) -> Result<ChunkEntities, String> {
        if self.empty_chunks.contains(&pos) {
            return Ok(ChunkEntities::empty(pos));
        }
        match self.chunks.get(&pos) {
            Some(tag) => ChunkEntities::from_nbt(pos, tag),
            None => {
                self.empty_chunks.insert(pos);
                Ok(ChunkEntities::empty(pos))
            }
        }
    }

    pub fn store_entities(&mut self, chunk: ChunkEntities, data_version: i32) {
        if chunk.is_empty() {
            self.chunks.remove(&chunk.pos);
            self.empty_chunks.insert(chunk.pos);
        } else {
            self.empty_chunks.remove(&chunk.pos);
            self.chunks.insert(chunk.pos, chunk.to_nbt(data_version));
        }
    }

    pub fn raw_chunk(&self, pos: ChunkPos) -> Option<&Tag> {
        self.chunks.get(&pos)
    }

    pub fn is_marked_empty(&self, pos: ChunkPos) -> bool {
        self.empty_chunks.contains(&pos)
    }
}

fn compound(tag: &Tag) -> Result<&[(String, Tag)], String> {
    match tag {
        Tag::Compound(values) => Ok(values),
        _ => Err("expected NBT compound".to_string()),
    }
}

fn field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a Tag, String> {
    compound
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| format!("missing NBT field {name}"))
}

fn string_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a str, String> {
    match field(compound, name)? {
        Tag::String(value) => Ok(value),
        _ => Err(format!("NBT field {name} must be a string")),
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkEntities, EntityStorage, StoredEntity};
    use crate::storage::datafix::TARGET_DATA_VERSION;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;

    fn pig(uuid: &str) -> StoredEntity {
        StoredEntity {
            uuid: uuid.to_string(),
            entity_type: "minecraft:pig".to_string(),
            data: Tag::Compound(vec![("Health".to_string(), Tag::Float(10.0))]),
        }
    }

    #[test]
    fn entity_chunk_round_trips_vanilla_nbt_shape() {
        let pos = ChunkPos { x: 3, z: -4 };
        let chunk = ChunkEntities {
            pos,
            entities: vec![pig("00000000-0000-0000-0000-000000000001")],
        };

        let decoded = ChunkEntities::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.pos, pos);
        assert_eq!(decoded.entities[0].entity_type, "minecraft:pig");
        assert_eq!(
            decoded.entities[0].uuid,
            "00000000-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn entity_storage_marks_and_reuses_empty_chunks() {
        let mut storage = EntityStorage::default();
        let pos = ChunkPos { x: 0, z: 0 };

        assert!(storage.load_entities(pos).unwrap().is_empty());
        assert!(storage.is_marked_empty(pos));

        storage.store_entities(
            ChunkEntities {
                pos,
                entities: vec![pig("00000000-0000-0000-0000-000000000002")],
            },
            TARGET_DATA_VERSION,
        );
        assert!(!storage.is_marked_empty(pos));
        assert!(storage.raw_chunk(pos).is_some());

        storage.store_entities(ChunkEntities::empty(pos), TARGET_DATA_VERSION);
        assert!(storage.is_marked_empty(pos));
        assert!(storage.raw_chunk(pos).is_none());
    }

    #[test]
    fn entity_storage_rejects_misplaced_chunk_payloads() {
        let pos = ChunkPos { x: 1, z: 2 };
        let wrong = ChunkEntities {
            pos: ChunkPos { x: 9, z: 9 },
            entities: vec![pig("00000000-0000-0000-0000-000000000003")],
        }
        .to_nbt(TARGET_DATA_VERSION);

        let err = ChunkEntities::from_nbt(pos, &wrong).unwrap_err();

        assert!(err.contains("wrong position"));
    }

    #[test]
    fn entity_chunks_reject_missing_or_unsupported_data_versions() {
        let pos = ChunkPos { x: 1, z: 2 };
        let mut missing = ChunkEntities::empty(pos).to_nbt(TARGET_DATA_VERSION);
        if let Tag::Compound(values) = &mut missing {
            values.retain(|(name, _)| name != "DataVersion");
        }
        let err = ChunkEntities::from_nbt(pos, &missing).unwrap_err();
        assert!(err.contains("missing DataVersion"));

        let unsupported = ChunkEntities::empty(pos).to_nbt(TARGET_DATA_VERSION - 1);
        let err = ChunkEntities::from_nbt(pos, &unsupported).unwrap_err();
        assert!(err.contains("Unsupported world DataVersion"));
    }
}
