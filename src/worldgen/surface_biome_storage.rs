pub(super) fn populate_noise_chunk_biomes(
    chunk: &mut crate::storage::chunk::LevelChunk,
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    seed: i64,
    noise_router: NoiseRouter,
) {
    let debug_enabled = std::env::var_os("RUSTCRAFT_WORLDGEN_BIOME_DEBUG").is_some();
    let total_started = debug_enabled.then(Instant::now);
    let climate_started = debug_enabled.then(Instant::now);
    let climate_sampler = ClimateSampler::from_noise_router(&noise_router, seed, *settings);
    let climate_ms = climate_started.map(|started| started.elapsed().as_millis());
    let chunk_quart_x = chunk.pos.x * 4;
    let chunk_quart_z = chunk.pos.z * 4;
    let cache_started = debug_enabled.then(Instant::now);
    let overworld_2d_climate = overworld_biome_2d_climate_cache(
        biome_source_model,
        settings,
        &climate_sampler,
        chunk_quart_x,
        chunk_quart_z,
    );
    let cache_ms = cache_started.map(|started| started.elapsed().as_millis());
    let mut biome_tags: HashMap<&'static str, Tag> = HashMap::new();
    let overworld_column_biomes =
        overworld_column_biomes_from_climate(overworld_2d_climate.as_ref());
    let population = NoiseChunkBiomePopulation {
        biome_source_model,
        climate_sampler: &climate_sampler,
        overworld_2d_climate: overworld_2d_climate.as_ref(),
        overworld_column_biomes,
        chunk_quart_x,
        chunk_quart_z,
    };

    let fill_started = debug_enabled.then(Instant::now);
    let mut selections = 0_usize;
    for section in &mut chunk.sections {
        selections += populate_noise_chunk_biome_section(section, &population, &mut biome_tags);
    }
    log_biome_storage_debug(BiomeStorageDebugReport {
        enabled: debug_enabled,
        total_started,
        climate_ms,
        cache_ms,
        fill_started,
        sections: chunk.sections.len(),
        selections,
        tags: biome_tags.len(),
    });
}

#[derive(Clone, Copy)]
struct OverworldBiome2dClimate {
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth_offset: f64,
    weirdness: f32,
}

struct NoiseChunkBiomePopulation<'a> {
    biome_source_model: &'a BiomeSourceModel,
    climate_sampler: &'a ClimateSampler,
    overworld_2d_climate: Option<&'a [OverworldBiome2dClimate; 16]>,
    overworld_column_biomes: Option<[&'static str; 16]>,
    chunk_quart_x: i32,
    chunk_quart_z: i32,
}

impl NoiseChunkBiomePopulation<'_> {
    fn biome_at(
        &self,
        local_x: usize,
        local_y: usize,
        local_z: usize,
        section_quart_y: i32,
    ) -> &'static str {
        if let Some(column_biomes) = &self.overworld_column_biomes {
            return column_biomes[local_z * 4 + local_x];
        }

        let quart_x = self.chunk_quart_x + local_x as i32;
        let quart_y = section_quart_y + local_y as i32;
        let quart_z = self.chunk_quart_z + local_z as i32;
        self.overworld_2d_climate
            .and_then(|cache| {
                let cached = cache[local_z * 4 + local_x];
                let block_y = quart_y * 4;
                let depth = cached.depth_offset + OVERWORLD_DEPTH_GRADIENT_DENSITY.compute(block_y);
                let climate = climate_target(
                    cached.temperature,
                    cached.humidity,
                    cached.continentalness,
                    cached.erosion,
                    depth as f32,
                    cached.weirdness,
                );
                select_biome_from_source(
                    self.biome_source_model,
                    quart_x,
                    quart_y,
                    quart_z,
                    climate,
                    0.0,
                )
            })
            .or_else(|| {
                get_biome(
                    self.biome_source_model,
                    quart_x,
                    quart_y,
                    quart_z,
                    self.climate_sampler,
                )
            })
            .unwrap_or("minecraft:plains")
    }
}

fn overworld_column_biomes_from_climate(
    climate_cache: Option<&[OverworldBiome2dClimate; 16]>,
) -> Option<[&'static str; 16]> {
    climate_cache.map(|cache| {
        let mut biomes = ["minecraft:plains"; 16];
        for local_z in 0..4_usize {
            for local_x in 0..4_usize {
                let cached = cache[local_z * 4 + local_x];
                let climate = climate_target(
                    cached.temperature,
                    cached.humidity,
                    cached.continentalness,
                    cached.erosion,
                    0.0,
                    cached.weirdness,
                );
                biomes[local_z * 4 + local_x] =
                    select_climate_biome(overworld_biome_parameters(), climate)
                        .unwrap_or("minecraft:plains");
            }
        }
        biomes
    })
}

