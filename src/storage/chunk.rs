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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLightHandoffPlan {
    pub retain_data: bool,
    pub queued_sections: Vec<QueuedSectionLightData>,
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

    pub fn highest_generated_status(&self) -> Option<&'static str> {
        let persisted = chunk_status(&self.status)?.id;
        match self
            .below_zero_retrogen
            .as_ref()
            .and_then(below_zero_retrogen_target_status)
        {
            Some(target) => chunk_status_max(persisted, target),
            None => Some(persisted),
        }
    }

    pub fn set_persisted_status(&mut self, status: impl Into<String>) -> bool {
        let status = status.into();
        let Some(normalized_status) = chunk_status(&status).map(|status| status.id) else {
            return false;
        };
        self.status = normalized_status.to_string();
        if self
            .below_zero_retrogen
            .as_ref()
            .and_then(below_zero_retrogen_target_status)
            .and_then(|target| chunk_status_is_or_after(normalized_status, target))
            .unwrap_or(false)
        {
            self.below_zero_retrogen = None;
        }
        true
    }

    pub fn promote_to_full_chunk(&mut self) -> Vec<Tag> {
        let migrated_entities = std::mem::take(&mut self.entities);
        self.carving_mask = None;
        self.set_persisted_status("minecraft:full");
        migrated_entities
    }

    pub fn set_inhabited_time(&mut self, inhabited_time: i64) {
        self.inhabited_time = inhabited_time;
    }

    pub fn increment_inhabited_time(&mut self, inhabited_time_delta: i64) {
        self.inhabited_time += inhabited_time_delta;
    }

    pub fn light_handoff_plan(&self, has_sky_light: bool) -> ChunkLightHandoffPlan {
        let mut queued_sections = Vec::new();
        for section in &self.sections {
            if let Some(block_light) = &section.block_light {
                queued_sections.push(QueuedSectionLightData {
                    layer: LightLayer::Block,
                    section_y: section.y,
                    data: block_light.clone(),
                });
            }
            if has_sky_light {
                if let Some(sky_light) = &section.sky_light {
                    queued_sections.push(QueuedSectionLightData {
                        layer: LightLayer::Sky,
                        section_y: section.y,
                        data: sky_light.clone(),
                    });
                }
            }
        }
        ChunkLightHandoffPlan {
            retain_data: !queued_sections.is_empty(),
            queued_sections,
        }
    }

    pub fn set_structure_start_nbt(&mut self, structure_id: impl Into<String>, start: Tag) -> bool {
        if !matches!(start, Tag::Compound(_)) {
            return false;
        }
        let starts = structures_child_compound_mut(&mut self.structures, "starts");
        let structure_id = structure_id.into();
        match starts.iter_mut().find(|(name, _)| name == &structure_id) {
            Some((_, existing)) => *existing = start,
            None => starts.push((structure_id, start)),
        }
        true
    }

    pub fn add_structure_reference(
        &mut self,
        structure_id: impl Into<String>,
        reference_pos: ChunkPos,
    ) -> bool {
        if chunk_pos_chessboard_distance(self.pos, reference_pos) > 8 {
            return false;
        }
        let references = structures_child_compound_mut(&mut self.structures, "References");
        let structure_id = structure_id.into();
        let packed_reference = pack_chunk_pos_as_long(reference_pos);
        match references
            .iter_mut()
            .find(|(name, _)| name == &structure_id)
            .map(|(_, tag)| tag)
        {
            Some(Tag::LongArray(values)) => {
                if !values.contains(&packed_reference) {
                    values.push(packed_reference);
                }
            }
            Some(existing) => *existing = Tag::LongArray(vec![packed_reference]),
            None => references.push((structure_id, Tag::LongArray(vec![packed_reference]))),
        }
        true
    }

    pub fn heightmaps_to_prime(&self) -> Vec<HeightmapKind> {
        chunk_status(&self.status)
            .map(|status| {
                status
                    .heightmaps_after
                    .iter()
                    .copied()
                    .filter(|heightmap| !self.heightmaps.contains_key(heightmap.storage_name()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn prime_heightmaps(&mut self, heightmaps: &[HeightmapKind]) {
        for heightmap in heightmaps {
            let values = self.compute_heightmap_values(*heightmap);
            self.heightmaps.insert(
                heightmap.storage_name().to_string(),
                Tag::LongArray(pack_heightmap_values(&values)),
            );
        }
    }

    pub fn prime_missing_heightmaps(&mut self) {
        let missing = self.heightmaps_to_prime();
        self.prime_heightmaps(&missing);
    }

    pub fn compute_heightmap_values(&self, heightmap: HeightmapKind) -> [i32; 16 * 16] {
        let mut values = [0; 16 * 16];
        let Some(highest_section_y) = self.sections.iter().map(|section| section.y).max() else {
            return values;
        };
        let min_y = self.min_section_y * 16;
        let max_y = i32::from(highest_section_y) * 16 + 15;
        for x in 0..16 {
            for z in 0..16 {
                let index = z * 16 + x;
                for y in (min_y..=max_y).rev() {
                    let Some(block) = self.get_block_state(
                        self.pos.x * CHUNK_WIDTH + x as i32,
                        y,
                        self.pos.z * CHUNK_WIDTH + z as i32,
                    ) else {
                        continue;
                    };
                    if block != "minecraft:air" && heightmap_block_matches(heightmap, &block) {
                        values[index] = y + 1;
                        break;
                    }
                }
            }
        }
        values
    }

    fn update_heightmaps_after_block_change(
        &mut self,
        local_x: i32,
        world_y: i32,
        local_z: i32,
        block_name: &str,
    ) {
        let column_index = local_z as usize * 16 + local_x as usize;
        let min_y = self.min_section_y * 16;
        let existing_heightmaps = self
            .heightmaps
            .keys()
            .filter_map(|name| HeightmapKind::from_storage_name(name))
            .collect::<Vec<_>>();
        for heightmap in existing_heightmaps {
            let Some(Tag::LongArray(raw_values)) = self.heightmaps.get(heightmap.storage_name())
            else {
                continue;
            };
            let mut values = unpack_heightmap_values(raw_values);
            let first_available = values[column_index];
            if world_y <= first_available - 2 {
                continue;
            }
            if heightmap_block_matches(heightmap, block_name) {
                if world_y >= first_available {
                    values[column_index] = world_y + 1;
                    self.heightmaps.insert(
                        heightmap.storage_name().to_string(),
                        Tag::LongArray(pack_heightmap_values(&values)),
                    );
                }
            } else if first_available - 1 == world_y {
                values[column_index] = (min_y..world_y)
                    .rev()
                    .find(|y| {
                        self.get_block_state(
                            self.pos.x * CHUNK_WIDTH + local_x,
                            *y,
                            self.pos.z * CHUNK_WIDTH + local_z,
                        )
                        .is_some_and(|block| heightmap_block_matches(heightmap, &block))
                    })
                    .map(|y| y + 1)
                    .unwrap_or(min_y);
                self.heightmaps.insert(
                    heightmap.storage_name().to_string(),
                    Tag::LongArray(pack_heightmap_values(&values)),
                );
            }
        }
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
            structures: optional_structures_payload(root, pos),
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

fn heightmap_block_matches(heightmap: HeightmapKind, block: &str) -> bool {
    match heightmap {
        HeightmapKind::WorldSurface | HeightmapKind::WorldSurfaceWg => !matches!(
            block,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ),
        HeightmapKind::OceanFloor | HeightmapKind::OceanFloorWg | HeightmapKind::MotionBlocking => {
            motion_blocking_block(block)
        }
        HeightmapKind::MotionBlockingNoLeaves => {
            motion_blocking_block(block) && !block.ends_with("_leaves")
        }
    }
}

fn motion_blocking_block(block: &str) -> bool {
    !matches!(
        block,
        "minecraft:air"
            | "minecraft:cave_air"
            | "minecraft:void_air"
            | "minecraft:water"
            | "minecraft:lava"
            | "minecraft:snow"
    )
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

        let mut changed = false;
        if let Some(section) = self.sections.iter_mut().find(|s| s.y == section_y) {
            if let Ok(mut container) =
                PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
            {
                container.set_entry(index, entry);
                section.block_states = container.to_nbt();
                changed = true;
            }
        }
        if changed {
            self.update_heightmaps_after_block_change(
                local_x as i32,
                world_y,
                local_z as i32,
                block_name,
            );
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

pub fn chunk_pyramid_direct_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
) -> Option<Vec<&'static str>> {
    chunk_pyramid_dependencies(kind, target, false)
}

pub fn chunk_pyramid_accumulated_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
) -> Option<Vec<&'static str>> {
    chunk_pyramid_dependencies(kind, target, true)
}

pub fn chunk_generation_task_worst_case_radius(target: &str) -> Option<i32> {
    chunk_pyramid_accumulated_radius_of(ChunkPyramidKind::Generation, target, "minecraft:empty")
}

pub fn chunk_generation_task_layer_radius(
    target: &str,
    status: &str,
    needs_generation: bool,
) -> Option<i32> {
    let kind = if needs_generation {
        ChunkPyramidKind::Generation
    } else {
        ChunkPyramidKind::Loading
    };
    chunk_pyramid_accumulated_radius_of(kind, target, status)
}

pub fn chunk_generation_task_can_load_without_generation<F>(
    target: &str,
    center_x: i32,
    center_z: i32,
    mut persisted_status_at: F,
) -> Option<bool>
where
    F: FnMut(i32, i32) -> Option<&'static str>,
{
    let target = chunk_status(target)?;
    if target.id == "minecraft:empty" {
        return Some(true);
    }

    let center_status = chunk_status(persisted_status_at(center_x, center_z)?)?;
    if chunk_status_is_before(center_status.id, target.id)? {
        return Some(false);
    }

    let dependencies =
        chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Loading, target.id)?;
    let range = dependencies.len().saturating_sub(1) as i32;

    for x in center_x - range..=center_x + range {
        for z in center_z - range..=center_z + range {
            let distance = (center_x - x).abs().max((center_z - z).abs()) as usize;
            let required_status = dependencies.get(distance).copied()?;
            let persisted_status = chunk_status(persisted_status_at(x, z)?)?;
            if chunk_status_is_before(persisted_status.id, required_status)? {
                return Some(false);
            }
        }
    }

    Some(true)
}

pub fn chunk_generation_task_next_layer(
    scheduled_status: Option<&str>,
    needs_generation: bool,
    can_load_without_generation: bool,
) -> Option<ChunkGenerationLayerPlan> {
    match scheduled_status {
        None => Some(ChunkGenerationLayerPlan {
            status: "minecraft:empty",
            needs_generation,
        }),
        Some(status) => {
            let status = chunk_status(status)?;
            if !needs_generation && status.id == "minecraft:empty" && !can_load_without_generation {
                return Some(ChunkGenerationLayerPlan {
                    status: "minecraft:empty",
                    needs_generation: true,
                });
            }

            let next_status = CHUNK_STATUS_PIPELINE.get(status.index + 1)?;
            Some(ChunkGenerationLayerPlan {
                status: next_status.id,
                needs_generation,
            })
        }
    }
}

pub fn chunk_generation_task_chunk_step(
    status: &str,
    persisted_status: Option<&str>,
    needs_generation: bool,
) -> Option<ChunkGenerationChunkStepPlan> {
    let status = chunk_status(status)?;
    let generate = persisted_status
        .and_then(|persisted_status| chunk_status_is_after(status.id, persisted_status))
        .unwrap_or(false);

    if generate && !needs_generation {
        return Some(ChunkGenerationChunkStepPlan::UnexpectedGeneration);
    }

    let pyramid = if generate {
        ChunkPyramidKind::Generation
    } else {
        ChunkPyramidKind::Loading
    };

    Some(ChunkGenerationChunkStepPlan::Apply { pyramid, generate })
}

pub fn chunk_generation_task_wait_for_scheduled_layer(
    scheduled_layer: &[ChunkGenerationFutureState],
) -> ChunkGenerationWaitPlan {
    let mut remaining_layer = scheduled_layer.to_vec();
    let mut marked_for_cancellation = false;

    while let Some(result_now) = remaining_layer.last().copied() {
        match result_now {
            ChunkGenerationFutureState::Pending => {
                return ChunkGenerationWaitPlan {
                    waiting_for_index: Some(remaining_layer.len() - 1),
                    remaining_layer,
                    marked_for_cancellation,
                };
            }
            ChunkGenerationFutureState::Success => {
                remaining_layer.pop();
            }
            ChunkGenerationFutureState::Failure => {
                remaining_layer.pop();
                marked_for_cancellation = true;
            }
        }
    }

    ChunkGenerationWaitPlan {
        waiting_for_index: None,
        remaining_layer,
        marked_for_cancellation,
    }
}

pub fn chunk_generation_task_schedule_layer_positions<I>(
    target: &str,
    status: &str,
    needs_generation: bool,
    center_x: i32,
    center_z: i32,
    mut outcomes: I,
) -> Option<ChunkGenerationScheduleLayerPlan>
where
    I: Iterator<Item = bool>,
{
    let radius = chunk_generation_task_layer_radius(target, status, needs_generation)?;
    let mut visited_positions = Vec::new();
    let mut stopped_early = false;

    for x in center_x - radius..=center_x + radius {
        for z in center_z - radius..=center_z + radius {
            match outcomes.next() {
                Some(true) => visited_positions.push((x, z)),
                Some(false) => {
                    visited_positions.push((x, z));
                    stopped_early = true;
                    return Some(ChunkGenerationScheduleLayerPlan {
                        radius,
                        visited_positions,
                        stopped_early,
                    });
                }
                None => {
                    stopped_early = true;
                    return Some(ChunkGenerationScheduleLayerPlan {
                        radius,
                        visited_positions,
                        stopped_early,
                    });
                }
            }
        }
    }

    Some(ChunkGenerationScheduleLayerPlan {
        radius,
        visited_positions,
        stopped_early,
    })
}

pub fn chunk_generation_task_run_until_wait_decision(
    target: &str,
    scheduled_status: Option<&str>,
    needs_generation: bool,
    marked_for_cancellation: bool,
    can_load_without_generation: bool,
    scheduled_layer: &[ChunkGenerationFutureState],
) -> Option<ChunkGenerationRunPlan> {
    let target = chunk_status(target)?;
    let wait_plan = chunk_generation_task_wait_for_scheduled_layer(scheduled_layer);
    if let Some(waiting_for_index) = wait_plan.waiting_for_index {
        return Some(ChunkGenerationRunPlan::Waiting {
            waiting_for_index,
            remaining_layer: wait_plan.remaining_layer,
            marked_for_cancellation: wait_plan.marked_for_cancellation,
        });
    }

    let marked_for_cancellation = marked_for_cancellation || wait_plan.marked_for_cancellation;
    let target_reached = scheduled_status
        .and_then(chunk_status)
        .is_some_and(|scheduled| scheduled.id == target.id);
    if marked_for_cancellation || target_reached {
        return Some(ChunkGenerationRunPlan::Released);
    }

    chunk_generation_task_next_layer(
        scheduled_status,
        needs_generation,
        can_load_without_generation,
    )
    .map(ChunkGenerationRunPlan::Schedule)
}

fn chunk_pyramid_dependencies(
    kind: ChunkPyramidKind,
    target: &str,
    accumulated: bool,
) -> Option<Vec<&'static str>> {
    let target = chunk_status(target)?;
    let mut accumulated_by_status: Vec<Vec<&'static str>> = Vec::new();

    for status in CHUNK_STATUS_PIPELINE.iter().take(target.index + 1) {
        let direct = chunk_pyramid_direct_dependencies_for_status(kind, status)?;
        if !accumulated {
            accumulated_by_status.push(direct);
            continue;
        }

        if status.index == 0 {
            accumulated_by_status.push(direct);
            continue;
        }

        let parent_accumulated = accumulated_by_status.get(status.index - 1)?;
        let parent_id = CHUNK_STATUS_PIPELINE.get(status.index - 1)?.id;
        let parent_radius = direct
            .iter()
            .rposition(|dependency| chunk_status_is_or_after(dependency, parent_id) == Some(true))
            .unwrap_or(0);
        let len = direct.len().max(parent_radius + parent_accumulated.len());
        let mut combined = Vec::with_capacity(len);

        for distance in 0..len {
            let distance_in_parent = distance as isize - parent_radius as isize;
            let dependency = if distance_in_parent < 0
                || distance_in_parent as usize >= parent_accumulated.len()
            {
                direct[distance]
            } else if distance >= direct.len() {
                parent_accumulated[distance_in_parent as usize]
            } else {
                chunk_status_max(
                    direct[distance],
                    parent_accumulated[distance_in_parent as usize],
                )?
            };
            combined.push(dependency);
        }

        accumulated_by_status.push(combined);
    }

    accumulated_by_status.get(target.index).cloned()
}

fn chunk_pyramid_accumulated_radius_of(
    kind: ChunkPyramidKind,
    target: &str,
    dependency: &str,
) -> Option<i32> {
    if chunk_status(target)?.id == chunk_status(dependency)?.id {
        return Some(0);
    }

    let dependencies = chunk_pyramid_accumulated_dependencies(kind, target)?;
    chunk_dependencies_radius_of(&dependencies, dependency)
}

fn chunk_dependencies_radius_of(dependencies: &[&'static str], dependency: &str) -> Option<i32> {
    let dependency = chunk_status(dependency)?;
    if dependencies.is_empty() {
        return None;
    }

    let first = chunk_status(dependencies[0])?;
    if dependency.index > first.index {
        return None;
    }

    let mut radius_by_dependency = vec![0_i32; first.index + 1];
    for (radius, status_id) in dependencies.iter().enumerate() {
        let status = chunk_status(status_id)?;
        for status_index in 0..=status.index {
            radius_by_dependency[status_index] = radius as i32;
        }
    }

    radius_by_dependency.get(dependency.index).copied()
}

fn chunk_pyramid_direct_dependencies_for_status(
    kind: ChunkPyramidKind,
    status: &ChunkStatusEntry,
) -> Option<Vec<&'static str>> {
    if status.index == 0 {
        return Some(Vec::new());
    }

    let mut dependencies = vec![status.parent];
    for requirement in chunk_pyramid_direct_requirements(kind, status) {
        let required = chunk_status(requirement.status)?;
        if required.index >= status.index {
            return None;
        }

        let new_len = requirement.radius as usize + 1;
        if new_len > dependencies.len() {
            dependencies.resize(new_len, required.id);
        }

        let update_len = new_len.min(dependencies.len());
        for dependency in dependencies.iter_mut().take(update_len) {
            *dependency = chunk_status_max(dependency, required.id)?;
        }
    }

    Some(dependencies)
}

fn chunk_pyramid_direct_requirements(
    kind: ChunkPyramidKind,
    status: &ChunkStatusEntry,
) -> &'static [ChunkStatusRequirement] {
    match kind {
        ChunkPyramidKind::Generation => status.requirements,
        ChunkPyramidKind::Loading => match status.id {
            "minecraft:light" => INITIALIZE_LIGHT_DISTANCE_1_REQUIREMENT,
            _ => NO_REQUIREMENTS,
        },
    }
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

fn optional_compound_tag(compound: &[(String, Tag)], name: &str) -> Option<Tag> {
    match optional_field(compound, name) {
        Some(Tag::Compound(values)) => Some(Tag::Compound(values.clone())),
        Some(_) | None => None,
    }
}

fn optional_byte_array(compound: &[(String, Tag)], name: &str) -> Result<Option<Vec<i8>>, String> {
    match compound.iter().find(|(field_name, _)| field_name == name) {
        Some((_name, Tag::ByteArray(values))) => Ok(Some(values.clone())),
        Some((_name, _)) => Err(format!("NBT field {name} must be a byte array")),
        None => Ok(None),
    }
}

fn optional_light_array(compound: &[(String, Tag)], name: &str) -> Result<Option<Vec<i8>>, String> {
    optional_byte_array(compound, name)?
        .map(|values| {
            if values.len() == LIGHT_DATA_LAYER_LENGTH {
                Ok(values)
            } else {
                Err(format!(
                    "DataLayer should be 2048 bytes not: {} for NBT field {name}",
                    values.len()
                ))
            }
        })
        .transpose()
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

fn below_zero_retrogen_target_status(tag: &Tag) -> Option<&'static str> {
    let compound = compound(tag).ok()?;
    let target_status = string_field(compound, "target_status").ok()?;
    chunk_status(target_status).map(|status| status.id)
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_status, chunk_status_is_after, chunk_status_is_before, chunk_status_is_or_after,
        chunk_status_is_or_before, chunk_status_list, chunk_status_max, default_biomes_container,
        default_block_states_container, empty_structures_payload, pack_postprocessing_offset,
        saved_tick_tag, string_field, BlockStateEntry, ChunkPyramidKind, ChunkSection,
        ChunkStatusTaskKind, ChunkType, HeightmapKind, LevelChunk, LightLayer, PalettedContainer,
        QueuedSectionLightData, SectionBlockPos, TickPriority, BIOME_SECTION_VOLUME,
        CHUNK_STATUS_PIPELINE, CHUNK_WIDTH, FINAL_HEIGHTMAPS, LIGHT_DATA_LAYER_LENGTH,
        LIGHT_DATA_LAYER_NIBBLE_COUNT, LIGHT_DATA_LAYER_ROW_SIZE, LIGHT_DATA_LAYER_WIDTH,
        SECTION_VOLUME, WORLDGEN_HEIGHTMAPS,
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
    fn level_chunk_promote_to_full_migrates_proto_only_payloads() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        let pig = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:pig".to_string()),
        )]);
        chunk.status = "minecraft:spawn".to_string();
        chunk.entities = vec![pig.clone()];
        chunk.carving_mask = Some(vec![1, 2, 3]);
        chunk.below_zero_retrogen = Some(Tag::Compound(vec![
            (
                "target_status".to_string(),
                Tag::String("minecraft:carvers".to_string()),
            ),
            ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
        ]));

        let migrated_entities = chunk.promote_to_full_chunk();
        let encoded = chunk.to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &encoded else {
            panic!("chunk should encode as a compound");
        };

        assert_eq!(migrated_entities, vec![pig]);
        assert_eq!(chunk.status, "minecraft:full");
        assert!(chunk.entities.is_empty());
        assert!(chunk.carving_mask.is_none());
        assert!(chunk.below_zero_retrogen.is_none());
        assert!(fields.iter().all(|(name, _)| name != "entities"));
        assert!(fields.iter().all(|(name, _)| name != "carving_mask"));
    }

    #[test]
    fn level_chunk_updates_inhabited_time_like_chunk_access() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });

        chunk.set_inhabited_time(40);
        chunk.increment_inhabited_time(2);

        assert_eq!(chunk.inhabited_time, 42);

        let decoded = LevelChunk::from_nbt(chunk.pos, &chunk.to_nbt(TARGET_DATA_VERSION)).unwrap();

        assert_eq!(decoded.inhabited_time, 42);
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
    fn level_chunk_highest_generated_status_accounts_for_below_zero_retrogen() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.status = "minecraft:noise".to_string();
        chunk.below_zero_retrogen = Some(Tag::Compound(vec![
            (
                "target_status".to_string(),
                Tag::String("minecraft:carvers".to_string()),
            ),
            ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
        ]));

        assert_eq!(chunk.highest_generated_status(), Some("minecraft:carvers"));

        chunk.status = "minecraft:features".to_string();

        assert_eq!(chunk.highest_generated_status(), Some("minecraft:features"));
    }

    #[test]
    fn level_chunk_set_persisted_status_clears_completed_below_zero_retrogen() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.below_zero_retrogen = Some(Tag::Compound(vec![
            (
                "target_status".to_string(),
                Tag::String("minecraft:carvers".to_string()),
            ),
            ("missing_bedrock".to_string(), Tag::LongArray(Vec::new())),
        ]));

        assert!(chunk.set_persisted_status("noise"));
        assert_eq!(chunk.status, "minecraft:noise");
        assert!(chunk.below_zero_retrogen.is_some());
        assert!(chunk.set_persisted_status("minecraft:carvers"));
        assert_eq!(chunk.status, "minecraft:carvers");
        assert!(chunk.below_zero_retrogen.is_none());
        assert!(!chunk.set_persisted_status("minecraft:not_a_status"));
        assert_eq!(chunk.status, "minecraft:carvers");
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
    fn level_chunk_stores_structure_starts_and_nearby_references() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 10, z: -10 });
        let mineshaft = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:mineshaft".to_string()),
        )]);
        let village = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:village".to_string()),
        )]);

        assert!(chunk.set_structure_start_nbt("minecraft:mineshaft", mineshaft));
        assert!(chunk.set_structure_start_nbt("minecraft:mineshaft", village.clone()));
        assert!(!chunk.set_structure_start_nbt("minecraft:bad", Tag::List(Vec::new())));
        assert!(chunk.add_structure_reference("minecraft:village", ChunkPos { x: 18, z: -2 }));
        assert!(chunk.add_structure_reference("minecraft:village", ChunkPos { x: 18, z: -2 }));
        assert!(!chunk.add_structure_reference("minecraft:village", ChunkPos { x: 19, z: -10 }));

        let Tag::Compound(fields) = &chunk.structures else {
            panic!("structures payload should be a compound");
        };
        assert!(matches!(
            fields.iter().find(|(name, _)| name == "starts"),
            Some((_, Tag::Compound(starts)))
                if starts == &vec![("minecraft:mineshaft".to_string(), village)]
        ));
        assert!(matches!(
            fields.iter().find(|(name, _)| name == "References"),
            Some((_, Tag::Compound(references)))
                if references == &vec![(
                    "minecraft:village".to_string(),
                    Tag::LongArray(vec![super::pack_chunk_pos_as_long(ChunkPos { x: 18, z: -2 })])
                )]
        ));
        assert_eq!(
            super::chunk_pos_chessboard_distance(
                ChunkPos { x: 10, z: -10 },
                ChunkPos { x: 18, z: -2 }
            ),
            8
        );
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
    fn level_chunk_light_handoff_plan_matches_vanilla_section_queueing() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.sections = vec![
            ChunkSection {
                y: -1,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: Some(vec![1; LIGHT_DATA_LAYER_LENGTH]),
                sky_light: Some(vec![15; LIGHT_DATA_LAYER_LENGTH]),
            },
            ChunkSection {
                y: 0,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: Some(vec![7; LIGHT_DATA_LAYER_LENGTH]),
            },
            ChunkSection {
                y: 1,
                block_states: Tag::Compound(Vec::new()),
                biomes: Tag::Compound(Vec::new()),
                block_light: None,
                sky_light: None,
            },
        ];

        let overworld_plan = chunk.light_handoff_plan(true);

        assert!(overworld_plan.retain_data);
        assert_eq!(
            overworld_plan.queued_sections,
            vec![
                QueuedSectionLightData {
                    layer: LightLayer::Block,
                    section_y: -1,
                    data: vec![1; LIGHT_DATA_LAYER_LENGTH],
                },
                QueuedSectionLightData {
                    layer: LightLayer::Sky,
                    section_y: -1,
                    data: vec![15; LIGHT_DATA_LAYER_LENGTH],
                },
                QueuedSectionLightData {
                    layer: LightLayer::Sky,
                    section_y: 0,
                    data: vec![7; LIGHT_DATA_LAYER_LENGTH],
                },
            ]
        );

        let nether_plan = chunk.light_handoff_plan(false);

        assert!(nether_plan.retain_data);
        assert_eq!(
            nether_plan.queued_sections,
            vec![QueuedSectionLightData {
                layer: LightLayer::Block,
                section_y: -1,
                data: vec![1; LIGHT_DATA_LAYER_LENGTH],
            }]
        );
        assert!(
            !LevelChunk::empty(ChunkPos { x: 1, z: 1 })
                .light_handoff_plan(true)
                .retain_data
        );
    }

    #[test]
    fn light_data_layer_nibble_indexing_matches_vanilla() {
        assert_eq!(LIGHT_DATA_LAYER_WIDTH, 16);
        assert_eq!(LIGHT_DATA_LAYER_ROW_SIZE, 128);
        assert_eq!(LIGHT_DATA_LAYER_LENGTH, 2048);
        assert_eq!(LIGHT_DATA_LAYER_NIBBLE_COUNT, 4096);

        let index = super::light_data_layer_index(2, 3, 4);
        assert_eq!(index, 0x342);
        assert_eq!(super::light_data_layer_byte_index(index), 0x1a1);
        assert_eq!(super::light_data_layer_nibble_index(index), 0);

        let odd_index = super::light_data_layer_index(3, 3, 4);
        assert_eq!(odd_index, 0x343);
        assert_eq!(super::light_data_layer_byte_index(odd_index), 0x1a1);
        assert_eq!(super::light_data_layer_nibble_index(odd_index), 1);

        assert_eq!(super::light_data_layer_pack_filled(0), 0x00);
        assert_eq!(super::light_data_layer_pack_filled(15), -1);
        assert_eq!(super::light_data_layer_pack_filled(18), 0x22);

        let mut data = vec![0_i8; LIGHT_DATA_LAYER_LENGTH];
        data[0x1a1] = 0xab_u8 as i8;
        assert_eq!(super::light_data_layer_get(Some(&data), 0, 2, 3, 4), 0x0b);
        assert_eq!(super::light_data_layer_get(Some(&data), 0, 3, 3, 4), 0x0a);
        assert_eq!(super::light_data_layer_get(None, 15, 3, 3, 4), 15);
        assert_eq!(super::light_data_layer_get(None, 18, 3, 3, 4), 2);
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
    fn level_chunk_reports_missing_status_heightmaps_to_prime() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.status = "minecraft:full".to_string();
        chunk
            .heightmaps
            .insert("WORLD_SURFACE".to_string(), Tag::LongArray(vec![1]));
        chunk
            .heightmaps
            .insert("OCEAN_FLOOR".to_string(), Tag::LongArray(vec![2]));

        assert_eq!(
            chunk.heightmaps_to_prime(),
            vec![
                HeightmapKind::MotionBlocking,
                HeightmapKind::MotionBlockingNoLeaves,
            ]
        );

        chunk.status = "minecraft:surface".to_string();

        assert_eq!(
            chunk.heightmaps_to_prime(),
            vec![HeightmapKind::OceanFloorWg, HeightmapKind::WorldSurfaceWg,]
        );

        chunk.status = "minecraft:not_a_status".to_string();

        assert!(chunk.heightmaps_to_prime().is_empty());
    }

    #[test]
    fn level_chunk_primes_heightmaps_from_block_sections() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 2, z: -3 });
        chunk.min_section_y = 0;
        chunk.status = "minecraft:full".to_string();
        chunk.sections = vec![
            ChunkSection {
                y: 0,
                block_states: default_block_states_container(),
                biomes: default_biomes_container(),
                block_light: None,
                sky_light: None,
            },
            ChunkSection {
                y: 1,
                block_states: default_block_states_container(),
                biomes: default_biomes_container(),
                block_light: None,
                sky_light: None,
            },
        ];
        let world_x = chunk.pos.x * CHUNK_WIDTH + 3;
        let world_z = chunk.pos.z * CHUNK_WIDTH + 5;
        chunk.set_block_state(world_x, 20, world_z, "minecraft:water");
        chunk.set_block_state(world_x, 10, world_z, "minecraft:oak_leaves");
        chunk.set_block_state(world_x, 4, world_z, "minecraft:stone");

        let column_index = 5 * 16 + 3;

        assert_eq!(
            chunk.compute_heightmap_values(HeightmapKind::WorldSurface)[column_index],
            21
        );
        assert_eq!(
            chunk.compute_heightmap_values(HeightmapKind::MotionBlocking)[column_index],
            11
        );
        assert_eq!(
            chunk.compute_heightmap_values(HeightmapKind::MotionBlockingNoLeaves)[column_index],
            5
        );

        chunk.prime_missing_heightmaps();

        assert!(chunk.heightmaps_to_prime().is_empty());
        assert_eq!(
            chunk
                .heightmaps
                .get("WORLD_SURFACE")
                .and_then(|tag| match tag {
                    Tag::LongArray(values) => Some(values.len()),
                    _ => None,
                }),
            Some(36)
        );
        assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING"));
        assert!(chunk.heightmaps.contains_key("MOTION_BLOCKING_NO_LEAVES"));
    }

    #[test]
    fn level_chunk_updates_existing_heightmaps_after_block_changes() {
        let mut chunk = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
        chunk.min_section_y = 0;
        chunk.sections = vec![ChunkSection {
            y: 0,
            block_states: default_block_states_container(),
            biomes: default_biomes_container(),
            block_light: None,
            sky_light: None,
        }];
        chunk.prime_heightmaps(&[HeightmapKind::WorldSurface]);
        let column_index = 4 * 16 + 2;

        chunk.set_block_state(2, 3, 4, "minecraft:stone");
        let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
            Tag::LongArray(values) => super::unpack_heightmap_values(values),
            _ => panic!("heightmap should be a long array"),
        };
        assert_eq!(values[column_index], 4);

        chunk.set_block_state(2, 8, 4, "minecraft:stone");
        let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
            Tag::LongArray(values) => super::unpack_heightmap_values(values),
            _ => panic!("heightmap should be a long array"),
        };
        assert_eq!(values[column_index], 9);

        chunk.set_block_state(2, 8, 4, "minecraft:air");
        let values = match chunk.heightmaps.get("WORLD_SURFACE").unwrap() {
            Tag::LongArray(values) => super::unpack_heightmap_values(values),
            _ => panic!("heightmap should be a long array"),
        };
        assert_eq!(values[column_index], 4);
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
    fn level_chunk_defaults_wrong_type_optional_compounds_on_load() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, structures) = fields
            .iter_mut()
            .find(|(name, _)| name == "structures")
            .expect("structures should be present");
        *structures = Tag::List(Vec::new());
        let (_, sections) = fields
            .iter_mut()
            .find(|(name, _)| name == "sections")
            .expect("sections should be present");
        *sections = Tag::List(vec![Tag::Compound(vec![
            ("Y".to_string(), Tag::Byte(0)),
            (
                "block_states".to_string(),
                Tag::String("wrong-type".to_string()),
            ),
            ("biomes".to_string(), Tag::Int(7)),
        ])]);

        let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

        assert_eq!(decoded.structures, empty_structures_payload());
        assert_eq!(
            decoded.sections[0].block_states,
            default_block_states_container()
        );
        assert_eq!(decoded.sections[0].biomes, default_biomes_container());
    }

    #[test]
    fn level_chunk_load_normalizes_structure_payloads() {
        let pos = ChunkPos { x: 10, z: -10 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, structures) = fields
            .iter_mut()
            .find(|(name, _)| name == "structures")
            .expect("structures should be present");
        *structures = Tag::Compound(vec![
            (
                "starts".to_string(),
                Tag::Compound(vec![
                    (
                        "minecraft:village".to_string(),
                        Tag::Compound(vec![("id".to_string(), Tag::String("village".to_string()))]),
                    ),
                    ("minecraft:bad".to_string(), Tag::List(Vec::new())),
                ]),
            ),
            (
                "References".to_string(),
                Tag::Compound(vec![
                    (
                        "minecraft:village".to_string(),
                        Tag::LongArray(vec![
                            super::pack_chunk_pos_as_long(ChunkPos { x: 18, z: -2 }),
                            super::pack_chunk_pos_as_long(ChunkPos { x: 19, z: -10 }),
                        ]),
                    ),
                    ("minecraft:bad".to_string(), Tag::String("bad".to_string())),
                ]),
            ),
        ]);

        let decoded = LevelChunk::from_nbt(pos, &tag).unwrap();

        assert_eq!(
            decoded.structures,
            Tag::Compound(vec![
                (
                    "starts".to_string(),
                    Tag::Compound(vec![(
                        "minecraft:village".to_string(),
                        Tag::Compound(vec![("id".to_string(), Tag::String("village".to_string()))])
                    )])
                ),
                (
                    "References".to_string(),
                    Tag::Compound(vec![(
                        "minecraft:village".to_string(),
                        Tag::LongArray(vec![super::pack_chunk_pos_as_long(ChunkPos {
                            x: 18,
                            z: -2
                        })])
                    )])
                ),
            ])
        );
        assert_eq!(
            super::unpack_chunk_pos_from_long(super::pack_chunk_pos_as_long(ChunkPos {
                x: -1,
                z: -2
            })),
            ChunkPos { x: -1, z: -2 }
        );
    }

    #[test]
    fn level_chunk_rejects_invalid_section_light_arrays_on_load() {
        let pos = ChunkPos { x: 0, z: 0 };
        let mut tag = LevelChunk::empty(pos).to_nbt(TARGET_DATA_VERSION);
        let Tag::Compound(fields) = &mut tag else {
            panic!("chunk should encode as a compound");
        };
        let (_, sections) = fields
            .iter_mut()
            .find(|(name, _)| name == "sections")
            .expect("sections should be present");
        *sections = Tag::List(vec![Tag::Compound(vec![
            ("Y".to_string(), Tag::Byte(0)),
            ("BlockLight".to_string(), Tag::ByteArray(vec![0; 17])),
        ])]);

        let err = LevelChunk::from_nbt(pos, &tag).unwrap_err();

        assert!(err.contains("DataLayer should be 2048 bytes not: 17"));
        assert!(err.contains("BlockLight"));
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
        assert_eq!(chunk_status_is_after("features", "carvers"), Some(true));
        assert_eq!(chunk_status_is_after("carvers", "carvers"), Some(false));
        assert_eq!(chunk_status_is_before("noise", "features"), Some(true));
        assert_eq!(chunk_status_is_before("features", "features"), Some(false));
        assert_eq!(
            chunk_status_is_or_before("features", "features"),
            Some(true)
        );
        assert_eq!(chunk_status_is_or_before("full", "spawn"), Some(false));
        assert_eq!(
            chunk_status_max("noise", "features"),
            Some("minecraft:features")
        );
        assert_eq!(chunk_status_max("full", "spawn"), Some("minecraft:full"));
        assert_eq!(chunk_status_max("bad", "spawn"), None);
        assert_eq!(
            chunk_status_list(),
            vec![
                "minecraft:empty",
                "minecraft:structure_starts",
                "minecraft:structure_references",
                "minecraft:biomes",
                "minecraft:noise",
                "minecraft:surface",
                "minecraft:carvers",
                "minecraft:features",
                "minecraft:initialize_light",
                "minecraft:light",
                "minecraft:spawn",
                "minecraft:full",
            ]
        );

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

    #[test]
    fn chunk_pyramid_dependencies_match_generation_and_loading_radii() {
        assert_eq!(
            super::chunk_pyramid_direct_dependencies(ChunkPyramidKind::Generation, "features"),
            Some(vec![
                "minecraft:carvers",
                "minecraft:carvers",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
            ])
        );
        assert_eq!(
            super::chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Generation, "features"),
            Some(vec![
                "minecraft:carvers",
                "minecraft:carvers",
                "minecraft:biomes",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
                "minecraft:structure_starts",
            ])
        );
        assert_eq!(
            super::chunk_pyramid_accumulated_dependencies(ChunkPyramidKind::Loading, "features"),
            Some(vec!["minecraft:carvers"])
        );

        assert_eq!(
            super::chunk_generation_task_worst_case_radius("full"),
            Some(11)
        );
        assert_eq!(
            super::chunk_generation_task_layer_radius("full", "structure_starts", true),
            Some(11)
        );
        assert_eq!(
            super::chunk_generation_task_layer_radius("full", "features", true),
            Some(1)
        );
        assert_eq!(
            super::chunk_generation_task_layer_radius("full", "features", false),
            Some(1)
        );
        assert_eq!(
            super::chunk_generation_task_layer_radius("light", "initialize_light", false),
            Some(1)
        );
        assert_eq!(
            super::chunk_generation_task_layer_radius("light", "empty", false),
            Some(1)
        );
        assert_eq!(
            super::chunk_generation_task_worst_case_radius("unknown"),
            None
        );
    }

    #[test]
    fn chunk_generation_task_can_load_without_generation_matches_java_gate() {
        assert_eq!(
            super::chunk_generation_task_can_load_without_generation("empty", 0, 0, |_, _| None),
            Some(true)
        );
        assert_eq!(
            super::chunk_generation_task_can_load_without_generation("features", 0, 0, |x, z| {
                if x == 0 && z == 0 {
                    Some("minecraft:carvers")
                } else {
                    Some("minecraft:features")
                }
            }),
            Some(false)
        );
        assert_eq!(
            super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
                if x == 0 && z == 0 {
                    Some("minecraft:light")
                } else {
                    Some("minecraft:initialize_light")
                }
            }),
            Some(true)
        );
        assert_eq!(
            super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
                if x == 0 && z == 0 {
                    Some("minecraft:light")
                } else if x == 1 && z == 0 {
                    Some("minecraft:features")
                } else {
                    Some("minecraft:initialize_light")
                }
            }),
            Some(false)
        );
        assert_eq!(
            super::chunk_generation_task_can_load_without_generation("light", 0, 0, |x, z| {
                if x == 0 && z == 0 {
                    Some("minecraft:light")
                } else if x == 1 && z == 0 {
                    None
                } else {
                    Some("minecraft:initialize_light")
                }
            }),
            None
        );
    }

    #[test]
    fn chunk_generation_task_next_layer_matches_java_schedule_next_layer() {
        assert_eq!(
            super::chunk_generation_task_next_layer(None, false, false),
            Some(super::ChunkGenerationLayerPlan {
                status: "minecraft:empty",
                needs_generation: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("minecraft:empty"), false, false),
            Some(super::ChunkGenerationLayerPlan {
                status: "minecraft:empty",
                needs_generation: true,
            })
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("minecraft:empty"), false, true),
            Some(super::ChunkGenerationLayerPlan {
                status: "minecraft:structure_starts",
                needs_generation: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("minecraft:empty"), true, false),
            Some(super::ChunkGenerationLayerPlan {
                status: "minecraft:structure_starts",
                needs_generation: true,
            })
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("minecraft:carvers"), true, true),
            Some(super::ChunkGenerationLayerPlan {
                status: "minecraft:features",
                needs_generation: true,
            })
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("minecraft:full"), false, true),
            None
        );
        assert_eq!(
            super::chunk_generation_task_next_layer(Some("missing"), false, true),
            None
        );
    }

    #[test]
    fn chunk_generation_task_chunk_step_matches_java_schedule_chunk_in_layer_gate() {
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", Some("carvers"), true),
            Some(super::ChunkGenerationChunkStepPlan::Apply {
                pyramid: ChunkPyramidKind::Generation,
                generate: true,
            })
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", Some("carvers"), false),
            Some(super::ChunkGenerationChunkStepPlan::UnexpectedGeneration)
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", Some("features"), false),
            Some(super::ChunkGenerationChunkStepPlan::Apply {
                pyramid: ChunkPyramidKind::Loading,
                generate: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", Some("full"), true),
            Some(super::ChunkGenerationChunkStepPlan::Apply {
                pyramid: ChunkPyramidKind::Loading,
                generate: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", None, true),
            Some(super::ChunkGenerationChunkStepPlan::Apply {
                pyramid: ChunkPyramidKind::Loading,
                generate: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("missing", Some("features"), true),
            None
        );
        assert_eq!(
            super::chunk_generation_task_chunk_step("minecraft:features", Some("missing"), true),
            Some(super::ChunkGenerationChunkStepPlan::Apply {
                pyramid: ChunkPyramidKind::Loading,
                generate: false,
            })
        );
    }

    #[test]
    fn chunk_generation_task_wait_for_scheduled_layer_matches_java_stack_polling() {
        use super::ChunkGenerationFutureState::{Failure, Pending, Success};

        assert_eq!(
            super::chunk_generation_task_wait_for_scheduled_layer(&[]),
            super::ChunkGenerationWaitPlan {
                waiting_for_index: None,
                remaining_layer: vec![],
                marked_for_cancellation: false,
            }
        );
        assert_eq!(
            super::chunk_generation_task_wait_for_scheduled_layer(&[Success, Pending, Success]),
            super::ChunkGenerationWaitPlan {
                waiting_for_index: Some(1),
                remaining_layer: vec![Success, Pending],
                marked_for_cancellation: false,
            }
        );
        assert_eq!(
            super::chunk_generation_task_wait_for_scheduled_layer(&[Pending, Failure, Success]),
            super::ChunkGenerationWaitPlan {
                waiting_for_index: Some(0),
                remaining_layer: vec![Pending],
                marked_for_cancellation: true,
            }
        );
        assert_eq!(
            super::chunk_generation_task_wait_for_scheduled_layer(&[Success, Failure]),
            super::ChunkGenerationWaitPlan {
                waiting_for_index: None,
                remaining_layer: vec![],
                marked_for_cancellation: true,
            }
        );
    }

    #[test]
    fn chunk_generation_task_schedule_layer_positions_match_java_traversal() {
        assert_eq!(
            super::chunk_generation_task_schedule_layer_positions(
                "minecraft:light",
                "minecraft:initialize_light",
                false,
                5,
                -2,
                std::iter::repeat(true),
            ),
            Some(super::ChunkGenerationScheduleLayerPlan {
                radius: 1,
                visited_positions: vec![
                    (4, -3),
                    (4, -2),
                    (4, -1),
                    (5, -3),
                    (5, -2),
                    (5, -1),
                    (6, -3),
                    (6, -2),
                    (6, -1),
                ],
                stopped_early: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_schedule_layer_positions(
                "minecraft:light",
                "minecraft:initialize_light",
                false,
                0,
                0,
                [true, true, false, true].into_iter(),
            ),
            Some(super::ChunkGenerationScheduleLayerPlan {
                radius: 1,
                visited_positions: vec![(-1, -1), (-1, 0), (-1, 1)],
                stopped_early: true,
            })
        );
        assert_eq!(
            super::chunk_generation_task_schedule_layer_positions(
                "missing",
                "minecraft:empty",
                false,
                0,
                0,
                std::iter::repeat(true),
            ),
            None
        );
    }

    #[test]
    fn chunk_generation_task_run_until_wait_decision_matches_java_ordering() {
        use super::ChunkGenerationFutureState::{Failure, Pending, Success};

        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "minecraft:features",
                Some("minecraft:features"),
                false,
                false,
                true,
                &[Pending],
            ),
            Some(super::ChunkGenerationRunPlan::Waiting {
                waiting_for_index: 0,
                remaining_layer: vec![Pending],
                marked_for_cancellation: false,
            })
        );
        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "minecraft:features",
                Some("minecraft:features"),
                false,
                false,
                true,
                &[Success],
            ),
            Some(super::ChunkGenerationRunPlan::Released)
        );
        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "minecraft:features",
                Some("minecraft:carvers"),
                false,
                false,
                true,
                &[Failure],
            ),
            Some(super::ChunkGenerationRunPlan::Released)
        );
        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "minecraft:features",
                Some("minecraft:empty"),
                false,
                false,
                false,
                &[],
            ),
            Some(super::ChunkGenerationRunPlan::Schedule(
                super::ChunkGenerationLayerPlan {
                    status: "minecraft:empty",
                    needs_generation: true,
                }
            ))
        );
        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "minecraft:features",
                None,
                false,
                false,
                true,
                &[],
            ),
            Some(super::ChunkGenerationRunPlan::Schedule(
                super::ChunkGenerationLayerPlan {
                    status: "minecraft:empty",
                    needs_generation: false,
                }
            ))
        );
        assert_eq!(
            super::chunk_generation_task_run_until_wait_decision(
                "missing",
                None,
                false,
                false,
                true,
                &[],
            ),
            None
        );
    }
}
