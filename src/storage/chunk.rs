#![allow(dead_code)]

use std::collections::BTreeMap;

use super::datafix::require_current_tag_data_version;
use super::nbt::Tag;
use super::region::ChunkPos;
use crate::worldgen::{validate_blending_data_packed, BlendingDataPacked};

pub const CHUNK_WIDTH: i32 = 16;
pub const SECTION_HEIGHT: i32 = 16;
pub const SECTION_VOLUME: usize = 16 * 16 * 16;
pub const BIOME_SECTION_VOLUME: usize = 4 * 4 * 4;
pub const LIGHT_DATA_LAYER_LENGTH: usize = 2048;
pub const LIGHT_DATA_LAYER_NIBBLE_COUNT: usize = 4096;
pub const LIGHT_DATA_LAYER_WIDTH: usize = 16;
pub const LIGHT_DATA_LAYER_ROW_SIZE: usize = 128;

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
pub enum ChunkPyramidKind {
    Generation,
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkGenerationLayerPlan {
    pub status: &'static str,
    pub needs_generation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkGenerationChunkStepPlan {
    Apply {
        pyramid: ChunkPyramidKind,
        generate: bool,
    },
    UnexpectedGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkGenerationFutureState {
    Pending,
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGenerationWaitPlan {
    pub waiting_for_index: Option<usize>,
    pub remaining_layer: Vec<ChunkGenerationFutureState>,
    pub marked_for_cancellation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkGenerationScheduleLayerPlan {
    pub radius: i32,
    pub visited_positions: Vec<(i32, i32)>,
    pub stopped_early: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenRegionAccessChunk {
    pub chunk: ChunkPos,
    pub distance: i32,
    pub max_read_status: &'static str,
    pub can_write_blocks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldGenRegionAccessPlan {
    pub center: ChunkPos,
    pub target_status: &'static str,
    pub block_state_write_radius: i32,
    pub chunks: Vec<WorldGenRegionAccessChunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkGenerationRunPlan {
    Waiting {
        waiting_for_index: usize,
        remaining_layer: Vec<ChunkGenerationFutureState>,
        marked_for_cancellation: bool,
    },
    Released,
    Schedule(ChunkGenerationLayerPlan),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightLayer {
    Block,
    Sky,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedSectionLightData {
    pub layer: LightLayer,
    pub section_y: i8,
    pub data: Vec<i8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightSectionStatusUpdate {
    pub section_y: i8,
    pub has_only_air: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLightHandoffPlan {
    pub retain_data: bool,
    pub queued_sections: Vec<QueuedSectionLightData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkInitializeLightPlan {
    pub pre_update_section_statuses: Vec<LightSectionStatusUpdate>,
    pub post_update_light_enabled: bool,
    pub post_update_retain_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkLightCompletionPlan {
    pub initial_light_correct: bool,
    pub pre_update_propagate_light_sources: bool,
    pub completed_light_correct: bool,
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

pub fn pack_postprocessing_offset(x: i32, y: i32, z: i32) -> i16 {
    ((x & 15) | ((y & 15) << 4) | ((z & 15) << 8)) as i16
}

pub fn unpack_postprocessing_offset(
    packed: i16,
    section_y: i32,
    chunk_pos: ChunkPos,
) -> (i32, i32, i32) {
    let packed = packed as u16;
    let x = chunk_pos.x * CHUNK_WIDTH + i32::from(packed & 15);
    let y = section_y * SECTION_HEIGHT + i32::from((packed >> 4) & 15);
    let z = chunk_pos.z * CHUNK_WIDTH + i32::from((packed >> 8) & 15);
    (x, y, z)
}

pub fn light_data_layer_index(x: i32, y: i32, z: i32) -> usize {
    (((y & 15) << 8) | ((z & 15) << 4) | (x & 15)) as usize
}

pub fn light_data_layer_byte_index(index: usize) -> usize {
    index >> 1
}

pub fn light_data_layer_nibble_index(index: usize) -> usize {
    index & 1
}

pub fn light_data_layer_pack_filled(value: u8) -> i8 {
    let value = value & 15;
    (value | (value << 4)) as i8
}

pub fn light_data_layer_get(data: Option<&[i8]>, default_value: u8, x: i32, y: i32, z: i32) -> u8 {
    let index = light_data_layer_index(x, y, z);
    match data {
        Some(data) => {
            let byte = data[light_data_layer_byte_index(index)] as u8;
            (byte >> (4 * light_data_layer_nibble_index(index))) & 15
        }
        None => default_value & 15,
    }
}

pub fn light_data_layer_materialize(default_value: u8) -> Vec<i8> {
    vec![light_data_layer_pack_filled(default_value); LIGHT_DATA_LAYER_LENGTH]
}

pub fn light_data_layer_set(
    data: &mut Option<Vec<i8>>,
    default_value: u8,
    x: i32,
    y: i32,
    z: i32,
    value: u8,
) {
    let layer = data.get_or_insert_with(|| light_data_layer_materialize(default_value));
    let index = light_data_layer_index(x, y, z);
    let byte_index = light_data_layer_byte_index(index);
    let shift = 4 * light_data_layer_nibble_index(index);
    let mask = !(15_u8 << shift);
    let byte = layer[byte_index] as u8;
    layer[byte_index] = ((byte & mask) | ((value & 15) << shift)) as i8;
}

pub fn light_data_layer_is_definitely_homogenous(data: Option<&[i8]>) -> bool {
    data.is_none()
}

pub fn light_data_layer_is_definitely_filled_with(
    data: Option<&[i8]>,
    default_value: u8,
    value: u8,
) -> bool {
    data.is_none() && (default_value & 15) == (value & 15)
}

pub fn light_data_layer_is_empty(data: Option<&[i8]>, default_value: u8) -> bool {
    data.is_none() && (default_value & 15) == 0
}

pub fn pack_chunk_pos_as_long(pos: ChunkPos) -> i64 {
    (i64::from(pos.x) & 0xffff_ffff) | ((i64::from(pos.z) & 0xffff_ffff) << 32)
}

pub fn unpack_chunk_pos_from_long(packed: i64) -> ChunkPos {
    ChunkPos {
        x: packed as i32,
        z: (packed >> 32) as i32,
    }
}

pub fn chunk_pos_chessboard_distance(a: ChunkPos, b: ChunkPos) -> i32 {
    (a.x - b.x).abs().max((a.z - b.z).abs())
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

fn block_state_name(entry: &Tag) -> Option<&str> {
    let Tag::Compound(fields) = entry else {
        return None;
    };
    fields
        .iter()
        .find_map(|(key, value)| match (key.as_str(), value) {
            ("Name", Tag::String(name)) => Some(name.as_str()),
            _ => None,
        })
}

fn block_state_entry(entry: &Tag) -> Option<BlockStateEntry> {
    let Tag::Compound(fields) = entry else {
        return None;
    };
    let mut name = None;
    let mut properties = BTreeMap::new();
    for (key, value) in fields {
        match (key.as_str(), value) {
            ("Name", Tag::String(value)) => name = Some(value.clone()),
            ("Properties", Tag::Compound(values)) => {
                for (property, property_value) in values {
                    if let Tag::String(property_value) = property_value {
                        properties.insert(property.clone(), property_value.clone());
                    }
                }
            }
            _ => {}
        }
    }
    Some(BlockStateEntry {
        name: name?,
        properties,
    })
}

fn block_state_tag_from_name(block_name: &str) -> Tag {
    let Some((name, raw_properties)) = block_name.split_once('[') else {
        return Tag::Compound(vec![(
            "Name".to_string(),
            Tag::String(block_name.to_string()),
        )]);
    };
    let properties = raw_properties
        .trim_end_matches(']')
        .split(',')
        .filter_map(|property| {
            let (key, value) = property.split_once('=')?;
            Some((key.to_string(), Tag::String(value.to_string())))
        })
        .collect::<Vec<_>>();
    if properties.is_empty() {
        Tag::Compound(vec![("Name".to_string(), Tag::String(name.to_string()))])
    } else {
        Tag::Compound(vec![
            ("Name".to_string(), Tag::String(name.to_string())),
            ("Properties".to_string(), Tag::Compound(properties)),
        ])
    }
}

fn block_light_emission(block_name: &str) -> u8 {
    crate::block_metadata::representative_state_definition(block_name)
        .map(|definition| definition.physical.light_emission)
        .unwrap_or(0)
}

fn pack_heightmap_values(values: &[i32; 16 * 16]) -> Vec<i64> {
    const BITS_PER_ENTRY: usize = 9;
    let mut packed = vec![0_u64; (values.len() * BITS_PER_ENTRY).div_ceil(64)];
    for (index, value) in values.iter().copied().enumerate() {
        let bit_offset = index * BITS_PER_ENTRY;
        let word_index = bit_offset / 64;
        let bit_index = bit_offset % 64;
        let value = value.max(0) as u64 & ((1 << BITS_PER_ENTRY) - 1);
        packed[word_index] |= value << bit_index;
        let spill = bit_index + BITS_PER_ENTRY;
        if spill > 64 {
            packed[word_index + 1] |= value >> (64 - bit_index);
        }
    }
    packed.into_iter().map(|word| word as i64).collect()
}

fn unpack_heightmap_values(data: &[i64]) -> [i32; 16 * 16] {
    const BITS_PER_ENTRY: usize = 9;
    let mut values = [0; 16 * 16];
    let mask = (1_u64 << BITS_PER_ENTRY) - 1;
    for (index, value) in values.iter_mut().enumerate() {
        let bit_offset = index * BITS_PER_ENTRY;
        let word_index = bit_offset / 64;
        let bit_index = bit_offset % 64;
        let Some(word) = data.get(word_index).copied() else {
            continue;
        };
        let mut unpacked = (word as u64) >> bit_index;
        let spill = bit_index + BITS_PER_ENTRY;
        if spill > 64 {
            if let Some(next_word) = data.get(word_index + 1).copied() {
                unpacked |= (next_word as u64) << (64 - bit_index);
            }
        }
        *value = (unpacked & mask) as i32;
    }
    values
}

fn unpack_heightmap_value(data: &[i64], index: usize) -> i32 {
    const BITS_PER_ENTRY: usize = 9;
    let bit_offset = index * BITS_PER_ENTRY;
    let word_index = bit_offset / 64;
    let bit_index = bit_offset % 64;
    let Some(word) = data.get(word_index).copied() else {
        return 0;
    };
    let mut unpacked = (word as u64) >> bit_index;
    let spill = bit_index + BITS_PER_ENTRY;
    if spill > 64 {
        if let Some(next_word) = data.get(word_index + 1).copied() {
            unpacked |= (next_word as u64) << (64 - bit_index);
        }
    }
    (unpacked & ((1_u64 << BITS_PER_ENTRY) - 1)) as i32
}

fn heightmap_block_matches(heightmap: HeightmapKind, block: &str) -> bool {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => !matches!(
            block,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ),
        HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg => block_blocks_motion(block),
        HeightmapKind::MotionBlocking => block_blocks_motion(block) || block_has_fluid(block),
        HeightmapKind::MotionBlockingNoLeaves => {
            (block_blocks_motion(block) || block_has_fluid(block)) && !block_is_leaves(block)
        }
    }
}

fn block_blocks_motion(block: &str) -> bool {
    !matches!(
        block_state_id(block),
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:snow"
    )
}

fn block_has_fluid(block: &str) -> bool {
    matches!(block_state_id(block), "minecraft:water" | "minecraft:lava")
        || block.contains("waterlogged=true")
}

fn block_is_leaves(block: &str) -> bool {
    block_state_id(block).ends_with("_leaves")
}

fn block_state_id(block: &str) -> &str {
    block.split_once('[').map_or(block, |(id, _)| id)
}

fn empty_structures_payload() -> Tag {
    Tag::Compound(vec![
        ("starts".to_string(), Tag::Compound(Vec::new())),
        ("References".to_string(), Tag::Compound(Vec::new())),
    ])
}

fn optional_structures_payload(compound: &[(String, Tag)], pos: ChunkPos) -> Tag {
    optional_field(compound, "structures")
        .map(|tag| normalize_structures_payload(tag, pos))
        .unwrap_or_else(empty_structures_payload)
}

fn normalize_structures_payload(tag: &Tag, pos: ChunkPos) -> Tag {
    let Ok(fields) = compound(tag) else {
        return empty_structures_payload();
    };
    let starts = optional_field(fields, "starts")
        .and_then(|tag| compound(tag).ok())
        .map(|starts| {
            starts
                .iter()
                .filter(|(_, start)| matches!(start, Tag::Compound(_)))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let references = optional_field(fields, "References")
        .and_then(|tag| compound(tag).ok())
        .map(|references| {
            references
                .iter()
                .filter_map(|(id, value)| match value {
                    Tag::LongArray(values) => {
                        let values = values
                            .iter()
                            .copied()
                            .filter(|reference| {
                                chunk_pos_chessboard_distance(
                                    pos,
                                    unpack_chunk_pos_from_long(*reference),
                                ) <= 8
                            })
                            .collect::<Vec<_>>();
                        (!values.is_empty()).then(|| (id.clone(), Tag::LongArray(values)))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    Tag::Compound(vec![
        ("starts".to_string(), Tag::Compound(starts)),
        ("References".to_string(), Tag::Compound(references)),
    ])
}

fn structures_child_compound_mut<'a>(
    structures: &'a mut Tag,
    name: &str,
) -> &'a mut Vec<(String, Tag)> {
    if !matches!(structures, Tag::Compound(_)) {
        *structures = empty_structures_payload();
    }
    let Tag::Compound(fields) = structures else {
        unreachable!("structures payload was normalized to a compound");
    };
    let index = match fields.iter().position(|(field_name, _)| field_name == name) {
        Some(index) => {
            if !matches!(fields[index].1, Tag::Compound(_)) {
                fields[index].1 = Tag::Compound(Vec::new());
            }
            index
        }
        None => {
            fields.push((name.to_string(), Tag::Compound(Vec::new())));
            fields.len() - 1
        }
    };
    let Tag::Compound(child) = &mut fields[index].1 else {
        unreachable!("structure child was normalized to a compound");
    };
    child
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
            block_states: optional_compound_tag(compound, "block_states")
                .unwrap_or_else(default_block_states_container),
            biomes: optional_compound_tag(compound, "biomes")
                .unwrap_or_else(default_biomes_container),
            block_light: optional_light_array(compound, "BlockLight")?,
            sky_light: optional_light_array(compound, "SkyLight")?,
        })
    }

    pub fn has_only_air(&self) -> bool {
        let Ok(container) = PalettedContainer::from_nbt(&self.block_states, SECTION_VOLUME) else {
            return false;
        };
        if container
            .palette
            .iter()
            .all(|entry| block_state_name(entry) == Some("minecraft:air"))
        {
            return true;
        }
        for index in 0..SECTION_VOLUME {
            let Some(entry) = container.get_entry(index) else {
                return false;
            };
            if block_state_name(entry) != Some("minecraft:air") {
                return false;
            }
        }
        true
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

impl LevelChunk {
    pub fn heightmap_value(
        &self,
        heightmap: HeightmapKind,
        local_x: usize,
        local_z: usize,
    ) -> Option<i32> {
        if local_x >= 16 || local_z >= 16 {
            return None;
        }
        let Tag::LongArray(values) = self.heightmaps.get(heightmap.storage_name())? else {
            return None;
        };
        Some(unpack_heightmap_value(values, local_z * 16 + local_x))
    }

    pub fn get_block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<String> {
        self.get_block_state_name(world_x, world_y, world_z)
            .map(str::to_string)
    }

    pub fn get_block_state_name(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let section = self
            .sections
            .get((i32::from(section_y) - self.min_section_y) as usize)
            .filter(|section| section.y == section_y)
            .or_else(|| self.sections.iter().find(|section| section.y == section_y))?;
        let entry =
            paletted_container_entry_from_nbt(&section.block_states, SECTION_VOLUME, index)?;
        if let Tag::Compound(fields) = entry {
            fields.iter().find(|(k, _)| k == "Name").and_then(|(_, v)| {
                if let Tag::String(name) = v {
                    Some(name.as_str())
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    pub fn get_block_state_model(
        &self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> Option<BlockStateEntry> {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let section = self
            .sections
            .get((i32::from(section_y) - self.min_section_y) as usize)
            .filter(|section| section.y == section_y)
            .or_else(|| self.sections.iter().find(|section| section.y == section_y))?;
        let entry =
            paletted_container_entry_from_nbt(&section.block_states, SECTION_VOLUME, index)?;
        block_state_entry(entry)
    }

    pub fn set_block_state(&mut self, world_x: i32, world_y: i32, world_z: i32, block_name: &str) {
        if self.set_block_state_raw(world_x, world_y, world_z, block_name) {
            let local_x = world_x.rem_euclid(16);
            let local_z = world_z.rem_euclid(16);
            self.update_heightmaps_after_block_change(local_x, world_y, local_z, block_name);
        }
    }

    pub fn set_block_state_without_heightmap_update(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block_name: &str,
    ) {
        self.set_block_state_raw(world_x, world_y, world_z, block_name);
    }

    fn set_block_state_raw(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block_name: &str,
    ) -> bool {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;

        let entry = block_state_tag_from_name(block_name);

        let section_index = i32::from(section_y) - self.min_section_y;
        let has_direct_section = section_index >= 0
            && self
                .sections
                .get(section_index as usize)
                .is_some_and(|section| section.y == section_y);
        if !has_direct_section && self.sections.iter().all(|s| s.y != section_y) {
            self.sections.push(ChunkSection {
                y: section_y,
                block_states: default_block_states_container(),
                biomes: default_biomes_container(),
                block_light: None,
                sky_light: Some(vec![-1i8; LIGHT_DATA_LAYER_LENGTH]),
            });
            self.sections.sort_by_key(|section| section.y);
        }

        let section = if section_index >= 0 {
            self.sections
                .get_mut(section_index as usize)
                .filter(|section| section.y == section_y)
        } else {
            None
        };
        let section = match section {
            Some(section) => Some(section),
            None => self.sections.iter_mut().find(|s| s.y == section_y),
        };

        if let Some(section) = section {
            return set_paletted_container_entry_in_nbt(
                &mut section.block_states,
                SECTION_VOLUME,
                index,
                &entry,
            );
        }
        false
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

    pub fn from_storage_name(name: &str) -> Option<Self> {
        match name {
            "WORLD_SURFACE_WG" => Some(Self::WorldSurfaceWg),
            "WORLD_SURFACE" => Some(Self::WorldSurface),
            "OCEAN_FLOOR_WG" => Some(Self::OceanFloorWg),
            "OCEAN_FLOOR" => Some(Self::OceanFloor),
            "MOTION_BLOCKING" => Some(Self::MotionBlocking),
            "MOTION_BLOCKING_NO_LEAVES" => Some(Self::MotionBlockingNoLeaves),
            _ => None,
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

pub fn chunk_status_is_after(status: &str, other: &str) -> Option<bool> {
    Some(chunk_status(status)?.index > chunk_status(other)?.index)
}

pub fn chunk_status_is_or_before(status: &str, other: &str) -> Option<bool> {
    Some(chunk_status(status)?.index <= chunk_status(other)?.index)
}

pub fn chunk_status_is_before(status: &str, other: &str) -> Option<bool> {
    Some(chunk_status(status)?.index < chunk_status(other)?.index)
}

pub fn chunk_status_max(a: &str, b: &str) -> Option<&'static str> {
    let a = chunk_status(a)?;
    let b = chunk_status(b)?;
    Some(if a.index > b.index { a.id } else { b.id })
}

pub fn chunk_status_list() -> Vec<&'static str> {
    CHUNK_STATUS_PIPELINE
        .iter()
        .map(|status| status.id)
        .collect()
}

mod level_chunk_impl;
mod paletted_container;
mod chunk_generation;
mod nbt_helpers;

pub use level_chunk_impl::*;
pub use paletted_container::*;
use chunk_generation::*;
use nbt_helpers::*;

#[cfg(test)]
mod tests;
