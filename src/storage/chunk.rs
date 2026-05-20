#![allow(dead_code)]

use std::collections::BTreeMap;

use super::datafix::require_current_tag_data_version;
use super::nbt::Tag;
use super::region::ChunkPos;
use crate::worldgen::{validate_blending_data_packed, BlendingDataPacked};

pub const CHUNK_WIDTH: i32 = 16;
pub const SECTION_VOLUME: usize = 16 * 16 * 16;
pub const BIOME_SECTION_VOLUME: usize = 4 * 4 * 4;
pub const LIGHT_DATA_LAYER_LENGTH: usize = 2048;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickPriority {
    ExtremelyHigh,
    VeryHigh,
    High,
    Normal,
    Low,
    VeryLow,
    ExtremelyLow,
}

impl TickPriority {
    pub fn value(self) -> i32 {
        match self {
            Self::ExtremelyHigh => -3,
            Self::VeryHigh => -2,
            Self::High => -1,
            Self::Normal => 0,
            Self::Low => 1,
            Self::VeryLow => 2,
            Self::ExtremelyLow => 3,
        }
    }
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
    pub min_section_y: i32,
    pub last_update: i64,
    pub status: String,
    pub inhabited_time: i64,
    pub sections: Vec<ChunkSection>,
    pub heightmaps: BTreeMap<String, Tag>,
    pub block_entities: Vec<Tag>,
    pub entities: Vec<Tag>,
    pub structures: Tag,
    pub upgrade_data: Option<Tag>,
    pub blending_data: Option<Tag>,
    pub below_zero_retrogen: Option<Tag>,
    pub carving_mask: Option<Vec<i64>>,
    pub block_ticks: Vec<Tag>,
    pub fluid_ticks: Vec<Tag>,
    pub post_processing: Vec<Tag>,
    pub light_correct: bool,
}

impl LevelChunk {
    pub fn empty(pos: ChunkPos) -> Self {
        Self {
            pos,
            min_section_y: 0,
            last_update: 0,
            status: "minecraft:empty".to_string(),
            inhabited_time: 0,
            sections: Vec::new(),
            heightmaps: BTreeMap::new(),
            block_entities: Vec::new(),
            entities: Vec::new(),
            structures: empty_structures_payload(),
            upgrade_data: None,
            blending_data: None,
            below_zero_retrogen: None,
            carving_mask: None,
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            post_processing: Vec::new(),
            light_correct: false,
        }
    }

    pub fn to_nbt(&self, data_version: i32) -> Tag {
        let mut fields = vec![
            ("DataVersion".to_string(), Tag::Int(data_version)),
            ("xPos".to_string(), Tag::Int(self.pos.x)),
            ("yPos".to_string(), Tag::Int(self.min_section_y)),
            ("zPos".to_string(), Tag::Int(self.pos.z)),
            ("LastUpdate".to_string(), Tag::Long(self.last_update)),
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
        ];
        if let Some(upgrade_data) = &self.upgrade_data {
            fields.push(("UpgradeData".to_string(), upgrade_data.clone()));
        }
        if let Some(blending_data) = &self.blending_data {
            fields.push(("blending_data".to_string(), blending_data.clone()));
        }
        if let Some(below_zero_retrogen) = &self.below_zero_retrogen {
            fields.push((
                "below_zero_retrogen".to_string(),
                below_zero_retrogen.clone(),
            ));
        }
        if self.has_proto_only_storage_fields() {
            fields.push(("entities".to_string(), Tag::List(self.entities.clone())));
        }
        if self.has_proto_only_storage_fields() {
            if let Some(carving_mask) = &self.carving_mask {
                fields.push((
                    "carving_mask".to_string(),
                    Tag::LongArray(carving_mask.clone()),
                ));
            }
        }
        if self.light_correct {
            fields.push(("isLightOn".to_string(), Tag::Byte(1)));
        }
        Tag::Compound(fields)
    }

    pub fn mark_pos_for_postprocessing(&mut self, x: i32, y: i32, z: i32) -> bool {
        let section_y = y.div_euclid(16);
        let section_index = section_y - self.min_section_y;
        if section_index < 0 || section_index as usize >= self.sections.len() {
            return false;
        }
        let section_index = section_index as usize;
        while self.post_processing.len() <= section_index {
            self.post_processing.push(Tag::List(Vec::new()));
        }
        let packed = pack_postprocessing_offset(x, y, z);
        match &mut self.post_processing[section_index] {
            Tag::List(offsets) => offsets.push(Tag::Short(packed)),
            _ => self.post_processing[section_index] = Tag::List(vec![Tag::Short(packed)]),
        }
        true
    }

