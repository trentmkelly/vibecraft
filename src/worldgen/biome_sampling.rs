use super::*;

/// Samples climate parameters from density functions at quart-block coordinates.
///
/// Mirrors Java's `Climate.Sampler` record. The six density functions correspond to
/// `NoiseRouter` fields: temperature, vegetation (= humidity), continents
/// (= continentalness), erosion, depth, and ridges (= weirdness).
///
/// `sample()` converts quart coords to block coords (`coord * 4`), evaluates each
/// function, casts to `f32`, and calls `climate_target()` — exactly matching Java.
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/Climate.java
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClimateSampler {
    pub temperature: DensityFunction,
    pub humidity: DensityFunction,
    pub continentalness: DensityFunction,
    pub erosion: DensityFunction,
    pub depth: DensityFunction,
    pub weirdness: DensityFunction,
    pub seed: i64,
    pub biome_zoom_seed: i64,
    pub settings: NoiseGeneratorSettings,
}

impl ClimateSampler {
    /// Constructs a sampler from a `NoiseRouter` by mapping router fields to the six
    /// climate dimensions. Field mapping:
    /// - `temperature` → `router.temperature`
    /// - `humidity`    → `router.vegetation`
    /// - `continentalness` → `router.continents`
    /// - `erosion`     → `router.erosion`
    /// - `depth`       → `router.depth`
    /// - `weirdness`   → `router.ridges`
    pub fn from_noise_router(
        router: &NoiseRouter,
        seed: i64,
        settings: NoiseGeneratorSettings,
    ) -> Self {
        Self {
            temperature: router.temperature,
            humidity: router.vegetation,
            continentalness: router.continents,
            erosion: router.erosion,
            depth: router.depth,
            weirdness: router.ridges,
            seed,
            biome_zoom_seed: biome_manager_obfuscate_seed(seed),
            settings,
        }
    }

    /// Samples climate at quart-block coordinates. Converts quart → block (`× 4`),
    /// evaluates each density function, casts to `f32`, and calls `climate_target()`.
    /// Mirrors Java's `Sampler.sample(quartX, quartY, quartZ)`.
    pub fn sample(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> ClimateTarget {
        let bx = quart_x * 4;
        let by = quart_y * 4;
        let bz = quart_z * 4;
        let eval =
            |f: DensityFunction| f.compute_with_noise(self.seed, self.settings, bx, by, bz) as f32;
        climate_target(
            eval(self.temperature),
            eval(self.humidity),
            eval(self.continentalness),
            eval(self.erosion),
            eval(self.depth),
            eval(self.weirdness),
        )
    }
}

/// Returns the biome at quart-block coordinates for any biome source type.
///
/// Mirrors Java's `BiomeSource.getNoiseBiome(quartX, quartY, quartZ, sampler)`:
/// - `Fixed`: always the fixed biome.
/// - `Checkerboard`: index from `(quartX >> (scale+2)) + (quartZ >> (scale+2))`.
/// - `MultiNoisePreset`: samples climate via the sampler, finds nearest preset entry.
/// - `TheEnd`: evaluates the sampler's erosion function at the chunk-centre offset
///   position used by `TheEndBiomeSource`, as per Java.
///
/// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/…BiomeSource.java
pub fn get_biome(
    source: &BiomeSourceModel,
    quart_x: i32,
    quart_y: i32,
    quart_z: i32,
    sampler: &ClimateSampler,
) -> Option<&'static str> {
    match source {
        BiomeSourceModel::Fixed { biome } => Some(*biome),
        BiomeSourceModel::Checkerboard { biomes, scale } => {
            let bit_shift = scale + 2;
            let index =
                ((quart_x >> bit_shift) + (quart_z >> bit_shift)).rem_euclid(biomes.len() as i32);
            Some(biomes[index as usize])
        }
        BiomeSourceModel::MultiNoisePreset { .. } => {
            let climate = sampler.sample(quart_x, quart_y, quart_z);
            select_biome_from_source(source, quart_x, quart_y, quart_z, climate, 0.0)
        }
        BiomeSourceModel::TheEnd => {
            // Java uses the erosion density function evaluated at the *chunk-centre offset*
            // position: weirdBlockX = (chunkX * 2 + 1) * 8, weirdBlockZ = (chunkZ * 2 + 1) * 8.
            // select_end_biome handles the central island check itself, so we only need
            // to compute the erosion at the offset coords for the outer end.
            let block_x = quart_x * 4;
            let block_y = quart_y * 4;
            let block_z = quart_z * 4;
            let chunk_x = block_x.div_euclid(16);
            let chunk_z = block_z.div_euclid(16);
            let weird_block_x = (chunk_x * 2 + 1) * 8;
            let weird_block_z = (chunk_z * 2 + 1) * 8;
            let erosion_value = sampler.erosion.compute_with_noise(
                sampler.seed,
                sampler.settings,
                weird_block_x,
                block_y,
                weird_block_z,
            );
            Some(select_end_biome(quart_x, quart_y, quart_z, erosion_value))
        }
    }
}