fn populate_noise_chunk_biome_section(
    section: &mut crate::storage::chunk::ChunkSection,
    population: &NoiseChunkBiomePopulation<'_>,
    biome_tags: &mut HashMap<&'static str, Tag>,
) -> usize {
    let section_quart_y = i32::from(section.y) * 4;
    let mut palette_names: Vec<&'static str> = Vec::with_capacity(2);
    let mut indices = vec![0_u64; BIOME_SECTION_VOLUME];

    for local_y in 0..4_usize {
        for local_z in 0..4_usize {
            for local_x in 0..4_usize {
                let biome = population.biome_at(local_x, local_y, local_z, section_quart_y);
                let index = local_y * 16 + local_z * 4 + local_x;
                indices[index] = biome_palette_index(&mut palette_names, biome) as u64;
            }
        }
    }

    let palette = biome_palette_tags(&palette_names, biome_tags);
    section.biomes = biome_section_palette_nbt(palette, &indices);
    BIOME_SECTION_VOLUME
}

fn biome_palette_index(palette_names: &mut Vec<&'static str>, biome: &'static str) -> usize {
    match palette_names
        .iter()
        .position(|candidate| *candidate == biome)
    {
        Some(index) => index,
        None => {
            palette_names.push(biome);
            palette_names.len() - 1
        }
    }
}

fn biome_palette_tags(
    palette_names: &[&'static str],
    biome_tags: &mut HashMap<&'static str, Tag>,
) -> Vec<Tag> {
    palette_names
        .iter()
        .map(|biome| {
            biome_tags
                .entry(*biome)
                .or_insert_with(|| Tag::String((*biome).to_string()))
                .clone()
        })
        .collect()
}

fn biome_section_palette_nbt(palette: Vec<Tag>, indices: &[u64]) -> Tag {
    if palette.len() == 1 {
        return PalettedContainer::single(palette[0].clone(), BIOME_SECTION_VOLUME).to_nbt();
    }
    PalettedContainer {
        data: Some(pack_palette_indices(
            indices,
            palette_bits_for_size(palette.len()),
        )),
        palette,
        expected_entries: BIOME_SECTION_VOLUME,
    }
    .to_nbt()
}

struct BiomeStorageDebugReport {
    enabled: bool,
    total_started: Option<Instant>,
    climate_ms: Option<u128>,
    cache_ms: Option<u128>,
    fill_started: Option<Instant>,
    sections: usize,
    selections: usize,
    tags: usize,
}

fn log_biome_storage_debug(report: BiomeStorageDebugReport) {
    if !report.enabled {
        return;
    }
    eprintln!(
        "[biome-storage-debug] total={}ms climate={}ms cache={}ms fill={}ms sections={} selections={} tags={}",
        report
            .total_started
            .map(|started| started.elapsed().as_millis())
            .unwrap_or(0),
        report.climate_ms.unwrap_or(0),
        report.cache_ms.unwrap_or(0),
        report
            .fill_started
            .map(|started| started.elapsed().as_millis())
            .unwrap_or(0),
        report.sections,
        report.selections,
        report.tags
    );
}

fn overworld_biome_2d_climate_cache(
    biome_source_model: &BiomeSourceModel,
    settings: &NoiseGeneratorSettings,
    climate_sampler: &ClimateSampler,
    chunk_quart_x: i32,
    chunk_quart_z: i32,
) -> Option<[OverworldBiome2dClimate; 16]> {
    if !matches!(
        biome_source_model,
        BiomeSourceModel::MultiNoisePreset {
            preset: "minecraft:overworld"
        }
    ) || !matches!(
        settings.id,
        "minecraft:overworld" | "minecraft:large_biomes" | "minecraft:amplified"
    ) {
        return None;
    }

    let mut cache = [OverworldBiome2dClimate {
        temperature: 0.0,
        humidity: 0.0,
        continentalness: 0.0,
        erosion: 0.0,
        depth_offset: 0.0,
        weirdness: 0.0,
    }; 16];
    let y0 = 0;
    let depth_gradient_at_y0 = OVERWORLD_DEPTH_GRADIENT_DENSITY.compute(y0);
    for local_z in 0..4_usize {
        for local_x in 0..4_usize {
            let quart_x = chunk_quart_x + local_x as i32;
            let quart_z = chunk_quart_z + local_z as i32;
            let block_x = quart_x * 4;
            let block_z = quart_z * 4;
            let eval = |function: DensityFunction| {
                function.compute_with_noise(
                    climate_sampler.seed,
                    climate_sampler.settings,
                    block_x,
                    y0,
                    block_z,
                )
            };
            cache[local_z * 4 + local_x] = OverworldBiome2dClimate {
                temperature: eval(climate_sampler.temperature) as f32,
                humidity: eval(climate_sampler.humidity) as f32,
                continentalness: eval(climate_sampler.continentalness) as f32,
                erosion: eval(climate_sampler.erosion) as f32,
                depth_offset: eval(climate_sampler.depth) - depth_gradient_at_y0,
                weirdness: eval(climate_sampler.weirdness) as f32,
            };
        }
    }
    Some(cache)
}
