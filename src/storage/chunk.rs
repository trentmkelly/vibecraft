#![allow(dead_code)]

use std::collections::BTreeMap;

use super::nbt::Tag;
use super::region::ChunkPos;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSection {
    pub y: i8,
    pub block_states: Tag,
    pub biomes: Tag,
    pub block_light: Option<Vec<i8>>,
    pub sky_light: Option<Vec<i8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelChunk {
    pub pos: ChunkPos,
    pub status: String,
    pub inhabited_time: i64,
    pub sections: Vec<ChunkSection>,
    pub heightmaps: BTreeMap<String, Tag>,
    pub block_entities: Vec<Tag>,
    pub entities: Vec<Tag>,
    pub structures: Tag,
    pub block_ticks: Vec<Tag>,
    pub fluid_ticks: Vec<Tag>,
    pub post_processing: Vec<Tag>,
}

impl LevelChunk {
    pub fn empty(pos: ChunkPos) -> Self {
        Self {
            pos,
            status: "minecraft:empty".to_string(),
            inhabited_time: 0,
            sections: Vec::new(),
            heightmaps: BTreeMap::new(),
            block_entities: Vec::new(),
            entities: Vec::new(),
            structures: Tag::Compound(Vec::new()),
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            post_processing: Vec::new(),
        }
    }

    pub fn to_nbt(&self, data_version: i32) -> Tag {
        Tag::Compound(vec![
            ("DataVersion".to_string(), Tag::Int(data_version)),
            ("xPos".to_string(), Tag::Int(self.pos.x)),
            ("zPos".to_string(), Tag::Int(self.pos.z)),
            ("Status".to_string(), Tag::String(self.status.clone())),
            ("InhabitedTime".to_string(), Tag::Long(self.inhabited_time)),
            (
                "sections".to_string(),
                Tag::List(self.sections.iter().map(ChunkSection::to_nbt).collect()),
            ),
            (
                "Heightmaps".to_string(),
                Tag::Compound(heightmap_fields(self)),
            ),
            (
                "block_entities".to_string(),
                Tag::List(self.block_entities.clone()),
            ),
            ("entities".to_string(), Tag::List(self.entities.clone())),
            ("structures".to_string(), self.structures.clone()),
            (
                "block_ticks".to_string(),
                Tag::List(self.block_ticks.clone()),
            ),
            (
                "fluid_ticks".to_string(),
                Tag::List(self.fluid_ticks.clone()),
            ),
            (
                "PostProcessing".to_string(),
                Tag::List(self.post_processing.clone()),
            ),
        ])
    }

    pub fn from_nbt(expected_pos: ChunkPos, tag: &Tag) -> Result<Self, String> {
        let root = compound(tag)?;
        let pos = ChunkPos {
            x: int_field(root, "xPos")?,
            z: int_field(root, "zPos")?,
        };
        if pos != expected_pos {
            return Err(format!(
                "chunk stored at wrong position: expected {:?}, got {:?}",
                expected_pos, pos
            ));
        }

        Ok(Self {
            pos,
            status: string_field(root, "Status")?.to_string(),
            inhabited_time: long_field(root, "InhabitedTime")?,
            sections: list_field(root, "sections")?
                .iter()
                .map(ChunkSection::from_nbt)
                .collect::<Result<Vec<_>, _>>()?,
            heightmaps: compound(field(root, "Heightmaps")?)?
                .iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
            block_entities: list_field(root, "block_entities")?.to_vec(),
            entities: list_field(root, "entities")?.to_vec(),
            structures: field(root, "structures")?.clone(),
            block_ticks: list_field(root, "block_ticks")?.to_vec(),
            fluid_ticks: list_field(root, "fluid_ticks")?.to_vec(),
            post_processing: list_field(root, "PostProcessing")?.to_vec(),
        })
    }
}

impl ChunkSection {
    pub fn to_nbt(&self) -> Tag {
        let mut values = vec![
            ("Y".to_string(), Tag::Byte(self.y)),
            ("block_states".to_string(), self.block_states.clone()),
            ("biomes".to_string(), self.biomes.clone()),
        ];
        if let Some(block_light) = &self.block_light {
            values.push((
                "BlockLight".to_string(),
                Tag::ByteArray(block_light.clone()),
            ));
        }
        if let Some(sky_light) = &self.sky_light {
            values.push(("SkyLight".to_string(), Tag::ByteArray(sky_light.clone())));
        }
        Tag::Compound(values)
    }

    pub fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let compound = compound(tag)?;
        Ok(Self {
            y: byte_field(compound, "Y")?,
            block_states: field(compound, "block_states")?.clone(),
            biomes: field(compound, "biomes")?.clone(),
            block_light: optional_byte_array(compound, "BlockLight")?,
            sky_light: optional_byte_array(compound, "SkyLight")?,
        })
    }
}

