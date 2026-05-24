use super::*;

pub(super) fn build_underground_ore_decoration_context_chunks(
    target_pos: ChunkPos,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) -> HashMap<ChunkPos, LightweightTreeContextChunk> {
    let router_id = noise_router_id_for_settings(*settings);
    let noise_router = builtin_noise_router(router_id)
        .map(|entry| entry.router)
        .unwrap_or(NONE_NOISE_ROUTER);
    let mut context_positions = Vec::with_capacity(8);
    for z in target_pos.z - 1..=target_pos.z + 1 {
        for x in target_pos.x - 1..=target_pos.x + 1 {
            let pos = ChunkPos { x, z };
            if pos == target_pos {
                continue;
            }
            context_positions.push(pos);
        }
    }
    build_lightweight_tree_context_chunks(&context_positions, settings, seed, noise_router)
        .into_iter()
        .collect()
}

pub(super) fn biome_steps_share_decoration_step_features(
    biome_steps: &[&'static [&'static [&'static str]]],
    step: GenerationDecorationStep,
) -> bool {
    let Some((first, rest)) = biome_steps.split_first() else {
        return false;
    };
    let step_index = step as usize;
    let first_features = first.get(step_index).copied().unwrap_or(&[]);
    rest.iter()
        .all(|steps| steps.get(step_index).copied().unwrap_or(&[]) == first_features)
}

pub(super) fn possible_biome_feature_steps_for_decoration_region(
    center_pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
) -> Vec<&'static [&'static [&'static str]]> {
    let mut steps = Vec::new();
    let mut seen = Vec::new();
    let sample_quart_y = ((settings.sea_level + 1).clamp(
        settings.noise.min_y,
        settings.noise.min_y + settings.noise.height - 1,
    )) >> 2;

    for chunk_z in center_pos.z - 1..=center_pos.z + 1 {
        for chunk_x in center_pos.x - 1..=center_pos.x + 1 {
            let chunk_quart_x = chunk_x * 4;
            let chunk_quart_z = chunk_z * 4;
            for local_z in 0..4 {
                for local_x in 0..4 {
                    let Some(biome) = get_biome(
                        biome_source_model,
                        chunk_quart_x + local_x,
                        sample_quart_y,
                        chunk_quart_z + local_z,
                        climate_sampler,
                    ) else {
                        continue;
                    };
                    if seen.contains(&biome) {
                        continue;
                    }
                    if let Some(generation) = biome_generation_settings(biome) {
                        seen.push(biome);
                        steps.push(generation.feature_steps);
                    }
                }
            }
        }
    }

    if steps.is_empty() {
        possible_biome_feature_steps_for_source(biome_source_model)
    } else {
        steps
    }
}

pub(super) fn decoration_region_biome_steps_from_generated_chunk(
    chunk: &LevelChunk,
    biome_source_model: &BiomeSourceModel,
) -> Vec<&'static [&'static [&'static str]]> {
    let mut steps = Vec::new();
    let mut seen = Vec::new();
    for section in &chunk.sections {
        let Ok(biomes) = PalettedContainer::from_nbt(&section.biomes, BIOME_SECTION_VOLUME) else {
            continue;
        };
        for entry in biomes.palette {
            let Tag::String(biome) = entry else {
                continue;
            };
            if seen.iter().any(|candidate| candidate == &biome) {
                continue;
            }
            if let Some(generation) = biome_generation_settings(&biome) {
                seen.push(biome);
                steps.push(generation.feature_steps);
            }
        }
    }
    if steps.is_empty() {
        possible_biome_feature_steps_for_source(biome_source_model)
    } else {
        steps
    }
}

pub(super) fn decoration_region_biome_steps_for_chunk(
    pos: ChunkPos,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
) -> Vec<&'static [&'static [&'static str]]> {
    with_noise_snapshot_cache(|| {
        let Some(router) =
            builtin_noise_router(noise_router_id_for_settings(*settings)).map(|entry| entry.router)
        else {
            return Vec::new();
        };
        let climate_sampler = ClimateSampler::from_noise_router(&router, seed, *settings);
        possible_biome_feature_steps_for_decoration_region(
            pos,
            biome_source_model,
            settings,
            &climate_sampler,
        )
    })
}

pub(super) struct OreBlockCache {
    chunk_pos: ChunkPos,
    min_section_y: i32,
    ocean_floor_wg: [i32; 16 * 16],
    sections: Vec<OreSectionCache>,
    region_chunks: HashMap<ChunkPos, OreRegionChunkCache>,
    block_write_center: ChunkPos,
    block_write_radius: i32,
    read_context: HashMap<ChunkPos, LightweightTreeContextChunk>,
    block_state_entries: HashMap<&'static str, Tag>,
    palette_indices: HashMap<(ChunkPos, i8, &'static str), usize>,
}

struct OreRegionChunkCache {
    ocean_floor_wg: [i32; 16 * 16],
    sections: Vec<OreSectionCache>,
}

struct OreSectionCache {
    y: i8,
    palette: Vec<Tag>,
    palette_names: Vec<Option<String>>,
    indices: Vec<u64>,
    dirty: bool,
}

impl OreBlockCache {
    pub(super) fn from_chunk(chunk: &LevelChunk) -> Self {
        let mut ocean_floor_wg = [chunk.min_section_y * 16; 16 * 16];
        for z in 0..16 {
            for x in 0..16 {
                ocean_floor_wg[z * 16 + x] = chunk
                    .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                    .unwrap_or(chunk.min_section_y * 16);
            }
        }
        Self {
            chunk_pos: chunk.pos,
            min_section_y: chunk.min_section_y,
            ocean_floor_wg,
            sections: ore_section_caches_from_chunk(chunk),
            region_chunks: HashMap::new(),
            block_write_center: chunk.pos,
            block_write_radius: 0,
            read_context: HashMap::new(),
            block_state_entries: HashMap::new(),
            palette_indices: HashMap::new(),
        }
    }

    pub(super) fn from_chunk_with_read_context(
        chunk: &LevelChunk,
        context_chunks: &HashMap<ChunkPos, LightweightTreeContextChunk>,
    ) -> Self {
        let mut cache = Self::from_chunk(chunk);
        cache.read_context = context_chunks
            .iter()
            .map(|(pos, context)| (*pos, context.clone()))
            .collect();
        cache
    }

    pub(super) fn from_region_chunks(
        center_pos: ChunkPos,
        chunks: &BTreeMap<ChunkPos, LevelChunk>,
    ) -> Option<Self> {
        let center = chunks.get(&center_pos)?;
        let mut cache = Self::from_chunk(center);
        cache.block_write_center = center_pos;
        cache.block_write_radius = 1;
        for (pos, chunk) in chunks {
            if *pos == center_pos {
                continue;
            }
            cache.region_chunks.insert(
                *pos,
                OreRegionChunkCache {
                    ocean_floor_wg: ore_ocean_floor_wg_from_chunk(chunk),
                    sections: ore_section_caches_from_chunk(chunk),
                },
            );
        }
        Some(cache)
    }

    pub(super) fn block_state_name(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<&str> {
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if chunk_pos != self.chunk_pos {
            if let Some(region_chunk) = self.region_chunks.get(&chunk_pos) {
                return Self::block_state_name_from_sections(
                    &region_chunk.sections,
                    self.min_section_y,
                    world_x,
                    world_y,
                    world_z,
                );
            }
            return self
                .read_context
                .get(&chunk_pos)
                .map(|chunk| chunk.synthetic_block_state(world_x, world_y, world_z));
        }
        Self::block_state_name_from_sections(
            &self.sections,
            self.min_section_y,
            world_x,
            world_y,
            world_z,
        )
    }

    fn block_state_name_from_sections(
        sections: &[OreSectionCache],
        min_section_y: i32,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) -> Option<&str> {
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let section = Self::section_from_sections(sections, min_section_y, section_y)?;
        let palette_index = *section.indices.get(index)? as usize;
        section
            .palette_names
            .get(palette_index)
            .and_then(|name| name.as_deref())
    }

    pub(super) fn heightmap_value_by_scan(&self, heightmap: HeightmapKind, world_x: i32, world_z: i32) -> i32 {
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if chunk_pos != self.chunk_pos && !self.read_context.contains_key(&chunk_pos) {
            return self.min_section_y * 16;
        }
        let min_y = self.min_section_y * 16;
        let max_y = self.min_section_y * 16 + 383;
        for y in (min_y..=max_y).rev() {
            let block = self
                .block_state_name(world_x, y, world_z)
                .unwrap_or("minecraft:air");
            if heightmap_opaque(heightmap, block) {
                return y + 1;
            }
        }
        min_y
    }

    pub(super) fn ocean_floor_wg_height(&self, world_x: i32, world_z: i32) -> Option<i32> {
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_z * 16 + local_x;
        if chunk_pos == self.chunk_pos {
            return self.ocean_floor_wg.get(index).copied();
        }
        if let Some(region_chunk) = self.region_chunks.get(&chunk_pos) {
            return region_chunk.ocean_floor_wg.get(index).copied();
        }
        self.read_context.get(&chunk_pos).map(|chunk| {
            chunk
                .terrain_heights
                .local_height(HeightmapKind::OceanFloorWg, local_x, local_z)
        })
    }

    pub(super) fn set_block_state(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block_name: &'static str,
    ) {
        if !self.can_write_world_xz(world_x, world_z) {
            return;
        }
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        let section_y = world_y.div_euclid(16) as i8;
        let local_x = world_x.rem_euclid(16) as usize;
        let local_y = world_y.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let cache_key = (chunk_pos, section_y, block_name);
        let cached_palette_index = self.palette_indices.get(&cache_key).copied();
        let entry = if cached_palette_index.is_none() {
            Some(
                self.block_state_entries
                    .entry(block_name)
                    .or_insert_with(|| {
                        Tag::Compound(vec![(
                            "Name".to_string(),
                            Tag::String(block_name.to_string()),
                        )])
                    })
                    .clone(),
            )
        } else {
            None
        };
        if let Some(section) = self.section_mut_for_chunk(chunk_pos, section_y) {
            let palette_index = cached_palette_index.unwrap_or_else(|| {
                let entry = entry.as_ref().expect("entry exists when uncached");
                let palette_index = section
                    .palette
                    .iter()
                    .position(|candidate| candidate == entry)
                    .unwrap_or_else(|| {
                        section.palette.push(entry.clone());
                        section.palette_names.push(Some(block_name.to_string()));
                        section.palette.len() - 1
                    });
                if section.palette_names.len() < section.palette.len() {
                    section
                        .palette_names
                        .push(paletted_block_name(entry).map(str::to_string));
                }
                palette_index
            });
            if let Some(current) = section.indices.get_mut(index) {
                *current = palette_index as u64;
                section.dirty = true;
            }
            self.palette_indices.insert(cache_key, palette_index);
        }
    }

    pub(super) fn flush_to_chunk(self, chunk: &mut LevelChunk) {
        Self::flush_sections_to_chunk(self.sections, chunk);
    }

    pub(super) fn flush_to_chunks(self, chunks: &mut BTreeMap<ChunkPos, LevelChunk>) {
        if let Some(chunk) = chunks.get_mut(&self.chunk_pos) {
            Self::flush_sections_to_chunk(self.sections, chunk);
        }
        for (pos, region_chunk) in self.region_chunks {
            if let Some(chunk) = chunks.get_mut(&pos) {
                Self::flush_sections_to_chunk(region_chunk.sections, chunk);
            }
        }
    }

    fn flush_sections_to_chunk(sections: Vec<OreSectionCache>, chunk: &mut LevelChunk) {
        for section_cache in sections {
            if !section_cache.dirty {
                continue;
            }
            if let Some(section) = chunk
                .sections
                .iter_mut()
                .find(|section| section.y == section_cache.y)
            {
                let bits = palette_bits_for_size(section_cache.palette.len());
                let data = if section_cache.palette.len() == 1 {
                    None
                } else {
                    Some(pack_palette_indices(&section_cache.indices, bits))
                };
                section.block_states = PalettedContainer {
                    palette: section_cache.palette,
                    data,
                    expected_entries: SECTION_VOLUME,
                }
                .to_nbt();
            }
        }
    }

    fn section(&self, section_y: i8) -> Option<&OreSectionCache> {
        Self::section_from_sections(&self.sections, self.min_section_y, section_y)
    }

    fn section_from_sections(
        sections: &[OreSectionCache],
        min_section_y: i32,
        section_y: i8,
    ) -> Option<&OreSectionCache> {
        let section_index = i32::from(section_y) - min_section_y;
        if section_index >= 0
            && sections
                .get(section_index as usize)
                .is_some_and(|section| section.y == section_y)
        {
            return sections.get(section_index as usize);
        }
        sections.iter().find(|section| section.y == section_y)
    }

    fn section_mut_for_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        section_y: i8,
    ) -> Option<&mut OreSectionCache> {
        if chunk_pos == self.chunk_pos {
            return Self::section_mut_from_sections(
                &mut self.sections,
                self.min_section_y,
                section_y,
            );
        }
        self.region_chunks.get_mut(&chunk_pos).and_then(|chunk| {
            Self::section_mut_from_sections(&mut chunk.sections, self.min_section_y, section_y)
        })
    }

    fn section_mut_from_sections(
        sections: &mut [OreSectionCache],
        min_section_y: i32,
        section_y: i8,
    ) -> Option<&mut OreSectionCache> {
        let section_index = i32::from(section_y) - min_section_y;
        if section_index >= 0
            && sections
                .get(section_index as usize)
                .is_some_and(|section| section.y == section_y)
        {
            return sections.get_mut(section_index as usize);
        }
        sections.iter_mut().find(|section| section.y == section_y)
    }

    fn contains_world_xz(&self, world_x: i32, world_z: i32) -> bool {
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        chunk_pos == self.chunk_pos || self.region_chunks.contains_key(&chunk_pos)
    }

    pub(super) fn can_write_world_xz(&self, world_x: i32, world_z: i32) -> bool {
        let chunk_pos = ChunkPos {
            x: world_x.div_euclid(16),
            z: world_z.div_euclid(16),
        };
        if !self.contains_world_xz(world_x, world_z) {
            return false;
        }
        (chunk_pos.x - self.block_write_center.x)
            .abs()
            .max((chunk_pos.z - self.block_write_center.z).abs())
            <= self.block_write_radius
    }
}

fn ore_ocean_floor_wg_from_chunk(chunk: &LevelChunk) -> [i32; 16 * 16] {
    let mut ocean_floor_wg = [chunk.min_section_y * 16; 16 * 16];
    for z in 0..16 {
        for x in 0..16 {
            ocean_floor_wg[z * 16 + x] = chunk
                .heightmap_value(HeightmapKind::OceanFloorWg, x, z)
                .unwrap_or(chunk.min_section_y * 16);
        }
    }
    ocean_floor_wg
}

fn ore_section_caches_from_chunk(chunk: &LevelChunk) -> Vec<OreSectionCache> {
    chunk
        .sections
        .iter()
        .filter_map(|section| {
            let container =
                PalettedContainer::from_nbt(&section.block_states, SECTION_VOLUME).ok()?;
            let bits = palette_bits_for_size(container.palette.len());
            let indices = container
                .data
                .as_ref()
                .map(|data| unpack_palette_indices(data, bits, SECTION_VOLUME))
                .unwrap_or_else(|| vec![0_u64; SECTION_VOLUME]);
            let palette_names = container
                .palette
                .iter()
                .map(|tag| paletted_block_name(tag).map(str::to_string))
                .collect();
            Some(OreSectionCache {
                y: section.y,
                palette: container.palette,
                palette_names,
                indices,
                dirty: false,
            })
        })
        .collect()
}

fn paletted_block_name(tag: &Tag) -> Option<&str> {
    let Tag::Compound(fields) = tag else {
        return None;
    };
    fields.iter().find_map(|(name, value)| {
        (name == "Name").then_some(value).and_then(|value| {
            if let Tag::String(block_name) = value {
                Some(block_name.as_str())
            } else {
                None
            }
        })
    })
}