    pub fn set_block_entity_nbt(&mut self, entity_tag: Tag) -> bool {
        let Some(pos) = block_entity_tag_pos(&entity_tag) else {
            return false;
        };
        if let Some(existing) = self
            .block_entities
            .iter_mut()
            .find(|tag| block_entity_tag_pos(tag) == Some(pos))
        {
            *existing = entity_tag;
        } else {
            self.block_entities.push(entity_tag);
        }
        true
    }

    pub fn add_entity_nbt(&mut self, entity_tag: Tag) -> bool {
        if !matches!(entity_tag, Tag::Compound(_)) {
            return false;
        }
        self.entities.push(entity_tag);
        true
    }

    pub fn schedule_block_tick(
        &mut self,
        id: impl Into<String>,
        x: i32,
        y: i32,
        z: i32,
        delay: i32,
        priority: TickPriority,
    ) -> bool {
        if !self.contains_block_pos(x, z) {
            return false;
        }
        self.block_ticks
            .push(saved_tick_tag(id.into(), x, y, z, delay, priority));
        true
    }

    pub fn schedule_fluid_tick(
        &mut self,
        id: impl Into<String>,
        x: i32,
        y: i32,
        z: i32,
        delay: i32,
        priority: TickPriority,
    ) -> bool {
        if !self.contains_block_pos(x, z) {
            return false;
        }
        self.fluid_ticks
            .push(saved_tick_tag(id.into(), x, y, z, delay, priority));
        true
    }

    pub fn set_section_light_arrays(
        &mut self,
        section_y: i8,
        block_light: Option<Vec<i8>>,
        sky_light: Option<Vec<i8>>,
    ) -> bool {
        if !block_light
            .as_ref()
            .is_none_or(|light| light.len() == LIGHT_DATA_LAYER_LENGTH)
            || !sky_light
                .as_ref()
                .is_none_or(|light| light.len() == LIGHT_DATA_LAYER_LENGTH)
        {
            return false;
        }
        let Some(section) = self
            .sections
            .iter_mut()
            .find(|section| section.y == section_y)
        else {
            return false;
        };
        section.block_light = block_light;
        section.sky_light = sky_light;
        self.light_correct = self
            .sections
            .iter()
            .all(|section| section.block_light.is_some() || section.sky_light.is_some());
        true
    }

    fn contains_block_pos(&self, x: i32, z: i32) -> bool {
        x.div_euclid(CHUNK_WIDTH) == self.pos.x && z.div_euclid(CHUNK_WIDTH) == self.pos.z
    }

    fn has_proto_only_storage_fields(&self) -> bool {
        chunk_status(&self.status).is_some_and(|status| status.chunk_type == ChunkType::ProtoChunk)
    }

    pub fn from_nbt(expected_pos: ChunkPos, tag: &Tag) -> Result<Self, String> {
        require_current_tag_data_version("chunk", tag)?;
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

        let status = chunk_status_field(root)?;
        let status_heightmaps = chunk_status(&status)
            .map(|status| status.heightmaps_after)
            .unwrap_or(&[]);

        Ok(Self {
            pos,
            min_section_y: optional_int_field(root, "yPos")?.unwrap_or(0),
            last_update: optional_long_field(root, "LastUpdate")?.unwrap_or(0),
            status,
            inhabited_time: optional_long_field(root, "InhabitedTime")?.unwrap_or(0),
            sections: chunk_sections_from_list(
                optional_list_field(root, "sections")?.unwrap_or_default(),
            )?,
            heightmaps: optional_compound_field(root, "Heightmaps")?
                .unwrap_or(&[])
                .iter()
                .filter(|(name, _)| {
                    status_heightmaps
                        .iter()
                        .any(|heightmap| heightmap.storage_name() == name)
                })
                .filter(|(_, value)| matches!(value, Tag::LongArray(_)))
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
            block_entities: compound_list_entries(
                optional_list_field(root, "block_entities")?.unwrap_or_default(),
            ),
            entities: compound_list_entries(
                optional_list_field(root, "entities")?.unwrap_or_default(),
            ),
            structures: optional_field(root, "structures")
                .cloned()
                .unwrap_or_else(empty_structures_payload),
            upgrade_data: optional_field(root, "UpgradeData").cloned(),
            blending_data: optional_blending_data(root)?,
            below_zero_retrogen: optional_below_zero_retrogen(root)?,
            carving_mask: optional_long_array(root, "carving_mask")?,
            block_ticks: filter_saved_ticks_for_chunk(
                optional_list_field(root, "block_ticks")?.unwrap_or_default(),
                pos,
            ),
            fluid_ticks: filter_saved_ticks_for_chunk(
                optional_list_field(root, "fluid_ticks")?.unwrap_or_default(),
                pos,
            ),
            post_processing: post_processing_sections(
                optional_list_field(root, "PostProcessing")?.unwrap_or_default(),
            ),
            light_correct: optional_bool_field(root, "isLightOn")?.unwrap_or(false),
        })
    }
}