fn heightmap_fields(chunk: &LevelChunk) -> Vec<(String, Tag)> {
    chunk
        .heightmaps
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
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

fn byte_field(compound: &[(String, Tag)], name: &str) -> Result<i8, String> {
    match field(compound, name)? {
        Tag::Byte(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be a byte")),
    }
}

fn int_field(compound: &[(String, Tag)], name: &str) -> Result<i32, String> {
    match field(compound, name)? {
        Tag::Int(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be an int")),
    }
}

fn long_field(compound: &[(String, Tag)], name: &str) -> Result<i64, String> {
    match field(compound, name)? {
        Tag::Long(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be a long")),
    }
}

fn string_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a str, String> {
    match field(compound, name)? {
        Tag::String(value) => Ok(value),
        _ => Err(format!("NBT field {name} must be a string")),
    }
}

fn list_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Result<&'a [Tag], String> {
    match field(compound, name)? {
        Tag::List(values) => Ok(values),
        _ => Err(format!("NBT field {name} must be a list")),
    }
}

fn optional_byte_array(compound: &[(String, Tag)], name: &str) -> Result<Option<Vec<i8>>, String> {
    match compound.iter().find(|(field_name, _)| field_name == name) {
        Some((_name, Tag::ByteArray(values))) => Ok(Some(values.clone())),
        Some((_name, _)) => Err(format!("NBT field {name} must be a byte array")),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{ChunkSection, LevelChunk};
    use crate::storage::datafix::TARGET_DATA_VERSION;
    use crate::storage::nbt::Tag;
    use crate::storage::region::ChunkPos;
    use std::collections::BTreeMap;

    #[test]
    fn level_chunk_round_trips_vanilla_storage_sections_and_side_payloads() {
        let pos = ChunkPos { x: 4, z: -2 };
        let mut heightmaps = BTreeMap::new();
        heightmaps.insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1, 2, 3]));
        let chunk = LevelChunk {
            pos,
            status: "minecraft:full".to_string(),
            inhabited_time: 42,
            sections: vec![ChunkSection {
                y: 0,
                block_states: Tag::Compound(vec![("palette".to_string(), Tag::List(Vec::new()))]),
                biomes: Tag::Compound(vec![("palette".to_string(), Tag::List(Vec::new()))]),
                block_light: Some(vec![0; 2048]),
                sky_light: Some(vec![15; 2048]),
            }],
            heightmaps,
            block_entities: vec![Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:chest".to_string()),
            )])],
            entities: vec![Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:pig".to_string()),
            )])],
            structures: Tag::Compound(vec![("starts".to_string(), Tag::Compound(Vec::new()))]),
            block_ticks: vec![Tag::Compound(vec![(
                "i".to_string(),
                Tag::String("minecraft:stone".to_string()),
            )])],
            fluid_ticks: vec![Tag::Compound(vec![(
                "i".to_string(),
                Tag::String("minecraft:water".to_string()),
            )])],
            post_processing: vec![Tag::List(vec![Tag::Short(1), Tag::Short(2)])],
        };

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.pos, pos);
        assert_eq!(decoded.status, "minecraft:full");
        assert_eq!(
            decoded.sections[0].block_light.as_ref().unwrap().len(),
            2048
        );
        assert_eq!(decoded.sections[0].sky_light.as_ref().unwrap()[0], 15);
        assert!(decoded.heightmaps.contains_key("WORLD_SURFACE"));
        assert_eq!(decoded.block_entities.len(), 1);
        assert_eq!(decoded.entities.len(), 1);
        assert_eq!(decoded.block_ticks.len(), 1);
        assert_eq!(decoded.fluid_ticks.len(), 1);
        assert_eq!(decoded.post_processing.len(), 1);
    }

    #[test]
    fn level_chunk_rejects_misplaced_payloads() {
        let tag = LevelChunk::empty(ChunkPos { x: 9, z: 9 }).to_nbt(TARGET_DATA_VERSION);
        let err = LevelChunk::from_nbt(ChunkPos { x: 0, z: 0 }, &tag).unwrap_err();
        assert!(err.contains("wrong position"));
    }
}