pub(super) fn biome_manager_obfuscate_seed(seed: i64) -> i64 {
    let digest = Sha256::digest(seed.to_le_bytes());
    i64::from_le_bytes([
        digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
    ])
}

pub(super) fn biome_manager_lcg_next(value: i64, salt: i64) -> i64 {
    value
        .wrapping_mul(
            value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407),
        )
        .wrapping_add(salt)
}

pub(super) fn biome_manager_fiddle(value: i64) -> f64 {
    // Java: floorMod(value >> 24, 1024).  Since the modulus is 2^10, the
    // low-bit mask is equivalent for two's-complement signed integers and
    // avoids a hot signed remainder in BiomeManager's 8-corner search.
    let uniform = (((value >> 24) & 1023) as f64) / 1024.0;
    (uniform - 0.5) * 0.9
}

pub(super) fn biome_manager_fiddled_distance(
    seed: i64,
    random_x: i32,
    random_y: i32,
    random_z: i32,
    distance_x: f64,
    distance_y: f64,
    distance_z: f64,
) -> f64 {
    let mut value = seed;
    value = biome_manager_lcg_next(value, i64::from(random_x));
    value = biome_manager_lcg_next(value, i64::from(random_y));
    value = biome_manager_lcg_next(value, i64::from(random_z));
    value = biome_manager_lcg_next(value, i64::from(random_x));
    value = biome_manager_lcg_next(value, i64::from(random_y));
    value = biome_manager_lcg_next(value, i64::from(random_z));
    let fiddle_x = biome_manager_fiddle(value);
    value = biome_manager_lcg_next(value, seed);
    let fiddle_y = biome_manager_fiddle(value);
    value = biome_manager_lcg_next(value, seed);
    let fiddle_z = biome_manager_fiddle(value);
    let dx = distance_x + fiddle_x;
    let dy = distance_y + fiddle_y;
    let dz = distance_z + fiddle_z;
    dz * dz + dy * dy + dx * dx
}

pub(super) fn biome_manager_get_biome(
    source: &BiomeSourceModel,
    biome_zoom_seed: i64,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    sampler: &ClimateSampler,
) -> Option<&'static str> {
    let absolute_x = block_x - 2;
    let absolute_y = block_y - 2;
    let absolute_z = block_z - 2;
    let parent_x = absolute_x >> 2;
    let parent_y = absolute_y >> 2;
    let parent_z = absolute_z >> 2;
    let fract_x = f64::from(absolute_x & 3) / 4.0;
    let fract_y = f64::from(absolute_y & 3) / 4.0;
    let fract_z = f64::from(absolute_z & 3) / 4.0;

    let mut nearest_corner = 0;
    let mut nearest_distance = f64::INFINITY;
    for corner in 0..8 {
        let x_even = (corner & 4) == 0;
        let y_even = (corner & 2) == 0;
        let z_even = (corner & 1) == 0;
        let corner_x = if x_even { parent_x } else { parent_x + 1 };
        let corner_y = if y_even { parent_y } else { parent_y + 1 };
        let corner_z = if z_even { parent_z } else { parent_z + 1 };
        let distance_x = if x_even { fract_x } else { fract_x - 1.0 };
        let distance_y = if y_even { fract_y } else { fract_y - 1.0 };
        let distance_z = if z_even { fract_z } else { fract_z - 1.0 };
        let distance = biome_manager_fiddled_distance(
            biome_zoom_seed,
            corner_x,
            corner_y,
            corner_z,
            distance_x,
            distance_y,
            distance_z,
        );
        if nearest_distance > distance {
            nearest_corner = corner;
            nearest_distance = distance;
        }
    }

    let biome_x = if (nearest_corner & 4) == 0 {
        parent_x
    } else {
        parent_x + 1
    };
    let biome_y = if (nearest_corner & 2) == 0 {
        parent_y
    } else {
        parent_y + 1
    };
    let biome_z = if (nearest_corner & 1) == 0 {
        parent_z
    } else {
        parent_z + 1
    };
    get_biome(source, biome_x, biome_y, biome_z, sampler)
}