pub fn pack_postprocessing_offset(x: i32, y: i32, z: i32) -> i16 {
    ((x & 15) | ((y & 15) << 4) | ((z & 15) << 8)) as i16
}

fn block_entity_tag_pos(tag: &Tag) -> Option<(i32, i32, i32)> {
    let fields = compound(tag).ok()?;
    Some((
        int_field(fields, "x").ok()?,
        int_field(fields, "y").ok()?,
        int_field(fields, "z").ok()?,
    ))
}

fn saved_tick_tag(id: String, x: i32, y: i32, z: i32, delay: i32, priority: TickPriority) -> Tag {
    Tag::Compound(vec![
        ("i".to_string(), Tag::String(id)),
        ("x".to_string(), Tag::Int(x)),
        ("y".to_string(), Tag::Int(y)),
        ("z".to_string(), Tag::Int(z)),
        ("t".to_string(), Tag::Int(delay)),
        ("p".to_string(), Tag::Int(priority.value())),
    ])
}

fn empty_structures_payload() -> Tag {
    Tag::Compound(vec![
        ("starts".to_string(), Tag::Compound(Vec::new())),
        ("References".to_string(), Tag::Compound(Vec::new())),
    ])
}

fn default_block_states_container() -> Tag {
    PalettedContainer::single(
        BlockStateEntry::new("minecraft:air").to_nbt(),
        SECTION_VOLUME,
    )
    .to_nbt()
}

