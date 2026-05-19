#![allow(dead_code)]

use std::collections::BTreeMap;

use super::nbt::Tag;
use super::region::ChunkPos;

pub const CHUNK_WIDTH: i32 = 16;
pub const SECTION_VOLUME: usize = 16 * 16 * 16;
pub const BIOME_SECTION_VOLUME: usize = 4 * 4 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlockPos {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateEntry {
    pub name: String,
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PalettedContainer {
    pub palette: Vec<Tag>,
    pub data: Option<Vec<i64>>,
    pub expected_entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightmapKind {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    ProtoChunk,
    LevelChunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkStatusEntry {
    pub id: &'static str,
    pub parent: &'static str,
    pub index: usize,
    pub chunk_type: ChunkType,
    pub heightmaps_after: &'static [HeightmapKind],
    pub task: ChunkStatusTaskKind,
    pub region_dependencies: i32,
    pub requirements: &'static [ChunkStatusRequirement],
    pub block_state_write_radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkStatusRequirement {
    pub status: &'static str,
    pub radius: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkStatusTaskKind {
    PassThrough,
    GenerateStructureStarts,
    GenerateStructureReferences,
    GenerateBiomes,
    GenerateNoise,
    GenerateSurface,
    GenerateCarvers,
    GenerateFeatures,
    InitializeLight,
    Light,
    GenerateSpawn,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSection {
    pub y: i8,
    pub block_states: Tag,
    pub biomes: Tag,
    pub block_light: Option<Vec<i8>>,
    pub sky_light: Option<Vec<i8>>,
}

pub const WORLDGEN_HEIGHTMAPS: &[HeightmapKind] =
    &[HeightmapKind::OceanFloorWg, HeightmapKind::WorldSurfaceWg];

pub const FINAL_HEIGHTMAPS: &[HeightmapKind] = &[
    HeightmapKind::OceanFloor,
    HeightmapKind::WorldSurface,
    HeightmapKind::MotionBlocking,
    HeightmapKind::MotionBlockingNoLeaves,
];

pub const NO_REQUIREMENTS: &[ChunkStatusRequirement] = &[];
pub const STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT: &[ChunkStatusRequirement] =
    &[ChunkStatusRequirement {
        status: "minecraft:structure_starts",
        radius: 8,
    }];
pub const STRUCTURE_STARTS_DISTANCE_8_AND_BIOMES_DISTANCE_1_REQUIREMENTS:
    &[ChunkStatusRequirement] = &[
    ChunkStatusRequirement {
        status: "minecraft:structure_starts",
        radius: 8,
    },
    ChunkStatusRequirement {
        status: "minecraft:biomes",
        radius: 1,
    },
];
pub const STRUCTURE_STARTS_DISTANCE_8_AND_CARVERS_DISTANCE_1_REQUIREMENTS:
    &[ChunkStatusRequirement] = &[
    ChunkStatusRequirement {
        status: "minecraft:structure_starts",
        radius: 8,
    },
    ChunkStatusRequirement {
        status: "minecraft:carvers",
        radius: 1,
    },
];
pub const INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT: &[ChunkStatusRequirement] =
    &[ChunkStatusRequirement {
        status: "minecraft:initialize_light",
        radius: 1,
    }];
pub const BIOMES_DISTANCE_1_REQUIREMENT: &[ChunkStatusRequirement] = &[ChunkStatusRequirement {
    status: "minecraft:biomes",
    radius: 1,
}];

pub const CHUNK_STATUS_PIPELINE: &[ChunkStatusEntry] = &[
    status_entry(
        "minecraft:empty",
        "minecraft:empty",
        0,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::PassThrough,
        0,
        NO_REQUIREMENTS,
        0,
    ),
    status_entry(
        "minecraft:structure_starts",
        "minecraft:empty",
        1,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateStructureStarts,
        0,
        NO_REQUIREMENTS,
        0,
    ),
    status_entry(
        "minecraft:structure_references",
        "minecraft:structure_starts",
        2,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateStructureReferences,
        8,
        STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT,
        0,
    ),
    status_entry(
        "minecraft:biomes",
        "minecraft:structure_references",
        3,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateBiomes,
        8,
        STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT,
        0,
    ),
    status_entry(
        "minecraft:noise",
        "minecraft:biomes",
        4,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateNoise,
        8,
        STRUCTURE_STARTS_DISTANCE_8_AND_BIOMES_DISTANCE_1_REQUIREMENTS,
        0,
    ),
    status_entry(
        "minecraft:surface",
        "minecraft:noise",
        5,
        ChunkType::ProtoChunk,
        WORLDGEN_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateSurface,
        8,
        STRUCTURE_STARTS_DISTANCE_8_AND_BIOMES_DISTANCE_1_REQUIREMENTS,
        0,
    ),
    status_entry(
        "minecraft:carvers",
        "minecraft:surface",
        6,
        ChunkType::ProtoChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateCarvers,
        8,
        STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT,
        0,
    ),
    status_entry(
        "minecraft:features",
        "minecraft:carvers",
        7,
        ChunkType::ProtoChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateFeatures,
        8,
        STRUCTURE_STARTS_DISTANCE_8_AND_CARVERS_DISTANCE_1_REQUIREMENTS,
        1,
    ),
    status_entry(
        "minecraft:initialize_light",
        "minecraft:features",
        8,
        ChunkType::ProtoChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::InitializeLight,
        1,
        NO_REQUIREMENTS,
        0,
    ),
    status_entry(
        "minecraft:light",
        "minecraft:initialize_light",
        9,
        ChunkType::ProtoChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::Light,
        1,
        INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT,
        0,
    ),
    status_entry(
        "minecraft:spawn",
        "minecraft:light",
        10,
        ChunkType::ProtoChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::GenerateSpawn,
        1,
        BIOMES_DISTANCE_1_REQUIREMENT,
        0,
    ),
    status_entry(
        "minecraft:full",
        "minecraft:spawn",
        11,
        ChunkType::LevelChunk,
        FINAL_HEIGHTMAPS,
        ChunkStatusTaskKind::Full,
        0,
        NO_REQUIREMENTS,
        0,
    ),
];

const fn status_entry(
    id: &'static str,
    parent: &'static str,
    index: usize,
    chunk_type: ChunkType,
    heightmaps_after: &'static [HeightmapKind],
    task: ChunkStatusTaskKind,
    region_dependencies: i32,
    requirements: &'static [ChunkStatusRequirement],
    block_state_write_radius: i32,
) -> ChunkStatusEntry {
    ChunkStatusEntry {
        id,
        parent,
        index,
        chunk_type,
        heightmaps_after,
        task,
        region_dependencies,
        requirements,
        block_state_write_radius,
    }
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

impl SectionBlockPos {
    pub fn new(x: u8, y: u8, z: u8) -> Option<Self> {
        if x < 16 && y < 16 && z < 16 {
            Some(Self { x, y, z })
        } else {
            None
        }
    }

    pub fn block_state_index(self) -> usize {
        self.y as usize * 16 * 16 + self.z as usize * 16 + self.x as usize
    }

    pub fn biome_index(self) -> usize {
        (self.y as usize / 4) * 4 * 4 + (self.z as usize / 4) * 4 + (self.x as usize / 4)
    }
}

impl BlockStateEntry {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn to_nbt(&self) -> Tag {
        let mut fields = vec![("Name".to_string(), Tag::String(self.name.clone()))];
        if !self.properties.is_empty() {
            fields.push((
                "Properties".to_string(),
                Tag::Compound(
                    self.properties
                        .iter()
                        .map(|(key, value)| (key.clone(), Tag::String(value.clone())))
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }
}

pub fn palette_bits_for_size(palette_len: usize) -> usize {
    let needed = usize::BITS as usize - (palette_len.saturating_sub(1)).leading_zeros() as usize;
    needed.max(4)
}

pub fn pack_palette_indices(indices: &[u64], bits_per_entry: usize) -> Vec<i64> {
    let values_per_long = 64 / bits_per_entry;
    let mut packed = vec![0_u64; indices.len().div_ceil(values_per_long)];
    for (i, &value) in indices.iter().enumerate() {
        let word = i / values_per_long;
        let bit = (i % values_per_long) * bits_per_entry;
        packed[word] |= value << bit;
    }
    packed.into_iter().map(|w| w as i64).collect()
}

pub fn unpack_palette_indices(data: &[i64], bits_per_entry: usize, count: usize) -> Vec<u64> {
    let values_per_long = 64 / bits_per_entry;
    let mask = (1_u64 << bits_per_entry) - 1;
    let mut indices = vec![0_u64; count];
    for i in 0..count {
        let word = i / values_per_long;
        let bit = (i % values_per_long) * bits_per_entry;
        if word < data.len() {
            indices[i] = (data[word] as u64 >> bit) & mask;
        }
    }
    indices
}

impl PalettedContainer {
    pub fn single(entry: Tag, expected_entries: usize) -> Self {
        Self {
            palette: vec![entry],
            data: None,
            expected_entries,
        }
    }

    pub fn to_nbt(&self) -> Tag {
        let mut fields = vec![("palette".to_string(), Tag::List(self.palette.clone()))];
        if let Some(data) = &self.data {
            fields.push(("data".to_string(), Tag::LongArray(data.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn from_nbt(tag: &Tag, expected_entries: usize) -> Result<Self, String> {
        let compound = compound(tag)?;
        let palette = list_field(compound, "palette")?.to_vec();
        if palette.is_empty() {
            return Err("paletted container palette cannot be empty".to_string());
        }
        let data = match compound.iter().find(|(field_name, _)| field_name == "data") {
            Some((_name, Tag::LongArray(values))) => Some(values.clone()),
            Some((_name, _)) => {
                return Err("paletted container data must be a long array".to_string())
            }
            None => None,
        };
        Ok(Self {
            palette,
            data,
            expected_entries,
        })
    }

    pub fn get_entry(&self, index: usize) -> Option<&Tag> {
        if self.palette.len() == 1 {
            return self.palette.first();
        }
        let bits = palette_bits_for_size(self.palette.len());
        let indices = unpack_palette_indices(
            self.data.as_deref().unwrap_or(&[]),
            bits,
            self.expected_entries,
        );
        let palette_idx = *indices.get(index)? as usize;
        self.palette.get(palette_idx)
    }

    pub fn set_entry(&mut self, index: usize, entry: Tag) {
        let palette_idx = match self.palette.iter().position(|e| e == &entry) {
            Some(i) => i,
            None => {
                self.palette.push(entry);
                self.palette.len() - 1
            }
        };
        if self.palette.len() == 1 {
            self.data = None;
            return;
        }
        let bits = palette_bits_for_size(self.palette.len());
        let count = self.expected_entries;
        let mut indices = match &self.data {
            Some(d) => {
                let old_bits = palette_bits_for_size(self.palette.len().saturating_sub(1).max(1));
                unpack_palette_indices(d, old_bits, count)
            }
            None => vec![0_u64; count],
        };
        if index < indices.len() {
            indices[index] = palette_idx as u64;
        }
        self.data = Some(pack_palette_indices(&indices, bits));
    }
}

impl LevelChunk {
    pub fn get_block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<String> {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let section = self.sections.iter().find(|s| s.y == section_y)?;
        let container = PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME).ok()?;
        let entry = container.get_entry(index)?;
        if let Tag::Compound(fields) = entry {
            fields.iter().find(|(k, _)| k == "Name").and_then(|(_, v)| {
                if let Tag::String(name) = v {
                    Some(name.clone())
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub fn set_block_state(&mut self, world_x: i32, world_y: i32, world_z: i32, block_name: &str) {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;

        let entry = Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String(block_name.to_string()),
        )]);

        if let Some(section) = self.sections.iter_mut().find(|s| s.y == section_y) {
            if let Ok(mut container) =
                PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
            {
                container.set_entry(index, entry);
                section.block_states = container.to_nbt();
            }
        }
    }
}

impl HeightmapKind {
    pub fn storage_name(self) -> &'static str {
        match self {
            Self::WorldSurfaceWg => "WORLD_SURFACE_WG",
            Self::WorldSurface => "WORLD_SURFACE",
            Self::OceanFloorWg => "OCEAN_FLOOR_WG",
            Self::OceanFloor => "OCEAN_FLOOR",
            Self::MotionBlocking => "MOTION_BLOCKING",
            Self::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
        }
    }
}

pub fn chunk_status(id: &str) -> Option<&'static ChunkStatusEntry> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    CHUNK_STATUS_PIPELINE.iter().find(|status| {
        status
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|status_name| status_name == name)
    })
}

pub fn chunk_status_is_or_after(status: &str, required: &str) -> Option<bool> {
    Some(chunk_status(status)?.index >= chunk_status(required)?.index)
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
    use super::{
        chunk_status, chunk_status_is_or_after, BlockStateEntry, ChunkSection, ChunkStatusTaskKind,
        ChunkType, HeightmapKind, LevelChunk, PalettedContainer, SectionBlockPos,
        BIOME_SECTION_VOLUME, CHUNK_STATUS_PIPELINE, FINAL_HEIGHTMAPS, SECTION_VOLUME,
        WORLDGEN_HEIGHTMAPS,
    };
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

    #[test]
    fn section_positions_and_palettes_match_vanilla_shapes() {
        assert_eq!(SECTION_VOLUME, 4096);
        assert_eq!(BIOME_SECTION_VOLUME, 64);
        assert!(SectionBlockPos::new(16, 0, 0).is_none());

        let pos = SectionBlockPos::new(3, 5, 7).unwrap();
        assert_eq!(pos.block_state_index(), 5 * 256 + 7 * 16 + 3);
        assert_eq!(pos.biome_index(), 1 * 16 + 1 * 4);

        let mut oak = BlockStateEntry::new("minecraft:oak_log");
        oak.properties.insert("axis".to_string(), "y".to_string());
        let block_states = PalettedContainer::single(oak.to_nbt(), SECTION_VOLUME);
        let decoded = PalettedContainer::from_nbt(&block_states.to_nbt(), SECTION_VOLUME).unwrap();
        assert_eq!(decoded.palette.len(), 1);
        assert!(decoded.data.is_none());
        assert_eq!(decoded.expected_entries, SECTION_VOLUME);
    }

    #[test]
    fn chunk_status_pipeline_matches_vanilla_order_and_dependencies() {
        assert_eq!(CHUNK_STATUS_PIPELINE.len(), 12);
        assert_eq!(CHUNK_STATUS_PIPELINE[0].id, "minecraft:empty");
        assert_eq!(CHUNK_STATUS_PIPELINE[0].parent, "minecraft:empty");
        assert_eq!(CHUNK_STATUS_PIPELINE[11].id, "minecraft:full");
        assert_eq!(CHUNK_STATUS_PIPELINE[11].parent, "minecraft:spawn");
        assert_eq!(CHUNK_STATUS_PIPELINE[11].chunk_type, ChunkType::LevelChunk);
        assert_eq!(chunk_status("full").unwrap().index, 11);
        assert_eq!(
            chunk_status("minecraft:carvers").unwrap().parent,
            "minecraft:surface"
        );
        assert_eq!(chunk_status_is_or_after("features", "carvers"), Some(true));
        assert_eq!(chunk_status_is_or_after("noise", "features"), Some(false));

        assert_eq!(
            WORLDGEN_HEIGHTMAPS
                .iter()
                .map(|kind| kind.storage_name())
                .collect::<Vec<_>>(),
            vec!["OCEAN_FLOOR_WG", "WORLD_SURFACE_WG"]
        );
        assert_eq!(
            FINAL_HEIGHTMAPS
                .iter()
                .map(|kind| kind.storage_name())
                .collect::<Vec<_>>(),
            vec![
                "OCEAN_FLOOR",
                "WORLD_SURFACE",
                "MOTION_BLOCKING",
                "MOTION_BLOCKING_NO_LEAVES"
            ]
        );
        assert_eq!(
            chunk_status("features").unwrap().heightmaps_after,
            FINAL_HEIGHTMAPS
        );
        assert_eq!(
            chunk_status("structure_starts").unwrap().task,
            ChunkStatusTaskKind::GenerateStructureStarts
        );
        assert_eq!(
            chunk_status("structure_references").unwrap().task,
            ChunkStatusTaskKind::GenerateStructureReferences
        );
        assert_eq!(
            chunk_status("structure_references")
                .unwrap()
                .region_dependencies,
            8
        );
        assert_eq!(
            chunk_status("structure_references").unwrap().requirements,
            super::STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT
        );
        assert_eq!(
            chunk_status("biomes").unwrap().task,
            ChunkStatusTaskKind::GenerateBiomes
        );
        assert_eq!(chunk_status("biomes").unwrap().region_dependencies, 8);
        assert_eq!(
            chunk_status("biomes").unwrap().requirements,
            super::STRUCTURE_STARTS_DISTANCE_8_REQUIREMENT
        );
        assert_eq!(
            chunk_status("noise").unwrap().requirements,
            super::STRUCTURE_STARTS_DISTANCE_8_AND_BIOMES_DISTANCE_1_REQUIREMENTS
        );
        assert_eq!(chunk_status("noise").unwrap().block_state_write_radius, 0);
        assert_eq!(
            chunk_status("features").unwrap().task,
            ChunkStatusTaskKind::GenerateFeatures
        );
        assert_eq!(
            chunk_status("features").unwrap().requirements,
            super::STRUCTURE_STARTS_DISTANCE_8_AND_CARVERS_DISTANCE_1_REQUIREMENTS
        );
        assert_eq!(
            chunk_status("features").unwrap().block_state_write_radius,
            1
        );
        assert_eq!(
            chunk_status("light").unwrap().requirements,
            super::INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT
        );
        assert_eq!(
            chunk_status("spawn").unwrap().requirements,
            super::BIOMES_DISTANCE_1_REQUIREMENT
        );
        assert_eq!(
            chunk_status("full").unwrap().task,
            ChunkStatusTaskKind::Full
        );
        assert_eq!(chunk_status("full").unwrap().region_dependencies, 0);
        assert_eq!(
            chunk_status("biomes").unwrap().heightmaps_after,
            WORLDGEN_HEIGHTMAPS
        );
        assert_eq!(
            HeightmapKind::MotionBlockingNoLeaves.storage_name(),
            "MOTION_BLOCKING_NO_LEAVES"
        );
    }
}