pub(super) fn biome_manager_get_biome_cached(
    source: &BiomeSourceModel,
    biome_zoom_seed: i64,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    sampler: &ClimateSampler,
    chunk_biomes: Option<&ChunkNoiseBiomeCache>,
    noise_biome_cache: &mut HashMap<(i32, i32, i32), &'static str>,
) -> Option<&'static str> {
    let absolute_x = block_x - 2;
    let absolute_y = block_y - 2;
    let absolute_z = block_z - 2;
    let parent_x = absolute_x >> 2;
    let parent_y = absolute_y >> 2;
    let parent_z = absolute_z >> 2;
    let fract_x = f64::from(absolute_x & 3) / 4.0;
    let fract_y = f64::from(absolute_y & 3) / 4.0;
    let fract_z = f64::from(absolute_z & 3) / 4.0;

    let mut nearest_corner = 0;
    let mut nearest_distance = f64::INFINITY;
    for corner in 0..8 {
        let x_even = (corner & 4) == 0;
        let y_even = (corner & 2) == 0;
        let z_even = (corner & 1) == 0;
        let corner_x = if x_even { parent_x } else { parent_x + 1 };
        let corner_y = if y_even { parent_y } else { parent_y + 1 };
        let corner_z = if z_even { parent_z } else { parent_z + 1 };
        let distance_x = if x_even { fract_x } else { fract_x - 1.0 };
        let distance_y = if y_even { fract_y } else { fract_y - 1.0 };
        let distance_z = if z_even { fract_z } else { fract_z - 1.0 };
        let distance = biome_manager_fiddled_distance(
            biome_zoom_seed,
            corner_x,
            corner_y,
            corner_z,
            distance_x,
            distance_y,
            distance_z,
        );
        if nearest_distance > distance {
            nearest_corner = corner;
            nearest_distance = distance;
        }
    }

    let biome_x = if (nearest_corner & 4) == 0 {
        parent_x
    } else {
        parent_x + 1
    };
    let biome_y = if (nearest_corner & 2) == 0 {
        parent_y
    } else {
        parent_y + 1
    };
    let biome_z = if (nearest_corner & 1) == 0 {
        parent_z
    } else {
        parent_z + 1
    };
    if let Some(chunk_biomes) = chunk_biomes {
        if let Some(biome) = chunk_biomes.get(biome_x, biome_y, biome_z) {
            return Some(biome);
        }
    }
    if let Some(biome) = noise_biome_cache.get(&(biome_x, biome_y, biome_z)).copied() {
        return Some(biome);
    }
    let biome = get_biome(source, biome_x, biome_y, biome_z, sampler)?;
    noise_biome_cache.insert((biome_x, biome_y, biome_z), biome);
    Some(biome)
}

pub(super) struct ChunkNoiseBiomeCache {
    chunk_pos: ChunkPos,
    min_section_y: i32,
    max_section_y: i32,
    sections: Vec<[&'static str; BIOME_SECTION_VOLUME]>,
}

impl ChunkNoiseBiomeCache {
    pub(super) fn from_chunk(chunk: &LevelChunk) -> Self {
        let min_section_y = chunk.min_section_y;
        let max_section_y = chunk
            .sections
            .iter()
            .map(|section| i32::from(section.y))
            .max()
            .unwrap_or(min_section_y);
        let mut sections = vec![["minecraft:plains"; BIOME_SECTION_VOLUME]; chunk.sections.len()];
        for section in &chunk.sections {
            let section_index = i32::from(section.y) - min_section_y;
            if section_index < 0 || section_index as usize >= sections.len() {
                continue;
            }
            let Ok(container) = PalettedContainer::from_nbt(&section.biomes, BIOME_SECTION_VOLUME)
            else {
                continue;
            };
            for (index, section_biome) in sections[section_index as usize].iter_mut().enumerate() {
                let Some(Tag::String(biome)) = container.get_entry(index) else {
                    continue;
                };
                *section_biome = static_biome_id(biome).unwrap_or("minecraft:plains");
            }
        }
        Self {
            chunk_pos: chunk.pos,
            min_section_y,
            max_section_y,
            sections,
        }
    }

    pub(super) fn get(&self, quart_x: i32, quart_y: i32, quart_z: i32) -> Option<&'static str> {
        let local_x = quart_x - self.chunk_pos.x * 4;
        let local_z = quart_z - self.chunk_pos.z * 4;
        if !(0..4).contains(&local_x) || !(0..4).contains(&local_z) {
            return None;
        }
        let clamped_quart_y = quart_y.clamp(self.min_section_y * 4, self.max_section_y * 4 + 3);
        let section_y = clamped_quart_y.div_euclid(4);
        let section_index = section_y - self.min_section_y;
        if section_index < 0 || section_index as usize >= self.sections.len() {
            return None;
        }
        let local_y = clamped_quart_y - section_y * 4;
        let index = local_y as usize * 16 + local_z as usize * 4 + local_x as usize;
        Some(self.sections[section_index as usize][index])
    }
}
