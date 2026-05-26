use super::*;

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
        if block_light
            .as_ref()
            .is_some_and(|light| light.len() != LIGHT_DATA_LAYER_LENGTH)
            || sky_light
                .as_ref()
                .is_some_and(|light| light.len() != LIGHT_DATA_LAYER_LENGTH)
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

    pub fn light_section_status_updates(&self) -> Vec<LightSectionStatusUpdate> {
        self.sections
            .iter()
            .map(|section| LightSectionStatusUpdate {
                section_y: section.y,
                has_only_air: section.has_only_air(),
            })
            .collect()
    }

    pub fn initialize_light_plan(&self, lighted: bool) -> ChunkInitializeLightPlan {
        ChunkInitializeLightPlan {
            pre_update_section_statuses: self
                .light_section_status_updates()
                .into_iter()
                .filter(|status| !status.has_only_air)
                .collect(),
            post_update_light_enabled: lighted,
            post_update_retain_data: false,
        }
    }

    pub fn light_completion_plan(&self, lighted: bool) -> ChunkLightCompletionPlan {
        ChunkLightCompletionPlan {
            initial_light_correct: false,
            pre_update_propagate_light_sources: !lighted,
            completed_light_correct: true,
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

    pub fn find_block_light_sources(&self) -> Vec<(i32, i32, i32, String, u8)> {
        let mut sources = Vec::new();
        for section in &self.sections {
            let Ok(container) = PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME)
            else {
                continue;
            };
            let section_has_light = container.palette.iter().any(|entry| {
                block_state_name(entry).is_some_and(|name| block_light_emission(name) != 0)
            });
            if !section_has_light {
                continue;
            }
            for y in 0..SECTION_HEIGHT {
                for z in 0..CHUNK_WIDTH {
                    for x in 0..CHUNK_WIDTH {
                        let index = (y * 256 + z * 16 + x) as usize;
                        let Some(entry) = container.get_entry(index) else {
                            continue;
                        };
                        let Some(name) = block_state_name(entry) else {
                            continue;
                        };
                        let emission = block_light_emission(name);
                        if emission == 0 {
                            continue;
                        }
                        sources.push((
                            self.pos.x * CHUNK_WIDTH + x,
                            i32::from(section.y) * SECTION_HEIGHT + y,
                            self.pos.z * CHUNK_WIDTH + z,
                            name.to_string(),
                            emission,
                        ));
                    }
                }
            }
        }
        sources
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
                    let Some(block) = self.get_block_state_name(
                        self.pos.x * CHUNK_WIDTH + x as i32,
                        y,
                        self.pos.z * CHUNK_WIDTH + z as i32,
                    ) else {
                        continue;
                    };
                    if block != "minecraft:air" && heightmap_block_matches(heightmap, block) {
                        values[index] = y + 1;
                        break;
                    }
                }
            }
        }
        values
    }

    pub(super) fn update_heightmaps_after_block_change(
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
            let first_available = unpack_heightmap_value(raw_values, column_index);
            if world_y <= first_available - 2 {
                continue;
            }
            if heightmap_block_matches(heightmap, block_name) {
                if world_y >= first_available {
                    let mut values = unpack_heightmap_values(raw_values);
                    values[column_index] = world_y + 1;
                    self.heightmaps.insert(
                        heightmap.storage_name().to_string(),
                        Tag::LongArray(pack_heightmap_values(&values)),
                    );
                }
            } else if first_available - 1 == world_y {
                let mut values = unpack_heightmap_values(raw_values);
                values[column_index] = (min_y..world_y)
                    .rev()
                    .find(|y| {
                        self.get_block_state_name(
                            self.pos.x * CHUNK_WIDTH + local_x,
                            *y,
                            self.pos.z * CHUNK_WIDTH + local_z,
                        )
                        .is_some_and(|block| heightmap_block_matches(heightmap, block))
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