fn default_biomes_container() -> Tag {
    PalettedContainer::single(
        Tag::String("minecraft:plains".to_string()),
        BIOME_SECTION_VOLUME,
    )
    .to_nbt()
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
            y: optional_byte_field(compound, "Y")?.unwrap_or(0),
            block_states: optional_field(compound, "block_states")
                .cloned()
                .unwrap_or_else(default_block_states_container),
            biomes: optional_field(compound, "biomes")
                .cloned()
                .unwrap_or_else(default_biomes_container),
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

fn filter_saved_ticks_for_chunk(ticks: Vec<Tag>, pos: ChunkPos) -> Vec<Tag> {
    ticks
        .into_iter()
        .filter(|tick| saved_tick_belongs_to_chunk(tick, pos))
        .collect()
}

fn compound_list_entries(entries: Vec<Tag>) -> Vec<Tag> {
    entries
        .into_iter()
        .filter(|entry| matches!(entry, Tag::Compound(_)))
        .collect()
}

fn chunk_sections_from_list(entries: Vec<Tag>) -> Result<Vec<ChunkSection>, String> {
    entries
        .into_iter()
        .filter(|entry| matches!(entry, Tag::Compound(_)))
        .map(|entry| ChunkSection::from_nbt(&entry))
        .collect()
}

fn post_processing_sections(entries: Vec<Tag>) -> Vec<Tag> {
    entries
        .into_iter()
        .map(|entry| match entry {
            Tag::List(offsets) => Tag::List(
                offsets
                    .into_iter()
                    .map(|offset| match offset {
                        Tag::Short(value) => Tag::Short(value),
                        _ => Tag::Short(0),
                    })
                    .collect(),
            ),
            _ => Tag::List(Vec::new()),
        })
        .collect()
}

fn saved_tick_belongs_to_chunk(tick: &Tag, pos: ChunkPos) -> bool {
    let Ok(fields) = compound(tick) else {
        return false;
    };
    let Ok(x) = int_field(fields, "x") else {
        return false;
    };
    let Ok(z) = int_field(fields, "z") else {
        return false;
    };
    x.div_euclid(CHUNK_WIDTH) == pos.x && z.div_euclid(CHUNK_WIDTH) == pos.z
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

fn optional_field<'a>(compound: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    compound
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
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

fn optional_int_field(compound: &[(String, Tag)], name: &str) -> Result<Option<i32>, String> {
    match optional_field(compound, name) {
        Some(Tag::Int(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be an int")),
        None => Ok(None),
    }
}

fn optional_byte_field(compound: &[(String, Tag)], name: &str) -> Result<Option<i8>, String> {
    match optional_field(compound, name) {
        Some(Tag::Byte(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be a byte")),
        None => Ok(None),
    }
}

fn long_field(compound: &[(String, Tag)], name: &str) -> Result<i64, String> {
    match field(compound, name)? {
        Tag::Long(value) => Ok(*value),
        _ => Err(format!("NBT field {name} must be a long")),
    }
}

fn optional_long_field(compound: &[(String, Tag)], name: &str) -> Result<Option<i64>, String> {
    match optional_field(compound, name) {
        Some(Tag::Long(value)) => Ok(Some(*value)),
        Some(_) => Err(format!("NBT field {name} must be a long")),
        None => Ok(None),
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

fn optional_list_field<'a>(
    compound: &'a [(String, Tag)],
    name: &str,
) -> Result<Option<Vec<Tag>>, String> {
    match optional_field(compound, name) {
        Some(Tag::List(values)) => Ok(Some(values.clone())),
        Some(_) => Err(format!("NBT field {name} must be a list")),
        None => Ok(None),
    }
}

fn optional_compound_field<'a>(
    compound: &'a [(String, Tag)],
    name: &str,
) -> Result<Option<&'a [(String, Tag)]>, String> {
    match optional_field(compound, name) {
        Some(Tag::Compound(values)) => Ok(Some(values)),
        Some(_) => Err(format!("NBT field {name} must be a compound")),
        None => Ok(None),
    }
}

fn optional_byte_array(compound: &[(String, Tag)], name: &str) -> Result<Option<Vec<i8>>, String> {
    match compound.iter().find(|(field_name, _)| field_name == name) {
        Some((_name, Tag::ByteArray(values))) => Ok(Some(values.clone())),
        Some((_name, _)) => Err(format!("NBT field {name} must be a byte array")),
        None => Ok(None),
    }
}

fn optional_long_array(compound: &[(String, Tag)], name: &str) -> Result<Option<Vec<i64>>, String> {
    match optional_field(compound, name) {
        Some(Tag::LongArray(values)) => Ok(Some(values.clone())),
        Some(_) => Err(format!("NBT field {name} must be a long array")),
        None => Ok(None),
    }
}

fn optional_bool_field(compound: &[(String, Tag)], name: &str) -> Result<Option<bool>, String> {
    match optional_field(compound, name) {
        Some(Tag::Byte(value)) => Ok(Some(*value != 0)),
        Some(_) => Err(format!("NBT field {name} must be a byte boolean")),
        None => Ok(None),
    }
}

fn chunk_status_field(compound: &[(String, Tag)]) -> Result<String, String> {
    let status = string_field(compound, "Status")?;
    if status.is_empty() {
        return Err("chunk Status cannot be empty".to_string());
    }
    Ok(chunk_status(status)
        .map(|status| status.id.to_string())
        .unwrap_or_else(|| "minecraft:empty".to_string()))
}

fn optional_blending_data(compound: &[(String, Tag)]) -> Result<Option<Tag>, String> {
    let Some(tag) = optional_field(compound, "blending_data") else {
        return Ok(None);
    };
    validate_blending_data(tag)?;
    Ok(Some(tag.clone()))
}

fn validate_blending_data(tag: &Tag) -> Result<(), String> {
    let compound = compound(tag)?;
    let min_section = int_field(compound, "min_section")?;
    let max_section = int_field(compound, "max_section")?;
    let heights = optional_double_list(compound, "heights")?;
    validate_blending_data_packed(BlendingDataPacked {
        min_section,
        max_section,
        heights: heights.as_deref(),
    })
}

fn optional_double_list(
    compound: &[(String, Tag)],
    name: &str,
) -> Result<Option<Vec<f64>>, String> {
    let Some(tag) = optional_field(compound, name) else {
        return Ok(None);
    };
    let Tag::List(values) = tag else {
        return Err(format!("NBT field {name} must be a double list"));
    };
    values
        .iter()
        .map(|value| match value {
            Tag::Double(value) => Ok(*value),
            _ => Err(format!("NBT field {name} must be a double list")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn optional_below_zero_retrogen(compound: &[(String, Tag)]) -> Result<Option<Tag>, String> {
    let Some(tag) = optional_field(compound, "below_zero_retrogen") else {
        return Ok(None);
    };
    validate_below_zero_retrogen(tag)?;
    Ok(Some(tag.clone()))
}

fn validate_below_zero_retrogen(tag: &Tag) -> Result<(), String> {
    let compound = compound(tag)?;
    let target_status = string_field(compound, "target_status")?;
    let target_status_name = target_status
        .strip_prefix("minecraft:")
        .unwrap_or(target_status);
    if target_status_name == "empty" {
        return Err("below_zero_retrogen target_status cannot be empty".to_string());
    }
    if chunk_status(target_status_name).is_none() {
        return Err(format!(
            "below_zero_retrogen target_status {target_status} is not a known chunk status"
        ));
    }
    optional_long_array(compound, "missing_bedrock")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_status, chunk_status_is_or_after, default_biomes_container,
        default_block_states_container, pack_postprocessing_offset, saved_tick_tag, string_field,
        BlockStateEntry, ChunkSection, ChunkStatusTaskKind, ChunkType, HeightmapKind, LevelChunk,
        PalettedContainer, SectionBlockPos, TickPriority, BIOME_SECTION_VOLUME,
        CHUNK_STATUS_PIPELINE, FINAL_HEIGHTMAPS, LIGHT_DATA_LAYER_LENGTH, SECTION_VOLUME,
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
            min_section_y: -4,
            last_update: 1234,
            status: "minecraft:spawn".to_string(),
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
            upgrade_data: Some(Tag::Compound(vec![("Sides".to_string(), Tag::Int(0))])),
            blending_data: Some(Tag::Compound(vec![
                ("min_section".to_string(), Tag::Int(-4)),
                ("max_section".to_string(), Tag::Int(20)),
            ])),
            below_zero_retrogen: Some(Tag::Compound(vec![(
                "target_status".to_string(),
                Tag::String("minecraft:noise".to_string()),
            )])),
            carving_mask: Some(vec![7, 8, 9]),
            block_ticks: vec![saved_tick_tag(
                "minecraft:stone".to_string(),
                64,
                70,
                -31,
                4,
                TickPriority::Normal,
            )],
            fluid_ticks: vec![saved_tick_tag(
                "minecraft:water".to_string(),
                65,
                63,
                -32,
                2,
                TickPriority::High,
            )],
            post_processing: vec![Tag::List(vec![Tag::Short(1), Tag::Short(2)])],
            light_correct: true,
        };

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.pos, pos);
        assert_eq!(decoded.min_section_y, -4);
        assert_eq!(decoded.last_update, 1234);
        assert_eq!(decoded.status, "minecraft:spawn");
        assert_eq!(
            decoded.sections[0].block_light.as_ref().unwrap().len(),
            2048
        );
        assert_eq!(decoded.sections[0].sky_light.as_ref().unwrap()[0], 15);
        assert!(decoded.heightmaps.contains_key("WORLD_SURFACE"));
        assert_eq!(decoded.block_entities.len(), 1);
        assert_eq!(decoded.entities.len(), 1);
        assert!(decoded.upgrade_data.is_some());
        assert!(decoded.blending_data.is_some());
        assert!(decoded.below_zero_retrogen.is_some());
        assert_eq!(decoded.carving_mask, Some(vec![7, 8, 9]));
        assert_eq!(decoded.block_ticks.len(), 1);
        assert_eq!(decoded.fluid_ticks.len(), 1);
        assert_eq!(decoded.post_processing.len(), 1);
        assert!(decoded.light_correct);
    }

    #[test]
    fn full_chunk_serialization_omits_proto_only_entity_and_carver_fields() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.status = "minecraft:full".to_string();
        chunk.entities = vec![Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:pig".to_string()),
        )])];
        chunk.carving_mask = Some(vec![1, 2, 3]);

        let encoded = chunk.to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &encoded else {
            panic!("chunk should encode as a compound");
        };
        assert!(fields.iter().all(|(name, _)| name != "entities"));
        assert!(fields.iter().all(|(name, _)| name != "carving_mask"));

        let decoded = LevelChunk::from_nbt(chunk.pos, &encoded).unwrap();
        assert!(decoded.entities.is_empty());
        assert!(decoded.carving_mask.is_none());
    }

    #[test]
    fn level_chunk_validates_blending_data_payload_shape() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut encoded = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut encoded else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "blending_data".to_string(),
            Tag::Compound(vec![
                ("min_section".to_string(), Tag::Int(-4)),
                ("max_section".to_string(), Tag::Int(20)),
                (
                    "heights".to_string(),
                    Tag::List((0..16).map(|value| Tag::Double(f64::from(value))).collect()),
                ),
            ]),
        ));

        let decoded = LevelChunk::from_nbt(pos, &encoded).unwrap();

        assert!(decoded.blending_data.is_some());
    }

    #[test]
    fn level_chunk_rejects_invalid_blending_data_payload_shape() {
        let pos = ChunkPos { x: 0, z: 0 };

        let mut missing_sections = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut missing_sections else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "blending_data".to_string(),
            Tag::Compound(vec![("heights".to_string(), Tag::List(Vec::new()))]),
        ));
        let err = LevelChunk::from_nbt(pos, &missing_sections).unwrap_err();
        assert!(err.contains("missing NBT field min_section"));

        let mut wrong_heights_len = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut wrong_heights_len else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "blending_data".to_string(),
            Tag::Compound(vec![
                ("min_section".to_string(), Tag::Int(-4)),
                ("max_section".to_string(), Tag::Int(20)),
                (
                    "heights".to_string(),
                    Tag::List(vec![Tag::Double(0.0), Tag::Double(1.0)]),
                ),
            ]),
        ));
        let err = LevelChunk::from_nbt(pos, &wrong_heights_len).unwrap_err();
        assert!(err.contains("heights has to be of length 16"));

        let mut wrong_heights_type = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut wrong_heights_type else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "blending_data".to_string(),
            Tag::Compound(vec![
                ("min_section".to_string(), Tag::Int(-4)),
                ("max_section".to_string(), Tag::Int(20)),
                ("heights".to_string(), Tag::List(vec![Tag::Int(0)])),
            ]),
        ));
        let err = LevelChunk::from_nbt(pos, &wrong_heights_type).unwrap_err();
        assert!(err.contains("NBT field heights must be a double list"));
    }

    #[test]
    fn level_chunk_accepts_valid_below_zero_retrogen_payload() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut encoded = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut encoded else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "below_zero_retrogen".to_string(),
            Tag::Compound(vec![
                (
                    "target_status".to_string(),
                    Tag::String("minecraft:noise".to_string()),
                ),
                ("missing_bedrock".to_string(), Tag::LongArray(vec![3])),
            ]),
        ));

        let decoded = LevelChunk::from_nbt(pos, &encoded).unwrap();

        assert!(decoded.below_zero_retrogen.is_some());
    }

    #[test]
    fn level_chunk_rejects_invalid_below_zero_retrogen_payloads() {
        let pos = ChunkPos { x: 0, z: 0 };

        let mut empty_status = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut empty_status else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "below_zero_retrogen".to_string(),
            Tag::Compound(vec![(
                "target_status".to_string(),
                Tag::String("minecraft:empty".to_string()),
            )]),
        ));
        let err = LevelChunk::from_nbt(pos, &empty_status).unwrap_err();
        assert!(err.contains("target_status cannot be empty"));

        let mut unknown_status = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut unknown_status else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "below_zero_retrogen".to_string(),
            Tag::Compound(vec![(
                "target_status".to_string(),
                Tag::String("minecraft:not_a_status".to_string()),
            )]),
        ));
        let err = LevelChunk::from_nbt(pos, &unknown_status).unwrap_err();
        assert!(err.contains("not a known chunk status"));

        let mut wrong_bedrock_shape = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut wrong_bedrock_shape else {
            panic!("chunk should encode as a compound");
        };
        fields.push((
            "below_zero_retrogen".to_string(),
            Tag::Compound(vec![
                (
                    "target_status".to_string(),
                    Tag::String("minecraft:noise".to_string()),
                ),
                ("missing_bedrock".to_string(), Tag::List(Vec::new())),
            ]),
        ));
        let err = LevelChunk::from_nbt(pos, &wrong_bedrock_shape).unwrap_err();
        assert!(err.contains("NBT field missing_bedrock must be a long array"));
    }

    #[test]
    fn empty_level_chunk_uses_vanilla_structures_payload_shape() {
        let chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        let Tag::Compound(fields) = &chunk.structures else {
            panic!("structures payload should be a compound");
        };
        assert!(matches!(
            fields.iter().find(|(name, _)| name == "starts"),
            Some((_, Tag::Compound(starts))) if starts.is_empty()
        ));
        assert!(matches!(
            fields.iter().find(|(name, _)| name == "References"),
            Some((_, Tag::Compound(references))) if references.is_empty()
        ));
    }

    #[test]
    fn level_chunk_marks_postprocessing_offsets_by_section() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 3, z: -2 });
        chunk.min_section_y = -1;
        chunk.sections = vec![
            ChunkSection {
                y: -1,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: None,
            },
            ChunkSection {
                y: 0,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: None,
            },
        ];

        assert!(chunk.mark_pos_for_postprocessing(48, -1, -17));
        assert!(chunk.mark_pos_for_postprocessing(63, 0, -32));
        assert!(!chunk.mark_pos_for_postprocessing(48, 16, -17));

        assert_eq!(
            chunk.post_processing,
            vec![
                Tag::List(vec![Tag::Short(pack_postprocessing_offset(48, -1, -17))]),
                Tag::List(vec![Tag::Short(pack_postprocessing_offset(63, 0, -32))]),
            ]
        );
    }

    #[test]
    fn level_chunk_sets_pending_block_entity_nbt_by_position() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 1, z: 1 });
        let chest = Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:chest".to_string())),
            ("x".to_string(), Tag::Int(20)),
            ("y".to_string(), Tag::Int(64)),
            ("z".to_string(), Tag::Int(23)),
            ("keep".to_string(), Tag::Byte(1)),
        ]);
        let barrel = Tag::Compound(vec![
            (
                "id".to_string(),
                Tag::String("minecraft:barrel".to_string()),
            ),
            ("x".to_string(), Tag::Int(20)),
            ("y".to_string(), Tag::Int(64)),
            ("z".to_string(), Tag::Int(23)),
        ]);
        let malformed = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:furnace".to_string()),
        )]);

        assert!(chunk.set_block_entity_nbt(chest));
        assert!(chunk.set_block_entity_nbt(barrel.clone()));
        assert!(!chunk.set_block_entity_nbt(malformed));

        assert_eq!(chunk.block_entities, vec![barrel]);
    }

    #[test]
    fn level_chunk_adds_proto_entity_nbt_in_generation_order() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        let pig = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:pig".to_string()),
        )]);
        let cow = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:cow".to_string()),
        )]);

        assert!(chunk.add_entity_nbt(pig.clone()));
        assert!(!chunk.add_entity_nbt(Tag::String("minecraft:bat".to_string())));
        assert!(chunk.add_entity_nbt(cow.clone()));

        assert_eq!(chunk.entities, vec![pig, cow]);
    }

    #[test]
    fn level_chunk_schedules_block_and_fluid_ticks_for_own_chunk() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: -2, z: 3 });

        assert!(chunk.schedule_block_tick(
            "minecraft:oak_sapling",
            -17,
            65,
            48,
            7,
            TickPriority::High
        ));
        assert!(chunk.schedule_fluid_tick(
            "minecraft:water",
            -32,
            -5,
            63,
            1,
            TickPriority::VeryLow
        ));
        assert!(!chunk.schedule_block_tick(
            "minecraft:stone",
            -33,
            65,
            48,
            0,
            TickPriority::Normal
        ));

        assert_eq!(
            chunk.block_ticks,
            vec![Tag::Compound(vec![
                (
                    "i".to_string(),
                    Tag::String("minecraft:oak_sapling".to_string())
                ),
                ("x".to_string(), Tag::Int(-17)),
                ("y".to_string(), Tag::Int(65)),
                ("z".to_string(), Tag::Int(48)),
                ("t".to_string(), Tag::Int(7)),
                ("p".to_string(), Tag::Int(-1)),
            ])]
        );
        assert_eq!(
            chunk.fluid_ticks,
            vec![Tag::Compound(vec![
                ("i".to_string(), Tag::String("minecraft:water".to_string())),
                ("x".to_string(), Tag::Int(-32)),
                ("y".to_string(), Tag::Int(-5)),
                ("z".to_string(), Tag::Int(63)),
                ("t".to_string(), Tag::Int(1)),
                ("p".to_string(), Tag::Int(2)),
            ])]
        );
    }

    #[test]
    fn level_chunk_load_filters_saved_ticks_to_own_chunk() {
        let pos = ChunkPos { x: -2, z: 3 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.schedule_block_tick("minecraft:oak_sapling", -17, 65, 48, 7, TickPriority::High);
        chunk.block_ticks.push(saved_tick_tag(
            "minecraft:stone".to_string(),
            -33,
            65,
            48,
            0,
            TickPriority::Normal,
        ));
        chunk.schedule_fluid_tick("minecraft:water", -32, -5, 63, 1, TickPriority::VeryLow);
        chunk.fluid_ticks.push(saved_tick_tag(
            "minecraft:lava".to_string(),
            -17,
            20,
            64,
            0,
            TickPriority::Normal,
        ));

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.block_ticks.len(), 1);
        assert_eq!(decoded.fluid_ticks.len(), 1);
        assert!(matches!(
            &decoded.block_ticks[0],
            Tag::Compound(fields)
                if string_field(fields, "i").unwrap() == "minecraft:oak_sapling"
        ));
        assert!(matches!(
            &decoded.fluid_ticks[0],
            Tag::Compound(fields) if string_field(fields, "i").unwrap() == "minecraft:water"
        ));
    }

    #[test]
    fn level_chunk_sets_valid_section_light_arrays() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.sections = vec![
            ChunkSection {
                y: -1,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: None,
            },
            ChunkSection {
                y: 0,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: None,
            },
        ];

        assert!(!chunk.set_section_light_arrays(-1, Some(vec![0; 17]), None));
        assert!(!chunk.set_section_light_arrays(1, Some(vec![0; LIGHT_DATA_LAYER_LENGTH]), None));
        assert!(chunk.set_section_light_arrays(-1, Some(vec![0; LIGHT_DATA_LAYER_LENGTH]), None));
        assert!(!chunk.light_correct);
        assert!(chunk.set_section_light_arrays(0, None, Some(vec![15; LIGHT_DATA_LAYER_LENGTH])));

        assert_eq!(
            chunk.sections[0].block_light.as_ref().unwrap().len(),
            LIGHT_DATA_LAYER_LENGTH
        );
        assert_eq!(chunk.sections[1].sky_light.as_ref().unwrap()[0], 15);
        assert!(chunk.light_correct);
    }

    #[test]
    fn level_chunk_rejects_misplaced_payloads() {
        let tag = LevelChunk::empty(ChunkPos { x: 9, z: 9 }).to_nbt(TARGET_DATA_VERSION);
        let err = LevelChunk::from_nbt(ChunkPos { x: 0, z: 0 }, &tag).unwrap_err();
        assert!(err.contains("wrong position"));
    }

    #[test]
    fn level_chunk_rejects_missing_or_unsupported_data_versions() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut missing = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        if let Tag::Compound(values) = &mut missing {
            values.retain(|(name, _)| name != "DataVersion");
        }
        let err = LevelChunk::from_nbt(pos, &missing).unwrap_err();
        assert!(err.contains("missing DataVersion"));

        let unsupported =
            LevelChunk::empty(pos).to_nbt(crate::storage::datafix::TARGET_DATA_VERSION - 1);
        let err = LevelChunk::from_nbt(pos, &unsupported).unwrap_err();
        assert!(err.contains("Unsupported world DataVersion"));
    }

    #[test]
    fn level_chunk_normalizes_unknown_status_like_vanilla_storage_codec() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, status) = fields
            .iter_mut()
            .find(|(name, _)| name == "Status")
            .expect("Status should be present");
        *status = Tag::String("minecraft:not_a_status".to_string());

        let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

        assert_eq!(decoded.status, "minecraft:empty");
    }

    #[test]
    fn level_chunk_rejects_empty_status_like_vanilla_parse_null_path() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, status) = fields
            .iter_mut()
            .find(|(name, _)| name == "Status")
            .expect("Status should be present");
        *status = Tag::String(String::new());

        let err = LevelChunk::from_nbt(pos, &tag).unwrap_err();

        assert!(err.contains("Status cannot be empty"));
    }

    #[test]
    fn level_chunk_defaults_absent_vanilla_optional_collections() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        fields.retain(|(name, _)| {
            matches!(
                name.as_str(),
                "DataVersion" | "xPos" | "zPos" | "Status" | "LastUpdate"
            )
        });

        let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

        assert_eq!(decoded.inhabited_time, 0);
        assert!(decoded.sections.is_empty());
        assert!(decoded.heightmaps.is_empty());
        assert!(decoded.block_entities.is_empty());
        assert!(decoded.entities.is_empty());
        assert!(decoded.block_ticks.is_empty());
        assert!(decoded.fluid_ticks.is_empty());
        assert!(decoded.post_processing.is_empty());
        assert!(matches!(decoded.structures, Tag::Compound(_)));
    }

    #[test]
    fn level_chunk_loads_only_heightmaps_valid_for_status() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.status = "minecraft:noise".to_string();
        chunk
            .heightmaps
            .insert("WORLD_SURFACE_WG".to_string(), Tag::LongArray(vec![1]));
        chunk
            .heightmaps
            .insert("MOTION_BLOCKING".to_string(), Tag::LongArray(vec![2]));
        chunk
            .heightmaps
            .insert("OCEAN_FLOOR_WG".to_string(), Tag::Int(3));

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert!(decoded.heightmaps.contains_key("WORLD_SURFACE_WG"));
        assert!(!decoded.heightmaps.contains_key("OCEAN_FLOOR_WG"));
        assert!(!decoded.heightmaps.contains_key("MOTION_BLOCKING"));
    }

    #[test]
    fn level_chunk_load_ignores_non_compound_entity_entries() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.status = "minecraft:spawn".to_string();
        chunk.entities = vec![
            Tag::String("not-an-entity".to_string()),
            Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:pig".to_string()),
            )]),
        ];
        chunk.block_entities = vec![
            Tag::Int(7),
            Tag::Compound(vec![(
                "id".to_string(),
                Tag::String("minecraft:chest".to_string()),
            )]),
        ];

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.entities.len(), 1);
        assert_eq!(decoded.block_entities.len(), 1);
        assert!(matches!(&decoded.entities[0], Tag::Compound(_)));
        assert!(matches!(&decoded.block_entities[0], Tag::Compound(_)));
    }

    #[test]
    fn level_chunk_load_normalizes_postprocessing_sections() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut chunk = LevelChunk::empty(pos);
        chunk.post_processing = vec![
            Tag::List(vec![Tag::Short(12), Tag::Int(99)]),
            Tag::String("not-a-section-list".to_string()),
        ];

        let decoded = LevelChunk::from_nbt(pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(
            decoded.post_processing,
            vec![
                Tag::List(vec![Tag::Short(12), Tag::Short(0)]),
                Tag::List(Vec::new()),
            ]
        );
    }

    #[test]
    fn level_chunk_load_defaults_sparse_section_payloads() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, sections) = fields
            .iter_mut()
            .find(|(name, _)| name == "sections")
            .expect("sections should be present");
        *sections = Tag::List(vec![
            Tag::String("not-a-section".to_string()),
            Tag::Compound(Vec::new()),
        ]);

        let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

        assert_eq!(decoded.sections.len(), 1);
        assert_eq!(decoded.sections[0].y, 0);
        assert_eq!(
            decoded.sections[0].block_states,
            default_block_states_container()
        );
        assert_eq!(decoded.sections[0].biomes, default_biomes_container());
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
